//! The picture: the game as one JSON state for Jev, written by actor and qualitative first (Jev is documented
//! weak at arithmetic and at comparing quantities, and distracted by unrelated state), with the player's
//! instructions at the top. Places and enemy parties are named here so that the menus' answers can name them back.

use std::collections::{BTreeMap, BTreeSet};
use crate::strategist::shared::Allowance;

use bot_protocol::{EnemyUnit, Event, OwnUnit, Tick, UnitDefId, Vec3};
use serde_json::{Value, json};

use super::super::roster::Kit;
use super::glossary;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::groups::Body;
use super::{GroupTask, Pianist, Task};

/// Free spots the picture lists, nearest home on foot first.
const FREE_SPOTS: usize = 10;
/// Enemy extractors known, nearest first.
const ENEMY_SPOTS: usize = 6;
/// How many never-looked and long-unseen spots the enemy evidence lists.
/// Named spots per list in the enemy section: the scout's target is chosen in code (diet decision 1), and every
/// name here kept its place entry in the block (wake-1: 29 never-looked names, 38 to 52 place entries a call).
const NEVER_LOOKED: usize = 3;
/// A spot last in sight longer ago than this is listed as long unseen.
const LONG_UNSEEN: i32 = 5 * 60 * FRAMES_PER_SECOND;
const PASSAGES: usize = 3;
/// Enemies this close together are one party.
/// A unit whose position is this far below the water's surface is under it: seen by sonar only, hit by water weapons only.
const SUBMERGED_Y: f32 = -5.0;
const PARTY_RADIUS: f32 = 400.0;
/// A kept party (H-HANDS-PARTIES-KEPT): a member stays while it stands this near the party's centre, and an enemy
/// without a party joins one whose centre is this near.
const KEPT_RADIUS: f32 = 400.0;
/// Two kept parties whose centres come this near are one.
const KEPT_MERGE: f32 = 250.0;
/// The enemies of other parties this near a party's centre are in a fight with it (H-HANDS-PARTY-BESIDE).
const BESIDE: f32 = 600.0;
/// A turret whose reach ends this short of a party's centre still covers its edge.
const TURRET_MARGIN: f32 = 150.0;
/// A turret whose reach the chase's line enters within this of the group's front is in the `chase` line.
const CHASE_AHEAD: f32 = 600.0;

/// What a chase is running into (`Brain::chase_met`): the words for the group's `chase` line, and one key per thing
/// met for the event that asks the group when a new one appears.
/// A factory's or a builder's draw at full speed on what it builds now (H-HANDS-ENERGY-DRAW; the player's `spending`
/// line reads the same, H-PLAYER-SPENDING-LINE).
#[derive(Clone, Debug)]
pub(crate) struct ProductionDraw {
    pub who: String,
    pub factory: bool,
    pub target: UnitDefId,
    /// The actor's own build power.
    pub own_power: f32,
    /// Energy and metal a second at its own build power.
    pub energy: f32,
    pub metal: f32,
    /// Build power helping a factory now (builders guarding it, construction turrets told to guard it): it takes that
    /// much more, in proportion to its own.
    pub helping: f32,
}

pub(super) struct ChaseMet {
    pub words: String,
    pub keys: Vec<String>,
}
/// Odds at this ratio or over, and under 1.3, are a narrow win and said so; under it down to 0.8, an even fight.
const NARROW: f32 = 1.1;
/// A party whose reach beats the group's longest by this much kills it on the approach (a Bull's 460 or a beamer's 490 against a Stout's 350).
const OUTRANGE_MARGIN: f32 = 60.0;
/// A party this near a place or an actor is "near" it: the party in an actor's entry.
pub(super) const NEAR: f32 = 800.0;
/// Wreck fields the picture lists (those with 100 metal or more): the fields a builder can be sent to take apart.
pub(super) const WRECK_FIELDS: usize = 3;
/// A point this close to a named place stands at it.
const AT_PLACE: f32 = 400.0;
/// A group's own shooter estimate this far from the side's gets its own place, `shelling_<group>`.
const SHOOTER_APART: f32 = 300.0;
/// The enemy's soldiers seen within this long count in what we know of its army.
const ARMY_MEMORY: i32 = 3 * 60 * FRAMES_PER_SECOND;

#[derive(Clone, Debug)]
pub(crate) struct Place {
    pub name: String,
    pub at: Vec3,
    pub spot: Option<usize>,
    /// A region the player marked (`mark` with a cell range): the cells it covers; `at` is its centre. A group told
    /// to rove within it looks only there (docs/design/2026-10-17-rove-destinations.md).
    pub cells: Option<crate::strategist::shared::Cells>,
}

impl Party {
    /// The enemy unit is in a fight with this party: a member, or one of another party beside it.
    pub(crate) fn in_fight(&self, id: bot_protocol::UnitId) -> bool {
        self.ids.contains(&id) || self.beside.contains(&id)
    }

    /// How many units a fight with it meets.
    pub(crate) fn fighters(&self) -> usize {
        self.ids.len() + self.beside.len()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Party {
    pub name: String,
    pub ids: Vec<bot_protocol::UnitId>,
    pub at: Vec3,
    pub metal: f32,
    pub composition: String,
    /// Their commander is one of its members.
    pub has_commander: bool,
    /// Every member is identified and flies: the group's whole-group attack and shelling of it carry the forbidden
    /// question, as its detachments do (the impedance tally, 2026-10-16: "aircraft are never chased by any group" in
    /// four games, and the mark sat on detachments alone).
    pub air: bool,
    /// What it is doing to us now, the verb with it, from the hits and the reclaims of the last seconds
    /// (H-HANDS-PARTY-KILLING): "killing our Vehicle Plant (armvp), 2 of our Solar Collector (armsolar)", "taking
    /// apart our Metal Extractor (armmex), 46% left and gone in about 5 s", and the metal of those units.
    pub harming: Option<(String, f32)>,
    /// Every member is identified and none has a weapon, and no turret of theirs covers it: it cannot fight back.
    pub unarmed: bool,
    /// The enemy's armed buildings whose reach covers where it stands (a lone unit under a nest's turrets is not a
    /// lone unit): the metal of those that hit ground, of those that hit air, and words ("3 turrets: 1 armhlt,
    /// 2 armllt"; empty under none).
    pub turret_metal: f32,
    pub turret_metal_air: f32,
    /// The mobile enemies of other parties within `BESIDE` of its centre, and words naming their parties with what
    /// stands there in all ("with party_500 and party_513 within 600, 6900 metal of his stands there"); empty
    /// without the `beside` layer or with nobody there.
    pub beside: Vec<bot_protocol::UnitId>,
    pub beside_words: String,
    pub turrets: String,
}

/// A party's last sighting, kept after it leaves sight (H-HANDS-ENEMY-MEMORY).
#[derive(Clone, Debug)]
pub(crate) struct PartySeen {
    pub name: String,
    pub composition: String,
    pub metal: f32,
    pub at: Vec3,
    pub seen: i32,
    pub heading: String,
}

#[derive(Clone)]
pub(crate) struct Picture {
    pub state: Value,
    /// The rules text, its parts that bear on one kind of actor marked (`diet::rules_for`): a request's state
    /// gets the parts its questions can use.
    pub rules: String,
    pub places: Vec<Place>,
    pub parties: Vec<Party>,
}

/// The named place a point stands at (within `AT_PLACE`), if any.
pub(super) fn place_of(places: &[Place], at: Vec3) -> Option<&str> {
    places.iter().map(|pl| (pl.at.dist2d(at), pl)).min_by(|a, b| a.0.total_cmp(&b.0)).filter(|(d, _)| *d < AT_PLACE).map(|(_, pl)| pl.name.as_str())
}

/// The share of a group's metal lost lately, in words: Jev is not a calculator (docs.typesafe.ai/model-jaggedness).
pub(super) fn share_words(share: f32) -> &'static str {
    if share < 0.1 {
        "a few"
    } else if share < 0.25 {
        "a noticeable share"
    } else if share < 0.5 {
        "a large share"
    } else {
        "most of it"
    }
}

/// The fight's verdict from the share lost to the party in question and the turrets covering it: the step-back
/// rule's input ("whose `losses` line says it is losing this fight"). Losses to anything else carry no verdict
/// (human-11, 6:37: three Pawns a Sentry had killed, the Sentry dead for five seconds, read as "losing this fight"
/// against the one unarmed constructor in sight, and the raid walked home).
pub(super) fn fight_verdict(share_to_this_fight: f32) -> &'static str {
    if share_to_this_fight < 0.25 {
        ""
    } else if share_to_this_fight < 0.5 {
        ": it is losing this fight"
    } else {
        ": it is being wiped out"
    }
}

/// A group's losses of the last while, said with the killers: "162 metal, a large share, to a Sentry (dead now),
/// none to this party" and the verdict against `party` (the party in the group's entry) from the losses to it and
/// to turrets. `metal` is what the group still stands at.
pub(super) struct LossWords {
    pub count: usize,
    pub metal: f32,
    pub share: &'static str,
    pub to_whom: String,
    pub verdict: &'static str,
}

impl Brain {
    pub(super) fn loss_words(&self, lost: &[&(i32, UnitDefId, Option<(bot_protocol::UnitId, UnitDefId)>)], standing_metal: f32, party: Option<&Party>, parties: &[Party]) -> LossWords {
        let cost = |def: UnitDefId| self.world.def(def).map_or(0.0, |d| d.metal_cost);
        let metal: f32 = lost.iter().map(|(_, def, _)| cost(*def)).sum();
        let mut to_fight = 0.0;
        let mut by: BTreeMap<String, usize> = BTreeMap::new();
        for (_, def, killer) in lost {
            let words = match killer {
                None => "something unseen".to_string(),
                Some((id, kdef)) => {
                    let building = self.world.def(*kdef).is_some_and(|d| d.speed <= 0.0);
                    let this = party.is_some_and(|p| p.ids.contains(id));
                    if this || building {
                        to_fight += cost(*def);
                    }
                    if this {
                        "this party".to_string()
                    } else if building {
                        format!("a {}{}", self.short_words(*kdef), if self.enemy_buildings.contains_key(id) { "" } else { " (dead now)" })
                    } else if let Some(other) = parties.iter().find(|p| p.ids.contains(id)) {
                        other.name.clone()
                    } else {
                        format!("a {} out of sight now", self.short_words(*kdef))
                    }
                }
            };
            *by.entry(words).or_default() += 1;
        }
        let mut to_whom: Vec<String> = by.iter().map(|(who, n)| if *n == 1 { who.clone() } else { format!("{n} to {who}") }).collect();
        if party.is_some() && !by.contains_key("this party") {
            to_whom.push("none to this party".to_string());
        }
        let share = metal / (metal + standing_metal).max(1.0);
        LossWords { count: lost.len(), metal, share: share_words(share), to_whom: to_whom.join(", "), verdict: if party.is_some() { fight_verdict(to_fight / (metal + standing_metal).max(1.0)) } else { "" } }
    }
}

pub(crate) fn clock(frame: i32) -> String {
    let seconds = frame / FRAMES_PER_SECOND;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub(crate) fn stock_words(current: f32, storage: f32) -> &'static str {
    let share = if storage > 0.0 { current / storage } else { 0.0 };
    if share < 0.1 {
        "nearly empty"
    } else if share < 0.3 {
        "low"
    } else if share < 0.7 {
        "some in store"
    } else if share < 0.95 {
        "plenty in store"
    } else {
        "full: what comes in beyond this is wasted"
    }
}

pub(crate) fn flow_words(income: f32, usage: f32, current: f32, storage: f32) -> String {
    let share = if storage > 0.0 { current / storage } else { 0.0 };
    // The engine caps spending at income once the store is empty, so a stall reads as "in balance" by the numbers.
    if share < 0.05 && usage >= income * 0.9 {
        "STALLING: the store is empty and everything that needs it builds slowly; more income is needed".into()
    } else if usage > income * 1.15 && share < 0.25 {
        "spending faster than it comes in: stalling, everything builds slowly".into()
    } else if usage > income * 1.15 {
        // Seconds to empty beside the trend (human-3: the lab was chosen at 351 energy running down 56 a second).
        format!("spending faster than it comes in, running the store down: empty in about {:.0} s", current / (usage - income))
    } else if income > usage * 1.3 && share > 0.6 {
        "more coming in than is spent: banking it unused".into()
    } else {
        "in balance".into()
    }
}

fn health_words(share: f32) -> &'static str {
    if share > 0.95 {
        "full"
    } else if share > 0.7 {
        "lightly hurt"
    } else if share > 0.4 {
        "hurt"
    } else {
        "badly hurt, near death"
    }
}

/// How many constructors we have, against the extractors they serve.
pub(crate) fn soldier_words(soldiers: usize, metal: f32) -> &'static str {
    if soldiers == 0 {
        "none: we have no army at all"
    } else if soldiers < 4 {
        "a handful"
    } else if metal < 1500.0 {
        "a group"
    } else {
        "a real army"
    }
}

pub(crate) fn distance_words(elmos: f32) -> &'static str {
    if elmos < 500.0 {
        "right here"
    } else if elmos < 1200.0 {
        "near"
    } else if elmos < 2500.0 {
        "some way off"
    } else {
        "far"
    }
}

impl Brain {
    /// A unit type in a few words, for the menus and the picture.
    /// A definition's role in a word or two: the glossary's class, else read off the definition.
    pub(super) fn class_words(&self, def: UnitDefId) -> String {
        if let Some(e) = glossary::entry(self.name(def)).filter(|e| !e.class.is_empty()) {
            return e.class.clone();
        }
        let Some(d) = self.world.def(def) else { return "unit".into() };
        let mobile = d.speed > 0.0;
        let word = match () {
            _ if self.world.is_commander_def(def) => "commander",
            _ if d.extracts_metal > 0.0 => "metal extractor",
            _ if !mobile && !d.build_options.is_empty() => "factory",
            _ if !mobile && d.build_speed > 0.0 => "construction turret",
            _ if !mobile && d.weapon_count > 0 => "defence",
            _ if !mobile && d.radar_range > 0.0 => "radar",
            _ if !mobile && d.converter.is_some() => "energy converter",
            _ if !mobile && (d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.wind_cap > 0.0) => "energy building",
            _ if !mobile && (d.metal_storage > 0.0 || d.energy_storage > 0.0) => "storage",
            _ if !mobile => "building",
            _ if d.build_speed > 0.0 => "constructor",
            _ if d.weapon_count > 0 => "soldier",
            _ => "unit",
        };
        word.to_string()
    }

    /// A unit for a menu line or an actor's `is`: "Pawn (armpw; raider bot, 54 metal): gloss", with the splash of a
    /// weapon that hurts a neighbour two hulls away ("Bull (armbull; ..., 400 metal, its shells hit everything within
    /// 65 of where they land)": the area from the definitions, K-units-duel-spacing-decides-area-damage).
    pub(super) fn unit_words(&self, def: UnitDefId) -> String {
        let internal = self.name(def);
        let metal = self.world.def(def).map_or(0.0, |d| d.metal_cost);
        let area = self.world.def(def).map_or(0.0, |d| d.blast_radius);
        let splash = if area >= 30.0 { format!(", its shells hit everything within {area:.0} of where they land") } else { String::new() };
        match glossary::entry(internal) {
            Some(e) if !e.name.is_empty() => {
                let gloss = if e.gloss.is_empty() { String::new() } else { format!(": {}", e.gloss) };
                format!("{} ({internal}; {}, {metal:.0} metal{splash}){gloss}", e.name, if e.class.is_empty() { self.class_words(def) } else { e.class.clone() })
            }
            _ if self.world.is_commander_def(def) => format!("{internal} (our commander; the strongest builder, a good fighter; the game is lost if it dies)"),
            _ => format!("{internal} ({}; {metal:.0} metal{splash})", self.class_words(def)),
        }
    }

    /// A type in the picture's prose: "Solar Collector (armsolar)", or the internal name and its class.
    pub(crate) fn short_words(&self, def: UnitDefId) -> String {
        let internal = self.name(def);
        match glossary::entry(internal) {
            Some(e) if !e.name.is_empty() => format!("{} ({internal})", e.name),
            _ => format!("{internal} ({})", self.class_words(def)),
        }
    }

    pub(super) fn composition_words(&self, units: &[&OwnUnit]) -> String {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for unit in units {
            *counts.entry(self.short_words(unit.def)).or_default() += 1;
        }
        counts.iter().map(|(name, n)| format!("{n} {name}")).collect::<Vec<_>>().join(", ")
    }

    /// The ground's height at a place (water level 0: negative is that deep under water); 0 without terrain.
    pub(super) fn ground_height(&self, pos: Vec3) -> f32 {
        let t = &self.world.hello.terrain;
        if t.width == 0 || t.cell <= 0.0 {
            return 0.0;
        }
        let (x, z) = (((pos.x / t.cell) as u32).min(t.width - 1), ((pos.z / t.cell) as u32).min(t.height.saturating_sub(1)));
        t.heights.get((z * t.width + x) as usize).map_or(0.0, |h| f32::from(*h))
    }

    /// The player's whitelist for an actor (`produce`, H-HANDS-PRODUCE): its own, else `all_builders` for a builder,
    /// else `all`, else none.
    pub(super) fn allowed_units(&self, actor: &str) -> Option<Allowance> {
        let shared = self.strategist.as_ref()?;
        let allowed = shared.allowed.lock().unwrap();
        let builder = actor.starts_with("commander") || actor.starts_with("constructor_");
        let mut allowance = allowed.get(actor).or_else(|| if builder { allowed.get("all_builders") } else { None }).or_else(|| allowed.get("all")).cloned()?;
        // A role word in an entry ("constructor:1", "raider") is the faction's unit from here on: the actor's own
        // faction (a factory or constructor given by a seat of another faction makes its own), else the seat's.
        if let Some(kit) = self.kit {
            let plant = actor.starts_with("plant_");
            let roster = actor.rsplit('_').next().and_then(|n| n.parse::<i32>().ok()).and_then(|id| self.known_units.get(&bot_protocol::UnitId(id))).map_or(kit.roster, |(def, _)| crate::brain::roster::of_unit(self.name(*def)));
            allowance.units = allowance.units.iter().map(|e| if plant { roster.resolve_words_at_the_plant(e) } else { roster.resolve_words(e) }).collect();
        }
        Some(allowance)
    }

    /// What each rover of a roving group is doing this second, from the lane (H-MICRO-ROVE).
    pub(super) fn rover_words(&self, group: &super::Group, own: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit], places: &[Place]) -> String {
        let words: Vec<String> = group
            .units(own)
            .iter()
            .map(|u| {
                let who = format!("its {} at {}", self.short_words(u.def), self.place_words(places, u.pos));
                let Some(r) = self.lane.roving(u.id) else { return format!("{who} is starting") };
                let what = match r.doing {
                    "evading" => "is stepping out of the reach of something that can shoot it".to_string(),
                    "attacking" => match r.target.and_then(|t| enemies.iter().find(|e| e.id == t)) {
                        Some(e) => format!("is attacking his unguarded {} at {}", e.def.map_or("unit".to_string(), |d| self.short_words(d)), self.place_words(places, e.pos)),
                        None => "is attacking an unguarded unit of his".to_string(),
                    },
                    "looking" => r.goal.map_or_else(String::new, |(name, look)| format!("is on its way to look at {name}, {:.0} to go", u.pos.dist2d(look))),
                    _ => "waits where it is safe: every way to a place worth a look passes the reach of something that can shoot it".to_string(),
                };
                let evaded = if r.evasions > 0 { format!(" ({} evasion{} so far)", r.evasions, if r.evasions == 1 { "" } else { "s" }) } else { String::new() };
                format!("{who} {what}{evaded}")
            })
            .collect();
        words.join("; ")
    }

    /// The named place nearest `pos`, with its grid cell, or the grid cell alone.
    pub(super) fn place_words(&self, places: &[Place], pos: Vec3) -> String {
        let grid = self.world.grid(pos);
        match places.iter().filter(|p| p.cells.is_none()).map(|p| (p.at.dist2d(pos), p)).min_by(|a, b| a.0.total_cmp(&b.0)) {
            Some((d, place)) if d < 300.0 => format!("{} ({grid})", place.name),
            Some((d, place)) if d < 900.0 => format!("{:.0} from {} ({grid})", d, place.name),
            _ => format!("({grid})"),
        }
    }

    /// Enemy mobile units in sight, grouped into parties (`chained_parties`, or `kept_parties` under the `kept` layer),
    /// nearest home first. A party keeps the name it had in the
    /// last picture while any member of it is still in it (H-HANDS-PARTY-NAMES: named by distance from home, the
    /// names shuffled as parties moved, and the player's "party_2" in its orders meant another party by the next
    /// ask); a new party takes a number never used in this game.
    pub(super) fn enemy_parties(&self, enemies: &[EnemyUnit], previous: &[Party], next_name: &std::cell::Cell<usize>, kept: bool, beside: bool) -> Vec<Party> {
        let mobile: Vec<&EnemyUnit> = enemies.iter().filter(|e| e.def.is_none_or(|d| self.world.def(d).is_some_and(|d| d.speed > 0.0))).collect();
        let groups = if kept { Self::kept_parties(&mobile, previous) } else { Self::chained_parties(&mobile) };
        let mut parties: Vec<Party> = Vec::new();
        for members in groups {
            let n = members.len() as f32;
            let at = members.iter().fold(Vec3::default(), |sum, i| Vec3 { x: sum.x + mobile[*i].pos.x / n, y: 0.0, z: sum.z + mobile[*i].pos.z / n });
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            let mut metal = 0.0;
            for i in &members {
                let name = mobile[*i].def.map_or("unidentified", |d| self.name(d));
                *counts.entry(name).or_default() += 1;
                metal += mobile[*i].def.and_then(|d| self.world.def(d)).map_or(110.0, |d| d.metal_cost);
            }
            let composition = counts.iter().map(|(name, n)| format!("{n} {name}")).collect::<Vec<_>>().join(", ");
            let has_commander = members.iter().any(|i| mobile[*i].def.is_some_and(|d| self.world.is_commander_def(d)));
            let air = members.iter().all(|i| mobile[*i].def.is_some_and(|d| self.world.domain_of(d) == crate::world::Domain::Air));
            let ids: Vec<bot_protocol::UnitId> = members.iter().map(|i| mobile[*i].id).collect();
            let harming = self.harm_words(&ids);
            let (turret_metal, turret_metal_air, turrets) = self.turrets_covering(at);
            let unarmed = turret_metal == 0.0 && turret_metal_air == 0.0 && members.iter().all(|i| mobile[*i].def.and_then(|d| self.world.def(d)).is_some_and(|d| d.weapon_count == 0));
            parties.push(Party { name: String::new(), ids, at, metal, composition, has_commander, air, harming, unarmed, turret_metal, turret_metal_air, turrets, beside: Vec::new(), beside_words: String::new() });
        }
        parties.sort_by(|a, b| a.at.dist2d(self.home).total_cmp(&b.at.dist2d(self.home)));
        // Names: the last picture's for a party sharing a member with one of them (the larger overlap wins a name
        // two of them claim), a fresh number for the rest.
        let mut taken: Vec<String> = Vec::new();
        for party in parties.iter_mut() {
            let kept = previous
                .iter()
                .filter(|old| !taken.contains(&old.name))
                .map(|old| (old.ids.iter().filter(|id| party.ids.contains(id)).count(), &old.name))
                .filter(|(shared, _)| *shared > 0)
                .max_by_key(|(shared, _)| *shared)
                .map(|(_, name)| name.clone());
            if let Some(name) = kept {
                taken.push(name.clone());
                party.name = name;
            }
        }
        // The side's registry (H-HANDS-SIDE-PARTIES): a party another seat has named keeps that name here, and a
        // fresh number comes from the side's counter, so the two seats' pictures and the player's report say one name.
        // Under a commander the names are the side's, one registry for every seat whatever its player.
        let registry = self.strategist.as_ref().map(|s| s.side.get().unwrap_or(s).parties.lock().unwrap());
        match registry {
            Some(mut registry) => {
                for party in parties.iter_mut().filter(|p| p.name.is_empty()) {
                    party.name = registry.name_for(&party.ids, &taken);
                    taken.push(party.name.clone());
                }
                for party in &parties {
                    registry.record(&party.name, &party.ids);
                }
            }
            None => {
                for party in parties.iter_mut().filter(|p| p.name.is_empty()) {
                    party.name = format!("party_{}", next_name.get());
                    next_name.set(next_name.get() + 1);
                }
            }
        }
        if beside {
            let others: Vec<(String, f32, Vec<bot_protocol::UnitId>)> = parties
                .iter()
                .map(|p| {
                    let near: Vec<&&EnemyUnit> = mobile.iter().filter(|e| !p.ids.contains(&e.id) && e.pos.dist2d(p.at) < BESIDE).collect();
                    let names: Vec<&str> = parties.iter().filter(|o| near.iter().any(|e| o.ids.contains(&e.id))).map(|o| o.name.as_str()).collect();
                    let metal: f32 = near.iter().map(|e| e.def.and_then(|d| self.world.def(d)).map_or(110.0, |d| d.metal_cost)).sum();
                    (names.join(" and "), metal, near.iter().map(|e| e.id).collect())
                })
                .collect();
            for (party, (names, metal, ids)) in parties.iter_mut().zip(others) {
                if !ids.is_empty() {
                    party.beside_words = format!("with {names} within {BESIDE:.0}, {:.0} metal of his stands there", party.metal + metal);
                    party.beside = ids;
                }
            }
        }
        parties
    }

    /// Today's rule: every mobile enemy within `PARTY_RADIUS` of a member is a member.
    fn chained_parties(mobile: &[&EnemyUnit]) -> Vec<Vec<usize>> {
        let mut taken = vec![false; mobile.len()];
        let mut groups = Vec::new();
        for start in 0..mobile.len() {
            if taken[start] {
                continue;
            }
            taken[start] = true;
            let mut members = vec![start];
            let mut next = 0;
            while next < members.len() {
                let from = mobile[members[next]].pos;
                for other in 0..mobile.len() {
                    if !taken[other] && mobile[other].pos.dist2d(from) < PARTY_RADIUS {
                        taken[other] = true;
                        members.push(other);
                    }
                }
                next += 1;
            }
            groups.push(members);
        }
        groups
    }

    /// Parties kept from the last picture (H-HANDS-PARTIES-KEPT). A chain makes one party of a front (two thirds of
    /// the enemies in sight in a party wider than 800 in glitters-8v8-hard-1, a fifth of its metal over 600 from the
    /// place the picture gave it), and cutting a wide party each second renames a tenth of the units a second
    /// (`run/party_rules.py`). So a party lasts: its members still within `KEPT_RADIUS` of its centre stay; two
    /// parties whose centres come within `KEPT_MERGE` are one; an enemy without a party joins the nearest whose
    /// centre is within `KEPT_RADIUS`; the rest seed new ones, the unit with the most others within `KEPT_RADIUS`
    /// first.
    fn kept_parties(mobile: &[&EnemyUnit], previous: &[Party]) -> Vec<Vec<usize>> {
        let centre = |members: &[usize]| {
            let n = members.len() as f32;
            members.iter().fold(Vec3::default(), |sum, i| Vec3 { x: sum.x + mobile[*i].pos.x / n, y: 0.0, z: sum.z + mobile[*i].pos.z / n })
        };
        let mut taken = vec![false; mobile.len()];
        let mut groups: Vec<(Vec<usize>, Vec3)> = Vec::new();
        for old in previous {
            let here: Vec<usize> = (0..mobile.len()).filter(|i| !taken[*i] && old.ids.contains(&mobile[*i].id)).collect();
            if here.is_empty() {
                continue;
            }
            let at = centre(&here);
            let stay: Vec<usize> = here.into_iter().filter(|i| mobile[*i].pos.dist2d(at) < KEPT_RADIUS).collect();
            if stay.is_empty() {
                continue;
            }
            for i in &stay {
                taken[*i] = true;
            }
            let at = centre(&stay);
            groups.push((stay, at));
        }
        for a in 0..groups.len() {
            for b in a + 1..groups.len() {
                if groups[a].0.is_empty() || groups[b].0.is_empty() || groups[a].1.dist2d(groups[b].1) >= KEPT_MERGE {
                    continue;
                }
                let (big, small) = if groups[a].0.len() >= groups[b].0.len() { (a, b) } else { (b, a) };
                let moved = std::mem::take(&mut groups[small].0);
                groups[big].0.extend(moved);
                groups[big].1 = centre(&groups[big].0);
            }
        }
        groups.retain(|(members, _)| !members.is_empty());
        for i in 0..mobile.len() {
            if taken[i] {
                continue;
            }
            let nearest = groups.iter_mut().map(|g| (g.1.dist2d(mobile[i].pos), g)).filter(|(d, _)| *d < KEPT_RADIUS).min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, group)) = nearest {
                group.0.push(i);
                taken[i] = true;
            }
        }
        let mut groups: Vec<Vec<usize>> = groups.into_iter().map(|(members, _)| members).collect();
        loop {
            let free: Vec<usize> = (0..mobile.len()).filter(|i| !taken[*i]).collect();
            let around = |i: usize| free.iter().copied().filter(|o| mobile[*o].pos.dist2d(mobile[i].pos) < KEPT_RADIUS).collect::<Vec<usize>>();
            let Some(seed) = free.iter().copied().max_by_key(|i| around(*i).len()) else { break };
            let members = around(seed);
            for i in &members {
                taken[*i] = true;
            }
            groups.push(members);
        }
        groups
    }

    /// What the units `ids` are doing to us now, the verb with it: "killing" what they have hit in the last seconds
    /// (`killing_words`), "taking apart" the buildings of ours whose health falls beside a builder of theirs
    /// (`Brain::track_takings`), with how much is left and when it is gone; and the metal of all of it.
    pub(crate) fn harm_words(&self, ids: &[bot_protocol::UnitId]) -> Option<(String, f32)> {
        let mut parts: Vec<String> = Vec::new();
        let mut metal = 0.0;
        if let Some((what, m)) = self.killing_words(ids, None) {
            parts.push(format!("killing {what}"));
            metal += m;
        }
        let taken: Vec<String> = self
            .takings
            .iter()
            .filter(|t| ids.contains(&t.taker))
            .map(|t| {
                metal += self.world.def(t.victim_def).map_or(0.0, |d| d.metal_cost);
                let left = 100.0 * t.health / t.max_health.max(1.0);
                let fell = t.from - t.health;
                let gone = if fell > 0.0 && t.frame > t.since { format!(" and gone in about {:.0} s", t.health / fell * (t.frame - t.since) as f32 / super::super::FRAMES_PER_SECOND as f32) } else { String::new() };
                format!("our {}, {left:.0}% left{gone}", self.short_words(t.victim_def))
            })
            .collect();
        if !taken.is_empty() {
            parts.push(format!("taking apart {}", taken.join(", ")));
        }
        (!parts.is_empty()).then(|| (parts.join(" and "), metal))
    }

    /// What the units `ids` have hit in the last seconds, buildings first, one entry per kind with its count, and the
    /// metal of the victims; nothing when they have hit nothing of ours. `among`: only these victims (a group's own
    /// soldiers) count.
    pub(crate) fn killing_words(&self, ids: &[bot_protocol::UnitId], among: Option<&[bot_protocol::UnitId]>) -> Option<(String, f32)> {
        let mut victims: Vec<(bot_protocol::UnitId, UnitDefId)> = Vec::new();
        for h in self.hits.iter().filter(|h| ids.contains(&h.attacker) && among.is_none_or(|a| a.contains(&h.victim))) {
            if !victims.iter().any(|(id, _)| *id == h.victim) {
                victims.push((h.victim, h.victim_def));
            }
        }
        if victims.is_empty() {
            return None;
        }
        victims.sort_by_key(|(_, def)| self.world.def(*def).is_none_or(|d| d.speed > 0.0));
        let metal: f32 = victims.iter().map(|(_, def)| self.world.def(*def).map_or(0.0, |d| d.metal_cost)).sum();
        let mut counts: Vec<(UnitDefId, usize)> = Vec::new();
        for (_, def) in &victims {
            match counts.iter_mut().find(|(d, _)| d == def) {
                Some((_, n)) => *n += 1,
                None => counts.push((*def, 1)),
            }
        }
        let words = counts.iter().map(|(def, n)| if *n == 1 { format!("our {}", self.short_words(*def)) } else { format!("{n} of our {}", self.short_words(*def)) }).collect::<Vec<_>>().join(", ");
        Some((words, metal))
    }

    /// The fight simulator's odds of `units` against a party, in words (Jev compares nothing itself).
    /// What each factory and building builder would draw at full speed on what it builds now. A factory on what
    /// stands on its pad, else its queue's first unit, at its own power, with the power helping it (builders guarding
    /// it, construction turrets told to guard it) counted beside; a builder on its build task
    /// (docs/design/2026-09-22-energy-draw.md, docs/design/2026-10-17-spending-line.md).
    pub(crate) fn production_draws(&self, own: &[OwnUnit], pianist: &Pianist) -> Vec<ProductionDraw> {
        let mut draws = Vec::new();
        for unit in own.iter().filter(|u| !u.being_built) {
            let Some(def) = self.world.def(unit.def) else { continue };
            let factory = self.world.is_factory_def(unit.def);
            let building = if factory {
                own.iter().find(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| u.def).or_else(|| pianist.lab_queue.get(&unit.id).and_then(|q| q.first()).map(|(d, _)| *d))
            } else if self.world.is_mobile_builder(unit.def) {
                match pianist.tasks.get(&unit.id) {
                    Some(Task::Build { def, .. }) => Some(def.clone()),
                    _ => None,
                }
            } else {
                None
            };
            let Some(target) = building else { continue };
            let (energy, metal) = self.draw_rate(def.build_speed, target);
            if energy <= 0.0 {
                continue;
            }
            let helping = if factory {
                own.iter()
                    .filter(|u| !u.being_built && u.id != unit.id)
                    .filter(|u| matches!(pianist.tasks.get(&u.id), Some(Task::Assist { lab, .. }) if *lab == unit.id) || self.nano_guards.get(&u.id).is_some_and(|(f, _)| *f == unit.id))
                    .filter_map(|u| self.world.def(u.def))
                    .map(|d| d.build_speed)
                    .sum()
            } else {
                0.0
            };
            draws.push(ProductionDraw { who: self.actor_name(unit.id), factory, target, own_power: def.build_speed, energy, metal, helping });
        }
        draws
    }

    /// Energy and metal a second that `power` build power takes on `def` at full speed: power x cost / build time.
    pub(crate) fn draw_rate(&self, power: f32, def: UnitDefId) -> (f32, f32) {
        self.world.def(def).filter(|d| d.build_time > 0.0).map_or((0.0, 0.0), |d| (power * d.energy_cost / d.build_time, power * d.metal_cost / d.build_time))
    }

    /// Whether our faction's build tree (from the commander) reaches a definition.
    pub(super) fn faction_reaches(&self, def: UnitDefId) -> bool {
        self.kit.map(|k| self.world.reachable_from(k.commander).contains(&def)).unwrap_or(false)
    }

    /// For the commander's own line: which of a party must come inside its D-gun to hit it, and which outreach it
    /// (a handful of Pawns die to the D-gun; a line of Stouts kills it from beyond it).
    pub(super) fn commander_reach_words(&self, def: UnitDefId, party_ids: &[bot_protocol::UnitId], enemies: &[bot_protocol::EnemyUnit]) -> String {
        let dgun = self.dgun_reach(def);
        if dgun <= 0.0 {
            return String::new();
        }
        let mut inside: Vec<(String, f32)> = Vec::new();
        let mut beyond: Vec<(String, f32)> = Vec::new();
        let mut unknown = 0;
        for enemy in enemies.iter().filter(|e| party_ids.contains(&e.id)) {
            let Some(their) = enemy.def else { unknown += 1; continue };
            let Some((reach, dps, _)) = self.sim_stats(their) else { continue };
            if dps <= 0.0 {
                continue;
            }
            let name = self.short_words(their);
            let list = if reach <= dgun + 20.0 { &mut inside } else { &mut beyond };
            if !list.iter().any(|(n, _)| *n == name) {
                list.push((name, reach));
            }
        }
        let count = |list: &[(String, f32)], ids: &[bot_protocol::UnitId]| -> usize {
            enemies.iter().filter(|e| ids.contains(&e.id) && e.def.is_some_and(|d| list.iter().any(|(n, _)| *n == self.short_words(d)))).count()
        };
        let mut words = String::new();
        let n_in = count(&inside, party_ids);
        let n_out = count(&beyond, party_ids);
        if n_in > 0 {
            words.push_str(&format!("; {n_in} of them must come inside its D-gun ({dgun:.0}) to hit it ({})", inside.iter().map(|(n, r)| format!("{n} {r:.0}")).collect::<Vec<_>>().join(", ")));
        }
        if n_out > 0 {
            words.push_str(&format!("; {n_out} of them outreach its D-gun and it cannot answer them ({})", beyond.iter().map(|(n, r)| format!("{n} {r:.0}")).collect::<Vec<_>>().join(", ")));
        }
        if unknown > 0 {
            words.push_str(&format!("; {unknown} of unknown type"));
        }
        words
    }

    pub(super) fn odds_words(&self, units: &[&OwnUnit], party: &Party, enemies: &[EnemyUnit]) -> String {
        let mut theirs = super::super::combat::Force::default();
        for enemy in enemies.iter().filter(|e| party.in_fight(e.id)) {
            match enemy.def {
                Some(def) => theirs.add(def),
                None => theirs.unidentified += 1,
            }
        }
        // The turrets covering it fight with it (worlds-2, 18:33: "party_99 (1 armsnipe, 1 armwar, at nest) met
        // with 5310 metal: we outweigh it heavily", and the nest's Overwatch, beamer and light turrets killed 16
        // Stouts in 15 s).
        theirs.turret_metal += party.turret_metal;
        theirs.turret_metal_air += party.turret_metal_air;
        let ours = Brain::force_of(units);
        let ratio = self.odds(&ours, &theirs);
        // Reach: a party that outranges everything in the group, or turrets covering it that do, kills the group on
        // its approach before it shoots, whatever the metal says (bluegecko-3v1-comet-catcher-9: 94 tier-1 tanks
        // of reach 315-350 died to Bulls of 460 and 24 to beamers of 490, most of them to shooters out of sight,
        // under "we outweigh it"; the user: dozens of bad trades by the wrong unit in the wrong place).
        let reach_of = |def: UnitDefId| self.world.def(def).map_or(0.0, |d| d.reach);
        let our_reach = units.iter().map(|u| reach_of(u.def)).fold(0.0, f32::max);
        let party_reach = enemies.iter().filter(|e| party.in_fight(e.id)).filter_map(|e| e.def).map(reach_of).fold(0.0, f32::max);
        let turret_reach = self
            .enemy_buildings
            .values()
            .filter_map(|(def, pos, _)| self.world.def(*def).filter(|d| d.weapon_count > 0 && d.reach > 0.0 && pos.dist2d(party.at) <= d.reach + TURRET_MARGIN && self.can_hit(*def, self.all_air(&ours))).map(|d| d.reach))
            .fold(0.0, f32::max);
        let their_reach = party_reach.max(turret_reach);
        let outranged = our_reach > 0.0 && their_reach > our_reach + OUTRANGE_MARGIN;
        // The reach is said beside the metal, never over it: a turret that outranges a group is no bar to pushing in
        // and killing it (the user, 2026-09-27), it is a cost paid on the approach that a group pays as one body.
        let reach_words = if outranged {
            format!(
                ", and it outranges us ({} to our {:.0}{}): the approach is paid under its fire, so the group goes in as one body or not at all",
                if turret_reach > party_reach { format!("turrets covering it reach {turret_reach:.0}") } else { format!("it reaches {party_reach:.0}") },
                our_reach,
                if turret_reach > party_reach && party_reach > 0.0 { format!(", the party itself {party_reach:.0}") } else { String::new() }
            )
        } else {
            String::new()
        };
        // Tier, speed, the commander and aircraft beside the metal (H-HANDS-ODDS-WHAT-SHOOTS): a Bull was a metal
        // price only ("an even fight" for 13 Stouts against 3 Bulls, game 9); "it outruns this group at 87" was a
        // radar blip priced as a Pawn while the Welders ran at 48 (Cape Violet 10:20); eleven Incisors walked into a
        // commander's D-gun and thirteen Brutes into its death blast (game 7); a mixed party's air part was chased
        // by tanks that could not hit it (game 3 14:21).
        let identified: Vec<(bot_protocol::UnitId, UnitDefId, Vec3)> = enemies.iter().filter(|e| party.in_fight(e.id)).filter_map(|e| e.def.map(|d| (e.id, d, e.pos))).collect();
        let tier_of = |def: UnitDefId| glossary::entry(self.name(def)).map_or(1, |e| e.tier);
        let their_tier = identified.iter().map(|(_, d, _)| tier_of(*d)).max().unwrap_or(1);
        let our_tier = units.iter().map(|u| tier_of(u.def)).max().unwrap_or(1);
        let mut more = String::new();
        if their_tier > our_tier {
            let top: Vec<String> = {
                let mut counts: BTreeMap<UnitDefId, usize> = BTreeMap::new();
                for (_, d, _) in identified.iter().filter(|(_, d, _)| tier_of(*d) == their_tier) {
                    *counts.entry(*d).or_default() += 1;
                }
                counts.iter().map(|(d, n)| format!("{n} {} ({:.0} metal each)", self.short_words(*d), self.world.def(*d).map_or(0.0, |x| x.metal_cost))).collect()
            };
            more.push_str(&format!("; it outclasses us: tier {their_tier} against our tier {our_tier} ({})", top.join(", ")));
        } else if our_tier > their_tier && !identified.is_empty() {
            more.push_str(&format!("; we outclass it (tier {our_tier} against its tier {their_tier})"));
        }
        let speed_of = |def: UnitDefId| self.world.def(def).map_or(0.0, |d| d.speed);
        let their_slowest = identified.iter().map(|(_, d, _)| speed_of(*d)).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let their_fastest = identified.iter().map(|(_, d, _)| speed_of(*d)).fold(0.0, f32::max);
        let our_fastest = units.iter().map(|u| speed_of(u.def)).fold(0.0, f32::max);
        let our_slowest = units.iter().map(|u| speed_of(u.def)).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let unidentified = party.fighters() - identified.len();
        if identified.is_empty() {
            more.push_str("; its speed is unknown (radar contacts only)");
        } else {
            if their_slowest.is_finite() && their_slowest > our_fastest {
                more.push_str(&format!("; it outruns everything here (its slowest {their_slowest:.0} against our fastest {our_fastest:.0}): a chase drives it off, a kill needs it to stand"));
            } else if their_slowest.is_finite() && our_fastest > their_slowest {
                more.push_str(&format!("; our fastest ({our_fastest:.0}) catch its slowest ({their_slowest:.0})"));
            }
            if our_slowest.is_finite() && their_fastest > our_slowest && !party.unarmed {
                more.push_str(&format!("; its fastest ({their_fastest:.0}) catch our slowest ({our_slowest:.0}): it chooses the fight"));
            }
            if unidentified > 0 {
                more.push_str(&format!("; {unidentified} more of unknown type in it"));
            }
        }
        if party.has_commander
            && let Some((_, def, at)) = identified.iter().find(|(_, d, _)| self.world.is_commander_def(*d))
        {
            let dgun = self.dgun_reach(*def);
            let inside = units.iter().filter(|u| u.pos.dist2d(*at) <= dgun).count();
            more.push_str(&format!("; its commander is in it: it D-guns anything within {dgun:.0} (one shot kills; {inside} of ours stand inside now)"));
            if let Some(blast) = self.world.def(*def).and_then(|d| d.death_blast) {
                let in_blast = units.iter().filter(|u| u.pos.dist2d(*at) <= blast.radius).count();
                more.push_str(&format!(", and its death takes everything within {:.0} ({:.0} damage; {in_blast} of ours inside now)", blast.radius, blast.damage));
            }
        }
        let air_in_it = identified.iter().filter(|(_, d, _)| self.world.domain_of(*d) == crate::world::Domain::Air).count();
        if air_in_it > 0 && air_in_it < party.fighters() && !self.force_can_hit(&ours, true) {
            more.push_str(&format!("; {air_in_it} of it are aircraft nothing in this group hits"));
        }
        // Under the water (the engine: a unit below the surface is seen by sonar only and hit by water weapons
        // only; a shooter under the surface hits only what is in the water): what the odds say of it.
        let submerged_in_it = enemies.iter().filter(|e| party.in_fight(e.id) && e.pos.y < SUBMERGED_Y).count();
        let ours_in_water = units.iter().filter(|u| u.pos.y <= 0.0).count();
        if submerged_in_it > 0 && submerged_in_it < party.fighters() && !self.force_can_hit_submerged(&ours) {
            more.push_str(&format!("; {submerged_in_it} of it {} under the water, where only torpedoes and depth charges reach, and this group has none", if submerged_in_it == 1 { "is" } else { "are" }));
        }
        let their_water_only = !identified.is_empty() && identified.iter().all(|(_, d, _)| self.world.def(*d).is_some_and(|x| x.water_only));
        if their_water_only && ours_in_water == 0 {
            more.push_str("; its weapons are torpedoes, which reach only what is in the water: it cannot hit this group on land");
        }
        if units.is_empty() {
            "we have nobody to send".to_string()
        } else if submerged_in_it > 0 && submerged_in_it == party.fighters() && !self.force_can_hit_submerged(&ours) {
            format!("we cannot hit it: it is under the water, where only torpedoes and depth charges reach, and this group has none{more}")
        } else if !self.force_can_hit(&ours, self.all_air(&theirs)) {
            format!("we cannot hit it: nothing in this group shoots at what it is{more}")
        } else if party.unarmed {
            // Not "it cannot hit us" and never "it outweighs us": a Lazarus taking an extractor apart read as no
            // threat, and one Rover sent at it as the weaker side (player-31, 6:21-6:52).
            format!("it is unarmed and cannot fight back{more}")
        } else if !self.force_can_hit(&theirs, self.all_air(&ours)) {
            format!("it cannot hit us: nothing there shoots at what this group is{more}")
        } else if ratio >= 2.5 {
            format!("we outweigh it heavily{reach_words}{more}")
        } else if ratio >= 1.3 {
            format!("we outweigh it{reach_words}{more}")
        } else if ratio >= NARROW {
            // A win, said as one: by the square law 1.1 to 1.3 leaves the winner 40 to 65% of its force, and the
            // simulator agrees (7 Blitzes on a Blitz under two Sentries, 1.24: 16 of 16 won, 53% left). Under
            // "an even fight" a raid of eight took its fall-back from 280 metal (player-50 6:17).
            format!("we outweigh it narrowly: a win that costs about half the group{reach_words}{more}")
        } else if ratio >= 0.8 {
            format!("an even fight{reach_words}{more}")
        } else {
            format!("it outweighs us{reach_words}{more}")
        }
    }

    /// The unseen shooter counted against a group under fire from out of sight (H-HANDS-ODDS-WHAT-SHOOTS): its
    /// reach against ours, and the metal verdict when the shooter is attributed to a known type (game 3, 19:30: a
    /// group under Bull fire read "we outweigh it heavily" of the four blips it could see).
    pub(super) fn unseen_shooter_words(&self, units: &[&OwnUnit], shelling: &super::super::shelling::Shelling) -> String {
        let our_reach = units.iter().map(|u| self.world.def(u.def).map_or(0.0, |d| d.reach)).fold(0.0, f32::max);
        let outranges = if shelling.range > our_reach + OUTRANGE_MARGIN { format!("; it outranges everything here ({:.0} against our {our_reach:.0}): nothing of ours answers it from where it stands", shelling.range) } else { format!("; within our reach ({our_reach:.0} against its {:.0}) once it is found", shelling.range) };
        let attributed = shelling.attributed.as_ref().and_then(|a| self.world.def_named(&a.name));
        match attributed {
            Some(def) => {
                let mut theirs = super::super::combat::Force::default();
                theirs.add(def);
                let ratio = self.odds(&Brain::force_of(units), &theirs);
                let verdict = if ratio >= 2.5 { "we outweigh it heavily" } else if ratio >= 1.3 { "we outweigh it" } else if ratio >= NARROW { "we outweigh it narrowly" } else if ratio >= 0.8 { "an even fight" } else { "it outweighs us" };
                format!("counted as one {} ({:.0} metal): {verdict}{outranges}", self.short_words(def), self.world.def(def).map_or(0.0, |d| d.metal_cost))
            }
            None => format!("of a type not seen, a {} with range {:.0}{outranges}", self.weapon_words(&shelling.weapon), shelling.range),
        }
    }

    /// A group's odds against a party, priced on the part of it that is near the party when the group is strung out
    /// (H-HANDS-GROUP-BODY): the core members within `NEAR` of the party, with words saying how many that is and
    /// where the rest are. A compact group is priced whole. Game 3, 8:35: 17 of 48 Blitzes were in B3 under two
    /// Beamers and five Welders while the odds were priced on all 48; game 9: 11-13 Pounders inside Bull reach while
    /// "we outweigh it heavily" was said of the centre.
    /// Returns the verdict of the priced part (as `odds_words` gives it, for the checks) and the full words.
    /// The allied seats' armed soldiers within `NEAR` of the party are said beside our odds, with the combined
    /// verdict ("with 12 allied soldiers of seat t2 (1,400 metal) beside it: together we outweigh it"), so "together"
    /// is a fact Jev can see (games 4, 6, 9: three armies that could not be one group).
    pub(super) fn group_odds(&self, body: &Body, party: &Party, enemies: &[EnemyUnit], allies: &[bot_protocol::AllyUnit]) -> (String, String) {
        let (verdict, mut words) = if !body.strung_out() {
            let v = self.odds_words(&body.core, party, enemies);
            (v.clone(), v)
        } else {
            let in_fight = body.within(party.at, NEAR);
            if in_fight.is_empty() || in_fight.len() == body.core.len() {
                let v = self.odds_words(&body.core, party, enemies);
                (v.clone(), v)
            } else {
                let behind = body.core.len() - in_fight.len();
                let farthest = body.core.iter().map(|u| u.pos.dist2d(party.at)).fold(0.0, f32::max);
                let verdict = self.odds_words(&in_fight, party, enemies);
                // The whole group's weight beside the near soldiers' (human-10, the D4 outpost: "for the 6 of its 51
                // soldiers near it, it outweighs us" over a group of 4,930 metal against 340 and four turrets, and the
                // rules' step-back sentence walked the column back on alternate seconds; offline with this clause and
                // the rule's exception the walk back was offered at the bar in 1 of 15 advancing moments against 7,
                // `docs/studies/2026-10-02-thrash-words.md`).
                let mut theirs = super::super::combat::Force::default();
                for enemy in enemies.iter().filter(|e| party.in_fight(e.id)) {
                    match enemy.def {
                        Some(def) => theirs.add(def),
                        None => theirs.unidentified += 1,
                    }
                }
                theirs.turret_metal += party.turret_metal;
                theirs.turret_metal_air += party.turret_metal_air;
                let ratio = self.odds(&Brain::force_of(&body.core), &theirs);
                let whole = if ratio >= 2.5 { "outweighs it heavily" } else if ratio >= 1.3 { "outweighs it" } else if ratio >= NARROW { "outweighs it narrowly" } else if ratio >= 0.8 { "matches it" } else { "is outweighed by it" };
                let metal: f32 = body.core.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
                let against = if party.turret_metal > 0.0 { format!("the party and the turrets covering it ({:.0} and {:.0}, the turrets counted three times: {:.0})", party.metal, party.turret_metal, party.metal + 3.0 * party.turret_metal) } else { format!("the party ({:.0})", party.metal) };
                let words = format!("for the {} of its {} soldiers near it, {verdict}; the whole group, {} soldiers worth {metal:.0} metal, against {against} {whole} ({ratio:.1} to 1); the other {behind} are {:.0}-{farthest:.0} behind and not near it yet", in_fight.len(), body.core.len(), body.core.len(), NEAR);
                (verdict, words)
            }
        };
        let beside: Vec<&bot_protocol::AllyUnit> = allies.iter().filter(|a| !a.being_built && a.pos.dist2d(party.at) <= NEAR && self.world.def(a.def).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0)).collect();
        if !beside.is_empty() {
            let mut ours = Brain::force_of(&body.within(party.at, NEAR.max(body.length)));
            for a in &beside {
                ours.add(a.def);
            }
            let mut theirs = super::super::combat::Force::default();
            for enemy in enemies.iter().filter(|e| party.in_fight(e.id)) {
                match enemy.def {
                    Some(def) => theirs.add(def),
                    None => theirs.unidentified += 1,
                }
            }
            theirs.turret_metal += party.turret_metal;
            theirs.turret_metal_air += party.turret_metal_air;
            let ratio = self.odds(&ours, &theirs);
            let together = if ratio >= 2.5 { "together we outweigh it heavily" } else if ratio >= 1.3 { "together we outweigh it" } else if ratio >= NARROW { "together we outweigh it narrowly" } else if ratio >= 0.8 { "together an even fight" } else { "even together it outweighs us" };
            let metal: f32 = beside.iter().filter_map(|a| self.world.def(a.def)).map(|d| d.metal_cost).sum();
            let seats: BTreeSet<i32> = beside.iter().map(|a| a.team).collect();
            words.push_str(&format!("; with {} allied soldiers of seat {} ({metal:.0} metal) within reach of it: {together}", beside.len(), seats.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(" and ")));
        }
        (verdict, words)
    }

    /// The enemy's armed buildings whose reach covers `at`, with a margin for a party's spread: the metal of those
    /// that hit ground, of those that hit air, and words for the lines. Standing buildings are remembered where
    /// they were seen, so a nest counts before it is in sight again.
    pub(super) fn turrets_covering(&self, at: Vec3) -> (f32, f32, String) {
        let mut ground = 0.0;
        let mut air = 0.0;
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for (id, (def, pos, _)) in &self.enemy_buildings {
            let Some(d) = self.world.def(*def) else { continue };
            if d.weapon_count == 0 || d.reach <= 0.0 || pos.dist2d(at) > d.reach + TURRET_MARGIN || self.enemy_unfinished.contains(id) {
                continue;
            }
            if self.can_hit(*def, false) {
                ground += d.metal_cost;
            }
            if self.can_hit(*def, true) {
                air += d.metal_cost;
            }
            *counts.entry(self.name(*def)).or_default() += 1;
        }
        let n: usize = counts.values().sum();
        let words = if n == 0 { String::new() } else { format!("{n} turret{}: {}", if n == 1 { "" } else { "s" }, counts.iter().map(|(k, v)| format!("{v} {k}")).collect::<Vec<_>>().join(", ")) };
        (ground, air, words)
    }

    /// A party as a group that is not at it meets it (K-hands-a-partys-turrets-were-counted-against-a-group-out-of-
    /// their-reach): the turrets covering the party's place count in the group's fight only when one of them
    /// reaches a soldier of the group where it stands. When none does, the party comes back without them and
    /// with words saying where they are; the moves that go to the party (attack, shell, a detachment) keep them.
    pub(super) fn party_where_we_stand(&self, party: &Party, units: &[&OwnUnit]) -> (Party, String) {
        if party.turrets.is_empty() || units.is_empty() {
            return (party.clone(), String::new());
        }
        let mut nearest = f32::INFINITY;
        for (id, (def, pos, _)) in &self.enemy_buildings {
            let Some(d) = self.world.def(*def) else { continue };
            if d.weapon_count == 0 || d.reach <= 0.0 || pos.dist2d(party.at) > d.reach + TURRET_MARGIN || self.enemy_unfinished.contains(id) {
                continue;
            }
            let to_us = units.iter().map(|u| u.pos.dist2d(*pos)).fold(f32::INFINITY, f32::min);
            if to_us <= d.reach + TURRET_MARGIN {
                return (party.clone(), String::new());
            }
            nearest = nearest.min(to_us);
        }
        if !nearest.is_finite() {
            return (party.clone(), String::new());
        }
        let here = Party { turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new(), ..party.clone() };
        let away = format!("; the {} covering it stand where it is now, {} from this group, and shoot only what goes there", party.turrets, (nearest / 50.0).round() * 50.0);
        (here, away)
    }

    /// For a group hunting a quarry or attacking a party (H-HANDS-CHASE): his armed buildings whose reach the line
    /// from the group's front to the quarry enters within `CHASE_AHEAD` of the front, and the parties within `NEAR`
    /// of its soldiers that are not the one chased, each with the odds; None on any other course or when nothing is
    /// met. The hunt itself ends only on a kill, six seconds out of sight or the leash (`micro`), and the whole-group
    /// attack follows its party with no leash: the withdrawal is Jev's pick on these words and the event
    /// (`schedule.rs`) that asks for it (the user, 2026-10-17: "a hunting party is able to be recalled by jev when
    /// it encounters something unexpected"; predict-sweep-1 seed 8 7:58: a detachment of four followed a raider
    /// into two Pounders; sightings-player-1 6:23: Blitzes sent after single Pawns died under the turret they ran to).
    pub(super) fn chase_met(&self, group: &super::Group, own: &[OwnUnit], parties: &[Party], places: &[Place]) -> Option<ChaseMet> {
        let (what, target, party_name) = match &group.task {
            GroupTask::Hunt(h) => ("hunting", h.at, h.party.clone()),
            GroupTask::Engage { party, at, .. } => ("attacking", *at, parties.iter().find(|p| p.ids.iter().any(|id| party.contains(id))).map_or(String::new(), |p| p.name.clone())),
            _ => return None,
        };
        let units = group.units(own);
        let body = group.body(own, Some(target))?;
        let front = body.front;
        let (dx, dz) = (target.x - front.x, target.z - front.z);
        let len = dx.hypot(dz);
        let (ux, uz) = if len > 1.0 { (dx / len, dz / len) } else { (0.0, 0.0) };
        let mut keys = Vec::new();
        let mut parts = Vec::new();
        let mut turrets: Vec<(f32, String)> = Vec::new();
        let air = group.domain == crate::world::Domain::Air;
        for (id, (def, pos, _)) in &self.enemy_buildings {
            let Some(d) = self.world.def(*def) else { continue };
            if d.weapon_count == 0 || d.reach <= 0.0 || self.enemy_unfinished.contains(id) || !self.can_hit(*def, air) {
                continue;
            }
            let reach = d.reach + TURRET_MARGIN;
            let (bx, bz) = (pos.x - front.x, pos.z - front.z);
            let along = (bx * ux + bz * uz).clamp(0.0, len.max(0.0));
            let (cx, cz) = (front.x + ux * along, front.z + uz * along);
            let nearest = (pos.x - cx).hypot(pos.z - cz);
            if nearest > reach {
                continue;
            }
            // Where the line enters the reach: back from its closest point by the half-chord.
            let half = (reach * reach - nearest * nearest).max(0.0).sqrt();
            let entry = (along - half).max(0.0);
            if entry > CHASE_AHEAD {
                continue;
            }
            let inside = front.dist2d(*pos) <= reach;
            keys.push(format!("turret {}", id.0));
            let begins = if inside { "the front is inside its reach now".to_string() } else { format!("its reach begins {entry:.0} ahead of the front") };
            turrets.push((entry, format!("his {} at {} (reach {:.0}), {begins}", self.short_words(*def), self.place_words(places, *pos), d.reach)));
        }
        turrets.sort_by(|a, b| a.0.total_cmp(&b.0));
        if !turrets.is_empty() {
            parts.push(format!("ahead on the quarry's line: {}", turrets.iter().map(|(_, w)| w.as_str()).collect::<Vec<_>>().join("; ")));
        }
        let ours: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let mut beside = Vec::new();
        for p in parties.iter().filter(|p| p.name != party_name) {
            let d = units.iter().map(|u| u.pos.dist2d(p.at)).fold(f32::INFINITY, f32::min);
            if d > NEAR {
                continue;
            }
            let theirs = p.metal + p.turret_metal * 3.0;
            let verdict = if theirs > ours * 1.2 { "it outweighs this group" } else if ours > theirs * 1.2 { "this group outweighs it" } else { "an even fight" };
            keys.push(p.name.clone());
            beside.push(format!("{} ({}, {:.0} metal{}) {d:.0} from the nearest soldier, {verdict}", p.name, p.composition, p.metal, if p.turrets.is_empty() { String::new() } else { format!(", under {}", p.turrets) }));
        }
        if !beside.is_empty() {
            parts.push(format!("beside the chase: {}", beside.join("; ")));
        }
        if parts.is_empty() {
            return None;
        }
        let quarry = if party_name.is_empty() { "its party".to_string() } else { party_name };
        let leash = match &group.task {
            GroupTask::Hunt(h) => {
                // The quarry's run from where it stood when the hunt began, of the leash that lets it go.
                let run = h.at.dist2d(h.from);
                format!(", its quarry {run:.0} of the {:.0} it is let go at", super::groups::HUNT_LEASH)
            }
            _ => String::new(),
        };
        Some(ChaseMet { words: format!("{what} {quarry} ({len:.0} ahead of the front){leash}; {}; any other pick ends the chase", parts.join("; ")), keys })
    }

    /// Enemy parties at our extractors, for a group's entry: which spot, how far from the group.
    pub(super) fn threats_words(&self, parties: &[Party], places: &[Place], own: &[OwnUnit], kit: &Kit, from: Vec3) -> Vec<String> {
        parties
            .iter()
            .filter_map(|p| {
                let extractor = own.iter().filter(|u| kit.is_extractor(u.def) && u.pos.dist2d(p.at) < NEAR).min_by(|a, b| a.pos.dist2d(p.at).total_cmp(&b.pos.dist2d(p.at)))?;
                Some(format!("{} ({}) at our extractor at {}, {} ({:.0}) from this group", p.name, p.composition, self.place_words(places, extractor.pos), distance_words(p.at.dist2d(from)), p.at.dist2d(from)))
            })
            .collect()
    }

    fn party_words(&self, parties: &[Party], pos: Vec3) -> Option<String> {
        parties
            .iter()
            .map(|p| (p.at.dist2d(pos), p))
            .filter(|(d, _)| *d < NEAR)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(d, p)| format!("{} ({}) {:.0} away", p.name, p.composition, d))
    }

    fn task_words(&self, task: Option<&Task>, unit: &OwnUnit, places: &[Place], frame: i32, own: &[OwnUnit]) -> String {
        let ago = |since: i32| format!("{} s ago", (frame - since) / FRAMES_PER_SECOND);
        let mut words = self.task_course(task, unit, places, frame, own);
        // A builder whose moves the engine gives up says so, as a group's soldiers do (pace-1: constructor_9823 read
        // "walking to build ... 951 to go" for four minutes wedged behind the first plant).
        if let Some(stuck) = self.stuck.get(&unit.id) {
            words.push_str(&format!("; stuck: it cannot move from {}, its moves have failed since {}", self.place_words(places, stuck.at), ago(stuck.since)));
        }
        // Standing in a factory's exit lane (a builder working from the pad): the factory's units cannot leave
        // (wake-3: the plant made nothing from 4:47 to 7:45 behind a constructor building nano turrets on its pad).
        if let Some((since, factory)) = self.lane_stander(unit.id, frame) {
            words.push_str(&format!("; standing in {}'s exit lane since {}: its units cannot leave while it stands there", self.actor_name(factory), ago(since)));
        }
        words
    }

    pub(super) fn task_course(&self, task: Option<&Task>, unit: &OwnUnit, places: &[Place], frame: i32, own: &[OwnUnit]) -> String {
        // No "since" or "ordered N s ago" on a task (2026-10-24: the policy of t-1 does not enter the words); the
        // frame's percentage and the standing-at-the-site clock are the state of the build, and stay.
        match task {
            None if unit.idle => "idle, waiting for an order".into(),
            None => "finishing an order".into(),
            Some(Task::Build { def, near, started, frame: own_frame, .. }) => {
                let what = self.short_words(*def);
                let walk = unit.pos.dist2d(*near);
                if *started {
                    // How far along, from the builder's own frame (games 7, 9: "armavp ordered 200 s ago" with no
                    // percentage was believed standing at 14:45 and finished at 16:12; by type within 200 the entry
                    // showed a neighbour's 98 % for a frame at 8: the unfinished-structures sweep, 2026-10-17).
                    let frame_unit = own_frame.and_then(|f| own.iter().find(|u| u.id == f));
                    let percent = frame_unit.map(|u| u.build_progress * 100.0);
                    // Its name once it stands, so a list or a `produce` for it can be staged now.
                    let named = frame_unit.and_then(|u| self.future_name(u)).map_or(String::new(), |n| format!("; it will be {n}: a `queue` or `produce` for it now waits for it to stand"));
                    format!("building a {what} at {} ({}){named}", self.place_words(places, *near), percent.map_or("its frame is gone".to_string(), |p| format!("{p:.0} % built")))
                } else if walk > 120.0 {
                    format!("walking to build a {what} at {}, {walk:.0} to go", self.place_words(places, *near))
                } else if let Some(since) = self.pianist.as_ref().and_then(|p| p.at_site.get(&unit.id).copied()).filter(|s| frame - s >= 3 * FRAMES_PER_SECOND) {
                    format!("standing at the site of a {what} at {} for {} s with nothing started", self.place_words(places, *near), (frame - since) / FRAMES_PER_SECOND)
                } else {
                    format!("about to build a {what} at {}", self.place_words(places, *near))
                }
            }
            Some(Task::Assist { lab, .. }) => format!("helping lab_{} build", lab.0),
            Some(Task::Reclaim { at, .. }) => format!("taking apart wrecks at {}", self.place_words(places, *at)),
            Some(Task::ReclaimUnit { target, .. }) => format!("taking apart our {} on the player's order", self.known_units.get(target).map_or("unit".to_string(), |(def, _)| format!("{}_{}", self.name(*def), target.0))),
            Some(Task::Repair { target, .. }) => format!("repairing our {}", self.known_units.get(target).map_or("unit", |(def, _)| self.name(*def))),
            // The engine has dropped its order when the unit stands idle short of the place: said here until the
            // housekeeping ends the task.
            Some(Task::Walk { place, to, .. }) => format!("walking to {place}, {:.0} to go", unit.pos.dist2d(*to)),
            Some(Task::Follow { group, .. }) => format!("following group_{group}"),
            Some(Task::Attack { party, to, .. }) => format!("attacking {party}, {:.0} from it", unit.pos.dist2d(*to)),
        }
    }

    /// The whole picture for this second.
    pub(super) fn picture(&self, tick: &Tick, kit: &Kit) -> Picture {
        let pianist = self.pianist.as_ref().expect("pianist mode");
        let snapshot = &tick.snapshot;
        let own = &snapshot.own_units;
        let frame = tick.frame;
        let spots = &self.world.hello.metal_spots;

        // Places.
        let mut places: Vec<Place> = vec![Place { name: "home".into(), at: self.home, spot: None, cells: None }];
        let extractor_at = |spot: Vec3| own.iter().find(|u| kit.is_extractor(u.def) && u.pos.dist2d(spot) < self.spot_occupied_radius());
        let taken_by_task: Vec<usize> = pianist.tasks.values().filter_map(|t| if let Task::Build { spot: Some(i), .. } = t { Some(*i) } else { None }).collect();
        let mut listed: Vec<usize> = Vec::new();
        for (i, spot) in spots.iter().enumerate() {
            if extractor_at(*spot).is_some() || taken_by_task.contains(&i) {
                listed.push(i);
            }
        }
        let enemy_extractors: Vec<(Vec3, i32)> = self
            .enemy_buildings
            .values()
            .filter(|(def, _, _)| self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0))
            .map(|(_, pos, seen)| (*pos, *seen))
            .collect();
        let their_spot = |spot: Vec3| enemy_extractors.iter().find(|(pos, _)| pos.dist2d(spot) < self.spot_occupied_radius()).map(|(_, seen)| *seen);
        let mut free: Vec<(f32, usize)> = spots
            .iter()
            .enumerate()
            .filter(|(i, s)| !listed.contains(i) && their_spot(**s).is_none() && self.reachable_by_any_class(**s) && self.spot_open_to_us(*i, **s, frame))
            .map(|(i, s)| (self.walk_from_home(*s), i))
            .collect();
        free.sort_by(|a, b| a.0.total_cmp(&b.0));
        listed.extend(free.iter().take(FREE_SPOTS).map(|(_, i)| *i));
        let mut theirs: Vec<(f32, usize)> = spots.iter().enumerate().filter(|(_, s)| their_spot(**s).is_some()).map(|(i, s)| (s.dist2d(self.home), i)).collect();
        theirs.sort_by(|a, b| a.0.total_cmp(&b.0));
        listed.extend(theirs.iter().take(ENEMY_SPOTS).map(|(_, i)| *i));
        for i in &listed {
            places.push(Place { name: format!("spot_{i}"), at: spots[*i], spot: Some(*i), cells: None });
        }
        // Every spot with buildings of his within reach of it is a place, so that an attack can be aimed at all of
        // them (the list above holds the nearest few of his extractors only).
        for (_, pos, _) in self.enemy_buildings.values() {
            if places.iter().any(|p| p.at.dist2d(*pos) < super::menu::HIS_AT) {
                continue;
            }
            if let Some((i, spot)) = spots.iter().enumerate().map(|(i, s)| (i, *s)).filter(|(_, s)| s.dist2d(*pos) < super::menu::HIS_AT).min_by(|a, b| a.1.dist2d(*pos).total_cmp(&b.1.dist2d(*pos))) {
                places.push(Place { name: format!("spot_{i}"), at: spot, spot: Some(i), cells: None });
            }
        }
        let passages = self.passages();
        for (n, passage) in passages.iter().take(PASSAGES).enumerate() {
            places.push(Place { name: format!("passage_{}{}", n + 1, self.seat_tag()), at: passage.at, spot: None, cells: None });
        }
        // H-HANDS-NAMED-PLACES: every spot and passage the instructions name is a place, however far (pianist-player-5:
        // the player named spot_36 in the south for four turns and it was never on the menu, the list being the
        // nearest free spots and the nearest of theirs), and so is every place the player marked.
        // The player's packet, else the fixed one from a file (`--packet`), else the hands' default opening.
        let instructions = self
            .strategist
            .as_ref()
            .map(|s| s.instructions.lock().unwrap().clone())
            .filter(|i| !i.trim().is_empty())
            .or_else(|| self.pianist.as_ref().and_then(|p| p.packet.clone()))
            .unwrap_or_else(|| crate::texts::read(&crate::texts::HANDS_DEFAULT));
        // Spots in runs ("spot_10, 5, 6") and spots the lists name are places too (13.2, 7.4).
        let mut extra_spots: Vec<usize> = super::diet::spot_runs(&instructions);
        for steps in pianist.scripts.values() {
            for step in steps {
                if let Some(i) = step.split_whitespace().nth(1).and_then(|p| p.strip_prefix("spot_")).and_then(|n| n.parse::<usize>().ok()) {
                    extra_spots.push(i);
                }
            }
        }
        for i in extra_spots {
            if i < spots.len() && !places.iter().any(|p| p.spot == Some(i)) && self.reachable_by_any_class(spots[i]) {
                places.push(Place { name: format!("spot_{i}"), at: spots[i], spot: Some(i), cells: None });
            }
        }
        for token in instructions.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            if let Some(i) = token.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                // An islet spot nobody can walk to is no place to send anyone (pianist-player-7: the ball stood 151 s
                // short of spot_31 on the shore while the player named it).
                if i < spots.len() && !places.iter().any(|p| p.spot == Some(i)) && self.reachable_by_any_class(spots[i]) {
                    places.push(Place { name: format!("spot_{i}"), at: spots[i], spot: Some(i), cells: None });
                }
            } else if let Some(n) = token.strip_prefix("passage_").and_then(|n| n.parse::<usize>().ok()) {
                if n > PASSAGES && n <= passages.len() && !places.iter().any(|p| p.name == *token) {
                    places.push(Place { name: format!("passage_{n}{}", self.seat_tag()), at: passages[n - 1].at, spot: None, cells: None });
                }
            }
        }
        let marks: BTreeMap<String, crate::strategist::shared::Mark> = self.strategist.as_ref().map(|s| s.marks_all()).unwrap_or_default();
        for (name, m) in &marks {
            if !places.iter().any(|p| p.name == *name) {
                places.push(Place { name: name.clone(), at: Vec3 { x: m.x, y: 0.0, z: m.z }, spot: None, cells: m.cells });
            }
        }
        // H-HANDS-SHELLED: where fire from out of sight likeliest comes from, while it lasts: the side's estimate as
        // `shelling`, and a group's own (from the hits on its members) as `shelling_<group>` when it stands apart
        // from the side's (player-9 21:15: one estimate, a Beamer at E4, was said to every group on the map).
        let shelling = self.shelling();
        if let Some(s) = &shelling {
            places.push(Place { name: "shelling".into(), at: s.at, spot: None, cells: None });
        }
        // His guns out of sight as standing places (`guns.rs`, H-GUNS): a packet can name them.
        for gun in &self.guns {
            places.push(Place { name: gun.name.clone(), at: gun.at, spot: None, cells: None });
        }
        let mut group_shelling: BTreeMap<String, (super::super::shelling::Shelling, String)> = BTreeMap::new();
        if let Some(p) = self.pianist.as_ref() {
            for g in &p.groups {
                let Some(own_shelling) = self.shelling_for(&g.members) else { continue };
                let apart = shelling.as_ref().is_none_or(|s| s.at.dist2d(own_shelling.at) > SHOOTER_APART);
                let place_name = if apart { format!("shelling_{}", g.name) } else { "shelling".to_string() };
                if apart {
                    places.push(Place { name: place_name.clone(), at: own_shelling.at, spot: None, cells: None });
                }
                group_shelling.insert(g.name.clone(), (own_shelling, place_name));
            }
        }
        let parties = match self.pianist.as_ref() {
            Some(p) => self.enemy_parties(&snapshot.enemies, &p.parties, &p.next_party, p.layers.kept, p.layers.beside),
            None => self.enemy_parties(&snapshot.enemies, &[], &std::cell::Cell::new(1), false, false),
        };

        let mut place_entries: BTreeMap<String, Value> = BTreeMap::new();
        for place in &places {
            let mut entry = json!({ "grid": self.world.grid(place.at) });
            if let Some(cells) = place.cells {
                entry["grid"] = json!(format!("the cells {} (its point the centre, {})", crate::strategist::shared::cells_words(cells), self.world.grid(place.at)));
            }
            let walk = self.walk_from_home(place.at);
            let straight = place.at.dist2d(self.home);
            // An alcove: far longer on foot than as the crow flies (9b.4; Cape Violet's spot_18: 6,838 on foot
            // against 1,605, chosen as a gather point twice).
            let alcove = if straight > 300.0 && walk > 1.8 * straight { format!("; an alcove or a pocket: {walk:.0} on foot against {straight:.0} straight, it opens from the far side") } else { String::new() };
            entry["from_home"] = json!(format!("{} ({walk:.0} on foot){alcove}", distance_words(walk)));
            let what = match place.spot {
                Some(i) => {
                    let spot = spots[i];
                    // The extractor on the spot itself (within its radius) is what takes it; the buildings beside it
                    // are said separately (the user, 2026-09-22: a starting spot never taken, every game: the
                    // neighbouring spot's extractor 300 away read as "our extractor" here, and the spot was never free).
                    let on_spot = own.iter().find(|u| kit.is_extractor(u.def) && u.pos.dist2d(spot) < self.spot_occupied_radius());
                    let beside: Vec<String> = own
                        .iter()
                        .filter(|u| !kit.is_extractor(u.def) && u.pos.dist2d(spot) < 200.0 && self.world.def(u.def).is_some_and(|d| d.speed == 0.0))
                        .map(|u| format!("our {}{}", self.short_words(u.def), if u.being_built { " (being built)" } else { "" }))
                        .collect();
                    let mut beside_words = if beside.is_empty() { String::new() } else { format!(" (beside it: {})", beside.join(", ")) };
                    // Under the water and off our ground are different jobs: game 12 on SailAway 2 sent construction
                    // ships for islet spots that read "under water or off our ground".
                    if !self.reachable_on_foot(spot) {
                        beside_words.push_str(if self.world.under_water(spot) {
                            "; under water: an amphibious or hover constructor, or a construction ship, can take it; ordinary constructors cannot"
                        } else {
                            "; on ground our walkers cannot reach (an islet, a cliff top): a hover or amphibious constructor gets there; a construction ship only to its shore"
                        });
                    }
                    if let Some(seen) = their_spot(spot) {
                        let turrets = self.enemy_buildings.values().filter(|(def, pos, _)| pos.dist2d(spot) < 500.0 && self.world.def(*def).is_some_and(|d| d.weapon_count > 0)).count();
                        format!("their extractor, seen {} ago{}", clock(frame - seen), if turrets > 0 { format!(", {turrets} turret(s) beside it") } else { String::new() })
                    } else if let Some(u) = on_spot {
                        format!("our extractor{}{beside_words}", if u.being_built { " (being built)" } else { "" })
                    } else {
                        let taker = pianist.tasks.iter().find_map(|(id, t)| matches!(t, Task::Build { spot: Some(s), .. } if *s == i).then_some(*id));
                        let ally_taker = self.team_mates.spot_claims.contains(&i);
                        match (taker, self.spot_seen(i)) {
                            (Some(id), _) => format!("free metal spot, {} is on its way to take it{beside_words}", self.actor_name(id)),
                            (None, _) if ally_taker => format!("free metal spot, a builder of another seat of ours is on its way to take it{beside_words}"),
                            (None, None) => format!("metal spot never in our sight: nobody knows what stands here{beside_words}"),
                            (None, Some(seen)) if frame - seen > LONG_UNSEEN => format!("free metal spot when last in sight, {} ago{beside_words}", clock(frame - seen)),
                            (None, Some(_)) => format!("free metal spot{beside_words}"),
                        }
                    }
                }
                None if place.name == "home" => "our start: the lab and the base stand here".into(),
                None if place.name.starts_with("shelling") => {
                    let (s, whom) = match place.name.strip_prefix("shelling_") {
                        Some(g) => (&group_shelling.get(g).expect("a group's shelling place has its shelling").0, format!("group_{g}")),
                        None => (shelling.as_ref().expect("a shelling place has a shelling"), "us".to_string()),
                    };
                    // An estimate, said as one: the old words ("advancing a group onto it kills it") sent groups under blind
                    // fire toward a moving point for two minutes at a time (escalate-5 12:29, escalate-7 12:29, fixes-1 15:33).
                    match &s.attributed {
                        Some(a) => format!("where the {} whose fire hits {whom} from out of our sight was last seen, {} s ago (not a sighting now: it may have moved); its range is {:.0}, {} hits on {whom} in the last {} s, the last {} s ago; anything of ours within {:.0} of it is in its reach; what sees it or outranges it decides, as the instructions say", a.name, (frame - a.seen) / super::super::FRAMES_PER_SECOND, s.range, s.hits, super::super::shelling::SHELL_MEMORY / super::super::FRAMES_PER_SECOND, (frame - s.last) / super::super::FRAMES_PER_SECOND, s.range),
                        None => format!("an estimate, not a sighting: where the {} shelling {whom} from out of our sight likeliest stands, four fifths of its range along the hits' direction (it moves with each hit); its range is {:.0}, {} hits on {whom} in the last {} s from the {}, the last {} s ago; anything of ours within {:.0} of it is in its reach and cannot see it; what sees it (a scout, a radar) or outranges it decides, as the instructions say", self.weapon_words(&s.weapon), s.range, s.hits, super::super::shelling::SHELL_MEMORY / super::super::FRAMES_PER_SECOND, super::super::shelling::compass(s.dir), (frame - s.last) / super::super::FRAMES_PER_SECOND, s.range),
                    }
                }
                None if place.cells.is_some() => {
                    let (a, b) = self.world.cells_rect(place.cells.unwrap());
                    let inside = spots.iter().filter(|s| s.x >= a.x && s.x <= b.x && s.z >= a.z && s.z <= b.z).count();
                    format!("a region the player marked ({inside} metal spots inside): a group told to rove within it (`rove_{}`, `rove_1_{}`) looks only at places inside it; as a place to go to, its centre", place.name, place.name)
                }
                None if marks.contains_key(&place.name) => {
                    if self.reachable_on_foot(place.at) { "a place the player marked".into() } else { "a place the player marked; our ground units cannot get there from home (water or a cliff: aircraft can, hovercraft over water can); a ground group sent here stops at the nearest ground it can reach".into() }
                }
                None => "a narrow passage between the two sides".into(),
            };
            entry["what"] = json!(what);
            if let Some(words) = self.party_words(&parties, place.at) {
                entry["enemies_near"] = json!(words);
            }
            // Its buildings remembered within 500: what a group sent here meets (docs/design/2026-09-22-enemy-evidence.md, decision 4).
            let mut near: BTreeMap<String, usize> = BTreeMap::new();
            let mut oldest = frame;
            for (id, (def, pos, seen)) in &self.enemy_buildings {
                if pos.dist2d(place.at) < 500.0 {
                    let unfinished = if self.enemy_unfinished.contains(id) { " (being built)" } else { "" };
                    // A shield's bubble and what of ours it stops (H-REPORT-SHIELDS); a cloaking type is said so.
                    let shield = self.shield_words(*def, own).map_or(String::new(), |w| format!(" ({w})"));
                    let cloaks = if self.world.def(*def).is_some_and(|d| d.cloak.is_some()) { " (cloaks: hidden until ours stand beside it)" } else { "" };
                    *near.entry(format!("{}{unfinished}{shield}{cloaks}", self.short_words(*def))).or_default() += 1;
                    oldest = oldest.min(*seen);
                }
            }
            if !near.is_empty() {
                let turrets = self.enemy_buildings.iter().filter(|(id, (def, pos, _))| pos.dist2d(place.at) < 500.0 && !self.enemy_unfinished.contains(id) && self.world.def(*def).is_some_and(|d| d.weapon_count > 0)).count();
                entry["their_buildings_near"] = json!(format!("{} ({} armed; last seen {} ago)", near.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), turrets, clock(frame - oldest)));
            }
            if let Some(i) = place.spot
                && let Some(lost) = self.spot_losses.get(&i)
            {
                entry["raided"] = json!(format!("we lost an extractor here {lost} time(s)"));
            }
            place_entries.insert(place.name.clone(), entry);
        }

        // Economy and what we have. The energy budget: what our factories and builders would draw at full speed
        // (docs/design/2026-09-22-energy-draw.md), so the size of a stall is a number before it is a stall.
        let m = &snapshot.metal;
        let e = &snapshot.energy;
        let draws = self.production_draws(own, pianist);
        let draw_total: f32 = draws.iter().map(|d| d.energy).sum();
        let budget = if draws.is_empty() {
            String::new()
        } else {
            let short = draw_total - e.income;
            let mut fixes: Vec<String> = Vec::new();
            // Each generator's energy a second from its unit file (`energymake`; the advanced solars 80/80/100, the fusions
            // 750/850/950: the 75 and 1000 written here until 2026-10-14 were wrong).
            for (name, unit_income) in [("armsolar", 20.0), ("corsolar", 20.0), ("legsolar", 20.0), ("armadvsol", 80.0), ("coradvsol", 80.0), ("legadvsol", 100.0), ("armfus", 750.0), ("corfus", 850.0), ("legfus", 950.0)] {
                if let Some(def) = self.world.def_named(name) && self.faction_reaches(def) {
                    fixes.push(format!("{:.1} {}", short.max(0.0) / unit_income, self.short_words(def)));
                }
            }
            format!(" Our production at full speed would draw {draw_total:.0} a second ({}); against the income that is {}", draws.iter().map(|d| format!("{} {:.0}", d.who, d.energy)).collect::<Vec<_>>().join(", "), if short > 0.0 { format!("short by {short:.0}, which is {}", fixes.join(" or ")) } else { format!("{:.0} to spare", -short) }) + "."
        };
        let economy = json!({
            "metal": format!("{:.0} of {:.0} stored ({}); {:.1} a second coming in, {:.1} going out: {}", m.current, m.storage, stock_words(m.current, m.storage), m.income, m.usage, flow_words(m.income, m.usage, m.current, m.storage)),
            "energy": format!("{:.0} of {:.0} stored ({}); {:.0} a second coming in, {:.0} going out: {}.{budget} The wind now: {:.0} of this map's {:.0} to {:.0} ({})", e.current, e.storage, stock_words(e.current, e.storage), e.income, e.usage, flow_words(e.income, e.usage, e.current, e.storage), snapshot.wind, self.world.hello.map.wind_min, self.world.hello.map.wind_max, if snapshot.wind < 6.0 { "weak: wind generators give little at the moment" } else { "blowing: wind generators pay" }),
        });
        let count = |f: &dyn Fn(&OwnUnit) -> bool| own.iter().filter(|u| !u.being_built && f(u)).count();
        let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
        let army_metal: f32 = soldiers.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let mut wreck_fields: Vec<String> = self
            .reclaim
            .fields
            .iter()
            .filter(|f| f.metal >= 100.0)
            .take(WRECK_FIELDS)
            .map(|f| format!("{:.0} metal of wrecks at {}{}", f.metal, self.place_words(&places, f.at), if f.safe { "" } else { " (not safe)" }))
            .collect();
        let (all_wrecks, safe_wrecks): (f32, f32) = self.reclaim.fields.iter().fold((0.0, 0.0), |(a, s), f| (a + f.metal, s + if f.safe { f.metal } else { 0.0 }));
        if all_wrecks >= 100.0 {
            wreck_fields.insert(0, format!("{all_wrecks:.0} metal of wrecks known, {safe_wrecks:.0} of it on ground we hold"));
        }
        let crew: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && kit.is_resurrector(u.def)).collect();
        let crew_words = if crew.is_empty() {
            format!("none (`produce` {}: they raise wrecked soldiers and take the rest apart on their own)", self.name(kit.resurrector))
        } else {
            format!("{}, {} working a wreck field, {} idle", crew.len(), crew.iter().filter(|u| !u.idle).count(), crew.iter().filter(|u| u.idle).count())
        };
        let extractors = count(&|u| self.world.is_extractor_def(u.def));
        let constructors = count(&|u| self.world.is_constructor_def(u.def));
        // The silos and anti-nukes with their rounds: the game fills each to its kept count on its own, a silo's
        // missile drawing its energy over its stockpile time (K-rules-a-stockpile-fills-itself-to-its-limit); the
        // player's `fire` tool sends a ready missile at a place.
        let stockpilers: Vec<String> = own
            .iter()
            .filter(|u| !u.being_built)
            .filter_map(|u| {
                let d = self.world.def(u.def).filter(|d| d.stockpile_seconds > 0.0)?;
                let (kind, round) = if d.coverage > 0.0 { (format!("anti-nuke, takes a missile aimed within {:.0} of it", d.coverage), "interceptor") } else { ("nuclear silo".to_string(), "missile") };
                let piling = if u.stockpile_queued > 0 {
                    format!(", {} stockpiling (one every {:.0} s at full energy, drawing about {:.0} energy a second while it does)", u.stockpile_queued, d.stockpile_seconds, d.stockpile_energy / d.stockpile_seconds.max(1.0))
                } else {
                    String::new()
                };
                let fire = if d.coverage == 0.0 && u.stockpile > 0 { "; `fire` sends one at a place" } else { "" };
                Some(format!("{} at {} ({kind}): {} {round}{} ready{piling}{fire}", self.handle(u), self.place_words(&places, u.pos), u.stockpile, if u.stockpile == 1 { "" } else { "s" }))
            })
            .collect();
        let ours = json!({
            "extractors": extractors,
            "labs": count(&|u| self.world.is_factory_def(u.def)),
            "constructors": constructors,
            "turrets": count(&|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && d.weapon_count > 0)),
            "radars": count(&|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && d.radar_range > 0.0)),
            "generators": count(&|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && (d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.wind_cap > 0.0))),
            "soldiers": format!("{}: {} worth {:.0} metal ({})", soldier_words(soldiers.len(), army_metal), soldiers.len(), army_metal.max(0.0), self.composition_words(&soldiers)),
            "wrecks": wreck_fields,
            "resurrection_bots": crew_words,
            "stockpilers": stockpilers,
        });

        // The enemy.
        let in_sight: Vec<String> = parties
            .iter()
            .map(|p| {
                let heading = {
                    let vel: Vec3 = snapshot.enemies.iter().filter(|e| p.ids.contains(&e.id)).fold(Vec3::default(), |s, e| Vec3 { x: s.x + e.vel.x, y: 0.0, z: s.z + e.vel.z });
                    let speed = vel.x.hypot(vel.z);
                    if speed < 0.3 {
                        "standing still"
                    } else {
                        let toward_home = (self.home.x - p.at.x) * vel.x + (self.home.z - p.at.z) * vel.z > 0.0;
                        if toward_home { "moving toward our side" } else { "moving away from us" }
                    }
                };
                let near_ours = own
                    .iter()
                    .filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0) || self.world.is_mobile_builder(u.def))
                    .map(|u| (u.pos.dist2d(p.at), u))
                    .filter(|(d, _)| *d < NEAR)
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|(d, u)| format!("; {d:.0} from our {}", self.name(u.def)));
                let under = if p.turrets.is_empty() { String::new() } else { format!(" under {} ({:.0} metal)", p.turrets, p.turret_metal) };
                let beside = if p.beside_words.is_empty() { String::new() } else { format!("; {}", p.beside_words) };
                format!("{}: {} worth {:.0} metal at {}{under}{beside}, {} from home, {heading}{}{}", p.name, p.composition, p.metal, self.place_words(&places, p.at), distance_words(p.at.dist2d(self.home)), near_ours.unwrap_or_default(), p.harming.as_ref().map_or(String::new(), |(what, metal)| format!("; {what} ({metal:.0} metal) now")))
            })
            .collect();
        // The parties' memory (H-HANDS-ENEMY-MEMORY): every party in sight refreshes its entry; one gone from sight
        // is said with its last place and age for `ARMY_MEMORY`.
        {
            let mut memory = pianist.party_memory.borrow_mut();
            for p in &parties {
                let heading = in_sight.iter().find(|l| l.starts_with(&format!("{}:", p.name))).and_then(|l| ["moving toward our side", "moving away from us", "standing still"].into_iter().find(|h| l.contains(h))).unwrap_or("").to_string();
                match memory.iter_mut().find(|m| m.name == p.name) {
                    Some(m) => *m = PartySeen { name: p.name.clone(), composition: p.composition.clone(), metal: p.metal, at: p.at, seen: frame, heading },
                    None => memory.push(PartySeen { name: p.name.clone(), composition: p.composition.clone(), metal: p.metal, at: p.at, seen: frame, heading }),
                }
            }
            memory.retain(|m| frame - m.seen < ARMY_MEMORY);
        }
        let memory = pianist.party_memory.borrow();
        let mut out_of_sight: Vec<&PartySeen> = memory.iter().filter(|m| !parties.iter().any(|p| p.name == m.name)).collect();
        out_of_sight.sort_by(|a, b| b.metal.total_cmp(&a.metal));
        let out_of_sight: Vec<String> = out_of_sight.iter().take(6).map(|m| format!("{} ({}, {:.0} metal) last seen {} ago at {}{}", m.name, m.composition, m.metal, clock(frame - m.seen), self.place_words(&places, m.at), if m.heading.is_empty() { String::new() } else { format!(", then {}", m.heading) })).collect();
        let biggest = memory.iter().max_by(|a, b| a.metal.total_cmp(&b.metal)).map(|m| format!("{} ({}, {:.0} metal), {} at {}", m.name, m.composition, m.metal, if parties.iter().any(|p| p.name == m.name) { "in sight now".to_string() } else { format!("last seen {} ago", clock(frame - m.seen)) }, self.place_words(&places, m.at)));
        let known_soldiers: Vec<&(UnitDefId, Vec3, i32)> = self.enemy_soldiers.values().filter(|(_, _, seen)| frame - seen < ARMY_MEMORY).collect();
        let known_metal: f32 = known_soldiers.iter().filter_map(|(def, _, _)| self.world.def(*def)).map(|d| d.metal_cost).sum();
        // His buildings by place, with when seen and what guards them (6.6 (d)): "B3: 1 armllt, 1 armmex" per cell
        // gave a group nothing to go and kill.
        let mut remembered: BTreeMap<String, (BTreeMap<String, usize>, i32, usize)> = BTreeMap::new();
        for (id, (def, pos, seen)) in &self.enemy_buildings {
            let key = place_of(&places, *pos).map_or_else(|| format!("{} (no named place)", self.world.grid(*pos)), |p| format!("{p} ({})", self.world.grid(*pos)));
            let entry = remembered.entry(key).or_insert((BTreeMap::new(), frame, 0));
            let unfinished = self.enemy_unfinished.contains(id);
            // His anti-nuke is named with its coverage, so a place inside it is known to be covered; his silo as a silo.
            let kind = match self.world.def(*def) {
                Some(d) if d.coverage > 0.0 => format!("{} (anti-nuke: takes a missile aimed within {:.0} of it)", self.name(*def), d.coverage),
                Some(d) if d.stockpile_seconds > 0.0 => format!("{} (nuclear silo)", self.name(*def)),
                _ => self.name(*def).to_string(),
            };
            *entry.0.entry(if unfinished { format!("{kind} (being built)") } else { kind }).or_default() += 1;
            entry.1 = entry.1.min(*seen);
            if !unfinished && self.world.def(*def).is_some_and(|d| d.weapon_count > 0 && d.reach > 0.0) {
                entry.2 += 1;
            }
        }
        let remembered: Vec<String> = remembered.iter().map(|(place, (kinds, seen, turrets))| format!("{place}: {}; seen {} ago; {}", kinds.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), clock(frame - seen), if *turrets == 0 { "no turret among them".to_string() } else { format!("{turrets} armed") })).collect();
        // Evidence, not a base: where its factories were seen, its start box, and where nobody of ours has looked
        // (docs/design/2026-09-22-enemy-evidence.md).
        let mut factories: Vec<(i32, String)> = self
            .enemy_buildings
            .values()
            .filter(|(def, _, _)| self.world.is_factory_def(*def))
            .map(|(def, pos, seen)| (*seen, format!("{} at {} ({:.0}, {:.0}), last in sight {} ago", self.short_words(*def), self.world.grid(*pos), pos.x, pos.z, clock(frame - seen))))
            .collect();
        factories.extend(self.enemy_factories_gone.iter().map(|(def, pos, at)| (*at, format!("{} at {} ({:.0}, {:.0}), destroyed at {}", self.short_words(*def), self.world.grid(*pos), pos.x, pos.z, clock(*at)))));
        factories.sort_by_key(|(seen, _)| std::cmp::Reverse(*seen));
        let factories: Vec<String> = factories.into_iter().map(|(_, words)| words).collect();
        let our_ally = self.world.hello.ally_team;
        let their_boxes: Vec<&bot_protocol::StartBox> = self.world.hello.start_boxes.iter().filter(|b| b.ally_team != our_ally).collect();
        let in_their_box = |spot: Vec3| their_boxes.iter().any(|b| b.contains(spot));
        let start_box = if their_boxes.is_empty() {
            "the lobby gave it no start box: it started anywhere".to_string()
        } else {
            format!("its commander was placed at 0:00 somewhere inside the lobby's box for its team, cells {}; where it stands now, and where it has built since, is known only from what our units see", their_boxes.iter().map(|b| self.world.box_cells(b)).collect::<Vec<_>>().join(" and "))
        };
        let mut never: Vec<(f32, usize)> = (0..spots.len()).filter(|i| self.spot_seen(*i).is_none()).map(|i| (spots[i].dist2d(self.home), i)).collect();
        never.sort_by(|a, b| a.0.total_cmp(&b.0));
        let never_total = never.len();
        let words = |list: &[&(f32, usize)]| if list.is_empty() { "none".to_string() } else { format!("{}{}", list.iter().take(NEVER_LOOKED).map(|(_, i)| format!("spot_{i} ({})", self.world.grid(spots[*i]))).collect::<Vec<_>>().join(", "), if list.len() > NEVER_LOOKED { ", ..." } else { "" }) };
        let in_box: Vec<&(f32, usize)> = never.iter().filter(|(_, i)| in_their_box(spots[*i])).collect();
        let elsewhere: Vec<&(f32, usize)> = never.iter().filter(|(_, i)| !in_their_box(spots[*i])).collect();
        let mut stale: Vec<(i32, usize)> = (0..spots.len()).filter_map(|i| self.spot_seen(i).filter(|seen| frame - seen > LONG_UNSEEN).map(|seen| (seen, i))).collect();
        stale.sort();
        let stale: Vec<String> = stale.iter().take(NEVER_LOOKED).map(|(seen, i)| format!("spot_{i} ({}) {} ago", self.world.grid(spots[*i]), clock(frame - seen))).collect();
        let enemy = json!({
            "in_sight": in_sight,
            "out_of_sight": if out_of_sight.is_empty() { json!("no party seen in the last three minutes is out of sight") } else { json!(out_of_sight) },
            "biggest_party_known": biggest.unwrap_or_else(|| "none seen in the last three minutes".to_string()),
            "factories_seen": if factories.is_empty() { json!("none, ever: nothing of ours has had one in sight") } else { json!(factories) },
            "start_box": start_box,
            "never_looked": format!("{never_total} of the map's {} metal spots have never been within sight of a unit of ours; a base is always beside metal. Inside its start box, nearest home first: {}. Elsewhere, nearest home first: {}", spots.len(), words(&in_box), words(&elsewhere)),
            "looked_long_ago": if stale.is_empty() { json!("none") } else { json!(stale) },
            "buildings_seen": remembered,
            "army_known": format!("{} soldiers worth {:.0} metal seen in the last three minutes and not seen to die; it may have much more", known_soldiers.len(), known_metal.max(0.0)),
            "commander": self.enemy_commander_seen.map_or("never seen".to_string(), |(pos, seen)| format!("seen at {} {} ago{}", self.place_words(&places, pos), clock(frame - seen), if self.reachable_on_foot(pos) { "" } else { " (in the water or on ground our bots cannot walk to: it is amphibious, our soldiers are not)" })),
        });

        // Actors.
        let damaged_by: BTreeMap<bot_protocol::UnitId, Vec<String>> = tick.events.iter().fold(BTreeMap::new(), |mut m, e| {
            if let Event::UnitDamaged { unit, attacker, .. } = e {
                let who = attacker.and_then(|id| snapshot.enemies.iter().find(|e| e.id == id)).and_then(|e| e.def).map_or("something unseen".to_string(), |d| self.name(d).to_string());
                m.entry(*unit).or_default().push(who);
            }
            m
        });
        let mut actors: BTreeMap<String, Value> = BTreeMap::new();
        for unit in own.iter().filter(|u| !u.being_built) {
            let builder = self.world.is_mobile_builder(unit.def);
            let lab = self.world.is_factory_def(unit.def);
            if !builder && !lab {
                continue;
            }
            let name = self.actor_name(unit.id);
            let mut entry = json!({
                "is": self.unit_words(unit.def),
                "at": self.place_words(&places, unit.pos),
            });
            if builder {
                entry["health"] = json!(format!("{} ({:.0}%)", health_words(unit.health / unit.max_health), unit.health / unit.max_health * 100.0));
                entry["doing"] = json!(self.task_words(pianist.tasks.get(&unit.id), unit, &places, frame, own));
                if let Some(next) = pianist.queued.get(&unit.id) {
                    entry["next"] = json!(format!("queued to start the moment this is done: {}", self.task_words(Some(next), unit, &places, frame, own)));
                }
                if let Some(steps) = pianist.scripts.get(&name).filter(|s| !s.is_empty()) {
                    let waits = if pianist.list_held.contains(&unit.id) {
                        let covered = self.step_site(steps.front().map(String::as_str), &places).and_then(|site| pianist.parties.iter().find(|p| p.at.dist2d(site) < super::menu::THREATENED).map(|p| (p.name.clone(), site)));
                        match covered {
                            Some((party, site)) => format!("; waiting: the hands sent this builder away from an enemy, and {party} stands within {:.0} of the site of its next step ({}); the list goes on when no enemy is on the builder or on that site", super::menu::THREATENED, self.place_words(&places, site)),
                            None => "; waiting: the hands sent this builder away from an enemy, and its list goes on when no enemy is on it".to_string(),
                        }
                    } else {
                        String::new()
                    };
                    // Its first step is an id's building it cannot make, and nobody has started that building yet.
                    let for_starter = steps.front().and_then(|first| {
                        let (body, tag) = super::lists::step_id(first);
                        let tag = tag?;
                        let def = self.world.def_named(body.split_whitespace().next()?)?;
                        let cannot = !self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
                        (cannot && !pianist.tagged.contains_key(tag)).then(|| format!("; its first step waits for a builder that can build #{tag} to start it, and then helps"))
                    }).unwrap_or_default();
                    entry["list"] = json!(format!("the player's list, done by the bot without asking: {}{waits}{for_starter}", steps.iter().cloned().collect::<Vec<_>>().join(", ")));
                }
                let from_home = unit.pos.dist2d(self.home);
                entry["from_home"] = json!(format!("{} ({from_home:.0})", distance_words(from_home)));
                if let Some(party) = parties.iter().filter(|p| p.at.dist2d(unit.pos) < NEAR).min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos))) {
                    let alone: Vec<&OwnUnit> = vec![unit];
                    // Time to contact (8.2): the party's fastest against this builder's own pace, if it stays or
                    // walks away (game 7: the commander helped the plant while the block walked in from 671 to 250).
                    let their_fastest = party.ids.iter().filter_map(|id| snapshot.enemies.iter().find(|e| e.id == *id).and_then(|e| e.def)).filter_map(|d| self.world.def(d)).map(|d| d.speed).fold(0.0, f32::max);
                    let mine = self.world.def(unit.def).map_or(0.0, |d| d.speed);
                    let d = party.at.dist2d(unit.pos);
                    let contact = if their_fastest > 0.0 {
                        let stays = d / their_fastest;
                        let away = if their_fastest > mine { format!("{:.0} s if it walks away", d / (their_fastest - mine)) } else { "never if it walks away (it is faster)".to_string() };
                        format!("; contact in {stays:.0} s if it stays, {away}")
                    } else {
                        String::new()
                    };
                    let mut line = format!("{} ({}) {:.0} away: against this unit alone, {}{contact}", party.name, party.composition, d, self.odds_words(&alone, party, &snapshot.enemies));
                    // The commander's odds are its fighting worth (H-HANDS-COMMANDER-WORTH); the reach words say
                    // why: what must walk into its D-gun and what shoots it from beyond.
                    if unit.def == kit.commander {
                        line.push_str(&self.commander_reach_words(unit.def, &party.ids, &snapshot.enemies));
                    }
                    entry["enemies_near"] = json!(line);
                }
                if let Some(by) = damaged_by.get(&unit.id) {
                    entry["under_fire"] = json!(format!("yes, hit this second by {}", by.join(", ")));
                }
            } else {
                let queue = pianist.lab_queue.get(&unit.id).map(Vec::as_slice).unwrap_or_default();
                // What stands on its pad now (the queue holds only what is not yet started, human-6: "building "
                // with the queue empty and a Grunt on the pad).
                let on_pad = own.iter().filter(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| format!("{} ({:.0}% built{})", self.short_words(u.def), u.build_progress * 100.0, self.future_name(u).map_or(String::new(), |n| format!("; it will be {n}: a `queue` for it now waits for it to stand")))).next();
                entry["doing"] = json!(match (on_pad, queue.is_empty()) {
                    (None, true) if unit.idle => "idle: building nothing".to_string(),
                    (None, true) => "starting a unit".to_string(),
                    (None, false) => format!("building {}", queue.iter().map(|(def, _)| self.short_words(*def)).collect::<Vec<_>>().join(" then ")),
                    (Some(now), true) => format!("building a {now}"),
                    (Some(now), false) => format!("building a {now}, then {}", queue.iter().map(|(def, _)| self.short_words(*def)).collect::<Vec<_>>().join(" then ")),
                });
                let (nanos, nanos_idle) = self.nanos_on(unit, own);
                if nanos > 0 {
                    entry["nanos"] = json!(format!(
                        "{nanos} construction turret{} in reach guard it{}",
                        if nanos == 1 { "" } else { "s" },
                        if nanos_idle > 0 { format!(" ({nanos_idle} idle this second)") } else { String::new() }
                    ));
                }
                if let Some(lane) = self.lane_of(unit) {
                    let stuck = self.stuck_in_lane(&lane, own);
                    if !stuck.is_empty() {
                        let longest = stuck.iter().map(|(_, since)| frame - since).max().unwrap_or(0);
                        let blockers = self.lane_blockers(&lane, own, unit.id);
                        entry["yard"] = json!(format!(
                            "blocked: {} of ours have stood in its exit lane unable to move, the longest for {}; nothing it finishes can leave. {}",
                            stuck.len(),
                            clock(longest),
                            if blockers.is_empty() { "Nothing of ours stands in the lane: the jam is the units themselves or the ground.".to_string() } else { format!("In the lane: {}; the player's remove tool takes them away.", blockers.iter().map(|u| self.handle(u)).collect::<Vec<_>>().join(", ")) }
                        ));
                    }
                }
                let coming = own.iter().filter(|u| u.being_built && self.world.is_constructor_def(u.def)).count() + queue.iter().filter(|(def, _)| self.world.is_constructor_def(*def)).count();
                entry["we_have"] = json!(format!("constructors {constructors}{} for {extractors} extractors; soldiers {} ({})", if coming > 0 { format!(" and {coming} being made") } else { String::new() }, soldiers.len(), soldier_words(soldiers.len(), army_metal)));
                if let Some(Allowance { units: list, .. }) = self.allowed_units(&name).filter(|a| !a.units.is_empty()) {
                    // Only what this lab builds: an advanced plant's line said "allows only: Stout" when the list was
                    // the whole army's (onepass-player-8), and the player took it for the plant's own list.
                    let can_build: &[UnitDefId] = self.world.def(unit.def).map(|d| d.build_options.as_slice()).unwrap_or(&[]);
                    // Each entry's count is cumulative over the earlier entries of the same name (`entry_cap`): the
                    // second "armfav:10" after "armfav:5" reads "10 more allowed" at five made, "no more" at fifteen.
                    let words: Vec<String> = (0..list.len())
                        .filter_map(|k| {
                            let (n, _) = super::allowance(&list[k]);
                            let def = self.world.def_named(n).filter(|d| can_build.contains(d))?;
                            let made = pianist.produced.get(&(unit.id, n.to_string())).copied().unwrap_or(0);
                            Some(match super::entry_cap(&list, k) {
                                Some(cap) if made >= cap => format!("{} (all {} allowed made: no more)", self.short_words(def), super::allowance(&list[k]).1.unwrap_or(cap)),
                                Some(cap) => format!("{} ({} more allowed)", self.short_words(def), cap - made),
                                None => self.short_words(def),
                            })
                        })
                        .collect();
                    let used_up = (0..list.len()).filter(|k| self.world.def_named(super::allowance(&list[*k]).0).is_some_and(|d| can_build.contains(&d))).all(|k| {
                        let (n, _) = super::allowance(&list[k]);
                        !super::entry_permits(&list, k, pianist.produced.get(&(unit.id, n.to_string())).copied().unwrap_or(0))
                    });
                    entry["allowed"] = json!(if words.is_empty() {
                        "the player's list names nothing this lab can build: it builds nothing until a new `produce` list".to_string()
                    } else if used_up {
                        "the player's allowance is used up (every count made): it builds nothing until a new `produce` list".to_string()
                    } else if let Some(next) = self.sequence_unit(unit, &name, pianist) {
                        format!("the player's list, its counted entries made in order by the bot without asking, now a {}: {}", self.short_words(next), words.join(", "))
                    } else {
                        format!("the player allows only: {}", words.join(", "))
                    });
                }
                // Anti-air the plant can make, once his air has been seen (8.3; Cape Violet: the first Liche came
                // unannounced and no flak stood).
                if self.first_seen.iter().any(|d| self.world.domain_of(*d) == crate::world::Domain::Air) {
                    let can_build: &[UnitDefId] = self.world.def(unit.def).map(|d| d.build_options.as_slice()).unwrap_or(&[]);
                    let aa: Vec<String> = can_build.iter().filter(|d| glossary::entry(self.name(**d)).is_some_and(|e| e.class.contains("anti-air") || e.class.contains("fighter"))).map(|d| self.short_words(*d)).collect();
                    if !aa.is_empty() {
                        entry["anti_air"] = json!(format!("his aircraft have been seen; what this plant makes against them: {}", aa.join(", ")));
                    }
                }
                let allowance = self.allowed_units(&name);
                let named = allowance.as_ref().and_then(|a| a.group.clone());
                let call = allowance.as_ref().map_or(0, |a| a.call);
                let rally = pianist.rally.get(&(unit.id, None)).map(|(c, g)| (*c, format!("group_{g}")));
                // Under `by_type`: the groups per unit type this factory has founded.
                let by_type: Vec<String> = pianist.rally.iter().filter(|((f, def), _)| *f == unit.id && def.is_some()).map(|((_, def), (_, g))| format!("{} group_{g}", self.name(def.unwrap()))).collect();
                let join_in = allowance.as_ref().and_then(|a| a.join_in).map_or(String::new(), |n| format!("; its new soldiers muster beside it and walk to their group {n} together (`join_in`)"));
                entry["output_joins"] = json!(match (named.as_deref(), rally) {
                    (Some("new"), Some((founded, g))) if founded == call => format!("{g}, the fresh group your `produce` asked for, founded by its first soldier{join_in}"),
                    (Some("new"), _) => format!("a fresh group of this factory's own from the next soldier (`produce` ... group){join_in}"),
                    (Some("by_type"), _) => format!("one group per unit type, by your `produce` order ({}){join_in}", if by_type.is_empty() { "none founded yet".to_string() } else { by_type.join(", ") }),
                    (Some(g), _) => format!("{g}, by your `produce` order{join_in}"),
                    (None, Some((_, g))) => format!("{g}, this factory's own group (name another in `produce`, or `new`, or `by_type`){join_in}"),
                    (None, None) => format!("a group of this factory's own, made on its first soldier (name one in `produce`){join_in}"),
                });
                entry["health"] = json!(health_words(unit.health / unit.max_health));
            }
            if let Some(d) = draws.iter().find(|d| d.who == name) {
                // A factory's words carry what each 100 build power helping it adds, so a sink can be sized from them
                // (the player's report keeps a factory's draw line, H-PLAYER-SPENDING-LINE).
                entry["draw"] = json!(if d.factory {
                    let (e100, m100) = self.draw_rate(100.0, d.target);
                    format!(
                        "draws about {:.0} energy and {:.1} metal a second at full speed on what it builds now, alone; each 100 build power helping it adds {e100:.0} and {m100:.1}{}",
                        d.energy,
                        d.metal,
                        if d.helping > 0.0 { format!("; {:.0} is helping it now", d.helping) } else { String::new() }
                    )
                } else {
                    format!("draws about {:.0} energy and {:.1} metal a second at full speed on what it builds now", d.energy, d.metal)
                });
            }
            actors.insert(name, entry);
        }
        let scouts: Vec<String> = pianist.groups.iter().filter(|g| g.roving()).map(|g| format!("group_{} roving: {}", g.name, self.rover_words(g, own, &snapshot.enemies, &places))).collect();
        for group in &pianist.groups {
            let units = group.units(own);
            let own_shelling = group_shelling.get(&group.name);
            // The group as a body (H-HANDS-GROUP-BODY): its front toward its goal, else toward the nearest party
            // any member is near; the tail; who is still on the way to join.
            let goal = match &group.task {
                GroupTask::Move { to, .. } => Some(*to),
                GroupTask::Engage { at, .. } | GroupTask::Follow { at, .. } => Some(*at),
                GroupTask::Hunt(h) => Some(h.at),
                GroupTask::Hold { .. } | GroupTask::Rove { .. } => None,
            };
            let nearest_to_any = |p: &Party| units.iter().map(|u| u.pos.dist2d(p.at)).fold(f32::INFINITY, f32::min);
            let toward = goal.or_else(|| parties.iter().map(|p| (nearest_to_any(p), p)).filter(|(d, _)| *d < NEAR).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p.at));
            let Some(body) = group.body(own, toward) else { continue };
            // A declared group (the lane's formation, the migration design chunk 2) is read from the lane's state:
            // its place is the anchor, and the `formation` line below carries what the body words said.
            let lane_state = if group.declared() { self.lane.group_state(&group.lane_name()) } else { None };
            let centre = lane_state.map_or(body.at, |s| s.anchor);
            let metal: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            let shape = |to: Vec3| {
                let n = body.core.len();
                let tail_short = body.tail.dist2d(to);
                if body.strung_out() {
                    format!("its front {:.0} to go, {} of {n} arrived, its tail {tail_short:.0} short", body.front.dist2d(to), body.arrived)
                } else {
                    format!("{:.0} to go, {} of {n} arrived", body.at.dist2d(to), body.arrived)
                }
            };
            let doing = match &group.task {
                // No clocks on a course and no "on your pick" (2026-10-24: the policy of t-1 does not enter the words;
                // `docs/design/2026-10-24-pick-rework.md` §6).
                GroupTask::Hold { picked: true, .. } => "holding here".to_string(),
                GroupTask::Hold { committed, .. } => format!("standing with no order{}", if *committed { ", fighting everything here, turrets included" } else { "" }),
                GroupTask::Move { place, fight, .. } if lane_state.is_some() => format!("{} to {place}", if *fight { "advancing" } else { "walking" }),
                GroupTask::Move { to, place, fight, .. } => format!("{} to {place}: {}", if *fight { "advancing" } else { "walking" }, shape(*to)),
                // Told to attack is not attacking: the entry says where the group stands against the party
                // (H-HANDS-STANDING).
                GroupTask::Engage { party, at, target, .. } => {
                    let aimed = parties.iter().find(|p| p.ids.iter().any(|id| party.contains(id)));
                    let name = aimed.map_or("a party now out of sight".to_string(), |p| p.name.clone());
                    let standing = aimed.map_or(String::new(), |p| format!(": {}", self.standing(&body.core, p, &snapshot.enemies).words()));
                    format!("told to attack {name}{} at {}{standing}", if target.is_some() { " (one named unit of it, until it dies)" } else { "" }, self.place_words(&places, *at))
                }
                GroupTask::Follow { name: ward, at, .. } => format!("following {ward} ({:.0} from it)", body.at.dist2d(*at)),
                GroupTask::Hunt(hunt) => format!("hunting {} (its quarry {} at {})", hunt.party, if frame - hunt.last_seen < FRAMES_PER_SECOND { "seen" } else { "last seen" }, self.place_words(&places, hunt.at)),
                // What each rover is doing this second; the menu's course words are these (`group_course`).
                GroupTask::Rove { within, .. } => format!("roving{}: {}", within.as_ref().map_or(String::new(), |w| format!(" within {w}")), self.rover_words(group, own, &snapshot.enemies, &places)),
            };
            // Health as a distribution, not an average (game 9: "full on average" at 22 of 39 lost), with the
            // losses since the player's last orders, which is what it asked about.
            let full = units.iter().filter(|u| u.health >= 0.95 * u.max_health).count();
            let under_half = units.iter().filter(|u| u.health < 0.5 * u.max_health).count();
            let (lost_since_orders, lost_metal_since) = group.lost_since(group.losses_since, &self.world);
            let health = format!(
                "{full} of {} at full, {under_half} under half{}",
                units.len(),
                if lost_since_orders > 0 { format!(", {lost_since_orders} lost ({lost_metal_since:.0} metal) since the player's last orders") } else { String::new() }
            );
            let at = if body.strung_out() && lane_state.is_none() {
                format!("front at {}; its tail is {:.0} behind at {}", self.place_words(&places, body.front), body.length, self.place_words(&places, body.tail))
            } else {
                self.place_words(&places, centre)
            };
            let mut entry = json!({
                "units": format!("{}: {} ({} soldiers worth {metal:.0} metal{})", soldier_words(units.len(), metal), self.composition_words(&units), units.len(), if group.domain == crate::world::Domain::Ground { String::new() } else { format!("; an {} group", group.domain.word()) }),
                "at": at,
                "health": health,
                "doing": doing,
            });
            // The lane's reading of a declared group (docs/design/2026-10-25-hands-migration.md §4): one number set
            // per group. None until a kind is attached (chunk 2): a hunt and a rove are the lane's by other means
            // and form no ranks.
            if let Some(state) = lane_state {
                let to = goal.map(|p| self.place_words(&places, p));
                let engaged = state.engaged.and_then(|t| snapshot.enemies.iter().find(|e| e.id == t)).map(|e| format!("his {} at {}", e.def.map_or("unit".to_string(), |d| self.short_words(d)), self.place_words(&places, e.pos)));
                entry["formation"] = json!(formation_words(state, group.members.len(), to.as_deref(), &self.place_words(&places, state.anchor), engaged.as_deref()));
                // The companies (the companies design §4), when the army is more than one or a muster stands.
                if state.companies.len() > 1 || state.companies.iter().any(|c| c.role == micro::Role::Mustering) {
                    entry["companies"] = json!(company_words(state, self.company_of(&group.name), pianist.musters.get(&group.name).and_then(|(_, r)| *r), &|p| self.place_words(&places, p)));
                }
            }
            // A shelling group's screen, the lane's second formation of it (chunk 5).
            if group.shelling && let Some(state) = self.lane.group_state(&group.screen_lane_name()) {
                let members = state.formed.len() + state.exceptions.len();
                let engaged = state.engaged.and_then(|t| snapshot.enemies.iter().find(|e| e.id == t)).map(|e| format!("his {} at {}", e.def.map_or("unit".to_string(), |d| self.short_words(d)), self.place_words(&places, e.pos)));
                entry["screen"] = json!(format!("its screen ({members} soldiers between the artillery and the target): {}", formation_words(state, members, Some("its screen point"), &self.place_words(&places, state.anchor), engaged.as_deref())));
            }
            // The registers: the named places it has reached and what it has met since the last; Jev reads where it
            // is on the way its instructions describe from these (the last pick is no register since 2026-10-24).
            if !group.reached.is_empty() {
                entry["reached"] = json!(group.reached.iter().map(|(p, f)| format!("{p} ({})", clock(*f))).collect::<Vec<_>>().join(", "));
            }
            // What it fired lately and what of that was seen to land (H-HANDS-FIRING-LINE).
            if let Some(words) = self.firing_words(&group.members) {
                entry["firing"] = json!(words);
            }
            if let Some((f, words)) = &group.met {
                entry["met"] = json!(format!("since its last stop: {words} ({})", clock(*f)));
            }
            if !body.joining.is_empty() {
                let farthest = body.joining.iter().map(|u| u.pos.dist2d(body.at)).fold(0.0, f32::max);
                let from: BTreeMap<String, usize> = body.joining.iter().fold(BTreeMap::new(), |mut m, u| {
                    let plant = pianist.produced_by.get(&u.id).map_or("elsewhere".to_string(), |f| self.actor_name(*f));
                    *m.entry(plant).or_default() += 1;
                    m
                });
                entry["reinforcements"] = json!(format!(
                    "{} of its soldiers are on the way to join the body, the farthest {farthest:.0} behind (from {}); they are not in its fights yet",
                    body.joining.len(),
                    from.iter().map(|(p, n)| format!("{n} from {p}")).collect::<Vec<_>>().join(", ")
                ));
            }
            if !body.stragglers.is_empty() && lane_state.is_none() {
                let farthest = body.stragglers.iter().map(|u| u.pos.dist2d(body.at)).fold(0.0, f32::max);
                entry["stragglers"] = json!(format!("{} of its soldiers are stragglers, up to {farthest:.0} from the body; they are not in its fights yet", body.stragglers.len()));
            }
            // The lane as it plays: a roving group is the lane's whole (H-MICRO-ROVE, a pick), else the player's
            // footwork setting.
            if let GroupTask::Rove { within: Some(region), .. } = &group.task {
                let cells = places.iter().find(|p| p.name == *region).and_then(|p| p.cells).map_or(String::new(), |c| format!(" (the cells {})", crate::strategist::shared::cells_words(c)));
                entry["lane"] = json!(format!("roving within {region}{cells}: its soldiers look only at places inside it, kill what they find unguarded there (extractors, radars, lone constructors) and keep out of the reach of anything that can shoot it; a pick of yours brings them back, and a plain `rove` frees them of the region"));
            } else if group.roving() {
                entry["lane"] = json!("roving: its soldiers look at the places nothing of ours has seen (his start box first), kill what they find unguarded (extractors, radars, lone constructors) and keep out of the reach of anything that can shoot it; a pick of yours brings them back");
            } else if let Some(words) = self.footwork_of(&group.name).words() {
                entry["lane"] = json!(words);
            }
            if group.roving() && !group.rove_log.is_empty() {
                entry["rove"] = json!(group.rove_log.iter().map(|(_, line)| line.as_str()).collect::<Vec<_>>());
            }
            // The last 30 s, not "since the player's last orders": turns come every ten seconds now, and the line read
            // "lost 1" while the ball had lost eleven in eleven seconds (wake-1, 18:14). A share in words beside the
            // number: Jev is not a calculator (docs.typesafe.ai/model-jaggedness).
            let lost_lately: Vec<&(i32, UnitDefId, Option<(bot_protocol::UnitId, UnitDefId)>)> = group.losses.iter().filter(|(f, ..)| frame - f <= 30 * FRAMES_PER_SECOND).collect();
            if !lost_lately.is_empty() {
                // To whom, and the verdict against the party in its entry alone (`loss_words`).
                let near = parties.iter().map(|p| (nearest_to_any(p), p)).filter(|(d, _)| *d < NEAR).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
                let w = self.loss_words(&lost_lately, metal, near, &parties);
                let last = lost_lately.iter().map(|(f, ..)| *f).max().unwrap_or(frame);
                entry["losses"] = json!(format!("lost {} of its {} soldiers ({:.0} metal, {}{}) in the last 30 s, the last {} s ago: {}", w.count, units.len() + lost_lately.len(), w.metal, w.share, w.verdict, (frame - last) / FRAMES_PER_SECOND, w.to_whom));
            }
            // The army in pieces, said on both sides (escalate-6: 19 detachments and 15 splits in ten minutes, five to
            // nine groups alive, dying one by one; nothing in the picture counted them).
            let children: Vec<&super::Group> = pianist.groups.iter().filter(|g| g.parent.as_deref() == Some(group.name.as_str()) && !g.members.is_empty()).collect();
            if !children.is_empty() {
                let soldiers: usize = children.iter().map(|g| g.members.len()).sum();
                entry["detachments_out"] = json!(format!(
                    "{} group{} of {soldiers} soldiers split from it, still out: {}",
                    children.len(), if children.len() == 1 { "" } else { "s" },
                    children.iter().map(|g| format!("group_{} ({})", g.name, g.members.len())).collect::<Vec<_>>().join(", ")
                ));
            }
            if let Some(parent) = &group.parent {
                entry["split_from"] = json!(format!("group_{parent}"));
            }
            if let Some(chase) = self.chase_met(group, own, &parties, &places) {
                entry["chase"] = json!(chase.words);
            }
            let stuck: Vec<&OwnUnit> = units.iter().filter(|u| self.stuck.contains_key(&u.id)).copied().collect();
            if !stuck.is_empty() {
                let longest = stuck.iter().filter_map(|u| self.stuck.get(&u.id)).map(|s| frame - s.since).max().unwrap_or(0);
                let yard = own.iter().filter(|f| self.lane_of(f).is_some_and(|lane| stuck.iter().any(|u| lane.contains(u.pos)))).map(|f| self.actor_name(f.id)).next();
                entry["stuck"] = json!(format!(
                    "{} of its {} soldiers cannot move (the engine gives up their moves), the longest for {}{}",
                    stuck.len(),
                    units.len(),
                    clock(longest),
                    match yard { Some(name) => format!(", standing in {name}'s exit lane"), None => format!(", at {}", self.place_words(&places, stuck[0].pos)) }
                ));
            }
            // Soldiers standing with an enemy in their reach that have not fired: where and, when the ground says
            // it, why (H-HANDS-CANNOT-FIRE).
            let silent: Vec<(&&OwnUnit, i32)> = units.iter().filter_map(|u| Some((u, pianist.silent(u.id, self.world.def(u.def)?.reload, frame)?))).collect();
            if !silent.is_empty() {
                let depth = |u: &OwnUnit| -self.ground_height(u.pos);
                let wet: Vec<&(&&OwnUnit, i32)> = silent.iter().filter(|(u, _)| depth(u) > 0.0).collect();
                let longest = silent.iter().map(|(_, s)| *s).max().unwrap_or(0);
                let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
                for (u, _) in &silent {
                    *kinds.entry(self.short_words(u.def)).or_default() += 1;
                }
                let why = if wet.len() == silent.len() {
                    format!("all of them stand in water {:.0} to {:.0} deep, where a weapon mounted low is under the surface", wet.iter().map(|(u, _)| depth(u)).fold(f32::INFINITY, f32::min), wet.iter().map(|(u, _)| depth(u)).fold(0.0, f32::max))
                } else if !wet.is_empty() {
                    format!("{} of them stand in water up to {:.0} deep, where a weapon mounted low is under the surface; the others on dry ground have a friend in their line of fire or a target their weapon cannot hit", wet.len(), wet.iter().map(|(u, _)| depth(u)).fold(0.0, f32::max))
                } else {
                    "they stand on dry ground: a friend in their line of fire, or a target their weapon cannot hit".to_string()
                };
                entry["cannot_fire"] = json!(format!(
                    "{} of its {} soldiers ({}) have had an enemy inside their own reach for up to {longest} s and have not fired a shot: {why}; they are in the fight's count and do nothing in it where they stand",
                    silent.len(), units.len(), kinds.iter().map(|(k, n)| format!("{n} {k}")).collect::<Vec<_>>().join(", ")
                ));
            }
            let fleeing = units.iter().filter(|u| self.lane.fleeing(u.id)).count();
            if fleeing > 0 {
                entry["footwork"] = json!(format!("{fleeing} of its {} soldiers are being held back by their own footwork this second: stepping out of a turret's reach they were not sent against, or out of a fight they would die in", units.len()));
            }
            // The nearest party with the odds against this group and what it is killing: the words the `do` question
            // weighs (they were only on the engage option's line before). Near any member, not the centre: the
            // Pounder ball had no line while Bulls stood 669 from its nearest Pounder (game 9, 15:44-16:05).
            if let Some((d, p)) = parties.iter().map(|p| (nearest_to_any(p), p)).filter(|(d, _)| *d < NEAR).min_by(|a, b| a.0.total_cmp(&b.0)) {
                // Its own soldiers under fire, said as such; what the party kills elsewhere is not this group's loss
                // (wake-3: "killing 3 of our Blitz" on a group's line read as its own, and it retreated from a party it
                // outweighed in 7 of 51 such asks, 0 of 69 without the clause).
                let own_ids: Vec<bot_protocol::UnitId> = units.iter().map(|u| u.id).collect();
                // An unarmed party that damages ours is taking them apart (a constructor reclaiming a Pawn, human-11
                // 6:37: "it is unarmed and cannot fight back ... it is hitting this group now" in one line).
                let killing = match (self.killing_words(&p.ids, Some(&own_ids)), &p.harming) {
                    (Some((what, metal)), _) if p.unarmed => format!("; it is taking apart {what} ({metal:.0} metal) of this group now, which an unarmed unit can do to what stands still beside it"),
                    (Some((what, metal)), _) => format!("; it is hitting this group now: {what} ({metal:.0} metal)"),
                    (None, Some((what, metal))) => format!("; it is {what} ({metal:.0} metal) elsewhere, not this group"),
                    (None, None) => String::new(),
                };
                // The party's turrets are in this line only when they reach the group where it stands.
                let (here, away) = self.party_where_we_stand(p, &body.core);
                let under = if here.turrets.is_empty() { String::new() } else { format!(", under {}", here.turrets) };
                let touching = body.within(p.at, NEAR).len();
                entry["enemies_near"] = json!(format!("{} ({}{under}) {d:.0} from its nearest soldier, within {NEAR:.0} of {touching} of its {}: {}{away}{killing}", p.name, p.composition, body.core.len(), self.group_odds(&body, &here, &snapshot.enemies, &snapshot.allies).1));
            }
            let threats = self.threats_words(&parties, &places, own, kit, centre);
            if !threats.is_empty() {
                entry["enemies_at_our_extractors"] = json!(threats);
            }
            if !scouts.is_empty() && group.members.len() > 1 {
                entry["scouts_out"] = json!(scouts);
            }
            let shooters: Vec<&String> = units.iter().filter_map(|u| damaged_by.get(&u.id)).flatten().collect();
            if !shooters.is_empty() {
                let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
                for who in shooters {
                    *kinds.entry(who.as_str()).or_default() += 1;
                }
                let words: Vec<String> = kinds
                    .iter()
                    .map(|(who, n)| {
                        if *who != "something unseen" {
                            return format!("{who} ({n} hits)");
                        }
                        match &own_shelling {
                            Some((s, place_name)) => match &s.attributed {
                                Some(a) => format!("something out of our sight, {n} hits: a {} with range {:.0}, likely the {} seen {} s ago at {} (the place `{place_name}`)", self.weapon_words(&s.weapon), s.range, a.name, (frame - a.seen) / super::super::FRAMES_PER_SECOND, self.place_words(&places, s.at)),
                                None => format!("something out of our sight, {n} hits: a {} with range {:.0} from the {}; its likeliest place is `{place_name}` at {}", self.weapon_words(&s.weapon), s.range, super::super::shelling::compass(s.dir), self.place_words(&places, s.at)),
                            },
                            None => format!("something out of our sight, {n} hits: a turret or artillery that outranges us"),
                        }
                    })
                    .collect();
                entry["under_fire"] = json!(format!("yes, this second, by {}", words.join(", ")));
                if let Some((s, place_name)) = &own_shelling
                    && kinds.contains_key("something unseen")
                {
                    entry["unseen_shooter"] = json!(format!("the shooter out of sight, likeliest at `{place_name}` ({}), {}", self.place_words(&places, s.at), self.unseen_shooter_words(&units, s)));
                }
            }
            actors.insert(format!("group_{}", group.name), entry);
        }

        // The other seats' groups (H-HANDS-ALLIED-GROUPS): position, size, task and what they engage, as their own
        // pictures said them a second ago, so "together" and "as one body" have something to hold to. They are
        // theirs to order: no slot is made for them.
        let mut allies: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(shared) = &self.strategist {
            let hands = shared.hands.lock().unwrap();
            for (team, h) in hands.iter().filter(|(team, _)| **team != self.world.hello.team) {
                if let Some(entries) = h.picture["actors"].as_object() {
                    for (name, entry) in entries.iter().filter(|(n, _)| n.starts_with("group_")) {
                        let mut line = json!({ "seat": format!("t{team}, an allied seat of ours: its group, not yours to order") });
                        for key in ["units", "at", "doing", "enemies_near", "losses"] {
                            if let Some(v) = entry.get(key) {
                                line[key] = v.clone();
                            }
                        }
                        allies.insert(name.clone(), line);
                    }
                }
            }
        }
        let wind = (self.world.hello.map.wind_min + self.world.hello.map.wind_max) / 2.0;
        let rules = format!(
            "{}<<builders>>This map's wind averages about {wind:.0}: {}.<</builders>>",
            crate::texts::read(&crate::texts::HANDS_RULES),
            if wind >= 8.0 { "wind generators (40 metal) beat solar collectors here" } else { "solar collectors are the reliable energy here" }
        );
        // Jev's answers move with the order of these sections, and the order sent is by key (the JSON map is
        // sorted): a new section's name decides where it sits (K-jev-answers-move-with-the-order-of-the-pictures-sections).
        let mut state = json!({
            "instructions": instructions,
            "clock": clock(frame),
            "economy": economy,
            "ours": ours,
            "enemy": enemy,
            "places": place_entries,
            "actors": actors,
            "allies": if allies.is_empty() { json!("no other seat of ours has a group") } else { json!(allies) },
            "recent": pianist.recent(frame),
        });
        // The signals as they stand: a fact the hands act on where they do not act on a time written in the
        // packet (K-jev-a-clock-condition-in-the-packet-is-not-followed). Absent when none is set.
        if let Some(shared) = &self.strategist {
            let (signals, _) = shared.signals_all(frame);
            if !signals.is_empty() {
                state["signals"] = json!(signals);
            }
        }
        // H-HANDS-FALL-BACK: whether the player can answer now. Under the think penalty its orders land as long after
        // the turn as it took to decide, and nothing new comes from it meanwhile.
        if let Some(shared) = &self.strategist {
            let turn = shared.last_turn_frame.load(std::sync::atomic::Ordering::Relaxed);
            let landing = shared.delayed.lock().unwrap().as_ref().map(|(at, _)| *at);
            state["player"] = json!(match landing {
                Some(at) => format!("deciding: its orders from {} s ago reach you in {} s; it has not seen what has happened since, so what the picture shows now outranks an instruction it contradicts", (frame - turn) / FRAMES_PER_SECOND, (at - frame).max(0) / FRAMES_PER_SECOND),
                None if turn > 0 => format!("watching: its last orders reached you {} s ago; a group losing soldiers or meeting a party it does not outweigh, an extractor threatened or lost, wakes it within a few seconds", (frame - turn) / FRAMES_PER_SECOND),
                None => "has not spoken yet".to_string(),
            });
        }
        Picture { state, rules, places, parties }
    }

    /// The name a unit still being built will answer to once it stands (a builder or a factory; None for a soldier):
    /// the picture says it on its builder's or its lab's line, so a `queue` or `produce` for it can be given now and
    /// waits for it (the user, 2026-10-03: "a mechanic for staging build order lists for units that are currently
    /// under construction").
    pub(crate) fn future_name(&self, unit: &OwnUnit) -> Option<String> {
        let def = unit.def;
        if self.world.is_factory_def(def) {
            Some(if self.name(def).ends_with("vp") { format!("plant_{}", unit.id.0) } else if self.name(def).contains("lab") { format!("lab_{}", unit.id.0) } else { format!("factory_{}", unit.id.0) })
        } else if self.world.is_mobile_builder(def) {
            Some(format!("constructor_{}", unit.id.0))
        } else {
            None
        }
    }

    /// How an actor is named in the picture and the questions.
    pub(crate) fn actor_name(&self, unit: bot_protocol::UnitId) -> String {
        // By what the definition is, not by the Kit: a captured factory of the other faction is played like ours.
        match self.known_units.get(&unit).map(|(def, _)| *def) {
            Some(def) if self.world.is_commander_def(def) => self.commander_handle(),
            Some(def) if self.world.is_factory_def(def) && self.name(def).ends_with("vp") => format!("plant_{}", unit.0),
            Some(def) if self.world.is_factory_def(def) && self.name(def).contains("lab") => format!("lab_{}", unit.0),
            Some(def) if self.world.is_factory_def(def) => format!("factory_{}", unit.0),
            _ => format!("constructor_{}", unit.0),
        }
    }
}

/// The army's companies (the companies design §4): how many of up to the cap, each marching one by its size, its
/// formed count, where it stands and how far from the main body (the largest, whose numbers the formation words
/// give), and a muster by its plant's place and what releases it.
pub(crate) fn company_words(state: &micro::GroupState, cap: usize, release: Option<usize>, place: &dyn Fn(Vec3) -> String) -> String {
    use micro::Role;
    let mut parts: Vec<String> = Vec::new();
    for c in &state.companies {
        parts.push(match c.role {
            Role::Marching => {
                let off = c.anchor.dist2d(state.anchor);
                let from = if off < 1.0 { " (the main body)".to_string() } else { format!(", {off:.0} from the main body") };
                format!("{} ({} formed) at {}{from}", c.members, c.formed, place(c.anchor))
            }
            Role::Mustering => format!(
                "{} mustering at {} (they leave at {} or after {} s)",
                c.members,
                c.at.map_or("their plant".to_string(), place),
                release.unwrap_or(cap).max(1),
                micro::army::MUSTER_PATIENCE / 30
            ),
        });
    }
    format!("{} compan{} of up to {cap}, each marching on its own way: {}", state.companies.len(), if state.companies.len() == 1 { "y" } else { "ies" }, parts.join("; "))
}

/// The lane's reading of a declared group (docs/design/2026-10-25-hands-migration.md §4): observations only, one
/// number set per group where the body words gave a front, a tail and a fifth. `to`: where it is going, in the
/// picture's place words, when it is going somewhere; `at`: the anchor's place words; `engaged`: the unit the arc
/// stands on, when it does. A column (two files or fewer) says the ground narrowed it.
pub(crate) fn formation_words(state: &micro::GroupState, members: usize, to: Option<&str>, at: &str, engaged: Option<&str>) -> String {
    use micro::Cause;
    let count = |cause: Cause| state.exceptions.iter().filter(|(_, c, _)| *c == cause).count();
    let farthest = |cause: Cause| state.exceptions.iter().filter(|(_, c, _)| *c == cause).map(|(_, _, s)| *s).fold(0.0f32, f32::max);
    let formed = state.formed.len();
    let mut outliers = Vec::new();
    let behind = count(Cause::Behind) + count(Cause::Away) + count(Cause::Stuck);
    if behind > 0 {
        outliers.push(format!("{behind} behind (the farthest {:.0} s)", farthest(Cause::Behind).max(farthest(Cause::Away)).max(farthest(Cause::Stuck))));
    }
    if count(Cause::Joining) > 0 {
        outliers.push(format!("{} joining", count(Cause::Joining)));
    }
    if count(Cause::CutOff) > 0 {
        outliers.push(format!("{} cut off by the ground", count(Cause::CutOff)));
    }
    if count(Cause::Fled) > 0 {
        outliers.push(format!("{} fleeing", count(Cause::Fled)));
    }
    let outliers = if outliers.is_empty() { String::new() } else { format!(", {}", outliers.join(", ")) };
    let ranks = if state.files <= 2 { format!("in a column of {}", state.files) } else { format!("in ranks of {}", state.files) };
    let mut out = Vec::new();
    if state.way.is_none()
        && let Some(to) = to
    {
        out.push(format!("no way on foot from here to {to} for this group"));
    }
    if let Some(target) = engaged {
        out.push(format!("engaged at {at}: the arc on {target}, {formed} of {members} in rank{outliers}"));
    } else if state.arrived {
        out.push(format!("arrived at {}, {formed} of {members} in rank{outliers}", to.unwrap_or(at)));
    } else if state.pace <= 0.0 {
        out.push(format!("held at {at} {ranks}, {formed} of {members} formed{outliers}"));
    } else {
        let share = if state.full_pace > 0.0 { state.pace / state.full_pace * 100.0 } else { 0.0 };
        let going = to.map_or(String::new(), |to| format!(" to {to}"));
        let points = if state.legs_left > 1 { format!(", {} points to go", state.legs_left) } else { String::new() };
        let way = state.way.map_or(String::new(), |w| format!(", {w:.0} on its way"));
        out.push(format!("advancing{going} {ranks}, {formed} of {members} formed{outliers}; walking at {share:.0}% of its pace{points}{way}"));
    }
    out.join("; ")
}

#[cfg(test)]
mod tests {
    use bot_protocol::{Event, OwnUnit, Resource, Snapshot, Tick, UnitDefId, UnitId};

    use super::super::fixtures::{at, brain_of, enemy, own};

    /// The formation words (the migration design §4): the march with its outliers by cause, the column, the hold,
    /// the arrival, the contact, and "no way on foot" ahead of the rest when the ground gives none.
    #[test]
    fn the_formation_words_read_the_lanes_state() {
        use micro::{Cause, GroupState};
        let state = GroupState {
            anchor: at(100.0, 100.0),
            heading: (1.0, 0.0),
            pace: 70.0,
            full_pace: 87.0,
            formed: vec![UnitId(1), UnitId(2), UnitId(3)],
            exceptions: vec![(UnitId(4), Cause::Behind, 2.0), (UnitId(5), Cause::Away, 9.6), (UnitId(6), Cause::Joining, 20.0), (UnitId(7), Cause::CutOff, 0.0)],
            arrived: false,
            legs_left: 3,
            engaged: None,
            files: 6,
            room: Some(300.0),
            way: Some(2345.0),
            companies: Vec::new(),
        };
        let words = super::formation_words(&state, 8, Some("spot_3 (D4)"), "(C4)", None);
        assert_eq!(words, "advancing to spot_3 (D4) in ranks of 6, 3 of 8 formed, 2 behind (the farthest 10 s), 1 joining, 1 cut off by the ground; walking at 80% of its pace, 3 points to go, 2345 on its way");
        let column = GroupState { files: 2, exceptions: Vec::new(), legs_left: 1, ..state.clone() };
        assert_eq!(super::formation_words(&column, 3, Some("spot_3 (D4)"), "(C4)", None), "advancing to spot_3 (D4) in a column of 2, 3 of 3 formed; walking at 80% of its pace, 2345 on its way");
        let held = GroupState { pace: 0.0, ..state.clone() };
        assert!(super::formation_words(&held, 8, Some("spot_3 (D4)"), "(C4)", None).starts_with("held at (C4) in ranks of 6, 3 of 8 formed, 2 behind"));
        let arrived = GroupState { arrived: true, pace: 0.0, exceptions: Vec::new(), ..state.clone() };
        assert_eq!(super::formation_words(&arrived, 3, Some("spot_3 (D4)"), "(C4)", None), "arrived at spot_3 (D4), 3 of 3 in rank");
        let engaged = GroupState { pace: 0.0, engaged: Some(UnitId(99)), ..state.clone() };
        assert!(super::formation_words(&engaged, 8, None, "(C4)", Some("his Pawn at (C5)")).starts_with("engaged at (C4): the arc on his Pawn at (C5), 3 of 8 in rank, 2 behind"));
        let no_way = GroupState { way: None, ..state.clone() };
        assert!(super::formation_words(&no_way, 8, Some("spot_3 (D4)"), "(C4)", None).starts_with("no way on foot from here to spot_3 (D4) for this group; advancing to spot_3 (D4)"));
    }

    /// A hunting group's `chase` line says the turret whose reach the line to its quarry enters within reach of the
    /// front, where that reach begins, and the party beside the chase that is not its quarry with the odds; a group
    /// holding has no such line; the keys name each thing once.
    #[test]
    fn a_chase_says_the_turret_ahead_and_the_party_beside_it() {
        use super::super::groups::{Group, GroupTask, Hunt};
        use super::super::fixtures::e3;
        let (brain, ours, _, parties) = e3();
        let quarry = at(4950.0, 1700.0);
        let hunt = Hunt { quarry: UnitId(77), party: "party_99".into(), from: quarry, since: 0, last_seen: 0, at: quarry };
        let group = Group::new("H".into(), crate::world::Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::Hunt(hunt), 0);
        let met = brain.chase_met(&group, &ours, &parties, &[]).expect("the chase meets the turrets and the Centurions");
        assert!(met.words.starts_with("hunting party_99 (") && !met.words.contains(" s, ") && met.words.contains("it is let go at; ahead on the quarry's line: his "), "{}", met.words);
        assert!(met.words.contains("its reach begins") && met.words.contains("ahead of the front") && met.words.contains("beside the chase: party_12 (3 armwar, 810 metal)") && met.words.ends_with("any other pick ends the chase"), "{}", met.words);
        assert!(met.keys.iter().any(|k| k == "turret 10") && met.keys.iter().any(|k| k == "party_12"), "{:?}", met.keys);
        let holding = Group::new("H".into(), crate::world::Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(bot_protocol::Vec3::default()), 0);
        assert!(brain.chase_met(&holding, &ours, &parties, &[]).is_none());
        // Chasing away from the turrets, with the Centurions out of reach: nothing met.
        let away = Hunt { quarry: UnitId(77), party: "party_99".into(), from: at(3400.0, 2600.0), since: 0, last_seen: 0, at: at(3400.0, 2600.0) };
        let group = Group::new("H".into(), crate::world::Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::Hunt(away), 0);
        let mut far = parties.clone();
        far[0].at = at(6000.0, 6000.0);
        assert!(brain.chase_met(&group, &ours, &far, &[]).is_none());
    }

    /// A party's turrets count against a group only where they reach it: a group out of their reach meets the party
    /// without them and is told where they stand; one beside them meets the party as it is.
    #[test]
    fn a_partys_turrets_count_only_where_they_reach_the_group() {
        let (brain, ours, _, parties) = super::super::fixtures::e3();
        let (ground, air, words) = brain.turrets_covering(parties[0].at);
        let party = super::Party { turret_metal: ground, turret_metal_air: air, turrets: words, ..parties[0].clone() };
        assert!(party.turret_metal > 0.0 && party.turrets == "2 turrets: 2 armllt", "{}", party.turrets);
        let far: Vec<&OwnUnit> = ours.iter().collect();
        let (here, away) = brain.party_where_we_stand(&party, &far);
        assert!(here.turret_metal == 0.0 && here.turrets.is_empty(), "{here:?}");
        assert!(away.starts_with("; the 2 turrets: 2 armllt covering it stand where it is now, ") && away.ends_with(" from this group, and shoot only what goes there"), "{away}");
        let beside = super::super::fixtures::own(901, 1, super::super::fixtures::at(4700.0, 1600.0));
        let (here, away) = brain.party_where_we_stand(&party, &[&beside]);
        assert!(here.turret_metal == party.turret_metal && away.is_empty(), "{here:?} {away}");
    }

    /// The losses line says to whom, and "losing this fight" counts only the losses to the party in the entry and
    /// to turrets: three Blitzes a dead Sentry killed are no verdict against an unrelated party; the same three to
    /// the party are.
    #[test]
    fn losses_are_said_with_their_killers_and_the_verdict_is_the_partys_alone() {
        let (brain, _, _, parties) = super::super::fixtures::e3();
        let party = &parties[0];
        let sentry = UnitId(10);
        let dead_sentry = UnitId(99);
        let flash = UnitDefId(1);
        let lost = vec![(60, flash, Some((dead_sentry, UnitDefId(3)))), (70, flash, Some((dead_sentry, UnitDefId(3)))), (80, flash, None)];
        let refs: Vec<&(i32, UnitDefId, Option<(UnitId, UnitDefId)>)> = lost.iter().collect();
        let w = brain.loss_words(&refs, 300.0, Some(party), &parties);
        assert_eq!(w.count, 3);
        assert_eq!(w.to_whom, "2 to a Sentry (armllt) (dead now), something unseen, none to this party");
        assert_eq!(w.verdict, ": it is losing this fight", "turrets count as the fight, dead or standing");
        let to_party = vec![(60, flash, Some((party.ids[0], UnitDefId(2)))), (70, flash, Some((party.ids[1], UnitDefId(2))))];
        let refs: Vec<&(i32, UnitDefId, Option<(UnitId, UnitDefId)>)> = to_party.iter().collect();
        let w = brain.loss_words(&refs, 300.0, Some(party), &parties);
        assert_eq!((w.to_whom.as_str(), w.verdict), ("2 to this party", ": it is losing this fight"));
        let elsewhere = vec![(60, flash, Some((UnitId(77), UnitDefId(2)))), (70, flash, Some((UnitId(78), UnitDefId(2))))];
        let refs: Vec<&(i32, UnitDefId, Option<(UnitId, UnitDefId)>)> = elsewhere.iter().collect();
        let w = brain.loss_words(&refs, 300.0, Some(party), &parties);
        assert_eq!((w.to_whom.as_str(), w.verdict), ("2 to a Centurion (armwar) out of sight now, none to this party", ""), "losses to something else carry no verdict");
        let standing = brain.loss_words(&refs, 300.0, Some(party), &parties);
        assert_eq!(standing.share, "a large share");
        let _ = sentry;
    }

    /// A front of enemies 250 apart is one party by the chain; kept, it is several, none with a member over
    /// `KEPT_RADIUS` from its place, each unit under the same name a second later when the front has walked on, and
    /// with `beside` a party's fight counts its neighbours within `BESIDE` and its words name them.
    #[test]
    fn a_front_is_kept_as_bounded_parties_under_steady_names() {
        let brain = brain_of(&["armflash"]);
        let front = |shift: f32| (0..11).map(|i| enemy(100 + i, 1, at(1000.0 + shift + 250.0 * i as f32, 1000.0))).collect::<Vec<_>>();
        let enemies = front(0.0);
        let next = std::cell::Cell::new(1);
        assert_eq!(brain.enemy_parties(&enemies, &[], &next, false, false).len(), 1);
        let first = brain.enemy_parties(&enemies, &[], &next, true, true);
        assert!(first.len() > 2, "{}", first.len());
        for party in &first {
            assert!(enemies.iter().filter(|e| party.ids.contains(&e.id)).all(|e| e.pos.dist2d(party.at) < super::KEPT_RADIUS), "{party:?}");
            assert!(party.fighters() > party.ids.len() && party.beside_words.contains("within 600"), "{party:?}");
        }
        assert_eq!(first.iter().map(|p| p.ids.len()).sum::<usize>(), 11);
        let walked = front(60.0);
        let second = brain.enemy_parties(&walked, &first, &next, true, true);
        for enemy in &walked {
            let name = |parties: &[super::Party]| parties.iter().find(|p| p.ids.contains(&enemy.id)).map(|p| p.name.clone());
            assert_eq!(name(&first), name(&second));
        }
    }

    /// A building of ours whose health falls beside an enemy builder, with no hit on it, is said on that builder's
    /// party as being taken apart, with what is left and when it is gone; a party without a weapon is said unarmed,
    /// never as outweighing us or choosing the fight.
    #[test]
    fn a_builder_taking_a_building_apart_is_said_and_an_unarmed_party_reads_unarmed() {
        let mut brain = brain_of(&["armflash", "armllt", "armfav"]);
        // The Rover stands in for a resurrection bot: no weapon, build power, a short build reach.
        let lazarus = &mut brain.world.hello.unit_defs[2];
        (lazarus.weapon_count, lazarus.build_speed, lazarus.build_distance) = (0, 200.0, 96.0);
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, health: f32, events: Vec<Event>| Tick {
            frame,
            late: 0,
            events,
            snapshot: Snapshot {
                metal: resource(),
                energy: resource(),
                wind: 0.0,
                own_units: vec![OwnUnit { health, max_health: 100.0, ..own(1, 2, at(1000.0, 1000.0)) }, own(2, 1, at(1500.0, 1000.0))],
                allies: Vec::new(),
                enemies: vec![enemy(9, 3, at(1100.0, 1000.0))],
                wrecks: None,
                sensors: None,
            },
        };
        brain.track_takings(&tick(30, 100.0, Vec::new()));
        brain.track_takings(&tick(60, 90.0, Vec::new()));
        let last = tick(90, 80.0, Vec::new());
        brain.track_takings(&last);
        let parties = brain.enemy_parties(&last.snapshot.enemies, &[], &std::cell::Cell::new(1), false, false);
        let (what, _) = parties[0].harming.clone().expect("the taking");
        assert_eq!(what, "taking apart our Sentry (armllt), 80% left and gone in about 8 s");
        assert!(parties[0].unarmed);
        let odds = brain.odds_words(&[&last.snapshot.own_units[1]], &parties[0], &last.snapshot.enemies);
        assert!(odds.starts_with("it is unarmed and cannot fight back") && !odds.contains("chooses the fight") && !odds.contains("outweighs"), "{odds}");
        // Health lost to a weapon is a hit, not a reclaim: nothing is taken apart.
        let mut brain = brain_of(&["armflash", "armllt", "armfav"]);
        let lazarus = &mut brain.world.hello.unit_defs[2];
        (lazarus.weapon_count, lazarus.build_speed, lazarus.build_distance) = (0, 200.0, 96.0);
        brain.track_takings(&tick(30, 100.0, Vec::new()));
        brain.track_takings(&tick(60, 90.0, vec![Event::UnitDamaged { unit: UnitId(1), attacker: None, damage: 10.0, from: None, weapon: None }]));
        assert!(brain.takings.is_empty());
    }
}
