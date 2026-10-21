//! The tile map: terrain, elevation and the occupancy grid used by pathfinding and
//! building placement.

use crate::fixed::Vec2;
use crate::ids::{EntityId, Terrain};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub elevation: u8,
    /// Visual variation seed for the renderers.
    pub variant: u8,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            terrain: Terrain::Grass,
            elevation: 0,
            variant: 0,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    /// Entity occupying each tile (buildings, trees, resource piles). Units never block.
    pub occupancy: Vec<Option<EntityId>>,
}

impl Map {
    pub fn new(width: i32, height: i32, terrain: Terrain) -> Map {
        let n = (width * height) as usize;
        Map {
            width,
            height,
            tiles: vec![
                Tile {
                    terrain,
                    elevation: 0,
                    variant: 0
                };
                n
            ],
            occupancy: vec![None; n],
        }
    }
    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }
    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.in_bounds(x, y) {
            Some(&self.tiles[self.idx(x, y)])
        } else {
            None
        }
    }
    pub fn terrain(&self, x: i32, y: i32) -> Terrain {
        self.tile(x, y)
            .map(|t| t.terrain)
            .unwrap_or(Terrain::DeepWater)
    }
    pub fn set_terrain(&mut self, x: i32, y: i32, t: Terrain) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.tiles[i].terrain = t;
        }
    }
    pub fn occupant(&self, x: i32, y: i32) -> Option<EntityId> {
        if self.in_bounds(x, y) {
            self.occupancy[self.idx(x, y)]
        } else {
            None
        }
    }
    pub fn set_occupant(&mut self, x: i32, y: i32, e: Option<EntityId>) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.occupancy[i] = e;
        }
    }
    /// Passable for a land unit: land terrain and not occupied by a solid object.
    pub fn land_passable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.terrain(x, y).land_passable() && self.occupant(x, y).is_none()
    }
    pub fn water_passable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.terrain(x, y).water_passable() && self.occupant(x, y).is_none()
    }
    pub fn passable(&self, x: i32, y: i32, naval: bool) -> bool {
        if naval {
            self.water_passable(x, y)
        } else {
            self.land_passable(x, y)
        }
    }
    /// Terrain-only passability (ignores occupancy), used to check footprints.
    pub fn terrain_passable(&self, x: i32, y: i32, naval: bool) -> bool {
        self.in_bounds(x, y)
            && if naval {
                self.terrain(x, y).water_passable()
            } else {
                self.terrain(x, y).land_passable()
            }
    }
    pub fn clamp(&self, p: Vec2) -> Vec2 {
        use crate::fixed::Fixed;
        Vec2::new(
            p.x.clamp(
                Fixed::ZERO,
                Fixed::from_int(self.width) - Fixed::from_decimal("0.01"),
            ),
            p.y.clamp(
                Fixed::ZERO,
                Fixed::from_int(self.height) - Fixed::from_decimal("0.01"),
            ),
        )
    }
    /// Can a building with footprint (w, h) be placed with its top-left tile at (x, y)?
    pub fn can_place(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        placement: crate::types::Placement,
    ) -> bool {
        use crate::types::Placement;
        let mut has_land = false;
        let mut has_water = false;
        for ty in y..y + h {
            for tx in x..x + w {
                if !self.in_bounds(tx, ty) || self.occupant(tx, ty).is_some() {
                    return false;
                }
                let t = self.terrain(tx, ty);
                match placement {
                    Placement::Land => {
                        if !t.buildable() {
                            return false;
                        }
                    }
                    Placement::Water => {
                        if !t.is_water() {
                            return false;
                        }
                    }
                    Placement::Shore => {
                        if t == Terrain::Forest || t == Terrain::Cliff {
                            return false;
                        }
                        if t.is_water() || t == Terrain::Shallows {
                            has_water = true;
                        } else {
                            has_land = true;
                        }
                    }
                    Placement::Any => {}
                }
            }
        }
        if placement == Placement::Shore {
            // A dock needs water under it and land next to it.
            if !has_water {
                return false;
            }
            if !has_land {
                // check the ring around the footprint for land
                let mut land_adjacent = false;
                for ty in y - 1..=y + h {
                    for tx in x - 1..=x + w {
                        if self.in_bounds(tx, ty) && self.terrain(tx, ty).buildable() {
                            land_adjacent = true;
                        }
                    }
                }
                if !land_adjacent {
                    return false;
                }
            }
        }
        true
    }
    pub fn fill_occupancy(&mut self, x: i32, y: i32, w: i32, h: i32, e: Option<EntityId>) {
        for ty in y..y + h {
            for tx in x..x + w {
                self.set_occupant(tx, ty, e);
            }
        }
    }
    /// Nearest passable tile to (x, y) by ring search, for spawning and un-garrisoning.
    pub fn nearest_passable(&self, x: i32, y: i32, naval: bool, max_r: i32) -> Option<(i32, i32)> {
        if self.passable(x, y, naval) {
            return Some((x, y));
        }
        for r in 1..=max_r {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (tx, ty) = (x + dx, y + dy);
                    if self.passable(tx, ty, naval) {
                        return Some((tx, ty));
                    }
                }
            }
        }
        None
    }
}
