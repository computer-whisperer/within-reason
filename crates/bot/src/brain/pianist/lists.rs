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
use super::picture::{Picture, Place, clock};
use super::menu::{Order, Site};
use super::{Pianist, TagState, Tagged, Task};

/// A list step's words and its id: a last word beginning with `#` names the building the step is ("armavp
/// avp_yard #avp1"), so that steps carrying the same id on several builders' lists, or in a list sent again, are
/// one building (`Pianist::tag_state`). Steps reach here in `step_canonical`'s order, the id last.
pub(crate) fn step_id(step: &str) -> (&str, Option<&str>) {
    match step.trim_end().rsplit_once(char::is_whitespace) {
        Some((words, last)) if last.len() > 1 && last.starts_with('#') => (words.trim_end(), Some(&last[1..])),
        _ => (step, None),
    }
}

/// A step as the tools store it: the builder's words, then `low`, then `start`, then the id. The player writes the
/// three in any order after the building's name (`armafus #af1 start`, `armsolar #s3 low`: econ-flagship-8 and
/// 1794105281-human each lost a turn's lists to the order when the id had to come last), and every reader after
/// the tool sees one order. Refused: an id before the building's name, a bare `#`, two ids.
pub(crate) fn step_canonical(step: &str) -> Result<String, String> {
    let mut words: Vec<&str> = Vec::new();
    let (mut low, mut start, mut id) = (false, false, None);
    for (i, word) in step.split_whitespace().enumerate() {
        if let Some(name) = word.strip_prefix('#') {
            if i == 0 {
                return Err("an id follows the building's name (armavp avp_yard #avp1); it does not begin the step".to_string());
            }
            if name.is_empty() {
                return Err("an id is `#` and a name (armavp avp_yard #avp1)".to_string());
            }
            if id.replace(word).is_some() {
                return Err("one id a step (armavp avp_yard #avp1)".to_string());
            }
        } else if word == "low" {
            low = true;
        } else if word == "start" {
            start = true;
        } else {
            words.push(word);
        }
    }
    words.extend(low.then_some("low"));
    words.extend(start.then_some("start"));
    words.extend(id);
    Ok(words.join(" "))
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

/// A list step's words without the word `start`, and whether it carried it: the builder places the frame and
/// takes its next step the moment the frame stands up, leaving it to whoever finishes it (the tier-1 constructors
/// and nanos the player lists with `finish`, the pick's finish moves), the way the people run one advanced
/// constructor over many moho frames (H-HANDS-LIST-START; Center Command 2v4, 2026-10-19: u6bkep's borrowed Cortex
/// constructor left two of its frames to his Armada constructors). "cormoho spot_28 start", "armfus home start #f1".
/// A count on a step's first word (`extractor:20`, `corwin:6 low`, `corllt:2 spot_3`) repeats the step, as a
/// `produce` entry's count repeats a unit: the step without it and how many. A step without a count is one. A
/// count of 0 or over 60 is refused with the reason (a builder's standing work is twenty extractors, eight minutes).
pub(crate) fn step_count(step: &str) -> Result<(String, u32), String> {
    let step = step.trim();
    let (first, rest) = step.split_once(' ').map_or((step, ""), |(a, b)| (a, b));
    let Some((kind, count)) = first.split_once(':') else { return Ok((step.to_string(), 1)) };
    let count: u32 = count.parse().map_err(|_| format!("a count is a number after the step's first word (extractor:20), not '{first}'"))?;
    if !(1..=60).contains(&count) {
        return Err(format!("a count is 1 to 60 (extractor:20), not {count}"));
    }
    if kind.is_empty() {
        return Err(format!("a count goes after the step's first word (extractor:20), not '{first}'"));
    }
    Ok((if rest.is_empty() { kind.to_string() } else { format!("{kind} {rest}") }, count))
}

/// A list as the player reads it back: runs of one step (without an id) as the tool's own count (`extractor:20`),
/// so a list of twenty reads as one entry and goes back in as written.
pub(crate) fn compress_steps<'a>(steps: impl IntoIterator<Item = &'a String>) -> String {
    let mut runs: Vec<(String, usize)> = Vec::new();
    for step in steps {
        match runs.last_mut() {
            Some((last, n)) if last == step && step_id(step).1.is_none() => *n += 1,
            _ => runs.push((step.clone(), 1)),
        }
    }
    runs.into_iter()
        .map(|(step, n)| match (n, step.split_once(' ')) {
            (1, _) => step,
            (n, Some((kind, rest))) => format!("{kind}:{n} {rest}"),
            (n, None) => format!("{step}:{n}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn step_start(body: &str) -> (String, bool) {
    let start = body.split_whitespace().any(|w| w == "start");
    (body.split_whitespace().filter(|w| *w != "start").collect::<Vec<_>>().join(" "), start)
}

/// Whether a whole step carries the word `start`.
pub(crate) fn step_is_start(step: &str) -> bool {
    step_start(&step_low(step_id(step).0).0).1
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
        let task = if queue { self.queued.get(&builder).and_then(|q| q.back()).map(|q| &q.task) } else { self.tasks.get(&builder) }.cloned();
        match task {
            // A build on a frame that stands already (the id's frame finished by this builder) keeps the frame as
            // the id's building: inserted with none, the id was free again the moment the frame stood and its
            // builder moved on, and the next list to reach the step built the thing again (Center Command 2v4,
            // 2026-10-19: a second advanced plant at 8:41, 62 s after the first stood, from a constructor listed
            // at 4:27 while it was being built).
            Some(Task::Build { def, frame, .. }) => {
                let claimed = self.tagged.get(tag).filter(|_| frame.is_some()).map_or(frame_now, |t| t.claimed);
                self.tagged.insert(tag.to_string(), Tagged { def, claim: Some(builder), claimed, building: frame });
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
    pub(crate) fn play_lists(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, commands: &mut Vec<Command>) {
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
            let (queued, doing) = (pianist.queue_of(builder).any(|t| help(Some(t))), help(pianist.tasks.get(&builder)));
            let def = pianist.tagged.get(&id).map(|t| t.def);
            let wanted = def.is_some_and(|def| matches!(pianist.tag_state(&id, def, builder, own), TagState::UnderWay(_) | TagState::Claimed(_)));
            if wanted && (queued || doing) {
                continue;
            }
            pianist.helping.remove(&builder);
            if queued {
                // Not begun yet (ordered behind the helper's own build): the help is taken off, the build goes on.
                if let Some(queue) = pianist.queued.get_mut(&builder) {
                    queue.retain(|q| !help(Some(&q.task)));
                    if queue.is_empty() {
                        pianist.queued.remove(&builder);
                    }
                }
            } else if doing {
                pianist.tasks.remove(&builder);
                pianist.list_steps.remove(&builder);
                commands.push(Command::Stop { unit: builder });
            }
            // Neither: the pass sent the helper elsewhere, and its step is back on its list (`execute_builder`).
        }
        let builders: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)).collect();
        for unit in builders {
            let name = self.actor_name(unit.id);
            // A list that has run out (an empty deque: the player's lists are only ever inserted whole) is noted once,
            // with its frame, unless its last step was a bare `assist`, which stands to the horizon (H-WAKE-LISTS-ENDED).
            if pianist.scripts.get(&name).is_some_and(|s| s.is_empty()) && !pianist.list_ended.contains_key(&name) {
                let standing_assist = matches!(pianist.tasks.get(&unit.id), Some(Task::Assist { .. }))
                    && pianist.list_steps.get(&unit.id).is_some_and(|(step, _)| step.split_whitespace().next() == Some("assist") && timed_assist(step).is_none());
                if !standing_assist {
                    pianist.list_ended.insert(name.clone(), frame);
                }
            }
            if pianist.scripts.get(&name).is_none_or(|s| s.is_empty()) || pianist.helping.contains_key(&unit.id) {
                continue;
            }
            let status = self.builder_status(&pianist, unit, picture, &under_fire, own);
            // A list plays under threat too: it is the player's order (onepass-player-2, 12:23: the commander's list
            // to spot_68 away from the block never started, the block being within 800). Not after the pick has
            // sent the builder away from an enemy, while that enemy is still on it: the pick's way out stands
            // (player-31: 83 of 92 such retreats were undone by the list within three seconds).
            // The hold outlasts the builder's own danger while the site of its waiting step is under a party
            // (Cape Violet 2026-10-18: constructor_31731 flipped eight times in 35 s between its torpedo-launcher step
            // at spot_16 and the pick's walk away from the Dolphin covering that spot, the hold lifting each time the
            // builder was out of the Dolphin's reach and the list sending it back in).
            let site_covered = pianist.list_held.contains(&unit.id)
                && self.step_site(pianist.scripts.get(&name).and_then(|s| s.front()).map(String::as_str), &picture.places).is_some_and(|site| picture.parties.iter().any(|p| p.at.dist2d(site) < super::menu::THREATENED));
            // A builder healing a rover beside it (H-HANDS-HEAL) takes no step until the rover leaves.
            if pianist.list_is_held(unit.id, status.threatened || site_covered || pianist.healing.contains_key(&unit.id)) {
                continue;
            }
            let task = pianist.tasks.get(&unit.id).cloned();
            // The player's list outranks the bot's fillers: a builder helping a factory, walking, reclaiming or
            // repairing takes its next step at once (plan-1: the commander helped a plant from 3:46 to 14:24 while
            // four lists waited, since a guard order never ends). A build in progress keeps the 60 % rule.
            let list_frame = pianist.script_frame.get(&name).copied().unwrap_or(0);
            // A build on a `start` step: the builder leaves the frame the moment it stands up, and its next order
            // replaces the build rather than queueing behind it (H-HANDS-LIST-START).
            let leaving = matches!(&task, Some(Task::Build { started: true, frame: Some(_), .. })) && pianist.list_steps.get(&unit.id).is_some_and(|(step, _)| step_is_start(step));
            let ready = match &task {
                None => unit.idle,
                Some(Task::Build { started: true, .. }) => status.queue_ahead || leaving,
                // A list's own reclaim step (`reclaim <handle|cell|place>`) runs until the builder is idle: the unit,
                // or the wrecks it was given, are gone. The bot's filler reclaim gives way to a list at once.
                Some(Task::Reclaim { .. }) | Some(Task::ReclaimUnit { .. }) if pianist.list_steps.get(&unit.id).is_some_and(|(step, _)| step.starts_with("reclaim ")) => unit.idle,
                // A build the engine has not started yet gives way to a list set after it was ordered, unless that
                // list's first step is this very build (`list_caught_up`): then the order stands until the frame is up.
                Some(Task::Build { ordered, .. }) => status.queue_ahead || (list_frame > *ordered && pianist.list_caught_up.get(&name) != Some(ordered)),
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
            // A field step done with the wrecks it was given (five an order) goes again while a field is still at
            // the cell or place; else the wrecks are gone and the list goes on.
            if let Some(Task::Reclaim { .. }) = &task
                && let Some((step, _)) = pianist.list_steps.get(&unit.id).cloned()
                && let Some(what) = step.strip_prefix("reclaim ").map(|w| step_id(w).0.trim())
            {
                if self.wreck_field_at(what, &picture.places).is_some() {
                    if let Some(list) = pianist.scripts.get_mut(&name) {
                        list.push_front(step);
                    }
                } else {
                    let text = format!("the wrecks at {what} are taken apart; its list goes on");
                    pianist.done.push(format!("{} {name}: {text}", clock(frame)));
                    pianist.note(frame, format!("{name}: {text}"));
                }
            }
            // The stand (docs/design/2026-10-29-base-layout.md §1.1, H-ECO-STAND): solved once for the commander
            // from its list's leading steps; then every leading step whose slot or spot the stand holds goes now,
            // each behind the one before, as one queue, so the commander never idles between them. The first step
            // not the stand's waits for its turn as before.
            if self.stand.is_none() && kit.commander == unit.def {
                self.solve_stand(unit, &mut pianist, picture, own, frame, kit);
            }
            let mut queue = status.queue_ahead && !leaving;
            let mut prefer = self.stand_spots(unit, &pianist, picture, own, frame, kit);
            let (mut batched_spots, mut batched_defs): (Vec<usize>, Vec<UnitDefId>) = (Vec::new(), Vec::new());
            while let Some((order, step)) = self.next_list_step(unit, &name, &mut pianist, picture, own, frame, kit, &mut prefer, &mut batched_spots) {
                let on_the_stand = match &order {
                    Order::Build(def, Site::Planned) => self.free_stand_slots(unit.id, *def, own, &pianist).len() > batched_defs.iter().filter(|d| *d == def).count(),
                    Order::Build(_, Site::Spot(i)) => self.stand.as_ref().is_some_and(|(who, s)| *who == unit.id && s.slots.iter().any(|slot| matches!(slot, layout::Slot::Extractor { spot: Some(spot), .. } if spot == i))),
                    _ => false,
                };
                if let Order::Build(def, Site::Planned) = &order {
                    batched_defs.push(*def);
                }
                steps.push((unit.id, order, step, queue));
                let next_on_the_stand = pianist.scripts.get(&name).and_then(|s| s.front()).is_some_and(|next| self.step_on_the_stand(next, unit, &pianist, own, &prefer, &batched_defs));
                if !on_the_stand || !next_on_the_stand {
                    break;
                }
                queue = true;
            }
        }
        // Several builders reaching one id's step on this tick: the nearest starts it, the rest help it.
        let site = |order: &Order| match order {
            Order::Build(_, Site::Place(name)) => picture.places.iter().find(|p| p.name == *name).map(|p| p.at),
            Order::Build(_, Site::Spot(i)) => self.world.hello.metal_spots.get(*i).copied(),
            Order::Build(_, Site::Tile(at)) => Some(*at),
            _ => None,
        };
        nearest_first(&mut steps, site, |id| own.iter().find(|u| u.id == id).map(|u| u.pos));
        self.pianist = Some(pianist);
        let mut claimed: HashMap<String, UnitId> = HashMap::new();
        // A step of a builder's batch that could not be played: the steps behind it go back to the front of its list
        // in their order, not out of turn behind it.
        let mut failed: HashMap<UnitId, Vec<String>> = HashMap::new();
        for (id, order, step, queue) in steps {
            let name = self.actor_name(id);
            if let Some(back) = failed.get_mut(&id) {
                back.push(step);
                continue;
            }
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
                    let did = if step_is_start(&step) { format!("{did}; it leaves the frame to other builders once it stands up (start)") } else { did };
                    pianist.done.push(format!("{} {name}: {did} (from its list)", clock(frame)));
                    pianist.played.push(json!({ "actor": name, "kind": "builder", "played": step, "did": did, "source": "list" }));
                    self.journal.note_from("list", frame, "builder", json!({ "actor": name, "step": step }), json!({ "did": did }));
                }
                // Never lost silently (13.3: 39 of 139 lists lost extractor steps): the same build under way is
                // the step done; anything else is said.
                None => {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    let text = match pianist.tasks.get(&id) {
                        // The build it was ordered and has not begun (a list re-sent with that step first): the step
                        // stays at the front and the order stands until the frame is up. Taken as done before, the
                        // next step's order cancelled the walk and the build never happened (the impedance tally,
                        // 2026-10-16: 104 such lines in 18 games; apex-v022-carrot-sym50-opus-1 2:45 and 3:15, two
                        // opening extractors 3.3 minutes late; human-27 6:56 and 11:07, held to four extractors).
                        Some(Task::Build { started: false, ordered, .. }) => {
                            let ordered = *ordered;
                            let text = format!("the step '{step}' of its list is the build it was ordered and has not begun: the step waits for the frame");
                            if let Some(script) = pianist.scripts.get_mut(&name) {
                                script.push_front(step.clone());
                            }
                            pianist.list_caught_up.insert(name.clone(), ordered);
                            text
                        }
                        Some(Task::Build { started: true, .. }) => format!("the step '{step}' of its list is the build already under way: taken as done"),
                        _ => format!("the step '{step}' of its list could not be played this second and was dropped"),
                    };
                    pianist.done.push(format!("{} {name}: {text}", clock(frame)));
                    pianist.note(frame, format!("{name}: {text}"));
                    failed.insert(id, Vec::new());
                }
            }
        }
        let failed: Vec<(String, Vec<String>)> = failed.into_iter().map(|(id, back)| (self.actor_name(id), back)).collect();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (name, back) in failed {
            if let Some(list) = pianist.scripts.get_mut(&name) {
                for step in back.into_iter().rev() {
                    list.push_front(step);
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
        if self.extractor_of_ours_at(i, own, kit) {
            return format!("spot_{i} holds our extractor already");
        }
        if self.enemy_buildings.values().any(|(def, pos, _)| pos.dist2d(spot) < radius && self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)) {
            return format!("spot_{i} holds his extractor: it is taken by killing that, not by a list");
        }
        if let Some((id, _)) = pianist.tasks.iter().chain(pianist.queued_tasks()).find(|(id, t)| **id != unit.id && matches!(t, Task::Build { spot: Some(s), .. } if *s == i)) {
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

    /// An extractor of ours, standing or being built, on this spot.
    fn extractor_of_ours_at(&self, i: usize, own: &[OwnUnit], kit: &Kit) -> bool {
        let Some(spot) = self.world.hello.metal_spots.get(i).copied() else { return false };
        let radius = self.spot_occupied_radius();
        own.iter().any(|u| kit.is_extractor(u.def) && u.pos.dist2d(spot) < radius)
    }

    /// The next step of a builder's list as an order, its words kept. Steps that cannot be done are dropped and
    /// said; `assist` before any factory exists waits.
    /// The wreck field a list step names: the richest in the grid cell (`E4`), else the nearest within two fields'
    /// radius of the named place.
    /// Where a list step builds, when its second word is a spot or a place: the site the hold on a sent-away builder's
    /// list watches (`play_lists`), and the words for its picture entry.
    pub(super) fn step_site(&self, step: Option<&str>, places: &[Place]) -> Option<Vec3> {
        let (body, _) = step_id(step?);
        let (body, _) = step_low(body);
        let (body, _) = step_start(&body);
        let place = body.split_whitespace().nth(1)?;
        match place.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
            Some(i) => self.world.hello.metal_spots.get(i).copied(),
            None => places.iter().find(|p| p.name == place).map(|p| p.at),
        }
    }

    /// An `extractor [place]` step's tile on a metal map: nearest the builder within `TILE_REACH`, or within the
    /// place's reach (`Brain::place_reach`); the refusal says which.
    fn tile_for_step(&self, unit: &OwnUnit, pianist: &Pianist, own: &[OwnUnit], frame: i32, place: Option<&Place>) -> Result<Vec3, String> {
        match place {
            // The stand's own extractor tiles first, so its other slots stay clear; then the metal nearest the builder.
            None => self
                .stand_tiles(unit, pianist, own, frame)
                .into_iter()
                .next()
                .or_else(|| self.tile_site(unit, own, pianist, unit.pos, super::super::economy::TILE_REACH, frame))
                .ok_or_else(|| format!("no free metal within {:.0} of it", super::super::economy::TILE_REACH)),
            Some(p) => self.tile_site(unit, own, pianist, p.at, self.place_reach(p), frame).ok_or_else(|| format!("no free metal within {} of {}", self.place_reach(p) as i32, p.name)),
        }
    }

    pub(super) fn wreck_field_at(&self, what: &str, places: &[Place]) -> Option<Vec3> {
        if let Some(cell) = self.world.cells.parse(what) {
            return self.reclaim.fields.iter().filter(|f| self.world.cells.cell_of(f.at) == cell).max_by(|a, b| a.metal.total_cmp(&b.metal)).map(|f| f.at);
        }
        let place = places.iter().find(|p| p.name == what.trim())?;
        self.reclaim.fields.iter().filter(|f| f.at.dist2d(place.at) < 2.0 * super::super::reclaim::FIELD_RADIUS).min_by(|a, b| a.at.dist2d(place.at).total_cmp(&b.at.dist2d(place.at))).map(|f| f.at)
    }

    #[allow(clippy::too_many_arguments)]
    /// `prefer`: the spots the stand holds for this builder, taken first by its bare `extractor` steps and removed
    /// as they are; `batched`: the spots its steps took this tick already, which a bare step passes over (several
    /// steps of one builder go in one tick at the stand, and a spot is this builder's only once).
    #[allow(clippy::too_many_arguments)]
    fn next_list_step(&self, unit: &OwnUnit, name: &str, pianist: &mut Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit, prefer: &mut Vec<usize>, batched: &mut Vec<usize>) -> Option<(Order, String)> {
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        loop {
            let step = pianist.scripts.get_mut(name)?.pop_front()?;
            let (body, tag) = step_id(&step);
            let (body, _) = step_low(body);
            let (body, _) = step_start(&body);
            let mut words = body.split_whitespace();
            let kind = words.next().unwrap_or_default();
            // `extractor nearest` is the bare `extractor`: the free spot this builder reaches soonest, whatever the
            // start (the opening list is written before the start is known).
            let place = words.next().filter(|p| !(kind == "extractor" && *p == "nearest")).map(str::to_string);
            // An id's building this builder cannot make: the step is its help on it (`tag_state` below), the
            // place and the start another builder's (the impedance tally, 2026-10-16: "15 constructors put on it
            // (#silo1)", the tier-1 constructors' steps skipped as "this builder cannot build it").
            let helps = |def: UnitDefId| tag.is_some() && !can(def);
            let at_place = |def: UnitDefId| match &place {
                _ if !can(def) && !helps(def) => Err("this builder cannot build it".to_string()),
                Some(p) if picture.places.iter().any(|q| q.name == *p) => Ok(Order::Build(def, Site::Place(p.clone()))),
                Some(p) => Err(format!("{p} is not a place in the picture")),
                None if helps(def) => Ok(Order::Build(def, Site::Planned)),
                None => Err("it needs a place".to_string()),
            };
            // The word `extractor` is the builder's own: the tier-2 one for an advanced constructor, which builds no other.
            let extractor = if can(kit.extractor) { kit.extractor } else { kit.advanced_extractor };
            let resolved: Result<Order, String> = match kind {
                "extractor" if !can(extractor) => Err("this builder cannot build it".to_string()),
                // On a metal map (no spots): the coverage's best tile nearest the builder, or within the named place
                // (a cell, a field, a mark), given to the engine exactly (the metal-maps design §1.3).
                "extractor" if self.coverage.is_some() => match &place {
                    None => self.tile_for_step(unit, pianist, own, frame, None),
                    Some(p) => match picture.places.iter().find(|q| q.name == *p) {
                        Some(q) => self.tile_for_step(unit, pianist, own, frame, Some(q)),
                        None => Err(format!("{p} is not a place in the picture")),
                    },
                }
                .map(|at| Order::Build(extractor, Site::Tile(at))),
                "extractor" => {
                    let mut spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                    spots.retain(|(i, _)| !batched.contains(i));
                    if let Some(k) = prefer.iter().position(|p| spots.iter().any(|(j, _)| j == p)) {
                        let i = prefer.remove(k);
                        spots.retain(|(j, _)| *j != i);
                        spots.insert(0, (i, 0.0));
                    }
                    let mut taking = |i: usize| {
                        batched.push(i);
                        Ok(Order::Build(extractor, Site::Spot(i)))
                    };
                    match &place {
                        Some(p) => match p.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                            Some(i) if spots.iter().any(|(j, _)| *j == i) => taking(i),
                            // Why it is not free (9.3): "not free" covered a spot nothing of ours reached, one the
                            // enemy held, one under wrecks and one another builder was taking (Cape Violet, 333 skips).
                            Some(i) => Err(self.why_not_free(i, unit, pianist, own, kit)),
                            None => Err(format!("{p} is not a spot")),
                        },
                        None => match spots.first() {
                            Some((i, _)) => taking(*i),
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
                // `reclaim <handle>`: one unit of ours, taken apart for its metal (the `remove` tool); `reclaim <cell>`
                // or `reclaim <place>`: the wreck field there (the impedance tally, 2026-10-16: "every other
                // constructor takes apart wreck fields" rated 0.14 to 0.27 on the menu's three nearest fields).
                "reclaim" => {
                    let what = place.as_deref().unwrap_or_default();
                    match self.unit_named(what, own) {
                        Some(target) if target.id == unit.id => Err("a builder cannot take itself apart".to_string()),
                        Some(target) => Ok(Order::TakeApartUnit(target.id)),
                        None => match self.wreck_field_at(what, &picture.places) {
                            Some(at) => Ok(Order::TakeApart(at)),
                            None if what.is_empty() => Err("reclaim takes a handle (armsolar_31002), a grid cell (E4) or a place (spot_40)".to_string()),
                            None => Err(format!("no wreck field is known at {what}, and nothing of ours has that name")),
                        },
                    }
                }
                // `finish <handle>`: an unfinished structure of ours, whoever began it, as this builder's own build
                // (the report's `unfinished structures` line gives the handles).
                "finish" => match place.as_deref() {
                    None => Err("finish takes the handle of an unfinished structure of ours (armsolar_31002), as the report's `unfinished structures` line writes it".to_string()),
                    Some(handle) => match self.unit_by_handle(handle, own) {
                        Some(target) if target.being_built => Ok(Order::Finish(target.id)),
                        Some(_) => Err(format!("{handle} stands finished already")),
                        None => Err(format!("nothing of ours by the name {handle} stands: the frame is gone")),
                    },
                },
                // A defence at a spot whose extractor the engine refused lately waits with its extractor, unless an
                // extractor of ours is there after all (another builder's frame took the spot first, so this one's
                // was refused: the impedance tally, 2026-10-16, 150 turret steps skipped so, leg-player-1's spot_14
                // undefended and lost to a Tick at 24:05).
                other if self.world.def_named(other).is_some_and(|def| self.placed_at_place(def) && self.world.def(def).is_some_and(|d| d.weapon_count > 0))
                    && place.as_deref().and_then(|p| p.strip_prefix("spot_")).and_then(|n| n.parse::<usize>().ok()).is_some_and(|i| pianist.refused_spots.get(&i).is_some_and(|until| *until > frame) && !self.extractor_of_ours_at(i, own, kit)) =>
                {
                    Err(format!("the extractor at {} was refused lately; a defence there waits for it", place.as_deref().unwrap_or("?")))
                }
                // Any unit by its internal name; one that stands at a place needs the place, and any other building
                // takes one when given.
                other => match self.world.def_named(other) {
                    Some(def) if self.placed_at_place(def) || place.is_some() => at_place(def),
                    Some(def) if can(def) || helps(def) => Ok(Order::Build(def, Site::Planned)),
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
                        // Nobody has started #tag and this builder cannot: the step waits at the front of the list
                        // for a builder that can (the entry says so).
                        Some(TagState::Free) if def.is_some_and(|d| !can(d)) => {
                            pianist.scripts.get_mut(name)?.push_front(step);
                            return None;
                        }
                        None | Some(TagState::Free) => Ok(order),
                        // The id's frame stands: this builder finishes it as its own build (a repair task was a
                        // filler to the list, overrun by its next step a second later: rove-move-1 17:57).
                        Some(TagState::UnderWay(frame)) => Ok(Order::Finish(frame)),
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
    /// The site of a step: a spot by its number in the map's list, a place by its name, nothing for a step without one.
    #[test]
    fn a_steps_site_is_its_spot_or_its_place() {
        use super::super::fixtures::{at, brain_of};
        use super::super::Place;
        let mut brain = brain_of(&["corcom", "cordl"]);
        brain.world.hello.metal_spots = vec![at(100.0, 100.0), at(2000.0, 900.0)];
        let places = vec![Place { name: "radar_hill".to_string(), at: at(500.0, 700.0), spot: None, cells: None }];
        assert_eq!(brain.step_site(Some("cordl spot_1"), &places), Some(at(2000.0, 900.0)));
        assert_eq!(brain.step_site(Some("corllt radar_hill #t1"), &places), Some(at(500.0, 700.0)));
        assert_eq!(brain.step_site(Some("corllt nowhere"), &places), None);
        assert_eq!(brain.step_site(Some("assist 25"), &places), None);
        assert_eq!(brain.step_site(None, &places), None);
    }

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
        let build = |started| Task::Build { def: UnitDefId(7), near: Vec3::default(), spot: None, ordered: 100, started, frame: started.then_some(UnitId(70)), facing: 0 };
        let queued = |pianist: &mut super::Pianist, builder| {
            pianist.tasks.insert(builder, build(true));
            pianist.queue_push(builder, build(false), Some("extractor".to_string()));
            pianist.list_steps.insert(builder, ("extractor".to_string(), 100));
        };
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let (kept, replaced, cancelled, unlisted) = (UnitId(1), UnitId(2), UnitId(3), UnitId(4));
        for builder in [kept, replaced, cancelled, unlisted] {
            queued(&mut pianist, builder);
        }
        // The pass's own queued build, on a builder without a list: a cancel of no list leaves it.
        pianist.queued.get_mut(&unlisted).unwrap().front_mut().unwrap().step = None;
        pianist.list_replaced(replaced, true);
        pianist.list_replaced(cancelled, false);
        pianist.list_replaced(unlisted, false);
        assert!(!pianist.list_steps.contains_key(&replaced), "the old list's step no longer returns to a list");
        assert!(pianist.take_queued(kept, "commander", 200, None).is_none());
        assert!(matches!(pianist.tasks.get(&kept), Some(Task::Build { started: false, ordered: 200, .. })));
        assert_eq!(pianist.list_steps.get(&kept), Some(&("extractor".to_string(), 200)));
        assert!(pianist.take_queued(unlisted, "constructor_4", 200, None).is_none());
        assert!(matches!(pianist.tasks.get(&unlisted), Some(Task::Build { started: false, .. })));
        for builder in [replaced, cancelled] {
            assert!(matches!(pianist.take_queued(builder, "commander", 200, None), Some(Command::Stop { unit }) if unit == builder));
            assert!(!pianist.tasks.contains_key(&builder) && !pianist.queued.contains_key(&builder));
        }
        assert!(pianist.done.last().unwrap().contains("commander: dropped the step 'extractor' its old list had queued"), "{:?}", pianist.done);
        // A queued build the engine has begun as the list was replaced is kept, its frame the task's (migration-3-1
        // 0:19: the opening's solar dropped at 3% built); a begun frame of another type is not it.
        let (begun, other) = (UnitId(5), UnitId(6));
        queued(&mut pianist, begun);
        queued(&mut pianist, other);
        pianist.list_replaced(begun, true);
        pianist.list_replaced(other, true);
        let def = match pianist.queue_of(begun).next() { Some(Task::Build { def, .. }) => *def, _ => unreachable!() };
        let frame_pos = Vec3 { x: 100.0, y: 0.0, z: 200.0 };
        assert!(pianist.take_queued(begun, "commander", 200, Some((UnitId(50), def, frame_pos))).is_none());
        assert!(matches!(pianist.tasks.get(&begun), Some(Task::Build { started: true, frame: Some(UnitId(50)), near, .. }) if near.x == 100.0), "{:?}", pianist.tasks.get(&begun));
        assert!(pianist.done.last().unwrap().contains("commander: kept the step 'extractor' its old list had queued"), "{:?}", pianist.done);
        assert!(matches!(pianist.take_queued(other, "commander", 200, Some((UnitId(51), UnitDefId(def.0 + 1), frame_pos))), Some(Command::Stop { .. })));
    }

    /// A `reclaim` step names a wreck field by its grid cell (the richest there) or by a place (the nearest within
    /// two fields' radius), or a unit of ours by handle; an id step on a builder that cannot make the type waits at
    /// the front of its list for a builder that can to start it, and then is its help.
    #[test]
    fn a_reclaim_step_finds_the_field_and_an_id_step_the_builder_cannot_make_waits_then_helps() {
        use super::super::fixtures::{at, e3};
        use super::super::menu::Order;
        use super::super::picture::{Picture, Place};
        use super::{Pianist, Tagged, Task};
        use crate::brain::reclaim::WreckField;
        use crate::brain::roster::Kit;
        use bot_protocol::UnitDefId;
        use std::collections::VecDeque;
        let (mut brain, ours, _enemies, parties) = e3();
        // Two fields in E3 (x 4096-5120, z 2048-3072 of a map 8192 by 6144), one far away.
        brain.reclaim.fields = vec![
            WreckField { at: at(4500.0, 2600.0), metal: 300.0, safe: true, wrecks: Vec::new() },
            WreckField { at: at(4700.0, 2700.0), metal: 500.0, safe: true, wrecks: Vec::new() },
            WreckField { at: at(1000.0, 1000.0), metal: 200.0, safe: true, wrecks: Vec::new() },
        ];
        let places = vec![Place { name: "spot_40".into(), at: at(4000.0, 2600.0), spot: Some(40), cells: None }, Place { name: "spot_1".into(), at: at(7000.0, 5000.0), spot: Some(1), cells: None }];
        assert_eq!(brain.wreck_field_at("E3", &places), Some(at(4700.0, 2700.0)), "the richest in the cell");
        assert_eq!(brain.wreck_field_at("e3", &places), Some(at(4700.0, 2700.0)));
        assert_eq!(brain.wreck_field_at("E7", &places), None, "no seventh row on a map 6,144 high");
        assert_eq!(brain.wreck_field_at("spot_40", &places), Some(at(4500.0, 2600.0)), "the nearest to the place");
        assert_eq!(brain.wreck_field_at("spot_1", &places), None);
        assert_eq!(brain.wreck_field_at("Z9", &places), None);
        // The steps, played by a unit of the fixture (no build options: it can make nothing itself).
        let id = UnitDefId(3);
        let kit = Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, vehicle_constructor: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &crate::brain::roster::ROSTERS[0] };
        let picture = Picture { rules: String::new(), state: serde_json::json!({}), places, parties };
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let unit = &ours[0];
        let name = "constructor_100".to_string();
        pianist.scripts.insert(name.clone(), VecDeque::from(["reclaim E3".to_string(), "reclaim Z9".to_string(), "reclaim".to_string(), "armwar #w1".to_string(), "armllt spot_40".to_string()]));
        let step = brain.next_list_step(unit, &name, &mut pianist, &picture, &ours, 100, &kit, &mut Vec::new(), &mut Vec::new());
        assert!(matches!(&step, Some((Order::TakeApart(field), s)) if *field == at(4700.0, 2700.0) && s == "reclaim E3"), "{step:?}");
        // The next two are skipped and said; the id step waits at the front for a builder that can start #w1.
        assert!(brain.next_list_step(unit, &name, &mut pianist, &picture, &ours, 100, &kit, &mut Vec::new(), &mut Vec::new()).is_none());
        assert!(pianist.done.iter().any(|d| d.contains("skipped the step 'reclaim Z9'") && d.contains("no wreck field is known at Z9")), "{:?}", pianist.done);
        assert!(pianist.done.iter().any(|d| d.contains("skipped the step 'reclaim'") && d.contains("reclaim takes a handle")), "{:?}", pianist.done);
        assert_eq!(pianist.scripts[&name].front().map(String::as_str), Some("armwar #w1"));
        // Another builder claims #w1: this one's step is its help.
        pianist.tagged.insert("w1".into(), Tagged { def: UnitDefId(2), claim: Some(UnitId(101)), claimed: 90, building: None });
        pianist.tasks.insert(UnitId(101), Task::Build { def: UnitDefId(2), near: at(0.0, 0.0), spot: None, ordered: 90, started: false, frame: None, facing: 0 });
        let step = brain.next_list_step(unit, &name, &mut pianist, &picture, &ours, 110, &kit, &mut Vec::new(), &mut Vec::new());
        assert!(matches!(&step, Some((Order::Help(UnitId(101)), _))), "{step:?}");
        // Without an id, a step it cannot make is skipped as before.
        let step = brain.next_list_step(unit, &name, &mut pianist, &picture, &ours, 120, &kit, &mut Vec::new(), &mut Vec::new());
        assert!(step.is_none() && pianist.done.iter().any(|d| d.contains("skipped the step 'armllt spot_40'") && d.contains("cannot build")), "{step:?} {:?}", pianist.done);
    }

    /// The word `low` anywhere in a step marks it and is not one of its words; a step without it is high.
    #[test]
    fn the_word_low_marks_a_step_and_leaves_its_words() {
        use super::{step_is_low, step_is_start, step_low, step_start, timed_assist};
        assert_eq!(step_start("cormoho spot_28 start"), ("cormoho spot_28".to_string(), true));
        assert_eq!(step_start("armsolar"), ("armsolar".to_string(), false));
        assert!(step_is_start("armfus home start low #f1"));
        assert!(!step_is_start("armfus home low #f1"));
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

    /// The id, `low` and `start` are written in any order after the building's name and stored in one
    /// (econ-flagship-8: `armafus #af1 start` refused a turn's lists; 1794105281-human: `solar #s3 low`).
    #[test]
    fn a_steps_id_low_and_start_are_written_in_any_order_and_stored_in_one() {
        use super::step_canonical;
        for written in ["armafus #af1 start", "armafus start #af1", "armafus #af1 low start", "armafus low #af1 start"] {
            let stored = step_canonical(written).unwrap();
            let expected = if written.contains("low") { "armafus low start #af1" } else { "armafus start #af1" };
            assert_eq!(stored, expected, "{written}");
            assert!(super::step_is_start(&stored) && step_canonical(&stored).unwrap() == stored, "{written}");
        }
        assert_eq!(step_canonical("armvp #vp2 home").unwrap(), "armvp home #vp2");
        assert_eq!(step_canonical("solar #s3 low").unwrap(), "solar low #s3");
        assert_eq!(step_canonical("assist 90 low").unwrap(), "assist 90 low");
        assert_eq!(step_canonical("extractor spot_5").unwrap(), "extractor spot_5");
        assert!(step_canonical("#vp1 armvp").unwrap_err().contains("does not begin the step"));
        assert!(step_canonical("armvp #a #b").unwrap_err().contains("one id a step"));
        assert!(step_canonical("armvp #").unwrap_err().contains("`#` and a name"));
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
        let unit = |id: i32, def: i32, being_built: bool| OwnUnit { id: UnitId(id), def: UnitDefId(def), pos: Vec3::default(), vel: Vec3::default(), health: 1.0, max_health: 1.0, being_built, idle: false, reload_frame: 0, facing: 0, stockpile: 0, stockpile_queued: 0, priority_low: false, build_progress: if being_built { 0.5 } else { 1.0 }, stunned: 0.0 };
        let (plant, solar) = (UnitDefId(7), UnitDefId(8));
        let (first, second, frame) = (UnitId(1), UnitId(2), UnitId(50));
        let mut pianist = super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut own = vec![unit(1, 3, false), unit(2, 3, false)];
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Free);
        // The first builder is ordered to build it: the id is its claim while it is on that build.
        pianist.tasks.insert(first, Task::Build { def: plant, near: Vec3::default(), spot: None, ordered: 100, started: false, frame: None, facing: 0 });
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
        // A builder finishing the id's frame (a `finish`, a Task::Build with the frame) keeps the frame as the id's
        // building: Center Command 2v4, 2026-10-19, the finish re-inserted the id with no building, and a builder
        // listed while being built reached the step 62 s after the plant stood and built a second one.
        pianist.tasks.insert(second, Task::Build { def: plant, near: Vec3::default(), spot: None, ordered: 120, started: true, frame: Some(frame), facing: 0 });
        assert_eq!(pianist.note_tag("avp1", second, &own, "finish our plant".into(), false, 120), "finish our plant (#avp1)");
        assert_eq!(pianist.tagged.get("avp1").and_then(|t| t.building), Some(frame));
        // It stands: every step with the id is done, also in a list sent again.
        own[2].being_built = false;
        pianist.tasks.remove(&first);
        pianist.tasks.remove(&second);
        assert_eq!(pianist.tag_state("avp1", plant, first, &own), TagState::Stands);
        let late = UnitId(3);
        assert_eq!(pianist.tag_state("avp1", plant, late, &own), TagState::Stands);
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

#[cfg(test)]
mod finish_tests {
    use bot_protocol::{Command, OwnUnit, UnitDefId, UnitId};
    use std::collections::VecDeque;

    use super::super::fixtures::{at, brain_of, own};
    use super::super::menu::Order;
    use super::super::picture::Picture;
    use super::super::{Pianist, Task};
    use crate::brain::roster::Kit;

    /// `finish <handle>` on a list resumes an unfinished frame of ours, whoever began it, as the builder's own
    /// build: a started build task on that frame (the finish-first law and the 60 % rule hold) and the engine's
    /// repair command; a handle that stands finished, or one nothing bears, is skipped and said.
    #[test]
    fn a_finish_step_resumes_an_unfinished_frame_as_the_builders_own_build() {
        let mut brain = brain_of(&["armcv", "armsolar"]);
        let ours = vec![own(1, 1, at(1000.0, 950.0)), OwnUnit { being_built: true, health: 62.0, max_health: 100.0, build_progress: 0.62, ..own(20, 2, at(1000.0, 1000.0)) }, own(21, 2, at(1200.0, 1000.0))];
        let id = UnitDefId(9);
        let kit = Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, vehicle_constructor: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &crate::brain::roster::ROSTERS[0] };
        let picture = Picture { rules: String::new(), state: serde_json::json!({}), places: Vec::new(), parties: Vec::new() };
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let name = "constructor_1".to_string();
        pianist.scripts.insert(name.clone(), VecDeque::from(["finish armsolar_21".to_string(), "finish armsolar_99".to_string(), "finish".to_string(), "finish armsolar_20".to_string()]));
        let step = brain.next_list_step(&ours[0], &name, &mut pianist, &picture, &ours, 100, &kit, &mut Vec::new(), &mut Vec::new());
        assert!(matches!(&step, Some((Order::Finish(UnitId(20)), s)) if s == "finish armsolar_20"), "{step:?}");
        assert!(pianist.done.iter().any(|d| d.contains("skipped the step 'finish armsolar_21'") && d.contains("stands finished already")), "{:?}", pianist.done);
        assert!(pianist.done.iter().any(|d| d.contains("skipped the step 'finish armsolar_99'") && d.contains("the frame is gone")), "{:?}", pianist.done);
        assert!(pianist.done.iter().any(|d| d.contains("skipped the step 'finish'") && d.contains("finish takes the handle")), "{:?}", pianist.done);
        // Executed: a repair command on the frame, and a started build task that keeps the frame's id.
        brain.pianist = Some(pianist);
        let resource = || bot_protocol::Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = bot_protocol::Tick { frame: 100, late: 0, events: Vec::new(), snapshot: bot_protocol::Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.clone(), allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None, conversion: 0.0 } };
        let mut commands = Vec::new();
        let did = brain.execute_builder(&tick, &kit, &picture, UnitId(1), &Order::Finish(UnitId(20)), false, Some("finish armsolar_20"), &mut commands);
        assert!(did.as_deref().is_some_and(|d| d.starts_with("finish our Solar Collector")), "{did:?}");
        assert!(commands.iter().any(|c| matches!(c, Command::Repair { unit: UnitId(1), target: UnitId(20), .. })), "{commands:?}");
        let task = brain.pianist.as_ref().unwrap().tasks.get(&UnitId(1)).cloned();
        assert!(matches!(task, Some(Task::Build { def: UnitDefId(2), started: true, frame: Some(UnitId(20)), .. })), "{task:?}");
        // The same finish again is the build going on, not a new order.
        let again = brain.execute_builder(&tick, &kit, &picture, UnitId(1), &Order::Finish(UnitId(20)), false, None, &mut commands);
        assert!(again.is_none(), "{again:?}");
    }

    /// The lists line counts the builders whose lists ran out with nothing in their place, with the median and the
    /// longest, and the lists still running; a builder whose list ended and got a new one is running again
    /// (H-WAKE-LISTS-ENDED: econ-flagship-13's last three minutes, 26 to 31 of 40 builders on the fallback unsaid).
    #[test]
    fn the_lists_line_counts_the_ended_lists_and_how_long_ago() {
        let mut brain = brain_of(&["armcv", "armsolar"]);
        // The fixture's units carry no build speed: the constructor is made a mobile builder (90, builds the solar).
        if let Some(d) = brain.world.hello.unit_defs.iter_mut().find(|d| d.id == UnitDefId(1)) {
            d.build_speed = 90.0;
            d.build_options.push(UnitDefId(2));
        }
        let ours = vec![own(1, 1, at(1000.0, 950.0)), own(2, 1, at(1100.0, 950.0)), own(3, 1, at(1200.0, 950.0))];
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        brain.pianist = Some(pianist);
        assert!(brain.lists_line(&ours, 3000).is_none(), "nothing ended, no line");
        pianist = brain.pianist.take().unwrap();
        pianist.scripts.insert("constructor_1".into(), VecDeque::new());
        pianist.list_ended.insert("constructor_1".into(), 3000 - 71 * 30);
        pianist.scripts.insert("constructor_2".into(), VecDeque::new());
        pianist.list_ended.insert("constructor_2".into(), 3000 - 150 * 30);
        pianist.scripts.insert("constructor_3".into(), VecDeque::from(["armsolar start".to_string()]));
        brain.pianist = Some(pianist);
        let line = brain.lists_line(&ours, 3000).expect("a line");
        assert_eq!(line, "lists: 2 builders' lists have ended with nothing in their place (median 150 s ago, the longest 150 s), on the hands' fallback meanwhile; 1 list running");
        // A new list on constructor_2 (the queue tool clears its record): one ended, two running.
        let mut pianist = brain.pianist.take().unwrap();
        pianist.list_ended.remove("constructor_2");
        pianist.scripts.insert("constructor_2".into(), VecDeque::from(["armsolar".to_string()]));
        brain.pianist = Some(pianist);
        assert!(brain.lists_line(&ours, 3000).unwrap().starts_with("lists: 1 builder's list has ended with nothing in its place (median 71 s ago, the longest 71 s), on the hands' fallback meanwhile; 2 lists running"));
    }

    /// A builder leaving a frame on a `start` step takes the next step of the same type as the next building, not
    /// as the build already under way: ten constructors' `armsolar:8 start` built one solar each (econ-flagship-12,
    /// 10:49, the seven repeats "already under way: taken as done"). Without `start` the same-type step is the build
    /// going on, as before.
    #[test]
    fn a_start_steps_repeat_is_the_next_frame_not_the_one_being_left() {
        use super::super::menu::Site;
        let mut brain = brain_of(&["armcv", "armsolar"]);
        // The fixture's constructor builds the solar (the site search refuses a type the builder cannot make).
        if let Some(d) = brain.world.hello.unit_defs.iter_mut().find(|d| d.id == UnitDefId(1)) {
            d.build_options.push(UnitDefId(2));
        }
        let frame = OwnUnit { being_built: true, health: 10.0, max_health: 100.0, build_progress: 0.1, ..own(20, 2, at(1000.0, 1000.0)) };
        let ours = vec![own(1, 1, at(1000.0, 950.0)), frame];
        let id = UnitDefId(9);
        let kit = Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, vehicle_constructor: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &crate::brain::roster::ROSTERS[0] };
        let picture = Picture { rules: String::new(), state: serde_json::json!({}), places: Vec::new(), parties: Vec::new() };
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.tasks.insert(UnitId(1), Task::Build { def: UnitDefId(2), near: at(1000.0, 1000.0), spot: None, ordered: 50, started: true, frame: Some(UnitId(20)), facing: 0 });
        brain.pianist = Some(pianist);
        brain.home = at(1000.0, 1000.0);
        brain.kit = Some(kit.clone());
        let resource = || bot_protocol::Resource { current: 1000.0, income: 10.0, usage: 0.0, storage: 2000.0 };
        let tick = bot_protocol::Tick { frame: 100, late: 0, events: Vec::new(), snapshot: bot_protocol::Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.clone(), allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None, conversion: 0.0 } };
        let mut commands = Vec::new();
        // Without `start`: the same type is the build under way.
        let same = brain.execute_builder(&tick, &kit, &picture, UnitId(1), &Order::Build(UnitDefId(2), Site::Planned), false, Some("armsolar"), &mut commands);
        assert!(same.is_none(), "{same:?}");
        assert!(commands.is_empty());
        // With `start`: a new frame is ordered and becomes the task.
        let next = brain.execute_builder(&tick, &kit, &picture, UnitId(1), &Order::Build(UnitDefId(2), Site::Planned), false, Some("armsolar start"), &mut commands);
        assert!(next.as_deref().is_some_and(|d| d.starts_with("build a")), "{next:?}");
        assert!(commands.iter().any(|c| matches!(c, Command::Build { unit: UnitId(1), def: UnitDefId(2), .. })), "{commands:?}");
        let task = brain.pianist.as_ref().unwrap().tasks.get(&UnitId(1)).cloned();
        assert!(matches!(task, Some(Task::Build { def: UnitDefId(2), started: false, frame: None, .. })), "{task:?}");
    }
}

#[cfg(test)]
mod count_tests {
    use super::{compress_steps, step_count};

    #[test]
    fn a_count_on_the_first_word_repeats_the_step_and_reads_back_the_same() {
        assert_eq!(step_count("extractor:20"), Ok(("extractor".to_string(), 20)));
        assert_eq!(step_count("corwin:6 low"), Ok(("corwin low".to_string(), 6)));
        assert_eq!(step_count("corllt:2 spot_3"), Ok(("corllt spot_3".to_string(), 2)));
        assert_eq!(step_count("armsolar low #s3"), Ok(("armsolar low #s3".to_string(), 1)));
        assert!(step_count("extractor:0").is_err());
        assert!(step_count("extractor:61").is_err());
        assert!(step_count("extractor:many").is_err());
        let steps: Vec<String> = ["extractor", "extractor", "extractor", "corwin low", "corwin low", "armvp #vp1", "armvp #vp1", "assist"].iter().map(|s| s.to_string()).collect();
        assert_eq!(compress_steps(steps.iter()), "extractor:3, corwin:2 low, armvp #vp1, armvp #vp1, assist");
    }
}
