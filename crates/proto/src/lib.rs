//! wars-proto: messages between clients and the server.
//!
//! Over the WebSocket every frame is one `ClientMsg` or `ServerMsg` encoded with postcard
//! (binary frames). The REST API uses JSON and the same structs where they overlap.

use serde::{Deserialize, Serialize};
use wars_mapgen::{MapParams, MapSize, MapType};
use wars_sim::command::{Command, PlayerCommand};
use wars_sim::ids::PlayerId;
use wars_sim::world::Settings;

pub const PROTOCOL_VERSION: u32 = 1;
/// Commands sent at wall-clock tick k execute at k + COMMAND_DELAY on every participant.
pub const COMMAND_DELAY: u32 = 3;
/// Clients report a hash every this many ticks; the server compares and resyncs on mismatch.
pub const HASH_INTERVAL: u32 = 20;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Stable id: the accounts `sub` or `guest:<uuid>`.
    pub id: String,
    pub handle: String,
    pub display: String,
    pub avatar_url: Option<String>,
    pub guest: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub player: Option<PlayerInfo>,
    pub civ: String,
    pub team: u8,
    pub is_ai: bool,
    pub ai_level: u8,
    pub ready: bool,
    pub color: u8,
    /// Closed slots are skipped when the match starts.
    pub open: bool,
}

impl Slot {
    pub fn empty(i: usize) -> Slot {
        Slot {
            player: None,
            civ: [
                "britons",
                "franks",
                "mongols",
                "teutons",
                "japanese",
                "saracens",
                "celts",
                "byzantines",
            ][i % 8]
                .into(),
            team: (i % 8) as u8 + 1,
            is_ai: false,
            ai_level: 1,
            ready: false,
            color: i as u8,
            open: i < 2,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MatchSource {
    Random {
        map_type: MapType,
        size: MapSize,
        seed: u64,
    },
    /// A scenario stored on the server.
    Scenario { id: String, name: String },
    /// A saved game stored on the server (resumes with the saved slots).
    Save { id: String, name: String },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Lobby {
    pub id: String,
    pub room_code: String,
    pub name: String,
    pub host: String,
    pub source: MatchSource,
    pub settings: Settings,
    pub speed_pct: u32,
    pub slots: Vec<Slot>,
    pub public: bool,
    pub started: bool,
    pub created_at: String,
}

impl Lobby {
    pub fn map_params(&self) -> Option<MapParams> {
        let MatchSource::Random {
            map_type,
            size,
            seed,
        } = self.source
        else {
            return None;
        };
        let players = self
            .slots
            .iter()
            .filter(|s| s.open && (s.player.is_some() || s.is_ai))
            .map(|s| wars_mapgen::PlayerSetup {
                name: s
                    .player
                    .as_ref()
                    .map(|p| p.display.clone())
                    .unwrap_or_else(|| "Computer".into()),
                civ: s.civ.clone(),
                team: s.team,
                is_ai: s.is_ai,
                ai_level: s.ai_level,
            })
            .collect();
        Some(MapParams {
            map_type,
            size,
            seed,
            players,
            settings: self.settings.clone(),
        })
    }
    /// Live slot index -> sim player id (1-based, skipping closed/empty slots).
    pub fn player_ids(&self) -> Vec<(usize, PlayerId)> {
        let mut out = Vec::new();
        let mut next = 1u8;
        for (i, s) in self.slots.iter().enumerate() {
            if s.open && (s.player.is_some() || s.is_ai) {
                out.push((i, PlayerId(next)));
                next += 1;
            }
        }
        out
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LobbySummary {
    pub id: String,
    pub name: String,
    pub host: String,
    pub map: String,
    pub players: usize,
    pub max_players: usize,
    pub started: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SlotChange {
    Civ(String),
    Team(u8),
    Ready(bool),
    Ai {
        is_ai: bool,
        level: u8,
    },
    Open(bool),
    Color(u8),
    /// Move the sender to another slot.
    Move(usize),
    Kick,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    /// First frame: authenticate with an accounts JWT or a wars guest token.
    Hello {
        token: String,
        version: u32,
    },
    CreateLobby {
        name: String,
        source: MatchSource,
        settings: Settings,
        public: bool,
    },
    JoinLobby {
        id: Option<String>,
        room_code: Option<String>,
    },
    LeaveLobby,
    /// Change a slot (hosts may change any slot; players only their own).
    SetSlot {
        slot: usize,
        change: SlotChange,
    },
    SetSource {
        source: MatchSource,
    },
    SetSettings {
        settings: Settings,
        speed_pct: u32,
    },
    StartMatch,
    /// Commands to schedule on the next tick batch.
    Commands {
        commands: Vec<Command>,
    },
    Chat {
        text: String,
        to_allies: bool,
    },
    Hash {
        tick: u32,
        hash: u64,
    },
    RequestSnapshot,
    /// Host only: pause/resume or change speed mid-game.
    Pause(bool),
    Speed(u32),
    SaveGame {
        name: String,
    },
    Ping {
        t: u64,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome {
        you: PlayerInfo,
        server_version: String,
    },
    Error {
        code: String,
        message: String,
    },
    LobbyState(Lobby),
    LobbyClosed,
    /// The match begins. `snapshot` is the full initial world; `you` is the sim player id
    /// (None for spectators).
    MatchStart {
        lobby: Lobby,
        you: Option<PlayerId>,
        snapshot: Vec<u8>,
        tick_rate: u32,
    },
    /// Authoritative command batch for one tick, in execution order.
    Tick {
        tick: u32,
        commands: Vec<PlayerCommand>,
    },
    /// Replace the local world (late join, reconnect or desync recovery).
    Snapshot {
        tick: u32,
        snapshot: Vec<u8>,
    },
    Desync {
        tick: u32,
    },
    Chat {
        from: Option<PlayerId>,
        name: String,
        text: String,
        to_allies: bool,
    },
    PlayerLeft {
        player: PlayerId,
        name: String,
    },
    Paused(bool),
    SpeedChanged(u32),
    Saved {
        id: String,
        name: String,
    },
    MatchEnded {
        winner_team: Option<u8>,
    },
    Pong {
        t: u64,
    },
}

pub fn encode<T: Serialize>(m: &T) -> Vec<u8> {
    postcard::to_allocvec(m).expect("message serialises")
}
pub fn decode<'a, T: Deserialize<'a>>(b: &'a [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(b)
}

/// REST bodies.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GuestRequest {
    pub nickname: String,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub player: PlayerInfo,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SaveSummary {
    pub id: String,
    pub name: String,
    pub tick: u32,
    pub players: Vec<String>,
    pub created_at: String,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub public: bool,
    pub players: usize,
    pub size: i32,
    pub updated_at: String,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub handle: String,
    pub display: String,
    pub elo: i32,
    pub wins: i32,
    pub losses: i32,
}
