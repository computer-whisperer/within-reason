//! T-PROMISE, an instrument (no decision changed): what each plan expects to stand of every unit type at its start
//! and one, two and five minutes on, beside what stands in the game every STANDING_LOG. Each plan's promise of income
//! came about 0.6 true over five minutes whatever the hands and the walk prices (terr157-192,
//! `docs/design/2026-10-25-territory-walks.md`); this says which types fall short. Read by `run/territory_promise.py`.

use std::collections::BTreeMap;

use buildorder::lp::LpPlan;

use crate::{Territory, SECOND};

/// The `standing:` line every this many frames.
pub(crate) const STANDING_LOG: i32 = 30 * SECOND;
/// The seconds from a plan's snapshot its counts are logged at.
const PROMISE_AT: [f64; 4] = [0.0, 60.0, 120.0, 300.0];
/// A type the plan holds less of than this at every time logged is left out of the line.
const PROMISE_LEAST: f64 = 0.05;

impl Territory {
    /// `promise:` at each plan adopted: its snapshot's game second and, per unit type, its count at PROMISE_AT from
    /// it (the count runs with the work paid, so a frame half built is half a unit; a spot upgraded counts as a basic
    /// extractor and an upgrade).
    pub(crate) fn log_promise(&self, plan: &LpPlan) {
        let units = &self.game().units;
        let cells: Vec<String> = plan
            .types
            .iter()
            .filter_map(|&u| {
                let counts: Vec<f64> = PROMISE_AT.iter().map(|dt| plan.count_at(u, plan.t0 + dt)).collect();
                counts.iter().any(|c| *c >= PROMISE_LEAST).then(|| format!("{} {}", units.list[u].name, counts.iter().map(|c| format!("{c:.2}")).collect::<Vec<_>>().join("/")))
            })
            .collect();
        self.log(format_args!("promise: t0={:.0} | {}", plan.t0, cells.join(" | ")));
    }

    /// `standing:` every STANDING_LOG: per unit type of ours, the units finished, the frames, and the frames' progress
    /// summed (a frame's health over its full health, which grows with the work paid on it), as `name finished frames
    /// progress`.
    pub(crate) fn log_standing(&self) {
        let units = &self.game().units;
        let mut per: BTreeMap<usize, (u32, u32, f64)> = BTreeMap::new();
        for unit in self.own.values() {
            let Some(i) = self.unit_index(unit.def) else { continue };
            let entry = per.entry(i).or_default();
            if unit.being_built {
                entry.1 += 1;
                entry.2 += f64::from((unit.health / unit.max_health.max(1.0)).clamp(0.0, 1.0));
            } else {
                entry.0 += 1;
            }
        }
        let cells: Vec<String> = per.iter().map(|(i, (finished, frames, progress))| format!("{} {finished} {frames} {progress:.2}", units.list[*i].name)).collect();
        self.log(format_args!("standing: {}", cells.join(" | ")));
    }
}
