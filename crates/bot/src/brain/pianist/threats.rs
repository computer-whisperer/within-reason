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
/// A group this far from a party has no state against it.
const RESPONSE_REACH: f32 = RAIDER_REACH;
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
            let quarry_speed = party.ids.iter().map(|id| speed_of(enemies.iter().find(|e| e.id == *id).and_then(|e| e.def))).fold(0.0, f32::max);
            let place = place_of(&picture.places, party.at).map_or("in sight".to_string(), |pl| format!("at {pl}"));
            let killing = party.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"));
            let state = |id: String, actor: String, response: Response, words: String, metal: f32, default: bool, current: bool| State { id, actor, response, words, metal, dim: "threat", default, current, pair_only: false };
            let mut states = vec![state(format!("{}.leave", party.name), String::new(), Response::Leave, format!("nobody moves for {} ({}, {place}{}{killing})", party.name, party.composition, under(party)), 0.0, false, false)];
            // The nearest group with a raider rule sets the default.
            let mut default_set = false;
            let mut groups: Vec<(f32, &super::groups::Group)> = pianist.groups.iter().filter_map(|g| super::groups::centre_of(&g.units(own)).map(|c| (c.dist2d(party.at), g))).collect();
            groups.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (distance, group) in groups {
                if distance > RESPONSE_REACH {
                    continue;
                }
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
                let odds = self.odds_words(&units, party, enemies);
                // `no_chase`: the whole group goes only after a party within reach of its station (else its last
                // hold, else home); the course slot's hold ends an engagement the quarry carries beyond it.
                let anchor = rules.get("station").and_then(|s| picture.places.iter().find(|p| p.name == *s)).map(|p| p.at).or(group.last_hold).unwrap_or(home);
                let chase_allowed = !rules.get("no_chase").is_some_and(|v| v == "yes") || party.at.dist2d(anchor) <= RAIDER_REACH;
                // A group slower than the party can drive it off, not kill it: said in the line.
                let group_speed = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
                let outrun = if group_speed < quarry_speed { format!(" (it outruns this group at {quarry_speed:.0} against {group_speed:.0}: a chase drives it off, a kill needs faster hunters)") } else { String::new() };
                let standing: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
                let hunting_it = group.hunt.as_ref().is_some_and(|h| party.ids.contains(&h.quarry));
                let declined = group.declined.iter().any(|(p, f)| *p == party.name && tick.frame - f < super::groups::DECLINE_FRAMES);
                let engaging_it = matches!(&group.task, GroupTask::Engage { party: ids, .. } if ids.iter().any(|id| party.ids.contains(id)));
                let leave = match &group.task {
                    GroupTask::Hold { .. } => format!(", leaving {} unguarded", picture.state["actors"][&name]["at"].as_str().unwrap_or("where it stands")),
                    GroupTask::Move { place, .. } => format!(", abandoning its way to {place}"),
                    GroupTask::Engage { .. } => ", leaving the party it was attacking".to_string(),
                };
                // The hunt: the fastest members that outrun the party, the fewest that outweigh it.
                if party.ids.len() <= HUNT_PARTY_MAX && group.domain != crate::world::Domain::Air && !rules.get("no_detachments").is_some_and(|v| v == "yes") && (!declined || hunting_it) {
                    let mut fast: Vec<&OwnUnit> = units.iter().copied().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed > quarry_speed)).collect();
                    fast.sort_by(|a, b| a.pos.dist2d(party.at).total_cmp(&b.pos.dist2d(party.at)));
                    let wanted = raider_rule.strip_prefix("detachment:").and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    if let Some(k) = self.hunters_for(&fast, party, enemies) {
                        let k = k.max(wanted).min(fast.len());
                        let hunters: Vec<UnitId> = fast[..k].iter().map(|u| u.id).collect();
                        let default = !default_set && !declined && raider_rule.starts_with("detachment");
                        let speed = fast[..k].iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
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
                            format!("{who} of {name} hunt {} ({}{}) at {speed:.0} against its {quarry_speed:.0}, {distance:.0} away{rest}", party.name, party.composition, under(party)),
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
                    states.push(state(
                        format!("{}.whole_{name}", party.name),
                        name.clone(),
                        Response::Whole,
                        format!("{name} attacks {} ({}{}) with the whole group, {distance:.0} away{leave}{outrun}", party.name, party.composition, under(party)),
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
                        format!("{name} falls back to {to} from {} ({}{}), which outweighs it, {distance:.0} away", party.name, party.composition, under(party)),
                        0.0,
                        false,
                        current,
                    ));
                }
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
