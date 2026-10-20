//! The hands: the plan in force carried out one order per builder, placed as the simulator places things
//! (docs/design/2026-10-20-territory-bot.md, decision 5; the README's T-ORDERS, T-SITES).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bot_protocol::{BuildSite, Command, Lane, OwnUnit, Placement, UnitDefId, UnitId, Vec3};
use buildorder::lp::{Kind, LpPlan};
use buildorder::plan::{Item, Plan, Step};
use buildorder::units::Role;

use crate::{Territory, MINUTE, SECOND};

/// Frames a unit is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const GRACE: i32 = 45;
/// T-ORDERS: a build order with no frame begun after this long is held for nothing and dropped (the engine starts no
/// frame while it reclaims a wreck on the site or waits for a blocker, and drops a blocked build itself after thirty
/// tries; the distilled bot's D-ORDERS found a shorter watchdog took orders from builders in range of their spots).
const HELD: i32 = 2 * MINUTE;
/// ... counted from the first frame the builder stands this near the site (terr14-lp-nullai-comet-1: counted from
/// the order, every walk past two minutes was dropped as the builder came within 300 of a far spot, 130 extractor
/// orders in four games, the begun ages a wall at 120 s).
const HELD_NEAR: f32 = 300.0;
/// A step that came to nothing is given once more, at another site, then skipped.
const ATTEMPTS: u32 = 2;
/// The simulator's `assist_chunk`.
const ASSIST_CHUNK: i32 = 20 * SECOND;
/// The simulator's `base_radius`: a builder this near home puts an economy building beside itself.
const BESIDE: f32 = 600.0;
/// The engine drops orders given in its first seconds; what comes to nothing before this counts no attempt.
const FIRST_ORDERS: i32 = 3 * SECOND;
/// A construction turret helps the factory within this of it.
const NANO_REACH: f32 = 330.0;
/// A construction turret is asked for this far behind its factory: outside the yard's keep-out, inside its reach.
const NANO_BEHIND: f32 = 250.0;
/// Nothing is placed within this of a factory's centre: a site tested against an open yard passes and the engine
/// then drops the order (terr15-lp-nullai-comet-1: seven second labs and 48 construction turrets dropped against
/// the first lab, its yard open round a constructor's frame).
const FACTORY_YARD: f32 = 200.0;
/// A building site placed but not yet begun keeps other sites this far off: the host's search knows nothing of
/// sites chosen seconds before (terr16-lp-nullai-comet-1: 72 economy orders in four games dropped by the engine
/// 5-30 s after the order with the builder idle a few hundred elmos from where it was ordered, forty constructors
/// placing solars beside themselves round one base).
const PENDING_SITE: f32 = 70.0;
/// The commander takes extractor spots and upgrades within this of home only (terr15-lp-nullai-comet-1: following
/// its extractor flows spot by spot it crossed the map, set up a base in the far corner, and died to his commander
/// 7,300 elmos from home, two games of four lost).
const COMMANDER_LEASH: f32 = 1500.0;
/// A spot with an extractor, a frame or an order of ours this near is taken.
const SPOT_TAKEN: f32 = 100.0;
/// A refused spot is left alone this long.
const SPOT_REFUSED: i32 = MINUTE;
/// A unit type the host found no site for is left alone this long by every builder (terr18-lp-nullai-comet-1: the
/// geothermal plant, with no vent on the map, asked for by every constructor every half second, 5,500 drops a game
/// and nothing else built for ten minutes).
const TYPE_REFUSED: i32 = MINUTE;
/// The energy class fires only while the energy stock is under this share of its storage: with the stock at the
/// cap the program's planned energy is for spending that is not happening, and solars ahead of the flows left the
/// metal at zero from 2:00 to 8:00 (terr18-lp-nullai-comet-1).
const ENERGY_LOW: f64 = 0.5;
/// ... and only while fewer than this many energy orders stand without a frame.
const ENERGY_PENDING: usize = 3;
/// T-DISPATCH: the program's targets are read this far ahead of the clock, so a builder starts what the plan wants
/// next rather than what it wanted at the last whole bucket. Not shorter: the program's counts ramp from the
/// snapshot, and every re-solve restarts the ramp, so a target read 20 s ahead never crossed its threshold for
/// anything the program wanted slowly (terr6-lp-nullai-comet-1: no factory in four games, the commander alone).
const LEAD: f64 = 45.0;
/// T-DISPATCH: a class of lumpy things (the factories as a whole; a factory's products of one role) starts a unit
/// when the program's count of the class runs this far ahead of what stands, is framed or is ordered.
const START_AT: f64 = 0.5;
/// What stands within this of a dropped order's site is named in its log line (the held extractor orders).
const ON_SITE: f32 = 80.0;
/// A unit starts only when the metal stock and this many seconds of income cover its metal cost: the program builds
/// fractions of expensive things from the first minute (a tenth of a fusion in the ceiling's second minute, half by
/// 5:00; terr17-lp-nullai-comet-1 committed the advanced plant whole at 3:55 on an income of ten, and the early game
/// starved: income 7-23 at 5:00 against the ceiling's 51). Ninety seconds, not forty-five: at forty-five the plant
/// waited to 9:04-10:27 (terr20) where the ceiling takes its first third of one at 7:00.
pub(crate) const AFFORD_SECONDS: f64 = 90.0;
/// T-COMMIT: a factory type the program wants at all within this long, whose products include one that nothing
/// standing, framed or ordered makes, is started whole: the program builds fractions of a factory for minutes (0.3
/// of a lab in the first minute of the ceiling, 0.3 of an advanced plant from 7:00 for every moho it makes), and a
/// fraction of a factory makes nothing in the engine.
const COMMIT_WITHIN: f64 = 180.0;
const COMMIT_AT: f64 = 0.05;
/// T-STRUCK: a unit type whose orders have come to nothing this many times, more often than its frames have begun,
/// is struck from the palette for the rest of the game, and its remaining steps are passed over (terr1-nullai-comet-1:
/// every wind generator order on Comet was dropped by the engine, 9-21 a game, and every Cortex tier-2 extractor,
/// 94 in one game; the search kept writing them).
const STRIKE: u32 = 4;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum What {
    /// `unit` indexes the simulator's unit table; `near` is where the site was asked for; `frame` the building
    /// begun, once the engine names it.
    Build { unit: usize, def: UnitDefId, near: Vec3, frame: Option<UnitId> },
    Assist { factory: UnitId, until: i32 },
    Help { frame: UnitId },
}

#[derive(Clone, Debug)]
pub(crate) struct Order {
    pub what: What,
    pub given: i32,
    /// The last frame the engine was told (the first order, or a repair re-issued to an idle builder).
    pub told: i32,
    /// The first frame the builder stood within `HELD_NEAR` of a build site (the held clock starts there).
    pub arrived: Option<i32>,
    /// The step of the builder's queue it carries out, so a failed one can be given again; none once the plan has
    /// changed under it.
    pub step: Option<usize>,
}

enum Review {
    Keep,
    Done,
    Failed(&'static str),
}

/// A queue index meaning "no queue in the plan": a constructor past the plan's queues takes the nearest free spot.
const NO_QUEUE: usize = usize::MAX;

pub(crate) struct Hands {
    pub plan: Plan,
    /// The program's targets (T-DISPATCH), when the planner is the program.
    pub targets: Option<LpPlan>,
    /// Build-power-seconds committed per (builder type, unit type) since the targets were adopted: an order's cost
    /// to its builder's type, a guard's chunk of its power.
    pub committed: BTreeMap<(usize, usize), f64>,
    pub orders: BTreeMap<UnitId, Order>,
    /// Per builder: its queue in the plan and the index of its next step.
    pub queues: BTreeMap<UnitId, (usize, usize)>,
    /// The plan's queues for factories and constructors that did not stand when it was made, in order.
    pub spare_factory_queues: VecDeque<usize>,
    pub spare_constructor_queues: VecDeque<usize>,
    pub attempts: BTreeMap<UnitId, u32>,
    /// Spots (indices into the game's list) refused by the host or the engine, until this frame.
    pub refused: BTreeMap<usize, i32>,
    /// Unit types the host found no site for, until this frame.
    pub type_refused: BTreeMap<usize, i32>,
    pub nano_told: BTreeMap<UnitId, i32>,
    /// Per builder, the frames it has begun (frame, unit): what a plan searched from an older snapshot has to skip.
    pub begun_log: BTreeMap<UnitId, Vec<(i32, usize)>>,
    /// Per unit type: orders that came to nothing, and frames begun (T-STRUCK).
    pub failures: BTreeMap<usize, u32>,
    pub successes: BTreeMap<usize, u32>,
    pub struck: BTreeSet<usize>,
    /// Base buildings placed on the spiral so far (the simulator's `base_sites`).
    pub base_sites: usize,
    pub dropped: usize,
    pub begun: usize,
    pub skipped: usize,
}

impl Default for Hands {
    fn default() -> Self {
        Hands {
            plan: Plan::empty(0, 0),
            targets: None,
            committed: BTreeMap::new(),
            orders: BTreeMap::new(),
            queues: BTreeMap::new(),
            spare_factory_queues: VecDeque::new(),
            spare_constructor_queues: VecDeque::new(),
            attempts: BTreeMap::new(),
            refused: BTreeMap::new(),
            type_refused: BTreeMap::new(),
            nano_told: BTreeMap::new(),
            begun_log: BTreeMap::new(),
            failures: BTreeMap::new(),
            successes: BTreeMap::new(),
            struck: BTreeSet::new(),
            base_sites: 0,
            dropped: 0,
            begun: 0,
            skipped: 0,
        }
    }
}

/// The simulator's `base_site`: a golden-angle spiral around home.
pub(crate) fn base_site(home: Vec3, n: usize) -> Vec3 {
    let radius = 120.0 + 28.0 * n as f32;
    let angle = 2.4 * n as f32;
    Vec3 { x: home.x + radius * angle.cos(), y: home.y, z: home.z + radius * angle.sin() }
}

impl Territory {
    /// A plan found from a snapshot taken at `snapshot` whose builders stood in `builders`' order replaces the plan
    /// in force (docs/design/2026-10-20-territory-bot.md, decision 1): each standing builder takes the queue of its
    /// rank, the queues of builders not yet standing wait for them, and a builder's leading steps are skipped as far
    /// as they are the frames it has begun since the snapshot and then the order it holds (the re-basing: a plan
    /// arrives 20-35 game-seconds after its snapshot at speed 50, a step or two of a constructor's queue).
    pub(crate) fn adopt(&mut self, plan: Plan, builders: &[UnitId], snapshot: i32) {
        let role_of = |me: &Self, id: UnitId| me.own.get(&id).and_then(|u| me.role_of_def(u.def));
        let (mut factories, mut constructors) = (0, 0);
        let mut queues = BTreeMap::new();
        for id in builders {
            let queue = match role_of(self, *id) {
                Some(Role::Commander) => 0,
                Some(Role::Factory) => {
                    factories += 1;
                    plan.factory_queue(factories - 1)
                }
                Some(Role::Builder) => {
                    constructors += 1;
                    plan.constructor_queue(constructors - 1)
                }
                _ => continue,
            };
            let steps = plan.queue(queue);
            let mut next = 0;
            for (_, unit) in self.hands.begun_log.get(id).map_or(&[][..], Vec::as_slice).iter().filter(|(frame, _)| *frame > snapshot) {
                match steps.get(next) {
                    Some(Step { item: Item::Build(u), .. }) if u == unit => next += 1,
                    _ => break,
                }
            }
            if let Some(order) = self.hands.orders.get_mut(id) {
                let begun = matches!(&order.what, What::Build { frame: Some(_), .. });
                let same = match (&order.what, steps.get(next).map(|s| s.item)) {
                    // A frame begun since the snapshot is in the log above; one begun before it was the snapshot's job.
                    (What::Build { unit, .. }, Some(Item::Build(u))) => *unit == u && !begun,
                    (What::Assist { .. }, Some(Item::Assist)) | (What::Help { .. }, Some(Item::Help)) => true,
                    _ => false,
                };
                order.step = if same { Some(next) } else { None };
                next += usize::from(same);
            }
            queues.insert(*id, (queue, next));
        }
        self.hands.spare_factory_queues = (factories..plan.factories.len()).map(|nth| plan.factory_queue(nth)).collect();
        self.hands.spare_constructor_queues = (constructors..plan.constructors.len()).map(|nth| plan.constructor_queue(nth)).collect();
        // A builder finished since the snapshot keeps no queue of the old plan; it takes a spare one when it is next
        // looked at.
        for (id, order) in self.hands.orders.iter_mut() {
            if !queues.contains_key(id) {
                order.step = None;
            }
        }
        self.hands.queues = queues;
        self.hands.plan = plan;
        for log in self.hands.begun_log.values_mut() {
            log.retain(|(frame, _)| *frame > snapshot);
        }
    }

    /// Every half second: each builder's order reviewed, and the next step given to one that has none.
    pub(crate) fn run_hands(&mut self, commands: &mut Vec<Command>) {
        let mut builders: Vec<(UnitId, Role)> = self
            .own
            .values()
            .filter(|u| !u.being_built)
            .filter_map(|u| self.role_of_def(u.def).filter(|r| matches!(r, Role::Commander | Role::Factory | Role::Builder)).map(|r| (u.id, r)))
            .collect();
        builders.sort_by_key(|(id, _)| id.0);
        for (id, role) in builders {
            // The queues are the anneal's; the program's targets need none (terr7-lp-nullai-comet-1: a factory
            // with no spare queue was passed over before the dispatch, and no game made a constructor).
            if self.hands.targets.is_none() && !self.hands.queues.contains_key(&id) {
                let spare = match role {
                    Role::Commander => Some(0),
                    Role::Factory => self.hands.spare_factory_queues.pop_front(),
                    _ => self.hands.spare_constructor_queues.pop_front(),
                };
                match (spare, role) {
                    (Some(queue), _) => self.hands.queues.insert(id, (queue, 0)),
                    (None, Role::Builder) => self.hands.queues.insert(id, (NO_QUEUE, 0)),
                    (None, _) => continue,
                };
            }
            if let Some(order) = self.hands.orders.get(&id).cloned() {
                match self.review(id, &order, commands) {
                    Review::Keep => continue,
                    Review::Done => {
                        self.hands.orders.remove(&id);
                    }
                    Review::Failed(why) => self.failed(id, why),
                }
            }
            if self.hands.targets.is_some() {
                self.dispatch(id, role, commands);
            } else {
                self.give_next(id, role, commands);
            }
        }
    }

    /// The committed power when targets are adopted: what each builder's standing order still costs its type (the
    /// whole cost with no frame yet, the unbuilt share with one), since the program's new flows carry that remainder
    /// as work to do and would otherwise buy it twice.
    pub(crate) fn seed_committed(&mut self) {
        let mut committed: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        let Some(targets) = self.hands.targets.as_ref() else { return };
        for (id, order) in &self.hands.orders {
            let Some(builder) = self.own.get(id).and_then(|u| self.unit_index(u.def)) else { continue };
            let What::Build { unit, frame, .. } = &order.what else { continue };
            let left = frame.and_then(|f| self.own.get(&f)).map_or(1.0, |f| 1.0 - (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
            let cost = targets.flows.iter().find(|f| f.builder == builder && f.unit == *unit).map_or(self.game().units.list[*unit].build_time, |f| f.cost);
            *committed.entry((builder, *unit)).or_default() += cost * left;
        }
        self.hands.committed = committed;
    }

    /// T-DISPATCH: the idle builder follows the program. The lumpy things go by class: the factories as a whole,
    /// and a factory's products by role, start a unit when the program's count of the class runs half a unit ahead
    /// of what stands, is framed or is ordered, and the type started is the one of the class the program wants
    /// most. Read by type nothing ever crossed a threshold: the program is indifferent between factory types and
    /// spreads a plant over all of them, and grows its constructors through a fractional advanced plant
    /// (terr13-lp-nullai-comet-1, game 01: the lab idle from 7:00 to 14:00 with both stocks at their caps, the
    /// program asking three constructors more each minute, all of the tier-2 kind, from a plant nobody started).
    /// Everything else goes by the program's flows for this builder's type: its credit on a unit type is the power
    /// the program has the type spend on it to the clock plus the lead, less what the type has committed to it
    /// since the plan was adopted (the standing orders' unbuilt cost seeds it); the largest credit as a share of
    /// the unit's cost is taken, however small, since standing idle is never what the program planned: a building
    /// built where T-SITES puts it, a factory product as a guard on the factory that makes it. With no credit, the
    /// largest count shortfall of a type it builds; with none, help on the dearest frame.
    fn dispatch(&mut self, id: UnitId, role: Role, commands: &mut Vec<Command>) {
        let Some(builder) = self.own.get(&id).cloned() else { return };
        let Some(unit) = self.unit_index(builder.def) else { return };
        let now = self.frame as f64 / SECOND as f64;
        let ahead = now + LEAD;
        let shortfalls = self.shortfalls(ahead);
        // T-COMMIT: a factory type the program wants at all within the window, among whose products is one the
        // program wants that no factory standing, framed or ordered makes, is started whole, before anything else:
        // the first lab, and later the plant of the tier-2 constructors the upgrades need (terr16-lp-nullai-comet-1:
        // the program wanted 0.3 of an advanced plant from 7:00 beside more labs, the class rule took the larger
        // fraction, four labs stood by 9:48 and no upgrade all game; its own ceiling runs on 0.8 of an advanced
        // vehicle plant). Not a kind whose products are all made already: the program spreads fractions over
        // every factory type, and terr7-lp-nullai-comet-1 stood three kinds by 5:00 on one commander's income.
        let commit: Option<usize> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let ordered = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } => Some(*unit), _ => None });
            let covered: Vec<usize> = self.own.values().filter_map(|f| self.unit_index(f.def)).chain(ordered).filter(|f| units.list[*f].role == Role::Factory).flat_map(|f| units.list[f].builds.iter().copied()).collect();
            let wanted_soon = |u: usize| (0..=6).any(|k| targets.count_at(u, now + k as f64 * COMMIT_WITHIN / 6.0) >= COMMIT_AT);
            targets
                .types
                .iter()
                .copied()
                .filter(|f| units.list[*f].role == Role::Factory && def.builds.contains(f) && !self.hands.struck.contains(f))
                .filter(|f| wanted_soon(*f) && units.list[*f].builds.iter().any(|p| !covered.contains(p) && wanted_soon(*p)))
                .max_by(|a, b| targets.count_at(*a, now + COMMIT_WITHIN).total_cmp(&targets.count_at(*b, now + COMMIT_WITHIN)))
        };
        // The classes: the factories as a whole for a mobile builder; a factory's products by role.
        let class_pick: Option<usize> = commit.or_else(|| {
            let units = &self.game().units;
            let def = &units.list[unit];
            let mine = |u: &usize| def.builds.contains(u) && !self.hands.struck.contains(u);
            // A product's class is its role and its tier (an advanced constructor builds an upgrade): a lab
            // substitutes its kind for the program's basic wants only, never for the advanced plant's (terr20:
            // substituting for the program's fractional advanced vehicle constructors, which nothing stood to
            // make, the lab ran to 136 constructors once the caps no longer stopped it).
            let advanced = |u: usize| units.list[u].builds.iter().any(|b| self.is_upgrade(*b));
            let mut classes: Vec<(Role, bool)> = Vec::new();
            if role == Role::Factory {
                for (u, _) in shortfalls.iter().filter(|(u, _)| mine(u)) {
                    let class = (units.list[*u].role, advanced(*u));
                    if !classes.contains(&class) {
                        classes.push(class);
                    }
                }
            } else {
                classes.push((Role::Factory, false));
            }
            // The type started is the one of the class the program wants most. A builder that cannot build it leaves
            // the class to one that can; a factory that does not make it substitutes its own best of the class only
            // while no factory standing, framed or ordered makes it (terr15-lp-nullai-comet-1: the lab filled the
            // forty-constructor cap with its own kind while the program wanted the advanced plant's, which then made
            // seven and stopped; terr19: substituting through the plant's frame it stood 51 against the tier's cap
            // of forty, and the program had no feasible point from then on).
            let ordered = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } => Some(*unit), _ => None });
            let covered: Vec<usize> = self.own.values().filter_map(|f| self.unit_index(f.def)).chain(ordered).filter(|f| units.list[*f].role == Role::Factory).flat_map(|f| units.list[f].builds.iter().copied()).collect();
            let made_by_a_standing_factory = |u: usize| covered.contains(&u);
            classes
                .into_iter()
                .filter_map(|(class, tier)| {
                    let in_class = |u: usize| units.list[u].role == class && (class != Role::Builder || advanced(u) == tier);
                    let of_class = |u: &usize| in_class(*u) && !self.hands.struck.contains(u);
                    let total: f64 = shortfalls.iter().filter(|(u, _)| in_class(*u)).map(|(_, s)| *s).sum();
                    if total < START_AT {
                        return None;
                    }
                    let wanted = shortfalls.iter().filter(|(u, _)| of_class(u)).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _)| *u)?;
                    let pick = if mine(&wanted) {
                        wanted
                    } else if role == Role::Factory && !made_by_a_standing_factory(wanted) {
                        shortfalls.iter().filter(|(u, _)| of_class(u) && mine(u)).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _)| *u)?
                    } else {
                        return None;
                    };
                    Some((pick, total))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(u, _)| u)
        });
        if let Some(u) = class_pick && self.order_one(id, unit, role, u, self.game().units.list[u].build_time, commands) {
            return;
        }
        if role == Role::Factory {
            return;
        }
        // The energy as a class: the program's planned energy income at the lead, less what stands, is framed or is
        // ordered, realized by the best affordable energy building this builder builds. The program's cheapest energy
        // per metal is a fusion, which it builds as a fraction from its second minute and draws energy from at once;
        // the engine's first energy is solars, and the program's solar flows are small beside the fusion's, so the
        // early game stalled at zero energy from 1:00 to 5:00 (terr17-lp-nullai-comet-1).
        // At most three orders at a time: the program's planned energy runs on a fraction of a fusion, so the
        // shortfall is always large (terr19-lp-nullai-comet-1: read by every idle builder it put the first
        // constructors on four advanced solars by 4:31; terr20: two orders by constructors alone left the energy at
        // zero from 1:00 to 8:00 and the income at seven at 5:00, since the engine's first energy is solars and the
        // lab alone wants ninety a second). The backstop since terr22: the program is offered affordable types only
        // and its energy flows are followed like any other (excluded, the fusions the late program wants by the
        // dozen never came: eight by 20:00 in terr21, the bank at 49,000).
        let energy: Option<(usize, f64)> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let rate = |u: usize| units.list[u].energy_make;
            let pending = self.hands.orders.values().filter(|o| matches!(&o.what, What::Build { unit, frame: None, .. } if self.is_energy(*unit))).count();
            let had: f64 = self.own.values().filter_map(|o| self.unit_index(o.def)).filter(|u| self.is_energy(*u)).map(rate).sum::<f64>()
                + self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } if self.is_energy(*unit) => Some(rate(*unit)), _ => None }).sum::<f64>();
            let short = if pending < ENERGY_PENDING && self.energy.0 < ENERGY_LOW * self.energy.1 { targets.energy_income_at(ahead) - units.list[self.game().commander].energy_make - had } else { 0.0 };
            // Of the program's own types (the roster leaves out what stands only on a geothermal vent).
            targets
                .types
                .iter()
                .copied()
                .filter(|u| def.builds.contains(u) && self.is_energy(*u) && !self.hands.struck.contains(u) && self.affordable(*u))
                .max_by(|a, b| (rate(*a) / units.list[*a].metal_cost.max(1.0)).total_cmp(&(rate(*b) / units.list[*b].metal_cost.max(1.0))))
                .filter(|u| short >= 0.5 * rate(*u))
                .map(|u| (u, units.list[u].build_time))
        };
        if let Some((u, cost)) = energy && self.order_one(id, unit, role, u, cost, commands) {
            return;
        }
        // The flows of this builder's type, the largest credit as a share of the unit's cost first. A builder of a
        // type the plan had none of at its start has no flows worth reading (the program gave a type with no
        // units no power, and what little a fraction of one got ranks a construction turret first:
        // terr16-lp-nullai-comet-1, the first constructor's first job at 1:23); it goes by the count shortfalls
        // until the re-plan its coming asks for arrives.
        let planned_type = unit == self.game().commander || self.hands.targets.as_ref().is_some_and(|t| t.count_at(unit, t.t0) >= 0.5);
        let mut flows: Vec<(usize, Kind, f64, f64)> = {
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let units = &self.game().units;
            targets
                .flows
                .iter()
                .filter(|f| planned_type && f.builder == unit && units.list[f.unit].role != Role::Factory)
                .map(|f| (f.unit, f.kind, targets.planned_power(unit, f.unit, ahead) - self.hands.committed.get(&(unit, f.unit)).copied().unwrap_or(0.0), f.cost))
                .filter(|(_, _, credit, _)| *credit > 0.0)
                .collect()
        };
        flows.sort_by(|a, b| (b.2 / b.3.max(1.0)).total_cmp(&(a.2 / a.3.max(1.0))));
        for (u, kind, _, cost) in flows {
            match kind {
                Kind::Direct => {
                    if self.order_one(id, unit, role, u, cost, commands) {
                        return;
                    }
                }
                Kind::Assist | Kind::Help => {
                    let Some(factory) = self.factory_making(u, builder.pos) else { continue };
                    commands.push(Command::Guard { unit: id, target: factory });
                    self.hands.orders.insert(id, Order { what: What::Assist { factory, until: self.frame + ASSIST_CHUNK }, given: self.frame, told: self.frame, arrived: None, step: None });
                    *self.hands.committed.entry((unit, u)).or_default() += self.game().units.list[unit].worker_time * ASSIST_CHUNK as f64 / SECOND as f64;
                    return;
                }
            }
        }
        // No credit: the largest count shortfall of a type it builds, in build-power-seconds; then the dearest frame.
        let fallback: Option<usize> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            shortfalls
                .iter()
                .filter(|(u, short)| *short > 0.0 && units.list[*u].role != Role::Factory && def.builds.contains(u) && !self.hands.struck.contains(u))
                .max_by(|a, b| (a.1 * units.list[a.0].build_time).total_cmp(&(b.1 * units.list[b.0].build_time)))
                .map(|(u, _)| *u)
        };
        if let Some(u) = fallback && self.order_one(id, unit, role, u, self.game().units.list[u].build_time, commands) {
            return;
        }
        if let Some(frame) = self.dearest_frame() {
            commands.push(Command::Repair { unit: id, target: frame, queue: false });
            self.hands.orders.insert(id, Order { what: What::Help { frame }, given: self.frame, told: self.frame, arrived: None, step: None });
        }
    }

    /// One unit of `u` ordered from this builder (T-SITES places it) and charged to its type's commitment; false
    /// when the hands pass it over (not affordable, no site, the cap, a struck type).
    fn order_one(&mut self, id: UnitId, unit: usize, role: Role, u: usize, cost: f64, commands: &mut Vec<Command>) -> bool {
        if !self.affordable(u) || self.hands.type_refused.get(&u).is_some_and(|until| self.frame < *until) {
            return false;
        }
        let attempt = self.hands.attempts.get(&id).copied().unwrap_or(0);
        let Some((command, what)) = self.give(id, unit, role, Step::build(u), attempt) else { return false };
        commands.push(command);
        self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: None });
        *self.hands.committed.entry((unit, u)).or_default() += cost;
        true
    }

    /// Whether the stock and `AFFORD_SECONDS` of income cover a unit's metal cost.
    fn affordable(&self, u: usize) -> bool {
        self.game().units.list[u].metal_cost <= self.stock.0 + self.stock.1 * AFFORD_SECONDS
    }

    /// An energy building: makes energy, extracts and converts nothing (the commander is not one).
    pub(crate) fn is_energy(&self, u: usize) -> bool {
        let unit = &self.game().units.list[u];
        unit.role == Role::Eco && unit.energy_make > 0.0 && unit.extracts_metal == 0.0 && unit.conv_capacity == 0.0
    }

    /// The standing factory nearest `from` that makes `u`.
    fn factory_making(&self, u: usize, from: Vec3) -> Option<UnitId> {
        let units = &self.game().units;
        self.own
            .values()
            .filter(|f| !f.being_built && self.unit_index(f.def).is_some_and(|fi| units.list[fi].role == Role::Factory && units.list[fi].builds.contains(&u)))
            .min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from)))
            .map(|f| f.id)
    }

    /// Per unit type in the program: its target at `t` less what stands, is framed, or is ordered.
    fn shortfalls(&self, t: f64) -> Vec<(usize, f64)> {
        let Some(targets) = self.hands.targets.as_ref() else { return Vec::new() };
        let mut have: BTreeMap<usize, f64> = BTreeMap::new();
        for u in self.own.values() {
            if let Some(i) = self.unit_index(u.def) {
                *have.entry(i).or_default() += 1.0;
            }
        }
        for order in self.hands.orders.values() {
            if let What::Build { unit, frame: None, .. } = &order.what {
                *have.entry(*unit).or_default() += 1.0;
            }
        }
        targets.types.iter().map(|u| (*u, targets.count_at(*u, t) - have.get(u).copied().unwrap_or(0.0))).collect()
    }

    fn review(&mut self, id: UnitId, order: &Order, commands: &mut Vec<Command>) -> Review {
        let Some(builder) = self.own.get(&id) else { return Review::Done };
        let age = self.frame - order.given;
        let since_told = self.frame - order.told;
        match &order.what {
            What::Build { frame: Some(frame), .. } => match self.own.get(frame) {
                Some(u) if u.being_built => {
                    // The builder let go of a frame that still stands (an idle flag with a frame beside it): put it
                    // back on it.
                    if builder.idle && since_told > GRACE {
                        commands.push(Command::Repair { unit: id, target: *frame, queue: false });
                        self.hands.orders.get_mut(&id).expect("reviewed").told = self.frame;
                    }
                    Review::Keep
                }
                _ => Review::Done,
            },
            What::Build { frame: None, near, .. } => {
                // Held: standing within reach of the site this long with no frame begun, from the first frame the
                // builder stood that near; a builder still walking (an extractor 4,600 elmos out, terr3-nullai-comet-1
                // game 00) is given the walk, up to three times the limit.
                let arrived = order.arrived.or_else(|| (builder.pos.dist2d(*near) < HELD_NEAR).then_some(self.frame));
                if arrived != order.arrived && let Some(o) = self.hands.orders.get_mut(&id) {
                    o.arrived = arrived;
                }
                if arrived.is_some_and(|a| self.frame - a > HELD) || age > 3 * HELD {
                    Review::Failed("held")
                } else if builder.idle && since_told > GRACE {
                    Review::Failed("dropped")
                } else {
                    Review::Keep
                }
            }
            What::Assist { factory, until } => {
                if self.frame >= *until || !self.own.contains_key(factory) {
                    Review::Done
                } else {
                    Review::Keep
                }
            }
            What::Help { frame } => match self.own.get(frame) {
                Some(u) if u.being_built => {
                    if builder.idle && since_told > GRACE {
                        commands.push(Command::Repair { unit: id, target: *frame, queue: false });
                        self.hands.orders.get_mut(&id).expect("reviewed").told = self.frame;
                    }
                    Review::Keep
                }
                _ => Review::Done,
            },
        }
    }

    /// An order that came to nothing: logged, and its step given again once (at another site) before it is skipped.
    pub(crate) fn failed(&mut self, id: UnitId, why: &str) {
        let Some(order) = self.hands.orders.remove(&id) else { return };
        self.hands.dropped += 1;
        let what = match &order.what {
            What::Build { def, near, .. } => {
                // What stands on the site and what of his is near: the held extractor orders (terr13-lp-nullai-comet-1:
                // 21 in one game, two minutes of a constructor each, at spots all over the map).
                let name = |d: UnitDefId| self.def(d).map_or("?", |d| d.name.as_str());
                let on_site: Vec<String> = self.own.values().filter(|u| u.id != id && u.pos.dist2d(*near) < ON_SITE).map(|u| format!("{}{}", name(u.def), if u.being_built { "(frame)" } else if u.idle { "(idle)" } else { "" })).collect();
                let his: Vec<String> = self.enemies.iter().filter(|e| e.pos.dist2d(*near) < 5.0 * ON_SITE).map(|e| e.def.map_or("?", name).to_string()).collect();
                let builder = self.own.get(&id).map_or("gone".to_string(), |b| format!("{:.0} off{}", b.pos.dist2d(*near), if b.idle { ", idle" } else { "" }));
                format!("{} at ({:.1},{:.1}); builder {builder}; on the site: {}; his: {}", name(*def), near.x, near.z, if on_site.is_empty() { "-".to_string() } else { on_site.join(" ") }, if his.is_empty() { "-".to_string() } else { his.join(" ") })
            }
            What::Assist { .. } => "assist".to_string(),
            What::Help { .. } => "help".to_string(),
        };
        self.log(format_args!("dropped: unit {} {what} after {:.1} s: {why}", id.0, (self.frame - order.given) as f32 / SECOND as f32));
        if let What::Build { unit, near, .. } = &order.what
            && self.game().units.list[*unit].extracts_metal > 0.0
            && let Some(spot) = self.hello.metal_spots.iter().position(|s| s.dist2d(*near) < SPOT_TAKEN)
        {
            self.hands.refused.insert(spot, self.frame + SPOT_REFUSED);
        }
        if why == "no site" && let What::Build { unit, .. } = &order.what {
            self.hands.type_refused.insert(*unit, self.frame + TYPE_REFUSED);
        }
        let counted = self.frame >= FIRST_ORDERS;
        // Only an order the engine dropped or rejected says the type cannot be built; a site the host could not find,
        // or a builder held mid-walk, says nothing of the type (terr8-lp-nullai-comet-1 struck converters and
        // construction turrets on site failures round a crowded base); and never a factory.
        if counted && (why == "dropped" || why == "rejected") && let What::Build { unit, .. } = &order.what && self.game().units.list[*unit].role != Role::Factory {
            let failures = self.hands.failures.entry(*unit).or_default();
            *failures += 1;
            let failures = *failures;
            let successes = self.hands.successes.get(unit).copied().unwrap_or(0);
            if failures >= STRIKE && failures > successes && self.hands.struck.insert(*unit) {
                let name = self.game().units.list[*unit].name.clone();
                self.log(format_args!("struck: {name} after {failures} orders came to nothing and {successes} frames begun"));
                if let Some(planner) = self.planner.as_mut() {
                    let gone = |i: &Item| matches!(i, Item::Build(u) if *u == *unit);
                    planner.palette.mobile.retain(|i| !gone(i));
                    planner.palette.factory.retain(|i| !gone(i));
                }
            }
        }
        let attempts = self.hands.attempts.entry(id).or_default();
        if counted {
            *attempts += 1;
        }
        if *attempts < ATTEMPTS {
            if let (Some(step), Some((_, next))) = (order.step, self.hands.queues.get_mut(&id)) {
                *next = step;
            }
        } else {
            *attempts = 0;
            self.hands.skipped += 1;
        }
    }

    fn give_next(&mut self, id: UnitId, role: Role, commands: &mut Vec<Command>) {
        let Some(&(queue, mut next)) = self.hands.queues.get(&id) else { return };
        let Some(unit) = self.own.get(&id).and_then(|u| self.unit_index(u.def)) else { return };
        loop {
            let step = if queue == NO_QUEUE { None } else { self.hands.plan.queue(queue).get(next).copied() };
            let (step, from_queue) = match step {
                Some(step) => (step, true),
                // The simulator's default for a constructor whose queue has run out: the cheapest extractor it can
                // build, on the nearest free spot.
                None if role == Role::Builder => match self.game().units.extractor(unit) {
                    Some(extractor) => (Step::build(extractor), false),
                    None => return,
                },
                None => return,
            };
            if from_queue {
                next += 1;
                self.hands.queues.insert(id, (queue, next));
            }
            let attempt = self.hands.attempts.get(&id).copied().unwrap_or(0);
            if let Some((command, what)) = self.give(id, unit, role, step, attempt) {
                commands.push(command);
                self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: from_queue.then_some(next - 1) });
                return;
            }
            if !from_queue {
                return;
            }
        }
    }

    /// One step as an order, placed as the simulator places it (T-SITES); none when the step is passed over, as the
    /// simulator passes it over (no free spot, a unit this builder cannot build, a factory beyond the plan's queues).
    fn give(&mut self, id: UnitId, builder_unit: usize, role: Role, step: Step, attempt: u32) -> Option<(Command, What)> {
        let builder = self.own.get(&id)?.clone();
        let game = self.game();
        let units = &game.units;
        match step.item {
            Item::Assist => {
                if role == Role::Factory {
                    return None;
                }
                let factory = self.first_factory()?;
                Some((Command::Guard { unit: id, target: factory }, What::Assist { factory, until: self.frame + ASSIST_CHUNK }))
            }
            Item::Help => {
                if role == Role::Factory {
                    return None;
                }
                let frame = self.dearest_frame()?;
                Some((Command::Repair { unit: id, target: frame, queue: false }, What::Help { frame }))
            }
            Item::Build(unit) => {
                if !units.list[builder_unit].builds.contains(&unit) || self.hands.struck.contains(&unit) {
                    return None;
                }
                let def = self.hello.unit_defs[unit].id;
                let target = &units.list[unit];
                if role == Role::Factory {
                    return Some((Command::Build { unit: id, def, site: None, queue: false }, What::Build { unit, def, near: builder.pos, frame: None }));
                }
                let placements = || vec![Placement { facing: self.facing_enemy(), keep_out: self.keep_out() }];
                let site = if target.role == Role::Factory {
                    // The simulator's rule for the anneal's plan: no more factories than it has queues for. With
                    // the program's targets the cap is the program's own (terr6-lp-nullai-comet-1: the empty plan's
                    // zero queues refused every lab, and the commander played alone for twenty minutes).
                    let cap = if self.hands.targets.is_some() { self.options.factories } else { self.hands.plan.factories.len() };
                    if self.factories_planned() >= cap {
                        return None;
                    }
                    // Toward his start; each factory after the first a quarter turn further round home, and a dropped
                    // order one more (terr3-nullai-comet-1 game 00: the advanced lab's order dropped twice at the one
                    // site and the game never went to tier 2; terr15: second labs asked for at the first lab's site).
                    let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
                    let length = dx.hypot(dz).max(1.0);
                    let (ux, uz) = (dx / length, dz / length);
                    let (rx, rz) = match (self.factories_planned() as u32 + attempt) % 4 {
                        0 => (ux, uz),
                        1 => (-uz, ux),
                        2 => (uz, -ux),
                        _ => (-ux, -uz),
                    };
                    let near = Vec3 { x: self.home.x + rx * 350.0, y: self.home.y, z: self.home.z + rz * 350.0 };
                    BuildSite { near, search_radius: 900.0, min_dist: 8, placements: placements() }
                } else if self.is_upgrade(unit) {
                    // Over the standing basic extractor nearest the builder with nothing ordered over it.
                    let ordered: Vec<Vec3> = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, near, .. } if self.is_upgrade(*unit) => Some(*near), _ => None }).collect();
                    let standing = self
                        .own
                        .values()
                        .filter(|u| !u.being_built && u.id != id)
                        .filter(|u| role != Role::Commander || u.pos.dist2d(self.home) < COMMANDER_LEASH)
                        .filter(|u| self.unit_index(u.def).is_some_and(|i| units.list[i].extracts_metal > 0.0 && !self.is_upgrade(i)))
                        .filter(|u| !ordered.iter().any(|o| o.dist2d(u.pos) < SPOT_TAKEN))
                        .min_by(|a, b| a.pos.dist2d(builder.pos).total_cmp(&b.pos.dist2d(builder.pos)))?;
                    // Logged to the hundredth: the game's "Deny Invalid Mex Orders" rule allows an extractor over
                    // ours only at its exact position, and every Cortex order was dropped in terr1-nullai-comet-1.
                    self.log(format_args!("upgrade: {} over unit {} at ({:.2},{:.2}) by unit {}", target.name, standing.id.0, standing.pos.x, standing.pos.z, id.0));
                    BuildSite { near: standing.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new() }
                } else if target.extracts_metal > 0.0 {
                    let leash = (role == Role::Commander).then_some((self.home, COMMANDER_LEASH));
                    let spot = self.free_spot(builder.pos, step.site, leash)?;
                    BuildSite { near: self.hello.metal_spots[spot], search_radius: 0.0, min_dist: 0, placements: Vec::new() }
                } else if target.role == Role::Nano {
                    // Behind the nearest factory, inside the turret's reach of it; the base spiral without one.
                    match self.nearest_factory(builder.pos) {
                        Some(factory) => {
                            let (dx, dz) = (self.enemy_home.x - factory.pos.x, self.enemy_home.z - factory.pos.z);
                            let length = dx.hypot(dz).max(1.0);
                            let near = Vec3 { x: factory.pos.x - dx / length * NANO_BEHIND, y: factory.pos.y, z: factory.pos.z - dz / length * NANO_BEHIND };
                            BuildSite { near, search_radius: 300.0, min_dist: 1, placements: placements() }
                        }
                        None => self.base_site_next(placements()),
                    }
                } else if target.role == Role::Eco && attempt == 0 && (self.hands.targets.is_some() || builder.pos.dist2d(self.home) < BESIDE) {
                    // Beside a builder that is about the base anyway, as the simulator charges no walk for it; under
                    // the program beside the builder wherever it stands (a constructor afield sent home for a solar
                    // walked 10-32 s at the median, terr13-lp-nullai-comet-1; nothing of his raids an outpost here).
                    BuildSite { near: builder.pos, search_radius: 500.0, min_dist: 3, placements: placements() }
                } else {
                    self.base_site_next(placements())
                };
                let near = site.near;
                Some((Command::Build { unit: id, def, site: Some(site), queue: false }, What::Build { unit, def, near, frame: None }))
            }
        }
    }

    fn base_site_next(&mut self, placements: Vec<Placement>) -> BuildSite {
        self.hands.base_sites += 1;
        BuildSite { near: base_site(self.home, self.hands.base_sites - 1), search_radius: 400.0, min_dist: 3, placements }
    }

    /// The free spot of the scenario's list nearest `from`; with `named`, the free spot within 100 of it or none;
    /// with `leash`, within that radius of that point.
    fn free_spot(&self, from: Vec3, named: Option<(f64, f64)>, leash: Option<(Vec3, f32)>) -> Option<usize> {
        let spots = &self.hello.metal_spots;
        let taken = |i: usize| {
            let at = spots[i];
            self.hands.refused.get(&i).is_some_and(|until| self.frame < *until)
                || self.own.values().any(|u| self.unit_index(u.def).is_some_and(|x| self.game().units.list[x].extracts_metal > 0.0) && u.pos.dist2d(at) < SPOT_TAKEN)
                || self.hands.orders.values().any(|o| matches!(&o.what, What::Build { unit, near, .. } if self.game().units.list[*unit].extracts_metal > 0.0 && near.dist2d(at) < SPOT_TAKEN))
        };
        let near_named = |i: usize| named.is_none_or(|(x, z)| spots[i].dist2d(Vec3 { x: x as f32, y: 0.0, z: z as f32 }) < SPOT_TAKEN);
        let leashed = |i: usize| leash.is_none_or(|(centre, radius)| spots[i].dist2d(centre) < radius);
        self.spots.iter().copied().filter(|i| !taken(*i) && near_named(*i) && leashed(*i)).min_by(|a, b| spots[*a].dist2d(from).total_cmp(&spots[*b].dist2d(from)))
    }

    /// Factories standing or ordered, so a plan cannot start more than it has queues for (the simulator's rule).
    fn factories_planned(&self) -> usize {
        let standing = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Factory)).count();
        let ordered = self.hands.orders.values().filter(|o| matches!(&o.what, What::Build { unit, frame: None, .. } if self.game().units.list[*unit].role == Role::Factory)).count();
        standing + ordered
    }

    fn first_factory(&self) -> Option<UnitId> {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Factory)).map(|u| u.id).min()
    }

    fn nearest_factory(&self, from: Vec3) -> Option<&OwnUnit> {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Factory)).min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from)))
    }

    /// The building frame with the most metal still to build per builder on it, one more counted (a factory's pad
    /// is not a frame): the help spreads over the frames rather than piling on the dearest (terr22-lp-nullai-comet-1:
    /// 48 constructors on one frame at 20:00 while five fusions stood with a builder each, and the program's power
    /// on fusions, committed to those frames, never cleared for new ones).
    fn dearest_frame(&self) -> Option<UnitId> {
        let helpers = |frame: UnitId| self.hands.orders.values().filter(|o| matches!(&o.what, What::Help { frame: f } | What::Build { frame: Some(f), .. } if *f == frame)).count() as f32;
        self.own
            .values()
            .filter(|u| u.being_built)
            .filter_map(|u| {
                let d = self.def(u.def)?;
                (d.speed == 0.0).then(|| (u.id, d.metal_cost * (1.0 - u.health / u.max_health.max(1.0)) / (1.0 + helpers(u.id))))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }

    /// The engine named a frame begun by one of our builders: the builder's order has its frame, and the engine
    /// log the order's age and the builder's distance (the walk calibration).
    pub(crate) fn frame_begun(&mut self, unit: UnitId, builder: UnitId) {
        let Some(frame) = self.own.get(&unit).cloned() else { return };
        let begun_unit = self.unit_index(frame.def);
        let Some(order) = self.hands.orders.get_mut(&builder) else { return };
        let What::Build { def, frame: slot, .. } = &mut order.what else { return };
        if *def != frame.def || slot.is_some() {
            return;
        }
        *slot = Some(unit);
        let given = order.given;
        // A frame begun clears the builder's failures: the attempt count is per run of failures, and left standing it
        // sent every economy building of a builder that had once failed to the base spiral, from wherever it stood
        // (terr14-lp-nullai-comet-1: 30 of 128 economy drops with the builder more than 1,500 off, one 5,400).
        self.hands.attempts.remove(&builder);
        if let Some(u) = begun_unit {
            self.hands.begun_log.entry(builder).or_default().push((self.frame, u));
            *self.hands.successes.entry(u).or_default() += 1;
        }
        self.hands.begun += 1;
        let age = (self.frame - given) as f32 / SECOND as f32;
        let distance = self.own.get(&builder).map_or(0.0, |b| b.pos.dist2d(frame.pos));
        let name = self.def(frame.def).map_or("?", |d| d.name.as_str());
        self.log(format_args!("begun: {name} by unit {} at {distance:.0}, {age:.1} s after its order, site ({:.0},{:.0})", builder.0, frame.pos.x, frame.pos.z));
    }

    /// The host placed a builder's site: the order's position is where the frame will stand (the hold clock and the
    /// dropped line read it), and other sites keep off it until the frame stands.
    pub(crate) fn sited(&mut self, builder: UnitId, at: Vec3) {
        if let Some(order) = self.hands.orders.get_mut(&builder)
            && let What::Build { frame: None, near, .. } = &mut order.what
        {
            *near = at;
        }
    }

    /// A unit of ours is gone: a builder's orders and queue with it, and an order whose frame it was.
    pub(crate) fn unit_lost(&mut self, unit: UnitId) {
        self.hands.orders.remove(&unit);
        self.hands.queues.remove(&unit);
        let holders: Vec<UnitId> = self.hands.orders.iter().filter(|(_, o)| matches!(&o.what, What::Build { frame: Some(f), .. } | What::Help { frame: f } if *f == unit)).map(|(id, _)| *id).collect();
        for holder in holders {
            self.failed(holder, "frame lost");
        }
    }

    /// Construction turrets feed the nearest factory within their reach (the calibrated replays' behaviour; the
    /// simulator's `nanos_help_frames` is off to match).
    pub(crate) fn run_nanos(&mut self, commands: &mut Vec<Command>) {
        let nanos: Vec<(UnitId, Vec3)> = self.own.values().filter(|u| !u.being_built && u.idle && self.role_of_def(u.def) == Some(Role::Nano)).map(|u| (u.id, u.pos)).collect();
        for (nano, at) in nanos {
            if self.hands.nano_told.get(&nano).is_some_and(|told| self.frame - told < GRACE) {
                continue;
            }
            // With the program's targets a turret helps the dearest building frame in its reach, as the program
            // prices its power (bounded by the builders' own); else, and with none in reach, the nearest factory.
            let frame = self.hands.targets.as_ref().and_then(|_| {
                self.own
                    .values()
                    .filter(|u| u.being_built && u.pos.dist2d(at) < NANO_REACH && self.def(u.def).is_some_and(|d| d.speed == 0.0))
                    .max_by(|a, b| (self.def(a.def).map_or(0.0, |d| d.metal_cost) * (1.0 - a.health / a.max_health.max(1.0))).total_cmp(&(self.def(b.def).map_or(0.0, |d| d.metal_cost) * (1.0 - b.health / b.max_health.max(1.0)))))
                    .map(|u| u.id)
            });
            if let Some(frame) = frame {
                self.hands.nano_told.insert(nano, self.frame);
                commands.push(Command::Repair { unit: nano, target: frame, queue: false });
                continue;
            }
            let Some(factory) = self.nearest_factory(at).filter(|f| f.pos.dist2d(at) < NANO_REACH).map(|f| f.id) else { continue };
            self.hands.nano_told.insert(nano, self.frame);
            commands.push(Command::Guard { unit: nano, target: factory });
        }
    }

    /// The engine's facing (0 south +z, 1 east +x, 2 north -z, 3 west -x) that looks most toward his start.
    fn facing_enemy(&self) -> i32 {
        let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        match (dx.abs() > dz.abs(), dx > 0.0, dz > 0.0) {
            (true, true, _) => 1,
            (true, false, _) => 3,
            (false, _, true) => 0,
            (false, _, false) => 2,
        }
    }

    /// Our factories' exit lanes and yards, and the building sites placed and not yet begun: nothing is built in
    /// them (a lane of no length is a disc).
    fn keep_out(&self) -> Vec<Lane> {
        let units = &self.game().units;
        let pending = self.hands.orders.values().filter_map(|o| match &o.what {
            What::Build { unit, near, frame: None, .. } if units.list[*unit].speed == 0.0 => Some(Lane { from: *near, to: *near, half_width: PENDING_SITE }),
            _ => None,
        });
        self.own
            .values()
            .filter(|u| self.role_of_def(u.def) == Some(Role::Factory))
            .flat_map(|plant| {
                let (dx, dz) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][plant.facing.rem_euclid(4) as usize];
                [
                    Lane { from: plant.pos, to: Vec3 { x: plant.pos.x + dx * 400.0, y: plant.pos.y, z: plant.pos.z + dz * 400.0 }, half_width: 120.0 },
                    Lane { from: plant.pos, to: plant.pos, half_width: FACTORY_YARD },
                ]
            })
            .chain(pending)
            .collect()
    }

    /// The builders' orders counted by kind, for the jobs line.
    pub(crate) fn jobs_line(&self) -> String {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for (id, order) in &self.hands.orders {
            let role = self.own.get(id).and_then(|u| self.role_of_def(u.def));
            let kind = match role {
                Some(Role::Commander) => "com",
                Some(Role::Factory) => "fac",
                _ => "con",
            };
            let word = match &order.what {
                What::Build { def, frame, .. } => format!("{}{}", self.def(*def).map_or("?", |d| d.name.as_str()), if frame.is_some() { "" } else { "(walking)" }),
                What::Assist { .. } => "assist".to_string(),
                What::Help { .. } => "help".to_string(),
            };
            *counts.entry(format!("{kind}:{word}")).or_default() += 1;
        }
        counts.iter().map(|(k, n)| if *n > 1 { format!("{k} x{n}") } else { k.clone() }).collect::<Vec<_>>().join(" ")
    }
}
