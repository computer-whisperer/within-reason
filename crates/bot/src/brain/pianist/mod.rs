//! The pianist (`docs/design/2026-09-21-pianist.md`): Opus plays the game, Jev plays the keyboard. Once a game
//! second the brain writes a picture of the game (`picture.rs`), offers a menu for every actor that is free or due
//! for review (`menu.rs`), asks Jev once for all of them, and plays the answers through the actuators (`hands.rs`,
//! `groups.rs`). No decision heuristic runs beneath it; the control lane (`micro.rs`) and the tracking do.
//!
//! In lockstep the call holds the game (the arena's shim waits for the reply); against a live engine it costs a
//! late tick a second. The call fails safe: every actor keeps its task until the next answer.

pub mod glossary;
mod groups;
mod hands;
mod menu;
mod picture;
mod remove;
mod diet;
mod schedule;
pub(crate) mod standing;
mod threats;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

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
pub(crate) use standing::{Mode as StandingMode, Standing};

/// Game seconds between calls (`WITHIN_REASON_JEV_INTERVAL` overrides).
const INTERVAL_SECONDS: f32 = 1.0;
/// A busy actor is asked again this often, with "continue" on its menu.
pub(super) const REVIEW_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// H-HANDS-SWITCH: a busy actor changes course only when the winning option beats "continue" by this much probability.
pub(super) const SWITCH_MARGIN: f64 = 0.15;
/// Events the picture shows, and for how long.
const RECENT_EVENTS: usize = 12;
const RECENT_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
/// H-HANDS-REFUSED: a spot where the engine refused an extractor is left off every menu for this long (smoke-4: the
/// commander asked for the same refused spot thirty times running beside the enemy base, and died there).
const REFUSED_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
/// What a builder or a lab is committed to.
#[derive(Clone, Debug)]
pub(super) enum Task {
    /// A build order: the type, the site asked for, the spot if an extractor, when it was ordered, whether the
    /// engine has started it (a nanoframe by this builder since the order).
    Build { def: UnitDefId, near: Vec3, spot: Option<usize>, ordered: i32, started: bool },
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

/// A factory's last real answer (H-HANDS-DIET, decision 7).
pub(super) struct Replay {
    /// The question, the actor's entry and the instructions with their numbers struck.
    pub key: String,
    pub probabilities: BTreeMap<String, f64>,
    pub confidence: f64,
    /// The frame of the real ask.
    pub since: i32,
}

/// Jev's answers with the replayed menus' answers beside them.
fn replayed_answers(menus: &[menu::Menu], answers: &BTreeMap<String, jev::Answer>) -> BTreeMap<String, jev::Answer> {
    let mut all = answers.clone();
    for m in menus {
        if let Some((id, answer)) = &m.replay {
            all.insert(id.clone(), answer.clone());
        }
    }
    all
}


/// The menus' questions as built (a group's flat `do` among them), for the log: the readers and the offline replays
/// take the flat layout, and the raw `kind` and refinement answers sit beside the composed `do` in `answers`.
fn flat_questions(menus: &[menu::Menu]) -> BTreeMap<String, jev::Question> {
    menus.iter().filter(|m| m.replay.is_none()).flat_map(|m| m.questions.iter().cloned()).collect()
}

#[derive(Default)]
struct Stats {
    /// Calls not made because every real question was answered from a factory's last answer.
    replayed: u32,
    calls: u32,
    errors: u32,
    /// Realtime: requests whose answer had not come after `STALE_FRAMES`, dropped unplayed.
    dropped: u32,
    latencies_ms: Vec<f32>,
    tokens: u64,
    questions: u32,
    switches: u32,
    kept: u32,
}

pub struct Pianist {
    /// Jev, when `--pianist`; without it only the lists and the standing orders play.
    client: Option<jev::Client>,
    /// Standing orders (`standing.rs`, `docs/design/2026-09-25-standing-orders.md`): the packet's decompressed rules
    /// and the `standing` tool's; the executor's mode (`WITHIN_REASON_STANDING`: off, on, filter).
    pub(super) standing: Standing,
    pub(super) standing_mode: StandingMode,
    /// A fixed packet from a file (`WITHIN_REASON_PACKET`, the arena's `--packet`): what the hands play from when no
    /// player writes one, the arena instrument of the micro A/Bs (`docs/design/2026-09-25-one-decider.md`, §2).
    pub(super) packet: Option<String>,
    /// A decompression request on the worker (realtime): its id and the packet's frame.
    pending_decompression: Option<(u64, i32)>,
    /// Realtime (`WITHIN_REASON_REALTIME`): the call runs on this thread and its answer is played on the tick it
    /// arrives, so the game and the control lane never wait on Jev; in lockstep the call is made in place.
    worker: Option<Worker>,
    pending: Option<Pending>,
    next_request: u64,
    interval_frames: i32,
    last_ask_frame: i32,
    /// Builders' and labs' tasks, by unit.
    pub(super) tasks: HashMap<UnitId, Task>,
    /// A builder's next task, already ordered behind the one in progress (H-HANDS-QUEUE): it becomes the task when
    /// the frame in progress is finished, or when the engine starts it.
    pub(super) queued: HashMap<UnitId, Task>,
    /// The player's lists of steps per builder (by actor name), done by the bot without asking (H-HANDS-SCRIPT).
    pub(super) scripts: HashMap<String, VecDeque<String>>,
    /// What the hands have ordered each builder to build, in order, by unit name: a list arriving after the hands
    /// began the opening (the player's first orders land 15 to 25 s in) skips the leading steps already ordered
    /// (bank-1, 2v1-medium, 2v1-hard: the default had built two extractors and a solar by 0:25, the list's three
    /// solars made four before the plant). The hands' own orders, not the engine's created events: keyed on those,
    /// the skip never fired for the commander in 2v1-hard while its narration showed the three orders.
    pub(super) ordered: HashMap<UnitId, Vec<String>>,
    /// The list step each builder is on (its words, and the clock of the task it became): diverted from that task
    /// by the hands, the builder gets the step back at the front of its list (pace-1: an extractor step at spot_7
    /// was lost to a "go home", and the list went on to the turret).
    pub(super) list_steps: HashMap<UnitId, (String, i32)>,
    /// When each builder, lab or group (by name) was last asked.
    last_asked: HashMap<String, i32>,
    /// Units a lab has been told to build and not yet started, oldest first.
    pub(super) lab_queue: HashMap<UnitId, Vec<(UnitDefId, i32)>>,
    pub(super) groups: Vec<Group>,
    next_group: usize,
    /// Which factory made each soldier (the engine's creation events), while it lives: a newcomer joins its
    /// factory's group (H-HANDS-GROUPS: groups are the player's, nothing merges by proximity).
    pub(super) produced_by: HashMap<UnitId, UnitId>,
    /// Each factory's own group, by factory: the group its soldiers gather in unless `produce` names another; made
    /// on the first soldier and remade when it has died out.
    pub(super) rally: HashMap<UnitId, String>,
    /// The threat passes (`threats.rs`): the threats of the last ask, the worlds of its second call, and the threat
    /// picture's signature and frame at the last ask (ask on change).
    pub(super) threats: Vec<threats::Threat>,
    pub(super) threat_worlds: Vec<threats::World>,
    pub(super) threat_sig: Option<(String, i32)>,
    /// The hunts' ends since the last standing line (`groups.rs` `tick_hunt`), for the log.
    pub(super) hunt_events: Vec<String>,
    /// Spots where the engine refused an extractor, and until when they are left off the menus (H-HANDS-REFUSED).
    pub(super) refused_spots: HashMap<usize, i32>,
    /// Sites the engine refused for a building (the type, the point, until when): kept out of that type's site
    /// search for `REFUSED_FRAMES` (escalate-7: a nano turret's site was refused at 8:19 and re-ordered at the same
    /// point from the list's next step, twice, while two new lists waited).
    pub(super) refused_sites: Vec<(UnitDefId, Vec3, i32)>,
    /// The frame each builder's list was last set by the player (`queue`): a list newer than the builder's task
    /// displaces a build the engine has not started (H-HANDS-SCRIPT).
    pub(super) script_frame: HashMap<String, i32>,
    /// The frame the player's packet last changed. A task ordered before it is not protected by the switch margin
    /// (H-HANDS-SWITCH): the new packet is asked afresh (escalate-3, -5, -7: a group's walk outlived three packets
    /// until one named it abandoned).
    pub(super) packet_frame: i32,
    /// Actors an event or a new packet put on the next call, whatever their review period (H-HANDS-SCHEDULE).
    pub(super) due_now: HashSet<String>,
    /// Actors for the call after the next one: a packet's builders, asked half a second after its groups so one call
    /// does not carry every actor (schedule-1: 41 questions and 47k of the 64k tokens with 13 actors).
    pub(super) due_next: HashSet<String>,
    /// Our units hit since the last call, with the frame: the menus' under-fire set, across the ticks between calls.
    pub(super) hits: HashMap<UnitId, i32>,
    /// (builder, party name) pairs with the party inside the builder's alarm reach: a party's arrival puts the
    /// builder on the call once (H-HANDS-SCHEDULE; wake-4: the commander was asked only when hit).
    pub(super) alarmed: HashSet<(UnitId, String)>,
    /// The token diet's level and knobs (H-HANDS-DIET).
    pub(super) diet: diet::Diet,
    /// A factory's last real answer, by actor name, replayed while its inputs change only in numbers.
    pub(super) replays: HashMap<String, Replay>,
    /// The picture's place names at the last call: a change (a mark, a lane) puts every group on the call.
    pub(super) places_seen: BTreeSet<String>,
    /// The packet text `packet_frame` was set for, so one change sets it once.
    packet_seen: String,
    /// Units each lab has started since its allowance was set, by unit name (`produce` caps, "corck:1").
    pub(super) produced: HashMap<(UnitId, String), usize>,
    /// The allowance each lab (by name) was last seen with; a change restarts its counts.
    pub(super) allowed_seen: HashMap<String, Allowance>,
    /// What the last picture named, so an answer's place or party can be looked up.
    pub(super) places: Vec<Place>,
    pub(super) parties: Vec<Party>,
    /// The number the next new enemy party is named with (H-HANDS-PARTY-NAMES); a `Cell` because the picture is
    /// built through `&self`.
    pub(super) next_party: std::cell::Cell<usize>,
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
    /// What the hands did with this call's answers (`hands.rs`), for the log line.
    pub(super) played: Vec<serde_json::Value>,
    stats: Stats,
    /// The versioned model has been said in the game chat (once, after the first answer).
    announced: bool,
}

/// The pianist's log format version (`docs/harness/record-format.md`).
const LOG_VERSION: u32 = 1;
/// Realtime: an answer older than this judges a picture too old to play.
const STALE_FRAMES: i32 = 3 * FRAMES_PER_SECOND;

/// The thread that talks to Jev in real time: requests in, answers out, each with the request's number.
struct Worker {
    to: std::sync::mpsc::Sender<(u64, jev::Request)>,
    from: std::sync::mpsc::Receiver<(u64, Result<jev::Response, jev::Error>)>,
}

/// A request in flight: what it was built from, so its answers can be played when they come.
/// What `apply_orders` did with a set of orders.
#[derive(Default)]
struct Applied {
    answers: BTreeMap<String, jev::Answer>,
    taken: Vec<menu::Menu>,
    illegal: Vec<String>,
    continued: Vec<String>,
}

struct Pending {
    id: u64,
    frame: i32,
    picture: picture::Picture,
    menus: Vec<menu::Menu>,
    request: jev::Request,
}

fn spawn_worker(client: jev::Client) -> Worker {
    let (to, requests) = std::sync::mpsc::channel::<(u64, jev::Request)>();
    let (answers, from) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        while let Ok((mut id, mut request)) = requests.recv() {
            // Behind by a slow call, the worker answers the newest request only: answered in order, a burst of
            // one-second calls made every later answer stale (human-1, second game: three of 35 dropped).
            while let Ok((newer_id, newer)) = requests.try_recv() {
                (id, request) = (newer_id, newer);
            }
            if answers.send((id, client.ask(&request))).is_err() {
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
    /// be had. Without Jev the lists and the standing orders still play.
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
        Ok(Pianist {
            client,
            standing: Standing::default(),
            standing_mode: StandingMode::from_env(),
            packet,
            pending_decompression: None,
            worker,
            pending: None,
            next_request: 0,
            interval_frames: ((seconds * FRAMES_PER_SECOND as f32) as i32).max(super::BRAIN_FRAMES),
            last_ask_frame: i32::MIN / 2,
            tasks: HashMap::new(),
            queued: HashMap::new(),
            last_asked: HashMap::new(),
            lab_queue: HashMap::new(),
            groups: Vec::new(),
            next_group: 0,
            produced_by: HashMap::new(),
            rally: HashMap::new(),
            threats: Vec::new(),
            threat_worlds: Vec::new(),
            threat_sig: None,
            hunt_events: Vec::new(),
            refused_spots: HashMap::new(),
            refused_sites: Vec::new(),
            script_frame: HashMap::new(),
            packet_frame: 0,
            due_now: HashSet::new(),
            due_next: HashSet::new(),
            diet: diet::Diet::from_env(),
            replays: HashMap::new(),
            hits: HashMap::new(),
            alarmed: HashSet::new(),
            places_seen: BTreeSet::new(),
            packet_seen: String::new(),
            produced: HashMap::new(),
            allowed_seen: HashMap::new(),
            places: Vec::new(),
            parties: Vec::new(),
            next_party: std::cell::Cell::new(1),
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
                "interval_frames": self.interval_frames, "rules": rules, "hands_effort": self.diet.level_name(),
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
        if n < 26 { letter.to_string() } else { format!("{letter}{}", n / 26) }
    }
}

impl Brain {
    /// The pianist's whole turn of the brain: bookkeeping every think, a call to Jev when one is due.
    /// A timed list step `assist N` ends after N seconds: the task is dropped and the builder is put on the next call,
    /// where its list's next step plays (H-HANDS-SCRIPT). The engine keeps the guard until the next order replaces it.
    fn end_timed_assists(&mut self, frame: i32, commands: &mut Vec<Command>) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let mut ended: Vec<UnitId> = Vec::new();
        for (unit, (step, _)) in &pianist.list_steps {
            let Some(seconds) = menu::timed_assist(step) else { continue };
            if let Some(Task::Assist { since, .. }) = pianist.tasks.get(unit)
                && frame - since >= seconds * FRAMES_PER_SECOND
            {
                ended.push(*unit);
            }
        }
        if ended.is_empty() {
            return;
        }
        let names: Vec<(UnitId, String)> = ended.iter().map(|u| (*u, self.actor_name(*u))).collect();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (unit, name) in names {
            pianist.tasks.remove(&unit);
            pianist.list_steps.remove(&unit);
            pianist.due_now.insert(name);
            // The engine keeps a guard until another order: the stop makes the builder idle, and its list's next
            // step plays at the call the `due_now` brings forward.
            commands.push(Command::Stop { unit });
        }
    }

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
        // conditions read; the heuristic brain keeps them inside its army rules.
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
            // Realtime: one request in flight at a time; this second's ask waits for the answer. An event that put an
            // actor on the call brings the call forward to half a second after the last (H-HANDS-SCHEDULE).
            let since = tick.frame - pianist.last_ask_frame;
            pianist.pending.is_none() && (since >= pianist.interval_frames || (!pianist.due_now.is_empty() && since >= schedule::EVENT_CALL_GAP))
        };
        if !due {
            return;
        }
        self.pianist.as_mut().expect("pianist mode").last_ask_frame = tick.frame;
        if let Some(shared) = &self.strategist {
            let lists = std::mem::take(&mut *shared.queues.lock().unwrap());
            for (name, list) in lists {
                // A list beginning with `stop` (or the bare word) drops what the builder is doing now: the build in
                // progress is abandoned and its frame decays. `null` cancels the list and lets that build finish
                // (escalate-1, comet-5: the player asked to cancel a queued plant and the hands finished it).
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
                    eprintln!("[ai {}] f={} first list for {name}: the hands had ordered [{}], its first {k} steps skipped", self.world.hello.ai_id, tick.frame, ordered.join(", "));
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
                match steps {
                    Some(steps) => {
                        pianist.script_frame.insert(name.clone(), tick.frame);
                        pianist.scripts.insert(name, steps.into());
                    }
                    None => {
                        pianist.script_frame.insert(name.clone(), tick.frame);
                        pianist.scripts.remove(&name);
                    }
                }
            }
        }
        self.apply_standing_changes(tick.frame);
        let picture = self.picture(tick, kit);
        let mut packet_changed = false;
        {
            // A changed packet is asked afresh: no standing task is protected by the switch margin against it.
            let pianist = self.pianist.as_mut().expect("pianist mode");
            let instructions = picture.state["instructions"].as_str().unwrap_or_default();
            if !instructions.is_empty() && instructions != pianist.logged_instructions && instructions != pianist.packet_seen {
                pianist.packet_seen = instructions.to_string();
                pianist.packet_frame = tick.frame;
                packet_changed = true;
            }
        }
        if packet_changed {
            self.schedule_all_for_packet(tick);
            self.decompress_packet(tick, &picture);
        }
        let mut menus = self.menus(tick, kit, &picture);
        if menus.is_empty() {
            return;
        }
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.places = picture.places.clone();
            pianist.parties = picture.parties.clone();
        }
        // The standing orders take the actors a rule decides for; the rest go to Jev, or keep their course without it.
        self.standing_pass(tick, kit, &picture, &mut menus, commands);
        let status_due = tick.due() % (60 * FRAMES_PER_SECOND) < self.pianist.as_ref().expect("pianist mode").interval_frames;
        if self.pianist.as_ref().expect("pianist mode").client.is_none() {
            // Without Jev the players' lists still play themselves (H-HANDS-SCRIPT); everything else keeps its course.
            let listed: Vec<menu::Menu> = menus.into_iter().filter(|m| m.scripted.is_some()).collect();
            if !listed.is_empty() {
                self.play(tick, kit, &picture, listed, &BTreeMap::new(), commands);
            }
            self.publish_hands(&picture, &BTreeMap::new());
            if status_due {
                self.pianist_status_line(tick.frame);
            }
            return;
        }
        let mut questions: BTreeMap<String, jev::Question> = BTreeMap::new();
        for menu in menus.iter().filter(|m| m.replay.is_none()) {
            for (id, question) in &menu.questions {
                questions.insert(id.clone(), question.clone());
            }
        }
        // Every real question answered from a replay: nothing to ask; the replays play without a call.
        if menus.iter().any(|m| m.replay.is_some()) && menus.iter().all(|m| m.replay.is_some() || m.questions.is_empty() || (matches!(m.actor, menu::Actor::Global) && m.name != "worlds")) {
            let answers = replayed_answers(&menus, &BTreeMap::new());
            self.pianist.as_mut().expect("pianist mode").stats.replayed += 1;
            self.play(tick, kit, &picture, menus, &answers, commands);
            self.publish_hands(&picture, &answers);
            return;
        }
        let state = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            self.trim_state(pianist, &picture, &menus, tick)
        };
        let request = jev::Request { state, questions };
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            if pianist.stats.calls == 0 && pianist.logged_instructions.is_empty() {
                let rules = picture.state["rules"].as_str().unwrap_or_default().to_string();
                pianist.log_header(self.world.hello.ai_id, &rules);
            }
            pianist.stats.calls += 1;
            pianist.stats.questions += request.questions.len() as u32;
            if pianist.worker.is_some() {
                let id = pianist.next_request;
                pianist.next_request += 1;
                if pianist.worker.as_ref().expect("checked").to.send((id, request.clone())).is_ok() {
                    pianist.pending = Some(Pending { id, frame: tick.frame, picture, menus, request });
                }
                if status_due {
                    self.pianist_status_line(tick.frame);
                }
                return;
            }
        }
        let (response, ai) = {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.played.clear();
            (pianist.client.as_ref().expect("checked above").ask(&request), self.world.hello.ai_id)
        };
        match response {
            Ok(response) => {
                {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
                    pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                }
                self.announce_hands(&response, commands);
                let answers = replayed_answers(&menus, &response.answers);
                self.threat_resolve(tick, &picture, &menus, &answers, commands);
                let second = self.threat_follow_up(tick, &mut menus, &answers);
                let flat = flat_questions(&menus);
                self.play(tick, kit, &picture, menus, &answers, commands);
                self.publish_hands(&picture, &answers);
                self.log_call(tick, &request, &response, flat, &answers);
                if let Some(second) = second {
                    self.second_call(tick, kit, picture, second, request.state.clone(), commands);
                }
            }
            Err(e) => {
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.stats.errors += 1;
                if let Some(log) = &mut pianist.log {
                    let _ = writeln!(log, "{}", json!({ "t": "error", "f": tick.frame, "error": e.to_string() }));
                }
                eprintln!("[ai {ai}] f={} pianist: {e}; every actor keeps its task", tick.frame);
            }
        }
        if status_due {
            self.pianist_status_line(tick.frame);
        }
    }

    /// An order that is what the actor is doing already: a group advancing to that place or holding, a builder
    /// helping that factory or walking there. Builds are covered by H-HANDS-STARTED in `play_one`.
    fn same_as_current(&self, menu: &menu::Menu, order: &standing::Order) -> bool {
        let pianist = self.pianist.as_ref().expect("pianist mode");
        let where_ = order.params.get("where").map(String::as_str);
        match &menu.actor {
            menu::Actor::Group(name) => match pianist.groups.iter().find(|g| g.name == *name).map(|g| &g.task) {
                Some(GroupTask::Hold { .. }) => order.choice == "hold",
                Some(GroupTask::Move { place, fight, .. }) => ((order.choice == "fight_to" && *fight) || (order.choice == "move_to" && !*fight)) && where_ == Some(place.as_str()),
                Some(GroupTask::Engage { target, .. }) => (order.choice == "engage" && target.is_none()) || (order.choice == "attack_unit" && target.is_some()),
                None => false,
            },
            menu::Actor::Builder(id) => match pianist.tasks.get(id) {
                Some(Task::Assist { .. }) => order.choice == "assist_lab",
                Some(Task::Walk { place, .. }) => (order.choice == "walk_to" && where_ == Some(place.as_str())) || (order.choice == "retreat_home" && place == "home"),
                _ => false,
            },
            _ => false,
        }
    }

    /// The standing orders applied to the menus of the second: each names an actor asked this second, an option on
    /// its menu and a place the picture has; an order equal to the actor's current task is `continue`; the rest
    /// become answers at probability one and their menus are taken out of `menus`.
    fn apply_orders(&mut self, picture: &picture::Picture, menus: &mut Vec<menu::Menu>, orders: Vec<(String, standing::Order)>) -> Applied {
        let mut applied = Applied::default();
        for (actor, order) in orders {
            let Some(i) = menus.iter().position(|m| m.name == actor && !matches!(m.actor, menu::Actor::Global)) else {
                applied.illegal.push(format!("{actor}: not asked this second"));
                continue;
            };
            if menus[i].scripted.is_some() {
                applied.illegal.push(format!("{actor}: on a list from `queue`, which plays itself; cancel the list to order it"));
                continue;
            }
            if !menus[i].options.contains_key(&order.choice) {
                applied.illegal.push(format!("{actor}: {} is not on its menu", order.choice));
                continue;
            }
            if let Some(place) = order.params.get("where").or_else(|| order.params.get("where_scout")).or_else(|| order.params.get("where_extractor"))
                && !picture.places.iter().any(|p| p.name == *place)
            {
                applied.illegal.push(format!("{actor}: {place} is not a place in the picture"));
                continue;
            }
            let mut menu = menus.remove(i);
            if self.same_as_current(&menu, &order) {
                applied.continued.push(actor.clone());
                continue;
            }
            menu.standing_played = true;
            let qid = if matches!(menu.actor, menu::Actor::Lab(_)) { format!("{actor}.next") } else { format!("{actor}.do") };
            let sure = |choice: &str| jev::Answer::Choice { choice: choice.to_string(), probabilities: BTreeMap::from([(choice.to_string(), 1.0)]), confidence: 1.0 };
            applied.answers.insert(qid, sure(&order.choice));
            for (k, v) in &order.params {
                applied.answers.insert(format!("{actor}.{k}"), sure(v));
            }
            if order.choice == "extractor"
                && let Some(place) = order.params.get("where")
                && !order.params.contains_key("where_extractor")
            {
                applied.answers.insert(format!("{actor}.where_extractor"), sure(place));
            }
            applied.taken.push(menu);
        }
        applied
    }

    /// The standing orders' pass (`standing.rs`): each actor a rule decides for is played from the rule at
    /// probability one (mode `on`), or asked with the rule's order beside its menu as a `standing` question (mode
    /// `filter`); an order equal to the actor's task saves the ask. Mode `off` only logs what would have fired.
    fn standing_pass(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, menus: &mut Vec<menu::Menu>, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let (mode, any) = match self.pianist.as_ref() {
            Some(p) => (p.standing_mode, p.standing.counts() != (0, 0)),
            None => return,
        };
        if !any {
            return;
        }
        let mut orders: Vec<(String, standing::Order, String)> = Vec::new();
        // A group whose unit types changed since its tool orders were set is said once (standing-1, 4:26).
        {
            let own = &tick.snapshot.own_units;
            let changed: Vec<(String, BTreeSet<String>)> = self.pianist.as_ref().expect("pianist mode").groups.iter().map(|g| (format!("group_{}", g.name), g.units(own).iter().map(|u| self.name(u.def).to_string()).collect::<BTreeSet<_>>())).collect();
            let pianist = self.pianist.as_mut().expect("pianist mode");
            for (name, types) in changed {
                if let Some(text) = pianist.standing.composition_changed(&name, types) {
                    pianist.done.push(format!("{} standing: {text}", picture::clock(frame)));
                }
            }
        }
        for menu in menus.iter() {
            if matches!(menu.actor, menu::Actor::Global) || menu.scripted.is_some() || menu.replay.is_some() {
                continue;
            }
            if let Some((order, rule)) = self.standing_order(tick, kit, picture, menu) {
                orders.push((menu.name.clone(), order, rule));
            }
        }
        if orders.is_empty() && !matches!(mode, StandingMode::Worlds | StandingMode::On) {
            return;
        }
        let mut orders_json = serde_json::Map::new();
        for (actor, order, rule) in &orders {
            orders_json.insert(actor.clone(), json!({ "do": order.choice, "params": order.params, "rule": rule }));
        }
        let mut line = json!({ "t": "standing", "f": frame, "mode": mode.name(), "orders": orders_json });
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            if !pianist.hunt_events.is_empty() {
                line["hunts"] = json!(std::mem::take(&mut pianist.hunt_events));
            }
        }
        match mode {
            StandingMode::Off => {}
            StandingMode::Filter => {
                // The rule's order goes with the menu: the actor is asked, and the verdict decides (`hands.rs`).
                for (actor, order, rule) in orders {
                    let Some(menu) = menus.iter_mut().find(|m| m.name == actor) else { continue };
                    let words = format!("{}{}", order.choice, if order.params.is_empty() { String::new() } else { format!(" ({})", order.params.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", ")) });
                    let question = jev::Question::choice(
                        format!("The standing order for {actor} (its rule `{rule}`, from the player's packet or `standing` tool) says: {words}. Given `actors.{actor}` and the picture, what should happen?"),
                        [
                            ("rule", format!("exactly that: {words}")),
                            ("near", "the same kind of action but a different target, size or place: the answer to the menu question is played instead".to_string()),
                            ("other", "something else entirely: the answer to the menu question is played instead".to_string()),
                            ("panic", format!("this is going badly for {actor}: it falls back and the player is woken")),
                        ],
                    );
                    menu.questions.push((format!("{actor}.standing"), question));
                    menu.standing = Some((order, rule));
                }
            }
            StandingMode::On | StandingMode::Worlds => {
                let plain: Vec<(String, standing::Order)> = orders.iter().map(|(a, o, _)| (a.clone(), o.clone())).collect();
                let Applied { answers, taken, illegal, continued } = self.apply_orders(picture, menus, plain);
                let pianist = self.pianist.as_mut().expect("pianist mode");
                for (actor, _, rule) in &orders {
                    if !illegal.iter().any(|i| i.starts_with(&format!("{actor}:"))) {
                        *pianist.standing.fired.entry(format!("{actor} {rule}")).or_insert(0) += 1;
                        if rule == "retreat_when_enemy_near" {
                            pianist.standing.retreated.insert(actor.clone(), frame);
                        }
                    }
                }
                pianist.standing.saved += (taken.len() + continued.len()) as u32;
                line["illegal"] = json!(illegal);
                line["continued"] = json!(continued);
                if !taken.is_empty() {
                    // Plays of a replayed lab answer since the last call would otherwise sit in this line's list.
                    self.pianist.as_mut().expect("pianist mode").played.clear();
                    self.play(tick, kit, picture, taken, &answers, commands);
                }
                // The threats (`threats.rs`): the plan's base put in force, the gate's nouls asked on change.
                self.threat_pass(tick, picture, menus, commands, &mut line, mode);
            }
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if matches!(mode, StandingMode::On | StandingMode::Worlds) {
            line["played"] = json!(std::mem::take(&mut pianist.played));
        }
        if let Some(log) = &mut pianist.log {
            let _ = writeln!(log, "{line}");
        }
    }

    /// The threat passes (`threats.rs`, `docs/design/2026-09-26-threat-response.md` §4-6): the threats and the
    /// states against each this second; the plan's base (the states in force, the rules' defaults) put in force at
    /// once; when something is open and the threat picture changed since the last ask (or `RE_ASK` passed), the
    /// gate's nouls go with this call and the worlds question with the next (`threat_follow_up`). Under `on`, or
    /// without Jev, the base is the plan.
    fn threat_pass(&mut self, tick: &Tick, picture: &picture::Picture, menus: &mut Vec<menu::Menu>, commands: &mut Vec<Command>, line: &mut serde_json::Value, mode: StandingMode) {
        let threats = self.threats(tick, picture);
        if threats.is_empty() {
            return;
        }
        let base: threats::World = threats.iter().map(threats::Threat::base).collect();
        let started = self.apply_plan(tick, picture, &threats, &base, "rule", commands);
        line["threats"] = threats::log_threats(&threats);
        if !started.is_empty() {
            line["plan"] = json!(started);
        }
        let questions = threats::gate_questions(&threats);
        let sig = threats::signature(&threats);
        let jev = self.pianist.as_ref().is_some_and(|p| p.client.is_some());
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let changed = pianist.threat_sig.as_ref().is_none_or(|(s, f)| *s != sig || tick.frame - *f >= threats::RE_ASK);
        if questions.is_empty() || mode != StandingMode::Worlds || !jev {
            return;
        }
        if !changed {
            line["quiet"] = json!("the threat picture is as at the last ask: the plan stands");
            pianist.standing.saved += 1;
            return;
        }
        pianist.threat_sig = Some((sig, tick.frame));
        pianist.threats = threats;
        pianist.threat_worlds.clear();
        let mut gate = menu::Menu::carrier("gate");
        gate.questions = questions;
        line["gate"] = json!(gate.questions.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>());
        menus.push(gate);
    }

    /// Puts a world's states in force: a state already current stands; Whole starts the engagement, Hunt the hunt,
    /// Back the walk; Leave ends a current hunt or engagement of that party and declines it for a while. Returns
    /// what changed, for the log and the `done` lines.
    fn apply_plan(&mut self, tick: &Tick, picture: &picture::Picture, threats: &[threats::Threat], world: &threats::World, source: &str, commands: &mut Vec<Command>) -> Vec<String> {
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let frame = tick.frame;
        let mut done: Vec<(String, String)> = Vec::new();
        let Some(mut pianist) = self.pianist.take() else { return Vec::new() };
        for (t, si) in threats.iter().zip(world) {
            let s = &t.states[*si];
            if s.current {
                continue;
            }
            match &s.response {
                threats::Response::Leave => {
                    for group in pianist.groups.iter_mut() {
                        if group.hunt.as_ref().is_some_and(|h| t.party.ids.contains(&h.quarry)) {
                            let hunters: Vec<UnitId> = group.hunt.as_ref().map(|h| h.hunters.clone()).unwrap_or_default();
                            group.hunt = None;
                            group.declined.push((t.party.name.clone(), frame));
                            let units: Vec<&bot_protocol::OwnUnit> = own.iter().filter(|u| hunters.contains(&u.id)).collect();
                            commands.extend(group.rejoin_orders(&units));
                            done.push((format!("group_{}", group.name), format!("its hunt of {} called off", t.party.name)));
                        }
                        if matches!(&group.task, GroupTask::Engage { party, .. } if party.iter().any(|id| t.party.ids.contains(id))) {
                            let units = group.units(own);
                            commands.extend(group.hold_orders(&units));
                            group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                            group.declined.push((t.party.name.clone(), frame));
                            done.push((format!("group_{}", group.name), format!("leaves {} and holds", t.party.name)));
                        }
                    }
                }
                threats::Response::Whole(g) => {
                    let air_target = pianist.groups.iter().find(|x| format!("group_{}", x.name) == *g).filter(|x| x.domain == crate::world::Domain::Air).and_then(|_| self.party_target(&t.party, enemies)).map(|(id, _)| id);
                    if let Some(group) = pianist.groups.iter_mut().find(|x| format!("group_{}", x.name) == *g) {
                        let text = Brain::engage_group(group, &t.party, own, air_target, frame, commands);
                        done.push((g.clone(), text));
                    }
                }
                threats::Response::Hunt(g, hunters) => {
                    if let Some(group) = pianist.groups.iter_mut().find(|x| format!("group_{}", x.name) == *g) {
                        let text = Brain::start_hunt(group, hunters.clone(), &t.party, own, enemies, frame, commands);
                        done.push((g.clone(), text));
                    }
                }
                threats::Response::Back(g, place) => {
                    if let Some(group) = pianist.groups.iter_mut().find(|x| format!("group_{}", x.name) == *g)
                        && let Some(p) = picture.places.iter().find(|x| x.name == *place)
                    {
                        let units = group.units(own);
                        let to = p.at;
                        commands.extend(group.release_orders(&units));
                        commands.extend(units.iter().map(|u| Command::Move { unit: u.id, to, queue: false }));
                        group.set_task(GroupTask::Move { to, place: place.clone(), fight: false, since: frame }, frame);
                        group.last_order = frame;
                        done.push((g.clone(), format!("falls back to {place} from {}", t.party.name)));
                    }
                }
            }
        }
        for (actor, text) in &done {
            pianist.done.push(format!("{} {actor}: {text} ({source})", picture::clock(frame)));
            pianist.played.push(json!({ "actor": actor, "kind": "group", "played": text, "source": source }));
        }
        self.pianist = Some(pianist);
        done.into_iter().map(|(a, t)| format!("{a}: {t}")).collect()
    }

    /// After the gate's answers: the worlds over the flagged states as the second call's `worlds` menu, or nothing
    /// to ask. Logs what the gate said and the worlds' lines.
    fn threat_follow_up(&mut self, tick: &Tick, menus: &mut [menu::Menu], answers: &BTreeMap<String, jev::Answer>) -> Option<Vec<menu::Menu>> {
        if !menus.iter().any(|m| m.name == "gate") {
            return None;
        }
        let threats = self.pianist.as_ref().expect("pianist mode").threats.clone();
        let mut flags: BTreeMap<String, f64> = BTreeMap::new();
        let worlds = threats::compose(&threats, answers, &mut flags);
        let lines: Vec<String> = worlds.as_ref().map(|ws| ws.iter().map(|w| threats::consequence(w, &threats)).collect()).unwrap_or_default();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if let Some(log) = &mut pianist.log {
            let _ = writeln!(log, "{}", json!({ "t": "worlds_gate", "f": tick.frame, "flags": flags, "worlds": worlds, "lines": lines }));
        }
        let ws = worlds?;
        let mut carrier = menu::Menu::carrier("worlds");
        carrier.questions = vec![("worlds.pick".to_string(), threats::question(&lines))];
        carrier.worlds = ws.clone();
        pianist.threat_worlds = ws;
        Some(vec![carrier])
    }

    /// The second call's pick put in force, and logged as a `plan` line.
    fn threat_resolve(&mut self, tick: &Tick, picture: &picture::Picture, menus: &[menu::Menu], answers: &BTreeMap<String, jev::Answer>, commands: &mut Vec<Command>) {
        if !menus.iter().any(|m| m.name == "worlds") {
            return;
        }
        let (threats, worlds) = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            (pianist.threats.clone(), pianist.threat_worlds.clone())
        };
        let Some((wi, confidence)) = threats::pick(answers, &worlds) else { return };
        let changed = self.apply_plan(tick, picture, &threats, &worlds[wi], "plan", commands);
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if let Some(log) = &mut pianist.log {
            let _ = writeln!(log, "{}", json!({ "t": "plan", "f": tick.frame, "pick": wi + 1, "confidence": confidence, "changed": changed }));
        }
        self.journal.note_from("plan", tick.frame, "group", json!({ "worlds": worlds.len() }), json!({ "pick": wi + 1, "confidence": confidence, "changed": changed }));
    }

    /// The second call of the threat passes: the worlds question alone over the same state. In lockstep it is made here;
    /// in realtime it goes to the worker and lands with `collect_answer`.
    fn second_call(&mut self, tick: &Tick, kit: &Kit, picture: picture::Picture, menus: Vec<menu::Menu>, state: serde_json::Value, commands: &mut Vec<Command>) {
        let questions: BTreeMap<String, jev::Question> = menus.iter().filter(|m| m.name == "worlds").flat_map(|m| m.questions.iter().cloned()).collect();
        let request = jev::Request { state, questions };
        let ai = self.world.hello.ai_id;
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.stats.calls += 1;
            pianist.stats.questions += request.questions.len() as u32;
            if pianist.worker.is_some() {
                let id = pianist.next_request;
                pianist.next_request += 1;
                if pianist.worker.as_ref().expect("checked").to.send((id, request.clone())).is_ok() {
                    pianist.pending = Some(Pending { id, frame: tick.frame, picture, menus, request });
                }
                return;
            }
        }
        let response = {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.played.clear();
            pianist.client.as_ref().expect("a second call needs Jev").ask(&request)
        };
        match response {
            Ok(response) => {
                {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
                    pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                }
                let menus = menus;
                let answers = replayed_answers(&menus, &response.answers);
                self.threat_resolve(tick, &picture, &menus, &answers, commands);
                let flat = flat_questions(&menus);
                self.play(tick, kit, &picture, menus, &answers, commands);
                self.publish_hands(&picture, &answers);
                self.log_call(tick, &request, &response, flat, &answers);
            }
            Err(e) => {
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.stats.errors += 1;
                if let Some(log) = &mut pianist.log {
                    let _ = writeln!(log, "{}", json!({ "t": "error", "f": tick.frame, "error": e.to_string() }));
                }
                eprintln!("[ai {ai}] f={} pianist: the second call failed: {e}; the groups keep their course", tick.frame);
            }
        }
    }

    /// The packet, changed, goes to Jev once as extraction questions over the standing vocabulary; the answers
    /// replace the packet's standing orders (`standing::orders_from`). In lockstep the call is made here; in
    /// realtime it goes through the worker and lands in `collect_answer`.
    fn decompress_packet(&mut self, tick: &Tick, picture: &picture::Picture) {
        let ai = self.world.hello.ai_id;
        let text = picture.state["instructions"].as_str().unwrap_or_default().to_string();
        let places: Vec<String> = picture.places.iter().map(|p| p.name.clone()).collect();
        let questions = standing::extraction_questions(&text, &places);
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if questions.is_empty() || pianist.client.is_none() {
            pianist.standing.set_packet(BTreeMap::new(), tick.frame);
            return;
        }
        let request = jev::Request { state: standing::extraction_state(&text), questions };
        if let Some(worker) = &pianist.worker {
            let id = pianist.next_request;
            pianist.next_request += 1;
            if worker.to.send((id, request)).is_ok() {
                pianist.pending_decompression = Some((id, tick.frame));
            }
            return;
        }
        let result = pianist.client.as_ref().expect("checked").ask(&request);
        self.take_decompression(tick.frame, tick.frame, result, ai);
    }

    fn take_decompression(&mut self, frame: i32, packet_frame: i32, result: Result<jev::Response, jev::Error>, ai: i32) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        match result {
            Ok(response) => {
                let orders = standing::orders_from(&response.answers);
                let n: usize = orders.values().map(|r| r.len()).sum();
                let actors = orders.len();
                pianist.standing.set_packet(orders.clone(), packet_frame);
                pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                pianist.done.push(format!("{} the packet was read into {n} standing orders for {actors} actors ({})", picture::clock(frame), pianist.standing_mode.name()));
                if let Some(log) = &mut pianist.log {
                    let _ = writeln!(log, "{}", json!({ "t": "decompress", "f": frame, "packet_frame": packet_frame, "ms": (response.latency.as_secs_f32() * 1000.0) as u32, "usage": response.usage, "orders": orders, "answers": response.answers }));
                }
            }
            Err(e) => {
                pianist.stats.errors += 1;
                if let Some(log) = &mut pianist.log {
                    let _ = writeln!(log, "{}", json!({ "t": "error", "f": frame, "error": format!("decompression: {e}") }));
                }
                eprintln!("[ai {ai}] f={frame} pianist: the packet's decompression failed: {e}; the standing orders stand as they were");
            }
        }
    }

    /// The player's `standing` calls since the last ask: orders set or cleared, checked against the picture.
    fn apply_standing_changes(&mut self, frame: i32) {
        let Some(shared) = &self.strategist else { return };
        let changes = std::mem::take(&mut *shared.standing.lock().unwrap());
        if changes.is_empty() {
            return;
        }
        // The marks too: a mark and a rule naming it come in one turn, before the picture has the mark (worlds-1,
        // 14:15: "marks refused in standing"); and every spot our bots can walk to, listed in the picture or not
        // (worlds-2: fifteen sets refused for a spot the picture's short list lacked; the picture lists the
        // tool's places from then on).
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
                            Ok(n) => format!("{actor}: {n} rules set"),
                            Err(e) => format!("refused: {e}"),
                        });
                    }
                    said.join("; ")
                }
                crate::strategist::shared::StandingChange::Clear(actors) => format!("{} tool orders cleared", pianist.standing.clear_tool(actors.as_deref())),
            };
            pianist.done.push(format!("{} standing: {said}", picture::clock(frame)));
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
        let mut decompressed: Option<(i32, Result<jev::Response, jev::Error>)> = None;
        let arrived = {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            let Some(worker) = &pianist.worker else { return };
            let mut got = None;
            while let Ok((id, result)) = worker.from.try_recv() {
                if let Some((did, dframe)) = pianist.pending_decompression
                    && did == id
                {
                    pianist.pending_decompression = None;
                    decompressed = Some((dframe, result));
                    continue;
                }
                // An answer to a request already dropped is not played.
                if pianist.pending.as_ref().is_some_and(|p| p.id == id) {
                    got = Some(result);
                }
            }
            match got {
                Some(result) => Some((pianist.pending.take().expect("a matched request is pending"), result)),
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
        if let Some((dframe, result)) = decompressed {
            self.take_decompression(tick.frame, dframe, result, ai);
        }
        let Some((pending, result)) = arrived else { return };
        match result {
            Ok(response) => {
                {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    pianist.stats.latencies_ms.push(response.latency.as_secs_f32() * 1000.0);
                    pianist.stats.tokens += response.usage["input_tokens"].as_u64().unwrap_or(0);
                    pianist.places = pending.picture.places.clone();
                    pianist.parties = pending.picture.parties.clone();
                    pianist.played.clear();
                }
                self.announce_hands(&response, commands);
                let mut menus = pending.menus;
                let answers = replayed_answers(&menus, &response.answers);
                self.threat_resolve(tick, &pending.picture, &menus, &answers, commands);
                let second = self.threat_follow_up(tick, &mut menus, &answers);
                let flat = flat_questions(&menus);
                self.play(tick, kit, &pending.picture, menus, &answers, commands);
                self.publish_hands(&pending.picture, &answers);
                self.log_call(tick, &pending.request, &response, flat, &answers);
                if let Some(second) = second {
                    self.second_call(tick, kit, pending.picture, second, pending.request.state.clone(), commands);
                }
            }
            Err(e) => {
                let pianist = self.pianist.as_mut().expect("pianist mode");
                pianist.stats.errors += 1;
                if let Some(log) = &mut pianist.log {
                    let _ = writeln!(log, "{}", json!({ "t": "error", "f": tick.frame, "error": e.to_string() }));
                }
                eprintln!("[ai {ai}] f={} pianist: {e}; every actor keeps its task", tick.frame);
            }
        }
    }

    /// One line of the log per call: the request (the instructions and the rules only when they changed; the rules
    /// are in the header and re-read from disk each call), the answers, what the hands played, and the groups, places
    /// and parties by name so a reader can draw them.
    fn log_call(&mut self, tick: &Tick, request: &jev::Request, response: &jev::Response, questions: BTreeMap<String, jev::Question>, answers: &BTreeMap<String, jev::Answer>) {
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
        // The kind and refinement answers behind a composed `do` (H-HANDS-TWO-LEVEL): the readers take the flat layout,
        // so they sit apart under `raw`.
        let mut line = json!({
            "t": "call", "f": tick.frame, "ms": (response.latency.as_secs_f32() * 1000.0) as u32, "model": response.model, "usage": response.usage,
            "retries": response.retries, "state": state, "questions": questions, "answers": answers,
            "played": std::mem::take(&mut pianist.played), "groups": groups, "places": places, "parties": parties,
        });
        if rules_changed {
            line["rules"] = json!(rules);
        }
        if changed {
            line["instructions"] = json!(instructions);
        }
        if let Some(log) = &mut pianist.log {
            let _ = writeln!(log, "{line}");
        }
    }

    /// Tasks and queues against what the engine says: builds started or refused, labs' units begun, units gone.
    fn pianist_housekeeping(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let frame = tick.frame;
        let mut notes: Vec<String> = Vec::new();
        // What did not happen, for the player's report as well as the picture.
        let mut done: Vec<String> = Vec::new();
        let mut refused: Vec<(UnitId, UnitDefId, Vec3)> = Vec::new();
        let Some(mut pianist) = self.pianist.take() else { return };
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
                    // free to be asked.
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
    /// lines since its last turn, the groups that began an engagement this call, the global probabilities.
    fn publish_hands(&mut self, picture: &picture::Picture, answers: &BTreeMap<String, jev::Answer>) {
        let Some(shared) = &self.strategist else { return };
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let mut state = picture.state.clone();
        if let Some(fields) = state.as_object_mut() {
            fields.remove("instructions");
            fields.remove("rules");
        }
        let mut hands = shared.hands.lock().unwrap();
        hands.picture = state;
        hands.done.append(&mut pianist.done);
        hands.standing_text = pianist.standing.in_force();
        hands.standing_mode = pianist.standing_mode.name().to_string();
        let (packet, tool) = pianist.standing.counts();
        hands.standing_counts = (packet, tool);
        for (k, v) in std::mem::take(&mut pianist.standing.fired) {
            *hands.standing_fired.entry(k).or_insert(0) += v;
        }
        hands.standing_saved += std::mem::take(&mut pianist.standing.saved);
        for (k, v) in std::mem::take(&mut pianist.standing.verdicts) {
            *hands.standing_verdicts.entry(k).or_insert(0) += v;
        }
        hands.engaged = pianist.played.iter().filter(|p| p["did"].as_str().is_some_and(|d| d.starts_with("attack "))).filter_map(|p| p["actor"].as_str().map(str::to_string)).collect();
        hands.globals = answers.iter().filter_map(|(id, a)| id.strip_prefix("global.").map(|q| (q.to_string(), a.probability_of("yes")))).collect();
    }

    fn pianist_status_line(&mut self, frame: i32) {
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let stats = std::mem::take(&mut pianist.stats);
        if stats.calls == 0 {
            return;
        }
        let mut latencies = stats.latencies_ms;
        latencies.sort_by(f32::total_cmp);
        let median = latencies.get(latencies.len() / 2).copied().unwrap_or(0.0);
        let max = latencies.last().copied().unwrap_or(0.0);
        eprintln!(
            "[ai {}] f={frame} pianist this minute: {} calls ({} failed, {} dropped, {} replayed), median {median:.0} ms, longest {max:.0}, {} tokens in, {} questions; busy actors changed course {} times, kept {}; groups {}, tasks {}",
            self.world.hello.ai_id, stats.calls, stats.errors, stats.dropped, stats.replayed, stats.tokens, stats.questions, stats.switches, stats.kept, pianist.groups.len(), pianist.tasks.len()
        );
    }
}
