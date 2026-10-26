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
/// A helper is on its frame within its build distance of the frame's edge, read as the frame's half-diagonal and this
/// much besides. The constructors' moving time (faster than a fifth of their top speed) was a first measure, and it
/// read 0.81-1.58 seconds of walk to a second of an extractor's work where the order-to-frame time read 1.5-2.3
/// (terr175-180, 5:00-10:00: 30-46 s from the order to the frame, then the frame's 20 s): a builder turning, slowed or
/// standing short of its site is not working either.
const ON_FRAME_SLACK: f32 = 16.0;
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
    /// Per constructor type: seconds walking under an order for no unit (the cascade's walk home).
    home: BTreeMap<usize, f64>,
    measured: Option<i32>,
    logged: i32,
}

impl Territory {
    /// Every hands tick: each constructor holding an order, walking or working, credited to its type and the order's
    /// unit type: a build walks until its frame stands and works on it after; help walks until the helper is in reach
    /// of the frame; a walk home walks for no unit. A guard on a factory is not counted, as the program prices an
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
        for walking in self.walks.home.values_mut() {
            *walking *= keep;
        }
        let units = &self.game().units;
        let mut walked: Vec<(usize, Option<usize>, bool)> = Vec::new();
        for (id, order) in &self.hands.orders {
            let Some(builder) = self.own.get(id) else { continue };
            let Some(b) = self.unit_index(builder.def) else { continue };
            if units.list[b].role != Role::Builder {
                continue;
            }
            let (unit, walking) = match &order.what {
                What::Build { unit, frame, .. } => (Some(*unit), frame.is_none()),
                What::Help { frame } => {
                    let Some(f) = self.own.get(frame) else { continue };
                    let Some(u) = self.unit_index(f.def) else { continue };
                    let (fx, fz) = self.def(f.def).map_or((0, 0), |d| d.footprint);
                    let reach = units.list[b].build_distance as f32 + 4.0 * (fx as f32).hypot(fz as f32) + ON_FRAME_SLACK;
                    (Some(u), builder.pos.dist2d(f.pos) > reach)
                }
                What::Walk { .. } => (None, true),
                What::Assist { .. } => continue,
            };
            walked.push((b, unit, walking));
        }
        for (b, unit, walking) in walked {
            match unit {
                Some(u) => {
                    let entry = self.walks.per.entry((b, u)).or_default();
                    if walking { entry.0 += dt } else { entry.1 += dt }
                }
                None => *self.walks.home.entry(b).or_default() += dt,
            }
        }
    }

    /// The measured walks as the program takes them: a constructor type's walk home spread over its pairs by their
    /// work.
    pub(crate) fn walked(&self) -> Vec<Walked> {
        let mut worked: BTreeMap<usize, f64> = BTreeMap::new();
        for (&(b, _), &(_, working)) in &self.walks.per {
            *worked.entry(b).or_default() += working;
        }
        self.walks
            .per
            .iter()
            .map(|(&(builder, unit), &(walking, working))| {
                let total = worked.get(&builder).copied().unwrap_or(0.0);
                let home = self.walks.home.get(&builder).copied().unwrap_or(0.0);
                let walking = walking + if total > 0.0 { home * working / total } else { 0.0 };
                Walked { builder, unit, walking, working }
            })
            .collect()
    }

    /// T-WALKS every WALKS_LOG: per constructor type and unit type, most work first, the ratio of walking to working
    /// measured (the walk home spread in), the seconds of work in the decayed sums, and the ratio the program's last
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
        let home: f64 = self.walks.home.values().sum::<f64>() + 0.0;
        let line: Vec<String> = pairs
            .iter()
            .map(|w| format!("{}>{} {:.2} of {:.0} s (paid {})", units.list[w.builder].name, units.list[w.unit].name, w.walking / w.working, w.working, paid(w).map_or("-".to_string(), |r| format!("{r:.2}"))))
            .collect();
        self.log(format_args!("walks: {} | walking home {home:.0} s", line.join(" | ")));
    }
}
