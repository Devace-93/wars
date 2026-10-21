//! wars-app: the whole game front-end, independent of the window system. The desktop
//! shell (winit) and the mobile shells feed `Input`s and present the `DrawList`.

pub mod editor;
pub mod game;
pub mod hud;
pub mod menus;
#[cfg(feature = "net")]
pub mod net_ui;

use wars_render::draw::DrawList;
use wars_render::text::TextSystem;
use wars_sim::types::GameData;
use wars_ui::{Frame, Input, Key, Mods, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Main,
    SinglePlayer,
    LoadGame,
    Scenarios,
    Options,
    Multiplayer,
    Game,
    Editor,
    GameOver,
    Credits,
}

pub struct App {
    pub data: &'static GameData,
    pub screen: Screen,
    pub frame: Frame,
    pub focus: Option<u64>,
    pub dl: DrawList,
    pub text: TextSystem,
    pub settings: serde_json::Value,
    pub game: Option<game::GameState>,
    pub editor: Option<editor::EditorState>,
    /// The editor's live world while a test game runs.
    pub editor_game: Option<game::GameState>,
    pub menu: menus::MenuState,
    #[cfg(feature = "net")]
    pub net: net_ui::NetState,
    pub toasts: Vec<(String, f64)>,
    pub quit: bool,
    pub screen_w: f32,
    pub screen_h: f32,
    pub time: f64,
    pending_keys: Vec<(Key, Mods)>,
    pending_chars: Vec<char>,
    mouse: (f32, f32),
    down: bool,
    pressed: bool,
    released: bool,
    right_pressed: bool,
    right_released: bool,
    middle_down: bool,
    wheel: f32,
    mods: Mods,
    pub touch: bool,
    pub pending_keys_push: Option<Key>,
}

impl App {
    pub fn new(w: f32, h: f32) -> App {
        let settings = wars_client::saves::load_settings();
        #[cfg(feature = "net")]
        let net = net_ui::NetState::load(&settings);
        let menu = menus::MenuState::new(&settings);
        App {
            data: wars_data::data(),
            screen: Screen::Main,
            frame: Frame::default(),
            focus: None,
            dl: DrawList::new(),
            text: TextSystem::new(),
            menu,
            settings,
            game: None,
            editor: None,
            editor_game: None,
            #[cfg(feature = "net")]
            net,
            toasts: Vec::new(),
            quit: false,
            screen_w: w,
            screen_h: h,
            time: 0.0,
            pending_keys: Vec::new(),
            pending_chars: Vec::new(),
            mouse: (w / 2.0, h / 2.0),
            down: false,
            pressed: false,
            released: false,
            right_pressed: false,
            right_released: false,
            middle_down: false,
            wheel: 0.0,
            mods: Mods::default(),
            touch: false,
            pending_keys_push: None,
        }
    }

    pub fn input(&mut self, i: Input) {
        match i {
            Input::MouseMove { x, y } => self.mouse = (x, y),
            Input::MouseDown { button, x, y } => {
                self.mouse = (x, y);
                match button {
                    0 => {
                        self.down = true;
                        self.pressed = true;
                    }
                    1 => self.right_pressed = true,
                    _ => self.middle_down = true,
                }
            }
            Input::MouseUp { button, x, y } => {
                self.mouse = (x, y);
                match button {
                    0 => {
                        self.down = false;
                        self.released = true;
                    }
                    1 => self.right_released = true,
                    _ => self.middle_down = false,
                }
            }
            Input::Wheel { dy, .. } => self.wheel += dy,
            Input::Key { key, down, mods } => {
                self.mods = mods;
                if down {
                    self.pending_keys.push((key, mods));
                }
                if let Some(g) = &mut self.game {
                    g.key_state(key, down);
                }
            }
            Input::Char(c) => self.pending_chars.push(c),
            Input::Resize { w, h } => {
                self.screen_w = w;
                self.screen_h = h;
                if let Some(g) = &mut self.game {
                    g.resize(w, h);
                }
                if let Some(e) = &mut self.editor {
                    e.resize(w, h);
                }
            }
            Input::Pinch { scale, x, y } => {
                self.touch = true;
                if let Some(g) = &mut self.game {
                    g.iso.zoom_at(scale, x, y, 0.4, 2.5);
                }
                if let Some(e) = &mut self.editor {
                    e.iso.zoom_at(scale, x, y, 0.4, 2.5);
                }
            }
            Input::Pan { dx, dy } => {
                self.touch = true;
                if let Some(g) = &mut self.game {
                    g.iso.pan(-dx, -dy);
                }
                if let Some(e) = &mut self.editor {
                    e.iso.pan(-dx, -dy);
                }
            }
            Input::LongPress { x, y } => {
                self.touch = true;
                if let Some(g) = &mut self.game {
                    g.long_press = Some((x, y));
                }
            }
            Input::DoubleTap { x, y } => {
                self.touch = true;
                if let Some(g) = &mut self.game {
                    g.double_tap = Some((x, y));
                }
            }
            Input::Back => {
                self.pending_keys.push((Key::Escape, Mods::default()));
            }
        }
    }

    /// Latest cursor position (before the next frame snapshot).
    pub fn mouse(&self) -> (f32, f32) {
        self.mouse
    }

    pub fn toast(&mut self, s: impl Into<String>) {
        self.toasts.push((s.into(), self.time + 5.0));
    }

    /// Advance the game state and rebuild the draw list for this frame.
    pub fn frame(&mut self, dt: f32) {
        self.time += dt as f64;
        self.frame = Frame {
            mouse: self.mouse,
            down: self.down,
            pressed: self.pressed,
            released: self.released,
            right_pressed: self.right_pressed,
            right_released: self.right_released,
            middle_down: self.middle_down,
            wheel: self.wheel,
            keys: std::mem::take(&mut self.pending_keys),
            chars: std::mem::take(&mut self.pending_chars),
            mods: self.mods,
            screen: (self.screen_w, self.screen_h),
            dt,
            time: self.time,
        };
        self.pressed = false;
        self.released = false;
        self.right_pressed = false;
        self.right_released = false;
        self.wheel = 0.0;
        self.dl.clear();
        match self.screen {
            Screen::Game => game::frame(self),
            Screen::Editor => editor::frame(self),
            Screen::Multiplayer => {
                #[cfg(feature = "net")]
                net_ui::frame(self);
                #[cfg(not(feature = "net"))]
                {
                    self.screen = Screen::Main;
                }
            }
            _ => menus::frame(self),
        }
        let now = self.time;
        self.toasts.retain(|(_, until)| *until > now);
        let mut ui = Ui::new(&mut self.dl, &mut self.text, &self.frame, &mut self.focus);
        App::toasts_ui(&mut ui, &self.toasts, self.screen_w);
    }

    /// Screens call this at the end with their Ui to draw the toast stack.
    pub fn toasts_ui(ui: &mut Ui, toasts: &[(String, f64)], screen_w: f32) {
        let mut y = 60.0;
        for (msg, _) in toasts.iter().rev().take(6) {
            let w = ui.measure(15, msg) + 24.0;
            let x = (screen_w - w) / 2.0;
            ui.dl.rect(
                x,
                y,
                w,
                26.0,
                wars_render::draw::Color::rgba(20, 14, 8, 220),
            );
            ui.dl
                .rect_outline(x, y, w, 26.0, 1.0, wars_ui::THEME.accent_dark);
            ui.label(x + 12.0, y + 4.0, msg);
            y += 30.0;
        }
    }
}
