//! The game screen: cursor, viewport, modes and keyboard handling over a `Session`.

use crate::actions::{actions_for, build_menu, Action, ActionKind};
use crate::view::{tile_center, Viewport};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::time::Instant;
use wars_client::saves;
use wars_client::wars_sim::command::{Command, RallyTarget};
use wars_client::wars_sim::ids::{EntityId, PlayerId, Resource, Stance, TypeId};
use wars_client::wars_sim::types::{GameData, ObjectClass};
use wars_client::wars_sim::world::{Event, World};
use wars_client::{Controller, LocalMatch, Session};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    BuildMenu,
    Placing(TypeId),
    /// Anchor corner of a box selection.
    BoxSelect(i32, i32),
    Command(String),
    Help,
    ConfirmQuit,
    /// `g` was pressed: a digit stores the selection as that group.
    GroupPrefix,
}

const LOG_LEN: usize = 50;

pub struct Game {
    pub session: Session,
    pub ctrl: Controller,
    pub cursor: (i32, i32),
    pub vp: Viewport,
    pub mode: Mode,
    pub log: VecDeque<String>,
    pub status: String,
    pub actions: Vec<Action>,
    pub build_menu: Vec<Action>,
    /// Set when the player wants to go back to the menu.
    pub leave: bool,
    last_frame: Instant,
    tab_idx: usize,
    idle_idx: usize,
    chat_seen: usize,
    last_alert_tick: u32,
    last_pick: Option<EntityId>,
    data: &'static GameData,
}

/// The first human player of a world (1 if every player is an AI).
pub fn human_player(world: &World) -> PlayerId {
    world
        .players
        .iter()
        .find(|p| !p.id.is_gaia() && !p.is_ai)
        .map(|p| p.id)
        .unwrap_or(PlayerId(1))
}

impl Game {
    pub fn new(session: Session) -> Game {
        let data = wars_client::wars_data::data();
        let mut g = Game {
            session,
            ctrl: Controller::new(),
            cursor: (0, 0),
            vp: Viewport::default(),
            mode: Mode::Normal,
            log: VecDeque::new(),
            status: "? for help".into(),
            actions: Vec::new(),
            build_menu: Vec::new(),
            leave: false,
            last_frame: Instant::now(),
            tab_idx: 0,
            idle_idx: 0,
            chat_seen: 0,
            last_alert_tick: 0,
            last_pick: None,
            data,
        };
        g.vp.resize(80, 24, g.map_w(), g.map_h());
        g.jump_to_town_center();
        g
    }

    pub fn data(&self) -> &'static GameData {
        self.data
    }
    fn map_w(&self) -> i32 {
        self.session.world().map.width
    }
    fn map_h(&self) -> i32 {
        self.session.world().map.height
    }
    pub fn me(&self) -> PlayerId {
        self.session.me()
    }

    /// Called by the renderer when the map panel size is known.
    pub fn set_view_size(&mut self, w: u16, h: u16) {
        let (mw, mh) = (self.map_w(), self.map_h());
        if self.vp.w != w as i32 || self.vp.h != h as i32 {
            self.vp.resize(w as i32, h as i32, mw, mh);
            let (cx, cy) = self.cursor;
            self.vp.follow(cx, cy, mw, mh);
        }
    }

    pub fn cursor_target(&self) -> Option<EntityId> {
        Controller::pick(
            self.session.world(),
            self.data,
            tile_center(self.cursor.0, self.cursor.1),
            self.me(),
        )
    }

    fn push_log(&mut self, s: String) {
        self.log.push_back(s);
        while self.log.len() > LOG_LEN {
            self.log.pop_front();
        }
    }

    /// Advance the simulation by the wall-clock time since the last frame.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f64() * 1000.0;
        self.last_frame = now;
        let events = self.session.advance(self.data, dt);
        for ev in events {
            self.on_event(ev);
        }
        if let Session::Remote(m) = &self.session {
            let new: Vec<String> = m.chat[self.chat_seen.min(m.chat.len())..]
                .iter()
                .map(|(n, t)| format!("{n}: {t}"))
                .collect();
            self.chat_seen = m.chat.len();
            for s in new {
                self.push_log(s);
            }
        }
        let me = self.me();
        self.ctrl.prune(self.session.world(), me);
        self.actions = actions_for(self.session.world(), self.data, me, &self.ctrl.selection);
        if self.mode == Mode::BuildMenu {
            self.build_menu = build_menu(self.session.world(), self.data, me);
        }
    }

    fn on_event(&mut self, ev: Event) {
        let me = self.me();
        let data = self.data;
        let name = |w: &World, p: PlayerId| w.player(p).name.clone();
        let msg = match ev {
            Event::UnderAttack {
                player,
                at,
                building,
            } if player == me => {
                let tick = self.session.world().tick;
                if tick.saturating_sub(self.last_alert_tick) < 100 {
                    return;
                }
                self.last_alert_tick = tick;
                let (x, y) = at.tile();
                Some(format!(
                    "{} under attack at {x},{y}",
                    if building { "Building" } else { "Unit" }
                ))
            }
            Event::ResearchComplete { player, tech } if player == me => {
                Some(format!("Research complete: {}", data.tech(tech).name))
            }
            Event::AgeAdvanced { player, age } => Some(format!(
                "{} advanced to the {}",
                name(self.session.world(), player),
                age.name()
            )),
            Event::BuildingComplete { player, entity } if player == me => self
                .session
                .world()
                .get(entity)
                .map(|e| format!("{} complete", data.object(e.ty).name)),
            Event::PlayerDefeated { player } => Some(format!(
                "{} was defeated",
                name(self.session.world(), player)
            )),
            Event::PlayerResigned { player } => {
                Some(format!("{} resigned", name(self.session.world(), player)))
            }
            Event::Chat { player, text, .. } => {
                Some(format!("{}: {}", name(self.session.world(), player), text))
            }
            Event::NotEnough { player, resource } if player == me => {
                Some(format!("Not enough {}", resource.name()))
            }
            Event::PopCapReached { player } if player == me => {
                Some("Population limit: build more houses".into())
            }
            Event::Cheat { player, code } => Some(format!(
                "{} cheats: {}",
                name(self.session.world(), player),
                code
            )),
            Event::Victory { team } => Some(format!("Team {team} is victorious")),
            Event::Tribute { from, to, amount } if to == me => Some(format!(
                "Tribute from {}: {}",
                name(self.session.world(), from),
                crate::actions::cost_str(&amount)
            )),
            _ => None,
        };
        if let Some(m) = msg {
            self.status = m.clone();
            self.push_log(m);
        }
    }

    // ---- cursor helpers -------------------------------------------------------------

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let (mw, mh) = (self.map_w(), self.map_h());
        self.cursor.0 = (self.cursor.0 + dx).clamp(0, mw - 1);
        self.cursor.1 = (self.cursor.1 + dy).clamp(0, mh - 1);
        self.vp.follow(self.cursor.0, self.cursor.1, mw, mh);
    }

    fn jump_to(&mut self, x: i32, y: i32) {
        let (mw, mh) = (self.map_w(), self.map_h());
        self.cursor = (x.clamp(0, mw - 1), y.clamp(0, mh - 1));
        self.vp.center_on(self.cursor.0, self.cursor.1, mw, mh);
    }

    fn jump_to_entity(&mut self, id: EntityId) {
        if let Some(e) = self.session.world().get(id) {
            let (x, y) = e.pos.tile();
            self.jump_to(x, y);
        }
    }

    fn jump_to_town_center(&mut self) {
        let me = self.me();
        let data = self.data;
        let w = self.session.world();
        let tc = w
            .entities_of(me)
            .find(|e| data.object(e.ty).is_town_center)
            .or_else(|| w.entities_of(me).next())
            .map(|e| e.id);
        match tc {
            Some(id) => self.jump_to_entity(id),
            None => self.jump_to(self.map_w() / 2, self.map_h() / 2),
        }
    }

    fn own_ids(&self) -> Vec<EntityId> {
        let data = self.data;
        let w = self.session.world();
        w.entities_of(self.me())
            .filter(|e| e.inside.is_none())
            .filter(|e| {
                matches!(
                    data.object(e.ty).class,
                    ObjectClass::Unit | ObjectClass::Building
                )
            })
            .map(|e| e.id)
            .collect()
    }

    fn select_under_cursor(&mut self, add: bool) {
        let Some(id) = self.cursor_target() else {
            if !add {
                self.ctrl.select(Vec::new(), false);
            }
            self.last_pick = None;
            return;
        };
        let w = self.session.world();
        let Some(e) = w.get(id) else { return };
        if e.owner != self.me() {
            self.status = format!(
                "{} belongs to {}",
                self.data.object(e.ty).name,
                if e.owner.is_gaia() {
                    "nobody".to_string()
                } else {
                    w.player(e.owner).name.clone()
                }
            );
            self.last_pick = None;
            return;
        }
        // Second press on the same unit: every own unit of that type on screen.
        if !add && self.last_pick == Some(id) && self.ctrl.selection == vec![id] {
            let ty = e.ty;
            let vp = self.vp;
            let ids: Vec<EntityId> = w
                .entities_of(self.me())
                .filter(|o| o.ty == ty && o.inside.is_none())
                .filter(|o| {
                    let (x, y) = o.pos.tile();
                    vp.contains(x, y)
                })
                .map(|o| o.id)
                .collect();
            let n = ids.len();
            self.ctrl.select(ids, false);
            self.status = format!("{n} x {} selected", self.data.object(ty).name);
            return;
        }
        self.last_pick = Some(id);
        self.ctrl.select(vec![id], add);
    }

    fn selected_villagers(&self) -> Vec<EntityId> {
        let data = self.data;
        let w = self.session.world();
        self.ctrl
            .selection
            .iter()
            .copied()
            .filter(|id| {
                w.get(*id)
                    .map(|e| data.object(e.ty).is_villager)
                    .unwrap_or(false)
            })
            .collect()
    }

    fn first_building(&self) -> Option<EntityId> {
        self.ctrl
            .selected_buildings(self.session.world(), self.data)
            .first()
            .copied()
    }

    // ---- actions --------------------------------------------------------------------

    fn run_action(&mut self, kind: ActionKind, shift: bool) {
        let data = self.data;
        let me = self.me();
        let at = tile_center(self.cursor.0, self.cursor.1);
        let target = self.cursor_target();
        let units = self.ctrl.selected_units(self.session.world(), data);
        match kind {
            ActionKind::BuildMenu => {
                self.build_menu = build_menu(self.session.world(), data, me);
                self.mode = Mode::BuildMenu;
            }
            ActionKind::Place(ty) => {
                self.mode = Mode::Placing(ty);
                self.status = format!("placing {}: Enter confirms", data.object(ty).name);
            }
            ActionKind::Gather => {
                let villagers = self.selected_villagers();
                match target {
                    Some(t) if !villagers.is_empty() => {
                        let ok = self
                            .session
                            .world()
                            .get(t)
                            .map(|e| data.object(e.ty).yields.is_some())
                            .unwrap_or(false);
                        if ok {
                            self.session.issue(Command::Gather {
                                units: villagers,
                                target: t,
                                queue: shift,
                            });
                            self.status = "gathering".into();
                        } else {
                            self.status = "nothing to gather there".into();
                        }
                    }
                    _ => self.status = "put the cursor on a resource first".into(),
                }
            }
            ActionKind::Repair => {
                let villagers = self.selected_villagers();
                match target {
                    Some(t) if !villagers.is_empty() => {
                        self.session.issue(Command::Repair {
                            units: villagers,
                            target: t,
                            queue: shift,
                        });
                        self.status = "repairing".into();
                    }
                    _ => self.status = "put the cursor on a building first".into(),
                }
            }
            ActionKind::Move => {
                if !units.is_empty() {
                    self.session.issue(Command::Move {
                        units,
                        to: at,
                        queue: shift,
                    });
                    self.status = format!("moving to {},{}", self.cursor.0, self.cursor.1);
                }
            }
            ActionKind::AttackMove => {
                if !units.is_empty() {
                    self.session.issue(Command::AttackMove {
                        units,
                        to: at,
                        queue: shift,
                    });
                    self.status = format!("attack-moving to {},{}", self.cursor.0, self.cursor.1);
                }
            }
            ActionKind::Stop => {
                if !units.is_empty() {
                    self.session.issue(Command::Stop { units });
                    self.status = "stopped".into();
                }
            }
            ActionKind::Garrison => match target {
                Some(t) if !units.is_empty() => {
                    self.session.issue(Command::Garrison { units, target: t });
                    self.status = "garrisoning".into();
                }
                _ => self.status = "put the cursor on a building first".into(),
            },
            ActionKind::Delete => {
                let ids = self.ctrl.selection.clone();
                for id in &ids {
                    self.session.issue(Command::Delete { entity: *id });
                }
                self.status = format!("deleted {}", ids.len());
            }
            ActionKind::Context => {
                let w = self.session.world();
                let cmd = self.ctrl.context_command(w, data, me, at, target);
                let action = self.ctrl.context_action(w, data, me, target);
                match cmd {
                    Some(c) => {
                        self.session.issue(c);
                        self.status = format!("{action:?}").to_lowercase();
                    }
                    None => self.status = "nothing to do there".into(),
                }
            }
            ActionKind::TogglePack => {
                if !units.is_empty() {
                    self.session.issue(Command::TogglePack { units });
                }
            }
            ActionKind::Unload => {
                if !units.is_empty() {
                    self.session.issue(Command::Unload { units });
                }
            }
            ActionKind::Train(ty) => {
                if let Some(b) = self.first_building() {
                    let n = if shift { 5 } else { 1 };
                    for _ in 0..n {
                        self.session.issue(Command::Train {
                            building: b,
                            unit: ty,
                        });
                    }
                    self.status = format!("training {}", data.object(ty).name);
                }
            }
            ActionKind::Research(t) => {
                if let Some(b) = self.first_building() {
                    self.session.issue(Command::Research {
                        building: b,
                        tech: t,
                    });
                    self.status = format!("researching {}", data.tech(t).name);
                }
            }
            ActionKind::Ungarrison => {
                if let Some(b) = self.first_building() {
                    self.session.issue(Command::Ungarrison { building: b });
                }
            }
            ActionKind::Rally => {
                let buildings = self.ctrl.selected_buildings(self.session.world(), data);
                if !buildings.is_empty() {
                    let to = match target {
                        Some(t) => RallyTarget::Entity(t),
                        None => RallyTarget::Point(at),
                    };
                    self.session.issue(Command::SetRally { buildings, to });
                    self.status = "rally point set".into();
                }
            }
            ActionKind::CancelQueue => {
                if let Some(b) = self.first_building() {
                    let len = self
                        .session
                        .world()
                        .get(b)
                        .map(|e| e.queue.len())
                        .unwrap_or(0);
                    if len > 0 {
                        self.session.issue(Command::CancelQueue {
                            building: b,
                            index: len - 1,
                        });
                    }
                }
            }
        }
    }

    fn confirm_placement(&mut self, ty: TypeId) {
        let def = self.data.object(ty);
        let (fw, fh) = (def.size.0 as i32, def.size.1 as i32);
        let (x, y) = self.cursor;
        if !self
            .session
            .world()
            .map
            .can_place(x, y, fw, fh, def.placement)
        {
            self.status = "cannot build there".into();
            return;
        }
        let villagers = self.selected_villagers();
        if villagers.is_empty() {
            self.status = "select villagers first".into();
            self.mode = Mode::Normal;
            return;
        }
        self.session.issue(Command::Build {
            units: villagers,
            building: ty,
            x,
            y,
            queue: false,
        });
        self.status = format!("building {} at {x},{y}", def.name);
        self.mode = Mode::Normal;
    }

    fn quick_save(&mut self, name: &str) {
        match saves::save_world(name, self.session.world()) {
            Ok(p) => self.status = format!("saved {}", p.display()),
            Err(e) => self.status = format!("save failed: {e}"),
        }
    }

    fn run_command(&mut self, line: &str) {
        let line = line.trim();
        let (cmd, arg) = match line.split_once(' ') {
            Some((c, a)) => (c, a.trim()),
            None => (line, ""),
        };
        match cmd {
            "save" | "w" => {
                let name = if arg.is_empty() { "quicksave" } else { arg };
                self.quick_save(name);
            }
            "load" => {
                let list = saves::list_saves();
                let found = list
                    .iter()
                    .find(|(n, _)| n == arg)
                    .or_else(|| if arg.is_empty() { list.last() } else { None })
                    .cloned();
                match found {
                    Some((n, p)) => match saves::load_world(&p) {
                        Ok(w) => {
                            let me = human_player(&w);
                            self.session = Session::Local(LocalMatch::new(w, me));
                            self.ctrl = Controller::new();
                            self.jump_to_town_center();
                            self.status = format!("loaded {n}");
                        }
                        Err(e) => self.status = format!("load failed: {e}"),
                    },
                    None => self.status = format!("no save named {arg}"),
                }
            }
            "chat" | "say" => {
                if !arg.is_empty() {
                    self.session.chat(arg.to_string(), false);
                }
            }
            "team" => {
                if !arg.is_empty() {
                    self.session.chat(arg.to_string(), true);
                }
            }
            "cheat" => {
                self.session.issue(Command::Cheat {
                    code: arg.to_string(),
                });
                self.status = format!("cheat: {arg}");
            }
            "speed" => match arg.parse::<u32>() {
                Ok(s) => self.session.set_speed_pct(s),
                Err(_) => self.status = "usage: :speed <percent>".into(),
            },
            "buy" | "sell" => match parse_resource(arg) {
                Some(r) => {
                    if cmd == "buy" {
                        self.session.issue(Command::MarketBuy { resource: r });
                    } else {
                        self.session.issue(Command::MarketSell { resource: r });
                    }
                }
                None => self.status = "usage: :buy food|wood|stone".into(),
            },
            "stance" => {
                let units = self.ctrl.selected_units(self.session.world(), self.data);
                let stance = match arg {
                    "aggressive" | "a" => Some(Stance::Aggressive),
                    "defensive" | "d" => Some(Stance::Defensive),
                    "stand" | "standground" | "s" => Some(Stance::StandGround),
                    "noattack" | "n" => Some(Stance::NoAttack),
                    _ => None,
                };
                match stance {
                    Some(s) if !units.is_empty() => {
                        self.session.issue(Command::SetStance { units, stance: s })
                    }
                    _ => self.status = "usage: :stance aggressive|defensive|stand|noattack".into(),
                }
            }
            "goto" => {
                let mut it = arg.split([' ', ',']).filter(|s| !s.is_empty());
                match (
                    it.next().and_then(|s| s.parse::<i32>().ok()),
                    it.next().and_then(|s| s.parse::<i32>().ok()),
                ) {
                    (Some(x), Some(y)) => self.jump_to(x, y),
                    _ => self.status = "usage: :goto x y".into(),
                }
            }
            "resign" => self.session.issue(Command::Resign),
            "quit" | "q" => self.leave = true,
            "help" | "h" => self.mode = Mode::Help,
            "" => {}
            other => self.status = format!("unknown command: {other}"),
        }
    }

    // ---- keys -----------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        self.ctrl.shift = shift;
        match self.mode.clone() {
            Mode::Help => {
                self.mode = Mode::Normal;
            }
            Mode::ConfirmQuit => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => self.leave = true,
                    _ => {}
                }
                self.mode = Mode::Normal;
            }
            Mode::Command(mut s) => match key.code {
                KeyCode::Enter => {
                    self.mode = Mode::Normal;
                    self.run_command(&s);
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => {
                    s.pop();
                    self.mode = Mode::Command(s);
                }
                KeyCode::Char(c) => {
                    s.push(c);
                    self.mode = Mode::Command(s);
                }
                _ => {}
            },
            Mode::GroupPrefix => {
                self.mode = Mode::Normal;
                if let KeyCode::Char(c) = key.code {
                    if let Some(n) = c.to_digit(10) {
                        self.ctrl.set_group(n as usize);
                        self.status =
                            format!("group {n} set ({} units)", self.ctrl.selection.len());
                        return;
                    }
                }
                self.handle_key(key);
            }
            Mode::BuildMenu => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Char(c) => {
                    if let Some(a) = self.build_menu.iter().find(|a| a.key == c) {
                        if a.enabled {
                            let kind = a.kind.clone();
                            self.run_action(kind, shift);
                        } else {
                            self.status = format!(
                                "{} unavailable{}",
                                a.label,
                                if a.note.is_empty() {
                                    ": not enough resources".to_string()
                                } else {
                                    format!(": {}", a.note)
                                }
                            );
                        }
                    }
                }
                _ => {}
            },
            Mode::Placing(ty) => {
                if self.movement(key, shift) {
                    return;
                }
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.status = "placement cancelled".into();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => self.confirm_placement(ty),
                    _ => {}
                }
            }
            Mode::BoxSelect(ax, ay) => {
                if self.movement(key, shift) {
                    return;
                }
                match key.code {
                    KeyCode::Esc => self.mode = Mode::Normal,
                    KeyCode::Enter
                    | KeyCode::Char(' ')
                    | KeyCode::Char('v')
                    | KeyCode::Char('V') => {
                        let ids = Controller::box_select(
                            self.session.world(),
                            self.data,
                            self.me(),
                            tile_center(ax, ay),
                            tile_center(self.cursor.0, self.cursor.1),
                        );
                        self.status = format!("{} selected", ids.len());
                        self.ctrl.select(ids, shift);
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
            }
            Mode::Normal => self.normal_key(key, shift, ctrl),
        }
    }

    /// Cursor movement shared by every cursor mode. Returns true if handled.
    fn movement(&mut self, key: KeyEvent, shift: bool) -> bool {
        let step = if shift { 5 } else { 1 };
        let (dx, dy) = match key.code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => (-1, 0),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => (1, 0),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => (0, -1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => (0, 1),
            KeyCode::PageUp => (0, -self.vp.h / 2),
            KeyCode::PageDown => (0, self.vp.h / 2),
            KeyCode::Home => (-self.vp.w / 2, 0),
            KeyCode::End => (self.vp.w / 2, 0),
            _ => return false,
        };
        self.move_cursor(dx * step, dy * step);
        true
    }

    fn normal_key(&mut self, key: KeyEvent, shift: bool, ctrl: bool) {
        if self.movement(key, shift) {
            return;
        }
        // Control groups: Ctrl+digit stores, digit recalls.
        if let KeyCode::Char(c) = key.code {
            if let Some(n) = c.to_digit(10) {
                if ctrl {
                    self.ctrl.set_group(n as usize);
                    self.status = format!("group {n} set ({} units)", self.ctrl.selection.len());
                } else {
                    self.ctrl.recall_group(n as usize);
                    if let Some(id) = self.ctrl.selection.first().copied() {
                        self.jump_to_entity(id);
                    }
                    self.status = format!("group {n}: {} units", self.ctrl.selection.len());
                }
                return;
            }
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => self.select_under_cursor(shift),
            KeyCode::Esc => {
                self.ctrl.select(Vec::new(), false);
                self.status.clear();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let ids = self.own_ids();
                if ids.is_empty() {
                    return;
                }
                if key.code == KeyCode::BackTab {
                    self.tab_idx = (self.tab_idx + ids.len() - 1) % ids.len();
                } else {
                    self.tab_idx = (self.tab_idx + 1) % ids.len();
                }
                let id = ids[self.tab_idx];
                self.ctrl.select(vec![id], false);
                self.jump_to_entity(id);
            }
            KeyCode::F(5) => self.quick_save("quicksave"),
            KeyCode::Char(c) => self.char_key(c, shift),
            _ => {}
        }
    }

    fn char_key(&mut self, c: char, shift: bool) {
        let lc = c.to_ascii_lowercase();
        // Dynamic actions first (train/research letters, and the fixed verbs for units).
        if let Some(a) = self.actions.iter().find(|a| a.key == c || a.key == lc) {
            let kind = a.kind.clone();
            let enabled = a.enabled;
            let label = a.label.clone();
            if enabled {
                self.run_action(kind.clone(), shift);
            } else {
                self.status = format!("{label}: not enough resources");
            }
            if lc == 'g' && kind == ActionKind::Gather {
                // `g` then a digit still stores a control group.
                self.mode = Mode::GroupPrefix;
            }
            return;
        }
        match lc {
            '.' => {
                let idle = Controller::idle_villagers(self.session.world(), self.data, self.me());
                if idle.is_empty() {
                    self.status = "no idle villagers".into();
                } else {
                    self.idle_idx = (self.idle_idx + 1) % idle.len();
                    let id = idle[self.idle_idx];
                    self.ctrl.select(vec![id], false);
                    self.jump_to_entity(id);
                    self.status = format!("idle villager {}/{}", self.idle_idx + 1, idle.len());
                }
            }
            ',' => {
                self.jump_to_town_center();
                if let Some(id) = self.cursor_target() {
                    self.ctrl.select(vec![id], false);
                }
            }
            'v' => self.mode = Mode::BoxSelect(self.cursor.0, self.cursor.1),
            'g' => self.mode = Mode::GroupPrefix,
            'x' => self.run_action(ActionKind::Context, shift),
            'd' => self.run_action(ActionKind::Delete, shift),
            'm' => self.run_action(ActionKind::Move, shift),
            'a' => self.run_action(ActionKind::AttackMove, shift),
            's' => self.run_action(ActionKind::Stop, shift),
            'b' => {
                if self.selected_villagers().is_empty() {
                    self.status = "select villagers to build".into();
                } else {
                    self.run_action(ActionKind::BuildMenu, shift);
                }
            }
            'p' => {
                let p = !self.session.paused();
                self.session.set_paused(p);
                self.status = if p { "paused".into() } else { "resumed".into() };
            }
            '+' | '=' => {
                let s = self.session.speed_pct() + 25;
                self.session.set_speed_pct(s);
            }
            '-' | '_' => {
                let s = self.session.speed_pct().saturating_sub(25);
                self.session.set_speed_pct(s);
            }
            ':' => self.mode = Mode::Command(String::new()),
            '?' => self.mode = Mode::Help,
            'q' => self.mode = Mode::ConfirmQuit,
            _ => {}
        }
    }
}

fn parse_resource(s: &str) -> Option<Resource> {
    match s {
        "food" | "f" => Some(Resource::Food),
        "wood" | "w" => Some(Resource::Wood),
        "stone" | "s" => Some(Resource::Stone),
        "gold" | "g" => Some(Resource::Gold),
        _ => None,
    }
}
