//! Scenario triggers: conditions and effects evaluated every tick, in order.

use crate::fixed::Vec2;
use crate::ids::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Area {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Area {
    pub fn contains(&self, p: Vec2) -> bool {
        let (x, y) = p.tile();
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Condition {
    /// Game time in seconds reached.
    Timer {
        seconds: u32,
    },
    /// Player owns at least `count` objects of a type (None = any).
    OwnObjects {
        player: PlayerId,
        object: Option<TypeId>,
        count: i32,
    },
    OwnFewerObjects {
        player: PlayerId,
        object: Option<TypeId>,
        count: i32,
    },
    /// Player has at least `count` objects (any type) inside the area.
    ObjectsInArea {
        player: PlayerId,
        area: Area,
        count: i32,
    },
    /// Specific entity destroyed (looked up by scenario object index).
    DestroyObject {
        object: u32,
    },
    /// Player stockpile of a resource at least `amount`.
    AccumulateAttribute {
        player: PlayerId,
        resource: Resource,
        amount: i32,
    },
    ResearchedTech {
        player: PlayerId,
        tech: TechId,
    },
    PlayerDefeated {
        player: PlayerId,
    },
    /// Other trigger fired at least once.
    TriggerFired {
        trigger: u32,
    },
    /// Random chance percent, evaluated once per tick while the trigger is active.
    Chance {
        pct: i32,
    },
    /// An object of the scenario reached the area.
    BringObjectToArea {
        object: u32,
        area: Area,
    },
    /// Player reached the age.
    InAge {
        player: PlayerId,
        age: Age,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TriggerEffect {
    SendChat {
        player: Option<PlayerId>,
        text: String,
    },
    DisplayInstructions {
        player: Option<PlayerId>,
        text: String,
    },
    ChangeDiplomacy {
        source: PlayerId,
        target: PlayerId,
        stance: Diplomacy,
    },
    ResearchTech {
        player: PlayerId,
        tech: TechId,
    },
    Tribute {
        from: PlayerId,
        to: PlayerId,
        amount: Stockpile,
    },
    /// Add resources from nowhere (negative to take away).
    ChangeResources {
        player: PlayerId,
        amount: Stockpile,
    },
    CreateObject {
        player: PlayerId,
        object: TypeId,
        x: i32,
        y: i32,
    },
    /// Kill every object of `player` (optionally only of a type) inside an area.
    KillObjects {
        player: PlayerId,
        object: Option<TypeId>,
        area: Option<Area>,
    },
    RemoveObjects {
        player: PlayerId,
        object: Option<TypeId>,
        area: Option<Area>,
    },
    ChangeOwnership {
        from: PlayerId,
        to: PlayerId,
        object: Option<TypeId>,
        area: Option<Area>,
    },
    DeclareVictory {
        player: PlayerId,
    },
    DeclareDefeat {
        player: PlayerId,
    },
    ActivateTrigger {
        trigger: u32,
    },
    DeactivateTrigger {
        trigger: u32,
    },
    /// Move the player's camera (clients honour it).
    ChangeView {
        player: PlayerId,
        x: i32,
        y: i32,
    },
    /// Task units of the player in an area to move to a point.
    TaskObjects {
        player: PlayerId,
        area: Area,
        to_x: i32,
        to_y: i32,
    },
    /// Set an object's HP (percent of max) for everything matching.
    DamageObjects {
        player: PlayerId,
        area: Area,
        amount: i32,
    },
    /// Change the player's age without research.
    ChangeAge {
        player: PlayerId,
        age: Age,
    },
    PlaySound {
        player: Option<PlayerId>,
        sound: String,
    },
    Unlock {
        player: PlayerId,
        object: TypeId,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub enabled: bool,
    pub looping: bool,
    pub conditions: Vec<Condition>,
    pub effects: Vec<TriggerEffect>,
    /// Show in the objectives panel.
    pub objective: bool,
    pub description: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TriggerState {
    pub active: bool,
    pub fired: u32,
    pub done: bool,
}

/// Mapping from scenario object indexes (used by editor triggers) to live entities.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct ObjectRefs {
    pub refs: Vec<Option<EntityId>>,
}
