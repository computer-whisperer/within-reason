//! The walks measured in the game (`docs/design/2026-10-25-territory-walks.md`, change 1; T-WALKS): per constructor
//! type and unit type, the seconds its constructors walked to the work and worked on it, decayed over WALK_MEMORY.
//! The program prices each pair from them (`LpOptions::walked`) where it priced every walk of a kind at one constant:
//! a basic constructor's help on the upgrade frames at a tenth of a second of walk a second of help, where the hands
//! walked two, and every plan's promise of the next two minutes came about half true (terr157-162).

use std::collections::BTreeMap;

use buildorder::lp::{Class, Walked};
use buildorder::units::Role;

use crate::hands::What;
use crate::{Territory, MINUTE, SECOND};

/// The sums decay with this time constant, in seconds: the extractors' walk grows as the near spots go (a median 12-20
/// s an extractor to 5:00, 23-74 s from 5:00 to 10:00, terr157-162).
const WALK_MEMORY: f64 = 120.0;
/// A constructor moving faster than this share of its top speed is walking; a builder stands while it builds.
const MOVING: f64 = 0.2;
/// The constant walk counts as this many seconds of work beside the measured (`LpOptions::walk_prior`).
pub(crate) const WALK_PRIOR: f64 = 30.0;
/// The `walks:` line every this many frames.
const WALKS_LOG: i32 = MINUTE;
/// A pair with less work than this in its decayed sums is left out of the line.
const WALKS_SHOWN: f64 = 5.0;

#[derive(Default)]
pub(crate) struct Walks {
    /// Per (constructor type, unit type): seconds walking, seconds working.
    per: BTreeMap<(usize, usize), (f64, f64)>,
    measured: Option<i32>,
    logged: i32,
}

impl Territory {
    /// Every hands tick: each constructor holding an order, walking or working, credited to its type and the order's
    /// unit type (a build's type, a helped frame's type); a guard on a factory is not counted, as the program prices an
    /// assist with no walk.
    pub(crate) fn measure_walks(&mut self) {
        let last = self.walks.measured.replace(self.frame);
        let Some(last) = last else { return };
        let dt = f64::from(self.frame - last) / f64::from(SECOND);
        if dt <= 0.0 {
            return;
        }
        let keep = (-dt / WALK_MEMORY).exp();
        for (walking, working) in self.walks.per.values_mut() {
            *walking *= keep;
            *working *= keep;
        }
        let units = &self.game().units;
        let mut walked: Vec<(usize, usize, bool)> = Vec::new();
        for (id, order) in &self.hands.orders {
            let Some(builder) = self.own.get(id) else { continue };
            let Some(b) = self.unit_index(builder.def) else { continue };
            if units.list[b].role != Role::Builder {
                continue;
            }
            let moving = f64::from(builder.vel.x.hypot(builder.vel.z)) * f64::from(SECOND) > MOVING * units.list[b].speed;
            let unit = match &order.what {
                What::Build { unit, .. } => *unit,
                What::Help { frame } => match self.own.get(frame).and_then(|f| self.unit_index(f.def)) {
                    Some(u) => u,
                    None => continue,
                },
                What::Assist { .. } => continue,
            };
            walked.push((b, unit, moving));
        }
        for (b, u, moving) in walked {
            let entry = self.walks.per.entry((b, u)).or_default();
            if moving { entry.0 += dt } else { entry.1 += dt }
        }
    }

    /// The measured walks as the program takes them.
    pub(crate) fn walked(&self) -> Vec<Walked> {
        self.walks.per.iter().map(|(&(builder, unit), &(walking, working))| Walked { builder, unit, walking, working }).collect()
    }

    /// T-WALKS every WALKS_LOG: per constructor type and unit type, most work first, the ratio of walking to working
    /// measured, the seconds of work in the decayed sums, and the ratio the program's last
    /// plan priced the pair at (its cost over the unit's build time, less one).
    pub(crate) fn log_walks(&mut self) {
        if self.frame < self.walks.logged + WALKS_LOG {
            return;
        }
        self.walks.logged = self.frame;
        let units = &self.game().units;
        let mut pairs = self.walked();
        pairs.retain(|w| w.working >= WALKS_SHOWN);
        pairs.sort_by(|a, b| b.working.total_cmp(&a.working));
        let paid = |w: &Walked| -> Option<f64> {
            let targets = self.hands.targets.as_ref()?;
            let flow = targets.flows.iter().find(|f| f.builder == w.builder && f.unit == w.unit && f.class != Class::Frames)?;
            Some(flow.cost / units.list[w.unit].build_time.max(1.0) - 1.0)
        };
        let line: Vec<String> = pairs
            .iter()
            .map(|w| format!("{}>{} {:.2} of {:.0} s (paid {})", units.list[w.builder].name, units.list[w.unit].name, w.walking / w.working, w.working, paid(w).map_or("-".to_string(), |r| format!("{r:.2}"))))
            .collect();
        self.log(format_args!("walks: {}", line.join(" | ")));
    }
}
