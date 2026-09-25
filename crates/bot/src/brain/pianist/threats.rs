//! Threat response (`docs/design/2026-09-26-threat-response.md` §4-6): the hands answer raids in seconds and the
//! standing orders only bend it. Enumerated by threat, not by group: for each enemy party in our half, near a
//! structure of ours or coming toward home, the code lists the partial action states it can execute (a group
//! whole, the fastest few of a group hunting it, a group falling back), the standing orders prune states and set
//! defaults, a first Jev call asks one noul per state, and a second composes the worlds over the states the nouls
//! flagged. The pick is a plan that stands in code until the threat picture changes. worlds-2 (2026-09-26): four
//! Rovers never caught one Pawn, the hands vetoed a hand-set station seven times in 22 s, and a detachment became a
//! group with a stranger's rule; nouls-1 the same shape.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{EnemyUnit, OwnUnit, Tick, UnitId, Vec3};
use jev::{Answer, Question};
use serde_json::{Value, json};

use super::groups::GroupTask;
use super::menu::{ALARM, DETACH_PARTY_MAX};
use super::picture::{Party, Picture, Place};
use super::standing::{NEVER_REACH, RAIDER_REACH};
use super::{Brain, FRAMES_PER_SECOND};

/// Worlds in one question at most: past eight the pick diffuses (p(top) 0.63 under 8, 0.36 at 8-15, 0.23 at 24).
pub(super) const CAP: usize = 8;
/// A noul at or above this flags its state for the second call.
pub(super) const FLAG: f64 = 0.5;
/// A party this large is met whole, never hunted by a detachment.
const HUNT_PARTY_MAX: usize = 2 * DETACH_PARTY_MAX;
/// A group this far from a party has no state against it.
const RESPONSE_REACH: f32 = RAIDER_REACH;
/// A standing plan is asked again this long after its last ask even when nothing changed (a threat that lingers).
pub(super) const RE_ASK: i32 = 20 * FRAMES_PER_SECOND;
/// The speed an unidentified contact is taken to have when a hunt is sized against it (a Pawn's).
const UNIDENTIFIED_SPEED: f32 = 87.0;

/// What a group does about a threat.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Response {
    /// Nobody moves for it: the turrets, or nothing.
    Leave,
    /// The whole group attacks the party (an engagement that follows it within the leash).
    Whole(String),
    /// These members of the group hunt one unit of the party by id, raw, until it dies, is lost or the leash ends
    /// (§1 of the design; the micro engine's primitive when it lands, the brain's re-issued attack meanwhile).
    Hunt(String, Vec<UnitId>),
    /// The group falls back to the named place.
    Back(String, String),
}

impl Response {
    pub(super) fn group(&self) -> Option<&str> {
        match self {
            Response::Leave => None,
            Response::Whole(g) | Response::Hunt(g, _) | Response::Back(g, _) => Some(g),
        }
    }
}

/// One partial action state against a threat.
#[derive(Clone, Debug)]
pub(crate) struct State {
    /// The noul's id: `party_N.whole_group_X`, `party_N.hunt_group_X`, `party_N.back_group_X`; `party_N.leave` for
    /// the nothing state.
    pub id: String,
    pub response: Response,
    /// The state in a world's line: "3 Rovers of group_A hunt party_9 (168 against its 132)".
    pub words: String,
    /// Metal sent at the party.
    pub metal: f32,
    /// A standing rule's default (world 1 without asking) or the state already in force.
    pub default: bool,
    pub current: bool,
}

/// An enemy party that wants answering, and the states against it (index 0 is always Leave).
#[derive(Clone, Debug)]
pub(crate) struct Threat {
    pub party: Party,
    pub place: String,
    pub states: Vec<State>,
}

impl Threat {
    /// The state world 1 takes: the one in force, else a rule's default, else Leave.
    pub(super) fn base(&self) -> usize {
        self.states.iter().position(|s| s.current).or_else(|| self.states.iter().position(|s| s.default)).unwrap_or(0)
    }
}

/// One world: for each threat, the index of its state.
pub(crate) type World = Vec<usize>;

fn odds_by_metal(ours: f32, theirs: f32) -> &'static str {
    let ratio = ours / theirs.max(1.0);
    if ratio >= 2.5 {
        "we outweigh it heavily"
    } else if ratio >= 1.3 {
        "we outweigh it"
    } else if ratio >= 0.8 {
        "an even fight"
    } else {
        "it outweighs us"
    }
}

/// ", under 3 turrets: 1 armhlt, 2 armllt" for a party the enemy's armed buildings cover; empty otherwise.
fn under(p: &Party) -> String {
    if p.turrets.is_empty() { String::new() } else { format!(", under {}", p.turrets) }
}

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
    pub(super) fn threats(&self, tick: &Tick, picture: &Picture) -> Vec<Threat> {
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
            let mut states = vec![State { id: format!("{}.leave", party.name), response: Response::Leave, words: format!("nobody moves for {} ({}, {place}{}{killing})", party.name, party.composition, under(party)), metal: 0.0, default: false, current: false }];
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
                let all_ids: Vec<UnitId> = units.iter().map(|u| u.id).collect();
                let odds = self.odds_words(&units, party, enemies);
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
                    let enough = self.hunters_for(&fast, party, enemies);
                    if let Some(k) = enough {
                        let k = k.max(wanted).min(fast.len());
                        let hunters: Vec<UnitId> = fast[..k].iter().map(|u| u.id).collect();
                        let current = hunting_it;
                        let default = !default_set && !declined && raider_rule.starts_with("detachment");
                        let speed = fast[..k].iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
                        let what: BTreeMap<&str, usize> = fast[..k].iter().fold(BTreeMap::new(), |mut m, u| {
                            *m.entry(self.name(u.def)).or_default() += 1;
                            m
                        });
                        let who = what.iter().map(|(n, c)| format!("{c} {n}")).collect::<Vec<_>>().join(", ");
                        let rest = if k == units.len() { String::new() } else { format!(", the rest of {name} keeping its course") };
                        states.push(State {
                            id: format!("{}.hunt_{name}", party.name),
                            response: Response::Hunt(name.clone(), hunters),
                            words: format!("{who} of {name} hunt {} ({}{}) at {speed:.0} against its {quarry_speed:.0}, {distance:.0} away{rest}", party.name, party.composition, under(party)),
                            metal: fast[..k].iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum(),
                            default,
                            current,
                        });
                        default_set |= default;
                    }
                }
                // The whole group.
                if odds != "it outweighs us" && !odds.starts_with("we cannot hit") && !units.is_empty() {
                    let current = engaging_it;
                    let default = !default_set && !declined && (raider_rule == "whole_group" || named);
                    states.push(State {
                        id: format!("{}.whole_{name}", party.name),
                        response: Response::Whole(name.clone()),
                        words: format!("{name} attacks {} ({}{}) with the whole group, {distance:.0} away{leave}", party.name, party.composition, under(party)),
                        metal: standing,
                        default,
                        current,
                    });
                    default_set |= default;
                }
                // The way back.
                if odds == "it outweighs us" && distance < ALARM {
                    let to = rules.get("fall_back_to").cloned().unwrap_or_else(|| "home".to_string());
                    let current = matches!(&group.task, GroupTask::Move { place, fight: false, .. } if *place == to);
                    states.push(State {
                        id: format!("{}.back_{name}", party.name),
                        response: Response::Back(name.clone(), to.clone()),
                        words: format!("{name} falls back to {to} from {} ({}{}), which outweighs it, {distance:.0} away", party.name, party.composition, under(party)),
                        metal: 0.0,
                        default: false,
                        current,
                    });
                }
                let _ = all_ids;
            }
            out.push(Threat { party: party.clone(), place, states });
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

/// What the threat picture looks like for ask-on-change: the parties, where they stand, their size, whether they
/// are killing something, and which states are in force. A change asks; the same picture does not.
pub(super) fn signature(threats: &[Threat]) -> String {
    threats.iter().map(|t| format!("{}@{}:{}{}{}", t.party.name, t.place, t.party.ids.len(), if t.party.killing.is_some() { "!" } else { "" }, t.states.iter().filter(|s| s.current).map(|s| format!("[{}]", s.id)).collect::<String>())).collect::<Vec<_>>().join(" ")
}

/// The first call's nouls: for each threat with a state to consider beyond its base, whether it needs answering,
/// and one per such state.
pub(super) fn gate_questions(threats: &[Threat]) -> Vec<(String, Question)> {
    let mut out = Vec::new();
    for t in threats {
        let base = t.base();
        let open: Vec<&State> = t.states.iter().enumerate().filter(|(i, s)| *i != base && s.response != Response::Leave).map(|(_, s)| s).collect();
        if open.is_empty() {
            continue;
        }
        let standing = match &t.states[base].response {
            Response::Leave => "nobody moves for it".to_string(),
            _ => t.states[base].words.clone(),
        };
        out.push((
            format!("{}.answer", t.party.name),
            Question::noul(json!(format!(
                "Given `enemy`, `actors`, `player` and the player's `instructions`: does {} ({}, {}{}{}) need answering this second by someone other than what stands ({standing})? Yes when it is killing or about to kill something of ours that nothing there can stop; no when a turret handles it, it is leaving, or the instructions say to leave such a party.",
                t.party.name, t.party.composition, t.place, under(&t.party), t.party.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"))
            ))),
        ));
        for s in open {
            out.push((s.id.clone(), Question::noul(json!(format!("Is this the move to make against {} now, rather than {standing}? The move: {}.", t.party.name, s.words)))));
        }
    }
    out
}

/// The worlds after the gate: world 1 is every threat's base (the state in force, else a rule's default, else
/// nobody); a threat the gate says needs answering (`answer` at or above `FLAG`) opens every state against it as a
/// deviation, the states the gate rated higher first (threats-smoke-1: the answer noul said yes 68 of 79 times and
/// no state noul reached 0.5, so the state nouls rank and the answer opens); a threat with no `answer` noul opens
/// the states rated at or above `FLAG`; Leave is offered for a threat whose current state the gate said no longer
/// needs answering; a state whose group world 1 already sends elsewhere is pruned; at most `CAP`. `flags` receives
/// what the gate said. None when nothing opened.
pub(super) fn compose(threats: &[Threat], answers: &BTreeMap<String, Answer>, flags: &mut BTreeMap<String, f64>) -> Option<Vec<World>> {
    let noul = |id: &str| match answers.get(id) {
        Some(Answer::Noul { noul }) => Some(*noul),
        _ => None,
    };
    let base: World = threats.iter().map(Threat::base).collect();
    let taken: BTreeSet<&str> = threats.iter().zip(&base).filter_map(|(t, b)| t.states[*b].response.group()).collect();
    // (priority, the state's own noul negated for the sort, world)
    let mut deviations: Vec<(u8, i64, World)> = Vec::new();
    for (ti, t) in threats.iter().enumerate() {
        let answer = noul(&format!("{}.answer", t.party.name));
        if let Some(p) = answer {
            flags.insert(format!("{}.answer", t.party.name), p);
        }
        let opened = answer.is_some_and(|a| a >= FLAG);
        for (si, s) in t.states.iter().enumerate() {
            if si == base[ti] {
                continue;
            }
            if s.response == Response::Leave {
                // Dropping a current state the gate no longer wants.
                if t.states[base[ti]].current && answer.is_some_and(|p| p < FLAG) {
                    let mut w = base.clone();
                    w[ti] = si;
                    deviations.push((1, 0, w));
                }
                continue;
            }
            let p = noul(&s.id);
            if let Some(p) = p {
                flags.insert(s.id.clone(), p);
            }
            let rated = p.unwrap_or(0.0);
            if !(opened || (answer.is_none() && rated >= FLAG)) {
                continue;
            }
            if let Some(g) = s.response.group()
                && taken.contains(g)
                && t.states[base[ti]].response.group() != Some(g)
            {
                continue;
            }
            let mut w = base.clone();
            w[ti] = si;
            deviations.push((0, -((rated * 1000.0) as i64), w));
        }
    }
    if deviations.is_empty() {
        return None;
    }
    deviations.sort_by_key(|(p, r, _)| (*p, *r));
    let mut out = vec![base];
    out.extend(deviations.into_iter().map(|(_, _, w)| w));
    out.truncate(CAP);
    Some(out)
}

/// A world's line: the moves, and what follows: which threats are met with what metal and odds (turrets counted),
/// which are left to nobody.
pub(super) fn consequence(world: &World, threats: &[Threat]) -> String {
    let mut moves: Vec<String> = Vec::new();
    let (mut met, mut unmet): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for (t, si) in threats.iter().zip(world) {
        let s = &t.states[*si];
        let p = &t.party;
        let composition = if p.has_commander { format!("THEIR COMMANDER, whose death wins the game, with {}", p.composition) } else { p.composition.clone() };
        let killing = p.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"));
        match &s.response {
            Response::Leave => unmet.push(format!("{} ({composition}, {}{}{killing})", p.name, t.place, under(p))),
            Response::Back(..) => {
                moves.push(s.words.clone());
                unmet.push(format!("{} ({composition}, {}{}{killing})", p.name, t.place, under(p)));
            }
            Response::Whole(_) | Response::Hunt(..) => {
                moves.push(s.words.clone());
                let theirs = p.metal + p.turret_metal;
                let with = if p.turrets.is_empty() { String::new() } else { format!(" with {} ({:.0} metal)", p.turrets, p.turret_metal) };
                met.push(format!("{} ({composition}, {:.0} metal, {}){with} met with {:.0} metal: {}{killing}", p.name, p.metal, t.place, s.metal, odds_by_metal(s.metal, theirs)));
            }
        }
    }
    let mut parts: Vec<String> = Vec::new();
    if moves.is_empty() {
        parts.push("Nobody moves".to_string());
    } else {
        parts.push(moves.join("; "));
    }
    if !met.is_empty() {
        parts.push(format!("Met: {}", met.join("; ")));
    }
    if !unmet.is_empty() {
        parts.push(format!("Left to nobody: {}", unmet.join("; ")));
    }
    format!("{}.", parts.join(". "))
}

/// The one question of the second call: a Choice over the worlds' lines.
pub(super) fn question(lines: &[String]) -> Question {
    let instructions = json!(
        "Given `enemy`, `actors`, `player` and the player's `instructions`, which answer to the enemy parties in our ground is best this second? Each option is one world: who moves against what, what is met with what metal and odds, and what is left to nobody. w1 is what stands now (the moves in force and the player's standing rules); every other world changes one party's answer. The instructions were written before this picture: where they name a place, a party or a rule, follow them; where the situation has changed, pick the world they would call for."
    );
    Question::Choice { instructions, criteria: lines.iter().enumerate().map(|(i, l)| (format!("w{}", i + 1), json!(l))).collect() }
}

/// The picked world's index and the pick's confidence.
pub(super) fn pick(answers: &BTreeMap<String, Answer>, worlds: &[World]) -> Option<(usize, f64)> {
    match answers.get("worlds.pick") {
        Some(Answer::Choice { choice, confidence, .. }) => choice.strip_prefix('w').and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=worlds.len()).contains(n)).map(|n| (n - 1, *confidence)),
        _ => None,
    }
}

/// The threats for the log.
pub(super) fn log_threats(threats: &[Threat]) -> Value {
    json!(threats
        .iter()
        .map(|t| {
            json!({
                "party": t.party.name,
                "place": t.place,
                "states": t.states.iter().map(|s| json!({ "id": s.id, "words": s.words, "metal": s.metal, "default": s.default, "current": s.current })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn party(name: &str, n: usize) -> Party {
        Party { name: name.to_string(), ids: (0..n as i32).map(|i| UnitId(1000 + i)).collect(), at: Vec3::default(), metal: 54.0 * n as f32, composition: format!("{n} armpw"), has_commander: false, killing: None, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() }
    }

    fn state(id: &str, response: Response, default: bool, current: bool) -> State {
        State { id: id.to_string(), response, words: id.to_string(), metal: 300.0, default, current }
    }

    fn threat(name: &str, states: Vec<State>) -> Threat {
        let mut all = vec![state(&format!("{name}.leave"), Response::Leave, false, false)];
        all.extend(states);
        Threat { party: party(name, 1), place: "at spot_1".into(), states: all }
    }

    #[test]
    fn world_one_is_the_defaults_and_flags_make_the_deviations() {
        let threats = vec![
            threat("party_1", vec![state("party_1.hunt_group_A", Response::Hunt("group_A".into(), vec![UnitId(1)]), true, false), state("party_1.whole_group_A", Response::Whole("group_A".into()), false, false)]),
            threat("party_2", vec![state("party_2.whole_group_B", Response::Whole("group_B".into()), false, false), state("party_2.whole_group_A", Response::Whole("group_A".into()), false, false)]),
        ];
        let noul = |p: f64| Answer::Noul { noul: p };
        let answers = BTreeMap::from([
            ("party_1.answer".to_string(), noul(0.9)),
            ("party_1.whole_group_A".to_string(), noul(0.7)),
            ("party_2.answer".to_string(), noul(0.8)),
            ("party_2.whole_group_B".to_string(), noul(0.6)),
            ("party_2.whole_group_A".to_string(), noul(0.9)),
        ]);
        let mut flags = BTreeMap::new();
        let worlds = compose(&threats, &answers, &mut flags).expect("flagged");
        assert_eq!(worlds[0], vec![1, 0], "world 1: party_1's default hunt, party_2 left");
        assert!(worlds.contains(&vec![2, 0]), "the flagged whole-group fight at party_1");
        assert!(worlds.contains(&vec![1, 1]), "group_B at party_2");
        assert!(!worlds.contains(&vec![1, 2]), "group_A is taken by party_1 in world 1: not sent to party_2 too");
        assert_eq!(flags["party_2.whole_group_A"], 0.9);
        let line = consequence(&worlds[0], &threats);
        assert!(line.contains("Met: party_1") && line.contains("Left to nobody: party_2"), "{line}");
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w2".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((1, 0.6)));
    }

    #[test]
    fn an_unanswered_threat_and_a_dropped_current_state() {
        let threats = vec![threat("party_3", vec![state("party_3.whole_group_A", Response::Whole("group_A".into()), false, true)])];
        let mut flags = BTreeMap::new();
        assert!(compose(&threats, &BTreeMap::from([("party_3.answer".to_string(), Answer::Noul { noul: 0.9 })]), &mut flags).is_none(), "the current state stands, nothing else flagged");
        let worlds = compose(&threats, &BTreeMap::from([("party_3.answer".to_string(), Answer::Noul { noul: 0.1 })]), &mut flags).expect("the drop is offered");
        assert_eq!(worlds, vec![vec![1], vec![0]]);
        assert!(gate_questions(&threats).is_empty(), "a threat with its only state in force asks nothing");
    }
}
