//! Integration tests: two computer players on a generated map, run headless through
//! `wars_client::LocalMatch`.

use wars_client::LocalMatch;
use wars_mapgen::{generate, MapParams, MapSize, MapType, PlayerSetup};
use wars_sim::ids::{Age, PlayerId};
use wars_sim::types::{GameData, ObjectClass};
use wars_sim::world::{QueueItem, World, TICKS_PER_SECOND};

fn setup(seed: u64, level: u8) -> LocalMatch {
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
    LocalMatch::new(generate(data, &params), PlayerId(1))
}

fn run_minutes(m: &mut LocalMatch, data: &GameData, minutes: u32) {
    for _ in 0..minutes * 60 * TICKS_PER_SECOND {
        if m.world.game_over {
            break;
        }
        m.step_once(data);
    }
}

fn villagers(w: &World, data: &GameData, p: PlayerId) -> usize {
    w.entities_of(p)
        .filter(|e| data.object(e.ty).is_villager)
        .count()
}

fn military(w: &World, data: &GameData, p: PlayerId) -> usize {
    w.entities_of(p)
        .filter(|e| {
            let def = data.object(e.ty);
            def.class == ObjectClass::Unit && !def.is_villager && !def.attacks.is_empty()
        })
        .count()
}

/// Feudal reached, or the research queued at a town centre.
fn feudal_underway(w: &World, data: &GameData, p: PlayerId) -> bool {
    let tech = data.tk("feudal_age");
    w.player(p).age >= Age::Feudal
        || w.player(p).researching.contains(&tech)
        || w.entities_of(p)
            .any(|e| e.queue.iter().any(|q| *q == QueueItem::Tech(tech)))
}

#[test]
fn ten_minutes_grow_the_economy_and_start_feudal() {
    let data = wars_data::data();
    let mut m = setup(7, 1);
    run_minutes(&mut m, data, 10);
    let w = &m.world;
    for p in [PlayerId(1), PlayerId(2)] {
        let n = villagers(w, data, p);
        assert!(n >= 15, "player {:?} has {n} villagers after 10 minutes", p);
    }
    assert!(
        feudal_underway(w, data, PlayerId(1)) || feudal_underway(w, data, PlayerId(2)),
        "nobody is advancing to Feudal after 10 minutes"
    );
    // Two and a half more minutes cover the research time itself.
    run_minutes(&mut m, data, 2);
    for _ in 0..30 * TICKS_PER_SECOND {
        m.step_once(data);
    }
    let w = &m.world;
    assert!(
        w.player(PlayerId(1)).age >= Age::Feudal || w.player(PlayerId(2)).age >= Age::Feudal,
        "nobody reached the Feudal Age by 12:30"
    );
}

#[test]
fn thirty_minutes_reach_castle_age_with_an_army() {
    let data = wars_data::data();
    let mut m = setup(7, 1);
    run_minutes(&mut m, data, 30);
    let w = &m.world;
    let castle = [PlayerId(1), PlayerId(2)]
        .iter()
        .any(|p| w.player(*p).age >= Age::Castle);
    assert!(castle, "nobody reached the Castle Age in 30 minutes");
    let army: usize = [PlayerId(1), PlayerId(2)]
        .iter()
        .map(|p| military(w, data, *p))
        .sum();
    assert!(army > 0, "no military units exist after 30 minutes");
    // The AIs trained plenty of units overall even if a battle just happened.
    let trained: i32 = w
        .players
        .iter()
        .filter(|p| !p.id.is_gaia())
        .map(|p| p.units_trained)
        .sum();
    assert!(trained >= 60, "only {trained} units trained in 30 minutes");
}

#[test]
fn same_seed_is_deterministic() {
    let data = wars_data::data();
    let mut a = setup(42, 2);
    let mut b = setup(42, 2);
    run_minutes(&mut a, data, 5);
    run_minutes(&mut b, data, 5);
    assert_eq!(
        a.world.hash(),
        b.world.hash(),
        "world hash differs after 5 minutes"
    );
    assert_eq!(a.log.len(), b.log.len(), "command logs differ");
    // The AI memory is part of the deterministic state too (it is saved with games).
    let ma = postcard::to_allocvec(&a.ais).unwrap();
    let mb = postcard::to_allocvec(&b.ais).unwrap();
    assert_eq!(ma, mb, "AI memory differs");
}

#[test]
fn easy_ai_acts_every_three_seconds() {
    let data = wars_data::data();
    let mut m = setup(5, 0);
    run_minutes(&mut m, data, 2);
    let seconds: std::collections::BTreeSet<u32> = m
        .log
        .iter()
        .map(|(tick, _)| tick / TICKS_PER_SECOND)
        .collect();
    assert!(!seconds.is_empty(), "easy AI issued no commands");
    assert!(
        seconds.iter().all(|s| s % 3 == 0),
        "easy AI acted outside its 3-second cadence: {seconds:?}"
    );
}
