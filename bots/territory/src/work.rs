//! The plan's debts by column (T-LAGS, step 3 of `docs/design/2026-10-25-territory-regions.md`): per column the
//! program runs over the lead, how far the hands' commitment runs behind the plan, in seconds of the program's own
//! rate on the column. The matching rule step 4 built on it was not kept (terr163-174), and the note of its pick at
//! every order went with it.

use std::collections::BTreeMap;

use buildorder::lp::{Class, Kind};
use buildorder::units::Role;

use crate::hands::LEAD;
use crate::{Territory, SECOND};

/// The `lags:` line comes every this many frames.
pub(crate) const LAG_LOG: i32 = 10 * SECOND;
/// A column is run over the lead when the program plans at least this share of a unit of it within the lead: the
/// interior point leaves a trickle on every type, and a trickle's seconds behind say nothing (the amphibious
/// constructor read 30 s behind with nothing owed at 0:20 of terr157 game 00).
const RUN_LEAST: f64 = 0.1;

/// One column of the program as the hands owe it: the program's power on it to now + LEAD over every builder type's
/// flows, its power a second over the lead, what the hands have committed to it since the plan was adopted, and a
/// unit's least cost.
pub(crate) struct Owed {
    pub unit: usize,
    pub class: Class,
    pub planned: f64,
    pub rate: f64,
    pub committed: f64,
    pub cost: f64,
}

impl Owed {
    /// Seconds of the program's rate on the column that the commitment runs behind the plan's at the clock: zero on
    /// plan, positive behind, negative ahead (the design's lag, (planned to now + LEAD less committed) / rate,
    /// less LEAD).
    pub(crate) fn behind(&self) -> f64 {
        (self.planned - self.committed) / self.rate - LEAD
    }
}

impl Territory {
    /// The program's columns run over the lead, every builder type's flows on each as T-DISPATCH counts them (the
    /// turrets' help on frames and their assists apart).
    pub(crate) fn owed(&self) -> Vec<Owed> {
        let Some(targets) = self.hands.targets.as_ref() else { return Vec::new() };
        let units = &self.game().units;
        let now = self.frame as f64 / SECOND as f64;
        let ahead = now + LEAD;
        // Per column: planned to the lead and to now, and the least cost.
        let mut per: BTreeMap<(usize, Class), [f64; 3]> = BTreeMap::new();
        for f in targets.flows.iter().filter(|f| f.kind != Kind::Reclaim && units.list[f.unit].role != Role::Factory && !(units.list[f.builder].role == Role::Nano && (f.class == Class::Frames || f.kind == Kind::Assist))) {
            let e = per.entry((f.unit, f.class)).or_insert([0.0, 0.0, f64::MAX]);
            e[0] += targets.planned_power_class(f.builder, f.unit, f.class, ahead);
            e[1] += targets.planned_power_class(f.builder, f.unit, f.class, now);
            e[2] = e[2].min(f.cost);
        }
        per.into_iter()
            .filter_map(|((unit, class), [planned, at_now, cost])| {
                let rate = (planned - at_now) / LEAD;
                if rate * LEAD < RUN_LEAST * cost {
                    return None;
                }
                let committed = if class == Class::Frames {
                    self.frames_power(unit, targets.t0, None)
                } else {
                    self.hands.committed.get(&(unit, class)).copied().unwrap_or(0.0)
                };
                Some(Owed { unit, class, planned, rate, committed, cost })
            })
            .collect()
    }

    /// The column's name for the log: the unit, with its class.
    fn item_name(&self, unit: usize, class: Class) -> String {
        let name = &self.game().units.list[unit].name;
        let class = match class {
            Class::Frames => "(frames)".to_string(),
            Class::Ring(k) if k > 0 => format!("[r{k}]"),
            _ => String::new(),
        };
        format!("{name}{class}")
    }

    /// T-LAGS every LAG_LOG: every column the program runs over the lead, most behind first, as seconds behind,
    /// units behind and the program's rate in units a minute.
    pub(crate) fn log_lags(&mut self) {
        if self.hands.targets.is_none() || self.frame < self.hands.lag_logged + LAG_LOG {
            return;
        }
        self.hands.lag_logged = self.frame;
        let mut owed = self.owed();
        owed.sort_by(|a, b| b.behind().total_cmp(&a.behind()));
        let line: Vec<String> = owed
            .iter()
            .map(|o| format!("{} {:.0} s ({:.1}, {:.1}/min)", self.item_name(o.unit, o.class), o.behind(), (o.planned - o.committed) / o.cost - o.rate * LEAD / o.cost, o.rate * 60.0 / o.cost))
            .collect();
        self.log(format_args!("lags: {}", line.join(" | ")));
    }
}
