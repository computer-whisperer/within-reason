//! The picture: the game as one JSON state for Jev, written by actor and qualitative first (Jev is documented
//! weak at arithmetic and at comparing quantities, and distracted by unrelated state), with the player's
//! instructions at the top. Places and enemy parties are named here so that the menus' answers can name them back.

use std::collections::BTreeMap;
use crate::strategist::shared::Allowance;

use bot_protocol::{EnemyUnit, Event, OwnUnit, Tick, UnitDefId, Vec3};
use serde_json::{Value, json};

use super::super::roster::Kit;
use super::glossary;
use super::super::{Brain, FRAMES_PER_SECOND};
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
const PARTY_RADIUS: f32 = 400.0;
/// A turret whose reach ends this short of a party's centre still covers its edge.
const TURRET_MARGIN: f32 = 150.0;
/// A party whose reach beats the group's longest by this much kills it on the approach (a Bull's 460 or a beamer's 490 against a Stout's 350).
const OUTRANGE_MARGIN: f32 = 60.0;
/// A party this near a place or an actor is "near" it.
const NEAR: f32 = 800.0;
/// The enemy's soldiers seen within this long count in what we know of its army.
const ARMY_MEMORY: i32 = 3 * 60 * FRAMES_PER_SECOND;

#[derive(Clone, Debug)]
pub(crate) struct Place {
    pub name: String,
    pub at: Vec3,
    pub spot: Option<usize>,
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
    /// What it is shooting now, from the hits of the last seconds (H-HANDS-PARTY-KILLING): "our Vehicle Plant
    /// (armvp), 2 of our Solar Collector (armsolar)" and the metal of those units.
    pub killing: Option<(String, f32)>,
    /// The enemy's armed buildings whose reach covers where it stands (a lone unit under a nest's turrets is not a
    /// lone unit): the metal of those that hit ground, of those that hit air, and words ("3 turrets: 1 armhlt,
    /// 2 armllt"; empty under none).
    pub turret_metal: f32,
    pub turret_metal_air: f32,
    pub turrets: String,
}

#[derive(Clone)]
pub(crate) struct Picture {
    pub state: Value,
    pub places: Vec<Place>,
    pub parties: Vec<Party>,
    /// The metal store as a number, for the hands' own rules (a timed assist at an empty store is skipped).
    pub metal_stored: f32,
}

pub(crate) fn clock(frame: i32) -> String {
    let seconds = frame / FRAMES_PER_SECOND;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn stock_words(current: f32, storage: f32) -> &'static str {
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

fn flow_words(income: f32, usage: f32, current: f32, storage: f32) -> String {
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

    /// A unit for a menu line or an actor's `is`: "Pawn (armpw; raider bot, 54 metal): gloss".
    pub(super) fn unit_words(&self, def: UnitDefId) -> String {
        let internal = self.name(def);
        let metal = self.world.def(def).map_or(0.0, |d| d.metal_cost);
        match glossary::entry(internal) {
            Some(e) if !e.name.is_empty() => {
                let gloss = if e.gloss.is_empty() { String::new() } else { format!(": {}", e.gloss) };
                format!("{} ({internal}; {}, {metal:.0} metal){gloss}", e.name, if e.class.is_empty() { self.class_words(def) } else { e.class.clone() })
            }
            _ if self.world.is_commander_def(def) => format!("{internal} (our commander; the strongest builder, a good fighter; the game is lost if it dies)"),
            _ => format!("{internal} ({}; {metal:.0} metal)", self.class_words(def)),
        }
    }

    /// A type in the picture's prose: "Solar Collector (armsolar)", or the internal name and its class.
    pub(super) fn short_words(&self, def: UnitDefId) -> String {
        let internal = self.name(def);
        match glossary::entry(internal) {
            Some(e) if !e.name.is_empty() => format!("{} ({internal})", e.name),
            _ => format!("{internal} ({})", self.class_words(def)),
        }
    }

    fn composition_words(&self, units: &[&OwnUnit]) -> String {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for unit in units {
            *counts.entry(self.short_words(unit.def)).or_default() += 1;
        }
        counts.iter().map(|(name, n)| format!("{n} {name}")).collect::<Vec<_>>().join(", ")
    }

    /// The player's whitelist for an actor (`produce`, H-HANDS-PRODUCE): its own, else `all_builders` for a builder,
    /// else `all`, else none.
    pub(super) fn allowed_units(&self, actor: &str) -> Option<Allowance> {
        let shared = self.strategist.as_ref()?;
        let allowed = shared.allowed.lock().unwrap();
        let builder = actor.starts_with("commander") || actor.starts_with("constructor_");
        allowed.get(actor).or_else(|| if builder { allowed.get("all_builders") } else { None }).or_else(|| allowed.get("all")).cloned()
    }

    /// The named place nearest `pos`, with its grid cell, or the grid cell alone.
    pub(super) fn place_words(&self, places: &[Place], pos: Vec3) -> String {
        let grid = self.world.grid(pos);
        match places.iter().map(|p| (p.at.dist2d(pos), p)).min_by(|a, b| a.0.total_cmp(&b.0)) {
            Some((d, place)) if d < 300.0 => format!("{} ({grid})", place.name),
            Some((d, place)) if d < 900.0 => format!("{:.0} from {} ({grid})", d, place.name),
            _ => format!("({grid})"),
        }
    }

    /// Enemy mobile units in sight, grouped into parties, nearest home first. A party keeps the name it had in the
    /// last picture while any member of it is still in it (H-HANDS-PARTY-NAMES: named by distance from home, the
    /// names shuffled as parties moved, and the player's "party_2" in its orders meant another party by the next
    /// ask); a new party takes a number never used in this game.
    pub(super) fn enemy_parties(&self, enemies: &[EnemyUnit], previous: &[Party], next_name: &std::cell::Cell<usize>) -> Vec<Party> {
        let mobile: Vec<&EnemyUnit> = enemies.iter().filter(|e| e.def.is_none_or(|d| self.world.def(d).is_some_and(|d| d.speed > 0.0))).collect();
        let mut party_of: Vec<Option<usize>> = vec![None; mobile.len()];
        let mut parties: Vec<Party> = Vec::new();
        for start in 0..mobile.len() {
            if party_of[start].is_some() {
                continue;
            }
            party_of[start] = Some(parties.len());
            let mut members = vec![start];
            let mut next = 0;
            while next < members.len() {
                let from = mobile[members[next]].pos;
                for other in 0..mobile.len() {
                    if party_of[other].is_none() && mobile[other].pos.dist2d(from) < PARTY_RADIUS {
                        party_of[other] = Some(parties.len());
                        members.push(other);
                    }
                }
                next += 1;
            }
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
            let ids: Vec<bot_protocol::UnitId> = members.iter().map(|i| mobile[*i].id).collect();
            let killing = self.killing_words(&ids, None);
            let (turret_metal, turret_metal_air, turrets) = self.turrets_covering(at);
            parties.push(Party { name: String::new(), ids, at, metal, composition, has_commander, killing, turret_metal, turret_metal_air, turrets });
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
        for party in parties.iter_mut().filter(|p| p.name.is_empty()) {
            party.name = format!("party_{}{}", next_name.get(), self.seat_tag());
            next_name.set(next_name.get() + 1);
        }
        parties
    }

    /// What the units `ids` have hit in the last seconds, buildings first, one entry per kind with its count, and the
    /// metal of the victims; nothing when they have hit nothing of ours.
    /// What the party `ids` is hitting now; `among`: only these victims (a group's own soldiers) count.
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
    /// What each factory and building builder would draw at full speed on what it builds now: (actor name, energy a
    /// second, metal a second). A factory on what stands on its pad, else its queue's first unit; a builder on its
    /// build task (docs/design/2026-09-22-energy-draw.md).
    pub(super) fn production_draws(&self, own: &[OwnUnit], pianist: &Pianist) -> Vec<(String, f32, f32)> {
        let rate = |power: f32, def: UnitDefId| self.world.def(def).filter(|d| d.build_time > 0.0).map_or((0.0, 0.0), |d| (power * d.energy_cost / d.build_time, power * d.metal_cost / d.build_time));
        let mut draws = Vec::new();
        for unit in own.iter().filter(|u| !u.being_built) {
            let Some(def) = self.world.def(unit.def) else { continue };
            let building = if self.world.is_factory_def(unit.def) {
                own.iter().find(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| u.def).or_else(|| pianist.lab_queue.get(&unit.id).and_then(|q| q.first()).map(|(d, _)| *d))
            } else if self.world.is_mobile_builder(unit.def) {
                match pianist.tasks.get(&unit.id) {
                    Some(Task::Build { def, .. }) => Some(def.clone()),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(target) = building {
                let (energy, metal) = rate(def.build_speed, target);
                if energy > 0.0 {
                    draws.push((self.actor_name(unit.id), energy, metal));
                }
            }
        }
        draws
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
        for enemy in enemies.iter().filter(|e| party.ids.contains(&e.id)) {
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
        let party_reach = enemies.iter().filter(|e| party.ids.contains(&e.id)).filter_map(|e| e.def).map(reach_of).fold(0.0, f32::max);
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
        if units.is_empty() {
            "we have nobody to send".to_string()
        } else if !self.force_can_hit(&ours, self.all_air(&theirs)) {
            "we cannot hit it: nothing in this group shoots at what it is".to_string()
        } else if !self.force_can_hit(&theirs, self.all_air(&ours)) {
            "it cannot hit us: nothing there shoots at what this group is".to_string()
        } else if ratio >= 2.5 {
            format!("we outweigh it heavily{reach_words}")
        } else if ratio >= 1.3 {
            format!("we outweigh it{reach_words}")
        } else if ratio >= 0.8 {
            format!("an even fight{reach_words}")
        } else {
            format!("it outweighs us{reach_words}")
        }
    }

    /// The enemy's armed buildings whose reach covers `at`, with a margin for a party's spread: the metal of those
    /// that hit ground, of those that hit air, and words for the lines. Standing buildings are remembered where
    /// they were seen, so a nest counts before it is in sight again.
    pub(super) fn turrets_covering(&self, at: Vec3) -> (f32, f32, String) {
        let mut ground = 0.0;
        let mut air = 0.0;
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for (def, pos, _) in self.enemy_buildings.values() {
            let Some(d) = self.world.def(*def) else { continue };
            if d.weapon_count == 0 || d.reach <= 0.0 || pos.dist2d(at) > d.reach + TURRET_MARGIN {
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

    fn task_words(&self, task: Option<&Task>, unit: &OwnUnit, places: &[Place], frame: i32) -> String {
        let ago = |since: i32| format!("{} s ago", (frame - since) / FRAMES_PER_SECOND);
        let mut words = self.task_course(task, unit, places, frame);
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

    pub(super) fn task_course(&self, task: Option<&Task>, unit: &OwnUnit, places: &[Place], frame: i32) -> String {
        let ago = |since: i32| format!("{} s ago", (frame - since) / FRAMES_PER_SECOND);
        match task {
            None if unit.idle => "idle, waiting for an order".into(),
            None => "finishing an order".into(),
            Some(Task::Build { def, near, started, ordered, .. }) => {
                let what = self.short_words(*def);
                let walk = unit.pos.dist2d(*near);
                if *started {
                    format!("building a {what} at {}, ordered {}", self.place_words(places, *near), ago(*ordered))
                } else if walk > 120.0 {
                    format!("walking to build a {what} at {}, {walk:.0} to go, ordered {}", self.place_words(places, *near), ago(*ordered))
                } else {
                    format!("about to build a {what} at {}, ordered {}", self.place_words(places, *near), ago(*ordered))
                }
            }
            Some(Task::Assist { lab, since }) => format!("helping lab_{} build, since {}", lab.0, ago(*since)),
            Some(Task::Reclaim { at, since }) => format!("taking apart wrecks at {}, since {}", self.place_words(places, *at), ago(*since)),
            Some(Task::ReclaimUnit { target, since }) => format!("taking apart our {} on the player's order, since {}", self.known_units.get(target).map_or("unit".to_string(), |(def, _)| format!("{}_{}", self.name(*def), target.0)), ago(*since)),
            Some(Task::Repair { target, since }) => format!("repairing our {}, since {}", self.known_units.get(target).map_or("unit", |(def, _)| self.name(*def)), ago(*since)),
            Some(Task::Walk { place, to, since }) => format!("walking to {place}, {:.0} to go, since {}", unit.pos.dist2d(*to), ago(*since)),
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
        let mut places: Vec<Place> = vec![Place { name: "home".into(), at: self.home, spot: None }];
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
            places.push(Place { name: format!("spot_{i}"), at: spots[*i], spot: Some(*i) });
        }
        let passages = self.passages();
        for (n, passage) in passages.iter().take(PASSAGES).enumerate() {
            places.push(Place { name: format!("passage_{}{}", n + 1, self.seat_tag()), at: passage.at, spot: None });
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
        let tool_words = pianist.standing.tool_words();
        for token in instructions.split(|c: char| !c.is_ascii_alphanumeric() && c != '_').chain(tool_words.iter().map(String::as_str)) {
            if let Some(i) = token.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                // An islet spot nobody can walk to is no place to send anyone (pianist-player-7: the ball stood 151 s
                // short of spot_31 on the shore while the player named it).
                if i < spots.len() && !places.iter().any(|p| p.spot == Some(i)) && self.reachable_by_any_class(spots[i]) {
                    places.push(Place { name: format!("spot_{i}"), at: spots[i], spot: Some(i) });
                }
            } else if let Some(n) = token.strip_prefix("passage_").and_then(|n| n.parse::<usize>().ok()) {
                if n > PASSAGES && n <= passages.len() && !places.iter().any(|p| p.name == *token) {
                    places.push(Place { name: format!("passage_{n}{}", self.seat_tag()), at: passages[n - 1].at, spot: None });
                }
            }
        }
        let marks: BTreeMap<String, (f32, f32)> = self.strategist.as_ref().map(|s| s.marks.lock().unwrap().clone()).unwrap_or_default();
        for (name, (x, z)) in &marks {
            if !places.iter().any(|p| p.name == *name) {
                places.push(Place { name: name.clone(), at: Vec3 { x: *x, y: 0.0, z: *z }, spot: None });
            }
        }
        // H-HANDS-SHELLED: where fire from out of sight likeliest comes from, while it lasts.
        let shelling = self.shelling();
        if let Some(s) = &shelling {
            places.push(Place { name: "shelling".into(), at: s.at, spot: None });
        }
        let parties = match self.pianist.as_ref() {
            Some(p) => self.enemy_parties(&snapshot.enemies, &p.parties, &p.next_party),
            None => self.enemy_parties(&snapshot.enemies, &[], &std::cell::Cell::new(1)),
        };

        let mut place_entries: BTreeMap<String, Value> = BTreeMap::new();
        for place in &places {
            let mut entry = json!({ "grid": self.world.grid(place.at) });
            let walk = self.walk_from_home(place.at);
            entry["from_home"] = json!(format!("{} ({walk:.0} on foot)", distance_words(walk)));
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
                    if !self.reachable_on_foot(spot) {
                        beside_words.push_str("; under water or off our ground: our soldiers and ordinary constructors cannot get there, an amphibious or hover constructor or a construction ship can");
                    }
                    if let Some(seen) = their_spot(spot) {
                        let turrets = self.enemy_buildings.values().filter(|(def, pos, _)| pos.dist2d(spot) < 500.0 && self.world.def(*def).is_some_and(|d| d.weapon_count > 0)).count();
                        format!("their extractor, seen {} ago{}", clock(frame - seen), if turrets > 0 { format!(", {turrets} turret(s) beside it") } else { String::new() })
                    } else if let Some(u) = on_spot {
                        format!("our extractor{}{beside_words}", if u.being_built { " (being built)" } else { "" })
                    } else {
                        let taker = pianist.tasks.iter().find_map(|(id, t)| matches!(t, Task::Build { spot: Some(s), .. } if *s == i).then_some(*id));
                        match (taker, self.spot_seen(i)) {
                            (Some(id), _) => format!("free metal spot, {} is on its way to take it{beside_words}", self.actor_name(id)),
                            (None, None) => format!("metal spot never in our sight: nobody knows what stands here{beside_words}"),
                            (None, Some(seen)) if frame - seen > LONG_UNSEEN => format!("free metal spot when last in sight, {} ago{beside_words}", clock(frame - seen)),
                            (None, Some(_)) => format!("free metal spot{beside_words}"),
                        }
                    }
                }
                None if place.name == "home" => "our start: the lab and the base stand here".into(),
                None if place.name == "shelling" => {
                    let s = shelling.as_ref().expect("a shelling place has a shelling");
                    // An estimate, said as one: the old words ("advancing a group onto it kills it") sent groups under blind
                    // fire toward a moving point for two minutes at a time (escalate-5 12:29, escalate-7 12:29, fixes-1 15:33).
                    match &s.attributed {
                        Some(a) => format!("where the {} whose fire hits us from out of our sight was last seen, {} s ago (not a sighting now: it may have moved); its range is {:.0}, {} hits on us in the last {} s, the last {} s ago; anything of ours within {:.0} of it is in its reach; what sees it or outranges it decides, as the instructions say", a.name, (frame - a.seen) / super::super::FRAMES_PER_SECOND, s.range, s.hits, super::super::shelling::SHELL_MEMORY / super::super::FRAMES_PER_SECOND, (frame - s.last) / super::super::FRAMES_PER_SECOND, s.range),
                        None => format!("an estimate, not a sighting: where the {} shelling us from out of our sight likeliest stands, four fifths of its range along the hits' direction (it moves with each hit); its range is {:.0}, {} hits on us in the last {} s from the {}, the last {} s ago; anything of ours within {:.0} of it is in its reach and cannot see it; what sees it (a scout, a radar) or outranges it decides, as the instructions say", self.weapon_words(&s.weapon), s.range, s.hits, super::super::shelling::SHELL_MEMORY / super::super::FRAMES_PER_SECOND, super::super::shelling::compass(s.dir), (frame - s.last) / super::super::FRAMES_PER_SECOND, s.range),
                    }
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
            for (def, pos, seen) in self.enemy_buildings.values() {
                if pos.dist2d(place.at) < 500.0 {
                    *near.entry(self.short_words(*def)).or_default() += 1;
                    oldest = oldest.min(*seen);
                }
            }
            if !near.is_empty() {
                let turrets = self.enemy_buildings.values().filter(|(def, pos, _)| pos.dist2d(place.at) < 500.0 && self.world.def(*def).is_some_and(|d| d.weapon_count > 0)).count();
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
        let draws: Vec<(String, f32, f32)> = self.production_draws(own, pianist);
        let draw_total: f32 = draws.iter().map(|(_, e, _)| e).sum();
        let budget = if draws.is_empty() {
            String::new()
        } else {
            let short = draw_total - e.income;
            let mut fixes: Vec<String> = Vec::new();
            for (name, unit_income) in [("armsolar", 20.0), ("corsolar", 20.0), ("armadvsol", 75.0), ("coradvsol", 75.0), ("armfus", 1000.0), ("corfus", 1000.0)] {
                if let Some(def) = self.world.def_named(name) && self.faction_reaches(def) {
                    fixes.push(format!("{:.1} {}", short.max(0.0) / unit_income, self.short_words(def)));
                }
            }
            format!(" Our production at full speed would draw {draw_total:.0} a second ({}); against the income that is {}", draws.iter().map(|(who, e, _)| format!("{who} {e:.0}")).collect::<Vec<_>>().join(", "), if short > 0.0 { format!("short by {short:.0}, which is {}", fixes.join(" or ")) } else { format!("{:.0} to spare", -short) }) + "."
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
            .take(3)
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
                format!("{}: {} worth {:.0} metal at {}{under}, {} from home, {heading}{}{}", p.name, p.composition, p.metal, self.place_words(&places, p.at), distance_words(p.at.dist2d(self.home)), near_ours.unwrap_or_default(), p.killing.as_ref().map_or(String::new(), |(what, metal)| format!("; killing {what} ({metal:.0} metal) now")))
            })
            .collect();
        let known_soldiers: Vec<&(UnitDefId, Vec3, i32)> = self.enemy_soldiers.values().filter(|(_, _, seen)| frame - seen < ARMY_MEMORY).collect();
        let known_metal: f32 = known_soldiers.iter().filter_map(|(def, _, _)| self.world.def(*def)).map(|d| d.metal_cost).sum();
        let mut remembered: BTreeMap<String, BTreeMap<&str, usize>> = BTreeMap::new();
        for (def, pos, _) in self.enemy_buildings.values() {
            *remembered.entry(self.world.grid(*pos)).or_default().entry(self.name(*def)).or_default() += 1;
        }
        let remembered: Vec<String> = remembered.iter().map(|(grid, kinds)| format!("{grid}: {}", kinds.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "))).collect();
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
                entry["doing"] = json!(self.task_words(pianist.tasks.get(&unit.id), unit, &places, frame));
                if let Some(next) = pianist.queued.get(&unit.id) {
                    entry["next"] = json!(format!("queued to start the moment this is done: {}", self.task_words(Some(next), unit, &places, frame)));
                }
                if let Some(steps) = pianist.scripts.get(&name).filter(|s| !s.is_empty()) {
                    entry["list"] = json!(format!("the player's list, done by the bot without asking: {}", steps.iter().cloned().collect::<Vec<_>>().join(", ")));
                }
                if let Some(words) = pianist.standing.words(&name) {
                    entry["standing"] = json!(format!("standing orders in force, played by the bot when they apply: {words}"));
                }
                let from_home = unit.pos.dist2d(self.home);
                entry["from_home"] = json!(format!("{} ({from_home:.0})", distance_words(from_home)));
                if let Some(party) = parties.iter().filter(|p| p.at.dist2d(unit.pos) < NEAR).min_by(|a, b| a.at.dist2d(unit.pos).total_cmp(&b.at.dist2d(unit.pos))) {
                    let alone: Vec<&OwnUnit> = vec![unit];
                    let mut line = format!("{} ({}) {:.0} away: against this unit alone, {}", party.name, party.composition, party.at.dist2d(unit.pos), self.odds_words(&alone, party, &snapshot.enemies));
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
                let on_pad = own.iter().filter(|u| u.being_built && u.pos.dist2d(unit.pos) < 120.0 && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| format!("{} ({:.0}% built)", self.short_words(u.def), u.health / u.max_health.max(1.0) * 100.0)).next();
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
                    let words: Vec<String> = list
                        .iter()
                        .filter_map(|e| {
                            let (n, cap) = super::allowance(e);
                            let def = self.world.def_named(n).filter(|d| can_build.contains(d))?;
                            let made = pianist.produced.get(&(unit.id, n.to_string())).copied().unwrap_or(0);
                            Some(match cap {
                                Some(cap) if made >= cap => format!("{} (all {cap} allowed made: no more)", self.short_words(def)),
                                Some(cap) => format!("{} ({} more allowed)", self.short_words(def), cap - made),
                                None => self.short_words(def),
                            })
                        })
                        .collect();
                    let used_up = list.iter().filter(|e| self.world.def_named(super::allowance(e).0).is_some_and(|d| can_build.contains(&d))).all(|e| {
                        let (n, cap) = super::allowance(e);
                        cap.is_some_and(|cap| pianist.produced.get(&(unit.id, n.to_string())).copied().unwrap_or(0) >= cap)
                    });
                    entry["allowed"] = json!(if words.is_empty() {
                        "the player's list names nothing this lab can build: it builds nothing until a new `produce` list".to_string()
                    } else if used_up {
                        "the player's allowance is used up (every count made): it builds nothing until a new `produce` list".to_string()
                    } else {
                        format!("the player allows only: {}", words.join(", "))
                    });
                }
                let named = self.allowed_units(&name).and_then(|a| a.group);
                let rally = pianist.rally.get(&unit.id).map(|g| format!("group_{g}"));
                entry["output_joins"] = json!(match (named.as_deref(), rally) {
                    (Some("new"), _) => "a fresh group of this factory's own from the next soldier (`produce` ... group)".to_string(),
                    (Some(g), _) => format!("{g}, by your `produce` order"),
                    (None, Some(g)) => format!("{g}, this factory's own group (name another in `produce`, or `new`)"),
                    (None, None) => "a group of this factory's own, made on its first soldier (name one in `produce`)".to_string(),
                });
                entry["health"] = json!(health_words(unit.health / unit.max_health));
            }
            if let Some((_, energy, metal)) = draws.iter().find(|(who, _, _)| *who == name) {
                entry["draw"] = json!(format!("draws about {energy:.0} energy and {metal:.1} metal a second at full speed on what it builds now"));
            }
            actors.insert(name, entry);
        }
        let scouts: Vec<String> = pianist
            .groups
            .iter()
            .filter(|g| g.members.len() == 1)
            .filter_map(|g| match &g.task {
                GroupTask::Move { place, to, fight: false, .. } => {
                    let unit = g.units(own).first().copied()?;
                    Some(format!("group_{} ({}) walking to look at {place}, {:.0} to go", g.name, self.name(unit.def), unit.pos.dist2d(*to)))
                }
                _ => None,
            })
            .collect();
        for group in &pianist.groups {
            let units = group.units(own);
            let Some(centre) = super::groups::centre_of(&units) else { continue };
            let metal: f32 = units.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            let health: f32 = units.iter().map(|u| u.health / u.max_health).sum::<f32>() / units.len().max(1) as f32;
            let ago = |since: i32| format!("{} s", (frame - since) / FRAMES_PER_SECOND);
            let doing = match &group.task {
                GroupTask::Hold { since, committed } => format!("holding for {}{}", ago(*since), if *committed { ", fighting everything here, turrets included, since it arrived by advancing" } else { "" }),
                GroupTask::Move { to, place, fight, since } => format!("{} to {place}, {:.0} to go, for {}", if *fight { "advancing" } else { "walking" }, centre.dist2d(*to), ago(*since)),
                GroupTask::Engage { party, at, since, target, .. } => {
                    let name = parties.iter().find(|p| p.ids.iter().any(|id| party.contains(id))).map_or("a party now out of sight".to_string(), |p| p.name.clone());
                    format!("attacking {name}{} at {}, for {}", if target.is_some() { " (one named unit of it, until it dies)" } else { "" }, self.place_words(&places, *at), ago(*since))
                }
            };
            // A hunt out is the group's doing too: the entry said "attacking party_22, for 15 s" while four of its
            // eight hunted and the rest had no order (onepass-player-8 7:22-7:31).
            let doing = match &group.hunt {
                Some(hunt) => format!("{} of its soldiers hunting {} since {} ago; the rest {doing}", hunt.hunters.len(), hunt.party, ago(hunt.since)),
                None => doing,
            };
            let mut entry = json!({
                "units": format!("{}: {} ({} soldiers worth {metal:.0} metal{})", soldier_words(units.len(), metal), self.composition_words(&units), units.len(), if group.domain == crate::world::Domain::Ground { String::new() } else { format!("; an {} group", group.domain.word()) }),
                "at": self.place_words(&places, centre),
                "health": format!("{} on average", health_words(health)),
                "doing": doing,
            });
            if let Some(words) = self.footwork_of(&group.name).words() {
                entry["lane"] = json!(words);
            }
            if let Some(words) = pianist.standing.words(&format!("group_{}", group.name)) {
                entry["standing"] = json!(format!("standing orders in force, played by the bot when they apply: {words}"));
            }
            // The last 30 s, not "since the player's last orders": turns come every ten seconds now, and the line read
            // "lost 1" while the ball had lost eleven in eleven seconds (wake-1, 18:14). A share in words beside the
            // number: Jev is not a calculator (docs.typesafe.ai/model-jaggedness).
            let lost_lately: Vec<&(i32, UnitDefId)> = group.losses.iter().filter(|(f, _)| frame - f <= 30 * FRAMES_PER_SECOND).collect();
            if !lost_lately.is_empty() {
                let lost_metal: f32 = lost_lately.iter().filter_map(|(_, def)| self.world.def(*def)).map(|d| d.metal_cost).sum();
                let share = lost_metal / (lost_metal + metal).max(1.0);
                let words = if share < 0.1 { "a few" } else if share < 0.25 { "a noticeable share" } else if share < 0.5 { "a large share: it is losing this fight" } else { "most of it: it is being wiped out" };
                let last = lost_lately.iter().map(|(f, _)| *f).max().unwrap_or(frame);
                entry["losses"] = json!(format!("lost {} of its {} soldiers ({lost_metal:.0} metal, {words}) in the last 30 s, the last {} s ago", lost_lately.len(), units.len() + lost_lately.len(), (frame - last) / FRAMES_PER_SECOND));
            }
            if let Some(seconds) = group.stalled_seconds(frame).filter(|s| *s >= 20) {
                entry["progress"] = json!(format!("has not got nearer its goal for {seconds} s: stalled"));
            }
            // The army in pieces, said on both sides (escalate-6: 19 detachments and 15 splits in ten minutes, five to
            // nine groups alive, dying one by one; nothing in the picture counted them).
            let children: Vec<&super::Group> = pianist.groups.iter().filter(|g| g.parent.as_deref() == Some(group.name.as_str()) && !g.members.is_empty()).collect();
            if !children.is_empty() {
                let soldiers: usize = children.iter().map(|g| g.members.len()).sum();
                let oldest = children.iter().map(|g| frame - g.born).max().unwrap_or(0) / FRAMES_PER_SECOND;
                entry["detachments_out"] = json!(format!(
                    "{} group{} of {soldiers} soldiers split from it in the last {} s, still out: {}",
                    children.len(), if children.len() == 1 { "" } else { "s" }, oldest,
                    children.iter().map(|g| format!("group_{} ({})", g.name, g.members.len())).collect::<Vec<_>>().join(", ")
                ));
            }
            if let Some(parent) = &group.parent {
                entry["split_from"] = json!(format!("group_{parent}, {} ago", ago(group.born)));
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
            let fleeing = units.iter().filter(|u| self.lane.fleeing(u.id)).count();
            if fleeing > 0 {
                entry["footwork"] = json!(format!("{fleeing} of its {} soldiers are being held back by their own footwork this second: stepping out of a turret's reach they were not sent against, or out of a fight they would die in", units.len()));
            }
            // The nearest party with the odds against this group and what it is killing: the words the `do` question
            // weighs (they were only on the engage option's line before).
            if let Some((d, p)) = parties.iter().map(|p| (p.at.dist2d(centre), p)).filter(|(d, _)| *d < NEAR).min_by(|a, b| a.0.total_cmp(&b.0)) {
                // Its own soldiers under fire, said as such; what the party kills elsewhere is not this group's loss
                // (wake-3: "killing 3 of our Blitz" on a group's line read as its own, and it retreated from a party it
                // outweighed in 7 of 51 such asks, 0 of 69 without the clause).
                let own_ids: Vec<bot_protocol::UnitId> = units.iter().map(|u| u.id).collect();
                let killing = match (self.killing_words(&p.ids, Some(&own_ids)), &p.killing) {
                    (Some((what, metal)), _) => format!("; it is hitting this group now: {what} ({metal:.0} metal)"),
                    (None, Some((what, metal))) => format!("; it is killing {what} ({metal:.0} metal) elsewhere, not this group"),
                    (None, None) => String::new(),
                };
                let under = if p.turrets.is_empty() { String::new() } else { format!(", under {}", p.turrets) };
                entry["enemies_near"] = json!(format!("{} ({}{under}) {d:.0} away: {}{killing}", p.name, p.composition, self.odds_words(&units, p, &snapshot.enemies)));
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
                        match &shelling {
                            Some(s) => match &s.attributed {
                                Some(a) => format!("something out of our sight, {n} hits: a {} with range {:.0}, likely the {} seen {} s ago at {} (the place `shelling`)", self.weapon_words(&s.weapon), s.range, a.name, (frame - a.seen) / super::super::FRAMES_PER_SECOND, self.place_words(&places, s.at)),
                                None => format!("something out of our sight, {n} hits: a {} with range {:.0} from the {}; its likeliest place is `shelling` at {}", self.weapon_words(&s.weapon), s.range, super::super::shelling::compass(s.dir), self.place_words(&places, s.at)),
                            },
                            None => format!("something out of our sight, {n} hits: a turret or artillery that outranges us"),
                        }
                    })
                    .collect();
                entry["under_fire"] = json!(format!("yes, this second, by {}", words.join(", ")));
            }
            actors.insert(format!("group_{}", group.name), entry);
        }

        let wind = (self.world.hello.map.wind_min + self.world.hello.map.wind_max) / 2.0;
        let rules = format!(
            "{}This map's wind averages about {wind:.0}: {}.",
            crate::texts::read(&crate::texts::HANDS_RULES),
            if wind >= 8.0 { "wind generators (40 metal) beat solar collectors here" } else { "solar collectors are the reliable energy here" }
        );
        let mut state = json!({
            "instructions": instructions,
            "clock": clock(frame),
            "rules": rules,
            "economy": economy,
            "ours": ours,
            "enemy": enemy,
            "places": place_entries,
            "actors": actors,
            "recent": pianist.recent(frame),
        });
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
        Picture { state, places, parties, metal_stored: tick.snapshot.metal.current }
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

