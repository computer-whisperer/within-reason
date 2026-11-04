//! The lane tests' host: a `View` of seven unit types under the game's names (the stats are a fixture's, not the
//! game's: the light laser turret does 80 a second here and 160 in the game, the Rover 40 against 35) and the
//! fixtures the rover's (`rove.rs`) and the group's (`group.rs`) lane-level tests share.

use bot_protocol::{Command, EnemyUnit, MoveClass, MoveKind, OwnUnit, Snapshot, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

use crate::rove::RoveGoal;
use crate::{GroupKind, GroupOrder, Lane, Stats, View};

pub(crate) const ROVER: UnitDefId = UnitDefId(1);

pub(crate) const PAWN: UnitDefId = UnitDefId(2);

pub(crate) const TOWER: UnitDefId = UnitDefId(3);

pub(crate) const EXTRACTOR: UnitDefId = UnitDefId(4);

pub(crate) const CONSTRUCTOR: UnitDefId = UnitDefId(5);

pub(crate) const BANSHEE: UnitDefId = UnitDefId(6);

pub(crate) const THISTLE: UnitDefId = UnitDefId(7);

pub(crate) fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

pub(crate) fn def(id: UnitDefId, name: &str, speed: f32, weapons: i32, build_speed: f32, extracts: f32) -> UnitDefInfo {
    UnitDefInfo {
        id,
        name: name.to_string(),
        metal_cost: 50.0,
        energy_cost: 0.0,
        speed,
        build_speed,
        build_time: 1.0,
        build_distance: 0.0,
        extracts_metal: extracts,
        metal_make: 0.0,
        energy_make: 0.0,
        energy_upkeep: 0.0,
        wind_cap: 0.0,
        tidal_make: 0.0,
        sonar_range: 0.0,
        hits_submerged: false,
        water_only: false,
        metal_storage: 0.0,
        energy_storage: 0.0,
        radar_range: 0.0,
        converter: None,
        weapon_count: weapons,
        build_options: Vec::new(),
        move_class: (speed > 0.0).then_some(MoveClass { kind: MoveKind::Tank, max_slope: 0.5, depth: 20.0, slope_mod: 0.0 }),
        footprint: (2, 2),
        max_height_dif: 0.0,
        min_water_depth: 0.0,
        max_water_depth: 0.0,
        floater: false,
        death_blast: None,
        self_destruct_blast: None,
        self_destruct_seconds: 0.0,
        reach: 0.0,
        reload: 0.0,
        blast_radius: 0.0,
        stockpile_seconds: 0.0,
        stockpile_energy: 0.0,
        coverage: 0.0,
        sight: 0.0,
        jam_range: 0.0,
        stealth: false,
        cloak: None,
        shield: None,
        intercepted_by: 0,
        health: 0.0,
        reclaim_speed: 0.0,
        reclaimable: false,
        stun_taken: 1.0,
        stun: None,
    }
}

pub(crate) struct Host {
    pub(crate) defs: Vec<UnitDefInfo>,
    pub(crate) buildings: Vec<(UnitId, UnitDefId, Vec3)>,
    pub(crate) goals: Vec<RoveGoal>,
    /// The map's terrain, when a test wants ground (the needle routes over it); none by default.
    pub(crate) terrain: Option<bot_protocol::Terrain>,
}

/// A flat map the test host's grid spec names: 256 cells of 16 a side, 4,096 elmos.
pub(crate) fn flat_terrain() -> bot_protocol::Terrain {
    let (w, h) = (256usize, 256usize);
    bot_protocol::Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes: vec![0u8; w * h], metal: Vec::new() }
}

impl Host {
    pub(crate) fn new(goals: Vec<RoveGoal>) -> Host {
        let defs = vec![
            def(ROVER, "armfav", 168.0, 1, 0.0, 0.0),
            def(PAWN, "armpw", 87.0, 1, 0.0, 0.0),
            def(TOWER, "armllt", 0.0, 1, 0.0, 0.0),
            // Its death blast reaches 120 (a stand keeps outside it).
            UnitDefInfo { death_blast: Some(bot_protocol::Blast { radius: 120.0, damage: 50.0 }), ..def(EXTRACTOR, "armmex", 0.0, 0, 0.0, 0.002) },
            def(CONSTRUCTOR, "armck", 36.0, 0, 80.0, 0.0),
            UnitDefInfo { move_class: None, ..def(BANSHEE, "armkam", 185.0, 1, 0.0, 0.0) },
            def(THISTLE, "corrl", 0.0, 1, 0.0, 0.0),
        ];
        Host { defs, buildings: Vec::new(), goals, terrain: None }
    }
}

impl View for Host {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.defs.iter().find(|d| d.id == def)
    }
    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        Some(match def {
            ROVER => Stats { reach: 180.0, dps: 40.0, speed: 168.0, health: 105.0, dgun: 0.0, sight: 635.0, area: 0.0, falloff: 0.5, ..Stats::default() },
            PAWN => Stats { reach: 180.0, dps: 60.0, speed: 87.0, health: 370.0, dgun: 0.0, sight: 450.0, area: 0.0, ..Stats::default() },
            TOWER => Stats { reach: 430.0, dps: 80.0, speed: 0.0, health: 700.0, dgun: 0.0, sight: 500.0, area: 0.0, air_reach: 430.0, air_dps: 10.7, falloff: 0.5 },
            BANSHEE => Stats { reach: 350.0, dps: 38.6, speed: 185.0, health: 560.0, sight: 520.0, ..Stats::default() },
            THISTLE => Stats { health: 335.0, sight: 500.0, air_reach: 765.0, air_dps: 67.6, ..Stats::default() },
            EXTRACTOR => Stats { health: 300.0, ..Stats::default() },
            CONSTRUCTOR => Stats { speed: 36.0, health: 760.0, ..Stats::default() },
            _ => return None,
        })
    }
    fn grid_spec(&self) -> (f32, usize, usize) {
        (16.0, 256, 256)
    }
    fn terrain(&self) -> Option<&bot_protocol::Terrain> {
        self.terrain.as_ref()
    }
    fn home(&self) -> Vec3 {
        at(200.0, 200.0)
    }
    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        self.buildings.clone()
    }
    fn building_at(&self, id: UnitId) -> Option<Vec3> {
        self.buildings.iter().find(|b| b.0 == id).map(|b| b.2)
    }
    fn enemy_known(&self, _id: UnitId) -> bool {
        true
    }
    fn blip_def(&self) -> Option<UnitDefId> {
        None
    }
    fn enabled(&self, _rule: &str) -> bool {
        true
    }
    fn mine(&self, _unit: UnitId) -> bool {
        true
    }
    fn label(&self) -> String {
        "[test]".to_string()
    }
    fn rove_goals(&self, _def: UnitDefId) -> Vec<RoveGoal> {
        self.goals.clone()
    }
}

pub(crate) fn goal(name: &str, x: f32, z: f32, seen: Option<i32>, theirs: bool) -> RoveGoal {
    RoveGoal { name: name.to_string(), at: at(x, z), seen, theirs }
}

pub(crate) fn rover(id: i32, x: f32, z: f32) -> OwnUnit {
    OwnUnit { id: UnitId(id), def: ROVER, pos: at(x, z), vel: at(0.0, 0.0), health: 105.0, max_health: 105.0, being_built: false, idle: false, reload_frame: 0, facing: 0, stockpile: 0, stockpile_queued: 0, priority_low: false, build_progress: 1.0, stunned: 0.0 }
}

pub(crate) fn enemy(id: i32, def: UnitDefId, x: f32, z: f32) -> EnemyUnit {
    EnemyUnit { id: UnitId(id), def: Some(def), pos: at(x, z), vel: at(0.0, 0.0), health: 300.0, team: Some(1), being_built: false, stunned: 0.0 }
}

pub(crate) fn tick(frame: i32, own: Vec<OwnUnit>, enemies: Vec<EnemyUnit>) -> Tick {
    Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { own_units: own, enemies, ..Snapshot::default() } }
}

/// A lane with these units declared as one roving group (design D15).
pub(crate) fn roving_lane(ids: &[i32]) -> Lane {
    let mut lane = Lane::default();
    lane.set_group("scouts", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Rove { within: None }, Vec::new()));
    lane
}

pub(crate) fn moves_of(commands: &[Command], unit: i32) -> Vec<Vec3> {
    commands.iter().filter_map(|c| match c { Command::Move { unit: u, to, .. } if u.0 == unit => Some(*to), _ => None }).collect()
}
