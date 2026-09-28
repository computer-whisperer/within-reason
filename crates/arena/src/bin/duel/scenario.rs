//! A scenario: a recorded engagement fought again, the units of both sides spawned where they stood.
//!
//! The file is JSON, written by `run/engagement.py` (or by hand):
//!
//! ```json
//! { "format": "within-reason-scenario", "version": 1,
//!   "map": "Comet Catcher Remake 1.8",          // the engine map's name; `duel --scenario` plays on it
//!   "map_size": [8192, 6144],                   // elmos; the commanders start in the two corners farthest away
//!   "centre": [3177, 4080],                     // where the engagement is: the field the director clears
//!   "source": { ... },                          // where it was cut from; not read
//!   "sides": [                                  // side 0, then side 1; each gets its own team
//!     { "name": "ours", "units": [
//!       { "type": "armstump", "x": 3845, "z": 4079,
//!         "health": 0.32,                       // share of full health (1 when absent), or
//!         "hp": 420,                            // absolute hit points (a record's enemies: no maximum known)
//!         "heading": [0.71, -0.71],             // which way it was moving; absent when it stood
//!         "order": { "kind": "fight", "x": 3900, "z": 3300 } }  // or "move", or {"kind":"attack","target":i}
//!     ] },                                      //   (i: a unit of the other side, by its place in the list)
//!     { "name": "theirs", "units": [ ... ],
//!       "track": [ { "t": 2.0, "unit": 1, "x": 4466, "z": 1496 } ] } ] }   // optional: where each unit was, by time
//! ```
//!
//! `track` (`run/engagement.py --track`, 2026-09-28): the recorded places of a side's units after the cut, `t` seconds
//! from the start of the fight (rows before 0 allowed). Under `duel --theirs track` the director walks side 1 along
//! it: each unit is sent (a move) to its first place later than now, whenever that place changes and is more than
//! `TRACK_STILL` from the last one it was sent to. A unit that died in the record is not killed by the clock; it
//! dies when it dies in the engine. When its rows run out it stands (and fires at will).
//!
//! A **script** (`duel --script FILE`, 2026-09-28, `docs/design/2026-09-28-tas-micro.md`) is a separate file: timed
//! orders for side 0, which replace the director's orders for that side (and its lane):
//!
//! ```json
//! { "format": "within-reason-script", "version": 1, "note": "anything",
//!   "rows": [
//!     { "t": 0.0, "unit": [0, 1, 2], "cmd": "move", "x": 4300, "z": 1900 },   // unit: an index into side 0, a list, or "all"
//!     { "t": 0.0, "unit": 3, "cmd": "attack", "target": 4 },                    // target: an index into side 1
//!     { "t": 4.5, "unit": "all", "cmd": "fight", "x": 4527, "z": 1436, "queue": true },
//!     { "t": 9.0, "unit": 5, "cmd": "guard", "target": 6 },                     // guard: an index into side 0
//!     { "t": 9.0, "unit": 5, "cmd": "stop" },
//!     { "t": 9.0, "unit": 5, "cmd": "firestate", "state": 0 } ] }           // 0 hold fire, 1 return fire, 2 fire at will
//! ```
//!
//! (A bare JSON array of rows is read too.) `t` is seconds from the fight's first orders; the director issues a row on
//! its first tick at or after `t` plus `--script-delay`. Ticks are three frames (0.1 s) apart, so a row lands up to
//! 0.1 s after its time; rows due on one tick go out in file order, in that tick's one batch of commands, and the
//! engine carries them out in that order on the same frame. A row for a dead unit, or an attack or guard on a dead
//! target, is skipped (the per-second log says so). Before its first row a unit has only the heading step (below);
//! after its last order is done it stands and fires at will.
//!
//! How it is spawned (`director.rs`, `Phase::Preparing`), since the AI interface's cheat gives a unit at a place and
//! nothing else (`COMMAND_CHEATS_GIVE_ME_NEW_UNIT`: no health, no facing; no Lua gadget of the game answers an AI's
//! message either):
//! 1. **Health by damage after spawning.** The units to be hurt are given first, both sides, each told to hold fire
//!    as soon as it is seen (a tick, three frames, after it appears). For each, the other side is given a Pawn
//!    (`armpw`, a small precise gun) 90 elmos from it on the side away from the centre, told to attack it and stopped
//!    when its health reaches the file's: the Pawn's last shot overshoots by at most one shot (about 1% of a Stout).
//!    A unit still above its health after 60 s is left there and counted. Then each Pawn walks off 600 and
//!    destroys itself; its wreck lies out there.
//! 2. **Then everyone else** is given at full health, and when all are there both sides are set to fire at will and
//!    get their first orders: the file's `order` (fight, move, or attack a unit of the other side), else a fight
//!    at the other side's centre, as in a duel.
//! 3. **Facing by a short move.** A unit with a `heading` is first sent 24 elmos along it and its order is queued
//!    behind: it ends up pointing the way it went, a fraction of a second late.
//!
//! Scoring, clearing and the fire instrument are the duel's: side 0 is `x`, side 1 is `y`, and `value_left` is
//! metal-weighted over the side's units, each weighted by its health at the end against its health at the start.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Scenario {
    pub format: String,
    pub version: u32,
    pub map: String,
    /// The map's width and height in elmos: where the commanders are put out of the way.
    pub map_size: [f32; 2],
    pub centre: [f32; 2],
    pub sides: [Side; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct Side {
    pub name: String,
    pub units: Vec<Unit>,
    /// Where each unit was after the cut (`run/engagement.py --track`); walked under `duel --theirs track`.
    #[serde(default)]
    pub track: Vec<TrackRow>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct TrackRow {
    pub t: f32,
    pub unit: usize,
    pub x: f32,
    pub z: f32,
}

/// A track's next place is not sent while it is this close to the last one sent: a unit standing in the record stands.
pub const TRACK_STILL: f32 = 16.0;

/// Timed orders for side 0 (`duel --script`).
#[derive(Clone, Debug)]
pub struct Script {
    /// The file's name without its extension, for `duels.csv`.
    pub label: String,
    /// Sorted by time; rows of one time keep the file's order.
    pub rows: Vec<ScriptRow>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScriptRow {
    pub t: f32,
    pub unit: Units,
    pub cmd: ScriptCmd,
    pub x: Option<f32>,
    pub z: Option<f32>,
    pub target: Option<usize>,
    #[serde(default)]
    pub queue: bool,
    pub state: Option<i32>,
}

/// Which units of side 0 a row orders.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Units {
    One(usize),
    Many(Vec<usize>),
    All(String),
}

impl Units {
    pub fn indices(&self, count: usize) -> Vec<usize> {
        match self {
            Units::One(i) => vec![*i],
            Units::Many(list) => list.clone(),
            Units::All(_) => (0..count).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptCmd {
    Move,
    Fight,
    Attack,
    Stop,
    Guard,
    Firestate,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ScriptFile {
    Rows(Vec<ScriptRow>),
    Wrapped { rows: Vec<ScriptRow> },
}

impl Script {
    /// Reads and checks a script against the scenario it will be fought in.
    pub fn load(path: &std::path::Path, scenario: &Scenario) -> Result<Script, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file: ScriptFile = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut rows = match file {
            ScriptFile::Rows(rows) | ScriptFile::Wrapped { rows } => rows,
        };
        let (ours, theirs) = (scenario.sides[0].units.len(), scenario.sides[1].units.len());
        for (n, row) in rows.iter().enumerate() {
            let problem = |what: &str| Err(format!("{}: row {n}: {what}", path.display()));
            if let Units::All(word) = &row.unit
                && word != "all"
            {
                return problem("unit is an index, a list of indices, or \"all\"");
            }
            if row.unit.indices(ours).iter().any(|&i| i >= ours) {
                return problem(&format!("a unit index past side 0's {ours} units"));
            }
            match row.cmd {
                ScriptCmd::Move | ScriptCmd::Fight if row.x.is_none() || row.z.is_none() => return problem("move and fight need x and z"),
                ScriptCmd::Attack if row.target.is_none_or(|t| t >= theirs) => return problem(&format!("attack needs a target below {theirs}")),
                ScriptCmd::Guard if row.target.is_none_or(|t| t >= ours) => return problem(&format!("guard needs a target (of side 0) below {ours}")),
                ScriptCmd::Firestate if !row.state.is_some_and(|s| (0..=2).contains(&s)) => return problem("firestate needs a state 0, 1 or 2"),
                _ => {}
            }
        }
        rows.sort_by(|a, b| a.t.total_cmp(&b.t));
        let label = path.file_stem().map_or_else(|| "script".to_string(), |s| s.to_string_lossy().replace(',', "_"));
        Ok(Script { label, rows })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Unit {
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f32,
    pub z: f32,
    pub health: Option<f32>,
    pub hp: Option<f32>,
    pub heading: Option<[f32; 2]>,
    pub order: Option<Order>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Order {
    Fight { x: f32, z: f32 },
    Move { x: f32, z: f32 },
    Attack { target: usize },
}

impl Scenario {
    pub fn load(path: &std::path::Path) -> Result<Scenario, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let scenario: Scenario = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if scenario.format != "within-reason-scenario" || scenario.version != 1 {
            return Err(format!("{}: not a version 1 within-reason-scenario", path.display()));
        }
        for (s, side) in scenario.sides.iter().enumerate() {
            if side.units.is_empty() {
                return Err(format!("{}: side {s} ({}) has no units", path.display(), side.name));
            }
            for unit in &side.units {
                if let Some(Order::Attack { target }) = unit.order
                    && target >= scenario.sides[1 - s].units.len()
                {
                    return Err(format!("{}: an attack order names unit {target} of the other side, which has fewer", path.display()));
                }
            }
        }
        Ok(scenario)
    }
}

impl Unit {
    /// The health it should start with: a share of full health, or absolute hit points against `max_health`.
    pub fn wanted_health(&self, max_health: f32) -> f32 {
        match (self.health, self.hp) {
            (Some(share), _) => share.clamp(0.01, 1.0),
            (None, Some(hp)) => (hp / max_health.max(1.0)).clamp(0.01, 1.0),
            (None, None) => 1.0,
        }
    }

    /// Whether it must be hurt before the fight: anything under 99.5% (a hair of damage is not worth a Pawn).
    pub fn needs_hurting(&self, max_health: Option<f32>) -> bool {
        match (self.health, self.hp, max_health) {
            (Some(share), _, _) => share < 0.995,
            (None, Some(hp), Some(max)) => hp < 0.995 * max,
            // Absolute hit points and the maximum not yet known: hurt it, and the check against the spawned unit
            // decides how far.
            (None, Some(_), None) => true,
            _ => false,
        }
    }
}
