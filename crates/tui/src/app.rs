//! Application state machine: menus, single-player setup, load/scenario lists, options,
//! the game screen and the post-game score table.

use crate::game::{human_player, Game};
use crate::net_ui::{LobbiesScreen, LobbyScreen, LoginScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Row, Table, Wrap};
use std::path::PathBuf;
use wars_client::saves;
use wars_client::wars_mapgen::{generate, MapParams, MapSize, MapType, PlayerSetup};
use wars_client::wars_sim::ids::{Age, PlayerId};
use wars_client::wars_sim::world::{GameMode, Settings, World};
use wars_client::{LocalMatch, Session};

pub const MAIN_ITEMS: [&str; 6] = [
    "Single player",
    "Multiplayer",
    "Load game",
    "Scenarios",
    "Options",
    "Quit",
];

pub struct Setup {
    pub field: usize,
    pub map_type: usize,
    pub size: usize,
    pub civ: usize,
    pub opponents: u8,
    pub ai_level: u8,
    pub mode: usize,
    pub age: usize,
    pub reveal: bool,
    pub cheats: bool,
    pub seed: u64,
}

pub const SETUP_FIELDS: [&str; 10] = [
    "Map type",
    "Map size",
    "Civilisation",
    "AI opponents",
    "AI level",
    "Game mode",
    "Starting age",
    "Reveal map",
    "Allow cheats",
    "Seed",
];
const MODES: [&str; 3] = ["Random map", "Deathmatch", "Regicide"];
const AI_LEVELS: [&str; 3] = ["Easy", "Standard", "Hard"];

impl Setup {
    pub fn new() -> Setup {
        Setup {
            field: 0,
            map_type: 0,
            size: 0,
            civ: 0,
            opponents: 1,
            ai_level: 1,
            mode: 0,
            age: 0,
            reveal: false,
            cheats: false,
            seed: wars_client::session::now_ms() % 100_000,
        }
    }
    pub fn value(&self, i: usize) -> String {
        let data = wars_client::wars_data::data();
        match i {
            0 => MapType::ALL[self.map_type].name().to_string(),
            1 => MapSize::ALL[self.size].name().to_string(),
            2 => data.civs[self.civ].name.clone(),
            3 => self.opponents.to_string(),
            4 => AI_LEVELS[self.ai_level as usize].to_string(),
            5 => MODES[self.mode].to_string(),
            6 => Age::ALL[self.age].name().to_string(),
            7 => if self.reveal { "yes" } else { "no" }.to_string(),
            8 => if self.cheats { "yes" } else { "no" }.to_string(),
            _ => format!("{} (r: random)", self.seed),
        }
    }
    pub fn adjust(&mut self, dir: i32) {
        let data = wars_client::wars_data::data();
        fn cycle(v: usize, n: usize, dir: i32) -> usize {
            ((v as i32 + dir).rem_euclid(n as i32)) as usize
        }
        match self.field {
            0 => self.map_type = cycle(self.map_type, MapType::ALL.len(), dir),
            1 => self.size = cycle(self.size, MapSize::ALL.len(), dir),
            2 => self.civ = cycle(self.civ, data.civs.len(), dir),
            3 => self.opponents = (self.opponents as i32 + dir).clamp(1, 7) as u8,
            4 => self.ai_level = (self.ai_level as i32 + dir).clamp(0, 2) as u8,
            5 => self.mode = cycle(self.mode, MODES.len(), dir),
            6 => self.age = cycle(self.age, Age::ALL.len(), dir),
            7 => self.reveal = !self.reveal,
            8 => self.cheats = !self.cheats,
            _ => self.seed = (self.seed as i64 + dir as i64).max(0) as u64,
        }
    }
    pub fn params(&self) -> MapParams {
        let data = wars_client::wars_data::data();
        let mode = match self.mode {
            1 => GameMode::Deathmatch,
            2 => GameMode::Regicide,
            _ => GameMode::RandomMap,
        };
        let settings = Settings {
            mode,
            starting_age: Age::ALL[self.age],
            reveal_map: self.reveal,
            allow_cheats: self.cheats,
            ..Default::default()
        };
        let mut players = vec![PlayerSetup {
            name: "You".into(),
            civ: data.civs[self.civ].key.clone(),
            team: 1,
            is_ai: false,
            ai_level: 1,
        }];
        let n = data.civs.len() as u64;
        for i in 0..self.opponents {
            let civ = &data.civs[((self.seed + 7 * (i as u64 + 1)) % n) as usize];
            players.push(PlayerSetup {
                name: format!("Computer {}", i + 1),
                civ: civ.key.clone(),
                team: i + 2,
                is_ai: true,
                ai_level: self.ai_level,
            });
        }
        MapParams {
            map_type: MapType::ALL[self.map_type],
            size: MapSize::ALL[self.size],
            seed: self.seed,
            players,
            settings,
        }
    }
}

pub struct Options {
    pub field: usize,
    pub values: [String; 3],
}
const OPTION_FIELDS: [&str; 3] = ["Server URL", "Accounts URL", "Nickname"];

impl Options {
    pub fn load() -> Options {
        let s = saves::load_settings();
        let get = |k: &str, d: &str| s.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
        Options {
            field: 0,
            values: [
                get("server_url", "https://wars.3m4.net"),
                get("accounts_url", "https://accounts.3m4.net"),
                get("nickname", "nerd"),
            ],
        }
    }
    pub fn save(&self) {
        let mut s = saves::load_settings();
        s["server_url"] = serde_json::Value::String(self.values[0].clone());
        s["accounts_url"] = serde_json::Value::String(self.values[1].clone());
        s["nickname"] = serde_json::Value::String(self.values[2].clone());
        saves::save_settings(&s);
    }
}

pub struct ScoreRow {
    pub name: String,
    pub civ: String,
    pub team: u8,
    pub score: i32,
    pub kills: i32,
    pub losses: i32,
    pub razings: i32,
    pub result: String,
    pub color: PlayerId,
}

pub struct GameOver {
    pub won: Option<bool>,
    pub rows: Vec<ScoreRow>,
    pub minutes: u32,
}

impl GameOver {
    pub fn from_world(w: &World, me: PlayerId) -> GameOver {
        let data = wars_client::wars_data::data();
        let my_team = w.player(me).team;
        let rows = w
            .players
            .iter()
            .filter(|p| !p.id.is_gaia())
            .map(|p| ScoreRow {
                name: p.name.clone(),
                civ: data.civ(p.civ).name.clone(),
                team: p.team,
                score: p.score,
                kills: p.kills,
                losses: p.losses,
                razings: p.razings,
                result: if w.winner_team == Some(p.team) {
                    "winner".into()
                } else if p.resigned {
                    "resigned".into()
                } else if !p.alive {
                    "defeated".into()
                } else {
                    "alive".into()
                },
                color: p.id,
            })
            .collect();
        GameOver {
            won: w.winner_team.map(|t| t == my_team),
            rows,
            minutes: w.seconds() / 60,
        }
    }
}

pub enum Screen {
    Main(usize),
    Setup(Setup),
    /// Saved games or scenarios: (title, entries, cursor).
    List(&'static str, Vec<(String, PathBuf)>, usize),
    Options(Options),
    Login(LoginScreen),
    Lobbies(LobbiesScreen),
    Lobby(LobbyScreen),
    Game(Box<Game>),
    GameOver(GameOver),
}

pub struct App {
    pub screen: Screen,
    pub quit: bool,
    pub message: String,
    /// Size of the map panel from the last frame, so the game knows its viewport.
    pub map_panel: (u16, u16),
}

impl App {
    pub fn new() -> App {
        App {
            screen: Screen::Main(0),
            quit: false,
            message: String::new(),
            map_panel: (80, 24),
        }
    }

    pub fn in_game(&self) -> bool {
        matches!(self.screen, Screen::Game(_))
    }

    pub fn start_local(&mut self, world: World) {
        let me = human_player(&world);
        self.start_session(Session::Local(LocalMatch::new(world, me)));
    }

    pub fn start_session(&mut self, session: Session) {
        let mut g = Game::new(session);
        g.set_view_size(self.map_panel.0, self.map_panel.1);
        self.screen = Screen::Game(Box::new(g));
    }

    /// Per-frame work: advance the match, poll the lobby socket, detect game over.
    pub fn tick(&mut self) {
        match &mut self.screen {
            Screen::Game(g) => {
                g.tick();
                if g.leave {
                    self.screen = Screen::Main(0);
                    return;
                }
                if g.session.world().game_over {
                    let over = GameOver::from_world(g.session.world(), g.me());
                    self.screen = Screen::GameOver(over);
                }
            }
            Screen::Lobby(l) => {
                if let Some(session) = l.poll() {
                    self.start_session(session);
                } else if l.left {
                    self.screen = Screen::Lobbies(LobbiesScreen::new(l.conn.take_settings()));
                }
            }
            _ => {}
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        match &mut self.screen {
            Screen::Main(idx) => {
                let n = MAIN_ITEMS.len();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => *idx = (*idx + n - 1) % n,
                    KeyCode::Down | KeyCode::Char('j') => *idx = (*idx + 1) % n,
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let d = c.to_digit(10).unwrap() as usize;
                        if (1..=n).contains(&d) {
                            *idx = d - 1;
                            let i = *idx;
                            self.main_select(i);
                        }
                    }
                    KeyCode::Enter => {
                        let i = *idx;
                        self.main_select(i);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
                    _ => {}
                }
            }
            Screen::Setup(s) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    s.field = (s.field + SETUP_FIELDS.len() - 1) % SETUP_FIELDS.len()
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                    s.field = (s.field + 1) % SETUP_FIELDS.len()
                }
                KeyCode::Left | KeyCode::Char('h') => s.adjust(-1),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => s.adjust(1),
                KeyCode::Char('r') => s.seed = wars_client::session::now_ms() % 1_000_000,
                KeyCode::Char(c) if c.is_ascii_digit() && s.field == 9 => {
                    s.seed = (s.seed * 10 + c.to_digit(10).unwrap() as u64) % 10_000_000_000;
                }
                KeyCode::Backspace if s.field == 9 => s.seed /= 10,
                KeyCode::Enter => {
                    let params = s.params();
                    self.message = "generating map...".into();
                    let world = generate(wars_client::wars_data::data(), &params);
                    self.start_local(world);
                }
                KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Main(0),
                _ => {}
            },
            Screen::List(title, items, idx) => match key.code {
                KeyCode::Up | KeyCode::Char('k') if !items.is_empty() => {
                    *idx = (*idx + items.len() - 1) % items.len()
                }
                KeyCode::Down | KeyCode::Char('j') if !items.is_empty() => {
                    *idx = (*idx + 1) % items.len()
                }
                KeyCode::Enter if !items.is_empty() => {
                    let (name, path) = items[*idx].clone();
                    let scenario = *title == "Scenarios";
                    let loaded = if scenario {
                        saves::load_scenario(&path).map(|sc| {
                            sc.to_world(
                                wars_client::wars_data::data(),
                                wars_client::session::now_ms(),
                            )
                        })
                    } else {
                        saves::load_world(&path)
                    };
                    match loaded {
                        Ok(w) => {
                            self.message = format!("loaded {name}");
                            self.start_local(w);
                        }
                        Err(e) => self.message = format!("cannot load {name}: {e}"),
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Main(0),
                _ => {}
            },
            Screen::Options(o) => match key.code {
                KeyCode::Up | KeyCode::BackTab => {
                    o.field = (o.field + OPTION_FIELDS.len() - 1) % OPTION_FIELDS.len()
                }
                KeyCode::Down | KeyCode::Tab => o.field = (o.field + 1) % OPTION_FIELDS.len(),
                KeyCode::Backspace => {
                    o.values[o.field].pop();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    o.values[o.field].push(c)
                }
                KeyCode::Enter | KeyCode::Esc => {
                    o.save();
                    self.message = "settings saved".into();
                    self.screen = Screen::Main(4);
                }
                _ => {}
            },
            Screen::Login(l) => match l.key(key) {
                crate::net_ui::LoginOutcome::Stay => {}
                crate::net_ui::LoginOutcome::Back => self.screen = Screen::Main(1),
                crate::net_ui::LoginOutcome::LoggedIn(conn) => {
                    self.screen = Screen::Lobbies(LobbiesScreen::new(conn));
                }
            },
            Screen::Lobbies(l) => match l.key(key) {
                crate::net_ui::LobbiesOutcome::Stay => {}
                crate::net_ui::LobbiesOutcome::Back => self.screen = Screen::Main(1),
                crate::net_ui::LobbiesOutcome::Joined(lobby) => {
                    self.screen = Screen::Lobby(*lobby);
                }
            },
            Screen::Lobby(l) => l.key(key),
            Screen::Game(g) => g.handle_key(key),
            Screen::GameOver(_) => {
                if matches!(
                    key.code,
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(' ')
                ) {
                    self.screen = Screen::Main(0);
                }
            }
        }
    }

    fn main_select(&mut self, i: usize) {
        self.message.clear();
        match i {
            0 => self.screen = Screen::Setup(Setup::new()),
            1 => self.screen = Screen::Login(LoginScreen::new()),
            2 => self.screen = Screen::List("Load game", saves::list_saves(), 0),
            3 => self.screen = Screen::List("Scenarios", saves::list_scenarios(), 0),
            4 => self.screen = Screen::Options(Options::load()),
            _ => self.quit = true,
        }
    }

    // ---- drawing --------------------------------------------------------------------

    pub fn draw(&mut self, f: &mut Frame) {
        match &mut self.screen {
            Screen::Game(g) => {
                // Mirror the layout in view::draw_game to size the viewport.
                let area = f.area();
                let right_w = crate::view::right_column_width(area.width);
                let map_w = area.width.saturating_sub(right_w).max(1);
                let map_h = area
                    .height
                    .saturating_sub(crate::view::TOP_H + crate::view::BOTTOM_H)
                    .max(1);
                self.map_panel = (map_w, map_h);
                g.set_view_size(map_w, map_h);
                crate::view::draw_game(f, g);
            }
            Screen::Main(idx) => draw_menu(f, *idx, &self.message),
            Screen::Setup(s) => draw_setup(f, s),
            Screen::List(title, items, idx) => draw_list(f, title, items, *idx, &self.message),
            Screen::Options(o) => draw_options(f, o),
            Screen::Login(l) => l.draw(f),
            Screen::Lobbies(l) => l.draw(f),
            Screen::Lobby(l) => l.draw(f),
            Screen::GameOver(o) => draw_game_over(f, o),
        }
    }
}

pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

const LOGO: &str = "\
 __      __
/  \\    /  \\_____ _______  ______
\\   \\/\\/   /\\__  \\\\_  __ \\/  ___/
 \\        /  / __ \\|  | \\/\\___ \\
  \\__/\\  /  (____  /__|  /____  >
       \\/        \\/           \\/";

fn draw_menu(f: &mut Frame, idx: usize, message: &str) {
    let area = f.area();
    let r = centered(area, 44, 18);
    let mut lines: Vec<Line> = LOGO
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::new().fg(Color::Rgb(245, 158, 11)))))
        .collect();
    lines.push(Line::raw(""));
    for (i, item) in MAIN_ITEMS.iter().enumerate() {
        let style = if i == idx {
            Style::new()
                .fg(Color::Black)
                .bg(Color::Rgb(245, 158, 11))
                .bold()
        } else {
            Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", i + 1), Style::new().fg(Color::DarkGray)),
            Span::styled(format!(" {item:<20}"), style),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "arrows/jk + Enter, or a number",
        Style::new().fg(Color::DarkGray),
    )));
    if !message.is_empty() {
        lines.push(Line::from(Span::styled(
            message,
            Style::new().fg(Color::Yellow),
        )));
    }
    f.render_widget(Paragraph::new(lines), r);
}

fn draw_setup(f: &mut Frame, s: &Setup) {
    let area = f.area();
    let r = centered(area, 62, SETUP_FIELDS.len() as u16 + 5);
    let mut lines = Vec::new();
    for (i, name) in SETUP_FIELDS.iter().enumerate() {
        let style = if i == s.field {
            Style::new()
                .fg(Color::Black)
                .bg(Color::Rgb(245, 158, 11))
                .bold()
        } else {
            Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {name:<14}"), Style::new().fg(Color::Gray)),
            Span::styled(format!(" {:<28}", s.value(i)), style),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        " up/down field, left/right change, Enter start, Esc back",
        Style::new().fg(Color::DarkGray),
    )));
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Single player ")),
        r,
    );
}

fn draw_list(f: &mut Frame, title: &str, items: &[(String, PathBuf)], idx: usize, message: &str) {
    let area = f.area();
    let r = centered(area, 60, (items.len() as u16 + 6).clamp(8, area.height));
    let mut lines = Vec::new();
    if items.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" nothing in {}", saves::data_dir().display()),
            Style::new().fg(Color::DarkGray),
        )));
    }
    let max = r.height.saturating_sub(5) as usize;
    let start = idx.saturating_sub(max.saturating_sub(1));
    for (i, (name, _)) in items.iter().enumerate().skip(start).take(max.max(1)) {
        let style = if i == idx {
            Style::new()
                .fg(Color::Black)
                .bg(Color::Rgb(245, 158, 11))
                .bold()
        } else {
            Style::new()
        };
        lines.push(Line::from(Span::styled(format!(" {name:<56}"), style)));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        " Enter load, Esc back",
        Style::new().fg(Color::DarkGray),
    )));
    if !message.is_empty() {
        lines.push(Line::from(Span::styled(
            message,
            Style::new().fg(Color::Yellow),
        )));
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(format!(" {title} "))),
        r,
    );
}

fn draw_options(f: &mut Frame, o: &Options) {
    let area = f.area();
    let r = centered(area, 60, 9);
    let mut lines = Vec::new();
    for (i, name) in OPTION_FIELDS.iter().enumerate() {
        let style = if i == o.field {
            Style::new().fg(Color::Black).bg(Color::Rgb(245, 158, 11))
        } else {
            Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {name:<14}"), Style::new().fg(Color::Gray)),
            Span::styled(format!(" {:<40}", o.values[i]), style),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        " type to edit, Tab next field, Enter/Esc save",
        Style::new().fg(Color::DarkGray),
    )));
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Options ")),
        r,
    );
}

fn draw_game_over(f: &mut Frame, o: &GameOver) {
    let area = f.area();
    let r = centered(area, 78, o.rows.len() as u16 + 8);
    let [title_area, table_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(2),
        Constraint::Length(1),
    ])
    .areas(r);
    let (title, color) = match o.won {
        Some(true) => ("VICTORY", Color::Rgb(80, 230, 80)),
        Some(false) => ("DEFEAT", Color::Rgb(255, 80, 80)),
        None => ("GAME OVER", Color::Yellow),
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("{title}  ({} min)", o.minutes),
            Style::new().fg(color).bold(),
        )))
        .wrap(Wrap { trim: true })
        .centered(),
        title_area,
    );
    let rows: Vec<Row> = o
        .rows
        .iter()
        .map(|r| {
            Row::new(vec![
                r.name.clone(),
                r.civ.clone(),
                r.team.to_string(),
                r.score.to_string(),
                r.kills.to_string(),
                r.losses.to_string(),
                r.razings.to_string(),
                r.result.clone(),
            ])
            .style(Style::new().fg(crate::view::player_color(r.color)))
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec![
            "Player", "Civ", "Team", "Score", "Kills", "Losses", "Razings", "Result",
        ])
        .style(Style::new().bold().underlined()),
    )
    .block(Block::bordered().title(" score "));
    f.render_widget(table, table_area);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Enter: back to the menu",
            Style::new().fg(Color::DarkGray),
        )))
        .centered(),
        hint_area,
    );
}
