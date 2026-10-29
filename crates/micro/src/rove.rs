//! H-MICRO-ROVE: a fast soldier (a Rover, a Tick) the host hands over whole. Ten times a second, in this order:
//! it never stands inside the reach of anything that can shoot it (plus a margin: what a mobile shooter covers in
//! 1.5 s, 60 for a building), stepping to the least threatened ground nearby when it is or is about to be; it kills
//! what it finds unguarded within 1,000 (a lone constructor first, then an extractor, a radar, anything else unarmed
//! it kills within 45 s), when nothing armed covers the target and the way there is clear; and otherwise it drives to
//! look at the place we know least (the spots in his start box and his base never seen or not seen for a minute,
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

use bot_protocol::{Command, EnemyUnit, OwnUnit, UnitDefId, UnitId, Vec3};

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

/// A place a rover may look at: its name in the host's words, where it is, the frame it was last within sight of
/// anything of ours (`None`: never), and whether it is his (in his start box, or his base).
#[derive(Clone, Debug, PartialEq)]
pub struct RoveGoal {
    pub name: String,
    pub at: Vec3,
    pub seen: Option<i32>,
    pub theirs: bool,
}

/// What a rover did that the host should hear about, each once.
#[derive(Clone, Debug, PartialEq)]
pub enum RoveWhat {
    /// An enemy unit came within its sight for the first time any rover saw it.
    Found { enemy: UnitId, def: UnitDefId, at: Vec3 },
    /// It started attacking this unguarded unit.
    Attacks { target: UnitId, def: UnitDefId, at: Vec3 },
    /// It stepped away from this shooter (`None`: a radar contact), standing here.
    Evades { from: Option<UnitDefId>, at: Vec3 },
    /// It gave up a place, and why.
    GivesUp { goal: String, why: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoveEvent {
    pub rover: UnitId,
    pub what: RoveWhat,
}

/// What a rover is doing this tick, for the host's picture.
#[derive(Clone, Debug, PartialEq)]
pub struct RoveReport {
    /// "evading", "attacking", "looking" or "waiting" (every way to a place worth a look passes a reach).
    pub doing: &'static str,
    /// The place it is going to look at, and where that is.
    pub goal: Option<(String, Vec3)>,
    pub target: Option<UnitId>,
    /// Evasions since it began roving.
    pub evasions: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Order {
    Move(Vec3),
    Attack(UnitId),
    Stop,
}

impl Order {
    fn same(self, other: Order) -> bool {
        match (self, other) {
            (Order::Move(a), Order::Move(b)) => a.dist2d(b) < SAME_MOVE,
            (Order::Attack(a), Order::Attack(b)) => a == b,
            (Order::Stop, Order::Stop) => true,
            _ => false,
        }
    }
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
    /// The cell it is stepping to, and when it was sent there.
    evading: Option<(Vec3, i32)>,
    sent: Option<(Order, i32)>,
    /// Places it will not take again until the frame.
    blocked: Vec<(String, i32)>,
    health: f32,
    evasions: u32,
}

/// How far beyond its reach a shooter is kept.
fn margin(s: &Source) -> f32 {
    if s.mobile { (s.speed * MARGIN_SECONDS).max(MARGIN_MIN) } else { MARGIN_STATIC }
}

/// A shooter's reach with its margin: its D-gun counts.
fn keep_off(s: &Source) -> f32 {
    s.reach.max(s.dgun) + margin(s)
}

/// How deep `p` stands inside the kept-off distances widened by `slack`, summed over the shooters. An evasion
/// prices its cells with a slack of its step: with none, every cell beyond the margin tied, the tie went to the one
/// nearest its goal or home, and the rover slid round the margin's rim while a Pawn closed on it (a chase test at
/// 189 from a Pawn of reach 180).
fn depth(danger: &[&Source], p: Vec3, slack: f32) -> f32 {
    danger.iter().map(|s| (keep_off(s) + slack - s.pos.dist2d(p)).max(0.0)).sum()
}

/// Nothing that can shoot covers the way from `a` to `b`, `b` included.
fn clear_way(danger: &[&Source], a: Vec3, b: Vec3) -> bool {
    danger.iter().all(|s| to_segment(s.pos, a, b) >= keep_off(s))
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
    /// What this unit is doing as a rover, if it is one (H-MICRO-ROVE).
    pub fn roving(&self, unit: UnitId) -> Option<RoveReport> {
        let r = self.rovers.get(&unit)?;
        let doing = if r.evading.is_some() {
            "evading"
        } else if r.target.is_some() {
            "attacking"
        } else if r.goal.is_some() {
            "looking"
        } else {
            "waiting"
        };
        Some(RoveReport { doing, goal: r.goal.as_ref().map(|g| (g.name.clone(), g.look)), target: r.target, evasions: r.evasions })
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
            let mut rover = match self.rovers.remove(&unit.id) {
                Some(r) => r,
                None => {
                    fired.push("H-MICRO-ROVE");
                    self.counts.0 += 1;
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} roves", view.label(), unit.id.0);
                    }
                    Rover { goal: None, target: None, evading: None, sent: None, blocked: Vec::new(), health: unit.health, evasions: 0 }
                }
            };
            rover.blocked.retain(|(_, until)| *until > frame);
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

            // 1. Out of every reach. Where it stands, and where half a second of its velocity takes it.
            let next = Vec3 { x: unit.pos.x + unit.vel.x * LOOKAHEAD_FRAMES, y: 0.0, z: unit.pos.z + unit.vel.z * LOOKAHEAD_FRAMES };
            let threat = danger.iter().copied().filter(|s| s.pos.dist2d(unit.pos) < keep_off(s) || s.pos.dist2d(next) < keep_off(s)).min_by(|a, b| (a.pos.dist2d(unit.pos) - keep_off(a)).total_cmp(&(b.pos.dist2d(unit.pos) - keep_off(b))));
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
                if fresh {
                    rover.evasions += 1;
                    rover.target = None;
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
                // Only a move is sent again for standing still: an attacker stands to shoot, a stopped rover stands.
                let stale = matches!(order, Order::Move(_)) && still && rover.sent.is_none_or(|(_, at)| frame - at >= RESEND_FRAMES);
                if !same || after_evading || stale {
                    rover.sent = Some((order, frame));
                    lane.counts.1 += 1;
                    commands.push(match order {
                        Order::Move(to) => Command::Move { unit: unit.id, to, queue: false },
                        Order::Attack(target) => Command::Attack { unit: unit.id, target, queue: false },
                        Order::Stop => Command::Stop { unit: unit.id },
                    });
                }
            };

            // 2. Kill what is unguarded near it.
            let undefended = |p: Vec3| danger.iter().all(|s| s.pos.dist2d(p) >= keep_off(s));
            let attackable = |e: &EnemyUnit| -> Option<u8> {
                let d = view.def(e.def?)?;
                let air = d.speed > 0.0 && d.move_class.is_none();
                let kill_seconds = e.health.max(1.0) / stats.dps.max(0.01);
                // Stunned for longer than the way there and the kill take: harmless until it is dead.
                let asleep = e.stunned > e.pos.dist2d(unit.pos) / stats.speed.max(1.0) + kill_seconds;
                // Unarmed, or a turret still being built: harmless and worth its whole metal.
                ((d.weapon_count == 0 || e.being_built || asleep) && !air && stats.dps > 0.0 && kill_seconds <= KILL_SECONDS && e.pos.dist2d(unit.pos) <= ATTACK_RADIUS && inside(e.pos) && undefended(e.pos) && clear_way(&danger, unit.pos, e.pos)).then(|| target_class(d))
            };
            rover.target = rover.target.filter(|t| enemies.iter().find(|e| e.id == *t).and_then(&attackable).is_some());
            if rover.target.is_none() {
                let others: HashSet<UnitId> = self.rovers.values().filter_map(|r| r.target).collect();
                let best = enemies.iter().filter_map(|e| Some((attackable(e)?, !others.contains(&e.id), e.pos.dist2d(unit.pos), e))).min_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2)));
                if let Some((_, _, _, e)) = best {
                    rover.target = Some(e.id);
                    events.push(RoveEvent { rover: unit.id, what: RoveWhat::Attacks { target: e.id, def: e.def.expect("attackable has a type"), at: e.pos } });
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} attacks {} at ({:.0}, {:.0})", view.label(), unit.id.0, e.id.0, e.pos.x, e.pos.z);
                    }
                }
            }
            if let Some(target) = rover.target {
                send(self, &mut rover, Order::Attack(target), commands);
                self.rovers.insert(unit.id, rover);
                continue;
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
                } else if !clear_way(&danger, unit.pos, goal.look) {
                    give_up(&mut rover, "its way passes the reach of something that can shoot it".to_string(), BLOCK_FRAMES, events);
                }
            }
            if rover.goal.is_none() {
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
                let best = goals
                    .iter()
                    .filter(|g| inside(g.at))
                    .filter(|g| !rover.blocked.iter().any(|(n, _)| *n == g.name))
                    // In its sight now: being looked at already.
                    .filter(|g| g.at.dist2d(unit.pos) > sight)
                    .map(|g| (g, look_at(g.at)))
                    .filter(|(_, look)| clear_way(&danger, unit.pos, *look))
                    .map(|(g, look)| {
                        // By the minute, nearest first; a place seen this last minute after every other, the oldest of
                        // those first (by the minute alone it tied with the place just looked at, the nearest).
                        let age = g.seen.map_or(0, |f| frame - f);
                        let recent = tier(g) == 2 && age < STALE_STEP;
                        let staleness = if tier(g) != 2 { 0 } else if recent { -age } else { -(age / STALE_STEP) };
                        ((taken.contains(g.name.as_str()), tier(g), recent, staleness), unit.pos.dist2d(look), g, look)
                    })
                    .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
                if let Some((_, _, g, look)) = best {
                    if debug {
                        eprintln!("{} f={frame} micro: {name}#{} goes to look at {} from ({:.0}, {:.0})", view.label(), unit.id.0, g.name, look.x, look.z);
                    }
                    rover.goal = Some(Goal { name: g.name.clone(), look, since: frame, arrived: None });
                }
            }
            match rover.goal.as_ref().map(|g| g.look) {
                Some(look) => send(self, &mut rover, Order::Move(look), commands),
                // Nowhere to go without passing a reach: it stands where it is safe.
                None if rover.sent.is_some_and(|(o, _)| !matches!(o, Order::Stop)) => send(self, &mut rover, Order::Stop, commands),
                None => {}
            }
            self.rovers.insert(unit.id, rover);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Stats;
    use super::*;
    use crate::{GroupKind, GroupOrder, HuntOrder};
    use bot_protocol::{MoveClass, MoveKind, Snapshot, Tick, UnitDefInfo};

    const ROVER: UnitDefId = UnitDefId(1);
    const PAWN: UnitDefId = UnitDefId(2);
    const TOWER: UnitDefId = UnitDefId(3);
    const EXTRACTOR: UnitDefId = UnitDefId(4);
    const CONSTRUCTOR: UnitDefId = UnitDefId(5);
    const BANSHEE: UnitDefId = UnitDefId(6);
    const THISTLE: UnitDefId = UnitDefId(7);

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn def(id: UnitDefId, name: &str, speed: f32, weapons: i32, build_speed: f32, extracts: f32) -> UnitDefInfo {
        UnitDefInfo {
            id,
            name: name.to_string(),
            metal_cost: 50.0,
            energy_cost: 0.0,
            speed,
            build_speed,
            build_time: 1.0,
            build_distance: 0.0,
            extracts_metal: extracts,
            metal_make: 0.0,
            energy_make: 0.0,
            energy_upkeep: 0.0,
            wind_cap: 0.0,
            tidal_make: 0.0,
            sonar_range: 0.0,
            hits_submerged: false,
            water_only: false,
            metal_storage: 0.0,
            energy_storage: 0.0,
            radar_range: 0.0,
            converter: None,
            weapon_count: weapons,
            build_options: Vec::new(),
            move_class: (speed > 0.0).then_some(MoveClass { kind: MoveKind::Tank, max_slope: 0.5, depth: 20.0, slope_mod: 0.0 }),
            footprint: (2, 2),
            death_blast: None,
            self_destruct_blast: None,
            self_destruct_seconds: 0.0,
            reach: 0.0,
            reload: 0.0,
            blast_radius: 0.0,
            stockpile_seconds: 0.0,
            stockpile_energy: 0.0,
            coverage: 0.0,
            sight: 0.0,
            jam_range: 0.0,
            stealth: false,
            cloak: None,
            shield: None,
            intercepted_by: 0,
            health: 0.0,
            reclaim_speed: 0.0,
            reclaimable: false,
            stun_taken: 1.0,
            stun: None,
        }
    }

    struct Host {
        defs: Vec<UnitDefInfo>,
        buildings: Vec<(UnitId, UnitDefId, Vec3)>,
        goals: Vec<RoveGoal>,
    }

    impl Host {
        fn new(goals: Vec<RoveGoal>) -> Host {
            let defs = vec![
                def(ROVER, "armfav", 168.0, 1, 0.0, 0.0),
                def(PAWN, "armpw", 87.0, 1, 0.0, 0.0),
                def(TOWER, "armllt", 0.0, 1, 0.0, 0.0),
                def(EXTRACTOR, "armmex", 0.0, 0, 0.0, 0.002),
                def(CONSTRUCTOR, "armck", 36.0, 0, 80.0, 0.0),
                UnitDefInfo { move_class: None, ..def(BANSHEE, "armkam", 185.0, 1, 0.0, 0.0) },
                def(THISTLE, "corrl", 0.0, 1, 0.0, 0.0),
            ];
            Host { defs, buildings: Vec::new(), goals }
        }
    }

    impl View for Host {
        fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
            self.defs.iter().find(|d| d.id == def)
        }
        fn stats(&self, def: UnitDefId) -> Option<Stats> {
            Some(match def {
                ROVER => Stats { reach: 180.0, dps: 40.0, speed: 168.0, health: 105.0, dgun: 0.0, sight: 635.0, area: 0.0, ..Stats::default() },
                PAWN => Stats { reach: 180.0, dps: 60.0, speed: 87.0, health: 370.0, dgun: 0.0, sight: 450.0, area: 0.0, ..Stats::default() },
                TOWER => Stats { reach: 430.0, dps: 80.0, speed: 0.0, health: 700.0, dgun: 0.0, sight: 500.0, area: 0.0, air_reach: 430.0, air_dps: 10.7 },
                BANSHEE => Stats { reach: 350.0, dps: 38.6, speed: 185.0, health: 560.0, sight: 520.0, ..Stats::default() },
                THISTLE => Stats { health: 335.0, sight: 500.0, air_reach: 765.0, air_dps: 67.6, ..Stats::default() },
                EXTRACTOR => Stats { health: 300.0, ..Stats::default() },
                CONSTRUCTOR => Stats { speed: 36.0, health: 760.0, ..Stats::default() },
                _ => return None,
            })
        }
        fn grid_spec(&self) -> (f32, usize, usize) {
            (16.0, 256, 256)
        }
        fn terrain(&self) -> Option<&bot_protocol::Terrain> {
            None
        }
        fn home(&self) -> Vec3 {
            at(200.0, 200.0)
        }
        fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
            self.buildings.clone()
        }
        fn building_at(&self, id: UnitId) -> Option<Vec3> {
            self.buildings.iter().find(|b| b.0 == id).map(|b| b.2)
        }
        fn enemy_known(&self, _id: UnitId) -> bool {
            true
        }
        fn blip_def(&self) -> Option<UnitDefId> {
            None
        }
        fn enabled(&self, _rule: &str) -> bool {
            true
        }
        fn mine(&self, _unit: UnitId) -> bool {
            true
        }
        fn label(&self) -> String {
            "[test]".to_string()
        }
        fn rove_goals(&self, _def: UnitDefId) -> Vec<RoveGoal> {
            self.goals.clone()
        }
    }

    fn goal(name: &str, x: f32, z: f32, seen: Option<i32>, theirs: bool) -> RoveGoal {
        RoveGoal { name: name.to_string(), at: at(x, z), seen, theirs }
    }

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
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { unit, target, .. } if unit.0 == 10 && target.0 == 50)), "free, it attacks the extractor beside it: {:?}", out.commands);
        let mut confined = Lane::default();
        let within = Some((at(2000.0, 2000.0), at(4000.0, 4000.0)));
        confined.set_group("scouts", vec![UnitId(10)], GroupOrder::new(GroupKind::Rove { within }, Vec::new()));
        let host = Host::new(goals);
        let out = confined.tick(&host, &tick(30, vec![rover(10, 1500.0, 1500.0)], vec![extractor]), &mut Vec::new(), false);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "confined, the extractor outside is left: {:?}", out.commands);
        let report = confined.roving(UnitId(10)).expect("roving");
        assert_eq!((report.doing, report.goal.map(|(name, _)| name)), ("looking", Some("spot_2".to_string())), "the place inside, not the nearer one outside");
    }

    /// A stunned unit shoots nothing until it wakes: a rover inside a stunned turret's reach stays, attacks it when
    /// it stays stunned longer than the way there and the kill, and leaves it alone when it wakes sooner; a stunned
    /// rover of ours is sent nothing.
    #[test]
    fn a_stunned_turret_is_no_threat_and_a_target_while_it_sleeps() {
        let tower = |stunned: f32| EnemyUnit { stunned, ..enemy(60, TOWER, 1300.0, 1000.0) };
        let attacks = |out: &crate::Output| out.commands.iter().any(|c| matches!(c, Command::Attack { unit, target, .. } if unit.0 == 10 && target.0 == 60));
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
        let attacks = |out: &crate::Output| out.commands.iter().find_map(|c| match c { Command::Attack { unit, target, .. } if unit.0 == 10 => Some(target.0), _ => None });
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

    /// The companies design (`docs/design/2026-10-25-companies.md` §2): forty-eight Pawns declared at a cap of 24
    /// march as two companies, each on its own way and formed on its own; the state reads the army at the top and the
    /// companies under it. The cap lifted on the march (companies-1: the player's `company: 0` left the companies
    /// standing split) makes them one company.
    #[test]
    fn a_capped_group_marches_as_companies() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let mut body: Vec<OwnUnit> = (0..48).map(|i| { let mut u = rover(100 + i, 1000.0 - (i / 6) as f32 * 96.0, 1000.0 + (i % 6) as f32 * 64.0); u.def = PAWN; u }).collect();
        let ids: Vec<UnitId> = body.iter().map(|u| u.id).collect();
        let goal = at(6000.0, 1160.0);
        lane.set_group("a", ids.clone(), GroupOrder::new(GroupKind::Go, vec![goal]).with_cap(24));
        let mut sent: HashMap<UnitId, Vec3> = HashMap::new();
        let mut log = Vec::new();
        for step in 0..200 {
            let frame = 30 + 3 * step;
            let out = lane.tick(&host, &tick(frame, body.clone(), Vec::new()), &mut Vec::new(), false);
            for c in &out.commands {
                if let Command::Move { unit, to, .. } = c {
                    sent.insert(*unit, *to);
                }
            }
            // Each unit walks 87 a second toward its last order.
            for u in body.iter_mut() {
                if let Some(to) = sent.get(&u.id) {
                    let d = u.pos.dist2d(*to);
                    let step = (87.0f32 * 0.1).min(d);
                    if d > 0.1 {
                        u.pos = Vec3 { x: u.pos.x + (to.x - u.pos.x) / d * step, y: 0.0, z: u.pos.z + (to.z - u.pos.z) / d * step };
                        u.vel = at((to.x - u.pos.x) / d * 2.9, (to.z - u.pos.z) / d * 2.9);
                    }
                }
            }
            if step % 10 == 9 {
                let s = lane.group_state("a").expect("a state");
                log.push(format!(
                    "{}s anchor ({:.0},{:.0}) pace {:.0} formed {} of 48 arrived {} | {}",
                    (frame - 30) / 30,
                    s.anchor.x,
                    s.anchor.z,
                    s.pace,
                    s.formed.len(),
                    s.arrived,
                    s.companies.iter().map(|c| format!("{} {} at ({:.0},{:.0}) formed {}/{}", c.role.word(), c.members, c.anchor.x, c.anchor.z, c.formed, c.members)).collect::<Vec<_>>().join("; ")
                ));
            }
        }
        let text = log.join("\n");
        let s = lane.group_state("a").expect("a state");
        assert_eq!(s.companies.len(), 2, "{text}");
        assert!(s.companies.iter().all(|c| c.role == crate::Role::Marching && c.members == 24), "{text}");
        assert!(s.companies.iter().all(|c| c.formed >= 20), "each company formed on its own: {text}");
        lane.set_group("a", ids.clone(), GroupOrder::new(GroupKind::Go, vec![goal]));
        lane.tick(&host, &tick(30 + 3 * 200, body.clone(), Vec::new()), &mut Vec::new(), false);
        let lifted = lane.group_state("a").expect("a state");
        assert_eq!((lifted.companies.len(), lifted.companies[0].members), (1, 48), "the cap lifted: one company");
    }

    fn rover(id: i32, x: f32, z: f32) -> OwnUnit {
        OwnUnit { id: UnitId(id), def: ROVER, pos: at(x, z), vel: at(0.0, 0.0), health: 105.0, max_health: 105.0, being_built: false, idle: false, reload_frame: 0, facing: 0, stockpile: 0, stockpile_queued: 0, priority_low: false, build_progress: 1.0, stunned: 0.0 }
    }

    fn enemy(id: i32, def: UnitDefId, x: f32, z: f32) -> EnemyUnit {
        EnemyUnit { id: UnitId(id), def: Some(def), pos: at(x, z), vel: at(0.0, 0.0), health: 300.0, team: Some(1), being_built: false, stunned: 0.0 }
    }

    fn tick(frame: i32, own: Vec<OwnUnit>, enemies: Vec<EnemyUnit>) -> Tick {
        Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { own_units: own, enemies, ..Snapshot::default() } }
    }

    /// A lane with these units declared as one roving group (design D15).
    fn roving_lane(ids: &[i32]) -> Lane {
        let mut lane = Lane::default();
        lane.set_group("scouts", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Rove { within: None }, Vec::new()));
        lane
    }

    fn moves_of(commands: &[Command], unit: i32) -> Vec<Vec3> {
        commands.iter().filter_map(|c| match c { Command::Move { unit: u, to, .. } if u.0 == unit => Some(*to), _ => None }).collect()
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

    /// Contact holds the ranks (`docs/design/2026-10-28-contact-cleanup.md` §5): a group advancing to a station does
    /// not stop for a Pawn one file can reach, halts with its slots standing once every member has it in reach (each
    /// still ordered to its slot: a Move, never a Fight or an Attack), and walks on to its station when the Pawn has
    /// run, never after it (player-21 5:41-6:46: six Rovers formed on a Tick under the old contact and trailed it
    /// 4,700 elmos into his base).
    #[test]
    fn a_group_halts_when_all_can_reach_and_never_follows_what_runs() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let ids = [10, 11, 12, 13, 14, 15];
        let station = at(1000.0, 3600.0);
        lane.set_group("a", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::FightTo, vec![station]));
        let rovers = |z: f32| ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 40.0 * (i % 3) as f32, z + 40.0 * (i / 3) as f32)).collect::<Vec<_>>();
        let orders_of = |commands: &[Command], unit: i32| commands.iter().filter_map(|c| match c { Command::Move { unit: u, to, .. } if u.0 == unit => Some(*to), _ => None }).collect::<Vec<Vec3>>();
        // A Pawn 500 east, inside the old 700 horizon but out of every Rover's reach (180): the march goes on south.
        let mut z = 1000.0;
        lane.tick(&host, &tick(30, rovers(z), vec![enemy(50, PAWN, 1540.0, z)]), &mut Vec::new(), false);
        let out = lane.tick(&host, &tick(33, rovers(z), vec![enemy(50, PAWN, 1540.0, z)]), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, None);
        assert!(state.pace > 0.0, "not halted for a Pawn out of reach: {:?}", state.pace);
        assert!(orders_of(&out.commands, 10).iter().all(|p| p.z > z && (p.x - 1040.0).abs() < 200.0), "the orders lead south toward the station: {:?}", orders_of(&out.commands, 10));
        // The Pawn closes to 100 east of the near file: two files of three have it in reach, the far file (180) not;
        // the block walks on, in contact.
        z += 30.0;
        lane.tick(&host, &tick(36, rovers(z), vec![enemy(50, PAWN, 1180.0, z)]), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, Some(UnitId(50)));
        assert!(state.pace > 0.0, "not halted while a file is out of reach");
        // The Pawn at 60 east of the near file: every member has it in reach; the group halts, the slots stand, and
        // every member is still ordered to its slot (a Move).
        lane.tick(&host, &tick(39, rovers(z), vec![enemy(50, PAWN, 1140.0, z)]), &mut Vec::new(), false);
        let anchor = lane.group_state("a").expect("a state").anchor;
        let out = lane.tick(&host, &tick(69, rovers(z), vec![enemy(50, PAWN, 1140.0, z)]), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, Some(UnitId(50)));
        assert_eq!(state.pace, 0.0, "halted while every member has the Pawn in reach");
        assert!(state.anchor.dist2d(anchor) < 1.0, "the anchor stands: {:?} from {anchor:?}", state.anchor);
        assert!(out.commands.iter().all(|c| matches!(c, Command::Move { .. })), "slots as Moves and nothing else: {:?}", out.commands);
        // The Pawn runs east out of every reach: the march resumes toward the station from where the block stood.
        let mut last = Vec::new();
        for f in 1..=4 {
            let out = lane.tick(&host, &tick(69 + 30 * f, rovers(z), vec![enemy(50, PAWN, 1140.0 + 300.0 * f as f32, z)]), &mut Vec::new(), false);
            last.extend(orders_of(&out.commands, 10));
        }
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, None);
        assert!(state.pace > 0.0 && state.heading.1 > 0.9, "walking on south: pace {} heading {:?}", state.pace, state.heading);
        assert!(!last.is_empty() && last.iter().all(|p| p.x < 1300.0 && p.z > z), "the orders lead to the station, not the Pawn: {last:?}");
    }

    /// The close (`docs/design/2026-10-28-contact-cleanup.md` §5): an advance whose route ends short of a turret's
    /// reach walks on at it from the end, at the pace, and halts when every member has it in reach or the front is
    /// inside half its reach; a hold at the same end stands.
    #[test]
    fn an_advance_at_its_end_closes_on_a_turret_until_all_are_in_reach_and_a_hold_stands() {
        let host = Host::new(Vec::new());
        let ids = [10, 11, 12, 13, 14, 15];
        let spawn = || ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 40.0 * (i % 3) as f32, 1000.0 + 40.0 * (i / 3) as f32)).collect::<Vec<OwnUnit>>();
        // The route ends 300 short of the tower; a Rover reaches 180.
        let end = at(1040.0, 1500.0);
        let tower = enemy(50, TOWER, 1040.0, 1800.0);
        // Each tick every unit walks toward the last point it was sent to at its speed, to within the brake zone's
        // trail (the engine's part, as the group tests do it).
        let run = |lane: &mut Lane, body: &mut Vec<OwnUnit>| -> Option<Vec3> {
            let mut goals: HashMap<UnitId, Vec3> = HashMap::new();
            let mut frame = 30;
            while frame < 30 * 90 {
                let out = lane.tick(&host, &tick(frame, body.clone(), vec![tower.clone()]), &mut Vec::new(), false);
                assert!(out.commands.iter().all(|c| matches!(c, Command::Move { .. } | Command::MoveState { .. })), "Moves only: {:?}", out.commands);
                for c in &out.commands {
                    if let Command::Move { unit, to, .. } = c {
                        goals.insert(*unit, *to);
                    }
                }
                for u in body.iter_mut() {
                    let Some(to) = goals.get(&u.id) else { continue };
                    let d = u.pos.dist2d(*to);
                    let step = 16.8f32.min((d - 20.0).max(0.0));
                    u.vel = at(0.0, 0.0);
                    if d > 0.0 && step > 0.0 {
                        let (dx, dz) = ((to.x - u.pos.x) / d * step, (to.z - u.pos.z) / d * step);
                        u.pos = at(u.pos.x + dx, u.pos.z + dz);
                        u.vel = at(dx / 3.0, dz / 3.0);
                    }
                }
                let state = lane.group_state("a").expect("a state");
                if state.anchor.z > end.z + 50.0 && state.pace == 0.0 {
                    return Some(state.anchor);
                }
                frame += 3;
            }
            None
        };
        let mut lane = Lane::default();
        let mut body = spawn();
        lane.set_group("a", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::FightTo, vec![end]));
        let anchor = run(&mut lane, &mut body).expect("the advance closed past its end and halted");
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, Some(UnitId(50)));
        assert!(anchor.z < end.z + crate::group::CLOSE_LEASH, "within the leash: {anchor:?}");
        let distances: Vec<f32> = body.iter().map(|u| u.pos.dist2d(tower.pos).round()).collect();
        assert!(distances.iter().all(|d| *d < 200.0) || distances.iter().any(|d| *d < 110.0), "all in reach, or the front inside half: {distances:?}");
        assert!(distances.iter().any(|d| *d < 200.0), "somebody reaches the tower: {distances:?}");
        // A hold at the same end stands there with the tower out of reach.
        let mut lane = Lane::default();
        let mut body = spawn();
        lane.set_group("a", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Hold, vec![end]));
        assert!(run(&mut lane, &mut body).is_none(), "a hold closes on nothing");
        let state = lane.group_state("a").expect("a state");
        assert!(state.arrived && state.engaged.is_none(), "{state:?}");
    }

    /// §5.2: the halt is all in reach and nothing else. A straggler of his inside half a corner member's reach, with
    /// the far corner out of every reach, halts nothing (until 2026-10-29 it did: advance-2 18:23-18:37, a company
    /// held halted with 5 of 11 in reach while a Maverick beyond the rear's reach killed six); and the front's guard
    /// pins the anchor at half the shortest reach from the nearest armed enemy, on the way, while the rear has nothing
    /// in reach.
    #[test]
    fn a_fighting_block_walks_on_with_a_straggler_inside_half_reach_and_pins_at_the_stop() {
        let host = Host::new(Vec::new());
        let ids: Vec<i32> = (10..19).collect();
        let members = ids.iter().map(|id| UnitId(*id)).collect::<Vec<_>>();
        // Three files 100 apart, three ranks 100 apart; the station far south.
        let block = || ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 100.0 * (i % 3) as f32, 1000.0 + 100.0 * (i / 3) as f32)).collect::<Vec<_>>();
        let mut lane = Lane::default();
        lane.set_group("a", members.clone(), GroupOrder::new(GroupKind::FightTo, vec![at(1100.0, 3600.0)]));
        lane.tick(&host, &tick(30, block(), Vec::new()), &mut Vec::new(), false);
        // A Pawn 70 beyond the north-east corner: inside half of that member's reach (90), out of the far corner's
        // (sqrt(270^2 + 200^2) = 336 against 180). Not halted, not pinned: the block walks on, in contact.
        let pawn = enemy(50, PAWN, 1270.0, 1000.0);
        lane.tick(&host, &tick(33, block(), vec![pawn.clone()]), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert_eq!(state.engaged, Some(UnitId(50)));
        assert!(state.pace > 0.0, "a straggler inside half one member's reach halts nothing: {state:?}");
        // A tower on the route, ahead of the block: the anchor walks to half the reach from it and stands there with
        // the rear out of reach (the old halt would have stopped the block there too; the new one never does, the
        // guard does). The stand-in engine walks every unit to its last Move goal.
        let tower = enemy(51, TOWER, 1100.0, 1700.0);
        let mut lane = Lane::default();
        let mut body = block();
        lane.set_group("a", members, GroupOrder::new(GroupKind::FightTo, vec![at(1100.0, 3600.0)]));
        let mut goals: HashMap<UnitId, Vec3> = HashMap::new();
        let mut frame = 30;
        let mut pinned_at = None;
        while frame < 30 * 60 {
            let out = lane.tick(&host, &tick(frame, body.clone(), vec![tower.clone()]), &mut Vec::new(), false);
            for c in &out.commands {
                if let Command::Move { unit, to, .. } = c {
                    goals.insert(*unit, *to);
                }
            }
            for u in body.iter_mut() {
                let Some(to) = goals.get(&u.id) else { continue };
                let d = u.pos.dist2d(*to);
                let step = 16.8f32.min((d - 20.0).max(0.0));
                u.vel = at(0.0, 0.0);
                if d > 0.0 && step > 0.0 {
                    let (dx, dz) = ((to.x - u.pos.x) / d * step, (to.z - u.pos.z) / d * step);
                    u.pos = at(u.pos.x + dx, u.pos.z + dz);
                    u.vel = at(dx / 3.0, dz / 3.0);
                }
            }
            let state = lane.group_state("a").expect("a state");
            if state.pace == 0.0 && state.anchor.z > 1300.0 {
                pinned_at = Some(state.anchor);
                break;
            }
            frame += 3;
        }
        let anchor = pinned_at.expect("the anchor stood short of the tower");
        let d = anchor.dist2d(tower.pos);
        assert!((70.0..=112.0).contains(&d), "pinned about half the reach (90) from the tower: {d:.0}");
        let in_reach = body.iter().filter(|u| u.pos.dist2d(tower.pos) <= 180.0).count();
        assert!(in_reach < body.len(), "the rear has nothing in reach, and the block stands at the stop all the same: {in_reach} of {}", body.len());
    }

    /// §5.2: an attack on a party ends each formation's way at the nearest member of the party in its sight, and the
    /// host's centre (re-sent every tick as the sightings move it) only while none is seen; the host's moving route
    /// re-aims nothing. (advance-2 18:25-18:41: a company walked 17 s at a centroid pulled by units it could not
    /// reach, with nothing in reach.)
    #[test]
    fn an_attack_on_a_party_aims_each_formation_at_its_nearest_member() {
        let host = Host::new(Vec::new());
        let ids = [10, 11, 12, 13, 14, 15];
        let members = ids.iter().map(|id| UnitId(*id)).collect::<Vec<_>>();
        let block = || ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 40.0 * (i % 3) as f32, 1000.0 + 40.0 * (i / 3) as f32)).collect::<Vec<_>>();
        let order = |centre: Vec3| GroupOrder { party: vec![UnitId(50), UnitId(51)], ..GroupOrder::new(GroupKind::FightTo, vec![centre]) };
        let mut lane = Lane::default();
        // The party's centre south; its nearest member east, its other far south.
        lane.set_group("a", members.clone(), order(at(1040.0, 2200.0)));
        let seen = vec![enemy(50, PAWN, 1600.0, 1100.0), enemy(51, PAWN, 1040.0, 2600.0)];
        lane.tick(&host, &tick(30, block(), seen.clone()), &mut Vec::new(), false);
        lane.tick(&host, &tick(33, block(), seen.clone()), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert!(state.heading.0 > 0.8, "facing the nearest member, east, not the centre: {:?}", state.heading);
        // The host moves the centre as the sightings move it: the way still ends at the nearest member.
        lane.set_group("a", members.clone(), order(at(1300.0, 2300.0)));
        lane.tick(&host, &tick(36, block(), seen.clone()), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert!(state.heading.0 > 0.8, "the host's moving centre re-aims nothing: {:?}", state.heading);
        // Sticky: the other member a little nearer than the one the way is on changes nothing; much nearer, it is
        // the end (`PARTY_SWITCH`).
        let a_little = vec![enemy(50, PAWN, 1600.0, 1100.0), enemy(51, PAWN, 1040.0, 1500.0)];
        lane.tick(&host, &tick(39, block(), a_little.clone()), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert!(state.heading.0 > 0.8, "a member nearer by under the margin changes nothing: {:?}", state.heading);
        let much = vec![enemy(50, PAWN, 1600.0, 1100.0), enemy(51, PAWN, 1040.0, 1250.0)];
        lane.tick(&host, &tick(42, block(), much.clone()), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert!(state.heading.1 > 0.8, "a member nearer by the margin is the end: {:?}", state.heading);
        // Nobody of the party in sight: the way ends at the host's centre.
        lane.set_group("a", members, order(at(1600.0, 2200.0)));
        lane.tick(&host, &tick(45, block(), Vec::new()), &mut Vec::new(), false);
        lane.tick(&host, &tick(48, block(), Vec::new()), &mut Vec::new(), false);
        let state = lane.group_state("a").expect("a state");
        assert!(state.heading.0 > 0.3 && state.heading.1 > 0.8, "the centre, south-south-east, while none is seen: {:?}", state.heading);
    }

    /// H-MICRO-FORM-HOLDPOS: a formation group's soldiers are put at hold position once, so nobody idle in its slot
    /// chases; a member that leaves the group, or a group re-declared as a rove, is at manoeuvre again, once.
    #[test]
    fn a_formation_groups_soldiers_hold_position_and_leave_it_with_the_group() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let ids = [10, 11, 12];
        let body = || ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 64.0 * i as f32, 1000.0)).collect::<Vec<_>>();
        let states = |commands: &[Command]| {
            let mut v: Vec<(i32, i32)> = commands.iter().filter_map(|c| match c { Command::MoveState { unit, state } => Some((unit.0, *state)), _ => None }).collect();
            v.sort();
            v
        };
        let members = |ids: &[i32]| ids.iter().map(|id| UnitId(*id)).collect::<Vec<_>>();
        lane.set_group("a", members(&ids), GroupOrder::new(GroupKind::FightTo, vec![at(1064.0, 3000.0)]));
        let out = lane.tick(&host, &tick(30, body(), Vec::new()), &mut Vec::new(), false);
        assert_eq!(states(&out.commands), vec![(10, 0), (11, 0), (12, 0)]);
        let out = lane.tick(&host, &tick(33, body(), Vec::new()), &mut Vec::new(), false);
        assert!(states(&out.commands).is_empty(), "once: {:?}", states(&out.commands));
        lane.set_group("a", members(&[10, 11]), GroupOrder::new(GroupKind::FightTo, vec![at(1064.0, 3000.0)]));
        let out = lane.tick(&host, &tick(36, body(), Vec::new()), &mut Vec::new(), false);
        assert_eq!(states(&out.commands), vec![(12, 1)], "the one that left manoeuvres again");
        lane.set_group("a", members(&[10, 11]), GroupOrder::new(GroupKind::Rove { within: None }, Vec::new()));
        let out = lane.tick(&host, &tick(39, body(), Vec::new()), &mut Vec::new(), false);
        assert_eq!(states(&out.commands), vec![(10, 1), (11, 1)], "rovers manoeuvre");
    }

    /// A formation re-declared as a rove loses its group and state (no reader sees a formed hold while the members
    /// roam), and declared as a formation again it starts afresh on the body: every member is ordered to a slot
    /// about where it stands on the first tick (migration-2-1: a hold's anchor kept through a 27 s rove had the
    /// advance after it walk alone 4,000 from the members).
    #[test]
    fn a_rove_drops_the_formation_and_a_formation_after_it_starts_on_the_body() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let ids = [10, 11, 12];
        let members: Vec<UnitId> = ids.iter().map(|id| UnitId(*id)).collect();
        let body = |x: f32| ids.iter().enumerate().map(|(i, id)| rover(*id, x + 64.0 * i as f32, 1000.0)).collect::<Vec<_>>();
        lane.set_group("a", members.clone(), GroupOrder::new(GroupKind::Hold, vec![at(1064.0, 1000.0)]));
        for f in 1..=10 {
            lane.tick(&host, &tick(30 * f, body(1000.0), Vec::new()), &mut Vec::new(), false);
        }
        assert!(lane.group_state("a").is_some_and(|s| s.arrived), "a hold formed: {:?}", lane.group_state("a"));
        lane.set_group("a", members.clone(), GroupOrder::new(GroupKind::Rove { within: None }, Vec::new()));
        lane.tick(&host, &tick(330, body(1000.0), Vec::new()), &mut Vec::new(), false);
        assert!(lane.group_state("a").is_none(), "a roving group has no formation state");
        // The rovers roam 3,000 east; told to advance north from there, the slots are about them, not at the old hold.
        lane.set_group("a", members, GroupOrder::new(GroupKind::FightTo, vec![at(4064.0, -3000.0)]));
        let out = lane.tick(&host, &tick(360, body(4000.0), Vec::new()), &mut Vec::new(), false);
        let to: Vec<Vec3> = out.commands.iter().filter_map(|c| match c { Command::Move { to, .. } => Some(*to), _ => None }).collect();
        assert_eq!(to.len(), 3, "every member ordered: {:?}", out.commands);
        assert!(to.iter().all(|p| p.dist2d(at(4064.0, 1000.0)) < 500.0), "the orders about the body (the lead ahead at most): {to:?}");
        let state = lane.group_state("a").expect("a state");
        assert!(state.anchor.dist2d(at(4064.0, 1000.0)) < 200.0, "the anchor on the body: {:?}", state.anchor);
        assert!(state.exceptions.is_empty(), "{:?}", state.exceptions);
    }

    /// A hunt that ends into a hold (the hands' way: a Stop to each hunter, then the group re-declared as a Hold about
    /// the hunters): the lane orders every former hunter to a slot on the hold's first tick, Stop or no Stop.
    #[test]
    fn a_hunt_ending_in_a_hold_orders_the_hunters_into_their_slots() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let members = vec![UnitId(10), UnitId(11)];
        let hunt = HuntOrder { quarry: UnitId(99), leash: 2000.0, lost_frames: crate::HUNT_LOST_FRAMES, since: 30 };
        lane.set_group("a", members.clone(), GroupOrder::new(GroupKind::Hunt(hunt), Vec::new()));
        let hunters = |x: f32| vec![rover(10, x, 1000.0), rover(11, x + 200.0, 1150.0)];
        let out = lane.tick(&host, &tick(30, hunters(2500.0), vec![enemy(99, ROVER, 2800.0, 1000.0)]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { unit, target, .. } if unit.0 == 10 && target.0 == 99)), "{:?}", out.commands);
        // The quarry is gone; the host (told by the lane's event, in the game) stops the hunters and declares the
        // hold about them.
        lane.tick(&host, &tick(60, hunters(2600.0), Vec::new()), &mut Vec::new(), false);
        let mut host_orders: Vec<Command> = members.iter().map(|id| Command::Stop { unit: *id }).collect();
        lane.note_standing_orders(&host_orders, 63, |_| true);
        lane.set_group("a", members.clone(), GroupOrder::new(GroupKind::Hold, vec![at(2700.0, 1075.0)]));
        let out = lane.tick(&host, &tick(63, hunters(2600.0), Vec::new()), &mut host_orders, false);
        let ordered: Vec<i32> = out.commands.iter().filter_map(|c| match c { Command::Move { unit, .. } => Some(unit.0), _ => None }).collect();
        assert_eq!(ordered.len(), 2, "every former hunter sent to a slot on the hold's first tick: {:?} (host orders left {:?})", out.commands, host_orders);
        let state = lane.group_state("a").expect("a hold's state");
        assert!(state.anchor.dist2d(at(2700.0, 1075.0)) < 300.0, "the anchor about the hunters: {:?}", state.anchor);
    }

    /// The group is the lane's whole: the host's per-unit orders to its members are dropped, and a unit outside the
    /// group keeps its order (player-22, 22:21: 110 stops for 55 units in one row, when the host's stop was doubled).
    #[test]
    fn the_hosts_orders_to_a_groups_members_are_dropped() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let ids = [10, 11, 12, 13, 14, 15];
        lane.set_group("a", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::FightTo, vec![at(1000.0, 3600.0)]));
        let mut rovers = ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 40.0 * (i % 3) as f32, 1000.0 + 40.0 * (i / 3) as f32)).collect::<Vec<_>>();
        rovers.push(rover(20, 3000.0, 3000.0));
        let mut host_orders: Vec<Command> = ids.iter().chain([20].iter()).map(|id| Command::Stop { unit: UnitId(*id) }).collect();
        lane.note_standing_orders(&host_orders, 30, |_| true);
        let out = lane.tick(&host, &tick(30, rovers, Vec::new()), &mut host_orders, false);
        let kept: Vec<i32> = host_orders.iter().filter_map(|c| match c { Command::Stop { unit } => Some(unit.0), _ => None }).collect();
        assert_eq!((host_orders.len(), kept), (1, vec![20]), "the members' stops dropped, the outsider's kept: {host_orders:?}");
        // A fighting group's slot orders are Moves too (§5).
        let moves: Vec<i32> = out.commands.iter().filter_map(|c| match c { Command::Move { unit, .. } => Some(unit.0), _ => None }).collect();
        assert_eq!(moves.len(), 6, "every member to its slot: {:?}", out.commands);
        assert!(!moves.contains(&20));
    }

    /// The group's march (`docs/design/2026-10-24-formation-drills.md` §2): every member is sent to its slot, an
    /// offset from the anchor that walks at the slowest type's speed times the fraction, again every `cadence`
    /// frames as the slot moves; a fast unit's slot is beside the slow ones, never the goal ahead of them; the state
    /// says who is formed and at what pace the group walks.
    #[test]
    fn a_groups_march_paces_the_fast_unit_beside_the_slow_ones() {
        let host = Host::new(Vec::new());
        let mut lane = Lane::default();
        let ids = [10, 11, 12, 13, 14, 15];
        // Five Pawns (87) and one Rover (168), together at x 1000; the goal 3,000 east.
        let mut body: Vec<OwnUnit> = ids.iter().enumerate().map(|(i, id)| rover(*id, 1000.0 + 40.0 * (i % 3) as f32, 1000.0 + 40.0 * (i / 3) as f32)).collect();
        for u in body.iter_mut().skip(1) {
            u.def = PAWN;
        }
        let goal = at(4000.0, 1000.0);
        lane.set_group("a", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Go, vec![goal]));
        let first_of = |commands: &[Command], id: i32| commands.iter().find_map(|c| match c { Command::Move { unit, to, queue: false } if unit.0 == id => Some(*to), _ => None });
        let out = lane.tick(&host, &tick(30, body.clone(), Vec::new()), &mut Vec::new(), false);
        let rover_to = first_of(&out.commands, 10).expect("the Rover is ordered");
        assert!(rover_to.x - 1000.0 < 300.0 && rover_to.dist2d(goal) > 2500.0, "the Rover's slot is by the body, not the goal: {rover_to:?}");
        // Six slot orders, and on the first tick each member put at hold position (H-MICRO-FORM-HOLDPOS).
        assert_eq!(out.commands.iter().filter(|c| !matches!(c, Command::MoveState { .. })).count(), 6);
        assert_eq!(out.commands.iter().filter(|c| matches!(c, Command::MoveState { state: 0, .. })).count(), 6);
        let state = lane.group_state("a").expect("a state").clone();
        assert!((state.full_pace - 87.0).abs() < 0.5, "the pace is the Pawns': {}", state.full_pace);
        // The block spawned at 40 spacing against a rank of six at 64: the two end units start out of their slots,
        // joiners with under two seconds to go, and they do not hold the anchor.
        assert_eq!(state.formed.len(), 4, "{:?}", state.exceptions);
        assert!(state.exceptions.iter().all(|(_, cause, s)| *cause == crate::Cause::Joining && *s < 2.0), "{:?}", state.exceptions);
        assert!(!state.arrived);
        // Three frames on the slots have walked 8.7 elmos: every member is sent again; two frames on, nobody.
        let out = lane.tick(&host, &tick(33, body.clone(), Vec::new()), &mut Vec::new(), false);
        assert_eq!(out.commands.len(), 6, "{:?}", out.commands);
        let again = first_of(&out.commands, 10).unwrap();
        assert!((again.x - rover_to.x - 8.7).abs() < 1.0, "the slot moved a tick's pace east: {again:?} from {rover_to:?}");
        let out = lane.tick(&host, &tick(35, body, Vec::new()), &mut Vec::new(), false);
        assert!(out.commands.is_empty(), "{:?}", out.commands);
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
        assert!(moves_of(&out.commands, 10).is_empty(), "no evasion from a tower that cannot shoot: {:?}", out.commands);
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { unit: UnitId(10), target: UnitId(60), .. })), "{:?}", out.commands);
    }

    #[test]
    fn a_rover_kills_the_unguarded_extractor_and_leaves_the_guarded_one() {
        let mut host = Host::new(vec![goal("spot_2", 3000.0, 3000.0, None, true)]);
        // The nearer extractor stands 200 from a tower.
        host.buildings.push((UnitId(60), TOWER, at(1500.0, 1000.0)));
        let mut lane = roving_lane(&[10]);
        let guarded = enemy(70, EXTRACTOR, 1300.0, 1000.0);
        let open = enemy(71, EXTRACTOR, 1000.0, 500.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![guarded, open]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { unit: UnitId(10), target: UnitId(71), .. })), "{:?}", out.commands);
        assert_eq!(lane.roving(UnitId(10)).unwrap().doing, "attacking");
        // A constructor walking about comes before an extractor.
        let mut lane = roving_lane(&[10]);
        let builder = enemy(72, CONSTRUCTOR, 700.0, 1000.0);
        let out = lane.tick(&host, &tick(30, vec![rover(10, 1000.0, 1000.0)], vec![enemy(71, EXTRACTOR, 1000.0, 800.0), builder]), &mut Vec::new(), false);
        assert!(out.commands.iter().any(|c| matches!(c, Command::Attack { target: UnitId(72), .. })), "{:?}", out.commands);
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
