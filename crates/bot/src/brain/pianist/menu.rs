//! The menus: for every actor that is free or due, one Choice of next actions (always with a no-change option) and,
//! speculatively, the place and the party an action may need; and the global questions. Jev cannot choose what is
//! not offered, so the options are the whole vocabulary of the hands.

use std::collections::BTreeMap;

use bot_protocol::{OwnUnit, Tick, UnitDefId, UnitId, Vec3};
use jev::Question;
use serde_json::{Value, json};

use super::super::roster::Kit;
use super::Pianist;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Party, Picture, distance_words};
use crate::strategist::shared::Allowance;
use super::{REVIEW_FRAMES, Task};

/// Free spots on a builder's menu, nearest by its own walking first.
const NEAR_SPOTS: usize = 6;
/// A holding group is asked this often; a busy one every `REVIEW_FRAMES`.
const HOLD_REVIEW_FRAMES: i32 = 5 * FRAMES_PER_SECOND;
/// A lab that answered "nothing" is not asked again for this long.
const LAB_REVIEW_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// H-HANDS-PARTY-KILLING: how near a party shooting something of ours must be for the hold option to say that
/// holding leaves it to die.
const KILLER_REACH: f32 = 1200.0;
/// Wrecks and things to repair within this of a builder are offered.
const RECLAIM_WITHIN: f32 = 1800.0;
const REPAIR_WITHIN: f32 = 1200.0;
/// A builder farther than this from home is offered the way home.
const AWAY: f32 = 400.0;
/// An enemy party this close to a group is news that gets it asked at once.
const ALARM: f32 = 600.0;
/// A builder on a started build is asked again only with an enemy party this close (H-HANDS-STARTED).
const STARTED_ALARM: f32 = 800.0;
/// A builder whose started build is this far along is asked what comes next, and the answer is queued behind it
/// (H-HANDS-QUEUE: human-5, a generator every 8.5 s against Matt's 6, the difference an idle pick and a walk each).
const QUEUE_AT: f32 = 0.6;
/// A builder is offered the attack on a party only this close: raiders outrun a commander, and one sent after a party
/// 500 away walked after it for a minute instead of helping the lab (human-6).
const ATTACK_REACH: f32 = 320.0;
/// A party this close is offered too when it is busy: standing at a building of ours, or fought by a group of ours,
/// which the commander's guns then join (human-8, the user: it did not use its commander to improve the exchange
/// rate; the Grunts died 380 to 525 from it).
const ATTACK_JOIN: f32 = 500.0;
/// A group this small is not offered a detachment (pianist-player-14: groups of one sent one soldier at a time).
const DETACH_FROM: usize = 4;
/// A party bigger than this is an attack, not a raider to be met by a detachment (human-1: two soldiers sent against
/// eight Pawns four times, each pair back in the group a second later).
pub(super) const DETACH_PARTY_MAX: usize = 3;

#[derive(Clone, Debug)]
pub(crate) enum Pick {
    Continue,
    Wait,
    /// An extractor at the free spot answered in `where`, else the nearest of the menu's `spots`.
    Extractor,
    /// A building placed as the base layout has it (`place_planned`).
    Building(UnitDefId),
    /// A building at the place answered in `where`.
    BuildingAt(UnitDefId),
    AssistLab(UnitId),
    Reclaim(Vec3),
    /// Take one unit of ours apart (a `reclaim <handle>` list step from the `remove` tool).
    ReclaimUnit(UnitId),
    Repair(UnitId),
    WalkTo,
    RetreatHome,
    /// Attack the nearest enemy party (a builder that outweighs it; the commander against raiders).
    Attack(Vec3, String),
    /// Labs.
    Unit(UnitDefId),
    Nothing,
    /// Groups.
    Hold,
    MoveTo { fight: bool },
    Engage,
    /// Every member attacks one unit of the party in `whom`: its commander, else its dearest (the domains design).
    AttackUnit,
    Split,
    /// `how_many` soldiers nearest the party in `whom` go and attack it as a new group; the rest carry on.
    Detach,
    /// One soldier, a raider if there is one, walks to `where` and stands there.
    Scout,
    Join(String),
    Retreat,
}

#[derive(Clone, Debug)]
pub(crate) enum Actor {
    Builder(UnitId),
    Lab(UnitId),
    Group(String),
    Global,
}

pub(crate) struct Menu {
    pub actor: Actor,
    /// The actor's name in the picture; question ids are `<name>.<what>`.
    pub name: String,
    pub busy: bool,
    /// A builder asked what to do after the build in progress: the answer is ordered behind it, not instead of it.
    pub queue_ahead: bool,
    pub questions: Vec<(String, Question)>,
    pub options: BTreeMap<String, Pick>,
    /// A builder's free spots, nearest by its own walking first.
    pub spots: Vec<usize>,
    /// A step from the player's list (H-HANDS-SCRIPT): the option to play without asking, and its place, if any.
    pub scripted: Option<(String, Option<String>)>,
    /// Ordered by the player's policy (`policy.rs`), not by Jev.
    pub policy: bool,
}

impl Brain {
    /// The menus for this second. Updates when each actor was last asked.
    pub(super) fn menus(&mut self, tick: &Tick, kit: &Kit, picture: &Picture) -> Vec<Menu> {
        let own = &tick.snapshot.own_units;
        let frame = tick.frame;
        let Some(mut pianist) = self.pianist.take() else { return Vec::new() };
        let mut menus: Vec<Menu> = Vec::new();
        let place_names: Vec<(String, String)> = picture
            .places
            .iter()
            .map(|p| (p.name.clone(), format!("{} ({})", picture.state["places"][&p.name]["what"].as_str().unwrap_or_default(), self.world.grid(p.at))))
            .collect();
        // One place question per kind of action, each with its premise stated: a single "where, if the action needs a
        // place, else home" was answered "home" nearly every time, since it cannot see which action was chosen
        // (pianist-smoke-5: scouts sent home, advances to home, forty one-unit groups).
        let where_question = |premise: &str, spots: &[(usize, f32)], only_spots: bool| {
            let criteria: BTreeMap<String, Value> = place_names
                .iter()
                .filter(|(n, _)| !only_spots || spots.iter().any(|(i, _)| format!("spot_{i}") == *n))
                .map(|(n, d)| {
                    let walk = spots.iter().find(|(i, _)| format!("spot_{i}") == *n).map_or(String::new(), |(_, s)| format!("; {s:.0} s of walking for this builder"));
                    (n.clone(), json!(format!("{d}{walk}")))
                })
                .collect();
            Question::Choice { instructions: json!(premise.to_string()), criteria }
        };
        let energy_words = picture.state["economy"]["energy"].as_str().unwrap_or_default().to_string();
        let metal_words = picture.state["economy"]["metal"].as_str().unwrap_or_default().to_string();

        // Builders.
        let under_fire: Vec<UnitId> = tick.events.iter().filter_map(|e| if let bot_protocol::Event::UnitDamaged { unit, .. } = e { Some(*unit) } else { None }).collect();
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            let task = pianist.tasks.get(&unit.id).cloned();
            let last = pianist.last_asked.get(&name).copied().unwrap_or(i32::MIN / 2);
            // H-HANDS-STARTED: a build the engine has started (a nanoframe stands) is not put to the question until it
            // is done, unless the builder is under fire or an enemy party is within reach; a builder re-asked every ten
            // seconds walked away from two labs and fourteen generators in pianist-player-1, and each frame decayed.
            // The task's own flag says the build is started (set on the engine's created event; the task is dropped
            // when the frame dies); the frame is looked up only for its progress words. Re-finding it by position was
            // what let pianist-player-13's commander be asked again over a started extractor and switch to a windmill.
            let started = match &task {
                Some(Task::Build { def, near, started: true, .. }) => {
                    let share = own.iter().filter(|u| u.being_built && u.def == *def).map(|u| (u.pos.dist2d(*near), u.health / u.max_health.max(1.0))).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(0.0, |(_, share)| share);
                    Some((self.short_words(*def), share))
                }
                _ => None,
            };
            let threatened = under_fire.contains(&unit.id) || picture.parties.iter().any(|p| p.at.dist2d(unit.pos) < STARTED_ALARM);
            let queue_ahead = started.as_ref().is_some_and(|(_, share)| *share >= QUEUE_AT) && !threatened && !pianist.queued.contains_key(&unit.id);
            let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
            // H-HANDS-SCRIPT: a builder with a list from the player's `queue` tool is not asked; the bot orders the
            // next step when the builder is free, or when the build in progress is 60% done (queued behind it), and
            // skips a step it cannot do. An enemy on the builder puts it back on the menu until that is over.
            if !threatened && pianist.scripts.get(&name).is_some_and(|steps| !steps.is_empty()) {
                // The player's list outranks the bot's fillers: a builder helping a factory, walking, reclaiming or
                // repairing takes its next step at once (plan-1: the commander helped a plant from 3:46 to 14:24 while
                // four lists waited, since a guard order never ends). A build in progress keeps the 60 % rule.
                let list_frame = pianist.script_frame.get(&name).copied().unwrap_or(0);
                let ready = match &task {
                    None => unit.idle,
                    Some(Task::Build { started: true, .. }) => queue_ahead,
                    // A build the engine has not started yet gives way to a list set after it was ordered
                    // (escalate-7: two constructors held a refused nano site through three new lists).
                    Some(Task::Build { ordered, .. }) => queue_ahead || list_frame > *ordered,
                    Some(_) => true,
                };
                if ready && let Some(menu) = self.scripted_step(unit, &name, &mut pianist, picture, own, frame, queue_ahead, task.is_some(), kit) {
                    pianist.last_asked.insert(name.clone(), frame);
                    menus.push(menu);
                }
                continue;
            }
            let (free, due) = match &task {
                None => (unit.idle || frame - last >= LAB_REVIEW_FRAMES, true),
                Some(_) if started.is_some() && !threatened => (false, queue_ahead && frame - last >= LAB_REVIEW_FRAMES),
                Some(task) => (false, frame - last >= REVIEW_FRAMES && frame - task.since() >= LAB_REVIEW_FRAMES),
            };
            if !(free || due) {
                continue;
            }
            // Busy for the switch margin only when the task was set under the current packet (H-HANDS-SWITCH).
            let busy = task.as_ref().is_some_and(|t| t.since() >= pianist.packet_frame);
            let mut options: BTreeMap<String, Pick> = BTreeMap::new();
            let mut criteria: BTreeMap<String, Value> = BTreeMap::new();
            let mut offer = |key: &str, pick: Pick, words: String| {
                options.insert(key.to_string(), pick);
                criteria.insert(key.to_string(), json!(words));
            };
            if let Some((what, share)) = &started
                && queue_ahead
            {
                offer("continue", Pick::Continue, format!("Queue nothing: finish the {what} ({:.0}% built) and be asked again when it is done.", share * 100.0));
            } else if let Some((what, share)) = &started {
                offer("continue", Pick::Continue, format!("Finish the {what} it has started here ({:.0}% built). Leaving it now wastes the metal already put in; the frame decays.", share * 100.0));
            } else if busy {
                offer("continue", Pick::Continue, "Carry on with what it is doing now.".into());
            } else {
                offer("wait", Pick::Wait, "Do nothing for now (only when nothing on this list is worth doing).".into());
            }
            let mut spots: Vec<(usize, f32)> = Vec::new();
            if can(kit.extractor) {
                spots = self.free_spots(unit, &pianist, picture, own, frame, kit);
                spots.truncate(NEAR_SPOTS);
                if !spots.is_empty() {
                    // One option, the spot in `where`: six spot options split the vote and lost to any single
                    // alternative (pianist-smoke-2: constructors helped the lab on four extractors).
                    let list: Vec<String> = spots
                        .iter()
                        .map(|(i, s)| {
                            let place = &picture.state["places"][format!("spot_{i}")];
                            format!(
                                "spot_{i} ({:.0} s of walking, ground {}{})",
                                s, place["ground"].as_str().unwrap_or_default(),
                                place["enemies_near"].as_str().map_or(String::new(), |e| format!(", enemies near: {e}"))
                            )
                        })
                        .collect();
                    offer("extractor", Pick::Extractor, format!("Build a metal extractor (income) at the free spot answered in `where`; the nearest free spots for it: {}.", list.join(", ")));
                }
            }
            // Every building the player allows for this builder, else the faction's usual list, keyed by the game's
            // internal name and worded from the definition and the glossary (docs/design/2026-09-22-full-roster.md).
            // The whole build list goes into `options` for the policy, which may order anything the builder can
            // build; Jev is asked only what is offered, so the vote is not split over forty options.
            let build_list: Vec<UnitDefId> = self.world.def(unit.def).map(|d| d.build_options.clone()).unwrap_or_default();
            let allowed = self.allowed_units(&name);
            // A changed allowance restarts the builder's counts against its caps ("armllt:2": two more, then off).
            if pianist.allowed_seen.get(&name) != allowed.as_ref() {
                pianist.produced.retain(|(b, _), _| *b != unit.id);
                match &allowed {
                    Some(a) => pianist.allowed_seen.insert(name.clone(), a.clone()),
                    None => pianist.allowed_seen.remove(&name),
                };
            }
            let permits = |list: &[String], b: UnitDefId| {
                let unit_name = self.name(b).to_string();
                list.iter().map(|e| super::allowance(e)).any(|(n, cap)| n == unit_name && cap.is_none_or(|cap| pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0) < cap))
            };
            let usual = super::super::roster::usual_menu(self.name(unit.def));
            let offered: Vec<UnitDefId> = match &allowed {
                Some(Allowance { units: list, .. }) if build_list.iter().any(|b| permits(list, *b)) => build_list.iter().copied().filter(|b| permits(list, *b)).collect(),
                _ => build_list.iter().copied().filter(|b| usual.contains(&self.name(*b))).collect(),
            };
            let reach = self.world.def(unit.def).map_or(100.0, |d| d.build_distance) + 300.0;
            let factories = own.iter().filter(|u| self.world.is_factory_def(u.def)).count();
            let generators = format!(
                "We have {} solar collectors and {} wind generators standing or started.",
                own.iter().filter(|u| u.def == kit.solar || u.def == kit.advanced_solar).count(),
                own.iter().filter(|u| u.def == kit.wind).count()
            );
            let mut off_menu: Vec<(String, Pick)> = Vec::new();
            for def in &build_list {
                if *def == kit.extractor {
                    continue;
                }
                // A sea building only where there is water for it.
                if super::glossary::entry(self.name(*def)).is_some_and(|e| e.has_flag("on_water")) && !self.world.water_within(unit.pos, reach) {
                    continue;
                }
                let pick = if self.placed_at_place(*def) { Pick::BuildingAt(*def) } else { Pick::Building(*def) };
                if offered.contains(def) {
                    // A construction turret only beside a standing factory.
                    let nano = self.world.def(*def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty());
                    if nano && !own.iter().any(|u| self.world.is_factory_def(u.def) && !u.being_built) {
                        continue;
                    }
                    let words = self.building_words(unit, *def, tick, &energy_words, &metal_words, factories, &generators);
                    offer(self.name(*def), pick, words);
                } else {
                    off_menu.push((self.name(*def).to_string(), pick));
                }
            }
            // Helping the lab is offered on a queue-ahead ask too, ordered by the bot as the build finishes (human-6: a
            // commander kept building by the queue-ahead was never offered it and did not help the lab for two minutes).
            if let Some(lab) = own.iter().filter(|u| self.world.is_factory_def(u.def) && !u.being_built).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))) {
                offer("assist_lab", Pick::AssistLab(lab.id), if queue_ahead { "Then help the nearest factory (lab or plant) build: adds this builder's build power to whatever it makes, until told otherwise.".into() } else { "Help the nearest factory (lab or plant) build: adds this builder's build power to whatever it makes.".into() });
            }
            if let Some(field) = self.reclaim.fields.iter().filter(|f| f.metal >= 100.0 && f.at.dist2d(unit.pos) < RECLAIM_WITHIN).max_by(|a, b| a.metal.total_cmp(&b.metal)) {
                offer("reclaim", Pick::Reclaim(field.at), format!("Take apart the wrecks at {} ({:.0} metal lying there{}).", self.place_words(&picture.places, field.at), field.metal, if field.safe { "" } else { "; not safe ground" }));
            }
            let hurt = own
                .iter()
                .filter(|u| u.id != unit.id && !u.being_built && u.health < u.max_health * 0.7 && u.pos.dist2d(unit.pos) < REPAIR_WITHIN)
                .filter(|u| self.world.is_commander_def(u.def) || self.world.def(u.def).is_some_and(|d| d.speed == 0.0))
                .min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
            if let Some(hurt) = hurt {
                offer("repair", Pick::Repair(hurt.id), format!("Repair our {} at {} ({:.0}% health).", self.name(hurt.def), self.place_words(&picture.places, hurt.pos), hurt.health / hurt.max_health * 100.0));
            }
            offer("walk_to", Pick::WalkTo, "Walk to the place answered in `where` and wait there.".into());
            if unit.pos.dist2d(self.home) > AWAY {
                offer("retreat_home", Pick::RetreatHome, format!("Go home now ({} away, {:.0}); the way home avoids known threats.", distance_words(unit.pos.dist2d(self.home)), unit.pos.dist2d(self.home)));
            }
            // H-HANDS-COMMANDER-FIGHTS: a builder is offered the attack on a party beside it that it outweighs alone; the
            // commander's D-gun is the early answer to raiders (human-1, second game: the player ordered it in five
            // packets and nothing on the menu could do it while two Pawns razed the base).
            let busy_party = |p: &Party| {
                own.iter().any(|u| u.pos.dist2d(p.at) < 150.0 && self.world.def(u.def).is_some_and(|d| d.speed == 0.0))
                    || pianist.groups.iter().any(|g| matches!(&g.task, super::GroupTask::Engage { party, .. } if party.iter().any(|id| p.ids.contains(id))))
            };
            let armed = self.world.def(unit.def).is_some_and(|d| d.weapon_count > 0);
            if let Some(party) = picture.parties.iter().filter(|_| armed).filter(|p| p.at.dist2d(unit.pos) < ATTACK_REACH || (p.at.dist2d(unit.pos) < ATTACK_JOIN && busy_party(p))).min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos))) {
                let odds = self.odds_words(&[unit], party, tick.snapshot.enemies.as_slice());
                if odds.starts_with("we outweigh") {
                    let why = if party.at.dist2d(unit.pos) < ATTACK_REACH { "within reach" } else { "busy at our buildings or fighting our soldiers, so it can be caught" };
                    offer("attack", Pick::Attack(party.at, party.name.clone()), format!("Attack {} ({}, {:.0} away, {why}) now and come back to what it was doing: against this unit alone, {odds}; its guns beside our soldiers turn an even trade. Raiders outrun it: a party farther off is not offered.", party.name, party.composition, party.at.dist2d(unit.pos)));
                }
            }
            options.extend(off_menu);
            let instructions = json!(if let Some((what, share)) = started.as_ref().filter(|_| queue_ahead) {
                format!(
                    "The {what} {name} is building is {:.0}% done. Given `actors.{name}` and the player's `instructions`, what should it do the moment that is finished? The answer is ordered behind it now, so it starts without a pause. Energy now: {energy_words}. Metal now: {metal_words}.",
                    share * 100.0
                )
            } else {
                format!("Given `actors.{name}` and the player's `instructions`, what should {name} do next? Prefer what the instructions say; keep to the plan unless the situation has changed. Energy now: {energy_words}. Metal now: {metal_words}.")
            });
            let mut questions = vec![
                (format!("{name}.do"), Question::Choice { instructions, criteria }),
                (format!("{name}.where"), where_question(&format!("Suppose {name} builds something that stands at a place (a defence, a radar, a tier-2 extractor over a spot), or walks somewhere: at which place? Choose the place the instructions and the situation call for."), &spots, false)),
            ];
            if !spots.is_empty() {
                questions.push((format!("{name}.where_extractor"), where_question(&format!("Suppose {name} builds a metal extractor next: at which of these free spots? Nearer is sooner; ground held by us is safer; enemies near a spot get the builder killed."), &spots, true)));
            }
            pianist.last_asked.insert(name.clone(), frame);
            menus.push(Menu {
                actor: Actor::Builder(unit.id),
                questions,
                name,
                busy,
                queue_ahead,
                options,
                spots: spots.iter().map(|(i, _)| *i).collect(),
                scripted: None, policy: false,
            });
        }

        // Factories: labs and plants.
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_factory_def(u.def)) {
            let name = self.actor_name(unit.id);
            let queued = pianist.lab_queue.get(&unit.id).map_or(0, Vec::len);
            let last = pianist.last_asked.get(&name).copied().unwrap_or(i32::MIN / 2);
            // One order waiting at most (H-HANDS-MENU): with two, three constructors were ordered in six seconds before
            // the first stood, and the player's whitelist of 2:13 waited behind them until 3:00 (human-3).
            if queued >= 1 || frame - last < LAB_REVIEW_FRAMES {
                continue;
            }
            let Some(def) = self.world.def(unit.def) else { continue };
            let extractors = own.iter().filter(|u| !u.being_built && self.world.is_extractor_def(u.def)).count();
            let constructors = own.iter().filter(|u| !u.being_built && self.world.is_constructor_def(u.def)).count();
            let constructors_coming = own.iter().filter(|u| u.being_built && self.world.is_constructor_def(u.def)).count() + queued;
            let coming_words = if constructors_coming > 0 { format!(" and {constructors_coming} more being made") } else { String::new() };
            let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
            let army_metal: f32 = soldiers.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            let mut options: BTreeMap<String, Pick> = BTreeMap::new();
            let mut criteria: BTreeMap<String, Value> = BTreeMap::new();
            options.insert("nothing".into(), Pick::Nothing);
            criteria.insert("nothing".into(), json!("Build nothing now and save the metal (only when metal is short: a unit chosen here waits behind the one being made, so choosing it never interrupts that unit; human-8: the lab chose nothing seven times while its constructor was being made)."));
            // H-HANDS-PRODUCE: the player's whitelist, when it names something this lab can build, is the whole menu.
            let allowed = self.allowed_units(&name);
            // A changed allowance restarts the lab's counts against its caps ("corck:1": one more, then off the list).
            if pianist.allowed_seen.get(&name) != allowed.as_ref() {
                pianist.produced.retain(|(lab, _), _| *lab != unit.id);
                match &allowed {
                    Some(a) => pianist.allowed_seen.insert(name.clone(), a.clone()),
                    None => pianist.allowed_seen.remove(&name),
                };
            }
            let permits = |list: &[String], b: UnitDefId| {
                let unit_name = self.name(b).to_string();
                list.iter().map(|e| super::allowance(e)).any(|(n, cap)| n == unit_name && cap.is_none_or(|cap| pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0) < cap))
            };
            let buildables: Vec<UnitDefId> = match &allowed {
                Some(Allowance { units: list, .. }) if def.build_options.iter().any(|b| permits(list, *b)) => def.build_options.iter().copied().filter(|b| permits(list, *b)).collect(),
                _ => def.build_options.clone(),
            };
            for buildable in &buildables {
                let key = self.name(*buildable).to_string();
                options.insert(key.clone(), Pick::Unit(*buildable));
                // The count in words beside the option: Jev does not count what it has against a plan
                // (pianist-smoke-1: thirty constructors and no soldier by minute nine).
                let have = if self.world.is_constructor_def(*buildable) {
                    format!(" We have {constructors} constructors{coming_words} for {extractors} extractors.")
                } else if self.world.def(*buildable).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0) {
                    format!(" Our soldiers: {}.", super::picture::soldier_words(soldiers.len(), army_metal))
                } else {
                    String::new()
                };
                criteria.insert(key, json!(format!("Build a {}.{have}", self.unit_words(*buildable))));
            }
            let instructions = json!(format!(
                "Given `actors.{name}`, `ours`, `economy` and the player's `instructions`, which unit should {name} build next? We have {constructors} constructors{coming_words} for {extractors} extractors and {} soldiers ({}).{}",
                soldiers.len(), super::picture::soldier_words(soldiers.len(), army_metal),
                if allowed.is_some() && buildables.len() < def.build_options.len() { " The player allows only the units offered here." } else { "" }
            ));
            pianist.last_asked.insert(name.clone(), frame);
            menus.push(Menu { actor: Actor::Lab(unit.id), questions: vec![(format!("{name}.next"), Question::Choice { instructions, criteria })], name, busy: queued > 0, queue_ahead: false, options, spots: Vec::new(), scripted: None, policy: false });
        }

        // Groups.
        let group_names: Vec<(String, Option<Vec3>, crate::world::Domain)> = pianist.groups.iter().map(|g| (g.name.clone(), super::groups::centre_of(&g.units(own)), g.domain)).collect();
        let scout_out = pianist.groups.iter().any(|g| g.members.len() == 1 && matches!(g.task, super::GroupTask::Move { fight: false, .. }));
        // H-HANDS-DETACH: a party some group of ours already engages is not offered for a detachment; asked every
        // five seconds, a group sent four soldiers after the same Flash squad eleven times in forty seconds and the
        // detachments sent detachments (pianist-player-14: "we are feeding it four soldiers at a time").
        let engaged: Vec<UnitId> = pianist.groups.iter().filter_map(|g| if let super::GroupTask::Engage { party, .. } = &g.task { Some(party.clone()) } else { None }).flatten().collect();
        for group in &mut pianist.groups {
            let name = format!("group_{}", group.name);
            let units = group.units(own);
            let Some(centre) = super::groups::centre_of(&units) else { continue };
            let last = pianist.last_asked.get(&name).copied().unwrap_or(i32::MIN / 2);
            let enemies_near = picture.parties.iter().any(|p| p.at.dist2d(centre) < ALARM);
            let alarm = enemies_near && !group.enemies_near;
            group.enemies_near = enemies_near;
            let review = if group.task.busy() { REVIEW_FRAMES } else { HOLD_REVIEW_FRAMES };
            if !(alarm || frame - last >= review) {
                continue;
            }
            // Busy for the switch margin only when the task was set under the current packet (H-HANDS-SWITCH).
            let busy = group.task.busy() && group.task.since() >= pianist.packet_frame;
            let mut options: BTreeMap<String, Pick> = BTreeMap::new();
            let mut criteria: BTreeMap<String, Value> = BTreeMap::new();
            let mut offer = |key: &str, pick: Pick, words: String| {
                options.insert(key.to_string(), pick);
                criteria.insert(key.to_string(), json!(words));
            };
            if busy {
                offer("continue", Pick::Continue, "Carry on with what it is doing.".into());
            }
            let enemies = tick.snapshot.enemies.as_slice();
            // A party with their commander in it says so in plain words, and every party says how far it is from this
            // group (plan-1: offered "3 unidentified at E4" and "1 armcom at H2", Jev sent nine Bulls 3,000 away at the
            // first while the commander stood 447 away).
            // H-HANDS-PARTY-KILLING: a party shooting something of ours says so on its line, and the hold option says
            // what holding leaves to die (the replay: the cost on the hold option's own words moved the hands from 34
            // to 52 % right beside a base under attack, the party line alone hardly at all).
            let party_words = |p: &Party| format!("{} at {}, {} from this group ({:.0} away){}: {}", if p.has_commander { format!("THEIR COMMANDER, the unit whose death wins the game ({})", p.composition) } else { p.composition.clone() }, self.place_words(&picture.places, p.at), distance_words(p.at.dist2d(centre)), p.at.dist2d(centre), p.killing.as_ref().map_or(String::new(), |(what, metal)| format!(", killing {what} ({metal:.0} metal) now")), self.odds_words(&units, p, enemies));
            let parties_words: Vec<String> = picture.parties.iter().map(|p| format!("{}: {}", p.name, party_words(p))).collect();
            // Only a killer within reach of an answer (hands-2: the sentence named a raider 5,000 away killing one
            // Blitz, and the group was asked to weigh that; within 1,200 it fought 63 asks of 85, beyond it held).
            let killers: Vec<&str> = picture.parties.iter().filter(|p| p.killing.is_some() && p.at.dist2d(centre) <= KILLER_REACH).map(|p| p.name.as_str()).collect();
            let hold_words = match killers.as_slice() {
                [] => "Stand where it is; fight whatever mobile comes within reach and step out of turret reach. Nothing beyond reach is protected by this.".to_string(),
                names => format!("Stand where it is; fight whatever mobile comes within reach and step out of turret reach. Nothing beyond reach is protected by this. Holding now leaves what {} is killing to die.", names.join(" and ")),
            };
            offer("hold", Pick::Hold, hold_words);
            offer("move_to", Pick::MoveTo { fight: false }, "Walk to the place in `where` without stopping to fight on the way (it runs from everything).".into());
            offer("fight_to", Pick::MoveTo { fight: true }, "Advance to the place in `where`, arriving together and fighting everything on the way and there, turrets included: it does not stop at a turret's reach, so it is the attack. What is known to stand at a place is in its entry in the picture; nothing here weighs it.".into());
            if !picture.parties.is_empty() {
                offer("engage", Pick::Engage, format!("Attack the enemy party named in `whom` now and follow it. In sight: {}.", parties_words.join("; ")));
                offer("attack_unit", Pick::AttackUnit, "Every soldier of this group attacks one unit of the party named in `whom`: its commander when it is there, else its dearest unit; they chase it until it dies or is lost, then hold. The order that kills a commander, and the only order by which aircraft pick their target.".into());
            }
            offer("retreat", Pick::Retreat, "Fall back to our base.".into());
            if units.len() >= 2 {
                offer("split", Pick::Split, "Send a detachment, the number in `how_many` of the nearest soldiers, to advance to the place in `where`; the rest carry on as they were.".into());
                let unengaged = picture.parties.iter().any(|p| p.ids.len() <= DETACH_PARTY_MAX && !p.ids.iter().any(|id| engaged.contains(id)));
                if unengaged && units.len() >= DETACH_FROM {
                    // H-HANDS-DETACH: a raider at a structure is met by a few soldiers, not the ball (realtime-2: 11 of
                    // 18 engagements were the whole ball after one Fav, Stump or Beaver, while a Fav killed a lab at home).
                    offer("send_against", Pick::Detach, "Send a detachment, the number in `how_many` of the soldiers nearest to the enemy party named in `whom`, to attack it and follow it; the rest carry on as they were. The answer to a raider at one of our extractors while this group stays: one soldier catches a single Tick or scout car, a few catch a small party, the whole group chasing one does not.".into());
                }
                // One scout out at a time (smoke-6: a raider every ten seconds to the enemy base, five dead by 5:00).
                if !scout_out {
                    offer("scout", Pick::Scout, "Send one soldier (a raider if the group has one) to look at the place in `where_scout` and stand there watching; the rest carry on. This is how the enemy base and its army get seen.".into());
                }
            }
            if let Some((other, _, _)) = group_names.iter().filter(|(n, c, d)| *n != group.name && c.is_some() && *d == group.domain).min_by(|a, b| a.1.unwrap().dist2d(centre).total_cmp(&b.1.unwrap().dist2d(centre))) {
                offer(&format!("join_group_{other}"), Pick::Join(other.clone()), format!("Merge into group_{other} and take its task."));
            }
            let instructions = json!(format!("Given `actors.{name}`, `enemy` and the player's `instructions`, what should {name} do next?"));
            let mut questions = vec![
                (format!("{name}.do"), Question::Choice { instructions, criteria }),
                (format!("{name}.where"), where_question(&format!("Suppose {name} advances, moves or sends a detachment: to which place? Choose where the instructions and the situation call for it to stand or fight."), &[], false)),
                (format!("{name}.where_scout"), where_question(&format!("Suppose {name} sends one soldier to look at a place: which place needs looking at? The enemy base if it is not found or not seen lately, else the spots we know least about."), &[], false)),
                (format!("{name}.how_many"), Question::choice(format!("If {name} sends a detachment, how many soldiers go?"), [("1", "one: enough for a single scout car or Tick"), ("2", "two"), ("4", "four"), ("8", "eight"), ("half", "half of the group")])),
            ];
            if !picture.parties.is_empty() {
                let criteria: BTreeMap<String, Value> = picture.parties.iter().map(|p| (p.name.clone(), json!(party_words(p)))).collect();
                questions.push((format!("{name}.whom"), Question::Choice { instructions: json!(format!("If {name} attacks an enemy party, which one?")), criteria }));
            }
            pianist.last_asked.insert(name.clone(), frame);
            menus.push(Menu { actor: Actor::Group(group.name.clone()), name, busy, queue_ahead: false, questions, options, spots: Vec::new(), scripted: None, policy: false });
        }

        // Global.
        if !menus.is_empty() {
            let mut questions = vec![
                ("global.base_in_danger".to_string(), Question::noul("Given `enemy` and `places`, is our base or our commander in danger right now?")),
                ("global.attack_coming".to_string(), Question::noul("Given `enemy`, is a large enemy attack on us likely within the next minute or two?")),
            ];
            if self.strategist.is_some() {
                questions.push(("global.needs_player".to_string(), Question::noul("Given everything, does the situation need the player's attention now: something the `instructions` do not cover, or a plan that has stopped fitting the game?")));
            }
            menus.push(Menu { actor: Actor::Global, name: "global".into(), busy: false, queue_ahead: false, questions, options: BTreeMap::new(), spots: Vec::new(), scripted: None, policy: false });
        }
        self.pianist = Some(pianist);
        menus
    }
}

impl Brain {
    /// What building `def` with this builder draws in energy a second against our income (human-3: the commander's
    /// lab drew 80 a second against 30 coming in, and the store was empty for the next minute and a half).
    /// The free spots this builder could take, nearest by its own walking first: not held, not another builder's
    /// task or queued spot (queue-ahead-smoke: two constructors queued spot_14 at once), not refused lately, and
    /// free in the picture's words.
    fn free_spots(&self, unit: &OwnUnit, pianist: &Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) -> Vec<(usize, f32)> {
        let taken: Vec<usize> = pianist
            .tasks
            .iter()
            .chain(pianist.queued.iter())
            .filter_map(|(id, t)| if let (true, Task::Build { spot: Some(i), .. }) = (*id != unit.id, t) { Some(*i) } else { None })
            .collect();
        let mut spots: Vec<(usize, f32)> = picture
            .places
            .iter()
            .filter_map(|p| p.spot.map(|i| (i, p.at)))
            .filter(|(i, _)| !taken.contains(i) && !pianist.refused_spots.get(i).is_some_and(|until| *until > frame))
            .filter(|(_, at)| !own.iter().any(|u| kit.is_extractor(u.def) && u.pos.dist2d(*at) < 100.0))
            // Free as far as we know: no extractor of theirs remembered on it, and not an ally's ground. A spot
            // nobody of ours has looked at counts as free (evidence-2-bulldogs: the words "never in our sight" failed
            // an earlier test on the words starting with "free", and every unscouted spot was refused for ten minutes).
            .filter(|(i, at)| !self.enemy_buildings.values().any(|(def, pos, _)| pos.dist2d(*at) < 100.0 && self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)) && self.spot_open_to_us(*i, *at, frame))
            // By this builder's own movement class: an air constructor reaches every spot, a tank fewer than a bot.
            .filter(|(_, at)| self.reachable_for(self.walker_of(unit.def), *at))
            .map(|(i, _)| (i, self.seconds_to_spot(unit.def, i, unit.pos)))
            .collect();
        spots.sort_by(|a, b| a.1.total_cmp(&b.1));
        spots
    }

    /// The next step of a builder's list as a menu of one option, played without asking (H-HANDS-SCRIPT). Steps
    /// that cannot be done are dropped and said; `assist` before any factory exists waits.
    #[allow(clippy::too_many_arguments)]
    fn scripted_step(&self, unit: &OwnUnit, name: &str, pianist: &mut Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, queue_ahead: bool, busy: bool, kit: &Kit) -> Option<Menu> {
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        loop {
            let step = pianist.scripts.get_mut(name)?.pop_front()?;
            let mut words = step.split_whitespace();
            let kind = words.next().unwrap_or_default();
            let place = words.next().map(str::to_string);
            let building = |def: UnitDefId| if can(def) { Ok((Pick::Building(def), Vec::new(), None)) } else { Err("this builder cannot build it".to_string()) };
            let at_place = |def: UnitDefId| match &place {
                _ if !can(def) => Err("this builder cannot build it".to_string()),
                Some(p) if picture.places.iter().any(|q| q.name == *p) => Ok((Pick::BuildingAt(def), Vec::new(), Some(p.clone()))),
                Some(p) => Err(format!("{p} is not a place in the picture")),
                None => Err("it needs a place".to_string()),
            };
            let resolved: Result<(Pick, Vec<usize>, Option<String>), String> = match kind {
                "extractor" if !can(kit.extractor) => Err("this builder cannot build it".to_string()),
                "extractor" => {
                    let spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                    match &place {
                        Some(p) => match p.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                            Some(i) if spots.iter().any(|(j, _)| *j == i) => Ok((Pick::Extractor, vec![i], Some(p.clone()))),
                            Some(_) => Err(format!("{p} is not free")),
                            None => Err(format!("{p} is not a spot")),
                        },
                        None if spots.is_empty() => Err("no free spot in reach".to_string()),
                        None => Ok((Pick::Extractor, spots.iter().map(|(i, _)| *i).collect(), None)),
                    }
                }
                "assist" => match own.iter().filter(|u| self.world.is_factory_def(u.def)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))) {
                    Some(factory) => Ok((Pick::AssistLab(factory.id), Vec::new(), None)),
                    None => {
                        // Nothing to help yet: the step waits at the front of the list.
                        pianist.scripts.get_mut(name)?.push_front(step);
                        return None;
                    }
                },
                // `reclaim <handle>`: one unit of ours, taken apart for its metal (the `remove` tool).
                "reclaim" => match place.as_deref().and_then(|h| self.unit_by_handle(h, own)) {
                    Some(target) if target.id == unit.id => Err("a builder cannot take itself apart".to_string()),
                    Some(target) => Ok((Pick::ReclaimUnit(target.id), Vec::new(), None)),
                    None => Err(format!("{} no longer stands", place.as_deref().unwrap_or("it"))),
                },
                // Any unit by its internal name; one that stands at a place (a defence, a radar) needs the place, and any
                // other building takes one when given (escalate-1-easy: `armvp home` from a constructor at the strip's
                // north end put the plant there, the place word read and dropped).
                other => match self.world.def_named(other) {
                    Some(def) if self.placed_at_place(def) || place.is_some() => at_place(def),
                    Some(def) => building(def),
                    None => Err(format!("'{other}' is not a unit name the game knows")),
                },
            };
            match resolved {
                Ok((pick, spots, where_)) => {
                    let key = kind.to_string();
                    return Some(Menu {
                        actor: Actor::Builder(unit.id),
                        name: name.to_string(),
                        busy,
                        queue_ahead,
                        questions: Vec::new(),
                        options: BTreeMap::from([(key.clone(), pick)]),
                        spots,
                        scripted: Some((key, where_)), policy: false,
                    });
                }
                Err(why) => {
                    let text = format!("skipped the step '{step}' of its list: {why}");
                    pianist.done.push(format!("{} {name}: {text}", super::picture::clock(frame)));
                    pianist.note(frame, format!("{name} {text}"));
                }
            }
        }
    }

    /// A building that stands at the place answered in `where` rather than beside the builder or in the yard: a
    /// defence, a radar, a jammer or sonar, a tier-2 extractor over a spot (the roster design, decision 3).
    pub(super) fn placed_at_place(&self, def: UnitDefId) -> bool {
        let Some(d) = self.world.def(def) else { return false };
        let flagged = super::glossary::entry(&d.name).is_some_and(|e| e.has_flag("radar_jammer") || e.has_flag("sonar"));
        d.speed == 0.0 && d.build_speed == 0.0 && d.build_options.is_empty() && (d.weapon_count > 0 || d.radar_range > 0.0 || d.extracts_metal > 0.0 || flagged)
    }

    /// The words on a building's option: the unit's words, where it goes, and what the picture knows that bears on
    /// it (the generators standing, the factory count, the store's fate over the build, the energy and metal lines).
    #[allow(clippy::too_many_arguments)]
    fn building_words(&self, builder: &OwnUnit, def: UnitDefId, tick: &Tick, energy_words: &str, metal_words: &str, factories: usize, generators: &str) -> String {
        let Some(d) = self.world.def(def) else { return String::new() };
        let map = &self.world.hello.map;
        let energy_maker = d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.wind_cap > 0.0;
        let placing = if d.extracts_metal > 0.0 {
            "over the extractor at the spot answered in `where`"
        } else if self.placed_at_place(def) {
            "at the place answered in `where`"
        } else if !d.build_options.is_empty() {
            "in the base yard"
        } else if d.build_speed > 0.0 {
            "beside the nearest factory: adds its build power to whatever that makes (metal must be flowing in faster than the factory spends it)"
        } else {
            "beside itself"
        };
        let mut tail: Vec<String> = Vec::new();
        if d.wind_cap > 0.0 {
            tail.push(format!("gives {:.0} to {:.0} energy a second here, {:.0} on average; building it draws energy", map.wind_min, map.wind_max, (map.wind_min + map.wind_max) / 2.0));
        } else if energy_maker {
            let steady = d.energy_make + (-d.energy_upkeep).max(0.0);
            tail.push(format!("a steady {steady:.0} energy a second{}", if d.energy_cost <= 0.0 { "; building it draws no energy, so it is the generator to build while the store is empty" } else { "" }));
        }
        if energy_maker {
            tail.push(generators.to_string());
            tail.push(format!("Our energy now: {energy_words}"));
        }
        if !d.build_options.is_empty() {
            // What of this very type is already under way, so a standing sentence in the packet ("an Advanced Vehicle
            // Plant goes up at west_yard") is not read by every free builder as an order for one more (fixes-1: three
            // tier-2 plants from three builders within 30 s, one asked for; escalate-3 and -7: four plants each).
            let mut under_way: Vec<String> = tick
                .snapshot
                .own_units
                .iter()
                .filter(|u| u.being_built && u.def == def)
                .map(|u| format!("one being built at {} ({:.0}% done)", self.world.grid(u.pos), 100.0 * u.health / u.max_health.max(1.0)))
                .collect();
            if let Some(pianist) = self.pianist.as_ref() {
                under_way.extend(pianist.tasks.iter().filter_map(|(builder, t)| match t {
                    Task::Build { def: d2, near, started: false, .. } if *d2 == def => Some(format!("one ordered by {} at {}, not started", self.actor_name(*builder), self.world.grid(*near))),
                    _ => None,
                }));
            }
            if !under_way.is_empty() {
                tail.push(format!("{} of this very type already under way: {}; this would be another", under_way.len(), under_way.join("; ")));
            }
            tail.push(match factories {
                0 => "we have no factory yet: nothing makes soldiers or constructors without one".to_string(),
                1 => "we have one factory already; a second doubles production when metal is banking up".to_string(),
                n => format!("we have {n} factories already"),
            });
            tail.push(format!("Our metal now: {metal_words}"));
        }
        if d.converter.is_some() {
            tail.push("turns energy into metal: only with a large energy surplus and no free spots".to_string());
        }
        if d.metal_storage > 0.0 || d.energy_storage > 0.0 {
            tail.push(format!("adds {:.0} metal and {:.0} energy to what we can store", d.metal_storage, d.energy_storage));
        }
        if d.radar_range > 0.0 {
            tail.push(format!("sees {:.0} around it", d.radar_range));
        }
        let draw = if !d.build_options.is_empty() || d.metal_cost >= 500.0 { self.build_draw_words(builder, def, tick) } else { String::new() };
        let tail = if tail.is_empty() { String::new() } else { format!(" {}.", tail.join(". ")) };
        format!("Build a {} {placing}.{tail}{draw}", self.unit_words(def))
    }

    fn build_draw_words(&self, builder: &OwnUnit, def: UnitDefId, tick: &Tick) -> String {
        let (Some(b), Some(d)) = (self.world.def(builder.def), self.world.def(def)) else { return String::new() };
        if d.build_time <= 0.0 || d.energy_cost <= 0.0 {
            return String::new();
        }
        // The store's fate over the build, not a warning (comet-1: "the store empties unless generators come first"
        // beside the plant kept the hands on extractors and solars until 1:40 with 890 metal banked; the store
        // would have run out for the last few seconds of a nineteen-second build).
        let seconds = d.build_time / b.build_speed.max(1.0);
        let draw = d.energy_cost / seconds;
        let energy = &tick.snapshot.energy;
        let spare = energy.income - energy.usage;
        let short = d.energy_cost - (energy.current + spare * seconds);
        if short <= 0.0 {
            format!(" Building it takes this builder about {seconds:.0} s and {:.0} energy ({draw:.0} a second); the store covers it.", d.energy_cost)
        } else {
            format!(" Building it takes this builder about {seconds:.0} s and {:.0} energy ({draw:.0} a second); the store runs out about {:.0} s before it is done and the build slows for that long.", d.energy_cost, (short / draw).min(seconds))
        }
    }
}

/// The soldiers of a group nearest a point, for a detachment.
pub(crate) fn nearest_of<'a>(units: &[&'a OwnUnit], to: Vec3, n: usize) -> Vec<&'a OwnUnit> {
    let mut sorted: Vec<&OwnUnit> = units.to_vec();
    sorted.sort_by(|a, b| a.pos.dist2d(to).total_cmp(&b.pos.dist2d(to)));
    sorted.into_iter().take(n).collect()
}
