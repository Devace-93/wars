//! Multiplayer screens: login, lobby list, lobby room; hands a RemoteMatch to the game
//! screen when the host starts.

use crate::game::GameState;
use crate::{App, Screen};
use wars_client::net::{self, ws_url};
use wars_client::Deferred;
use wars_client::{RemoteMatch, Session, Transport};
use wars_mapgen::{MapSize, MapType};
use wars_proto::*;
use wars_proto::{LobbySummary, PlayerInfo, TokenResponse};
use wars_sim::ids::Age;
use wars_sim::world::{GameMode, Settings, VictoryMode, World};
use wars_ui::{Key, Rect, Ui, THEME};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Stage {
    #[default]
    Login,
    List,
    Room,
}

/// An in-flight network operation the screens poll every frame.
pub enum PendingOp {
    Guest(Deferred<Result<TokenResponse, String>>),
    Login(Deferred<Result<(String, String), String>>),
    Refresh(Deferred<Result<(String, String), String>>),
    Connect(Deferred<Result<Box<dyn Transport>, String>>),
    Lobbies(Deferred<Result<Vec<LobbySummary>, String>>),
    Me(Deferred<Result<Option<PlayerInfo>, String>>),
}

#[derive(Default)]
pub struct NetState {
    pub stage: Stage,
    pub token: Option<String>,
    pub refresh: Option<String>,
    pub me: Option<PlayerInfo>,
    pub transport: Option<Box<dyn Transport>>,
    pub busy_op: Option<PendingOp>,
    pub checked_cookie: bool,
    pub lobby: Option<Lobby>,
    pub lobbies: Vec<LobbySummary>,
    pub lobby_sel: Option<usize>,
    pub lobby_scroll: f32,
    pub email: String,
    pub password: String,
    pub code: String,
    pub room_name: String,
    pub error: Option<String>,
    pub busy: Option<String>,
    pub last_refresh: f64,
    pub chat: Vec<(String, String)>,
    pub chat_text: String,
    pub map_type: usize,
    pub map_size: usize,
    pub mode: usize,
    pub age: usize,
    pub speed: usize,
    pub public: bool,
    pub cheats: bool,
    pub lock_teams: bool,
}

fn poll(app: &mut App) {
    let data = app.data;
    let mut start: Option<(Lobby, Option<wars_sim::ids::PlayerId>, Vec<u8>, u32)> = None;
    {
        let net = &mut app.net;
        let Some(t) = net.transport.as_mut() else {
            return;
        };
        if t.is_closed() {
            net.error = Some("Connection lost".into());
            net.transport = None;
            net.stage = Stage::Login;
            return;
        }
        while let Some(frame) = t.try_recv() {
            match decode::<ServerMsg>(&frame) {
                Ok(ServerMsg::Welcome { you, .. }) => net.me = Some(you),
                Ok(ServerMsg::LobbyState(l)) => {
                    net.lobby = Some(l);
                    net.stage = Stage::Room;
                }
                Ok(ServerMsg::LobbyClosed) => {
                    net.lobby = None;
                    net.stage = Stage::List;
                }
                Ok(ServerMsg::Error { message, .. }) => net.error = Some(message),
                Ok(ServerMsg::Chat { name, text, .. }) => net.chat.push((name, text)),
                Ok(ServerMsg::MatchStart {
                    lobby,
                    you,
                    snapshot,
                    tick_rate,
                }) => {
                    start = Some((lobby, you, snapshot, tick_rate));
                    break;
                }
                _ => {}
            }
        }
    }
    if let Some((lobby, you, snapshot, tick_rate)) = start {
        match World::from_snapshot(&snapshot) {
            Ok(world) => {
                let transport = app.net.transport.take().unwrap();
                let rm = RemoteMatch::new(world, you, lobby, tick_rate, transport);
                app.game = Some(GameState::new(
                    Session::Remote(rm),
                    app.screen_w,
                    app.screen_h,
                    data,
                ));
                app.net.lobby = None;
                app.net.stage = Stage::List;
                app.screen = Screen::Game;
            }
            Err(e) => app.net.error = Some(format!("Bad snapshot: {e}")),
        }
    }
}

fn server_url(app: &App) -> String {
    // In the browser the game is served by the server it talks to.
    net::same_origin_server().unwrap_or_else(|| app.menu.server_url.clone())
}

fn connect(app: &mut App) {
    let server = server_url(app);
    let token = app.net.token.clone().unwrap_or_default();
    app.net.busy = Some("Connecting...".into());
    app.net.busy_op = Some(PendingOp::Connect(net::connect_deferred(
        ws_url(&server),
        token,
    )));
}

fn refresh_list(app: &mut App) {
    if app.net.busy_op.is_some() {
        return;
    }
    let server = server_url(app);
    let token = app.net.token.clone().unwrap_or_default();
    app.net.busy_op = Some(PendingOp::Lobbies(net::lobbies_deferred(server, token)));
    app.net.last_refresh = app.time;
}

/// Advance whatever network call is in flight.
fn poll_pending(app: &mut App) {
    let Some(op) = app.net.busy_op.as_mut() else {
        return;
    };
    let done: Option<Result<(), String>> = match op {
        PendingOp::Guest(d) => d.try_take().map(|r| {
            r.map(|t| {
                app.net.token = Some(t.token);
                app.net.me = Some(t.player);
            })
        }),
        PendingOp::Login(d) | PendingOp::Refresh(d) => d.try_take().map(|r| {
            r.map(|(jwt, refresh)| {
                app.net.token = Some(jwt);
                app.net.refresh = Some(refresh.clone());
                app.settings["refresh"] = serde_json::Value::String(refresh);
                app.settings["email"] = serde_json::Value::String(app.net.email.clone());
                wars_client::saves::save_settings(&app.settings);
                app.net.password.clear();
            })
        }),
        PendingOp::Connect(d) => d.try_take().map(|r| {
            r.map(|t| {
                app.net.transport = Some(t);
                app.net.stage = Stage::List;
                app.net.error = None;
            })
        }),
        PendingOp::Lobbies(d) => d.try_take().map(|r| r.map(|l| app.net.lobbies = l)),
        PendingOp::Me(d) => d.try_take().map(|r| {
            r.map(|me| {
                if let Some(p) = me {
                    // A cookie session: connect with an empty token, the server reads the cookie.
                    app.net.me = Some(p);
                    app.net.token = Some(String::new());
                }
            })
        }),
    };
    if let Some(result) = done {
        let was = app.net.busy_op.take();
        app.net.busy = None;
        match (was, result) {
            (_, Err(e)) => app.net.error = Some(e),
            (
                Some(PendingOp::Guest(_)) | Some(PendingOp::Login(_)) | Some(PendingOp::Refresh(_)),
                Ok(()),
            ) => {
                crate::menus::save_settings(app);
                connect(app);
            }
            (Some(PendingOp::Me(_)), Ok(())) => {
                if app.net.token.is_some() {
                    connect(app);
                }
            }
            (Some(PendingOp::Connect(_)), Ok(())) => refresh_list(app),
            _ => {}
        }
    }
}

fn send(app: &mut App, m: ClientMsg) {
    if let Some(t) = app.net.transport.as_mut() {
        t.send(encode(&m));
    }
}

pub fn frame(app: &mut App) {
    poll_pending(app);
    poll(app);
    if app.screen != Screen::Multiplayer {
        return;
    }
    if net::WEB
        && !app.net.checked_cookie
        && app.net.busy_op.is_none()
        && app.net.transport.is_none()
    {
        // The browser may already hold a 3m4 session cookie (after the login redirect).
        app.net.checked_cookie = true;
        app.net.busy = Some("Checking your session...".into());
        app.net.busy_op = Some(PendingOp::Me(net::me_deferred(
            server_url(app),
            String::new(),
        )));
    }
    let (w, h) = (app.screen_w, app.screen_h);
    let busy_now = app.net.busy_op.is_some();
    let data = app.data;
    let mut action: Option<&'static str> = None;
    let mut slot_change: Option<(usize, SlotChange)> = None;
    let mut source_change = false;
    let mut settings_change = false;
    let esc = app.frame.key(Key::Escape) && app.focus.is_none();
    if app.net.stage == Stage::List
        && app.net.transport.is_some()
        && app.time - app.net.last_refresh > 5.0
    {
        refresh_list(app);
    }
    {
        let net = &mut app.net;
        let menu = &mut app.menu;
        let mut ui = Ui::new(&mut app.dl, &mut app.text, &app.frame, &mut app.focus);
        ui.dl.rect(0.0, 0.0, w, h, THEME.bg);
        match net.stage {
            Stage::Login => {
                ui.heading(40.0, 30.0, "Multiplayer");
                let cw = (w - 120.0).min(520.0);
                let mut y = 80.0;
                let shown_server =
                    net::same_origin_server().unwrap_or_else(|| menu.server_url.clone());
                let hint = if net::WEB {
                    ""
                } else {
                    "  (change in Options)"
                };
                ui.label_dim(40.0, y, &format!("Server: {shown_server}{hint}"));
                y += 30.0;
                ui.text_field(
                    wars_ui::id("nick"),
                    Rect::new(40.0, y, cw, 30.0),
                    "Nickname",
                    &mut menu.nickname,
                    "Guest name",
                    false,
                );
                y += 36.0;
                if ui.button_styled(
                    Rect::new(40.0, y, 200.0, 36.0),
                    "Play as guest",
                    !busy_now,
                    THEME.accent_dark,
                ) {
                    action = Some("guest");
                }
                y += 60.0;
                if net::WEB {
                    ui.label_dim(40.0, y, "Or log in with your 3m4 account (needed for online saves and the scenario editor):");
                    y += 26.0;
                    if ui.button(Rect::new(40.0, y, 260.0, 36.0), "Log in with 3m4 account") {
                        action = Some("login_redirect");
                    }
                } else {
                    ui.label_dim(
                        40.0,
                        y,
                        "Or log in with your 3m4 account (email + password):",
                    );
                    y += 26.0;
                    ui.text_field(
                        wars_ui::id("email"),
                        Rect::new(40.0, y, cw, 30.0),
                        "Email",
                        &mut net.email,
                        "you@example.com",
                        false,
                    );
                    y += 36.0;
                    ui.text_field(
                        wars_ui::id("pw"),
                        Rect::new(40.0, y, cw, 30.0),
                        "Password",
                        &mut net.password,
                        "",
                        true,
                    );
                    y += 36.0;
                    if ui.button_styled(
                        Rect::new(40.0, y, 200.0, 36.0),
                        "Log in",
                        !busy_now,
                        THEME.button,
                    ) {
                        action = Some("login");
                    }
                    if net.refresh.is_some()
                        && ui.button(
                            Rect::new(260.0, y, 220.0, 36.0),
                            "Continue previous session",
                        )
                    {
                        action = Some("refresh");
                    }
                }
                y += 50.0;
                if ui.button(Rect::new(40.0, y, 120.0, 36.0), "Back") {
                    action = Some("back");
                }
                if let Some(e) = &net.error {
                    ui.label_color(40.0, y + 50.0, 14, e, THEME.danger);
                }
                if let Some(b) = &net.busy {
                    ui.label_color(40.0, y + 50.0, 14, b, THEME.text_dim);
                }
            }
            Stage::List => {
                ui.heading(40.0, 30.0, "Lobbies");
                if let Some(me) = &net.me {
                    ui.label_dim(
                        300.0,
                        36.0,
                        &format!(
                            "Logged in as {}{}",
                            me.display,
                            if me.guest { " (guest)" } else { "" }
                        ),
                    );
                }
                let items: Vec<String> = net
                    .lobbies
                    .iter()
                    .map(|l| {
                        format!(
                            "{}  ·  {}  ·  {}/{} players  ·  host {}",
                            l.name, l.map, l.players, l.max_players, l.host
                        )
                    })
                    .collect();
                let lr = Rect::new(40.0, 80.0, (w - 80.0).min(760.0), h - 260.0);
                if items.is_empty() {
                    ui.panel(lr);
                    ui.label_dim(
                        lr.x + 16.0,
                        lr.y + 16.0,
                        "No public games right now. Create one, or join a friend's room code.",
                    );
                } else {
                    ui.list(lr, &items, &mut net.lobby_sel, &mut net.lobby_scroll, 28.0);
                }
                let mut y = lr.bottom() + 16.0;
                if ui.button_styled(
                    Rect::new(40.0, y, 140.0, 36.0),
                    "Join",
                    net.lobby_sel.is_some(),
                    THEME.accent_dark,
                ) {
                    action = Some("join_sel");
                }
                if ui.button(Rect::new(200.0, y, 120.0, 36.0), "Refresh") {
                    action = Some("refresh_list");
                }
                ui.text_field(
                    wars_ui::id("code"),
                    Rect::new(340.0, y + 3.0, 140.0, 30.0),
                    "",
                    &mut net.code,
                    "Room code",
                    false,
                );
                if ui.button(Rect::new(490.0, y, 140.0, 36.0), "Join by code") {
                    action = Some("join_code");
                }
                y += 50.0;
                ui.text_field(
                    wars_ui::id("rname"),
                    Rect::new(40.0, y + 3.0, 280.0, 30.0),
                    "",
                    &mut net.room_name,
                    "New game name",
                    false,
                );
                ui.checkbox(Rect::new(340.0, y, 120.0, 36.0), "Public", &mut net.public);
                if ui.button_styled(
                    Rect::new(490.0, y, 140.0, 36.0),
                    "Create game",
                    true,
                    THEME.accent_dark,
                ) {
                    action = Some("create");
                }
                y += 50.0;
                if ui.button(Rect::new(40.0, y, 120.0, 36.0), "Log out") {
                    action = Some("logout");
                }
                if let Some(e) = &net.error {
                    ui.label_color(180.0, y + 8.0, 14, e, THEME.danger);
                }
            }
            Stage::Room => {
                let Some(lobby) = net.lobby.clone() else {
                    return;
                };
                let my_id = net.me.as_ref().map(|m| m.id.clone()).unwrap_or_default();
                let is_host = net
                    .me
                    .as_ref()
                    .map(|m| m.display == lobby.host)
                    .unwrap_or(false);
                ui.heading(
                    40.0,
                    24.0,
                    &format!("{}   —   room code {}", lobby.name, lobby.room_code),
                );
                ui.label_dim(
                    40.0,
                    54.0,
                    &format!(
                        "Host: {}   ·   {}",
                        lobby.host,
                        if lobby.public { "public" } else { "private" }
                    ),
                );
                // Slots table.
                let mut y = 90.0;
                ui.label_dim(40.0, y, "Slot");
                ui.label_dim(90.0, y, "Player");
                ui.label_dim(300.0, y, "Civilisation");
                ui.label_dim(520.0, y, "Team");
                ui.label_dim(640.0, y, "Ready");
                y += 24.0;
                let civ_names: Vec<String> = data.civs.iter().map(|c| c.name.clone()).collect();
                for (i, s) in lobby.slots.iter().enumerate() {
                    let r = Rect::new(40.0, y, 760.0, 30.0);
                    let mine = s.player.as_ref().map(|p| p.id == my_id).unwrap_or(false);
                    if mine {
                        ui.dl
                            .rect(r.x, r.y, r.w, r.h, THEME.accent_dark.alpha(0.25));
                    }
                    ui.dl.rect(
                        r.x,
                        r.y + 6.0,
                        10.0,
                        18.0,
                        wars_render::art::PLAYER_COLORS[(s.color as usize + 1) % 9],
                    );
                    ui.label(r.x + 18.0, y + 6.0, &format!("{}", i + 1));
                    if !s.open {
                        ui.label_dim(90.0, y + 6.0, "closed");
                        if is_host
                            && ui.button(Rect::new(760.0 - 60.0, y + 2.0, 60.0, 26.0), "Open")
                        {
                            slot_change = Some((i, SlotChange::Open(true)));
                        }
                        y += 34.0;
                        continue;
                    }
                    let name = match (&s.player, s.is_ai) {
                        (Some(p), _) => p.display.clone(),
                        (None, true) => format!(
                            "Computer ({})",
                            ["Easy", "Standard", "Hard"][s.ai_level as usize % 3]
                        ),
                        (None, false) => "— open —".into(),
                    };
                    ui.label(90.0, y + 6.0, &name);
                    let editable = is_host || mine;
                    let mut ci = data.civs.iter().position(|c| c.key == s.civ).unwrap_or(0);
                    if editable {
                        if ui.select(
                            Rect::new(300.0, y + 2.0, 210.0, 26.0),
                            "",
                            &civ_names[ci],
                            &mut ci,
                            civ_names.len(),
                        ) {
                            slot_change = Some((i, SlotChange::Civ(data.civs[ci].key.clone())));
                        }
                        let mut ti = s.team as usize;
                        if ui.select(
                            Rect::new(520.0, y + 2.0, 110.0, 26.0),
                            "",
                            &format!("{}", ti),
                            &mut ti,
                            9,
                        ) {
                            slot_change = Some((i, SlotChange::Team(ti as u8)));
                        }
                    } else {
                        ui.label(300.0, y + 6.0, &civ_names[ci]);
                        ui.label(520.0, y + 6.0, &format!("{}", s.team));
                    }
                    if s.player.is_some() {
                        let mut ready = s.ready;
                        if mine {
                            if ui.checkbox(Rect::new(640.0, y + 2.0, 60.0, 26.0), "", &mut ready) {
                                slot_change = Some((i, SlotChange::Ready(ready)));
                            }
                        } else {
                            ui.label_color(
                                640.0,
                                y + 6.0,
                                15,
                                if s.ready { "yes" } else { "no" },
                                if s.ready { THEME.ok } else { THEME.text_dim },
                            );
                        }
                    }
                    if is_host {
                        if s.player.is_none() {
                            let label = if s.is_ai { "No AI" } else { "Add AI" };
                            if ui.button(Rect::new(700.0, y + 2.0, 60.0, 26.0), label) {
                                slot_change = Some((
                                    i,
                                    SlotChange::Ai {
                                        is_ai: !s.is_ai,
                                        level: 1,
                                    },
                                ));
                            }
                            if ui.button(Rect::new(765.0, y + 2.0, 60.0, 26.0), "Close") {
                                slot_change = Some((i, SlotChange::Open(false)));
                            }
                        } else if !mine && ui.button(Rect::new(700.0, y + 2.0, 60.0, 26.0), "Kick")
                        {
                            slot_change = Some((i, SlotChange::Kick));
                        }
                    } else if s.player.is_none()
                        && !s.is_ai
                        && ui.button(Rect::new(700.0, y + 2.0, 60.0, 26.0), "Move")
                    {
                        slot_change = Some((i, SlotChange::Move(i)));
                    }
                    y += 34.0;
                }
                // Settings (host editable).
                y += 10.0;
                let mt = match &lobby.source {
                    MatchSource::Random { map_type, size, .. } => {
                        format!("{} · {}", map_type.name(), size.name())
                    }
                    MatchSource::Scenario { name, .. } => format!("Scenario: {name}"),
                    MatchSource::Save { name, .. } => format!("Save: {name}"),
                };
                if is_host {
                    if ui.select(
                        Rect::new(40.0, y, 360.0, 28.0),
                        "Map",
                        MapType::ALL[net.map_type % 6].name(),
                        &mut net.map_type,
                        6,
                    ) {
                        source_change = true;
                    }
                    if ui.select(
                        Rect::new(420.0, y, 300.0, 28.0),
                        "Size",
                        MapSize::ALL[net.map_size % 5].name(),
                        &mut net.map_size,
                        5,
                    ) {
                        source_change = true;
                    }
                    y += 34.0;
                    if ui.select(
                        Rect::new(40.0, y, 360.0, 28.0),
                        "Mode",
                        ["Random map", "Deathmatch", "Regicide"][net.mode % 3],
                        &mut net.mode,
                        3,
                    ) {
                        settings_change = true;
                    }
                    if ui.select(
                        Rect::new(420.0, y, 300.0, 28.0),
                        "Start age",
                        Age::ALL[net.age % 4].name(),
                        &mut net.age,
                        4,
                    ) {
                        settings_change = true;
                    }
                    y += 34.0;
                    if ui.select(
                        Rect::new(40.0, y, 360.0, 28.0),
                        "Speed",
                        ["Slow (0.75x)", "Normal", "Fast (1.5x)", "Very fast (2x)"][net.speed % 4],
                        &mut net.speed,
                        4,
                    ) {
                        settings_change = true;
                    }
                    if ui.checkbox(Rect::new(420.0, y, 140.0, 28.0), "Cheats", &mut net.cheats) {
                        settings_change = true;
                    }
                    if ui.checkbox(
                        Rect::new(580.0, y, 140.0, 28.0),
                        "Lock teams",
                        &mut net.lock_teams,
                    ) {
                        settings_change = true;
                    }
                } else {
                    ui.label_dim(
                        40.0,
                        y,
                        &format!("Map: {mt}   ·   speed {}%", lobby.speed_pct),
                    );
                    y += 34.0;
                }
                y += 44.0;
                // Chat.
                let chat_r = Rect::new(40.0, y, 760.0, (h - y - 90.0).max(60.0));
                ui.dl
                    .rect(chat_r.x, chat_r.y, chat_r.w, chat_r.h, THEME.field);
                let mut cy = chat_r.y + 6.0;
                let start = net
                    .chat
                    .len()
                    .saturating_sub(((chat_r.h - 12.0) / 18.0) as usize);
                for (n, t) in net.chat.iter().skip(start) {
                    ui.label_color(chat_r.x + 8.0, cy, 14, &format!("{n}: {t}"), THEME.text);
                    cy += 18.0;
                }
                let bottom = h - 80.0;
                ui.text_field(
                    wars_ui::id("lchat"),
                    Rect::new(40.0, bottom, 520.0, 30.0),
                    "",
                    &mut net.chat_text,
                    "Chat (Enter)",
                    false,
                );
                if is_host {
                    let n = lobby
                        .slots
                        .iter()
                        .filter(|s| s.open && (s.player.is_some() || s.is_ai))
                        .count();
                    if ui.button_styled(
                        Rect::new(580.0, bottom - 4.0, 220.0, 38.0),
                        "Start game",
                        n >= 2,
                        THEME.accent_dark,
                    ) {
                        action = Some("start");
                    }
                } else {
                    ui.label_dim(580.0, bottom + 6.0, "Waiting for the host to start...");
                }
                if ui.button(Rect::new(40.0, bottom + 40.0, 120.0, 30.0), "Leave") {
                    action = Some("leave");
                }
                if let Some(e) = &net.error {
                    ui.label_color(180.0, bottom + 46.0, 14, e, THEME.danger);
                }
            }
        }
    }
    // Chat enter.
    if app.net.stage == Stage::Room
        && app.frame.key(Key::Enter)
        && !app.net.chat_text.trim().is_empty()
    {
        let t = std::mem::take(&mut app.net.chat_text);
        send(
            app,
            ClientMsg::Chat {
                text: t,
                to_allies: false,
            },
        );
    }
    if let Some((slot, change)) = slot_change {
        send(app, ClientMsg::SetSlot { slot, change });
    }
    if source_change {
        let seed = (app.time * 1000.0) as u64 | 1;
        let source = MatchSource::Random {
            map_type: MapType::ALL[app.net.map_type % 6],
            size: MapSize::ALL[app.net.map_size % 5],
            seed,
        };
        send(app, ClientMsg::SetSource { source });
    }
    if settings_change {
        let settings = Settings {
            mode: [
                GameMode::RandomMap,
                GameMode::Deathmatch,
                GameMode::Regicide,
            ][app.net.mode % 3]
                .clone(),
            victory: VictoryMode::Standard,
            starting_age: Age::ALL[app.net.age % 4],
            allow_cheats: app.net.cheats,
            lock_teams: app.net.lock_teams,
            ..Settings::default()
        };
        let speed_pct = [75, 100, 150, 200][app.net.speed % 4];
        send(
            app,
            ClientMsg::SetSettings {
                settings,
                speed_pct,
            },
        );
    }
    if esc {
        action = Some(match app.net.stage {
            Stage::Login => "back",
            Stage::List => "logout",
            Stage::Room => "leave",
        });
    }
    match action {
        Some("back") => app.screen = Screen::Main,
        Some("guest") if app.net.busy_op.is_none() => {
            let nick = if app.menu.nickname.trim().is_empty() {
                "Guest".to_string()
            } else {
                app.menu.nickname.clone()
            };
            app.net.error = None;
            app.net.busy = Some("Getting a guest pass...".into());
            app.net.busy_op = Some(PendingOp::Guest(net::guest_token_deferred(
                server_url(app),
                nick,
            )));
        }
        Some("login") if app.net.busy_op.is_none() => {
            app.net.error = None;
            app.net.busy = Some("Logging in...".into());
            app.net.busy_op = Some(PendingOp::Login(net::login_deferred(
                app.menu.accounts_url.clone(),
                app.net.email.trim().to_string(),
                app.net.password.clone(),
            )));
        }
        Some("login_redirect") => {
            let here = net::same_origin_server()
                .map(|o| format!("{o}/play/"))
                .unwrap_or_default();
            net::login_redirect(&app.menu.accounts_url, &here);
        }
        Some("refresh") if app.net.busy_op.is_none() => {
            if let Some(r) = app.net.refresh.clone() {
                app.net.busy = Some("Resuming your session...".into());
                app.net.busy_op = Some(PendingOp::Refresh(net::refresh_deferred(
                    app.menu.accounts_url.clone(),
                    r,
                )));
            }
        }
        Some("logout") => {
            app.net.transport = None;
            app.net.token = None;
            app.net.me = None;
            app.net.busy_op = None;
            app.net.stage = Stage::Login;
        }
        Some("refresh_list") => refresh_list(app),
        Some("join_sel") => {
            if let Some(i) = app.net.lobby_sel {
                let id = app.net.lobbies[i].id.clone();
                send(
                    app,
                    ClientMsg::JoinLobby {
                        id: Some(id),
                        room_code: None,
                    },
                );
            }
        }
        Some("join_code") => {
            let code = app.net.code.trim().to_uppercase();
            if !code.is_empty() {
                send(
                    app,
                    ClientMsg::JoinLobby {
                        id: None,
                        room_code: Some(code),
                    },
                );
            }
        }
        Some("create") => {
            let name = app.net.room_name.clone();
            let seed = (app.time * 1000.0) as u64 | 1;
            let source = MatchSource::Random {
                map_type: MapType::ALL[app.net.map_type % 6],
                size: MapSize::ALL[app.net.map_size % 5],
                seed,
            };
            let public = app.net.public;
            app.net.chat.clear();
            send(
                app,
                ClientMsg::CreateLobby {
                    name,
                    source,
                    settings: Settings::default(),
                    public,
                },
            );
        }
        Some("leave") => {
            send(app, ClientMsg::LeaveLobby);
            app.net.lobby = None;
            app.net.stage = Stage::List;
        }
        Some("start") => send(app, ClientMsg::StartMatch),
        _ => {}
    }
    let _ = data;
}

impl NetState {
    pub fn load(settings: &serde_json::Value) -> NetState {
        NetState {
            refresh: settings
                .get("refresh")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            email: settings
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            public: true,
            lock_teams: true,
            map_size: 1,
            speed: 1,
            ..Default::default()
        }
    }
}
