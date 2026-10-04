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
    /// This seat's lobby colour by name (how people in the game call it), empty when the script gave none.
    pub colour: String,
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

/// One named cell of the map as units of ours last saw it. (Player-33: the main group was sent to raid "his
/// southern strip", where the roving Rover had seen every spot empty three minutes before; the report listed the
/// spots never seen and nothing said what had been seen, when, or that it was empty.)
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CellLook {
    pub cell: String,
    /// The share of the cell's ground ever within sight of a unit of ours.
    pub seen_share: f32,
    /// Game seconds since the look (the middle one of its seen tiles was last in sight); `None` when never seen.
    pub ago: Option<i32>,
    /// His buildings remembered in the cell (seen and not seen destroyed): internal name and count.
    pub his: Vec<(String, usize)>,
    /// His commander's last sighting was in this cell.
    pub his_commander: bool,
    pub in_his_box: bool,
    /// The metal spots in the cell, by number.
    pub spots: Vec<usize>,
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
    /// What it is doing to us now, the verb with it ("killing ...", "taking apart ..."), from the hits and reclaims
    /// of the last seconds (H-HANDS-PARTY-KILLING), with the victims' metal.
    pub harming: Option<(String, f32)>,
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
    /// Every unit the commander reaches by build lists: the faction's whole roster, with metal cost. Merged over
    /// seats of different factions (a lobby seat on side Random), it holds both factions' units.
    pub roster: Vec<(String, u32)>,
    /// This seat's faction as the start script names it (Armada, Cortex, Legion); merged, every seat's with its team.
    pub factions: Vec<(i32, String)>,
    pub score: Score,
    /// Wreck fields known: place, metal, whether it is safe to work (no enemy in sight near it, a way to walk there).
    pub wreck_fields: Vec<(Place, u32, bool)>,
    pub resurrection_bots: usize,
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
    pub next_free: Vec<(usize, Place, u32)>,
    /// Spots the opponent is known to hold (its extractors seen and not seen dead): each spot's number in the map's
    /// list and its place, nearest our start first. (Player-18: the line said "known to hold 12" and never where,
    /// and the whole army went four times at a strip where nothing of his stood, his extractors being on the
    /// D1-E4 column and at his start.)
    pub enemy_spots: Vec<(usize, Place)>,
    pub soldiers: usize,
    pub army_metal: u32,
    pub soldiers_near_home: usize,
    /// Our soldiers standing in his half now, and their metal; and how long since anything of ours last stood there
    /// (None: never). The pressure the players ask for, as a fact beside the floor on his army (H-PLAYER-HIS-HALF).
    pub soldiers_in_his_half: usize,
    pub army_metal_in_his_half: u32,
    pub seconds_since_ours_in_his_half: Option<i32>,
    /// Opponent extractors seen and not known to be dead: a floor, since we see little of their side.
    pub metal_income: f32,
    /// (minutes ago, extractors, metal income, army metal) for 3 and 6 minutes ago, when the game is that old.
    pub trend: Vec<(i32, usize, f32, u32)>,
    pub extractors_lost_3_min: usize,
    /// Metal of ours destroyed and of theirs we saw destroyed (units and buildings): in the last three minutes, and
    /// in the whole game. Kills out of our sight are not counted. Their side is recomputed at the merge from
    /// `enemy_deaths`, so a death two seats both saw counts once (every ledger trade of 2026-09-27 counted it up to
    /// three times).
    pub traded_3_min: (u32, u32),
    pub traded: (u32, u32),
    /// Every enemy death this seat saw: unit id, frame, metal.
    pub enemy_deaths: Vec<(u32, i32, u32)>,
    /// The frame this score was taken at.
    pub frame: i32,
    /// The map's name and this seat's faction ("arm", "cor"), for the report's reference line.
    pub map: String,
    pub faction: String,
    /// Metal of his buildings seen and not seen destroyed: with the deaths and the soldiers seen, a floor on what
    /// he has spent.
    pub enemy_buildings_metal: u32,
    /// Game seconds since the commander's previous turn began; `None` before its first.
    pub seconds_since_turn: Option<i32>,
    /// The opponent's lobby start boxes, one per enemy ally team, as grid cell ranges: where its commander was
    /// placed at 0:00 and nothing more (docs/design/2026-09-22-enemy-evidence.md).
    pub enemy_start_boxes: Vec<String>,
    /// Every named cell of the map as units of ours last saw it, columns A to H and rows 1 to 8 in order (the
    /// report's `scouting` block, `docs/design/2026-10-01-scouting-glance.md`).
    pub scouting: Vec<CellLook>,
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
}

impl TurnOutputs {
    /// This state with the consumed fields cut to what `before` did not have.
    fn delta_from(mut self, before: &TurnOutputs) -> TurnOutputs {
        self.queues.retain(|k, v| before.queues.get(k) != Some(v));
        // Removals and standing changes are appended during a turn and never reordered: the tail is the turn's.
        if self.removals.len() >= before.removals.len() {
            self.removals.drain(..before.removals.len());
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
}

/// The side's party names: enemy unit id to the party it was last named in, and the next number.
#[derive(Default)]
pub struct PartyRegistry {
    pub by_unit: std::collections::HashMap<bot_protocol::UnitId, String>,
    pub next: usize,
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

/// A transfer between seats of ours from the `transfer` tool (docs/design/2026-09-27-posing-changes.md §12b):
/// metal and energy from one seat to another, or units (by handle or actor name) to a seat. Each seat's brain takes
/// the ones that are its to give.
#[derive(Clone, Debug)]
pub enum Transfer {
    Resources { from_team: i32, to_team: i32, metal: f32, energy: f32 },
    Units { handles: Vec<String>, to_team: i32 },
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
    /// Losses and kills since the commander last looked ("lost armpw to corak in our half" to count).
    pub fights: Mutex<BTreeMap<String, u32>>,
    /// The commander's own notes, carried across session restarts.
    pub notes: Mutex<Vec<String>>,
    /// The player's standing instructions to the pianist (`instruct` tool), the whole packet, replaced each time.
    pub instructions: Mutex<String>,
    /// Builders' lists from the `queue` tool, by actor name: `Some` replaces the list, `None` cancels it; the brain drains
    /// this at each ask (H-HANDS-SCRIPT).
    pub queues: Mutex<BTreeMap<String, Option<Vec<String>>>>,
    /// What each seat's pianist publishes for the player (`brain/pianist`), by team; the report reads the merge
    /// (`seats.rs` `hands_merged`).
    pub hands: Mutex<BTreeMap<i32, Hands>>,
    /// One name for one enemy party across every seat of ours (H-HANDS-SIDE-PARTIES): the party a unit was last in,
    /// by unit id, and the next number. Each seat named its own parties with a seat tag before, so a party seen by
    /// two seats had two names, and the player's `engage_party` on one seat's name was refused on another (games
    /// 3, 4, 6, 9).
    pub parties: Mutex<PartyRegistry>,
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
    /// republishes them every ask (docs/design/2026-09-22-yard-and-reclaim.md). Per seat (team): a single list was overwritten by whichever seat published last, and the `remove` tool
    /// knew only that seat's units (bluegecko-3v1-comet-catcher-3: `cormex_344` "nothing of ours has that name").
    pub own_cards: Mutex<BTreeMap<i32, Vec<UnitCard>>>,
    /// Unit name to (death blast, self-destruct blast, self-destruct seconds), each blast (radius, damage); once.
    pub blasts: Mutex<BTreeMap<String, (Option<(f32, f32)>, Option<(f32, f32)>, f32)>>,
    /// The `remove` tool's orders, taken by the pianist.
    pub removals: Mutex<Vec<Removal>>,
    /// The `transfer` tool's orders, taken by the seat that owns what is sent.
    pub transfers: Mutex<Vec<Transfer>>,
    pub wake: Mutex<Wake>,
    /// Chat from people in the game, unread by the player: (frame, player number, the line with who said it).
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
    /// The most game seconds a turn's outputs wait under the penalty (`WITHIN_REASON_THINK_CAP`; 0 = no cap). The user,
    /// 2026-09-27: a provider's slow week should not decide the game while the arena iterates.
    pub think_cap: Mutex<f32>,
    /// A turn's orders waiting out their delay: the frame they take effect, and what then becomes live.
    pub delayed: Mutex<Option<(i32, TurnOutputs)>>,
    /// The opening turn, taken before the game (`docs/design/2026-09-30-opening-turn.md`).
    pub opening: Mutex<Opening>,
    /// The first turn of the game has ended, however it ended: the hands may ask and order.
    pub opening_over: std::sync::atomic::AtomicBool,
    /// This is the state of the side's commander (`command.rs`), not of a player: its tools are the commander's.
    pub commander: std::sync::atomic::AtomicBool,
    /// A player's: the side's state, when the game has a commander. The brain publishes there what it publishes
    /// here, and the player's report opens with the direction written there.
    pub side: std::sync::OnceLock<std::sync::Arc<Shared>>,
    /// The side's: the players under the commander, for their notes and packets.
    pub players: Mutex<super::command::Players>,
    /// The side's: the chat lines sent for the commander (each seat hears them back and must not take them for a
    /// person's), and the lines of people already heard (every seat hears each).
    pub said: Mutex<Vec<String>>,
    pub heard: Mutex<Vec<(i32, i32, String)>>,
    /// The side's: every death the seats have published, for the commander's fights block (`fights.rs`).
    pub deaths: Mutex<Vec<super::fights::Death>>,
    /// The side's: the commander's direction.
    pub direction: Mutex<super::command::Direction>,
}

/// The opening turn's book: asked for when every seat of ours has said Hello, before any tick.
#[derive(Default)]
pub struct Opening {
    /// Each seat of ours that has said Hello, by team: its line for the turn's prompt.
    pub seats: BTreeMap<i32, String>,
    /// The turn has been asked for.
    pub asked: bool,
    /// The driver has taken the turn's prompt.
    pub prompted: bool,
    /// The first report of the running game still owes the map with our start in it.
    pub map_owed: bool,
}

impl Shared {
    /// The places named for this player's hands: its own marks and, under a commander, the side's (the same name
    /// to every seat; the side's wins a clash).
    pub fn marks_all(&self) -> BTreeMap<String, (f32, f32)> {
        let mut marks = self.marks.lock().unwrap().clone();
        if let Some(side) = self.side.get() {
            marks.extend(side.marks.lock().unwrap().iter().map(|(name, at)| (name.clone(), *at)));
        }
        marks
    }

    /// Brain side, under a commander: a chat line heard by this seat. `false` when it is the commander's own line
    /// coming back or another seat has already passed it on; else it is put before the commander.
    pub fn hear(&self, frame: i32, player: i32, text: &str, line: String) -> bool {
        if self.said.lock().unwrap().iter().any(|s| s == text || text.contains(s.get(..24).unwrap_or(s.as_str()))) {
            return false;
        }
        let mut heard = self.heard.lock().unwrap();
        if heard.iter().any(|(f, p, t)| *p == player && t == text && (frame - f).abs() <= 90) {
            return false;
        }
        heard.push((frame, player, text.to_string()));
        let excess = heard.len().saturating_sub(32);
        heard.drain(..excess);
        self.chat_in.lock().unwrap().push((frame, player, line));
        true
    }

    /// Brain side: a loss or a kill, counted for the player's next report and for the commander's.
    pub fn fight(&self, line: &str) {
        *self.fights.lock().unwrap().entry(line.to_string()).or_default() += 1;
        if let Some(side) = self.side.get() {
            *side.fights.lock().unwrap().entry(line.to_string()).or_default() += 1;
        }
    }

    pub fn trigger(&self, text: String) {
        self.triggers.lock().unwrap().push(text);
    }

    /// Brain side, at Hello: a seat of ours is in, with its line for the opening turn's prompt and the map as it
    /// can be known before the start. When all `expected` seats are in, the opening turn is asked for; nobody
    /// waits here (lockstep waits at its first tick, `hold_for_opening`).
    pub fn seat_said_hello(&self, team: i32, expected: usize, line: String, map: serde_json::Value) {
        {
            let mut opening = self.opening.lock().unwrap();
            opening.seats.insert(team, line);
            if opening.asked || opening.seats.len() < expected {
                return;
            }
            opening.asked = true;
        }
        *self.map.lock().unwrap() = map;
        // Not 0, which reads as "no turn yet" (`wake_commander_if_due`'s fallback first turn).
        self.last_turn_frame.store(1, std::sync::atomic::Ordering::Relaxed);
        self.request_turn("the game has not begun: the opening is yours".into());
    }

    /// Brain side, lockstep, every tick: the game stands until the opening turn asked for at Hello is over, as a
    /// game with people stands in its countdown. Its orders are in force when it ends: no think penalty.
    pub fn hold_for_opening(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        if self.opening_over.load(Relaxed) || !self.opening.lock().unwrap().asked {
            return;
        }
        let gate = self.gate.lock().unwrap();
        let _held = self.gate_changed.wait_while(gate, |g| !g.closed && !self.opening_over.load(Relaxed)).unwrap();
    }

    /// Whether the hands still wait for the player's first turn to end: a session takes turns, it has not gone,
    /// and no turn has ended yet.
    pub fn opening_pending(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        self.gated.load(Relaxed) && !self.opening_over.load(Relaxed) && !self.gate.lock().unwrap().closed
    }

    /// Brain side: ask for a turn and hold the game until it is over. Returns at once when no session can answer.
    /// `opening`: the game's first turn, asked at a tick because no opening turn was asked at Hello; its orders
    /// are in force when it ends, as the opening turn's are.
    pub fn hold_for_turn(&self, reason: String, frame: i32, opening: bool) {
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
        if penalty > 0.0 && !opening {
            let cap = *self.think_cap.lock().unwrap();
            let seconds = started.elapsed().as_secs_f32() * penalty;
            let delay = (if cap > 0.0 { seconds.min(cap) } else { seconds } * 30.0) as i32;
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

    /// Brain side, realtime: a turn is asked for or in hand, so the game is running on orders the player is still
    /// writing: the thinking window, which lockstep's think penalty makes explicit (`brain/wake.rs`, the flight).
    pub fn turn_in_hand(&self) -> bool {
        let gate = self.gate.lock().unwrap();
        gate.in_progress || gate.requested.is_some()
    }

    /// MCP side, at the commander's `wait`: the game runs on and the rest of this response is refused.
    pub fn end_turn_at_wait(&self) {
        self.turn_over.store(true, std::sync::atomic::Ordering::Relaxed);
        self.end_turn();
    }

    pub fn end_turn(&self) {
        let mut gate = self.gate.lock().unwrap();
        gate.in_progress = false;
        self.opening_over.store(true, std::sync::atomic::Ordering::Relaxed);
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;

    /// Realtime: a turn is in hand from the brain's request to its end, and a request made meanwhile is dropped
    /// (the next report carries the state); the wake logic keeps its reasons for after the turn (`brain/wake.rs`).
    #[test]
    fn a_realtime_turn_is_in_hand_from_its_request_to_its_end() {
        let shared = Shared::default();
        assert!(!shared.turn_in_hand());
        shared.request_turn("a raid".into());
        assert!(shared.turn_in_hand());
        shared.request_turn("another".into());
        assert_eq!(shared.gate.lock().unwrap().requested.as_deref(), Some("a raid"), "a second request while one waits is dropped");
        {
            let mut gate = shared.gate.lock().unwrap();
            gate.requested = None;
            gate.in_progress = true;
        }
        assert!(shared.turn_in_hand());
        shared.end_turn();
        assert!(!shared.turn_in_hand());
    }

    /// The opening turn (`docs/design/2026-09-30-opening-turn.md`): asked for when the last seat of ours says Hello,
    /// the lockstep tick held and the hands waiting until a turn ends or the session is gone.
    #[test]
    fn the_opening_turn_is_asked_when_every_seat_is_in_and_its_end_frees_the_game() {
        let shared = std::sync::Arc::new(Shared::default());
        shared.gated.store(true, Relaxed);
        assert!(shared.opening_pending());
        shared.seat_said_hello(0, 2, "commander_t0 (team 0): Armada".into(), serde_json::json!({ "name": "one" }));
        assert!(shared.gate.lock().unwrap().requested.is_none(), "one seat of two");
        shared.hold_for_opening();
        shared.seat_said_hello(1, 2, "commander_t1 (team 1): Cortex".into(), serde_json::json!({ "name": "both" }));
        assert_eq!(shared.gate.lock().unwrap().requested.as_deref(), Some("the game has not begun: the opening is yours"));
        assert_eq!(shared.last_turn_frame.load(Relaxed), 1);
        assert_eq!(shared.map.lock().unwrap()["name"], "both");
        assert_eq!(shared.opening.lock().unwrap().seats.len(), 2);
        let held = shared.clone();
        let tick = std::thread::spawn(move || held.hold_for_opening());
        std::thread::sleep(std::time::Duration::from_millis(60));
        assert!(!tick.is_finished(), "the first tick waits for the turn");
        shared.end_turn();
        tick.join().unwrap();
        assert!(!shared.opening_pending());
        // A session that is gone ends the wait as well.
        let gone = Shared::default();
        gone.gated.store(true, Relaxed);
        gone.seat_said_hello(0, 1, "commander (team 0): Armada".into(), serde_json::Value::Null);
        gone.close_gate();
        gone.hold_for_opening();
        assert!(!gone.opening_pending());
    }

    /// The fallback first turn asked at a tick lands at once under the penalty, as the opening turn does.
    #[test]
    fn the_games_first_turn_is_not_delayed_by_the_think_penalty() {
        let shared = std::sync::Arc::new(Shared::default());
        *shared.think_penalty.lock().unwrap() = 1.0;
        let driver = shared.clone();
        let stop = std::sync::atomic::AtomicBool::new(false);
        let turn = std::thread::spawn(move || {
            driver.next_turn_request(&stop).unwrap();
            *driver.instructions.lock().unwrap() = "the plan".into();
            std::thread::sleep(std::time::Duration::from_millis(40));
            driver.end_turn();
        });
        shared.hold_for_turn("the game begins".into(), 15, true);
        turn.join().unwrap();
        assert!(shared.delayed.lock().unwrap().is_none());
        assert_eq!(*shared.instructions.lock().unwrap(), "the plan");
        assert_eq!(shared.last_landing.load(Relaxed), 15);
    }
}
