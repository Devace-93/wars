//! Military: production, upgrades, scouting, defence, garrisoning and the attack cycle.

use crate::economy::{civ_style, Style};
use crate::view::{dist2, isqrt, tiles, within, View};
use crate::{AiMemory, Ctx, THINK_INTERVAL};
use wars_sim::command::Command;
use wars_sim::fixed::{Fixed, Vec2, ONE};
use wars_sim::ids::*;
use wars_sim::stats;
use wars_sim::types::{Effect, GameData, GatherKind, Role};
use wars_sim::world::{Entity, QueueItem, Task};

/// Enemies closer than this to a building trigger the defence.
const DEFEND_RADIUS: i32 = 12;
/// Enemies closer than this to the town centre send the villagers inside.
const GARRISON_RADIUS: i32 = 8;
/// Distance from the town centre of the army's holding point.
const RALLY_DIST: i32 = 7;

// ---------------------------------------------------------------------------------
// Defence (runs every think, before the economy)
// ---------------------------------------------------------------------------------

pub(crate) fn defend(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    garrison(ctx, view, mem);
    let Some(threat) = threat(view) else {
        return;
    };
    if ctx.tick.saturating_sub(mem.last_defend_tick) < 5 * THINK_INTERVAL {
        return;
    }
    // While the army is away only the units still at home respond.
    let mut units: Vec<EntityId> = view
        .military
        .iter()
        .filter(|u| !mem.attacking || within(u.pos, view.home, 30))
        .filter(|u| !ctx.busy.contains(&u.id))
        .map(|u| u.id)
        .collect();
    if mem.scout_done {
        if let Some(s) = view.scout {
            units.push(s.id);
        }
    }
    if units.is_empty() {
        return;
    }
    mem.last_defend_tick = ctx.tick;
    ctx.mark_busy(&units);
    ctx.push(Command::AttackMove {
        units,
        to: threat.pos,
        queue: false,
    });
}

/// Enemy fighters are next to one of our buildings.
pub(crate) fn under_threat(view: &View) -> bool {
    threat(view).is_some()
}

/// The enemy fighter nearest to home that is within `DEFEND_RADIUS` of one of our
/// buildings.
fn threat<'a>(view: &View<'a>) -> Option<&'a Entity> {
    let mut best: Option<(i64, EntityId, &'a Entity)> = None;
    for e in &view.enemy_units {
        let near_building = view
            .buildings
            .iter()
            .chain(view.foundations.iter())
            .any(|b| within(b.pos, e.pos, DEFEND_RADIUS));
        if !near_building {
            continue;
        }
        let key = (dist2(e.pos, view.home), e.id);
        if best.map(|(d, id, _)| key < (d, id)).unwrap_or(true) {
            best = Some((key.0, key.1, e));
        }
    }
    best.map(|(_, _, e)| e)
}

/// Villagers hide in the town centre while enemy fighters are next to it and come out
/// once the coast is clear.
fn garrison(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    let Some(tc) = view.tc else {
        mem.tc_garrisoned = false;
        return;
    };
    let near = view.enemies_near(tc.pos, GARRISON_RADIUS);
    if !near.is_empty() {
        let cap = stats::stats_for(ctx.world, ctx.data, ctx.me, tc.ty).garrison_capacity;
        let room = (cap - tc.garrisoned.len() as i32).max(0) as usize;
        let units: Vec<EntityId> = view
            .villagers
            .iter()
            .filter(|v| within(v.pos, tc.pos, 10) && !ctx.busy.contains(&v.id))
            .map(|v| v.id)
            .take(room)
            .collect();
        if !units.is_empty() {
            ctx.mark_busy(&units);
            ctx.push(Command::Garrison {
                units,
                target: tc.id,
            });
        }
        mem.tc_garrisoned = true;
        return;
    }
    if (mem.tc_garrisoned || !tc.garrisoned.is_empty())
        && view.enemies_near(tc.pos, DEFEND_RADIUS + 2).is_empty()
    {
        if !tc.garrisoned.is_empty() {
            ctx.push(Command::Ungarrison { building: tc.id });
        }
        mem.tc_garrisoned = false;
    }
}

// ---------------------------------------------------------------------------------
// Production, research, scouting and the army (every `military_period` seconds)
// ---------------------------------------------------------------------------------

pub(crate) fn run(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    let counts = LineCounts::new(ctx, view);
    production(ctx, view, &counts);
    research(ctx, view, &counts);
    scout(ctx, view, mem);
    army(ctx, view, mem);
}

/// How many units of each line we field (upgraded types count for their base line).
struct LineCounts {
    spear: usize,
    militia: usize,
    archer: usize,
    skirm: usize,
    scout: usize,
    knight: usize,
    ram: usize,
    mangonel: usize,
    unique: usize,
    infantry: usize,
    archers: usize,
    cavalry: usize,
}

impl LineCounts {
    fn new(ctx: &Ctx, view: &View) -> LineCounts {
        let data = ctx.data;
        let civ = data.civ(view.player.civ);
        let count = |base: &str| -> usize {
            let types = line_types(data, base);
            view.military
                .iter()
                .filter(|u| types.contains(&u.ty))
                .count()
        };
        let role = |r: Role| -> usize {
            view.military
                .iter()
                .filter(|u| data.object(u.ty).role == r)
                .count()
        };
        LineCounts {
            spear: count("spearman"),
            militia: count("militia"),
            archer: count("archer"),
            skirm: count("skirmisher"),
            scout: count("scout_cavalry"),
            knight: count("knight"),
            ram: count("battering_ram"),
            mangonel: count("mangonel"),
            unique: count(&civ.unique_unit),
            infantry: role(Role::Infantry),
            archers: role(Role::Archer),
            cavalry: role(Role::Cavalry) + role(Role::Scout),
        }
    }
}

/// Every type in an upgrade chain starting at `base` (whether researched or not).
fn line_types(data: &GameData, base: &str) -> Vec<TypeId> {
    let mut v = Vec::new();
    let Some(mut cur) = data.object_id(base) else {
        return v;
    };
    for _ in 0..8 {
        v.push(cur);
        match &data.object(cur).upgrades_to {
            Some((_, to)) => match data.object_id(to) {
                Some(t) => cur = t,
                None => break,
            },
            None => break,
        }
    }
    v
}

/// Keep every production building working on the unit the plan wants from it.
fn production(ctx: &mut Ctx, view: &View, counts: &LineCounts) {
    let data = ctx.data;
    if view.military.len() >= ctx.params.max_army || view.age == Age::Dark {
        return;
    }
    let style = civ_style(&data.civ(view.player.civ).key);
    let limit = if ctx.params.military_period == 1 {
        4
    } else {
        3
    };
    let mut trained = 0;
    let mut room = view.pop_room();
    let mut counts_spear = counts.spear;
    let mut counts_skirm = counts.skirm;
    for b in &view.buildings {
        if trained >= limit || room <= 0 {
            break;
        }
        let queued = b
            .queue
            .iter()
            .filter(|q| matches!(q, QueueItem::Unit(_)))
            .count();
        if queued >= 1 {
            continue;
        }
        let key = data.object(b.ty).key.as_str();
        let Some(unit) = pick_unit(ctx, view, counts, style, key, counts_spear, counts_skirm)
        else {
            continue;
        };
        let Some(base) = data.object_id(unit) else {
            continue;
        };
        let ty = stats::resolve_upgrade(ctx.world, data, ctx.me, base);
        if !stats::is_available(ctx.world, data, ctx.me, ty) {
            continue;
        }
        let st = stats::stats_for(ctx.world, data, ctx.me, ty);
        // Saving for an age still leaves room for a small defensive force.
        let defensive =
            view.military.len() + trained < ctx.params.attack_threshold[view.age.idx()] / 2;
        let ok = if defensive {
            ctx.can_afford_defensive(&st.cost)
        } else {
            ctx.can_afford(&st.cost)
        };
        if !ok {
            continue;
        }
        ctx.pay(&st.cost);
        ctx.push(Command::Train {
            building: b.id,
            unit: ty,
        });
        trained += 1;
        room -= st.population;
        match unit {
            "spearman" => counts_spear += 1,
            "skirmisher" => counts_skirm += 1,
            _ => {}
        }
    }
}

/// The unit a building should train now, by age, civ style and enemy composition.
fn pick_unit(
    ctx: &Ctx,
    view: &View,
    c: &LineCounts,
    style: Style,
    building: &str,
    spear: usize,
    skirm: usize,
) -> Option<&'static str> {
    let age = view.age;
    let gold = ctx.available.gold - ctx.reserve.gold;
    let monks = view.monks.len();
    match building {
        "barracks" => {
            if view.enemy_cavalry >= 6 && spear < view.enemy_cavalry {
                return Some("spearman");
            }
            if style == Style::Infantry {
                return if spear * 2 < c.militia {
                    Some("spearman")
                } else {
                    Some("militia")
                };
            }
            // Spearmen cost no gold, so they are also the gold-starved fallback.
            if spear < (c.archer + c.knight + c.militia) / 2 + 2 || gold < 100 {
                Some("spearman")
            } else {
                None
            }
        }
        "archery_range" => {
            if view.enemy_archers >= 6 && skirm < view.enemy_archers {
                return Some("skirmisher");
            }
            if style == Style::Cavalry && age >= Age::Castle {
                return if skirm < 6 { Some("skirmisher") } else { None };
            }
            if gold >= 120 {
                Some("archer")
            } else {
                Some("skirmisher")
            }
        }
        "stable" => {
            if age >= Age::Castle && gold >= 150 {
                Some("knight")
            } else if (style == Style::Cavalry || age >= Age::Castle) && c.scout < 4 {
                Some("scout_cavalry")
            } else {
                None
            }
        }
        "siege_workshop" => {
            if c.ram < 2 {
                Some("battering_ram")
            } else if view.enemy_archers >= 8 && c.mangonel < 2 {
                Some("mangonel")
            } else {
                None
            }
        }
        "castle" => {
            if c.unique < 12 && gold >= 150 {
                Some(unique_unit_key(ctx.data, view.player.civ))
            } else {
                None
            }
        }
        "monastery" => {
            if monks < 2 && age >= Age::Castle && gold >= 300 {
                Some("monk")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Unique unit key as a static str (looked up in the data's own string).
fn unique_unit_key(data: &GameData, civ: CivId) -> &'static str {
    // The data lives in a process-wide table in practice, but we cannot rely on that
    // for the lifetime, so map the known civs to literals.
    match data.civ(civ).unique_unit.as_str() {
        "longbowman" => "longbowman",
        "cataphract" => "cataphract",
        "woad_raider" => "woad_raider",
        "chu_ko_nu" => "chu_ko_nu",
        "throwing_axeman" => "throwing_axeman",
        "huskarl" => "huskarl",
        "samurai" => "samurai",
        "mangudai" => "mangudai",
        "war_elephant" => "war_elephant",
        "mameluke" => "mameluke",
        "teutonic_knight" => "teutonic_knight",
        "janissary" => "janissary",
        "berserk" => "berserk",
        _ => "militia",
    }
}

/// Blacksmith, unit-line and university upgrades for the lines we actually use.
fn research(ctx: &mut Ctx, view: &View, c: &LineCounts) {
    let data = ctx.data;
    let inf = c.infantry >= 4;
    let arc = c.archers >= 4;
    let cav = c.cavalry >= 4;
    let civ = data.civ(view.player.civ);
    let elite = format!("elite_{}", civ.unique_unit);
    let list: Vec<(&str, &str, bool)> = vec![
        ("forging", "blacksmith", inf || cav),
        ("fletching", "blacksmith", arc),
        ("scale_mail_armor", "blacksmith", inf),
        ("padded_archer_armor", "blacksmith", arc),
        ("scale_barding_armor", "blacksmith", cav),
        ("man_at_arms", "barracks", c.militia >= 4),
        ("pikeman", "barracks", c.spear >= 4),
        ("crossbowman", "archery_range", c.archer >= 4),
        ("elite_skirmisher", "archery_range", c.skirm >= 4),
        ("bloodlines", "stable", c.cavalry >= 3),
        ("light_cavalry", "stable", c.scout >= 2),
        ("iron_casting", "blacksmith", inf || cav),
        ("bodkin_arrow", "blacksmith", arc),
        ("chain_mail_armor", "blacksmith", inf),
        ("leather_archer_armor", "blacksmith", arc),
        ("chain_barding_armor", "blacksmith", cav),
        ("long_swordsman", "barracks", c.militia >= 4),
        ("squires", "barracks", c.infantry >= 8),
        ("husbandry", "stable", c.cavalry >= 4),
        ("cavalier", "stable", c.knight >= 4),
        ("capped_ram", "siege_workshop", c.ram >= 2),
        ("ballistics", "university", c.archers >= 6),
        ("masonry", "university", true),
        ("thumb_ring", "archery_range", c.archer >= 8),
        (elite.as_str(), "castle", c.unique >= 6),
        ("blast_furnace", "blacksmith", inf || cav),
        ("bracer", "blacksmith", arc),
        ("plate_mail_armor", "blacksmith", inf),
        ("ring_archer_armor", "blacksmith", arc),
        ("plate_barding_armor", "blacksmith", cav),
        ("two_handed_swordsman", "barracks", c.militia >= 4),
        ("champion", "barracks", c.militia >= 4),
        ("halberdier", "barracks", c.spear >= 4),
        ("arbalest", "archery_range", c.archer >= 4),
        ("hussar", "stable", c.scout >= 2),
        ("paladin", "stable", c.knight >= 4),
        ("siege_ram", "siege_workshop", c.ram >= 2),
        ("onager", "siege_workshop", c.mangonel >= 1),
        ("chemistry", "university", c.archers >= 6),
        ("siege_engineers", "university", c.ram >= 2),
        ("conscription", "castle", true),
    ];
    let buffer = Stockpile::new(50, 0, 25, 0);
    for (tech, at, wanted) in list {
        if !wanted {
            continue;
        }
        let (Some(tech), Some(at)) = (data.tech_id(tech), data.object_id(at)) else {
            continue;
        };
        let def = data.tech(tech);
        if !ctx.params.imperial_unit_upgrades
            && def.age >= Age::Imperial
            && def
                .effects
                .iter()
                .any(|e| matches!(e, Effect::Upgrade { .. }))
        {
            continue;
        }
        if !stats::tech_available(ctx.world, data, ctx.me, tech) {
            continue;
        }
        let Some(b) = view.buildings_of(at).find(|b| b.queue.is_empty()) else {
            continue;
        };
        let (cost, _) = stats::tech_cost(ctx.world, data, ctx.me, tech);
        let mut with_buffer = cost;
        with_buffer.add(&buffer);
        if !ctx.can_afford(&with_buffer) {
            continue;
        }
        ctx.pay(&cost);
        ctx.push(Command::Research {
            building: b.id,
            tech,
        });
        return;
    }
}

// ---------------------------------------------------------------------------------
// Scouting
// ---------------------------------------------------------------------------------

/// Sixteen unit directions scaled by 1000, clockwise from the east.
const DIRS: [(i32, i32); 16] = [
    (1000, 0),
    (924, 383),
    (707, 707),
    (383, 924),
    (0, 1000),
    (-383, 924),
    (-707, 707),
    (-924, 383),
    (-1000, 0),
    (-924, -383),
    (-707, -707),
    (-383, -924),
    (0, -1000),
    (383, -924),
    (707, -707),
    (924, -383),
];

/// The scout walks an expanding spiral until the sheep around the base are found,
/// then comes home and joins the army.
fn scout(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    if mem.scout_done {
        return;
    }
    let Some(s) = view.scout else {
        return;
    };
    if ctx.busy.contains(&s.id) {
        return;
    }
    if s.stance != Stance::NoAttack {
        ctx.push(Command::SetStance {
            units: vec![s.id],
            stance: Stance::NoAttack,
        });
    }
    if !s.is_idle() {
        return;
    }
    let data = ctx.data;
    let wild_sheep = view.nodes.iter().any(|n| {
        n.owner.is_gaia()
            && data.object(n.ty).yields.map(|y| y.kind) == Some(GatherKind::Sheep)
            && within(n.pos, view.home, 22)
    });
    if mem.scout_step >= 24 || (mem.scout_step >= 8 && !wild_sheep) {
        mem.scout_done = true;
        ctx.push(Command::SetStance {
            units: vec![s.id],
            stance: Stance::Defensive,
        });
        ctx.push(Command::Move {
            units: vec![s.id],
            to: rally_point(ctx, view),
            queue: false,
        });
        return;
    }
    let step = mem.scout_step;
    mem.scout_step += 1;
    let (dx, dy) = DIRS[((step * 5) % 16) as usize];
    let radius = 6 + 2 * step as i32;
    let (hx, hy) = view.home.tile();
    let map = &ctx.world.map;
    let x = (hx + dx * radius / 1000).clamp(1, map.width - 2);
    let y = (hy + dy * radius / 1000).clamp(1, map.height - 2);
    if let Some((px, py)) = map.nearest_passable(x, y, false, 6) {
        ctx.busy.insert(s.id);
        ctx.push(Command::Move {
            units: vec![s.id],
            to: Vec2::tile_center(px, py),
            queue: false,
        });
    }
}

// ---------------------------------------------------------------------------------
// Army
// ---------------------------------------------------------------------------------

/// Holding point: a few tiles from the town centre towards the enemy.
fn rally_point(ctx: &Ctx, view: &View) -> Vec2 {
    let toward = view
        .enemy_buildings
        .iter()
        .map(|b| (dist2(b.pos, view.home), b.id, b.pos))
        .min()
        .map(|(_, _, p)| p)
        .or_else(|| view.enemy_units.first().map(|u| u.pos))
        .unwrap_or(Vec2::tile_center(
            ctx.world.map.width / 2,
            ctx.world.map.height / 2,
        ));
    let dx = toward.x.0 as i64 - view.home.x.0 as i64;
    let dy = toward.y.0 as i64 - view.home.y.0 as i64;
    let len = isqrt(dx * dx + dy * dy).max(1);
    let k = RALLY_DIST as i64 * ONE as i64;
    let p = Vec2::new(
        Fixed((view.home.x.0 as i64 + dx * k / len) as i32),
        Fixed((view.home.y.0 as i64 + dy * k / len) as i32),
    );
    ctx.world.map.clamp(p)
}

/// The next thing to attack from `from`: town centres first, then other production
/// buildings, then whatever is nearest.
fn pick_target<'a>(ctx: &Ctx, view: &View<'a>, from: Vec2) -> Option<&'a Entity> {
    let data = ctx.data;
    let mut best: Option<(i32, EntityId, &'a Entity)> = None;
    for b in &view.enemy_buildings {
        let def = data.object(b.ty);
        if def.role == Role::Wall || def.role == Role::Gate {
            continue;
        }
        let mut score = tiles(from, b.pos);
        if def.is_town_center {
            score -= 25;
        } else if data.trains_at.contains_key(&b.ty) {
            score -= 8;
        }
        if best
            .map(|(s, id, _)| (score, b.id) < (s, id))
            .unwrap_or(true)
        {
            best = Some((score, b.id, b));
        }
    }
    if best.is_none() {
        for u in &view.enemy_units {
            let score = tiles(from, u.pos);
            if best
                .map(|(s, id, _)| (score, u.id) < (s, id))
                .unwrap_or(true)
            {
                best = Some((score, u.id, u));
            }
        }
    }
    best.map(|(_, _, e)| e)
}

fn army(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    let units: Vec<&Entity> = view
        .military
        .iter()
        .filter(|u| !ctx.busy.contains(&u.id))
        .copied()
        .collect();
    let ids: Vec<EntityId> = units.iter().map(|u| u.id).collect();
    let n = view.military.len();
    let rally = rally_point(ctx, view);
    let enemy_present = !view.enemy_buildings.is_empty() || !view.enemy_units.is_empty();
    let under_threat = threat(view).is_some();

    if mem.attacking {
        mem.army_peak = mem.army_peak.max(n as u32);
        if n == 0 || (n as u32) * 10 < mem.army_peak * 3 || !enemy_present {
            // The push failed: regroup at home and try again later.
            mem.attacking = false;
            mem.attack_target = None;
            mem.army_peak = 0;
            mem.retreat_until = ctx.tick + 60 * THINK_INTERVAL;
            if !ids.is_empty() {
                set_stance(ctx, &units, Stance::Defensive);
                ctx.push(Command::Move {
                    units: ids,
                    to: rally,
                    queue: false,
                });
            }
            return;
        }
        let target_alive = mem
            .attack_target
            .and_then(|t| ctx.world.get(t))
            .map(|t| ctx.world.enemy(ctx.me, t.owner))
            .unwrap_or(false);
        if !target_alive || ctx.tick.saturating_sub(mem.last_attack_tick) >= 30 * THINK_INTERVAL {
            let from = if target_alive {
                view.army_centre()
            } else {
                view.home
            };
            match pick_target(ctx, view, from) {
                Some(t) if !ids.is_empty() => {
                    mem.attack_target = Some(t.id);
                    mem.last_attack_tick = ctx.tick;
                    set_stance(ctx, &units, Stance::Aggressive);
                    ctx.mark_busy(&ids);
                    ctx.push(Command::AttackMove {
                        units: ids,
                        to: t.pos,
                        queue: false,
                    });
                }
                _ => {
                    mem.attacking = false;
                    mem.attack_target = None;
                }
            }
        }
        return;
    }

    let threshold = ctx.params.attack_threshold[view.age.idx()];
    if ctx.tick >= mem.retreat_until && n >= threshold && !under_threat && enemy_present {
        if let Some(t) = pick_target(ctx, view, view.home) {
            if !ids.is_empty() {
                mem.attacking = true;
                mem.attack_target = Some(t.id);
                mem.army_peak = n as u32;
                mem.last_attack_tick = ctx.tick;
                set_stance(ctx, &units, Stance::Aggressive);
                ctx.mark_busy(&ids);
                ctx.push(Command::AttackMove {
                    units: ids,
                    to: t.pos,
                    queue: false,
                });
                return;
            }
        }
    }

    // At home: hold around the rally point on a defensive stance.
    if ctx.tick.saturating_sub(mem.last_rally_tick) >= 10 * THINK_INTERVAL {
        mem.last_rally_tick = ctx.tick;
        set_stance(ctx, &units, Stance::Defensive);
        let mut far: Vec<EntityId> = units
            .iter()
            .filter(|u| u.is_idle() && !within(u.pos, rally, 5))
            .map(|u| u.id)
            .collect();
        for m in &view.monks {
            if let Some(e) = ctx.world.get(*m) {
                if e.is_idle() && !within(e.pos, rally, 6) {
                    far.push(*m);
                }
            }
        }
        if !far.is_empty() {
            ctx.mark_busy(&far);
            ctx.push(Command::Move {
                units: far,
                to: rally,
                queue: false,
            });
        }
    }
}

/// One `SetStance` for every unit that is not already on `stance`.
fn set_stance(ctx: &mut Ctx, units: &[&Entity], stance: Stance) {
    let ids: Vec<EntityId> = units
        .iter()
        .filter(|u| u.stance != stance && !matches!(u.task(), Task::Garrison { .. }))
        .map(|u| u.id)
        .collect();
    if !ids.is_empty() {
        ctx.push(Command::SetStance { units: ids, stance });
    }
}
