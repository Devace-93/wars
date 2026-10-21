//! The world state: map, players and entities. Fully serialisable, so a snapshot is
//! also a save file, a scenario and the late-join payload.

use crate::fixed::{Fixed, Vec2};
use crate::ids::*;
use crate::map::Map;
use crate::rng::{fnv1a, Rng};
use crate::types::GameData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const TICKS_PER_SECOND: u32 = 20;
pub const MAX_PLAYERS: usize = 9; // gaia + 8

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Task {
    Idle,
    Move {
        to: Vec2,
        path: Vec<(i32, i32)>,
        attack_move: bool,
        patrol_from: Option<Vec2>,
    },
    Attack {
        target: EntityId,
        chasing: bool,
    },
    Gather {
        target: EntityId,
        kind: crate::types::GatherKind,
    },
    /// Carry resources back to the nearest drop-off, then resume gathering `resume`.
    DropOff {
        drop: EntityId,
        resume: Option<EntityId>,
    },
    Build {
        target: EntityId,
    },
    Repair {
        target: EntityId,
    },
    Garrison {
        target: EntityId,
    },
    Follow {
        target: EntityId,
    },
    Guard {
        target: EntityId,
    },
    Convert {
        target: EntityId,
    },
    Heal {
        target: EntityId,
    },
    /// Trade: walking to the far market (`to_market`) or coming home with gold.
    Trade {
        home: EntityId,
        far: EntityId,
        outbound: bool,
    },
    /// Pick up a relic / deliver a relic to a monastery.
    PickRelic {
        target: EntityId,
    },
    DeliverRelic {
        target: EntityId,
    },
    /// Trebuchet packing/unpacking countdown.
    Packing {
        ticks: u32,
        unpack: bool,
    },
}

/// Production queue entry: a unit or a technology.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum QueueItem {
    Unit(TypeId),
    Tech(TechId),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub owner: PlayerId,
    pub ty: TypeId,
    pub pos: Vec2,
    pub hp: i32,
    pub facing: u8,
    pub tasks: VecDeque<Task>,
    pub stance: Stance,
    pub formation: Formation,
    /// Attack cooldown in ticks.
    pub reload: u32,
    /// Carried resources (villagers, trade units) and the gather kind they came from.
    pub carry: i32,
    pub carry_kind: Option<crate::types::GatherKind>,
    /// Remaining resource units for resource objects / animals / farms.
    pub resource_left: i32,
    /// Buildings under construction: 0..=1000 (per-mille). 1000 = complete.
    pub build_progress: i32,
    /// Production queue (front item is in progress) and its progress in ticks.
    pub queue: VecDeque<QueueItem>,
    pub queue_progress: u32,
    pub rally: Option<crate::command::RallyTarget>,
    pub garrisoned: Vec<EntityId>,
    pub inside: Option<EntityId>,
    /// Relic carried by a monk / count of relics in a monastery.
    pub relics: u8,
    /// Trebuchet / siege packed state.
    pub packed: bool,
    /// Ticks until this entity is removed (corpses, depleted resources).
    pub decay: Option<u32>,
    /// Faith for monks (0..=100), conversion progress.
    pub faith: i32,
    /// Wonder/relic victory countdown started at this tick (buildings).
    pub completed_tick: Option<u32>,
    /// Animal: has been attacked, so it is now a huntable carcass.
    pub alerted: bool,
    /// Villager mind: last resource kind to prefer when idle-reseeding farms etc.
    pub home_drop: Option<EntityId>,
    /// Visual: animation state hint for clients (0 idle, 1 walk, 2 attack, 3 work, 4 die).
    pub anim: u8,
    pub anim_tick: u32,
    /// Current tile path and the goal tile it was computed for.
    pub path: Vec<(i32, i32)>,
    pub path_goal: Option<(i32, i32)>,
    pub repath_tick: u32,
    /// Fractional gather accumulator.
    pub carry_frac: Fixed,
    /// Generic per-task progress counter (conversion, packing, repair charge).
    pub work: u32,
    /// Where a villager was gathering last (to look for the next node nearby).
    pub last_gather_pos: Option<Vec2>,
}

impl Entity {
    pub fn task(&self) -> &Task {
        self.tasks.front().unwrap_or(&Task::Idle)
    }
    pub fn is_idle(&self) -> bool {
        self.tasks.is_empty() || matches!(self.tasks.front(), Some(Task::Idle))
    }
    pub fn tile(&self) -> (i32, i32) {
        self.pos.tile()
    }
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub civ: CivId,
    pub team: u8,
    pub is_ai: bool,
    pub ai_level: u8,
    pub color: u8,
    pub resources: Stockpile,
    pub age: Age,
    pub techs: BTreeSet<TechId>,
    /// Techs currently being researched (so they cannot be queued twice).
    pub researching: BTreeSet<TechId>,
    pub pop: i32,
    pub pop_support: i32,
    pub pop_cap: i32,
    pub alive: bool,
    pub resigned: bool,
    pub diplomacy: [Diplomacy; MAX_PLAYERS],
    /// Explored tiles (fog of war). Visible tiles are recomputed by `vision`.
    pub explored: Vec<bool>,
    pub visible: Vec<u8>,
    pub score: i32,
    pub kills: i32,
    pub losses: i32,
    pub razings: i32,
    pub relics_held: i32,
    pub wonder_tick: Option<u32>,
    pub relic_tick: Option<u32>,
    /// Market prices in gold per 100 (buy price; sell is the inverse minus fee).
    pub market_food: i32,
    pub market_wood: i32,
    pub market_stone: i32,
    pub cheats_used: bool,
    /// Resource gathered totals for the post-game screen.
    pub gathered: Stockpile,
    pub trade_profit: i32,
    pub tribute_sent: i32,
    pub tribute_received: i32,
    pub units_trained: i32,
    pub buildings_built: i32,
    pub conversions: i32,
    pub idle_hint: Option<EntityId>,
    pub last_alert_tick: u32,
    /// Relic gold accumulator in hundredths.
    pub gold_frac: i32,
    pub defeated_tick: Option<u32>,
}

impl Player {
    pub fn is_ally(&self, other: PlayerId) -> bool {
        self.id == other || self.diplomacy[other.idx()] == Diplomacy::Ally
    }
    pub fn is_enemy(&self, other: PlayerId) -> bool {
        self.id != other && self.diplomacy[other.idx()] == Diplomacy::Enemy
    }
    pub fn has_tech(&self, t: TechId) -> bool {
        self.techs.contains(&t)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VictoryMode {
    /// Conquest, wonder or relics all count.
    Standard,
    Conquest,
    /// Score at the time limit (in minutes) or earlier by conquest.
    TimeLimit(u32),
    /// Only the scenario's triggers decide.
    Scenario,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GameMode {
    RandomMap,
    Deathmatch,
    Regicide,
    Scenario,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub mode: GameMode,
    pub victory: VictoryMode,
    pub pop_cap: i32,
    pub starting_age: Age,
    pub reveal_map: bool,
    pub allow_cheats: bool,
    pub team_together: bool,
    pub lock_teams: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: GameMode::RandomMap,
            victory: VictoryMode::Standard,
            pop_cap: 200,
            starting_age: Age::Dark,
            reveal_map: false,
            allow_cheats: false,
            team_together: true,
            lock_teams: true,
        }
    }
}

/// Things that happened during a tick that clients want to show (sounds, alerts) and
/// that are not part of the authoritative state.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Event {
    UnderAttack {
        player: PlayerId,
        at: Vec2,
        building: bool,
    },
    UnitTrained {
        player: PlayerId,
        entity: EntityId,
    },
    ResearchComplete {
        player: PlayerId,
        tech: TechId,
    },
    AgeAdvanced {
        player: PlayerId,
        age: Age,
    },
    BuildingComplete {
        player: PlayerId,
        entity: EntityId,
    },
    Died {
        entity: EntityId,
        at: Vec2,
        ty: TypeId,
        owner: PlayerId,
    },
    Converted {
        entity: EntityId,
        from: PlayerId,
        to: PlayerId,
    },
    PlayerDefeated {
        player: PlayerId,
    },
    PlayerResigned {
        player: PlayerId,
    },
    Victory {
        team: u8,
    },
    WonderStarted {
        player: PlayerId,
    },
    RelicsStarted {
        player: PlayerId,
    },
    Chat {
        player: PlayerId,
        text: String,
        to_allies: bool,
    },
    Shot {
        from: Vec2,
        to: Vec2,
        ticks: u32,
    },
    Cheat {
        player: PlayerId,
        code: String,
    },
    NotEnough {
        player: PlayerId,
        resource: Resource,
    },
    PopCapReached {
        player: PlayerId,
    },
    Tribute {
        from: PlayerId,
        to: PlayerId,
        amount: Stockpile,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct World {
    pub tick: u32,
    pub map: Map,
    pub players: Vec<Player>,
    pub entities: Vec<Option<Entity>>,
    pub free: Vec<u32>,
    pub generations: Vec<u32>,
    pub rng: Rng,
    pub settings: Settings,
    pub winner_team: Option<u8>,
    pub game_over: bool,
    #[serde(skip)]
    pub events: Vec<Event>,
    /// Scenario triggers (empty for random maps).
    pub triggers: Vec<crate::trigger::Trigger>,
    pub trigger_state: Vec<crate::trigger::TriggerState>,
    pub scenario_name: String,
    /// Scenario object references for triggers (editor index -> entity).
    pub object_refs: Vec<Option<EntityId>>,
    /// Total relics placed on the map at start (for the relic victory).
    pub total_relics: i32,
    /// Pending ranged shots: (arrive_tick, attacker owner, target, damage, from pos).
    pub shots: Vec<Shot>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Shot {
    pub arrive: u32,
    pub attacker: EntityId,
    pub owner: PlayerId,
    pub target: EntityId,
    pub target_pos: Vec2,
    pub attacks: Vec<crate::types::Attack>,
    pub ballistics: bool,
    pub blast: Fixed,
}

impl World {
    pub fn new(map: Map, seed: u64, settings: Settings) -> World {
        World {
            tick: 0,
            map,
            players: Vec::new(),
            entities: Vec::new(),
            free: Vec::new(),
            generations: Vec::new(),
            rng: Rng::new(seed),
            settings,
            winner_team: None,
            game_over: false,
            events: Vec::new(),
            triggers: Vec::new(),
            trigger_state: Vec::new(),
            scenario_name: String::new(),
            object_refs: Vec::new(),
            total_relics: 0,
            shots: Vec::new(),
        }
    }

    pub fn add_player(
        &mut self,
        name: &str,
        civ: CivId,
        team: u8,
        is_ai: bool,
        resources: Stockpile,
    ) -> PlayerId {
        let id = PlayerId(self.players.len() as u8);
        let n = (self.map.width * self.map.height) as usize;
        let mut diplomacy = [Diplomacy::Enemy; MAX_PLAYERS];
        diplomacy[0] = Diplomacy::Neutral; // gaia
        diplomacy[id.idx()] = Diplomacy::Ally;
        for p in &mut self.players {
            if p.id.is_gaia() {
                continue;
            }
            if p.team != 0 && p.team == team {
                p.diplomacy[id.idx()] = Diplomacy::Ally;
                diplomacy[p.id.idx()] = Diplomacy::Ally;
            }
        }
        if id.is_gaia() {
            diplomacy = [Diplomacy::Neutral; MAX_PLAYERS];
        }
        self.players.push(Player {
            id,
            name: name.to_string(),
            civ,
            team,
            is_ai,
            ai_level: 1,
            color: id.0,
            resources,
            age: self.settings.starting_age,
            techs: BTreeSet::new(),
            researching: BTreeSet::new(),
            pop: 0,
            pop_support: 0,
            pop_cap: self.settings.pop_cap,
            alive: true,
            resigned: false,
            diplomacy,
            explored: vec![self.settings.reveal_map || id.is_gaia(); n],
            visible: vec![0; n],
            score: 0,
            kills: 0,
            losses: 0,
            razings: 0,
            relics_held: 0,
            wonder_tick: None,
            relic_tick: None,
            market_food: 100,
            market_wood: 100,
            market_stone: 130,
            cheats_used: false,
            gathered: Stockpile::ZERO,
            trade_profit: 0,
            tribute_sent: 0,
            tribute_received: 0,
            units_trained: 0,
            buildings_built: 0,
            conversions: 0,
            idle_hint: None,
            last_alert_tick: 0,
            gold_frac: 0,
            defeated_tick: None,
        });
        id
    }

    pub fn player(&self, id: PlayerId) -> &Player {
        &self.players[id.idx()]
    }
    pub fn player_mut(&mut self, id: PlayerId) -> &mut Player {
        &mut self.players[id.idx()]
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        match self.entities.get(id.index as usize) {
            Some(Some(e)) if e.id.gen == id.gen && e.hp > 0 => Some(e),
            _ => None,
        }
    }
    /// Like `get` but also returns dead-but-not-yet-removed entities (corpses).
    pub fn get_any(&self, id: EntityId) -> Option<&Entity> {
        match self.entities.get(id.index as usize) {
            Some(Some(e)) if e.id.gen == id.gen => Some(e),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        match self.entities.get_mut(id.index as usize) {
            Some(Some(e)) if e.id.gen == id.gen => Some(e),
            _ => None,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities
            .iter()
            .filter_map(|e| e.as_ref())
            .filter(|e| e.hp > 0)
    }
    pub fn iter_all(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter_map(|e| e.as_ref())
    }
    pub fn ids(&self) -> Vec<EntityId> {
        self.iter().map(|e| e.id).collect()
    }
    pub fn count(&self) -> usize {
        self.iter().count()
    }

    /// Spawn an object. Buildings are spawned complete unless `foundation` is true.
    /// Returns None if a solid object cannot be placed there.
    pub fn spawn(
        &mut self,
        data: &GameData,
        owner: PlayerId,
        ty: TypeId,
        pos: Vec2,
        foundation: bool,
    ) -> Option<EntityId> {
        let def = data.object(ty);
        let (w, h) = (def.size.0 as i32, def.size.1 as i32);
        let solid = matches!(
            def.class,
            crate::types::ObjectClass::Building | crate::types::ObjectClass::Resource
        ) && !def.walkable;
        let (tx, ty_) = pos.tile();
        if solid && !self.map.can_place(tx, ty_, w, h, def.placement) {
            // Walkable buildings (farms) still need free tiles.
            return None;
        }
        if !solid && def.class == crate::types::ObjectClass::Building {
            // farms, gates: tiles must be free of other solids
            for yy in ty_..ty_ + h {
                for xx in tx..tx + w {
                    if !self.map.in_bounds(xx, yy) || self.map.occupant(xx, yy).is_some() {
                        return None;
                    }
                }
            }
        }
        let index = match self.free.pop() {
            Some(i) => i,
            None => {
                self.entities.push(None);
                self.generations.push(0);
                (self.entities.len() - 1) as u32
            }
        };
        let gen = self.generations[index as usize];
        let id = EntityId { index, gen };
        let stats = crate::stats::stats_for(self, data, owner, ty);
        let center = if def.class == crate::types::ObjectClass::Building
            || def.class == crate::types::ObjectClass::Resource
        {
            Vec2::new(
                Fixed::from_int(tx) + Fixed::from_int(w) / Fixed::from_int(2),
                Fixed::from_int(ty_) + Fixed::from_int(h) / Fixed::from_int(2),
            )
        } else {
            pos
        };
        let is_building = def.class == crate::types::ObjectClass::Building;
        let e = Entity {
            id,
            owner,
            ty,
            pos: center,
            hp: if foundation {
                1.max(stats.hp / 20)
            } else {
                stats.hp
            },
            facing: 0,
            tasks: VecDeque::new(),
            stance: if def.role == crate::types::Role::Villager
                || def.is_naval && def.role == crate::types::Role::Fishing
            {
                Stance::NoAttack
            } else {
                Stance::Aggressive
            },
            formation: Formation::Line,
            reload: 0,
            carry: 0,
            carry_kind: None,
            resource_left: def.yields.map(|y| y.amount).unwrap_or(0),
            build_progress: if is_building && foundation { 0 } else { 1000 },
            queue: VecDeque::new(),
            queue_progress: 0,
            rally: None,
            garrisoned: Vec::new(),
            inside: None,
            relics: 0,
            packed: def.key == "trebuchet",
            decay: None,
            faith: 100,
            completed_tick: if is_building && !foundation {
                Some(self.tick)
            } else {
                None
            },
            alerted: false,
            home_drop: None,
            anim: 0,
            anim_tick: 0,
            path: Vec::new(),
            path_goal: None,
            repath_tick: 0,
            carry_frac: Fixed::ZERO,
            work: 0,
            last_gather_pos: None,
        };
        self.entities[index as usize] = Some(e);
        if is_building || def.class == crate::types::ObjectClass::Resource {
            // Farms and gates are walkable but still own their tiles for placement purposes:
            // we mark occupancy only for solid objects.
            if !def.walkable {
                self.map.fill_occupancy(tx, ty_, w, h, Some(id));
            }
        }
        if !owner.is_gaia() {
            let p = &mut self.players[owner.idx()];
            if def.class == crate::types::ObjectClass::Unit {
                p.pop += stats.population;
            }
            if !foundation {
                p.pop_support += stats.pop_support;
            }
        }
        Some(id)
    }

    /// Remove an entity immediately (no corpse), freeing its tiles and population.
    pub fn despawn(&mut self, data: &GameData, id: EntityId) {
        let Some(e) = self.get_any(id).cloned() else {
            return;
        };
        let def = data.object(e.ty);
        let (w, h) = (def.size.0 as i32, def.size.1 as i32);
        if !def.walkable
            && matches!(
                def.class,
                crate::types::ObjectClass::Building | crate::types::ObjectClass::Resource
            )
        {
            let (tx, ty) = (
                e.pos.x - Fixed::from_int(w) / Fixed::from_int(2),
                e.pos.y - Fixed::from_int(h) / Fixed::from_int(2),
            );
            let (tx, ty) = (tx.round(), ty.round());
            for yy in ty..ty + h {
                for xx in tx..tx + w {
                    if self.map.occupant(xx, yy) == Some(id) {
                        self.map.set_occupant(xx, yy, None);
                    }
                }
            }
        }
        if !e.owner.is_gaia() {
            let stats = crate::stats::stats_for(self, data, e.owner, e.ty);
            let p = &mut self.players[e.owner.idx()];
            if def.class == crate::types::ObjectClass::Unit {
                p.pop -= stats.population;
            }
            if e.build_progress >= 1000 {
                p.pop_support -= stats.pop_support;
            }
        }
        self.entities[id.index as usize] = None;
        self.generations[id.index as usize] = self.generations[id.index as usize].wrapping_add(1);
        self.free.push(id.index);
    }

    /// Top-left tile of a building's footprint.
    pub fn footprint(&self, data: &GameData, e: &Entity) -> (i32, i32, i32, i32) {
        let def = data.object(e.ty);
        let (w, h) = (def.size.0 as i32, def.size.1 as i32);
        if def.class == crate::types::ObjectClass::Unit
            || def.class == crate::types::ObjectClass::Animal
        {
            let (x, y) = e.pos.tile();
            return (x, y, 1, 1);
        }
        let tx = (e.pos.x - Fixed::from_int(w) / Fixed::from_int(2)).round();
        let ty = (e.pos.y - Fixed::from_int(h) / Fixed::from_int(2)).round();
        (tx, ty, w, h)
    }

    /// Distance from a point to the edge of an entity's footprint (0 if inside).
    pub fn edge_distance(&self, data: &GameData, from: Vec2, e: &Entity) -> Fixed {
        let (tx, ty, w, h) = self.footprint(data, e);
        let def = data.object(e.ty);
        if def.class == crate::types::ObjectClass::Unit
            || def.class == crate::types::ObjectClass::Animal
        {
            return from.dist(e.pos);
        }
        let minx = Fixed::from_int(tx);
        let maxx = Fixed::from_int(tx + w);
        let miny = Fixed::from_int(ty);
        let maxy = Fixed::from_int(ty + h);
        let dx = if from.x < minx {
            minx - from.x
        } else if from.x > maxx {
            from.x - maxx
        } else {
            Fixed::ZERO
        };
        let dy = if from.y < miny {
            miny - from.y
        } else if from.y > maxy {
            from.y - maxy
        } else {
            Fixed::ZERO
        };
        Vec2::new(dx, dy).len()
    }

    /// Canonical hash of the authoritative state (events excluded) for desync checks.
    pub fn hash(&self) -> u64 {
        let bytes = postcard::to_allocvec(self).expect("world serialises");
        fnv1a(&bytes)
    }

    pub fn snapshot(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("world serialises")
    }

    pub fn from_snapshot(bytes: &[u8]) -> Result<World, postcard::Error> {
        postcard::from_bytes(bytes)
    }

    pub fn seconds(&self) -> u32 {
        self.tick / TICKS_PER_SECOND
    }

    pub fn ally(&self, a: PlayerId, b: PlayerId) -> bool {
        a == b || self.players[a.idx()].diplomacy[b.idx()] == Diplomacy::Ally
    }
    pub fn enemy(&self, a: PlayerId, b: PlayerId) -> bool {
        if a == b {
            return false;
        }
        if a.is_gaia() || b.is_gaia() {
            return false;
        }
        self.players[a.idx()].diplomacy[b.idx()] == Diplomacy::Enemy
    }

    /// Entities within `radius` tiles of `pos` (by centre), sorted by distance then id.
    pub fn nearby(&self, pos: Vec2, radius: Fixed) -> Vec<(Fixed, EntityId)> {
        let r2 = radius * radius;
        let mut v: Vec<(Fixed, EntityId)> = self
            .iter()
            .filter_map(|e| {
                let d2 = e.pos.dist_sq(pos);
                if d2 <= r2 {
                    Some((d2, e.id))
                } else {
                    None
                }
            })
            .collect();
        v.sort();
        v
    }

    pub fn entities_of(&self, p: PlayerId) -> impl Iterator<Item = &Entity> {
        self.iter().filter(move |e| e.owner == p)
    }

    pub fn type_count(&self, p: PlayerId, ty: TypeId) -> usize {
        self.iter()
            .filter(|e| e.owner == p && e.ty == ty && e.build_progress >= 1000)
            .count()
    }

    pub fn has_building(&self, data: &GameData, p: PlayerId, key: &str) -> bool {
        match data.object_id(key) {
            Some(ty) => self.type_count(p, ty) > 0,
            None => false,
        }
    }

    pub fn push_event(&mut self, ev: Event) {
        self.events.push(ev);
    }

    /// Explored/visible maps indexed by tile.
    pub fn is_visible(&self, p: PlayerId, x: i32, y: i32) -> bool {
        if !self.map.in_bounds(x, y) {
            return false;
        }
        self.players[p.idx()].visible[self.map.idx(x, y)] > 0
    }
    pub fn is_explored(&self, p: PlayerId, x: i32, y: i32) -> bool {
        if !self.map.in_bounds(x, y) {
            return false;
        }
        self.players[p.idx()].explored[self.map.idx(x, y)]
    }

    /// Entities a player can currently see (own and allied always; others when visible).
    pub fn visible_entities(&self, p: PlayerId) -> Vec<&Entity> {
        self.iter()
            .filter(|e| {
                if self.ally(p, e.owner) {
                    return true;
                }
                let (x, y) = e.pos.tile();
                self.is_visible(p, x, y)
            })
            .collect()
    }

    pub fn mut_two(&mut self, a: EntityId, b: EntityId) -> Option<(&mut Entity, &mut Entity)> {
        if a.index == b.index {
            return None;
        }
        let (ia, ib) = (a.index as usize, b.index as usize);
        let (lo, hi) = if ia < ib { (ia, ib) } else { (ib, ia) };
        let (left, right) = self.entities.split_at_mut(hi);
        let el = left[lo].as_mut()?;
        let er = right[0].as_mut()?;
        if el.id.gen != (if ia < ib { a.gen } else { b.gen })
            || er.id.gen != (if ia < ib { b.gen } else { a.gen })
        {
            return None;
        }
        if ia < ib {
            Some((el, er))
        } else {
            Some((er, el))
        }
    }

    pub fn alive_players(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|p| !p.id.is_gaia() && p.alive)
            .map(|p| p.id)
            .collect()
    }

    pub fn player_type_counts(&self, p: PlayerId) -> BTreeMap<TypeId, usize> {
        let mut m = BTreeMap::new();
        for e in self.entities_of(p) {
            *m.entry(e.ty).or_insert(0) += 1;
        }
        m
    }
}
