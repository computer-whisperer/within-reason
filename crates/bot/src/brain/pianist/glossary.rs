//! The unit glossary (`crates/bot/data/units.json`, docs/design/2026-09-22-full-roster.md): the words for every
//! reachable unit of both factions. The numbers, English name, tier, class and `made_by` are generated from the game
//! source by `run/unit_stats.py`; `gloss` (a clause for a menu line) and `prose` (a paragraph for the player) are
//! written by hand. A unit the file does not know gets words made from its definition alone.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Entry {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub faction: String,
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub made_by: Vec<String>,
    #[serde(default)]
    pub metal: f32,
    #[serde(default)]
    pub energy: f32,
    #[serde(default)]
    pub health: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub range: f32,
    #[serde(default)]
    pub dps: Option<f32>,
    #[serde(default)]
    pub sight: f32,
    #[serde(default)]
    pub build_power: f32,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub gloss: String,
    #[serde(default)]
    pub prose: String,
}

impl Entry {
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }

    /// One line for a roster table: "Pawn armpw (lab, t1, 54 metal): gloss".
    pub fn line(&self, internal: &str) -> String {
        let by = if self.made_by.is_empty() { String::new() } else { format!("{}, ", self.made_by.join("/")) };
        let gloss = if self.gloss.is_empty() { String::new() } else { format!(": {}", self.gloss) };
        format!("{} {internal} ({by}t{}, {:.0} metal){gloss}", self.name, self.tier, self.metal)
    }

    /// The numbers a player wants beside the prose.
    pub fn numbers(&self) -> String {
        let mut parts = vec![format!("{:.0} metal", self.metal), format!("{:.0} energy", self.energy)];
        if self.health > 0.0 {
            parts.push(format!("{:.0} health", self.health));
        }
        if self.speed > 0.0 {
            parts.push(format!("speed {:.0}", self.speed));
        }
        if self.range > 0.0 {
            parts.push(format!("range {:.0}", self.range));
        }
        if let Some(dps) = self.dps.filter(|d| *d > 0.0) {
            parts.push(format!("{dps:.0} damage a second"));
        }
        if self.sight > 0.0 {
            parts.push(format!("sees {:.0}", self.sight));
        }
        if self.build_power > 0.0 {
            parts.push(format!("build power {:.0}", self.build_power));
        }
        if !self.flags.is_empty() {
            parts.push(self.flags.join(", "));
        }
        parts.join(", ")
    }
}

static GLOSSARY: LazyLock<HashMap<String, Entry>> = LazyLock::new(|| parse(include_str!("../../../data/units.json")));

/// The entries keyed by internal name, from either layout: `{"units": {name: entry}}` or entries at the top level.
fn parse(text: &str) -> HashMap<String, Entry> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else { return HashMap::new() };
    let table = match value.get("units") {
        Some(serde_json::Value::Object(units)) => units.clone(),
        _ => value.as_object().cloned().unwrap_or_default(),
    };
    table
        .into_iter()
        .filter_map(|(name, v)| {
            if !v.is_object() || v.get("name").is_none() {
                return None;
            }
            serde_json::from_value::<Entry>(v).ok().map(|e| (name, e))
        })
        .collect()
}

pub fn entry(internal: &str) -> Option<&'static Entry> {
    GLOSSARY.get(internal)
}

/// How many units the glossary knows.
pub fn len() -> usize {
    GLOSSARY.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_layouts_parse_and_hand_fields_default() {
        let nested = parse(r#"{"source": "abc", "units": {"armpw": {"name": "Pawn", "tier": 1, "metal": 54, "gloss": "fast"}}}"#);
        assert_eq!(nested["armpw"].name, "Pawn");
        assert_eq!(nested["armpw"].line("armpw"), "Pawn armpw (t1, 54 metal): fast");
        let flat = parse(r#"{"source": "abc", "corak": {"name": "Grunt", "made_by": ["corlab"], "flags": ["amphibious"]}}"#);
        assert_eq!(flat.len(), 1);
        assert!(flat["corak"].has_flag("amphibious"));
        assert!(flat["corak"].prose.is_empty());
        assert!(parse("not json").is_empty());
    }
}
