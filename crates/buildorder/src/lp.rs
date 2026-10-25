//! The planner as a linear program (docs/design/2026-10-20-territory-lp.md): how much build power of each builder
//! type goes to each column in each ten-second bucket to the game's end, so that the most metal is made. Solved
//! globally by an interior-point method (clarabel), so no chain of prerequisites is a valley: the tier-2 branch the
//! anneal missed in half of terr5-all-nullai-comet-1's games is a flow like any other here.
//!
//! A column is a unit type and a class: the whole units of it started from nothing, the frames of it under way at
//! the start, or (for extractors) the spots of one distance ring from home. The relaxation: counts are fractional,
//! walks are an overhead per builder and unit type plus the ring's distance at the builder's speed, geometry is the
//! hands', converters are bounded by capacity alone (the engine's three-quarter threshold costs one transient), and
//! stocks never go negative (the plan does not stall).

use std::time::Instant;

use clarabel::algebra::CscMatrix;
use clarabel::solver::{DefaultSettings, DefaultSolver, IPSolver, SolverStatus, SupportedConeT};

use crate::sim::{Scenario, State, Wind};
use crate::units::{Role, Units};

/// Metal a reclaim costs the objective per unit taken down: nothing in the game, a tie-breaker here, so that the
/// solver takes a building down only for a gain and never in the last bucket's indifference (without it the
/// interior point reclaimed thirty-six solars, twelve converters and six fusions in minute 19, where nothing
/// counts any more, and a tenth of a solar in every minute before).
const RECLAIM_TIE: f64 = 1.0;

#[derive(Clone, Debug)]
pub struct LpOptions {
    /// Seconds a bucket near the present; buckets grow to `mid_dt` after `mid_from` seconds and to `coarse_dt`
    /// after `coarse_from`: a solve's cost grows with the square of the bucket count (120 ten-second buckets took
    /// 30 s, 60 took 7 s), and the tail of a rolling plan is re-planned before it is reached.
    pub dt: f64,
    pub mid_from: f64,
    pub mid_dt: f64,
    pub coarse_from: f64,
    pub coarse_dt: f64,
    /// Seconds from the state's clock to the end.
    pub horizon: f64,
    /// The hands' capacity: constructors of each tier, and factories, at most (the first solves ran to 420
    /// constructors and 37 factories by 20:00, an exponential the engine's base cannot hold). By tier, so that a
    /// cap the hands fill with the basic kind before an advanced plant stands leaves the plan room for the tier-2
    /// constructors the upgrades need (terr16-lp-nullai-comet-1: forty of the lab's kind by 10:00, no upgrade all
    /// game).
    pub max_constructors: f64,
    pub max_factories: f64,
    /// Seconds of the final income added to the metal made.
    pub terminal: f64,
    /// The share of the energy storage the plan keeps in stock from its second bucket on (the first holds what
    /// stands): a plan that spends energy to the edge is a plan the hands' whole units cannot follow without a
    /// stall (terr25-lp-nullai-comet-1: energy at zero from 12:00 to 15:00 in three games of four).
    pub energy_reserve: f64,
    /// Energy production a bucket must exceed the build draw by, as a share of the draw, from the second bucket
    /// (converters take what is left): the hands start whole units where the plan runs fractions, and a plan whose
    /// production is exactly its draw is one their overshoot stalls, or one the energy budget refuses at (terr40
    /// game 01: the energy at its cap at 12:00 with the income the usage, and the plant's constructors refused
    /// twenty times in the minute). The margin is the room the overshoot lives in.
    pub energy_margin: f64,
    /// The game second the margin applies from (the bucket starting at or after it): the margin is for the mid-game's
    /// stalls, and in the opening it priced the plan's first extractors against solars the commander had no time
    /// for (the program's own 5:00 is 50 without it and 40 with it from the first bucket).
    pub margin_from: f64,
    /// No factory begun before this game second: the pros' opening puts three extractors before the plant (K-open-
    /// comet-pro-order; thebluegecko's at 1:01), where the program's fractional lab produces fractional constructors
    /// from its first bucket and so opens lab-first. 0 leaves the program free.
    pub factory_from: f64,
    /// Build power mobile builders may put on factory products, as a multiple of the factories' own (the yard
    /// clears between units).
    pub assist_factor: f64,
    /// The plants' rings: per factory type standing or committed, the construction turrets its plants' rings hold,
    /// the turrets standing or framed within the ring plus the free ring points a turret could take, as the hands
    /// count them every plan (T-PLANT): the bound on the turrets' assist on that type's products, a turret's power
    /// times the count. Empty: no turret assist pair. thebluegecko's ramp (against InactiveAI, 2026-10-21): 29
    /// turrets within 400 of his advanced vehicle plant by 10:00, 9.7 times its 600, and an advanced constructor
    /// finished every 6-8 s from 9:25 (16,000 build time at 6,400: 2.5 s of work and the pad's roll-off), fifteen by
    /// 10:15. Before terr96 the bound was a multiple of the plants' own power (`nano_assist`; a turret in the model
    /// is power with no reach to run out of, so the first solves bought 285-441 of them, and under the multiple the
    /// program asked for one or two turrets' worth at 10:00, terr67-69; `docs/design/2026-10-25-territory-plant-ramp.md`).
    pub plant_rings: Vec<(usize, f64)>,
    /// Build power construction turrets may put on a column in a bucket, as a multiple of what mobile builders
    /// put on it directly: a turret helps frames in its reach and starts none, so without this bound the program
    /// buys turrets as free build power and runs a converter spiral to 4,000 metal a second (the first solve on
    /// terr5-all's seed 1: 441 turrets, 2,900 converters).
    pub help_factor: f64,
    /// Build power a mobile builder may put on the frames of an extractor upgrade it cannot start (the basic
    /// constructor on the moho's), per column and bucket, as a multiple of the mobile starters' direct power on it; 0 is no such
    /// pair. Without it every upgrade is priced on the advanced constructors' power alone and the program buys the
    /// constructors that implies: 29-32 by 15:00 against thebluegecko's 15 (terr97). His basic constructors put
    /// 26 % of the builder-seconds on his moho frames, his advanced 73 % (his record of 2026-10-21, the per-second
    /// builder targets): 0.35 of the starters' power. Extractors only: the farm's frames are the turrets' (row 3b),
    /// and a basic constructor's share there would be a walk for a hundredth of the work (terr59's help flood).
    pub mobile_help: f64,
    /// Construction turrets at most, per factory standing: a turret's reach holds a few frames round a factory and
    /// no more (the first solves bought 285-441 turrets).
    pub nanos_per_factory: f64,
    /// Construction turrets at most, as a number, in place of the bound per factory when above zero: a farm
    /// (thebluegecko against InactiveAI, 2026-10-21: 311 turrets in a block south of his base, his advanced fusions
    /// and converters placed inside their reach, 39 and 28 turrets within 300 of a frame at its placing). The
    /// farm's size is the base's ground, which the program cannot see; the number is the arm.
    pub max_nanos: f64,
    /// Under the farm (`max_nanos` above zero) and when above zero itself, a turret helps only a turret or a building
    /// of this much build time or more: the farm's turrets reach the core's frames (fusions, advanced fusions,
    /// advanced converters, the turrets beside them) and nothing beside the builders or at the spots. Without it the
    /// program prices a basic converter at the placer's forty-first share, four seconds, where the hands pay 27 s
    /// and a walk for every one, and floods them (terr57: 257-310 a game). Off by default: at 20,000 with the
    /// turrets themselves excluded, the program bought more constructors from the first minutes for the dearer
    /// placing (13-14 at 5:00 against 10-12, the extractors 13-16 against 17-22) and fewer turrets, and sent the
    /// constructors to help the core's frames in their place; terr58 fell at every mark (5:00 32-39 against 36-46,
    /// 20:00 796-892 against 818-912).
    pub farm_big: f64,
    /// The metal in the bank at the horizon counts in the objective as metal made. Off by default: without it
    /// spending is free at the end and the program dumps the bank into whatever is quickest (terr61 game 00's plan
    /// from 15:00: 423 solars and 435 basic converters in its last two minutes), but with it the program hoards
    /// instead, since metal kept reads as income while the marks are income (terr64: turrets 22-39 against 58-92,
    /// fusions 7-11 against 16-28, 43,000-76,000 metal kept, 20:00 846-876 against terr62's 881-980).
    pub bank: bool,
    /// Whether the program may take a standing building down for its metal. The engine returns a reclaimed unit's
    /// metal whole in the second the reclaim ends and none of its energy, for the unit's build time at the
    /// reclaimer's power (his advanced plant, 28,000, in about seven seconds under the turrets round it). Off for
    /// hands that give no reclaim order: a plan that counts on 2,600 back at 10:22 is wrong by that much when
    /// nothing takes the plant down.
    pub reclaim: bool,
    /// Seconds a constructor loses per unit it builds, by kind: an extractor, its upgrade, any other building, a
    /// factory; and the commander's own three. From the territory bot's `begun:` lines (the order's age at its
    /// frame): with one ring the extractor's is the whole walk, a median 56-66 s over every batch to terr69, when
    /// the hands walked home between spots; chained spot to spot (terr72) a constructor's extractor frame comes
    /// 12-19 s after its order through 9:00 and its upgrade's 28-42 s from 10:00 (the far spots, the walk from
    /// the base). At 65 the program's own plan from the start was 18 extractors and 43 a second at 5:00, the
    /// hands' exactly; at 15 it is 25 and 59, thebluegecko's 26 and 62 (terr73-75). The territory bot grows the
    /// extractor's walk with the plan's t0 (its `walk_late`, 40 at 10:00: at one 15-s walk the plans from 10:00
    /// wanted 21-24 constructors against 25-28 and 20:00 read 40-70 under); here the defaults are a plan from the
    /// start. The commander's the territory bot derives every plan from the free spots within its leash, their mean
    /// distance at its speed plus the ring overhead, and bounds its extractors by their number (`commander_spots`):
    /// one walk for the whole map gave it either nine spots by 5:00 with the fifth 1,950 out (16, terr72-75) or
    /// none after the opening's (40, terr76-89), where his commander took two at 1,007 and 1,215 at 2:06-2:13 and
    /// none after.
    pub walk_extractor: f64,
    pub walk_upgrade: f64,
    /// The advanced constructor's walk to a basic spot. The territory bot derives it every plan from the free spots'
    /// mean distance from home at its speed plus the ring overhead (the proposal's second change, terr97): born at
    /// the plant, it reaches a free spot across the map, not at a constructor's chain walk, so from the minute the
    /// free spots lie far its upgrade beside home wins, as his did from 7:00 (mohos 1, 2, 5 and 10 in minutes 7-10
    /// while his basic constructors placed 7, 11, 4 and 2 extractors). Priced as a constructor's chain walk the
    /// plan's upgrades waited for the last spot, 11:00.
    pub advanced_walk_extractor: f64,
    pub walk_base: f64,
    pub walk_factory: f64,
    pub commander_walk_extractor: f64,
    pub commander_walk_base: f64,
    pub commander_walk_factory: f64,
    /// The commander takes extractors in rings whose mean distance from home is within this (the hands' leash).
    pub commander_reach: f64,
    /// The commander's basic extractors over the horizon at most: the free spots within its leash, counted by the
    /// bot every plan (unbounded here); with its extractor walk their mean, the pair prices the commander's spots
    /// as what stands free near home rather than one walk for the whole map.
    pub commander_spots: f64,
    /// Distance rings the spots fall in, by quantile from home, each with a like number of spots. With one ring an
    /// extractor's walk is the constant `walk_extractor`; with more, `ring_overhead` plus the ring's mean distance
    /// from home at the builder's speed. Three rings priced the far ring at 177 s where the hands' realized walk
    /// stayed at a median 56-66 s whatever the plan (terr31-33 against terr30): a constructor takes its next spot
    /// from where it stands, not from home, and the plan deferred the far spots for nothing (35-46 extractors at
    /// 10:00 against 50-61).
    pub rings: usize,
    pub ring_overhead: f64,
}

impl LpOptions {
    /// The bucket lengths over the horizon: fine, then mid, then coarse, the last one cut to the horizon.
    pub fn schedule(&self) -> Vec<f64> {
        let mut out = Vec::new();
        let mut t = 0.0;
        while t < self.horizon - 1e-9 {
            let dt = if t < self.mid_from { self.dt } else if t < self.coarse_from { self.mid_dt } else { self.coarse_dt };
            let dt = dt.min(self.horizon - t).max(1.0);
            out.push(dt);
            t += dt;
        }
        out
    }
}

impl Default for LpOptions {
    fn default() -> Self {
        LpOptions {
            dt: 10.0,
            mid_from: 60.0,
            mid_dt: 30.0,
            coarse_from: 300.0,
            coarse_dt: 60.0,
            horizon: 1200.0,
            max_constructors: 40.0,
            max_factories: 4.0,
            terminal: 0.0,
            energy_reserve: 0.2,
            energy_margin: 0.0,
            margin_from: 0.0,
            factory_from: 0.0,
            assist_factor: 2.0,
            plant_rings: Vec::new(),
            help_factor: 3.0,
            mobile_help: 0.0,
            nanos_per_factory: 6.0,
            max_nanos: 0.0,
            farm_big: 0.0,
            bank: false,
            reclaim: false,
            walk_extractor: 15.0,
            walk_upgrade: 40.0,
            advanced_walk_extractor: 15.0,
            walk_base: 3.0,
            walk_factory: 40.0,
            commander_walk_extractor: 40.0,
            commander_walk_base: 5.0,
            commander_walk_factory: 9.0,
            commander_reach: 1500.0,
            commander_spots: f64::INFINITY,
            rings: 1,
            ring_overhead: 5.0,
        }
    }
}

/// What a column counts: the whole units of a type, the frames of it under way, or its extractors in one ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    Whole,
    /// The frames of the type the snapshot reported as jobs, framed or still walking: the column's count runs from
    /// zero to their number as the work left on them is paid, a frame costing the mean of that work and no walk,
    /// so a frame produces as it is finished and nothing at the start (as a fraction of a standing unit, a fusion a
    /// third built was a third of a fusion's energy from the first bucket; terr25-lp-nullai-comet-1 read a
    /// thousand energy a second from four such frames, planned upgrades on it, and the hands followed it into a
    /// three-minute stall). Extractor frames stay fractions in their ring: a spot is what they hold.
    Frames,
    /// Extractors on the spots of distance ring `k` from home.
    Ring(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub unit: usize,
    pub class: Class,
}

/// A distance ring of spots: its outer bound from home, the spots in it, their mean metal and mean distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub bound: f64,
    pub spots: usize,
    pub metal: f64,
    pub distance: f64,
}

/// The plan: cumulative counts of every column at every bucket's start, the curves they imply, and the prices.
#[derive(Clone, Debug)]
pub struct LpPlan {
    pub t0: f64,
    /// The bucket lengths, and each bucket's start as seconds from `t0` (one more entry: the horizon).
    pub lengths: Vec<f64>,
    pub starts: Vec<f64>,
    pub buckets: usize,
    /// The distinct unit types in the program, as indices into the unit table.
    pub types: Vec<usize>,
    /// The columns, and per column (in order) the count at the start of bucket 0 ..= buckets.
    pub columns: Vec<Column>,
    pub counts: Vec<Vec<f64>>,
    /// The rings the extractor columns are over, nearest first; the last bound is unbounded.
    pub rings: Vec<Ring>,
    /// Metal a second at the start of each bucket, converters included; energy produced a second.
    pub metal_income: Vec<f64>,
    pub energy_income: Vec<f64>,
    pub metal: Vec<f64>,
    pub energy: Vec<f64>,
    /// Metal made over the horizon (the objective's main term).
    pub made: f64,
    pub objective: f64,
    /// The dual of each bucket's metal and energy balance: what one more metal or energy at that time is worth, in
    /// metal made over the rest of the game; and of each builder type's capacity: a build-power-second's worth.
    pub metal_price: Vec<f64>,
    pub energy_price: Vec<f64>,
    pub power_price: Vec<(usize, Vec<f64>)>,
    /// Every (builder type, column) the program gave power to.
    pub flows: Vec<Flow>,
    pub solve_time: f64,
    pub status: String,
    pub variables: usize,
    pub rows: usize,
}

impl LpPlan {
    /// The bucket `t` falls in and how far through it, as a share.
    fn slot(&self, t: f64) -> (usize, f64) {
        let s = (t - self.t0).max(0.0);
        if s >= self.starts[self.buckets] {
            return (self.buckets, 0.0);
        }
        let i = self.starts.iter().take(self.buckets).rposition(|start| *start <= s).unwrap_or(0);
        (i, ((s - self.starts[i]) / self.lengths[i]).clamp(0.0, 1.0))
    }

    /// The bucket whose start is the latest at or before `t`.
    pub fn bucket_at(&self, t: f64) -> usize {
        self.slot(t).0
    }

    fn line(&self, row: &[f64], t: f64) -> f64 {
        let (k, share) = self.slot(t);
        if k >= self.buckets { row[self.buckets] } else { row[k] + (row[k + 1] - row[k]) * share }
    }

    /// The plan's count of a unit type at absolute second `t`, over all its columns, between buckets by the line.
    pub fn count_at(&self, unit: usize, t: f64) -> f64 {
        self.columns.iter().enumerate().filter(|(_, c)| c.unit == unit).map(|(i, _)| self.line(&self.counts[i], t)).sum()
    }

    /// The plan's count of one column at absolute second `t`.
    pub fn count_at_class(&self, unit: usize, class: Class, t: f64) -> f64 {
        self.columns.iter().enumerate().filter(|(_, c)| c.unit == unit && c.class == class).map(|(i, _)| self.line(&self.counts[i], t)).sum()
    }

    /// Build-power-seconds the program has `builder` spend on `unit`, over all its columns, from its start to
    /// absolute second `t`.
    pub fn planned_power(&self, builder: usize, unit: usize, t: f64) -> f64 {
        self.flows.iter().filter(|f| f.builder == builder && f.unit == unit).map(|f| self.flow_to(f, t)).sum()
    }

    /// The same for one column.
    pub fn planned_power_class(&self, builder: usize, unit: usize, class: Class, t: f64) -> f64 {
        self.flows.iter().filter(|f| f.builder == builder && f.unit == unit && f.class == class).map(|f| self.flow_to(f, t)).sum()
    }

    fn flow_to(&self, f: &Flow, t: f64) -> f64 {
        let (k, share) = self.slot(t);
        f.per_bucket.iter().take(k).sum::<f64>() + if k < self.buckets { f.per_bucket[k] * share } else { 0.0 }
    }

    /// The plan's metal and energy stocks at absolute second `t`.
    pub fn metal_at(&self, t: f64) -> f64 {
        self.line(&self.metal, t)
    }

    pub fn energy_at(&self, t: f64) -> f64 {
        self.line(&self.energy, t)
    }

    pub fn income_at(&self, t: f64) -> f64 {
        self.line(&self.metal_income, t)
    }

    /// Energy produced a second at absolute second `t` (the constant maker included), between buckets by the line.
    pub fn energy_income_at(&self, t: f64) -> f64 {
        self.line(&self.energy_income, t)
    }

    /// The ring a point falls in, by the rings' bounds from home.
    pub fn ring_of(&self, distance_from_home: f64) -> usize {
        self.rings.iter().position(|r| distance_from_home < r.bound).unwrap_or(self.rings.len().saturating_sub(1))
    }
}

/// How a builder type's power reaches a column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Direct,
    /// A mobile builder guarding a factory: its power on the factory's products.
    Assist,
    /// A construction turret on a frame.
    Help,
    /// A builder taking a standing unit of the column down for its metal.
    Reclaim,
}

/// One builder type's power on one column over the horizon: build-power-seconds per bucket, and what a unit
/// costs that builder (the walk included; the work left on a frame for a Frames column), so the hands can follow
/// the program's allocation (T-DISPATCH).
#[derive(Clone, Debug)]
pub struct Flow {
    pub builder: usize,
    pub unit: usize,
    pub class: Class,
    pub kind: Kind,
    pub cost: f64,
    pub per_bucket: Vec<f64>,
}

/// One (builder type, column) the program lets spend build power.
struct Pair {
    builder: usize,
    column: usize,
    kind: Kind,
    /// Build-power-seconds a unit costs this builder, the walk included.
    cost: f64,
    /// Metal and energy drawn per build-power-second on it.
    metal_per_power: f64,
    energy_per_power: f64,
}

/// `offer`: the unit types the plan may build (the palette's build items). The commander is the constant builder.
pub fn solve(units: &Units, scenario: &Scenario, state: &State, offer: &[usize], options: &LpOptions) -> Result<LpPlan, String> {
    let began = Instant::now();
    let lengths = options.schedule();
    let buckets = lengths.len().max(1);
    let t_n = buckets + 1;
    let mut starts = vec![0.0; t_n];
    for t in 0..buckets {
        starts[t + 1] = starts[t] + lengths[t];
    }
    let dt_of = |t: usize| lengths[t.min(buckets - 1)];
    let commander = scenario.commander;
    let basic = units.basic_extraction();
    let is_extractor = |u: usize| units.list[u].extracts_metal > 0.0;
    let is_upgrade = |u: usize| units.list[u].extracts_metal > basic * 1.5;

    // The types: what is offered, plus whatever stands or is under way (its count and power matter), the commander
    // apart. The land lines only: a factory that makes no ground builder (air and hover lines, the gantry) and a
    // builder that flies or hovers leave the program, since fractional factories of every kind are cheap to it and
    // the hands would stand one of each.
    let ground = |u: usize| units.list[u].move_class.is_some_and(|m| matches!(m.kind, bot_protocol::MoveKind::Tank | bot_protocol::MoveKind::Bot));
    let builds_economy = |u: usize| units.list[u].builds.iter().any(|b| units.list[*b].extracts_metal > 0.0 || units.list[*b].role == Role::Factory);
    let land_builder = |u: usize| units.list[u].role == Role::Builder && ground(u) && builds_economy(u);
    let land_factory = |u: usize| units.list[u].role == Role::Factory && units.list[u].builds.iter().any(|b| land_builder(*b));
    let kept = |u: usize| match units.list[u].role {
        Role::Factory => land_factory(u),
        Role::Builder => land_builder(u),
        _ => true,
    };
    let mut types: Vec<usize> = offer.iter().copied().filter(|u| *u != commander && kept(*u)).collect();
    for u in state.standing.iter().map(|s| s.unit).chain(state.builders.iter().map(|b| b.unit)).chain(state.builders.iter().filter_map(|b| b.job.as_ref().map(|j| j.unit))).chain(state.committed.iter().map(|c| c.unit)) {
        if u != commander && !types.contains(&u) && matches!(units.list[u].role, Role::Eco | Role::Factory | Role::Builder | Role::Nano) {
            types.push(u);
        }
    }

    // The rings: the spots by distance quantile from home, each ring's mean metal and mean distance.
    let home = scenario.home;
    let dist = |p: (f64, f64)| ((p.0 - home.0).powi(2) + (p.1 - home.1).powi(2)).sqrt();
    let rings_n = options.rings.max(1);
    let mut sorted: Vec<f64> = scenario.spots.iter().map(|s| dist(s.at)).collect();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let bounds: Vec<f64> = (1..=rings_n).map(|k| if k == rings_n { f64::INFINITY } else { sorted.get(k * sorted.len() / rings_n).copied().unwrap_or(f64::INFINITY) }).collect();
    let ring_of = |p: (f64, f64)| bounds.iter().position(|b| dist(p) < *b).unwrap_or(rings_n - 1);
    let rings: Vec<Ring> = (0..rings_n)
        .map(|k| {
            let spots: Vec<&crate::game::Spot> = scenario.spots.iter().filter(|s| ring_of(s.at) == k).collect();
            let n = spots.len().max(1) as f64;
            let standing: Vec<f64> = state.standing.iter().filter(|s| is_extractor(s.unit) && ring_of(s.site) == k).map(|s| dist(s.site)).collect();
            let distance = if !spots.is_empty() { spots.iter().map(|s| dist(s.at)).sum::<f64>() / n } else if !standing.is_empty() { standing.iter().sum::<f64>() / standing.len() as f64 } else { 0.0 };
            Ring { bound: bounds[k], spots: spots.len(), metal: spots.iter().map(|s| s.metal).sum::<f64>() / n, distance }
        })
        .collect();

    // The columns: extractors by ring, everything else whole, and the frames under way of what has jobs.
    let mut columns: Vec<Column> = Vec::new();
    for &u in &types {
        if is_extractor(u) {
            for k in 0..rings_n {
                columns.push(Column { unit: u, class: Class::Ring(k) });
            }
        } else {
            columns.push(Column { unit: u, class: Class::Whole });
        }
    }
    // Per type with frames: their number and the work left on them, in build-power-seconds.
    let mut frames: Vec<(usize, f64, f64)> = Vec::new();
    for b in &state.builders {
        if let Some(job) = &b.job && !is_extractor(job.unit) && types.contains(&job.unit) {
            let left = units.list[job.unit].build_time * (1.0 - job.progress.clamp(0.0, 1.0));
            match frames.iter_mut().find(|(u, _, _)| *u == job.unit) {
                Some(f) => {
                    f.1 += 1.0;
                    f.2 += left;
                }
                None => frames.push((job.unit, 1.0, left)),
            }
        }
    }
    for (u, _, _) in &frames {
        columns.push(Column { unit: *u, class: Class::Frames });
    }
    let frame_of = |i: usize| if columns[i].class == Class::Frames { frames.iter().find(|(u, _, _)| *u == columns[i].unit).copied() } else { None };
    let col_index = |u: usize, class: Class| columns.iter().position(|c| c.unit == u && c.class == class);
    let cols_of = |u: usize| -> Vec<usize> { columns.iter().enumerate().filter(|(_, c)| c.unit == u).map(|(i, _)| i).collect() };
    let builders: Vec<usize> = types.iter().copied().filter(|u| units.list[*u].worker_time > 0.0).collect();
    let factories: Vec<usize> = types.iter().copied().filter(|u| units.list[*u].role == Role::Factory).collect();
    let products: Vec<usize> = types.iter().copied().filter(|u| factories.iter().any(|f| units.list[*f].builds.contains(u))).collect();
    // An advanced factory makes a constructor that builds an extractor upgrade; the turrets' assist is for its
    // products only (thebluegecko's 29 turrets were the advanced plant's; offered on every factory the program
    // pumped basic constructors from the first plant instead and held the advanced plant at a fraction to 7:48-9:00,
    // terr67-68: 38-50 basic constructors by 10:00 against 29-36 and the same extractors, 15:00 down 50-100 in all
    // eight games).
    let advanced_factory = |f: usize| units.list[f].builds.iter().any(|c| units.list[*c].builds.iter().any(|u| is_upgrade(*u)));
    let advanced_products: Vec<usize> = types.iter().copied().filter(|u| factories.iter().any(|f| advanced_factory(*f) && units.list[*f].builds.contains(u))).collect();
    let mobile = |b: usize| units.list[b].speed > 0.0;
    // The walk a mobile builder loses per unit of a column: the overhead by kind, plus the ring's distance at its speed.
    let walk = |b: usize, i: usize| -> f64 {
        let u = columns[i].unit;
        if !mobile(b) || units.list[u].speed > 0.0 || columns[i].class == Class::Frames {
            return 0.0;
        }
        let com = b == commander;
        match columns[i].class {
            Class::Ring(_) if rings_n == 1 => {
                if com {
                    options.commander_walk_extractor
                } else if is_upgrade(u) {
                    options.walk_upgrade
                } else if units.list[b].builds.iter().any(|v| is_upgrade(*v)) {
                    options.advanced_walk_extractor
                } else {
                    options.walk_extractor
                }
            }
            Class::Ring(k) => options.ring_overhead + rings[k].distance / units.list[b].speed.max(1.0),
            _ if units.list[u].role == Role::Factory => {
                if com { options.commander_walk_factory } else { options.walk_factory }
            }
            _ if com => options.commander_walk_base,
            _ => options.walk_base,
        }
    };
    let mut pairs: Vec<Pair> = Vec::new();
    for b in builders.iter().copied().chain(std::iter::once(commander)) {
        let def = &units.list[b];
        for (i, column) in columns.iter().enumerate() {
            let u = column.unit;
            let kind = if def.builds.contains(&u) {
                Kind::Direct
            } else if mobile(b) && def.role != Role::Factory && products.contains(&u) {
                Kind::Assist
            } else if options.mobile_help > 0.0 && mobile(b) && b != commander && def.role != Role::Factory && is_upgrade(u) && builders.iter().any(|x| mobile(*x) && units.list[*x].builds.contains(&u)) {
                // A mobile builder on the frame of an extractor upgrade it cannot start (the engine lets any builder
                // repair any frame): the basic constructor's help on the mohos, bounded by row 3d to `mobile_help` of
                // the mobile starters' direct power on the column. The walk is the upgrade's (the closure above).
                // Upgrades only: on the basic extractor the pair would be the advanced constructor's, a dear builder
                // walking to a cheap frame.
                Kind::Help
            } else if def.role == Role::Nano && products.contains(&u) {
                // A turret's assist on an advanced factory, under its own row (its Help pair here was bounded to
                // nothing: row 3b counts no factory's direct power).
                if advanced_products.contains(&u) && options.plant_rings.iter().any(|(f, capacity)| *capacity > 0.0 && units.list[*f].builds.contains(&u)) { Kind::Assist } else { continue }
            } else if def.role == Role::Nano && !is_extractor(u) && (options.max_nanos <= 0.0 || options.farm_big <= 0.0 || def.role == units.list[u].role || units.list[u].build_time >= options.farm_big) && builders.iter().chain(std::iter::once(&commander)).any(|x| units.list[*x].builds.contains(&u)) {
                Kind::Help
            } else {
                continue;
            };
            // The commander takes a ring whose nearest spots are within its reach (the ring's inner bound, not its
            // mean: ring 0 of Comet runs to 3,000 with a mean of 1,900, and against the mean the commander had no
            // extractor pair at all in terr31 to terr34, the openings at 7 constructors and 4-8 extractors by 5:00).
            if let Class::Ring(k) = column.class && b == commander && k > 0 && bounds[k - 1] > options.commander_reach {
                continue;
            }
            let cost = match column.class {
                Class::Frames => frame_of(i).map_or(units.list[u].build_time, |(_, n, left)| (left / n).max(1.0)),
                _ => units.list[u].build_time + def.worker_time * walk(b, i),
            };
            // A frame's work left draws metal and energy at the unit's own rate per build-power-second; a whole
            // unit's at that rate thinned by the walk, so the walk's power draws nothing.
            let (metal_per_power, energy_per_power) = match column.class {
                Class::Frames => (units.list[u].metal_cost / units.list[u].build_time.max(1.0), units.list[u].energy_cost / units.list[u].build_time.max(1.0)),
                _ => (units.list[u].metal_cost / cost, units.list[u].energy_cost / cost),
            };
            pairs.push(Pair { builder: b, column: i, kind, cost, metal_per_power, energy_per_power });
        }
    }
    // Reclaim: any builder but a factory may take a standing building of a Whole column down for its metal (not an
    // extractor, whose upgrade replaces it; not a frame; nothing mobile): the work is the unit's build time at the
    // builder's power, the metal comes back whole, the energy not at all. A turret reaches what stands round it and
    // the commander takes its base walk, as a mobile builder does. The completion row counts the pair against the
    // column, so a plan may build a thing, use it, and take it back for the next tier's metal: his vehicle plant
    // at 4:41 once its nine constructors were out, his advanced plant at 10:22 once its fifteen were, his solars as
    // the fusions stood.
    if options.reclaim {
        for b in builders.iter().copied().chain(std::iter::once(commander)) {
            let def = &units.list[b];
            if def.role == Role::Factory {
                continue;
            }
            for (i, column) in columns.iter().enumerate() {
                let u = column.unit;
                if column.class != Class::Whole || units.list[u].speed > 0.0 || is_extractor(u) || !matches!(units.list[u].role, Role::Eco | Role::Factory | Role::Nano) {
                    continue;
                }
                let walk = if !mobile(b) { 0.0 } else if b == commander { options.commander_walk_base } else { options.walk_base };
                let cost = units.list[u].build_time + def.worker_time * walk;
                pairs.push(Pair { builder: b, column: i, kind: Kind::Reclaim, cost, metal_per_power: -units.list[u].metal_cost / cost, energy_per_power: 0.0 });
            }
        }
    }
    let converters: Vec<usize> = (0..columns.len()).filter(|&i| units.list[columns[i].unit].conv_capacity > 0.0).collect();
    if columns.is_empty() || pairs.is_empty() {
        return Err("nothing to build".into());
    }

    // The data per column.
    let wind = match &scenario.wind {
        Wind::Constant(w) => *w,
        Wind::Trace(v) => v.iter().sum::<f64>() / v.len().max(1) as f64,
    };
    let metal_rate = |i: usize| -> f64 {
        let u = columns[i].unit;
        let def = &units.list[u];
        let extraction = match columns[i].class {
            Class::Ring(k) if is_upgrade(u) => rings[k].metal * (def.extracts_metal / basic - 1.0),
            Class::Ring(k) => rings[k].metal,
            _ => 0.0,
        };
        def.metal_make + extraction
    };
    let energy_rate = |i: usize| -> f64 {
        let u = columns[i].unit;
        units.list[u].energy_make + wind.min(units.list[u].wind_cap)
    };
    let const_metal = units.list[commander].metal_make;
    let const_energy = units.list[commander].energy_make;

    // The start: standing units, extractor frames as fractions in their ring (a standing upgrade counts as a basic
    // extractor of its ring and an upgrade), the other frames in their Frames column from zero.
    let mut n0 = vec![0.0; columns.len()];
    let mut standing_metal_storage = 0.0;
    let mut standing_energy_storage = 0.0;
    let column_for = |u: usize, site: (f64, f64)| -> Option<usize> { if is_extractor(u) { col_index(u, Class::Ring(ring_of(site))) } else { col_index(u, Class::Whole) } };
    for s in &state.standing {
        if let Some(i) = column_for(s.unit, s.site) {
            n0[i] += 1.0;
            standing_metal_storage += units.list[s.unit].metal_storage;
            standing_energy_storage += units.list[s.unit].energy_storage;
            if is_upgrade(s.unit) && let Some(bu) = types.iter().copied().find(|u| is_extractor(*u) && !is_upgrade(*u)) && let Some(bi) = col_index(bu, Class::Ring(ring_of(s.site))) {
                n0[bi] += 1.0;
            }
        }
    }
    for b in &state.builders {
        if let Some(i) = col_index(b.unit, Class::Whole) {
            n0[i] += 1.0;
            standing_metal_storage += units.list[b.unit].metal_storage;
            standing_energy_storage += units.list[b.unit].energy_storage;
        }
        if let Some(job) = &b.job && is_extractor(job.unit) && let Some(i) = column_for(job.unit, job.site) {
            n0[i] += job.progress;
        }
    }
    // The hands' commitments stand, the rest of their cost drawn from the stocks now; what the stock cannot pay the
    // engine draws from the income as it comes, which the plan overstates by that much for a minute.
    let (mut metal0, mut energy0) = (state.metal, state.energy);
    for c in &state.committed {
        if let Some(i) = col_index(c.unit, Class::Whole) {
            n0[i] += 1.0;
            metal0 -= units.list[c.unit].metal_cost * c.left;
            energy0 -= units.list[c.unit].energy_cost * c.left;
        }
    }
    let (metal0, energy0) = (metal0.max(0.0), energy0.max(0.0));
    // What the stock may exceed the standing storage by: the engine's figure, or the stock itself when it stands
    // above that (a cap on the first bucket must hold the start).
    let n0_metal_storage: f64 = columns.iter().enumerate().map(|(i, c)| units.list[c.unit].metal_storage * n0[i]).sum();
    let n0_energy_storage: f64 = columns.iter().enumerate().map(|(i, c)| units.list[c.unit].energy_storage * n0[i]).sum();
    let base_metal_storage = (state.metal_storage - standing_metal_storage).max(metal0 - n0_metal_storage).max(0.0);
    let base_energy_storage = (state.energy_storage - standing_energy_storage).max(energy0 - n0_energy_storage).max(0.0);

    // Variables: x[pair, t] for t < buckets; n[column, t] for t <= buckets; m[t], e[t] for t <= buckets; c[converter, t].
    let nx = pairs.len() * buckets;
    let nn = columns.len() * t_n;
    let x_at = |p: usize, t: usize| p * buckets + t;
    let n_at = |i: usize, t: usize| nx + i * t_n + t;
    let m_at = |t: usize| nx + nn + t;
    let e_at = |t: usize| nx + nn + t_n + t;
    let c_at = |k: usize, t: usize| nx + nn + 2 * t_n + k * buckets + t;
    let nvars = nx + nn + 2 * t_n + converters.len() * buckets;

    // Rows, equalities first: completion, the start; then the inequalities.
    let mut rows_i: Vec<usize> = Vec::new();
    let mut cols: Vec<usize> = Vec::new();
    let mut vals: Vec<f64> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    let mut row = 0usize;
    let put = |r: usize, c: usize, v: f64, rows_i: &mut Vec<usize>, cols: &mut Vec<usize>, vals: &mut Vec<f64>| {
        rows_i.push(r);
        cols.push(c);
        vals.push(v);
    };
    // 1. Completion: n[i, t+1] - n[i, t] - sum_p x[p, t] / cost_p = 0, a reclaim pair's power counting the other way.
    for i in 0..columns.len() {
        for t in 0..buckets {
            put(row, n_at(i, t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
            put(row, n_at(i, t), -1.0, &mut rows_i, &mut cols, &mut vals);
            for (p, pair) in pairs.iter().enumerate().filter(|(_, p)| p.column == i) {
                let sign = if pair.kind == Kind::Reclaim { 1.0 } else { -1.0 };
                put(row, x_at(p, t), sign / pair.cost, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(0.0);
            row += 1;
        }
    }
    // Caps: constructors of each tier (an advanced constructor builds an upgrade) and factories at most, every bucket.
    let advanced = |u: usize| units.list[u].builds.iter().any(|b| is_upgrade(*b));
    let basic_constructors: Vec<usize> = (0..columns.len()).filter(|&i| units.list[columns[i].unit].role == Role::Builder && !advanced(columns[i].unit)).collect();
    let advanced_constructors: Vec<usize> = (0..columns.len()).filter(|&i| units.list[columns[i].unit].role == Role::Builder && advanced(columns[i].unit)).collect();
    let factory_columns: Vec<usize> = (0..columns.len()).filter(|&i| units.list[columns[i].unit].role == Role::Factory).collect();
    let mut caps: Vec<(Vec<usize>, f64)> = vec![(basic_constructors, options.max_constructors), (advanced_constructors, options.max_constructors), (factory_columns.clone(), options.max_factories)];
    caps.retain(|(set, _)| !set.is_empty());
    // The start.
    for i in 0..columns.len() {
        put(row, n_at(i, 0), 1.0, &mut rows_i, &mut cols, &mut vals);
        b.push(n0[i]);
        row += 1;
    }
    put(row, m_at(0), 1.0, &mut rows_i, &mut cols, &mut vals);
    b.push(metal0);
    row += 1;
    put(row, e_at(0), 1.0, &mut rows_i, &mut cols, &mut vals);
    b.push(energy0);
    row += 1;
    let equalities = row;

    // A cap bounds what the plan adds, never what stands: with the hands past a cap (terr19-lp-nullai-comet-1: 51
    // basic constructors against forty, the lab substituting its kind while the advanced plant was a frame) a cap on
    // the first bucket has no feasible point, and every solve from 9:00 on failed in all four games.
    for (set, cap) in &caps {
        let standing: f64 = set.iter().map(|&i| n0[i]).sum();
        for t in 0..t_n {
            for &i in set {
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(cap.max(standing));
            row += 1;
        }
    }
    // The commander's extractors at most `commander_spots` over the horizon: its Direct pairs on the basic
    // extractor's ring columns, in units of the unit (x / cost), summed over every bucket.
    if options.commander_spots.is_finite() {
        let mut any = false;
        for (p, pair) in pairs.iter().enumerate() {
            if pair.builder == commander && pair.kind == Kind::Direct && matches!(columns[pair.column].class, Class::Ring(_)) && !is_upgrade(columns[pair.column].unit) {
                any = true;
                for t in 0..buckets {
                    put(row, x_at(p, t), 1.0 / pair.cost, &mut rows_i, &mut cols, &mut vals);
                }
            }
        }
        if any {
            b.push(options.commander_spots.max(0.0));
            row += 1;
        }
    }
    // No factory before `factory_from`: the factory columns hold what stands until the bucket that starts at or
    // after it.
    if options.factory_from > 0.0 {
        for &i in &factory_columns {
            for t in 1..t_n {
                if state.t0 + starts[t] > options.factory_from {
                    break;
                }
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
                b.push(n0[i]);
                row += 1;
            }
        }
    }
    // A Frames column counts its frames at most.
    for i in 0..columns.len() {
        if let Some((_, n, _)) = frame_of(i) {
            for t in 0..t_n {
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
                b.push(n);
                row += 1;
            }
        }
    }

    // 2. Build power: sum_u x[b, u, t] - bp[b] dt sum over b's columns n[., t] <= 0; the commander's a constant.
    let mut power_rows: Vec<(usize, Vec<usize>)> = Vec::new();
    for bt in builders.iter().copied().chain(std::iter::once(commander)) {
        let mut rows_of = Vec::new();
        let own = cols_of(bt);
        for t in 0..buckets {
            for (p, _) in pairs.iter().enumerate().filter(|(_, p)| p.builder == bt) {
                put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            if own.is_empty() {
                b.push(units.list[bt].worker_time * dt_of(t));
            } else {
                for &i in &own {
                    put(row, n_at(i, t), -units.list[bt].worker_time * dt_of(t), &mut rows_i, &mut cols, &mut vals);
                }
                b.push(0.0);
            }
            rows_of.push(row);
            row += 1;
        }
        power_rows.push((bt, rows_of));
    }
    // 3. Assist, the mobile builders: sum over their assisting pairs <= A dt sum_f bp[f] n[f, t] (the yard clears
    // between units for the guards).
    for t in 0..buckets {
        for (p, _) in pairs.iter().enumerate().filter(|(_, p)| p.kind == Kind::Assist && mobile(p.builder)) {
            put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
        }
        for &i in &factory_columns {
            put(row, n_at(i, t), -options.assist_factor * units.list[columns[i].unit].worker_time * dt_of(t), &mut rows_i, &mut cols, &mut vals);
        }
        b.push(0.0);
        row += 1;
    }
    // 3a. Assist, the turrets: per factory type and bucket, the turrets' power on its products at most a turret's
    // power times the turrets its plants' rings hold (`plant_rings`: the hands' count of the turrets standing or
    // framed within PLANT_RING of each plant standing or committed, plus the free ring points, packed as the hands
    // would pack them; his 29 within 400 of his advanced plant). On the plants standing or committed, never the
    // plan's fractions: against n[f, t] a tenth of an advanced plant with ten times its power in turrets round it
    // was a whole plant's production for a tenth of its price, and terr67 held the advanced plant at a fraction from
    // 4:00 to 7:50-9:00 while buying the turrets, which stood idle at the farm; a turret stands round a plant that
    // exists.
    let turret_power: f64 = types.iter().copied().filter(|u| units.list[*u].role == Role::Nano).map(|u| units.list[u].worker_time).fold(0.0, f64::max);
    for (f, capacity) in &options.plant_rings {
        let turret_pairs: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.kind == Kind::Assist && !mobile(p.builder) && units.list[*f].builds.contains(&columns[p.column].unit)).map(|(p, _)| p).collect();
        if turret_pairs.is_empty() {
            continue;
        }
        for t in 0..buckets {
            for &p in &turret_pairs {
                put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(turret_power * capacity * dt_of(t));
            row += 1;
        }
    }
    // 3b. Help: per column and bucket, the turrets' power on it <= H times the mobile builders' direct power on it.
    for i in 0..columns.len() {
        let helping: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.column == i && p.kind == Kind::Help && !mobile(p.builder)).map(|(p, _)| p).collect();
        if helping.is_empty() {
            continue;
        }
        let direct: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.column == i && p.kind == Kind::Direct && units.list[p.builder].role != Role::Factory).map(|(p, _)| p).collect();
        for t in 0..buckets {
            for &p in &helping {
                put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            for &p in &direct {
                put(row, x_at(p, t), -options.help_factor, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(0.0);
            row += 1;
        }
    }
    // 3d. Mobile help: per column and bucket, the power of the mobile builders that cannot start the type <= M times
    // the mobile starters' direct power on it (his basic constructors' share of the moho frames, 0.35).
    if options.mobile_help > 0.0 {
        for i in 0..columns.len() {
            let helping: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.column == i && p.kind == Kind::Help && mobile(p.builder)).map(|(p, _)| p).collect();
            if helping.is_empty() {
                continue;
            }
            let direct: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.column == i && p.kind == Kind::Direct && mobile(p.builder)).map(|(p, _)| p).collect();
            for t in 0..buckets {
                for &p in &helping {
                    put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
                }
                for &p in &direct {
                    put(row, x_at(p, t), -options.mobile_help, &mut rows_i, &mut cols, &mut vals);
                }
                b.push(0.0);
                row += 1;
            }
        }
    }
    // 3c. Turrets: a number at most (the farm), else per factory: sum_nano n[nano, t] - N sum_f n[f, t] <= 0.
    let nanos: Vec<usize> = (0..columns.len()).filter(|&i| units.list[columns[i].unit].role == Role::Nano).collect();
    if !nanos.is_empty() {
        let standing = nanos.iter().map(|&i| n0[i]).sum::<f64>();
        let farm = options.max_nanos > 0.0;
        let over = if farm { standing - options.max_nanos } else { standing - options.nanos_per_factory * factory_columns.iter().map(|&i| n0[i]).sum::<f64>() };
        for t in 0..t_n {
            for &i in &nanos {
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            if !farm {
                for &i in &factory_columns {
                    put(row, n_at(i, t), -options.nanos_per_factory, &mut rows_i, &mut cols, &mut vals);
                }
            }
            b.push(if farm { options.max_nanos + over.max(0.0) } else { over.max(0.0) });
            row += 1;
        }
    }
    // 4. Metal: m[t+1] - m[t] - dt sum mm n[i, t] - sum cf c[k, t] + sum cm x[p, t] <= dt const_metal.
    let mut metal_rows = Vec::new();
    for t in 0..buckets {
        put(row, m_at(t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
        put(row, m_at(t), -1.0, &mut rows_i, &mut cols, &mut vals);
        for i in 0..columns.len() {
            let rate = metal_rate(i);
            if rate != 0.0 {
                put(row, n_at(i, t), -dt_of(t) * rate, &mut rows_i, &mut cols, &mut vals);
            }
        }
        for (k, &i) in converters.iter().enumerate() {
            put(row, c_at(k, t), -units.list[columns[i].unit].conv_efficiency, &mut rows_i, &mut cols, &mut vals);
        }
        for (p, pair) in pairs.iter().enumerate() {
            if pair.metal_per_power != 0.0 {
                put(row, x_at(p, t), pair.metal_per_power, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(dt_of(t) * const_metal);
        metal_rows.push(row);
        row += 1;
    }
    // 5. Energy: e[t+1] - e[t] - dt sum er n[i, t] + sum c[k, t] + sum ce x[p, t] <= dt const_energy.
    let mut energy_rows = Vec::new();
    for t in 0..buckets {
        put(row, e_at(t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
        put(row, e_at(t), -1.0, &mut rows_i, &mut cols, &mut vals);
        for i in 0..columns.len() {
            let rate = energy_rate(i);
            if rate != 0.0 {
                put(row, n_at(i, t), -dt_of(t) * rate, &mut rows_i, &mut cols, &mut vals);
            }
        }
        for k in 0..converters.len() {
            put(row, c_at(k, t), 1.0, &mut rows_i, &mut cols, &mut vals);
        }
        for (p, pair) in pairs.iter().enumerate() {
            if pair.energy_per_power != 0.0 {
                put(row, x_at(p, t), pair.energy_per_power, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(dt_of(t) * const_energy);
        energy_rows.push(row);
        row += 1;
    }
    // Storage: m[t] - sum sm n[i, t] <= base; the same for energy.
    for t in 0..t_n {
        put(row, m_at(t), 1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, c) in columns.iter().enumerate() {
            if units.list[c.unit].metal_storage != 0.0 {
                put(row, n_at(i, t), -units.list[c.unit].metal_storage, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(base_metal_storage);
        row += 1;
        put(row, e_at(t), 1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, c) in columns.iter().enumerate() {
            if units.list[c.unit].energy_storage != 0.0 {
                put(row, n_at(i, t), -units.list[c.unit].energy_storage, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(base_energy_storage);
        row += 1;
    }
    // The energy reserve: reserve (base + sum se n[i, t]) - e[t] <= slack from the second bucket, the slack what
    // the start lacks of it (a stall is where the hands are, not a reason for no plan).
    let slack = (options.energy_reserve * (base_energy_storage + n0_energy_storage) - energy0).max(0.0);
    for t in 1..t_n {
        put(row, e_at(t), -1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, c) in columns.iter().enumerate() {
            if units.list[c.unit].energy_storage != 0.0 {
                put(row, n_at(i, t), options.energy_reserve * units.list[c.unit].energy_storage, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(slack - options.energy_reserve * base_energy_storage);
        row += 1;
    }
    // The energy margin: (1 + margin) sum ce x[p, t] + sum c[k, t] - dt sum er n[i, t] <= dt const_energy from the
    // second bucket: the bucket's production covers its build draw with the margin to spare, the converters on
    // what is left.
    if options.energy_margin > 0.0 {
        for t in 1..buckets {
            if state.t0 + starts[t] < options.margin_from {
                continue;
            }
            for (p, pair) in pairs.iter().enumerate() {
                if pair.energy_per_power != 0.0 {
                    put(row, x_at(p, t), (1.0 + options.energy_margin) * pair.energy_per_power, &mut rows_i, &mut cols, &mut vals);
                }
            }
            for k in 0..converters.len() {
                put(row, c_at(k, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            for i in 0..columns.len() {
                let rate = energy_rate(i);
                if rate != 0.0 {
                    put(row, n_at(i, t), -dt_of(t) * rate, &mut rows_i, &mut cols, &mut vals);
                }
            }
            b.push(dt_of(t) * const_energy);
            row += 1;
        }
    }
    // 6. Conversion: c[k, t] - dt cc n[k, t] <= 0.
    for (k, &i) in converters.iter().enumerate() {
        for t in 0..buckets {
            put(row, c_at(k, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            put(row, n_at(i, t), -dt_of(t) * units.list[columns[i].unit].conv_capacity, &mut rows_i, &mut cols, &mut vals);
            b.push(0.0);
            row += 1;
        }
    }
    // 7. Spots and upgrades, per ring.
    for k in 0..rings_n {
        let basics: Vec<usize> = (0..columns.len()).filter(|&i| columns[i].class == Class::Ring(k) && !is_upgrade(columns[i].unit)).collect();
        let upgrades: Vec<usize> = (0..columns.len()).filter(|&i| columns[i].class == Class::Ring(k) && is_upgrade(columns[i].unit)).collect();
        let basics_standing: f64 = basics.iter().map(|&i| n0[i]).sum();
        for t in 0..t_n {
            if !basics.is_empty() {
                for &i in &basics {
                    put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
                }
                b.push((rings[k].spots as f64).max(basics_standing));
                row += 1;
            }
            if !upgrades.is_empty() {
                for &i in &upgrades {
                    put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
                }
                for &i in &basics {
                    put(row, n_at(i, t), -1.0, &mut rows_i, &mut cols, &mut vals);
                }
                b.push(0.0);
                row += 1;
            }
        }
    }
    // A reclaim in the last bucket returns metal nothing can spend and frees a reserve nothing needs (the interior
    // point took down six fusions and the storages in minute 19 to loosen the energy reserve at the horizon): none.
    for (p, _) in pairs.iter().enumerate().filter(|(_, p)| p.kind == Kind::Reclaim) {
        put(row, x_at(p, buckets - 1), 1.0, &mut rows_i, &mut cols, &mut vals);
        b.push(0.0);
        row += 1;
    }
    // 8. Nothing negative.
    for v in 0..nvars {
        put(row, v, -1.0, &mut rows_i, &mut cols, &mut vals);
        b.push(0.0);
        row += 1;
    }
    let nrows = row;

    // The objective, minimized: minus the metal made (and the terminal income).
    let mut q = vec![0.0; nvars];
    for i in 0..columns.len() {
        let rate = metal_rate(i);
        for t in 0..buckets {
            q[n_at(i, t)] -= dt_of(t) * rate;
        }
        q[n_at(i, buckets)] -= options.terminal * rate;
    }
    for (k, &i) in converters.iter().enumerate() {
        for t in 0..buckets {
            q[c_at(k, t)] -= units.list[columns[i].unit].conv_efficiency;
        }
    }
    if options.bank {
        q[m_at(buckets)] -= 1.0;
    }
    for (p, pair) in pairs.iter().enumerate().filter(|(_, p)| p.kind == Kind::Reclaim) {
        for t in 0..buckets {
            q[x_at(p, t)] += RECLAIM_TIE / pair.cost;
        }
    }

    let a = CscMatrix::new_from_triplets(nrows, nvars, rows_i, cols, vals);
    let p = CscMatrix::zeros((nvars, nvars));
    let cones = [SupportedConeT::ZeroConeT(equalities), SupportedConeT::NonnegativeConeT(nrows - equalities)];
    let settings = DefaultSettings { verbose: false, tol_gap_abs: 1e-4, tol_gap_rel: 1e-5, tol_feas: 1e-6, ..Default::default() };
    let mut solver = DefaultSolver::new(&p, &q, &a, &b, &cones, settings);
    solver.solve();
    let solution = &solver.solution;
    let status = format!("{:?}", solution.status);
    if !matches!(solution.status, SolverStatus::Solved | SolverStatus::AlmostSolved) {
        return Err(format!("the program did not solve: {status}"));
    }
    let x = &solution.x;
    let counts: Vec<Vec<f64>> = (0..columns.len()).map(|i| (0..t_n).map(|t| x[n_at(i, t)].max(0.0)).collect()).collect();
    let converted_at = |t: usize| -> f64 { converters.iter().enumerate().map(|(k, &i)| units.list[columns[i].unit].conv_efficiency * x[c_at(k, t.min(buckets - 1))].max(0.0)).sum::<f64>() / dt_of(t) };
    let metal_income: Vec<f64> = (0..t_n).map(|t| const_metal + (0..columns.len()).map(|i| metal_rate(i) * counts[i][t]).sum::<f64>() + converted_at(t)).collect();
    let energy_income: Vec<f64> = (0..t_n).map(|t| const_energy + (0..columns.len()).map(|i| energy_rate(i).max(0.0) * counts[i][t]).sum::<f64>()).collect();
    let made: f64 = (0..buckets).map(|t| dt_of(t) * metal_income[t]).sum();
    let flows: Vec<Flow> = pairs
        .iter()
        .enumerate()
        .map(|(p, pair)| Flow {
            builder: pair.builder,
            unit: columns[pair.column].unit,
            class: columns[pair.column].class,
            kind: pair.kind,
            cost: pair.cost,
            per_bucket: (0..buckets).map(|t| x[x_at(p, t)].max(0.0)).collect(),
        })
        .filter(|f| f.per_bucket.iter().sum::<f64>() > 1e-6)
        .collect();
    Ok(LpPlan {
        t0: state.t0,
        lengths,
        starts,
        buckets,
        types,
        columns,
        counts,
        rings,
        metal_income,
        energy_income,
        metal: (0..t_n).map(|t| x[m_at(t)]).collect(),
        energy: (0..t_n).map(|t| x[e_at(t)]).collect(),
        made,
        objective: -solution.obj_val,
        metal_price: metal_rows.iter().map(|r| solution.z[*r]).collect(),
        energy_price: energy_rows.iter().map(|r| solution.z[*r]).collect(),
        power_price: power_rows.iter().map(|(bt, rows)| (*bt, rows.iter().map(|r| solution.z[*r]).collect())).collect(),
        flows,
        solve_time: began.elapsed().as_secs_f64(),
        status,
        variables: nvars,
        rows: nrows,
    })
}
