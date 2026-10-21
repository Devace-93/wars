//! wars-ai: the scripted computer player.
//!
//! `AiPlayer::think` is called by the match runner every `THINK_INTERVAL` ticks with a
//! read-only view of the world and returns the commands the AI wants to issue. It must
//! be deterministic: no randomness outside `world.rng`-independent integer logic (use
//! the tick and entity ids for variety), because every participant runs the AI for
//! offline matches and the server runs it for online ones.

use serde::{Deserialize, Serialize};
use wars_sim::command::Command;
use wars_sim::ids::PlayerId;
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiPlayer {
    pub player: PlayerId,
    pub difficulty: Difficulty,
    // Internal memory lives here (build order index, last attack tick, scouting targets...).
    pub memory: AiMemory,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AiMemory {
    pub last_attack_tick: u32,
    pub build_step: usize,
    pub notes: Vec<String>,
}

impl AiPlayer {
    pub fn new(player: PlayerId, level: u8) -> AiPlayer {
        AiPlayer {
            player,
            difficulty: Difficulty::from_level(level),
            memory: AiMemory::default(),
        }
    }

    /// Decide what to do this second. Filled in by the AI milestone.
    pub fn think(&mut self, world: &World, data: &GameData) -> Vec<Command> {
        let _ = (world, data);
        Vec::new()
    }
}
