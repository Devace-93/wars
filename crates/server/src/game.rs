//! The authoritative match runner: one tokio task per match that owns the world, runs
//! the AIs, batches commands per tick, broadcasts the batches, checks hashes and
//! persists saves, results and the replay.

use crate::lobby::{ConnId, Tx};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use wars_ai::{AiPlayer, THINK_INTERVAL};
use wars_proto::*;
use wars_sim::command::{Command, PlayerCommand};
use wars_sim::ids::PlayerId;
use wars_sim::types::GameData;
use wars_sim::world::{World, TICKS_PER_SECOND};

pub enum MatchInput {
    Join {
        conn: ConnId,
        info: PlayerInfo,
        tx: Tx,
    },
    Leave {
        conn: ConnId,
    },
    Commands {
        conn: ConnId,
        commands: Vec<Command>,
    },
    Hash {
        conn: ConnId,
        tick: u32,
        hash: u64,
    },
    Chat {
        conn: ConnId,
        text: String,
        to_allies: bool,
    },
    Pause {
        conn: ConnId,
        paused: bool,
    },
    Speed {
        conn: ConnId,
        pct: u32,
    },
    Save {
        conn: ConnId,
        name: String,
    },
    Snapshot {
        conn: ConnId,
    },
    Abandon,
}

#[derive(Clone)]
pub struct MatchHandle {
    pub tx: UnboundedSender<MatchInput>,
    pub id: uuid::Uuid,
}

impl MatchHandle {
    pub fn send(&self, m: MatchInput) {
        let _ = self.tx.send(m);
    }
    pub fn abandon(&self) {
        let _ = self.tx.send(MatchInput::Abandon);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Replay {
    pub lobby: Lobby,
    pub seed: u64,
    pub commands: Vec<(u32, PlayerCommand)>,
    pub ticks: u32,
    pub winner_team: Option<u8>,
}

struct Member {
    info: PlayerInfo,
    tx: Tx,
    player: Option<PlayerId>,
}

/// Build the initial world from the lobby source.
pub async fn build_world(
    db: &PgPool,
    data: &GameData,
    lobby: &Lobby,
) -> Result<(World, u64), String> {
    match &lobby.source {
        MatchSource::Random { seed, .. } => {
            let params = lobby.map_params().ok_or("bad_source")?;
            Ok((wars_mapgen::generate(data, &params), *seed))
        }
        MatchSource::Scenario { id, .. } => {
            let uid = uuid::Uuid::parse_str(id).map_err(|_| "bad_scenario")?;
            let bytes: Vec<u8> = sqlx::query_scalar("SELECT data FROM scenarios WHERE id = $1")
                .bind(uid)
                .fetch_one(db)
                .await
                .map_err(|_| "no_scenario")?;
            let sc =
                wars_mapgen::scenario::Scenario::from_bytes(&bytes).map_err(|_| "bad_scenario")?;
            let seed = uuid::Uuid::new_v4().as_u128() as u64;
            let mut w = sc.to_world(data, seed);
            // Rename the active slots after the lobby and mark AI/humans.
            let ids = lobby.player_ids();
            for (i, (slot, pid)) in ids.iter().enumerate() {
                if pid.idx() < w.players.len() {
                    let s = &lobby.slots[*slot];
                    let p = w.player_mut(*pid);
                    p.name = s
                        .player
                        .as_ref()
                        .map(|p| p.display.clone())
                        .unwrap_or_else(|| format!("Computer {}", i + 1));
                    p.is_ai = s.is_ai;
                    p.ai_level = s.ai_level;
                    if let Some(c) = data.civ_id(&s.civ) {
                        p.civ = c;
                    }
                    p.team = s.team;
                }
            }
            Ok((w, seed))
        }
        MatchSource::Save { id, .. } => {
            let uid = uuid::Uuid::parse_str(id).map_err(|_| "bad_save")?;
            let bytes: Vec<u8> = sqlx::query_scalar("SELECT snapshot FROM saves WHERE id = $1")
                .bind(uid)
                .fetch_one(db)
                .await
                .map_err(|_| "no_save")?;
            let w = World::from_snapshot(&bytes).map_err(|_| "bad_save")?;
            let seed = w.rng.seed();
            Ok((w, seed))
        }
    }
}

/// Start a match for a room: builds the world, records it, spawns the runner.
pub async fn start(
    db: PgPool,
    data: &'static GameData,
    lobby: Lobby,
    members: Vec<(ConnId, PlayerInfo, Tx)>,
    host_id: String,
) -> Result<MatchHandle, String> {
    let (world, seed) = build_world(&db, data, &lobby).await?;
    let id = uuid::Uuid::new_v4();
    let lobby_json = serde_json::to_value(&lobby).unwrap_or_default();
    let _ = sqlx::query(
        "INSERT INTO matches (id, name, host_id, lobby, seed) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(&lobby.name)
    .bind(&host_id)
    .bind(&lobby_json)
    .bind(seed as i64)
    .execute(&db)
    .await;
    for (slot, pid) in lobby.player_ids() {
        let s = &lobby.slots[slot];
        let player_id = s.player.as_ref().map(|p| p.id.clone());
        let _ = sqlx::query("INSERT INTO match_players (match_id, slot, player_id, sim_player, civ, team, is_ai) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(id)
            .bind(slot as i32)
            .bind(player_id)
            .bind(pid.0 as i32)
            .bind(&s.civ)
            .bind(s.team as i32)
            .bind(s.is_ai)
            .execute(&db)
            .await;
    }
    let (tx, rx) = unbounded_channel();
    let handle = MatchHandle { tx, id };
    let runner = Runner::new(db, data, id, lobby, world, seed, members);
    tokio::spawn(runner.run(rx));
    Ok(handle)
}

struct Runner {
    db: PgPool,
    data: &'static GameData,
    id: uuid::Uuid,
    lobby: Lobby,
    world: World,
    seed: u64,
    members: HashMap<ConnId, Member>,
    ais: Vec<AiPlayer>,
    pending: Vec<PlayerCommand>,
    log: Vec<(u32, PlayerCommand)>,
    hashes: VecDeque<(u32, u64)>,
    paused: bool,
    tick_rate: u32,
    host_conn: Option<ConnId>,
    last_autosave: u32,
}

impl Runner {
    fn new(
        db: PgPool,
        data: &'static GameData,
        id: uuid::Uuid,
        lobby: Lobby,
        world: World,
        seed: u64,
        members: Vec<(ConnId, PlayerInfo, Tx)>,
    ) -> Runner {
        let tick_rate = TICKS_PER_SECOND * lobby.speed_pct.clamp(25, 300) / 100;
        let ais = world
            .players
            .iter()
            .filter(|p| p.is_ai && !p.id.is_gaia())
            .map(|p| AiPlayer::new(p.id, p.ai_level))
            .collect();
        let mut r = Runner {
            db,
            data,
            id,
            lobby,
            world,
            seed,
            members: HashMap::new(),
            ais,
            pending: Vec::new(),
            log: Vec::new(),
            hashes: VecDeque::new(),
            paused: false,
            tick_rate,
            host_conn: None,
            last_autosave: 0,
        };
        for (conn, info, tx) in members {
            r.attach(conn, info, tx);
        }
        r.host_conn = r.members.keys().min().copied();
        r
    }

    fn player_for(&self, info: &PlayerInfo) -> Option<PlayerId> {
        self.lobby
            .player_ids()
            .into_iter()
            .find(|(slot, _)| {
                self.lobby.slots[*slot]
                    .player
                    .as_ref()
                    .map(|p| p.id == info.id)
                    .unwrap_or(false)
            })
            .map(|(_, pid)| pid)
    }

    fn attach(&mut self, conn: ConnId, info: PlayerInfo, tx: Tx) {
        let player = self.player_for(&info);
        let snapshot = self.world.snapshot();
        let _ = tx.send(ServerMsg::MatchStart {
            lobby: self.lobby.clone(),
            you: player,
            snapshot,
            tick_rate: self.tick_rate,
        });
        self.members.insert(conn, Member { info, tx, player });
    }

    fn broadcast(&self, msg: &ServerMsg) {
        for m in self.members.values() {
            let _ = m.tx.send(msg.clone());
        }
    }

    async fn run(mut self, mut rx: UnboundedReceiver<MatchInput>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(
            (1000 / self.tick_rate.max(1)) as u64,
        ));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut current_rate = self.tick_rate;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if current_rate != self.tick_rate {
                        current_rate = self.tick_rate;
                        interval = tokio::time::interval(std::time::Duration::from_millis((1000 / self.tick_rate.max(1)) as u64));
                        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                    }
                    if !self.paused && !self.world.game_over {
                        self.step();
                        if self.world.game_over {
                            self.finish().await;
                            return;
                        }
                        if self.world.tick - self.last_autosave >= 120 * TICKS_PER_SECOND {
                            self.last_autosave = self.world.tick;
                            self.autosave().await;
                        }
                    }
                }
                input = rx.recv() => {
                    match input {
                        None | Some(MatchInput::Abandon) => {
                            self.abandon().await;
                            return;
                        }
                        Some(i) => {
                            if self.handle(i).await {
                                // Everyone left.
                                self.abandon().await;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn step(&mut self) {
        let next = self.world.tick + 1;
        if next.is_multiple_of(THINK_INTERVAL) {
            for ai in self.ais.iter_mut() {
                if !self.world.player(ai.player).alive {
                    continue;
                }
                for c in ai.think(&self.world, self.data) {
                    self.pending.push(PlayerCommand {
                        player: ai.player,
                        command: c,
                    });
                }
            }
        }
        let batch = std::mem::take(&mut self.pending);
        for c in &batch {
            self.log.push((next, c.clone()));
        }
        wars_sim::step::step(&mut self.world, self.data, &batch);
        self.broadcast(&ServerMsg::Tick {
            tick: self.world.tick,
            commands: batch,
        });
        if self.world.tick.is_multiple_of(HASH_INTERVAL) {
            let h = self.world.hash();
            self.hashes.push_back((self.world.tick, h));
            while self.hashes.len() > 60 {
                self.hashes.pop_front();
            }
        }
    }

    /// Returns true when the match should be abandoned (no humans left).
    async fn handle(&mut self, input: MatchInput) -> bool {
        match input {
            MatchInput::Join { conn, info, tx } => {
                self.attach(conn, info, tx);
                if self.host_conn.is_none() {
                    self.host_conn = Some(conn);
                }
            }
            MatchInput::Leave { conn } => {
                if let Some(m) = self.members.remove(&conn) {
                    if let Some(p) = m.player {
                        self.broadcast(&ServerMsg::PlayerLeft {
                            player: p,
                            name: m.info.display.clone(),
                        });
                    }
                }
                if self.host_conn == Some(conn) {
                    self.host_conn = self.members.keys().min().copied();
                }
                if self.members.is_empty() {
                    return true;
                }
            }
            MatchInput::Commands { conn, commands } => {
                if let Some(p) = self.members.get(&conn).and_then(|m| m.player) {
                    for c in commands {
                        // Clients never spawn or cheat unless the match allows it.
                        if matches!(c, Command::Spawn { .. }) {
                            continue;
                        }
                        self.pending.push(PlayerCommand {
                            player: p,
                            command: c,
                        });
                    }
                }
            }
            MatchInput::Hash { conn, tick, hash } => {
                if let Some((_, h)) = self.hashes.iter().find(|(t, _)| *t == tick) {
                    if *h != hash {
                        tracing::warn!("desync in match {} at tick {tick} (conn {conn})", self.id);
                        if let Some(m) = self.members.get(&conn) {
                            let _ = m.tx.send(ServerMsg::Desync { tick });
                            let _ = m.tx.send(ServerMsg::Snapshot {
                                tick: self.world.tick,
                                snapshot: self.world.snapshot(),
                            });
                        }
                    }
                }
            }
            MatchInput::Chat {
                conn,
                text,
                to_allies,
            } => {
                if let Some(m) = self.members.get(&conn) {
                    let from = m.player;
                    let name = m.info.display.clone();
                    let text: String = text.chars().take(300).collect();
                    let msg = ServerMsg::Chat {
                        from,
                        name,
                        text: text.clone(),
                        to_allies,
                    };
                    if to_allies {
                        if let Some(p) = from {
                            for other in self.members.values() {
                                let ok =
                                    other.player.map(|o| self.world.ally(p, o)).unwrap_or(false);
                                if ok {
                                    let _ = other.tx.send(msg.clone());
                                }
                            }
                        }
                    } else {
                        self.broadcast(&msg);
                    }
                    self.log.push((
                        self.world.tick,
                        PlayerCommand {
                            player: from.unwrap_or(PlayerId::GAIA),
                            command: Command::Chat { text, to_allies },
                        },
                    ));
                }
            }
            MatchInput::Pause { conn, paused } => {
                if Some(conn) == self.host_conn {
                    self.paused = paused;
                    self.broadcast(&ServerMsg::Paused(paused));
                }
            }
            MatchInput::Speed { conn, pct } => {
                if Some(conn) == self.host_conn {
                    let pct = pct.clamp(25, 300);
                    self.tick_rate = TICKS_PER_SECOND * pct / 100;
                    self.broadcast(&ServerMsg::SpeedChanged(pct));
                }
            }
            MatchInput::Save { conn, name } => {
                if let Some(m) = self.members.get(&conn) {
                    if m.info.guest {
                        let _ = m.tx.send(ServerMsg::Error {
                            code: "guest".into(),
                            message: "Guests cannot save online games. Log in with a 3m4 account."
                                .into(),
                        });
                    } else {
                        let owner = m.info.id.clone();
                        let tx = m.tx.clone();
                        match self.save(&owner, &name).await {
                            Ok(id) => {
                                let _ = tx.send(ServerMsg::Saved {
                                    id: id.to_string(),
                                    name,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(ServerMsg::Error {
                                    code: "save_failed".into(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
            }
            MatchInput::Snapshot { conn } => {
                if let Some(m) = self.members.get(&conn) {
                    let _ = m.tx.send(ServerMsg::Snapshot {
                        tick: self.world.tick,
                        snapshot: self.world.snapshot(),
                    });
                }
            }
            MatchInput::Abandon => return true,
        }
        false
    }

    async fn save(&self, owner: &str, name: &str) -> Result<uuid::Uuid, String> {
        let id = uuid::Uuid::new_v4();
        let name: String = if name.trim().is_empty() {
            format!("{} ({}m)", self.lobby.name, self.world.seconds() / 60)
        } else {
            name.chars().take(60).collect()
        };
        let lobby_json = serde_json::to_value(&self.lobby).unwrap_or_default();
        sqlx::query("INSERT INTO saves (id, owner_id, name, tick, lobby, snapshot) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(id)
            .bind(owner)
            .bind(&name)
            .bind(self.world.tick as i32)
            .bind(&lobby_json)
            .bind(self.world.snapshot())
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    async fn autosave(&self) {
        // One autosave per non-guest human, replaced each time.
        for m in self.members.values() {
            if m.info.guest || m.player.is_none() {
                continue;
            }
            let name = format!("Autosave: {}", self.lobby.name);
            let _ = sqlx::query("DELETE FROM saves WHERE owner_id = $1 AND name = $2")
                .bind(&m.info.id)
                .bind(&name)
                .execute(&self.db)
                .await;
            let _ = self.save(&m.info.id, &name).await;
        }
    }

    async fn finish(&mut self) {
        let winner = self.world.winner_team;
        self.broadcast(&ServerMsg::MatchEnded {
            winner_team: winner,
        });
        let _ = sqlx::query("UPDATE matches SET status = 'finished', winner_team = $2, ticks = $3, ended_at = now() WHERE id = $1")
            .bind(self.id)
            .bind(winner.map(|w| w as i32))
            .bind(self.world.tick as i32)
            .execute(&self.db)
            .await;
        // Results and ELO.
        let mut winners: Vec<String> = Vec::new();
        let mut losers: Vec<String> = Vec::new();
        for (slot, pid) in self.lobby.player_ids() {
            let s = &self.lobby.slots[slot];
            let p = self.world.player(pid);
            let team = if p.team == 0 { 100 + pid.0 } else { p.team };
            let won = winner == Some(team);
            let result = if won {
                "won"
            } else if p.resigned {
                "resigned"
            } else {
                "lost"
            };
            let _ = sqlx::query("UPDATE match_players SET result = $3, score = $4 WHERE match_id = $1 AND slot = $2")
                .bind(self.id)
                .bind(slot as i32)
                .bind(result)
                .bind(p.score)
                .execute(&self.db)
                .await;
            if let Some(info) = &s.player {
                if !info.guest {
                    if won {
                        winners.push(info.id.clone())
                    } else {
                        losers.push(info.id.clone())
                    }
                }
            }
        }
        if !winners.is_empty() && !losers.is_empty() {
            let wa = average_elo(&self.db, &winners).await;
            let la = average_elo(&self.db, &losers).await;
            let expected_w = 1.0 / (1.0 + 10f64.powf((la - wa) / 400.0));
            let delta = (32.0 * (1.0 - expected_w)).round() as i32;
            for id in &winners {
                let _ =
                    sqlx::query("UPDATE players SET elo = elo + $2, wins = wins + 1 WHERE id = $1")
                        .bind(id)
                        .bind(delta)
                        .execute(&self.db)
                        .await;
            }
            for id in &losers {
                let _ = sqlx::query("UPDATE players SET elo = GREATEST(100, elo - $2), losses = losses + 1 WHERE id = $1").bind(id).bind(delta).execute(&self.db).await;
            }
        }
        let replay = Replay {
            lobby: self.lobby.clone(),
            seed: self.seed,
            commands: std::mem::take(&mut self.log),
            ticks: self.world.tick,
            winner_team: winner,
        };
        if let Ok(bytes) = postcard::to_allocvec(&replay) {
            let _ = sqlx::query("INSERT INTO replays (match_id, data) VALUES ($1, $2) ON CONFLICT (match_id) DO UPDATE SET data = EXCLUDED.data").bind(self.id).bind(bytes).execute(&self.db).await;
        }
    }

    async fn abandon(&mut self) {
        let _ = sqlx::query("UPDATE matches SET status = 'abandoned', ticks = $2, ended_at = now() WHERE id = $1 AND status = 'running'")
            .bind(self.id)
            .bind(self.world.tick as i32)
            .execute(&self.db)
            .await;
        self.broadcast(&ServerMsg::MatchEnded { winner_team: None });
    }
}

async fn average_elo(db: &PgPool, ids: &[String]) -> f64 {
    let mut sum = 0i64;
    for id in ids {
        let e: Option<i32> = sqlx::query_scalar("SELECT elo FROM players WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
        sum += e.unwrap_or(1000) as i64;
    }
    (sum / ids.len().max(1) as i64) as f64
}
