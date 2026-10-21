//! Scenario files: a map, the objects on it, player setup and triggers. The editor
//! saves these; the server stores them; `to_world` turns one into a playable world.

use serde::{Deserialize, Serialize};
use wars_sim::ids::*;
use wars_sim::map::Map;
use wars_sim::trigger::Trigger;
use wars_sim::types::{GameData, ObjectClass};
use wars_sim::world::{Settings, World};
use wars_sim::Vec2;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ScenarioObject {
    pub owner: u8,
    pub key: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ScenarioPlayer {
    pub name: String,
    pub civ: String,
    pub team: u8,
    pub is_ai: bool,
    pub ai_level: u8,
    pub resources: Stockpile,
    pub age: Age,
    pub active: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub version: u32,
    pub name: String,
    pub author: String,
    pub description: String,
    pub instructions: String,
    pub map: Map,
    pub objects: Vec<ScenarioObject>,
    pub players: Vec<ScenarioPlayer>,
    pub triggers: Vec<Trigger>,
    pub settings: Settings,
}

impl Scenario {
    pub fn blank(name: &str, size: i32) -> Scenario {
        let mut map = Map::new(size, size, Terrain::Grass);
        for t in map.tiles.iter_mut() {
            t.variant = 0;
        }
        let players = (1..=8)
            .map(|i| ScenarioPlayer {
                name: format!("Player {i}"),
                civ: [
                    "britons",
                    "franks",
                    "mongols",
                    "teutons",
                    "japanese",
                    "saracens",
                    "celts",
                    "byzantines",
                ][i - 1]
                    .into(),
                team: i as u8,
                is_ai: i > 1,
                ai_level: 1,
                resources: Stockpile::new(200, 200, 100, 200),
                age: Age::Dark,
                active: i <= 2,
            })
            .collect();
        Scenario {
            version: FORMAT_VERSION,
            name: name.into(),
            author: String::new(),
            description: String::new(),
            instructions: String::new(),
            map,
            objects: Vec::new(),
            players,
            triggers: Vec::new(),
            settings: Settings {
                mode: wars_sim::world::GameMode::Scenario,
                ..Settings::default()
            },
        }
    }

    /// Capture the current world into a scenario (what the editor does on save).
    pub fn from_world(world: &World, data: &GameData, name: &str) -> Scenario {
        let mut sc = Scenario::blank(name, world.map.width);
        sc.map = world.map.clone();
        for t in sc.map.occupancy.iter_mut() {
            *t = None;
        }
        sc.objects = world
            .iter()
            .map(|e| {
                let (x, y, _, _) = world.footprint(data, e);
                ScenarioObject {
                    owner: e.owner.0,
                    key: data.object(e.ty).key.clone(),
                    x,
                    y,
                }
            })
            .collect();
        sc.players = world
            .players
            .iter()
            .skip(1)
            .map(|p| ScenarioPlayer {
                name: p.name.clone(),
                civ: data.civ(p.civ).key.clone(),
                team: p.team,
                is_ai: p.is_ai,
                ai_level: p.ai_level,
                resources: p.resources,
                age: p.age,
                active: true,
            })
            .collect();
        while sc.players.len() < 8 {
            let i = sc.players.len() + 1;
            sc.players.push(ScenarioPlayer {
                name: format!("Player {i}"),
                civ: "britons".into(),
                team: i as u8,
                is_ai: true,
                ai_level: 1,
                resources: Stockpile::new(200, 200, 100, 200),
                age: Age::Dark,
                active: false,
            });
        }
        sc.triggers = world.triggers.clone();
        sc.settings = world.settings.clone();
        sc
    }

    pub fn to_world(&self, data: &GameData, seed: u64) -> World {
        let mut map = self.map.clone();
        map.occupancy = vec![None; (map.width * map.height) as usize];
        let mut world = World::new(map, seed, self.settings.clone());
        world.scenario_name = self.name.clone();
        world.add_player("Gaia", CivId(0), 0, false, Stockpile::ZERO);
        let active: Vec<&ScenarioPlayer> = self.players.iter().filter(|p| p.active).collect();
        for p in &active {
            let civ = data.civ_id(&p.civ).unwrap_or(CivId(0));
            let pid = world.add_player(&p.name, civ, p.team, p.is_ai, p.resources);
            world.player_mut(pid).ai_level = p.ai_level;
            for (tech, age) in [
                ("feudal_age", Age::Feudal),
                ("castle_age", Age::Castle),
                ("imperial_age", Age::Imperial),
            ] {
                if p.age >= age {
                    wars_sim::unit::grant_tech(&mut world, data, pid, data.tk(tech));
                }
            }
            world.player_mut(pid).age = p.age;
        }
        // Map scenario player slots (1..=8) to live player ids (only active ones exist).
        let mut slot_to_pid = [PlayerId::GAIA; 9];
        let mut next = 1u8;
        for (i, p) in self.players.iter().enumerate() {
            if p.active {
                slot_to_pid[i + 1] = PlayerId(next);
                next += 1;
            }
        }
        world.object_refs = Vec::with_capacity(self.objects.len());
        for o in &self.objects {
            let Some(ty) = data.object_id(&o.key) else {
                world.object_refs.push(None);
                continue;
            };
            let owner = slot_to_pid
                .get(o.owner as usize)
                .copied()
                .unwrap_or(PlayerId::GAIA);
            if o.owner != 0 && owner.is_gaia() {
                world.object_refs.push(None);
                continue;
            }
            let def = data.object(ty);
            let pos = if matches!(def.class, ObjectClass::Unit | ObjectClass::Animal) {
                Vec2::tile_center(o.x, o.y)
            } else {
                Vec2::from_tile(o.x, o.y)
            };
            let id = world.spawn(data, owner, ty, pos, false);
            world.object_refs.push(id);
        }
        world.triggers = self.triggers.clone();
        world.total_relics = world
            .iter()
            .filter(|e| data.object(e.ty).key == "relic")
            .count() as i32;
        wars_sim::vision::update_vision(&mut world, data);
        world
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("scenario serialises")
    }
    pub fn from_bytes(b: &[u8]) -> Result<Scenario, postcard::Error> {
        postcard::from_bytes(b)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("scenario serialises")
    }
    pub fn from_json(s: &str) -> Result<Scenario, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_play() {
        let data = wars_data::data();
        let params = crate::MapParams {
            size: crate::MapSize::Tiny,
            ..Default::default()
        };
        let world = crate::generate(data, &params);
        let sc = Scenario::from_world(&world, data, "Test");
        let bytes = sc.to_bytes();
        let sc2 = Scenario::from_bytes(&bytes).unwrap();
        assert_eq!(sc, sc2);
        let json = sc.to_json();
        let sc3 = Scenario::from_json(&json).unwrap();
        assert_eq!(sc.objects.len(), sc3.objects.len());
        let w2 = sc2.to_world(data, 5);
        assert_eq!(w2.count(), world.count());
        let mut w2 = w2;
        for _ in 0..100 {
            wars_sim::step::step(&mut w2, data, &[]);
        }
    }
}
