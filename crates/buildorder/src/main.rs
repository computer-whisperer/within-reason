use std::fmt::Write as _;

use buildorder::anneal::{anneal_restarts, Objective, Palette, Search, Contact};
use std::sync::Arc;

use buildorder::game::{Game, Ground, Spot, Straight};
use buildorder::lp::{self, LpOptions, LpPlan};
use buildorder::plan::Plan;
use buildorder::record;
use buildorder::sim::{simulate, Outcome, Sample, Scenario, State, Wind};

const USAGE: &str = "usage: (the game, that is map, start, faction and unit numbers, comes from a match record's header, or from
                       a territory bot's dump of its Hello with --hello FILE in place of --game)
  buildorder optimize  --game RECORD.jsonl [--factory lab|vp] [--palette kit|roster] [--builders-only] [--all-spots] [--also silo,amd] [--minutes 10]
                       [--objective income|army|mix|tempo|expect|reach:N|made[:TERMINAL]|target:UNIT[:COUNT][@SECONDS][,round:UNIT..][,income:N@SECONDS]] [--contact WALK[,AT,WEIGHT]]
                       [--iterations 40000] [--restarts 8] [--seed 1] [--wind MEAN] [--detour X] [--factories 2] [--constructors 6] [--leash ELMOS]
                       [--no-nano] [--turret llt] [--nanos-to-factory] [--tax FROM:RATE,..] [--tax-energy 8] [--seed-chain UNIT[:COUNT]] [--csv FILE] [--plan-out FILE] [--json-out FILE]
                       (--detour X: open ground, every walk X straight lines, in place of the map's own ground; --also: unit types the
                       roster's rule leaves out; --builders-only: the factories make nothing but builders (the territory bot's economy);
                       reach:N: the fastest order to N metal a second; made[:T]: metal made over the horizon plus T seconds of the final
                       income, the territory bot's score; round:UNIT: a stockpile round, the first
                       missile; --tax: metal a second to soldiers from each time, M:SS or seconds, before any build; turrets help the
                       dearest frame in reach unless --nanos-to-factory)
  buildorder lp        --game RECORD.jsonl | --hello DUMP.json [--minutes 20] [--dt 10] [--mid-from 60] [--mid-dt 30] [--coarse-from 300] [--coarse-dt 60] [--terminal 0] [--energy-reserve 0.2] [--energy-margin 0] [--assist 2] [--help-factor 3]
                       [--nanos-per-factory 6] [--max-constructors 40 (per tier)] [--max-factories 4] [--all-spots] [--builders-only]
                       [--walk-extractor 65] [--walk-base 3] [--walk-factory 40] [--commander-walk-extractor 16] [--commander-walk-base 5] [--commander-reach 1500] [--rings 1] [--ring-overhead 5]
                       [--commander-walk-factory 9] [--affordable METAL] [--flows N] [--csv FILE]
                       (the planner as a linear program, docs/design/2026-10-20-territory-lp.md: the plan by minute with
                       the prices of metal, energy and build power; the csv in the simulator's format)
  buildorder simulate  --game RECORD.jsonl --plan FILE [--minutes ..] [--wind ..] [--detour X] [--help-frames] [--tax ..] [--csv FILE]
                       (plan words: unit names without the faction prefix, assist, help; help puts the builder's power on the dearest frame)
  buildorder walks     RECORD.jsonl... [--minutes 6]      (builders' ways between builds: recorded against predicted)
  buildorder calibrate RECORD.jsonl... [--minutes 10] [--from SECONDS] [--detour X] [--constant-wind] [--trace] [--csv-dir DIR]
                       (--from S: replay from the record's own state at second S with what was started after it)";

struct Args(Vec<String>);

impl Args {
    fn value(&self, flag: &str) -> Option<&str> {
        self.0.iter().position(|a| a == flag).and_then(|i| self.0.get(i + 1)).map(String::as_str)
    }
    fn text(&self, flag: &str, default: &str) -> String {
        self.value(flag).unwrap_or(default).to_string()
    }
    fn number<T: std::str::FromStr>(&self, flag: &str, default: T) -> T {
        self.value(flag).map_or(default, |v| v.parse().unwrap_or_else(|_| die(&format!("bad value for {flag}: {v}"))))
    }
    fn has(&self, flag: &str) -> bool {
        self.0.iter().any(|a| a == flag)
    }
    /// Arguments that are neither flags nor flag values.
    fn positional(&self, flags_without_value: &[&str]) -> Vec<&str> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.0.len() {
            let a = self.0[i].as_str();
            if a.starts_with("--") {
                i += if flags_without_value.contains(&a) { 1 } else { 2 };
            } else {
                out.push(a);
                i += 1;
            }
        }
        out
    }
}

fn die(message: &str) -> ! {
    eprintln!("{message}\n{USAGE}");
    std::process::exit(2)
}

/// The game of `--game`'s record, or of `--hello`'s dump: the territory bot writes `{"home": [x, z], "commander": ID,
/// "hello": {...}}` (`bots/territory`), so the oracle runs from the start the bot had.
fn game(args: &Args) -> Game {
    if let Some(path) = args.value("--hello") {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{path}: {e}")));
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|e| die(&format!("{path}: {e}")));
        let hello: bot_protocol::Hello = serde_json::from_value(dump["hello"].clone()).unwrap_or_else(|e| die(&format!("{path}: {e}")));
        let home = (dump["home"][0].as_f64().unwrap_or(0.0), dump["home"][1].as_f64().unwrap_or(0.0));
        let commander = bot_protocol::UnitDefId(dump["commander"].as_i64().unwrap_or(-1) as i32);
        return Game::from_hello(&hello, commander, home).unwrap_or_else(|| die(&format!("{path}: the commander's type is not in the unit table")));
    }
    let path = args.value("--game").unwrap_or_else(|| die("--game RECORD.jsonl is required (or --hello DUMP.json)"));
    record::read(path, 60.0).unwrap_or_else(|e| die(&e)).game
}

/// The map's own ground; with `--detour X`, open ground where every walk is X straight lines.
fn ground(game: &Game, args: &Args) -> Arc<dyn Ground> {
    match args.value("--detour") {
        Some(_) => Arc::new(Straight { detour: args.number("--detour", 1.05) }),
        None => game.ground(),
    }
}

fn scenario(game: &Game, args: &Args) -> Scenario {
    // `--all-spots`: every spot on the map, nearest first, in place of our half (the territory bot's `spots=all`
    // against an opponent that does nothing; the flagship's player took 78 of Comet's 80 in terr-flagship-nullai-1).
    let spots = if args.has("--all-spots") {
        let mut spots: Vec<Spot> = game.spots.iter().map(|(at, amount)| Spot { at: *at, metal: game.spot_metal(*amount) }).collect();
        spots.sort_by(|a, b| buildorder::game::distance(a.at, game.home).total_cmp(&buildorder::game::distance(b.at, game.home)));
        spots
    } else {
        game.own_half()
    };
    let mut scenario = game.scenario(spots, ground(game, args));
    scenario.wind = Wind::Constant(args.number("--wind", game.mean_wind()));
    scenario.constructors_default_to_extractors = true;
    scenario.commander_leash = args.number("--leash", f64::MAX);
    // `--help-frames`: construction turrets help the dearest frame in reach, else the first factory (optimize turns
    // it on unless `--nanos-to-factory`); `--tax FROM:RATE,...`: metal a second to soldiers from each time (M:SS or
    // seconds), before any build; `--tax-energy R`: energy a metal the tax draws.
    scenario.nanos_help_frames = args.has("--help-frames");
    scenario.tax = parse_tax(&args.text("--tax", ""));
    scenario.tax_energy_ratio = args.number("--tax-energy", scenario.tax_energy_ratio);
    scenario
}

/// `FROM:RATE,FROM:RATE`, the time as `M:SS` or seconds, sorted by time.
fn parse_tax(text: &str) -> Vec<(f64, f64)> {
    let mut schedule: Vec<(f64, f64)> = text
        .split(',')
        .filter(|e| !e.is_empty())
        .map(|entry| {
            let (from, rate) = entry.rsplit_once(':').unwrap_or_else(|| die(&format!("--tax: {entry:?} is not FROM:RATE")));
            let seconds = match from.split_once(':') {
                Some((m, s)) => 60.0 * m.parse::<f64>().unwrap_or_else(|_| die(&format!("--tax: {from:?} is not a time"))) + s.parse::<f64>().unwrap_or_else(|_| die(&format!("--tax: {from:?} is not a time"))),
                None => from.parse::<f64>().unwrap_or_else(|_| die(&format!("--tax: {from:?} is not a time"))),
            };
            (seconds, rate.parse::<f64>().unwrap_or_else(|_| die(&format!("--tax: {rate:?} is not metal a second"))))
        })
        .collect();
    schedule.sort_by(|a, b| a.0.total_cmp(&b.0));
    schedule
}

/// `--also NAME,NAME`: unit types offered to the roster by name (with or without the faction prefix).
fn also(game: &Game, args: &Args) -> Vec<usize> {
    args.text("--also", "")
        .split(',')
        .filter(|n| !n.is_empty())
        .map(|n| game.units.index(n).or_else(|| game.units.index(&format!("{}{n}", game.side()))).unwrap_or_else(|| die(&format!("--also: no unit type {n} in this game"))))
        .collect()
}

const CSV_HEADER: &str = "t,metal,energy,metal_income,energy_income,extractors,converters,build_power,factories,constructors,nanos,army_count,army_value,stall,metal_wasted,energy_wasted,rounds,tax_paid,tax_unpaid";

fn csv_row(s: &Sample) -> String {
    format!(
        "{},{:.0},{:.0},{:.2},{:.1},{},{},{:.0},{},{},{},{},{:.0},{:.2},{:.0},{:.0},{},{:.0},{:.0}",
        s.t, s.metal, s.energy, s.metal_income, s.energy_income, s.extractors, s.converters, s.build_power, s.factories,
        s.constructors, s.nanos, s.army_count, s.army_value, s.stall, s.metal_wasted, s.energy_wasted, s.rounds, s.tax_paid, s.tax_unpaid
    )
}

fn write_csv(path: &str, outcome: &Outcome) {
    let mut text = format!("{CSV_HEADER}\n");
    for sample in outcome.samples.iter().filter(|s| (s.t as u32).is_multiple_of(10)) {
        text.push_str(&csv_row(sample));
        text.push('\n');
    }
    std::fs::write(path, text).unwrap_or_else(|e| die(&format!("{path}: {e}")));
}

const MILESTONES: [f64; 8] = [3.0, 5.0, 8.0, 10.0, 15.0, 20.0, 25.0, 30.0];

/// Markdown rows: one per milestone minute the outcome reaches.
fn milestone_rows(label: &str, outcome: &Outcome) -> String {
    let mut out = String::new();
    for minute in MILESTONES {
        let Some(s) = outcome.samples.iter().find(|s| s.t == minute * 60.0) else { continue };
        let income = outcome.mean_metal_income(s.t, 30.0);
        let stall: f64 = outcome.samples.iter().filter(|x| x.t <= s.t).map(|x| x.stall).sum::<f64>() / s.t;
        let _ = writeln!(
            out,
            "| {label} | {minute:.0} | {} | {income:.1} | {:.0} | {:.0} | {} | {} | {} | {} | {:.0} | {} | {:.0} / {:.0} | {} | {:.2} | {:.0} / {:.0} |",
            s.extractors, s.energy_income, s.build_power, s.factories, s.constructors, s.nanos, s.converters, s.army_value,
            s.army_count, s.tax_paid, s.tax_unpaid, s.rounds, stall, s.metal_wasted, s.energy_wasted
        );
    }
    out
}

const MILESTONE_HEADER: &str = "| run | min | mex | metal/s | energy/s | build power | labs | cons | nanos | conv | army metal | army n | tax paid / owed | rounds | mean stall | wasted M / E |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";

fn optimize(args: &Args) {
    let game = game(args);
    let units = &game.units;
    let factory = args.text("--factory", "lab");
    let objective_text = args.text("--objective", "mix");
    let mut objective = if objective_text.starts_with("target") { Objective::parse_target(&objective_text, units).unwrap_or_else(|e| die(&e)) } else { Objective::parse(&objective_text).unwrap_or_else(|| die("unknown objective")) };
    // `--contact WALK[,AT,WEIGHT]`: the tempo objective's first-contact term (docs/design/2026-09-20-rush-benchmark.md).
    if let Objective::Tempo { contact, .. } | Objective::Expect { contact, .. } = &mut objective
        && args.has("--contact")
    {
        let parts: Vec<f64> = args.text("--contact", "").split(',').filter_map(|p| p.parse().ok()).collect();
        let walk = *parts.first().unwrap_or_else(|| die("--contact WALK[,AT,WEIGHT]"));
        *contact = Some(Contact { walk, at: parts.get(1).copied().unwrap_or(150.0), weight: parts.get(2).copied().unwrap_or(2.0), window: 120.0 });
    }
    let minutes: f64 = args.number("--minutes", 10.0);
    let mut scenario = scenario(&game, args);
    // The search plays the late game as the §4.7 rule has it; a replay keeps the recorded games' turret behaviour.
    scenario.nanos_help_frames = !args.has("--nanos-to-factory");
    let factory_unit = game.factory(&factory).unwrap_or_else(|| die(&format!("the commander builds no {factory}")));
    let turret = units.index(&format!("{}{}", game.side(), args.text("--turret", "llt")));
    let water = game.terrain.heights.iter().any(|h| *h < 0);
    let mut palette = match args.text("--palette", "kit").as_str() {
        "roster" => Palette::roster(units, game.commander, turret, water, &also(&game, args)),
        _ => Palette::new(units, game.commander, factory_unit, !args.has("--no-nano"), turret),
    };
    if args.has("--builders-only") {
        palette.factory.retain(|i| matches!(i, buildorder::plan::Item::Build(u) if units.list[*u].role == buildorder::units::Role::Builder));
    }
    let (factories_n, constructors_n) = (args.number::<usize>("--factories", 2), args.number::<usize>("--constructors", 6));
    let wind_mean = match scenario.wind { buildorder::sim::Wind::Constant(w) => w, _ => 10.0 };
    // `--seed-chain UNIT[:COUNT]`: start from the seed with the production chain to this unit laid through it, whatever
    // the objective (a reach objective from the tier-2 extractor's chain finds the mass upgrade the plain seed does not).
    let start = match (args.value("--seed-chain"), &objective) {
        (Some(spec), _) => {
            let (name, count) = spec.split_once(':').map_or((spec, 4), |(n, c)| (n, c.parse().unwrap_or_else(|_| die("--seed-chain UNIT[:COUNT]"))));
            let unit = units.index(name).or_else(|| units.index(&format!("{}{name}", game.side()))).unwrap_or_else(|| die(&format!("--seed-chain: no unit type {name} in this game")));
            Some(palette.chain_seed(units, game.commander, &[(unit, count)], factories_n, constructors_n, wind_mean))
        }
        (None, Objective::Target { goals, .. }) => {
            let goals: Vec<(usize, usize)> = goals.iter().map(|g| (g.unit, g.count.unwrap_or(6))).collect();
            Some(palette.chain_seed(units, game.commander, &goals, factories_n, constructors_n, wind_mean))
        }
        _ => None,
    };
    let search = Search {
        objective,
        horizon: minutes * 60.0,
        iterations: args.number("--iterations", 40_000),
        seed: args.number("--seed", 1),
        factories: factories_n,
        constructors: constructors_n,
        hot: args.number("--hot", 0.02),
        start,
    };
    let found = anneal_restarts(units, &scenario, &State::start(&scenario), &palette, &search, args.number("--restarts", 8));
    println!("# {} {factory} from {:.0},{:.0}, objective {} at {minutes} min, score {:.1}", game.side(), game.home.0, game.home.1, search.objective.name(), found.score);
    print!("{}", found.plan.to_text(units));
    print!("\n{MILESTONE_HEADER}{}", milestone_rows("best", &found.outcome));
    if let Some(path) = args.value("--csv") {
        write_csv(path, &found.outcome);
    }
    if let Some(path) = args.value("--plan-out") {
        std::fs::write(path, found.plan.to_text(units)).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    }
    // `--json-out FILE`: the run for a generator (run/frontier_set.py): the plan, the first finish of every unit type,
    // the first stockpile round of every stockpiler, when the 30-second income first reached 50 / 100 / 200, and the
    // state at every whole minute.
    if let Some(path) = args.value("--json-out") {
        let o = &found.outcome;
        let mut first_finished = std::collections::BTreeMap::new();
        for f in &o.finished {
            first_finished.entry(units.list[f.unit].name.as_str()).or_insert(f.t);
        }
        let mut first_round = std::collections::BTreeMap::new();
        for (t, unit) in &o.stockpiled {
            first_round.entry(units.list[*unit].name.as_str()).or_insert(*t);
        }
        let reach = |n: f64| o.samples.iter().filter(|s| s.t >= 30.0).find(|s| o.mean_metal_income(s.t, 30.0) >= n).map(|s| s.t);
        let by_minute: Vec<serde_json::Value> = o
            .samples
            .iter()
            .filter(|s| (s.t as u32).is_multiple_of(60))
            .map(|s| serde_json::json!({ "t": s.t, "income": o.mean_metal_income(s.t, 30.0), "energy_income": s.energy_income, "extractors": s.extractors, "constructors": s.constructors, "nanos": s.nanos, "converters": s.converters, "build_power": s.build_power, "army": s.army_value, "tax_paid": s.tax_paid, "tax_unpaid": s.tax_unpaid, "rounds": s.rounds, "metal_wasted": s.metal_wasted, "energy_wasted": s.energy_wasted, "stall": s.stall }))
            .collect();
        let out = serde_json::json!({
            "side": game.side(), "home": [game.home.0, game.home.1], "spots": scenario.spots.len(), "objective": objective_text, "minutes": minutes,
            "score": found.score, "tax": scenario.tax, "plan": found.plan.to_text(units), "first_finished": first_finished, "first_round": first_round,
            "reach": { "50": reach(50.0), "100": reach(100.0), "200": reach(200.0) }, "by_minute": by_minute,
        });
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap()).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    }
}

/// The linear program from the empty start (docs/design/2026-10-20-territory-lp.md): the plan by minute, the counts at
/// the end, the prices.
fn linear_program(args: &Args) {
    let game = game(args);
    let units = &game.units;
    let scenario = scenario(&game, args);
    let water = game.terrain.heights.iter().any(|h| *h < 0);
    let mut palette = Palette::roster(units, game.commander, None, water, &also(&game, args));
    if args.has("--builders-only") {
        palette.factory.retain(|i| matches!(i, buildorder::plan::Item::Build(u) if units.list[*u].role == buildorder::units::Role::Builder));
    }
    // `--affordable M`: the types costing at most M metal, as the territory bot offers the program what its hands
    // could start now (the stock and ninety seconds of income).
    let affordable: f64 = args.number("--affordable", f64::INFINITY);
    let offer: Vec<usize> = palette.mobile.iter().chain(palette.factory.iter()).filter_map(|i| match i { buildorder::plan::Item::Build(u) => Some(*u), _ => None }).filter(|u| units.list[*u].metal_cost <= affordable).collect();
    let minutes: f64 = args.number("--minutes", 20.0);
    let options = LpOptions {
        dt: args.number("--dt", 10.0),
        mid_from: args.number("--mid-from", 60.0),
        mid_dt: args.number("--mid-dt", 30.0),
        coarse_from: args.number("--coarse-from", 300.0),
        coarse_dt: args.number("--coarse-dt", 60.0),
        horizon: minutes * 60.0,
        max_constructors: args.number("--max-constructors", 40.0),
        max_factories: args.number("--max-factories", 4.0),
        terminal: args.number("--terminal", 0.0),
        energy_reserve: args.number("--energy-reserve", 0.2),
        energy_margin: args.number("--energy-margin", 0.0),
        assist_factor: args.number("--assist", 2.0),
        help_factor: args.number("--help-factor", 3.0),
        nanos_per_factory: args.number("--nanos-per-factory", 6.0),
        walk_extractor: args.number("--walk-extractor", 65.0),
        walk_base: args.number("--walk-base", 3.0),
        walk_factory: args.number("--walk-factory", 40.0),
        commander_walk_extractor: args.number("--commander-walk-extractor", 16.0),
        commander_walk_base: args.number("--commander-walk-base", 5.0),
        commander_walk_factory: args.number("--commander-walk-factory", 9.0),
        commander_reach: args.number("--commander-reach", 1500.0),
        rings: args.number("--rings", 1.0) as usize,
        ring_overhead: args.number("--ring-overhead", 5.0),
    };
    let plan = lp::solve(units, &scenario, &State::start(&scenario), &offer, &options).unwrap_or_else(|e| die(&e));
    println!(
        "# {} from {:.0},{:.0}: {} types, {} buckets ({} s to {} s), {} spots, {} variables, {} rows, {} in {:.0} ms; metal made {:.0}, objective {:.0}",
        game.side(), game.home.0, game.home.1, plan.types.len(), plan.buckets, plan.lengths[0], plan.lengths[plan.buckets - 1], scenario.spots.len(), plan.variables, plan.rows, plan.status, plan.solve_time * 1000.0, plan.made, plan.objective
    );
    let basic = units.basic_extraction();
    let count = |plan: &LpPlan, t: usize, test: &dyn Fn(&buildorder::units::Unit) -> bool| -> f64 { plan.columns.iter().enumerate().filter(|(_, c)| test(&units.list[c.unit])).map(|(i, _)| plan.counts[i][t]).sum() };
    let class_name = |class: buildorder::lp::Class| match class {
        buildorder::lp::Class::Whole => String::new(),
        buildorder::lp::Class::Frames => "(frames)".to_string(),
        buildorder::lp::Class::Ring(k) => format!("[r{k}]"),
    };
    println!("rings: {}", plan.rings.iter().enumerate().map(|(k, r)| format!("r{k} {} spots to {:.0}, mean {:.0} elmos, {:.2} metal/s", r.spots, r.bound, r.distance, r.metal)).collect::<Vec<_>>().join("; "));
    println!("| min | metal/s | energy/s | mex | upgrades | cons | fac | nanos | conv | energy bldgs | metal | energy | price of metal | of energy |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    // The bucket at each whole minute (the latest starting at or before it).
    let minutes_shown: Vec<(usize, usize)> = (0..=minutes as usize).map(|m| (m, plan.bucket_at(plan.t0 + m as f64 * 60.0))).collect();
    for (minute, t) in &minutes_shown {
        let t = *t;
        let price = |v: &Vec<f64>| v.get(t.min(plan.buckets - 1)).copied().unwrap_or(0.0);
        println!(
            "| {} | {:.1} | {:.0} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.0} | {:.0} | {:.2} | {:.4} |",
            minute, plan.metal_income[t], plan.energy_income[t],
            count(&plan, t, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
            count(&plan, t, &|u| u.extracts_metal > basic * 1.5),
            count(&plan, t, &|u| u.role == buildorder::units::Role::Builder),
            count(&plan, t, &|u| u.role == buildorder::units::Role::Factory),
            count(&plan, t, &|u| u.role == buildorder::units::Role::Nano),
            count(&plan, t, &|u| u.conv_capacity > 0.0),
            count(&plan, t, &|u| u.role == buildorder::units::Role::Eco && u.extracts_metal == 0.0 && u.conv_capacity == 0.0 && (u.energy_make > 0.0 || u.wind_cap > 0.0)),
            plan.metal[t], plan.energy[t], price(&plan.metal_price), price(&plan.energy_price)
        );
    }
    let mut at_end: Vec<(f64, String)> = plan.columns.iter().enumerate().map(|(i, c)| (plan.counts[i][plan.buckets], format!("{}{}", units.list[c.unit].name, class_name(c.class)))).filter(|(n, _)| *n > 0.05).collect();
    at_end.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("at the end: {}", at_end.iter().map(|(n, name)| format!("{name} {n:.1}")).collect::<Vec<_>>().join(", "));
    for (bt, prices) in &plan.power_price {
        let shown: Vec<String> = minutes_shown.iter().step_by(2).map(|(_, t)| format!("{:.3}", prices.get(*t).copied().unwrap_or(0.0))).collect();
        println!("price of a build-power-second of {} every two minutes: {}", units.list[*bt].name, shown.join(" "));
    }
    // `--flows N`: each builder type's power on each unit type in the first N buckets, in units of the unit's cost to
    // that builder: how the program sequences the opening (which the hands follow by credits, T-DISPATCH).
    if let Some(n) = args.value("--flows").and_then(|v| v.parse::<usize>().ok()) {
        let n = n.min(plan.buckets);
        println!("flows in the first {n} buckets ({} s), units of the unit's cost to the builder:", plan.starts[n]);
        let mut rows: Vec<(f64, String)> = plan
            .flows
            .iter()
            .map(|f| {
                let per: Vec<String> = f.per_bucket.iter().take(n).map(|x| format!("{:.2}", x / f.cost)).collect();
                let total: f64 = f.per_bucket.iter().take(n).sum::<f64>() / f.cost;
                (total, format!("  {} -> {}{} ({:?}, cost {:.0}): {} = {total:.2}", units.list[f.builder].name, units.list[f.unit].name, class_name(f.class), f.kind, f.cost, per.join(" ")))
            })
            .filter(|(total, _)| *total > 0.005)
            .collect();
        rows.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, line) in rows {
            println!("{line}");
        }
    }
    if let Some(path) = args.value("--csv") {
        let mut text = format!("{CSV_HEADER}\n");
        for t in 0..=plan.buckets {
            let bp: f64 = units.list[game.commander].worker_time + plan.columns.iter().enumerate().map(|(i, c)| units.list[c.unit].worker_time * plan.counts[i][t]).sum::<f64>();
            let _ = writeln!(
                text,
                "{},{:.0},{:.0},{:.2},{:.1},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},0,0,1.00,0,0,0,0,0",
                plan.t0 + plan.starts[t], plan.metal[t], plan.energy[t], plan.metal_income[t], plan.energy_income[t],
                count(&plan, t, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
                count(&plan, t, &|u| u.conv_capacity > 0.0), bp,
                count(&plan, t, &|u| u.role == buildorder::units::Role::Factory),
                count(&plan, t, &|u| u.role == buildorder::units::Role::Builder),
                count(&plan, t, &|u| u.role == buildorder::units::Role::Nano)
            );
        }
        std::fs::write(path, text).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    }
}

fn run_plan(args: &Args) {
    let game = game(args);
    let units = &game.units;
    let path = args.value("--plan").unwrap_or_else(|| die("--plan FILE is required"));
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    let plan = Plan::from_text(&text, game.side(), units).unwrap_or_else(|e| die(&e));
    let scenario = scenario(&game, args);
    let outcome = simulate(units, &scenario, &State::start(&scenario), &plan, args.number("--minutes", 10.0) * 60.0);
    print!("{MILESTONE_HEADER}{}", milestone_rows(path, &outcome));
    for done in &outcome.finished {
        println!("{:6.1} s  {}  (queue {})", done.t, units.list[done.unit].name, done.queue);
    }
    for (t, unit) in &outcome.stockpiled {
        println!("{t:6.1} s  {} round stockpiled", units.list[*unit].name);
    }
    if let Some(path) = args.value("--csv") {
        write_csv(path, &outcome);
    }
}

fn calibrate(args: &Args) {
    let minutes: f64 = args.number("--minutes", 10.0);
    let from: f64 = args.number("--from", 0.0);
    let records = args.positional(&["--constant-wind", "--trace"]);
    if records.is_empty() {
        die("calibrate needs at least one record");
    }
    // Per minute: sums of |sim - observed| and of observed. Once over every record, once over the records that had
    // lost at most `QUIET_LOSSES` units by that minute (the simulator knows no losses, so only those test its arithmetic).
    const QUIET_LOSSES: u32 = 2;
    let mut errors = vec![[[0.0f64; 8]; 2]; minutes as usize + 1];
    // Per minute, per record: recorded (extractors alive, metal/s, energy/s, army metal built), for the medians.
    let mut recorded = vec![Vec::<[f64; 4]>::new(); minutes as usize + 1];
    for path in &records {
        let replay = record::read(path, minutes * 60.0).unwrap_or_else(|e| die(&e));
        let (game, units) = (&replay.game, &replay.game.units);
        let mut scenario = game.scenario(game.own_half(), ground(game, args));
        if !args.has("--constant-wind") {
            let mut last = game.mean_wind();
            scenario.wind = Wind::Trace(std::iter::once(last).chain(replay.wind.iter().map(|w| { last = w.unwrap_or(last); last })).collect());
        }
        let (state, plan) = if from > 0.0 {
            replay.state_at(from, &scenario).unwrap_or_else(|| die(&format!("{path}: no sample at {from} s")))
        } else {
            (State::start(&scenario), replay.plan.clone())
        };
        let outcome = simulate(units, &scenario, &state, &plan, minutes * 60.0 - state.t0);
        println!("\n## {path}\nside {}, home {:.0},{:.0}; builds outside the unit table: {:?}; started but never finished (left out): {}",
            game.side(), game.home.0, game.home.1, replay.unknown, replay.abandoned);
        // The record's army metal counts from the start; from a snapshot the simulator counts from there.
        let army_before = replay.observed.iter().find(|o| (o.t - from).abs() < 1e-6).map_or(0.0, |o| o.army_value);
        if from > 0.0 {
            println!("from {from} s: {} standing, {} builders ({} on a job), metal {:.0} energy {:.0}; army metal built counted from there", state.standing.len(), state.builders.len(), state.builders.iter().filter(|b| b.job.is_some()).count(), state.metal, state.energy);
        }
        let sim_factory = outcome.finished.iter().find(|f| units.list[f.unit].role == buildorder::units::Role::Factory).map(|f| f.t);
        println!("first factory finished: recorded {:?} s, simulated {:?} s", replay.first_factory_finished, sim_factory);
        println!("| min | mex alive (built) rec / sim | metal/s rec/sim | energy/s rec/sim | cons rec/sim | army metal built rec/sim | units lost (rec) |\n|---|---|---|---|---|---|---|");
        for minute in 1..=minutes as usize {
            let t = minute as f64 * 60.0;
            if t <= from {
                continue;
            }
            let (Some(o), Some(s)) = (replay.observed.iter().find(|o| o.t == t), outcome.samples.iter().find(|s| s.t == t)) else { continue };
            // Both incomes as 30 s means: wind and converters make single seconds noisy.
            let mean = |pick: &dyn Fn(f64) -> Option<f64>| { let v: Vec<f64> = (0..30).filter_map(|k| pick(t - k as f64)).collect(); v.iter().sum::<f64>() / v.len().max(1) as f64 };
            let om = mean(&|at| replay.observed.iter().find(|o| o.t == at).map(|o| o.metal_income));
            let oe = mean(&|at| replay.observed.iter().find(|o| o.t == at).map(|o| o.energy_income));
            let sm = mean(&|at| outcome.samples.iter().find(|s| s.t == at).map(|s| s.metal_income));
            let se = mean(&|at| outcome.samples.iter().find(|s| s.t == at).map(|s| s.energy_income));
            let army = o.army_value - army_before;
            println!("| {minute} | {} ({}) / {} | {om:.1} / {sm:.1} | {oe:.0} / {se:.0} | {} / {} | {:.0} / {:.0} | {} |",
                o.extractors, o.extractors_built, s.extractors, o.constructors, s.constructors, army, s.army_value, o.losses);
            recorded[minute].push([o.extractors as f64, om, oe, army]);
            for e in errors[minute].iter_mut().take(if o.losses <= QUIET_LOSSES { 2 } else { 1 }) {
            e[0] += (s.extractors as f64 - o.extractors as f64).abs(); e[1] += o.extractors as f64;
            e[2] += (sm - om).abs(); e[3] += om;
            e[4] += (s.army_value - army).abs(); e[5] += army;
            e[6] += sm - om; e[7] += 1.0;
            }
        }
        if args.has("--trace") {
            print!("replayed plan:\n{}", plan.to_text(units));
            for done in &outcome.finished {
                println!("{:6.1} s  {}  (queue {})", done.t, units.list[done.unit].name, done.queue);
            }
        }
        if let Some(dir) = args.value("--csv-dir") {
            let name = path.trim_end_matches("/record-0.jsonl").rsplit('/').take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("-");
            let mut text = String::from("t,rec_metal_income,sim_metal_income,rec_energy_income,sim_energy_income,rec_extractors,sim_extractors,rec_army_value,sim_army_value,rec_losses,rec_metal,sim_metal,rec_energy,sim_energy\n");
            for (o, s) in replay.observed.iter().zip(&outcome.samples).filter(|(o, _)| (o.t as u32).is_multiple_of(10)) {
                let _ = writeln!(text, "{},{:.2},{:.2},{:.1},{:.1},{},{},{:.0},{:.0},{},{:.0},{:.0},{:.0},{:.0}", o.t, o.metal_income, s.metal_income, o.energy_income, s.energy_income, o.extractors, s.extractors, o.army_value, s.army_value, o.losses, o.metal, s.metal, o.energy, s.energy);
            }
            let file = format!("{dir}/calibration-{name}.csv");
            std::fs::write(&file, text).unwrap_or_else(|e| die(&format!("{file}: {e}")));
        }
    }
    println!("\n## What our bot achieved: median over {} records\n| min | mex alive | metal/s | energy/s | army metal built |\n|---|---|---|---|---|", records.len());
    for (minute, rows) in recorded.iter().enumerate().filter(|(_, rows)| !rows.is_empty()) {
        let median = |k: usize| { let mut v: Vec<f64> = rows.iter().map(|r| r[k]).collect(); v.sort_by(f64::total_cmp); (v[(v.len() - 1) / 2] + v[v.len() / 2]) / 2.0 };
        println!("| {minute} | {:.1} | {:.1} | {:.0} | {:.0} |", median(0), median(1), median(2), median(3));
    }
    for (which, title) in ["all records", "only records with at most 2 units lost by that minute"].iter().enumerate() {
    println!("\n## Summary, {title}: mean absolute error (and as a share of the recorded mean); metal/s bias = sim - rec");
    println!("| min | records | mex | metal/s | metal/s bias | army metal built |\n|---|---|---|---|---|---|");
    for (minute, e) in errors.iter().map(|e| e[which]).enumerate().skip(1).filter(|(_, e)| e[7] > 0.0) {
        let n = e[7];
        let share = |err: f64, total: f64| if total > 0.0 { 100.0 * err / total } else { 0.0 };
        println!("| {minute} | {n:.0} | {:.1} ({:.0} %) | {:.1} ({:.0} %) | {:+.1} | {:.0} ({:.0} %) |",
            e[0] / n, share(e[0], e[1]), e[2] / n, share(e[2], e[3]), e[6] / n, e[4] / n, share(e[4], e[5]));
    }
    }
}

/// Every mobile builder's way from one build to the next in the records' first minutes: the seconds it took against
/// what the simulator charges, over the map's own ground and over open ground, by how far the walk was.
fn walks(args: &Args) {
    let minutes: f64 = args.number("--minutes", 6.0);
    const BINS: [(&str, f64, f64); 5] = [("first build of the game", 0.0, 0.0), ("site in reach", 0.0, 0.0), ("walk under 300", 0.0, 300.0), ("walk 300-800", 300.0, 800.0), ("walk over 800", 800.0, f64::MAX)];
    // Per bin: (recorded, predicted on the map's ground, predicted on open ground, walked / straight).
    let mut rows: Vec<Vec<[f64; 4]>> = vec![Vec::new(); BINS.len()];
    for path in args.positional(&[]) {
        let replay = record::read(path, minutes * 60.0).unwrap_or_else(|e| die(&e));
        let game = &replay.game;
        let on_map = game.scenario(Vec::new(), game.ground());
        let open = game.scenario(Vec::new(), Arc::new(Straight { detour: 1.05 }));
        for trip in &replay.trips {
            let unit = &game.units.list[trip.builder];
            let beyond = buildorder::game::distance(trip.from, trip.to) - unit.build_distance - on_map.reach_bonus;
            // Where the builder stood is known only as "within reach of its last site": predict from the site itself.
            let predicted = |sc: &Scenario| sc.trip(trip.from, trip.to, unit.build_distance, unit.speed).0;
            let bin = if trip.first { 0 } else if beyond <= 0.0 { 1 } else { BINS.iter().position(|b| b.2 > 0.0 && beyond >= b.1 && beyond < b.2).unwrap() };
            let ratio = on_map.ground.walk(trip.from, trip.to) / buildorder::game::distance(trip.from, trip.to).max(1.0);
            rows[bin].push([trip.took, predicted(&on_map), predicted(&open), ratio]);
        }
    }
    println!("| way | trips | recorded s (median) | predicted, map's ground | predicted, open ground | median error map / open | walked over straight (median, max) |\n|---|---|---|---|---|---|---|");
    for ((label, _, _), trips) in BINS.iter().zip(&rows).filter(|(_, t)| !t.is_empty()) {
        let median = |pick: &dyn Fn(&[f64; 4]) -> f64| { let mut v: Vec<f64> = trips.iter().map(pick).collect(); v.sort_by(f64::total_cmp); v[v.len() / 2] };
        let longest = trips.iter().map(|t| t[3]).fold(0.0, f64::max);
        println!("| {label} | {} | {:.1} | {:.1} | {:.1} | {:+.1} / {:+.1} | {:.2}, {:.2} |", trips.len(), median(&|t| t[0]), median(&|t| t[1]), median(&|t| t[2]),
            median(&|t| t[1] - t[0]), median(&|t| t[2] - t[0]), median(&|t| t[3]), longest);
    }
}

fn main() {
    let args = Args(std::env::args().skip(1).collect());
    match args.0.first().map(String::as_str) {
        Some("optimize") => optimize(&args),
        Some("simulate") => run_plan(&args),
        Some("lp") => linear_program(&args),
        Some("calibrate") => calibrate(&args),
        Some("walks") => walks(&args),
        _ => die("missing or unknown command"),
    }
}
