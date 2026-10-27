//! The brain's side of the player's session: the briefing, the field, events and triggers out.

use std::collections::{BTreeMap, HashMap, HashSet};

use bot_protocol::{Death, Event, OwnUnit, Tick, UnitDefId, UnitId, Vec3};
use serde_json::json;

use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};
use crate::strategist::shared::{Briefing, Counts, Draw, EnemyCluster, EnemyDeath, ExtractorStatus, Field, Group, NextFree, Place, RememberedBuilding, Score, Spending, TradeKind, Traded, OWN_TRADE_FRAMES, SIDE_TRADE_FRAMES};

/// One death in the trade log: a loss of ours (`ours`), or a kill by one of our own units.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Trade {
    pub frame: i32,
    pub ours: bool,
    pub metal: f32,
    pub kind: TradeKind,
}

/// The score's "soldiers near home" radius and how many free spots it names.
const SCORE_AT_HOME: f32 = 800.0;
const NEXT_FREE: usize = 5;

/// How another seat of this bot is described: with a player a seat the other seats are allies with players of their own.
fn sibling_words() -> &'static str {
    if crate::strategist::player_per_seat() { "another seat of this bot, with its own player as you are this seat's: it reads its own report and orders its own units" } else { "another seat of this bot" }
}

fn centre_of_units(units: &[&&OwnUnit]) -> Option<Vec3> {
    if units.is_empty() {
        return None;
    }
    let n = units.len() as f32;
    Some(units.iter().fold(Vec3::default(), |sum, u| Vec3 { x: sum.x + u.pos.x / n, y: 0.0, z: sum.z + u.pos.z / n }))
}

const MAX_RECENT_EVENTS: usize = 25;
/// Within this of home a loss is "at home".
const BASE_RADIUS: f32 = 1400.0;
/// The same kind of trigger wakes the strategist at most this often.
const TRIGGER_COOLDOWN_FRAMES: i32 = 60 * FRAMES_PER_SECOND;

fn clock(frame: i32) -> String {
    let seconds = frame / FRAMES_PER_SECOND;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// What killed a unit of ours, for its loss line and the strike line (H-REPORT-BLAST-CAUSE).
#[derive(Clone, Debug, PartialEq)]
enum Cause {
    /// A unit of his the engine named.
    Killer(String),
    /// A warhead whose range spans more than the map: a silo's missile. The engine names no attacker for it.
    Missile,
    /// A death blast: the engine names no attacker for one, and the last hit's weapon has no range (the game's
    /// explosion weapons carry the engine's default). Named after the nearest unit that died with a blast reaching
    /// the dead one within the window, his or our own; with none, the blast stays unnamed.
    Blast { name: Option<String>, ours: bool },
    /// Our own unit's shot: the engine names one of ours as the attacker for its fire and for its reclaim only.
    FriendlyFire(String),
    /// Taken apart by our own constructor, named when the engine names it.
    TakenApart(Option<String>),
    SelfDestruct,
    /// A gun's hit with nobody named: fire from out of sight, the shooter's type read from the weapon's name
    /// (`<unit>_<weapon>`) when one of the game's units matches.
    UnseenFire(Option<String>),
    /// The engine's own verdict: a collision, a crash, a crushing, a transport's or a factory's death.
    Engine(&'static str),
    Unseen,
}

impl std::fmt::Display for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cause::Killer(name) => write!(f, "{name}"),
            Cause::Missile => write!(f, "a missile"),
            Cause::Blast { name: Some(name), ours: false } => write!(f, "the death blast of a {name}"),
            Cause::Blast { name: Some(name), ours: true } => write!(f, "our own {name}'s death blast"),
            Cause::Blast { name: None, .. } => write!(f, "a death blast"),
            Cause::FriendlyFire(name) => write!(f, "our own {name}'s fire"),
            Cause::TakenApart(Some(name)) => write!(f, "our own {name} taking it apart"),
            Cause::TakenApart(None) => write!(f, "a reclaim"),
            Cause::SelfDestruct => write!(f, "its own self-destruct"),
            Cause::UnseenFire(Some(shooter)) => write!(f, "a {shooter} out of sight"),
            Cause::UnseenFire(None) => write!(f, "unseen fire"),
            Cause::Engine(what) => write!(f, "{what}"),
            Cause::Unseen => write!(f, "unseen"),
        }
    }
}

/// A unit that died with a death blast worth naming, within the window: what it was, where, how far the blast
/// reaches, and when it fell.
pub(super) struct RecentBlast {
    frame: i32,
    unit: UnitId,
    name: String,
    at: Vec3,
    radius: f32,
    ours: bool,
}

/// A death blast worth naming as a cause: a commander's is 5,000 at its centre and a Juggernaut's 10,600; a tank's
/// is tens, and a tank that died beside another did not kill it.
const BLAST_WORTH_NAMING: f32 = 500.0;
/// A building's footprint and the blast's falloff: a unit this far outside the radius still counts as reached.
const BLAST_SLACK: f32 = 100.0;
/// How long a death blast stays a cause after its unit's death: the victims fall after the exploder, 9-18 frames
/// after a fusion, 21-84 along a converter chain, about 210 after an advanced fusion (its death animation runs
/// before the explosion), 127 of a Juggernaut's 136 within 30 (apex60-1, apex60-2).
const BLAST_WINDOW_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// The range under which a weapon is an explosion, not a gun: the game's death explosions set no range and carry
/// the engine's default of 10 (`weapons/Unit_Explosions.lua`, `WeaponDef.cpp`); the shortest gun is a
/// commander's 262.
const EXPLOSION_RANGE_CEILING: f32 = 20.0;

/// What became of an unfinished frame of ours: killed by a named attacker, shot by something out of sight, or
/// decayed with nobody shooting it (its builder was sent elsewhere, or died).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FrameFate {
    Killed,
    ShotUnseen,
    Decayed { builder_dead: bool },
}

/// An unfinished frame of ours destroyed this think: its type, where it stood, the furthest it had been built
/// (the share of its cost put in), and what became of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FrameLoss {
    pub def: UnitDefId,
    pub pos: Vec3,
    pub share: f32,
    pub fate: FrameFate,
}

impl Brain {
    /// The player's `spending` line (H-PLAYER-SPENDING-LINE): what our factories and builders would take a second at
    /// full speed on what they build now, from the picture's draws (one source with the hands' words), and the
    /// builders with no draw of their own: helping a factory or another builder, or walking.
    fn spending(&self, own: &[OwnUnit]) -> Spending {
        use super::pianist::Task;
        let Some(pianist) = self.pianist.as_ref() else { return Spending::default() };
        let draws: Vec<Draw> = self
            .production_draws(own, pianist)
            .into_iter()
            .map(|d| Draw { who: d.who, factory: d.factory, target: self.short_words(d.target), own_power: d.own_power, energy: d.energy, metal: d.metal, helping: d.helping })
            .collect();
        let (mut helping, mut without) = (0, 0);
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            if draws.iter().any(|d| d.who == name) {
                continue;
            }
            if matches!(pianist.tasks.get(&unit.id), Some(Task::Assist { .. })) {
                helping += 1;
            } else {
                without += 1;
            }
        }
        Spending { draws, builders_helping: helping, builders_without_a_build: without }
    }

    pub(super) fn place(&self, pos: Vec3) -> Place {
        Place { grid: self.world.grid(pos), x: pos.x as i32, z: pos.z as i32 }
    }

    /// Notes something for the strategist's next look.
    pub(super) fn event(&mut self, frame: i32, text: String) {
        self.journal.note(frame, "event", serde_json::Value::Null, json!(text));
        if self.strategist.is_none() {
            return;
        }
        if self.recent_events.len() == MAX_RECENT_EVENTS {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(format!("{} {text}", clock(frame)));
    }

    /// Wakes the strategist now, unless the same kind of trigger fired within the cooldown.
    pub(super) fn trigger(&mut self, kind: &'static str, frame: i32, text: String) {
        let Some(shared) = self.strategist.clone() else { return };
        let last = self.last_trigger_frame.get(kind).copied().unwrap_or(i32::MIN / 2);
        if frame - last < TRIGGER_COOLDOWN_FRAMES {
            return;
        }
        self.last_trigger_frame.insert(kind, frame);
        self.event(frame, text.clone());
        shared.trigger(text);
    }

    /// What a death was for the trade line: a building stands still, a builder moves and builds, a soldier moves.
    fn trade_kind(&self, def: UnitDefId) -> TradeKind {
        match self.world.def(def) {
            Some(d) if d.speed == 0.0 => TradeKind::Building,
            Some(d) if d.build_speed > 0.0 => TradeKind::Builder,
            _ => TradeKind::Soldier,
        }
    }

    /// Under a commander: a death of ours, or of his that we saw, with what stood near it, for the side's fights
    /// block (`strategist/fights.rs`).
    fn publish_death(&self, tick: &Tick, kit: &Kit, unit: u32, ours: bool, name: &str, metal: f32, at: Vec3) {
        use crate::strategist::fights::{Death, THERE};
        let Some(side) = self.strategist.as_ref().and_then(|s| s.side.get()) else { return };
        let soldiers: Vec<f32> = tick.snapshot.own_units.iter().filter(|u| !u.being_built && self.is_army(u, kit) && u.pos.dist2d(at) <= THERE).map(|u| self.world.def(u.def).map_or(0.0, |d| d.metal_cost)).collect();
        let mut his: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        let mut his_metal = 0.0;
        for enemy in tick.snapshot.enemies.iter().filter(|e| !e.being_built && e.pos.dist2d(at) <= THERE) {
            let Some(def) = enemy.def.and_then(|d| self.world.def(d)) else { continue };
            *his.entry(def.name.clone()).or_default() += 1;
            his_metal += def.metal_cost;
        }
        side.deaths.lock().unwrap().push(Death {
            frame: tick.frame,
            unit,
            ours,
            name: name.to_string(),
            metal,
            at: (at.x, at.z),
            cell: self.world.grid(at),
            ours_there: (soldiers.len(), soldiers.iter().sum()),
            his_there: his.into_iter().collect(),
            his_there_metal: his_metal,
        });
    }

    /// What killed a unit of ours at `pos` (H-REPORT-BLAST-CAUSE), from the kill event's attacker and cause. The
    /// engine names an attacker for a shot (his, or our own: friendly fire) and for a reclaim (our own constructor),
    /// never for a missile or a death blast; those are told from the killing weapon (a warhead reaching beyond the
    /// map: a silo's missile, `crblmssl` at 72,000 against a map of 8,192; an explosion, which has no range) and from
    /// the units that died with a death blast reaching it within the window, his or ours. A gun's hit with nobody
    /// named is fire from out of sight, the shooter's type read from the weapon's name, never a blast.
    fn death_cause(&self, unit: UnitId, pos: Vec3, attacker: Option<UnitId>, death: &Death, blasts: &[RecentBlast], own_defs_now: &HashMap<UnitId, UnitDefId>, dead_enemies_now: &HashMap<UnitId, UnitDefId>) -> Cause {
        let map = &self.world.hello.map;
        let weapon = match death {
            Death::Weapon(w) => Some(w),
            _ => None,
        };
        let explosion = matches!(death, Death::Debris) || weapon.is_some_and(|w| w.range <= EXPLOSION_RANGE_CEILING);
        if let Some(id) = attacker {
            // A killer that died this tick itself (a Juggernaut's last shots as it fell) is still the killer.
            if let Some(def) = self.enemy_defs.get(&id).or_else(|| dead_enemies_now.get(&id)) {
                return Cause::Killer(self.name(*def).to_string());
            }
            if id != unit && let Some(def) = own_defs_now.get(&id) {
                let name = self.name(*def).to_string();
                return match death {
                    Death::Reclaimed => Cause::TakenApart(Some(name)),
                    _ if explosion => Cause::Blast { name: Some(name), ours: true },
                    // An older shim's event carries no cause: fire is the commoner reading (16 of 20 in apex60-2).
                    _ => Cause::FriendlyFire(name),
                };
            }
        }
        match death {
            Death::Reclaimed => return Cause::TakenApart(None),
            Death::SelfDestruct => return Cause::SelfDestruct,
            Death::Other(code) => {
                return match code {
                    2 | 3 => Cause::Engine("a collision"),
                    7 => Cause::Engine("being crushed"),
                    8 => Cause::Engine("a crash"),
                    14 => Cause::Engine("its transport's death"),
                    15 => Cause::Engine("its factory's death"),
                    _ => Cause::Unseen,
                };
            }
            _ => {}
        }
        if weapon.is_some_and(|w| w.range > map.width + map.height) {
            return Cause::Missile;
        }
        if let Some(w) = weapon
            && !explosion
        {
            return Cause::UnseenFire(self.unit_type_of_weapon(&w.name));
        }
        let reached = blasts.iter().filter(|b| b.unit != unit && pos.dist2d(b.at) <= b.radius + BLAST_SLACK);
        if let Some(blast) = reached.min_by(|a, b| pos.dist2d(a.at).total_cmp(&pos.dist2d(b.at))) {
            return Cause::Blast { name: Some(blast.name.clone()), ours: blast.ours };
        }
        if explosion {
            return Cause::Blast { name: None, ours: false };
        }
        Cause::Unseen
    }

    /// The unit type behind a weapon's name: the game names a unit's weapons `<unit>_<weapon>`
    /// (`corstorm_cor_bot_rocket`, `corcom_disintegrator`); an explosion's name carries no unit. The longest match
    /// wins (`armmex` against `armmexp`... `corak` against `corakm`).
    fn unit_type_of_weapon(&self, weapon: &str) -> Option<String> {
        let weapon = weapon.to_ascii_lowercase();
        self.world
            .hello
            .unit_defs
            .iter()
            .map(|d| d.name.as_str())
            .filter(|n| weapon.len() > n.len() && weapon.starts_with(*n) && weapon.as_bytes()[n.len()] == b'_')
            .max_by_key(|n| n.len())
            .map(str::to_string)
    }

    /// One line a cause for the deaths a missile or a death blast took this tick (H-REPORT-BLAST-CAUSE): where it
    /// landed (the centre of the dead), how many of ours and what they were; `true` marks a missile, which wakes the
    /// player.
    fn strike_lines(&self, strikes: &[(Cause, Vec3, UnitDefId, f32)]) -> Vec<(bool, String)> {
        let mut by_cause: BTreeMap<String, Vec<(Vec3, UnitDefId, f32)>> = BTreeMap::new();
        for (cause, pos, def, cost) in strikes {
            by_cause.entry(cause.to_string()).or_default().push((*pos, *def, *cost));
        }
        let mut lines = Vec::new();
        for (cause, deaths) in by_cause {
            let n = deaths.len() as f32;
            let at = Vec3 { x: deaths.iter().map(|d| d.0.x).sum::<f32>() / n, y: 0.0, z: deaths.iter().map(|d| d.0.z).sum::<f32>() / n };
            let metal: f32 = deaths.iter().map(|d| d.2).sum();
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            for (_, def, _) in &deaths {
                *counts.entry(self.name(*def)).or_default() += 1;
            }
            let mut counts: Vec<(usize, &str)> = counts.into_iter().map(|(name, n)| (n, name)).collect();
            counts.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
            let types: Vec<String> = counts.iter().take(6).map(|(n, name)| format!("{n} {name}")).collect();
            let missile = cause == "a missile";
            let what = if missile { "a missile landed".to_string() } else { format!("{cause} went off") };
            lines.push((missile, format!("{what} at {} ({:.0}, {:.0}): {} of ours died, {metal:.0} metal ({})", self.world.grid(at), at.x, at.z, deaths.len(), types.join(", "))));
        }
        lines
    }

    /// Notes our own losses by name, and wakes the strategist when extractors go down in numbers.
    pub(super) fn track_losses(&mut self, tick: &Tick, kit: &Kit) {
        const LOSSES_WORTH_WAKING_FOR: usize = 2;
        self.frame_losses_now.clear();
        // A frame's builder, so a decayed frame's line says whether its builder died, and a `stop` spares a frame
        // begun after the report the player read.
        for event in &tick.events {
            if let Event::UnitCreated { unit, builder: Some(builder) } = event {
                self.frame_builders.insert(*unit, (*builder, tick.frame));
            }
        }
        for enemy in &tick.snapshot.enemies {
            if let Some(def) = enemy.def {
                self.enemy_defs.insert(enemy.id, def);
            }
            self.enemy_places.insert(enemy.id, enemy.pos);
            // What each team fields, by its units' names (`sides`): a person picks the faction at the start, after
            // the script was written.
            if let (Some(team), Some(def)) = (enemy.team, enemy.def.and_then(|d| self.world.def(d)))
                && let Some(faction) = ["arm", "cor", "leg"].into_iter().find(|p| def.name.starts_with(p))
            {
                let seen = self.factions_seen.entry(team).or_default();
                seen.0.entry(faction).or_default().insert(enemy.id);
                if self.world.is_commander_def(enemy.def.expect("matched above")) {
                    seen.1 = Some(faction);
                }
            }
        }
        // Whose hand an enemy died by: the engine names the killer when it is allied with us, and it is ours when
        // it is one of our own units, standing or dead this very tick (a death blast, a shot fired as it died).
        let ours_now: HashSet<UnitId> = tick
            .snapshot
            .own_units
            .iter()
            .map(|u| u.id)
            .chain(self.known_units.keys().copied())
            .chain(tick.events.iter().filter_map(|e| match e {
                Event::UnitDestroyed { unit, .. } => Some(*unit),
                _ => None,
            }))
            .collect();
        // What killed a unit of ours when the engine names no attacker (H-REPORT-BLAST-CAUSE): the kill event's
        // weapon, and the units that died with a death blast in the last seconds. The engine withholds the attacker
        // of a missile strike and of a death blast alike (apex60-1: 15 of ours at 33:12 and 21 at 50:36 under
        // missiles, 77 at 57:17 under a Juggernaut's blast, each "lost ... to unseen"; the player wrote "I suspect a
        // second missile ... but the report doesn't confirm it" of a Juggernaut's blast at 34:43).
        let own_defs_now: HashMap<UnitId, UnitDefId> = self.known_units.iter().map(|(id, (def, _))| (*id, *def)).collect();
        let dead_enemies_now: HashMap<UnitId, UnitDefId> = tick
            .events
            .iter()
            .filter_map(|e| match e {
                Event::EnemyDestroyed { enemy, .. } => self.enemy_defs.get(enemy).map(|d| (*enemy, *d)),
                _ => None,
            })
            .collect();
        // The units that died this tick with a death blast join the recent ones: the victims fall after the exploder.
        self.recent_blasts.retain(|b| tick.frame - b.frame <= BLAST_WINDOW_FRAMES);
        for event in &tick.events {
            let (id, def, at, ours) = match event {
                Event::EnemyDestroyed { enemy, .. } => (*enemy, self.enemy_defs.get(enemy).copied(), self.enemy_places.get(enemy).copied(), false),
                Event::UnitDestroyed { unit, .. } => (*unit, self.known_units.get(unit).map(|(d, _)| *d), self.known_units.get(unit).map(|(_, p)| *p), true),
                _ => continue,
            };
            if let (Some(def), Some(at)) = (def, at)
                && let Some(blast) = self.world.def(def).and_then(|d| d.death_blast)
                && blast.damage >= BLAST_WORTH_NAMING
            {
                self.recent_blasts.push(RecentBlast { frame: tick.frame, unit: id, name: self.name(def).to_string(), at, radius: blast.radius, ours });
            }
        }
        let mut strikes: Vec<(Cause, Vec3, UnitDefId, f32)> = Vec::new();
        for event in &tick.events {
            if let Event::EnemyDestroyed { enemy, attacker } = event {
                let def = self.enemy_defs.remove(enemy);
                let worth = def.and_then(|d| self.world.def(d)).map_or(0.0, |d| d.metal_cost);
                let by_us = attacker.is_some_and(|a| ours_now.contains(&a));
                if by_us {
                    let kind = def.map_or(TradeKind::Soldier, |d| self.trade_kind(d));
                    self.trade_log.push(Trade { frame: tick.frame, ours: false, metal: worth, kind });
                    self.enemy_deaths.push(EnemyDeath { unit: enemy.0 as u32, frame: tick.frame, metal: worth, kind });
                }
                let place = self.enemy_places.remove(enemy);
                let name = def.map_or("unseen", |d| self.name(d));
                // The commander's fights block keeps every death we saw, with what stood near it.
                if let Some(at) = place {
                    self.publish_death(tick, kit, enemy.0 as u32, false, name, worth, at);
                }
                // The fights line: our kills as "killed", an ally's as its own, a death by no allied hand (the
                // engine names none: his own fire, a decay, a self-destruct) not at all.
                let line = if by_us {
                    format!("killed {name}")
                } else if attacker.is_some() {
                    format!("an ally killed {name}")
                } else {
                    continue;
                };
                if let Some(shared) = &self.strategist {
                    shared.fight(&line);
                }
                *self.fight_ledger.entry(line).or_default() += 1;
            }
            let Event::UnitDestroyed { unit, attacker, death } = event else { continue };
            let Some((def, pos)) = self.known_units.remove(unit) else { continue };
            // An extractor of ours that our own advanced extractor replaces dies by nobody's hand: an upgrade, not a
            // loss (the impedance tally, 2026-10-16: apex-v024-carrot-sym50-opus-1 15:22 to 16:36 read "extractors
            // lost in the last 3 min: 14" on nine such, and the commander ordered home guards on it).
            if attacker.is_none()
                && kit.is_extractor(def)
                && def != kit.advanced_extractor
                && tick.snapshot.own_units.iter().any(|u| u.def == kit.advanced_extractor && u.pos.dist2d(pos) < self.spot_occupied_radius())
            {
                continue;
            }
            let cost = self.world.def(def).map_or(0.0, |d| d.metal_cost);
            let builder = self.frame_builders.remove(unit).map(|(b, _)| b);
            self.frame_alone_since.remove(unit);
            // An unfinished frame: the metal put in is its peak share of the cost (left to decay it reads near 0 at
            // its end, and the report said "0% built" of every decay until 2026-10-17), and what became of it is
            // the kill event's cause: a weapon's hit with nobody named was a shot from out of sight (106 of 311
            // "decayed" frames in 23 games had been hit in their last seconds, and the report said of each that its
            // builder had been sent elsewhere, docs/studies/2026-10-17-unfinished-structures.md §2.2). An older
            // shim's event carries no cause, and the frame reads as abandoned, the commoner case (205 of the 311).
            let frame_loss = self.unfinished.remove(unit).map(|share| {
                let fate = if attacker.is_some() {
                    FrameFate::Killed
                } else if matches!(death, Death::Weapon(_) | Death::Debris) {
                    FrameFate::ShotUnseen
                } else {
                    FrameFate::Decayed { builder_dead: builder.is_some_and(|b| !tick.snapshot.own_units.iter().any(|u| u.id == b)) }
                };
                FrameLoss { def, pos, share, fate }
            });
            if let Some(loss) = frame_loss.as_ref().filter(|l| matches!(l.fate, FrameFate::Decayed { .. })) {
                // A nanoframe its builder walked away from decays: what was put in is lost, and nothing killed it
                // (pianist-player-1: two labs and fourteen generators "lost to something unseen at home", read as air).
                let share = loss.share;
                self.trade_log.push(Trade { frame: tick.frame, ours: true, metal: cost * share, kind: self.trade_kind(def) });
                let line = format!("abandoned an unfinished {} ({:.0}% built)", self.name(def), share * 100.0);
                if let Some(shared) = &self.strategist {
                    shared.fight(&line);
                }
                *self.fight_ledger.entry(line).or_default() += 1;
                let (name, grid) = (self.name(def).to_string(), self.world.grid(pos));
                self.event(tick.frame, format!("abandoned an unfinished {name} at {grid} ({:.0}% built)", share * 100.0));
                self.frame_losses_now.insert(*unit, frame_loss.expect("some"));
                continue;
            }
            let built = frame_loss.as_ref().map_or(String::new(), |l| format!(" ({:.0}% built)", l.share * 100.0));
            let cost = frame_loss.as_ref().map_or(cost, |l| cost * l.share);
            if let Some(loss) = frame_loss {
                self.frame_losses_now.insert(*unit, loss);
            }
            self.trade_log.push(Trade { frame: tick.frame, ours: true, metal: cost, kind: self.trade_kind(def) });
            self.publish_death(tick, kit, unit.0 as u32, true, self.name(def), cost, pos);
            self.unit_losses.push_back((tick.frame, *unit, def, attacker.and_then(|id| self.enemy_defs.get(&id).map(|d| (id, *d)))));
            if kit.is_extractor(def)
                && let Some(index) = self.world.hello.metal_spots.iter().position(|s| s.dist2d(pos) < self.spot_occupied_radius())
            {
                *self.spot_losses.entry(index).or_default() += 1;
            }
            let cause = self.death_cause(*unit, pos, *attacker, death, &self.recent_blasts, &own_defs_now, &dead_enemies_now);
            let place = if pos.dist2d(self.home) < BASE_RADIUS { "at home" } else if pos.dist2d(self.home) < pos.dist2d(self.world.mirrored(self.home)) { "in our half" } else { "in their half" };
            let line = format!("lost {}{built} to {cause} {place}", self.name(def));
            if matches!(cause, Cause::Missile | Cause::Blast { .. }) {
                strikes.push((cause.clone(), pos, def, cost));
            }
            if let Some(shared) = &self.strategist {
                shared.fight(&line);
            }
            *self.fight_ledger.entry(line).or_default() += 1;
            if self.world.def(def).is_some_and(|d| d.speed > 0.0 && d.build_speed == 0.0) {
                continue; // soldiers die all the time
            }
            let (name, grid) = (self.name(def).to_string(), self.world.grid(pos));
            self.event(tick.frame, format!("lost {name}{built} at {grid}"));
            if kit.is_extractor(def) {
                self.extractor_losses.push_back(tick.frame);
            }
        }
        // One line a strike for the events, and a missile wakes the player (H-REPORT-BLAST-CAUSE): what landed where
        // and what it took, since the losses line counts the dead one type at a time.
        for (missile, text) in self.strike_lines(&strikes) {
            let cooled = tick.frame - self.last_trigger_frame.get("missile").copied().unwrap_or(i32::MIN / 2) >= TRIGGER_COOLDOWN_FRAMES;
            if missile && cooled {
                self.trigger("missile", tick.frame, text);
            } else {
                self.event(tick.frame, text);
            }
        }
        while self.unit_losses.front().is_some_and(|(f, ..)| tick.frame - f > 3 * 60 * FRAMES_PER_SECOND) {
            self.unit_losses.pop_front();
        }
        while self.extractor_losses.front().is_some_and(|f| tick.frame - f > TRIGGER_COOLDOWN_FRAMES) {
            self.extractor_losses.pop_front();
        }
        if self.extractor_losses.len() >= LOSSES_WORTH_WAKING_FOR {
            let lost = self.extractor_losses.len();
            self.trigger("extractors", tick.frame, format!("We lost {lost} extractors in the last minute."));
        }
        for unit in &tick.snapshot.own_units {
            self.known_units.insert(unit.id, (unit.def, unit.pos));
        }
        // The frames still being built, each at the furthest it has been.
        let mut peak = std::mem::take(&mut self.unfinished);
        self.unfinished = tick
            .snapshot
            .own_units
            .iter()
            .filter(|u| u.being_built)
            .map(|u| {
                let share = u.build_progress;
                (u.id, peak.remove(&u.id).map_or(share, |p| p.max(share)))
            })
            .collect();
        self.frame_builders.retain(|id, _| self.unfinished.contains_key(id));
        // Since when each frame has stood with nobody of ours building it.
        let on = self.builders_on_frames();
        let (unfinished, alone) = (&self.unfinished, &mut self.frame_alone_since);
        alone.retain(|id, _| unfinished.contains_key(id));
        for id in unfinished.keys() {
            if on.contains_key(id) {
                alone.remove(id);
            } else {
                alone.entry(*id).or_insert(tick.frame);
            }
        }
    }

    /// The builders working on each unfinished frame of ours, by the hands' tasks: the frame's own builder, a
    /// builder finishing or repairing it, one guarding it, or one guarding its builder.
    pub(super) fn builders_on_frames(&self) -> HashMap<UnitId, Vec<String>> {
        use super::pianist::Task;
        let mut on: HashMap<UnitId, Vec<String>> = HashMap::new();
        let Some(pianist) = self.pianist.as_ref() else { return on };
        let frame_of = |builder: UnitId| match pianist.tasks.get(&builder) {
            Some(Task::Build { frame: Some(f), .. }) | Some(Task::Repair { target: f, .. }) => Some(*f),
            Some(Task::Assist { lab, .. }) if self.unfinished.contains_key(lab) => Some(*lab),
            _ => None,
        };
        for builder in pianist.tasks.keys() {
            let frame = frame_of(*builder).or_else(|| match pianist.tasks.get(builder) {
                Some(Task::Assist { lab, .. }) => frame_of(*lab),
                _ => None,
            });
            if let Some(frame) = frame {
                on.entry(frame).or_default().push(self.actor_name(*builder));
            }
        }
        for names in on.values_mut() {
            names.sort();
        }
        on
    }

    /// Every unfinished structure of ours for the report: those nobody is building first, the furthest along first
    /// among them.
    fn unfinished_frames(&self, own: &[OwnUnit], frame: i32) -> Vec<crate::strategist::shared::UnfinishedFrame> {
        let on = self.builders_on_frames();
        let mut frames: Vec<crate::strategist::shared::UnfinishedFrame> = own
            .iter()
            .filter(|u| u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0))
            .map(|u| crate::strategist::shared::UnfinishedFrame {
                handle: self.handle(u),
                unit: self.name(u.def).to_string(),
                at: self.place(u.pos),
                share: u.build_progress,
                cost: self.world.def(u.def).map_or(0.0, |d| d.metal_cost),
                builders: on.get(&u.id).cloned().unwrap_or_default(),
                alone_seconds: self.frame_alone_since.get(&u.id).map(|since| (frame - since) / FRAMES_PER_SECOND),
            })
            .collect();
        frames.sort_by(|a, b| b.builders.is_empty().cmp(&a.builders.is_empty()).then(b.share.total_cmp(&a.share)));
        frames
    }

    pub(super) fn track_enemy_buildings(&mut self, tick: &Tick) {
        /// How long a remembered building's place must be in sight with nothing there before it is gone: well past
        /// the grid's 30-frame refresh, so two reads of the grid have covered the place with nothing shown there.
        const GONE_DWELL_FRAMES: i32 = 2 * FRAMES_PER_SECOND;
        let mut gone = std::mem::take(&mut self.razed);
        let mut seen = Vec::new();
        let mut seen_unfinished: Vec<(UnitId, bool, i32)> = Vec::new();
        for event in &tick.events {
            if let Event::EnemyDestroyed { enemy, .. } = event {
                if let Some((def, pos, _)) = self.enemy_buildings.remove(enemy)
                    && self.world.is_factory_def(def)
                {
                    self.enemy_factories_gone.push((def, pos, tick.frame));
                }
                self.enemy_soldiers.remove(enemy);
                self.enemy_unfinished.remove(enemy);
                gone.push(*enemy);
            }
        }
        let mut firsts: Vec<String> = Vec::new();
        for enemy in &tick.snapshot.enemies {
            let Some(def) = enemy.def else { continue };
            let Some(info) = self.world.def(def) else { continue };
            if self.is_commander_def(def) {
                self.enemy_commander_seen = Some((enemy.pos, tick.frame));
            }
            // The first of each tier-2 or air type of his in sight is said at once (game 9: the Bull switch was
            // invisible, "factories_seen: none, ever" at 15:49; the first Liche of Cape Violet came unannounced).
            if !self.first_seen.contains(&def) {
                let tier = super::pianist::glossary::entry(&info.name).map_or(1, |e| e.tier);
                let air = self.world.domain_of(def) == crate::world::Domain::Air;
                if tier >= 2 || air {
                    self.first_seen.insert(def);
                    firsts.push(format!("first {} of his seen: {} at {}{}{}", if air && tier >= 2 { "tier-2 air unit" } else if air { "air unit" } else { "tier-2 unit" }, self.name(def), self.world.grid(enemy.pos), if enemy.being_built { " (being built)" } else { "" }, if info.build_speed > 0.0 { " (a builder: tier 2 is up)" } else { "" }));
                }
            }
            if info.speed == 0.0 {
                self.enemy_buildings.insert(enemy.id, (def, enemy.pos, tick.frame));
                if enemy.being_built {
                    self.enemy_unfinished.insert(enemy.id);
                } else {
                    self.enemy_unfinished.remove(&enemy.id);
                }
                seen_unfinished.push((enemy.id, enemy.being_built, tick.frame));
                seen.push((enemy.id, (def, enemy.pos, tick.frame)));
            } else if info.weapon_count > 0 && info.build_speed == 0.0 {
                self.enemy_soldiers.insert(enemy.id, (def, enemy.pos, tick.frame));
            }
        }
        // A remembered building whose place our side sees, with nothing of his there, is gone (the impedance
        // tally, 2026-10-16: nuke-1-medium's report listed his labs as standing "in sight now" for twelve minutes
        // after our missiles killed them out of sight, and two more missiles went at them). A radar contact at the
        // place is still in the snapshot by its id, so it keeps the memory. Sight is the engine's grid when the
        // shim sends it (cut by terrain: a hill between them is no sight), else a fifth short of a unit's circle.
        // A type that cloaks is never declared gone by sight alone: it is hidden until something of ours stands
        // within its decloak distance (H-ENEMY-GONE-NEEDS-SIGHT, docs/design/2026-10-18-sight-jammers-cloak-shields.md).
        let in_snapshot: std::collections::HashSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        let standing: Vec<(Vec3, f32)> = tick.snapshot.own_units.iter().filter(|u| !u.being_built).map(|u| (u.pos, self.sight_of(u.def) * 0.8)).collect();
        // Three guards, one for each way "in sight with nothing there" has lied (sight-regress-1 and
        // apex-v032-altored-3v3-1, read against the truth files): the engine's grid, because a hill between our
        // unit and the place is no sight whatever the distance; this seat's own units' sight circles (a fifth
        // short), because the grid is up to 15 frames old on every other think and the place may have left our
        // sight since; and the dwell below, so that no single look decides.
        // Without the grid (a shim that sends none) the circle stands alone, as before.
        let (grid, grid_frame) = match self.sensors_fresh(tick.frame) {
            Some(s) => (Some(s.los.clone()), self.sensors.as_ref().map_or(tick.frame, |(_, at)| *at)),
            None => (None, tick.frame),
        };
        let in_circle = |pos: Vec3| standing.iter().any(|(at, sight)| at.dist2d(pos) <= *sight);
        let sees = |pos: Vec3| in_circle(pos) && grid.as_ref().is_none_or(|los| los.at(pos.x, pos.z));
        let decloaked = |def: UnitDefId, pos: Vec3| match self.world.def(def).and_then(|d| d.cloak) {
            Some(cloak) => standing.iter().any(|(at, _)| at.dist2d(pos) <= cloak.decloak_distance),
            None => true,
        };
        // The absence must last (GONE_DWELL_FRAMES): the grid is read every 30 frames and the brain thinks every 15,
        // so half the thinks hold a half-second-old grid, while the engine's enemy list is the current frame's (its
        // visibility flags are refreshed every frame, one frame behind its LOS map). A building just uncovered is
        // \"in sight with nothing there\" for up to half a second: in sight-regress-1 the rule fired 107 times, 96 of
        // them on the stale-grid thinks, and the report listed 2 of the 12 buildings in sight at his base (the memory
        // held 61% of the truth's buildings in cells seen within the minute; 83-86% with the dwell).
        let mut razed: Vec<UnitId> = Vec::new();
        for (id, (def, pos, _)) in &self.enemy_buildings {
            if in_snapshot.contains(id) || !sees(*pos) || !decloaked(*def, *pos) {
                self.missing_in_sight.remove(id);
                continue;
            }
            let since = *self.missing_in_sight.entry(*id).or_insert(tick.frame);
            if tick.frame - since >= GONE_DWELL_FRAMES {
                razed.push(*id);
            }
        }
        self.missing_in_sight.retain(|id, _| self.enemy_buildings.contains_key(id));
        for id in razed {
            let since = self.missing_in_sight.remove(&id).unwrap_or(tick.frame);
            if let Some((def, pos, _)) = self.enemy_buildings.remove(&id) {
                if self.world.is_factory_def(def) {
                    self.enemy_factories_gone.push((def, pos, tick.frame));
                }
                // The evidence in the event, for the next review against the truth file.
                let nearest = standing.iter().map(|(at, _)| at.dist2d(pos)).fold(f32::INFINITY, f32::min);
                self.event(
                    tick.frame,
                    format!(
                        "his {} at {} is gone: the place has been in sight {} s with nothing standing there (ours {:.0} away, the grid {} frames old)",
                        self.name(def),
                        self.world.grid(pos),
                        (tick.frame - since) / FRAMES_PER_SECOND,
                        nearest,
                        if grid.is_some() { tick.frame - grid_frame } else { -1 }
                    ),
                );
            }
            self.enemy_unfinished.remove(&id);
            gone.push(id);
        }
        for text in firsts {
            self.trigger("first_seen", tick.frame, text);
        }
        // H-TEAM-BOARD: what one seat of ours has seen, all know.
        if self.enabled("H-TEAM-BOARD") {
            self.enemy_buildings = self.board.pool_buildings(&seen, &gone);
            self.enemy_unfinished = self.board.pool_unfinished(&seen_unfinished, &gone);
        }
    }

    fn group(&self, units: &[&OwnUnit]) -> Group {
        let mut composition: BTreeMap<&str, usize> = BTreeMap::new();
        for unit in units {
            *composition.entry(self.name(unit.def)).or_default() += 1;
        }
        let n = units.len().max(1) as f32;
        let centre = units.iter().fold(Vec3::default(), |sum, u| Vec3 { x: sum.x + u.pos.x / n, y: 0.0, z: sum.z + u.pos.z / n });
        Group {
            size: units.len(),
            idle: units.iter().filter(|u| u.idle).count(),
            centre: (!units.is_empty()).then(|| self.place(centre)),
            composition: composition.into_iter().map(|(name, count)| (name.to_string(), count)).collect(),
        }
    }

    pub(super) fn publish_briefing(&mut self, tick: &Tick, kit: &Kit) {
        let Some(shared) = self.strategist.clone() else { return };
        let snapshot = &tick.snapshot;
        let count = |def: UnitDefId| snapshot.own_units.iter().filter(|u| u.def == def && !u.being_built).count();
        let soldiers: Vec<&OwnUnit> = snapshot.own_units.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
        if snapshot.own_units.iter().any(|u| !u.being_built && self.his_half(u.pos)) {
            self.ours_in_his_half = Some(tick.frame);
        }

        // A frame is said as one ("armguard 1 (being built)"): player-16's 13:19 packet read a Gauntlet at 8% health
        // as "the F2 fortress" and banned F2 to the end; it was gone at 13:54 and the first standing one came 16:52.
        let mut cells: BTreeMap<String, (Vec3, BTreeMap<String, usize>, usize, Vec<UnitId>)> = BTreeMap::new();
        for enemy in &snapshot.enemies {
            let cell = cells.entry(self.world.grid(enemy.pos)).or_insert((enemy.pos, BTreeMap::new(), 0, Vec::new()));
            let name = enemy.def.map_or("unidentified", |def| self.name(def));
            *cell.1.entry(if enemy.being_built { format!("{name} (being built)") } else { name.to_string() }).or_default() += 1;
            cell.2 += 1;
            cell.3.push(enemy.id);
        }
        let enemies_visible = cells
            .into_values()
            .map(|(pos, composition, units, ids)| EnemyCluster {
                at: self.place(pos),
                units,
                composition: composition.into_iter().collect(),
                distance_from_home: pos.dist2d(self.home) as i32,
                harming: self.harm_words(&ids),
            })
            .collect();
        let mut enemy_buildings_remembered: Vec<RememberedBuilding> = self
            .enemy_buildings
            .values()
            .map(|(def, pos, seen)| RememberedBuilding { name: self.name(*def).to_string(), at: self.place(*pos), last_seen: clock(*seen) })
            .collect();
        enemy_buildings_remembered.sort_by(|a, b| a.at.grid.cmp(&b.at.grid).then(a.name.cmp(&b.name)));

        let sides = self.sides();
        let standing: Vec<&bot_protocol::OwnUnit> = snapshot.own_units.iter().filter(|u| !u.being_built).collect();
        let allies = self.ally_seats(tick);
        let (game_end, side_commanders) = self.game_end(&standing);
        let briefing = Briefing {
            sides,
            allies,
            jammers: self.jammer_lines(tick.frame),
            guns: self.gun_lines(tick.frame),
            shields: self.shield_lines(tick),
            sensors_known: self.sensors_fresh(tick.frame).is_some(),
            allied_teams: self.allied_teams(),
            game_end,
            side_commanders,
            side_spots: self.side_spots(),
            seats: Vec::new(),
            build_power: self.build_power(&snapshot.own_units),
            spending: self.spending(&snapshot.own_units),
            game_time: clock(tick.frame),
            frame: tick.frame,
            metal: snapshot.metal,
            energy: snapshot.energy,
            wind: snapshot.wind,
            wind_range: (self.world.hello.map.wind_min, self.world.hello.map.wind_max),
            // By what a definition is, not by the kit's tier-1 land names: on SailAway 2 the line read "constructors 0
            // labs 0" all game beside twenty hover constructors and two hover platforms. Standing units only: the
            // score line's "extractors 2" stood beside the eco line's "extractors 3" while a frame was up (player-12
            // 1:45), and "labs 1" was said at 1:00 of a plant standing at 1:12.
            counts: Counts {
                extractors: standing.iter().filter(|u| self.world.is_extractor_def(u.def)).count(),
                generators: standing.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && (d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.wind_cap > 0.0 || d.tidal_make > 0.0))).count(),
                converters: count(kit.converter),
                labs: standing.iter().filter(|u| self.world.is_factory_def(u.def)).count(),
                factories_begun: snapshot.own_units.iter().filter(|u| self.world.is_factory_def(u.def)).count(),
                construction_turrets_begun: snapshot.own_units.iter().filter(|u| self.is_static_builder(u.def)).count(),
                turrets: standing.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && d.weapon_count > 0)).count(),
                constructors: standing.iter().filter(|u| self.world.is_constructor_def(u.def)).count(),
                army: soldiers.len(),
            },
            stockpilers: standing
                .iter()
                .filter_map(|u| {
                    let d = self.world.def(u.def).filter(|d| d.stockpile_seconds > 0.0)?;
                    Some(crate::strategist::shared::Stockpiler { handle: self.handle(u), anti_nuke: d.coverage > 0.0, ready: u.stockpile, queued: u.stockpile_queued, seconds: d.stockpile_seconds, coverage: d.coverage })
                })
                .collect(),
            frames: self.unfinished_frames(&snapshot.own_units, tick.frame),
            home: self.place(self.home),
            colour: self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).and_then(|t| t.color).map(colour_name).unwrap_or_default(),
            home_group: self.group(&soldiers),
            enemies_visible,
            enemy_buildings_remembered,
            recent_events: self.recent_events.iter().cloned().collect(),
        };
        // Under a commander the side is published what the player is (`strategist/command.rs`).
        if let Some(side) = shared.side.get() {
            side.publish_briefing(self.world.hello.team, self.home, briefing.clone());
        }
        shared.publish_briefing(self.world.hello.team, self.home, briefing);
    }

    /// What of ours gets to a spot our soldiers cannot walk to (H-REPORT-FREE-SPOTS-BY-REACH): "under water: our corcom
    /// can take it now; a cormuskrat (from corvp) or a corch (from corhp) would". The mobile builders of our roster
    /// whose movement class reaches the place; with `own`, those standing now apart from those a plant must make.
    /// (Cape Violet 2026-10-18: thirteen spots of our half under water, the commander 1,176 from the nearest, and no
    /// report named one or said who could take it.)
    pub(super) fn reach_words(&self, spot: Vec3, own: Option<&[OwnUnit]>) -> String {
        let where_ = if self.world.under_water(spot) { "under water" } else { "off the ground our bots walk (an islet, a cliff top)" };
        let roster: Vec<UnitDefId> = self.kit.map(|k| self.world.reachable_from(k.commander)).unwrap_or_default();
        let (mut now, mut later, mut any): (Vec<String>, Vec<String>, Vec<String>) = (Vec::new(), Vec::new(), Vec::new());
        for d in self.world.hello.unit_defs.iter().filter(|d| d.speed > 0.0 && d.build_speed > 0.0 && roster.contains(&d.id) && self.reachable_for(self.walker_of(d.id), spot)) {
            let makers: Vec<&str> = self.world.hello.unit_defs.iter().filter(|f| f.speed == 0.0 && f.build_options.contains(&d.id) && roster.contains(&f.id)).map(|f| f.name.as_str()).collect();
            let made = if makers.is_empty() { String::new() } else { format!(" (from {})", makers.join(" or ")) };
            match own.map(|own| own.iter().filter(|u| u.def == d.id && !u.being_built).count()) {
                Some(0) => later.push(format!("a {}{made}", d.name)),
                Some(1) => now.push(format!("our {}", d.name)),
                Some(n) => now.push(format!("our {n} {}s", d.name)),
                None => any.push(if d.id == self.kit.map(|k| k.commander).unwrap_or(d.id) { format!("our {}", d.name) } else { format!("a {}{made}", d.name) }),
            }
        }
        if own.is_none() {
            return if any.is_empty() { format!("{where_}: nothing of ours reaches it") } else { format!("{where_}: reached by {}", any.join(", ")) };
        }
        match (now.is_empty(), later.is_empty()) {
            (true, true) => format!("{where_}: nothing of ours reaches it"),
            (false, true) => format!("{where_}: {} can take it now", now.join(" or ")),
            (true, false) => format!("{where_}: {} would take it; nothing of ours standing reaches it", later.join(" or ")),
            (false, false) => format!("{where_}: {} can take it now; {} would", now.join(" or "), later.join(" or ")),
        }
    }

    /// The map's spots by what of ours reaches them: "72 spots: 46 our bots walk to, 26 under water (reached by our
    /// corcom, a cormuskrat (from corvp), ...), 0 on ground they cannot reach, 0 nothing of ours reaches".
    fn spots_summary(&self) -> String {
        let spots = &self.world.hello.metal_spots;
        let on_foot = spots.iter().filter(|s| self.reachable_on_foot(**s)).count();
        let water: Vec<Vec3> = spots.iter().filter(|s| !self.reachable_on_foot(**s) && self.world.under_water(**s) && self.reachable_by_any_class(**s)).copied().collect();
        let ground: Vec<Vec3> = spots.iter().filter(|s| !self.reachable_on_foot(**s) && !self.world.under_water(**s) && self.reachable_by_any_class(**s)).copied().collect();
        let none = spots.len() - on_foot - water.len() - ground.len();
        let reach = |of: &[Vec3]| of.first().map_or(String::new(), |s| format!(" ({})", self.reach_words(*s, None).replace("under water: ", "").replace("off the ground our bots walk (an islet, a cliff top): ", "")));
        format!("{} spots: {on_foot} our bots walk to, {} under water{}, {} on ground they cannot reach{}, {none} nothing of ours reaches", spots.len(), water.len(), reach(&water), ground.len(), reach(&ground))
    }

    /// The sea: how much of the map is under water and what of ours can cross it (the user, on pianist-player-7: the
    /// map is an island, the enemy commander was out in the ocean building, and the player had never been told).
    fn water_description(&self) -> serde_json::Value {
        let terrain = &self.world.hello.terrain;
        let under = terrain.heights.iter().filter(|h| **h < 0).count();
        let share = if terrain.heights.is_empty() { 0.0 } else { under as f32 / terrain.heights.len() as f32 * 100.0 };
        let lab_builds: Vec<UnitDefId> = self.kit.and_then(|k| self.world.def(k.lab)).map(|d| d.build_options.clone()).unwrap_or_default();
        // By the commander's roster, not the lobby side's prefix: a seat on side Random ("ran") listed none (Cape
        // Violet, the water note's `amphibious_of_ours: []`).
        let roster: Vec<UnitDefId> = self.kit.map(|k| self.world.reachable_from(k.commander)).unwrap_or_default();
        let crosses = |d: &bot_protocol::UnitDefInfo| d.move_class.is_some_and(|m| matches!(m.kind, bot_protocol::MoveKind::Hover) || m.depth >= 1000.0) && d.speed > 0.0;
        let ours: Vec<String> = self
            .world
            .hello
            .unit_defs
            .iter()
            .filter(|d| crosses(d) && roster.contains(&d.id) && (d.weapon_count > 0 || d.build_speed > 0.0))
            .map(|d| format!("{} ({}{})", d.name, match (d.weapon_count > 0, d.build_speed > 0.0) { (true, true) => "armed, a builder", (true, false) => "armed", _ => "a builder" }, if lab_builds.contains(&d.id) { ", from the bot lab" } else { "" }))
            .collect();
        json!({
            "share": format!("{share:.0}% of the map is under water (the sketch's ~)"),
            "note": "Our bots and vehicles stop at the shore, except through fords their class wades (tanks to a depth of about 20, bots likewise: a ford is a crossing, deep water is not); a spot or a place in deep water is reached only by what is listed below, and an advance of tanks toward it stalls at the shore. Our commander is amphibious: it walks on the sea floor and builds an extractor on a spot under water from the first minute (the same extractor type; there is no special one), as do the amphibious, hover and ship constructors below once a plant makes them; the under-water spots are a share of the map's metal that is his alone unless some of these take them (Cape Violet 2026-10-18: 26 of 72 spots, none taken, the person's fleet and Epoch from them). The enemy's commander is amphibious too and can retreat into the sea when its base is gone and build on the shore from the water; its construction ships take the under-water spots and its ships shell the shore from beyond our reach.",
            "amphibious_of_ours": ours,
        })
    }

    /// At Hello, before any tick (`docs/design/2026-09-30-opening-turn.md`): this seat's line for the opening
    /// turn's prompt, and the map as it can be known before our start is.
    pub fn before_the_game(&self) {
        let Some(shared) = &self.strategist else { return };
        let hello = &self.world.hello;
        let me = hello.teams.iter().find(|t| t.team == hello.team);
        let faction = match self.faction() {
            f if f.is_empty() || f.eq_ignore_ascii_case("random") => "its faction chosen by the game at the start".to_string(),
            f => f,
        };
        let start = match (hello.start_pos_type, me.and_then(|t| t.start_pos)) {
            (Some(0) | Some(3), Some(at)) => format!("its start is fixed by the lobby at {} ({:.0}, {:.0})", self.world.grid(at), at.x, at.z),
            _ => match hello.start_boxes.iter().find(|b| b.ally_team == hello.ally_team) {
                Some(b) => format!("its start will be somewhere inside our start box (cells {}), placed by the game or by a teammate", self.world.box_cells(b)),
                None => "its start is not known".to_string(),
            },
        };
        // A player with this seat alone may call its one commander `commander`, whatever the seat's tag.
        if self.seats_of_the_player() == 1 {
            *shared.own_commander.lock().unwrap() = Some(self.commander_handle());
        }
        let line = format!("{} (team {}): {faction}; {start}", self.commander_handle(), hello.team);
        shared.seat_said_hello(hello.team, self.seats_of_the_player(), line, self.map_before_the_game());
    }

    /// The `map` tool's answer before the game: what does not depend on where we start.
    fn map_before_the_game(&self) -> serde_json::Value {
        let hello = &self.world.hello;
        let spots: Vec<_> = hello.metal_spots.iter().enumerate().map(|(n, s)| json!({ "n": n, "grid": self.world.grid(*s), "x": s.x as i32, "z": s.z as i32 })).collect();
        json!({
            "people": self.people_description(),
            "name": hello.map.name, "width": hello.map.width, "height": hello.map.height,
            "grid": "8x8 cells; columns A-H run west to east (x), rows 1-8 run north to south (z)",
            "our_start": "not known before the game begins",
            "start_boxes": hello.start_boxes.iter().map(|b| json!({
                "ally_team": b.ally_team, "ours": b.ally_team == hello.ally_team, "cells": self.world.box_cells(b),
                "left": b.left as i32, "top": b.top as i32, "right": b.right as i32, "bottom": b.bottom as i32,
            })).collect::<Vec<_>>(),
            "metal_spots": spots,
            "note": "spot_N is metal spot n. The walking distances from our start, the terrain sketch, the water and the passages come with the first report of the running game, when our start is known.",
        })
    }

    /// His start as the lobby gave it and nothing guessed by us (the user, 2026-10-17: where he is, the harness has no
    /// authority to say; until then the sheet priced the bot's own base guess as "where his start may be"): the
    /// script's position for each enemy team when the lobby fixed the starts, else the centre of each enemy team's
    /// box while the boxes are honoured (his commander was placed somewhere inside it at 0:00), else nothing.
    fn his_starts_as_given(&self) -> Vec<(String, Vec3)> {
        let hello = &self.world.hello;
        if matches!(hello.start_pos_type, Some(0) | Some(3)) {
            return hello
                .teams
                .iter()
                .filter(|t| t.ally_team != hello.ally_team && t.controller != bot_protocol::Controller::Gaia)
                .filter_map(|t| t.start_pos.map(|at| (format!("team {}'s start by the script", t.team), at)))
                .collect();
        }
        if !self.boxes_honoured {
            return Vec::new();
        }
        hello
            .start_boxes
            .iter()
            .filter(|b| b.ally_team != hello.ally_team)
            .map(|b| (format!("the centre of his lobby box {} (his commander was placed somewhere inside it at 0:00)", self.world.box_cells(b)), b.centre()))
            .collect()
    }

    /// The map sheet (`terrain::sheet`) from our start to his start as the lobby gave it.
    fn map_sheet(&self) -> serde_json::Value {
        let hello = &self.world.hello;
        let rate = hello.unit_defs.iter().map(|d| d.extracts_metal).filter(|r| *r > 0.0).min_by(f32::total_cmp).unwrap_or(0.0);
        let mut starts = vec![("ours".to_string(), self.home)];
        starts.extend(self.his_starts_as_given());
        terrain::sheet::sheet(&terrain::sheet::Input {
            name: &hello.map.name,
            terrain: &hello.terrain,
            size: (hello.map.width, hello.map.height),
            grid: (8.0, 8.0),
            wind: (hello.map.wind_min, hello.map.wind_max),
            tidal: hello.map.tidal,
            spots: hello.metal_spots.iter().map(|s| (*s, Some(s.y * rate))).collect(),
            movers: hello.unit_defs.iter().filter_map(|d| Some(terrain::sheet::Mover { name: d.name.clone(), speed: d.speed, class: d.move_class? })).collect(),
            starts,
        })
    }

    /// The `map` tool's answer and the first report's map, for the player's session and the side's: built when our
    /// start is known (`adopt_faction`, after the survey) and again when his start's guess moves (`resurvey_enemy`),
    /// the only times its inputs change. It is costly: the sheet prices a route field per movement class per start,
    /// 28 fields in a 1v1 (350 ms), 126 in an 8v8; built on every think tick for the first ten seconds, it held the
    /// 8v8 seats' ticks at 4 to 5 s while the opening ran (K-bot-the-first-ten-seconds-rebuild-the-map-sheet-every-tick).
    pub(super) fn publish_map(&self) {
        let Some(shared) = &self.strategist else { return };
        let map = self.map_description();
        if let Some(side) = shared.side.get() {
            *side.map.lock().unwrap() = map.clone();
        }
        *shared.map.lock().unwrap() = map;
    }

    fn map_description(&self) -> serde_json::Value {
        let map = &self.world.hello.map;
        let spots: Vec<_> = self
            .world
            .hello
            .metal_spots
            .iter()
            .enumerate()
            .map(|(n, s)| {
                let walk = self.reachable_on_foot(*s).then(|| self.walk_from_home(*s) as i32);
                let straight = s.dist2d(self.home) as i32;
                let mut entry = json!({ "n": n, "grid": self.world.grid(*s), "x": s.x as i32, "z": s.z as i32, "walk_from_home": walk, "straight_from_home": straight });
                // A spot our soldiers cannot walk to says what of ours gets there (H-REPORT-FREE-SPOTS-BY-REACH).
                if walk.is_none() {
                    entry["reached_by"] = json!(if self.reachable_by_any_class(*s) { self.reach_words(*s, None) } else { "nothing of ours reaches it".to_string() });
                }
                if let Some(w) = walk && straight > 300 && w as f32 > 1.8 * straight as f32 {
                    entry["alcove"] = json!(format!("an alcove or a pocket: {w} on foot against {straight} straight; it opens from the far side, so a group sent here walks round"));
                }
                entry
            })
            .collect();
        json!({
            "name": map.name, "width": map.width, "height": map.height,
            "grid": "8x8 cells; columns A-H run west to east (x), rows 1-8 run north to south (z)",
            "our_start": self.place(self.home),
            "start_boxes": self.world.hello.start_boxes.iter().map(|b| json!({
                "ally_team": b.ally_team, "ours": b.ally_team == self.world.hello.ally_team, "cells": self.world.box_cells(b),
                "left": b.left as i32, "top": b.top as i32, "right": b.right as i32, "bottom": b.bottom as i32,
            })).collect::<Vec<_>>(),
            "start_boxes_note": match self.world.hello.start_pos_type {
                Some(0) | Some(3) => "the lobby fixed every seat's start (the script's startpostype 0 or 3): the enemy's start below is exact, from the script, and the boxes are the map's defaults, meaning nothing",
                _ if self.boxes_honoured => "the lobby's start boxes: each team's commander was placed somewhere inside its box at 0:00. The engine tells nobody where; where the opponent stands now is known only from what our units see",
                _ => "the lobby's start boxes were NOT honoured: our own start lies in another team's box or outside ours, so the host placed the seats by hand and the boxes say nothing of where he is",
            },
            "metal_spots": spots,
            "metal_spots_summary": self.spots_summary(),
            "metal_spots_note": "n is the spot's number for the `expansion` tool; walk_from_home is the walking distance for our bots (null: they cannot walk there, and `reached_by` says what of ours can: our commander from the first minute, and the amphibious, hover and ship constructors once a plant makes them) beside straight_from_home; a spot marked `alcove` is far longer on foot than straight, and a group sent at it by the straight line huddles short of it",
            "terrain": self.terrain_sketch(),
            "people": self.people_description(),
            "sheet": self.map_sheet(),
            "sheet_note": "the map in the numbers the decisions turn on, the same table for every map (the brief holds other maps' sheets to set this one against): what each way of moving can stand on, the walk and the seconds to his start as the lobby gave it (the script's position, or the centre of his box), whose spots are whose by walk, how wide the ways there are at their tightest against their median (a pass is a fraction of the median), and how many spots each kind of builder reaches in 30, 60, 90 and 120 seconds",
            "water": self.water_description(),
            "passages": self.passages().iter().map(|p| json!({ "at": self.place(p.at), "width": p.width as i32, "share_of_the_way_from_our_start": (p.along * 100.0) as i32 })).collect::<Vec<_>>(),
            "passages_note": "narrow places every walking route between our start and the opponent's goes through (cliffs or water on both sides), the narrowest first",
        })
    }
}

impl Brain {
    /// This seat's faction by name: from the commander's type when the script's side is Random (the engine keeps
    /// the word; the game picks the faction at start), else the script's side capitalised.
    pub(super) fn faction(&self) -> String {
        let side = &self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).map(|t| t.side.as_str()).unwrap_or("");
        match self.kit.and_then(|k| self.world.def(k.commander)).map(|d| d.name.as_str()) {
            Some(n) if side.eq_ignore_ascii_case("random") || side.is_empty() => match &n[..n.len().min(3)] {
                "arm" => "Armada".to_string(),
                "cor" => "Cortex".to_string(),
                "leg" => "Legion".to_string(),
                _ => n.to_string(),
            },
            _ => {
                let mut c = side.chars();
                c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
            }
        }
    }

    /// The faction a team has been seen to field: its commander unit's when that was seen, else the one most of
    /// its units seen belong to (a resurrected unit of ours fights for him under our faction's name).
    fn seen_faction(&self, team: i32) -> Option<String> {
        let (counts, commander) = self.factions_seen.get(&team)?;
        let prefix = commander.or_else(|| counts.iter().max_by_key(|(_, ids)| ids.len()).map(|(p, _)| *p))?;
        Some(match prefix { "arm" => "Armada", "cor" => "Cortex", _ => "Legion" }.to_string())
    }

    /// Every ally team of the game with its seats in words, ours first (from the start script's controllers).
    /// Who plays a seat, as the sides and allies lines word it.
    fn who_plays(&self, t: &bot_protocol::TeamInfo, sibling: &str) -> String {
        use bot_protocol::Controller;
        match &t.controller {
            _ if t.team == self.world.hello.team => "you (WReason)".to_string(),
            Controller::Person { name, skill: Some(skill) } => format!("{name} (a person, lobby skill {skill:.0})"),
            Controller::Person { name, skill: None } => format!("{name} (a person)"),
            Controller::Ai { short_name, profile: Some(profile), .. } if short_name.eq_ignore_ascii_case("wreason") => format!("Within Reason {profile} ({sibling})"),
            Controller::Ai { short_name, .. } if short_name.eq_ignore_ascii_case("wreason") => format!("Within Reason ({sibling})"),
            Controller::Ai { short_name, profile: Some(profile), .. } => format!("{short_name} {profile} (an AI)"),
            Controller::Ai { short_name, .. } => format!("{short_name} (an AI)"),
            Controller::Gaia => "Gaia".to_string(),
            Controller::Unknown => format!("team {} (who plays it is not in the script)", t.team),
        }
    }

    /// The teams of our side that are seats of neither this bot nor Gaia: `transfer` may send them metal and energy.
    pub(super) fn allied_teams(&self) -> Vec<i32> {
        use bot_protocol::Controller;
        let hello = &self.world.hello;
        hello
            .teams
            .iter()
            .filter(|t| t.ally_team == hello.ally_team && t.team != hello.team && t.controller != Controller::Gaia)
            .filter(|t| !matches!(&t.controller, Controller::Ai { short_name, .. } if short_name.eq_ignore_ascii_case("wreason")))
            .map(|t| t.team)
            .collect()
    }

    /// The other seats of our side, each summed from the allied units in sight this tick (allies share sight, so
    /// every one of them is): what it has built, by class, and where its soldiers stand (H-REPORT-ALLIES-LINE).
    pub(super) fn ally_seats(&mut self, tick: &Tick) -> Vec<crate::strategist::shared::AllySeat> {
        use crate::strategist::shared::AllySeat;
        let hello = &self.world.hello;
        let sibling = sibling_words();
        let mut by_team: BTreeMap<i32, Vec<&bot_protocol::AllyUnit>> = BTreeMap::new();
        for a in &self.allies {
            by_team.entry(a.team).or_default().push(a);
        }
        let spot_radius = self.spot_occupied_radius();
        let mut seats = Vec::new();
        for (team, units) in by_team {
            let info = hello.teams.iter().find(|t| t.team == team);
            let mut seat = AllySeat {
                team,
                who: info.map(|t| self.who_plays(t, sibling)).unwrap_or_else(|| format!("team {team}")),
                colour: info.and_then(|t| t.color).map(colour_name).unwrap_or_default(),
                ..AllySeat::default()
            };
            let multiplier = info.and_then(|t| t.income_multiplier).unwrap_or(1.0);
            let mut worth = 2.0;
            let mut tier2 = false;
            let mut army_pos: Vec<Vec3> = Vec::new();
            for u in units.iter().filter(|u| !u.being_built) {
                let Some(d) = self.world.def(u.def) else { continue };
                if seat.faction.is_empty() {
                    let mut f = crate::world::World::faction_of(&d.name).chars();
                    seat.faction = f.next().map(|c| c.to_ascii_uppercase().to_string() + f.as_str()).unwrap_or_default();
                }
                if d.extracts_metal > 0.0 {
                    // A moho draws four times an extractor's share of its spot (K-rules-mex-upgrade-in-place).
                    let moho = d.extracts_metal >= 0.002;
                    if moho { seat.mohos += 1 } else { seat.extractors += 1 }
                    if let Some(spot) = hello.metal_spots.iter().find(|s| s.dist2d(u.pos) < spot_radius) {
                        worth += spot.y * if moho { 4.0 } else { 1.0 };
                    }
                } else if d.speed == 0.0 && !d.build_options.is_empty() {
                    seat.factories += 1;
                    // Tier-1 labs and plants build at 150, the advanced ones at 600, the gantries at 1,800.
                    tier2 |= d.build_speed >= 400.0;
                } else if d.speed > 0.0 && d.build_speed > 0.0 && !d.build_options.is_empty() && !self.world.is_commander_def(u.def) {
                    // Advanced constructors cost 330 to 580 metal, the tier-1 ones 100 to 145.
                    if d.metal_cost >= 250.0 { seat.advanced_constructors += 1 } else { seat.constructors += 1 }
                } else if d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty() {
                    seat.nanos += 1;
                } else if d.converter.is_some() {
                    seat.converters += 1;
                } else if d.speed == 0.0 && d.energy_make >= 80.0 {
                    seat.big_energy += 1;
                } else if d.weapon_count > 0 && d.speed > 0.0 && d.build_speed == 0.0 {
                    seat.soldiers += 1;
                    seat.army_metal += d.metal_cost as u32;
                    army_pos.push(u.pos);
                }
            }
            if tier2 {
                seat.tier2_since = Some(*self.ally_tier2_since.entry(team).or_insert(tick.frame));
            }
            if !army_pos.is_empty() {
                let n = army_pos.len() as f32;
                let centre = Vec3 { x: army_pos.iter().map(|p| p.x).sum::<f32>() / n, y: 0.0, z: army_pos.iter().map(|p| p.z).sum::<f32>() / n };
                seat.army_at = self.world.grid(centre);
            }
            seat.income_estimate = worth * multiplier;
            seats.push(seat);
        }
        seats
    }

    /// The lobby's game-end rule in words, with our side's commanders standing against the seats it started with
    /// (H-REPORT-GAME-END; K-rules-commander-death-ends-game: under `com` a side loses only when none is left).
    pub(super) fn game_end(&self, standing: &[&OwnUnit]) -> (String, (usize, usize)) {
        use bot_protocol::Controller;
        let hello = &self.world.hello;
        let ours = standing.iter().filter(|u| self.world.is_commander_def(u.def)).count();
        let theirs = self.allies.iter().filter(|a| !a.being_built && self.world.is_commander_def(a.def)).count();
        let seats = hello.teams.iter().filter(|t| t.ally_team == hello.ally_team && t.controller != Controller::Gaia).count();
        let commanders = (ours + theirs, seats);
        let words = match super::people::mod_option(&hello.script, "deathmode").as_deref() {
            Some("com") if seats <= 1 => "the game ends when a commander dies, his or ours (the lobby's rule: a side loses when it has no commanders left)".to_string(),
            Some("com") => format!("a side loses when it has no commanders left (the lobby's rule), so a seat whose commander dies plays on while another of ours stands; our side has {} commanders standing of the {} its seats started with", commanders.0, commanders.1),
            Some("own_com") => "a seat loses when its own commander dies (the lobby's rule)".to_string(),
            Some("builders") => "a side loses when it has no builders left (the lobby's rule)".to_string(),
            Some("neverend") => "nobody is eliminated (the lobby's rule)".to_string(),
            Some("territorial_domination") => "territorial domination: points for territory held decide the game (the lobby's rule)".to_string(),
            Some(other) => format!("the lobby's game-end rule is `{other}`"),
            None => String::new(),
        };
        (words, commanders)
    }

    /// Spots nearer our side's start box than his, over the map's total and the seats of our side sharing them
    /// (H-REPORT-SIDE-SPOTS; human-8v8-1: 43 of 84 in the south half for five seats, all taken by 12:00, and the
    /// players read "stuck at 9 extractors" as a fault of theirs).
    pub(super) fn side_spots(&self) -> Option<(usize, usize, usize)> {
        use bot_protocol::Controller;
        let hello = &self.world.hello;
        let (ours, theirs): (Vec<_>, Vec<_>) = hello.start_boxes.iter().partition(|b| b.ally_team == hello.ally_team);
        if ours.is_empty() || theirs.is_empty() {
            return None;
        }
        let dist = |b: &bot_protocol::StartBox, p: &Vec3| {
            let dx = (b.left - p.x).max(0.0).max(p.x - b.right);
            let dz = (b.top - p.z).max(0.0).max(p.z - b.bottom);
            (dx * dx + dz * dz).sqrt()
        };
        let nearest = |boxes: &[&bot_protocol::StartBox], p: &Vec3| boxes.iter().map(|b| dist(b, p)).fold(f32::MAX, f32::min);
        let near = hello.metal_spots.iter().filter(|s| nearest(&ours, s) < nearest(&theirs, s)).count();
        let seats = hello.teams.iter().filter(|t| t.ally_team == hello.ally_team && t.controller != Controller::Gaia).count();
        Some((near, hello.metal_spots.len(), seats))
    }

    pub(super) fn sides(&self) -> Vec<crate::strategist::shared::Side> {
        use bot_protocol::Controller;
        let hello = &self.world.hello;
        let mut ally_teams: Vec<i32> = hello.teams.iter().filter(|t| t.controller != Controller::Gaia).map(|t| t.ally_team).collect();
        ally_teams.sort_unstable();
        ally_teams.dedup();
        ally_teams.sort_by_key(|a| *a != hello.ally_team);
        ally_teams
            .into_iter()
            .map(|ally_team| {
                // With a player a seat the other seats of ours are allies with players of their own.
                let sibling = sibling_words();
                let seats = hello
                    .teams
                    .iter()
                    .filter(|t| t.ally_team == ally_team && t.controller != Controller::Gaia)
                    .map(|t| {
                        let who = self.who_plays(t, sibling);
                        // The engine lowercases faction names; a lobby seat on side Random keeps the word, its
                        // faction picked by the game at start (our own is read from the commander).
                        let side = if t.team == hello.team {
                            self.faction()
                        } else if t.side.eq_ignore_ascii_case("random") {
                            if t.ally_team == hello.ally_team { "faction in the seats line".to_string() } else { "faction chosen at start (Random in the lobby)".to_string() }
                        } else {
                            let mut side = t.side.chars();
                            let lobby = side.next().map(|c| c.to_ascii_uppercase().to_string() + side.as_str()).unwrap_or_default();
                            // The lobby's word stands only until the seat's units are seen (human-19, human-22:
                            // "Armada" all game of a person who took Cortex at the start).
                            match self.seen_faction(t.team) {
                                Some(seen) if seen == lobby => lobby,
                                Some(seen) => format!("{seen} by the units seen (the lobby said {lobby}: a person may change faction at the start)"),
                                None if t.ally_team != hello.ally_team && matches!(t.controller, Controller::Person { .. }) => format!("{lobby} in the lobby (a person may change faction at the start; none of its units seen yet)"),
                                None => lobby,
                            }
                        };
                        let colour = t.color.map(|c| format!(", {} in the lobby's colours", colour_name(c))).unwrap_or_default();
                        if side.is_empty() { format!("{who}{colour}") } else { format!("{who}, {side}{colour}") }
                    })
                    .collect();
                crate::strategist::shared::Side { ours: ally_team == hello.ally_team, ally_team, seats }
            })
            .collect()
    }
}

/// A colour's everyday name from its lobby RGB (0-1): the word people in the game use for a seat ("the purple
/// one"). Hue in twelve bands, "dark" or "pale" from lightness, "grey" when there is no colour to speak of.
pub(crate) fn colour_name(c: [f32; 3]) -> String {
    let [r, g, b] = c;
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let light = (max + min) / 2.0;
    let sat = if max == min { 0.0 } else { (max - min) / (1.0 - (2.0 * light - 1.0).abs()).max(1e-6) };
    if sat < 0.15 {
        return match light {
            l if l < 0.2 => "black",
            l if l < 0.7 => "grey",
            _ => "white",
        }
        .to_string();
    }
    let d = max - min;
    let hue = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let name = match hue {
        h if h < 15.0 || h >= 335.0 => "red",
        h if h < 40.0 => "orange",
        h if h < 65.0 => "yellow",
        h if h < 80.0 => "lime",
        h if h < 160.0 => "green",
        h if h < 195.0 => "teal",
        h if h < 215.0 => "sky blue",
        h if h < 255.0 => "blue",
        h if h < 290.0 => "purple",
        _ => "magenta",
    };
    match light {
        l if l < 0.25 && name != "red" => format!("dark {name}"),
        l if l > 0.75 && name == "red" => "pink".to_string(),
        l if l > 0.75 => format!("pale {name}"),
        _ => name.to_string(),
    }
}

impl Brain {
    /// The field the player reads (the `situation` tool, the roster and buildable checks) and the wake conditions watch.
    pub(super) fn publish_field(&self, tick: &Tick, kit: &Kit, soldiers: &[&OwnUnit], shared: &crate::strategist::shared::Shared) {
        let own = &tick.snapshot.own_units;
        let composition = |units: &[&&OwnUnit]| {
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for u in units {
                *counts.entry(self.name(u.def).to_string()).or_default() += 1;
            }
            counts.into_iter().collect::<Vec<_>>()
        };
        let pool: Vec<&&OwnUnit> = soldiers.iter().collect();
        let turrets: Vec<Vec3> = own.iter().filter(|u| u.def == kit.turret).map(|u| u.pos).collect();
        let extractors = own
            .iter()
            .filter(|u| kit.is_extractor(u.def))
            .map(|x| ExtractorStatus {
                at: self.place(x.pos),
                spot: self.world.hello.metal_spots.iter().position(|s| s.dist2d(x.pos) < self.spot_occupied_radius()),
                enemies_within_600: tick.snapshot.enemies.iter().filter(|e| e.pos.dist2d(x.pos) < 600.0).count(),
                turret_within_300: turrets.iter().any(|t| t.dist2d(x.pos) < 300.0),
            })
            .collect();
        // Everything a builder or factory of ours standing now can build (comet-1: the list was the bot lab's alone,
        // and every `produce` naming a Blitz or a Mason was refused all game), and the whole roster the commander
        // reaches by build lists (docs/design/2026-09-22-full-roster.md), each with its metal.
        let mut buildable: Vec<(String, u32)> = own
            .iter()
            .filter(|u| !u.being_built)
            .filter_map(|u| self.world.def(u.def))
            .filter(|d| !d.build_options.is_empty())
            .flat_map(|maker| maker.build_options.iter().filter_map(|id| self.world.def(*id)).map(|d| (d.name.clone(), d.metal_cost as u32)))
            .collect();
        buildable.sort();
        buildable.dedup_by(|a, b| a.0 == b.0);
        let roster: Vec<(String, u32)> = self.world.reachable_from(kit.commander).iter().filter_map(|id| self.world.def(*id)).map(|d| (d.name.clone(), d.metal_cost as u32)).collect();
        // Each builder's list words (`Field::builders`): its build options by internal name, the role words that
        // name one of them, and `extractor` when it builds the faction's; each with whether it stands at a place.
        let mut builders: BTreeMap<String, BTreeMap<String, bool>> = BTreeMap::new();
        for unit in own.iter().filter(|u| self.world.is_mobile_builder(u.def)) {
            let Some(d) = self.world.def(unit.def) else { continue };
            let mut words: BTreeMap<String, bool> = BTreeMap::new();
            for id in &d.build_options {
                if let Some(b) = self.world.def(*id) {
                    words.insert(b.name.clone(), self.placed_at_place(*id));
                }
            }
            // A role word names the builder's own faction's unit (a constructor given by a seat of another faction).
            let roster = super::roster::of_unit(&d.name);
            for role in super::roster::ROLE_WORDS {
                if let Some(name) = roster.role(role)
                    && let Some(id) = self.world.def_named(name)
                    && d.build_options.contains(&id)
                {
                    words.insert(role.to_string(), self.placed_at_place(id));
                }
            }
            // `extractor` is the builder's own: the tier-2 one for an advanced constructor (lists.rs `next_list_step`).
            if d.build_options.contains(&kit.extractor) || d.build_options.contains(&kit.advanced_extractor) {
                words.insert("extractor".to_string(), false);
            }
            builders.insert(self.actor_name(unit.id), words);
        }
        let enemy_extractors: Vec<(Vec3, i32)> = self
            .enemy_buildings
            .values()
            .filter(|(def, _, _)| self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0))
            .map(|(_, pos, seen)| (*pos, *seen))
            .collect();
        let enemy_extractor_at: Vec<Vec3> = enemy_extractors.iter().map(|(pos, _)| *pos).collect();
        let our_extractors: Vec<Vec3> = own.iter().filter(|u| kit.is_extractor(u.def)).map(|u| u.pos).collect();
        let held = |spot: Vec3, by: &[Vec3]| by.iter().any(|p| p.dist2d(spot) < self.spot_occupied_radius());
        // Free spots something of ours reaches (H-REPORT-FREE-SPOTS-BY-REACH): on foot, or only by the commander and
        // the amphibious, hover and ship constructors (under water, an islet); and by half.
        let free: Vec<(usize, Vec3)> = self
            .world
            .hello
            .metal_spots
            .iter()
            .enumerate()
            .filter(|(_, s)| self.reachable_by_any_class(**s))
            .filter(|(_, s)| !held(**s, &our_extractors) && !held(**s, &enemy_extractor_at) && !self.allied_extractor_on(**s))
            .map(|(n, s)| (n, *s))
            .collect();
        let ours_half = |s: Vec3| !self.his_half(s);
        let free_on_foot_ours = free.iter().filter(|(_, s)| self.reachable_on_foot(*s) && ours_half(*s)).count();
        let free_other_ours = free.iter().filter(|(_, s)| !self.reachable_on_foot(*s) && ours_half(*s)).count();
        let free_in_his_half = free.iter().filter(|(_, s)| !ours_half(*s)).count();
        let recent = |seen: &i32| tick.frame - seen < SIDE_TRADE_FRAMES;
        let metal = |u: &&OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.metal_cost);
        let traded = |since: i32| {
            let mut t = Traded::default();
            for e in self.trade_log.iter().filter(|e| e.frame >= since) {
                if e.ours { t.lost.add(e.kind, e.metal) } else { t.killed.add(e.kind, e.metal) }
            }
            t
        };
        let score = Score {
            map: self.world.hello.map.name.clone(),
            faction: self.faction().to_lowercase().chars().take(3).collect(),
            extractors: own.iter().filter(|u| kit.is_extractor(u.def) && !u.being_built).count(),
            extractor_peak: self.wake.extractor_peak,
            seconds_since_growth: (tick.frame - self.wake.growth_frame) / FRAMES_PER_SECOND,
            free_spots: free.len(),
            free_on_foot_ours,
            free_other_ours,
            free_in_his_half,
            other_reach: {
                let mut other: Vec<Vec3> = free.iter().filter(|(_, s)| !self.reachable_on_foot(*s) && ours_half(*s)).map(|(_, s)| *s).collect();
                other.sort_by(|a, b| a.dist2d(self.home).total_cmp(&b.dist2d(self.home)));
                other.first().map(|s| self.reach_words(*s, Some(own)))
            },
            next_free: {
                // Our half's nearest first (the walk for a spot our soldiers reach, the straight line for the rest),
                // his half's filling the list when ours has fewer than NEXT_FREE.
                let distance = |s: Vec3| if self.reachable_on_foot(s) { self.walk_from_home(s) } else { s.dist2d(self.home) };
                let mut ours: Vec<(usize, Vec3)> = free.iter().copied().filter(|(_, s)| ours_half(*s)).collect();
                let mut his: Vec<(usize, Vec3)> = free.iter().copied().filter(|(_, s)| !ours_half(*s)).collect();
                ours.sort_by(|a, b| distance(a.1).total_cmp(&distance(b.1)));
                his.sort_by(|a, b| distance(a.1).total_cmp(&distance(b.1)));
                let how = |s: Vec3, his: bool| match (his, self.reachable_on_foot(s)) {
                    (false, true) => None,
                    (false, false) => Some(format!("straight line; {}", self.reach_words(s, Some(own)))),
                    (true, true) => Some("in his half".to_string()),
                    (true, false) => Some(format!("in his half; straight line; {}", self.reach_words(s, Some(own)))),
                };
                ours.into_iter()
                    .map(|(n, s)| NextFree { n, place: self.place(s), distance: distance(s) as u32, how: how(s, false) })
                    .chain(his.into_iter().map(|(n, s)| NextFree { n, place: self.place(s), distance: distance(s) as u32, how: how(s, true) }))
                    .take(NEXT_FREE)
                    .collect()
            },
            soldiers: soldiers.len(),
            army_metal: soldiers.iter().map(metal).sum::<f32>() as u32,
            soldiers_near_home: soldiers.iter().filter(|u| u.pos.dist2d(self.home) < SCORE_AT_HOME).count(),
            soldiers_in_his_half: soldiers.iter().filter(|u| self.his_half(u.pos)).count(),
            army_metal_in_his_half: soldiers.iter().filter(|u| self.his_half(u.pos)).map(metal).sum::<f32>() as u32,
            seconds_since_ours_in_his_half: self.ours_in_his_half.map(|f| (tick.frame - f) / FRAMES_PER_SECOND),
            metal_income: tick.snapshot.metal.income,
            trend: [3, 6].into_iter().filter_map(|m| self.minutes_ago(tick.frame, m).map(|(x, income, army)| (m, x, income, army))).collect(),
            extractors_lost_3_min: self.wake.losses.len(),
            traded_1_min: traded(tick.frame - OWN_TRADE_FRAMES),
            traded_3_min: traded(tick.frame - SIDE_TRADE_FRAMES),
            traded: traded(i32::MIN),
            enemy_deaths: self.enemy_deaths.clone(),
            frame: tick.frame,
            seconds_since_turn: Some(shared.last_turn_frame.load(std::sync::atomic::Ordering::Relaxed)).filter(|at| *at > 0).map(|at| (tick.frame - at) / FRAMES_PER_SECOND),
            enemy_start_boxes: self.world.hello.start_boxes.iter().filter(|b| b.ally_team != self.world.hello.ally_team).map(|b| self.world.box_cells(b)).collect(),
            scouting: self.scouting(tick),
            enemy_spots: {
                // Each spot with the latest look at an extractor of his on it: a sighting and its age, not his count.
                let radius = self.spot_occupied_radius();
                let mut spots: Vec<(f32, usize, i32)> = self
                    .world
                    .hello
                    .metal_spots
                    .iter()
                    .enumerate()
                    .filter_map(|(n, s)| {
                        let seen = enemy_extractors.iter().filter(|(p, _)| p.dist2d(*s) < radius).map(|(_, seen)| *seen).max()?;
                        Some((s.dist2d(self.home), n, (tick.frame - seen) / FRAMES_PER_SECOND))
                    })
                    .collect();
                spots.sort_by(|a, b| a.0.total_cmp(&b.0));
                spots.into_iter().map(|(_, n, ago)| (n, self.place(self.world.hello.metal_spots[n]), ago)).collect()
            },
            enemy_factories: self
                .enemy_buildings
                .values()
                .filter(|(def, _, _)| self.world.def(*def).is_some_and(|d| !d.build_options.is_empty()))
                .map(|(def, pos, _)| (self.name(*def).to_string(), self.place(*pos)))
                .collect(),
            enemy_factories_gone: self.enemy_factories_gone.iter().map(|(_, pos, at)| (self.place(*pos), at / FRAMES_PER_SECOND)).collect(),
            enemy_commander: self.enemy_commander_seen.map(|(pos, seen)| (self.place(pos), (tick.frame - seen) / FRAMES_PER_SECOND)),
            enemy_commander_afloat: self.enemy_commander_seen.is_some_and(|(pos, _)| !self.reachable_on_foot(pos)),
            enemy_soldiers_seen: self.enemy_soldiers.values().filter(|(_, _, seen)| recent(seen)).count(),
            enemy_soldiers_seen_metal: self.enemy_soldiers.values().filter(|(_, _, seen)| recent(seen)).map(|(def, _, _)| self.world.def(*def).map_or(0.0, |d| d.metal_cost)).sum::<f32>() as u32,
        };
        let mut wreck_fields: Vec<(crate::strategist::shared::Place, u32, bool)> = self.reclaim.fields.iter().map(|f| (self.place(f.at), f.metal as u32, f.safe)).collect();
        wreck_fields.sort_by_key(|f| std::cmp::Reverse(f.1));
        shared.publish_field(self.world.hello.team, Field {
            score,
            wreck_fields,
            resurrection_bots: own.iter().filter(|u| kit.is_resurrector(u.def)).count(),
            unassigned: composition(&pool),
            unassigned_centre: centre_of_units(&pool).map(|c| self.place(c)),
            extractors,
            turrets: turrets.iter().map(|t| self.place(*t)).collect(),
            buildable,
            roster,
            builders,
            factions: vec![(self.world.hello.team, self.faction())],
        });
    }
}

#[cfg(test)]
mod colour_tests {
    use super::colour_name;

    #[test]
    fn lobby_colours_get_their_everyday_names() {
        // The lobby colours of the games with people on 2026-09-27.
        assert_eq!(colour_name([0.63922, 0.08235, 0.88235]), "purple");
        assert_eq!(colour_name([0.0, 0.96863, 0.0]), "green");
        assert_eq!(colour_name([0.0, 0.0, 0.92941]), "blue");
        assert_eq!(colour_name([0.86667, 0.0, 0.27059]), "red");
        assert_eq!(colour_name([0.78039, 0.0, 0.89804]), "magenta");
        assert_eq!(colour_name([0.72549, 0.88627, 0.69020]), "pale green");
        assert_eq!(colour_name([0.5, 0.5, 0.5]), "grey");
        assert_eq!(colour_name([0.44706, 0.0, 0.85098]), "purple");
    }
}

#[cfg(test)]
mod frame_loss_tests {
    use bot_protocol::{Death, Event, OwnUnit, Resource, Snapshot, Tick, UnitDefId, UnitId, Vec3, Weapon};

    use super::super::pianist::fixtures::{at, brain_of, own};
    use super::super::roster::Kit;
    use super::{Cause, FrameFate, BLAST_WINDOW_FRAMES};

    fn frame(id: i32, pos: Vec3, share: f32) -> OwnUnit {
        OwnUnit { being_built: true, health: share * 100.0, max_health: 100.0, build_progress: share, ..own(id, 2, pos) }
    }

    fn tick(frame: i32, ours: Vec<OwnUnit>, events: Vec<Event>) -> Tick {
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        Tick { frame, late: 0, events, snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours, allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None } }
    }

    /// An unfinished frame destroyed by nobody named is a decay when the kill event says so, and a shot from out of
    /// sight when a weapon killed it; with a named attacker it was killed. Each is counted at the furthest the frame
    /// had been built, not the share left after decay (the report said "0% built" of a frame left at 98 %).
    #[test]
    fn an_unfinished_frame_is_told_a_decay_from_a_kill_and_counted_at_its_peak() {
        let mut brain = brain_of(&["armcom", "armsolar"]);
        let none = UnitDefId(99);
        let kit = Kit { commander: UnitDefId(1), extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let builders = || vec![own(10, 1, at(1000.0, 950.0)), own(11, 1, at(3000.0, 950.0))];
        let with = |frames: Vec<OwnUnit>| builders().into_iter().chain(frames).collect::<Vec<_>>();
        // Four frames rise: 20 and 23 reach 70 %, 21 reaches 60 %, 22 reaches 30 %.
        brain.track_losses(&tick(30, with(vec![frame(20, at(1000.0, 1000.0), 0.4), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.7)]), vec![Event::UnitCreated { unit: UnitId(20), builder: Some(UnitId(10)) }, Event::UnitCreated { unit: UnitId(23), builder: Some(UnitId(11)) }]), &kit);
        brain.track_losses(&tick(60, with(vec![frame(20, at(1000.0, 1000.0), 0.7), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.7)]), Vec::new()), &kit);
        // Frame 20 decays to 10 % with its builder gone elsewhere; 23's builder dies.
        brain.track_losses(&tick(90, with(vec![frame(20, at(1000.0, 1000.0), 0.1), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.05)]), Vec::new()), &kit);
        let gun = || Death::Weapon(Weapon { name: "corak_gator_laser".into(), range: 190.0 });
        let events = vec![
            Event::UnitDestroyed { unit: UnitId(20), attacker: None, death: Death::Decayed },
            Event::UnitDestroyed { unit: UnitId(21), attacker: None, death: gun() },
            Event::UnitDestroyed { unit: UnitId(22), attacker: Some(UnitId(500)), death: gun() },
            Event::UnitDestroyed { unit: UnitId(11), attacker: Some(UnitId(500)), death: gun() },
            Event::UnitDestroyed { unit: UnitId(23), attacker: None, death: Death::Decayed },
        ];
        brain.track_losses(&tick(120, vec![own(10, 1, at(1000.0, 950.0))], events), &kit);
        let loss = |id: i32| brain.frame_losses_now.get(&UnitId(id)).copied().expect("a frame loss");
        assert_eq!((loss(20).fate, (loss(20).share * 100.0).round()), (FrameFate::Decayed { builder_dead: false }, 70.0));
        assert_eq!((loss(21).fate, (loss(21).share * 100.0).round()), (FrameFate::ShotUnseen, 60.0));
        assert_eq!((loss(22).fate, (loss(22).share * 100.0).round()), (FrameFate::Killed, 30.0));
        assert_eq!(loss(23).fate, FrameFate::Decayed { builder_dead: true });
        let lines: Vec<&String> = brain.fight_ledger.keys().collect();
        assert!(lines.iter().any(|l| l.as_str() == "abandoned an unfinished armsolar (70% built)"), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("lost armsolar (60% built) to unseen fire")), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("lost armsolar (30% built) to unseen")), "{lines:?}");
        // The metal traded away is the peak share of each frame's cost (a solar is 155), not the share at its end.
        let cost = brain.world.def(UnitDefId(2)).expect("a solar").metal_cost;
        let frames_metal: f32 = brain.trade_log.iter().filter(|t| t.frame == 120 && t.ours && t.metal < cost * 0.95).map(|t| t.metal).sum();
        assert!((frames_metal - cost * (0.7 + 0.6 + 0.3 + 0.7)).abs() < 1.0, "{frames_metal} of {cost}");
    }

    /// A loss with no attacker named is told its cause (H-REPORT-BLAST-CAUSE): a hit by a warhead that reaches beyond
    /// the map is a missile, a death within a unit's death blast as that unit dies is that blast (his, or our own),
    /// a named killer stays the killer, and the rest stays unseen; one line a strike says what landed where
    /// (apex60-1: 15 of ours at 33:12 and 21 at 50:36 under missiles, 77 at 57:17 under a Juggernaut's blast, every
    /// one "lost ... to unseen").
    #[test]
    fn a_missile_and_a_death_blast_are_named_as_the_cause_of_unattributed_losses() {
        use bot_protocol::{Blast, EnemyUnit};
        let mut brain = brain_of(&["armcom", "armsolar", "corkorg"]);
        for d in brain.world.hello.unit_defs.iter_mut() {
            match d.name.as_str() {
                "corkorg" => d.death_blast = Some(Blast { radius: 640.0, damage: 10600.0 }),
                "armcom" => d.death_blast = Some(Blast { radius: 350.0, damage: 5000.0 }),
                _ => {}
            }
        }
        let none = UnitDefId(99);
        let kit = Kit { commander: UnitDefId(1), extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        // Solars 20 to 23 near the Juggernaut (500) at (1150, 1050), 23 far from it; the commander (10) and solar 24 far away.
        let ours = || vec![own(10, 1, at(5000.0, 5000.0)), own(20, 2, at(1000.0, 1000.0)), own(21, 2, at(1100.0, 1000.0)), own(22, 2, at(1200.0, 1000.0)), own(23, 2, at(3000.0, 1000.0)), own(24, 2, at(5100.0, 5000.0))];
        let mut first = tick(30, ours(), Vec::new());
        first.snapshot.enemies = vec![EnemyUnit { id: UnitId(500), def: Some(UnitDefId(3)), pos: at(1150.0, 1050.0), vel: Vec3::default(), health: 100.0, team: Some(1), being_built: false }];
        brain.track_losses(&first, &kit);
        let warhead = || Death::Weapon(Weapon { name: "corsilo_crblmssl".into(), range: 72000.0 });
        let explosion = || Death::Weapon(Weapon { name: "korgexplosion".into(), range: 10.0 });
        let gun = || Death::Weapon(Weapon { name: "corkorg_corkorg_laser".into(), range: 900.0 });
        let events = vec![
            Event::UnitDestroyed { unit: UnitId(20), attacker: None, death: warhead() },
            Event::EnemyDestroyed { enemy: UnitId(500), attacker: None },
            Event::UnitDestroyed { unit: UnitId(21), attacker: None, death: explosion() },
            Event::UnitDestroyed { unit: UnitId(22), attacker: Some(UnitId(500)), death: gun() },
            Event::UnitDestroyed { unit: UnitId(23), attacker: None, death: Death::Unknown },
            Event::UnitDestroyed { unit: UnitId(10), attacker: None, death: Death::Unknown },
            Event::UnitDestroyed { unit: UnitId(24), attacker: None, death: Death::Unknown },
        ];
        brain.track_losses(&tick(60, Vec::new(), events), &kit);
        let lines: Vec<&String> = brain.fight_ledger.keys().collect();
        let has = |start: &str| lines.iter().any(|l| l.starts_with(start));
        assert!(has("lost armsolar to a missile "), "{lines:?}");
        assert!(has("lost armsolar to the death blast of a corkorg "), "{lines:?}");
        assert!(has("lost armsolar to corkorg "), "{lines:?}");
        assert!(has("lost armsolar to our own armcom's death blast "), "{lines:?}");
        assert!(has("lost armsolar to unseen "), "{lines:?}");
        assert!(has("lost armcom to unseen "), "{lines:?}");
        // The strike lines: one a cause, at the centre of its dead, a missile's marked for the wake.
        let strikes = vec![(Cause::Missile, at(1000.0, 1000.0), UnitDefId(2), 155.0), (Cause::Missile, at(1200.0, 1000.0), UnitDefId(2), 155.0), (Cause::Blast { name: Some("corkorg".into()), ours: false }, at(1100.0, 1000.0), UnitDefId(2), 155.0)];
        let texts = brain.strike_lines(&strikes);
        assert_eq!(texts.len(), 2, "{texts:?}");
        let missile = texts.iter().find(|(m, _)| *m).map(|(_, t)| t.as_str()).expect("a missile line");
        assert!(missile.starts_with("a missile landed at ") && missile.ends_with("(1100, 1000): 2 of ours died, 310 metal (2 armsolar)"), "{missile}");
        let blast = texts.iter().find(|(m, _)| !*m).map(|(_, t)| t.as_str()).expect("a blast line");
        assert!(blast.starts_with("the death blast of a corkorg went off at ") && blast.ends_with(": 1 of ours died, 155 metal (1 armsolar)"), "{blast}");
    }

    /// The engine names one of ours as the attacker for friendly fire and for a reclaim, never for a death blast, and
    /// a blast's victims fall after the exploder (a fusion's 9-18 frames later, an advanced fusion's about 210): a
    /// loss to our own unit is its fire, or its taking apart when nothing hit the dead one; a hit by an explosion
    /// weapon (no range) is the blast of the nearest unit that died with one in the last ten seconds, or "a death
    /// blast" with none; a gun's hit with nobody named is fire from out of sight, never a blast (apex60-2: all 20
    /// "our own <type>'s death blast" lines were Mauser, Janus and Bulldog shells or a constructor's reclaim, every
    /// attacker alive after, and the fusion's four victims at 16:43 read "unseen").
    #[test]
    fn friendly_fire_a_reclaim_and_a_late_blast_are_told_apart() {
        use bot_protocol::Blast;
        let mut brain = brain_of(&["armcom", "armsolar", "armjanus", "armcv", "armfus", "corstorm"]);
        for d in brain.world.hello.unit_defs.iter_mut() {
            if d.name == "armfus" {
                d.death_blast = Some(Blast { radius: 240.0, damage: 2650.0 });
            }
        }
        let none = UnitDefId(99);
        let kit = Kit { commander: UnitDefId(1), extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        // The fusion (40) at (1000, 1000) with solars 22, 23 and 25 beside it; the Janus (30), the constructor (31)
        // and solars 20, 21 far off together; solar 24 alone.
        let ours = || {
            vec![
                own(10, 1, at(5000.0, 5000.0)),
                own(30, 3, at(4000.0, 4000.0)),
                own(31, 4, at(4100.0, 4000.0)),
                own(40, 5, at(1000.0, 1000.0)),
                own(20, 2, at(4200.0, 4000.0)),
                own(21, 2, at(4300.0, 4000.0)),
                own(22, 2, at(1100.0, 1000.0)),
                own(23, 2, at(1200.0, 1000.0)),
                own(24, 2, at(3000.0, 3000.0)),
                own(25, 2, at(1150.0, 1000.0)),
                own(26, 2, at(3200.0, 3000.0)),
            ]
        };
        brain.track_losses(&tick(30, ours(), Vec::new()), &kit);
        let gun = |name: &str, range: f32| Death::Weapon(Weapon { name: name.into(), range });
        // The Janus's shell kills solar 20, the constructor takes solar 21 apart, a Dragon out of sight kills the
        // fusion, solar 26 self-destructs.
        let events = vec![
            Event::UnitDestroyed { unit: UnitId(20), attacker: Some(UnitId(30)), death: gun("armjanus_janus", 450.0) },
            Event::UnitDestroyed { unit: UnitId(21), attacker: Some(UnitId(31)), death: Death::Reclaimed },
            Event::UnitDestroyed { unit: UnitId(40), attacker: None, death: gun("corcrwh_dragon_laser", 840.0) },
            Event::UnitDestroyed { unit: UnitId(26), attacker: None, death: Death::SelfDestruct },
        ];
        brain.track_losses(&tick(60, Vec::new(), events), &kit);
        // Fifteen frames on: the fusion's explosion takes solar 22; a Storm's rocket from out of sight takes solar
        // 23 beside it; an explosion with no unit dying near it takes solar 24.
        let events = vec![
            Event::UnitDestroyed { unit: UnitId(22), attacker: None, death: gun("fusionexplosion", 10.0) },
            Event::UnitDestroyed { unit: UnitId(23), attacker: None, death: gun("corstorm_cor_bot_rocket", 600.0) },
            Event::UnitDestroyed { unit: UnitId(24), attacker: None, death: gun("metalmaker", 10.0) },
        ];
        brain.track_losses(&tick(75, Vec::new(), events), &kit);
        // Eleven seconds after the fusion fell, a death beside its site with nobody named and no cause is unseen.
        brain.track_losses(&tick(60 + BLAST_WINDOW_FRAMES + 30, Vec::new(), vec![Event::UnitDestroyed { unit: UnitId(25), attacker: None, death: Death::Unknown }]), &kit);
        let lines: Vec<&String> = brain.fight_ledger.keys().collect();
        let has = |start: &str| lines.iter().any(|l| l.starts_with(start));
        assert!(has("lost armsolar to our own armjanus's fire "), "{lines:?}");
        assert!(has("lost armsolar to our own armcv taking it apart "), "{lines:?}");
        assert!(has("lost armsolar to our own armfus's death blast "), "{lines:?}");
        assert!(has("lost armsolar to a corstorm out of sight "), "{lines:?}");
        assert!(has("lost armfus to unseen fire "), "{lines:?}");
        assert!(has("lost armsolar to its own self-destruct "), "{lines:?}");
        assert!(has("lost armsolar to a death blast "), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("lost armsolar to unseen ") && !l.starts_with("lost armsolar to unseen fire")), "{lines:?}");
        assert!(!lines.iter().any(|l| l.contains("armjanus's death blast") || l.contains("armcv's death blast")), "{lines:?}");
        let texts = brain.strike_lines(&[(Cause::Blast { name: None, ours: false }, at(3000.0, 3000.0), UnitDefId(2), 155.0)]);
        assert!(texts.len() == 1 && texts[0].1.starts_with("a death blast went off at "), "{texts:?}");
    }

    /// With the engine's sight grid, a remembered building is gone only when its square has been in that sight, with
    /// the building absent, for the dwell (two seconds: the grid is half a second old on every other think, so a one-look absence
    /// means nothing); a
    /// type that cloaks is hidden, not gone, until one of ours stands within its decloak distance for as long
    /// (H-ENEMY-GONE-NEEDS-SIGHT; Altored Divide, human-8v8-1: the ridge hid the whole north-east from the G4 radar;
    /// sight-regress-1: 100 of 107 "gone" events named a building still standing).
    #[test]
    fn a_building_is_gone_only_in_the_engines_sight_for_the_dwell_and_a_cloaked_one_only_when_decloaked() {
        use bot_protocol::{Cloak, EnemyUnit, Grid, Sensors};
        let mut brain = brain_of(&["armcom", "armsolar", "armpb"]);
        let enemy = |id: i32, def: i32, pos: Vec3| EnemyUnit { id: UnitId(id), def: Some(UnitDefId(def)), pos, vel: Vec3::default(), health: 100.0, team: Some(1), being_built: false };
        let look = |frame: i32, ours: Vec<OwnUnit>, enemies: Vec<EnemyUnit>| {
            let mut t = tick(frame, ours, Vec::new());
            t.snapshot.enemies = enemies;
            t
        };
        // The engine's grid says we see the west half of the map and nothing east of x 4096.
        let west = || Grid::pack(128, 128, 64.0, (0..128 * 128).map(|i| i % 128 < 64));
        let grid = |brain: &mut super::Brain, frame: i32| brain.sensors = Some((Sensors { los: west(), radar: Grid::default(), air: Grid::default() }, frame));
        let has = |brain: &super::Brain, id: i32| brain.enemy_buildings.contains_key(&UnitId(id));
        // Seen: a solar at (1000, 1000) and a Pitbull at (3000, 1000); the Pitbull cloaks (55).
        grid(&mut brain, 30);
        brain.track_enemy_buildings(&look(30, vec![own(10, 1, at(900.0, 900.0))], vec![enemy(50, 2, at(1000.0, 1000.0)), enemy(51, 3, at(3000.0, 1000.0))]));
        assert_eq!(brain.enemy_buildings.len(), 2);
        let pitbull = brain.world.hello.unit_defs.iter().position(|d| d.id == UnitDefId(3)).expect("the Pitbull");
        brain.world.hello.unit_defs[pitbull].cloak = Some(Cloak { cost: 16.0, decloak_distance: 55.0, decloak_on_fire: true, starts_cloaked: false });
        // One look with neither in the snapshot: the solar's square is in sight with nothing there (a second unit
        // of ours stands 100 from it), but a single look is the lag's, not the truth's; the commander stands 200
        // from the Pitbull, which cloaks.
        let ours = || vec![own(10, 1, at(2800.0, 1000.0)), own(11, 1, at(1100.0, 1000.0))];
        grid(&mut brain, 45);
        brain.track_enemy_buildings(&look(45, ours(), Vec::new()));
        assert!(has(&brain, 50), "one look is not the dwell");
        // Back in the snapshot at 60: the clock resets.
        grid(&mut brain, 60);
        brain.track_enemy_buildings(&look(60, ours(), vec![enemy(50, 2, at(1000.0, 1000.0))]));
        // Absent from 75 on: kept at 75, 105 and 120, gone at 135 (two seconds since 75).
        for frame in [75, 105, 120] {
            grid(&mut brain, frame);
            brain.track_enemy_buildings(&look(frame, ours(), Vec::new()));
            assert!(has(&brain, 50), "{frame}: in sight with nothing there, but not yet for the dwell");
        }
        grid(&mut brain, 135);
        brain.track_enemy_buildings(&look(135, ours(), Vec::new()));
        assert!(!has(&brain, 50), "the solar's square was in the engine's sight, within ours' circle, with nothing there for two seconds: gone");
        assert!(has(&brain, 51), "the Pitbull cloaks: hidden, not gone, with ours 200 away all along");
        // The grid alone is not sight: a place the grid covers with none of this seat's units within their circle
        // of it (an ally's sight, or a unit that has since stepped away) keeps its building however long.
        brain.track_enemy_buildings(&look(150, vec![own(10, 1, at(2800.0, 1000.0))], vec![enemy(53, 2, at(1500.0, 1500.0))]));
        for frame in [165, 195, 225, 255] {
            grid(&mut brain, frame);
            brain.track_enemy_buildings(&look(frame, vec![own(10, 1, at(2800.0, 1000.0))], Vec::new()));
        }
        assert!(has(&brain, 53), "the grid covers the place but none of ours stands within sight of it: kept");
        // The commander walks to 40 of the Pitbull: within its decloak distance it would show, so its absence
        // counts, and after the dwell it is gone.
        for frame in [150, 180] {
            grid(&mut brain, frame);
            brain.track_enemy_buildings(&look(frame, vec![own(10, 1, at(2960.0, 1000.0))], Vec::new()));
            assert!(has(&brain, 51), "{frame}: decloaked and absent, not yet for the dwell");
        }
        grid(&mut brain, 210);
        brain.track_enemy_buildings(&look(210, vec![own(10, 1, at(2960.0, 1000.0))], Vec::new()));
        assert!(!has(&brain, 51));
        // A building behind the hill (east, no sight in the grid) with ours standing 100 away is kept however long:
        // the circle no longer decides.
        grid(&mut brain, 240);
        brain.track_enemy_buildings(&look(240, vec![own(10, 1, at(5000.0, 1000.0))], vec![enemy(52, 2, at(5100.0, 1000.0))]));
        for frame in [270, 330, 390] {
            grid(&mut brain, frame);
            brain.track_enemy_buildings(&look(frame, vec![own(10, 1, at(5000.0, 1000.0))], Vec::new()));
        }
        assert!(has(&brain, 52), "no sight over the place in the engine's grid: kept");
    }
}
