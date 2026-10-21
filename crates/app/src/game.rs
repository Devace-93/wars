//! In-game screen: camera, selection, commands, visual effects and the HUD glue.

use crate::{App, Screen};
use wars_client::{Controller, Session};
use wars_render::art::{self, EntityDraw, Vis};
use wars_render::draw::Color;
use wars_render::iso::Iso;
use wars_sim::command::Command;
use wars_sim::ids::*;
use wars_sim::types::{GameData, ObjectClass};
use wars_sim::world::{Event, World, TICKS_PER_SECOND};
use wars_sim::{Fixed, Vec2};
use wars_ui::{Key, Rect};

pub const TOP_BAR: f32 = 30.0;
pub const BOTTOM_PANEL: f32 = 170.0;

pub struct Shot {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub start: u32,
    pub end: u32,
}

pub struct GameState {
    pub session: Session,
    pub ctrl: Controller,
    pub iso: Iso,
    pub box_start: Option<(f32, f32)>,
    pub placing: Option<TypeId>,
    pub wall_start: Option<(i32, i32)>,
    pub build_menu: bool,
    pub chat_open: bool,
    pub chat_text: String,
    pub menu_open: bool,
    pub save_name: String,
    pub show_scores: bool,
    pub shots: Vec<Shot>,
    pub deaths: Vec<(Vec2, u32)>,
    pub messages: Vec<(String, f64, Color)>,
    pub last_alert: Option<Vec2>,
    pub held: std::collections::HashSet<Key>,
    pub last_click: (f64, (f32, f32)),
    pub long_press: Option<(f32, f32)>,
    pub double_tap: Option<(f32, f32)>,
    pub saved_msg: Option<String>,
    pub editor_test: bool,
    pub tooltip: Vec<String>,
    pub confirm_quit: bool,
    pub load_scroll: f32,
    pub load_sel: Option<usize>,
    pub speed_before_pause: u32,
    pub pending_point: Option<crate::hud::Pending>,
}

impl GameState {
    pub fn new(session: Session, w: f32, h: f32, data: &GameData) -> GameState {
        let mut iso = Iso::new(w, h);
        iso.top = TOP_BAR;
        iso.bottom = BOTTOM_PANEL;
        let me = session.me();
        let world = session.world();
        if let Some(tc) = world
            .entities_of(me)
            .find(|e| data.object(e.ty).is_town_center)
            .or_else(|| world.entities_of(me).next())
        {
            iso.cx = tc.pos.x.to_f32();
            iso.cy = tc.pos.y.to_f32();
        } else {
            iso.cx = world.map.width as f32 / 2.0;
            iso.cy = world.map.height as f32 / 2.0;
        }
        GameState {
            session,
            ctrl: Controller::new(),
            iso,
            box_start: None,
            placing: None,
            wall_start: None,
            build_menu: false,
            chat_open: false,
            chat_text: String::new(),
            menu_open: false,
            save_name: String::new(),
            show_scores: false,
            shots: Vec::new(),
            deaths: Vec::new(),
            messages: Vec::new(),
            last_alert: None,
            held: Default::default(),
            last_click: (0.0, (0.0, 0.0)),
            long_press: None,
            double_tap: None,
            saved_msg: None,
            editor_test: false,
            tooltip: Vec::new(),
            confirm_quit: false,
            load_scroll: 0.0,
            load_sel: None,
            speed_before_pause: 100,
            pending_point: None,
        }
    }
    pub fn resize(&mut self, w: f32, h: f32) {
        self.iso.screen_w = w;
        self.iso.screen_h = h;
    }
    pub fn key_state(&mut self, k: Key, down: bool) {
        if down {
            self.held.insert(k);
        } else {
            self.held.remove(&k);
        }
    }
    pub fn me(&self) -> PlayerId {
        self.session.me()
    }
    pub fn world(&self) -> &World {
        self.session.world()
    }
    pub fn message(&mut self, s: impl Into<String>, time: f64, c: Color) {
        self.messages.push((s.into(), time + 8.0, c));
        if self.messages.len() > 8 {
            self.messages.remove(0);
        }
    }
    pub fn hud_rect(&self) -> Rect {
        Rect::new(
            0.0,
            self.iso.screen_h - BOTTOM_PANEL,
            self.iso.screen_w,
            BOTTOM_PANEL,
        )
    }
    pub fn map_rect(&self) -> Rect {
        Rect::new(
            0.0,
            TOP_BAR,
            self.iso.screen_w,
            self.iso.screen_h - TOP_BAR - BOTTOM_PANEL,
        )
    }
}

fn vis_of(world: &World, viewer: PlayerId, x: i32, y: i32) -> Vis {
    if world.settings.reveal_map || world.is_visible(viewer, x, y) {
        Vis::Visible
    } else if world.is_explored(viewer, x, y) {
        Vis::Explored
    } else {
        Vis::Unexplored
    }
}

pub fn frame(app: &mut App) {
    let dt = app.frame.dt;
    let time = app.time;
    let Some(g) = app.game.as_mut() else {
        app.screen = Screen::Main;
        return;
    };
    let data = app.data;
    // 1. Advance the simulation.
    let events = if g.menu_open && !g.session.is_online() {
        Vec::new()
    } else {
        g.session.advance(data, dt as f64 * 1000.0)
    };
    for ev in &events {
        handle_event(g, ev, time, data);
    }
    g.ctrl.prune(g.session.world(), g.me());
    // 2. Camera from keys and edge scrolling.
    camera_input(g, app.frame.dt, app.frame.mouse, app.touch);
    // 3. Draw the world.
    draw_world(app);
    // 4. HUD (consumes clicks over panels), then map interaction with what is left.
    crate::hud::draw_and_handle(app);
    let g = app.game.as_mut().unwrap();
    if g.session.world().game_over && !g.show_scores {
        g.show_scores = true;
        let w = g.session.world();
        let me_team = w.player(g.me()).team;
        let my_team = if me_team == 0 {
            100 + g.me().0
        } else {
            me_team
        };
        let won = w.winner_team == Some(my_team);
        g.message(
            if won { "Victory!" } else { "Defeat." },
            time,
            if won {
                Color::rgb(80, 240, 80)
            } else {
                Color::rgb(240, 80, 80)
            },
        );
    }
}

fn handle_event(g: &mut GameState, ev: &Event, time: f64, data: &GameData) {
    let world = g.session.world();
    let me = g.me();
    match ev {
        Event::UnderAttack { player, at, .. } if *player == me => {
            if g.last_alert
                .map(|p| p.dist(*at) > Fixed::from_int(10))
                .unwrap_or(true)
            {
                g.message("You are under attack!", time, Color::rgb(240, 90, 60));
            }
            g.last_alert = Some(*at);
        }
        Event::ResearchComplete { player, tech } if *player == me => {
            let t = data.tech(*tech);
            if t.advances_to.is_none() {
                g.message(
                    format!("Research complete: {}", t.name),
                    time,
                    Color::rgb(220, 220, 240),
                );
            }
        }
        Event::AgeAdvanced { player, age } => {
            let name = world.player(*player).name.clone();
            g.message(
                format!("{name} has advanced to the {}", age.name()),
                time,
                Color::rgb(245, 200, 80),
            );
        }
        Event::Shot { from, to, ticks } => {
            g.shots.push(Shot {
                from: (from.x.to_f32(), from.y.to_f32()),
                to: (to.x.to_f32(), to.y.to_f32()),
                start: world.tick,
                end: world.tick + ticks,
            });
        }
        Event::Died { at, .. } => g.deaths.push((*at, world.tick)),
        Event::PlayerDefeated { player } => {
            let name = world.player(*player).name.clone();
            g.message(
                format!("{name} has been defeated"),
                time,
                Color::rgb(220, 160, 160),
            );
        }
        Event::PlayerResigned { player } => {
            let name = world.player(*player).name.clone();
            g.message(format!("{name} resigned"), time, Color::rgb(220, 160, 160));
        }
        Event::Chat { player, text, .. } => {
            let name = if player.is_gaia() {
                "Scenario".to_string()
            } else {
                world.player(*player).name.clone()
            };
            g.message(
                format!("{name}: {text}"),
                time,
                art::player_color(*player).mix(Color::WHITE, 0.4),
            );
        }
        Event::NotEnough { player, resource } if *player == me => {
            g.message(
                format!("Not enough {}", resource.name()),
                time,
                Color::rgb(240, 120, 90),
            );
        }
        Event::PopCapReached { player } if *player == me => {
            g.message("Build more houses", time, Color::rgb(240, 160, 90));
        }
        Event::WonderStarted { player } => {
            let name = world.player(*player).name.clone();
            g.message(
                format!("{name} has built a Wonder! Destroy it within 10 minutes."),
                time,
                Color::rgb(245, 200, 80),
            );
        }
        Event::RelicsStarted { player } => {
            let name = world.player(*player).name.clone();
            g.message(
                format!("{name} holds all the relics!"),
                time,
                Color::rgb(245, 200, 80),
            );
        }
        Event::Cheat { player, code } => {
            let name = world.player(*player).name.clone();
            g.message(
                format!("{name} used a cheat: {code}"),
                time,
                Color::rgb(200, 200, 200),
            );
        }
        Event::Tribute { from, to, amount } if *to == me => {
            let name = world.player(*from).name.clone();
            g.message(
                format!("{name} sent you tribute ({} resources)", amount.total()),
                time,
                Color::rgb(200, 240, 200),
            );
        }
        _ => {}
    }
}

fn camera_input(g: &mut GameState, dt: f32, mouse: (f32, f32), touch: bool) {
    let speed = 900.0 * dt;
    let mut dx = 0.0;
    let mut dy = 0.0;
    if g.held.contains(&Key::Left) {
        dx -= speed;
    }
    if g.held.contains(&Key::Right) {
        dx += speed;
    }
    if g.held.contains(&Key::Up) {
        dy -= speed;
    }
    if g.held.contains(&Key::Down) {
        dy += speed;
    }
    if !touch && !g.menu_open && !g.chat_open {
        let edge = 6.0;
        if mouse.0 <= edge && mouse.0 >= 0.0 {
            dx -= speed;
        }
        if mouse.0 >= g.iso.screen_w - edge {
            dx += speed;
        }
        if mouse.1 <= edge && mouse.1 >= 0.0 {
            dy -= speed;
        }
        if mouse.1 >= g.iso.screen_h - edge {
            dy += speed;
        }
    }
    if dx != 0.0 || dy != 0.0 {
        g.iso.pan(dx, dy);
    }
    let w = g.session.world();
    g.iso.clamp_to(w.map.width, w.map.height);
}

pub fn draw_world_public(app: &mut App) {
    draw_world(app);
}

fn draw_world(app: &mut App) {
    let g = app.game.as_mut().unwrap();
    let data = app.data;
    let world = g.session.world();
    let me = g.me();
    let dl = &mut app.dl;
    let map_rect = g.map_rect();
    dl.set_clip(Some(map_rect.clip()));
    dl.rect(
        map_rect.x,
        map_rect.y,
        map_rect.w,
        map_rect.h,
        Color::rgb(8, 6, 4),
    );
    // Terrain.
    let (x0, y0, x1, y1) = g.iso.visible_tiles();
    let (x0, y0) = (x0.max(0), y0.max(0));
    let (x1, y1) = (x1.min(world.map.width - 1), y1.min(world.map.height - 1));
    // Painter's order: by x + y diagonal.
    for s in (x0 + y0)..=(x1 + y1) {
        for x in x0..=x1 {
            let y = s - x;
            if y < y0 || y > y1 {
                continue;
            }
            let tile = &world.map.tiles[world.map.idx(x, y)];
            let vis = vis_of(world, me, x, y);
            art::draw_tile(dl, &g.iso, x, y, tile, vis);
        }
    }
    // Building placement preview.
    if let Some(ty) = g.placing {
        let (mx, my) = app.frame.mouse;
        if map_rect.contains((mx, my)) {
            let (wx, wy) = g.iso.to_world(mx, my);
            let def = data.object(ty);
            let (w, h) = (def.size.0 as i32, def.size.1 as i32);
            let tx = (wx - w as f32 / 2.0).round() as i32;
            let ty_ = (wy - h as f32 / 2.0).round() as i32;
            let ok = world.map.can_place(tx, ty_, w, h, def.placement)
                && wars_sim::stats::is_available(world, data, me, ty);
            let c = if ok {
                Color::rgba(80, 240, 80, 110)
            } else {
                Color::rgba(240, 60, 60, 110)
            };
            for yy in ty_..ty_ + h {
                for xx in tx..tx + w {
                    dl.poly(&g.iso.tile_poly(xx, yy), c);
                }
            }
            if let Some((sx, sy)) = g.wall_start {
                for (xx, yy) in line_tiles(sx, sy, tx, ty_) {
                    dl.poly(&g.iso.tile_poly(xx, yy), c);
                }
            }
        }
    }
    // Entities sorted by depth.
    let mut ents: Vec<(&wars_sim::world::Entity, Vis)> = Vec::new();
    for e in world.iter() {
        if e.inside.is_some() {
            continue;
        }
        let (x, y) = e.pos.tile();
        if x < x0 - 6 || x > x1 + 6 || y < y0 - 6 || y > y1 + 6 {
            continue;
        }
        let vis = vis_of(world, me, x, y);
        if vis == Vis::Unexplored {
            continue;
        }
        let def = data.object(e.ty);
        if vis == Vis::Explored
            && matches!(def.class, ObjectClass::Unit | ObjectClass::Animal)
            && !world.ally(me, e.owner)
        {
            continue;
        }
        ents.push((e, vis));
    }
    ents.sort_by(|(a, _), (b, _)| {
        let ka = depth_key(world, data, a);
        let kb = depth_key(world, data, b);
        ka.partial_cmp(&kb)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });
    let tick = world.tick;
    for (e, vis) in ents {
        let ctx = EntityDraw {
            world,
            data,
            tick,
            selected: g.ctrl.selection.contains(&e.id),
            vis,
            viewer: me,
        };
        art::draw_entity(dl, &g.iso, e, &ctx);
    }
    // Rally points for selected buildings.
    for id in &g.ctrl.selection {
        if let Some(e) = world.get(*id) {
            if let Some(wars_sim::command::RallyTarget::Point(p)) = e.rally {
                let a = g.iso.to_screen(e.pos.x.to_f32(), e.pos.y.to_f32());
                let b = g.iso.to_screen(p.x.to_f32(), p.y.to_f32());
                dl.line([a.0, a.1], [b.0, b.1], 1.5, Color::rgba(255, 255, 255, 120));
                dl.circle(b.0, b.1, 4.0, Color::rgb(255, 230, 120));
            }
        }
    }
    // Projectiles and deaths.
    g.shots.retain(|s| s.end + 2 >= tick);
    for s in &g.shots {
        let t = ((tick.saturating_sub(s.start)) as f32 / (s.end - s.start).max(1) as f32)
            .clamp(0.0, 1.0);
        let wx = s.from.0 + (s.to.0 - s.from.0) * t;
        let wy = s.from.1 + (s.to.1 - s.from.1) * t;
        let arc = (t * std::f32::consts::PI).sin() * 18.0 * g.iso.zoom;
        let (sx, sy) = g.iso.to_screen(wx, wy);
        dl.circle(
            sx,
            sy - 10.0 * g.iso.zoom - arc,
            2.0 * g.iso.zoom,
            Color::rgb(240, 230, 200),
        );
    }
    g.deaths.retain(|(_, t)| tick.saturating_sub(*t) < 20);
    for (p, t) in &g.deaths {
        let age = tick.saturating_sub(*t) as f32 / 20.0;
        let (sx, sy) = g.iso.to_screen(p.x.to_f32(), p.y.to_f32());
        dl.ellipse(
            sx,
            sy,
            (10.0 + age * 6.0) * g.iso.zoom,
            (5.0 + age * 3.0) * g.iso.zoom,
            Color::rgba(120, 20, 20, ((1.0 - age) * 140.0) as u8),
        );
    }
    // Box selection rectangle.
    if let Some((bx, by)) = g.box_start {
        let (mx, my) = app.frame.mouse;
        let x = bx.min(mx);
        let y = by.min(my);
        dl.rect_outline(
            x,
            y,
            (mx - bx).abs(),
            (my - by).abs(),
            1.5,
            Color::rgb(220, 255, 220),
        );
    }
    dl.set_clip(None);
}

fn depth_key(world: &World, data: &GameData, e: &wars_sim::world::Entity) -> f32 {
    let def = data.object(e.ty);
    if def.class == ObjectClass::Building || def.class == ObjectClass::Resource {
        let (x, y, w, h) = world.footprint(data, e);
        let base = (x + w) as f32 + (y + h) as f32;
        if def.walkable {
            base - 1.5
        } else {
            base - 0.001
        }
    } else {
        e.pos.x.to_f32() + e.pos.y.to_f32() + 0.5
    }
}

/// Tiles along a straight line (Bresenham) for wall placement.
pub fn line_tiles(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    let (mut x, mut y) = (x0, y0);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        v.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        if v.len() > 400 {
            break;
        }
    }
    v
}

/// What the map wants to do after reading the world (two-phase to satisfy the borrow checker).
enum Act {
    Issue(Command),
    Select(Vec<EntityId>, bool),
    SelectType(EntityId),
    ClearSel,
    Msg(String),
    PlaceDone,
    WallStart(i32, i32),
}

/// Map-area mouse handling (called by the HUD after panels consumed their clicks).
pub fn map_input(app: &mut App, ui_consumed: bool) {
    let time = app.time;
    let data = app.data;
    let g = app.game.as_mut().unwrap();
    let f = app.frame.clone();
    let map_rect = g.map_rect();
    let on_map = map_rect.contains(f.mouse) && !ui_consumed;
    let shift = f.mods.shift;
    g.ctrl.shift = shift;
    if f.wheel != 0.0 && map_rect.contains(f.mouse) {
        let factor = if f.wheel > 0.0 { 1.15 } else { 1.0 / 1.15 };
        g.iso.zoom_at(factor, f.mouse.0, f.mouse.1, 0.4, 2.5);
    }
    let (wx, wy) = g.iso.to_world(f.mouse.0, f.mouse.1);
    let at = Vec2::new(
        Fixed::from_decimal(&format!("{:.3}", wx.max(0.0))),
        Fixed::from_decimal(&format!("{:.3}", wy.max(0.0))),
    );
    let double_tap = g.double_tap.take();
    let long_press = g.long_press.take();
    let mut acts: Vec<Act> = Vec::new();
    // Phase 1: read the world and decide.
    {
        let world = g.session.world();
        let me = g.me();
        let iso = g.iso;
        let ctrl = &g.ctrl;
        if let Some((x, y)) = double_tap {
            let (wx, wy) = iso.to_world(x, y);
            let p = Vec2::new(
                Fixed::from_decimal(&format!("{:.3}", wx)),
                Fixed::from_decimal(&format!("{:.3}", wy)),
            );
            if let Some(id) = Controller::pick(world, data, p, me) {
                acts.push(Act::SelectType(id));
            }
        }
        if let Some((x, y)) = long_press {
            let (wx, wy) = iso.to_world(x, y);
            let p = Vec2::new(
                Fixed::from_decimal(&format!("{:.3}", wx)),
                Fixed::from_decimal(&format!("{:.3}", wy)),
            );
            let target = Controller::pick(world, data, p, me);
            if let Some(cmd) = ctrl.context_command(world, data, me, p, target) {
                acts.push(Act::Issue(cmd));
                acts.push(Act::PlaceDone);
            }
        }
        if let Some(ty) = g.placing {
            if f.right_released || f.key(Key::Escape) {
                acts.push(Act::PlaceDone);
            } else if on_map && f.released {
                let def = data.object(ty);
                let (w, h) = (def.size.0 as i32, def.size.1 as i32);
                let tx = (wx - w as f32 / 2.0).round() as i32;
                let ty_ = (wy - h as f32 / 2.0).round() as i32;
                let builders: Vec<EntityId> = ctrl
                    .selected_units(world, data)
                    .into_iter()
                    .filter(|u| data.object(world.get(*u).unwrap().ty).is_villager)
                    .collect();
                let is_wall = def.role == wars_sim::types::Role::Wall;
                if is_wall {
                    match g.wall_start {
                        None => acts.push(Act::WallStart(tx, ty_)),
                        Some((sx, sy)) => {
                            for (xx, yy) in line_tiles(sx, sy, tx, ty_) {
                                if world.map.can_place(xx, yy, 1, 1, def.placement) {
                                    acts.push(Act::Issue(Command::Build {
                                        units: builders.clone(),
                                        building: ty,
                                        x: xx,
                                        y: yy,
                                        queue: true,
                                    }));
                                }
                            }
                            acts.push(Act::WallStart(i32::MIN, 0));
                            if !shift {
                                acts.push(Act::PlaceDone);
                            }
                        }
                    }
                } else if world.map.can_place(tx, ty_, w, h, def.placement) {
                    acts.push(Act::Issue(Command::Build {
                        units: builders,
                        building: ty,
                        x: tx,
                        y: ty_,
                        queue: shift,
                    }));
                    if !shift {
                        acts.push(Act::PlaceDone);
                    }
                } else {
                    acts.push(Act::Msg("Cannot build there".into()));
                }
            }
        } else {
            if on_map && f.pressed && !g.chat_open {
                g.box_start = Some(f.mouse);
            }
            if f.released {
                if let Some(start) = g.box_start.take() {
                    let moved =
                        (start.0 - f.mouse.0).abs() > 4.0 || (start.1 - f.mouse.1).abs() > 4.0;
                    if moved {
                        let corners = [
                            iso.to_world(start.0, start.1),
                            iso.to_world(f.mouse.0, f.mouse.1),
                            iso.to_world(start.0, f.mouse.1),
                            iso.to_world(f.mouse.0, start.1),
                        ];
                        let minx = corners.iter().map(|c| c.0).fold(f32::MAX, f32::min);
                        let maxx = corners.iter().map(|c| c.0).fold(f32::MIN, f32::max);
                        let miny = corners.iter().map(|c| c.1).fold(f32::MAX, f32::min);
                        let maxy = corners.iter().map(|c| c.1).fold(f32::MIN, f32::max);
                        let a = Vec2::new(
                            Fixed::from_decimal(&format!("{:.2}", minx)),
                            Fixed::from_decimal(&format!("{:.2}", miny)),
                        );
                        let b = Vec2::new(
                            Fixed::from_decimal(&format!("{:.2}", maxx)),
                            Fixed::from_decimal(&format!("{:.2}", maxy)),
                        );
                        let ids: Vec<EntityId> = Controller::box_select(world, data, me, a, b)
                            .into_iter()
                            .filter(|id| {
                                let e = world.get(*id).unwrap();
                                let (sx, sy) = iso.to_screen(e.pos.x.to_f32(), e.pos.y.to_f32());
                                sx >= start.0.min(f.mouse.0)
                                    && sx <= start.0.max(f.mouse.0)
                                    && sy >= start.1.min(f.mouse.1)
                                    && sy <= start.1.max(f.mouse.1)
                            })
                            .collect();
                        if !ids.is_empty() {
                            acts.push(Act::Select(ids, shift));
                        }
                    } else if on_map {
                        let picked = Controller::pick(world, data, at, me);
                        let dbl = time - g.last_click.0 < 0.35
                            && (g.last_click.1 .0 - f.mouse.0).abs() < 6.0
                            && (g.last_click.1 .1 - f.mouse.1).abs() < 6.0;
                        g.last_click = (time, f.mouse);
                        match picked {
                            Some(id) => {
                                let e = world.get(id).unwrap();
                                if e.owner == me && dbl {
                                    acts.push(Act::SelectType(id));
                                } else if e.owner == me {
                                    acts.push(Act::Select(vec![id], shift));
                                } else {
                                    acts.push(Act::Select(vec![id], false));
                                }
                            }
                            None => {
                                if !shift {
                                    acts.push(Act::ClearSel);
                                }
                            }
                        }
                    }
                }
            }
            if on_map && f.right_released {
                let target = Controller::pick(world, data, at, me);
                if let Some(cmd) = ctrl.context_command(world, data, me, at, target) {
                    acts.push(Act::Issue(cmd));
                }
            }
        }
    }
    // Phase 2: apply.
    for a in acts {
        match a {
            Act::Issue(c) => g.session.issue(c),
            Act::Select(ids, add) => {
                g.ctrl.select(ids, add);
                g.build_menu = false;
            }
            Act::SelectType(id) => {
                select_all_of_type(g, id, data);
                g.build_menu = false;
            }
            Act::ClearSel => {
                g.ctrl.selection.clear();
                g.build_menu = false;
            }
            Act::Msg(m) => g.message(m, time, Color::rgb(240, 120, 90)),
            Act::PlaceDone => {
                g.placing = None;
                g.wall_start = None;
            }
            Act::WallStart(x, y) => g.wall_start = if x == i32::MIN { None } else { Some((x, y)) },
        }
    }
}

pub fn select_all_of_type(g: &mut GameState, id: EntityId, data: &GameData) {
    let world = g.session.world();
    let me = g.me();
    let Some(e) = world.get(id) else { return };
    let ty = e.ty;
    let (x0, y0, x1, y1) = g.iso.visible_tiles();
    let ids: Vec<EntityId> = world
        .entities_of(me)
        .filter(|o| o.ty == ty && o.inside.is_none())
        .filter(|o| {
            let (x, y) = o.pos.tile();
            x >= x0 && x <= x1 && y >= y0 && y <= y1
        })
        .map(|o| o.id)
        .collect();
    let _ = data;
    g.ctrl.select(ids, false);
}

pub fn game_time(world: &World) -> String {
    let s = world.tick / TICKS_PER_SECOND;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// Keyboard shortcuts in the game screen that are not grid commands.
pub fn hotkeys(app: &mut App) {
    let time = app.time;
    let data = app.data;
    let g = app.game.as_mut().unwrap();
    let keys = app.frame.keys.clone();
    for (k, m) in keys {
        if g.chat_open {
            match k {
                Key::Enter => {
                    let t = std::mem::take(&mut g.chat_text);
                    if !t.trim().is_empty() {
                        if t.starts_with('/')
                            || t.chars().any(|c| c == ' ')
                                && g.session.world().settings.allow_cheats
                                && is_cheat(&t)
                        {
                            g.session.issue(Command::Cheat {
                                code: t.trim_start_matches('/').to_string(),
                            });
                        } else {
                            g.session.chat(t, m.shift);
                        }
                    }
                    g.chat_open = false;
                    app.focus = None;
                }
                Key::Escape => {
                    g.chat_open = false;
                    app.focus = None;
                }
                _ => {}
            }
            continue;
        }
        if g.menu_open {
            if k == Key::Escape {
                g.menu_open = false;
                g.confirm_quit = false;
            }
            continue;
        }
        let world = g.session.world();
        let me = g.me();
        match k {
            Key::Escape => {
                if g.placing.is_some() {
                    g.placing = None;
                    g.wall_start = None;
                } else if g.build_menu {
                    g.build_menu = false;
                } else if !g.ctrl.selection.is_empty() {
                    g.ctrl.selection.clear();
                } else {
                    g.menu_open = true;
                }
            }
            Key::F(10) => g.menu_open = true,
            Key::F(3) => {
                let p = !g.session.paused();
                g.session.set_paused(p);
            }
            Key::F(11) => {
                let name = format!("Quick save {}", game_time(world));
                match wars_client::saves::save_world(&name, world) {
                    Ok(_) => g.message("Game saved", time, Color::rgb(200, 240, 200)),
                    Err(e) => {
                        g.message(format!("Save failed: {e}"), time, Color::rgb(240, 120, 90))
                    }
                }
            }
            Key::Enter => {
                g.chat_open = true;
                app.focus = Some(wars_ui::id("chat"));
            }
            Key::Space => {
                if let Some(p) = g.last_alert {
                    g.iso.cx = p.x.to_f32();
                    g.iso.cy = p.y.to_f32();
                }
            }
            Key::Char('=') | Key::Char('+') => {
                let s = g.session.speed_pct();
                g.session.set_speed_pct((s + 25).min(300));
            }
            Key::Char('-') => {
                let s = g.session.speed_pct();
                g.session.set_speed_pct(s.saturating_sub(25).max(25));
            }
            Key::Char('h') if !m.ctrl => {
                // Town centre: select and centre.
                if let Some(tc) = world
                    .entities_of(me)
                    .find(|e| data.object(e.ty).is_town_center)
                {
                    g.iso.cx = tc.pos.x.to_f32();
                    g.iso.cy = tc.pos.y.to_f32();
                    g.ctrl.selection = vec![tc.id];
                    g.build_menu = false;
                }
            }
            Key::Char('.') => {
                let idle = Controller::idle_villagers(world, data, me);
                if !idle.is_empty() {
                    let cur = g
                        .ctrl
                        .selection
                        .first()
                        .and_then(|s| idle.iter().position(|i| i == s));
                    let next = idle[cur.map(|c| (c + 1) % idle.len()).unwrap_or(0)];
                    let e = world.get(next).unwrap();
                    g.iso.cx = e.pos.x.to_f32();
                    g.iso.cy = e.pos.y.to_f32();
                    g.ctrl.selection = vec![next];
                    g.build_menu = false;
                }
            }
            Key::Char(c) if c.is_ascii_digit() => {
                let n = c.to_digit(10).unwrap() as usize;
                if m.ctrl {
                    g.ctrl.set_group(n);
                    g.message(format!("Group {n} set"), time, Color::rgb(200, 200, 200));
                } else {
                    g.ctrl.recall_group(n);
                    g.build_menu = false;
                }
            }
            Key::Delete => {
                if let Some(id) = g.ctrl.selection.first().copied() {
                    g.session.issue(Command::Delete { entity: id });
                }
            }
            _ => {}
        }
    }
}

fn is_cheat(t: &str) -> bool {
    matches!(
        t.trim().to_lowercase().as_str(),
        "cheese steak jimmy's"
            | "lumberjack"
            | "robin hood"
            | "rock on"
            | "marco"
            | "polo"
            | "aegis"
            | "i r winner"
            | "torpedo"
            | "how do you turn this on"
    )
}
