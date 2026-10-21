//! End-to-end simulation tests on a hand-built map.

use wars_sim::command::*;
use wars_sim::ids::*;
use wars_sim::map::Map;
use wars_sim::step::step;
use wars_sim::types::*;
use wars_sim::world::*;
use wars_sim::{Fixed, Vec2};

fn setup() -> (World, &'static GameData) {
    let data = wars_data::data();
    let map = Map::new(40, 40, Terrain::Grass);
    let settings = Settings {
        allow_cheats: true,
        ..Settings::default()
    };
    let mut w = World::new(map, 42, settings);
    w.add_player("Gaia", CivId(0), 0, false, Stockpile::ZERO);
    w.add_player(
        "Blue",
        data.civ_id("britons").unwrap(),
        1,
        false,
        Stockpile::new(200, 200, 100, 200),
    );
    w.add_player(
        "Red",
        data.civ_id("franks").unwrap(),
        2,
        true,
        Stockpile::new(200, 200, 100, 200),
    );
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let tc = data.obj("town_center");
    let vil = data.obj("villager");
    w.spawn(data, p1, tc, Vec2::from_tile(5, 5), false).unwrap();
    for i in 0..3 {
        w.spawn(data, p1, vil, Vec2::tile_center(5 + i, 10), false)
            .unwrap();
    }
    w.spawn(data, p2, tc, Vec2::from_tile(30, 30), false)
        .unwrap();
    for i in 0..3 {
        w.spawn(data, p2, vil, Vec2::tile_center(30 + i, 28), false)
            .unwrap();
    }
    // Resources near blue.
    let tree = data.obj("tree");
    for y in 12..16 {
        for x in 2..6 {
            w.spawn(data, PlayerId::GAIA, tree, Vec2::from_tile(x, y), false);
        }
    }
    w.spawn(
        data,
        PlayerId::GAIA,
        data.obj("gold_mine"),
        Vec2::from_tile(12, 6),
        false,
    )
    .unwrap();
    w.spawn(
        data,
        PlayerId::GAIA,
        data.obj("berry_bush"),
        Vec2::from_tile(12, 9),
        false,
    )
    .unwrap();
    w.spawn(
        data,
        PlayerId::GAIA,
        data.obj("sheep"),
        Vec2::tile_center(8, 12),
        false,
    )
    .unwrap();
    wars_sim::vision::update_vision(&mut w, data);
    (w, data)
}

fn run(w: &mut World, data: &GameData, ticks: u32, cmds: &[(u32, PlayerCommand)]) {
    for _ in 0..ticks {
        let t = w.tick + 1;
        let batch: Vec<PlayerCommand> = cmds
            .iter()
            .filter(|(at, _)| *at == t)
            .map(|(_, c)| c.clone())
            .collect();
        step(w, data, &batch);
    }
}

fn units_of(w: &World, p: PlayerId, key: &str, data: &GameData) -> Vec<EntityId> {
    let ty = data.obj(key);
    w.entities_of(p)
        .filter(|e| e.ty == ty)
        .map(|e| e.id)
        .collect()
}

#[test]
fn villagers_gather_wood_and_drop_it_off() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let vils = units_of(&w, p1, "villager", data);
    let tree = w
        .iter()
        .find(|e| data.object(e.ty).key == "tree")
        .unwrap()
        .id;
    let cmd = PlayerCommand {
        player: p1,
        command: Command::Gather {
            units: vils.clone(),
            target: tree,
            queue: false,
        },
    };
    let before = w.player(p1).resources.wood;
    run(&mut w, data, 20 * 90, &[(1, cmd)]);
    let after = w.player(p1).resources.wood;
    assert!(after > before + 30, "wood {before} -> {after}");
}

#[test]
fn build_house_then_train_villager() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let vils = units_of(&w, p1, "villager", data);
    let house = data.obj("house");
    let tc = units_of(&w, p1, "town_center", data)[0];
    let cmds = vec![
        (
            1,
            PlayerCommand {
                player: p1,
                command: Command::Build {
                    units: vec![vils[0]],
                    building: house,
                    x: 10,
                    y: 12,
                    queue: false,
                },
            },
        ),
        (
            2,
            PlayerCommand {
                player: p1,
                command: Command::Train {
                    building: tc,
                    unit: data.obj("villager"),
                },
            },
        ),
        (
            3,
            PlayerCommand {
                player: p1,
                command: Command::Train {
                    building: tc,
                    unit: data.obj("villager"),
                },
            },
        ),
    ];
    run(&mut w, data, 20 * 80, &cmds);
    let houses = w
        .entities_of(p1)
        .filter(|e| e.ty == house && e.build_progress >= 1000)
        .count();
    assert_eq!(houses, 1, "house completed");
    let n = units_of(&w, p1, "villager", data).len();
    assert_eq!(
        n,
        5,
        "two villagers trained (pop {} / support {})",
        w.player(p1).pop,
        w.player(p1).pop_support
    );
    assert_eq!(w.player(p1).pop, 5);
}

#[test]
fn research_and_age_up() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let tc = units_of(&w, p1, "town_center", data)[0];
    w.player_mut(p1).resources = Stockpile::new(5000, 5000, 5000, 5000);
    let cmds = vec![
        (
            1,
            PlayerCommand {
                player: p1,
                command: Command::Research {
                    building: tc,
                    tech: data.tk("loom"),
                },
            },
        ),
        (
            2,
            PlayerCommand {
                player: p1,
                command: Command::Research {
                    building: tc,
                    tech: data.tk("feudal_age"),
                },
            },
        ),
    ];
    run(&mut w, data, 20 * 170, &cmds);
    let p = w.player(p1);
    assert!(p.has_tech(data.tk("loom")));
    assert_eq!(p.age, Age::Feudal);
    let st = wars_sim::stats::stats_for(&w, data, p1, data.obj("villager"));
    assert_eq!(st.hp, 40, "loom adds 15 hp");
}

#[test]
fn militia_kill_enemy_villagers_and_conquest_ends_the_game() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let champ = data.obj("champion");
    let mut army = Vec::new();
    for i in 0..12 {
        army.push(
            w.spawn(
                data,
                p1,
                champ,
                Vec2::tile_center(10 + i % 4, 14 + i / 4),
                false,
            )
            .unwrap(),
        );
    }
    let ram = data.obj("siege_ram");
    for i in 0..4 {
        army.push(
            w.spawn(data, p1, ram, Vec2::tile_center(10 + i, 18), false)
                .unwrap(),
        );
    }
    let cmds = vec![(
        1,
        PlayerCommand {
            player: p1,
            command: Command::AttackMove {
                units: army,
                to: Vec2::tile_center(31, 29),
                queue: false,
            },
        },
    )];
    run(&mut w, data, 20 * 400, &cmds);
    assert!(
        units_of(&w, p2, "villager", data).is_empty(),
        "enemy villagers dead"
    );
    assert!(
        units_of(&w, p2, "town_center", data).is_empty(),
        "enemy TC razed"
    );
    assert!(w.game_over, "game over");
    assert_eq!(w.winner_team, Some(1));
}

#[test]
fn deterministic_replay_hashes_match() {
    let (mut a, data) = setup();
    let (mut b, _) = setup();
    let p1 = PlayerId(1);
    let vils = units_of(&a, p1, "villager", data);
    let tree = a
        .iter()
        .find(|e| data.object(e.ty).key == "tree")
        .unwrap()
        .id;
    let tc = units_of(&a, p1, "town_center", data)[0];
    let cmds = vec![
        (
            1,
            PlayerCommand {
                player: p1,
                command: Command::Gather {
                    units: vils.clone(),
                    target: tree,
                    queue: false,
                },
            },
        ),
        (
            5,
            PlayerCommand {
                player: p1,
                command: Command::Train {
                    building: tc,
                    unit: data.obj("villager"),
                },
            },
        ),
        (
            40,
            PlayerCommand {
                player: p1,
                command: Command::Build {
                    units: vec![vils[1]],
                    building: data.obj("house"),
                    x: 10,
                    y: 12,
                    queue: false,
                },
            },
        ),
        (
            300,
            PlayerCommand {
                player: p1,
                command: Command::Cheat {
                    code: "cheese steak jimmy's".into(),
                },
            },
        ),
    ];
    for _ in 0..3 {
        run(&mut a, data, 20 * 30, &cmds);
        run(&mut b, data, 20 * 30, &cmds);
        assert_eq!(a.hash(), b.hash(), "hash at tick {}", a.tick);
    }
    // Snapshot round trip keeps the hash.
    let bytes = a.snapshot();
    let c = World::from_snapshot(&bytes).unwrap();
    assert_eq!(c.hash(), a.hash());
    let mut c = c;
    run(&mut a, data, 100, &[]);
    run(&mut c, data, 100, &[]);
    assert_eq!(a.hash(), c.hash());
}

#[test]
fn sheep_get_captured_and_eaten() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let vils = units_of(&w, p1, "villager", data);
    let sheep = w
        .iter()
        .find(|e| data.object(e.ty).key == "sheep")
        .unwrap()
        .id;
    run(&mut w, data, 30, &[]);
    assert_eq!(
        w.get(sheep).unwrap().owner,
        p1,
        "sheep captured by proximity"
    );
    let cmd = PlayerCommand {
        player: p1,
        command: Command::Gather {
            units: vils,
            target: sheep,
            queue: false,
        },
    };
    let before = w.player(p1).resources.food;
    run(&mut w, data, 20 * 60, &[(31, cmd)]);
    assert!(
        w.player(p1).resources.food >= before + 25,
        "food {} -> {}",
        before,
        w.player(p1).resources.food
    );
}

#[test]
fn market_and_tribute() {
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    w.spawn(data, p1, data.obj("market"), Vec2::from_tile(14, 14), false)
        .unwrap();
    w.player_mut(p1).resources = Stockpile::new(500, 500, 500, 500);
    let cmds = vec![
        (
            1,
            PlayerCommand {
                player: p1,
                command: Command::MarketBuy {
                    resource: Resource::Food,
                },
            },
        ),
        (
            2,
            PlayerCommand {
                player: p1,
                command: Command::MarketSell {
                    resource: Resource::Wood,
                },
            },
        ),
        (
            3,
            PlayerCommand {
                player: p1,
                command: Command::Tribute {
                    to: PlayerId(2),
                    amount: Stockpile::new(100, 0, 0, 0),
                },
            },
        ),
    ];
    run(&mut w, data, 5, &cmds);
    let p = w.player(p1);
    assert_eq!(p.resources.food, 500);
    assert_eq!(p.resources.wood, 400);
    assert!(p.resources.gold < 500 + 100 && p.resources.gold > 400);
    assert_eq!(w.player(PlayerId(2)).resources.food, 300);
}

#[test]
fn fixed_speed_sanity() {
    // A villager (0.8 tiles/s) walks about 8 tiles in 10 seconds.
    let (mut w, data) = setup();
    let p1 = PlayerId(1);
    let v = units_of(&w, p1, "villager", data)[0];
    let start = w.get(v).unwrap().pos;
    let cmd = PlayerCommand {
        player: p1,
        command: Command::Move {
            units: vec![v],
            to: Vec2::tile_center(25, 10),
            queue: false,
        },
    };
    run(&mut w, data, 200, &[(1, cmd)]);
    let d = w.get(v).unwrap().pos.dist(start);
    assert!(
        d > Fixed::from_int(7) && d < Fixed::from_int(9),
        "walked {d}"
    );
}
