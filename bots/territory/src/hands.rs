//! The hands: the plan in force carried out one order per builder, placed as the simulator places things
//! (docs/design/2026-10-20-territory-bot.md, decision 5; the README's T-ORDERS, T-SITES).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bot_protocol::{BuildSite, Command, Lane, OwnUnit, Placement, UnitDefId, UnitId, Vec3};
use buildorder::plan::{Item, Plan, Step};
use buildorder::units::Role;

use crate::{Territory, MINUTE, SECOND};

/// Frames a unit is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const GRACE: i32 = 45;
/// T-ORDERS: a build order with no frame begun after this long is held for nothing and dropped (the engine starts no
/// frame while it reclaims a wreck on the site or waits for a blocker, and drops a blocked build itself after thirty
/// tries; the distilled bot's D-ORDERS found a shorter watchdog took orders from builders in range of their spots).
const HELD: i32 = 2 * MINUTE;
/// ... counted from when the builder stands this near the site.
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
/// A spot with an extractor, a frame or an order of ours this near is taken.
const SPOT_TAKEN: f32 = 100.0;
/// A refused spot is left alone this long.
const SPOT_REFUSED: i32 = MINUTE;
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
    pub orders: BTreeMap<UnitId, Order>,
    /// Per builder: its queue in the plan and the index of its next step.
    pub queues: BTreeMap<UnitId, (usize, usize)>,
    /// The plan's queues for factories and constructors that did not stand when it was made, in order.
    pub spare_factory_queues: VecDeque<usize>,
    pub spare_constructor_queues: VecDeque<usize>,
    pub attempts: BTreeMap<UnitId, u32>,
    /// Spots (indices into the game's list) refused by the host or the engine, until this frame.
    pub refused: BTreeMap<usize, i32>,
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
            orders: BTreeMap::new(),
            queues: BTreeMap::new(),
            spare_factory_queues: VecDeque::new(),
            spare_constructor_queues: VecDeque::new(),
            attempts: BTreeMap::new(),
            refused: BTreeMap::new(),
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
            if !self.hands.queues.contains_key(&id) {
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
            self.give_next(id, role, commands);
        }
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
                // Held: standing within reach of the site this long with no frame begun; a builder still walking
                // (an extractor 4,600 elmos out, terr3-nullai-comet-1 game 00) is given the walk, up to three times
                // the limit.
                let arrived = builder.pos.dist2d(*near) < HELD_NEAR;
                if (arrived && age > HELD) || age > 3 * HELD {
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
            What::Build { def, near, .. } => format!("{} at ({:.1},{:.1})", self.def(*def).map_or("?", |d| d.name.as_str()), near.x, near.z),
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
        let counted = self.frame >= FIRST_ORDERS;
        if counted && let What::Build { unit, .. } = &order.what {
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
                self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, step: from_queue.then_some(next - 1) });
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
                let placements = || vec![Placement { facing: self.facing_enemy(), keep_out: self.exit_lanes() }];
                let site = if target.role == Role::Factory {
                    if self.factories_planned() >= self.hands.plan.factories.len() {
                        return None;
                    }
                    // Toward his start; after a dropped order a quarter turn round home from there (terr3-nullai-comet-1
                    // game 00: the advanced lab's order dropped twice at the one site and the game never went to tier 2).
                    let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
                    let length = dx.hypot(dz).max(1.0);
                    let (ux, uz) = (dx / length, dz / length);
                    let (rx, rz) = match attempt % 4 {
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
                        .filter(|u| self.unit_index(u.def).is_some_and(|i| units.list[i].extracts_metal > 0.0 && !self.is_upgrade(i)))
                        .filter(|u| !ordered.iter().any(|o| o.dist2d(u.pos) < SPOT_TAKEN))
                        .min_by(|a, b| a.pos.dist2d(builder.pos).total_cmp(&b.pos.dist2d(builder.pos)))?;
                    // Logged to the hundredth: the game's "Deny Invalid Mex Orders" rule allows an extractor over
                    // ours only at its exact position, and every Cortex order was dropped in terr1-nullai-comet-1.
                    self.log(format_args!("upgrade: {} over unit {} at ({:.2},{:.2}) by unit {}", target.name, standing.id.0, standing.pos.x, standing.pos.z, id.0));
                    BuildSite { near: standing.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new() }
                } else if target.extracts_metal > 0.0 {
                    let spot = self.free_spot(builder.pos, step.site)?;
                    BuildSite { near: self.hello.metal_spots[spot], search_radius: 0.0, min_dist: 0, placements: Vec::new() }
                } else if target.role == Role::Nano {
                    // Behind the nearest factory, inside the turret's reach of it; the base spiral without one.
                    match self.nearest_factory(builder.pos) {
                        Some(factory) => {
                            let (dx, dz) = (self.enemy_home.x - factory.pos.x, self.enemy_home.z - factory.pos.z);
                            let length = dx.hypot(dz).max(1.0);
                            let near = Vec3 { x: factory.pos.x - dx / length * 150.0, y: factory.pos.y, z: factory.pos.z - dz / length * 150.0 };
                            BuildSite { near, search_radius: 300.0, min_dist: 1, placements: placements() }
                        }
                        None => self.base_site_next(placements()),
                    }
                } else if target.role == Role::Eco && attempt == 0 && builder.pos.dist2d(self.home) < BESIDE {
                    // Beside a builder that is about the base anyway, as the simulator charges no walk for it.
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

    /// The free spot of the scenario's list nearest `from`; with `named`, the free spot within 100 of it or none.
    fn free_spot(&self, from: Vec3, named: Option<(f64, f64)>) -> Option<usize> {
        let spots = &self.hello.metal_spots;
        let taken = |i: usize| {
            let at = spots[i];
            self.hands.refused.get(&i).is_some_and(|until| self.frame < *until)
                || self.own.values().any(|u| self.unit_index(u.def).is_some_and(|x| self.game().units.list[x].extracts_metal > 0.0) && u.pos.dist2d(at) < SPOT_TAKEN)
                || self.hands.orders.values().any(|o| matches!(&o.what, What::Build { unit, near, .. } if self.game().units.list[*unit].extracts_metal > 0.0 && near.dist2d(at) < SPOT_TAKEN))
        };
        let near_named = |i: usize| named.is_none_or(|(x, z)| spots[i].dist2d(Vec3 { x: x as f32, y: 0.0, z: z as f32 }) < SPOT_TAKEN);
        self.spots.iter().copied().filter(|i| !taken(*i) && near_named(*i)).min_by(|a, b| spots[*a].dist2d(from).total_cmp(&spots[*b].dist2d(from)))
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

    /// The building frame with the most metal still to build (a factory's pad is not a frame).
    fn dearest_frame(&self) -> Option<UnitId> {
        self.own
            .values()
            .filter(|u| u.being_built)
            .filter_map(|u| {
                let d = self.def(u.def)?;
                (d.speed == 0.0).then(|| (u.id, d.metal_cost * (1.0 - u.health / u.max_health.max(1.0))))
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
        if let Some(u) = begun_unit {
            self.hands.begun_log.entry(builder).or_default().push((self.frame, u));
            *self.hands.successes.entry(u).or_default() += 1;
        }
        self.hands.begun += 1;
        let age = (self.frame - given) as f32 / SECOND as f32;
        let distance = self.own.get(&builder).map_or(0.0, |b| b.pos.dist2d(frame.pos));
        let name = self.def(frame.def).map_or("?", |d| d.name.as_str());
        self.log(format_args!("begun: {name} by unit {} at {distance:.0}, {age:.1} s after its order", builder.0));
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

    /// Our factories' exit lanes: nothing is built in them.
    fn exit_lanes(&self) -> Vec<Lane> {
        self.own
            .values()
            .filter(|u| self.role_of_def(u.def) == Some(Role::Factory))
            .map(|plant| {
                let (dx, dz) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][plant.facing.rem_euclid(4) as usize];
                Lane { from: plant.pos, to: Vec3 { x: plant.pos.x + dx * 400.0, y: plant.pos.y, z: plant.pos.z + dz * 400.0 }, half_width: 120.0 }
            })
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
