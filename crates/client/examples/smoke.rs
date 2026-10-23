//! End-to-end smoke test against a running server (default: production):
//! `cargo run -p wars-client --features net --example smoke -- https://wars.3m4.net`
//! Two guests create/join a lobby with one AI, start, and play 100 lockstep ticks.

use std::time::{Duration, Instant};
use wars_client::net::{guest_token, ws_url, WsTransport};
use wars_client::{RemoteMatch, Transport};
use wars_proto::*;
use wars_sim::ids::PlayerId;
use wars_sim::world::World;

fn recv<F: Fn(&ServerMsg) -> bool>(t: &WsTransport, pred: F, what: &str) -> ServerMsg {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(frame) = t.recv_timeout(200) {
            let m: ServerMsg = decode(&frame).expect("decode");
            if let ServerMsg::Error { code, message } = &m {
                eprintln!("server error {code}: {message}");
            }
            if pred(&m) {
                return m;
            }
        }
    }
    panic!("timed out waiting for {what}");
}

fn main() {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://wars.3m4.net".into());
    let data = wars_data::data();
    let t0 = Instant::now();
    let a_tok = guest_token(&url, "Smoke A").expect("guest A");
    let b_tok = guest_token(&url, "Smoke B").expect("guest B");
    println!("guest tokens ok ({:?})", t0.elapsed());
    let mut a = WsTransport::connect(&ws_url(&url), &a_tok.token).expect("ws A");
    let mut b = WsTransport::connect(&ws_url(&url), &b_tok.token).expect("ws B");
    let w = recv(&a, |m| matches!(m, ServerMsg::Welcome { .. }), "welcome A");
    if let ServerMsg::Welcome { server_version, .. } = w {
        println!("server version {server_version}");
    }
    recv(&b, |m| matches!(m, ServerMsg::Welcome { .. }), "welcome B");
    a.send(encode(&ClientMsg::CreateLobby {
        name: "smoke".into(),
        source: MatchSource::Random {
            map_type: wars_mapgen::MapType::Arabia,
            size: wars_mapgen::MapSize::Tiny,
            seed: 3,
        },
        settings: wars_sim::world::Settings::default(),
        public: false,
    }));
    let lobby = match recv(&a, |m| matches!(m, ServerMsg::LobbyState(_)), "lobby") {
        ServerMsg::LobbyState(l) => l,
        _ => unreachable!(),
    };
    println!("lobby {} code {}", lobby.id, lobby.room_code);
    b.send(encode(&ClientMsg::JoinLobby {
        id: None,
        room_code: Some(lobby.room_code.clone()),
    }));
    recv(
        &b,
        |m| matches!(m, ServerMsg::LobbyState(l) if l.slots[1].player.is_some()),
        "B seated",
    );
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
        "ai",
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
    assert_eq!(wa.hash(), wb.hash(), "initial snapshots agree");
    println!(
        "match started: A={ya:?} B={yb:?} players={} ({:?})",
        wa.players.len(),
        t0.elapsed()
    );
    let mut ma = RemoteMatch::new(wa, ya, la, 20, Box::new(a));
    let mut mb = RemoteMatch::new(wb, yb, lb, 20, Box::new(b));
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
        to: tc + wars_sim::Vec2::from_tile(5, 5),
        queue: false,
    });
    let deadline = Instant::now() + Duration::from_secs(60);
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
    println!(
        "100 lockstep ticks ok, {checked} hash checks agree, desyncs={} ({:?})",
        ma.desyncs + mb.desyncs,
        t0.elapsed()
    );
    mb.send(&ClientMsg::LeaveLobby);
    ma.send(&ClientMsg::LeaveLobby);
    println!("SMOKE OK");
}
