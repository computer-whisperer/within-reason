//! Drills (`duel --drill walk|reverse|gather`; `docs/design/2026-10-24-formation-drills.md` §3): one army, one order,
//! no enemy, measured. **walk**: the army spawned in its formation at the west end of a strip is given one order (a
//! fight, the hands' advance, or a move, the hands' walk) to a point `distance` east, and the drill ends when every
//! living unit is within `ARRIVED` of it. **reverse**: the walk, and `turn_at` after the first order the same order
//! back to the start, the drill ending when everyone is back within `ARRIVED` of it. **gather**: a lab at the west end
//! builds the force under the resource cheat, and every soldier it makes is adopted into the army as it appears and
//! sent to the rally `distance` east. Nothing of the director's goes out between the order and the end but
//! `reissue`, the same order to every living unit every so many frames (the hands' re-send, as an arm), and the lane
//! runs over the orders as it does in a duel (`--lane`).
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
//! `width_contact`, `ramp_half_s`, `ramp_all_s`).
//! The director's side (`director.rs` `drill_orders`, `drill_sample`) owns the army; this module owns the geometry,
//! the clock and the score.

use std::collections::HashMap;

use bot_protocol::{Command, UnitDefId, UnitId, Vec3};

pub const FPS: i32 = 30;
/// A unit within this of its goal has arrived (the hands' own arrival number, `pianist/groups.rs` `ARRIVED`).
pub const ARRIVED: f32 = 300.0;
/// The median unit at this share of its type's speed is up to speed.
pub const UP_TO_SPEED: f32 = 0.9;
/// A soldier appearing within this of the gather's lab is the lab's output, adopted into the army.
pub const AT_THE_LAB: f32 = 400.0;
/// The gather's stores are topped up to full whenever they fall under this share (the cheat gives nothing beyond the
/// storage, and a tier-1 list costs more than the start's store holds).
pub const TOP_UP_UNDER: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Walk,
    Reverse,
    Gather,
}

impl Kind {
    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "walk" => Some(Kind::Walk),
            "reverse" => Some(Kind::Reverse),
            "gather" => Some(Kind::Gather),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Walk => "walk",
            Kind::Reverse => "reverse",
            Kind::Gather => "gather",
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
    /// The order again to every living unit every this many frames (`--reissue S`); `None`: once.
    pub reissue: Option<i32>,
    /// `reverse`: frames after the first order at which the order goes back to the start (`--turn-at S`).
    pub turn_at: i32,
    /// `gather`: the plant that builds the force (`--lab`).
    pub lab: String,
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
    pub turn_at: i32,
    pub start: Vec3,
    pub goal: Vec3,
    /// The goal in force: `start` after a reverse's turn.
    pub toward: Vec3,
    pub turned: Option<i32>,
    /// The frame the last order went out to the whole army.
    pub last_order: Option<i32>,
    /// Per spawn index: the frame first within `ARRIVED` of the goal in force (cleared at a reverse's turn).
    pub arrived: HashMap<usize, i32>,
    /// `gather`: the lab's spawn index, what it builds, whether the list has been queued, and the frame each adopted
    /// unit appeared.
    pub lab: Option<usize>,
    pub wanted: Vec<(UnitDefId, u32)>,
    pub queued: bool,
    pub appeared: HashMap<usize, i32>,
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
}

impl Drill {
    /// The strip's ends from the site's centre: the start `distance / 2` west of it, the goal as far east.
    pub fn new(options: &Options, centre: Vec3, wanted: Vec<(UnitDefId, u32)>) -> Drill {
        let start = Vec3 { x: centre.x - options.distance / 2.0, ..centre };
        let goal = Vec3 { x: centre.x + options.distance / 2.0, ..centre };
        Drill {
            kind: options.kind,
            fight: options.fight,
            reissue: options.reissue,
            turn_at: options.turn_at,
            start,
            goal,
            toward: goal,
            turned: None,
            last_order: None,
            arrived: HashMap::new(),
            lab: None,
            wanted,
            queued: false,
            appeared: HashMap::new(),
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
        }
    }

    /// The drill's order to one unit: the goal in force, a fight or a move.
    pub fn order(&self, unit: UnitId) -> Command {
        if self.fight { Command::Fight { unit, to: self.toward, queue: false } } else { Command::Move { unit, to: self.toward, queue: false } }
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
            _ => walking,
        }
    }

    /// The body's heading on the strip: from the end behind to the goal in force.
    pub fn heading(&self) -> Vec3 {
        let behind = if self.toward == self.goal { self.start } else { self.goal };
        Vec3 { x: self.toward.x - behind.x, y: 0.0, z: self.toward.z - behind.z }
    }

    /// Once a second: speeds, arrivals, the spin-up and recovery clocks, the spread at the median arrival, and the
    /// body's shape (kept at contact). `expected` is how many units the drill waits for (`expected`).
    pub fn sample(&mut self, frame: i32, walkers: &[Walker], expected: usize) {
        self.shape = Shape::of(walkers, self.heading());
        let mut shares: Vec<f32> = Vec::new();
        let mut now: HashMap<UnitId, Vec3> = HashMap::new();
        for w in walkers {
            // A unit that has arrived stands; its share counts until then.
            if let Some(last) = self.last.get(&w.id)
                && w.speed > 0.0
                && !self.arrived.contains_key(&w.index)
            {
                shares.push((last.dist2d(w.at) / w.speed).min(1.5));
            }
            now.insert(w.id, w.at);
            if w.at.dist2d(self.toward) <= ARRIVED {
                self.arrived.entry(w.index).or_insert(frame);
            }
        }
        self.last = now;
        if self.shape_at_contact.is_none() && !self.arrived.is_empty() {
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

    /// Whether the drill is over: every unit it waits for has arrived at the goal in force (after the turn, for a
    /// reverse; the whole list produced and arrived, for a gather).
    pub fn done(&self, walking: usize) -> bool {
        let expected = self.expected(walking);
        if expected == 0 || self.last_order.is_none() {
            return false;
        }
        match self.kind {
            Kind::Reverse => self.turned.is_some() && self.arrived.len() >= expected,
            Kind::Gather => self.appeared.len() >= expected && self.arrived.len() >= expected,
            Kind::Walk => self.arrived.len() >= expected,
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
        let mut joins: Vec<f32> = self.appeared.iter().filter_map(|(i, appeared)| self.arrived.get(i).map(|a| (a - appeared) as f32 / FPS as f32)).collect();
        joins.sort_by(f32::total_cmp);
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
        Options { kind, distance: 3000.0, fight: true, reissue: None, turn_at: 20 * FPS, lab: "armvp".into() }
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
            drill.sample(t * FPS, &walkers(t), 2);
            if t < 27 {
                assert!(!drill.done(2));
            }
        }
        assert!(drill.done(2));
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
        drill.sample(21 * FPS, &walker(5500.0), 1);
        drill.sample(22 * FPS, &walker(5450.0), 1);
        drill.sample(23 * FPS, &walker(5350.0), 1);
        assert!(!drill.done(1));
        assert_eq!(drill.recover, Some(23 * FPS));
        drill.sample(24 * FPS, &walker(3700.0), 1);
        assert!(drill.done(1));
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
        assert!(!drill.done(1));
        drill.appeared.insert(1, 60);
        drill.arrived.insert(1, 120);
        assert!(drill.done(2));
        let score = drill.score(0);
        assert_eq!((score.produced, score.join_median_s), (2, Some(2.0)));
    }
}
