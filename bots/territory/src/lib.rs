//! A heuristic bot that plans its economy by a rolling-horizon search over the build-order simulator
//! (`crates/buildorder`). Lobby name `WRTerritory`. The design: `docs/design/2026-10-20-territory-bot.md`; what
//! decides, and the arms of the rows, in this directory's `README.md` under `T-` ids.
//!
//! A bot of `bots/`: code decides everything here, and the flagship's rule against that (`docs/code-decisions.md`)
//! does not govern it (`bots/README.md`). The decider is the search: a snapshot of the game as the simulator's
//! state, the plan with the most metal made to the game's end, the plan's next step for every builder. No rule
//! names a unit; the search picks from the roster.

mod defence;
mod hands;
mod his_start;
mod options;
mod planner;
mod promise;
mod race;
mod snapshot;
mod walks;
mod work;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use bot_protocol::{Bot, Command, Commands, EnemyUnit, Event, Hello, OwnUnit, Resource, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use buildorder::anneal::Palette;
use buildorder::game::{Game, Ground};
use buildorder::lp::LpOptions;
use buildorder::regions::{Regions, REGION_SECONDS};
use buildorder::plan::{Item, Plan};
use buildorder::units::Role;

pub(crate) const SECOND: i32 = 30;
pub(crate) const MINUTE: i32 = 60 * SECOND;
/// A builder's coming asks for a re-plan, but never this soon after the last.
const REPLAN_LEAST: i32 = 10 * SECOND;
/// The horizon never shrinks below this, whatever the clock says of the game's end.
const HORIZON_LEAST: f64 = 120.0;

pub(crate) struct Territory {
    pub hello: Hello,
    pub defs: BTreeMap<UnitDefId, UnitDefInfo>,
    /// Each type's index in the simulator's unit table (the Hello's list order).
    index: BTreeMap<UnitDefId, usize>,
    game: Option<Arc<Game>>,
    ground: Option<Arc<dyn Ground>>,
    pub(crate) planner: Option<planner::Planner>,
    pub home: Vec3,
    pub enemy_home: Vec3,
    pub frame: i32,
    pub own: BTreeMap<UnitId, OwnUnit>,
    /// His units in sight this tick (the dead spots; what stands by a dropped order's site).
    pub enemies: Vec<EnemyUnit>,
    /// The metal and the energy this tick, stock, income, usage and storage: what a unit must be affordable against
    /// (T-ENERGY's budget reads the energy's).
    pub metal: Resource,
    pub energy: Resource,
    pub options: options::Options,
    pub hands: hands::Hands,
    /// The constructors' walks measured in the game, which the program prices its pairs from (T-WALKS).
    pub walks: walks::Walks,
    /// The spots the hands may take (the scenario's), as indices into the game's list.
    pub spots: Vec<usize>,
    /// The map's spots in regions (T-REGIONS), made at the start.
    pub regions: Option<Regions>,
    /// The race of force per region, an instrument (T-RACE).
    pub race: race::Race,
    /// The turrets and radars wanted (T-COVER, T-RADAR).
    pub defence: defence::Defence,
    /// Metal made so far: the engine's income integrated over the ticks.
    made: f64,
    last_frame: i32,
    last_replan: i32,
    builders_seen: usize,
    plans: usize,
    started: bool,
    announced: bool,
}

impl Territory {
    pub(crate) fn game(&self) -> &Game {
        self.game.as_ref().expect("the game is known once the bot has started")
    }

    pub(crate) fn ground(&self) -> Arc<dyn Ground> {
        self.ground.clone().expect("the ground is known once the bot has started")
    }

    pub(crate) fn def(&self, id: UnitDefId) -> Option<&UnitDefInfo> {
        self.defs.get(&id)
    }

    pub(crate) fn unit_index(&self, def: UnitDefId) -> Option<usize> {
        self.index.get(&def).copied()
    }

    pub(crate) fn role_of_def(&self, def: UnitDefId) -> Option<Role> {
        self.unit_index(def).map(|i| self.game().units.list[i].role)
    }

    pub(crate) fn log(&self, text: std::fmt::Arguments) {
        eprintln!("[territory ai={}] f={} {text}", self.hello.ai_id, self.frame);
    }

    fn start(&mut self, tick: &Tick) {
        let Some(commander) = tick.snapshot.own_units.iter().find(|u| self.def(u.def).is_some_and(|d| !d.build_options.is_empty() && d.speed > 0.0)).cloned() else { return };
        self.home = commander.pos;
        // His start (T-SCENARIO): the script's, else the game's own guess in his box, else the mirror. His box's
        // centre, taken before, was 100-2,437 from where he stood on five maps (terr103-107), and with the starts
        // pinned the script's own position went unread: every one of terr102's 45 deaths to his weapons came within
        // 300 of his start, 2,320-3,160 from the point the dead radius was drawn round (his box's centre or the mirror).
        let (enemy_home, source) = his_start::his_start(&self.hello, self.home);
        self.enemy_home = enemy_home;
        let Some(game) = Game::from_hello(&self.hello, commander.def, (self.home.x as f64, self.home.z as f64)) else {
            self.log(format_args!("the commander's type {} is not in the unit table: nothing will be built", commander.def.0));
            return;
        };
        let water = self.hello.terrain.heights.iter().any(|h| *h < 0);
        let mut palette = Palette::roster(&game.units, game.commander, None, water, &[]);
        if !self.options.soldiers {
            palette.factory.retain(|i| matches!(i, Item::Build(u) if game.units.list[*u].role == Role::Builder));
        }
        // The line arm: the factories the program may open with.
        palette.keep_line(&game.units, game.commander, &self.options.line);
        let names = |items: &[Item]| items.iter().map(|i| match i { Item::Build(u) => game.units.list[*u].name.clone(), Item::Assist => "assist".into(), Item::Help => "help".into() }).collect::<Vec<_>>().join(" ");
        let (mobile, factory) = (names(&palette.mobile), names(&palette.factory));
        let builders = palette.builders(&game.units, game.commander);
        let line_constructor = palette.constructor;
        self.ground = Some(game.ground());
        let game = Arc::new(game);
        self.game = Some(game.clone());
        self.planner = Some(planner::Planner::new(game, palette));
        let scenario = self.scenario();
        self.spots = self.spot_indices(&scenario);
        // The map's spots in regions, each within REGION_SECONDS' walk of the line's first constructor (step 1 of
        // `docs/design/2026-10-25-territory-regions.md`: logged, not yet planned over).
        let began = std::time::Instant::now();
        let game = self.game.clone().expect("made above");
        let radius = REGION_SECONDS * game.units.list[line_constructor].speed;
        let regions = Regions::new(&game, &builders, radius);
        self.log(format_args!(
            "regions: {} over {} spots within {radius:.0} ({REGION_SECONDS:.0} s of {}) in {} ms: {}",
            regions.list.len(),
            game.spots.len(),
            game.units.list[line_constructor].name,
            began.elapsed().as_millis(),
            regions.describe(&game, &builders, &self.spots).join(" | ")
        ));
        self.log(format_args!("regions of spots: {}", regions.of_spot.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(" ")));
        self.regions = Some(regions);
        // The ground's distance fields are made on first use, one per place walked to; made during the first
        // search they inflated its iteration-cost probe and it stopped at a third of its budget (terr-smoke-1: a
        // first plan worth 35,000 against the 86,000 found 28 s later). Every place a plan walks to: the spots,
        // the farm's first slots round its origin and round home (T-FARM), home.
        let began = std::time::Instant::now();
        let ground = self.ground();
        let home = (self.home.x as f64, self.home.z as f64);
        let mut places: Vec<(f64, f64)> = scenario.spots.iter().map(|s| s.at).collect();
        let origin = self.farm_origin();
        let mut slots = hands::farm_slots(origin, origin, 500.0, false);
        slots.extend(hands::farm_slots(origin, self.home, 400.0, false));
        places.extend(slots.iter().map(|p| (p.x as f64, p.z as f64)));
        places.push(home);
        for place in &places {
            ground.walk(home, *place);
        }
        let warmed = began.elapsed();
        self.log(format_args!("ground: {} places walked in {} ms", places.len(), warmed.as_millis()));
        self.log(format_args!(
            "home ({:.0}, {:.0}), enemy start taken as ({:.0}, {:.0}) ({source}); {} spots of {} in the scenario; arms: {}; palette mobile: {mobile}; factory: {factory}",
            self.home.x, self.home.z, self.enemy_home.x, self.enemy_home.z, self.spots.len(), self.hello.metal_spots.len(), self.options.describe()
        ));
        if self.options.dump {
            self.dump(commander.def);
        }
        self.started = true;
        // The first plan, on this thread: the engine waits a second once, where twenty game-seconds of an idle
        // commander at speed 50 would cost more.
        let request = self.request(tick);
        let planned = self.planner.as_ref().expect("made above").blocking(request, Duration::from_millis(self.options.first_budget_ms), self.options.threads);
        self.adopt_planned(planned);
    }

    /// The game as the simulator sees it, beside the match for the oracle (`buildorder optimize --hello FILE`).
    fn dump(&self, commander: UnitDefId) {
        let Some(dir) = std::env::var_os("WITHIN_REASON_TRUTH_DIR") else { return };
        let path = std::path::Path::new(&dir).join(format!("territory-{}.json", self.hello.ai_id));
        let text = serde_json::json!({ "home": [self.home.x, self.home.z], "commander": commander.0, "hello": &self.hello }).to_string();
        match std::fs::write(&path, text) {
            Ok(()) => self.log(format_args!("dump: {}", path.display())),
            Err(e) => self.log(format_args!("dump: {}: {e}", path.display())),
        }
    }

    fn request(&mut self, tick: &Tick) -> planner::Request {
        // The spots in play follow his seen units (the dead spots, decision 4 and stage 2's first form).
        let scenario = self.scenario();
        self.spots = self.spot_indices(&scenario);
        let (state, builders) = self.snapshot(tick);
        let horizon = (self.options.end - state.t0).max(HORIZON_LEAST);
        let start = self.warm_start(&builders);
        let o = &self.options;
        // The commander's spots: the free ones within its leash, the program's pair priced at their mean walk and
        // bounded by their number (his commander's two at 1,007 and 1,215 at 2:06-2:13 and none after; one walk of
        // 40 for the whole map gave it none after the opening's, terr76-89).
        let spots = &self.hello.metal_spots;
        let near: Vec<f64> = self.spots.iter().copied().filter(|&i| !self.spot_taken(i) && spots[i].dist2d(self.home) < o.com_leash).map(|i| f64::from(spots[i].dist2d(self.home))).collect();
        let commander_speed = self.game().units.list[self.game().commander].speed.max(1.0);
        // The advanced constructor's spots priced from home (the plant ramp's second change, terr97): born at the
        // plant, it reaches a free spot at the free spots' mean distance from home, not at a constructor's chain
        // walk, so from the minute the free spots lie far its upgrade beside home wins, as his did from 7:00.
        let free_far: Vec<f64> = self.spots.iter().copied().filter(|&i| !self.spot_taken(i)).map(|i| f64::from(spots[i].dist2d(self.home))).collect();
        let advanced_speed = self.game().units.list.iter().filter(|d| d.role == Role::Builder && d.builds.iter().any(|u| self.is_upgrade(*u))).map(|d| d.speed).fold(0.0, f64::max).max(1.0);
        // The plants' rings (T-PLANT): per factory type, the turrets its plants' rings hold, the program's bound on
        // the turrets' assist on that type's products.
        let mut plant_rings: Vec<(usize, f64)> = Vec::new();
        for (unit, _, standing, free) in self.plant_rings() {
            match plant_rings.iter_mut().find(|(u, _)| *u == unit) {
                Some(entry) => entry.1 += (standing + free) as f64,
                None => plant_rings.push((unit, (standing + free) as f64)),
            }
        }
        let lp = o.lp.then(|| LpOptions {
            dt: o.dt,
            mid_from: o.mid_from,
            mid_dt: o.mid_dt,
            coarse_from: o.coarse_from,
            coarse_dt: o.coarse_dt,
            horizon,
            max_constructors: o.constructors as f64,
            max_factories: o.factories as f64,
            terminal: o.terminal,
            energy_reserve: o.energy_reserve,
            energy_margin: o.energy_margin,
            margin_from: o.margin_from,
            factory_from: o.factory_from,
            assist_factor: o.assist_factor,
            plant_rings: plant_rings.clone(),
            help_factor: o.help_factor,
            mobile_help: o.mobile_help,
            nanos_per_factory: o.nanos_per_factory,
            max_nanos: o.max_nanos,
            farm_big: o.farm_big,
            bank: o.bank,
            reclaim: o.reclaim,
            walk_extractor: o.walk_extractor,
            walk_upgrade: o.walk_upgrade,
            advanced_walk_extractor: o.ring_overhead + free_far.iter().sum::<f64>() / free_far.len().max(1) as f64 / advanced_speed,
            walk_base: o.walk_base,
            walk_factory: o.walk_factory,
            commander_walk_extractor: o.ring_overhead + near.iter().sum::<f64>() / near.len().max(1) as f64 / commander_speed,
            commander_spots: near.len() as f64,
            commander_walk_base: o.commander_walk_base,
            commander_walk_factory: o.commander_walk_factory,
            commander_reach: o.com_leash as f64,
            rings: o.rings,
            ring_overhead: o.ring_overhead,
            // T-WALKS: the constructors' walks measured in the game; the constant walks above are the prior.
            walked: self.walked(),
            walk_prior: walks::WALK_PRIOR,
        });
        planner::Request {
            scenario,
            state,
            builders,
            start,
            horizon,
            terminal: self.options.terminal,
            factories: self.options.factories,
            constructors: self.options.constructors,
            seed: 1 + self.plans as u64,
            frame: tick.frame,
            lp,
            affordable: self.metal.current as f64 + self.metal.income as f64 * hands::AFFORD_SECONDS,
        }
    }

    /// The plan in force with the steps taken removed, laid over the snapshot's builders in their order: what the
    /// next search starts from.
    fn warm_start(&self, builders: &[UnitId]) -> Option<Plan> {
        if self.plans == 0 {
            return None;
        }
        let plan = &self.hands.plan;
        let remaining = |id: &UnitId| self.hands.queues.get(id).map_or(Vec::new(), |(queue, next)| if *queue == usize::MAX { Vec::new() } else { plan.queue(*queue).get(*next..).map_or(Vec::new(), <[_]>::to_vec) });
        let role = |id: &UnitId| self.own.get(id).and_then(|u| self.role_of_def(u.def));
        let commander = builders.iter().find(|b| role(b) == Some(Role::Commander)).map_or(Vec::new(), remaining);
        let mut factories: Vec<Vec<_>> = builders.iter().filter(|b| role(b) == Some(Role::Factory)).map(remaining).collect();
        factories.extend(self.hands.spare_factory_queues.iter().map(|q| plan.queue(*q).clone()));
        let mut constructors: Vec<Vec<_>> = builders.iter().filter(|b| role(b) == Some(Role::Builder)).map(remaining).collect();
        constructors.extend(self.hands.spare_constructor_queues.iter().map(|q| plan.queue(*q).clone()));
        Some(Plan { commander, factories, constructors })
    }

    fn adopt_planned(&mut self, planned: planner::Planned) {
        self.plans += 1;
        self.last_replan = self.frame;
        let found = match &planned.answer {
            planner::Answer::Lp(plan) => {
                let at = |dt: f64| plan.income_at(planned.t0 + dt);
                let end = plan.t0 + plan.starts[plan.buckets];
                let units = &self.game().units;
                let count = |test: &dyn Fn(&buildorder::units::Unit) -> bool| -> f64 { plan.types.iter().enumerate().filter(|(_, u)| test(&units.list[**u])).map(|(i, _)| plan.counts[i][plan.buckets]).sum() };
                let basic = units.basic_extraction();
                self.log(format_args!(
                    "plan: n={} t0={:.0} wall={}ms score={:.0} horizon={:.0} predicted +60: income {:.1} mex {:.0} | +120: income {:.1} mex {:.0} | +300: income {:.1} mex {:.0} | end: income {:.1} mex {:.0} cons {:.0} fac {:.0} nanos {:.0} made {:.0}; stale {:.1} s; lp {} buckets {} vars {} rows {}",
                    self.plans, planned.t0, planned.wall.as_millis(), plan.objective, planned.horizon,
                    at(60.0), self.plan_count(plan, planned.t0 + 60.0, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
                    at(120.0), self.plan_count(plan, planned.t0 + 120.0, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
                    at(300.0), self.plan_count(plan, planned.t0 + 300.0, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
                    plan.income_at(end), count(&|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5), count(&|u| u.role == Role::Builder), count(&|u| u.role == Role::Factory), count(&|u| u.role == Role::Nano), plan.made,
                    (self.frame - planned.frame) as f32 / SECOND as f32, plan.status, plan.buckets, plan.variables, plan.rows
                ));
                // The targets by minute, one line: what the program wants standing at each whole minute from now.
                let mut by_minute = Vec::new();
                let mut m = 0.0;
                while m <= planned.horizon {
                    let t = planned.t0 + m;
                    by_minute.push(format!(
                        "{:.0}:mex{:.0}+{:.0} cons{:.0} fac{:.0} nano{:.0} mmkr{:.0} makr{:.0} afus{:.0} fus{:.0} eco{:.0} inc{:.0} bank{:.0}/{:.0}",
                        m / 60.0,
                        self.plan_count(plan, t, &|u| u.extracts_metal > 0.0 && u.extracts_metal <= basic * 1.5),
                        self.plan_count(plan, t, &|u| u.extracts_metal > basic * 1.5),
                        self.plan_count(plan, t, &|u| u.role == Role::Builder),
                        self.plan_count(plan, t, &|u| u.role == Role::Factory),
                        self.plan_count(plan, t, &|u| u.role == Role::Nano),
                        self.plan_count(plan, t, &|u| u.conv_capacity >= 500.0),
                        self.plan_count(plan, t, &|u| u.conv_capacity > 0.0 && u.conv_capacity < 500.0),
                        self.plan_count(plan, t, &|u| u.energy_make >= 2000.0),
                        self.plan_count(plan, t, &|u| u.energy_make >= 500.0 && u.energy_make < 2000.0),
                        self.plan_count(plan, t, &|u| u.role == Role::Eco && u.extracts_metal == 0.0 && u.conv_capacity == 0.0 && u.energy_make < 500.0 && (u.energy_make > 0.0 || u.wind_cap > 0.0)),
                        plan.income_at(t),
                        plan.metal_at(t),
                        plan.energy_at(t)
                    ));
                    m += 60.0;
                }
                // The factory kinds the program ends with, since the kinds decide the tier-2 line, with each kind's count
                // at +1, +3 and +5 minutes (T-COMMIT fires on the count within COMMIT_WITHIN: in terr49 the Armada plans
                // wanted 0.5-0.9 of a vehicle plant at the end from 3:35 and the commit came at 6:20, the Cortex plans a
                // whole advanced lab from 4:17 and the commit at 4:55).
                let kinds: Vec<String> = plan.types.iter().enumerate().filter(|(_, u)| units.list[**u].role == Role::Factory && plan.count_at(**u, end) >= 0.05).map(|(_, u)| format!("{} {:.2}/{:.2}/{:.2}/{:.1}", units.list[*u].name, plan.count_at(*u, plan.t0 + 60.0), plan.count_at(*u, plan.t0 + 180.0), plan.count_at(*u, plan.t0 + 300.0), plan.count_at(*u, end))).collect();
                let rings: Vec<String> = plan.rings.iter().enumerate().map(|(k, r)| format!("r{k} {} spots to {:.0} at {:.0} ({:.2}/s)", r.spots, r.bound, r.distance, r.metal)).collect();
                self.log(format_args!("targets: {} | factory kinds at +1/+3/+5 min/the end: {} | rings: {}", by_minute.join(" | "), kinds.join(" "), rings.join(", ")));
                self.log_promise(plan);
                self.hands.targets = Some(plan.clone());
                self.seed_committed();
                return;
            }
            planner::Answer::Anneal(found) => found,
        };
        let outcome = &found.outcome;
        let at = |dt: f64| {
            let t = planned.t0 + dt;
            outcome.samples.iter().find(|s| s.t >= t).map_or((0.0, 0, 0), |s| (outcome.mean_metal_income(s.t, 30.0), s.extractors, s.constructors))
        };
        let (i1, m1, _) = at(60.0);
        let (i2, m2, _) = at(120.0);
        let (i5, m5, _) = at(300.0);
        let last = outcome.last();
        let made: f64 = outcome.samples.iter().map(|s| s.metal_income).sum();
        self.log(format_args!(
            "plan: n={} t0={:.0} wall={}ms score={:.0} horizon={:.0} predicted +60: income {i1:.1} mex {m1} | +120: income {i2:.1} mex {m2} | +300: income {i5:.1} mex {m5} | end: income {:.1} mex {} cons {} fac {} nanos {} made {made:.0}; stale {:.1} s",
            self.plans, planned.t0, planned.wall.as_millis(), found.score, planned.horizon, outcome.mean_metal_income(last.t, 30.0), last.extractors, last.constructors, last.factories, last.nanos,
            (self.frame - planned.frame) as f32 / SECOND as f32
        ));
        for line in found.plan.to_text(&self.game().units).lines() {
            self.log(format_args!("queue: {line}"));
        }
        self.adopt(found.plan.clone(), &planned.builders, planned.frame);
    }

    /// The program's count of the types passing `test` at absolute second `t`.
    fn plan_count(&self, plan: &buildorder::lp::LpPlan, t: f64, test: &dyn Fn(&buildorder::units::Unit) -> bool) -> f64 {
        let units = &self.game().units;
        plan.types.iter().filter(|u| test(&units.list[**u])).map(|u| plan.count_at(*u, t)).sum()
    }

    fn banner(&self) -> String {
        let side = self.hello.teams.iter().find(|t| t.team == self.hello.team).map_or("?", |t| t.side.as_str());
        format!("{{name}} is WRTerritory {} | arms: {} | seat ai{} team {} {side}", ai_host::commit(), self.options.describe(), self.hello.ai_id, self.hello.team)
    }

    pub(crate) fn count_role(&self, role: Role) -> usize {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(role)).count()
    }

    fn curve_line(&mut self, tick: &Tick) {
        let game = self.game();
        let extractors = self.own.values().filter(|u| !u.being_built && self.unit_index(u.def).is_some_and(|i| game.units.list[i].extracts_metal > 0.0)).count();
        let build_power: f32 = self.own.values().filter(|u| !u.being_built && matches!(self.role_of_def(u.def), Some(Role::Commander | Role::Builder | Role::Factory | Role::Nano))).filter_map(|u| self.def(u.def)).map(|d| d.build_speed).sum();
        let (m, e) = (&tick.snapshot.metal, &tick.snapshot.energy);
        self.log(format_args!(
            "curve: extractors={extractors} constructors={} factories={} nanos={} build_power={build_power:.0} metal {:.0}/{:.0} +{:.1} -{:.1} energy {:.0}/{:.0} +{:.0} -{:.0} made={:.0} orders={} begun={} dropped={} skipped={} energy_refused={} plans={}",
            self.count_role(Role::Builder), self.count_role(Role::Factory), self.count_role(Role::Nano), m.current, m.storage, m.income, m.usage, e.current, e.storage, e.income, e.usage,
            self.made, self.hands.orders.len(), self.hands.begun, self.hands.dropped, self.hands.skipped, self.hands.energy_refused, self.plans
        ));
        // T-FARM: the turrets' use, and the frames none of them reaches.
        let turrets: Vec<&OwnUnit> = self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Nano)).collect();
        let busy = turrets.iter().filter(|u| !u.idle).count();
        let frames: Vec<Vec3> = self.own.values().filter(|u| u.being_built && self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let unreached = frames.iter().filter(|f| !turrets.iter().any(|t| t.pos.dist2d(**f) < hands::NANO_REACH)).count();
        // T-PLANT: the turrets within reach of a factory (his 29 by 10:00).
        let plants: Vec<Vec3> = self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Factory)).map(|u| u.pos).collect();
        let at_plants = turrets.iter().filter(|t| plants.iter().any(|p| p.dist2d(t.pos) < hands::NANO_REACH)).count();
        let (wanted, plant) = self.plant_turrets_wanted();
        let plant = plant.map_or("none".to_string(), |p| format!("({:.0},{:.0})", p.x, p.z));
        let rings: Vec<String> = self.plant_rings().iter().map(|(u, c, s, f)| format!("{} ({:.0},{:.0}) {s} of {} in its ring", self.game().units.list[*u].name, c.x, c.z, s + f)).collect();
        let rings = if rings.is_empty() { "none".to_string() } else { rings.join(", ") };
        self.log(format_args!("turrets: {} standing, {busy} busy, {at_plants} at the plants, the program's assist {wanted:.1} turrets, placing at {plant}; frames {} of which {unreached} beyond every turret's reach; rings: {rings}", turrets.len(), frames.len()));
        self.log(format_args!("jobs: {}", self.jobs_line()));
        let energy = self.energy_line();
        self.log(format_args!("energy: {energy}"));
        let ordered = self.ordered_line();
        self.log(format_args!("ordered: {ordered}"));
    }
}

impl Bot for Territory {
    fn start(hello: Hello) -> Self {
        let defs = hello.unit_defs.iter().map(|d| (d.id, d.clone())).collect();
        let index = hello.unit_defs.iter().enumerate().map(|(i, d)| (d.id, i)).collect();
        Territory {
            hello,
            defs,
            index,
            game: None,
            ground: None,
            planner: None,
            home: Vec3::default(),
            enemy_home: Vec3::default(),
            frame: 0,
            own: BTreeMap::new(),
            enemies: Vec::new(),
            metal: Resource::default(),
            energy: Resource::default(),
            options: options::Options::from_env(),
            hands: hands::Hands::default(),
            walks: walks::Walks::default(),
            spots: Vec::new(),
            regions: None,
            race: race::Race::default(),
            defence: defence::Defence::default(),
            made: 0.0,
            last_frame: 0,
            last_replan: 0,
            builders_seen: 0,
            plans: 0,
            started: false,
            announced: false,
        }
    }

    fn tick(&mut self, tick: Tick) -> Commands {
        self.frame = tick.frame;
        self.own = tick.snapshot.own_units.iter().map(|u| (u.id, u.clone())).collect();
        self.enemies = tick.snapshot.enemies.clone();
        self.metal = tick.snapshot.metal;
        self.energy = tick.snapshot.energy;
        if let Some(sensors) = &tick.snapshot.sensors {
            self.race.sensors = Some(sensors.clone());
        }
        if !self.started {
            self.start(&tick);
            if !self.started {
                return Commands::default();
            }
        }
        for event in &tick.events {
            match event {
                Event::UnitCreated { unit, builder: Some(builder) } => self.frame_begun(*unit, *builder),
                Event::BuildSiteNotFound { unit, .. } => self.failed(*unit, "no site"),
                Event::BuildSited { unit, at, .. } => self.sited(*unit, *at),
                Event::CommandRejected { unit, .. } => self.failed(*unit, "rejected"),
                Event::UnitDestroyed { unit, attacker, death } => {
                    // Every death goes to the log: terr5-all-nullai-comet-1 lost one to three constructors a game
                    // with no enemy on the map, and nothing said how.
                    if let Some(u) = self.own.get(unit) {
                        let name = self.def(u.def).map_or("?", |d| d.name.as_str());
                        // A basic extractor an upgrade of ours stands over is replaced, not lost (the engine reports
                        // it destroyed with no attacker: terr19-lp-nullai-comet-1, 120 of 141 deaths).
                        let replaced = self.unit_index(u.def).is_some_and(|i| self.game().units.list[i].extracts_metal > 0.0 && !self.is_upgrade(i))
                            && self.own.values().any(|o| o.id != *unit && o.pos.dist2d(u.pos) < 50.0 && self.unit_index(o.def).is_some_and(|i| self.is_upgrade(i)));
                        if replaced {
                            self.log(format_args!("replaced: {name} unit {} at ({:.0},{:.0}) by an upgrade", unit.0, u.pos.x, u.pos.z));
                        } else {
                            self.log(format_args!("died: {name} unit {} at ({:.0},{:.0}) death={death:?} attacker={attacker:?}", unit.0, u.pos.x, u.pos.z));
                        }
                    }
                    self.unit_lost(*unit);
                }
                _ => {}
            }
        }
        let mut commands: Vec<Command> = Vec::new();
        if !self.announced && tick.frame >= 2 * SECOND {
            self.announced = true;
            commands.push(Command::Say { text: self.banner() });
        }
        self.made += tick.snapshot.metal.income as f64 * (tick.frame - self.last_frame) as f64 / SECOND as f64;
        self.last_frame = tick.frame;

        // A re-plan on the clock, or when a factory or constructor has come, while none is under way.
        let builders_now = self.count_role(Role::Factory) + self.count_role(Role::Builder);
        let due = tick.frame - self.last_replan >= self.options.replan || (builders_now != self.builders_seen && tick.frame - self.last_replan >= REPLAN_LEAST);
        if due && self.planner.as_ref().is_some_and(|p| !p.busy()) {
            self.builders_seen = builders_now;
            self.last_replan = tick.frame;
            let request = self.request(&tick);
            let (budget, threads) = (Duration::from_millis(self.options.budget_ms), self.options.threads);
            self.planner.as_mut().expect("started").request(request, budget, threads);
        }
        if let Some(planned) = self.planner.as_mut().and_then(|p| p.poll()) {
            self.adopt_planned(planned);
        }
        if tick.due() % (SECOND / 2) == 0 {
            self.measure_walks();
            self.log_walks();
            self.observe_raids();
            self.plan_defence();
            self.run_hands(&mut commands);
            self.run_nanos(&mut commands);
            self.run_priorities(&mut commands);
        }
        if tick.due() % MINUTE == 0 && tick.frame > 0 {
            self.curve_line(&tick);
        }
        if tick.due() % promise::STANDING_LOG == 0 {
            self.log_standing();
        }
        if tick.due() % race::RACE_LOG == 0 && tick.frame > 0 {
            self.log_race();
        }
        if tick.due() % defence::DEFENCE_LOG == 0 && tick.frame > 0 {
            self.log_defence();
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Territory>);
