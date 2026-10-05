//! The economy and production: the scripts the flagship's player wrote in every game it won, as rules over what
//! stands (`README.md`, D-OPENING to D-PRODUCTION). A builder is given a job when it has none; a factory a unit when
//! its queue is empty.

use std::collections::BTreeMap;

use bot_protocol::{BuildSite, Command, Lane, OwnUnit, Placement, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

use crate::{Distilled, MINUTE, SECOND};

/// Frames a unit is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const GRACE: i32 = 45;
/// How often a builder that is assisting is asked whether something better has come up.
const ASSIST_REVIEW: i32 = 10 * SECOND;
/// A spot whose extractor could not be placed is left alone this long.
const REFUSED: i32 = 60 * SECOND;
/// The commander builds extractors this far from home and no farther (D-COMMANDER).
const COMMANDER_WALK: f32 = 700.0;
/// An extractor farther from home than this gets a sentry beside it, from the builder that built it (D-SENTRY).
const SENTRY_FROM: f32 = 900.0;
/// A spot with an armed enemy this near is not walked to.
const SPOT_DANGER: f32 = 600.0;
/// Builders on energy at one time: two, and one more for every four constructors.
const ENERGY_BUILDERS: usize = 2;
const ENERGY_BUILDERS_PER: usize = 4;
/// Plants at the most, and construction turrets a plant: one before another plant is built, three once the plants
/// are all there. A plant's yard has to clear between units, so build power piled on one plant stops paying
/// (distilled-v1-hard game 1: one plant with six turrets, the metal store full for four minutes).
const PLANTS: usize = 8;
const TURRETS_A_PLANT: usize = 1;
const TURRETS_A_PLANT_AT_MOST: usize = 3;
/// The plants should be able to spend this share of the metal income on soldiers (D-BUILD-POWER); the rest is buildings.
const SOLDIERS_SHARE: f32 = 0.85;
/// Energy income is kept this much over what the soldiers being made draw (D-ENERGY).
const ENERGY_MARGIN: f32 = 1.25;
/// A constructor builds the advanced solar instead of the solar from this metal income.
const ADVANCED_FROM: f32 = 15.0;
const BUILD_POWER_FROM: i32 = 5 * MINUTE / 2;
const TURRET_RETELL: i32 = 10 * SECOND;
/// A construction turret is built within this of its plant's middle, inside its own reach of it.
const TURRET_REACH: f32 = 330.0;
/// The factory makes raiders until this frame and tanks with artillery after (D-PRODUCTION).
const TANKS_FROM: i32 = 8 * MINUTE;
const SCOUTS_FIRST: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Job {
    Extractor(usize),
    Solar,
    AdvancedSolar,
    Plant,
    /// A construction turret beside this plant.
    Turret(UnitId),
    Assist,
}

pub(crate) struct Order {
    job: Job,
    since: i32,
}

pub(crate) struct Economy {
    orders: BTreeMap<UnitId, Order>,
    /// Per metal spot, the frame until which it is left alone.
    refused: Vec<i32>,
    factory_ordered: BTreeMap<UnitId, i32>,
    turret_told: BTreeMap<UnitId, i32>,
    /// What the factories have been told to make: all, the scouts, and soldiers since the last constructor and scout.
    made: usize,
    scouts_made: usize,
    since_constructor: usize,
    since_scout: usize,
}

impl Economy {
    pub(crate) fn new(spots: usize) -> Self {
        Economy { orders: BTreeMap::new(), refused: vec![0; spots], factory_ordered: BTreeMap::new(), turret_told: BTreeMap::new(), made: 0, scouts_made: 0, since_constructor: 0, since_scout: 0 }
    }

    /// Units the factories have been told to make.
    pub(crate) fn made(&self) -> usize {
        self.made
    }

    fn working_on(&self, job: impl Fn(Job) -> bool) -> usize {
        self.orders.values().filter(|o| job(o.job)).count()
    }
}

impl Distilled {
    pub(crate) fn order_failed(&mut self, unit: UnitId, def: Option<UnitDefId>) {
        // The sentry queued behind an extractor found no site: the extractor's order stands.
        if def.is_some() && def == self.kit.sentry {
            return;
        }
        if let Some(Order { job: Job::Extractor(spot), .. }) = self.economy.orders.remove(&unit) {
            self.economy.refused[spot] = self.frame + REFUSED;
        }
    }

    pub(crate) fn run_builders(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        self.economy.orders.retain(|id, _| self.own.contains_key(id));
        let builders: Vec<OwnUnit> = self.own.values().filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed > 0.0 && !d.build_options.is_empty())).cloned().collect();
        for unit in builders {
            let free = match self.economy.orders.get(&unit.id) {
                None => true,
                Some(order) if order.job == Job::Assist => self.frame - order.since >= ASSIST_REVIEW,
                Some(order) => unit.idle && self.frame - order.since > GRACE,
            };
            if !free {
                continue;
            }
            let was_assisting = self.economy.orders.get(&unit.id).is_some_and(|o| o.job == Job::Assist);
            self.economy.orders.remove(&unit.id);
            let Some(job) = self.next_job(&unit, tick) else { continue };
            if !(job == Job::Assist && was_assisting && !unit.idle) {
                self.order(&unit, job, commands);
            }
            self.economy.orders.insert(unit.id, Order { job, since: self.frame });
        }
    }

    /// D-OPENING, D-COMMANDER, D-EXPAND, D-ENERGY, D-PLANTS: what a builder with nothing to do does next.
    fn next_job(&self, unit: &OwnUnit, tick: &Tick) -> Option<Job> {
        let commander = Some(unit.def) != self.kit.constructor;
        let on_energy = self.economy.working_on(|job| matches!(job, Job::Solar | Job::AdvancedSolar));
        let energy = self.energy_short(tick) && on_energy < ENERGY_BUILDERS + self.count(self.kit.constructor) / ENERGY_BUILDERS_PER;
        let assist = || self.finished(self.kit.plant).next().map(|_| Job::Assist);
        if commander {
            // The opening, read off what stands: two extractors, two solars, the plant.
            if self.count(self.kit.extractor) < 2
                && let Some(spot) = self.free_spot(unit, tick, f32::MAX)
            {
                return Some(Job::Extractor(spot));
            }
            if self.count(self.kit.solar) < 2 {
                return Some(Job::Solar);
            }
            if self.count(self.kit.plant) == 0 {
                return Some(Job::Plant);
            }
            if energy {
                return Some(Job::Solar);
            }
            return self.free_spot(unit, tick, COMMANDER_WALK).map(Job::Extractor).or_else(assist);
        }
        if energy {
            return Some(if tick.snapshot.metal.income >= ADVANCED_FROM && self.kit.advanced_solar.is_some() { Job::AdvancedSolar } else { Job::Solar });
        }
        if self.economy.working_on(|job| matches!(job, Job::Plant | Job::Turret(_))) == 0
            && let Some(job) = self.build_power_job(tick)
        {
            return Some(job);
        }
        self.free_spot(unit, tick, f32::MAX).map(Job::Extractor).or_else(assist)
    }

    /// The soldier the plants make now.
    fn soldier(&self) -> Option<&UnitDefInfo> {
        let def = if self.frame < TANKS_FROM { self.kit.raider.or(self.kit.tank) } else { self.kit.tank.or(self.kit.raider) };
        def.and_then(|def| self.def(def))
    }

    /// Metal a second the plants and their turrets can turn into soldiers, working without a pause: build power
    /// times the soldier's metal over its build time (the engine's own arithmetic). Those still being built count;
    /// turrets beyond three a plant do not.
    pub(crate) fn can_spend(&self) -> f32 {
        let Some(soldier) = self.soldier() else { return 0.0 };
        let power = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).map_or(0.0, |d| d.build_speed);
        let (plants, turrets) = (self.count(self.kit.plant), self.count(self.kit.turret));
        let total = plants as f32 * power(self.kit.plant) + turrets.min(TURRETS_A_PLANT_AT_MOST * plants) as f32 * power(self.kit.turret);
        total * soldier.metal_cost / soldier.build_time.max(1.0)
    }

    /// D-ENERGY: the store is low, or it is falling and not full, or the income is under what the soldiers the metal
    /// income pays for would draw.
    fn energy_short(&self, tick: &Tick) -> bool {
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        let soldiers_draw = self.soldier().map_or(0.0, |s| self.can_spend().min(metal.income) * s.energy_cost / s.metal_cost.max(1.0));
        energy.current < 0.3 * energy.storage || (energy.income < energy.usage && energy.current < 0.6 * energy.storage) || energy.income < ENERGY_MARGIN * soldiers_draw
    }

    /// D-BUILD-POWER: while the plants cannot spend the soldiers' share of the income, or metal piles up in the
    /// store, a construction turret beside the plant with the fewest, and another plant when each has two.
    fn build_power_job(&self, tick: &Tick) -> Option<Job> {
        let metal = tick.snapshot.metal;
        let short = self.can_spend() < SOLDIERS_SHARE * metal.income || metal.current > 0.5 * metal.storage;
        if self.frame < BUILD_POWER_FROM || !short {
            return None;
        }
        let plants: Vec<&OwnUnit> = self.finished(self.kit.plant).collect();
        // A turret is its nearest plant's, wherever the site search put it.
        let nearest = |at: Vec3| plants.iter().min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id);
        let turrets_at = |plant: &OwnUnit| self.own.values().filter(|u| Some(u.def) == self.kit.turret && nearest(u.pos) == Some(plant.id)).count();
        let fewest = plants.iter().map(|p| (turrets_at(p), p.id)).min();
        match fewest {
            Some((n, plant)) if self.kit.turret.is_some() && n < TURRETS_A_PLANT => Some(Job::Turret(plant)),
            _ if self.count(self.kit.plant) < PLANTS => Some(Job::Plant),
            Some((n, plant)) if self.kit.turret.is_some() && n < TURRETS_A_PLANT_AT_MOST => Some(Job::Turret(plant)),
            _ => None,
        }
    }

    /// D-BUILD-POWER: an idle construction turret helps the nearest plant in its reach, told again every ten seconds.
    pub(crate) fn run_turrets(&mut self, commands: &mut Vec<Command>) {
        let idle: Vec<(UnitId, Vec3)> = self.finished(self.kit.turret).filter(|t| t.idle && self.economy.turret_told.get(&t.id).is_none_or(|at| self.frame - at >= TURRET_RETELL)).map(|t| (t.id, t.pos)).collect();
        for (turret, at) in idle {
            let Some(plant) = self.finished(self.kit.plant).filter(|p| p.pos.dist2d(at) < TURRET_REACH).min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id) else { continue };
            self.economy.turret_told.insert(turret, self.frame);
            commands.push(Command::Guard { unit: turret, target: plant });
        }
    }

    /// D-EXPAND: the free spot that is nearest, counting the walk from the builder whole and the distance from home half.
    fn free_spot(&self, unit: &OwnUnit, tick: &Tick, reach: f32) -> Option<usize> {
        let extractor = |def: UnitDefId| self.def(def).is_some_and(|d| d.extracts_metal > 0.0);
        let armed = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_none_or(|d| d.weapon_count > 0);
        let free = |i: &usize| {
            let at = self.hello.metal_spots[*i];
            self.frame >= self.economy.refused[*i]
                && at.dist2d(self.home) <= reach
                && !self.economy.orders.values().any(|o| o.job == Job::Extractor(*i))
                && !self.own.values().any(|u| extractor(u.def) && u.pos.dist2d(at) < 90.0)
                && !tick.snapshot.allies.iter().any(|u| extractor(u.def) && u.pos.dist2d(at) < 90.0)
                && !self.army.his_buildings.values().any(|b| b.pos.dist2d(at) < 200.0)
                && !tick.snapshot.enemies.iter().any(|e| armed(e.def) && e.pos.dist2d(at) < SPOT_DANGER)
        };
        let cost = |i: &usize| unit.pos.dist2d(self.hello.metal_spots[*i]) + 0.5 * self.home.dist2d(self.hello.metal_spots[*i]);
        (0..self.hello.metal_spots.len()).filter(free).min_by(|a, b| cost(a).total_cmp(&cost(b)))
    }

    fn order(&self, unit: &OwnUnit, job: Job, commands: &mut Vec<Command>) {
        let build = |def: Option<UnitDefId>, site: BuildSite, queue: bool| def.map(|def| Command::Build { unit: unit.id, def, site: Some(site), queue });
        // Nothing is built in a plant's exit lane, and a plant faces the enemy: its units leave through its front.
        let placements = || vec![Placement { facing: self.facing_enemy(), keep_out: self.exit_lanes() }];
        match job {
            Job::Extractor(spot) => {
                let at = self.hello.metal_spots[spot];
                commands.extend(build(self.kit.extractor, BuildSite { near: at, search_radius: 0.0, min_dist: 0, placements: Vec::new() }, false));
                // D-SENTRY: an outer extractor gets a sentry on its enemy side, queued behind it.
                if at.dist2d(self.home) > SENTRY_FROM {
                    commands.extend(build(self.kit.sentry, BuildSite { near: self.toward_enemy(at, 120.0), search_radius: 200.0, min_dist: 1, placements: Vec::new() }, true));
                }
            }
            Job::Solar | Job::AdvancedSolar => {
                let def = if job == Job::Solar { self.kit.solar } else { self.kit.advanced_solar };
                commands.extend(build(def, BuildSite { near: self.toward_enemy(self.home, -250.0), search_radius: 1200.0, min_dist: 3, placements: placements() }, false));
            }
            Job::Turret(plant) => {
                let Some(plant) = self.own.get(&plant) else { return };
                // Behind the plant, away from its front.
                commands.extend(build(self.kit.turret, BuildSite { near: self.toward_enemy(plant.pos, -150.0), search_radius: 150.0, min_dist: 1, placements: placements() }, false));
            }
            Job::Plant => commands.extend(build(self.kit.plant, BuildSite { near: self.toward_enemy(self.home, 350.0), search_radius: 900.0, min_dist: 8, placements: placements() }, false)),
            Job::Assist => {
                let nearest = self.finished(self.kit.plant).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                commands.extend(nearest.map(|plant| Command::Guard { unit: unit.id, target: plant.id }));
            }
        }
    }

    /// The engine's facing (0 south +z, 1 east +x, 2 north -z, 3 west -x) that looks most toward the enemy's start.
    fn facing_enemy(&self) -> i32 {
        let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        match (dx.abs() > dz.abs(), dx > 0.0, dz > 0.0) {
            (true, true, _) => 1,
            (true, false, _) => 3,
            (false, _, true) => 0,
            (false, _, false) => 2,
        }
    }

    fn exit_lanes(&self) -> Vec<Lane> {
        self.own
            .values()
            .filter(|u| Some(u.def) == self.kit.plant)
            .map(|plant| {
                let (dx, dz) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][plant.facing.rem_euclid(4) as usize];
                Lane { from: plant.pos, to: Vec3 { x: plant.pos.x + dx * 400.0, y: plant.pos.y, z: plant.pos.z + dz * 400.0 }, half_width: 120.0 }
            })
            .collect()
    }

    pub(crate) fn run_factories(&mut self, commands: &mut Vec<Command>) {
        let idle: Vec<UnitId> = self.finished(self.kit.plant).filter(|p| p.idle && self.economy.factory_ordered.get(&p.id).is_none_or(|at| self.frame - at > GRACE)).map(|p| p.id).collect();
        for plant in idle {
            let Some(def) = self.next_unit() else { continue };
            self.economy.factory_ordered.insert(plant, self.frame);
            commands.push(Command::Build { unit: plant, def, site: None, queue: false });
        }
    }

    /// D-PRODUCTION: two scouts and a constructor, two more scouts, then soldiers with a constructor after every few
    /// until the constructors are as many as the clock asks; raiders before `TANKS_FROM`, then five tanks to one artillery.
    fn next_unit(&mut self) -> Option<UnitDefId> {
        let kit = &self.kit;
        let constructors = self.count(kit.constructor);
        let minutes = self.frame as f32 / MINUTE as f32;
        let constructors_wanted = if minutes < 3.0 { 3 } else { ((3.0 + 1.2 * (minutes - 3.0)) as usize).min(14) };
        let between = if constructors < 3 { 2 } else { 4 };
        let economy = &mut self.economy;
        economy.made += 1;
        if kit.constructor.is_some() && (economy.made == 3 || (economy.made > 3 && constructors < constructors_wanted && economy.since_constructor >= between)) {
            economy.since_constructor = 0;
            return kit.constructor;
        }
        economy.since_constructor += 1;
        let scouts_alive = self.own.values().filter(|u| Some(u.def) == kit.scout).count();
        if kit.scout.is_some() && (economy.scouts_made < SCOUTS_FIRST || (self.frame < TANKS_FROM && scouts_alive < 2 && economy.since_scout >= 8)) {
            economy.scouts_made += 1;
            economy.since_scout = 0;
            return kit.scout;
        }
        economy.since_scout += 1;
        if self.frame < TANKS_FROM {
            return kit.raider.or(kit.tank);
        }
        if economy.made % 6 == 5 { kit.artillery.or(kit.tank) } else { kit.tank }.or(kit.raider)
    }
}
