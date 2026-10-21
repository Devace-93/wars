//! Selection and command building shared by every front-end.

use wars_sim::command::{Command, RallyTarget};
use wars_sim::ids::*;
use wars_sim::types::{GameData, ObjectClass};
use wars_sim::world::World;
use wars_sim::{Fixed, Vec2};

#[derive(Clone, Debug, Default)]
pub struct Controller {
    pub selection: Vec<EntityId>,
    pub groups: [Vec<EntityId>; 10],
    /// Building type being placed (after pressing a build button).
    pub placing: Option<TypeId>,
    pub placing_wall_start: Option<(i32, i32)>,
    pub shift: bool,
}

/// What a context ("right-click") action on a target turns into for the selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContextAction {
    Move,
    Attack,
    Gather,
    Build,
    Repair,
    Garrison,
    Convert,
    Trade,
    Follow,
    PickRelic,
    None,
}

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Drop dead or converted entities from selection and groups.
    pub fn prune(&mut self, world: &World, me: PlayerId) {
        let keep = |id: &EntityId| {
            world
                .get(*id)
                .map(|e| e.owner == me && e.inside.is_none())
                .unwrap_or(false)
        };
        self.selection.retain(keep);
        for g in self.groups.iter_mut() {
            g.retain(keep);
        }
    }

    pub fn select(&mut self, ids: Vec<EntityId>, add: bool) {
        if add {
            for id in ids {
                if !self.selection.contains(&id) {
                    self.selection.push(id);
                }
            }
        } else {
            self.selection = ids;
        }
        self.selection.truncate(60);
    }

    pub fn set_group(&mut self, n: usize) {
        self.groups[n % 10] = self.selection.clone();
    }
    pub fn recall_group(&mut self, n: usize) {
        self.selection = self.groups[n % 10].clone();
    }

    /// Units in the selection (not buildings), for movement commands.
    pub fn selected_units(&self, world: &World, data: &GameData) -> Vec<EntityId> {
        self.selection
            .iter()
            .copied()
            .filter(|id| {
                world
                    .get(*id)
                    .map(|e| data.object(e.ty).class == ObjectClass::Unit)
                    .unwrap_or(false)
            })
            .collect()
    }
    pub fn selected_buildings(&self, world: &World, data: &GameData) -> Vec<EntityId> {
        self.selection
            .iter()
            .copied()
            .filter(|id| {
                world
                    .get(*id)
                    .map(|e| data.object(e.ty).class == ObjectClass::Building)
                    .unwrap_or(false)
            })
            .collect()
    }
    pub fn first<'w>(&self, world: &'w World) -> Option<&'w wars_sim::world::Entity> {
        self.selection.first().and_then(|id| world.get(*id))
    }

    /// Decide what a context action on `target` means for the current selection.
    pub fn context_action(
        &self,
        world: &World,
        data: &GameData,
        me: PlayerId,
        target: Option<EntityId>,
    ) -> ContextAction {
        let units = self.selected_units(world, data);
        if units.is_empty() {
            return ContextAction::None;
        }
        let Some(tid) = target else {
            return ContextAction::Move;
        };
        let Some(t) = world.get(tid) else {
            return ContextAction::Move;
        };
        let tdef = data.object(t.ty);
        let any_villager = units
            .iter()
            .any(|u| data.object(world.get(*u).unwrap().ty).is_villager);
        let any_monk = units
            .iter()
            .any(|u| data.object(world.get(*u).unwrap().ty).can_convert);
        let any_trade = units
            .iter()
            .any(|u| data.object(world.get(*u).unwrap().ty).role == wars_sim::types::Role::Trade);
        let any_fisher = units
            .iter()
            .any(|u| data.object(world.get(*u).unwrap().ty).role == wars_sim::types::Role::Fishing);
        if world.ally(me, t.owner) && !t.owner.is_gaia() {
            if tdef.class == ObjectClass::Building {
                if t.build_progress < 1000 && any_villager {
                    return ContextAction::Build;
                }
                if tdef.yields.is_some() && any_villager {
                    return ContextAction::Gather;
                }
                if any_trade && (tdef.is_market || tdef.is_dock) {
                    return ContextAction::Trade;
                }
                let st = wars_sim::stats::stats_for(world, data, t.owner, t.ty);
                if t.hp < st.hp && any_villager {
                    return ContextAction::Repair;
                }
                if st.garrison_capacity > 0 || tdef.carries_units > 0 {
                    return ContextAction::Garrison;
                }
                return ContextAction::Move;
            }
            if tdef.carries_units > 0 {
                return ContextAction::Garrison;
            }
            return ContextAction::Follow;
        }
        if t.owner.is_gaia() {
            match tdef.class {
                ObjectClass::Resource | ObjectClass::Animal if tdef.yields.is_some() => {
                    let fishy = tdef
                        .yields
                        .map(|y| y.kind == wars_sim::types::GatherKind::Fish)
                        .unwrap_or(false);
                    if (fishy && any_fisher) || (!fishy && any_villager) {
                        return ContextAction::Gather;
                    }
                    if tdef.class == ObjectClass::Animal {
                        return ContextAction::Attack;
                    }
                    return ContextAction::Move;
                }
                ObjectClass::Relic if any_monk => return ContextAction::PickRelic,
                ObjectClass::Animal => return ContextAction::Attack,
                _ => return ContextAction::Move,
            }
        }
        if any_monk
            && !units
                .iter()
                .any(|u| !data.object(world.get(*u).unwrap().ty).attacks.is_empty())
        {
            return ContextAction::Convert;
        }
        ContextAction::Attack
    }

    /// Build the command for a context action at a point / on a target.
    pub fn context_command(
        &self,
        world: &World,
        data: &GameData,
        me: PlayerId,
        at: Vec2,
        target: Option<EntityId>,
    ) -> Option<Command> {
        let units = self.selected_units(world, data);
        let queue = self.shift;
        if units.is_empty() {
            // Buildings: set rally point.
            let b = self.selected_buildings(world, data);
            if !b.is_empty() {
                let to = match target {
                    Some(t) => RallyTarget::Entity(t),
                    None => RallyTarget::Point(at),
                };
                return Some(Command::SetRally { buildings: b, to });
            }
            return None;
        }
        match self.context_action(world, data, me, target) {
            ContextAction::Move => Some(Command::Move {
                units,
                to: at,
                queue,
            }),
            ContextAction::Attack => Some(Command::Attack {
                units,
                target: target?,
                queue,
            }),
            ContextAction::Gather => Some(Command::Gather {
                units,
                target: target?,
                queue,
            }),
            ContextAction::Build | ContextAction::Repair => Some(Command::Repair {
                units,
                target: target?,
                queue,
            }),
            ContextAction::Garrison => Some(Command::Garrison {
                units,
                target: target?,
            }),
            ContextAction::Convert => Some(Command::Convert {
                units,
                target: target?,
            }),
            ContextAction::Trade => Some(Command::Trade {
                units,
                target: target?,
            }),
            ContextAction::Follow => Some(Command::Follow {
                units,
                target: target?,
            }),
            ContextAction::PickRelic => {
                // Monks walk to the relic; the sim picks it up on arrival via a Gather-like task.
                let monks: Vec<EntityId> = units
                    .into_iter()
                    .filter(|u| data.object(world.get(*u).unwrap().ty).can_convert)
                    .collect();
                Some(Command::Attack {
                    units: monks,
                    target: target?,
                    queue,
                })
            }
            ContextAction::None => None,
        }
    }

    /// Entities under a point (units first, then buildings), nearest first.
    pub fn pick(world: &World, data: &GameData, at: Vec2, viewer: PlayerId) -> Option<EntityId> {
        let mut best: Option<(i32, Fixed, EntityId)> = None;
        for e in world.visible_entities(viewer) {
            let def = data.object(e.ty);
            let (prio, d) = if matches!(def.class, ObjectClass::Unit | ObjectClass::Animal) {
                let d = e.pos.dist(at);
                if d > Fixed::from_decimal("0.7") {
                    continue;
                }
                (0, d)
            } else {
                let (x, y, w, h) = world.footprint(data, e);
                let (ax, ay) = at.tile();
                if ax < x || ay < y || ax >= x + w || ay >= y + h {
                    continue;
                }
                (1, e.pos.dist(at))
            };
            if best.map(|(bp, bd, _)| (prio, d) < (bp, bd)).unwrap_or(true) {
                best = Some((prio, d, e.id));
            }
        }
        best.map(|(_, _, id)| id)
    }

    /// Own units inside a tile rectangle (box selection).
    pub fn box_select(
        world: &World,
        data: &GameData,
        me: PlayerId,
        a: Vec2,
        b: Vec2,
    ) -> Vec<EntityId> {
        let (x1, x2) = (a.x.min(b.x), a.x.max(b.x));
        let (y1, y2) = (a.y.min(b.y), a.y.max(b.y));
        let mut v: Vec<EntityId> = world
            .entities_of(me)
            .filter(|e| data.object(e.ty).class == ObjectClass::Unit && e.inside.is_none())
            .filter(|e| e.pos.x >= x1 && e.pos.x <= x2 && e.pos.y >= y1 && e.pos.y <= y2)
            .map(|e| e.id)
            .collect();
        // Prefer military over villagers when both are in the box.
        let military: Vec<EntityId> = v
            .iter()
            .copied()
            .filter(|id| !data.object(world.get(*id).unwrap().ty).is_villager)
            .collect();
        if !military.is_empty() && military.len() < v.len() {
            v = military;
        }
        v
    }

    /// Idle villagers of a player, in id order.
    pub fn idle_villagers(world: &World, data: &GameData, me: PlayerId) -> Vec<EntityId> {
        world
            .entities_of(me)
            .filter(|e| data.object(e.ty).is_villager && e.is_idle() && e.inside.is_none())
            .map(|e| e.id)
            .collect()
    }
}
