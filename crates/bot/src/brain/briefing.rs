//! The brain's side of the player's session: the briefing, the field, events and triggers out.

use std::collections::{BTreeMap, HashMap, HashSet};

use bot_protocol::{UnitId, Event, OwnUnit, Tick, UnitDefId, Vec3};
use serde_json::json;

use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};
use crate::strategist::shared::{Briefing, Counts, Draw, EnemyCluster, EnemyDeath, ExtractorStatus, Field, Group, Place, RememberedBuilding, Score, Spending, TradeKind, Traded, OWN_TRADE_FRAMES, SIDE_TRADE_FRAMES};

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

/// A frame destroyed with no attacker named after a hit in these seconds was shot by something out of sight, not
/// left to decay.
const SHOT_WINDOW: i32 = 15 * FRAMES_PER_SECOND;

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

    /// Notes our own losses by name, and wakes the strategist when extractors go down in numbers.
    pub(super) fn track_losses(&mut self, tick: &Tick, kit: &Kit) {
        const LOSSES_WORTH_WAKING_FOR: usize = 2;
        self.frame_losses_now.clear();
        // A frame's builder and its last hit, so a frame destroyed with no attacker named (the engine names none out
        // of sight) is told a decay, which nobody shot, from a kill: 106 of 311 "decayed" frames in 23 games had
        // taken damage in their last seconds, and the report said of each that its builder had been sent elsewhere
        // (docs/studies/2026-10-17-unfinished-structures.md §2.2).
        for event in &tick.events {
            match event {
                Event::UnitCreated { unit, builder: Some(builder) } => {
                    self.frame_builders.insert(*unit, (*builder, tick.frame));
                }
                Event::UnitDamaged { unit, .. } if self.unfinished.contains_key(unit) => {
                    self.frame_hits.insert(*unit, tick.frame);
                }
                _ => {}
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
            let Event::UnitDestroyed { unit, attacker } = event else { continue };
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
            let last_hit = self.frame_hits.remove(unit);
            self.frame_alone_since.remove(unit);
            // An unfinished frame: the metal put in is its peak share of the cost (left to decay it reads near 0 at
            // its end, and the report said "0% built" of every decay until 2026-10-17), and what became of it is
            // read from its last hit and its builder.
            let frame_loss = self.unfinished.remove(unit).map(|share| {
                let fate = if attacker.is_some() {
                    FrameFate::Killed
                } else if last_hit.is_some_and(|f| tick.frame - f <= SHOT_WINDOW) {
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
            let unseen_fire = frame_loss.as_ref().is_some_and(|l| matches!(l.fate, FrameFate::ShotUnseen));
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
            let killer = attacker.and_then(|id| self.enemy_defs.get(&id)).map_or(if unseen_fire { "unseen fire" } else { "unseen" }, |d| self.name(*d));
            let place = if pos.dist2d(self.home) < BASE_RADIUS { "at home" } else if pos.dist2d(self.home) < pos.dist2d(self.world.mirrored(self.home)) { "in our half" } else { "in their half" };
            let line = format!("lost {}{built} to {killer} {place}", self.name(def));
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
                let share = (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0);
                (u.id, peak.remove(&u.id).map_or(share, |p| p.max(share)))
            })
            .collect();
        self.frame_hits.retain(|id, _| self.unfinished.contains_key(id));
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
                share: (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0),
                cost: self.world.def(u.def).map_or(0.0, |d| d.metal_cost),
                builders: on.get(&u.id).cloned().unwrap_or_default(),
                alone_seconds: self.frame_alone_since.get(&u.id).map(|since| (frame - since) / FRAMES_PER_SECOND),
            })
            .collect();
        frames.sort_by(|a, b| b.builders.is_empty().cmp(&a.builders.is_empty()).then(b.share.total_cmp(&a.share)));
        frames
    }

    pub(super) fn track_enemy_buildings(&mut self, tick: &Tick) {
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
        // A remembered building whose place a unit of ours sees, with nothing of his there, is gone (the impedance
        // tally, 2026-10-16: nuke-1-medium's report listed his labs as standing "in sight now" for twelve minutes
        // after our missiles killed them out of sight, and two more missiles went at them). A radar contact at the
        // place is still in the snapshot by its id, so it keeps the memory; the sight is taken a fifth short of the
        // unit's, a hill between them being invisible here.
        let in_snapshot: std::collections::HashSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        let sights: Vec<(Vec3, f32)> = tick.snapshot.own_units.iter().filter(|u| !u.being_built).map(|u| (u.pos, self.sight_of(u.def) * 0.8)).collect();
        let razed: Vec<UnitId> = self
            .enemy_buildings
            .iter()
            .filter(|(id, (_, pos, _))| !in_snapshot.contains(id) && sights.iter().any(|(at, sight)| at.dist2d(*pos) <= *sight))
            .map(|(id, _)| *id)
            .collect();
        for id in razed {
            if let Some((def, pos, _)) = self.enemy_buildings.remove(&id) {
                if self.world.is_factory_def(def) {
                    self.enemy_factories_gone.push((def, pos, tick.frame));
                }
                self.event(tick.frame, format!("his {} at {} is gone: the place is in sight and nothing stands there", self.name(def), self.world.grid(pos)));
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
        let briefing = Briefing {
            sides,
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
            .map(|d| format!("{} ({}{})", d.name, if d.weapon_count > 0 { "armed" } else { "a builder" }, if lab_builds.contains(&d.id) { ", from the bot lab" } else { "" }))
            .collect();
        json!({
            "share": format!("{share:.0}% of the map is under water (the sketch's ~)"),
            "note": "Our bots and vehicles stop at the shore, except through fords their class wades (tanks to a depth of about 20, bots likewise: a ford is a crossing, deep water is not); a spot or a place in deep water is reached only by what is listed below, and an advance of tanks toward it stalls at the shore. The commander is amphibious: it walks on the sea floor, and so does the enemy's, which can retreat into the sea when its base is gone and build on the shore from the water; its construction ships take the under-water spots and its ships shell the shore from beyond our reach.",
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
            "metal_spots_note": "n is the spot's number for the `expansion` tool; walk_from_home is the walking distance for our bots (null: they cannot walk there) beside straight_from_home; a spot marked `alcove` is far longer on foot than straight, and a group sent at it by the straight line huddles short of it",
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
                let sibling = if crate::strategist::player_per_seat() { "another seat of this bot, with its own player as you are this seat's: it reads its own report and orders its own units" } else { "another seat of this bot" };
                let seats = hello
                    .teams
                    .iter()
                    .filter(|t| t.ally_team == ally_team && t.controller != Controller::Gaia)
                    .map(|t| {
                        let who = match &t.controller {
                            _ if t.team == hello.team => "you (WReason)".to_string(),
                            Controller::Person { name, skill: Some(skill) } => format!("{name} (a person, lobby skill {skill:.0})"),
                            Controller::Person { name, skill: None } => format!("{name} (a person)"),
                            Controller::Ai { short_name, profile: Some(profile), .. } if short_name.eq_ignore_ascii_case("wreason") => format!("Within Reason {profile} ({})", sibling),
                            Controller::Ai { short_name, .. } if short_name.eq_ignore_ascii_case("wreason") => format!("Within Reason ({sibling})"),
                            Controller::Ai { short_name, profile: Some(profile), .. } => format!("{short_name} {profile} (an AI)"),
                            Controller::Ai { short_name, .. } => format!("{short_name} (an AI)"),
                            Controller::Gaia => unreachable!("filtered above"),
                            Controller::Unknown => format!("team {} (who plays it is not in the script)", t.team),
                        };
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
            for role in super::roster::ROLE_WORDS {
                if let Some(name) = kit.roster.role(role)
                    && let Some(id) = self.world.def_named(name)
                    && d.build_options.contains(&id)
                {
                    words.insert(role.to_string(), self.placed_at_place(id));
                }
            }
            if d.build_options.contains(&kit.extractor) {
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
        let free: Vec<Vec3> = self
            .world
            .hello
            .metal_spots
            .iter()
            .filter(|s| self.reachable_on_foot(**s))
            .filter(|s| !held(**s, &our_extractors) && !held(**s, &enemy_extractor_at) && !self.allied_extractor_on(**s))
            .copied()
            .collect();
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
            next_free: {
                let mut nearest: Vec<(usize, Vec3, f32)> = self
                    .world
                    .hello
                    .metal_spots
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| free.iter().any(|f| f.dist2d(**s) < 1.0))
                    .map(|(n, s)| (n, *s, self.walk_from_home(*s)))
                    .collect();
                nearest.sort_by(|a, b| a.2.total_cmp(&b.2));
                nearest.into_iter().take(NEXT_FREE).map(|(n, s, walk)| (n, self.place(s), walk as u32)).collect()
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
            scouting: self.scouting(tick.frame),
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
    use bot_protocol::{Event, OwnUnit, Resource, Snapshot, Tick, UnitDefId, UnitId, Vec3};

    use super::super::pianist::fixtures::{at, brain_of, own};
    use super::super::roster::Kit;
    use super::FrameFate;

    fn frame(id: i32, pos: Vec3, share: f32) -> OwnUnit {
        OwnUnit { being_built: true, health: share * 100.0, max_health: 100.0, ..own(id, 2, pos) }
    }

    fn tick(frame: i32, ours: Vec<OwnUnit>, events: Vec<Event>) -> Tick {
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        Tick { frame, late: 0, events, snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours, allies: Vec::new(), enemies: Vec::new(), wrecks: None } }
    }

    /// An unfinished frame destroyed by nobody named is a decay only when nothing shot it: after a hit it was shot
    /// by something out of sight; with a named attacker it was killed. Each is counted at the furthest the frame
    /// had been built, not the share left after decay (the report said "0% built" of a frame left at 98 %).
    #[test]
    fn an_unfinished_frame_is_told_a_decay_from_a_kill_and_counted_at_its_peak() {
        let mut brain = brain_of(&["armcom", "armsolar"]);
        let none = UnitDefId(99);
        let kit = Kit { commander: UnitDefId(1), extractor: none, converter: none, lab: none, turret: none, constructor: none, plant: none, vehicle_constructor: none, advanced_lab: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let builders = || vec![own(10, 1, at(1000.0, 950.0)), own(11, 1, at(3000.0, 950.0))];
        let with = |frames: Vec<OwnUnit>| builders().into_iter().chain(frames).collect::<Vec<_>>();
        // Four frames rise: 20 and 23 reach 70 %, 21 reaches 60 %, 22 reaches 30 %.
        brain.track_losses(&tick(30, with(vec![frame(20, at(1000.0, 1000.0), 0.4), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.7)]), vec![Event::UnitCreated { unit: UnitId(20), builder: Some(UnitId(10)) }, Event::UnitCreated { unit: UnitId(23), builder: Some(UnitId(11)) }]), &kit);
        brain.track_losses(&tick(60, with(vec![frame(20, at(1000.0, 1000.0), 0.7), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.7)]), Vec::new()), &kit);
        // Frame 20 decays to 10 % with its builder gone elsewhere; 23's builder dies.
        brain.track_losses(&tick(90, with(vec![frame(20, at(1000.0, 1000.0), 0.1), frame(21, at(1100.0, 1000.0), 0.6), frame(22, at(1200.0, 1000.0), 0.3), frame(23, at(3000.0, 1000.0), 0.05)]), Vec::new()), &kit);
        let events = vec![
            Event::UnitDestroyed { unit: UnitId(20), attacker: None },
            Event::UnitDamaged { unit: UnitId(21), attacker: None, damage: 300.0, from: None, weapon: None },
            Event::UnitDestroyed { unit: UnitId(21), attacker: None },
            Event::UnitDestroyed { unit: UnitId(22), attacker: Some(UnitId(500)) },
            Event::UnitDestroyed { unit: UnitId(11), attacker: Some(UnitId(500)) },
            Event::UnitDestroyed { unit: UnitId(23), attacker: None },
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
}
