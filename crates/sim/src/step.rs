//! One simulation tick: apply the scheduled commands, then advance every entity.

use crate::command::{Command, PlayerCommand, RallyTarget};
use crate::fixed::{Fixed, Vec2};
use crate::ids::*;
use crate::stats;
use crate::types::*;
use crate::world::*;

/// Advance the world by one tick with the given commands (already ordered by the server).
pub fn step(world: &mut World, data: &GameData, commands: &[PlayerCommand]) {
    world.events.clear();
    if world.game_over {
        return;
    }
    world.tick += 1;
    for pc in commands {
        apply_command(world, data, pc.player, &pc.command);
    }
    crate::combat::resolve_shots(world, data);
    let ids = world.ids();
    for id in ids {
        if world.get(id).is_none() {
            continue;
        }
        crate::unit::update_entity(world, data, id);
    }
    crate::unit::update_production(world, data);
    tick_relics_and_farms(world, data);
    if world.tick % 5 == 0 {
        crate::vision::update_vision(world, data);
    }
    if world.tick % 20 == 0 {
        update_scores(world, data);
    }
    crate::trigger_exec::run_triggers(world, data);
    check_victory(world, data);
}

/// Validate that every id belongs to the player and is alive and not garrisoned.
fn own_units(world: &World, player: PlayerId, units: &[EntityId]) -> Vec<EntityId> {
    units
        .iter()
        .copied()
        .filter(|id| {
            world
                .get(*id)
                .map(|e| e.owner == player && e.inside.is_none())
                .unwrap_or(false)
        })
        .collect()
}

fn set_task(world: &mut World, id: EntityId, task: Task, queue: bool) {
    if let Some(e) = world.get_mut(id) {
        if !queue {
            e.tasks.clear();
            e.path.clear();
            e.path_goal = None;
        }
        e.tasks.push_back(task);
    }
}

/// Spread a group over a small grid around `to` so they do not stack on one point.
fn group_offsets(n: usize) -> Vec<Vec2> {
    let mut v = Vec::with_capacity(n);
    let side = ((n as f64).sqrt().ceil() as i32).max(1);
    let spacing = Fixed::from_decimal("0.8");
    for i in 0..n as i32 {
        let gx = i % side;
        let gy = i / side;
        let off_x =
            (Fixed::from_int(gx) - Fixed::from_int(side - 1) / Fixed::from_int(2)) * spacing;
        let off_y =
            (Fixed::from_int(gy) - Fixed::from_int(side - 1) / Fixed::from_int(2)) * spacing;
        v.push(Vec2::new(off_x, off_y));
    }
    v
}

pub fn apply_command(world: &mut World, data: &GameData, player: PlayerId, cmd: &Command) {
    if player.idx() >= world.players.len() {
        return;
    }
    if !world.player(player).alive && !matches!(cmd, Command::Chat { .. }) {
        return;
    }
    match cmd {
        Command::Move { units, to, queue } | Command::AttackMove { units, to, queue } => {
            let attack_move = matches!(cmd, Command::AttackMove { .. });
            let units = own_units(world, player, units);
            let offs = group_offsets(units.len());
            for (i, id) in units.iter().enumerate() {
                let dest = world.map.clamp(*to + offs[i]);
                let def = data.object(world.get(*id).unwrap().ty);
                if def.speed == Fixed::ZERO {
                    continue;
                }
                let dest = if units.len() == 1 {
                    world.map.clamp(*to)
                } else {
                    dest
                };
                set_task(
                    world,
                    *id,
                    Task::Move {
                        to: dest,
                        path: Vec::new(),
                        attack_move,
                        patrol_from: None,
                    },
                    *queue,
                );
            }
        }
        Command::Patrol { units, to } => {
            for id in own_units(world, player, units) {
                let from = world.get(id).unwrap().pos;
                set_task(
                    world,
                    id,
                    Task::Move {
                        to: *to,
                        path: Vec::new(),
                        attack_move: true,
                        patrol_from: Some(from),
                    },
                    false,
                );
            }
        }
        Command::Attack {
            units,
            target,
            queue,
        } => {
            if world.get(*target).is_none() {
                return;
            }
            for id in own_units(world, player, units) {
                let e = world.get(id).unwrap();
                let def = data.object(e.ty);
                // Monks sent at a relic pick it up.
                let tdef = data.object(world.get(*target).unwrap().ty);
                if tdef.class == ObjectClass::Relic {
                    if def.can_convert {
                        set_task(world, id, Task::PickRelic { target: *target }, *queue);
                    }
                    continue;
                }
                if def.attacks.is_empty() {
                    continue;
                }
                if def.is_villager {
                    // Villagers sent at a resource gather; at an animal they hunt.
                    let t = world.get(*target).unwrap();
                    let tdef = data.object(t.ty);
                    if tdef.class == ObjectClass::Resource || tdef.class == ObjectClass::Animal {
                        if let Some(y) = tdef.yields {
                            set_task(
                                world,
                                id,
                                Task::Gather {
                                    target: *target,
                                    kind: y.kind,
                                },
                                *queue,
                            );
                            continue;
                        }
                    }
                }
                set_task(
                    world,
                    id,
                    Task::Attack {
                        target: *target,
                        chasing: true,
                    },
                    *queue,
                );
            }
        }
        Command::Follow { units, target } => {
            for id in own_units(world, player, units) {
                set_task(world, id, Task::Follow { target: *target }, false);
            }
        }
        Command::Guard { units, target } => {
            for id in own_units(world, player, units) {
                set_task(world, id, Task::Guard { target: *target }, false);
            }
        }
        Command::Stop { units } => {
            for id in own_units(world, player, units) {
                if let Some(e) = world.get_mut(id) {
                    e.tasks.clear();
                    e.path.clear();
                    e.path_goal = None;
                }
            }
        }
        Command::Gather {
            units,
            target,
            queue,
        } => {
            let Some(t) = world.get(*target) else { return };
            let tdef = data.object(t.ty);
            let Some(y) = tdef.yields else { return };
            // Farms must be complete and ours/allied.
            for id in own_units(world, player, units) {
                let e = world.get(id).unwrap();
                let def = data.object(e.ty);
                let ok = if y.kind == GatherKind::Fish {
                    def.role == Role::Fishing
                } else {
                    def.is_villager
                };
                if ok {
                    set_task(
                        world,
                        id,
                        Task::Gather {
                            target: *target,
                            kind: y.kind,
                        },
                        *queue,
                    );
                }
            }
        }
        Command::Build {
            units,
            building,
            x,
            y,
            queue,
        } => {
            let bdef = data.object(*building);
            if bdef.class != ObjectClass::Building {
                return;
            }
            if !stats::is_available(world, data, player, *building) {
                return;
            }
            let units = own_units(world, player, units)
                .into_iter()
                .filter(|id| data.object(world.get(*id).unwrap().ty).is_villager)
                .collect::<Vec<_>>();
            if units.is_empty() {
                return;
            }
            let st = stats::stats_for(world, data, player, *building);
            if !world.player(player).resources.covers(&st.cost) {
                let missing = Resource::ALL
                    .iter()
                    .copied()
                    .find(|r| world.player(player).resources.get(*r) < st.cost.get(*r));
                if let Some(r) = missing {
                    world.push_event(Event::NotEnough {
                        player,
                        resource: r,
                    });
                }
                return;
            }
            let pos = Vec2::from_tile(*x, *y);
            let Some(fid) = world.spawn(data, player, *building, pos, true) else {
                return;
            };
            world.player_mut(player).resources.sub(&st.cost);
            for id in units {
                set_task(world, id, Task::Build { target: fid }, *queue);
            }
        }
        Command::Repair {
            units,
            target,
            queue,
        } => {
            let Some(t) = world.get(*target) else { return };
            if !world.ally(player, t.owner) {
                return;
            }
            let tdef = data.object(t.ty);
            let task = if t.build_progress < 1000 {
                Task::Build { target: *target }
            } else {
                Task::Repair { target: *target }
            };
            if tdef.class != ObjectClass::Building && tdef.role != Role::Siege && !tdef.is_naval {
                return;
            }
            for id in own_units(world, player, units) {
                if data.object(world.get(id).unwrap().ty).is_villager {
                    set_task(world, id, task.clone(), *queue);
                }
            }
        }
        Command::Train { building, unit } => {
            let Some(b) = world.get(*building) else {
                return;
            };
            if b.owner != player || b.build_progress < 1000 {
                return;
            }
            let bty = b.ty;
            let unit = stats::resolve_upgrade(world, data, player, *unit);
            let trains = data
                .trains_at
                .get(&bty)
                .map(|v| {
                    v.iter()
                        .any(|t| stats::resolve_upgrade(world, data, player, *t) == unit)
                })
                .unwrap_or(false);
            if !trains || !stats::is_available(world, data, player, unit) {
                return;
            }
            if b.queue.len() >= 15 {
                return;
            }
            let st = stats::stats_for(world, data, player, unit);
            if !world.player(player).resources.covers(&st.cost) {
                let missing = Resource::ALL
                    .iter()
                    .copied()
                    .find(|r| world.player(player).resources.get(*r) < st.cost.get(*r));
                if let Some(r) = missing {
                    world.push_event(Event::NotEnough {
                        player,
                        resource: r,
                    });
                }
                return;
            }
            world.player_mut(player).resources.sub(&st.cost);
            world
                .get_mut(*building)
                .unwrap()
                .queue
                .push_back(QueueItem::Unit(unit));
        }
        Command::Research { building, tech } => {
            let Some(b) = world.get(*building) else {
                return;
            };
            if b.owner != player || b.build_progress < 1000 {
                return;
            }
            let bty = b.ty;
            let here = data
                .techs_at
                .get(&bty)
                .map(|v| v.contains(tech))
                .unwrap_or(false);
            if !here || !stats::tech_available(world, data, player, *tech) {
                return;
            }
            if b.queue.len() >= 15 {
                return;
            }
            let (cost, _) = stats::tech_cost(world, data, player, *tech);
            if !world.player(player).resources.covers(&cost) {
                let missing = Resource::ALL
                    .iter()
                    .copied()
                    .find(|r| world.player(player).resources.get(*r) < cost.get(*r));
                if let Some(r) = missing {
                    world.push_event(Event::NotEnough {
                        player,
                        resource: r,
                    });
                }
                return;
            }
            world.player_mut(player).resources.sub(&cost);
            world.player_mut(player).researching.insert(*tech);
            world
                .get_mut(*building)
                .unwrap()
                .queue
                .push_back(QueueItem::Tech(*tech));
        }
        Command::CancelQueue { building, index } => {
            let Some(b) = world.get(*building) else {
                return;
            };
            if b.owner != player || *index >= b.queue.len() {
                return;
            }
            let item = b.queue[*index];
            let refund = match item {
                QueueItem::Unit(u) => stats::stats_for(world, data, player, u).cost,
                QueueItem::Tech(t) => stats::tech_cost(world, data, player, t).0,
            };
            let b = world.get_mut(*building).unwrap();
            b.queue.remove(*index);
            if *index == 0 {
                b.queue_progress = 0;
            }
            if let QueueItem::Tech(t) = item {
                world.player_mut(player).researching.remove(&t);
            }
            world.player_mut(player).resources.add(&refund);
        }
        Command::Delete { entity } => {
            let Some(e) = world.get(*entity) else { return };
            if e.owner != player {
                return;
            }
            let ty = e.ty;
            if e.build_progress < 1000 {
                // Refund the foundation fully.
                let st = stats::stats_for(world, data, player, ty);
                world.player_mut(player).resources.add(&st.cost);
            }
            crate::combat::kill(world, data, *entity, None);
        }
        Command::SetRally { buildings, to } => {
            for id in buildings {
                if let Some(b) = world.get_mut(*id) {
                    if b.owner == player {
                        b.rally = match to {
                            RallyTarget::None => None,
                            other => Some(*other),
                        };
                    }
                }
            }
        }
        Command::Garrison { units, target } => {
            let Some(t) = world.get(*target) else { return };
            if !world.ally(player, t.owner) {
                return;
            }
            for id in own_units(world, player, units) {
                set_task(world, id, Task::Garrison { target: *target }, false);
            }
        }
        Command::Ungarrison { building } => {
            let Some(b) = world.get(*building) else {
                return;
            };
            if b.owner != player {
                return;
            }
            crate::unit::ungarrison_all(world, data, *building);
        }
        Command::Unload { units } => {
            for id in own_units(world, player, units) {
                let e = world.get(id).unwrap();
                if e.relics > 0 {
                    crate::unit::drop_relic(world, data, id);
                } else {
                    crate::unit::ungarrison_all(world, data, id);
                }
            }
        }
        Command::SetStance { units, stance } => {
            for id in own_units(world, player, units) {
                world.get_mut(id).unwrap().stance = *stance;
            }
        }
        Command::SetFormation { units, formation } => {
            for id in own_units(world, player, units) {
                world.get_mut(id).unwrap().formation = *formation;
            }
        }
        Command::Convert { units, target } => {
            let Some(t) = world.get(*target) else { return };
            if world.ally(player, t.owner) {
                return;
            }
            for id in own_units(world, player, units) {
                if data.object(world.get(id).unwrap().ty).can_convert {
                    set_task(world, id, Task::Convert { target: *target }, false);
                }
            }
        }
        Command::Trade { units, target } => {
            let Some(t) = world.get(*target) else { return };
            let tdef = data.object(t.ty);
            if !(tdef.is_market || tdef.is_dock) || t.build_progress < 1000 {
                return;
            }
            for id in own_units(world, player, units) {
                let e = world.get(id).unwrap();
                let def = data.object(e.ty);
                if def.role != Role::Trade {
                    continue;
                }
                // Home = nearest own market (or dock for cogs).
                let want_dock = def.is_naval;
                let home = crate::unit::nearest_building(world, data, player, e.pos, &|d| {
                    if want_dock {
                        d.is_dock
                    } else {
                        d.is_market
                    }
                });
                if let Some(home) = home {
                    if home != *target {
                        set_task(
                            world,
                            id,
                            Task::Trade {
                                home,
                                far: *target,
                                outbound: true,
                            },
                            false,
                        );
                    }
                }
            }
        }
        Command::MarketBuy { resource } => {
            if *resource == Resource::Gold || !world.has_building(data, player, "market") {
                return;
            }
            let fee = stats::market_fee(world, data, player);
            let p = world.player_mut(player);
            let price = match resource {
                Resource::Food => &mut p.market_food,
                Resource::Wood => &mut p.market_wood,
                _ => &mut p.market_stone,
            };
            let cost = *price * (100 + fee) / 100;
            if p.resources.gold < cost {
                world.push_event(Event::NotEnough {
                    player,
                    resource: Resource::Gold,
                });
                return;
            }
            p.resources.gold -= cost;
            *p.resources.get_mut(*resource) += 100;
            let price = match resource {
                Resource::Food => &mut p.market_food,
                Resource::Wood => &mut p.market_wood,
                _ => &mut p.market_stone,
            };
            *price = (*price + 3).min(9999);
        }
        Command::MarketSell { resource } => {
            if *resource == Resource::Gold || !world.has_building(data, player, "market") {
                return;
            }
            let fee = stats::market_fee(world, data, player);
            let p = world.player_mut(player);
            if p.resources.get(*resource) < 100 {
                world.push_event(Event::NotEnough {
                    player,
                    resource: *resource,
                });
                return;
            }
            let price = match resource {
                Resource::Food => &mut p.market_food,
                Resource::Wood => &mut p.market_wood,
                _ => &mut p.market_stone,
            };
            let gain = *price * (100 - fee) / 100;
            *price = (*price - 3).max(20);
            *p.resources.get_mut(*resource) -= 100;
            p.resources.gold += gain;
        }
        Command::Tribute { to, amount } => {
            if to.idx() >= world.players.len() || to.is_gaia() || *to == player {
                return;
            }
            let amount = Stockpile::new(
                amount.food.max(0),
                amount.wood.max(0),
                amount.gold.max(0),
                amount.stone.max(0),
            );
            if !world.player(player).resources.covers(&amount) {
                return;
            }
            world.player_mut(player).resources.sub(&amount);
            world.player_mut(player).tribute_sent += amount.total();
            world.player_mut(*to).resources.add(&amount);
            world.player_mut(*to).tribute_received += amount.total();
            world.push_event(Event::Tribute {
                from: player,
                to: *to,
                amount,
            });
        }
        Command::SetDiplomacy { with, stance } => {
            if world.settings.lock_teams
                || with.idx() >= world.players.len()
                || with.is_gaia()
                || *with == player
            {
                return;
            }
            world.player_mut(player).diplomacy[with.idx()] = *stance;
        }
        Command::TogglePack { units } => {
            for id in own_units(world, player, units) {
                let e = world.get(id).unwrap();
                if data.object(e.ty).key == "trebuchet" && !matches!(e.task(), Task::Packing { .. })
                {
                    let unpack = e.packed;
                    set_task(
                        world,
                        id,
                        Task::Packing {
                            ticks: 5 * TICKS_PER_SECOND,
                            unpack,
                        },
                        false,
                    );
                }
            }
        }
        Command::Resign => {
            resign(world, data, player);
        }
        Command::Cheat { code } => {
            if !world.settings.allow_cheats {
                return;
            }
            apply_cheat(world, data, player, code);
        }
        Command::Spawn {
            owner,
            object,
            x,
            y,
        } => {
            // Only the editor/scenario path uses this: the server never relays it from clients.
            if owner.idx() < world.players.len() {
                let def = data.object(*object);
                let pos = if def.class == ObjectClass::Unit || def.class == ObjectClass::Animal {
                    Vec2::tile_center(*x, *y)
                } else {
                    Vec2::from_tile(*x, *y)
                };
                world.spawn(data, *owner, *object, pos, false);
            }
        }
        Command::Chat { text, to_allies } => {
            world.push_event(Event::Chat {
                player,
                text: text.clone(),
                to_allies: *to_allies,
            });
        }
    }
}

pub fn resign(world: &mut World, data: &GameData, player: PlayerId) {
    let tick = world.tick;
    let p = world.player_mut(player);
    if !p.alive {
        return;
    }
    p.alive = false;
    p.resigned = true;
    p.defeated_tick = Some(tick);
    world.push_event(Event::PlayerResigned { player });
    let ids: Vec<EntityId> = world.entities_of(player).map(|e| e.id).collect();
    for id in ids {
        crate::combat::kill(world, data, id, None);
    }
}

fn apply_cheat(world: &mut World, data: &GameData, player: PlayerId, code: &str) {
    let code = code.trim().to_lowercase();
    let mut applied = true;
    match code.as_str() {
        "cheese steak jimmy's" => world.player_mut(player).resources.food += 1000,
        "lumberjack" => world.player_mut(player).resources.wood += 1000,
        "robin hood" => world.player_mut(player).resources.gold += 1000,
        "rock on" => world.player_mut(player).resources.stone += 1000,
        "marco" | "polo" => {
            let p = world.player_mut(player);
            for t in p.explored.iter_mut() {
                *t = true;
            }
        }
        "aegis" => {
            let ids: Vec<EntityId> = world
                .entities_of(player)
                .filter(|e| e.build_progress < 1000)
                .map(|e| e.id)
                .collect();
            for id in ids {
                crate::unit::complete_building(world, data, id);
            }
        }
        "i r winner" => {
            let team = world.player(player).team;
            world.winner_team = Some(team);
            world.game_over = true;
            world.push_event(Event::Victory { team });
        }
        "how do you turn this on" => {
            if let Some(ty) = data.object_id("cobra_car") {
                let tc = world
                    .entities_of(player)
                    .find(|e| data.object(e.ty).is_town_center)
                    .map(|e| e.pos);
                if let Some(pos) = tc {
                    let (x, y) = pos.tile();
                    if let Some((sx, sy)) = world.map.nearest_passable(x, y + 3, false, 6) {
                        world.spawn(data, player, ty, Vec2::tile_center(sx, sy), false);
                    }
                }
            }
        }
        "torpedo" => {
            // Destroys all enemy units in sight. Kept gentle: kills the nearest enemy army of 10.
            let enemies: Vec<EntityId> = world
                .iter()
                .filter(|e| {
                    world.enemy(player, e.owner) && data.object(e.ty).class == ObjectClass::Unit
                })
                .map(|e| e.id)
                .take(10)
                .collect();
            for id in enemies {
                crate::combat::kill(world, data, id, Some(player));
            }
        }
        _ => applied = false,
    }
    if applied {
        world.player_mut(player).cheats_used = true;
        world.push_event(Event::Cheat { player, code });
    }
}

fn tick_relics_and_farms(world: &mut World, _data: &GameData) {
    if world.tick % TICKS_PER_SECOND != 0 {
        return;
    }
    // Relic gold: data.relic_gold_per_minute per relic, in hundredths per second.
    let per_second_hundredths = _data.relic_gold_per_minute * 100 / 60;
    for pid in 0..world.players.len() {
        let p = &world.players[pid];
        if p.id.is_gaia() || p.relics_held <= 0 {
            continue;
        }
        let pct = stats::relic_gold_pct(world, _data, p.id);
        let p = &mut world.players[pid];
        p.gold_frac += p.relics_held * per_second_hundredths * pct / 100;
        if p.gold_frac >= 100 {
            let whole = p.gold_frac / 100;
            p.gold_frac -= whole * 100;
            p.resources.gold += whole;
        }
    }
}

fn update_scores(world: &mut World, data: &GameData) {
    for pid in 1..world.players.len() {
        let id = PlayerId(pid as u8);
        let mut military = 0;
        let mut economy = 0;
        let mut society = 0;
        for e in world.entities_of(id) {
            let def = data.object(e.ty);
            let cost = def.cost.total();
            match def.class {
                ObjectClass::Unit => {
                    if def.is_villager || def.role == Role::Trade || def.role == Role::Fishing {
                        economy += cost / 5;
                    } else {
                        military += cost / 5;
                    }
                }
                ObjectClass::Building => {
                    society += cost / 5;
                    if def.is_wonder {
                        society += 1000;
                    }
                }
                _ => {}
            }
        }
        let p = &world.players[pid];
        let tech = p.techs.len() as i32 * 10 + p.age.idx() as i32 * 100;
        let kills = p.kills * 20 + p.razings * 50;
        let gathered = p.gathered.total() / 10;
        world.players[pid].score =
            military + economy + society + tech + kills + gathered + p.relics_held * 100;
    }
}

/// Is the player still "in the game"? No villagers, no military and no production buildings = out.
fn player_has_presence(world: &World, data: &GameData, p: PlayerId) -> bool {
    for e in world.entities_of(p) {
        let def = data.object(e.ty);
        match def.class {
            ObjectClass::Unit => {
                if def.is_villager
                    || (!def.attacks.is_empty() && def.role != Role::Fishing)
                    || def.is_king
                {
                    return true;
                }
            }
            ObjectClass::Building => {
                if e.build_progress >= 1000
                    && (data.trains_at.contains_key(&e.ty) || def.is_town_center || def.is_wonder)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

pub fn check_victory(world: &mut World, data: &GameData) {
    if world.game_over {
        return;
    }
    if matches!(world.settings.victory, VictoryMode::Scenario) {
        return;
    }
    // Defeat checks.
    let ids: Vec<PlayerId> = world.alive_players();
    for p in ids {
        let regicide_dead = world.settings.mode == GameMode::Regicide
            && !world.entities_of(p).any(|e| data.object(e.ty).is_king);
        if regicide_dead || !player_has_presence(world, data, p) {
            let tick = world.tick;
            let pl = world.player_mut(p);
            pl.alive = false;
            pl.defeated_tick = Some(tick);
            world.push_event(Event::PlayerDefeated { player: p });
        }
    }
    let alive = world.alive_players();
    if alive.is_empty() {
        world.game_over = true;
        return;
    }
    // A lone player (sandbox / editor test) never wins by elimination.
    if world.players.iter().filter(|p| !p.id.is_gaia()).count() < 2 {
        return;
    }
    let first_team = world.player(alive[0]).team;
    let all_same_team =
        first_team != 0 && alive.iter().all(|p| world.player(*p).team == first_team);
    if alive.len() == 1 || all_same_team {
        let team = if first_team == 0 {
            100 + alive[0].0
        } else {
            first_team
        };
        world.winner_team = Some(team);
        world.game_over = true;
        world.push_event(Event::Victory { team });
        return;
    }
    if matches!(world.settings.victory, VictoryMode::Conquest) {
        return;
    }
    if let VictoryMode::TimeLimit(mins) = world.settings.victory {
        if world.seconds() >= mins * 60 {
            let best = alive
                .iter()
                .max_by_key(|p| (world.player(**p).score, std::cmp::Reverse(p.0)))
                .copied()
                .unwrap();
            let t = world.player(best).team;
            let team = if t == 0 { 100 + best.0 } else { t };
            world.winner_team = Some(team);
            world.game_over = true;
            world.push_event(Event::Victory { team });
            return;
        }
    }
    // Wonder and relic countdowns.
    let countdown = data.wonder_countdown_secs as u32 * TICKS_PER_SECOND;
    let relic_countdown = data.relic_countdown_secs as u32 * TICKS_PER_SECOND;
    for p in alive.clone() {
        let has_wonder = world
            .entities_of(p)
            .any(|e| data.object(e.ty).is_wonder && e.build_progress >= 1000);
        let pl = world.player(p);
        match (has_wonder, pl.wonder_tick) {
            (true, None) => {
                world.player_mut(p).wonder_tick = Some(world.tick);
                world.push_event(Event::WonderStarted { player: p });
            }
            (false, Some(_)) => world.player_mut(p).wonder_tick = None,
            (true, Some(t)) if world.tick - t >= countdown => {
                let team = if pl.team == 0 { 100 + p.0 } else { pl.team };
                world.winner_team = Some(team);
                world.game_over = true;
                world.push_event(Event::Victory { team });
                return;
            }
            _ => {}
        }
    }
    if world.total_relics > 0 {
        // Relic countdown: one team holds every relic.
        let mut team_relics: std::collections::BTreeMap<u8, i32> =
            std::collections::BTreeMap::new();
        for p in &alive {
            let pl = world.player(*p);
            let t = if pl.team == 0 { 100 + p.0 } else { pl.team };
            *team_relics.entry(t).or_insert(0) += pl.relics_held;
        }
        for p in alive {
            let pl = world.player(p);
            let t = if pl.team == 0 { 100 + p.0 } else { pl.team };
            let holds_all = team_relics.get(&t).copied().unwrap_or(0) >= world.total_relics;
            match (holds_all, pl.relic_tick) {
                (true, None) => {
                    world.player_mut(p).relic_tick = Some(world.tick);
                    world.push_event(Event::RelicsStarted { player: p });
                }
                (false, Some(_)) => world.player_mut(p).relic_tick = None,
                (true, Some(start)) if world.tick - start >= relic_countdown => {
                    world.winner_team = Some(t);
                    world.game_over = true;
                    world.push_event(Event::Victory { team: t });
                    return;
                }
                _ => {}
            }
        }
    }
}
