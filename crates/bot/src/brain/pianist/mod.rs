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
mod policy;

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

use bot_protocol::{Command, Event, Tick, UnitDefId, UnitId, Vec3};
use serde_json::{Value, json};

use super::economy::FIRST_ORDER_FRAME;
use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};
pub(super) use groups::{Group, GroupTask};
pub(super) use picture::{Party, Place};
pub(crate) use picture::clock;
pub use policy::{Policy, PolicyStats};

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
/// H-HANDS-PLAYER-WAKE: the player is woken when Jev says the game needs it for this many calls running, at most this
/// often.
const NEEDS_PLAYER_PROBABILITY: f64 = 0.8;
const NEEDS_PLAYER_CALLS: u32 = 3;
const NEEDS_PLAYER_COOLDOWN: i32 = 60 * FRAMES_PER_SECOND;

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
    fn since(&self) -> i32 {
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
    switches: u32,
    kept: u32,
}

pub struct Pianist {
    /// Jev, when `--pianist`; without it the actors the policy leaves out keep their course.
    client: Option<jev::Client>,
    /// The player's Lua policy (`policy.rs`), when `--policy`: none until the player sets one.
    policy_on: bool,
    pub(super) policy: Option<Policy>,
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
    /// When each builder, lab or group (by name) was last asked.
    last_asked: HashMap<String, i32>,
    /// Units a lab has been told to build and not yet started, oldest first.
    pub(super) lab_queue: HashMap<UnitId, Vec<(UnitDefId, i32)>>,
    pub(super) groups: Vec<Group>,
    next_group: usize,
    /// Spots where the engine refused an extractor, and until when they are left off the menus (H-HANDS-REFUSED).
    pub(super) refused_spots: HashMap<usize, i32>,
    /// Units each lab has started since its allowance was set, by unit name (`produce` caps, "corck:1").
    pub(super) produced: HashMap<(UnitId, String), usize>,
    /// The allowance each lab (by name) was last seen with; a change restarts its counts.
    pub(super) allowed_seen: HashMap<String, Vec<String>>,
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
    needs_player_run: u32,
    last_player_wake: i32,
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

    /// The hands: Jev from the environment (the key file or `TYPESAFE_API_KEY`) when `jev`, the policy runtime when
    /// `policy`, either or both; `Err` says why Jev cannot be had.
    pub fn new(jev: bool, policy: bool, log_dir: &Path, ai_id: i32) -> Result<Pianist, String> {
        let client = if jev { Some(jev::Client::from_env().map_err(|e| e.to_string())?) } else { None };
        let seconds: f32 = std::env::var("WITHIN_REASON_JEV_INTERVAL").ok().and_then(|v| v.parse().ok()).unwrap_or(INTERVAL_SECONDS);
        let log = match std::env::var("WITHIN_REASON_JEV_LOG").ok().filter(|v| !v.is_empty() && v != "0") {
            Some(_) => File::create(log_dir.join(format!("jev-{ai_id}.jsonl"))).ok(),
            None => None,
        };
        let worker = if jev && crate::strategist::realtime() { jev::Client::from_env().ok().map(spawn_worker) } else { None };
        Ok(Pianist {
            client,
            policy_on: policy,
            policy: None,
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
            refused_spots: HashMap::new(),
            produced: HashMap::new(),
            allowed_seen: HashMap::new(),
            places: Vec::new(),
            parties: Vec::new(),
            next_party: std::cell::Cell::new(1),
            recent: VecDeque::new(),
            scripts: HashMap::new(),
            done: Vec::new(),
            needs_player_run: 0,
            last_player_wake: i32::MIN / 2,
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
                "interval_frames": self.interval_frames, "rules": rules,
            });
            let _ = writeln!(log, "{line}");
        }
    }

    /// The hands by name, for the banner and the log: "Jev jev-latest", "policy", or both.
    pub fn model(&self) -> String {
        match (&self.client, self.policy_on) {
            (Some(client), true) => format!("Jev {} + policy", client.model()),
            (Some(client), false) => format!("Jev {}", client.model()),
            (None, _) => "policy".to_string(),
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
    pub(super) fn run_pianist(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        self.pianist_housekeeping(tick, kit, commands);
        self.keep_groups(tick, kit, commands);
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
        let due = {
            let pianist = self.pianist.as_ref().expect("pianist mode");
            // Realtime: one request in flight at a time; this second's ask waits for the answer.
            pianist.pending.is_none() && tick.frame - pianist.last_ask_frame >= pianist.interval_frames
        };
        if !due {
            return;
        }
        self.pianist.as_mut().expect("pianist mode").last_ask_frame = tick.frame;
        if let Some(shared) = &self.strategist {
            let lists = std::mem::take(&mut *shared.queues.lock().unwrap());
            let pianist = self.pianist.as_mut().expect("pianist mode");
            for (name, list) in lists {
                match list {
                    Some(steps) => {
                        pianist.scripts.insert(name, steps.into());
                    }
                    None => {
                        pianist.scripts.remove(&name);
                    }
                }
            }
        }
        self.apply_policy_changes(tick.frame);
        let picture = self.picture(tick, kit);
        let mut menus = self.menus(tick, kit, &picture);
        if menus.is_empty() {
            return;
        }
        {
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.places = picture.places.clone();
            pianist.parties = picture.parties.clone();
        }
        // The policy takes the actors it names; the rest go to Jev, or keep their course without it.
        self.policy_pass(tick, kit, &picture, &mut menus, commands);
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
        if menus.iter().all(|m| matches!(m.actor, menu::Actor::Global)) && menus.len() < 2 && self.pianist.as_ref().expect("pianist mode").policy.is_some() {
            // Only the global questions are left and the policy has the actors: nothing to ask this second.
            self.publish_hands(&picture, &BTreeMap::new());
            return;
        }
        let mut questions: BTreeMap<String, jev::Question> = BTreeMap::new();
        for menu in &menus {
            for (id, question) in &menu.questions {
                questions.insert(id.clone(), question.clone());
            }
        }
        let request = jev::Request { state: picture.state.clone(), questions };
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
                self.play(tick, kit, &picture, menus, &response.answers, commands);
                self.pianist_globals(tick, &response.answers);
                self.publish_hands(&picture, &response.answers);
                self.log_call(tick, &request, &response);
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

    /// The player's `policy` calls since the last ask: a new script, or amendments to the one in force.
    fn apply_policy_changes(&mut self, frame: i32) {
        let Some(shared) = self.strategist.clone() else { return };
        let changes = std::mem::take(&mut *shared.policy.lock().unwrap());
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for change in changes {
            let outcome = match change {
                crate::strategist::shared::PolicyChange::Set(script) => match Policy::set(&script) {
                    Ok(policy) => {
                        let lines = policy.lines();
                        pianist.policy = Some(policy);
                        Ok(format!("policy set ({lines} lines)"))
                    }
                    Err(e) => Err(format!("the new policy failed to load and the old one stands: {e}")),
                },
                crate::strategist::shared::PolicyChange::Amend(chunk) => match pianist.policy.as_mut() {
                    Some(policy) => policy.amend(&chunk).map(|done| format!("policy amended: {}", done.join(", "))).map_err(|e| format!("the amendment failed to load: {e}")),
                    None => Err("no policy is in force to amend".into()),
                },
            };
            let text = match outcome {
                Ok(text) => text,
                Err(text) => text,
            };
            pianist.done.push(format!("{} {text}", picture::clock(frame)));
            pianist.note(frame, text);
        }
    }

    /// The policy's orders for this second: the menus it names are played from its answers at probability one and
    /// taken out of `menus`; an order equal to the actor's current task counts as continue. Returns how many it took.
    fn policy_pass(&mut self, tick: &Tick, kit: &Kit, picture: &picture::Picture, menus: &mut Vec<menu::Menu>, commands: &mut Vec<Command>) -> usize {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        if self.pianist.as_ref().is_none_or(|p| p.policy.is_none()) {
            return 0;
        }
        let groups = self.groups_json(own);
        let places = self.places_json();
        let mut options: BTreeMap<String, BTreeMap<String, serde_json::Value>> = BTreeMap::new();
        for menu in menus.iter() {
            // A builder on a list plays its next step itself (H-HANDS-SCRIPT); the policy is not offered it
            // (policy-2-medium: the policy's assist_lab overrode the commander's opening list every second).
            if matches!(menu.actor, menu::Actor::Global) || menu.scripted.is_some() {
                continue;
            }
            let qid = if matches!(menu.actor, menu::Actor::Lab(_)) { format!("{}.next", menu.name) } else { format!("{}.do", menu.name) };
            let mut criteria = menu.questions.iter().find(|(id, _)| *id == qid).and_then(|(_, q)| if let jev::Question::Choice { criteria, .. } = q { Some(criteria.clone()) } else { None }).unwrap_or_default();
            // The policy reaches everything the builder can build, not only what Jev is offered (the roster design,
            // decision 6): the rest of the menu's options, worded from the glossary.
            for (key, pick) in &menu.options {
                if !criteria.contains_key(key) {
                    let words = match pick {
                        menu::Pick::Building(def) | menu::Pick::BuildingAt(def) => format!("{} (not on the hands' menu; the policy may order it{})", self.unit_words(*def), if matches!(pick, menu::Pick::BuildingAt(_)) { ", with `where`" } else { "" }),
                        _ => "(not on the hands' menu)".to_string(),
                    };
                    criteria.insert(key.clone(), json!(words));
                }
            }
            options.insert(menu.name.clone(), criteria);
        }
        let state = Policy::state_for(&picture.state, &options, &groups, &places, frame);
        let started = std::time::Instant::now();
        let result = self.pianist.as_mut().and_then(|p| p.policy.as_mut()).expect("checked above").decide(&state);
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        let mut answers: BTreeMap<String, jev::Answer> = BTreeMap::new();
        let mut taken: Vec<menu::Menu> = Vec::new();
        let mut illegal: Vec<String> = Vec::new();
        let mut continued: Vec<String> = Vec::new();
        let mut orders_json = serde_json::Map::new();
        let mut error = None;
        match result {
            Err(e) => error = Some(e),
            Ok(orders) => {
                for (actor, order) in orders {
                    orders_json.insert(actor.clone(), json!({ "do": order.choice, "params": order.params }));
                    let Some(i) = menus.iter().position(|m| m.name == actor && !matches!(m.actor, menu::Actor::Global)) else {
                        illegal.push(format!("{actor}: not asked this second"));
                        continue;
                    };
                    if menus[i].scripted.is_some() {
                        illegal.push(format!("{actor}: on a list from `queue`, which plays itself; cancel the list to order it"));
                        continue;
                    }
                    if !menus[i].options.contains_key(&order.choice) {
                        illegal.push(format!("{actor}: {} is not on its menu", order.choice));
                        continue;
                    }
                    if let Some(place) = order.params.get("where").or_else(|| order.params.get("where_scout")).or_else(|| order.params.get("where_extractor"))
                        && !picture.places.iter().any(|p| p.name == *place)
                    {
                        illegal.push(format!("{actor}: {place} is not a place in the picture"));
                        continue;
                    }
                    let mut menu = menus.remove(i);
                    if self.same_as_current(&menu, &order) {
                        continued.push(actor.clone());
                        continue;
                    }
                    menu.policy = true;
                    let qid = if matches!(menu.actor, menu::Actor::Lab(_)) { format!("{actor}.next") } else { format!("{actor}.do") };
                    let sure = |choice: &str| jev::Answer::Choice { choice: choice.to_string(), probabilities: BTreeMap::from([(choice.to_string(), 1.0)]), confidence: 1.0 };
                    answers.insert(qid, sure(&order.choice));
                    for (k, v) in &order.params {
                        answers.insert(format!("{actor}.{k}"), sure(v));
                    }
                    if order.choice == "extractor"
                        && let Some(place) = order.params.get("where")
                        && !order.params.contains_key("where_extractor")
                    {
                        answers.insert(format!("{actor}.where_extractor"), sure(place));
                    }
                    taken.push(menu);
                }
            }
        }
        let n = taken.len();
        {
            let policy = self.pianist.as_mut().and_then(|p| p.policy.as_mut()).expect("checked above");
            for menu in &taken {
                *policy.stats.given.entry(answers.get(&format!("{}.do", menu.name)).or_else(|| answers.get(&format!("{}.next", menu.name))).and_then(|a| if let jev::Answer::Choice { choice, .. } = a { Some(choice.clone()) } else { None }).unwrap_or_default()).or_insert(0) += 1;
            }
            if !continued.is_empty() {
                *policy.stats.given.entry("continue".into()).or_insert(0) += continued.len() as u32;
            }
            if policy.stats.illegal.len() < 20 {
                policy.stats.illegal.extend(illegal.iter().cloned());
            }
        }
        if let Some(e) = &error
            && self.pianist.as_ref().and_then(|p| p.policy.as_ref()).is_some_and(|p| p.stats.errors == 1)
        {
            let text = format!("policy error: {e}");
            let pianist = self.pianist.as_mut().expect("pianist mode");
            pianist.done.push(format!("{} {text}", picture::clock(frame)));
            pianist.note(frame, text);
        }
        if n > 0 {
            self.play(tick, kit, picture, taken, &answers, commands);
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        let played = std::mem::take(&mut pianist.played);
        if let Some(log) = &mut pianist.log {
            // Without Jev this line is the only record of the picture (the scorecard and the audit read it).
            let mut state = picture.state.clone();
            if let Some(fields) = state.as_object_mut() {
                fields.remove("instructions");
                fields.remove("rules");
            }
            let line = json!({ "t": "policy", "f": frame, "ms": ms, "orders": orders_json, "error": error, "illegal": illegal, "continued": continued, "played": played, "version": pianist.policy.as_ref().map_or(0, |p| p.version), "state": if pianist.client.is_none() { state } else { Value::Null }, "groups": groups, "places": places });
            let _ = writeln!(log, "{line}");
        }
        n
    }

    /// An order that is what the actor is doing already: a group advancing to that place or holding, a builder
    /// helping that factory or walking there. Builds are covered by H-HANDS-STARTED in `play_one`.
    fn same_as_current(&self, menu: &menu::Menu, order: &policy::Order) -> bool {
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

    /// The groups as the log and the policy see them: name, members, centre, task.
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
            while let Ok((id, result)) = worker.from.try_recv() {
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
                self.play(tick, kit, &pending.picture, pending.menus, &response.answers, commands);
                self.pianist_globals(tick, &response.answers);
                self.publish_hands(&pending.picture, &response.answers);
                self.log_call(tick, &pending.request, &response);
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
        let names: Vec<(UnitId, String, Vec3)> = own.iter().map(|u| (u.id, self.name(u.def).to_string(), u.pos)).collect();
        let mut notes: Vec<String> = Vec::new();
        // What did not happen, for the player's report as well as the picture.
        let mut done: Vec<String> = Vec::new();
        let mut refused: Vec<(UnitId, UnitDefId, Vec3)> = Vec::new();
        let Some(mut pianist) = self.pianist.take() else { return };
        pianist.tasks.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.queued.retain(|id, _| own.iter().any(|u| u.id == *id));
        pianist.lab_queue.retain(|id, _| own.iter().any(|u| u.id == *id));
        for event in &tick.events {
            match *event {
                Event::UnitCreated { unit, builder: Some(builder) } => {
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
                    if let Some((_, name, pos)) = names.iter().find(|(id, _, _)| *id == unit)
                        && let Some(def) = own.iter().find(|u| u.id == unit).map(|u| u.def)
                        && self.world.def(def).is_some_and(|d| d.speed == 0.0)
                    {
                        notes.push(format!("finished {name} at {}", self.world.grid(*pos)));
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
        hands.policy_on = pianist.policy_on;
        if let Some(policy) = &mut pianist.policy {
            hands.policy_stats.merge(std::mem::take(&mut policy.stats));
            hands.policy_text = policy.text();
            hands.policy_version = policy.version;
        }
        hands.engaged = pianist.played.iter().filter(|p| p["did"].as_str().is_some_and(|d| d.starts_with("attack "))).filter_map(|p| p["actor"].as_str().map(str::to_string)).collect();
        hands.globals = answers.iter().filter_map(|(id, a)| id.strip_prefix("global.").map(|q| (q.to_string(), a.probability_of("yes")))).collect();
    }

    /// The global answers: the player's wake (H-HANDS-PLAYER-WAKE).
    fn pianist_globals(&mut self, tick: &Tick, answers: &BTreeMap<String, jev::Answer>) {
        let needs = answers.get("global.needs_player").map_or(0.0, |a| a.probability_of("yes"));
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.needs_player_run = if needs >= NEEDS_PLAYER_PROBABILITY { pianist.needs_player_run + 1 } else { 0 };
        if pianist.needs_player_run >= NEEDS_PLAYER_CALLS && tick.frame - pianist.last_player_wake >= NEEDS_PLAYER_COOLDOWN {
            pianist.last_player_wake = tick.frame;
            pianist.needs_player_run = 0;
            if let Some(shared) = &self.strategist {
                shared.trigger(format!("your hands say the situation needs you (probability {needs:.2} for three seconds running)"));
            }
        }
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
            "[ai {}] f={frame} pianist this minute: {} calls ({} failed, {} dropped), median {median:.0} ms, longest {max:.0}, {} tokens in, {} questions; busy actors changed course {} times, kept {}; groups {}, tasks {}",
            self.world.hello.ai_id, stats.calls, stats.errors, stats.dropped, stats.tokens, stats.questions, stats.switches, stats.kept, pianist.groups.len(), pianist.tasks.len()
        );
    }
}
