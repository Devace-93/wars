use wars_sim::ids::*;
use wars_sim::types::*;

fn tech(
    key: &str,
    name: &str,
    cost: (i32, i32, i32, i32),
    time: i32,
    age: Age,
    building: &str,
    desc: &str,
) -> TechDef {
    TechDef {
        key: key.into(),
        name: name.into(),
        cost: Stockpile::new(cost.0, cost.1, cost.2, cost.3),
        time,
        age,
        building: Some(building.into()),
        requires: vec![],
        effects: vec![],
        advances_to: None,
        description: desc.into(),
        unique_to: None,
    }
}

fn add(target: Target, attr: Attr, value: i32) -> Effect {
    Effect::Add {
        target,
        attr,
        value,
    }
}
fn mul(target: Target, attr: Attr, pct: i32) -> Effect {
    Effect::Mul { target, attr, pct }
}
fn obj(k: &str) -> Target {
    Target::Object(k.into())
}
fn role(r: Role) -> Target {
    Target::Role(r)
}
fn ac(c: ArmourClass) -> Target {
    Target::ArmourClass(c)
}
fn up(from: &str, to: &str) -> Effect {
    Effect::Upgrade {
        from: from.into(),
        to: to.into(),
    }
}

/// Unit line upgrade: the tech that turns every `from` into `to`.
fn line(
    key: &str,
    name: &str,
    cost: (i32, i32, i32, i32),
    time: i32,
    age: Age,
    building: &str,
    from: &str,
    requires: &[&str],
) -> TechDef {
    let mut t = tech(
        key,
        name,
        cost,
        time,
        age,
        building,
        &format!("Upgrade {from} to {name}."),
    );
    t.effects.push(up(from, key));
    t.requires = requires.iter().map(|s| s.to_string()).collect();
    t
}

pub fn techs() -> Vec<TechDef> {
    use ArmourClass as C;
    use Attr as A;
    let mut v = Vec::new();

    // ---------------- Ages ----------------
    let mut t = tech(
        "feudal_age",
        "Feudal Age",
        (500, 0, 0, 0),
        130,
        Age::Dark,
        "town_center",
        "Advance to the Feudal Age.",
    );
    t.advances_to = Some(Age::Feudal);
    t.effects.push(add(obj("scout_cavalry"), A::MeleeAttack, 2));
    t.effects.push(add(obj("scout_cavalry"), A::Los, 2));
    t.effects.push(add(obj("scout_cavalry"), A::Speed, 25));
    v.push(t);
    let mut t = tech(
        "castle_age",
        "Castle Age",
        (800, 0, 200, 0),
        160,
        Age::Feudal,
        "town_center",
        "Advance to the Castle Age.",
    );
    t.advances_to = Some(Age::Castle);
    t.requires = vec!["feudal_age".into()];
    v.push(t);
    let mut t = tech(
        "imperial_age",
        "Imperial Age",
        (1000, 0, 800, 0),
        190,
        Age::Castle,
        "town_center",
        "Advance to the Imperial Age.",
    );
    t.advances_to = Some(Age::Imperial);
    t.requires = vec!["castle_age".into()];
    v.push(t);

    // ---------------- Town centre ----------------
    let mut t = tech(
        "loom",
        "Loom",
        (0, 0, 50, 0),
        25,
        Age::Dark,
        "town_center",
        "Villagers +15 HP, +1/+1 armour.",
    );
    t.effects.push(add(obj("villager"), A::Hp, 15));
    t.effects.push(add(obj("villager"), A::MeleeArmour, 1));
    t.effects.push(add(obj("villager"), A::PierceArmour, 1));
    v.push(t);
    let mut t = tech(
        "wheelbarrow",
        "Wheelbarrow",
        (175, 50, 0, 0),
        75,
        Age::Feudal,
        "town_center",
        "Villagers +10% speed, +3 carry.",
    );
    t.effects.push(add(obj("villager"), A::Speed, 10));
    t.effects.push(Effect::CarryCapacity { value: 3 });
    v.push(t);
    let mut t = tech(
        "hand_cart",
        "Hand Cart",
        (300, 200, 0, 0),
        55,
        Age::Castle,
        "town_center",
        "Villagers +10% speed, +4 carry.",
    );
    t.requires = vec!["wheelbarrow".into()];
    t.effects.push(add(obj("villager"), A::Speed, 10));
    t.effects.push(Effect::CarryCapacity { value: 4 });
    v.push(t);
    let mut t = tech(
        "town_watch",
        "Town Watch",
        (75, 0, 0, 0),
        25,
        Age::Feudal,
        "town_center",
        "Buildings +4 line of sight.",
    );
    t.effects
        .push(add(Target::Class(ObjectClass::Building), A::Los, 4));
    v.push(t);
    let mut t = tech(
        "town_patrol",
        "Town Patrol",
        (300, 0, 100, 0),
        40,
        Age::Castle,
        "town_center",
        "Buildings +4 line of sight.",
    );
    t.requires = vec!["town_watch".into()];
    t.effects
        .push(add(Target::Class(ObjectClass::Building), A::Los, 4));
    v.push(t);

    // ---------------- Mill / lumber / mining ----------------
    let mut t = tech(
        "horse_collar",
        "Horse Collar",
        (75, 75, 0, 0),
        20,
        Age::Feudal,
        "mill",
        "Farms +75 food.",
    );
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Farm),
        pct: 110,
    });
    v.push(t);
    let mut t = tech(
        "heavy_plow",
        "Heavy Plow",
        (125, 125, 0, 0),
        40,
        Age::Castle,
        "mill",
        "Farms +125 food, farmers carry +1.",
    );
    t.requires = vec!["horse_collar".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Farm),
        pct: 110,
    });
    t.effects.push(Effect::CarryCapacity { value: 1 });
    v.push(t);
    let mut t = tech(
        "crop_rotation",
        "Crop Rotation",
        (250, 250, 0, 0),
        70,
        Age::Imperial,
        "mill",
        "Farms +175 food.",
    );
    t.requires = vec!["heavy_plow".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Farm),
        pct: 110,
    });
    v.push(t);
    let mut t = tech(
        "double_bit_axe",
        "Double-Bit Axe",
        (100, 50, 0, 0),
        25,
        Age::Feudal,
        "lumber_camp",
        "Lumberjacks +20%.",
    );
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Wood),
        pct: 120,
    });
    v.push(t);
    let mut t = tech(
        "bow_saw",
        "Bow Saw",
        (150, 100, 0, 0),
        50,
        Age::Castle,
        "lumber_camp",
        "Lumberjacks +20%.",
    );
    t.requires = vec!["double_bit_axe".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Wood),
        pct: 120,
    });
    v.push(t);
    let mut t = tech(
        "two_man_saw",
        "Two-Man Saw",
        (300, 200, 0, 0),
        100,
        Age::Imperial,
        "lumber_camp",
        "Lumberjacks +10%.",
    );
    t.requires = vec!["bow_saw".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Wood),
        pct: 110,
    });
    v.push(t);
    let mut t = tech(
        "gold_mining",
        "Gold Mining",
        (100, 75, 0, 0),
        30,
        Age::Feudal,
        "mining_camp",
        "Gold miners +15%.",
    );
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Gold),
        pct: 115,
    });
    v.push(t);
    let mut t = tech(
        "gold_shaft_mining",
        "Gold Shaft Mining",
        (200, 150, 0, 0),
        75,
        Age::Castle,
        "mining_camp",
        "Gold miners +15%.",
    );
    t.requires = vec!["gold_mining".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Gold),
        pct: 115,
    });
    v.push(t);
    let mut t = tech(
        "stone_mining",
        "Stone Mining",
        (100, 75, 0, 0),
        30,
        Age::Feudal,
        "mining_camp",
        "Stone miners +15%.",
    );
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Stone),
        pct: 115,
    });
    v.push(t);
    let mut t = tech(
        "stone_shaft_mining",
        "Stone Shaft Mining",
        (200, 150, 0, 0),
        75,
        Age::Castle,
        "mining_camp",
        "Stone miners +15%.",
    );
    t.requires = vec!["stone_mining".into()];
    t.effects.push(Effect::GatherRate {
        kind: Some(GatherKind::Stone),
        pct: 115,
    });
    v.push(t);

    // ---------------- Blacksmith ----------------
    let melee = |t: &mut TechDef, val: i32| {
        t.effects
            .push(add(role(Role::Infantry), A::MeleeAttack, val));
        t.effects
            .push(add(role(Role::Cavalry), A::MeleeAttack, val));
        t.effects.push(add(role(Role::Scout), A::MeleeAttack, val));
        t.effects.push(add(role(Role::King), A::MeleeAttack, 0));
    };
    let mut t = tech(
        "forging",
        "Forging",
        (150, 0, 0, 0),
        50,
        Age::Feudal,
        "blacksmith",
        "Infantry and cavalry +1 attack.",
    );
    melee(&mut t, 1);
    v.push(t);
    let mut t = tech(
        "iron_casting",
        "Iron Casting",
        (220, 0, 120, 0),
        75,
        Age::Castle,
        "blacksmith",
        "Infantry and cavalry +1 attack.",
    );
    t.requires = vec!["forging".into()];
    melee(&mut t, 1);
    v.push(t);
    let mut t = tech(
        "blast_furnace",
        "Blast Furnace",
        (275, 0, 225, 0),
        100,
        Age::Imperial,
        "blacksmith",
        "Infantry and cavalry +2 attack.",
    );
    t.requires = vec!["iron_casting".into()];
    melee(&mut t, 2);
    v.push(t);
    let inf_armour = |t: &mut TechDef, m: i32, p: i32| {
        t.effects.push(add(role(Role::Infantry), A::MeleeArmour, m));
        t.effects
            .push(add(role(Role::Infantry), A::PierceArmour, p));
    };
    let mut t = tech(
        "scale_mail_armor",
        "Scale Mail Armor",
        (100, 0, 0, 0),
        40,
        Age::Feudal,
        "blacksmith",
        "Infantry +1/+1 armour.",
    );
    inf_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "chain_mail_armor",
        "Chain Mail Armor",
        (200, 0, 100, 0),
        55,
        Age::Castle,
        "blacksmith",
        "Infantry +1/+1 armour.",
    );
    t.requires = vec!["scale_mail_armor".into()];
    inf_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "plate_mail_armor",
        "Plate Mail Armor",
        (300, 0, 150, 0),
        70,
        Age::Imperial,
        "blacksmith",
        "Infantry +1/+2 armour.",
    );
    t.requires = vec!["chain_mail_armor".into()];
    inf_armour(&mut t, 1, 2);
    v.push(t);
    let cav_armour = |t: &mut TechDef, m: i32, p: i32| {
        for r in [Role::Cavalry, Role::Scout, Role::CavalryArcher] {
            t.effects.push(add(role(r), A::MeleeArmour, m));
            t.effects.push(add(role(r), A::PierceArmour, p));
        }
    };
    let mut t = tech(
        "scale_barding_armor",
        "Scale Barding Armor",
        (150, 0, 0, 0),
        45,
        Age::Feudal,
        "blacksmith",
        "Cavalry +1/+1 armour.",
    );
    cav_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "chain_barding_armor",
        "Chain Barding Armor",
        (250, 0, 150, 0),
        60,
        Age::Castle,
        "blacksmith",
        "Cavalry +1/+1 armour.",
    );
    t.requires = vec!["scale_barding_armor".into()];
    cav_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "plate_barding_armor",
        "Plate Barding Armor",
        (350, 0, 200, 0),
        75,
        Age::Imperial,
        "blacksmith",
        "Cavalry +1/+2 armour.",
    );
    t.requires = vec!["chain_barding_armor".into()];
    cav_armour(&mut t, 1, 2);
    v.push(t);
    let arrows = |t: &mut TechDef| {
        for r in [Role::Archer, Role::CavalryArcher, Role::Defensive] {
            t.effects.push(add(role(r), A::PierceAttack, 1));
            t.effects.push(add(role(r), A::Range, 1));
        }
        t.effects.push(add(obj("town_center"), A::PierceAttack, 1));
        t.effects.push(add(obj("town_center"), A::Range, 1));
        t.effects.push(add(obj("castle"), A::PierceAttack, 1));
        t.effects.push(add(obj("castle"), A::Range, 1));
        t.effects.push(add(role(Role::Naval), A::PierceAttack, 1));
        t.effects.push(add(role(Role::Naval), A::Range, 1));
    };
    let mut t = tech(
        "fletching",
        "Fletching",
        (100, 0, 50, 0),
        30,
        Age::Feudal,
        "blacksmith",
        "Archers, towers and ships +1 attack, +1 range.",
    );
    arrows(&mut t);
    v.push(t);
    let mut t = tech(
        "bodkin_arrow",
        "Bodkin Arrow",
        (200, 0, 100, 0),
        35,
        Age::Castle,
        "blacksmith",
        "Archers, towers and ships +1 attack, +1 range.",
    );
    t.requires = vec!["fletching".into()];
    arrows(&mut t);
    v.push(t);
    let mut t = tech(
        "bracer",
        "Bracer",
        (300, 0, 200, 0),
        40,
        Age::Imperial,
        "blacksmith",
        "Archers, towers and ships +1 attack, +1 range.",
    );
    t.requires = vec!["bodkin_arrow".into()];
    arrows(&mut t);
    v.push(t);
    let arch_armour = |t: &mut TechDef, m: i32, p: i32| {
        for r in [Role::Archer, Role::CavalryArcher] {
            t.effects.push(add(role(r), A::MeleeArmour, m));
            t.effects.push(add(role(r), A::PierceArmour, p));
        }
    };
    let mut t = tech(
        "padded_archer_armor",
        "Padded Archer Armor",
        (100, 0, 0, 0),
        40,
        Age::Feudal,
        "blacksmith",
        "Archers +1/+1 armour.",
    );
    arch_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "leather_archer_armor",
        "Leather Archer Armor",
        (150, 0, 150, 0),
        55,
        Age::Castle,
        "blacksmith",
        "Archers +1/+1 armour.",
    );
    t.requires = vec!["padded_archer_armor".into()];
    arch_armour(&mut t, 1, 1);
    v.push(t);
    let mut t = tech(
        "ring_archer_armor",
        "Ring Archer Armor",
        (250, 0, 250, 0),
        70,
        Age::Imperial,
        "blacksmith",
        "Archers +1/+2 armour.",
    );
    t.requires = vec!["leather_archer_armor".into()];
    arch_armour(&mut t, 1, 2);
    v.push(t);

    // ---------------- Barracks ----------------
    v.push(line(
        "man_at_arms",
        "Man-at-Arms",
        (100, 0, 40, 0),
        40,
        Age::Feudal,
        "barracks",
        "militia",
        &[],
    ));
    v.push(line(
        "long_swordsman",
        "Long Swordsman",
        (200, 0, 65, 0),
        45,
        Age::Castle,
        "barracks",
        "man_at_arms",
        &["man_at_arms"],
    ));
    v.push(line(
        "two_handed_swordsman",
        "Two-Handed Swordsman",
        (300, 0, 100, 0),
        75,
        Age::Imperial,
        "barracks",
        "long_swordsman",
        &["long_swordsman"],
    ));
    v.push(line(
        "champion",
        "Champion",
        (750, 0, 350, 0),
        100,
        Age::Imperial,
        "barracks",
        "two_handed_swordsman",
        &["two_handed_swordsman"],
    ));
    v.push(line(
        "pikeman",
        "Pikeman",
        (215, 0, 90, 0),
        45,
        Age::Castle,
        "barracks",
        "spearman",
        &[],
    ));
    v.push(line(
        "halberdier",
        "Halberdier",
        (300, 0, 600, 0),
        50,
        Age::Imperial,
        "barracks",
        "pikeman",
        &["pikeman"],
    ));
    let mut t = tech(
        "squires",
        "Squires",
        (100, 0, 0, 0),
        40,
        Age::Castle,
        "barracks",
        "Infantry +10% speed.",
    );
    t.effects.push(add(role(Role::Infantry), A::Speed, 10));
    v.push(t);
    let mut t = tech(
        "arson",
        "Arson",
        (150, 0, 50, 0),
        25,
        Age::Castle,
        "barracks",
        "Infantry +2 attack vs standard buildings.",
    );
    t.effects.push(add(
        role(Role::Infantry),
        A::BonusAttack(C::StandardBuilding),
        2,
    ));
    v.push(t);

    // ---------------- Archery range ----------------
    v.push(line(
        "crossbowman",
        "Crossbowman",
        (125, 0, 75, 0),
        35,
        Age::Castle,
        "archery_range",
        "archer",
        &[],
    ));
    v.push(line(
        "arbalest",
        "Arbalest",
        (350, 0, 300, 0),
        50,
        Age::Imperial,
        "archery_range",
        "crossbowman",
        &["crossbowman"],
    ));
    v.push(line(
        "elite_skirmisher",
        "Elite Skirmisher",
        (0, 230, 130, 0),
        50,
        Age::Castle,
        "archery_range",
        "skirmisher",
        &[],
    ));
    v.push(line(
        "heavy_cavalry_archer",
        "Heavy Cavalry Archer",
        (900, 0, 500, 0),
        50,
        Age::Imperial,
        "archery_range",
        "cavalry_archer",
        &[],
    ));
    let mut t = tech(
        "thumb_ring",
        "Thumb Ring",
        (300, 0, 250, 0),
        45,
        Age::Castle,
        "archery_range",
        "Archers fire faster and never miss.",
    );
    t.effects.push(add(role(Role::Archer), A::Accuracy, 100));
    t.effects
        .push(add(role(Role::CavalryArcher), A::Accuracy, 100));
    t.effects.push(add(role(Role::Archer), A::ReloadTime, -10));
    t.effects
        .push(add(role(Role::CavalryArcher), A::ReloadTime, -10));
    v.push(t);
    let mut t = tech(
        "parthian_tactics",
        "Parthian Tactics",
        (200, 0, 250, 0),
        65,
        Age::Imperial,
        "archery_range",
        "Cavalry archers +1/+2 armour, +4 vs spearmen.",
    );
    t.effects
        .push(add(role(Role::CavalryArcher), A::MeleeArmour, 1));
    t.effects
        .push(add(role(Role::CavalryArcher), A::PierceArmour, 2));
    t.effects.push(add(
        role(Role::CavalryArcher),
        A::BonusAttack(C::Spearman),
        4,
    ));
    v.push(t);

    // ---------------- Stable ----------------
    v.push(line(
        "light_cavalry",
        "Light Cavalry",
        (150, 0, 50, 0),
        45,
        Age::Castle,
        "stable",
        "scout_cavalry",
        &[],
    ));
    v.push(line(
        "hussar",
        "Hussar",
        (500, 0, 600, 0),
        50,
        Age::Imperial,
        "stable",
        "light_cavalry",
        &["light_cavalry"],
    ));
    v.push(line(
        "cavalier",
        "Cavalier",
        (300, 0, 300, 0),
        100,
        Age::Imperial,
        "stable",
        "knight",
        &[],
    ));
    v.push(line(
        "paladin",
        "Paladin",
        (1300, 0, 750, 0),
        170,
        Age::Imperial,
        "stable",
        "cavalier",
        &["cavalier"],
    ));
    v.push(line(
        "heavy_camel",
        "Heavy Camel Rider",
        (325, 0, 360, 0),
        125,
        Age::Imperial,
        "stable",
        "camel",
        &[],
    ));
    let mut t = tech(
        "bloodlines",
        "Bloodlines",
        (150, 0, 100, 0),
        50,
        Age::Feudal,
        "stable",
        "Mounted units +20 HP.",
    );
    for r in [Role::Cavalry, Role::Scout, Role::CavalryArcher] {
        t.effects.push(add(role(r), A::Hp, 20));
    }
    v.push(t);
    let mut t = tech(
        "husbandry",
        "Husbandry",
        (150, 0, 0, 0),
        40,
        Age::Castle,
        "stable",
        "Mounted units +10% speed.",
    );
    for r in [Role::Cavalry, Role::Scout, Role::CavalryArcher] {
        t.effects.push(add(role(r), A::Speed, 10));
    }
    v.push(t);

    // ---------------- Siege workshop ----------------
    v.push(line(
        "capped_ram",
        "Capped Ram",
        (300, 0, 0, 0),
        50,
        Age::Castle,
        "siege_workshop",
        "battering_ram",
        &[],
    ));
    v.push(line(
        "siege_ram",
        "Siege Ram",
        (1000, 0, 0, 0),
        75,
        Age::Imperial,
        "siege_workshop",
        "capped_ram",
        &["capped_ram"],
    ));
    v.push(line(
        "onager",
        "Onager",
        (800, 0, 500, 0),
        75,
        Age::Imperial,
        "siege_workshop",
        "mangonel",
        &[],
    ));
    v.push(line(
        "siege_onager",
        "Siege Onager",
        (1450, 0, 750, 0),
        150,
        Age::Imperial,
        "siege_workshop",
        "onager",
        &["onager"],
    ));
    v.push(line(
        "heavy_scorpion",
        "Heavy Scorpion",
        (1000, 1100, 0, 0),
        50,
        Age::Imperial,
        "siege_workshop",
        "scorpion",
        &[],
    ));

    // ---------------- Dock ----------------
    v.push(line(
        "war_galley",
        "War Galley",
        (230, 0, 100, 0),
        50,
        Age::Castle,
        "dock",
        "galley",
        &[],
    ));
    v.push(line(
        "galleon",
        "Galleon",
        (400, 500, 0, 0),
        65,
        Age::Imperial,
        "dock",
        "war_galley",
        &["war_galley"],
    ));
    v.push(line(
        "fast_fire_ship",
        "Fast Fire Ship",
        (280, 0, 250, 0),
        50,
        Age::Imperial,
        "dock",
        "fire_ship",
        &[],
    ));
    let mut t = tech(
        "careening",
        "Careening",
        (250, 0, 150, 0),
        50,
        Age::Castle,
        "dock",
        "Ships +1 pierce armour, transports carry +5.",
    );
    t.effects.push(add(ac(C::Ship), A::PierceArmour, 1));
    v.push(t);
    let mut t = tech(
        "dry_dock",
        "Dry Dock",
        (600, 0, 400, 0),
        60,
        Age::Imperial,
        "dock",
        "Ships +15% speed.",
    );
    t.requires = vec!["careening".into()];
    t.effects.push(add(ac(C::Ship), A::Speed, 15));
    v.push(t);
    let mut t = tech(
        "shipwright",
        "Shipwright",
        (1000, 0, 300, 0),
        60,
        Age::Imperial,
        "dock",
        "Ships cost 20% less wood and build faster.",
    );
    t.effects.push(Effect::CostDiscount {
        target: ac(C::Ship),
        resource: Some(Resource::Wood),
        pct: 20,
    });
    v.push(t);

    // ---------------- Market ----------------
    let mut t = tech(
        "coinage",
        "Coinage",
        (200, 0, 100, 0),
        50,
        Age::Castle,
        "market",
        "Tribute is free of fees.",
    );
    t.effects.push(Effect::TradeFee { pct: 25 });
    v.push(t);
    let mut t = tech(
        "caravan",
        "Caravan",
        (200, 0, 200, 0),
        40,
        Age::Castle,
        "market",
        "Trade units +50% speed.",
    );
    t.effects.push(add(role(Role::Trade), A::Speed, 50));
    v.push(t);
    let mut t = tech(
        "banking",
        "Banking",
        (300, 0, 200, 0),
        70,
        Age::Imperial,
        "market",
        "Market fee reduced.",
    );
    t.requires = vec!["coinage".into()];
    t.effects.push(Effect::TradeFee { pct: 20 });
    v.push(t);
    let mut t = tech(
        "guilds",
        "Guilds",
        (300, 0, 200, 0),
        50,
        Age::Imperial,
        "market",
        "Market fee 15%.",
    );
    t.requires = vec!["banking".into()];
    t.effects.push(Effect::TradeFee { pct: 15 });
    v.push(t);

    // ---------------- Monastery ----------------
    let mut t = tech(
        "redemption",
        "Redemption",
        (0, 0, 475, 0),
        50,
        Age::Castle,
        "monastery",
        "Monks can convert buildings and siege.",
    );
    t.effects.push(add(obj("monk"), A::Los, 0));
    v.push(t);
    let mut t = tech(
        "atonement",
        "Atonement",
        (0, 0, 325, 0),
        40,
        Age::Castle,
        "monastery",
        "Monks can convert other monks.",
    );
    t.effects.push(add(obj("monk"), A::Los, 0));
    v.push(t);
    let mut t = tech(
        "herbal_medicine",
        "Herbal Medicine",
        (0, 0, 350, 0),
        35,
        Age::Castle,
        "monastery",
        "Garrisoned units heal faster.",
    );
    t.effects.push(add(obj("monk"), A::HealRate, 0));
    v.push(t);
    let mut t = tech(
        "heresy",
        "Heresy",
        (0, 0, 1000, 0),
        60,
        Age::Castle,
        "monastery",
        "Converted units die instead of changing sides (simplified: +8 conversion resistance).",
    );
    t.effects.push(Effect::ConversionResist { value: 8 });
    v.push(t);
    let mut t = tech(
        "sanctity",
        "Sanctity",
        (0, 0, 120, 0),
        60,
        Age::Castle,
        "monastery",
        "Monks +15 HP.",
    );
    t.effects.push(add(obj("monk"), A::Hp, 15));
    v.push(t);
    let mut t = tech(
        "fervor",
        "Fervor",
        (0, 0, 140, 0),
        50,
        Age::Castle,
        "monastery",
        "Monks +15% speed.",
    );
    t.effects.push(add(obj("monk"), A::Speed, 15));
    v.push(t);
    let mut t = tech(
        "faith",
        "Faith",
        (750, 0, 1000, 0),
        60,
        Age::Imperial,
        "monastery",
        "Units resist conversion.",
    );
    t.effects.push(Effect::ConversionResist { value: 4 });
    v.push(t);
    let mut t = tech(
        "illumination",
        "Illumination",
        (0, 0, 120, 0),
        65,
        Age::Imperial,
        "monastery",
        "Monks regain faith faster.",
    );
    t.effects.push(add(obj("monk"), A::ReloadTime, -50));
    v.push(t);
    let mut t = tech(
        "block_printing",
        "Block Printing",
        (0, 0, 200, 0),
        55,
        Age::Imperial,
        "monastery",
        "Monks +3 conversion range.",
    );
    t.effects.push(add(obj("monk"), A::Range, 3));
    v.push(t);
    let mut t = tech(
        "theocracy",
        "Theocracy",
        (0, 0, 200, 0),
        75,
        Age::Imperial,
        "monastery",
        "Monks heal twice as fast.",
    );
    t.effects.push(add(obj("monk"), A::HealRate, 2));
    v.push(t);

    // ---------------- University ----------------
    let mut t = tech(
        "masonry",
        "Masonry",
        (150, 175, 0, 0),
        50,
        Age::Castle,
        "university",
        "Buildings +10% HP, +1/+1 armour.",
    );
    t.effects
        .push(mul(Target::Class(ObjectClass::Building), A::Hp, 110));
    t.effects
        .push(add(Target::Class(ObjectClass::Building), A::MeleeArmour, 1));
    t.effects.push(add(
        Target::Class(ObjectClass::Building),
        A::PierceArmour,
        1,
    ));
    v.push(t);
    let mut t = tech(
        "architecture",
        "Architecture",
        (200, 300, 0, 0),
        70,
        Age::Imperial,
        "university",
        "Buildings +10% HP, +1/+1 armour.",
    );
    t.requires = vec!["masonry".into()];
    t.effects
        .push(mul(Target::Class(ObjectClass::Building), A::Hp, 110));
    t.effects
        .push(add(Target::Class(ObjectClass::Building), A::MeleeArmour, 1));
    t.effects.push(add(
        Target::Class(ObjectClass::Building),
        A::PierceArmour,
        1,
    ));
    v.push(t);
    let mut t = tech(
        "ballistics",
        "Ballistics",
        (0, 300, 175, 0),
        60,
        Age::Castle,
        "university",
        "Ranged units hit moving targets.",
    );
    t.effects.push(add(Target::All, A::Ballistics, 1));
    v.push(t);
    let mut t = tech(
        "murder_holes",
        "Murder Holes",
        (200, 0, 0, 100),
        60,
        Age::Castle,
        "university",
        "Towers and castles have no minimum range.",
    );
    t.effects.push(add(role(Role::Defensive), A::MinRange, -1));
    v.push(t);
    let mut t = tech(
        "treadmill_crane",
        "Treadmill Crane",
        (300, 200, 0, 0),
        50,
        Age::Castle,
        "university",
        "Villagers build 20% faster.",
    );
    t.effects.push(Effect::WorkRate {
        target: obj("villager"),
        pct: 120,
    });
    v.push(t);
    v.push(line(
        "guard_tower",
        "Guard Tower",
        (100, 250, 0, 0),
        30,
        Age::Castle,
        "university",
        "watch_tower",
        &[],
    ));
    v.push(line(
        "keep",
        "Keep",
        (500, 350, 0, 0),
        75,
        Age::Imperial,
        "university",
        "guard_tower",
        &["guard_tower"],
    ));
    v.push(line(
        "fortified_wall",
        "Fortified Wall",
        (200, 0, 0, 100),
        50,
        Age::Castle,
        "university",
        "stone_wall",
        &[],
    ));
    let mut t = tech(
        "chemistry",
        "Chemistry",
        (300, 0, 200, 0),
        100,
        Age::Imperial,
        "university",
        "Archers +1 attack; enables gunpowder units.",
    );
    t.effects.push(add(role(Role::Archer), A::PierceAttack, 1));
    t.effects
        .push(add(role(Role::CavalryArcher), A::PierceAttack, 1));
    t.effects
        .push(add(role(Role::Defensive), A::PierceAttack, 1));
    v.push(t);
    let mut t = tech(
        "bombard_tower",
        "Bombard Tower",
        (800, 0, 0, 400),
        60,
        Age::Imperial,
        "university",
        "Enables bombard towers.",
    );
    t.requires = vec!["chemistry".into()];
    t.effects.push(Effect::Enable {
        object: "bombard_tower".into(),
    });
    v.push(t);
    let mut t = tech(
        "siege_engineers",
        "Siege Engineers",
        (500, 600, 0, 0),
        45,
        Age::Imperial,
        "university",
        "Siege +1 range, +20% vs buildings.",
    );
    t.effects.push(add(role(Role::Siege), A::Range, 1));
    t.effects
        .push(add(role(Role::Siege), A::BonusAttack(C::Building), 20));
    v.push(t);
    let mut t = tech(
        "heated_shot",
        "Heated Shot",
        (350, 0, 100, 0),
        30,
        Age::Castle,
        "university",
        "Towers and castles +bonus vs ships.",
    );
    t.effects
        .push(add(role(Role::Defensive), A::BonusAttack(C::Ship), 6));
    t.effects
        .push(add(obj("castle"), A::BonusAttack(C::Ship), 6));
    v.push(t);

    // ---------------- Castle ----------------
    let mut t = tech(
        "conscription",
        "Conscription",
        (150, 0, 150, 0),
        60,
        Age::Imperial,
        "castle",
        "Military buildings work 33% faster.",
    );
    for b in [
        "barracks",
        "archery_range",
        "stable",
        "siege_workshop",
        "castle",
    ] {
        t.effects.push(Effect::WorkRate {
            target: obj(b),
            pct: 133,
        });
    }
    v.push(t);
    let mut t = tech(
        "hoardings",
        "Hoardings",
        (400, 400, 0, 0),
        75,
        Age::Imperial,
        "castle",
        "Castles +1000 HP.",
    );
    t.effects.push(add(obj("castle"), A::Hp, 1000));
    v.push(t);
    let mut t = tech(
        "sappers",
        "Sappers",
        (400, 0, 200, 0),
        30,
        Age::Imperial,
        "castle",
        "Villagers +15 attack vs buildings.",
    );
    t.effects
        .push(add(obj("villager"), A::BonusAttack(C::Building), 15));
    v.push(t);

    // Elite unique units.
    let elites: [(&str, &str, (i32, i32, i32, i32), &str); 13] = [
        (
            "longbowman",
            "Elite Longbowman",
            (850, 0, 850, 0),
            "britons",
        ),
        (
            "cataphract",
            "Elite Cataphract",
            (1200, 0, 800, 0),
            "byzantines",
        ),
        (
            "woad_raider",
            "Elite Woad Raider",
            (1000, 0, 800, 0),
            "celts",
        ),
        ("chu_ko_nu", "Elite Chu Ko Nu", (760, 0, 760, 0), "chinese"),
        (
            "throwing_axeman",
            "Elite Throwing Axeman",
            (1000, 0, 750, 0),
            "franks",
        ),
        ("huskarl", "Elite Huskarl", (1200, 0, 550, 0), "goths"),
        ("samurai", "Elite Samurai", (950, 0, 875, 0), "japanese"),
        ("mangudai", "Elite Mangudai", (1100, 0, 675, 0), "mongols"),
        (
            "war_elephant",
            "Elite War Elephant",
            (1600, 0, 1200, 0),
            "persians",
        ),
        ("mameluke", "Elite Mameluke", (600, 0, 500, 0), "saracens"),
        (
            "teutonic_knight",
            "Elite Teutonic Knight",
            (1200, 0, 600, 0),
            "teutons",
        ),
        ("janissary", "Elite Janissary", (850, 0, 750, 0), "turks"),
        ("berserk", "Elite Berserk", (1075, 0, 775, 0), "vikings"),
    ];
    for (base, name, cost, civ) in elites {
        let key = format!("elite_{base}");
        let mut t = line(&key, name, cost, 60, Age::Imperial, "castle", base, &[]);
        t.unique_to = Some(civ.into());
        v.push(t);
    }

    // Unique technologies (one per civ, Imperial Age, researched at the castle).
    let mut t = tech(
        "yeomen",
        "Yeomen",
        (0, 750, 450, 0),
        60,
        Age::Imperial,
        "castle",
        "Foot archers +1 range, towers +2 attack.",
    );
    t.unique_to = Some("britons".into());
    t.effects.push(add(role(Role::Archer), A::Range, 1));
    t.effects
        .push(add(role(Role::Defensive), A::PierceAttack, 2));
    v.push(t);
    let mut t = tech(
        "logistica",
        "Logistica",
        (800, 0, 600, 0),
        50,
        Age::Imperial,
        "castle",
        "Cataphracts +6 attack vs infantry.",
    );
    t.unique_to = Some("byzantines".into());
    for k in ["cataphract", "elite_cataphract"] {
        t.effects.push(add(obj(k), A::BonusAttack(C::Infantry), 6));
    }
    v.push(t);
    let mut t = tech(
        "furor_celtica",
        "Furor Celtica",
        (750, 0, 450, 0),
        50,
        Age::Imperial,
        "castle",
        "Siege workshop units +40% HP.",
    );
    t.unique_to = Some("celts".into());
    t.effects.push(mul(role(Role::Siege), A::Hp, 140));
    v.push(t);
    let mut t = tech(
        "rocketry",
        "Rocketry",
        (0, 600, 600, 0),
        60,
        Age::Imperial,
        "castle",
        "Chu Ko Nu +2 attack, scorpions +4 attack.",
    );
    t.unique_to = Some("chinese".into());
    for k in ["chu_ko_nu", "elite_chu_ko_nu"] {
        t.effects.push(add(obj(k), A::PierceAttack, 2));
    }
    for k in ["scorpion", "heavy_scorpion"] {
        t.effects.push(add(obj(k), A::PierceAttack, 4));
    }
    v.push(t);
    let mut t = tech(
        "bearded_axe",
        "Bearded Axe",
        (400, 0, 400, 0),
        60,
        Age::Imperial,
        "castle",
        "Throwing Axemen +1 range.",
    );
    t.unique_to = Some("franks".into());
    for k in ["throwing_axeman", "elite_throwing_axeman"] {
        t.effects.push(add(obj(k), A::Range, 1));
    }
    v.push(t);
    let mut t = tech(
        "anarchy",
        "Anarchy",
        (450, 0, 250, 0),
        40,
        Age::Castle,
        "castle",
        "Huskarls cost 20% less and train faster.",
    );
    t.unique_to = Some("goths".into());
    for k in ["huskarl", "elite_huskarl"] {
        t.effects.push(Effect::CostDiscount {
            target: obj(k),
            resource: None,
            pct: 20,
        });
        t.effects.push(mul(obj(k), A::TrainTime, 70));
    }
    v.push(t);
    let mut t = tech(
        "perfusion",
        "Perfusion",
        (0, 400, 600, 0),
        40,
        Age::Imperial,
        "castle",
        "Barracks work 100% faster.",
    );
    t.unique_to = Some("goths".into());
    t.effects.push(Effect::WorkRate {
        target: obj("barracks"),
        pct: 200,
    });
    v.push(t);
    let mut t = tech(
        "kataparuto",
        "Kataparuto",
        (0, 750, 400, 0),
        60,
        Age::Imperial,
        "castle",
        "Trebuchets fire and pack faster.",
    );
    t.unique_to = Some("japanese".into());
    t.effects.push(add(obj("trebuchet"), A::ReloadTime, -33));
    v.push(t);
    let mut t = tech(
        "drill",
        "Drill",
        (0, 500, 450, 0),
        35,
        Age::Imperial,
        "castle",
        "Siege workshop units +50% speed.",
    );
    t.unique_to = Some("mongols".into());
    t.effects.push(add(role(Role::Siege), A::Speed, 50));
    v.push(t);
    let mut t = tech(
        "mahouts",
        "Mahouts",
        (300, 0, 300, 0),
        50,
        Age::Imperial,
        "castle",
        "War Elephants +30% speed.",
    );
    t.unique_to = Some("persians".into());
    for k in ["war_elephant", "elite_war_elephant"] {
        t.effects.push(add(obj(k), A::Speed, 30));
    }
    v.push(t);
    let mut t = tech(
        "zealotry",
        "Zealotry",
        (750, 0, 800, 0),
        50,
        Age::Imperial,
        "castle",
        "Camels and Mamelukes +30 HP.",
    );
    t.unique_to = Some("saracens".into());
    t.effects.push(add(ac(C::Camel), A::Hp, 30));
    v.push(t);
    let mut t = tech(
        "crenellations",
        "Crenellations",
        (600, 0, 0, 400),
        60,
        Age::Imperial,
        "castle",
        "Castles +3 range; garrisoned infantry fire arrows.",
    );
    t.unique_to = Some("teutons".into());
    t.effects.push(add(obj("castle"), A::Range, 3));
    v.push(t);
    let mut t = tech(
        "artillery",
        "Artillery",
        (0, 500, 450, 0),
        40,
        Age::Imperial,
        "castle",
        "Bombard cannons, towers and cannon galleons +2 range.",
    );
    t.unique_to = Some("turks".into());
    for k in ["bombard_cannon", "bombard_tower", "cannon_galleon"] {
        t.effects.push(add(obj(k), A::Range, 2));
    }
    v.push(t);
    let mut t = tech(
        "berserkergang",
        "Berserkergang",
        (500, 0, 850, 0),
        40,
        Age::Imperial,
        "castle",
        "Berserks +15 HP (regeneration simplified).",
    );
    t.unique_to = Some("vikings".into());
    for k in ["berserk", "elite_berserk"] {
        t.effects.push(add(obj(k), A::Hp, 15));
    }
    v.push(t);

    v
}
