//! Runs the duels of one engine match. Both teams' AI sessions call into the same director, so it sees both
//! sides whole: it spawns the armies, sends them at each other, scores the fight and clears the field.
//!
//! A duel is two armies of one type each in a formation (`sites::Formation`); a scenario (`scenario.rs`) is a recorded
//! engagement, every unit at its own place with its own health, heading and first order. Both are an `Army` per
//! side: a list of `Spawn`s.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bot_protocol::{Command, Event, Hello, Terrain, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use micro::{Commitment, Lane, Stats, View};

use crate::fire::{self, Fire, Tally};
use crate::plan::{Force, Job, LaneMode, Sizing};
use crate::scenario::{self, Order, Scenario};
use crate::sites::{self, Footprint, Formation, FormationKind, Site};

const FPS: i32 = 30;
/// The armies stand this long after spawning before they are ordered forward.
const SETTLE: i32 = FPS;
/// An army that has not fully appeared by now never will (unknown unit, blocked ground).
const SPAWN_ALLOWANCE: i32 = 10 * FPS;
/// A fight nobody has been hurt in for this long is a stalemate (anti-air against anti-air).
const STALEMATE: i32 = 60 * FPS;
/// After the last unit is gone the field rests this long: explosions finish, wrecks settle.
const REST: i32 = 2 * FPS;
/// Wrecks are blown up by crawling bombs on a grid this fine; the blast (the small one, since neighbours set each
/// other off) reaches 140 elmos.
const SWEEP_SPACING: f32 = 200.0;
/// A self-destruct order still not carried out after this long was cancelled and is given again. (The countdown is
/// 5 s. BAR's "Self-Destruct Resign" gadget cancels the first two attempts of a team to destroy 95% of its units at
/// once, which a big surviving army beside one commander is.)
const DESTRUCT_RETRY: i32 = 8 * FPS;
const SWEEP_BOMBS: [&str; 2] = ["corroach", "armvader"];
/// Self-destruct takes a few seconds, retries and bombing a few more; a site still not clear by now is abandoned.
const CLEANUP_ALLOWANCE: i32 = 120 * FPS;
/// A spawned unit is recognised by type and by standing no further than this from the place it was given at (a
/// tight formation pushes units apart as they appear); bombs by standing this near the wreckage.
const CLAIM_MARGIN: f32 = 150.0;
/// A scenario's units stand where a real fight left them: a tighter test, so neighbours of one type are not swapped.
const SCENARIO_CLAIM: f32 = 80.0;
/// What hurts a scenario's units to their recorded health (`scenario.rs`): the Rover, a beam laser (no spray, no
/// area: the Pawn's sprayed gun, tried first, hurt the target's neighbours and killed some), 35 a shot, one a second,
/// range 180.
const HURTER: &str = "armfav";
/// How far from its target a hurter may be given (inside its range), and the room it wants round it.
const HURTER_STANDOFFS: [f32; 2] = [100.0, 130.0];
/// A hurter that has not hurt its target for this long is walked up to `HURTER_CLOSE` of it and told again (a rise
/// between them can keep its beam off: seen from 160 with 37 elmos of height between them).
const HURTER_PATIENCE: i32 = 8 * FPS;
const HURTER_CLOSE: f32 = 70.0;
const HURTER_ROOM: f32 = 45.0;
/// The hurters are dismissed by now whatever is left undone (counted in `health_error`): a Stout from full to 5%
/// takes the Rover 49 s.
const HURT_ALLOWANCE: i32 = 90 * FPS;
/// The hurting phase ends by now whatever is left of it.
const PREPARE_ALLOWANCE: i32 = 150 * FPS;
/// A unit with a heading is sent this far along it before its first order.
const HEADING_STEP: f32 = 24.0;
/// The shape instrument samples the armies at first damage and this much later (`run/replays/shapes.py`: t0 and
/// t0 + 10 s).
const SHAPE_LATER: i32 = 10 * FPS;

/// What the matches of a batch share: the duels still to run and where results go.
pub struct Batch {
    pub queue: Mutex<VecDeque<Job>>,
    pub results: Mutex<Vec<DuelResult>>,
    pub sizing: Sizing,
    /// Game seconds after which an undecided fight is scored as it stands.
    pub time_limit: i32,
    /// Rounds of bombing after each duel: wrecks take one or two to become heaps, heaps another.
    pub sweep_waves: u32,
    /// How the two armies (`x`, `y`) stand and advance, per shape of the batch (`Job::shape`). Tight ranks flatter
    /// area damage.
    pub shapes: Vec<[Formation; 2]>,
    /// Whether the control lane (`crates/micro`) runs for each army (`x`, `y`) over the director's orders.
    pub lanes: [LaneMode; 2],
    /// The simulator's unit table, for the lane's fighting numbers.
    pub units: combatsim::units::Units,
    /// Fight this recorded engagement in every job instead of a pairing (`duel --scenario`).
    pub scenario: Option<Scenario>,
    pub on_result: Box<dyn Fn(&DuelResult) + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct DuelResult {
    pub match_index: usize,
    pub site: usize,
    /// Position of this duel in the site's sequence; wrecks of earlier duels lie on the field.
    pub sequence: u32,
    pub job: Job,
    pub count: [u32; 2],
    /// Each army's metal at the start, each unit weighted by its health then (a duel's units start whole).
    pub metal: [f32; 2],
    /// "x", "y" or "draw".
    pub winner: &'static str,
    /// "wiped", "timeout", "stalemate" or "spawn_failed".
    pub reason: &'static str,
    pub seconds: f32,
    /// Seconds from the advance order to the first damage.
    pub contact_seconds: Option<f32>,
    pub survivors: [u32; 2],
    /// Surviving metal value, each survivor weighted by its health, as a fraction of `metal`.
    pub value_left: [f32; 2],
    /// Hit points of damage each army took in the fight.
    pub damage_taken: [f32; 2],
    /// How spread out each army was when the first shot landed: root-mean-square distance of its units from its
    /// own centre, in elmos. The probe `docs/studies/combat-sim.md` asks for first: an army's fighting formation
    /// is an outcome, not the spacing it was spawned at.
    pub spread_at_contact: [f32; 2],
    /// How each army stood (`Formation::label`, or "scenario").
    pub formation: [String; 2],
    /// Each army's fire (`fire.rs`, `run/fire.py`'s definitions).
    pub fire: [Tally; 2],
    /// Each army's shape at contact (`fire.rs` `Shape`, `run/replays/shapes.py`'s definitions).
    pub shape: [fire::Shape; 2],
    /// How each army was driven (`LaneMode::label`).
    pub lane: [String; 2],
    /// A scenario's units' mean distance between the health they started the fight with and the file's (a share of
    /// full health); 0 in a duel.
    pub health_error: [f32; 2],
}

/// One unit to give.
struct Spawn {
    def: UnitDefId,
    name: String,
    at: Vec3,
    metal: f32,
    /// The type's reach and reload (`UnitDefInfo`), for the fire instrument.
    reach: f32,
    reload: f32,
    mobile: bool,
    /// A scenario's unit as the file has it (health, heading, first order); `None` in a duel.
    plan: Option<scenario::Unit>,
    /// Given in the first round and hurt to its health before everyone else appears.
    hurt: bool,
}

/// A unit given to one team to hurt a unit of the other to its recorded health (`scenario.rs`).
struct Hurter {
    /// The other army's spawn it hurts.
    target: usize,
    at: Vec3,
    unit: Option<UnitId>,
    attacking: bool,
    /// The target's health when it last went down, and when.
    last_health: f32,
    last_progress: i32,
    destruct_at: Option<i32>,
    gone: bool,
}

struct Army {
    team: i32,
    spawns: Vec<Spawn>,
    /// A duel's shape; `None` for a scenario's side.
    formation: Option<Formation>,
    /// Spawn rounds given so far: 1 the units to hurt and the hurters (a scenario only), 2 everything.
    given: u8,
    destruct_ordered: Option<i32>,
    units: HashSet<UnitId>,
    spawn_of: HashMap<UnitId, usize>,
    unit_of: HashMap<usize, UnitId>,
    /// Health as a fraction, per living unit, as of `reported`.
    alive: HashMap<UnitId, f32>,
    max_health: HashMap<UnitId, f32>,
    /// Health each unit started the fight with; `value_left` is measured against it.
    start: HashMap<UnitId, f32>,
    /// Where each unit was last seen; a unit that dies leaves its wreck there.
    seen_at: HashMap<UnitId, Vec3>,
    centroid: Vec3,
    /// Root-mean-square distance of the living units from `centroid`, this tick.
    dispersion: f32,
    /// `dispersion` at the tick the duel's first damage was seen.
    spread_at_contact: Option<f32>,
    damage_taken: f32,
    /// The part of `damage_taken` the enemy army's units did (the rest is friendly fire and the unattributed).
    taken_from_enemy: f32,
    fire: Fire,
    /// Units told to hold fire while a scenario is prepared.
    held: HashSet<UnitId>,
    /// Hurters this army's team gives, for the other army's units, and whether they have been given.
    hurters: Vec<Hurter>,
    hurters_given: bool,
    /// Frame the first orders went out; the idle rule leaves a scenario's units alone for a second after it.
    ordered_at: Option<i32>,
    /// Frame of the last tick this army's team reported on.
    reported: i32,
    /// The control lane over this army's orders, when the batch runs it for this side.
    lane: Option<Lane>,
    lane_mode: LaneMode,
    /// The shape instrument's samples.
    shape: fire::Shape,
}

impl Army {
    fn new(team: i32, spawns: Vec<Spawn>, formation: Option<Formation>, centroid: Vec3, lane_mode: LaneMode) -> Army {
        Army {
            team,
            spawns,
            formation,
            given: 0,
            destruct_ordered: None,
            units: HashSet::new(),
            spawn_of: HashMap::new(),
            unit_of: HashMap::new(),
            alive: HashMap::new(),
            max_health: HashMap::new(),
            start: HashMap::new(),
            seen_at: HashMap::new(),
            centroid,
            dispersion: 0.0,
            spread_at_contact: None,
            damage_taken: 0.0,
            taken_from_enemy: 0.0,
            fire: Fire::default(),
            held: HashSet::new(),
            hurters: Vec::new(),
            hurters_given: false,
            ordered_at: None,
            reported: -1,
            lane: (lane_mode != LaneMode::Off).then(Lane::default),
            lane_mode,
            shape: fire::Shape::default(),
        }
    }

    fn start_of(&self, unit: UnitId) -> f32 {
        self.start.get(&unit).copied().unwrap_or(1.0)
    }

    /// Metal at the start of the fight, each unit weighted by its health then.
    fn metal(&self) -> f32 {
        (self.unit_of.iter()).map(|(&i, &u)| self.spawns[i].metal * self.start_of(u)).sum()
    }

    fn value_left(&self) -> f32 {
        let left: f32 = (self.alive.iter()).map(|(u, h)| self.spawns[self.spawn_of[u]].metal * h).sum();
        (left / self.metal().max(1.0)).max(0.0)
    }

    /// Whether every spawn of the round given so far has appeared.
    fn complete(&self, round: u8) -> bool {
        (self.spawns.iter().enumerate()).filter(|(_, s)| round >= 2 || s.hurt).all(|(i, _)| self.unit_of.contains_key(&i))
    }
}

enum Phase {
    Spawning { since: i32, round: u8 },
    /// A scenario's first-round units have appeared and hold fire; after `SETTLE` (they push each other apart as
    /// they appear) the hurters are placed by where they stand and given.
    Arming { since: i32, planned: bool },
    /// A scenario's first-round units are hurt to their recorded health; then the hurters are dismissed together.
    Hurting { since: i32, dismissed: Option<i32> },
    Fighting { advance_at: i32, first_damage: Option<i32>, last_damage: i32 },
    Clearing { since: i32, sweep: Sweep },
}

/// Clearing the field: first the survivors destroy themselves, then bombs are spawned over the wrecks and set off,
/// `waves_left` times, then the field rests.
enum Sweep {
    Survivors,
    Bombs { waves_left: u32, spawned_at: Option<i32>, set_off: HashMap<UnitId, i32> },
    Rest { since: i32 },
    Done,
}

struct Duel {
    job: Job,
    /// `x` then `y`.
    armies: [Army; 2],
    phase: Phase,
    sequence: u32,
    /// Frame of the fire instrument's next once-a-second sample.
    next_sample: i32,
    /// Frames the shape instrument still has to sample at (first damage, and ten seconds later).
    shape_due: Vec<i32>,
    /// Bounding box (min x, min z, max x, max z) of where units have died.
    wreckage: Option<[f32; 4]>,
}

impl Duel {
    fn is_scenario(&self) -> bool {
        self.armies[0].formation.is_none()
    }

    fn wreck_at(&mut self, at: Vec3) {
        let [x0, z0, x1, z1] = self.wreckage.unwrap_or([at.x, at.z, at.x, at.z]);
        self.wreckage = Some([x0.min(at.x), z0.min(at.z), x1.max(at.x), z1.max(at.z)]);
    }
}

struct Field {
    site: Site,
    duel: Option<Duel>,
    fought: u32,
    abandoned: bool,
}

pub struct Director {
    batch: Arc<Batch>,
    match_index: usize,
    wanted_sites: usize,
    /// No more than this many duels are started in this match (wrecks pile up on the sites).
    duel_budget: u32,
    defs: HashMap<String, UnitDefInfo>,
    defs_by_id: HashMap<UnitDefId, UnitDefInfo>,
    /// Lane rule IDs switched off for this batch (`WITHIN_REASON_DISABLE`).
    disabled: Vec<String>,
    hello: Option<Hello>,
    /// First sighting of each team's own units: the commander.
    commanders: HashMap<i32, Vec3>,
    fields: Option<Vec<Field>>,
    pub last_tick: Instant,
    pub finished: bool,
    pub fatal: Option<String>,
}

impl Director {
    pub fn new(batch: Arc<Batch>, match_index: usize, wanted_sites: usize, duel_budget: u32) -> Self {
        Director {
            batch,
            match_index,
            wanted_sites,
            duel_budget,
            defs: HashMap::new(),
            defs_by_id: HashMap::new(),
            disabled: std::env::var("WITHIN_REASON_DISABLE").map_or_else(|_| Vec::new(), |ids| ids.split(',').map(str::to_string).collect()),
            hello: None,
            commanders: HashMap::new(),
            fields: None,
            last_tick: Instant::now(),
            finished: false,
            fatal: None,
        }
    }

    pub fn hello(&mut self, hello: Hello) {
        if self.hello.is_none() {
            self.defs = hello.unit_defs.iter().map(|d| (d.name.clone(), d.clone())).collect();
            self.defs_by_id = hello.unit_defs.iter().map(|d| (d.id, d.clone())).collect();
            self.hello = Some(hello);
        }
    }

    /// Duels that were running when the match stopped; they go back on the queue.
    pub fn unfinished(&mut self) -> Vec<Job> {
        self.fields.iter_mut().flatten().filter_map(|f| f.duel.take()).map(|d| d.job).collect()
    }

    pub fn tick(&mut self, team: i32, tick: &Tick) -> Vec<Command> {
        self.last_tick = Instant::now();
        if self.finished || self.fatal.is_some() {
            return Vec::new();
        }
        if self.fields.is_none() {
            if let Some(unit) = tick.snapshot.own_units.first() {
                self.commanders.entry(team).or_insert(unit.pos);
            }
            if self.commanders.len() < 2 {
                return Vec::new();
            }
            self.lay_out_fields();
            if self.fatal.is_some() {
                return Vec::new();
            }
        }
        let mut commands = Vec::new();
        let mut fields = self.fields.take().unwrap_or_default();
        let hurter = self.defs.get(HURTER).map(|d| d.id);
        for (index, field) in fields.iter_mut().enumerate() {
            if field.duel.is_none() && !field.abandoned && self.duel_budget > 0 {
                field.duel = self.next_duel(field);
            }
            let Some(duel) = &mut field.duel else { continue };
            let bomb = SWEEP_BOMBS.iter().find_map(|name| self.defs.get(*name)).map(|def| def.id);
            let sweep = bomb.map(|bomb| (bomb, self.batch.sweep_waves));
            let hello = self.hello.as_ref().expect("hello precedes ticks");
            let rules = Rules {
                time_limit: self.batch.time_limit,
                sweep,
                hurter,
                centre: field.site.centre,
                defs: &self.defs_by_id,
                units: &self.batch.units,
                terrain: &hello.terrain,
                commanders: &self.commanders,
                disabled: &self.disabled,
            };
            if let Some(result) = advance(duel, team, tick, &rules, &mut commands) {
                let result = DuelResult { match_index: self.match_index, site: index, ..result };
                (self.batch.on_result)(&result);
                self.batch.results.lock().unwrap().push(result);
            }
            if let Phase::Clearing { since, sweep } = &duel.phase {
                if matches!(sweep, Sweep::Done) {
                    field.duel = None;
                } else if tick.frame - *since > CLEANUP_ALLOWANCE {
                    eprintln!("match {}: site {index} would not clear; abandoned", self.match_index);
                    field.duel = None;
                    field.abandoned = true;
                }
            }
        }
        let idle = fields.iter().all(|f| f.duel.is_none());
        let usable = fields.iter().any(|f| !f.abandoned);
        self.fields = Some(fields);
        if idle && (self.duel_budget == 0 || !usable || self.batch.queue.lock().unwrap().is_empty()) {
            self.finished = true;
        }
        commands
    }

    fn lay_out_fields(&mut self) {
        let hello = self.hello.as_ref().expect("hello precedes ticks");
        if let Some(scenario) = &self.batch.scenario {
            let unknown = scenario.sides.iter().flat_map(|s| &s.units).find(|u| !self.defs.contains_key(&u.kind));
            if let Some(unit) = unknown {
                self.fatal = Some(format!("the game has no unit called {}", unit.kind));
            } else if !self.defs.contains_key(HURTER) {
                self.fatal = Some(format!("the game has no {HURTER} to hurt units with"));
            }
            let [x, z] = scenario.centre;
            let centre = Vec3 { x, y: ground(&hello.terrain, x, z), z };
            let near: Vec<String> = (self.commanders.values())
                .filter(|c| c.dist2d(centre) < 2000.0)
                .map(|c| format!("({:.0},{:.0}) {:.0} away", c.x, c.z, c.dist2d(centre)))
                .collect();
            let near = if near.is_empty() { "none".to_string() } else { near.join(" ") };
            eprintln!("match {}: scenario at ({x:.0},{z:.0}); commanders within 2000: {near}", self.match_index);
            self.fields = Some(vec![Field { site: Site { centre }, duel: None, fought: 0, abandoned: false }]);
            return;
        }
        let queue = self.batch.queue.lock().unwrap();
        let mut max_slope = 1.0f32;
        for name in queue.iter().flat_map(|job| [&job.x, &job.y]).flat_map(|spec| Force::parse(spec).parts).map(|(name, _)| name) {
            match self.defs.get(&name) {
                None => self.fatal = Some(format!("the game has no unit called {name}")),
                Some(def) => max_slope = max_slope.min(def.move_class.map_or(1.0, |class| class.max_slope)),
            }
        }
        // The ground must hold the largest army of the batch in its formation.
        let mut armies = Vec::new();
        for job in queue.iter() {
            if let Some([x, y]) = self.sized(job) {
                let [form_x, form_y] = self.batch.shapes[job.shape];
                armies.extend([(form_x, x.iter().map(|(_, n)| n).sum()), (form_y, y.iter().map(|(_, n)| n).sum())]);
            }
        }
        drop(queue);
        let footprint = Footprint::holding(armies);
        let commanders: Vec<Vec3> = self.commanders.values().copied().collect();
        let sites = sites::choose(&hello.terrain, max_slope, &commanders, self.wanted_sites, footprint);
        if sites.is_empty() {
            self.fatal = Some(format!("no flat site of {:.0} x {:.0} found on {}", footprint.length, footprint.width, hello.map.name));
        }
        let at: Vec<String> = sites.iter().map(|s| format!("({:.0},{:.0})", s.centre.x, s.centre.z)).collect();
        let commanders: Vec<String> = commanders.iter().map(|c| format!("({:.0},{:.0})", c.x, c.z)).collect();
        eprintln!(
            "match {}: {} site(s) of {:.0} x {:.0} at {}; commanders at {}",
            self.match_index, sites.len(), footprint.length, footprint.width, at.join(" "), commanders.join(" ")
        );
        self.fields = Some(sites.into_iter().map(|site| Field { site, duel: None, fought: 0, abandoned: false }).collect());
    }

    fn spawn(def: &UnitDefInfo, at: Vec3, plan: Option<scenario::Unit>) -> Spawn {
        let hurt = plan.as_ref().is_some_and(|u| u.needs_hurting(None));
        Spawn { def: def.id, name: def.name.clone(), at, metal: def.metal_cost, reach: def.reach, reload: def.reload, mobile: def.speed > 0.0, plan, hurt }
    }

    /// The two forces of a job with their counts: a plain unit name takes the batch's sizing against the other side
    /// (equal metal, or the count); a mixed force (`name*count+...`) is spawned as written, and a plain name against
    /// it takes the count that matches its metal. `None` when a name is unknown.
    fn sized(&self, job: &Job) -> Option<[Vec<(String, u32)>; 2]> {
        let (x, y) = (Force::parse(&job.x), Force::parse(&job.y));
        let metal = |force: &Force| -> Option<f32> { force.parts.iter().map(|(n, c)| Some(self.defs.get(n)?.metal_cost * *c as f32)).sum() };
        let cost = |force: &Force| -> Option<f32> { Some(self.defs.get(&force.parts[0].0)?.metal_cost) };
        Some(match (x.is_plain(), y.is_plain()) {
            (true, true) => {
                let (nx, ny) = self.batch.sizing.counts(cost(&x)?, cost(&y)?);
                [vec![(x.parts[0].0.clone(), nx)], vec![(y.parts[0].0.clone(), ny)]]
            }
            (true, false) => {
                let nx = (metal(&y)? / cost(&x)?).round().max(1.0) as u32;
                [vec![(x.parts[0].0.clone(), nx)], y.parts]
            }
            (false, true) => {
                let ny = (metal(&x)? / cost(&y)?).round().max(1.0) as u32;
                [x.parts, vec![(y.parts[0].0.clone(), ny)]]
            }
            (false, false) => [x.parts, y.parts],
        })
    }

    fn next_duel(&mut self, field: &mut Field) -> Option<Duel> {
        let job = loop {
            let job = self.batch.queue.lock().unwrap().pop_front()?;
            // Two buildings never meet.
            let mobile = |spec: &str| Force::parse(spec).parts.iter().any(|(n, _)| self.defs.get(n).is_some_and(|d| d.speed > 0.0));
            if self.batch.scenario.is_some() || mobile(&job.x) || mobile(&job.y) {
                break job;
            }
        };
        self.duel_budget -= 1;
        let teams = [job.x_team(), 1 - job.x_team()];
        let armies = if let Some(scenario) = &self.batch.scenario {
            let terrain = &self.hello.as_ref().expect("hello precedes ticks").terrain;
            let centre = field.site.centre;
            [0, 1].map(|side| {
                let spawns = (scenario.sides[side].units.iter())
                    .map(|u| Self::spawn(&self.defs[&u.kind], Vec3 { x: u.x, y: ground(terrain, u.x, u.z), z: u.z }, Some(u.clone())))
                    .collect();
                Army::new(teams[side], spawns, None, centre, self.batch.lanes[side])
            })
        } else {
            let [force_x, force_y] = self.sized(&job)?;
            let [form_x, form_y] = self.batch.shapes[job.shape];
            let army = |force: &[(String, u32)], team: i32, west: bool, formation: Formation, lane: LaneMode| {
                let front = field.site.end(west);
                let count = force.iter().map(|(_, n)| n).sum();
                let defs: Vec<&UnitDefInfo> = force.iter().flat_map(|(n, c)| std::iter::repeat_n(&self.defs[n], *c as usize)).collect();
                let spawns = formation.places(front, west, count).into_iter().zip(defs).map(|(at, def)| Self::spawn(def, at, None)).collect();
                Army::new(team, spawns, Some(formation), front, lane)
            };
            [
                army(&force_x, teams[0], job.x_is_west(), form_x, self.batch.lanes[0]),
                army(&force_y, teams[1], !job.x_is_west(), form_y, self.batch.lanes[1]),
            ]
        };
        field.fought += 1;
        Some(Duel { job, armies, phase: Phase::Spawning { since: -1, round: 1 }, sequence: field.fought, next_sample: 0, shape_due: Vec::new(), wreckage: None })
    }
}

/// Ground height at a point, from the terrain grid (0 off the map or without one).
fn ground(terrain: &Terrain, x: f32, z: f32) -> f32 {
    if terrain.cell <= 0.0 || terrain.width == 0 {
        return 0.0;
    }
    let (col, row) = ((x / terrain.cell) as i64, (z / terrain.cell) as i64);
    if col < 0 || row < 0 || col >= i64::from(terrain.width) || row >= i64::from(terrain.height) {
        return 0.0;
    }
    f32::from(terrain.heights[row as usize * terrain.width as usize + col as usize].max(0))
}

/// Each unit of one side to be hurt gets a hurter from the other side's team, 100-160 from where it stands in the
/// direction (of sixteen) whose line to it passes furthest from every other unit standing and every hurter already
/// placed (a beam stops at the first hull it meets), with room round it.
fn plan_hurters(duel: &mut Duel) {
    let mut standing: Vec<Vec3> = duel.armies.iter().flat_map(|a| a.units.iter().filter_map(|u| a.seen_at.get(u).copied())).collect();
    for side in 0..2 {
        let army = &duel.armies[side];
        let targets: Vec<(usize, Vec3)> = (army.unit_of.iter()).filter_map(|(&i, u)| Some((i, army.seen_at.get(u).copied()?))).collect();
        for (target, at) in targets {
            // Scored by the line's clearance, but never closer than `HURTER_ROOM` to anybody: a tank that is pushed
            // into a Rover crushes it (the first placement lost a third of the hurters that way).
            let score = |(out, d): ((f32, f32), f32)| {
                let from = Vec3 { x: at.x + out.0 * d, y: at.y, z: at.z + out.1 * d };
                let others = standing.iter().filter(|p| p.dist2d(at) > 1.0);
                let line = others.clone().map(|p| near_segment(*p, from, at)).fold(f32::INFINITY, f32::min);
                let room = standing.iter().map(|p| p.dist2d(from)).fold(f32::INFINITY, f32::min);
                (room >= HURTER_ROOM, line.min(200.0), room)
            };
            let candidates = (0..16).flat_map(|k| HURTER_STANDOFFS.map(|d| ((k as f32 * std::f32::consts::TAU / 16.0).sin_cos(), d)));
            let (out, d) = candidates.max_by(|a, b| score(*a).partial_cmp(&score(*b)).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(((0.0, 1.0), HURTER_STANDOFFS[0]));
            let place = |d: f32| Vec3 { x: at.x + out.0 * d, y: at.y, z: at.z + out.1 * d };
            standing.push(place(d));
            duel.armies[1 - side].hurters.push(Hurter {
                target,
                at: place(d),
                unit: None,
                attacking: false,
                last_health: 1.0,
                last_progress: 0,
                destruct_at: None,
                gone: false,
            });
        }
    }
}

/// Distance from `p` to the segment `a`-`b`, in the ground plane.
fn near_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let l2 = (dx * dx + dz * dz).max(1e-6);
    let t = (((p.x - a.x) * dx + (p.z - a.z) * dz) / l2).clamp(0.0, 1.0);
    (p.x - (a.x + t * dx)).hypot(p.z - (a.z + t * dz))
}

/// What every duel of the batch is fought under, and what the lane reads of the match.
struct Rules<'a> {
    time_limit: i32,
    /// The bomb unit and how many rounds of it clear the wrecks afterwards.
    sweep: Option<(UnitDefId, u32)>,
    /// The hurter's type (`HURTER`).
    hurter: Option<UnitDefId>,
    centre: Vec3,
    defs: &'a HashMap<UnitDefId, UnitDefInfo>,
    units: &'a combatsim::units::Units,
    terrain: &'a Terrain,
    /// Where each team's commander stands: the lane's `home` (a flee prefers cells toward it among equals).
    commanders: &'a HashMap<i32, Vec3>,
    /// Lane rule IDs switched off (`WITHIN_REASON_DISABLE`).
    disabled: &'a [String],
}

/// What the lane reads of a duel: the game's definitions, the simulator's numbers, the map, and the other army.
struct DuelView<'a> {
    rules: &'a Rules<'a>,
    team: i32,
    side: usize,
    own: &'a HashSet<UnitId>,
    enemy: &'a HashSet<UnitId>,
    mode: LaneMode,
}

impl View for DuelView<'_> {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.rules.defs.get(&def)
    }

    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        let name = &self.rules.defs.get(&def)?.name;
        let unit = &self.rules.units.list[self.rules.units.index(name)?];
        let dgun = unit.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max);
        Some(Stats { reach: unit.reach(), dps: unit.dps(), speed: unit.speed, health: unit.health, dgun })
    }

    fn grid_spec(&self) -> (f32, usize, usize) {
        let t = self.rules.terrain;
        (t.cell, t.width as usize, t.height as usize)
    }

    fn passable(&self) -> Option<&[bool]> {
        None
    }

    fn snap(&self, pos: Vec3) -> Vec3 {
        pos
    }

    fn home(&self) -> Vec3 {
        self.rules.commanders.get(&self.team).copied().unwrap_or(self.rules.centre)
    }

    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        Vec::new()
    }

    fn building_at(&self, _id: UnitId) -> Option<Vec3> {
        None
    }

    fn enemy_known(&self, id: UnitId) -> bool {
        self.enemy.contains(&id)
    }

    fn blip_def(&self) -> Option<UnitDefId> {
        None
    }

    /// `old` is the lane without H-MICRO-FORM; `WITHIN_REASON_DISABLE` (rule IDs, comma-separated) switches rules
    /// off as it does in the bot, for ablations in the harness.
    fn enabled(&self, rule: &str) -> bool {
        if self.mode == LaneMode::Old && rule == "H-MICRO-FORM" {
            return false;
        }
        !self.rules.disabled.iter().any(|id| id == rule)
    }

    fn mine(&self, unit: UnitId) -> bool {
        self.own.contains(&unit)
    }

    fn label(&self) -> String {
        format!("[duel side {}]", self.side)
    }
}

/// Where each of `units` is sent when a `Line` advances: the enemy's centre shifted across the approach by the
/// unit's place in the line, `spacing` apart, units keeping their left-to-right order.
fn line_abreast(units: &mut [(UnitId, Vec3)], from: Vec3, to: Vec3, spacing: f32) -> Vec<(UnitId, Vec3)> {
    let (dx, dz) = (to.x - from.x, to.z - from.z);
    let len = dx.hypot(dz).max(1.0);
    let across = (-dz / len, dx / len);
    let sideways = |p: &Vec3| p.x * across.0 + p.z * across.1;
    units.sort_by(|a, b| sideways(&a.1).total_cmp(&sideways(&b.1)));
    let n = units.len() as f32;
    (units.iter().enumerate())
        .map(|(file, (id, _))| {
            let side = (file as f32 - (n - 1.0) / 2.0) * spacing;
            (*id, Vec3 { x: to.x + across.0 * side, y: to.y, z: to.z + across.1 * side })
        })
        .collect()
}

/// Recognises the units this team was just given: each new unit of ours takes the nearest unclaimed spawn of its
/// type given so far, within `margin`; a hurter takes its place likewise.
fn claim(army: &mut Army, tick: &Tick, hurter: Option<UnitDefId>, margin: f32) {
    let taken: HashSet<UnitId> = army.hurters.iter().filter_map(|h| h.unit).chain(army.units.iter().copied()).collect();
    for unit in tick.snapshot.own_units.iter().filter(|u| !taken.contains(&u.id)) {
        if Some(unit.def) == hurter {
            let free = army.hurters.iter_mut().filter(|h| h.unit.is_none() && h.at.dist2d(unit.pos) < SCENARIO_CLAIM);
            if let Some(h) = free.min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos))) {
                h.unit = Some(unit.id);
                continue;
            }
        }
        let round = army.given;
        let free = (army.spawns.iter().enumerate())
            .filter(|(i, s)| s.def == unit.def && (round >= 2 || s.hurt) && !army.unit_of.contains_key(i) && s.at.dist2d(unit.pos) < margin);
        if let Some((i, _)) = free.min_by(|a, b| a.1.at.dist2d(unit.pos).total_cmp(&b.1.at.dist2d(unit.pos))) {
            army.units.insert(unit.id);
            army.spawn_of.insert(unit.id, i);
            army.unit_of.insert(i, unit.id);
        }
    }
}

/// One team's tick for one duel. Returns the result on the tick that decides it.
fn advance(duel: &mut Duel, team: i32, tick: &Tick, rules: &Rules, commands: &mut Vec<Command>) -> Option<DuelResult> {
    let frame = tick.frame;
    let scenario = duel.is_scenario();
    let side = duel.armies.iter().position(|a| a.team == team)?;
    let enemy_centroid = duel.armies[1 - side].centroid;
    let mut wrecks = Vec::new();
    let standing: HashMap<UnitId, Vec3> = tick.snapshot.own_units.iter().map(|u| (u.id, u.pos)).collect();
    let fighting = matches!(duel.phase, Phase::Fighting { .. });
    let mut hurt = false;
    {
        let [first, second] = &mut duel.armies;
        let (army, enemy) = if side == 0 { (first, &*second) } else { (second, &*first) };

        // What this team's snapshot says about its army.
        if matches!(duel.phase, Phase::Spawning { .. } | Phase::Arming { .. }) {
            claim(army, tick, rules.hurter, if scenario { SCENARIO_CLAIM } else { CLAIM_MARGIN });
        }
        army.alive.clear();
        let mut sum = (0.0, 0.0);
        for unit in tick.snapshot.own_units.iter().filter(|u| army.units.contains(&u.id)) {
            army.alive.insert(unit.id, (unit.health / unit.max_health.max(1.0)).clamp(0.0, 1.0));
            army.max_health.insert(unit.id, unit.max_health);
            army.seen_at.insert(unit.id, unit.pos);
            sum = (sum.0 + unit.pos.x, sum.1 + unit.pos.z);
        }
        let alive = &army.alive;
        let lost: Vec<(UnitId, Vec3)> = army.seen_at.extract_if(|unit, _| !alive.contains_key(unit)).collect();
        if !fighting && army.ordered_at.is_none() {
            // Died while the scenario was prepared: it is given again, whole, with everyone else.
            for (unit, _) in &lost {
                if let Some(i) = army.spawn_of.remove(unit) {
                    army.units.remove(unit);
                    army.unit_of.remove(&i);
                    army.spawns[i].hurt = false;
                    eprintln!("scenario: side {side} lost unit {i} of the file while it was prepared; it is given again, whole");
                }
            }
        }
        wrecks.extend(lost.into_iter().map(|(_, at)| at));
        for hurter in army.hurters.iter_mut().filter(|h| !h.gone) {
            if let Some(unit) = hurter.unit
                && !standing.contains_key(&unit)
            {
                hurter.gone = true;
                wrecks.push(hurter.at);
            }
        }
        if !army.alive.is_empty() {
            let n = army.alive.len() as f32;
            army.centroid = Vec3 { x: sum.0 / n, y: army.centroid.y, z: sum.1 / n };
            let centroid = army.centroid;
            let spread: f32 = army.units.iter().filter_map(|u| standing.get(u)).map(|p| p.dist2d(centroid).powi(2)).sum();
            army.dispersion = (spread / n).sqrt();
        }
        army.reported = frame;
        for event in &tick.events {
            match event {
                Event::UnitDamaged { unit, damage, attacker, .. } if fighting && army.units.contains(unit) => {
                    army.damage_taken += damage;
                    hurt = true;
                    // Damage is booked to the army that did it: to itself (friendly fire) or to the enemy army.
                    match attacker {
                        Some(a) if army.units.contains(a) => {
                            army.fire.tally.friendly_fire += damage;
                            let name = |u: &UnitId| army.spawn_of.get(u).map(|&i| army.spawns[i].name.clone()).unwrap_or_default();
                            *army.fire.tally.victims.entry((name(a), name(unit))).or_default() += damage;
                        }
                        Some(a) if enemy.units.contains(a) => army.taken_from_enemy += damage,
                        _ => {}
                    }
                }
                Event::WeaponFired { unit, .. } if fighting && army.units.contains(unit) => army.fire.shot(*unit),
                _ => {}
            }
        }
        // A scenario's units hold fire until the fight starts; so do the hurters, except at their targets.
        if scenario && army.ordered_at.is_none() {
            let hurters = army.hurters.iter().filter_map(|h| h.unit);
            let unheld: Vec<UnitId> = army.units.iter().copied().chain(hurters).filter(|u| !army.held.contains(u)).collect();
            for unit in unheld {
                army.held.insert(unit);
                // Holding fire only stops a weapon choosing a new target; a stop drops the one it may have taken in
                // the frames since it appeared (`CCommandAI::ExecuteStop`).
                commands.push(Command::FireState { unit, state: 0 });
                commands.push(Command::Stop { unit });
            }
        }
    }

    let result = match &mut duel.phase {
        Phase::Spawning { since, round } => {
            if *since < 0 {
                *since = frame;
            }
            let (since, round) = (*since, *round);
            let nobody_hurt = duel.armies.iter().all(|a| a.spawns.iter().all(|s| !s.hurt));
            let army = &mut duel.armies[side];
            if army.given < round {
                // A duel, or a scenario with nobody to hurt, gives everything at once.
                let everything = round >= 2 || !scenario || nobody_hurt;
                army.given = if everything { 2 } else { 1 };
                let wanted = (army.spawns.iter().enumerate()).filter(|(i, s)| if everything { !army.unit_of.contains_key(i) } else { s.hurt });
                commands.extend(wanted.map(|(_, s)| Command::GiveUnit { def: s.def, at: s.at }));
            }
            let given = duel.armies.iter().map(|a| a.given).min().unwrap_or(0);
            if given >= round && duel.armies.iter().all(|a| a.complete(given)) {
                duel.phase = if given == 1 {
                    Phase::Arming { since: frame, planned: false }
                } else {
                    let settle = if scenario { 0 } else { SETTLE };
                    Phase::Fighting { advance_at: frame + settle, first_damage: None, last_damage: frame + settle }
                };
                None
            } else if frame - since > SPAWN_ALLOWANCE {
                Some(conclude(duel, frame, frame, None, "spawn_failed"))
            } else {
                None
            }
        }
        Phase::Arming { since, planned } => {
            let since = *since;
            if frame - since >= SETTLE {
                if !*planned {
                    *planned = true;
                    plan_hurters(duel);
                }
                let army = &mut duel.armies[side];
                if !army.hurters_given
                    && let Some(hurter) = rules.hurter
                {
                    army.hurters_given = true;
                    commands.extend(army.hurters.iter().map(|h| Command::GiveUnit { def: hurter, at: h.at }));
                }
                let armed = duel.armies.iter().all(|a| a.hurters_given && a.hurters.iter().all(|h| h.unit.is_some()));
                if armed || frame - since > SETTLE + SPAWN_ALLOWANCE {
                    for hurter in duel.armies.iter_mut().flat_map(|a| a.hurters.iter_mut()).filter(|h| h.unit.is_none()) {
                        hurter.gone = true;
                    }
                    duel.phase = Phase::Hurting { since: frame, dismissed: None };
                }
            }
            None
        }
        Phase::Hurting { since, dismissed } => {
            let since = *since;
            // Everyone is hurt, or the time is up: every hurter walks off at once, so that the units do not heal (a
            // unit left alone heals slowly: hurt first and left for a minute and a half, a Stout was whole again).
            if dismissed.is_none() {
                let ready = (0..2).all(|s| {
                    let (own, other) = (&duel.armies[s], &duel.armies[1 - s]);
                    own.hurters.iter().all(|h| {
                        let target = other.unit_of.get(&h.target);
                        let health = target.and_then(|t| other.alive.get(t));
                        let wanted = (target.zip(other.spawns[h.target].plan.as_ref()))
                            .map_or(1.0, |(t, plan)| plan.wanted_health(other.max_health.get(t).copied().unwrap_or(1.0)));
                        h.gone || health.is_none_or(|&h| h <= wanted)
                    })
                });
                if ready || frame - since >= HURT_ALLOWANCE {
                    *dismissed = Some(frame);
                }
            }
            let dismissed = *dismissed;
            let [first, second] = &mut duel.armies;
            let (army, enemy) = if side == 0 { (first, &*second) } else { (second, &*first) };
            for hurter in army.hurters.iter_mut().filter(|h| !h.gone) {
                let Some(unit) = hurter.unit else { continue };
                let target = enemy.unit_of.get(&hurter.target).copied();
                let health = target.and_then(|t| enemy.alive.get(&t).copied());
                match dismissed {
                    None => {
                        let wanted = (target.zip(enemy.spawns[hurter.target].plan.as_ref()))
                            .map_or(1.0, |(t, plan)| plan.wanted_health(enemy.max_health.get(&t).copied().unwrap_or(1.0)));
                        match (target, health) {
                            // Shoot while it is above its health (and again if it heals past it), hold while not.
                            (Some(t), Some(h)) if h > wanted => {
                                if !hurter.attacking {
                                    hurter.attacking = true;
                                    hurter.last_progress = frame;
                                    commands.push(Command::Attack { unit, target: t, queue: false });
                                }
                                if h < hurter.last_health - 0.001 {
                                    (hurter.last_health, hurter.last_progress) = (h, frame);
                                } else if frame - hurter.last_progress >= HURTER_PATIENCE
                                    && let (Some(from), Some(to)) = (standing.get(&unit), enemy.seen_at.get(&t))
                                {
                                    hurter.last_progress = frame;
                                    let len = from.dist2d(*to).max(1.0);
                                    let k = HURTER_CLOSE / len;
                                    let near = Vec3 { x: to.x + (from.x - to.x) * k, y: to.y, z: to.z + (from.z - to.z) * k };
                                    commands.push(Command::Move { unit, to: near, queue: false });
                                    commands.push(Command::Attack { unit, target: t, queue: true });
                                }
                            }
                            _ => {
                                if hurter.attacking {
                                    hurter.attacking = false;
                                    commands.push(Command::Stop { unit });
                                }
                            }
                        }
                    }
                    Some(_) => {
                        if hurter.attacking {
                            hurter.attacking = false;
                            commands.push(Command::Stop { unit });
                        }
                        // In place: its blast (22 across, 56 damage) reaches nobody, and walking off gave the units
                        // ten more seconds to heal.
                        if hurter.destruct_at.is_none_or(|at| frame - at >= DESTRUCT_RETRY) {
                            hurter.destruct_at = Some(frame);
                            commands.push(Command::SelfDestruct { unit });
                        }
                    }
                }
            }
            let all_gone = duel.armies.iter().all(|a| a.hurters.iter().all(|h| h.gone));
            if all_gone || frame - since > PREPARE_ALLOWANCE {
                duel.phase = Phase::Spawning { since: frame, round: 2 };
            }
            None
        }
        Phase::Fighting { advance_at, first_damage, last_damage } => {
            if hurt {
                first_damage.get_or_insert(frame);
                *last_damage = frame;
            }
            let (advance_at, first_damage, last_damage) = (*advance_at, *first_damage, *last_damage);
            let [first, second] = &mut duel.armies;
            let (army, enemy) = if side == 0 { (first, &*second) } else { (second, &*first) };
            if first_damage.is_some() && army.spread_at_contact.is_none() {
                army.spread_at_contact = Some(army.dispersion);
            }
            let from = commands.len();
            if frame >= advance_at {
                if army.ordered_at.is_none() {
                    army.ordered_at = Some(frame);
                    army.start = army.alive.clone();
                    if scenario {
                        first_orders(army, enemy, enemy_centroid, commands);
                    }
                }
                let settled = army.ordered_at.is_some_and(|at| !scenario || frame - at >= FPS);
                if settled {
                    send_idle(army, tick, enemy_centroid, commands);
                }
            }
            // The lane over this army's orders: what the director sent this tick is the standing order.
            if let Some(mut lane) = army.lane.take() {
                let mine: Vec<Command> = commands[from..].to_vec();
                let members = army.units.clone();
                lane.note_standing_orders(&mine, frame, |u| members.contains(&u));
                lane.set_commitments(members.iter().map(|u| (*u, Commitment::All)).collect(), HashMap::new());
                let view = DuelView { rules, team, side, own: &members, enemy: &enemy.units, mode: army.lane_mode };
                let output = lane.tick(&view, tick, std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some());
                commands.extend(output.commands);
                army.lane = Some(lane);
            }
            // Judge once both teams have reported this frame.
            if duel.armies.iter().any(|a| a.reported != frame) {
                None
            } else {
                if duel.next_sample <= advance_at {
                    duel.next_sample = advance_at + FPS;
                } else if frame >= duel.next_sample {
                    duel.next_sample += FPS;
                    sample_fire(&mut duel.armies, frame);
                }
                if let Some(t0) = first_damage
                    && duel.shape_due.is_empty()
                    && duel.armies.iter().all(|a| a.shape.samples == 0)
                {
                    duel.shape_due = vec![t0, t0 + SHAPE_LATER];
                }
                if duel.shape_due.first().is_some_and(|due| frame >= *due) {
                    duel.shape_due.remove(0);
                    sample_shape(&mut duel.armies);
                }
                let reason = if duel.armies.iter().any(|a| a.alive.is_empty()) {
                    Some("wiped")
                } else if frame - advance_at >= rules.time_limit * FPS {
                    Some("timeout")
                } else if frame - last_damage >= STALEMATE {
                    Some("stalemate")
                } else {
                    None
                };
                reason.map(|reason| conclude(duel, advance_at, frame, first_damage, reason))
            }
        }
        Phase::Clearing { sweep: stage, .. } => {
            let army = &mut duel.armies[side];
            // Not again while the countdown may be running: a second self-destruct order cancels the first.
            if army.destruct_ordered.is_none_or(|at| frame - at >= DESTRUCT_RETRY) {
                army.destruct_ordered = Some(frame);
                commands.extend(army.alive.keys().map(|&unit| Command::SelfDestruct { unit }));
            }
            // The bombs belong to the first army's team, so only its ticks move the sweep along.
            let sweeper = side == 0;
            match stage {
                Sweep::Survivors => {
                    if duel.armies.iter().all(|a| a.destruct_ordered.is_some() && a.alive.is_empty() && a.reported == frame) {
                        *stage = match (rules.sweep, duel.wreckage) {
                            (Some((_, waves @ 1..)), Some(_)) => Sweep::Bombs { waves_left: waves, spawned_at: None, set_off: HashMap::new() },
                            _ => Sweep::Rest { since: frame },
                        };
                    }
                }
                Sweep::Bombs { waves_left, spawned_at, set_off } if sweeper => {
                    let (Some((bomb, _)), Some([x0, z0, x1, z1])) = (rules.sweep, duel.wreckage) else { unreachable!("checked on entry") };
                    let bombs = tick.snapshot.own_units.iter().filter(|u| {
                        u.def == bomb && u.pos.x > x0 - CLAIM_MARGIN && u.pos.x < x1 + CLAIM_MARGIN && u.pos.z > z0 - CLAIM_MARGIN && u.pos.z < z1 + CLAIM_MARGIN
                    });
                    match *spawned_at {
                        None => {
                            *spawned_at = Some(frame);
                            let along = |low: f32, high: f32| {
                                let steps = ((high - low) / SWEEP_SPACING).ceil().max(1.0) as u32;
                                (0..=steps).map(move |i| low + (high - low) * i as f32 / steps as f32)
                            };
                            for x in along(x0, x1) {
                                commands.extend(along(z0, z1).map(|z| Command::GiveUnit { def: bomb, at: Vec3 { x, y: rules.centre.y, z } }));
                            }
                        }
                        Some(spawned) => {
                            // Bombs go off at once, so one still standing after the retry time had its order cancelled.
                            let mut standing = 0;
                            for bomb in bombs {
                                standing += 1;
                                if set_off.get(&bomb.id).is_none_or(|at| frame - at >= DESTRUCT_RETRY) {
                                    set_off.insert(bomb.id, frame);
                                    commands.push(Command::SelfDestruct { unit: bomb.id });
                                }
                            }
                            let all_gone = standing == 0 && frame - spawned >= FPS;
                            if all_gone {
                                *waves_left -= 1;
                                *spawned_at = None;
                                if *waves_left == 0 {
                                    *stage = Sweep::Rest { since: frame };
                                }
                            }
                        }
                    }
                }
                Sweep::Bombs { .. } => {}
                Sweep::Rest { since } => {
                    if frame - *since >= REST {
                        *stage = Sweep::Done;
                    }
                }
                Sweep::Done => {}
            }
            None
        }
    };
    for at in wrecks {
        duel.wreck_at(at);
    }
    result
}

/// A scenario's first orders: fire at will, then each unit's heading step and its order from the file, or a fight at
/// the other side's centre.
fn first_orders(army: &Army, enemy: &Army, enemy_centroid: Vec3, commands: &mut Vec<Command>) {
    commands.extend(army.held.iter().filter(|u| army.alive.contains_key(u)).map(|&unit| Command::FireState { unit, state: 2 }));
    for (&unit, &i) in &army.spawn_of {
        let spawn = &army.spawns[i];
        if !spawn.mobile || !army.alive.contains_key(&unit) {
            continue;
        }
        let plan = spawn.plan.as_ref();
        let mut queue = false;
        if let (Some([hx, hz]), Some(at)) = (plan.and_then(|p| p.heading), army.seen_at.get(&unit)) {
            commands.push(Command::Move { unit, to: Vec3 { x: at.x + hx * HEADING_STEP, y: at.y, z: at.z + hz * HEADING_STEP }, queue: false });
            queue = true;
        }
        let order = match plan.and_then(|p| p.order) {
            Some(Order::Move { x, z }) => Command::Move { unit, to: Vec3 { x, y: 0.0, z }, queue },
            Some(Order::Fight { x, z }) => Command::Fight { unit, to: Vec3 { x, y: 0.0, z }, queue },
            Some(Order::Attack { target }) => match enemy.unit_of.get(&target).filter(|t| enemy.alive.contains_key(t)) {
                Some(&t) => Command::Attack { unit, target: t, queue },
                None => Command::Fight { unit, to: enemy.spawns[target].at, queue },
            },
            None => Command::Fight { unit, to: enemy_centroid, queue },
        };
        commands.push(order);
    }
}

/// Attack-move at where the enemy is now for every idle unit (arrived, or never ordered), in the army's formation.
/// A unit the lane holds is left to the lane: one standing at contact, or held out of a threat, is idle by design
/// (a re-send throttle tried first cost the lane's arms 0.1 of margin by itself: form-stout-old-nolane).
fn send_idle(army: &mut Army, tick: &Tick, enemy_centroid: Vec3, commands: &mut Vec<Command>) {
    let mobile = |u: &UnitId| army.spawn_of.get(u).is_some_and(|&i| army.spawns[i].mobile);
    let held = |u: UnitId| army.lane.as_ref().is_some_and(|l| l.holding(u).is_some());
    let idle: Vec<UnitId> =
        tick.snapshot.own_units.iter().filter(|u| u.idle && army.units.contains(&u.id) && mobile(&u.id) && !held(u.id)).map(|u| u.id).collect();
    if idle.is_empty() {
        return;
    }
    let line = army.formation.filter(|f| matches!(f.kind, FormationKind::Line));
    if let Some(formation) = line {
        // A line is a property of the whole army, so the places are laid out over everyone alive and the idle ones
        // are sent to their own place.
        let mut alive: Vec<(UnitId, Vec3)> =
            (tick.snapshot.own_units.iter().filter(|u| army.units.contains(&u.id))).map(|u| (u.id, u.pos)).collect();
        let places = line_abreast(&mut alive, army.centroid, enemy_centroid, formation.spacing);
        let wanted = places.into_iter().filter(|(id, _)| idle.contains(id));
        commands.extend(wanted.map(|(unit, to)| Command::Fight { unit, to, queue: false }));
    } else {
        commands.extend(idle.into_iter().map(|unit| Command::Fight { unit, to: enemy_centroid, queue: false }));
    }
}

fn conclude(duel: &mut Duel, started: i32, frame: i32, first_damage: Option<i32>, reason: &'static str) -> DuelResult {
    let [x, y] = &duel.armies;
    let survivors = [x.alive.len() as u32, y.alive.len() as u32];
    let winner = match (reason, survivors) {
        ("wiped", [0, 0]) => "draw",
        ("wiped", [_, 0]) => "x",
        ("wiped", [0, _]) => "y",
        _ => "draw",
    };
    let label = |a: &Army| a.formation.map_or_else(|| "scenario".to_string(), |f| f.label());
    // How far each side's units started from the file's health.
    let health_error = |a: &Army| {
        let errors: Vec<f32> = (a.unit_of.iter())
            .filter_map(|(&i, u)| {
                let plan = a.spawns[i].plan.as_ref()?;
                Some((a.start_of(*u) - plan.wanted_health(a.max_health.get(u).copied().unwrap_or(1.0))).abs())
            })
            .collect();
        if errors.is_empty() { 0.0 } else { errors.iter().sum::<f32>() / errors.len() as f32 }
    };
    let result = DuelResult {
        match_index: 0,
        site: 0,
        sequence: duel.sequence,
        job: duel.job.clone(),
        count: [x.spawns.len() as u32, y.spawns.len() as u32],
        metal: [x.metal(), y.metal()],
        winner,
        reason,
        seconds: (frame - started) as f32 / FPS as f32,
        contact_seconds: first_damage.map(|f| (f - started) as f32 / FPS as f32),
        survivors,
        value_left: [x.value_left(), y.value_left()],
        damage_taken: [x.damage_taken, y.damage_taken],
        spread_at_contact: [x.spread_at_contact.unwrap_or(0.0), y.spread_at_contact.unwrap_or(0.0)],
        formation: [label(x), label(y)],
        fire: [Tally { dealt: y.taken_from_enemy, ..x.fire.tally.clone() }, Tally { dealt: x.taken_from_enemy, ..y.fire.tally.clone() }],
        shape: [x.shape.clone(), y.shape.clone()],
        lane: [x.lane_mode.label().to_string(), y.lane_mode.label().to_string()],
        health_error: [health_error(x), health_error(y)],
    };
    duel.phase = Phase::Clearing { since: frame, sweep: Sweep::Survivors };
    result
}

/// The fire instrument's once-a-second sample over both armies, from where each unit was last reported.
fn sample_fire(armies: &mut [Army; 2], frame: i32) {
    let (x, y) = (fire_places(&armies[0], false), fire_places(&armies[1], false));
    let positions = |list: &[(UnitId, Vec3, f32, f32)]| list.iter().map(|u| u.1).collect::<Vec<_>>();
    let (at_x, at_y) = (positions(&x), positions(&y));
    let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
    let [first, second] = armies;
    for (side, army, own, theirs) in [(0, first, &x, &at_y), (1, second, &y, &at_x)] {
        let muzzled = army.fire.sample(own, theirs);
        if debug {
            for (unit, cause, range) in muzzled {
                let held = army.lane.as_ref().and_then(|l| l.holding(unit)).unwrap_or_else(|| "-".to_string());
                let idle = army.alive.contains_key(&unit);
                eprintln!("[duel side {side}] f={frame} muzzled: unit {} {cause:?} at range {range:.0}, held by {held}{}", unit.0, if idle { "" } else { " (dead)" });
            }
        }
    }
}

/// The shape instrument's sample over both armies: nearest-friend spacing and the friend-on-the-line share, from
/// where each unit was last reported (`run/replays/shapes.py`'s definitions, soldiers only).
fn sample_shape(armies: &mut [Army; 2]) {
    let (x, y) = (fire_places(&armies[0], true), fire_places(&armies[1], true));
    let positions = |list: &[(UnitId, Vec3, f32, f32)]| list.iter().map(|u| u.1).collect::<Vec<_>>();
    let (at_x, at_y) = (positions(&x), positions(&y));
    let [first, second] = armies;
    first.shape.sample(&x, &at_y);
    second.shape.sample(&y, &at_x);
}

/// An army's living units (the mobile ones only, for the shape instrument) with their type's reach and reload,
/// where each was last seen.
fn fire_places(army: &Army, mobile_only: bool) -> Vec<(UnitId, Vec3, f32, f32)> {
    (army.alive.keys())
        .filter_map(|id| {
            let spawn = &army.spawns[army.spawn_of[id]];
            (spawn.mobile || !mobile_only).then(|| army.seen_at.get(id).map(|at| (*id, *at, spawn.reach, spawn.reload))).flatten()
        })
        .collect()
}
