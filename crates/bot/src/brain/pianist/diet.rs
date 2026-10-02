//! The Jev token diet (H-HANDS-DIET, docs/design/2026-09-23-jev-token-diet.md). Two 30-minute hard_aggressive games
//! spent a dollar each: 21 % of every call was the places block and 12 % entries of actors nobody asked about. One
//! setting, `WITHIN_REASON_HANDS_EFFORT`, sets how far the trimming goes: `lean` for the bulk arena games where the
//! cost adds up, `full` for a lobby game worth every token (the user, 2026-09-23). The per-question trims of the
//! menus went with the menus (`docs/design/2026-09-26-one-pass.md`).

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::Vec3;
use serde_json::Value;

use super::super::Brain;
use super::picture::{Picture, Place};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandsEffort {
    Lean,
    Normal,
    Full,
}

/// What a level trims.
#[derive(Clone, Debug)]
pub(crate) struct Diet {
    pub level: HandsEffort,
    /// The places block carries only the places in play: those named in the text, the spots within this of an
    /// asked actor (None: every place) ...
    pub places_reach: Option<f32>,
    /// ... plus every spot we hold.
    pub places_held_too: bool,
    /// Actors not asked in the call get one line, not their full entry.
    pub actors_brief: bool,
}

impl Diet {
    pub fn level(level: HandsEffort) -> Diet {
        match level {
            HandsEffort::Lean => Diet { level, places_reach: Some(1_500.0), places_held_too: false, actors_brief: true },
            HandsEffort::Normal => Diet { level, places_reach: Some(2_500.0), places_held_too: true, actors_brief: true },
            HandsEffort::Full => Diet { level, places_reach: None, places_held_too: true, actors_brief: false },
        }
    }

    /// `WITHIN_REASON_HANDS_EFFORT`: lean, normal (the default) or full.
    pub fn from_env() -> Diet {
        Diet::level(parse(std::env::var("WITHIN_REASON_HANDS_EFFORT").ok().as_deref()))
    }

    pub fn level_name(&self) -> &'static str {
        match self.level {
            HandsEffort::Lean => "lean",
            HandsEffort::Normal => "normal",
            HandsEffort::Full => "full",
        }
    }
}

fn parse(value: Option<&str>) -> HandsEffort {
    match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("lean") | Some("low") => HandsEffort::Lean,
        Some("full") | Some("high") => HandsEffort::Full,
        _ => HandsEffort::Normal,
    }
}

fn first_named(text: &str, place: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = text[from..].find(place) {
        let end = from + i + place.len();
        if !text[end..].chars().next().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            return Some(from + i);
        }
        from = end;
    }
    None
}

/// Whether `text` names `place` as a whole word: "spot_4" is not named by "spot_45".
pub(super) fn names(text: &str, place: &str) -> bool {
    first_named(text, place).is_some()
}

/// Whether the text names spot `i`, written in full ("spot_34") or in a run after one ("spot_34, 29, 35": the
/// brief's own shorthand, which game 10's packet used and nothing read: the seats' spots matched nothing and the
/// first constructors went to the scouting sentence's far spots).
pub(super) fn names_spot(text: &str, i: usize) -> bool {
    if names(text, &format!("spot_{i}")) {
        return true;
    }
    spot_runs(text).contains(&i)
}

/// The spots named in runs: after a "spot_N", every ", M" that follows with only digits, commas and spaces between.
pub(super) fn spot_runs(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (start, _) in text.match_indices("spot_") {
        let rest = &text[start + 5..];
        let first: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if first.is_empty() {
            continue;
        }
        let mut tail = &rest[first.len()..];
        loop {
            let trimmed = tail.trim_start_matches([',', ' ']);
            if trimmed.len() == tail.len() || !trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                break;
            }
            let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
            let after = &trimmed[digits.len()..];
            // "spot_10, 5, 6" names three and "spot_34, 29 and spot_35" names 29; "spot_10, 5 constructors" ends
            // at the word, the 5 being a count.
            let next_word: String = after.trim_start().chars().take_while(|c| c.is_ascii_alphabetic()).collect();
            if !next_word.is_empty() && !matches!(next_word.as_str(), "and" | "then" | "or") {
                break;
            }
            if let Ok(n) = digits.parse::<usize>() {
                out.push(n);
            }
            tail = after;
        }
    }
    out
}

impl Brain {
    /// The request's state: the picture's, with the places block cut to the places in play and the entries of
    /// actors not asked cut to a line, as the diet says. `asked` is each asked actor and where it stands.
    pub(super) fn trim_state(&self, diet: &Diet, picture: &Picture, asked: &[(String, Option<Vec3>)]) -> Value {
        let mut state = picture.state.clone();
        if diet.places_reach.is_none() && !diet.actors_brief {
            return state;
        }
        let names_asked: BTreeSet<&str> = asked.iter().map(|(n, _)| n.as_str()).collect();
        let positions: Vec<Vec3> = asked.iter().filter_map(|(_, p)| *p).collect();
        let instructions = state["instructions"].as_str().unwrap_or_default().to_string();
        let mut text = instructions.clone();
        text.push_str(&state["enemy"].to_string());
        if let Some(actors) = state["actors"].as_object() {
            for name in &names_asked {
                if let Some(e) = actors.get(*name) {
                    text.push_str(&e.to_string());
                }
            }
        }
        if let Some(reach) = diet.places_reach
            && let Some(places) = state["places"].as_object_mut()
        {
            let by_name: BTreeMap<&str, &Place> = picture.places.iter().map(|p| (p.name.as_str(), p)).collect();
            places.retain(|name, entry| {
                let Some(p) = by_name.get(name.as_str()) else { return true };
                p.spot.is_none()
                    || names(&text, name)
                    || positions.iter().any(|c| c.dist2d(p.at) <= reach)
                    || (diet.places_held_too && entry["what"].as_str().is_some_and(|w| w.starts_with("our ")))
            });
        }
        if diet.actors_brief
            && let Some(actors) = state["actors"].as_object_mut()
        {
            for (name, entry) in actors.iter_mut() {
                if names_asked.contains(name.as_str()) || names(&instructions, name) {
                    continue;
                }
                brief(entry);
            }
        }
        state
    }
}

/// The characters of state a call keeps under. Player-10-routes: the largest call answered was 65,587 tokens in
/// (19:26, 22 actors in the picture), and from 19:33 to 26:31 thirty calls came back `max_tokens_exceeded` while the
/// F4 push fell apart on the hands' last course; about 3.7 characters a token there, so this is some 43,000 tokens,
/// leaving the questions and the answer room under the model's limit.
pub(super) const STATE_CHARS: usize = 160_000;

/// Cuts `state` toward `budget` characters, least useful first: `recent` to its last six lines, then the places the
/// instructions do not name, then the largest actors' entries to a line one by one. Returns what was cut, for the
/// log; empty when nothing was.
pub(super) fn shed(state: &mut Value, budget: usize) -> Vec<String> {
    let size = |s: &Value| s.to_string().len();
    let mut shed = Vec::new();
    if size(state) <= budget {
        return shed;
    }
    if let Some(recent) = state["recent"].as_array_mut()
        && recent.len() > 6
    {
        let n = recent.len();
        recent.drain(..n - 6);
        shed.push(format!("recent cut to the last 6 of {n}"));
    }
    if size(state) <= budget {
        return shed;
    }
    let instructions = state["instructions"].as_str().unwrap_or_default().to_string();
    if let Some(places) = state["places"].as_object_mut() {
        let before = places.len();
        places.retain(|name, _| names(&instructions, name));
        if places.len() < before {
            shed.push(format!("places cut to the {} the instructions name, of {before}", places.len()));
        }
    }
    if size(state) <= budget {
        return shed;
    }
    let rest = size(state) - size(&state["actors"]);
    if let Some(actors) = state["actors"].as_object_mut() {
        let mut by_size: Vec<(usize, String)> = actors.iter().filter(|(_, e)| e.is_object()).map(|(n, e)| (e.to_string().len(), n.clone())).collect();
        by_size.sort_by(|a, b| b.cmp(a));
        let mut cut = Vec::new();
        for (_, name) in by_size {
            if let Some(entry) = actors.get_mut(&name) {
                // Units and place only: `brief` keeps `doing`, which is what a long entry is made of.
                let field = |k: &str| entry[k].as_str().unwrap_or_default().to_string();
                *entry = Value::String(format!("{}; at {}; its entry cut for room", field("units"), field("at")));
                cut.push(name);
            }
            if rest + state_size(actors) <= budget {
                break;
            }
        }
        if !cut.is_empty() {
            shed.push(format!("entries cut to a line: {}", cut.join(" ")));
        }
    }
    let now = size(state);
    if now > budget {
        shed.push(format!("still {now} characters, over {budget}"));
    }
    shed
}

fn state_size(actors: &serde_json::Map<String, Value>) -> usize {
    Value::Object(actors.clone()).to_string().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_over_budget_is_cut_least_useful_first() {
        let mut state = serde_json::json!({
            "instructions": "group_A: spot_1 then spot_2.",
            "recent": (0..40).map(|i| format!("{i}:00 something happened at spot_{i} with many words in it to take room")).collect::<Vec<_>>(),
            "places": (0..60).map(|i| (format!("spot_{i}"), serde_json::json!({"what": "x".repeat(200)}))).collect::<serde_json::Map<_, _>>(),
            "actors": {
                "group_A": {"units": "10 Blitz", "at": "spot_1", "doing": "y".repeat(3000), "reached": "spot_1 (3:10)"},
                "group_B": {"units": "3 Stout", "at": "home", "doing": "z".repeat(5000)},
                "commander": {"units": "commander", "at": "home", "doing": "w".repeat(100)}
            }
        });
        let before = state.to_string().len();
        let cut = shed(&mut state, 6_000);
        assert!(state.to_string().len() < before);
        assert_eq!(state["recent"].as_array().unwrap().len(), 6);
        assert_eq!(state["places"].as_object().unwrap().len(), 2, "spot_1 and spot_2 are named");
        assert!(cut.iter().any(|s| s.starts_with("recent cut")) && cut.iter().any(|s| s.starts_with("places cut")), "{cut:?}");
        assert!(cut.iter().any(|s| s.starts_with("entries cut to a line: group_B")), "the largest entry goes first: {cut:?}");
        assert!(state["actors"]["commander"].is_object(), "the smallest entry stays whole: {cut:?}");
        let mut small = serde_json::json!({"instructions": "x", "recent": ["a"], "places": {}, "actors": {}});
        assert!(shed(&mut small, 6_000).is_empty());
    }

    #[test]
    fn the_level_is_read_from_the_setting_and_defaults_to_normal() {
        assert_eq!(parse(Some("lean")), HandsEffort::Lean);
        assert_eq!(parse(Some("FULL")), HandsEffort::Full);
        assert_eq!(parse(None), HandsEffort::Normal);
        assert_eq!(parse(Some("bogus")), HandsEffort::Normal);
    }

    #[test]
    fn a_run_after_a_spot_names_every_number_in_it() {
        assert_eq!(spot_runs("North seat's spots: spot_10, 5, 6, 12. Middle: spot_34, 29 and spot_35."), vec![5, 6, 12, 29]);
        assert!(names_spot("spot_10, 5, 6", 6));
        assert!(!names_spot("spot_10, 5 constructors", 5));
        assert!(names_spot("go to spot_45", 45));
    }

    #[test]
    fn a_place_is_named_as_a_whole_word() {
        assert!(names("go to spot_4 now", "spot_4"));
        assert!(!names("go to spot_45 now", "spot_4"));
        assert!(names("spot_45 and spot_4.", "spot_4"));
    }
}

/// An actor's entry cut to one line, not a four-field object (wake-1, 11:23: fifteen unasked actors were 3.4k of a
/// 12.6k-character state as objects).
pub(super) fn brief(entry: &mut Value) {
    if let Some(full) = entry.as_object() {
        let field = |k: &str| full.get(k).and_then(Value::as_str).map(str::to_string);
        let mut parts: Vec<String> = Vec::new();
        if let Some(units) = field("units") {
            parts.push(units);
        }
        if let Some(at) = field("at") {
            parts.push(format!("at {at}"));
        }
        if let Some(doing) = field("doing") {
            parts.push(doing);
        }
        *entry = Value::String(parts.join(", "));
    }
}
