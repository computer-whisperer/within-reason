//! The pianist (`docs/design/2026-09-21-pianist.md`): Opus plays the game, Jev plays the keyboard. Once a game
//! second the brain writes a picture of the game (`picture.rs`) and lists every actor's moves (`menu.rs`: one menu
//! per actor, a move being a verb and what it is aimed at), and when the picture changed it asks Jev: a gate of
//! nouls, then a pick over the worlds joined from the best-rated moves (`compose.rs`,
//! `docs/design/2026-10-01-hands-rebuild.md`). The savings on what is asked are layers with a switch and an audit
//! each (`layers.rs`). The pick runs through the actuators (`execute.rs`, `groups.rs`) until an order ends or the
//! next pick changes it. No decision heuristic runs beneath it; the control lane (`micro.rs`) and the tracking do.
//!
//! In lockstep the calls hold the game (the arena's shim waits for the reply); against a live engine they cost a
//! late tick a second. A call fails safe: every actor keeps its task until the next answer.

pub mod glossary;
pub(super) mod groups;
mod execute;
mod lists;
pub(crate) use lists::step_id;
pub(super) mod picture;
mod remove;
#[cfg(test)]
pub(crate) mod fixtures;
mod transfer;
mod compose;
mod decode;
mod diet;
mod layers;
mod menu;
mod schedule;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use super::HandleOwner;

use crate::strategist::shared::Allowance;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

use bot_protocol::{Command, Event, Tick, UnitDefId, UnitId, Vec3};
use serde_json::json;

use super::economy::FIRST_ORDER_FRAME;
use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};
pub(super) use groups::{Group, GroupTask};
pub(super) use picture::{Party, Place};
pub(crate) use picture::clock;

/// Game seconds between calls (`WITHIN_REASON_JEV_INTERVAL` overrides).
const INTERVAL_SECONDS: f32 = 1.0;
/// Events the picture shows, and for how long.
const RECENT_EVENTS: usize = 12;
const RECENT_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
/// H-HANDS-REFUSED: a spot where the engine refused an extractor is left off every state for this long (smoke-4: the
/// commander asked for the same refused spot thirty times running beside the enemy base, and died there).
const REFUSED_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
/// A unit of ours is under fire this long after its last hit (the brain's own `HIT_MEMORY` for what a party shoots).
pub(super) const UNDER_FIRE_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// What a builder or a lab is committed to.
#[derive(Clone, Debug)]
pub(super) enum Task {
    /// A build order: the type, the site asked for, the spot if an extractor, when it was ordered, whether the
    /// engine has started it (a nanoframe by this builder since the order).
    Build { def: UnitDefId, near: Vec3, spot: Option<usize>, ordered: i32, started: bool },
    /// Guarding a factory or another builder.
    Assist { lab: UnitId, since: i32 },
    Reclaim { at: Vec3, since: i32 },
    /// Taking one unit of ours apart on the player's order (`remove` tool).
    ReclaimUnit { target: UnitId, since: i32 },
    Repair { target: UnitId, since: i32 },
    Walk { to: Vec3, place: String, since: i32 },
    /// Walking beside a group of ours wherever it goes; `at`: where the group stood at the builder's last order.
    Follow { group: String, at: Vec3, since: i32 },
    /// Attacking a party; `to`: where the party stood at the builder's last order.
    Attack { party: String, to: Vec3, since: i32 },
}

/// A building named by an id on the player's list steps: its type, the builder ordered to start it while no frame
/// stands yet, and the frame or building once the engine has made it.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Tagged {
    pub def: UnitDefId,
    pub claim: Option<UnitId>,
    /// The frame the claim was made: of a builder's claims without a frame yet, the oldest gets its next frame.
    pub claimed: i32,
    pub building: Option<UnitId>,
}

/// What a list step with an id finds (`Pianist::tag_state`).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum TagState {
    /// Nobody builds it and nothing of it stands: this builder starts it.
    Free,
    /// The building stands, finished: the step is done.
    Stands,
    /// Its frame stands unfinished: this builder helps build it.
    UnderWay(UnitId),
    /// Another builder is on its way to start it: this builder helps that builder.
    Claimed(UnitId),
    /// This builder is the one building it: the step is the build under way.
    Mine,
    /// The id is another type's building.
    Other(UnitDefId),
}

impl Task {
    pub(super) fn since(&self) -> i32 {
        match self {
            Task::Build { ordered, .. } => *ordered,
            Task::Assist { since, .. } | Task::Reclaim { since, .. } | Task::ReclaimUnit { since, .. } | Task::Repair { since, .. } | Task::Walk { since, .. } | Task::Follow { since, .. } | Task::Attack { since, .. } => *since,
        }
    }
}

#[derive(Default)]
struct Stats {
    calls: u32,
    errors: u32,
    /// Realtime: requests whose answer had not come after `STALE_FRAMES`, dropped unplayed.
    dropped: u32,
    latencies_ms: Vec<f32>,
    tokens: u64,
    questions: u32,
    /// Seconds with something open and nothing asked (the picture as at the last ask).
    quiet: u32,
}

pub struct Pianist {
    /// Jev, when `--pianist`; without it only the lists and the rules' defaults play.
    client: Option<jev::Client>,
    /// A fixed packet from a file (`WITHIN_REASON_PACKET`, the arena's `--packet`): what the hands play from when no
    /// player writes one, the arena instrument of the micro A/Bs (`docs/design/2026-09-25-one-decider.md`, §2).
    pub(super) packet: Option<String>,
    /// Realtime (`WITHIN_REASON_REALTIME`): the call runs on this thread and its answer is played on the tick it
    /// arrives, so the game and the control lane never wait on Jev; in lockstep the call is made in place.
    worker: Option<Worker>,
    pending: Option<Pending>,
    next_request: u64,
    interval_frames: i32,
    last_ask_frame: i32,
    /// Worlds in the pick at most (`WITHIN_REASON_WORLDS`).
    cap: usize,
    /// The layers that are on (`WITHIN_REASON_HANDS_LAYERS`, `layers.rs`).
    layers: layers::Layers,
    /// Builders' and labs' tasks, by unit.
    pub(super) tasks: HashMap<UnitId, Task>,
    /// A builder's next task, already ordered behind the one in progress (H-HANDS-QUEUE): it becomes the task when
    /// the frame in progress is finished, or when the engine starts it.
    pub(super) queued: HashMap<UnitId, Task>,
    /// The player's lists of steps per builder (by actor name), done by the bot without asking (H-HANDS-SCRIPT).
    pub(super) scripts: HashMap<String, VecDeque<String>>,
    /// The list step each builder is on (its words, and the clock of the task it became): diverted from that task
    /// by the pass, the builder gets the step back at the front of its list.
    pub(super) list_steps: HashMap<UnitId, (String, i32)>,
    /// The builders with a list that the pick has sent away from an enemy: their list takes no step while they
    /// are still in the pass for one (`list_is_held`).
    pub(super) list_held: HashSet<UnitId>,
    /// The list step a queued task came from (`queued`), moved to `list_steps` when the task is promoted.
    pub(super) queued_steps: HashMap<UnitId, String>,
    /// The builders whose queued task was ordered before the player replaced or cancelled their list: it is dropped
    /// when its turn comes (`take_queued`).
    pub(super) stale_queue: HashSet<UnitId>,
    /// The buildings the player's lists name by an id (`#name` on a step, `lists::step_id`): one building however
    /// many builders are sent to it (`tag_state`).
    pub(super) tagged: HashMap<String, Tagged>,
    /// The builders helping an id's building up, and which: they take no list step until it stands or is gone.
    pub(super) helping: HashMap<UnitId, String>,
    /// Units a lab has been told to build and not yet started, oldest first.
    pub(super) lab_queue: HashMap<UnitId, Vec<(UnitDefId, i32)>>,
    pub(super) groups: Vec<Group>,
    next_group: usize,
    /// The seat's name tag (`Brain::seat_tag`), on every group name this seat makes.
    pub(super) seat_tag: String,
    /// Which factory made each soldier (the engine's creation events), while it lives: a newcomer joins its
    /// factory's group (H-HANDS-GROUPS: groups are the player's, nothing merges by proximity).
    pub(super) produced_by: HashMap<UnitId, UnitId>,
    /// Each factory's own group, by factory: the group its soldiers gather in unless `produce` names another; made
    /// on the first soldier and remade when it has died out.
    pub(super) rally: HashMap<UnitId, String>,
    /// The pass (`compose.rs`): the worlds of the last pick, the picture's signature and frame at the last gate,
    /// and the events since (`schedule.rs`).
    pub(super) worlds: Vec<compose::World>,
    /// The pick's candidate this second: the world stage one's sample drew, and its probability (for the log).
    pub(super) candidate: Option<(usize, f64)>,
    pub(super) sig: Option<(String, i32)>,
    /// The `news` layer (`layers::news`): each actor's course, the party in its entry and the frame at its last
    /// ask, and the store's extremes when the gate last went.
    pub(super) asked: BTreeMap<String, layers::Asked>,
    pub(super) asked_store: String,
    /// The `same` layer (`layers::same`): each noul as it was last asked.
    said: HashMap<String, layers::Said>,
    /// The `fuse` layer (`decode.rs`): what Jev read of the packet in force.
    decode: decode::Decode,
    /// The `openers` layer (`layers::openers`): each actor's `change` and each party's `answer` at its last ask,
    /// with the frame.
    opened: HashMap<String, (f64, i32)>,
    /// What the gate in flight took, put back if it fails (`gate_failed`).
    undo: Option<Undo>,
    /// The parties in the picture at the last pass: a new one is an event.
    parties_seen: BTreeSet<String>,
    pub(super) events: BTreeSet<String>,
    /// The registers (law 8): what each actor last chose, when, and what it left; printed in its entry.
    pub(super) registers: BTreeMap<String, menu::Register>,
    /// The two moves the gate last rated best for each actor, for the player's report.
    pub(super) rated: compose::Rated,
    /// The names of the places a move can be aimed at as of the last picture (`menu::named_places`): a group that
    /// comes to one has reached it (`groups.rs`).
    pub(super) named: BTreeSet<String>,
    /// The hunts' ends since the last pass line (`groups.rs` `tick_hunt`), for the log.
    pub(super) hunt_events: Vec<String>,
    /// Hunts that ended without a kill: the party, when, and why, said when a hunt of it is offered again.
    pub(super) hunts_failed: Vec<(String, i32, String)>,
    /// The rovers' events since the last pass line (`groups.rs` `rove_events`), for the log.
    pub(super) rove_events: Vec<String>,
    /// Spots where the engine refused an extractor, and until when they are left off (H-HANDS-REFUSED).
    pub(super) refused_spots: HashMap<usize, i32>,
    /// Sites the engine refused for a building (the type, the point, until when): kept out of that type's site
    /// search for `REFUSED_FRAMES`.
    pub(super) refused_sites: Vec<(UnitDefId, Vec3, i32)>,
    /// The frame each builder's list was last set by the player (`queue`): a list newer than the builder's task
    /// displaces a build the engine has not started (H-HANDS-SCRIPT).
    pub(super) script_frame: HashMap<String, i32>,
    /// The frame the player's packet last changed: part of the signature, so a new packet asks at once.
    pub(super) packet_frame: i32,
    /// Our units hit in the last `UNDER_FIRE_FRAMES`, with the frame of the last hit: the pass's and the lists'
    /// under-fire set (`under_fire`), across the ticks between calls.
    pub(super) hits: HashMap<UnitId, i32>,
    /// (builder, party name) pairs with the party inside the builder's alarm reach: a party's arrival is an event once.
    pub(super) alarmed: HashSet<(UnitId, String)>,
    /// The token diet's level and knobs (H-HANDS-DIET).
    pub(super) diet: diet::Diet,
    /// The picture's place names at the last call: a change (a mark, a lane) is an event.
    pub(super) places_seen: BTreeSet<String>,
    /// The packet text `packet_frame` was set for, so one change sets it once.
    packet_seen: String,
    /// Units each lab has started since its allowance was set, by unit name (`produce` caps, "corck:1").
    pub(super) produced: HashMap<(UnitId, String), usize>,
    /// The allowance each lab (by name) was last seen with; a change restarts its counts.
    pub(super) allowed_seen: HashMap<String, Allowance>,
    /// What the last picture named, so a plan's place or party can be looked up.
    pub(super) places: Vec<Place>,
    pub(super) parties: Vec<Party>,
    /// The number the next new enemy party is named with (H-HANDS-PARTY-NAMES); a `Cell` because the picture is
    /// built through `&self`.
    pub(super) next_party: std::cell::Cell<usize>,
    /// Every party seen, by name, with its last sighting: the enemy section says where a party that left sight was
    /// last seen and how long ago (H-HANDS-ENEMY-MEMORY; game 3: nothing said where the block went after 19:30).
    pub(super) party_memory: std::cell::RefCell<Vec<picture::PartySeen>>,
    /// Things worth telling: (frame, text).
    recent: VecDeque<(i32, String)>,
    /// What the hands did this call, for the player's report (`Shared.hands`).
    pub(super) done: Vec<String>,
    /// The whole request and answer per call, when `WITHIN_REASON_JEV_LOG` is set (`docs/harness/record-format.md`,
    /// "The pianist's log").
    log: Option<File>,
    /// The instructions as last written to the log: a call carries them only when they changed.
    logged_instructions: String,
    /// The rules as last logged (the header, then every change: they are read from disk each call).
    logged_rules: String,
    /// What the pass played this second (`execute.rs`, `lists.rs`), for the log line.
    pub(super) played: Vec<serde_json::Value>,
    stats: Stats,
    /// The log's header line is written (before the first request).
    header_logged: bool,
    /// The versioned model has been said in the game chat (once, after the first answer).
    announced: bool,
}

/// The pianist's log format version (`docs/harness/record-format.md`).
const LOG_VERSION: u32 = 3;
/// Realtime: an answer older than this judges a picture too old to play.
const STALE_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// An event alone (a hit, a sighting, an alarm) asks again no oftener than this.
const EVENT_GAP: i32 = 5 * FRAMES_PER_SECOND;

/// The thread that talks to Jev in real time: requests in, answers out, each with the request's number. A gate
/// request carries what the worker needs to compose the worlds and make the pick the instant the gate answers
/// (the user, 2026-09-27: the two calls as fast as they can be made), so the pick is not held for the next tick.
struct Worker {
    to: std::sync::mpsc::Sender<(u64, jev::Request, Follow)>,
    from: std::sync::mpsc::Receiver<(u64, Result<jev::Response, jev::Error>, Option<Followed>)>,
}

/// What composing the worlds after a gate takes, sent along with the gate request.
#[derive(Clone)]
struct Follow {
    menus: Vec<menu::Menu>,
    parties: Vec<Party>,
    places: Vec<Place>,
    store: String,
    cap: usize,
    /// The answers the `same` layer lets stand for the questions it did not send, and those it sent anyway for
    /// its audit with the answer that stands for each.
    stand: BTreeMap<String, jev::Answer>,
    audited: BTreeMap<String, f64>,
}

/// What follows a gate: what the gate flagged, each actor's best-rated moves, the worlds and their lines (None
/// when nothing opened), and the pick's two stages with their answers.
struct Followed {
    flags: BTreeMap<String, f64>,
    rated: compose::Rated,
    worlds: Option<Vec<compose::World>>,
    lines: Vec<String>,
    pick: Option<compose::PickRun>,
}

/// The worlds over what a gate flagged, and the pick's two calls made at once: by the worker in realtime, in
/// place in lockstep.
fn followed(ask: &dyn Fn(&jev::Request) -> Result<jev::Response, jev::Error>, follow: &Follow, answers: &BTreeMap<String, jev::Answer>, state: &serde_json::Value) -> Followed {
    // The standing answers are the ones played, an audited question's too.
    let mut all = answers.clone();
    all.extend(follow.stand.clone());
    let next = compose::follow_up(&follow.menus, &follow.parties, &follow.places, &all, &follow.store, follow.cap, state);
    match next.worlds {
        Some((worlds, lines, state)) => {
            let run = compose::run_pick(ask, &state, &lines, compose::draw());
            Followed { flags: next.flags, rated: next.rated, worlds: Some(worlds), lines, pick: Some(run) }
        }
        None => Followed { flags: next.flags, rated: next.rated, worlds: None, lines: Vec::new(), pick: None },
    }
}

/// A gate request in flight: what it was built from, so its answer can be played when it comes.
struct Pending {
    id: u64,
    frame: i32,
    picture: picture::Picture,
    menus: Vec<menu::Menu>,
    request: jev::Request,
    audited: BTreeMap<String, f64>,
}

/// What a gate takes when it goes: put back when it fails.
struct Undo {
    sig: Option<(String, i32)>,
    asked: BTreeMap<String, layers::Asked>,
    asked_store: String,
    events: BTreeSet<String>,
}

fn spawn_worker(client: jev::Client) -> Worker {
    let (to, requests) = std::sync::mpsc::channel::<(u64, jev::Request, Follow)>();
    let (answers, from) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        while let Ok((mut id, mut request, mut follow)) = requests.recv() {
            // Behind by a slow call, the worker answers the newest request only: answered in order, a burst of
            // one-second calls made every later answer stale (human-1, second game: three of 35 dropped).
            while let Ok((newer_id, newer, newer_follow)) = requests.try_recv() {
                (id, request, follow) = (newer_id, newer, newer_follow);
            }
            let result = client.ask(&request);
            let after = result.as_ref().ok().map(|response| followed(&|r| client.ask(r), &follow, &response.answers, &request.state));
            if answers.send((id, result, after)).is_err() {
                break;
            }
        }
    });
    Worker { to, from }
}

/// A `produce` list entry: the unit name and, after a colon, how many more of it are allowed ("corck:1").

pub fn allowance(entry: &str) -> (&str, Option<usize>) {
    match entry.split_once(':') {
        Some((name, count)) => (name, count.trim().parse::<usize>().ok().filter(|n| *n >= 1)),
        None => (entry, None),
    }
}

/// The cap on the k-th entry of a `produce` list, counted with every earlier capped entry of the same name: the list
/// is a sequence, so in `armfav:5, armcv:1, armfav:10, armcv:1, armfav` the second Rover entry is used up at fifteen
/// made, not ten, and the last is never used up. Counted per name alone, the second entry read "5 more allowed" the
/// moment the first was used up, and the plant's default (the first entry whose unit is permitted) stayed on Rovers
/// past the constructor: player-14 made nine Rovers on a cap of five and no constructor until the player rewrote
/// the list at 2:15. None for an uncapped entry.
pub fn entry_cap(list: &[String], k: usize) -> Option<usize> {
    let (name, cap) = allowance(&list[k]);
    let cap = cap?;
    Some(cap + list[..k].iter().filter_map(|e| { let (n, c) = allowance(e); if n == name { c } else { None } }).sum::<usize>())
}

/// Whether the k-th entry of a `produce` list still permits its unit, `made` being how many of that unit the builder
/// has made.
pub fn entry_permits(list: &[String], k: usize, made: usize) -> bool {
    entry_cap(list, k).is_none_or(|cap| made < cap)
}

/// The entry of a `produce` list a factory makes next without asking (the user's law, 2026-10-01: "make produce a
/// sequence the code follows"): the first entry with a count, in the list's order, whose unit the factory can make
/// and whose count is not yet made. None when no counted entry is left: the factory is then asked among the entries
/// without a count. Until now every unit named was on offer while its total lasted and Jev picked among them:
/// player-30's plant, told `armcv:1, armfav:3, armcv:1, armflash:4`, made a constructor, a Rover, a Blitz, a
/// constructor, a Rover, a constructor, three constructors by 2:23, while the brief told the player the counted
/// entries were built in order.
pub fn sequence_next(list: &[String], can_build: impl Fn(&str) -> bool, made: impl Fn(&str) -> usize) -> Option<usize> {
    (0..list.len()).find(|k| {
        let (name, count) = allowance(&list[*k]);
        count.is_some() && can_build(name) && entry_permits(list, *k, made(name))
    })
}

#[cfg(test)]
mod allowance_tests {
    use super::*;
    #[test]
    fn a_list_naming_a_unit_twice_is_a_sequence() {
        let list: Vec<String> = ["armflash:1", "armfav:5", "armcv:1", "armfav:10", "armcv:1", "armfav"].iter().map(|s| s.to_string()).collect();
        assert_eq!(entry_cap(&list, 1), Some(5));
        assert_eq!(entry_cap(&list, 3), Some(15));
        assert_eq!(entry_cap(&list, 4), Some(2));
        assert_eq!(entry_cap(&list, 5), None);
        // Five Rovers made: the first Rover entry is used up, the second still permits, and the first entry that
        // permits its unit is the constructor.
        assert!(!entry_permits(&list, 1, 5));
        assert!(entry_permits(&list, 3, 5));
        let first = (0..list.len()).find(|k| entry_permits(&list, *k, match allowance(&list[*k]).0 { "armfav" => 5, "armflash" => 1, _ => 0 }));
        assert_eq!(first, Some(2));
    }

    /// The brief's Comet list as a sequence: one constructor, three Rovers, a constructor, six Blitzes, a
    /// constructor, then nothing counted is left and the open Blitz entry is Jev's.
    #[test]
    fn a_factory_makes_the_counted_entries_in_order() {
        let list: Vec<String> = ["armcv:1", "armfav:3", "armcv:1", "armflash:6", "armcv:1", "armflash"].iter().map(|s| s.to_string()).collect();
        let next = |cv: usize, fav: usize, flash: usize| sequence_next(&list, |_| true, |name| match name { "armcv" => cv, "armfav" => fav, _ => flash }).map(|k| allowance(&list[k]).0.to_string());
        assert_eq!(next(0, 0, 0).as_deref(), Some("armcv"));
        assert_eq!(next(1, 0, 0).as_deref(), Some("armfav"));
        assert_eq!(next(1, 2, 0).as_deref(), Some("armfav"));
        assert_eq!(next(1, 3, 0).as_deref(), Some("armcv"));
        assert_eq!(next(2, 3, 0).as_deref(), Some("armflash"));
        assert_eq!(next(2, 3, 6).as_deref(), Some("armcv"));
        assert_eq!(next(3, 3, 6), None, "the open entry is asked about");
        // A factory that cannot make a unit passes over its entries.
        assert_eq!(sequence_next(&list, |name| name != "armcv", |_| 0), Some(1));
        // A list of open entries alone is no sequence.
        assert_eq!(sequence_next(&["armpw".to_string(), "armham".to_string()], |_| true, |_| 0), None);
    }
}

impl Pianist {
    /// What a step naming the building `id` of type `def` finds for `builder` (the law, 2026-10-01: steps carrying
    /// the same id are one building: the first builder to reach the step starts it, any other builder reaching a
    /// step with that id helps build it while it is unfinished, and the step is done once the building stands). An
    /// id whose building is gone, or whose claiming builder is dead or no longer on that build, is free again.
    pub(super) fn tag_state(&mut self, id: &str, def: UnitDefId, builder: UnitId, own: &[bot_protocol::OwnUnit]) -> TagState {
        let Some(tagged) = self.tagged.get(id).cloned() else { return TagState::Free };
        let building = tagged.building.and_then(|unit| own.iter().find(|u| u.id == unit));
        let on_it = |b: UnitId| self.tasks.get(&b).into_iter().chain(self.queued.get(&b)).any(|t| matches!(t, Task::Build { def: d, .. } if *d == tagged.def));
        let claimed = tagged.claim.filter(|b| own.iter().any(|u| u.id == *b) && on_it(*b));
        if building.is_none() && (tagged.building.is_some() || claimed.is_none()) {
            self.tagged.remove(id);
            return TagState::Free;
        }
        if tagged.def != def {
            return TagState::Other(tagged.def);
        }
        match (building, claimed) {
            (Some(unit), _) if !unit.being_built => TagState::Stands,
            (_, Some(b)) if b == builder => TagState::Mine,
            (Some(unit), _) => TagState::UnderWay(unit.id),
            (None, Some(b)) => TagState::Claimed(b),
            (None, None) => TagState::Free,
        }
    }

    /// The player has replaced (`new_list`) or cancelled a builder's list: the old list is gone whole. The hold the
    /// pick's way out put on it ends, the step its builder is on no longer returns to a list when the pass diverts
    /// the builder, and what is queued behind the build in progress is marked to be dropped when its turn comes
    /// (`take_queued`): whatever is queued under a new list, a list's own step under a cancelled one.
    pub(super) fn list_replaced(&mut self, builder: UnitId, new_list: bool) {
        self.list_held.remove(&builder);
        self.list_steps.remove(&builder);
        self.helping.remove(&builder);
        if self.queued.contains_key(&builder) && (new_list || self.queued_steps.contains_key(&builder)) {
            self.stale_queue.insert(builder);
        }
    }

    /// The builder's queued task when its turn comes (the build before it is finished, or the engine has begun the
    /// queued one): it becomes the task, or, queued before the player replaced or cancelled the builder's list, it
    /// is dropped with a stop and said. The law (2026-10-01): a new list leaves the build in progress to finish and
    /// drops what was queued behind it (player-32-hard, 0:17: the opening list queued its third extractor half a
    /// second before the list of the 0:10 turn came into force; the commander built it, then the new list's, four
    /// extractors for the three both lists asked for, and the plant began at 1:34 for 1:08).
    pub(super) fn take_queued(&mut self, builder: UnitId, name: &str, frame: i32) -> Option<Command> {
        if !self.stale_queue.remove(&builder) {
            return self.promote(builder, frame);
        }
        self.queued.remove(&builder)?;
        let what = self.queued_steps.remove(&builder).map_or("the build".to_string(), |step| format!("the step '{step}'"));
        self.tasks.remove(&builder);
        self.done.push(format!("{} {name}: dropped {what} its old list had queued behind the build it was on: you have replaced or cancelled that list since", picture::clock(frame)));
        Some(Command::Stop { unit: builder })
    }

    /// The builder's queued task becomes its task, its clock starting now. Helping the lab is the one task the engine
    /// could not hold queued (the guard order has no queue flag), so it is ordered here, as the build finishes.
    fn promote(&mut self, builder: UnitId, frame: i32) -> Option<Command> {
        let mut next = self.queued.remove(&builder)?;
        if let Some(step) = self.queued_steps.remove(&builder) {
            self.list_steps.insert(builder, (step, frame));
        }
        let mut order = None;
        match &mut next {
            Task::Build { ordered, started, .. } => {
                *ordered = frame;
                *started = false;
            }
            Task::Assist { lab, since } => {
                *since = frame;
                order = Some(Command::Guard { unit: builder, target: *lab });
            }
            Task::ReclaimUnit { target, since } => {
                *since = frame;
                order = Some(Command::ReclaimUnit { unit: builder, target: *target, queue: false });
            }
            Task::Reclaim { since, .. } | Task::Repair { since, .. } | Task::Walk { since, .. } | Task::Follow { since, .. } | Task::Attack { since, .. } => *since = frame,
        }
        self.tasks.insert(builder, next);
        order
    }

    /// The hands: Jev from the environment (the key file or `TYPESAFE_API_KEY`) when `jev`; `Err` says why Jev cannot
    /// be had. Without Jev the lists and the rules' defaults still play.
    pub fn new(jev: bool, log_dir: &Path, ai_id: i32) -> Result<Pianist, String> {
        let client = if jev { Some(jev::Client::from_env().map_err(|e| e.to_string())?) } else { None };
        let seconds: f32 = std::env::var("WITHIN_REASON_JEV_INTERVAL").ok().and_then(|v| v.parse().ok()).unwrap_or(INTERVAL_SECONDS);
        let log = match std::env::var("WITHIN_REASON_JEV_LOG").ok().filter(|v| !v.is_empty() && v != "0") {
            Some(_) => File::create(log_dir.join(format!("jev-{ai_id}.jsonl"))).ok(),
            None => None,
        };
        let worker = if jev && crate::strategist::realtime() { jev::Client::from_env().ok().map(spawn_worker) } else { None };
        let packet = match std::env::var_os("WITHIN_REASON_PACKET") {
            Some(path) => Some(std::fs::read_to_string(&path).map_err(|e| format!("the packet file {} cannot be read: {e}", path.to_string_lossy()))?),
            None => None,
        };
        let cap = std::env::var("WITHIN_REASON_WORLDS").ok().and_then(|v| v.parse::<usize>().ok()).filter(|n| *n >= 2).unwrap_or(compose::CAP);
        let layers = layers::Layers::from_env()?;
        Ok(Pianist {
            client,
            packet,
            worker,
            pending: None,
            next_request: 0,
            interval_frames: ((seconds * FRAMES_PER_SECOND as f32) as i32).max(super::BRAIN_FRAMES),
            last_ask_frame: i32::MIN / 2,
            cap,
            layers,
            tasks: HashMap::new(),
            queued: HashMap::new(),
            lab_queue: HashMap::new(),
            groups: Vec::new(),
            next_group: 0,
            seat_tag: String::new(),
            produced_by: HashMap::new(),
            rally: HashMap::new(),
            worlds: Vec::new(),
            candidate: None,
            sig: None,
            asked: BTreeMap::new(),
            asked_store: String::new(),
            said: HashMap::new(),
            decode: decode::Decode::default(),
            opened: HashMap::new(),
            undo: None,
            parties_seen: BTreeSet::new(),
            events: BTreeSet::new(),
            registers: BTreeMap::new(),
            rated: compose::Rated::new(),
            named: BTreeSet::new(),
            hunt_events: Vec::new(),
            hunts_failed: Vec::new(),
            rove_events: Vec::new(),
            refused_spots: HashMap::new(),
            refused_sites: Vec::new(),
            script_frame: HashMap::new(),
            packet_frame: 0,
            diet: diet::Diet::from_env(),
            hits: HashMap::new(),
            alarmed: HashSet::new(),
            places_seen: BTreeSet::new(),
            packet_seen: String::new(),
            produced: HashMap::new(),
            allowed_seen: HashMap::new(),
            places: Vec::new(),
            parties: Vec::new(),
            next_party: std::cell::Cell::new(1),
            party_memory: std::cell::RefCell::new(Vec::new()),
            recent: VecDeque::new(),
            scripts: HashMap::new(),
            list_steps: HashMap::new(),
            list_held: HashSet::new(),
            queued_steps: HashMap::new(),
            stale_queue: HashSet::new(),
            tagged: HashMap::new(),
            helping: HashMap::new(),
            done: Vec::new(),
            log,
            logged_instructions: String::new(),
            logged_rules: String::new(),
            played: Vec::new(),
            stats: Stats::default(),
            header_logged: false,
            announced: false,
        })
    }

    /// The log's first line: what every call shares.
    pub fn log_header(&mut self, ai_id: i32, rules: &str) {
        self.logged_rules = rules.to_string();
        let model = self.model();
        if let Some(log) = &mut self.log {
            let line = json!({
                "t": "header", "format": "within-reason-jev", "version": LOG_VERSION, "ai_id": ai_id, "model": model,
                "interval_frames": self.interval_frames, "rules": rules, "hands_effort": self.diet.level_name(), "worlds_cap": self.cap, "layers": self.layers.names(),
            });
            let _ = writeln!(log, "{line}");
        }
    }

    /// The hands by name, for the banner and the log: "Jev jev-latest", or the rules alone.
    pub fn model(&self) -> String {
        match &self.client {
            Some(client) => format!("Jev {}", client.model()),
            None => "standing orders and lists, no Jev".to_string(),
        }
    }

    pub(super) fn note(&mut self, frame: i32, text: String) {
        if self.recent.len() == RECENT_EVENTS {
            self.recent.pop_front();
        }
        self.recent.push_back((frame, text));
    }

    pub(super) fn recent(&self, frame: i32) -> Vec<String> {
        self.recent.iter().filter(|(f, _)| frame - f < RECENT_FRAMES).map(|(f, text)| format!("{} {text}", picture::clock(*f))).collect()
    }

    /// An actor's register as its entry prints it: "14:02 picked the walk to spot_30, leaving its course (attacks
    /// party_7); 6 s ago".
    pub(super) fn register_words(&self, actor: &str, frame: i32) -> Option<String> {
        self.registers.get(actor).map(|r| format!("{} {}; {} s ago", picture::clock(r.frame), r.words, (frame - r.frame) / FRAMES_PER_SECOND))
    }

    /// Whether this builder's list waits this second (H-HANDS-SCRIPT, 2026-10-01): the pick sent it away from an
    /// enemy (`execute_builder`) and it is still threatened, which is while the pass still asks about it. The hold
    /// ends when the enemy is off it; a new list from the player ends it too.
    pub(super) fn list_is_held(&mut self, builder: UnitId, threatened: bool) -> bool {
        if !threatened {
            self.list_held.remove(&builder);
        }
        self.list_held.contains(&builder)
    }

    /// Our units hit in the last `UNDER_FIRE_FRAMES`. The record was cleared after each ask until the one pass
    /// (cf66874, 2026-09-25) and never since: a unit hit once was under fire for the rest of the game, so a builder
    /// once shot stayed in the pass, never queued a step behind its build, and (player-32) never came off the hold
    /// the pick's retreat had put on its list: the commander stood idle from 33:57 to the end with the nearest
    /// party 2,679 away.
    pub(super) fn under_fire(&self, frame: i32) -> impl Iterator<Item = UnitId> + '_ {
        self.hits.iter().filter(move |(_, hit)| frame - **hit <= UNDER_FIRE_FRAMES).map(|(id, _)| *id)
    }

    /// A new group's name: A, B, C, ...
    pub(super) fn new_group_name(&mut self) -> String {
        let n = self.next_group;
        self.next_group += 1;
        let letter = (b'A' + (n % 26) as u8) as char;
        let base = if n < 26 { letter.to_string() } else { format!("{letter}{}", n / 26) };
        format!("{base}{}", self.seat_tag)
    }

    fn write_log(&mut self, line: serde_json::Value) {
        if let Some(log) = &mut self.log {
            let _ = writeln!(log, "{line}");
        }
    }
}

impl Brain {
    /// A timed list step `assist N` ends after N seconds: the task is dropped and the builder's list goes on
    /// (H-HANDS-SCRIPT). The engine keeps the guard until the next order replaces it.
    fn end_timed_assists(&mut self, frame: i32, commands: &mut Vec<Command>) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let mut ended: Vec<UnitId> = Vec::new();
        for (unit, (step, _)) in &pianist.list_steps {
            let Some(seconds) = lists::timed_assist(step) else { continue };
            if let Some(Task::Assist { since, .. }) = pianist.tasks.get(unit)
                && frame - since >= seconds * FRAMES_PER_SECOND
            {
                ended.push(*unit);
            }
        }
        for unit in ended {
            pianist.tasks.remove(&unit);
            pianist.list_steps.remove(&unit);
            // The engine keeps a guard until another order: the stop makes the builder idle, and its list's next
            // step plays.
            commands.push(Command::Stop { unit });
        }
    }

    /// The pianist's whole turn of the brain: bookkeeping every think, the pass when one is due.
    pub(super) fn run_pianist(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        self.pianist_housekeeping(tick, kit, commands);
        self.keep_groups(tick, kit, commands);
        self.end_timed_assists(tick.frame, commands);
        // H-REC-CREW under the pianist: resurrection bots are the player's to produce and the bot's to work. They build
        // nothing, so the hands never ask them, and the economy loop that drives them does not run here (2026-09-24:
        // a produced Lazarus would have idled all game).
        for (bot, raised) in std::mem::take(&mut self.reclaim.to_mend) {
            commands.push(Command::Repair { unit: bot, target: raised, queue: false });
        }
        let crew: Vec<bot_protocol::OwnUnit> = tick.snapshot.own_units.iter().filter(|u| !u.being_built && u.idle && kit.is_resurrector(u.def)).cloned().collect();
        for unit in &crew {
            self.work_wrecks(unit, tick, commands);
        }
        // The growth history (the curves and the stagnation wake) and the field the player's report and wake
        // conditions read.
        self.track_growth(tick, kit);
        if let Some(shared) = self.strategist.clone() {
            let soldiers: Vec<&bot_protocol::OwnUnit> = tick.snapshot.own_units.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
            self.publish_field(tick, kit, &soldiers, &shared);
            self.publish_cards(tick, &shared);
            self.take_removals(tick, commands, &shared);
            self.take_transfers(tick, commands, &shared);
        }
        if tick.frame < FIRST_ORDER_FRAME {
            return;
        }
        // Nothing of ours is asked or ordered until the player's first turn is over (the user's law, 2026-09-30;
        // `docs/design/2026-09-30-opening-turn.md`): the hands played those seconds under the default "no player
        // is connected" instructions, a fourth solar and the plant ten seconds late in players 29 and 30.
        if self.strategist.as_ref().is_some_and(|shared| shared.opening_pending()) {
            return;
        }
        if self.pianist.as_ref().expect("pianist mode").worker.is_some() {
            self.collect_answer(tick, kit, commands);
        }
        self.schedule_asks(tick);
        let due = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            // Realtime: one request in flight at a time; this second's pass waits for the answer.
            pianist.pending.is_none() && tick.frame - pianist.last_ask_frame >= pianist.interval_frames
        };
        if !due {
            return;
        }
        self.pianist.as_mut().expect("pianist mode").last_ask_frame = tick.frame;
        self.take_lists(tick, commands);
        let picture = self.picture(tick, kit);
        {
            // A changed packet is asked afresh: its frame is part of the signature.
            let pianist = self.pianist.as_mut().expect("pianist mode");
            let instructions = picture.state["instructions"].as_str().unwrap_or_default();
            if !instructions.is_empty() && instructions != pianist.logged_instructions && instructions != pianist.packet_seen {
                pianist.packet_seen = instructions.to_string();
                pianist.packet_frame = tick.frame;
                pianist.events.insert("packet".to_string());
            }
            let place_set: BTreeSet<String> = picture.places.iter().map(|p| p.name.clone()).collect();
            if !pianist.places_seen.is_empty() && place_set != pianist.places_seen {
                pianist.events.insert("places".to_string());
            }
            pianist.places_seen = place_set;
            pianist.named = menu::named_places(&picture.places, instructions, &pianist.scripts).into_iter().map(|p| p.name.clone()).collect();
            pianist.places = picture.places.clone();
            pianist.parties = picture.parties.clone();
            // The registers and the gate's ratings of actors that are gone go with them.
            let actors = picture.state["actors"].as_object();
            pianist.registers.retain(|name, _| actors.is_some_and(|a| a.contains_key(name)));
            pianist.rated.retain(|name, _| actors.is_some_and(|a| a.contains_key(name)));
        }
        self.play_lists(tick, kit, &picture, commands);
        self.play_sequences(tick, commands);
        self.pass(tick, kit, &picture, commands);
        self.publish_hands(&picture);
        let status_due = tick.due() % (60 * FRAMES_PER_SECOND) < self.pianist.as_ref().expect("pianist mode").interval_frames;
        if status_due {
            self.pianist_status_line(tick.frame);
        }
    }

    /// The player's `queue` calls since the last pass: lists set or cancelled per builder.
    fn take_lists(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(shared) = &self.strategist else { return };
        let lists = std::mem::take(&mut *shared.queues.lock().unwrap());
        let mut others: BTreeMap<String, Option<Vec<String>>> = BTreeMap::new();
        for (name, list) in lists {
            // A list is for the seat that owns the builder: another seat's stays for it (bluegecko-2v1-great-divide:
            // the Cortex list went to the Armada seat, which could build none of it, and the Cortex seat got nothing).
            match self.handle_owner(&name, &tick.snapshot.own_units) {
                HandleOwner::Ours => {}
                HandleOwner::AnotherSeat => {
                    others.insert(name, list);
                    continue;
                }
                HandleOwner::Nobody => {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.done.push(format!("{} {name}: no builder of ours by that name stands; its list is dropped", picture::clock(tick.frame)));
                    continue;
                }
            }
            // A list beginning with `stop` (or the bare word) drops what the builder is doing now: the build in
            // progress is abandoned and its frame decays. `null` cancels the list and lets that build finish.
            let mut steps = list;
            if let Some(s) = steps.as_mut()
                && s.first().is_some_and(|w| w == "stop")
            {
                s.remove(0);
                if let Some(unit) = self.unit_by_handle(&name, &tick.snapshot.own_units).map(|u| u.id) {
                    commands.push(Command::Stop { unit });
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.tasks.remove(&unit);
                    // The stop empties the engine's queue: what was queued behind the build is gone with it.
                    pianist.queued.remove(&unit);
                    pianist.queued_steps.remove(&unit);
                    pianist.done.push(format!("{} {name}: stopped what it was doing on your `stop`", picture::clock(tick.frame)));
                }
                if s.is_empty() {
                    steps = None;
                }
            }
            // A new list is the player's order now: it ends the hold the pick's retreat put on the old one.
            let listed = tick.snapshot.own_units.iter().find(|u| self.actor_name(u.id) == name).map(|u| u.id);
            let pianist = self.pianist.as_mut().expect("pianist mode");
            if let Some(id) = listed {
                pianist.list_replaced(id, steps.is_some());
            }
            pianist.script_frame.insert(name.clone(), tick.frame);
            match steps {
                Some(steps) => {
                    pianist.scripts.insert(name, steps.into());
                }
                None => {
                    pianist.scripts.remove(&name);
                }
            }
            pianist.events.insert("lists".to_string());
        }
        if !others.is_empty() {
            shared.queues.lock().unwrap().extend(others);
        }
    }

    /// The pass (`menu.rs`, `compose.rs`): the menus this second, and when the picture changed since the last gate
    /// (or an event, or an actor's re-ask, calls for it) the gate's nouls go out for every party and for the actors
    /// the layers leave open (`layers.rs`); the worlds and the pick follow in `gate_answered`.
    fn pass(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let mut menus = self.menus(tick, kit, picture);
        let parties = &picture.parties;
        let mut line = json!({ "t": "pass", "f": frame });
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            // A party in the picture this second and not the last: said among the events, so the log shows what
            // opened the ask (onepass-hard-4, 5:04: the reader could not tell why group_A moved).
            let now: BTreeSet<String> = parties.iter().map(|p| p.name.clone()).collect();
            for name in now.difference(&pianist.parties_seen) {
                pianist.events.insert(format!("{name} appeared"));
            }
            pianist.parties_seen = now;
            if !pianist.hunt_events.is_empty() {
                line["hunts"] = json!(std::mem::take(&mut pianist.hunt_events));
            }
            if !pianist.rove_events.is_empty() {
                line["rove"] = json!(std::mem::take(&mut pianist.rove_events));
            }
        }
        // The economy at its extremes only: the stock words' five buckets flapped at their edges (onepass-medium-3:
        // 58 of 663 asks); and whether the store now covers the cheapest unit an idle factory could make: the store
        // crossing that cost is what an idle factory waits for (onepass-norules-hard-4: each Stout cycle ran 5 s of
        // building and 21 s of waiting).
        let idle_factory_afford = menus.iter().any(|m| {
            matches!(m.kind, menu::Kind::Factory(_)) && m.idle && m.moves.iter().filter_map(|mv| if let menu::Order::Make(def) = &mv.order { self.world.def(*def).map(|d| d.metal_cost) } else { None }).fold(f32::INFINITY, f32::min) <= tick.snapshot.metal.current
        });
        let eco = format!("{}|{}|{}", if tick.snapshot.metal.current < 100.0 { "empty" } else if tick.snapshot.metal.current >= tick.snapshot.metal.storage - 1.0 { "full" } else { "" }, picture.state["economy"]["energy"].as_str().is_some_and(|e| e.contains("STALLING")), idle_factory_afford);
        // The `news` layer: the actors with a course and no news are closed this gate.
        let jev_on = self.pianist.as_ref().is_some_and(|p| p.client.is_some());
        let (skipped, audited) = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            if pianist.layers.news {
                let news = layers::News { frame, events: &pianist.events, orders: ["packet", "lists"].iter().any(|e| pianist.events.contains(*e)), store: eco != pianist.asked_store };
                layers::news(&mut menus, &pianist.asked, &news, &mut compose::draw)
            } else {
                (0, 0)
            }
        };
        // The log's header, before the first request of any kind (the decode's may be the first).
        if jev_on && self.pianist.as_ref().is_some_and(|p| !p.header_logged) {
            let rules = picture.state["rules"].as_str().unwrap_or_default().to_string();
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.header_logged = true;
            pianist.log_header(self.world.hello.ai_id, &rules);
        }
        // The `fuse` layer: what the packet clearly does not give an actor is not asked.
        let (fused, fuse_audited) = if self.pianist.as_ref().is_some_and(|p| p.layers.fuse && p.client.is_some()) { self.decode_and_fuse(tick, picture, &mut menus) } else { (0, 0) };
        // The `openers` layer: a move whose openers said no at the last gate waits for one to open.
        let (held, held_audited) = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            if pianist.layers.openers { layers::openers(&mut menus, &pianist.opened, frame, compose::FLAG, &mut compose::draw) } else { (0, 0) }
        };
        let questions = compose::gate_questions(&menus, parties, &picture.places);
        // The `same` layer: a question in the words of its last ask is not sent again.
        let asked_whole = questions.len();
        let same = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            if pianist.layers.same { layers::same(questions, &pianist.said, frame, &mut compose::draw) } else { layers::Same { send: questions, stand: BTreeMap::new(), audited: BTreeMap::new() } }
        };
        let questions = same.send;
        let sig = format!("{}|{eco}|{}", layers::signature(&menus, parties), self.pianist.as_ref().expect("pianist mode").packet_frame);
        let jev = self.pianist.as_ref().is_some_and(|p| p.client.is_some());
        let (events, changed) = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            let events: Vec<String> = pianist.events.iter().cloned().collect();
            // An event alone asks again no oftener than `EVENT_GAP` (onepass-medium-3: a group under fire asked every
            // second, 118 of 663 asks on "hit" alone).
            let since_ask = pianist.sig.as_ref().map_or(i32::MAX, |(_, f)| frame - *f);
            // A place reached or a first sighting since the last stop asks at once (routes-in-prose §4.3): the next
            // move starts within a second of the arrival.
            // An opener that opened over held moves (the `openers` layer) is asked about again at once, with them.
            let urgent = events.iter().any(|e| e.contains(" reached ") || e.contains(" met ") || e.ends_with(" opened"));
            // An actor's own re-ask comes due on its own clock, whatever the rest of the picture does.
            let due = menus.iter().any(|m| m.asks_own() && !m.audit && pianist.asked.get(&m.name).is_none_or(|a| frame - a.frame >= layers::RE_ASK));
            let changed = pianist.sig.as_ref().is_none_or(|(s, f)| *s != sig || frame - *f >= layers::RE_ASK) || (!events.is_empty() && since_ask >= EVENT_GAP) || urgent || due;
            // The `tick` layer: a gate that no event calls for waits until `TICK` after the last one.
            let changed = changed && (!pianist.layers.tick || !events.is_empty() || since_ask >= layers::TICK);
            (events, changed)
        };
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if asked_whole == 0 || !jev {
            line["played"] = json!(std::mem::take(&mut pianist.played));
            if !line["played"].as_array().is_some_and(Vec::is_empty) || line.get("hunts").is_some() || line.get("rove").is_some() {
                pianist.write_log(line);
            }
            return;
        }
        if !changed || questions.is_empty() {
            line["quiet"] = json!(if changed { "every question is in the words of its last ask: its answers stand" } else { "the picture is as at the last gate: every course stands" });
            pianist.stats.quiet += 1;
            line["played"] = json!(std::mem::take(&mut pianist.played));
            pianist.write_log(line);
            return;
        }
        // What a failed gate puts back, so that it is asked again at the next second.
        pianist.undo = Some(Undo { sig: pianist.sig.clone(), asked: pianist.asked.clone(), asked_store: pianist.asked_store.clone(), events: pianist.events.clone() });
        pianist.sig = Some((sig, frame));
        pianist.events.clear();
        pianist.worlds.clear();
        pianist.asked.retain(|_, a| frame - a.frame < layers::RE_ASK);
        for m in menus.iter().filter(|m| m.open()) {
            pianist.asked.insert(m.name.clone(), layers::Asked { course: layers::course(m), party: m.near_party.clone(), frame });
        }
        pianist.asked_store = eco;
        line["open"] = json!(menus.iter().filter(|m| m.open()).map(|m| m.name.as_str()).collect::<Vec<_>>());
        let closed: Vec<&str> = menus.iter().filter(|m| m.quiet).map(|m| m.name.as_str()).collect();
        if !closed.is_empty() {
            line["closed"] = json!(closed);
        }
        line["layers"] = json!({ "news": { "on": pianist.layers.news, "skipped": skipped, "audited": audited }, "same": { "on": pianist.layers.same, "skipped": same.stand.len() - same.audited.len(), "audited": same.audited.len() }, "fuse": { "on": pianist.layers.fuse, "skipped": fused, "audited": fuse_audited }, "openers": { "on": pianist.layers.openers, "skipped": held, "audited": held_audited } });
        line["menus"] = compose::log_menus(&menus);
        line["gate"] = json!(questions.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>());
        if !events.is_empty() {
            line["events"] = json!(events);
        }
        line["played"] = json!(std::mem::take(&mut pianist.played));
        pianist.write_log(line);
        // The state: the picture trimmed to the asked actors (`diet.rs`). Jev is not sent `places` (offline on
        // player-28, 24 gate calls: without it a noul moved 0.023 against 0.017 re-asked and 48 flags flipped
        // against 36, for 18% fewer tokens): a move's words say its place's distance and what stands at it.
        let asked: Vec<(String, Option<Vec3>)> = menus.iter().filter(|m| m.asked()).map(|m| (m.name.clone(), m.at)).collect();
        let diet = pianist.diet.clone();
        let mut state = self.trim_state(&diet, picture, &asked);
        if let Some(obj) = state.as_object_mut() {
            obj.remove("places");
        }
        let shed = diet::shed(&mut state, diet::STATE_CHARS);
        if !shed.is_empty() {
            self.pianist.as_mut().expect("pianist mode").write_log(json!({ "t": "shed", "f": tick.frame, "shed": shed }));
        }
        let request = jev::Request { state, questions: questions.into_iter().collect() };
        let follow = Follow { menus, parties: picture.parties.clone(), places: picture.places.clone(), store: Self::store_words(picture), cap: self.pianist.as_ref().expect("pianist mode").cap, stand: same.stand, audited: same.audited };
        self.call(tick, kit, picture.clone(), request, follow, commands);
    }

    /// The `fuse` layer's turn of the pass (`decode.rs`): the readings the menus need that the decode does not hold
    /// (all of them when the packet is new) are asked of Jev in one lean request over the instructions alone, made in
    /// place; then the clear readings are put on the menus. A decode that fails fuses nothing: its questions are
    /// then the second's own, as in the base. Returns the questions not sent and the moves audited.
    fn decode_and_fuse(&mut self, tick: &Tick, picture: &picture::Picture, menus: &mut [menu::Menu]) -> (usize, usize) {
        let instructions = picture.state["instructions"].as_str().unwrap_or_default();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if pianist.decode.packet != instructions {
            pianist.decode = decode::Decode { packet: instructions.to_string(), reads: BTreeMap::new() };
        }
        let what = |actor: &str| {
            let entry = &picture.state["actors"][actor];
            entry["units"].as_str().or(entry["is"].as_str()).unwrap_or_default().chars().take(120).collect::<String>()
        };
        let asks = decode::needed(menus, &pianist.decode, &what);
        if !asks.is_empty() {
            let request = jev::Request { state: json!({ "instructions": instructions }), questions: asks };
            pianist.stats.calls += 1;
            pianist.stats.questions += request.questions.len() as u32;
            match pianist.client.as_ref().expect("the fuse layer needs Jev").ask(&request) {
                Ok(response) => {
                    pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
                    pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                    let reads: BTreeMap<String, f64> = response.answers.iter().filter_map(|(k, a)| if let jev::Answer::Noul { noul } = a { Some((k.clone(), *noul)) } else { None }).collect();
                    pianist.write_log(json!({ "t": "decode", "f": tick.frame, "ms": (response.latency.as_secs_f32() * 1000.0) as u32, "usage": response.usage, "batches": response.batches, "asked": request.questions.len(), "reads": reads }));
                    pianist.decode.reads.extend(reads);
                }
                Err(e) => {
                    pianist.stats.errors += 1;
                    pianist.write_log(json!({ "t": "error", "f": tick.frame, "error": format!("the decode: {e}") }));
                }
            }
        }
        decode::fuse(menus, &pianist.decode, &mut compose::draw)
    }

    /// The store's words for the worlds' lines ("340 of 500 stored").
    fn store_words(picture: &picture::Picture) -> String {
        picture.state["economy"]["metal"].as_str().and_then(|m| m.split(';').next()).unwrap_or_default().to_string()
    }

    /// The gate call to Jev: through the worker in realtime (its answer lands in `collect_answer`), in place in
    /// lockstep.
    fn call(&mut self, tick: &Tick, kit: &Kit, picture: picture::Picture, request: jev::Request, follow: Follow, commands: &mut Vec<Command>) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.stats.calls += 1;
        pianist.stats.questions += request.questions.len() as u32;
        if pianist.worker.is_some() {
            let id = pianist.next_request;
            pianist.next_request += 1;
            if pianist.worker.as_ref().expect("checked").to.send((id, request.clone(), follow.clone())).is_ok() {
                pianist.pending = Some(Pending { id, frame: tick.frame, picture, menus: follow.menus, request, audited: follow.audited });
            }
            return;
        }
        let client = pianist.client.as_ref().expect("a call needs Jev");
        let result = client.ask(&request);
        let followed = result.as_ref().ok().map(|response| followed(&|r| client.ask(r), &follow, &response.answers, &request.state));
        self.gate_answered(tick, kit, &picture, follow.menus, &follow.audited, &request, result, followed, commands);
    }

    /// The gate's answer: logged, and what followed it (the worlds and the pick's two calls) played. A gate that
    /// failed puts back what it took, so that it is asked again at the next second; every actor keeps its course.
    #[allow(clippy::too_many_arguments)]
    fn gate_answered(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, menus: Vec<menu::Menu>, same_audited: &BTreeMap<String, f64>, request: &jev::Request, result: Result<jev::Response, jev::Error>, followed: Option<Followed>, commands: &mut Vec<Command>) {
        let response = match result {
            Ok(response) => response,
            Err(e) => {
                self.gate_failed(tick.frame, &e.to_string());
                return;
            }
        };
        self.pianist.as_mut().expect("pianist mode").undo = None;
        self.count_call(&response);
        self.announce_hands(&response, commands);
        self.log_call(tick, request, &response);
        let Some(followed) = followed else { return };
        // The layers' audits: what a layer assumed of an actor it closed, beside what the gate said of it.
        let fresh = |id: &str| match response.answers.get(id) {
            Some(jev::Answer::Noul { noul }) => Some(*noul),
            _ => None,
        };
        // What the gate said of a question, sent this second or standing from its last ask.
        let noul = |id: &str| followed.flags.get(id).copied().or_else(|| fresh(id));
        let mut audits: Vec<serde_json::Value> = same_audited
            .iter()
            .filter_map(|(id, stood)| fresh(id).map(|fresh| json!({ "t": "audit", "f": tick.frame, "layer": "same", "id": id, "assumed": stood, "fresh": fresh, "fault": (fresh >= compose::FLAG) != (*stood >= compose::FLAG) })))
            .collect();
        audits.extend(menus
            .iter()
            .filter(|m| m.audit)
            .map(|m| {
                let change = noul(&format!("{}.change", m.name)).unwrap_or(0.0);
                let best = m.moves.iter().skip(1).filter_map(|mv| noul(&m.id(mv)).map(|p| (m.id(mv), p))).max_by(|a, b| a.1.total_cmp(&b.1));
                let fault = change >= compose::FLAG && best.as_ref().is_some_and(|(_, p)| *p >= compose::FLAG);
                json!({ "t": "audit", "f": tick.frame, "layer": "news", "actor": m.name, "assumed": "no change", "change": change, "best": best.as_ref().map(|(id, _)| id.clone()), "best_p": best.map(|(_, p)| p), "fault": fault })
            }));
        // The `fuse` layer's audits: a fused move asked anyway and rated 0.5 or over, or a decoded mark the
        // second's own reading puts on the other side of the bar.
        for m in &menus {
            for mv in m.moves.iter().filter(|mv| mv.audit) {
                let id = m.id(mv);
                if mv.held {
                    // A held move asked anyway: a fault is one that would have gone to the pick this second.
                    let opener = [mv.party.as_ref().map(|p| format!("{p}.answer")), Some(format!("{}.change", m.name))].into_iter().flatten().filter_map(|k| noul(&k)).fold(0.0, f64::max);
                    if let Some(p) = noul(&id) {
                        audits.push(json!({ "t": "audit", "f": tick.frame, "layer": "openers", "id": id, "assumed": "its opener says no", "opener": opener, "fresh": p, "fault": opener >= compose::FLAG && p >= compose::FLAG }));
                    }
                } else if let Some(mark) = mv.marked {
                    if let Some(p) = noul(&format!("{id}.forbidden")) {
                        audits.push(json!({ "t": "audit", "f": tick.frame, "layer": "fuse", "id": format!("{id}.forbidden"), "assumed": mark, "fresh": p, "fault": (p >= compose::FORBIDDEN) != mark }));
                    }
                } else if let Some(p) = noul(&id) {
                    audits.push(json!({ "t": "audit", "f": tick.frame, "layer": "fuse", "id": id, "assumed": "off", "fresh": p, "fault": p >= compose::FLAG }));
                }
            }
        }
        // The openers as they came back; one that opened over held moves asks again the next second, with them.
        let openers: Vec<(String, f64)> = followed.flags.iter().filter_map(|(k, p)| k.strip_suffix(".change").or_else(|| k.strip_suffix(".answer")).filter(|name| !name.contains('.')).map(|name| (name.to_string(), *p))).collect();
        let reopened: Vec<String> = openers
            .iter()
            .filter(|(name, p)| *p >= compose::FLAG && menus.iter().any(|m| m.moves.iter().any(|mv| mv.held && (m.name == *name || mv.party.as_deref() == Some(name.as_str())))))
            .map(|(name, _)| format!("{name} opened"))
            .collect();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.opened.retain(|_, (_, f)| tick.frame - *f < layers::RE_ASK);
        for (name, p) in openers {
            pianist.opened.insert(name, (p, tick.frame));
        }
        pianist.events.extend(reopened);
        layers::remember(&mut pianist.said, &request.questions, &response.answers, same_audited, tick.frame);
        for audit in audits {
            pianist.write_log(audit);
        }
        pianist.write_log(json!({ "t": "worlds_gate", "f": tick.frame, "flags": followed.flags, "worlds": followed.worlds, "lines": followed.lines }));
        pianist.rated.extend(followed.rated);
        let (Some(worlds), Some(run)) = (followed.worlds, followed.pick) else { return };
        pianist.worlds = worlds.clone();
        pianist.candidate = run.candidate;
        // Stage one's call, logged; stage two's answer is the pick.
        if let Some((first, result)) = run.stage_one {
            match result {
                Ok(response) => {
                    self.pianist.as_mut().expect("pianist mode").stats.calls += 1;
                    self.pianist.as_mut().expect("pianist mode").stats.questions += 1;
                    self.count_call(&response);
                    self.log_call(tick, &first, &response);
                }
                Err(e) => self.call_failed(tick.frame, &e.to_string()),
            }
        }
        let Some((second, result)) = run.stage_two else { return };
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.stats.calls += 1;
            pianist.stats.questions += 1;
        }
        match result {
            Ok(response) => {
                self.count_call(&response);
                self.log_call(tick, &second, &response);
                self.after_pick(tick, kit, picture, &menus, &worlds, &response.answers, commands);
            }
            Err(e) => self.call_failed(tick.frame, &e.to_string()),
        }
    }

    fn count_call(&mut self, response: &jev::Response) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
        pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
    }

    fn call_failed(&mut self, frame: i32, error: &str) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.stats.errors += 1;
        pianist.write_log(json!({ "t": "error", "f": frame, "error": error }));
        eprintln!("[ai {}] f={frame} pianist: {error}; every actor keeps its course", self.world.hello.ai_id);
    }

    /// A gate that failed or whose answer came too late: what it took is put back (who was asked, the events it
    /// cleared, the signature), so the next second asks it again.
    fn gate_failed(&mut self, frame: i32, error: &str) {
        self.call_failed(frame, error);
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if let Some(undo) = pianist.undo.take() {
            pianist.sig = undo.sig;
            pianist.asked = undo.asked;
            pianist.asked_store = undo.asked_store;
            pianist.events.extend(undo.events);
        }
    }

    /// The pick put in force, and logged as a `plan` line.
    #[allow(clippy::too_many_arguments)]
    fn after_pick(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, menus: &[menu::Menu], worlds: &[compose::World], answers: &BTreeMap<String, jev::Answer>, commands: &mut Vec<Command>) {
        let Some((wi, confidence)) = compose::pick(answers, worlds) else { return };
        let candidate = self.pianist.as_ref().expect("pianist mode").candidate;
        let changed = self.apply_world(tick, kit, picture, menus, &worlds[wi], commands);
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let played = std::mem::take(&mut pianist.played);
        pianist.write_log(json!({ "t": "plan", "f": tick.frame, "pick": wi + 1, "confidence": confidence, "candidate": candidate.map(|(c, _)| c + 1), "sampled_p": candidate.map(|(_, p)| p), "changed": changed, "played": played }));
        self.journal.note_from("plan", tick.frame, "worlds", json!({ "worlds": worlds.len() }), json!({ "pick": wi + 1, "confidence": confidence, "changed": changed }));
    }

    /// The groups as the log sees them: name, members, centre, task.
    fn groups_json(&self, own: &[bot_protocol::OwnUnit]) -> Vec<serde_json::Value> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        pianist
            .groups
            .iter()
            .map(|g| {
                let units = g.units(own);
                let centre = groups::centre_of(&units);
                let task = match &g.task {
                    GroupTask::Hold { picked, .. } => json!({ "kind": "hold", "picked": picked }),
                    GroupTask::Move { to, place, fight, .. } => json!({ "kind": if *fight { "fight_to" } else { "move_to" }, "place": place, "to": [to.x as i32, to.z as i32] }),
                    GroupTask::Engage { at, target, .. } => json!({ "kind": if target.is_some() { "attack_unit" } else { "engage" }, "to": [at.x as i32, at.z as i32] }),
                    GroupTask::Hunt(h) => json!({ "kind": "hunt", "party": h.party, "quarry": h.quarry.0, "to": [h.at.x as i32, h.at.z as i32] }),
                    GroupTask::Follow { name, at, .. } => json!({ "kind": "follow", "ward": name, "to": [at.x as i32, at.z as i32] }),
                };
                json!({ "name": g.name, "members": g.members.iter().map(|id| id.0).collect::<Vec<_>>(), "at": centre.map(|c| [c.x as i32, c.z as i32]), "task": task })
            })
            .collect()
    }

    fn places_json(&self) -> Vec<serde_json::Value> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        pianist.places.iter().map(|p| json!({ "name": p.name, "x": p.at.x as i32, "z": p.at.z as i32, "spot": p.spot })).collect()
    }

    /// The hands' versioned model, said in the game chat once the first answer names it (the start banner can only
    /// give the alias asked for).
    fn announce_hands(&mut self, response: &jev::Response, commands: &mut Vec<Command>) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if pianist.announced {
            return;
        }
        pianist.announced = true;
        let tail = format!("'s hands: Jev {} ({} ms to the first answer)", response.model, response.latency.as_millis());
        self.said.push(tail.clone());
        commands.push(Command::Say { text: format!("{{name}}{tail}") });
    }

    /// Realtime: plays the answer to the request in flight when it has come, and drops a request whose answer is too
    /// late to judge the picture it was built from.
    fn collect_answer(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        let mut stale: Option<String> = None;
        let arrived = {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            let Some(worker) = &pianist.worker else { return };
            let mut got = None;
            while let Ok((id, result, followed)) = worker.from.try_recv() {
                // An answer to a request already dropped is not played.
                if pianist.pending.as_ref().is_some_and(|p| p.id == id) {
                    got = Some((result, followed));
                }
            }
            match got {
                Some((result, followed)) => Some((pianist.pending.take().expect("a matched request is pending"), result, followed)),
                None => {
                    if let Some(p) = &pianist.pending
                        && tick.frame - p.frame > STALE_FRAMES
                    {
                        pianist.stats.dropped += 1;
                        stale = Some(format!("the answer to the request of frame {} has not come in {} s; dropped", p.frame, STALE_FRAMES / FRAMES_PER_SECOND));
                        pianist.pending = None;
                    }
                    None
                }
            }
        };
        if let Some(why) = stale {
            self.gate_failed(tick.frame, &why);
        }
        let Some((pending, result, followed)) = arrived else { return };
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.places = pending.picture.places.clone();
            pianist.parties = pending.picture.parties.clone();
        }
        self.gate_answered(tick, kit, &pending.picture, pending.menus, &pending.audited, &pending.request, result, followed, commands);
    }

    /// Every tick between thinks: an answer that has come is played now rather than at the next think (the user,
    /// 2026-09-27: the two calls as fast as they can be made; a think is every 15 frames, a tick every 3).
    pub(super) fn poll_hands(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(kit) = self.kit else { return };
        if tick.frame < FIRST_ORDER_FRAME || self.pianist.as_ref().is_none_or(|p| p.worker.is_none()) {
            return;
        }
        self.collect_answer(tick, &kit, commands);
    }

    /// One line of the log per call: the request (the instructions and the rules only when they changed; the rules
    /// are in the header and re-read from disk each call), the answers, and the groups, places and parties by name
    /// so a reader can draw them.
    fn log_call(&mut self, tick: &Tick, request: &jev::Request, response: &jev::Response) {
        let own = &tick.snapshot.own_units;
        if self.pianist.as_ref().is_none_or(|p| p.log.is_none()) {
            return;
        }
        let groups = self.groups_json(own);
        let places = self.places_json();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let mut state = request.state.clone();
        let instructions = state["instructions"].as_str().unwrap_or_default().to_string();
        let rules = state["rules"].as_str().unwrap_or_default().to_string();
        let rules_changed = rules != pianist.logged_rules;
        if rules_changed {
            pianist.logged_rules = rules.clone();
        }
        if let Some(fields) = state.as_object_mut() {
            fields.remove("instructions");
            fields.remove("rules");
        }
        let changed = instructions != pianist.logged_instructions;
        if changed {
            pianist.logged_instructions = instructions.clone();
        }
        let parties: Vec<serde_json::Value> = pianist.parties.iter().map(|p| json!({ "name": p.name, "ids": p.ids.iter().map(|id| id.0).collect::<Vec<_>>(), "x": p.at.x as i32, "z": p.at.z as i32, "metal": p.metal as i32, "composition": p.composition })).collect();
        let mut line = json!({
            "t": "call", "f": tick.frame, "ms": (response.latency.as_secs_f32() * 1000.0) as u32, "model": response.model, "usage": response.usage,
            "retries": response.retries, "batches": response.batches, "state": state, "questions": request.questions, "answers": response.answers,
            "groups": groups, "places": places, "parties": parties,
        });
        if rules_changed {
            line["rules"] = json!(rules);
        }
        if changed {
            line["instructions"] = json!(instructions);
        }
        pianist.write_log(line);
    }

    /// Tasks and queues against what the engine says: builds started or refused, labs' units begun, units gone;
    /// a changed `produce` allowance restarts an actor's counts against its caps ("armllt:2": two more, then off);
    /// a group whose unit types changed since its tool orders were set is said once (standing-1, 4:26).
    fn pianist_housekeeping(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let frame = tick.frame;
        let mut notes: Vec<String> = Vec::new();
        // What did not happen, for the player's report as well as the picture.
        let mut done: Vec<String> = Vec::new();
        let mut refused: Vec<(UnitId, UnitDefId, Vec3)> = Vec::new();
        let allowances: Vec<(UnitId, String, Option<Allowance>)> = own.iter().filter(|u| !u.being_built && (self.world.is_mobile_builder(u.def) || self.world.is_factory_def(u.def))).map(|u| (u.id, self.actor_name(u.id), self.allowed_units(&self.actor_name(u.id)))).collect();
        let Some(mut pianist) = self.pianist.take() else { return };
        for (id, name, allowed) in allowances {
            if pianist.allowed_seen.get(&name) != allowed.as_ref() {
                pianist.produced.retain(|(b, _), _| *b != id);
                match allowed {
                    Some(a) => pianist.allowed_seen.insert(name, a),
                    None => pianist.allowed_seen.remove(&name),
                };
            }
        }
        pianist.tasks.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.produced_by.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.queued.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.queued_steps.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.stale_queue.retain(|id| pianist.queued.contains_key(id));
        pianist.helping.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.lab_queue.retain(|id, _| own.iter().any(|u| u.id == *id));
        for event in &tick.events {
            match *event {
                Event::UnitCreated { unit, builder: Some(builder) } => {
                    if own.iter().any(|u| u.id == builder && self.world.is_factory_def(u.def)) {
                        pianist.produced_by.insert(unit, builder);
                    }
                    // The frame of a building the player's lists name by an id, begun by the builder that claimed it.
                    if let Some(def) = own.iter().find(|u| u.id == unit).map(|u| u.def) {
                        if let Some(tagged) = pianist.tagged.values_mut().filter(|t| t.claim == Some(builder) && t.building.is_none() && t.def == def).min_by_key(|t| t.claimed) {
                            tagged.building = Some(unit);
                        }
                    }
                    // A frame appearing while the task is a started build is the queued build beginning (the
                    // finished event promoted it already, unless the frame in progress died): promote now.
                    if matches!(pianist.tasks.get(&builder), Some(Task::Build { started: true, .. })) {
                        commands.extend(pianist.take_queued(builder, &self.actor_name(builder), frame));
                    }
                    // The frame stands where the engine put it, up to a building's width from the point ordered
                    // (pianist-player-6: two windmills 200 from their ordered point were not seen as started, and
                    // the next "generator" answer ordered a third; both decayed). The task's site is the frame.
                    if let Some(Task::Build { started, near, def, .. }) = pianist.tasks.get_mut(&builder) {
                        if !*started {
                            // The builder's count against its `produce` caps ("armllt:2" from this builder).
                            *pianist.produced.entry((builder, self.world.def(*def).map_or(String::new(), |d| d.name.clone()))).or_insert(0) += 1;
                        }
                        *started = true;
                        if let Some(frame_unit) = own.iter().find(|u| u.id == unit) {
                            *near = frame_unit.pos;
                        }
                    }
                    if let Some(queue) = pianist.lab_queue.get_mut(&builder) {
                        let made = own.iter().find(|u| u.id == unit).map(|u| u.def);
                        if let Some(def) = made {
                            *pianist.produced.entry((builder, self.world.def(def).map_or(String::new(), |d| d.name.clone()))).or_insert(0) += 1;
                        }
                        if let Some(i) = queue.iter().position(|(def, _)| Some(*def) == made) {
                            queue.remove(i);
                        } else if !queue.is_empty() {
                            queue.remove(0);
                        }
                    }
                }
                Event::UnitFinished { unit } => {
                    // The builder whose started build this was moves on to what it queued (H-HANDS-QUEUE), or is
                    // free for the pass.
                    if let Some(done) = own.iter().find(|u| u.id == unit) {
                        let builders: Vec<UnitId> = pianist
                            .tasks
                            .iter()
                            .filter(|(_, t)| matches!(t, Task::Build { def, near, started: true, .. } if *def == done.def && near.dist2d(done.pos) < 250.0))
                            .map(|(id, _)| *id)
                            .collect();
                        for builder in builders {
                            if pianist.queued.contains_key(&builder) {
                                commands.extend(pianist.take_queued(builder, &self.actor_name(builder), frame));
                            } else {
                                pianist.tasks.remove(&builder);
                            }
                        }
                    }
                }
                Event::UnitDestroyed { unit, attacker } => {
                    if let Some((def, pos)) = self.known_units.get(&unit) {
                        let building = self.world.def(*def).is_some_and(|d| d.speed == 0.0 || *def == kit.commander || d.build_speed > 0.0);
                        if let Some(share) = self.abandoned(unit, attacker) {
                            let text = format!("abandoned an unfinished {} at {} ({:.0}% built): its builder was sent elsewhere and the frame decayed", self.name(*def), self.world.grid(*pos), share * 100.0);
                            done.push(format!("{} {text}", picture::clock(frame)));
                            notes.push(text);
                        } else if building {
                            let killer = attacker.and_then(|id| self.enemy_defs.get(&id)).map_or("something unseen".to_string(), |d| self.name(*d).to_string());
                            notes.push(format!("lost our {} at {} to {killer}", self.name(*def), self.world.grid(*pos)));
                        }
                    }
                }
                Event::EnemyDestroyed { enemy } => {
                    if let Some(def) = self.enemy_defs.get(&enemy) {
                        notes.push(format!("killed their {}", self.name(*def)));
                    }
                }
                _ => {}
            }
        }
        // A build the engine never started: the builder is idle again after the order's grace with no nanoframe.
        for unit in own.iter().filter(|u| u.idle && !u.being_built) {
            match pianist.tasks.get(&unit.id) {
                Some(Task::Build { def, near, ordered, started: false, .. }) if frame - ordered > super::economy::ORDER_GRACE_FRAMES => {
                    refused.push((unit.id, *def, *near));
                }
                Some(Task::Build { started: true, ordered, .. }) if frame - ordered > super::economy::ORDER_GRACE_FRAMES => {
                    pianist.tasks.remove(&unit.id);
                }
                Some(Task::Reclaim { since, .. }) | Some(Task::ReclaimUnit { since, .. }) | Some(Task::Repair { since, .. }) if frame - since > super::economy::ORDER_GRACE_FRAMES => {
                    pianist.tasks.remove(&unit.id);
                }
                // A walk ends where the builder arrives. Standing idle short of the place, the engine has dropped
                // the order: that is the walk's end too, and it is said (the clocks that ended a walk after 40 s
                // and an attack after 10 s are gone: the wait is words in the builder's entry).
                Some(Task::Walk { to, .. }) if unit.pos.dist2d(*to) < 150.0 => {
                    pianist.tasks.remove(&unit.id);
                }
                Some(Task::Walk { to, since, place }) if frame - since > super::economy::ORDER_GRACE_FRAMES => {
                    done.push(format!("{} {}: its walk to {place} ended {:.0} short of it: the engine gave the order up", picture::clock(frame), self.actor_name(unit.id), unit.pos.dist2d(*to)));
                    pianist.tasks.remove(&unit.id);
                }
                _ => {}
            }
        }
        // A builder attacking a party follows it: the fight order is to where the party stood when the pick was made,
        // and a raider moves. A builder following a group goes after it when the group has moved on.
        let bodies: Vec<(String, Vec3)> = pianist.groups.iter().filter_map(|g| g.body(own, None).map(|b| (g.name.clone(), b.at))).collect();
        let mut moved: Vec<(UnitId, Command)> = Vec::new();
        let mut left: Vec<UnitId> = Vec::new();
        for (id, task) in pianist.tasks.iter_mut() {
            match task {
                Task::Attack { party, to, .. } => {
                    if let Some(p) = pianist.parties.iter().find(|p| p.name == *party).filter(|p| p.at.dist2d(*to) > 150.0) {
                        *to = p.at;
                        moved.push((*id, Command::Fight { unit: *id, to: p.at, queue: false }));
                    }
                }
                Task::Follow { group, at, .. } => match bodies.iter().find(|(name, _)| name == group) {
                    Some((_, body)) => {
                        if let Some(to) = groups::follow_step(*body, *at) {
                            *at = to;
                            moved.push((*id, Command::Move { unit: *id, to, queue: false }));
                        }
                    }
                    None => left.push(*id),
                },
                _ => {}
            }
        }
        commands.extend(moved.into_iter().map(|(_, command)| command));
        for id in left {
            if let Some(Task::Follow { group, .. }) = pianist.tasks.remove(&id) {
                commands.push(Command::Stop { unit: id });
                done.push(format!("{} {}: group_{group}, which it followed, is gone; it stops where it is", picture::clock(frame), self.actor_name(id)));
            }
        }
        // A party that is gone (dead or out of sight) ends the attack at once: the builder stops where it is and
        // the pass sees it free this second, instead of walking to where the party was and idling there
        // (onepass-norules-hard-3, 2:55: the Flea died the second the pick sent the commander, which walked 18 s to
        // the spot and stood 16 s).
        let gone: Vec<UnitId> = pianist
            .tasks
            .iter()
            .filter(|(_, t)| matches!(t, Task::Attack { party, .. } if !pianist.parties.iter().any(|p| p.name == *party)))
            .map(|(id, _)| *id)
            .collect();
        for id in gone {
            pianist.tasks.remove(&id);
            commands.push(Command::Stop { unit: id });
            done.push(format!("{} {}: its attack ends, the party is gone", picture::clock(frame), self.actor_name(id)));
        }
        for text in notes {
            pianist.note(frame, text);
        }
        pianist.done.append(&mut done);
        self.pianist = Some(pianist);
        for (unit, def, near) in refused {
            self.dropped_orders += 1;
            // An extractor refused off its centre: that spot takes the exact centre from now on (as the ladder does).
            if def == kit.extractor
                && let Some(i) = self.world.hello.metal_spots.iter().position(|s| s.dist2d(near) < super::economy::MEX_PATCH + 1.0 && s.dist2d(near) > 1.0)
            {
                self.centre_only.insert(i);
            }
            let name = self.name(def).to_string();
            let refused_spot = (def == kit.extractor).then(|| self.world.hello.metal_spots.iter().position(|s| s.dist2d(near) < super::economy::MEX_PATCH + 1.0)).flatten();
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.tasks.remove(&unit);
            if let Some(i) = refused_spot {
                pianist.refused_spots.insert(i, frame + REFUSED_FRAMES);
            } else {
                pianist.refused_sites.retain(|(_, _, until)| *until > frame);
                pianist.refused_sites.push((def, near, frame + REFUSED_FRAMES));
            }
            let text = format!("the engine refused a {name} at {}: the site was bad", self.world.grid(near));
            let actor = self.actor_name(unit);
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.done.push(format!("{} {actor}: {text}", picture::clock(frame)));
            pianist.note(frame, text);
            eprintln!("[ai {}] f={frame} pianist: {name} order for unit {} near ({:.0}, {:.0}) never started", self.world.hello.ai_id, unit.0, near.x, near.z);
        }
    }

    /// What the player's session reads (`Shared.hands`): the picture without the instructions and rules, the `did`
    /// lines since its last turn, the groups that began an engagement this second, the standing orders in force.
    fn publish_hands(&mut self, picture: &picture::Picture) {
        let Some(shared) = &self.strategist else { return };
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let mut state = picture.state.clone();
        if let Some(fields) = state.as_object_mut() {
            fields.remove("instructions");
            fields.remove("rules");
        }
        // For the player alone: the two moves the gate last rated best for each actor, so that a paragraph that is
        // not being read as meant shows within a turn.
        for (actor, moves) in &pianist.rated {
            if let Some(entry) = state["actors"].get_mut(actor).filter(|e| e.is_object()) {
                entry["best_rated"] = json!(moves.iter().map(|(said, p)| format!("{said} ({p:.2})")).collect::<Vec<_>>().join(", "));
            }
        }
        // And, with the `fuse` layer on, what the decode read of the packet for each group.
        if pianist.layers.fuse && let Some(actors) = state["actors"].as_object_mut() {
            for (actor, entry) in actors.iter_mut().filter(|(name, e)| name.starts_with("group_") && e.is_object()) {
                if let Some(words) = decode::reads_words(&pianist.decode, actor) {
                    entry["reads"] = json!(words);
                }
            }
        }
        let mut all = shared.hands.lock().unwrap();
        let hands = all.entry(self.world.hello.team).or_default();
        hands.picture = state;
        hands.done.append(&mut pianist.done);
        hands.engaged = pianist.played.iter().filter(|p| p["did"].as_str().is_some_and(|d| d.starts_with("attack "))).filter_map(|p| p["actor"].as_str().map(str::to_string)).collect();
    }

    fn pianist_status_line(&mut self, frame: i32) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let stats = std::mem::take(&mut pianist.stats);
        if stats.calls == 0 && stats.quiet == 0 {
            return;
        }
        let mut latencies = stats.latencies_ms;
        latencies.sort_by(f32::total_cmp);
        let median = latencies.get(latencies.len() / 2).copied().unwrap_or(0.0);
        let max = latencies.last().copied().unwrap_or(0.0);
        eprintln!(
            "[ai {}] f={frame} pianist this minute: {} calls ({} failed, {} dropped), {} quiet seconds, median {median:.0} ms, longest {max:.0}, {} tokens in, {} questions; groups {}, tasks {}",
            self.world.hello.ai_id, stats.calls, stats.errors, stats.dropped, stats.quiet, stats.tokens, stats.questions, pianist.groups.len(), pianist.tasks.len()
        );
    }
}
