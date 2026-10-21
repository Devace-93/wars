//! Available actions for the current selection: the signature feature of the console
//! client. Everything here is a pure function of the world so it can be unit-tested.

use wars_client::wars_sim::ids::{Age, EntityId, PlayerId, Stockpile, TechId, TypeId};
use wars_client::wars_sim::stats;
use wars_client::wars_sim::types::{GameData, ObjectClass};
use wars_client::wars_sim::world::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// Open the build menu (villagers).
    BuildMenu,
    /// Pick a building in the build menu and start placing it.
    Place(TypeId),
    Gather,
    Repair,
    Move,
    AttackMove,
    Stop,
    Garrison,
    Delete,
    /// Context action on the cursor target (`Controller::context_command`).
    Context,
    TogglePack,
    Unload,
    Train(TypeId),
    Research(TechId),
    Ungarrison,
    Rally,
    CancelQueue,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub key: char,
    pub label: String,
    pub cost: Option<Stockpile>,
    pub enabled: bool,
    /// Why it is disabled (age, missing building) or empty.
    pub note: String,
    pub kind: ActionKind,
}

impl Action {
    fn fixed(key: char, label: &str, kind: ActionKind) -> Action {
        Action {
            key,
            label: label.to_string(),
            cost: None,
            enabled: true,
            note: String::new(),
            kind,
        }
    }
}

/// Keys that keep their global meaning on the game screen and are never handed to a
/// train/research entry: cursor movement, quit, pause, groups, context, delete.
pub const GLOBAL_KEYS: &[char] = &['h', 'j', 'k', 'l', 'q', 'p', 'g', 'x', 'd'];
/// Extra keys a selected building uses for its own fixed verbs.
const BUILDING_KEYS: &[char] = &['e', 'c'];

/// Pick a shortcut letter for `name`: the preferred glyph first, then the letters of the
/// name in order, then a-z and A-Z, skipping reserved and already taken keys.
pub fn assign_key(
    preferred: Option<char>,
    name: &str,
    taken: &mut Vec<char>,
    reserved: &[char],
) -> char {
    let lower = name.to_lowercase();
    let candidates = preferred
        .into_iter()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .chain(lower.chars().filter(|c| c.is_ascii_lowercase()))
        .chain('a'..='z')
        .chain('A'..='Z');
    for c in candidates {
        if !reserved.contains(&c) && !taken.contains(&c) {
            taken.push(c);
            return c;
        }
    }
    '?'
}

/// `50f 30w` style cost string (zero resources omitted).
pub fn cost_str(c: &Stockpile) -> String {
    let mut parts = Vec::new();
    if c.food > 0 {
        parts.push(format!("{}f", c.food));
    }
    if c.wood > 0 {
        parts.push(format!("{}w", c.wood));
    }
    if c.gold > 0 {
        parts.push(format!("{}g", c.gold));
    }
    if c.stone > 0 {
        parts.push(format!("{}s", c.stone));
    }
    parts.join(" ")
}

/// Actions for a selection owned by `me`, in display order.
pub fn actions_for(
    world: &World,
    data: &GameData,
    me: PlayerId,
    selection: &[EntityId],
) -> Vec<Action> {
    let ents: Vec<_> = selection
        .iter()
        .filter_map(|id| world.get(*id))
        .filter(|e| e.owner == me)
        .collect();
    let units: Vec<_> = ents
        .iter()
        .filter(|e| data.object(e.ty).class == ObjectClass::Unit)
        .collect();
    let mut out = Vec::new();
    if !units.is_empty() {
        let any_villager = units.iter().any(|e| data.object(e.ty).is_villager);
        if any_villager {
            out.push(Action::fixed('b', "build", ActionKind::BuildMenu));
            out.push(Action::fixed('g', "gather", ActionKind::Gather));
            out.push(Action::fixed('r', "repair", ActionKind::Repair));
        }
        out.push(Action::fixed('m', "move", ActionKind::Move));
        out.push(Action::fixed('a', "attack-move", ActionKind::AttackMove));
        out.push(Action::fixed('s', "stop", ActionKind::Stop));
        out.push(Action::fixed('i', "garrison", ActionKind::Garrison));
        if units.iter().any(|e| data.object(e.ty).key == "trebuchet") {
            out.push(Action::fixed('t', "pack/unpack", ActionKind::TogglePack));
        }
        if units
            .iter()
            .any(|e| data.object(e.ty).carries_units > 0 || e.relics > 0)
        {
            out.push(Action::fixed('u', "unload", ActionKind::Unload));
        }
        out.push(Action::fixed('x', "context", ActionKind::Context));
        out.push(Action::fixed('d', "delete", ActionKind::Delete));
        return out;
    }
    let Some(b) = ents
        .iter()
        .find(|e| e.build_progress >= 1000)
        .or(ents.first())
    else {
        return out;
    };
    if b.build_progress < 1000 {
        out.push(Action::fixed('d', "cancel foundation", ActionKind::Delete));
        return out;
    }
    let mut reserved: Vec<char> = GLOBAL_KEYS.to_vec();
    reserved.extend_from_slice(BUILDING_KEYS);
    let mut taken = Vec::new();
    let stock = world.player(me).resources;
    if let Some(list) = data.trains_at.get(&b.ty) {
        for ty in list {
            if !stats::is_available(world, data, me, *ty) {
                continue;
            }
            let def = data.object(*ty);
            let st = stats::stats_for(world, data, me, *ty);
            let key = assign_key(Some(def.glyph), &def.name, &mut taken, &reserved);
            out.push(Action {
                key,
                label: format!("train {}", def.name),
                cost: Some(st.cost),
                enabled: stock.covers(&st.cost) && b.queue.len() < 15,
                note: String::new(),
                kind: ActionKind::Train(*ty),
            });
        }
    }
    if let Some(list) = data.techs_at.get(&b.ty) {
        for t in list {
            if !stats::tech_available(world, data, me, *t) {
                continue;
            }
            let def = data.tech(*t);
            let (cost, _) = stats::tech_cost(world, data, me, *t);
            let key = assign_key(None, &def.name, &mut taken, &reserved);
            out.push(Action {
                key,
                label: format!("research {}", def.name),
                cost: Some(cost),
                enabled: stock.covers(&cost) && b.queue.len() < 15,
                note: String::new(),
                kind: ActionKind::Research(*t),
            });
        }
    }
    if !b.garrisoned.is_empty() {
        out.push(Action::fixed('e', "eject garrison", ActionKind::Ungarrison));
    }
    if !b.queue.is_empty() {
        out.push(Action::fixed(
            'c',
            "cancel last queued",
            ActionKind::CancelQueue,
        ));
    }
    out.push(Action::fixed('x', "rally to cursor", ActionKind::Rally));
    out.push(Action::fixed('d', "delete", ActionKind::Delete));
    out
}

/// The build menu: every building villagers can construct, greyed when it is not yet
/// available (age, prerequisites) or not affordable.
pub fn build_menu(world: &World, data: &GameData, me: PlayerId) -> Vec<Action> {
    let p = world.player(me);
    let mut taken = Vec::new();
    let mut out = Vec::new();
    for ty in &data.buildable {
        let def = data.object(*ty);
        let st = stats::stats_for(world, data, me, *ty);
        let available = stats::is_available(world, data, me, *ty);
        let affordable = p.resources.covers(&st.cost);
        let note = if !available {
            if def.age > p.age {
                Age::from_idx(def.age.idx()).name().to_string()
            } else if let Some(r) = def
                .requires
                .iter()
                .find(|r| !world.has_building(data, me, r))
            {
                format!(
                    "needs {}",
                    data.object_id(r)
                        .map(|id| data.object(id).name.clone())
                        .unwrap_or_else(|| r.clone())
                )
            } else {
                "unavailable".to_string()
            }
        } else {
            String::new()
        };
        let key = assign_key(None, &def.name, &mut taken, &[]);
        out.push(Action {
            key,
            label: def.name.clone(),
            cost: Some(st.cost),
            enabled: available && affordable,
            note,
            kind: ActionKind::Place(*ty),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use wars_client::wars_mapgen::{generate, MapParams, MapSize};

    fn world() -> World {
        let data = wars_client::wars_data::data();
        let params = MapParams {
            size: MapSize::Tiny,
            seed: 3,
            ..Default::default()
        };
        generate(data, &params)
    }

    #[test]
    fn assign_key_skips_reserved_and_taken() {
        let mut taken = vec!['v'];
        let k = assign_key(Some('v'), "villager", &mut taken, &['i']);
        assert_eq!(k, 'l'); // v taken, i reserved, l is the next letter of the name
        let k2 = assign_key(None, "loom", &mut taken, GLOBAL_KEYS);
        assert_eq!(k2, 'o'); // l is a movement key
        assert_eq!(taken, vec!['v', 'l', 'o']);
    }

    #[test]
    fn villager_actions_have_build_gather_and_move() {
        let data = wars_client::wars_data::data();
        let w = world();
        let me = PlayerId(1);
        let villagers: Vec<EntityId> = w
            .entities_of(me)
            .filter(|e| data.object(e.ty).is_villager)
            .map(|e| e.id)
            .collect();
        assert!(!villagers.is_empty());
        let acts = actions_for(&w, data, me, &villagers);
        let keys: String = acts.iter().map(|a| a.key).collect();
        assert_eq!(keys, "bgrmasixd");
        assert!(acts.iter().all(|a| a.enabled));
    }

    #[test]
    fn town_center_trains_villagers_and_lists_techs() {
        let data = wars_client::wars_data::data();
        let mut w = world();
        let me = PlayerId(1);
        let tc = w
            .entities_of(me)
            .find(|e| data.object(e.ty).is_town_center)
            .map(|e| e.id)
            .unwrap();
        let acts = actions_for(&w, data, me, &[tc]);
        let v = acts
            .iter()
            .find(|a| a.kind == ActionKind::Train(data.obj("villager")))
            .expect("villager entry");
        assert_eq!(v.key, 'v');
        assert!(v.enabled, "200 food covers a villager");
        assert!(acts
            .iter()
            .any(|a| matches!(a.kind, ActionKind::Research(_))));
        // Keys are unique and never collide with the global ones.
        let mut keys: Vec<char> = acts.iter().map(|a| a.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n);
        assert!(acts
            .iter()
            .filter(|a| matches!(a.kind, ActionKind::Train(_) | ActionKind::Research(_)))
            .all(|a| !GLOBAL_KEYS.contains(&a.key)));
        // With no food the villager entry is greyed out.
        w.player_mut(me).resources.food = 0;
        let acts = actions_for(&w, data, me, &[tc]);
        let v = acts
            .iter()
            .find(|a| a.kind == ActionKind::Train(data.obj("villager")))
            .unwrap();
        assert!(!v.enabled);
    }

    #[test]
    fn build_menu_greys_later_age_buildings() {
        let data = wars_client::wars_data::data();
        let w = world();
        let menu = build_menu(&w, data, PlayerId(1));
        let house = menu.iter().find(|a| a.label == "House").unwrap();
        assert!(house.enabled);
        let castle = menu.iter().find(|a| a.label == "Castle").unwrap();
        assert!(!castle.enabled);
        assert_eq!(castle.note, "Castle Age");
        let mut keys: Vec<char> = menu.iter().map(|a| a.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n);
    }

    #[test]
    fn cost_string_skips_zero_resources() {
        assert_eq!(cost_str(&Stockpile::new(50, 0, 0, 0)), "50f");
        assert_eq!(cost_str(&Stockpile::new(0, 175, 0, 25)), "175w 25s");
    }
}
