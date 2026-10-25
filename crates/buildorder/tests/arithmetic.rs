//! The simulator's arithmetic against hand calculations from the unit numbers of a real game: the fixture is the
//! header of a Quicksilver record (2026-09-20) cut down to both factions' tier-1 land units, and its commander.

use std::sync::Arc;

use buildorder::anneal::{anneal, Goal, Objective, Palette, Search};
use buildorder::game::{Game, Spot, Straight};
use buildorder::plan::{Item, Plan, Step};
use buildorder::sim::{simulate, Outcome, Scenario, Standing, State, StateBuilder};
use buildorder::units::{Role, Units};

const HOME: (f64, f64) = (1000.0, 1000.0);

fn game() -> Game {
    buildorder::record::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/quicksilver-nw.jsonl"), 60.0).unwrap().game
}

/// No walking overheads and a fine step, so that times can be checked against `buildtime / workertime`.
fn bare(game: &Game) -> Scenario {
    let spots = [(1000.0, 1000.0), (1000.0, 3000.0)].map(|at| Spot { at, metal: 2.0 }).to_vec();
    let mut scenario = Scenario::new(game.commander, HOME, spots, Arc::new(Straight { detour: 1.0 }), 10.0);
    scenario.dt = 0.1;
    scenario.reach_bonus = 0.0;
    scenario.mobile_overhead = 0.0;
    scenario.walk_overhead = 0.0;
    scenario.factory_overhead = 0.0;
    scenario
}

/// The plan from the empty start.
fn run(units: &Units, scenario: &Scenario, plan: &Plan, seconds: f64) -> Outcome {
    simulate(units, scenario, &State::start(scenario), plan, seconds)
}

fn at_home(units: &Units, names: &[&str]) -> Vec<Step> {
    names.iter().map(|n| Step { item: Item::Build(units.index(n).unwrap()), site: Some(HOME) }).collect()
}

fn finish_time(units: &Units, outcome: &Outcome, name: &str) -> f64 {
    outcome.finished.iter().find(|f| units.list[f.unit].name == name).unwrap_or_else(|| panic!("{name} never finished")).t
}

#[test]
fn the_engine_reports_the_documented_numbers() {
    let game = game();
    let units = &game.units;
    let mex = units.get("armmex");
    assert_eq!((mex.metal_cost, mex.energy_cost, mex.build_time), (50.0, 500.0, 1800.0));
    assert_eq!(mex.energy_make, -3.0, "extractor upkeep");
    assert_eq!(units.get("armsolar").energy_make, 20.0, "solars produce through a negative upkeep");
    assert_eq!(units.get("armcom").worker_time, 300.0);
    assert_eq!(units.get("armnanotc").worker_time, 200.0);
    assert_eq!(units.get("armck").role, Role::Builder);
    assert_eq!(units.get("armlab").role, Role::Factory);
    assert_eq!(units.get("armpw").role, Role::Army);
    assert_eq!(units.get("armllt").role, Role::Turret);
    assert_eq!(units.get("armnanotc").role, Role::Nano);
    assert_eq!(units.list[game.commander].role, Role::Commander);
    assert!((game.spot_metal(game.spots[0].1) - 2.0).abs() < 0.01, "a Quicksilver spot pays 2 metal/s");
}

#[test]
fn build_time_is_buildtime_over_build_power_and_cost_is_paid_once() {
    let game = game();
    let units = &game.units;
    let mut plan = Plan::empty(1, 0);
    plan.commander = at_home(units, &["armwin"]);
    let outcome = run(units, &bare(&game), &plan, 20.0);
    // 1600 / 300 = 5.33 s, after the 0.1 s step in which the order is taken up.
    assert!((finish_time(units, &outcome, "armwin") - 5.43).abs() < 0.11);
    let end = outcome.last();
    // Metal: 1000 + 2/s from the commander (capped at 1000 until the turbine starts draining) - 40.
    assert!((end.metal - (1000.0 - 40.0 + 2.0 * 20.0)).abs() < 1.5, "metal {}", end.metal);
    assert!((end.energy_income - 40.0).abs() < 1e-9, "30 from the commander + 10 wind, got {}", end.energy_income);
}

#[test]
fn a_stalled_build_runs_at_the_speed_of_the_scarce_resource() {
    let game = game();
    let units = &game.units;
    let mut scenario = bare(&game);
    scenario.start_energy = 0.0;
    let mut plan = Plan::empty(1, 0);
    plan.commander = at_home(units, &["armmex"]);
    let outcome = run(units, &scenario, &plan, 30.0);
    // 500 E at the commander's 30 E/s: 16.7 s instead of 1800 / 300 = 6 s.
    assert!((finish_time(units, &outcome, "armmex") - 16.8).abs() < 0.3);
    let stalled = &outcome.samples[5];
    assert!((stalled.stall - 30.0 / (500.0 / 6.0)).abs() < 0.02, "stall factor {}", stalled.stall);
    assert_eq!(outcome.last().extractors, 1);
    assert!((outcome.last().metal_income - 4.0).abs() < 1e-9, "commander 2 + one spot at 2");
}

#[test]
fn walking_takes_distance_beyond_reach_over_speed() {
    let game = game();
    let units = &game.units;
    let mut plan = Plan::empty(1, 0);
    plan.commander = vec![Step { item: Item::Build(units.index("armmex").unwrap()), site: Some((1000.0, 3000.0)) }];
    let outcome = run(units, &bare(&game), &plan, 80.0);
    let expected = (2000.0 - 145.0) / 37.5 + 6.0;
    assert!((finish_time(units, &outcome, "armmex") - expected).abs() < 0.3);
}

#[test]
fn converters_burn_only_energy_above_three_quarters_of_storage() {
    let game = game();
    let units = &game.units;
    let mut plan = Plan::empty(1, 0);
    plan.commander = at_home(units, &["armmakr", "armsolar", "armsolar", "armsolar"]);
    let outcome = run(units, &bare(&game), &plan, 200.0);
    let built = finish_time(units, &outcome, "armmakr");
    // Right after the converter finishes, stored energy is below 750 (1150 was just spent): no conversion.
    let early = outcome.samples.iter().find(|s| s.t > built + 1.0).unwrap();
    assert!(early.energy < 750.0 && (early.metal_income - 2.0).abs() < 1e-9);
    // At the end 30 + 3 x 20 = 90 E/s comes in and the converter takes its full 70 E/s for 70 x 0.01429 = 1.0 M/s.
    let end = outcome.last();
    assert!((end.metal_income - 3.0).abs() < 0.01, "metal income {}", end.metal_income);
    assert!(end.energy >= 0.75 * 1250.0 - 1.0, "energy {} held at or above the converter level", end.energy);
}

#[test]
fn construction_turrets_add_their_build_power_to_the_factory() {
    let game = game();
    let units = &game.units;
    let time_of_second_pawn = |with_nano: bool| {
        let mut scenario = bare(&game);
        (scenario.start_metal, scenario.start_energy, scenario.base_storage) = (20_000.0, 20_000.0, 20_000.0);
        // The commander has no construction turret in its menu; a constructor does.
        let mut plan = Plan::empty(1, 1);
        plan.commander = at_home(units, &["armlab"]);
        plan.factories[0] = at_home(units, &[&["armck"][..], &["armpw"; 20][..]].concat());
        plan.constructors[0] = at_home(units, if with_nano { &["armnanotc"] } else { &[] });
        let outcome = run(units, &scenario, &plan, 400.0);
        let pawns: Vec<f64> = outcome.finished.iter().filter(|f| units.list[f.unit].name == "armpw").map(|f| f.t).collect();
        pawns[19] - pawns[18]
    };
    assert!((time_of_second_pawn(false) - 1650.0 / 150.0).abs() < 0.11);
    assert!((time_of_second_pawn(true) - 1650.0 / 350.0).abs() < 0.11);
}

#[test]
fn plan_text_round_trips() {
    let game = game();
    let units = &game.units;
    let text = "com: mex win lab@1952,1472 assist\nfac0: ck pw\ncon0: mex nanotc\n";
    let plan = Plan::from_text(text, "arm", units).unwrap();
    assert_eq!(plan.to_text(units), text);
    assert!(Plan::from_text("com: nosuchunit", "arm", units).is_err());
}

#[test]
fn annealing_is_deterministic_and_beats_its_seed_plan() {
    let game = game();
    let units = &game.units;
    let mut scenario = game.scenario(game.own_half(), game.ground());
    scenario.constructors_default_to_extractors = true;
    let palette = Palette::new(units, game.commander, game.factory("lab").unwrap(), true, units.index("armllt"));
    let search = Search { objective: Objective::Mix, horizon: 300.0, iterations: 1500, seed: 7, factories: 2, constructors: 6, hot: 0.02, start: None };
    let seed_outcome = run(units, &scenario, &palette.seed_plan(2, 6), 300.0);
    let (a, b) = (anneal(units, &scenario, &State::start(&scenario), &palette, &search), anneal(units, &scenario, &State::start(&scenario), &palette, &search));
    assert_eq!(a.plan, b.plan);
    assert_eq!(a.score, b.score);
    assert!(a.score > Objective::Mix.score(&game.units, &seed_outcome, 300.0));
    // The plan handed back reproduces its score when simulated afresh.
    let again = run(units, &scenario, &a.plan, 300.0);
    assert!((Objective::Mix.score(&game.units, &again, 300.0) - a.score).abs() < 1e-9);
}

/// A `target` with several goals (docs/design/2026-09-22-plan-search.md, decision 4 amended 2026-09-22 night): each
/// goal counts only up to its count, the chain seed lays every goal's chain, and a soldier no goal names is worth
/// nothing more than its shaping.
#[test]
fn target_goals_count_up_to_their_count_and_seed_every_chain() {
    let game = game();
    let units = &game.units;
    let (flash, bull, ck, vp) = (units.index("armflash").unwrap(), units.index("armbull").unwrap(), units.index("armck").unwrap(), units.index("armvp").unwrap());
    let parsed = Objective::parse_target("target armflash:4 by 3:30, armbull by 9:00", units).unwrap();
    assert_eq!(parsed.goals(), &[Goal { unit: flash, count: Some(4), by: Some(210.0), round: false }, Goal { unit: bull, count: None, by: Some(540.0), round: false }]);
    assert_eq!(Objective::parse_target("target:armbull@540", units).unwrap().goals(), &[Goal { unit: bull, count: None, by: Some(540.0), round: false }]);
    assert!(Objective::parse_target("target armnothing", units).unwrap_err().contains("no such unit"));
    assert!(Objective::parse_target("target armflash:many", units).unwrap_err().contains("whole number"));
    assert!(Objective::parse_target("target", units).unwrap_err().contains("not a target objective"));
    // The seed lays both chains: the commander's factory is the vehicle plant, which makes the Flashes and the
    // constructor that builds the advanced plant, which makes the Bulls.
    let palette = Palette::roster(units, game.commander, units.index("armllt"), false, &[]);
    let seed = palette.chain_seed(units, game.commander, &[(flash, 4), (bull, 6)], 2, 3, 3.0);
    assert!(seed.commander.iter().any(|s| s.item == Item::Build(vp)), "{seed:?}");
    assert_eq!(seed.factories[0].iter().filter(|s| s.item == Item::Build(flash)).count(), 4);
    assert_eq!(seed.factories[1].iter().filter(|s| s.item == Item::Build(bull)).count(), 6);
    assert!(seed.constructors[0].iter().any(|s| s.item == Item::Build(units.index("armavp").unwrap())));
    // Up to the count: two more Pawns than asked add less than one Pawn's metal, and with no count they add two.
    let scenario = game.scenario(game.own_half(), game.ground());
    let pw = units.index("armpw").unwrap();
    let list = |n: usize| {
        let mut plan = Plan::empty(1, 1);
        plan.commander = [Item::Build(units.extractor(game.commander).unwrap()); 2].into_iter().chain([Item::Build(units.index("armwin").unwrap()); 2]).chain([Item::Build(units.index("armlab").unwrap())]).map(|item| Step { item, site: None }).collect();
        plan.factories[0] = std::iter::once(Step::build(ck)).chain(std::iter::repeat_n(Step::build(pw), n)).collect();
        run(units, &scenario, &plan, 300.0)
    };
    let (four, six) = (list(4), list(6));
    assert!(six.finished.iter().filter(|f| f.unit == pw).count() >= 6, "the lab finished six Pawns in five minutes");
    let counted = Objective::Target { goals: vec![Goal { unit: pw, count: Some(4), by: None, round: false }], income: vec![] };
    let open = Objective::Target { goals: vec![Goal { unit: pw, count: None, by: None, round: false }], income: vec![] };
    let cost = units.list[pw].metal_cost;
    assert!((counted.score(units, &six, 300.0) - counted.score(units, &four, 300.0)).abs() < cost, "extras beyond the count are not rewarded");
    assert!(open.score(units, &six, 300.0) - open.score(units, &four, 300.0) > 1.5 * cost, "without a count every one counts");
    // An income goal: short of it at its time costs three minutes of the missing income; met, nothing.
    let at_4 = four.metal_income_at(240.0);
    let met = Objective::Target { goals: vec![], income: vec![(at_4 - 1.0, 240.0)] };
    let missed = Objective::Target { goals: vec![], income: vec![(at_4 + 2.0, 240.0)] };
    assert!((met.score(units, &four, 300.0) - missed.score(units, &four, 300.0) - 3.0 * 60.0 * 2.0).abs() < 1e-6);
    assert_eq!(Objective::parse_target("target income:40 by 10:00", units).unwrap().income_goals(), &[(40.0, 600.0)]);
    assert!(Objective::parse_target("target income:40", units).unwrap_err().contains("needs its time"));
    // A deadline is a requirement: six Pawns asked by the time the fourth finished are two short, three metal each.
    let fourth = four.finished.iter().filter(|f| f.unit == pw).nth(3).unwrap().t;
    let due = Objective::Target { goals: vec![Goal { unit: pw, count: Some(6), by: Some(fourth), round: false }], income: vec![] };
    let loose = Objective::Target { goals: vec![Goal { unit: pw, count: Some(6), by: None, round: false }], income: vec![] };
    assert!((loose.score(units, &four, 300.0) - due.score(units, &four, 300.0) - 2.0 * buildorder::anneal::TARGET_SHORTFALL * cost).abs() < 1e-6);
}

/// Resources no build can exhaust, so that durations are build time over power alone.
fn rich(game: &Game) -> Scenario {
    let mut scenario = bare(game);
    scenario.start_metal = 1e6;
    scenario.start_energy = 1e6;
    scenario.base_storage = 1e6;
    scenario
}

/// `help`: a builder's power lands on the dearest frame begun (the most metal still to build, not the dearest unit)
/// from the moment it stands beside it, the frame finishes when the summed power has paid its build time, and a help
/// step with no frame to help is passed over (docs/design/2026-10-09-late-economy.md §9).
#[test]
fn help_puts_a_builders_power_on_the_dearest_frame_until_it_stands() {
    let game = game();
    let units = &game.units;
    let cv = units.index("armcv").unwrap();
    // The plant makes two constructors; the first raises a turret (230 metal, 59 s alone), the second a solar (155, 29 s).
    // The commander, free after four storages, helps when both frames stand: the solar, just begun, has the most metal
    // left. The second constructor's help, after its solar, finds the turret still building.
    let with_help = |help: bool| {
        let mut plan = Plan::empty(1, 2);
        plan.commander = at_home(units, &["armvp", "armestor", "armestor", "armestor", "armestor"]);
        plan.factories[0] = vec![Step::build(cv), Step::build(cv)];
        plan.constructors[0] = at_home(units, &["armnanotc"]);
        plan.constructors[1] = at_home(units, &["armsolar"]);
        if help {
            plan.commander.push(Step { item: Item::Help, site: None });
            plan.constructors[1].push(Step { item: Item::Help, site: None });
        }
        run(units, &rich(&game), &plan, 300.0)
    };
    let begun = |outcome: &Outcome| -> Vec<f64> { outcome.finished.iter().filter(|f| f.unit == cv).map(|f| f.t + 0.1).collect() };
    let alone = with_help(false);
    let began = begun(&alone);
    assert_eq!(began.len(), 2, "two constructors came out");
    assert!((finish_time(units, &alone, "armnanotc") - (began[0] + 5300.0 / 90.0)).abs() < 0.3, "alone, the turret takes 59 s at 90");
    assert!((finish_time(units, &alone, "armsolar") - (began[1] + 2600.0 / 90.0)).abs() < 0.3);
    let helped = with_help(true);
    let began = begun(&helped);
    let commander = helped.finished.iter().filter(|f| units.list[f.unit].name == "armestor").map(|f| f.t).fold(0.0, f64::max) + 0.1;
    assert!(commander > began[1] && commander < finish_time(units, &alone, "armnanotc"), "the commander is free while both frames stand");
    // The solar: 90 (t - began) + 300 (t - commander) = 2600.
    let solar = finish_time(units, &helped, "armsolar");
    let expected = (2600.0 + 90.0 * began[1] + 300.0 * commander) / 390.0;
    assert!((solar - expected).abs() < 1.0, "the solar, with the most metal left, got the commander: stands at {solar}, the power integral says {expected}");
    // The turret: 90 (t - began) + 90 (t - the second constructor's join, after its solar) = 5300.
    let turret = finish_time(units, &helped, "armnanotc");
    let expected = (5300.0 + 90.0 * began[0] + 90.0 * (solar + 0.1)) / 180.0;
    assert!((turret - expected).abs() < 1.0, "the turret got the second constructor after its solar: stands at {turret}, the integral says {expected}");
    assert_eq!(helped.effective[0].last().map(|s| s.item), Some(Item::Help), "the commander's help step was taken up");
    assert_eq!(helped.effective[3].len(), 2, "the second constructor's solar and help were both taken up");
}

/// A standing construction turret helps the dearest frame within its reach when the scenario says so, and feeds the
/// first factory (here: nothing) otherwise; a frame out of its reach gets nothing from it.
#[test]
fn a_turret_helps_the_dearest_frame_in_its_reach_when_told() {
    let game = game();
    let units = &game.units;
    let far = (HOME.0, HOME.1 + 1000.0);
    let storage_at = |site: (f64, f64), help: bool| {
        let mut scenario = rich(&game);
        scenario.nanos_help_frames = help;
        let mut state = State::start(&scenario);
        state.standing.push(Standing { unit: units.index("armnanotc").unwrap(), site: HOME, pays: 0.0 });
        let mut plan = Plan::empty(0, 0);
        plan.commander = vec![Step { item: Item::Build(units.index("armestor").unwrap()), site: Some(site) }];
        finish_time(units, &simulate(units, &scenario, &state, &plan, 200.0), "armestor")
    };
    assert!((storage_at(HOME, true) - (0.1 + 4110.0 / 500.0)).abs() < 0.3, "300 + 200 on the frame: {}", storage_at(HOME, true));
    assert!((storage_at(HOME, false) - (0.1 + 4110.0 / 300.0)).abs() < 0.3, "turrets to the factory, none standing: the commander alone, {}", storage_at(HOME, false));
    let commander = &units.list[game.commander];
    let walk = (1000.0 - commander.build_distance) / commander.speed;
    assert!((storage_at(far, true) - (0.1 + walk + 4110.0 / 300.0)).abs() < 1.0, "out of the turret's reach: the commander alone after its walk, {}", storage_at(far, true));
}

/// The tax is paid before any build and what the stock cannot pay is owed, never deferred: a build that would have
/// finished on the income alone never does under a tax above it.
#[test]
fn the_tax_is_paid_first_and_what_cannot_be_paid_is_owed() {
    let game = game();
    let units = &game.units;
    let mut plan = Plan::empty(0, 0);
    plan.commander = at_home(units, &["armwin"]);
    let mut scenario = bare(&game);
    scenario.start_metal = 20.0;
    let free = run(units, &scenario, &plan, 60.0);
    // 20 in the bank and 2 a second: the 40-metal turbine is paid for at ten seconds.
    assert!((finish_time(units, &free, "armwin") - 10.0).abs() < 0.6, "{}", finish_time(units, &free, "armwin"));
    assert_eq!((free.last().tax_paid, free.last().tax_unpaid), (0.0, 0.0));
    scenario.tax = vec![(0.0, 10.0)];
    let taxed = run(units, &scenario, &plan, 60.0);
    assert!(taxed.finished.is_empty(), "under a tax of 10 a second on an income of 2 the turbine never gets its metal");
    let last = taxed.last();
    assert!((last.tax_paid + last.tax_unpaid - 600.0).abs() < 1e-6, "every second's tax is paid or owed: {} + {}", last.tax_paid, last.tax_unpaid);
    assert!(last.tax_paid > 100.0 && last.tax_unpaid > 300.0, "paid {} owed {}", last.tax_paid, last.tax_unpaid);
    // The tax in force is the schedule's last entry at or before the second.
    scenario.tax = vec![(30.0, 10.0)];
    let later = run(units, &scenario, &plan, 60.0);
    assert!((finish_time(units, &later, "armwin") - 10.0).abs() < 0.6, "nothing is taxed before the schedule starts");
    assert!((later.last().tax_paid + later.last().tax_unpaid - 300.0).abs() < 1e-6);
}

/// A standing silo stockpiles a missile every 120 s at the engine's fixed rate, drawing 1,000 metal and 125,000 energy
/// a round in the same queue as the builds, and no helper shortens it (nothing in the engine lets one).
#[test]
fn a_silo_stockpiles_at_the_engines_rate_and_nobody_can_help_it() {
    let game = game();
    let units = &game.units;
    let silo = units.index("armsilo").unwrap();
    assert_eq!(units.stockpile(silo).map(|s| (s.metal, s.energy, s.seconds)), Some((1000.0, 125_000.0, 120.0)));
    assert_eq!(units.stockpile(units.index("armwin").unwrap()), None);
    let scenario = rich(&game);
    let mut state = State::start(&scenario);
    state.standing.push(Standing { unit: silo, site: HOME, pays: 0.0 });
    let piled = |plan: &Plan| simulate(units, &scenario, &state, plan, 250.0);
    let idle = piled(&Plan::empty(0, 0));
    let rounds: Vec<(f64, usize)> = idle.stockpiled.clone();
    assert_eq!(rounds.len(), 2, "{rounds:?}");
    assert!((rounds[0].0 - 120.0).abs() < 0.2 && (rounds[1].0 - 240.0).abs() < 0.2 && rounds.iter().all(|r| r.1 == silo), "{rounds:?}");
    assert_eq!(idle.last().rounds, 2);
    // Two rounds and ten seconds of the third paid for; the commander's 2 metal and 30 energy a second came in.
    let commander = &units.list[game.commander];
    let paid = 2.0 + 10.0 / 120.0;
    assert!((idle.last().metal - (1e6 + 250.0 * commander.metal_make - paid * 1000.0)).abs() < 2.0, "metal {}", idle.last().metal);
    assert!((idle.last().energy - (1e6 + 250.0 * commander.energy_make - paid * 125_000.0)).abs() < 60.0, "energy {}", idle.last().energy);
    let mut helping = Plan::empty(0, 0);
    helping.commander = vec![Step { item: Item::Help, site: None }];
    let helped = piled(&helping);
    assert_eq!(helped.stockpiled.iter().map(|r| (r.0 * 10.0).round()).collect::<Vec<_>>(), rounds.iter().map(|r| (r.0 * 10.0).round()).collect::<Vec<_>>(), "the rounds come at the same seconds");
    assert!(helped.effective[0].is_empty(), "a stockpile is not a frame: the help step was passed over");
}

/// `reach:N` prefers the plan whose 30-second income first reaches N soonest; `round:UNIT` is a goal of a stockpiler's
/// round and refuses a unit that stockpiles nothing; `help` reads and writes in plan text.
#[test]
fn the_reach_objective_round_goals_and_the_help_word() {
    let game = game();
    let units = &game.units;
    let scenario = bare(&game);
    let mex = units.index("armmex").unwrap();
    let mut quick = Plan::empty(0, 0);
    quick.commander = vec![Step::build(mex), Step::build(mex)];
    let mut slow = Plan::empty(0, 0);
    slow.commander = at_home(units, &["armsolar", "armsolar"]);
    slow.commander.extend([Step::build(mex), Step::build(mex)]);
    let objective = Objective::parse("reach:5.5").unwrap();
    assert_eq!(objective, Objective::Reach { income: 5.5 });
    assert_eq!(Objective::parse("reach:x"), None);
    let (a, b) = (run(units, &scenario, &quick, 300.0), run(units, &scenario, &slow, 300.0));
    assert!(a.mean_metal_income(300.0, 30.0) > 5.5 && b.mean_metal_income(300.0, 30.0) > 5.5, "both reach it by the horizon");
    assert!(objective.score(units, &a, 300.0) > objective.score(units, &b, 300.0), "the two extractors first reach 5.5 a second sooner");
    let silo = units.index("armsilo").unwrap();
    assert_eq!(Objective::parse_target("target:round:armsilo@1200", units).unwrap().goals(), &[Goal { unit: silo, count: None, by: Some(1200.0), round: true }]);
    assert!(Objective::parse_target("target:round:armwin", units).is_err());
    let plan = Plan::from_text("com: mex help\ncon0: help assist", "arm", units).unwrap();
    assert_eq!(plan.commander[1].item, Item::Help);
    assert_eq!(plan.to_text(units), "com: mex help\ncon0: help assist\n");
}

/// The territory bot's score (`docs/design/2026-10-20-territory-bot.md`, decision 2): metal made over the horizon is
/// the sum of every second's income, a terminal value adds that many seconds of the income at the end, and nothing
/// else counts: a plan that builds a soldier scores no more than one that does not.
#[test]
fn made_is_the_income_summed_plus_the_terminal_seconds() {
    let game = game();
    let units = &game.units;
    let scenario = bare(&game);
    let plan = Plan { commander: at_home(units, &["armmex", "armmex"]), factories: Vec::new(), constructors: Vec::new() };
    let outcome = run(units, &scenario, &plan, 120.0);
    let summed: f64 = outcome.samples.iter().map(|s| s.metal_income).sum();
    let plain = Objective::Made { terminal: 0.0 }.score(units, &outcome, 120.0);
    assert!((plain - summed).abs() < 1.0, "made {plain} against the income summed {summed} (the shaping term is below one)");
    let with_terminal = Objective::Made { terminal: 90.0 }.score(units, &outcome, 120.0);
    let final_income = outcome.mean_metal_income(120.0, 30.0);
    assert!((with_terminal - plain - 90.0 * final_income).abs() < 1e-6, "{with_terminal} against {plain} + 90 x {final_income}");
    assert_eq!(Objective::parse("made"), Some(Objective::Made { terminal: 0.0 }));
    assert_eq!(Objective::parse("made:90"), Some(Objective::Made { terminal: 90.0 }));
    assert_eq!(Objective::parse("made").map(|o| o.name()), Some("made"));
}

/// The linear program (docs/design/2026-10-20-territory-lp.md) on the commander alone with two spots: both are
/// taken by the end, never more than the build power allows a bucket (300 build power on a 1,800 extractor with a
/// ten-second walk charged as 3,000: 0.625 a bucket), the stocks never negative, the final income the commander's
/// plus both spots', the prices non-negative.
#[test]
fn the_linear_program_fills_the_spots_within_the_build_power() {
    use buildorder::lp::{solve, LpOptions};
    let game = game();
    let units = &game.units;
    let scenario = bare(&game);
    let mex = units.index("armmex").unwrap();
    let offer = vec![mex, units.index("armsolar").unwrap()];
    let options = LpOptions { dt: 10.0, mid_from: 300.0, coarse_from: 300.0, horizon: 300.0, ..Default::default() };
    let plan = solve(units, &scenario, &State::start(&scenario), &offer, &options).unwrap();
    assert_eq!(plan.buckets, 30);
    let at_end = plan.count_at(mex, 300.0);
    assert!((at_end - 2.0).abs() < 0.05, "extractors at the end {at_end}");
    assert!(plan.count_at(mex, 10.0) <= 0.625 + 1e-3, "{} after one bucket", plan.count_at(mex, 10.0));
    assert!(plan.metal.iter().all(|m| *m >= -1e-6) && plan.energy.iter().all(|e| *e >= -1e-6));
    let expected = units.list[game.commander].metal_make + 4.0;
    assert!((plan.metal_income[plan.buckets] - expected).abs() < 0.1, "final income {} against {expected}", plan.metal_income[plan.buckets]);
    assert!(plan.made > 0.0 && plan.metal_price.iter().all(|p| *p >= -1e-6) && plan.solve_time < 2.0);
}

/// A standing lab that nothing offered needs, no metal in stock, eight spots, and five builders to take them (the
/// commander and four constructors), so that the metal binds and not the builders' time: with `reclaim` on the
/// program takes the lab down for its metal in the first minute, as much of it as the extractors want (a third:
/// a `Reclaim` flow on it), and makes clearly more than without, since the commander's income alone would wait on
/// every extractor; without the option the lab stands to the end. (With the commander alone the model rightly keeps
/// the lab: its 29 s on the reclaim cost more than the metal saved, the commander's time being the bound.) The count
/// after the first minute says nothing: once the spots are taken the metal is worth nothing to the objective and
/// the interior point builds labs with it for free.
#[test]
fn the_linear_program_takes_an_idle_building_down_for_its_metal_when_allowed() {
    use buildorder::lp::{solve, Kind, LpOptions};
    let game = game();
    let units = &game.units;
    let spots: Vec<Spot> = (0..8).map(|k| Spot { at: (1000.0 + 200.0 * k as f64, 1400.0), metal: 2.0 }).collect();
    let scenario = Scenario::new(game.commander, HOME, spots, Arc::new(Straight { detour: 1.0 }), 10.0);
    let mex = units.index("armmex").unwrap();
    let lab = units.index("armlab").unwrap();
    let offer = vec![mex];
    let mut state = State::start(&scenario);
    state.metal = 0.0;
    state.energy = 100_000.0;
    state.energy_storage = 100_000.0;
    state.standing.push(Standing { unit: lab, site: HOME, pays: 0.0 });
    let ck = units.index("armck").unwrap();
    for _ in 0..4 {
        state.builders.push(StateBuilder { unit: ck, place: HOME, job: None });
    }
    let without = LpOptions { dt: 10.0, mid_from: 300.0, coarse_from: 300.0, horizon: 300.0, ..Default::default() };
    let with = LpOptions { reclaim: true, ..without.clone() };
    let plain = solve(units, &scenario, &state, &offer, &without).unwrap();
    let plan = solve(units, &scenario, &state, &offer, &with).unwrap();
    assert!(plain.count_at(lab, 300.0) > 0.95, "without the option the lab stands: {}", plain.count_at(lab, 300.0));
    assert!(plan.count_at(lab, 60.0) < 0.9, "with it the lab is being taken down in the first minute: {}", plan.count_at(lab, 60.0));
    let reclaimed: f64 = plan.flows.iter().filter(|f| f.kind == Kind::Reclaim && f.unit == lab).map(|f| f.per_bucket.iter().sum::<f64>() / f.cost).sum();
    assert!(reclaimed > 0.25, "a reclaim flow on the lab: {reclaimed}");
    assert!(plan.made > plain.made + 50.0, "the lab's metal bought extractors: {} against {}", plan.made, plain.made);
    assert!(plan.metal.iter().all(|m| *m >= -1e-6));
}

/// A standing advanced lab with six construction turrets standing beside it, its constructor and extractors
/// offered, forty spots to take and the stocks full, so that the constructors' pace binds: with `nano_assist` at ten
/// the turrets' 1,200 build power goes on the lab's products through an Assist flow of the turret type (the lab's
/// own power and the commander's guard at twice it besides), and the first minute makes clearly more constructors
/// than without the option, when the turrets have nothing to do (their Help pairs are bounded by the mobile
/// builders' direct power on a column, and nothing offered is a building they could help). The pair is for an
/// advanced factory's products only: a basic lab under the same option gets none (terr67-68: offered on every
/// factory, the program pumped basic constructors and held the advanced plant to 8:00). thebluegecko's ramp: 29
/// turrets within 400 of his advanced plant by 10:00, an advanced constructor every 6-8 s.
#[test]
fn the_linear_program_puts_turrets_on_a_factorys_products_when_allowed() {
    use buildorder::lp::{solve, Kind, LpOptions};
    let game = game();
    let units = &game.units;
    let spots: Vec<Spot> = (0..40).map(|k| Spot { at: (1000.0 + 150.0 * (k % 10) as f64, 1400.0 + 300.0 * (k / 10) as f64), metal: 2.0 }).collect();
    let scenario = Scenario::new(game.commander, HOME, spots, Arc::new(Straight { detour: 1.0 }), 10.0);
    let mex = units.index("armmex").unwrap();
    let lab = units.index("armalab").unwrap();
    let ck = units.index("armack").unwrap();
    let nano = units.index("armnanotc").unwrap();
    let offer = vec![mex, ck];
    let mut state = State::start(&scenario);
    state.metal = 20_000.0;
    state.metal_storage = 20_000.0;
    state.energy = 1_000_000.0;
    state.energy_storage = 1_000_000.0;
    state.standing.push(Standing { unit: lab, site: HOME, pays: 0.0 });
    for _ in 0..6 {
        state.standing.push(Standing { unit: nano, site: HOME, pays: 0.0 });
    }
    // One factory at most: the commander would otherwise build labs from the stock, fractions that produce from
    // the first bucket (19.6 constructors in the first minute without the cap).
    let without = LpOptions { dt: 10.0, mid_from: 300.0, coarse_from: 300.0, horizon: 300.0, max_factories: 1.0, ..Default::default() };
    // The ring bound (terr96): the advanced lab's ring holds the six turrets standing.
    let with = LpOptions { plant_rings: vec![(lab, 6.0)], ..without.clone() };
    let plain = solve(units, &scenario, &state, &offer, &without).unwrap();
    let plan = solve(units, &scenario, &state, &offer, &with).unwrap();
    let (before, after) = (plain.count_at(ck, 60.0), plan.count_at(ck, 60.0));
    let own = units.list[lab].worker_time;
    let most = 60.0 * 3.0 * own / units.list[ck].build_time;
    assert!(before < most + 0.1, "without the option the lab and the commander's guard make at most {most:.1} constructors a minute: {before}");
    assert!(after > 1.5 * before, "with it the turrets' power is on the products: {after} against {before}");
    let assisted: f64 = plan.flows.iter().filter(|f| f.kind == Kind::Assist && f.builder == nano && f.unit == ck).map(|f| f.per_bucket.iter().sum::<f64>()).sum();
    assert!(assisted > 1_000.0, "an Assist flow of the turret type on the constructor: {assisted}");
    assert!(plain.flows.iter().all(|f| !(f.kind == Kind::Assist && f.builder == nano)), "no such flow without the option");
    assert!(plan.made > plain.made, "the constructors took the spots sooner: {} against {}", plan.made, plain.made);
    // The basic lab's products get no pair.
    let basic = units.index("armlab").unwrap();
    state.standing.retain(|s| s.unit != lab);
    state.standing.push(Standing { unit: basic, site: HOME, pays: 0.0 });
    let basic_plan = solve(units, &scenario, &state, &[mex, units.index("armck").unwrap()], &with).unwrap();
    assert!(basic_plan.flows.iter().all(|f| !(f.kind == Kind::Assist && f.builder == nano)), "no turret assist on a basic lab's products");
}
