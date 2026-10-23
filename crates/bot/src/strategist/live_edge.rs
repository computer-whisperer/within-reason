//! The live edge (docs/design/2026-10-21-brief-rework.md §11): the last games against this opponent on this map, shown to
//! the player and the commander as a part of their prompts. `run/live_edge.py close` writes an entry a match into the
//! store (its report in plain numbers and the previous player's and commander's own words) and a row into the store's
//! `index.jsonl`; this module reads the store named by `WITHIN_REASON_LIVE_EDGE` once, at the first hello, keys the game
//! by its map and its enemy seats, and keeps the entries of the same key within two weeks, newest first, at most three.
//! Unset, or an empty store: nothing is added and the prompts are as before. The key is computed the same way by the
//! tool (from the record's header and the start script) and here (from the hello), so the two must change together.

use std::path::Path;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use bot_protocol::{Controller, Hello, TeamInfo};

/// How old an entry may be, in days, and how many are shown.
const DAYS: u64 = 14;
const MOST: usize = 3;

static TEXT: OnceLock<String> = OnceLock::new();

/// The enemy seats sorted, each a name with its bonus: a person's lobby name; an AI's short name, version and profile;
/// `+N%` after it when the seat's income multiplier is not one. Gaia and our own side are left out.
pub fn key_of(ally_team: i32, teams: &[TeamInfo]) -> String {
    let mut names: Vec<String> = teams
        .iter()
        .filter(|t| t.ally_team != ally_team && t.controller != Controller::Gaia)
        .map(|t| {
            let name = match &t.controller {
                Controller::Person { name, .. } => name.clone(),
                Controller::Ai { short_name, version, profile, name } => {
                    let mut parts = vec![if short_name.is_empty() { name.clone() } else { short_name.clone() }];
                    if !version.is_empty() {
                        parts.push(version.clone());
                    }
                    if let Some(profile) = profile.as_ref().filter(|p| !p.is_empty()) {
                        parts.push(profile.clone());
                    }
                    parts.join(" ")
                }
                Controller::Gaia | Controller::Unknown => "unknown".to_string(),
            };
            let percent = t.income_multiplier.map(|m| ((m - 1.0) * 100.0).round() as i32).unwrap_or(0);
            if percent != 0 { format!("{name} {percent:+}%") } else { name }
        })
        .collect();
    names.sort();
    names.join(", ")
}

pub fn key(hello: &Hello) -> String {
    key_of(hello.ally_team, &hello.teams)
}

/// The store's entries for this map and key within `DAYS` of `now`, newest first, at most `MOST`: each entry's text
/// and its id. An unreadable index or entry is reported and skipped.
pub fn select(store: &Path, map: &str, key: &str, now: u64) -> Vec<(String, String)> {
    let index = match std::fs::read_to_string(store.join("index.jsonl")) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("[strategist] live edge: no index at {} ({e}); nothing shown", store.display());
            return Vec::new();
        }
    };
    let mut rows: Vec<(u64, String, String)> = index
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|row| row["map"].as_str() == Some(map) && row["key"].as_str() == Some(key))
        .filter_map(|row| {
            let date = row["date"].as_u64()?;
            let file = row["file"].as_str()?.to_string();
            let id = row["id"].as_str().unwrap_or("?").to_string();
            (now.saturating_sub(date) <= DAYS * 86_400).then_some((date, file, id))
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.truncate(MOST);
    rows.into_iter()
        .filter_map(|(_, file, id)| match std::fs::read_to_string(&file) {
            Ok(text) => Some((id, text)),
            Err(e) => {
                eprintln!("[strategist] live edge: entry {id} unreadable ({e})");
                None
            }
        })
        .collect()
}

/// The part's text for a set of entries: the heading and what the reader is to make of them, then the entries.
fn part(entries: &[(String, String)]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## The last games against this opponent on this map (the live edge)\n\n\
         Written when those games ended: the report by a tool from the record and the opponent's ground truth, the words \
         by the player and the commander who sat here then, in their own words, and that player's debrief for you. Newest \
         first; nothing older than two weeks, at most three games. The words are one player's reading of one game and may \
         be wrong; the report beside them is what the curves showed. Nothing here is a rule.\n\n{}\n",
        entries.iter().map(|(_, text)| text.as_str()).collect::<Vec<_>>().join("\n")
    )
}

/// Reads the store once, from the first seat's hello, and keeps the text the prompts carry.
pub fn note(hello: &Hello) {
    TEXT.get_or_init(|| {
        let Some(store) = std::env::var_os("WITHIN_REASON_LIVE_EDGE") else {
            return String::new();
        };
        let store = Path::new(&store);
        let key = key(hello);
        let map = hello.map.name.as_str();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let entries = select(store, map, &key, now);
        let against = if key.is_empty() { "nobody" } else { key.as_str() };
        if entries.is_empty() {
            eprintln!("[strategist] live edge: no entry for {map} against {against} in {}", store.display());
        } else {
            eprintln!("[strategist] live edge: {} entries for {map} against {against} ({})", entries.len(), entries.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>().join(", "));
        }
        part(&entries)
    });
}

/// The part the player's and the commander's prompts carry: empty without a store or an entry.
pub fn text() -> String {
    TEXT.get().cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(team: i32, ally_team: i32, controller: Controller, multiplier: Option<f32>) -> TeamInfo {
        TeamInfo { team, ally_team, side: "Armada".into(), controller, start_pos: None, color: None, income_multiplier: multiplier }
    }

    #[test]
    fn the_key_names_the_enemy_seats_sorted_with_their_bonus_as_the_tool_does() {
        let teams = vec![
            team(0, 0, Controller::Ai { name: "WReason0.1(1)".into(), short_name: "WReason".into(), version: "0.1".into(), profile: None }, Some(1.0)),
            team(1, 1, Controller::Ai { name: "BARb(2)".into(), short_name: "BARb".into(), version: "stable".into(), profile: Some("hard_aggressive".into()) }, Some(1.35)),
            team(2, 2, Controller::Gaia, None),
        ];
        assert_eq!(key_of(0, &teams), "BARb stable hard_aggressive +35%");
        let people = vec![
            team(0, 0, Controller::Person { name: "[gecko]u6bkep".into(), skill: Some(29.9) }, Some(1.0)),
            team(1, 0, Controller::Person { name: "[gecko]thebluegecko".into(), skill: None }, None),
            team(2, 1, Controller::Ai { name: "WReason0.1(1)".into(), short_name: "WReason".into(), version: "".into(), profile: None }, Some(1.0)),
        ];
        assert_eq!(key_of(1, &people), "[gecko]thebluegecko, [gecko]u6bkep");
        assert_eq!(key_of(0, &people), "WReason");
    }

    #[test]
    fn the_selection_keeps_the_newest_three_of_this_map_and_key_within_two_weeks() {
        let store = std::env::temp_dir().join(format!("live-edge-test-{}", std::process::id()));
        std::fs::create_dir_all(&store).unwrap();
        let now = 1_800_000_000u64;
        let mut index = String::new();
        for (i, (age_days, map, key)) in [(1, "Comet", "BARb"), (3, "Comet", "BARb"), (2, "Comet", "BARb"), (5, "Comet", "BARb"), (1, "Glitters", "BARb"), (20, "Comet", "BARb"), (1, "Comet", "Apex")].iter().enumerate() {
            let file = store.join(format!("e{i}.md"));
            std::fs::write(&file, format!("entry {i}\n")).unwrap();
            index.push_str(&format!("{}\n", serde_json::json!({"id": format!("g{i}"), "date": now - age_days * 86_400, "map": map, "key": key, "file": file})));
        }
        std::fs::write(store.join("index.jsonl"), index).unwrap();
        let picked: Vec<String> = select(&store, "Comet", "BARb", now).into_iter().map(|(id, _)| id).collect();
        assert_eq!(picked, vec!["g0", "g2", "g1"]);
        assert!(select(&store, "Comet", "nobody", now).is_empty());
        assert!(part(&[]).is_empty());
        assert!(part(&select(&store, "Comet", "BARb", now)).contains("entry 0"));
        std::fs::remove_dir_all(&store).unwrap();
    }
}
