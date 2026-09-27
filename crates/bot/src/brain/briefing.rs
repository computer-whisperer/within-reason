//! The brain's side of the player's session: the briefing, the field, events and triggers out.

use std::collections::BTreeMap;

use bot_protocol::{UnitId, Event, OwnUnit, Tick, UnitDefId, Vec3};
use serde_json::json;

use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};
use crate::strategist::shared::{Briefing, Counts, EnemyCluster, ExtractorStatus, Field, Group, Place, RememberedBuilding, Score};

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

impl Brain {
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

    /// Notes our own losses by name, and wakes the strategist when extractors go down in numbers.
    pub(super) fn track_losses(&mut self, tick: &Tick, kit: &Kit) {
        const LOSSES_WORTH_WAKING_FOR: usize = 2;
        self.abandoned_now.clear();
        for enemy in &tick.snapshot.enemies {
            if let Some(def) = enemy.def {
                self.enemy_defs.insert(enemy.id, def);
            }
        }
        for event in &tick.events {
            if let Event::EnemyDestroyed { enemy } = event {
                let def = self.enemy_defs.remove(enemy);
                let worth = def.and_then(|d| self.world.def(d)).map_or(0.0, |d| d.metal_cost);
                self.trade_log.push((tick.frame, 0.0, worth));
                let name = def.map_or("unseen", |d| self.name(d));
                let line = format!("killed {name}");
                if let Some(shared) = &self.strategist {
                    *shared.fights.lock().unwrap().entry(line.clone()).or_default() += 1;
                }
                *self.fight_ledger.entry(line).or_default() += 1;
            }
            let Event::UnitDestroyed { unit, attacker } = event else { continue };
            let Some((def, pos)) = self.known_units.remove(unit) else { continue };
            let cost = self.world.def(def).map_or(0.0, |d| d.metal_cost);
            // A nanoframe its builder walked away from decays: what was put in is lost, and nothing killed it
            // (pianist-player-1: two labs and fourteen generators "lost to something unseen at home", read as air).
            if let Some(share) = self.abandoned(*unit, *attacker) {
                self.abandoned_now.insert(*unit, share);
                self.trade_log.push((tick.frame, cost * share, 0.0));
                let line = format!("abandoned an unfinished {} ({:.0}% built)", self.name(def), share * 100.0);
                if let Some(shared) = &self.strategist {
                    *shared.fights.lock().unwrap().entry(line.clone()).or_default() += 1;
                }
                *self.fight_ledger.entry(line).or_default() += 1;
                let (name, grid) = (self.name(def).to_string(), self.world.grid(pos));
                self.event(tick.frame, format!("abandoned an unfinished {name} at {grid}"));
                continue;
            }
            self.trade_log.push((tick.frame, cost, 0.0));
            self.unit_losses.push_back((tick.frame, *unit, def));
            if kit.is_extractor(def)
                && let Some(index) = self.world.hello.metal_spots.iter().position(|s| s.dist2d(pos) < self.spot_occupied_radius())
            {
                *self.spot_losses.entry(index).or_default() += 1;
            }
            let killer = attacker.and_then(|id| self.enemy_defs.get(&id)).map_or("unseen", |d| self.name(*d));
            let place = if pos.dist2d(self.home) < BASE_RADIUS { "at home" } else if pos.dist2d(self.home) < pos.dist2d(self.world.mirrored(self.home)) { "in our half" } else { "in their half" };
            let line = format!("lost {} to {killer} {place}", self.name(def));
            if let Some(shared) = &self.strategist {
                *shared.fights.lock().unwrap().entry(line.clone()).or_default() += 1;
            }
            *self.fight_ledger.entry(line).or_default() += 1;
            if self.world.def(def).is_some_and(|d| d.speed > 0.0 && d.build_speed == 0.0) {
                continue; // soldiers die all the time
            }
            let (name, grid) = (self.name(def).to_string(), self.world.grid(pos));
            self.event(tick.frame, format!("lost {name} at {grid}"));
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
        self.unfinished = tick.snapshot.own_units.iter().filter(|u| u.being_built).map(|u| (u.id, (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0))).collect();
    }

    /// How far along a destroyed unit of ours was if it was an unfinished nanoframe nobody attacked: the builder left
    /// it and it decayed. `None` for a finished unit or one with a known attacker.
    pub(super) fn abandoned(&self, unit: UnitId, attacker: Option<UnitId>) -> Option<f32> {
        if attacker.is_some() {
            return None;
        }
        self.abandoned_now.get(&unit).or_else(|| self.unfinished.get(&unit)).copied()
    }

    pub(super) fn track_enemy_buildings(&mut self, tick: &Tick) {
        let mut gone = std::mem::take(&mut self.razed);
        let mut seen = Vec::new();
        for event in &tick.events {
            if let Event::EnemyDestroyed { enemy } = event {
                if let Some((def, pos, _)) = self.enemy_buildings.remove(enemy)
                    && self.world.is_factory_def(def)
                {
                    self.enemy_factories_gone.push((def, pos, tick.frame));
                }
                self.enemy_soldiers.remove(enemy);
                gone.push(*enemy);
            }
        }
        for enemy in &tick.snapshot.enemies {
            let Some(def) = enemy.def else { continue };
            let Some(info) = self.world.def(def) else { continue };
            if self.is_commander_def(def) {
                self.enemy_commander_seen = Some((enemy.pos, tick.frame));
            }
            if info.speed == 0.0 {
                self.enemy_buildings.insert(enemy.id, (def, enemy.pos, tick.frame));
                seen.push((enemy.id, (def, enemy.pos, tick.frame)));
            } else if info.weapon_count > 0 && info.build_speed == 0.0 {
                self.enemy_soldiers.insert(enemy.id, (def, enemy.pos, tick.frame));
            }
        }
        // H-TEAM-BOARD: what one seat of ours has seen, all know.
        if self.enabled("H-TEAM-BOARD") {
            self.enemy_buildings = self.board.pool_buildings(&seen, &gone);
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
        if tick.frame <= super::TICK_FRAMES_HINT {
            *shared.map.lock().unwrap() = self.map_description();
        }
        let count = |def: UnitDefId| snapshot.own_units.iter().filter(|u| u.def == def).count();
        let soldiers: Vec<&OwnUnit> = snapshot.own_units.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();

        let mut cells: BTreeMap<String, (Vec3, BTreeMap<&str, usize>, usize, Vec<UnitId>)> = BTreeMap::new();
        for enemy in &snapshot.enemies {
            let cell = cells.entry(self.world.grid(enemy.pos)).or_insert((enemy.pos, BTreeMap::new(), 0, Vec::new()));
            *cell.1.entry(enemy.def.map_or("unidentified", |def| self.name(def))).or_default() += 1;
            cell.2 += 1;
            cell.3.push(enemy.id);
        }
        let enemies_visible = cells
            .into_values()
            .map(|(pos, composition, units, ids)| EnemyCluster {
                at: self.place(pos),
                units,
                composition: composition.into_iter().map(|(name, n)| (name.to_string(), n)).collect(),
                distance_from_home: pos.dist2d(self.home) as i32,
                killing: self.killing_words(&ids, None),
            })
            .collect();
        let mut enemy_buildings_remembered: Vec<RememberedBuilding> = self
            .enemy_buildings
            .values()
            .map(|(def, pos, seen)| RememberedBuilding { name: self.name(*def).to_string(), at: self.place(*pos), last_seen: clock(*seen) })
            .collect();
        enemy_buildings_remembered.sort_by(|a, b| a.at.grid.cmp(&b.at.grid).then(a.name.cmp(&b.name)));

        let sides = self.sides();
        let briefing = Briefing {
            sides,
            seats: Vec::new(),
            game_time: clock(tick.frame),
            frame: tick.frame,
            metal: snapshot.metal,
            energy: snapshot.energy,
            wind: snapshot.wind,
            wind_range: (self.world.hello.map.wind_min, self.world.hello.map.wind_max),
            counts: Counts {
                extractors: count(kit.extractor) + count(kit.advanced_extractor),
                generators: count(kit.solar) + count(kit.wind) + count(kit.advanced_solar),
                converters: count(kit.converter),
                labs: count(kit.lab) + count(kit.plant),
                turrets: count(kit.turret),
                constructors: count(kit.constructor) + count(kit.vehicle_constructor),
                army: soldiers.len(),
            },
            home: self.place(self.home),
            home_group: self.group(&soldiers),
            enemies_visible,
            enemy_buildings_remembered,
            recent_events: self.recent_events.iter().cloned().collect(),
        };
        shared.publish_briefing(self.world.hello.team, self.home, briefing);
    }

    /// The sea: how much of the map is under water and what of ours can cross it (the user, on pianist-player-7: the
    /// map is an island, the enemy commander was out in the ocean building, and the player had never been told).
    fn water_description(&self) -> serde_json::Value {
        let terrain = &self.world.hello.terrain;
        let under = terrain.heights.iter().filter(|h| **h < 0).count();
        let share = if terrain.heights.is_empty() { 0.0 } else { under as f32 / terrain.heights.len() as f32 * 100.0 };
        let lab_builds: Vec<UnitDefId> = self.kit.and_then(|k| self.world.def(k.lab)).map(|d| d.build_options.clone()).unwrap_or_default();
        let side = self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).map(|t| t.side.to_lowercase()).unwrap_or_default();
        let crosses = |d: &bot_protocol::UnitDefInfo| d.move_class.is_some_and(|m| matches!(m.kind, bot_protocol::MoveKind::Hover) || m.depth >= 1000.0) && d.speed > 0.0;
        let ours: Vec<String> = self
            .world
            .hello
            .unit_defs
            .iter()
            .filter(|d| crosses(d) && d.name.starts_with(&side[..3.min(side.len())]) && (d.weapon_count > 0 || d.build_speed > 0.0))
            .map(|d| format!("{} ({}{})", d.name, if d.weapon_count > 0 { "armed" } else { "a builder" }, if lab_builds.contains(&d.id) { ", from the bot lab" } else { "" }))
            .collect();
        json!({
            "share": format!("{share:.0}% of the map is under water (the sketch's ~)"),
            "note": "Our bots and vehicles stop at the shore; a spot or a place in the water cannot be reached and an advance toward it stalls. The commander is amphibious: it walks on the sea floor, and so does the enemy's, which can retreat into the sea when its base is gone and build on the shore from the water. When it does, nothing on your hands' menu today reaches it (only the bot lab and the advanced bot lab can be built; the plants that make amphibians are not offered), and the referee ends the game once its economy is gone.",
            "amphibious_of_ours": ours,
        })
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
                json!({ "n": n, "grid": self.world.grid(*s), "x": s.x as i32, "z": s.z as i32, "walk_from_home": walk })
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
            "start_boxes_note": "the lobby's start boxes: each team's commander was placed somewhere inside its box at 0:00. The engine tells nobody where; where the opponent stands now is known only from what our units see",
            "metal_spots": spots,
            "metal_spots_note": "n is the spot's number for the `expansion` tool; walk_from_home is the walking distance for our bots; null means they cannot walk there",
            "terrain": self.terrain_sketch(),
            "water": self.water_description(),
            "passages": self.passages().iter().map(|p| json!({ "at": self.place(p.at), "width": p.width as i32, "share_of_the_way_from_our_start": (p.along * 100.0) as i32 })).collect::<Vec<_>>(),
            "passages_note": "narrow places every walking route between our start and the opponent's goes through (cliffs or water on both sides), the narrowest first: whoever holds one decides who crosses, and soldiers and turrets there cover everything behind them",
        })
    }
}

impl Brain {
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
                let seats = hello
                    .teams
                    .iter()
                    .filter(|t| t.ally_team == ally_team && t.controller != Controller::Gaia)
                    .map(|t| {
                        let who = match &t.controller {
                            _ if t.team == hello.team => "you (WReason)".to_string(),
                            Controller::Person { name, skill: Some(skill) } => format!("{name} (a person, lobby skill {skill:.0})"),
                            Controller::Person { name, skill: None } => format!("{name} (a person)"),
                            Controller::Ai { short_name, profile: Some(profile), .. } if short_name.eq_ignore_ascii_case("wreason") => format!("Within Reason {profile} (another seat of this bot)"),
                            Controller::Ai { short_name, .. } if short_name.eq_ignore_ascii_case("wreason") => "Within Reason (another seat of this bot)".to_string(),
                            Controller::Ai { short_name, profile: Some(profile), .. } => format!("{short_name} {profile} (an AI)"),
                            Controller::Ai { short_name, .. } => format!("{short_name} (an AI)"),
                            Controller::Gaia => unreachable!("filtered above"),
                            Controller::Unknown => format!("team {} (who plays it is not in the script)", t.team),
                        };
                        // The engine lowercases faction names.
                        let mut side = t.side.chars();
                        let side = side.next().map(|c| c.to_ascii_uppercase().to_string() + side.as_str()).unwrap_or_default();
                        if side.is_empty() { who } else { format!("{who}, {side}") }
                    })
                    .collect();
                crate::strategist::shared::Side { ours: ally_team == hello.ally_team, ally_team, seats }
            })
            .collect()
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
        let enemy_extractors: Vec<Vec3> = self
            .enemy_buildings
            .values()
            .filter(|(def, _, _)| self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0))
            .map(|(_, pos, _)| *pos)
            .collect();
        let our_extractors: Vec<Vec3> = own.iter().filter(|u| kit.is_extractor(u.def)).map(|u| u.pos).collect();
        let held = |spot: Vec3, by: &[Vec3]| by.iter().any(|p| p.dist2d(spot) < self.spot_occupied_radius());
        let free: Vec<Vec3> = self
            .world
            .hello
            .metal_spots
            .iter()
            .filter(|s| self.reachable_on_foot(**s))
            .filter(|s| !held(**s, &our_extractors) && !held(**s, &enemy_extractors) && !self.allied_extractor_on(**s))
            .copied()
            .collect();
        let recent = |seen: &i32| tick.frame - seen < 3 * 60 * FRAMES_PER_SECOND;
        let metal = |u: &&OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.metal_cost);
        let traded = |when: &dyn Fn(i32) -> bool| {
            let sum = |pick: &dyn Fn(&(i32, f32, f32)) -> f32| self.trade_log.iter().filter(|t| when(t.0)).map(pick).sum::<f32>() as u32;
            (sum(&|t| t.1), sum(&|t| t.2))
        };
        let score = Score {
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
            metal_income: tick.snapshot.metal.income,
            trend: [3, 6].into_iter().filter_map(|m| self.minutes_ago(tick.frame, m).map(|(x, income, army)| (m, x, income, army))).collect(),
            extractors_lost_3_min: self.wake.losses.len(),
            traded_3_min: traded(&|frame| recent(&frame)),
            traded: traded(&|_| true),
            seconds_since_turn: Some(shared.last_turn_frame.load(std::sync::atomic::Ordering::Relaxed)).filter(|at| *at > 0).map(|at| (tick.frame - at) / FRAMES_PER_SECOND),
            enemy_start_boxes: self.world.hello.start_boxes.iter().filter(|b| b.ally_team != self.world.hello.ally_team).map(|b| self.world.box_cells(b)).collect(),
            never_looked: {
                let boxes: Vec<&bot_protocol::StartBox> = self.world.hello.start_boxes.iter().filter(|b| b.ally_team != self.world.hello.ally_team).collect();
                let mut never: Vec<(f32, usize)> = self.world.hello.metal_spots.iter().enumerate().filter(|(i, _)| self.spot_seen(*i).is_none()).map(|(i, s)| (s.dist2d(self.home), i)).collect();
                never.sort_by(|a, b| a.0.total_cmp(&b.0));
                never.into_iter().map(|(_, i)| { let s = self.world.hello.metal_spots[i]; (i, self.place(s), boxes.iter().any(|b| b.contains(s))) }).collect()
            },
            enemy_spots_seen: enemy_extractors.len(),
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
            factions: vec![(self.world.hello.team, self.world.hello.teams.iter().find(|t| t.team == self.world.hello.team).map_or(String::from("?"), |t| t.side.clone()))],
        });
    }
}
