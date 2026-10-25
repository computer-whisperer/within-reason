//! The plan's work as items with a place (`docs/design/2026-10-25-territory-regions.md`, part 4). Per column the
//! program runs over the lead: how far the hands' commitment runs behind the plan, in seconds of the program's own
//! rate on the column; and for an idle builder, where it would work each item and the walk there. Step 3 logs them
//! beside the cascade's choice (T-LAGS); the hands do not dispatch by them yet.

use std::collections::BTreeMap;

use bot_protocol::{OwnUnit, UnitId, Vec3};
use buildorder::lp::{Class, Kind};
use buildorder::units::Role;

use crate::hands::{FARM_BIG, LEAD, START_AT};
use crate::{Territory, SECOND};

/// The `lags:` line comes every this many frames.
pub(crate) const LAG_LOG: i32 = 10 * SECOND;
/// A column is run over the lead when the program plans at least this share of a unit of it within the lead: the
/// interior point leaves a trickle on every type, and a trickle's seconds behind say nothing (the amphibious
/// constructor read 30 s behind with nothing owed at 0:20 of terr157 game 00).
const RUN_LEAST: f64 = 0.1;

/// One column of the program as the hands owe it: the program's power on it to now + LEAD over every builder type's
/// flows (or a helper type's own, on a type it cannot start), its power a second over the lead, what the hands have
/// committed to it since the plan was adopted, and a unit's least cost.
pub(crate) struct Owed {
    pub unit: usize,
    pub class: Class,
    pub planned: f64,
    pub rate: f64,
    pub committed: f64,
    pub cost: f64,
    /// The helper's own help on a type it cannot start (the program's `mobile_help`).
    pub help: bool,
}

impl Owed {
    /// Seconds of the program's rate on the column that the commitment runs behind the plan's at the clock: zero on
    /// plan, positive behind, negative ahead (the design's lag, (planned to now + LEAD less committed) / rate,
    /// less LEAD).
    pub(crate) fn behind(&self) -> f64 {
        (self.planned - self.committed) / self.rate - LEAD
    }

    /// Enough of it to start: half a unit's cost for a whole unit, any for help on frames (T-DISPATCH's START_AT).
    pub(crate) fn open(&self, kind: Kind) -> bool {
        let credit = self.planned - self.committed;
        if self.class == Class::Frames || kind == Kind::Help { credit > 0.0 } else { credit >= START_AT * self.cost }
    }
}

/// An item of work for one builder: the column, how the builder serves it, where, and its walk there in seconds.
pub(crate) struct WorkItem {
    pub owed: Owed,
    pub kind: Kind,
    pub at: Option<Vec3>,
    pub walk: f64,
}

impl Territory {
    /// The program's columns run over the lead, every builder type's flows on each as T-DISPATCH counts them (the
    /// turrets' help on frames and their assists apart); with `helper`, that type's own help on the columns it
    /// helps and cannot start.
    pub(crate) fn owed(&self, helper: Option<usize>) -> Vec<Owed> {
        let Some(targets) = self.hands.targets.as_ref() else { return Vec::new() };
        let units = &self.game().units;
        let now = self.frame as f64 / SECOND as f64;
        let ahead = now + LEAD;
        // Per column: planned to the lead and to now, the least cost, and the helper's own to the lead and to now.
        let mut per: BTreeMap<(usize, Class), [f64; 5]> = BTreeMap::new();
        for f in targets.flows.iter().filter(|f| f.kind != Kind::Reclaim && units.list[f.unit].role != Role::Factory && !(units.list[f.builder].role == Role::Nano && (f.class == Class::Frames || f.kind == Kind::Assist))) {
            let e = per.entry((f.unit, f.class)).or_insert([0.0, 0.0, f64::MAX, 0.0, 0.0]);
            let (to_lead, to_now) = (targets.planned_power_class(f.builder, f.unit, f.class, ahead), targets.planned_power_class(f.builder, f.unit, f.class, now));
            e[0] += to_lead;
            e[1] += to_now;
            e[2] = e[2].min(f.cost);
            if f.kind == Kind::Help && Some(f.builder) == helper {
                e[3] += to_lead;
                e[4] += to_now;
            }
        }
        per.into_iter()
            .filter_map(|((unit, class), [to_lead, to_now, cost, own_lead, own_now])| {
                let help = own_lead > 0.0;
                let (planned, at_now) = if help { (own_lead, own_now) } else { (to_lead, to_now) };
                let rate = (planned - at_now) / LEAD;
                if rate * LEAD < RUN_LEAST * cost {
                    return None;
                }
                let committed = if help {
                    self.frames_power(unit, targets.t0, helper)
                } else if class == Class::Frames {
                    self.frames_power(unit, targets.t0, None)
                } else {
                    self.hands.committed.get(&(unit, class)).copied().unwrap_or(0.0)
                };
                Some(Owed { unit, class, planned, rate, committed, cost, help })
            })
            .collect()
    }

    /// The items an idle builder could take: per column it can build, help or guard for, where it would work it (an
    /// extractor's free spot nearest it, an upgrade's basic extractor, the frame that finishes soonest with it on
    /// it, the factory making a product; a turret or a big building at the farm, a cheap one beside the builder, as
    /// `farm_site` places them) and its walk there at its speed.
    pub(crate) fn work_items(&self, id: UnitId, unit: usize, role: Role, builder: &OwnUnit) -> Vec<WorkItem> {
        let units = &self.game().units;
        let def = &units.list[unit];
        let speed = def.speed.max(1.0);
        let leash = (role == Role::Commander).then_some((self.home, self.options.com_leash));
        self.owed(Some(unit))
            .into_iter()
            .filter_map(|owed| {
                let u = owed.unit;
                let building = units.list[u].speed == 0.0;
                let kind = if (owed.class == Class::Frames || owed.help) && building {
                    Kind::Help
                } else if def.builds.contains(&u) {
                    Kind::Direct
                } else if !building && self.factory_making(u, builder.pos).is_some() {
                    Kind::Assist
                } else {
                    return None;
                };
                let ring = match owed.class {
                    Class::Ring(k) => Some(k),
                    _ => None,
                };
                let at: Option<Vec3> = match kind {
                    Kind::Help => self.frame_to_finish(u, unit, builder.pos).and_then(|f| self.own.get(&f)).map(|f| f.pos),
                    Kind::Assist => self.factory_making(u, builder.pos).and_then(|f| self.own.get(&f)).map(|f| f.pos),
                    _ if self.is_upgrade(u) => self.basic_to_upgrade(builder.pos, id, role, ring).map(|b| b.pos),
                    _ if units.list[u].extracts_metal > 0.0 => self.free_spot(builder.pos, None, leash, ring).map(|i| self.hello.metal_spots[i]),
                    _ if units.list[u].role == Role::Nano || units.list[u].build_time >= FARM_BIG => Some(self.hands.farm_origin.unwrap_or(self.home)),
                    _ => Some(builder.pos),
                };
                let walk = at.map_or(f64::INFINITY, |p| f64::from(builder.pos.dist2d(p)) / speed);
                Some(WorkItem { owed, kind, at, walk })
            })
            .collect()
    }

    /// The column's name for the log: the unit, with its class and how this builder serves it.
    fn item_name(&self, unit: usize, class: Class, kind: Option<Kind>) -> String {
        let name = &self.game().units.list[unit].name;
        let class = match class {
            Class::Frames => "(frames)".to_string(),
            Class::Ring(k) if k > 0 => format!("[r{k}]"),
            _ => String::new(),
        };
        let how = match kind {
            Some(Kind::Help) => "help:",
            Some(Kind::Assist) => "guard:",
            _ => "",
        };
        format!("{how}{name}{class}")
    }

    /// T-LAGS at a dispatch: what the matching rule would give this builder (the open item with the most seconds
    /// behind less its walk) and the next three, held for the order line the cascade writes (`note_order`).
    pub(crate) fn note_lags(&self, id: UnitId, unit: usize, role: Role, builder: &OwnUnit) {
        let mut items: Vec<(f64, String)> = self
            .work_items(id, unit, role, builder)
            .into_iter()
            .filter(|w| w.at.is_some() && w.owed.open(w.kind))
            .map(|w| (w.owed.behind() - w.walk, format!("{} {:.0}-{:.0}", self.item_name(w.owed.unit, w.owed.class, Some(w.kind)), w.owed.behind(), w.walk)))
            .collect();
        items.sort_by(|a, b| b.0.total_cmp(&a.0));
        let note = if items.is_empty() { "nothing open".to_string() } else { items.iter().take(4).map(|(_, s)| s.as_str()).collect::<Vec<_>>().join(" | ") };
        *self.hands.lag_note.borrow_mut() = Some((id, self.frame, note));
    }

    /// T-LAGS every LAG_LOG: every column the program runs over the lead, most behind first, as seconds behind,
    /// units behind and the program's rate in units a minute.
    pub(crate) fn log_lags(&mut self) {
        if self.hands.targets.is_none() || self.frame < self.hands.lag_logged + LAG_LOG {
            return;
        }
        self.hands.lag_logged = self.frame;
        let mut owed = self.owed(None);
        owed.sort_by(|a, b| b.behind().total_cmp(&a.behind()));
        let line: Vec<String> = owed
            .iter()
            .map(|o| format!("{} {:.0} s ({:.1}, {:.1}/min)", self.item_name(o.unit, o.class, None), o.behind(), (o.planned - o.committed) / o.cost - o.rate * LEAD / o.cost, o.rate * 60.0 / o.cost))
            .collect();
        self.log(format_args!("lags: {}", line.join(" | ")));
    }
}
