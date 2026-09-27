//! The pianist (`docs/design/2026-09-21-pianist.md`): Opus plays the game, Jev plays the keyboard. Once a game
//! second the brain writes a picture of the game (`picture.rs`), enumerates every actor's executable states
//! (`plan.rs`, `threats.rs`), puts the base world in force, and when the picture changed asks Jev twice: the
//! pre-pass's nouls, then one Choice over the composed worlds (`docs/design/2026-09-26-one-pass.md`). The pick runs
//! through the actuators (`execute.rs`, `groups.rs`) until the picture changes again. No decision heuristic runs
//! beneath it; the control lane (`micro.rs`) and the tracking do.
//!
//! In lockstep the calls hold the game (the arena's shim waits for the reply); against a live engine they cost a
//! late tick a second. A call fails safe: every actor keeps its task until the next answer.

pub mod glossary;
pub(super) mod groups;
mod execute;
mod lists;
pub(super) mod picture;
mod remove;
mod diet;
mod plan;
mod schedule;
pub(crate) mod standing;
mod threats;

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
pub(crate) use standing::Standing;

/// Game seconds between calls (`WITHIN_REASON_JEV_INTERVAL` overrides).
const INTERVAL_SECONDS: f32 = 1.0;
/// Events the picture shows, and for how long.
const RECENT_EVENTS: usize = 12;
const RECENT_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
/// H-HANDS-REFUSED: a spot where the engine refused an extractor is left off every state for this long (smoke-4: the
/// commander asked for the same refused spot thirty times running beside the enemy base, and died there).
const REFUSED_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
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
}

impl Task {
    pub(super) fn since(&self) -> i32 {
        match self {
            Task::Build { ordered, .. } => *ordered,
            Task::Assist { since, .. } | Task::Reclaim { since, .. } | Task::ReclaimUnit { since, .. } | Task::Repair { since, .. } | Task::Walk { since, .. } => *since,
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
    /// Standing orders (`standing.rs`, `docs/design/2026-09-25-standing-orders.md`): the `standing` tool's rules;
    /// the pass's pruning and defaults.
    pub(super) standing: Standing,
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
    /// Worlds in the second call at most (`WITHIN_REASON_WORLDS`).
    cap: usize,
    /// The told noul beside each builder state's move noul (`WITHIN_REASON_TOLD=1`; `plan::TOLD_BAR`).
    told: bool,
    /// Builders' and labs' tasks, by unit.
    pub(super) tasks: HashMap<UnitId, Task>,
    /// A builder's next task, already ordered behind the one in progress (H-HANDS-QUEUE): it becomes the task when
    /// the frame in progress is finished, or when the engine starts it.
    pub(super) queued: HashMap<UnitId, Task>,
    /// The player's lists of steps per builder (by actor name), done by the bot without asking (H-HANDS-SCRIPT).
    pub(super) scripts: HashMap<String, VecDeque<String>>,
    /// What the hands have ordered each builder to build, in order, by unit name: a list arriving after the hands
    /// began the opening skips the leading steps already ordered.
    pub(super) ordered: HashMap<UnitId, Vec<String>>,
    /// The list step each builder is on (its words, and the clock of the task it became): diverted from that task
    /// by the pass, the builder gets the step back at the front of its list.
    pub(super) list_steps: HashMap<UnitId, (String, i32)>,
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
    /// The pass (`plan.rs`): the slots of the last ask, the worlds of its second call, the picture's signature and
    /// frame at the last ask (ask on change), and the events since (`schedule.rs`).
    pub(super) slots: Vec<plan::Slot>,
    pub(super) worlds: Vec<plan::World>,
    pub(super) sig: Option<(String, i32)>,
    pub(super) events: BTreeSet<String>,
    /// The hunts' ends since the last pass line (`groups.rs` `tick_hunt`), for the log.
    pub(super) hunt_events: Vec<String>,
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
    /// Our units hit since the last call, with the frame: the pass's under-fire set, across the ticks between calls.
    pub(super) hits: HashMap<UnitId, i32>,
    /// When each builder last went home from an enemy (any source): its building defaults hold off for
    /// `RETREAT_HOLD` after (standing-2: sent home and back to the same extractor the next second, 11 times).
    pub(super) retreated: HashMap<UnitId, i32>,
    /// When each actor's course was last set by a pick: a rule default does not displace it for `PICK_HOLD`.
    pub(super) picked: HashMap<String, i32>,
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
    /// The versioned model has been said in the game chat (once, after the first answer).
    announced: bool,
}

/// The pianist's log format version (`docs/harness/record-format.md`).
const LOG_VERSION: u32 = 2;
/// Realtime: an answer older than this judges a picture too old to play.
const STALE_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// An event alone (a hit, a sighting, an alarm) asks again no oftener than this.
const EVENT_GAP: i32 = 5 * FRAMES_PER_SECOND;

/// The thread that talks to Jev in real time: requests in, answers out, each with the request's number. A gate
/// request carries what the worker needs to compose the worlds and make the pick the instant the gate answers
/// (the user, 2026-09-27: the two calls as fast as they can be made), so the pick is not held for the next tick.
struct Worker {
    to: std::sync::mpsc::Sender<(u64, jev::Request, Option<Follow>)>,
    from: std::sync::mpsc::Receiver<(u64, Result<jev::Response, jev::Error>, Option<Followed>)>,
}

/// What composing the worlds after a gate takes, sent along with the gate request.
struct Follow {
    slots: Vec<plan::Slot>,
    store: String,
    cap: usize,
}

/// The worker's follow-up to a gate: what the gate flagged, the worlds and their lines (None when nothing opened),
/// and the pick's request with its answer.
struct Followed {
    flags: BTreeMap<String, f64>,
    worlds: Option<Vec<plan::World>>,
    lines: Vec<String>,
    pick: Option<(jev::Request, Result<jev::Response, jev::Error>)>,
}

/// A request in flight: what it was built from, so its answer can be played when it comes. `worlds` is None for
/// the pre-pass and the composed worlds for the pick.
struct Pending {
    id: u64,
    frame: i32,
    picture: picture::Picture,
    slots: Vec<plan::Slot>,
    worlds: Option<Vec<plan::World>>,
    request: jev::Request,
}

fn spawn_worker(client: jev::Client) -> Worker {
    let (to, requests) = std::sync::mpsc::channel::<(u64, jev::Request, Option<Follow>)>();
    let (answers, from) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        while let Ok((mut id, mut request, mut follow)) = requests.recv() {
            // Behind by a slow call, the worker answers the newest request only: answered in order, a burst of
            // one-second calls made every later answer stale (human-1, second game: three of 35 dropped).
            while let Ok((newer_id, newer, newer_follow)) = requests.try_recv() {
                (id, request, follow) = (newer_id, newer, newer_follow);
            }
            let result = client.ask(&request);
            let followed = match (&result, follow) {
                (Ok(response), Some(f)) => {
                    let (flags, next) = plan::follow_up(&f.slots, &response.answers, &f.store, f.cap, &request.state);
                    let (worlds, lines, pick) = match next {
                        Some((worlds, lines, second)) => {
                            let answer = client.ask(&second);
                            (Some(worlds), lines, Some((second, answer)))
                        }
                        None => (None, Vec::new(), None),
                    };
                    Some(Followed { flags, worlds, lines, pick })
                }
                _ => None,
            };
            if answers.send((id, result, followed)).is_err() {
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

impl Pianist {
    /// The builder's queued task becomes its task, its clock starting now. Helping the lab is the one task the engine
    /// could not hold queued (the guard order has no queue flag), so it is ordered here, as the build finishes.
    pub(super) fn promote(&mut self, builder: UnitId, frame: i32) -> Option<Command> {
        let mut next = self.queued.remove(&builder)?;
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
            Task::Reclaim { since, .. } | Task::Repair { since, .. } | Task::Walk { since, .. } => *since = frame,
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
        let cap = std::env::var("WITHIN_REASON_WORLDS").ok().and_then(|v| v.parse::<usize>().ok()).filter(|n| *n >= 2).unwrap_or(plan::CAP);
        let told = std::env::var("WITHIN_REASON_TOLD").ok().is_some_and(|v| matches!(v.trim(), "1" | "on" | "yes"));
        Ok(Pianist {
            client,
            standing: Standing::default(),
            packet,
            worker,
            pending: None,
            next_request: 0,
            interval_frames: ((seconds * FRAMES_PER_SECOND as f32) as i32).max(super::BRAIN_FRAMES),
            last_ask_frame: i32::MIN / 2,
            cap,
            told,
            tasks: HashMap::new(),
            queued: HashMap::new(),
            lab_queue: HashMap::new(),
            groups: Vec::new(),
            next_group: 0,
            seat_tag: String::new(),
            produced_by: HashMap::new(),
            rally: HashMap::new(),
            slots: Vec::new(),
            worlds: Vec::new(),
            sig: None,
            events: BTreeSet::new(),
            hunt_events: Vec::new(),
            refused_spots: HashMap::new(),
            refused_sites: Vec::new(),
            script_frame: HashMap::new(),
            packet_frame: 0,
            diet: diet::Diet::from_env(),
            hits: HashMap::new(),
            retreated: HashMap::new(),
            picked: HashMap::new(),
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
            ordered: HashMap::new(),
            list_steps: HashMap::new(),
            done: Vec::new(),
            log,
            logged_instructions: String::new(),
            logged_rules: String::new(),
            played: Vec::new(),
            stats: Stats::default(),
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
                "interval_frames": self.interval_frames, "rules": rules, "hands_effort": self.diet.level_name(), "worlds_cap": self.cap,
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
        }
        if tick.frame < FIRST_ORDER_FRAME {
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
        self.take_lists(tick, kit, commands);
        self.apply_standing_changes(tick);
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
            pianist.places = picture.places.clone();
            pianist.parties = picture.parties.clone();
        }
        self.play_lists(tick, kit, &picture, commands);
        self.pass(tick, kit, &picture, commands);
        self.publish_hands(&picture);
        let status_due = tick.due() % (60 * FRAMES_PER_SECOND) < self.pianist.as_ref().expect("pianist mode").interval_frames;
        if status_due {
            self.pianist_status_line(tick.frame);
        }
    }

    /// The player's `queue` calls since the last pass: lists set or cancelled per builder.
    fn take_lists(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
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
                    pianist.done.push(format!("{} {name}: stopped what it was doing on your `stop`", picture::clock(tick.frame)));
                }
                if s.is_empty() {
                    steps = None;
                }
            }
            // The first list of the game for a builder skips the leading steps the hands' default opening has
            // already built (a step matches a build by kind: `extractor` the extractor, else the unit's name).
            if let Some(s) = steps.as_mut()
                && !self.pianist.as_ref().expect("pianist mode").script_frame.contains_key(&name)
                && let Some(unit) = self.unit_by_handle(&name, &tick.snapshot.own_units).map(|u| u.id)
            {
                let extractor = self.world.def(kit.extractor).map(|d| d.name.clone()).unwrap_or_default();
                let ordered = self.pianist.as_ref().expect("pianist mode").ordered.get(&unit).cloned().unwrap_or_default();
                let matches = |step: &String, name: &String| {
                    let kind = step.split_whitespace().next().unwrap_or_default();
                    (kind == "extractor" && *name == extractor) || kind == name
                };
                let k = s.iter().zip(ordered.iter()).take_while(|(step, name)| matches(step, name)).count();
                if k > 0 {
                    let skipped: Vec<String> = s.drain(..k).collect();
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.done.push(format!("{} {name}: its list arrived after the hands had built its first {k} steps ({}); it goes on from the next", picture::clock(tick.frame), skipped.join(", ")));
                }
                if s.is_empty() {
                    steps = None;
                }
            }
            let pianist = self.pianist.as_mut().expect("pianist mode");
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

    /// The pass (`plan.rs`): the slots this second, the base world put in force at once, and when something is
    /// open and the picture changed since the last ask (or `RE_ASK` passed), the pre-pass's nouls sent; the worlds
    /// question follows in `after_gate`. Without Jev the base is the plan.
    fn pass(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let slots = self.slots(tick, kit, picture);
        let mut line = json!({ "t": "pass", "f": frame });
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            // A party with a threat slot this second and none last second: said among the events, so the log
            // shows what opened the ask (onepass-hard-4, 5:04: the reader could not tell why group_A moved).
            let threat = |slots: &[plan::Slot]| -> BTreeSet<String> { slots.iter().filter_map(|s| match &s.kind { plan::Kind::Threat(p, _) => Some(p.name.clone()), _ => None }).collect() };
            let before = threat(&pianist.slots);
            for name in threat(&slots).difference(&before) {
                pianist.events.insert(format!("{name} appeared"));
            }
            if !pianist.hunt_events.is_empty() {
                line["hunts"] = json!(std::mem::take(&mut pianist.hunt_events));
            }
        }
        if slots.is_empty() {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            line["played"] = json!(std::mem::take(&mut pianist.played));
            if !line["played"].as_array().is_some_and(Vec::is_empty) || line.get("hunts").is_some() {
                pianist.write_log(line);
            }
            return;
        }
        let mut base: plan::World = slots.iter().map(plan::Slot::base).collect();
        plan::hold_current(&slots, &mut base);
        plan::resolve(&slots, &mut base);
        let open: Vec<&str> = slots.iter().filter(|s| s.open()).map(|s| s.name.as_str()).collect();
        line["open"] = json!(open);
        let questions = plan::gate_questions(&slots, self.pianist.as_ref().is_some_and(|p| p.told));
        // The economy in the signature at its extremes only: the stock words' five buckets flapped at their edges
        // (onepass-medium-3: 58 of 663 asks).
        // ... and whether the store now covers the cheapest unit an idle lab could make: the store crossing that cost
        // is what an idle lab waits for (onepass-norules-hard-4: each Stout cycle ran 5 s of building and 21 s of
        // waiting, the pick having kept w1 at 138 metal and the pass staying quiet for the 20 s re-ask).
        let idle_lab_afford = slots.iter().any(|s| {
            matches!(s.kind, plan::Kind::Lab(_))
                && s.base() == 0
                && s.states.iter().filter_map(|st| if let plan::Response::Next(def) = &st.response { self.world.def(*def).map(|d| d.metal_cost) } else { None }).fold(f32::INFINITY, f32::min) <= tick.snapshot.metal.current
        });
        let eco = format!("{}|{}|{}", if tick.snapshot.metal.current < 100.0 { "empty" } else if tick.snapshot.metal.current >= tick.snapshot.metal.storage - 1.0 { "full" } else { "" }, picture.state["economy"]["energy"].as_str().is_some_and(|e| e.contains("STALLING")), idle_lab_afford);
        let sig = format!("{}|{eco}|{}", plan::signature(&slots), self.pianist.as_ref().expect("pianist mode").packet_frame);
        let jev = self.pianist.as_ref().is_some_and(|p| p.client.is_some());
        let (events, changed) = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            let events: Vec<String> = pianist.events.iter().cloned().collect();
            // An event alone asks again no oftener than `EVENT_GAP` (onepass-medium-3: a group under fire asked every
            // second, 118 of 663 asks on "hit" alone).
            let since_ask = pianist.sig.as_ref().map_or(i32::MAX, |(_, f)| frame - *f);
            let changed = pianist.sig.as_ref().is_none_or(|(s, f)| *s != sig || frame - *f >= plan::RE_ASK) || (!events.is_empty() && since_ask >= EVENT_GAP);
            (events, changed)
        };
        // The base world goes in force now only where there is nothing to decide. A slot with a real alternative
        // waits for the pick when a question goes out this second (the user, 2026-09-27: "I never meant for things
        // to fire before jev calls it"; onepass-player-8 7:13-7:38, the rule's whole-group attack fired the second
        // the party appeared, the pick's hunt stacked on it, and the group flipped between the two). Without Jev, or
        // when the picture is as at the last ask, the base is the plan and stands.
        let asking = jev && !questions.is_empty() && changed;
        let held: Vec<bool> = slots.iter().map(|s| asking && s.open()).collect();
        let started = self.apply_plan(tick, kit, picture, &slots, &base, "rule", &held, commands);
        if !started.is_empty() {
            line["plan"] = json!(started);
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if questions.is_empty() || !jev {
            line["played"] = json!(std::mem::take(&mut pianist.played));
            pianist.write_log(line);
            return;
        }
        if !changed {
            line["quiet"] = json!("the picture is as at the last ask: the plan stands");
            pianist.stats.quiet += 1;
            line["played"] = json!(std::mem::take(&mut pianist.played));
            pianist.write_log(line);
            return;
        }
        pianist.sig = Some((sig, frame));
        pianist.events.clear();
        pianist.worlds.clear();
        line["slots"] = plan::log_slots(&slots);
        line["gate"] = json!(questions.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>());
        if !events.is_empty() {
            line["events"] = json!(events);
        }
        line["played"] = json!(std::mem::take(&mut pianist.played));
        pianist.write_log(line);
        // The state: the picture trimmed to the asked actors and the places in play.
        let own = &tick.snapshot.own_units;
        let asked: Vec<(String, Option<Vec3>)> = slots
            .iter()
            .filter(|s| s.open())
            .map(|s| {
                let at = match &s.kind {
                    plan::Kind::Builder(id) | plan::Kind::Lab(id) => own.iter().find(|u| u.id == *id).map(|u| u.pos),
                    plan::Kind::Group(g) => pianist.groups.iter().find(|x| x.name == *g).and_then(|x| groups::centre_of(&x.units(own))),
                    plan::Kind::Threat(p, _) => Some(p.at),
                };
                (s.name.clone(), at)
            })
            .collect();
        let diet = pianist.diet.clone();
        let state = self.trim_state(&diet, picture, &asked);
        let request = jev::Request { state, questions: questions.into_iter().collect() };
        if self.pianist.as_ref().expect("pianist mode").stats.calls == 0 && self.pianist.as_ref().expect("pianist mode").logged_instructions.is_empty() {
            let rules = picture.state["rules"].as_str().unwrap_or_default().to_string();
            self.pianist.as_mut().expect("pianist mode").log_header(self.world.hello.ai_id, &rules);
        }
        let follow = self.pianist.as_ref().is_some_and(|p| p.worker.is_some()).then(|| Follow { slots: slots.clone(), store: Self::store_words(picture), cap: self.pianist.as_ref().expect("pianist mode").cap });
        self.call(tick, kit, picture.clone(), slots, None, request, follow, commands);
    }

    /// The store's words for the worlds' lines ("340 of 500 stored").
    fn store_words(picture: &picture::Picture) -> String {
        picture.state["economy"]["metal"].as_str().and_then(|m| m.split(';').next()).unwrap_or_default().to_string()
    }

    /// A call to Jev: through the worker in realtime (its answer lands in `collect_answer`), in place in lockstep.
    #[allow(clippy::too_many_arguments)]
    fn call(&mut self, tick: &Tick, kit: &Kit, picture: picture::Picture, slots: Vec<plan::Slot>, worlds: Option<Vec<plan::World>>, request: jev::Request, follow: Option<Follow>, commands: &mut Vec<Command>) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.stats.calls += 1;
        pianist.stats.questions += request.questions.len() as u32;
        if pianist.worker.is_some() {
            let id = pianist.next_request;
            pianist.next_request += 1;
            if pianist.worker.as_ref().expect("checked").to.send((id, request.clone(), follow)).is_ok() {
                pianist.pending = Some(Pending { id, frame: tick.frame, picture, slots, worlds, request });
            }
            return;
        }
        let result = pianist.client.as_ref().expect("a call needs Jev").ask(&request);
        self.answered(tick, kit, &picture, slots, worlds, &request, result, None, commands);
    }

    /// A call's answer: the pre-pass's composes the worlds and makes the second call; the pick's puts the plan in force.
    #[allow(clippy::too_many_arguments)]
    fn answered(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, slots: Vec<plan::Slot>, worlds: Option<Vec<plan::World>>, request: &jev::Request, result: Result<jev::Response, jev::Error>, followed: Option<Followed>, commands: &mut Vec<Command>) {
        let ai = self.world.hello.ai_id;
        match result {
            Ok(response) => {
                {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
                    pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                }
                self.announce_hands(&response, commands);
                self.log_call(tick, request, &response);
                match worlds {
                    None => self.after_gate(tick, kit, picture, slots, request, &response.answers, followed, commands),
                    Some(ws) => self.after_pick(tick, kit, picture, &slots, &ws, &response.answers, commands),
                }
            }
            Err(e) => {
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.stats.errors += 1;
                pianist.write_log(json!({ "t": "error", "f": tick.frame, "error": e.to_string() }));
                eprintln!("[ai {ai}] f={} pianist: {e}; every actor keeps its course", tick.frame);
            }
        }
    }

    /// After the pre-pass: the worlds over the flagged states as the second call, or nothing to ask. Logs what the
    /// gate said and the worlds' lines.
    #[allow(clippy::too_many_arguments)]
    fn after_gate(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, slots: Vec<plan::Slot>, request: &jev::Request, answers: &BTreeMap<String, jev::Answer>, followed: Option<Followed>, commands: &mut Vec<Command>) {
        let cap = self.pianist.as_ref().expect("pianist mode").cap;
        // Realtime: the worker composed the worlds and made the pick as soon as the gate answered; lockstep: here.
        let (flags, composed) = match followed {
            Some(f) => (
                f.flags,
                match (f.worlds, f.pick) {
                    (Some(worlds), Some((second, answer))) => Some((worlds, f.lines, second, Some(answer))),
                    _ => None,
                },
            ),
            None => {
                let (flags, next) = plan::follow_up(&slots, answers, &Self::store_words(picture), cap, &request.state);
                (flags, next.map(|(worlds, lines, second)| (worlds, lines, second, None)))
            }
        };
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.write_log(json!({ "t": "worlds_gate", "f": tick.frame, "flags": flags, "worlds": composed.as_ref().map(|c| c.0.clone()), "lines": composed.as_ref().map(|c| c.1.clone()).unwrap_or_default() }));
        let Some((ws, _, second, answer)) = composed else { return };
        pianist.worlds = ws.clone();
        pianist.slots = slots.clone();
        match answer {
            Some(result) => self.answered(tick, kit, picture, slots, Some(ws), &second, result, None, commands),
            None => self.call(tick, kit, picture.clone(), slots, Some(ws), second, None, commands),
        }
    }

    /// The second call's pick put in force, and logged as a `plan` line.
    #[allow(clippy::too_many_arguments)]
    fn after_pick(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, slots: &[plan::Slot], worlds: &[plan::World], answers: &BTreeMap<String, jev::Answer>, commands: &mut Vec<Command>) {
        let Some((wi, confidence)) = plan::pick(answers, worlds) else { return };
        let changed = self.apply_plan(tick, kit, picture, slots, &worlds[wi], "plan", &[], commands);
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let played = std::mem::take(&mut pianist.played);
        pianist.write_log(json!({ "t": "plan", "f": tick.frame, "pick": wi + 1, "confidence": confidence, "changed": changed, "played": played }));
        self.journal.note_from("plan", tick.frame, "worlds", json!({ "worlds": worlds.len() }), json!({ "pick": wi + 1, "confidence": confidence, "changed": changed }));
    }

    /// The player's `standing` calls since the last pass: orders set or cleared, checked against the picture.
    fn apply_standing_changes(&mut self, tick: &Tick) {
        let frame = tick.frame;
        let Some(shared) = &self.strategist else { return };
        let team = self.world.hello.team;
        // This seat's slice of the list, then the entries every seat has applied are dropped.
        let changes: Vec<crate::strategist::shared::StandingChange> = {
            let mut all = shared.standing.lock().unwrap();
            let mut seen = shared.standing_seen.lock().unwrap();
            let from = seen.get(&team).copied().unwrap_or(0).min(all.len());
            let mine: Vec<_> = all[from..].to_vec();
            seen.insert(team, all.len());
            if seen.len() >= self.seats_of_ours() {
                let done = seen.values().copied().min().unwrap_or(0).min(all.len());
                if done > 0 {
                    all.drain(..done);
                    for c in seen.values_mut() {
                        *c -= done.min(*c);
                    }
                }
            }
            mine
        };
        if changes.is_empty() {
            return;
        }
        // Which actors of the change are this seat's: its commander (or a plain `commander` for every seat's),
        // `constructors`, its own constructors and its own groups; another seat's are left to that seat.
        let own_units = &tick.snapshot.own_units;
        let commander = self.commander_handle();
        let group_names: Vec<String> = self.pianist.as_ref().expect("pianist mode").groups.iter().map(|g| format!("group_{}", g.name)).collect();
        let mine = |actor: &str| -> bool {
            actor == commander
                || actor == "commander"
                || actor == "constructors"
                || (actor.starts_with("group_") && (self.seat_tag().is_empty() || group_names.iter().any(|g| g == actor)))
                || (!actor.starts_with("group_") && !actor.starts_with("commander") && self.unit_by_handle(actor, own_units).is_some())
        };
        let changes: Vec<crate::strategist::shared::StandingChange> = changes
            .into_iter()
            .filter_map(|change| match change {
                crate::strategist::shared::StandingChange::Set(map) => {
                    let map: BTreeMap<String, serde_json::Value> = map.into_iter().filter(|(a, _)| mine(a)).collect();
                    (!map.is_empty()).then_some(crate::strategist::shared::StandingChange::Set(map))
                }
                crate::strategist::shared::StandingChange::Clear(Some(actors)) => {
                    let actors: Vec<String> = actors.into_iter().filter(|a| mine(a)).collect();
                    (!actors.is_empty()).then_some(crate::strategist::shared::StandingChange::Clear(Some(actors)))
                }
                all => Some(all),
            })
            .collect();
        if changes.is_empty() {
            return;
        }
        // The marks too: a mark and a rule naming it come in one turn, before the picture has the mark (worlds-1,
        // 14:15: "marks refused in standing"); and every spot our bots can walk to, listed in the picture or not.
        let mut places: Vec<String> = self.pianist.as_ref().expect("pianist mode").places.iter().map(|p| p.name.clone()).collect();
        places.extend(shared.marks.lock().unwrap().keys().cloned());
        places.extend(self.world.hello.metal_spots.iter().enumerate().filter(|(_, s)| self.reachable_on_foot(**s)).map(|(i, _)| format!("spot_{i}")));
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let parties: Vec<String> = pianist.parties.iter().map(|p| p.name.clone()).collect();
        for change in changes {
            let said = match change {
                crate::strategist::shared::StandingChange::Set(map) => {
                    let mut said = Vec::new();
                    for (actor, rules) in &map {
                        said.push(match pianist.standing.set_tool(actor, rules, &places, &parties) {
                            Ok((n, refused)) if refused.is_empty() => format!("{actor}: {n} rules set"),
                            Ok((n, refused)) => format!("{actor}: {n} rules set; refused: {}", refused.join("; ")),
                            Err(e) => format!("refused: {e}"),
                        });
                    }
                    said.join("; ")
                }
                crate::strategist::shared::StandingChange::Clear(actors) => format!("{} tool orders cleared", pianist.standing.clear_tool(actors.as_deref())),
            };
            pianist.done.push(format!("{} standing: {said}", picture::clock(frame)));
            pianist.events.insert("rules".to_string());
        }
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
                    GroupTask::Hold { .. } => json!({ "kind": "hold" }),
                    GroupTask::Move { to, place, fight, .. } => json!({ "kind": if *fight { "fight_to" } else { "move_to" }, "place": place, "to": [to.x as i32, to.z as i32] }),
                    GroupTask::Engage { at, target, .. } => json!({ "kind": if target.is_some() { "attack_unit" } else { "engage" }, "to": [at.x as i32, at.z as i32] }),
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
        let ai = self.world.hello.ai_id;
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
                        eprintln!("[ai {ai}] f={} pianist: the answer to the request of frame {} has not come in {} s; dropped", tick.frame, p.frame, STALE_FRAMES / FRAMES_PER_SECOND);
                        pianist.pending = None;
                    }
                    None
                }
            }
        };
        let Some((pending, result, followed)) = arrived else { return };
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.places = pending.picture.places.clone();
            pianist.parties = pending.picture.parties.clone();
        }
        self.answered(tick, kit, &pending.picture, pending.slots, pending.worlds, &pending.request, result, followed, commands);
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
            "retries": response.retries, "state": state, "questions": request.questions, "answers": response.answers,
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
        let types: Vec<(String, BTreeSet<String>)> = self.pianist.as_ref().map(|p| p.groups.iter().map(|g| (format!("group_{}", g.name), g.units(own).iter().map(|u| self.name(u.def).to_string()).collect::<BTreeSet<_>>())).collect()).unwrap_or_default();
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
        for (name, types) in types {
            if let Some(text) = pianist.standing.composition_changed(&name, types) {
                done.push(format!("{} standing: {text}", picture::clock(frame)));
            }
        }
        pianist.tasks.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.produced_by.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.queued.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.lab_queue.retain(|id, _| own.iter().any(|u| u.id == *id));
        for event in &tick.events {
            match *event {
                Event::UnitCreated { unit, builder: Some(builder) } => {
                    if own.iter().any(|u| u.id == builder && self.world.is_factory_def(u.def)) {
                        pianist.produced_by.insert(unit, builder);
                    }
                    // A frame appearing while the task is a started build is the queued build beginning (the
                    // finished event promoted it already, unless the frame in progress died): promote now.
                    if matches!(pianist.tasks.get(&builder), Some(Task::Build { started: true, .. })) {
                        commands.extend(pianist.promote(builder, frame));
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
                                commands.extend(pianist.promote(builder, frame));
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
                Some(Task::Walk { to, since, place }) if unit.pos.dist2d(*to) < 150.0 || frame - since > 40 * FRAMES_PER_SECOND || (place.starts_with("party_") && frame - since > 10 * FRAMES_PER_SECOND) => {
                    pianist.tasks.remove(&unit.id);
                }
                _ => {}
            }
        }
        // A walker the engine has given up on (`yards.rs` `stuck`) for 20 s is not walking: its task goes, and the
        // pass sees it free (onepass-medium-1: three constructors "walking to home" for six minutes wedged 440 from it).
        let wedged: Vec<UnitId> = pianist.tasks.iter().filter(|(id, t)| matches!(t, Task::Walk { .. }) && self.stuck.get(id).is_some_and(|s| frame - s.since > 20 * FRAMES_PER_SECOND)).map(|(id, _)| *id).collect();
        for id in wedged {
            pianist.tasks.remove(&id);
            commands.push(Command::Stop { unit: id });
        }
        // A builder attacking a party follows it: the fight order is to where the party stood when the pick was made,
        // and a raider moves (the state stays current, so the pass does not re-issue it).
        let moved: Vec<(UnitId, Vec3)> = pianist
            .tasks
            .iter()
            .filter_map(|(id, t)| match t {
                Task::Walk { to, place, .. } if place.starts_with("party_") => pianist.parties.iter().find(|p| p.name == *place).filter(|p| p.at.dist2d(*to) > 150.0).map(|p| (*id, p.at)),
                _ => None,
            })
            .collect();
        for (id, at) in moved {
            if let Some(Task::Walk { to, .. }) = pianist.tasks.get_mut(&id) {
                *to = at;
            }
            commands.push(Command::Fight { unit: id, to: at, queue: false });
        }
        // A party that is gone (dead or out of sight) ends the attack at once: the builder stops where it is and
        // the pass sees it free this second, instead of walking to where the party was and idling there
        // (onepass-norules-hard-3, 2:55: the Flea died the second the pick sent the commander, which walked 18 s to
        // the spot and stood 16 s).
        let gone: Vec<UnitId> = pianist
            .tasks
            .iter()
            .filter(|(_, t)| matches!(t, Task::Walk { place, .. } if place.starts_with("party_") && !pianist.parties.iter().any(|p| p.name == *place)))
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
        let mut all = shared.hands.lock().unwrap();
        let hands = all.entry(self.world.hello.team).or_default();
        hands.picture = state;
        hands.done.append(&mut pianist.done);
        hands.standing_text = pianist.standing.in_force();
        hands.standing_count = pianist.standing.count();
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
