//! The game's numbers a second, a team at a time, worked out from the tape's tables alone (no slice is read): what
//! the page's strip, charts and Stats tab show (`viewer/stats.js`, through `wr_stats`), and what `tape stats` prints
//! for a script or a review agent. `docs/design/2026-11-16-viewer-stats.md`.
//!
//! Every metric is additive: a side's number is the sum of its teams'. Three sources: the `TEAM` table's rows a
//! second (what is stored, the capacity, income, spend, pull; the converters' energy in use and capacity; the build
//! power and how much is in use; the constructors and plants and the idle of each; the orders given; what was
//! reclaimed; what was sent and received, summed here into running totals), the engine's own running statistics in
//! the same table, and the `UNIT` table with the types (how many stand finished, the extractors, count and metal
//! worth by class, the metal worth killed and lost). The wind is the game's, not a team's: `Stats::wind`.
//!
//! A class is the game's own (its team stats gadget's `bucketOf`, rule for rule).

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

/// What a unit counts as, the game's own classes in its order: the order of the `value_*` and `count_*` metrics.
pub const CLASSES: [&str; 9] = ["army", "air", "sea", "defence", "strategic", "factories", "builders", "economy", "utility"];

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
    m("build_power_used", "build power in use", "live", Kind::Level),
    m("conv_use", "conversion: energy in use", "live", Kind::Rate),
    m("conv_capacity", "conversion: capacity", "live", Kind::Rate),
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
    m("metal_reclaimed", "metal reclaimed", "economy", Kind::Total),
    m("energy_reclaimed", "energy reclaimed", "economy", Kind::Total),
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
    m("value_sea", "sea worth", "composition", Kind::Value),
    m("value_defence", "defence worth", "composition", Kind::Value),
    m("value_strategic", "strategic worth", "composition", Kind::Value),
    m("value_factories", "factories worth", "composition", Kind::Value),
    m("value_builders", "builders worth", "composition", Kind::Value),
    m("value_economy", "economy worth", "composition", Kind::Value),
    m("value_utility", "utility worth", "composition", Kind::Value),
    m("count_army", "army", "composition", Kind::Count),
    m("count_air", "aircraft", "composition", Kind::Count),
    m("count_sea", "ships", "composition", Kind::Count),
    m("count_defence", "defences", "composition", Kind::Count),
    m("count_strategic", "strategic buildings", "composition", Kind::Count),
    m("count_factories", "factories", "composition", Kind::Count),
    m("count_builders", "builders", "composition", Kind::Count),
    m("count_economy", "economy buildings", "composition", Kind::Count),
    m("count_utility", "utility", "composition", Kind::Count),
    m("cons", "constructors", "activity", Kind::Count),
    m("idle_cons", "constructors without an order", "activity", Kind::Count),
    m("labs", "plants", "activity", Kind::Count),
    m("idle_labs", "plants with an empty line", "activity", Kind::Count),
    m("orders", "orders given a second", "activity", Kind::Rate),
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
    /// What the class was read from: the game's `unitgroup` and the type's move class.
    pub group: String,
    pub move_class: String,
}

pub struct Stats {
    /// How many seconds have a value: second 0 to the tape's last.
    pub seconds: usize,
    pub teams: Vec<Team>,
    pub types: Vec<Type>,
    /// `[team][metric][second]`, the teams in `teams`' order and the metrics in `METRICS`'.
    pub values: Vec<f32>,
    /// The wind's strength a second.
    pub wind: Vec<f32>,
    /// The metrics this game has no numbers for (a game version that keeps no count of what is reclaimed): their
    /// values are zeros and no reader should show them.
    pub absent: Vec<&'static str>,
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

/// The class of a unit type (an index into `CLASSES`): the game's own rule, its team stats gadget's `bucketOf`.
/// Builders come before the rest, so a constructor with a gun is a builder; strategic and economy come before the
/// split by medium, so a floating nuke silo is strategic, not sea. `armed` is whether the type has a weapon at all.
pub fn class_of(def: &Value, armed: bool) -> usize {
    let class = |name: &str| CLASSES.iter().position(|c| *c == name).unwrap();
    let group = def.get("group").and_then(Value::as_str).unwrap_or("");
    if yes(def, "factory") {
        return class("factories");
    }
    if yes(def, "builder") {
        return class("builders");
    }
    if group == "nuke" || group == "antinuke" || yes(def, "lrpc") {
        return class("strategic");
    }
    if group == "energy" || group == "metal" {
        return class("economy");
    }
    if group == "util" {
        return class("utility");
    }
    let armed = armed || ["weapon", "aa", "sub", "emp", "explo", "weaponaa", "weaponsub"].contains(&group);
    if num(def, "speed") == 0.0 {
        return class(if armed { "defence" } else { "utility" });
    }
    if yes(def, "air") {
        return class("air");
    }
    // A move class is named by its kind and a number. The game's files write BOAT4, UBOAT4, EPICSHIP; the engine
    // hands a widget the name in lower case (boat4, uboat4: read off a tape, 2026-11-16, after the first cut of
    // this rule matched the files' spelling alone and counted every ship as army).
    let kind: String = def.get("move_class").and_then(Value::as_str).unwrap_or("").chars().take_while(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_uppercase()).collect();
    if ["BOAT", "UBOAT", "EPICSHIP"].contains(&kind.as_str()) {
        return class("sea");
    }
    class(if armed { "army" } else { "utility" })
}

/// The unit types of a tape's `DEFS`.
pub fn types(defs: &Value) -> Vec<Type> {
    defs.get("units")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|d| {
            let armed = d.get("weapons").and_then(Value::as_array).is_some_and(|w| !w.is_empty());
            let text = |key: &str| d.get(key).and_then(Value::as_str).unwrap_or("").to_string();
            Type {
                id: num(d, "id") as u32,
                name: text("name"),
                human: text("human"),
                metal: num(d, "metal") as f32,
                class: class_of(d, armed),
                extractor: num(d, "extracts") > 0.0,
                group: text("group"),
                move_class: text("move_class"),
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
    let mut stats = Stats { seconds, teams, types, values: Vec::new(), wind: vec![0.0; seconds], absent: Vec::new() };
    stats.values = vec![0.0; stats.teams.len() * METRICS.len() * seconds];
    let index_of: std::collections::HashMap<u8, usize> = stats.teams.iter().enumerate().map(|(i, t)| (t.team, i)).collect();
    let stride = METRICS.len() * seconds;
    let cell = |team: usize, metric: usize, second: usize| team * stride + metric * seconds + second;

    // The table's rows a second: a level or a rate stands until the next row; what was sent and received that
    // second runs up into a total.
    let columns: Vec<&str> = table.get("columns").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
    let frames: Vec<u32> = table.get("frames").and_then(Value::as_array).into_iter().flatten().map(|f| f.as_f64().unwrap_or(0.0) as u32).collect();
    let held = [
        "metal", "metal_store", "metal_income", "metal_spend", "metal_pull", "energy", "energy_store", "energy_income", "energy_spend", "energy_pull", "conv_use", "conv_capacity",
        "metal_reclaimed", "energy_reclaimed", "build_power", "build_power_used", "cons", "idle_cons", "labs", "idle_labs", "orders",
    ];
    let summed = ["metal_sent", "metal_received", "energy_sent", "energy_received"];
    // The wind, a value a row.
    let winds: Vec<f32> = table.get("wind").and_then(Value::as_array).into_iter().flatten().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect();
    let (mut next, mut value) = (0, 0.0f32);
    for second in 0..seconds {
        while next < winds.len() && next < frames.len() && ((frames[next] + FPS / 2) / FPS) as usize <= second {
            value = winds[next];
            next += 1;
        }
        stats.wind[second] = value;
    }
    // What the game counts only in some of its versions is written as -1 where it does not: absent, when no team
    // ever has a count.
    for name in ["metal_reclaimed", "energy_reclaimed"] {
        let column = columns.iter().position(|c| *c == name);
        let counted = column.is_some_and(|column| {
            table.get("teams").and_then(Value::as_object).into_iter().flat_map(|t| t.values()).filter_map(Value::as_array).flatten().any(|row| row.get(column).and_then(Value::as_f64).is_some_and(|v| v >= 0.0))
        });
        if !counted {
            stats.absent.push(METRICS[at(name)].key);
        }
    }
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
                        let v = rows[next].get(column).and_then(Value::as_f64).unwrap_or(0.0).max(0.0) as f32;
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
    let (units, extractors) = (at("units"), at("extractors"));
    let (value0, count0) = (at("value_army"), at("count_army"));
    let (killed, lost) = (at("killed_value"), at("lost_value"));
    let stepped: Vec<usize> = [units, extractors, killed, lost].into_iter().chain((0..CLASSES.len()).flat_map(|c| [value0 + c, count0 + c])).collect();
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
            for (metric, v) in [(units, 1.0), (extractors, kind.extractor as u8 as f32), (value0 + kind.class, kind.metal), (count0 + kind.class, 1.0)] {
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
            "wind": self.wind,
            "absent": self.absent,
            "types": self.types.iter().map(|t| serde_json::json!({ "id": t.id, "class": CLASSES[t.class] })).collect::<Vec<_>>(),
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Pose, Vital};
    use crate::tables::UnitRow;

    fn unit(def: u32, team: u8, finished: u32, destroyed: u32, killer: u32, given: Vec<(u32, u8)>) -> UnitRow {
        UnitRow {
            engine_id: 0, def, team, created: 0, finished, destroyed, killer, builder: 0, mobile: true, max_health: 100.0,
            birth: Pose::default(), birth_vital: Vital { health: 0, build: 0, paralyze: 0, flags: 0 }, death: [0; 3], given,
        }
    }

    /// Two teams on two sides: a tank each (ours killed by theirs at frame 90), an extractor of ours taken by them
    /// at frame 150, a constructor of theirs never finished.
    fn game() -> Tape {
        let head = serde_json::json!({ "teams": [
            { "team": 0, "ally": 0, "name": "a" }, { "team": 1, "ally": 1, "name": "" , "side": "cortex" }, { "team": 2, "ally": 2, "gaia": true, "name": "gaia" } ] });
        let defs = serde_json::json!({
            "units": [
                { "id": 1, "name": "tank", "human": "Tank", "speed": 60, "metal": 100, "weapons": [7], "group": "weapon", "move_class": "tank3" },
                { "id": 2, "name": "mex", "human": "Extractor", "speed": 0, "metal": 50, "extracts": 0.001, "weapons": [], "group": "metal" },
                { "id": 3, "name": "con", "human": "Constructor", "speed": 40, "builder": true, "build_speed": 80, "metal": 120, "weapons": [7], "group": "builder" },
                { "id": 4, "name": "gun", "human": "Gun", "speed": 0, "metal": 80, "energy_store": 100, "weapons": [7], "group": "weapon" },
                { "id": 5, "name": "wall", "human": "Wall", "speed": 0, "metal": 5, "weapons": [] },
                { "id": 6, "name": "ship", "human": "Ship", "speed": 50, "metal": 300, "weapons": [7], "group": "weapon", "move_class": "boat4" },
                { "id": 14, "name": "sub", "human": "Sub", "speed": 50, "metal": 300, "weapons": [7], "group": "sub", "move_class": "uboat4" },
                { "id": 15, "name": "flagship", "human": "Flagship", "speed": 30, "metal": 21000, "weapons": [7], "group": "weapon", "move_class": "EPICSHIP" },
                { "id": 16, "name": "hover", "human": "Hovercraft", "speed": 80, "metal": 75, "weapons": [7], "group": "weapon", "move_class": "hover2" },
                { "id": 7, "name": "silo", "human": "Silo", "speed": 0, "metal": 8000, "weapons": [7], "group": "nuke" },
                { "id": 8, "name": "floating silo", "human": "Silo", "speed": 20, "metal": 8000, "weapons": [7], "group": "nuke", "move_class": "boat4" },
                { "id": 9, "name": "radar", "human": "Radar", "speed": 0, "metal": 60, "weapons": [], "group": "util" },
                { "id": 10, "name": "plane", "human": "Plane", "speed": 200, "air": 1, "metal": 150, "weapons": [7], "group": "weapon" },
                { "id": 11, "name": "scout", "human": "Scout", "speed": 90, "metal": 30, "weapons": [], "move_class": "bot1" },
                { "id": 12, "name": "plant", "human": "Plant", "speed": 0, "factory": 1, "builder": 1, "metal": 600, "weapons": [] },
                { "id": 13, "name": "long gun", "human": "Long gun", "speed": 0, "metal": 4500, "weapons": [7], "group": "weapon", "lrpc": 1 } ],
            "weapons": [{ "id": 7, "damage": 10 }],
        });
        // (a row: the sixteen of format 5, then conversion in use and capacity, what was reclaimed (-1: not counted),
        // build power and its use, constructors and the idle, plants and the idle, the orders that second)
        let row = |income: f64| serde_json::json!([10, 500, 3, income, 4, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 3, 10, -1, -1, 300, 150, 2, 1, 1, 0, 4]);
        let teams = serde_json::json!({
            "frames": [0, 31, 60, 89],
            "columns": ["metal", "metal_store", "metal_pull", "metal_income", "metal_spend", "energy", "energy_store", "energy_pull", "energy_income", "energy_spend", "metal_excess", "metal_sent", "metal_received", "energy_excess", "energy_sent", "energy_received",
                "conv_use", "conv_capacity", "metal_reclaimed", "energy_reclaimed", "build_power", "build_power_used", "cons", "idle_cons", "labs", "idle_labs", "orders"],
            "wind": [5, 6.5, 7, 8],
            "teams": { "0": [row(1.0), row(2.0), row(5.0), row(7.0)], "1": [null, row(2.0), row(2.0), row(2.0)] },
            "stat_columns": ["metal_used", "metal_produced", "metal_excess", "energy_used", "energy_produced", "energy_excess", "damage_dealt", "damage_received", "units_produced", "units_died", "units_killed"],
            "stats": { "0": [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [45, 3, 10, 0, 0, 0, 0, 0, 0, 1, 0, 0]] },
        });
        let mut tape = Tape::default();
        (tape.head, tape.defs, tape.teams, tape.last_frame) = (head.to_string(), defs.to_string(), teams.to_string(), 300);
        tape.units = vec![unit(1, 0, 30, 90, 2, vec![]), unit(1, 1, 30, NEVER, 0, vec![]), unit(2, 0, 60, NEVER, 0, vec![(150, 1)]), unit(3, 1, NEVER, NEVER, 0, vec![])];
        tape
    }

    #[test]
    fn the_numbers_follow_the_unit_lives_and_the_table() {
        let stats = compute(&game()).unwrap();
        assert_eq!(stats.seconds, 11);
        assert_eq!(stats.teams.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["a", "cortex"], "Gaia is no team of the game; a nameless team goes by its faction");
        let of = |team: usize, key: &str| stats.series(team, metric(key).unwrap()).to_vec();
        // The tank stands finished from second 1 and is dead by second 3; the extractor from second 2, theirs from 5.
        assert_eq!(of(0, "units")[..7], [0.0, 1.0, 2.0, 1.0, 1.0, 0.0, 0.0]);
        assert_eq!(of(1, "units")[..7], [0.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0]);
        assert_eq!(of(0, "value_army")[..5], [0.0, 100.0, 100.0, 0.0, 0.0]);
        assert_eq!(of(0, "extractors")[..7], [0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0]);
        assert_eq!(of(1, "extractors")[5], 1.0);
        assert_eq!(of(0, "value_economy")[4], 50.0);
        // A unit never finished counts nowhere.
        assert_eq!(of(1, "count_builders")[10], 0.0);
        // The death is the owner's loss and its killer's side's kill, from the second after it.
        assert_eq!(of(0, "lost_value")[2..4], [0.0, 100.0]);
        assert_eq!(of(1, "killed_value")[2..4], [0.0, 100.0]);
        assert_eq!(of(0, "killed_value")[10], 0.0);
        // The table: a level or a flow stands until the next row (a row is the second its frame is nearest), what
        // was sent runs up, a team has nothing before its first row, the engine's statistics stand from their frame.
        assert_eq!(of(0, "metal_income")[..5], [1.0, 2.0, 5.0, 7.0, 7.0]);
        assert_eq!(of(0, "metal_store")[4], 500.0);
        assert_eq!(of(0, "metal_sent")[..5], [1.0, 2.0, 3.0, 4.0, 4.0]);
        assert_eq!(of(1, "metal_income")[..3], [0.0, 2.0, 2.0]);
        assert_eq!(of(0, "metal_produced")[..4], [0.0, 0.0, 10.0, 10.0]);
        assert_eq!(of(0, "units_produced")[2], 1.0);
        // The widget's sums of a second: the build power and its use, the idle, the conversion, the orders; the wind
        // is the game's; what the game's version keeps no count of is absent, its values zeros.
        assert_eq!((of(0, "build_power")[2], of(0, "build_power_used")[2], of(0, "idle_cons")[2], of(0, "conv_use")[2], of(0, "orders")[2]), (300.0, 150.0, 1.0, 3.0, 4.0));
        assert_eq!(of(1, "build_power")[..2], [0.0, 300.0]);
        assert_eq!(stats.wind[..5], [5.0, 6.5, 7.0, 8.0, 8.0]);
        assert_eq!(stats.absent, ["metal_reclaimed", "energy_reclaimed"]);
        assert_eq!(of(0, "metal_reclaimed")[3], 0.0);
        // A side is the sum of its teams, and its name its first team's.
        assert_eq!(stats.sides().len(), 2);
        assert_eq!(stats.side_name(&stats.sides()[1].1), "cortex");
    }

    #[test]
    fn a_class_is_the_games_own() {
        let kinds = types(&serde_json::from_str(&game().defs).unwrap());
        let class = |name: &str| CLASSES[kinds.iter().find(|t| t.name == name).unwrap().class];
        assert_eq!(class("tank"), "army");
        assert_eq!(class("mex"), "economy");
        // A constructor with a gun is a builder, a plant a factory though it builds.
        assert_eq!(class("con"), "builders");
        assert_eq!(class("plant"), "factories");
        // A building that shoots is a defence though it keeps energy for its gun; one without a weapon is utility.
        assert_eq!(class("gun"), "defence");
        assert_eq!(class("wall"), "utility");
        assert_eq!(class("radar"), "utility");
        // A nuke silo is strategic even afloat, and so is a long-range gun; a ship is sea, a plane air.
        assert_eq!(class("silo"), "strategic");
        assert_eq!(class("floating silo"), "strategic");
        assert_eq!(class("long gun"), "strategic");
        // (The move class as the engine hands it to a widget, lower case, and as the game's files spell it.)
        assert_eq!(class("ship"), "sea");
        assert_eq!(class("sub"), "sea");
        assert_eq!(class("flagship"), "sea");
        assert_eq!(class("hover"), "army");
        assert_eq!(class("plane"), "air");
        // What moves on the ground without a weapon is utility.
        assert_eq!(class("scout"), "utility");
    }
}
