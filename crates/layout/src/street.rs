//! The street rule (the design §1.2): no placement may close a plant's way to the open. A plant has an apron, the
//! ground before its door kept clear outright, and a street: wherever its widest product can walk from the door to
//! the open, through the ground its units stand on and round every footprint. The check floods a window round the
//! plant with the candidate footprint added; a site that closes any plant's street, ours or an ally's, is passed over.
//! The same check reports a street's state for the player ("closed for Bulls by armsolar_3041"). The same flood,
//! from where a builder of ours stands, keeps a working builder a way out (`way_out`): a bank of turbines chained
//! edge to edge round a constructor at its extractor walled it in for two minutes (1793769060-human, t16 at 4:55).

use bot_protocol::Vec3;
use serde::{Deserialize, Serialize};

use crate::{Footprint, GRID, Ground, Passable, Rect, facing_dir, lane_rect};

/// The ground before a plant's door kept clear of everything: the engine's empty-spot search walks the front
/// (`Factory.cpp` `SendToEmptySpot`), and the people clear about 100 (the study §4); six cells of the grid.
pub const APRON: f32 = 96.0;
/// How far from its door a plant's street must reach: the open.
pub const STREET_REACH: f32 = 1200.0;

/// A plant whose street the rule keeps: ours or an ally's.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub centre: Vec3,
    /// Before facing.
    pub footprint: Footprint,
    pub facing: i32,
    /// Half the width of its widest product, in elmos.
    pub product_half: f32,
    pub ours: bool,
    /// How the report names it.
    pub name: String,
    /// What it makes, for the report's words ("closed for Bulls").
    pub widest: String,
    /// The ground its products stand on (`Facts::product_passable`); its street runs over it, and over the caller's
    /// `open` when it has none.
    #[serde(with = "crate::dump::walk_bits")]
    pub passable: Option<Passable>,
}

impl Plant {
    /// Whether its products stand at `p`: by its own walk set, else by the caller's `open`.
    pub fn stands(&self, ground: &Ground, open: &dyn Fn(Vec3) -> bool, p: Vec3) -> bool {
        match &self.passable {
            Some(walk) => ground.walks(walk, p.x, p.z),
            None => open(p),
        }
    }

    pub fn rect(&self) -> Rect {
        self.footprint.faced(self.facing).rect(self.centre)
    }

    /// The ground before its door kept clear outright.
    pub fn apron(&self) -> Rect {
        lane_rect(self.footprint, self.centre, self.facing, APRON, self.product_half)
    }

    /// The cells of its widest product: a unit of `k` cells a side.
    fn product_cells(&self) -> i32 {
        ((self.product_half * 2.0) / GRID).ceil().max(1.0) as i32
    }

    /// Where its units stand once off the pad: before the front face, half the product and a cell out.
    pub fn door(&self) -> Vec3 {
        let (_, half_depth) = self.footprint.faced(self.facing).half();
        let (fx, fz) = facing_dir(self.facing);
        let out = half_depth + (self.product_cells() as f32 / 2.0 + 1.0) * GRID;
        Vec3 { x: self.centre.x + fx * out, y: self.centre.y, z: self.centre.z + fz * out }
    }

    /// Whether `rect` lies where it could affect this plant's street.
    pub fn concerns(&self, rect: &Rect) -> bool {
        window(self.centre).overlaps(rect)
    }
}

/// The ground a flood from `centre` runs over: within `STREET_REACH` of it.
fn window(centre: Vec3) -> Rect {
    Rect { x0: centre.x - STREET_REACH, z0: centre.z - STREET_REACH, x1: centre.x + STREET_REACH, z1: centre.z + STREET_REACH }
}

/// A mobile builder of ours that must keep a way out: where it stands, half its footprint's width and the ground
/// it walks (the caller's `open` when it has none).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Builder {
    pub at: Vec3,
    pub half: f32,
    #[serde(with = "crate::dump::walk_bits")]
    pub passable: Option<Passable>,
}

impl Builder {
    /// The ground its body takes: no site goes over it (`choose::candidate`).
    pub fn body(&self) -> Rect {
        Rect { x0: self.at.x - self.half, z0: self.at.z - self.half, x1: self.at.x + self.half, z1: self.at.z + self.half }
    }

    /// Whether `rect` lies where it could wall this builder in.
    pub fn concerns(&self, rect: &Rect) -> bool {
        window(self.at).overlaps(rect)
    }

    fn stands(&self, ground: &Ground, open: &dyn Fn(Vec3) -> bool, p: Vec3) -> bool {
        match &self.passable {
            Some(walk) => ground.walks(walk, p.x, p.z),
            None => open(p),
        }
    }
}

/// Whether the builder walks from where it stands to the open, over its own ground and round every rect of
/// `blocks`: the street test from a unit's own position, its own body the product.
pub fn way_out(ground: &Ground, builder: &Builder, blocks: &[Rect], open: &dyn Fn(Vec3) -> bool) -> bool {
    let k = ((builder.half * 2.0) / GRID).ceil().max(1.0) as i32;
    Flood::around(ground, builder.at, builder.at, k, &|p| builder.stands(ground, open, p), blocks).reaches_the_open()
}

/// The state of a plant's street.
#[derive(Clone, Debug, PartialEq)]
pub enum StreetState {
    Open,
    /// Closed, and the index into `blocks` of a footprint whose removal opens it, if one does.
    Closed { opener: Option<usize> },
}

/// Whether the plant's widest product walks from its door to the open, over the plant's own ground (`open` for a
/// plant without one) and round every rect of `blocks` (the plant's own footprint among them or not: its door lies
/// outside it).
pub fn street_open(ground: &Ground, plant: &Plant, blocks: &[Rect], open: &dyn Fn(Vec3) -> bool) -> bool {
    Flood::new(ground, plant, blocks, open).reaches_the_open()
}

/// The street's state, and the one block of `ours` (indices into `blocks`) whose removal would open a closed one:
/// the nearest such to the door.
pub fn street_state(ground: &Ground, plant: &Plant, blocks: &[Rect], ours: &[usize], open: &dyn Fn(Vec3) -> bool) -> StreetState {
    if street_open(ground, plant, blocks, open) {
        return StreetState::Open;
    }
    let door = plant.door();
    let mut candidates: Vec<usize> = ours.iter().copied().filter(|i| plant.concerns(&blocks[*i])).collect();
    candidates.sort_by(|a, b| blocks[*a].centre().dist2d(door).total_cmp(&blocks[*b].centre().dist2d(door)));
    for i in candidates {
        let without: Vec<Rect> = blocks.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, r)| *r).collect();
        if street_open(ground, plant, &without, open) {
            return StreetState::Closed { opener: Some(i) };
        }
    }
    StreetState::Closed { opener: None }
}

/// A reached cell on the window's edge counts as the open when it is at least this far from the door, when the
/// window has such cells (a small map's edge beside the door is no way out).
const OPEN_FROM_DOOR: f32 = 300.0;

/// The flood's window: the ground within `STREET_REACH` of the plant, clamped to the map, in cells of GRID; blocked
/// cells summed for the product's block test.
struct Flood {
    nx: i32,
    nz: i32,
    k: i32,
    x0: f32,
    z0: f32,
    /// Prefix sums of blocked cells, (nx + 1) by (nz + 1).
    sums: Vec<i32>,
    start: Option<(i32, i32)>,
    door: Vec3,
}

impl Flood {
    fn new(ground: &Ground, plant: &Plant, blocks: &[Rect], open: &dyn Fn(Vec3) -> bool) -> Flood {
        Flood::around(ground, plant.centre, plant.door(), plant.product_cells(), &|p| plant.stands(ground, open, p), blocks)
    }

    /// The flood over the window round `centre`, starting with a block of `k` cells a side at `door`, over the
    /// cells `stands` and round every rect of `blocks` in the window.
    fn around(ground: &Ground, centre: Vec3, door: Vec3, k: i32, stands: &dyn Fn(Vec3) -> bool, blocks: &[Rect]) -> Flood {
        let (map_w, map_h) = ground.extent();
        let x0 = (centre.x - STREET_REACH).max(0.0);
        let z0 = (centre.z - STREET_REACH).max(0.0);
        let x1 = (centre.x + STREET_REACH).min(map_w);
        let z1 = (centre.z + STREET_REACH).min(map_h);
        let nx = ((x1 - x0) / GRID).floor().max(1.0) as i32;
        let nz = ((z1 - z0) / GRID).floor().max(1.0) as i32;
        let win = window(centre);
        let near: Vec<&Rect> = blocks.iter().filter(|r| win.overlaps(r)).collect();
        let mut sums = vec![0i32; ((nx + 1) * (nz + 1)) as usize];
        let at = |i: i32, j: i32| ((j * (nx + 1)) + i) as usize;
        for j in 0..nz {
            for i in 0..nx {
                let p = Vec3 { x: x0 + (i as f32 + 0.5) * GRID, y: 0.0, z: z0 + (j as f32 + 0.5) * GRID };
                let blocked = !stands(p) || near.iter().any(|r| r.contains(p));
                sums[at(i + 1, j + 1)] = sums[at(i, j + 1)] + sums[at(i + 1, j)] - sums[at(i, j)] + i32::from(blocked);
            }
        }
        let (di, dj) = (((door.x - x0) / GRID).floor() as i32 - k / 2, ((door.z - z0) / GRID).floor() as i32 - k / 2);
        let start = (di >= 0 && dj >= 0 && di + k <= nx && dj + k <= nz).then_some((di, dj));
        Flood { nx, nz, k, x0, z0, sums, start, door }
    }

    /// Whether the product's block with its corner at (i, j) stands on unblocked cells.
    fn free(&self, i: i32, j: i32) -> bool {
        let (nx, nz, k) = (self.nx, self.nz, self.k);
        if i < 0 || j < 0 || i + k > nx || j + k > nz {
            return false;
        }
        let at = |i: i32, j: i32| ((j * (nx + 1)) + i) as usize;
        let blocked = self.sums[at(i + k, j + k)] - self.sums[at(i, j + k)] - self.sums[at(i + k, j)] + self.sums[at(i, j)];
        blocked == 0
    }

    fn on_edge(&self, i: i32, j: i32) -> bool {
        i == 0 || j == 0 || i + self.k == self.nx || j + self.k == self.nz
    }

    fn centre_of(&self, i: i32, j: i32) -> Vec3 {
        Vec3 { x: self.x0 + (i as f32 + self.k as f32 / 2.0) * GRID, y: 0.0, z: self.z0 + (j as f32 + self.k as f32 / 2.0) * GRID }
    }

    fn reaches_the_open(&self) -> bool {
        let Some(start) = self.start else { return false };
        if !self.free(start.0, start.1) {
            return false;
        }
        // Does the window have an edge far enough from the door at all (a small map may not)?
        let far_edge = {
            let mut any = false;
            for i in 0..=(self.nx - self.k) {
                for j in [0, self.nz - self.k] {
                    any |= self.centre_of(i, j).dist2d(self.door) >= OPEN_FROM_DOOR;
                }
            }
            for j in 0..=(self.nz - self.k) {
                for i in [0, self.nx - self.k] {
                    any |= self.centre_of(i, j).dist2d(self.door) >= OPEN_FROM_DOOR;
                }
            }
            any
        };
        let (nx, nz, k) = (self.nx, self.nz, self.k);
        let mut seen = vec![false; (nx * nz) as usize];
        let mut stack = vec![start];
        seen[(start.1 * nx + start.0) as usize] = true;
        while let Some((i, j)) = stack.pop() {
            if self.on_edge(i, j) && (!far_edge || self.centre_of(i, j).dist2d(self.door) >= OPEN_FROM_DOOR) {
                return true;
            }
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (ni, nj) = (i + di, j + dj);
                if ni < 0 || nj < 0 || ni + k > nx || nj + k > nz || seen[(nj * nx + ni) as usize] {
                    continue;
                }
                seen[(nj * nx + ni) as usize] = true;
                if self.free(ni, nj) {
                    stack.push((ni, nj));
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn flat() -> Terrain {
        let (w, h) = (256u32, 256u32);
        Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn plant(facing: i32) -> Plant {
        Plant { centre: at(2048.0, 2048.0), footprint: Footprint { x: 12, z: 12 }, facing, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into(), passable: None }
    }

    /// A plant alone on open ground has its street; a ring of footprints round it, closing every gap under a Bull's
    /// width, closes it; one footprint taken out of the ring opens it again, and the state names that footprint.
    #[test]
    fn a_ring_round_the_plant_closes_the_street_and_one_gap_opens_it() {
        let t = flat();
        let ground = Ground::new(&t, 4096.0, 4096.0);
        let open = |_: Vec3| true;
        let p = plant(1);
        assert!(street_open(&ground, &p, &[], &open));
        // The ring: solars (80 wide) touching, 240 from the plant's centre on every side.
        let solar = |x: f32, z: f32| Footprint { x: 10, z: 10 }.rect(at(x, z));
        let mut ring: Vec<Rect> = Vec::new();
        for k in -3..=3 {
            let o = 80.0 * k as f32;
            ring.push(solar(2288.0, 2048.0 + o));
            ring.push(solar(1808.0, 2048.0 + o));
            ring.push(solar(2048.0 + o, 1808.0));
            ring.push(solar(2048.0 + o, 2288.0));
        }
        assert!(!street_open(&ground, &p, &ring, &open));
        let ours: Vec<usize> = (0..ring.len()).collect();
        match street_state(&ground, &p, &ring, &ours, &open) {
            StreetState::Closed { opener: Some(i) } => assert!(ring[i].centre().x == 2288.0 && (ring[i].centre().z - 2048.0).abs() <= 80.0, "the opener is before the door: {:?}", ring[i]),
            other => panic!("{other:?}"),
        }
        // A gap of one solar (80) passes a Bull (64) but not a Thor (96 wide).
        let east_middle = ring.iter().position(|r| r.centre() == at(2288.0, 2048.0)).unwrap();
        let gapped: Vec<Rect> = ring.iter().enumerate().filter(|(i, _)| *i != east_middle).map(|(_, r)| *r).collect();
        assert!(street_open(&ground, &p, &gapped, &open));
        let thor = Plant { product_half: 48.0, ..plant(1) };
        assert!(!street_open(&ground, &thor, &gapped, &open));
        // A plant beside the map's edge on a small map still has its street when it reaches that edge.
        let small = Terrain { cell: 16.0, width: 64, height: 64, heights: vec![50; 64 * 64], slopes: vec![0; 64 * 64], metal: Vec::new() };
        let corner = Plant { centre: at(512.0, 512.0), ..plant(1) };
        assert!(street_open(&Ground::new(&small, 1024.0, 1024.0), &corner, &[], &open));
    }

    #[test]
    fn the_door_lies_before_the_front_face() {
        let p = plant(1);
        // Facing east: the front face at x = 2096; a Bull is 4 cells, so the door is 3 cells out.
        assert_eq!(p.door(), at(2096.0 + 48.0, 2048.0));
        assert_eq!(p.apron(), Rect { x0: 2096.0, z0: 2048.0 - 48.0 - 32.0, x1: 2192.0, z1: 2048.0 + 48.0 + 32.0 });
        let t = flat();
        let f = Flood::new(&Ground::new(&t, 4096.0, 4096.0), &p, &[], &|_| true);
        assert_eq!((f.x0, f.z0), (2048.0 - STREET_REACH, 2048.0 - STREET_REACH));
        assert!(f.start.is_some());
    }
}

#[cfg(test)]
mod sea_tests {
    use super::*;
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// Land at 50 west of x 2000; water east of it at `deep`, except where `shallow` says -10.
    fn shore(deep: i16, shallow: impl Fn(f32, f32) -> bool) -> Terrain {
        let (w, h) = (256u32, 256u32);
        let mut heights = Vec::new();
        for j in 0..h {
            for i in 0..w {
                let (x, z) = (i as f32 * 16.0 + 8.0, j as f32 * 16.0 + 8.0);
                heights.push(if x < 2000.0 { 50 } else if shallow(x, z) { -10 } else { deep });
            }
        }
        Terrain { cell: 16.0, width: w, height: h, heights, slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    /// A plant's street runs over its own products' ground, whatever the caller's `open` says: a shipyard's over
    /// deep water with every land cell impassable, closed when shallows pen the water before its door.
    #[test]
    fn a_shipyards_street_is_the_deep_water_before_its_door() {
        let nothing_walks = |_: Vec3| false;
        let yard = |t: &Terrain| Plant { centre: at(2100.0, 2048.0), footprint: Footprint { x: 6, z: 6 }, facing: 1, product_half: 32.0, ours: true, name: "yard".into(), widest: "Corsair".into(), passable: Some(crate::sea_walk(t, 30.0)) };
        let deep = shore(-40, |_, _| false);
        assert!(street_open(&Ground::new(&deep, 4096.0, 4096.0), &yard(&deep), &[], &nothing_walks));
        // A pocket of deep water 400 across before the door, shallows all round it.
        let penned = shore(-40, |x, z| !(x < 2500.0 && (z - 2048.0).abs() < 250.0));
        assert!(!street_open(&Ground::new(&penned, 4096.0, 4096.0), &yard(&penned), &[], &nothing_walks));
        // Water a ship's floor shallow (-20 for a floor of 30) is as closed as land.
        let too_shallow = shore(-20, |_, _| false);
        assert!(!street_open(&Ground::new(&too_shallow, 4096.0, 4096.0), &yard(&too_shallow), &[], &nothing_walks));
    }

    /// A vehicle plant's street is judged on its vehicles' ground, not the caller's: a rim every bot climbs and no
    /// vehicle does pens it (apex-v0409-comet-sym0-opus-2, the crater's rim north of the three vehicle plants).
    #[test]
    fn a_vehicle_plants_street_is_judged_on_its_vehicles_ground() {
        let t = shore(50, |_, _| false);
        let ground = Ground::new(&t, 4096.0, 4096.0);
        // Bots walk everything; vehicles nothing east of x 2300 (a rim).
        let bots_walk = |_: Vec3| true;
        let vehicles: Passable = (0..256 * 256).map(|k| ((k % 256) as f32 * 16.0 + 8.0) < 2300.0).collect();
        let plant = |passable: Option<Passable>| Plant { centre: at(2100.0, 2048.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant".into(), widest: "Stumpy".into(), passable };
        // A wall of footprints north and south of the door, so the only way out is east over the rim.
        let solar = |x: f32, z: f32| Footprint { x: 10, z: 10 }.rect(at(x, z));
        let mut walls: Vec<Rect> = Vec::new();
        for k in 0..40 {
            walls.push(solar(1000.0 + 80.0 * k as f32, 1800.0));
            walls.push(solar(1000.0 + 80.0 * k as f32, 2300.0));
        }
        for k in 0..7 {
            walls.push(solar(1000.0, 1800.0 + 80.0 * k as f32));
        }
        assert!(street_open(&ground, &plant(None), &walls, &bots_walk), "on the bots' ground the street is open");
        assert!(!street_open(&ground, &plant(Some(vehicles)), &walls, &bots_walk), "on the vehicles' ground the rim pens it");
    }

    /// A builder standing in a bay of turbines has its way out while one side is open; the turbine that shuts the
    /// bay walls it in (t16's advanced constructor at its extractor, 1793769060-human 4:55).
    #[test]
    fn a_builder_in_a_bay_of_turbines_keeps_its_way_out_until_the_bay_shuts() {
        let t = shore(50, |_, _| false);
        let ground = Ground::new(&t, 4096.0, 4096.0);
        let open = |_: Vec3| true;
        let builder = Builder { at: at(2048.0, 2048.0), half: 24.0, passable: None };
        let wind = |x: f32, z: f32| Footprint { x: 6, z: 6 }.rect(at(x, z));
        // Turbines (48 square) touching, on the north, east and south of the builder, 64 off it.
        let mut bay: Vec<Rect> = Vec::new();
        for k in -2..=2 {
            bay.push(wind(2048.0 + 48.0 * k as f32, 1960.0));
            bay.push(wind(2048.0 + 48.0 * k as f32, 2136.0));
            bay.push(wind(2136.0, 2048.0 + 48.0 * k as f32));
        }
        assert!(way_out(&ground, &builder, &bay, &open), "the west is open");
        for k in -2..=2 {
            bay.push(wind(1960.0, 2048.0 + 48.0 * k as f32));
        }
        assert!(!way_out(&ground, &builder, &bay, &open), "walled in");
    }
}
