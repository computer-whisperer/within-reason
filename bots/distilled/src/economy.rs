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
/// A spot whose extractor could not be placed is left alone this long; a plant with no room for a turret, this long.
const REFUSED: i32 = 60 * SECOND;
const TURRET_REFUSED: i32 = 2 * MINUTE;
/// The commander builds extractors this far from home and no farther (D-COMMANDER).
const COMMANDER_WALK: f32 = 700.0;
/// An extractor farther from home than this gets a sentry beside it, from the builder that built it (D-SENTRY).
const SENTRY_FROM: f32 = 900.0;
/// A spot with an armed enemy this near is not walked to.
pub(crate) const SPOT_DANGER: f32 = 600.0;
/// A frame counts as within a turret's reach this far beyond its build distance (the footprint's half).
pub(crate) const TURRET_REACH_SLACK: f32 = 100.0;
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
/// Energy income is kept this much over what the soldiers being made draw (D-ENERGY), but nothing is built for the
/// anticipated draw while the store is nearly full (distilled-t2b-30-hard: 40 to 56 solars and the store at a
/// 16,000 cap for an income of 150, the anticipated draw being the tier-2 tank's 13.7 energy a metal at full speed).
const ENERGY_MARGIN: f32 = 1.25;
const ENERGY_FULL: f32 = 0.9;
/// A constructor builds the advanced solar instead of the solar from this metal income.
const ADVANCED_FROM: f32 = 15.0;
const BUILD_POWER_FROM: i32 = 5 * MINUTE / 2;
/// One more builder on build power for every this much banked, up to this many, and every free one when the store
/// is nearly full (distilled-t2-smoke-4: the store at its cap from 26:00 with income overflowing); a second advanced
/// plant (600 power) from this income when the metal piles up.
const BUILD_POWER_BANK_STEP: f32 = 1000.0;
const BUILD_POWER_BUILDERS: usize = 6;
const STORE_NEARLY_FULL: f32 = 0.9;
const SECOND_ADVANCED_PLANT_FROM: f32 = 100.0;
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
    /// A construction turret beside this plant, or within reach of this project's frame (`power=ring`).
    Turret(UnitId),
    Assist,
    /// The projects: dear buildings several builders raise together (D-PROJECT, `tier2.rs`).
    AdvancedPlant,
    Fusion,
    AdvancedFusion,
    /// Help raise this frame.
    Help(UnitId),
    /// A tier-2 extractor over this extractor of ours (D-T2-MOHO).
    Moho(UnitId),
    /// Surplus energy into metal, at saturation (D-CONVERT).
    Converter,
    AdvancedConverter,
}

pub(crate) struct Order {
    pub(crate) job: Job,
    since: i32,
}

pub(crate) struct Economy {
    pub(crate) orders: BTreeMap<UnitId, Order>,
    /// Per metal spot, the frame until which it is left alone.
    refused: Vec<i32>,
    factory_ordered: BTreeMap<UnitId, i32>,
    /// Per construction turret, when it was last told and the project's frame it was told to help, if any.
    turret_told: BTreeMap<UnitId, (i32, Option<UnitId>)>,
    /// What the factories have been told to make: all, the scouts, and soldiers since the last constructor and scout.
    made: usize,
    scouts_made: usize,
    since_constructor: usize,
    since_scout: usize,
    /// Tier 2 (`tier2.rs`): units the advanced plant has been told to make, those since its last advanced
    /// constructor, and extractors whose tier-2 frame found no site, with the frame until which each is left alone.
    pub t2_made: usize,
    pub since_advanced_constructor: usize,
    pub moho_refused: BTreeMap<UnitId, i32>,
    /// Plants (and project frames) beside which a construction turret found no site, with the frame until which each
    /// is left alone (distilled-t2-smoke-4: the ladder asked the same crowded plant for a turret every half second
    /// for twenty minutes and never reached "another plant").
    turret_refused: BTreeMap<UnitId, i32>,
    /// The frame until which no plant is asked for, after a plant found no site (distilled-t2b-30-hard: two plants
    /// and a 9,000 bank at 25:00, the yard asked for a plant every half second).
    plant_refused: i32,
}

impl Economy {
    pub(crate) fn new(spots: usize) -> Self {
        Economy { orders: BTreeMap::new(), refused: vec![0; spots], factory_ordered: BTreeMap::new(), turret_told: BTreeMap::new(), made: 0, scouts_made: 0, since_constructor: 0, since_scout: 0, t2_made: 0, since_advanced_constructor: 0, moho_refused: BTreeMap::new(), turret_refused: BTreeMap::new(), plant_refused: 0 }
    }

    /// Units the factories have been told to make.
    pub(crate) fn made(&self) -> usize {
        self.made
    }

    pub(crate) fn working_on(&self, job: impl Fn(Job) -> bool) -> usize {
        self.orders.values().filter(|o| job(o.job)).count()
    }
}

impl Distilled {
    pub(crate) fn order_failed(&mut self, unit: UnitId, def: Option<UnitDefId>) {
        // The sentry queued behind an extractor found no site: the extractor's order stands.
        if def.is_some() && def == self.kit.sentry {
            return;
        }
        match self.economy.orders.remove(&unit).map(|order| order.job) {
            Some(Job::Extractor(spot)) => self.economy.refused[spot] = self.frame + REFUSED,
            Some(Job::Moho(target)) => {
                self.economy.moho_refused.insert(target, self.frame + REFUSED);
            }
            Some(Job::Turret(beside)) => {
                self.economy.turret_refused.insert(beside, self.frame + TURRET_REFUSED);
            }
            Some(Job::Plant | Job::AdvancedPlant) => self.economy.plant_refused = self.frame + TURRET_REFUSED,
            _ => {}
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

    /// D-OPENING, D-COMMANDER, D-EXPAND, D-ENERGY, D-BUILD-POWER, D-PROJECT, D-T2-GATE, D-CONVERT: what a builder with
    /// nothing to do does next. An advanced constructor has its own ladder (`tier2.rs`, D-T2-ADVANCED).
    fn next_job(&self, unit: &OwnUnit, tick: &Tick) -> Option<Job> {
        let def = self.def(unit.def)?;
        if Some(unit.def) == self.kit.advanced_constructor {
            return self.advanced_job(tick);
        }
        let commander = Some(unit.def) == self.kit.commander;
        let on_energy = self.economy.working_on(|job| matches!(job, Job::Solar | Job::AdvancedSolar));
        let energy = self.energy_short(tick) && on_energy < ENERGY_BUILDERS + self.count(self.kit.constructor) / ENERGY_BUILDERS_PER;
        let assist = || (self.finished(self.kit.plant).next().is_some() || self.finished(self.kit.advanced_plant).next().is_some()).then_some(Job::Assist);
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
            // D-PROJECT: the commander's 300 is the biggest build power we own.
            if let Some(help) = self.project_help(tick) {
                return Some(help);
            }
            return self.free_spot(unit, tick, COMMANDER_WALK).map(Job::Extractor).or_else(assist);
        }
        if energy {
            let advanced = self.kit.advanced_solar.filter(|d| def.build_options.contains(d));
            return Some(if tick.snapshot.metal.income >= ADVANCED_FROM && advanced.is_some() { Job::AdvancedSolar } else { Job::Solar });
        }
        if let Some(help) = self.project_help(tick) {
            return Some(help);
        }
        if self.t2_gate(def, tick) {
            return Some(Job::AdvancedPlant);
        }
        if let Some(job) = self.ring_turret_job(tick) {
            return Some(job);
        }
        // No new plant or turret while a project is going up on a thin bank: its metal is the project's (D-PROJECT).
        // With metal piling up, several builders add build power at once (distilled-t2-smoke-2: 21,000 banked at
        // 40:00 with one build-power job at a time).
        let metal = tick.snapshot.metal;
        let parallel = if metal.current >= STORE_NEARLY_FULL * metal.storage { usize::MAX } else { (1 + (metal.current / BUILD_POWER_BANK_STEP) as usize).min(BUILD_POWER_BUILDERS) };
        if (!self.project_under_way() || metal.current >= 0.25 * metal.storage)
            && self.economy.working_on(|job| matches!(job, Job::Plant | Job::Turret(_) | Job::AdvancedPlant)) < parallel
            && let Some(job) = self.build_power_job(def, tick)
        {
            return Some(job);
        }
        self.free_spot(unit, tick, f32::MAX).map(Job::Extractor).or_else(|| self.saturated_job(tick)).or_else(assist)
    }

    /// The soldier the plants make now.
    fn soldier(&self) -> Option<&UnitDefInfo> {
        let def = if self.frame < TANKS_FROM { self.kit.raider.or(self.kit.tank) } else { self.kit.tank.or(self.kit.raider) };
        def.and_then(|def| self.def(def))
    }

    fn t2_soldier(&self) -> Option<&UnitDefInfo> {
        self.kit.t2_tank.and_then(|def| self.def(def))
    }

    /// Metal a second the plants and their turrets can turn into soldiers, working without a pause: build power
    /// times the soldier's metal over its build time (the engine's own arithmetic), the tier-1 plants on their
    /// soldier and the advanced plants on theirs. Those still being built count; turrets beyond three a plant do not.
    pub(crate) fn can_spend(&self) -> f32 {
        let power = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).map_or(0.0, |d| d.build_speed);
        let rate = |soldier: Option<&UnitDefInfo>| soldier.map_or(0.0, |s| s.metal_cost / s.build_time.max(1.0));
        let (plants, advanced, turrets) = (self.count(self.kit.plant), self.count(self.kit.advanced_plant), self.count(self.kit.turret));
        let tier1 = (plants as f32 * power(self.kit.plant) + turrets.min(TURRETS_A_PLANT_AT_MOST * (plants + advanced)) as f32 * power(self.kit.turret)) * rate(self.soldier());
        let tier2 = advanced as f32 * power(self.kit.advanced_plant) * rate(self.t2_soldier());
        tier1 + tier2
    }

    /// D-ENERGY: the store is low, or it is falling and not full, or the income is under what the soldiers the metal
    /// income pays for and the project's frame would draw together (the tier-2 soldier's energy per metal once an
    /// advanced plant stands).
    pub(crate) fn energy_short(&self, tick: &Tick) -> bool {
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        let soldier = if self.count(self.kit.advanced_plant) > 0 { self.t2_soldier().or_else(|| self.soldier()) } else { self.soldier() };
        let soldiers_draw = soldier.map_or(0.0, |s| self.can_spend().min(metal.income) * s.energy_cost / s.metal_cost.max(1.0));
        energy.current < 0.3 * energy.storage
            || (energy.income < energy.usage && energy.current < 0.6 * energy.storage)
            || (energy.income < ENERGY_MARGIN * (soldiers_draw + self.project_draw(metal.income)) && energy.current < ENERGY_FULL * energy.storage)
    }

    /// D-BUILD-POWER: while the plants cannot spend the soldiers' share of the income, or metal piles up in the
    /// store, a construction turret beside the plant with the fewest (advanced plants among them), another plant when
    /// each has one, and with metal piling up turrets to three a plant first (four times the power a metal of a
    /// plant) and a second advanced plant from `SECOND_ADVANCED_PLANT_FROM` of income.
    fn build_power_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let metal = tick.snapshot.metal;
        let piling = metal.current > 0.5 * metal.storage;
        let short = self.can_spend() < SOLDIERS_SHARE * metal.income || piling;
        if self.frame < BUILD_POWER_FROM || !short {
            return None;
        }
        let plants: Vec<&OwnUnit> = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)).collect();
        // A turret is its nearest plant's, wherever the site search put it.
        let nearest = |at: Vec3| plants.iter().min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id);
        let turrets_at = |plant: &OwnUnit| self.own.values().filter(|u| Some(u.def) == self.kit.turret && nearest(u.pos) == Some(plant.id)).count();
        let room = |plant: &OwnUnit| self.economy.turret_refused.get(&plant.id).is_none_or(|until| self.frame >= *until);
        let fewest = plants.iter().filter(|p| room(p)).map(|p| (turrets_at(p), p.id)).min();
        let turrets_first = if piling { TURRETS_A_PLANT_AT_MOST } else { TURRETS_A_PLANT };
        let yard_room = self.frame >= self.economy.plant_refused;
        let second_advanced = piling && metal.income >= SECOND_ADVANCED_PLANT_FROM && self.kit.advanced_plant.is_some_and(|d| builder.build_options.contains(&d)) && self.count(self.kit.advanced_plant) == 1 && !self.project_under_way();
        match fewest {
            Some((n, plant)) if self.kit.turret.is_some() && n < turrets_first => Some(Job::Turret(plant)),
            _ if second_advanced && yard_room => Some(Job::AdvancedPlant),
            _ if self.count(self.kit.plant) < PLANTS && yard_room => Some(Job::Plant),
            Some((n, plant)) if self.kit.turret.is_some() && n < TURRETS_A_PLANT_AT_MOST => Some(Job::Turret(plant)),
            _ => None,
        }
    }

    /// D-BUILD-POWER, D-PROJECT: a construction turret with the project's frame in its reach helps raise it; otherwise
    /// an idle turret helps the nearest plant in its reach, told again every ten seconds.
    pub(crate) fn run_turrets(&mut self, commands: &mut Vec<Command>) {
        let project = self.project_frame().map(|f| (f.id, f.pos));
        let turrets: Vec<(UnitId, Vec3, f32, bool)> = self.finished(self.kit.turret).map(|t| (t.id, t.pos, self.def(t.def).map_or(TURRET_REACH, |d| d.build_distance), t.idle)).collect();
        for (turret, at, reach, idle) in turrets {
            let told = self.economy.turret_told.get(&turret).copied();
            if let Some((frame, pos)) = project
                && pos.dist2d(at) <= reach + TURRET_REACH_SLACK
            {
                if told.is_none_or(|(_, on)| on != Some(frame)) {
                    self.economy.turret_told.insert(turret, (self.frame, Some(frame)));
                    commands.push(Command::Repair { unit: turret, target: frame, queue: false });
                }
                continue;
            }
            if !idle || told.is_some_and(|(at, on)| on.is_none() && self.frame - at < TURRET_RETELL) {
                continue;
            }
            let plants = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant));
            let Some(plant) = plants.filter(|p| p.pos.dist2d(at) < TURRET_REACH).min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id) else { continue };
            self.economy.turret_told.insert(turret, (self.frame, None));
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
                // The back field, well behind the yard: generators crowding the yard left no site for a plant or a
                // turret (distilled-t2b-30-hard).
                commands.extend(build(def, BuildSite { near: self.toward_enemy(self.home, -450.0), search_radius: 1500.0, min_dist: 3, placements: placements() }, false));
            }
            Job::Turret(target) => {
                let Some(target) = self.own.get(&target) else { return };
                let plant = Some(target.def) == self.kit.plant || Some(target.def) == self.kit.advanced_plant;
                // Behind a plant, away from its front; round a project's frame, inside the turret's reach of it.
                let site = if plant {
                    BuildSite { near: self.toward_enemy(target.pos, -150.0), search_radius: 350.0, min_dist: 1, placements: placements() }
                } else {
                    BuildSite { near: target.pos, search_radius: 300.0, min_dist: 2, placements: placements() }
                };
                commands.extend(build(self.kit.turret, site, false));
            }
            Job::Plant => commands.extend(build(self.kit.plant, BuildSite { near: self.toward_enemy(self.home, 350.0), search_radius: 900.0, min_dist: 8, placements: placements() }, false)),
            Job::AdvancedPlant => commands.extend(build(self.kit.advanced_plant, BuildSite { near: self.toward_enemy(self.home, 350.0), search_radius: 900.0, min_dist: 8, placements: placements() }, false)),
            Job::Fusion | Job::AdvancedFusion => {
                // The back field, with the generators.
                let def = if job == Job::Fusion { self.kit.fusion } else { self.kit.advanced_fusion };
                commands.extend(build(def, BuildSite { near: self.toward_enemy(self.home, -350.0), search_radius: 1500.0, min_dist: 4, placements: placements() }, false));
            }
            Job::Converter | Job::AdvancedConverter => {
                let def = if job == Job::Converter { self.kit.converter } else { self.kit.advanced_converter };
                commands.extend(build(def, BuildSite { near: self.toward_enemy(self.home, -450.0), search_radius: 1500.0, min_dist: 2, placements: placements() }, false));
            }
            Job::Help(target) => commands.push(Command::Repair { unit: unit.id, target, queue: false }),
            Job::Moho(target) => {
                // Over the extractor, where the engine lets the tier-2 one replace it (K-mech-tier-2-extractor-stands-on-ours).
                let Some(extractor) = self.own.get(&target) else { return };
                commands.extend(build(self.kit.moho, BuildSite { near: extractor.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new() }, false));
            }
            Job::Assist => {
                let nearest = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
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
            .filter(|u| Some(u.def) == self.kit.plant || Some(u.def) == self.kit.advanced_plant)
            .map(|plant| {
                let (dx, dz) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][plant.facing.rem_euclid(4) as usize];
                Lane { from: plant.pos, to: Vec3 { x: plant.pos.x + dx * 400.0, y: plant.pos.y, z: plant.pos.z + dz * 400.0 }, half_width: 120.0 }
            })
            .collect()
    }

    pub(crate) fn run_factories(&mut self, commands: &mut Vec<Command>) {
        let idle = |me: &Self, def: Option<UnitDefId>| -> Vec<UnitId> { me.finished(def).filter(|p| p.idle && me.economy.factory_ordered.get(&p.id).is_none_or(|at| me.frame - at > GRACE)).map(|p| p.id).collect() };
        // D-T2-PAUSE: the tier-1 plants wait through the transition.
        if !self.plants_paused() {
            for plant in idle(self, self.kit.plant) {
                let Some(def) = self.next_unit() else { continue };
                self.economy.factory_ordered.insert(plant, self.frame);
                commands.push(Command::Build { unit: plant, def, site: None, queue: false });
            }
        }
        for plant in idle(self, self.kit.advanced_plant) {
            let Some(def) = self.t2_next_unit() else { continue };
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
