//! Headless AI vs AI match on Arabia Tiny: prints one status line per game minute and
//! stops at game over or after 40 game minutes.
//!
//! Run with `cargo run -p wars-ai --example botmatch [seed] [level]` (level 0..2).

use wars_client::LocalMatch;
use wars_mapgen::{generate, MapParams, MapSize, MapType, PlayerSetup};
use wars_sim::ids::{PlayerId, Resource};
use wars_sim::types::ObjectClass;
use wars_sim::world::{World, TICKS_PER_SECOND};

const MAX_MINUTES: u32 = 40;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(7);
    let level: u8 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let data = wars_data::data();
    let params = MapParams {
        map_type: MapType::Arabia,
        size: MapSize::Tiny,
        seed,
        players: vec![
            PlayerSetup {
                name: "Blue".into(),
                civ: "britons".into(),
                team: 1,
                is_ai: true,
                ai_level: level,
            },
            PlayerSetup {
                name: "Red".into(),
                civ: "franks".into(),
                team: 2,
                is_ai: true,
                ai_level: level,
            },
        ],
        ..MapParams::default()
    };
    let world = generate(data, &params);
    let mut m = LocalMatch::new(world, PlayerId(1));
    let start = std::time::Instant::now();
    for minute in 1..=MAX_MINUTES {
        for _ in 0..60 * TICKS_PER_SECOND {
            m.step_once(data);
            if m.world.game_over {
                break;
            }
        }
        println!("{}", status(&m.world, minute, data));
        if std::env::var_os("WARS_AI_DEBUG").is_some() {
            debug(&m.world, data);
        }
        if m.world.game_over {
            break;
        }
    }
    let w = &m.world;
    let winner = match w.winner_team {
        Some(t) => w
            .players
            .iter()
            .find(|p| !p.id.is_gaia() && p.team == t)
            .map(|p| format!("{} (team {t})", p.name))
            .unwrap_or_else(|| format!("team {t}")),
        None => "nobody (time limit)".to_string(),
    };
    println!(
        "RESULT tick {} ({}m{:02}s): winner {}; {} commands logged; {:.1}s wall",
        w.tick,
        w.seconds() / 60,
        w.seconds() % 60,
        winner,
        m.log.len(),
        start.elapsed().as_secs_f64()
    );
}

fn status(w: &World, minute: u32, data: &wars_sim::types::GameData) -> String {
    let mut s = format!("min {minute:2} tick {:6}", w.tick);
    for p in w.players.iter().filter(|p| !p.id.is_gaia()) {
        let mut vils = 0;
        let mut mil = 0;
        let mut bld = 0;
        for e in w.entities_of(p.id) {
            let def = data.object(e.ty);
            match def.class {
                ObjectClass::Unit if def.is_villager => vils += 1,
                ObjectClass::Unit if !def.attacks.is_empty() => mil += 1,
                ObjectClass::Building if e.build_progress >= 1000 => bld += 1,
                _ => {}
            }
        }
        let r = p.resources;
        s.push_str(&format!(
            " | {} {} v{} m{} b{} f{} w{} g{} s{} sc{}{}",
            p.name,
            short_age(p.age),
            vils,
            mil,
            bld,
            r.get(Resource::Food),
            r.get(Resource::Wood),
            r.get(Resource::Gold),
            r.get(Resource::Stone),
            p.score,
            if p.alive { "" } else { " DEAD" }
        ));
    }
    s
}

/// With `WARS_AI_DEBUG` set: foundations and their age, and a building tally.
fn debug(w: &World, data: &wars_sim::types::GameData) {
    for p in w.players.iter().filter(|p| !p.id.is_gaia()) {
        let mut tally: std::collections::BTreeMap<&str, i32> = Default::default();
        let mut found = Vec::new();
        for e in w.entities_of(p.id) {
            let def = data.object(e.ty);
            if def.class != ObjectClass::Building {
                continue;
            }
            if e.build_progress >= 1000 {
                *tally.entry(def.key.as_str()).or_insert(0) += 1;
            } else {
                let builders = w
                    .entities_of(p.id)
                    .filter(|v| matches!(v.task(), wars_sim::world::Task::Build { target } if *target == e.id))
                    .count();
                found.push(format!(
                    "{}@{:?} {}% builders {}",
                    def.key,
                    e.pos.tile(),
                    e.build_progress / 10,
                    builders
                ));
            }
        }
        let idle = w
            .entities_of(p.id)
            .filter(|e| data.object(e.ty).is_villager && e.is_idle())
            .count();
        // Villager jobs: task kind and distance of the target from the town centre.
        let home = w
            .entities_of(p.id)
            .find(|e| data.object(e.ty).is_town_center)
            .map(|e| e.pos);
        let mut jobs: std::collections::BTreeMap<String, Vec<i32>> = Default::default();
        for e in w
            .entities_of(p.id)
            .filter(|e| data.object(e.ty).is_villager)
        {
            let (name, target) = match e.task() {
                wars_sim::world::Task::Gather { target, kind } => {
                    (format!("gather {kind:?}"), Some(*target))
                }
                wars_sim::world::Task::DropOff { resume, .. } => ("dropoff".to_string(), *resume),
                wars_sim::world::Task::Attack { target, .. } => ("hunt".to_string(), Some(*target)),
                wars_sim::world::Task::Build { target } => ("build".to_string(), Some(*target)),
                t => (
                    format!("{t:?}")
                        .split_whitespace()
                        .next()
                        .unwrap_or("?")
                        .to_string(),
                    None,
                ),
            };
            let d = match (target.and_then(|t| w.get(t)), home) {
                (Some(t), Some(h)) => t.pos.dist(h).to_int(),
                _ => -1,
            };
            jobs.entry(name).or_default().push(d);
        }
        println!(
            "    {}: idle villagers {idle}; {:?}; foundations {:?}; jobs {:?}",
            p.name, tally, found, jobs
        );
    }
}

fn short_age(a: wars_sim::ids::Age) -> &'static str {
    match a {
        wars_sim::ids::Age::Dark => "Dark",
        wars_sim::ids::Age::Feudal => "Feud",
        wars_sim::ids::Age::Castle => "Cast",
        wars_sim::ids::Age::Imperial => "Impe",
    }
}
