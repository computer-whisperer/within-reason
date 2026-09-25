//! The brain's side of the control lane (`crates/micro`, `docs/design/2026-09-20-micro-lane.md`,
//! `docs/design/2026-09-25-formation-micro.md`): what the brain tells the lane (standing orders, commitments, the
//! footwork rules of the pianist's groups) and what the lane reads of the brain (`BrainView`).

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{Command, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use combatsim::sim::Rules;
pub use micro::{Commitment, Footwork, Lane, Stats, View};

/// The enemy commander's D-gun is not in the simulator (nobody presses the button there) and it kills a Pawn a shot:
/// a party smaller than this much metal does not fight within reach of the commander (rush-smoke2: four Pawns dead to
/// it in six seconds); a second player's twelve Pawns killed BARb's.
pub(super) const COMMANDER_PARTY_METAL: f32 = 450.0;

/// The combat simulator's tables, and its unit type for each of the game's (filled on first use).
pub(super) struct Sim {
    pub(super) rules: Arc<Rules>,
    pub(super) defs: HashMap<UnitDefId, usize>,
}

impl Default for Sim {
    fn default() -> Self {
        // Their commander presses its D-gun (K-barb-commander-dgun-beats-a-pawn-party); ours never does in a scene.
        let tuning = combatsim::sim::Tuning { dgun: true, ..Default::default() };
        Sim { rules: Arc::new(Rules::new(combatsim::units::Units::default(), tuning)), defs: HashMap::new() }
    }
}

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

    /// What each soldier is priced against in the lane: the pianist's groups' commitments and footwork.
    pub(super) fn note_commitments(&mut self) {
        let mut commitment: HashMap<UnitId, Commitment> = HashMap::new();
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
                    super::pianist::GroupTask::Engage { .. } => Commitment::Priced { turrets: Vec::new(), commander: metal >= COMMANDER_PARTY_METAL },
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

    /// Every tick: the lane over the brain's view, over the brain's own orders of this tick (`commands`, which
    /// it may rewrite); its firings go to the journal; its own commands are appended.
    pub(super) fn micro(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        if self.kit.is_none() {
            return;
        }
        if self.sim.defs.is_empty() {
            self.survey_sim_defs();
        }
        let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
        let mut lane = std::mem::take(&mut self.lane);
        let output = lane.tick(&BrainView { brain: self }, tick, commands, debug);
        self.lane = lane;
        for rule in output.fired {
            self.fire(rule);
        }
        self.journal.milling += output.milling;
        commands.extend(output.commands);
    }

    pub(super) fn survey_sim_defs(&mut self) {
        let rules = self.sim.rules.clone();
        // Every definition the table has, aircraft included; a type it lacks gets the glossary's numbers from
        // `sim_stats` rather than a stand-in by metal (docs/design/2026-09-22-domains.md, decision 7).
        for def in &self.world.hello.unit_defs {
            if let Some(index) = rules.units.index(&def.name) {
                self.sim.defs.insert(def.id, index);
            }
        }
    }

    /// A type's reach, damage a second and speed against ground: the simulator's table, else the glossary's numbers
    /// for a type the table lacks (never a stand-in by metal).
    pub(super) fn sim_stats(&self, def: UnitDefId) -> Option<(f32, f32, f32)> {
        if let Some(i) = self.sim.defs.get(&def) {
            let unit = &self.sim.rules.units.list[*i];
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
        let (health, dgun) = match self.brain.sim.defs.get(&def) {
            Some(i) => {
                let unit = &self.brain.sim.rules.units.list[*i];
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
