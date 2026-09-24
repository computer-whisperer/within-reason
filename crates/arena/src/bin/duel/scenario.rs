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
//!     { "name": "theirs", "units": [ ... ] } ] }
//! ```
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
