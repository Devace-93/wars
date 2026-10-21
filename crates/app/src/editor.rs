//! Scenario editor: paint terrain, place objects for any player, set players and
//! starting conditions, add triggers from templates, save, and test-play.

use crate::game::GameState;
use crate::{App, Screen};
use wars_client::{LocalMatch, Session};
use wars_mapgen::scenario::Scenario;
use wars_render::draw::Color;
use wars_sim::ids::*;
use wars_sim::trigger::{Area, Condition, Trigger, TriggerEffect};
use wars_sim::types::ObjectClass;
use wars_sim::world::Settings;
use wars_sim::Vec2;
use wars_ui::{Key, Rect, Ui, THEME};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Terrain(Terrain),
    Object(usize),
    Erase,
}

pub struct EditorState {
    pub scenario: Scenario,
    pub tool: Tool,
    pub brush: i32,
    pub owner: u8,
    pub objects: Vec<TypeId>,
    pub obj_scroll: f32,
    pub obj_sel: Option<usize>,
    pub tab: usize,
    pub name: String,
    pub description: String,
    pub trig_scroll: f32,
    pub trig_sel: Option<usize>,
    pub timer_secs: String,
    pub trig_text: String,
    pub iso: wars_render::iso::Iso,
    pub status: Option<String>,
}

impl EditorState {
    pub fn resize(&mut self, w: f32, h: f32) {
        self.iso.screen_w = w;
        self.iso.screen_h = h;
    }
}

pub fn open(app: &mut App, sc: Scenario) {
    let data = app.data;
    let mut world = sc.to_world(data, 1);
    world.settings.reveal_map = true;
    for p in world.players.iter_mut() {
        p.explored.iter_mut().for_each(|t| *t = true);
    }
    let mut m = LocalMatch::new(world, PlayerId(1));
    m.paused = true;
    let mut g = GameState::new(Session::Local(m), app.screen_w, app.screen_h, data);
    g.iso.cx = sc.map.width as f32 / 2.0;
    g.iso.cy = sc.map.height as f32 / 2.0;
    let objects: Vec<TypeId> = data
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.cost.total() > 0
                || o.yields.is_some()
                || o.class == ObjectClass::Relic
                || o.class == ObjectClass::Animal
                || o.key == "king"
        })
        .map(|(i, _)| TypeId(i as u16))
        .collect();
    let iso = g.iso;
    app.editor = Some(EditorState {
        name: sc.name.clone(),
        description: sc.description.clone(),
        scenario: sc,
        tool: Tool::Terrain(Terrain::Grass),
        brush: 1,
        owner: 1,
        objects,
        obj_scroll: 0.0,
        obj_sel: None,
        tab: 0,
        trig_scroll: 0.0,
        trig_sel: None,
        timer_secs: "60".into(),
        trig_text: "Welcome to the scenario!".into(),
        iso,
        status: None,
    });
    app.game = Some(g);
}

const PANEL_W: f32 = 300.0;

pub fn frame(app: &mut App) {
    let data = app.data;
    let (w, h) = (app.screen_w, app.screen_h);
    if app.game.is_none() || app.editor.is_none() {
        app.screen = Screen::Main;
        return;
    }
    // Camera and world drawing reuse the game screen's code.
    {
        let g = app.game.as_mut().unwrap();
        g.iso.bottom = 0.0;
        g.iso.top = crate::game::TOP_BAR;
        let held = g.held.clone();
        let speed = 900.0 * app.frame.dt;
        let (mut dx, mut dy) = (0.0, 0.0);
        if held.contains(&Key::Left) {
            dx -= speed;
        }
        if held.contains(&Key::Right) {
            dx += speed;
        }
        if held.contains(&Key::Up) {
            dy -= speed;
        }
        if held.contains(&Key::Down) {
            dy += speed;
        }
        if app.frame.middle_down || (app.frame.down && app.frame.mods.alt) {
            // handled below via drag
        }
        g.iso.pan(dx, dy);
        let wmap = g.session.world();
        g.iso.clamp_to(wmap.map.width, wmap.map.height);
        if app.frame.wheel != 0.0 && app.frame.mouse.0 < w - PANEL_W {
            let f = if app.frame.wheel > 0.0 {
                1.15
            } else {
                1.0 / 1.15
            };
            g.iso
                .zoom_at(f, app.frame.mouse.0, app.frame.mouse.1, 0.4, 2.5);
        }
    }
    crate::game::draw_world_public(app);
    let mut action: Option<&'static str> = None;
    let panel = Rect::new(w - PANEL_W, 0.0, PANEL_W, h);
    let on_map = !panel.contains(app.frame.mouse) && app.frame.mouse.1 > crate::game::TOP_BAR;
    {
        let ed = app.editor.as_mut().unwrap();
        let g = app.game.as_mut().unwrap();
        let mut ui = Ui::new(&mut app.dl, &mut app.text, &app.frame, &mut app.focus);
        ui.panel_plain(
            Rect::new(0.0, 0.0, w, crate::game::TOP_BAR),
            Color::rgb(28, 20, 12),
        );
        ui.label_color(
            10.0,
            6.0,
            15,
            &format!("Scenario editor — {}", ed.name),
            THEME.accent,
        );
        ui.panel(panel);
        let x = panel.x + 12.0;
        let mut y = 12.0;
        // Tabs.
        for (i, t) in ["Terrain", "Objects", "Players", "Triggers", "File"]
            .iter()
            .enumerate()
        {
            let r = Rect::new(x + i as f32 * 55.0, y, 52.0, 26.0);
            if ui.tile_button(r, t, true, ed.tab == i) {
                ed.tab = i;
            }
        }
        y += 36.0;
        match ed.tab {
            0 => {
                ui.label_dim(x, y, "Click and drag on the map to paint.");
                y += 24.0;
                let terrains = [
                    Terrain::Grass,
                    Terrain::Dirt,
                    Terrain::Sand,
                    Terrain::Forest,
                    Terrain::Water,
                    Terrain::Shallows,
                    Terrain::DeepWater,
                    Terrain::Snow,
                    Terrain::Road,
                    Terrain::Beach,
                    Terrain::Cliff,
                ];
                for (i, t) in terrains.iter().enumerate() {
                    let r = Rect::new(
                        x + (i % 3) as f32 * 92.0,
                        y + (i / 3) as f32 * 32.0,
                        88.0,
                        28.0,
                    );
                    let sel = ed.tool == Tool::Terrain(*t);
                    ui.dl.rect(
                        r.x,
                        r.y,
                        10.0,
                        r.h,
                        wars_render::art::terrain_color(*t, 1, 0),
                    );
                    if ui.tile_button(
                        Rect::new(r.x + 12.0, r.y, r.w - 12.0, r.h),
                        t.name(),
                        true,
                        sel,
                    ) {
                        ed.tool = Tool::Terrain(*t);
                    }
                }
                y += 4.0 * 32.0 + 10.0;
                let mut b = ed.brush as f32;
                ui.slider(
                    Rect::new(x, y, PANEL_W - 24.0, 26.0),
                    &format!("Brush {}", ed.brush),
                    &mut b,
                    1.0,
                    8.0,
                );
                ed.brush = b.round() as i32;
                y += 34.0;
                if ui.tile_button(
                    Rect::new(x, y, 120.0, 28.0),
                    "Erase objects",
                    true,
                    ed.tool == Tool::Erase,
                ) {
                    ed.tool = Tool::Erase;
                }
            }
            1 => {
                ui.label_dim(x, y, "Owner (0 = nature):");
                y += 22.0;
                for o in 0..9u8 {
                    let r = Rect::new(
                        x + (o % 5) as f32 * 54.0,
                        y + (o / 5) as f32 * 30.0,
                        50.0,
                        26.0,
                    );
                    let c = wars_render::art::player_color(PlayerId(o));
                    ui.dl.rect(r.x, r.y, 8.0, r.h, c);
                    if ui.tile_button(
                        Rect::new(r.x + 10.0, r.y, r.w - 10.0, r.h),
                        &format!("{o}"),
                        true,
                        ed.owner == o,
                    ) {
                        ed.owner = o;
                    }
                }
                y += 66.0;
                let items: Vec<String> = ed
                    .objects
                    .iter()
                    .map(|t| data.object(*t).name.clone())
                    .collect();
                let lr = Rect::new(x, y, PANEL_W - 24.0, h - y - 60.0);
                if let Some(i) = ui.list(lr, &items, &mut ed.obj_sel, &mut ed.obj_scroll, 24.0) {
                    ed.tool = Tool::Object(i);
                }
                if let Tool::Object(i) = ed.tool {
                    ui.label_dim(
                        x,
                        h - 50.0,
                        &format!(
                            "Placing: {} (right click: erase)",
                            data.object(ed.objects[i]).name
                        ),
                    );
                }
            }
            2 => {
                ui.label_dim(x, y, "Active players and their civilisation:");
                y += 24.0;
                let civs: Vec<String> = data.civs.iter().map(|c| c.name.clone()).collect();
                for i in 0..8 {
                    let p = &mut ed.scenario.players[i];
                    let r = Rect::new(x, y, 24.0, 24.0);
                    ui.dl.rect(
                        r.x,
                        r.y,
                        8.0,
                        r.h,
                        wars_render::art::player_color(PlayerId(i as u8 + 1)),
                    );
                    ui.checkbox(
                        Rect::new(x + 12.0, y, 60.0, 24.0),
                        &format!("P{}", i + 1),
                        &mut p.active,
                    );
                    let mut ci = data.civs.iter().position(|c| c.key == p.civ).unwrap_or(0);
                    if ui.select(
                        Rect::new(x + 76.0, y, 150.0, 24.0),
                        "",
                        &civs[ci],
                        &mut ci,
                        civs.len(),
                    ) {
                        p.civ = data.civs[ci].key.clone();
                    }
                    let mut ai = p.is_ai;
                    if ui.checkbox(Rect::new(x + 232.0, y, 50.0, 24.0), "AI", &mut ai) {
                        p.is_ai = ai;
                    }
                    y += 30.0;
                }
                y += 10.0;
                ui.label_dim(x, y, "Starting resources apply to all active players:");
                y += 22.0;
                let mut food = ed.scenario.players[0].resources.food as f32;
                ui.slider(
                    Rect::new(x, y, PANEL_W - 24.0, 24.0),
                    &format!("Food {}", food as i32),
                    &mut food,
                    0.0,
                    5000.0,
                );
                y += 28.0;
                let mut wood = ed.scenario.players[0].resources.wood as f32;
                ui.slider(
                    Rect::new(x, y, PANEL_W - 24.0, 24.0),
                    &format!("Wood {}", wood as i32),
                    &mut wood,
                    0.0,
                    5000.0,
                );
                y += 28.0;
                let mut gold = ed.scenario.players[0].resources.gold as f32;
                ui.slider(
                    Rect::new(x, y, PANEL_W - 24.0, 24.0),
                    &format!("Gold {}", gold as i32),
                    &mut gold,
                    0.0,
                    5000.0,
                );
                y += 28.0;
                let mut stone = ed.scenario.players[0].resources.stone as f32;
                ui.slider(
                    Rect::new(x, y, PANEL_W - 24.0, 24.0),
                    &format!("Stone {}", stone as i32),
                    &mut stone,
                    0.0,
                    5000.0,
                );
                y += 32.0;
                for p in ed.scenario.players.iter_mut() {
                    p.resources = Stockpile::new(
                        food as i32 / 50 * 50,
                        wood as i32 / 50 * 50,
                        gold as i32 / 50 * 50,
                        stone as i32 / 50 * 50,
                    );
                }
                let mut age = ed.scenario.players[0].age.idx();
                if ui.select(
                    Rect::new(x, y, PANEL_W - 24.0, 26.0),
                    "Starting age",
                    Age::ALL[age].name(),
                    &mut age,
                    4,
                ) {
                    for p in ed.scenario.players.iter_mut() {
                        p.age = Age::ALL[age];
                    }
                }
            }
            3 => {
                let items: Vec<String> = ed
                    .scenario
                    .triggers
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                let lr = Rect::new(x, y, PANEL_W - 24.0, 160.0);
                ui.list(lr, &items, &mut ed.trig_sel, &mut ed.trig_scroll, 24.0);
                y += 170.0;
                if ui.button_styled(
                    Rect::new(x, y, 130.0, 28.0),
                    "Remove",
                    ed.trig_sel.is_some(),
                    THEME.danger.mul(0.6),
                ) {
                    if let Some(i) = ed.trig_sel.take() {
                        ed.scenario.triggers.remove(i);
                    }
                }
                y += 40.0;
                ui.label_dim(x, y, "Templates:");
                y += 22.0;
                ui.text_field(
                    wars_ui::id("tsecs"),
                    Rect::new(x, y, PANEL_W - 24.0, 26.0),
                    "Seconds",
                    &mut ed.timer_secs,
                    "60",
                    false,
                );
                y += 30.0;
                ui.text_field(
                    wars_ui::id("ttext"),
                    Rect::new(x, y, PANEL_W - 24.0, 26.0),
                    "Message",
                    &mut ed.trig_text,
                    "text",
                    false,
                );
                y += 34.0;
                let secs: u32 = ed.timer_secs.trim().parse().unwrap_or(60);
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "Timer → chat message",
                ) {
                    ed.scenario.triggers.push(Trigger {
                        name: format!("At {secs}s: message"),
                        enabled: true,
                        looping: false,
                        conditions: vec![Condition::Timer { seconds: secs }],
                        effects: vec![TriggerEffect::SendChat {
                            player: None,
                            text: ed.trig_text.clone(),
                        }],
                        objective: false,
                        description: String::new(),
                    });
                }
                y += 32.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "Timer → +500 of each resource (P1)",
                ) {
                    ed.scenario.triggers.push(Trigger {
                        name: format!("At {secs}s: resources"),
                        enabled: true,
                        looping: false,
                        conditions: vec![Condition::Timer { seconds: secs }],
                        effects: vec![TriggerEffect::ChangeResources {
                            player: PlayerId(1),
                            amount: Stockpile::new(500, 500, 500, 500),
                        }],
                        objective: false,
                        description: String::new(),
                    });
                }
                y += 32.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "P1 reaches Castle Age → victory",
                ) {
                    ed.scenario.triggers.push(Trigger {
                        name: "Castle Age wins".into(),
                        enabled: true,
                        looping: false,
                        conditions: vec![Condition::InAge {
                            player: PlayerId(1),
                            age: Age::Castle,
                        }],
                        effects: vec![TriggerEffect::DeclareVictory {
                            player: PlayerId(1),
                        }],
                        objective: true,
                        description: "Reach the Castle Age".into(),
                    });
                }
                y += 32.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "P1 has 20 units in centre → victory",
                ) {
                    let n = ed.scenario.map.width;
                    ed.scenario.triggers.push(Trigger {
                        name: "Hold the centre".into(),
                        enabled: true,
                        looping: false,
                        conditions: vec![Condition::ObjectsInArea {
                            player: PlayerId(1),
                            area: Area {
                                x1: n / 2 - 6,
                                y1: n / 2 - 6,
                                x2: n / 2 + 6,
                                y2: n / 2 + 6,
                            },
                            count: 20,
                        }],
                        effects: vec![TriggerEffect::DeclareVictory {
                            player: PlayerId(1),
                        }],
                        objective: true,
                        description: "Bring 20 units to the centre of the map".into(),
                    });
                }
                y += 32.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "Timer → P2 gets 10 knights at centre",
                ) {
                    let n = ed.scenario.map.width;
                    let mut effects = Vec::new();
                    for i in 0..10 {
                        effects.push(TriggerEffect::CreateObject {
                            player: PlayerId(2),
                            object: data.obj("knight"),
                            x: n / 2 + i % 5,
                            y: n / 2 + i / 5,
                        });
                    }
                    ed.scenario.triggers.push(Trigger {
                        name: format!("At {secs}s: knights"),
                        enabled: true,
                        looping: false,
                        conditions: vec![Condition::Timer { seconds: secs }],
                        effects,
                        objective: false,
                        description: String::new(),
                    });
                }
                y += 40.0;
                ui.label_dim(x, y, "Scenario victory is decided by triggers only");
                ui.label_dim(x, y + 16.0, "when the game mode is Scenario (default).");
            }
            _ => {
                ui.text_field(
                    wars_ui::id("scname"),
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "Name",
                    &mut ed.name,
                    "name",
                    false,
                );
                y += 34.0;
                ui.text_field(
                    wars_ui::id("scdesc"),
                    Rect::new(x, y, PANEL_W - 24.0, 28.0),
                    "Description",
                    &mut ed.description,
                    "",
                    false,
                );
                y += 40.0;
                if ui.button_styled(
                    Rect::new(x, y, PANEL_W - 24.0, 32.0),
                    "Save scenario",
                    true,
                    THEME.accent_dark,
                ) {
                    action = Some("save");
                }
                y += 40.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 32.0),
                    "Test play (as player 1)",
                ) {
                    action = Some("test");
                }
                y += 40.0;
                if ui.button(
                    Rect::new(x, y, PANEL_W - 24.0, 32.0),
                    "Random map as a base...",
                ) {
                    action = Some("random");
                }
                y += 40.0;
                if ui.button_styled(
                    Rect::new(x, y, PANEL_W - 24.0, 32.0),
                    "Exit editor",
                    true,
                    THEME.danger.mul(0.5),
                ) {
                    action = Some("exit");
                }
                y += 50.0;
                if let Some(s) = &ed.status {
                    ui.label_color(x, y, 13, s, THEME.ok);
                }
                ui.label_dim(
                    x,
                    h - 40.0,
                    "Files live in the wars data folder (scenarios/).",
                );
            }
        }
        // Map interaction.
        if on_map && app.focus.is_none() {
            let (wx, wy) = g.iso.to_world(app.frame.mouse.0, app.frame.mouse.1);
            let (tx, ty) = (wx.floor() as i32, wy.floor() as i32);
            let world = g.session.world_mut_editor();
            match ed.tool {
                Tool::Terrain(t) if app.frame.down => {
                    let r = ed.brush - 1;
                    for yy in ty - r..=ty + r {
                        for xx in tx - r..=tx + r {
                            if (xx - tx) * (xx - tx) + (yy - ty) * (yy - ty) <= r * r + r
                                && world.map.in_bounds(xx, yy)
                                && world.map.occupant(xx, yy).is_none()
                            {
                                world.map.set_terrain(xx, yy, t);
                            }
                        }
                    }
                }
                Tool::Object(i) if app.frame.released => {
                    let ty_ = ed.objects[i];
                    let def = data.object(ty_);
                    let owner = PlayerId(ed.owner);
                    let owner = if owner.idx() >= world.players.len() {
                        PlayerId::GAIA
                    } else {
                        owner
                    };
                    let pos = if matches!(def.class, ObjectClass::Unit | ObjectClass::Animal) {
                        Vec2::tile_center(tx, ty)
                    } else {
                        Vec2::from_tile(tx - def.size.0 as i32 / 2, ty - def.size.1 as i32 / 2)
                    };
                    if world.spawn(data, owner, ty_, pos, false).is_none() {
                        ed.status = Some("Cannot place there".into());
                    }
                }
                _ if app.frame.right_released || (ed.tool == Tool::Erase && app.frame.released) => {
                    let at = Vec2::tile_center(tx, ty);
                    if let Some(id) = wars_client::Controller::pick(world, data, at, PlayerId(1))
                        .or_else(|| world.map.occupant(tx, ty))
                    {
                        world.despawn(data, id);
                    }
                }
                _ => {}
            }
        }
        if app.frame.key(Key::Escape) && app.focus.is_none() {
            action = Some("exit_confirm");
        }
    }
    match action {
        Some("save") => {
            let ed = app.editor.as_mut().unwrap();
            let g = app.game.as_ref().unwrap();
            let mut sc = Scenario::from_world(g.session.world(), data, &ed.name);
            // Keep player settings and triggers from the editor state.
            sc.players = ed.scenario.players.clone();
            sc.triggers = ed.scenario.triggers.clone();
            sc.description = ed.description.clone();
            sc.settings = Settings {
                mode: wars_sim::world::GameMode::Scenario,
                victory: if sc.triggers.iter().any(|t| {
                    t.effects
                        .iter()
                        .any(|e| matches!(e, TriggerEffect::DeclareVictory { .. }))
                }) {
                    wars_sim::world::VictoryMode::Scenario
                } else {
                    wars_sim::world::VictoryMode::Standard
                },
                ..Settings::default()
            };
            sc.map.occupancy.iter_mut().for_each(|o| *o = None);
            match wars_client::saves::save_scenario(&sc) {
                Ok(p) => ed.status = Some(format!("Saved {}", p.display())),
                Err(e) => ed.status = Some(format!("Save failed: {e}")),
            }
            ed.scenario = sc;
        }
        Some("test") => {
            let ed = app.editor.as_ref().unwrap();
            let g = app.game.as_ref().unwrap();
            let mut sc = Scenario::from_world(g.session.world(), data, &ed.name);
            sc.players = ed.scenario.players.clone();
            sc.triggers = ed.scenario.triggers.clone();
            sc.settings = Settings {
                mode: wars_sim::world::GameMode::Scenario,
                victory: if sc.triggers.iter().any(|t| {
                    t.effects
                        .iter()
                        .any(|e| matches!(e, TriggerEffect::DeclareVictory { .. }))
                }) {
                    wars_sim::world::VictoryMode::Scenario
                } else {
                    wars_sim::world::VictoryMode::Standard
                },
                ..Settings::default()
            };
            sc.map.occupancy.iter_mut().for_each(|o| *o = None);
            let world = sc.to_world(data, 7);
            let mut gs = GameState::new(
                Session::Local(LocalMatch::new(world, PlayerId(1))),
                app.screen_w,
                app.screen_h,
                data,
            );
            gs.editor_test = true;
            app.editor.as_mut().unwrap().scenario = sc;
            // Keep the editor's world in the editor state by stashing the current game.
            let editor_game = app.game.take().unwrap();
            app.editor_game = Some(editor_game);
            app.game = Some(gs);
            app.screen = Screen::Game;
        }
        Some("random") => {
            let ed = app.editor.as_ref().unwrap();
            let players: Vec<wars_mapgen::PlayerSetup> = ed
                .scenario
                .players
                .iter()
                .filter(|p| p.active)
                .map(|p| wars_mapgen::PlayerSetup {
                    name: p.name.clone(),
                    civ: p.civ.clone(),
                    team: p.team,
                    is_ai: p.is_ai,
                    ai_level: p.ai_level,
                })
                .collect();
            let params = wars_mapgen::MapParams {
                map_type: wars_mapgen::MapType::Arabia,
                size: wars_mapgen::MapSize::Tiny,
                seed: (app.time * 1000.0) as u64 | 1,
                players,
                settings: Settings::default(),
            };
            let mut world = wars_mapgen::generate(data, &params);
            world.settings.reveal_map = true;
            let mut m = LocalMatch::new(world, PlayerId(1));
            m.paused = true;
            let mut g = GameState::new(Session::Local(m), app.screen_w, app.screen_h, data);
            g.iso.cx = params.size.tiles() as f32 / 2.0;
            g.iso.cy = params.size.tiles() as f32 / 2.0;
            app.game = Some(g);
        }
        Some("exit") | Some("exit_confirm") => {
            app.editor = None;
            app.game = None;
            app.menu.scenarios.clear();
            app.screen = Screen::Scenarios;
        }
        _ => {}
    }
}
