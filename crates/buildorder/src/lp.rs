//! The planner as a linear program (docs/design/2026-10-20-territory-lp.md): how much build power of each builder
//! type goes to each unit type in each ten-second bucket to the game's end, so that the most metal is made. Solved
//! globally by an interior-point method (clarabel), so no chain of prerequisites is a valley: the tier-2 branch the
//! anneal missed in half of terr5-all-nullai-comet-1's games is a flow like any other here.
//!
//! The relaxation: counts are fractional, walks are a calibrated overhead per builder and unit type, geometry is
//! the hands', converters are bounded by capacity alone (the engine's three-quarter threshold costs one transient),
//! and stocks never go negative (the plan does not stall).

use std::time::Instant;

use clarabel::algebra::CscMatrix;
use clarabel::solver::{DefaultSettings, DefaultSolver, IPSolver, SolverStatus, SupportedConeT};

use crate::sim::{Scenario, State, Wind};
use crate::units::{Role, Units};

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
    /// Build power mobile builders may put on factory products, as a multiple of the factories' own (the yard
    /// clears between units).
    pub assist_factor: f64,
    /// Build power construction turrets may put on a unit type in a bucket, as a multiple of what mobile builders
    /// put on it directly: a turret helps frames in its reach and starts none, so without this bound the program
    /// buys turrets as free build power and runs a converter spiral to 4,000 metal a second (the first solve on
    /// terr5-all's seed 1: 441 turrets, 2,900 converters).
    pub help_factor: f64,
    /// Construction turrets at most, per factory standing: a turret's reach holds a few frames round a factory and
    /// no more (the first solves bought 285-441 turrets).
    pub nanos_per_factory: f64,
    /// Seconds a constructor loses per unit it builds, by kind: an extractor or its upgrade (the spot), any other
    /// building, a factory; and the commander's own three. From the territory bot's `begun:` lines (the order's
    /// age at its frame): terr9-lp-nullai-comet-1 measured a constructor's extractor at a median 41 s and its base
    /// building at 23 s, the commander's at 18 and 26, where the first solves had assumed 10 and 3.5; terr16, with
    /// economy buildings beside the builder wherever it stands and the far spots reached, a constructor's extractor
    /// 61-76 (22-28 for the spots within 1,500 of home, 134-191 beyond 6,000), its other building 2, its factory
    /// 19-127; the commander's 14-18, 2-5 and 9.
    pub walk_extractor: f64,
    pub walk_base: f64,
    pub walk_factory: f64,
    pub commander_walk_extractor: f64,
    pub commander_walk_base: f64,
    pub commander_walk_factory: f64,
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
        LpOptions { dt: 10.0, mid_from: 60.0, mid_dt: 30.0, coarse_from: 300.0, coarse_dt: 60.0, horizon: 1200.0, max_constructors: 40.0, max_factories: 4.0, terminal: 0.0, assist_factor: 2.0, help_factor: 3.0, nanos_per_factory: 6.0, walk_extractor: 65.0, walk_base: 3.0, walk_factory: 40.0, commander_walk_extractor: 16.0, commander_walk_base: 5.0, commander_walk_factory: 9.0 }
    }
}

/// The plan: cumulative counts of every type at every bucket's start, the curves they imply, and the prices.
#[derive(Clone, Debug)]
pub struct LpPlan {
    pub t0: f64,
    /// The bucket lengths, and each bucket's start as seconds from `t0` (one more entry: the horizon).
    pub lengths: Vec<f64>,
    pub starts: Vec<f64>,
    pub buckets: usize,
    /// Unit types in the program, as indices into the unit table.
    pub types: Vec<usize>,
    /// Per type (in `types` order), the count at the start of bucket 0 ..= buckets.
    pub counts: Vec<Vec<f64>>,
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
    /// Every (builder type, unit type) the program gave power to.
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

    /// The plan's count of a type at absolute second `t`, between buckets by the line.
    pub fn count_at(&self, unit: usize, t: f64) -> f64 {
        let Some(i) = self.types.iter().position(|u| *u == unit) else { return 0.0 };
        let (k, share) = self.slot(t);
        let row = &self.counts[i];
        if k >= self.buckets { row[self.buckets] } else { row[k] + (row[k + 1] - row[k]) * share }
    }

    /// Build-power-seconds the program has `builder` spend on `unit` from its start to absolute second `t`.
    pub fn planned_power(&self, builder: usize, unit: usize, t: f64) -> f64 {
        let (k, share) = self.slot(t);
        self.flows
            .iter()
            .filter(|f| f.builder == builder && f.unit == unit)
            .map(|f| f.per_bucket.iter().take(k).sum::<f64>() + if k < self.buckets { f.per_bucket[k] * share } else { 0.0 })
            .sum()
    }

    pub fn income_at(&self, t: f64) -> f64 {
        let (k, share) = self.slot(t);
        if k >= self.buckets { self.metal_income[self.buckets] } else { self.metal_income[k] + (self.metal_income[k + 1] - self.metal_income[k]) * share }
    }

    /// Energy produced a second at absolute second `t` (the constant maker included), between buckets by the line.
    pub fn energy_income_at(&self, t: f64) -> f64 {
        let (k, share) = self.slot(t);
        if k >= self.buckets { self.energy_income[self.buckets] } else { self.energy_income[k] + (self.energy_income[k + 1] - self.energy_income[k]) * share }
    }
}

/// How a builder type's power reaches a unit type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Direct,
    /// A mobile builder guarding a factory: its power on the factory's products.
    Assist,
    /// A construction turret on a frame.
    Help,
}

/// One builder type's power on one unit type over the horizon: build-power-seconds per bucket, and what a unit
/// costs that builder (the walk included), so the hands can follow the program's allocation (T-DISPATCH).
#[derive(Clone, Debug)]
pub struct Flow {
    pub builder: usize,
    pub unit: usize,
    pub kind: Kind,
    pub cost: f64,
    pub per_bucket: Vec<f64>,
}

/// One (builder type, unit type) the program lets spend build power.
struct Pair {
    builder: usize,
    unit: usize,
    kind: Kind,
    /// Build-power-seconds a unit costs this builder, the walk included.
    cost: f64,
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
    let index = |u: usize| types.iter().position(|x| *x == u);
    let builders: Vec<usize> = types.iter().copied().filter(|u| units.list[*u].worker_time > 0.0).collect();
    let factories: Vec<usize> = types.iter().copied().filter(|u| units.list[*u].role == Role::Factory).collect();
    let products: Vec<usize> = types.iter().copied().filter(|u| factories.iter().any(|f| units.list[*f].builds.contains(u))).collect();
    let mobile = |b: usize| units.list[b].speed > 0.0;
    let walk = |b: usize, u: usize| -> f64 {
        if !mobile(b) || units.list[u].speed > 0.0 {
            return 0.0;
        }
        let com = b == commander;
        if is_extractor(u) {
            if com { options.commander_walk_extractor } else { options.walk_extractor }
        } else if units.list[u].role == Role::Factory {
            if com { options.commander_walk_factory } else { options.walk_factory }
        } else if com {
            options.commander_walk_base
        } else {
            options.walk_base
        }
    };
    let mut pairs: Vec<Pair> = Vec::new();
    for b in builders.iter().copied().chain(std::iter::once(commander)) {
        let def = &units.list[b];
        for &u in &types {
            let kind = if def.builds.contains(&u) {
                Kind::Direct
            } else if mobile(b) && def.role != Role::Factory && products.contains(&u) {
                Kind::Assist
            } else if def.role == Role::Nano && !is_extractor(u) && builders.iter().chain(std::iter::once(&commander)).any(|x| units.list[*x].builds.contains(&u)) {
                Kind::Help
            } else {
                continue;
            };
            pairs.push(Pair { builder: b, unit: u, kind, cost: units.list[u].build_time + def.worker_time * walk(b, u) });
        }
    }
    let converters: Vec<usize> = types.iter().copied().filter(|u| units.list[*u].conv_capacity > 0.0).collect();
    if types.is_empty() || pairs.is_empty() {
        return Err("nothing to build".into());
    }

    // The data per type.
    let mean_spot = scenario.spots.iter().map(|s| s.metal).sum::<f64>() / scenario.spots.len().max(1) as f64;
    let wind = match &scenario.wind {
        Wind::Constant(w) => *w,
        Wind::Trace(v) => v.iter().sum::<f64>() / v.len().max(1) as f64,
    };
    let metal_rate = |u: usize| -> f64 {
        let def = &units.list[u];
        let extraction = if !is_extractor(u) { 0.0 } else if is_upgrade(u) { mean_spot * (def.extracts_metal / basic - 1.0) } else { mean_spot };
        def.metal_make + extraction
    };
    let energy_rate = |u: usize| -> f64 { units.list[u].energy_make + wind.min(units.list[u].wind_cap) };
    let const_metal = units.list[commander].metal_make;
    let const_energy = units.list[commander].energy_make;

    // The start: standing units and frames as fractions; a standing upgrade counts as a basic extractor and an upgrade.
    let mut n0 = vec![0.0; types.len()];
    let mut standing_metal_storage = 0.0;
    let mut standing_energy_storage = 0.0;
    for s in &state.standing {
        if let Some(i) = index(s.unit) {
            n0[i] += 1.0;
            standing_metal_storage += units.list[s.unit].metal_storage;
            standing_energy_storage += units.list[s.unit].energy_storage;
            if is_upgrade(s.unit) && let Some(bi) = types.iter().position(|u| is_extractor(*u) && !is_upgrade(*u)) {
                n0[bi] += 1.0;
            }
        }
    }
    for b in &state.builders {
        if let Some(i) = index(b.unit) {
            n0[i] += 1.0;
            standing_metal_storage += units.list[b.unit].metal_storage;
            standing_energy_storage += units.list[b.unit].energy_storage;
        }
        if let Some(job) = &b.job && let Some(i) = index(job.unit) {
            n0[i] += job.progress;
        }
    }
    // The hands' commitments stand, the rest of their cost drawn from the stocks now; what the stock cannot pay the
    // engine draws from the income as it comes, which the plan overstates by that much for a minute.
    let (mut metal0, mut energy0) = (state.metal, state.energy);
    for c in &state.committed {
        if let Some(i) = index(c.unit) {
            n0[i] += 1.0;
            metal0 -= units.list[c.unit].metal_cost * c.left;
            energy0 -= units.list[c.unit].energy_cost * c.left;
        }
    }
    let (metal0, energy0) = (metal0.max(0.0), energy0.max(0.0));
    // What the stock may exceed the standing storage by: the engine's figure, or the stock itself when it stands
    // above that (a cap on the first bucket must hold the start).
    let n0_metal_storage: f64 = types.iter().enumerate().map(|(i, &u)| units.list[u].metal_storage * n0[i]).sum();
    let n0_energy_storage: f64 = types.iter().enumerate().map(|(i, &u)| units.list[u].energy_storage * n0[i]).sum();
    let base_metal_storage = (state.metal_storage - standing_metal_storage).max(metal0 - n0_metal_storage).max(0.0);
    let base_energy_storage = (state.energy_storage - standing_energy_storage).max(energy0 - n0_energy_storage).max(0.0);

    // Variables: x[pair, t] for t < buckets; n[type, t] for t <= buckets; m[t], e[t] for t <= buckets; c[converter, t].
    let nx = pairs.len() * buckets;
    let nn = types.len() * t_n;
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
    // 1. Completion: n[i, t+1] - n[i, t] - sum_p x[p, t] / cost_p = 0.
    for (i, &u) in types.iter().enumerate() {
        for t in 0..buckets {
            put(row, n_at(i, t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
            put(row, n_at(i, t), -1.0, &mut rows_i, &mut cols, &mut vals);
            for (p, pair) in pairs.iter().enumerate().filter(|(_, p)| p.unit == u) {
                put(row, x_at(p, t), -1.0 / pair.cost, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(0.0);
            row += 1;
        }
    }
    // Caps: constructors of each tier (an advanced constructor builds an upgrade) and factories at most, every bucket.
    let advanced = |u: usize| units.list[u].builds.iter().any(|b| is_upgrade(*b));
    let basic_constructors: Vec<usize> = (0..types.len()).filter(|&i| units.list[types[i]].role == Role::Builder && !advanced(types[i])).collect();
    let advanced_constructors: Vec<usize> = (0..types.len()).filter(|&i| units.list[types[i]].role == Role::Builder && advanced(types[i])).collect();
    let factory_types: Vec<usize> = (0..types.len()).filter(|&i| units.list[types[i]].role == Role::Factory).collect();
    let mut caps: Vec<(Vec<usize>, f64)> = vec![(basic_constructors, options.max_constructors), (advanced_constructors, options.max_constructors), (factory_types, options.max_factories)];
    caps.retain(|(set, _)| !set.is_empty());
    // The start.
    for i in 0..types.len() {
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

    // 2. Build power: sum_u x[b, u, t] - bp[b] dt n[b, t] <= 0; the commander's a constant.
    let mut power_rows: Vec<(usize, Vec<usize>)> = Vec::new();
    for bt in builders.iter().copied().chain(std::iter::once(commander)) {
        let mut rows_of = Vec::new();
        for t in 0..buckets {
            for (p, _) in pairs.iter().enumerate().filter(|(_, p)| p.builder == bt) {
                put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            match index(bt) {
                Some(i) => {
                    put(row, n_at(i, t), -units.list[bt].worker_time * dt_of(t), &mut rows_i, &mut cols, &mut vals);
                    b.push(0.0);
                }
                None => b.push(units.list[bt].worker_time * dt_of(t)),
            }
            rows_of.push(row);
            row += 1;
        }
        power_rows.push((bt, rows_of));
    }
    // 3. Assist: sum over assisting pairs <= A dt sum_f bp[f] n[f, t].
    for t in 0..buckets {
        for (p, _) in pairs.iter().enumerate().filter(|(_, p)| p.kind == Kind::Assist) {
            put(row, x_at(p, t), 1.0, &mut rows_i, &mut cols, &mut vals);
        }
        for &f in &factories {
            put(row, n_at(index(f).unwrap(), t), -options.assist_factor * units.list[f].worker_time * dt_of(t), &mut rows_i, &mut cols, &mut vals);
        }
        b.push(0.0);
        row += 1;
    }
    // 3b. Help: per unit type and bucket, the turrets' power on it <= H times the mobile builders' direct power on it.
    for (i, &u) in types.iter().enumerate() {
        let _ = i;
        let helping: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.unit == u && p.kind == Kind::Help).map(|(p, _)| p).collect();
        if helping.is_empty() {
            continue;
        }
        let direct: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.unit == u && p.kind == Kind::Direct && units.list[p.builder].role != Role::Factory).map(|(p, _)| p).collect();
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
    // 3c. Turrets per factory: sum_nano n[nano, t] - N sum_f n[f, t] <= 0.
    let nanos: Vec<usize> = (0..types.len()).filter(|&i| units.list[types[i]].role == Role::Nano).collect();
    if !nanos.is_empty() {
        let over = nanos.iter().map(|&i| n0[i]).sum::<f64>() - options.nanos_per_factory * factories.iter().map(|&f| n0[index(f).unwrap()]).sum::<f64>();
        for t in 0..t_n {
            for &i in &nanos {
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            for &f in &factories {
                put(row, n_at(index(f).unwrap(), t), -options.nanos_per_factory, &mut rows_i, &mut cols, &mut vals);
            }
            b.push(over.max(0.0));
            row += 1;
        }
    }
    // 4. Metal: m[t+1] - m[t] - dt sum mm n[u, t] - sum cf c[k, t] + sum cm/cost x[p, t] <= dt const_metal.
    let mut metal_rows = Vec::new();
    for t in 0..buckets {
        put(row, m_at(t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
        put(row, m_at(t), -1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, &u) in types.iter().enumerate() {
            let rate = metal_rate(u);
            if rate != 0.0 {
                put(row, n_at(i, t), -dt_of(t) * rate, &mut rows_i, &mut cols, &mut vals);
            }
        }
        for (k, &u) in converters.iter().enumerate() {
            put(row, c_at(k, t), -units.list[u].conv_efficiency, &mut rows_i, &mut cols, &mut vals);
        }
        for (p, pair) in pairs.iter().enumerate() {
            let cost = units.list[pair.unit].metal_cost / pair.cost;
            if cost != 0.0 {
                put(row, x_at(p, t), cost, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(dt_of(t) * const_metal);
        metal_rows.push(row);
        row += 1;
    }
    // 5. Energy: e[t+1] - e[t] - dt sum er n[u, t] + sum c[k, t] + sum ce/cost x[p, t] <= dt const_energy.
    let mut energy_rows = Vec::new();
    for t in 0..buckets {
        put(row, e_at(t + 1), 1.0, &mut rows_i, &mut cols, &mut vals);
        put(row, e_at(t), -1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, &u) in types.iter().enumerate() {
            let rate = energy_rate(u);
            if rate != 0.0 {
                put(row, n_at(i, t), -dt_of(t) * rate, &mut rows_i, &mut cols, &mut vals);
            }
        }
        for k in 0..converters.len() {
            put(row, c_at(k, t), 1.0, &mut rows_i, &mut cols, &mut vals);
        }
        for (p, pair) in pairs.iter().enumerate() {
            let cost = units.list[pair.unit].energy_cost / pair.cost;
            if cost != 0.0 {
                put(row, x_at(p, t), cost, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(dt_of(t) * const_energy);
        energy_rows.push(row);
        row += 1;
    }
    // Storage: m[t] - sum sm n[u, t] <= base; the same for energy.
    for t in 0..t_n {
        put(row, m_at(t), 1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, &u) in types.iter().enumerate() {
            if units.list[u].metal_storage != 0.0 {
                put(row, n_at(i, t), -units.list[u].metal_storage, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(base_metal_storage);
        row += 1;
        put(row, e_at(t), 1.0, &mut rows_i, &mut cols, &mut vals);
        for (i, &u) in types.iter().enumerate() {
            if units.list[u].energy_storage != 0.0 {
                put(row, n_at(i, t), -units.list[u].energy_storage, &mut rows_i, &mut cols, &mut vals);
            }
        }
        b.push(base_energy_storage);
        row += 1;
    }
    // 6. Conversion: c[k, t] - dt cc n[k, t] <= 0.
    for (k, &u) in converters.iter().enumerate() {
        for t in 0..buckets {
            put(row, c_at(k, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            put(row, n_at(index(u).unwrap(), t), -dt_of(t) * units.list[u].conv_capacity, &mut rows_i, &mut cols, &mut vals);
            b.push(0.0);
            row += 1;
        }
    }
    // 7. Spots and upgrades.
    let basics: Vec<usize> = (0..types.len()).filter(|&i| is_extractor(types[i]) && !is_upgrade(types[i])).collect();
    let upgrades: Vec<usize> = (0..types.len()).filter(|&i| is_upgrade(types[i])).collect();
    let basics_standing: f64 = basics.iter().map(|&i| n0[i]).sum();
    for t in 0..t_n {
        if !basics.is_empty() {
            for &i in &basics {
                put(row, n_at(i, t), 1.0, &mut rows_i, &mut cols, &mut vals);
            }
            b.push((scenario.spots.len() as f64).max(basics_standing));
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
    // 8. Nothing negative.
    for v in 0..nvars {
        put(row, v, -1.0, &mut rows_i, &mut cols, &mut vals);
        b.push(0.0);
        row += 1;
    }
    let nrows = row;

    // The objective, minimized: minus the metal made (and the terminal income).
    let mut q = vec![0.0; nvars];
    for (i, &u) in types.iter().enumerate() {
        let rate = metal_rate(u);
        for t in 0..buckets {
            q[n_at(i, t)] -= dt_of(t) * rate;
        }
        q[n_at(i, buckets)] -= options.terminal * rate;
    }
    for (k, &u) in converters.iter().enumerate() {
        for t in 0..buckets {
            q[c_at(k, t)] -= units.list[u].conv_efficiency;
        }
    }

    let a = CscMatrix::new_from_triplets(nrows, nvars, rows_i, cols, vals);
    let p = CscMatrix::<f64>::zeros((nvars, nvars));
    let cones = [SupportedConeT::ZeroConeT(equalities), SupportedConeT::NonnegativeConeT(nrows - equalities)];
    // Four digits are plenty for a plan the hands round to whole units; the defaults' eight cost half the iterations.
    let settings = DefaultSettings { verbose: false, tol_gap_abs: 1e-4, tol_gap_rel: 1e-5, tol_feas: 1e-6, ..Default::default() };
    let mut solver = DefaultSolver::new(&p, &q, &a, &b, &cones, settings);
    solver.solve();
    let solution = &solver.solution;
    let status = format!("{:?}", solution.status);
    if !matches!(solution.status, SolverStatus::Solved | SolverStatus::AlmostSolved) {
        return Err(format!("the program did not solve: {status}"));
    }
    let x = &solution.x;
    let counts: Vec<Vec<f64>> = (0..types.len()).map(|i| (0..t_n).map(|t| x[n_at(i, t)].max(0.0)).collect()).collect();
    let converted_at = |t: usize| -> f64 { converters.iter().enumerate().map(|(k, &u)| units.list[u].conv_efficiency * x[c_at(k, t.min(buckets - 1))].max(0.0)).sum::<f64>() / dt_of(t) };
    let metal_income: Vec<f64> = (0..t_n).map(|t| const_metal + types.iter().enumerate().map(|(i, &u)| metal_rate(u) * counts[i][t]).sum::<f64>() + converted_at(t)).collect();
    let energy_income: Vec<f64> = (0..t_n).map(|t| const_energy + types.iter().enumerate().map(|(i, &u)| energy_rate(u).max(0.0) * counts[i][t]).sum::<f64>()).collect();
    let made: f64 = (0..buckets).map(|t| dt_of(t) * metal_income[t]).sum();
    Ok(LpPlan {
        t0: state.t0,
        lengths,
        starts,
        buckets,
        types,
        counts,
        metal_income,
        energy_income,
        metal: (0..t_n).map(|t| x[m_at(t)]).collect(),
        energy: (0..t_n).map(|t| x[e_at(t)]).collect(),
        made,
        objective: -solution.obj_val,
        metal_price: metal_rows.iter().map(|r| solution.z[*r]).collect(),
        energy_price: energy_rows.iter().map(|r| solution.z[*r]).collect(),
        power_price: power_rows.iter().map(|(bt, rows)| (*bt, rows.iter().map(|r| solution.z[*r]).collect())).collect(),
        flows: pairs
            .iter()
            .enumerate()
            .map(|(p, pair)| Flow { builder: pair.builder, unit: pair.unit, kind: pair.kind, cost: pair.cost, per_bucket: (0..buckets).map(|t| x[x_at(p, t)].max(0.0)).collect() })
            .filter(|f| f.per_bucket.iter().sum::<f64>() > 1e-6)
            .collect(),
        solve_time: began.elapsed().as_secs_f64(),
        status,
        variables: nvars,
        rows: nrows,
    })
}
