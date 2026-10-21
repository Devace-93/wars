//! Menu screens: main, single player setup, load, scenarios, options, credits.

use crate::game::GameState;
use crate::{App, Screen};
use wars_client::{LocalMatch, Session};
use wars_mapgen::{MapParams, MapSize, MapType, PlayerSetup};
use wars_render::draw::Color;
use wars_sim::ids::{Age, PlayerId};
use wars_sim::world::{GameMode, Settings, VictoryMode};
use wars_ui::{Key, Rect, Ui, THEME};

pub struct MenuState {
    pub player_name: String,
    pub map_type: usize,
    pub map_size: usize,
    pub civ: usize,
    pub opponents: usize,
    pub ai_level: usize,
    pub mode: usize,
    pub start_age: usize,
    pub victory: usize,
    pub reveal: bool,
    pub cheats: bool,
    pub teams: bool,
    pub seed: String,
    pub saves: Vec<(String, std::path::PathBuf)>,
    pub save_sel: Option<usize>,
    pub save_scroll: f32,
    pub scenarios: Vec<(String, std::path::PathBuf)>,
    pub scen_sel: Option<usize>,
    pub scen_scroll: f32,
    pub server_url: String,
    pub accounts_url: String,
    pub nickname: String,
    pub return_to_game: bool,
    pub editor_name: String,
    pub editor_size: usize,
    pub error: Option<String>,
}

impl MenuState {
    pub fn new(settings: &serde_json::Value) -> MenuState {
        let s = |k: &str, d: &str| {
            settings
                .get(k)
                .and_then(|v| v.as_str())
                .unwrap_or(d)
                .to_string()
        };
        MenuState {
            player_name: s("player_name", "Player"),
            map_type: 0,
            map_size: 1,
            civ: 0,
            opponents: 1,
            ai_level: 1,
            mode: 0,
            start_age: 0,
            victory: 0,
            reveal: false,
            cheats: false,
            teams: false,
            seed: String::new(),
            saves: Vec::new(),
            save_sel: None,
            save_scroll: 0.0,
            scenarios: Vec::new(),
            scen_sel: None,
            scen_scroll: 0.0,
            server_url: s("server_url", "https://wars.3m4.net"),
            accounts_url: s("accounts_url", "https://accounts.3m4.net"),
            nickname: s("nickname", ""),
            return_to_game: false,
            editor_name: "My scenario".into(),
            editor_size: 0,
            error: None,
        }
    }
}

pub fn save_settings(app: &mut App) {
    app.settings["player_name"] = serde_json::Value::String(app.menu.player_name.clone());
    app.settings["server_url"] = serde_json::Value::String(app.menu.server_url.clone());
    app.settings["accounts_url"] = serde_json::Value::String(app.menu.accounts_url.clone());
    app.settings["nickname"] = serde_json::Value::String(app.menu.nickname.clone());
    wars_client::saves::save_settings(&app.settings);
}

fn background(ui: &mut Ui, w: f32, h: f32, time: f64) {
    ui.dl.rect(0.0, 0.0, w, h, THEME.bg);
    // A slow-moving isometric pattern for atmosphere.
    let t = (time * 8.0) as f32 % 64.0;
    let mut y = -64.0;
    while y < h + 64.0 {
        let mut x = -64.0 - t;
        while x < w + 64.0 {
            let c = Color::rgba(245, 158, 11, 10);
            ui.dl.poly(
                &[
                    [x, y],
                    [x + 32.0, y + 16.0],
                    [x, y + 32.0],
                    [x - 32.0, y + 16.0],
                ],
                c,
            );
            x += 64.0;
        }
        y += 32.0;
    }
}

pub fn start_single_player(app: &mut App) {
    let m = &app.menu;
    let data = app.data;
    let civs: Vec<String> = data.civs.iter().map(|c| c.key.clone()).collect();
    let mut players = vec![PlayerSetup {
        name: m.player_name.clone(),
        civ: civs[m.civ % civs.len()].clone(),
        team: 1,
        is_ai: false,
        ai_level: 1,
    }];
    for i in 0..m.opponents {
        let civ = civs[(m.civ + 1 + i * 3) % civs.len()].clone();
        let team = if m.teams { 2 } else { (i + 2) as u8 };
        players.push(PlayerSetup {
            name: format!("Computer {}", i + 1),
            civ,
            team,
            is_ai: true,
            ai_level: m.ai_level as u8,
        });
    }
    let mode = [
        GameMode::RandomMap,
        GameMode::Deathmatch,
        GameMode::Regicide,
    ][m.mode % 3]
        .clone();
    let victory = [
        VictoryMode::Standard,
        VictoryMode::Conquest,
        VictoryMode::TimeLimit(30),
    ][m.victory % 3]
        .clone();
    let settings = Settings {
        mode,
        victory,
        starting_age: Age::ALL[m.start_age % 4],
        reveal_map: m.reveal,
        allow_cheats: m.cheats,
        ..Settings::default()
    };
    let seed = if m.seed.trim().is_empty() {
        (app.time * 1000.0) as u64 ^ 0x5eed
    } else {
        m.seed.trim().parse().unwrap_or_else(|_| {
            m.seed
                .bytes()
                .fold(7u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))
        })
    };
    let params = MapParams {
        map_type: MapType::ALL[m.map_type % 6],
        size: MapSize::ALL[m.map_size % 5],
        seed,
        players,
        settings,
    };
    let world = wars_mapgen::generate(data, &params);
    let session = Session::Local(LocalMatch::new(world, PlayerId(1)));
    app.game = Some(GameState::new(session, app.screen_w, app.screen_h, data));
    app.screen = Screen::Game;
    save_settings(app);
}

pub fn frame(app: &mut App) {
    let (w, h) = (app.screen_w, app.screen_h);
    let time = app.time;
    let data = app.data;
    let screen = app.screen;
    let mut next: Option<Screen> = None;
    let mut action: Option<&'static str> = None;
    let esc = app.frame.key(Key::Escape);
    {
        let menu = &mut app.menu;
        let mut ui = Ui::new(&mut app.dl, &mut app.text, &app.frame, &mut app.focus);
        background(&mut ui, w, h, time);
        match screen {
            Screen::Main => {
                ui.title(w / 2.0, 70.0, "WARS");
                ui.centered(
                    w / 2.0,
                    125.0,
                    15,
                    "an age-advancing real-time strategy game",
                    THEME.text_dim,
                );
                let bw = 300.0;
                let x = (w - bw) / 2.0;
                let mut y = 190.0;
                for (label, target) in [
                    ("Single player", "single"),
                    ("Multiplayer", "multi"),
                    ("Load game", "load"),
                    ("Scenario editor", "editor"),
                    ("Options", "options"),
                    ("Credits", "credits"),
                    ("Quit", "quit"),
                ] {
                    if ui.button(Rect::new(x, y, bw, 40.0), label) {
                        action = Some(target);
                    }
                    y += 50.0;
                }
                ui.centered(
                    w / 2.0,
                    h - 30.0,
                    12,
                    "wars.3m4.net · MIT · built with Rust",
                    THEME.text_dim,
                );
            }
            Screen::SinglePlayer => {
                ui.heading(40.0, 20.0, "Single player");
                let compact = h < 620.0;
                let x = 40.0;
                let cw = if compact {
                    (w / 2.0 - 60.0).min(520.0)
                } else {
                    (w - 120.0).min(640.0)
                };
                let x2 = if compact { w / 2.0 + 10.0 } else { x };
                let mut y = 60.0;
                let row = if compact { 32.0 } else { 36.0 };
                let fh = row - 6.0;
                ui.text_field(
                    wars_ui::id("pname"),
                    Rect::new(x, y, cw, fh),
                    "Your name",
                    &mut menu.player_name,
                    "Player",
                    false,
                );
                y += row;
                let civ_names: Vec<String> = data.civs.iter().map(|c| c.name.clone()).collect();
                ui.select(
                    Rect::new(x, y, cw, fh),
                    "Civilisation",
                    &civ_names[menu.civ % civ_names.len()],
                    &mut menu.civ,
                    civ_names.len(),
                );
                y += row;
                let desc = &data.civs[menu.civ % civ_names.len()];
                ui.label_dim(x + 150.0, y, &desc.description);
                y += 20.0;
                for (i, b) in desc.bonus_text.iter().enumerate().take(3) {
                    ui.label_color(
                        x + 150.0,
                        y + i as f32 * 15.0,
                        12,
                        &format!("· {b}"),
                        THEME.text_dim,
                    );
                }
                y += 52.0;
                ui.select(
                    Rect::new(x, y, cw, fh),
                    "Map",
                    MapType::ALL[menu.map_type % 6].name(),
                    &mut menu.map_type,
                    6,
                );
                y += row;
                ui.select(
                    Rect::new(x, y, cw, fh),
                    "Size",
                    &format!(
                        "{} ({} tiles)",
                        MapSize::ALL[menu.map_size % 5].name(),
                        MapSize::ALL[menu.map_size % 5].tiles()
                    ),
                    &mut menu.map_size,
                    5,
                );
                y += row;
                ui.select(
                    Rect::new(x, y, cw, fh),
                    "Opponents",
                    &format!("{}", menu.opponents),
                    &mut menu.opponents,
                    8,
                );
                if menu.opponents == 0 {
                    menu.opponents = 1;
                }
                y += row;
                ui.select(
                    Rect::new(x, y, cw, fh),
                    "AI level",
                    ["Easy", "Standard", "Hard"][menu.ai_level % 3],
                    &mut menu.ai_level,
                    3,
                );
                y += row;
                let mut y2 = if compact { 60.0 } else { y };
                ui.select(
                    Rect::new(x2, y2, cw, fh),
                    "Game mode",
                    ["Random map", "Deathmatch", "Regicide"][menu.mode % 3],
                    &mut menu.mode,
                    3,
                );
                y2 += row;
                ui.select(
                    Rect::new(x2, y2, cw, fh),
                    "Starting age",
                    Age::ALL[menu.start_age % 4].name(),
                    &mut menu.start_age,
                    4,
                );
                y2 += row;
                ui.select(
                    Rect::new(x2, y2, cw, fh),
                    "Victory",
                    ["Standard", "Conquest", "Score at 30 min"][menu.victory % 3],
                    &mut menu.victory,
                    3,
                );
                y2 += row;
                ui.checkbox(
                    Rect::new(x2, y2, 200.0, 26.0),
                    "Opponents in one team",
                    &mut menu.teams,
                );
                y2 += row;
                ui.checkbox(
                    Rect::new(x2, y2, 160.0, 26.0),
                    "Reveal map",
                    &mut menu.reveal,
                );
                ui.checkbox(
                    Rect::new(x2 + 180.0, y2, 160.0, 26.0),
                    "Allow cheats",
                    &mut menu.cheats,
                );
                y2 += row;
                ui.text_field(
                    wars_ui::id("seed"),
                    Rect::new(x2, y2, cw, fh),
                    "Seed (optional)",
                    &mut menu.seed,
                    "random",
                    false,
                );
                y2 += row + 8.0;
                if ui.button_styled(
                    Rect::new(x2, y2, 200.0, 40.0),
                    "Start game",
                    true,
                    THEME.accent_dark,
                ) {
                    action = Some("start");
                }
                if ui.button(Rect::new(x2 + 220.0, y2, 120.0, 40.0), "Back") {
                    next = Some(Screen::Main);
                }
            }
            Screen::LoadGame => {
                ui.heading(40.0, 30.0, "Load game");
                if menu.saves.is_empty() {
                    menu.saves = wars_client::saves::list_saves();
                }
                let items: Vec<String> = menu.saves.iter().map(|(n, _)| n.clone()).collect();
                let lr = Rect::new(40.0, 80.0, (w - 80.0).min(640.0), h - 200.0);
                if items.is_empty() {
                    ui.panel(lr);
                    ui.label_dim(
                        lr.x + 16.0,
                        lr.y + 16.0,
                        "No saved games yet. Save with F11 during a match.",
                    );
                } else if let Some(i) =
                    ui.list(lr, &items, &mut menu.save_sel, &mut menu.save_scroll, 28.0)
                {
                    let _ = i;
                }
                let y = lr.bottom() + 16.0;
                if ui.button_styled(
                    Rect::new(40.0, y, 160.0, 40.0),
                    "Load",
                    menu.save_sel.is_some(),
                    THEME.accent_dark,
                ) {
                    action = Some("load");
                }
                if ui.button_styled(
                    Rect::new(220.0, y, 120.0, 40.0),
                    "Delete",
                    menu.save_sel.is_some(),
                    THEME.danger.mul(0.6),
                ) {
                    action = Some("delete_save");
                }
                if ui.button(Rect::new(360.0, y, 120.0, 40.0), "Back") {
                    action = Some("back_from_load");
                }
            }
            Screen::Scenarios => {
                ui.heading(40.0, 30.0, "Scenarios");
                if menu.scenarios.is_empty() {
                    menu.scenarios = wars_client::saves::list_scenarios();
                }
                let items: Vec<String> = menu.scenarios.iter().map(|(n, _)| n.clone()).collect();
                let lr = Rect::new(40.0, 80.0, (w - 80.0).min(640.0), h - 260.0);
                if items.is_empty() {
                    ui.panel(lr);
                    ui.label_dim(
                        lr.x + 16.0,
                        lr.y + 16.0,
                        "No scenarios yet. Create one below.",
                    );
                } else {
                    ui.list(lr, &items, &mut menu.scen_sel, &mut menu.scen_scroll, 28.0);
                }
                let mut y = lr.bottom() + 16.0;
                if ui.button_styled(
                    Rect::new(40.0, y, 160.0, 40.0),
                    "Play",
                    menu.scen_sel.is_some(),
                    THEME.accent_dark,
                ) {
                    action = Some("play_scenario");
                }
                if ui.button_styled(
                    Rect::new(220.0, y, 120.0, 40.0),
                    "Edit",
                    menu.scen_sel.is_some(),
                    THEME.button,
                ) {
                    action = Some("edit_scenario");
                }
                if ui.button(Rect::new(360.0, y, 120.0, 40.0), "Back") {
                    next = Some(Screen::Main);
                }
                y += 50.0;
                ui.text_field(
                    wars_ui::id("edname"),
                    Rect::new(40.0, y, 320.0, 30.0),
                    "New scenario",
                    &mut menu.editor_name,
                    "Name",
                    false,
                );
                ui.select(
                    Rect::new(380.0, y, 220.0, 30.0),
                    "",
                    &format!("{} tiles", [96, 120, 144][menu.editor_size % 3]),
                    &mut menu.editor_size,
                    3,
                );
                if ui.button(Rect::new(620.0, y - 5.0, 120.0, 40.0), "Create") {
                    action = Some("new_scenario");
                }
                if let Some(e) = &menu.error {
                    ui.label_color(40.0, y + 40.0, 14, e, THEME.danger);
                }
            }
            Screen::Options => {
                ui.heading(40.0, 30.0, "Options");
                let cw = (w - 120.0).min(640.0);
                let mut y = 80.0;
                ui.text_field(
                    wars_ui::id("srv"),
                    Rect::new(40.0, y, cw, 30.0),
                    "Server URL",
                    &mut menu.server_url,
                    "https://wars.3m4.net",
                    false,
                );
                y += 40.0;
                ui.text_field(
                    wars_ui::id("acc"),
                    Rect::new(40.0, y, cw, 30.0),
                    "Accounts URL",
                    &mut menu.accounts_url,
                    "https://accounts.3m4.net",
                    false,
                );
                y += 40.0;
                ui.text_field(
                    wars_ui::id("nick"),
                    Rect::new(40.0, y, cw, 30.0),
                    "Nickname",
                    &mut menu.nickname,
                    "shown online",
                    false,
                );
                y += 50.0;
                ui.label_dim(40.0, y, "Controls: left click select, drag box-select, right click act, wheel zoom, arrows/edges scroll.");
                y += 20.0;
                ui.label_dim(40.0, y, "Grid keys Q W E R T / A S D F G / Z X C V B match the command panel. H town centre, . idle villager.");
                y += 20.0;
                ui.label_dim(40.0, y, "Ctrl+1-9 set control group, 1-9 recall. Enter chat. F3 pause. F11 quick save. Space: last alert.");
                y += 40.0;
                if ui.button_styled(
                    Rect::new(40.0, y, 160.0, 40.0),
                    "Save",
                    true,
                    THEME.accent_dark,
                ) {
                    action = Some("save_options");
                }
                if ui.button(Rect::new(220.0, y, 120.0, 40.0), "Back") {
                    next = Some(Screen::Main);
                }
            }
            Screen::Credits => {
                ui.heading(40.0, 30.0, "Credits");
                let lines = [
                    "Wars is an original, open-source real-time strategy game by Enrique Magallón.",
                    "Inspired by the classic age-advancing strategy games of the late nineties.",
                    "Everything you see is drawn procedurally; there are no bitmap assets.",
                    "Fonts: DejaVu Sans (Bitstream Vera licence). Built with Rust, wgpu, winit, axum.",
                    "Source: github.com/Devace-93/wars · MIT licence.",
                ];
                for (i, l) in lines.iter().enumerate() {
                    ui.label(40.0, 90.0 + i as f32 * 26.0, l);
                }
                if ui.button(Rect::new(40.0, 260.0, 120.0, 40.0), "Back") {
                    next = Some(Screen::Main);
                }
            }
            Screen::GameOver => next = Some(Screen::Main),
            _ => {}
        }
    }
    if esc && app.focus.is_none() {
        match screen {
            Screen::Main => {}
            Screen::LoadGame if app.menu.return_to_game => action = Some("back_from_load"),
            _ => next = Some(Screen::Main),
        }
    }
    match action {
        Some("single") => next = Some(Screen::SinglePlayer),
        Some("multi") => next = Some(Screen::Multiplayer),
        Some("load") if app.menu.save_sel.is_some() => {
            let i = app.menu.save_sel.unwrap();
            let path = app.menu.saves[i].1.clone();
            match wars_client::saves::load_world(&path) {
                Ok(world) => {
                    let me = world
                        .players
                        .iter()
                        .find(|p| !p.id.is_gaia() && !p.is_ai)
                        .map(|p| p.id)
                        .unwrap_or(PlayerId(1));
                    let session = Session::Local(LocalMatch::new(world, me));
                    app.game = Some(GameState::new(session, app.screen_w, app.screen_h, data));
                    app.menu.return_to_game = false;
                    next = Some(Screen::Game);
                }
                Err(e) => app.toast(format!("Could not load: {e}")),
            }
        }
        Some("load") => next = Some(Screen::LoadGame),
        Some("delete_save") => {
            if let Some(i) = app.menu.save_sel.take() {
                let _ = std::fs::remove_file(&app.menu.saves[i].1);
                app.menu.saves.clear();
            }
        }
        Some("back_from_load") => {
            if app.menu.return_to_game && app.game.is_some() {
                app.menu.return_to_game = false;
                next = Some(Screen::Game);
            } else {
                next = Some(Screen::Main);
            }
        }
        Some("editor") => {
            app.menu.scenarios.clear();
            next = Some(Screen::Scenarios);
        }
        Some("new_scenario") => {
            let size = [96, 120, 144][app.menu.editor_size % 3];
            let sc = wars_mapgen::scenario::Scenario::blank(&app.menu.editor_name, size);
            crate::editor::open(app, sc);
            next = Some(Screen::Editor);
        }
        Some("edit_scenario") | Some("play_scenario") => {
            if let Some(i) = app.menu.scen_sel {
                let path = app.menu.scenarios[i].1.clone();
                match wars_client::saves::load_scenario(&path) {
                    Ok(sc) => {
                        if action == Some("edit_scenario") {
                            crate::editor::open(app, sc);
                            next = Some(Screen::Editor);
                        } else {
                            let world = sc.to_world(data, (app.time * 1000.0) as u64 | 1);
                            let me = world
                                .players
                                .iter()
                                .find(|p| !p.id.is_gaia() && !p.is_ai)
                                .map(|p| p.id)
                                .unwrap_or(PlayerId(1));
                            let session = Session::Local(LocalMatch::new(world, me));
                            app.game =
                                Some(GameState::new(session, app.screen_w, app.screen_h, data));
                            next = Some(Screen::Game);
                        }
                    }
                    Err(e) => app.menu.error = Some(format!("Could not load: {e}")),
                }
            }
        }
        Some("options") => next = Some(Screen::Options),
        Some("save_options") => {
            save_settings(app);
            app.toast("Options saved");
            next = Some(Screen::Main);
        }
        Some("credits") => next = Some(Screen::Credits),
        Some("quit") => app.quit = true,
        Some("start") => start_single_player(app),
        _ => {}
    }
    if let Some(s) = next {
        if s == Screen::LoadGame {
            app.menu.saves.clear();
            app.menu.save_sel = None;
        }
        app.screen = s;
        app.focus = None;
    }
}
