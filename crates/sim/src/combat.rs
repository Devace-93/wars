//! Damage, death and projectiles.

use crate::fixed::{Fixed, Vec2};
use crate::ids::*;
use crate::stats;
use crate::types::*;
use crate::world::*;

/// Apply `attacks` from `attacker` (may be dead already) to `target`.
pub fn deal_damage(
    world: &mut World,
    data: &GameData,
    attacker: Option<EntityId>,
    attacker_owner: PlayerId,
    target: EntityId,
    attacks: &[Attack],
) {
    let Some(t) = world.get(target) else { return };
    let tdef = data.object(t.ty);
    // Carcasses are food, not targets.
    if tdef.class == ObjectClass::Animal && t.decay.is_some() {
        return;
    }
    let tstats = stats::stats_for(world, data, t.owner, t.ty);
    let mut armours = tstats.armours.clone();
    // Foundations and unfinished buildings have no armour.
    if t.build_progress < 1000 {
        armours.iter_mut().for_each(|a| a.value = 0);
    }
    let dmg = stats::damage(attacks, &armours, 100);
    let owner = t.owner;
    let pos = t.pos;
    let is_building = tdef.class == ObjectClass::Building;
    let is_animal = tdef.class == ObjectClass::Animal;
    let t = world.get_mut(target).unwrap();
    t.hp -= dmg;
    if is_animal {
        t.alerted = true;
    }
    let dead = t.hp <= 0;
    // Alert the owner (throttled).
    if !owner.is_gaia() && !is_animal {
        let tick = world.tick;
        let p = world.player_mut(owner);
        if tick.saturating_sub(p.last_alert_tick) >= 5 * TICKS_PER_SECOND || p.last_alert_tick == 0
        {
            p.last_alert_tick = tick;
            world.push_event(Event::UnderAttack {
                player: owner,
                at: pos,
                building: is_building,
            });
        }
    }
    // Idle defenders fight back when hit.
    if !dead {
        if let Some(a) = attacker {
            retaliate(world, data, target, a);
        }
    }
    if dead {
        kill(world, data, target, Some(attacker_owner));
    }
}

fn retaliate(world: &mut World, data: &GameData, victim: EntityId, attacker: EntityId) {
    let Some(v) = world.get(victim).cloned() else {
        return;
    };
    let def = data.object(v.ty);
    if def.class != ObjectClass::Unit || def.attacks.is_empty() || v.stance == Stance::NoAttack {
        return;
    }
    if v.is_idle()
        || matches!(
            v.task(),
            Task::Move {
                attack_move: true,
                ..
            }
        )
    {
        if world.get(attacker).is_some() {
            let v = world.get_mut(victim).unwrap();
            v.tasks.push_front(Task::Attack {
                target: attacker,
                chasing: v.stance != Stance::StandGround,
            });
        }
    }
}

/// Remove an entity from play. Animals become carcasses; everything else is despawned.
pub fn kill(world: &mut World, data: &GameData, id: EntityId, killer: Option<PlayerId>) {
    let Some(e) = world.get_any(id).cloned() else {
        return;
    };
    let def = data.object(e.ty);
    if def.class == ObjectClass::Animal && def.yields.is_some() && e.hp <= 0 && e.decay.is_none() {
        // Dead animal: becomes a food carcass for a while.
        let e = world.get_mut(id).unwrap();
        e.hp = 1;
        e.alerted = true;
        e.tasks.clear();
        e.decay = Some(150 * TICKS_PER_SECOND);
        return;
    }
    world.push_event(Event::Died {
        entity: id,
        at: e.pos,
        ty: e.ty,
        owner: e.owner,
    });
    if let Some(k) = killer {
        if !k.is_gaia() && k != e.owner && !e.owner.is_gaia() {
            if def.class == ObjectClass::Building {
                world.player_mut(k).razings += 1;
            } else if def.class == ObjectClass::Unit {
                world.player_mut(k).kills += 1;
            }
        }
    }
    if !e.owner.is_gaia() && def.class == ObjectClass::Unit {
        world.player_mut(e.owner).losses += 1;
    }
    // Eject or drown garrisoned units.
    let inside: Vec<EntityId> = e.garrisoned.clone();
    if !inside.is_empty() {
        if def.is_naval {
            for u in inside {
                if let Some(x) = world.get_mut(u) {
                    x.inside = None;
                }
                kill(world, data, u, killer);
            }
        } else {
            crate::unit::ungarrison_all(world, data, id);
        }
    }
    // Monks drop relics.
    if e.relics > 0 && def.class == ObjectClass::Unit {
        crate::unit::drop_relic(world, data, id);
    }
    if e.relics > 0 && def.is_monastery {
        let n = e.relics as i32;
        if !e.owner.is_gaia() {
            world.player_mut(e.owner).relics_held -= n;
        }
        if let Some(rty) = data.object_id("relic") {
            let (x, y) = e.pos.tile();
            for i in 0..n {
                if let Some((sx, sy)) = world.map.nearest_passable(x + i, y + 2, false, 6) {
                    world.spawn(data, PlayerId::GAIA, rty, Vec2::from_tile(sx, sy), false);
                }
            }
        }
    }
    // Units inside something: remove from the container.
    if let Some(container) = e.inside {
        if let Some(c) = world.get_mut(container) {
            c.garrisoned.retain(|x| *x != id);
        }
    }
    world.despawn(data, id);
}

/// Called by a unit or building whose reload is 0 and whose target is in range.
pub fn attack(world: &mut World, data: &GameData, attacker: EntityId, target: EntityId) {
    let Some(a) = world.get(attacker).cloned() else {
        return;
    };
    let Some(t) = world.get(target).cloned() else {
        return;
    };
    let st = stats::stats_for(world, data, a.owner, a.ty);
    let owner = a.owner;
    let apos = a.pos;
    let tpos = t.pos;
    let def = data.object(a.ty);
    let arrows = if def.garrison_arrows > 0 {
        1 + (a.garrisoned.len() as i32).min(def.garrison_capacity) * def.garrison_arrows
    } else {
        1
    };
    let blast = if def.key.contains("mangonel")
        || def.key.contains("onager")
        || def.key == "trebuchet"
        || def.key == "bombard_cannon"
        || def.key == "cannon_galleon"
    {
        Fixed::from_decimal("1.0")
    } else {
        Fixed::ZERO
    };
    {
        let tick = world.tick;
        let a = world.get_mut(attacker).unwrap();
        a.reload = st.reload_ticks;
        a.facing = apos.facing_toward(tpos);
        a.anim = 2;
        a.anim_tick = tick;
    }
    if st.projectile_speed == Fixed::ZERO {
        deal_damage(world, data, Some(attacker), owner, target, &st.attacks);
        return;
    }
    let dist = apos.dist(tpos);
    let flight = ((dist / st.projectile_speed).mul_int(TICKS_PER_SECOND as i32))
        .round()
        .max(1) as u32;
    world.push_event(Event::Shot {
        from: apos,
        to: tpos,
        ticks: flight,
    });
    for i in 0..arrows {
        let mut attacks = st.attacks.clone();
        if i > 0 {
            for at in attacks.iter_mut() {
                at.value = (at.value / 2).max(1);
            }
        }
        world.shots.push(Shot {
            arrive: world.tick + flight,
            attacker,
            owner,
            target,
            target_pos: tpos,
            attacks,
            ballistics: st.ballistics,
            blast,
        });
    }
}

pub fn resolve_shots(world: &mut World, data: &GameData) {
    let now = world.tick;
    let (due, later): (Vec<Shot>, Vec<Shot>) = std::mem::take(&mut world.shots)
        .into_iter()
        .partition(|s| s.arrive <= now);
    world.shots = later;
    for s in due {
        if s.blast > Fixed::ZERO {
            // Area damage around where the target was (or is, with ballistics).
            let center = match world.get(s.target) {
                Some(t) if s.ballistics => t.pos,
                _ => s.target_pos,
            };
            let victims: Vec<EntityId> = world
                .nearby(center, s.blast)
                .into_iter()
                .map(|(_, id)| id)
                .filter(|id| {
                    world
                        .get(*id)
                        .map(|e| !world.ally(s.owner, e.owner) || e.owner.is_gaia())
                        .unwrap_or(false)
                })
                .collect();
            for v in victims {
                deal_damage(world, data, Some(s.attacker), s.owner, v, &s.attacks);
            }
            continue;
        }
        let Some(t) = world.get(s.target) else {
            continue;
        };
        // Moving targets dodge unless the shooter has ballistics.
        let moved = t.pos.dist(s.target_pos) > Fixed::from_decimal("0.4");
        let def = data.object(t.ty);
        if moved && !s.ballistics && def.class == ObjectClass::Unit {
            let acc = world
                .get(s.attacker)
                .map(|a| stats::stats_for(world, data, a.owner, a.ty).accuracy)
                .unwrap_or(80);
            if !world.rng.chance(acc / 2) {
                continue;
            }
        }
        deal_damage(world, data, Some(s.attacker), s.owner, s.target, &s.attacks);
    }
}
