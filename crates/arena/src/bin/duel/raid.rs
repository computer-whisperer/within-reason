//! The raid scenario (`duel --scenario raid`; `docs/design/2026-09-26-threat-response.md` §3): a picket body at a
//! station in the middle of a strip of four extractors 600 apart, a constructor at the far one, and two raiders (a
//! Tick and a Pawn by default) on a scripted route from beyond the far end: they hit the constructor and the
//! extractors in turn, run from any soldier of ours that comes near, and come back when it has gone. Three minutes.
//! Scored by extractors lost, seconds to the first raider killed, hunters lost, and the hunts' ends.
//!
//! The picket answers as the standing raider rule of the bot does (a raider at a structure of ours within 1,200 of
//! the station): the units faster than the raider, two at most, go after it. With the lane on they go as a hunt
//! (H-MICRO-HUNT: `micro::Commitment::Hunt`, an attack by id every tick, the lane reporting the end); with the lane
//! off they go as the hands went before (a fight-to-point at the raider's place every two seconds), judged by the
//! same ends (dead, out of sight six seconds, the 900 leash, a third of health) in the director. Either way a
//! hunter whose hunt ended walks back to its place at the station.

use std::collections::HashMap;

use bot_protocol::{Command, EnemyUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use micro::{Commitment, Hunt, HuntEvent};

const FPS: i32 = 30;
/// The buildings of the strip: the extractor and the constructor working at the far one.
pub const BUILDINGS: [&str; 2] = ["armmex", "armcv"];
/// Elmos between the extractors along the strip.
pub const STRIP_SPACING: f32 = 600.0;
/// Where the raiders appear: this far from the station, beyond the far extractor.
pub const RAIDER_START: f32 = 1400.0;
/// The station's distance west of the site's centre: half of (the raiders' start east of it minus the strip's west end).
const STATION_OFFSET: f32 = (RAIDER_START - 1.5 * STRIP_SPACING) / 2.0;
/// A raider this near a building of ours is at it (the standing rule's `at_structure`).
pub const AT_STRUCTURE: f32 = 250.0;
/// The standing raider rules reach this far from the station (`micro::RAIDER_REACH` in the bot).
pub const TRIGGER_REACH: f32 = 1200.0;
/// The hunt's leash from where the hunters stood when it began.
pub const LEASH: f32 = 900.0;
/// Hunters sent after one raider.
pub const HUNTERS_PER_QUARRY: usize = 2;
/// A raider runs from a soldier of ours this near, this far, and comes back this long after it is clear.
pub const FLEE_RANGE: f32 = 350.0;
pub const FLEE_STEP: f32 = 500.0;
pub const RESUME_AFTER: i32 = 3 * FPS;
/// The lane-off arm's fight-to-point cadence (the hands' engage re-issue before the hunt).
pub const REISSUE_OFF: i32 = 2 * FPS;
/// A quarry out of sight this long ends the lane-off arm's hunt (the lane's own constant is the same).
pub const LOST_FRAMES: i32 = 6 * FPS;
/// A hunter under this share of its health drops out (as in the lane).
pub const DROP_HEALTH: f32 = 1.0 / 3.0;
/// A raider is sent at a building from this far short of it, so its attack lands when the building is in sight.
const APPROACH: f32 = 120.0;

/// The raid's places, from the site's centre: the station, the extractors far to near along +x (the raiders'
/// route), the constructor beside the far one, the raiders' start points, and the pickets' places round the station.
pub struct Layout {
    pub station: Vec3,
    pub extractors: Vec<Vec3>,
    pub constructor: Vec3,
    pub raiders: Vec<Vec3>,
    pub pickets: Vec<Vec3>,
}

impl Layout {
    pub fn at(centre: Vec3, pickets: usize, raiders: usize) -> Layout {
        // The station sits west of the site's centre, so the strip and the raiders' approach share the site's length.
        let station = Vec3 { x: centre.x - STATION_OFFSET, ..centre };
        let p = |dx: f32, dz: f32| Vec3 { x: station.x + dx, y: station.y, z: station.z + dz };
        let extractors = vec![p(1.5 * STRIP_SPACING, 0.0), p(0.5 * STRIP_SPACING, 0.0), p(-0.5 * STRIP_SPACING, 0.0), p(-1.5 * STRIP_SPACING, 0.0)];
        let constructor = p(1.5 * STRIP_SPACING, 90.0);
        let raiders = (0..raiders).map(|i| p(RAIDER_START + 80.0 * i as f32, 120.0 * i as f32 - 60.0)).collect();
        // A block behind the strip's line, 64 apart, three to a rank.
        let pickets = (0..pickets).map(|i| p(-64.0 + 64.0 * (i % 3) as f32, 160.0 + 64.0 * (i / 3) as f32)).collect();
        Layout { station, extractors, constructor, raiders, pickets }
    }

    /// The ground the raid needs: the strip and the raiders' approach along x, a little across.
    pub fn footprint() -> (f32, f32) {
        (RAIDER_START + 1.5 * STRIP_SPACING + 300.0, 400.0)
    }
}

/// One raider's script state.
#[derive(Default)]
struct RaiderState {
    /// The target spawn (of side 0) it was last sent at, and when.
    sent: Option<(usize, i32)>,
    fleeing_since: Option<i32>,
    last_flee_order: i32,
    /// The last frame no soldier of ours was near it.
    last_clear: i32,
}

/// One hunt, in progress or over.
pub struct HuntRun {
    pub quarry: UnitId,
    pub quarry_name: String,
    pub hunters: Vec<UnitId>,
    pub started: i32,
    /// Where the hunters stood when the hunt began: the leash is measured from here.
    pub from: Vec3,
    /// Frame and reason when over.
    pub ended: Option<(i32, String)>,
    last_seen: i32,
    resent: i32,
}

pub struct Raid {
    pub station: Vec3,
    /// Side 0's spawn indices: the extractors far to near, the constructor, the pickets.
    pub extractors: Vec<usize>,
    pub constructor: usize,
    pub pickets: Vec<usize>,
    /// Side 1's spawn indices.
    pub raiders: Vec<usize>,
    raider_state: HashMap<usize, RaiderState>,
    pub hunts: Vec<HuntRun>,
    /// Frame the first raider died at.
    pub first_kill: Option<i32>,
    /// Each picket's place at the station, by unit, once it is claimed.
    pub place_of: HashMap<UnitId, Vec3>,
}

impl Raid {
    pub fn new(station: Vec3, extractors: Vec<usize>, constructor: usize, pickets: Vec<usize>, raiders: Vec<usize>) -> Raid {
        Raid { station, extractors, constructor, pickets, raiders, raider_state: HashMap::new(), hunts: Vec::new(), first_kill: None, place_of: HashMap::new() }
    }

    /// The raiders' route: the constructor first, then the extractors far to near.
    fn route(&self) -> Vec<usize> {
        std::iter::once(self.constructor).chain(self.extractors.iter().copied()).collect()
    }

    /// Whether every building and the constructor of ours is gone: nothing left to raid.
    pub fn nothing_left(&self, alive_x: impl Fn(usize) -> bool) -> bool {
        !self.route().into_iter().any(alive_x)
    }

    pub fn hunts_line(&self, started_at: i32) -> String {
        self.hunts
            .iter()
            .map(|h| match &h.ended {
                Some((f, why)) => format!("{}:{}@{:.0}s+{:.0}s", h.quarry_name, why.replace(' ', "_"), (h.started - started_at) as f32 / FPS as f32, (f - h.started) as f32 / FPS as f32),
                None => format!("{}:open@{:.0}s", h.quarry_name, (h.started - started_at) as f32 / FPS as f32),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Side 1's tick: each raider at its next target of the route, unless a soldier of ours is near, when it runs.
    /// `alive_x` and `at_x` are side 0's units by spawn index as side 1 knows them (the director's truth; a
    /// raider attacks what it can see, so it walks up first); `soldiers_x` is where side 0's mobile soldiers stand.
    #[allow(clippy::too_many_arguments)]
    pub fn raider_orders(&mut self, tick: &Tick, unit_of_y: &HashMap<usize, UnitId>, alive_x: &dyn Fn(usize) -> Option<(UnitId, Vec3)>, soldiers_x: &[Vec3], map: (f32, f32), commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let route = self.route();
        for &i in &self.raiders {
            let Some(&unit) = unit_of_y.get(&i) else { continue };
            let Some(own) = tick.snapshot.own_units.iter().find(|u| u.id == unit) else { continue };
            let state = self.raider_state.entry(i).or_default();
            let nearest = soldiers_x.iter().map(|p| (p.dist2d(own.pos), *p)).min_by(|a, b| a.0.total_cmp(&b.0));
            let threatened = nearest.is_some_and(|(d, _)| d < FLEE_RANGE);
            if threatened {
                if state.fleeing_since.is_none() {
                    state.fleeing_since = Some(frame);
                }
                if frame - state.last_flee_order >= FPS {
                    state.last_flee_order = frame;
                    let (_, from) = nearest.unwrap();
                    let (dx, dz) = (own.pos.x - from.x, own.pos.z - from.z);
                    let len = dx.hypot(dz).max(1.0);
                    let to = Vec3 {
                        x: (own.pos.x + dx / len * FLEE_STEP).clamp(100.0, map.0 - 100.0),
                        y: own.pos.y,
                        z: (own.pos.z + dz / len * FLEE_STEP).clamp(100.0, map.1 - 100.0),
                    };
                    commands.push(Command::Move { unit, to, queue: false });
                    state.sent = None;
                }
                continue;
            }
            state.last_clear = frame;
            if let Some(since) = state.fleeing_since {
                // Back to work three seconds after the last soldier went.
                if frame - since < RESUME_AFTER {
                    continue;
                }
                state.fleeing_since = None;
            }
            let Some((target_index, (target, at))) = route.iter().find_map(|&t| alive_x(t).map(|u| (t, u))) else { continue };
            let resend = match state.sent {
                Some((t, _)) if t != target_index => true,
                Some((_, f)) => own.idle && frame - f >= FPS,
                None => true,
            };
            if resend {
                state.sent = Some((target_index, frame));
                let (dx, dz) = (at.x - own.pos.x, at.z - own.pos.z);
                let len = dx.hypot(dz).max(1.0);
                if len > APPROACH + 40.0 {
                    let short = Vec3 { x: at.x - dx / len * APPROACH, y: at.y, z: at.z - dz / len * APPROACH };
                    commands.push(Command::Move { unit, to: short, queue: false });
                    commands.push(Command::Attack { unit, target, queue: true });
                } else {
                    commands.push(Command::Attack { unit, target, queue: false });
                }
            }
        }
    }

    /// Side 0's tick: the standing raider rule (a raider at a building of ours within the trigger reach, with no hunt
    /// on it yet) starts a hunt with the faster pickets; the lane-off arm's hunts are driven and judged here, the
    /// lane-on arm's are handed to the lane as commitments (the return value) and end by its events. `pickets` are
    /// side 0's living mobile soldiers with their type; `buildings` where the living extractors and constructor
    /// stand; `enemy_alive` whether a raider is alive (the director's truth, for "dead").
    #[allow(clippy::too_many_arguments)]
    pub fn picket_orders(
        &mut self,
        tick: &Tick,
        lane_on: bool,
        pickets: &[(&bot_protocol::OwnUnit, &UnitDefInfo)],
        buildings: &[Vec3],
        enemies: &[EnemyUnit],
        defs: &HashMap<UnitDefId, UnitDefInfo>,
        enemy_alive: &dyn Fn(UnitId) -> bool,
        commands: &mut Vec<Command>,
    ) -> HashMap<UnitId, Commitment> {
        let frame = tick.frame;
        let station = self.station;
        let hunting: Vec<UnitId> = self.hunts.iter().filter(|h| h.ended.is_none()).flat_map(|h| h.hunters.iter().copied()).collect();
        // New hunts.
        for e in enemies {
            if self.hunts.iter().any(|h| h.quarry == e.id && h.ended.is_none()) {
                continue;
            }
            let at_structure = buildings.iter().any(|b| b.dist2d(e.pos) < AT_STRUCTURE);
            if !at_structure || e.pos.dist2d(station) > TRIGGER_REACH {
                continue;
            }
            let Some(def) = e.def.and_then(|d| defs.get(&d)) else { continue };
            let mut candidates: Vec<&(&bot_protocol::OwnUnit, &UnitDefInfo)> = pickets
                .iter()
                .filter(|(u, d)| !hunting.contains(&u.id) && u.health >= u.max_health * DROP_HEALTH && d.speed > def.speed)
                .collect();
            candidates.sort_by(|a, b| a.0.pos.dist2d(e.pos).total_cmp(&b.0.pos.dist2d(e.pos)));
            let chosen: Vec<&(&bot_protocol::OwnUnit, &UnitDefInfo)> = candidates.into_iter().take(HUNTERS_PER_QUARRY).collect();
            if chosen.is_empty() {
                continue;
            }
            let n = chosen.len() as f32;
            let from = Vec3 { x: chosen.iter().map(|(u, _)| u.pos.x).sum::<f32>() / n, y: station.y, z: chosen.iter().map(|(u, _)| u.pos.z).sum::<f32>() / n };
            let hunters: Vec<UnitId> = chosen.iter().map(|(u, _)| u.id).collect();
            eprintln!("raid f={frame}: {} hunters after {} ({}) at ({:.0},{:.0})", hunters.len(), e.id.0, def.name, e.pos.x, e.pos.z);
            self.hunts.push(HuntRun { quarry: e.id, quarry_name: def.name.clone(), hunters, started: frame, from, ended: None, last_seen: frame, resent: -1 });
        }
        let mut commitments: HashMap<UnitId, Commitment> = HashMap::new();
        let place_of = self.place_of.clone();
        for run in self.hunts.iter_mut().filter(|h| h.ended.is_none()) {
            let seen = enemies.iter().find(|e| e.id == run.quarry);
            if seen.is_some() {
                run.last_seen = frame;
            }
            if lane_on {
                for &h in &run.hunters {
                    commitments.insert(h, Commitment::Hunt(Hunt { quarry: run.quarry, leash_from: run.from, leash: LEASH }));
                }
                continue;
            }
            // The lane-off arm: the hands' engage as it was, and the same ends judged here.
            let mut ended: Option<&str> = None;
            if seen.is_none() && !enemy_alive(run.quarry) {
                ended = Some("dead");
            } else if seen.is_none() && frame - run.last_seen >= LOST_FRAMES {
                ended = Some("lost");
            }
            let mut dropped: Vec<UnitId> = Vec::new();
            if ended.is_none() {
                for &h in &run.hunters {
                    let Some((u, _)) = pickets.iter().find(|(u, _)| u.id == h) else { dropped.push(h); continue };
                    if u.pos.dist2d(run.from) > LEASH {
                        ended = Some("leash");
                        break;
                    }
                    if u.health < u.max_health * DROP_HEALTH {
                        dropped.push(h);
                    }
                }
            }
            for h in &dropped {
                run.hunters.retain(|x| x != h);
                if let Some(&to) = place_of.get(h) {
                    commands.push(Command::Move { unit: *h, to, queue: false });
                }
            }
            if ended.is_none() && run.hunters.is_empty() {
                ended = Some("no hunters");
            }
            if let Some(why) = ended {
                run.ended = Some((frame, why.to_string()));
                for &h in &run.hunters {
                    if let Some(&to) = place_of.get(&h) {
                        commands.push(Command::Move { unit: h, to, queue: false });
                    }
                }
                continue;
            }
            if let Some(e) = seen
                && (run.resent < 0 || frame - run.resent >= REISSUE_OFF)
            {
                run.resent = frame;
                commands.extend(run.hunters.iter().map(|&unit| Command::Fight { unit, to: e.pos, queue: false }));
            }
        }
        commitments
    }

    /// The lane's hunt events (the lane-on arm): a hunt's end or a hunter's drop sends the hunters back to their
    /// places at the station.
    pub fn note_hunt_events(&mut self, events: &[HuntEvent], frame: i32, commands: &mut Vec<Command>) {
        for event in events {
            let Some(run) = self.hunts.iter_mut().find(|h| h.quarry == event.quarry && h.ended.is_none()) else { continue };
            match event.hunter {
                Some(h) => {
                    run.hunters.retain(|x| *x != h);
                    if let Some(&to) = self.place_of.get(&h) {
                        commands.push(Command::Move { unit: h, to, queue: false });
                    }
                }
                None => {
                    run.ended = Some((frame, event.why.to_string()));
                    for &h in &run.hunters {
                        if let Some(&to) = self.place_of.get(&h) {
                            commands.push(Command::Move { unit: h, to, queue: false });
                        }
                    }
                }
            }
        }
    }
}
