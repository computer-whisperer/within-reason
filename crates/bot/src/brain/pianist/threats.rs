//! The threat family of the pass (`docs/design/2026-09-26-threat-response.md` §4-6, folded into
//! `2026-09-26-one-pass.md`): for each enemy party in our half, near a structure of ours or coming toward home, the
//! states the groups within reach can execute against it (the whole group, the fastest few hunting it, the way
//! back), the standing orders pruning and setting defaults. worlds-2 (2026-09-26): four Rovers never caught one
//! Pawn, the hands vetoed a hand-set station seven times in 22 s, and a detachment became a group with a stranger's
//! rule; the raid scenario measured the hunt at 11 s to the first kill against 45-100 s without.

use std::collections::BTreeMap;

use bot_protocol::{EnemyUnit, OwnUnit, Tick, UnitId, Vec3};
use jev::Question;
use serde_json::json;

use super::groups::GroupTask;
use super::picture::{Party, Picture, Place};
use super::plan::{ALARM, DETACH_PARTY_MAX, Kind, Response, Slot, State, under};
use super::standing::{NEVER_REACH, RAIDER_REACH};
use super::Brain;

/// A party this large is met whole, never hunted by a detachment.
const HUNT_PARTY_MAX: usize = 2 * DETACH_PARTY_MAX;
/// Groups offered against one party at most, nearest first: the question count stays bounded on a map of many groups.
const GROUPS_PER_PARTY: usize = 3;
/// The speed an unidentified contact is taken to have when a hunt is sized against it (a Pawn's).
const UNIDENTIFIED_SPEED: f32 = 87.0;

/// The named place a point stands at (within 400), if any.
pub(super) fn place_of(places: &[Place], at: Vec3) -> Option<&str> {
    places.iter().map(|pl| (pl.at.dist2d(at), pl)).min_by(|a, b| a.0.total_cmp(&b.0)).filter(|(d, _)| *d < 400.0).map(|(_, pl)| pl.name.as_str())
}

impl Brain {
    /// The threats this second and the states against each: every party in our half, within reach of a structure
    /// of ours or coming toward home; for each group within reach the whole-group fight when it outweighs the party
    /// (turrets counted), the hunt by the fastest few that outrun it and outweigh it, the way back when the party
    /// outweighs the group within the alarm reach. The standing orders prune (`never`, `ignore`, `no_detachments`)
    /// and set defaults (`whole_group`, `detachment:N`, `engage_party`); a group already engaging or hunting the
    /// party has that state as current.
    pub(super) fn threat_slots(&self, tick: &Tick, picture: &Picture) -> Vec<Slot> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        let Some(kit) = self.kit else { return Vec::new() };
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let home = self.home;
        let enemy_start = self.journal.intent.enemy_start;
        let structures: Vec<Vec3> = own
            .iter()
            .filter(|u| !u.being_built && (kit.is_extractor(u.def) || u.def == kit.turret || self.world.is_factory_def(u.def) || self.world.is_mobile_builder(u.def) || self.world.def(u.def).is_some_and(|d| d.speed == 0.0)))
            .map(|u| u.pos)
            .collect();
        let speed_of = |def: Option<bot_protocol::UnitDefId>| def.and_then(|d| self.world.def(d)).map_or(UNIDENTIFIED_SPEED, |d| d.speed);
        let mut out = Vec::new();
        for party in &picture.parties {
            let toward_home = {
                let vel = enemies.iter().filter(|e| party.ids.contains(&e.id)).fold(Vec3::default(), |s, e| Vec3 { x: s.x + e.vel.x, y: 0.0, z: s.z + e.vel.z });
                vel.x.hypot(vel.z) >= 0.3 && (home.x - party.at.x) * vel.x + (home.z - party.at.z) * vel.z > 0.0
            };
            let ours = party.at.dist2d(home) < party.at.dist2d(enemy_start);
            let near_ours = structures.iter().any(|s| s.dist2d(party.at) <= RAIDER_REACH);
            if !(ours || near_ours || toward_home) {
                continue;
            }
            // Whether we catch it is its slowest identified member's pace (H-HANDS-ODDS-WHAT-SHOOTS); a party of
            // radar contacts only has no known speed, and is not priced as a Pawn's 87 (Cape Violet 10:20: "it
            // outruns this group at 87 against 75" of Welders at 48).
            let identified_speeds: Vec<f32> = party.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).map(|d| speed_of(Some(d))).filter(|s| *s > 0.0).collect();
            let quarry_speed = identified_speeds.iter().copied().fold(f32::INFINITY, f32::min);
            let quarry_known = quarry_speed.is_finite();
            let quarry_speed = if quarry_known { quarry_speed } else { UNIDENTIFIED_SPEED };
            let place = place_of(&picture.places, party.at).map_or("in sight".to_string(), |pl| format!("at {pl}"));
            let killing = party.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"));
            let state = |id: String, actor: String, response: Response, words: String, metal: f32, default: bool, current: bool| State { id, actor, response, words, metal, dim: "threat", default, current, pair_only: false };
            let mut states = vec![state(format!("{}.leave", party.name), String::new(), Response::Leave, format!("nobody moves for {} ({}, {place}{}{killing})", party.name, party.composition, under(party)), 0.0, false, false)];
            // The nearest group with a raider rule sets the default.
            let mut default_set = false;
            // By the nearest member, not the centre (H-HANDS-GROUP-BODY): the odds on the part in the fight, the
            // tail said when the group is strung out.
            let mut groups: Vec<(f32, super::groups::Body, &super::groups::Group)> = pianist.groups.iter().filter_map(|g| g.body(own, Some(party.at)).map(|b| (b.front.dist2d(party.at), b, g))).collect();
            groups.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (distance, body, group) in groups.into_iter().take(GROUPS_PER_PARTY) {
                let name = format!("group_{}", group.name);
                let rules = pianist.standing.rules_for(&name);
                let never = pianist.standing.never_places(&name);
                let raider_rule = rules.get(if party.ids.len() == 1 { "raiders_lone" } else { "raiders_party" }).cloned().unwrap_or_default();
                let named = rules.get("engage_party").is_some_and(|p| *p == party.name);
                if raider_rule == "ignore" && !named {
                    continue;
                }
                if never.iter().any(|n| picture.places.iter().any(|pl| pl.name == *n && pl.at.dist2d(party.at) < NEVER_REACH)) {
                    continue;
                }
                let units = group.units(own);
                let (odds, odds_words) = self.group_odds(&body, party, enemies, &tick.snapshot.allies);
                let tail = if body.strung_out() { format!(", its tail {:.0} behind its front", body.length) } else { String::new() };
                // `no_chase`: the whole group goes only after a party within reach of its station (else its last
                // hold, else home); the course slot's hold ends an engagement the quarry carries beyond it.
                let anchor = rules.get("station").and_then(|s| picture.places.iter().find(|p| p.name == *s)).map(|p| p.at).or(group.last_hold).unwrap_or(home);
                let chase_allowed = !rules.get("no_chase").is_some_and(|v| v == "yes") || party.at.dist2d(anchor) <= RAIDER_REACH;
                // The speeds are in the odds words now (`odds_words`); the walk below uses the group's slowest.
                let group_speed = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
                let outrun = String::new();
                let standing: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
                let hunting_it = group.hunt.as_ref().is_some_and(|h| party.ids.contains(&h.quarry));
                let declined = group.declined.iter().any(|(p, f)| *p == party.name && tick.frame - f < super::groups::DECLINE_FRAMES);
                let engaging_it = matches!(&group.task, GroupTask::Engage { party: ids, .. } if ids.iter().any(|id| party.ids.contains(id)));
                let leave = match &group.task {
                    GroupTask::Hold { .. } => format!(", leaving {} unguarded", picture.state["actors"][&name]["at"].as_str().unwrap_or("where it stands")),
                    GroupTask::Move { place, .. } => format!(", abandoning its way to {place}"),
                    GroupTask::Engage { .. } => ", leaving the party it was attacking".to_string(),
                };
                // The hunt: the armed members that outrun the party, else the group's fastest armed members (a
                // Blitz at 101 never outruns a Tick at 132, so no Blitz was ever offered against one and the pick
                // sent the commander on a 31 s walk instead, onepass-player-4 5:59; the user: the Blitz would have
                // had the Tick out of our base sooner, and that time is what to present); the fewest that outweigh
                // it. The words say in how many seconds the hunters drive it off and whether they can catch it.
                if party.ids.len() <= HUNT_PARTY_MAX && group.domain != crate::world::Domain::Air && !rules.get("no_detachments").is_some_and(|v| v == "yes") && (!declined || hunting_it) {
                    let armed = |u: &&OwnUnit| self.world.def(u.def).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0);
                    let mut fast: Vec<&OwnUnit> = units.iter().copied().filter(armed).filter(|u| self.world.def(u.def).is_some_and(|d| d.speed > quarry_speed)).collect();
                    if fast.is_empty() {
                        let top = units.iter().copied().filter(armed).filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(0.0, f32::max);
                        fast = units.iter().copied().filter(armed).filter(|u| self.world.def(u.def).is_some_and(|d| d.speed >= top - 1.0)).collect();
                    }
                    fast.sort_by(|a, b| a.pos.dist2d(party.at).total_cmp(&b.pos.dist2d(party.at)));
                    let wanted = raider_rule.strip_prefix("detachment:").and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    if let Some(k) = self.hunters_for(&fast, party, enemies) {
                        let k = k.max(wanted).min(fast.len());
                        let hunters: Vec<UnitId> = fast[..k].iter().map(|u| u.id).collect();
                        let default = !default_set && !declined && raider_rule.starts_with("detachment");
                        let speed = fast[..k].iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
                        let nearest = fast[..k].iter().map(|u| u.pos.dist2d(party.at)).fold(f32::INFINITY, f32::min);
                        let drive = if speed.is_finite() && speed > 0.0 { format!(": they drive it off in {:.0} s from {nearest:.0} away", nearest / speed) } else { String::new() };
                        let catching = if !quarry_known { " (its speed is unknown: radar contacts only)".to_string() } else if speed > quarry_speed { format!(" and can catch it ({speed:.0} against its slowest {quarry_speed:.0})") } else { format!(" without catching it ({speed:.0} against its slowest {quarry_speed:.0}): it leaves when they arrive, or stands and dies") };
                        let what: BTreeMap<&str, usize> = fast[..k].iter().fold(BTreeMap::new(), |mut m, u| {
                            *m.entry(self.name(u.def)).or_default() += 1;
                            m
                        });
                        let who = what.iter().map(|(n, c)| format!("{c} {n}")).collect::<Vec<_>>().join(", ");
                        let rest = if k == units.len() { String::new() } else { format!(", the rest of {name} keeping its course") };
                        states.push(state(
                            format!("{}.hunt_{name}", party.name),
                            name.clone(),
                            Response::Hunt(hunters),
                            format!("{who} of {name} hunt {} ({}{}{killing}){drive}{catching}{rest}", party.name, party.composition, under(party)),
                            fast[..k].iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum(),
                            default,
                            hunting_it,
                        ));
                        default_set |= default;
                    }
                }
                // The whole group.
                if odds != "it outweighs us" && !odds.starts_with("we cannot hit") && !units.is_empty() && (chase_allowed || engaging_it) {
                    let default = !default_set && !declined && (raider_rule == "whole_group" || named);
                    let walk = if group_speed.is_finite() && group_speed > 0.0 { format!(", {:.0} s of walking", distance / group_speed) } else { String::new() };
                    // A radar contact is priced as a Pawn (`combat.rs`): at minute 14 a column of 23 blips said 2,530
                    // metal against our 4,820 and cost ten Blitzes (onepass-player-3), so the words call it a floor.
                    let theirs = if party.composition.contains("unidentified") { format!("its {:.0} at least, the unidentified contacts priced as Pawns", party.metal) } else { format!("its {:.0}", party.metal) };
                    states.push(state(
                        format!("{}.whole_{name}", party.name),
                        name.clone(),
                        Response::Whole,
                        format!("{name} attacks {} ({}{}) with the whole group ({standing:.0} metal against {theirs}: {odds_words}), {distance:.0} from its front{tail}{walk}{leave}{outrun}", party.name, party.composition, under(party)),
                        standing,
                        default,
                        engaging_it,
                    ));
                    default_set |= default;
                }
                // The way back.
                if odds == "it outweighs us" && distance < ALARM {
                    let to = rules.get("fall_back_to").cloned().unwrap_or_else(|| "home".to_string());
                    let current = matches!(&group.task, GroupTask::Move { place, fight: false, .. } if *place == to);
                    states.push(state(
                        format!("{}.back_{name}", party.name),
                        name.clone(),
                        Response::Back(to.clone()),
                        format!("{name} falls back to {to} from {} ({}{}), which outweighs it ({odds_words}), {distance:.0} from its front{tail}", party.name, party.composition, under(party)),
                        0.0,
                        false,
                        current,
                    ));
                }
            }
            // An armed builder that outweighs the party alone attacks it: a state for any party within the raider reach,
            // the walk in the words, the pick weighing it. Fixed reaches hid the answer: a Pawn 550 from the commander
            // killed a Sentry and an extractor for a quarter minute (onepass-norules-hard-1, 2:44), and a Pawn 1,000
            // from it hit a constructor at spot_45 for half a minute (onepass-player-3, 2:36-3:04) with the attack never
            // a question; the user: chase off or kill intrusions fast, the raider getting away alive is the lesser
            // issue. The words say the walk and, for a raider that outruns it, that this drives it off rather than
            // kills it.
            for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def) && self.world.def(u.def).is_some_and(|d| d.weapon_count > 0)) {
                let distance = party.at.dist2d(unit.pos);
                if distance > RAIDER_REACH {
                    continue;
                }
                let odds = self.odds_words(&[unit], party, enemies);
                if !odds.starts_with("we outweigh") {
                    continue;
                }
                let name = self.actor_name(unit.id);
                let rules = pianist.standing.rules_for(&name);
                let current = matches!(pianist.tasks.get(&unit.id), Some(super::Task::Walk { place, .. }) if *place == party.name);
                let default = !default_set && rules.get("attack_raiders").is_some_and(|v| v == "yes");
                let speed = self.world.def(unit.def).map_or(0.0, |d| d.speed);
                let walk = if speed > 0.0 { format!(": it drives it off in {:.0} s of walking", distance / speed) } else { String::new() };
                let killing = party.killing.as_ref().map_or(String::new(), |(what, metal)| format!(", killing {what} ({metal:.0} metal) now"));
                let chase = if !quarry_known { "; its speed is unknown (radar contacts only)".to_string() } else if quarry_speed > speed { format!("; it outruns {name} at {quarry_speed:.0} against {speed:.0}: this drives it off if it stays, and kills it only if it stands and fights") } else { String::new() };
                // A skirmisher outranges a commander (onepass-player-3, 22:14: it walked at a Hound, 650 against 300,
                // that backed off shooting; the player's packet keeps it home against anything that outranges it).
                let reach = self.world.def(unit.def).map_or(0.0, |d| d.reach);
                let their_reach = party.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).filter_map(|d| self.world.def(d)).map(|d| d.reach).fold(0.0, f32::max);
                let range = if their_reach > reach { format!("; it outranges {name} ({their_reach:.0} against {reach:.0}): it is hit on the way in and lands nothing unless the party stands") } else { String::new() };
                let doing = pianist.tasks.get(&unit.id).map_or(String::new(), |t| format!(", leaving {}", self.task_course(Some(t), unit, &picture.places, tick.frame, own)));
                states.push(state(
                    format!("{}.attack_{name}", party.name),
                    name.clone(),
                    Response::Attack(party.name.clone()),
                    format!("{name} attacks {} ({}{}, {distance:.0} away{killing}) and comes back to what it was doing{walk}; against it alone, {odds}{chase}{range}{doing}", party.name, party.composition, under(party)),
                    self.world.def(unit.def).map_or(0.0, |d| d.metal_cost),
                    default,
                    current,
                ));
                default_set |= default;
            }
            out.push(Slot { name: party.name.clone(), kind: Kind::Threat(party.clone(), place), states, queue_ahead: false, idle: false });
        }
        out
    }

    /// The fewest of the fast units (nearest first) that outweigh the party by the combat table, or None when
    /// none of them together do.
    fn hunters_for(&self, fast: &[&OwnUnit], party: &Party, enemies: &[EnemyUnit]) -> Option<usize> {
        if fast.is_empty() {
            return None;
        }
        let mut theirs = super::super::combat::Force::default();
        for enemy in enemies.iter().filter(|e| party.ids.contains(&e.id)) {
            match enemy.def {
                Some(def) => theirs.add(def),
                None => theirs.unidentified += 1,
            }
        }
        theirs.turret_metal += party.turret_metal;
        theirs.turret_metal_air += party.turret_metal_air;
        (1..=fast.len()).find(|n| self.odds(&Brain::force_of(&fast[..*n]), &theirs) >= 1.3)
    }
}

/// A threat's nouls: whether it needs answering by someone other than what stands, and one per open state.
pub(super) fn gate_questions(slot: &Slot) -> Vec<(String, Question)> {
    let Kind::Threat(party, place) = &slot.kind else { return Vec::new() };
    let base = slot.base();
    let open: Vec<&State> = slot.states.iter().enumerate().filter(|(i, _)| *i != base && *i != 0).map(|(_, s)| s).collect();
    if open.is_empty() {
        return Vec::new();
    }
    let standing = if base == 0 { "nobody moves for it".to_string() } else { slot.states[base].words.clone() };
    let mut out = vec![(
        format!("{}.answer", party.name),
        Question::noul(json!(format!(
            "Given `enemy`, `actors`, `player` and the player's `instructions`: does {} ({}, {place}{}{}) need answering this second by someone other than what stands ({standing})? Yes when it is killing or about to kill something of ours that nothing there can stop; no when a turret handles it, it is leaving, or the instructions say to leave such a party.",
            party.name, party.composition, under(party), party.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"))
        ))),
    )];
    for s in open {
        out.push((s.id.clone(), Question::noul(json!(format!("Is this the move to make against {} now, rather than {standing}? The move: {}.", party.name, s.words)))));
    }
    out
}
