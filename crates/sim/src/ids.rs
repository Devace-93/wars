//! Identifiers and small enums shared by the simulation and the game data.

use serde::{Deserialize, Serialize};

/// Index of an object type (unit, building, resource) in `GameData::objects`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TypeId(pub u16);

/// Index of a technology in `GameData::techs`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TechId(pub u16);

/// Index of a civilisation in `GameData::civs`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct CivId(pub u8);

/// Player slot. 0 is Gaia (nature: trees, animals, relics); 1..=8 are players.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const GAIA: PlayerId = PlayerId(0);
    pub fn is_gaia(self) -> bool {
        self.0 == 0
    }
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

/// Entity handle: index + generation so stale handles never alias a new entity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct EntityId {
    pub index: u32,
    pub gen: u32,
}

impl EntityId {
    pub fn to_u64(self) -> u64 {
        ((self.gen as u64) << 32) | self.index as u64
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Resource {
    Food = 0,
    Wood = 1,
    Gold = 2,
    Stone = 3,
}

impl Resource {
    pub const ALL: [Resource; 4] = [
        Resource::Food,
        Resource::Wood,
        Resource::Gold,
        Resource::Stone,
    ];
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Resource::Food => "food",
            Resource::Wood => "wood",
            Resource::Gold => "gold",
            Resource::Stone => "stone",
        }
    }
}

/// A bundle of the four resources (costs, stockpiles, tributes).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Stockpile {
    pub food: i32,
    pub wood: i32,
    pub gold: i32,
    pub stone: i32,
}

impl Stockpile {
    pub const ZERO: Stockpile = Stockpile {
        food: 0,
        wood: 0,
        gold: 0,
        stone: 0,
    };
    pub const fn new(food: i32, wood: i32, gold: i32, stone: i32) -> Stockpile {
        Stockpile {
            food,
            wood,
            gold,
            stone,
        }
    }
    pub fn get(&self, r: Resource) -> i32 {
        match r {
            Resource::Food => self.food,
            Resource::Wood => self.wood,
            Resource::Gold => self.gold,
            Resource::Stone => self.stone,
        }
    }
    pub fn get_mut(&mut self, r: Resource) -> &mut i32 {
        match r {
            Resource::Food => &mut self.food,
            Resource::Wood => &mut self.wood,
            Resource::Gold => &mut self.gold,
            Resource::Stone => &mut self.stone,
        }
    }
    pub fn covers(&self, cost: &Stockpile) -> bool {
        self.food >= cost.food
            && self.wood >= cost.wood
            && self.gold >= cost.gold
            && self.stone >= cost.stone
    }
    pub fn add(&mut self, o: &Stockpile) {
        self.food += o.food;
        self.wood += o.wood;
        self.gold += o.gold;
        self.stone += o.stone;
    }
    pub fn sub(&mut self, o: &Stockpile) {
        self.food -= o.food;
        self.wood -= o.wood;
        self.gold -= o.gold;
        self.stone -= o.stone;
    }
    pub fn scaled(&self, pct: i32) -> Stockpile {
        Stockpile {
            food: self.food * pct / 100,
            wood: self.wood * pct / 100,
            gold: self.gold * pct / 100,
            stone: self.stone * pct / 100,
        }
    }
    pub fn total(&self) -> i32 {
        self.food + self.wood + self.gold + self.stone
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Age {
    Dark = 0,
    Feudal = 1,
    Castle = 2,
    Imperial = 3,
}

impl Age {
    pub const ALL: [Age; 4] = [Age::Dark, Age::Feudal, Age::Castle, Age::Imperial];
    pub fn next(self) -> Option<Age> {
        match self {
            Age::Dark => Some(Age::Feudal),
            Age::Feudal => Some(Age::Castle),
            Age::Castle => Some(Age::Imperial),
            Age::Imperial => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Age::Dark => "Dark Age",
            Age::Feudal => "Feudal Age",
            Age::Castle => "Castle Age",
            Age::Imperial => "Imperial Age",
        }
    }
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn from_idx(i: usize) -> Age {
        Age::ALL[i.min(3)]
    }
}

/// Armour classes used by the damage formula. A target only takes bonus damage of a
/// class it belongs to (i.e. has an armour entry for).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ArmourClass {
    Melee,
    Pierce,
    Infantry,
    Cavalry,
    Archer,
    Siege,
    Building,
    StandardBuilding,
    Castle,
    Wall,
    Ship,
    Camel,
    Spearman,
    Monk,
    UniqueUnit,
    Eagle,
    Ram,
    Elephant,
    Villager,
    Fishing,
    Gunpowder,
    CavalryArcher,
    Skirmisher,
    Tree,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Terrain {
    Grass = 0,
    Dirt = 1,
    Sand = 2,
    Forest = 3,
    Water = 4,
    Shallows = 5,
    DeepWater = 6,
    Snow = 7,
    Road = 8,
    Cliff = 9,
    Farmland = 10,
    Beach = 11,
}

impl Terrain {
    pub fn is_water(self) -> bool {
        matches!(self, Terrain::Water | Terrain::DeepWater)
    }
    pub fn land_passable(self) -> bool {
        !matches!(self, Terrain::Water | Terrain::DeepWater | Terrain::Cliff)
    }
    pub fn water_passable(self) -> bool {
        matches!(
            self,
            Terrain::Water | Terrain::DeepWater | Terrain::Shallows
        )
    }
    pub fn buildable(self) -> bool {
        matches!(
            self,
            Terrain::Grass
                | Terrain::Dirt
                | Terrain::Sand
                | Terrain::Snow
                | Terrain::Road
                | Terrain::Beach
        )
    }
    pub fn name(self) -> &'static str {
        match self {
            Terrain::Grass => "grass",
            Terrain::Dirt => "dirt",
            Terrain::Sand => "sand",
            Terrain::Forest => "forest",
            Terrain::Water => "water",
            Terrain::Shallows => "shallows",
            Terrain::DeepWater => "deep water",
            Terrain::Snow => "snow",
            Terrain::Road => "road",
            Terrain::Cliff => "cliff",
            Terrain::Farmland => "farmland",
            Terrain::Beach => "beach",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Diplomacy {
    Ally,
    Neutral,
    Enemy,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum Stance {
    #[default]
    Aggressive,
    Defensive,
    StandGround,
    NoAttack,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum Formation {
    #[default]
    Line,
    Box,
    Staggered,
    Flank,
}
