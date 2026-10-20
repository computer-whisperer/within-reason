//! The search beside the game: a snapshot goes out, the best plan found in the budget comes back
//! (docs/design/2026-10-20-territory-bot.md, decision 1). The first plan is searched on the engine thread.

use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use bot_protocol::UnitId;
use buildorder::anneal::{anneal_within, Found, Objective, Palette, Search};
use buildorder::game::Game;
use buildorder::plan::Plan;
use buildorder::sim::{Scenario, State};

/// What a search was asked for and what it found.
pub(crate) struct Planned {
    pub found: Found,
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
}

pub(crate) struct Planner {
    pub game: Arc<Game>,
    pub palette: Palette,
    pending: Option<mpsc::Receiver<Planned>>,
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
    let search = search_of(&request);
    let found = anneal_within(&game.units, &request.scenario, &request.state, palette, &search, budget, threads);
    Planned { found, builders: request.builders, t0: request.state.t0, horizon: request.horizon, wall: began.elapsed(), frame: request.frame }
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
