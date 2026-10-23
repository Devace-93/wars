//! The classic HUD: resource bar on top; command grid, selection panel and minimap at
//! the bottom; the in-game menu, chat line and score screen.

use crate::game::{self, GameState, BOTTOM_PANEL, TOP_BAR};
use crate::{App, Screen};
use wars_client::Session;
use wars_render::art::{self, player_color};
use wars_render::draw::Color;
use wars_sim::command::{Command, RallyTarget};
use wars_sim::ids::*;
use wars_sim::stats;
use wars_sim::types::{GameData, ObjectClass, Role};
use wars_sim::world::{QueueItem, World};
use wars_ui::{Key, Rect, Ui, THEME};

const GRID_KEYS: [char; 15] = [
    'q', 'w', 'e', 'r', 't', 'a', 's', 'd', 'f', 'g', 'z', 'x', 'c', 'v', 'b',
];

#[derive(Clone)]
enum Cmd {
    Train(TypeId),
    Research(TechId),
    BuildMenu,
    Place(TypeId),
    AttackMove,
    Stop,
    Stance(Stance),
    Patrol,
    Garrison,
    Ungarrison,
    Delete,
    Back,
    Cancel,
    Pack,
    MarketBuy(Resource),
    MarketSell(Resource),
    Unload,
}

struct Cell {
    label: String,
    cmd: Cmd,
    enabled: bool,
    tooltip: Vec<String>,
    highlight: bool,
}

fn cost_str(c: &Stockpile) -> String {
    let mut parts = Vec::new();
    if c.food > 0 {
        parts.push(format!("{}F", c.food));
    }
    if c.wood > 0 {
        parts.push(format!("{}W", c.wood));
    }
    if c.gold > 0 {
        parts.push(format!("{}G", c.gold));
    }
    if c.stone > 0 {
        parts.push(format!("{}S", c.stone));
    }
    parts.join(" ")
}

fn short(name: &str) -> String {
    if name.len() <= 8 {
        return name.to_string();
    }
    let words: Vec<&str> = name.split(' ').collect();
    if words.len() > 1 {
        let s: String = words
            .iter()
            .map(|w| w.chars().next().unwrap_or(' '))
            .collect();
        if s.len() >= 2 {
            return format!(
                "{}.{}",
                &words[0][..words[0].len().min(6)],
                s.chars().skip(1).collect::<String>()
            );
        }
    }
    name.chars().take(7).collect::<String>() + "."
}

/// Build the 15-cell command grid for the current selection.
fn grid_cells(g: &GameState, data: &GameData) -> Vec<Cell> {
    let world = g.session.world();
    let me = g.me();
    let mut cells = Vec::new();
    let sel: Vec<&wars_sim::world::Entity> = g
        .ctrl
        .selection
        .iter()
        .filter_map(|id| world.get(*id))
        .collect();
    if sel.is_empty() {
        return cells;
    }
    let first = sel[0];
    if first.owner != me {
        return cells;
    }
    let def = data.object(first.ty);
    let res = world.player(me).resources;
    if g.build_menu {
        // Buildings available to villagers.
        for ty in &data.buildable {
            let bdef = data.object(*ty);
            if bdef.is_town_center && world.player(me).age < Age::Castle {
                continue;
            }
            let st = stats::stats_for(world, data, me, *ty);
            let available = stats::is_available(world, data, me, *ty);
            let affordable = res.covers(&st.cost);
            let mut tip = vec![bdef.name.clone(), cost_str(&st.cost)];
            if !available {
                let why = if bdef.age > world.player(me).age {
                    format!("Requires {}", bdef.age.name())
                } else if let Some(r) = bdef
                    .requires
                    .iter()
                    .find(|r| !world.has_building(data, me, r))
                {
                    format!("Requires {}", data.object(data.obj(r)).name)
                } else if bdef.requires_tech.is_some() {
                    "Requires a technology".into()
                } else {
                    "Not available to your civilisation".into()
                };
                tip.push(why);
            }
            cells.push(Cell {
                label: short(&bdef.name),
                cmd: Cmd::Place(*ty),
                enabled: available && affordable,
                tooltip: tip,
                highlight: g.placing == Some(*ty),
            });
            if cells.len() == 14 {
                break;
            }
        }
        cells.push(Cell {
            label: "Back".into(),
            cmd: Cmd::Back,
            enabled: true,
            tooltip: vec!["Back".into()],
            highlight: false,
        });
        return cells;
    }
    match def.class {
        ObjectClass::Unit => {
            let all_villagers = sel.iter().all(|e| data.object(e.ty).is_villager);
            if all_villagers {
                cells.push(Cell {
                    label: "Build".into(),
                    cmd: Cmd::BuildMenu,
                    enabled: true,
                    tooltip: vec!["Build (B)".into()],
                    highlight: false,
                });
            }
            let fighter = sel.iter().any(|e| !data.object(e.ty).attacks.is_empty());
            if fighter {
                cells.push(Cell {
                    label: "Attack".into(),
                    cmd: Cmd::AttackMove,
                    enabled: true,
                    tooltip: vec![
                        "Attack-move".into(),
                        "Click a point; fight anything on the way".into(),
                    ],
                    highlight: false,
                });
            }
            cells.push(Cell {
                label: "Stop".into(),
                cmd: Cmd::Stop,
                enabled: true,
                tooltip: vec!["Stop".into()],
                highlight: false,
            });
            if fighter && !all_villagers {
                cells.push(Cell {
                    label: "Patrol".into(),
                    cmd: Cmd::Patrol,
                    enabled: true,
                    tooltip: vec!["Patrol between here and a point".into()],
                    highlight: false,
                });
                for (s, name) in [
                    (Stance::Aggressive, "Aggr."),
                    (Stance::Defensive, "Defend"),
                    (Stance::StandGround, "Stand"),
                    (Stance::NoAttack, "NoAtk"),
                ] {
                    cells.push(Cell {
                        label: name.into(),
                        cmd: Cmd::Stance(s),
                        enabled: true,
                        tooltip: vec![format!("Stance: {name}")],
                        highlight: first.stance == s,
                    });
                }
            }
            cells.push(Cell {
                label: "Garrison".into(),
                cmd: Cmd::Garrison,
                enabled: true,
                tooltip: vec![
                    "Garrison into a building".into(),
                    "Then click the building".into(),
                ],
                highlight: false,
            });
            if def.key == "trebuchet" {
                cells.push(Cell {
                    label: if first.packed {
                        "Unpack".into()
                    } else {
                        "Pack".into()
                    },
                    cmd: Cmd::Pack,
                    enabled: true,
                    tooltip: vec!["Pack / unpack the trebuchet".into()],
                    highlight: false,
                });
            }
            if def.carries_units > 0 || first.relics > 0 {
                cells.push(Cell {
                    label: "Unload".into(),
                    cmd: Cmd::Unload,
                    enabled: true,
                    tooltip: vec!["Unload units / drop relic".into()],
                    highlight: false,
                });
            }
            while cells.len() < 14 {
                cells.push(Cell {
                    label: String::new(),
                    cmd: Cmd::Back,
                    enabled: false,
                    tooltip: vec![],
                    highlight: false,
                });
            }
            cells.push(Cell {
                label: "Delete".into(),
                cmd: Cmd::Delete,
                enabled: true,
                tooltip: vec!["Delete unit (Del)".into()],
                highlight: false,
            });
        }
        ObjectClass::Building => {
            if first.build_progress < 1000 {
                cells.push(Cell {
                    label: "Cancel".into(),
                    cmd: Cmd::Cancel,
                    enabled: true,
                    tooltip: vec!["Cancel construction (refund)".into()],
                    highlight: false,
                });
                return cells;
            }
            if let Some(units) = data.trains_at.get(&first.ty) {
                for u in units {
                    let ty = stats::resolve_upgrade(world, data, me, *u);
                    // Skip types that are superseded in the list (e.g. militia when man-at-arms exists).
                    if ty != *u && units.contains(&ty) {
                        continue;
                    }
                    let udef = data.object(ty);
                    let available = stats::is_available(world, data, me, ty);
                    if !available
                        && udef
                            .armours
                            .iter()
                            .any(|a| a.class == ArmourClass::UniqueUnit)
                    {
                        continue;
                    }
                    let st = stats::stats_for(world, data, me, ty);
                    let mut tip = vec![
                        format!("{} ({}s)", udef.name, st.train_ticks / 20),
                        cost_str(&st.cost),
                    ];
                    tip.push(format!(
                        "HP {}  Atk {}  Arm {}/{}  Rng {}",
                        st.hp,
                        st.attacks
                            .iter()
                            .filter(|a| matches!(a.class, ArmourClass::Melee | ArmourClass::Pierce))
                            .map(|a| a.value)
                            .sum::<i32>(),
                        st.armours
                            .iter()
                            .find(|a| a.class == ArmourClass::Melee)
                            .map(|a| a.value)
                            .unwrap_or(0),
                        st.armours
                            .iter()
                            .find(|a| a.class == ArmourClass::Pierce)
                            .map(|a| a.value)
                            .unwrap_or(0),
                        st.range.to_int()
                    ));
                    if !available {
                        tip.push(if udef.age > world.player(me).age {
                            format!("Requires {}", udef.age.name())
                        } else {
                            "Not available".into()
                        });
                    }
                    cells.push(Cell {
                        label: short(&udef.name),
                        cmd: Cmd::Train(ty),
                        enabled: available && res.covers(&st.cost),
                        tooltip: tip,
                        highlight: false,
                    });
                }
            }
            if let Some(techs) = data.techs_at.get(&first.ty) {
                for t in techs {
                    let tdef = data.tech(*t);
                    let p = world.player(me);
                    if p.has_tech(*t) {
                        continue;
                    }
                    // Hide techs whose prerequisite tech is not done yet, and other civs' uniques.
                    if let Some(u) = &tdef.unique_to {
                        if *u != data.civ(p.civ).key {
                            continue;
                        }
                    }
                    if data.civ(p.civ).disabled_techs.contains(&tdef.key) {
                        continue;
                    }
                    let prereq_missing = tdef
                        .requires
                        .iter()
                        .any(|r| data.tech_id(r).map(|id| !p.has_tech(id)).unwrap_or(false));
                    if prereq_missing {
                        continue;
                    }
                    let (cost, ticks) = stats::tech_cost(world, data, me, *t);
                    let available = stats::tech_available(world, data, me, *t);
                    let mut tip = vec![
                        format!("{} ({}s)", tdef.name, ticks / 20),
                        cost_str(&cost),
                        tdef.description.clone(),
                    ];
                    if p.researching.contains(t) {
                        tip.push("Researching...".into());
                    } else if !available && tdef.age > p.age {
                        tip.push(format!("Requires {}", tdef.age.name()));
                    } else if !available && tdef.advances_to.is_some() {
                        tip.push("Finish the previous age first".into());
                    }
                    cells.push(Cell {
                        label: short(&tdef.name),
                        cmd: Cmd::Research(*t),
                        enabled: available && res.covers(&cost),
                        tooltip: tip,
                        highlight: false,
                    });
                    if cells.len() >= 12 {
                        break;
                    }
                }
            }
            if def.is_market {
                for r in [Resource::Food, Resource::Wood, Resource::Stone] {
                    let p = world.player(me);
                    let price = match r {
                        Resource::Food => p.market_food,
                        Resource::Wood => p.market_wood,
                        _ => p.market_stone,
                    };
                    cells.push(Cell {
                        label: format!("Buy {}", r.name()),
                        cmd: Cmd::MarketBuy(r),
                        enabled: true,
                        tooltip: vec![format!(
                            "Buy 100 {} for ~{} gold",
                            r.name(),
                            price * 13 / 10
                        )],
                        highlight: false,
                    });
                    cells.push(Cell {
                        label: format!("Sell {}", r.name()),
                        cmd: Cmd::MarketSell(r),
                        enabled: true,
                        tooltip: vec![format!(
                            "Sell 100 {} for ~{} gold",
                            r.name(),
                            price * 7 / 10
                        )],
                        highlight: false,
                    });
                }
            }
            while cells.len() < 13 {
                cells.push(Cell {
                    label: String::new(),
                    cmd: Cmd::Back,
                    enabled: false,
                    tooltip: vec![],
                    highlight: false,
                });
            }
            cells.truncate(13);
            if !first.garrisoned.is_empty() {
                cells.push(Cell {
                    label: "Ungarr.".into(),
                    cmd: Cmd::Ungarrison,
                    enabled: true,
                    tooltip: vec!["Ungarrison all".into()],
                    highlight: false,
                });
            } else {
                cells.push(Cell {
                    label: String::new(),
                    cmd: Cmd::Back,
                    enabled: false,
                    tooltip: vec![],
                    highlight: false,
                });
            }
            cells.push(Cell {
                label: "Delete".into(),
                cmd: Cmd::Delete,
                enabled: true,
                tooltip: vec!["Delete building (Del)".into()],
                highlight: false,
            });
        }
        _ => {}
    }
    cells
}

fn run_cmd(app: &mut App, cmd: Cmd) {
    let data = app.data;
    let time = app.time;
    let g = app.game.as_mut().unwrap();
    let world = g.session.world();
    let sel = g.ctrl.selection.clone();
    let units = g.ctrl.selected_units(world, data);
    match cmd {
        Cmd::Train(ty) => {
            for b in g.ctrl.selected_buildings(world, data) {
                g.session.issue(Command::Train {
                    building: b,
                    unit: ty,
                });
            }
        }
        Cmd::Research(t) => {
            if let Some(b) = g.ctrl.selected_buildings(world, data).first().copied() {
                g.session.issue(Command::Research {
                    building: b,
                    tech: t,
                });
            }
        }
        Cmd::BuildMenu => g.build_menu = true,
        Cmd::Back => g.build_menu = false,
        Cmd::Place(ty) => {
            g.placing = Some(ty);
            g.wall_start = None;
        }
        Cmd::AttackMove => {
            g.message(
                "Click where to attack-move",
                time,
                Color::rgb(220, 220, 220),
            );
            g.pending_point = Some(Pending::AttackMove);
        }
        Cmd::Patrol => {
            g.message("Click where to patrol", time, Color::rgb(220, 220, 220));
            g.pending_point = Some(Pending::Patrol);
        }
        Cmd::Garrison => {
            g.message(
                "Click the building to garrison in",
                time,
                Color::rgb(220, 220, 220),
            );
            g.pending_point = Some(Pending::Garrison);
        }
        Cmd::Stop => g.session.issue(Command::Stop { units }),
        Cmd::Stance(s) => g.session.issue(Command::SetStance { units, stance: s }),
        Cmd::Ungarrison => {
            for b in g.ctrl.selected_buildings(world, data) {
                g.session.issue(Command::Ungarrison { building: b });
            }
        }
        Cmd::Delete | Cmd::Cancel => {
            if let Some(id) = sel.first().copied() {
                g.session.issue(Command::Delete { entity: id });
            }
        }
        Cmd::Pack => g.session.issue(Command::TogglePack { units }),
        Cmd::Unload => g.session.issue(Command::Unload { units }),
        Cmd::MarketBuy(r) => g.session.issue(Command::MarketBuy { resource: r }),
        Cmd::MarketSell(r) => g.session.issue(Command::MarketSell { resource: r }),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pending {
    AttackMove,
    Patrol,
    Garrison,
}

/// Draw every HUD element and handle its input; then let the map handle the rest.
pub fn draw_and_handle(app: &mut App) {
    let data = app.data;
    let time = app.time;
    let (sw, sh) = (app.screen_w, app.screen_h);
    // Hotkeys first (grid keys are matched against the cells below).
    game::hotkeys(app);
    let cells = grid_cells(app.game.as_ref().unwrap(), data);
    // Grid hotkeys.
    let mut fire: Option<Cmd> = None;
    {
        let g = app.game.as_ref().unwrap();
        if !g.chat_open && !g.menu_open && app.focus.is_none() {
            for (k, m) in &app.frame.keys {
                if let Key::Char(c) = k {
                    if m.ctrl {
                        continue;
                    }
                    if let Some(i) = GRID_KEYS.iter().position(|gk| gk == c) {
                        if let Some(cell) = cells.get(i) {
                            if cell.enabled {
                                fire = Some(cell.cmd.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(c) = fire {
        run_cmd(app, c);
    }
    let cells = grid_cells(app.game.as_ref().unwrap(), data);
    let mut clicked: Option<Cmd> = None;
    let mut tooltip: Vec<String> = Vec::new();
    let mut minimap_click: Option<(f32, f32, bool)> = None;
    let mut consumed;
    let mut menu_action: Option<MenuAction> = None;
    {
        let g = app.game.as_mut().unwrap();
        let world = g.session.world();
        let me = g.me();
        let p = world.player(me);
        let mut ui = Ui::new(&mut app.dl, &mut app.text, &app.frame, &mut app.focus);
        // ---- Top bar ----
        ui.panel_plain(Rect::new(0.0, 0.0, sw, TOP_BAR), Color::rgb(28, 20, 12));
        ui.dl.rect(0.0, TOP_BAR - 2.0, sw, 2.0, THEME.panel_border);
        let mut x = 10.0;
        for (name, val, c) in [
            ("Food", p.resources.food, Color::rgb(220, 70, 60)),
            ("Wood", p.resources.wood, Color::rgb(150, 100, 50)),
            ("Gold", p.resources.gold, Color::rgb(240, 200, 40)),
            ("Stone", p.resources.stone, Color::rgb(170, 170, 175)),
        ] {
            ui.dl.rect(x, 8.0, 14.0, 14.0, c);
            ui.label(x + 20.0, 6.0, &format!("{val}"));
            let w = ui.measure(15, &format!("{val}"));
            let _ = name;
            x += 20.0 + w + 22.0;
        }
        let cap = stats::pop_cap(world, data, me).min(p.pop_support);
        let pop_col = if p.pop >= cap {
            Color::rgb(240, 90, 60)
        } else {
            THEME.text
        };
        ui.label_color(x, 6.0, 15, &format!("Pop {}/{}", p.pop, cap), pop_col);
        x += 110.0;
        ui.label_color(
            x,
            6.0,
            15,
            &format!("{}  ·  {}", p.age.name(), data.civ(p.civ).name),
            THEME.accent,
        );
        let t = game::game_time(world);
        let speed = g.session.speed_pct();
        let right = format!(
            "{t}   {}x{}",
            speed as f32 / 100.0,
            if g.session.paused() { "  PAUSED" } else { "" }
        );
        ui.text.draw_right(
            ui.dl,
            wars_render::text::Font::Regular,
            15,
            sw - 10.0,
            6.0,
            &right,
            THEME.text,
        );
        if g.session.is_online() {
            if let Session::Remote(r) = &g.session {
                let s = format!(
                    "{} {}",
                    if r.transport.is_closed() {
                        "disconnected"
                    } else {
                        "online"
                    },
                    r.last_rtt_ms.map(|m| format!("{m} ms")).unwrap_or_default()
                );
                ui.text.draw_right(
                    ui.dl,
                    wars_render::text::Font::Regular,
                    12,
                    sw - 10.0,
                    17.0,
                    &s,
                    THEME.text_dim,
                );
            }
        }
        // Idle villager button.
        let idle = wars_client::Controller::idle_villagers(world, data, me);
        if !idle.is_empty() {
            let r = Rect::new(sw - 330.0, 3.0, 90.0, 24.0);
            if ui.button_styled(r, &format!("Idle: {}", idle.len()), true, THEME.accent_dark) {
                app.pending_keys_push = Some(Key::Char('.'));
            }
        }
        // ---- Messages (bottom-left above the panel) ----
        let now = time;
        g.messages.retain(|(_, until, _)| *until > now);
        let mut my = sh - BOTTOM_PANEL - 12.0 - g.messages.len() as f32 * 19.0;
        for (m, _, c) in &g.messages {
            let mw = ui.measure(14, m) + 12.0;
            ui.dl
                .rect(8.0, my - 2.0, mw, 19.0, Color::rgba(0, 0, 0, 140));
            ui.label_color(14.0, my, 14, m, *c);
            my += 19.0;
        }
        // ---- Bottom panel ----
        let panel = Rect::new(0.0, sh - BOTTOM_PANEL, sw, BOTTOM_PANEL);
        ui.panel(panel);
        // Command grid: 5 x 3 cells on the left.
        let cell = 52.0;
        let gap = 4.0;
        let gx = 12.0;
        let gy = panel.y + 10.0;
        for (i, c) in cells.iter().enumerate().take(15) {
            let r = Rect::new(
                gx + (i % 5) as f32 * (cell + gap),
                gy + (i / 5) as f32 * (cell + gap),
                cell,
                cell,
            );
            if c.label.is_empty() {
                ui.dl.rect(r.x, r.y, r.w, r.h, THEME.field.alpha(0.5));
                continue;
            }
            if ui.tile_button(r, &c.label, c.enabled, c.highlight) {
                clicked = Some(c.cmd.clone());
            }
            ui.label_color(
                r.x + 3.0,
                r.y + 1.0,
                11,
                &GRID_KEYS[i].to_uppercase().to_string(),
                THEME.accent,
            );
            if r.contains(app.frame.mouse) {
                tooltip = c.tooltip.clone();
            }
        }
        // Selection panel in the middle.
        let sx = gx + 5.0 * (cell + gap) + 16.0;
        let sel_r = Rect::new(sx, panel.y + 10.0, sw - sx - 236.0, BOTTOM_PANEL - 20.0);
        ui.dl
            .rect(sel_r.x, sel_r.y, sel_r.w, sel_r.h, THEME.field.alpha(0.6));
        draw_selection_panel(&mut ui, g, data, sel_r);
        // Minimap on the right.
        let mm = Rect::new(sw - 226.0, panel.y + 10.0, 216.0, BOTTOM_PANEL - 20.0);
        ui.dl.rect(mm.x, mm.y, mm.w, mm.h, Color::rgb(10, 8, 6));
        ui.dl
            .rect_outline(mm.x, mm.y, mm.w, mm.h, 1.5, THEME.panel_border);
        draw_minimap(&mut ui, g, data, mm);
        if mm.contains(app.frame.mouse) {
            ui.hovered_any = true;
            if app.frame.down || app.frame.right_released {
                let (wx, wy) = minimap_to_world(g.session.world(), mm, app.frame.mouse);
                minimap_click = Some((wx, wy, app.frame.right_released));
            }
        }
        consumed = ui.consumed_click
            || panel.contains(app.frame.mouse)
            || Rect::new(0.0, 0.0, sw, TOP_BAR).contains(app.frame.mouse);
        // ---- Chat line ----
        if g.chat_open {
            let r = Rect::new(10.0, sh - BOTTOM_PANEL - 40.0, 500.0, 28.0);
            ui.text_field(
                wars_ui::id("chat"),
                r,
                "",
                &mut g.chat_text,
                "Say something (Enter sends, Shift+Enter to allies)",
                false,
            );
            consumed = true;
        }
        // ---- Tooltip ----
        if !tooltip.is_empty() {
            ui.tooltip(&tooltip);
        }
        // ---- In-game menu ----
        if g.menu_open {
            consumed = true;
            let w = 320.0;
            let h = 400.0;
            let r = Rect::new((sw - w) / 2.0, (sh - h) / 2.0 - 40.0, w, h);
            ui.dl.rect(0.0, 0.0, sw, sh, Color::rgba(0, 0, 0, 120));
            ui.panel(r);
            ui.heading(r.x + 20.0, r.y + 16.0, "Game menu");
            let mut y = r.y + 60.0;
            let bw = w - 40.0;
            if ui.button(Rect::new(r.x + 20.0, y, bw, 34.0), "Resume") {
                menu_action = Some(MenuAction::Resume);
            }
            y += 42.0;
            ui.text_field(
                wars_ui::id("savename"),
                Rect::new(r.x + 20.0, y, bw, 30.0),
                "",
                &mut g.save_name,
                "Save name",
                false,
            );
            y += 36.0;
            if ui.button(
                Rect::new(r.x + 20.0, y, bw, 34.0),
                if g.session.is_online() {
                    "Save online"
                } else {
                    "Save game"
                },
            ) {
                menu_action = Some(MenuAction::Save);
            }
            y += 42.0;
            if !g.session.is_online() && ui.button(Rect::new(r.x + 20.0, y, bw, 34.0), "Load game")
            {
                menu_action = Some(MenuAction::Load);
            }
            y += 42.0;
            let pause_label = if g.session.paused() {
                "Unpause"
            } else {
                "Pause"
            };
            if ui.button(Rect::new(r.x + 20.0, y, bw, 34.0), pause_label) {
                menu_action = Some(MenuAction::TogglePause);
            }
            y += 42.0;
            if ui.button_styled(
                Rect::new(r.x + 20.0, y, bw, 34.0),
                "Resign",
                true,
                THEME.danger.mul(0.7),
            ) {
                menu_action = Some(MenuAction::Resign);
            }
            y += 42.0;
            let quit_label = if g.confirm_quit {
                "Really quit? (click again)"
            } else {
                "Quit to main menu"
            };
            if ui.button_styled(
                Rect::new(r.x + 20.0, y, bw, 34.0),
                quit_label,
                true,
                THEME.danger.mul(0.5),
            ) {
                menu_action = Some(MenuAction::Quit);
            }
            y += 42.0;
            ui.label_dim(r.x + 20.0, y, "Esc closes · F11 quick save · F3 pause");
        }
        // ---- Score screen ----
        if g.show_scores {
            consumed = true;
            let w = 560.0;
            let h = 120.0 + world.players.len() as f32 * 26.0;
            let r = Rect::new((sw - w) / 2.0, 80.0, w, h);
            ui.panel(r);
            let my_team = {
                let t = world.player(me).team;
                if t == 0 {
                    100 + me.0
                } else {
                    t
                }
            };
            let title = if world.winner_team == Some(my_team) {
                "Victory!"
            } else {
                "Defeat"
            };
            ui.title(r.x + w / 2.0, r.y + 10.0, title);
            let mut y = r.y + 64.0;
            ui.label_dim(r.x + 20.0, y, "Player");
            ui.label_dim(r.x + 220.0, y, "Civ");
            ui.label_dim(r.x + 330.0, y, "Score");
            ui.label_dim(r.x + 400.0, y, "Kills");
            ui.label_dim(r.x + 460.0, y, "Age");
            y += 22.0;
            for p in world.players.iter().skip(1) {
                ui.label_color(r.x + 20.0, y, 15, &p.name, player_color(p.id));
                ui.label(r.x + 220.0, y, &data.civ(p.civ).name);
                ui.label(r.x + 330.0, y, &format!("{}", p.score));
                ui.label(r.x + 400.0, y, &format!("{}", p.kills));
                ui.label(r.x + 460.0, y, p.age.name().split(' ').next().unwrap_or(""));
                y += 26.0;
            }
            if ui.button(
                Rect::new(r.x + w - 160.0, r.bottom() - 44.0, 140.0, 32.0),
                "Main menu",
            ) {
                menu_action = Some(MenuAction::QuitNow);
            }
            if ui.button(
                Rect::new(r.x + 20.0, r.bottom() - 44.0, 140.0, 32.0),
                "Keep watching",
            ) {
                g.show_scores = false;
            }
        }
    }
    if let Some(k) = app.pending_keys_push.take() {
        app.frame.keys.push((k, Default::default()));
        game::hotkeys(app);
    }
    if let Some(c) = clicked {
        run_cmd(app, c);
    }
    if let Some((wx, wy, right)) = minimap_click {
        let g = app.game.as_mut().unwrap();
        if right {
            let world = g.session.world();
            let units = g.ctrl.selected_units(world, data);
            if !units.is_empty() {
                g.session.issue(Command::Move {
                    units,
                    to: wars_sim::Vec2::new(
                        wars_sim::Fixed::from_decimal(&format!("{wx:.2}")),
                        wars_sim::Fixed::from_decimal(&format!("{wy:.2}")),
                    ),
                    queue: app.frame.mods.shift,
                });
            }
        } else {
            g.iso.cx = wx;
            g.iso.cy = wy;
        }
    }
    if let Some(a) = menu_action {
        menu_do(app, a);
    }
    // Pending point commands (attack-move, patrol, garrison) take the next map click.
    {
        // Quitting from the menu drops the game: nothing left to do this frame.
        let Some(g) = app.game.as_mut() else {
            return;
        };
        if let Some(p) = g.pending_point {
            let f = &app.frame;
            if f.right_released || f.key(Key::Escape) {
                g.pending_point = None;
            } else if f.released && !consumed && g.map_rect().contains(f.mouse) {
                let (wx, wy) = g.iso.to_world(f.mouse.0, f.mouse.1);
                let at = wars_sim::Vec2::new(
                    wars_sim::Fixed::from_decimal(&format!("{wx:.2}")),
                    wars_sim::Fixed::from_decimal(&format!("{wy:.2}")),
                );
                let world = g.session.world();
                let me = g.me();
                let units = g.ctrl.selected_units(world, data);
                match p {
                    Pending::AttackMove => g.session.issue(Command::AttackMove {
                        units,
                        to: at,
                        queue: f.mods.shift,
                    }),
                    Pending::Patrol => g.session.issue(Command::Patrol { units, to: at }),
                    Pending::Garrison => {
                        if let Some(t) = wars_client::Controller::pick(world, data, at, me) {
                            g.session.issue(Command::Garrison { units, target: t });
                        }
                    }
                }
                g.pending_point = None;
                consumed = true;
                g.box_start = None;
            }
        }
    }
    if !app.game.as_ref().unwrap().menu_open && !app.game.as_ref().unwrap().show_scores {
        game::map_input(app, consumed);
    }
}

#[derive(Clone, Copy)]
enum MenuAction {
    Resume,
    Save,
    Load,
    TogglePause,
    Resign,
    Quit,
    QuitNow,
}

fn menu_do(app: &mut App, a: MenuAction) {
    let time = app.time;
    let g = app.game.as_mut().unwrap();
    match a {
        MenuAction::Resume => {
            g.menu_open = false;
            g.confirm_quit = false;
        }
        MenuAction::Save => {
            let name = if g.save_name.trim().is_empty() {
                format!("Save {}", game::game_time(g.session.world()))
            } else {
                g.save_name.clone()
            };
            match &mut g.session {
                Session::Local(m) => match wars_client::saves::save_world(&name, &m.world) {
                    Ok(p) => g.message(
                        format!("Saved to {}", p.display()),
                        time,
                        Color::rgb(200, 240, 200),
                    ),
                    Err(e) => {
                        g.message(format!("Save failed: {e}"), time, Color::rgb(240, 120, 90))
                    }
                },
                Session::Remote(r) => {
                    r.send(&wars_proto::ClientMsg::SaveGame { name });
                    g.message("Save requested", time, Color::rgb(200, 240, 200));
                }
            }
            g.menu_open = false;
        }
        MenuAction::Load => {
            app.menu.return_to_game = true;
            app.screen = Screen::LoadGame;
        }
        MenuAction::TogglePause => {
            let p = !g.session.paused();
            g.session.set_paused(p);
            g.menu_open = false;
        }
        MenuAction::Resign => {
            g.session.issue(Command::Resign);
            g.menu_open = false;
        }
        MenuAction::Quit => {
            if g.confirm_quit {
                leave_game(app);
            } else {
                g.confirm_quit = true;
            }
        }
        MenuAction::QuitNow => leave_game(app),
    }
}

fn draw_selection_panel(ui: &mut Ui, g: &GameState, data: &GameData, r: Rect) {
    let world = g.session.world();
    let me = g.me();
    let sel: Vec<&wars_sim::world::Entity> = g
        .ctrl
        .selection
        .iter()
        .filter_map(|id| world.get(*id))
        .collect();
    if sel.is_empty() {
        ui.label_dim(r.x + 12.0, r.y + 10.0, "Nothing selected");
        ui.label_dim(
            r.x + 12.0,
            r.y + 32.0,
            "Left click / drag: select · Right click: act · Grid keys: commands",
        );
        ui.label_dim(
            r.x + 12.0,
            r.y + 50.0,
            "H: town centre · .: idle villager · Enter: chat · Esc: menu",
        );
        return;
    }
    let e = sel[0];
    let def = data.object(e.ty);
    let st = stats::stats_for(world, data, e.owner, e.ty);
    let owner = world.player(e.owner);
    let name = if sel.len() > 1 {
        format!("{} ×{}", def.name, sel.len())
    } else {
        def.name.clone()
    };
    ui.label_color(r.x + 12.0, r.y + 8.0, 17, &name, THEME.accent);
    ui.label_color(
        r.x + 12.0,
        r.y + 30.0,
        13,
        &owner.name,
        player_color(e.owner),
    );
    // HP bar.
    let hp_r = Rect::new(r.x + 12.0, r.y + 50.0, 160.0, 10.0);
    ui.progress(
        hp_r,
        e.hp as f32 / st.hp.max(1) as f32,
        if e.hp * 3 > st.hp * 2 {
            THEME.ok
        } else {
            THEME.danger
        },
    );
    ui.label_dim(r.x + 178.0, r.y + 46.0, &format!("{}/{}", e.hp, st.hp));
    let mut y = r.y + 68.0;
    if def.class == ObjectClass::Unit || !def.attacks.is_empty() {
        let atk: i32 = st
            .attacks
            .iter()
            .filter(|a| matches!(a.class, ArmourClass::Melee | ArmourClass::Pierce))
            .map(|a| a.value)
            .sum();
        let ma = st
            .armours
            .iter()
            .find(|a| a.class == ArmourClass::Melee)
            .map(|a| a.value)
            .unwrap_or(0);
        let pa = st
            .armours
            .iter()
            .find(|a| a.class == ArmourClass::Pierce)
            .map(|a| a.value)
            .unwrap_or(0);
        let mut s = format!("Attack {atk}   Armour {ma}/{pa}");
        if st.range > wars_sim::Fixed::ZERO {
            s += &format!("   Range {}", st.range.to_int());
        }
        if def.class == ObjectClass::Unit {
            s += &format!("   Speed {:.1}", st.speed.to_f32());
        }
        ui.label_dim(r.x + 12.0, y, &s);
        y += 18.0;
    }
    if e.owner == me || world.ally(me, e.owner) {
        if def.class == ObjectClass::Unit {
            let task = match e.task() {
                wars_sim::world::Task::Idle => "Idle".to_string(),
                wars_sim::world::Task::Move {
                    attack_move: true, ..
                } => "Attack-moving".into(),
                wars_sim::world::Task::Move { .. } => "Moving".into(),
                wars_sim::world::Task::Attack { .. } => "Attacking".into(),
                wars_sim::world::Task::Gather { kind, .. } => {
                    format!("Gathering {}", kind.resource().name())
                }
                wars_sim::world::Task::DropOff { .. } => "Returning resources".into(),
                wars_sim::world::Task::Build { .. } => "Building".into(),
                wars_sim::world::Task::Repair { .. } => "Repairing".into(),
                wars_sim::world::Task::Garrison { .. } => "Garrisoning".into(),
                wars_sim::world::Task::Follow { .. } => "Following".into(),
                wars_sim::world::Task::Guard { .. } => "Guarding".into(),
                wars_sim::world::Task::Convert { .. } => "Converting".into(),
                wars_sim::world::Task::Heal { .. } => "Healing".into(),
                wars_sim::world::Task::Trade { .. } => "Trading".into(),
                wars_sim::world::Task::PickRelic { .. }
                | wars_sim::world::Task::DeliverRelic { .. } => "Carrying relic".into(),
                wars_sim::world::Task::Packing { .. } => "Packing".into(),
            };
            let carry = if e.carry > 0 {
                format!(
                    "   carrying {} {}",
                    e.carry,
                    e.carry_kind.map(|k| k.resource().name()).unwrap_or("")
                )
            } else {
                String::new()
            };
            ui.label_dim(r.x + 12.0, y, &format!("{task}{carry}"));
            y += 18.0;
        }
        if def.class == ObjectClass::Building {
            if e.build_progress < 1000 {
                ui.label_dim(
                    r.x + 12.0,
                    y,
                    &format!("Under construction {}%", e.build_progress / 10),
                );
                y += 18.0;
            }
            if !e.queue.is_empty() {
                let (label, total) = match e.queue[0] {
                    QueueItem::Unit(u) => (
                        data.object(u).name.clone(),
                        stats::stats_for(world, data, e.owner, u).train_ticks,
                    ),
                    QueueItem::Tech(t) => (
                        data.tech(t).name.clone(),
                        stats::tech_cost(world, data, e.owner, t).1,
                    ),
                };
                ui.label_dim(
                    r.x + 12.0,
                    y,
                    &format!("{label} ({} queued)", e.queue.len()),
                );
                ui.progress(
                    Rect::new(r.x + 12.0, y + 18.0, 200.0, 8.0),
                    e.queue_progress as f32 / total.max(1) as f32,
                    THEME.accent,
                );
                y += 30.0;
            }
            if !e.garrisoned.is_empty() {
                ui.label_dim(
                    r.x + 12.0,
                    y,
                    &format!("Garrisoned: {}", e.garrisoned.len()),
                );
                y += 18.0;
            }
            if let Some(y_) = def.yields {
                ui.label_dim(
                    r.x + 12.0,
                    y,
                    &format!("{} left: {}", y_.resource.name(), e.resource_left),
                );
                y += 18.0;
            }
            if def.is_monastery && e.relics > 0 {
                ui.label_dim(r.x + 12.0, y, &format!("Relics: {}", e.relics));
            }
        }
    } else if def.yields.is_some() {
        ui.label_dim(
            r.x + 12.0,
            y,
            &format!(
                "{} left: {}",
                def.yields.unwrap().resource.name(),
                e.resource_left
            ),
        );
    }
    let _ = (Role::None, RallyTarget::None);
}

fn draw_minimap(ui: &mut Ui, g: &GameState, data: &GameData, r: Rect) {
    let world = g.session.world();
    let me = g.me();
    let n = world.map.width.max(world.map.height) as f32;
    // Diamond fit: width of the diamond = r.w, height = r.h.
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let step = ((n / 70.0).ceil() as i32).max(1);
    let half_w = r.w / 2.0 / n * step as f32;
    let half_h = r.h / 2.0 / n * step as f32;
    let to_mm = |x: f32, y: f32| {
        (
            cx + (x - y) / n * r.w / 2.0,
            cy + (x + y - n) / n * r.h / 2.0,
        )
    };
    let mut y = 0;
    while y < world.map.height {
        let mut x = 0;
        while x < world.map.width {
            let tile = &world.map.tiles[world.map.idx(x, y)];
            let explored = world.settings.reveal_map || world.is_explored(me, x, y);
            if explored {
                let mut c = art::terrain_color(tile.terrain, 1, 0);
                if !world.is_visible(me, x, y) && !world.settings.reveal_map {
                    c = c.mul(0.6);
                }
                let (px, py) = to_mm(x as f32 + step as f32 / 2.0, y as f32 + step as f32 / 2.0);
                ui.dl.poly(
                    &[
                        [px, py - half_h],
                        [px + half_w, py],
                        [px, py + half_h],
                        [px - half_w, py],
                    ],
                    c,
                );
            }
            x += step;
        }
        y += step;
    }
    for e in world.iter() {
        let (x, y) = e.pos.tile();
        if e.inside.is_some()
            || (!world.ally(me, e.owner)
                && !world.is_visible(me, x, y)
                && !world.settings.reveal_map)
        {
            continue;
        }
        if !world.is_explored(me, x, y) && !world.settings.reveal_map {
            continue;
        }
        let def = data.object(e.ty);
        let c = match def.class {
            ObjectClass::Resource => match def.yields.map(|y| y.resource) {
                Some(Resource::Gold) => Color::rgb(240, 200, 40),
                Some(Resource::Stone) => Color::rgb(190, 190, 195),
                Some(Resource::Wood) => continue,
                _ => Color::rgb(200, 60, 80),
            },
            ObjectClass::Relic => Color::WHITE,
            ObjectClass::Animal => continue,
            _ => player_color(e.owner),
        };
        let (px, py) = to_mm(e.pos.x.to_f32(), e.pos.y.to_f32());
        let s = if def.class == ObjectClass::Building {
            2.5
        } else {
            1.5
        };
        ui.dl.rect(px - s, py - s, s * 2.0, s * 2.0, c);
    }
    // Viewport rectangle.
    let corners = [
        (0.0, TOP_BAR),
        (g.iso.screen_w, TOP_BAR),
        (g.iso.screen_w, g.iso.screen_h - BOTTOM_PANEL),
        (0.0, g.iso.screen_h - BOTTOM_PANEL),
    ];
    let pts: Vec<[f32; 2]> = corners
        .iter()
        .map(|(sx, sy)| {
            let (wx, wy) = g.iso.to_world(*sx, *sy);
            let (px, py) = to_mm(wx.clamp(0.0, n), wy.clamp(0.0, n));
            [px, py]
        })
        .collect();
    ui.dl
        .poly_outline(&pts, 1.0, Color::rgba(255, 255, 255, 200));
    if let Some(a) = g.last_alert {
        let (px, py) = to_mm(a.x.to_f32(), a.y.to_f32());
        if (g.session.world().tick / 5) % 2 == 0 {
            ui.dl.circle(px, py, 5.0, Color::rgba(255, 60, 60, 200));
        }
    }
}

fn minimap_to_world(world: &World, r: Rect, m: (f32, f32)) -> (f32, f32) {
    let n = world.map.width.max(world.map.height) as f32;
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let a = (m.0 - cx) / (r.w / 2.0) * n; // x - y
    let b = (m.1 - cy) / (r.h / 2.0) * n + n; // x + y
    let x = (a + b) / 2.0;
    let y = (b - a) / 2.0;
    (
        x.clamp(0.0, world.map.width as f32 - 1.0),
        y.clamp(0.0, world.map.height as f32 - 1.0),
    )
}

/// Leave the current match: back to the editor after a test play, else the main menu.
pub fn leave_game(app: &mut App) {
    let was_test = app.game.as_ref().map(|g| g.editor_test).unwrap_or(false);
    app.game = None;
    if was_test && app.editor.is_some() {
        app.game = app.editor_game.take();
        if app.game.is_some() {
            app.screen = Screen::Editor;
            return;
        }
    }
    app.editor = None;
    app.screen = Screen::Main;
}
