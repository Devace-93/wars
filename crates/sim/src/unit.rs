//! Per-entity behaviour: movement, gathering, building, attacking, garrisoning,
//! conversion, trade; plus production queues and research.

use crate::combat;
use crate::command::RallyTarget;
use crate::fixed::{Fixed, Vec2};
use crate::ids::*;
use crate::path::find_path;
use crate::stats;
use crate::types::*;
use crate::world::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goal {
    Entity(EntityId),
    Point(Vec2),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Approach {
    Arrived,
    Moving,
    Blocked,
}

const REPATH_INTERVAL: u32 = 30;

fn goal_tile(world: &World, goal: Goal) -> Option<(i32, i32)> {
    match goal {
        Goal::Entity(id) => world.get(id).map(|e| e.pos.tile()),
        Goal::Point(p) => Some(p.tile()),
    }
}

/// Distance from the entity to the goal: centre distance for points and units, edge
/// distance for buildings/resources.
fn goal_distance(world: &World, data: &GameData, from: Vec2, goal: Goal) -> Option<Fixed> {
    match goal {
        Goal::Entity(id) => world.get(id).map(|e| world.edge_distance(data, from, e)),
        Goal::Point(p) => Some(from.dist(p)),
    }
}

/// Move the entity toward the goal, returning whether it is within `reach`.
pub fn approach(
    world: &mut World,
    data: &GameData,
    id: EntityId,
    goal: Goal,
    reach: Fixed,
) -> Approach {
    let Some(e) = world.get(id).cloned() else {
        return Approach::Blocked;
    };
    let def = data.object(e.ty);
    let st = stats::stats_for(world, data, e.owner, e.ty);
    let naval = def.is_naval;
    let pos = e.pos;
    let Some(dist) = goal_distance(world, data, pos, goal) else {
        return Approach::Blocked;
    };
    if dist <= reach {
        let e = world.get_mut(id).unwrap();
        e.path.clear();
        e.path_goal = None;
        if e.anim == 1 {
            e.anim = 0;
        }
        return Approach::Arrived;
    }
    if st.speed == Fixed::ZERO || e.packed && def.key == "trebuchet" {
        return Approach::Blocked;
    }
    let Some(gt) = goal_tile(world, goal) else {
        return Approach::Blocked;
    };
    let need_path = e.path.is_empty() || e.path_goal != Some(gt) || world.tick >= e.repath_tick;
    if need_path {
        let (sx, sy) = pos.tile();
        let reach_check = reach + Fixed::from_decimal("0.75");
        let path = match goal {
            Goal::Entity(tid) => {
                let target = world.get(tid).cloned();
                match target {
                    Some(t) => {
                        let ok = |x: i32, y: i32| {
                            world.edge_distance(data, Vec2::tile_center(x, y), &t) <= reach_check
                        };
                        find_path(&world.map, (sx, sy), gt, naval, &ok)
                    }
                    None => Vec::new(),
                }
            }
            Goal::Point(p) => {
                let (gx, gy) = p.tile();
                let ok = |x: i32, y: i32| (x, y) == (gx, gy);
                find_path(&world.map, (sx, sy), gt, naval, &ok)
            }
        };
        let tick = world.tick;
        let e = world.get_mut(id).unwrap();
        e.path = path;
        e.path_goal = Some(gt);
        e.repath_tick = tick + REPATH_INTERVAL;
        if e.path.is_empty() {
            // Already on the goal tile but not within reach: walk straight at it.
            if dist < Fixed::from_int(2) {
                let target = match goal {
                    Goal::Entity(tid) => world.get(tid).map(|t| t.pos).unwrap_or(pos),
                    Goal::Point(p) => p,
                };
                let step = st.speed.div_int(TICKS_PER_SECOND as i32);
                let (np, _) = pos.step_toward(target, step);
                let clamped = world.map.clamp(np);
                let e = world.get_mut(id).unwrap();
                e.facing = pos.facing_toward(target);
                e.pos = clamped;
                e.anim = 1;
                return Approach::Moving;
            }
            return Approach::Blocked;
        }
    }
    // Walk along the path.
    let mut budget = st.speed.div_int(TICKS_PER_SECOND as i32);
    let mut cur = pos;
    let mut facing = e.facing;
    let mut path = std::mem::take(&mut world.get_mut(id).unwrap().path);
    let mut blocked = false;
    while budget > Fixed::ZERO {
        let Some(&(nx, ny)) = path.first() else { break };
        // Re-check passability of the next tile (buildings may have appeared).
        if !world.map.passable(nx, ny, naval) {
            let is_goal_tile = match goal {
                Goal::Entity(tid) => world.map.occupant(nx, ny) == Some(tid),
                Goal::Point(_) => false,
            };
            if !is_goal_tile {
                blocked = true;
                break;
            }
        }
        let wp = Vec2::tile_center(nx, ny);
        let (np, reached) = cur.step_toward(wp, budget);
        facing = cur.facing_toward(wp);
        let consumed = cur.dist(np);
        budget -= consumed.max(Fixed(1));
        cur = np;
        if reached {
            path.remove(0);
        } else {
            break;
        }
    }
    let tick = world.tick;
    let e = world.get_mut(id).unwrap();
    e.pos = cur;
    e.facing = facing;
    e.anim = 1;
    e.anim_tick = tick;
    if blocked {
        e.path.clear();
        e.path_goal = None;
        e.repath_tick = 0;
    } else {
        e.path = path;
    }
    if let Some(d) = goal_distance(world, data, cur, goal) {
        if d <= reach {
            let e = world.get_mut(id).unwrap();
            e.path.clear();
            e.path_goal = None;
            return Approach::Arrived;
        }
    }
    Approach::Moving
}

pub fn nearest_building(
    world: &World,
    data: &GameData,
    player: PlayerId,
    from: Vec2,
    pred: &dyn Fn(&ObjectType) -> bool,
) -> Option<EntityId> {
    let mut best: Option<(Fixed, EntityId)> = None;
    for e in world.iter() {
        if !world.ally(player, e.owner) || e.build_progress < 1000 {
            continue;
        }
        let def = data.object(e.ty);
        if def.class != ObjectClass::Building || !pred(def) {
            continue;
        }
        let d = world.edge_distance(data, from, e);
        if best
            .map(|(bd, bid)| d < bd || (d == bd && e.id < bid))
            .unwrap_or(true)
        {
            best = Some((d, e.id));
        }
    }
    best.map(|(_, id)| id)
}

pub fn nearest_drop_off(
    world: &World,
    data: &GameData,
    player: PlayerId,
    from: Vec2,
    kind: GatherKind,
) -> Option<EntityId> {
    nearest_building(world, data, player, from, &|d| d.drop_off.contains(&kind))
}

/// Nearest resource of the same kind within `radius` of a point (for villagers to continue).
pub fn nearest_resource(
    world: &World,
    data: &GameData,
    from: Vec2,
    kind: GatherKind,
    radius: Fixed,
    player: PlayerId,
) -> Option<EntityId> {
    let mut best: Option<(Fixed, EntityId)> = None;
    for e in world.iter() {
        let def = data.object(e.ty);
        let Some(y) = def.yields else { continue };
        if y.kind != kind || e.resource_left <= 0 {
            continue;
        }
        if def.class == ObjectClass::Animal && !e.alerted && kind != GatherKind::Sheep {
            continue;
        }
        if def.class == ObjectClass::Building
            && (!world.ally(player, e.owner) || e.build_progress < 1000)
        {
            continue;
        }
        let d = world.edge_distance(data, from, e);
        if d > radius {
            continue;
        }
        if best
            .map(|(bd, bid)| d < bd || (d == bd && e.id < bid))
            .unwrap_or(true)
        {
            best = Some((d, e.id));
        }
    }
    best.map(|(_, id)| id)
}

/// Nearest visible enemy of `player` within `radius` of `from` that the attacker can hit.
pub fn nearest_enemy(
    world: &World,
    data: &GameData,
    player: PlayerId,
    from: Vec2,
    radius: Fixed,
    can_hit_buildings: bool,
) -> Option<EntityId> {
    let mut best: Option<(Fixed, i32, EntityId)> = None;
    for e in world.iter() {
        if e.inside.is_some() || !world.enemy(player, e.owner) {
            continue;
        }
        let def = data.object(e.ty);
        if def.class == ObjectClass::Building && !can_hit_buildings {
            continue;
        }
        if def.class != ObjectClass::Unit && def.class != ObjectClass::Building {
            continue;
        }
        let d = world.edge_distance(data, from, e);
        if d > radius {
            continue;
        }
        // Prefer units that can fight over villagers over buildings.
        let prio = if def.class == ObjectClass::Unit {
            if def.attacks.is_empty() {
                1
            } else {
                0
            }
        } else {
            2
        };
        let key = (prio, d, e.id);
        if best
            .map(|(bd, bp, bid)| key < (bp, bd, bid))
            .unwrap_or(true)
        {
            best = Some((d, prio, e.id));
        }
    }
    best.map(|(_, _, id)| id)
}

fn pop_task(world: &mut World, id: EntityId) {
    if let Some(e) = world.get_mut(id) {
        e.tasks.pop_front();
        e.path.clear();
        e.path_goal = None;
        e.work = 0;
        if e.tasks.is_empty() {
            e.anim = 0;
        }
    }
}

fn replace_task(world: &mut World, id: EntityId, t: Task) {
    if let Some(e) = world.get_mut(id) {
        e.tasks.pop_front();
        e.tasks.push_front(t);
        e.path.clear();
        e.path_goal = None;
        e.work = 0;
    }
}

pub fn update_entity(world: &mut World, data: &GameData, id: EntityId) {
    let Some(e) = world.get(id).cloned() else {
        return;
    };
    if let Some(d) = e.decay {
        if d == 0 || e.resource_left <= 0 && e.alerted {
            world.despawn(data, id);
            return;
        }
        world.get_mut(id).unwrap().decay = Some(d - 1);
    }
    let e = world.get(id).unwrap().clone();
    if e.inside.is_some() {
        return;
    }
    if e.reload > 0 {
        world.get_mut(id).unwrap().reload -= 1;
    }
    let def = data.object(e.ty);
    match def.class {
        ObjectClass::Building => update_building(world, data, id),
        ObjectClass::Unit => update_unit(world, data, id),
        ObjectClass::Animal => update_animal(world, data, id),
        _ => {}
    }
}

fn update_animal(world: &mut World, data: &GameData, id: EntityId) {
    let e = world.get(id).unwrap().clone();
    if e.alerted || e.owner != PlayerId::GAIA {
        // Carcass or captured sheep: nothing to do (sheep follow is handled by capture).
        return;
    }
    let def = data.object(e.ty);
    // Sheep get captured by the first player unit that sees them.
    if def
        .yields
        .map(|y| y.kind == GatherKind::Sheep)
        .unwrap_or(false)
        && world.tick % 10 == id.index % 10
    {
        let pos = e.pos;
        let mut captor: Option<PlayerId> = None;
        for (_, oid) in world.nearby(pos, Fixed::from_int(4)) {
            let o = world.get(oid).unwrap();
            if !o.owner.is_gaia() && data.object(o.ty).class == ObjectClass::Unit {
                captor = Some(o.owner);
                break;
            }
        }
        if let Some(p) = captor {
            change_owner(world, data, id, p);
        }
        return;
    }
    // Wild animals wander a little and fight back when hunted (boar).
    if !def.attacks.is_empty() {
        if let Task::Attack { target, .. } = e.task().clone() {
            do_attack_task(world, data, id, target, true);
            return;
        }
    }
    if world.tick % 60 == id.index % 60 {
        let dx = world.rng.range(-2, 3);
        let dy = world.rng.range(-2, 3);
        let (x, y) = e.pos.tile();
        if world.map.land_passable(x + dx, y + dy) {
            let e = world.get_mut(id).unwrap();
            e.tasks.clear();
            e.tasks.push_back(Task::Move {
                to: Vec2::tile_center(x + dx, y + dy),
                path: Vec::new(),
                attack_move: false,
                patrol_from: None,
            });
        }
    }
    if let Task::Move { to, .. } = e.task().clone() {
        if approach(world, data, id, Goal::Point(to), Fixed::from_decimal("0.2"))
            != Approach::Moving
        {
            pop_task(world, id);
        }
    }
}

fn update_building(world: &mut World, data: &GameData, id: EntityId) {
    let e = world.get(id).unwrap().clone();
    let def = data.object(e.ty);
    if e.build_progress < 1000 {
        return;
    }
    // Garrison healing.
    if world.tick % TICKS_PER_SECOND == 0
        && !e.garrisoned.is_empty()
        && (def.is_town_center || def.garrison_arrows > 0 || def.key == "castle")
    {
        let inside = e.garrisoned.clone();
        for u in inside {
            if let Some(ue) = world.get(u) {
                let max = stats::stats_for(world, data, ue.owner, ue.ty).hp;
                let ue = world.get_mut(u).unwrap();
                ue.hp = (ue.hp + 2).min(max);
            }
        }
    }
    // Defensive fire.
    if !def.attacks.is_empty() && e.reload == 0 && world.tick % 5 == id.index % 5 {
        let st = stats::stats_for(world, data, e.owner, e.ty);
        let pos = e.pos;
        let range = st.range + Fixed::from_int(def.size.0 as i32) / Fixed::from_int(2);
        if let Some(t) = nearest_enemy(world, data, e.owner, pos, range, true) {
            let tpos = world.get(t).unwrap().pos;
            if pos.dist(tpos) >= st.min_range {
                combat::attack(world, data, id, t);
            }
        }
    }
}

fn update_unit(world: &mut World, data: &GameData, id: EntityId) {
    let e = world.get(id).unwrap().clone();
    let def = data.object(e.ty);
    let task = e.task().clone();
    match task {
        Task::Idle => idle_behaviour(world, data, id),
        Task::Move {
            to,
            attack_move,
            patrol_from,
            ..
        } => {
            if attack_move
                && e.stance != Stance::NoAttack
                && world.tick % 5 == id.index % 5
                && !def.attacks.is_empty()
            {
                let st = stats::stats_for(world, data, e.owner, e.ty);
                if let Some(t) = nearest_enemy(
                    world,
                    data,
                    e.owner,
                    e.pos,
                    st.los,
                    def.role != Role::Villager,
                ) {
                    world.get_mut(id).unwrap().tasks.push_front(Task::Attack {
                        target: t,
                        chasing: true,
                    });
                    return;
                }
            }
            match approach(
                world,
                data,
                id,
                Goal::Point(to),
                Fixed::from_decimal("0.15"),
            ) {
                Approach::Moving => {}
                Approach::Arrived | Approach::Blocked => {
                    if let Some(from) = patrol_from {
                        replace_task(
                            world,
                            id,
                            Task::Move {
                                to: from,
                                path: Vec::new(),
                                attack_move: true,
                                patrol_from: Some(to),
                            },
                        );
                    } else {
                        pop_task(world, id);
                    }
                }
            }
        }
        Task::Attack { target, chasing } => do_attack_task(world, data, id, target, chasing),
        Task::Gather { target, kind } => do_gather(world, data, id, target, kind),
        Task::DropOff { drop, resume } => do_drop_off(world, data, id, drop, resume),
        Task::Build { target } => do_build(world, data, id, target),
        Task::Repair { target } => do_repair(world, data, id, target),
        Task::Garrison { target } => do_garrison(world, data, id, target),
        Task::Follow { target } | Task::Guard { target } => {
            if world.get(target).is_none() {
                pop_task(world, id);
                return;
            }
            if matches!(task, Task::Guard { .. })
                && !def.attacks.is_empty()
                && world.tick % 5 == id.index % 5
            {
                let tpos = world.get(target).unwrap().pos;
                if let Some(en) =
                    nearest_enemy(world, data, e.owner, tpos, Fixed::from_int(6), true)
                {
                    world.get_mut(id).unwrap().tasks.push_front(Task::Attack {
                        target: en,
                        chasing: true,
                    });
                    return;
                }
            }
            approach(
                world,
                data,
                id,
                Goal::Entity(target),
                Fixed::from_decimal("1.5"),
            );
        }
        Task::Convert { target } => do_convert(world, data, id, target),
        Task::Heal { target } => do_heal(world, data, id, target),
        Task::Trade {
            home,
            far,
            outbound,
        } => do_trade(world, data, id, home, far, outbound),
        Task::PickRelic { target } => {
            if world.get(target).is_none() || e.relics > 0 {
                pop_task(world, id);
                return;
            }
            if approach(
                world,
                data,
                id,
                Goal::Entity(target),
                Fixed::from_decimal("0.6"),
            ) == Approach::Arrived
            {
                world.despawn(data, target);
                world.get_mut(id).unwrap().relics = 1;
                pop_task(world, id);
                let owner = world.get(id).unwrap().owner;
                let pos = world.get(id).unwrap().pos;
                if let Some(m) = nearest_building(world, data, owner, pos, &|d| d.is_monastery) {
                    world
                        .get_mut(id)
                        .unwrap()
                        .tasks
                        .push_front(Task::DeliverRelic { target: m });
                }
            }
        }
        Task::DeliverRelic { target } => {
            let Some(m) = world.get(target).cloned() else {
                pop_task(world, id);
                return;
            };
            if e.relics == 0 || !data.object(m.ty).is_monastery {
                pop_task(world, id);
                return;
            }
            if approach(
                world,
                data,
                id,
                Goal::Entity(target),
                Fixed::from_decimal("0.6"),
            ) == Approach::Arrived
            {
                let owner = m.owner;
                world.get_mut(id).unwrap().relics = 0;
                world.get_mut(target).unwrap().relics += 1;
                world.player_mut(owner).relics_held += 1;
                pop_task(world, id);
            }
        }
        Task::Packing { ticks, unpack } => {
            if ticks == 0 {
                let e = world.get_mut(id).unwrap();
                e.packed = !unpack;
                pop_task(world, id);
            } else {
                if let Some(Task::Packing { ticks, .. }) =
                    world.get_mut(id).unwrap().tasks.front_mut()
                {
                    *ticks -= 1;
                }
            }
        }
    }
}

fn idle_behaviour(world: &mut World, data: &GameData, id: EntityId) {
    let e = world.get(id).unwrap().clone();
    let def = data.object(e.ty);
    if world.tick % 10 != id.index % 10 {
        return;
    }
    // Monks heal nearby wounded allies.
    if def.can_heal {
        let pos = e.pos;
        let owner = e.owner;
        let mut best: Option<(Fixed, EntityId)> = None;
        for (d2, oid) in world.nearby(pos, Fixed::from_int(6)) {
            let o = world.get(oid).unwrap();
            if oid == id
                || !world.ally(owner, o.owner)
                || data.object(o.ty).class != ObjectClass::Unit
                || data.object(o.ty).role == Role::Siege
            {
                continue;
            }
            let max = stats::stats_for(world, data, o.owner, o.ty).hp;
            if o.hp < max && best.map(|(bd, _)| d2 < bd).unwrap_or(true) {
                best = Some((d2, oid));
            }
        }
        if let Some((_, t)) = best {
            world
                .get_mut(id)
                .unwrap()
                .tasks
                .push_front(Task::Heal { target: t });
        }
        return;
    }
    if def.attacks.is_empty() || e.stance == Stance::NoAttack || def.is_villager {
        return;
    }
    let st = stats::stats_for(world, data, e.owner, e.ty);
    let radius = match e.stance {
        Stance::Aggressive => st.los,
        Stance::Defensive => st.range.max(Fixed::from_int(4)),
        Stance::StandGround => st.range.max(Fixed::from_decimal("1.2")),
        Stance::NoAttack => return,
    };
    if let Some(t) = nearest_enemy(
        world,
        data,
        e.owner,
        e.pos,
        radius,
        def.role != Role::Villager,
    ) {
        let chasing = e.stance == Stance::Aggressive;
        world
            .get_mut(id)
            .unwrap()
            .tasks
            .push_front(Task::Attack { target: t, chasing });
    }
}

fn do_attack_task(
    world: &mut World,
    data: &GameData,
    id: EntityId,
    target: EntityId,
    chasing: bool,
) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    if t.inside.is_some() {
        pop_task(world, id);
        return;
    }
    let e = world.get(id).unwrap().clone();
    let def = data.object(e.ty);
    if def.attacks.is_empty() {
        pop_task(world, id);
        return;
    }
    if def.key == "trebuchet" && e.packed {
        // Unpack first.
        world.get_mut(id).unwrap().tasks.push_front(Task::Packing {
            ticks: 5 * TICKS_PER_SECOND,
            unpack: true,
        });
        return;
    }
    let tdef = data.object(t.ty);
    // Dead animals are gathered, not attacked.
    if tdef.class == ObjectClass::Animal && t.decay.is_some() {
        match (tdef.yields, def.is_villager) {
            (Some(y), true) => replace_task(
                world,
                id,
                Task::Gather {
                    target,
                    kind: y.kind,
                },
            ),
            _ => pop_task(world, id),
        }
        return;
    }
    let st = stats::stats_for(world, data, e.owner, e.ty);
    let melee = st.range == Fixed::ZERO;
    let reach = if melee {
        Fixed::from_decimal("0.6")
    } else {
        st.range + Fixed::from_decimal("0.2")
    };
    let dist = world.edge_distance(data, e.pos, &t);
    // Stand ground / defensive units do not chase far.
    if !chasing && dist > reach {
        pop_task(world, id);
        return;
    }
    if dist <= reach {
        if st.min_range > Fixed::ZERO && e.pos.dist(t.pos) < st.min_range {
            // Too close: step back a little.
            let away = e.pos + (e.pos - t.pos).scale(Fixed::from_decimal("0.3"));
            let away = world.map.clamp(away);
            let e = world.get_mut(id).unwrap();
            e.tasks.push_front(Task::Move {
                to: away,
                path: Vec::new(),
                attack_move: false,
                patrol_from: None,
            });
            return;
        }
        let e = world.get_mut(id).unwrap();
        e.path.clear();
        e.path_goal = None;
        e.facing = e.pos.facing_toward(t.pos);
        if e.reload == 0 {
            combat::attack(world, data, id, target);
            // Hunting: villagers turn to gathering when the animal dies.
            if tdef.class == ObjectClass::Animal {
                if let Some(a) = world.get(target) {
                    if a.alerted && a.hp <= 1 {
                        if let Some(y) = tdef.yields {
                            replace_task(
                                world,
                                id,
                                Task::Gather {
                                    target,
                                    kind: y.kind,
                                },
                            );
                        }
                    }
                }
            }
        }
        return;
    }
    if approach(world, data, id, Goal::Entity(target), reach) == Approach::Blocked {
        pop_task(world, id)
    }
}

fn do_gather(world: &mut World, data: &GameData, id: EntityId, target: EntityId, kind: GatherKind) {
    let e = world.get(id).unwrap().clone();
    let owner = e.owner;
    let pos = e.pos;
    let capacity = if data.object(e.ty).role == Role::Fishing {
        15
    } else {
        stats::carry_capacity(world, data, owner)
    };
    // Switching resource kinds drops what we carry if it is a different kind.
    if e.carry > 0
        && e.carry_kind
            .map(|k| k.resource() != kind.resource())
            .unwrap_or(false)
    {
        let e = world.get_mut(id).unwrap();
        e.carry = 0;
        e.carry_kind = None;
        e.carry_frac = Fixed::ZERO;
    }
    let target_ok = world
        .get(target)
        .map(|t| t.resource_left > 0 || data.object(t.ty).class == ObjectClass::Animal)
        .unwrap_or(false);
    if !target_ok {
        // Look for another node of the same kind nearby, else drop off what we have.
        let around = e.last_gather_pos.unwrap_or(pos);
        if let Some(next) = nearest_resource(world, data, around, kind, Fixed::from_int(10), owner)
        {
            replace_task(world, id, Task::Gather { target: next, kind });
            return;
        }
        if e.carry > 0 {
            if let Some(drop) = nearest_drop_off(world, data, owner, pos, kind) {
                replace_task(world, id, Task::DropOff { drop, resume: None });
                return;
            }
        }
        pop_task(world, id);
        return;
    }
    let t = world.get(target).unwrap();
    let tdef = data.object(t.ty);
    // Animals must be killed first (sheep die in one hit from a villager).
    if tdef.class == ObjectClass::Animal && !t.alerted {
        let tid = t.id;
        world.get_mut(id).unwrap().tasks.push_front(Task::Attack {
            target: tid,
            chasing: true,
        });
        return;
    }
    if e.carry >= capacity {
        match nearest_drop_off(world, data, owner, pos, kind) {
            Some(drop) => replace_task(
                world,
                id,
                Task::DropOff {
                    drop,
                    resume: Some(target),
                },
            ),
            None => pop_task(world, id),
        }
        return;
    }
    let reach = if tdef.class == ObjectClass::Building {
        Fixed::from_decimal("0.3")
    } else {
        Fixed::from_decimal("0.6")
    };
    match approach(world, data, id, Goal::Entity(target), reach) {
        Approach::Arrived => {
            let rate = if data.object(e.ty).role == Role::Fishing {
                Fixed::from_decimal("0.6").div_int(TICKS_PER_SECOND as i32)
            } else {
                stats::gather_rate(world, data, owner, kind)
            };
            let tpos = world.get(target).unwrap().pos;
            let tick = world.tick;
            let e = world.get_mut(id).unwrap();
            e.anim = 3;
            e.anim_tick = tick;
            e.facing = e.pos.facing_toward(tpos);
            e.carry_kind = Some(kind);
            e.last_gather_pos = Some(e.pos);
            e.carry_frac += rate;
            if e.carry_frac >= Fixed::ONE {
                let whole = e.carry_frac.to_int();
                e.carry_frac -= Fixed::from_int(whole);
                let t = world.get_mut(target).unwrap();
                let taken = whole.min(t.resource_left).max(0);
                t.resource_left -= taken;
                let depleted = t.resource_left <= 0;
                let e = world.get_mut(id).unwrap();
                e.carry += taken;
                if depleted {
                    on_depleted(world, data, target);
                }
            }
        }
        Approach::Moving => {}
        Approach::Blocked => pop_task(world, id),
    }
}

/// A resource node ran out: trees and piles vanish, farms reseed if affordable, carcasses rot.
fn on_depleted(world: &mut World, data: &GameData, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        return;
    };
    let def = data.object(t.ty);
    if def.class == ObjectClass::Building {
        // Farm: auto-reseed when the owner can pay, otherwise it disappears.
        let cost = stats::stats_for(world, data, t.owner, t.ty).cost;
        if !t.owner.is_gaia() && world.player(t.owner).resources.covers(&cost) {
            world.player_mut(t.owner).resources.sub(&cost);
            let y = def.yields.map(|y| y.amount).unwrap_or(0);
            world.get_mut(target).unwrap().resource_left = y;
        } else {
            world.despawn(data, target);
        }
        return;
    }
    if def.class == ObjectClass::Animal {
        world.despawn(data, target);
        return;
    }
    if t.decay.is_none() {
        world.despawn(data, target);
    }
}

fn do_drop_off(
    world: &mut World,
    data: &GameData,
    id: EntityId,
    drop: EntityId,
    resume: Option<EntityId>,
) {
    let e = world.get(id).unwrap().clone();
    if world.get(drop).is_none() {
        let owner = e.owner;
        let pos = e.pos;
        let kind = e.carry_kind;
        match kind.and_then(|k| nearest_drop_off(world, data, owner, pos, k)) {
            Some(d) => replace_task(world, id, Task::DropOff { drop: d, resume }),
            None => pop_task(world, id),
        }
        return;
    }
    if approach(
        world,
        data,
        id,
        Goal::Entity(drop),
        Fixed::from_decimal("0.4"),
    ) == Approach::Arrived
    {
        let e = world.get_mut(id).unwrap();
        let carry = e.carry;
        let kind = e.carry_kind;
        e.carry = 0;
        e.carry_frac = Fixed::ZERO;
        let owner = e.owner;
        if let Some(k) = kind {
            let p = world.player_mut(owner);
            *p.resources.get_mut(k.resource()) += carry;
            *p.gathered.get_mut(k.resource()) += carry;
        }
        pop_task(world, id);
        match (resume, kind) {
            (Some(r), Some(k)) if world.get(r).is_some() => {
                world
                    .get_mut(id)
                    .unwrap()
                    .tasks
                    .push_front(Task::Gather { target: r, kind: k });
            }
            (_, Some(k)) => {
                let around = world
                    .get(id)
                    .unwrap()
                    .last_gather_pos
                    .unwrap_or(world.get(id).unwrap().pos);
                if let Some(next) =
                    nearest_resource(world, data, around, k, Fixed::from_int(10), owner)
                {
                    world.get_mut(id).unwrap().tasks.push_front(Task::Gather {
                        target: next,
                        kind: k,
                    });
                }
            }
            _ => {}
        }
    }
}

fn do_build(world: &mut World, data: &GameData, id: EntityId, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    if t.build_progress >= 1000 {
        pop_task(world, id);
        // Farms: start working on them.
        if data
            .object(t.ty)
            .yields
            .map(|y| y.kind == GatherKind::Farm)
            .unwrap_or(false)
        {
            world.get_mut(id).unwrap().tasks.push_front(Task::Gather {
                target,
                kind: GatherKind::Farm,
            });
        }
        return;
    }
    if approach(
        world,
        data,
        id,
        Goal::Entity(target),
        Fixed::from_decimal("0.4"),
    ) == Approach::Arrived
    {
        let bst = stats::stats_for(world, data, t.owner, t.ty);
        let builder_rate = stats::stats_for(
            world,
            data,
            world.get(id).unwrap().owner,
            world.get(id).unwrap().ty,
        )
        .work_rate_pct;
        let total = bst.train_ticks.max(1) as i32;
        // Per-mille per tick for one builder, scaled by work rate; extra builders add 2/3 each.
        let builders = world
            .iter()
            .filter(|b| {
                matches!(b.task(), Task::Build { target: tt } if *tt == target) && b.id != id
            })
            .count() as i32;
        let share = 1000 * builder_rate / 100 / total;
        let share = if builders > 0 {
            share.max(1) * 3 / (builders + 3) + 1
        } else {
            share.max(1)
        };
        let max_hp = bst.hp;
        let tpos = t.pos;
        let tick = world.tick;
        let e = world.get_mut(id).unwrap();
        e.anim = 3;
        e.anim_tick = tick;
        e.facing = e.pos.facing_toward(tpos);
        let b = world.get_mut(target).unwrap();
        b.build_progress = (b.build_progress + share).min(1000);
        b.hp = (max_hp * b.build_progress / 1000).max(1);
        if b.build_progress >= 1000 {
            complete_building(world, data, target);
        }
    }
}

pub fn complete_building(world: &mut World, data: &GameData, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        return;
    };
    let st = stats::stats_for(world, data, t.owner, t.ty);
    let owner = t.owner;
    let tick = world.tick;
    let b = world.get_mut(target).unwrap();
    let was_complete = b.build_progress >= 1000 && b.completed_tick.is_some();
    b.build_progress = 1000;
    b.hp = st.hp;
    b.completed_tick = Some(tick);
    if !was_complete && !owner.is_gaia() {
        let p = world.player_mut(owner);
        p.pop_support += st.pop_support;
        p.buildings_built += 1;
        world.push_event(Event::BuildingComplete {
            player: owner,
            entity: target,
        });
    }
}

fn do_repair(world: &mut World, data: &GameData, id: EntityId, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    let st = stats::stats_for(world, data, t.owner, t.ty);
    if t.hp >= st.hp {
        pop_task(world, id);
        return;
    }
    if approach(
        world,
        data,
        id,
        Goal::Entity(target),
        Fixed::from_decimal("0.4"),
    ) == Approach::Arrived
    {
        let owner = world.get(id).unwrap().owner;
        let per_tick = (st.hp / (st.train_ticks.max(1) as i32)).max(1);
        // Repairing costs half the wood/stone of the object, charged in small steps.
        let cost = Stockpile::new(0, st.cost.wood / 2, 0, st.cost.stone / 2);
        let step_cost = Stockpile::new(
            0,
            (cost.wood * per_tick / st.hp.max(1)).max(if cost.wood > 0 { 0 } else { 0 }),
            0,
            cost.stone * per_tick / st.hp.max(1),
        );
        let e = world.get_mut(id).unwrap();
        e.anim = 3;
        e.work += 1;
        let work = e.work;
        let p = world.player_mut(owner);
        // Charge one unit every few ticks so it adds up to roughly half the cost.
        let charge = if work % 5 == 0 {
            Stockpile::new(
                0,
                (cost.wood * 5 * per_tick / st.hp.max(1)).max(if cost.wood > 0 { 1 } else { 0 }),
                0,
                (cost.stone * 5 * per_tick / st.hp.max(1)).max(if cost.stone > 0 { 1 } else { 0 }),
            )
        } else {
            Stockpile::ZERO
        };
        let _ = step_cost;
        if !p.resources.covers(&charge) {
            world.push_event(Event::NotEnough {
                player: owner,
                resource: if charge.wood > 0 {
                    Resource::Wood
                } else {
                    Resource::Stone
                },
            });
            pop_task(world, id);
            return;
        }
        p.resources.sub(&charge);
        let b = world.get_mut(target).unwrap();
        b.hp = (b.hp + per_tick).min(st.hp);
    }
}

fn do_garrison(world: &mut World, data: &GameData, id: EntityId, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    let tdef = data.object(t.ty);
    let st = stats::stats_for(world, data, t.owner, t.ty);
    let e = world.get(id).unwrap().clone();
    let edef = data.object(e.ty);
    let cap = if tdef.carries_units > 0 {
        tdef.carries_units
    } else {
        st.garrison_capacity
    };
    if cap == 0
        || t.garrisoned.len() as i32 >= cap
        || t.build_progress < 1000
        || edef.role == Role::Siege && !tdef.is_naval
        || edef.is_naval
    {
        pop_task(world, id);
        return;
    }
    let reach = if tdef.is_naval {
        Fixed::from_decimal("1.2")
    } else {
        Fixed::from_decimal("0.5")
    };
    if approach(world, data, id, Goal::Entity(target), reach) == Approach::Arrived {
        pop_task(world, id);
        let e = world.get_mut(id).unwrap();
        e.inside = Some(target);
        e.tasks.clear();
        e.anim = 0;
        world.get_mut(target).unwrap().garrisoned.push(id);
    }
}

pub fn ungarrison_all(world: &mut World, data: &GameData, building: EntityId) {
    let Some(b) = world.get(building).cloned() else {
        return;
    };
    let (tx, ty, w, h) = world.footprint(data, &b);
    let naval_container = data.object(b.ty).is_naval;
    let inside = b.garrisoned.clone();
    world.get_mut(building).unwrap().garrisoned.clear();
    let mut spots: Vec<(i32, i32)> = Vec::new();
    for yy in ty - 1..=ty + h {
        for xx in tx - 1..=tx + w {
            let edge = yy == ty - 1 || yy == ty + h || xx == tx - 1 || xx == tx + w;
            if edge && world.map.land_passable(xx, yy) {
                spots.push((xx, yy));
            }
        }
    }
    if naval_container {
        // Transports unload onto adjacent land only.
        spots.retain(|(x, y)| world.map.terrain(*x, *y).land_passable());
        if spots.is_empty() {
            // put them back
            world.get_mut(building).unwrap().garrisoned = inside;
            return;
        }
    }
    for (i, u) in inside.iter().enumerate() {
        let spot = if spots.is_empty() {
            b.pos.tile()
        } else {
            spots[i % spots.len()]
        };
        if let Some(e) = world.get_mut(*u) {
            e.inside = None;
            e.pos = Vec2::tile_center(spot.0, spot.1);
            e.tasks.clear();
        }
    }
}

pub fn drop_relic(world: &mut World, data: &GameData, id: EntityId) {
    let Some(e) = world.get_any(id).cloned() else {
        return;
    };
    if e.relics == 0 {
        return;
    }
    let (x, y) = e.pos.tile();
    world.get_mut(id).unwrap().relics = 0;
    if let Some(rty) = data.object_id("relic") {
        if let Some((sx, sy)) = world.map.nearest_passable(x, y, false, 6) {
            world.spawn(data, PlayerId::GAIA, rty, Vec2::from_tile(sx, sy), false);
        }
    }
}

fn do_convert(world: &mut World, data: &GameData, id: EntityId, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    let e = world.get(id).unwrap().clone();
    if world.ally(e.owner, t.owner) || t.owner.is_gaia() {
        pop_task(world, id);
        return;
    }
    let tdef = data.object(t.ty);
    let me = world.player(e.owner);
    let redemption = data
        .tech_id("redemption")
        .map(|t| me.has_tech(t))
        .unwrap_or(false);
    let atonement = data
        .tech_id("atonement")
        .map(|t| me.has_tech(t))
        .unwrap_or(false);
    if tdef.class == ObjectClass::Building && !redemption
        || tdef.is_town_center
        || tdef.is_wonder
        || tdef.is_monastery
        || tdef.key == "castle"
    {
        pop_task(world, id);
        return;
    }
    if tdef.role == Role::Siege && !redemption || tdef.can_convert && !atonement {
        pop_task(world, id);
        return;
    }
    if e.reload > 0 {
        // Resting faith after a conversion: wait near the target.
        approach(world, data, id, Goal::Entity(target), Fixed::from_int(9));
        return;
    }
    let st = stats::stats_for(world, data, e.owner, e.ty);
    let range = st.range.max(Fixed::from_int(9));
    if approach(world, data, id, Goal::Entity(target), range) == Approach::Arrived {
        let resist = stats::stats_for(world, data, t.owner, t.ty).conversion_resist;
        let tpos = t.pos;
        let e = world.get_mut(id).unwrap();
        e.anim = 2;
        e.facing = e.pos.facing_toward(tpos);
        e.work += 1;
        let work = e.work;
        // 4–10 s base, plus resistance, decided by a roll every second.
        let min_ticks = 4 * TICKS_PER_SECOND + (resist as u32) * TICKS_PER_SECOND;
        if work >= min_ticks && work % TICKS_PER_SECOND == 0 {
            let roll = world.rng.chance(28);
            if roll || work >= min_ticks + 6 * TICKS_PER_SECOND {
                let new_owner = world.get(id).unwrap().owner;
                let old = world.get(target).unwrap().owner;
                change_owner(world, data, target, new_owner);
                world.push_event(Event::Converted {
                    entity: target,
                    from: old,
                    to: new_owner,
                });
                world.player_mut(new_owner).conversions += 1;
                let e = world.get_mut(id).unwrap();
                e.reload = 60 * TICKS_PER_SECOND; // faith recharge
                pop_task(world, id);
            }
        }
    }
}

pub fn change_owner(world: &mut World, data: &GameData, id: EntityId, new_owner: PlayerId) {
    let Some(e) = world.get(id).cloned() else {
        return;
    };
    if new_owner.idx() >= world.players.len() {
        return;
    }
    let old = e.owner;
    let def = data.object(e.ty);
    let old_stats = stats::stats_for(world, data, old, e.ty);
    let new_stats = stats::stats_for(world, data, new_owner, e.ty);
    let complete = e.build_progress >= 1000;
    let hp_pct = e.hp * 1000 / old_stats.hp.max(1);
    if !old.is_gaia() {
        let p = world.player_mut(old);
        if def.class == ObjectClass::Unit {
            p.pop -= old_stats.population;
        }
        if complete {
            p.pop_support -= old_stats.pop_support;
        }
    }
    if !new_owner.is_gaia() {
        let p = world.player_mut(new_owner);
        if def.class == ObjectClass::Unit {
            p.pop += new_stats.population;
        }
        if complete {
            p.pop_support += new_stats.pop_support;
        }
    }
    let e = world.get_mut(id).unwrap();
    e.owner = new_owner;
    e.hp = (new_stats.hp * hp_pct / 1000).max(1);
    e.tasks.clear();
    e.path.clear();
    e.path_goal = None;
}

fn do_heal(world: &mut World, data: &GameData, id: EntityId, target: EntityId) {
    let Some(t) = world.get(target).cloned() else {
        pop_task(world, id);
        return;
    };
    let max = stats::stats_for(world, data, t.owner, t.ty).hp;
    if t.hp >= max {
        pop_task(world, id);
        return;
    }
    if approach(world, data, id, Goal::Entity(target), Fixed::from_int(4)) == Approach::Arrived {
        let e = world.get(id).unwrap().clone();
        let rate = stats::stats_for(world, data, e.owner, e.ty)
            .heal_rate
            .max(1);
        let tpos = t.pos;
        let e = world.get_mut(id).unwrap();
        e.anim = 3;
        e.facing = e.pos.facing_toward(tpos);
        if world.tick % (TICKS_PER_SECOND / (rate as u32).clamp(1, TICKS_PER_SECOND)) == 0 {
            let t = world.get_mut(target).unwrap();
            t.hp = (t.hp + 1).min(max);
        }
    }
}

fn do_trade(
    world: &mut World,
    data: &GameData,
    id: EntityId,
    home: EntityId,
    far: EntityId,
    outbound: bool,
) {
    if world.get(home).is_none() || world.get(far).is_none() {
        pop_task(world, id);
        return;
    }
    let goal = if outbound { far } else { home };
    if approach(
        world,
        data,
        id,
        Goal::Entity(goal),
        Fixed::from_decimal("0.5"),
    ) == Approach::Arrived
    {
        let hpos = world.get(home).unwrap().pos;
        let fpos = world.get(far).unwrap().pos;
        let dist = hpos.dist(fpos);
        if outbound {
            // Gold grows with distance; capped so a cross-map trip is worth ~100.
            let gold = (dist.mul_int(3).div_int(4)).to_int().clamp(1, 120);
            let e = world.get_mut(id).unwrap();
            e.carry = gold;
            e.carry_kind = Some(GatherKind::Gold);
            replace_task(
                world,
                id,
                Task::Trade {
                    home,
                    far,
                    outbound: false,
                },
            );
        } else {
            let e = world.get_mut(id).unwrap();
            let gold = e.carry;
            e.carry = 0;
            e.carry_kind = None;
            let owner = e.owner;
            let p = world.player_mut(owner);
            p.resources.gold += gold;
            p.trade_profit += gold;
            replace_task(
                world,
                id,
                Task::Trade {
                    home,
                    far,
                    outbound: true,
                },
            );
        }
    }
}

/// Production queues and research for every building.
pub fn update_production(world: &mut World, data: &GameData) {
    let ids: Vec<EntityId> = world
        .iter()
        .filter(|e| !e.queue.is_empty() && e.build_progress >= 1000)
        .map(|e| e.id)
        .collect();
    for id in ids {
        let Some(b) = world.get(id).cloned() else {
            continue;
        };
        let Some(item) = b.queue.front().copied() else {
            continue;
        };
        let owner = b.owner;
        let bst = stats::stats_for(world, data, owner, b.ty);
        let rate = bst.work_rate_pct.max(1) as u32;
        match item {
            QueueItem::Unit(ty) => {
                let st = stats::stats_for(world, data, owner, ty);
                let progress = b.queue_progress
                    + rate / 100
                    + if (world.tick * rate / 100) % 100 < rate % 100 {
                        1
                    } else {
                        0
                    };
                if progress < st.train_ticks {
                    world.get_mut(id).unwrap().queue_progress = progress;
                    continue;
                }
                // Population check.
                let p = world.player(owner);
                let cap = stats::pop_cap(world, data, owner).min(p.pop_support);
                if p.pop + st.population > cap {
                    if world.tick % (5 * TICKS_PER_SECOND) == 0 {
                        world.push_event(Event::PopCapReached { player: owner });
                    }
                    world.get_mut(id).unwrap().queue_progress = st.train_ticks;
                    continue;
                }
                let (tx, ty_, w, h) = world.footprint(data, &b);
                let def = data.object(ty);
                let spawn_tile = if def.is_naval {
                    // Water next to the dock.
                    let mut found = None;
                    'outer: for r in 1..=3 {
                        for yy in ty_ - r..=ty_ + h - 1 + r {
                            for xx in tx - r..=tx + w - 1 + r {
                                if world.map.water_passable(xx, yy) {
                                    found = Some((xx, yy));
                                    break 'outer;
                                }
                            }
                        }
                    }
                    found
                } else {
                    // Below the building, then anywhere around.
                    world.map.nearest_passable(tx + w / 2, ty_ + h, false, 8)
                };
                let Some((sx, sy)) = spawn_tile else { continue };
                let nid = world.spawn(data, owner, ty, Vec2::tile_center(sx, sy), false);
                let bb = world.get_mut(id).unwrap();
                bb.queue.pop_front();
                bb.queue_progress = 0;
                if let Some(nid) = nid {
                    world.player_mut(owner).units_trained += 1;
                    world.push_event(Event::UnitTrained {
                        player: owner,
                        entity: nid,
                    });
                    apply_rally(world, data, nid, b.rally);
                }
            }
            QueueItem::Tech(t) => {
                let (_, ticks) = stats::tech_cost(world, data, owner, t);
                let progress = b.queue_progress + rate / 100;
                if progress < ticks {
                    world.get_mut(id).unwrap().queue_progress = progress;
                    continue;
                }
                let bb = world.get_mut(id).unwrap();
                bb.queue.pop_front();
                bb.queue_progress = 0;
                world.player_mut(owner).researching.remove(&t);
                grant_tech(world, data, owner, t);
            }
        }
    }
}

fn apply_rally(world: &mut World, data: &GameData, unit: EntityId, rally: Option<RallyTarget>) {
    match rally {
        Some(RallyTarget::Point(p)) => {
            world.get_mut(unit).unwrap().tasks.push_back(Task::Move {
                to: p,
                path: Vec::new(),
                attack_move: false,
                patrol_from: None,
            });
        }
        Some(RallyTarget::Entity(t)) => {
            let Some(te) = world.get(t) else { return };
            let tdef = data.object(te.ty);
            let udef = data.object(world.get(unit).unwrap().ty);
            let task = if let Some(y) = tdef.yields {
                if udef.is_villager || udef.role == Role::Fishing {
                    Task::Gather {
                        target: t,
                        kind: y.kind,
                    }
                } else {
                    Task::Move {
                        to: te.pos,
                        path: Vec::new(),
                        attack_move: false,
                        patrol_from: None,
                    }
                }
            } else if tdef.class == ObjectClass::Building
                && te.build_progress < 1000
                && udef.is_villager
            {
                Task::Build { target: t }
            } else if tdef.class == ObjectClass::Building
                && world.ally(te.owner, world.get(unit).unwrap().owner)
            {
                Task::Garrison { target: t }
            } else {
                Task::Follow { target: t }
            };
            world.get_mut(unit).unwrap().tasks.push_back(task);
        }
        _ => {}
    }
}

/// Grant a technology to a player and apply its one-off effects.
pub fn grant_tech(world: &mut World, data: &GameData, player: PlayerId, tech: TechId) {
    if player.idx() >= world.players.len() || world.player(player).has_tech(tech) {
        return;
    }
    let def = data.tech(tech).clone();
    // Snapshot HP percentages so stat changes keep units proportionally healthy.
    let before: Vec<(EntityId, i32, i32)> = world
        .entities_of(player)
        .map(|e| (e.id, e.hp, stats::stats_for(world, data, player, e.ty).hp))
        .collect();
    world.player_mut(player).techs.insert(tech);
    world.push_event(Event::ResearchComplete { player, tech });
    if let Some(age) = def.advances_to {
        world.player_mut(player).age = age;
        world.push_event(Event::AgeAdvanced { player, age });
    }
    for eff in &def.effects {
        match eff {
            Effect::Upgrade { from, to } => {
                let (f, t) = (data.obj(from), data.obj(to));
                let ids: Vec<EntityId> = world
                    .entities_of(player)
                    .filter(|e| e.ty == f)
                    .map(|e| e.id)
                    .collect();
                for id in ids {
                    if let Some(e) = world.get_mut(id) {
                        e.ty = t;
                    }
                    for b in world.entities.iter_mut().flatten() {
                        for q in b.queue.iter_mut() {
                            if *q == QueueItem::Unit(f) {
                                *q = QueueItem::Unit(t);
                            }
                        }
                    }
                }
            }
            Effect::Resources {
                food,
                wood,
                gold,
                stone,
            } => {
                world
                    .player_mut(player)
                    .resources
                    .add(&Stockpile::new(*food, *wood, *gold, *stone));
            }
            Effect::FreeTech { tech: t } => {
                if let Some(tid) = data.tech_id(t) {
                    grant_tech(world, data, player, tid);
                }
            }
            _ => {}
        }
    }
    // Re-scale HP after the change (both for upgrades and +HP techs).
    for (id, hp, old_max) in before {
        if let Some(e) = world.get(id) {
            let new_max = stats::stats_for(world, data, player, e.ty).hp;
            if new_max != old_max && e.build_progress >= 1000 {
                let nh = (hp as i64 * new_max as i64 / old_max.max(1) as i64) as i32;
                world.get_mut(id).unwrap().hp = nh.max(1);
            }
        }
    }
    // Pop support may change with techs; recompute from scratch for safety.
    let support: i32 = world
        .entities_of(player)
        .filter(|e| e.build_progress >= 1000)
        .map(|e| stats::stats_for(world, data, player, e.ty).pop_support)
        .sum();
    world.player_mut(player).pop_support = support;
}
