//! Test fixtures shared by the pianist's tests: a brain over the simulator's unit table, units, and the E3 scene of
//! player-14 at 9:17 as the TAS study cut it (fourteen Blitzes south-west of three Centurions and two light turrets).
#![cfg(test)]

use bot_protocol::{EnemyUnit, Hello, MapInfo, MoveClass, MoveKind, OwnUnit, Terrain, UnitDefId, UnitDefInfo, UnitId, Vec3};

use super::super::Brain;
use super::picture::Party;


/// A brain over the named types of the simulator's table (no terrain: every point is reachable).
pub(crate) fn brain_of(names: &[&str]) -> Brain {
    let units = combatsim::units::Units::default();
    let unit_defs: Vec<UnitDefInfo> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let u = &units.list[units.index(name).expect(name)];
            UnitDefInfo {
                id: UnitDefId(i as i32 + 1),
                name: name.to_string(),
                metal_cost: u.metal,
                energy_cost: u.energy,
                speed: u.speed,
                build_speed: 0.0,
                blast_radius: 0.0,
                build_time: 1.0,
                build_distance: 0.0,
                extracts_metal: 0.0,
                metal_make: 0.0,
                energy_make: 0.0,
                energy_upkeep: 0.0,
                wind_cap: 0.0,
                tidal_make: 0.0,
                metal_storage: 0.0,
                energy_storage: 0.0,
                sonar_range: 0.0,
                hits_submerged: false,
                water_only: false,
                radar_range: 0.0,
                converter: None,
                weapon_count: u.weapons.len() as i32,
                build_options: Vec::new(),
                move_class: u.mobile().then_some(MoveClass { kind: MoveKind::Tank, max_slope: 0.1, depth: 20.0, slope_mod: 0.0 }),
                footprint: (2, 2),
                death_blast: None,
                self_destruct_blast: None,
                self_destruct_seconds: 5.0,
                reach: u.reach(),
                reload: 1.0,
            }
        })
        .collect();
    let hello = Hello {
        ai_id: 0,
        team: 0,
        ally_team: 0,
        game_id: 0,
        teams: Vec::new(),
        start_boxes: Vec::new(),
        start_pos_type: None,
        script: String::new(),
        frame: 0,
        tick_frames: 3,
        map: MapInfo { name: "test".into(), width: 8192.0, height: 6144.0, wind_min: 0.0, wind_max: 0.0, tidal: 0.0, extractor_radius: 120.0 },
        unit_defs,
        metal_spots: Vec::new(),
        metal_spot_squares: Vec::new(),
        terrain: Terrain { cell: 16.0, width: 0, height: 0, heights: Vec::new(), slopes: Vec::new(), metal: Vec::new() },
    };
    let mut brain = Brain::new(crate::world::World::new(hello), None, std::sync::Arc::new(crate::team::TeamBoard::default()), String::new(), None);
    brain.survey_sim_defs();
    brain
}

pub(crate) fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

pub(crate) fn own(id: i32, def: i32, pos: Vec3) -> OwnUnit {
    OwnUnit { id: UnitId(id), def: UnitDefId(def), pos, vel: Vec3::default(), health: 1.0, max_health: 1.0, being_built: false, idle: false, reload_frame: 0, facing: 0 }
}

pub(crate) fn enemy(id: i32, def: i32, pos: Vec3) -> EnemyUnit {
    EnemyUnit { id: UnitId(id), def: Some(UnitDefId(def)), pos, vel: Vec3::default(), health: 1000.0, team: Some(1), being_built: false }
}

/// The E3 scene of player-14 at 9:17, as the TAS study cut it: fourteen Blitzes south-west of three Centurions
/// at a rock's south edge and two light turrets 400-600 east of them.
pub(crate) fn e3() -> (Brain, Vec<OwnUnit>, Vec<EnemyUnit>, Vec<Party>) {
    let mut brain = brain_of(&["armflash", "armwar", "armllt"]);
    let ours: Vec<OwnUnit> = (0..14).map(|i| own(100 + i, 1, at(4150.0 + 20.0 * (i % 7) as f32, 1900.0 + 40.0 * (i / 7) as f32))).collect();
    let centurions = vec![enemy(1, 2, at(4404.0, 1228.0)), enemy(2, 2, at(4576.0, 1430.0)), enemy(3, 2, at(4527.0, 1436.0))];
    let turrets = [(10, at(4912.0, 1632.0)), (11, at(5024.0, 1616.0))];
    for (id, pos) in turrets {
        brain.enemy_buildings.insert(UnitId(id), (UnitDefId(3), pos, 0));
    }
    let mut enemies = centurions.clone();
    enemies.extend(turrets.iter().map(|(id, pos)| enemy(*id, 3, *pos)));
    let party = Party { name: "party_12".into(), ids: centurions.iter().map(|e| e.id).collect(), at: at(4502.0, 1365.0), metal: 810.0, composition: "3 armwar".into(), has_commander: false, harming: None, unarmed: false, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() };
    (brain, ours, enemies, vec![party])
}
