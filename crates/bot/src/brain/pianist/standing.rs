//! Standing orders (`docs/design/2026-09-25-standing-orders.md`): per-actor rules with a parameter, held in the hands
//! and read in code every second against the picture. They come from one place, the player's `standing` tool,
//! which sets them directly and keeps them until cleared. The packet's decompression into these rules by Jev
//! (2026-09-25 to 2026-09-27) is gone: over eleven games with people it bridged a few rules the tool carried anyway
//! and hurt through switch-offs that could not work, pruning lifted from other actors' sentences or with conditions
//! dropped, one-seat packets read into every seat, and a re-read every quiet second
//! (`docs/design/2026-09-27-posing-changes.md` §13b). The packet stands as prose in the picture for the pass.
//! In the pass (`plan.rs`, `threats.rs`) a rule prunes states or makes one the default; nothing here orders
//! (`docs/design/2026-09-26-one-pass.md` §6).
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The raider rules and the threat states reach this far from the group (the detectors' reach).
pub(super) const RAIDER_REACH: f32 = 1200.0;
/// A party this close to a place a group never goes stands at it: no state is offered against it, and a chase that
/// reaches it ends there.
pub(super) const NEVER_REACH: f32 = 400.0;

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
/// A builder's `no_chase` and `job follow_list` were in the vocabulary and read by nothing (the decompression audit,
/// 2026-09-27): gone.
pub(crate) const BUILDER_RULES: &[(&str, &[&str])] = &[
    ("job", &["help_factory", "expand"]),
    ("attack_raiders", &["yes"]),
    ("retreat_when_enemy_near", &["yes"]),
    ("solar", &["only_when_stalling", "never", "freely"]),
    ("turrets", &["beside_each_outer_extractor", "beside_each_extractor", "none"]),
    ("never", &["places"]),
];

pub(crate) type Rules = BTreeMap<String, String>;

#[derive(Default)]
pub(crate) struct Standing {
    /// From the `standing` tool, kept until cleared: an actor's own rules, or a class's (`constructors`, `commander`).
    tool: BTreeMap<String, Rules>,
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
    /// The rules in force for an actor: its class's (`constructors` for a constructor, `commander` for every
    /// seat's commander), then its own over them.
    pub(crate) fn rules_for(&self, actor: &str) -> Rules {
        let mut out = Rules::new();
        let class = if actor.starts_with("constructor_") {
            Some("constructors")
        } else if actor.starts_with("commander_t") {
            Some("commander")
        } else {
            None
        };
        if let Some(c) = class
            && let Some(rules) = self.tool.get(c)
        {
            out.extend(rules.clone());
        }
        // An actor's own empty value is its "no" over the class's rule (the decompression audit, 2026-09-27: a
        // "no" that only cleared the actor's own entry left the rule underneath in force, and the player could
        // not switch it off).
        if let Some(rules) = self.tool.get(actor) {
            for (rule, value) in rules {
                if value.is_empty() {
                    out.remove(rule);
                } else {
                    out.insert(rule.clone(), value.clone());
                }
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
        Some(rules.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join("; "))
    }

    /// Sets tool orders after checking each against the vocabulary and the picture's places and parties: the
    /// number set, and the rules refused with why (a refusal never takes the actor's other rules with it: 7.2).
    pub(crate) fn set_tool(&mut self, actor: &str, rules: &Value, places: &[String], parties: &[String]) -> Result<(usize, Vec<String>), String> {
        let (checked, refused) = Self::check_tool(actor, rules, places, parties)?;
        self.tool_seen.remove(actor);
        let under_a_class = actor.starts_with("constructor_") || actor.starts_with("commander_t");
        let entry = self.tool.entry(actor.to_string()).or_default();
        let n = checked.len();
        for (rule, value) in checked {
            if value.is_empty() && !under_a_class {
                entry.remove(&rule);
            } else {
                // Under a class the empty value stays as the actor's "no" over the class's rule (`rules_for`).
                entry.insert(rule, value);
            }
        }
        if entry.is_empty() {
            self.tool.remove(actor);
        }
        Ok((n, refused))
    }

    /// One actor's tool rules checked against the vocabulary and the picture's places and parties: the rules to set
    /// (an empty value clears one) and the rules refused with why. Only a set that is not an object is refused
    /// whole. The `standing` tool runs this in the player's turn so a refusal is answered at once (worlds-2, 17:07:
    /// "a set with an unknown place is refused whole, silently in my view"); the hands run it again at their next
    /// second. Until 2026-09-27 evening one unknown place refused the actor's whole set (game 6 16:42).
    pub(crate) fn check_tool(actor: &str, rules: &Value, places: &[String], parties: &[String]) -> Result<(Rules, Vec<String>), String> {
        let Some(map) = rules.as_object() else { return Err(format!("{actor}: rules must be an object of rule to value")) };
        let vocabulary = if is_group(actor) { GROUP_RULES } else { BUILDER_RULES };
        let mut checked = Rules::new();
        let mut refused: Vec<String> = Vec::new();
        for (rule, value) in map {
            let Some((_, allowed)) = vocabulary.iter().find(|(r, _)| r == rule) else {
                refused.push(format!("{actor}: {rule} is not a rule ({})", vocabulary.iter().map(|(r, _)| *r).collect::<Vec<_>>().join(", ")));
                continue;
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
                other => {
                    refused.push(format!("{actor}: {rule} takes a string, not {other}"));
                    continue;
                }
            };
            let problem = if !valid_value(rule, &value, allowed) {
                Some(format!("{actor}: {rule} takes {}, not {value}", allowed.join(" | ")))
            } else if allowed.contains(&"place") && !places.iter().any(|p| *p == value) {
                Some(format!("{actor}: {value} is not a place in the picture"))
            } else if rule == "never" && let Some(bad) = value.split_whitespace().find(|p| !places.iter().any(|q| q == p)) {
                Some(format!("{actor}: {bad} is not a place in the picture"))
            } else if allowed.contains(&"party") && !parties.iter().any(|p| *p == value) {
                Some(format!("{actor}: {value} is not a party in the picture"))
            } else if allowed.contains(&"group") && (!value.starts_with("group_") || value == actor) {
                Some(format!("{actor}: {rule} takes another group's name (group_X), not {value}"))
            } else {
                None
            };
            match problem {
                Some(why) => refused.push(why),
                None => {
                    checked.insert(rule.clone(), value);
                }
            }
        }
        Ok((checked, refused))
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
        if self.tool.is_empty() {
            return "no standing orders are in force".to_string();
        }
        self.tool.keys().filter_map(|a| self.words(a).map(|w| format!("{a}: {w}"))).collect::<Vec<_>>().join("\n")
    }

    /// How many rules are in force (an actor's "no" over its class is not one).
    pub(crate) fn count(&self) -> usize {
        self.tool.values().flat_map(Rules::values).filter(|v| !v.is_empty()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_tool_check_names_the_unknown_place_and_passes_the_rest() {
        let places = ["spot_61", "spot_9"].map(String::from);
        let (checked, refused) = Standing::check_tool("group_C", &json!({ "never": "shelling spot_9", "station": "spot_61" }), &places, &[]).unwrap();
        assert_eq!(refused, vec!["group_C: shelling is not a place in the picture".to_string()]);
        assert_eq!(checked, Rules::from([("station".to_string(), "spot_61".to_string())]), "the rest of the set stands");
        let (checked, refused) = Standing::check_tool("group_C", &json!({ "never": "spot_9", "station": "spot_61", "raiders_lone": null }), &places, &[]).unwrap();
        assert!(refused.is_empty());
        assert_eq!(checked, Rules::from([("never".to_string(), "spot_9".to_string()), ("station".to_string(), "spot_61".to_string()), ("raiders_lone".to_string(), String::new())]));
    }

    #[test]
    fn tool_orders_are_checked_and_a_class_rule_reaches_its_actors() {
        let mut s = Standing::default();
        let places = ["spot_61", "spot_9"].map(String::from);
        assert_eq!(s.set_tool("group_B", &json!({ "station": "spot_7" }), &places, &[]).unwrap().0, 0);
        assert_eq!(s.set_tool("group_B", &json!({ "raiders_lone": "detachment:3" }), &places, &[]).unwrap().1.len(), 1);
        assert_eq!(s.set_tool("group_B", &json!({ "bogus": "yes" }), &places, &[]).unwrap().0, 0);
        assert_eq!(s.set_tool("constructor_4", &json!({ "job": "follow_list" }), &places, &[]).unwrap().0, 0, "follow_list left the vocabulary");
        assert!(s.set_tool("group_B", &json!(["not", "an", "object"]), &places, &[]).is_err());
        assert_eq!(s.set_tool("group_B", &json!({ "station": "spot_9", "raiders_lone": "detachment:2" }), &places, &[]).unwrap(), (2, Vec::new()));
        assert_eq!(s.set_tool("group_B", &json!({ "station": "spot_9 spot_61", "never": "bogus_place" }), &places, &[]).unwrap().0, 0, "a station list is refused: a route is prose (routes-in-prose 4.1)");
        s.set_tool("group_B", &json!({ "station": "spot_9" }), &places, &[]).unwrap();
        let r = s.rules_for("group_B");
        assert_eq!(r["station"], "spot_9");
        assert_eq!(s.words("group_B").unwrap(), "raiders_lone detachment:2; station spot_9");
        s.set_tool("constructors", &json!({ "job": "expand" }), &places, &[]).unwrap();
        assert_eq!(s.rules_for("constructor_4")["job"], "expand");
        s.set_tool("commander", &json!({ "attack_raiders": "yes" }), &places, &[]).unwrap();
        s.set_tool("commander_t2", &json!({ "attack_raiders": "no" }), &places, &[]).unwrap();
        assert_eq!(s.rules_for("commander_t1")["attack_raiders"], "yes", "the class rule is every seat's commander's");
        assert!(!s.rules_for("commander_t2").contains_key("attack_raiders"), "a seat's `no` is its own over the class's rule");
        assert!(s.words("commander_t2").is_none());
        assert_eq!(s.count(), 4);
        assert_eq!(s.clear_tool(Some(&["group_B".to_string()])), 1);
        assert!(s.rules_for("group_B").is_empty());
        s.set_tool("group_B", &json!({ "no_detachments": true, "hold_line": "yes" }), &places, &[]).unwrap();
        assert_eq!(s.rules_for("group_B")["hold_line"], "yes");
        s.set_tool("group_B", &json!({ "hold_line": "no", "no_detachments": false }), &places, &[]).unwrap();
        assert!(!s.rules_for("group_B").contains_key("hold_line"));
        assert!(s.in_force().contains("constructors: job expand"));
    }
}
