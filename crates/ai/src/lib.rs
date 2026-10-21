//! wars-ai: the scripted computer player.
//!
//! `AiPlayer::think` is called by the match runner every `THINK_INTERVAL` ticks with a
//! read-only view of the world and returns the commands the AI wants to issue. It must
//! be deterministic: no randomness outside `world.rng`-independent integer logic (use
//! the tick and entity ids for variety), because every participant runs the AI for
//! offline matches and the server runs it for online ones.
//!
//! The player is rule based and organised in a few passes that run every think:
//!
//! - [`view`] scans the world once and groups everything the rules need (own
//!   villagers by job, buildings, army, enemies, gatherable nodes).
//! - [`economy`] trains villagers, keeps the population headroom, walks the build
//!   order, assigns idle villagers to resources, builds farms and camps and researches
//!   the economic technologies.
//! - [`military`] keeps the production buildings busy with an age and civ appropriate
//!   composition, researches upgrades, defends the base, garrisons villagers, scouts
//!   and runs the attack / retreat cycle.
//! - [`place`] finds free tiles for buildings around the town centre.
//!
//! Every pass appends to a shared command list; `think` trims it to a small budget so
//! the command stream stays light even in the late game.

mod economy;
mod military;
mod place;
mod view;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wars_sim::command::Command;
use wars_sim::ids::{EntityId, PlayerId, Stockpile};
use wars_sim::types::GameData;
use wars_sim::world::World;

/// Ticks between two `think` calls (one game second).
pub const THINK_INTERVAL: u32 = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Easy = 0,
    Standard = 1,
    Hard = 2,
}

impl Difficulty {
    pub fn from_level(l: u8) -> Difficulty {
        match l {
            0 => Difficulty::Easy,
            1 => Difficulty::Standard,
            _ => Difficulty::Hard,
        }
    }

    /// Tuning knobs of each difficulty. Hard gets higher targets and faster decisions,
    /// never free resources.
    pub fn params(self) -> Params {
        match self {
            Difficulty::Easy => Params {
                period: 3,
                military_period: 3,
                villager_target: [16, 20, 22, 22],
                age_up_villagers: [16, 22, 26],
                attack_threshold: [25, 25, 25, 25],
                max_army: 35,
                max_foundations: 1,
                farm_target: [2, 6, 8, 10],
                imperial_unit_upgrades: false,
                wonder: false,
                max_commands: 12,
            },
            Difficulty::Standard => Params {
                period: 1,
                military_period: 2,
                villager_target: [20, 30, 35, 35],
                age_up_villagers: [19, 30, 34],
                attack_threshold: [15, 15, 20, 20],
                max_army: 60,
                max_foundations: 2,
                farm_target: [3, 8, 12, 14],
                imperial_unit_upgrades: true,
                wonder: false,
                max_commands: 12,
            },
            Difficulty::Hard => Params {
                period: 1,
                military_period: 1,
                villager_target: [20, 40, 55, 60],
                age_up_villagers: [19, 34, 40],
                attack_threshold: [12, 12, 12, 12],
                max_army: 90,
                max_foundations: 3,
                farm_target: [4, 10, 16, 20],
                imperial_unit_upgrades: true,
                wonder: true,
                max_commands: 12,
            },
        }
    }
}

/// Difficulty dependent constants. Indexed arrays are per age (Dark, Feudal, Castle,
/// Imperial) unless stated otherwise.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Params {
    /// Seconds between two full decision passes.
    pub period: u32,
    /// Seconds between two army / production passes.
    pub military_period: u32,
    /// Villagers wanted per age.
    pub villager_target: [i32; 4],
    /// Villagers required before researching Feudal, Castle and Imperial.
    pub age_up_villagers: [i32; 3],
    /// Military population needed before the army leaves the base, per age.
    pub attack_threshold: [usize; 4],
    /// Never train more military units than this.
    pub max_army: usize,
    /// Non-house foundations allowed at the same time.
    pub max_foundations: usize,
    /// Farms wanted per age.
    pub farm_target: [i32; 4],
    /// Research Imperial Age unit-line upgrades.
    pub imperial_unit_upgrades: bool,
    /// Build a wonder late in the game.
    pub wonder: bool,
    /// Commands per think, hard cap.
    pub max_commands: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiPlayer {
    pub player: PlayerId,
    pub difficulty: Difficulty,
    // Internal memory lives here (build order index, last attack tick, scouting targets...).
    pub memory: AiMemory,
}

/// Everything the AI remembers between thinks. Serialisable so a saved game keeps the
/// computer players' state. Extra fields use `serde(default)` so old saves still load.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AiMemory {
    /// Tick of the last attack-move order.
    pub last_attack_tick: u32,
    pub build_step: usize,
    pub notes: Vec<String>,
    /// Largest army size seen during the current attack (for the retreat rule).
    #[serde(default)]
    pub army_peak: u32,
    /// The army is out attacking.
    #[serde(default)]
    pub attacking: bool,
    /// Current attack target (an enemy building or unit).
    #[serde(default)]
    pub attack_target: Option<EntityId>,
    /// No new attack before this tick (set after a retreat).
    #[serde(default)]
    pub retreat_until: u32,
    /// The starting scout and its progress along the exploration spiral.
    #[serde(default)]
    pub scout: Option<EntityId>,
    #[serde(default)]
    pub scout_step: u32,
    #[serde(default)]
    pub scout_done: bool,
    /// Villagers were sent into the town centre because of a raid.
    #[serde(default)]
    pub tc_garrisoned: bool,
    #[serde(default)]
    pub last_defend_tick: u32,
    #[serde(default)]
    pub last_rally_tick: u32,
    #[serde(default)]
    pub last_rebalance_tick: u32,
    #[serde(default)]
    pub last_market_tick: u32,
    #[serde(default)]
    pub last_repair_tick: u32,
    /// Watchdog: last known position of each working villager (entity index -> raw x,
    /// raw y, tick since when it has not moved) to catch villagers stuck on an
    /// unreachable node or foundation.
    #[serde(default)]
    pub watch: BTreeMap<u32, (i32, i32, u32)>,
    /// Nodes and foundations found unreachable, with the tick the ban expires.
    #[serde(default)]
    pub banned: BTreeMap<EntityId, u32>,
    /// Building types that could not be placed recently (type index -> tick), so
    /// the build order skips them for a while instead of stalling.
    #[serde(default)]
    pub place_failed: BTreeMap<u16, u32>,
}

/// Per-think scratch state shared by the passes: the world, the command list being
/// built and the resources still available after the commands already queued.
pub(crate) struct Ctx<'a> {
    pub world: &'a World,
    pub data: &'a GameData,
    pub me: PlayerId,
    pub params: Params,
    pub tick: u32,
    /// Game second of this think (`(tick + 1) / THINK_INTERVAL`).
    pub second: u32,
    pub cmds: Vec<Command>,
    /// Stockpile minus what the commands of this think will spend.
    pub available: Stockpile,
    /// Food/gold kept aside for the next age (0 when not saving up).
    pub reserve: Stockpile,
    /// Cost of the next building of the build order (plus a little wood for farm
    /// reseeding), kept away from military spending so the base keeps growing.
    pub build_reserve: Stockpile,
    /// Units that already received a command this think.
    pub busy: BTreeSet<EntityId>,
}

impl<'a> Ctx<'a> {
    /// Affordable after keeping both reserves (military units and research).
    pub fn can_afford(&self, cost: &Stockpile) -> bool {
        let mut left = self.available;
        left.sub(&self.reserve);
        left.sub(&self.build_reserve);
        left.covers(cost)
    }
    /// Affordable after keeping the age-up reserve only (farms, camps, buildings).
    pub fn can_afford_eco(&self, cost: &Stockpile) -> bool {
        let mut left = self.available;
        left.sub(&self.reserve);
        left.covers(cost)
    }
    /// Affordable after keeping the build reserve only (a minimal defensive army
    /// while saving up for an age).
    pub fn can_afford_defensive(&self, cost: &Stockpile) -> bool {
        let mut left = self.available;
        left.sub(&self.build_reserve);
        left.covers(cost)
    }
    /// Affordable ignoring the reserve (villagers, houses, the age-up itself).
    pub fn can_afford_essential(&self, cost: &Stockpile) -> bool {
        self.available.covers(cost)
    }
    pub fn pay(&mut self, cost: &Stockpile) {
        self.available.sub(cost);
    }
    pub fn push(&mut self, c: Command) {
        self.cmds.push(c);
    }
    pub fn mark_busy(&mut self, ids: &[EntityId]) {
        self.busy.extend(ids.iter().copied());
    }
}

impl AiPlayer {
    pub fn new(player: PlayerId, level: u8) -> AiPlayer {
        AiPlayer {
            player,
            difficulty: Difficulty::from_level(level),
            memory: AiMemory::default(),
        }
    }

    /// Decide what to do this second.
    pub fn think(&mut self, world: &World, data: &GameData) -> Vec<Command> {
        let params = self.difficulty.params();
        // `think` runs at ticks 19, 39, ...: the second about to be completed.
        let second = (world.tick + 1) / THINK_INTERVAL;
        if second % params.period != 0 {
            return Vec::new();
        }
        if self.player.idx() >= world.players.len() || !world.player(self.player).alive {
            return Vec::new();
        }
        let view = view::View::build(world, data, self.player, &mut self.memory);
        if view.villagers.is_empty() && view.buildings.is_empty() && view.military.is_empty() {
            return Vec::new();
        }
        let mut ctx = Ctx {
            world,
            data,
            me: self.player,
            params,
            tick: world.tick,
            second,
            cmds: Vec::new(),
            available: world.player(self.player).resources,
            reserve: Stockpile::ZERO,
            build_reserve: Stockpile::ZERO,
            busy: BTreeSet::new(),
        };
        ctx.reserve = economy::age_reserve(&ctx, &view);
        ctx.build_reserve = economy::build_reserve(&ctx, &view, &self.memory);

        // Order matters: the list is trimmed from the end, so emergencies and the
        // economy come first; the army repeats its orders anyway.
        military::defend(&mut ctx, &view, &mut self.memory);
        economy::core(&mut ctx, &view, &mut self.memory);
        economy::villagers(&mut ctx, &view, &mut self.memory);
        if second % params.military_period == 0 {
            military::run(&mut ctx, &view, &mut self.memory);
        }
        economy::extras(&mut ctx, &view, &mut self.memory);

        let max = params.max_commands;
        ctx.cmds.truncate(max);
        ctx.cmds
    }
}

/// Deterministic integer mixing of a few values (for tie-breaking and variety).
pub(crate) fn mix(a: u32, b: u32) -> u32 {
    let mut h = (a as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ ((b as u64) << 17);
    h ^= h >> 29;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 32;
    h as u32
}
