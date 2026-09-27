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
//! reaches it (`lib.rs` `tick` drops them), so the hands' states cannot pull it off.
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
    pub(crate) fn rove(&mut self, view: &dyn View, rovers: &[&OwnUnit], enemies: &[EnemyUnit], sources: &[Source], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, events: &mut Vec<RoveEvent>, debug: bool) {
        let danger: Vec<&Source> = sources.iter().filter(|s| s.weight >= FAINT).collect();
        let mut goals_of: HashMap<UnitDefId, Vec<RoveGoal>> = HashMap::new();
        for unit in rovers {
            let stats = view.stats(unit.def).unwrap_or_default();
            let sight = if stats.sight > 0.0 { stats.sight } else { DEFAULT_SIGHT };
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
            let hit_unseen = threat.is_none() && unit.health < rover.health - 1.0;
            rover.health = unit.health;
            if let Some(s) = threat {
                // From anything as fast as it, toward home; else round the threat toward where it was going.
                let prefer = if s.mobile && s.speed >= stats.speed { view.home() } else { rover.goal.as_ref().map_or(view.home(), |g| g.look) };
                let cell = match self.grid.as_ref() {
                    Some(grid) => grid.lowest_within_by(unit.pos, STEP_RADIUS, view.passable(), prefer, &|c| depth(&danger, c, STEP_RADIUS)),
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
                    let far = view.snap(Vec3 { x: unit.pos.x + (cell.x - unit.pos.x) * STEP_REACH, y: 0.0, z: unit.pos.z + (cell.z - unit.pos.z) * STEP_REACH });
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
                // Unarmed, or a turret still being built: harmless and worth its whole metal.
                ((d.weapon_count == 0 || e.being_built) && !air && stats.dps > 0.0 && kill_seconds <= KILL_SECONDS && e.pos.dist2d(unit.pos) <= ATTACK_RADIUS && undefended(e.pos) && clear_way(&danger, unit.pos, e.pos)).then(|| target_class(d))
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
            let look_at = |at: Vec3| -> Vec3 {
                let (dx, dz) = (unit.pos.x - at.x, unit.pos.z - at.z);
                let d = dx.hypot(dz);
                let short = (sight * LOOK_SHARE).min(d);
                if d < 1.0 { at } else { view.snap(Vec3 { x: at.x + dx / d * short, y: 0.0, z: at.z + dz / d * short }) }
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
    use super::super::{Commitment, Stats};
    use super::*;
    use bot_protocol::{MoveClass, MoveKind, Snapshot, Tick, UnitDefInfo};

    const ROVER: UnitDefId = UnitDefId(1);
    const PAWN: UnitDefId = UnitDefId(2);
    const TOWER: UnitDefId = UnitDefId(3);
    const EXTRACTOR: UnitDefId = UnitDefId(4);
    const CONSTRUCTOR: UnitDefId = UnitDefId(5);

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
                ROVER => Stats { reach: 180.0, dps: 40.0, speed: 168.0, health: 105.0, dgun: 0.0, sight: 635.0 },
                PAWN => Stats { reach: 180.0, dps: 60.0, speed: 87.0, health: 370.0, dgun: 0.0, sight: 450.0 },
                TOWER => Stats { reach: 430.0, dps: 80.0, speed: 0.0, health: 700.0, dgun: 0.0, sight: 500.0 },
                EXTRACTOR => Stats { health: 300.0, ..Stats::default() },
                CONSTRUCTOR => Stats { speed: 36.0, health: 760.0, ..Stats::default() },
                _ => return None,
            })
        }
        fn grid_spec(&self) -> (f32, usize, usize) {
            (16.0, 256, 256)
        }
        fn passable(&self) -> Option<&[bool]> {
            None
        }
        fn snap(&self, pos: Vec3) -> Vec3 {
            pos
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

    fn rover(id: i32, x: f32, z: f32) -> OwnUnit {
        OwnUnit { id: UnitId(id), def: ROVER, pos: at(x, z), vel: at(0.0, 0.0), health: 105.0, max_health: 105.0, being_built: false, idle: false, reload_frame: 0, facing: 0 }
    }

    fn enemy(id: i32, def: UnitDefId, x: f32, z: f32) -> EnemyUnit {
        EnemyUnit { id: UnitId(id), def: Some(def), pos: at(x, z), vel: at(0.0, 0.0), health: 300.0, team: Some(1), being_built: false }
    }

    fn tick(frame: i32, own: Vec<OwnUnit>, enemies: Vec<EnemyUnit>) -> Tick {
        Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { own_units: own, enemies, ..Snapshot::default() } }
    }

    fn roving_lane(ids: &[i32]) -> Lane {
        let mut lane = Lane::default();
        lane.set_commitments(ids.iter().map(|id| (UnitId(*id), Commitment::Rove)).collect(), HashMap::new());
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
}
