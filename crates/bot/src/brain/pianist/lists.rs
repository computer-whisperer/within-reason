//! The player's lists (H-HANDS-SCRIPT, the `queue` tool; and the counted entries of a factory's `produce` list, made
//! in order, `play_sequences`): a builder with a list from the player has no menu;
//! the bot orders the next step when the builder is free, or when the build in progress is 60% done (queued behind
//! it), and skips a step it cannot do. A new list replaces the old one whole: the build in progress finishes and
//! what the old list queued behind it is dropped (`Pianist::list_replaced`, `take_queued`). An enemy on the builder
//! gives it a menu of its ways out until that is over, and a builder the pick has sent away from an enemy takes no
//! step while the enemy is still on it (`Pianist::list_is_held`). A bypass, not a decider: nothing in the pass
//! reads a list beyond the spots its steps name (`docs/design/2026-09-26-one-pass.md` §6).

use std::collections::HashMap;

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};
use serde_json::json;

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Picture, clock};
use super::menu::{Order, Site};
use super::{Pianist, TagState, Tagged, Task};

/// A list step's words and its id: a last word beginning with `#` names the building the step is ("armavp
/// avp_yard #avp1"), so that steps carrying the same id on several builders' lists, or in a list sent again, are
/// one building (`Pianist::tag_state`).
pub(crate) fn step_id(step: &str) -> (&str, Option<&str>) {
    match step.trim_end().rsplit_once(char::is_whitespace) {
        Some((words, last)) if last.len() > 1 && last.starts_with('#') => (words.trim_end(), Some(&last[1..])),
        _ => (step, None),
    }
}

/// A list step's words without the word `low`, and whether it carried it: the builder draws only the team's surplus
/// while it is on the step (H-HANDS-LIST-LOW, the game's builder-priority flag sent as `Command::Priority`:
/// K-mech-builder-priority-is-a-builder-flag). "assist 90 low", "armsolar low #s1".
pub(crate) fn step_low(body: &str) -> (String, bool) {
    let low = body.split_whitespace().any(|w| w == "low");
    (body.split_whitespace().filter(|w| *w != "low").collect::<Vec<_>>().join(" "), low)
}

/// Whether a whole step (words, `low`, id) carries the word `low`.
pub(crate) fn step_is_low(step: &str) -> bool {
    step_low(step_id(step).0).1
}

/// The seconds of a timed list step `assist N`; None for any other step.
pub(super) fn timed_assist(step: &str) -> Option<i32> {
    let mut words = step.split_whitespace();
    (words.next() == Some("assist")).then(|| words.next().and_then(|n| n.parse::<i32>().ok())).flatten()
}

/// The steps of one tick that resolved to a build of one id, in the order they are played: the builder nearest
/// the site is moved to the front of its id's steps, so that it starts the building and the others, played after
/// it, help it (`same_tick_help`). A `queue` call lands its lists on one frame and a builder on a filler takes
/// its step at once, so several builders reach one id's step on the same tick; the id's claim is written only as
/// a step is played (`Pianist::note_tag`), so every one of them saw the id free and got its own build (human-33,
/// 12:41: two constructors began #avp1's advanced vehicle plant on one tick, `build_site_for`'s keep-out put the
/// second frame beside the first, and two 2,600-metal plants stood on an income of 30; 13 such double starts in
/// the game, the others merged by the engine at one site).
pub(super) fn nearest_first(steps: &mut [(UnitId, Order, String, bool)], site: impl Fn(&Order) -> Option<Vec3>, pos: impl Fn(UnitId) -> Option<Vec3>) {
    let mut first: HashMap<String, usize> = HashMap::new();
    for i in 0..steps.len() {
        let (Order::Build(..), Some(tag)) = (&steps[i].1, step_id(&steps[i].2).1) else { continue };
        let Some(lead) = first.get(tag).copied() else {
            first.insert(tag.to_string(), i);
            continue;
        };
        let nearer = match (site(&steps[i].1), pos(steps[i].0), pos(steps[lead].0)) {
            (Some(at), Some(this), Some(leading)) => this.dist2d(at) < leading.dist2d(at),
            _ => false,
        };
        if nearer {
            steps.swap(lead, i);
        }
    }
}

/// The order a step plays as among the steps of one tick: a build of an id that a build played earlier in the
/// tick claimed (`claimed`, written as each is played) is help for that builder.
pub(super) fn same_tick_help(order: Order, tag: Option<&str>, claimed: &HashMap<String, UnitId>) -> Order {
    match (&order, tag.and_then(|t| claimed.get(t))) {
        (Order::Build(..), Some(claimant)) => Order::Help(*claimant),
        _ => order,
    }
}

impl Pianist {
    /// A step with an id has been ordered for `builder`: a build claims the id until the engine makes the frame; a
    /// help order (the id's frame, or the builder claiming it) makes the builder a helper of the id, and a frame
    /// being helped is the id's building. The order's words, with the id when it is help.
    fn note_tag(&mut self, tag: &str, builder: UnitId, own: &[OwnUnit], did: String, queue: bool, frame_now: i32) -> String {
        let task = if queue { self.queued.get(&builder) } else { self.tasks.get(&builder) }.cloned();
        match task {
            Some(Task::Build { def, .. }) => {
                self.tagged.insert(tag.to_string(), Tagged { def, claim: Some(builder), claimed: frame_now, building: None });
                format!("{did} (#{tag})")
            }
            Some(Task::Repair { target, .. }) | Some(Task::Assist { lab: target, .. }) => {
                self.helping.insert(builder, tag.to_string());
                if let Some(frame) = own.iter().find(|u| u.id == target && u.being_built) {
                    let (claim, claimed) = self.tagged.get(tag).map_or((None, frame_now), |t| (t.claim, t.claimed));
                    self.tagged.insert(tag.to_string(), Tagged { def: frame.def, claim, claimed, building: Some(frame.id) });
                }
                format!("help build #{tag} ({did})")
            }
            _ => did,
        }
    }
}

impl Brain {
    /// Every builder with a list: its next step ordered when it is ready for one.
    pub(super) fn play_lists(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let frame = tick.frame;
        let Some(mut pianist) = self.pianist.take() else { return };
        let mut under_fire: Vec<UnitId> = tick.events.iter().filter_map(|e| if let bot_protocol::Event::UnitDamaged { unit, .. } = e { Some(*unit) } else { None }).collect();
        under_fire.extend(pianist.under_fire(frame));
        let mut steps: Vec<(UnitId, Order, String, bool)> = Vec::new();
        pianist.list_held.retain(|id| own.iter().any(|u| u.id == *id));
        // A builder helping an id's building up is free when it stands or is gone: its guard or repair order is
        // ended (a guard never ends by itself), and its list goes on, or the pass has it back.
        let helpers: Vec<(UnitId, String)> = pianist.helping.iter().map(|(builder, id)| (*builder, id.clone())).collect();
        for (builder, id) in helpers {
            let help = |task: Option<&Task>| matches!(task, Some(Task::Repair { .. }) | Some(Task::Assist { .. }));
            let (queued, doing) = (help(pianist.queued.get(&builder)), help(pianist.tasks.get(&builder)));
            let def = pianist.tagged.get(&id).map(|t| t.def);
            let wanted = def.is_some_and(|def| matches!(pianist.tag_state(&id, def, builder, own), TagState::UnderWay(_) | TagState::Claimed(_)));
            if wanted && (queued || doing) {
                continue;
            }
            pianist.helping.remove(&builder);
            if queued {
                // Not begun yet (ordered behind the helper's own build): the help is taken off, the build goes on.
                pianist.queued.remove(&builder);
                pianist.queued_steps.remove(&builder);
            } else if doing {
                pianist.tasks.remove(&builder);
                pianist.list_steps.remove(&builder);
                commands.push(Command::Stop { unit: builder });
            }
            // Neither: the pass sent the helper elsewhere, and its step is back on its list (`execute_builder`).
        }
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            if pianist.scripts.get(&name).is_none_or(|s| s.is_empty()) || pianist.helping.contains_key(&unit.id) {
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
            if let Some((order, step)) = self.next_list_step(unit, &name, &mut pianist, picture, own, frame, kit) {
                steps.push((unit.id, order, step, status.queue_ahead));
            }
        }
        // Several builders reaching one id's step on this tick: the nearest starts it, the rest help it.
        let site = |order: &Order| match order {
            Order::Build(_, Site::Place(name)) => picture.places.iter().find(|p| p.name == *name).map(|p| p.at),
            Order::Build(_, Site::Spot(i)) => self.world.hello.metal_spots.get(*i).copied(),
            _ => None,
        };
        nearest_first(&mut steps, site, |id| own.iter().find(|u| u.id == id).map(|u| u.pos));
        self.pianist = Some(pianist);
        let mut claimed: HashMap<String, UnitId> = HashMap::new();
        for (id, order, step, queue) in steps {
            let name = self.actor_name(id);
            let tag = step_id(&step).1;
            let order = same_tick_help(order, tag, &claimed);
            match self.execute_builder(tick, kit, picture, id, &order, queue, Some(&step), commands) {
                Some(did) => {
                    if let (Order::Build(..), Some(tag)) = (&order, tag) {
                        claimed.entry(tag.to_string()).or_insert(id);
                    }
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    let did = match tag {
                        Some(tag) => pianist.note_tag(tag, id, own, did, queue, frame),
                        None => did,
                    };
                    let did = if step_is_low(&step) { format!("{did}; low priority on this step: it draws only surplus") } else { did };
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
    /// without asking. A bypass like the builders' lists: a factory in its sequence has no menu.
    pub(super) fn play_sequences(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let Some(pianist) = self.pianist.as_ref() else { return };
        let next: Vec<(UnitId, UnitDefId)> = own
            .iter()
            .filter(|u| !u.being_built && self.world.is_factory_def(u.def) && pianist.lab_queue.get(&u.id).is_none_or(|q| q.is_empty()))
            .filter_map(|u| self.sequence_unit(u, &self.actor_name(u.id), pianist).map(|def| (u.id, def)))
            .collect();
        for (id, def) in next {
            let name = self.actor_name(id);
            if let Some(did) = self.execute_factory(tick, id, &Order::Make(def), commands) {
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
        if pianist.refused_spots.contains_key(&i) {
            return format!("the engine refused a build at spot_{i} lately (wrecks or a unit on it)");
        }
        format!("spot_{i} is not free")
    }

    /// The next step of a builder's list as an order, its words kept. Steps that cannot be done are dropped and
    /// said; `assist` before any factory exists waits.
    #[allow(clippy::too_many_arguments)]
    fn next_list_step(&self, unit: &OwnUnit, name: &str, pianist: &mut Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) -> Option<(Order, String)> {
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        loop {
            let step = pianist.scripts.get_mut(name)?.pop_front()?;
            let (body, tag) = step_id(&step);
            let (body, _) = step_low(body);
            let mut words = body.split_whitespace();
            let kind = words.next().unwrap_or_default();
            // `extractor nearest` is the bare `extractor`: the free spot this builder reaches soonest, whatever the
            // start (the opening list is written before the start is known).
            let place = words.next().filter(|p| !(kind == "extractor" && *p == "nearest")).map(str::to_string);
            let at_place = |def: UnitDefId| match &place {
                _ if !can(def) => Err("this builder cannot build it".to_string()),
                Some(p) if picture.places.iter().any(|q| q.name == *p) => Ok(Order::Build(def, Site::Place(p.clone()))),
                Some(p) => Err(format!("{p} is not a place in the picture")),
                None => Err("it needs a place".to_string()),
            };
            let resolved: Result<Order, String> = match kind {
                "extractor" if !can(kit.extractor) => Err("this builder cannot build it".to_string()),
                "extractor" => {
                    let spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                    match &place {
                        Some(p) => match p.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                            Some(i) if spots.iter().any(|(j, _)| *j == i) => Ok(Order::Build(kit.extractor, Site::Spot(i))),
                            // Why it is not free (9.3): "not free" covered a spot nothing of ours reached, one the
                            // enemy held, one under wrecks and one another builder was taking (Cape Violet, 333 skips).
                            Some(i) => Err(self.why_not_free(i, unit, pianist, own, kit)),
                            None => Err(format!("{p} is not a spot")),
                        },
                        None => match spots.first() {
                            Some((i, _)) => Ok(Order::Build(kit.extractor, Site::Spot(*i))),
                            None => Err("no free spot in reach".to_string()),
                        },
                    }
                }
                "assist" => match own.iter().filter(|u| self.world.is_factory_def(u.def)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))) {
                    Some(factory) => Ok(Order::Help(factory.id)),
                    None => {
                        // Nothing to help yet: the step waits at the front of the list.
                        pianist.scripts.get_mut(name)?.push_front(step);
                        return None;
                    }
                },
                // `reclaim <handle>`: one unit of ours, taken apart for its metal (the `remove` tool).
                "reclaim" => match place.as_deref().and_then(|h| self.unit_by_handle(h, own)) {
                    Some(target) if target.id == unit.id => Err("a builder cannot take itself apart".to_string()),
                    Some(target) => Ok(Order::TakeApartUnit(target.id)),
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
                    Some(def) if can(def) => Ok(Order::Build(def, Site::Planned)),
                    Some(_) => Err("this builder cannot build it".to_string()),
                    None => Err(format!("'{other}' is not a unit name the game knows")),
                },
            };
            // A step with an id is one building (`Pianist::tag_state`): started by the first builder to reach it,
            // helped up by any other, done once it stands.
            let resolved = match (resolved, tag) {
                (Ok(order), Some(tag)) => {
                    let def = match &order {
                        Order::Build(def, _) => Some(*def),
                        _ => None,
                    };
                    match def.map(|def| pianist.tag_state(tag, def, unit.id, own)) {
                        None | Some(TagState::Free) => Ok(order),
                        Some(TagState::UnderWay(frame)) => Ok(Order::Repair(frame)),
                        Some(TagState::Claimed(builder)) => Ok(Order::Help(builder)),
                        Some(TagState::Stands) => Err(format!("#{tag} stands already")),
                        Some(TagState::Mine) => Err(format!("#{tag} is the build this builder is on")),
                        Some(TagState::Other(other)) => Err(format!("#{tag} is a {}: an id is one building", self.name(other))),
                    }
                }
                (resolved, _) => resolved,
            };
            match resolved {
                Ok(order) => return Some((order, step)),
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

    /// The word `low` anywhere in a step marks it and is not one of its words; a step without it is high.
    #[test]
    fn the_word_low_marks_a_step_and_leaves_its_words() {
        use super::{step_is_low, step_low, timed_assist};
        assert_eq!(step_low("assist 90 low"), ("assist 90".to_string(), true));
        assert_eq!(step_low("armsolar low"), ("armsolar".to_string(), true));
        assert_eq!(step_low("extractor spot_5"), ("extractor spot_5".to_string(), false));
        assert!(step_is_low("armsolar home low #s1") && !step_is_low("armsolar home #s1"));
        assert_eq!(timed_assist("assist 90 low"), Some(90));
    }

    /// A step's last word beginning with `#` is its id; the rest are its words.
    #[test]
    fn a_steps_id_is_its_last_word() {
        use super::step_id;
        assert_eq!(step_id("armavp avp_yard #avp1"), ("armavp avp_yard", Some("avp1")));
        assert_eq!(step_id("armsolar #s_2 "), ("armsolar", Some("s_2")));
        assert_eq!(step_id("extractor spot_5"), ("extractor spot_5", None));
        assert_eq!(step_id("armsolar #"), ("armsolar #", None));
        assert_eq!(step_id("#x"), ("#x", None));
    }

    /// Several builders reaching one id's step on the same tick: the nearest to the site plays first and starts
    /// it, and each later step with the id, once a build has claimed it, helps that builder.
    #[test]
    fn one_ids_steps_on_one_tick_are_one_build_by_the_nearest() {
        use super::super::menu::{Order, Site};
        use bot_protocol::{UnitDefId, Vec3};
        use std::collections::HashMap;
        let at = |x: f32| Vec3 { x, y: 0.0, z: 0.0 };
        let plant = UnitDefId(7);
        let build = |id: i32, step: &str| (UnitId(id), Order::Build(plant, Site::Place("avp_yard".into())), step.to_string(), false);
        let order = |steps: &[(UnitId, Order, String, bool)]| steps.iter().map(|s| s.0.0).collect::<Vec<_>>();
        let pos = |id: UnitId| Some(at(match id.0 { 1 => 0.0, 2 => 10.0, 3 => 900.0, 4 => 400.0, _ => 0.0 }));
        let mut steps = vec![build(1, "coravp avp_yard #avp1"), build(2, "corsolar #s1"), build(3, "coravp avp_yard #avp1"), build(4, "coravp avp_yard #avp1")];
        super::nearest_first(&mut steps, |_| Some(at(1000.0)), pos);
        assert_eq!(order(&steps), vec![3, 2, 1, 4]);
        // Without a site, or without an id, the order of play stands.
        let mut planned = vec![build(1, "coravp #avp1"), build(3, "coravp #avp1"), build(4, "coravp avp_yard"), build(2, "coravp avp_yard")];
        super::nearest_first(&mut planned, |_| None, pos);
        assert_eq!(order(&planned), vec![1, 3, 4, 2]);
        // Played in order: the first build claims the id for the tick, and the later builds of it become help.
        let mut claimed: HashMap<String, UnitId> = HashMap::new();
        assert!(matches!(super::same_tick_help(steps[0].1.clone(), Some("avp1"), &claimed), Order::Build(..)));
        claimed.insert("avp1".into(), UnitId(3));
        assert!(matches!(super::same_tick_help(steps[2].1.clone(), Some("avp1"), &claimed), Order::Help(UnitId(3))));
        assert!(matches!(super::same_tick_help(steps[1].1.clone(), Some("s1"), &claimed), Order::Build(..)));
        assert!(matches!(super::same_tick_help(steps[1].1.clone(), None, &claimed), Order::Build(..)));
    }

    /// Steps carrying the same id are one building: the first builder to reach the step starts it, another helps
    /// the builder on its way and then the frame, the step is done once the building stands, and the id is free
    /// again when the building is gone or the builder that claimed it is off that build.
    #[test]
    fn steps_with_the_same_id_are_one_building() {
        use super::super::{TagState, Tagged, Task};
        use bot_protocol::{OwnUnit, UnitDefId, Vec3};
        let unit = |id: i32, def: i32, being_built: bool| OwnUnit { id: UnitId(id), def: UnitDefId(def), pos: Vec3::default(), vel: Vec3::default(), health: 1.0, max_health: 1.0, being_built, idle: false, reload_frame: 0, facing: 0, stockpile: 0, stockpile_queued: 0, priority_low: false };
        let (plant, solar) = (UnitDefId(7), UnitDefId(8));
        let (first, second, frame) = (UnitId(1), UnitId(2), UnitId(50));
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut own = vec![unit(1, 3, false), unit(2, 3, false)];
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Free);
        // The first builder is ordered to build it: the id is its claim while it is on that build.
        pianist.tasks.insert(first, Task::Build { def: plant, near: Vec3::default(), spot: None, ordered: 100, started: false });
        assert_eq!(pianist.note_tag("avp1", first, &own, "build a armavp at avp_yard".into(), false, 100), "build a armavp at avp_yard (#avp1)");
        assert_eq!(pianist.tag_state("avp1", plant, second, &own), TagState::Claimed(first));
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Mine);
        assert_eq!(pianist.tag_state("avp1", solar, second, &own), TagState::Other(plant));
        // The second is sent to help the first, and is a helper of the id.
        pianist.tasks.insert(second, Task::Assist { lab: first, since: 110 });
        assert_eq!(pianist.note_tag("avp1", second, &own, "help constructor_1 build".into(), false, 110), "help build #avp1 (help constructor_1 build)");
        assert_eq!(pianist.helping.get(&second).map(String::as_str), Some("avp1"));
        // The engine makes the frame: another builder helps the frame; the first is on its own build.
        own.push(unit(50, 7, true));
        pianist.tagged.get_mut("avp1").unwrap().building = Some(frame);
        assert_eq!(pianist.tag_state("avp1", plant, second, &own), TagState::UnderWay(frame));
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Mine);
        // It stands: every step with the id is done, also in a list sent again.
        own[2].being_built = false;
        pianist.tasks.remove(&first);
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Stands);
        assert_eq!(pianist.tag_state("avp1", plant, second, &own), TagState::Stands);
        // It is destroyed: the id is free, and a step with it builds anew.
        own.pop();
        assert_eq!(pianist.tag_state("avp1", plant, second, &own), TagState::Free);
        assert!(!pianist.tagged.contains_key("avp1"));
        // A claim lapses when its builder is off the build (diverted, a new list) or dead.
        pianist.tagged.insert("s1".into(), Tagged { def: solar, claim: Some(first), claimed: 200, building: None });
        assert_eq!(pianist.tag_state("s1", solar, second, &own), TagState::Free);
    }

    /// A unit is under fire for three seconds after its last hit, not for the rest of the game.
    #[test]
    fn a_hit_is_forgotten_after_three_seconds() {
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.hits.insert(UnitId(1), 100);
        assert_eq!(pianist.under_fire(160).collect::<Vec<_>>(), vec![UnitId(1)]);
        // A soldier with an enemy in its reach since frame 100 is silent after six seconds without a shot, not
        // before, and not when it has fired since.
        pianist.in_reach.insert(UnitId(1), 100);
        assert_eq!(pianist.silent(UnitId(1), 0.3, 100 + 5 * 30), None);
        assert_eq!(pianist.silent(UnitId(1), 0.3, 100 + 7 * 30), Some(7));
        assert_eq!(pianist.silent(UnitId(1), 4.0, 100 + 7 * 30), None);
        pianist.fired.insert(UnitId(1), 130);
        assert_eq!(pianist.silent(UnitId(1), 0.3, 100 + 7 * 30), None);
        assert_eq!(pianist.silent(UnitId(2), 0.3, 500), None);
        assert_eq!(pianist.under_fire(100 + super::super::UNDER_FIRE_FRAMES + 1).count(), 0);
    }
}
