//! Evaluate scenario triggers once per tick.

use crate::command::Command;
use crate::fixed::Vec2;
use crate::ids::*;
use crate::trigger::*;
use crate::types::*;
use crate::world::*;

fn condition_true(world: &World, data: &GameData, c: &Condition) -> bool {
    match c {
        Condition::Timer { seconds } => world.seconds() >= *seconds,
        Condition::OwnObjects {
            player,
            object,
            count,
        } => {
            let n = world
                .entities_of(*player)
                .filter(|e| object.map(|o| e.ty == o).unwrap_or(true))
                .count() as i32;
            n >= *count
        }
        Condition::OwnFewerObjects {
            player,
            object,
            count,
        } => {
            let n = world
                .entities_of(*player)
                .filter(|e| object.map(|o| e.ty == o).unwrap_or(true))
                .count() as i32;
            n < *count
        }
        Condition::ObjectsInArea {
            player,
            area,
            count,
        } => {
            let n = world
                .entities_of(*player)
                .filter(|e| area.contains(e.pos))
                .count() as i32;
            n >= *count
        }
        Condition::DestroyObject { object } => match world.object_refs.get(*object as usize) {
            Some(Some(id)) => world.get(*id).is_none(),
            _ => true,
        },
        Condition::AccumulateAttribute {
            player,
            resource,
            amount,
        } => world.player(*player).resources.get(*resource) >= *amount,
        Condition::ResearchedTech { player, tech } => world.player(*player).has_tech(*tech),
        Condition::PlayerDefeated { player } => !world.player(*player).alive,
        Condition::TriggerFired { trigger } => world
            .trigger_state
            .get(*trigger as usize)
            .map(|s| s.fired > 0)
            .unwrap_or(false),
        Condition::Chance { pct } => {
            // Deterministic pseudo-chance from the tick so the RNG stream is untouched.
            ((world.tick.wrapping_mul(2654435761) >> 8) % 100) < *pct as u32
        }
        Condition::BringObjectToArea { object, area } => {
            match world.object_refs.get(*object as usize) {
                Some(Some(id)) => world
                    .get(*id)
                    .map(|e| area.contains(e.pos))
                    .unwrap_or(false),
                _ => false,
            }
        }
        Condition::InAge { player, age } => world.player(*player).age >= *age,
    }
    .then_some(())
    .is_some()
        && {
            let _ = data;
            true
        }
}

pub fn run_triggers(world: &mut World, data: &GameData) {
    if world.triggers.is_empty() {
        return;
    }
    if world.trigger_state.len() != world.triggers.len() {
        world.trigger_state = world
            .triggers
            .iter()
            .map(|t| TriggerState {
                active: t.enabled,
                fired: 0,
                done: false,
            })
            .collect();
    }
    for i in 0..world.triggers.len() {
        let st = world.trigger_state[i].clone();
        if !st.active || st.done {
            continue;
        }
        let trig = world.triggers[i].clone();
        if !trig
            .conditions
            .iter()
            .all(|c| condition_true(world, data, c))
        {
            continue;
        }
        for eff in &trig.effects {
            run_effect(world, data, eff);
        }
        let st = &mut world.trigger_state[i];
        st.fired += 1;
        if !trig.looping {
            st.done = true;
        }
    }
}

fn run_effect(world: &mut World, data: &GameData, eff: &TriggerEffect) {
    match eff {
        TriggerEffect::SendChat { player, text }
        | TriggerEffect::DisplayInstructions { player, text } => {
            world.push_event(Event::Chat {
                player: player.unwrap_or(PlayerId::GAIA),
                text: text.clone(),
                to_allies: false,
            });
        }
        TriggerEffect::ChangeDiplomacy {
            source,
            target,
            stance,
        } => {
            if source.idx() < world.players.len() && target.idx() < world.players.len() {
                world.player_mut(*source).diplomacy[target.idx()] = *stance;
            }
        }
        TriggerEffect::ResearchTech { player, tech } => {
            crate::unit::grant_tech(world, data, *player, *tech);
        }
        TriggerEffect::Tribute { from, to, amount } => {
            crate::step::apply_command(
                world,
                data,
                *from,
                &Command::Tribute {
                    to: *to,
                    amount: *amount,
                },
            );
        }
        TriggerEffect::ChangeResources { player, amount } => {
            if player.idx() < world.players.len() {
                let p = world.player_mut(*player);
                p.resources.add(amount);
                p.resources.food = p.resources.food.max(0);
                p.resources.wood = p.resources.wood.max(0);
                p.resources.gold = p.resources.gold.max(0);
                p.resources.stone = p.resources.stone.max(0);
            }
        }
        TriggerEffect::CreateObject {
            player,
            object,
            x,
            y,
        } => {
            let def = data.object(*object);
            let pos = if matches!(def.class, ObjectClass::Unit | ObjectClass::Animal) {
                Vec2::tile_center(*x, *y)
            } else {
                Vec2::from_tile(*x, *y)
            };
            world.spawn(data, *player, *object, pos, false);
        }
        TriggerEffect::KillObjects {
            player,
            object,
            area,
        }
        | TriggerEffect::RemoveObjects {
            player,
            object,
            area,
        } => {
            let ids: Vec<EntityId> = world
                .entities_of(*player)
                .filter(|e| {
                    object.map(|o| e.ty == o).unwrap_or(true)
                        && area.as_ref().map(|a| a.contains(e.pos)).unwrap_or(true)
                })
                .map(|e| e.id)
                .collect();
            for id in ids {
                if matches!(eff, TriggerEffect::KillObjects { .. }) {
                    crate::combat::kill(world, data, id, None);
                } else {
                    world.despawn(data, id);
                }
            }
        }
        TriggerEffect::ChangeOwnership {
            from,
            to,
            object,
            area,
        } => {
            let ids: Vec<EntityId> = world
                .entities_of(*from)
                .filter(|e| {
                    object.map(|o| e.ty == o).unwrap_or(true)
                        && area.as_ref().map(|a| a.contains(e.pos)).unwrap_or(true)
                })
                .map(|e| e.id)
                .collect();
            for id in ids {
                crate::unit::change_owner(world, data, id, *to);
            }
        }
        TriggerEffect::DeclareVictory { player } => {
            let t = world.player(*player).team;
            let team = if t == 0 { 100 + player.0 } else { t };
            world.winner_team = Some(team);
            world.game_over = true;
            world.push_event(Event::Victory { team });
        }
        TriggerEffect::DeclareDefeat { player } => {
            crate::step::resign(world, data, *player);
        }
        TriggerEffect::ActivateTrigger { trigger } => {
            if let Some(s) = world.trigger_state.get_mut(*trigger as usize) {
                s.active = true;
                s.done = false;
            }
        }
        TriggerEffect::DeactivateTrigger { trigger } => {
            if let Some(s) = world.trigger_state.get_mut(*trigger as usize) {
                s.active = false;
            }
        }
        TriggerEffect::ChangeView { .. } | TriggerEffect::PlaySound { .. } => {
            // Client-side only; clients read the trigger log through events in a later version.
        }
        TriggerEffect::TaskObjects {
            player,
            area,
            to_x,
            to_y,
        } => {
            let units: Vec<EntityId> = world
                .entities_of(*player)
                .filter(|e| area.contains(e.pos))
                .map(|e| e.id)
                .collect();
            crate::step::apply_command(
                world,
                data,
                *player,
                &Command::Move {
                    units,
                    to: Vec2::tile_center(*to_x, *to_y),
                    queue: false,
                },
            );
        }
        TriggerEffect::DamageObjects {
            player,
            area,
            amount,
        } => {
            let ids: Vec<EntityId> = world
                .entities_of(*player)
                .filter(|e| area.contains(e.pos))
                .map(|e| e.id)
                .collect();
            for id in ids {
                if let Some(e) = world.get_mut(id) {
                    e.hp -= *amount;
                }
                if world.get(id).map(|e| e.hp <= 0).unwrap_or(false) {
                    crate::combat::kill(world, data, id, None);
                }
            }
        }
        TriggerEffect::ChangeAge { player, age } => {
            if player.idx() < world.players.len() {
                world.player_mut(*player).age = *age;
                world.push_event(Event::AgeAdvanced {
                    player: *player,
                    age: *age,
                });
            }
        }
        TriggerEffect::Unlock { .. } => {}
    }
}
