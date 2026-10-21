use crate::f;
use std::collections::BTreeMap;
use wars_sim::ids::*;
use wars_sim::types::*;
use wars_sim::Fixed;

fn base(key: &str, name: &str, class: ObjectClass, role: Role) -> ObjectType {
    ObjectType {
        key: key.into(),
        name: name.into(),
        class,
        role,
        hp: 1,
        attacks: vec![],
        armours: vec![],
        range: Fixed::ZERO,
        min_range: Fixed::ZERO,
        los: f("4"),
        speed: Fixed::ZERO,
        reload: f("2"),
        projectile_speed: Fixed::ZERO,
        accuracy: 100,
        cost: Stockpile::ZERO,
        train_time: 0,
        population: 0,
        pop_support: 0,
        size: (1, 1),
        garrison_capacity: 0,
        garrison_arrows: 0,
        trained_at: None,
        age: Age::Dark,
        requires: vec![],
        requires_tech: None,
        yields: None,
        drop_off: vec![],
        is_villager: false,
        can_convert: false,
        can_heal: false,
        is_naval: false,
        carries_units: 0,
        walkable: false,
        heal_rate: 0,
        conversion_resist: 0,
        trains: vec![],
        upgrades_to: None,
        is_market: false,
        is_dock: false,
        is_monastery: false,
        is_town_center: false,
        is_wonder: false,
        is_king: false,
        placement: Placement::Land,
        glyph: '?',
        art: 0,
    }
}

#[derive(Clone, Copy)]
enum Atk {
    Melee(i32),
    Pierce(i32),
    None,
}

/// cost = (food, wood, gold, stone); armour = (melee, pierce)
#[allow(clippy::too_many_arguments)]
fn unit(
    key: &str,
    name: &str,
    role: Role,
    cost: (i32, i32, i32, i32),
    hp: i32,
    atk: Atk,
    armour: (i32, i32),
    speed: &str,
    reload: &str,
    train: i32,
    at: &str,
    age: Age,
    glyph: char,
) -> ObjectType {
    let mut o = base(key, name, ObjectClass::Unit, role);
    o.cost = Stockpile::new(cost.0, cost.1, cost.2, cost.3);
    o.hp = hp;
    match atk {
        Atk::Melee(v) => o.attacks.push(Attack {
            class: ArmourClass::Melee,
            value: v,
        }),
        Atk::Pierce(v) => o.attacks.push(Attack {
            class: ArmourClass::Pierce,
            value: v,
        }),
        Atk::None => {}
    }
    o.armours.push(Armour {
        class: ArmourClass::Melee,
        value: armour.0,
    });
    o.armours.push(Armour {
        class: ArmourClass::Pierce,
        value: armour.1,
    });
    o.speed = f(speed);
    o.reload = f(reload);
    o.train_time = train;
    o.trained_at = Some(at.into());
    o.age = age;
    o.population = 1;
    o.los = f("5");
    o.glyph = glyph;
    o
}

#[allow(clippy::too_many_arguments)]
fn building(
    key: &str,
    name: &str,
    role: Role,
    cost: (i32, i32, i32, i32),
    hp: i32,
    armour: (i32, i32),
    size: (u8, u8),
    build: i32,
    age: Age,
    glyph: char,
) -> ObjectType {
    let mut o = base(key, name, ObjectClass::Building, role);
    o.cost = Stockpile::new(cost.0, cost.1, cost.2, cost.3);
    o.hp = hp;
    o.armours.push(Armour {
        class: ArmourClass::Melee,
        value: armour.0,
    });
    o.armours.push(Armour {
        class: ArmourClass::Pierce,
        value: armour.1,
    });
    o.armours.push(Armour {
        class: ArmourClass::Building,
        value: 0,
    });
    o.armours.push(Armour {
        class: ArmourClass::StandardBuilding,
        value: 0,
    });
    o.size = size;
    o.train_time = build;
    o.age = age;
    o.los = f("6");
    o.glyph = glyph;
    o
}

fn resource(key: &str, name: &str, kind: GatherKind, amount: i32, glyph: char) -> ObjectType {
    let mut o = base(key, name, ObjectClass::Resource, Role::None);
    o.hp = 1;
    o.yields = Some(Yield {
        resource: kind.resource(),
        amount,
        kind,
    });
    o.armours.push(Armour {
        class: ArmourClass::Melee,
        value: 0,
    });
    o.armours.push(Armour {
        class: ArmourClass::Pierce,
        value: 0,
    });
    o.glyph = glyph;
    o.placement = Placement::Any;
    o
}

fn bonus(o: &mut ObjectType, class: ArmourClass, value: i32) {
    o.attacks.push(Attack { class, value });
}
fn cls(o: &mut ObjectType, class: ArmourClass) {
    o.armours.push(Armour { class, value: 0 });
}
fn ranged(o: &mut ObjectType, range: &str, los: &str, projectile: &str, accuracy: i32) {
    o.range = f(range);
    o.los = f(los);
    o.projectile_speed = f(projectile);
    o.accuracy = accuracy;
}
fn upgrade(o: &mut ObjectType, tech: &str, to: &str) {
    o.upgrades_to = Some((tech.into(), to.into()));
}

pub fn gather_rates() -> BTreeMap<GatherKind, Fixed> {
    let mut m = BTreeMap::new();
    m.insert(GatherKind::Berries, f("0.31"));
    m.insert(GatherKind::Sheep, f("0.33"));
    m.insert(GatherKind::Hunt, f("0.41"));
    m.insert(GatherKind::Farm, f("0.32"));
    m.insert(GatherKind::Fish, f("0.43"));
    m.insert(GatherKind::Wood, f("0.39"));
    m.insert(GatherKind::Gold, f("0.38"));
    m.insert(GatherKind::Stone, f("0.36"));
    m
}

pub fn objects() -> Vec<ObjectType> {
    use ArmourClass as C;
    use Atk::{Melee, Pierce};
    let mut v: Vec<ObjectType> = Vec::new();

    // ---------------- Villager and economy units ----------------
    let mut o = unit(
        "villager",
        "Villager",
        Role::Villager,
        (50, 0, 0, 0),
        25,
        Melee(3),
        (0, 0),
        "0.8",
        "2.0",
        25,
        "town_center",
        Age::Dark,
        'v',
    );
    o.is_villager = true;
    o.los = f("4");
    bonus(&mut o, C::Building, 3);
    bonus(&mut o, C::Tree, 10);
    cls(&mut o, C::Villager);
    cls(&mut o, C::Infantry);
    v.push(o);

    let mut o = unit(
        "trade_cart",
        "Trade Cart",
        Role::Trade,
        (0, 100, 50, 0),
        70,
        Atk::None,
        (0, 0),
        "1.0",
        "2",
        51,
        "market",
        Age::Feudal,
        't',
    );
    o.los = f("7");
    v.push(o);

    let mut o = unit(
        "king",
        "King",
        Role::King,
        (0, 0, 0, 0),
        75,
        Atk::None,
        (0, 0),
        "1.32",
        "2",
        1,
        "castle",
        Age::Dark,
        'K',
    );
    o.is_king = true;
    o.los = f("6");
    o.trained_at = None;
    v.push(o);

    // ---------------- Barracks ----------------
    let mut o = unit(
        "militia",
        "Militia",
        Role::Infantry,
        (60, 0, 20, 0),
        40,
        Melee(4),
        (0, 1),
        "0.9",
        "2.0",
        21,
        "barracks",
        Age::Dark,
        'm',
    );
    cls(&mut o, C::Infantry);
    upgrade(&mut o, "man_at_arms", "man_at_arms");
    v.push(o);
    let mut o = unit(
        "man_at_arms",
        "Man-at-Arms",
        Role::Infantry,
        (60, 0, 20, 0),
        45,
        Melee(6),
        (0, 1),
        "0.9",
        "2.0",
        21,
        "barracks",
        Age::Feudal,
        'm',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 2);
    bonus(&mut o, C::StandardBuilding, 2);
    o.requires_tech = Some("man_at_arms".into());
    upgrade(&mut o, "long_swordsman", "long_swordsman");
    v.push(o);
    let mut o = unit(
        "long_swordsman",
        "Long Swordsman",
        Role::Infantry,
        (60, 0, 20, 0),
        60,
        Melee(9),
        (0, 1),
        "0.9",
        "2.0",
        21,
        "barracks",
        Age::Castle,
        'M',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 4);
    bonus(&mut o, C::StandardBuilding, 3);
    o.requires_tech = Some("long_swordsman".into());
    upgrade(&mut o, "two_handed_swordsman", "two_handed_swordsman");
    v.push(o);
    let mut o = unit(
        "two_handed_swordsman",
        "Two-Handed Swordsman",
        Role::Infantry,
        (60, 0, 20, 0),
        60,
        Melee(12),
        (0, 1),
        "0.9",
        "2.0",
        21,
        "barracks",
        Age::Imperial,
        'M',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 6);
    bonus(&mut o, C::StandardBuilding, 4);
    o.requires_tech = Some("two_handed_swordsman".into());
    upgrade(&mut o, "champion", "champion");
    v.push(o);
    let mut o = unit(
        "champion",
        "Champion",
        Role::Infantry,
        (60, 0, 20, 0),
        70,
        Melee(13),
        (1, 1),
        "0.9",
        "2.0",
        21,
        "barracks",
        Age::Imperial,
        'M',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 8);
    bonus(&mut o, C::StandardBuilding, 4);
    o.requires_tech = Some("champion".into());
    v.push(o);

    let mut o = unit(
        "spearman",
        "Spearman",
        Role::Infantry,
        (35, 25, 0, 0),
        45,
        Melee(3),
        (0, 0),
        "1.0",
        "3.0",
        22,
        "barracks",
        Age::Feudal,
        's',
    );
    cls(&mut o, C::Infantry);
    cls(&mut o, C::Spearman);
    bonus(&mut o, C::Cavalry, 15);
    bonus(&mut o, C::Elephant, 15);
    bonus(&mut o, C::Camel, 12);
    bonus(&mut o, C::Ship, 9);
    bonus(&mut o, C::Eagle, 1);
    upgrade(&mut o, "pikeman", "pikeman");
    v.push(o);
    let mut o = unit(
        "pikeman",
        "Pikeman",
        Role::Infantry,
        (35, 25, 0, 0),
        55,
        Melee(4),
        (0, 0),
        "1.0",
        "3.0",
        22,
        "barracks",
        Age::Castle,
        's',
    );
    cls(&mut o, C::Infantry);
    cls(&mut o, C::Spearman);
    bonus(&mut o, C::Cavalry, 22);
    bonus(&mut o, C::Elephant, 25);
    bonus(&mut o, C::Camel, 18);
    bonus(&mut o, C::Ship, 16);
    bonus(&mut o, C::Eagle, 1);
    o.requires_tech = Some("pikeman".into());
    upgrade(&mut o, "halberdier", "halberdier");
    v.push(o);
    let mut o = unit(
        "halberdier",
        "Halberdier",
        Role::Infantry,
        (35, 25, 0, 0),
        60,
        Melee(6),
        (0, 0),
        "1.0",
        "3.0",
        22,
        "barracks",
        Age::Imperial,
        'S',
    );
    cls(&mut o, C::Infantry);
    cls(&mut o, C::Spearman);
    bonus(&mut o, C::Cavalry, 32);
    bonus(&mut o, C::Elephant, 28);
    bonus(&mut o, C::Camel, 26);
    bonus(&mut o, C::Ship, 17);
    bonus(&mut o, C::Eagle, 1);
    o.requires_tech = Some("halberdier".into());
    v.push(o);

    // ---------------- Archery range ----------------
    let mut o = unit(
        "archer",
        "Archer",
        Role::Archer,
        (0, 25, 45, 0),
        30,
        Pierce(4),
        (0, 0),
        "0.96",
        "2.0",
        35,
        "archery_range",
        Age::Feudal,
        'a',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "4", "6", "7", 80);
    bonus(&mut o, C::Spearman, 3);
    upgrade(&mut o, "crossbowman", "crossbowman");
    v.push(o);
    let mut o = unit(
        "crossbowman",
        "Crossbowman",
        Role::Archer,
        (0, 25, 45, 0),
        35,
        Pierce(5),
        (0, 0),
        "0.96",
        "2.0",
        27,
        "archery_range",
        Age::Castle,
        'a',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "5", "7", "7", 85);
    bonus(&mut o, C::Spearman, 3);
    o.requires_tech = Some("crossbowman".into());
    upgrade(&mut o, "arbalest", "arbalest");
    v.push(o);
    let mut o = unit(
        "arbalest",
        "Arbalest",
        Role::Archer,
        (0, 25, 45, 0),
        40,
        Pierce(6),
        (0, 0),
        "0.96",
        "2.0",
        27,
        "archery_range",
        Age::Imperial,
        'A',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "5", "7", "7", 90);
    bonus(&mut o, C::Spearman, 3);
    o.requires_tech = Some("arbalest".into());
    v.push(o);

    let mut o = unit(
        "skirmisher",
        "Skirmisher",
        Role::Archer,
        (25, 35, 0, 0),
        30,
        Pierce(2),
        (0, 3),
        "0.96",
        "3.0",
        22,
        "archery_range",
        Age::Feudal,
        'k',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Skirmisher);
    ranged(&mut o, "4", "6", "7", 90);
    o.min_range = f("1");
    bonus(&mut o, C::Archer, 3);
    bonus(&mut o, C::Spearman, 3);
    bonus(&mut o, C::CavalryArcher, 2);
    upgrade(&mut o, "elite_skirmisher", "elite_skirmisher");
    v.push(o);
    let mut o = unit(
        "elite_skirmisher",
        "Elite Skirmisher",
        Role::Archer,
        (25, 35, 0, 0),
        35,
        Pierce(3),
        (0, 4),
        "0.96",
        "3.0",
        22,
        "archery_range",
        Age::Castle,
        'k',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Skirmisher);
    ranged(&mut o, "5", "7", "7", 90);
    o.min_range = f("1");
    bonus(&mut o, C::Archer, 4);
    bonus(&mut o, C::Spearman, 3);
    bonus(&mut o, C::CavalryArcher, 3);
    o.requires_tech = Some("elite_skirmisher".into());
    v.push(o);

    let mut o = unit(
        "cavalry_archer",
        "Cavalry Archer",
        Role::CavalryArcher,
        (0, 40, 70, 0),
        50,
        Pierce(6),
        (0, 0),
        "1.4",
        "2.0",
        34,
        "archery_range",
        Age::Castle,
        'c',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::CavalryArcher);
    ranged(&mut o, "4", "5", "7", 50);
    bonus(&mut o, C::Spearman, 2);
    upgrade(&mut o, "heavy_cavalry_archer", "heavy_cavalry_archer");
    v.push(o);
    let mut o = unit(
        "heavy_cavalry_archer",
        "Heavy Cavalry Archer",
        Role::CavalryArcher,
        (0, 40, 70, 0),
        60,
        Pierce(7),
        (1, 0),
        "1.4",
        "2.0",
        27,
        "archery_range",
        Age::Imperial,
        'C',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::CavalryArcher);
    ranged(&mut o, "4", "6", "7", 50);
    bonus(&mut o, C::Spearman, 2);
    o.requires_tech = Some("heavy_cavalry_archer".into());
    v.push(o);

    // ---------------- Stable ----------------
    let mut o = unit(
        "scout_cavalry",
        "Scout Cavalry",
        Role::Scout,
        (80, 0, 0, 0),
        45,
        Melee(3),
        (0, 2),
        "1.2",
        "2.0",
        30,
        "stable",
        Age::Dark,
        'h',
    );
    cls(&mut o, C::Cavalry);
    o.los = f("4");
    bonus(&mut o, C::Monk, 6);
    o.conversion_resist = 4;
    upgrade(&mut o, "light_cavalry", "light_cavalry");
    v.push(o);
    let mut o = unit(
        "light_cavalry",
        "Light Cavalry",
        Role::Cavalry,
        (80, 0, 0, 0),
        60,
        Melee(7),
        (0, 2),
        "1.5",
        "2.0",
        30,
        "stable",
        Age::Castle,
        'h',
    );
    cls(&mut o, C::Cavalry);
    o.los = f("8");
    bonus(&mut o, C::Monk, 10);
    o.conversion_resist = 4;
    o.requires_tech = Some("light_cavalry".into());
    upgrade(&mut o, "hussar", "hussar");
    v.push(o);
    let mut o = unit(
        "hussar",
        "Hussar",
        Role::Cavalry,
        (80, 0, 0, 0),
        75,
        Melee(7),
        (0, 2),
        "1.5",
        "1.9",
        30,
        "stable",
        Age::Imperial,
        'H',
    );
    cls(&mut o, C::Cavalry);
    o.los = f("10");
    bonus(&mut o, C::Monk, 12);
    o.conversion_resist = 4;
    o.requires_tech = Some("hussar".into());
    v.push(o);

    let mut o = unit(
        "knight",
        "Knight",
        Role::Cavalry,
        (60, 0, 75, 0),
        100,
        Melee(10),
        (2, 2),
        "1.35",
        "1.8",
        30,
        "stable",
        Age::Castle,
        'n',
    );
    cls(&mut o, C::Cavalry);
    v.push(o);
    let _ = upgrade;
    v.last_mut().map(|o| upgrade(o, "cavalier", "cavalier"));
    let mut o = unit(
        "cavalier",
        "Cavalier",
        Role::Cavalry,
        (60, 0, 75, 0),
        120,
        Melee(12),
        (2, 2),
        "1.35",
        "1.8",
        30,
        "stable",
        Age::Imperial,
        'n',
    );
    cls(&mut o, C::Cavalry);
    o.requires_tech = Some("cavalier".into());
    upgrade(&mut o, "paladin", "paladin");
    v.push(o);
    let mut o = unit(
        "paladin",
        "Paladin",
        Role::Cavalry,
        (60, 0, 75, 0),
        160,
        Melee(14),
        (2, 3),
        "1.35",
        "1.9",
        30,
        "stable",
        Age::Imperial,
        'N',
    );
    cls(&mut o, C::Cavalry);
    o.requires_tech = Some("paladin".into());
    v.push(o);

    let mut o = unit(
        "camel",
        "Camel Rider",
        Role::Cavalry,
        (55, 0, 60, 0),
        100,
        Melee(5),
        (0, 0),
        "1.45",
        "2.0",
        22,
        "stable",
        Age::Castle,
        'd',
    );
    cls(&mut o, C::Camel);
    bonus(&mut o, C::Cavalry, 9);
    bonus(&mut o, C::Camel, 5);
    bonus(&mut o, C::Ship, 5);
    upgrade(&mut o, "heavy_camel", "heavy_camel");
    v.push(o);
    let mut o = unit(
        "heavy_camel",
        "Heavy Camel Rider",
        Role::Cavalry,
        (55, 0, 60, 0),
        120,
        Melee(7),
        (0, 0),
        "1.45",
        "2.0",
        22,
        "stable",
        Age::Imperial,
        'D',
    );
    cls(&mut o, C::Camel);
    bonus(&mut o, C::Cavalry, 18);
    bonus(&mut o, C::Camel, 9);
    bonus(&mut o, C::Ship, 9);
    o.requires_tech = Some("heavy_camel".into());
    v.push(o);

    // ---------------- Monastery ----------------
    let mut o = unit(
        "monk",
        "Monk",
        Role::Monk,
        (0, 0, 100, 0),
        30,
        Atk::None,
        (0, 0),
        "0.7",
        "2",
        51,
        "monastery",
        Age::Castle,
        'p',
    );
    cls(&mut o, C::Monk);
    o.can_convert = true;
    o.can_heal = true;
    o.heal_rate = 2;
    o.range = f("9");
    o.los = f("11");
    v.push(o);

    // ---------------- Siege workshop ----------------
    let mut o = unit(
        "battering_ram",
        "Battering Ram",
        Role::Siege,
        (0, 160, 75, 0),
        175,
        Melee(2),
        (-3, 180),
        "0.5",
        "5.0",
        36,
        "siege_workshop",
        Age::Castle,
        'r',
    );
    cls(&mut o, C::Siege);
    cls(&mut o, C::Ram);
    bonus(&mut o, C::Building, 125);
    bonus(&mut o, C::Siege, 40);
    o.garrison_capacity = 4;
    o.conversion_resist = 4;
    upgrade(&mut o, "capped_ram", "capped_ram");
    v.push(o);
    let mut o = unit(
        "capped_ram",
        "Capped Ram",
        Role::Siege,
        (0, 160, 75, 0),
        200,
        Melee(3),
        (-3, 190),
        "0.5",
        "5.0",
        36,
        "siege_workshop",
        Age::Imperial,
        'r',
    );
    cls(&mut o, C::Siege);
    cls(&mut o, C::Ram);
    bonus(&mut o, C::Building, 150);
    bonus(&mut o, C::Siege, 50);
    o.garrison_capacity = 5;
    o.conversion_resist = 4;
    o.requires_tech = Some("capped_ram".into());
    upgrade(&mut o, "siege_ram", "siege_ram");
    v.push(o);
    let mut o = unit(
        "siege_ram",
        "Siege Ram",
        Role::Siege,
        (0, 160, 75, 0),
        270,
        Melee(4),
        (-3, 195),
        "0.6",
        "5.0",
        36,
        "siege_workshop",
        Age::Imperial,
        'R',
    );
    cls(&mut o, C::Siege);
    cls(&mut o, C::Ram);
    bonus(&mut o, C::Building, 200);
    bonus(&mut o, C::Siege, 65);
    o.garrison_capacity = 6;
    o.conversion_resist = 4;
    o.requires_tech = Some("siege_ram".into());
    v.push(o);

    let mut o = unit(
        "mangonel",
        "Mangonel",
        Role::Siege,
        (0, 160, 135, 0),
        50,
        Melee(40),
        (0, 6),
        "0.6",
        "6.0",
        46,
        "siege_workshop",
        Age::Castle,
        'g',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "7", "9", "4", 100);
    o.min_range = f("3");
    bonus(&mut o, C::Building, 12);
    bonus(&mut o, C::Siege, 12);
    o.conversion_resist = 4;
    upgrade(&mut o, "onager", "onager");
    v.push(o);
    let mut o = unit(
        "onager",
        "Onager",
        Role::Siege,
        (0, 160, 135, 0),
        60,
        Melee(50),
        (0, 7),
        "0.6",
        "6.0",
        46,
        "siege_workshop",
        Age::Imperial,
        'g',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "8", "10", "4", 100);
    o.min_range = f("3");
    bonus(&mut o, C::Building, 12);
    bonus(&mut o, C::Siege, 12);
    o.conversion_resist = 4;
    o.requires_tech = Some("onager".into());
    upgrade(&mut o, "siege_onager", "siege_onager");
    v.push(o);
    let mut o = unit(
        "siege_onager",
        "Siege Onager",
        Role::Siege,
        (0, 160, 135, 0),
        70,
        Melee(75),
        (0, 8),
        "0.6",
        "6.0",
        46,
        "siege_workshop",
        Age::Imperial,
        'G',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "8", "10", "4", 100);
    o.min_range = f("3");
    bonus(&mut o, C::Building, 12);
    bonus(&mut o, C::Siege, 12);
    o.conversion_resist = 4;
    o.requires_tech = Some("siege_onager".into());
    v.push(o);

    let mut o = unit(
        "scorpion",
        "Scorpion",
        Role::Siege,
        (0, 75, 75, 0),
        40,
        Pierce(12),
        (0, 7),
        "0.65",
        "3.6",
        30,
        "siege_workshop",
        Age::Castle,
        'x',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "7", "9", "6", 100);
    o.min_range = f("2");
    bonus(&mut o, C::Elephant, 6);
    bonus(&mut o, C::Building, 2);
    o.conversion_resist = 4;
    upgrade(&mut o, "heavy_scorpion", "heavy_scorpion");
    v.push(o);
    let mut o = unit(
        "heavy_scorpion",
        "Heavy Scorpion",
        Role::Siege,
        (0, 75, 75, 0),
        50,
        Pierce(16),
        (0, 7),
        "0.65",
        "3.6",
        30,
        "siege_workshop",
        Age::Imperial,
        'X',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "7", "9", "6", 100);
    o.min_range = f("2");
    bonus(&mut o, C::Elephant, 8);
    bonus(&mut o, C::Building, 4);
    o.conversion_resist = 4;
    o.requires_tech = Some("heavy_scorpion".into());
    v.push(o);

    let mut o = unit(
        "bombard_cannon",
        "Bombard Cannon",
        Role::Siege,
        (0, 225, 225, 0),
        80,
        Melee(40),
        (2, 5),
        "0.7",
        "6.5",
        56,
        "siege_workshop",
        Age::Imperial,
        'b',
    );
    cls(&mut o, C::Siege);
    cls(&mut o, C::Gunpowder);
    ranged(&mut o, "12", "14", "3", 95);
    o.min_range = f("5");
    bonus(&mut o, C::Building, 200);
    bonus(&mut o, C::Ship, 40);
    bonus(&mut o, C::Siege, 20);
    o.requires_tech = Some("chemistry".into());
    o.conversion_resist = 4;
    v.push(o);

    let mut o = unit(
        "trebuchet",
        "Trebuchet",
        Role::Siege,
        (0, 200, 200, 0),
        150,
        Melee(200),
        (2, 8),
        "0.8",
        "10.0",
        50,
        "castle",
        Age::Imperial,
        'T',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "16", "19", "3", 15);
    o.min_range = f("4");
    bonus(&mut o, C::Building, 250);
    o.conversion_resist = 8;
    v.push(o);

    // ---------------- Dock ----------------
    let mut o = unit(
        "fishing_ship",
        "Fishing Ship",
        Role::Fishing,
        (0, 75, 0, 0),
        60,
        Atk::None,
        (0, 4),
        "1.26",
        "2",
        40,
        "dock",
        Age::Dark,
        'f',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    cls(&mut o, C::Fishing);
    o.placement = Placement::Water;
    v.push(o);
    let mut o = unit(
        "transport_ship",
        "Transport Ship",
        Role::Transport,
        (0, 125, 0, 0),
        100,
        Atk::None,
        (4, 8),
        "1.45",
        "2",
        46,
        "dock",
        Age::Dark,
        'u',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    o.carries_units = 5;
    o.placement = Placement::Water;
    v.push(o);
    let mut o = unit(
        "trade_cog",
        "Trade Cog",
        Role::Trade,
        (0, 100, 50, 0),
        80,
        Atk::None,
        (0, 6),
        "1.32",
        "2",
        36,
        "dock",
        Age::Feudal,
        'o',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    o.placement = Placement::Water;
    v.push(o);
    let mut o = unit(
        "galley",
        "Galley",
        Role::Naval,
        (0, 90, 30, 0),
        120,
        Pierce(6),
        (0, 6),
        "1.43",
        "3.0",
        60,
        "dock",
        Age::Feudal,
        'y',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    ranged(&mut o, "5", "7", "7", 90);
    bonus(&mut o, C::Ship, 6);
    bonus(&mut o, C::Building, 6);
    upgrade(&mut o, "war_galley", "war_galley");
    v.push(o);
    let mut o = unit(
        "war_galley",
        "War Galley",
        Role::Naval,
        (0, 90, 30, 0),
        135,
        Pierce(7),
        (0, 6),
        "1.43",
        "3.0",
        36,
        "dock",
        Age::Castle,
        'y',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    ranged(&mut o, "6", "8", "7", 90);
    bonus(&mut o, C::Ship, 7);
    bonus(&mut o, C::Building, 7);
    o.requires_tech = Some("war_galley".into());
    upgrade(&mut o, "galleon", "galleon");
    v.push(o);
    let mut o = unit(
        "galleon",
        "Galleon",
        Role::Naval,
        (0, 90, 30, 0),
        165,
        Pierce(8),
        (0, 8),
        "1.43",
        "3.0",
        36,
        "dock",
        Age::Imperial,
        'Y',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    ranged(&mut o, "7", "9", "7", 90);
    bonus(&mut o, C::Ship, 8);
    bonus(&mut o, C::Building, 8);
    o.requires_tech = Some("galleon".into());
    v.push(o);
    let mut o = unit(
        "fire_ship",
        "Fire Ship",
        Role::Naval,
        (0, 75, 45, 0),
        100,
        Pierce(2),
        (0, 6),
        "1.35",
        "0.25",
        36,
        "dock",
        Age::Castle,
        'i',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    ranged(&mut o, "2.5", "5", "0", 100);
    bonus(&mut o, C::Ship, 2);
    bonus(&mut o, C::Building, 1);
    upgrade(&mut o, "fast_fire_ship", "fast_fire_ship");
    v.push(o);
    let mut o = unit(
        "fast_fire_ship",
        "Fast Fire Ship",
        Role::Naval,
        (0, 75, 45, 0),
        120,
        Pierce(3),
        (0, 8),
        "1.43",
        "0.25",
        36,
        "dock",
        Age::Imperial,
        'I',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    ranged(&mut o, "2.5", "6", "0", 100);
    bonus(&mut o, C::Ship, 3);
    bonus(&mut o, C::Building, 1);
    o.requires_tech = Some("fast_fire_ship".into());
    v.push(o);
    let mut o = unit(
        "cannon_galleon",
        "Cannon Galleon",
        Role::Naval,
        (0, 200, 150, 0),
        120,
        Melee(35),
        (0, 6),
        "1.1",
        "10.0",
        46,
        "dock",
        Age::Imperial,
        'Q',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    cls(&mut o, C::Gunpowder);
    ranged(&mut o, "13", "15", "2", 50);
    o.min_range = f("3");
    bonus(&mut o, C::Building, 200);
    bonus(&mut o, C::Siege, 40);
    o.requires_tech = Some("chemistry".into());
    v.push(o);
    let mut o = unit(
        "longboat",
        "Longboat",
        Role::Naval,
        (0, 100, 50, 0),
        130,
        Pierce(7),
        (0, 6),
        "1.54",
        "3.0",
        25,
        "dock",
        Age::Castle,
        'L',
    );
    o.is_naval = true;
    cls(&mut o, C::Ship);
    cls(&mut o, C::UniqueUnit);
    ranged(&mut o, "6", "8", "7", 90);
    bonus(&mut o, C::Ship, 6);
    bonus(&mut o, C::Building, 8);
    o.requires = vec!["castle".into()];
    v.push(o);

    // ---------------- Unique units (castle) ----------------
    fn uu(v: &mut Vec<ObjectType>, mut o: ObjectType, elite_tech: Option<&str>) {
        cls(&mut o, ArmourClass::UniqueUnit);
        if let Some(t) = elite_tech {
            let elite = format!("elite_{}", o.key);
            upgrade(&mut o, t, &elite);
        } else {
            o.requires_tech = Some(format!("elite_{}", o.key.trim_start_matches("elite_")));
        }
        v.push(o);
    }
    let mut o = unit(
        "longbowman",
        "Longbowman",
        Role::Archer,
        (0, 35, 40, 0),
        35,
        Pierce(6),
        (0, 0),
        "0.96",
        "2.0",
        19,
        "castle",
        Age::Castle,
        'l',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "5", "7", "7", 70);
    bonus(&mut o, C::Spearman, 2);
    uu(&mut v, o, Some("elite_longbowman"));
    let mut o = unit(
        "elite_longbowman",
        "Elite Longbowman",
        Role::Archer,
        (0, 35, 40, 0),
        40,
        Pierce(7),
        (0, 1),
        "0.96",
        "2.0",
        19,
        "castle",
        Age::Imperial,
        'L',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "6", "8", "7", 75);
    bonus(&mut o, C::Spearman, 2);
    uu(&mut v, o, None);

    let mut o = unit(
        "cataphract",
        "Cataphract",
        Role::Cavalry,
        (70, 0, 75, 0),
        110,
        Melee(9),
        (2, 1),
        "1.35",
        "1.8",
        20,
        "castle",
        Age::Castle,
        'q',
    );
    cls(&mut o, C::Cavalry);
    bonus(&mut o, C::Infantry, 9);
    uu(&mut v, o, Some("elite_cataphract"));
    let mut o = unit(
        "elite_cataphract",
        "Elite Cataphract",
        Role::Cavalry,
        (70, 0, 75, 0),
        150,
        Melee(12),
        (2, 1),
        "1.35",
        "1.7",
        20,
        "castle",
        Age::Imperial,
        'Q',
    );
    cls(&mut o, C::Cavalry);
    bonus(&mut o, C::Infantry, 12);
    uu(&mut v, o, None);

    let mut o = unit(
        "woad_raider",
        "Woad Raider",
        Role::Infantry,
        (65, 0, 25, 0),
        65,
        Melee(8),
        (0, 1),
        "1.2",
        "2.0",
        10,
        "castle",
        Age::Castle,
        'w',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::StandardBuilding, 2);
    bonus(&mut o, C::Eagle, 2);
    uu(&mut v, o, Some("elite_woad_raider"));
    let mut o = unit(
        "elite_woad_raider",
        "Elite Woad Raider",
        Role::Infantry,
        (65, 0, 25, 0),
        80,
        Melee(13),
        (0, 1),
        "1.2",
        "2.0",
        10,
        "castle",
        Age::Imperial,
        'W',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::StandardBuilding, 3);
    bonus(&mut o, C::Eagle, 3);
    uu(&mut v, o, None);

    let mut o = unit(
        "chu_ko_nu",
        "Chu Ko Nu",
        Role::Archer,
        (0, 40, 35, 0),
        45,
        Pierce(8),
        (0, 0),
        "0.96",
        "3.0",
        16,
        "castle",
        Age::Castle,
        'z',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "4", "6", "7", 85);
    bonus(&mut o, C::Spearman, 2);
    uu(&mut v, o, Some("elite_chu_ko_nu"));
    let mut o = unit(
        "elite_chu_ko_nu",
        "Elite Chu Ko Nu",
        Role::Archer,
        (0, 40, 35, 0),
        50,
        Pierce(10),
        (0, 0),
        "0.96",
        "3.0",
        13,
        "castle",
        Age::Imperial,
        'Z',
    );
    cls(&mut o, C::Archer);
    ranged(&mut o, "4", "6", "7", 85);
    bonus(&mut o, C::Spearman, 2);
    uu(&mut v, o, None);

    let mut o = unit(
        "throwing_axeman",
        "Throwing Axeman",
        Role::Infantry,
        (55, 0, 25, 0),
        50,
        Melee(7),
        (0, 0),
        "1.0",
        "2.0",
        17,
        "castle",
        Age::Castle,
        'e',
    );
    cls(&mut o, C::Infantry);
    ranged(&mut o, "3", "5", "5", 100);
    bonus(&mut o, C::StandardBuilding, 1);
    uu(&mut v, o, Some("elite_throwing_axeman"));
    let mut o = unit(
        "elite_throwing_axeman",
        "Elite Throwing Axeman",
        Role::Infantry,
        (55, 0, 25, 0),
        60,
        Melee(8),
        (1, 0),
        "1.0",
        "2.0",
        17,
        "castle",
        Age::Imperial,
        'E',
    );
    cls(&mut o, C::Infantry);
    ranged(&mut o, "4", "6", "5", 100);
    bonus(&mut o, C::StandardBuilding, 2);
    uu(&mut v, o, None);

    let mut o = unit(
        "huskarl",
        "Huskarl",
        Role::Infantry,
        (80, 0, 40, 0),
        60,
        Melee(10),
        (0, 6),
        "1.05",
        "2.0",
        13,
        "castle",
        Age::Castle,
        'j',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Archer, 6);
    bonus(&mut o, C::Eagle, 2);
    uu(&mut v, o, Some("elite_huskarl"));
    let mut o = unit(
        "elite_huskarl",
        "Elite Huskarl",
        Role::Infantry,
        (80, 0, 40, 0),
        70,
        Melee(12),
        (0, 8),
        "1.05",
        "2.0",
        13,
        "castle",
        Age::Imperial,
        'J',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Archer, 10);
    bonus(&mut o, C::Eagle, 3);
    uu(&mut v, o, None);

    let mut o = unit(
        "samurai",
        "Samurai",
        Role::Infantry,
        (60, 0, 30, 0),
        60,
        Melee(8),
        (1, 1),
        "1.0",
        "1.9",
        9,
        "castle",
        Age::Castle,
        '1',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::UniqueUnit, 10);
    bonus(&mut o, C::Eagle, 2);
    uu(&mut v, o, Some("elite_samurai"));
    let mut o = unit(
        "elite_samurai",
        "Elite Samurai",
        Role::Infantry,
        (60, 0, 30, 0),
        80,
        Melee(12),
        (1, 1),
        "1.0",
        "1.9",
        9,
        "castle",
        Age::Imperial,
        '!',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::UniqueUnit, 12);
    bonus(&mut o, C::Eagle, 3);
    uu(&mut v, o, None);

    let mut o = unit(
        "mangudai",
        "Mangudai",
        Role::CavalryArcher,
        (0, 55, 65, 0),
        60,
        Pierce(6),
        (0, 0),
        "1.45",
        "2.1",
        26,
        "castle",
        Age::Castle,
        '2',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::CavalryArcher);
    ranged(&mut o, "4", "6", "7", 95);
    bonus(&mut o, C::Siege, 3);
    bonus(&mut o, C::Spearman, 1);
    uu(&mut v, o, Some("elite_mangudai"));
    let mut o = unit(
        "elite_mangudai",
        "Elite Mangudai",
        Role::CavalryArcher,
        (0, 55, 65, 0),
        60,
        Pierce(8),
        (1, 0),
        "1.45",
        "2.1",
        26,
        "castle",
        Age::Imperial,
        '@',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::CavalryArcher);
    ranged(&mut o, "4", "6", "7", 95);
    bonus(&mut o, C::Siege, 5);
    bonus(&mut o, C::Spearman, 1);
    uu(&mut v, o, None);

    let mut o = unit(
        "war_elephant",
        "War Elephant",
        Role::Cavalry,
        (200, 0, 75, 0),
        450,
        Melee(15),
        (1, 2),
        "0.6",
        "2.0",
        31,
        "castle",
        Age::Castle,
        '3',
    );
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::Elephant);
    bonus(&mut o, C::Building, 7);
    o.conversion_resist = 4;
    uu(&mut v, o, Some("elite_war_elephant"));
    let mut o = unit(
        "elite_war_elephant",
        "Elite War Elephant",
        Role::Cavalry,
        (200, 0, 75, 0),
        600,
        Melee(20),
        (1, 3),
        "0.6",
        "2.0",
        31,
        "castle",
        Age::Imperial,
        '#',
    );
    cls(&mut o, C::Cavalry);
    cls(&mut o, C::Elephant);
    bonus(&mut o, C::Building, 10);
    o.conversion_resist = 4;
    uu(&mut v, o, None);

    let mut o = unit(
        "mameluke",
        "Mameluke",
        Role::Cavalry,
        (55, 0, 85, 0),
        65,
        Melee(7),
        (0, 0),
        "1.4",
        "2.0",
        23,
        "castle",
        Age::Castle,
        '4',
    );
    cls(&mut o, C::Camel);
    ranged(&mut o, "3", "5", "5", 100);
    bonus(&mut o, C::Cavalry, 9);
    bonus(&mut o, C::Camel, 5);
    uu(&mut v, o, Some("elite_mameluke"));
    let mut o = unit(
        "elite_mameluke",
        "Elite Mameluke",
        Role::Cavalry,
        (55, 0, 85, 0),
        80,
        Melee(10),
        (1, 0),
        "1.4",
        "2.0",
        23,
        "castle",
        Age::Imperial,
        '$',
    );
    cls(&mut o, C::Camel);
    ranged(&mut o, "3", "5", "5", 100);
    bonus(&mut o, C::Cavalry, 12);
    bonus(&mut o, C::Camel, 7);
    uu(&mut v, o, None);

    let mut o = unit(
        "teutonic_knight",
        "Teutonic Knight",
        Role::Infantry,
        (85, 0, 40, 0),
        80,
        Melee(12),
        (5, 2),
        "0.7",
        "2.0",
        12,
        "castle",
        Age::Castle,
        '5',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 4);
    uu(&mut v, o, Some("elite_teutonic_knight"));
    let mut o = unit(
        "elite_teutonic_knight",
        "Elite Teutonic Knight",
        Role::Infantry,
        (85, 0, 40, 0),
        100,
        Melee(17),
        (10, 2),
        "0.7",
        "2.0",
        12,
        "castle",
        Age::Imperial,
        '%',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 4);
    uu(&mut v, o, None);

    let mut o = unit(
        "janissary",
        "Janissary",
        Role::Archer,
        (60, 0, 55, 0),
        35,
        Pierce(17),
        (1, 0),
        "0.96",
        "3.45",
        17,
        "castle",
        Age::Castle,
        '6',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Gunpowder);
    ranged(&mut o, "8", "10", "5.5", 50);
    o.min_range = f("0");
    uu(&mut v, o, Some("elite_janissary"));
    let mut o = unit(
        "elite_janissary",
        "Elite Janissary",
        Role::Archer,
        (60, 0, 55, 0),
        40,
        Pierce(22),
        (2, 0),
        "0.96",
        "3.45",
        17,
        "castle",
        Age::Imperial,
        '^',
    );
    cls(&mut o, C::Archer);
    cls(&mut o, C::Gunpowder);
    ranged(&mut o, "8", "10", "5.5", 50);
    uu(&mut v, o, None);

    let mut o = unit(
        "berserk",
        "Berserk",
        Role::Infantry,
        (65, 0, 25, 0),
        54,
        Melee(9),
        (0, 1),
        "1.05",
        "2.0",
        14,
        "castle",
        Age::Castle,
        '7',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 2);
    uu(&mut v, o, Some("elite_berserk"));
    let mut o = unit(
        "elite_berserk",
        "Elite Berserk",
        Role::Infantry,
        (65, 0, 25, 0),
        62,
        Melee(14),
        (2, 1),
        "1.05",
        "2.0",
        14,
        "castle",
        Age::Imperial,
        '&',
    );
    cls(&mut o, C::Infantry);
    bonus(&mut o, C::Eagle, 3);
    uu(&mut v, o, None);

    // The joke cheat unit.
    let mut o = unit(
        "cobra_car",
        "Cobra Car",
        Role::Siege,
        (0, 0, 0, 0),
        500,
        Pierce(200),
        (10, 10),
        "1.6",
        "1.0",
        1,
        "castle",
        Age::Dark,
        '8',
    );
    cls(&mut o, C::Siege);
    ranged(&mut o, "6", "10", "20", 100);
    o.trained_at = None;
    o.population = 0;
    v.push(o);

    // ---------------- Buildings ----------------
    let mut o = building(
        "town_center",
        "Town Center",
        Role::DropOff,
        (0, 275, 0, 100),
        2400,
        (3, 5),
        (4, 4),
        150,
        Age::Castle,
        'T',
    );
    o.is_town_center = true;
    o.pop_support = 5;
    o.garrison_capacity = 15;
    o.garrison_arrows = 1;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 5,
    });
    ranged(&mut o, "6", "8", "7", 100);
    o.drop_off = GatherKind::ALL.to_vec();
    o.los = f("8");
    o.reload = f("2");
    v.push(o);

    let mut o = building(
        "house",
        "House",
        Role::Housing,
        (0, 25, 0, 0),
        550,
        (0, 7),
        (2, 2),
        25,
        Age::Dark,
        'h',
    );
    o.pop_support = 5;
    v.push(o);

    let mut o = building(
        "mill",
        "Mill",
        Role::DropOff,
        (0, 100, 0, 0),
        600,
        (0, 7),
        (2, 2),
        35,
        Age::Dark,
        'm',
    );
    o.drop_off = vec![
        GatherKind::Berries,
        GatherKind::Sheep,
        GatherKind::Hunt,
        GatherKind::Farm,
    ];
    v.push(o);
    let mut o = building(
        "lumber_camp",
        "Lumber Camp",
        Role::DropOff,
        (0, 100, 0, 0),
        600,
        (0, 7),
        (2, 2),
        35,
        Age::Dark,
        'l',
    );
    o.drop_off = vec![GatherKind::Wood];
    v.push(o);
    let mut o = building(
        "mining_camp",
        "Mining Camp",
        Role::DropOff,
        (0, 100, 0, 0),
        600,
        (0, 7),
        (2, 2),
        35,
        Age::Dark,
        'n',
    );
    o.drop_off = vec![GatherKind::Gold, GatherKind::Stone];
    v.push(o);
    let mut o = building(
        "farm",
        "Farm",
        Role::Economic,
        (0, 60, 0, 0),
        480,
        (0, 0),
        (3, 3),
        15,
        Age::Dark,
        '=',
    );
    o.walkable = true;
    o.yields = Some(Yield {
        resource: Resource::Food,
        amount: 175,
        kind: GatherKind::Farm,
    });
    o.requires = vec!["mill".into()];
    o.los = f("1");
    v.push(o);

    let mut o = building(
        "dock",
        "Dock",
        Role::Military,
        (0, 150, 0, 0),
        1800,
        (0, 7),
        (3, 3),
        35,
        Age::Dark,
        'd',
    );
    o.is_dock = true;
    o.placement = Placement::Shore;
    o.drop_off = vec![GatherKind::Fish];
    v.push(o);

    v.push(building(
        "barracks",
        "Barracks",
        Role::Military,
        (0, 175, 0, 0),
        1200,
        (0, 7),
        (3, 3),
        50,
        Age::Dark,
        'B',
    ));
    let mut o = building(
        "archery_range",
        "Archery Range",
        Role::Military,
        (0, 175, 0, 0),
        1200,
        (0, 7),
        (3, 3),
        50,
        Age::Feudal,
        'A',
    );
    o.requires = vec!["barracks".into()];
    v.push(o);
    let mut o = building(
        "stable",
        "Stable",
        Role::Military,
        (0, 175, 0, 0),
        1200,
        (0, 7),
        (3, 3),
        50,
        Age::Feudal,
        'S',
    );
    o.requires = vec!["barracks".into()];
    v.push(o);
    v.push(building(
        "blacksmith",
        "Blacksmith",
        Role::Economic,
        (0, 150, 0, 0),
        2100,
        (0, 7),
        (3, 3),
        40,
        Age::Feudal,
        'k',
    ));
    let mut o = building(
        "market",
        "Market",
        Role::Economic,
        (0, 175, 0, 0),
        2100,
        (0, 7),
        (4, 4),
        60,
        Age::Feudal,
        'M',
    );
    o.is_market = true;
    o.requires = vec!["mill".into()];
    v.push(o);
    let mut o = building(
        "monastery",
        "Monastery",
        Role::Economic,
        (0, 175, 0, 0),
        2100,
        (0, 7),
        (3, 3),
        40,
        Age::Castle,
        'P',
    );
    o.is_monastery = true;
    o.garrison_capacity = 10;
    v.push(o);
    let mut o = building(
        "siege_workshop",
        "Siege Workshop",
        Role::Military,
        (0, 200, 0, 0),
        1500,
        (0, 7),
        (3, 3),
        40,
        Age::Castle,
        'W',
    );
    o.requires = vec!["blacksmith".into()];
    v.push(o);
    v.push(building(
        "university",
        "University",
        Role::Economic,
        (0, 200, 0, 0),
        2100,
        (0, 7),
        (3, 3),
        60,
        Age::Castle,
        'U',
    ));

    let mut o = building(
        "castle",
        "Castle",
        Role::Defensive,
        (0, 0, 0, 650),
        4800,
        (8, 11),
        (4, 4),
        200,
        Age::Castle,
        'C',
    );
    o.armours.retain(|a| a.class != C::StandardBuilding);
    cls(&mut o, C::Castle);
    o.pop_support = 20;
    o.garrison_capacity = 20;
    o.garrison_arrows = 1;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 11,
    });
    ranged(&mut o, "8", "11", "7", 100);
    o.reload = f("2");
    v.push(o);

    let mut o = building(
        "outpost",
        "Outpost",
        Role::Defensive,
        (0, 25, 0, 5),
        500,
        (0, 0),
        (1, 1),
        15,
        Age::Dark,
        'o',
    );
    o.los = f("10");
    v.push(o);
    let mut o = building(
        "watch_tower",
        "Watch Tower",
        Role::Defensive,
        (0, 25, 0, 125),
        1020,
        (1, 7),
        (1, 1),
        80,
        Age::Feudal,
        't',
    );
    o.garrison_capacity = 5;
    o.garrison_arrows = 1;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 5,
    });
    ranged(&mut o, "8", "10", "7", 100);
    o.min_range = f("1");
    o.reload = f("2");
    upgrade(&mut o, "guard_tower", "guard_tower");
    v.push(o);
    let mut o = building(
        "guard_tower",
        "Guard Tower",
        Role::Defensive,
        (0, 25, 0, 125),
        1500,
        (1, 8),
        (1, 1),
        80,
        Age::Castle,
        't',
    );
    o.garrison_capacity = 5;
    o.garrison_arrows = 1;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 6,
    });
    ranged(&mut o, "8", "10", "7", 100);
    o.min_range = f("1");
    o.reload = f("2");
    o.requires_tech = Some("guard_tower".into());
    upgrade(&mut o, "keep", "keep");
    v.push(o);
    let mut o = building(
        "keep",
        "Keep",
        Role::Defensive,
        (0, 25, 0, 125),
        2250,
        (2, 9),
        (1, 1),
        80,
        Age::Imperial,
        'K',
    );
    o.garrison_capacity = 5;
    o.garrison_arrows = 1;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 7,
    });
    ranged(&mut o, "8", "10", "7", 100);
    o.min_range = f("1");
    o.reload = f("2");
    o.requires_tech = Some("keep".into());
    v.push(o);
    let mut o = building(
        "bombard_tower",
        "Bombard Tower",
        Role::Defensive,
        (0, 0, 100, 125),
        2220,
        (2, 9),
        (1, 1),
        80,
        Age::Imperial,
        'b',
    );
    o.garrison_capacity = 5;
    o.attacks.push(Attack {
        class: C::Pierce,
        value: 120,
    });
    bonus(&mut o, C::Ship, 40);
    ranged(&mut o, "8", "10", "4", 100);
    o.min_range = f("1");
    o.reload = f("6");
    o.requires_tech = Some("bombard_tower".into());
    v.push(o);

    let mut o = building(
        "palisade_wall",
        "Palisade Wall",
        Role::Wall,
        (0, 2, 0, 0),
        250,
        (0, 2),
        (1, 1),
        5,
        Age::Dark,
        '|',
    );
    o.armours.retain(|a| a.class != C::StandardBuilding);
    cls(&mut o, C::Wall);
    o.los = f("2");
    v.push(o);
    let mut o = building(
        "stone_wall",
        "Stone Wall",
        Role::Wall,
        (0, 0, 0, 5),
        900,
        (8, 10),
        (1, 1),
        10,
        Age::Feudal,
        '#',
    );
    o.armours.retain(|a| a.class != C::StandardBuilding);
    cls(&mut o, C::Wall);
    o.los = f("2");
    upgrade(&mut o, "fortified_wall", "fortified_wall");
    v.push(o);
    let mut o = building(
        "fortified_wall",
        "Fortified Wall",
        Role::Wall,
        (0, 0, 0, 5),
        3000,
        (12, 12),
        (1, 1),
        10,
        Age::Castle,
        '#',
    );
    o.armours.retain(|a| a.class != C::StandardBuilding);
    cls(&mut o, C::Wall);
    o.los = f("2");
    o.requires_tech = Some("fortified_wall".into());
    v.push(o);
    let mut o = building(
        "gate",
        "Gate",
        Role::Gate,
        (0, 0, 0, 30),
        1375,
        (6, 6),
        (1, 1),
        20,
        Age::Feudal,
        '/',
    );
    o.armours.retain(|a| a.class != C::StandardBuilding);
    cls(&mut o, C::Wall);
    o.walkable = true;
    o.los = f("3");
    v.push(o);

    let mut o = building(
        "wonder",
        "Wonder",
        Role::Wonder,
        (0, 1000, 1000, 1000),
        4800,
        (5, 10),
        (5, 5),
        2000,
        Age::Imperial,
        '*',
    );
    o.is_wonder = true;
    o.armours.retain(|a| a.class != C::StandardBuilding);
    v.push(o);

    // ---------------- Gaia: resources and animals ----------------
    let mut o = resource("tree", "Tree", GatherKind::Wood, 100, 'T');
    cls(&mut o, C::Tree);
    o.glyph = '♣';
    v.push(o);
    let mut o = resource("pine", "Pine Tree", GatherKind::Wood, 100, '♠');
    cls(&mut o, C::Tree);
    v.push(o);
    let mut o = resource("gold_mine", "Gold Mine", GatherKind::Gold, 800, '$');
    o.art = 1;
    v.push(o);
    let mut o = resource("stone_mine", "Stone Mine", GatherKind::Stone, 350, '◆');
    o.art = 2;
    v.push(o);
    let mut o = resource("berry_bush", "Forage Bush", GatherKind::Berries, 125, '%');
    o.walkable = false;
    v.push(o);
    let mut o = resource("fish", "Fish", GatherKind::Fish, 350, '~');
    o.placement = Placement::Water;
    v.push(o);
    let mut o = resource("shore_fish", "Shore Fish", GatherKind::Fish, 200, '≈');
    o.placement = Placement::Water;
    v.push(o);
    let mut o = base("relic", "Relic", ObjectClass::Relic, Role::None);
    o.hp = 1;
    o.glyph = '†';
    o.placement = Placement::Any;
    v.push(o);

    fn animal(
        key: &str,
        name: &str,
        hp: i32,
        atk: i32,
        food: i32,
        kind: GatherKind,
        speed: &str,
        glyph: char,
    ) -> ObjectType {
        let mut o = base(key, name, ObjectClass::Animal, Role::None);
        o.hp = hp;
        if atk > 0 {
            o.attacks.push(Attack {
                class: ArmourClass::Melee,
                value: atk,
            });
        }
        o.armours.push(Armour {
            class: ArmourClass::Melee,
            value: 0,
        });
        o.armours.push(Armour {
            class: ArmourClass::Pierce,
            value: 0,
        });
        o.yields = Some(Yield {
            resource: Resource::Food,
            amount: food,
            kind,
        });
        o.speed = f(speed);
        o.los = f("2");
        o.reload = f("2");
        o.glyph = glyph;
        o.placement = Placement::Any;
        o
    }
    v.push(animal(
        "sheep",
        "Sheep",
        7,
        0,
        100,
        GatherKind::Sheep,
        "0.8",
        'ʂ',
    ));
    v.push(animal(
        "turkey",
        "Turkey",
        7,
        0,
        100,
        GatherKind::Sheep,
        "0.8",
        'ʈ',
    ));
    v.push(animal(
        "deer",
        "Deer",
        5,
        0,
        140,
        GatherKind::Hunt,
        "1.2",
        'ɗ',
    ));
    v.push(animal(
        "boar",
        "Wild Boar",
        75,
        7,
        340,
        GatherKind::Hunt,
        "0.9",
        'ɓ',
    ));
    v.push(animal(
        "wolf",
        "Wolf",
        25,
        3,
        0,
        GatherKind::Hunt,
        "1.2",
        'ɯ',
    ));
    v
}
