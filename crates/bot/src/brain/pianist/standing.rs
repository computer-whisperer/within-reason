//! Standing orders (`docs/design/2026-09-25-standing-orders.md`): per-actor rules with a parameter, held in the hands
//! and evaluated in code every second against the picture, so the actor is answered without a Jev ask when a rule
//! fires. They come from two places: the player's packet, decompressed by Jev once per packet into the vocabulary
//! here (K-jev-a-packet-decompresses-to-standing-orders, `run/decompress.py` is the offline twin of the questions),
//! and the player's `standing` tool, which sets them directly and outranks the packet for the same (actor, rule).
//! In the pass (`plan.rs`, `threats.rs`) a rule prunes states or makes one the default; nothing here orders
//! (`docs/design/2026-09-26-one-pass.md` §6).
use std::collections::{BTreeMap, BTreeSet};

use jev::{Answer, Question};
use serde_json::{Value, json};

/// The raider rules and the threat states reach this far from the group (the detectors' reach).
pub(super) const RAIDER_REACH: f32 = 1200.0;
/// A party this close to a place a group never goes stands at it: no state is offered against it, and a chase that
/// reaches it ends there.
pub(super) const NEVER_REACH: f32 = 400.0;
/// A hedged extraction answer (p_top under this) sets nothing.
const SURE: f64 = 0.6;

/// The vocabulary: rule name to the values it takes (`place`, `places`, `party` and `size` are checked against the
/// picture or the number grammar).
pub(crate) const GROUP_RULES: &[(&str, &[&str])] = &[
    ("station", &["place"]),
    ("station_mode", &["walk", "advance"]),
    ("raiders_lone", &["whole_group", "detachment", "detachment:N", "ignore"]),
    ("raiders_party", &["whole_group", "detachment", "detachment:N", "ignore"]),
    ("no_chase", &["yes"]),
    ("no_detachments", &["yes"]),
    ("hold_line", &["yes"]),
    ("fall_back_to", &["place"]),
    ("engage_party", &["party"]),
    ("join", &["group"]),
    ("never", &["places"]),
];
pub(crate) const BUILDER_RULES: &[(&str, &[&str])] = &[
    ("job", &["help_factory", "expand", "follow_list"]),
    ("attack_raiders", &["yes"]),
    ("retreat_when_enemy_near", &["yes"]),
    ("solar", &["only_when_stalling", "never", "freely"]),
    ("turrets", &["beside_each_outer_extractor", "beside_each_extractor", "none"]),
    ("no_chase", &["yes"]),
    ("never", &["places"]),
];

pub(crate) type Rules = BTreeMap<String, String>;

#[derive(Default)]
pub(crate) struct Standing {
    /// From the packet, replaced whole at each decompression.
    packet: BTreeMap<String, Rules>,
    /// From the `standing` tool, kept until cleared; outranks the packet's for the same (actor, rule).
    tool: BTreeMap<String, Rules>,
    /// The frame of the packet the packet orders came from.
    pub packet_frame: i32,
    /// The unit types a group had when its tool orders were set, and whether the change was said: standing-1's
    /// `raiders ignore`, set for two Rovers, held the Blitzes that joined the group while five extractors died.
    pub tool_seen: BTreeMap<String, (BTreeSet<String>, bool)>,
}

fn is_group(actor: &str) -> bool {
    actor.starts_with("group_")
}

fn valid_value(rule: &str, value: &str, allowed: &[&str]) -> bool {
    allowed.iter().any(|a| match *a {
        "place" | "places" | "party" | "group" => !value.is_empty(),
        "detachment:N" => value.strip_prefix("detachment:").is_some_and(|n| matches!(n, "1" | "2" | "4" | "8" | "half")),
        other => other == value,
    }) || (rule == "never" && !value.is_empty())
}

impl Standing {
    /// The rules in force for an actor: the packet's for it (a constructor's own after the `constructors` class),
    /// then the tool's over them.
    pub(crate) fn rules_for(&self, actor: &str) -> Rules {
        let mut out = Rules::new();
        let class = if actor.starts_with("constructor_") {
            Some("constructors")
        } else if actor.starts_with("commander_t") {
            // A packet paragraph headed `commander:` is every seat's commander; `commander_t2:` that seat's only.
            Some("commander")
        } else {
            None
        };
        for source in [&self.packet, &self.tool] {
            if let Some(c) = class
                && let Some(rules) = source.get(c)
            {
                out.extend(rules.clone());
            }
            if let Some(rules) = source.get(actor) {
                out.extend(rules.clone());
            }
        }
        out
    }

    /// Every word of the tool's rule values: the places and parties the player named through the tool, for the
    /// picture to list (worlds-2: `station: spot_27` set by the tool was "not a place in the picture" fifteen times
    /// over, the picture listing only the spots the instructions name).
    pub(crate) fn tool_words(&self) -> Vec<String> {
        self.tool.values().flat_map(|rules| rules.values()).flat_map(|v| v.split_whitespace().map(str::to_string)).collect()
    }

    pub(crate) fn never_places(&self, actor: &str) -> Vec<String> {
        self.rules_for(actor).get("never").map(|v| v.split_whitespace().map(str::to_string).collect()).unwrap_or_default()
    }

    /// The `standing` line of an actor's picture entry.
    pub(crate) fn words(&self, actor: &str) -> Option<String> {
        let rules = self.rules_for(actor);
        if rules.is_empty() {
            return None;
        }
        let tool = self.tool.get(actor).map(|r| r.keys().cloned().collect::<BTreeSet<_>>()).unwrap_or_default();
        Some(rules.iter().map(|(k, v)| format!("{k} {v}{}", if tool.contains(k) { " (tool)" } else { "" })).collect::<Vec<_>>().join("; "))
    }

    pub(crate) fn set_packet(&mut self, orders: BTreeMap<String, Rules>, frame: i32) {
        self.packet = orders;
        self.packet_frame = frame;
    }

    /// Sets tool orders after checking each against the vocabulary and the picture's places and parties.
    pub(crate) fn set_tool(&mut self, actor: &str, rules: &Value, places: &[String], parties: &[String]) -> Result<usize, String> {
        let checked = Self::check_tool(actor, rules, places, parties)?;
        self.tool_seen.remove(actor);
        let entry = self.tool.entry(actor.to_string()).or_default();
        let n = checked.len();
        for (rule, value) in checked {
            if value.is_empty() {
                entry.remove(&rule);
            } else {
                entry.insert(rule, value);
            }
        }
        if entry.is_empty() {
            self.tool.remove(actor);
        }
        Ok(n)
    }

    /// One actor's tool rules checked against the vocabulary and the picture's places and parties: the rules to set
    /// (an empty value clears one), or why the actor's orders are refused whole. The `standing` tool runs this in
    /// the player's turn so a refusal is answered at once (worlds-2, 17:07: "a set with an unknown place is refused
    /// whole, silently in my view"); the hands run it again at their next second.
    pub(crate) fn check_tool(actor: &str, rules: &Value, places: &[String], parties: &[String]) -> Result<Rules, String> {
        let Some(map) = rules.as_object() else { return Err(format!("{actor}: rules must be an object of rule to value")) };
        let vocabulary = if is_group(actor) { GROUP_RULES } else { BUILDER_RULES };
        let mut checked = Rules::new();
        for (rule, value) in map {
            let Some((_, allowed)) = vocabulary.iter().find(|(r, _)| r == rule) else {
                return Err(format!("{actor}: {rule} is not a rule ({})", vocabulary.iter().map(|(r, _)| *r).collect::<Vec<_>>().join(", ")));
            };
            // null, false and "no" clear the rule (standing-1: the player wrote `retreat_when_enemy_near: "no"`
            // three turns running and was refused each time).
            let value = match value {
                Value::Null | Value::Bool(false) => {
                    checked.insert(rule.clone(), String::new());
                    continue;
                }
                Value::String(s) if allowed == &["yes"] && matches!(s.trim(), "no" | "off" | "false") => {
                    checked.insert(rule.clone(), String::new());
                    continue;
                }
                Value::String(s) => s.trim().to_string(),
                Value::Bool(true) => "yes".to_string(),
                Value::Number(n) => n.to_string(),
                other => return Err(format!("{actor}: {rule} takes a string, not {other}")),
            };
            if !valid_value(rule, &value, allowed) {
                return Err(format!("{actor}: {rule} takes {}, not {value}", allowed.join(" | ")));
            }
            if allowed.contains(&"place") && !places.iter().any(|p| *p == value) {
                return Err(format!("{actor}: {value} is not a place in the picture"));
            }
            if rule == "never" && let Some(bad) = value.split_whitespace().find(|p| !places.iter().any(|q| q == p)) {
                return Err(format!("{actor}: {bad} is not a place in the picture"));
            }
            if allowed.contains(&"party") && !parties.iter().any(|p| *p == value) {
                return Err(format!("{actor}: {value} is not a party in the picture"));
            }
            if allowed.contains(&"group") && (!value.starts_with("group_") || value == actor) {
                return Err(format!("{actor}: {rule} takes another group's name (group_X), not {value}"));
            }
            checked.insert(rule.clone(), value);
        }
        Ok(checked)
    }

    pub(crate) fn clear_tool(&mut self, actors: Option<&[String]>) -> usize {
        match actors {
            None => {
                self.tool_seen.clear();
                std::mem::take(&mut self.tool).len()
            }
            Some(list) => list.iter().filter(|a| {
                self.tool_seen.remove(*a);
                self.tool.remove(*a).is_some()
            }).count(),
        }
    }

    /// A group with tool orders whose unit types have changed since they were set: said once, as a line for the
    /// player's report (None when nothing changed or it was said already).
    pub(crate) fn composition_changed(&mut self, actor: &str, types: BTreeSet<String>) -> Option<String> {
        if !self.tool.contains_key(actor) {
            return None;
        }
        match self.tool_seen.get_mut(actor) {
            None => {
                self.tool_seen.insert(actor.to_string(), (types, false));
                None
            }
            Some((then, warned)) => {
                if *warned || *then == types || then.is_empty() {
                    return None;
                }
                *warned = true;
                Some(format!("{actor}'s tool orders were set when it was {}; it is {} now: clear or reset them if they were meant for what it was", then.iter().cloned().collect::<Vec<_>>().join(", "), types.iter().cloned().collect::<Vec<_>>().join(", ")))
            }
        }
    }

    /// What is in force, for the tool's answer and the report.
    pub(crate) fn in_force(&self) -> String {
        let actors: BTreeSet<&String> = self.packet.keys().chain(self.tool.keys()).collect();
        if actors.is_empty() {
            return "no standing orders are in force".to_string();
        }
        actors.iter().filter_map(|a| self.words(a).map(|w| format!("{a}: {w}"))).collect::<Vec<_>>().join("\n")
    }

    pub(crate) fn counts(&self) -> (usize, usize) {
        (self.packet.values().map(Rules::len).sum(), self.tool.values().map(Rules::len).sum())
    }
}

// The decompression: the questions and the reading of the answers.

fn group_names(packet: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (i, _) in packet.match_indices("group_") {
        let rest = &packet[i + 6..];
        let mut name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        // The seat's tag is part of the name (`group_A_t2` in a game with several seats of ours); the packet's
        // `group_A_t2:` was read as `group_A` and reached nobody (bluegecko-3v1-comet-catcher).
        if let Some(after) = rest[name.len()..].strip_prefix("_t") {
            let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                name.push_str("_t");
                name.push_str(&digits);
            }
        }
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            let full = format!("group_{name}");
            if !out.contains(&full) {
                out.push(full);
            }
        }
    }
    out
}

/// The paragraphs of the packet that name the actor (the constructors: any naming a constructor).
fn paragraphs_about(packet: &str, actor: &str) -> String {
    let needle = if actor == "constructors" { "constructor" } else { actor };
    // `commander` is not `commander_t2`, and `group_A` not `group_A_t2`: the name ends where a tag would begin.
    let names = |p: &str| p.match_indices(needle).any(|(i, _)| !p[i + needle.len()..].starts_with("_t") || actor.contains("_t"));
    packet.split("\n\n").filter(|p| if actor == "constructors" { p.to_lowercase().contains(needle) } else { names(p) }).collect::<Vec<_>>().join("\n\n")
}

/// The builders a packet speaks about: `commander`, every `commander_t<N>` it names, and `constructors`.
fn builder_names(packet: &str) -> Vec<String> {
    let mut out = vec!["commander".to_string()];
    for (i, _) in packet.match_indices("commander_t") {
        let digits: String = packet[i + 11..].chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            let name = format!("commander_t{digits}");
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out.push("constructors".to_string());
    out
}

fn places_in(text: &str, places: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in places {
        if text.match_indices(p.as_str()).any(|(i, _)| {
            let before = text[..i].chars().next_back();
            let after = text[i + p.len()..].chars().next();
            !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        }) && !out.contains(p)
        {
            out.push(p.clone());
        }
    }
    out
}

fn parties_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in text.match_indices("party_") {
        let digits: String = text[i + 6..].chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            // A seat's tag stays on the name (`party_3_t2` in a game with several seats of ours).
            let rest = &text[i + 6 + digits.len()..];
            let tag: String = match rest.strip_prefix("_t") {
                Some(after) => {
                    let n: String = after.chars().take_while(char::is_ascii_digit).collect();
                    if n.is_empty() { String::new() } else { format!("_t{n}") }
                }
                None => String::new(),
            };
            let name = format!("party_{digits}{tag}");
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out
}

/// The state the extraction questions read: each actor's own paragraphs under `paragraphs`, since a question over
/// the whole packet took another actor's rule (standing-1: the constructors' turrets given to the commander at 0.91).
pub(crate) fn extraction_state(packet: &str) -> Value {
    let mut paragraphs = serde_json::Map::new();
    for g in group_names(packet) {
        paragraphs.insert(g.clone(), json!(paragraphs_about(packet, &g)));
    }
    for b in builder_names(packet) {
        let b = b.as_str();
        let paras = paragraphs_about(packet, b);
        if !paras.is_empty() {
            paragraphs.insert(b.to_string(), json!(paras));
        }
    }
    json!({ "paragraphs": paragraphs })
}

/// One extraction question per (actor, rule) for the packet; `places` are the picture's place names (spots,
/// passages, home, the player's marks). Each question reads its actor's field of `extraction_state`.
pub(crate) fn extraction_questions(packet: &str, places: &[String]) -> BTreeMap<String, Question> {
    let mut qs: BTreeMap<String, Question> = BTreeMap::new();
    let place_options = |names: &[String]| -> Vec<(String, Value)> { names.iter().map(|p| (p.clone(), json!(format!("the place {p}")))).collect() };
    for g in group_names(packet) {
        let paras = paragraphs_about(packet, &g);
        let named = places_in(&paras, places);
        let mut station = place_options(&named);
        station.push(("none".into(), json!(format!("the packet names no place for {g} to stand"))));
        qs.insert(format!("{g}.station"), Question::choice(format!("Read `paragraphs.{g}`, the packet's lines about {g}. Where do they tell {g} to stand, gather, hold or be stationed? `none` when they name no such place for {g}."), station));
        qs.insert(format!("{g}.station_mode"), Question::choice(format!("Read `paragraphs.{g}`. When {g} goes to its place, does it advance fighting on the way (fight_to, attack), or walk?"), [("advance", "advances, fighting on the way"), ("walk", "walks"), ("not_said", "not said, or it is not sent anywhere")]));
        for (rule, what) in [("raiders_lone", "a single enemy raider (one scout car, Tick or Pawn)"), ("raiders_party", "a small enemy raider party (two to six)")] {
            qs.insert(
                format!("{g}.{rule}"),
                Question::choice(
                    format!("Read `paragraphs.{g}`. When {what} appears at one of our extractors, turrets or constructors near {g}, what do they tell {g} to do?"),
                    [("whole_group", format!("{g} attacks it (engages, kills it on sight) as a group")), ("detachment", format!("{g} sends a detachment (send_against, a few soldiers, one soldier) and the rest stay")), ("ignore", format!("the packet tells {g} not to answer it (hold, keep its walk, never chase)")), ("not_said", format!("the packet says nothing about this for {g}"))],
                ),
            );
        }
        qs.insert(format!("{g}.detachment_size"), Question::choice(format!("Read `paragraphs.{g}`. When {g} sends a detachment (send_against) against a raider, how many soldiers do they say go with it?"), [("1", "one soldier (send_against 1)"), ("2", "two soldiers (send_against 2)"), ("4", "four soldiers (send_against 4)"), ("8", "eight soldiers (send_against 8)"), ("half", "half the group"), ("not_said", "no number is given, or no detachment is ordered")]));
        qs.insert(format!("{g}.no_chase"), Question::noul(format!("Read `paragraphs.{g}`. Do they say {g} never chases raiders far, or never leaves its place after them?")));
        qs.insert(format!("{g}.no_detachments"), Question::noul(format!("Read `paragraphs.{g}`. Do they say {g} sends no detachments, or never splits?")));
        qs.insert(format!("{g}.hold_line"), Question::noul(format!("Read `paragraphs.{g}`. Do they tell {g} not to fall back or retreat while the fight is even or we outweigh the enemy?")));
        let mut back = place_options(&named);
        back.push(("home".into(), json!("home")));
        back.push(("not_said".into(), json!("not said")));
        qs.insert(format!("{g}.fall_back_to"), Question::choice(format!("Read `paragraphs.{g}`. Where do they tell {g} to fall back to when a party outweighs it, if they say?"), back));
        let parties = parties_in(&paras);
        if !parties.is_empty() {
            let mut opts: Vec<(String, Value)> = parties.iter().map(|p| (p.clone(), json!(format!("the enemy party {p}")))).collect();
            opts.push(("none".into(), json!("no party is named for it to attack")));
            qs.insert(format!("{g}.engage_party"), Question::choice(format!("Read `paragraphs.{g}`. Which enemy party, if any, is {g} told to engage, attack or kill?"), opts));
        }
        for p in &named {
            qs.insert(format!("{g}.never_{p}"), Question::noul(format!("Read `paragraphs.{g}`. Do they say {g} never goes to, stands at or advances to {p}, or that no soldier stands at {p}?")));
        }
    }
    for b in builder_names(packet) {
        let b = b.as_str();
        let paras = paragraphs_about(packet, b);
        if paras.is_empty() {
            continue;
        }
        let who = if b.starts_with("commander") { "the commander" } else { "a constructor" };
        let named = places_in(&paras, places);
        qs.insert(format!("{b}.job"), Question::choice(format!("Read `paragraphs.{b}`, the packet's lines about {who}. What is {who}'s standing job when it has nothing else to do?"), [("help_factory", "help (assist, guard) the factory or plant"), ("follow_list", "follow its build list from the player"), ("expand", "take free metal spots, build extractors"), ("not_said", "the packet does not say")]));
        qs.insert(format!("{b}.attack_raiders"), Question::noul(format!("Read `paragraphs.{b}`. Do they tell {who} to attack a raider party at one of our buildings near it?")));
        qs.insert(format!("{b}.no_chase"), Question::noul(format!("Read `paragraphs.{b}`. Do they say {who} never chases scout cars or Ticks?")));
        qs.insert(format!("{b}.solar"), Question::choice(format!("Read `paragraphs.{b}`. When may {who} build a solar collector or generator?"), [("only_when_stalling", "only when energy reads STALLING or is low"), ("never", "never, no more solars"), ("freely", "as it sees fit, or a number of them"), ("not_said", "the packet does not say")]));
        qs.insert(format!("{b}.turrets"), Question::choice(format!("Read `paragraphs.{b}`. Where do they tell {who} to build light turrets?"), [("beside_each_outer_extractor", "beside each outer or far extractor, or each pair"), ("beside_each_extractor", "beside every extractor"), ("none", "no turrets, or it forbids them"), ("not_said", "the packet does not say")]));
        qs.insert(format!("{b}.retreat_when_enemy_near"), Question::noul(format!("Read `paragraphs.{b}`. Do they tell {who} to walk back toward home or the commander when enemy soldiers are near?")));
        for p in &named {
            qs.insert(format!("{b}.never_{p}"), Question::noul(format!("Read `paragraphs.{b}`. Do they say {who} never goes to, walks past or builds at {p}?")));
        }
    }
    qs
}

/// The orders read from the extraction answers: a hedged answer (p_top under `SURE`) sets nothing, `not_said` and
/// `none` set nothing, a detachment takes its size, `never_<place>` answers gather into `never`.
pub(crate) fn orders_from(answers: &BTreeMap<String, Answer>) -> BTreeMap<String, Rules> {
    let mut out: BTreeMap<String, Rules> = BTreeMap::new();
    let sure_choice = |a: &Answer| match a {
        Answer::Choice { choice, probabilities, .. } if probabilities.get(choice).copied().unwrap_or(0.0) >= SURE => Some(choice.clone()),
        _ => None,
    };
    let sure_yes = |a: &Answer| matches!(a, Answer::Noul { noul } if *noul >= SURE);
    let mut sizes: BTreeMap<String, String> = BTreeMap::new();
    let mut nevers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (qid, answer) in answers {
        let Some((actor, rule)) = qid.split_once('.') else { continue };
        if let Some(place) = rule.strip_prefix("never_") {
            if sure_yes(answer) {
                nevers.entry(actor.to_string()).or_default().push(place.to_string());
            }
            continue;
        }
        match rule {
            "detachment_size" => {
                if let Some(c) = sure_choice(answer).filter(|c| c != "not_said") {
                    sizes.insert(actor.to_string(), c);
                }
            }
            "no_chase" | "no_detachments" | "hold_line" | "attack_raiders" | "retreat_when_enemy_near" => {
                if sure_yes(answer) {
                    out.entry(actor.to_string()).or_default().insert(rule.to_string(), "yes".to_string());
                }
            }
            _ => {
                if let Some(c) = sure_choice(answer).filter(|c| c != "not_said" && c != "none") {
                    out.entry(actor.to_string()).or_default().insert(rule.to_string(), c);
                }
            }
        }
    }
    for (actor, size) in sizes {
        if let Some(rules) = out.get_mut(&actor) {
            for rule in ["raiders_lone", "raiders_party"] {
                if rules.get(rule).is_some_and(|v| v == "detachment") {
                    rules.insert(rule.to_string(), format!("detachment:{size}"));
                }
            }
        }
    }
    for (actor, places) in nevers {
        out.entry(actor).or_default().insert("never".to_string(), places.join(" "));
    }
    out.retain(|_, rules| !rules.is_empty());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_seats_paragraphs_reach_their_own_actors() {
        let packet = "commander: never chases scout cars.\n\ncommander_t2: helps the plant.\n\ngroup_A_t2: stands at spot_61.\n\ngroup_B: stands at spot_24.";
        assert_eq!(group_names(packet), vec!["group_A_t2".to_string(), "group_B".to_string()]);
        assert_eq!(builder_names(packet), vec!["commander".to_string(), "commander_t2".to_string(), "constructors".to_string()]);
        let state = extraction_state(packet);
        assert!(state["paragraphs"]["commander"].as_str().unwrap().contains("scout cars") && !state["paragraphs"]["commander"].as_str().unwrap().contains("helps the plant"));
        assert!(state["paragraphs"]["commander_t2"].as_str().unwrap().contains("helps the plant"));
        assert!(state["paragraphs"]["group_A_t2"].as_str().unwrap().contains("spot_61"));
        let ids: Vec<String> = extraction_questions(packet, &["spot_61".to_string(), "spot_24".to_string()]).keys().cloned().collect();
        assert!(ids.iter().any(|i| i.starts_with("group_A_t2.")) && ids.iter().any(|i| i.starts_with("commander_t2.")), "{ids:?}");
        let mut standing = Standing::default();
        let mut orders: BTreeMap<String, Rules> = BTreeMap::new();
        orders.insert("commander".into(), BTreeMap::from([("no_chase".to_string(), "yes".to_string())]));
        orders.insert("commander_t2".into(), BTreeMap::from([("job".to_string(), "help_factory".to_string())]));
        standing.set_packet(orders, 1);
        let rules = standing.rules_for("commander_t2");
        assert_eq!(rules.get("no_chase").map(String::as_str), Some("yes"), "the plain commander paragraph is every seat's");
        assert_eq!(rules.get("job").map(String::as_str), Some("help_factory"));
    }

    #[test]
    fn party_names_keep_their_seat_tag() {
        assert_eq!(parties_in("engage party_3 and party_12_t2, not party_3_t2 again"), vec!["party_3".to_string(), "party_12_t2".to_string(), "party_3_t2".to_string()]);
    }

    const PACKET: &str = "commander: helps the plant; it never chases scout cars or Ticks. It never goes to spot_38.\n\ngroup_B: stands at spot_61. A lone Tick at an extractor is met by two Blitzes (send_against 2). It never sends detachments elsewhere.\n\ngroup_C: kills party_12 at spot_24.";

    #[test]
    fn the_questions_cover_every_named_actor_and_place() {
        let places = ["spot_61", "spot_38", "spot_24", "home"].map(String::from);
        let qs = extraction_questions(PACKET, &places);
        let ids: Vec<&String> = qs.keys().collect();
        assert!(ids.contains(&&"group_B.station".to_string()) && ids.contains(&&"group_B.never_spot_61".to_string()));
        assert!(ids.contains(&&"group_C.engage_party".to_string()) && ids.contains(&&"commander.never_spot_38".to_string()));
        assert!(!ids.contains(&&"group_B.never_spot_38".to_string()));
        assert!(!ids.iter().any(|id| id.starts_with("constructors.")));
    }

    #[test]
    fn the_answers_become_orders_and_hedged_ones_do_not() {
        let choice = |c: &str, p: f64| Answer::Choice { choice: c.into(), probabilities: BTreeMap::from([(c.to_string(), p)]), confidence: p };
        let answers = BTreeMap::from([
            ("group_B.station".to_string(), choice("spot_61", 0.95)),
            ("group_B.raiders_lone".to_string(), choice("detachment", 0.9)),
            ("group_B.detachment_size".to_string(), choice("2", 0.9)),
            ("group_B.raiders_party".to_string(), choice("whole_group", 0.4)),
            ("group_B.no_chase".to_string(), Answer::Noul { noul: 0.8 }),
            ("group_B.never_spot_38".to_string(), Answer::Noul { noul: 0.9 }),
            ("group_B.never_spot_24".to_string(), Answer::Noul { noul: 0.2 }),
            ("commander.job".to_string(), choice("not_said", 0.9)),
        ]);
        let orders = orders_from(&answers);
        let b = &orders["group_B"];
        assert_eq!(b["station"], "spot_61");
        assert_eq!(b["raiders_lone"], "detachment:2");
        assert!(!b.contains_key("raiders_party"));
        assert_eq!(b["no_chase"], "yes");
        assert_eq!(b["never"], "spot_38");
        assert!(!orders.contains_key("commander"));
    }

    #[test]
    fn a_tool_check_names_the_unknown_place_and_passes_the_rest() {
        let places = ["spot_61", "spot_9"].map(String::from);
        let refused = Standing::check_tool("group_C", &json!({ "never": "shelling spot_9", "station": "spot_61" }), &places, &[]).unwrap_err();
        assert_eq!(refused, "group_C: shelling is not a place in the picture");
        let checked = Standing::check_tool("group_C", &json!({ "never": "spot_9", "station": "spot_61", "raiders_lone": null }), &places, &[]).unwrap();
        assert_eq!(checked, Rules::from([("never".to_string(), "spot_9".to_string()), ("station".to_string(), "spot_61".to_string()), ("raiders_lone".to_string(), String::new())]));
    }

    #[test]
    fn tool_orders_are_checked_and_outrank_the_packet() {
        let mut s = Standing::default();
        s.set_packet(BTreeMap::from([("group_B".to_string(), Rules::from([("station".to_string(), "spot_61".to_string()), ("no_chase".to_string(), "yes".to_string())]))]), 100);
        let places = ["spot_61", "spot_9"].map(String::from);
        assert!(s.set_tool("group_B", &json!({ "station": "spot_7" }), &places, &[]).is_err());
        assert!(s.set_tool("group_B", &json!({ "raiders_lone": "detachment:3" }), &places, &[]).is_err());
        assert!(s.set_tool("group_B", &json!({ "bogus": "yes" }), &places, &[]).is_err());
        assert_eq!(s.set_tool("group_B", &json!({ "station": "spot_9", "raiders_lone": "detachment:2" }), &places, &[]), Ok(2));
        let r = s.rules_for("group_B");
        assert_eq!(r["station"], "spot_9");
        assert_eq!(r["no_chase"], "yes");
        assert!(s.words("group_B").unwrap().contains("station spot_9 (tool)"));
        s.set_tool("constructors", &json!({ "job": "expand" }), &places, &[]).unwrap();
        assert_eq!(s.rules_for("constructor_4")["job"], "expand");
        assert_eq!(s.clear_tool(Some(&["group_B".to_string()])), 1);
        assert_eq!(s.rules_for("group_B")["station"], "spot_61");
        s.set_tool("group_B", &json!({ "no_detachments": true, "hold_line": "yes" }), &places, &[]).unwrap();
        assert_eq!(s.rules_for("group_B")["hold_line"], "yes");
        s.set_tool("group_B", &json!({ "hold_line": "no", "no_detachments": false }), &places, &[]).unwrap();
        assert!(!s.rules_for("group_B").contains_key("hold_line"));
    }
}
