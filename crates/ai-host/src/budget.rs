//! H-HOST-ORDER-BUDGET (`docs/design/2026-11-10-order-budget.md`, rule 2): a seat's unit orders are held to the Apex
//! average. Every unit order is a message to every client of the game, and the people's far clients fell behind once
//! the stream outgrew their links (K-engine-a-far-client-falls-behind-when-the-stream-outgrows-its-link). Only a move
//! that replaces the unit's orders is ever held: it waits, one per unit, the newest replacing an older, and goes out
//! as the bucket refills, the largest change first. Every other order goes out when given.

use std::collections::HashMap;

use bot_protocol::{Command, UnitId, Vec3};

/// Orders a second a seat sends on the average: the six Apex seats' mean in the lobby 10v8 on Erebos Lakes
/// (`1794293029-human`: 3.1-3.7 KB/s each late on, about 100 messages a second), the user's standard ("go with the
/// Apex average and moves only", 2026-11-10). Eight AI seats at this rate are the far clients' ceiling.
pub const RATE: f32 = 100.0;
/// The bucket holds this many seconds of the rate: a fresh order to a group of a hundred goes out at once.
const DEPTH_S: f32 = 1.0;
const FRAMES_PER_SECOND: f32 = 30.0;

/// What an order does to a move of the same unit's that is waiting.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Effect {
    /// Sets a state (move or fire state, repeat, target, stockpile, priority): the waiting move stays.
    State,
    /// Joins the unit's queue: the waiting move goes out first, so the queue keeps its order.
    Appends,
    /// Replaces the unit's queue, as the waiting move would have been replaced: it is dropped.
    Replaces,
}

/// The unit an order is for and what it does to a waiting move; `None` for an order of no unit.
fn effect(command: &Command) -> Option<(UnitId, Effect)> {
    let queued = |queue: bool| if queue { Effect::Appends } else { Effect::Replaces };
    Some(match *command {
        Command::Move { unit, queue, .. }
        | Command::Build { unit, queue, .. }
        | Command::Fight { unit, queue, .. }
        | Command::FightReclaimingEnemies { unit, queue, .. }
        | Command::ReclaimFeature { unit, queue, .. }
        | Command::ReclaimUnit { unit, queue, .. }
        | Command::Resurrect { unit, queue, .. }
        | Command::Repair { unit, queue, .. }
        | Command::Attack { unit, queue, .. } => (unit, queued(queue)),
        Command::Tend { unit, .. }
        | Command::Stop { unit }
        | Command::Guard { unit, .. }
        | Command::SelfDestruct { unit }
        | Command::DGun { unit, .. }
        | Command::FireAt { unit, .. } => (unit, Effect::Replaces),
        Command::SetRepeat { unit, .. }
        | Command::SetTarget { unit, .. }
        | Command::ClearTarget { unit }
        | Command::MoveState { unit, .. }
        | Command::FireState { unit, .. }
        | Command::Stockpile { unit, .. }
        | Command::Priority { unit, .. } => (unit, Effect::State),
        _ => return None,
    })
}

struct Waiting {
    to: Vec3,
    /// The frame the unit began to wait (a newer move keeps it).
    since: i32,
}

/// The budget's minute, for the host's `orders` line.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Minute {
    /// Unit orders sent, and moves among them.
    pub sent: u32,
    pub moves: u32,
    /// Moves that had to wait; of those, replaced by a newer move while waiting, and dropped by a later order.
    pub held: u32,
    pub replaced: u32,
    pub dropped: u32,
    /// The longest wait of a move sent, in frames.
    pub longest: i32,
}

pub struct Budget {
    /// Orders a frame; 0 with the budget off.
    per_frame: f32,
    depth: f32,
    tokens: f32,
    frame: i32,
    waiting: HashMap<UnitId, Waiting>,
    /// Where each unit's last move sent was going: a waiting move's weight is its distance from it.
    last_move: HashMap<UnitId, Vec3>,
    minute: Minute,
}

impl Budget {
    /// A budget of `rate` orders a second (0: off).
    pub fn new(rate: f32) -> Budget {
        let per_frame = rate.max(0.0) / FRAMES_PER_SECOND;
        let depth = rate.max(0.0) * DEPTH_S;
        Budget { per_frame, depth, tokens: depth, frame: 0, waiting: HashMap::new(), last_move: HashMap::new(), minute: Minute::default() }
    }

    /// `WITHIN_REASON_ORDER_CAP`, orders a second (0 turns the budget off), else `RATE`.
    pub fn from_env() -> Budget {
        Budget::new(std::env::var("WITHIN_REASON_ORDER_CAP").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(RATE))
    }

    pub fn on(&self) -> bool {
        self.per_frame > 0.0
    }

    fn refill(&mut self, frame: i32) {
        if frame > self.frame {
            self.tokens = (self.tokens + self.per_frame * (frame - self.frame) as f32).min(self.depth);
            self.frame = frame;
        }
    }

    /// An order given at `frame`: what goes out now is pushed onto `out`, in the order it is to go.
    pub fn offer(&mut self, frame: i32, command: Command, out: &mut Vec<Command>) {
        if !self.on() {
            out.push(command);
            return;
        }
        self.refill(frame);
        let Some((unit, effect)) = effect(&command) else {
            out.push(command);
            return;
        };
        match (command, effect) {
            (Command::Move { to, queue: false, .. }, _) => {
                if self.waiting.is_empty() && self.tokens >= 1.0 {
                    self.emit(Command::Move { unit, to, queue: false }, out);
                } else if let Some(w) = self.waiting.get_mut(&unit) {
                    w.to = to;
                    self.minute.replaced += 1;
                } else {
                    self.waiting.insert(unit, Waiting { to, since: frame });
                    self.minute.held += 1;
                }
            }
            (command, Effect::Appends) => {
                if let Some(w) = self.waiting.remove(&unit) {
                    self.minute.longest = self.minute.longest.max(frame - w.since);
                    self.emit(Command::Move { unit, to: w.to, queue: false }, out);
                }
                self.emit(command, out);
            }
            (command, Effect::Replaces) => {
                if self.waiting.remove(&unit).is_some() {
                    self.minute.dropped += 1;
                }
                self.emit(command, out);
            }
            (command, Effect::State) => self.emit(command, out),
        }
    }

    fn emit(&mut self, command: Command, out: &mut Vec<Command>) {
        self.tokens -= 1.0;
        self.minute.sent += 1;
        if let Command::Move { unit, to, queue } = command {
            self.minute.moves += 1;
            if !queue {
                self.last_move.insert(unit, to);
            }
        }
        out.push(command);
    }

    /// Once a frame: as many waiting moves as the bucket holds go out, the heaviest first. A move's weight is its
    /// distance from the unit's last move sent, times one plus the seconds it has waited; a unit with no move sent
    /// yet goes first. So a large change goes before a nudge, and a nudge that has waited long enough goes too.
    pub fn release(&mut self, frame: i32, out: &mut Vec<Command>) {
        if !self.on() {
            return;
        }
        self.refill(frame);
        let room = self.tokens.floor();
        if room < 1.0 || self.waiting.is_empty() {
            return;
        }
        let mut ranked: Vec<(f32, UnitId)> = self.waiting.iter().map(|(unit, w)| (self.weight(*unit, w, frame), *unit)).collect();
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        for (_, unit) in ranked.into_iter().take(room as usize) {
            let w = self.waiting.remove(&unit).expect("ranked from the waiting");
            self.minute.longest = self.minute.longest.max(frame - w.since);
            self.emit(Command::Move { unit, to: w.to, queue: false }, out);
        }
    }

    fn weight(&self, unit: UnitId, w: &Waiting, frame: i32) -> f32 {
        match self.last_move.get(&unit) {
            None => f32::INFINITY,
            Some(last) => last.dist2d(w.to) * (1.0 + (frame - w.since) as f32 / FRAMES_PER_SECOND),
        }
    }

    /// A unit that died, or went to another team: its waiting move and its last move are forgotten.
    pub fn forget(&mut self, unit: UnitId) {
        self.waiting.remove(&unit);
        self.last_move.remove(&unit);
    }

    /// The minute's numbers, and the moves waiting now; the minute starts again.
    pub fn take_minute(&mut self) -> (Minute, usize) {
        (std::mem::take(&mut self.minute), self.waiting.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z: 0.0 }
    }

    fn mv(unit: i32, x: f32) -> Command {
        Command::Move { unit: UnitId(unit), to: at(x), queue: false }
    }

    fn units_of(out: &[Command]) -> Vec<i32> {
        out.iter().filter_map(|c| effect(c).map(|(u, _)| u.0)).collect()
    }

    /// Off: everything goes out as given.
    #[test]
    fn a_budget_of_nothing_holds_nothing() {
        let mut b = Budget::new(0.0);
        let mut out = Vec::new();
        for u in 0..500 {
            b.offer(0, mv(u, 10.0), &mut out);
        }
        b.release(1, &mut out);
        assert_eq!(out.len(), 500);
    }

    /// A second's worth goes out at once; the rest wait, one per unit, and go out at the rate, so a seat sends no
    /// more than the rate over any stretch longer than the bucket's depth.
    #[test]
    fn moves_beyond_the_bucket_wait_and_go_out_at_the_rate() {
        let mut b = Budget::new(RATE);
        let mut out = Vec::new();
        for u in 0..300 {
            b.offer(0, mv(u, 10.0), &mut out);
        }
        assert_eq!(out.len(), 100);
        let mut sent = out.len();
        for frame in 1..=60 {
            let mut more = Vec::new();
            b.release(frame, &mut more);
            sent += more.len();
        }
        // Two seconds on: the 100 of the bucket and two seconds' rate.
        assert!((299..=300).contains(&sent), "{sent}");
        let (minute, waiting) = b.take_minute();
        assert_eq!((minute.held, waiting), (200, 300 - sent));
    }

    /// A newer move for a waiting unit replaces the older, which never goes out; the unit keeps its place.
    #[test]
    fn a_newer_move_replaces_the_one_waiting() {
        let mut b = Budget::new(30.0);
        let mut out = Vec::new();
        for u in 0..30 {
            b.offer(0, mv(u, 10.0), &mut out);
        }
        b.offer(0, mv(99, 100.0), &mut out);
        b.offer(0, mv(99, 200.0), &mut out);
        assert_eq!(out.len(), 30);
        let mut more = Vec::new();
        b.release(1, &mut more);
        assert_eq!(more.len(), 1);
        assert!(matches!(more[0], Command::Move { unit: UnitId(99), to, .. } if to.x == 200.0));
        assert_eq!(b.take_minute().0.replaced, 1);
    }

    /// The largest change goes first; a unit never sent a move before goes before both.
    #[test]
    fn the_largest_change_goes_out_first() {
        let mut b = Budget::new(3.0);
        let mut out = Vec::new();
        b.offer(0, mv(1, 0.0), &mut out);
        b.offer(0, mv(2, 0.0), &mut out);
        b.offer(0, mv(3, 0.0), &mut out);
        assert_eq!(out.len(), 3);
        // The bucket is empty: unit 1 nudged 5, unit 2 moved 300, unit 4 new.
        b.offer(0, mv(1, 5.0), &mut out);
        b.offer(0, mv(2, 300.0), &mut out);
        b.offer(0, mv(4, 50.0), &mut out);
        let mut more = Vec::new();
        b.release(10, &mut more);
        assert_eq!(units_of(&more), vec![4]);
        more.clear();
        b.release(20, &mut more);
        assert_eq!(units_of(&more), vec![2]);
    }

    /// A nudge that has waited long enough goes out ahead of a fresh larger move.
    #[test]
    fn a_nudge_that_has_waited_goes_out() {
        let mut b = Budget::new(3.0);
        let mut out = Vec::new();
        for u in 1..=3 {
            b.offer(0, mv(u, 0.0), &mut out);
        }
        b.offer(0, mv(1, 20.0), &mut out);
        // Nine seconds later unit 2 moves 150: the nudge weighs 20 x 10 = 200, the move 150 x 1.
        b.offer(270, mv(2, 150.0), &mut out);
        let mut more = Vec::new();
        b.release(270, &mut more);
        assert_eq!(units_of(&more)[0], 1, "{more:?}");
    }

    /// Every other order goes out when given, whatever the bucket; a queued one sends the unit's waiting move
    /// first, a replacing one drops it, a state one leaves it.
    #[test]
    fn other_orders_go_out_and_settle_the_waiting_move() {
        let mut b = Budget::new(1.0);
        let mut out = Vec::new();
        b.offer(0, mv(1, 0.0), &mut out);
        for u in [1, 2, 3] {
            b.offer(0, mv(u, 100.0), &mut out);
        }
        out.clear();
        b.offer(0, Command::FireState { unit: UnitId(1), state: 2 }, &mut out);
        b.offer(0, Command::Move { unit: UnitId(2), to: at(500.0), queue: true }, &mut out);
        b.offer(0, Command::Stop { unit: UnitId(3) }, &mut out);
        assert_eq!(out.len(), 4, "{out:?}");
        assert!(matches!(out[0], Command::FireState { .. }));
        assert!(matches!(out[1], Command::Move { unit: UnitId(2), queue: false, .. }));
        assert!(matches!(out[2], Command::Move { unit: UnitId(2), queue: true, .. }));
        assert!(matches!(out[3], Command::Stop { unit: UnitId(3) }));
        let (minute, waiting) = b.take_minute();
        assert_eq!((minute.dropped, waiting), (1, 1));
        b.forget(UnitId(1));
        assert_eq!(b.take_minute().1, 0);
    }
}
