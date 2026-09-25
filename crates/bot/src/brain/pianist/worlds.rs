//! The worlds question (`docs/design/2026-09-25-one-decider.md` §4): the standing rules generate each group's few
//! candidates, and one Choice over the groups' joined worlds is the army's only decider. World 1 is what the rules
//! say (each group's rule result, or its course); every other world changes one group's move, so a deviation
//! winning is the record of a rule breaking down. Offline (`docs/studies/2026-09-25-joint-worlds.md`,
//! K-jev-one-question-over-joined-worlds-coordinates) one question over joined worlds answered the raiders at 60-83%
//! with no two groups sent after one lone raider, and the pick diffused past eight worlds: hence the cap.

use std::collections::BTreeMap;

use bot_protocol::Tick;
use jev::{Answer, Question};
use serde_json::{Value, json};

use super::groups::{GroupTask, centre_of};
use super::menu::{ALARM, Actor, DETACH_PARTY_MAX, Menu};
use super::picture::{Party, Picture};
use super::standing::{Order, RAIDER_REACH};
use super::{Brain, FRAMES_PER_SECOND};

/// Worlds in one question at most: past eight the pick diffuses (p(top) 0.63 under 8, 0.36 at 8-15, 0.23 at 24).
pub(super) const CAP: usize = 8;
/// A party this small needs one taker: a second group sent at it is pruned.
const SMALL_PARTY: usize = 2;
/// Candidates a group has at most: its course, its rule, a fight or two, the way back.
const CANDIDATES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Keep,
    Rule,
    Fight,
    Back,
}

#[derive(Clone, Debug)]
pub(super) struct Candidate {
    pub kind: Kind,
    pub order: Order,
    /// The move in a world's line: "attacks party_3 (2 armpw) with the whole group, leaving spot_38 unguarded".
    pub words: String,
    /// The party a fight is at, and its size.
    pub party: Option<(String, usize)>,
    /// Metal sent at the party by a fight.
    pub metal: f32,
}

/// One world: for each varying group, the index of its candidate.
pub(super) type World = Vec<(String, usize)>;

fn order(choice: &str, params: &[(&str, &str)]) -> Order {
    Order { choice: choice.to_string(), params: params.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() }
}

/// An order in a world's words.
fn order_words(o: &Order) -> String {
    let p = |k: &str| o.params.get(k).cloned().unwrap_or_default();
    match o.choice.as_str() {
        "move_to" => format!("walks to {}", p("where")),
        "fight_to" => format!("advances to {}", p("where")),
        "engage" => format!("attacks {} with the whole group", p("whom")),
        "send_against" => format!("sends {} soldiers against {}, the rest carrying on", p("how_many"), p("whom")),
        "hold" => "holds where it stands".to_string(),
        "continue" => "carries on".to_string(),
        "retreat" => "falls back home".to_string(),
        "fall_back" => "falls back to where it last held".to_string(),
        other => match other.strip_prefix("join_") {
            Some(g) => format!("merges into {g}"),
            None => other.to_string(),
        },
    }
}

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

impl Brain {
    /// A group's candidates this second: its course, its standing rule's result when it differs, a fight or two at
    /// the nearest party within reach, the way back when the fight is against it. Empty for a group that is not asked.
    pub(super) fn candidates(&self, tick: &Tick, picture: &Picture, menu: &Menu, rule: Option<&Order>) -> Vec<Candidate> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        let Actor::Group(gname) = &menu.actor else { return Vec::new() };
        let Some(group) = pianist.groups.iter().find(|g| g.name == *gname) else { return Vec::new() };
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let units = group.units(own);
        let Some(centre) = centre_of(&units) else { return Vec::new() };
        let offered = |o: &str| menu.options.contains_key(o);
        let at = picture.state["actors"][&menu.name]["at"].as_str().unwrap_or("where it stands").to_string();
        let (course, leave) = match &group.task {
            GroupTask::Hold { .. } => (format!("keeps holding at {at}"), format!(", leaving {at} unguarded")),
            GroupTask::Move { place, fight, .. } => (
                format!("keeps {} to {place} (the course it was given)", if *fight { "advancing" } else { "walking" }),
                format!(", abandoning its {} to {place} (the course it was given)", if *fight { "advance" } else { "walk" }),
            ),
            GroupTask::Engage { .. } => ("keeps attacking its party".to_string(), ", leaving the party it was attacking".to_string()),
        };
        let keep = if offered("continue") { "continue" } else { "hold" };
        let mut out = vec![Candidate { kind: Kind::Keep, order: order(keep, &[]), words: course, party: None, metal: 0.0 }];
        let standing: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let party_of = |name: &str| picture.parties.iter().find(|p| p.name == name);
        let leaves = |o: &Order| !matches!(o.choice.as_str(), "continue" | "hold");
        if let Some(rule) = rule
            && !self.same_as_current(menu, rule)
            && rule.choice != keep
        {
            let (party, metal) = match rule.choice.as_str() {
                "engage" => (rule.params.get("whom").and_then(|w| party_of(w)).map(|p| (p.name.clone(), p.ids.len())), standing),
                "send_against" => {
                    let n: f32 = rule.params.get("how_many").and_then(|n| n.parse().ok()).unwrap_or(1.0);
                    (rule.params.get("whom").and_then(|w| party_of(w)).map(|p| (p.name.clone(), p.ids.len())), n * standing / units.len().max(1) as f32)
                }
                _ => (None, 0.0),
            };
            let words = format!("{} (its standing rule){}", order_words(rule), if leaves(rule) { leave.as_str() } else { "" });
            out.push(Candidate { kind: Kind::Rule, order: rule.clone(), words, party, metal });
        }
        // The nearest party within reach, of any size: the whole group at it, or a detachment that outweighs it. A
        // block is a party too (hold-1-comet-easy, 21:40: 32 Blitzes and 5 Stouts at our plant, 680 from a group of
        // 90 Stouts that walked back to its station under their fire, because parties over six were not candidates).
        let nearest = picture
            .parties
            .iter()
            .filter(|p| !p.ids.is_empty() && p.at.dist2d(centre) <= RAIDER_REACH)
            .min_by(|a, b| a.at.dist2d(centre).total_cmp(&b.at.dist2d(centre)));
        if let Some(party) = nearest {
            let odds = self.odds_words(&units, party, enemies);
            if offered("engage") && odds != "it outweighs us" && !odds.starts_with("we cannot hit") {
                let o = order("engage", &[("whom", &party.name)]);
                if !out.iter().any(|c| c.order == o) {
                    let words = format!("attacks {} ({}) with the whole group{leave}", party.name, party.composition);
                    out.push(Candidate { kind: Kind::Fight, order: o, words, party: Some((party.name.clone(), party.ids.len())), metal: standing });
                }
            }
            if offered("send_against") && party.ids.len() <= DETACH_PARTY_MAX && units.len() >= 2 {
                let n = self.detachment_for(&units, party, enemies);
                if n < units.len() {
                    let o = order("send_against", &[("whom", &party.name), ("how_many", super::standing::how_many(n))]);
                    if !out.iter().any(|c| c.order == o) {
                        let words = format!("sends {n} soldiers against {} ({}), the rest {}", party.name, party.composition, out[0].words.replacen("keeps", "keeping", 1));
                        out.push(Candidate { kind: Kind::Fight, order: o, words, party: Some((party.name.clone(), party.ids.len())), metal: n as f32 * standing / units.len() as f32 });
                    }
                }
            }
        }
        // The way back when the fight is against it: a party within the alarm reach it does not outweigh, or a
        // tenth of its metal lost in 30 s.
        let against = picture.parties.iter().filter(|p| p.at.dist2d(centre) < ALARM).any(|p| {
            let odds = self.odds_words(&units, p, enemies);
            !odds.starts_with("we outweigh") && !odds.starts_with("it cannot hit us")
        });
        let (_, lost) = group.lost_since(tick.frame - 30 * FRAMES_PER_SECOND, &self.world);
        if (against || lost >= 0.1 * (lost + standing)) && out.len() < CANDIDATES {
            for back in ["fall_back", "retreat"] {
                if offered(back) {
                    let o = order(back, &[]);
                    if !out.iter().any(|c| c.order == o) {
                        out.push(Candidate { kind: Kind::Back, order: o.clone(), words: format!("{}{leave}", order_words(&o)), party: None, metal: 0.0 });
                    }
                    break;
                }
            }
        }
        out.truncate(CANDIDATES);
        out
    }
}

/// The worlds: world 1 is every group's rule (else its course); the rest change one group's move, fights first,
/// then the ways back, then a course kept against its rule; a second taker of a small party is pruned; at most `CAP`.
pub(super) fn worlds(cands: &[(String, Vec<Candidate>)]) -> Vec<World> {
    let base: World = cands.iter().map(|(g, c)| (g.clone(), c.iter().position(|x| x.kind == Kind::Rule).unwrap_or(0))).collect();
    let taken = |world: &World, party: &str, except: usize| -> bool {
        world.iter().enumerate().any(|(gi, (_, ci))| gi != except && cands[gi].1[*ci].party.as_ref().is_some_and(|(p, _)| p == party))
    };
    let mut deviations: Vec<(u8, World)> = Vec::new();
    for (gi, (_, c)) in cands.iter().enumerate() {
        for (ci, cand) in c.iter().enumerate() {
            if ci == base[gi].1 {
                continue;
            }
            if let Some((party, size)) = &cand.party
                && *size <= SMALL_PARTY
                && taken(&base, party, gi)
            {
                continue;
            }
            let priority = match cand.kind {
                Kind::Fight | Kind::Rule => 0,
                Kind::Back => 1,
                Kind::Keep => 2,
            };
            let mut world = base.clone();
            world[gi].1 = ci;
            deviations.push((priority, world));
        }
    }
    deviations.sort_by_key(|(p, _)| *p);
    let mut out = vec![base];
    out.extend(deviations.into_iter().map(|(_, w)| w));
    out.truncate(CAP);
    out
}

/// A world's line: the moves, the courses kept, and what follows: which parties within reach are met with what
/// metal and odds, and which go unanswered (and what they are killing).
pub(super) fn consequence(world: &World, cands: &[(String, Vec<Candidate>)], parties: &[Party], centres: &[(String, bot_protocol::Vec3)], places: &[super::picture::Place]) -> String {
    let cand = |gi: usize, ci: usize| &cands[gi].1[ci];
    let moves: Vec<String> = world.iter().enumerate().filter(|(gi, (_, ci))| cand(*gi, *ci).kind != Kind::Keep).map(|(gi, (g, ci))| format!("{g} {}", cand(gi, *ci).words)).collect();
    let keeps: Vec<&str> = world.iter().enumerate().filter(|(gi, (_, ci))| cand(*gi, *ci).kind == Kind::Keep).map(|(_, (g, _))| g.as_str()).collect();
    let mut parts: Vec<String> = Vec::new();
    if !moves.is_empty() {
        parts.push(moves.join("; "));
    }
    if !keeps.is_empty() {
        parts.push(format!("{} keep{} {} course", keeps.join(", "), if keeps.len() == 1 { "s" } else { "" }, if keeps.len() == 1 { "its" } else { "their" }));
    }
    let in_reach: Vec<&Party> = parties.iter().filter(|p| !p.ids.is_empty() && centres.iter().any(|(_, c)| c.dist2d(p.at) <= RAIDER_REACH)).collect();
    let (mut met, mut unmet): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for p in in_reach {
        let takers: Vec<(usize, usize)> = world.iter().enumerate().filter(|(gi, (_, ci))| cand(*gi, *ci).party.as_ref().is_some_and(|(n, _)| *n == p.name)).map(|(gi, (_, ci))| (gi, *ci)).collect();
        let place = places.iter().map(|pl| (pl.at.dist2d(p.at), pl)).min_by(|a, b| a.0.total_cmp(&b.0)).filter(|(d, _)| *d < 400.0).map_or("in sight".to_string(), |(_, pl)| format!("at {}", pl.name));
        if takers.is_empty() {
            let killing = p.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"));
            unmet.push(format!("{} ({}, {place}{killing})", p.name, p.composition));
        } else {
            let metal: f32 = takers.iter().map(|(gi, ci)| cand(*gi, *ci).metal).sum();
            let nearest = takers.iter().filter_map(|(gi, _)| centres.iter().find(|(g, _)| *g == cands[*gi].0).map(|(_, c)| c.dist2d(p.at))).fold(f32::INFINITY, f32::min);
            met.push(format!("{} ({}, {:.0} metal, {place}) met with {metal:.0} metal: {}, {nearest:.0} away", p.name, p.composition, p.metal, odds_by_metal(metal, p.metal)));
        }
    }
    if !met.is_empty() {
        parts.push(format!("Met: {}", met.join("; ")));
    }
    if !unmet.is_empty() {
        parts.push(format!("Unanswered: {}", unmet.join("; ")));
    }
    format!("{}.", parts.join(". "))
}

/// The one question: a Choice over the worlds' lines.
pub(super) fn question(names: &[String], lines: &[String]) -> Question {
    let instructions = json!(format!(
        "Given `actors`, `enemy`, `player` and the player's `instructions`, which joint course for {} is best this second? Each option is one world: what every group does and what follows from it (which enemy parties within reach are met, with what metal and odds, which go unanswered, and what each move gives up). w1 is what the player's standing rules say; every other world changes one group's move. The instructions were written before this picture: where they name a place, a party or a rule, follow them; where the situation has changed, pick the world they would call for.",
        names.join(", ")
    ));
    Question::Choice { instructions, criteria: lines.iter().enumerate().map(|(i, l)| (format!("w{}", i + 1), json!(l))).collect() }
}

/// The picked world, played: each varying group's chosen candidate becomes its `do` answer at the pick's confidence
/// and its forced order (the hands' `standing` path, rule `worlds`, so no switch margin applies). Returns the pick.
pub(super) fn resolve(menus: &mut [Menu], answers: &mut BTreeMap<String, Answer>) -> Option<(usize, f64)> {
    let wi = menus.iter().position(|m| m.name == "worlds")?;
    let (choice, confidence) = match answers.get("worlds.pick") {
        Some(Answer::Choice { choice, confidence, .. }) => (choice.clone(), *confidence),
        _ => return None,
    };
    let index: usize = choice.strip_prefix('w')?.parse::<usize>().ok()?.checked_sub(1)?;
    let world = menus[wi].worlds.get(index)?.clone();
    for (g, ci) in world {
        let Some(menu) = menus.iter_mut().find(|m| m.name == g) else { continue };
        let Some(cand) = menu.worlds_candidates.get(ci).cloned() else { continue };
        menu.standing = Some((cand.order.clone(), "worlds".to_string()));
        menu.worlds_played = true;
        let sure = |c: &str| Answer::Choice { choice: c.to_string(), probabilities: BTreeMap::from([(c.to_string(), 1.0)]), confidence };
        answers.insert(format!("{g}.do"), sure(&cand.order.choice));
        answers.insert(format!("{g}.standing"), sure("rule"));
    }
    Some((index, confidence))
}

/// The log line's view of the candidates.
pub(super) fn log_candidates(cands: &[(String, Vec<Candidate>)]) -> Value {
    json!(cands.iter().map(|(g, c)| (g.clone(), json!(c.iter().map(|x| json!({ "kind": format!("{:?}", x.kind).to_lowercase(), "do": x.order.choice, "params": x.order.params, "words": x.words })).collect::<Vec<_>>()))).collect::<BTreeMap<_, _>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(kind: Kind, choice: &str, party: Option<(&str, usize)>) -> Candidate {
        Candidate { kind, order: order(choice, &[]), words: choice.to_string(), party: party.map(|(p, n)| (p.to_string(), n)), metal: 100.0 }
    }

    #[test]
    fn world_one_is_the_rules_and_the_rest_change_one_group_fights_first_under_the_cap() {
        let cands = vec![
            ("group_A".to_string(), vec![cand(Kind::Keep, "hold", None), cand(Kind::Rule, "engage", Some(("party_1", 1))), cand(Kind::Back, "retreat", None)]),
            ("group_B".to_string(), vec![cand(Kind::Keep, "hold", None), cand(Kind::Fight, "engage", Some(("party_1", 1))), cand(Kind::Fight, "send_against", Some(("party_9", 4)))]),
        ];
        let w = worlds(&cands);
        assert_eq!(w[0], vec![("group_A".to_string(), 1), ("group_B".to_string(), 0)]);
        // group_B's engage at party_1 is pruned: group_A's rule takes that lone raider; its send_against at party_9 stays.
        assert!(!w.iter().any(|x| x[1].1 == 1));
        assert_eq!(w[1], vec![("group_A".to_string(), 1), ("group_B".to_string(), 2)]);
        // The way back comes after the fights, the kept course last.
        assert_eq!(w[2], vec![("group_A".to_string(), 2), ("group_B".to_string(), 0)]);
        assert_eq!(w[3], vec![("group_A".to_string(), 0), ("group_B".to_string(), 0)]);
        assert!(w.len() <= CAP);
    }

    #[test]
    fn the_pick_becomes_each_groups_forced_order() {
        let cands = vec![("group_A".to_string(), vec![cand(Kind::Keep, "hold", None), cand(Kind::Fight, "engage", Some(("party_1", 1)))])];
        let w = worlds(&cands);
        let mut menus = vec![
            Menu::test_group("group_A", cands[0].1.clone()),
            Menu::test_worlds(w.clone()),
        ];
        let mut answers = BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w2".into(), probabilities: BTreeMap::new(), confidence: 0.8 })]);
        assert_eq!(resolve(&mut menus, &mut answers), Some((1, 0.8)));
        assert!(menus[0].worlds_played);
        assert_eq!(menus[0].standing.as_ref().map(|(o, r)| (o.choice.as_str(), r.as_str())), Some(("engage", "worlds")));
        assert!(matches!(answers.get("group_A.do"), Some(Answer::Choice { choice, .. }) if choice == "engage"));
    }
}
