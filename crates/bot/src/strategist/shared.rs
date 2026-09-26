//! What the brain and the player's session exchange: the briefing, the field and the hands going up, the turn's orders coming down.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Condvar, Mutex};

use bot_protocol::{Resource};
use serde::Serialize;

/// The brain's summary of the game, published every tick for the strategist to read.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Briefing {
    /// Every side of the game, ours first: who plays each seat (H-PLAYER-SIDES).
    pub sides: Vec<Side>,
    pub game_time: String,
    pub frame: i32,
    pub metal: Resource,
    pub energy: Resource,
    /// The wind now, and the map's range (the people build solar when the wind happens to be low, human-4).
    pub wind: f32,
    pub wind_range: (f32, f32),
    pub counts: Counts,
    pub home: Place,
    pub home_group: Group,
    /// Enemies in sight or radar right now, grouped by map grid cell.
    pub enemies_visible: Vec<EnemyCluster>,
    /// Enemy buildings seen earlier and not known to be destroyed.
    pub enemy_buildings_remembered: Vec<RememberedBuilding>,
    /// Newest last.
    pub recent_events: Vec<String>,
    /// Our seats in this game, one line each; filled by the merge (`seats.rs`).
    pub seats: Vec<super::seats::SeatLine>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Counts {
    pub extractors: usize,
    pub generators: usize,
    pub converters: usize,
    pub labs: usize,
    pub turrets: usize,
    pub constructors: usize,
    pub army: usize,
}

/// A position with its grid cell name, so it can be talked about.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Place {
    pub grid: String,
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Group {
    pub size: usize,
    pub idle: usize,
    pub centre: Option<Place>,
    /// Unit name to count.
    pub composition: Vec<(String, usize)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EnemyCluster {
    pub at: Place,
    pub units: usize,
    /// Unit name to count; radar-only contacts are "unidentified".
    pub composition: Vec<(String, usize)>,
    pub distance_from_home: i32,
    /// What it is shooting now, from the hits of the last seconds (H-HANDS-PARTY-KILLING), with the victims' metal.
    pub killing: Option<(String, f32)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RememberedBuilding {
    pub name: String,
    pub at: Place,
    pub last_seen: String,
}

/// What the player is shown each turn beyond the briefing: the `situation` tool's answer, the roster and buildable
/// checks, the wake conditions' field.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Field {
    /// Our soldiers by unit name, and their centre.
    pub unassigned: Vec<(String, usize)>,
    pub unassigned_centre: Option<Place>,
    pub extractors: Vec<ExtractorStatus>,
    pub turrets: Vec<Place>,
    /// What our standing builders and factories can build now, with metal cost.
    pub buildable: Vec<(String, u32)>,
    /// Every unit the commander reaches by build lists: the faction's whole roster, with metal cost.
    pub roster: Vec<(String, u32)>,
    pub score: Score,
    pub ground: GroundReport,
    /// Wreck fields known: place, metal, whether it is safe to work (held ground, no enemy in sight near it).
    pub wreck_fields: Vec<(Place, u32, bool)>,
    pub resurrection_bots: usize,
}

/// Whose ground is whose, as the bot's territory grid has it (`brain/territory.rs`): held (we can answer there sooner
/// and harder than the opponent can arrive), contested, theirs.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GroundReport {
    /// Free metal spots we can walk to, by the ground they lie on: held, contested, theirs.
    pub free_spots: (usize, usize, usize),
    /// Our extractors standing on ground that is not held.
    pub extractors_exposed: Vec<Place>,
    /// Where the opponent's soldiers were seen or ours died in the last three minutes, the heaviest first, in metal.
    pub raided: Vec<(Place, u32)>,
}

/// How the game stands, in every report: a commander shown only threats plays only defence.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Score {
    pub extractors: usize,
    pub extractor_peak: usize,
    /// Game seconds since our extractor count last reached a new high.
    pub seconds_since_growth: i32,
    /// Metal spots we can walk to that nobody is known to hold: how many, and the nearest few on foot with their
    /// number in the map's list and walking distance. (A count of those "within 2500" read 0 from minute 6 of commander
    /// game 9 while 18 lay at 2700-5400, and the commander made no expansion call for ten minutes.)
    pub free_spots: usize,
    pub next_free: Vec<(usize, Place, u32, &'static str)>,
    /// Spots the opponent is known to hold (its extractors seen and not seen dead).
    pub enemy_spots_seen: usize,
    pub soldiers: usize,
    pub army_metal: u32,
    pub soldiers_near_home: usize,
    /// Opponent extractors seen and not known to be dead: a floor, since we see little of their side.
    pub metal_income: f32,
    /// (minutes ago, extractors, metal income, army metal) for 3 and 6 minutes ago, when the game is that old.
    pub trend: Vec<(i32, usize, f32, u32)>,
    pub extractors_lost_3_min: usize,
    /// Metal of ours destroyed and of theirs we saw destroyed (units and buildings): in the last three minutes, and
    /// in the whole game. Kills out of our sight are not counted.
    pub traded_3_min: (u32, u32),
    pub traded: (u32, u32),
    /// Game seconds since the commander's previous turn began; `None` before its first.
    pub seconds_since_turn: Option<i32>,
    /// The opponent's lobby start boxes, one per enemy ally team, as grid cell ranges: where its commander was
    /// placed at 0:00 and nothing more (docs/design/2026-09-22-enemy-evidence.md).
    pub enemy_start_boxes: Vec<String>,
    /// Metal spots never within sight of a unit of ours, nearest home first: number, place, inside an enemy box.
    pub never_looked: Vec<(usize, Place, bool)>,
    /// The opponent's soldiers seen in the last three minutes and not seen to die, and their metal. Older sightings
    /// are left out: most of its soldiers die where we cannot see, and a count that never forgets only grows.
    pub enemy_soldiers_seen: usize,
    pub enemy_soldiers_seen_metal: u32,
    /// Its factories seen and not seen destroyed; those seen destroyed or found razed, with the game second; and its
    /// commander's last sighting with its age in seconds.
    /// Enemy factories standing as far as we know: the unit's internal name and where.
    pub enemy_factories: Vec<(String, Place)>,
    pub enemy_factories_gone: Vec<(Place, i32)>,
    pub enemy_commander: Option<(Place, i32)>,
    /// Its last seen position is off ground our bots can walk to: in the sea, where only amphibians follow.
    pub enemy_commander_afloat: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExtractorStatus {
    pub at: Place,
    /// The spot's number in the map's list, as the `expansion` tool takes it.
    pub spot: Option<usize>,
    pub enemies_within_600: usize,
    pub turret_within_300: bool,
}

/// When the commander wants to be woken. It sets these itself (`wait` tool); they hold until changed.
#[derive(Clone, Debug, Serialize)]
pub struct Wake {
    /// Game seconds after which it is woken whatever happens.
    pub max_seconds: u32,
    /// Enemies appear within 600 of an extractor that had none near it.
    pub enemy_near_extractor: bool,
    /// A posted squad starts fighting.
    pub squad_engaged: bool,
    pub extractor_lost: bool,
    /// Unit name to count: woken when that many of the type stand unassigned.
    pub pool_reaches: BTreeMap<String, usize>,
    /// A person in the game says something.
    pub chat: bool,
}

impl Default for Wake {
    fn default() -> Self {
        Wake { max_seconds: 30, enemy_near_extractor: true, squad_engaged: true, extractor_lost: true, pool_reaches: BTreeMap::new(), chat: true }
    }
}

pub use micro::Footwork;

/// What one turn changed, held back by the think penalty (`hold_for_turn`) and put into force by `apply_delayed`.
/// Whole-state fields carry the turn's final value; the consumed ones (`queues`, `removals`, `standing`) carry only
/// what the turn added, so nothing the brain already took is played twice.
#[derive(Clone, Debug)]
pub struct TurnOutputs {
    pub wake: Wake,
    pub instructions: String,
    pub queues: BTreeMap<String, Option<Vec<String>>>,
    pub lane: BTreeMap<String, Footwork>,
    pub marks: BTreeMap<String, (f32, f32)>,
    pub allowed: BTreeMap<String, Allowance>,
    pub removals: Vec<Removal>,
    pub standing: Vec<StandingChange>,
}

impl TurnOutputs {
    /// This state with the consumed fields cut to what `before` did not have.
    fn delta_from(mut self, before: &TurnOutputs) -> TurnOutputs {
        self.queues.retain(|k, v| before.queues.get(k) != Some(v));
        // Removals and standing changes are appended during a turn and never reordered: the tail is the turn's.
        if self.removals.len() >= before.removals.len() {
            self.removals.drain(..before.removals.len());
        }
        if self.standing.len() >= before.standing.len() {
            self.standing.drain(..before.standing.len());
        }
        self
    }
}

/// The pianist's side of the player's report (`docs/design/2026-09-21-pianist.md`, "The player"): the picture Jev
/// was last shown (without the instructions and rules), what the hands did since the player's last turn, the groups
/// that began an engagement in the last call, and Jev's global judgements.
#[derive(Clone, Debug, Default)]
pub struct Hands {
    pub picture: serde_json::Value,
    /// "m:ss actor: what", oldest first; drained by the driver at each turn.
    pub done: Vec<String>,
    pub engaged: Vec<String>,
    /// The standing orders (`brain/pianist/standing.rs`): what is in force, and the counts from the packet and
    /// from the tool.
    pub standing_text: String,
    pub standing_counts: (usize, usize),
    /// Whether the packet is read into standing orders at all (`WITHIN_REASON_RULES`); false: prose every second.
    pub packet_rules: bool,
}

/// A change to the standing orders from the `standing` tool: rules per actor to set (checked by the brain against
/// the vocabulary and the picture), or the actors whose tool orders to clear (None: all).
#[derive(Clone, Debug)]
pub enum StandingChange {
    Set(BTreeMap<String, serde_json::Value>),
    Clear(Option<Vec<String>>),
}

/// Lockstep turns: the brain asks for a turn and holds the game (its reply to the engine) until the turn is over.
#[derive(Default)]
pub struct Gate {
    /// Why the commander is being woken; taken by the driver.
    pub requested: Option<String>,
    pub in_progress: bool,
    /// The session is gone: never hold the game again.
    pub closed: bool,
}

/// What the `plan` and `search` tools simulate from: the game's unit table and ground (once), the scenario and the
/// state as of `frame`, and the standing builders' actor names by plan queue (docs/design/2026-09-22-plan-search.md,
/// decision 8). Published by the brain every few seconds; the tools never touch the brain.
pub struct PlanContext {
    pub game: std::sync::Arc<buildorder::game::Game>,
    pub scenario: buildorder::sim::Scenario,
    pub state: buildorder::sim::State,
    /// (actor name as the picture names it, plan queue index): the commander is queue 0, then the factories in
    /// order, then the mobile builders.
    pub actors: Vec<(String, usize)>,
    pub standing_factories: usize,
    pub standing_constructors: usize,
    /// Every metal spot of the map, by the picture's spot number.
    pub spots: Vec<(f64, f64)>,
    /// How far from a spot's centre a site still names that spot (`MapInfo::spot_radius`).
    pub spot_radius: f64,
    pub turret: Option<usize>,
    pub water: bool,
    pub wind: f64,
    pub frame: i32,
}

/// One unit of ours as the player may name it: `handle` is `<unit>_<id>` (armsolar_31002), `actor` the name the
/// picture gives a builder or factory (constructor_N, plant_N, commander).
#[derive(Clone, Debug, Default)]
pub struct UnitCard {
    pub handle: String,
    pub actor: Option<String>,
    pub unit: String,
    pub at: (f32, f32),
    pub health: f32,
    pub metal: f32,
    /// (radius, damage) of its self-destruct; None for none.
    pub self_destruct: Option<(f32, f32)>,
    pub self_destruct_seconds: f32,
}

/// What the player wants removed (`remove` tool): taken apart by a builder (most of the metal comes back) or blown up.
#[derive(Clone, Debug)]
pub enum Removal {
    Reclaim { targets: Vec<String>, by: Option<String> },
    Destruct { targets: Vec<String> },
}

/// State shared between the brain's thread, the MCP server and the strategist driver.
#[derive(Default)]
pub struct Shared {
    pub plan_context: Mutex<Option<std::sync::Arc<PlanContext>>>,
    /// Answers of searches run beside the game, for the player's next report.
    pub search_results: Mutex<Vec<String>>,
    /// The commander ended its turn with `wait` and has not been given its next report yet: it may order nothing.
    /// (Given an immediate answer to `wait`, it took the wait to be over and went on polling and ordering on the
    /// running game, in one endless response: commander game 10, first attempt.)
    pub turn_over: std::sync::atomic::AtomicBool,
    /// What each seat of ours last published; read merged, through `briefing()` and `field()` (`seats.rs`).
    pub seats: Mutex<BTreeMap<i32, super::seats::SeatView>>,
    /// When the commander's last turn began (team-wide: whichever seat leads asks for the next).
    pub last_turn_frame: std::sync::atomic::AtomicI32,
    /// The frame the last turn's orders came into force (at once without a penalty), for the report's landing line.
    pub last_landing: std::sync::atomic::AtomicI32,
    /// What the API backend has spent this game, in millionths of a dollar, across session restarts; and whether it
    /// reached the cap (the player is silent from then on).
    pub spent_micro_usd: std::sync::atomic::AtomicU64,
    pub cost_capped: std::sync::atomic::AtomicBool,
    /// Things worth waking the strategist for, drained by the driver.
    pub triggers: Mutex<Vec<String>>,
    /// Static map description, filled once at game start.
    pub map: Mutex<serde_json::Value>,
    /// The territory grid as text, redrawn by the lead seat: one character per 256-elmo cell.
    pub ground_sketch: Mutex<Vec<String>>,
    /// Losses and kills since the commander last looked ("lost armpw to corak in our half" to count).
    pub fights: Mutex<BTreeMap<String, u32>>,
    /// The commander's own notes, carried across session restarts.
    pub notes: Mutex<Vec<String>>,
    /// The player's standing instructions to the pianist (`instruct` tool), the whole packet, replaced each time.
    pub instructions: Mutex<String>,
    /// Builders' lists from the `queue` tool, by actor name: `Some` replaces the list, `None` cancels it; the brain drains
    /// this at each ask (H-HANDS-SCRIPT).
    pub queues: Mutex<BTreeMap<String, Option<Vec<String>>>>,
    /// What the pianist publishes for the player (`brain/pianist`), read into its turn report.
    pub hands: Mutex<Hands>,
    /// The `standing` tool's changes, taken by the brain at its next ask.
    pub standing: Mutex<Vec<StandingChange>>,
    /// The player's footwork settings by group name (`group_A`) or `all` (`lane` tool, H-HANDS-LANE).
    pub lane: Mutex<BTreeMap<String, Footwork>>,
    /// Places the player named (`mark` tool): name to (x, z). They join the picture's places (H-HANDS-NAMED-PLACES).
    pub marks: Mutex<BTreeMap<String, (f32, f32)>>,
    /// What the player lets each lab build (`produce` tool, H-HANDS-PRODUCE): lab name (`lab_N`) or `all` to unit
    /// names; a lab not listed builds anything.
    pub allowed: Mutex<BTreeMap<String, Allowance>>,
    /// Counts `produce` calls, so a list re-issued word for word is a fresh allowance (escalate-5: `armcv:2` asked
    /// again at 4:15 and 4:43 gave no constructor, the counts having been spent on the same list at 1:15).
    pub produce_calls: AtomicU64,
    /// One card per finished unit of ours, by the handle the player names it with (`remove` tool); the pianist
    /// republishes them every ask (docs/design/2026-09-22-yard-and-reclaim.md).
    pub own_cards: Mutex<Vec<UnitCard>>,
    /// Unit name to (death blast, self-destruct blast, self-destruct seconds), each blast (radius, damage); once.
    pub blasts: Mutex<BTreeMap<String, (Option<(f32, f32)>, Option<(f32, f32)>, f32)>>,
    /// The `remove` tool's orders, taken by the pianist.
    pub removals: Mutex<Vec<Removal>>,
    pub wake: Mutex<Wake>,
    /// Chat from people in the game, unread by the player: (frame, player number, text).
    pub chat_in: Mutex<Vec<(i32, i32, String)>>,
    /// What the player wants said in the game chat (`say` tool), sent by the brain on its next tick.
    pub chat_out: Mutex<Vec<String>>,
    /// True when turns are taken in lockstep with the game (the field commander, the player in arena study runs).
    pub lockstep: std::sync::atomic::AtomicBool,
    /// True when a commander or player session takes turns at the brain's request at all (either way of holding).
    pub gated: std::sync::atomic::AtomicBool,
    pub gate: Mutex<Gate>,
    pub gate_changed: Condvar,
    /// WITHIN_REASON_THINK_PENALTY: the commander's orders take effect this many game seconds late per wall second
    /// it thought (1 = as if the game had kept running while it thought; 0 = at once). Set once at start.
    pub think_penalty: Mutex<f32>,
    /// A turn's orders waiting out their delay: the frame they take effect, and what then becomes live.
    pub delayed: Mutex<Option<(i32, TurnOutputs)>>,
}

impl Shared {
    pub fn trigger(&self, text: String) {
        self.triggers.lock().unwrap().push(text);
    }

    /// Brain side: ask for a turn and hold the game until it is over. Returns at once when no session can answer.
    pub fn hold_for_turn(&self, reason: String, frame: i32) {
        let before = self.outputs();
        let started = std::time::Instant::now();
        {
            let mut gate = self.gate.lock().unwrap();
            if gate.closed {
                return;
            }
            gate.requested = Some(reason);
            self.gate_changed.notify_all();
            let _gate = self.gate_changed.wait_while(gate, |g| !g.closed && (g.requested.is_some() || g.in_progress)).unwrap();
        }
        // The game stood still while the player thought. With a penalty, what it ordered waits as long (in game time)
        // as it took to decide, and the old orders stand meanwhile: the latency it would have in a live game. Until
        // 2026-09-23 only the commander mode's outputs waited; the player's packet, lists, production, marks, policy,
        // footwork and removals landed at once, so the penalty did nothing in a pianist or Lua game (the user: "we
        // want to compress arena games to get through them faster, but otherwise be representative of realtime games").
        let penalty = *self.think_penalty.lock().unwrap();
        if penalty > 0.0 {
            let delay = (started.elapsed().as_secs_f32() * penalty * 30.0) as i32;
            let after = self.outputs();
            self.restore(before.clone());
            *self.delayed.lock().unwrap() = Some((frame + delay, after.delta_from(&before)));
        } else {
            self.last_landing.store(frame, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Everything a turn can change that the game reads.
    fn outputs(&self) -> TurnOutputs {
        TurnOutputs {
            wake: self.wake.lock().unwrap().clone(),
            instructions: self.instructions.lock().unwrap().clone(),
            queues: self.queues.lock().unwrap().clone(),
            lane: self.lane.lock().unwrap().clone(),
            marks: self.marks.lock().unwrap().clone(),
            allowed: self.allowed.lock().unwrap().clone(),
            removals: self.removals.lock().unwrap().clone(),
            standing: self.standing.lock().unwrap().clone(),
        }
    }

    fn restore(&self, o: TurnOutputs) {
        *self.wake.lock().unwrap() = o.wake;
        *self.instructions.lock().unwrap() = o.instructions;
        *self.queues.lock().unwrap() = o.queues;
        *self.lane.lock().unwrap() = o.lane;
        *self.marks.lock().unwrap() = o.marks;
        *self.allowed.lock().unwrap() = o.allowed;
        *self.removals.lock().unwrap() = o.removals;
        *self.standing.lock().unwrap() = o.standing;
    }

    /// Brain side, realtime: ask for a turn and go on; the game does not wait (the user, 2026-09-22: "the main
    /// requirement for realtime is to disable pausing"). A request while a turn is in hand is dropped: the next
    /// report carries the state anyway.
    pub fn request_turn(&self, reason: String) {
        let mut gate = self.gate.lock().unwrap();
        if gate.closed || gate.in_progress || gate.requested.is_some() {
            return;
        }
        gate.requested = Some(reason);
        self.gate_changed.notify_all();
    }

    /// Brain side, every tick: puts a delayed turn's orders into force when their time has come. True while one waits.
    pub fn apply_delayed(&self, frame: i32) -> bool {
        let mut delayed = self.delayed.lock().unwrap();
        match delayed.take() {
            Some((at, o)) if frame >= at => {
                // The turn's outputs land: the whole-state fields are set, the consumed ones (lists, removals,
                // standing changes) are added to what has come since.
                self.last_landing.store(frame, std::sync::atomic::Ordering::Relaxed);
                *self.wake.lock().unwrap() = o.wake;
                *self.instructions.lock().unwrap() = o.instructions;
                *self.lane.lock().unwrap() = o.lane;
                *self.marks.lock().unwrap() = o.marks;
                *self.allowed.lock().unwrap() = o.allowed;
                self.queues.lock().unwrap().extend(o.queues);
                self.removals.lock().unwrap().extend(o.removals);
                self.standing.lock().unwrap().extend(o.standing);
                false
            }
            waiting => {
                let pending = waiting.is_some();
                *delayed = waiting;
                pending
            }
        }
    }

    /// Driver side: wait for the brain to ask for a turn. `None` when `give_up` turns true.
    pub fn next_turn_request(&self, give_up: &std::sync::atomic::AtomicBool) -> Option<String> {
        let mut gate = self.gate.lock().unwrap();
        loop {
            if give_up.load(std::sync::atomic::Ordering::Relaxed) {
                return None;
            }
            if let Some(reason) = gate.requested.take() {
                gate.in_progress = true;
                return Some(reason);
            }
            gate = self.gate_changed.wait_timeout(gate, std::time::Duration::from_millis(250)).unwrap().0;
        }
    }

    /// Driver side: whether the brain is still held for the turn in hand.
    pub fn turn_in_progress(&self) -> bool {
        self.gate.lock().unwrap().in_progress
    }

    /// MCP side, at the commander's `wait`: the game runs on and the rest of this response is refused.
    pub fn end_turn_at_wait(&self) {
        self.turn_over.store(true, std::sync::atomic::Ordering::Relaxed);
        self.end_turn();
    }

    pub fn end_turn(&self) {
        self.gate.lock().unwrap().in_progress = false;
        self.gate_changed.notify_all();
    }

    pub fn close_gate(&self) {
        let mut gate = self.gate.lock().unwrap();
        gate.closed = true;
        gate.in_progress = false;
        self.gate_changed.notify_all();
    }
}

/// What the player lets an actor build (`produce`): the units, with the number of the call that set them, so the
/// hands restart their caps whenever the player speaks again, even in the same words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allowance {
    pub call: u64,
    pub units: Vec<String>,
    /// For a factory: the group its new soldiers join (`group_X`), or `new` for a fresh group of this factory's own
    /// (H-HANDS-GROUPS: groups are the player's, nothing merges by proximity).
    pub group: Option<String>,
}

/// One ally team as the player is told of it: ours or not, and its seats in words ("keithphw (a person, lobby skill
/// 43)", "BARb medium (an AI)", "you (WReason)").
#[derive(Clone, Debug, Default, Serialize)]
pub struct Side {
    pub ours: bool,
    pub ally_team: i32,
    pub seats: Vec<String>,
}

