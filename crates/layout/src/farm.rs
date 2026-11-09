//! The farm (docs/design/2026-11-08-farm-holds.md §1.3, steps 2 and 3): the free ground within reach of the block's
//! construction turrets, read from the turrets standing, as a grid of cells each carrying how many turrets reach it.
//! Nothing is drawn in advance: the rows of frame slots the block reserved until 2026-11-08 were a hand-drawn stand-in
//! for "where the turrets will gather", and they told the player the block was full (2 of 12 slots) while the ground
//! in reach of thirty turrets held two dozen advanced fusions (econ-flagship-15, -19). The farm is what the turrets
//! make it, and it is measured, not promised: a frame's site is the free cell where the power in reach is greatest
//! (`choose`, `Purpose::Frame`), a turret's the lattice point that covers the most free farm ground already in some
//! turret's reach (`Purpose::Turret`), and the report's room is the free cells over a power line counted in footprints.

use bot_protocol::Vec3;

use crate::choose::World;
use crate::{Facts, Footprint, GRID, Rect, SQUARE};

/// Turrets farther than this from the block's centre are not the farm's (one by a far plant, a defence's helper).
pub const FARM_EXTENT: f32 = 1_600.0;
/// The two power lines the room is counted over: a frame in reach of this many turrets stands at the farm's pace.
pub const ROOM_HIGH: u16 = 30;
pub const ROOM_LOW: u16 = 10;
/// Nothing but an extractor stands within this of a spot's centre (the shim's keep-out box).
const SPOT_BOX_HALF: f32 = 32.0;
/// The ground test's footprint: one cell.
const CELL: Footprint = Footprint { x: 2, z: 2 };

/// The farm's cells: a window round the block covering every turret's reach.
pub struct Farm {
    x0: f32,
    z0: f32,
    nx: i32,
    nz: i32,
    /// Whether a building's centre could stand on the cell: buildable ground, nothing standing on it, off the
    /// spots, every plant's apron and every reservation.
    free: Vec<bool>,
    /// How many turrets reach the cell's centre.
    turrets: Vec<u16>,
    /// Row prefix sums of the cells that are free and in reach of at least one turret: (nx + 1) a row.
    farm_sums: Vec<i32>,
}

impl Farm {
    /// The farm over `world` round `centre`: every construction turret (`world.workers`) within FARM_EXTENT of it,
    /// the window their reach covers (at least `reach` round the centre), each cell tested for `facts`' ground.
    pub fn over(world: &World, centre: Vec3, reach: f32, facts: &Facts) -> Farm {
        let turrets: Vec<&crate::choose::Worker> = world.workers.iter().filter(|w| w.at.dist2d(centre) <= FARM_EXTENT).collect();
        let (mut x0, mut z0, mut x1, mut z1) = (centre.x - reach, centre.z - reach, centre.x + reach, centre.z + reach);
        for w in &turrets {
            x0 = x0.min(w.at.x - w.reach);
            z0 = z0.min(w.at.z - w.reach);
            x1 = x1.max(w.at.x + w.reach);
            z1 = z1.max(w.at.z + w.reach);
        }
        let x0 = (x0 / GRID).floor() * GRID;
        let z0 = (z0 / GRID).floor() * GRID;
        let nx = ((x1 - x0) / GRID).ceil().max(1.0) as i32;
        let nz = ((z1 - z0) / GRID).ceil().max(1.0) as i32;
        let n = (nx * nz) as usize;
        let mut free = vec![false; n];
        for j in 0..nz {
            for i in 0..nx {
                let p = Vec3 { x: x0 + (i as f32 + 0.5) * GRID, y: 0.0, z: z0 + (j as f32 + 0.5) * GRID };
                let cell = CELL.rect(p);
                if !world.ground.stands(CELL, p, facts) {
                    continue;
                }
                if world.standing_any(&cell, |s| s.rect.overlaps(&cell)) {
                    continue;
                }
                if world.spots_any(&cell, |s| s.dist2d(p) < world.spot_keep_out || Rect { x0: s.x - SPOT_BOX_HALF, z0: s.z - SPOT_BOX_HALF, x1: s.x + SPOT_BOX_HALF, z1: s.z + SPOT_BOX_HALF }.overlaps(&cell)) {
                    continue;
                }
                if world.plants.iter().any(|pl| pl.apron().overlaps(&cell)) || world.reservations.iter().any(|r| r.rect.overlaps(&cell)) {
                    continue;
                }
                free[(j * nx + i) as usize] = true;
            }
        }
        let mut count = vec![0u16; n];
        for w in &turrets {
            let i0 = (((w.at.x - w.reach - x0) / GRID).floor() as i32).max(0);
            let i1 = (((w.at.x + w.reach - x0) / GRID).ceil() as i32).min(nx);
            let j0 = (((w.at.z - w.reach - z0) / GRID).floor() as i32).max(0);
            let j1 = (((w.at.z + w.reach - z0) / GRID).ceil() as i32).min(nz);
            let r2 = w.reach * w.reach;
            for j in j0..j1 {
                let dz = z0 + (j as f32 + 0.5) * GRID - w.at.z;
                for i in i0..i1 {
                    let dx = x0 + (i as f32 + 0.5) * GRID - w.at.x;
                    if dx * dx + dz * dz <= r2 {
                        count[(j * nx + i) as usize] = count[(j * nx + i) as usize].saturating_add(1);
                    }
                }
            }
        }
        let mut farm_sums = vec![0i32; ((nx + 1) * nz) as usize];
        for j in 0..nz {
            for i in 0..nx {
                let k = (j * nx + i) as usize;
                farm_sums[(j * (nx + 1) + i + 1) as usize] = farm_sums[(j * (nx + 1) + i) as usize] + i32::from(free[k] && count[k] >= 1);
            }
        }
        Farm { x0, z0, nx, nz, free, turrets: count, farm_sums }
    }

    /// How many turrets reach the cell under `p` (0 off the window).
    pub fn turrets_at(&self, p: Vec3) -> u16 {
        let i = ((p.x - self.x0) / GRID).floor() as i32;
        let j = ((p.z - self.z0) / GRID).floor() as i32;
        if i < 0 || j < 0 || i >= self.nx || j >= self.nz {
            return 0;
        }
        self.turrets[(j * self.nx + i) as usize]
    }

    /// The free cells in reach of at least `at_least` turrets.
    pub fn room(&self, at_least: u16) -> usize {
        self.free.iter().zip(&self.turrets).filter(|(f, t)| **f && **t >= at_least).count()
    }

    /// The free farm cells (in reach of some standing turret) within `reach` of `at`: what a turret there would
    /// add its power to. Row by row over the circle's chord, from the prefix sums.
    pub fn gain(&self, at: Vec3, reach: f32) -> usize {
        let j0 = (((at.z - reach - self.z0) / GRID).floor() as i32).max(0);
        let j1 = (((at.z + reach - self.z0) / GRID).ceil() as i32).min(self.nz);
        let mut total = 0i32;
        for j in j0..j1 {
            let dz = self.z0 + (j as f32 + 0.5) * GRID - at.z;
            let half = (reach * reach - dz * dz).max(0.0).sqrt();
            // The cells whose centres lie within the chord.
            let i0 = (((at.x - half - self.x0) / GRID - 0.5).ceil() as i32).clamp(0, self.nx);
            let i1 = (((at.x + half - self.x0) / GRID - 0.5).floor() as i32 + 1).clamp(0, self.nx);
            if i1 > i0 {
                total += self.farm_sums[(j * (self.nx + 1) + i1) as usize] - self.farm_sums[(j * (self.nx + 1) + i0) as usize];
            }
        }
        total.max(0) as usize
    }

    /// Whether the farm has any ground yet (some turret stands and reaches a free cell).
    pub fn stands(&self) -> bool {
        self.farm_sums.iter().any(|s| *s > 0)
    }
}

/// How many cells a footprint covers.
pub fn cells_of(fp: Footprint) -> usize {
    let across = |squares: i32| ((squares as f32 * SQUARE) / GRID).ceil().max(1.0) as usize;
    across(fp.x) * across(fp.z)
}

/// About how many footprints `cells` of free ground hold.
pub fn footprints(cells: usize, fp: Footprint) -> usize {
    cells / cells_of(fp).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choose::{Standing, Worker};
    use crate::Ground;
    use bot_protocol::{Terrain, UnitDefId};

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn flat() -> Terrain {
        let (w, h) = (256u32, 256u32);
        Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn facts() -> Facts {
        Facts { def: UnitDefId(7), footprint: Footprint { x: 12, z: 12 }, build_distance: 0.0, build_speed: 0.0, build_time: 312_500.0, metal: 9700.0, health: 8300.0, blast: None, height_dif: 10.7, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false }
    }

    fn world<'a>(t: &'a Terrain, open: &'a dyn Fn(Vec3) -> bool) -> World<'a> {
        World { ground: Ground::new(t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), refused: Vec::new(), home: at(2048.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open), lattice: None, farm_frame: None, walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(), farm: Default::default(), fixed: Default::default(), walk_sums: Default::default() }
    }

    /// With no turret the farm has no ground; with one, the free cells within its reach, less what stands on
    /// them; with two overlapping, the overlap reaches two; a turret site beside the pair gains the cells it
    /// shares with them and a site far off gains nothing.
    #[test]
    fn the_farm_is_the_free_ground_the_turrets_reach_and_a_turret_gains_what_it_shares() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let c = at(2000.0, 2000.0);
        let f = facts();
        let farm = Farm::over(&w, c, 400.0, &f);
        assert!(!farm.stands() && farm.room(1) == 0);
        w.workers.push(Worker { at: c, reach: 400.0, power: 200.0 });
        // A solar standing in reach takes its cells.
        w.standing.push(Standing { rect: Footprint { x: 10, z: 10 }.rect(at(2100.0, 2000.0)), def: Some(UnitDefId(3)), kind: "armsolar".into(), ours: true, bank: true, metal: 155.0, health: 340.0, blast: None, name: "armsolar_1".into() });
        let farm = Farm::over(&w, c, 400.0, &f);
        let disc = (std::f32::consts::PI * 400.0 * 400.0 / (GRID * GRID)) as usize;
        let room = farm.room(1);
        assert!(room > disc * 9 / 10 && room < disc, "one turret's disc less a solar: {room} of {disc}");
        assert_eq!(farm.room(2), 0);
        assert_eq!(farm.turrets_at(c), 1);
        assert_eq!(farm.turrets_at(at(2100.0, 2000.0)), 1, "a cell under the solar is reached, not free");
        w.workers.push(Worker { at: at(2096.0, 2000.0), reach: 400.0, power: 200.0 });
        let farm = Farm::over(&w, c, 400.0, &f);
        assert!(farm.room(2) > 0 && farm.room(2) < farm.room(1));
        // A site beside the pair shares most of its disc with them; one 1,000 away shares nothing.
        let beside = farm.gain(at(2048.0, 2096.0), 400.0);
        assert!(beside > disc / 2, "{beside} of {disc}");
        assert_eq!(farm.gain(at(3100.0, 2000.0), 400.0), 0);
        assert!(farm.gain(at(2500.0, 2000.0), 400.0) < beside);
        // Footprints: an advanced fusion is 96 by 96, thirty-six cells.
        assert_eq!(cells_of(Footprint { x: 12, z: 12 }), 36);
        assert_eq!(footprints(360, Footprint { x: 12, z: 12 }), 10);
    }
}
