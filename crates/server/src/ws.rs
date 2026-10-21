//! WebSocket connections: authenticate, then route lobby and match messages.

use crate::game::MatchInput;
use crate::lobby::ConnId;
use crate::App;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::unbounded_channel;
use wars_proto::*;

static NEXT_CONN: AtomicU64 = AtomicU64::new(1);

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<App>) -> Response {
    ws.max_message_size(16 * 1024 * 1024)
        .on_upgrade(move |socket| handle(socket, app))
}

async fn handle(socket: WebSocket, app: App) {
    let (mut sink, mut stream) = socket.split();
    // First frame must be Hello within 10 seconds.
    let hello = tokio::time::timeout(std::time::Duration::from_secs(10), stream.next()).await;
    let token = match hello {
        Ok(Some(Ok(Message::Binary(b)))) => match decode::<ClientMsg>(&b) {
            Ok(ClientMsg::Hello { token, version }) if version == PROTOCOL_VERSION => token,
            Ok(ClientMsg::Hello { .. }) => {
                let _ = sink
                    .send(Message::Binary(encode(&ServerMsg::Error {
                        code: "version".into(),
                        message: "Please update Wars: protocol mismatch.".into(),
                    })))
                    .await;
                return;
            }
            _ => return,
        },
        _ => return,
    };
    let identity = match app.auth.verify(&token).await {
        Ok(i) => i,
        Err(code) => {
            let _ = sink
                .send(Message::Binary(encode(&ServerMsg::Error {
                    code: code.into(),
                    message: "Authentication failed".into(),
                })))
                .await;
            return;
        }
    };
    let _ = crate::db::upsert_player(
        &app.db,
        &identity.id,
        &identity.handle,
        identity.name.as_deref(),
        identity.avatar_url.as_deref(),
        identity.guest,
    )
    .await;
    let nickname = crate::db::nickname(&app.db, &identity.id).await;
    let info = identity.info(nickname);
    let conn: ConnId = NEXT_CONN.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = unbounded_channel::<ServerMsg>();
    let _ = tx.send(ServerMsg::Welcome {
        you: info.clone(),
        server_version: app.cfg.version.clone(),
    });
    // Writer task.
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(Message::Binary(encode(&msg))).await.is_err() {
                break;
            }
        }
    });
    let mut game: Option<crate::game::MatchHandle> = None;
    while let Some(Ok(msg)) = stream.next().await {
        let bytes = match msg {
            Message::Binary(b) => b,
            Message::Close(_) => break,
            _ => continue,
        };
        let Ok(cm) = decode::<ClientMsg>(&bytes) else {
            continue;
        };
        match cm {
            ClientMsg::Hello { .. } => {}
            ClientMsg::Ping { t } => {
                let _ = tx.send(ServerMsg::Pong { t });
            }
            ClientMsg::CreateLobby {
                name,
                source,
                settings,
                public,
            } => {
                if let Some(g) = game.take() {
                    g.send(MatchInput::Leave { conn });
                }
                let mut l = app.lobbies.lock().await;
                let room = l.create(
                    conn,
                    info.clone(),
                    tx.clone(),
                    name,
                    source,
                    settings,
                    public,
                );
                room.broadcast_state();
            }
            ClientMsg::JoinLobby { id, room_code } => {
                if let Some(g) = game.take() {
                    g.send(MatchInput::Leave { conn });
                }
                let mut l = app.lobbies.lock().await;
                match l.join(conn, info.clone(), tx.clone(), id, room_code) {
                    Ok(room) => {
                        if let Some(g) = room.game.clone() {
                            // Match already running: late join / reconnect.
                            g.send(MatchInput::Join {
                                conn,
                                info: info.clone(),
                                tx: tx.clone(),
                            });
                            game = Some(g);
                        } else {
                            room.broadcast_state();
                        }
                    }
                    Err(code) => {
                        let _ = tx.send(ServerMsg::Error {
                            code: code.into(),
                            message: "That room does not exist".into(),
                        });
                    }
                }
            }
            ClientMsg::LeaveLobby => {
                if let Some(g) = game.take() {
                    g.send(MatchInput::Leave { conn });
                }
                app.lobbies.lock().await.leave(conn);
                let _ = tx.send(ServerMsg::LobbyClosed);
            }
            ClientMsg::SetSlot { slot, change } => {
                let r = app.lobbies.lock().await.set_slot(conn, slot, change);
                if let Err(code) = r {
                    let _ = tx.send(ServerMsg::Error {
                        code: code.into(),
                        message: "Cannot change that slot".into(),
                    });
                }
            }
            ClientMsg::SetSource { source } => {
                let mut l = app.lobbies.lock().await;
                if let Some(room) = l.room_of(conn) {
                    if room.host == conn && !room.lobby.started {
                        room.lobby.source = source;
                        room.broadcast_state();
                    }
                }
            }
            ClientMsg::SetSettings {
                settings,
                speed_pct,
            } => {
                let mut l = app.lobbies.lock().await;
                if let Some(room) = l.room_of(conn) {
                    if room.host == conn && !room.lobby.started {
                        room.lobby.settings = settings;
                        room.lobby.speed_pct = speed_pct.clamp(25, 300);
                        room.broadcast_state();
                    }
                }
            }
            ClientMsg::StartMatch => {
                let start = {
                    let mut l = app.lobbies.lock().await;
                    match l.room_of(conn) {
                        Some(room) if room.host == conn && !room.lobby.started => {
                            let players = room
                                .lobby
                                .slots
                                .iter()
                                .filter(|s| s.open && (s.player.is_some() || s.is_ai))
                                .count();
                            if players < 1 {
                                let _ = tx.send(ServerMsg::Error {
                                    code: "no_players".into(),
                                    message: "Add at least one opponent".into(),
                                });
                                None
                            } else {
                                // Resolve random civs and mark started.
                                let hint = room.lobby.seed_hint();
                                for (i, s) in room.lobby.slots.iter_mut().enumerate() {
                                    if s.civ == "random" {
                                        let civs = &app.data.civs;
                                        s.civ = civs[(i + hint) % civs.len()].key.clone();
                                    }
                                }
                                room.lobby.started = true;
                                let members: Vec<_> = room
                                    .members
                                    .iter()
                                    .map(|(c, m)| (*c, m.info.clone(), m.tx.clone()))
                                    .collect();
                                Some((room.lobby.clone(), members))
                            }
                        }
                        _ => None,
                    }
                };
                if let Some((lobby, members)) = start {
                    match crate::game::start(
                        app.db.clone(),
                        app.data,
                        lobby.clone(),
                        members,
                        info.id.clone(),
                    )
                    .await
                    {
                        Ok(handle) => {
                            let mut l = app.lobbies.lock().await;
                            if let Some(room) = l.rooms.get_mut(&lobby.id) {
                                room.game = Some(handle.clone());
                            }
                            game = Some(handle);
                        }
                        Err(e) => {
                            let mut l = app.lobbies.lock().await;
                            if let Some(room) = l.rooms.get_mut(&lobby.id) {
                                room.lobby.started = false;
                                room.broadcast(&ServerMsg::Error {
                                    code: "start_failed".into(),
                                    message: e.to_string(),
                                });
                                room.broadcast_state();
                            }
                        }
                    }
                }
            }
            ClientMsg::Commands { commands } => {
                if game.is_none() {
                    game = app
                        .lobbies
                        .lock()
                        .await
                        .room_of(conn)
                        .and_then(|r| r.game.clone());
                }
                if let Some(g) = &game {
                    g.send(MatchInput::Commands { conn, commands });
                }
            }
            ClientMsg::Hash { tick, hash } => {
                if let Some(g) = &game {
                    g.send(MatchInput::Hash { conn, tick, hash });
                }
            }
            ClientMsg::Chat { text, to_allies } => {
                if game.is_none() {
                    game = app
                        .lobbies
                        .lock()
                        .await
                        .room_of(conn)
                        .and_then(|r| r.game.clone());
                }
                match &game {
                    Some(g) => g.send(MatchInput::Chat {
                        conn,
                        text,
                        to_allies,
                    }),
                    None => {
                        let l = app.lobbies.lock().await;
                        if let Some(id) = l.where_is.get(&conn) {
                            if let Some(room) = l.rooms.get(id) {
                                room.broadcast(&ServerMsg::Chat {
                                    from: None,
                                    name: info.display.clone(),
                                    text: text.chars().take(300).collect(),
                                    to_allies: false,
                                });
                            }
                        }
                    }
                }
            }
            ClientMsg::RequestSnapshot => {
                if let Some(g) = &game {
                    g.send(MatchInput::Snapshot { conn });
                }
            }
            ClientMsg::Pause(p) => {
                if let Some(g) = &game {
                    g.send(MatchInput::Pause { conn, paused: p });
                }
            }
            ClientMsg::Speed(s) => {
                if let Some(g) = &game {
                    g.send(MatchInput::Speed { conn, pct: s });
                }
            }
            ClientMsg::SaveGame { name } => {
                if let Some(g) = &game {
                    g.send(MatchInput::Save { conn, name });
                }
            }
        }
    }
    if let Some(g) = game.take() {
        g.send(MatchInput::Leave { conn });
    }
    app.lobbies.lock().await.leave(conn);
    writer.abort();
}

trait SeedHint {
    fn seed_hint(&self) -> usize;
}
impl SeedHint for Lobby {
    fn seed_hint(&self) -> usize {
        self.room_code.bytes().map(|b| b as usize).sum()
    }
}
