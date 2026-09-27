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

/// The paragraph of the instructions addressed to `actor` ("group_A (Blitzes): ..." or "group_A: ..."): the
/// packet's convention is one paragraph per actor, so a route or a job written there is that actor's alone
/// (docs/design/2026-09-28-routes-in-prose.md §4.1).
pub(super) fn paragraph<'a>(text: &'a str, actor: &str) -> Option<&'a str> {
    text.split("\n\n").map(str::trim).find(|p| {
        let head = p.split(':').next().unwrap_or("");
        head.split('(').next().unwrap_or("").trim() == actor
    })
}

/// The places named in `text`, in the order they are first named: a route's stops as the packet wrote them.
pub(super) fn named_in_order<'a>(text: &str, places: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut found: Vec<(usize, &str)> = places.filter_map(|p| first_named(text, p).map(|i| (i, p))).collect();
    found.sort();
    found.into_iter().map(|(_, p)| p).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_actors_paragraph_and_its_places_in_order() {
        let packet = "Plan: hold.\n\ngroup_A (Blitzes): spot_49, then spot_46, spot_40 and back to spot_64.\n\ngroup_B: scouts spot_72, then spot_60.\n\nconstructors: spot_69, spot_62.";
        assert_eq!(paragraph(packet, "group_A"), Some("group_A (Blitzes): spot_49, then spot_46, spot_40 and back to spot_64."));
        assert_eq!(paragraph(packet, "group_B"), Some("group_B: scouts spot_72, then spot_60."));
        assert_eq!(paragraph(packet, "group_C"), None);
        let places = ["spot_64", "spot_40", "spot_4", "spot_49", "spot_46", "spot_69"];
        assert_eq!(named_in_order(paragraph(packet, "group_A").unwrap(), places.iter().copied()), vec!["spot_49", "spot_46", "spot_40", "spot_64"], "the route's stops in the packet's order; spot_4 is not named by spot_40 or spot_49");
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
