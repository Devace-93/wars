//! The game-data schema: object types (units, buildings, resources), technologies,
//! civilisations and the `GameData` table that `wars-data` fills in.

use crate::fixed::Fixed;
use crate::ids::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ObjectClass {
    /// Civilian or military unit.
    Unit,
    /// Any building, including walls, gates and farms.
    Building,
    /// Gaia resource: tree, gold/stone pile, berry bush, fish.
    Resource,
    /// Huntable animal (also a food resource once attacked).
    Animal,
    /// Relic (carried by monks, stored in a monastery).
    Relic,
}

/// Broad unit role, mainly for the AI, the UI and the stat cache.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Role {
    Villager,
    Infantry,
    Archer,
    Cavalry,
    CavalryArcher,
    Siege,
    Monk,
    Trade,
    Naval,
    Transport,
    Fishing,
    Scout,
    King,
    Hero,
    Economic,
    Military,
    Defensive,
    Wall,
    Gate,
    Housing,
    DropOff,
    Wonder,
    None,
}

/// What a resource object yields.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Yield {
    pub resource: Resource,
    pub amount: i32,
    /// Gather rate class: which villager gather rate applies.
    pub kind: GatherKind,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum GatherKind {
    Berries,
    Sheep,
    Hunt,
    Farm,
    Fish,
    Wood,
    Gold,
    Stone,
}

impl GatherKind {
    pub fn resource(self) -> Resource {
        match self {
            GatherKind::Berries
            | GatherKind::Sheep
            | GatherKind::Hunt
            | GatherKind::Farm
            | GatherKind::Fish => Resource::Food,
            GatherKind::Wood => Resource::Wood,
            GatherKind::Gold => Resource::Gold,
            GatherKind::Stone => Resource::Stone,
        }
    }
    pub const ALL: [GatherKind; 8] = [
        GatherKind::Berries,
        GatherKind::Sheep,
        GatherKind::Hunt,
        GatherKind::Farm,
        GatherKind::Fish,
        GatherKind::Wood,
        GatherKind::Gold,
        GatherKind::Stone,
    ];
}

/// Attack entry: damage of a given armour class.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Attack {
    pub class: ArmourClass,
    pub value: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Armour {
    pub class: ArmourClass,
    pub value: i32,
}

/// Static definition of a unit, building or resource. All times in game seconds,
/// speeds in tiles per second, ranges in tiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ObjectType {
    pub key: String,
    pub name: String,
    pub class: ObjectClass,
    pub role: Role,
    pub hp: i32,
    pub attacks: Vec<Attack>,
    pub armours: Vec<Armour>,
    /// Attack range in tiles (0 = melee).
    pub range: Fixed,
    pub min_range: Fixed,
    pub los: Fixed,
    pub speed: Fixed,
    /// Seconds between attacks.
    pub reload: Fixed,
    /// Projectile travel: 0 for melee, else tiles per second (damage arrives after flight).
    pub projectile_speed: Fixed,
    /// Hit chance on moving targets without ballistics (percent). 100 = always.
    pub accuracy: i32,
    pub cost: Stockpile,
    pub train_time: i32,
    pub population: i32,
    /// Population headroom this object provides (houses, town centre, castle).
    pub pop_support: i32,
    /// Footprint in tiles (units: 1x1; buildings: w x h).
    pub size: (u8, u8),
    pub garrison_capacity: i32,
    /// Arrows fired by a garrisoned building per unit inside (0 = none).
    pub garrison_arrows: i32,
    /// Which building (key) trains/builds this object. Empty for Gaia things and buildings
    /// (villagers build them).
    pub trained_at: Option<String>,
    pub age: Age,
    /// Required building keys (e.g. a stable needs a barracks).
    pub requires: Vec<String>,
    /// Required tech key (e.g. unique units need a castle only; champion needs Champion tech).
    pub requires_tech: Option<String>,
    /// Resource contents for resource/animal objects.
    pub yields: Option<Yield>,
    /// Drop-off point for these gather kinds (mill, lumber camp, mining camp, TC, dock).
    pub drop_off: Vec<GatherKind>,
    /// Can gather, build and repair.
    pub is_villager: bool,
    pub can_convert: bool,
    pub can_heal: bool,
    pub is_naval: bool,
    pub carries_units: i32,
    /// True if units can walk through (farms, gates for allies).
    pub walkable: bool,
    /// Buildings' construction time in seconds (same field as train_time for buildings).
    pub heal_rate: i32,
    /// Conversion resistance in "faith" seconds (monks, siege are harder).
    pub conversion_resist: i32,
    /// Research performed here (tech keys) — resolved into `GameData::techs_at`.
    pub trains: Vec<String>,
    /// Upgrade chain target (e.g. militia -> man-at-arms) via tech key.
    pub upgrades_to: Option<(String, String)>,
    /// Trade: gold per trip base (markets/docks).
    pub is_market: bool,
    pub is_dock: bool,
    pub is_monastery: bool,
    pub is_town_center: bool,
    pub is_wonder: bool,
    pub is_king: bool,
    /// Terrain the object needs: land, water or shore (docks).
    pub placement: Placement,
    /// Glyph for the console client and a palette index for procedural art.
    pub glyph: char,
    pub art: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Placement {
    Land,
    Water,
    Shore,
    Any,
}

/// Effects applied by technologies, civ bonuses and age advances.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Effect {
    /// Add `value` to an attribute of all matching object types.
    Add {
        target: Target,
        attr: Attr,
        value: i32,
    },
    /// Multiply an attribute by `pct` percent (e.g. 110 = +10%).
    Mul {
        target: Target,
        attr: Attr,
        pct: i32,
    },
    /// Replace all objects of `from` type with `to` (unit line upgrades).
    Upgrade { from: String, to: String },
    /// Make an object type available (e.g. civ tech trees, age unlocks).
    Enable { object: String },
    /// Remove an object type from availability.
    Disable { object: String },
    /// Multiply a gather rate by `pct`.
    GatherRate { kind: Option<GatherKind>, pct: i32 },
    /// Add to villager carry capacity.
    CarryCapacity { value: i32 },
    /// Reduce the cost of an object by `pct` percent for the given resource (None = all).
    CostDiscount {
        target: Target,
        resource: Option<Resource>,
        pct: i32,
    },
    /// Tech research cost discount.
    TechDiscount { pct: i32, building: Option<String> },
    /// Work rate (build/research/train speed) multiplier.
    WorkRate { target: Target, pct: i32 },
    /// Population cap bonus.
    PopCap { value: i32 },
    /// Free resources once (civ bonus or cheat).
    Resources {
        food: i32,
        wood: i32,
        gold: i32,
        stone: i32,
    },
    /// Make a tech free (Mongols style, etc.).
    FreeTech { tech: String },
    /// Market trade fee percent (default 30).
    TradeFee { pct: i32 },
    /// Relic gold rate percent multiplier.
    RelicGold { pct: i32 },
    /// Conversion resistance bonus for all units of the player.
    ConversionResist { value: i32 },
    /// Monk heal range/rate bonus etc. kept generic through Attr.
    AdvanceAge,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Target {
    Object(String),
    Role(Role),
    Class(ObjectClass),
    ArmourClass(ArmourClass),
    All,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Attr {
    Hp,
    MeleeAttack,
    PierceAttack,
    MeleeArmour,
    PierceArmour,
    /// Bonus attack vs a class, encoded via `Add` with the class in `BonusAttack`.
    BonusAttack(ArmourClass),
    BonusArmour(ArmourClass),
    Range,
    Los,
    /// Speed as percent (use Mul).
    Speed,
    ReloadTime,
    Accuracy,
    TrainTime,
    GarrisonCapacity,
    Population,
    PopSupport,
    HealRate,
    ConversionResist,
    ProjectileSpeed,
    MinRange,
    Ballistics,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TechDef {
    pub key: String,
    pub name: String,
    pub cost: Stockpile,
    pub time: i32,
    pub age: Age,
    /// Building key where it is researched; None = automatic (civ bonus / age effect).
    pub building: Option<String>,
    pub requires: Vec<String>,
    pub effects: Vec<Effect>,
    /// Age-up techs set this.
    pub advances_to: Option<Age>,
    pub description: String,
    pub unique_to: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CivDef {
    pub key: String,
    pub name: String,
    pub description: String,
    pub bonuses: Vec<Effect>,
    pub bonus_text: Vec<String>,
    pub team_bonus: Vec<Effect>,
    pub team_bonus_text: String,
    pub unique_unit: String,
    pub unique_unit_elite: String,
    pub unique_techs: Vec<String>,
    /// Techs and objects this civ cannot get.
    pub disabled_techs: Vec<String>,
    pub disabled_objects: Vec<String>,
    /// Language/voice flavour for the UI.
    pub language: String,
}

/// Everything static about the game. Built once at startup by `wars-data`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameData {
    pub objects: Vec<ObjectType>,
    pub techs: Vec<TechDef>,
    pub civs: Vec<CivDef>,
    pub object_index: BTreeMap<String, TypeId>,
    pub tech_index: BTreeMap<String, TechId>,
    pub civ_index: BTreeMap<String, CivId>,
    /// Trained units per building type.
    pub trains_at: BTreeMap<TypeId, Vec<TypeId>>,
    /// Techs researched per building type.
    pub techs_at: BTreeMap<TypeId, Vec<TechId>>,
    /// Buildings villagers can construct, in UI order.
    pub buildable: Vec<TypeId>,
    /// Base gather rates (resource units per second) by kind, for villagers.
    pub gather_rates: BTreeMap<GatherKind, Fixed>,
    pub carry_capacity: i32,
    pub base_pop_cap: i32,
    pub market_fee_pct: i32,
    pub relic_gold_per_minute: i32,
    pub wonder_countdown_secs: i32,
    pub relic_countdown_secs: i32,
}

impl GameData {
    pub fn object(&self, id: TypeId) -> &ObjectType {
        &self.objects[id.0 as usize]
    }
    pub fn tech(&self, id: TechId) -> &TechDef {
        &self.techs[id.0 as usize]
    }
    pub fn civ(&self, id: CivId) -> &CivDef {
        &self.civs[id.0 as usize]
    }
    pub fn object_id(&self, key: &str) -> Option<TypeId> {
        self.object_index.get(key).copied()
    }
    pub fn tech_id(&self, key: &str) -> Option<TechId> {
        self.tech_index.get(key).copied()
    }
    pub fn civ_id(&self, key: &str) -> Option<CivId> {
        self.civ_index.get(key).copied()
    }
    /// Panics on an unknown key: data bugs should fail loudly in tests.
    pub fn obj(&self, key: &str) -> TypeId {
        self.object_id(key)
            .unwrap_or_else(|| panic!("unknown object key {key}"))
    }
    pub fn tk(&self, key: &str) -> TechId {
        self.tech_id(key)
            .unwrap_or_else(|| panic!("unknown tech key {key}"))
    }
    /// Rebuild the indexes after `objects`, `techs` and `civs` are filled.
    pub fn finish(&mut self) {
        self.object_index.clear();
        for (i, o) in self.objects.iter().enumerate() {
            self.object_index.insert(o.key.clone(), TypeId(i as u16));
        }
        self.tech_index.clear();
        for (i, t) in self.techs.iter().enumerate() {
            self.tech_index.insert(t.key.clone(), TechId(i as u16));
        }
        self.civ_index.clear();
        for (i, c) in self.civs.iter().enumerate() {
            self.civ_index.insert(c.key.clone(), CivId(i as u8));
        }
        self.trains_at.clear();
        self.techs_at.clear();
        self.buildable.clear();
        for (i, o) in self.objects.iter().enumerate() {
            let id = TypeId(i as u16);
            if let Some(b) = &o.trained_at {
                let bid = self.object_index[b];
                self.trains_at.entry(bid).or_default().push(id);
            }
            if o.class == ObjectClass::Building && o.trained_at.is_none() && o.cost.total() > 0 {
                self.buildable.push(id);
            }
        }
        for (i, t) in self.techs.iter().enumerate() {
            if let Some(b) = &t.building {
                let bid = self.object_index[b];
                self.techs_at.entry(bid).or_default().push(TechId(i as u16));
            }
        }
    }
    /// Validate every cross reference; returns the list of problems.
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        for o in &self.objects {
            if let Some(b) = &o.trained_at {
                if !self.object_index.contains_key(b) {
                    errs.push(format!("{}: trained_at {b} unknown", o.key));
                }
            }
            for r in &o.requires {
                if !self.object_index.contains_key(r) {
                    errs.push(format!("{}: requires {r} unknown", o.key));
                }
            }
            if let Some(t) = &o.requires_tech {
                if !self.tech_index.contains_key(t) {
                    errs.push(format!("{}: requires_tech {t} unknown", o.key));
                }
            }
            if let Some((t, to)) = &o.upgrades_to {
                if !self.tech_index.contains_key(t) {
                    errs.push(format!("{}: upgrade tech {t} unknown", o.key));
                }
                if !self.object_index.contains_key(to) {
                    errs.push(format!("{}: upgrade target {to} unknown", o.key));
                }
            }
        }
        for t in &self.techs {
            if let Some(b) = &t.building {
                if !self.object_index.contains_key(b) {
                    errs.push(format!("tech {}: building {b} unknown", t.key));
                }
            }
            for r in &t.requires {
                if !self.tech_index.contains_key(r) && !self.object_index.contains_key(r) {
                    errs.push(format!("tech {}: requires {r} unknown", t.key));
                }
            }
            for e in &t.effects {
                errs.extend(self.validate_effect(&t.key, e));
            }
        }
        for c in &self.civs {
            for k in [&c.unique_unit, &c.unique_unit_elite] {
                if !self.object_index.contains_key(k) {
                    errs.push(format!("civ {}: unique unit {k} unknown", c.key));
                }
            }
            for t in &c.unique_techs {
                if !self.tech_index.contains_key(t) {
                    errs.push(format!("civ {}: unique tech {t} unknown", c.key));
                }
            }
            for t in &c.disabled_techs {
                if !self.tech_index.contains_key(t) {
                    errs.push(format!("civ {}: disabled tech {t} unknown", c.key));
                }
            }
            for o in &c.disabled_objects {
                if !self.object_index.contains_key(o) {
                    errs.push(format!("civ {}: disabled object {o} unknown", c.key));
                }
            }
            for e in c.bonuses.iter().chain(c.team_bonus.iter()) {
                errs.extend(self.validate_effect(&c.key, e));
            }
        }
        errs
    }
    fn validate_effect(&self, owner: &str, e: &Effect) -> Vec<String> {
        let mut errs = Vec::new();
        let check_obj = |k: &str, errs: &mut Vec<String>| {
            if !self.object_index.contains_key(k) {
                errs.push(format!("{owner}: effect references unknown object {k}"));
            }
        };
        match e {
            Effect::Add { target, .. }
            | Effect::Mul { target, .. }
            | Effect::WorkRate { target, .. } => {
                if let Target::Object(k) = target {
                    check_obj(k, &mut errs);
                }
            }
            Effect::CostDiscount { target, .. } => {
                if let Target::Object(k) = target {
                    check_obj(k, &mut errs);
                }
            }
            Effect::Upgrade { from, to } => {
                check_obj(from, &mut errs);
                check_obj(to, &mut errs);
            }
            Effect::Enable { object } | Effect::Disable { object } => check_obj(object, &mut errs),
            Effect::FreeTech { tech } => {
                if !self.tech_index.contains_key(tech) {
                    errs.push(format!("{owner}: free tech {tech} unknown"));
                }
            }
            Effect::TechDiscount {
                building: Some(b), ..
            } => check_obj(b, &mut errs),
            _ => {}
        }
        errs
    }
}
