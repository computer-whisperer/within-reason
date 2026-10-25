//! T-RACE, an instrument (no decision changed): the race of force per region (`docs/design/2026-10-25-territory-
//! defence.md`, step 2). For each region holding something of ours, how many seconds a raid needs from the edge of
//! our sight to bring its guns onto it, and how much of our armed metal is in reach by then and 15, 30 and 60 s
//! later. Read with the deaths and his units' truth by `run/territory_race.py`.

use std::collections::BTreeMap;
use std::time::Instant;

use bot_protocol::{MoveClass, Sensors, UnitDefId, Vec3};
use buildorder::units::Role;
use terrain::Field;

use crate::{Territory, SECOND};

/// The `race:` line every this many frames.
pub(crate) const RACE_LOG: i32 = 30 * SECOND;
/// The raid of the first form: a Flash's speed and reach (101 and 180; an Incisor's are alike), walked on the
/// commanders' ground.
const RAID_SPEED: f32 = 100.0;
const RAID_REACH: f32 = 180.0;
/// Our armed mobile units this near one another, of one movement class, answer as a group walked by one field.
const GROUP_APART: f32 = 400.0;
/// Our armed metal in reach is logged at his arrival and this many seconds after it.
const AFTER: [f32; 4] = [0.0, 15.0, 30.0, 60.0];

#[derive(Default)]
pub(crate) struct Race {
    /// The engine's sight and radar coverage, the last read (the host reads it once a second).
    pub sensors: Option<Sensors>,
    /// The movement cost layers, by the unit type whose class they were made for.
    costs: BTreeMap<UnitDefId, Vec<u32>>,
}

fn same(a: &MoveClass, b: &MoveClass) -> bool {
    a.kind == b.kind && a.max_slope == b.max_slope && a.depth == b.depth && a.slope_mod == b.slope_mod
}

/// One of our armed units: where, its metal, its reach, its speed (0 for a turret), its type.
struct Armed {
    pos: Vec3,
    metal: f32,
    reach: f32,
    speed: f32,
    def: UnitDefId,
}

impl Territory {
    /// `race:` every RACE_LOG: the coverage, then per region with something of ours standing, `r{k} {metal standing}
    /// warn {seconds} ours {armed metal in reach at the warning and AFTER it} turrets {metal}`. The warning is the raid's
    /// walk from the nearest square neither sight nor radar covers to its reach of the region's nearest building, at
    /// RAID_SPEED; a frame counts at its progress. Our mobile armed units arrive at their own speed over their group's
    /// field, their reach short of that building; a turret counts when a building of the region stands in its reach.
    pub(crate) fn log_race(&mut self) {
        let began = Instant::now();
        let Some(regions) = &self.regions else { return };
        let Some(sensors) = &self.race.sensors else { return };
        let game = self.game.clone().expect("started");
        let terrain = &self.hello.terrain;
        let units = &game.units;
        let of = |p: Vec3| -> usize {
            let near = (0..game.spots.len()).min_by(|a, b| {
                let d = |s: usize| (game.spots[s].0.0 - f64::from(p.x)).hypot(game.spots[s].0.1 - f64::from(p.z));
                d(*a).total_cmp(&d(*b))
            });
            near.map_or(0, |s| regions.of_spot[s])
        };
        let mut standing: Vec<Vec<(Vec3, f32)>> = vec![Vec::new(); regions.list.len()];
        let mut armed: Vec<Armed> = Vec::new();
        for u in self.own.values() {
            let Some(d) = self.def(u.def) else { continue };
            let role = self.unit_index(u.def).map(|i| units.list[i].role);
            if d.speed == 0.0 {
                let worth = if u.being_built { d.metal_cost * (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0) } else { d.metal_cost };
                standing[of(u.pos)].push((u.pos, worth));
            }
            if !u.being_built && d.weapon_count > 0 && d.reach > 0.0 && matches!(role, Some(Role::Commander | Role::Army | Role::Turret)) {
                armed.push(Armed { pos: u.pos, metal: d.metal_cost, reach: d.reach, speed: d.speed, def: u.def });
            }
        }
        // His approach: from every square at the edge of what we see or hear on radar.
        let covered = |x: f32, z: f32| sensors.los.at(x, z) || sensors.radar.at(x, z);
        let (cell, width, height) = (terrain.cell, terrain.width as i32, terrain.height as i32);
        let centre = |x: i32, z: i32| ((x as f32 + 0.5) * cell, (z as f32 + 0.5) * cell);
        let mut edge: Vec<Vec3> = Vec::new();
        for z in 0..height {
            for x in 0..width {
                let (px, pz) = centre(x, z);
                if covered(px, pz) {
                    continue;
                }
                let beside_seen = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
                    let (nx, nz) = (x + dx, z + dz);
                    nx >= 0 && nz >= 0 && nx < width && nz < height && {
                        let (qx, qz) = centre(nx, nz);
                        covered(qx, qz)
                    }
                });
                if beside_seen {
                    edge.push(Vec3 { x: px, y: 0.0, z: pz });
                }
            }
        }
        // The cost layers: the commanders' ground for his raid, each armed type's own for ours.
        let commander = self.hello.unit_defs[game.commander].id;
        for def in std::iter::once(commander).chain(armed.iter().filter(|a| a.speed > 0.0).map(|a| a.def)) {
            if !self.race.costs.contains_key(&def)
                && let Some(class) = self.def(def).and_then(|d| d.move_class)
            {
                let layer = terrain::costs(terrain, class);
                self.race.costs.insert(def, layer);
            }
        }
        let costs = &self.race.costs;
        let his = costs.get(&commander).and_then(|c| Field::from_costs(terrain, c, &edge));
        // Our mobile armed units in groups: one movement class, each within GROUP_APART of another of the group.
        let class = |def: UnitDefId| self.def(def).and_then(|d| d.move_class);
        let mobile: Vec<usize> = (0..armed.len()).filter(|i| armed[*i].speed > 0.0).collect();
        let mut group_of: Vec<Option<usize>> = vec![None; armed.len()];
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for &i in &mobile {
            if group_of[i].is_some() {
                continue;
            }
            let g = groups.len();
            let mut members = vec![i];
            group_of[i] = Some(g);
            let mut next = 0;
            while next < members.len() {
                let m = members[next];
                next += 1;
                for &j in &mobile {
                    let alike = match (class(armed[m].def), class(armed[j].def)) {
                        (Some(a), Some(b)) => same(&a, &b),
                        _ => false,
                    };
                    if group_of[j].is_none() && alike && armed[m].pos.dist2d(armed[j].pos) <= GROUP_APART {
                        group_of[j] = Some(g);
                        members.push(j);
                    }
                }
            }
            groups.push(members);
        }
        let fields: Vec<Option<Field>> = groups
            .iter()
            .map(|members| {
                let layer = costs.get(&armed[members[0]].def)?;
                let origins: Vec<Vec3> = members.iter().map(|m| armed[*m].pos).collect();
                Field::from_costs(terrain, layer, &origins)
            })
            .collect();
        let mut cells: Vec<String> = Vec::new();
        for (k, here) in standing.iter().enumerate() {
            let metal: f32 = here.iter().map(|(_, w)| w).sum();
            if metal < 1.0 {
                continue;
            }
            let warn = his.as_ref().and_then(|f| here.iter().filter_map(|(p, _)| f.distance(*p)).reduce(f32::min)).map(|d| ((d - RAID_REACH) / RAID_SPEED).max(0.0));
            let mut ours = [0.0f32; AFTER.len()];
            let mut turrets = 0.0f32;
            for (i, a) in armed.iter().enumerate() {
                if a.speed == 0.0 {
                    if here.iter().any(|(p, _)| p.dist2d(a.pos) <= a.reach) {
                        turrets += a.metal;
                    }
                    continue;
                }
                let Some(field) = group_of[i].and_then(|g| fields[g].as_ref()) else { continue };
                let Some(walk) = here.iter().filter_map(|(p, _)| field.distance(*p)).reduce(f32::min) else { continue };
                let arrives = (walk - a.reach).max(0.0) / a.speed;
                for (n, after) in AFTER.iter().enumerate() {
                    if warn.is_none_or(|w| arrives <= w + after) {
                        ours[n] += a.metal;
                    }
                }
            }
            let warn = warn.map_or("-".to_string(), |w| format!("{w:.0}"));
            let ours: Vec<String> = ours.iter().map(|m| format!("{m:.0}")).collect();
            cells.push(format!("r{k} {metal:.0} warn {warn} ours {} turrets {turrets:.0}", ours.join("/")));
        }
        self.log(format_args!(
            "race: seen {:.2} radar {:.2}, edge {} squares, {} groups, {} ms | {}",
            sensors.los.share(),
            sensors.radar.share(),
            edge.len(),
            groups.len(),
            began.elapsed().as_millis(),
            cells.join(" | ")
        ));
    }
}
