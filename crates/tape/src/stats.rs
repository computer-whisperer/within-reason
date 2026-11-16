//! The game's numbers a second, a team at a time, worked out from the tape's tables alone (no slice is read): what
//! the page's strip, charts and Stats tab show (`viewer/stats.js`, through `wr_stats`), and what `tape stats` prints
//! for a script or a review agent. `docs/design/2026-11-16-viewer-stats.md`.
//!
//! Every metric is additive: a side's number is the sum of its teams'. Three sources: the `TEAM` table's rows a
//! second (what is stored, the capacity, income, spend, pull; what was sent and received, summed here into running
//! totals), the engine's own running statistics in the same table, and the `UNIT` table with the types (how many
//! stand finished, the extractors, the build power, count and metal worth by class, the metal worth killed and
//! lost).
//!
//! A class is an approximation of BAR's own (its team stats gadget's `bucketOf`): a format 5 tape's types do not
//! say `unitgroup` or the move class, so ships count as army and the strategic buildings as defence.

use serde_json::Value;

use crate::format::{Error, NEVER, Result};
use crate::read::Tape;

/// Frames a second: a metric has a value a second, the state at frame `second * 30`.
pub const FPS: u32 = 30;

/// How a metric reads: what stands now, a flow a second, a total that only runs up, a number of units, metal worth.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Level,
    Rate,
    Total,
    Count,
    Value,
}

impl Kind {
    pub fn word(self) -> &'static str {
        match self {
            Kind::Level => "level",
            Kind::Rate => "rate",
            Kind::Total => "total",
            Kind::Count => "count",
            Kind::Value => "value",
        }
    }
}

pub struct Metric {
    pub key: &'static str,
    pub label: &'static str,
    /// The view it belongs to, after the game's own statistics window: live, economy, combat, units, composition.
    pub group: &'static str,
    pub kind: Kind,
}

const fn m(key: &'static str, label: &'static str, group: &'static str, kind: Kind) -> Metric {
    Metric { key, label, group, kind }
}

/// What a unit counts as. The order is the order of the `value_*` and `count_*` metrics.
pub const CLASSES: [&str; 7] = ["army", "air", "defence", "factories", "builders", "economy", "utility"];

pub const METRICS: &[Metric] = &[
    m("metal", "metal stored", "live", Kind::Level),
    m("metal_store", "metal capacity", "live", Kind::Level),
    m("metal_income", "metal income", "live", Kind::Rate),
    m("metal_spend", "metal spend", "live", Kind::Rate),
    m("metal_pull", "metal pull", "live", Kind::Rate),
    m("energy", "energy stored", "live", Kind::Level),
    m("energy_store", "energy capacity", "live", Kind::Level),
    m("energy_income", "energy income", "live", Kind::Rate),
    m("energy_spend", "energy spend", "live", Kind::Rate),
    m("energy_pull", "energy pull", "live", Kind::Rate),
    m("build_power", "build power", "live", Kind::Level),
    m("metal_produced", "metal produced", "economy", Kind::Total),
    m("metal_used", "metal used", "economy", Kind::Total),
    m("metal_excess", "metal wasted", "economy", Kind::Total),
    m("metal_sent", "metal sent", "economy", Kind::Total),
    m("metal_received", "metal received", "economy", Kind::Total),
    m("energy_produced", "energy produced", "economy", Kind::Total),
    m("energy_used", "energy used", "economy", Kind::Total),
    m("energy_excess", "energy wasted", "economy", Kind::Total),
    m("energy_sent", "energy sent", "economy", Kind::Total),
    m("energy_received", "energy received", "economy", Kind::Total),
    m("extractors", "extractors", "economy", Kind::Count),
    m("damage_dealt", "damage dealt", "combat", Kind::Total),
    m("damage_received", "damage received", "combat", Kind::Total),
    m("units_killed", "units killed", "combat", Kind::Total),
    m("units_died", "units lost", "combat", Kind::Total),
    m("killed_value", "metal worth killed", "combat", Kind::Total),
    m("lost_value", "metal worth lost", "combat", Kind::Total),
    m("units", "units", "units", Kind::Count),
    m("units_produced", "units made", "units", Kind::Total),
    m("value_army", "army worth", "composition", Kind::Value),
    m("value_air", "air worth", "composition", Kind::Value),
    m("value_defence", "defence worth", "composition", Kind::Value),
    m("value_factories", "factories worth", "composition", Kind::Value),
    m("value_builders", "builders worth", "composition", Kind::Value),
    m("value_economy", "economy worth", "composition", Kind::Value),
    m("value_utility", "utility worth", "composition", Kind::Value),
    m("count_army", "army", "composition", Kind::Count),
    m("count_air", "aircraft", "composition", Kind::Count),
    m("count_defence", "defences", "composition", Kind::Count),
    m("count_factories", "factories", "composition", Kind::Count),
    m("count_builders", "builders", "composition", Kind::Count),
    m("count_economy", "economy buildings", "composition", Kind::Count),
    m("count_utility", "utility", "composition", Kind::Count),
];

pub fn metric(key: &str) -> Option<usize> {
    METRICS.iter().position(|m| m.key == key)
}

fn at(key: &str) -> usize {
    metric(key).unwrap_or_else(|| panic!("no metric {key}"))
}

/// A team of the game (Gaia left out), in the header's order.
#[derive(Debug, Clone)]
pub struct Team {
    pub team: u8,
    pub ally: u8,
    pub name: String,
}

/// A unit type as the numbers need it.
#[derive(Debug, Clone)]
pub struct Type {
    pub id: u32,
    pub name: String,
    pub human: String,
    pub metal: f32,
    pub class: usize,
    pub extractor: bool,
    pub build_power: f32,
}

pub struct Stats {
    /// How many seconds have a value: second 0 to the tape's last.
    pub seconds: usize,
    pub teams: Vec<Team>,
    pub types: Vec<Type>,
    /// `[team][metric][second]`, the teams in `teams`' order and the metrics in `METRICS`'.
    pub values: Vec<f32>,
}

fn parse(text: &str, what: &str) -> Result<Value> {
    if text.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(text).map_err(|e| Error(format!("the tape's {what} does not parse: {e}")))
}

fn num(v: &Value, key: &str) -> f64 {
    match v.get(key) {
        Some(Value::Bool(b)) => *b as u8 as f64,
        Some(other) => other.as_f64().unwrap_or(0.0),
        None => 0.0,
    }
}

/// Whether a field says yes: true, a number that is not zero, a list or a table with something in it.
fn yes(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        _ => false,
    }
}

/// The class of a unit type (an index into `CLASSES`), from what a format 5 tape says of it. The order follows the
/// game's own rule: a factory, then anything that builds (a commander, a constructor, a construction turret), then
/// by whether it moves.
pub fn class_of(def: &Value, armed: bool) -> usize {
    let class = |name: &str| CLASSES.iter().position(|c| *c == name).unwrap();
    if yes(def, "factory") {
        return class("factories");
    }
    if yes(def, "builder") {
        return class("builders");
    }
    if !yes(def, "mobile") {
        // What draws metal or turns wind, tide or energy into income is the economy's even with a gun on it; a
        // building that shoots is a defence even when it keeps a store of energy for its own gun (a Pulsar, a
        // Bulwark); the rest of what makes or stores is the economy's.
        let draws = num(def, "extracts") > 0.0 || num(def, "wind") > 0.0 || num(def, "tidal") > 0.0 || yes(def, "converter");
        let makes = num(def, "energy_make") > 0.0 || num(def, "energy_upkeep") < 0.0 || num(def, "metal_store") > 0.0 || num(def, "energy_store") > 0.0;
        return class(if draws { "economy" } else if armed { "defence" } else if makes { "economy" } else { "utility" });
    }
    class(if yes(def, "air") { "air" } else if armed { "army" } else { "utility" })
}

/// The unit types of a tape's `DEFS`.
pub fn types(defs: &Value) -> Vec<Type> {
    // A weapon that does damage: a shield is a weapon type too, and so is a target painter.
    let harmful: std::collections::HashSet<u64> = defs
        .get("weapons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|w| !yes(w, "shield") && num(w, "damage") > 0.0)
        .filter_map(|w| w.get("id").and_then(Value::as_u64))
        .collect();
    defs.get("units")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|d| {
            let armed = d.get("weapons").and_then(Value::as_array).into_iter().flatten().any(|w| w.as_u64().is_some_and(|id| harmful.contains(&id)));
            let text = |key: &str| d.get(key).and_then(Value::as_str).unwrap_or("").to_string();
            Type {
                id: num(d, "id") as u32,
                name: text("name"),
                human: text("human"),
                metal: num(d, "metal") as f32,
                class: class_of(d, armed),
                extractor: num(d, "extracts") > 0.0,
                build_power: if yes(d, "builder") || yes(d, "factory") { num(d, "build_speed") as f32 } else { 0.0 },
            }
        })
        .collect()
}

/// The first second whose state has what happened at a frame.
fn second_of(frame: u32) -> usize {
    frame.div_ceil(FPS) as usize
}

pub fn compute(tape: &Tape) -> Result<Stats> {
    let head = parse(&tape.head, "header")?;
    let defs = parse(&tape.defs, "types")?;
    let table = parse(&tape.teams, "teams table")?;
    let teams: Vec<Team> = head
        .get("teams")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|t| !yes(t, "gaia"))
        .map(|t| {
            let name = t.get("name").and_then(Value::as_str).unwrap_or("").trim().to_string();
            let side = t.get("side").and_then(Value::as_str).unwrap_or("");
            let team = num(t, "team") as u8;
            Team { team, ally: num(t, "ally") as u8, name: if !name.is_empty() { name } else if !side.is_empty() { side.to_string() } else { format!("team {team}") } }
        })
        .collect();
    let seconds = (tape.last_frame / FPS) as usize + 1;
    let types = types(&defs);
    let mut stats = Stats { seconds, teams, types, values: Vec::new() };
    stats.values = vec![0.0; stats.teams.len() * METRICS.len() * seconds];
    let index_of: std::collections::HashMap<u8, usize> = stats.teams.iter().enumerate().map(|(i, t)| (t.team, i)).collect();
    let stride = METRICS.len() * seconds;
    let cell = |team: usize, metric: usize, second: usize| team * stride + metric * seconds + second;

    // The table's rows a second: a level or a rate stands until the next row; what was sent and received that
    // second runs up into a total.
    let columns: Vec<&str> = table.get("columns").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
    let frames: Vec<u32> = table.get("frames").and_then(Value::as_array).into_iter().flatten().map(|f| f.as_f64().unwrap_or(0.0) as u32).collect();
    let held = ["metal", "metal_store", "metal_income", "metal_spend", "metal_pull", "energy", "energy_store", "energy_income", "energy_spend", "energy_pull"];
    let summed = ["metal_sent", "metal_received", "energy_sent", "energy_received"];
    for (ti, team) in stats.teams.iter().enumerate() {
        let rows = table.get("teams").and_then(|t| t.get(team.team.to_string())).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        for (names, running) in [(&held[..], false), (&summed[..], true)] {
            for name in names {
                let Some(column) = columns.iter().position(|c| c == name) else { continue };
                let metric = at(name);
                let mut next = 0;
                let mut value = 0.0f32;
                for second in 0..seconds {
                    // A row is the second its frame is nearest: the widget writes one a second, not always on it.
                    while next < rows.len() && next < frames.len() && ((frames[next] + FPS / 2) / FPS) as usize <= second {
                        let v = rows[next].get(column).and_then(Value::as_f64).unwrap_or(0.0) as f32;
                        value = if running { value + v } else { v };
                        next += 1;
                    }
                    stats.values[cell(ti, metric, second)] = value;
                }
            }
        }
        // The engine's statistics, a row when they changed: each stands until the next.
        let stat_columns: Vec<&str> = table.get("stat_columns").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
        let rows = table.get("stats").and_then(|t| t.get(team.team.to_string())).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        for (column, name) in stat_columns.iter().enumerate() {
            let Some(metric) = metric(name) else { continue };
            let mut next = 0;
            let mut value = 0.0f32;
            for second in 0..seconds {
                while next < rows.len() && rows[next].get(0).and_then(Value::as_f64).unwrap_or(0.0) as u32 <= second as u32 * FPS {
                    value = rows[next].get(column + 1).and_then(Value::as_f64).unwrap_or(0.0) as f32;
                    next += 1;
                }
                stats.values[cell(ti, metric, second)] = value;
            }
        }
    }

    // The unit lives: a step up at the second a unit stands finished and a step down at the second it is gone (or
    // given away), summed after; a death is a step up in what its team lost and its killer's team killed.
    let type_of: std::collections::HashMap<u32, &Type> = stats.types.iter().map(|t| (t.id, t)).collect();
    let (units, extractors, build_power) = (at("units"), at("extractors"), at("build_power"));
    let (value0, count0) = (at("value_army"), at("count_army"));
    let (killed, lost) = (at("killed_value"), at("lost_value"));
    let stepped: Vec<usize> = [units, extractors, build_power, killed, lost].into_iter().chain((0..CLASSES.len()).flat_map(|c| [value0 + c, count0 + c])).collect();
    let mut steps = vec![0.0f32; stats.values.len()];
    let ally_of = |team: u8| index_of.get(&team).map(|i| stats.teams[*i].ally);
    for unit in &tape.units {
        let Some(kind) = type_of.get(&unit.def) else { continue };
        if unit.finished == NEVER || unit.finished >= unit.destroyed {
            continue;
        }
        // Whose it was, a stretch at a time.
        let mut owners = vec![(unit.created, unit.team)];
        owners.extend(unit.given.iter().copied());
        for (i, (from, team)) in owners.iter().enumerate() {
            let Some(ti) = index_of.get(team) else { continue };
            let until = owners.get(i + 1).map_or(unit.destroyed, |(f, _)| *f).min(unit.destroyed);
            let from = (*from).max(unit.finished);
            if from >= until {
                continue;
            }
            let (a, b) = (second_of(from), if until == NEVER { seconds } else { second_of(until) });
            if a >= seconds || a >= b {
                continue;
            }
            for (metric, v) in [(units, 1.0), (extractors, kind.extractor as u8 as f32), (build_power, kind.build_power), (value0 + kind.class, kind.metal), (count0 + kind.class, 1.0)] {
                steps[cell(*ti, metric, a)] += v;
                if b < seconds {
                    steps[cell(*ti, metric, b)] -= v;
                }
            }
        }
        if unit.destroyed != NEVER && second_of(unit.destroyed) < seconds {
            let second = second_of(unit.destroyed);
            let owner = unit.team_at(unit.destroyed.saturating_sub(1));
            if let Some(ti) = index_of.get(&owner) {
                steps[cell(*ti, lost, second)] += kind.metal;
            }
            // (The killer's row is one more than its place in the table; none is zero.)
            if let Some(killer) = unit.killer.checked_sub(1).and_then(|row| tape.units.get(row as usize)) {
                let by = killer.team_at(unit.destroyed.saturating_sub(1));
                if let (Some(ki), true) = (index_of.get(&by), ally_of(by) != ally_of(owner)) {
                    steps[cell(*ki, killed, second)] += kind.metal;
                }
            }
        }
    }
    for team in 0..stats.teams.len() {
        for metric in &stepped {
            let mut sum = 0.0f32;
            for second in 0..seconds {
                sum += steps[cell(team, *metric, second)];
                stats.values[cell(team, *metric, second)] = sum;
            }
        }
    }
    Ok(stats)
}

impl Stats {
    /// A team's values of a metric, one a second.
    pub fn series(&self, team: usize, metric: usize) -> &[f32] {
        let from = (team * METRICS.len() + metric) * self.seconds;
        &self.values[from..from + self.seconds]
    }

    /// The sides (ally teams) in order, each with the places in `teams` of its teams.
    pub fn sides(&self) -> Vec<(u8, Vec<usize>)> {
        let mut allies: Vec<u8> = self.teams.iter().map(|t| t.ally).collect();
        allies.sort();
        allies.dedup();
        allies.into_iter().map(|ally| (ally, (0..self.teams.len()).filter(|i| self.teams[*i].ally == ally).collect())).collect()
    }

    /// A side's name: its first team's, and how many more.
    pub fn side_name(&self, teams: &[usize]) -> String {
        let first = teams.first().map_or("", |i| self.teams[*i].name.as_str());
        if teams.len() > 1 { format!("{first} +{}", teams.len() - 1) } else { first.to_string() }
    }

    /// What the page reads beside the values: the seconds, the teams and the metrics, as JSON.
    pub fn describe(&self) -> String {
        serde_json::json!({
            "seconds": self.seconds,
            "teams": self.teams.iter().map(|t| serde_json::json!({ "team": t.team, "ally": t.ally, "name": t.name })).collect::<Vec<_>>(),
            "metrics": METRICS.iter().map(|m| serde_json::json!({ "key": m.key, "label": m.label, "group": m.group, "kind": m.kind.word() })).collect::<Vec<_>>(),
            "classes": CLASSES,
            "types": self.types.iter().map(|t| serde_json::json!({ "id": t.id, "class": CLASSES[t.class] })).collect::<Vec<_>>(),
        })
        .to_string()
    }
}
