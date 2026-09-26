//! The player's lists (H-HANDS-SCRIPT, the `queue` tool): a builder with a list from the player is not in the pass;
//! the bot orders the next step when the builder is free, or when the build in progress is 60% done (queued behind
//! it), and skips a step it cannot do. An enemy on the builder puts it back in the pass until that is over. A
//! bypass, not a decider: nothing in the pass reads a list (`docs/design/2026-09-26-one-pass.md` §6).

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId};
use serde_json::json;

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Picture, clock};
use super::plan::Response;
use super::{Pianist, Task};

/// A timed assist step is skipped while the metal store is under this (the pool's bank at 2:00 to 3:00 is 100 to 150).
const ASSIST_STORE_FLOOR: f32 = 20.0;

/// The seconds of a timed list step `assist N`; None for any other step.
pub(super) fn timed_assist(step: &str) -> Option<i32> {
    let mut words = step.split_whitespace();
    (words.next() == Some("assist")).then(|| words.next().and_then(|n| n.parse::<i32>().ok())).flatten()
}

impl Brain {
    /// Every builder with a list: its next step ordered when it is ready for one.
    pub(super) fn play_lists(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let frame = tick.frame;
        let Some(mut pianist) = self.pianist.take() else { return };
        let mut under_fire: Vec<UnitId> = tick.events.iter().filter_map(|e| if let bot_protocol::Event::UnitDamaged { unit, .. } = e { Some(*unit) } else { None }).collect();
        under_fire.extend(pianist.hits.keys().copied());
        let mut steps: Vec<(UnitId, Response, String, bool)> = Vec::new();
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            if !pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) {
                continue;
            }
            let status = self.builder_status(&pianist, unit, picture, &under_fire, own);
            // A list plays under threat too: it is the player's order (onepass-player-2, 12:23: the commander's list
            // to spot_68 away from the block never started, the block being within 800). Not in the hold after the
            // pass sent the builder home: the way out first.
            if pianist.retreated.get(&unit.id).is_some_and(|f| frame - f < super::plan::RETREAT_HOLD) {
                continue;
            }
            let task = pianist.tasks.get(&unit.id).cloned();
            // The player's list outranks the bot's fillers: a builder helping a factory, walking, reclaiming or
            // repairing takes its next step at once (plan-1: the commander helped a plant from 3:46 to 14:24 while
            // four lists waited, since a guard order never ends). A build in progress keeps the 60 % rule.
            let list_frame = pianist.script_frame.get(&name).copied().unwrap_or(0);
            let ready = match &task {
                None => unit.idle,
                Some(Task::Build { started: true, .. }) => status.queue_ahead,
                // A build the engine has not started yet gives way to a list set after it was ordered.
                Some(Task::Build { ordered, .. }) => status.queue_ahead || list_frame > *ordered,
                // A timed `assist N` step keeps the builder at the factory for its N seconds; `end_timed_assists`
                // frees it.
                Some(Task::Assist { since, .. }) => match pianist.list_steps.get(&unit.id).and_then(|(step, _)| timed_assist(step)) {
                    Some(seconds) => frame - since >= seconds * FRAMES_PER_SECOND,
                    None => true,
                },
                Some(_) => true,
            };
            if !ready {
                continue;
            }
            if let Some((response, step)) = self.next_list_step(unit, &name, &mut pianist, picture, own, frame, kit) {
                steps.push((unit.id, response, step, status.queue_ahead));
            }
        }
        self.pianist = Some(pianist);
        for (id, response, step, queue) in steps {
            let name = self.actor_name(id);
            if let Some(did) = self.execute_builder(tick, kit, picture, id, &response, queue, Some(&step), commands) {
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.done.push(format!("{} {name}: {did} (from its list)", clock(frame)));
                pianist.played.push(json!({ "actor": name, "kind": "builder", "played": step, "did": did, "source": "list" }));
                self.journal.note_from("list", frame, "builder", json!({ "actor": name, "step": step }), json!({ "did": did }));
            }
        }
    }

    /// The next step of a builder's list as a state, its words kept. Steps that cannot be done are dropped and
    /// said; `assist` before any factory exists waits.
    #[allow(clippy::too_many_arguments)]
    fn next_list_step(&self, unit: &OwnUnit, name: &str, pianist: &mut Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) -> Option<(Response, String)> {
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        loop {
            let step = pianist.scripts.get_mut(name)?.pop_front()?;
            let mut words = step.split_whitespace();
            let kind = words.next().unwrap_or_default();
            let place = words.next().map(str::to_string);
            let at_place = |def: UnitDefId| match &place {
                _ if !can(def) => Err("this builder cannot build it".to_string()),
                Some(p) if picture.places.iter().any(|q| q.name == *p) => Ok(Response::BuildingAt(def, p.clone())),
                Some(p) => Err(format!("{p} is not a place in the picture")),
                None => Err("it needs a place".to_string()),
            };
            let resolved: Result<Response, String> = match kind {
                "extractor" if !can(kit.extractor) => Err("this builder cannot build it".to_string()),
                "extractor" => {
                    let spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                    match &place {
                        Some(p) => match p.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                            Some(i) if spots.iter().any(|(j, _)| *j == i) => Ok(Response::Extractor(i)),
                            Some(_) => Err(format!("{p} is not free")),
                            None => Err(format!("{p} is not a spot")),
                        },
                        None => match spots.first() {
                            Some((i, _)) => Ok(Response::Extractor(*i)),
                            None => Err("no free spot in reach".to_string()),
                        },
                    }
                }
                // `assist` or `assist N`: the seconds, when given, end the step. A timed assist at an empty store is
                // skipped: helping adds build power, and an empty store has nothing for it to spend
                // (K-open-comet-our-plant-starves).
                "assist" if timed_assist(&step).is_some() && picture.metal_stored < ASSIST_STORE_FLOOR => {
                    Err(format!("the store holds {:.0} metal, and helping the plant adds nothing at an empty store; the list goes on", picture.metal_stored))
                }
                "assist" => match own.iter().filter(|u| self.world.is_factory_def(u.def)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))) {
                    Some(factory) => Ok(Response::Assist(factory.id)),
                    None => {
                        // Nothing to help yet: the step waits at the front of the list.
                        pianist.scripts.get_mut(name)?.push_front(step);
                        return None;
                    }
                },
                // `reclaim <handle>`: one unit of ours, taken apart for its metal (the `remove` tool).
                "reclaim" => match place.as_deref().and_then(|h| self.unit_by_handle(h, own)) {
                    Some(target) if target.id == unit.id => Err("a builder cannot take itself apart".to_string()),
                    Some(target) => Ok(Response::ReclaimUnit(target.id)),
                    None => Err(format!("{} no longer stands", place.as_deref().unwrap_or("it"))),
                },
                // A defence at a spot whose extractor the engine refused lately waits with its extractor.
                other if self.world.def_named(other).is_some_and(|def| self.placed_at_place(def) && self.world.def(def).is_some_and(|d| d.weapon_count > 0))
                    && place.as_deref().and_then(|p| p.strip_prefix("spot_")).and_then(|n| n.parse::<usize>().ok()).is_some_and(|i| pianist.refused_spots.get(&i).is_some_and(|until| *until > frame)) =>
                {
                    Err(format!("the extractor at {} was refused lately; a defence there waits for it", place.as_deref().unwrap_or("?")))
                }
                // Any unit by its internal name; one that stands at a place needs the place, and any other building
                // takes one when given.
                other => match self.world.def_named(other) {
                    Some(def) if self.placed_at_place(def) || place.is_some() => at_place(def),
                    Some(def) if can(def) => Ok(Response::Building(def)),
                    Some(_) => Err("this builder cannot build it".to_string()),
                    None => Err(format!("'{other}' is not a unit name the game knows")),
                },
            };
            match resolved {
                Ok(response) => return Some((response, step)),
                Err(why) => {
                    let text = format!("skipped the step '{step}' of its list: {why}");
                    pianist.done.push(format!("{} {name}: {text}", clock(frame)));
                    pianist.note(frame, format!("{name} {text}"));
                }
            }
        }
    }
}
