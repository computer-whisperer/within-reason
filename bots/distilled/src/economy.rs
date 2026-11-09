//! The economy and production: the scripts the flagship's player wrote in every game it won, as rules over what
//! stands (`README.md`, D-OPENING to D-PRODUCTION). A builder is given a job when it has none; a factory a unit when
//! its queue is empty.

use std::collections::BTreeMap;

use bot_protocol::{BuildSite, Command, FeatureId, Lane, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use layout::{Facts, Ground, Rect};

use crate::{Distilled, Kit, MINUTE, SECOND};

/// Frames a unit is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const GRACE: i32 = 45;
/// D-ORDERS: a build order the engine drops at once (the builder idle again this soon after it) counts as refused,
/// as one the host found no site for does (apex-v029-distilled-comet-1 game 01: an advanced constructor told the
/// same advanced solar 38 times in a minute, the engine dropping each); a builder that has stood still this long on
/// a build order with no frame begun beside it holds an order it cannot carry out (the commander of every +0 game of
/// that battery, given a construction turret it has no build option for, stood at the plant from 2:30 to the end,
/// never idle, so never re-asked), and the order is dropped the same way. Two minutes, not twenty seconds: the
/// engine starts no frame while it first reclaims a wreck on the site or waits for a blocking unit to move off, and
/// drops a blocked build itself after thirty tries (`CBuilder::StartBuild`, `CBuilderCAI::ExecuteBuildCmd`); at
/// twenty seconds the watchdog took 27-65 extractor orders a game from builders in range of their spots
/// (apex-v029-distilled-comet-2, 1-7). A builder refused or dropped this many times in a row helps a plant for a
/// minute instead.
const DROPPED: i32 = 3 * SECOND;
const HELD: i32 = 2 * MINUTE;
/// A builder that has not moved this far from one place is standing still there (the held test reads displacement,
/// not the velocity: a builder jammed against its own buildings jitters at a few elmos a minute and never reads
/// still by its velocity, dist94).
const STILL_MOVE: f32 = 40.0;
/// An order this old with no frame of ours in the builder's reach is dropped whether the builder stands still or not
/// (dist82: the commander walked at one solar order for 744 s, jammed among its own buildings, never still).
const HELD_LONG: i32 = 5 * MINUTE;
const ATTEMPTS: u32 = 4;
/// How often a builder that is assisting is asked whether something better has come up.
const ASSIST_REVIEW: i32 = 10 * SECOND;
/// A spot whose extractor could not be placed is left alone this long; a plant with no room for a turret, this long.
const REFUSED: i32 = 60 * SECOND;
const TURRET_REFUSED: i32 = 2 * MINUTE;
/// A layout slot that came to nothing is passed over by the layer this long (the flagship's `REFUSED_FRAMES`).
const SITE_REFUSED: i32 = 90 * SECOND;
/// The commander builds extractors and the front's radar this far from home and no farther (D-COMMANDER). 700 until
/// 2026-11-03 night: Comet has four spots within 700 of a start and the next at 1,081-1,427, so from 4:00 the
/// commander assisted a metal-starved plant in fifteen of sixteen games (dist95) while the flagship's commander
/// built the extractors 1,000 and 1,353 out at 3:39 and 4:00 (apex-v0504-comet-sym0-opus-1, its constructors on
/// the farther ones). Its solars go where it stands (the layer's band, D-LAYOUT), out there too: the rule that
/// kept its solar field within 700 of home (the flagship's K-eco-generators-built-on-the-frontier) lived in the
/// engine-search siting and has not played since the layer took the solars on 2026-11-03.
const COMMANDER_WALK: f32 = 1400.0;
/// Before the plant stands the commander walks this far from home for an extractor and no farther (2026-11-06). The
/// opening took two extractors first whatever the walk: on Hooked the second spot is 1,000 from the first, the
/// commander walked 75 s to it before any solar, and the plant was begun at 2:38 against 0:37 to 0:54 on eleven other
/// maps; all sixteen games were lost, eleven by 9:30 (dist155, dist167). The far spot is the commander's after the
/// plant (`COMMANDER_WALK`) or a constructor's.
const OPENING_WALK: f32 = 500.0;
/// An extractor farther from home than this gets a sentry beside it, from the builder that built it (D-SENTRY).
const SENTRY_FROM: f32 = 900.0;
/// ... and only at extractors this far along the way to his start, with no sentry of ours within `SENTRY_SHARED`.
const SENTRY_TOWARD: f32 = 0.0;
const SENTRY_SHARED: f32 = 450.0;
/// D-FLEE (2026-10-20): a constructor with an armed mobile ground enemy of his within `FLEE_FROM` that outweighs
/// the soldiers of ours within `FLEE_COVER` of it drops its job and walks `FLEE_WALK` toward home, for `FLEE_FOR`
/// (16-29 builders a game died to Grunt packs with nothing of ours near, dist9-army-apex-v032-comet-cold-1).
const FLEE_FROM: f32 = 500.0;
const FLEE_COVER: f32 = 400.0;
/// ... and only from a threat of this much metal (a lone Grunt sent every builder running 107-150 times a game,
/// dist10-army-apex-v032-comet-cold-1, for no fewer builders lost).
const FLEE_THREAT: f32 = 100.0;
const FLEE_WALK: f32 = 700.0;
const FLEE_FOR: i32 = 8 * SECOND;
/// A spot with an armed enemy this near is not walked to.
pub(crate) const SPOT_DANGER: f32 = 600.0;
/// D-RADAR (2026-11-03, the brief's raid and reach articles: a radar at the front by 3:00, "the second most critical
/// building after a mex", 60 metal for twenty seconds' warning; our games with one by 4:00 lost three fewer
/// extractors by 16:00): from `RADAR_FROM`, a radar `RADAR_BACK` behind our finished extractor nearest his start when
/// our radar does not cover it and no radar of ours stands within `RADAR_SHARED`; one going up at a time, `RADARS_AT_MOST`
/// in all, none at a point with an armed enemy within `SPOT_DANGER`.
const RADAR_FROM: i32 = 2 * MINUTE;
const RADAR_BACK: f32 = 150.0;
const RADAR_SHARED: f32 = 1000.0;
const RADARS_AT_MOST: usize = 6;
/// D-RECLAIM (2026-11-03, the brief's metal article: "the metal on the ground is the part of the quantity we have
/// never collected"; a construction vehicle collects about 38 a second of wreck, the commander 75; the pool's winners
/// run above their extractors' yield in 12 % of their minutes): a wreck worth `WRECK_LEAST` within `RECLAIM_COVER`
/// of a plant, generator or construction turret of ours (the base and its field, not an outpost: taken from any
/// building of ours the constructors went to the hold point's wrecks, fled his raiders there, came back and went
/// again, 20-264 orders a game, and the extractors at 10:00 fell from 26 to 24: dist88 against dist91), with no
/// armed enemy within `SPOT_DANGER`, is taken by a constructor with nothing better within `WRECK_WALK`, ahead of the
/// next extractor when it pays `RECLAIM_RATE`, and by the commander within `COMMANDER_RECLAIM` of it; not into a
/// store nearly full.
const WRECK_LEAST: f32 = 40.0;
const WRECK_WALK: f32 = 1500.0;
const COMMANDER_RECLAIM: f32 = 300.0;
const RECLAIM_COVER: f32 = 1200.0;
/// ... ahead of the next extractor when the wreck pays this much metal a second of the builder's time, the walk and
/// the engine's reclaim time counted (a Blitz wreck 500 away pays a construction vehicle 6 a second, a Stout's 1,000
/// away the same; an extractor's 50 is 20 s of its time for 2.3 a second after).
const RECLAIM_RATE: f32 = 4.0;
/// Builders on energy at one time: three, and one more for every three constructors (2026-10-20: the flagship's
/// twelve Comet wins hold a solar per extractor through 10:00, 18 for 20, where the bot held 13 for 26 and stalled on
/// energy at 9:00 and 12:00).
const ENERGY_BUILDERS: usize = 3;
const ENERGY_BUILDERS_PER: usize = 3;
/// A constructor builds energy ahead of the free spots only while the energy store is under this share (D-ENERGY,
/// 2026-11-04): the commander is the energy engine, the constructors its backstop in a stall.
const ENERGY_STALL: f32 = 0.4;
/// D-ENERGY's ratio (2026-10-20): the energy income is kept at this much a second for every finished extractor, the
/// flagship's 555 for 20 at 10:00 and 1,020 for 29 at 15:00 (28 and 35); the bot ran 18 and 25.
const ENERGY_PER_EXTRACTOR: f32 = 28.0;
const CONSTRUCTOR_ENERGY_PER_EXTRACTOR: f32 = 18.0;
/// The commander builds for the ratio only while the energy store is under this share; above it, it assists the
/// plant between solars, as the flagship's does ("assist 60 low" at 2:51 of barb-hard-1 between its solar bursts).
const COMMANDER_SOLAR_STORE: f32 = 0.7;
/// Plants at the most, and construction turrets a plant: one before another plant is built, three once the plants
/// are all there. A plant's yard has to clear between units, so build power piled on one plant stops paying
/// (distilled-v1-hard game 1: one plant with six turrets, the metal store full for four minutes).
const PLANTS: usize = 8;
const TURRETS_A_PLANT: usize = 1;
const TURRETS_A_PLANT_AT_MOST: usize = 3;
/// Turrets go to three a plant only once this many plants stand: build power piled on one plant does not spend (its
/// yard has to clear), and at +50 % income three turrets on the first plant left the store full through minute 5 while
/// the second plant waited (apex-v022-distilled-comet-sym50-1).
const TURRETS_TO_THREE_FROM_PLANTS: usize = 3;
/// A piling metal store asks for build power only with the energy store over this share: the solars first.
const PILING_ENERGY_STORE: f32 = 0.5;
/// The plants should be able to spend this share of the metal income on soldiers (D-BUILD-POWER); the rest is
/// buildings (2026-10-20: at 0.85 the bot banked 1,800 at 10:00 and 2,800 at 12:00 on an income it could not spend,
/// can_spend 35 and 56 against 59 and 77; the flagship's bank peaks near 1,000 and is spent by 15:00).
const SOLDIERS_SHARE: f32 = 1.0;
/// Energy income is kept this much over what the soldiers being made draw (D-ENERGY), but nothing is built for the
/// anticipated draw while the store is nearly full (distilled-t2b-30-hard: 40 to 56 solars and the store at a
/// 16,000 cap for an income of 150, the anticipated draw being the tier-2 tank's 13.7 energy a metal at full speed).
const ENERGY_MARGIN: f32 = 1.25;
const ENERGY_FULL: f32 = 0.9;
/// A constructor builds the advanced solar instead of the solar from this metal income, with the energy store over
/// half (the flagship's first advanced solar comes at 7:35-18:39, median 12:00, income about 40; its 4,000 energy
/// to build is the store's whole early on).
const ADVANCED_FROM: f32 = 40.0;
const ADVANCED_ENERGY_STORE: f32 = 0.5;
const BUILD_POWER_FROM: i32 = 5 * MINUTE / 2;
/// One more builder on build power for every this much banked, up to this many, and every free one when the store
/// is nearly full (distilled-t2-smoke-4: the store at its cap from 26:00 with income overflowing); a second advanced
/// plant (600 power) from this income when the metal piles up.
const BUILD_POWER_BANK_STEP: f32 = 500.0;
/// The commander raises build power before its home spots while this much metal is banked (its 300 build power
/// stands a nano in a third of a constructor's time).
const COMMANDER_BUILD_POWER_BANK: f32 = 600.0;
const BUILD_POWER_BUILDERS: usize = 6;
const STORE_NEARLY_FULL: f32 = 0.9;
/// With metal piling up, one advanced plant for every this much metal income (the second from 100, the third from
/// 200), and a project under way no bar to one while the bank holds its bill (2026-10-25, the 2v2 bench with
/// `gate=tech`: 187-219 a second at 30:00-35:00 against 156-210 the plants could spend, 5,800-17,800 banked, a
/// second advanced plant in one seat of 32: the fusions the advanced constructors raised were always a project).
const ADVANCED_PLANT_PER_INCOME: f32 = 100.0;
const TURRET_RETELL: i32 = 10 * SECOND;
/// A construction turret is built within this of its plant's middle, inside its own reach of it.
const TURRET_REACH: f32 = 330.0;
/// The factory makes raiders until this frame and tanks with artillery after (D-PRODUCTION).
const TANKS_FROM: i32 = 8 * MINUTE;
/// Four scouts before the first soldier. Two were tried 2026-11-03 night (the guard empty until ~3:00 while BARb's
/// first Grunt reaches our extractors at 2:50; the flagship sends two): the first soldier came at ~2:20 and the
/// guard held three at 3:00, and the rows were worse on both counts, 4-3-9 against Apex and 89 of ours dead before
/// 10:00 against 67 (dist108 against dist106), 5-3-8 and 132 against 114 against BARb (dist109 against dist107):
/// one or two raiders at the hold point meet a Grunt at a home extractor no sooner, and two scouts see less.
const SCOUTS_FIRST: usize = 4;
/// Constructors wanted by the income when that is more than the clock asks: one for every this much metal a second
/// (at +50 % income the clock's count took the same spots at the same pace with the metal piling up).
const CONSTRUCTOR_PER_INCOME: f32 = 6.0;

/// Where an extractor goes: a published metal spot (its index) on a spot map, or a tile of metal on a metal map (the
/// footprint's centre in elmos, chosen from the coverage: `docs/design/2026-11-01-metal-maps.md` §1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub(crate) enum Site {
    Spot(usize),
    Tile(i32, i32),
}

impl Site {
    pub(crate) fn at(self, hello: &bot_protocol::Hello) -> Vec3 {
        match self {
            Site::Spot(i) => hello.metal_spots[i],
            Site::Tile(x, z) => Vec3 { x: x as f32, y: 0.0, z: z as f32 },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Job {
    Extractor(Site),
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
    /// An anti-air turret by the plants while the fighters are short (D-ANTI-AIR, `air.rs`).
    AntiAir,
    /// The aircraft plant (D-AIR-PLANT), and help for it while the fighters are short (D-AIR-POWER).
    AirPlant,
    AirAssist,
    /// A nuclear silo (D-NUKE, `nuke.rs`), a project.
    Silo,
    /// A radar tower at this point (D-RADAR, `radar_site`).
    Radar(Vec3),
    /// Take this wreck apart (D-RECLAIM).
    Reclaim(FeatureId),
    /// The commander's opening as one queue at the stand's slots (D-STAND, `siting.rs` `solve_stand`).
    Stand,
}

impl Job {
    /// The building this job places; `None` for helping.
    pub(crate) fn def(self, kit: &Kit) -> Option<UnitDefId> {
        match self {
            Job::Extractor(_) => kit.extractor,
            Job::Solar => kit.solar,
            Job::AdvancedSolar => kit.advanced_solar,
            Job::Plant => kit.plant,
            Job::Turret(_) => kit.turret,
            Job::AdvancedPlant => kit.advanced_plant,
            Job::Fusion => kit.fusion,
            Job::AdvancedFusion => kit.advanced_fusion,
            Job::Moho(_) => kit.moho,
            Job::Converter => kit.converter,
            Job::AdvancedConverter => kit.advanced_converter,
            Job::AntiAir => kit.aa_turret,
            Job::AirPlant => kit.air_plant,
            Job::Silo => kit.silo,
            Job::Radar(_) => kit.radar,
            Job::Assist | Job::AirAssist | Job::Help(_) | Job::Reclaim(_) | Job::Stand => None,
        }
    }
}

pub(crate) struct Order {
    pub(crate) job: Job,
    /// The slot the layout layer chose for it, and its facing (D-LAYOUT, `siting.rs`): ground taken until the frame begins.
    pub(crate) site: Option<(Vec3, i32)>,
    since: i32,
    /// Where the builder last moved `STILL_MOVE` from, and when: it has stood still since (D-ORDERS; a builder
    /// jittering against its own buildings never reads as still by its velocity, dist94: the commander held one solar
    /// order 270-300 s in three games of sixteen, moving a few elmos a minute).
    anchor: (Vec3, i32),
}

/// What a builder's recorded order says at a pass of `run_builders` (D-ORDERS).
enum Review {
    /// Carrying it out, or helping and not due for a look.
    Busy,
    /// No order, or an assist due for a look.
    Free,
    /// A build order carried out: the builder is idle again after it.
    Done,
    /// A build order the engine dropped at once.
    Dropped,
    /// A build order the builder has stood still on with no frame begun beside it.
    Held,
}

pub(crate) struct Economy {
    pub(crate) orders: BTreeMap<UnitId, Order>,
    /// Per kind, the minute the layer's refusal was last said (D-LAYOUT).
    layout_said: std::cell::RefCell<BTreeMap<UnitDefId, i32>>,
    /// The layer's slots the engine refused or a builder could not carry out, by kind, each with the frame until
    /// which the layer passes it over (D-LAYOUT, `World.refused`): without them the refused slot is the best again
    /// at the next ask.
    pub(crate) sites_refused: Vec<(UnitDefId, Vec3, i32)>,
    /// Per builder, orders refused or dropped in a row, and the frame until which it is sidelined (D-ORDERS).
    attempts: BTreeMap<UnitId, u32>,
    sidelined: BTreeMap<UnitId, i32>,
    /// Per metal spot, the frame until which it is left alone.
    refused: Vec<i32>,
    /// Per builder, the frame until which it is fleeing (D-FLEE).
    fleeing: BTreeMap<UnitId, i32>,
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
    /// The frame until which no silo is started, after its site was refused (D-ORDERS).
    pub silo_refused: i32,
    /// Metal-map tiles whose extractor could not be placed, left alone until the frame.
    pub tiles_refused: BTreeMap<(i32, i32), i32>,
    /// D-RECLAIM: reclaim orders given, and the metal the wrecks held when ordered (the minute line).
    pub reclaims: usize,
    pub reclaim_metal: f32,
    /// D-STAND: the stand's queue has gone to the commander; the opening's rungs place what it refused.
    pub stand_issued: bool,
}

impl Economy {
    pub(crate) fn new(spots: usize) -> Self {
        Economy { orders: BTreeMap::new(), layout_said: Default::default(), sites_refused: Vec::new(), attempts: BTreeMap::new(), sidelined: BTreeMap::new(), refused: vec![0; spots], fleeing: BTreeMap::new(), factory_ordered: BTreeMap::new(), turret_told: BTreeMap::new(), made: 0, scouts_made: 0, since_constructor: 0, since_scout: 0, t2_made: 0, since_advanced_constructor: 0, moho_refused: BTreeMap::new(), turret_refused: BTreeMap::new(), plant_refused: 0, silo_refused: 0, tiles_refused: BTreeMap::new(), reclaims: 0, reclaim_metal: 0.0, stand_issued: false }
    }

    /// An extractor site that came to nothing is left alone until `until`.
    pub(crate) fn refuse(&mut self, site: Site, until: i32) {
        match site {
            Site::Spot(i) => self.refused[i] = until,
            Site::Tile(x, z) => {
                self.tiles_refused.insert((x, z), until);
            }
        }
    }

    pub(crate) fn is_refused(&self, site: Site, frame: i32) -> bool {
        match site {
            Site::Spot(i) => frame < self.refused[i],
            Site::Tile(x, z) => self.tiles_refused.get(&(x, z)).is_some_and(|until| frame < *until),
        }
    }

    /// Units the factories have been told to make.
    pub(crate) fn made(&self) -> usize {
        self.made
    }

    pub(crate) fn working_on(&self, job: impl Fn(Job) -> bool) -> usize {
        self.orders.values().filter(|o| job(o.job)).count()
    }

    /// Frames since a builder's recorded order, if it has one.
    pub(crate) fn order_age(&self, unit: UnitId, frame: i32) -> Option<i32> {
        self.orders.get(&unit).map(|o| frame - o.since)
    }

    /// Frames a builder has stood still on its build order (D-ORDERS' held test: within `STILL_MOVE` of one place).
    pub(crate) fn still_for(&self, unit: UnitId, frame: i32) -> Option<i32> {
        self.orders.get(&unit).map(|o| frame - o.anchor.1)
    }

    /// A builder's recorded job as a word (the jobs line).
    pub(crate) fn job_word(&self, unit: UnitId) -> String {
        if self.fleeing.contains_key(&unit) {
            return "flee".into();
        }
        match self.orders.get(&unit).map(|o| o.job) {
            None => "none".into(),
            Some(Job::Extractor(_)) => "extractor".into(),
            Some(Job::Solar) | Some(Job::AdvancedSolar) => "energy".into(),
            Some(Job::Plant) | Some(Job::AdvancedPlant) => "plant".into(),
            Some(Job::Turret(_)) => "nano".into(),
            Some(Job::Assist) => "assist".into(),
            Some(Job::Help(_)) => "help".into(),
            Some(Job::Fusion) | Some(Job::AdvancedFusion) => "fusion".into(),
            Some(Job::Moho(_)) => "moho".into(),
            Some(Job::Converter) | Some(Job::AdvancedConverter) => "converter".into(),
            Some(Job::AntiAir) => "anti-air".into(),
            Some(Job::AirPlant) => "air-plant".into(),
            Some(Job::AirAssist) => "assist-air".into(),
            Some(Job::Silo) => "silo".into(),
            Some(Job::Radar(_)) => "radar".into(),
            Some(Job::Reclaim(_)) => "reclaim".into(),
            Some(Job::Stand) => "stand".into(),
        }
    }
}

impl Distilled {
    /// A builder's order came to nothing: the host found no site, the engine refused or dropped it, or the builder
    /// held it without carrying it out. Its target is left alone for a while, and the builder sidelined after a run
    /// of them (D-ORDERS).
    pub(crate) fn order_failed(&mut self, unit: UnitId, def: Option<UnitDefId>) {
        // The sentry queued behind an extractor found no site: the extractor's order stands.
        if def.is_some() && def == self.kit.sentry {
            return;
        }
        let Some(order) = self.economy.orders.remove(&unit) else { return };
        // The layer's slot came to nothing: it is passed over for a while, so the next ask finds another.
        if let (Some((at, _)), Some(def)) = (order.site, order.job.def(&self.kit)) {
            let frame = self.frame;
            self.economy.sites_refused.retain(|(_, _, until)| frame < *until);
            self.economy.sites_refused.push((def, at, frame + SITE_REFUSED));
        }
        let attempts = self.economy.attempts.entry(unit).or_default();
        *attempts += 1;
        if *attempts >= ATTEMPTS {
            *attempts = 0;
            self.economy.sidelined.insert(unit, self.frame + REFUSED);
        }
        match order.job {
            Job::Extractor(site) => self.economy.refuse(site, self.frame + REFUSED),
            Job::Moho(target) => {
                self.economy.moho_refused.insert(target, self.frame + REFUSED);
            }
            Job::Turret(beside) => {
                self.economy.turret_refused.insert(beside, self.frame + TURRET_REFUSED);
            }
            Job::Plant | Job::AdvancedPlant => self.economy.plant_refused = self.frame + TURRET_REFUSED,
            Job::AirPlant => self.air.plant_refused = self.frame + TURRET_REFUSED,
            Job::Silo => self.economy.silo_refused = self.frame + TURRET_REFUSED,
            _ => {}
        }
    }

    pub(crate) fn run_builders(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        self.economy.orders.retain(|id, _| self.own.contains_key(id));
        let builders: Vec<OwnUnit> = self.own.values().filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed > 0.0 && !d.build_options.is_empty())).cloned().collect();
        self.economy.fleeing.retain(|id, until| self.own.contains_key(id) && self.frame < *until);
        for unit in builders {
            if self.economy.fleeing.contains_key(&unit.id) {
                continue;
            }
            if let Some(threat) = self.flee_from(&unit, tick) {
                let to = self.toward_point(unit.pos, self.home, FLEE_WALK);
                eprintln!("[distilled ai={}] f={} flee: unit {} at ({:.0},{:.0}) from w{:.0} at ({:.0},{:.0})", self.hello.ai_id, self.frame, unit.id.0, unit.pos.x, unit.pos.z, threat.1, threat.0.x, threat.0.z);
                if let Some(order) = self.economy.orders.remove(&unit.id)
                    && let Job::Extractor(site) = order.job
                {
                    self.economy.refuse(site, self.frame + REFUSED);
                }
                self.economy.fleeing.insert(unit.id, self.frame + FLEE_FOR);
                commands.push(Command::Move { unit: unit.id, to, queue: false });
                continue;
            }
            match self.review(&unit) {
                Review::Busy => continue,
                Review::Free => {}
                Review::Done => {
                    self.economy.attempts.remove(&unit.id);
                }
                verdict @ (Review::Dropped | Review::Held) => {
                    let job = self.economy.orders.get(&unit.id).map(|o| o.job);
                    let nearest = self.own.values().filter(|u| u.being_built).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                    let nearest = nearest.map_or("no frame".to_string(), |u| format!("nearest frame {} at {:.0} ({:.0}, {:.0})", self.def(u.def).map_or("?", |d| d.name.as_str()), u.pos.dist2d(unit.pos), u.pos.x, u.pos.z));
                    eprintln!(
                        "[distilled ai={}] f={} {} order: unit {} {job:?} at ({:.0}, {:.0}) idle={} vel={:.2} reach={:.0}; {nearest}",
                        self.hello.ai_id,
                        self.frame,
                        if matches!(verdict, Review::Dropped) { "dropped" } else { "held" },
                        unit.id.0,
                        unit.pos.x,
                        unit.pos.z,
                        unit.idle,
                        unit.vel.x.hypot(unit.vel.z),
                        self.def(unit.def).map_or(0.0, |d| d.build_distance)
                    );
                    self.order_failed(unit.id, None);
                }
            }
            let was = self.economy.orders.get(&unit.id).map(|o| o.job);
            self.economy.orders.remove(&unit.id);
            let Some(mut job) = self.next_job(&unit, tick) else { continue };
            // No order a builder cannot carry out: the ladder's rungs look at the builder's options, and this is the
            // guard behind them.
            if let Some(def) = job.def(&self.kit)
                && self.def(unit.def).is_some_and(|d| !d.build_options.contains(&def))
            {
                eprintln!("[distilled ai={}] f={} unit {} cannot build {job:?}: helping instead", self.hello.ai_id, self.frame, unit.id.0);
                job = Job::Assist;
            }
            let mut site = None;
            if !(matches!(job, Job::Assist | Job::AirAssist) && was == Some(job) && !unit.idle) {
                site = self.order(&unit, job, tick, commands);
                if job == Job::Stand {
                    self.economy.stand_issued = true;
                }
                if let Job::Reclaim(feature) = job {
                    self.economy.reclaims += 1;
                    self.economy.reclaim_metal += self.wrecks.iter().find(|w| w.id == feature).map_or(0.0, |w| w.metal);
                }
            }
            self.economy.orders.insert(unit.id, Order { job, site, since: self.frame, anchor: (unit.pos, self.frame) });
        }
    }

    /// D-FLEE: the armed mobile ground enemy within `FLEE_FROM` of a constructor (not the commander) that outweighs
    /// our soldiers within `FLEE_COVER` of it, with its weight.
    fn flee_from(&self, unit: &OwnUnit, tick: &Tick) -> Option<(Vec3, f32)> {
        if Some(unit.def) == self.kit.commander {
            return None;
        }
        let threats: Vec<(Vec3, f32)> = tick
            .snapshot
            .enemies
            .iter()
            .filter(|e| !e.being_built && e.pos.dist2d(unit.pos) < FLEE_FROM)
            .filter_map(|e| {
                let d = self.def(e.def?)?;
                (d.speed > 0.0 && d.weapon_count > 0 && d.move_class.is_some()).then_some((e.pos, d.metal_cost))
            })
            .collect();
        let threat: f32 = threats.iter().map(|(_, w)| w).sum();
        if threat < FLEE_THREAT {
            return None;
        }
        let cover: f32 = self.own.values().filter(|u| !u.being_built && u.pos.dist2d(unit.pos) < FLEE_COVER).filter_map(|u| self.def(u.def)).filter(|d| d.weapon_count > 0 && d.build_options.is_empty()).map(|d| if d.speed == 0.0 { 3.0 * d.metal_cost } else { d.metal_cost }).sum();
        (threat > cover).then(|| (threats.iter().min_by(|a, b| a.0.dist2d(unit.pos).total_cmp(&b.0.dist2d(unit.pos))).map(|t| t.0).unwrap_or(unit.pos), threat))
    }

    /// D-ORDERS: what a builder's recorded order says now. A build order is dropped when the builder is idle again
    /// within `DROPPED` of it (the engine's idle flag clears a few frames after an order lands, hence `GRACE`), and held
    /// when the builder has stood still on it for `HELD` with no building of ours under construction within its reach.
    fn review(&mut self, unit: &OwnUnit) -> Review {
        let frame = self.frame;
        let Some(order) = self.economy.orders.get_mut(&unit.id) else { return Review::Free };
        if matches!(order.job, Job::Assist | Job::AirAssist) {
            return if frame - order.since >= ASSIST_REVIEW { Review::Free } else { Review::Busy };
        }
        let age = frame - order.since;
        if order.job.def(&self.kit).is_none() {
            return if unit.idle && age > GRACE { Review::Done } else { Review::Busy };
        }
        if unit.idle {
            // A building that stands at the order's slot was built, however soon: under a farm's turrets a converter
            // is raised inside `DROPPED` (D-TURRETS; dist295: 51 converter orders in eight games read as dropped,
            // each a step toward sidelining its builder).
            let def = order.job.def(&self.kit);
            let built = order.site.is_some_and(|(at, _)| self.own.values().any(|u| Some(u.def) == def && u.pos.dist2d(at) < 1.0));
            return if age <= GRACE {
                Review::Busy
            } else if age <= DROPPED && !built {
                Review::Dropped
            } else {
                Review::Done
            };
        }
        if unit.pos.dist2d(order.anchor.0) > STILL_MOVE {
            order.anchor = (unit.pos, frame);
        }
        let stuck = frame - order.anchor.1 >= HELD || age >= HELD_LONG;
        if stuck && !self.frame_beside(unit) { Review::Held } else { Review::Busy }
    }

    /// A building of ours under construction within the builder's reach: what a builder standing still on a build
    /// order has beside it (a plant's soldiers do not count: the held commander stood at the plant).
    pub(crate) fn frame_beside(&self, unit: &OwnUnit) -> bool {
        let reach = self.def(unit.def).map_or(TURRET_REACH, |d| d.build_distance) + 150.0;
        self.own.values().any(|u| u.being_built && u.pos.dist2d(unit.pos) <= reach && self.def(u.def).is_some_and(|d| d.speed == 0.0))
    }

    /// D-OPENING, D-COMMANDER, D-EXPAND, D-ENERGY, D-BUILD-POWER, D-PROJECT, D-T2-GATE, D-CONVERT: what a builder with
    /// nothing to do does next. An advanced constructor has its own ladder (`tier2.rs`, D-T2-ADVANCED).
    fn next_job(&self, unit: &OwnUnit, tick: &Tick) -> Option<Job> {
        let def = self.def(unit.def)?;
        // D-ORDERS: a builder whose last orders all came to nothing helps a plant for a while.
        if self.economy.sidelined.get(&unit.id).is_some_and(|until| self.frame < *until) {
            return Some(Job::Assist);
        }
        if Some(unit.def) == self.kit.advanced_constructor {
            return self.advanced_job(unit, def, tick);
        }
        let commander = Some(unit.def) == self.kit.commander;
        // D-ENERGY (2026-11-04): the commander is the energy engine and the constructors answer a stall alone (the
        // store under `ENERGY_STALL`, or no commander), three and one per three of them; otherwise their energy comes
        // after the free spots. The constructors used to fill the energy quota first and the commander, finding it
        // filled, assisted the plant: on the econ bench four or five of eight to twelve constructors were on solars
        // from 5:00 to 9:00 and one to four on extractors (dist115), where the flagship's commander built its solars
        // and its constructors claimed forty spots in those minutes (econ-flagship-4: 20 to 60 extractors).
        let on_energy = self.economy.working_on(|job| matches!(job, Job::Solar | Job::AdvancedSolar));
        let energy_room = on_energy < ENERGY_BUILDERS + self.count(self.kit.constructor) / ENERGY_BUILDERS_PER;
        let stall = tick.snapshot.energy.current < ENERGY_STALL * tick.snapshot.energy.storage || self.count(self.kit.commander) == 0;
        let energy = self.energy_short(tick) && energy_room && stall;
        // D-AIR-POWER: while the fighters are short, help goes to the aircraft plant.
        let assist = || {
            if self.air_power_wanted() {
                return Some(Job::AirAssist);
            }
            (self.finished(self.kit.plant).next().is_some() || self.finished(self.kit.advanced_plant).next().is_some()).then_some(Job::Assist)
        };
        if commander {
            // D-STAND: the opening as one queue at the stand's slots; the rungs below place what the stand did not.
            if self.stand.is_some() && !self.economy.stand_issued {
                return Some(Job::Stand);
            }
            // The opening, read off what stands: two extractors, two solars, the plant.
            if self.count(self.kit.extractor) < 2
                && let Some(site) = self.free_site(unit, tick, if self.options.ground { OPENING_WALK } else { f32::MAX })
            {
                return Some(Job::Extractor(site));
            }
            if self.count(self.kit.solar) < 2 {
                return Some(Job::Solar);
            }
            if self.count(self.kit.plant) == 0 {
                return Some(Job::Plant);
            }
            // D-COMMANDER (2026-10-20): the commander is the energy engine, as the flagship's is: a solar whenever the
            // energy income is under the ratio (D-ENERGY's `ENERGY_PER_EXTRACTOR`, the extractors under construction
            // counted) or energy is short by the other tests; its 300 build power stands a solar in nine seconds and
            // the flagship's commander built two to three a minute from 1:30 to 10:00 (barb-hard-1, spending-player-1,
            // apex-v029-comet-sym0-opus-1), where ours averaged one with three-minute gaps of assisting.
            if self.energy_short(tick) || (self.energy_below_ratio(tick) && tick.snapshot.energy.current < COMMANDER_SOLAR_STORE * tick.snapshot.energy.storage) {
                return Some(Job::Solar);
            }
            // D-RADAR: the front's radar when it is within the commander's walk.
            if self.kit.radar.is_some_and(|r| def.build_options.contains(&r))
                && let Some(at) = self.radar_site(tick).filter(|at| at.dist2d(self.home) <= COMMANDER_WALK)
            {
                return Some(Job::Radar(at));
            }
            if self.anti_air_wanted(def) {
                return Some(Job::AntiAir);
            }
            if self.air_plant_wanted(def) {
                return Some(Job::AirPlant);
            }
            // D-PROJECT: the commander's 300 is the biggest build power we own.
            if let Some(help) = self.project_help(tick, false) {
                return Some(help);
            }
            // D-RECLAIM: a wreck beside it, at 75 metal a second.
            if let Some(job) = self.wreck_job(unit, tick, COMMANDER_RECLAIM, WRECK_LEAST, 0.0) {
                return Some(job);
            }
            // With metal banked, the build power first; else the home spots: an extractor compounds, a plant can wait.
            if tick.snapshot.metal.current >= COMMANDER_BUILD_POWER_BANK
                && let Some(job) = self.build_power_wanted(def, tick)
            {
                return Some(job);
            }
            if let Some(site) = self.free_site(unit, tick, COMMANDER_WALK) {
                return Some(Job::Extractor(site));
            }
            // ...then it raises the build power itself when that is wanted, a plant in a third of a constructor's time
            // (D-BUILD-POWER; at +50 % income the constructors were on extractors and energy while the store sat full).
            if let Some(job) = self.build_power_wanted(def, tick) {
                return Some(job);
            }
            return assist();
        }
        let energy_job = || {
            let advanced = self.kit.advanced_solar.filter(|d| def.build_options.contains(d));
            let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
            Some(if metal.income >= ADVANCED_FROM && energy.current >= ADVANCED_ENERGY_STORE * energy.storage && advanced.is_some() { Job::AdvancedSolar } else { Job::Solar })
        };
        if energy {
            return energy_job();
        }
        // D-RADAR: the front's radar before the next extractor ("if not metal starved, prioritize over even mexes").
        if self.kit.radar.is_some_and(|r| def.build_options.contains(&r))
            && let Some(at) = self.radar_site(tick)
        {
            return Some(Job::Radar(at));
        }
        if self.anti_air_wanted(def) {
            return Some(Job::AntiAir);
        }
        if self.air_plant_wanted(def) {
            return Some(Job::AirPlant);
        }
        // D-RECLAIM: a wreck that pays the builder's time well comes before the next extractor (a Stout's 135 is
        // nearly three extractors' price, and its ground is ours now).
        if let Some(job) = self.wreck_job(unit, tick, WRECK_WALK, WRECK_LEAST, RECLAIM_RATE) {
            return Some(job);
        }
        // D-FLOAT: on a floating bank a converter on the spare before a hand on a reactor's frame (dist83: 39
        // constructors helping six reactor frames with the energy store full and 21-43 converters, against 43-59 with
        // the helpers capped); before the bank floats the converter keeps its place after the expansion (dist84: the
        // rung ahead of the extractors put converters up at 5:00 and the income at 5:00 fell from 35 to 23). Never
        // ahead of a free spot (2026-11-04): a bank floats at 6:00-7:00 because the plant is short of constructors,
        // not because the spots are gone, and three constructors of eight were on converters with forty spots free
        // (dist115); a spot is two metal a second for 50, a converter one for 100 and 70 energy.
        let site = self.free_site(unit, tick, f32::MAX);
        if site.is_none()
            && tick.snapshot.metal.current >= crate::tier2::FLOAT
            && let Some(job) = self.saturated_job(tick)
        {
            return Some(job);
        }
        // D-PROJECT: a hand on the project's frame ahead of the spots through the transition (the first advanced plant,
        // a sprint); after it the spots come first (2026-11-04: with a floating bank from 10:00 the reactors' frames
        // took 16-26 of 20-40 constructors at 11:00-12:00 while twenty spots were free, dist116).
        let project_help = self.project_help(tick, false);
        if self.transition()
            && let Some(help) = project_help
        {
            return Some(help);
        }
        if let Some(why) = self.t2_gate(def, tick) {
            let metal = tick.snapshot.metal;
            eprintln!("[distilled ai={}] f={} t2 gate: {why} (bank {:.0} of {:.0}, income {:.1}, field won {})", self.hello.ai_id, self.frame, metal.current, metal.storage, metal.income, u8::from(self.field_won()));
            return Some(Job::AdvancedPlant);
        }
        if let Some(job) = self.ring_turret_job(tick, site.is_none()) {
            return Some(job);
        }
        if let Some(job) = self.build_power_wanted(def, tick) {
            return Some(job);
        }
        // D-FLOAT for the tier-1 constructors: with the spots taken, an advanced solar (75 energy a second for 350
        // metal, 7,950 of build time: forty constructors stand twenty-four a minute) whenever the builders' energy
        // draw is not covered, ahead of a hand on a frame or a converter (2026-11-04: the frames starved behind the
        // converters while 17,000-43,000 metal sat banked, dist120-dist121; before, an advanced solar only once no
        // spare was left for a converter, and 23-30 constructors assisted idle plants, dist82).
        // Paced by the energy (2026-11-04): each one draws fifty a second for its hundred seconds, and sixteen to
        // twenty-three begun at once on an income of 2,900 emptied the store at 13:00 and stalled every frame for two
        // minutes (dist122); one more only while the spare and what the store holds above the stall line, spread
        // over the build, cover its draw.
        let float_energy = || {
            let advanced_def = self.kit.advanced_solar.filter(|d| def.build_options.contains(d))?;
            let advanced = self.def(advanced_def)?;
            let energy = tick.snapshot.energy;
            let seconds = advanced.build_time / def.build_speed.max(1.0);
            let draw = advanced.energy_cost / seconds;
            let buffer = (energy.current - ENERGY_STALL * energy.storage) / seconds;
            let spare = self.build_spare(tick);
            (tick.snapshot.metal.current >= crate::tier2::FLOAT && spare < CONSTRUCTOR_ENERGY_PER_EXTRACTOR * 4.0 && spare + buffer >= draw).then_some(Job::AdvancedSolar)
        };
        let energy_after_spots = || (self.energy_short(tick) && energy_room).then(energy_job).flatten();
        site.map(Job::Extractor).or_else(energy_after_spots).or_else(float_energy).or(project_help).or_else(|| self.wreck_job(unit, tick, WRECK_WALK, WRECK_LEAST, 0.0)).or_else(|| self.saturated_job(tick)).or_else(assist)
    }

    /// D-RADAR: where the next radar goes, if one is wanted: `RADAR_BACK` behind our finished extractor nearest his
    /// start, when our radar (the shim's coverage grid) does not reach that extractor, no radar of ours stands within
    /// `RADAR_SHARED` of it, none is going up, fewer than `RADARS_AT_MOST` stand, and no armed enemy is within
    /// `SPOT_DANGER` of the point. Nothing before `RADAR_FROM`, nothing on the econ bench (`army=off`).
    pub(crate) fn radar_site(&self, tick: &Tick) -> Option<Vec3> {
        if !self.options.army || !self.options.radar || self.frame < RADAR_FROM || self.kit.radar.is_none() {
            return None;
        }
        if self.count(self.kit.radar) >= RADARS_AT_MOST || self.economy.working_on(|job| matches!(job, Job::Radar(_))) > 0 {
            return None;
        }
        let sensors = self.sensors.as_ref()?;
        let frontier = self.finished(self.kit.extractor).map(|u| u.pos).min_by(|a, b| a.dist2d(self.enemy_home).total_cmp(&b.dist2d(self.enemy_home)))?;
        if sensors.radar.at(frontier.x, frontier.z) || self.own.values().any(|u| Some(u.def) == self.kit.radar && u.pos.dist2d(frontier) <= RADAR_SHARED) {
            return None;
        }
        let at = self.toward_point(frontier, self.home, RADAR_BACK);
        let armed = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_none_or(|d| d.weapon_count > 0);
        (!tick.snapshot.enemies.iter().any(|e| armed(e.def) && e.pos.dist2d(at) < SPOT_DANGER)).then_some(at)
    }

    /// D-RECLAIM: the wreck worth this builder's walk: holding `least` metal or more, within `within` of the builder,
    /// within `RECLAIM_COVER` of a building of ours, no armed enemy within `SPOT_DANGER` of it, not another builder's,
    /// paying `rate` metal a second of the builder's time or more (the walk at its speed to its reach, then the
    /// engine's reclaim time, six times the metal over 1.05 times the build power plus 135: F-eco-reclaim); the best
    /// by that rate. None into a store nearly full (the metal lands in one second and the store clamps).
    fn wreck_job(&self, unit: &OwnUnit, tick: &Tick, within: f32, least: f32, rate: f32) -> Option<Job> {
        let metal = tick.snapshot.metal;
        if !self.options.reclaim || self.wrecks.is_empty() || metal.current >= STORE_NEARLY_FULL * metal.storage {
            return None;
        }
        let def = self.def(unit.def)?;
        let (speed, reach, power) = (def.speed.max(1.0), def.build_distance, def.build_speed);
        let armed = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_none_or(|d| d.weapon_count > 0);
        let claimed = |id: FeatureId| self.economy.orders.values().any(|o| o.job == Job::Reclaim(id));
        let base = |def: UnitDefId| Some(def) == self.kit.plant || Some(def) == self.kit.advanced_plant || Some(def) == self.kit.air_plant || Some(def) == self.kit.turret || Some(def) == self.kit.solar || Some(def) == self.kit.advanced_solar || Some(def) == self.kit.fusion || Some(def) == self.kit.advanced_fusion;
        let buildings: Vec<Vec3> = self.own.values().filter(|u| base(u.def)).map(|u| u.pos).collect();
        let pays = |w: &bot_protocol::Wreck| {
            let walk = (w.pos.dist2d(unit.pos) - reach).max(0.0) / speed;
            let take = 6.0 * w.metal / (1.05 * power + 135.0);
            w.metal / (walk + take + 1.0)
        };
        self.wrecks
            .iter()
            .filter(|w| w.metal >= least && w.pos.dist2d(unit.pos) <= within && !claimed(w.id) && pays(w) >= rate)
            .filter(|w| buildings.iter().any(|b| b.dist2d(w.pos) <= RECLAIM_COVER))
            .filter(|w| !tick.snapshot.enemies.iter().any(|e| armed(e.def) && e.pos.dist2d(w.pos) < SPOT_DANGER))
            .max_by(|a, b| pays(a).total_cmp(&pays(b)))
            .map(|w| Job::Reclaim(w.id))
    }

    /// D-BUILD-POWER's job for this builder, when one is wanted and not too many builders are on it already. No new
    /// plant or turret while a project is going up on a thin bank: its metal is the project's (D-PROJECT). With metal
    /// piling up, several builders add build power at once (distilled-t2-smoke-2: 21,000 banked at 40:00 with one
    /// build-power job at a time).
    fn build_power_wanted(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let metal = tick.snapshot.metal;
        let parallel = if metal.current >= STORE_NEARLY_FULL * metal.storage { usize::MAX } else { (1 + (metal.current / BUILD_POWER_BANK_STEP) as usize).min(BUILD_POWER_BUILDERS) };
        if (self.project_under_way() && metal.current < 0.25 * metal.storage)
            || self.economy.working_on(|job| matches!(job, Job::Plant | Job::Turret(_) | Job::AdvancedPlant)) >= parallel
        {
            return None;
        }
        self.build_power_job(builder, tick)
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

    /// Metal a second the builders with a build order draw at full speed (a helper what the frame it helps draws):
    /// what the income pays before the plants see any of it. D-BUILD-POWER's short test is net of it (2026-10-18):
    /// gross, the test fired at 2:30 in every +0 game, and the second plant and turret at 3:00-6:00 cost 820 metal
    /// while the store sat at zero from 4:00 either way, so fewer units were made by 6:00 than with one plant
    /// (apex-v029-distilled-comet-3 against -1: 17-23 against 21-25); the pros' first turret comes at 5:56 and the
    /// second factory at 8:11, the Opus player's at 5:32 and 5:50 on fourteen extractors.
    pub(crate) fn builders_draw(&self) -> f32 {
        self.builders_draw_of(|def| def.metal_cost)
    }

    /// Energy a second the builders with a build order draw at full speed, the same way (D-FLOAT, 2026-11-04: the
    /// converters are sized net of it, else they eat the energy the reactors' frames need: dist121's advanced fusion
    /// at 2,700 power assigned and 180 effective behind 81 converters, the energy usage level with its income).
    pub(crate) fn builders_energy_draw(&self) -> f32 {
        self.builders_draw_of(|def| def.energy_cost)
    }

    fn builders_draw_of(&self, cost: impl Fn(&UnitDefInfo) -> f32) -> f32 {
        self.economy
            .orders
            .iter()
            .filter_map(|(id, order)| {
                let built = match order.job {
                    Job::Help(target) => self.own.get(&target).map(|u| u.def),
                    job => job.def(&self.kit),
                }?;
                let (builder, def) = (self.def(self.own.get(id)?.def)?, self.def(built)?);
                Some(builder.build_speed * cost(def) / def.build_time.max(1.0))
            })
            .sum()
    }

    /// D-ENERGY's ratio: the energy income is under `ENERGY_PER_EXTRACTOR` for every extractor standing or begun,
    /// with the store not full. The commander answers this one (D-COMMANDER); the constructors answer it only at
    /// `CONSTRUCTOR_ENERGY_PER_EXTRACTOR`, far behind, so the expansion keeps them (2026-10-20: with every builder
    /// on the ratio the extractors fell from 22 to 18 at 10:00 and the income with them).
    pub(crate) fn energy_below_ratio(&self, tick: &Tick) -> bool {
        let energy = tick.snapshot.energy;
        let extractors = self.count(self.kit.extractor) as f32;
        energy.income < ENERGY_PER_EXTRACTOR * extractors && energy.current < ENERGY_FULL * energy.storage
    }

    /// D-ENERGY: the store is low, or it is falling and not full, or the income is under what the soldiers the metal
    /// income pays for, the project's frame and the silos (D-NUKE) would draw together (the tier-2 soldier's energy per
    /// metal once an advanced plant stands), or far under the ratio.
    pub(crate) fn energy_short(&self, tick: &Tick) -> bool {
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        let soldier = if self.count(self.kit.advanced_plant) > 0 { self.t2_soldier().or_else(|| self.soldier()) } else { self.soldier() };
        // What the plants spend is the income, or the whole of their build power while the store feeds them (at +50 %
        // income the bank fed two plants beyond the income at 3:00 and the energy ran out with the draw read as the
        // income's; apex-v022-distilled-comet-sym50-2).
        let spend = if metal.current > 0.25 * metal.storage { self.can_spend() } else { self.can_spend().min(metal.income) };
        // The aircraft plant's draw with it (D-AIR-POWER): a fighter is 40 energy a metal; and the silos' stockpiles,
        // one more while the bank holds the next silo's metal (D-NUKE).
        let soldiers_draw = soldier.map_or(0.0, |s| spend * s.energy_cost / s.metal_cost.max(1.0)) + self.air_draw() + self.silo_draw(tick);
        let extractors = self.finished(self.kit.extractor).count() as f32;
        energy.current < 0.3 * energy.storage
            || (energy.income < energy.usage && energy.current < 0.6 * energy.storage)
            || (energy.income < ENERGY_MARGIN * (soldiers_draw + self.project_draw(metal.income)) && energy.current < ENERGY_FULL * energy.storage)
            || (energy.income < CONSTRUCTOR_ENERGY_PER_EXTRACTOR * extractors && energy.current < ENERGY_FULL * energy.storage)
    }

    /// D-BUILD-POWER: while the plants cannot spend the soldiers' share of the income, or metal piles up in the
    /// store, a construction turret beside the plant with the fewest (advanced plants among them), another plant when
    /// each has one, and with metal piling up turrets to three a plant first (four times the power a metal of a
    /// plant) and advanced plants to one per `ADVANCED_PLANT_PER_INCOME` of income.
    fn build_power_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        // Metal piling up asks for build power only while the energy can follow: at +50 % income the second plant
        // and its turret at 2:00-3:00 took the energy store from full to nothing and every builder stalled through
        // minute 4 (apex-v022-distilled-comet-sym50-3); below half an energy store the solars come first (D-ENERGY).
        let piling = metal.current > 0.5 * metal.storage && energy.current > PILING_ENERGY_STORE * energy.storage;
        // The clock gates the income test alone: with the store piling up the rule fires as soon as a plant stands
        // (apex-v022-distilled-comet-sym50-1: at +50 % income the store sat full from 1:30 and nothing was built for
        // it before 2:30).
        let short = self.frame >= BUILD_POWER_FROM && self.can_spend() < SOLDIERS_SHARE * (metal.income - self.builders_draw()).max(0.0);
        if !short && !piling {
            return None;
        }
        // `army=off` (the econ bench): the plants make constructors alone, so two plants with a turret each are the
        // build power; a floating bank goes to converters and reactors (D-FLOAT), not to a pile of turrets (dist76:
        // 60 construction turrets and 4 advanced plants at 20:00 on a bank of 34,000).
        if !self.options.army && self.count(self.kit.plant) >= 2 && self.count(self.kit.turret) >= 2 {
            return None;
        }
        // The aircraft plant counts among the plants, and is one to add power to while the fighters are short
        // (D-AIR-POWER: dist22-airsmoke3-glitters game 01 wanted 30 fighters at 28:30 and had 1-4, its fourteen
        // constructors on turrets by the vehicle plants).
        let plants: Vec<&OwnUnit> = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)).chain(self.finished(self.kit.air_plant)).collect();
        let air_power = self.air_power_wanted();
        // Nothing to add power to before the first plant stands.
        if plants.is_empty() {
            return None;
        }
        // A turret is its nearest plant's, wherever the site search put it.
        let nearest = |at: Vec3| plants.iter().min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id);
        let turrets_at = |plant: &OwnUnit| self.own.values().filter(|u| Some(u.def) == self.kit.turret && nearest(u.pos) == Some(plant.id)).count();
        let room = |plant: &OwnUnit| self.economy.turret_refused.get(&plant.id).is_none_or(|until| self.frame >= *until);
        let fewest = plants.iter().filter(|p| room(p) && (air_power || Some(p.def) != self.kit.air_plant)).map(|p| (turrets_at(p), p.id)).min();
        // Plants before turrets while the plants are few (TURRETS_TO_THREE_FROM_PLANTS).
        // Vehicle plants only: with the aircraft plant counted, two vehicle plants and it stacked turrets three a plant
        // where the third vehicle plant should have come (dist35-airon-barb: 2 plants and 10 turrets at 20:00 against 4
        // and 6 with `air=off`, dist34).
        let factories = plants.iter().filter(|p| Some(p.def) != self.kit.air_plant).count();
        let turrets_first = if piling && factories >= TURRETS_TO_THREE_FROM_PLANTS { TURRETS_A_PLANT_AT_MOST } else { TURRETS_A_PLANT };
        let yard_room = self.frame >= self.economy.plant_refused;
        let advanced = self.count(self.kit.advanced_plant);
        let bill = self.kit.advanced_plant.and_then(|d| self.def(d)).map_or(f32::MAX, |d| d.metal_cost);
        let more_advanced = piling
            && advanced >= 1
            && (advanced as f32) < (metal.income / ADVANCED_PLANT_PER_INCOME).floor()
            && self.kit.advanced_plant.is_some_and(|d| builder.build_options.contains(&d))
            && (!self.project_under_way() || metal.current >= bill);
        let job = match fewest {
            Some((n, plant)) if self.kit.turret.is_some() && n < turrets_first => Some(Job::Turret(plant)),
            _ if more_advanced && yard_room => Some(Job::AdvancedPlant),
            _ if self.count(self.kit.plant) < PLANTS && yard_room => Some(Job::Plant),
            Some((n, plant)) if self.kit.turret.is_some() && n < TURRETS_A_PLANT_AT_MOST => Some(Job::Turret(plant)),
            _ => None,
        };
        // The rung wanted is the job; a builder that cannot build it takes nothing here, and the next constructor to
        // come free takes the rung. The commander has no construction turret among its options: given the first
        // turret job at 2:30 in every +0 game of apex-v029-distilled-comet-1, it held the order to the end and no
        // plant or turret was built by anyone until 1,000 metal was banked (docs/studies/2026-10-18-distilled-apex-losses.md).
        job.filter(|job| job.def(&self.kit).is_some_and(|def| builder.build_options.contains(&def)))
    }

    /// D-PRIORITY: the game's builder-priority flag (a low builder draws only the team's surplus, full speed or nothing;
    /// K-mech-builder-priority-is-a-builder-flag). Every plant and construction turret is low, as every human's are
    /// from the stock widget; the commander is low while its job is to help a plant, as the pros set it in the
    /// opening stall so the constructors' extractors take the metal first (K-econ-commander-low-while-assisting),
    /// and high on a build of its own; constructors stay high. Not low on its solars: the pros' solar is an opening
    /// build with energy in hand, D-COMMANDER's is the answer to a shortage, and low there threw the energy store
    /// (prio-distilled-comet-sym50-1: 488 banked at 10:00 against 2,598, four of eight lost against two). The flag is
    /// read back each tick, so an order goes only when it differs from what is wanted.
    pub(crate) fn run_priorities(&self, commands: &mut Vec<Command>) {
        for unit in self.own.values() {
            let Some(def) = self.def(unit.def) else { continue };
            let low = if def.build_speed > 0.0 && def.speed == 0.0 {
                true
            } else if Some(unit.def) == self.kit.commander {
                self.economy.orders.get(&unit.id).is_some_and(|o| matches!(o.job, Job::Assist | Job::AirAssist))
            } else if def.build_speed > 0.0 && !def.build_options.is_empty() {
                false
            } else {
                continue;
            };
            if unit.priority_low != low {
                commands.push(Command::Priority { unit: unit.id, low });
            }
        }
    }

    /// The ground a building of ours covers where it stands.
    pub(crate) fn rect_of(&self, unit: &OwnUnit) -> Option<layout::Rect> {
        let (x, z) = self.def(unit.def)?.footprint;
        Some(layout::Footprint { x, z }.faced(unit.facing).rect(unit.pos))
    }

    /// Whether a construction turret reaches a frame: its build distance to the frame's footprint, the layout
    /// layer's own test (`layout::Worker`), so the power the layer counted at a site is the power that works there.
    /// Until 2026-11-09 the test was the centres' distance with a hundred of slack, a reactor's half footprint and
    /// more than a small frame's; asked of the projects alone then, of every frame now.
    pub(crate) fn turret_reaches(&self, turret: &OwnUnit, frame: &OwnUnit) -> bool {
        let reach = self.def(turret.def).map_or(TURRET_REACH, |d| d.build_distance);
        self.rect_of(frame).is_some_and(|rect| layout::distance_to(&rect, turret.pos) <= reach)
    }

    /// D-TURRETS: a construction turret works the frame in its reach that ranks first, a construction turret's
    /// frame before any other (build power compounds) and then the dearest, told again when the answer changes;
    /// with no frame in reach an idle turret helps the nearest plant in its reach, told again every ten seconds.
    /// Until 2026-11-09 a turret helped the one dearest project's frame when it was in reach and otherwise a plant:
    /// a turret among the reactors with another reactor, an advanced converter or a turret going up in its reach
    /// and no plant stood idle (the audit of 2026-11-06: the construction turrets idle 37 % of their seconds with
    /// the bank full), and a farm (`block.rs`) is worth what its turrets work.
    pub(crate) fn run_turrets(&mut self, commands: &mut Vec<Command>) {
        let frames: Vec<(&OwnUnit, layout::Rect, bool, f32)> = self
            .own
            .values()
            .filter(|u| u.being_built)
            .filter_map(|u| {
                let d = self.def(u.def).filter(|d| d.speed == 0.0)?;
                Some((u, self.rect_of(u)?, Some(u.def) == self.kit.turret, d.metal_cost))
            })
            .collect();
        let work: Vec<(UnitId, Vec3, bool, Option<UnitId>)> = self
            .finished(self.kit.turret)
            .map(|t| {
                let reach = self.def(t.def).map_or(TURRET_REACH, |d| d.build_distance);
                let best = frames
                    .iter()
                    .filter(|(_, rect, _, _)| layout::distance_to(rect, t.pos) <= reach)
                    .max_by(|a, b| a.2.cmp(&b.2).then(a.3.total_cmp(&b.3)).then(b.0.id.0.cmp(&a.0.id.0)))
                    .map(|(frame, ..)| frame.id);
                (t.id, t.pos, t.idle, best)
            })
            .collect();
        for (turret, at, idle, best) in work {
            let told = self.economy.turret_told.get(&turret).copied();
            if let Some(frame) = best {
                if told.is_none_or(|(_, on)| on != Some(frame)) {
                    self.economy.turret_told.insert(turret, (self.frame, Some(frame)));
                    commands.push(Command::Repair { unit: turret, target: frame, queue: false });
                }
                continue;
            }
            if !idle || told.is_some_and(|(at, on)| on.is_none() && self.frame - at < TURRET_RETELL) {
                continue;
            }
            // The aircraft plant only for a turret with no vehicle plant in reach: one built beside it (D-AIR-POWER).
            let nearest = |plants: &mut dyn Iterator<Item = &OwnUnit>| plants.filter(|p| p.pos.dist2d(at) < TURRET_REACH).min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|p| p.id);
            let vehicle = nearest(&mut self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)));
            let Some(plant) = vehicle.or_else(|| nearest(&mut self.finished(self.kit.air_plant))) else { continue };
            self.economy.turret_told.insert(turret, (self.frame, None));
            commands.push(Command::Guard { unit: turret, target: plant });
        }
    }

    /// D-EXPAND: where this builder's next extractor goes within `reach` of home, or nowhere. On a spot map the free
    /// spot that is nearest, counting the walk from the builder whole and the distance from home half; on a metal map
    /// the tile of the most unclaimed metal nearest the builder (`layout::Coverage::best_site`): the ground stands,
    /// nothing of ours or an ally's covers it, no extractor is ordered there, it is out of the plants' exit lanes, no
    /// building of his within 200 and no armed enemy within `SPOT_DANGER`. Touching extractors share nothing on
    /// today's metal maps (K-mech-a-metal-map-publishes-no-spots), so the carpet grows outward from the builder.
    fn free_site(&self, unit: &OwnUnit, tick: &Tick, reach: f32) -> Option<Site> {
        let extractor = |def: UnitDefId| self.def(def).is_some_and(|d| d.extracts_metal > 0.0);
        let armed = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_none_or(|d| d.weapon_count > 0);
        if let Some(coverage) = &self.coverage {
            let def = self.def(self.kit.extractor?)?;
            let facts = Facts::of(def);
            let ground = Ground::new(&self.hello.terrain, self.hello.map.width, self.hello.map.height);
            let lanes = self.exit_lanes();
            let rect_of = |def: UnitDefId, pos: Vec3| self.def(def).filter(|d| d.speed <= 0.0).map(|d| Facts::of(d).footprint.rect(pos));
            let mut taken: Vec<Rect> = self.own.values().filter_map(|u| rect_of(u.def, u.pos)).collect();
            taken.extend(tick.snapshot.allies.iter().filter_map(|u| rect_of(u.def, u.pos)));
            taken.extend(self.economy.orders.values().filter_map(|o| match o.job {
                Job::Extractor(site @ Site::Tile(..)) => Some(facts.footprint.rect(site.at(&self.hello))),
                _ => None,
            }));
            let ok = |p: Vec3, rect: &Rect| {
                !self.economy.is_refused(Site::Tile(p.x as i32, p.z as i32), self.frame)
                    && ground.stands(facts.footprint, p, &facts)
                    && !taken.iter().any(|r| r.overlaps(rect))
                    && !lanes.iter().any(|l| l.contains(p))
                    && !self.army.his_buildings.values().any(|b| b.pos.dist2d(p) < 200.0)
                    && !tick.snapshot.enemies.iter().any(|e| armed(e.def) && e.pos.dist2d(p) < SPOT_DANGER)
            };
            return coverage.best_site(facts.footprint, unit.pos, self.home, reach, def.extracts_metal, &ok).map(|(p, _)| Site::Tile(p.x as i32, p.z as i32));
        }
        // D-GROUND: a spot is this builder's only when its kind can walk there from home, and the nearest is the
        // nearest on foot (from where it stands over the coarse ground, twice the straight line where that has no
        // answer; from home at half weight as before).
        let from = self.field_from(unit.def, unit.pos);
        let walk = |at: Vec3| from.as_ref().map_or(unit.pos.dist2d(at), |field| field.distance(at).unwrap_or(2.0 * unit.pos.dist2d(at)));
        let home_walk = |at: Vec3| self.walk_from_home(Some(unit.def), at);
        let free = |i: &usize| {
            let at = self.hello.metal_spots[*i];
            !self.economy.is_refused(Site::Spot(*i), self.frame)
                && home_walk(at).is_some_and(|w| w <= reach)
                && !self.economy.orders.values().any(|o| o.job == Job::Extractor(Site::Spot(*i)))
                && !self.own.values().any(|u| extractor(u.def) && u.pos.dist2d(at) < 90.0)
                && !tick.snapshot.allies.iter().any(|u| extractor(u.def) && u.pos.dist2d(at) < 90.0)
                && !self.army.his_buildings.values().any(|b| b.pos.dist2d(at) < 200.0)
                && !tick.snapshot.enemies.iter().any(|e| armed(e.def) && e.pos.dist2d(at) < SPOT_DANGER)
        };
        let cost = |i: &usize| walk(self.hello.metal_spots[*i]) + 0.5 * home_walk(self.hello.metal_spots[*i]).unwrap_or(f32::MAX);
        (0..self.hello.metal_spots.len()).filter(free).min_by(|a, b| cost(a).total_cmp(&cost(b))).map(Site::Spot)
    }

    /// The job's order to the engine. A building is sited by the layout layer within its kind's band (D-LAYOUT,
    /// `siting.rs`) and the slot is returned: ground taken until the frame begins. When the layer has no ground the
    /// engine searches round the band's centre off the plants' lanes (`fallback_site`). An extractor goes to its
    /// spot or tile, a tier-2 one over ours.
    fn order(&self, unit: &OwnUnit, job: Job, tick: &Tick, commands: &mut Vec<Command>) -> Option<(Vec3, i32)> {
        let build = |def: Option<UnitDefId>, site: BuildSite, queue: bool| def.map(|def| Command::Build { unit: unit.id, def, site: Some(site), queue });
        if let Job::Radar(at) = job {
            eprintln!("[distilled ai={}] f={} radar: unit {} at ({:.0},{:.0}), {:.0} from home", self.hello.ai_id, self.frame, unit.id.0, at.x, at.z, at.dist2d(self.home));
        }
        if let Some((def, centre, _)) = self.band_for(job, unit) {
            let began = std::time::Instant::now();
            let world = self.layout_world(tick, Some(unit));
            let chosen = self.layout_site(&world, job, unit);
            let ms = began.elapsed().as_secs_f32() * 1000.0;
            match chosen {
                Ok(site) => {
                    // The layer's words for what the farm and the yard are made of: a frame's build power in reach,
                    // a turret's ground, a plant's turrets and facing.
                    if !matches!(self.purpose_of(def), layout::Purpose::Generator { .. } | layout::Purpose::Place) {
                        eprintln!("[distilled ai={}] f={} site: {} for unit {} at ({:.0},{:.0}), {:.0} from the band's centre: {}; {ms:.0} ms", self.hello.ai_id, self.frame, self.name_of(def), unit.id.0, site.at.x, site.at.z, site.at.dist2d(centre), site.words);
                    }
                    commands.extend(build(Some(def), Self::slot_site(site.at, site.facing), false));
                    return Some((site.at, site.facing));
                }
                // Said once a minute per kind: the layer is asked again at every order of the kind (dist77: 382 lines
                // for one turret site in a game).
                Err(why) => {
                    if self.economy.layout_said.borrow_mut().insert(def, self.frame / MINUTE) != Some(self.frame / MINUTE) {
                        eprintln!("[distilled ai={}] f={} no layout site for {job:?} for unit {}: {why}; the engine's search; {ms:.0} ms", self.hello.ai_id, self.frame, unit.id.0);
                    }
                    commands.extend(build(Some(def), self.fallback_site(def, centre, tick), false));
                    return None;
                }
            }
        }
        match job {
            Job::Extractor(site) => {
                let at = site.at(&self.hello);
                // A tile was chosen over the coverage and the ground: the shim searches nowhere for it.
                let exact = matches!(site, Site::Tile(..));
                commands.extend(build(self.kit.extractor, BuildSite { near: at, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact }, false));
                // D-SENTRY: an outer extractor on his side of our ground gets a sentry on its enemy side, queued behind
                // it, unless one stands within `SENTRY_SHARED` already (2026-10-20: nineteen sentries by 10:00 against
                // Apex cost 12,900 energy, more than the thirteen solars of the same minutes made).
                let toward = ((at.x - self.home.x) * (self.enemy_home.x - self.home.x) + (at.z - self.home.z) * (self.enemy_home.z - self.home.z)) / self.home.dist2d(self.enemy_home).max(1.0).powi(2);
                let shared = self.own.values().any(|u| Some(u.def) == self.kit.sentry && u.pos.dist2d(at) < SENTRY_SHARED);
                if self.options.army && self.options.sentry && at.dist2d(self.home) > SENTRY_FROM && toward >= SENTRY_TOWARD && !shared {
                    commands.extend(build(self.kit.sentry, BuildSite { near: self.toward_enemy(at, 120.0), search_radius: 200.0, min_dist: 1, placements: Vec::new(), exact: false }, true));
                }
            }
            // D-STAND: every slot of the stand as one queue, the first plain, the rest behind it; the buildings at
            // their slots exactly, the extractors at the stand's site on their spot (the shim asks the engine for an
            // extractor's point as it is).
            Job::Stand => {
                let Some(stand) = &self.stand else { return None };
                let mut queued = false;
                for slot in &stand.slots {
                    let (def, site) = match *slot {
                        // An extractor beyond the opening's walk is not the opening's (`OPENING_WALK`).
                        layout::Slot::Extractor { at, .. } if self.options.ground && self.walk_from_home(Some(unit.def), at).is_none_or(|w| w > OPENING_WALK) => continue,
                        layout::Slot::Extractor { at, .. } => (self.kit.extractor, BuildSite { near: at, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: true }),
                        layout::Slot::Building { def, at, facing } => (Some(def), Self::slot_site(at, facing)),
                    };
                    commands.extend(build(def, site, queued));
                    queued = true;
                }
            }
            Job::Reclaim(feature) => {
                if let Some(w) = self.wrecks.iter().find(|w| w.id == feature) {
                    eprintln!("[distilled ai={}] f={} reclaim: unit {} wreck {} of {:.0} metal at ({:.0},{:.0}), {:.0} away", self.hello.ai_id, self.frame, unit.id.0, feature.0, w.metal, w.pos.x, w.pos.z, w.pos.dist2d(unit.pos));
                }
                commands.push(Command::ReclaimFeature { unit: unit.id, feature, queue: false });
            }
            Job::AirAssist => {
                let nearest = self.finished(self.kit.air_plant).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                commands.extend(nearest.map(|plant| Command::Guard { unit: unit.id, target: plant.id }));
            }
            Job::Help(target) => commands.push(Command::Repair { unit: unit.id, target, queue: false }),
            Job::Moho(target) => {
                // Over the extractor, where the engine lets the tier-2 one replace it (K-mech-tier-2-extractor-stands-on-ours).
                let Some(extractor) = self.own.get(&target) else { return None };
                commands.extend(build(self.kit.moho, BuildSite { near: extractor.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false }, false));
            }
            Job::Assist => {
                let nearest = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                commands.extend(nearest.map(|plant| Command::Guard { unit: unit.id, target: plant.id }));
            }
            // A building the layer sites: ordered above, or a kind the kit lacks, which is no order.
            Job::Solar | Job::AdvancedSolar | Job::Plant | Job::Turret(_) | Job::AdvancedPlant | Job::Fusion | Job::AdvancedFusion | Job::Converter | Job::AdvancedConverter | Job::AntiAir | Job::AirPlant | Job::Silo | Job::Radar(_) => {}
        }
        None
    }

    /// The exit lanes of our vehicle plants: 400 out from each, 120 a side (the metal map's tiles keep off them,
    /// `free_site`).
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

    pub(crate) fn run_factories(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
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
        // D-FIGHTERS, D-BOMB: the aircraft plant.
        for plant in idle(self, self.kit.air_plant) {
            let Some(def) = self.next_aircraft(tick.snapshot.energy) else { continue };
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
        let by_clock = if minutes < 3.0 { 3 } else { (3.0 + 1.2 * (minutes - 3.0)) as usize };
        let by_income = (self.metal_income / CONSTRUCTOR_PER_INCOME) as usize;
        let constructors_wanted = by_clock.max(by_income).min(if self.options.army { 14 } else { crate::tier2::ECON_CONSTRUCTORS });
        // `army=off` (the econ bench): constructors alone.
        if !self.options.army {
            return (constructors < constructors_wanted).then_some(kit.constructor).flatten();
        }
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
        // D-RAIDERS: a raider in five after the tanks begin, so the group can be refilled.
        if self.options.raiders > 0 && economy.made % 5 == 2 && kit.raider.is_some() {
            return kit.raider;
        }
        if economy.made % 6 == 5 { kit.artillery.or(kit.tank) } else { kit.tank }.or(kit.raider)
    }
}
