//! The search beside the game: a snapshot goes out, the best plan found in the budget comes back
//! (docs/design/2026-10-20-territory-bot.md, decision 1). The first plan is searched on the engine thread.

use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use bot_protocol::UnitId;
use buildorder::anneal::{anneal_within, Found, Objective, Palette, Search};
use buildorder::game::Game;
use buildorder::lp::{self, LpOptions, LpPlan};
use buildorder::plan::{Item, Plan};
use buildorder::sim::{Scenario, State};
use buildorder::units::Role;

/// What a planner found: the anneal's plan or the program's targets.
pub(crate) enum Answer {
    Anneal(Found),
    Lp(LpPlan),
}

/// What a search was asked for and what it found.
pub(crate) struct Planned {
    pub answer: Answer,
    /// The snapshot's builders in queue order: the commander, the factories, the constructors.
    pub builders: Vec<UnitId>,
    pub t0: f64,
    pub horizon: f64,
    pub wall: Duration,
    /// The frame the snapshot was taken at.
    pub frame: i32,
}

pub(crate) struct Request {
    pub scenario: Scenario,
    pub state: State,
    pub builders: Vec<UnitId>,
    pub start: Option<Plan>,
    pub horizon: f64,
    pub terminal: f64,
    pub factories: usize,
    pub constructors: usize,
    pub seed: u64,
    pub frame: i32,
    /// The program's options when the planner is the program; none for the anneal.
    pub lp: Option<LpOptions>,
    /// Metal the hands could start a unit against now (the stock and `AFFORD_SECONDS` of income): the program is
    /// offered the types that cost no more, since a fraction of what cannot be started is energy and build power
    /// the engine never has (terr21-lp-nullai-comet-1: the fusion's fraction carried the program's energy from its
    /// second minute while the engine's first energy is solars). Factories are offered whatever they cost: T-COMMIT
    /// starts one whole on the same affordability, and a plant the program plans for from 4:00 stood at 5:49-6:38
    /// (terr21) where one it may not plan until affordable stood at 7:13-8:46 (terr23).
    pub affordable: f64,
}

pub(crate) struct Planner {
    pub game: Arc<Game>,
    pub palette: Palette,
    pending: Option<mpsc::Receiver<Planned>>,
}

/// The factory kinds the program may plan once a factory stands or is committed: the line the first lab opened. A
/// basic factory kind only if one of it stands or is committed; an advanced one only if a constructor of the
/// line's factories builds it, and so on up the line. None before the first factory: the program chooses the
/// line. The program on its own wants the vehicle line's advanced plant (0.5-0.9 of one by the end from 3:35 in
/// terr49's Armada games) through a vehicle lab it plans 0.1 of, while the bot's lab is a bot lab whose
/// constructors cannot build it; the advanced bot lab, its second choice, passed the commit threshold at 6:20
/// against the Cortex games' 4:17, and those made 733-846 at 20:00 against 566-731.
fn factory_line(game: &Game, state: &State) -> Option<Vec<usize>> {
    let units = &game.units;
    let basic = units.basic_extraction();
    let is_upgrade = |u: usize| units.list[u].extracts_metal > basic * 1.5;
    let advanced = |f: usize| units.list[f].builds.iter().any(|p| units.list[*p].builds.iter().any(|b| is_upgrade(*b)));
    // A standing factory is a builder of the state (it has build power), a framed or ordered one a commitment.
    let mut line: Vec<usize> = state.builders.iter().map(|b| b.unit).chain(state.committed.iter().map(|c| c.unit)).filter(|u| units.list[*u].role == Role::Factory).collect();
    line.sort_unstable();
    line.dedup();
    if line.is_empty() {
        return None;
    }
    // Up the line: the factories the line's constructors build, advanced ones only, to a fixpoint.
    loop {
        let before = line.len();
        let reachable: Vec<usize> = line
            .iter()
            .flat_map(|f| units.list[*f].builds.iter().copied())
            .flat_map(|c| units.list[c].builds.iter().copied())
            .filter(|f| units.list[*f].role == Role::Factory && advanced(*f) && !line.contains(f))
            .collect();
        line.extend(reachable);
        line.sort_unstable();
        line.dedup();
        if line.len() == before {
            return Some(line);
        }
    }
}

fn search_of(request: &Request) -> Search {
    Search {
        objective: Objective::Made { terminal: request.terminal },
        horizon: request.horizon,
        iterations: 0,
        seed: request.seed,
        factories: request.factories,
        constructors: request.constructors,
        hot: 0.02,
        start: request.start.clone(),
    }
}

fn run(game: &Game, palette: &Palette, request: Request, budget: Duration, threads: usize) -> Planned {
    let began = Instant::now();
    let answer = match &request.lp {
        Some(options) => {
            let line = factory_line(game, &request.state);
            let offer: Vec<usize> = palette
                .mobile
                .iter()
                .chain(palette.factory.iter())
                .filter_map(|i| match i { Item::Build(u) => Some(*u), _ => None })
                .filter(|u| if game.units.list[*u].role == Role::Factory { line.as_ref().is_none_or(|l| l.contains(u)) } else { game.units.list[*u].metal_cost <= request.affordable })
                .collect();
            match lp::solve(&game.units, &request.scenario, &request.state, &offer, options) {
                Ok(plan) => Answer::Lp(plan),
                Err(e) => {
                    eprintln!("[territory] the program did not solve ({e}); the anneal answers instead");
                    let search = search_of(&request);
                    Answer::Anneal(anneal_within(&game.units, &request.scenario, &request.state, palette, &search, budget, threads))
                }
            }
        }
        None => {
            let search = search_of(&request);
            Answer::Anneal(anneal_within(&game.units, &request.scenario, &request.state, palette, &search, budget, threads))
        }
    };
    Planned { answer, builders: request.builders, t0: request.state.t0, horizon: request.horizon, wall: began.elapsed(), frame: request.frame }
}

impl Planner {
    pub(crate) fn new(game: Arc<Game>, palette: Palette) -> Planner {
        Planner { game, palette, pending: None }
    }

    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some()
    }

    /// The search on the caller's thread: the first plan, at the first orders.
    pub(crate) fn blocking(&self, request: Request, budget: Duration, threads: usize) -> Planned {
        run(&self.game, &self.palette, request, budget, threads)
    }

    /// The search on its own threads; `poll` gives the answer.
    pub(crate) fn request(&mut self, request: Request, budget: Duration, threads: usize) {
        let (tx, rx) = mpsc::channel();
        let (game, palette) = (self.game.clone(), self.palette.clone());
        std::thread::spawn(move || {
            let planned = run(&game, &palette, request, budget, threads);
            let _ = tx.send(planned);
        });
        self.pending = Some(rx);
    }

    pub(crate) fn poll(&mut self) -> Option<Planned> {
        let rx = self.pending.as_ref()?;
        match rx.try_recv() {
            Ok(planned) => {
                self.pending = None;
                Some(planned)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                None
            }
        }
    }
}
