//! Match sessions: offline (we run the sim and the AIs) and online (we follow the
//! server's tick stream, running the same sim).

use std::collections::VecDeque;
use wars_ai::{AiPlayer, THINK_INTERVAL};
use wars_proto::{ClientMsg, ServerMsg, COMMAND_DELAY, HASH_INTERVAL};
use wars_sim::command::{Command, PlayerCommand};
use wars_sim::ids::PlayerId;
use wars_sim::types::GameData;
use wars_sim::world::{Event, World, TICKS_PER_SECOND};

/// Something that moves bytes to and from the server. Implemented by `net::WsTransport`
/// on desktop/console and by the platform layer on mobile.
pub trait Transport: Send {
    fn send(&mut self, bytes: Vec<u8>);
    /// Non-blocking: the next complete frame, if any.
    fn try_recv(&mut self) -> Option<Vec<u8>>;
    fn is_closed(&self) -> bool;
}

pub struct LocalMatch {
    pub world: World,
    pub me: PlayerId,
    pub ais: Vec<AiPlayer>,
    pending: Vec<PlayerCommand>,
    acc_ms: f64,
    pub speed_pct: u32,
    pub paused: bool,
    /// Full command log for replays and saves.
    pub log: Vec<(u32, PlayerCommand)>,
}

impl LocalMatch {
    pub fn new(world: World, me: PlayerId) -> LocalMatch {
        let ais = world
            .players
            .iter()
            .filter(|p| p.is_ai && !p.id.is_gaia())
            .map(|p| AiPlayer::new(p.id, p.ai_level))
            .collect();
        LocalMatch {
            world,
            me,
            ais,
            pending: Vec::new(),
            acc_ms: 0.0,
            speed_pct: 100,
            paused: false,
            log: Vec::new(),
        }
    }
    pub fn issue(&mut self, cmd: Command) {
        self.pending.push(PlayerCommand {
            player: self.me,
            command: cmd,
        });
    }
    /// Advance by wall-clock time; returns the events of every tick stepped.
    pub fn advance(&mut self, data: &GameData, dt_ms: f64) -> Vec<Event> {
        let mut events = Vec::new();
        if self.paused || self.world.game_over {
            return events;
        }
        let tick_ms = 1000.0 / TICKS_PER_SECOND as f64 * 100.0 / self.speed_pct.max(10) as f64;
        self.acc_ms += dt_ms.min(500.0);
        let mut steps = 0;
        while self.acc_ms >= tick_ms && steps < 8 {
            self.acc_ms -= tick_ms;
            steps += 1;
            events.extend(self.step_once(data));
        }
        events
    }
    /// Step exactly one tick (used by tests and the headless bot match).
    pub fn step_once(&mut self, data: &GameData) -> Vec<Event> {
        let next = self.world.tick + 1;
        if next % THINK_INTERVAL == 0 {
            for ai in self.ais.iter_mut() {
                if !self.world.player(ai.player).alive {
                    continue;
                }
                let cmds = ai.think(&self.world, data);
                for c in cmds {
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
        wars_sim::step::step(&mut self.world, data, &batch);
        self.world.events.clone()
    }
}

/// Online match: we hold the world and apply the server's batches in order.
pub struct RemoteMatch {
    pub world: World,
    pub me: Option<PlayerId>,
    pub transport: Box<dyn Transport>,
    pub lobby: wars_proto::Lobby,
    queue: VecDeque<(u32, Vec<PlayerCommand>)>,
    pub tick_rate: u32,
    pub paused: bool,
    pub last_rtt_ms: Option<u64>,
    pub chat: Vec<(String, String)>,
    pub ended: Option<Option<u8>>,
    pub desyncs: u32,
    /// Ticks we have applied since the last snapshot (for the UI's connection status).
    pub behind: usize,
}

impl RemoteMatch {
    pub fn new(
        world: World,
        me: Option<PlayerId>,
        lobby: wars_proto::Lobby,
        tick_rate: u32,
        transport: Box<dyn Transport>,
    ) -> RemoteMatch {
        RemoteMatch {
            world,
            me,
            transport,
            lobby,
            queue: VecDeque::new(),
            tick_rate,
            paused: false,
            last_rtt_ms: None,
            chat: Vec::new(),
            ended: None,
            desyncs: 0,
            behind: 0,
        }
    }
    pub fn issue(&mut self, cmd: Command) {
        self.transport
            .send(wars_proto::encode(&ClientMsg::Commands {
                commands: vec![cmd],
            }));
    }
    pub fn send(&mut self, msg: &ClientMsg) {
        self.transport.send(wars_proto::encode(msg));
    }
    /// Drain the socket into the queue without stepping.
    pub fn pump(&mut self) {
        while let Some(frame) = self.transport.try_recv() {
            self.handle_frame(frame);
        }
    }

    fn handle_frame(&mut self, frame: Vec<u8>) {
        match wars_proto::decode::<ServerMsg>(&frame) {
            Ok(ServerMsg::Tick { tick, commands }) => self.queue.push_back((tick, commands)),
            Ok(ServerMsg::Snapshot { snapshot, .. }) => {
                if let Ok(w) = World::from_snapshot(&snapshot) {
                    self.world = w;
                    self.queue.retain(|(t, _)| *t > self.world.tick);
                }
            }
            Ok(ServerMsg::Desync { .. }) => self.desyncs += 1,
            Ok(ServerMsg::Chat { name, text, .. }) => self.chat.push((name, text)),
            Ok(ServerMsg::Paused(p)) => self.paused = p,
            Ok(ServerMsg::SpeedChanged(s)) => self.tick_rate = TICKS_PER_SECOND * s / 100,
            Ok(ServerMsg::MatchEnded { winner_team }) => self.ended = Some(winner_team),
            Ok(ServerMsg::Pong { t }) => {
                let now = now_ms();
                self.last_rtt_ms = Some(now.saturating_sub(t));
            }
            Ok(ServerMsg::LobbyState(l)) => self.lobby = l,
            Ok(ServerMsg::PlayerLeft { name, .. }) => self
                .chat
                .push(("system".into(), format!("{name} left the game"))),
            Ok(ServerMsg::Error { message, .. }) => self.chat.push(("error".into(), message)),
            _ => {}
        }
    }

    /// Apply exactly one queued batch if the next tick is available. Returns its events.
    pub fn advance_one(&mut self, data: &GameData) -> Option<Vec<Event>> {
        loop {
            let (tick, _) = self.queue.front()?;
            if *tick <= self.world.tick {
                self.queue.pop_front();
                continue;
            }
            if *tick != self.world.tick + 1 {
                return None;
            }
            let (_, cmds) = self.queue.pop_front().unwrap();
            wars_sim::step::step(&mut self.world, data, &cmds);
            if self.world.tick % HASH_INTERVAL == 0 {
                let h = self.world.hash();
                let t = self.world.tick;
                self.send(&ClientMsg::Hash { tick: t, hash: h });
            }
            self.behind = self.queue.len();
            return Some(self.world.events.clone());
        }
    }

    /// Drain the socket, then apply the batches that are ready (a few per call to stay
    /// smooth, more when far behind). Returns events.
    pub fn advance(&mut self, data: &GameData) -> Vec<Event> {
        self.pump();
        let mut events = Vec::new();
        let max = if self.queue.len() > 10 { 6 } else { 2 };
        for _ in 0..max {
            match self.advance_one(data) {
                Some(ev) => events.extend(ev),
                None => break,
            }
        }
        self.behind = self.queue.len();
        events
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub enum Session {
    Local(LocalMatch),
    Remote(RemoteMatch),
}

impl Session {
    pub fn world(&self) -> &World {
        match self {
            Session::Local(m) => &m.world,
            Session::Remote(m) => &m.world,
        }
    }
    pub fn world_mut(&mut self) -> &mut World {
        match self {
            Session::Local(m) => &mut m.world,
            Session::Remote(m) => &mut m.world,
        }
    }
    /// Mutable world access for the scenario editor (offline only).
    pub fn world_mut_editor(&mut self) -> &mut World {
        self.world_mut()
    }
    pub fn me(&self) -> PlayerId {
        match self {
            Session::Local(m) => m.me,
            Session::Remote(m) => m.me.unwrap_or(PlayerId(1)),
        }
    }
    pub fn is_spectator(&self) -> bool {
        matches!(self, Session::Remote(m) if m.me.is_none())
    }
    pub fn issue(&mut self, cmd: Command) {
        match self {
            Session::Local(m) => m.issue(cmd),
            Session::Remote(m) => m.issue(cmd),
        }
    }
    pub fn advance(&mut self, data: &GameData, dt_ms: f64) -> Vec<Event> {
        match self {
            Session::Local(m) => m.advance(data, dt_ms),
            Session::Remote(m) => m.advance(data),
        }
    }
    pub fn is_online(&self) -> bool {
        matches!(self, Session::Remote(_))
    }
    pub fn paused(&self) -> bool {
        match self {
            Session::Local(m) => m.paused,
            Session::Remote(m) => m.paused,
        }
    }
    pub fn set_paused(&mut self, p: bool) {
        match self {
            Session::Local(m) => m.paused = p,
            Session::Remote(m) => m.send(&ClientMsg::Pause(p)),
        }
    }
    pub fn speed_pct(&self) -> u32 {
        match self {
            Session::Local(m) => m.speed_pct,
            Session::Remote(m) => m.tick_rate * 100 / TICKS_PER_SECOND,
        }
    }
    pub fn set_speed_pct(&mut self, s: u32) {
        match self {
            Session::Local(m) => m.speed_pct = s.clamp(25, 400),
            Session::Remote(m) => m.send(&ClientMsg::Speed(s)),
        }
    }
    pub fn chat(&mut self, text: String, to_allies: bool) {
        match self {
            Session::Local(m) => m.issue(Command::Chat { text, to_allies }),
            Session::Remote(m) => m.send(&ClientMsg::Chat { text, to_allies }),
        }
    }
    pub fn command_delay(&self) -> u32 {
        match self {
            Session::Local(_) => 0,
            Session::Remote(_) => COMMAND_DELAY,
        }
    }
}
