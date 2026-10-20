//! A heuristic bot that plans its economy by a rolling-horizon search over the build-order simulator
//! (`crates/buildorder`). Lobby name `WRTerritory`. The design: `docs/design/2026-10-20-territory-bot.md`; what
//! decides, and the arms of the rows, in this directory's `README.md` under `T-` ids.
//!
//! A bot of `bots/`: code decides everything here, and the flagship's rule against that (`docs/code-decisions.md`)
//! does not govern it (`bots/README.md`). The decider is the search: a snapshot of the game as the simulator's
//! state, the plan with the most metal made to the game's end, the plan's next step for every builder. No rule
//! names a unit; the search picks from the roster.

mod hands;
mod options;
mod planner;
mod snapshot;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use bot_protocol::{Bot, Command, Commands, Event, Hello, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use buildorder::anneal::Palette;
use buildorder::game::{Game, Ground};
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
    pub options: options::Options,
    pub hands: hands::Hands,
    /// The spots the hands may take (the scenario's), as indices into the game's list.
    pub spots: Vec<usize>,
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
        let ours = self.hello.ally_team;
        let map = Vec3 { x: self.hello.map.width, y: 0.0, z: self.hello.map.height };
        // His start box when the script gives one, else our start mirrored through the map's centre.
        self.enemy_home = self.hello.start_boxes.iter().find(|b| b.ally_team != ours).map_or(Vec3 { x: map.x - self.home.x, y: 0.0, z: map.z - self.home.z }, |b| b.centre());
        let Some(game) = Game::from_hello(&self.hello, commander.def, (self.home.x as f64, self.home.z as f64)) else {
            self.log(format_args!("the commander's type {} is not in the unit table: nothing will be built", commander.def.0));
            return;
        };
        let water = self.hello.terrain.heights.iter().any(|h| *h < 0);
        let mut palette = Palette::roster(&game.units, game.commander, None, water, &[]);
        if !self.options.soldiers {
            palette.factory.retain(|i| matches!(i, Item::Build(u) if game.units.list[*u].role == Role::Builder));
        }
        let names = |items: &[Item]| items.iter().map(|i| match i { Item::Build(u) => game.units.list[*u].name.clone(), Item::Assist => "assist".into(), Item::Help => "help".into() }).collect::<Vec<_>>().join(" ");
        let (mobile, factory) = (names(&palette.mobile), names(&palette.factory));
        self.ground = Some(game.ground());
        let game = Arc::new(game);
        self.game = Some(game.clone());
        self.planner = Some(planner::Planner::new(game, palette));
        let scenario = self.scenario();
        self.spots = self.spot_indices(&scenario);
        // The ground's distance fields are made on first use, one per place walked to; made during the first
        // search they inflated its iteration-cost probe and it stopped at a third of its budget (terr-smoke-1: a
        // first plan worth 35,000 against the 86,000 found 28 s later). Every place a plan walks to: the spots,
        // the base spiral, home.
        let began = std::time::Instant::now();
        let ground = self.ground();
        let home = (self.home.x as f64, self.home.z as f64);
        let mut places: Vec<(f64, f64)> = scenario.spots.iter().map(|s| s.at).collect();
        places.extend((0..60).map(|n| { let p = hands::base_site(self.home, n); (p.x as f64, p.z as f64) }));
        places.push(home);
        for place in &places {
            ground.walk(home, *place);
        }
        let warmed = began.elapsed();
        self.log(format_args!("ground: {} places walked in {} ms", places.len(), warmed.as_millis()));
        self.log(format_args!(
            "home ({:.0}, {:.0}), enemy start taken as ({:.0}, {:.0}); {} spots of {} in the scenario; arms: {}; palette mobile: {mobile}; factory: {factory}",
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

    fn request(&self, tick: &Tick) -> planner::Request {
        let (state, builders) = self.snapshot(tick);
        let horizon = (self.options.end - state.t0).max(HORIZON_LEAST);
        let start = self.warm_start(&builders);
        planner::Request {
            scenario: self.scenario(),
            state,
            builders,
            start,
            horizon,
            terminal: self.options.terminal,
            factories: self.options.factories,
            constructors: self.options.constructors,
            seed: 1 + self.plans as u64,
            frame: tick.frame,
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
        let found = &planned.found;
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

    fn banner(&self) -> String {
        let side = self.hello.teams.iter().find(|t| t.team == self.hello.team).map_or("?", |t| t.side.as_str());
        format!("{{name}} is WRTerritory {} | arms: {} | seat ai{} team {} {side}", ai_host::commit(), self.options.describe(), self.hello.ai_id, self.hello.team)
    }

    fn count_role(&self, role: Role) -> usize {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(role)).count()
    }

    fn curve_line(&self, tick: &Tick) {
        let game = self.game();
        let extractors = self.own.values().filter(|u| !u.being_built && self.unit_index(u.def).is_some_and(|i| game.units.list[i].extracts_metal > 0.0)).count();
        let build_power: f32 = self.own.values().filter(|u| !u.being_built && matches!(self.role_of_def(u.def), Some(Role::Commander | Role::Builder | Role::Factory | Role::Nano))).filter_map(|u| self.def(u.def)).map(|d| d.build_speed).sum();
        let (m, e) = (&tick.snapshot.metal, &tick.snapshot.energy);
        self.log(format_args!(
            "curve: extractors={extractors} constructors={} factories={} nanos={} build_power={build_power:.0} metal {:.0}/{:.0} +{:.1} -{:.1} energy {:.0}/{:.0} +{:.0} -{:.0} made={:.0} orders={} begun={} dropped={} skipped={} plans={}",
            self.count_role(Role::Builder), self.count_role(Role::Factory), self.count_role(Role::Nano), m.current, m.storage, m.income, m.usage, e.current, e.storage, e.income, e.usage,
            self.made, self.hands.orders.len(), self.hands.begun, self.hands.dropped, self.hands.skipped, self.plans
        ));
        self.log(format_args!("jobs: {}", self.jobs_line()));
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
            options: options::Options::from_env(),
            hands: hands::Hands::default(),
            spots: Vec::new(),
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
                Event::CommandRejected { unit, .. } => self.failed(*unit, "rejected"),
                Event::UnitDestroyed { unit, .. } => self.unit_lost(*unit),
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
            self.run_hands(&mut commands);
            self.run_nanos(&mut commands);
        }
        if tick.due() % MINUTE == 0 && tick.frame > 0 {
            self.curve_line(&tick);
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Territory>);
