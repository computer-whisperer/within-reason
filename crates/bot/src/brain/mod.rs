//! The brain: one instance per AI; `decide` runs once per tick. The world model (`routes`, `bases`,
//! `reclaim`, `shelling`, `yards`, the spot survey) is kept here each tick; the decisions are the pianist's
//! (`pianist/`) and the control lane's (`micro.rs`). The heuristic deciders were deleted 2026-09-25
//! (`docs/design/2026-09-25-one-decider.md`).

mod allies;
mod bases;
mod briefing;
mod combat;
mod echo;
mod economy;
mod guns;
pub mod journal;
mod micro;
mod people;
pub mod pianist;
mod planner;
mod scout;
mod block;
mod stand;
mod reclaim;
mod shelling;
mod wake;
pub(crate) mod roster;
mod routes;
mod nanos;
pub use nanos::{HOLDS_KINDS, TEND_CATEGORIES};
mod yards;
mod ways;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// A tick's spacing in frames, for periodic work keyed on `frame % N` (ticks come every few frames).
const TICK_FRAMES_GUESS: i32 = 8;

use std::sync::atomic::Ordering;

use bot_protocol::{ChatTo, Command, Event, OwnUnit, SayTo, Tick, UnitDefId, UnitId, Vec3};

use crate::session::{BrainResult, LaneReport};
use crate::strategist::shared::Shared;
use crate::world::World;
use roster::{Kit, ROSTERS};

/// How long a hit counts as "shooting now" in the picture (H-HANDS-PARTY-KILLING).
const HIT_MEMORY: i32 = 3 * FRAMES_PER_SECOND;

/// One of our units hit by a known enemy unit.
#[derive(Clone, Copy, Debug)]
struct Hit {
    frame: i32,
    victim: UnitId,
    victim_def: UnitDefId,
    attacker: UnitId,
}

/// A building of ours losing health to an enemy builder beside it and to no weapon: it is being taken apart (a
/// reclaim is no hit: the engine sends no damage event for it). `since` and `from` are the frame and health of the
/// first drop seen, `frame` and `health` of the last, for the rate.
#[derive(Clone, Copy, Debug)]
struct Taking {
    victim: UnitId,
    victim_def: UnitDefId,
    taker: UnitId,
    since: i32,
    from: f32,
    frame: i32,
    health: f32,
    max_health: f32,
}

const FRAMES_PER_SECOND: i32 = 30;
/// Frames between runs of the whole brain (`think`), 2 Hz: the shim's ticks come more often than that, for the
/// control lane (`micro.rs`), and must divide this (`docs/design/2026-09-20-micro-lane.md`).
pub(crate) const BRAIN_FRAMES: i32 = 15;
/// How long what allies put into our stores is averaged over (`Event::Received` comes once a second).
const RECEIVED_WINDOW: i32 = 30 * FRAMES_PER_SECOND;

pub struct Brain {
    world: World,
    kit: Option<Kit>,
    home: Vec3,
    /// This tick's allied units, where each allied team's commander was first seen, and when each metal spot last
    /// held an allied extractor (`allies.rs`).
    allies: Vec<bot_protocol::AllyUnit>,
    /// `_t<team>` when we play more than one seat of this game, else empty: the suffix on every per-seat name the
    /// player sees (commander, group_A, party_N, passage_N), so two seats' names never collide in one report
    /// (bluegecko-2v1-great-divide: both commanders were "commander", the Cortex list went to the Armada seat).
    seat_tag: String,
    ally_starts: HashMap<i32, Vec3>,
    ally_spot_held: HashMap<usize, i32>,
    /// The frame each allied team's first tier-2 factory was seen standing (`briefing.rs`, the allies line).
    ally_tier2_since: HashMap<i32, i32>,
    /// Shared with the other seats of ours in this game (`team.rs`), and what they posted last tick.
    board: Arc<crate::team::TeamBoard>,
    team_mates: crate::team::Others,
    /// What this seat posts this tick; the army fills in its part.
    team_post: crate::team::Post,
    /// One per enemy seat: guessed, found or dead (`bases.rs`).
    enemy_bases: Vec<bases::EnemyBase>,
    /// Whether the lobby's start boxes are honoured (our own start inside our box and in no other team's): when
    /// not, the boxes say nothing of where he is (`bases::guess_enemy_bases`).
    pub(crate) boxes_honoured: bool,
    /// Metal spot index to the frame it was claimed at.
    spot_claims: HashMap<usize, i32>,
    /// Walking distances over the terrain, once our faction (and so our movement class) is known.
    routes: Option<routes::Routes>,
    /// On a metal map (no spots published): the metal with the extractors' claims over it (`layout::Coverage`),
    /// rebuilt every tick from what stands, ours, allied and his as seen; where an extractor goes is its best tile
    /// (the metal-maps design §1.3).
    coverage: Option<layout::Coverage>,
    /// On a metal map: each grid cell's share of the map's metal, by the grid's index (the metal-maps design §1.4);
    /// empty on a spot map.
    cell_metal: Vec<f32>,
    /// The commander's stand (`layout::stand`, `docs/design/2026-10-29-base-layout.md` §1.1): the opening's slots
    /// solved once, when its list first plays, each taken by a step of its kind (`place_planned`) and the steps
    /// ordered as one queue (`play_lists`). The builder it was solved for, and the stand.
    stand: Option<(UnitId, layout::Stand)>,
    /// The block (`layout::lay_block`, the design §1.1 step 3; `block.rs`): laid once an advanced plant of ours is
    /// placed, at the player's `block` mark or beside the plant; its rows and gantry slot are reservations and its
    /// sub-lattice is where construction turrets stand. When it was last tried and why it failed, for the report.
    block: Option<layout::Block>,
    block_tried: i32,
    block_failed: Option<String>,
    /// Why the block's next tier of rows was not laid when its slots ran short, for the report (`block.rs`).
    /// The last refusal said for each kind (the menu asks where every kind would go each turn; the same answer is
    /// logged once).
    layout_refusal_said: std::cell::RefCell<HashMap<UnitDefId, String>>,
    /// The failing site searches dumped for replay (`economy.rs` `dump_layout_failure`): how many, and how many a
    /// kind.
    layout_dumped: std::cell::RefCell<(usize, HashMap<UnitDefId, usize>)>,
    /// The site questions of the builder menus' pass under way, kept while the pass is watched for a dump
    /// (`menu.rs` `fill_builder_menus`, `economy.rs` `dump_layout_pass`); None outside a watched pass.
    layout_asks: std::cell::RefCell<Option<Vec<layout::Request>>>,
    /// Each factory type's products' ground, as found for the route survey in force (`passable_for_factory`).
    factory_walks: std::cell::RefCell<HashMap<UnitDefId, Option<layout::Passable>>>,
    /// The slow passes dumped this game and the time of the last of them.
    layout_passes_dumped: std::cell::Cell<(usize, f32)>,
    /// Every site search the layout layer refused this game, by kind: the engine placed those. Said once a minute
    /// while it grows, so a game's pile is sized from the log without its dumps.
    layout_refused: std::cell::RefCell<HashMap<UnitDefId, usize>>,
    /// Wrecks seen, the fields they lie in and who works them (`reclaim.rs`).
    reclaim: reclaim::Reclaim,
    /// The simulator's view of this game and its ground, built once for the `plan` and `search` tools.
    plan_game: Option<(Arc<buildorder::game::Game>, Arc<dyn buildorder::game::Ground>)>,
    /// The pianist (`pianist/`): Jev plays every actor from the player's instructions. Always present since the
    /// heuristic bot was deleted (2026-09-25); `Option` until the pianist's own state is folded in.
    pianist: Option<pianist::Pianist>,
    matchups: combat::Matchups,
    /// The combat simulator's tables and unit types (`micro.rs`).
    sim: micro::Sim,
    /// The tier-1 square-law strength per metal (`combat.rs` `worth_scale`), computed once; 0 until then.
    worth_scale: std::cell::Cell<f32>,
    /// What the bot says in the game chat at its first orders: commit and settings (`main.rs`), once.
    banner: Option<String>,
    /// When each metal spot was last in sight, and the raider out looking (`scout.rs`).
    spots: scout::Spots,
    wake: wake::WakeState,
    /// How often each heuristic (docs/heuristics.md) acted since the last status line.
    fired: BTreeMap<&'static str, u32>,
    /// Present when the player's session is attached; the brain publishes to it and reads its orders.
    strategist: Option<Arc<Shared>>,
    /// Enemy buildings seen and not known to be destroyed: definition, position, frame last seen.
    enemy_buildings: HashMap<UnitId, (UnitDefId, Vec3, i32)>,
    /// Of `enemy_buildings`, the ones last seen still being built: no threat, no shooter, no wall, and a target.
    enemy_unfinished: HashSet<UnitId>,
    /// His units stunned, with the seconds each stays so as of `stun_frame`: from his units in sight, and counted down
    /// for those out of sight since (a stunned turret is no threat until it wakes, seen or not).
    enemy_stunned: HashMap<UnitId, f32>,
    stun_frame: i32,
    /// Hits from out of sight in the last twenty seconds (`shelling.rs`), and when the player was last woken for them.
    shelling: Vec<shelling::Shell>,
    shelling_warned: i32,
    /// Our side's sensor coverage grids as the shim last sent them (once a second) and the frame they came: sight
    /// and radar cut by terrain, the engine's own (`docs/design/2026-10-18-sight-jammers-cloak-shields.md`); None
    /// from a shim before 2026-10-18, when sight is the glossary's circle (`scout.rs`).
    sensors: Option<(bot_protocol::Sensors, i32)>,
    /// Fire from out of sight as standing places (`guns.rs`, H-GUNS): the guns, the count that names them, and the
    /// frame up to which the hits have been read into them.
    guns: Vec<guns::Gun>,
    guns_named: usize,
    guns_until: i32,
    /// Our shots and the hits of ours the engine reported landing, in the last `FIRING_WINDOW` (`guns.rs`).
    fired_log: Vec<(i32, UnitId)>,
    landed_log: Vec<(i32, UnitId)>,
    /// Hits on our units with a known attacker in the last `HIT_MEMORY` frames: what each enemy party is shooting
    /// (H-HANDS-PARTY-KILLING).
    hits: Vec<Hit>,
    /// Buildings of ours an enemy builder is taking apart, seen in the last `HIT_MEMORY` frames, and each building's
    /// health at the last tick (and that tick's frame), which is how a reclaim shows (H-HANDS-PARTY-KILLING).
    takings: Vec<Taking>,
    building_health: HashMap<UnitId, f32>,
    building_health_frame: i32,
    /// Our factories' exit lanes this tick, kept clear of new buildings; our mobile units that cannot move; the
    /// factories whose blocked yard the player has been told of (`yards.rs`).
    stuck: HashMap<UnitId, yards::Stuck>,
    /// The way-out watch (`ways.rs`): each mobile builder's way to the open at the last look, the frame of that
    /// look, our standing footprints' handles then (a pen's closer is one that came up since), the builders penned
    /// and every penned or stuck unit's cheapest way out; the veto's verdicts from the site searches since the
    /// last tick, written to the record by the watch.
    ways: HashMap<UnitId, bool>,
    ways_frame: i32,
    ways_seen: HashSet<String>,
    pens: HashMap<UnitId, ways::Pen>,
    freeings: HashMap<UnitId, ways::Freeing>,
    layout_verdicts: std::cell::RefCell<Vec<(serde_json::Value, serde_json::Value)>>,
    /// Mobile units standing still in a standing factory's exit lane: since when, where they stood, which factory.
    lane_standers: HashMap<UnitId, (i32, Vec3, UnitId)>,
    yard_warned: HashSet<UnitId>,
    /// Construction turrets' standing orders (H-ECO-NANO-GUARD).
    nanos: nanos::Nanos,
    /// Chat lines of ours not yet seen back from the engine, which echoes every line as a chat event from our own
    /// host player (human-1: the player was woken by its own "gl hf").
    said: Vec<String>,
    /// The frame of the last chat line that was not our own echo (the wake reads it; comet-2: the banner's echo woke the player).
    heard_chat_at: i32,
    /// A chat line has come with its destination: the relay widget runs on the host's client.
    chat_relay: bool,
    /// The player has been told (no relay widget, spectator chat off), once each.
    chat_told: (bool, bool),
    /// Buildings `forget_razed_buildings` dropped this tick, for the team board.
    razed: Vec<UnitId>,
    /// Remembered buildings whose place is in sight with nothing there, by the frame that began (the gone rule's
    /// dwell, `track_enemy_buildings`).
    missing_in_sight: HashMap<UnitId, i32>,
    /// Factories of theirs seen destroyed or found razed: definition, position, frame. Evidence for the player of
    /// where the opponent has been (docs/design/2026-09-22-enemy-evidence.md).
    enemy_factories_gone: Vec<(UnitDefId, Vec3, i32)>,
    /// Every enemy soldier seen and not known dead, with when it was last seen: what we know of their army, a floor.
    /// Enemy soldiers seen: definition, where, when (the shelling attribution and the army counts read it).
    enemy_soldiers: HashMap<UnitId, (UnitDefId, Vec3, i32)>,
    /// Where and when the enemy commander was last seen: killing it wins the game.
    enemy_commander_seen: Option<(Vec3, i32)>,
    /// The last frame any unit of ours stood in his half (`his_half`); None until one has (H-PLAYER-HIS-HALF).
    ours_in_his_half: Option<i32>,
    recent_events: VecDeque<String>,
    /// Our units as last seen, to name what a destroyed-unit event refers to.
    known_units: HashMap<UnitId, (UnitDefId, Vec3)>,
    /// Our frames still being built at the last look, each with the furthest along it has been (its peak health
    /// share: a frame left to decay reads near 0 at its end), so a `UnitDestroyed` for one says what was put in.
    unfinished: HashMap<UnitId, f32>,
    /// The builder each frame was created by and the frame it was made, so a decayed frame's line says whether its
    /// builder died (`briefing.rs` `track_losses`), and a `stop` spares a frame begun after the report the player read.
    frame_builders: HashMap<UnitId, (UnitId, i32)>,
    /// Since when each unfinished frame has had no builder of ours on it and its share has not risen (the report's
    /// `unfinished structures` line).
    frame_alone_since: HashMap<UnitId, i32>,
    /// Each unfinished frame's share at the ticks of the last `briefing::RATE_WINDOW_FRAMES`, oldest first: its rate of
    /// building, which is the only read of the construction turrets' work on it (the hands see no turret's target;
    /// econ-flagship-8: an advanced fusion rising a percent a second under fourteen turrets read "nobody is building
    /// it, decaying", and the player bought builders instead of turrets).
    frame_progress: HashMap<UnitId, std::collections::VecDeque<(i32, f32)>>,
    /// The metal that went into frames within a construction turret's reach, by tick, over the last minute: the
    /// block line's measured intake (`block_line`; the farm-holds design §1.4).
    farm_intake: std::collections::VecDeque<(i32, f32)>,
    /// The unfinished frames among this think's `UnitDestroyed` events and what became of each (`track_losses` fills
    /// it; the pianist's housekeeping runs after and words the line).
    frame_losses_now: HashMap<UnitId, briefing::FrameLoss>,
    /// The units that died with a death blast worth naming in the last seconds, his and ours, for the cause of a
    /// loss nobody is named for (`track_losses`).
    recent_blasts: Vec<briefing::RecentBlast>,
    /// Every enemy unit's type once seen, so a killer that has left sight still has a name.
    enemy_defs: HashMap<UnitId, UnitDefId>,
    /// What each other team has been seen to field: team to (faction prefix to the units of it seen), and the
    /// faction of its commander unit once seen (`briefing.rs` `seen_faction`).
    factions_seen: HashMap<i32, (HashMap<&'static str, std::collections::HashSet<UnitId>>, Option<&'static str>)>,
    /// Where each enemy was last seen, for the place of its death (`briefing.rs` `publish_death`).
    enemy_places: HashMap<UnitId, Vec3>,
    /// This minute's fight ledger for the log: "lost X to Y near home" and "killed Y", with counts.
    fight_ledger: std::collections::BTreeMap<String, u32>,
    /// One entry per death that is ours to count: a loss of ours, or a kill by one of our own units (the engine
    /// names the killer; an ally's kill in our sight is not ours, H-PLAYER-OWN-TRADE), with the frame, the metal and
    /// what died: what the fighting costs each side, which neither army count shows.
    trade_log: Vec<briefing::Trade>,
    /// Every enemy death our units made (the merge across seats counts each once).
    enemy_deaths: Vec<crate::strategist::shared::EnemyDeath>,
    /// Types of his seen at least once: the first tier-2 or air unit of a type is said when it appears.
    first_seen: HashSet<UnitDefId>,
    /// Extractors lost so far at each metal spot (its index in the map's list).
    spot_losses: HashMap<usize, u32>,
    /// This minute's soldier move failures by 200-elmo cell.
    stuck_cells: HashMap<(i32, i32), u32>,
    /// Frames at which we lost an extractor, within the trigger cooldown.
    extractor_losses: VecDeque<i32>,
    /// Every unit of ours destroyed by the enemy in the last three minutes: frame, unit, type. The wakes read it (a
    /// group's losses since the player's last orders, losses while its orders were on their way, the wait floor).
    /// (frame, the unit, its type, the killer and its type when the attacker was known and identified).
    pub(crate) unit_losses: VecDeque<(i32, UnitId, UnitDefId, Option<(UnitId, UnitDefId)>)>,
    /// Metal spots (by index) where an extractor offset toward the builder was refused: the exact centre from then on.
    centre_only: HashSet<usize>,
    dropped_orders: u32,
    /// The refused-searches total last said in the minute line (`layout_refused`).
    layout_refused_said: usize,
    move_failures: u32,
    /// Heuristic IDs switched off for an ablation run (`WITHIN_REASON_DISABLE=H-A,H-B`).
    disabled: Vec<String>,
    last_trigger_frame: HashMap<&'static str, i32>,
    /// Decisions since `main` last collected them for the match record.
    journal: journal::Journal,
    /// The lane's declared groups' states and the rovers' reports as the lane last reported them (`session.rs`
    /// `LaneReport`: the lane runs on the session's fast thread, `micro.rs` is the brain's side of it).
    lane_groups: HashMap<String, micro::GroupState>,
    lane_roving: HashMap<UnitId, micro::RoveReport>,
    /// Whether the lane's fixed view has gone out (once, with the first answer after the faction is adopted).
    lane_fixed_sent: bool,
    /// Events from the ticks since `think` last ran, in order, for its next run.
    carried_events: Vec<Event>,
    /// Metal and energy allies put into our stores (`Event::Received`: gifts and overflow, the engine's figure for
    /// the last second, once a second), kept for RECEIVED_WINDOW. The engine's income leaves it out, so a seat fed
    /// by its allies read "in balance" with its store at cap (the Glitters 2v6 of 2026-10-25: 30 to 47 a second
    /// arriving unread from 19:00 to 41:00).
    received: std::collections::VecDeque<(i32, f32, f32)>,
    /// Ticks that came late (`Tick::late`) and the most frames one waited, since the last per-minute line.
    late_ticks: (u32, i32),
    /// How late the engine runs our orders, timed by our own chat lines (H-HANDS-ORDER-LAG).
    echo: echo::Echo,
    /// Units told to self-destruct and the frame their countdown should be over by (`remove`): the engine's order is
    /// a toggle, so a second order inside the countdown would stop it.
    self_destructing: HashMap<UnitId, i32>,
}

/// The engine sends an AI's chat as its host player's (the user, in a game with people), so every line of the
/// player's says who wrote it, and the engine cuts a chat line at 127 characters (bluegecko-3v1-comet-catcher-4:
/// every line over that lost its end), so a long text goes out as several lines that fit. A prefix the player
/// wrote itself is not doubled (the same game: "[WReason] [WReason] Hi!").
pub(crate) const CHAT_PREFIX: &str = "[WReason] ";
const CHAT_LINE: usize = 127;

pub(crate) fn chat_lines(text: &str) -> Vec<String> {
    let mut rest = text.trim();
    while let Some(after) = rest.strip_prefix(CHAT_PREFIX.trim_end()) {
        rest = after.trim_start();
    }
    let room = CHAT_LINE - CHAT_PREFIX.chars().count();
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in rest.split_whitespace() {
        let word: String = if word.chars().count() > room { word.chars().take(room).collect() } else { word.to_string() };
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > room {
            lines.push(format!("{CHAT_PREFIX}{line}"));
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&word);
    }
    if !line.is_empty() {
        lines.push(format!("{CHAT_PREFIX}{line}"));
    }
    lines
}

#[cfg(test)]
mod chat_tests {
    use super::{CHAT_LINE, chat_lines};

    #[test]
    fn chat_is_prefixed_once_and_split_to_fit_the_engine() {
        assert_eq!(chat_lines("[WReason] [WReason] Hi! Good luck."), vec!["[WReason] Hi! Good luck."]);
        let long = "Taking your advice: gathering our north and middle armies at D3, then pushing their base at B3 together. South Incisors hit their B8 expansion.";
        let lines = chat_lines(long);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= CHAT_LINE && l.starts_with("[WReason] ")), "{lines:?}");
        assert_eq!(lines.join(" ").replace("[WReason] ", ""), long);
        assert!(chat_lines("   ").is_empty());
    }
}

impl Brain {
    pub fn new(world: World, strategist: Option<Arc<Shared>>, board: Arc<crate::team::TeamBoard>, banner: String, pianist: Option<pianist::Pianist>) -> Self {
        let h = &world.hello;
        eprintln!(
            "[ai {}] team {} on {} ({}x{}), {} unit defs, {} metal spots",
            h.ai_id, h.team, h.map.name, h.map.width, h.map.height, h.unit_defs.len(), h.metal_spots.len()
        );
        let tag = seat_tag(h);
        let mut pianist = pianist;
        if let Some(p) = pianist.as_mut() {
            p.seat_tag = tag.clone();
        }
        let coverage = world.hello.map.metal_map.then(|| layout::Coverage::new(&world.hello.terrain, world.hello.map.metal_scale, world.hello.map.extractor_radius)).flatten();
        let cell_metal = if world.hello.map.metal_map { terrain::sheet::metal_shares(&world.hello.terrain, &world.cells) } else { Vec::new() };
        Brain {
            cell_metal,
            world,
            kit: None,
            home: Vec3::default(),
            board,
            team_mates: Default::default(),
            team_post: Default::default(),
            allies: Vec::new(),
            seat_tag: tag,
            ally_starts: HashMap::new(),
            ally_spot_held: HashMap::new(),
            ally_tier2_since: HashMap::new(),
            enemy_bases: Vec::new(),
            boxes_honoured: true,
            spot_claims: HashMap::new(),
            routes: None,
            coverage,
            stand: None,
            block: None,
            block_tried: i32::MIN / 2,
            block_failed: None,
            layout_refusal_said: std::cell::RefCell::new(HashMap::new()),
            layout_dumped: Default::default(),
            layout_asks: Default::default(),
            factory_walks: Default::default(),
            layout_passes_dumped: Default::default(),
            layout_refused: Default::default(),
            reclaim: Default::default(),
            plan_game: None,
            pianist,
            matchups: Default::default(),
            sim: Default::default(),
            worth_scale: std::cell::Cell::new(0.0),
            spots: scout::Spots::default(),
            banner: Some(banner),
            wake: Default::default(),
            fired: BTreeMap::new(),
            strategist,
            enemy_buildings: HashMap::new(),
            enemy_unfinished: HashSet::new(),
            enemy_stunned: HashMap::new(),
            stun_frame: 0,
            shelling: Vec::new(),
            sensors: None,
            guns: Vec::new(),
            guns_named: 0,
            guns_until: 0,
            fired_log: Vec::new(),
            landed_log: Vec::new(),
            shelling_warned: i32::MIN / 2,
            hits: Vec::new(),
            takings: Vec::new(),
            building_health: HashMap::new(),
            building_health_frame: 0,
            stuck: HashMap::new(),
            ways: HashMap::new(),
            ways_frame: i32::MIN / 2,
            ways_seen: HashSet::new(),
            pens: HashMap::new(),
            freeings: HashMap::new(),
            layout_verdicts: std::cell::RefCell::new(Vec::new()),
            lane_standers: HashMap::new(),
            yard_warned: HashSet::new(),
            nanos: nanos::Nanos::default(),
            said: Vec::new(),
            heard_chat_at: -1,
            chat_relay: false,
            chat_told: (false, false),
            razed: Vec::new(),
            missing_in_sight: HashMap::new(),
            enemy_factories_gone: Vec::new(),
            enemy_soldiers: HashMap::new(),
            enemy_commander_seen: None,
            ours_in_his_half: None,
            recent_events: VecDeque::new(),
            known_units: HashMap::new(),
            unfinished: HashMap::new(),
            frame_builders: HashMap::new(),
            frame_alone_since: HashMap::new(),
            frame_progress: HashMap::new(),
            farm_intake: std::collections::VecDeque::new(),
            frame_losses_now: HashMap::new(),
            recent_blasts: Vec::new(),
            enemy_defs: HashMap::new(),
            enemy_places: HashMap::new(),
            factions_seen: HashMap::new(),
            fight_ledger: Default::default(),
            trade_log: Vec::new(),
            enemy_deaths: Vec::new(),
            first_seen: HashSet::new(),
            spot_losses: HashMap::new(),
            stuck_cells: HashMap::new(),
            extractor_losses: VecDeque::new(),
            unit_losses: VecDeque::new(),
            centre_only: HashSet::new(),
            dropped_orders: 0,
            layout_refused_said: 0,
            move_failures: 0,
            disabled: std::env::var("WITHIN_REASON_DISABLE").map_or_else(|_| Vec::new(), |ids| ids.split(',').map(str::to_string).collect()),
            last_trigger_frame: HashMap::new(),
            journal: Default::default(),
            lane_groups: HashMap::new(),
            lane_roving: HashMap::new(),
            lane_fixed_sent: false,
            carried_events: Vec::new(),
            received: Default::default(),
            late_ticks: (0, 0),
            echo: Default::default(),
            self_destructing: HashMap::new(),
        }
    }

    /// Chat from the people in the game goes to the player's next report, as a player on our side would be shown it
    /// (H-CHAT-PERMISSIONS: `hears`); what the player said goes into the game (the user, 2026-09-22: in human games,
    /// letting Opus talk "could actually help gain direct feedback from experienced players"). Under a commander
    /// chat is the commander's: people's lines go to it and its lines go out here, by whichever seat comes first
    /// (`docs/design/2026-10-03-commander-seat.md`).
    fn relay_chat(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(shared) = self.strategist.clone() else { return };
        let host_watches = self.host_watches();
        shared.host_watches.store(host_watches, Ordering::Relaxed);
        if let Some(side) = shared.side.get() {
            side.host_watches.store(host_watches, Ordering::Relaxed);
        }
        for event in &tick.events {
            let Event::Chat { player, text, to } = event else { continue };
            if *to != ChatTo::Unknown {
                self.chat_relay = true;
            }
            if self.own_echo(&shared, text) {
                self.read_echo(&shared, tick.frame, *to);
                continue;
            }
            if !self.hears(*player, *to, text) {
                eprintln!("[ai {}] f={} chat from player {player} to {to:?}, not for a player on our side: {text}", self.world.hello.ai_id, tick.frame);
                continue;
            }
            let line = format!("{}: {text}", self.speaker(*player));
            let heard = match shared.side.get() {
                Some(side) => side.hear(tick.frame, *player, text, line),
                None => {
                    shared.chat_in.lock().unwrap().push((tick.frame, *player, line));
                    self.heard_chat_at = tick.frame;
                    true
                }
            };
            if heard {
                eprintln!("[ai {}] f={} chat from player {player} to {to:?}: {text}", self.world.hello.ai_id, tick.frame);
            }
        }
        let out = std::mem::take(&mut *shared.side.get().map_or(shared.as_ref(), |side| side.as_ref()).chat_out.lock().unwrap());
        for (to, text) in out {
            for line in chat_lines(&text) {
                self.remember_said(line.clone());
                commands.push(Command::Say { text: line, to });
            }
        }
    }

    /// A line heard is one of ours coming back (K-mech-chat-echoes-our-own-lines): this seat's, dropped from its
    /// waiting list, or under a commander any seat's of the side (every seat hears each, so the side's list keeps
    /// it). The game cuts a line at about 120 characters, so a long line of ours is known by its first words after
    /// the name the game filled in.
    fn own_echo(&mut self, shared: &Shared, text: &str) -> bool {
        let matches = |s: &String| s == text || text.contains(s.get(..24).unwrap_or(s.as_str()));
        if let Some(i) = self.said.iter().position(matches) {
            self.said.remove(i);
            return true;
        }
        shared.side.get().is_some_and(|side| side.said.lock().unwrap().iter().any(matches))
    }

    /// What the echo of our own line says of the host's chat, told to the player once each: no label at all (no
    /// relay widget on the host's client: destinations unknown), or the spectators' channel (spectator chat is off
    /// in this game: no player sees our lines).
    fn read_echo(&mut self, shared: &Shared, frame: i32, to: ChatTo) {
        let notice = match to {
            ChatTo::Unknown if !self.chat_relay && !self.chat_told.0 => {
                self.chat_told.0 = true;
                "the chat relay widget is not running on the client hosting you, so where a line went is not known: you are shown allied players' lines only, and whether the players see yours is not known"
            }
            ChatTo::Spectators if !self.chat_told.1 => {
                self.chat_told.1 = true;
                "spectator chat is off in this game: the players do not see your lines, the spectators do"
            }
            _ => return,
        };
        eprintln!("[ai {}] f={frame} chat: {notice}", self.world.hello.ai_id);
        shared.side.get().map_or(shared, |side| side.as_ref()).chat_in.lock().unwrap().push((frame, -1, format!("(the game: {notice})")));
    }

    /// Whether a line is one a player on our side would be shown (H-CHAT-PERMISSIONS, the engine's rule in
    /// `CGame::HandleChatMsg`): all-chat from players, team chat from our own allies, spectators' lines when the
    /// players can see them. With no label (no relay widget) only allied players' lines; with the widget running an
    /// unlabelled line is one the host was not shown. Our host's lines for the other seats of ours (banners, the
    /// hands' lines, the player's own lines sent by another seat) are nobody's words.
    fn hears(&self, player: i32, to: ChatTo, text: &str) -> bool {
        if Some(player) == self.host_player() && (text.contains(" is Within Reason ") || text.contains(" is WR") || text.contains("'s hands: ") || text.contains(CHAT_PREFIX.trim_end())) {
            return false;
        }
        people::hearing(to, self.plays_with_us(player), self.chat_relay)
    }

    /// A line this seat is about to say by itself (its banner, its hands' line): its echo is not chat, to this
    /// seat's player or, under a commander, to the side (human-19: the banners of both seats reached the commander
    /// as four lines of chat from the host, and the hands' line woke it at 1:34).
    pub(crate) fn remember_said(&mut self, line: String) {
        if let Some(side) = self.strategist.as_ref().and_then(|s| s.side.get()) {
            let mut said = side.said.lock().unwrap();
            said.push(line.clone());
            let excess = said.len().saturating_sub(16);
            said.drain(..excess);
        }
        self.said.push(line);
        let excess = self.said.len().saturating_sub(16);
        self.said.drain(..excess);
    }

    /// One tick: the lane's report of the last tick taken up first; then the whole brain (`think`) on the frames due at
    /// its own interval, with every event since it last ran, or between thinks the hands' answers that have come. The
    /// answer carries the orders, the groups declared to the lane and the lane's view; the lane itself runs on the
    /// session's fast thread (`session.rs`).
    pub fn decide(&mut self, tick: &Tick, reports: Vec<LaneReport>) -> BrainResult {
        if let Some(sensors) = &tick.snapshot.sensors {
            self.sensors = Some((sensors.clone(), tick.frame));
        }
        // Lockstep: the game stands at its first tick until the opening turn asked for at Hello is over.
        if let Some(shared) = &self.strategist
            && shared.lockstep.load(std::sync::atomic::Ordering::Relaxed)
        {
            shared.hold_for_opening();
            // ... and, under a commander, never gets further ahead of a commander's turn in hand than that turn's
            // wall time is worth: the lockstep game runs many times faster than the wall between the player's
            // turns, and a direction written in 20 s would land a minute and a half of game late (commander-smoke:
            // the direction from the report of 0:25 was first shown at 2:00).
            if let Some(side) = shared.side.get() {
                side.pace_with_the_commander(tick.frame);
            }
        }
        if tick.late > 0 {
            self.late_ticks = (self.late_ticks.0 + 1, self.late_ticks.1.max(tick.late));
        }
        for event in &tick.events {
            if let Event::Chat { text, .. } = event
                && let Some(lag) = self.echo.heard(text, tick.frame)
            {
                eprintln!("[ai {}] f={} our line came back {lag} frames after we said it: orders run that late", self.world.hello.ai_id, tick.frame);
            }
        }
        for event in &tick.events {
            if let Event::Received { metal, energy } = event {
                self.received.push_back((tick.frame, *metal, *energy));
            }
        }
        self.received.retain(|(at, _, _)| tick.frame - at < RECEIVED_WINDOW);
        let mut commands = Vec::new();
        self.timed("lane_reports", |b| {
            for report in reports {
                b.take_lane_report(report, tick, &mut commands);
            }
        });
        let (declared, scene) = if tick.due() % BRAIN_FRAMES != 0 {
            self.carried_events.extend(tick.events.iter().cloned());
            if self.pianist.is_some() {
                self.timed("poll", |b| b.poll_hands(tick, &mut commands));
            }
            // The hands' answers arrive on any tick in a realtime game, and what they order is the soldiers'
            // standing order as much as what a thinking tick orders. Without this the lane rewrote a body's new
            // advance into the slots of the order it had before, and went on marching to the old goal while the
            // group's task named the new one (human-20 4:47: "advance to spot_13", 3,000 west, issued as fight
            // commands at our own base, the body standing there 22 s; human-22 11:30: a body under "advance to
            // spot_46" walking home; 13 of 118 walks and advances played between thinking ticks in four games
            // with people pointed elsewhere, 1 of 97 played on them).
            ((!commands.is_empty()).then(|| self.timed("declare", |b| b.declare_groups())), None)
        } else {
            let thought = if self.carried_events.is_empty() {
                self.timed("think", |b| b.think(tick))
            } else {
                let mut events = std::mem::take(&mut self.carried_events);
                events.extend(tick.events.iter().cloned());
                let whole = Tick { frame: tick.frame, late: tick.late, events, snapshot: tick.snapshot.clone() };
                self.timed("think", |b| b.think(&whole))
            };
            commands.extend(thought);
            (Some(self.timed("declare", |b| b.declare_groups())), self.kit.is_some().then(|| self.timed("scene", |b| b.lane_scene(tick.frame))))
        };
        self.echo.said(&commands, tick.frame);
        let fixed = (self.kit.is_some() && !self.lane_fixed_sent).then(|| {
            self.lane_fixed_sent = true;
            self.lane_fixed()
        });
        BrainResult { frame: tick.frame, commands, declared, fixed, scene, journal: self.take_journal(), decide_ms: 0.0 }
    }

    /// One phase of a decide, its wall time journaled (the record's `phase_ms`, the slowest of each phase per sample):
    /// what a brain that stalled for seconds was doing (K-bot-a-stalled-brain-drops-its-seat-from-the-side; in a
    /// realtime game the seat's hands and its place in the side wait on it).
    pub(super) fn timed<T>(&mut self, phase: &'static str, run: impl FnOnce(&mut Self) -> T) -> T {
        let started = std::time::Instant::now();
        let out = run(self);
        self.journal.phase(phase, started.elapsed().as_secs_f32() * 1000.0);
        out
    }

    fn think(&mut self, tick: &Tick) -> Vec<Command> {
        if self.kit.is_none() {
            self.timed("adopt", |b| b.adopt_faction(&tick.snapshot.own_units));
        }
        let Some(kit) = self.kit else { return Vec::new() };
        self.timed("track", |b| {
            b.note_allies(tick);
            b.track_enemy_buildings(tick);
            b.refresh_coverage(tick);
            b.survey_spots(tick);
            b.track_enemy_bases(tick);
            b.track_wrecks(tick);
            b.track_losses(tick, &kit);
            b.track_yards(tick);
        });
        self.timed("ways", |b| b.track_ways(tick));
        let mut commands = Vec::new();
        self.timed("nanos", |b| b.tend_nanos(tick, &mut commands));
        self.timed("chat", |b| b.relay_chat(tick, &mut commands));
        // The game names AIs at random (ai_namer.lua), so the bot says who it is, once the engine takes orders.
        if tick.frame >= 2 * FRAMES_PER_SECOND
            && let Some(banner) = self.banner.take()
        {
            let side = self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).map_or("?", |t| t.side.as_str());
            let tail = format!(" is {banner} | seat ai{} team {} {side}", self.world.hello.ai_id, self.world.hello.team);
            // The echo comes back with the name filled in, so the tail is what `relay_chat` drops (comet-1: the
            // banner woke the player at 0:05 as "someone in the game said something").
            self.remember_said(tail.clone());
            commands.push(Command::Say { text: format!("{{name}}{tail}"), to: SayTo::Everyone });
        }
        self.timed("fire", |b| {
            b.track_shelling(tick);
            b.track_guns(tick);
            b.track_fire(tick);
            b.track_hits(tick);
            b.track_takings(tick);
        });
        self.timed("block", |b| b.tend_block(&tick.snapshot.own_units, &kit, tick.frame));
        self.timed("pianist", |b| b.run_pianist(tick, &kit, &mut commands));
        if tick.frame % planner::PLAN_CONTEXT_FRAMES < TICK_FRAMES_GUESS {
            self.timed("plan_context", |b| b.publish_plan_context(tick, &kit));
        }
        // The spots this seat's builders are on their way to, for the team board (13.1: the board shared no claims,
        // `spot_claims` was never filled, and 57 of 114 abandoned extractor orders were spots another seat took).
        self.spot_claims = self.pianist.as_ref().map(|p| p.tasks.values().chain(p.queued_tasks().map(|(_, t)| t)).filter_map(|t| if let pianist::Task::Build { spot: Some(i), .. } = t { Some((*i, tick.frame)) } else { None }).collect()).unwrap_or_default();
        self.timed("exchange", |b| b.exchange_with_team(tick));
        self.journal_intent();
        self.timed("report", |b| b.report(tick, &kit));
        self.timed("briefing", |b| b.publish_briefing(tick, &kit));
        self.timed("wake", |b| b.wake_commander_if_due(tick, &kit));
        commands
    }

    /// Picks the faction from whichever commander we own; does nothing until one exists.
    fn adopt_faction(&mut self, own: &[OwnUnit]) {
        for roster in &ROSTERS {
            let Some(commander) = self.world.def_named(roster.commander) else { continue };
            let Some(unit) = own.iter().find(|u| u.def == commander) else { continue };
            match roster.resolve(&self.world) {
                Ok(kit) => self.kit = Some(kit),
                Err(missing) => {
                    eprintln!("[ai {}] this game has no unit named {missing}", self.ai());
                    return;
                }
            }
            self.home = unit.pos;
            for b in &self.world.hello.start_boxes {
                eprintln!("[ai {}] start box of ally team {}: ({:.0}, {:.0}) to ({:.0}, {:.0}){}", self.world.hello.ai_id, b.ally_team, b.left, b.top, b.right, b.bottom, if b.ally_team == self.world.hello.ally_team { " (ours)" } else { "" });
            }
            self.guess_enemy_bases();
            eprintln!("[ai {}] playing {} from ({:.0}, {:.0})", self.ai(), roster.commander, unit.pos.x, unit.pos.z);
            if let Some(kit) = self.kit {
                self.survey(&kit);
            }
            self.publish_map();
            return;
        }
    }

    /// False when an ablation run has switched this heuristic off.
    fn enabled(&self, rule: &str) -> bool {
        !self.disabled.iter().any(|id| id == rule)
    }

    fn fire(&mut self, rule: &'static str) {
        *self.fired.entry(rule).or_default() += 1;
        self.journal.rule(rule);
    }

    /// A point `distance` elmos from home towards the nearest enemy base: along the walking route when we know the terrain, and
    /// always on ground our soldiers can reach. A negative distance is behind home, away from the enemy.
    fn forward_of_home(&self, distance: f32) -> Vec3 {
        if distance > 0.0
            && let Some(point) = self.on_the_way_to(self.enemy_base(self.home), distance)
        {
            return point;
        }
        let enemy = self.enemy_base(self.home);
        let (dx, dz) = (enemy.x - self.home.x, enemy.z - self.home.z);
        let len = dx.hypot(dz).max(1.0);
        self.snap_to_reachable(Vec3 { x: self.home.x + dx / len * distance, y: 0.0, z: self.home.z + dz / len * distance })
    }

    /// The hits of this tick with a known attacker are kept for `HIT_MEMORY` frames, so the picture can say what a
    /// party in sight is shooting (H-HANDS-PARTY-KILLING: the replay of recorded holds beside a base under attack
    /// showed the hold option's words carrying the cost is what moves the hands, K-jev-hold-words-carry-the-cost).
    fn track_hits(&mut self, tick: &Tick) {
        self.hits.retain(|h| tick.frame - h.frame <= HIT_MEMORY);
        for event in &tick.events {
            let Event::UnitDamaged { unit, attacker: Some(attacker), .. } = *event else { continue };
            if let Some(victim) = tick.snapshot.own_units.iter().find(|u| u.id == unit) {
                self.hits.push(Hit { frame: tick.frame, victim: unit, victim_def: victim.def, attacker });
            }
        }
    }

    /// A finished building of ours whose health fell since the last tick with no hit on it, while an enemy builder
    /// stands within its build reach of it, is being taken apart by that builder (player-31, 6:23-6:50: a Lazarus
    /// took two extractors from full health in ten seconds each, and nothing in the picture said so).
    fn track_takings(&mut self, tick: &Tick) {
        /// A reclaim reaches the building's edge, not its centre.
        const EDGE: f32 = 120.0;
        let own = &tick.snapshot.own_units;
        self.takings.retain(|t| tick.frame - t.frame <= HIT_MEMORY && own.iter().any(|u| u.id == t.victim));
        let hit: HashSet<UnitId> = tick.events.iter().filter_map(|e| if let Event::UnitDamaged { unit, .. } = *e { Some(unit) } else { None }).collect();
        let mut health = HashMap::new();
        for unit in own.iter().filter(|u| !u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0)) {
            health.insert(unit.id, unit.health);
            let Some(before) = self.building_health.get(&unit.id).copied() else { continue };
            if unit.health >= before - 0.1 || hit.contains(&unit.id) {
                continue;
            }
            let taker = tick
                .snapshot
                .enemies
                .iter()
                .filter(|e| !e.being_built && e.def.and_then(|d| self.world.def(d)).is_some_and(|d| d.build_speed > 0.0 && e.pos.dist2d(unit.pos) <= d.build_distance + EDGE))
                .min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
            let Some(taker) = taker else { continue };
            match self.takings.iter_mut().find(|t| t.victim == unit.id) {
                Some(t) => (t.taker, t.frame, t.health) = (taker.id, tick.frame, unit.health),
                None => self.takings.push(Taking { victim: unit.id, victim_def: unit.def, taker: taker.id, since: self.building_health_frame, from: before, frame: tick.frame, health: unit.health, max_health: unit.max_health }),
            }
        }
        self.building_health = health;
        self.building_health_frame = tick.frame;
    }

    fn report(&mut self, tick: &Tick, kit: &Kit) {
        for event in &tick.events {
            match *event {
                Event::BuildSiteNotFound { unit, def } => {
                    eprintln!("[ai {}] f={} no site for {} (builder {})", self.ai(), tick.frame, self.name(def), unit.0);
                    // Said to the player (the impedance tally, 2026-10-16: eleven such drops in the human games and
                    // no report line; human-30 Eclipsed's tier-2 plant, "the priority", first stood at 15:21 after
                    // four silent failures from 13:23).
                    let line = format!("{} {}: no site could be found for a {} where it was asked, so the order is dropped: name another place for it", pianist::picture::clock(tick.frame), self.actor_name(unit), self.name(def));
                    if let Some(pianist) = self.pianist.as_mut() {
                        pianist.done.push(line);
                    }
                }
                Event::CommandRejected { unit, code } => {
                    eprintln!("[ai {}] f={} engine rejected command for unit {} (code {code})", self.ai(), tick.frame, unit.0);
                }
                _ => {}
            }
        }
        if tick.due() % (60 * FRAMES_PER_SECOND) == 0 {
            let s = &tick.snapshot;
            let count = |def: UnitDefId| s.own_units.iter().filter(|u| u.def == def).count();
            eprintln!(
                "[ai {}] f={} ({:.0} min) metal {:.0} (+{:.1}/-{:.1}) energy {:.0}/{:.0} (+{:.0}/-{:.0}) | mex {} labs {} cons {} army {} | enemies visible {}",
                self.ai(), tick.frame, tick.frame as f32 / 1800.0, s.metal.current, s.metal.income, s.metal.usage,
                s.energy.current, s.energy.storage, s.energy.income, s.energy.usage,
                count(kit.extractor) + count(kit.advanced_extractor), s.own_units.iter().filter(|u| self.world.is_factory_def(u.def)).count(), count(kit.constructor) + count(kit.vehicle_constructor) + count(kit.advanced_constructor),
                s.own_units.iter().filter(|u| self.is_army(u, kit)).count(), s.enemies.len()
            );
            let (fields, bytes) = self.field_cache_size();
            eprintln!("[ai {}] f={} place fields cached: {fields} ({:.0} MB)", self.ai(), tick.frame, bytes as f32 / 1_048_576.0);
            if !self.stuck_cells.is_empty() {
                let mut cells: Vec<_> = std::mem::take(&mut self.stuck_cells).into_iter().collect();
                cells.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
                let worst: Vec<String> = cells.iter().take(4).map(|((x, z), n)| format!("({}, {}) x{n}", x * 200 + 100, z * 200 + 100)).collect();
                eprintln!("[ai {}] f={} soldiers' moves failed around: {}", self.ai(), tick.frame, worst.join(", "));
            }
            if !self.fight_ledger.is_empty() {
                let ledger: Vec<String> = std::mem::take(&mut self.fight_ledger).into_iter().map(|(what, n)| format!("{what} x{n}")).collect();
                eprintln!("[ai {}] f={} fights: {}", self.ai(), tick.frame, ledger.join(", "));
            }
            eprintln!("[ai {}] f={} dropped build orders so far: {}, move failures: {}", self.ai(), tick.frame, self.dropped_orders, self.move_failures);
            {
                let refused = self.layout_refused.borrow();
                let total: usize = refused.values().sum();
                if total > 0 && total != self.layout_refused_said {
                    self.layout_refused_said = total;
                    let mut kinds: Vec<(&UnitDefId, &usize)> = refused.iter().collect();
                    kinds.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
                    let kinds: Vec<String> = kinds.iter().map(|(d, n)| format!("{} x{n}", self.name(**d))).collect();
                    eprintln!("[ai {}] f={} site searches refused so far: {total} ({})", self.ai(), tick.frame, kinds.join(", "));
                }
            }
            if self.late_ticks.0 > 0 {
                let (count, most) = std::mem::take(&mut self.late_ticks);
                eprintln!("[ai {}] f={} ticks late this minute: {count}, the latest by {most} frames", self.ai(), tick.frame);
            }
            let rules: Vec<String> = self.fired.iter().map(|(rule, n)| format!("{rule}={n}")).collect();
            eprintln!("[ai {}] f={} rules: {}", self.ai(), tick.frame, rules.join(" "));
            self.fired.clear();
        }
    }

    /// Armed, mobile, and not a builder.
    fn is_army(&self, unit: &OwnUnit, kit: &Kit) -> bool {
        unit.def != kit.commander
            && self.world.def(unit.def).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0 && d.build_speed == 0.0)
    }

    fn ai(&self) -> i32 {
        self.world.hello.ai_id
    }

    /// What allies put into our stores a second, metal and energy, averaged over RECEIVED_WINDOW: an income the
    /// engine's figure leaves out (the eco line says it beside the income and counts it in the flow words).
    pub(super) fn received_rate(&self) -> (f32, f32) {
        let seconds = (RECEIVED_WINDOW / FRAMES_PER_SECOND) as f32;
        let (metal, energy) = self.received.iter().fold((0.0, 0.0), |(m, e), (_, rm, re)| (m + rm, e + re));
        (metal / seconds, energy / seconds)
    }

    fn name(&self, def: UnitDefId) -> &str {
        self.world.def(def).map_or("?", |d| d.name.as_str())
    }
}

/// The seat's name tag: `_t<team>` in a game where we play more than one seat, else nothing (one-seat games keep
/// their plain names).
fn seat_tag(hello: &bot_protocol::Hello) -> String {
    let ours = hello.teams.iter().filter(|t| t.ally_team == hello.ally_team && matches!(&t.controller, bot_protocol::Controller::Ai { short_name, .. } if short_name == "WReason")).count();
    if ours > 1 { format!("_t{}", hello.team) } else { String::new() }
}

impl Brain {
    /// The suffix on this seat's per-seat names (`seat_tag`).
    pub(crate) fn seat_tag(&self) -> &str {
        &self.seat_tag
    }

    /// How the player names this seat's commander: `commander`, or `commander_t2` beside other seats of ours.
    pub(crate) fn commander_handle(&self) -> String {
        format!("commander{}", self.seat_tag)
    }

    /// The seats this seat's player plays: all of ours with one player for the team, this one alone with a player
    /// a seat.
    pub(crate) fn seats_of_the_player(&self) -> usize {
        if crate::strategist::player_per_seat() { 1 } else { self.seats_of_ours() }
    }

    /// How many seats of ours the start script lists on our side.
    pub(crate) fn seats_of_ours(&self) -> usize {
        let h = &self.world.hello;
        h.teams.iter().filter(|t| t.ally_team == h.ally_team && matches!(&t.controller, bot_protocol::Controller::Ai { short_name, .. } if short_name == "WReason")).count()
    }
}

/// Whose a handle from the player is, for the orders that are consumed (lists, removals).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandleOwner {
    Ours,
    AnotherSeat,
    Nobody,
}

impl Brain {
    /// Ours when the handle names a unit of this seat; another seat's when it carries a different seat's tag or names
    /// an allied unit by id; nobody's otherwise (dead, or never a unit).
    pub(crate) fn handle_owner(&self, handle: &str, own: &[bot_protocol::OwnUnit]) -> HandleOwner {
        if self.unit_named(handle, own).is_some() {
            return HandleOwner::Ours;
        }
        if let Some(rest) = handle.strip_prefix("commander") {
            let theirs = rest.strip_prefix("_t").and_then(|n| n.parse::<i32>().ok());
            return match theirs {
                Some(team) if team != self.world.hello.team && self.world.hello.teams.iter().any(|t| t.team == team && t.ally_team == self.world.hello.ally_team) => HandleOwner::AnotherSeat,
                _ => HandleOwner::Nobody,
            };
        }
        let id = handle.rsplit_once('_').and_then(|(_, n)| n.parse::<i32>().ok());
        match id {
            Some(id) if self.allies.iter().any(|a| a.id.0 == id) => HandleOwner::AnotherSeat,
            _ => HandleOwner::Nobody,
        }
    }
}
