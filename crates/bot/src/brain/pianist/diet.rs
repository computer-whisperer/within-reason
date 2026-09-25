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

/// Whether `text` names `place` as a whole word: "spot_4" is not named by "spot_45".
pub(super) fn names(text: &str, place: &str) -> bool {
    let mut from = 0;
    while let Some(i) = text[from..].find(place) {
        let end = from + i + place.len();
        if !text[end..].chars().next().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            return true;
        }
        from = end;
    }
    false
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
                // One line, not a four-field object (wake-1, 11:23: fifteen unasked actors were 3.4k of a 12.6k-character
                // state as objects).
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
        }
        state
    }

    /// Where a scout goes: the enemy's base when one is known and nothing of ours has looked at it for three
    /// minutes, else the nearest never-looked spot inside the enemy's start box, else the spot longest out of sight.
    pub(super) fn scout_target(&self, from: Vec3, picture: &Picture, frame: i32) -> Option<Place> {
        const LOOKED_LATELY: i32 = 3 * 60 * super::super::FRAMES_PER_SECOND;
        let spots = &self.world.hello.metal_spots;
        // A place a scout of ours is already walking to is spoken for (diet-1: three scouts to spot_10 in 90 s).
        let bound_for: Vec<String> = self.pianist.as_ref().map(|p| p.groups.iter().filter_map(|g| g.task.place().map(str::to_string)).collect()).unwrap_or_default();
        let free = |p: &Place| !bound_for.contains(&p.name);
        if let Some(base) = self.found_enemy_base() {
            let looked = (0..spots.len()).filter(|i| spots[*i].dist2d(base) < 600.0).filter_map(|i| self.spot_seen(i)).max();
            if looked.is_none_or(|f| frame - f > LOOKED_LATELY) {
                let name = picture.places.iter().filter(|p| p.at.dist2d(base) < 300.0).min_by(|a, b| a.at.dist2d(base).total_cmp(&b.at.dist2d(base))).map_or_else(|| "enemy base".to_string(), |p| p.name.clone());
                return Some(Place { name, at: base, spot: None });
            }
        }
        let ours = self.world.hello.ally_team;
        let in_their_box = |at: Vec3| self.world.hello.start_boxes.iter().any(|b| b.ally_team != ours && b.contains(at));
        let never: Option<&Place> = picture.places.iter().filter(|p| p.spot.is_some_and(|i| self.spot_seen(i).is_none()) && in_their_box(p.at) && free(p)).min_by(|a, b| a.at.dist2d(from).total_cmp(&b.at.dist2d(from)));
        if let Some(p) = never {
            return Some(p.clone());
        }
        picture.places.iter().filter(|p| p.spot.is_some() && free(p)).min_by_key(|p| p.spot.and_then(|i| self.spot_seen(i)).unwrap_or(i32::MIN)).cloned()
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
    fn a_place_is_named_as_a_whole_word() {
        assert!(names("go to spot_4 now", "spot_4"));
        assert!(!names("go to spot_45 now", "spot_4"));
        assert!(names("spot_45 and spot_4.", "spot_4"));
    }
}
