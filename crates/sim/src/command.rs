//! Player commands: the only way anything changes in the world. Commands are produced
//! by the clients and the AI, ordered by the server (or the local runner) and applied
//! at a fixed tick by every participant in lockstep.

use crate::fixed::Vec2;
use crate::ids::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Move the selected units to a point. `queue` appends after the current task.
    Move {
        units: Vec<EntityId>,
        to: Vec2,
        queue: bool,
    },
    /// Move, attacking anything hostile on the way.
    AttackMove {
        units: Vec<EntityId>,
        to: Vec2,
        queue: bool,
    },
    /// Attack a specific target (unit, building or animal).
    Attack {
        units: Vec<EntityId>,
        target: EntityId,
        queue: bool,
    },
    Patrol {
        units: Vec<EntityId>,
        to: Vec2,
    },
    Follow {
        units: Vec<EntityId>,
        target: EntityId,
    },
    Guard {
        units: Vec<EntityId>,
        target: EntityId,
    },
    Stop {
        units: Vec<EntityId>,
    },
    /// Villagers gather from a resource (tree, pile, bush, animal, farm, fish).
    Gather {
        units: Vec<EntityId>,
        target: EntityId,
        queue: bool,
    },
    /// Place a building foundation at tile (x, y) and send the villagers to build it.
    Build {
        units: Vec<EntityId>,
        building: TypeId,
        x: i32,
        y: i32,
        queue: bool,
    },
    /// Send villagers to work on an existing foundation / repair a damaged building or siege.
    Repair {
        units: Vec<EntityId>,
        target: EntityId,
        queue: bool,
    },
    /// Queue a unit at a building.
    Train {
        building: EntityId,
        unit: TypeId,
    },
    /// Queue a technology at a building.
    Research {
        building: EntityId,
        tech: TechId,
    },
    /// Remove queue entry `index` (0 = currently in progress) and refund it.
    CancelQueue {
        building: EntityId,
        index: usize,
    },
    /// Cancel a foundation (refund) or delete the object.
    Delete {
        entity: EntityId,
    },
    SetRally {
        buildings: Vec<EntityId>,
        to: RallyTarget,
    },
    Garrison {
        units: Vec<EntityId>,
        target: EntityId,
    },
    /// Eject every garrisoned unit (or the given ones).
    Ungarrison {
        building: EntityId,
    },
    SetStance {
        units: Vec<EntityId>,
        stance: Stance,
    },
    SetFormation {
        units: Vec<EntityId>,
        formation: Formation,
    },
    /// Monks: convert a target; heal is automatic.
    Convert {
        units: Vec<EntityId>,
        target: EntityId,
    },
    /// Trade carts / cogs go to and from an allied market/dock.
    Trade {
        units: Vec<EntityId>,
        target: EntityId,
    },
    /// Market: buy 100 of `resource` with gold or sell 100 for gold.
    MarketBuy {
        resource: Resource,
    },
    MarketSell {
        resource: Resource,
    },
    Tribute {
        to: PlayerId,
        amount: Stockpile,
    },
    SetDiplomacy {
        with: PlayerId,
        stance: Diplomacy,
    },
    /// Unload a transport (at its current position) or drop a relic.
    Unload {
        units: Vec<EntityId>,
    },
    /// Pack/unpack a trebuchet.
    TogglePack {
        units: Vec<EntityId>,
    },
    Resign,
    /// Cheats (only applied when the match allows them).
    Cheat {
        code: String,
    },
    /// Scenario/editor: spawn an object directly (used by the editor and triggers).
    Spawn {
        owner: PlayerId,
        object: TypeId,
        x: i32,
        y: i32,
    },
    /// Chat is not a sim command but travels in the same stream so replays keep it.
    Chat {
        text: String,
        to_allies: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RallyTarget {
    Point(Vec2),
    Entity(EntityId),
    None,
}

/// A command with its issuing player, as scheduled by the server for a tick.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PlayerCommand {
    pub player: PlayerId,
    pub command: Command,
}
