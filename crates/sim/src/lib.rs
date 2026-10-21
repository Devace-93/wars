#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::manual_is_multiple_of,
    clippy::field_reassign_with_default,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::if_same_then_else,
    clippy::needless_range_loop,
    clippy::option_map_unit_fn,
    clippy::comparison_chain,
    clippy::match_like_matches_macro
)]
//! wars-sim: the deterministic simulation shared by every Wars surface.
//!
//! Rules: no floats in game logic, one seeded RNG, stable iteration order, and every
//! participant applies the same commands at the same tick.

pub mod combat;
pub mod command;
pub mod fixed;
pub mod ids;
pub mod map;
pub mod path;
pub mod rng;
pub mod stats;
pub mod step;
pub mod trigger;
pub mod trigger_exec;
pub mod types;
pub mod unit;
pub mod vision;
pub mod world;

pub use command::{Command, PlayerCommand, RallyTarget};
pub use fixed::{Fixed, Vec2};
pub use ids::*;
pub use map::{Map, Tile};
pub use types::*;
pub use world::{
    Entity, Event, GameMode, Player, QueueItem, Settings, Task, VictoryMode, World,
    TICKS_PER_SECOND,
};
