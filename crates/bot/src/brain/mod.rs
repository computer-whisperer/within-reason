//! The brain: one instance per AI; `decide` runs once per tick. The world model (`routes`, `bases`,
//! `reclaim`, `shelling`, `yards`, the spot survey) is kept here each tick; the decisions are the pianist's
//! (`pianist/`) and the control lane's (`micro.rs`). The heuristic deciders were deleted 2026-09-25
//! (`docs/design/2026-09-25-one-decider.md`).

mod allies;
mod bases;
mod briefing;
mod combat;
mod economy;
pub mod journal;
mod micro;
pub mod pianist;
mod planner;
mod scout;
mod reclaim;
mod shelling;
mod wake;
pub(crate) mod roster;
mod routes;
mod nanos;
mod yards;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// A tick's spacing in frames, for periodic work keyed on `frame % N` (ticks come every few frames).
const TICK_FRAMES_GUESS: i32 = 8;

use bot_protocol::{Command, Event, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

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
/// The static map description is published during the first few seconds, once home is known.
const TICK_FRAMES_HINT: i32 = 10 * FRAMES_PER_SECOND;

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
    /// Hits from out of sight in the last twenty seconds (`shelling.rs`), and when the player was last woken for them.
    shelling: Vec<shelling::Shell>,
    shelling_warned: i32,
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
    lanes: Vec<bot_protocol::Lane>,
    stuck: HashMap<UnitId, yards::Stuck>,
    /// Mobile units standing still in a standing factory's exit lane: since when, where they stood, which factory.
    lane_standers: HashMap<UnitId, (i32, Vec3, UnitId)>,
    yard_warned: HashSet<UnitId>,
    /// Construction turrets told to guard a factory (H-ECO-NANO-GUARD).
    nano_guards: nanos::NanoGuards,
    /// Chat lines of ours not yet seen back from the engine, which echoes every line as a chat event from our own
    /// host player (human-1: the player was woken by its own "gl hf").
    said: Vec<String>,
    /// The frame of the last chat line that was not our own echo (the wake reads it; comet-2: the banner's echo woke the player).
    heard_chat_at: i32,
    /// Buildings `forget_razed_buildings` dropped this tick, for the team board.
    razed: Vec<UnitId>,
    /// Factories of theirs seen destroyed or found razed: definition, position, frame. Evidence for the player of
    /// where the opponent has been (docs/design/2026-09-22-enemy-evidence.md).
    enemy_factories_gone: Vec<(UnitDefId, Vec3, i32)>,
    /// Every enemy soldier seen and not known dead, with when it was last seen: what we know of their army, a floor.
    /// Enemy soldiers seen: definition, where, when (the shelling attribution and the army counts read it).
    enemy_soldiers: HashMap<UnitId, (UnitDefId, Vec3, i32)>,
    /// Where and when the enemy commander was last seen: killing it wins the game.
    enemy_commander_seen: Option<(Vec3, i32)>,
    recent_events: VecDeque<String>,
    /// Our units as last seen, to name what a destroyed-unit event refers to.
    known_units: HashMap<UnitId, (UnitDefId, Vec3)>,
    /// Our units still being built at the last look, with how far along they were (health share): a `UnitDestroyed`
    /// for one of these with no attacker is a nanoframe its builder abandoned, not a loss to the enemy.
    unfinished: HashMap<UnitId, f32>,
    /// The abandoned frames among this think's `UnitDestroyed` events (`track_losses` fills it; the pianist's
    /// housekeeping runs after and asks).
    abandoned_now: HashMap<UnitId, f32>,
    /// Every enemy unit's type once seen, so a killer that has left sight still has a name.
    enemy_defs: HashMap<UnitId, UnitDefId>,
    /// This minute's fight ledger for the log: "lost X to Y near home" and "killed Y", with counts.
    fight_ledger: std::collections::BTreeMap<String, u32>,
    /// (frame, metal of ours destroyed, metal of theirs we saw destroyed), one entry per death: what the fighting costs
    /// each side, which neither army count shows.
    trade_log: Vec<(i32, f32, f32)>,
    /// Every enemy death seen: unit id, frame, metal (the merge across seats counts each once).
    enemy_deaths: Vec<(u32, i32, u32)>,
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
    pub(crate) unit_losses: VecDeque<(i32, UnitId, UnitDefId)>,
    /// Metal spots (by index) where an extractor offset toward the builder was refused: the exact centre from then on.
    centre_only: HashSet<usize>,
    dropped_orders: u32,
    move_failures: u32,
    /// Heuristic IDs switched off for an ablation run (`WITHIN_REASON_DISABLE=H-A,H-B`).
    disabled: Vec<String>,
    last_trigger_frame: HashMap<&'static str, i32>,
    /// Decisions since `main` last collected them for the match record.
    journal: journal::Journal,
    /// The control lane's state (`micro.rs`): standing orders, commitments, claims, the threat grid.
    lane: micro::Lane,
    /// Events from the ticks since `think` last ran, in order, for its next run.
    carried_events: Vec<Event>,
    /// Ticks that came late (`Tick::late`) and the most frames one waited, since the last per-minute line.
    late_ticks: (u32, i32),
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
        Brain {
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
            enemy_bases: Vec::new(),
            boxes_honoured: true,
            spot_claims: HashMap::new(),
            routes: None,
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
            shelling: Vec::new(),
            shelling_warned: i32::MIN / 2,
            hits: Vec::new(),
            takings: Vec::new(),
            building_health: HashMap::new(),
            building_health_frame: 0,
            lanes: Vec::new(),
            stuck: HashMap::new(),
            lane_standers: HashMap::new(),
            yard_warned: HashSet::new(),
            nano_guards: HashMap::new(),
            said: Vec::new(),
            heard_chat_at: -1,
            razed: Vec::new(),
            enemy_factories_gone: Vec::new(),
            enemy_soldiers: HashMap::new(),
            enemy_commander_seen: None,
            recent_events: VecDeque::new(),
            known_units: HashMap::new(),
            unfinished: HashMap::new(),
            abandoned_now: HashMap::new(),
            enemy_defs: HashMap::new(),
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
            move_failures: 0,
            disabled: std::env::var("WITHIN_REASON_DISABLE").map_or_else(|_| Vec::new(), |ids| ids.split(',').map(str::to_string).collect()),
            last_trigger_frame: HashMap::new(),
            journal: Default::default(),
            lane: Default::default(),
            carried_events: Vec::new(),
            late_ticks: (0, 0),
        }
    }

    /// Chat from the people in the game goes to the player's next report; what the player said goes into the game
    /// (the user, 2026-09-22: in human games, letting Opus talk "could actually help gain direct feedback from
    /// experienced players").
    fn relay_chat(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(shared) = &self.strategist else { return };
        for event in &tick.events {
            if let Event::Chat { player, text } = event {
                // The game cuts a chat line at about 120 characters, so a long line of ours is known by its first
                // words after the name the game filled in.
                if let Some(i) = self.said.iter().position(|s| s == text || text.contains(s.get(..24).unwrap_or(s.as_str()))) {
                    self.said.remove(i);
                    continue;
                }
                eprintln!("[ai {}] f={} chat from player {player}: {text}", self.world.hello.ai_id, tick.frame);
                shared.chat_in.lock().unwrap().push((tick.frame, *player, text.clone()));
                self.heard_chat_at = tick.frame;
            }
        }
        for text in std::mem::take(&mut *shared.chat_out.lock().unwrap()) {
            for line in chat_lines(&text) {
                self.said.push(line.clone());
                self.said.truncate(16);
                commands.push(Command::Say { text: line });
            }
        }
    }

    /// One tick: the control lane every time, the whole brain (`think`) on the frames due at its own interval, with
    /// every event since it last ran.
    pub fn decide(&mut self, tick: &Tick) -> Vec<Command> {
        // Lockstep: the game stands at its first tick until the opening turn asked for at Hello is over.
        if let Some(shared) = &self.strategist
            && shared.lockstep.load(std::sync::atomic::Ordering::Relaxed)
        {
            shared.hold_for_opening();
        }
        if tick.late > 0 {
            self.late_ticks = (self.late_ticks.0 + 1, self.late_ticks.1.max(tick.late));
        }
        if tick.due() % BRAIN_FRAMES != 0 {
            self.carried_events.extend(tick.events.iter().cloned());
            let mut commands = Vec::new();
            if self.pianist.is_some() {
                self.poll_hands(tick, &mut commands);
            }
            self.micro(tick, &mut commands);
            return commands;
        }
        let mut commands = if self.carried_events.is_empty() {
            self.think(tick)
        } else {
            let mut events = std::mem::take(&mut self.carried_events);
            events.extend(tick.events.iter().cloned());
            let whole = Tick { frame: tick.frame, late: tick.late, events, snapshot: tick.snapshot.clone() };
            self.think(&whole)
        };
        self.note_standing_orders(&commands, tick.frame);
        self.note_commitments();
        self.micro(tick, &mut commands);
        commands
    }

    fn think(&mut self, tick: &Tick) -> Vec<Command> {
        if self.kit.is_none() {
            self.adopt_faction(&tick.snapshot.own_units);
        }
        let Some(kit) = self.kit else { return Vec::new() };
        self.receive_spot_fields();
        self.note_allies(tick);
        self.track_enemy_buildings(tick);
        self.survey_spots(tick);
        self.track_enemy_bases(tick);
        self.track_wrecks(tick);
        self.track_losses(tick, &kit);
        self.track_yards(tick);
        let mut commands = Vec::new();
        self.tend_nanos(tick, &mut commands);
        self.relay_chat(tick, &mut commands);
        // The game names AIs at random (ai_namer.lua), so the bot says who it is, once the engine takes orders.
        if tick.frame >= 2 * FRAMES_PER_SECOND
            && let Some(banner) = self.banner.take()
        {
            let side = self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).map_or("?", |t| t.side.as_str());
            let tail = format!(" is {banner} | seat ai{} team {} {side}", self.world.hello.ai_id, self.world.hello.team);
            // The echo comes back with the name filled in, so the tail is what `relay_chat` drops (comet-1: the
            // banner woke the player at 0:05 as "someone in the game said something").
            self.said.push(tail.clone());
            commands.push(Command::Say { text: format!("{{name}}{tail}") });
        }
        self.track_shelling(tick);
        self.track_hits(tick);
        self.track_takings(tick);
        self.run_pianist(tick, &kit, &mut commands);
        if tick.frame % planner::PLAN_CONTEXT_FRAMES < TICK_FRAMES_GUESS {
            self.publish_plan_context(tick, &kit);
        }
        // The spots this seat's builders are on their way to, for the team board (13.1: the board shared no claims,
        // `spot_claims` was never filled, and 57 of 114 abandoned extractor orders were spots another seat took).
        self.spot_claims = self.pianist.as_ref().map(|p| p.tasks.values().chain(p.queued.values()).filter_map(|t| if let pianist::Task::Build { spot: Some(i), .. } = t { Some((*i, tick.frame)) } else { None }).collect()).unwrap_or_default();
        self.exchange_with_team(tick);
        self.journal_intent();
        self.report(tick, &kit);
        self.publish_briefing(tick, &kit);
        self.wake_commander_if_due(tick, &kit);
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
                count(kit.extractor) + count(kit.advanced_extractor), count(kit.lab) + count(kit.plant) + count(kit.advanced_lab), count(kit.constructor) + count(kit.vehicle_constructor) + count(kit.advanced_constructor),
                s.own_units.iter().filter(|u| self.is_army(u, kit)).count(), s.enemies.len()
            );
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
        if self.unit_by_handle(handle, own).is_some() {
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
