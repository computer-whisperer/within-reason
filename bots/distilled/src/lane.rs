//! D-LANE: the control lane under the army's orders. The shared `crates/micro` lane (the flagship's hands' footwork
//! and group control: flee from what outguns a unit, fan under a D-gun, kite, and the formations of the drills of
//! 2026-10-24: ranks, companies, the advance, the facing, the stand) runs every tick over the soldiers' standing
//! orders, which the army rules give at one hertz (`army.rs`), and walks the strike body as a declared formation
//! (`declare_groups`). The lane never re-prices a fight; it spends a unit's order on staying alive, shooting well and
//! standing in its slot. What it reads of this bot is `LaneView`; the fighting numbers come from the combat
//! simulator's table (`crates/combatsim/data/units.json`), as the flagship's do. On by default since 2026-11-03
//! (`lane=off` for a row without it): with the strike body a formation the bot went 7-2-7 against Apex v0.5.04 on
//! Comet where the loose body went 2-4-10 (dist66, dist59), and 8-4-4 against BARb hard_aggressive for 8-2-6.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use bot_protocol::{Command, Terrain, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use combatsim::sim::Rules;
use micro::{Commitment, GroupKind, GroupOrder, Lane, Stats, View};

use crate::army::Stance;
use crate::Distilled;

/// The lane, the simulator's tables and the table's row for each of the game's unit types (filled at the start).
pub(crate) struct Control {
    pub(crate) lane: Lane,
    pub(crate) rules: Arc<Rules>,
    pub(crate) rows: HashMap<UnitDefId, usize>,
}

impl Default for Control {
    fn default() -> Self {
        // Their commander presses its D-gun; ours never does in a scene (the flagship's tuning).
        let tuning = combatsim::sim::Tuning { dgun: true, ..Default::default() };
        Control { lane: Lane::default(), rules: Arc::new(Rules::new(combatsim::units::Units::default(), tuning)), rows: HashMap::new() }
    }
}

impl Control {
    /// The table's row for each of the game's unit types (at the start; the lane and D-REACH read them).
    pub(crate) fn survey(&mut self, defs: &BTreeMap<UnitDefId, UnitDefInfo>) {
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
        Some(Stats { reach: unit.reach(), dps: unit.dps(), speed: unit.speed, health: unit.health, dgun, sight: unit.sight, area: self.bot.def(def).map_or(0.0, |d| d.blast_radius), air_reach: unit.reach_air(), air_dps: unit.dps_air(), falloff: unit.falloff(), aim: unit.mid, top: unit.top, radius: unit.radius })
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

    fn terrain(&self) -> Option<&Terrain> {
        let terrain = &self.bot.hello.terrain;
        (terrain.width > 0 && terrain.cell > 0.0).then_some(terrain)
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

    /// The simulator's verdict on the units as they stand (H-MICRO-PUSH); the raiders do not rove yet, so unasked.
    fn odds(&self, ours: &[(UnitDefId, Vec3)], theirs: &[(UnitDefId, Vec3)]) -> Option<(f32, f32)> {
        let rows = &self.bot.control.rows;
        let placed = |side: &[(UnitDefId, Vec3)]| side.iter().map(|(def, at)| Some((*rows.get(def)?, combatsim::scenario::Vec2::new(at.x, at.z)))).collect::<Option<Vec<_>>>();
        Some(combatsim::scenario::odds_placed(&self.bot.control.rules, &placed(ours)?, &placed(theirs)?, 8))
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
        let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
        let mut lane = std::mem::take(&mut self.control.lane);
        let soldier = |unit: UnitId| self.own.get(&unit).and_then(|u| self.def(u.def)).is_some_and(micro::is_soldier);
        lane.note_standing_orders(commands, self.frame, soldier);
        let output = lane.tick(&LaneView { bot: self }, tick, commands, debug);
        self.control.lane = lane;
        commands.extend(output.commands);
    }

    /// D-LANE: the strike body is a formation of the lane's (the flagship's group control: ranks, the advance that
    /// halts with every member in reach and fires on the move, the facing, the stand, the companies): it fights to
    /// its target (`FightTo`) and walks home on a retreat (`Go`); the army's rules choose the places once a second
    /// (`army.rs`), the lane walks them every tick and the stragglers are its exceptions. The guard and the raiders
    /// stay loose on their own Fight orders under the footwork, priced against the soldiers in sight and no turret,
    /// so the lane steps them out of a turret's reach; a scout is under every rule. (A guard declared as a `Hold`
    /// formation stood at the engine's hold position while BARb's raiders stripped the economy round it:
    /// dist63-lane-group-barb seed 2 lost 64 extractors and builders by 14:00 against 14 with the guard loose, the
    /// row 6-8-2 against 8-2-6. Until 2026-11-03 the strike body was loose too, each member committed to everything
    /// and walked on its own Fight order: apex-v029-distilled-comet-10, 1-14-1 against 3-12-1 without the lane.)
    pub(crate) fn declare_groups(&mut self) {
        if !self.options.lane {
            return;
        }
        let hold = self.hold_point();
        let mut lane = std::mem::take(&mut self.control.lane);
        let strike: Vec<UnitId> = self.army.strike.iter().copied().collect();
        if strike.is_empty() {
            lane.clear_group(STRIKE);
        } else {
            // The retreat is marched too: unmarched (every member running to its slot at the hold point at its own
            // speed) the row against Apex went 3-6-7 where the marched retreat's had gone 7-2-7 (dist68, dist66),
            // and against BARb neither moved it (dist67 8-3-5, dist65 8-4-4).
            let (kind, to) = match (self.army.stance(), self.army.strike_target()) {
                (Some(Stance::Retreat(_)), _) => (GroupKind::Go, hold),
                (_, Some((to, _))) => (GroupKind::FightTo, to),
                _ => (GroupKind::Hold, hold),
            };
            lane.set_group(STRIKE, strike, GroupOrder::new(kind, vec![to]).with_cap(COMPANY));
        }
        let loose = self.army.home_guard.iter().chain(&self.army.raiders);
        let commitment: HashMap<UnitId, Commitment> = loose.map(|unit| (*unit, Commitment::Priced { turrets: Vec::new(), commander: false })).collect();
        lane.set_commitments(commitment, HashMap::new());
        self.control.lane = lane;
    }
}

/// The lane's name for the strike body, and the members a company holds at most (the flagship's cap).
pub(crate) const STRIKE: &str = "strike";
const COMPANY: usize = 24;
