//! Building placement: spiral outward from a centre tile and take the first footprint
//! that fits. Solid buildings keep a one-tile corridor around them so villagers and the
//! army can always get through the base; farms are walkable and pack tightly.

use crate::view::within;
use std::collections::BTreeSet;
use wars_sim::fixed::Vec2;
use wars_sim::ids::{PlayerId, TypeId};
use wars_sim::types::{GameData, GatherKind, ObjectClass};
use wars_sim::world::World;

pub(crate) struct Placer {
    /// Tiles under own farms (the sim does not mark walkable buildings as occupied, so
    /// without this two farms could overlap).
    farm_tiles: BTreeSet<(i32, i32)>,
    /// Resource piles near home that houses should keep clear of.
    piles: Vec<(i32, i32)>,
    /// Tiles with a unit or animal on them: units never block placement in the sim,
    /// so a building could trap a sheep (or a villager) inside its footprint.
    unit_tiles: BTreeSet<(i32, i32)>,
}

impl Placer {
    pub fn new(world: &World, data: &GameData, me: PlayerId, home: Vec2) -> Placer {
        let mut farm_tiles = BTreeSet::new();
        let mut piles = Vec::new();
        let mut unit_tiles = BTreeSet::new();
        for e in world.iter() {
            let def = data.object(e.ty);
            if matches!(def.class, ObjectClass::Unit | ObjectClass::Animal) {
                if e.inside.is_none() && within(e.pos, home, 40) {
                    unit_tiles.insert(e.pos.tile());
                }
            } else if e.owner == me && def.class == ObjectClass::Building && def.walkable {
                let (tx, ty, w, h) = world.footprint(data, e);
                for y in ty..ty + h {
                    for x in tx..tx + w {
                        farm_tiles.insert((x, y));
                    }
                }
            } else if e.owner.is_gaia() && def.class == ObjectClass::Resource {
                let pile = matches!(
                    def.yields.map(|y| y.kind),
                    Some(GatherKind::Gold | GatherKind::Stone | GatherKind::Berries)
                );
                if pile && within(e.pos, home, 30) {
                    piles.push(e.pos.tile());
                }
            }
        }
        Placer {
            farm_tiles,
            piles,
            unit_tiles,
        }
    }

    /// Can `ty` stand with its top-left tile at (x, y)?
    pub fn fits(&self, world: &World, data: &GameData, ty: TypeId, x: i32, y: i32) -> bool {
        let def = data.object(ty);
        let (w, h) = (def.size.0 as i32, def.size.1 as i32);
        let map = &world.map;
        if !map.can_place(x, y, w, h, def.placement) {
            return false;
        }
        for yy in y..y + h {
            for xx in x..x + w {
                if self.farm_tiles.contains(&(xx, yy)) {
                    return false;
                }
                if !def.walkable && self.unit_tiles.contains(&(xx, yy)) {
                    return false;
                }
            }
        }
        if def.walkable {
            return true;
        }
        // Corridor: the ring around a solid building must not touch another building
        // and must have at least one walkable tile so the door is never blocked.
        let mut open = false;
        for yy in y - 1..=y + h {
            for xx in x - 1..=x + w {
                let edge = yy == y - 1 || yy == y + h || xx == x - 1 || xx == x + w;
                if !edge {
                    continue;
                }
                if let Some(o) = map.occupant(xx, yy) {
                    if let Some(oe) = world.get(o) {
                        if data.object(oe.ty).class == ObjectClass::Building {
                            return false;
                        }
                    }
                } else if map.land_passable(xx, yy) && !self.farm_tiles.contains(&(xx, yy)) {
                    open = true;
                }
            }
        }
        open
    }

    /// First fitting top-left tile for `ty` on the square rings `min_r..=max_r` around
    /// `centre`. `rot` rotates the starting point of each ring so repeated calls spread
    /// buildings around instead of stacking them on one side.
    pub fn find(
        &self,
        world: &World,
        data: &GameData,
        ty: TypeId,
        centre: (i32, i32),
        min_r: i32,
        max_r: i32,
        avoid_piles: bool,
        rot: u32,
    ) -> Option<(i32, i32)> {
        let def = data.object(ty);
        let (w, h) = (def.size.0 as i32, def.size.1 as i32);
        let (cx, cy) = centre;
        for r in min_r..=max_r {
            let ring = ring_offsets(r);
            let n = ring.len();
            let start = if n == 0 { 0 } else { (rot as usize) % n };
            for i in 0..n {
                let (dx, dy) = ring[(start + i) % n];
                let x = cx + dx - w / 2;
                let y = cy + dy - h / 2;
                if avoid_piles && self.near_pile(x, y, w, h, 3) {
                    continue;
                }
                if self.fits(world, data, ty, x, y) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn near_pile(&self, x: i32, y: i32, w: i32, h: i32, r: i32) -> bool {
        self.piles
            .iter()
            .any(|&(px, py)| px >= x - r && px < x + w + r && py >= y - r && py < y + h + r)
    }

    /// Record a footprint chosen this think so a second placement does not reuse it.
    pub fn reserve(&mut self, data: &GameData, ty: TypeId, x: i32, y: i32) {
        let def = data.object(ty);
        for yy in y..y + def.size.1 as i32 {
            for xx in x..x + def.size.0 as i32 {
                self.farm_tiles.insert((xx, yy));
            }
        }
    }
}

/// Offsets with Chebyshev distance exactly `r`, clockwise from the east.
fn ring_offsets(r: i32) -> Vec<(i32, i32)> {
    if r <= 0 {
        return vec![(0, 0)];
    }
    let mut v = Vec::with_capacity((8 * r) as usize);
    for dy in -r..=r {
        v.push((r, dy));
    }
    for dx in (-r..r).rev() {
        v.push((dx, r));
    }
    for dy in (-r..r).rev() {
        v.push((-r, dy));
    }
    for dx in -r + 1..r {
        v.push((dx, -r));
    }
    v
}
