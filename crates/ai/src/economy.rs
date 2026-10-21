//! Economy: villagers, housing, the build order, resource assignment, farms and camps,
//! economic research, repairs and the market.

use crate::place::Placer;
use crate::view::{dist2, tiles, View};
use crate::{mix, AiMemory, Ctx};
use std::collections::BTreeMap;
use wars_sim::command::Command;
use wars_sim::fixed::Vec2;
use wars_sim::ids::*;
use wars_sim::stats;
use wars_sim::types::{GatherKind, ObjectClass, Role};
use wars_sim::world::{Entity, QueueItem, Task};

/// Villager split per age in percent: food, wood, gold, stone.
fn ratio(age: Age) -> [i32; 4] {
    match age {
        Age::Dark => [68, 32, 0, 0],
        Age::Feudal => [42, 40, 13, 5],
        Age::Castle => [40, 30, 25, 5],
        Age::Imperial => [35, 30, 25, 10],
    }
}

/// How many villagers may share one node of each kind.
fn node_capacity(kind: GatherKind, class: ObjectClass) -> i32 {
    match kind {
        GatherKind::Farm => 1,
        GatherKind::Berries => 2,
        GatherKind::Wood => 2,
        GatherKind::Gold | GatherKind::Stone => 3,
        GatherKind::Sheep => 4,
        GatherKind::Hunt => {
            if class == ObjectClass::Animal {
                4
            } else {
                3
            }
        }
        GatherKind::Fish => 0,
    }
}

/// Which civ plays which composition. Used by the build order and production.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Style {
    Archer,
    Cavalry,
    Infantry,
}

pub(crate) fn civ_style(key: &str) -> Style {
    match key {
        "franks" | "persians" | "mongols" | "saracens" | "byzantines" => Style::Cavalry,
        "goths" | "celts" | "vikings" | "japanese" | "teutons" => Style::Infantry,
        _ => Style::Archer,
    }
}

/// The next age-up tech and its cost once the villager count and the prerequisite
/// buildings are there.
pub(crate) fn age_goal(ctx: &Ctx, view: &View) -> Option<(TechId, Stockpile)> {
    let data = ctx.data;
    let p = view.player;
    let (key, prereq) = match view.age {
        Age::Dark => ("feudal_age", true),
        Age::Feudal => (
            "castle_age",
            view.has(data, "blacksmith")
                && (view.has(data, "archery_range") || view.has(data, "stable")),
        ),
        Age::Castle => (
            "imperial_age",
            view.has(data, "siege_workshop") || view.has(data, "castle"),
        ),
        Age::Imperial => return None,
    };
    if view.villager_count() < ctx.params.age_up_villagers[view.age.idx()] || !prereq {
        return None;
    }
    let tech = data.tech_id(key)?;
    if p.has_tech(tech) || p.researching.contains(&tech) {
        return None;
    }
    let (cost, _) = stats::tech_cost(ctx.world, data, ctx.me, tech);
    Some((tech, cost))
}

/// Food and gold kept aside while saving up for the next age. Dropped when enemies are
/// at the gates so the army can still be reinforced.
pub(crate) fn age_reserve(ctx: &Ctx, view: &View) -> Stockpile {
    if crate::military::under_threat(view) {
        return Stockpile::ZERO;
    }
    match age_goal(ctx, view) {
        Some((_, cost)) => Stockpile::new(cost.food, 0, cost.gold, 0),
        None => Stockpile::ZERO,
    }
}

/// Houses, villagers, the age-up, the build order and crews for lonely foundations.
pub(crate) fn core(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    let mut placer = Placer::new(ctx.world, ctx.data, ctx.me, view.home);
    houses(ctx, view, &mut placer);
    train_villagers(ctx, view);
    age_up(ctx, view);
    build_order(ctx, view, mem, &mut placer);
    crew_foundations(ctx, view);
}

/// Idle villagers and the periodic rebalancing between resources.
pub(crate) fn villagers(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    let mut placer = Placer::new(ctx.world, ctx.data, ctx.me, view.home);
    assign(ctx, view, mem, &mut placer);
}

/// Low priority: economic research, repairs, market trades, the wonder.
pub(crate) fn extras(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    research(ctx, view);
    repair(ctx, view, mem);
    market(ctx, view, mem);
    wonder(ctx, view);
}

// ---------------------------------------------------------------------------------
// Housing and villagers
// ---------------------------------------------------------------------------------

fn houses(ctx: &mut Ctx, view: &View, placer: &mut Placer) {
    let data = ctx.data;
    let Some(house) = data.object_id("house") else {
        return;
    };
    let raw_cap = stats::pop_cap(ctx.world, data, ctx.me);
    let p = view.player;
    if p.pop_support >= raw_cap {
        return;
    }
    let room = p.pop_support - p.pop - view.queued_units;
    let lookahead = if view.age >= Age::Castle { 6 } else { 4 };
    if room >= lookahead {
        return;
    }
    let pending = view.pending_count(house);
    let max_pending = if room < 1 && view.age >= Age::Feudal && ctx.params.max_foundations > 1 {
        2
    } else {
        1
    };
    if pending >= max_pending {
        return;
    }
    let rot = mix(ctx.tick, pending as u32);
    issue_build(
        ctx,
        view,
        placer,
        house,
        view.home.tile(),
        7,
        24,
        true,
        1,
        true,
        rot,
    );
}

fn train_villagers(ctx: &mut Ctx, view: &View) {
    let data = ctx.data;
    let Some(vil) = data.object_id("villager") else {
        return;
    };
    let Some(tc_ty) = data.object_id("town_center") else {
        return;
    };
    let target = ctx.params.villager_target[view.age.idx()];
    let mut queued = view.queued_villagers;
    let cost = stats::stats_for(ctx.world, data, ctx.me, vil).cost;
    for tc in view.buildings_of(tc_ty) {
        let in_queue = tc
            .queue
            .iter()
            .filter(|q| matches!(q, QueueItem::Unit(_)))
            .count();
        if in_queue >= 2 || tc.queue.len() >= 3 {
            continue;
        }
        if view.villager_count() + queued >= target || view.pop_room() - queued <= 0 {
            break;
        }
        // While saving for an age the reserve applies; otherwise villagers come first.
        let ok = if ctx.reserve == Stockpile::ZERO {
            ctx.can_afford_essential(&cost)
        } else {
            ctx.can_afford(&cost)
        };
        if !ok {
            break;
        }
        ctx.pay(&cost);
        ctx.push(Command::Train {
            building: tc.id,
            unit: vil,
        });
        queued += 1;
    }
}

fn age_up(ctx: &mut Ctx, view: &View) {
    let Some((tech, cost)) = age_goal(ctx, view) else {
        return;
    };
    if !ctx.can_afford_essential(&cost) {
        return;
    }
    let Some(tc) = view.tc else {
        return;
    };
    if tc.queue.iter().any(|q| matches!(q, QueueItem::Tech(_))) {
        return;
    }
    ctx.pay(&cost);
    ctx.push(Command::Research {
        building: tc.id,
        tech,
    });
}

// ---------------------------------------------------------------------------------
// Build order
// ---------------------------------------------------------------------------------

/// One entry of the build order: have `want` of `key` once `min_villagers` exist.
struct Step {
    key: &'static str,
    want: i32,
    min_villagers: i32,
}

fn step(key: &'static str, want: i32, min_villagers: i32) -> Step {
    Step {
        key,
        want,
        min_villagers,
    }
}

/// The build order up to the current age. Entries are checked in order every think and
/// the first missing, affordable, placeable one is started.
fn order(age: Age, style: Style) -> Vec<Step> {
    let mut v = vec![
        step("town_center", 1, 0),
        step("lumber_camp", 1, 7),
        step("mill", 1, 9),
        step("barracks", 1, 13),
    ];
    if age >= Age::Feudal {
        v.push(step("archery_range", 1, 0));
        v.push(step("blacksmith", 1, 0));
        if style == Style::Cavalry {
            v.push(step("stable", 1, 24));
        }
        v.push(step("market", 1, 26));
        match style {
            Style::Archer => v.push(step("archery_range", 2, 30)),
            Style::Infantry => v.push(step("barracks", 2, 30)),
            Style::Cavalry => {}
        }
    }
    if age >= Age::Castle {
        v.push(step("stable", 1, 0));
        v.push(step("siege_workshop", 1, 0));
        v.push(step("castle", 1, 0));
        v.push(step("monastery", 1, 30));
        v.push(step("university", 1, 32));
        match style {
            Style::Archer => v.push(step("archery_range", 2, 36)),
            Style::Cavalry => v.push(step("stable", 2, 36)),
            Style::Infantry => v.push(step("barracks", 2, 36)),
        }
    }
    if age >= Age::Imperial {
        v.push(step("barracks", 2, 0));
        match style {
            Style::Archer => v.push(step("archery_range", 3, 40)),
            Style::Cavalry => v.push(step("stable", 3, 40)),
            Style::Infantry => v.push(step("barracks", 3, 40)),
        }
        v.push(step("siege_workshop", 2, 45));
    }
    v
}

/// The next building the build order wants (index into `order`, type, cost). Steps
/// that failed to place recently are skipped; the castle is skipped while its stone is
/// missing so cheaper buildings are not held back by it.
fn next_step(ctx: &Ctx, view: &View, mem: &AiMemory) -> Option<(usize, TypeId, Stockpile)> {
    let data = ctx.data;
    let style = civ_style(&data.civ(view.player.civ).key);
    let steps = order(view.age, style);
    for (i, st) in steps.iter().enumerate() {
        let Some(ty) = data.object_id(st.key) else {
            continue;
        };
        if view.count(ty) + view.pending_count(ty) >= st.want {
            continue;
        }
        if view.pending_count(ty) > 0 || view.villager_count() < st.min_villagers {
            continue;
        }
        if !stats::is_available(ctx.world, data, ctx.me, ty) {
            continue;
        }
        if let Some(t) = mem.place_failed.get(&ty.0) {
            if ctx.tick.saturating_sub(*t) < 60 * crate::THINK_INTERVAL {
                continue;
            }
        }
        let cost = stats::stats_for(ctx.world, data, ctx.me, ty).cost;
        if st.key == "castle" && !ctx.can_afford_eco(&cost) {
            continue;
        }
        return Some((i, ty, cost));
    }
    None
}

/// What military spending must leave alone: the next building of the build order
/// and the wood farms need to reseed.
pub(crate) fn build_reserve(ctx: &Ctx, view: &View, mem: &AiMemory) -> Stockpile {
    let mut r = Stockpile::ZERO;
    if let Some((_, _, cost)) = next_step(ctx, view, mem) {
        r.add(&cost);
    }
    if view.farm_count(ctx.data) > 0 {
        r.wood += 60;
    }
    // Wood cutters walking more than eight tiles to a drop-off need a new camp.
    let data = ctx.data;
    let camp_pending = data
        .object_id("lumber_camp")
        .map(|c| view.pending_count(c) > 0)
        .unwrap_or(true);
    if !camp_pending {
        let far = view.gather_ids[Resource::Wood.idx()].iter().any(|id| {
            let Some(v) = ctx.world.get(*id) else {
                return false;
            };
            let Task::Gather { target, kind } = v.task() else {
                return false;
            };
            ctx.world
                .get(*target)
                .and_then(|n| view.nearest_drop(ctx.world, data, n.pos, *kind))
                .map(|(d, _)| d > 8)
                .unwrap_or(false)
        });
        if far {
            r.wood += 100;
        }
    }
    r
}

fn build_order(ctx: &mut Ctx, view: &View, mem: &mut AiMemory, placer: &mut Placer) {
    let data = ctx.data;
    let house = data.object_id("house");
    let farm = data.object_id("farm");
    let other_foundations = view
        .foundations
        .iter()
        .filter(|f| Some(f.ty) != house && Some(f.ty) != farm)
        .count();
    if other_foundations >= ctx.params.max_foundations {
        return;
    }
    let Some((i, ty, cost)) = next_step(ctx, view, mem) else {
        return;
    };
    let key = data.object(ty).key.as_str();
    let essential = key == "town_center";
    let affordable = if essential {
        ctx.can_afford_essential(&cost)
    } else {
        ctx.can_afford_eco(&cost)
    };
    if !affordable {
        return;
    }
    mem.build_step = i;
    let (centre, min_r, max_r, avoid) = site_for(ctx, view, key);
    let builders = match key {
        "castle" | "town_center" => 3,
        "lumber_camp" | "mill" | "mining_camp" => 1,
        _ => 2,
    };
    let rot = mix(ctx.tick, ty.0 as u32);
    if !issue_build(
        ctx, view, placer, ty, centre, min_r, max_r, avoid, builders, essential, rot,
    ) {
        mem.place_failed.insert(ty.0, ctx.tick);
    }
}

/// Where a building of the build order should go: camps next to their resource, the
/// mill next to the berries, everything else around the town centre.
fn site_for(ctx: &Ctx, view: &View, key: &str) -> ((i32, i32), i32, i32, bool) {
    let home = view.home.tile();
    let near = |kind: GatherKind| -> Option<(i32, i32)> {
        view.nodes
            .iter()
            .filter(|n| ctx.data.object(n.ty).yields.map(|y| y.kind) == Some(kind))
            .map(|n| (dist2(n.pos, view.home), n.id, n.pos.tile()))
            .min()
            .map(|(_, _, t)| t)
    };
    match key {
        "lumber_camp" => match near(GatherKind::Wood) {
            Some(t) => (t, 1, 5, false),
            None => (home, 4, 20, true),
        },
        "mill" => match near(GatherKind::Berries) {
            Some(t) => (t, 1, 5, false),
            None => (home, 4, 12, true),
        },
        "mining_camp" => match near(GatherKind::Gold) {
            Some(t) => (t, 1, 5, false),
            None => (home, 4, 20, true),
        },
        "town_center" => (home, 0, 12, false),
        "castle" => (home, 5, 20, true),
        _ => (home, 4, 22, true),
    }
}

/// Find a spot for `ty` and send `n` villagers to build it. Returns true when a Build
/// command was issued.
fn issue_build(
    ctx: &mut Ctx,
    view: &View,
    placer: &mut Placer,
    ty: TypeId,
    centre: (i32, i32),
    min_r: i32,
    max_r: i32,
    avoid_piles: bool,
    n: usize,
    essential: bool,
    rot: u32,
) -> bool {
    let data = ctx.data;
    let cost = stats::stats_for(ctx.world, data, ctx.me, ty).cost;
    let ok = if essential {
        ctx.can_afford_essential(&cost)
    } else {
        ctx.can_afford_eco(&cost)
    };
    if !ok || !stats::is_available(ctx.world, data, ctx.me, ty) {
        return false;
    }
    let Some((x, y)) = placer.find(ctx.world, data, ty, centre, min_r, max_r, avoid_piles, rot)
    else {
        return false;
    };
    let spot = Vec2::tile_center(x, y);
    let units = pick_builders(ctx, view, spot, n, true);
    if units.is_empty() {
        return false;
    }
    placer.reserve(data, ty, x, y);
    ctx.pay(&cost);
    ctx.mark_busy(&units);
    ctx.push(Command::Build {
        units,
        building: ty,
        x,
        y,
        queue: false,
    });
    true
}

/// Villagers for a job at `spot`: idle first, then wood/gold/stone gatherers, then (if
/// allowed) food gatherers; nearest first. Builders and busy villagers are skipped.
fn pick_builders(ctx: &Ctx, view: &View, spot: Vec2, n: usize, allow_food: bool) -> Vec<EntityId> {
    let mut cands: Vec<(i32, i64, EntityId)> = Vec::new();
    for v in &view.villagers {
        if ctx.busy.contains(&v.id) {
            continue;
        }
        let job = match v.task() {
            Task::Gather { kind, .. } => Some(gather_resource(v, *kind)),
            Task::DropOff { .. } => {
                Some(v.carry_kind.map(|k| k.resource()).unwrap_or(Resource::Food))
            }
            Task::Attack { .. } => Some(Resource::Food),
            _ => None,
        };
        let rank = match (v.task(), job) {
            (Task::Idle, _) => 0,
            (Task::Move { .. }, _) => 1,
            (_, Some(Resource::Food)) if allow_food => 2,
            (_, Some(Resource::Food)) => continue,
            (_, Some(_)) => 1,
            _ => continue,
        };
        cands.push((rank, dist2(v.pos, spot), v.id));
    }
    cands.sort();
    cands.into_iter().take(n).map(|(_, _, id)| id).collect()
}

fn gather_resource(e: &Entity, kind: GatherKind) -> Resource {
    e.carry_kind
        .map(|k| k.resource())
        .unwrap_or(kind.resource())
}

/// Foundations nobody is building (the builder died or got reassigned) get a villager.
fn crew_foundations(ctx: &mut Ctx, view: &View) {
    let mut sent = 0;
    for f in &view.foundations {
        if view.crewed.contains(&f.id) || sent >= 2 {
            continue;
        }
        let big = matches!(
            ctx.data.object(f.ty).key.as_str(),
            "castle" | "wonder" | "town_center"
        );
        let units = pick_builders(ctx, view, f.pos, if big { 3 } else { 1 }, true);
        if units.is_empty() {
            return;
        }
        ctx.mark_busy(&units);
        ctx.push(Command::Repair {
            units,
            target: f.id,
            queue: false,
        });
        sent += 1;
    }
}

// ---------------------------------------------------------------------------------
// Resource assignment
// ---------------------------------------------------------------------------------

/// What a villager should do for a resource.
enum Choice {
    Gather(EntityId),
    /// Plant a farm at this tile (the villager gathers it once built).
    Farm(i32, i32),
    /// Build a drop-off camp next to the node, then gather it.
    Camp(TypeId, i32, i32, EntityId),
}

fn assign(ctx: &mut Ctx, view: &View, mem: &mut AiMemory, placer: &mut Placer) {
    let total = view.villager_count();
    if total == 0 {
        return;
    }
    let data = ctx.data;
    let ratio = ratio(view.age);
    let mut desired = [0i32; 4];
    for r in Resource::ALL {
        desired[r.idx()] = total * ratio[r.idx()] / 100;
    }
    // Do not over-mine: once there is plenty of stone, put those villagers on food.
    if view.player.resources.stone > 900 {
        desired[Resource::Food.idx()] += desired[Resource::Stone.idx()];
        desired[Resource::Stone.idx()] = 0;
    }
    if view.player.resources.gold > 2500 {
        desired[Resource::Food.idx()] += desired[Resource::Gold.idx()];
        desired[Resource::Gold.idx()] = 0;
    }
    // Stock-aware correction: a third of the villagers on a resource we are swimming
    // in go to the resource we are out of (wood pays for farms, so it starves first).
    let stock = view.player.resources;
    let short = Resource::ALL
        .iter()
        .copied()
        .filter(|r| desired[r.idx()] > 0 && stock.get(*r) < 150)
        .min_by_key(|r| (stock.get(*r), r.idx()));
    if let Some(short) = short {
        for r in Resource::ALL {
            if r != short && stock.get(r) > 500 && desired[r.idx()] >= 3 {
                let moved = desired[r.idx()] / 3;
                desired[r.idx()] -= moved;
                desired[short.idx()] += moved;
            }
        }
    }
    let mut current = [0i32; 4];
    for r in Resource::ALL {
        current[r.idx()] = view.gatherers(r);
    }
    let mut workers = view.workers.clone();
    let mut batches: BTreeMap<EntityId, Vec<EntityId>> = BTreeMap::new();
    let mut builds = 0;
    let max_builds = 2;

    // Idle villagers first, then the ones the watchdog found stuck.
    let mut todo: Vec<EntityId> = view
        .idle_villagers
        .iter()
        .copied()
        .filter(|id| !ctx.busy.contains(id))
        .collect();
    for id in watchdog(ctx, view, mem) {
        if !todo.contains(&id) {
            todo.push(id);
        }
    }
    // The sim lets villagers hop to the next node of the same kind on their own, which
    // can lead them far from any drop-off: bring those back into the fold.
    for v in &view.villagers {
        if ctx.busy.contains(&v.id) || todo.contains(&v.id) {
            continue;
        }
        let (target, kind) = match v.task() {
            Task::Gather { target, kind } => (*target, *kind),
            _ => continue,
        };
        let Some(node) = ctx.world.get(target) else {
            continue;
        };
        let far = view
            .nearest_drop(ctx.world, data, node.pos, kind)
            .map(|(d, _)| d > 14)
            .unwrap_or(true);
        if far {
            current[kind.resource().idx()] -= 1;
            todo.push(v.id);
        }
    }

    // Every ten seconds move a few villagers from the most over-staffed resource to
    // the most under-staffed one.
    if ctx.tick.saturating_sub(mem.last_rebalance_tick) >= 10 * crate::THINK_INTERVAL {
        mem.last_rebalance_tick = ctx.tick;
        let mut deficit = (0, Resource::Food);
        let mut surplus = (0, Resource::Food);
        for r in Resource::ALL {
            let d = desired[r.idx()] - current[r.idx()];
            if d > deficit.0 {
                deficit = (d, r);
            }
            if -d > surplus.0 {
                surplus = (-d, r);
            }
        }
        if deficit.0 >= 2 && surplus.0 >= 2 {
            let n = deficit.0.min(surplus.0).min(3) as usize;
            let mut movers: Vec<(i32, EntityId)> = view.gather_ids[surplus.1.idx()]
                .iter()
                .filter(|id| !ctx.busy.contains(id))
                .filter_map(|id| ctx.world.get(*id))
                .map(|e| (e.carry, e.id))
                .collect();
            movers.sort();
            for (_, id) in movers.into_iter().take(n) {
                current[surplus.1.idx()] -= 1;
                todo.push(id);
            }
        }
    }

    for id in todo {
        let Some(vil) = ctx.world.get(id) else {
            continue;
        };
        // Resources by deficit, largest first; ties keep the food, wood, gold, stone order.
        let mut prefs: Vec<(i32, usize)> = Resource::ALL
            .iter()
            .map(|r| (current[r.idx()] - desired[r.idx()], r.idx()))
            .collect();
        prefs.sort();
        // When food is wanted but no farm can be afforded, wood is the next best job
        // (it pays for the farms) whatever the other deficits say.
        if prefs.first().map(|p| p.1) == Some(Resource::Food.idx()) {
            prefs.retain(|p| p.1 != Resource::Wood.idx());
            prefs.insert(1, (0, Resource::Wood.idx()));
        }
        let mut done = false;
        for (_, ri) in prefs {
            let res = Resource::ALL[ri];
            if desired[ri] == 0 && res != Resource::Wood {
                continue;
            }
            let allow_build = builds < max_builds;
            let Some(choice) = pick_node(
                ctx,
                view,
                placer,
                vil,
                res,
                &mut workers,
                &mem.banned,
                allow_build,
            ) else {
                continue;
            };
            match choice {
                Choice::Gather(node) => batches.entry(node).or_default().push(id),
                Choice::Farm(x, y) => {
                    let farm = data.obj("farm");
                    let cost = stats::stats_for(ctx.world, data, ctx.me, farm).cost;
                    ctx.pay(&cost);
                    placer.reserve(data, farm, x, y);
                    ctx.push(Command::Build {
                        units: vec![id],
                        building: farm,
                        x,
                        y,
                        queue: false,
                    });
                    builds += 1;
                }
                Choice::Camp(camp, x, y, node) => {
                    let cost = stats::stats_for(ctx.world, data, ctx.me, camp).cost;
                    ctx.pay(&cost);
                    placer.reserve(data, camp, x, y);
                    ctx.push(Command::Build {
                        units: vec![id],
                        building: camp,
                        x,
                        y,
                        queue: false,
                    });
                    ctx.push(Command::Gather {
                        units: vec![id],
                        target: node,
                        queue: true,
                    });
                    builds += 1;
                }
            }
            ctx.busy.insert(id);
            current[ri] += 1;
            done = true;
            break;
        }
        if !done {
            // Nothing to gather at all: at least walk home so the next scan sees it idle.
            continue;
        }
    }
    for (node, units) in batches {
        ctx.push(Command::Gather {
            units,
            target: node,
            queue: false,
        });
    }
}

/// Villagers that have not moved for a while although their job needs them to walk
/// (a node or foundation trapped behind buildings or trees). Their target is banned
/// for a few minutes, unfinishable foundations are deleted, and the villagers are
/// returned for reassignment.
fn watchdog(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) -> Vec<EntityId> {
    const STUCK_SECS: u32 = 12;
    const BAN_SECS: u32 = 240;
    let tick = ctx.tick;
    mem.banned.retain(|_, until| *until > tick);
    let mut seen = std::collections::BTreeSet::new();
    let mut stuck = Vec::new();
    for v in &view.villagers {
        let target = match v.task() {
            Task::Gather { target, .. } | Task::Attack { target, .. } | Task::Build { target } => {
                *target
            }
            _ => continue,
        };
        seen.insert(v.id.index);
        // Working (anim 3) or attacking (anim 2) villagers are where they should be.
        if v.anim == 3 || v.anim == 2 {
            mem.watch.remove(&v.id.index);
            continue;
        }
        let key = (v.pos.x.0, v.pos.y.0);
        match mem.watch.get(&v.id.index).copied() {
            Some((x, y, since)) if (x, y) == key => {
                if tick.saturating_sub(since) >= STUCK_SECS * crate::THINK_INTERVAL {
                    mem.banned
                        .insert(target, tick + BAN_SECS * crate::THINK_INTERVAL);
                    if matches!(v.task(), Task::Build { .. }) {
                        if let Some(f) = ctx.world.get(target) {
                            if f.owner == ctx.me && f.build_progress < 1000 {
                                ctx.push(Command::Delete { entity: target });
                            }
                        }
                    }
                    stuck.push(v.id);
                    mem.watch.remove(&v.id.index);
                }
            }
            _ => {
                mem.watch.insert(v.id.index, (key.0, key.1, tick));
            }
        }
    }
    mem.watch.retain(|k, _| seen.contains(k));
    stuck
}

/// Choose the node a villager should work for resource `res`.
fn pick_node(
    ctx: &Ctx,
    view: &View,
    placer: &mut Placer,
    vil: &Entity,
    res: Resource,
    workers: &mut BTreeMap<EntityId, i32>,
    mem_banned: &BTreeMap<EntityId, u32>,
    allow_build: bool,
) -> Option<Choice> {
    let data = ctx.data;
    let world = ctx.world;
    // (priority, distance, id, node)
    let mut best: Option<(i32, i32, EntityId, &Entity)> = None;
    for n in &view.nodes {
        let def = data.object(n.ty);
        let Some(y) = def.yields else { continue };
        if y.resource != res {
            continue;
        }
        if def.class == ObjectClass::Animal {
            // Live boar and wolves fight back: leave them alone.
            if !n.alerted && !def.attacks.is_empty() {
                continue;
            }
            // Sheep belong to whoever got there first.
            if y.kind == GatherKind::Sheep && !(n.owner == ctx.me || n.owner.is_gaia()) {
                continue;
            }
        }
        if def.class == ObjectClass::Building && n.owner != ctx.me {
            continue;
        }
        let used = workers.get(&n.id).copied().unwrap_or(0);
        if used >= node_capacity(y.kind, def.class) || mem_banned.contains_key(&n.id) {
            continue;
        }
        // The walk to the drop-off is paid on every trip, so it dominates the score;
        // the villager's own walk there only breaks ties.
        let drop = view
            .nearest_drop(world, data, n.pos, y.kind)
            .map(|(dd, _)| dd)
            .unwrap_or(40);
        let penalty = match y.kind {
            GatherKind::Sheep => 0,
            GatherKind::Hunt if n.alerted => 0,
            GatherKind::Farm | GatherKind::Berries => 1,
            GatherKind::Hunt => 3,
            _ => 0,
        };
        let score = drop * 2 + penalty;
        let d = tiles(vil.pos, n.pos);
        let key = (score, d, n.id);
        if best
            .map(|(bp, bd, bid, _)| key < (bp, bd, bid))
            .unwrap_or(true)
        {
            best = Some((score, d, n.id, n));
        }
    }
    if res == Resource::Food {
        // Farms (next to a drop-off) beat wild food that is far from one, and are the
        // only food once the wild food is gone.
        let wild_far = best.map(|(score, _, _, _)| score > 12).unwrap_or(true);
        if wild_far && allow_build {
            if let Some(c) = plan_farm(ctx, view, placer) {
                return Some(c);
            }
        }
        if let Some((_, _, id, _)) = best {
            *workers.entry(id).or_insert(0) += 1;
            return Some(Choice::Gather(id));
        }
        return None;
    }
    let (_, _, id, node) = best?;
    let kind = data.object(node.ty).yields?.kind;
    let drop_d = view
        .nearest_drop(world, data, node.pos, kind)
        .map(|(d, _)| d)
        .unwrap_or(99);
    if drop_d > 8 && allow_build {
        let camp_key = if kind == GatherKind::Wood {
            "lumber_camp"
        } else {
            "mining_camp"
        };
        if let Some(camp) = data.object_id(camp_key) {
            let cost = stats::stats_for(world, data, ctx.me, camp).cost;
            if view.pending_count(camp) == 0
                && ctx.can_afford_eco(&cost)
                && stats::is_available(world, data, ctx.me, camp)
            {
                let rot = mix(ctx.tick, node.id.index);
                if let Some((x, y)) =
                    placer.find(world, data, camp, node.pos.tile(), 1, 5, false, rot)
                {
                    *workers.entry(id).or_insert(0) += 1;
                    return Some(Choice::Camp(camp, x, y, id));
                }
            }
        }
    }
    *workers.entry(id).or_insert(0) += 1;
    Some(Choice::Gather(id))
}

/// A farm spot around the town centre or a mill, when farms are allowed right now.
fn plan_farm(ctx: &Ctx, view: &View, placer: &mut Placer) -> Option<Choice> {
    let data = ctx.data;
    let farm = data.object_id("farm")?;
    if !stats::is_available(ctx.world, data, ctx.me, farm) {
        return None;
    }
    let max_pending = if ctx.params.max_foundations > 2 { 3 } else { 2 };
    if view.pending_count(farm) >= max_pending {
        return None;
    }
    let cap = ctx.params.villager_target[view.age.idx()] * 6 / 10 + 4;
    if view.farm_count(data) >= cap {
        return None;
    }
    let cost = stats::stats_for(ctx.world, data, ctx.me, farm).cost;
    if !ctx.can_afford_eco(&cost) {
        return None;
    }
    let tc = data.object_id("town_center");
    let mill = data.object_id("mill");
    let mut anchors: Vec<&Entity> = view
        .buildings
        .iter()
        .filter(|b| Some(b.ty) == tc || Some(b.ty) == mill)
        .copied()
        .collect();
    anchors.sort_by_key(|b| (if Some(b.ty) == tc { 0 } else { 1 }, b.id));
    for a in anchors {
        let rot = mix(ctx.tick, a.id.index);
        if let Some((x, y)) = placer.find(ctx.world, data, farm, a.pos.tile(), 2, 8, false, rot) {
            return Some(Choice::Farm(x, y));
        }
    }
    None
}

// ---------------------------------------------------------------------------------
// Research, repair, market, wonder
// ---------------------------------------------------------------------------------

/// Economic technologies in priority order with the condition that makes them worth it.
fn research(ctx: &mut Ctx, view: &View) {
    let data = ctx.data;
    let farms = view.farm_count(data);
    let vils = view.villager_count();
    let wood = view.gatherers(Resource::Wood);
    let gold = view.gatherers(Resource::Gold);
    let stone = view.gatherers(Resource::Stone);
    let tc_idle = view
        .tc
        .map(|tc| tc.queue.is_empty() && view.player.resources.food < 50)
        .unwrap_or(false);
    let list: [(&str, &str, bool); 13] = [
        ("loom", "town_center", view.age >= Age::Feudal || tc_idle),
        ("wheelbarrow", "town_center", vils >= 18),
        ("hand_cart", "town_center", vils >= 25),
        ("horse_collar", "mill", farms >= 3),
        ("heavy_plow", "mill", farms >= 6),
        ("crop_rotation", "mill", farms >= 10),
        ("double_bit_axe", "lumber_camp", wood >= 4),
        ("bow_saw", "lumber_camp", wood >= 6),
        ("two_man_saw", "lumber_camp", wood >= 8),
        ("gold_mining", "mining_camp", gold >= 3),
        ("gold_shaft_mining", "mining_camp", gold >= 5),
        ("stone_mining", "mining_camp", stone >= 2),
        ("stone_shaft_mining", "mining_camp", stone >= 3),
    ];
    let buffer = Stockpile::new(50, 50, 0, 0);
    for (tech, at, wanted) in list {
        if !wanted {
            continue;
        }
        let (Some(tech), Some(at)) = (data.tech_id(tech), data.object_id(at)) else {
            continue;
        };
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

/// Send a villager to the most damaged building when no enemy is around it.
fn repair(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    if ctx.tick.saturating_sub(mem.last_repair_tick) < 5 * crate::THINK_INTERVAL {
        return;
    }
    let data = ctx.data;
    let being_repaired: Vec<EntityId> = view
        .villagers
        .iter()
        .filter_map(|v| match v.task() {
            Task::Repair { target } => Some(*target),
            _ => None,
        })
        .collect();
    let mut worst: Option<(i32, EntityId, Vec2)> = None;
    for b in &view.buildings {
        let def = data.object(b.ty);
        if def.role == Role::Wall || being_repaired.contains(&b.id) {
            continue;
        }
        // Base hp is a cheap first filter; the exact maximum only matters for candidates.
        if b.hp * 3 >= def.hp * 2 {
            continue;
        }
        let max = stats::stats_for(ctx.world, data, ctx.me, b.ty).hp;
        if b.hp * 3 >= max * 2 || !view.enemies_near(b.pos, 10).is_empty() {
            continue;
        }
        let pct = b.hp * 100 / max.max(1);
        if worst
            .map(|(wp, wid, _)| (pct, b.id) < (wp, wid))
            .unwrap_or(true)
        {
            worst = Some((pct, b.id, b.pos));
        }
    }
    let Some((_, target, pos)) = worst else {
        return;
    };
    let units = pick_builders(ctx, view, pos, 1, false);
    if units.is_empty() {
        return;
    }
    mem.last_repair_tick = ctx.tick;
    ctx.mark_busy(&units);
    ctx.push(Command::Repair {
        units,
        target,
        queue: false,
    });
}

/// Simple market use: turn surplus stone or wood into gold and buy food in a pinch.
fn market(ctx: &mut Ctx, view: &View, mem: &mut AiMemory) {
    if ctx.params.period > 1 || !view.has(ctx.data, "market") {
        return;
    }
    let r = view.player.resources;
    // Wood starvation stalls everything: trade out of it quickly.
    let urgent = r.wood < 100 && (r.food > 1000 || r.gold > 400);
    let interval = if urgent { 4 } else { 15 };
    if ctx.tick.saturating_sub(mem.last_market_tick) < interval * crate::THINK_INTERVAL {
        return;
    }
    let cmd = if r.wood < 100 && r.gold > 400 {
        Some(Command::MarketBuy {
            resource: Resource::Wood,
        })
    } else if r.wood < 100 && r.food > 1000 {
        Some(Command::MarketSell {
            resource: Resource::Food,
        })
    } else if r.gold < 100 && r.stone > 400 {
        Some(Command::MarketSell {
            resource: Resource::Stone,
        })
    } else if r.food < 80 && r.gold > 350 {
        Some(Command::MarketBuy {
            resource: Resource::Food,
        })
    } else if r.wood > 1000 && r.gold < 150 {
        Some(Command::MarketSell {
            resource: Resource::Wood,
        })
    } else if r.stone > 900 && view.has(ctx.data, "castle") {
        Some(Command::MarketSell {
            resource: Resource::Stone,
        })
    } else {
        None
    };
    if let Some(c) = cmd {
        mem.last_market_tick = ctx.tick;
        ctx.push(c);
    }
}

/// Hard only: a wonder after 40 minutes in the Imperial Age.
fn wonder(ctx: &mut Ctx, view: &View) {
    if !ctx.params.wonder || view.age < Age::Imperial || ctx.second < 40 * 60 {
        return;
    }
    let data = ctx.data;
    let Some(w) = data.object_id("wonder") else {
        return;
    };
    if view.count(w) + view.pending_count(w) > 0 {
        return;
    }
    let mut placer = Placer::new(ctx.world, data, ctx.me, view.home);
    let rot = mix(ctx.tick, w.0 as u32);
    issue_build(
        ctx,
        view,
        &mut placer,
        w,
        view.home.tile(),
        6,
        26,
        true,
        6,
        false,
        rot,
    );
}
