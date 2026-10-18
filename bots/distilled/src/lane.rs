//! D-LANE: the control lane under the army's orders. The shared `crates/micro` lane (the flagship's hands' footwork:
//! flee from what outguns a unit, fan under a D-gun, kite, the body's formation at contact) runs every tick over the
//! soldiers' standing orders, which the army rules give at one hertz (`army.rs`). The lane never re-prices a fight;
//! it spends a unit's order on staying alive, shooting well and standing in its slot. What it reads of this bot is
//! `LaneView`; the fighting numbers come from the combat simulator's table (`crates/combatsim/data/units.json`), as
//! the flagship's do. Off unless the arm `lane=on` says so: as wired here (no ground snapped, no cell impassable,
//! the commitments by body) it moved neither the exchange nor the result (apex-v029-distilled-comet-10, 1-14-1 over
//! sixteen cold games against 3-12-1 without it, ours 12-24 dead by 10:00 for his 5-20 either way).

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use bot_protocol::{Command, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use combatsim::sim::Rules;
use micro::{Commitment, Lane, Stats, View};

use crate::Distilled;

/// The lane, the simulator's tables and the table's row for each of the game's unit types (filled at the start).
pub(crate) struct Control {
    lane: Lane,
    rules: Arc<Rules>,
    rows: HashMap<UnitDefId, usize>,
}

impl Default for Control {
    fn default() -> Self {
        // Their commander presses its D-gun; ours never does in a scene (the flagship's tuning).
        let tuning = combatsim::sim::Tuning { dgun: true, ..Default::default() };
        Control { lane: Lane::default(), rules: Arc::new(Rules::new(combatsim::units::Units::default(), tuning)), rows: HashMap::new() }
    }
}

impl Control {
    fn survey(&mut self, defs: &BTreeMap<UnitDefId, UnitDefInfo>) {
        for def in defs.values() {
            if let Some(row) = self.rules.units.index(&def.name) {
                self.rows.insert(def.id, row);
            }
        }
    }
}

/// What the lane reads of the bot.
struct LaneView<'a> {
    bot: &'a Distilled,
}

impl View for LaneView<'_> {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.bot.def(def)
    }

    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        let row = *self.bot.control.rows.get(&def)?;
        let unit = &self.bot.control.rules.units.list[row];
        let dgun = unit.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max);
        Some(Stats { reach: unit.reach(), dps: unit.dps(), speed: unit.speed, health: unit.health, dgun, sight: unit.sight, area: self.bot.def(def).map_or(0.0, |d| d.blast_radius) })
    }

    fn grid_spec(&self) -> (f32, usize, usize) {
        let terrain = &self.bot.hello.terrain;
        if terrain.width > 0 && terrain.cell > 0.0 {
            (terrain.cell, terrain.width as usize, terrain.height as usize)
        } else {
            let map = &self.bot.hello.map;
            (16.0, (map.width / 16.0).ceil() as usize, (map.height / 16.0).ceil() as usize)
        }
    }

    fn passable(&self) -> Option<&[bool]> {
        None
    }

    fn snap(&self, _def: UnitDefId, pos: Vec3) -> Vec3 {
        pos
    }

    fn home(&self) -> Vec3 {
        self.bot.home
    }

    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        self.bot
            .army
            .his_buildings
            .iter()
            .filter_map(|(id, seen)| {
                let def = seen.def?;
                self.bot.def(def).filter(|d| d.weapon_count > 0)?;
                Some((*id, def, seen.pos))
            })
            .collect()
    }

    fn building_at(&self, id: UnitId) -> Option<Vec3> {
        self.bot.army.his_buildings.get(&id).map(|seen| seen.pos)
    }

    fn enemy_known(&self, id: UnitId) -> bool {
        self.bot.army.his_army.contains_key(&id)
    }

    /// A radar contact is taken for his raider, as the army's weights take it.
    fn blip_def(&self) -> Option<UnitDefId> {
        self.bot.kit.raider
    }

    fn enabled(&self, _rule: &str) -> bool {
        true
    }

    fn mine(&self, _unit: UnitId) -> bool {
        true
    }

    fn label(&self) -> String {
        format!("distilled ai={}", self.bot.hello.ai_id)
    }
}

impl Distilled {
    /// Every tick: this tick's orders become the soldiers' standing orders, the lane runs over them (and may
    /// rewrite them), and its own orders follow.
    pub(crate) fn run_lane(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        if !self.options.lane {
            return;
        }
        if self.control.rows.is_empty() {
            let (control, defs) = (&mut self.control, &self.defs);
            control.survey(defs);
        }
        let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
        let mut lane = std::mem::take(&mut self.control.lane);
        let soldier = |unit: UnitId| self.own.get(&unit).and_then(|u| self.def(u.def)).is_some_and(micro::is_soldier);
        lane.note_standing_orders(commands, self.frame, soldier);
        let output = lane.tick(&LaneView { bot: self }, tick, commands, debug);
        self.control.lane = lane;
        commands.extend(output.commands);
    }

    /// What each soldier is priced against (D-LANE, with the army's bodies): the pushing body is committed to
    /// everything, as a wave is; the home guard and a raiding body to the soldiers in sight and no turret, so the
    /// lane steps them out of a turret's reach; a scout to nothing, so it flees every threat.
    pub(crate) fn note_commitments(&mut self) {
        if !self.options.lane {
            return;
        }
        let pushing = !self.raiding(self.strike_weight());
        let mut commitment: HashMap<UnitId, Commitment> = HashMap::new();
        for unit in &self.army.strike {
            commitment.insert(*unit, if pushing { Commitment::All } else { Commitment::Priced { turrets: Vec::new(), commander: false } });
        }
        for unit in &self.army.home_guard {
            commitment.insert(*unit, Commitment::Priced { turrets: Vec::new(), commander: false });
        }
        self.control.lane.set_commitments(commitment, HashMap::new());
    }
}
