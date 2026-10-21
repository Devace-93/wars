//! Effective stats of an object type for a given player: base values from the data plus
//! every technology, civ bonus and age effect the player has.

use crate::fixed::Fixed;
use crate::ids::*;
use crate::types::*;
use crate::world::World;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stats {
    pub hp: i32,
    pub attacks: Vec<Attack>,
    pub armours: Vec<Armour>,
    pub range: Fixed,
    pub min_range: Fixed,
    pub los: Fixed,
    pub speed: Fixed,
    pub reload_ticks: u32,
    pub accuracy: i32,
    pub ballistics: bool,
    pub cost: Stockpile,
    pub train_ticks: u32,
    pub population: i32,
    pub pop_support: i32,
    pub garrison_capacity: i32,
    pub garrison_arrows: i32,
    pub heal_rate: i32,
    pub conversion_resist: i32,
    pub projectile_speed: Fixed,
    pub work_rate_pct: i32,
}

fn matches(def: &ObjectType, key: &str, target: &Target) -> bool {
    match target {
        Target::Object(k) => k == key,
        Target::Role(r) => def.role == *r,
        Target::Class(c) => def.class == *c,
        Target::ArmourClass(ac) => def.armours.iter().any(|a| a.class == *ac),
        Target::All => true,
    }
}

/// All effects currently active for a player, in a stable order: civ bonuses, team bonuses
/// of allies, then researched techs in id order.
pub fn active_effects<'a>(
    world: &'a World,
    data: &'a GameData,
    player: PlayerId,
) -> Vec<&'a Effect> {
    let mut v = Vec::new();
    if player.is_gaia() {
        return v;
    }
    let p = world.player(player);
    let civ = data.civ(p.civ);
    v.extend(civ.bonuses.iter());
    for other in &world.players {
        if other.id.is_gaia() || !world.ally(player, other.id) {
            continue;
        }
        v.extend(data.civ(other.civ).team_bonus.iter());
    }
    for t in &p.techs {
        v.extend(data.tech(*t).effects.iter());
    }
    v
}

pub fn stats_for(world: &World, data: &GameData, player: PlayerId, ty: TypeId) -> Stats {
    let def = data.object(ty);
    let mut s = Stats {
        hp: def.hp,
        attacks: def.attacks.clone(),
        armours: def.armours.clone(),
        range: def.range,
        min_range: def.min_range,
        los: def.los,
        speed: def.speed,
        reload_ticks: (def.reload.mul_int(crate::world::TICKS_PER_SECOND as i32))
            .round()
            .max(1) as u32,
        accuracy: def.accuracy,
        ballistics: false,
        cost: def.cost,
        train_ticks: (def.train_time as u32) * crate::world::TICKS_PER_SECOND,
        population: def.population,
        pop_support: def.pop_support,
        garrison_capacity: def.garrison_capacity,
        garrison_arrows: def.garrison_arrows,
        heal_rate: def.heal_rate,
        conversion_resist: def.conversion_resist,
        projectile_speed: def.projectile_speed,
        work_rate_pct: 100,
    };
    if player.is_gaia() {
        return s;
    }
    let mut hp_pct = 100;
    let mut speed_pct = 100;
    let mut reload_pct = 100;
    let mut train_pct = 100;
    let mut cost_pct = [100i32; 4];
    for e in active_effects(world, data, player) {
        match e {
            Effect::Add {
                target,
                attr,
                value,
            } if matches(def, &def.key, target) => match attr {
                Attr::Hp => s.hp += value,
                Attr::MeleeAttack => add_attack(&mut s.attacks, ArmourClass::Melee, *value),
                Attr::PierceAttack => add_attack(&mut s.attacks, ArmourClass::Pierce, *value),
                Attr::MeleeArmour => add_armour(&mut s.armours, ArmourClass::Melee, *value),
                Attr::PierceArmour => add_armour(&mut s.armours, ArmourClass::Pierce, *value),
                Attr::BonusAttack(c) => add_attack(&mut s.attacks, *c, *value),
                Attr::BonusArmour(c) => add_armour(&mut s.armours, *c, *value),
                Attr::Range => {
                    if s.range > Fixed::ZERO || *value > 0 {
                        s.range += Fixed::from_int(*value);
                    }
                }
                Attr::MinRange => {
                    s.min_range = (s.min_range + Fixed::from_int(*value)).max(Fixed::ZERO)
                }
                Attr::Los => s.los += Fixed::from_int(*value),
                Attr::Speed => speed_pct += value,
                Attr::ReloadTime => reload_pct += value,
                Attr::Accuracy => s.accuracy = (s.accuracy + value).clamp(0, 100),
                Attr::TrainTime => train_pct += value,
                Attr::GarrisonCapacity => s.garrison_capacity += value,
                Attr::Population => s.population += value,
                Attr::PopSupport => s.pop_support += value,
                Attr::HealRate => s.heal_rate += value,
                Attr::ConversionResist => s.conversion_resist += value,
                Attr::ProjectileSpeed => s.projectile_speed += Fixed::from_int(*value),
                Attr::Ballistics => s.ballistics = true,
            },
            Effect::Mul { target, attr, pct } if matches(def, &def.key, target) => match attr {
                Attr::Hp => hp_pct = hp_pct * pct / 100,
                Attr::Speed => speed_pct = speed_pct * pct / 100,
                Attr::ReloadTime => reload_pct = reload_pct * pct / 100,
                Attr::TrainTime => train_pct = train_pct * pct / 100,
                Attr::MeleeAttack => {
                    for a in s.attacks.iter_mut() {
                        if a.class == ArmourClass::Melee {
                            a.value = a.value * pct / 100;
                        }
                    }
                }
                Attr::PierceAttack => {
                    for a in s.attacks.iter_mut() {
                        if a.class == ArmourClass::Pierce {
                            a.value = a.value * pct / 100;
                        }
                    }
                }
                Attr::Range => s.range = s.range.pct(*pct),
                Attr::Los => s.los = s.los.pct(*pct),
                _ => {}
            },
            Effect::CostDiscount {
                target,
                resource,
                pct,
            } if matches(def, &def.key, target) => match resource {
                Some(r) => cost_pct[r.idx()] = cost_pct[r.idx()] * (100 - pct) / 100,
                None => {
                    for c in cost_pct.iter_mut() {
                        *c = *c * (100 - pct) / 100;
                    }
                }
            },
            Effect::WorkRate { target, pct } if matches(def, &def.key, target) => {
                s.work_rate_pct = s.work_rate_pct * pct / 100;
            }
            Effect::ConversionResist { value } if def.class == ObjectClass::Unit => {
                s.conversion_resist += value
            }
            _ => {}
        }
    }
    s.hp = s.hp * hp_pct / 100;
    s.speed = s.speed.pct(speed_pct);
    s.reload_ticks = ((s.reload_ticks as i32 * reload_pct) / 100).max(1) as u32;
    s.train_ticks = ((s.train_ticks as i32 * train_pct) / 100).max(1) as u32;
    s.cost = Stockpile::new(
        s.cost.food * cost_pct[0] / 100,
        s.cost.wood * cost_pct[1] / 100,
        s.cost.gold * cost_pct[2] / 100,
        s.cost.stone * cost_pct[3] / 100,
    );
    s
}

fn add_attack(v: &mut Vec<Attack>, class: ArmourClass, value: i32) {
    if let Some(a) = v.iter_mut().find(|a| a.class == class) {
        a.value += value;
    } else {
        v.push(Attack { class, value });
    }
}
fn add_armour(v: &mut Vec<Armour>, class: ArmourClass, value: i32) {
    if let Some(a) = v.iter_mut().find(|a| a.class == class) {
        a.value += value;
    } else {
        v.push(Armour { class, value });
    }
}

/// Damage dealt by `attacks` to a target with `armours`: the classic formula, min 1 when any
/// class matches (otherwise 0 for things that are immune, which does not happen in practice).
pub fn damage(attacks: &[Attack], armours: &[Armour], elevation_pct: i32) -> i32 {
    let mut total = 0;
    for a in attacks {
        if let Some(arm) = armours.iter().find(|r| r.class == a.class) {
            total += (a.value - arm.value).max(0);
        }
    }
    let total = total * elevation_pct / 100;
    total.max(1)
}

/// Research cost and time for a tech after discounts.
pub fn tech_cost(
    world: &World,
    data: &GameData,
    player: PlayerId,
    tech: TechId,
) -> (Stockpile, u32) {
    let def = data.tech(tech);
    let mut cost = def.cost;
    let mut time_pct = 100;
    for e in active_effects(world, data, player) {
        match e {
            Effect::TechDiscount { pct, building } => {
                let applies = match building {
                    Some(b) => def.building.as_deref() == Some(b.as_str()),
                    None => true,
                };
                if applies {
                    cost = cost.scaled(100 - pct);
                }
            }
            Effect::FreeTech { tech: t } if *t == def.key => {
                cost = Stockpile::ZERO;
                time_pct = 1;
            }
            Effect::WorkRate {
                target: Target::Object(b),
                pct,
            } if def.building.as_deref() == Some(b.as_str()) => {
                time_pct = time_pct * 100 / (*pct).max(1);
            }
            _ => {}
        }
    }
    let ticks =
        ((def.time as u32 * crate::world::TICKS_PER_SECOND) as i32 * time_pct / 100).max(1) as u32;
    (cost, ticks)
}

/// Gather rate in resource units per tick (fixed) for a villager of `player`.
pub fn gather_rate(world: &World, data: &GameData, player: PlayerId, kind: GatherKind) -> Fixed {
    let base = data
        .gather_rates
        .get(&kind)
        .copied()
        .unwrap_or(Fixed::from_decimal("0.3"));
    let mut pct = 100;
    for e in active_effects(world, data, player) {
        if let Effect::GatherRate { kind: k, pct: p } = e {
            if k.is_none() || *k == Some(kind) {
                pct = pct * p / 100;
            }
        }
    }
    base.pct(pct).div_int(crate::world::TICKS_PER_SECOND as i32)
}

pub fn carry_capacity(world: &World, data: &GameData, player: PlayerId) -> i32 {
    let mut c = data.carry_capacity;
    for e in active_effects(world, data, player) {
        if let Effect::CarryCapacity { value } = e {
            c += value;
        }
    }
    c
}

pub fn pop_cap(world: &World, data: &GameData, player: PlayerId) -> i32 {
    let mut c = world.settings.pop_cap;
    for e in active_effects(world, data, player) {
        if let Effect::PopCap { value } = e {
            c += value;
        }
    }
    c
}

pub fn market_fee(world: &World, data: &GameData, player: PlayerId) -> i32 {
    let mut fee = data.market_fee_pct;
    for e in active_effects(world, data, player) {
        if let Effect::TradeFee { pct } = e {
            fee = *pct;
        }
    }
    fee
}

pub fn relic_gold_pct(world: &World, data: &GameData, player: PlayerId) -> i32 {
    let mut pct = 100;
    for e in active_effects(world, data, player) {
        if let Effect::RelicGold { pct: p } = e {
            pct = pct * p / 100;
        }
    }
    pct
}

/// Is this object type available to the player right now (age, civ tree, prerequisites)?
pub fn is_available(world: &World, data: &GameData, player: PlayerId, ty: TypeId) -> bool {
    let def = data.object(ty);
    let p = world.player(player);
    if def.age > p.age {
        return false;
    }
    let civ = data.civ(p.civ);
    if civ.disabled_objects.contains(&def.key) {
        return false;
    }
    // Unique units belong to one civ (plus a few shared ones flagged by role).
    if def
        .armours
        .iter()
        .any(|a| a.class == ArmourClass::UniqueUnit)
        && civ.unique_unit != def.key
        && civ.unique_unit_elite != def.key
    {
        return false;
    }
    for e in active_effects(world, data, player) {
        if let Effect::Disable { object } = e {
            if *object == def.key {
                return false;
            }
        }
    }
    if let Some(t) = &def.requires_tech {
        if !p.has_tech(data.tk(t)) {
            return false;
        }
    }
    for r in &def.requires {
        if !world.has_building(data, player, r) {
            return false;
        }
    }
    // Upgraded-away types are not trainable (e.g. militia after Man-at-Arms).
    if let Some((tech, _)) = &def.upgrades_to {
        if p.has_tech(data.tk(tech)) {
            return false;
        }
    }
    true
}

pub fn tech_available(world: &World, data: &GameData, player: PlayerId, tech: TechId) -> bool {
    let def = data.tech(tech);
    let p = world.player(player);
    if p.has_tech(tech) || p.researching.contains(&tech) {
        return false;
    }
    if def.age > p.age {
        return false;
    }
    let civ = data.civ(p.civ);
    if civ.disabled_techs.contains(&def.key) {
        return false;
    }
    if let Some(u) = &def.unique_to {
        if *u != civ.key {
            return false;
        }
    }
    for r in &def.requires {
        if let Some(t) = data.tech_id(r) {
            if !p.has_tech(t) {
                return false;
            }
        } else if !world.has_building(data, player, r) {
            return false;
        }
    }
    true
}

/// Resolve the current (upgraded) type for a base type the player wants to train.
pub fn resolve_upgrade(world: &World, data: &GameData, player: PlayerId, ty: TypeId) -> TypeId {
    let p = world.player(player);
    let mut cur = ty;
    for _ in 0..8 {
        let def = data.object(cur);
        match &def.upgrades_to {
            Some((tech, to)) if p.has_tech(data.tk(tech)) => cur = data.obj(to),
            _ => break,
        }
    }
    cur
}
