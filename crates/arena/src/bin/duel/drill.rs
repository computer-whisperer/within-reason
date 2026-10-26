//! Drills (`duel --drill walk|reverse|gather`; `docs/design/2026-10-24-formation-drills.md` §3): one army, one order,
//! no enemy, measured. **walk**: the army spawned in its formation at the west end of a strip is given one order (a
//! fight, the hands' advance, or a move, the hands' walk) to a point `distance` east, and the drill ends when every
//! living unit is within `ARRIVED` of it. **reverse**: the walk, and `turn_at` after the first order the same order
//! back to the start, the drill ending when everyone is back within `ARRIVED` of it. **gather**: a lab at the west end
//! builds the force under the resource cheat, and every soldier it makes is adopted into the army as it appears and
//! sent to the rally `distance` east. Nothing of the director's goes out between the order and the end but
//! `reissue`, the same order to every living unit every so many frames (the hands' re-send, as an arm), and the lane
//! runs over the orders as it does in a duel (`--lane`). **circuit** (design D12): a deployed army beside a lab walks
//! a loop of points across the map `laps` times while the lab builds a feed list, each new unit adopted into the army
//! as it appears (and so into the lane's group, a joiner; lane off, given the whole route as queued orders like
//! everyone else); the loop `laps` times over is one route, one order. **assault** (design D16): the walk into a
//! defence that shoots back, `--theirs` a force of armed buildings stood in one rank across the approach on the goal
//! (`DEFENCE_SPACING` apart), given once; the duel's judge decides (the defence wiped: `taken`; the army's mobile units
//! wiped: `wiped`; the time limit) and the duel's own columns (value left, the fire instrument) measure the fight,
//! beside the drill's (`taken_s`, `lost_before_shot`: units dead before their own first shot, the price of arriving
//! in pieces).
//!
//! Measured once a second from the first order (the per-second log carries every position; `run/drill_table.py`
//! reads it): each unit's first second within `ARRIVED` of its goal (the first, median and last arrival go to
//! `duels.csv`), the army's spread (root mean square from its centre) and its median nearest-friend distance at the
//! second the median unit arrived, each mobile unit's speed over the second as a share of its type's (`speed_share`:
//! the mean over units and seconds up to the median arrival, a unit counted until its own arrival; positions a second
//! apart under-read a turning unit a little), the first second the median share reaches `UP_TO_SPEED` after the order (`spin_up_s`: every army spawns
//! facing south and turns first, and every arm shares that turn) and after a reverse's turn (`recover_s`), the units
//! a gather's lab made (`produced`) and the median seconds from a unit's appearance to its arrival (`join_median_s`),
//! and the body's shape against its heading (`Shape`, design D8): its depth and width, and the engagement ramp, each
//! unit's depth behind the leader over its type's speed (the median of those over the army and the largest), logged
//! every second and kept at contact, the second the first unit arrives at the goal in force (`depth_contact`,
//! `width_contact`, `ramp_half_s`, `ramp_all_s`). A circuit counts each deployed unit's passes of the lab's corner
//! (within `AT_THE_LAB` after having been further: the start counts for nothing), its `laps`-th pass its arrival, the
//! median deployed unit's passes at the end (`laps`), the fed units that joined the deployed body (within `JOINED` of
//! its centroid; `joined`, and `join_median_s` from appearance) and under the group the formed share, formed members
//! over members averaged over the seconds (`formed_share`).
//! The director's side (`director.rs` `drill_orders`, `drill_sample`) owns the army; this module owns the geometry,
//! the clock and the score.

use std::collections::HashMap;

use bot_protocol::{Command, UnitDefId, UnitId, Vec3};

pub const FPS: i32 = 30;
/// A unit within this of its goal has arrived (the hands' own arrival number, `pianist/groups.rs` `ARRIVED`).
pub const ARRIVED: f32 = 300.0;
/// A circuit's lab stands this far from its corner toward the loop's inside (design D18): the corner is the route's
/// end, and a body stands off a structure by the room it wants (224 at the spacing of 64) plus the lab's half (48).
pub const LAB_OFFSET: f32 = 320.0;
/// The room the lab's place must have: its half (48) and a hull.
pub const LAB_ROOM: f32 = 112.0;
/// The median unit at this share of its type's speed is up to speed.
pub const UP_TO_SPEED: f32 = 0.9;
/// A soldier appearing within this of the gather's lab is the lab's output, adopted into the army.
pub const AT_THE_LAB: f32 = 400.0;
/// A circuit's deployed army spawns this far from the lab toward the walked route's first point, on the lab's flat
/// square (`sites::circuit`), clear of the lab's yard; it starts inside the corner's `AT_THE_LAB`, which the passes'
/// hysteresis counts for nothing.
pub const CIRCUIT_SPAWN: f32 = 240.0;
/// The director as host (D12): an away or stuck member that has come no nearer its slot for this long is taken out of
/// the group, since the anchor waits for every member of its body (what Jev would decide from the state's words).
pub const STRAGGLER_PATIENCE: i32 = 20 * FPS;
/// A circuit's fed unit within this of the deployed body's centroid has joined it (the group's own `formed_within`
/// is 128 of a slot; this is the arm-neutral measure).
pub const JOINED: f32 = 300.0;
/// The gather's stores are topped up to full whenever they fall under this share (the cheat gives nothing beyond the
/// storage, and a tier-1 list costs more than the start's store holds).
pub const TOP_UP_UNDER: f32 = 0.6;
/// The body's centroid must move this far in a second for its displacement to be the heading (design D10).
pub const HEADING_MOVED: f32 = 10.0;
/// `assault` (D16): the defence's buildings stand this far apart in one rank across the approach, centred on the goal.
pub const DEFENCE_SPACING: f32 = 128.0;
/// D21: the churned structure's sites stand this far past the formation's half-width off the axis; the crosser is
/// sent when the army's centroid is within this of its crossing point along the axis, to this far past the strip's
/// other side.
pub const CHURN_OFF: f32 = 160.0;
pub const CROSSER_TRIGGER: f32 = 500.0;
pub const CROSSER_BEYOND: f32 = 1200.0;
/// D23 `--nudge`: how far across the axis the goal is moved at each re-declaration (over `ROUTE_MOVED` 48).
pub const NUDGE: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Walk,
    Reverse,
    Gather,
    /// The walk into an enemy structure standing on the goal (design D9).
    Into,
    /// A deployed army round a loop of points while a lab feeds it (design D12).
    Circuit,
    /// The walk into a defence that shoots back, judged as a duel (design D16).
    Assault,
}

impl Kind {
    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "walk" => Some(Kind::Walk),
            "reverse" => Some(Kind::Reverse),
            "gather" => Some(Kind::Gather),
            "into" => Some(Kind::Into),
            "circuit" => Some(Kind::Circuit),
            "assault" => Some(Kind::Assault),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Walk => "walk",
            Kind::Reverse => "reverse",
            Kind::Gather => "gather",
            Kind::Into => "into",
            Kind::Circuit => "circuit",
            Kind::Assault => "assault",
        }
    }
}

/// `--drill` and its settings, shared by a batch's duels.
#[derive(Clone, Debug)]
pub struct Options {
    pub kind: Kind,
    /// From the start to the goal, elmos (`--distance`).
    pub distance: f32,
    /// A fight (the hands' advance) or a move (the hands' walk) (`--order`).
    pub fight: bool,
    /// The order again to every living unit every this many frames (`--reissue S`, fractional seconds); `None`: once.
    pub reissue: Option<i32>,
    /// The lead law's kill-test (design D11): with `--lead L` every re-issue sends each unit to the point `L` ahead of
    /// its own position along the axis (the goal itself when that is past it), so its goal is always `L` ahead.
    pub lead: Option<f32>,
    /// The paced slot (D11, the second test): with `--pace P` (elmos a second) every re-issue sends each unit to its
    /// own slot, its spawn place carried along the axis by `lead` plus `P` times the seconds since the first order
    /// (since the turn, for a reverse), stopping at the goal: the formation translated rigidly at the pace, the
    /// engine's own approach to a goal doing the following.
    pub pace: Option<f32>,
    /// `reverse`: frames after the first order at which the order goes back to the start (`--turn-at S`).
    pub turn_at: i32,
    /// `gather`, `circuit`: the plant that builds the force (`--lab`).
    pub lab: String,
    /// `circuit`: the list the lab builds while the army walks (`--feed name*count+...`; empty: no lab), the laps
    /// (`--laps`), and the loop's points in order, the lab's first (`--circuit X,Z;X,Z;...`; `None`: the map's
    /// inset corners, `sites::circuit`).
    pub feed: Vec<(String, u32)>,
    pub laps: u32,
    pub circuit: Option<Vec<(f32, f32)>>,
    /// Terrain cases (design D10): chosen ends on any map, or a detour of at least this ratio from the chooser.
    pub from: Option<(f32, f32)>,
    pub to: Option<(f32, f32)>,
    pub around: Option<f32>,
    /// `into`: the structure on the goal (`--target`; the gantry by default: 16,000 health, no weapon, a death blast of
    /// 1,330 over 420 where a fusion's is 10,600 over 1,280, and units fire at it where they ignore a wall).
    pub target: String,
    /// `assault`: `--theirs` on the march (design D18): side `y`'s mobile units get a Fight order toward the army's
    /// start when the army's first orders go out (and at the re-issue cadence), under `--lane x/y`'s second mode.
    pub advance: bool,
    /// D21, the live faults as arms (`docs/design/2026-10-24-formation-drills.md`): a rear party `--behind FORCE@D`
    /// spawned `D` behind the start, members from the first order; `--scatter D` pushes every spawn place back along
    /// the axis by its share of `D`; `--churn S` gives side `x` a structure (`churn_def`) every `S` frames beside the
    /// start, the one standing self-destructed first; `--stall T,S` holds the rearmost formed member out of the group
    /// for `S` frames from `T` frames after the first order; `--crosser NAME@D,W` is side `y`'s one unit `D` along the
    /// axis and `W` off it, sent across the strip when the army comes.
    pub behind: Vec<(String, u32)>,
    pub behind_distance: f32,
    pub scatter: f32,
    pub churn: Option<i32>,
    pub churn_def: String,
    pub stall: Option<(i32, i32)>,
    pub crosser: Option<(String, f32, f32)>,
    /// D23: the walk's goal re-declared every this many frames moved `NUDGE` across the axis, alternating sides
    /// (`--nudge S`): the hands' follow steps and party moves, a changed order to the lane.
    pub nudge: Option<i32>,
    /// D23: the rear party (`--behind`) stands out of the group, unordered, until this many frames after the first
    /// order, then joins it (`--late S`): the lab's fresh units declared into a group already marching.
    pub late: Option<i32>,
    /// The companies design (`--cap N`): members a company of the army holds at most; 0 for one formation.
    pub cap: usize,
}

/// D21's churned structure: its def, its two sites, the one standing, the next site, the frame of the last churn
/// and of a pending give, and the cadence.
pub struct Churn {
    pub def: UnitDefId,
    pub sites: [Vec3; 2],
    pub standing: Option<UnitId>,
    pub next: usize,
    pub last: Option<i32>,
    pub given: Option<i32>,
    pub every: i32,
}

/// D21's crosser: its spawn index in army `y`, where it crosses (along the axis from the start, off it), and whether
/// it has been sent.
pub struct Crosser {
    pub index: usize,
    pub along: f32,
    pub across: f32,
    pub sent: bool,
}

/// What a drill measured, for `duels.csv` (`report.rs`); empty outside a drill.
#[derive(Clone, Debug, Default)]
pub struct Score {
    pub kind: String,
    /// Seconds from the first order to the first, the median and the last arrival.
    pub arrive: [Option<f32>; 3],
    pub spread_arrival: Option<f32>,
    pub nn_arrival: Option<f32>,
    pub speed_share: Option<f32>,
    pub spin_up_s: Option<f32>,
    pub recover_s: Option<f32>,
    pub produced: u32,
    pub join_median_s: Option<f32>,
    /// The body's shape at contact (design D8).
    pub depth_contact: Option<f32>,
    pub width_contact: Option<f32>,
    pub ramp_half_s: Option<f32>,
    pub ramp_all_s: Option<f32>,
    /// `into` (design D9): the first shot from the first order, the shot ramp from it to the median unit's first
    /// shot and the last's, and the target's deaths.
    pub first_shot_s: Option<f32>,
    pub shot_ramp_half_s: Option<f32>,
    pub shot_ramp_all_s: Option<f32>,
    pub target_deaths: u32,
    /// `assault` (D16): seconds from the first order to the defence's last building dying, and the army's units dead
    /// before their own first shot.
    pub taken_s: Option<f32>,
    pub lost_before_shot: u32,
    /// Terrain (design D10): the crow's distance, the walk on foot, and the walk over the slowest type's speed.
    pub straight: f32,
    pub walk: Option<f32>,
    pub route_s: Option<f32>,
    /// `circuit` (D12): the median deployed unit's passes of the lab's corner, the formed share under the group, and
    /// the fed units that joined the deployed body.
    pub laps: u32,
    pub formed_share: Option<f32>,
    pub joined: u32,
    /// Members the host took out of the group as stragglers (`STRAGGLER_PATIENCE`).
    pub left_behind: u32,
    /// The companies design (`--cap`): the most companies the army had at once, and the mean over the ticks of the
    /// marching companies' own formed shares (a muster is not marching).
    pub companies: u32,
    pub company_formed: Option<f32>,
    /// D21: member-elmos walked away from the goal along the axis over those walked toward it, the elmos the anchor
    /// moved against its heading summed over ticks, and the seconds the group's pace was 0 while marching.
    pub backtrack_share: Option<f32>,
    pub anchor_back: f32,
    pub stopped_s: u32,
}

/// One living mobile unit as the sampler sees it: its spawn index, id, place and its type's speed (elmos a second).
pub struct Walker {
    pub index: usize,
    pub id: UnitId,
    pub at: Vec3,
    pub speed: f32,
}

/// The body's shape against its heading one second (design D8): the leading unit's projection on the heading less
/// the rearmost's, the extent across it, and the engagement ramp, each unit's depth behind the leader over its type's
/// speed, as the median over the army and the largest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub depth: f32,
    pub width: f32,
    pub ramp_half_s: f32,
    pub ramp_all_s: f32,
}

impl Shape {
    /// Of these units along `heading` (any non-zero vector); `None` with nobody.
    pub fn of(walkers: &[Walker], heading: Vec3) -> Option<Shape> {
        let norm = (heading.x * heading.x + heading.z * heading.z).sqrt();
        if walkers.is_empty() || norm <= 0.0 {
            return None;
        }
        let (hx, hz) = (heading.x / norm, heading.z / norm);
        let along: Vec<f32> = walkers.iter().map(|w| w.at.x * hx + w.at.z * hz).collect();
        let across: Vec<f32> = walkers.iter().map(|w| w.at.z * hx - w.at.x * hz).collect();
        let lead = along.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let depth = lead - along.iter().copied().fold(f32::INFINITY, f32::min);
        let width = across.iter().copied().fold(f32::NEG_INFINITY, f32::max) - across.iter().copied().fold(f32::INFINITY, f32::min);
        let mut ramps: Vec<f32> = walkers.iter().zip(&along).filter(|(w, _)| w.speed > 0.0).map(|(w, p)| (lead - p) / w.speed).collect();
        if ramps.is_empty() {
            return None;
        }
        ramps.sort_by(f32::total_cmp);
        Some(Shape { depth, width, ramp_half_s: ramps[ramps.len() / 2], ramp_all_s: ramps[ramps.len() - 1] })
    }
}

/// One drill's state and measurements, per duel.
pub struct Drill {
    pub kind: Kind,
    pub fight: bool,
    pub reissue: Option<i32>,
    pub lead: Option<f32>,
    pub pace: Option<f32>,
    pub turn_at: i32,
    pub start: Vec3,
    pub goal: Vec3,
    /// The goal in force: `start` after a reverse's turn.
    pub toward: Vec3,
    pub turned: Option<i32>,
    /// The frame the last order went out to the whole army, and the first.
    pub last_order: Option<i32>,
    pub first_order: Option<i32>,
    /// Per spawn index: the frame first within `ARRIVED` of the goal in force (cleared at a reverse's turn).
    pub arrived: HashMap<usize, i32>,
    /// `gather`: the lab's spawn index, what it builds, whether the list has been queued, and the frame each adopted
    /// unit appeared.
    pub lab: Option<usize>,
    pub wanted: Vec<(UnitDefId, u32)>,
    pub queued: bool,
    pub appeared: HashMap<usize, i32>,
    /// Under the lane's group a gather is a hold at the rally (`rally`) until the whole list stands there in
    /// formation, then the order to the goal: the frame the hold ended.
    pub gathered_at: Option<i32>,
    /// `circuit`: the route (the spawn point then the corners, the loop `laps` times over, the lab's corner last;
    /// the group's order and, lane off, the queued orders alike: the way between the corners is the lane's, design
    /// D17), the laps, the deployed units whose
    /// passes count (`core`: the initial mobile spawns, or the first fed unit when there were none), each one's
    /// (within the corner last second, passes), the frame each fed unit joined the deployed body, the formed share
    /// each second, and the lane's group as last noted: (formed, members, arrived with no exceptions).
    pub route: Vec<Vec3>,
    pub laps: u32,
    pub core: Vec<usize>,
    passes: HashMap<usize, (bool, u32)>,
    pub joined: HashMap<usize, i32>,
    formed_shares: Vec<f32>,
    company_shares: Vec<f32>,
    companies_max: usize,
    group: Option<(usize, usize, bool)>,
    /// Members the director took out of the group as stragglers, by spawn index.
    pub left_behind: Vec<usize>,
    /// Where each unit stood at the last once-a-second sample.
    last: HashMap<UnitId, Vec3>,
    /// The median speed share each second, from the first order.
    pub shares: Vec<(i32, f32)>,
    /// Every unit's speed share in the seconds up to the median arrival, each unit until its own arrival.
    shares_before_median: Vec<f32>,
    pub spin_up: Option<i32>,
    pub recover: Option<i32>,
    /// The frame the median unit arrived, and the army's spread and median nearest-friend distance then.
    pub median_arrival: Option<i32>,
    pub spread_at_median: Option<(f32, f32)>,
    /// `gather`: the team's stores at the last tick, [metal current, income, usage, storage, energy current, income,
    /// usage, storage] (the per-second log's `stores`; a starved lab shows here).
    pub stores: Option<[f32; 8]>,
    /// The body's shape this second (the per-second log's `shape`), and at contact: the second the first unit
    /// arrived at the goal in force (cleared at a reverse's turn, like the arrivals).
    pub shape: Option<Shape>,
    pub shape_at_contact: Option<Shape>,
    /// `into`: the target's spawn index in army `y`, whether it stood at the last tick, the frame of a pending give
    /// after a death, its deaths, and per spawn index of `x` the frame of the unit's first shot.
    pub target: Option<usize>,
    pub target_stood: bool,
    pub target_given: Option<i32>,
    pub target_deaths: u32,
    pub first_shot: HashMap<usize, i32>,
    /// `assault`: the frame the defence fell, and the army's units dead before their own first shot (the director
    /// counts them at the end).
    pub taken: Option<i32>,
    pub lost_before_shot: u32,
    /// `assault --advance` (D18): theirs march on the army's start; the frame their order last went out.
    pub advance: bool,
    pub advance_order: Option<i32>,
    /// The crow's distance from the start to the goal, the walk on foot over standable ground (the terrain crate's
    /// field; `None` when the goal is not reachable), and the slowest type's speed: `route_s` is the walk over it.
    pub straight: f32,
    pub walk: Option<f32>,
    pub slowest: f32,
    /// The body's centroid at the last sample, for the heading (its displacement over the second when 10 or more).
    last_centroid: Option<Vec3>,
    /// D21's arms: the churned structure, the stall (frames from the first order, for how long), the member held
    /// (id, until which frame) and whether the stall has been done, and the crosser.
    pub churn: Option<Churn>,
    pub stall: Option<(i32, i32)>,
    pub stalled: Option<(UnitId, i32)>,
    pub stall_done: bool,
    pub crosser: Option<Crosser>,
    /// D23 `--nudge`: frames between the goal's re-declarations, or `None`.
    pub nudge: Option<i32>,
    /// D23 `--late`: frames after the first order until the rear party joins, and the rear party's spawn indices.
    pub late: Option<i32>,
    pub cap: usize,
    pub rear: Vec<usize>,
    /// D21's measures: member-elmos along the axis toward the goal and away from it (this second's away in
    /// `back_second`); the anchor's steps against its heading, from the anchor and heading last noted; the group's
    /// pace, arrived and engaged as last noted and the seconds the pace was 0 on the march; the orders to the army
    /// this second with the ticks that had none, and the last whole second's.
    pub forward: f32,
    pub back: f32,
    pub back_second: f32,
    pub anchor_back: f32,
    last_anchor: Option<(Vec3, (f32, f32))>,
    group_pace: Option<(f32, bool, bool)>,
    pub stopped_s: u32,
    orders_second: (u32, u32),
    pub orders_last_second: (u32, u32),
}

impl Drill {
    /// The strip's ends from the site's centre: the start `distance / 2` west of it, the goal as far east.
    pub fn new(options: &Options, centre: Vec3, wanted: Vec<(UnitDefId, u32)>) -> Drill {
        // The strip holds the walk and a rear party's `behind_distance` (D21): the goal at the east end of that span.
        let span = options.distance + options.behind_distance;
        let goal = Vec3 { x: centre.x + span / 2.0, ..centre };
        let start = Vec3 { x: goal.x - options.distance, ..centre };
        Drill {
            kind: options.kind,
            fight: options.fight,
            reissue: options.reissue,
            lead: options.lead,
            pace: options.pace,
            turn_at: options.turn_at,
            start,
            goal,
            toward: goal,
            turned: None,
            last_order: None,
            first_order: None,
            arrived: HashMap::new(),
            lab: None,
            wanted,
            queued: false,
            appeared: HashMap::new(),
            gathered_at: None,
            route: Vec::new(),
            laps: options.laps,
            core: Vec::new(),
            passes: HashMap::new(),
            joined: HashMap::new(),
            formed_shares: Vec::new(),
            company_shares: Vec::new(),
            companies_max: 0,
            group: None,
            left_behind: Vec::new(),
            last: HashMap::new(),
            shares: Vec::new(),
            shares_before_median: Vec::new(),
            spin_up: None,
            recover: None,
            median_arrival: None,
            spread_at_median: None,
            stores: None,
            shape: None,
            shape_at_contact: None,
            target: None,
            target_stood: false,
            target_given: None,
            target_deaths: 0,
            first_shot: HashMap::new(),
            taken: None,
            lost_before_shot: 0,
            advance: options.advance,
            advance_order: None,
            straight: options.distance,
            walk: None,
            slowest: 0.0,
            last_centroid: None,
            churn: None,
            stall: options.stall,
            stalled: None,
            stall_done: false,
            crosser: None,
            nudge: options.nudge,
            late: options.late,
            cap: options.cap,
            rear: Vec::new(),
            forward: 0.0,
            back: 0.0,
            back_second: 0.0,
            anchor_back: 0.0,
            last_anchor: None,
            group_pace: None,
            stopped_s: 0,
            orders_second: (0, 0),
            orders_last_second: (0, 0),
        }
    }

    /// The drill's order to one unit standing at `at`, spawned at `spawned`: the goal in force; with `pace` its slot
    /// this frame; with `lead` alone the point that far ahead of the unit along the axis (D11); a fight or a move.
    pub fn order(&self, unit: UnitId, at: Vec3, spawned: Vec3, frame: i32) -> Command {
        let to = match (self.pace, self.lead) {
            (Some(pace), lead) => self.slot(spawned, lead.unwrap_or(0.0), pace, frame),
            (None, Some(lead)) => self.lead_point(at, lead),
            (None, None) => self.toward,
        };
        if self.fight { Command::Fight { unit, to, queue: false } } else { Command::Move { unit, to, queue: false } }
    }

    /// The drill's orders to one unit: a circuit's corners as queued orders (the first not queued: the hands' way of
    /// walking a route today, and the lane-off baseline; the group gets the same corners as one order), else the one
    /// `order`.
    pub fn orders(&self, unit: UnitId, at: Vec3, spawned: Vec3, frame: i32) -> Vec<Command> {
        if self.kind == Kind::Circuit {
            let fight = self.fight;
            return (self.route.iter().enumerate())
                .map(|(i, &to)| if fight { Command::Fight { unit, to, queue: i > 0 } } else { Command::Move { unit, to, queue: i > 0 } })
                .collect();
        }
        vec![self.order(unit, at, spawned, frame)]
    }

    /// The lane's group as the director read it this tick: how many are formed of how many members, and whether it
    /// stands at the route's end with no exceptions (a circuit ends on that word; `sample` takes the share).
    /// The army's companies this tick (the companies design): how many, and their own formed shares.
    pub fn note_companies(&mut self, companies: &[micro::CompanyState]) {
        self.companies_max = self.companies_max.max(companies.len());
        let marching: Vec<f32> = companies.iter().filter(|c| c.role != micro::Role::Mustering && c.members > 0).map(|c| c.formed as f32 / c.members as f32).collect();
        if !marching.is_empty() {
            self.company_shares.push(marching.iter().sum::<f32>() / marching.len() as f32);
        }
    }

    pub fn note_group(&mut self, formed: usize, members: usize, done: bool) {
        self.group = Some((formed, members, done));
    }

    /// D21: the group's anchor, heading, pace and standing this tick. A step of the anchor against its heading is
    /// summed in `anchor_back` (a snap back onto the body shows here, a march never does); the pace and standing are
    /// kept for the second's `stopped_s`.
    pub fn note_anchor(&mut self, anchor: Vec3, heading: (f32, f32), pace: f32, arrived: bool, engaged: bool) {
        if let Some((last, h)) = self.last_anchor {
            let step = (anchor.x - last.x) * h.0 + (anchor.z - last.z) * h.1;
            if step < 0.0 {
                self.anchor_back -= step;
            }
        }
        self.last_anchor = Some((anchor, heading));
        self.group_pace = Some((pace, arrived, engaged));
    }

    /// D21: how many commands went to the army this tick (the per-second line's `orders`: the count and the ticks
    /// with none, the burst-and-silence shape).
    pub fn note_orders(&mut self, count: usize) {
        self.orders_second.0 += count as u32;
        if count == 0 {
            self.orders_second.1 += 1;
        }
    }

    /// The paced slot of a unit spawned at `spawned`: its place in the formation carried from the end behind along the
    /// axis by `lead` plus `pace` times the seconds since the order in force began, and no further than the goal.
    fn slot(&self, spawned: Vec3, lead: f32, pace: f32, frame: i32) -> Vec3 {
        let behind = if self.toward == self.goal { self.start } else { self.goal };
        let since = self.turned.or(self.first_order).map_or(0, |f| frame - f).max(0) as f32 / FPS as f32;
        let axis = self.axis();
        let length = (axis.x * axis.x + axis.z * axis.z).sqrt();
        if length <= 0.0 {
            return self.toward;
        }
        let (dx, dz) = (axis.x / length, axis.z / length);
        let along = (lead + pace * since).min(length);
        let origin = Vec3 { x: spawned.x + behind.x - self.start.x, y: self.toward.y, z: spawned.z + behind.z - self.start.z };
        Vec3 { x: origin.x + dx * along, y: self.toward.y, z: origin.z + dz * along }
    }

    /// The point `lead` ahead of `at` along the axis to the goal in force, or the goal itself when `at` is within
    /// `lead` of it along the axis.
    fn lead_point(&self, at: Vec3, lead: f32) -> Vec3 {
        let axis = self.axis();
        let norm = (axis.x * axis.x + axis.z * axis.z).sqrt();
        if norm <= 0.0 {
            return self.toward;
        }
        let (dx, dz) = (axis.x / norm, axis.z / norm);
        let to_go = (self.toward.x - at.x) * dx + (self.toward.z - at.z) * dz;
        if to_go <= lead {
            return self.toward;
        }
        Vec3 { x: at.x + dx * lead, y: self.toward.y, z: at.z + dz * lead }
    }

    /// A gather's rally under the lane's group: `AT_THE_LAB` along the axis from the start, clear of the lab's yard.
    pub fn rally(&self) -> Vec3 {
        let axis = self.axis();
        let length = (axis.x * axis.x + axis.z * axis.z).sqrt().max(1.0);
        Vec3 { x: self.start.x + axis.x / length * AT_THE_LAB, y: self.start.y, z: self.start.z + axis.z / length * AT_THE_LAB }
    }

    /// Whether the reverse's turn is due: `turn_at` after the first order, once.
    pub fn turn_due(&self, frame: i32) -> bool {
        self.kind == Kind::Reverse && self.turned.is_none() && self.last_order.is_some_and(|f| frame - f >= self.turn_at)
    }

    /// The turn: the goal in force becomes the start, the arrivals start afresh.
    pub fn turn(&mut self, frame: i32) {
        self.shape_at_contact = None;
        self.toward = self.start;
        self.turned = Some(frame);
        self.arrived.clear();
    }

    /// Whether the order goes out again this frame (`reissue`).
    pub fn reissue_due(&self, frame: i32) -> bool {
        match (self.reissue, self.last_order) {
            (Some(every), Some(last)) => frame - last >= every,
            _ => false,
        }
    }

    /// How many units the drill waits for: a gather's whole list, else the units walking.
    pub fn expected(&self, walking: usize) -> usize {
        match self.kind {
            Kind::Gather => self.wanted.iter().map(|(_, n)| *n as usize).sum(),
            Kind::Circuit => self.core.len(),
            _ => walking,
        }
    }

    /// The drill's axis: from the end behind to the goal in force.
    /// D23 `--nudge`: the goal as declared at `frame`, moved `NUDGE` across the axis to one side or the other every
    /// `nudge` frames since the first order (the goal itself without the arm).
    pub fn nudged_goal(&self, frame: i32) -> Vec3 {
        let (Some(every), Some(first)) = (self.nudge, self.first_order) else { return self.toward };
        let step = (frame - first).max(0) / every.max(1);
        let side = if step % 2 == 0 { 1.0 } else { -1.0 };
        let axis = self.axis();
        let norm = axis.x.hypot(axis.z).max(1.0);
        Vec3 { x: self.toward.x - axis.z / norm * NUDGE * side, y: self.toward.y, z: self.toward.z + axis.x / norm * NUDGE * side }
    }

    /// D23 `--late`: whether the rear party still stands out of the group at `frame`.
    pub fn rear_waits(&self, frame: i32) -> bool {
        match (self.late, self.first_order) {
            (Some(late), Some(first)) => frame < first + late,
            (Some(_), None) => true,
            _ => false,
        }
    }

    pub fn axis(&self) -> Vec3 {
        let behind = if self.toward == self.goal { self.start } else { self.goal };
        Vec3 { x: self.toward.x - behind.x, y: 0.0, z: self.toward.z - behind.z }
    }

    /// The body's heading this second (design D10): the centroid's displacement since the last sample when it is
    /// `HEADING_MOVED` or more, the axis otherwise (a standing body, the first second, or a strip where the two agree).
    fn heading_now(&mut self, walkers: &[Walker]) -> Vec3 {
        if walkers.is_empty() {
            return self.axis();
        }
        let n = walkers.len() as f32;
        let centroid = walkers.iter().fold(Vec3 { x: 0.0, y: 0.0, z: 0.0 }, |c, w| Vec3 { x: c.x + w.at.x / n, y: 0.0, z: c.z + w.at.z / n });
        let moved = self.last_centroid.map(|last| Vec3 { x: centroid.x - last.x, y: 0.0, z: centroid.z - last.z });
        self.last_centroid = Some(centroid);
        match moved {
            Some(d) if (d.x * d.x + d.z * d.z).sqrt() >= HEADING_MOVED => d,
            _ => self.axis(),
        }
    }

    /// Contact: the first unit at the goal in force, or for `into` and `assault` the first shot.
    pub fn contact(&self) -> bool {
        if matches!(self.kind, Kind::Into | Kind::Assault) { !self.first_shot.is_empty() } else { !self.arrived.is_empty() }
    }

    /// Once a second: speeds, arrivals, first shots (`shooters`: the spawn indices that fired this second), the
    /// spin-up and recovery clocks, the spread at the median arrival, and the body's shape (kept at contact).
    /// `expected` is how many units the drill waits for (`expected`).
    pub fn sample(&mut self, frame: i32, walkers: &[Walker], expected: usize, shooters: &[usize]) {
        let heading = self.heading_now(walkers);
        self.shape = Shape::of(walkers, heading);
        for &i in shooters {
            self.first_shot.entry(i).or_insert(frame);
        }
        let mut shares: Vec<f32> = Vec::new();
        let mut now: HashMap<UnitId, Vec3> = HashMap::new();
        // D21: each member's step along the axis toward the goal in force since the last sample, forward and back.
        let axis = self.axis();
        let norm = (axis.x * axis.x + axis.z * axis.z).sqrt();
        let unit = if norm > 0.0 { (axis.x / norm, axis.z / norm) } else { (0.0, 0.0) };
        self.back_second = 0.0;
        for w in walkers {
            // A unit that has arrived stands; its share counts until then.
            if let Some(last) = self.last.get(&w.id)
                && w.speed > 0.0
                && !self.arrived.contains_key(&w.index)
            {
                shares.push((last.dist2d(w.at) / w.speed).min(1.5));
            }
            if let Some(last) = self.last.get(&w.id) {
                let step = (w.at.x - last.x) * unit.0 + (w.at.z - last.z) * unit.1;
                if step >= 0.0 {
                    self.forward += step;
                } else {
                    self.back -= step;
                    self.back_second -= step;
                }
            }
            now.insert(w.id, w.at);
            if self.kind != Kind::Circuit && w.at.dist2d(self.toward) <= ARRIVED {
                self.arrived.entry(w.index).or_insert(frame);
            }
        }
        self.last = now;
        if let Some((pace, arrived, engaged)) = self.group_pace
            && pace == 0.0
            && !arrived
            && !engaged
            && self.last_order.is_some()
        {
            self.stopped_s += 1;
        }
        self.orders_last_second = std::mem::take(&mut self.orders_second);
        if self.kind == Kind::Circuit {
            self.sample_circuit(frame, walkers);
        }
        // The formed share under a group (D12, D16): every second of a circuit; for the other drills every second
        // until contact, when the slots become the arc's and the group's books stop.
        if let Some((formed, members, _)) = self.group
            && members > 0
            && (self.kind == Kind::Circuit || !self.contact())
        {
            self.formed_shares.push(formed as f32 / members as f32);
        }
        if self.shape_at_contact.is_none() && self.contact() {
            self.shape_at_contact = self.shape;
        }
        if !shares.is_empty() {
            shares.sort_by(f32::total_cmp);
            let median = shares[shares.len() / 2];
            self.shares.push((frame, median));
            let since_turn = self.turned.is_some_and(|t| frame > t);
            if median >= UP_TO_SPEED {
                if since_turn {
                    self.recover.get_or_insert(frame);
                } else {
                    self.spin_up.get_or_insert(frame);
                }
            }
            if self.median_arrival.is_none() {
                self.shares_before_median.extend(shares.iter().copied());
            }
        }
        // The median unit has arrived: the army's shape at that second, taken once (after a reverse's turn, taken
        // again for the way back).
        let half = expected.div_ceil(2).max(1);
        let reset = self.turned.is_some_and(|t| self.median_arrival.is_some_and(|m| m < t));
        if (self.median_arrival.is_none() || reset) && self.arrived.len() >= half && !walkers.is_empty() {
            self.median_arrival = Some(frame);
            let n = walkers.len() as f32;
            let (cx, cz) = walkers.iter().fold((0.0, 0.0), |(x, z), w| (x + w.at.x / n, z + w.at.z / n));
            let centre = Vec3 { x: cx, y: 0.0, z: cz };
            let rms = (walkers.iter().map(|w| w.at.dist2d(centre).powi(2)).sum::<f32>() / n).sqrt();
            let mut nearest: Vec<f32> = walkers
                .iter()
                .map(|w| walkers.iter().filter(|o| o.id != w.id).map(|o| o.at.dist2d(w.at)).fold(f32::INFINITY, f32::min))
                .filter(|d| d.is_finite())
                .collect();
            nearest.sort_by(f32::total_cmp);
            let nn = nearest.get(nearest.len() / 2).copied().unwrap_or(0.0);
            self.spread_at_median = Some((rms, nn));
        }
    }

    /// A circuit's second (D12): each core unit's passes of the lab's corner (`goal`), with hysteresis, its `laps`-th
    /// pass its arrival; each fed unit's joining the core's body; the group's formed share when the director has
    /// noted one.
    fn sample_circuit(&mut self, frame: i32, walkers: &[Walker]) {
        let core_at: Vec<Vec3> = walkers.iter().filter(|w| self.core.contains(&w.index)).map(|w| w.at).collect();
        let n = core_at.len() as f32;
        let body = (!core_at.is_empty()).then(|| core_at.iter().fold(Vec3::default(), |c, p| Vec3 { x: c.x + p.x / n, y: 0.0, z: c.z + p.z / n }));
        for w in walkers {
            if self.core.contains(&w.index) {
                let near = w.at.dist2d(self.goal) <= AT_THE_LAB;
                let entry = self.passes.entry(w.index).or_insert((near, 0));
                if near && !entry.0 {
                    entry.1 += 1;
                    if entry.1 >= self.laps {
                        self.arrived.entry(w.index).or_insert(frame);
                    }
                }
                entry.0 = near;
            } else if self.appeared.contains_key(&w.index)
                && !self.joined.contains_key(&w.index)
                && let Some(body) = body
                && w.at.dist2d(body) <= JOINED
            {
                self.joined.insert(w.index, frame);
            }
        }
    }

    /// Whether the drill is over: every unit it waits for has arrived at the goal in force (after the turn, for a
    /// reverse; the whole list produced and arrived, for a gather), or for `into` every living mobile unit
    /// (`walking`, by spawn index) has fired; for a circuit every core unit has walked its laps and, under a group, the
    /// group stands at the route's end with no exceptions. An assault is never done by the drill: the duel's judge
    /// ends it (the defence or the army wiped, the time limit).
    pub fn done(&self, walking: &[usize]) -> bool {
        let expected = self.expected(walking.len());
        if expected == 0 || self.last_order.is_none() {
            return false;
        }
        match self.kind {
            Kind::Reverse => self.turned.is_some() && self.arrived.len() >= expected,
            Kind::Gather => self.appeared.len() >= expected && self.arrived.len() >= expected,
            // Under a group, the group's word ends it too: a formation of twenty is four ranks deep and its rear rank
            // stands at the arrival radius (d22-a7: one Pawn at 300.08 from the goal, formed and arrived, timed out).
            Kind::Walk => self.arrived.len() >= expected || self.group.is_some_and(|(_, members, done)| done && members >= expected),
            Kind::Into => walking.iter().all(|i| self.first_shot.contains_key(i)),
            Kind::Assault => false,
            Kind::Circuit => {
                let core: Vec<&usize> = self.core.iter().filter(|i| !self.left_behind.contains(i)).collect();
                !core.is_empty() && core.iter().all(|i| self.arrived.contains_key(i)) && self.group.is_none_or(|(_, _, done)| done)
            }
        }
    }

    /// The score, from the frame of the first order.
    pub fn score(&self, started: i32) -> Score {
        let seconds = |f: i32| (f - started) as f32 / FPS as f32;
        let mut arrivals: Vec<i32> = self.arrived.values().copied().collect();
        arrivals.sort();
        let at = |i: usize| arrivals.get(i).map(|f| seconds(*f));
        let arrive = if arrivals.is_empty() { [None; 3] } else { [at(0), at(arrivals.len() / 2), at(arrivals.len() - 1)] };
        let speed_share = (!self.shares_before_median.is_empty()).then(|| self.shares_before_median.iter().sum::<f32>() / self.shares_before_median.len() as f32);
        let mut shots: Vec<i32> = self.first_shot.values().copied().collect();
        shots.sort();
        // A gather's join is the arrival at the rally; a circuit's, reaching the deployed body.
        let joined_at = |i: &usize| if self.kind == Kind::Circuit { self.joined.get(i) } else { self.arrived.get(i) };
        let mut joins: Vec<f32> = self.appeared.iter().filter_map(|(i, appeared)| joined_at(i).map(|a| (a - appeared) as f32 / FPS as f32)).collect();
        joins.sort_by(f32::total_cmp);
        let mut passes: Vec<u32> = self.core.iter().map(|i| self.passes.get(i).map_or(0, |p| p.1)).collect();
        passes.sort_unstable();
        Score {
            kind: self.kind.label().to_string(),
            arrive,
            spread_arrival: self.spread_at_median.map(|(rms, _)| rms),
            nn_arrival: self.spread_at_median.map(|(_, nn)| nn),
            speed_share,
            spin_up_s: self.spin_up.map(seconds),
            recover_s: self.recover.zip(self.turned).map(|(r, t)| (r - t) as f32 / FPS as f32),
            produced: self.appeared.len() as u32,
            join_median_s: joins.get(joins.len() / 2).copied(),
            depth_contact: self.shape_at_contact.map(|s| s.depth),
            width_contact: self.shape_at_contact.map(|s| s.width),
            ramp_half_s: self.shape_at_contact.map(|s| s.ramp_half_s),
            ramp_all_s: self.shape_at_contact.map(|s| s.ramp_all_s),
            first_shot_s: shots.first().map(|f| seconds(*f)),
            shot_ramp_half_s: shots.first().map(|f| (shots[shots.len() / 2] - f) as f32 / FPS as f32),
            shot_ramp_all_s: shots.first().zip(shots.last()).map(|(f, l)| (l - f) as f32 / FPS as f32),
            target_deaths: self.target_deaths,
            taken_s: self.taken.map(seconds),
            lost_before_shot: self.lost_before_shot,
            straight: self.straight,
            walk: self.walk,
            route_s: self.walk.filter(|_| self.slowest > 0.0).map(|w| w / self.slowest),
            laps: passes.get(passes.len() / 2).copied().unwrap_or(0),
            formed_share: (!self.formed_shares.is_empty()).then(|| self.formed_shares.iter().sum::<f32>() / self.formed_shares.len() as f32),
            joined: self.joined.len() as u32,
            left_behind: self.left_behind.len() as u32,
            companies: self.companies_max as u32,
            company_formed: (!self.company_shares.is_empty()).then(|| self.company_shares.iter().sum::<f32>() / self.company_shares.len() as f32),
            backtrack_share: (self.forward > 0.0).then(|| self.back / self.forward),
            anchor_back: self.anchor_back,
            stopped_s: self.stopped_s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn options(kind: Kind) -> Options {
        Options { kind, distance: 3000.0, fight: true, reissue: None, lead: None, pace: None, turn_at: 20 * FPS, lab: "armvp".into(), target: "armshltx".into(), advance: false, from: None, to: None, around: None, feed: Vec::new(), laps: 1, circuit: None, behind: Vec::new(), behind_distance: 0.0, scatter: 0.0, churn: None, churn_def: "armsolar".into(), stall: None, crosser: None, nudge: None, late: None, cap: 0 }
    }

    /// D21's measures: a member's step along the axis toward the goal counts forward, a step away counts back
    /// (the share is back over forward); the anchor's step against its heading sums in `anchor_back` (a march never
    /// adds, a snap onto the body does); a second with the pace at 0 while marching is a stopped second; the orders
    /// each tick count with the ticks that had none.
    #[test]
    fn a_walk_counts_backtracking_the_anchors_snaps_and_stopped_seconds() {
        let mut drill = Drill::new(&options(Kind::Walk), at(2500.0, 1000.0), Vec::new());
        drill.last_order = Some(0);
        drill.first_order = Some(0);
        let walkers = |x: f32| vec![Walker { index: 0, id: UnitId(1), at: at(x, 1000.0), speed: 100.0 }];
        drill.sample(0, &walkers(1100.0), 1, &[]);
        drill.sample(30, &walkers(1200.0), 1, &[]);
        drill.sample(60, &walkers(1160.0), 1, &[]);
        assert!((drill.forward - 100.0).abs() < 0.01 && (drill.back - 40.0).abs() < 0.01, "{} / {}", drill.forward, drill.back);
        assert!((drill.back_second - 40.0).abs() < 0.01);
        // The anchor marches 10 a tick east, then snaps 400 west: only the snap counts.
        drill.note_anchor(at(1500.0, 1000.0), (1.0, 0.0), 100.0, false, false);
        drill.note_anchor(at(1510.0, 1000.0), (1.0, 0.0), 100.0, false, false);
        drill.note_anchor(at(1110.0, 1000.0), (1.0, 0.0), 100.0, false, false);
        assert!((drill.anchor_back - 400.0).abs() < 0.01, "{}", drill.anchor_back);
        // Pace 0 on the march is a stopped second; arrived or engaged it is not.
        drill.note_anchor(at(1110.0, 1000.0), (1.0, 0.0), 0.0, false, false);
        drill.sample(90, &walkers(1160.0), 1, &[]);
        drill.note_anchor(at(1110.0, 1000.0), (1.0, 0.0), 0.0, true, false);
        drill.sample(120, &walkers(1160.0), 1, &[]);
        assert_eq!(drill.stopped_s, 1);
        drill.note_orders(8);
        drill.note_orders(0);
        drill.note_orders(0);
        drill.sample(150, &walkers(1160.0), 1, &[]);
        assert_eq!(drill.orders_last_second, (8, 2));
        let score = drill.score(0);
        assert!((score.backtrack_share.unwrap() - 0.4).abs() < 0.01 && score.anchor_back == 400.0 && score.stopped_s == 1);
    }

    /// A circuit (D12): a core unit's passes of the lab's corner count with hysteresis (the start inside it counts
    /// for nothing), its second pass is its arrival at two laps; a fed unit joins when within 300 of the core's
    /// centroid; the drill ends on the core's laps alone without a group and waits for the group's word with one;
    /// lane off, the route goes out as queued orders.
    #[test]
    fn a_circuit_counts_passes_joiners_and_the_formed_share() {
        let mut drill = Drill::new(&Options { laps: 2, ..options(Kind::Circuit) }, at(5000.0, 1000.0), Vec::new());
        drill.goal = at(1000.0, 1000.0);
        drill.toward = drill.goal;
        drill.route = vec![at(3000.0, 1000.0), at(1000.0, 1000.0), at(3000.0, 1000.0), at(1000.0, 1000.0)];
        drill.core = vec![0];
        drill.last_order = Some(0);
        let walker = |x: f32| Walker { index: 0, id: UnitId(1), at: at(x, 1000.0), speed: 87.0 };
        drill.sample(0, &[walker(1200.0)], 1, &[]);
        drill.sample(FPS, &[walker(2000.0)], 1, &[]);
        drill.sample(2 * FPS, &[walker(1300.0)], 1, &[]);
        assert!(!drill.done(&[0]), "one pass of two");
        drill.sample(3 * FPS, &[walker(2500.0)], 1, &[]);
        drill.appeared.insert(1, 3 * FPS);
        let fed = |x: f32| Walker { index: 1, id: UnitId(2), at: at(x, 1000.0), speed: 87.0 };
        drill.sample(4 * FPS, &[walker(2000.0), fed(1000.0)], 1, &[]);
        assert!(drill.joined.is_empty(), "a thousand from the core");
        drill.sample(5 * FPS, &[walker(1200.0), fed(1400.0)], 1, &[]);
        assert_eq!(drill.joined.get(&1), Some(&(5 * FPS)));
        assert!(drill.done(&[0, 1]));
        let score = drill.score(0);
        assert_eq!((score.laps, score.joined, score.produced), (2, 1, 1));
        assert_eq!(score.arrive[0].map(|s| s.round() as i32), Some(5));
        assert_eq!(score.join_median_s, Some(2.0));
        assert_eq!(score.formed_share, None);
        drill.note_group(1, 2, false);
        assert!(!drill.done(&[0, 1]), "the group's word is awaited");
        drill.note_group(2, 2, true);
        assert!(drill.done(&[0, 1]));
        drill.sample(6 * FPS, &[walker(1200.0), fed(1400.0)], 1, &[]);
        assert_eq!(drill.score(0).formed_share, Some(1.0));
        let orders = drill.orders(UnitId(1), at(0.0, 0.0), at(0.0, 0.0), 0);
        assert_eq!(orders.len(), 4);
        assert!(matches!(orders[0], Command::Fight { queue: false, .. }) && matches!(orders[3], Command::Fight { queue: true, .. }));
    }

    #[test]
    fn a_lead_order_goes_the_lead_ahead_along_the_axis_and_to_the_goal_at_the_end() {
        let mut drill = Drill::new(&Options { lead: Some(100.0), fight: false, reissue: Some(3), ..options(Kind::Reverse) }, at(5000.0, 1000.0), Vec::new());
        // The strip runs from x 3500 to 6500; a unit at the start, 40 off the axis, is sent 100 east on its own line.
        let spawned = at(3500.0, 1040.0);
        let Command::Move { to, .. } = drill.order(UnitId(1), at(3500.0, 1040.0), spawned, 0) else { panic!("a move") };
        assert_eq!((to.x, to.z), (3600.0, 1040.0));
        // Within the lead of the goal: the goal itself.
        let Command::Move { to, .. } = drill.order(UnitId(1), at(6450.0, 1000.0), spawned, 0) else { panic!("a move") };
        assert_eq!((to.x, to.z), (6500.0, 1000.0));
        // After the turn the axis runs west.
        drill.turn(600);
        let Command::Move { to, .. } = drill.order(UnitId(1), at(6000.0, 1000.0), spawned, 0) else { panic!("a move") };
        assert_eq!((to.x, to.z), (5900.0, 1000.0));
    }

    #[test]
    fn a_paced_slot_carries_the_spawn_place_along_the_axis_at_the_pace_and_stops_at_the_goal() {
        let mut drill = Drill::new(&Options { pace: Some(60.0), lead: Some(60.0), fight: false, reissue: Some(3), ..options(Kind::Reverse) }, at(5000.0, 1000.0), Vec::new());
        drill.first_order = Some(100);
        // Spawned 40 north of the axis at the start: at the first order its slot is the lead ahead; ten seconds on,
        // 600 further; the unit's own position does not matter.
        let spawned = at(3500.0, 1040.0);
        let Command::Move { to, .. } = drill.order(UnitId(1), at(3400.0, 900.0), spawned, 100) else { panic!("a move") };
        assert_eq!((to.x, to.z), (3560.0, 1040.0));
        let Command::Move { to, .. } = drill.order(UnitId(1), at(3400.0, 900.0), spawned, 400) else { panic!("a move") };
        assert_eq!((to.x, to.z), (4160.0, 1040.0));
        // Past the strip's length the slot stands at the goal end, in its place.
        let Command::Move { to, .. } = drill.order(UnitId(1), at(3400.0, 900.0), spawned, 100 + 100 * FPS) else { panic!("a move") };
        assert_eq!((to.x, to.z), (6500.0, 1040.0));
        // After the turn the slot sets out from the goal end westward, timed from the turn.
        drill.turn(4000);
        let Command::Move { to, .. } = drill.order(UnitId(1), at(6400.0, 1000.0), spawned, 4000 + 5 * FPS) else { panic!("a move") };
        assert_eq!((to.x, to.z), (6500.0 - 60.0 - 300.0, 1040.0));
    }

    /// A walk: two units at full speed arrive 30 and 31 seconds in; the median arrival takes the shape; the score
    /// says first, median and last, the speed share and the spin-up.
    #[test]
    fn a_walk_is_scored_by_arrivals_shape_and_speed() {
        let mut drill = Drill::new(&options(Kind::Walk), at(5000.0, 1000.0), Vec::new());
        assert_eq!((drill.start.x, drill.goal.x), (3500.0, 6500.0));
        drill.last_order = Some(0);
        let walkers = |t: i32| vec![Walker { index: 0, id: UnitId(1), at: at(3500.0 + 100.0 * t as f32, 1000.0), speed: 100.0 }, Walker { index: 1, id: UnitId(2), at: at(3500.0 + 97.0 * t as f32, 1064.0), speed: 100.0 }];
        for t in 0..=32 {
            drill.sample(t * FPS, &walkers(t), 2, &[]);
            if t < 27 {
                assert!(!drill.done(&[0, 1]));
            }
        }
        assert!(drill.done(&[0, 1]));
        let score = drill.score(0);
        assert_eq!(score.arrive.map(|a| a.map(|s| s.round() as i32)), [Some(27), Some(28), Some(28)]);
        assert_eq!(score.spin_up_s, Some(1.0));
        assert!(score.speed_share.is_some_and(|s| s > 0.95 && s <= 1.0), "{:?}", score.speed_share);
        assert!(score.nn_arrival.is_some_and(|nn| nn > 60.0 && nn < 130.0), "{:?}", score.nn_arrival);
        assert_eq!(score.recover_s, None);
        // Contact at 27 s: the second unit is 81 behind and one rank across; at 100 a second it is 0.8 s from firing.
        assert_eq!(score.depth_contact.map(f32::round), Some(81.0));
        assert_eq!(score.width_contact.map(f32::round), Some(64.0));
        assert!(score.ramp_all_s.is_some_and(|r| (r - 0.81).abs() < 0.01), "{:?}", score.ramp_all_s);
        assert_eq!(score.ramp_half_s, score.ramp_all_s);
    }

    /// A walk into a target: the drill ends when every living unit has fired; the first shot and the shot ramp are
    /// scored, and contact for the shape is the first shot.
    #[test]
    fn an_into_ends_when_everyone_has_fired_and_scores_the_shot_ramp() {
        let mut drill = Drill::new(&options(Kind::Into), at(5000.0, 1000.0), Vec::new());
        drill.last_order = Some(0);
        let walkers = vec![Walker { index: 0, id: UnitId(1), at: at(6100.0, 1000.0), speed: 100.0 }, Walker { index: 1, id: UnitId(2), at: at(5900.0, 1064.0), speed: 100.0 }];
        drill.sample(20 * FPS, &walkers, 2, &[0]);
        assert!(!drill.done(&[0, 1]));
        assert_eq!(drill.shape_at_contact.map(|s| s.depth), Some(200.0));
        drill.sample(21 * FPS, &walkers, 2, &[]);
        drill.sample(23 * FPS, &walkers, 2, &[1]);
        assert!(drill.done(&[0, 1]));
        let score = drill.score(0);
        assert_eq!((score.first_shot_s, score.shot_ramp_half_s, score.shot_ramp_all_s), (Some(20.0), Some(3.0), Some(3.0)));
    }

    /// The heading is the body's: a body walking north on an east-west axis has its depth measured along north once
    /// its centroid has moved (design D10); the first sample takes the axis.
    #[test]
    fn the_heading_follows_the_body_once_it_moves() {
        let mut drill = Drill::new(&options(Kind::Walk), at(5000.0, 1000.0), Vec::new());
        drill.last_order = Some(0);
        // Two units abreast across the axis (north-south), 200 apart: along the axis they are 0 deep.
        let pair = |z: f32| vec![Walker { index: 0, id: UnitId(1), at: at(4000.0, z), speed: 100.0 }, Walker { index: 1, id: UnitId(2), at: at(4000.0, z + 200.0), speed: 100.0 }];
        drill.sample(FPS, &pair(1000.0), 2, &[]);
        assert_eq!(drill.shape.map(|s| (s.depth, s.width)), Some((0.0, 200.0)));
        // The body walks north 100 in the second: the heading is north and the pair is now 200 deep, 0 wide.
        drill.sample(2 * FPS, &pair(900.0), 2, &[]);
        assert_eq!(drill.shape.map(|s| (s.depth, s.width)), Some((200.0, 0.0)));
    }

    /// The shape against a heading: depth along it from the leader back, width across it, the ramp each unit's depth
    /// over its own speed (the median and the largest over the body).
    #[test]
    fn the_shape_is_taken_against_the_heading() {
        let walkers = vec![
            Walker { index: 0, id: UnitId(1), at: at(0.0, 0.0), speed: 100.0 },
            Walker { index: 1, id: UnitId(2), at: at(-100.0, 50.0), speed: 50.0 },
            Walker { index: 2, id: UnitId(3), at: at(-200.0, -50.0), speed: 100.0 },
        ];
        let east = Shape::of(&walkers, at(1.0, 0.0)).unwrap();
        assert_eq!((east.depth, east.width), (200.0, 100.0));
        assert_eq!((east.ramp_half_s, east.ramp_all_s), (2.0, 2.0));
        let north = Shape::of(&walkers, at(0.0, 1.0)).unwrap();
        assert_eq!((north.depth, north.width), (100.0, 200.0));
        assert_eq!((north.ramp_half_s, north.ramp_all_s), (0.5, 1.0));
        assert_eq!(Shape::of(&[], at(1.0, 0.0)), None);
        assert_eq!(Shape::of(&walkers, at(0.0, 0.0)), None);
    }

    /// A reverse: the turn is due `turn_at` after the first order, the goal in force becomes the start, the arrivals
    /// start afresh, and the drill is over when everyone is back.
    #[test]
    fn a_reverse_turns_once_and_ends_when_everyone_is_back() {
        let mut drill = Drill::new(&options(Kind::Reverse), at(5000.0, 1000.0), Vec::new());
        drill.last_order = Some(0);
        assert!(!drill.turn_due(19 * FPS));
        assert!(drill.turn_due(20 * FPS));
        drill.arrived.insert(0, 1);
        drill.turn(20 * FPS);
        assert!(drill.arrived.is_empty() && drill.toward == drill.start && !drill.turn_due(40 * FPS));
        let walker = |x: f32| vec![Walker { index: 0, id: UnitId(1), at: at(x, 1000.0), speed: 100.0 }];
        drill.sample(21 * FPS, &walker(5500.0), 1, &[]);
        drill.sample(22 * FPS, &walker(5450.0), 1, &[]);
        drill.sample(23 * FPS, &walker(5350.0), 1, &[]);
        assert!(!drill.done(&[0]));
        assert_eq!(drill.recover, Some(23 * FPS));
        drill.sample(24 * FPS, &walker(3700.0), 1, &[]);
        assert!(drill.done(&[0]));
        assert_eq!(drill.score(0).recover_s, Some(3.0));
    }

    /// A gather waits for its whole list, and the re-issue clock runs from the last order.
    #[test]
    fn a_gather_waits_for_its_list_and_the_reissue_runs_from_the_last_order() {
        let mut drill = Drill::new(&Options { reissue: Some(2 * FPS), ..options(Kind::Gather) }, at(5000.0, 1000.0), vec![(UnitDefId(1), 2)]);
        assert_eq!(drill.expected(0), 2);
        drill.last_order = Some(10);
        assert!(!drill.reissue_due(69) && drill.reissue_due(70));
        drill.appeared.insert(0, 0);
        drill.arrived.insert(0, 30);
        assert!(!drill.done(&[0]));
        drill.appeared.insert(1, 60);
        drill.arrived.insert(1, 120);
        assert!(drill.done(&[0, 1]));
        let score = drill.score(0);
        assert_eq!((score.produced, score.join_median_s), (2, Some(2.0)));
    }
}
