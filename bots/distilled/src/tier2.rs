//! Tier 2 and the projects: the gate that starts the advanced plant, the dear structures several builders raise
//! together, the advanced constructors' jobs (tier-2 extractors, fusion, converters), the advanced plant's production,
//! and the arms of the rows that test them (`README.md`, D-T2-GATE to D-CONVERT). The arms are read from the
//! environment variable `WITHIN_REASON_DISTILLED`, which the engine inherits from the arena (`Options`).
//!
//! The arithmetic the rules rest on is in `docs/knowledge/tier2.md`: K-t2-build-power-for-a-project-follows-the-spend
//! (a project's power is its build time times what we can spend on it over its metal; beyond that it stalls),
//! K-t2-mobile-build-power-by-cost, K-t2-moho-first, K-mech-converter-threshold.

use bot_protocol::{OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId};

use crate::economy::{Job, SPOT_DANGER, TURRET_REACH_SLACK};
use crate::{Distilled, MINUTE};

/// No advanced plant before this, whatever the income.
const T2_FROM: i32 = 6 * MINUTE;
/// The energy store must hold this share when the plant is started: a transition begun in a stall never ends.
const T2_ENERGY_STORE: f32 = 0.3;
/// `gate=bank`: the bank holds the plant's bill when it holds its metal, or, with a store smaller than that, when the
/// store is this full.
const BANK_FULL: f32 = 0.9;
/// The share of the metal income a project may draw, and the seconds over which the bank is spent beside it
/// (D-PROJECT). The players' "stop the other factories during the transition" is the share.
const PROJECT_SHARE: f32 = 0.75;
const BANK_SECONDS: f32 = 90.0;
/// Builders helping one frame at the most, the commander among them (distilled-t2-smoke-4: an advanced fusion at 830
/// power against a budget of ten thousand took seven minutes under a cap of six).
const HELPERS: usize = 12;
/// A fusion from this metal income when energy is short, an advanced fusion from this with a fusion standing; under
/// `energy=fusion` the first fusion comes before any tier-2 extractor, from this income.
const FUSION_FROM: f32 = 35.0;
const ADVANCED_FUSION_FROM: f32 = 120.0;
const FUSION_FIRST_FROM: f32 = 20.0;
/// A tier-2 extractor (20 energy a second of upkeep) is started while the store holds this share, or the income has
/// this much to spare.
const MOHO_ENERGY_STORE: f32 = 0.5;
const MOHO_ENERGY_SPARE: f32 = 25.0;
/// Advanced constructors: two first, then one every so many tier-2 units up to this many.
const ADVANCED_CONSTRUCTORS: usize = 4;
const ADVANCED_CONSTRUCTOR_EVERY: usize = 8;
/// Converters run only above 75% of energy storage (K-mech-converter-threshold), so one is built only while the store
/// is over that level and the surplus would feed it; never while a free spot is left to take.
const CONVERTER_LEVEL: f32 = 0.75;
const CONVERTER_FEED: f32 = 70.0;
const ADVANCED_CONVERTER_FEED: f32 = 600.0;
const CONVERTERS_AT_MOST: usize = 24;
/// `power=ring`: construction turrets within reach of the project's frame, up to this many. `power=swarm`: fast
/// assist units alive while a project is up, up to this many.
const RING: usize = 10;
const SWARM: usize = 12;
/// The tier-1 plants pause through the transition only while a home guard of this much metal stands (D-T2-PAUSE):
/// the first smoke (distilled-t2-smoke-1) tied the pause to "no raid in the last 30 s" and hard_aggressive raids
/// every second, so the pause never held and the plants starved the project anyway.
const GUARD_FLOOR: f32 = 550.0;
/// Every so many tier-2 soldiers, one is artillery.
const ARTILLERY_EVERY: usize = 4;

/// How a project's build power is massed, beyond the builders that help it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Power {
    Helpers,
    Ring,
    Swarm,
}

/// D-T2-GATE's arms: `gate=income` the advanced plant on income alone; `gate=field` and only once the field is won
/// (D-FIELD), the default; `gate=bank` once the field is won or the bank holds the plant's bill; `gate=tech` those or
/// once anything of his of tier 2 has been seen, each waiting while the strike body pushes in his half (2026-10-25:
/// on the 2v2 bench the seats then tech at 14-15 min and the team's income doubles by 30:00, but no arm won more, and
/// in duels `gate=tech` went 3-5-8 against the field gate's 9-3-4, dist42-49) ("past his advanced factory a tier-1 surplus is
/// spent on worse terms", D-lead-convert-it; "decide at his lab", D-tier2-sprint).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Gate {
    Bank,
    Field,
    Income,
    Tech,
}

/// The arms of the rows (`README.md`): `WITHIN_REASON_DISTILLED="t2=22,energy=mohos,converters=on,power=helpers,pause=on"`.
#[derive(Clone, Debug)]
pub(crate) struct Options {
    /// The metal income at which the advanced plant is started; `None` never (`t2=never`).
    pub t2_income: Option<f32>,
    /// `energy=fusion`: the advanced constructors' first job is a fusion; `energy=mohos`: tier-2 extractors first and
    /// energy as it runs short.
    pub fusion_first: bool,
    /// `converters=off`: never a converter.
    pub converters: bool,
    pub power: Power,
    /// `pause=off`: the tier-1 plants keep making soldiers through the transition.
    pub pause_t1: bool,
    /// What opens the advanced plant beside the income (D-T2-GATE).
    pub gate: Gate,
    /// `lane=on`: the control lane under the army's orders (D-LANE); off by default, as wired it did not move the
    /// exchange (apex-v029-distilled-comet-10: 1-14-1 over sixteen cold games against 3-12-1 without it).
    pub lane: bool,
    /// `raid=off`: no raider group (D-RAIDERS, 2026-10-20).
    pub raiders: bool,
    /// `received=off`: what allies' overflow puts into the stock is not counted as income (D-RECEIVED, 2026-10-25).
    pub received: bool,
    /// `air=off`: no aircraft plant, so no fighters and no bombers; the Nettles stay (`air.rs`, 2026-10-25).
    pub air: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { t2_income: Some(22.0), fusion_first: false, converters: true, power: Power::Helpers, pause_t1: true, gate: Gate::Field, lane: false, raiders: true, received: true, air: true }
    }
}

impl Options {
    pub(crate) fn from_env() -> Self {
        let mut options = Options::default();
        let text = std::env::var("WITHIN_REASON_DISTILLED").unwrap_or_default();
        for pair in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match (key.trim(), value.trim()) {
                ("t2", "never") => options.t2_income = None,
                ("t2", n) => options.t2_income = n.parse().ok().or(options.t2_income),
                ("energy", v) => options.fusion_first = v == "fusion",
                ("converters", v) => options.converters = v != "off",
                ("power", "ring") => options.power = Power::Ring,
                ("power", "swarm") => options.power = Power::Swarm,
                ("power", _) => options.power = Power::Helpers,
                ("pause", v) => options.pause_t1 = v != "off",
                ("gate", "field") => options.gate = Gate::Field,
                ("gate", "income") => options.gate = Gate::Income,
                ("gate", "tech") => options.gate = Gate::Tech,
                ("gate", "bank") => options.gate = Gate::Bank,
                ("gate", _) => options.gate = Gate::Field,
                ("lane", v) => options.lane = v == "on",
                ("raid", v) => options.raiders = v != "off",
                ("received", v) => options.received = v != "off",
                ("air", v) => options.air = v != "off",
                _ => eprintln!("[distilled] WITHIN_REASON_DISTILLED: unknown arm {pair:?}"),
            }
        }
        options
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "t2={}, gate={}, energy={}, converters={}, power={:?}, pause={}, lane={}, raid={}, received={}, air={}",
            self.t2_income.map_or("never".to_string(), |n| format!("{n:.0}")),
            match self.gate {
                Gate::Bank => "bank",
                Gate::Field => "field",
                Gate::Income => "income",
                Gate::Tech => "tech",
            },
            if self.fusion_first { "fusion" } else { "mohos" },
            if self.converters { "on" } else { "off" },
            self.power,
            if self.pause_t1 { "on" } else { "off" },
            if self.lane { "on" } else { "off" },
            if self.raiders { "on" } else { "off" },
            if self.received { "on" } else { "off" },
            if self.air { "on" } else { "off" }
        )
    }
}

impl Distilled {
    fn project_defs(&self) -> [Option<UnitDefId>; 3] {
        [self.kit.advanced_plant, self.kit.fusion, self.kit.advanced_fusion]
    }

    /// The current project: the dearest unfinished advanced plant, fusion or advanced fusion of ours.
    pub(crate) fn project_frame(&self) -> Option<&OwnUnit> {
        let defs = self.project_defs();
        let cost = |u: &OwnUnit| self.def(u.def).map_or(0.0, |d| d.metal_cost);
        self.own.values().filter(|u| u.being_built && defs.contains(&Some(u.def))).max_by(|a, b| cost(a).total_cmp(&cost(b)))
    }

    /// A project is under way: its frame stands, or a builder has been told to place one.
    pub(crate) fn project_under_way(&self) -> bool {
        self.project_frame().is_some() || self.economy.working_on(|job| matches!(job, Job::AdvancedPlant | Job::Fusion | Job::AdvancedFusion)) > 0
    }

    /// Build power on a frame: the build speeds of the builders raising or helping it and of the construction turrets
    /// within reach of it.
    pub(crate) fn power_on(&self, frame: &OwnUnit) -> f32 {
        let on_it = |job: Job| matches!(job, Job::Help(target) if target == frame.id) || matches!(job, Job::AdvancedPlant | Job::Fusion | Job::AdvancedFusion);
        let builders: f32 = self.economy.orders.iter().filter(|(_, order)| on_it(order.job)).filter_map(|(id, _)| self.own.get(id)).filter_map(|u| self.def(u.def)).map(|d| d.build_speed).sum();
        let turrets: f32 = self.finished(self.kit.turret).filter_map(|t| self.def(t.def).filter(|d| t.pos.dist2d(frame.pos) <= d.build_distance + TURRET_REACH_SLACK)).map(|d| d.build_speed).sum();
        builders + turrets
    }

    /// D-PROJECT: the power a project can use is its build time times what we can spend on it, over its metal: a
    /// share of the income and the bank spent over a minute and a half (K-t2-build-power-for-a-project-follows-the-spend).
    pub(crate) fn power_wanted(&self, frame: &OwnUnit, tick: &Tick) -> f32 {
        let Some(def) = self.def(frame.def) else { return 0.0 };
        let metal = tick.snapshot.metal;
        let spend = PROJECT_SHARE * metal.income + metal.current / BANK_SECONDS;
        def.build_time * spend / def.metal_cost.max(1.0)
    }

    /// A builder joins the project while the power on it is under what it can use and the helpers are few.
    pub(crate) fn project_help(&self, tick: &Tick) -> Option<Job> {
        let frame = self.project_frame()?;
        let helping = self.economy.working_on(|job| matches!(job, Job::Help(target) if target == frame.id));
        (helping < HELPERS && self.power_on(frame) < self.power_wanted(frame, tick)).then_some(Job::Help(frame.id))
    }

    /// What the project's frame draws in energy: the power on it at full speed, but no more than the metal it can get
    /// (its share of the income) buys, since a frame starved of metal draws no energy either (distilled-t2-smoke-3:
    /// 23 solars and 7 advanced for an income of 30 against a full-speed figure). K-energy-plant-draw-outruns-solars:
    /// the energy rule anticipates the draw instead of answering the stall.
    pub(crate) fn project_draw(&self, metal_income: f32) -> f32 {
        let Some(frame) = self.project_frame() else { return 0.0 };
        let Some(def) = self.def(frame.def) else { return 0.0 };
        let at_full_speed = self.power_on(frame) * def.energy_cost / def.build_time.max(1.0);
        let metal_allows = PROJECT_SHARE * metal_income * def.energy_cost / def.metal_cost.max(1.0);
        at_full_speed.min(metal_allows)
    }

    /// D-T2-GATE: a constructor that can build the advanced plant starts it once the metal income reaches the arm's
    /// figure, the energy store is not empty and no project is under way, and, under `gate=field`, the field is won
    /// (D-FIELD); under `gate=bank` the field is won or the bank holds the plant's bill (its metal, or `BANK_FULL` of a
    /// store smaller than that): "a bank is a plant paid" (D-tier2-sprint), the plant standing on metal nothing else
    /// was spending. The field alone kept the plant from every seat of the user's eight-seat game of 2026-10-25, the
    /// field never won in a team game, while the store sat full at 7,500-12,300 from 24:00 on 130-180 a second. One
    /// plant. The reason it opened, when it does.
    pub(crate) fn t2_gate(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<&'static str> {
        let (Some(from), Some(plant)) = (self.options.t2_income, self.kit.advanced_plant) else { return None };
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        let ready = self.frame >= T2_FROM
            && builder.build_options.contains(&plant)
            && self.count(Some(plant)) == 0
            && !self.project_under_way()
            && metal.income >= from
            && energy.current >= T2_ENERGY_STORE * energy.storage;
        if !ready {
            return None;
        }
        let bill = self.def(plant).map_or(f32::MAX, |d| d.metal_cost);
        // While the strike body pushes in his half the sprint waits: "if the surplus is in his half and his extractor
        // count is falling, the sprint waits for the kill" (D-tier2-sprint). Against BARb hard_aggressive in duels the
        // sighting and the bank opened the gate at 12.8 min with the body out, the plants paused for it, the army at
        // 15:00 was 4,506 against 7,927, and the row went 3-7-6 against the field gate's 9-3-4 (dist47, dist46).
        match self.options.gate {
            Gate::Income => Some("income"),
            _ if self.field_won() => Some("field"),
            Gate::Bank | Gate::Tech if self.pushing_in_his_half() => None,
            Gate::Bank | Gate::Tech if metal.current >= bill.min(BANK_FULL * metal.storage) => Some("bank"),
            Gate::Tech if self.army.his_tier2.is_some() => Some("his tier 2"),
            _ => None,
        }
    }

    /// The transition: the first advanced plant is going up, or stands with its first two constructors still to make
    /// (a later advanced plant, raised on a piling bank, pauses nothing).
    pub(crate) fn transition(&self) -> bool {
        let plant = self.kit.advanced_plant;
        let standing = self.finished(plant).next().is_some();
        let under_way = self.own.values().any(|u| Some(u.def) == plant && u.being_built) || self.economy.working_on(|job| job == Job::AdvancedPlant) > 0;
        (under_way && !standing) || (standing && self.count(self.kit.advanced_constructor) < 2)
    }

    /// D-T2-PAUSE: the tier-1 plants make nothing through the transition while a home guard stands.
    pub(crate) fn plants_paused(&self) -> bool {
        self.options.pause_t1 && self.transition() && self.home_guard_weight() >= GUARD_FLOOR
    }

    /// D-T2-ADVANCED: an advanced constructor's ladder. Under `energy=fusion` the first fusion before anything; energy
    /// when it is short at scale (a fusion from `FUSION_FROM`, an advanced fusion from `ADVANCED_FUSION_FROM`, else an
    /// advanced solar); a tier-2 extractor nearest home while energy allows; the project; a converter at saturation;
    /// else energy for the extractors to come, else help at the advanced plant.
    pub(crate) fn advanced_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let kit = &self.kit;
        let (metal, energy) = (tick.snapshot.metal, tick.snapshot.energy);
        let fusions = self.count(kit.fusion) + self.count(kit.advanced_fusion);
        let fusion_under_way = self.economy.working_on(|job| matches!(job, Job::Fusion | Job::AdvancedFusion)) > 0 || self.own.values().any(|u| u.being_built && (Some(u.def) == kit.fusion || Some(u.def) == kit.advanced_fusion));
        // Only what this builder can build: neither advanced constructor has the advanced solar among its options
        // (apex-v029-distilled-comet-1 game 01: told the same advanced solar 38 times in a minute, the engine
        // dropping each), so below the fusion's income it builds no energy at all.
        let can = |def: Option<UnitDefId>| def.is_some_and(|d| builder.build_options.contains(&d));
        let energy_job = || {
            if fusion_under_way {
                return None;
            }
            if can(kit.advanced_fusion) && metal.income >= ADVANCED_FUSION_FROM && fusions >= 1 {
                return Some(Job::AdvancedFusion);
            }
            if can(kit.fusion) && metal.income >= FUSION_FROM {
                return Some(Job::Fusion);
            }
            can(kit.advanced_solar).then_some(Job::AdvancedSolar)
        };
        if self.options.fusion_first && fusions == 0 && can(kit.fusion) && !fusion_under_way && metal.income >= FUSION_FIRST_FROM {
            return Some(Job::Fusion);
        }
        if self.energy_short(tick)
            && let Some(job) = energy_job()
        {
            return Some(job);
        }
        // The tier-2 extractor is what only an advanced constructor can build, so it comes before helping a frame
        // (distilled-t2-smoke-3: both sat on a fusion for twelve minutes and one extractor was upgraded).
        let energy_for_moho = energy.current >= MOHO_ENERGY_STORE * energy.storage || energy.income - energy.usage >= MOHO_ENERGY_SPARE;
        let moho = self.moho_target(tick);
        if energy_for_moho && let Some(target) = moho {
            return Some(Job::Moho(target));
        }
        if let Some(job) = self.project_help(tick) {
            return Some(job);
        }
        let converters = self.count(kit.converter) + self.count(kit.advanced_converter);
        if self.options.converters && kit.advanced_converter.is_some() && moho.is_none() && energy.current > CONVERTER_LEVEL * energy.storage && energy.income - energy.usage > ADVANCED_CONVERTER_FEED && converters < CONVERTERS_AT_MOST {
            return Some(Job::AdvancedConverter);
        }
        if moho.is_some()
            && let Some(job) = energy_job()
        {
            return Some(job);
        }
        Some(Job::Assist)
    }

    /// D-T2-MOHO: the finished tier-1 extractor of ours nearest home with no tier-2 frame on it, not refused lately,
    /// no armed enemy near it, and not another builder's.
    fn moho_target(&self, tick: &Tick) -> Option<UnitId> {
        let (extractor, moho) = (self.kit.extractor?, self.kit.moho?);
        let armed = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_none_or(|d| d.weapon_count > 0);
        let taken = |id: UnitId| self.economy.orders.values().any(|o| o.job == Job::Moho(id));
        self.finished(Some(extractor))
            .filter(|e| !taken(e.id) && self.economy.moho_refused.get(&e.id).is_none_or(|until| self.frame >= *until))
            .filter(|e| !self.own.values().any(|u| u.def == moho && u.pos.dist2d(e.pos) < 90.0))
            .filter(|e| !tick.snapshot.enemies.iter().any(|x| armed(x.def) && x.pos.dist2d(e.pos) < SPOT_DANGER))
            .min_by(|a, b| a.pos.dist2d(self.home).total_cmp(&b.pos.dist2d(self.home)))
            .map(|e| e.id)
    }

    /// D-CONVERT: with no free spot left to take, surplus energy becomes metal: a tier-1 converter while the store is
    /// over the converters' level and the surplus would feed one more.
    pub(crate) fn saturated_job(&self, tick: &Tick) -> Option<Job> {
        let energy = tick.snapshot.energy;
        let converters = self.count(self.kit.converter) + self.count(self.kit.advanced_converter);
        let wanted = self.options.converters && self.kit.converter.is_some() && energy.current > CONVERTER_LEVEL * energy.storage && energy.income - energy.usage > CONVERTER_FEED && converters < CONVERTERS_AT_MOST;
        (wanted && self.economy.working_on(|job| matches!(job, Job::Converter | Job::AdvancedConverter)) == 0).then_some(Job::Converter)
    }

    /// `power=ring`: construction turrets within reach of the project's frame, up to `RING`, one builder at a time.
    pub(crate) fn ring_turret_job(&self, tick: &Tick) -> Option<Job> {
        if self.options.power != Power::Ring {
            return None;
        }
        let (frame, turret) = (self.project_frame()?, self.kit.turret?);
        let reach = self.def(turret)?.build_distance;
        let around = self.own.values().filter(|u| u.def == turret && u.pos.dist2d(frame.pos) <= reach + TURRET_REACH_SLACK).count();
        // Only while the frame can use more power (distilled-t2-smoke-3: seven turrets and 980 power on a fusion whose
        // metal allowed 450).
        let wanted = self.power_on(frame) < self.power_wanted(frame, tick);
        (wanted && around < RING && self.economy.working_on(|job| job == Job::Turret(frame.id)) == 0).then_some(Job::Turret(frame.id))
    }

    /// D-T2-PRODUCTION: the advanced plant makes two advanced constructors first and one every few units up to four;
    /// under `power=swarm`, fast assist units while a project is up; else its line tank, every fourth unit artillery.
    pub(crate) fn t2_next_unit(&mut self) -> Option<UnitDefId> {
        let advanced = self.count(self.kit.advanced_constructor);
        let swarm = self.options.power == Power::Swarm && self.kit.assist.is_some() && self.project_under_way() && self.count(self.kit.assist) < SWARM;
        let kit = &self.kit;
        let economy = &mut self.economy;
        economy.t2_made += 1;
        if kit.advanced_constructor.is_some() && (advanced < 2 || (advanced < ADVANCED_CONSTRUCTORS && economy.since_advanced_constructor >= ADVANCED_CONSTRUCTOR_EVERY)) {
            economy.since_advanced_constructor = 0;
            return kit.advanced_constructor;
        }
        economy.since_advanced_constructor += 1;
        if swarm {
            return kit.assist;
        }
        if economy.t2_made.is_multiple_of(ARTILLERY_EVERY) { kit.t2_artillery.or(kit.t2_tank) } else { kit.t2_tank.or(kit.t2_artillery) }
    }

    /// A unit of ours begun or finished: the tier-2 milestones go to the engine log, where `run/bot_curves.py` reads
    /// them ("begun" for a project's frame, "standing" for anything of tier 2; a project's duration is the difference).
    pub(crate) fn t2_milestone(&self, unit: UnitId, how: &str) {
        let Some(def) = self.own.get(&unit).map(|u| u.def) else { return };
        let kit = &self.kit;
        let named = [(kit.advanced_plant, "advanced plant"), (kit.advanced_constructor, "advanced constructor"), (kit.moho, "tier-2 extractor"), (kit.fusion, "fusion"), (kit.advanced_fusion, "advanced fusion"), (kit.converter, "converter"), (kit.advanced_converter, "advanced converter"), (kit.assist, "fast assist unit")];
        let Some((_, what)) = named.iter().find(|(d, _)| *d == Some(def)) else { return };
        if how == "begun" && !self.project_defs().contains(&Some(def)) {
            return;
        }
        eprintln!("[distilled ai={}] f={} t2: {what} {how}, {} of them", self.hello.ai_id, self.frame, self.finished(Some(def)).count());
    }
}
