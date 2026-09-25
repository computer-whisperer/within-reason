//! Standing orders (`docs/design/2026-09-25-standing-orders.md`): per-actor rules with a parameter, held in the hands
//! and evaluated in code every second against the picture, so the actor is answered without a Jev ask when a rule
//! fires. They come from two places: the player's packet, decompressed by Jev once per packet into the vocabulary
//! here (K-jev-a-packet-decompresses-to-standing-orders, `run/decompress.py` is the offline twin of the questions),
//! and the player's `standing` tool, which sets them directly and outranks the packet for the same (actor, rule).
//! The executor (`Brain::standing_order`) gives at most one order per actor per second; `standing_pass` in `mod.rs`
//! applies it through the hands as an answer at probability one.
use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{OwnUnit, Tick, Vec3};
use jev::{Answer, Question};
use serde_json::{Value, json};

use super::super::Brain;
use super::super::roster::Kit;
use super::menu::{ALARM, Actor, DETACH_PARTY_MAX, Menu};
use super::picture::{Party, Picture};
use super::{GroupTask, Task};

/// One actor's order for this second, from a standing rule: the option and its parameters (`where`, `whom`,
/// `how_many`, `where_scout`, `where_extractor`).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Order {
    pub choice: String,
    pub params: BTreeMap<String, String>,
}

/// A raider party this close to a structure of ours stands "at" it.
const AT_STRUCTURE: f32 = 400.0;
/// The raider rules reach this far from the group (the detectors' reach).
pub(super) const RAIDER_REACH: f32 = 1200.0;
/// A group holding farther than this from its station walks there.
const STATION_SLACK: f32 = 300.0;
/// A turret this close to an extractor covers it.
const TURRET_COVER: f32 = 200.0;
/// An extractor beyond this from home is an outer one.
const OUTER: f32 = 500.0;
/// A builder's turret rule reaches this far.
const TURRET_REACH: f32 = 1200.0;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// The orders are kept and logged; the executor never plays them.
    Off,
    /// The executor takes the actors it decides for.
    On,
    /// The executor's order goes to Jev beside the actor's menu as a `standing` question (rule / near / other /
    /// panic), and plays only on `rule`.
    Filter,
    /// The groups' rules generate candidates and one question over the joined worlds decides (`worlds.rs`,
    /// `docs/design/2026-09-25-one-decider.md` §4); the builders' rules play as in `On`. The default.
    Worlds,
}

impl Mode {
    /// `WITHIN_REASON_STANDING`: off, on (the default) or filter.
    pub(crate) fn from_env() -> Mode {
        match std::env::var("WITHIN_REASON_STANDING").ok().as_deref() {
            Some("off") => Mode::Off,
            Some("filter") => Mode::Filter,
            Some("on") => Mode::On,
            _ => Mode::Worlds,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Mode::Off => "off",
            Mode::On => "on",
            Mode::Filter => "filter",
            Mode::Worlds => "worlds",
        }
    }
}

pub(crate) type Rules = BTreeMap<String, String>;

#[derive(Default)]
pub(crate) struct Standing {
    /// From the packet, replaced whole at each decompression.
    packet: BTreeMap<String, Rules>,
    /// From the `standing` tool, kept until cleared; outranks the packet's for the same (actor, rule).
    tool: BTreeMap<String, Rules>,
    /// The frame of the packet the packet orders came from.
    pub packet_frame: i32,
    /// Since the last report: "actor rule" to how many times it fired, and the asks it saved.
    pub fired: BTreeMap<String, u32>,
    pub saved: u32,
    /// The filter's verdicts since the last report: rule / near / other / panic.
    pub verdicts: BTreeMap<String, u32>,
    /// The unit types a group had when its tool orders were set, and whether the change was said: standing-1's
    /// `raiders ignore`, set for two Rovers, held the Blitzes that joined the group while five extractors died.
    pub tool_seen: BTreeMap<String, (BTreeSet<String>, bool)>,
    /// When `retreat_when_enemy_near` last sent each builder home: for `RETREAT_HOLD` frames after, its building
    /// rules (`turrets`, `job`, `solar`) do not fire (standing-2: a constructor sent home by the retreat rule was
    /// sent back to the extractor by the turret rule the next second, 11 times).
    pub retreated: BTreeMap<String, i32>,
}

/// A builder sent home by the retreat rule stays sent for this long.
const RETREAT_HOLD: i32 = 30 * super::super::FRAMES_PER_SECOND;

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
        let class = if actor.starts_with("constructor_") { Some("constructors") } else { None };
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

    pub(crate) fn has_rules(&self, actor: &str) -> bool {
        !self.rules_for(actor).is_empty()
    }

    pub(crate) fn never_places(&self, actor: &str) -> Vec<String> {
        self.rules_for(actor).get("never").map(|v| v.split_whitespace().map(str::to_string).collect()).unwrap_or_default()
    }

    /// The options a rule takes off the actor's menu (H-HANDS-FORBID): `no_detachments`, `hold_line` while the
    /// odds are not against it, `solar=never`, `turrets=none`.
    pub(crate) fn removals(&self, actor: &str, odds_against: bool, solar: &str, turret: &str) -> Vec<String> {
        let rules = self.rules_for(actor);
        let mut out = Vec::new();
        if rules.get("no_detachments").is_some_and(|v| v == "yes") {
            out.extend(["send_against", "split", "scout"].map(String::from));
        }
        if rules.get("hold_line").is_some_and(|v| v == "yes") && !odds_against {
            out.extend(["retreat", "fall_back"].map(String::from));
        }
        if rules.get("solar").is_some_and(|v| v == "never") {
            out.push(solar.to_string());
        }
        if rules.get("turrets").is_some_and(|v| v == "none") {
            out.push(turret.to_string());
        }
        out
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
        let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
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
    packet.split("\n\n").filter(|p| if actor == "constructors" { p.to_lowercase().contains(needle) } else { p.contains(needle) }).collect::<Vec<_>>().join("\n\n")
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
            let name = format!("party_{digits}");
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
    for b in ["commander", "constructors"] {
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
    for b in ["commander", "constructors"] {
        let paras = paragraphs_about(packet, b);
        if paras.is_empty() {
            continue;
        }
        let who = if b == "commander" { "the commander" } else { "a constructor" };
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

// The executor.

fn order(choice: &str, params: &[(&str, &str)]) -> Order {
    Order { choice: choice.to_string(), params: params.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() }
}

/// The `how_many` the hands understand, at or above `n`.
pub(super) fn how_many(n: usize) -> &'static str {
    match n {
        0 | 1 => "1",
        2 => "2",
        3 | 4 => "4",
        _ => "8",
    }
}

impl Brain {
    /// The standing order for the actor of `menu` this second, with the rule that gave it; None asks Jev.
    pub(super) fn standing_order(&self, tick: &Tick, kit: &Kit, picture: &Picture, menu: &Menu) -> Option<(Order, String)> {
        let pianist = self.pianist.as_ref()?;
        let rules = pianist.standing.rules_for(&menu.name);
        if rules.is_empty() || menu.scripted.is_some() {
            return None;
        }
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let offered = |o: &str| menu.options.contains_key(o);
        let place_at = |name: &str| picture.places.iter().find(|p| p.name == name).map(|p| p.at);
        let never = pianist.standing.never_places(&menu.name);
        match &menu.actor {
            Actor::Group(gname) => {
                let group = pianist.groups.iter().find(|g| g.name == *gname)?;
                let units = group.units(own);
                let centre = super::groups::centre_of(&units)?;
                let nearest = picture.parties.iter().map(|p| (p.at.dist2d(centre), p)).filter(|(d, _)| *d < ALARM).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
                let odds_against = nearest.is_some_and(|p| self.odds_words(&units, p, enemies) == "it outweighs us");
                let standing: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
                let (_, lost) = group.lost_since(tick.frame - 30 * super::super::FRAMES_PER_SECOND, &self.world);
                let losing = lost >= 0.25 * (lost + standing);
                let walking_back = matches!(&group.task, GroupTask::Move { fight: false, place, .. } if place == "home" || place.starts_with(super::groups::LAST_HOLD));
                // 1. Falling back to the named place when the fight is against it.
                if let Some(to) = rules.get("fall_back_to")
                    && (odds_against || losing)
                    && !walking_back
                    && offered("move_to")
                    && place_at(to).is_some()
                    && !matches!(&group.task, GroupTask::Move { place, .. } if place == to)
                {
                    return Some((order("move_to", &[("where", to)]), "fall_back_to".into()));
                }
                // 2. A merge the player ordered: into the named group, when it stands and the menu offers it.
                if let Some(other) = rules.get("join")
                    && offered(&format!("join_{other}"))
                {
                    return Some((order(&format!("join_{other}"), &[]), "join".into()));
                }
                // 3. A named party to kill.
                if let Some(target) = rules.get("engage_party")
                    && let Some(party) = picture.parties.iter().find(|p| p.name == *target)
                    && offered("engage")
                {
                    return Some((order("engage", &[("whom", &party.name)]), "engage_party".into()));
                }
                // 3. Raiders at our structures.
                let engaged: Vec<bot_protocol::UnitId> = pianist.groups.iter().filter_map(|g| if let GroupTask::Engage { party, .. } = &g.task { Some(party.clone()) } else { None }).flatten().collect();
                let at_structure = |p: &Party| own.iter().any(|u| !u.being_built && u.pos.dist2d(p.at) < AT_STRUCTURE && (kit.is_extractor(u.def) || u.def == kit.turret || self.world.is_factory_def(u.def) || self.world.is_constructor_def(u.def)));
                let mut raiders: Vec<&Party> = picture.parties.iter().filter(|p| p.at.dist2d(centre) < RAIDER_REACH && (1..=6).contains(&p.ids.len()) && at_structure(p)).collect();
                raiders.sort_by(|a, b| a.at.dist2d(centre).total_cmp(&b.at.dist2d(centre)));
                for party in raiders {
                    let mine = matches!(&group.task, GroupTask::Engage { party: ids, .. } if ids.iter().any(|id| party.ids.contains(id)));
                    if !mine && party.ids.iter().any(|id| engaged.contains(id)) {
                        continue;
                    }
                    let rule = if party.ids.len() == 1 { "raiders_lone" } else { "raiders_party" };
                    let Some(value) = rules.get(rule) else { continue };
                    if mine {
                        return Some((order("engage", &[("whom", &party.name)]), rule.into()));
                    }
                    let odds = self.odds_words(&units, party, enemies);
                    let can_detach = offered("send_against") && party.ids.len() <= DETACH_PARTY_MAX;
                    let choice = match value.as_str() {
                        "ignore" => continue,
                        "whole_group" => "engage",
                        _ if can_detach => "send_against",
                        _ => "engage",
                    };
                    if choice == "engage" {
                        if !offered("engage") || odds == "it outweighs us" || odds.starts_with("we cannot hit") {
                            continue;
                        }
                        return Some((order("engage", &[("whom", &party.name)]), rule.into()));
                    }
                    let n = match value.strip_prefix("detachment:") {
                        Some(n) => n.to_string(),
                        None => how_many(self.detachment_for(&units, party, enemies)).to_string(),
                    };
                    return Some((order("send_against", &[("whom", &party.name), ("how_many", &n)]), rule.into()));
                }
                // 4. No chasing: an engagement carried beyond reach of the station.
                if rules.get("no_chase").is_some_and(|v| v == "yes")
                    && let GroupTask::Engage { party, .. } = &group.task
                    && offered("hold")
                {
                    let anchor = rules.get("station").and_then(|s| place_at(s)).or(group.last_hold).unwrap_or(self.home);
                    let quarry = picture.parties.iter().find(|p| p.ids.iter().any(|id| party.contains(id)));
                    if quarry.is_none_or(|p| p.at.dist2d(anchor) > RAIDER_REACH) {
                        return Some((order("hold", &[]), "no_chase".into()));
                    }
                }
                // 5. The station: walk there when holding away from it; at it, hold without asking while nothing is
                // within the alarm reach and no raider stands at a structure of ours (standing-1, 11:10: "group_A sat
                // at spot_38 while parties raided north": the hold fired and Jev was never asked about the raids
                // beyond the raider rules' reach).
                if let Some(station) = rules.get("station")
                    && let Some(at) = place_at(station)
                    && !never.iter().any(|p| p == station)
                {
                    let advance = rules.get("station_mode").is_some_and(|m| m == "advance");
                    let go = if advance { "fight_to" } else { "move_to" };
                    let raids = picture.state["actors"][&menu.name]["enemies_at_our_extractors"].as_array().is_some_and(|a| !a.is_empty());
                    match &group.task {
                        GroupTask::Move { place, .. } if place == station => return Some((order(go, &[("where", station)]), "station".into())),
                        GroupTask::Hold { committed: false, .. } if centre.dist2d(at) > STATION_SLACK && offered(go) => return Some((order(go, &[("where", station)]), "station".into())),
                        GroupTask::Hold { .. } if nearest.is_none() && !raids && offered("hold") => return Some((order("hold", &[]), "station".into())),
                        _ => {}
                    }
                }
                None
            }
            Actor::Builder(id) => {
                let unit = own.iter().find(|u| u.id == *id)?;
                let task = pianist.tasks.get(id);
                let nearest = picture.parties.iter().map(|p| (p.at.dist2d(unit.pos), p)).filter(|(d, _)| *d < ALARM).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
                // 1. Away from enemy soldiers it does not outweigh; a lone scout (a Tick, a scout car) is not one
                // (standing-1, 9:59: "a lone Tick was sending three constructors home, which stalls expansion").
                let lone_scout = |p: &Party| p.ids.len() == 1 && enemies.iter().any(|e| e.id == p.ids[0] && e.def.is_some_and(|d| super::glossary::entry(self.name(d)).is_some_and(|g| g.class.contains("scout"))));
                if rules.get("retreat_when_enemy_near").is_some_and(|v| v == "yes")
                    && let Some(party) = nearest
                    && !lone_scout(party)
                    && !self.odds_words(&[unit], party, enemies).starts_with("we outweigh")
                    && !matches!(task, Some(Task::Walk { place, .. }) if place == "home")
                {
                    if offered("retreat_home") {
                        return Some((order("retreat_home", &[]), "retreat_when_enemy_near".into()));
                    }
                    if offered("walk_to") {
                        return Some((order("walk_to", &[("where", "home")]), "retreat_when_enemy_near".into()));
                    }
                }
                // 2. The attack the menu offers (a party it outweighs within reach).
                if rules.get("attack_raiders").is_some_and(|v| v == "yes") && offered("attack") {
                    return Some((order("attack", &[]), "attack_raiders".into()));
                }
                // The rest only when the builder is free or on a filler, never over a build, and not within the
                // retreat rule's hold after it sent this builder home.
                if matches!(task, Some(Task::Build { .. }) | Some(Task::Reclaim { .. }) | Some(Task::ReclaimUnit { .. }) | Some(Task::Repair { .. })) && !menu.queue_ahead {
                    return None;
                }
                if pianist.standing.retreated.get(&menu.name).is_some_and(|f| tick.frame - f < RETREAT_HOLD) {
                    return None;
                }
                // 3. A solar when energy stalls.
                if rules.get("solar").is_some_and(|v| v == "only_when_stalling") {
                    let solar = self.name(kit.solar).to_string();
                    let stalling = picture.state["economy"]["energy"].as_str().is_some_and(|e| e.contains("STALLING"));
                    if stalling && offered(&solar) && !own.iter().any(|u| u.being_built && u.def == kit.solar) {
                        return Some((order(&solar, &[]), "solar".into()));
                    }
                }
                // 4. A turret beside an uncovered extractor.
                if let Some(which) = rules.get("turrets").filter(|v| v.as_str() != "none") {
                    let turret = self.name(kit.turret).to_string();
                    if offered(&turret) {
                        let outer_only = which == "beside_each_outer_extractor";
                        let ordered_near = |pos: Vec3| pianist.tasks.values().chain(pianist.queued.values()).any(|t| matches!(t, Task::Build { def, near, .. } if *def == kit.turret && near.dist2d(pos) < TURRET_COVER));
                        let uncovered = own
                            .iter()
                            .filter(|u| kit.is_extractor(u.def) && !u.being_built && u.pos.dist2d(unit.pos) < TURRET_REACH)
                            .filter(|u| !outer_only || u.pos.dist2d(self.home) > OUTER)
                            // Not beside an extractor a party stands at: the builder would be sent home again. A lone
                            // scout is not a party here either way: the turret is what kills the Tick (hold-2-comet-medium:
                            // one Tick at spot_54 ate fifteen extractors in two minutes, rebuilt under it, no turret ever).
                            .filter(|u| !picture.parties.iter().any(|p| p.at.dist2d(u.pos) < ALARM && !lone_scout(p)))
                            .filter(|u| !own.iter().any(|t| t.def == kit.turret && t.pos.dist2d(u.pos) < TURRET_COVER) && !ordered_near(u.pos))
                            .min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                        if let Some(extractor) = uncovered
                            && let Some(place) = picture.places.iter().filter(|p| p.at.dist2d(extractor.pos) < AT_STRUCTURE).min_by(|a, b| a.at.dist2d(extractor.pos).total_cmp(&b.at.dist2d(extractor.pos)))
                            && !never.contains(&place.name)
                        {
                            return Some((order(&turret, &[("where", &place.name)]), "turrets".into()));
                        }
                    }
                }
                // 5. The standing job.
                match rules.get("job").map(String::as_str) {
                    Some("help_factory") if offered("assist_lab") => return Some((order("assist_lab", &[]), "job".into())),
                    Some("expand") if offered("extractor") => {
                        let spot = menu.spots.iter().map(|i| format!("spot_{i}")).find(|s| !never.contains(s));
                        if let Some(spot) = spot {
                            return Some((order("extractor", &[("where_extractor", &spot)]), "job".into()));
                        }
                    }
                    _ => {}
                }
                None
            }
            _ => None,
        }
    }

    /// The smallest detachment of the group's soldiers nearest the party that outweighs it (ratio 1.3), by the
    /// combat table; the whole group when none does.
    pub(super) fn detachment_for(&self, units: &[&OwnUnit], party: &Party, enemies: &[bot_protocol::EnemyUnit]) -> usize {
        let mut theirs = super::super::combat::Force::default();
        for enemy in enemies.iter().filter(|e| party.ids.contains(&e.id)) {
            match enemy.def {
                Some(def) => theirs.add(def),
                None => theirs.unidentified += 1,
            }
        }
        let mut sorted: Vec<&OwnUnit> = units.to_vec();
        sorted.sort_by(|a, b| a.pos.dist2d(party.at).total_cmp(&b.pos.dist2d(party.at)));
        for n in 1..=sorted.len() {
            if self.odds(&Brain::force_of(&sorted[..n]), &theirs) >= 1.3 {
                return n;
            }
        }
        sorted.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(s.removals("group_B", false, "armsolar", "armllt"), Vec::<String>::new());
        s.set_tool("group_B", &json!({ "no_detachments": true, "hold_line": "yes" }), &places, &[]).unwrap();
        assert_eq!(s.removals("group_B", false, "armsolar", "armllt"), ["send_against", "split", "scout", "retreat", "fall_back"]);
        s.set_tool("group_B", &json!({ "hold_line": "no", "no_detachments": false }), &places, &[]).unwrap();
        assert!(s.removals("group_B", false, "armsolar", "armllt").is_empty());
        s.set_tool("group_B", &json!({ "no_detachments": true, "hold_line": "yes" }), &places, &[]).unwrap();
        assert_eq!(s.removals("group_B", true, "armsolar", "armllt"), ["send_against", "split", "scout"]);
    }
}
