//! The Jev token diet (H-HANDS-DIET, docs/design/2026-09-23-jev-token-diet.md). Two 30-minute hard_aggressive games
//! spent a dollar each: 23 % of every call was two lists of all 55 places under each group question, 21 % the places
//! block, 18 % builder options that each repeated the economy, 12 % entries of actors nobody asked about, and 17 %
//! of the calls asked only holding groups with nothing near. One setting, `WITHIN_REASON_HANDS_EFFORT`, sets how far
//! the trimming goes: `lean` for the bulk arena games where the cost adds up, `full` for a lobby game worth every
//! token (the user, 2026-09-23: "high-level matches against humans may warrant escalation if we are otherwise close").

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Tick, Vec3};
use serde_json::Value;

use super::super::{Brain, FRAMES_PER_SECOND};
use super::menu::{Actor, Menu};
use super::picture::{Picture, Place};
use super::Pianist;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandsEffort {
    Lean,
    Normal,
    Full,
}

/// What a level trims. Decisions 1 (no `where_scout`) and 5 (the economy once per builder question) hold at every
/// level; these are the knobs that differ.
#[derive(Clone, Debug)]
pub(crate) struct Diet {
    pub level: HandsEffort,
    /// A group's `where` lists the spots within this of it (None: every place) ...
    pub where_reach: Option<f32>,
    /// ... at most this many spots, nearest first.
    pub where_most: usize,
    /// The places block carries only the places in play (the asked actors' reach, the names in the text).
    pub places_in_play: bool,
    /// ... plus every spot we hold.
    pub places_held_too: bool,
    /// Actors not asked in the call get one line, not their full entry.
    pub actors_brief: bool,
    /// A holding group with no enemy within twice the alarm reach and no hit is reviewed every this many frames.
    pub quiet_hold_review: i32,
    /// A factory whose question, entry and instructions changed only in numbers is answered from its last answer
    /// for this long (None: always asked).
    pub replay_frames: Option<i32>,
    /// The most characters of questions one call carries (H-HANDS-SCHEDULE's budget).
    pub question_budget: usize,
}

impl Diet {
    pub fn level(level: HandsEffort) -> Diet {
        match level {
            HandsEffort::Lean => Diet { level, where_reach: Some(1_500.0), where_most: 10, places_in_play: true, places_held_too: false, actors_brief: true, quiet_hold_review: 15 * FRAMES_PER_SECOND, replay_frames: Some(60 * FRAMES_PER_SECOND), question_budget: 60_000 },
            HandsEffort::Normal => Diet { level, where_reach: Some(2_500.0), where_most: 14, places_in_play: true, places_held_too: true, actors_brief: true, quiet_hold_review: 10 * FRAMES_PER_SECOND, replay_frames: Some(30 * FRAMES_PER_SECOND), question_budget: 80_000 },
            HandsEffort::Full => Diet { level, where_reach: None, where_most: usize::MAX, places_in_play: false, places_held_too: true, actors_brief: false, quiet_hold_review: 5 * FRAMES_PER_SECOND, replay_frames: None, question_budget: 80_000 },
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

/// The same text with every run of digits replaced by one mark: two asks that differ only in their numbers (a clock,
/// a store's level) compare equal.
pub(super) fn without_numbers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_digits = false;
    for c in text.chars() {
        if c.is_ascii_digit() {
            if !in_digits {
                out.push('#');
            }
            in_digits = true;
        } else {
            in_digits = false;
            out.push(c);
        }
    }
    out
}

/// One draw from a distribution, deterministic in `seed`: a replayed factory answer keeps the mix its probabilities
/// describe instead of repeating one unit.
pub(super) fn draw(probabilities: &BTreeMap<String, f64>, seed: u64) -> Option<String> {
    let total: f64 = probabilities.values().filter(|p| **p > 0.0).sum();
    if total <= 0.0 {
        return probabilities.iter().max_by(|a, b| a.1.total_cmp(b.1)).map(|(k, _)| k.clone());
    }
    // A 64-bit mix of the seed to a unit interval.
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x632B_E59B_D9B4_E019);
    x ^= x >> 31;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 29;
    let mut u = (x >> 11) as f64 / (1u64 << 53) as f64 * total;
    for (k, p) in probabilities.iter().filter(|(_, p)| **p > 0.0) {
        if u < *p {
            return Some(k.clone());
        }
        u -= p;
    }
    probabilities.iter().max_by(|a, b| a.1.total_cmp(b.1)).map(|(k, _)| k.clone())
}

impl Brain {
    /// The `where` choices for a group standing at `from`: every place that is not a spot, every place the
    /// instructions name, the group's own destination, and the nearest spots within the diet's reach.
    pub(super) fn places_in_reach<'a>(&self, diet: &Diet, picture: &'a Picture, from: Option<Vec3>, destination: Option<&str>) -> Vec<&'a Place> {
        let instructions = picture.state["instructions"].as_str().unwrap_or_default();
        let Some(reach) = diet.where_reach else { return picture.places.iter().collect() };
        let mut spots: Vec<(f32, &Place)> = Vec::new();
        let mut keep: Vec<&Place> = Vec::new();
        for p in &picture.places {
            if p.spot.is_none() || names(instructions, &p.name) || destination == Some(p.name.as_str()) {
                keep.push(p);
            } else if let Some(c) = from
                && c.dist2d(p.at) <= reach
            {
                spots.push((c.dist2d(p.at), p));
            }
        }
        spots.sort_by(|a, b| a.0.total_cmp(&b.0));
        keep.extend(spots.into_iter().take(diet.where_most).map(|(_, p)| p));
        keep
    }

    /// The request's state: the picture's, with the places block cut to the places in play and the entries of
    /// actors not asked cut to a line, as the diet says.
    pub(super) fn trim_state(&self, pianist: &Pianist, picture: &Picture, menus: &[Menu], tick: &Tick) -> Value {
        let diet = &pianist.diet;
        let mut state = picture.state.clone();
        if !diet.places_in_play && !diet.actors_brief {
            return state;
        }
        let own = &tick.snapshot.own_units;
        let mut asked: BTreeSet<String> = BTreeSet::new();
        let mut positions: Vec<Vec3> = Vec::new();
        for m in menus.iter().filter(|m| !m.questions.is_empty() && m.replay.is_none()) {
            asked.insert(m.name.clone());
            match &m.actor {
                Actor::Builder(id) | Actor::Lab(id) => positions.extend(own.iter().find(|u| u.id == *id).map(|u| u.pos)),
                Actor::Group(g) => positions.extend(pianist.groups.iter().find(|x| x.name == *g).and_then(|x| super::groups::centre_of(&x.units(own)))),
                Actor::Global => {}
            }
        }
        let instructions = state["instructions"].as_str().unwrap_or_default().to_string();
        let mut text = instructions.clone();
        text.push_str(&state["enemy"].to_string());
        if let Some(actors) = state["actors"].as_object() {
            for name in &asked {
                if let Some(e) = actors.get(name) {
                    text.push_str(&e.to_string());
                }
            }
        }
        if diet.places_in_play
            && let Some(places) = state["places"].as_object_mut()
        {
            let reach = diet.where_reach.unwrap_or(f32::MAX);
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
                if asked.contains(name) || names(&instructions, name) {
                    continue;
                }
                if let Some(full) = entry.as_object() {
                    let brief: serde_json::Map<String, Value> = full.iter().filter(|(k, _)| matches!(k.as_str(), "at" | "doing" | "units" | "health")).map(|(k, v)| (k.clone(), v.clone())).collect();
                    *entry = Value::Object(brief);
                }
            }
        }
        state
    }

    /// Where a scout goes (decision 1: the `where_scout` question is gone): the enemy's base when one is known and
    /// nothing of ours has looked at it for three minutes, else the nearest never-looked spot inside the enemy's
    /// start box, else the spot longest out of sight.
    pub(super) fn scout_target(&self, from: Vec3, picture: &Picture, frame: i32) -> Option<Place> {
        const LOOKED_LATELY: i32 = 3 * 60 * FRAMES_PER_SECOND;
        let spots = &self.world.hello.metal_spots;
        if let Some(base) = self.found_enemy_base() {
            let looked = (0..spots.len()).filter(|i| spots[*i].dist2d(base) < 600.0).filter_map(|i| self.spot_seen(i)).max();
            if looked.is_none_or(|f| frame - f > LOOKED_LATELY) {
                let name = picture.places.iter().filter(|p| p.at.dist2d(base) < 300.0).min_by(|a, b| a.at.dist2d(base).total_cmp(&b.at.dist2d(base))).map_or_else(|| "enemy base".to_string(), |p| p.name.clone());
                return Some(Place { name, at: base, spot: None });
            }
        }
        let ours = self.world.hello.ally_team;
        let in_their_box = |at: Vec3| self.world.hello.start_boxes.iter().any(|b| b.ally_team != ours && b.contains(at));
        let never: Option<&Place> = picture.places.iter().filter(|p| p.spot.is_some_and(|i| self.spot_seen(i).is_none()) && in_their_box(p.at)).min_by(|a, b| a.at.dist2d(from).total_cmp(&b.at.dist2d(from)));
        if let Some(p) = never {
            return Some(p.clone());
        }
        picture.places.iter().filter(|p| p.spot.is_some()).min_by_key(|p| p.spot.and_then(|i| self.spot_seen(i)).unwrap_or(i32::MIN)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_level_is_read_from_the_setting_and_defaults_to_normal() {
        assert_eq!(parse(Some("lean")), HandsEffort::Lean);
        assert_eq!(parse(Some(" Full ")), HandsEffort::Full);
        assert_eq!(parse(Some("anything")), HandsEffort::Normal);
        assert_eq!(parse(None), HandsEffort::Normal);
        assert!(Diet::level(HandsEffort::Full).where_reach.is_none() && Diet::level(HandsEffort::Lean).where_reach.is_some());
    }

    #[test]
    fn a_place_is_named_as_a_whole_word() {
        assert!(names("hold spot_4 and spot_12", "spot_4"));
        assert!(!names("hold spot_45", "spot_4"));
        assert!(names("go to home.", "home"));
    }

    #[test]
    fn numbers_are_one_mark_and_a_draw_follows_the_distribution() {
        assert_eq!(without_numbers("holding for 12 s, 3 soldiers"), "holding for # s, # soldiers");
        let p: BTreeMap<String, f64> = [("a".to_string(), 0.7), ("b".to_string(), 0.3)].into_iter().collect();
        let a = (0..1000u64).filter(|s| draw(&p, *s).as_deref() == Some("a")).count();
        assert!((600..800).contains(&a), "{a}");
    }
}
