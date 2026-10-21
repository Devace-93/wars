//! Rendering of the game screen: viewport maths, the glyph map, minimap and panels.

use crate::actions::{cost_str, Action};
use crate::game::{Game, Mode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use wars_client::wars_sim::ids::{PlayerId, Terrain};
use wars_client::wars_sim::types::{GameData, ObjectClass};
use wars_client::wars_sim::world::{Entity, QueueItem, Task, World};
use wars_client::wars_sim::{stats, Fixed, Vec2};
use wars_client::Session;

/// The part of the map shown in the main panel, in tiles (one cell per tile).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Viewport {
    pub x0: i32,
    pub y0: i32,
    pub w: i32,
    pub h: i32,
}

const FOLLOW_MARGIN: i32 = 3;

impl Viewport {
    pub fn resize(&mut self, w: i32, h: i32, map_w: i32, map_h: i32) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.clamp(map_w, map_h);
    }
    fn clamp(&mut self, map_w: i32, map_h: i32) {
        self.x0 = self.x0.clamp(0, (map_w - self.w).max(0));
        self.y0 = self.y0.clamp(0, (map_h - self.h).max(0));
    }
    /// Scroll so the cursor stays inside the view with a small margin.
    pub fn follow(&mut self, cx: i32, cy: i32, map_w: i32, map_h: i32) {
        let mx = FOLLOW_MARGIN.min(self.w / 2);
        let my = FOLLOW_MARGIN.min(self.h / 2);
        if cx < self.x0 + mx {
            self.x0 = cx - mx;
        } else if cx >= self.x0 + self.w - mx {
            self.x0 = cx + mx + 1 - self.w;
        }
        if cy < self.y0 + my {
            self.y0 = cy - my;
        } else if cy >= self.y0 + self.h - my {
            self.y0 = cy + my + 1 - self.h;
        }
        self.clamp(map_w, map_h);
    }
    pub fn center_on(&mut self, cx: i32, cy: i32, map_w: i32, map_h: i32) {
        self.x0 = cx - self.w / 2;
        self.y0 = cy - self.h / 2;
        self.clamp(map_w, map_h);
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x0 && y >= self.y0 && x < self.x0 + self.w && y < self.y0 + self.h
    }
    /// Tile -> cell offset inside the map area.
    pub fn cell_of(&self, x: i32, y: i32) -> Option<(u16, u16)> {
        if self.contains(x, y) {
            Some(((x - self.x0) as u16, (y - self.y0) as u16))
        } else {
            None
        }
    }
}

pub fn player_color(p: PlayerId) -> Color {
    match p.0 {
        1 => Color::Rgb(90, 140, 255),
        2 => Color::Rgb(255, 80, 80),
        3 => Color::Rgb(80, 230, 80),
        4 => Color::Rgb(255, 230, 60),
        5 => Color::Rgb(80, 230, 230),
        6 => Color::Rgb(230, 100, 230),
        7 => Color::Rgb(200, 200, 200),
        8 => Color::Rgb(255, 150, 40),
        _ => Color::White,
    }
}

pub fn terrain_bg(t: Terrain) -> Color {
    match t {
        Terrain::Grass => Color::Rgb(44, 96, 34),
        Terrain::Dirt => Color::Rgb(104, 82, 44),
        Terrain::Sand => Color::Rgb(176, 156, 86),
        Terrain::Forest => Color::Rgb(22, 62, 22),
        Terrain::Water => Color::Rgb(30, 72, 150),
        Terrain::Shallows => Color::Rgb(64, 124, 172),
        Terrain::DeepWater => Color::Rgb(16, 42, 110),
        Terrain::Snow => Color::Rgb(196, 200, 210),
        Terrain::Road => Color::Rgb(112, 102, 92),
        Terrain::Cliff => Color::Rgb(72, 62, 52),
        Terrain::Farmland => Color::Rgb(124, 102, 44),
        Terrain::Beach => Color::Rgb(192, 172, 112),
    }
}

fn terrain_glyph(t: Terrain) -> char {
    match t {
        Terrain::Water => '~',
        Terrain::DeepWater => '≈',
        Terrain::Shallows => '-',
        Terrain::Forest => '♣',
        Terrain::Cliff => '^',
        Terrain::Farmland => '"',
        _ => ' ',
    }
}

/// Darken a colour for explored-but-not-visible tiles.
pub fn dim(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r * 2 / 5, g * 2 / 5, b * 2 / 5),
        other => other,
    }
}

fn gaia_fg(def: &wars_client::wars_sim::types::ObjectType) -> Color {
    use wars_client::wars_sim::types::GatherKind;
    match def.yields.map(|y| y.kind) {
        Some(GatherKind::Wood) => Color::Rgb(120, 200, 90),
        Some(GatherKind::Gold) => Color::Rgb(255, 215, 0),
        Some(GatherKind::Stone) => Color::Rgb(220, 220, 220),
        Some(GatherKind::Berries) => Color::Rgb(255, 120, 180),
        Some(GatherKind::Fish) => Color::Rgb(160, 210, 255),
        _ => Color::Rgb(235, 235, 235),
    }
}

/// Rows taken by the top bar and the bottom panels; the map gets the rest.
pub const TOP_H: u16 = 1;
pub const BOTTOM_H: u16 = 9;
const ACTIONS_H: u16 = 6;

pub fn right_column_width(total: u16) -> u16 {
    30u16.min(total / 3).max(16)
}

pub fn draw_game(f: &mut Frame, g: &Game) {
    let area = f.area();
    let [top, middle, bottom] = Layout::vertical([
        Constraint::Length(TOP_H),
        Constraint::Min(5),
        Constraint::Length(BOTTOM_H),
    ])
    .areas(area);
    let right_w = right_column_width(area.width);
    let [map_area, right] =
        Layout::horizontal([Constraint::Min(10), Constraint::Length(right_w)]).areas(middle);
    let mini_h = (right_w / 2 + 2).min(middle.height / 2);
    let [mini_area, sel_area] =
        Layout::vertical([Constraint::Length(mini_h), Constraint::Min(3)]).areas(right);
    let [actions_area, log_area, status_area] = Layout::vertical([
        Constraint::Length(ACTIONS_H),
        Constraint::Length(BOTTOM_H - ACTIONS_H - 1),
        Constraint::Length(1),
    ])
    .areas(bottom);

    let buf = f.buffer_mut();
    draw_top_bar(buf, top, g);
    draw_map(buf, map_area, g);
    draw_minimap(buf, mini_area, g);
    draw_selection(buf, sel_area, g);
    draw_actions(buf, actions_area, g);
    draw_log(buf, log_area, g);
    draw_status(buf, status_area, g);
    match &g.mode {
        Mode::Help => draw_help(buf, area),
        Mode::ConfirmQuit => draw_confirm(buf, area, "Leave the match? (y/n)"),
        _ => {}
    }
}

fn draw_top_bar(buf: &mut Buffer, area: Rect, g: &Game) {
    let w = g.session.world();
    let me = g.session.me();
    let p = w.player(me);
    let data = g.data();
    let secs = w.seconds();
    let speed = g.session.speed_pct();
    let mut spans = vec![
        Span::styled(" F ", Style::new().fg(Color::Rgb(255, 120, 120))),
        Span::raw(format!("{:<5}", p.resources.food)),
        Span::styled("W ", Style::new().fg(Color::Rgb(140, 200, 100))),
        Span::raw(format!("{:<5}", p.resources.wood)),
        Span::styled("G ", Style::new().fg(Color::Rgb(255, 215, 0))),
        Span::raw(format!("{:<5}", p.resources.gold)),
        Span::styled("S ", Style::new().fg(Color::Rgb(210, 210, 210))),
        Span::raw(format!("{:<5}", p.resources.stone)),
        Span::raw(format!(
            "| Pop {}/{} | {} | {:02}:{:02} | {}.{:02}x",
            p.pop,
            p.pop_support.min(stats::pop_cap(w, data, me)),
            p.age.name(),
            secs / 60,
            secs % 60,
            speed / 100,
            speed % 100
        )),
    ];
    spans.push(Span::raw(format!(
        " | {} ({})",
        data.civ(p.civ).name,
        p.name
    )));
    if g.session.paused() {
        spans.push(Span::styled(
            " PAUSED",
            Style::new().fg(Color::Black).bg(Color::Yellow).bold(),
        ));
    }
    if !p.alive {
        spans.push(Span::styled(
            " DEFEATED",
            Style::new().fg(Color::White).bg(Color::Red).bold(),
        ));
    }
    if let Session::Remote(m) = &g.session {
        spans.push(Span::raw(format!(
            " | net {}ms{}",
            m.last_rtt_ms.unwrap_or(0),
            if m.transport.is_closed() {
                " DISCONNECTED"
            } else {
                ""
            }
        )));
    }
    let line = Line::from(spans).style(Style::new().bg(Color::Rgb(30, 30, 30)));
    Paragraph::new(line).render(area, buf);
}

/// Entities drawn on the map: everything visible, plus gaia resources on explored tiles
/// (trees and mines do not move, so remembering them is what a player expects).
fn drawable_entities<'a>(w: &'a World, data: &GameData, me: PlayerId) -> Vec<&'a Entity> {
    w.iter()
        .filter(|e| e.inside.is_none())
        .filter(|e| {
            let (x, y) = e.pos.tile();
            if w.ally(me, e.owner) || w.is_visible(me, x, y) {
                return true;
            }
            e.owner.is_gaia()
                && data.object(e.ty).class == ObjectClass::Resource
                && w.is_explored(me, x, y)
        })
        .collect()
}

fn draw_map(buf: &mut Buffer, area: Rect, g: &Game) {
    let w = g.session.world();
    let data = g.data();
    let me = g.session.me();
    let vp = g.vp;
    // Terrain and fog.
    for cy in 0..area.height.min(vp.h as u16) {
        for cx in 0..area.width.min(vp.w as u16) {
            let (tx, ty) = (vp.x0 + cx as i32, vp.y0 + cy as i32);
            let cell = &mut buf[(area.x + cx, area.y + cy)];
            if !w.map.in_bounds(tx, ty) {
                cell.set_char(' ').set_style(Style::new().bg(Color::Black));
                continue;
            }
            if !w.is_explored(me, tx, ty) {
                cell.set_char(' ').set_style(Style::new().bg(Color::Black));
                continue;
            }
            let t = w.map.terrain(tx, ty);
            let visible = w.is_visible(me, tx, ty);
            let bg = if visible {
                terrain_bg(t)
            } else {
                dim(terrain_bg(t))
            };
            let fg = if visible {
                Color::Rgb(150, 170, 150)
            } else {
                Color::Rgb(80, 90, 80)
            };
            cell.set_char(terrain_glyph(t))
                .set_style(Style::new().bg(bg).fg(fg));
        }
    }
    // Entities.
    for e in drawable_entities(w, data, me) {
        let def = data.object(e.ty);
        let (x, y, fw, fh) = w.footprint(data, e);
        let (ex, ey) = e.pos.tile();
        let visible = w.is_visible(me, ex, ey) || w.ally(me, e.owner);
        let selected = g.ctrl.selection.contains(&e.id);
        let mut style = if e.owner.is_gaia() {
            Style::new().fg(gaia_fg(def))
        } else {
            Style::new().fg(player_color(e.owner)).bold()
        };
        if !visible {
            style = style.fg(dim(style.fg.unwrap_or(Color::White)));
        }
        if e.build_progress < 1000 {
            style = style.remove_modifier(Modifier::BOLD).underlined();
        }
        if selected {
            style = style.bg(Color::Rgb(215, 215, 215)).fg(Color::Black);
        }
        for yy in y..y + fh {
            for xx in x..x + fw {
                if let Some((cx, cy)) = vp.cell_of(xx, yy) {
                    if cx >= area.width || cy >= area.height {
                        continue;
                    }
                    let cell = &mut buf[(area.x + cx, area.y + cy)];
                    let bg = if selected {
                        Color::Rgb(215, 215, 215)
                    } else {
                        cell.bg
                    };
                    cell.set_char(def.glyph).set_style(style.bg(bg));
                }
            }
        }
    }
    // Box-select rectangle.
    if let Mode::BoxSelect(ax, ay) = g.mode {
        let (x1, x2) = (ax.min(g.cursor.0), ax.max(g.cursor.0));
        let (y1, y2) = (ay.min(g.cursor.1), ay.max(g.cursor.1));
        for yy in y1..=y2 {
            for xx in x1..=x2 {
                if let Some((cx, cy)) = vp.cell_of(xx, yy) {
                    if cx < area.width && cy < area.height {
                        buf[(area.x + cx, area.y + cy)].set_bg(Color::Rgb(90, 90, 170));
                    }
                }
            }
        }
    }
    // Building placement preview.
    if let Mode::Placing(ty) = g.mode {
        let def = data.object(ty);
        let (fw, fh) = (def.size.0 as i32, def.size.1 as i32);
        let fits = w
            .map
            .can_place(g.cursor.0, g.cursor.1, fw, fh, def.placement);
        let bg = if fits {
            Color::Rgb(40, 170, 60)
        } else {
            Color::Rgb(190, 40, 40)
        };
        for yy in g.cursor.1..g.cursor.1 + fh {
            for xx in g.cursor.0..g.cursor.0 + fw {
                if let Some((cx, cy)) = vp.cell_of(xx, yy) {
                    if cx < area.width && cy < area.height {
                        buf[(area.x + cx, area.y + cy)]
                            .set_char(def.glyph)
                            .set_style(Style::new().fg(Color::White).bg(bg).bold());
                    }
                }
            }
        }
    }
    // Cursor.
    if let Some((cx, cy)) = vp.cell_of(g.cursor.0, g.cursor.1) {
        if cx < area.width && cy < area.height {
            let cell = &mut buf[(area.x + cx, area.y + cy)];
            if cell.symbol() == " " {
                cell.set_char('+');
            }
            cell.modifier |= Modifier::REVERSED | Modifier::BOLD;
        }
    }
}

fn draw_minimap(buf: &mut Buffer, area: Rect, g: &Game) {
    let block = Block::bordered().title(" map ");
    let inner = block.inner(area);
    block.render(area, buf);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let w = g.session.world();
    let data = g.data();
    let me = g.session.me();
    let (mw, mh) = (w.map.width, w.map.height);
    let (cw, ch) = (inner.width as i32, inner.height as i32);
    // Mark cells: (owner colour, glyph) for the most interesting entity in each block.
    let mut marks: Vec<Option<(Color, char)>> = vec![None; (cw * ch) as usize];
    for e in drawable_entities(w, data, me) {
        let def = data.object(e.ty);
        let (x, y) = e.pos.tile();
        let (cx, cy) = (x * cw / mw, y * ch / mh);
        let i = (cy * cw + cx) as usize;
        if i >= marks.len() {
            continue;
        }
        if e.owner.is_gaia() {
            use wars_client::wars_sim::types::GatherKind;
            let m = match def.yields.map(|y| y.kind) {
                Some(GatherKind::Gold) => Some((Color::Rgb(255, 215, 0), '$')),
                Some(GatherKind::Stone) => Some((Color::Rgb(220, 220, 220), '◆')),
                _ => None,
            };
            if marks[i].is_none() {
                marks[i] = m;
            }
        } else {
            let glyph = if def.class == ObjectClass::Building {
                '■'
            } else {
                '·'
            };
            let replace = match marks[i] {
                None => true,
                Some((_, c)) => c != '■',
            };
            if replace {
                marks[i] = Some((player_color(e.owner), glyph));
            }
        }
    }
    for cy in 0..ch {
        for cx in 0..cw {
            // Sample the centre tile of the block.
            let tx = ((cx * 2 + 1) * mw / (2 * cw)).clamp(0, mw - 1);
            let ty = ((cy * 2 + 1) * mh / (2 * ch)).clamp(0, mh - 1);
            let cell = &mut buf[(inner.x + cx as u16, inner.y + cy as u16)];
            if !w.is_explored(me, tx, ty) {
                cell.set_char(' ').set_style(Style::new().bg(Color::Black));
            } else {
                let bg = terrain_bg(w.map.terrain(tx, ty));
                let bg = if w.is_visible(me, tx, ty) {
                    bg
                } else {
                    dim(bg)
                };
                cell.set_char(' ').set_style(Style::new().bg(bg));
            }
            if let Some((c, ch_)) = marks[(cy * cw + cx) as usize] {
                cell.set_char(ch_).set_fg(c);
            }
            // Viewport rectangle outline.
            let vx1 = g.vp.x0 * cw / mw;
            let vx2 = ((g.vp.x0 + g.vp.w - 1) * cw / mw).min(cw - 1);
            let vy1 = g.vp.y0 * ch / mh;
            let vy2 = ((g.vp.y0 + g.vp.h - 1) * ch / mh).min(ch - 1);
            let on_x = cx == vx1 || cx == vx2;
            let on_y = cy == vy1 || cy == vy2;
            let inside = cx >= vx1 && cx <= vx2 && cy >= vy1 && cy <= vy2;
            if inside && (on_x || on_y) && cell.symbol() == " " {
                cell.set_char(if on_x && on_y {
                    '+'
                } else if on_x {
                    '|'
                } else {
                    '-'
                })
                .set_fg(Color::White);
            }
        }
    }
}

pub fn task_name(t: &Task) -> &'static str {
    match t {
        Task::Idle => "idle",
        Task::Move {
            attack_move,
            patrol_from,
            ..
        } => {
            if patrol_from.is_some() {
                "patrol"
            } else if *attack_move {
                "attack-move"
            } else {
                "move"
            }
        }
        Task::Attack { .. } => "attack",
        Task::Gather { .. } => "gather",
        Task::DropOff { .. } => "drop off",
        Task::Build { .. } => "build",
        Task::Repair { .. } => "repair",
        Task::Garrison { .. } => "garrison",
        Task::Follow { .. } => "follow",
        Task::Guard { .. } => "guard",
        Task::Convert { .. } => "convert",
        Task::Heal { .. } => "heal",
        Task::Trade { .. } => "trade",
        Task::PickRelic { .. } => "pick relic",
        Task::DeliverRelic { .. } => "deliver relic",
        Task::Packing { .. } => "packing",
    }
}

fn entity_summary(w: &World, data: &GameData, e: &Entity) -> String {
    let def = data.object(e.ty);
    let owner = if e.owner.is_gaia() {
        String::new()
    } else {
        format!(" [{}]", w.player(e.owner).name)
    };
    let extra = match def.class {
        ObjectClass::Resource | ObjectClass::Animal if def.yields.is_some() => {
            format!(
                " {} {}",
                e.resource_left,
                def.yields.unwrap().resource.name()
            )
        }
        _ => format!(" hp {}", e.hp),
    };
    format!("{}{}{}", def.name, owner, extra)
}

fn draw_selection(buf: &mut Buffer, area: Rect, g: &Game) {
    let block = Block::bordered().title(" selection ");
    let inner = block.inner(area);
    block.render(area, buf);
    let w = g.session.world();
    let data = g.data();
    let me = g.session.me();
    let mut lines: Vec<Line> = Vec::new();
    // Cursor line first: where we are and what is under it.
    let (cx, cy) = g.cursor;
    let terrain = w.map.terrain(cx, cy).name();
    let under = g
        .cursor_target()
        .and_then(|id| w.get(id))
        .map(|e| entity_summary(w, data, e))
        .unwrap_or_default();
    lines.push(Line::from(vec![
        Span::styled(format!("@{},{} ", cx, cy), Style::new().fg(Color::Yellow)),
        Span::styled(terrain, Style::new().fg(Color::DarkGray)),
        Span::raw(format!(" {under}")),
    ]));
    let sel: Vec<&Entity> = g
        .ctrl
        .selection
        .iter()
        .filter_map(|id| w.get(*id))
        .collect();
    if sel.is_empty() {
        lines.push(Line::from(Span::styled(
            "nothing selected",
            Style::new().fg(Color::DarkGray),
        )));
    } else if sel.len() > 1 {
        // Group summary by type.
        let mut counts: Vec<(String, usize)> = Vec::new();
        for e in &sel {
            let n = data.object(e.ty).name.clone();
            match counts.iter_mut().find(|(k, _)| *k == n) {
                Some((_, c)) => *c += 1,
                None => counts.push((n, 1)),
            }
        }
        lines.push(Line::from(Span::styled(
            format!("{} selected", sel.len()),
            Style::new().bold(),
        )));
        for (n, c) in counts {
            lines.push(Line::raw(format!("  {c} x {n}")));
        }
        let idle = sel.iter().filter(|e| e.is_idle()).count();
        lines.push(Line::raw(format!("  idle: {idle}")));
    } else {
        let e = sel[0];
        let def = data.object(e.ty);
        let st = stats::stats_for(w, data, e.owner, e.ty);
        lines.push(Line::from(Span::styled(
            def.name.clone(),
            Style::new().fg(player_color(e.owner)).bold(),
        )));
        lines.push(Line::raw(format!(
            "HP {}/{}  owner {}",
            e.hp,
            st.hp,
            if e.owner == me {
                "you".to_string()
            } else {
                w.player(e.owner).name.clone()
            }
        )));
        if def.class == ObjectClass::Unit {
            lines.push(Line::raw(format!(
                "task {}  stance {:?}",
                task_name(e.task()),
                e.stance
            )));
            if e.carry > 0 {
                lines.push(Line::raw(format!(
                    "carry {} {}",
                    e.carry,
                    e.carry_kind.map(|k| k.resource().name()).unwrap_or("")
                )));
            }
            if !st.attacks.is_empty() {
                lines.push(Line::raw(format!(
                    "atk {}  range {}",
                    st.attacks.iter().map(|a| a.value).max().unwrap_or(0),
                    st.range
                )));
            }
        }
        if def.class == ObjectClass::Building {
            if e.build_progress < 1000 {
                lines.push(Line::from(Span::styled(
                    format!("building {}%", e.build_progress / 10),
                    Style::new().fg(Color::Yellow),
                )));
            }
            if !e.queue.is_empty() {
                lines.push(Line::from(Span::styled(
                    "queue:",
                    Style::new().underlined(),
                )));
                for (i, q) in e.queue.iter().enumerate() {
                    let (name, total) = match q {
                        QueueItem::Unit(t) => (
                            data.object(*t).name.clone(),
                            stats::stats_for(w, data, e.owner, *t).train_ticks,
                        ),
                        QueueItem::Tech(t) => (
                            data.tech(*t).name.clone(),
                            stats::tech_cost(w, data, e.owner, *t).1,
                        ),
                    };
                    if i == 0 {
                        let pct = (e.queue_progress * 100 / total.max(1)).min(100);
                        lines.push(Line::raw(format!("  {name} {pct}%")));
                    } else {
                        lines.push(Line::raw(format!("  {name}")));
                    }
                }
            }
            if st.garrison_capacity > 0 {
                lines.push(Line::raw(format!(
                    "garrison {}/{}",
                    e.garrisoned.len(),
                    st.garrison_capacity
                )));
            }
            if let Some(r) = e.rally {
                let s = match r {
                    wars_client::wars_sim::RallyTarget::Point(p) => {
                        let (x, y) = p.tile();
                        format!("{x},{y}")
                    }
                    wars_client::wars_sim::RallyTarget::Entity(id) => w
                        .get(id)
                        .map(|t| data.object(t.ty).name.clone())
                        .unwrap_or_else(|| "?".into()),
                    wars_client::wars_sim::RallyTarget::None => "none".into(),
                };
                lines.push(Line::raw(format!("rally {s}")));
            }
        }
        if def.yields.is_some() {
            lines.push(Line::raw(format!("left {}", e.resource_left)));
        }
    }
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

fn action_spans<'a>(a: &'a Action, menu: bool) -> Vec<Span<'a>> {
    let key_style = if a.enabled {
        Style::new().fg(Color::Yellow).bold()
    } else {
        Style::new().fg(Color::DarkGray)
    };
    let label_style = if a.enabled {
        Style::new()
    } else {
        Style::new().fg(Color::DarkGray)
    };
    let mut v = vec![
        Span::styled(a.key.to_string(), key_style),
        Span::styled(format!(" {}", a.label), label_style),
    ];
    if let Some(c) = &a.cost {
        let cs = cost_str(c);
        if !cs.is_empty() {
            v.push(Span::styled(
                format!(" {cs}"),
                if a.enabled {
                    Style::new().fg(Color::Rgb(160, 160, 160))
                } else {
                    Style::new().fg(Color::DarkGray)
                },
            ));
        }
    }
    if menu && !a.note.is_empty() {
        v.push(Span::styled(
            format!(" ({})", a.note),
            Style::new().fg(Color::DarkGray),
        ));
    }
    v.push(Span::raw("   "));
    v
}

fn draw_actions(buf: &mut Buffer, area: Rect, g: &Game) {
    let (title, list, menu): (&str, &[Action], bool) = match &g.mode {
        Mode::BuildMenu => (" build (Esc cancels) ", &g.build_menu, true),
        Mode::Placing(_) => (" place: Enter confirms, Esc cancels ", &[], false),
        Mode::BoxSelect(..) => (" box select: Enter at the other corner ", &[], false),
        _ => (" actions ", &g.actions, false),
    };
    let block = Block::bordered().title(title);
    let inner = block.inner(area);
    block.render(area, buf);
    let mut spans = Vec::new();
    if list.is_empty() {
        let hint = match g.mode {
            Mode::Placing(_) | Mode::BoxSelect(..) => "move the cursor with arrows/hjkl",
            _ => "Enter selects the unit under the cursor; Tab cycles; . idle villager; , town centre; ? help",
        };
        spans.push(Span::styled(hint, Style::new().fg(Color::DarkGray)));
    }
    for a in list {
        spans.extend(action_spans(a, menu));
    }
    Paragraph::new(Line::from(spans))
        .wrap(Wrap { trim: true })
        .render(inner, buf);
}

fn draw_log(buf: &mut Buffer, area: Rect, g: &Game) {
    let n = area.height as usize;
    let lines: Vec<Line> = g
        .log
        .iter()
        .rev()
        .take(n)
        .rev()
        .map(|s| {
            Line::from(Span::styled(
                s.clone(),
                Style::new().fg(Color::Rgb(190, 190, 190)),
            ))
        })
        .collect();
    Paragraph::new(lines).render(area, buf);
}

fn draw_status(buf: &mut Buffer, area: Rect, g: &Game) {
    let line = match &g.mode {
        Mode::Command(s) => Line::from(vec![
            Span::styled(":", Style::new().fg(Color::Yellow)),
            Span::raw(s.clone()),
            Span::styled("_", Style::new().fg(Color::Yellow)),
        ]),
        Mode::GroupPrefix => Line::raw("group: press 0-9 to store the selection"),
        _ => Line::from(Span::styled(
            g.status.clone(),
            Style::new().fg(Color::Rgb(220, 220, 150)),
        )),
    };
    Paragraph::new(line)
        .style(Style::new().bg(Color::Rgb(30, 30, 30)))
        .render(area, buf);
}

pub const HELP_TEXT: &str = "\
 arrows / hjkl   move cursor (Shift: 5 tiles)   Enter/Space select (Shift adds; twice: all of that type on screen)
 v               box select (press, move, press)
 Tab             cycle own units/buildings      .  next idle villager    ,  town centre
 0-9             recall group                   Ctrl+0-9 or g then 0-9   store group
 x               context action on cursor       m move  a attack-move  s stop  i garrison  d delete
 b               build menu (letter picks, cursor places, Enter confirms, Esc cancels)
 g gather  r repair (villagers)                 letters listed in the panel train / research
 p pause   + / - speed   F5 quick save   Esc clear selection or menu   q leave match
 :save name  :load name  :chat text  :cheat code  :speed n  :buy res  :sell res  :resign  :quit  :help
 press any key to close";

fn draw_help(buf: &mut Buffer, area: Rect) {
    let lines: Vec<&str> = HELP_TEXT.lines().collect();
    let h = (lines.len() as u16 + 2).min(area.height);
    let w =
        (lines.iter().map(|l| l.chars().count()).max().unwrap_or(40) as u16 + 2).min(area.width);
    let r = Rect::new(
        area.x + (area.width.saturating_sub(w)) / 2,
        area.y + (area.height.saturating_sub(h)) / 2,
        w,
        h,
    );
    Clear.render(r, buf);
    Paragraph::new(lines.iter().map(|l| Line::raw(*l)).collect::<Vec<_>>())
        .block(Block::bordered().title(" keys "))
        .style(Style::new().bg(Color::Rgb(20, 20, 40)))
        .render(r, buf);
}

fn draw_confirm(buf: &mut Buffer, area: Rect, text: &str) {
    let w = (text.len() as u16 + 4).min(area.width);
    let r = Rect::new(
        area.x + (area.width.saturating_sub(w)) / 2,
        area.y + (area.height.saturating_sub(3)) / 2,
        w,
        3,
    );
    Clear.render(r, buf);
    Paragraph::new(Line::raw(text))
        .block(Block::bordered())
        .style(Style::new().bg(Color::Rgb(60, 20, 20)))
        .render(r, buf);
}

/// Centre of the cursor tile as a world position.
pub fn tile_center(x: i32, y: i32) -> Vec2 {
    Vec2::new(
        Fixed::from_int(x) + Fixed::HALF,
        Fixed::from_int(y) + Fixed::HALF,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_follow_scrolls_with_margin_and_clamps() {
        let mut vp = Viewport::default();
        vp.resize(40, 20, 96, 96);
        assert_eq!((vp.x0, vp.y0), (0, 0));
        vp.follow(10, 10, 96, 96);
        assert_eq!((vp.x0, vp.y0), (0, 0));
        vp.follow(39, 10, 96, 96);
        assert_eq!(vp.x0, 3); // 39 + 3 + 1 - 40
        vp.follow(95, 95, 96, 96);
        assert_eq!((vp.x0, vp.y0), (56, 76));
        vp.follow(0, 0, 96, 96);
        assert_eq!((vp.x0, vp.y0), (0, 0));
    }

    #[test]
    fn viewport_maps_tiles_to_cells() {
        let mut vp = Viewport::default();
        vp.resize(10, 5, 96, 96);
        vp.center_on(50, 50, 96, 96);
        assert_eq!((vp.x0, vp.y0), (45, 48));
        assert_eq!(vp.cell_of(45, 48), Some((0, 0)));
        assert_eq!(vp.cell_of(54, 52), Some((9, 4)));
        assert_eq!(vp.cell_of(55, 52), None);
        assert!(!vp.contains(44, 48));
    }

    #[test]
    fn viewport_larger_than_map_stays_at_origin() {
        let mut vp = Viewport::default();
        vp.resize(200, 100, 96, 96);
        vp.center_on(90, 90, 96, 96);
        assert_eq!((vp.x0, vp.y0), (0, 0));
    }

    #[test]
    fn dim_darkens_rgb() {
        assert_eq!(dim(Color::Rgb(100, 50, 10)), Color::Rgb(40, 20, 4));
        assert_eq!(dim(Color::Red), Color::Red);
    }
}
