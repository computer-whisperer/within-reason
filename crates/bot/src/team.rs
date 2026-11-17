//! The team board: what the seats we play on one ally team of one game tell each other. Each seat is its own session
//! with its own brain; without the board two of them walk to the same free metal spot, know different halves of the
//! enemy, and send two half-armies at two targets.
//!
//! The board holds facts, not decisions: each brain posts its own and reads the others' on its tick.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};

use bot_protocol::{Hello, UnitDefId, UnitId, Vec3};

/// A seat's post older than this is not counted: the seat is dead or its session gone.
const FRESH_FRAMES: i32 = 90;

/// An enemy building seen and not known destroyed: type, place, frame last seen.
pub type Building = (UnitDefId, Vec3, i32);

#[derive(Default)]
pub struct TeamBoard {
    inner: Mutex<Board>,
    /// The team's one LLM session, alive while a seat's session holds it.
    strategist: Mutex<Weak<crate::strategist::Strategist>>,
    /// The side's commander, when the game has one, alive while a seat's session holds it.
    commander: Mutex<Weak<crate::strategist::command::Commander>>,
}

#[derive(Default)]
struct Board {
    seats: BTreeMap<i32, Post>,
    buildings: HashMap<UnitId, Building>,
    /// Of the pooled buildings, the ones last seen still being built (by whichever seat saw them last).
    unfinished: HashMap<UnitId, (bool, i32)>,
}

/// What one seat says of itself, renewed every tick.
#[derive(Clone, Default)]
pub struct Post {
    pub frame: i32,
    /// Metal spots (indices into the map's list) its builders are on their way to.
    pub spot_claims: HashSet<usize>,
    /// Where its army would go, or is going.
    pub target: Option<Vec3>,
    /// Soldiers it would send now if the odds allowed (empty unless a wave is ready and its home is quiet).
    pub offer: Vec<UnitDefId>,
    /// Soldiers it has out attacking.
    pub committed: Vec<UnitDefId>,
    /// When it last launched a wave.
    pub launched: Option<i32>,
}

/// What the other seats say, as one seat reads it.
#[derive(Default)]
pub struct Others {
    pub spot_claims: HashSet<usize>,
    /// The target of the lowest-numbered other seat that has one and outranks the reader (a lower team number).
    pub lead_target: Option<Vec3>,
    /// Every other seat's offered and committed soldiers.
    pub with_us: Vec<UnitDefId>,
    /// The committed ones alone: out there with a group of ours.
    pub attacking: Vec<UnitDefId>,
    /// The latest wave launch by another seat.
    pub launched: Option<i32>,
}

static BOARDS: Mutex<Vec<(u64, i32, Weak<TeamBoard>)>> = Mutex::new(Vec::new());

/// Which AI plays each team of each game this process is in: the game, the team, the AI, the hold's own number.
static SEATS: Mutex<Vec<(u64, i32, i32, u64)>> = Mutex::new(Vec::new());

/// A session's hold on the team it plays, from its Hello to its end. A lobby can enter two AIs under one team, and
/// the engine then runs both over the same units (1794880397-human, `docs/harness/pitfalls.md`: the autohost's script
/// had `[ai1]` and `[ai2]` on team 3, and two whole sessions, a player and hands each, drove one commander for 26
/// minutes). The second to say Hello is refused the seat and sits out (`session::sit_out`).
pub struct Seat {
    hold: u64,
}

impl Seat {
    /// Takes `team` of `game` for `ai`, or names the other AI of this process that plays it. The same AI asking again
    /// takes its seat over: its connection dropped and came back before its old session saw the end.
    pub fn take(game: u64, team: i32, ai: i32) -> Result<Seat, i32> {
        static HOLDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let mut seats = SEATS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(&(_, _, holder, _)) = seats.iter().find(|(g, t, a, _)| (*g, *t) == (game, team) && *a != ai) {
            return Err(holder);
        }
        seats.retain(|(g, t, ..)| (*g, *t) != (game, team));
        let hold = HOLDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        seats.push((game, team, ai, hold));
        Ok(Seat { hold })
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        SEATS.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).retain(|(.., hold)| *hold != self.hold);
    }
}

impl TeamBoard {
    /// The board of this seat's game and ally team, shared with every other session of this process that plays it.
    pub fn of(hello: &Hello) -> Arc<TeamBoard> {
        let mut boards = BOARDS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        boards.retain(|(_, _, board)| board.strong_count() > 0);
        if let Some(board) = boards.iter().find(|(game, ally, _)| (*game, *ally) == (hello.game_id, hello.ally_team)).and_then(|(_, _, b)| b.upgrade()) {
            return board;
        }
        let board = Arc::new(TeamBoard::default());
        boards.push((hello.game_id, hello.ally_team, Arc::downgrade(&board)));
        board
    }

    /// The team's LLM session, started by the first seat to ask.
    pub fn strategist(&self, start: impl FnOnce() -> std::io::Result<Arc<crate::strategist::Strategist>>) -> std::io::Result<Arc<crate::strategist::Strategist>> {
        let mut held = self.strategist.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(running) = held.upgrade() {
            return Ok(running);
        }
        let started = start()?;
        *held = Arc::downgrade(&started);
        Ok(started)
    }

    /// The side's commander, started by the first seat to ask.
    pub fn commander(&self, start: impl FnOnce() -> std::io::Result<crate::strategist::command::Commander>) -> std::io::Result<Arc<crate::strategist::command::Commander>> {
        let mut held = self.commander.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(running) = held.upgrade() {
            return Ok(running);
        }
        let started = Arc::new(start()?);
        *held = Arc::downgrade(&started);
        Ok(started)
    }

    /// Posts `team`'s state and returns what the other live seats have posted.
    pub fn exchange(&self, team: i32, post: Post) -> Others {
        let mut board = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let frame = post.frame;
        board.seats.insert(team, post);
        let mut others = Others::default();
        for (seat, post) in board.seats.iter().filter(|(seat, post)| **seat != team && frame - post.frame < FRESH_FRAMES) {
            others.spot_claims.extend(&post.spot_claims);
            if *seat < team && others.lead_target.is_none() {
                others.lead_target = post.target;
            }
            others.with_us.extend(post.offer.iter().chain(&post.committed));
            others.attacking.extend(&post.committed);
            others.launched = others.launched.max(post.launched);
        }
        others
    }

    /// Pools enemy buildings: `seen` are this seat's sightings of this tick, `gone` what it knows destroyed or razed.
    /// Returns everything the team remembers.
    pub fn pool_buildings(&self, seen: &[(UnitId, Building)], gone: &[UnitId]) -> HashMap<UnitId, Building> {
        let mut board = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        for id in gone {
            board.buildings.remove(id);
        }
        board.buildings.extend(seen.iter().copied());
        board.buildings.clone()
    }

    /// The side's view of which enemy buildings are still being built: each seat posts what it saw this tick (the
    /// id, whether it was being built, the frame), the latest sighting per id wins, and dead ones go. One seat's
    /// "(being built)" stayed in every picture after another seat saw the turret finished or killed
    /// (bluegecko-3v1-comet-catcher-10: two heavy turrets dead at 7:59 and 11:23 shown being built at 24:00).
    pub fn pool_unfinished(&self, seen: &[(UnitId, bool, i32)], gone: &[UnitId]) -> HashSet<UnitId> {
        let mut board = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        for id in gone {
            board.unfinished.remove(id);
        }
        for (id, being_built, frame) in seen {
            let entry = board.unfinished.entry(*id).or_insert((*being_built, *frame));
            if *frame >= entry.1 {
                *entry = (*being_built, *frame);
            }
        }
        board.unfinished.iter().filter(|(_, (b, _))| *b).map(|(id, _)| *id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seat_reads_the_others_and_follows_only_a_lower_seats_target() {
        let board = TeamBoard::default();
        let at = |x: f32| Vec3 { x, y: 0.0, z: 0.0 };
        let post = |frame, target: f32, claim: usize| Post { frame, spot_claims: HashSet::from([claim]), target: Some(at(target)), offer: vec![UnitDefId(1)], committed: vec![UnitDefId(2)], launched: None };
        assert!(board.exchange(3, post(15, 30.0, 7)).with_us.is_empty());
        let second = board.exchange(5, post(15, 50.0, 9));
        assert_eq!(second.lead_target.map(|t| t.x), Some(30.0));
        assert_eq!(second.spot_claims, HashSet::from([7]));
        assert_eq!(second.with_us.len(), 2);
        assert!(board.exchange(3, post(30, 30.0, 7)).lead_target.is_none());
        // A seat that has stopped posting drops out.
        assert!(board.exchange(3, post(300, 30.0, 7)).with_us.is_empty());
    }

    #[test]
    fn a_team_is_played_by_one_ai_of_a_game() {
        // A game number of this test's own: the seats are the process's, and tests run side by side.
        let game = 0x5ea7_0001;
        let first = Seat::take(game, 3, 1).expect("the first AI on team 3 plays it");
        // 1794880397-human: `[ai2]` of the script on team 3 as well.
        assert_eq!(Seat::take(game, 3, 2).err(), Some(1));
        // Another team of the game, and the same team of another game, are other seats.
        let _other_team = Seat::take(game, 2, 0).expect("team 2 is free");
        let _other_game = Seat::take(game + 1, 3, 2).expect("another game's team 3 is free");
        // The holder's connection came back before its old session ended: it plays on, and the old session's end
        // does not free the seat under it.
        let again = Seat::take(game, 3, 1).expect("the same AI takes its seat over");
        drop(first);
        assert_eq!(Seat::take(game, 3, 2).err(), Some(1));
        // Its session over, the seat is free.
        drop(again);
        assert!(Seat::take(game, 3, 2).is_ok());
    }
}
