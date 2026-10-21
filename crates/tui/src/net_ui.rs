//! Multiplayer screens: login (guest or accounts), lobby list and the lobby room.
//! Blocking HTTP calls are fine here; the WebSocket is polled once per frame.

use crate::app::centered;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use wars_client::net::{accounts_login, get_json, guest_token, ws_url, WsTransport};
use wars_client::saves;
use wars_client::session::Transport;
use wars_client::wars_mapgen::{MapSize, MapType};
use wars_client::wars_proto::{
    ClientMsg, Lobby, LobbySummary, MatchSource, PlayerInfo, ServerMsg, SlotChange,
};
use wars_client::wars_sim::world::{Settings, World};
use wars_client::{RemoteMatch, Session};

const AMBER: Color = Color::Rgb(245, 158, 11);

/// What we know after logging in.
#[derive(Clone)]
pub struct Conn {
    pub server_url: String,
    pub token: String,
    pub nickname: String,
}

impl Conn {
    pub fn take_settings(&self) -> Conn {
        self.clone()
    }
}

pub struct LoginScreen {
    pub field: usize,
    pub values: [String; 4],
    pub error: String,
}
const LOGIN_FIELDS: [&str; 4] = ["Nickname (guest)", "Email", "Password", "Server URL"];

pub enum LoginOutcome {
    Stay,
    Back,
    LoggedIn(Conn),
}

impl LoginScreen {
    pub fn new() -> LoginScreen {
        let s = saves::load_settings();
        let get = |k: &str, d: &str| s.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
        LoginScreen {
            field: 0,
            values: [
                get("nickname", "nerd"),
                get("email", ""),
                String::new(),
                get("server_url", "https://wars.3m4.net"),
            ],
            error: String::new(),
        }
    }

    pub fn key(&mut self, key: KeyEvent) -> LoginOutcome {
        match key.code {
            KeyCode::Esc => return LoginOutcome::Back,
            KeyCode::Up | KeyCode::BackTab => {
                self.field = (self.field + LOGIN_FIELDS.len() - 1) % LOGIN_FIELDS.len()
            }
            KeyCode::Down | KeyCode::Tab => self.field = (self.field + 1) % LOGIN_FIELDS.len(),
            KeyCode::Backspace => {
                self.values[self.field].pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.values[self.field].push(c)
            }
            KeyCode::Enter => return self.submit(),
            _ => {}
        }
        LoginOutcome::Stay
    }

    fn submit(&mut self) -> LoginOutcome {
        let server = self.values[3].trim().to_string();
        let mut settings = saves::load_settings();
        settings["server_url"] = serde_json::Value::String(server.clone());
        let use_account = self.field == 1 || self.field == 2;
        let result = if use_account {
            let accounts = settings
                .get("accounts_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://accounts.3m4.net")
                .to_string();
            let email = self.values[1].trim().to_string();
            accounts_login(&accounts, &email, &self.values[2]).map(|(session, refresh)| {
                settings["email"] = serde_json::Value::String(email.clone());
                settings["refresh"] = serde_json::Value::String(refresh);
                (session, email)
            })
        } else {
            let nick = self.values[0].trim().to_string();
            if nick.is_empty() {
                return {
                    self.error = "nickname required".into();
                    LoginOutcome::Stay
                };
            }
            guest_token(&server, &nick).map(|t| (t.token, t.player.display))
        };
        match result {
            Ok((token, nickname)) => {
                settings["nickname"] = serde_json::Value::String(nickname.clone());
                settings["token"] = serde_json::Value::String(token.clone());
                saves::save_settings(&settings);
                LoginOutcome::LoggedIn(Conn {
                    server_url: server,
                    token,
                    nickname,
                })
            }
            Err(e) => {
                self.error = e;
                LoginOutcome::Stay
            }
        }
    }

    pub fn draw(&self, f: &mut Frame) {
        let r = centered(f.area(), 62, 11);
        let mut lines = Vec::new();
        for (i, name) in LOGIN_FIELDS.iter().enumerate() {
            let style = if i == self.field {
                Style::new().fg(Color::Black).bg(AMBER)
            } else {
                Style::new()
            };
            let shown = if i == 2 {
                "*".repeat(self.values[i].len())
            } else {
                self.values[i].clone()
            };
            lines.push(Line::from(vec![
                Span::styled(format!(" {name:<18}"), Style::new().fg(Color::Gray)),
                Span::styled(format!(" {shown:<38}"), style),
            ]));
        }
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            " Enter on the nickname: play as guest; on email/password: sign in",
            Style::new().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            " Tab next field, Esc back",
            Style::new().fg(Color::DarkGray),
        )));
        if !self.error.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" {}", self.error),
                Style::new().fg(Color::Red),
            )));
        }
        f.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(" Multiplayer login ")),
            r,
        );
    }
}

pub struct LobbiesScreen {
    pub conn: Conn,
    pub lobbies: Vec<LobbySummary>,
    pub idx: usize,
    pub error: String,
    /// Room code being typed after `j`.
    pub code: Option<String>,
}

pub enum LobbiesOutcome {
    Stay,
    Back,
    Joined(Box<LobbyScreen>),
}

impl LobbiesScreen {
    pub fn new(conn: Conn) -> LobbiesScreen {
        let mut s = LobbiesScreen {
            conn,
            lobbies: Vec::new(),
            idx: 0,
            error: String::new(),
            code: None,
        };
        s.refresh();
        s
    }

    pub fn refresh(&mut self) {
        match get_json::<Vec<LobbySummary>>(&self.conn.server_url, "/api/lobbies", &self.conn.token)
        {
            Ok(l) => {
                self.lobbies = l;
                self.error.clear();
            }
            Err(e) => self.error = format!("lobby list: {e}"),
        }
        self.idx = self.idx.min(self.lobbies.len().saturating_sub(1));
    }

    fn connect(&mut self, first: ClientMsg) -> LobbiesOutcome {
        match LobbyScreen::connect(self.conn.clone(), first) {
            Ok(l) => LobbiesOutcome::Joined(Box::new(l)),
            Err(e) => {
                self.error = e;
                LobbiesOutcome::Stay
            }
        }
    }

    pub fn key(&mut self, key: KeyEvent) -> LobbiesOutcome {
        if let Some(code) = &mut self.code {
            match key.code {
                KeyCode::Esc => self.code = None,
                KeyCode::Backspace => {
                    code.pop();
                }
                KeyCode::Char(c) => code.push(c.to_ascii_uppercase()),
                KeyCode::Enter => {
                    let code = self.code.take().unwrap_or_default();
                    return self.connect(ClientMsg::JoinLobby {
                        id: None,
                        room_code: Some(code),
                    });
                }
                _ => {}
            }
            return LobbiesOutcome::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return LobbiesOutcome::Back,
            KeyCode::Up | KeyCode::Char('k') if !self.lobbies.is_empty() => {
                self.idx = (self.idx + self.lobbies.len() - 1) % self.lobbies.len()
            }
            KeyCode::Down | KeyCode::Char('j') if !self.lobbies.is_empty() => {
                self.idx = (self.idx + 1) % self.lobbies.len()
            }
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('J') => self.code = Some(String::new()),
            KeyCode::Char('c') => {
                let name = format!("{}'s game", self.conn.nickname);
                return self.connect(ClientMsg::CreateLobby {
                    name,
                    source: MatchSource::Random {
                        map_type: MapType::Arabia,
                        size: MapSize::Tiny,
                        seed: wars_client::session::now_ms() % 1_000_000,
                    },
                    settings: Settings::default(),
                    public: true,
                });
            }
            KeyCode::Enter if !self.lobbies.is_empty() => {
                let id = self.lobbies[self.idx].id.clone();
                return self.connect(ClientMsg::JoinLobby {
                    id: Some(id),
                    room_code: None,
                });
            }
            _ => {}
        }
        LobbiesOutcome::Stay
    }

    pub fn draw(&self, f: &mut Frame) {
        let area = f.area();
        let r = centered(
            area,
            72,
            (self.lobbies.len() as u16 + 8).clamp(10, area.height),
        );
        let mut lines = vec![Line::from(Span::styled(
            format!(
                " {:<26}{:<14}{:<16}{:<8}{}",
                "Name", "Host", "Map", "Players", "State"
            ),
            Style::new().underlined(),
        ))];
        if self.lobbies.is_empty() {
            lines.push(Line::from(Span::styled(
                " no open lobbies",
                Style::new().fg(Color::DarkGray),
            )));
        }
        for (i, l) in self.lobbies.iter().enumerate() {
            let style = if i == self.idx {
                Style::new().fg(Color::Black).bg(AMBER)
            } else {
                Style::new()
            };
            lines.push(Line::from(Span::styled(
                format!(
                    " {:<26}{:<14}{:<16}{:<8}{}",
                    truncate(&l.name, 24),
                    truncate(&l.host, 12),
                    truncate(&l.map, 14),
                    format!("{}/{}", l.players, l.max_players),
                    if l.started { "playing" } else { "open" }
                ),
                style,
            )));
        }
        lines.push(Line::raw(""));
        match &self.code {
            Some(c) => lines.push(Line::from(vec![
                Span::raw(" room code: "),
                Span::styled(format!("{c}_"), Style::new().fg(Color::Yellow)),
            ])),
            None => lines.push(Line::from(Span::styled(
                " Enter join, c create, J join by room code, r refresh, Esc back",
                Style::new().fg(Color::DarkGray),
            ))),
        }
        if !self.error.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" {}", self.error),
                Style::new().fg(Color::Red),
            )));
        }
        f.render_widget(
            Paragraph::new(lines)
                .block(Block::bordered().title(format!(" Lobbies — {} ", self.conn.nickname))),
            r,
        );
    }
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub struct LobbyScreen {
    pub conn: Conn,
    transport: Option<WsTransport>,
    pub me: Option<PlayerInfo>,
    pub lobby: Option<Lobby>,
    pub slot_idx: usize,
    pub chat: Vec<String>,
    pub error: String,
    pub left: bool,
    pub chat_line: Option<String>,
}

impl LobbyScreen {
    /// Open the socket, wait for Welcome, send the create/join message and wait for the
    /// first lobby state (or an error).
    pub fn connect(conn: Conn, first: ClientMsg) -> Result<LobbyScreen, String> {
        let mut t = WsTransport::connect(&ws_url(&conn.server_url), &conn.token)?;
        let mut me = None;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while std::time::Instant::now() < deadline {
            let Some(frame) = t.recv_timeout(200) else {
                continue;
            };
            match wars_client::wars_proto::decode::<ServerMsg>(&frame) {
                Ok(ServerMsg::Welcome { you, .. }) => {
                    me = Some(you);
                    break;
                }
                Ok(ServerMsg::Error { message, .. }) => return Err(message),
                _ => {}
            }
        }
        let me = me.ok_or_else(|| "server did not answer the handshake".to_string())?;
        t.send(wars_client::wars_proto::encode(&first));
        let mut lobby = None;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while std::time::Instant::now() < deadline {
            let Some(frame) = t.recv_timeout(200) else {
                continue;
            };
            match wars_client::wars_proto::decode::<ServerMsg>(&frame) {
                Ok(ServerMsg::LobbyState(l)) => {
                    lobby = Some(l);
                    break;
                }
                Ok(ServerMsg::Error { message, .. }) => return Err(message),
                _ => {}
            }
        }
        let lobby = lobby.ok_or_else(|| "no lobby state received".to_string())?;
        Ok(LobbyScreen {
            conn,
            transport: Some(t),
            me: Some(me),
            lobby: Some(lobby),
            slot_idx: 0,
            chat: Vec::new(),
            error: String::new(),
            left: false,
            chat_line: None,
        })
    }

    fn send(&mut self, msg: &ClientMsg) {
        if let Some(t) = &mut self.transport {
            t.send(wars_client::wars_proto::encode(msg));
        }
    }

    pub fn my_slot(&self) -> Option<usize> {
        let me = self.me.as_ref()?;
        self.lobby
            .as_ref()?
            .slots
            .iter()
            .position(|s| s.player.as_ref().map(|p| p.id == me.id).unwrap_or(false))
    }
    pub fn is_host(&self) -> bool {
        match (&self.me, &self.lobby) {
            (Some(me), Some(l)) => l.host == me.id,
            _ => false,
        }
    }

    /// Drain the socket; returns a session when the match starts.
    pub fn poll(&mut self) -> Option<Session> {
        let mut frames = Vec::new();
        if let Some(t) = &mut self.transport {
            while let Some(f) = t.try_recv() {
                frames.push(f);
            }
            if t.is_closed() && frames.is_empty() {
                self.error = "connection closed".into();
            }
        }
        for frame in frames {
            match wars_client::wars_proto::decode::<ServerMsg>(&frame) {
                Ok(ServerMsg::LobbyState(l)) => self.lobby = Some(l),
                Ok(ServerMsg::Chat { name, text, .. }) => self.chat.push(format!("{name}: {text}")),
                Ok(ServerMsg::Error { message, .. }) => self.error = message,
                Ok(ServerMsg::LobbyClosed) => {
                    self.error = "lobby closed".into();
                    self.left = true;
                }
                Ok(ServerMsg::PlayerLeft { name, .. }) => self.chat.push(format!("{name} left")),
                Ok(ServerMsg::MatchStart {
                    lobby,
                    you,
                    snapshot,
                    tick_rate,
                }) => {
                    let world = match World::from_snapshot(&snapshot) {
                        Ok(w) => w,
                        Err(e) => {
                            self.error = format!("bad snapshot: {e}");
                            continue;
                        }
                    };
                    let transport = self.transport.take()?;
                    return Some(Session::Remote(RemoteMatch::new(
                        world,
                        you,
                        lobby,
                        tick_rate,
                        Box::new(transport),
                    )));
                }
                _ => {}
            }
        }
        None
    }

    pub fn key(&mut self, key: KeyEvent) {
        if let Some(line) = &mut self.chat_line {
            match key.code {
                KeyCode::Esc => self.chat_line = None,
                KeyCode::Backspace => {
                    line.pop();
                }
                KeyCode::Char(c) => line.push(c),
                KeyCode::Enter => {
                    let text = self.chat_line.take().unwrap_or_default();
                    let text = text.strip_prefix("chat ").unwrap_or(&text).to_string();
                    if !text.is_empty() {
                        self.send(&ClientMsg::Chat {
                            text,
                            to_allies: false,
                        });
                    }
                }
                _ => {}
            }
            return;
        }
        let n = self.lobby.as_ref().map(|l| l.slots.len()).unwrap_or(0);
        let my = self.my_slot();
        let host = self.is_host();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.send(&ClientMsg::LeaveLobby);
                self.left = true;
            }
            KeyCode::Up | KeyCode::Char('k') if n > 0 => {
                self.slot_idx = (self.slot_idx + n - 1) % n
            }
            KeyCode::Down | KeyCode::Char('j') if n > 0 => self.slot_idx = (self.slot_idx + 1) % n,
            KeyCode::Char(':') => self.chat_line = Some(String::new()),
            KeyCode::Char('c') => {
                if let (Some(slot), Some(l)) = (my, &self.lobby) {
                    let data = wars_client::wars_data::data();
                    let cur = &l.slots[slot].civ;
                    let i = data.civs.iter().position(|c| &c.key == cur).unwrap_or(0);
                    let next = data.civs[(i + 1) % data.civs.len()].key.clone();
                    self.send(&ClientMsg::SetSlot {
                        slot,
                        change: SlotChange::Civ(next),
                    });
                }
            }
            KeyCode::Char('t') => {
                if let (Some(slot), Some(l)) = (my, &self.lobby) {
                    let team = l.slots[slot].team % 8 + 1;
                    self.send(&ClientMsg::SetSlot {
                        slot,
                        change: SlotChange::Team(team),
                    });
                }
            }
            KeyCode::Char('r') => {
                if let (Some(slot), Some(l)) = (my, &self.lobby) {
                    let ready = !l.slots[slot].ready;
                    self.send(&ClientMsg::SetSlot {
                        slot,
                        change: SlotChange::Ready(ready),
                    });
                }
            }
            KeyCode::Char('a') if host => {
                if let Some(l) = &self.lobby {
                    if let Some(s) = l.slots.get(self.slot_idx) {
                        if s.player.is_none() {
                            self.send(&ClientMsg::SetSlot {
                                slot: self.slot_idx,
                                change: SlotChange::Ai {
                                    is_ai: !s.is_ai,
                                    level: 1,
                                },
                            });
                        }
                    }
                }
            }
            KeyCode::Char('o') if host => {
                if let Some(l) = &self.lobby {
                    if let Some(s) = l.slots.get(self.slot_idx) {
                        self.send(&ClientMsg::SetSlot {
                            slot: self.slot_idx,
                            change: SlotChange::Open(!s.open),
                        });
                    }
                }
            }
            KeyCode::Char('x') if host => {
                if let Some(l) = &self.lobby {
                    if l.slots
                        .get(self.slot_idx)
                        .map(|s| s.player.is_some())
                        .unwrap_or(false)
                        && my != Some(self.slot_idx)
                    {
                        self.send(&ClientMsg::SetSlot {
                            slot: self.slot_idx,
                            change: SlotChange::Kick,
                        });
                    }
                }
            }
            KeyCode::Char('s') if host => self.send(&ClientMsg::StartMatch),
            _ => {}
        }
    }

    pub fn draw(&self, f: &mut Frame) {
        let area = f.area();
        let r = centered(area, 76, area.height.min(24));
        let mut lines = Vec::new();
        let Some(l) = &self.lobby else {
            lines.push(Line::raw(" waiting for the lobby..."));
            f.render_widget(Paragraph::new(lines).block(Block::bordered()), r);
            return;
        };
        let source = match &l.source {
            MatchSource::Random {
                map_type,
                size,
                seed,
            } => format!("{} {} seed {}", map_type.name(), size.name(), seed),
            MatchSource::Scenario { name, .. } => format!("scenario {name}"),
            MatchSource::Save { name, .. } => format!("save {name}"),
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", l.name), Style::new().bold()),
            Span::styled(
                format!("room {}  {}  host {}", l.room_code, source, l.host),
                Style::new().fg(Color::Gray),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!(
                " {:<3}{:<22}{:<14}{:<6}{:<7}{}",
                "#", "Player", "Civ", "Team", "Ready", "State"
            ),
            Style::new().underlined(),
        )));
        let data = wars_client::wars_data::data();
        for (i, s) in l.slots.iter().enumerate() {
            let who = match &s.player {
                Some(p) => p.display.clone(),
                None if s.is_ai => format!(
                    "Computer ({})",
                    ["easy", "standard", "hard"][(s.ai_level as usize).min(2)]
                ),
                None => "-".into(),
            };
            let civ = data
                .civ_id(&s.civ)
                .map(|c| data.civ(c).name.clone())
                .unwrap_or_else(|| s.civ.clone());
            let mut style = Style::new().fg(crate::view::player_color(
                wars_client::wars_sim::ids::PlayerId(s.color + 1),
            ));
            if i == self.slot_idx {
                style = style.bg(Color::Rgb(50, 50, 50));
            }
            if !s.open {
                style = style.fg(Color::DarkGray);
            }
            lines.push(Line::from(Span::styled(
                format!(
                    " {:<3}{:<22}{:<14}{:<6}{:<7}{}",
                    i + 1,
                    truncate(&who, 20),
                    civ,
                    s.team,
                    if s.ready || s.is_ai { "yes" } else { "no" },
                    if s.open { "open" } else { "closed" }
                ),
                style,
            )));
        }
        lines.push(Line::raw(""));
        for c in self.chat.iter().rev().take(5).rev() {
            lines.push(Line::from(Span::styled(
                format!(" {c}"),
                Style::new().fg(Color::Rgb(190, 190, 190)),
            )));
        }
        lines.push(Line::raw(""));
        let mut hint = String::from(" c civ  t team  r ready  : chat  Esc leave");
        if self.is_host() {
            hint.push_str("  |  host: a add/remove AI  o open/close  x kick  s start");
        }
        lines.push(Line::from(Span::styled(
            hint,
            Style::new().fg(Color::DarkGray),
        )));
        if let Some(c) = &self.chat_line {
            lines.push(Line::from(vec![
                Span::styled(" :", Style::new().fg(Color::Yellow)),
                Span::raw(c.clone()),
                Span::styled("_", Style::new().fg(Color::Yellow)),
            ]));
        }
        if !self.error.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" {}", self.error),
                Style::new().fg(Color::Red),
            )));
        }
        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(" Lobby ")),
            r,
        );
    }
}
