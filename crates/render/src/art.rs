//! Procedural art: every tile, unit, building and resource is drawn from shapes, in
//! the owner's colour, with a few frames of animation. No bitmap assets anywhere.

use crate::draw::{Color, DrawList};
use crate::iso::{Iso, TILE_H, TILE_W};
use wars_sim::ids::{PlayerId, Terrain};
use wars_sim::map::Tile;
use wars_sim::types::{GameData, GatherKind, ObjectClass, Role};
use wars_sim::world::{Entity, Task, World};

pub const PLAYER_COLORS: [Color; 9] = [
    Color::rgb(140, 140, 140), // gaia
    Color::rgb(40, 90, 230),   // blue
    Color::rgb(220, 40, 40),   // red
    Color::rgb(40, 170, 60),   // green
    Color::rgb(235, 200, 30),  // yellow
    Color::rgb(40, 200, 220),  // cyan
    Color::rgb(200, 60, 200),  // purple
    Color::rgb(240, 140, 30),  // orange
    Color::rgb(120, 120, 130), // grey
];

pub fn player_color(p: PlayerId) -> Color {
    PLAYER_COLORS[(p.0 as usize) % PLAYER_COLORS.len()]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vis {
    Unexplored,
    Explored,
    Visible,
}

pub fn terrain_color(t: Terrain, variant: u8, elevation: u8) -> Color {
    let v = 1.0 + (variant as f32 - 1.5) * 0.03 + elevation as f32 * 0.04;
    let base = match t {
        Terrain::Grass => Color::rgb(92, 150, 62),
        Terrain::Dirt => Color::rgb(150, 122, 78),
        Terrain::Sand => Color::rgb(214, 196, 140),
        Terrain::Forest => Color::rgb(60, 110, 48),
        Terrain::Water => Color::rgb(46, 110, 180),
        Terrain::Shallows => Color::rgb(110, 160, 190),
        Terrain::DeepWater => Color::rgb(30, 76, 140),
        Terrain::Snow => Color::rgb(230, 236, 240),
        Terrain::Road => Color::rgb(170, 150, 110),
        Terrain::Cliff => Color::rgb(120, 100, 80),
        Terrain::Farmland => Color::rgb(140, 110, 60),
        Terrain::Beach => Color::rgb(222, 206, 160),
    };
    base.mul(v)
}

pub fn draw_tile(dl: &mut DrawList, iso: &Iso, x: i32, y: i32, tile: &Tile, vis: Vis) {
    if vis == Vis::Unexplored {
        return;
    }
    let mut c = terrain_color(tile.terrain, tile.variant, tile.elevation);
    if vis == Vis::Explored {
        c = c.mul(0.55);
    }
    let p = iso.tile_poly(x, y);
    dl.poly(&p, c);
    // Subtle grid line on the lower-right edges for the classic diamond look.
    if iso.zoom >= 0.75 {
        let edge = c.mul(0.9);
        dl.line(p[1], p[2], 1.0, edge);
        dl.line(p[2], p[3], 1.0, edge);
    }
    if tile.terrain.is_water() && vis == Vis::Visible {
        // Little wave strokes.
        let (cx, cy) = iso.to_screen(x as f32 + 0.5, y as f32 + 0.5);
        let k = ((x * 7 + y * 13) % 5) as f32;
        let w = 6.0 * iso.zoom;
        dl.line(
            [cx - w + k, cy + k * 0.5],
            [cx + w * 0.3 + k, cy + k * 0.5],
            1.0,
            c.mul(1.25),
        );
    }
}

/// Selection/HP-bar helper.
fn hp_bar(dl: &mut DrawList, x: f32, y: f32, w: f32, frac: f32, zoom: f32) {
    let h = (3.0 * zoom).max(2.0);
    dl.rect(x, y, w, h, Color::rgba(0, 0, 0, 160));
    let c = if frac > 0.6 {
        Color::rgb(60, 200, 60)
    } else if frac > 0.3 {
        Color::rgb(230, 200, 40)
    } else {
        Color::rgb(220, 50, 40)
    };
    dl.rect(x, y, w * frac.clamp(0.0, 1.0), h, c);
}

/// Direction unit vector on screen for the 8 facings (0 = +x world).
fn facing_vec(f: u8) -> (f32, f32) {
    // World dirs: 0:(1,0) 1:(1,1) 2:(0,1) 3:(-1,1) 4:(-1,0) 5:(-1,-1) 6:(0,-1) 7:(1,-1)
    let (wx, wy): (f32, f32) = match f % 8 {
        0 => (1.0, 0.0),
        1 => (1.0, 1.0),
        2 => (0.0, 1.0),
        3 => (-1.0, 1.0),
        4 => (-1.0, 0.0),
        5 => (-1.0, -1.0),
        6 => (0.0, -1.0),
        _ => (1.0, -1.0),
    };
    let sx = (wx - wy) * 0.5;
    let sy = (wx + wy) * 0.25;
    let l = (sx * sx + sy * sy).sqrt().max(0.001);
    (sx / l, sy / l)
}

pub struct EntityDraw<'a> {
    pub world: &'a World,
    pub data: &'a GameData,
    pub tick: u32,
    pub selected: bool,
    pub vis: Vis,
    pub viewer: PlayerId,
}

pub fn draw_entity(dl: &mut DrawList, iso: &Iso, e: &Entity, ctx: &EntityDraw) {
    let def = ctx.data.object(e.ty);
    let z = iso.zoom;
    let team = player_color(e.owner);
    let dim = ctx.vis == Vis::Explored;
    let shade = |c: Color| if dim { c.mul(0.5) } else { c };
    match def.class {
        ObjectClass::Building => draw_building(dl, iso, e, ctx, shade),
        ObjectClass::Resource => draw_resource(dl, iso, e, ctx, shade),
        ObjectClass::Relic => {
            let (sx, sy) = iso.to_screen(e.pos.x.to_f32(), e.pos.y.to_f32());
            dl.ellipse(sx, sy + 2.0 * z, 7.0 * z, 3.5 * z, Color::rgba(0, 0, 0, 90));
            dl.rect(
                sx - 5.0 * z,
                sy - 8.0 * z,
                10.0 * z,
                8.0 * z,
                shade(Color::rgb(210, 170, 40)),
            );
            dl.rect(
                sx - 1.5 * z,
                sy - 13.0 * z,
                3.0 * z,
                6.0 * z,
                shade(Color::rgb(240, 220, 120)),
            );
            dl.rect(
                sx - 4.0 * z,
                sy - 11.0 * z,
                8.0 * z,
                2.0 * z,
                shade(Color::rgb(240, 220, 120)),
            );
        }
        ObjectClass::Unit | ObjectClass::Animal => {
            if dim {
                return;
            }
            draw_unit(dl, iso, e, ctx, team);
        }
    }
}

fn draw_unit(dl: &mut DrawList, iso: &Iso, e: &Entity, ctx: &EntityDraw, team: Color) {
    let def = ctx.data.object(e.ty);
    let z = iso.zoom;
    let (sx, sy) = iso.to_screen(e.pos.x.to_f32(), e.pos.y.to_f32());
    let walking = e.anim == 1 && ctx.tick.saturating_sub(e.anim_tick) < 3;
    let working = e.anim == 3 && ctx.tick.saturating_sub(e.anim_tick) < 3;
    let attacking = e.anim == 2 && ctx.tick.saturating_sub(e.anim_tick) < 8;
    let bob = if walking {
        ((ctx.tick / 3) % 2) as f32 * 1.5 * z
    } else {
        0.0
    };
    let (fx, fy) = facing_vec(e.facing);
    let lunge = if attacking { 3.0 * z } else { 0.0 };
    let (bx, by) = (sx + fx * lunge, sy + fy * lunge - bob);
    // Selection ring and shadow.
    if ctx.selected {
        dl.ellipse(
            sx,
            sy + 1.0,
            11.0 * z,
            5.5 * z,
            Color::rgba(255, 255, 255, 60),
        );
        let ring = if ctx.world.ally(ctx.viewer, e.owner) {
            Color::rgb(80, 240, 80)
        } else {
            Color::rgb(240, 70, 70)
        };
        for i in 0..16 {
            let a0 = i as f32 / 16.0 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / 16.0 * std::f32::consts::TAU;
            dl.line(
                [sx + 11.0 * z * a0.cos(), sy + 5.5 * z * a0.sin()],
                [sx + 11.0 * z * a1.cos(), sy + 5.5 * z * a1.sin()],
                1.5,
                ring,
            );
        }
    }
    dl.ellipse(sx, sy + 1.0, 7.0 * z, 3.5 * z, Color::rgba(0, 0, 0, 70));
    let skin = Color::rgb(232, 190, 150);
    let dark = Color::rgb(60, 50, 40);
    match def.class {
        ObjectClass::Animal => {
            let (body, size) = match def.key.as_str() {
                "sheep" | "turkey" => (Color::rgb(240, 240, 235), 5.0),
                "boar" => (Color::rgb(90, 60, 40), 6.5),
                "wolf" => (Color::rgb(120, 120, 125), 5.5),
                _ => (Color::rgb(190, 150, 100), 5.0),
            };
            if e.alerted && e.decay.is_some() {
                // Carcass.
                dl.ellipse(
                    bx,
                    by - 2.0 * z,
                    size * 1.2 * z,
                    size * 0.6 * z,
                    body.mul(0.8),
                );
                return;
            }
            dl.ellipse(bx, by - 4.0 * z, size * z, size * 0.6 * z, body);
            dl.circle(
                bx + fx * size * 0.9 * z,
                by - 5.0 * z + fy * 2.0 * z,
                size * 0.4 * z,
                body.mul(0.9),
            );
            if def.key == "deer" {
                dl.line(
                    [bx + fx * 4.0 * z, by - 8.0 * z],
                    [bx + fx * 4.0 * z - 2.0 * z, by - 12.0 * z],
                    1.0,
                    dark,
                );
                dl.line(
                    [bx + fx * 4.0 * z, by - 8.0 * z],
                    [bx + fx * 4.0 * z + 2.0 * z, by - 12.0 * z],
                    1.0,
                    dark,
                );
            }
            for i in 0..2 {
                let lx = bx - 2.0 * z + i as f32 * 4.0 * z;
                let swing = if walking {
                    ((ctx.tick / 2 + i) % 2) as f32 * 1.5 * z
                } else {
                    0.0
                };
                dl.line(
                    [lx, by - 2.0 * z],
                    [lx + swing, by + 1.0 * z],
                    1.5 * z,
                    body.mul(0.7),
                );
            }
        }
        _ if def.is_naval => {
            // Hull + mast + sail in team colour.
            let hull = Color::rgb(110, 75, 45);
            let w = (10.0 + def.size.0 as f32 * 4.0) * z;
            dl.poly(
                &[
                    [bx - w, by - 3.0 * z],
                    [bx + w, by - 3.0 * z],
                    [bx + w * 0.7, by + 3.0 * z],
                    [bx - w * 0.7, by + 3.0 * z],
                ],
                hull,
            );
            if def.role != Role::Transport || true {
                dl.rect(bx - 1.0 * z, by - 20.0 * z, 2.0 * z, 17.0 * z, dark);
                let sail = if def.role == Role::Fishing {
                    Color::rgb(240, 235, 220)
                } else {
                    team
                };
                dl.tri(
                    [bx + 1.0 * z, by - 19.0 * z],
                    [bx + 10.0 * z, by - 9.0 * z],
                    [bx + 1.0 * z, by - 7.0 * z],
                    sail,
                );
            }
            if def.role == Role::Naval && !def.attacks.is_empty() {
                dl.rect(bx - 6.0 * z, by - 7.0 * z, 12.0 * z, 3.0 * z, team.mul(0.8));
            }
        }
        _ if def.role == Role::Siege => {
            let wood = Color::rgb(130, 95, 55);
            let w = 9.0 * z;
            dl.rect(bx - w, by - 8.0 * z, 2.0 * w, 6.0 * z, wood);
            dl.circle(bx - w * 0.7, by - 1.0 * z, 3.0 * z, dark);
            dl.circle(bx + w * 0.7, by - 1.0 * z, 3.0 * z, dark);
            dl.rect(bx - 3.0 * z, by - 11.0 * z, 6.0 * z, 3.0 * z, team);
            if def.key.contains("ram") {
                dl.poly(
                    &[
                        [bx - w, by - 8.0 * z],
                        [bx + w, by - 8.0 * z],
                        [bx, by - 15.0 * z],
                    ],
                    wood.mul(0.8),
                );
            } else if def.key == "trebuchet" {
                let arm = if e.packed { 6.0 } else { 18.0 };
                dl.line(
                    [bx, by - 8.0 * z],
                    [bx + fx * arm * z, by - 8.0 * z - arm * 0.8 * z],
                    2.0 * z,
                    wood.mul(0.8),
                );
            } else if def.key.contains("bombard") || def.key == "cobra_car" {
                dl.line(
                    [bx, by - 7.0 * z],
                    [bx + fx * 12.0 * z, by - 9.0 * z + fy * 4.0 * z],
                    3.0 * z,
                    dark,
                );
            } else {
                dl.line(
                    [bx, by - 9.0 * z],
                    [bx + fx * 9.0 * z, by - 13.0 * z + fy * 3.0 * z],
                    2.0 * z,
                    wood.mul(0.8),
                );
            }
        }
        _ => {
            let mounted = matches!(def.role, Role::Cavalry | Role::Scout | Role::CavalryArcher);
            let elephant = def.key.contains("elephant");
            let mut head_y = by - 12.0 * z;
            if elephant {
                dl.ellipse(
                    bx,
                    by - 7.0 * z,
                    11.0 * z,
                    7.0 * z,
                    Color::rgb(130, 125, 125),
                );
                dl.circle(
                    bx + fx * 9.0 * z,
                    by - 9.0 * z,
                    4.5 * z,
                    Color::rgb(130, 125, 125),
                );
                dl.line(
                    [bx + fx * 12.0 * z, by - 8.0 * z],
                    [bx + fx * 15.0 * z, by - 2.0 * z],
                    2.0 * z,
                    Color::rgb(120, 115, 115),
                );
                head_y = by - 20.0 * z;
                dl.rect(bx - 4.0 * z, by - 18.0 * z, 8.0 * z, 6.0 * z, team);
            } else if mounted {
                let horse = if def.key.contains("camel") || def.key.contains("mameluke") {
                    Color::rgb(190, 160, 100)
                } else {
                    Color::rgb(110, 75, 45)
                };
                dl.ellipse(bx, by - 6.0 * z, 9.0 * z, 4.5 * z, horse);
                dl.circle(bx + fx * 8.0 * z, by - 9.0 * z + fy * 1.0, 3.0 * z, horse);
                for i in 0..2 {
                    let lx = bx - 4.0 * z + i as f32 * 8.0 * z;
                    let swing = if walking {
                        ((ctx.tick / 2 + i) % 2) as f32 * 2.0 * z
                    } else {
                        0.0
                    };
                    dl.line(
                        [lx, by - 4.0 * z],
                        [lx + swing, by + 1.0],
                        1.6 * z,
                        horse.mul(0.7),
                    );
                }
                head_y = by - 17.0 * z;
                dl.rect(bx - 3.5 * z, by - 15.0 * z, 7.0 * z, 8.0 * z, team);
            } else {
                // Legs, body, head.
                for i in 0..2 {
                    let lx = bx - 2.0 * z + i as f32 * 4.0 * z;
                    let swing = if walking {
                        ((ctx.tick / 2 + i) % 2) as f32 * 2.0 * z - 1.0 * z
                    } else {
                        0.0
                    };
                    dl.line([lx, by - 6.0 * z], [lx + swing, by], 2.0 * z, dark);
                }
                let body = if def.role == Role::Monk {
                    Color::rgb(245, 240, 225)
                } else if def.is_villager {
                    team.mix(Color::rgb(200, 180, 140), 0.35)
                } else {
                    team
                };
                dl.rect(bx - 4.0 * z, by - 13.0 * z, 8.0 * z, 8.0 * z, body);
                if def.role == Role::Monk {
                    dl.rect(bx - 1.0 * z, by - 12.0 * z, 2.0 * z, 6.0 * z, team);
                    dl.rect(bx - 2.5 * z, by - 10.5 * z, 5.0 * z, 2.0 * z, team);
                }
            }
            // Head.
            dl.circle(bx, head_y, 3.5 * z, skin);
            if !def.is_villager && def.role != Role::Monk {
                // Helmet.
                dl.rect(
                    bx - 3.5 * z,
                    head_y - 4.0 * z,
                    7.0 * z,
                    2.5 * z,
                    Color::rgb(150, 150, 160),
                );
            } else if def.is_villager {
                dl.rect(
                    bx - 3.5 * z,
                    head_y - 4.0 * z,
                    7.0 * z,
                    2.0 * z,
                    Color::rgb(120, 85, 50),
                );
            }
            // Weapon / tool.
            let hand = (bx + fx * 5.0 * z, head_y + 5.0 * z);
            let swing = if attacking || working { 4.0 * z } else { 0.0 };
            match def.role {
                Role::Archer | Role::CavalryArcher => {
                    // Bow arc.
                    let (hx, hy) = hand;
                    for i in 0..5 {
                        let a0 = -0.9 + i as f32 * 0.45;
                        let a1 = a0 + 0.45;
                        dl.line(
                            [
                                hx + a0.cos() * 6.0 * z * fx.signum(),
                                hy + a0.sin() * 6.0 * z,
                            ],
                            [
                                hx + a1.cos() * 6.0 * z * fx.signum(),
                                hy + a1.sin() * 6.0 * z,
                            ],
                            1.2 * z,
                            Color::rgb(120, 80, 40),
                        );
                    }
                }
                Role::Villager => {
                    let tool = match e.carry_kind {
                        Some(GatherKind::Wood) => Color::rgb(160, 160, 170),
                        Some(GatherKind::Gold) | Some(GatherKind::Stone) => {
                            Color::rgb(140, 140, 150)
                        }
                        _ => Color::rgb(150, 110, 70),
                    };
                    dl.line(
                        [hand.0, hand.1],
                        [
                            hand.0 + fx * 6.0 * z + swing * fx,
                            hand.1 - 7.0 * z + fy * 2.0 * z,
                        ],
                        1.5 * z,
                        tool,
                    );
                    if e.carry > 0 {
                        let c = match e.carry_kind.map(|k| k.resource()) {
                            Some(wars_sim::ids::Resource::Food) => Color::rgb(220, 60, 60),
                            Some(wars_sim::ids::Resource::Wood) => Color::rgb(140, 90, 40),
                            Some(wars_sim::ids::Resource::Gold) => Color::rgb(240, 200, 40),
                            _ => Color::rgb(150, 150, 150),
                        };
                        dl.rect(
                            bx - fx * 5.0 * z - 2.0 * z,
                            by - 11.0 * z,
                            4.0 * z,
                            4.0 * z,
                            c,
                        );
                    }
                }
                Role::Monk | Role::Trade | Role::King => {
                    if def.role == Role::Trade {
                        dl.rect(
                            bx - 9.0 * z,
                            by - 8.0 * z,
                            7.0 * z,
                            5.0 * z,
                            Color::rgb(130, 95, 55),
                        );
                        dl.circle(bx - 6.0 * z, by - 2.0 * z, 2.5 * z, dark);
                    }
                    if def.role == Role::King {
                        dl.tri(
                            [bx - 4.0 * z, head_y - 3.0 * z],
                            [bx + 4.0 * z, head_y - 3.0 * z],
                            [bx, head_y - 9.0 * z],
                            Color::rgb(240, 200, 40),
                        );
                    }
                }
                _ => {
                    // Sword / spear + shield.
                    let spear = def.key.contains("spear")
                        || def.key.contains("pike")
                        || def.key.contains("halberd")
                        || def.key == "samurai";
                    let len = if spear { 12.0 } else { 7.0 };
                    dl.line(
                        [hand.0, hand.1],
                        [
                            hand.0 + fx * (len * z + swing),
                            hand.1 - 8.0 * z + fy * 3.0 * z,
                        ],
                        1.5 * z,
                        Color::rgb(200, 200, 210),
                    );
                    if !spear && !mounted {
                        dl.circle(bx - fx * 4.5 * z, head_y + 5.0 * z, 3.0 * z, team.mul(0.8));
                    }
                }
            }
        }
    }
    // HP bar when hurt or selected.
    let max = wars_sim::stats::stats_for(ctx.world, ctx.data, e.owner, e.ty).hp;
    if (e.hp < max || ctx.selected) && def.class == ObjectClass::Unit {
        hp_bar(
            dl,
            sx - 8.0 * z,
            sy - 24.0 * z,
            16.0 * z,
            e.hp as f32 / max.max(1) as f32,
            z,
        );
    }
    if e.relics > 0 {
        dl.rect(
            bx - 2.0 * z,
            by - 22.0 * z,
            4.0 * z,
            4.0 * z,
            Color::rgb(240, 210, 60),
        );
    }
}

fn draw_resource(
    dl: &mut DrawList,
    iso: &Iso,
    e: &Entity,
    ctx: &EntityDraw,
    shade: impl Fn(Color) -> Color,
) {
    let def = ctx.data.object(e.ty);
    let z = iso.zoom;
    let (sx, sy) = iso.to_screen(e.pos.x.to_f32(), e.pos.y.to_f32());
    let seed = e.id.index.wrapping_mul(2654435761) % 1000;
    let jitter = (seed % 7) as f32 - 3.0;
    match def.key.as_str() {
        "tree" => {
            let trunk = shade(Color::rgb(110, 75, 40));
            let leaf = shade(Color::rgb(44, 120, 40).mul(0.9 + (seed % 5) as f32 * 0.05));
            dl.ellipse(sx, sy, 9.0 * z, 4.0 * z, Color::rgba(0, 0, 0, 60));
            dl.rect(sx - 2.0 * z, sy - 14.0 * z, 4.0 * z, 14.0 * z, trunk);
            dl.circle(sx + jitter * 0.3 * z, sy - 20.0 * z, 11.0 * z, leaf);
            dl.circle(sx - 6.0 * z, sy - 14.0 * z, 7.0 * z, leaf.mul(0.9));
            dl.circle(sx + 6.0 * z, sy - 15.0 * z, 7.0 * z, leaf.mul(1.05));
        }
        "pine" => {
            let trunk = shade(Color::rgb(100, 65, 35));
            let leaf = shade(Color::rgb(30, 95, 50));
            dl.rect(sx - 2.0 * z, sy - 10.0 * z, 4.0 * z, 10.0 * z, trunk);
            dl.tri(
                [sx - 12.0 * z, sy - 8.0 * z],
                [sx + 12.0 * z, sy - 8.0 * z],
                [sx, sy - 24.0 * z],
                leaf,
            );
            dl.tri(
                [sx - 9.0 * z, sy - 17.0 * z],
                [sx + 9.0 * z, sy - 17.0 * z],
                [sx, sy - 32.0 * z],
                leaf.mul(1.1),
            );
        }
        "gold_mine" | "stone_mine" => {
            let base = if def.key == "gold_mine" {
                Color::rgb(225, 180, 40)
            } else {
                Color::rgb(150, 150, 155)
            };
            let frac = (e.resource_left as f32 / def.yields.map(|y| y.amount).unwrap_or(1) as f32)
                .clamp(0.3, 1.0);
            let p = iso.tile_poly(e.pos.x.to_int(), e.pos.y.to_int());
            dl.poly(&p, shade(base.mul(0.55)));
            for i in 0..4 {
                let ox = ((seed + i * 97) % 20) as f32 - 10.0;
                let oy = ((seed + i * 41) % 10) as f32 - 5.0;
                dl.circle(
                    sx + ox * z * 0.8,
                    sy + oy * z * 0.8 - 3.0 * z,
                    (5.0 + (i % 2) as f32 * 2.0) * z * frac,
                    shade(base.mul(0.85 + (i as f32) * 0.07)),
                );
            }
        }
        "berry_bush" => {
            dl.circle(sx, sy - 5.0 * z, 9.0 * z, shade(Color::rgb(60, 130, 50)));
            for i in 0..5 {
                let a = i as f32 * 1.3 + seed as f32;
                dl.circle(
                    sx + a.cos() * 5.0 * z,
                    sy - 6.0 * z + a.sin() * 3.0 * z,
                    1.6 * z,
                    shade(Color::rgb(210, 40, 70)),
                );
            }
        }
        "fish" | "shore_fish" => {
            let c = shade(Color::rgb(200, 220, 240));
            for i in 0..3 {
                let ox = (i as f32 - 1.0) * 6.0 * z;
                let phase = ((ctx.tick / 8 + i as u32) % 3) as f32;
                dl.line(
                    [sx + ox - 4.0 * z, sy + phase],
                    [sx + ox + 4.0 * z, sy - 2.0 * z + phase],
                    1.2,
                    c,
                );
            }
        }
        _ => {
            dl.circle(sx, sy, 6.0 * z, shade(Color::rgb(120, 120, 120)));
        }
    }
}

fn draw_building(
    dl: &mut DrawList,
    iso: &Iso,
    e: &Entity,
    ctx: &EntityDraw,
    shade: impl Fn(Color) -> Color,
) {
    let def = ctx.data.object(e.ty);
    let z = iso.zoom;
    let team = player_color(e.owner);
    let (tx, ty, w, h) = ctx.world.footprint(ctx.data, e);
    let age = ctx.world.player(e.owner).age;
    let foundation = e.build_progress < 1000;
    // Ground corners: top (x,y), right (x+w,y), bottom (x+w,y+h), left (x,y+h).
    let top = iso.to_screen(tx as f32, ty as f32);
    let right = iso.to_screen((tx + w) as f32, ty as f32);
    let bottom = iso.to_screen((tx + w) as f32, (ty + h) as f32);
    let left = iso.to_screen(tx as f32, (ty + h) as f32);
    let base_poly = [
        [top.0, top.1],
        [right.0, right.1],
        [bottom.0, bottom.1],
        [left.0, left.1],
    ];
    let key = def.key.as_str();
    // Wall colour by age and type.
    let (wall, roof) = match key {
        "castle" | "stone_wall" | "fortified_wall" | "gate" | "watch_tower" | "guard_tower"
        | "keep" | "bombard_tower" => (Color::rgb(165, 160, 150), Color::rgb(120, 115, 110)),
        "palisade_wall" | "outpost" => (Color::rgb(140, 100, 60), Color::rgb(110, 80, 45)),
        "farm" => (Color::rgb(150, 115, 60), Color::rgb(120, 140, 60)),
        "dock" => (Color::rgb(120, 85, 50), Color::rgb(100, 70, 40)),
        "wonder" => (Color::rgb(235, 225, 200), Color::rgb(230, 190, 70)),
        "monastery" => (Color::rgb(230, 225, 210), Color::rgb(140, 70, 60)),
        "market" => (Color::rgb(210, 190, 150), Color::rgb(190, 60, 50)),
        _ => match age {
            wars_sim::ids::Age::Dark => (Color::rgb(170, 130, 85), Color::rgb(160, 140, 90)),
            wars_sim::ids::Age::Feudal => (Color::rgb(190, 170, 130), Color::rgb(130, 90, 60)),
            wars_sim::ids::Age::Castle => (Color::rgb(205, 195, 170), Color::rgb(130, 60, 50)),
            wars_sim::ids::Age::Imperial => (Color::rgb(220, 215, 200), Color::rgb(90, 90, 120)),
        },
    };
    let wall = shade(wall);
    let roof = shade(roof);
    if ctx.selected {
        dl.poly_outline(&base_poly, 2.0, Color::rgb(255, 255, 255));
    }
    // Flat buildings: farm, walls.
    if key == "farm" {
        dl.poly(&base_poly, shade(Color::rgb(150, 118, 70)));
        for i in 1..6 {
            let t = i as f32 / 6.0;
            let a = [top.0 + (left.0 - top.0) * t, top.1 + (left.1 - top.1) * t];
            let b = [
                right.0 + (bottom.0 - right.0) * t,
                right.1 + (bottom.1 - right.1) * t,
            ];
            dl.line(a, b, 1.5 * z, shade(Color::rgb(120, 150, 60)));
        }
        if foundation {
            dl.poly(&base_poly, Color::rgba(0, 0, 0, 120));
        }
        return;
    }
    let height = match key {
        "palisade_wall" => 10.0,
        "stone_wall" | "fortified_wall" | "gate" => 16.0,
        "house" | "mill" | "lumber_camp" | "mining_camp" => 22.0,
        "outpost" => 34.0,
        "watch_tower" | "guard_tower" | "keep" | "bombard_tower" => 46.0,
        "castle" => 44.0,
        "wonder" => 60.0,
        "town_center" => 34.0,
        _ => 28.0,
    } * z;
    let progress = if foundation {
        (e.build_progress as f32 / 1000.0).max(0.08)
    } else {
        1.0
    };
    let hgt = height * progress;
    let lift = |p: (f32, f32)| [p.0, p.1 - hgt];
    // Side faces (left face uses left-bottom edge, right face uses bottom-right edge).
    dl.poly(
        &[
            [left.0, left.1],
            [bottom.0, bottom.1],
            lift(bottom),
            lift(left),
        ],
        wall.mul(0.78),
    );
    dl.poly(
        &[
            [bottom.0, bottom.1],
            [right.0, right.1],
            lift(right),
            lift(bottom),
        ],
        wall.mul(0.92),
    );
    // Roof / top.
    let top_poly = [lift(top), lift(right), lift(bottom), lift(left)];
    if foundation {
        dl.poly(&top_poly, wall.mul(0.6));
        dl.poly_outline(&base_poly, 1.5, Color::rgba(255, 255, 255, 120));
        // Progress bar.
        let cx = (top.0 + bottom.0) / 2.0;
        hp_bar(
            dl,
            cx - 16.0 * z,
            bottom.1 + 2.0,
            32.0 * z,
            e.build_progress as f32 / 1000.0,
            z,
        );
        return;
    }
    match key {
        "castle" | "keep" | "watch_tower" | "guard_tower" | "bombard_tower" | "stone_wall"
        | "fortified_wall" | "gate" => {
            dl.poly(&top_poly, roof.mul(1.15));
            // Crenellations along the top edges.
            let n = (w.max(h) * 2) as usize;
            for i in 0..n {
                let t = i as f32 / n as f32;
                let a = [
                    top_poly[0][0] + (top_poly[1][0] - top_poly[0][0]) * t,
                    top_poly[0][1] + (top_poly[1][1] - top_poly[0][1]) * t,
                ];
                if i % 2 == 0 {
                    dl.rect(a[0] - 1.5 * z, a[1] - 4.0 * z, 3.0 * z, 4.0 * z, wall);
                }
                let b = [
                    top_poly[3][0] + (top_poly[2][0] - top_poly[3][0]) * t,
                    top_poly[3][1] + (top_poly[2][1] - top_poly[3][1]) * t,
                ];
                if i % 2 == 0 {
                    dl.rect(b[0] - 1.5 * z, b[1] - 4.0 * z, 3.0 * z, 4.0 * z, wall);
                }
            }
            if key == "gate" {
                dl.rect(
                    bottom.0 - 4.0 * z,
                    bottom.1 - 12.0 * z,
                    8.0 * z,
                    12.0 * z,
                    shade(Color::rgb(80, 55, 30)),
                );
            }
        }
        "wonder" => {
            dl.poly(&top_poly, roof);
            let cx = (top_poly[0][0] + top_poly[2][0]) / 2.0;
            let cy = (top_poly[0][1] + top_poly[2][1]) / 2.0;
            dl.circle(cx, cy - 10.0 * z, 16.0 * z, roof.mul(1.1));
            dl.rect(
                cx - 2.0 * z,
                cy - 40.0 * z,
                4.0 * z,
                28.0 * z,
                shade(Color::rgb(240, 230, 200)),
            );
            dl.circle(cx, cy - 42.0 * z, 5.0 * z, shade(Color::rgb(250, 240, 120)));
        }
        "dock" => {
            dl.poly(&top_poly, roof);
            dl.rect(
                bottom.0 - 12.0 * z,
                bottom.1 - hgt - 2.0 * z,
                24.0 * z,
                3.0 * z,
                wall.mul(1.1),
            );
        }
        "monastery" => {
            dl.poly(&top_poly, roof);
            let cx = (top_poly[0][0] + top_poly[2][0]) / 2.0;
            let cy = (top_poly[0][1] + top_poly[2][1]) / 2.0;
            dl.rect(
                cx - 1.5 * z,
                cy - 20.0 * z,
                3.0 * z,
                14.0 * z,
                shade(Color::rgb(250, 240, 200)),
            );
            dl.rect(
                cx - 5.0 * z,
                cy - 16.0 * z,
                10.0 * z,
                3.0 * z,
                shade(Color::rgb(250, 240, 200)),
            );
        }
        _ => {
            // Pitched roof: ridge line from the lifted top to the lifted bottom.
            let ridge_h = 10.0 * z * (w as f32).sqrt() * 0.7;
            let ridge_a = [
                (top_poly[0][0] + top_poly[3][0]) / 2.0,
                (top_poly[0][1] + top_poly[3][1]) / 2.0 - ridge_h,
            ];
            let ridge_b = [
                (top_poly[1][0] + top_poly[2][0]) / 2.0,
                (top_poly[1][1] + top_poly[2][1]) / 2.0 - ridge_h,
            ];
            dl.poly(&[top_poly[0], top_poly[1], ridge_b, ridge_a], roof.mul(1.1));
            dl.poly(
                &[top_poly[3], top_poly[2], ridge_b, ridge_a],
                roof.mul(0.85),
            );
            dl.tri(top_poly[1], top_poly[2], ridge_b, wall.mul(0.95));
            if key == "town_center" {
                // Central tower.
                let cx = (top_poly[0][0] + top_poly[2][0]) / 2.0;
                let cy = (top_poly[0][1] + top_poly[2][1]) / 2.0;
                dl.rect(cx - 6.0 * z, cy - 26.0 * z, 12.0 * z, 20.0 * z, wall);
                dl.tri(
                    [cx - 8.0 * z, cy - 26.0 * z],
                    [cx + 8.0 * z, cy - 26.0 * z],
                    [cx, cy - 36.0 * z],
                    roof,
                );
            }
            if key == "mill" {
                let cx = (top_poly[0][0] + top_poly[2][0]) / 2.0;
                let cy = (top_poly[0][1] + top_poly[2][1]) / 2.0;
                let a = ctx.tick as f32 * 0.05;
                for i in 0..4 {
                    let ang = a + i as f32 * std::f32::consts::FRAC_PI_2;
                    dl.line(
                        [cx, cy - 14.0 * z],
                        [
                            cx + ang.cos() * 12.0 * z,
                            cy - 14.0 * z + ang.sin() * 12.0 * z,
                        ],
                        2.0 * z,
                        shade(Color::rgb(240, 235, 220)),
                    );
                }
            }
            if key == "blacksmith" || key == "siege_workshop" || key == "university" {
                let cx = (top_poly[0][0] + top_poly[2][0]) / 2.0;
                let cy = (top_poly[0][1] + top_poly[2][1]) / 2.0;
                dl.rect(
                    cx + 6.0 * z,
                    cy - 22.0 * z,
                    4.0 * z,
                    10.0 * z,
                    shade(Color::rgb(90, 80, 75)),
                );
            }
        }
    }
    // Door on the bottom-right face and a team flag.
    dl.rect(
        bottom.0 + 4.0 * z,
        bottom.1 - 8.0 * z - (right.1 - bottom.1).abs() * 0.0,
        5.0 * z,
        8.0 * z,
        shade(Color::rgb(70, 45, 25)),
    );
    if !e.owner.is_gaia()
        && !matches!(
            key,
            "farm" | "palisade_wall" | "stone_wall" | "fortified_wall"
        )
    {
        let fx = top.0;
        let fy = top.1 - hgt;
        dl.rect(
            fx - 1.0,
            fy - 16.0 * z,
            2.0,
            16.0 * z,
            shade(Color::rgb(60, 50, 40)),
        );
        dl.tri(
            [fx + 1.0, fy - 16.0 * z],
            [fx + 9.0 * z, fy - 13.0 * z],
            [fx + 1.0, fy - 10.0 * z],
            shade(team),
        );
    }
    // HP bar when damaged or selected; queue progress when selected.
    let max = wars_sim::stats::stats_for(ctx.world, ctx.data, e.owner, e.ty).hp;
    if e.hp < max || ctx.selected {
        let cx = (top.0 + bottom.0) / 2.0;
        hp_bar(
            dl,
            cx - 16.0 * z,
            bottom.1 + 2.0,
            32.0 * z,
            e.hp as f32 / max.max(1) as f32,
            z,
        );
    }
    // Garrison count.
    if !e.garrisoned.is_empty() && ctx.world.ally(ctx.viewer, e.owner) {
        let cx = (top.0 + bottom.0) / 2.0;
        dl.rect(
            cx - 6.0 * z,
            top.1 - hgt - 6.0 * z,
            12.0 * z,
            5.0 * z,
            Color::rgba(0, 0, 0, 150),
        );
        for i in 0..e.garrisoned.len().min(10) {
            dl.rect(
                cx - 5.0 * z + i as f32 * 1.1 * z,
                top.1 - hgt - 5.0 * z,
                0.8 * z,
                3.0 * z,
                Color::rgb(240, 240, 240),
            );
        }
    }
    let _ = (Task::Idle, TILE_W, TILE_H);
}
