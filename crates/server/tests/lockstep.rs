//! Two clients and one AI play through the real server: lobby, start, lockstep ticks,
//! hash agreement and desync recovery. Needs `WARS_TEST_DATABASE_URL`.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use wars_client::net::{guest_token, ws_url, WsTransport};
use wars_client::{RemoteMatch, Transport};
use wars_proto::*;
use wars_sim::ids::PlayerId;
use wars_sim::world::World;

struct Server {
    child: Child,
    url: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_server(db: &str) -> Server {
    let port = 18000 + (std::process::id() % 1000) as u16;
    #[allow(clippy::zombie_processes)]
    let child = Command::new(env!("CARGO_BIN_EXE_wars-server"))
        .env("DATABASE_URL", db)
        .env("PORT", port.to_string())
        .env("GUEST_SIGNING_KEY", "test-key")
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn server");
    let url = format!("http://127.0.0.1:{port}");
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if ureq::get(&format!("{url}/healthz"))
            .timeout(Duration::from_millis(500))
            .call()
            .is_ok()
        {
            return Server { child, url };
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("server did not start");
}

fn recv<F: Fn(&ServerMsg) -> bool>(t: &WsTransport, pred: F, what: &str) -> ServerMsg {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if let Some(frame) = t.recv_timeout(200) {
            let m: ServerMsg = decode(&frame).expect("decode");
            if pred(&m) {
                return m;
            }
        }
    }
    panic!("timed out waiting for {what}");
}

#[test]
fn two_clients_and_an_ai_stay_in_lockstep() {
    let Ok(db) = std::env::var("WARS_TEST_DATABASE_URL") else {
        eprintln!("WARS_TEST_DATABASE_URL not set; skipping");
        return;
    };
    let server = start_server(&db);
    let data = wars_data::data();
    let a_tok = guest_token(&server.url, "Alice").unwrap();
    let b_tok = guest_token(&server.url, "Bob").unwrap();
    let mut a = WsTransport::connect(&ws_url(&server.url), &a_tok.token).unwrap();
    let mut b = WsTransport::connect(&ws_url(&server.url), &b_tok.token).unwrap();
    recv(&a, |m| matches!(m, ServerMsg::Welcome { .. }), "welcome A");
    recv(&b, |m| matches!(m, ServerMsg::Welcome { .. }), "welcome B");
    a.send(encode(&ClientMsg::CreateLobby {
        name: "test".into(),
        source: MatchSource::Random {
            map_type: wars_mapgen::MapType::Arabia,
            size: wars_mapgen::MapSize::Tiny,
            seed: 7,
        },
        settings: wars_sim::world::Settings::default(),
        public: true,
    }));
    let lobby = match recv(&a, |m| matches!(m, ServerMsg::LobbyState(_)), "lobby A") {
        ServerMsg::LobbyState(l) => l,
        _ => unreachable!(),
    };
    b.send(encode(&ClientMsg::JoinLobby {
        id: None,
        room_code: Some(lobby.room_code.clone()),
    }));
    recv(
        &b,
        |m| matches!(m, ServerMsg::LobbyState(l) if l.slots[1].player.is_some()),
        "B seated",
    );
    // Public listing shows it.
    let list: Vec<LobbySummary> = ureq::get(&format!("{}/api/lobbies", server.url))
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].players, 2);
    // Add an AI in slot 2 and start.
    a.send(encode(&ClientMsg::SetSlot {
        slot: 2,
        change: SlotChange::Ai {
            is_ai: true,
            level: 1,
        },
    }));
    recv(
        &a,
        |m| matches!(m, ServerMsg::LobbyState(l) if l.slots[2].is_ai),
        "ai added",
    );
    a.send(encode(&ClientMsg::StartMatch));
    let (la, wa, ya) = match recv(&a, |m| matches!(m, ServerMsg::MatchStart { .. }), "start A") {
        ServerMsg::MatchStart {
            lobby,
            you,
            snapshot,
            ..
        } => (lobby, World::from_snapshot(&snapshot).unwrap(), you),
        _ => unreachable!(),
    };
    let (lb, wb, yb) = match recv(&b, |m| matches!(m, ServerMsg::MatchStart { .. }), "start B") {
        ServerMsg::MatchStart {
            lobby,
            you,
            snapshot,
            ..
        } => (lobby, World::from_snapshot(&snapshot).unwrap(), you),
        _ => unreachable!(),
    };
    assert_eq!(ya, Some(PlayerId(1)));
    assert_eq!(yb, Some(PlayerId(2)));
    assert_eq!(wa.hash(), wb.hash());
    assert_eq!(wa.players.len(), 4, "gaia + 2 humans + 1 ai");
    let mut ma = RemoteMatch::new(wa, ya, la, 20, Box::new(a));
    let mut mb = RemoteMatch::new(wb, yb, lb, 20, Box::new(b));
    // Alice moves her villagers; Bob trains one.
    let vils: Vec<_> = ma
        .world
        .entities_of(PlayerId(1))
        .filter(|e| data.object(e.ty).is_villager)
        .map(|e| e.id)
        .collect();
    let tc = ma
        .world
        .entities_of(PlayerId(1))
        .find(|e| data.object(e.ty).is_town_center)
        .unwrap()
        .pos;
    ma.issue(wars_sim::Command::Move {
        units: vils,
        to: tc + wars_sim::Vec2::from_tile(6, 6),
        queue: false,
    });
    let tcb = mb
        .world
        .entities_of(PlayerId(2))
        .find(|e| data.object(e.ty).is_town_center)
        .unwrap()
        .id;
    mb.issue(wars_sim::Command::Train {
        building: tcb,
        unit: data.obj("villager"),
    });
    mb.send(&ClientMsg::Chat {
        text: "hello".into(),
        to_allies: false,
    });
    // Step both to tick 100 one batch at a time and compare hashes along the way.
    let deadline = Instant::now() + Duration::from_secs(40);
    let mut checked = 0;
    while (ma.world.tick < 100 || mb.world.tick < 100) && Instant::now() < deadline {
        ma.pump();
        mb.pump();
        let mut progressed = false;
        if ma.world.tick <= mb.world.tick && ma.world.tick < 100 {
            progressed |= ma.advance_one(data).is_some();
        } else if mb.world.tick < 100 {
            progressed |= mb.advance_one(data).is_some();
        }
        if ma.world.tick == mb.world.tick && ma.world.tick % 20 == 0 && ma.world.tick > 0 {
            assert_eq!(
                ma.world.hash(),
                mb.world.hash(),
                "hash at tick {}",
                ma.world.tick
            );
            checked += 1;
        }
        if !progressed {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    assert!(
        ma.world.tick >= 100 && mb.world.tick >= 100,
        "ticks a={} b={}",
        ma.world.tick,
        mb.world.tick
    );
    assert!(checked >= 3);
    assert!(
        mb.world
            .entities_of(PlayerId(2))
            .any(|e| !e.queue.is_empty())
            || mb.world.player(PlayerId(2)).units_trained > 0,
        "Bob's train command was applied"
    );
    assert!(ma.chat.iter().any(|(_, t)| t == "hello"), "chat relayed");
    // Desync recovery: Bob corrupts his world and reports a wrong hash -> server sends a snapshot.
    mb.world.player_mut(PlayerId(2)).resources.food += 999;
    let bad = mb.world.hash();
    let t = mb.world.tick - mb.world.tick % 20;
    mb.send(&ClientMsg::Hash { tick: t, hash: bad });
    let deadline = Instant::now() + Duration::from_secs(10);
    while mb.desyncs == 0 && Instant::now() < deadline {
        mb.pump();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(mb.desyncs, 1, "server flagged the desync");
    // After the snapshot the corrupted food is gone.
    assert!(mb.world.player(PlayerId(2)).resources.food < 999);
    // Bob leaves; Alice keeps receiving ticks.
    mb.send(&ClientMsg::LeaveLobby);
    let before = ma.world.tick;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        ma.advance(data);
        if ma.world.tick > before + 20 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(ma.world.tick > before + 20);
    assert!(ma.chat.iter().any(|(_, t)| t.contains("left the game")));
}
