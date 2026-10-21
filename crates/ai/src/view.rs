//! One scan of the world per think, grouping everything the rules look at. Keeping the
//! grouping in one place keeps the passes cheap (each is a few loops over small lists)
//! and the iteration order fixed (entity index order), which the determinism rule needs.

use crate::AiMemory;
use std::collections::{BTreeMap, BTreeSet};
use wars_sim::fixed::{Fixed, Vec2};
use wars_sim::ids::*;
use wars_sim::types::{GameData, GatherKind, ObjectClass, Role};
use wars_sim::world::{Entity, Player, Task, World};

/// Squared distance in whole fixed units using i64, so even the far corners of a huge
/// map cannot overflow (the sim's `Vec2::len_sq` wraps past ~181 tiles).
pub(crate) fn dist2(a: Vec2, b: Vec2) -> i64 {
    let dx = (a.x.0 as i64) - (b.x.0 as i64);
    let dy = (a.y.0 as i64) - (b.y.0 as i64);
    dx * dx + dy * dy
}

/// `dist2` threshold for `r` tiles.
pub(crate) fn r2(r: i32) -> i64 {
    let r = r as i64 * wars_sim::fixed::ONE as i64;
    r * r
}

pub(crate) fn within(a: Vec2, b: Vec2, r: i32) -> bool {
    dist2(a, b) <= r2(r)
}

/// Integer square root (Newton's method; exact floor).
pub(crate) fn isqrt(v: i64) -> i64 {
    if v <= 0 {
        return 0;
    }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x
}

/// Distance in whole tiles between two points.
pub(crate) fn tiles(a: Vec2, b: Vec2) -> i32 {
    (isqrt(dist2(a, b)) >> wars_sim::fixed::FRAC_BITS) as i32
}

/// Integer tile distance (Chebyshev) from a point to a building footprint edge.
pub(crate) fn edge_tiles(world: &World, data: &GameData, from: Vec2, e: &Entity) -> i32 {
    let (tx, ty, w, h) = world.footprint(data, e);
    let (fx, fy) = from.tile();
    let dx = if fx < tx {
        tx - fx
    } else if fx >= tx + w {
        fx - (tx + w - 1)
    } else {
        0
    };
    let dy = if fy < ty {
        ty - fy
    } else if fy >= ty + h {
        fy - (ty + h - 1)
    } else {
        0
    };
    dx.max(dy)
}

pub(crate) struct View<'a> {
    pub player: &'a Player,
    pub age: Age,
    /// The (first complete) town centre and the point everything is measured from.
    pub tc: Option<&'a Entity>,
    pub home: Vec2,
    /// Own villagers that are outside buildings.
    pub villagers: Vec<&'a Entity>,
    pub idle_villagers: Vec<EntityId>,
    /// Villagers currently gathering each resource (including drop-off trips).
    pub gather_ids: [Vec<EntityId>; 4],
    /// Villagers on a resource node (gathering it or about to resume it).
    pub workers: BTreeMap<EntityId, i32>,
    /// Foundations that have at least one builder on the way.
    pub crewed: BTreeSet<EntityId>,
    /// Complete own buildings and own foundations.
    pub buildings: Vec<&'a Entity>,
    pub foundations: Vec<&'a Entity>,
    pub complete: BTreeMap<TypeId, i32>,
    pub pending: BTreeMap<TypeId, i32>,
    /// Attack-capable own units (not villagers; the scout only once scouting is over).
    pub military: Vec<&'a Entity>,
    pub monks: Vec<EntityId>,
    pub scout: Option<&'a Entity>,
    /// Enemy units able to fight and enemy buildings (full map knowledge).
    pub enemy_units: Vec<&'a Entity>,
    pub enemy_buildings: Vec<&'a Entity>,
    pub enemy_cavalry: usize,
    pub enemy_archers: usize,
    /// Gatherable nodes near home: gaia resources, animals, own farms.
    pub nodes: Vec<&'a Entity>,
    pub pop_cap: i32,
    /// Units queued in own buildings (population not yet counted).
    pub queued_units: i32,
    pub queued_villagers: i32,
}

/// Nodes further than this from home are ignored.
const NODE_RADIUS: i32 = 36;

impl<'a> View<'a> {
    pub fn build(
        world: &'a World,
        data: &'a GameData,
        me: PlayerId,
        mem: &mut AiMemory,
    ) -> View<'a> {
        let player = world.player(me);
        let mut v = View {
            player,
            age: player.age,
            tc: None,
            home: Vec2::ZERO,
            villagers: Vec::new(),
            idle_villagers: Vec::new(),
            gather_ids: Default::default(),
            workers: BTreeMap::new(),
            crewed: BTreeSet::new(),
            buildings: Vec::new(),
            foundations: Vec::new(),
            complete: BTreeMap::new(),
            pending: BTreeMap::new(),
            military: Vec::new(),
            monks: Vec::new(),
            scout: None,
            enemy_units: Vec::new(),
            enemy_buildings: Vec::new(),
            enemy_cavalry: 0,
            enemy_archers: 0,
            nodes: Vec::new(),
            pop_cap: wars_sim::stats::pop_cap(world, data, me).min(player.pop_support),
            queued_units: 0,
            queued_villagers: 0,
        };
        let villager_ty = data.object_id("villager");

        // Pick the scout once; forget it when it dies.
        if let Some(s) = mem.scout {
            if world.get(s).map(|e| e.owner != me).unwrap_or(true) {
                mem.scout = None;
                mem.scout_done = true;
            }
        }

        // First pass: own buildings (home must be known before measuring distances).
        for e in world.entities_of(me) {
            let def = data.object(e.ty);
            if def.class != ObjectClass::Building {
                continue;
            }
            if e.build_progress >= 1000 {
                v.buildings.push(e);
                *v.complete.entry(e.ty).or_insert(0) += 1;
                if def.is_town_center && v.tc.is_none() {
                    v.tc = Some(e);
                }
            } else {
                v.foundations.push(e);
                *v.pending.entry(e.ty).or_insert(0) += 1;
            }
            for q in &e.queue {
                if let wars_sim::world::QueueItem::Unit(u) = q {
                    v.queued_units += data.object(*u).population;
                    if Some(*u) == villager_ty {
                        v.queued_villagers += 1;
                    }
                }
            }
        }
        v.home = match v.tc {
            Some(tc) => tc.pos,
            None => v
                .buildings
                .first()
                .or(v.foundations.first())
                .map(|b| b.pos)
                .unwrap_or(Vec2::ZERO),
        };

        // Second pass: units of everyone and gatherable nodes.
        for e in world.iter() {
            let def = data.object(e.ty);
            if e.owner == me {
                if def.class != ObjectClass::Unit || e.inside.is_some() {
                    continue;
                }
                if def.is_villager {
                    v.villagers.push(e);
                    v.classify_villager(e);
                } else if def.can_convert {
                    v.monks.push(e.id);
                } else if !mem.scout_done
                    && (Some(e.id) == mem.scout || (def.role == Role::Scout && mem.scout.is_none()))
                {
                    // The starting scout explores until `scout_done`; then it fights.
                    mem.scout = Some(e.id);
                    v.scout = Some(e);
                } else if !def.attacks.is_empty() && !(def.key == "trebuchet" && e.packed) {
                    v.military.push(e);
                }
                if v.home == Vec2::ZERO {
                    v.home = e.pos;
                }
            } else if world.enemy(me, e.owner) {
                match def.class {
                    ObjectClass::Unit if !def.attacks.is_empty() && !def.is_villager => {
                        if e.inside.is_none() {
                            v.enemy_units.push(e);
                            match def.role {
                                Role::Cavalry | Role::Scout | Role::CavalryArcher => {
                                    v.enemy_cavalry += 1
                                }
                                Role::Archer => v.enemy_archers += 1,
                                _ => {}
                            }
                        }
                    }
                    ObjectClass::Building => v.enemy_buildings.push(e),
                    _ => {}
                }
            } else if e.owner.is_gaia() {
                if def.yields.is_some() && e.resource_left > 0 && within(e.pos, v.home, NODE_RADIUS)
                {
                    v.nodes.push(e);
                }
            }
        }
        // Own farms and captured sheep are nodes too.
        for e in world.entities_of(me) {
            let def = data.object(e.ty);
            if def.yields.is_some() && e.resource_left > 0 {
                if def.class == ObjectClass::Building && e.build_progress < 1000 {
                    continue;
                }
                v.nodes.push(e);
            }
        }
        if mem.scout.is_some() && v.scout.is_none() && !mem.scout_done {
            // Garrisoned or converted: treat the scouting as over.
            mem.scout_done = true;
        }
        v
    }

    /// Count a villager towards its current job.
    fn classify_villager(&mut self, e: &'a Entity) {
        match e.task() {
            Task::Idle => self.idle_villagers.push(e.id),
            Task::Gather { target, kind } => {
                self.gather_ids[kind.resource().idx()].push(e.id);
                *self.workers.entry(*target).or_insert(0) += 1;
            }
            Task::DropOff { resume, .. } => {
                if let Some(k) = e.carry_kind {
                    self.gather_ids[k.resource().idx()].push(e.id);
                }
                if let Some(r) = resume {
                    *self.workers.entry(*r).or_insert(0) += 1;
                }
            }
            Task::Attack { target, .. } => {
                // Hunting counts as food work.
                self.gather_ids[Resource::Food.idx()].push(e.id);
                *self.workers.entry(*target).or_insert(0) += 1;
            }
            Task::Build { target } | Task::Repair { target } => {
                self.crewed.insert(*target);
            }
            _ => {}
        }
    }

    pub fn count(&self, ty: TypeId) -> i32 {
        self.complete.get(&ty).copied().unwrap_or(0)
    }
    pub fn pending_count(&self, ty: TypeId) -> i32 {
        self.pending.get(&ty).copied().unwrap_or(0)
    }
    pub fn has(&self, data: &GameData, key: &str) -> bool {
        data.object_id(key)
            .map(|t| self.count(t) > 0)
            .unwrap_or(false)
    }
    pub fn villager_count(&self) -> i32 {
        self.villagers.len() as i32
    }
    /// Villagers working on resource `r` right now.
    pub fn gatherers(&self, r: Resource) -> i32 {
        self.gather_ids[r.idx()].len() as i32
    }
    pub fn pop_room(&self) -> i32 {
        self.pop_cap - self.player.pop - self.queued_units
    }
    /// Complete own buildings of a type, in id order.
    pub fn buildings_of(&self, ty: TypeId) -> impl Iterator<Item = &&'a Entity> {
        self.buildings.iter().filter(move |b| b.ty == ty)
    }
    /// Own completed farms (planted or being rebuilt).
    pub fn farm_count(&self, data: &GameData) -> i32 {
        data.object_id("farm")
            .map(|f| self.count(f) + self.pending_count(f))
            .unwrap_or(0)
    }
    /// Enemy fighting units within `r` tiles of a point, sorted by distance then id.
    pub fn enemies_near(&self, p: Vec2, r: i32) -> Vec<&'a Entity> {
        let mut v: Vec<(i64, EntityId, &'a Entity)> = self
            .enemy_units
            .iter()
            .filter(|e| within(e.pos, p, r))
            .map(|e| (dist2(e.pos, p), e.id, *e))
            .collect();
        v.sort_by_key(|(d, id, _)| (*d, *id));
        v.into_iter().map(|(_, _, e)| e).collect()
    }
    /// Nearest own drop-off accepting `kind` to a point, as (tile distance, entity).
    pub fn nearest_drop(
        &self,
        world: &World,
        data: &GameData,
        p: Vec2,
        kind: GatherKind,
    ) -> Option<(i32, &'a Entity)> {
        let mut best: Option<(i32, &'a Entity)> = None;
        for b in &self.buildings {
            if !data.object(b.ty).drop_off.contains(&kind) {
                continue;
            }
            let d = edge_tiles(world, data, p, b);
            if best
                .map(|(bd, bb)| d < bd || (d == bd && b.id < bb.id))
                .unwrap_or(true)
            {
                best = Some((d, b));
            }
        }
        best
    }
    /// Centre of the army (integer average), or home when there is none.
    pub fn army_centre(&self) -> Vec2 {
        if self.military.is_empty() {
            return self.home;
        }
        let n = self.military.len() as i64;
        let sx: i64 = self.military.iter().map(|e| e.pos.x.0 as i64).sum();
        let sy: i64 = self.military.iter().map(|e| e.pos.y.0 as i64).sum();
        Vec2::new(Fixed((sx / n) as i32), Fixed((sy / n) as i32))
    }
}
