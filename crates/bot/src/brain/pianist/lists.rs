//! The player's lists (H-HANDS-SCRIPT, the `queue` tool; and the counted entries of a factory's `produce` list, made
//! in order, `play_sequences`): a builder with a list from the player is not in the pass;
//! the bot orders the next step when the builder is free, or when the build in progress is 60% done (queued behind
//! it), and skips a step it cannot do. A new list replaces the old one whole: the build in progress finishes and
//! what the old list queued behind it is dropped (`Pianist::list_replaced`, `take_queued`). An enemy on the builder puts it back in the pass until that is over, and a
//! builder the pass has sent away from an enemy takes no step while it is still in the pass for one
//! (`Pianist::list_is_held`). A bypass, not a decider: nothing in the pass reads a list
//! (`docs/design/2026-09-26-one-pass.md` §6).

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId};
use serde_json::json;

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Picture, clock};
use super::plan::Response;
use super::{Pianist, Task};



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
        under_fire.extend(pianist.under_fire(frame));
        let mut steps: Vec<(UnitId, Response, String, bool)> = Vec::new();
        pianist.list_held.retain(|id| own.iter().any(|u| u.id == *id));
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            if !pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) {
                continue;
            }
            let status = self.builder_status(&pianist, unit, picture, &under_fire, own);
            // A list plays under threat too: it is the player's order (onepass-player-2, 12:23: the commander's list
            // to spot_68 away from the block never started, the block being within 800). Not after the pick has
            // sent the builder away from an enemy, while that enemy is still on it: the pick's way out stands
            // (player-31: 83 of 92 such retreats were undone by the list within three seconds).
            if pianist.list_is_held(unit.id, status.threatened) {
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
            match self.execute_builder(tick, kit, picture, id, &response, queue, Some(&step), commands) {
                Some(did) => {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.done.push(format!("{} {name}: {did} (from its list)", clock(frame)));
                    pianist.played.push(json!({ "actor": name, "kind": "builder", "played": step, "did": did, "source": "list" }));
                    self.journal.note_from("list", frame, "builder", json!({ "actor": name, "step": step }), json!({ "did": did }));
                }
                // Never lost silently (13.3: 39 of 139 lists lost extractor steps): the same build under way is
                // the step done; anything else is said.
                None => {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    let same = matches!(pianist.tasks.get(&id), Some(Task::Build { .. }));
                    let text = if same { format!("the step '{step}' of its list is the build already under way: taken as done") } else { format!("the step '{step}' of its list could not be played this second and was dropped") };
                    pianist.done.push(format!("{} {name}: {text}", clock(frame)));
                    pianist.note(frame, format!("{name}: {text}"));
                }
            }
        }
    }

    /// The unit a factory's `produce` list makes next as a sequence (`sequence_next`), if a counted entry is left.
    pub(super) fn sequence_unit(&self, factory: &OwnUnit, name: &str, pianist: &Pianist) -> Option<UnitDefId> {
        let allowed = self.allowed_units(name).filter(|a| !a.units.is_empty())?;
        let options = &self.world.def(factory.def)?.build_options;
        let can_build = |unit: &str| self.world.def_named(unit).is_some_and(|d| options.contains(&d));
        let made = |unit: &str| pianist.produced.get(&(factory.id, unit.to_string())).copied().unwrap_or(0);
        let k = super::sequence_next(&allowed.units, can_build, made)?;
        self.world.def_named(super::allowance(&allowed.units[k]).0)
    }

    /// Every factory with a counted entry left on its `produce` list: the next unit ordered, one ahead of the pad,
    /// without asking. A bypass like the builders' lists: a factory in its sequence has no slot in the pass.
    pub(super) fn play_sequences(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let Some(pianist) = self.pianist.as_ref() else { return };
        let next: Vec<(UnitId, UnitDefId)> = own
            .iter()
            .filter(|u| !u.being_built && self.world.is_factory_def(u.def) && !pianist.lab_queue.get(&u.id).is_some_and(|q| !q.is_empty()))
            .filter_map(|u| self.sequence_unit(u, &self.actor_name(u.id), pianist).map(|def| (u.id, def)))
            .collect();
        for (id, def) in next {
            let name = self.actor_name(id);
            if let Some(did) = self.execute_lab(tick, id, &Response::Next(def), commands) {
                let unit = self.name(def).to_string();
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.done.push(format!("{} {name}: {did} (from its produce list)", clock(tick.frame)));
                pianist.played.push(json!({ "actor": name, "kind": "lab", "played": unit, "did": did, "source": "list" }));
                self.journal.note_from("list", tick.frame, "lab", json!({ "actor": name, "step": unit }), json!({ "did": did }));
            }
        }
    }

    /// Why a spot is not free to this builder, for the list's skip line (9.3).
    fn why_not_free(&self, i: usize, unit: &OwnUnit, pianist: &Pianist, own: &[OwnUnit], kit: &Kit) -> String {
        let Some(spot) = self.world.hello.metal_spots.get(i).copied() else { return format!("spot_{i} is not a spot") };
        let radius = self.spot_occupied_radius();
        if own.iter().any(|u| kit.is_extractor(u.def) && u.pos.dist2d(spot) < radius) {
            return format!("spot_{i} holds our extractor already");
        }
        if self.enemy_buildings.values().any(|(def, pos, _)| pos.dist2d(spot) < radius && self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)) {
            return format!("spot_{i} holds his extractor: it is taken by killing that, not by a list");
        }
        if let Some((id, _)) = pianist.tasks.iter().chain(pianist.queued.iter()).find(|(id, t)| **id != unit.id && matches!(t, Task::Build { spot: Some(s), .. } if *s == i)) {
            return format!("spot_{i} is being taken by {}", self.actor_name(*id));
        }
        if self.team_mates.spot_claims.contains(&i) {
            return format!("spot_{i} is being taken by a builder of another seat of ours");
        }
        if !self.reachable_for(self.walker_of(unit.def), spot) {
            return format!("spot_{i} is off this builder's ground (water or a cliff): an amphibious or hover constructor or a construction ship takes it");
        }
        if pianist.refused_spots.get(&i).is_some() {
            return format!("the engine refused a build at spot_{i} lately (wrecks or a unit on it)");
        }
        format!("spot_{i} is not free")
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
            // `extractor nearest` is the bare `extractor`: the free spot this builder reaches soonest, whatever the
            // start (the opening list is written before the start is known).
            let place = words.next().filter(|p| !(kind == "extractor" && *p == "nearest")).map(str::to_string);
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
                            // Why it is not free (9.3): "not free" covered a spot nothing of ours reached, one the
                            // enemy held, one under wrecks and one another builder was taking (Cape Violet, 333 skips).
                            Some(i) => Err(self.why_not_free(i, unit, pianist, own, kit)),
                            None => Err(format!("{p} is not a spot")),
                        },
                        None => match spots.first() {
                            Some((i, _)) => Ok(Response::Extractor(*i)),
                            None => Err("no free spot in reach".to_string()),
                        },
                    }
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

#[cfg(test)]
mod tests {
    use bot_protocol::UnitId;

    /// A list waits only for a builder the pick sent away from an enemy, and only while an enemy is still on it:
    /// once the builder is clear the hold is over for good, whatever comes near it later.
    #[test]
    fn a_list_is_held_after_the_picks_way_out_while_the_builder_is_threatened() {
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let builder = UnitId(1);
        assert!(!pianist.list_is_held(builder, true));
        pianist.list_held.insert(builder);
        assert!(pianist.list_is_held(builder, true));
        assert!(!pianist.list_is_held(builder, false));
        assert!(!pianist.list_is_held(builder, true));
    }

    /// A new list drops what the old one had queued behind the build in progress, when that build is finished: the
    /// builder is stopped and free for the new list's step, and it is said. Without a new list the queued task
    /// becomes the task as before.
    #[test]
    fn a_step_the_old_list_queued_is_dropped_for_the_new_list() {
        use super::super::Task;
        use bot_protocol::{Command, UnitDefId, Vec3};
        let build = |started| Task::Build { def: UnitDefId(7), near: Vec3::default(), spot: None, ordered: 100, started };
        let queued = |pianist: &mut super::Pianist, builder| {
            pianist.tasks.insert(builder, build(true));
            pianist.queued.insert(builder, build(false));
            pianist.queued_steps.insert(builder, "extractor".to_string());
            pianist.list_steps.insert(builder, ("extractor".to_string(), 100));
        };
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let (kept, replaced, cancelled, unlisted) = (UnitId(1), UnitId(2), UnitId(3), UnitId(4));
        for builder in [kept, replaced, cancelled, unlisted] {
            queued(&mut pianist, builder);
        }
        // The pass's own queued build, on a builder without a list: a cancel of no list leaves it.
        pianist.queued_steps.remove(&unlisted);
        pianist.list_replaced(replaced, true);
        pianist.list_replaced(cancelled, false);
        pianist.list_replaced(unlisted, false);
        assert!(!pianist.list_steps.contains_key(&replaced), "the old list's step no longer returns to a list");
        assert!(pianist.take_queued(kept, "commander", 200).is_none());
        assert!(matches!(pianist.tasks.get(&kept), Some(Task::Build { started: false, ordered: 200, .. })));
        assert_eq!(pianist.list_steps.get(&kept), Some(&("extractor".to_string(), 200)));
        assert!(pianist.take_queued(unlisted, "constructor_4", 200).is_none());
        assert!(matches!(pianist.tasks.get(&unlisted), Some(Task::Build { started: false, .. })));
        for builder in [replaced, cancelled] {
            assert!(matches!(pianist.take_queued(builder, "commander", 200), Some(Command::Stop { unit }) if unit == builder));
            assert!(!pianist.tasks.contains_key(&builder) && !pianist.queued.contains_key(&builder) && !pianist.queued_steps.contains_key(&builder));
        }
        assert!(pianist.done.last().unwrap().contains("commander: dropped the step 'extractor' its old list had queued"), "{:?}", pianist.done);
    }

    /// A unit is under fire for three seconds after its last hit, not for the rest of the game.
    #[test]
    fn a_hit_is_forgotten_after_three_seconds() {
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.hits.insert(UnitId(1), 100);
        assert_eq!(pianist.under_fire(160).collect::<Vec<_>>(), vec![UnitId(1)]);
        assert_eq!(pianist.under_fire(100 + super::super::UNDER_FIRE_FRAMES + 1).count(), 0);
    }
}
