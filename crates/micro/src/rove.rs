//! H-MICRO-ROVE: a fast soldier (a Rover, a Tick) the host hands over whole. Ten times a second, in this order:
//! it never stands inside the reach of anything that can shoot it (plus a margin: what a mobile shooter covers in
//! 1.5 s, 60 for a building), stepping to the least threatened ground nearby when it is or is about to be; it kills
//! what it finds unguarded within 1,000 (a lone constructor first, then an extractor, a radar, anything else unarmed
//! it kills within 45 s) from a stand: a point within its reach of the target, outside every margin, with a clear way
//! to it, the one that kills soonest counting the walk and its laser's falloff (so an extractor 350 from a light
//! laser turret is killed from the rim of the turret's reach on the far side; `docs/design/2026-11-04-rove-2.md`
//! §2). It is only ever moved, with its target set beside the move (`Command::SetTarget`, BAR's target-on-the-move
//! gadget), so it shoots on the way in, where it stands and on the way out, and the stand is only where it stops
//! (the user, 2026-11-04: never stop attacking a structure on the way in or out of its range); and otherwise it drives to
//! look at the place we know least. Its way to a stand or a look point is the needle (design §5): the straight line
//! when that keeps every shooter at its bar, else a route over its own ground with every cell inside a bar closed and
//! the lips priced (so it walks the middle of a gap between two turrets, not a lip), straightened and cut into legs
//! of 320 queued as one chain of moves; a way is planned again every 5 s, when a shooter comes to cover a leg, and
//! after an evasion; one field a tick a rover, and none for 3 s after a search found nothing. The place (the spots in his start box and his base never seen or not seen for a minute,
//! first; then any spot never seen; then the spot longest out of sight), stopping short of it by six tenths of its
//! sight, on a way that passes no reach. His places go stale in a minute, not three, so it comes back to his
//! perimeter that often (the user, 2026-09-28: "the rovers tend to explore the whole map rather than bother the
//! perimeter of the known enemy base"); his places seen this minute never rank ahead of the map's unseen ones: ranked
//! there, a rover whose way into his base passed a turret's reach walked back and forth between the two edge spots it
//! could reach, each the oldest seen in turn (the first human game of 2026-09-28). Being seen costs it nothing: only what can hit it counts. No host order
//! reaches it (`lib.rs` `tick` drops them), so the hands' states cannot pull it off. A rover is a member of a declared
//! group of kind `Rove` (design D15), whose order may confine it to a region: outside it the rover looks at nothing and
//! attacks nothing, though it may pass through outside ground on its way.
//!
//! An aircraft roves too (a gunship, a bomber: an armed flier with a weapon that hits the ground; thebluegecko,
//! 2026-10-27, L-gecko-air-details: "make sure that Banshees work with Rove", one Banshee to one extractor across the
//! map). It keeps off what hits aircraft, not the ground's guns, and only what would kill it soon: a shooter counts
//! when it and every other within its reach would together take the aircraft's health in `AIR_DANGER_SECONDS`. A
//! Banshee (560) then attacks an extractor beside a light laser turret (10.7 a second against aircraft, 52 s) and
//! keeps off one beside a Thistle (67.6, 8 s); a Wasp (1,560) keeps off two Thistles and not one (23 s).
//!
//! player-9-posing: three scouts sent at his base at G1-G2 never arrived: one pulled home by the hands at the first
//! sight of his commander, one walked into two Pawns beside his lab, one diverted onto a hunt.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bot_protocol::{Command, EnemyUnit, OwnUnit, UnitDefId, UnitId, Vec3};

use super::ground::Ground;
use super::{FAINT, FRAMES_PER_SECOND, LOOKAHEAD_FRAMES, Lane, REORDER_DISTANCE, REORDER_FRAMES, STEP_REACH, Source, View, to_segment};

/// A mobile shooter is kept at its reach plus what it covers in this long, and never less than `MARGIN_MIN`: a
/// Pawn (87 a second, reach 180) at 310, the Rover (168) gaining 81 a second on it.
const MARGIN_SECONDS: f32 = 1.5;
const MARGIN_MIN: f32 = 80.0;
/// A building's reach plus this: a Rover crosses it in a third of a second, and the half-second lookahead is on top.
const MARGIN_STATIC: f32 = 60.0;
/// An evasion steps to the least threatened cell within this.
const STEP_RADIUS: f32 = 200.0;
/// A goal whose way became covered is not taken again for this long.
const BLOCK_FRAMES: i32 = 30 * FRAMES_PER_SECOND;
/// A goal not looked at within this long is given up.
const GOAL_FRAMES: i32 = 120 * FRAMES_PER_SECOND;
/// Standing this near the look point this long without the place counting as seen gives the goal up.
const ARRIVED: f32 = 64.0;
const ARRIVED_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// The look point stands this share of the rover's sight short of the place.
const LOOK_SHARE: f32 = 0.6;
/// Sight when the host has none for the type.
const DEFAULT_SIGHT: f32 = 350.0;
/// A place reached without counting as seen is left alone this long.
const STALE_FRAMES: i32 = 180 * FRAMES_PER_SECOND;
/// A place of his looked at longer ago than this is as good as never seen: the perimeter is walked again every minute.
const PERIMETER_STALE: i32 = 60 * FRAMES_PER_SECOND;
/// Targets are taken within this of the rover.
const ATTACK_RADIUS: f32 = 1000.0;
/// A target that would take one rover longer than this to kill is left: it would stop looking for that long.
const KILL_SECONDS: f32 = 45.0;
/// A rover standing still on its way is sent again after this long.
const RESEND_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// Elmos a frame under which a rover is standing still.
const STILL: f32 = 0.2;
/// Two move orders this close are the same order.
const SAME_MOVE: f32 = 32.0;
/// Staleness counts in steps of this among spots all seen once.
const STALE_STEP: i32 = 60 * FRAMES_PER_SECOND;
/// An aircraft rover keeps off a shooter whose cover, with every other shooter within its reach, would kill it in
/// this long (the module's comment).
pub const AIR_DANGER_SECONDS: f32 = 15.0;
/// A stand is at most this short of the rover's reach from its target (the engine's own in-range test has slack of
/// its own; a stand on the reach's edge would have the rover edging in and out).
const STAND_SLACK: f32 = 20.0;
/// No stand nearer its target than this: the target's body and the rover's (an extractor is 48 across, a Rover 21);
/// nor nearer than the target's death blast reaches plus `BLAST_SLACK` (drill N: a quarter of a Rover's health to
/// an extractor's blast at 80), within the rover's reach.
const STAND_MIN: f32 = 60.0;
const BLAST_SLACK: f32 = 10.0;
/// Stands are tried this many ways round the target, at radii this far apart in from the reach.
const STAND_DIRECTIONS: usize = 16;
const STAND_STEP: f32 = 40.0;
/// A rover standing still this far off the point it was sent to is sent again.
const STAND_DRIFT: f32 = 24.0;
/// A stand keeps this much beyond a building's reach, not the margin a moving rover keeps (the margin is for
/// crossing at speed): the engine's own overshoot on arrival (16 elmos in drill R) and a standing rover's jitter.
/// While it attacks from a stand the evade reads buildings at this bar too, else the stand's own rim starts it.
const STAND_CLEAR: f32 = 40.0;
/// Where it stands is kept as its stand unless another kills this much sooner: no shuffle for a hair of falloff.
const STAND_HOLD_SECONDS: f32 = 0.5;
/// The needle (the module's comment): cells within this of a shooter's bar cost `NEEDLE_LIP_COST` more a cell (the
/// flat rate is 10), so a way keeps to the middle of a gap.
const NEEDLE_SLACK: f32 = 100.0;
const NEEDLE_LIP_COST: u32 = 10;
/// A way is sent as legs no longer than this (two Rover-seconds), queued as one chain.
const NEEDLE_LEG: f32 = 320.0;
/// A way is planned again after this long, and when a shooter comes to cover one of its legs.
const NEEDLE_REPLAN_FRAMES: i32 = 5 * FRAMES_PER_SECOND;
/// After a search that found no way, no field is run for this long: the straight ways alone.
const NEEDLE_RETRY_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// A leg's cell quantisation: a leg this much inside a bar is not "covered" (a cell's half diagonal is 11).
const NEEDLE_TOLERANCE: f32 = 16.0;
/// A route keeps this much beyond the bar (the cells closed, the straightening's lines): the engine's path follows
/// a leg within a few elmos, and the evade reads the bar exactly where the rover stands.
const NEEDLE_CLEAR: f32 = 24.0;
/// A leg is passed when the rover is within this of its point.
const LEG_DONE: f32 = 48.0;
/// Standing at its stand this long without the target's health falling, it gives the target up: nothing in the
/// lane knows the ground between (drill R: an extractor on a mound, the beam from below blocked by the slope).
const STAND_PATIENCE_FRAMES: i32 = 6 * FRAMES_PER_SECOND;
/// The target is set again this often while it attacks: the gadget refuses a target out of sight as the order
/// goes out (K-mech-set-target-fires-on-the-move) and drops one unseen, so the set is a heartbeat, not a word.
const TARGET_RESEND_FRAMES: i32 = 3 * FRAMES_PER_SECOND;

/// A place a rover may look at: its name in the host's words, where it is, the frame it was last within sight of
/// anything of ours (`None`: never), and whether it is his (in his start box, or his base).
#[derive(Clone, Debug, PartialEq)]
pub struct RoveGoal {
    pub name: String,
    pub at: Vec3,
    pub seen: Option<i32>,
    pub theirs: bool,
}

/// Where a rover attacks a target from: the open (nothing armed reaches the target itself), or the rim of a guard's
/// reach (the target stands inside a margin and the stand outside it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandKind {
    Open,
    Rim,
}

/// What a rover did that the host should hear about, each once.
#[derive(Clone, Debug, PartialEq)]
pub enum RoveWhat {
    /// An enemy unit came within its sight for the first time any rover saw it.
    Found { enemy: UnitId, def: UnitDefId, at: Vec3 },
    /// It started attacking this unarmed unit, from this kind of stand.
    Attacks { target: UnitId, def: UnitDefId, at: Vec3, from: StandKind },
    /// It stepped away from this shooter (`None`: a radar contact), standing here.
    Evades { from: Option<UnitDefId>, at: Vec3 },
    /// It gave up a place, and why.
    GivesUp { goal: String, why: String },
    /// It gave up a target it stood at without hurting it (the ground between, the lane supposes), and why.
    Abandons { target: UnitId, def: UnitDefId, at: Vec3, why: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoveEvent {
    pub rover: UnitId,
    pub what: RoveWhat,
}

/// What a rover is doing this tick, for the host's picture.
#[derive(Clone, Debug, PartialEq)]
pub struct RoveReport {
    /// "evading", "threading" (on a way of more than one leg, to its stand or its look point), "attacking", "looking"
    /// or "waiting" (every way to a place worth a look passes a reach).
    pub doing: &'static str,
    /// The place it is going to look at, and where that is.
    pub goal: Option<(String, Vec3)>,
    pub target: Option<UnitId>,
    /// Where it stands to shoot the target, and whether that is the rim of a guard's reach.
    pub stand: Option<Vec3>,
    pub rim: bool,
    /// Evasions since it began roving.
    pub evasions: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Order {
    Move(Vec3),
    Stop,
}

impl Order {
    fn same(self, other: Order) -> bool {
        match (self, other) {
            (Order::Move(a), Order::Move(b)) => a.dist2d(b) < SAME_MOVE,
            (Order::Stop, Order::Stop) => true,
            _ => false,
        }
    }
}

/// A way to the rover's stand or look point: its legs, the point last, and when it was planned.
struct Way {
    legs: Vec<Vec3>,
    planned: i32,
    /// Planned this tick: sent whatever was sent before.
    fresh: bool,
}

/// What the needle found.
enum Found {
    Way(Vec<Vec3>),
    NoWay,
    /// The tick's one field was spent on another search; the straight way alone was tried.
    Untried,
}

/// The way from `from` to `to` keeping every shooter at its `bar`: the straight line when that is clear, else a route
/// over the rover's ground with every cell inside a bar closed and the lips priced, straightened (no straight line
/// across a bar) and cut into legs (the module's comment). One field a tick: `budget`.
fn needle(ground: Option<&Ground>, danger: &[&Source], bar: &dyn Fn(&Source) -> f32, from: Vec3, to: Vec3, budget: &mut bool) -> Found {
    if danger.iter().all(|s| to_segment(s.pos, from, to) >= bar(s)) {
        return Found::Way(vec![to]);
    }
    // The point itself inside a bar: no way reaches it, and no field is spent to say so.
    if danger.iter().any(|s| s.pos.dist2d(to) < bar(s)) {
        return Found::NoWay;
    }
    let Some(ground) = ground else { return Found::NoWay };
    if !*budget {
        return Found::Untried;
    }
    *budget = false;
    let mut costs = ground.costs().to_vec();
    let half_diagonal = ground.cell() * std::f32::consts::FRAC_1_SQRT_2;
    for s in danger {
        ground.stamp_disc(&mut costs, s.pos, bar(s) + NEEDLE_CLEAR + NEEDLE_SLACK, Some(NEEDLE_LIP_COST));
        ground.stamp_disc(&mut costs, s.pos, bar(s) + NEEDLE_CLEAR + half_diagonal, None);
    }
    // The way's end may itself lie within the clearance of a bar (a stand on the rim keeps the bar and no more):
    // the pocket round it is opened again as the ground has it, down to the bar.
    let original = ground.costs();
    for (i, centre) in ground.cells_within(to, NEEDLE_CLEAR + half_diagonal + ground.cell()) {
        if danger.iter().all(|s| s.pos.dist2d(centre) >= bar(s)) {
            costs[i] = original[i];
        }
    }
    let roomy = |a: Vec3, b: Vec3| danger.iter().all(|s| to_segment(s.pos, a, b) >= bar(s) + NEEDLE_CLEAR);
    match ground.way_over(&costs, from, to, &roomy) {
        Some(points) => Found::Way(legs_of(from, &points, to)),
        None => Found::NoWay,
    }
}

/// The point `along` elmos ahead on a way from `from` through `legs`: where a rover on the way will be.
fn along_way(from: Vec3, legs: &[Vec3], along: f32) -> Vec3 {
    let mut left = along;
    let mut a = from;
    for b in legs {
        let d = a.dist2d(*b);
        if d >= left {
            let t = if d > 0.0 { left / d } else { 0.0 };
            return Vec3 { x: a.x + (b.x - a.x) * t, y: 0.0, z: a.z + (b.z - a.z) * t };
        }
        left -= d;
        a = *b;
    }
    a
}

/// The way's points cut into legs no longer than `NEEDLE_LEG`, `to` itself last.
fn legs_of(from: Vec3, points: &[Vec3], to: Vec3) -> Vec<Vec3> {
    let mut legs = Vec::new();
    let mut last = from;
    for p in points {
        let n = (last.dist2d(*p) / NEEDLE_LEG).ceil().max(1.0) as usize;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            legs.push(Vec3 { x: last.x + (p.x - last.x) * t, y: 0.0, z: last.z + (p.z - last.z) * t });
        }
        last = *p;
    }
    if legs.last().is_none_or(|l| l.dist2d(to) > 1.0) {
        legs.push(to);
    }
    legs
}

/// Whether every leg of a way from `from` keeps every shooter at its bar, less a cell of tolerance.
fn legs_clear(danger: &[&Source], bar: &dyn Fn(&Source) -> f32, from: Vec3, legs: &[Vec3]) -> bool {
    let mut a = from;
    for b in legs {
        if danger.iter().any(|s| to_segment(s.pos, a, *b) < bar(s) - NEEDLE_TOLERANCE) {
            return false;
        }
        a = *b;
    }
    true
}

struct Goal {
    name: String,
    look: Vec3,
    since: i32,
    /// When it first stood within `ARRIVED` of the look point.
    arrived: Option<i32>,
}

/// One rover's memory.
pub(crate) struct Rover {
    goal: Option<Goal>,
    target: Option<UnitId>,
    /// The stand it attacks the target from, and whether that is the rim of a guard's reach.
    stand: Option<(Vec3, StandKind)>,
    /// The frame the target was last set on the unit (`Command::SetTarget`); `None` while nothing is set.
    target_sent: Option<i32>,
    /// Since when it has stood at its stand, and the target's health then; the patience clock.
    stand_since: Option<(i32, f32)>,
    /// The way it is on, to its stand or its look point.
    way: Option<Way>,
    /// No field is run before this frame (a search found no way).
    no_way_until: i32,
    /// Targets it will not take again until the frame.
    blocked_targets: Vec<(UnitId, i32)>,
    /// The cell it is stepping to, and when it was sent there.
    evading: Option<(Vec3, i32)>,
    sent: Option<(Order, i32)>,
    /// Places it will not take again until the frame.
    blocked: Vec<(String, i32)>,
    health: f32,
    evasions: u32,
}

impl Rover {
    fn report(&self) -> RoveReport {
        let doing = if self.evading.is_some() {
            "evading"
        } else if self.way.as_ref().is_some_and(|w| w.legs.len() > 1) {
            "threading"
        } else if self.target.is_some() {
            "attacking"
        } else if self.goal.is_some() {
            "looking"
        } else {
            "waiting"
        };
        RoveReport { doing, goal: self.goal.as_ref().map(|g| (g.name.clone(), g.look)), target: self.target, stand: self.stand.map(|s| s.0), rim: self.stand.is_some_and(|s| s.1 == StandKind::Rim), evasions: self.evasions }
    }
}

/// How far beyond its reach a shooter is kept.
fn margin(s: &Source) -> f32 {
    if s.mobile { (s.speed * MARGIN_SECONDS).max(MARGIN_MIN) } else { MARGIN_STATIC }
}

/// A shooter's reach with its margin: its D-gun counts.
fn keep_off(s: &Source) -> f32 {
    s.reach.max(s.dgun) + margin(s)
}

/// What a rover standing to shoot keeps off: a building's reach plus `STAND_CLEAR`, a mobile shooter's whole
/// margin (it closes).
fn keep_off_standing(s: &Source) -> f32 {
    if s.mobile { keep_off(s) } else { s.reach.max(s.dgun) + STAND_CLEAR }
}

/// How deep `p` stands inside the kept-off distances widened by `slack`, summed over the shooters. An evasion
/// prices its cells with a slack of its step: with none, every cell beyond the margin tied, the tie went to the one
/// nearest its goal or home, and the rover slid round the margin's rim while a Pawn closed on it (a chase test at
/// 189 from a Pawn of reach 180).
fn depth(danger: &[&Source], p: Vec3, slack: f32) -> f32 {
    danger.iter().map(|s| (keep_off(s) + slack - s.pos.dist2d(p)).max(0.0)).sum()
}

/// What an aircraft with this much health keeps off, of the shooters that hit aircraft (the module's comment).
fn air_danger(sources: &[Source], health: f32) -> Vec<&Source> {
    sources
        .iter()
        .filter(|s| {
            let cover: f32 = sources.iter().filter(|o| o.pos.dist2d(s.pos) <= s.reach).map(|o| o.weight).sum();
            cover * AIR_DANGER_SECONDS >= health
        })
        .collect()
}

/// What to kill first: a constructor walking about, an extractor, a radar, anything else.
fn target_class(d: &bot_protocol::UnitDefInfo) -> u8 {
    if d.build_speed > 0.0 && d.speed > 0.0 {
        0
    } else if d.extracts_metal > 0.0 {
        1
    } else if d.radar_range > 0.0 {
        2
    } else {
        3
    }
}

impl Lane {
    /// A rover type's ground for the needle: its class's passable grid priced for its half-width and a cell, built
    /// once and kept; `None` without terrain or a class.
    fn rover_ground(&mut self, view: &dyn View, def: UnitDefId) -> Option<Arc<Ground>> {
        if let Some(g) = self.rover_grounds.get(&def) {
            return g.clone();
        }
        let ground = view.terrain().zip(view.def(def)).and_then(|(terrain, d)| {
            let grid = self.classes.passable(terrain, d.move_class?);
            let wanted = d.footprint.0.max(d.footprint.1) as f32 * 4.0 + terrain.cell;
            Ground::new(terrain, &[grid], wanted).map(Arc::new)
        });
        self.rover_grounds.insert(def, ground.clone());
        ground
    }

    /// What this unit is doing as a rover, if it is one (H-MICRO-ROVE).
    pub fn roving(&self, unit: UnitId) -> Option<RoveReport> {
        self.rovers.get(&unit).map(Rover::report)
    }

    /// Every rover's report, for a host that reads the lane from another thread.
    pub fn rovers_roving(&self) -> HashMap<UnitId, RoveReport> {
        self.rovers.iter().map(|(id, r)| (*id, r.report())).collect()
    }

    /// H-MICRO-ROVE, every tick, for every rover: evade, else attack, else look (the module's comment).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn rove(&mut self, view: &dyn View, rovers: &[(&OwnUnit, Option<(Vec3, Vec3)>)], enemies: &[EnemyUnit], sources: &[Source], air_sources: &[Source], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, events: &mut Vec<RoveEvent>, debug: bool) {
        let danger_all: Vec<&Source> = sources.iter().filter(|s| s.weight >= FAINT).collect();
        let mut goals_of: HashMap<UnitDefId, Vec<RoveGoal>> = HashMap::new();
        for (unit, within) in rovers {
            let flies = view.def(unit.def).is_some_and(super::is_air_soldier);
            let danger: Vec<&Source> = if flies {
                air_danger(air_sources, unit.health)
            } else {
                // A torpedo shooter is no danger to a rover on land (the engine: water weapons reach only what is in the water).
                danger_all.iter().copied().filter(|s| !s.water_only || unit.pos.y <= 0.0).collect()
            };
            // What it knows can hit it where it stands: an aircraft hit by a shooter it chose not to keep off (a
            // light laser turret beside its extractor) is not hit by something unseen (the air rove drill: every
            // Banshee gave up its next place under the turret's fire).
            let known_cover = |p: Vec3| flies && air_sources.iter().any(|s| s.pos.dist2d(p) <= keep_off(s));
            let stats = view.stats(unit.def).unwrap_or_default();
            let sight = if stats.sight > 0.0 { stats.sight } else { DEFAULT_SIGHT };
            // A region the order confines it to: nothing outside it is looked at or attacked.
            let inside = |p: Vec3| within.is_none_or(|(a, b)| p.x >= a.x && p.x <= b.x && p.z >= a.z && p.z <= b.z);
            let name = view.def(unit.def).map_or("?", |d| d.name.as_str()).to_string();
            let ground = self.rover_ground(view, unit.def);
            let mut rover = match self.rovers.remove(&unit.id) {
                Some(r) => r,
                None => {
                    fired.push("H-MICRO-ROVE");
                    self.counts.0 += 1;
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} roves", view.label(), unit.id.0);
                    }
                    Rover { goal: None, target: None, stand: None, target_sent: None, stand_since: None, way: None, no_way_until: 0, blocked_targets: Vec::new(), evading: None, sent: None, blocked: Vec::new(), health: unit.health, evasions: 0 }
                }
            };
            rover.blocked.retain(|(_, until)| *until > frame);
            rover.blocked_targets.retain(|(_, until)| *until > frame);
            // Stunned: nothing it is sent does anything until it wakes.
            if unit.stunned > 0.0 {
                rover.health = unit.health;
                self.rovers.insert(unit.id, rover);
                continue;
            }
            let give_up = |rover: &mut Rover, why: String, block: i32, events: &mut Vec<RoveEvent>| {
                if let Some(goal) = rover.goal.take() {
                    rover.blocked.push((goal.name.clone(), frame + block));
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} gives up {} ({why})", view.label(), unit.id.0, goal.name);
                    }
                    events.push(RoveEvent { rover: unit.id, what: RoveWhat::GivesUp { goal: goal.name, why } });
                }
            };

            // What it sees, reported once for all rovers.
            for e in enemies.iter().filter(|e| e.pos.dist2d(unit.pos) <= sight) {
                if let Some(def) = e.def
                    && self.rove_found.insert(e.id)
                {
                    events.push(RoveEvent { rover: unit.id, what: RoveWhat::Found { enemy: e.id, def, at: e.pos } });
                }
            }

            // 1. Out of every reach. Where it stands, and where half a second of its velocity takes it: along its way
            // when it is on one (the way keeps the bar; the velocity's line poked into a margin on every bend of a
            // chain and at full speed onto the rim, drills N and R), the straight velocity otherwise.
            let next = match rover.way.as_ref() {
                Some(way) => along_way(unit.pos, &way.legs, unit.vel.x.hypot(unit.vel.z) * LOOKAHEAD_FRAMES),
                None => Vec3 { x: unit.pos.x + unit.vel.x * LOOKAHEAD_FRAMES, y: 0.0, z: unit.pos.z + unit.vel.z * LOOKAHEAD_FRAMES },
            };
            let bar = |s: &Source| if rover.stand.is_some() { keep_off_standing(s) } else { keep_off(s) };
            let threat = danger.iter().copied().filter(|s| s.pos.dist2d(unit.pos) < bar(s) || s.pos.dist2d(next) < bar(s)).min_by(|a, b| (a.pos.dist2d(unit.pos) - bar(a)).total_cmp(&(b.pos.dist2d(unit.pos) - bar(b))));
            let hit_unseen = threat.is_none() && unit.health < rover.health - 1.0 && !known_cover(unit.pos);
            rover.health = unit.health;
            if let Some(s) = threat {
                // From anything as fast as it, toward home; else round the threat toward where it was going.
                let prefer = if s.mobile && s.speed >= stats.speed { view.home() } else { rover.goal.as_ref().map_or(view.home(), |g| g.look) };
                let passable = self.classes.passable_for(view, unit.def);
                let cell = match self.grid.as_ref() {
                    Some(grid) => grid.lowest_within_by(unit.pos, STEP_RADIUS, passable.as_deref().map(Vec::as_slice), prefer, &|c| depth(&danger, c, STEP_RADIUS)),
                    None => None,
                }
                .unwrap_or_else(|| {
                    let (dx, dz) = (unit.pos.x - s.pos.x, unit.pos.z - s.pos.z);
                    let d = dx.hypot(dz).max(1.0);
                    Vec3 { x: unit.pos.x + dx / d * STEP_RADIUS, y: 0.0, z: unit.pos.z + dz / d * STEP_RADIUS }
                });
                let fresh = rover.evading.is_none();
                // The target stays set through the step: it shoots on the way out while the target is in reach. The
                // way is planned again from where the step ends.
                if fresh {
                    rover.evasions += 1;
                    rover.way = None;
                    events.push(RoveEvent { rover: unit.id, what: RoveWhat::Evades { from: s.def, at: unit.pos } });
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} evades a shooter {:.0} away (keeps {:.0}) to ({:.0}, {:.0})", view.label(), unit.id.0, s.pos.dist2d(unit.pos), keep_off(s), cell.x, cell.z);
                    }
                }
                // Again when the cell moved, or when it has got to where it was sent (a step not stretched ends there).
                let arrived = rover.sent.is_some_and(|(o, _)| matches!(o, Order::Move(t) if t.dist2d(unit.pos) < SAME_MOVE));
                let reorder = rover.evading.is_none_or(|(sent, at)| (sent.dist2d(cell) > REORDER_DISTANCE || arrived) && frame - at >= REORDER_FRAMES);
                if reorder {
                    // Stretched past the cell so the engine does not brake short of it, unless that runs into a margin.
                    let far = self.classes.snap_for(view, unit.def, Vec3 { x: unit.pos.x + (cell.x - unit.pos.x) * STEP_REACH, y: 0.0, z: unit.pos.z + (cell.z - unit.pos.z) * STEP_REACH });
                    let to = if depth(&danger, far, 0.0) > depth(&danger, cell, 0.0) { cell } else { far };
                    rover.evading = Some((cell, frame));
                    rover.sent = Some((Order::Move(to), frame));
                    self.counts.1 += 1;
                    commands.push(Command::Move { unit: unit.id, to, queue: false });
                }
                self.rovers.insert(unit.id, rover);
                continue;
            }
            let after_evading = rover.evading.take().is_some();
            // Something out of sight is shooting it where it goes: that place is not worth it. Once per goal held
            // `ARRIVED_FRAMES`: under steady fire it gave up a goal a tick and ran out of places in seconds.
            if hit_unseen && rover.goal.as_ref().is_some_and(|g| frame - g.since >= ARRIVED_FRAMES) {
                give_up(&mut rover, "hit by something out of sight on the way".to_string(), BLOCK_FRAMES, events);
            }
            let still = unit.vel.x.hypot(unit.vel.z) < STILL;
            let send = |lane: &mut Lane, rover: &mut Rover, order: Order, commands: &mut Vec<Command>| {
                let same = rover.sent.is_some_and(|(o, _)| o.same(order));
                if !same || after_evading {
                    rover.sent = Some((order, frame));
                    lane.counts.1 += 1;
                    commands.push(match order {
                        Order::Move(to) => Command::Move { unit: unit.id, to, queue: false },
                        Order::Stop => Command::Stop { unit: unit.id },
                    });
                }
            };
            // The way it is on, sent as one queued chain of moves: when planned this tick, when its end is new, after
            // an evasion, and when the rover stands still short of its end (not at it: a stopped rover stands).
            let send_way = |lane: &mut Lane, rover: &mut Rover, commands: &mut Vec<Command>| {
                let Some(way) = rover.way.as_mut() else { return };
                while way.legs.len() > 1 && unit.pos.dist2d(way.legs[0]) < LEG_DONE {
                    way.legs.remove(0);
                }
                let end = *way.legs.last().expect("a way has a leg");
                let order = Order::Move(end);
                let same = rover.sent.is_some_and(|(o, _)| o.same(order));
                let stale = still && end.dist2d(unit.pos) > STAND_DRIFT && rover.sent.is_none_or(|(_, at)| frame - at >= RESEND_FRAMES);
                if way.fresh || !same || after_evading || stale {
                    rover.sent = Some((order, frame));
                    lane.counts.1 += 1;
                    for (i, leg) in way.legs.iter().enumerate() {
                        commands.push(Command::Move { unit: unit.id, to: *leg, queue: i > 0 });
                    }
                }
                way.fresh = false;
            };
            // One field a tick, none for a while after a search found nothing.
            let mut budget = frame >= rover.no_way_until;

            // 2. Kill what is unarmed near it, from a stand (the module's comment; `docs/design/2026-11-04-rove-2.md`
            // §2): a point within its reach of the target, outside every margin, with a clear way to it. The rover is
            // only ever moved, its target set beside the move, so it shoots from the moment the target is in reach.
            let reach = stats.reach - STAND_SLACK;
            let damage_at = |d: f32| stats.dps * (1.0 - stats.falloff * (d / stats.reach.max(1.0)).min(1.0));
            let kill_at = |e: &EnemyUnit, d: f32| e.health.max(1.0) / damage_at(d).max(0.01);
            // Unarmed (or a turret still being built, or stunned for longer than the way there and the kill take:
            // harmless until it is dead), on the ground, killable from close in within the limit, within the radius
            // and the region.
            let blocked_targets: Vec<UnitId> = rover.blocked_targets.iter().map(|b| b.0).collect();
            let attackable = |e: &EnemyUnit| -> Option<u8> {
                let d = view.def(e.def?)?;
                let air = d.speed > 0.0 && d.move_class.is_none();
                let asleep = e.stunned > e.pos.dist2d(unit.pos) / stats.speed.max(1.0) + kill_at(e, STAND_MIN);
                let blocked = blocked_targets.contains(&e.id);
                ((d.weapon_count == 0 || e.being_built || asleep) && !air && !blocked && stats.dps > 0.0 && kill_at(e, STAND_MIN) <= KILL_SECONDS && e.pos.dist2d(unit.pos) <= ATTACK_RADIUS && inside(e.pos)).then(|| target_class(d))
            };
            // The nearest a stand comes to the target: its body, or its death blast.
            let stand_min = |e: &EnemyUnit| e.def.and_then(|d| view.def(d)).and_then(|d| d.death_blast).map_or(STAND_MIN, |b| (b.radius + BLAST_SLACK).max(STAND_MIN).min(reach));
            // A candidate snapped to its cell's centre moves up to half a diagonal: that much of the reach's slack
            // goes to the snap.
            let snap_slack = ground.as_ref().map_or(0.0, |g| g.cell() * std::f32::consts::FRAC_1_SQRT_2);
            // Outside what a standing rover keeps off; the way there is the needle's.
            let stand_ok = |p: Vec3, e: &EnemyUnit| p.dist2d(e.pos) <= reach + snap_slack && p.dist2d(e.pos) >= stand_min(e) && danger.iter().all(|s| s.pos.dist2d(p) >= keep_off_standing(s));
            let standing_bar = |s: &Source| keep_off_standing(s);
            // The stand that kills soonest among those with a way: the walk there and the kill from there, the
            // laser's falloff counted (a beam laser does half its damage at its full reach, so the rim is twice as
            // slow as close in); the straight ways cost nothing to test, the tick's one field goes to the best stand
            // whose straight way is covered.
            let mut stand_for = |e: &EnemyUnit, budget: &mut bool| -> Option<(Vec3, f32, Vec<Vec3>)> {
                // Where it stands, when that will do, with a head start: no shuffle for a hair of falloff.
                let mut candidates: Vec<(Vec3, f32)> = stand_ok(unit.pos, e).then(|| (unit.pos, kill_at(e, unit.pos.dist2d(e.pos)))).filter(|(_, kill)| *kill <= KILL_SECONDS).map(|(p, kill)| (p, kill - STAND_HOLD_SECONDS)).into_iter().collect();
                let mut r = reach;
                while r >= stand_min(e) {
                    for k in 0..STAND_DIRECTIONS {
                        let a = k as f32 * std::f32::consts::TAU / STAND_DIRECTIONS as f32;
                        let p = self.classes.snap_for(view, unit.def, Vec3 { x: e.pos.x + r * a.cos(), y: 0.0, z: e.pos.z + r * a.sin() });
                        if !stand_ok(p, e) {
                            continue;
                        }
                        let kill = kill_at(e, p.dist2d(e.pos));
                        if kill <= KILL_SECONDS {
                            candidates.push((p, unit.pos.dist2d(p) / stats.speed.max(1.0) + kill));
                        }
                    }
                    r -= STAND_STEP;
                }
                candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
                for (p, seconds) in candidates {
                    if let Found::Way(legs) = needle(ground.as_deref(), &danger, &standing_bar, unit.pos, p, budget) {
                        return Some((p, seconds, legs));
                    }
                }
                None
            };
            // A target whose stands a field found no way to is rested a while: the next tick's field goes elsewhere.
            let rest = |rover: &mut Rover, e: &EnemyUnit, had: bool, has: bool| {
                if had && !has {
                    rover.blocked_targets.push((e.id, frame + NEEDLE_RETRY_FRAMES));
                }
            };
            let kind_of = |e: &EnemyUnit| if depth(&danger, e.pos, 0.0) > 0.0 { StandKind::Rim } else { StandKind::Open };
            // Its target is kept while it is still worth it and it has a stand: the stand it has while that is still
            // outside every margin and in reach, with its way kept while its legs are clear and it is fresh (else
            // planned again), else another stand; neither, and the target is let go.
            let mut chosen: Option<(&EnemyUnit, Vec3)> = None;
            if let Some(e) = rover.target.and_then(|t| enemies.iter().find(|e| e.id == t)).filter(|e| attackable(e).is_some()) {
                let kept = rover.stand.map(|s| s.0).filter(|s| stand_ok(*s, e));
                match kept {
                    Some(stand) => {
                        let at_stand = stand.dist2d(unit.pos) <= STAND_DRIFT;
                        let way_ok = at_stand || rover.way.as_ref().is_some_and(|w| frame - w.planned < NEEDLE_REPLAN_FRAMES && legs_clear(&danger, &standing_bar, unit.pos, &w.legs));
                        if way_ok {
                            chosen = Some((e, stand));
                        } else {
                            match needle(ground.as_deref(), &danger, &standing_bar, unit.pos, stand, &mut budget) {
                                Found::Way(legs) => {
                                    rover.way = Some(Way { legs, planned: frame, fresh: true });
                                    chosen = Some((e, stand));
                                }
                                // The old way stays for the tick; the search comes next tick.
                                Found::Untried if rover.way.is_some() => chosen = Some((e, stand)),
                                _ => rover.way = None,
                            }
                        }
                    }
                    None => rover.way = None,
                }
                if chosen.is_none() {
                    let had = budget;
                    match stand_for(e, &mut budget) {
                        Some((stand, _, legs)) => {
                            rover.way = Some(Way { legs, planned: frame, fresh: true });
                            chosen = Some((e, stand));
                        }
                        None => rest(&mut rover, e, had, budget),
                    }
                }
            }
            if chosen.is_none() {
                let others: HashSet<UnitId> = self.rovers.values().filter_map(|r| r.target).collect();
                let mut ranked: Vec<(u8, bool, f32, &EnemyUnit)> = enemies.iter().filter_map(|e| Some((attackable(e)?, !others.contains(&e.id), e.pos.dist2d(unit.pos), e))).collect();
                ranked.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2)));
                let mut best = None;
                for (.., e) in ranked {
                    let had = budget;
                    if let Some((stand, _, legs)) = stand_for(e, &mut budget) {
                        best = Some((e, stand, legs));
                        break;
                    }
                    rest(&mut rover, e, had, budget);
                }
                if let Some((e, stand, legs)) = best {
                    rover.way = Some(Way { legs, planned: frame, fresh: true });
                    if rover.target != Some(e.id) {
                        let from = kind_of(e);
                        events.push(RoveEvent { rover: unit.id, what: RoveWhat::Attacks { target: e.id, def: e.def.expect("attackable has a type"), at: e.pos, from } });
                        if debug {
                            eprintln!("{} f={frame} micro: {name}#{} attacks {} at ({:.0}, {:.0}) from ({:.0}, {:.0}){}", view.label(), unit.id.0, e.id.0, e.pos.x, e.pos.z, stand.x, stand.z, if from == StandKind::Rim { ", the rim of its guard's reach" } else { "" });
                        }
                    }
                    chosen = Some((e, stand));
                }
            }
            // At its stand with the target's health not falling for the patience: the target is given up for a while.
            if let Some((e, stand)) = chosen
                && stand.dist2d(unit.pos) <= STAND_DRIFT
            {
                match rover.stand_since {
                    Some((_, h)) if e.health < h - 0.5 => rover.stand_since = Some((frame, e.health)),
                    Some((since, _)) if frame - since >= STAND_PATIENCE_FRAMES => {
                        let why = format!("stood {} s at its stand without hurting it", STAND_PATIENCE_FRAMES / FRAMES_PER_SECOND);
                        if debug {
                            eprintln!("{} f={frame} micro: {name}#{} gives up {} ({why})", view.label(), unit.id.0, e.id.0);
                        }
                        events.push(RoveEvent { rover: unit.id, what: RoveWhat::Abandons { target: e.id, def: e.def.expect("attackable has a type"), at: e.pos, why } });
                        rover.blocked_targets.push((e.id, frame + BLOCK_FRAMES));
                        chosen = None;
                    }
                    Some(_) => {}
                    None => rover.stand_since = Some((frame, e.health)),
                }
            }
            match chosen {
                Some((e, stand)) => {
                    if rover.target != Some(e.id) {
                        // A set target replaces the one the gadget held: no clear between two.
                        rover.target = Some(e.id);
                        rover.target_sent = None;
                        rover.stand_since = None;
                    }
                    if rover.stand.is_none_or(|s| s.0.dist2d(stand) > 1.0) {
                        rover.stand_since = None;
                    }
                    rover.stand = Some((stand, kind_of(e)));
                    if rover.target_sent.is_none_or(|at| frame - at >= TARGET_RESEND_FRAMES) {
                        rover.target_sent = Some(frame);
                        self.counts.1 += 1;
                        commands.push(Command::SetTarget { unit: unit.id, target: e.id });
                    }
                    send_way(self, &mut rover, commands);
                    self.rovers.insert(unit.id, rover);
                    continue;
                }
                None => {
                    // A target let go takes its stand and its way with it; a look's way is not the attack rung's.
                    if rover.target.take().is_some() {
                        if rover.target_sent.take().is_some() {
                            self.counts.1 += 1;
                            commands.push(Command::ClearTarget { unit: unit.id });
                        }
                        rover.way = None;
                    }
                    rover.stand = None;
                    rover.stand_since = None;
                }
            }

            // 3. Look at the place we know least.
            let goals = goals_of.entry(unit.def).or_insert_with(|| view.rove_goals(unit.def));
            let mut look_at = |at: Vec3| -> Vec3 {
                let (dx, dz) = (unit.pos.x - at.x, unit.pos.z - at.z);
                let d = dx.hypot(dz);
                let short = (sight * LOOK_SHARE).min(d);
                if d < 1.0 { at } else { self.classes.snap_for(view, unit.def, Vec3 { x: at.x + dx / d * short, y: 0.0, z: at.z + dz / d * short }) }
            };
            if let Some(goal) = rover.goal.as_mut() {
                let now = goals.iter().find(|g| g.name == goal.name);
                if goal.look.dist2d(unit.pos) < ARRIVED && goal.arrived.is_none() {
                    goal.arrived = Some(frame);
                }
                let looked = now.is_none_or(|g| g.seen.is_some_and(|f| f >= goal.since));
                let arrived_blind = goal.arrived.is_some_and(|at| frame - at >= ARRIVED_FRAMES);
                if looked {
                    rover.goal = None;
                } else if frame - goal.since >= GOAL_FRAMES {
                    give_up(&mut rover, format!("not reached in {} s", GOAL_FRAMES / FRAMES_PER_SECOND), BLOCK_FRAMES, events);
                } else if arrived_blind {
                    give_up(&mut rover, "reached, and it still does not count as seen".to_string(), STALE_FRAMES, events);
                } else {
                    // Its way: kept while its legs are clear and it is fresh, else planned again; none, and the
                    // place is given up.
                    let look = goal.look;
                    let way_ok = rover.way.as_ref().is_some_and(|w| frame - w.planned < NEEDLE_REPLAN_FRAMES && legs_clear(&danger, &keep_off, unit.pos, &w.legs));
                    if !way_ok {
                        match needle(ground.as_deref(), &danger, &keep_off, unit.pos, look, &mut budget) {
                            Found::Way(legs) => rover.way = Some(Way { legs, planned: frame, fresh: true }),
                            Found::Untried if rover.way.is_some() => {}
                            Found::Untried => rover.way = None,
                            Found::NoWay => {
                                rover.way = None;
                                give_up(&mut rover, "its way passes the reach of something that can shoot it".to_string(), BLOCK_FRAMES, events);
                            }
                        }
                    }
                }
            }
            if rover.goal.is_none() {
                rover.way = None;
                let taken: HashSet<&str> = self.rovers.values().filter_map(|r| r.goal.as_ref().map(|g| g.name.as_str())).collect();
                let tier = |g: &RoveGoal| -> u8 {
                    if g.theirs && g.seen.is_none_or(|f| frame - f > PERIMETER_STALE) {
                        0
                    } else if g.seen.is_none() {
                        1
                    } else {
                        2
                    }
                };
                let mut ranked: Vec<_> = goals
                    .iter()
                    .filter(|g| inside(g.at))
                    .filter(|g| !rover.blocked.iter().any(|(n, _)| *n == g.name))
                    // In its sight now: being looked at already.
                    .filter(|g| g.at.dist2d(unit.pos) > sight)
                    .map(|g| (g, look_at(g.at)))
                    .map(|(g, look)| {
                        // By the minute, nearest first; a place seen this last minute after every other, the oldest of
                        // those first (by the minute alone it tied with the place just looked at, the nearest).
                        let age = g.seen.map_or(0, |f| frame - f);
                        let recent = tier(g) == 2 && age < STALE_STEP;
                        let staleness = if tier(g) != 2 { 0 } else if recent { -age } else { -(age / STALE_STEP) };
                        ((taken.contains(g.name.as_str()), tier(g), recent, staleness), unit.pos.dist2d(look), g, look)
                    })
                    .collect();
                ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
                // The first in rank with a way: the straight ways cost nothing, the tick's one field goes to the best
                // place whose straight way is covered; a field that found nothing rests the fields for a while.
                let had_budget = budget;
                let mut best = None;
                for (_, _, g, look) in ranked {
                    let had = budget;
                    match needle(ground.as_deref(), &danger, &keep_off, unit.pos, look, &mut budget) {
                        Found::Way(legs) => {
                            best = Some((g, look, legs));
                            break;
                        }
                        // A field that found no way: the place is given up as one whose way is covered.
                        Found::NoWay if had && !budget => rover.blocked.push((g.name.clone(), frame + BLOCK_FRAMES)),
                        _ => {}
                    }
                }
                if best.is_none() && had_budget && !budget {
                    rover.no_way_until = frame + NEEDLE_RETRY_FRAMES;
                }
                if let Some((g, look, legs)) = best {
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} goes to look at {} from ({:.0}, {:.0}){}", view.label(), unit.id.0, g.name, look.x, look.z, if legs.len() > 1 { format!(" by {} legs", legs.len()) } else { String::new() });
                    }
                    rover.goal = Some(Goal { name: g.name.clone(), look, since: frame, arrived: None });
                    rover.way = Some(Way { legs, planned: frame, fresh: true });
                }
            }
            match (rover.goal.is_some(), rover.way.is_some()) {
                (true, true) => send_way(self, &mut rover, commands),
                // A way not yet found this tick (the field spent): nothing until the next.
                (true, false) => {}
                // Nowhere to go without passing a reach: it stands where it is safe.
                (false, _) if rover.sent.is_some_and(|(o, _)| !matches!(o, Order::Stop)) => send(self, &mut rover, Order::Stop, commands),
                (false, _) => {}
            }
            self.rovers.insert(unit.id, rover);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::*;
    use crate::{GroupKind, GroupOrder, HuntOrder};

    /// A rover confined to a region (the order's `within`) looks at no place and attacks no unit outside it:
    /// free, it takes the nearer unseen place and the unguarded extractor beside it; confined, it leaves both for
    /// the place inside the region.
    #[test]
    fn a_rover_confined_to_a_region_looks_and_attacks_inside_it_alone() {
        let goals = vec![goal("spot_1", 500.0, 500.0, None, false), goal("spot_2", 3000.0, 3000.0, None, false)];
        let extractor = enemy(50, EXTRACTOR, 1200.0, 1200.0);
        let mut free = roving_lane(&[10]);
        let host = Host::new(goals.clone());
        let out = free.tick(&host, &tick(30, vec![rover(10, 1500.0, 1500.0)], vec![extractor.clone()]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit, target } if unit.0 == 10 && target.0 == 50)), "free, it attacks the extractor beside it: {:?}", out.commands);
        let mut confined = Lane::default();
        let within = Some((at(2000.0, 2000.0), at(4000.0, 4000.0)));
        confined.set_group("scouts", vec![UnitId(10)], GroupOrder::new(GroupKind::Rove { within }, Vec::new()));
        let host = Host::new(goals);
        let out = confined.tick(&host, &tick(30, vec![rover(10, 1500.0, 1500.0)], vec![extractor]), &mut Vec::new(), false);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::SetTarget { .. })), "confined, the extractor outside is left: {:?}", out.commands);
        let report = confined.roving(UnitId(10)).expect("roving");
        assert_eq!((report.doing, report.goal.map(|(name, _)| name)), ("looking", Some("spot_2".to_string())), "the place inside, not the nearer one outside");
    }

    /// A stunned unit shoots nothing until it wakes: a rover inside a stunned turret's reach stays, attacks it when
    /// it stays stunned longer than the way there and the kill, and leaves it alone when it wakes sooner; a stunned
    /// rover of ours is sent nothing.
    #[test]
    fn a_stunned_turret_is_no_threat_and_a_target_while_it_sleeps() {
        let tower = |stunned: f32| EnemyUnit { stunned, ..enemy(60, TOWER, 1300.0, 1000.0) };
        let attacks = |out: &crate::Output| out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit, target } if unit.0 == 10 && target.0 == 60));
        let evades = |out: &crate::Output| out.rove.iter().any(|e| matches!(e.what, RoveWhat::Evades { .. }));
        let host = Host::new(Vec::new());
        let out = roving_lane(&[10]).tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![tower(0.0)]), &mut Vec::new(), false);
        assert!(evades(&out) && !attacks(&out), "awake: {:?}", out.commands);
        // 300 health at 40 a second is 7.5 s, and 300 to go at 168 another 1.8.
        let out = roving_lane(&[10]).tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![tower(12.0)]), &mut Vec::new(), false);
        assert!(!evades(&out) && attacks(&out), "stunned 12 s: {:?}", out.commands);
        let out = roving_lane(&[10]).tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![tower(5.0)]), &mut Vec::new(), false);
        assert!(!evades(&out) && !attacks(&out), "stunned 5 s: {:?}", out.commands);
        let stunned = OwnUnit { stunned: 3.0, ..rover(10, 1000.0, 1000.0) };
        let out = roving_lane(&[10]).tick(&host, &tick(30, vec![stunned], vec![tower(0.0)]), &mut Vec::new(), false);
        assert!(out.commands.is_empty(), "our rover stunned: {:?}", out.commands);
    }

    /// An aircraft rover keeps off his anti-air, not the ground's guns, and only what would kill it within
    /// `AIR_DANGER_SECONDS` (L-gecko-air-details): a Banshee attacks an extractor beside a light laser turret, which a
    /// Rover leaves; beside a Thistle it takes the unguarded one further off; with a Wasp's 1,560 health, one Thistle
    /// (23 s to kill it) does not keep it off.
    #[test]
    fn an_aircraft_rover_keeps_off_anti_air_that_would_kill_it_and_nothing_else() {
        let banshee = |health: f32| OwnUnit { def: BANSHEE, health, max_health: health, ..rover(10, 1000.0, 1500.0) };
        let attacks = |out: &crate::Output| out.commands.iter().find_map(|c| match c { Command::SetTarget { unit, target } if unit.0 == 10 => Some(target.0), _ => None });
        let guarded = enemy(50, EXTRACTOR, 1800.0, 1500.0);
        let open = enemy(51, EXTRACTOR, 1000.0, 2300.0);

        let tower = enemy(60, TOWER, 1850.0, 1550.0);
        let out = roving_lane(&[10]).tick(&Host::new(Vec::new()), &tick(30, vec![banshee(560.0)], vec![guarded.clone(), open.clone(), tower.clone()]), &mut Vec::new(), false);
        assert_eq!(attacks(&out), Some(50), "the light laser turret takes 52 s to kill it: the nearer extractor: {:?}", out.commands);
        let out = roving_lane(&[10]).tick(&Host::new(Vec::new()), &tick(30, vec![rover(10, 1000.0, 1500.0)], vec![guarded.clone(), open.clone(), tower]), &mut Vec::new(), false);
        assert_eq!(attacks(&out), Some(51), "a Rover leaves what the turret covers: {:?}", out.commands);

        let thistle = enemy(61, THISTLE, 1850.0, 1550.0);
        let out = roving_lane(&[10]).tick(&Host::new(Vec::new()), &tick(30, vec![banshee(560.0)], vec![guarded.clone(), open.clone(), thistle.clone()]), &mut Vec::new(), false);
        assert_eq!(attacks(&out), Some(51), "the Thistle kills it in 8 s: the unguarded one: {:?}", out.commands);
        let out = roving_lane(&[10]).tick(&Host::new(Vec::new()), &tick(30, vec![banshee(1560.0)], vec![guarded.clone(), open.clone(), thistle.clone()]), &mut Vec::new(), false);
        assert_eq!(attacks(&out), Some(50), "one Thistle against 1,560: {:?}", out.commands);
        let second = enemy(62, THISTLE, 1850.0, 1450.0);
        let out = roving_lane(&[10]).tick(&Host::new(Vec::new()), &tick(30, vec![banshee(1560.0)], vec![guarded, open, thistle, second]), &mut Vec::new(), false);
        assert_eq!(attacks(&out), Some(51), "two Thistles kill it in 11.5 s: {:?}", out.commands);
    }

    #[test]
    fn a_rover_looks_first_at_his_start_box_and_never_where_another_rover_goes() {
        let host = Host::new(vec![
            goal("spot_1", 800.0, 800.0, None, false),
            goal("spot_2", 3000.0, 3000.0, None, true),
            goal("spot_3", 3200.0, 2800.0, None, true),
            goal("spot_4", 2900.0, 3100.0, Some(900), true),
        ]);
        let mut lane = roving_lane(&[10, 11]);
        let out = lane.tick(&host, &tick(1000, vec![rover(10, 300.0, 300.0), rover(11, 320.0, 300.0)], Vec::new()), &mut Vec::new(), false);
        let goals: Vec<String> = [10, 11].iter().map(|id| lane.roving(UnitId(*id)).unwrap().goal.unwrap().0).collect();
        // Both unlooked spots in his box, one each: not spot_4 (his, looked at 3 s ago) nor spot_1 (never looked at,
        // but ours), though both are nearer.
        assert_eq!(goals, ["spot_2", "spot_3"]);
        // It stops short of the place by six tenths of its sight.
        let to = moves_of(&out.commands, 10)[0];
        let place = host.goals.iter().find(|g| g.name == goals[0]).unwrap().at;
        assert!((to.dist2d(place) - 0.6 * 635.0).abs() < 2.0, "{to:?}");
        assert_eq!(out.fired, vec!["H-MICRO-ROVE", "H-MICRO-ROVE"]);
    }

    #[test]
    fn a_rover_steps_away_from_a_pawn_before_it_is_in_reach() {
        let host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        // 250 from a Pawn: outside its reach (180), inside its reach and 1.5 s of its walking (310).
        let pawn = enemy(50, PAWN, 1250.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![pawn]), &mut Vec::new(), false);
        let to = moves_of(&out.commands, 10);
        assert_eq!(to.len(), 1);
        assert!(to[0].dist2d(at(1250.0, 1000.0)) > 250.0 + 150.0, "steps well away: {to:?}");
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "evading");
        assert!(out.rove.iter().any(|e| matches!(e.what, RoveWhat::Evades { from: Some(PAWN), .. })));
    }

    #[test]
    fn a_rover_inside_a_towers_reach_steps_out_of_it() {
        let mut host = Host::new(Vec::new());
        host.buildings.push((UnitId(60), TOWER, at(2000.0, 2000.0)));
        let mut lane = roving_lane(&[10]);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 2300.0, 2000.0)], Vec::new()), &mut Vec::new(), false);
        let to = moves_of(&out.commands, 10);
        assert!(to[0].x > 2300.0 + 100.0, "outward: {to:?}");
    }

    #[test]
    fn his_places_seen_this_minute_rank_after_the_maps_unseen_ones() {
        // His two edge spots were looked at 10 s and 20 s ago; a third of his is in a tower's reach; one of ours in
        // the middle was never seen. The rover goes to the unseen one, not back to his older edge spot (the first
        // human game of 2026-09-28: rovers walking between two edge spots).
        let mut host = Host::new(vec![
            goal("spot_1", 2500.0, 300.0, Some(1000 - 300), true),
            goal("spot_2", 4400.0, 300.0, Some(1000 - 600), true),
            goal("spot_3", 3500.0, 1200.0, None, true),
            goal("spot_4", 2000.0, 2500.0, None, false),
        ]);
        host.buildings.push((UnitId(60), TOWER, at(3500.0, 1200.0)));
        let mut lane = roving_lane(&[10]);
        lane.tick(&host, &tick(1000, vec![rover(10, 3500.0, 300.0)], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_4");
        // Once his edge spot is a minute stale it comes first again.
        let mut lane = roving_lane(&[10]);
        lane.tick(&host, &tick(1000 + PERIMETER_STALE, vec![rover(10, 3500.0, 300.0)], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_2");
    }

    #[test]
    fn a_tower_being_built_guards_nothing_and_is_itself_a_target() {
        let host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        let mut frame_of_tower = enemy(60, TOWER, 1100.0, 1000.0);
        (frame_of_tower.being_built, frame_of_tower.health) = (true, 50.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![frame_of_tower]), &mut Vec::new(), false);
        // Already within reach of it, it stands where it is: no evasion, no walk.
        assert!(moves_of(&out.commands, 10).iter().all(|m| m.dist2d(at(1000.0, 1000.0)) < 1.0), "no evasion from a tower that cannot shoot: {:?}", out.commands);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit: UnitId(10), target: UnitId(60) })), "{:?}", out.commands);
    }

    /// The target is set beside a move to the stand, never an attack order (the rover shoots on the way in and out,
    /// the stand is only where it stops); the stand is within its reach of the target and outside every margin.
    #[test]
    fn a_rover_kills_the_unguarded_extractor_and_leaves_the_guarded_one() {
        let mut host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        // The nearer extractor stands 200 from a tower: no point within the rover's reach of it is outside the
        // tower's reach with the stand's clearance (470).
        host.buildings.push((UnitId(60), TOWER, at(1500.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        let guarded = enemy(70, EXTRACTOR, 1300.0, 1000.0);
        let open = enemy(71, EXTRACTOR, 1000.0, 500.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![guarded, open]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit: UnitId(10), target: UnitId(71) })), "{:?}", out.commands);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "never an attack order: {:?}", out.commands);
        let stand = moves_of(&out.commands, 10);
        assert_eq!(stand.len(), 1, "{:?}", out.commands);
        assert!(stand[0].dist2d(at(1000.0, 500.0)) <= 160.0 && stand[0].dist2d(at(1000.0, 500.0)) >= 130.0, "the stand is within reach of the extractor and outside its death blast (120): {:?}", stand[0]);
        assert!(stand[0].dist2d(at(1500.0, 1000.0)) >= 470.0, "and outside the tower's reach with the stand's clearance: {:?}", stand[0]);
        let report = lane.roving(UnitId(10)).unwrap();
        assert_eq!((report.doing, report.target, report.rim), ("attacking", Some(UnitId(71)), false));
        assert!(out.rove.iter().any(|e| matches!(e.what, RoveWhat::Attacks { target: UnitId(71), from: StandKind::Open, .. })), "{:?}", out.rove);
        // A constructor walking about comes before an extractor.
        let mut lane = roving_lane(&[10]);
        let builder = enemy(72, CONSTRUCTOR, 700.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![enemy(71, EXTRACTOR, 1000.0, 800.0), builder]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { target: UnitId(72), .. })), "{:?}", out.commands);
    }

    /// The rim (the rove-2 design §2): an extractor 320 from a tower is inside the tower's margin, but a point 150
    /// or more beyond it on the far side is 470 or more from the tower, outside its reach with the stand's clearance,
    /// and within the rover's reach of the extractor (and outside its death blast of 120); the rover approaching
    /// from that side takes the stand there and sets the extractor as its target. Standing there still, it is
    /// neither sent again nor given an attack; the set target is sent again as a heartbeat every three seconds;
    /// the target dead, the set is cleared.
    #[test]
    fn a_rover_kills_a_guarded_extractor_from_the_rim_of_the_guards_reach() {
        // The place to look at afterwards lies away from the tower, so the way there is clear.
        let mut host = Host::new(vec![goal("spot_2", 300.0, 3000.0, None, true)]);
        host.buildings.push((UnitId(60), TOWER, at(1500.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        let guarded = enemy(70, EXTRACTOR, 1180.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 600.0, 1000.0)], vec![guarded.clone()]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit: UnitId(10), target: UnitId(70) })), "{:?}", out.commands);
        let stand = moves_of(&out.commands, 10);
        assert_eq!(stand.len(), 1, "{:?}", out.commands);
        let stand = stand[0];
        assert!(stand.dist2d(guarded.pos) <= 160.0, "within reach of the extractor: {stand:?}");
        assert!(stand.dist2d(at(1500.0, 1000.0)) >= 470.0, "outside the tower's reach with the stand's clearance: {stand:?}");
        assert!(stand.dist2d(at(1500.0, 1000.0)) < 490.0, "inside the margin a moving rover keeps (the rim is the gain): {stand:?}");
        assert!(stand.x < guarded.pos.x, "on the far side from the tower: {stand:?}");
        let report = lane.roving(UnitId(10)).unwrap();
        assert_eq!((report.doing, report.rim, report.stand), ("attacking", true, Some(stand)));
        assert!(out.rove.iter().any(|e| matches!(e.what, RoveWhat::Attacks { target: UnitId(70), from: StandKind::Rim, .. })), "{:?}", out.rove);
        // Fifty short of the stand at full speed toward the tower: no evasion, it brakes at the stand.
        let mut arriving = rover(10, stand.x - 50.0, stand.z);
        arriving.vel = at(5.6, 0.0);
        let out = lane.tick(&host, &tick(33, vec![arriving], vec![guarded.clone()]), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "attacking", "{:?}", out.commands);
        // At the stand, still: nothing sent for 3 s but the heartbeat, which comes at 3 s.
        let mut standing = rover(10, stand.x, stand.z);
        standing.vel = at(0.0, 0.0);
        let out = lane.tick(&host, &tick(60, vec![standing.clone()], vec![guarded.clone()]), &mut Vec::new(), false);
        assert!(out.commands.is_empty(), "{:?}", out.commands);
        let out = lane.tick(&host, &tick(150, vec![standing.clone()], vec![guarded.clone()]), &mut Vec::new(), false);
        assert_eq!(out.commands.len(), 1, "{:?}", out.commands);
        assert!(matches!(out.commands[0], Command::SetTarget { target: UnitId(70), .. }));
        // The extractor dead: the set target is cleared; the stand lay inside the margin a moving rover keeps, so the
        // next tick steps out of it before it goes to look at the next place.
        let out = lane.tick(&host, &tick(153, vec![standing.clone()], Vec::new()), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::ClearTarget { unit: UnitId(10) })), "{:?}", out.commands);
        assert_eq!(lane.roving(UnitId(10)).unwrap().target, None);
        lane.tick(&host, &tick(156, vec![standing], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "evading");
    }

    /// Standing at its stand six seconds without the target's health falling (drill R: an extractor on a mound,
    /// the beam from below blocked by the slope), the rover gives the target up for thirty seconds and says so.
    #[test]
    fn a_stand_that_hurts_nothing_for_six_seconds_is_given_up() {
        let host = Host::new(vec![goal("spot_2", 300.0, 3000.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        let target = enemy(70, EXTRACTOR, 1150.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![target.clone()]), &mut Vec::new(), false);
        let stand = lane.roving(UnitId(10)).unwrap().stand.expect("a stand");
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { target: UnitId(70), .. })), "{:?}", out.commands);
        let mut standing = rover(10, stand.x, stand.z);
        standing.vel = at(0.0, 0.0);
        // Five seconds at the stand, the health unchanged: still attacking.
        for f in [33, 60, 90, 120, 150, 180] {
            lane.tick(&host, &tick(f, vec![standing.clone()], vec![target.clone()]), &mut Vec::new(), false);
        }
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "attacking");
        // The health falling resets the clock; not falling for six seconds ends it.
        let mut hurt = target.clone();
        hurt.health = 250.0;
        lane.tick(&host, &tick(210, vec![standing.clone()], vec![hurt.clone()]), &mut Vec::new(), false);
        lane.tick(&host, &tick(360, vec![standing.clone()], vec![hurt.clone()]), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "attacking");
        let out = lane.tick(&host, &tick(393, vec![standing.clone()], vec![hurt.clone()]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::ClearTarget { unit: UnitId(10) })), "{:?}", out.commands);
        assert!(out.rove.iter().any(|e| matches!(&e.what, RoveWhat::Abandons { target: UnitId(70), .. })), "{:?}", out.rove);
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "looking");
        // Not taken again for thirty seconds.
        let out = lane.tick(&host, &tick(600, vec![standing.clone()], vec![hurt.clone()]), &mut Vec::new(), false);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::SetTarget { .. })), "{:?}", out.commands);
        let out = lane.tick(&host, &tick(393 + 30 * 30 + 3, vec![standing], vec![hurt]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { target: UnitId(70), .. })), "{:?}", out.commands);
    }

    /// The needle (the rove-2 design §5): with ground, a place whose straight way passes a tower's margin is reached
    /// round it: the way is a chain of queued moves, the first unqueued, every leg clear of the margin, the look
    /// point last; the rover is "threading" on it.
    #[test]
    fn a_place_behind_a_tower_is_reached_round_its_margin() {
        let mut host = Host::new(vec![goal("spot_2", 3000.0, 1000.0, None, true)]);
        host.terrain = Some(flat_terrain());
        host.buildings.push((UnitId(60), TOWER, at(2000.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], Vec::new()), &mut Vec::new(), false);
        let report = lane.roving(UnitId(10)).unwrap();
        assert_eq!((report.doing, report.goal.as_ref().map(|g| g.0.as_str())), ("threading", Some("spot_2")));
        let moves: Vec<(Vec3, bool)> = out.commands.iter().filter_map(|c| match c { Command::Move { unit: UnitId(10), to, queue } => Some((*to, *queue)), _ => None }).collect();
        assert!(moves.len() > 1, "{moves:?}");
        assert!(!moves[0].1 && moves[1..].iter().all(|m| m.1), "the first unqueued, the rest queued: {moves:?}");
        let mut a = at(1000.0, 1000.0);
        for (b, _) in &moves {
            assert!(to_segment(at(2000.0, 1000.0), a, *b) >= 490.0 - NEEDLE_TOLERANCE, "a leg inside the margin: {a:?} to {b:?} of {moves:?}");
            a = *b;
        }
        assert!(a.dist2d(at(2619.0, 1000.0)) < 20.0, "the look point last: {a:?}");
        // The same tick again sends nothing; three seconds on, standing still short of the end, the chain goes again.
        let out = lane.tick(&host, &tick(33, vec![rover(10, 1000.0, 1000.0)], Vec::new()), &mut Vec::new(), false);
        assert!(out.commands.is_empty(), "{:?}", out.commands);
    }

    /// The rim from the near side (drill R's missing half): the straight way to the stand on the far side of the
    /// extractor passes the tower; with ground the rover threads round the tower's reach to it.
    #[test]
    fn a_rim_stand_on_the_far_side_is_reached_round_the_tower() {
        let mut host = Host::new(vec![goal("spot_2", 300.0, 3000.0, None, true)]);
        host.terrain = Some(flat_terrain());
        host.buildings.push((UnitId(60), TOWER, at(1500.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        let guarded = enemy(70, EXTRACTOR, 1150.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1700.0, 1700.0)], vec![guarded.clone()]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::SetTarget { unit: UnitId(10), target: UnitId(70) })), "{:?}", out.commands);
        let report = lane.roving(UnitId(10)).unwrap();
        assert_eq!((report.doing, report.rim), ("threading", true));
        let stand = report.stand.expect("a stand");
        // Snapped to its cell's centre: up to half a diagonal (11) past the reach's slack.
        assert!(stand.x < guarded.pos.x && stand.dist2d(guarded.pos) <= 172.0, "the stand on the far side within reach: {stand:?}");
        let moves = moves_of(&out.commands, 10);
        assert!(moves.len() > 1, "{moves:?}");
        let mut a = at(1700.0, 1700.0);
        for b in &moves {
            assert!(to_segment(at(1500.0, 1000.0), a, *b) >= 470.0 - NEEDLE_TOLERANCE, "a leg inside the tower's reach with the clearance: {a:?} to {b:?} of {moves:?}");
            a = *b;
        }
        assert!(a.dist2d(stand) < 1.0, "the stand last: {a:?} against {stand:?}");
    }

    #[test]
    fn a_place_whose_way_passes_a_tower_is_not_taken() {
        let mut host = Host::new(vec![goal("spot_2", 3000.0, 1000.0, None, true), goal("spot_3", 1000.0, 3000.0, None, true)]);
        // The tower stands on the way east.
        host.buildings.push((UnitId(60), TOWER, at(2000.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_3");
    }

    #[test]
    fn the_hosts_orders_never_reach_a_rover() {
        let host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        let mut orders = vec![Command::Move { unit: UnitId(10), to: at(200.0, 200.0), queue: false }, Command::Move { unit: UnitId(11), to: at(200.0, 200.0), queue: false }];
        lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], Vec::new()), &mut orders, false);
        assert_eq!(orders.len(), 1);
        assert!(matches!(orders[0], Command::Move { unit: UnitId(11), .. }));
    }

    #[test]
    fn what_a_rover_sees_is_reported_once() {
        let host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        let far = enemy(80, EXTRACTOR, 1000.0, 2000.0);
        let near = enemy(81, EXTRACTOR, 1000.0, 1500.0);
        let found = |out: &super::super::Output| out.rove.iter().filter(|e| matches!(e.what, RoveWhat::Found { .. })).count();
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![far.clone(), near.clone()]), &mut Vec::new(), false);
        assert_eq!(found(&out), 1, "only what is within its sight");
        let out = lane.tick(&host, &tick(33, vec![rover(10, 1000.0, 1000.0)], vec![far, near]), &mut Vec::new(), false);
        assert_eq!(found(&out), 0);
    }

    #[test]
    fn a_place_looked_at_is_done_and_the_next_is_taken() {
        let mut host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true), goal("spot_3", 3000.0, 600.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        lane.tick(&host, &tick(30, vec![rover(10, 2500.0, 2500.0)], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_2");
        host.goals[0].seen = Some(45);
        lane.tick(&host, &tick(48, vec![rover(10, 2600.0, 2600.0)], Vec::new()), &mut Vec::new(), false);
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_3");
    }

    /// A minute of a Rover (168 a second, turning at once) on a map of 25 spots 800 apart (the east two columns his),
    /// chased by a Pawn at 87: moved each tick toward its last order, the Pawn straight at it. It never comes within
    /// the Pawn's reach, and it goes on looking at places the while.
    #[test]
    fn a_chased_rover_is_never_in_reach_and_keeps_looking() {
        let spots: Vec<RoveGoal> = (0..25).map(|i| goal(&format!("spot_{i}"), 400.0 + 800.0 * (i % 5) as f32, 400.0 + 800.0 * (i / 5) as f32, None, i % 5 >= 3)).collect();
        let mut host = Host::new(spots);
        let mut lane = roving_lane(&[10]);
        let (mut me, mut pawn) = (rover(10, 1000.0, 1000.0), enemy(50, PAWN, 1700.0, 1100.0));
        let mut heading_to = me.pos;
        let mut closest = f32::INFINITY;
        let (mut evasions, mut waiting) = (0, 0);
        for step in 0..600 {
            let frame = 30 + step * 3;
            let out = lane.tick(&host, &tick(frame, vec![me.clone()], vec![pawn.clone()]), &mut Vec::new(), false);
            evasions += out.rove.iter().filter(|e| matches!(e.what, RoveWhat::Evades { .. })).count();
            if let Some(to) = moves_of(&out.commands, 10).last() {
                heading_to = *to;
            }
            let go = |from: Vec3, to: Vec3, per_tick: f32| {
                let d = from.dist2d(to);
                if d <= per_tick { to } else { at(from.x + (to.x - from.x) / d * per_tick, from.z + (to.z - from.z) / d * per_tick) }
            };
            let was = me.pos;
            me.pos = go(me.pos, heading_to, 168.0 / 10.0);
            me.vel = at((me.pos.x - was.x) / 3.0, (me.pos.z - was.z) / 3.0);
            pawn.pos = go(pawn.pos, me.pos, 87.0 / 10.0);
            closest = closest.min(me.pos.dist2d(pawn.pos));
            // The host counts a place seen when the rover has it within its sight, as the survey does.
            waiting += usize::from(lane.roving(UnitId(10)).is_some_and(|r| r.doing == "waiting"));
            for g in host.goals.iter_mut().filter(|g| g.at.dist2d(me.pos) <= 635.0) {
                g.seen = Some(frame);
            }
        }
        assert!(closest > 180.0 + 40.0, "came within {closest:.0} of the Pawn");
        assert!(evasions >= 1);
        assert!(waiting < 60, "waited {waiting} ticks of 600");
        let looked = host.goals.iter().filter(|g| g.seen.is_some()).count();
        assert!(looked >= 15, "looked at {looked} of 25");
        // His two columns first: all ten looked at.
        assert!(host.goals.iter().filter(|g| g.theirs).all(|g| g.seen.is_some()));
    }

    #[test]
    fn hit_from_out_of_sight_it_gives_the_place_up() {
        let host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true), goal("spot_3", 3000.0, 600.0, None, true)]);
        let mut lane = roving_lane(&[10]);
        lane.tick(&host, &tick(30, vec![rover(10, 2500.0, 2500.0)], Vec::new()), &mut Vec::new(), false);
        let mut hurt = rover(10, 2550.0, 2550.0);
        hurt.health = 80.0;
        let out = lane.tick(&host, &tick(120, vec![hurt], Vec::new()), &mut Vec::new(), false);
        assert!(out.rove.iter().any(|e| matches!(&e.what, RoveWhat::GivesUp { goal, .. } if goal == "spot_2")));
        assert_eq!(lane.roving(UnitId(10)).unwrap().goal.unwrap().0, "spot_3");
    }

    /// The hunt (H-MICRO-HUNT; the fixture lives here): whether a hunter can catch or survive its quarry is the hands'
    /// call, so the engine keeps a hunter that is no faster than its quarry and one that is hurt (player-24: its own
    /// "slower" drop ended 46 of 47 hunts in their first tick, a Blitz after a Blitz or a Rover, against the hands' offer).
    #[test]
    fn a_hunter_no_faster_than_its_quarry_and_hurt_keeps_hunting() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let hunt = HuntOrder { quarry: UnitId(99), leash: 900.0, lost_frames: crate::HUNT_LOST_FRAMES, since: 30 };
        lane.set_group("hunt", vec![UnitId(10)], GroupOrder::new(GroupKind::Hunt(hunt), Vec::new()));
        let mut hunter = rover(10, 2500.0, 2500.0);
        hunter.health = 30.0;
        let out = lane.tick(&host, &tick(30, vec![hunter], vec![enemy(99, ROVER, 2800.0, 2500.0)]), &mut Vec::new(), false);
        assert!(out.hunts.is_empty(), "{:?}", out.hunts);
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { unit, target, .. } if unit.0 == 10 && target.0 == 99)), "{:?}", out.commands);
        assert_eq!(lane.hunting(UnitId(10)), Some(UnitId(99)));
    }
}
