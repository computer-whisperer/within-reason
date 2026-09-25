//! The brain's side of the control lane (`crates/micro`, `docs/design/2026-09-20-micro-lane.md`,
//! `docs/design/2026-09-25-formation-micro.md`): what the brain tells the lane (standing orders, commitments, the
//! footwork rules of the pianist's groups) and what the lane reads of the brain (`BrainView`).

use std::collections::HashMap;

use bot_protocol::{Command, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
pub use micro::{Commitment, Footwork, Lane, Stats, View};

use super::Brain;
use crate::world::Domain;

impl Brain {
    /// The brain's orders this tick become the standing orders of the soldiers they went to.
    pub(super) fn note_standing_orders(&mut self, commands: &[Command], frame: i32) {
        let Some(kit) = self.kit else { return };
        let (known, world) = (&self.known_units, &self.world);
        let soldier = |unit: UnitId| {
            known.get(&unit).is_some_and(|(def, _)| *def != kit.commander && world.def(*def).is_some_and(|d| d.speed > 0.0 && d.weapon_count > 0 && d.build_speed == 0.0) && world.domain_of(*def) != Domain::Air)
        };
        self.lane.note_standing_orders(commands, frame, soldier);
    }

    /// What each soldier's group was priced against, from the raid, the contact answers, the waves and the squads.
    pub(super) fn note_commitments(&mut self) {
        let mut commitment: HashMap<UnitId, Commitment> = HashMap::new();
        if self.raid.going() {
            let priced = Commitment::Priced { turrets: self.raid.turrets.clone(), commander: self.raid.fights_commander };
            for id in &self.raid.members {
                commitment.insert(*id, priced.clone());
            }
        }
        // An answer fights the commander only when it is the raid's size for it (the D-gun is not in the simulator;
        // micro-flee-debug: four Pawns answering the commander at our station walked into it and died in six seconds).
        for (members, turrets) in self.response_commitments() {
            let metal: f32 = members.iter().filter_map(|id| self.known_units.get(id)).filter_map(|(def, _)| self.world.def(*def)).map(|d| d.metal_cost).sum();
            let commander = metal >= super::raid::COMMANDER_PARTY_METAL;
            for id in members {
                commitment.insert(id, Commitment::Priced { turrets: turrets.clone(), commander });
            }
        }
        for (id, _) in self.known_units.iter() {
            if self.army.is_attacker(*id) || self.squads.contains(*id) {
                commitment.insert(*id, Commitment::All);
            }
        }
        // The pianist's groups (H-HANDS-GROUPS): one advancing (`fight_to`) is committed to everything, turrets and the
        // commander included, as a wave is: priced against no turret, a ball of eighty stood at the edge of the enemy
        // base's laser towers inside a Guardian's reach for five minutes while Jev said "continue" every ten seconds
        // (pianist-player-2). The hold it arrives in keeps that commitment (pianist-player-4: the ball that fought
        // into the base stood there stepping out of the towers' reach, 35 of 35 held back). A hold Jev chose stands
        // against mobile units and steps out of turret reach; an engaging group fights the commander when it is the
        // raid's size for it (pianist-player-4: 28 of 34 Maces stepped back from the lone enemy commander); one
        // walking without fighting flees everything. Each group's footwork rules come from the player (H-HANDS-LANE).
        let mut footwork: HashMap<UnitId, Footwork> = HashMap::new();
        if let Some(pianist) = &self.pianist {
            for group in &pianist.groups {
                let metal: f32 = group.members.iter().filter_map(|id| self.known_units.get(id)).filter_map(|(def, _)| self.world.def(*def)).map(|d| d.metal_cost).sum();
                let commitment_of = match &group.task {
                    super::pianist::GroupTask::Hold { committed: true, .. } | super::pianist::GroupTask::Move { fight: true, .. } => Commitment::All,
                    super::pianist::GroupTask::Hold { .. } => Commitment::Priced { turrets: Vec::new(), commander: false },
                    super::pianist::GroupTask::Engage { .. } => Commitment::Priced { turrets: Vec::new(), commander: metal >= super::raid::COMMANDER_PARTY_METAL },
                    super::pianist::GroupTask::Move { fight: false, .. } => Commitment::None,
                };
                let rules = self.footwork_of(&group.name);
                for id in &group.members {
                    commitment.insert(*id, commitment_of.clone());
                    footwork.insert(*id, rules);
                }
            }
        }
        self.lane.set_commitments(commitment, footwork);
    }

    /// The player's footwork setting for a pianist group (H-HANDS-LANE): the group's own, else `all`, else every rule.
    pub(super) fn footwork_of(&self, group: &str) -> Footwork {
        let Some(shared) = &self.strategist else { return Footwork::default() };
        let lane = shared.lane.lock().unwrap();
        lane.get(&format!("group_{group}")).or_else(|| lane.get("all")).copied().unwrap_or_default()
    }

    /// Every tick: the lane over the brain's view; its firings go to the journal.
    pub(super) fn micro(&mut self, tick: &Tick) -> Vec<Command> {
        if self.kit.is_none() {
            return Vec::new();
        }
        if self.contacts.sim_defs.is_empty() {
            self.survey_sim_defs();
        }
        let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
        let mut lane = std::mem::take(&mut self.lane);
        let output = lane.tick(&BrainView { brain: self }, tick, debug);
        self.lane = lane;
        for rule in output.fired {
            self.fire(rule);
        }
        self.journal.milling += output.milling;
        output.commands
    }

    /// A type's reach, damage a second and speed against ground: the simulator's table, else the glossary's numbers
    /// for a type the table lacks (never a stand-in by metal).
    pub(super) fn sim_stats(&self, def: UnitDefId) -> Option<(f32, f32, f32)> {
        if let Some(i) = self.contacts.sim_defs.get(&def) {
            let unit = &self.contacts.rules.units.list[*i];
            return Some((unit.reach(), unit.dps(), unit.speed));
        }
        let entry = super::pianist::glossary::entry(self.name(def))?;
        Some((entry.range, entry.dps.unwrap_or(0.0), entry.speed))
    }
}

/// What the lane reads of the brain.
struct BrainView<'a> {
    brain: &'a Brain,
}

impl View for BrainView<'_> {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.brain.world.def(def)
    }

    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        let (reach, dps, speed) = self.brain.sim_stats(def)?;
        let (health, dgun) = match self.brain.contacts.sim_defs.get(&def) {
            Some(i) => {
                let unit = &self.brain.contacts.rules.units.list[*i];
                (unit.health, unit.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max))
            }
            None => (super::pianist::glossary::entry(self.brain.name(def)).map_or(0.0, |e| e.health), 0.0),
        };
        Some(Stats { reach, dps, speed, health, dgun })
    }

    fn grid_spec(&self) -> (f32, usize, usize) {
        let terrain = &self.brain.world.hello.terrain;
        if terrain.width > 0 && terrain.cell > 0.0 {
            (terrain.cell, terrain.width as usize, terrain.height as usize)
        } else {
            let map = &self.brain.world.hello.map;
            (16.0, (map.width / 16.0).ceil() as usize, (map.height / 16.0).ceil() as usize)
        }
    }

    fn passable(&self) -> Option<&[bool]> {
        self.brain.passable()
    }

    fn snap(&self, pos: Vec3) -> Vec3 {
        self.brain.snap_to_reachable(pos)
    }

    fn home(&self) -> Vec3 {
        self.brain.home
    }

    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        self.brain.enemy_buildings.iter().map(|(id, (def, pos, _))| (*id, *def, *pos)).collect()
    }

    fn building_at(&self, id: UnitId) -> Option<Vec3> {
        self.brain.enemy_buildings.get(&id).map(|b| b.1)
    }

    fn enemy_known(&self, id: UnitId) -> bool {
        self.brain.enemy_soldiers.contains_key(&id)
    }

    /// A radar contact is taken for the soldier of theirs we have seen most, as the pricing does.
    fn blip_def(&self) -> Option<UnitDefId> {
        let mut counted: HashMap<UnitDefId, usize> = HashMap::new();
        self.brain.enemy_soldiers.values().for_each(|(def, _, _)| *counted.entry(*def).or_default() += 1);
        counted.into_iter().max_by_key(|(def, n)| (*n, def.0)).map(|(def, _)| def).or(self.brain.kit.as_ref().map(|k| k.line))
    }

    fn enabled(&self, rule: &str) -> bool {
        self.brain.enabled(rule)
    }

    fn mine(&self, _unit: UnitId) -> bool {
        true
    }

    fn label(&self) -> String {
        format!("[ai {}]", self.brain.ai())
    }
}
