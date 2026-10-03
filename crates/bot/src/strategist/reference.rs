//! The report's reference line: where this seat stands against the experienced players' games on this map, minute
//! by minute (`run/reference_set.py --json-out`, `docs/briefs/reference/<map>-<faction>.json`). A description of
//! their games, said every turn so the comparison is not left to memory; there is no line on a map or for a
//! faction without a table, in a game with more than one seat a side, or past the table's last minute.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;

use super::shared::{Briefing, Score};

fn table(map: &str, faction: &str) -> Option<Value> {
    static TABLES: Mutex<Option<HashMap<String, Option<Value>>>> = Mutex::new(None);
    let slug: String = map.to_lowercase().chars().map(|c| if c == ' ' { '-' } else { c }).collect();
    let path = format!("docs/briefs/reference/{slug}-{faction}.json");
    let mut tables = TABLES.lock().unwrap();
    tables.get_or_insert_with(HashMap::new).entry(path.clone()).or_insert_with(|| crate::texts::read_file(&path).and_then(|t| serde_json::from_str(&t).ok())).clone()
}

pub fn line(briefing: &Briefing, score: &Score) -> Option<String> {
    let table = table(&score.map, &score.faction)?;
    line_from(&table, briefing, score)
}

fn line_from(table: &Value, briefing: &Briefing, score: &Score) -> Option<String> {
    let minute = score.frame / 1800;
    let minutes = table["minutes"].as_object()?;
    let last = minutes.keys().filter_map(|k| k.parse::<i32>().ok()).max()?;
    if minute < 1 {
        return None;
    }
    if minute > last {
        return Some(format!("reference: the experienced players' games on this map are over by {last}:00; past it there is no table to stand against"));
    }
    // The table skips minutes late on: the nearest one at or before now.
    let at = minutes.keys().filter_map(|k| k.parse::<i32>().ok()).filter(|m| *m <= minute).max()?;
    let row = &minutes[&at.to_string()];
    let cell = |key: &str, ours: f32| {
        let q = &row[key];
        let n = |v: &Value| v.as_f64().map_or("-".to_string(), |v| format!("{v:.0}"));
        format!("{ours:.0} ({} / {} / {}, {})", n(&q["p25"]), n(&q["median"]), n(&q["p75"]), n(&q["won"]))
    };
    let faction = match table["faction"].as_str() {
        Some("arm") => "Armada",
        Some("cor") => "Cortex",
        _ => "",
    };
    Some(format!(
        "reference, {} {faction} sides of duels between people at OS {:.0} and above on this map, at {at}:00 (ours, then their lower quartile / median / upper quartile, and their winners' median): extractors {} | constructors {} | metal income {} | energy income {} | army metal {} | soldiers {} | army metal in his half {}. Their starts are {} apart; what they did, not a target.",
        table["sides"], table["floor"].as_f64().unwrap_or(0.0),
        cell("extractors", score.extractors as f32), cell("constructors", briefing.counts.constructors as f32), cell("metal_income", briefing.metal.income),
        cell("energy_income", briefing.energy.income), cell("army_metal", score.army_metal as f32), cell("soldiers", score.soldiers as f32),
        cell("army_metal_in_his_half", score.army_metal_in_his_half as f32), table["starts_apart"],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_line_stands_ours_beside_the_pool_at_the_minute() {
        let q = |m: f64| serde_json::json!({ "p25": m - 2.0, "median": m, "p75": m + 3.0, "won": m + 1.0 });
        let row = |m: f64| serde_json::json!({ "extractors": q(m), "constructors": q(5.0), "metal_income": q(36.0), "energy_income": q(320.0), "army_metal": q(1500.0), "soldiers": q(17.0), "army_metal_in_his_half": q(361.0) });
        let table = serde_json::json!({ "faction": "arm", "floor": 40, "sides": 41, "starts_apart": 5928, "minutes": { "8": row(15.0), "10": row(17.0) } });
        let mut score = Score { frame: 9 * 1800 + 40, extractors: 18, ..Score::default() };
        let briefing = Briefing::default();
        let line = line_from(&table, &briefing, &score).unwrap();
        assert!(line.contains("at 8:00") && line.contains("extractors 18 (13 / 15 / 18, 16)"), "{line}");
        score.frame = 11 * 1800;
        assert!(line_from(&table, &briefing, &score).unwrap().contains("over by 10:00"));
        score.frame = 600;
        assert!(line_from(&table, &briefing, &score).is_none());
    }
}
