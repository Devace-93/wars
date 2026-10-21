//! In-memory lobbies. Every connection in a room holds a channel the room broadcasts on.

use crate::game::MatchHandle;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use wars_proto::*;

pub type ConnId = u64;
pub type Tx = UnboundedSender<ServerMsg>;

pub struct Member {
    pub info: PlayerInfo,
    pub tx: Tx,
}

pub struct Room {
    pub lobby: Lobby,
    pub members: HashMap<ConnId, Member>,
    pub host: ConnId,
    pub game: Option<MatchHandle>,
}

#[derive(Default)]
pub struct Lobbies {
    pub rooms: HashMap<String, Room>,
    pub by_code: HashMap<String, String>,
    /// conn -> room id
    pub where_is: HashMap<ConnId, String>,
}

fn room_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let u = uuid::Uuid::new_v4();
    u.as_bytes()
        .iter()
        .take(6)
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect()
}

impl Room {
    pub fn broadcast(&self, msg: &ServerMsg) {
        for m in self.members.values() {
            let _ = m.tx.send(msg.clone());
        }
    }
    pub fn broadcast_state(&self) {
        self.broadcast(&ServerMsg::LobbyState(self.lobby.clone()));
    }
    pub fn slot_of(&self, conn: ConnId) -> Option<usize> {
        let id = &self.members.get(&conn)?.info.id;
        self.lobby
            .slots
            .iter()
            .position(|s| s.player.as_ref().map(|p| &p.id == id).unwrap_or(false))
    }
    pub fn summary(&self) -> LobbySummary {
        LobbySummary {
            id: self.lobby.id.clone(),
            name: self.lobby.name.clone(),
            host: self.lobby.host.clone(),
            map: match &self.lobby.source {
                MatchSource::Random { map_type, size, .. } => {
                    format!("{} ({})", map_type.name(), size.name())
                }
                MatchSource::Scenario { name, .. } => format!("Scenario: {name}"),
                MatchSource::Save { name, .. } => format!("Saved: {name}"),
            },
            players: self
                .lobby
                .slots
                .iter()
                .filter(|s| s.open && (s.player.is_some() || s.is_ai))
                .count(),
            max_players: self.lobby.slots.iter().filter(|s| s.open).count(),
            started: self.lobby.started,
        }
    }
}

impl Lobbies {
    pub fn create(
        &mut self,
        conn: ConnId,
        info: PlayerInfo,
        tx: Tx,
        name: String,
        source: MatchSource,
        settings: wars_sim::world::Settings,
        public: bool,
    ) -> &mut Room {
        self.leave(conn);
        let id = uuid::Uuid::new_v4().to_string();
        let code = loop {
            let c = room_code();
            if !self.by_code.contains_key(&c) {
                break c;
            }
        };
        let mut slots: Vec<Slot> = (0..8).map(Slot::empty).collect();
        slots[0].player = Some(info.clone());
        slots[0].open = true;
        let name = if name.trim().is_empty() {
            format!("{}'s game", info.display)
        } else {
            name.chars().take(40).collect()
        };
        let lobby = Lobby {
            id: id.clone(),
            room_code: code.clone(),
            name,
            host: info.display.clone(),
            source,
            settings,
            speed_pct: 100,
            slots,
            public,
            started: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let mut members = HashMap::new();
        members.insert(conn, Member { info, tx });
        self.rooms.insert(
            id.clone(),
            Room {
                lobby,
                members,
                host: conn,
                game: None,
            },
        );
        self.by_code.insert(code, id.clone());
        self.where_is.insert(conn, id.clone());
        self.rooms.get_mut(&id).unwrap()
    }

    pub fn join(
        &mut self,
        conn: ConnId,
        info: PlayerInfo,
        tx: Tx,
        id: Option<String>,
        code: Option<String>,
    ) -> Result<&mut Room, &'static str> {
        self.leave(conn);
        let room_id = match (id, code) {
            (Some(id), _) if self.rooms.contains_key(&id) => id,
            (_, Some(code)) => self
                .by_code
                .get(&code.trim().to_uppercase())
                .cloned()
                .ok_or("no_such_room")?,
            _ => return Err("no_such_room"),
        };
        let room = self.rooms.get_mut(&room_id).ok_or("no_such_room")?;
        // Reconnect to an existing slot, else take the first open empty slot; else spectate.
        let existing = room
            .lobby
            .slots
            .iter()
            .position(|s| s.player.as_ref().map(|p| p.id == info.id).unwrap_or(false));
        if existing.is_none() && !room.lobby.started {
            if let Some(i) = room
                .lobby
                .slots
                .iter()
                .position(|s| s.open && s.player.is_none() && !s.is_ai)
            {
                room.lobby.slots[i].player = Some(info.clone());
                room.lobby.slots[i].ready = false;
            }
        }
        room.members.insert(conn, Member { info, tx });
        self.where_is.insert(conn, room_id.clone());
        Ok(self.rooms.get_mut(&room_id).unwrap())
    }

    /// Remove a connection from its room. Returns the room id if the room still exists.
    pub fn leave(&mut self, conn: ConnId) -> Option<String> {
        let room_id = self.where_is.remove(&conn)?;
        let room = self.rooms.get_mut(&room_id)?;
        let left = room.members.remove(&conn);
        if let Some(m) = left {
            if !room.lobby.started {
                for s in room.lobby.slots.iter_mut() {
                    if s.player
                        .as_ref()
                        .map(|p| p.id == m.info.id)
                        .unwrap_or(false)
                    {
                        s.player = None;
                        s.ready = false;
                    }
                }
            }
        }
        if room.members.is_empty() {
            let code = room.lobby.room_code.clone();
            if let Some(g) = room.game.take() {
                g.abandon();
            }
            self.rooms.remove(&room_id);
            self.by_code.remove(&code);
            return None;
        }
        if room.host == conn {
            // Hand the host role to the first remaining member.
            let next = *room.members.keys().min().unwrap();
            room.host = next;
            room.lobby.host = room.members[&next].info.display.clone();
        }
        room.broadcast_state();
        Some(room_id)
    }

    pub fn room_of(&mut self, conn: ConnId) -> Option<&mut Room> {
        let id = self.where_is.get(&conn)?.clone();
        self.rooms.get_mut(&id)
    }

    pub fn public_list(&self) -> Vec<LobbySummary> {
        let mut v: Vec<LobbySummary> = self
            .rooms
            .values()
            .filter(|r| r.lobby.public && !r.lobby.started)
            .map(|r| r.summary())
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Apply a slot change. Hosts may edit any slot, players only their own slot's civ/team/ready.
    pub fn set_slot(
        &mut self,
        conn: ConnId,
        slot: usize,
        change: SlotChange,
    ) -> Result<(), &'static str> {
        let room = self.room_of(conn).ok_or("not_in_lobby")?;
        if room.lobby.started {
            return Err("already_started");
        }
        let is_host = room.host == conn;
        let mine = room.slot_of(conn) == Some(slot);
        if slot >= room.lobby.slots.len() {
            return Err("bad_slot");
        }
        match change {
            SlotChange::Civ(c) if is_host || mine => {
                if !wars_data::data().civ_index.contains_key(&c) && c != "random" {
                    return Err("bad_civ");
                }
                room.lobby.slots[slot].civ = c;
            }
            SlotChange::Team(t) if is_host || mine => room.lobby.slots[slot].team = t.min(8),
            SlotChange::Color(c) if is_host || mine => room.lobby.slots[slot].color = c % 8,
            SlotChange::Ready(r) if mine => room.lobby.slots[slot].ready = r,
            SlotChange::Ai { is_ai, level } if is_host => {
                let s = &mut room.lobby.slots[slot];
                if s.player.is_some() {
                    return Err("slot_taken");
                }
                s.is_ai = is_ai;
                s.ai_level = level.min(2);
                s.open = s.open || is_ai;
            }
            SlotChange::Open(o) if is_host => {
                let s = &mut room.lobby.slots[slot];
                if s.player.is_some() && !o {
                    return Err("slot_taken");
                }
                s.open = o;
                if !o {
                    s.is_ai = false;
                }
            }
            SlotChange::Move(to) => {
                if to >= room.lobby.slots.len() {
                    return Err("bad_slot");
                }
                let from = room.slot_of(conn).ok_or("not_seated")?;
                if room.lobby.slots[to].player.is_some()
                    || room.lobby.slots[to].is_ai
                    || !room.lobby.slots[to].open
                {
                    return Err("slot_taken");
                }
                let p = room.lobby.slots[from].player.take();
                room.lobby.slots[from].ready = false;
                room.lobby.slots[to].player = p;
            }
            SlotChange::Kick if is_host => {
                let s = &mut room.lobby.slots[slot];
                let mut kicked: Vec<ConnId> = Vec::new();
                if let Some(p) = s.player.take() {
                    kicked = room
                        .members
                        .iter()
                        .filter(|(_, m)| m.info.id == p.id)
                        .map(|(c, _)| *c)
                        .collect();
                    for c in &kicked {
                        if let Some(m) = room.members.remove(c) {
                            let _ = m.tx.send(ServerMsg::LobbyClosed);
                        }
                    }
                }
                room.broadcast_state();
                for c in kicked {
                    self.where_is.remove(&c);
                }
                return Ok(());
            }
            _ => return Err("forbidden"),
        }
        room.broadcast_state();
        Ok(())
    }
}
