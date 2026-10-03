//! The menus (`docs/design/2026-10-01-hands-rebuild.md`, H-HANDS-MENU): one menu per actor, one move per actor a
//! second. A move is a verb and what it is aimed at; the verbs are a short fixed list per kind of actor, and
//! everything the picture or the packet names is on the menu of every actor that can use it. Code chooses neither
//! what a move is aimed at nor for whom: a move is left out only when the unit cannot do it, or when it would
//! change nothing (the course in force is `stay`). Enemy parties, places and our own units are what a move is aimed
//! at; none of them has a menu.
//!
//! The words of a move are built from the same parts in the same order: the move; the way (distance and time in
//! word buckets); what is known at its end; how it stands to the party in the actor's entry, with that fight's facts;
//! what it leaves. Clocks live in the actor's entry (`picture.rs`), never in a move, so a move's words stay the same
//! from second to second.

use std::collections::{BTreeMap, HashMap, VecDeque};

use bot_protocol::{EnemyUnit, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::groups::{Group, Ward};
use super::picture::{NEAR, Party, Picture, Place, distance_words, place_of};
use super::{GroupTask, Pianist, Task};
use crate::strategist::shared::Allowance;
use crate::world::Domain;

/// A builder is threatened by a party this close, or by a hit in the last seconds: a listed builder then has its
/// ways out on a menu, and no builder queues a step behind its build.
const THREATENED: f32 = 800.0;
/// A builder whose started build is this far along has its moves ordered behind it (H-HANDS-QUEUE).
const QUEUE_AT: f32 = 0.6;
/// A place this close is where the actor stands: a move there would change nothing.
const HERE: f32 = 300.0;
/// His buildings within this of a place are at it.
pub(super) const HIS_AT: f32 = 500.0;
/// A party this close to a place is said in the words of a move that ends there.
const AT_END: f32 = 800.0;
/// A unit of ours under this share of its health is damaged: a builder can repair it.
const DAMAGED: f32 = 0.7;
/// A wreck field with this much metal is worth a builder's walk (the picture lists the same fields).
const FIELD_METAL: f32 = 100.0;
/// The D-gun's shot costs this much energy.
const DGUN_ENERGY: f32 = 500.0;
/// The sizes a detachment comes in, below the group's own size.
const LADDER: [usize; 4] = [1, 2, 4, 8];
/// A party moving slower than this stands still.
const STANDING: f32 = 0.3;

/// Where a building goes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Site {
    /// An extractor on this metal spot.
    Spot(usize),
    /// At the named place (a defence stands a little toward the enemy from it; a tier-2 extractor goes over ours).
    Place(String),
    /// Where the base layout has it (`place_planned`): beside the builder, in the yard, on the back field.
    Planned,
}

/// What a move is aimed at when it is his.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum His {
    Party(String),
    /// A place where buildings of his are known.
    Place(String),
}

/// A move: the verb and what it is aimed at.
#[derive(Clone, Debug)]
pub(crate) enum Order {
    /// The course in force goes on.
    Stay,
    /// Every soldier stops where it is, and the group stays there until told otherwise.
    Hold,
    /// The group closes up on its front.
    Gather,
    /// To the place, running from everything on the way (a group), or walking there and waiting (a builder).
    Go(String),
    /// To the place, fighting everything on the way and there.
    FightTo(String),
    /// The same order aimed at his buildings known at the place: the picture names the place, not the packet.
    Raid(String),
    /// The whole group, or an armed builder, attacks the party.
    Attack(String),
    /// The long-reach soldiers shell it from their reach, the rest standing between.
    Shell(His),
    /// These soldiers leave as a group of their own and hunt the party (H-MICRO-HUNT).
    Send(Vec<UnitId>, String),
    /// Into the other group, on its course.
    Join(String),
    /// Beside a group or a builder of ours, wherever it goes.
    Follow(Ward),
    /// One soldier leaves to rove (H-MICRO-ROVE).
    Scout,
    Build(UnitDefId, Site),
    /// Guard a factory or another builder: its build power goes to whatever that makes.
    Help(UnitId),
    /// The wrecks of the field at this point.
    TakeApart(Vec3),
    /// One unit of ours, on the player's order (a `reclaim <handle>` list step from the `remove` tool).
    TakeApartUnit(UnitId),
    Repair(UnitId),
    /// The commander's D-gun at the party's nearest unit.
    DGun(String),
    /// A factory's next unit.
    Make(UnitDefId),
}

/// One move of one actor.
#[derive(Clone, Debug)]
pub(crate) struct Move {
    /// The verb and what it is aimed at; the move's id is `<actor>.<key>`.
    pub key: String,
    pub order: Order,
    /// The move as a world's line and a question say it.
    pub words: String,
    /// The move in a few words, for world 1's line and the registers: "the advance to spot_46".
    pub said: String,
    /// The party the move is aimed at: it is asked as an answer to that party, and the party's `answer` opens it.
    pub party: Option<String>,
    /// A detachment: the gate also asks whether the player's instructions forbid it (the forbidden mark).
    pub detachment: bool,
    /// What of the player's instructions bears on the move, for the `fuse` layer's decode (`decode.rs`).
    pub reads: Vec<Read>,
    /// Fused off by the decode: the instructions clearly do not give this actor this move, and it is not asked.
    pub fused: bool,
    /// Its opener said no at the last gate (the `openers` layer): not asked this gate, and asked the second after
    /// the opener opens.
    pub held: bool,
    /// Its place's question said no at the last gate (the `places` layer): not asked this gate, and asked the
    /// second after the place opens.
    pub blanked: bool,
    /// Fused off, held or blanked, and asked anyway for the layer's audit: the answer is logged and not played.
    pub audit: bool,
    /// The forbidden mark as the decode has it for a detachment: on or off without asking each second; `None`
    /// when the decode is unsure or the layer is off, and the gate asks.
    pub marked: Option<bool>,
}

/// A fact about the player's instructions alone that bears on a move: the decode asks it once a packet.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Read {
    /// The instructions name this place for the actor.
    Place(String),
    /// They have the builder build this type (its internal name, and its words).
    Build(String, String),
    /// They tell the group to join this group.
    Join(String),
    /// They tell the actor to follow this group or builder.
    Follow(String),
    /// They forbid the group a detachment of this many.
    Detach(usize),
}

impl Move {
    pub(super) fn new(key: String, order: Order, words: String, said: String, party: Option<String>, detachment: bool) -> Move {
        Move { key, order, words, said, party, detachment, reads: Vec::new(), fused: false, held: false, blanked: false, audit: false, marked: None }
    }

    /// Whether the move's question goes out when its menu's own moves are asked.
    pub(super) fn asked(&self) -> bool {
        !(self.fused || self.held || self.blanked) || self.audit
    }

    /// Whether the move may go to the pick this gate.
    pub(super) fn playable(&self) -> bool {
        !(self.fused || self.held || self.blanked)
    }

    /// The place a move of the actor's own is aimed at (the `places` layer asks about it once a gate): a walk, an
    /// advance, a raid or a build at a named place or a spot; not a move aimed at a party, which its party opens.
    pub(super) fn place(&self) -> Option<String> {
        match &self.order {
            Order::Go(p) | Order::FightTo(p) | Order::Raid(p) | Order::Build(_, Site::Place(p)) | Order::Shell(His::Place(p)) => Some(p.clone()),
            Order::Build(_, Site::Spot(spot)) => Some(format!("spot_{spot}")),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Group(String),
    Builder(UnitId),
    Factory(UnitId),
}

impl Kind {
    pub(super) fn word(&self) -> &'static str {
        match self {
            Kind::Group(_) => "group",
            Kind::Builder(_) => "builder",
            Kind::Factory(_) => "lab",
        }
    }
}

/// An actor's menu this second.
#[derive(Clone, Debug)]
pub(crate) struct Menu {
    pub name: String,
    pub kind: Kind,
    /// Index 0 is `stay`.
    pub moves: Vec<Move>,
    /// The actor has no course: it is asked at every gate.
    pub idle: bool,
    /// A builder's moves are ordered behind the build in progress.
    pub queue_ahead: bool,
    /// The course in force by kind ("fight_to", "build", "idle"), for the news rule and the gate's signature.
    pub course: &'static str,
    /// The course's key as a move would have it ("go_spot_30"), for the registers.
    pub course_key: String,
    /// The course in words, without its clocks: "advances to spot_46, fighting on the way".
    pub course_words: String,
    /// The party the course in force is aimed at.
    pub aimed_at: Option<String>,
    /// Where the actor stands against that party, while the party is in the picture.
    pub against: Option<Standing>,
    /// The party in the actor's entry (the nearest within `NEAR` of it), for the news rule.
    pub near_party: Option<String>,
    /// The fight a group is in, in the words its course and each of its moves carry ("party_3 (2 armpw): for the 4
    /// of its 7 soldiers near it, we outweigh it heavily; ..."): a move's question says them once.
    pub fight: Option<String>,
    pub at: Option<Vec3>,
    /// World 1's words for the actor while it is idle: "group_A holds at spot_49, a stop it has reached".
    pub idle_words: String,
    /// Closed this gate by the `news` layer (`layers.rs`): its `change` and its own moves are not asked and do
    /// not go to the pick. Its moves aimed at a party still are and do: a party is asked about at every gate, and
    /// so are its answers (the rule of 2026-09-30, under which a threat was never closed).
    pub quiet: bool,
    /// Closed, and its own moves asked anyway for the layer's audit: those answers are logged and not played.
    pub audit: bool,
}

impl Menu {
    /// Whether the actor's `change` and its own moves are asked this gate.
    pub(super) fn asks_own(&self) -> bool {
        (!self.quiet || self.audit) && self.moves.len() > 1
    }

    /// Whether any question of the menu goes out this gate.
    pub(super) fn asked(&self) -> bool {
        self.asks_own() || self.moves.iter().any(|m| m.party.is_some())
    }

    /// Whether the pick may change this actor's course by a move of its own this gate.
    pub(super) fn open(&self) -> bool {
        !self.quiet && self.moves.len() > 1
    }

    pub(super) fn id(&self, m: &Move) -> String {
        format!("{}.{}", self.name, m.key)
    }
}

/// What an actor last chose (the registers, law 8): printed in its entry, never acted on.
#[derive(Clone, Debug)]
pub(crate) struct Register {
    pub frame: i32,
    /// The move in a few words, with what it left: "the walk to spot_30, leaving its attack on party_7".
    pub words: String,
    /// The key of the course the pick took the actor from: a move with this key undoes the pick, and says so.
    pub left: String,
}

/// Where an actor whose course is aimed at a party stands against it this second: measured on the ground, never
/// read off its task (H-HANDS-STANDING). player-35 7:53-8:28: nine Blitzes whose task read "attacking party_17"
/// stood with no order 1,000 to 1,500 from the Tick for 35 s while world 1 said "met by group_W" and the party's
/// question "group_W on it"; Jev rated the party's answer 0.31-0.47 and the Tick killed two extractors.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Standing {
    /// Soldiers with a member of the party inside their weapon's reach, of how many.
    pub in_reach: usize,
    pub of: usize,
    /// The nearest soldier's way to the party, in the menus' buckets.
    pub way: String,
    /// Soldiers with no order in the engine.
    pub idle: usize,
    /// Whether the actor's fastest outpaces the party's slowest; none while the party's speed is unknown.
    pub catches: Option<bool>,
}

impl Standing {
    /// Whether anything of the actor has the party in reach: the one case in which the party is "met".
    pub(super) fn met(&self) -> bool {
        self.in_reach > 0
    }

    /// "4 of its 9 soldiers have the party in reach", or what stands in the way of that.
    pub(super) fn words(&self) -> String {
        if self.met() {
            return if self.of == 1 { "it has the party in reach".to_string() } else { format!("{} of its {} soldiers have the party in reach", self.in_reach, self.of) };
        }
        let idle = match (self.idle, self.of) {
            (0, _) => "",
            (_, 1) => ", standing with no order",
            (i, n) if i == n => ", every one of them standing with no order",
            _ => ", some of them standing with no order",
        };
        let catches = match self.catches {
            Some(false) => "; the party outruns it: it reaches the party only where the party stands still",
            Some(true) => "; it is faster than the party",
            None => "",
        };
        let who = if self.of == 1 { "it is".to_string() } else { format!("the nearest of its {} soldiers is", self.of) };
        format!("nothing of it has the party in reach ({who} {} away{idle}){catches}", self.way)
    }
}

/// A builder's standing as the menus and the lists read it.
pub(super) struct BuilderStatus {
    /// The started build: its words and its share done.
    pub started: Option<(String, f32)>,
    pub threatened: bool,
    pub queue_ahead: bool,
}

/// ", under 3 turrets: 1 armhlt, 2 armllt" for a party the enemy's armed buildings cover; empty otherwise.
pub(super) fn under(p: &Party) -> String {
    if p.turrets.is_empty() { String::new() } else { format!(", under {}", p.turrets) }
}

/// A time in a word bucket: a move's words stay the same while the actor walks.
fn seconds_words(seconds: f32) -> String {
    if !seconds.is_finite() {
        "an unknown time".to_string()
    } else if seconds < 8.0 {
        "a few seconds".to_string()
    } else if seconds < 90.0 {
        format!("about {:.0} s", ((seconds / 10.0).round() * 10.0).max(10.0))
    } else {
        format!("about {:.0} min", (seconds / 60.0).round())
    }
}

/// The way to a point: "near, about 20 s of walking".
fn way_words(distance: f32, speed: f32) -> String {
    if speed.is_finite() && speed > 0.0 { format!("{}, {} of walking", distance_words(distance), seconds_words(distance / speed)) } else { distance_words(distance).to_string() }
}

/// How long ago something was seen, in a word bucket.
fn age_words(frames: i32) -> &'static str {
    let seconds = frames / FRAMES_PER_SECOND;
    if seconds < 10 {
        "in sight now"
    } else if seconds < 60 {
        "seen under a minute ago"
    } else if seconds < 300 {
        "seen minutes ago"
    } else {
        "seen long ago"
    }
}

/// A share in tenths, for words that carry a percentage.
fn tenths(share: f32) -> f32 {
    (share * 10.0).round() * 10.0
}

/// A verdict's first clause: "it outweighs us" of "it outweighs us, and it outranges us (...); ...". The odds of a
/// strung-out group are on the part of it near the party, and keep saying so: "for the 3 of its 8 soldiers near
/// it, it outweighs us".
fn verdict(odds: &str) -> String {
    let first = |text: &str| text.split([';', ',', ':']).next().unwrap_or(text).trim().to_string();
    match odds.split_once(" soldiers near it, ") {
        Some((part, rest)) if odds.starts_with("for the ") => format!("{part} soldiers near it, {}", first(rest)),
        _ => first(odds),
    }
}

fn ordinal(n: usize) -> String {
    match n % 10 {
        1 if n % 100 != 11 => format!("{n}st"),
        2 if n % 100 != 12 => format!("{n}nd"),
        3 if n % 100 != 13 => format!("{n}rd"),
        _ => format!("{n}th"),
    }
}

/// The places a move can be aimed at: every mark the player made, every spot, passage or `home` the packet's text
/// names, every spot a list step names. Code recognises names; what the packet means by them is Jev's reading.
pub(super) fn named_places<'a>(places: &'a [Place], instructions: &str, scripts: &HashMap<String, VecDeque<String>>) -> Vec<&'a Place> {
    let listed: Vec<usize> = scripts.values().flatten().filter_map(|step| step.split_whitespace().nth(1).and_then(|p| p.strip_prefix("spot_")).and_then(|n| n.parse::<usize>().ok())).collect();
    places
        .iter()
        .filter(|p| !p.name.starts_with("shelling"))
        .filter(|p| match p.spot {
            Some(i) => super::diet::names_spot(instructions, i) || listed.contains(&i),
            None if p.name == "home" || p.name.starts_with("passage_") => super::diet::names(instructions, &p.name),
            None => true,
        })
        .collect()
}

/// What the menus of one second share.
struct Scene<'a> {
    tick: &'a Tick,
    kit: &'a Kit,
    picture: &'a Picture,
    pianist: &'a Pianist,
    own: &'a [OwnUnit],
    enemies: &'a [EnemyUnit],
    frame: i32,
    /// The places a move can be aimed at: every mark, every spot, passage or `home` the packet names, every spot a
    /// list step names.
    named: Vec<&'a Place>,
    /// The places where buildings of his are known, with what stands there in words.
    his: Vec<(&'a Place, String)>,
    /// Each party as a move's words say it, and its slowest identified member's speed.
    parties: Vec<(&'a Party, String, Option<f32>)>,
    under_fire: Vec<UnitId>,
    builders: Vec<&'a OwnUnit>,
    /// Every group that is not roving: its name, where its body stands, its soldiers and its course in words.
    groups: Vec<(&'a Group, Vec3, Vec<&'a OwnUnit>, String)>,
}

impl Brain {
    /// The started build, the threat beside the builder, and whether its next moves queue behind the build.
    pub(super) fn builder_status(&self, pianist: &Pianist, unit: &OwnUnit, picture: &Picture, under_fire: &[UnitId], own: &[OwnUnit]) -> BuilderStatus {
        let started = match pianist.tasks.get(&unit.id) {
            Some(Task::Build { def, near, started: true, .. }) => {
                let share = own.iter().filter(|u| u.being_built && u.def == *def).map(|u| (u.pos.dist2d(*near), u.health / u.max_health.max(1.0))).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(0.0, |(_, share)| share);
                Some((self.short_words(*def), share))
            }
            _ => None,
        };
        let threatened = under_fire.contains(&unit.id) || picture.parties.iter().any(|p| p.at.dist2d(unit.pos) < THREATENED);
        let queue_ahead = started.as_ref().is_some_and(|(_, share)| *share >= QUEUE_AT) && !threatened && !pianist.queued.contains_key(&unit.id);
        BuilderStatus { started, threatened, queue_ahead }
    }

    /// The free spots this builder could take, soonest by its own walking first: not held, not another builder's
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
    /// jammer or sonar, a tier-2 extractor over a spot. A list step for one needs its place.
    pub(super) fn placed_at_place(&self, def: UnitDefId) -> bool {
        let Some(d) = self.world.def(def) else { return false };
        let flagged = super::glossary::entry(&d.name).is_some_and(|e| e.has_flag("radar_jammer") || e.has_flag("sonar"));
        d.speed == 0.0 && d.build_speed == 0.0 && d.build_options.is_empty() && (d.weapon_count > 0 || d.radar_range > 0.0 || d.extracts_metal > 0.0 || flagged)
    }

    /// Our count of a kind standing and being built, for the words ("our 3rd extractor").
    fn count_of(&self, def: UnitDefId, own: &[OwnUnit], pianist: &Pianist) -> (usize, usize) {
        let standing = own.iter().filter(|u| !u.being_built && u.def == def).count();
        let coming = own.iter().filter(|u| u.being_built && u.def == def).count() + pianist.tasks.values().filter(|t| matches!(t, Task::Build { def: d, started: false, .. } if *d == def)).count();
        (standing, coming)
    }

    /// The consequence of a build in code's words: the cost against the store and the income, what of the kind
    /// stands and is under way, the energy it draws.
    fn build_words(&self, pianist: &Pianist, unit: &OwnUnit, def: UnitDefId, tick: &Tick, own: &[OwnUnit]) -> String {
        let Some(d) = self.world.def(def) else { return String::new() };
        let map = &self.world.hello.map;
        let m = &tick.snapshot.metal;
        let mut parts: Vec<String> = Vec::new();
        let (standing, coming) = self.count_of(def, own, pianist);
        parts.push(format!("our {}{}", ordinal(standing + coming + 1), if coming > 0 { format!(" ({coming} under way already)") } else { String::new() }));
        let store = m.current + (m.income - m.usage).max(0.0) * 30.0;
        parts.push(if d.metal_cost <= m.current {
            format!("{:.0} metal, in the store", d.metal_cost)
        } else if d.metal_cost <= store {
            format!("{:.0} metal, more than is stored: builds as the metal comes in", d.metal_cost)
        } else {
            format!("{:.0} metal, far more than is stored or coming in soon: slow, the store empties", d.metal_cost)
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
            parts.push(if short <= 0.0 { format!("{} to build at {draw:.0} energy a second, which the store covers", seconds_words(seconds)) } else { format!("{} to build at {draw:.0} energy a second; the energy runs out before it is done", seconds_words(seconds)) });
        }
        parts.join("; ")
    }

    /// What the builder gives up: a started frame that decays, the factory it helps.
    fn leaves_words(&self, task: Option<&Task>, started: &Option<(String, f32)>) -> String {
        match (task, started) {
            (_, Some((what, share))) => format!("; leaves the {what} at about {:.0}%, which decays", tenths(*share)),
            (Some(Task::Assist { lab, .. }), _) => format!("; stops helping {}", self.actor_name(*lab)),
            (Some(Task::Build { def, .. }), _) => format!("; drops the {} it was going to build", self.short_words(*def)),
            (Some(Task::Follow { group, .. }), _) => format!("; leaves group_{group}"),
            (Some(Task::Attack { party, .. }), _) => format!("; leaves its attack on {party}"),
            _ => String::new(),
        }
    }

    /// A builder's course without its clocks: what `stay` keeps.
    fn builder_course(&self, task: Option<&Task>, unit: &OwnUnit, places: &[Place]) -> (&'static str, String, String) {
        match task {
            None if unit.idle => ("idle", String::new(), "stands idle, waiting for an order".to_string()),
            None => ("order", String::new(), "finishes an order".to_string()),
            Some(Task::Build { def, near, spot, started, .. }) => {
                let key = match spot {
                    Some(i) => format!("build_{}_spot_{i}", self.name(*def)),
                    None => format!("build_{}", self.name(*def)),
                };
                ("build", key, format!("{} a {} at {}", if *started { "builds" } else { "walks to build" }, self.short_words(*def), self.place_words(places, *near)))
            }
            Some(Task::Assist { lab, .. }) => ("help", format!("help_{}", self.actor_name(*lab)), format!("helps {} build", self.actor_name(*lab))),
            Some(Task::Reclaim { at, .. }) => ("take_apart", String::new(), format!("takes apart wrecks at {}", self.place_words(places, *at))),
            Some(Task::ReclaimUnit { .. }) => ("take_apart", String::new(), "takes apart a unit of ours on the player's order".to_string()),
            Some(Task::Repair { target, .. }) => ("repair", String::new(), format!("repairs our {}", self.known_units.get(target).map_or("unit", |(def, _)| self.name(*def)))),
            Some(Task::Walk { place, .. }) => ("go", format!("go_{place}"), format!("walks to {place}")),
            Some(Task::Follow { group, .. }) => ("follow", format!("follow_group_{group}"), format!("follows group_{group}")),
            Some(Task::Attack { party, .. }) => ("attack", format!("attack_{party}"), format!("attacks {party}")),
        }
    }

    /// Where `units` stand against a party (`Standing`): how many have a member of it inside their weapon's reach,
    /// the nearest one's way to it, how many have no order, and whether their fastest outpaces its slowest.
    pub(super) fn standing(&self, units: &[&OwnUnit], party: &Party, enemies: &[EnemyUnit]) -> Standing {
        let members: Vec<&EnemyUnit> = enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
        let to_party = |u: &OwnUnit| if members.is_empty() { u.pos.dist2d(party.at) } else { members.iter().map(|e| e.pos.dist2d(u.pos)).fold(f32::INFINITY, f32::min) };
        let def = |u: &OwnUnit| self.world.def(u.def);
        let in_reach = units.iter().filter(|u| def(u).is_some_and(|d| d.reach > 0.0 && to_party(u) <= d.reach)).count();
        let nearest = units.iter().copied().min_by(|a, b| to_party(a).total_cmp(&to_party(b)));
        let way = nearest.map_or(String::new(), |u| way_words(to_party(u), def(u).map_or(0.0, |d| d.speed)));
        let their_slowest = members.iter().filter_map(|e| e.def).filter_map(|d| self.world.def(d)).map(|d| d.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let our_fastest = units.iter().filter_map(|u| def(u)).map(|d| d.speed).fold(0.0, f32::max);
        let catches = their_slowest.is_finite().then_some(our_fastest > their_slowest);
        Standing { in_reach, of: units.len(), way, idle: units.iter().filter(|u| u.idle).count(), catches }
    }

    /// A group's course without its clocks: its kind, its key as a move would have it, its words, and the party
    /// it is aimed at.
    pub(super) fn group_course(&self, group: &Group, picture: &Picture) -> (&'static str, String, String, Option<String>) {
        let name = format!("group_{}", group.name);
        let at = picture.state["actors"][&name]["at"].as_str().unwrap_or("where it stands").to_string();
        match &group.task {
            GroupTask::Hold { picked: true, .. } => ("hold", "hold".to_string(), format!("holds at {at}"), None),
            GroupTask::Hold { committed, .. } => ("idle", String::new(), format!("stands at {at} with no order{}", if *committed { ", fighting everything there since it arrived by advancing" } else { "" }), None),
            GroupTask::Move { .. } if group.gathering => ("gather", "gather".to_string(), "gathers on its front".to_string(), None),
            // The key is the shell move's own (`shell_party_N`, `shell_spot_N`), so the menu leaves out the shell the
            // group is already on (player-49 11:20-11:22: an empty key left it on the menu, it was picked each
            // second and the whole group ordered again each time; 18 of group_O's 103 picks).
            GroupTask::Move { place, .. } if group.shelling => {
                let aimed = place.strip_prefix("standoff from ").unwrap_or(place);
                ("shell", format!("shell_{}", aimed.strip_prefix("his buildings at ").unwrap_or(aimed)), format!("shells from a {place}"), None)
            }
            GroupTask::Move { place, fight: true, .. } => ("fight_to", format!("fight_to_{place}"), format!("advances to {place}, fighting on the way"), None),
            GroupTask::Move { place, .. } => ("go", format!("go_{place}"), format!("walks to {place} without stopping to fight"), None),
            GroupTask::Engage { party, .. } => {
                let seen = picture.parties.iter().find(|p| p.ids.iter().any(|id| party.contains(id))).map(|p| p.name.clone());
                let key = seen.as_ref().map_or(String::new(), |p| format!("attack_{p}"));
                ("attack", key, format!("attacks {}", seen.as_deref().unwrap_or("a party now out of sight")), seen)
            }
            GroupTask::Hunt(h) => ("hunt", String::new(), format!("hunts {}", h.party), Some(h.party.clone())),
            GroupTask::Follow { name: ward, .. } => ("follow", format!("follow_{ward}"), format!("follows {ward}"), None),
        }
    }

    /// A party as a move's words say it: what it is, where, what it is doing to us, where it is heading and which
    /// extractor of ours is next on its way; and its slowest identified member's speed (none for radar contacts).
    fn party_as_aimed_at(&self, party: &Party, picture: &Picture, own: &[OwnUnit], enemies: &[EnemyUnit], kit: &Kit) -> (String, Option<f32>) {
        let members: Vec<&EnemyUnit> = enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
        let speed = members.iter().filter_map(|e| e.def).filter_map(|d| self.world.def(d)).map(|d| d.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let speed = speed.is_finite().then_some(speed);
        let vel = members.iter().fold(Vec3::default(), |s, e| Vec3 { x: s.x + e.vel.x, y: 0.0, z: s.z + e.vel.z });
        let pace = vel.x.hypot(vel.z);
        let heading = if pace < STANDING {
            String::new()
        } else {
            let (ux, uz) = (vel.x / pace, vel.z / pace);
            let next = own
                .iter()
                .filter(|u| !u.being_built && kit.is_extractor(u.def))
                .filter_map(|x| {
                    let (dx, dz) = (x.pos.x - party.at.x, x.pos.z - party.at.z);
                    let along = dx * ux + dz * uz;
                    let across = (dx * uz - dz * ux).abs();
                    (along > 200.0 && across < along * 0.6 + 200.0).then_some((x, along))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match next {
                Some((x, _)) => format!(", heading {}: our next extractor on its way is at {}", super::super::shelling::compass(vel), place_of(&picture.places, x.pos).unwrap_or("no named place")),
                None => format!(", heading {}", super::super::shelling::compass(vel)),
            }
        };
        let place = place_of(&picture.places, party.at).map_or("in sight".to_string(), |pl| format!("at {pl}"));
        let composition = if party.has_commander { format!("THEIR COMMANDER, whose death wins the game, with {}", party.composition) } else { party.composition.clone() };
        let harming = party.harming.as_ref().map_or(String::new(), |(what, _)| format!(", {what}"));
        (format!("{} ({composition}{}, {place}{harming}{heading})", party.name, under(party)), speed)
    }

    /// Every actor's menu this second: each group that is not roving, each builder off a list (a listed one only
    /// while it is threatened, and then with its ways out alone), each factory with nothing ordered ahead.
    pub(super) fn menus(&self, tick: &Tick, kit: &Kit, picture: &Picture) -> Vec<Menu> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        let own = tick.snapshot.own_units.as_slice();
        let enemies = tick.snapshot.enemies.as_slice();
        let frame = tick.frame;
        let instructions = picture.state["instructions"].as_str().unwrap_or_default();
        let named = named_places(&picture.places, instructions, &pianist.scripts);
        let his: Vec<(&Place, String)> = picture
            .places
            .iter()
            .filter(|p| p.name != "home" && !p.name.starts_with("shelling"))
            .filter_map(|p| self.his_words(p.at, frame).map(|words| (p, words)))
            .collect();
        let parties: Vec<(&Party, String, Option<f32>)> = picture
            .parties
            .iter()
            .map(|p| {
                let (words, speed) = self.party_as_aimed_at(p, picture, own, enemies, kit);
                (p, words, speed)
            })
            .collect();
        let mut under_fire: Vec<UnitId> = tick.events.iter().filter_map(|e| if let bot_protocol::Event::UnitDamaged { unit, .. } = e { Some(*unit) } else { None }).collect();
        under_fire.extend(pianist.under_fire(frame));
        let builders: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)).collect();
        let groups: Vec<(&Group, Vec3, Vec<&OwnUnit>, String)> = pianist
            .groups
            .iter()
            .filter(|g| !g.roving)
            .filter_map(|g| {
                let units = g.units(own);
                let at = g.body(own, None)?.at;
                Some((g, at, units, self.group_course(g, picture).2))
            })
            .collect();
        let scene = Scene { tick, kit, picture, pianist, own, enemies, frame, named, his, parties, under_fire, builders, groups };
        let mut menus: Vec<Menu> = Vec::new();
        menus.extend(scene.builders.iter().filter_map(|unit| self.builder_menu(&scene, unit)));
        menus.extend(own.iter().filter(|u| !u.being_built && self.world.is_factory_def(u.def)).filter_map(|unit| self.factory_menu(&scene, unit)));
        menus.extend(pianist.groups.iter().filter(|g| !g.roving).filter_map(|group| self.group_menu(&scene, group)));
        menus
    }

    /// His buildings known within `HIS_AT` of a point, in words: what they are, the turrets among them, how long
    /// ago they were seen, and the income his extractors there are. None when nothing of his is known there.
    fn his_words(&self, at: Vec3, frame: i32) -> Option<String> {
        let theirs: Vec<(UnitDefId, i32)> = self.enemy_buildings.values().filter(|(_, pos, _)| pos.dist2d(at) < HIS_AT).map(|(def, _, seen)| (*def, *seen)).collect();
        if theirs.is_empty() {
            return None;
        }
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for (def, _) in &theirs {
            *counts.entry(self.short_words(*def)).or_default() += 1;
        }
        let turrets = theirs.iter().filter(|(def, _)| self.world.def(*def).is_some_and(|d| d.weapon_count > 0 && d.reach > 0.0)).count();
        let guard = if turrets == 0 { "no turret among them".to_string() } else { format!("{turrets} of them armed") };
        let newest = theirs.iter().map(|(_, seen)| *seen).max().unwrap_or(frame);
        let extractors = theirs.iter().filter(|(def, _)| self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0)).count();
        let income = if extractors > 0 { format!("; {extractors} of his extractors' income") } else { String::new() };
        Some(format!("his {} ({guard}; {}{income})", counts.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), age_words(frame - newest)))
    }

    /// What is known at a place a move ends at: what of ours stands there, his buildings (unless the move's own
    /// words are about them), a party at it with the odds for `units` (unless it is `entry`, the party in the
    /// actor's entry, which the move's words stand to already), and for a spot nobody has in sight how long ago it
    /// was looked at.
    fn end_words(&self, scene: &Scene, place: &Place, units: &[&OwnUnit], with_his: bool, entry: Option<&Party>) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(what) = scene.picture.state["places"][&place.name]["what"].as_str().filter(|w| w.starts_with("our ")) {
            parts.push(what.to_string());
        }
        if with_his && let Some((_, words)) = scene.his.iter().find(|(p, _)| p.name == place.name) {
            parts.push(words.clone());
        }
        if let Some(p) = scene.picture.parties.iter().filter(|p| p.at.dist2d(place.at) < AT_END && entry.is_none_or(|e| e.name != p.name)).min_by(|a, b| a.at.dist2d(place.at).total_cmp(&b.at.dist2d(place.at))) {
            parts.push(format!("{} ({}{}) stands at it: {}", p.name, p.composition, under(p), verdict(&self.odds_words(units, p, scene.enemies))));
        }
        if parts.is_empty() && !scene.his.iter().any(|(p, _)| p.name == place.name) && let Some(spot) = place.spot {
            match self.spot_seen(spot) {
                Some(seen) if scene.frame - seen >= 10 * FRAMES_PER_SECOND => parts.push(format!("nothing of his was there when last looked at, {}", age_words(scene.frame - seen))),
                Some(_) => {}
                None => parts.push("never seen: nobody knows what stands there".to_string()),
            }
        }
        if parts.is_empty() { String::new() } else { format!("; at its end: {}", parts.join("; ")) }
    }

    fn group_menu(&self, scene: &Scene, group: &Group) -> Option<Menu> {
        let (own, enemies, picture, pianist) = (scene.own, scene.enemies, scene.picture, scene.pianist);
        let name = format!("group_{}", group.name);
        let units = group.units(own);
        let air = group.domain == Domain::Air;
        // The group as a body (H-HANDS-GROUP-BODY): distances from where the body stands, its front toward its goal
        // or the nearest party any member is near, the odds on the part near it.
        let goal = match &group.task {
            GroupTask::Move { to, .. } => Some(*to),
            GroupTask::Engage { at, .. } | GroupTask::Follow { at, .. } => Some(*at),
            GroupTask::Hunt(h) => Some(h.at),
            GroupTask::Hold { .. } => None,
        };
        let nearest_to = |p: &Party| units.iter().map(|u| u.pos.dist2d(p.at)).fold(f32::INFINITY, f32::min);
        // The party in the group's entry, as the picture has it.
        let near_party: Option<&Party> = picture.parties.iter().map(|p| (nearest_to(p), p)).filter(|(d, _)| *d < NEAR).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
        let body = group.body(own, goal.or(near_party.map(|p| p.at)))?;
        let walker = self.group_walker(group, own);
        let speed = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let metal: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let (course, course_key, course_words, aimed_at) = self.group_course(group, picture);
        let idle = course == "idle";
        let register = pianist.registers.get(&name);
        let undoes = |key: &str| if register.is_some_and(|r| !r.left.is_empty() && r.left == key) { "; this undoes its last pick" } else { "" };
        // The fight it is in, said once and repeated on every move that keeps it or leaves it: the odds for the
        // part of the group in reach, what the group lost in the last seconds, whether the party can follow.
        let fight: Option<(&Party, String)> = near_party.map(|p| {
            let odds = self.group_odds(&body, p, enemies, &scene.tick.snapshot.allies).1;
            let lost: Vec<&(i32, UnitDefId, Option<(UnitId, UnitDefId)>)> = group.losses.iter().filter(|(f, ..)| scene.frame - f <= 30 * FRAMES_PER_SECOND).collect();
            let losses = if lost.is_empty() {
                String::new()
            } else {
                let w = self.loss_words(&lost, metal, Some(p), &picture.parties);
                format!("; this group has lost {}{} in the last 30 s: {}", w.share, w.verdict, w.to_whom)
            };
            let their_fastest = p.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).filter_map(|d| self.world.def(d)).map(|d| d.speed).fold(0.0, f32::max);
            let follow = if p.unarmed || their_fastest <= 0.0 || !speed.is_finite() {
                ""
            } else if their_fastest > speed {
                "; it can follow this group"
            } else {
                "; it cannot keep up with this group"
            };
            (p, format!("{} ({}{}): {}{losses}{follow}", p.name, p.composition, under(p), verdict(&odds)))
        });
        let keeps = fight.as_ref().map_or(String::new(), |(_, words)| format!("; near {words}"));
        // Where it stands against the party its course is aimed at, on the course's own words: a group told to
        // attack is not thereby fighting.
        let against = aimed_at.as_ref().and_then(|name| picture.parties.iter().find(|p| p.name == *name)).map(|p| self.standing(&body.core, p, enemies));
        let against_words = against.as_ref().map_or(String::new(), |s| format!(": {}", s.words()));
        let stands = |to: Vec3| {
            fight.as_ref().map_or(String::new(), |(p, words)| {
                let (now, then) = (body.at.dist2d(p.at), to.dist2d(p.at));
                let how = if then > now + 200.0 { "stepping back from" } else if then < now - 200.0 { "closing on" } else { "staying as near to" };
                format!("; {how} {words}")
            })
        };
        let leaves = match &group.task {
            GroupTask::Hold { picked: true, .. } => "; leaves its hold".to_string(),
            GroupTask::Hold { .. } => String::new(),
            GroupTask::Move { .. } if group.gathering => "; stops gathering".to_string(),
            GroupTask::Move { place, .. } => format!("; abandons its way to {place}"),
            GroupTask::Engage { .. } => format!("; leaves {}", aimed_at.as_deref().unwrap_or("the party it was attacking")),
            GroupTask::Hunt(h) => format!("; leaves its hunt of {}", h.party),
            GroupTask::Follow { name: ward, .. } => format!("; leaves {ward}"),
        };
        let at = picture.state["actors"][&name]["at"].as_str().unwrap_or("where it stands").to_string();
        let mut moves = vec![Move::new("stay".into(), Order::Stay, format!("{name} {course_words}{against_words}{keeps}"), "its course".into(), None, false)];
        let mut push = |key: String, order: Order, words: String, said: String, party: Option<&Party>, detachment: bool| {
            if key != course_key {
                moves.push(Move::new(key, order, words, said, party.map(|p| p.name.clone()), detachment));
            }
        };

        // Hold and gather.
        push("hold".into(), Order::Hold, format!("{name} holds: every soldier stops where it stands, at {at}, and the group stays there until told otherwise{keeps}{leaves}{}", undoes("hold")), format!("the hold at {at}"), None, false);
        if !air && units.len() >= 2 {
            let tail = if body.length < HERE {
                "it stands together already"
            } else if body.length < super::groups::STRUNG_OUT {
                "its tail is a little behind"
            } else if body.length < 2.0 * super::groups::STRUNG_OUT {
                "its tail is strung out behind"
            } else {
                "its tail is far behind"
            };
            push("gather".into(), Order::Gather, format!("{name} gathers: its front stops and the rest close up on it ({tail}), and it holds there{keeps}{leaves}"), "gathering on its front".into(), None, false);
        }

        // Go and fight to: every named place, both ways; a place where his buildings are known is attacked.
        for place in scene.named.iter().copied().filter(|p| p.at.dist2d(body.at) > HERE && self.reachable_for(walker, p.at)) {
            let way = way_words(body.at.dist2d(place.at), speed);
            let his = scene.his.iter().any(|(p, _)| p.name == place.name);
            push(
                format!("go_{}", place.name),
                Order::Go(place.name.clone()),
                format!("{name} walks to {} ({way}) without stopping to fight on the way{}{}{leaves}{}", place.name, self.end_words(scene, place, &body.core, true, near_party), stands(place.at), undoes(&format!("go_{}", place.name))),
                format!("the walk to {}", place.name),
                None,
                false,
            );
            if !his && !air {
                push(
                    format!("fight_to_{}", place.name),
                    Order::FightTo(place.name.clone()),
                    format!("{name} advances to {} ({way}), fighting on the way{}{}{leaves}{}", place.name, self.end_words(scene, place, &body.core, true, near_party), stands(place.at), undoes(&format!("fight_to_{}", place.name))),
                    format!("the advance to {}", place.name),
                    None,
                    false,
                );
            }
        }
        for (place, words) in scene.his.iter().filter(|(p, _)| p.at.dist2d(body.front) > HERE && self.reachable_for(walker, p.at)) {
            push(
                format!("fight_to_{}", place.name),
                Order::Raid(place.name.clone()),
                format!("{name} attacks his buildings at {} ({}), fighting on the way: {words}{}{}{leaves}{}", place.name, way_words(body.front.dist2d(place.at), speed), self.end_words(scene, place, &body.core, false, near_party), stands(place.at), undoes(&format!("fight_to_{}", place.name))),
                format!("the attack on his buildings at {}", place.name),
                None,
                false,
            );
        }
        // The shooter out of sight that is hitting this group: its estimated place is one to advance on.
        if !air
            && let Some(s) = self.shelling_for(&group.members)
            && let Some(sp) = picture.places.iter().find(|p| p.name == format!("shelling_{}", group.name)).or_else(|| picture.places.iter().find(|p| p.name == "shelling"))
            && self.reachable_for(walker, s.at)
        {
            push(
                format!("fight_to_{}", sp.name),
                Order::FightTo(sp.name.clone()),
                format!("{name} closes on the shooter out of sight as one body, toward `{}` ({}, {}): {}{leaves}", sp.name, self.place_words(&picture.places, s.at), way_words(body.front.dist2d(s.at), speed), self.unseen_shooter_words(&units, &s)),
                "the advance on the shooter out of sight".into(),
                None,
                false,
            );
        }

        // Attack, shell, send: every party in the picture.
        let long_reach: Vec<&OwnUnit> = units.iter().copied().filter(|u| self.artillery(u)).collect();
        let arty_words = {
            let kinds: BTreeMap<&str, usize> = long_reach.iter().fold(BTreeMap::new(), |mut m, u| {
                *m.entry(self.name(u.def)).or_default() += 1;
                m
            });
            kinds.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")
        };
        let screen = units.len() - long_reach.len();
        // A detachment's soldiers: the fastest armed members, the nearest of them first.
        let armed: Vec<&OwnUnit> = units.iter().copied().filter(|u| self.world.def(u.def).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0)).collect();
        for (party, party_words, quarry_speed) in &scene.parties {
            if !self.reachable_for(walker, party.at) {
                continue;
            }
            let distance = nearest_to(party);
            let odds = self.group_odds(&body, party, enemies, &scene.tick.snapshot.allies).1;
            let other = if fight.as_ref().is_some_and(|(p, _)| p.name != party.name) { stands(party.at) } else { String::new() };
            let tail = if body.strung_out() { "; its tail is strung out behind its front" } else { "" };
            let key = format!("attack_{}", party.name);
            push(key.clone(), Order::Attack(party.name.clone()), format!("{name} attacks {party_words} with the whole group ({}): {odds}{tail}{other}{leaves}{}", way_words(distance, speed), undoes(&key)), format!("the attack on {}", party.name), Some(party), false);
            if !air && !long_reach.is_empty() {
                push(
                    format!("shell_{}", party.name),
                    Order::Shell(His::Party(party.name.clone())),
                    format!("{name} shells {party_words} ({}) with its {arty_words} from their reach, its other {screen} soldiers standing between as the screen: {}{other}{leaves}", way_words(distance, speed), verdict(&odds)),
                    format!("shelling {}", party.name),
                    Some(party),
                    false,
                );
            }
            if air || matches!(&group.task, GroupTask::Hunt(h) if h.party == party.name) {
                continue;
            }
            let mut fastest = armed.clone();
            fastest.sort_by(|a, b| {
                let pace = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.speed);
                pace(b).total_cmp(&pace(a)).then(a.pos.dist2d(party.at).total_cmp(&b.pos.dist2d(party.at)))
            });
            let failed = pianist.hunts_failed.iter().rev().find(|(p, _, _)| *p == party.name).map(|(_, _, why)| format!("; the last hunt of it ended without a kill: {why}")).unwrap_or_default();
            for n in LADDER.into_iter().filter(|n| *n < units.len() && *n <= fastest.len()) {
                let hunters = &fastest[..n];
                let pace = hunters.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).fold(f32::INFINITY, f32::min);
                let nearest = hunters.iter().map(|u| u.pos.dist2d(party.at)).fold(f32::INFINITY, f32::min);
                let catching = match quarry_speed {
                    None => "; its speed is unknown (radar contacts only)",
                    Some(quarry) if pace > *quarry => "; they can catch it",
                    Some(_) => "; it outruns them: it leaves when they arrive, or stands and dies",
                };
                let kinds: BTreeMap<&str, usize> = hunters.iter().fold(BTreeMap::new(), |mut m, u| {
                    *m.entry(self.name(u.def)).or_default() += 1;
                    m
                });
                let who = kinds.iter().map(|(n, c)| format!("{c} {n}")).collect::<Vec<_>>().join(", ");
                push(
                    format!("send_{n}_{}", party.name),
                    Order::Send(hunters.iter().map(|u| u.id).collect(), party.name.clone()),
                    format!("{who} of {name} (its fastest, the nearest first) leave it as a group of their own and hunt {party_words} ({}): against them alone, {}{catching}; the rest of {name} keeps its course{failed}", way_words(nearest, pace), verdict(&self.odds_words(hunters, party, enemies))),
                    format!("a detachment of {n} at {}", party.name),
                    Some(party),
                    true,
                );
            }
        }
        if !air && !long_reach.is_empty() {
            for (place, words) in scene.his.iter().filter(|(p, _)| self.reachable_for(walker, p.at)) {
                push(
                    format!("shell_{}", place.name),
                    Order::Shell(His::Place(place.name.clone())),
                    format!("{name} shells his buildings at {} ({}) with its {arty_words} from their reach, its other {screen} soldiers standing between as the screen: {words}{}{leaves}", place.name, way_words(body.front.dist2d(place.at), speed), stands(place.at)),
                    format!("shelling his buildings at {}", place.name),
                    None,
                    false,
                );
            }
        }

        // Join and follow: every other group, every builder.
        for (other, other_at, theirs, other_course) in scene.groups.iter().filter(|(g, ..)| g.name != group.name) {
            let way = way_words(body.at.dist2d(*other_at), speed);
            if other.domain == group.domain {
                let both: Vec<&OwnUnit> = units.iter().chain(theirs.iter()).copied().collect();
                push(
                    format!("join_group_{}", other.name),
                    Order::Join(other.name.clone()),
                    format!("{name} ({}) joins group_{} ({}, {way}; it {other_course}): one body of {} soldiers on group_{}'s course{}{leaves}", self.composition_words(&units), other.name, self.composition_words(theirs), both.len(), other.name, stands(*other_at)),
                    format!("joining group_{}", other.name),
                    None,
                    false,
                );
            }
            if !air {
                let key = format!("follow_group_{}", other.name);
                push(
                    key.clone(),
                    Order::Follow(Ward::Group(other.name.clone())),
                    format!("{name} follows group_{} ({}, {way}; it {other_course}): it stays beside it wherever it goes, a group of its own, and fights what comes at it{}{leaves}{}", other.name, self.composition_words(theirs), stands(*other_at), undoes(&key)),
                    format!("following group_{}", other.name),
                    None,
                    false,
                );
            }
        }
        if !air {
            for ward in &scene.builders {
                let ward_name = self.actor_name(ward.id);
                let key = format!("follow_{ward_name}");
                let (_, _, ward_course) = self.builder_course(pianist.tasks.get(&ward.id), ward, &picture.places);
                push(
                    key.clone(),
                    Order::Follow(Ward::Builder(ward.id)),
                    format!("{name} follows {ward_name} ({}; it {ward_course}): it stays beside it wherever it goes and fights what comes at it{}{leaves}{}", way_words(body.front.dist2d(ward.pos), speed), stands(ward.pos), undoes(&key)),
                    format!("following {ward_name}"),
                    None,
                    false,
                );
            }
            // The scout: one soldier leaves to rove (H-MICRO-ROVE), run in code with no orders from the hands.
            if units.len() >= 2
                && let Some(fastest) = units.iter().filter_map(|u| self.world.def(u.def)).max_by(|a, b| a.speed.total_cmp(&b.speed))
            {
                push("scout".into(), Order::Scout, format!("{name} sends its fastest soldier (a {}) to rove on its own: it looks at what we know least, his start box first, kills what it finds unguarded and keeps out of the reach of anything that can shoot it; the rest carry on", self.short_words(fastest.id)), "sending a scout to rove".into(), None, false);
            }
        }
        let reached_here = group.reached.last().is_some_and(|(last, _)| picture.places.iter().any(|p| p.name == *last && p.at.dist2d(body.at) < HERE));
        let idle_words = format!("{name} holds at {at}{}", if reached_here { ", a stop it has reached" } else { "" });
        for m in &mut moves {
            m.reads = self.reads_of(&m.order);
        }
        Some(Menu { name, kind: Kind::Group(group.name.clone()), moves, idle, queue_ahead: false, course, course_key, course_words, aimed_at, against, near_party: near_party.map(|p| p.name.clone()), fight: fight.as_ref().map(|(_, words)| words.clone()), at: Some(body.at), idle_words, quiet: false, audit: false })
    }

    fn builder_menu(&self, scene: &Scene, unit: &OwnUnit) -> Option<Menu> {
        let (own, enemies, picture, pianist, kit, frame) = (scene.own, scene.enemies, scene.picture, scene.pianist, scene.kit, scene.frame);
        let name = self.actor_name(unit.id);
        let task = pianist.tasks.get(&unit.id);
        let status = self.builder_status(pianist, unit, picture, &scene.under_fire, own);
        // On a list (H-HANDS-SCRIPT): the list is the player's order, and the builder has no menu while it runs. A
        // threatened one has its ways out. A builder on its list's last step has an empty list and a step in
        // progress, which is one whose task still stands.
        let listed = pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) || (pianist.list_steps.contains_key(&unit.id) && task.is_some());
        if listed && !status.threatened {
            return None;
        }
        let queue = status.queue_ahead && !listed;
        let (course, course_key, course_words) = self.builder_course(task, unit, &picture.places);
        let idle = course == "idle";
        let aimed_at = match task {
            Some(Task::Attack { party, .. }) => Some(party.clone()),
            _ => None,
        };
        let against = aimed_at.as_ref().and_then(|name| picture.parties.iter().find(|p| p.name == *name)).map(|p| self.standing(&[unit], p, enemies));
        let stay_words = match (&status.started, queue) {
            (Some((what, _)), true) => format!("{name} finishes the {what} and then waits for an order"),
            _ => format!("{name} {course_words}{}", against.as_ref().map_or(String::new(), |s| format!(": {}", s.words()))),
        };
        let leaves = if queue { String::new() } else { self.leaves_words(task, &status.started) };
        let then = if queue { "then " } else { "" };
        let walker = self.walker_of(unit.def);
        let speed = self.world.def(unit.def).map_or(0.0, |d| d.speed);
        let near_party: Option<&Party> = picture.parties.iter().filter(|p| p.at.dist2d(unit.pos) < NEAR).min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos)));
        let fight: Option<(&Party, String)> = near_party.map(|p| (p, format!("{} ({}{}): against it alone, {}", p.name, p.composition, under(p), verdict(&self.odds_words(&[unit], p, enemies)))));
        let stands = |to: Vec3| {
            fight.as_ref().map_or(String::new(), |(p, words)| {
                let (now, after) = (unit.pos.dist2d(p.at), to.dist2d(p.at));
                let how = if after > now + 200.0 { "stepping away from" } else if after < now - 200.0 { "walking toward" } else { "staying as near to" };
                format!("; {how} {words}")
            })
        };
        let register = pianist.registers.get(&name);
        let undoes = |key: &str| if register.is_some_and(|r| !r.left.is_empty() && r.left == key) { "; this undoes its last pick" } else { "" };
        let mut moves = vec![Move::new("stay".into(), Order::Stay, stay_words, "its course".into(), None, false)];
        let mut push = |key: String, order: Order, words: String, said: String, party: Option<&Party>| {
            if key != course_key {
                moves.push(Move::new(key, order, words, said, party.map(|p| p.name.clone()), false));
            }
        };
        let named: Vec<&Place> = scene.named.iter().copied().filter(|p| self.reachable_for(walker, p.at)).collect();
        let what_stands = |place: &Place| picture.state["places"][&place.name]["what"].as_str().map_or(String::new(), |w| format!(", {w}"));
        let near_words = |place: &Place| picture.state["places"][&place.name]["enemies_near"].as_str().map_or(String::new(), |e| format!("; enemies near it: {e}"));

        if !listed {
            // Build: an extractor at every free spot the packet or a list names, and at the free spot the builder
            // reaches soonest (its "beside itself"); every other type the allowance permits where the base layout
            // has it; a type that stands at a place (a defence, a radar, a jammer, a sonar) at every named place
            // too, and any type at every mark the player made. rebuild-smoke-1: with every type at every named place
            // a constructor's menu was 507 moves, 76,000 of the gate's 82,000 questions in five minutes were a
            // build at a place (a storage at spot_73, a factory at spot_45), and a builder's two best moves were
            // often one building at two places.
            let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
            if can(kit.extractor) {
                let spots = self.free_spots(unit, pianist, picture, own, frame, kit);
                let (standing, coming) = self.count_of(kit.extractor, own, pianist);
                for (k, (i, seconds)) in spots.iter().enumerate() {
                    let is_named = named.iter().any(|p| p.spot == Some(*i));
                    if !is_named && k != 0 {
                        continue;
                    }
                    let place = &picture.state["places"][format!("spot_{i}")];
                    push(
                        format!("build_{}_spot_{i}", self.name(kit.extractor)),
                        Order::Build(kit.extractor, Site::Spot(*i)),
                        format!(
                            "{then}{name} builds a metal extractor at spot_{i} ({} of walking, {}{}{}): our {}{}{leaves}",
                            seconds_words(*seconds),
                            place["grid"].as_str().unwrap_or_default(),
                            if k == 0 { ", the free spot it reaches soonest" } else { "" },
                            place["enemies_near"].as_str().map_or(String::new(), |e| format!(", enemies near: {e}")),
                            ordinal(standing + coming + 1),
                            if coming > 0 { format!(" ({coming} under way already)") } else { String::new() }
                        ),
                        format!("an extractor at spot_{i}"),
                        None,
                    );
                }
            }
            let build_list: Vec<UnitDefId> = self.world.def(unit.def).map(|d| d.build_options.clone()).unwrap_or_default();
            // An allowance with no units is no restriction: `produce` keeps an entry for the group alone after a null
            // list (onepass-player-7). A list naming nothing this builder can build leaves it unrestricted
            // (H-HANDS-PRODUCE: the opening's `produce {"all": [plant units]}` was every constructor's allowance
            // too, and no constructor was offered a turret, a solar or a nano turret for seven minutes, player-14).
            let allowed = self.allowed_units(&name).filter(|a| !a.units.is_empty()).filter(|a| a.units.iter().any(|e| self.world.def_named(super::allowance(e).0).is_some_and(|d| build_list.contains(&d))));
            let permits = |list: &[String], b: UnitDefId| {
                let unit_name = self.name(b).to_string();
                let made = pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0);
                (0..list.len()).any(|k| super::allowance(&list[k]).0 == unit_name && super::entry_permits(list, k, made))
            };
            let usual = super::super::roster::usual_menu(self.name(unit.def));
            // An allowance whose every count is used up permits nothing, not everything (models-medium-gpt6-astra).
            let offered: Vec<UnitDefId> = match &allowed {
                Some(Allowance { units: list, .. }) => build_list.iter().copied().filter(|b| permits(list, *b)).collect(),
                None => build_list.iter().copied().filter(|b| usual.contains(&self.name(*b))).collect(),
            };
            let reach = self.world.def(unit.def).map_or(100.0, |d| d.build_distance) + 300.0;
            let armed_building = |d: UnitDefId| self.world.def(d).is_some_and(|x| x.speed == 0.0 && x.weapon_count > 0);
            for def in offered.iter().copied().filter(|d| *d != kit.extractor) {
                let Some(d) = self.world.def(def) else { continue };
                let on_water = super::glossary::entry(&d.name).is_some_and(|e| e.has_flag("on_water"));
                let nano = d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty();
                if nano && !own.iter().any(|u| self.world.is_factory_def(u.def) && !u.being_built) {
                    continue;
                }
                let key = self.name(def).to_string();
                let costs = self.build_words(pianist, unit, def, scene.tick, own);
                if d.extracts_metal > 0.0 {
                    // A tier-2 extractor goes over an extractor of ours: at every named spot that holds one.
                    let radius = self.spot_occupied_radius();
                    for place in named.iter().filter(|p| p.spot.is_some()) {
                        let ours = own.iter().any(|u| kit.is_extractor(u.def) && u.def != def && !u.being_built && u.pos.dist2d(place.at) < radius);
                        if ours {
                            push(
                                format!("build_{key}_{}", place.name),
                                Order::Build(def, Site::Place(place.name.clone())),
                                format!("{then}{name} builds a {} over our extractor at {} ({}{}): {costs}{leaves}", self.short_words(def), place.name, way_words(unit.pos.dist2d(place.at), speed), near_words(place)),
                                format!("a {} at {}", self.short_words(def), place.name),
                                None,
                            );
                        }
                    }
                    continue;
                }
                // Where the base layout has it: said, since "beside itself" is not where a factory or a generator
                // far from home goes.
                let plan = self.place_planned(def, unit, own, kit);
                let anchor = match &plan {
                    super::super::economy::Plan::Extractor(at) | super::super::economy::Plan::Near(_, at) | super::super::economy::Plan::Beside(_, at) => *at,
                };
                if !on_water || self.world.water_within(unit.pos, reach) {
                    let where_ = if anchor.dist2d(unit.pos) < HERE { "beside itself".to_string() } else { format!("at {} ({})", self.place_words(&picture.places, anchor), way_words(unit.pos.dist2d(anchor), speed)) };
                    // A factory frame of this type already started nearby is helped up, not started again
                    // (H-HANDS-STARTED across builders): the words say what the order will do.
                    let frame_near = self.world.is_factory_def(def) && own.iter().any(|u| u.def == def && u.being_built && u.pos.dist2d(unit.pos) < super::execute::FRAME_HELP);
                    let helped = if frame_near { "; a frame of that type is already started near it: it helps that one up, not a second" } else { "" };
                    push(format!("build_{key}"), Order::Build(def, Site::Planned), format!("{then}{name} builds a {} {where_}: {costs}{helped}{leaves}", self.unit_words(def)), format!("a {} {where_}", self.short_words(def)), None);
                }
                if on_water {
                    continue;
                }
                // A place where the layout already puts it is the same site, not a second move.
                let stands_at_a_place = self.placed_at_place(def);
                let mark = |p: &Place| p.spot.is_none() && p.name != "home" && !p.name.starts_with("passage_");
                for place in named.iter().filter(|p| p.at.dist2d(unit.pos) > HERE && p.at.dist2d(anchor) > HERE && (stands_at_a_place || mark(p))) {
                    // A defence's cover is words: what armed building of ours stands or is ordered there already.
                    let cover = if armed_building(def) {
                        let covering = own.iter().filter(|t| armed_building(t.def) && t.pos.dist2d(place.at) < 350.0).count() + pianist.tasks.values().chain(pianist.queued.values()).filter(|t| matches!(t, Task::Build { def: td, near, .. } if armed_building(*td) && near.dist2d(place.at) < 350.0)).count();
                        if covering == 0 { "; no turret of ours covers it; it stands a little toward the enemy from the place".to_string() } else { format!("; {covering} armed building(s) of ours cover it already") }
                    } else {
                        String::new()
                    };
                    push(
                        format!("build_{key}_{}", place.name),
                        Order::Build(def, Site::Place(place.name.clone())),
                        format!("{then}{name} builds a {} at {} ({}{}){cover}{}: {costs}{}{leaves}", self.short_words(def), place.name, way_words(unit.pos.dist2d(place.at), speed), what_stands(place), near_words(place), stands(place.at)),
                        format!("a {} at {}", self.short_words(def), place.name),
                        None,
                    );
                }
            }
            // Help: every factory, every build another builder has under way.
            let draws = self.production_draws(own, pianist);
            let m = &scene.tick.snapshot.metal;
            for lab in own.iter().filter(|u| self.world.is_factory_def(u.def) && !u.being_built) {
                let lab_name = self.actor_name(lab.id);
                let worth = match draws.iter().find(|(who, _, _)| *who == lab_name).map(|(_, _, metal)| *metal) {
                    Some(draw) if m.current < 100.0 && m.income < draw => "the plant is starved (it would spend more than comes in, and the store is empty): helping adds nothing until metal comes".to_string(),
                    Some(_) if m.current >= m.storage - 1.0 => "the store is full: helping spends it".to_string(),
                    Some(_) => "helping makes its unit sooner while the store lasts".to_string(),
                    // The fact, not an argument (human-11: four newborn constructors went to the lab on "it adds its
                    // build power to whatever the factory makes" against instructions that named their spots).
                    None => match own.iter().find(|u| u.being_built && u.pos.dist2d(lab.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)) {
                        Some(on_pad) => format!("its build power goes to the lab's unit, a {} ({:.0}% built)", self.short_words(on_pad.def), on_pad.health / on_pad.max_health.max(1.0) * 100.0),
                        None => "the lab makes nothing at the moment: its build power would wait there".to_string(),
                    },
                };
                push(format!("help_{lab_name}"), Order::Help(lab.id), format!("{then}{name} helps {lab_name} build ({}): {worth}{}{leaves}", way_words(unit.pos.dist2d(lab.pos), speed), stands(lab.pos)), format!("helping {lab_name}"), None);
            }
            for other in scene.builders.iter().filter(|o| o.id != unit.id) {
                let Some(Task::Build { def, near, started: true, .. }) = pianist.tasks.get(&other.id) else { continue };
                let share = own.iter().filter(|f| f.being_built && f.def == *def).map(|f| (f.pos.dist2d(*near), f.health / f.max_health.max(1.0))).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(0.0, |(_, s)| s);
                let other_name = self.actor_name(other.id);
                push(format!("help_{other_name}"), Order::Help(other.id), format!("{then}{name} helps {other_name} build its {} (about {:.0}% done; {}){}{leaves}", self.short_words(*def), tenths(share), way_words(unit.pos.dist2d(other.pos), speed), stands(other.pos)), format!("helping {other_name}"), None);
            }
            // Take apart: every wreck field the picture lists. Repair: every damaged building or commander of ours.
            for field in self.reclaim.fields.iter().filter(|f| f.metal >= FIELD_METAL).take(super::picture::WRECK_FIELDS) {
                let at = self.place_words(&picture.places, field.at);
                push(format!("take_apart_{}", self.world.grid(field.at)), Order::TakeApart(field.at), format!("{then}{name} takes apart the wrecks at {at} ({}; about {:.0} metal lying there{}){}{leaves}", way_words(unit.pos.dist2d(field.at), speed), (field.metal / 50.0).round() * 50.0, if field.safe { "" } else { "; not safe ground" }, stands(field.at)), format!("taking apart the wrecks at {at}"), None);
            }
            for hurt in own.iter().filter(|u| u.id != unit.id && !u.being_built && u.health < u.max_health * DAMAGED).filter(|u| self.world.is_commander_def(u.def) || self.world.def(u.def).is_some_and(|d| d.speed == 0.0)) {
                let what = format!("our {} at {}", self.name(hurt.def), self.place_words(&picture.places, hurt.pos));
                push(format!("repair_{}", self.handle(hurt)), Order::Repair(hurt.id), format!("{then}{name} repairs {what} (about {:.0}% health; {}){}{leaves}", tenths(hurt.health / hurt.max_health), way_words(unit.pos.dist2d(hurt.pos), speed), stands(hurt.pos)), format!("repairing {what}"), None);
            }
        }

        // The moves that take a builder out of danger, which a listed builder has too: go, follow, attack, D-gun.
        for place in named.iter().filter(|p| p.at.dist2d(unit.pos) > HERE) {
            let key = format!("go_{}", place.name);
            push(key.clone(), Order::Go(place.name.clone()), format!("{name} walks to {} ({}{}) and waits there{}{}{leaves}{}", place.name, way_words(unit.pos.dist2d(place.at), speed), what_stands(place), near_words(place), stands(place.at), undoes(&key)), format!("the walk to {}", place.name), None);
        }
        for (group, at, theirs, group_course) in &scene.groups {
            let key = format!("follow_group_{}", group.name);
            push(key.clone(), Order::Follow(Ward::Group(group.name.clone())), format!("{name} follows group_{} ({}, {}; it {group_course}): it walks beside it wherever it goes{}{leaves}{}", group.name, self.composition_words(theirs), way_words(unit.pos.dist2d(*at), speed), stands(*at), undoes(&key)), format!("following group_{}", group.name), None);
        }
        let armed = self.world.def(unit.def).is_some_and(|d| d.weapon_count > 0);
        let dgun = self.dgun_reach(unit.def);
        let reach = self.world.def(unit.def).map_or(0.0, |d| d.reach);
        for (party, party_words, quarry_speed) in scene.parties.iter().filter(|_| armed) {
            let distance = party.at.dist2d(unit.pos);
            let odds = self.odds_words(&[unit], party, enemies);
            let chase = match quarry_speed {
                None => "; its speed is unknown (radar contacts only)",
                Some(quarry) if *quarry > speed => "; it outruns this unit: the attack drives it off if it stays, and kills it only if it stands and fights",
                Some(_) => "",
            };
            // A skirmisher outranges a commander (onepass-player-3, 22:14: it walked at a Hound that backed off
            // shooting).
            let their_reach = party.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).filter_map(|d| self.world.def(d)).map(|d| d.reach).fold(0.0, f32::max);
            let range = if their_reach > reach { "; it outranges this unit: it is hit on the way in and lands nothing unless the party stands" } else { "" };
            let key = format!("attack_{}", party.name);
            push(key.clone(), Order::Attack(party.name.clone()), format!("{name} attacks {party_words} ({}): against it alone, {odds}{chase}{range}{leaves}{}", way_words(distance, speed), undoes(&key)), format!("the attack on {}", party.name), Some(party));
            // The D-gun reaches what stands inside its reach: beyond it the commander cannot fire it, and walking
            // in is the attack above (rebuild-smoke-1: the shot was picked five times at parties 900 away).
            let inside = party.ids.iter().filter_map(|id| enemies.iter().find(|e| e.id == *id)).filter(|e| e.pos.dist2d(unit.pos) <= dgun).count();
            if dgun > 0.0 && inside > 0 {
                let energy = if scene.tick.snapshot.energy.current >= DGUN_ENERGY { "the energy for it is stored" } else { "the energy store is too low for it now" };
                push(format!("dgun_{}", party.name), Order::DGun(party.name.clone()), format!("{name} D-guns the nearest unit of {party_words}: {inside} of its {} stand inside the D-gun's reach; one shot kills; {energy}{leaves}", party.ids.len()), format!("the D-gun on {}", party.name), Some(party));
            }
        }
        let idle_words = format!("{name} idle, doing nothing");
        for m in &mut moves {
            m.reads = self.reads_of(&m.order);
        }
        Some(Menu { name, kind: Kind::Builder(unit.id), moves, idle, queue_ahead: queue, course, course_key, course_words, aimed_at, against, near_party: near_party.map(|p| p.name.clone()), fight: None, at: Some(unit.pos), idle_words, quiet: false, audit: false })
    }

    /// What of the player's instructions bears on a move (`Read`): the place a walk, an advance or a build is aimed
    /// at, the type a builder builds, the group joined, the ward followed, the size of a detachment. An attack on
    /// his buildings or on a party, and a shooter's estimated place, are the picture's and bear on nothing here.
    fn reads_of(&self, order: &Order) -> Vec<Read> {
        match order {
            Order::Go(place) | Order::FightTo(place) if !place.starts_with("shelling") => vec![Read::Place(place.clone())],
            Order::Build(def, site) => {
                let mut reads = vec![Read::Build(self.name(*def).to_string(), self.short_words(*def))];
                match site {
                    Site::Spot(i) => reads.push(Read::Place(format!("spot_{i}"))),
                    Site::Place(place) => reads.push(Read::Place(place.clone())),
                    Site::Planned => {}
                }
                reads
            }
            Order::Join(other) => vec![Read::Join(format!("group_{other}"))],
            Order::Follow(Ward::Group(other)) => vec![Read::Follow(format!("group_{other}"))],
            Order::Follow(Ward::Builder(id)) => vec![Read::Follow(self.actor_name(*id))],
            Order::Send(soldiers, _) => vec![Read::Detach(soldiers.len())],
            _ => Vec::new(),
        }
    }

    /// A factory's menu: one order ahead (H-HANDS-MENU). A factory with a unit ordered ahead has nothing to decide,
    /// and one with a counted entry left on its `produce` list makes it without asking (`sequence_next`).
    fn factory_menu(&self, scene: &Scene, unit: &OwnUnit) -> Option<Menu> {
        let (own, pianist, kit) = (scene.own, scene.pianist, scene.kit);
        let name = self.actor_name(unit.id);
        if pianist.lab_queue.get(&unit.id).is_some_and(|q| !q.is_empty()) || self.sequence_unit(unit, &name, pianist).is_some() {
            return None;
        }
        let def = self.world.def(unit.def)?;
        let on_pad = own.iter().find(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| self.short_words(u.def));
        let idle = on_pad.is_none() && unit.idle;
        let stay_words = match &on_pad {
            Some(what) => format!("{name} builds the {what} on its pad and then nothing: the next unit waits for an order"),
            None => format!("{name} stands idle, building nothing"),
        };
        let then = if on_pad.is_some() { "then " } else { "" };
        let mut moves = vec![Move::new("stay".into(), Order::Stay, stay_words, "its course".into(), None, false)];
        let allowed = self.allowed_units(&name).filter(|a| !a.units.is_empty());
        let permits = |list: &[String], b: UnitDefId| {
            let unit_name = self.name(b).to_string();
            let made = pianist.produced.get(&(unit.id, unit_name.clone())).copied().unwrap_or(0);
            (0..list.len()).any(|k| super::allowance(&list[k]).0 == unit_name && super::entry_permits(list, k, made))
        };
        let buildables: Vec<UnitDefId> = match &allowed {
            Some(Allowance { units: list, .. }) => def.build_options.iter().copied().filter(|b| permits(list, *b)).collect(),
            None => def.build_options.clone(),
        };
        let extractors = own.iter().filter(|u| !u.being_built && self.world.is_extractor_def(u.def)).count();
        let constructors = own.iter().filter(|u| !u.being_built && self.world.is_constructor_def(u.def)).count();
        let coming = own.iter().filter(|u| u.being_built && self.world.is_constructor_def(u.def)).count();
        let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
        let army_metal: f32 = soldiers.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        for buildable in buildables {
            let (standing, being_made) = self.count_of(buildable, own, pianist);
            let have = if self.world.is_constructor_def(buildable) {
                format!("we have {constructors} constructors{} for {extractors} extractors", if coming > 0 { format!(" and {coming} being made") } else { String::new() })
            } else if self.world.def(buildable).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0) {
                format!("our soldiers: {}", super::picture::soldier_words(soldiers.len(), army_metal))
            } else {
                String::new()
            };
            let cost = self.world.def(buildable).map_or(0.0, |d| d.metal_cost);
            moves.push(Move::new(
                format!("make_{}", self.name(buildable)),
                Order::Make(buildable),
                format!("{then}{name} makes a {} ({cost:.0} metal; we have {standing}{}): {have}", self.unit_words(buildable), if being_made > 0 { format!(" and {being_made} being made") } else { String::new() }),
                format!("a {}", self.short_words(buildable)),
                None,
                false,
            ));
        }
        let idle_words = format!("{name} idle, doing nothing");
        Some(Menu { name, kind: Kind::Factory(unit.id), moves, idle, queue_ahead: on_pad.is_some(), course: if idle { "idle" } else { "make" }, course_key: String::new(), course_words: if idle { "stands idle".to_string() } else { "builds the unit on its pad".to_string() }, aimed_at: None, against: None, near_party: None, fight: None, at: Some(unit.pos), idle_words, quiet: false, audit: false })
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub(crate) fn mv(key: &str, order: Order, party: Option<&str>) -> Move {
        Move::new(key.to_string(), order, key.to_string(), format!("the {key}"), party.map(str::to_string), matches!(key.split('_').next(), Some("send")))
    }

    pub(crate) fn menu(name: &str, kind: Kind, idle: bool, moves: Vec<Move>) -> Menu {
        let mut all = vec![mv("stay", Order::Stay, None)];
        all.extend(moves);
        Menu { name: name.to_string(), kind, moves: all, idle, queue_ahead: false, course: if idle { "idle" } else { "go" }, course_key: String::new(), course_words: "walks".to_string(), aimed_at: None, against: None, near_party: None, fight: None, at: None, idle_words: format!("{name} idle, doing nothing"), quiet: false, audit: false }
    }

    /// The E3 scene (fourteen Blitzes, three Centurions 600 from them, two of his turrets beyond): one menu for the
    /// group with every verb, aimed at everything named. The step back to the place its packet names stands beside
    /// its answers to the party, with the fight's facts on both; a place nobody named is no walk; his buildings are
    /// a place to attack though the packet does not name it; a detachment comes in every size of the ladder below
    /// the group's own.
    #[test]
    fn a_groups_menu_holds_every_verb_aimed_at_everything_named() {
        use super::super::fixtures::{at, e3};
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, enemies, parties) = e3();
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.groups.push(Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::Hold { since: 0, committed: false, picked: false }, 0));
        pianist.groups.push(Group::new("B".into(), Domain::Ground, vec![UnitId(900)], GroupTask::Hold { since: 0, committed: false, picked: true }, 0));
        brain.pianist = Some(pianist);
        let mut own = ours.clone();
        own.push(super::super::fixtures::own(900, 1, at(1000.0, 1000.0)));
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = Tick { frame: 300, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: own, allies: Vec::new(), enemies, wrecks: None } };
        let id = UnitDefId(1);
        let kit = Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, plant: id, vehicle_constructor: id, advanced_lab: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &crate::brain::roster::ROSTERS[0] };
        let place = |name: &str, x: f32, z: f32, spot: Option<usize>| Place { name: name.to_string(), at: at(x, z), spot };
        let picture = Picture {
            rules: String::new(),
            state: serde_json::json!({ "instructions": "group_A (Blitzes): holds its ground; against a party that outweighs it, it walks to spot_30.", "actors": { "group_A": { "at": "spot_40 (E3)" } }, "places": {} }),
            places: vec![place("home", 500.0, 5000.0, None), place("spot_30", 3000.0, 2600.0, Some(30)), place("spot_31", 2000.0, 2600.0, Some(31)), place("spot_47", 4960.0, 1620.0, Some(47))],
            parties,
        };
        let menus = brain.menus(&tick, &kit, &picture);
        let menu = menus.iter().find(|m| m.name == "group_A").expect("the group's menu");
        let keys: Vec<&str> = menu.moves.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(
            keys,
            ["stay", "hold", "gather", "go_spot_30", "fight_to_spot_30", "fight_to_spot_47", "attack_party_12", "send_1_party_12", "send_2_party_12", "send_4_party_12", "send_8_party_12", "join_group_B", "follow_group_B", "scout"],
            "no walk home and none to spot_31, which the packet does not name; his turrets at spot_47 are a place to attack"
        );
        assert!(menu.idle && menu.course == "idle" && menu.near_party.as_deref() == Some("party_12"));
        let words = |key: &str| menu.moves.iter().find(|m| m.key == key).unwrap().words.clone();
        // The fight's facts are on the move that keeps it and on the move that leaves it.
        assert!(words("stay").contains("; near party_12 (3 armwar"), "{}", words("stay"));
        assert!(words("go_spot_30").starts_with("group_A walks to spot_30 (some way off, about ") && words("go_spot_30").contains("; stepping back from party_12 (3 armwar"), "{}", words("go_spot_30"));
        assert!(words("fight_to_spot_47").starts_with("group_A attacks his buildings at spot_47 (") && words("fight_to_spot_47").contains("his 2 Sentry (armllt) (2 of them armed; seen under a minute ago); staying as near to party_12 (3 armwar): we outweigh it"), "{}", words("fight_to_spot_47"));
        let attack = menu.moves.iter().find(|m| m.key == "attack_party_12").unwrap();
        assert_eq!(attack.party.as_deref(), Some("party_12"));
        assert!(attack.words.starts_with("group_A attacks party_12 (3 armwar, in sight) with the whole group ("), "{}", attack.words);
        let send = menu.moves.iter().find(|m| m.key == "send_2_party_12").unwrap();
        assert!(send.detachment && matches!(&send.order, Order::Send(hunters, party) if hunters.len() == 2 && party == "party_12"));
        assert!(send.words.starts_with("2 armflash of group_A (its fastest, the nearest first) leave it as a group of their own and hunt party_12"), "{}", send.words);
        // Where the group stands against the party, measured: the Centurions are 600 off and a Blitz reaches 180.
        let core: Vec<&OwnUnit> = tick.snapshot.own_units.iter().take(14).collect();
        let standing = brain.standing(&core, &picture.parties[0], &tick.snapshot.enemies);
        assert_eq!((standing.in_reach, standing.of, standing.idle, standing.catches), (0, 14, 0, Some(true)), "{standing:?}");
        assert!(standing.words().starts_with("nothing of it has the party in reach (the nearest of its 14 soldiers is near, ") && standing.words().ends_with("; it is faster than the party"), "{}", standing.words());
        let beside = super::super::fixtures::own(901, 1, at(4420.0, 1240.0));
        assert_eq!(brain.standing(&[&beside], &picture.parties[0], &tick.snapshot.enemies).words(), "it has the party in reach");
        // The other group holds by a pick: that is a course, and its menu has no second `hold`.
        let other = menus.iter().find(|m| m.name == "group_B").expect("the other group's menu");
        assert!(!other.idle && other.course == "hold" && !other.moves.iter().any(|m| m.key == "hold" || m.key == "gather" || m.key == "scout"), "{:?}", other.moves.iter().map(|m| &m.key).collect::<Vec<_>>());
        // A move that would take the group back to the course its last pick left says so.
        brain.pianist.as_mut().unwrap().registers.insert("group_A".into(), Register { frame: 240, words: "picked the hold at spot_40, leaving its course (walks to spot_30)".into(), left: "go_spot_30".into() });
        let menus = brain.menus(&tick, &kit, &picture);
        let back = menus[0].moves.iter().find(|m| m.key == "go_spot_30").unwrap();
        assert!(back.words.ends_with("; this undoes its last pick"), "{}", back.words);
        // A shelling group's course carries the shell move's own key, so the menu leaves that move out.
        for (place, key) in [("standoff from party_12", "shell_party_12"), ("standoff from his buildings at spot_47", "shell_spot_47")] {
            let mut shelling = Group::new("S".into(), Domain::Ground, vec![UnitId(900)], GroupTask::Move { to: at(4000.0, 2000.0), place: place.into(), fight: true, since: 0 }, 0);
            shelling.shelling = true;
            let (course, course_key, ..) = brain.group_course(&shelling, &picture);
            assert_eq!((course, course_key.as_str()), ("shell", key));
        }
    }

    #[test]
    fn a_moves_words_carry_buckets_not_clocks() {
        assert_eq!(seconds_words(3.0), "a few seconds");
        assert_eq!(seconds_words(17.0), "about 20 s");
        assert_eq!(seconds_words(24.9), "about 20 s");
        assert_eq!(seconds_words(200.0), "about 3 min");
        assert_eq!(way_words(900.0, 45.0), "near, about 20 s of walking");
        assert_eq!(way_words(900.0, 0.0), "near");
        assert_eq!(age_words(5 * FRAMES_PER_SECOND), "in sight now");
        assert_eq!(age_words(130 * FRAMES_PER_SECOND), "seen minutes ago");
        assert_eq!(verdict("it outweighs us, and it outranges us (460 to our 350); its fastest catch our slowest"), "it outweighs us");
        assert_eq!(verdict("we cannot hit it: nothing in this group shoots at what it is"), "we cannot hit it");
        assert_eq!(verdict("for the 3 of its 8 soldiers near it, it outweighs us; its fastest catch our slowest; the other 5 are 800-1900 behind and not near it yet"), "for the 3 of its 8 soldiers near it, it outweighs us");
        assert_eq!(tenths(0.46), 50.0);
    }
}
