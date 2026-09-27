//! The player removes what we own (docs/design/2026-09-22-yard-and-reclaim.md, decision 4): every ask the pianist
//! publishes a card per unit of ours, by the handle the player names it with, and carries out the `remove` tool's
//! orders: `reclaim` steps put at the front of a builder's list, self-destructs sent to the engine.

use std::collections::VecDeque;

use bot_protocol::{Command, OwnUnit, Tick};

use super::super::Brain;
use crate::strategist::shared::{Removal, Shared, UnitCard};

impl Brain {
    /// The cards the `remove` tool reads: one per finished unit of ours, and the blasts of every unit type once.
    pub(in crate::brain) fn publish_cards(&self, tick: &Tick, shared: &Shared) {
        let blast = |b: Option<bot_protocol::Blast>| b.map(|b| (b.radius, b.damage));
        let cards: Vec<UnitCard> = tick
            .snapshot
            .own_units
            .iter()
            .filter(|u| !u.being_built)
            .map(|u| {
                let def = self.world.def(u.def);
                let actor = (self.world.is_mobile_builder(u.def) || self.world.is_factory_def(u.def)).then(|| self.actor_name(u.id));
                UnitCard {
                    handle: self.handle(u),
                    actor,
                    unit: self.name(u.def).to_string(),
                    at: (u.pos.x, u.pos.z),
                    health: u.health,
                    metal: def.map_or(0.0, |d| d.metal_cost),
                    self_destruct: blast(def.and_then(|d| d.self_destruct_blast)),
                    self_destruct_seconds: def.map_or(0.0, |d| d.self_destruct_seconds),
                }
            })
            .collect();
        shared.own_cards.lock().unwrap().insert(self.world.hello.team, cards);
        let mut blasts = shared.blasts.lock().unwrap();
        if blasts.is_empty() {
            for d in &self.world.hello.unit_defs {
                blasts.insert(d.name.clone(), (blast(d.death_blast), blast(d.self_destruct_blast), d.self_destruct_seconds));
            }
        }
    }

    /// The `remove` tool's orders since the last ask.
    pub(in crate::brain) fn take_removals(&mut self, tick: &Tick, commands: &mut Vec<Command>, shared: &Shared) {
        let removals = std::mem::take(&mut *shared.removals.lock().unwrap());
        if removals.is_empty() {
            return;
        }
        let own = &tick.snapshot.own_units;
        // Another seat's units are left for that seat (the handles are unit ids, unique across the game).
        let mut others: Vec<Removal> = Vec::new();
        let removals: Vec<Removal> = removals
            .into_iter()
            .filter_map(|removal| {
                let targets = match &removal {
                    Removal::Destruct { targets } | Removal::Reclaim { targets, .. } => targets,
                };
                let theirs = !targets.is_empty() && targets.iter().all(|h| self.handle_owner(h, own) == super::super::HandleOwner::AnotherSeat);
                if theirs {
                    others.push(removal);
                    None
                } else {
                    Some(removal)
                }
            })
            .collect();
        if !others.is_empty() {
            shared.removals.lock().unwrap().extend(others);
        }
        let frame = tick.frame;
        let mut notes: Vec<String> = Vec::new();
        let mut steps: Vec<(String, String)> = Vec::new();
        for removal in removals {
            match removal {
                Removal::Destruct { targets } => {
                    for handle in targets {
                        match self.unit_by_handle(&handle, own) {
                            Some(u) => {
                                commands.push(Command::SelfDestruct { unit: u.id });
                                notes.push(format!("{handle} is self-destructing on the player's order"));
                            }
                            None => notes.push(format!("{handle} no longer stands; nothing to destruct")),
                        }
                    }
                }
                Removal::Reclaim { targets, by } => {
                    for handle in targets {
                        let Some(target) = self.unit_by_handle(&handle, own) else {
                            notes.push(format!("{handle} no longer stands; nothing to take apart"));
                            continue;
                        };
                        let builder = match &by {
                            Some(name) => own.iter().find(|u| self.world.is_mobile_builder(u.def) && self.actor_name(u.id) == *name),
                            None => self.free_builder_near(target, own),
                        };
                        let Some(builder) = builder else {
                            notes.push(format!("no builder of ours can take {handle} apart{}", by.as_ref().map_or(String::new(), |b| format!(" ({b} is not a builder standing now)"))));
                            continue;
                        };
                        steps.push((self.actor_name(builder.id), handle));
                    }
                }
            }
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (builder, handle) in steps {
            pianist.scripts.entry(builder.clone()).or_insert_with(VecDeque::new).push_front(format!("reclaim {handle}"));
            notes.push(format!("{builder} takes {handle} apart next, at the front of its list"));
        }
        for text in notes {
            pianist.done.push(format!("{} {text}", super::clock(frame)));
            pianist.note(frame, text);
        }
    }

    /// The builder to take `target` apart when the player names none: the nearest mobile builder without a list,
    /// else the nearest with one; never the target itself.
    fn free_builder_near<'a>(&self, target: &OwnUnit, own: &'a [OwnUnit]) -> Option<&'a OwnUnit> {
        let scripts = &self.pianist.as_ref().expect("pianist mode").scripts;
        let listed = |u: &OwnUnit| scripts.get(&self.actor_name(u.id)).is_some_and(|s| !s.is_empty());
        own.iter()
            .filter(|u| u.id != target.id && !u.being_built && self.world.is_mobile_builder(u.def))
            .min_by(|a, b| (listed(a), a.pos.dist2d(target.pos)).partial_cmp(&(listed(b), b.pos.dist2d(target.pos))).unwrap_or(std::cmp::Ordering::Equal))
    }
}
