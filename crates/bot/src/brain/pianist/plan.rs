//! One pass (`docs/design/2026-09-26-one-pass.md`, H-HANDS-ONE-PASS): code enumerates every actor's executable
//! states each second, a pre-pass of nouls asks which actors should change course and which kinds of action matter
//! now, code composes worlds from what was flagged (each with its consequence in words), and one Choice picks a plan
//! for every actor. The plan runs in code until the picture's signature changes; nothing else decides. The threat
//! family (`threats.rs`) is one family of slots here.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{OwnUnit, Tick, UnitDefId, UnitId, Vec3};
use jev::{Answer, Question};
use serde_json::{Value, json};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Party, Picture, distance_words};
use super::{GroupTask, Pianist, Task};
use crate::strategist::shared::Allowance;

/// Worlds in one question at most, by default (`WITHIN_REASON_WORLDS` sets it): the API's cap on a Choice's options.
/// Battery J (`../jev_experiments/results/j.md` §2) holds 0.95-1.00 to 255 worlds when each carries its computed
/// consequence; the offline study's diffusion past 8 was a bare Choice over near-equivalent worlds. onepass-smoke-1
/// at 8: one actor's seven states filled every deviation and the others were starved (the round-robin fill below).
pub(super) const CAP: usize = 255;
/// Characters of deviation lines in one pick at most: the stage-one Choice cannot be split into batches, and the
/// service refuses a request past its context (player-27, 10:20-13:00: 145-255 worlds with 120-240k characters of
/// lines refused 172 times, HTTP 400 max_tokens_exceeded, no pick for three minutes with 2,200 metal banked; the
/// largest call that passed carried 66k characters of lines beside a 12-26k state, 33k tokens at 2.1 characters a
/// token for this dense text). The worlds are cut to the longest prefix that fits: the singles first, then the joint.
pub(super) const LINE_CHARS: usize = 40_000;
/// A noul at or above this flags its actor, state or threat for the second call.
pub(super) const FLAG: f64 = 0.5;
/// A hunt whose `forbidden` noul is at or above this says on its line in the pick that the player's instructions
/// forbid it (the user's law, 2026-09-30: "For each hunt on offer, Jev is asked whether the player's instructions
/// forbid it for that group, and a hunt it rates forbidden says so on its line in the pick"; hunts only, the bar
/// the user's). player-29-hard, `docs/studies/2026-09-29-pick-framing.md` §8: the packets said the ball "never
/// hunts and never sends detachments" from 12:39 on and the pick played 61 hunts by a few of its soldiers; the
/// hunt's own noul never names the instructions (0.37 for a forbidden hunt, 0.32 for an ordered one), a sentence
/// about forbidden changes in the pick's question moved nothing, and this noul rated the ball's hunts a median
/// 0.84 (p10 0.72) against the early ordered hunts' 0.53 (p90 0.68). With the sentence on the line the forbidden
/// hunts' stage-one mass fell from 0.40 to 0.08 and they beat world 1 in 10 of 181 picks for 172; the bar sits
/// between the two sets.
pub(super) const FORBIDDEN: f64 = 0.7;
/// A slot's deviations in the second call at most: its best states by the gate.
const DEPTH: usize = 2;
/// A standing plan is asked again this long after its last ask even when nothing changed.
pub(super) const RE_ASK: i32 = 20 * FRAMES_PER_SECOND;
/// An enemy party this close to a group or a builder is news.
pub(super) const ALARM: f32 = 600.0;
/// His buildings and parties this close to a leg's end are said in the leg's words (routes-in-prose §4.4).
const AHEAD: f32 = 800.0;
/// A builder on a started build has no other state unless an enemy party is this close (H-HANDS-STARTED).
const STARTED_ALARM: f32 = 800.0;
/// A builder whose started build is this far along has its next states, ordered behind it (H-HANDS-QUEUE).
const QUEUE_AT: f32 = 0.6;
/// Wrecks and things to repair within this of a builder are states.
const RECLAIM_WITHIN: f32 = 1800.0;
const REPAIR_WITHIN: f32 = 1200.0;
/// A builder farther than this from home has the way home.
const AWAY: f32 = 400.0;
/// Free spots the instructions name offered to a builder at most, nearest first; and the nearest free spots always
/// offered beside them.
const NAMED_SPOTS: usize = 8;
const NEAREST_SPOTS: usize = 2;
/// With `WITHIN_REASON_TOLD=1`, a builder's state is asked a second noul (is this the step the instructions name
/// next) and opens when it reaches this. Off by default: the told noul sits at 0.4-0.7 for the instructions' next
/// spot when the builder is idle and near, 0.2-0.35 when it is busy or far, 0.03-0.07 for a forbidden spot, under
/// every wording tried (onepass-norules-hard-5 replayed four ways, three times each), and reads the commander's
/// "helping the plant" as the constructors' job too (0.41 mean); opened at 0.25 it sent every constructor to the
/// plant and none to the strip (onepass-norules-hard-6: two extractors all game, 1,000 metal unspent).
const TOLD_BAR: f64 = 0.25;
/// A builder helps another builder's build within this.
const HELP_REACH: f32 = 900.0;
/// A build this dear, or any factory, is worth a second builder's hands: the pair world.
const PAIR_COST: f32 = 300.0;
/// A turret this close to an extractor covers it; an extractor beyond `OUTER` from home is an outer one; the turret
/// states reach this far from the builder.
const TURRET_COVER: f32 = 350.0;
const OUTER: f32 = 500.0;
const TURRET_REACH: f32 = 1200.0;
/// A place this close to a structure stands at it.
const AT_STRUCTURE: f32 = 400.0;
/// A group holding farther than this from its station walks there.
const STATION_SLACK: f32 = 300.0;
/// A named place this far from a group is not a walk state.
const WALK_REACH: f32 = 3000.0;
/// His buildings known this far from a group's front are raid states, the nearest few.
const RAID_REACH: f32 = 4000.0;
const RAID_STATES: usize = 3;
/// A soldier with this reach or more is artillery: it shells from a standoff.
const ARTILLERY_REACH: f32 = 600.0;
/// An idle actor with no state at the flag still puts its best state to the pick when it is rated this high
/// (onepass-medium-2: the commander idled six minutes on an extractor rated 0.48).
const IDLE_BAR: f64 = 0.3;

/// What an actor does in a state.
#[derive(Clone, Debug)]
pub(crate) enum Response {
    /// What it does now.
    Keep,
    // Builders.
    Extractor(usize),
    /// A building placed as the base layout has it (`place_planned`).
    Building(UnitDefId),
    /// A building at the named place (a defence beside an extractor, a radar at a mark, a tier-2 extractor over a
    /// spot).
    BuildingAt(UnitDefId, String),
    /// Guard a factory or another builder: its build power goes to whatever that makes.
    Assist(UnitId),
    Reclaim(Vec3),
    /// Take one unit of ours apart (a `reclaim <handle>` list step from the `remove` tool).
    ReclaimUnit(UnitId),
    Repair(UnitId),
    WalkTo(String),
    RetreatHome,
    /// Attack the named party (a builder that outweighs it alone).
    Attack(String),
    /// The commander's D-gun at the named party's nearest unit (8.1).
    DGun(String),
    // Labs.
    Next(UnitDefId),
    // Groups, against a threat.
    /// Nobody moves for it.
    Leave,
    /// The whole group attacks the party.
    Whole,
    /// These members hunt one unit of the party by id (H-MICRO-HUNT).
    Hunt(Vec<UnitId>),
    /// The group falls back to the named place from the party.
    Back(String),
    // Groups, their course.
    Walk { place: String, fight: bool },
    Retreat,
    FallBack,
    Scout,
    Split(usize, String),
    Join(String),
    /// Walk to the place and hold there until the tail is up (H-HANDS-GROUP-STATES).
    Gather(String),
    /// The long-reach members shell the named party from a standoff, the rest between.
    Shell(String),
}

/// One executable state of one actor.
#[derive(Clone, Debug)]
pub(crate) struct State {
    /// `<actor>.<what>`; a threat state `party_N.<what>_group_X`.
    pub id: String,
    /// The actor that executes it; empty for `Leave`.
    pub actor: String,
    pub response: Response,
    /// The state in a world's line.
    pub words: String,
    /// Metal sent at a party (threat states).
    pub metal: f32,
    /// The kind of action, for the pre-pass's dimension nouls; "threat" for the threat states.
    pub dim: &'static str,
    /// The state already in force.
    pub current: bool,
    /// Offered only beside another builder's build in a pair world, not on its own.
    pub pair_only: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Threat(Party, String),
    Builder(UnitId),
    Lab(UnitId),
    Group(String),
}

/// One decision of the pass: a threat with the states against it, or an actor with its states; index 0 is what
/// stands (`Leave` for a threat, `Keep` for an actor).
#[derive(Clone, Debug)]
pub(crate) struct Slot {
    pub name: String,
    pub kind: Kind,
    pub states: Vec<State>,
    /// A builder's states are what it does after the build in progress: ordered behind it.
    pub queue_ahead: bool,
    /// The actor does nothing now.
    pub idle: bool,
    /// A group holding at a place of its route it has reached while the packet names further places for it: the
    /// cost of changing nothing, said in world 1's line (routes-in-prose §4.2; K-jev-follows-a-prose-route-from-the-
    /// picture: with the sentence Jev picked the next leg 24 of 24 times, without it 2 of 24).
    pub stop_cost: Option<String>,
    /// A group walking a leg of a route its paragraph names (routes-in-prose §4.4): a party appearing or leaving
    /// does not open it (`settle`).
    pub on_route: bool,
    /// Closed this second (`settle`): no question goes out for it and world 1 keeps its course.
    pub quiet: bool,
}

impl Slot {
    /// World 1's index: the state in force, else what stands.
    pub(super) fn base(&self) -> usize {
        self.states.iter().position(|s| s.current).unwrap_or(0)
    }

    pub(super) fn open(&self) -> bool {
        if self.quiet {
            return false;
        }
        let base = self.base();
        self.states.iter().enumerate().any(|(i, s)| i != base && i != 0 && !s.pair_only)
    }
}

/// One world: for each slot, the index of its state.
pub(crate) type World = Vec<usize>;

fn odds_by_metal(ours: f32, theirs: f32) -> &'static str {
    let ratio = ours / theirs.max(1.0);
    if ratio >= 2.5 {
        "we outweigh it heavily"
    } else if ratio >= 1.3 {
        "we outweigh it"
    } else if ratio >= 0.8 {
        "an even fight"
    } else {
        "it outweighs us"
    }
}

/// ", under 3 turrets: 1 armhlt, 2 armllt" for a party the enemy's armed buildings cover; empty otherwise.
pub(super) fn under(p: &Party) -> String {
    if p.turrets.is_empty() { String::new() } else { format!(", under {}", p.turrets) }
}

/// A builder's standing as the pass and the lists read it.
pub(super) struct BuilderStatus {
    /// The started build: its words and its share done.
    pub started: Option<(String, f32)>,
    pub threatened: bool,
    pub queue_ahead: bool,
}

impl Brain {
    fn dim_of(&self, response: &Response) -> &'static str {
        match response {
            Response::Keep | Response::Leave | Response::Whole | Response::Hunt(_) | Response::Back(_) => "threat",
            Response::Extractor(_) => "extractor",
            Response::Building(def) | Response::BuildingAt(def, _) => match self.world.def(*def) {
                Some(d) if d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.wind_cap > 0.0 || d.tidal_make > 0.0 => "energy",
                Some(d) if !d.build_options.is_empty() => "factory",
                Some(d) if d.build_speed > 0.0 => "nano",
                Some(d) if d.weapon_count > 0 => "defence",
                Some(d) if d.radar_range > 0.0 || d.sonar_range > 0.0 => "radar",
                Some(d) if d.metal_storage > 0.0 || d.energy_storage > 0.0 => "storage",
                Some(d) if d.extracts_metal > 0.0 => "extractor",
                _ => "other",
            },
            Response::Assist(_) => "assist",
            Response::Reclaim(_) | Response::ReclaimUnit(_) => "reclaim",
            Response::Repair(_) => "repair",
            Response::WalkTo(_) | Response::RetreatHome => "walk",
            Response::Attack(_) | Response::DGun(_) => "fight",
            Response::Next(def) => {
                if self.world.is_constructor_def(*def) { "constructor" } else { "soldier" }
            }
            Response::Walk { .. } | Response::Retreat | Response::FallBack => "move",
            Response::Scout => "scout",
            Response::Split(..) => "split",
            Response::Join(_) => "join",
            Response::Gather(_) => "move",
            Response::Shell(_) => "fight",
        }
    }

    /// The started build, the threat beside the builder, and whether its next states queue behind the build.
    pub(super) fn builder_status(&self, pianist: &Pianist, unit: &OwnUnit, picture: &Picture, under_fire: &[UnitId], own: &[OwnUnit]) -> BuilderStatus {
        let started = match pianist.tasks.get(&unit.id) {
            Some(Task::Build { def, near, started: true, .. }) => {
                let share = own.iter().filter(|u| u.being_built && u.def == *def).map(|u| (u.pos.dist2d(*near), u.health / u.max_health.max(1.0))).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(0.0, |(_, share)| share);
                Some((self.short_words(*def), share))
            }
            _ => None,
        };
        let threatened = under_fire.contains(&unit.id) || picture.parties.iter().any(|p| p.at.dist2d(unit.pos) < STARTED_ALARM);
        let queue_ahead = started.as_ref().is_some_and(|(_, share)| *share >= QUEUE_AT) && !threatened && !pianist.queued.contains_key(&unit.id);
        BuilderStatus { started, threatened, queue_ahead }
    }

    /// The free spots this builder could take, nearest by its own walking first: not held, not another builder's
    /// task or queued spot, not refused lately, free as far as we know, reachable by its class.
    pub(super) fn free_spots(&self, unit: &OwnUnit, pianist: &Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) -> Vec<(usize, f32)> {
        // A spot is taken by another builder's extractor order, unless that order has not started and the builder
        // has a list of its own waiting to take it over (bluegecko-3v1-comet-catcher-8, 6:20: lists for five
        // constructors landed at once, each skipped the spot another's not-yet-started hands' order held, those
        // orders were dropped by the lists, and the constructors went on to their turrets with no extractor built).
        let taken: Vec<usize> = pianist
            .tasks
            .iter()
            .chain(pianist.queued.iter())
            .filter_map(|(id, t)| match t {
                Task::Build { spot: Some(i), started, .. } if *id != unit.id => {
                    let listed = pianist.scripts.get(&self.actor_name(*id)).is_some_and(|q| !q.is_empty());
                    (*started || !listed).then_some(*i)
                }
                _ => None,
            })
            .collect();
        let mut spots: Vec<(usize, f32)> = picture
            .places
            .iter()
            .filter_map(|p| p.spot.map(|i| (i, p.at)))
            .filter(|(i, _)| !taken.contains(i) && !self.team_mates.spot_claims.contains(i) && !pianist.refused_spots.get(i).is_some_and(|until| *until > frame))
            .filter(|(_, at)| !own.iter().any(|u| kit.is_extractor(u.def) && u.pos.dist2d(*at) < self.spot_occupied_radius()) && !self.allied_extractor_on(*at))
            .filter(|(i, at)| !self.enemy_buildings.values().any(|(def, pos, _)| pos.dist2d(*at) < self.spot_occupied_radius() && self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)) && self.spot_open_to_us(*i, *at, frame))
            .filter(|(_, at)| self.reachable_for(self.walker_of(unit.def), *at))
            .map(|(i, _)| (i, self.seconds_to_spot(unit.def, i, unit.pos)))
            .collect();
        spots.sort_by(|a, b| a.1.total_cmp(&b.1));
        spots
    }

    /// A building that stands at a place rather than beside the builder or in the yard: a defence, a radar, a
    /// jammer or sonar, a tier-2 extractor over a spot.
    pub(super) fn placed_at_place(&self, def: UnitDefId) -> bool {
        let Some(d) = self.world.def(def) else { return false };
        let flagged = super::glossary::entry(&d.name).is_some_and(|e| e.has_flag("radar_jammer") || e.has_flag("sonar"));
        d.speed == 0.0 && d.build_speed == 0.0 && d.build_options.is_empty() && (d.weapon_count > 0 || d.radar_range > 0.0 || d.extracts_metal > 0.0 || flagged)
    }

    /// Our count of a kind standing and being built, for the lookup table ("our 3rd extractor").
    fn count_of(&self, def: UnitDefId, own: &[OwnUnit], pianist: &Pianist) -> (usize, usize) {
        let standing = own.iter().filter(|u| !u.being_built && u.def == def).count();
        let coming = own.iter().filter(|u| u.being_built && u.def == def).count() + pianist.tasks.values().filter(|t| matches!(t, Task::Build { def: d, started: false, .. } if *d == def)).count();
        (standing, coming)
    }

    fn ordinal(n: usize) -> String {
        match n % 10 {
            1 if n % 100 != 11 => format!("{n}st"),
            2 if n % 100 != 12 => format!("{n}nd"),
            3 if n % 100 != 13 => format!("{n}rd"),
            _ => format!("{n}th"),
        }
    }

    /// The consequence of a build in code's words: the cost against the store and the income, the walk, what of
    /// the kind stands and is under way, the energy it draws, what the builder leaves.
    fn build_words(&self, pianist: &Pianist, unit: &OwnUnit, def: UnitDefId, at: Option<Vec3>, tick: &Tick, own: &[OwnUnit]) -> String {
        let Some(d) = self.world.def(def) else { return String::new() };
        let map = &self.world.hello.map;
        let m = &tick.snapshot.metal;
        let mut parts: Vec<String> = Vec::new();
        let (standing, coming) = self.count_of(def, own, pianist);
        parts.push(format!("our {}{}", Self::ordinal(standing + coming + 1), if coming > 0 { format!(" ({coming} under way already)") } else { String::new() }));
        if let Some(at) = at {
            let walk = unit.pos.dist2d(at);
            if walk > 150.0 {
                parts.push(format!("{} of walking", distance_words(walk)));
            }
        }
        let store = m.current + (m.income - m.usage).max(0.0) * 30.0;
        parts.push(if d.metal_cost <= m.current {
            format!("{:.0} metal, in the store", d.metal_cost)
        } else if d.metal_cost <= store {
            format!("{:.0} metal against {:.0} stored and {:.1} a second coming in: builds as the metal comes", d.metal_cost, m.current, m.income)
        } else {
            format!("{:.0} metal against {:.0} stored and {:.1} a second coming in: slow, the store empties", d.metal_cost, m.current, m.income)
        });
        if d.wind_cap > 0.0 {
            parts.push(format!("gives {:.0} to {:.0} energy a second here", map.wind_min, map.wind_max));
        } else if d.tidal_make > 0.0 {
            parts.push(format!("a steady {:.0} energy a second from this map's tide, on the water where it stands", map.tidal * d.tidal_make));
        } else if d.energy_make > 0.0 || d.energy_upkeep < 0.0 {
            parts.push(format!("a steady {:.0} energy a second", d.energy_make + (-d.energy_upkeep).max(0.0)));
        }
        if !d.build_options.is_empty() {
            let factories = own.iter().filter(|u| self.world.is_factory_def(u.def) && !u.being_built).count();
            parts.push(match factories {
                0 => "we have no factory yet: nothing makes soldiers or constructors without one".to_string(),
                1 => "we have one factory already".to_string(),
                n => format!("we have {n} factories already"),
            });
        }
        if d.build_speed > 0.0 && d.speed == 0.0 && d.build_options.is_empty() {
            parts.push("adds its build power to the nearest factory".to_string());
        }
        if d.converter.is_some() {
            parts.push("turns energy into metal".to_string());
        }
        if d.metal_storage > 0.0 || d.energy_storage > 0.0 {
            parts.push(format!("adds {:.0} metal and {:.0} energy to what we can store", d.metal_storage, d.energy_storage));
        }
        if d.radar_range > 0.0 {
            parts.push(format!("sees {:.0} around it", d.radar_range));
        }
        if d.sonar_range > 0.0 {
            // Short: the option is offered to every builder every second (game 12: the long sentence stood 2,900
            // times in one seat's log); the rule itself is in the brief.
            parts.push(format!("its sonar sees {:.0} under the water, where eyes and radar see nothing", d.sonar_range));
        }
        if let Some(b) = self.world.def(unit.def)
            && d.build_time > 0.0
            && d.energy_cost > 0.0
            && (!d.build_options.is_empty() || d.metal_cost >= 500.0)
        {
            let seconds = d.build_time / b.build_speed.max(1.0);
            let draw = d.energy_cost / seconds;
            let energy = &tick.snapshot.energy;
            let short = d.energy_cost - (energy.current + (energy.income - energy.usage) * seconds);
            parts.push(if short <= 0.0 {
                format!("about {seconds:.0} s and {draw:.0} energy a second, which the store covers")
            } else {
                format!("about {seconds:.0} s and {draw:.0} energy a second; the energy runs out {:.0} s before it is done", (short / draw).min(seconds))
            });
        }
        parts.join("; ")
    }

    /// What the builder gives up: a started frame that decays, the factory it helps.
    fn leaves_words(&self, task: Option<&Task>, started: &Option<(String, f32)>) -> String {
        match (task, started) {
            (_, Some((what, share))) => format!("; leaves the {what} at {:.0}%, which decays", share * 100.0),
            (Some(Task::Assist { lab, .. }), _) => format!("; stops helping {}", self.actor_name(*lab)),
            (Some(Task::Build { def, .. }), _) => format!("; drops the {} it was going to build", self.short_words(*def)),
            _ => String::new(),
        }
    }

    /// Every slot of the second: the threats (`threats.rs`), then each builder, lab and group with its states.
    pub(super) fn slots(&self, tick: &Tick, kit: &Kit, picture: &Picture) -> Vec<Slot> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let frame = tick.frame;
        let mut slots = self.threat_slots(tick, picture);
        let mut under_fire: Vec<UnitId> = tick.events.iter().filter_map(|e| if let bot_protocol::Event::UnitDamaged { unit, .. } = e { Some(*unit) } else { None }).collect();
        under_fire.extend(pianist.hits.keys().copied());
        let draws = self.production_draws(own, pianist);
        let lone_scout = |p: &Party| p.ids.len() == 1 && enemies.iter().any(|e| e.id == p.ids[0] && e.def.is_some_and(|d| super::glossary::entry(self.name(d)).is_some_and(|g| g.class.contains("scout"))));
        // The player's marks: the places that are neither spots nor the map's own (home, the passages, `shelling*`).
        let marks: Vec<&super::Place> = picture.places.iter().filter(|p| p.spot.is_none() && p.name != "home" && !p.name.starts_with("shelling") && !p.name.starts_with("passage_")).collect();

        // Builders.
        let builders: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)).collect();
        // Builders with a started build, for the help states and the pair worlds.
        let building: Vec<(&OwnUnit, UnitDefId, f32)> = builders
            .iter()
            .filter_map(|u| match pianist.tasks.get(&u.id) {
                Some(Task::Build { def, near, started: true, .. }) => {
                    let share = own.iter().filter(|f| f.being_built && f.def == *def).map(|f| (f.pos.dist2d(*near), f.health / f.max_health.max(1.0))).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(0.0, |(_, s)| s);
                    Some((*u, *def, share))
                }
                _ => None,
            })
            .collect();
        for unit in &builders {
            let name = self.actor_name(unit.id);
            let task = pianist.tasks.get(&unit.id);
            let status = self.builder_status(pianist, unit, picture, &under_fire, own);
            // On a list (H-HANDS-SCRIPT): not in the pass while it runs; a threatened builder is. A builder on its
            // list's last step has an empty list and a step in progress: still listed (onepass-player-4, 27:50-28:05:
            // treated as free, the `expand` rule's default walked it off its solar every second and the list took it
            // back the next, twenty times over, until every constructor died walking).
            // ... a step in progress is one whose task still stands: the register keeps the step until a play
            // diverts the builder, so after the last step finished on its own the builder was listed for good, out of
            // the pass and idle (onepass-player-5 and the models-medium bundle on 92d59cc: builders idle 22-30%,
            // the metal store full 15-45% of the game).
            let listed = pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) || (pianist.list_steps.contains_key(&unit.id) && task.is_some());
            // A builder on a list is offered a solar while energy drains (12.3): builders on lists were never
            // offered one through 120 s of STALLING (game 7 8:00-10:00). Offered, not a default: the list is the
            // player's order.
            let draining_now = tick.snapshot.energy.storage > 0.0 && tick.snapshot.energy.current < 0.25 * tick.snapshot.energy.storage && tick.snapshot.energy.usage > tick.snapshot.energy.income;
            if !status.threatened && listed {
                if draining_now
                    && self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&kit.solar))
                    && !own.iter().any(|u| u.being_built && u.def == kit.solar)
                {
                    let keep = State { id: format!("{name}.keep"), actor: name.clone(), response: Response::Keep, words: format!("{name} goes on with its list ({})", self.task_course(task, unit, &picture.places, frame, own)), metal: 0.0, dim: "threat", current: false, pair_only: false };
                    let solar = State { id: format!("{name}.{}", self.name(kit.solar)), actor: name.clone(), response: Response::Building(kit.solar), words: format!("{name} builds a {} beside itself now, its list waiting: the energy store is under a quarter and draining ({:.0} of {:.0}, {:.0} in against {:.0} out)", self.unit_words(kit.solar), tick.snapshot.energy.current, tick.snapshot.energy.storage, tick.snapshot.energy.income, tick.snapshot.energy.usage), metal: 0.0, dim: "energy", current: false, pair_only: false };
                    slots.push(Slot { name: name.clone(), kind: Kind::Builder(unit.id), states: vec![keep, solar], queue_ahead: status.queue_ahead, idle: false, stop_cost: None, on_route: false, quiet: false });
                }
                continue;
            }
            // A started build under the queue mark, unthreatened: nothing but the build (H-HANDS-STARTED).
            if status.started.is_some() && !status.queue_ahead && !status.threatened {
                continue;
            }
            let queue = status.queue_ahead;
            let keep_words = match (&status.started, queue) {
                (Some((what, share)), true) => format!("finishes the {what} ({:.0}% done) and then waits for an order", share * 100.0),
                _ => format!("{name} {}", self.task_course(task, unit, &picture.places, frame, own)),
            };
            let idle = task.is_none() && unit.idle;
            let leaves = if queue { String::new() } else { self.leaves_words(task, &status.started) };
            let then = if queue { "then " } else { "" };
            let mut states = vec![State { id: format!("{name}.keep"), actor: name.clone(), response: Response::Keep, words: keep_words, metal: 0.0, dim: "threat", current: false, pair_only: false }];
            let mut push = |key: &str, response: Response, words: String, current: bool, pair_only: bool| {
                let dim = self.dim_of(&response);
                states.push(State { id: format!("{name}.{key}"), actor: name.clone(), response, words, metal: 0.0, dim, current, pair_only });
            };
            let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
            // A party with nothing armed in it (air constructors, a Mason) is no threat to a builder: constructors
            // fled unarmed air constructors for a minute (bluegecko-3v1-comet-catcher-2, 19:50).
            let armed = |p: &&Party| p.ids.iter().any(|id| enemies.iter().find(|e| e.id == *id).is_none_or(|e| e.def.is_none_or(|d| self.world.def(d).is_some_and(|d| d.weapon_count > 0))));
            let nearest_party = picture.parties.iter().filter(armed).map(|p| (p.at.dist2d(unit.pos), p)).filter(|(d, _)| *d < ALARM).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
            let walking_home = matches!(task, Some(Task::Walk { place, .. }) if place == "home");
            // Home is no way out when the party stands at it or between (bluegecko-3v1-comet-catcher-2, 15:41-15:51:
            // the retreat rule walked commander_t1 home into thirteen Stouts; onepass-player-8, 30:00, into a
            // Razorback): then the step away below is the way, from home or away from it alike.
            let home_unsafe = |p: &Party| self.home.dist2d(p.at) < ALARM + 200.0 || self.home.dist2d(p.at) + 300.0 < unit.pos.dist2d(p.at);
            // A retreat is never queued behind a build: a threatened builder leaves the build (H-HANDS-STARTED's
            // "unless threatened"; bluegecko-3v1-comet-catcher-2, 18:23: the commander on a list with `assist`
            // could not be moved from eleven enemies at its home).
            // 1. Home from enemy soldiers it does not outweigh (a lone scout is not one); the rule's default.
            let threat = nearest_party.filter(|p| !lone_scout(p));
            // Home is unsafe from any armed party at it, not only the one at the builder: the last commander of
            // bluegecko-3v1-comet-catcher-9 (19:58) stood 630 from home with four Bulls at home, no party within its
            // own alarm reach, and the pick chose "go home" over the player's escape list with home banned.
            let home_party = picture.parties.iter().filter(armed).filter(|p| !lone_scout(p)).filter(|p| home_unsafe(p)).min_by(|a, b| a.at.dist2d(self.home).total_cmp(&b.at.dist2d(self.home)));
            if unit.pos.dist2d(self.home) > AWAY {
                let party = threat;
                let why = party.map_or(String::new(), |p| format!(" from {} ({}; {})", p.name, p.composition, self.odds_words(&[unit], p, enemies)));
                let at_home = home_party.map_or(String::new(), |p| format!("; {} ({}) stands at home", p.name, p.composition));
                push("retreat_home", Response::RetreatHome, format!("{name} goes home{why} ({} away){at_home}{}", distance_words(unit.pos.dist2d(self.home)), self.leaves_words(task, &status.started)), walking_home, false);
            }
            if let Some(party) = threat.or(home_party) {
                // 1b. At home with a party it does not outweigh in the alarm reach: the way home is no way out, so a
                // step to the nearest place of ours out of the party's reach (onepass-player-2, 12:03-12:40: the
                // commander helped the plant at home while the block walked in from 671 to 250 and killed it; the
                // packet said "walks away west or south" and no state could).
                let away: Option<&super::Place> = picture
                    .places
                    .iter()
                    .filter(|pl| pl.name != "home" && pl.at.dist2d(unit.pos) > 100.0 && pl.at.dist2d(party.at) > ALARM + 200.0 && pl.at.dist2d(party.at) > unit.pos.dist2d(party.at) + 300.0)
                    .filter(|pl| picture.state["places"][&pl.name]["what"].as_str().is_some_and(|w| w.starts_with("our ")) || pl.spot.is_none())
                    .filter(|pl| self.reachable_for(self.walker_of(unit.def), pl.at))
                    .min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos)));
                if let Some(pl) = away {
                    let current = matches!(task, Some(Task::Walk { place, .. }) if *place == pl.name);
                    // Whether it gets there before contact (8.2), said, never pruned.
                    let their_fastest = party.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).filter_map(|d| self.world.def(d)).map(|d| d.speed).fold(0.0, f32::max);
                    let mine = self.world.def(unit.def).map_or(0.0, |d| d.speed);
                    let timing = if their_fastest > mine && mine > 0.0 {
                        let contact = party.at.dist2d(unit.pos) / (their_fastest - mine);
                        let arrive = unit.pos.dist2d(pl.at) / mine;
                        if arrive < contact { format!("; it arrives in {arrive:.0} s, before contact ({contact:.0} s)") } else { format!("; it arrives in {arrive:.0} s, after contact ({contact:.0} s): it is caught on the way unless something covers it") }
                    } else {
                        String::new()
                    };
                    push(&format!("step_away_{}", pl.name), Response::WalkTo(pl.name.clone()), format!("{name} steps away from {} ({}, {:.0} away, which it does not outweigh) to {} ({} away), out of its reach{timing}{}", party.name, party.composition, party.at.dist2d(unit.pos), pl.name, distance_words(unit.pos.dist2d(pl.at)), self.leaves_words(task, &status.started)), current, false);
                }
            }
            // Under aircraft: the nearest place with anti-air of ours beside it (8.3), a walk offered, never pruned.
            if under_fire.contains(&unit.id) && self.first_seen.iter().any(|d| self.world.domain_of(*d) == crate::world::Domain::Air) {
                let aa: Vec<Vec3> = own.iter().filter(|u| !u.being_built && super::glossary::entry(self.name(u.def)).is_some_and(|e| e.class.contains("anti-air"))).map(|u| u.pos).collect();
                if let Some(pl) = picture.places.iter().filter(|pl| aa.iter().any(|a| a.dist2d(pl.at) < AT_STRUCTURE) && pl.at.dist2d(unit.pos) > 100.0).min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos))) {
                    let current = matches!(task, Some(Task::Walk { place, .. }) if *place == pl.name);
                    push(&format!("under_flak_{}", pl.name), Response::WalkTo(pl.name.clone()), format!("{name} walks under our anti-air at {} ({} away) while his aircraft are about{}", pl.name, distance_words(unit.pos.dist2d(pl.at)), self.leaves_words(task, &status.started)), current, false);
                }
            }
            // A builder on the player's list is in the pass only while threatened, and then with the ways out of
            // the threat alone: the list is the player's order and the pick keeps off it (onepass-player-1: 117
            // picks on listed builders; onepass-player-2: a constructor sent home from a Pawn never got its list
            // back, the pick built it a turret and an extractor elsewhere for two minutes).
            if listed {
                if states.len() > 1 {
                    slots.push(Slot { name, kind: Kind::Builder(unit.id), states, queue_ahead: queue, idle, stop_cost: None, on_route: false, quiet: false });
                }
                continue;
            }
            // 2. The attack on a party it outweighs alone is a state of the party's threat slot (`threats.rs`), opened by
            // the party's own "needs answering" noul: as a builder state its noul sat at 0.31-0.43 under the flag while a
            // Pawn killed an extractor 450 from the commander (onepass-norules-hard-2, 2:47-2:51).
            // The rest only when free or on a filler, never over a build it has not started, unless queued behind it.
            let over_a_build = matches!(task, Some(Task::Build { .. }) | Some(Task::Reclaim { .. }) | Some(Task::ReclaimUnit { .. }) | Some(Task::Repair { .. })) && !queue;
            if over_a_build {
                if states.len() > 1 {
                    slots.push(Slot { name, kind: Kind::Builder(unit.id), states, queue_ahead: queue, idle, stop_cost: None, on_route: false, quiet: false });
                }
                continue;
            }
            // 3. Extractors: the nearest free spot, and the nearest one no enemy is near when the nearest has one.
            if can(kit.extractor) {
                let spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                let enemies_near = |i: usize| picture.state["places"][format!("spot_{i}")]["enemies_near"].as_str().map(str::to_string);
                // Every free spot the instructions name, nearest first (onepass-smoke-1: the packet said spot_45 and
                // the commander was offered the nearest only, spot_28; onepass-norules-hard-1: the nearest three named
                // were the middle spots the packet names as never, and the strip it names as the job was cut off).
                // The nearest spot, and a safe one, only when the instructions name none.
                let instructions = picture.state["instructions"].as_str().unwrap_or_default();
                // The nearest free spots always stand beside the named ones, nearest first, and the default is the
                // nearest (7.5): a slot that held only spot_2 and spot_19 at 107-115 s of walking sent the first
                // constructor across the map (game 10).
                let mut offered: Vec<(usize, f32)> = spots.iter().copied().take(NEAREST_SPOTS).collect();
                for named in spots.iter().copied().filter(|(i, _)| super::diet::names_spot(instructions, *i)).take(NAMED_SPOTS) {
                    if !offered.iter().any(|(i, _)| *i == named.0) {
                        offered.push(named);
                    }
                }
                offered.sort_by(|a, b| a.1.total_cmp(&b.1));
                if let Some(first) = offered.first().copied()
                    && enemies_near(first.0).is_some()
                    && let Some(safe) = spots.iter().copied().find(|(i, _)| enemies_near(*i).is_none())
                    && !offered.iter().any(|(i, _)| *i == safe.0)
                {
                    offered.push(safe);
                }
                let (standing, coming) = self.count_of(kit.extractor, own, pianist);
                for (i, seconds) in offered.iter() {
                    let place = &picture.state["places"][format!("spot_{i}")];
                    let current = matches!(task, Some(Task::Build { spot: Some(s), .. }) if s == i);
                    push(
                        &format!("extractor_spot_{i}"),
                        Response::Extractor(*i),
                        format!(
                            "{then}{name} builds a metal extractor at spot_{i} ({seconds:.0} s of walking, ground {}{}): our {}{}{leaves}",
                            place["ground"].as_str().unwrap_or_default(),
                            enemies_near(*i).map_or(String::new(), |e| format!(", enemies near: {e}")),
                            Self::ordinal(standing + coming + 1),
                            if coming > 0 { format!(" ({coming} under way already)") } else { String::new() }
                        ),
                        current,
                        false,
                    );
                }
            }
            // 4. Every building the allowance permits, else the faction's usual list.
            let build_list: Vec<UnitDefId> = self.world.def(unit.def).map(|d| d.build_options.clone()).unwrap_or_default();
            // An allowance with no units is no restriction: `produce` keeps an entry for the group alone after a null
            // list (onepass-player-7: three plants with `group: new` had their lists lifted at 18:48 and were offered
            // nothing until 21:20, the picture saying "it builds anything").
            // A list naming nothing this builder can build leaves it unrestricted (H-HANDS-PRODUCE as registered, not
            // as it ran): the opening's `produce {"all": [plant units]}` was every constructor's allowance too, and
            // no constructor was offered a turret, a solar or a nano turret from 2:40 to 9:40 while the standing
            // order for turrets beside each outer extractor stood and thirteen extractors died (player-14).
            let allowed = self
                .allowed_units(&name)
                .filter(|a| !a.units.is_empty())
                .filter(|a| a.units.iter().any(|e| self.world.def_named(super::allowance(e).0).is_some_and(|d| build_list.contains(&d))));
            let permits = |list: &[String], b: UnitDefId| {
                let unit_name = self.name(b).to_string();
                let made = pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0);
                (0..list.len()).any(|k| super::allowance(&list[k]).0 == unit_name && super::entry_permits(list, k, made))
            };
            let usual = super::super::roster::usual_menu(self.name(unit.def));
            // An allowance whose every count is used up permits nothing, not everything (models-medium-gpt6-astra:
            // `armrectr:1` ran out and the lab made three more Lazarus under the usual menu; the player: "exhausted-only
            // production caps cannot be trusted").
            let offered: Vec<UnitDefId> = match &allowed {
                Some(Allowance { units: list, .. }) => build_list.iter().copied().filter(|b| permits(list, *b)).collect(),
                None => build_list.iter().copied().filter(|b| usual.contains(&self.name(*b))).collect(),
            };
            let reach = self.world.def(unit.def).map_or(100.0, |d| d.build_distance) + 300.0;
            for def in offered.iter().copied().filter(|d| *d != kit.extractor) {
                let Some(d) = self.world.def(def) else { continue };
                if super::glossary::entry(&d.name).is_some_and(|e| e.has_flag("on_water")) && !self.world.water_within(unit.pos, reach) {
                    continue;
                }
                let nano = d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty();
                if nano && !own.iter().any(|u| self.world.is_factory_def(u.def) && !u.being_built) {
                    continue;
                }
                let key = self.name(def).to_string();
                let current_def = matches!(task, Some(Task::Build { def: td, .. }) if *td == def);
                if !self.placed_at_place(def) {
                    // A solar the moment energy stalls is the rule's default (`only_when_stalling`); one under way is not
                    // another. And a solar from a free builder whenever the store is under a quarter and draining,
                    // whatever the packet says short of `solar never`: energy ran dry behind three plants at 7:35 in
                    // three games running (bluegecko-3v1-comet-catcher-7, -8, -9) while the player was on the front.
                    let words = format!("{then}{name} builds a {} beside itself: {}{leaves}", self.unit_words(def), self.build_words(pianist, unit, def, None, tick, own));
                    push(&key, Response::Building(def), words, current_def, false);
                    continue;
                }
                if d.extracts_metal > 0.0 {
                    // A tier-2 extractor over the nearest extractor of ours still to upgrade.
                    let radius = self.spot_occupied_radius();
                    let ours = own.iter().filter(|u| kit.is_extractor(u.def) && u.def != def && !u.being_built && !own.iter().any(|f| f.def == def && f.pos.dist2d(u.pos) < radius)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                    if let Some(ours) = ours
                        && let Some(place) = picture.places.iter().filter(|p| p.at.dist2d(ours.pos) < AT_STRUCTURE).min_by(|a, b| a.at.dist2d(ours.pos).total_cmp(&b.at.dist2d(ours.pos)))
                    {
                        let words = format!("{then}{name} builds a {} over our extractor at {}: {}{leaves}", self.unit_words(def), place.name, self.build_words(pianist, unit, def, Some(ours.pos), tick, own));
                        push(&format!("{key}_{}", place.name), Response::BuildingAt(def, place.name.clone()), words, current_def, false);
                    }
                    continue;
                }
                if d.weapon_count > 0 {
                    // A turret beside each extractor of ours within reach that none covers (nearest two); the rule's
                    // default at the nearest (outer only under `beside_each_outer_extractor`). Not beside an
                    // extractor a party stands at (a lone scout is what the turret is for).
                    // Any armed building standing, started or ordered within the cover covers it (onepass-smoke-3: 54
                    // turrets for 16 extractors, the frames placed past a cover of 200 and each one read as none).
                    let armed = |d: UnitDefId| self.world.def(d).is_some_and(|x| x.speed == 0.0 && x.weapon_count > 0);
                    let ordered_near = |pos: Vec3| pianist.tasks.values().chain(pianist.queued.values()).any(|t| matches!(t, Task::Build { def: td, near, .. } if armed(*td) && near.dist2d(pos) < TURRET_COVER));
                    let mut uncovered: Vec<&OwnUnit> = own
                        .iter()
                        .filter(|u| kit.is_extractor(u.def) && !u.being_built && u.pos.dist2d(unit.pos) < TURRET_REACH)
                        .filter(|u| !picture.parties.iter().any(|p| p.at.dist2d(u.pos) < ALARM && !lone_scout(p)))
                        .filter(|u| !own.iter().any(|t| armed(t.def) && t.pos.dist2d(u.pos) < TURRET_COVER) && !ordered_near(u.pos))
                        .collect();
                    uncovered.sort_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                    let mut n = 0;
                    for extractor in uncovered {
                        let Some(place) = picture.places.iter().filter(|p| p.at.dist2d(extractor.pos) < AT_STRUCTURE).min_by(|a, b| a.at.dist2d(extractor.pos).total_cmp(&b.at.dist2d(extractor.pos))) else { continue };
                        let outer = extractor.pos.dist2d(self.home) > OUTER;
                        let current = current_def && matches!(task, Some(Task::Build { near, .. }) if near.dist2d(extractor.pos) < TURRET_COVER);
                        let words = format!("{then}{name} builds a {} beside our {}extractor at {}, which no turret covers: {}{leaves}", self.unit_words(def), if outer { "outer " } else { "" }, place.name, self.build_words(pianist, unit, def, Some(extractor.pos), tick, own));
                        push(&format!("{key}_{}", place.name), Response::BuildingAt(def, place.name.clone()), words, current, false);
                        n += 1;
                        if n == 2 {
                            break;
                        }
                    }
                    continue;
                }
                // A radar, a jammer: at the nearest marked places, else beside the builder.
                let mut at: Vec<&super::Place> = marks.iter().copied().filter(|p| p.at.dist2d(unit.pos) < WALK_REACH).collect();
                at.sort_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos)));
                if at.is_empty() {
                    let words = format!("{then}{name} builds a {} beside itself: {}{leaves}", self.unit_words(def), self.build_words(pianist, unit, def, None, tick, own));
                    push(&key, Response::Building(def), words, current_def, false);
                }
                for place in at.into_iter().take(2) {
                    let words = format!("{then}{name} builds a {} at {}: {}{leaves}", self.unit_words(def), place.name, self.build_words(pianist, unit, def, Some(place.at), tick, own));
                    push(&format!("{key}_{}", place.name), Response::BuildingAt(def, place.name.clone()), words, false, false);
                }
            }
            // 5. Helping: the nearest factory (`job help_factory` makes it the default), and another builder's started
            // build within reach; a pair-only help beside any dear build another builder could start.
            if let Some(lab) = own.iter().filter(|u| self.world.is_factory_def(u.def) && !u.being_built).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))) {
                let current = matches!(task, Some(Task::Assist { lab: l, .. }) if *l == lab.id);
                let lab_name = self.actor_name(lab.id);
                // What the help is worth (13.6): a pick at 0.32 over an extractor was the default answer of an idle
                // constructor while the plant was starved or the store full.
                let m = &tick.snapshot.metal;
                let worth = match draws.iter().find(|(who, _, _)| *who == lab_name).map(|(_, _, metal)| *metal) {
                    Some(draw) if m.current < 100.0 && m.income < draw => format!("the plant is starved (it would spend {draw:.1} a second against {:.1} coming in and {:.0} stored): helping adds nothing until metal comes", m.income, m.current),
                    Some(draw) if m.current >= m.storage - 1.0 => format!("the store is full: helping spends it ({draw:.1} a second more)"),
                    Some(draw) => format!("the plant draws {draw:.1} a second against {:.1} coming in and {:.0} stored: helping makes its unit sooner while the store lasts", m.income, m.current),
                    None => "adds its build power to whatever it makes (metal must be coming in faster than the factory spends it)".to_string(),
                };
                push(&format!("help_{lab_name}"), Response::Assist(lab.id), format!("{then}{name} helps {lab_name} build: {worth}{leaves}"), current, false);
            }
            for (other, def, share) in building.iter().filter(|(o, ..)| o.id != unit.id && o.pos.dist2d(unit.pos) < HELP_REACH) {
                let current = matches!(task, Some(Task::Assist { lab: l, .. }) if *l == other.id);
                let other_name = self.actor_name(other.id);
                push(&format!("help_{other_name}"), Response::Assist(other.id), format!("{then}{name} helps {other_name} build its {} ({:.0}% done), {} away{leaves}", self.short_words(*def), share * 100.0, distance_words(unit.pos.dist2d(other.pos))), current, false);
            }
            for other in builders.iter().filter(|o| o.id != unit.id && o.pos.dist2d(unit.pos) < HELP_REACH && !building.iter().any(|(b, ..)| b.id == o.id)) {
                let other_name = self.actor_name(other.id);
                push(&format!("help_{other_name}"), Response::Assist(other.id), format!("{then}{name} helps {other_name} with what it builds, {} away{leaves}", distance_words(unit.pos.dist2d(other.pos))), false, true);
            }
            // 6. Wrecks, repairs, walks.
            if let Some(field) = self.reclaim.fields.iter().filter(|f| f.metal >= 100.0 && f.at.dist2d(unit.pos) < RECLAIM_WITHIN).max_by(|a, b| a.metal.total_cmp(&b.metal)) {
                let current = matches!(task, Some(Task::Reclaim { at, .. }) if at.dist2d(field.at) < 100.0);
                push("reclaim", Response::Reclaim(field.at), format!("{then}{name} takes apart the wrecks at {} ({:.0} metal lying there{}){leaves}", self.place_words(&picture.places, field.at), field.metal, if field.safe { "" } else { "; not safe ground" }), current, false);
            }
            let hurt = own
                .iter()
                .filter(|u| u.id != unit.id && !u.being_built && u.health < u.max_health * 0.7 && u.pos.dist2d(unit.pos) < REPAIR_WITHIN)
                .filter(|u| self.world.is_commander_def(u.def) || self.world.def(u.def).is_some_and(|d| d.speed == 0.0))
                .min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
            if let Some(hurt) = hurt {
                let current = matches!(task, Some(Task::Repair { target, .. }) if *target == hurt.id);
                push("repair", Response::Repair(hurt.id), format!("{then}{name} repairs our {} at {} ({:.0}% health){leaves}", self.name(hurt.def), self.place_words(&picture.places, hurt.pos), hurt.health / hurt.max_health * 100.0), current, false);
            }
            let mut walks: Vec<&super::Place> = marks.iter().copied().filter(|p| p.at.dist2d(unit.pos) < WALK_REACH && p.at.dist2d(unit.pos) > 150.0).collect();
            walks.sort_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos)));
            for place in walks.into_iter().take(2) {
                let current = matches!(task, Some(Task::Walk { place: p, .. }) if *p == place.name);
                push(&format!("walk_{}", place.name), Response::WalkTo(place.name.clone()), format!("{then}{name} walks to {} ({} away) and waits there{leaves}", place.name, distance_words(unit.pos.dist2d(place.at))), current, false);
            }
            if states.len() > 1 {
                slots.push(Slot { name, kind: Kind::Builder(unit.id), states, queue_ahead: queue, idle, stop_cost: None, on_route: false, quiet: false });
            }
        }

        // Labs: one order ahead (H-HANDS-MENU); a lab with one waiting has nothing to decide.
        let extractors = own.iter().filter(|u| !u.being_built && self.world.is_extractor_def(u.def)).count();
        let constructors = own.iter().filter(|u| !u.being_built && self.world.is_constructor_def(u.def)).count();
        let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
        let army_metal: f32 = soldiers.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_factory_def(u.def)) {
            let name = self.actor_name(unit.id);
            if pianist.lab_queue.get(&unit.id).is_some_and(|q| !q.is_empty()) {
                continue;
            }
            let Some(def) = self.world.def(unit.def) else { continue };
            let on_pad = own.iter().find(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| (self.short_words(u.def), u.health / u.max_health.max(1.0)));
            let idle = on_pad.is_none() && unit.idle;
            let keep_words = match &on_pad {
                Some((what, share)) => format!("{name} builds the {what} on its pad ({:.0}% done) and then nothing: the next unit waits for an order", share * 100.0),
                None => format!("{name} stands idle, building nothing"),
            };
            let then = if on_pad.is_some() { "then " } else { "" };
            let mut states = vec![State { id: format!("{name}.keep"), actor: name.clone(), response: Response::Keep, words: keep_words, metal: 0.0, dim: "threat", current: false, pair_only: false }];
            // An allowance with no units is no restriction: `produce` keeps an entry for the group alone after a null
            // list (onepass-player-7: three plants with `group: new` had their lists lifted at 18:48 and were offered
            // nothing until 21:20, the picture saying "it builds anything").
            let allowed = self.allowed_units(&name).filter(|a| !a.units.is_empty());
            let permits = |list: &[String], b: UnitDefId| {
                let unit_name = self.name(b).to_string();
                let made = pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0);
                (0..list.len()).any(|k| super::allowance(&list[k]).0 == unit_name && super::entry_permits(list, k, made))
            };
            // An allowance whose every count is used up permits nothing, not everything (models-medium-gpt6-astra:
            // `armrectr:1` ran out and the lab made three more Lazarus under the whole menu).
            let buildables: Vec<UnitDefId> = match &allowed {
                Some(Allowance { units: list, .. }) => def.build_options.iter().copied().filter(|b| permits(list, *b)).collect(),
                None => def.build_options.clone(),
            };
            let coming = own.iter().filter(|u| u.being_built && self.world.is_constructor_def(u.def)).count();
            for buildable in buildables {
                let (standing, being_made) = self.count_of(buildable, own, pianist);
                let have = if self.world.is_constructor_def(buildable) {
                    format!("we have {constructors} constructors{} for {extractors} extractors", if coming > 0 { format!(" and {coming} being made") } else { String::new() })
                } else if self.world.def(buildable).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0) {
                    format!("our soldiers: {}", super::picture::soldier_words(soldiers.len(), army_metal))
                } else {
                    String::new()
                };
                let key = self.name(buildable).to_string();
                let cost = self.world.def(buildable).map_or(0.0, |d| d.metal_cost);
                states.push(State {
                    id: format!("{name}.{key}"),
                    actor: name.clone(),
                    response: Response::Next(buildable),
                    words: format!("{then}{name} makes a {} ({cost:.0} metal; we have {standing}{}): {have}", self.unit_words(buildable), if being_made > 0 { format!(" and {being_made} being made") } else { String::new() }),
                    metal: 0.0,
                    dim: self.dim_of(&Response::Next(buildable)),
                    current: false,
                    pair_only: false,
                });
            }
            if states.len() > 1 {
                slots.push(Slot { name, kind: Kind::Lab(unit.id), states, queue_ahead: on_pad.is_some(), idle, stop_cost: None, on_route: false, quiet: false });
            }
        }

        // Groups: their course, beside the threat slots. A roving group has none: it is the lane's (H-MICRO-ROVE), and
        // nothing merges into it.
        let group_names: Vec<(String, Option<Vec3>, crate::world::Domain)> = pianist.groups.iter().filter(|g| !g.roving).map(|g| (g.name.clone(), super::groups::centre_of(&g.units(own)), g.domain)).collect();
        let scout_out = pianist.groups.iter().any(|g| g.scout && g.roving);
        let instructions = picture.state["instructions"].as_str().unwrap_or_default().to_string();
        let extractors_ours = own.iter().filter(|u| !u.being_built && self.world.is_extractor_def(u.def)).count();
        let income_ours = tick.snapshot.metal.income;
        for group in pianist.groups.iter().filter(|g| !g.roving) {
            let name = format!("group_{}", group.name);
            let units = group.units(own);
            // The group as a body (H-HANDS-GROUP-BODY): distances from where the body stands, the nearest party by
            // its nearest member, the odds on the part in the fight.
            let goal = match &group.task {
                GroupTask::Move { to, .. } => Some(*to),
                GroupTask::Engage { at, .. } => Some(*at),
                GroupTask::Hunt(h) => Some(h.at),
                GroupTask::Hold { .. } => None,
            };
            let nearest_to_any = |p: &super::Party| units.iter().map(|u| u.pos.dist2d(p.at)).fold(f32::INFINITY, f32::min);
            let toward = goal.or_else(|| picture.parties.iter().map(|p| (nearest_to_any(p), p)).filter(|(d, _)| *d < ALARM).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p.at));
            let Some(body) = group.body(own, toward) else { continue };
            let centre = body.at;
            let doing = picture.state["actors"][&name]["doing"].as_str().unwrap_or("holds").to_string();
            let hunting = matches!(group.task, GroupTask::Hunt(_));
            let idle = matches!(group.task, GroupTask::Hold { .. }) && !hunting;
            let mut states = vec![State { id: format!("{name}.keep"), actor: name.clone(), response: Response::Keep, words: format!("{name} {doing}{}", if hunting { " (hunting a raider with a few of its soldiers)" } else { "" }), metal: 0.0, dim: "threat", current: false, pair_only: false }];
            let mut push = |key: &str, response: Response, words: String, current: bool| {
                let dim = self.dim_of(&response);
                states.push(State { id: format!("{name}.{key}"), actor: name.clone(), response, words, metal: 0.0, dim, current, pair_only: false });
            };
            let nearest_party = picture.parties.iter().map(|p| (nearest_to_any(p), p)).filter(|(d, _)| *d < ALARM).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
            let walking_back = matches!(&group.task, GroupTask::Move { fight: false, place, .. } if place == "home" || place.starts_with(super::groups::LAST_HOLD));
            // A body on its engagement plan (H-HANDS-ENGAGEMENT-PLAN) takes no rule's default walk: the plan is Jev's
            // pick and a pick still changes it.
            let leave = match &group.task {
                GroupTask::Hold { .. } if !hunting => format!(", leaving {} unguarded", picture.state["actors"][&name]["at"].as_str().unwrap_or("where it stands")),
                GroupTask::Move { place, .. } => format!(", abandoning its way to {place}"),
                GroupTask::Engage { .. } => ", leaving the party it was attacking".to_string(),
                _ => String::new(),
            };
            // 2. Walks to named places: home, the player's marks and passages within reach, and every spot the packet
            // names, however far (the `instruct` tool promises it; a group's walks went to home and non-spot places
            // only, so "raids his north corner (spot_9, then spot_2, spot_7, spot_14)" put nothing on the menu and
            // the player advanced the station by hand every 30-40 s, game 10).
            // `shelling` is an estimate, never a walk (10.4): the shooter is answered by the states below.
            let named_spot = |p: &super::Place| p.spot.is_some() && super::diet::names(&instructions, &p.name);
            // The group's own paragraph: every place it names is on the menu, however far, in the packet's order
            // (routes-in-prose §4.1: a route's stops); of the other named places, the nearest three.
            let paragraph = super::diet::paragraph(&instructions, &name).unwrap_or(&instructions).to_string();
            let on_route = |p: &super::Place| p.name != "home" && super::diet::names(&paragraph, &p.name);
            let mut named: Vec<&super::Place> = picture.places.iter().filter(|p| !p.name.starts_with("shelling") && (p.name == "home" || p.spot.is_none() || named_spot(p)) && (p.at.dist2d(centre) < WALK_REACH || named_spot(p) || on_route(p)) && p.at.dist2d(centre) > STATION_SLACK).collect();
            named.sort_by(|a, b| a.at.dist2d(centre).total_cmp(&b.at.dist2d(centre)));
            let mut offered: Vec<&super::Place> = named.iter().copied().filter(|p| on_route(p)).collect();
            offered.extend(named.iter().copied().filter(|p| !on_route(p)).take(3));
            // A route's legs advance (fight on the way) when its paragraph says so, else walk; each leg's words say
            // what is known to stand at its end and the odds on a party there (§4.4: the E3 advance's whole text was
            // its station's name while every state against it carried its cost).
            let advancing = paragraph.contains("advanc");
            for place in &offered {
                let fight = advancing && on_route(place);
                let current = matches!(&group.task, GroupTask::Move { place: p, fight: f, .. } if *p == place.name && *f == fight);
                let mut ahead: Vec<String> = Vec::new();
                let mut his: BTreeMap<&str, usize> = BTreeMap::new();
                for (def, pos, _) in self.enemy_buildings.values() {
                    if pos.dist2d(place.at) < AHEAD {
                        *his.entry(self.name(*def)).or_default() += 1;
                    }
                }
                if !his.is_empty() {
                    ahead.push(format!("his {} there", his.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")));
                }
                if let Some(p) = picture.parties.iter().filter(|p| p.at.dist2d(place.at) < AHEAD).min_by(|a, b| a.at.dist2d(place.at).total_cmp(&b.at.dist2d(place.at))) {
                    ahead.push(format!("{} ({}{}) at it: {}", p.name, p.composition, under(p), self.group_odds(&body, p, enemies, &tick.snapshot.allies).0));
                }
                let ahead = if ahead.is_empty() { String::new() } else { format!("; ahead: {}", ahead.join("; ")) };
                let words = if fight {
                    format!("{name} advances to {} ({} away), fighting on the way{ahead}{leave}", place.name, distance_words(centre.dist2d(place.at)))
                } else {
                    format!("{name} walks to {} ({} away) without stopping to fight on the way{ahead}{leave}", place.name, distance_words(centre.dist2d(place.at)))
                };
                push(&format!("{}_{}", if fight { "advance" } else { "walk" }, place.name), Response::Walk { place: place.name.clone(), fight }, words, current);
            }
            // 2b. The group's own initiative (H-HANDS-GROUP-STATES): a raid on his buildings known within reach, a
            // sweep of the spots nothing of ours has looked at, a gather when strung out, the answers to a shooter
            // out of sight, and the artillery's standoff. Offered, never a default: the missing piece was the option
            // (the user, game 10 at 5:58: two groups in position with 22-27 unguarded buildings within 3,000 held
            // their stations for minutes; the packet's raid route had no state to land in).
            let walker = self.group_walker(group, own);
            let speed = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
            let walk_words = |d: f32| if speed.is_finite() && speed > 0.0 { format!("{d:.0} away, {:.0} s of walking", d / speed) } else { format!("{d:.0} away") };
            let mut raids: Vec<(f32, &super::Place, String)> = Vec::new();
            for place in picture.places.iter().filter(|p| p.name != "home" && !p.name.starts_with("shelling") && self.reachable_for(walker, p.at)) {
                let theirs: Vec<(UnitDefId, i32)> = self.enemy_buildings.values().filter(|(_, pos, _)| pos.dist2d(place.at) < 500.0).map(|(def, _, seen)| (*def, *seen)).collect();
                if theirs.is_empty() {
                    continue;
                }
                let d = body.front.dist2d(place.at);
                if d > RAID_REACH || d < STATION_SLACK {
                    continue;
                }
                let turrets: Vec<&UnitDefId> = theirs.iter().map(|(def, _)| def).filter(|def| self.world.def(**def).is_some_and(|d| d.weapon_count > 0 && d.reach > 0.0)).collect();
                let mut counts: BTreeMap<String, usize> = BTreeMap::new();
                for (def, _) in &theirs {
                    *counts.entry(self.short_words(*def)).or_default() += 1;
                }
                let oldest = theirs.iter().map(|(_, seen)| *seen).min().unwrap_or(frame);
                let extractors = theirs.iter().filter(|(def, _)| self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)).count();
                let income = if extractors > 0 && extractors_ours > 0 { format!("; about {:.1} a second of his income", extractors as f32 * income_ours / extractors_ours as f32) } else { String::new() };
                let guard = if turrets.is_empty() { "no turret within 500".to_string() } else { format!("under {} turret{} ({})", turrets.len(), if turrets.len() == 1 { "" } else { "s" }, turrets.iter().map(|d| self.short_words(**d)).collect::<Vec<_>>().join(", ")) };
                let words = format!("{name} raids {} and kills his {} there (seen {} ago; {guard}{income}): {}, fighting on the way{leave}", place.name, counts.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), super::picture::clock(frame - oldest), walk_words(d));
                raids.push((d, place, words));
            }
            raids.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (_, place, words) in raids.iter().take(RAID_STATES) {
                let current = matches!(&group.task, GroupTask::Move { place: p, fight: true, .. } if *p == place.name);
                push(&format!("raid_{}", place.name), Response::Walk { place: place.name.clone(), fight: true }, words.clone(), current);
            }
            // The sweep: the nearest spot nothing of ours has looked at that this group reaches; his start box first
            // when the group's paragraph is about his base or his side (routes-in-prose §4.5: at 6:01 the sweep named
            // four spots of our own half to a ball whose job was finding him); the group advances to it as a body
            // and the state is offered again from there.
            let never_looked: Vec<&super::Place> = picture.places.iter().filter(|p| p.spot.is_some_and(|i| self.spot_seen(i).is_none()) && self.reachable_for(walker, p.at) && p.at.dist2d(centre) > STATION_SLACK).collect();
            let seek_his = paragraph.contains("base") || paragraph.contains("his ");
            let our_ally = self.world.hello.ally_team;
            let in_his_box = |p: &super::Place| self.world.hello.start_boxes.iter().any(|b| b.ally_team != our_ally && b.contains(p.at));
            if let Some(next) = never_looked.iter().copied().min_by(|a, b| (seek_his && !in_his_box(a), a.at.dist2d(body.front)).partial_cmp(&(seek_his && !in_his_box(b), b.at.dist2d(body.front))).unwrap_or(std::cmp::Ordering::Equal)) {
                let mut then: Vec<(f32, String)> = never_looked.iter().filter(|p| p.name != next.name).map(|p| (p.at.dist2d(next.at), p.name.clone())).collect();
                then.sort_by(|a, b| a.0.total_cmp(&b.0));
                let then: Vec<String> = then.into_iter().map(|(_, n)| n).take(3).collect();
                let current = matches!(&group.task, GroupTask::Move { place: p, fight: true, .. } if *p == next.name);
                push("sweep", Response::Walk { place: next.name.clone(), fight: true }, format!("{name} sweeps the spots nothing of ours has looked at ({} of {}): {} first ({}), as one body fighting on the way{}{leave}", never_looked.len(), self.world.hello.metal_spots.len(), next.name, walk_words(body.front.dist2d(next.at)), if then.is_empty() { String::new() } else { format!(", then {}", then.join(", ")) }), current);
            }
            // The gather: a strung-out group holds at the place nearest its front until its tail is up.
            if body.strung_out() && group.task.busy() {
                if let Some(at) = picture.places.iter().filter(|p| !p.name.starts_with("shelling")).min_by(|a, b| a.at.dist2d(body.front).total_cmp(&b.at.dist2d(body.front))) {
                    let tail_seconds = if speed.is_finite() && speed > 0.0 { format!("{:.0} s", body.length / speed) } else { "a while".to_string() };
                    push(&format!("gather_{}", at.name), Response::Gather(at.name.clone()), format!("{name} gathers at {} ({} from its front): the front holds there until the tail is up ({} of {} arrived, the tail {:.0} behind, about {tail_seconds}), then goes on where told{leave}", at.name, distance_words(body.front.dist2d(at.at)), body.arrived, body.core.len(), body.length), group.gathering);
                }
            }
            // The shooter out of sight, read from the hits on this group's own members: close on it as one body when
            // the estimate says we outweigh it, or pull out of its reach to the nearest place beyond it (game 3 19:30:
            // a group under Bull fire had neither). Its place is the picture's `shelling_<group>` when the group's
            // shooter stands apart from the side's, else `shelling`.
            if let Some(s) = self.shelling_for(&group.members)
                && units.iter().any(|u| under_fire.contains(&u.id))
            {
                let verdict = self.unseen_shooter_words(&units, &s);
                let shooter_place = picture.places.iter().find(|p| p.name == format!("shelling_{}", group.name)).or_else(|| picture.places.iter().find(|p| p.name == "shelling"));
                if let Some(sp) = shooter_place
                    && self.reachable_for(walker, s.at)
                {
                    push("close_on_shooter", Response::Walk { place: sp.name.clone(), fight: true }, format!("{name} closes on the shooter out of sight as one body, toward `{}` ({}, {}): {verdict}{leave}", sp.name, self.place_words(&picture.places, s.at), walk_words(body.front.dist2d(s.at))), false);
                }
                if let Some(out) = picture.places.iter().filter(|p| !p.name.starts_with("shelling") && p.at.dist2d(s.at) > s.range + 150.0 && self.reachable_for(walker, p.at)).min_by(|a, b| a.at.dist2d(centre).total_cmp(&b.at.dist2d(centre))) {
                    push(&format!("pull_out_{}", out.name), Response::Walk { place: out.name.clone(), fight: false }, format!("{name} pulls out of the shooter's reach ({:.0}) to {} ({}), without fighting on the way{leave}", s.range, out.name, walk_words(centre.dist2d(out.at))), false);
                }
            }
            // The artillery: long-reach members shell the nearest party from a standoff, the rest standing between
            // (game 9: four Shellshockers never used in the nest fight).
            let long_reach: Vec<&OwnUnit> = units.iter().copied().filter(|u| self.world.def(u.def).is_some_and(|d| d.reach >= ARTILLERY_REACH && d.speed > 0.0)).collect();
            if !long_reach.is_empty()
                && let Some(p) = nearest_party
            {
                let reach = long_reach.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.reach).fold(0.0, f32::max);
                let arty: BTreeMap<&str, usize> = long_reach.iter().fold(BTreeMap::new(), |mut m, u| { *m.entry(self.name(u.def)).or_default() += 1; m });
                let current = group.shelling && matches!(&group.task, GroupTask::Move { place, .. } if *place == format!("standoff from {}", p.name));
                push(&format!("shell_{}", p.name), Response::Shell(p.name.clone()), format!("{name} shells {} ({}{}) with its {} (reach {reach:.0}) from {:.0} short of it, the other {} soldiers standing between as the screen; {}", p.name, p.composition, under(p), arty.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), reach * 0.85, units.len() - long_reach.len(), self.group_odds(&body, p, enemies, &tick.snapshot.allies).1), current);
            }
            // 3. The ways back.
            {
                let at_point = |at: Vec3| -> String {
                    let near: Vec<String> = picture.parties.iter().filter(|p| p.at.dist2d(at) < ALARM).map(|p| format!("{} ({}) is {:.0} from it", p.name, p.composition, p.at.dist2d(at))).collect();
                    if near.is_empty() { String::new() } else { format!("; into fire: {}", near.join(", ")) }
                };
                // A walk back in progress is the slot's current state, so the base world keeps it until the pick
                // changes it or the group arrives.
                let retreating = matches!(&group.task, GroupTask::Move { fight: false, place, .. } if place == "home");
                if retreating || (!walking_back && centre.dist2d(self.home) > STATION_SLACK) {
                    let words = if retreating { format!("{name} keeps falling back to our base ({} away){}", distance_words(centre.dist2d(self.home)), at_point(self.home)) } else { format!("{name} falls back to our base ({} away){}{leave}", distance_words(centre.dist2d(self.home)), at_point(self.home)) };
                    push("retreat", Response::Retreat, words, retreating);
                }
                if let GroupTask::Move { fight: false, place, to, .. } = &group.task
                    && place.starts_with(super::groups::LAST_HOLD)
                {
                    push("fall_back", Response::FallBack, format!("{name} keeps falling back to where it last held ({}, {} away) without fighting on the way", self.place_words(&picture.places, *to), distance_words(to.dist2d(centre))), true);
                }
                if !walking_back
                    && group.task.busy()
                    && let Some(back) = group.last_hold
                    && back.dist2d(centre) > 300.0
                    && nearest_party.is_none_or(|p| p.at.dist2d(back) > p.at.dist2d(centre) + 300.0)
                {
                    push("fall_back", Response::FallBack, format!("{name} falls back to where it last held ({}, {} away) without fighting on the way; less far than the base{}", self.place_words(&picture.places, back), distance_words(back.dist2d(centre)), at_point(back)), false);
                }
            }
            // 4. Detachments: a scout (one out at a time) and a detachment to a marked place; `no_detachments` prunes.
            if units.len() >= 2 {
                // The scout roves (H-MICRO-ROVE): the group's fastest soldier, run in code with no orders from the hands.
                if !scout_out
                    && group.domain != crate::world::Domain::Air
                    && let Some(fastest) = units.iter().filter_map(|u| self.world.def(u.def)).max_by(|a, b| a.speed.total_cmp(&b.speed))
                {
                    push("scout", Response::Scout, format!("{name} sends its fastest soldier (a {}, {:.0} a second) to rove on its own: it looks at what we know least, his start box first, kills what it finds unguarded and keeps out of the reach of anything that can shoot it; the rest carry on", self.short_words(fastest.id), fastest.speed), false);
                }
                for place in named.iter().filter(|p| p.name != "home").take(2) {
                    let n = (units.len() / 2).max(1);
                    push(&format!("split_{}", place.name), Response::Split(n, place.name.clone()), format!("{name} sends {n} of its {} soldiers to advance to {} ({} away) as a group of their own; the rest carry on", units.len(), place.name, distance_words(centre.dist2d(place.at))), false);
                }
            }
            // 5. A merge: into the nearest group of its domain, the words saying what the one body would be and what
            // it is doing (player-24, 7:26: "group_H merges into group_G and takes its task" rated 0.00-0.03 in the
            // pick, six such worlds a second, while ten groups of one Blitz stood on one spot and a hunt that had
            // failed the second before was picked at 0.44).
            if let Some((other, Some(other_centre), _)) = group_names.iter().filter(|(n, c, d)| *n != group.name && c.is_some() && *d == group.domain).min_by(|a, b| a.1.unwrap().dist2d(centre).total_cmp(&b.1.unwrap().dist2d(centre)))
                && let Some(other_group) = pianist.groups.iter().find(|g| g.name == *other)
            {
                let theirs = other_group.units(own);
                let both: Vec<&OwnUnit> = units.iter().chain(theirs.iter()).copied().collect();
                let metal: f32 = both.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
                let doing = picture.state["actors"][&format!("group_{other}")]["doing"].as_str().unwrap_or("holding").to_string();
                push(
                    &format!("join_group_{other}"),
                    Response::Join(other.clone()),
                    format!(
                        "{name} ({}) merges into group_{other} ({}, {} away, {doing}): one body of {} ({} soldiers worth {metal:.0} metal) on group_{other}'s course",
                        self.composition_words(&units),
                        self.composition_words(&theirs),
                        distance_words(centre.dist2d(*other_centre)),
                        self.composition_words(&both),
                        both.len()
                    ),
                    false,
                );
            }
            // The stop's cost (routes-in-prose §4.2): holding at a place of its route it has reached while the
            // packet names further places for it, said in world 1's line.
            let stop_cost = if idle
                && let Some((last, _)) = group.reached.last()
                && picture.places.iter().any(|p| p.name == *last && p.at.dist2d(centre) < 300.0)
            {
                let order = super::diet::named_in_order(&paragraph, picture.places.iter().map(|p| p.name.as_str()).filter(|n| *n != "home"));
                let done = |n: &str| group.reached.iter().any(|(r, _)| r == n);
                let after: Vec<&str> = order.iter().copied().skip_while(|n| *n != last.as_str()).skip(1).filter(|n| !done(n)).collect();
                let left: Vec<&str> = order.iter().copied().filter(|n| !done(n)).collect();
                match (after.first(), left.is_empty()) {
                    (Some(next), _) => Some(format!("{name} holds at {last}, a stop of its route it has reached, with the route's next stop {next} not ordered")),
                    (None, false) => Some(format!("{name} holds at {last}, a stop of its route it has reached, with further places named for it ({}) not ordered", left.join(", "))),
                    (None, true) => None,
                }
            } else {
                None
            };
            // On a leg of its route (§4.4): the paragraph names two or more places and the walk is to one of them.
            let on_route = super::diet::named_in_order(&paragraph, picture.places.iter().map(|p| p.name.as_str()).filter(|n| *n != "home")).len() >= 2
                && matches!(&group.task, GroupTask::Move { place, .. } if picture.places.iter().any(|p| p.name == *place && on_route(p)));
            if states.len() > 1 {
                slots.push(Slot { name, kind: Kind::Group(group.name.clone()), states, queue_ahead: false, idle, stop_cost, on_route, quiet: false });
            }
        }
        slots
    }
}

/// The pre-pass's nouls: per threat its answer and one per open state (`threats.rs`); per actor with an open
/// state whether it should change course (an idle actor is always open), and one per open state whether it is the
/// move. onepass-smoke-1 asked a noul per kind of action instead and they came back at 0.44-0.60 everywhere, coin
/// flips that pruned the one state the packet asked for.
/// The state's `asking` line: the preamble the bulk questions leave out.
pub(super) const ASKING: &str = "Each question below that names an actor and a move without more asks: is that what the actor should do now, rather than the course named after it? Judge it from that actor's entry under `actors`, `economy`, `ours` and the player's `instructions`.";

pub(super) fn gate_questions(slots: &[Slot], told: bool) -> Vec<(String, Question)> {
    let mut out = Vec::new();
    for slot in slots {
        match &slot.kind {
            Kind::Threat(..) => out.extend(super::threats::gate_questions(slot)),
            _ => {
                if !slot.open() {
                    continue;
                }
                let base = slot.base();
                let standing = &slot.states[base].words;
                if !slot.idle {
                    out.push((
                        format!("{}.change", slot.name),
                        Question::noul(json!(format!(
                            "Given `actors.{}`, `economy`, `enemy` and the player's `instructions`: should {} do something other than what it does now ({standing})? Yes when the instructions call for a different job now, when what it does is finished or pointless, or when something near it needs answering. No when its course is what the instructions want and nothing has changed.",
                            slot.name, slot.name
                        ))),
                    ));
                }
                for (i, s) in slot.states.iter().enumerate() {
                    if i != base && i != 0 && !s.pair_only {
                        // The bulk kinds (a builder's states, a group's walks and advances: 175k of player-28's 215k
                        // gate questions) carry the actor and the move alone under the state's `asking` line
                        // (ASKING); the rest keep the full form (offline, 24 calls of 6,407 nouls: the short form
                        // moved a noul 0.032 on average against 0.017 re-asked, flipped 56 flags against 36, and cut
                        // the gate's tokens by a third; the flips sat on the factory and change nouls, kept whole).
                        let bulk = matches!(slot.kind, Kind::Builder(_)) || s.id.contains(".walk_") || s.id.contains(".advance_");
                        let text = if bulk {
                            format!("{}, rather than {standing}: {}.", slot.name, s.words)
                        } else {
                            format!("Given `actors.{}`, `economy`, `ours` and the player's `instructions`: is this what {} should do now, rather than {standing}? The move: {}.", slot.name, slot.name, s.words)
                        };
                        out.push((s.id.clone(), Question::noul(json!(text))));
                        // A builder's state asked again as a fact about the instructions (K-jev-a-response-opens-by-
                        // the-party-noul-not-its-own: the move noul weighs a state against the builder's course and
                        // rated the packet's strip extractor 0.23-0.32 under the flag in every no-rules game).
                        if told && matches!(slot.kind, Kind::Builder(_)) {
                            out.push((format!("{}.told", s.id), Question::noul(json!(format!("Read the player's `instructions` alone, with `actors.{}` and `ours` for what stands: do the instructions make this {}'s next step now? The step: {}. Yes only when a sentence about {} or about its kind of unit calls for exactly this given what stands now; no when the instructions say never or not to, name a condition that does not hold now, or say nothing about it.", slot.name, slot.name, s.words, slot.name)))));
                        }
                    }
                }
            }
        }
    }
    out
}

/// What the picture looks like for ask-on-change: the threats, and every actor's course by kind and its idleness.
/// A change asks; the same picture does not. The kind, not the state: a constructor moving from one spot to the
/// next by its `job expand` default is on the same course (onepass-smoke-2: 309 asking seconds of 880, most of
/// them a constructor's base state changing spot or its slot coming and going with each build).
pub(super) fn signature(slots: &[Slot]) -> String {
    slots
        .iter()
        .map(|s| match &s.kind {
            Kind::Threat(p, place) => format!("{}@{place}:{}{}{}", p.name, p.ids.len(), if p.killing.is_some() { "!" } else { "" }, s.states.iter().filter(|st| st.current).map(|st| format!("[{}]", st.id)).collect::<String>()),
            _ => format!("{}:{}", s.name, course(s)),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// An actor's course as ask-on-change sees it: the kind of what it does, and its idleness.
pub(super) fn course(slot: &Slot) -> String {
    format!("{}{}", if slot.base() == 0 { "keep" } else { slot.states[slot.base()].dim }, if slot.idle { ":idle" } else { "" })
}

/// What has changed since the gate last went, beyond each actor's own course.
pub(super) struct News<'a> {
    pub frame: i32,
    /// The events since the last gate (`schedule.rs`); one that starts with an actor's name is that actor's.
    pub events: &'a BTreeSet<String>,
    /// The packet or a list changed.
    pub orders: bool,
    /// An enemy party came into the threats or left them.
    pub parties: bool,
    /// The store crossed empty or full, energy began or stopped stalling, or the store came to cover an idle lab's
    /// cheapest unit.
    pub store: bool,
}

/// Ask on change, actor by actor (the user's law, 2026-09-30): a busy actor is asked about again only when its own
/// course or idleness changes, an event names it, the orders change, an enemy party appears or leaves (a group off
/// its route's legs), the store crosses empty or full (a builder or a lab), or `RE_ASK` has passed since it was
/// last asked; an idle actor and every threat are asked whenever the gate goes. The rest are closed this second:
/// no question goes out for them and world 1 keeps their course. `asked` holds each actor's course and frame at
/// its last ask. player-29-hard (`docs/studies/2026-09-30-jev-load.md`): the whole-picture signature differed
/// every second past ten actors, every option of every actor was re-asked 58-60 times a minute, and the builders'
/// 258,000 option questions (57% of the gate's characters) bought 194 moves; replayed on its gates the rule sends
/// 14% of the builders' questions, 44% of the labs' and 85% of the groups', 40M gate tokens for 88M, and 138 of
/// the pick's 1,826 moves fall on a second their actor is closed (they come at its next event or re-ask). A group
/// walking a leg of its route is not opened by a party (routes-in-prose §4.4: the per-second re-decision against
/// fall-back, pull-out and gather flipped group_G 13 times in 53 s at E3, player-9 18:32-19:25; a mark or a new
/// shelling place opened 22% of the walkers' seconds in player-10, and a mark the packet names comes with a packet
/// event).
pub(super) fn settle(slots: &mut [Slot], asked: &BTreeMap<String, (String, i32)>, news: &News) {
    for slot in slots.iter_mut() {
        if matches!(slot.kind, Kind::Threat(..)) || slot.idle || news.orders {
            continue;
        }
        let Some((before, frame)) = asked.get(&slot.name) else { continue };
        let named = news.events.iter().any(|e| e.strip_prefix(slot.name.as_str()).is_some_and(|rest| rest.starts_with(' ')));
        let outside = match slot.kind {
            Kind::Group(_) => news.parties && !slot.on_route,
            _ => news.store,
        };
        slot.quiet = *before == course(slot) && news.frame - *frame < RE_ASK && !named && !outside;
    }
}

pub(super) fn resolve(slots: &[Slot], world: &mut World) {
    let sent: Vec<String> = slots.iter().zip(world.iter()).filter(|(s, i)| matches!(s.kind, Kind::Threat(..)) && **i != 0).map(|(s, i)| s.states[*i].actor.clone()).collect();
    for (sj, slot) in slots.iter().enumerate() {
        if !matches!(slot.kind, Kind::Threat(..)) && sent.contains(&slot.name) {
            world[sj] = 0;
        }
    }
}

/// The worlds after the pre-pass: world 1 is every slot's base; a threat the gate says needs answering opens every
/// state against it, ranked by its state noul; an actor the gate says should change (an idle one always) opens its
/// states ranked by their nouls; a dear build of one builder with a flagged neighbour's help makes a pair world; a
/// state whose actor world 1 already sends elsewhere is pruned. The single deviations are taken round-robin over
/// the slots, each slot's best first, so one actor's many states never starve another's (onepass-smoke-1); then the
/// joint worlds (the user's one-decider design, restored 2026-09-29; one-pass note §4): every combination of the
/// best-rated slots' deviations, one per slot, over as many slots as fit under `cap`, a world sending one actor at
/// two threats dropped, ordered by the product of their ranks (player-25: ten groups told to join one body needed
/// ten winning picks of one change each; 33 merges landed against 50 splits). `flags` receives what the gate said.
/// None when nothing opened.
pub(super) fn compose(slots: &[Slot], answers: &BTreeMap<String, Answer>, flags: &mut BTreeMap<String, f64>, cap: usize) -> Option<Vec<World>> {
    let noul = |id: &str| match answers.get(id) {
        Some(Answer::Noul { noul }) => Some(*noul),
        _ => None,
    };
    let mut base: World = slots.iter().map(Slot::base).collect();
    resolve(slots, &mut base);
    // Actors world 1 sends against a threat.
    let taken: BTreeSet<&str> = slots.iter().zip(&base).filter(|(s, b)| matches!(s.kind, Kind::Threat(..)) && **b != 0).map(|(s, b)| s.states[*b].actor.as_str()).collect();
    // Per slot, its deviations best first: (rank, world).
    let mut per_slot: Vec<Vec<(f64, World)>> = Vec::new();
    for (si, slot) in slots.iter().enumerate() {
        let mut mine: Vec<(f64, World)> = Vec::new();
        match &slot.kind {
            Kind::Threat(party, _) => {
                let answer = noul(&format!("{}.answer", party.name));
                if let Some(p) = answer {
                    flags.insert(format!("{}.answer", party.name), p);
                }
                let opened = answer.is_some_and(|a| a >= FLAG);
                for (ti, s) in slot.states.iter().enumerate() {
                    if ti == base[si] {
                        continue;
                    }
                    if ti == 0 {
                        // Dropping a current state the gate no longer wants.
                        if slot.states[base[si]].current && answer.is_some_and(|p| p < FLAG) {
                            let mut w = base.clone();
                            w[si] = 0;
                            mine.push((2.0, w));
                        }
                        continue;
                    }
                    let rated = noul(&s.id);
                    if let Some(p) = rated {
                        flags.insert(s.id.clone(), p);
                    }
                    let rated = rated.unwrap_or(0.0);
                    if !(opened || (answer.is_none() && rated >= FLAG)) {
                        continue;
                    }
                    if taken.contains(s.actor.as_str()) && slot.states[base[si]].actor != s.actor {
                        continue;
                    }
                    let mut w = base.clone();
                    w[si] = ti;
                    resolve(slots, &mut w);
                    mine.push((answer.unwrap_or(1.0) * rated.max(0.01), w));
                }
            }
            _ => {
                let change = if slot.idle { Some(1.0) } else { noul(&format!("{}.change", slot.name)) };
                if let Some(p) = change.filter(|_| !slot.idle) {
                    flags.insert(format!("{}.change", slot.name), p);
                }
                // The told nouls of a builder's states: recorded, and the best of them opens the slot as a change would.
                let told_of = |s: &State| noul(&format!("{}.told", s.id));
                let best_of = |a: Option<f64>, b: Option<f64>| match (a, b) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                let told_best = slot.states.iter().enumerate().filter(|(ti, s)| *ti != base[si] && *ti != 0 && !s.pair_only).filter_map(|(_, s)| told_of(s)).fold(0.0, f64::max);
                let change = change.map(|c| if told_best >= TOLD_BAR { c.max(1.0) } else { c });
                let Some(change) = change.filter(|c| *c >= FLAG) else {
                    per_slot.push(mine);
                    continue;
                };
                if taken.contains(slot.name.as_str()) {
                    per_slot.push(mine);
                    continue;
                }
                // Only a state the gate rated as the move (onepass-smoke-2: sixteen worlds of storage, radar and
                // converter beside the Blitz rated 0.73 put 0.5 on "nobody changes course" and 0.2 on the Blitz; the
                // plant idled fourteen minutes with a full store); a clear call against a rule's default; an idle
                // actor's best state at a low bar when none reaches the flag.
                let bar = FLAG;
                let best = slot.states.iter().enumerate().filter(|(ti, s)| *ti != base[si] && *ti != 0 && !s.pair_only).filter_map(|(_, s)| best_of(noul(&s.id), told_of(s))).fold(0.0, f64::max);
                let bar = if slot.idle && base[si] == 0 && best < bar { IDLE_BAR } else { bar };
                for (ti, s) in slot.states.iter().enumerate() {
                    if ti == base[si] || ti == 0 || s.pair_only {
                        continue;
                    }
                    let rated = noul(&s.id);
                    if let Some(p) = rated {
                        flags.insert(s.id.clone(), p);
                    }
                    let told = told_of(s);
                    if let Some(p) = told {
                        flags.insert(format!("{}.told", s.id), p);
                    }
                    let opened = rated.is_some_and(|r| r >= bar) || told.is_some_and(|t| t >= TOLD_BAR);
                    if !opened {
                        continue;
                    }
                    let rated = best_of(rated, told).unwrap_or(0.0);
                    let rank = change * rated;
                    let mut w = base.clone();
                    w[si] = ti;
                    mine.push((rank, w.clone()));
                    // The pair: a dear build with another builder's hands on it.
                    if let (Kind::Builder(id), Response::Building(_) | Response::BuildingAt(..)) = (&slot.kind, &s.response)
                        && (s.dim == "factory" || dear(&s.words))
                    {
                        for (sj, other) in slots.iter().enumerate() {
                            if sj == si || !matches!(other.kind, Kind::Builder(_)) || taken.contains(other.name.as_str()) {
                                continue;
                            }
                            let other_change = if other.idle { 1.0 } else { noul(&format!("{}.change", other.name)).unwrap_or(0.0) };
                            if other_change < FLAG {
                                continue;
                            }
                            if let Some(hi) = other.states.iter().position(|h| matches!(&h.response, Response::Assist(target) if target == id)) {
                                let mut pw = w.clone();
                                pw[sj] = hi;
                                mine.push((rank * 0.99, pw));
                            }
                        }
                    }
                }
            }
        }
        mine.sort_by(|a, b| b.0.total_cmp(&a.0));
        mine.truncate(DEPTH);
        per_slot.push(mine);
    }
    if per_slot.iter().all(Vec::is_empty) {
        return None;
    }
    // Round-robin: every slot's best, then every slot's second best, ...
    let cap = cap.max(2);
    let mut out = vec![base.clone()];
    let mut round = 0;
    while out.len() < cap {
        let mut any = false;
        for mine in &per_slot {
            if let Some((_, w)) = mine.get(round) {
                any = true;
                if !out.contains(w) {
                    out.push(w.clone());
                    if out.len() >= cap {
                        break;
                    }
                }
            }
        }
        if !any {
            break;
        }
        round += 1;
    }
    // The joint worlds: the slots ranked by their best deviation, as many as the cap has room for the product of.
    let mut ranked: Vec<usize> = (0..per_slot.len()).filter(|si| !per_slot[*si].is_empty()).collect();
    ranked.sort_by(|a, b| per_slot[*b][0].0.total_cmp(&per_slot[*a][0].0));
    let mut active: Vec<usize> = Vec::new();
    for si in ranked {
        let with: Vec<usize> = active.iter().copied().chain([si]).collect();
        let product: usize = with.iter().map(|sj| 1 + per_slot[*sj].len()).product();
        let singles: usize = with.iter().map(|sj| per_slot[*sj].len()).sum();
        if out.len() + product - 1 - singles > cap {
            break;
        }
        active = with;
    }
    if active.len() >= 2 {
        let mut joint: Vec<(f64, World)> = Vec::new();
        let radix: Vec<usize> = active.iter().map(|si| 1 + per_slot[*si].len()).collect();
        let mut digits = vec![0usize; active.len()];
        'combos: loop {
            // The next combination, mixed radix; done when it wraps.
            let mut carry = 0;
            while carry < digits.len() {
                digits[carry] += 1;
                if digits[carry] < radix[carry] {
                    break;
                }
                digits[carry] = 0;
                carry += 1;
            }
            if carry == digits.len() {
                break;
            }
            let chosen: Vec<(usize, usize)> = digits.iter().enumerate().filter(|(_, d)| **d > 0).map(|(k, d)| (active[k], d - 1)).collect();
            if chosen.len() < 2 {
                continue;
            }
            let mut w = base.clone();
            let mut rank = 1.0;
            for (si, di) in &chosen {
                let (r, dev) = &per_slot[*si][*di];
                rank *= r;
                for (k, (a, b)) in dev.iter().zip(&base).enumerate() {
                    if a != b {
                        if w[k] != base[k] && w[k] != *a {
                            continue 'combos; // two deviations set one slot differently
                        }
                        w[k] = *a;
                    }
                }
            }
            // One actor at two threats is no world.
            let mut actors: Vec<&str> = Vec::new();
            for (slot, i) in slots.iter().zip(&w) {
                if matches!(slot.kind, Kind::Threat(..)) && *i != 0 {
                    let actor = slot.states[*i].actor.as_str();
                    if actors.contains(&actor) {
                        continue 'combos;
                    }
                    actors.push(actor);
                }
            }
            resolve(slots, &mut w);
            if !out.contains(&w) && !joint.iter().any(|(_, j)| *j == w) {
                joint.push((rank, w));
            }
        }
        joint.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, w) in joint {
            if out.len() >= cap {
                break;
            }
            out.push(w);
        }
    }
    Some(out)
}

/// A build's words say a cost at or over `PAIR_COST`.
fn dear(words: &str) -> bool {
    words.split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse::<f32>().ok()).any(|n| n >= PAIR_COST) && words.contains("metal")
}

/// A world's line: its moves, then what follows: which threats are met with what metal and odds (turrets counted),
/// which are left to nobody, who stays idle. World 1's line carries every course in force; every other world's
/// says only what it changes from world 1 ("as w1, and ...") and the consequences that differ from world 1's, so
/// the change is the option's own words (K-jev-hold-words-carry-the-cost; onepass-hard-2: the plant's Blitz rated
/// 0.69 lost at 0.36 to a w1 whose line the deviation repeated in full, the change buried at its end; player-25:
/// sixteen lines each carrying the same 500 characters of parties left to nobody, idle actors and route holds).
/// A hunt the world starts whose id is in `forbidden` (the gate's reading of the instructions, `FORBIDDEN`) says so
/// in a sentence of its own at the line's end.
pub(super) fn consequence(world: &World, slots: &[Slot], store: &str, base: Option<&World>, forbidden: &BTreeSet<String>) -> String {
    let (moves, met, unmet, idle, stops) = parts_of(world, slots, base);
    // A deviation's consequences are what it adds to world 1's: a party now met, a party now left, an actor now
    // idle, a stop now reached. What it removes (the actor no longer idle, the party no longer left to nobody) is
    // said by its move (player-27 offline, the 10:44 pick: the Mace's line with world 1's idle list re-listed took
    // 0.48-0.50 against world 1, cut to its move 0.70; and the re-listed lines were what overflowed the call).
    let added = |mine: Vec<String>, theirs: &[String]| mine.into_iter().filter(|m| !theirs.contains(m)).collect::<Vec<_>>();
    let (met, unmet, idle, stops) = match base {
        Some(b) => {
            let (_, bmet, bunmet, bidle, bstops) = parts_of(b, slots, None);
            (added(met, &bmet), added(unmet, &bunmet), added(idle, &bidle), added(stops, &bstops))
        }
        None => (met, unmet, idle, stops),
    };
    // World 1's own words are the cost of changing nothing and no more: the courses in force are in `actors`
    // (K-jev-hold-words-carry-the-cost; onepass-hard-3: a w1 listing three helpers' courses took 0.45-0.49 against
    // the plant's Blitz at 0.28-0.33, rated 0.7 by the pre-pass).
    let mut parts: Vec<String> = Vec::new();
    parts.push(match base {
        None => "Nothing changes: every actor keeps the course its entry under `actors` describes".to_string(),
        Some(_) => format!("As w1, and: {}", moves.join("; ")),
    });
    if !met.is_empty() {
        parts.push(format!("Met: {}", met.join("; ")));
    }
    if !unmet.is_empty() {
        parts.push(format!("Left to nobody: {}", unmet.join("; ")));
    }
    if !idle.is_empty() {
        // The cost on the option's own words (K-jev-hold-words-carry-the-cost; onepass-smoke-3: "Idle: plant" beside
        // a full store lost to the Blitz rated 0.68 at 0.65 against 0.35).
        parts.push(format!("{} idle, doing nothing{}", idle.join(" and "), if store.is_empty() { String::new() } else { format!(", while the metal store reads {store}") }));
    }
    // A group at a reached stop of its route (routes-in-prose §4.2): the sentence that turned the next leg from
    // 2 of 24 picks to 24 of 24 in the offline replay.
    parts.extend(stops);
    if let Some(b) = base {
        for (k, (slot, si)) in slots.iter().zip(world).enumerate() {
            let s = &slot.states[*si];
            if b[k] != *si && forbidden.contains(&s.id) {
                parts.push(format!("The player's instructions forbid this hunt for {}", s.actor));
            }
        }
    }
    format!("{}.", parts.join(". "))
}

/// A world's moves (the states changed from `base`) and consequences: the parties met, the parties left to nobody,
/// the idle actors, the reached stops.
#[allow(clippy::type_complexity)]
fn parts_of(world: &World, slots: &[Slot], base: Option<&World>) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut moves: Vec<String> = Vec::new();
    let (mut met, mut unmet, mut idle): (Vec<String>, Vec<String>, Vec<String>) = (Vec::new(), Vec::new(), Vec::new());
    let mut stops: Vec<String> = Vec::new();
    for (k, (slot, si)) in slots.iter().zip(world).enumerate() {
        let s = &slot.states[*si];
        let unchanged = base.is_some_and(|b| b[k] == *si);
        match &slot.kind {
            Kind::Threat(p, place) => {
                let composition = if p.has_commander { format!("THEIR COMMANDER, whose death wins the game, with {}", p.composition) } else { p.composition.clone() };
                let killing = p.killing.as_ref().map_or(String::new(), |(what, _)| format!(", killing {what}"));
                match &s.response {
                    Response::Leave => unmet.push(format!("{} ({composition}, {place}{}{killing})", p.name, under(p))),
                    Response::Back(..) => {
                        if !unchanged {
                            moves.push(s.words.clone());
                        }
                        unmet.push(format!("{} ({composition}, {place}{}{killing})", p.name, under(p)));
                    }
                    _ => {
                        if !unchanged {
                            moves.push(s.words.clone());
                        }
                        let theirs = p.metal + p.turret_metal;
                        let with = if p.turrets.is_empty() { String::new() } else { format!(" with {} ({:.0} metal)", p.turrets, p.turret_metal) };
                        met.push(format!("{} ({composition}, {:.0} metal, {place}){with} met with {:.0} metal: {}{killing}", p.name, p.metal, s.metal, odds_by_metal(s.metal, theirs)));
                    }
                }
            }
            _ => {
                if *si == 0 {
                    if let Some(cost) = &slot.stop_cost {
                        stops.push(cost.clone());
                    } else if slot.idle {
                        idle.push(slot.name.clone());
                    }
                } else if !unchanged {
                    moves.push(s.words.clone());
                }
            }
        }
    }
    (moves, met, unmet, idle, stops)
}

/// The pick is two questions (the user, 2026-09-29, after the offline replay of player-26: with a median 96
/// worlds a pick, near-duplicate combinations split the mass and "nothing changes" took the plurality 142 of 315
/// picks; two stages built a factory 29 of 92 idle seconds against 9). Stage one: a Choice over the deviations
/// alone, its answer sampled, not taken at the top (the user's ruling: the mass a change holds across the worlds
/// it sits in is its chance of being the candidate). Stage two: a plain Choice between world 1 and the candidate.
const COMMON: &str = "Given `economy`, `ours`, `enemy`, `actors`, `player` and the player's `instructions`, which plan is best this second? Each option is one world: who changes course to do what, with what it costs and gives up, which enemy parties are met and which are left to nobody, who stays idle. An idle factory or builder with metal in the store is a cost, not a course, unless the instructions say to wait. The instructions were written before this picture: where they name a place, a party, a building, a unit or a rule, follow them; where the situation has changed, pick the world they would call for.";

/// Stage one: the deviations alone, keyed by their world numbers.
pub(super) fn stage_one(lines: &[String]) -> Question {
    let instructions = json!(format!("{COMMON} Every option is a change from what stands (w1, not offered here): one or more actors' courses changed, the line saying only those changes and what they alter. Pick the best of the changes; whether to change at all is asked next."));
    Question::Choice { instructions, criteria: lines.iter().enumerate().skip(1).map(|(i, l)| (format!("w{}", i + 1), json!(l))).collect() }
}

/// Stage two: world 1 against the candidate, under its own world number.
pub(super) fn stage_two(lines: &[String], candidate: usize) -> Question {
    let instructions = json!(format!("{COMMON} w1 changes nothing: every actor keeps its course and the idle ones stay idle. The other option is the best change on offer this second, its line saying only what it changes from w1 and what that alters. Pick one."));
    Question::Choice { instructions, criteria: BTreeMap::from([("w1".to_string(), json!(lines[0])), (format!("w{}", candidate + 1), json!(lines[candidate]))]) }
}

/// A world drawn from a Choice's probabilities with a uniform draw in `u` (0 to 1): the index and its probability.
pub(super) fn sample(answers: &BTreeMap<String, Answer>, worlds: usize, u: f64) -> Option<(usize, f64)> {
    let Some(Answer::Choice { probabilities, choice, .. }) = answers.get("worlds.pick") else { return None };
    let index = |k: &str| k.strip_prefix('w').and_then(|n| n.parse::<usize>().ok()).filter(|n| (2..=worlds).contains(n)).map(|n| n - 1);
    let total: f64 = probabilities.iter().filter(|(k, _)| index(k).is_some()).map(|(_, p)| p).sum();
    if total <= 0.0 {
        return index(choice).map(|i| (i, 1.0));
    }
    let mut acc = 0.0;
    let mut last = None;
    for (k, p) in probabilities {
        let Some(i) = index(k) else { continue };
        acc += p / total;
        last = Some((i, *p));
        if u < acc {
            return last;
        }
    }
    last
}

/// A uniform draw in 0 to 1 from the standard library's random hasher seed (no `rand` dependency).
pub(super) fn draw() -> f64 {
    use std::hash::{BuildHasher, Hasher};
    let h = std::collections::hash_map::RandomState::new().build_hasher().finish();
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// The two calls of the pick, made in place: stage one over the deviations (skipped when there is one), its answer
/// sampled into the candidate, then stage two. Each stage's request and answer come back for the log; the final
/// answer is stage two's, or, with one deviation, a stage two over it alone.
pub(super) struct PickRun {
    pub stage_one: Option<(jev::Request, Result<jev::Response, jev::Error>)>,
    pub candidate: Option<(usize, f64)>,
    pub stage_two: Option<(jev::Request, Result<jev::Response, jev::Error>)>,
}

pub(super) fn run_pick(ask: &dyn Fn(&jev::Request) -> Result<jev::Response, jev::Error>, state: &Value, lines: &[String], u: f64) -> PickRun {
    if lines.len() < 2 {
        return PickRun { stage_one: None, candidate: None, stage_two: None };
    }
    let (stage_one, candidate) = if lines.len() == 2 {
        (None, Some((1, 1.0)))
    } else {
        let request = jev::Request { state: state.clone(), questions: BTreeMap::from([("worlds.pick".to_string(), stage_one(lines))]) };
        let result = ask(&request);
        let candidate = result.as_ref().ok().and_then(|r| sample(&r.answers, lines.len(), u));
        (Some((request, result)), candidate)
    };
    let stage_two = candidate.map(|(c, _)| {
        let request = jev::Request { state: state.clone(), questions: BTreeMap::from([("worlds.pick".to_string(), stage_two(lines, c))]) };
        let result = ask(&request);
        (request, result)
    });
    PickRun { stage_one, candidate, stage_two }
}

/// The picked world's index and the pick's confidence.
pub(super) fn pick(answers: &BTreeMap<String, Answer>, worlds: &[World]) -> Option<(usize, f64)> {
    match answers.get("worlds.pick") {
        Some(Answer::Choice { choice, confidence, .. }) => choice.strip_prefix('w').and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=worlds.len()).contains(n)).map(|n| (n - 1, *confidence)),
        _ => None,
    }
}

/// After the gate: the worlds over what it flagged, their lines, and the pick's state, cut to what the worlds name
/// (`pick_state`). Pure, so the realtime worker runs it the instant the gate answers.
pub(super) fn follow_up(slots: &[Slot], answers: &BTreeMap<String, Answer>, store: &str, cap: usize, state: &Value) -> (BTreeMap<String, f64>, Option<(Vec<World>, Vec<String>, Value)>) {
    let mut flags: BTreeMap<String, f64> = BTreeMap::new();
    // The hunts the gate reads as forbidden by the instructions (`FORBIDDEN`): said on their lines.
    let mut forbidden: BTreeSet<String> = BTreeSet::new();
    for s in slots.iter().filter(|slot| matches!(slot.kind, Kind::Threat(..))).flat_map(|slot| &slot.states) {
        let id = format!("{}.forbidden", s.id);
        if let Some(Answer::Noul { noul }) = answers.get(&id) {
            flags.insert(id, *noul);
            if *noul >= FORBIDDEN {
                forbidden.insert(s.id.clone());
            }
        }
    }
    let Some(mut worlds) = compose(slots, answers, &mut flags, cap) else { return (flags, None) };
    let mut lines: Vec<String> = worlds.iter().enumerate().map(|(i, w)| consequence(w, slots, store, (i > 0).then_some(&worlds[0]), &forbidden)).collect();
    let keep = fit(&lines, LINE_CHARS);
    worlds.truncate(keep);
    lines.truncate(keep);
    let state = pick_state(state, slots, &worlds, &lines);
    (flags, Some((worlds, lines, state)))
}

/// How many worlds fit the line budget: world 1 and at least one deviation, then the longest prefix whose
/// deviation lines total `budget` characters or fewer.
pub(super) fn fit(lines: &[String], budget: usize) -> usize {
    let mut total = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        total += line.len();
        if total > budget && i > 1 {
            return i;
        }
    }
    lines.len()
}

/// The pick's state: the gate's, with every actor no world sends anywhere cut to one line and every place the
/// worlds' lines, the instructions and the kept actors do not name dropped. The gate's state carried every open
/// actor and every place near one (onepass-player-8: 25 KB a pick, `places` 10 KB of it, 238 ms a call); the pick
/// judges the lines, which say what each world does and to whom.
pub(super) fn pick_state(state: &Value, slots: &[Slot], worlds: &[World], lines: &[String]) -> Value {
    let mut state = state.clone();
    let sent: BTreeSet<&str> = worlds.iter().flat_map(|w| slots.iter().zip(w).filter(|(_, i)| **i != 0).map(|(s, i)| s.states[*i].actor.as_str())).collect();
    let mut text = lines.join("\n");
    text.push_str(state["instructions"].as_str().unwrap_or_default());
    if let Some(actors) = state["actors"].as_object_mut() {
        for (name, entry) in actors.iter_mut() {
            if sent.contains(name.as_str()) || super::diet::names(&text, name) {
                text.push_str(&entry.to_string());
            } else {
                super::diet::brief(entry);
            }
        }
    }
    if let Some(places) = state["places"].as_object_mut() {
        places.retain(|name, _| super::diet::names(&text, name));
    }
    state
}

/// The slots for the log.
pub(super) fn log_slots(slots: &[Slot]) -> Value {
    json!(slots
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "kind": match &s.kind { Kind::Threat(_, place) => format!("threat {place}"), Kind::Builder(_) => "builder".to_string(), Kind::Lab(_) => "lab".to_string(), Kind::Group(_) => "group".to_string() },
                "base": s.base(),
                "idle": s.idle,
                "states": s.states.iter().map(|st| json!({ "id": st.id, "actor": st.actor, "words": st.words, "dim": st.dim, "metal": st.metal, "current": st.current, "pair_only": st.pair_only })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn party(name: &str, n: usize) -> Party {
        Party { name: name.to_string(), ids: (0..n).map(|i| UnitId(i as i32)).collect(), at: Vec3::default(), metal: 100.0, composition: format!("{n} Ticks"), has_commander: false, killing: None, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() }
    }

    fn state(id: &str, actor: &str, response: Response, dim: &'static str, current: bool) -> State {
        State { id: id.to_string(), actor: actor.to_string(), response, words: id.to_string(), metal: 200.0, dim, current, pair_only: false }
    }

    fn threat(name: &str, states: Vec<State>) -> Slot {
        Slot { name: name.to_string(), kind: Kind::Threat(party(name, 1), "at spot_1".into()), states, queue_ahead: false, idle: false, stop_cost: None, on_route: false, quiet: false }
    }

    fn builder(name: &str, id: i32, states: Vec<State>) -> Slot {
        Slot { name: name.to_string(), kind: Kind::Builder(UnitId(id)), states, queue_ahead: false, idle: true, stop_cost: None, on_route: false, quiet: false }
    }

    #[test]
    fn world_one_is_what_stands_and_flags_make_the_deviations() {
        let slots = vec![
            threat("party_1", vec![state("party_1.leave", "", Response::Leave, "threat", false), state("party_1.hunt_group_A", "group_A", Response::Hunt(vec![UnitId(9)]), "threat", false), state("party_1.whole_group_A", "group_A", Response::Whole, "threat", false)]),
            threat("party_2", vec![state("party_2.leave", "", Response::Leave, "threat", false), state("party_2.whole_group_B", "group_B", Response::Whole, "threat", false), state("party_2.whole_group_A", "group_A", Response::Whole, "threat", false)]),
            builder("constructor_3", 3, vec![state("constructor_3.keep", "constructor_3", Response::Keep, "threat", false), state("constructor_3.extractor_spot_4", "constructor_3", Response::Extractor(4), "extractor", false), state("constructor_3.armsolar", "constructor_3", Response::Building(UnitDefId(1)), "energy", false)]),
        ];
        let answers = BTreeMap::from([
            ("party_1.answer".to_string(), Answer::Noul { noul: 0.2 }),
            ("party_2.answer".to_string(), Answer::Noul { noul: 0.9 }),
            ("party_2.whole_group_B".to_string(), Answer::Noul { noul: 0.7 }),
            ("party_2.whole_group_A".to_string(), Answer::Noul { noul: 0.4 }),
            ("constructor_3.extractor_spot_4".to_string(), Answer::Noul { noul: 0.9 }),
            ("constructor_3.armsolar".to_string(), Answer::Noul { noul: 0.2 }),
        ]);
        let mut flags = BTreeMap::new();
        let worlds = compose(&slots, &answers, &mut flags, 3).unwrap();
        assert_eq!(worlds[0], vec![0, 0, 0]);
        // The deviations round-robin: party_2 met by group_B (0.9*0.7) and the idle constructor's extractor (0.9)
        // before its solar (0.2), which the cap of 3 leaves out.
        assert!(worlds.contains(&vec![0, 1, 0]));
        assert!(worlds.contains(&vec![0, 0, 1]));
        assert!(!worlds.contains(&vec![0, 2, 0]));
        assert!(!worlds.contains(&vec![0, 0, 2]));
        assert_eq!(worlds.len(), 3);
        assert_eq!(gate_questions(&slots, true).iter().filter(|(id, _)| id.starts_with("constructor_3")).count(), 4, "an idle actor is not asked whether to change, only which move: two states, each as the move and as a fact about the instructions");
        assert_eq!(flags["party_2.answer"], 0.9);
        let line = consequence(&vec![1, 1, 0], &slots, "340 of 500 stored", Some(&worlds[0]), &BTreeSet::new());
        assert!(line.contains("Met: party_1") && line.contains("party_2 (") && !line.contains("idle"), "a deviation's line says only what differs from world 1: {line}");
        assert!(consequence(&worlds[0], &slots, "340 of 500 stored", None, &BTreeSet::new()).contains("constructor_3 idle"));
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w2".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((1, 0.6)));
    }

    /// The joint worlds (the user's one-decider design, restored 2026-09-29): every combination of the best-rated
    /// slots' deviations under the cap, one actor never sent at two threats, each line saying every change.
    #[test]
    fn the_worlds_join_the_flagged_deviations_under_the_cap() {
        let slots = vec![
            threat("party_1", vec![state("party_1.leave", "", Response::Leave, "threat", false), state("party_1.whole_group_A", "group_A", Response::Whole, "threat", false), state("party_1.whole_group_C", "group_C", Response::Whole, "threat", false)]),
            threat("party_2", vec![state("party_2.leave", "", Response::Leave, "threat", false), state("party_2.whole_group_B", "group_B", Response::Whole, "threat", false), state("party_2.whole_group_A", "group_A", Response::Whole, "threat", false)]),
            builder("constructor_3", 3, vec![state("constructor_3.keep", "constructor_3", Response::Keep, "threat", false), state("constructor_3.extractor_spot_4", "constructor_3", Response::Extractor(4), "extractor", false)]),
        ];
        let answers = BTreeMap::from([
            ("party_1.answer".to_string(), Answer::Noul { noul: 0.9 }),
            ("party_1.whole_group_A".to_string(), Answer::Noul { noul: 0.8 }),
            ("party_1.whole_group_C".to_string(), Answer::Noul { noul: 0.6 }),
            ("party_2.answer".to_string(), Answer::Noul { noul: 0.9 }),
            ("party_2.whole_group_B".to_string(), Answer::Noul { noul: 0.7 }),
            ("party_2.whole_group_A".to_string(), Answer::Noul { noul: 0.5 }),
            ("constructor_3.extractor_spot_4".to_string(), Answer::Noul { noul: 0.9 }),
        ]);
        let mut flags = BTreeMap::new();
        let worlds = compose(&slots, &answers, &mut flags, 255).unwrap();
        // The singles, then the product: both threats answered and the extractor built in one world.
        assert!(worlds.contains(&vec![1, 1, 1]), "{worlds:?}");
        assert!(worlds.contains(&vec![2, 1, 1]) && worlds.contains(&vec![1, 1, 0]) && worlds.contains(&vec![0, 1, 1]), "{worlds:?}");
        // group_A at both parties is no world; nor group_C at party_1 beside group_A at party_2? (that one stands).
        assert!(!worlds.contains(&vec![1, 2, 0]) && !worlds.contains(&vec![1, 2, 1]), "{worlds:?}");
        assert!(worlds.contains(&vec![2, 2, 0]), "{worlds:?}");
        // 1 base + 5 singles + the product's joint worlds (3*3*2 = 18 combinations, less the base, the 5 singles and
        // the two with group_A at both parties): 10 joint.
        assert_eq!(worlds.len(), 1 + 5 + 10, "{worlds:?}");
        let joint = consequence(&vec![1, 1, 1], &slots, "340 of 500 stored", Some(&worlds[0]), &BTreeSet::new());
        assert!(joint.starts_with("As w1, and: party_1.whole_group_A; party_2.whole_group_B; constructor_3.extractor_spot_4") && joint.contains("Met: party_1") && joint.contains("party_2 (") && !joint.contains("Left to nobody") && !joint.contains("idle"), "{joint}");
        // A cap of 6 holds the base and the five singles; nothing joint fits.
        let small = compose(&slots, &answers, &mut BTreeMap::new(), 6).unwrap();
        assert_eq!(small.len(), 6);
        assert!(small.iter().all(|w| w.iter().filter(|i| **i != 0).count() <= 1), "{small:?}");
    }

    /// The pick's two stages: stage one offers the deviations alone and its answer is sampled by the mass the
    /// worlds hold; stage two is world 1 against the candidate under its own number, which `pick` reads back.
    #[test]
    fn the_pick_samples_stage_one_and_asks_world_one_against_the_candidate() {
        let lines: Vec<String> = ["Nothing changes", "As w1, and: a", "As w1, and: b", "As w1, and: c"].iter().map(|s| s.to_string()).collect();
        let Question::Choice { criteria, .. } = stage_one(&lines) else { panic!("a choice") };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), vec!["w2", "w3", "w4"]);
        let answers = BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w2".into(), probabilities: BTreeMap::from([("w2".to_string(), 0.5), ("w3".to_string(), 0.3), ("w4".to_string(), 0.2)]), confidence: 0.5 })]);
        assert_eq!(sample(&answers, 4, 0.10), Some((1, 0.5)));
        assert_eq!(sample(&answers, 4, 0.55), Some((2, 0.3)));
        assert_eq!(sample(&answers, 4, 0.95), Some((3, 0.2)));
        // A stray key for world 1 or beyond the worlds is not drawn.
        let stray = BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w1".into(), probabilities: BTreeMap::from([("w1".to_string(), 0.9), ("w9".to_string(), 0.05), ("w3".to_string(), 0.05)]), confidence: 0.9 })]);
        assert_eq!(sample(&stray, 4, 0.5), Some((2, 0.05)));
        let Question::Choice { criteria, .. } = stage_two(&lines, 2) else { panic!("a choice") };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), vec!["w1", "w3"]);
        let worlds: Vec<World> = vec![vec![0], vec![1], vec![2], vec![3]];
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w3".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((2, 0.6)));
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w1".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((0, 0.6)));
        let u = draw();
        assert!((0.0..1.0).contains(&u));
    }

    /// The forbidden mark: a hunt is asked about a second time as a reading of the instructions, and one the gate
    /// rates at the bar or over says so on its line; under the bar, and for the group's whole attack, nothing is said.
    #[test]
    fn a_hunt_the_gate_reads_as_forbidden_says_so_on_its_line() {
        let slots = vec![threat("party_1", vec![state("party_1.leave", "", Response::Leave, "threat", false), state("party_1.hunt_group_A", "group_A", Response::Hunt(vec![UnitId(9)]), "threat", false), state("party_1.whole_group_A", "group_A", Response::Whole, "threat", false)])];
        let qs: BTreeMap<String, Question> = gate_questions(&slots, false).into_iter().collect();
        let Question::Noul { instructions, .. } = &qs["party_1.hunt_group_A.forbidden"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().starts_with("Read the player's `instructions` alone: do they forbid this move"));
        assert!(!qs.contains_key("party_1.whole_group_A.forbidden"), "hunts only");
        let gate = |forbidden: f64| BTreeMap::from([("party_1.answer".to_string(), Answer::Noul { noul: 0.8 }), ("party_1.hunt_group_A".to_string(), Answer::Noul { noul: 0.6 }), ("party_1.whole_group_A".to_string(), Answer::Noul { noul: 0.4 }), ("party_1.hunt_group_A.forbidden".to_string(), Answer::Noul { noul: forbidden })]);
        let hunt_line = |forbidden: f64| {
            let (flags, next) = follow_up(&slots, &gate(forbidden), "", CAP, &json!({}));
            assert_eq!(flags["party_1.hunt_group_A.forbidden"], forbidden);
            let (worlds, lines, _) = next.expect("the threat opened");
            (lines[worlds.iter().position(|w| w[0] == 1).expect("the hunt's world")].clone(), lines[worlds.iter().position(|w| w[0] == 2).expect("the whole group's world")].clone())
        };
        let (hunt, whole) = hunt_line(0.84);
        assert!(hunt.ends_with("The player's instructions forbid this hunt for group_A."), "{hunt}");
        assert!(!whole.contains("forbid"), "{whole}");
        assert!(!hunt_line(0.6).0.contains("forbid"), "under the bar");
    }

    /// The line budget (player-27: 145-255 worlds of 120-240k characters refused for three minutes).
    #[test]
    fn the_worlds_are_cut_to_the_line_budget_keeping_world_one_and_a_deviation() {
        let lines: Vec<String> = (0..10).map(|i| format!("w{i} {}", "x".repeat(100))).collect();
        assert_eq!(fit(&lines, 10_000), 10);
        assert_eq!(fit(&lines, 350), 4, "three deviations of 104 fit 350, the fourth does not");
        assert_eq!(fit(&lines, 10), 2, "the first deviation stays even over the budget");
    }

    /// The bulk kinds (a builder's states, a group's walks and advances) are asked short under the state's
    /// `asking` line; the other kinds keep the full form.
    #[test]
    fn the_bulk_gate_questions_are_short_and_the_rest_keep_the_preamble() {
        let slots = vec![
            builder("constructor_3", 3, vec![state("constructor_3.keep", "constructor_3", Response::Keep, "threat", false), state("constructor_3.extractor_spot_4", "constructor_3", Response::Extractor(4), "extractor", false)]),
            Slot { name: "group_A".into(), kind: Kind::Group("A".into()), states: vec![state("group_A.keep", "group_A", Response::Keep, "threat", false), state("group_A.walk_spot_4", "group_A", Response::Walk { place: "spot_4".into(), fight: false }, "walk", false), state("group_A.join_group_B", "group_A", Response::Join("B".into()), "join", false)], queue_ahead: false, idle: true, stop_cost: None, on_route: false, quiet: false },
        ];
        let qs: BTreeMap<String, Question> = gate_questions(&slots, false).into_iter().collect();
        let text = |id: &str| match &qs[id] { Question::Noul { instructions, .. } => instructions.as_str().unwrap().to_string(), _ => panic!("a noul") };
        assert!(text("constructor_3.extractor_spot_4").starts_with("constructor_3, rather than "), "{}", text("constructor_3.extractor_spot_4"));
        assert!(text("group_A.walk_spot_4").starts_with("group_A, rather than "));
        assert!(text("group_A.join_group_B").starts_with("Given `actors.group_A`"));
        assert!(ASKING.contains("rather than the course named after it"));
    }

    /// Ask on change, actor by actor: a busy actor is closed until its own course changes, an event names it, the
    /// orders change, its kind's outside news comes (a party for a group off its route, the store for a builder) or
    /// the re-ask is due; an idle actor and a threat are never closed.
    #[test]
    fn a_busy_actor_is_asked_again_only_on_its_own_news() {
        let busy = || {
            let mut s = builder("constructor_3", 3, vec![state("constructor_3.keep", "constructor_3", Response::Keep, "threat", false), state("constructor_3.extractor_spot_4", "constructor_3", Response::Extractor(4), "extractor", true), state("constructor_3.extractor_spot_5", "constructor_3", Response::Extractor(5), "extractor", false)]);
            s.idle = false;
            s
        };
        let walking = |on_route: bool| Slot { name: "group_A".into(), kind: Kind::Group("A".into()), states: vec![state("group_A.keep", "group_A", Response::Keep, "threat", false), state("group_A.walk_spot_4", "group_A", Response::Walk { place: "spot_4".into(), fight: false }, "walk", true), state("group_A.walk_spot_5", "group_A", Response::Walk { place: "spot_5".into(), fight: false }, "walk", false)], queue_ahead: false, idle: false, stop_cost: None, on_route, quiet: false };
        let none = BTreeSet::new();
        let calm = |frame: i32| News { frame, events: &none, orders: false, parties: false, store: false };
        let closed = |slot: Slot, asked: &BTreeMap<String, (String, i32)>, news: &News| {
            let mut slots = vec![slot, threat("party_1", vec![state("party_1.leave", "", Response::Leave, "threat", false), state("party_1.hunt_group_A", "group_A", Response::Hunt(vec![UnitId(9)]), "threat", false)])];
            settle(&mut slots, asked, news);
            assert!(!slots[1].quiet, "a threat is never closed");
            slots[0].quiet
        };
        let asked: BTreeMap<String, (String, i32)> = [("constructor_3".to_string(), ("extractor".to_string(), 300)), ("group_A".to_string(), ("walk".to_string(), 300))].into();
        assert!(!closed(busy(), &BTreeMap::new(), &calm(330)), "never asked: asked");
        assert!(closed(busy(), &asked, &calm(330)), "the same course a second later: closed");
        let mut slots = vec![busy()];
        settle(&mut slots, &asked, &calm(330));
        assert!(gate_questions(&slots, false).is_empty(), "no question goes out for a closed actor");
        assert!(!closed(busy(), &asked, &calm(300 + RE_ASK)), "the re-ask is due");
        let elsewhere: BTreeMap<String, (String, i32)> = [("constructor_3".to_string(), ("assist".to_string(), 300))].into();
        assert!(!closed(busy(), &elsewhere, &calm(330)), "its course changed");
        let mut idle = busy();
        idle.idle = true;
        assert!(!closed(idle, &[("constructor_3".to_string(), ("extractor:idle".to_string(), 300))].into(), &calm(330)), "an idle actor is asked whenever the gate goes");
        let hit: BTreeSet<String> = ["constructor_3 hit".to_string()].into();
        assert!(!closed(busy(), &asked, &News { events: &hit, ..calm(330) }), "an event names it");
        let other: BTreeSet<String> = ["constructor_30 hit".to_string(), "group_A hit".to_string()].into();
        assert!(closed(busy(), &asked, &News { events: &other, ..calm(330) }), "another actor's event");
        assert!(!closed(busy(), &asked, &News { orders: true, ..calm(330) }), "the orders changed");
        assert!(!closed(busy(), &asked, &News { store: true, ..calm(330) }) && closed(busy(), &asked, &News { parties: true, ..calm(330) }), "the store opens a builder, a party does not");
        assert!(!closed(walking(false), &asked, &News { parties: true, ..calm(330) }) && closed(walking(false), &asked, &News { store: true, ..calm(330) }), "a party opens a group, the store does not");
        assert!(closed(walking(true), &asked, &News { parties: true, ..calm(330) }) && !closed(walking(true), &asked, &News { events: &other, ..calm(330) }), "a group on a leg of its route: its own event, not a party");
    }

    #[test]
    fn world_one_says_what_holding_at_a_reached_stop_costs() {
        // routes-in-prose 4.2: the sentence that turned the next leg from 2 of 24 picks to 24 of 24 offline.
        let mut group = builder("group_A", 7, vec![state("group_A.keep", "group_A", Response::Keep, "threat", false), state("group_A.walk_spot_46", "group_A", Response::Walk { place: "spot_46".into(), fight: true }, "threat", false)]);
        group.kind = Kind::Group("A".into());
        group.stop_cost = Some("group_A holds at spot_49, a stop of its route it has reached, with the route's next stop spot_46 not ordered".into());
        let slots = vec![group];
        let line = consequence(&vec![0], &slots, "", None, &BTreeSet::new());
        assert!(line.contains("route's next stop spot_46 not ordered") && !line.contains("idle, doing nothing"), "{line}");
        let line = consequence(&vec![1], &slots, "", Some(&vec![0]), &BTreeSet::new());
        assert!(line.starts_with("As w1, and: group_A.walk_spot_46") && !line.contains("not ordered"), "{line}");
        // Closed (`settle`): no question goes out, world 1 keeps its course.
        let mut walking = slots.into_iter().next().unwrap();
        assert!(walking.open());
        walking.quiet = true;
        assert!(!walking.open());
        assert!(gate_questions(&[walking], true).is_empty());
    }

    #[test]
    fn the_picks_state_keeps_what_the_worlds_name_and_briefs_the_rest() {
        let slots = vec![
            threat("party_1", vec![state("party_1.leave", "", Response::Leave, "threat", false), state("party_1.hunt_group_A", "group_A", Response::Hunt(vec![UnitId(9)]), "threat", false)]),
            builder("constructor_3", 3, vec![state("constructor_3.keep", "constructor_3", Response::Keep, "threat", false), state("constructor_3.extractor_spot_4", "constructor_3", Response::Extractor(4), "extractor", false)]),
        ];
        let worlds: Vec<World> = vec![vec![0, 0], vec![1, 0]];
        let lines = vec!["Nothing changes.".to_string(), "As w1, and: 1 Blitz of group_A hunt party_1 at spot_4".to_string()];
        let state = json!({
            "instructions": "group_B stands at south_yard",
            "actors": {
                "group_A": { "units": "8 Blitz", "at": "spot_4", "doing": "holding", "standing": "station spot_4" },
                "constructor_3": { "units": "a constructor", "at": "home", "doing": "idle", "standing": "expand" },
                "group_B": { "units": "2 Stout", "at": "spot_9", "doing": "holding", "standing": "none" }
            },
            "places": { "spot_4": { "what": "free" }, "spot_9": { "what": "free" }, "south_yard": { "what": "a mark" }, "spot_77": { "what": "theirs" } }
        });
        let cut = pick_state(&state, &slots, &worlds, &lines);
        assert!(cut["actors"]["group_A"].is_object(), "an actor a world sends keeps its entry");
        assert!(cut["actors"]["group_B"].is_object(), "an actor the instructions name keeps its entry");
        assert_eq!(cut["actors"]["constructor_3"], json!("a constructor, at home, idle"), "an actor no world touches is one line");
        assert!(cut["places"].get("spot_4").is_some() && cut["places"].get("south_yard").is_some() && cut["places"].get("spot_9").is_some(), "{}", cut["places"]);
        assert!(cut["places"].get("spot_77").is_none(), "a place nothing names is dropped");
    }

    #[test]
    fn a_pair_world_puts_two_builders_on_one_build() {
        let slots = vec![
            builder("commander", 1, vec![state("commander.keep", "commander", Response::Keep, "threat", false), state("commander.armvp", "commander", Response::Building(UnitDefId(2)), "factory", false)]),
            builder("constructor_2", 2, vec![state("constructor_2.keep", "constructor_2", Response::Keep, "threat", false), State { pair_only: true, ..state("constructor_2.help_commander", "constructor_2", Response::Assist(UnitId(1)), "assist", false) }]),
        ];
        let answers = BTreeMap::from([("commander.armvp".to_string(), Answer::Noul { noul: 0.9 })]);
        let mut flags = BTreeMap::new();
        let worlds = compose(&slots, &answers, &mut flags, 8).unwrap();
        assert_eq!(worlds, vec![vec![0, 0], vec![1, 0], vec![1, 1]]);
        // A builder's state is asked twice: as the move, and as a fact about the instructions.
        assert_eq!(gate_questions(&slots, true).iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["commander.armvp", "commander.armvp.told"]);
        assert_eq!(gate_questions(&slots, false).iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["commander.armvp"]);
    }
}
