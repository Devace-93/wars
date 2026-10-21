#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::manual_is_multiple_of
)]
//! wars-mapgen: seeded random maps and the scenario file format.
//!
//! Everything here is integer-only so a seed produces the same map on every platform.

pub mod noise;
pub mod scenario;

use serde::{Deserialize, Serialize};
use wars_sim::ids::*;
use wars_sim::map::Map;
use wars_sim::rng::Rng;
use wars_sim::types::GameData;
use wars_sim::world::{GameMode, Settings, World};
use wars_sim::{Fixed, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MapType {
    Arabia,
    BlackForest,
    Islands,
    Arena,
    Highland,
    Coastal,
}

impl MapType {
    pub const ALL: [MapType; 6] = [
        MapType::Arabia,
        MapType::BlackForest,
        MapType::Islands,
        MapType::Arena,
        MapType::Highland,
        MapType::Coastal,
    ];
    pub fn name(self) -> &'static str {
        match self {
            MapType::Arabia => "Arabia",
            MapType::BlackForest => "Black Forest",
            MapType::Islands => "Islands",
            MapType::Arena => "Arena",
            MapType::Highland => "Highland",
            MapType::Coastal => "Coastal",
        }
    }
    pub fn from_name(s: &str) -> Option<MapType> {
        let s = s.to_lowercase().replace([' ', '_'], "");
        MapType::ALL
            .iter()
            .copied()
            .find(|m| m.name().to_lowercase().replace(' ', "") == s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MapSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl MapSize {
    pub const ALL: [MapSize; 5] = [
        MapSize::Tiny,
        MapSize::Small,
        MapSize::Medium,
        MapSize::Large,
        MapSize::Huge,
    ];
    pub fn tiles(self) -> i32 {
        match self {
            MapSize::Tiny => 96,
            MapSize::Small => 120,
            MapSize::Medium => 144,
            MapSize::Large => 168,
            MapSize::Huge => 200,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            MapSize::Tiny => "Tiny",
            MapSize::Small => "Small",
            MapSize::Medium => "Medium",
            MapSize::Large => "Large",
            MapSize::Huge => "Huge",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PlayerSetup {
    pub name: String,
    pub civ: String,
    pub team: u8,
    pub is_ai: bool,
    pub ai_level: u8,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MapParams {
    pub map_type: MapType,
    pub size: MapSize,
    pub seed: u64,
    pub players: Vec<PlayerSetup>,
    pub settings: Settings,
}

impl Default for MapParams {
    fn default() -> Self {
        MapParams {
            map_type: MapType::Arabia,
            size: MapSize::Small,
            seed: 1,
            players: vec![
                PlayerSetup {
                    name: "Player".into(),
                    civ: "britons".into(),
                    team: 1,
                    is_ai: false,
                    ai_level: 1,
                },
                PlayerSetup {
                    name: "Computer".into(),
                    civ: "franks".into(),
                    team: 2,
                    is_ai: true,
                    ai_level: 1,
                },
            ],
            settings: Settings::default(),
        }
    }
}

pub fn starting_resources(mode: &GameMode) -> Stockpile {
    match mode {
        GameMode::RandomMap => Stockpile::new(200, 200, 100, 200),
        GameMode::Deathmatch => Stockpile::new(20000, 20000, 10000, 5000),
        GameMode::Regicide => Stockpile::new(500, 500, 150, 150),
        GameMode::Scenario => Stockpile::new(200, 200, 100, 200),
    }
}

struct Gen<'a> {
    data: &'a GameData,
    rng: Rng,
    w: i32,
    h: i32,
}

/// Generate a complete, playable world.
pub fn generate(data: &GameData, params: &MapParams) -> World {
    let n = params.size.tiles();
    let mut g = Gen {
        data,
        rng: Rng::new(params.seed ^ 0x9e37_79b9_7f4a_7c15),
        w: n,
        h: n,
    };
    let mut map = Map::new(n, n, Terrain::Grass);
    let nplayers = params.players.len().clamp(1, 8);
    let spots = g.player_spots(nplayers, params);
    match params.map_type {
        MapType::Arabia => g.arabia(&mut map),
        MapType::BlackForest => g.black_forest(&mut map, &spots),
        MapType::Islands => g.islands(&mut map, &spots),
        MapType::Arena => g.arena(&mut map, &spots),
        MapType::Highland => g.highland(&mut map, &spots),
        MapType::Coastal => g.coastal(&mut map, &spots),
    }
    // Clear every player's base area so the start layout always fits.
    for &(cx, cy) in &spots {
        g.clear_circle(&mut map, cx, cy, 8, Terrain::Grass);
    }
    let mut world = World::new(map, params.seed, params.settings.clone());
    world.add_player("Gaia", CivId(0), 0, false, Stockpile::ZERO);
    let base = starting_resources(&params.settings.mode);
    for p in params.players.iter().take(8) {
        let civ = data.civ_id(&p.civ).unwrap_or(CivId(0));
        let pid = world.add_player(&p.name, civ, p.team, p.is_ai, base);
        world.player_mut(pid).ai_level = p.ai_level;
        // One-off civ resources (e.g. Persians +50/+50, Chinese -200/-50).
        for e in &data.civ(civ).bonuses {
            if let wars_sim::types::Effect::Resources {
                food,
                wood,
                gold,
                stone,
            } = e
            {
                world
                    .player_mut(pid)
                    .resources
                    .add(&Stockpile::new(*food, *wood, *gold, *stone));
            }
        }
    }
    for (i, &(cx, cy)) in spots.iter().enumerate() {
        g.place_start(&mut world, PlayerId(i as u8 + 1), cx, cy, params);
    }
    g.scatter_gaia(&mut world, &spots, params.map_type);
    g.place_relics(&mut world, &spots);
    // Walls for Arena are placed after the base so they do not collide with it.
    if params.map_type == MapType::Arena {
        for &(cx, cy) in &spots {
            g.ring_walls(&mut world, cx, cy, 13);
        }
    }
    // Starting age techs.
    let start_age = params.settings.starting_age;
    for pid in 1..=spots.len() {
        let pid = PlayerId(pid as u8);
        for (tech, age) in [
            ("feudal_age", Age::Feudal),
            ("castle_age", Age::Castle),
            ("imperial_age", Age::Imperial),
        ] {
            if start_age >= age {
                let t = data.tk(tech);
                wars_sim::unit::grant_tech(&mut world, data, pid, t);
            }
        }
        // Free civ techs apply immediately.
        let civ = world.player(pid).civ;
        for e in data.civ(civ).bonuses.clone() {
            if let wars_sim::types::Effect::FreeTech { tech } = e {
                let t = data.tk(&tech);
                if data.tech(t).age <= world.player(pid).age {
                    wars_sim::unit::grant_tech(&mut world, data, pid, t);
                }
            }
        }
        world.player_mut(pid).age = start_age;
    }
    world.total_relics = world
        .iter()
        .filter(|e| data.object(e.ty).key == "relic")
        .count() as i32;
    wars_sim::vision::update_vision(&mut world, data);
    world.events.clear();
    world
}

impl<'a> Gen<'a> {
    fn player_spots(&mut self, n: usize, params: &MapParams) -> Vec<(i32, i32)> {
        let cx = self.w / 2;
        let cy = self.h / 2;
        let r = (self.w * 37 / 100).max(14);
        let start = self.rng.below(360) as i32;
        // Order players so allies sit next to each other when team_together is on.
        let mut order: Vec<usize> = (0..n).collect();
        if params.settings.team_together {
            order.sort_by_key(|i| (params.players[*i].team, *i));
        }
        let mut spots = vec![(0, 0); n];
        for (slot, &pi) in order.iter().enumerate() {
            let angle = start + (360 * slot as i32) / n as i32;
            let (sx, sy) = noise::circle_point(angle);
            let x = cx + (r as i64 * sx as i64 / 1000) as i32;
            let y = cy + (r as i64 * sy as i64 / 1000) as i32;
            spots[pi] = (x.clamp(12, self.w - 13), y.clamp(12, self.h - 13));
        }
        spots
    }

    fn clear_circle(&mut self, map: &mut Map, cx: i32, cy: i32, r: i32, t: Terrain) {
        for y in cy - r..=cy + r {
            for x in cx - r..=cx + r {
                if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r && map.in_bounds(x, y) {
                    let i = map.idx(x, y);
                    map.tiles[i].terrain = t;
                }
            }
        }
    }

    fn paint_variation(&mut self, map: &mut Map) {
        let seed = self.rng.next_u64();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = map.idx(x, y);
                let v = noise::value_noise(x, y, 6, seed);
                map.tiles[i].variant = (v % 4) as u8;
                if map.tiles[i].terrain == Terrain::Grass {
                    let d = noise::value_noise(x, y, 11, seed ^ 0x55);
                    if d > 200 {
                        map.tiles[i].terrain = Terrain::Dirt;
                    }
                }
            }
        }
    }

    fn forest_blobs(
        &mut self,
        map: &mut Map,
        count: usize,
        min: i32,
        max: i32,
        avoid: &[(i32, i32)],
        avoid_r: i32,
    ) {
        for _ in 0..count {
            let x = self.rng.range(2, self.w - 2);
            let y = self.rng.range(2, self.h - 2);
            if avoid
                .iter()
                .any(|&(ax, ay)| (ax - x) * (ax - x) + (ay - y) * (ay - y) < avoid_r * avoid_r)
            {
                continue;
            }
            let size = self.rng.range(min, max);
            self.grow_blob(
                map,
                x,
                y,
                size,
                Terrain::Forest,
                &[Terrain::Grass, Terrain::Dirt],
            );
        }
    }

    fn grow_blob(
        &mut self,
        map: &mut Map,
        x: i32,
        y: i32,
        size: i32,
        t: Terrain,
        over: &[Terrain],
    ) {
        let mut frontier = vec![(x, y)];
        let mut placed = 0;
        let mut visited = std::collections::BTreeSet::new();
        while placed < size && !frontier.is_empty() {
            let i = self.rng.below(frontier.len() as u32) as usize;
            let (px, py) = frontier.swap_remove(i);
            if !map.in_bounds(px, py) || !visited.insert((px, py)) {
                continue;
            }
            if !over.contains(&map.terrain(px, py)) {
                continue;
            }
            map.set_terrain(px, py, t);
            placed += 1;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                frontier.push((px + dx, py + dy));
            }
        }
    }

    fn arabia(&mut self, map: &mut Map) {
        self.paint_variation(map);
        let n = (self.w * self.h / 450) as usize;
        self.forest_blobs(map, n, 12, 40, &[], 0);
    }

    fn black_forest(&mut self, map: &mut Map, spots: &[(i32, i32)]) {
        for t in map.tiles.iter_mut() {
            t.terrain = Terrain::Forest;
        }
        // Clearings around players and a central clearing, joined by winding paths.
        let cx = self.w / 2;
        let cy = self.h / 2;
        self.clear_circle(map, cx, cy, 10, Terrain::Grass);
        for &(x, y) in spots {
            self.clear_circle(map, x, y, 12, Terrain::Grass);
            self.carve_path(map, x, y, cx, cy, 3);
        }
        // Join neighbouring players so the map is not a star only.
        for i in 0..spots.len() {
            let (ax, ay) = spots[i];
            let (bx, by) = spots[(i + 1) % spots.len()];
            if self.rng.chance(60) {
                self.carve_path(map, ax, ay, bx, by, 2);
            }
        }
        self.paint_variation(map);
    }

    fn carve_path(&mut self, map: &mut Map, mut x: i32, mut y: i32, tx: i32, ty: i32, width: i32) {
        let mut steps = 0;
        while (x != tx || y != ty) && steps < 4000 {
            steps += 1;
            let dx = (tx - x).signum();
            let dy = (ty - y).signum();
            let wobble = self.rng.below(10);
            let (mx, my) = match wobble {
                0 => (dx, 0),
                1 => (0, dy),
                2 => (self.rng.range(-1, 2), 0),
                3 => (0, self.rng.range(-1, 2)),
                _ => {
                    if (tx - x).abs() > (ty - y).abs() {
                        (dx, 0)
                    } else {
                        (0, dy)
                    }
                }
            };
            x = (x + mx).clamp(1, self.w - 2);
            y = (y + my).clamp(1, self.h - 2);
            for oy in -width / 2..=width / 2 {
                for ox in -width / 2..=width / 2 {
                    if map.in_bounds(x + ox, y + oy)
                        && map.terrain(x + ox, y + oy) == Terrain::Forest
                    {
                        map.set_terrain(x + ox, y + oy, Terrain::Grass);
                    }
                }
            }
        }
    }

    fn islands(&mut self, map: &mut Map, spots: &[(i32, i32)]) {
        let seed = self.rng.next_u64();
        let island_r = (self.w / 5).max(14);
        for y in 0..self.h {
            for x in 0..self.w {
                let mut land = false;
                for &(sx, sy) in spots {
                    let d2 = (x - sx) * (x - sx) + (y - sy) * (y - sy);
                    let wobble = noise::value_noise(x, y, 9, seed) as i32 / 16; // 0..16
                    let r = island_r + wobble - 8;
                    if d2 <= r * r {
                        land = true;
                    }
                }
                let i = map.idx(x, y);
                map.tiles[i].terrain = if land { Terrain::Grass } else { Terrain::Water };
            }
        }
        self.shoreline(map);
        self.paint_variation(map);
        let n = (self.w * self.h / 700) as usize;
        self.forest_blobs(map, n, 8, 30, spots, 10);
    }

    fn shoreline(&mut self, map: &mut Map) {
        // Beach on land next to water, deep water far from land.
        let snapshot: Vec<Terrain> = map.tiles.iter().map(|t| t.terrain).collect();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = map.idx(x, y);
                let t = snapshot[i];
                let mut near_land = false;
                let mut near_water = false;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if map.in_bounds(x + dx, y + dy) {
                            let o = snapshot[map.idx(x + dx, y + dy)];
                            if o.is_water() {
                                near_water = true;
                            } else {
                                near_land = true;
                            }
                        }
                    }
                }
                if t == Terrain::Grass && near_water {
                    map.tiles[i].terrain = Terrain::Beach;
                }
                if t == Terrain::Water {
                    let mut far = true;
                    for dy in -3..=3 {
                        for dx in -3..=3 {
                            if map.in_bounds(x + dx, y + dy)
                                && !snapshot[map.idx(x + dx, y + dy)].is_water()
                            {
                                far = false;
                            }
                        }
                    }
                    if far {
                        map.tiles[i].terrain = Terrain::DeepWater;
                    }
                    let _ = near_land;
                }
            }
        }
    }

    fn arena(&mut self, map: &mut Map, spots: &[(i32, i32)]) {
        self.paint_variation(map);
        // A forest ring hugs each walled base; the middle is open.
        for &(x, y) in spots {
            for r in 14..=17 {
                for a in (0..360).step_by(3) {
                    let (sx, sy) = noise::circle_point(a);
                    let px = x + (r * sx) / 1000;
                    let py = y + (r * sy) / 1000;
                    if map.in_bounds(px, py) {
                        map.set_terrain(px, py, Terrain::Forest);
                    }
                }
            }
        }
        let n = (self.w * self.h / 900) as usize;
        self.forest_blobs(map, n, 6, 20, spots, 20);
    }

    fn highland(&mut self, map: &mut Map, spots: &[(i32, i32)]) {
        self.paint_variation(map);
        // A river crossing the map with a few shallow fords.
        let vertical = self.rng.chance(50);
        let mut pos = self.rng.range(self.w / 3, self.w * 2 / 3);
        let fords: Vec<i32> = (0..3)
            .map(|i| self.rng.range(i * self.h / 3 + 4, (i + 1) * self.h / 3 - 4))
            .collect();
        for t in 0..self.h {
            pos = (pos + self.rng.range(-1, 2)).clamp(6, self.w - 7);
            let is_ford = fords.iter().any(|f| (f - t).abs() <= 2);
            for o in -2..=2 {
                let (x, y) = if vertical { (pos + o, t) } else { (t, pos + o) };
                let near_spot = spots
                    .iter()
                    .any(|&(sx, sy)| (sx - x) * (sx - x) + (sy - y) * (sy - y) < 100);
                if map.in_bounds(x, y) && !near_spot {
                    map.set_terrain(
                        x,
                        y,
                        if is_ford {
                            Terrain::Shallows
                        } else {
                            Terrain::Water
                        },
                    );
                }
            }
        }
        // Hills shown as elevation for the renderers (no gameplay effect yet).
        let seed = self.rng.next_u64();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = map.idx(x, y);
                let v = noise::value_noise(x, y, 14, seed);
                map.tiles[i].elevation = (v / 64) as u8;
            }
        }
        let n = (self.w * self.h / 500) as usize;
        self.forest_blobs(map, n, 10, 35, spots, 9);
    }

    fn coastal(&mut self, map: &mut Map, spots: &[(i32, i32)]) {
        self.paint_variation(map);
        // Sea along one edge with a wavy coastline.
        let side = self.rng.below(4);
        let seed = self.rng.next_u64();
        let depth = self.w / 5;
        for y in 0..self.h {
            for x in 0..self.w {
                let (along, across) = match side {
                    0 => (x, y),
                    1 => (x, self.h - 1 - y),
                    2 => (y, x),
                    _ => (y, self.w - 1 - x),
                };
                let wobble = noise::value_noise(along, 0, 10, seed) as i32 / 24;
                if across < depth + wobble - 5 {
                    let i = map.idx(x, y);
                    map.tiles[i].terrain = Terrain::Water;
                }
            }
        }
        // Players near the sea keep their land: push the sea back around bases.
        for &(x, y) in spots {
            self.clear_circle(map, x, y, 11, Terrain::Grass);
        }
        self.shoreline(map);
        let n = (self.w * self.h / 600) as usize;
        self.forest_blobs(map, n, 10, 35, spots, 9);
    }

    /// Place a tile-aligned object of the given key near (x, y), searching outward.
    fn put(
        &mut self,
        world: &mut World,
        owner: PlayerId,
        key: &str,
        x: i32,
        y: i32,
        max_r: i32,
    ) -> Option<EntityId> {
        let ty = self.data.obj(key);
        let def = self.data.object(ty);
        let is_unit = matches!(
            def.class,
            wars_sim::types::ObjectClass::Unit | wars_sim::types::ObjectClass::Animal
        );
        for r in 0..=max_r {
            let mut candidates = Vec::new();
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    candidates.push((x + dx, y + dy));
                }
            }
            // Deterministic shuffle by rotating with the rng.
            if !candidates.is_empty() {
                let k = self.rng.below(candidates.len() as u32) as usize;
                candidates.rotate_left(k);
            }
            for (px, py) in candidates {
                if !world.map.in_bounds(px, py) {
                    continue;
                }
                if is_unit {
                    if !world.map.passable(px, py, def.is_naval) {
                        continue;
                    }
                    if let Some(id) =
                        world.spawn(self.data, owner, ty, Vec2::tile_center(px, py), false)
                    {
                        return Some(id);
                    }
                } else if let Some(id) =
                    world.spawn(self.data, owner, ty, Vec2::from_tile(px, py), false)
                {
                    return Some(id);
                }
            }
        }
        None
    }

    fn place_start(
        &mut self,
        world: &mut World,
        pid: PlayerId,
        cx: i32,
        cy: i32,
        params: &MapParams,
    ) {
        let data = self.data;
        let tc = data.obj("town_center");
        // Town centre centred on the spot (4x4 => top-left at -2).
        world.spawn(data, pid, tc, Vec2::from_tile(cx - 2, cy - 2), false);
        for i in 0..3 {
            self.put(world, pid, "villager", cx - 1 + i, cy + 3, 3);
        }
        let civ = data.civ(world.player(pid).civ);
        if civ.key == "chinese" {
            for i in 0..3 {
                self.put(world, pid, "villager", cx + 2 - i, cy + 4, 3);
            }
        }
        if civ.key == "persians" {
            // Persians start with the same layout; their bonus is resources.
        }
        self.put(world, pid, "scout_cavalry", cx + 4, cy - 4, 3);
        if params.settings.mode == GameMode::Regicide {
            self.put(world, pid, "king", cx, cy + 4, 3);
            let castle = data.obj("castle");
            let _ = world.spawn(data, pid, castle, Vec2::from_tile(cx + 5, cy + 2), false);
            for i in 0..4 {
                self.put(world, pid, "villager", cx - 3 + i, cy + 5, 3);
            }
        }
        if params.settings.mode == GameMode::Deathmatch {
            // Nothing extra: the stockpile does the talking.
        }
        // Sheep: 4 close, 4 a little further.
        for i in 0..4 {
            let a = 90 * i + self.rng.range(0, 60);
            let (sx, sy) = noise::circle_point(a);
            self.put(
                world,
                PlayerId::GAIA,
                "sheep",
                cx + 4 * sx / 1000,
                cy + 4 * sy / 1000,
                3,
            );
        }
        for i in 0..4 {
            let a = 90 * i + 45 + self.rng.range(0, 30);
            let (sx, sy) = noise::circle_point(a);
            self.put(
                world,
                PlayerId::GAIA,
                "sheep",
                cx + 9 * sx / 1000,
                cy + 9 * sy / 1000,
                3,
            );
        }
        // Berries: 6 bushes in a clump at distance 6..8.
        let a = self.rng.range(0, 360);
        let (sx, sy) = noise::circle_point(a);
        let (bx, by) = (cx + 7 * sx / 1000, cy + 7 * sy / 1000);
        for i in 0..6 {
            let (ox, oy) = [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)][i];
            self.put(world, PlayerId::GAIA, "berry_bush", bx + ox, by + oy, 2);
        }
        // Gold: 7 tiles main pile and a 4 pile; stone: 5 and 4.
        let a2 = a + 100 + self.rng.range(0, 60);
        self.pile(
            world,
            "gold_mine",
            cx,
            cy,
            a2,
            8,
            &[(0, 0), (1, 0), (0, 1), (1, 1), (2, 0), (2, 1), (1, 2)],
        );
        let a3 = a + 220 + self.rng.range(0, 60);
        self.pile(
            world,
            "stone_mine",
            cx,
            cy,
            a3,
            8,
            &[(0, 0), (1, 0), (0, 1), (1, 1), (2, 0)],
        );
        let a4 = a2 + 60;
        self.pile(
            world,
            "gold_mine",
            cx,
            cy,
            a4,
            13,
            &[(0, 0), (1, 0), (0, 1), (1, 1)],
        );
        let a5 = a3 + 60;
        self.pile(
            world,
            "stone_mine",
            cx,
            cy,
            a5,
            13,
            &[(0, 0), (1, 0), (0, 1), (1, 1)],
        );
        // Boar and deer.
        for i in 0..2 {
            let a = a + 180 + i * 40 + self.rng.range(0, 20);
            let (sx, sy) = noise::circle_point(a);
            self.put(
                world,
                PlayerId::GAIA,
                "boar",
                cx + 9 * sx / 1000,
                cy + 9 * sy / 1000,
                3,
            );
        }
        let a = self.rng.range(0, 360);
        let (sx, sy) = noise::circle_point(a);
        for _ in 0..4 {
            self.put(
                world,
                PlayerId::GAIA,
                "deer",
                cx + 11 * sx / 1000,
                cy + 11 * sy / 1000,
                3,
            );
        }
        // Starting forest: a blob at distance 9..11 (two of them), plus straggler trees.
        for k in 0..2 {
            let a = a + 120 * (k + 1) + self.rng.range(0, 40);
            let (sx, sy) = noise::circle_point(a);
            let (fx, fy) = (cx + 10 * sx / 1000, cy + 10 * sy / 1000);
            let mut map = std::mem::replace(&mut world.map, Map::new(1, 1, Terrain::Grass));
            self.grow_blob(
                &mut map,
                fx,
                fy,
                30,
                Terrain::Forest,
                &[Terrain::Grass, Terrain::Dirt],
            );
            world.map = map;
        }
        for _ in 0..5 {
            let x = cx + self.rng.range(-7, 8);
            let y = cy + self.rng.range(-7, 8);
            if (x - cx).abs() > 3 || (y - cy).abs() > 3 {
                self.put(world, PlayerId::GAIA, "tree", x, y, 1);
            }
        }
        // Starting population support comes from the TC; houses are the player's job.
    }

    fn pile(
        &mut self,
        world: &mut World,
        key: &str,
        cx: i32,
        cy: i32,
        angle: i32,
        dist: i32,
        shape: &[(i32, i32)],
    ) {
        let (sx, sy) = noise::circle_point(angle);
        let (px, py) = (cx + dist * sx / 1000, cy + dist * sy / 1000);
        for (ox, oy) in shape {
            self.put(world, PlayerId::GAIA, key, px + ox, py + oy, 2);
        }
    }

    /// Turn forest terrain into tree objects, add fish, wolves and extra piles.
    fn scatter_gaia(&mut self, world: &mut World, spots: &[(i32, i32)], map_type: MapType) {
        let data = self.data;
        let tree = data.obj("tree");
        let pine = data.obj("pine");
        let fish = data.obj("fish");
        let shore_fish = data.obj("shore_fish");
        let seed = self.rng.next_u64();
        for y in 0..self.h {
            for x in 0..self.w {
                match world.map.terrain(x, y) {
                    Terrain::Forest => {
                        let ty = if noise::value_noise(x, y, 5, seed) > 150 {
                            pine
                        } else {
                            tree
                        };
                        // Forest tiles are impassable even if a tree fails to spawn.
                        world.spawn(data, PlayerId::GAIA, ty, Vec2::from_tile(x, y), false);
                    }
                    Terrain::Water | Terrain::DeepWater => {
                        let v = noise::value_noise(x * 7 + 3, y * 7 + 1, 1, seed ^ 0x77);
                        let near_shore = (-2..=2).any(|dy| {
                            (-2..=2).any(|dx| {
                                world.map.in_bounds(x + dx, y + dy)
                                    && !world.map.terrain(x + dx, y + dy).is_water()
                            })
                        });
                        if near_shore && v > 243 {
                            world.spawn(
                                data,
                                PlayerId::GAIA,
                                shore_fish,
                                Vec2::from_tile(x, y),
                                false,
                            );
                        } else if !near_shore && v > 246 {
                            world.spawn(data, PlayerId::GAIA, fish, Vec2::from_tile(x, y), false);
                        }
                    }
                    _ => {}
                }
            }
        }
        // Neutral gold/stone piles and animals in the middle regions.
        let extra = (spots.len() as i32 * 2).max(4);
        for i in 0..extra {
            let x = self.rng.range(8, self.w - 8);
            let y = self.rng.range(8, self.h - 8);
            if spots
                .iter()
                .any(|&(sx, sy)| (sx - x) * (sx - x) + (sy - y) * (sy - y) < 18 * 18)
            {
                continue;
            }
            let key = if i % 2 == 0 {
                "gold_mine"
            } else {
                "stone_mine"
            };
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.put(world, PlayerId::GAIA, key, x + ox, y + oy, 2);
            }
        }
        if map_type != MapType::Arena {
            for _ in 0..(spots.len() as i32).max(2) {
                let x = self.rng.range(6, self.w - 6);
                let y = self.rng.range(6, self.h - 6);
                if spots
                    .iter()
                    .any(|&(sx, sy)| (sx - x) * (sx - x) + (sy - y) * (sy - y) < 16 * 16)
                {
                    continue;
                }
                self.put(world, PlayerId::GAIA, "wolf", x, y, 3);
            }
        }
        for _ in 0..(spots.len() as i32 * 2) {
            let x = self.rng.range(6, self.w - 6);
            let y = self.rng.range(6, self.h - 6);
            if spots
                .iter()
                .any(|&(sx, sy)| (sx - x) * (sx - x) + (sy - y) * (sy - y) < 14 * 14)
            {
                continue;
            }
            for _ in 0..3 {
                self.put(world, PlayerId::GAIA, "deer", x, y, 3);
            }
        }
    }

    fn place_relics(&mut self, world: &mut World, spots: &[(i32, i32)]) {
        let n = (spots.len() as i32 + 1).clamp(3, 5);
        let mut placed = 0;
        let mut tries = 0;
        while placed < n && tries < 400 {
            tries += 1;
            let x = self.rng.range(6, self.w - 6);
            let y = self.rng.range(6, self.h - 6);
            if spots
                .iter()
                .any(|&(sx, sy)| (sx - x) * (sx - x) + (sy - y) * (sy - y) < 15 * 15)
            {
                continue;
            }
            if !world.map.land_passable(x, y) {
                continue;
            }
            if self.put(world, PlayerId::GAIA, "relic", x, y, 1).is_some() {
                placed += 1;
            }
        }
    }

    fn ring_walls(&mut self, world: &mut World, cx: i32, cy: i32, r: i32) {
        let data = self.data;
        let wall = data.obj("stone_wall");
        let gate = data.obj("gate");
        let mut ring: Vec<(i32, i32)> = Vec::new();
        for a in 0..720 {
            let (sx, sy) = noise::circle_point(a / 2);
            let p = (cx + r * sx / 1000, cy + r * sy / 1000);
            if ring.last() != Some(&p) && !ring.contains(&p) {
                ring.push(p);
            }
        }
        let gate_every = ring.len() / 4;
        for (i, (x, y)) in ring.iter().enumerate() {
            if !world.map.in_bounds(*x, *y) || !world.map.terrain(*x, *y).buildable() {
                continue;
            }
            if world.map.occupant(*x, *y).is_some() {
                continue;
            }
            let ty = if gate_every > 0 && i % gate_every == gate_every / 2 {
                gate
            } else {
                wall
            };
            let owner = ring_owner(world, cx, cy);
            world.spawn(data, owner, ty, Vec2::from_tile(*x, *y), false);
        }
    }
}

fn ring_owner(world: &World, cx: i32, cy: i32) -> PlayerId {
    // The player whose town centre is nearest owns the ring.
    let p = Vec2::tile_center(cx, cy);
    let mut best = (Fixed::MAX, PlayerId::GAIA);
    for e in world.iter() {
        if !e.owner.is_gaia() {
            let d = e.pos.dist(p);
            if d < best.0 {
                best = (d, e.owner);
            }
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_map_type_generates_and_is_deterministic() {
        let data = wars_data::data();
        for mt in MapType::ALL {
            let params = MapParams {
                map_type: mt,
                size: MapSize::Tiny,
                seed: 7,
                ..Default::default()
            };
            let a = generate(data, &params);
            let b = generate(data, &params);
            assert_eq!(a.hash(), b.hash(), "{mt:?} deterministic");
            assert_eq!(a.players.len(), 3);
            let tcs = a
                .iter()
                .filter(|e| data.object(e.ty).is_town_center)
                .count();
            assert_eq!(tcs, 2, "{mt:?}: two town centres");
            let vils = a.iter().filter(|e| data.object(e.ty).is_villager).count();
            assert_eq!(vils, 6, "{mt:?}: six villagers");
            let trees = a
                .iter()
                .filter(|e| data.object(e.ty).key == "tree" || data.object(e.ty).key == "pine")
                .count();
            assert!(trees > 50, "{mt:?}: trees {trees}");
            assert!(a.total_relics >= 3);
        }
    }

    #[test]
    fn eight_players_fit_on_a_huge_map() {
        let data = wars_data::data();
        let civs = [
            "britons",
            "byzantines",
            "celts",
            "chinese",
            "franks",
            "goths",
            "japanese",
            "mongols",
        ];
        let players = civs
            .iter()
            .enumerate()
            .map(|(i, c)| PlayerSetup {
                name: format!("P{i}"),
                civ: c.to_string(),
                team: (i % 2) as u8 + 1,
                is_ai: true,
                ai_level: 1,
            })
            .collect();
        let params = MapParams {
            map_type: MapType::Arabia,
            size: MapSize::Huge,
            seed: 99,
            players,
            ..Default::default()
        };
        let w = generate(data, &params);
        let tcs = w
            .iter()
            .filter(|e| data.object(e.ty).is_town_center)
            .count();
        assert_eq!(tcs, 8);
    }
}
