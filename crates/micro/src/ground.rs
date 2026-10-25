//! The ground as the lane walks it (design D17, 2026-10-25, the user: "Since we are replacing a direct a->b order that
//! the engine would normally handle pathing for, we have to be responsible for running the pathing"). A unit under
//! the engine's own order gets the engine's pathfinder; a group's members are sent to slots hung from an anchor, so
//! the anchor's walk between the points the host names is the lane's to path, and every slot must be ground the body
//! can stand on. The host hands the lane its terrain once (`View::terrain`: the hello's height and slope maps at
//! 16-elmo cells) and the lane builds what it needs from the `terrain` crate: each movement class's passable grid on
//! first use (`Classes`, for the reflexes' steps and the contact arc's slots), and for each group a `Ground`: the
//! cells every member's class can stand on, the room about each (the distance to the nearest impassable cell or the
//! map's edge: `terrain::clearance`, D13) and the way priced by that room for the formation's half-width plus a hull
//! (`terrain::clearance_costs`: flat where the body has the room it wants, dearer toward a wall, so a corner is
//! rounded at that distance and a corridor narrower than the body is walked down its middle). The way for a leg is the
//! priced field from the leg's point descended from where the anchor stands and straightened by line of sight with
//! the room (the circuit drill's method before this, in `sites.rs`, the host's job no longer; its thinning to a point
//! every 256 before the straightening is gone: it dropped the corner at a wall's end and the straightening then
//! joined two points across the wall).
//! The terrain grid knows nothing of what stands on it, so the lane overlays the structures it sees (design D18): every
//! unit of ours or theirs without speed by its type's footprint, the host's remembered buildings out of sight, and the
//! wrecks the snapshot carries (`structures`: one more grid AND-ed into a group's ground, rebuilt when the marked
//! cells change). The engine's own pathing through the shim stays the later truth where that is not enough.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use bot_protocol::{MoveClass, MoveKind, Terrain, Vec3};
use terrain::Field;

/// A point is judged by the nearest standable cell within this many cells (a slot at a cliff's lip snaps onto the
/// ground beside it; a slot in the middle of a lake is cut off).
const SNAP_CELLS: i32 = 6;

/// A movement class as a key: its kind, its slope limit and its depth, rounded (the hands' `ClassKey`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassKey {
    kind: MoveKind,
    max_slope: i32,
    depth: i32,
}

impl ClassKey {
    pub fn of(class: MoveClass) -> ClassKey {
        ClassKey { kind: class.kind, max_slope: (class.max_slope * 255.0).round() as i32, depth: (class.depth * 10.0).round() as i32 }
    }
}

/// Each movement class's passable grid, built from the host's terrain on first use and kept.
#[derive(Default)]
pub struct Classes {
    grids: HashMap<ClassKey, Arc<Vec<bool>>>,
}

impl Classes {
    /// Where a unit of this class can stand, one flag a terrain cell.
    pub fn passable(&mut self, terrain: &Terrain, class: MoveClass) -> Arc<Vec<bool>> {
        self.grids.entry(ClassKey::of(class)).or_insert_with(|| Arc::new(terrain::passable(terrain, class))).clone()
    }

    /// The ground a unit of this class can stand on nearest `pos`, within `SNAP_CELLS`; `pos` itself when there is
    /// none that near (the engine then does what it can, as it would with the host's own order).
    pub fn snap(&mut self, terrain: &Terrain, class: MoveClass, pos: Vec3) -> Vec3 {
        let grid = self.passable(terrain, class);
        nearest(terrain.cell, terrain.width as usize, terrain.height as usize, &grid, pos).map_or(pos, |index| centre(terrain.cell, terrain.width as usize, index, pos.y))
    }
}

/// A structure standing on the ground: its centre and its half extents in elmos along x and z, after its facing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub at: Vec3,
    pub half_x: f32,
    pub half_z: f32,
}

impl Footprint {
    /// A type's footprint (8-elmo squares, x by z before facing) stood at `at` with the engine's facing (0 south,
    /// 1 east, 2 north, 3 west: the odd facings swap the sides).
    pub fn of(at: Vec3, footprint: (i32, i32), facing: i32) -> Footprint {
        let (x, z) = (footprint.0.max(1) as f32 * 4.0, footprint.1.max(1) as f32 * 4.0);
        let (half_x, half_z) = if facing % 2 == 1 { (z, x) } else { (x, z) };
        Footprint { at, half_x, half_z }
    }

    /// A type's footprint stood at `at` facing who knows where: the longer side both ways.
    pub fn facing_unknown(at: Vec3, footprint: (i32, i32)) -> Footprint {
        let half = footprint.0.max(footprint.1).max(1) as f32 * 4.0;
        Footprint { at, half_x: half, half_z: half }
    }

    /// Something of one cell (a wreck of no known type).
    pub fn cell(at: Vec3, cell: f32) -> Footprint {
        Footprint { at, half_x: cell / 2.0, half_z: cell / 2.0 }
    }
}

/// The cells no structure stands on (design D18), one flag a terrain cell, with a hash of the cells marked (the
/// ground's key: a change of what stands where is a change of ground). A footprint marks every cell it touches.
pub fn structures(terrain: &Terrain, footprints: &[Footprint]) -> (Arc<Vec<bool>>, u64) {
    let (width, height, cell) = (terrain.width as usize, terrain.height as usize, terrain.cell);
    let mut grid = vec![true; width * height];
    let mut marked: Vec<usize> = Vec::new();
    if width > 0 && height > 0 && cell > 0.0 {
        for f in footprints {
            let span = |lo: f32, hi: f32, n: usize| -> std::ops::RangeInclusive<usize> {
                let a = (lo / cell).floor().max(0.0) as usize;
                let b = ((hi - 0.01) / cell).floor().max(0.0) as usize;
                a.min(n - 1)..=b.min(n - 1)
            };
            if f.at.x + f.half_x <= 0.0 || f.at.z + f.half_z <= 0.0 || f.at.x - f.half_x >= width as f32 * cell || f.at.z - f.half_z >= height as f32 * cell {
                continue;
            }
            for z in span(f.at.z - f.half_z, f.at.z + f.half_z, height) {
                for x in span(f.at.x - f.half_x, f.at.x + f.half_x, width) {
                    let i = z * width + x;
                    if grid[i] {
                        grid[i] = false;
                        marked.push(i);
                    }
                }
            }
        }
    }
    marked.sort_unstable();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    marked.hash(&mut hasher);
    (Arc::new(grid), hasher.finish())
}

/// A group's ground: where every member's class can stand, the room about each cell and the way's price.
#[derive(Clone, Debug)]
pub struct Ground {
    cell: f32,
    width: usize,
    height: usize,
    passable: Vec<bool>,
    /// Each cell's distance to the nearest impassable cell or the map's edge, in elmos.
    room: Vec<f32>,
    /// The way's price a cell: the flat rate where the room is `wanted` or more, rising toward a wall.
    costs: Vec<u32>,
    /// The room the body wants: its half-width plus a hull.
    pub wanted: f32,
}

impl Ground {
    /// The ground common to these classes' grids (each one flag a cell of `terrain`), priced for a body wanting
    /// `wanted` of room. `None` without terrain or grids.
    pub fn new(terrain: &Terrain, grids: &[Arc<Vec<bool>>], wanted: f32) -> Option<Ground> {
        let (width, height) = (terrain.width as usize, terrain.height as usize);
        if width == 0 || terrain.cell <= 0.0 || grids.is_empty() || grids.iter().any(|g| g.len() != width * height) {
            return None;
        }
        let passable: Vec<bool> = (0..width * height).map(|i| grids.iter().all(|g| g[i])).collect();
        let room = terrain::clearance(terrain, &passable);
        let costs = terrain::clearance_costs(&passable, &room, wanted);
        Some(Ground { cell: terrain.cell, width, height, passable, room, costs, wanted })
    }

    /// The grid's cell edge, in elmos.
    pub fn cell(&self) -> f32 {
        self.cell
    }

    fn index(&self, pos: Vec3) -> Option<usize> {
        let (x, z) = ((pos.x / self.cell) as i64, (pos.z / self.cell) as i64);
        (x >= 0 && z >= 0 && x < self.width as i64 && z < self.height as i64).then(|| z as usize * self.width + x as usize)
    }

    /// Whether the body stands at `pos`.
    pub fn passable_at(&self, pos: Vec3) -> bool {
        self.index(pos).is_some_and(|i| self.passable[i])
    }

    /// The room about `pos`: its distance to the nearest impassable cell or the map's edge; none off the map.
    pub fn room_at(&self, pos: Vec3) -> f32 {
        self.index(pos).map_or(0.0, |i| self.room[i])
    }

    /// The body's ground nearest `pos` within `SNAP_CELLS`, at `pos`'s height; `None` when there is none that near.
    pub fn snap(&self, pos: Vec3) -> Option<Vec3> {
        nearest(self.cell, self.width, self.height, &self.passable, pos).map(|index| centre(self.cell, self.width, index, pos.y))
    }

    /// Where the body stands for a point: the cell nearest `pos` with the room the body wants, within twice that room
    /// (design D18: a route's end on or beside a structure, a factory the army is sent to, put the anchor against it
    /// with room for one file and the body in a column twenty deep; the body stands off it instead). `snap` when no
    /// cell that near has the room.
    pub fn stand(&self, pos: Vec3) -> Option<Vec3> {
        let reach = (2.0 * self.wanted / self.cell).ceil() as i32;
        let (cx, cz) = ((pos.x / self.cell) as i32, (pos.z / self.cell) as i32);
        let mut best: Option<(i32, usize)> = None;
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let (x, z) = (cx + dx, cz + dz);
                if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
                    continue;
                }
                let index = z as usize * self.width + x as usize;
                let apart = dx * dx + dz * dz;
                if self.passable[index] && self.room[index] >= self.wanted && best.is_none_or(|(d, _)| apart < d) {
                    best = Some((apart, index));
                }
            }
        }
        best.map(|(_, index)| centre(self.cell, self.width, index, pos.y)).or_else(|| self.snap(pos))
    }

    /// The way from `from` to `to` over the priced ground: the points after `from`, `to` (snapped) last, bending only
    /// where the ground bends it. `[to]` alone when the ground gives none (`way_found`): the straight line, and the
    /// engine does with each member what it can.
    pub fn way(&self, from: Vec3, to: Vec3) -> Vec<Vec3> {
        self.way_found(from, to).unwrap_or_else(|| vec![to])
    }

    /// `way`, or `None` when the ground gives none: `to` is on no ground the body stands on, or no walk over the ground
    /// joins `from` to it (a cliff, a lake, the sea). The group's state says so (`GroupState::way`), so the hands can
    /// say "no way on foot" instead of walking the body at it.
    pub fn way_found(&self, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
        let goal = self.snap(to)?;
        let field = Field::over_until(self.cell, self.width, self.height, &self.costs, &[goal], Some(from))?;
        let cells = field.route(from);
        if cells.is_empty() {
            return None;
        }
        let mut points = vec![from];
        points.extend(cells.into_iter().map(|p| Vec3 { y: to.y, ..p }).filter(|p| p.dist2d(goal) >= 1.0));
        points.push(goal);
        // Straightened: from each point the furthest later one the body reaches in a straight line with the room the
        // field's way itself kept between them (the way is a chain of cell moves at multiples of 45 degrees, a
        // staircase of bends that snapped a deep body's rear corners 150 at each point of the circuit before this;
        // a line allowed the room at its two ends alone cut toward a wall's lip the way had rounded with room).
        let rooms: Vec<f32> = points.iter().map(|p| self.room_at(*p)).collect();
        let mut way = Vec::new();
        let mut i = 0;
        while i + 1 < points.len() {
            let mut least = vec![f32::INFINITY; points.len()];
            let mut running = f32::INFINITY;
            for j in i..points.len() {
                running = running.min(rooms[j]);
                least[j] = running;
            }
            let mut j = points.len() - 1;
            while j > i + 1 && !self.clear_line(points[i], points[j], self.wanted.min(least[j]) * 0.9) {
                j -= 1;
            }
            way.push(points[j]);
            i = j;
        }
        Some(way)
    }

    /// Whether the body walks from `a` to `b` in a straight line with `needed` of room all the way (a sample a cell).
    fn clear_line(&self, a: Vec3, b: Vec3, needed: f32) -> bool {
        let steps = (a.dist2d(b) / self.cell).ceil().max(1.0) as usize;
        (0..=steps).all(|k| {
            let t = k as f32 / steps as f32;
            let p = Vec3 { x: a.x + (b.x - a.x) * t, y: 0.0, z: a.z + (b.z - a.z) * t };
            self.index(p).is_some_and(|i| self.passable[i] && self.room[i] >= needed)
        })
    }

    /// Whether the body walks from `from` to `to` over its ground within `cap` elmos: a search from `from` that
    /// stops at `to` or when every way out costs more than the cap. False when either end is off the ground.
    pub fn walk_within(&self, from: Vec3, to: Vec3, cap: f32) -> bool {
        let (Some(start), Some(goal)) = (nearest(self.cell, self.width, self.height, &self.passable, from), nearest(self.cell, self.width, self.height, &self.passable, to)) else {
            return false;
        };
        if start == goal {
            return true;
        }
        // Costs in tenths of a cell, as the fields keep them.
        let cap = (cap / self.cell * 10.0) as u32;
        let mut best: HashMap<usize, u32> = HashMap::new();
        let mut queue = BinaryHeap::new();
        best.insert(start, 0);
        queue.push(Reverse((0u32, start)));
        while let Some(Reverse((cost, index))) = queue.pop() {
            if cost > cap {
                return false;
            }
            if index == goal {
                return true;
            }
            if best.get(&index).is_some_and(|b| *b < cost) {
                continue;
            }
            let (x, z) = ((index % self.width) as i32, (index / self.width) as i32);
            for (dx, dz, step) in [(1, 0, 10u32), (-1, 0, 10), (0, 1, 10), (0, -1, 10), (1, 1, 14), (1, -1, 14), (-1, 1, 14), (-1, -1, 14)] {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= self.width as i32 || nz >= self.height as i32 {
                    continue;
                }
                let next = nz as usize * self.width + nx as usize;
                if !self.passable[next] {
                    continue;
                }
                // No cutting corners between two blocked cells.
                if dx != 0 && dz != 0 && !(self.passable[z as usize * self.width + nx as usize] && self.passable[nz as usize * self.width + x as usize]) {
                    continue;
                }
                let total = cost + step;
                if best.get(&next).is_none_or(|b| total < *b) {
                    best.insert(next, total);
                    queue.push(Reverse((total, next)));
                }
            }
        }
        false
    }
}

/// The cell nearest `pos` within `SNAP_CELLS` that `grid` says is standable.
fn nearest(cell: f32, width: usize, height: usize, grid: &[bool], pos: Vec3) -> Option<usize> {
    if width == 0 || cell <= 0.0 || grid.len() != width * height {
        return None;
    }
    let (cx, cz) = ((pos.x / cell) as i32, (pos.z / cell) as i32);
    let mut best: Option<(i32, usize)> = None;
    for dz in -SNAP_CELLS..=SNAP_CELLS {
        for dx in -SNAP_CELLS..=SNAP_CELLS {
            let (x, z) = (cx + dx, cz + dz);
            if x < 0 || z < 0 || x >= width as i32 || z >= height as i32 {
                continue;
            }
            let index = z as usize * width + x as usize;
            let apart = dx * dx + dz * dz;
            if grid[index] && best.is_none_or(|(d, _)| apart < d) {
                best = Some((apart, index));
            }
        }
    }
    best.map(|(_, index)| index)
}

fn centre(cell: f32, width: usize, index: usize, y: f32) -> Vec3 {
    Vec3 { x: ((index % width) as f32 + 0.5) * cell, y, z: ((index / width) as f32 + 0.5) * cell }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A flat 64 x 64 map at 16-elmo cells (1,024 elmos a side) with a wall of cliff down the middle from the north
    /// edge to 256 short of the south edge: the way round goes south.
    fn walled() -> Terrain {
        let (w, h) = (64usize, 64usize);
        let mut slopes = vec![0u8; w * h];
        for z in 0..h - 16 {
            for x in 30..34 {
                slopes[z * w + x] = 255;
            }
        }
        Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes, metal: Vec::new() }
    }

    fn tank() -> MoveClass {
        MoveClass { kind: MoveKind::Tank, max_slope: 0.3, depth: 20.0, slope_mod: 0.0 }
    }

    fn ground(wanted: f32) -> Ground {
        let terrain = walled();
        let mut classes = Classes::default();
        let grid = classes.passable(&terrain, tank());
        Ground::new(&terrain, &[grid], wanted).expect("ground")
    }

    /// A flat 64 x 64 map at 16-elmo cells.
    fn flat() -> Terrain {
        let (w, h) = (64usize, 64usize);
        Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes: vec![0u8; w * h], metal: Vec::new() }
    }

    #[test]
    fn a_structure_on_the_way_bends_the_way_round_it_and_a_slot_on_it_snaps_beside_it() {
        let terrain = flat();
        let mut classes = Classes::default();
        let tanks = classes.passable(&terrain, tank());
        // A factory of 8 x 6 squares (64 x 48 elmos about its centre) facing east at the map's middle: 80 x 64.
        let lab = Footprint::of(at(512.0, 512.0), (8, 6), 1);
        assert_eq!((lab.half_x, lab.half_z), (24.0, 32.0));
        let (blocked, hash) = structures(&terrain, &[lab]);
        let (_, same) = structures(&terrain, &[lab]);
        let (_, other) = structures(&terrain, &[Footprint::of(at(528.0, 512.0), (8, 6), 1)]);
        assert_eq!(hash, same);
        assert_ne!(hash, other);
        // The cells it touches: x 488-536 is cells 30-33, z 480-544 is cells 30-33 (544 lands on a boundary: 33 last).
        let marked: Vec<usize> = blocked.iter().enumerate().filter(|(_, b)| !**b).map(|(i, _)| i).collect();
        assert_eq!(marked.len(), 16, "{marked:?}");
        assert!(marked.contains(&(30 * 64 + 30)) && marked.contains(&(33 * 64 + 33)) && !marked.contains(&(34 * 64 + 33)));
        let bare = Ground::new(&terrain, &[tanks.clone()], 96.0).unwrap();
        let built = Ground::new(&terrain, &[tanks, blocked], 96.0).unwrap();
        let straight = bare.way(at(100.0, 520.0), at(900.0, 520.0));
        assert!(straight.iter().all(|p| (p.z - 520.0).abs() < 1.0), "on bare ground the way is the line: {straight:?}");
        let round = built.way(at(100.0, 520.0), at(900.0, 520.0));
        assert!(round.iter().any(|p| (p.z - 520.0).abs() > 64.0), "the way bends round the factory: {round:?}");
        assert!(round.iter().all(|p| built.passable_at(*p)));
        let stand = built.stand(at(512.0, 512.0)).expect("a stand near the factory");
        assert!(built.room_at(stand) >= 96.0 && stand.dist2d(at(512.0, 512.0)) < 200.0, "the body stands off the factory with its room: {stand:?} ({:.0})", built.room_at(stand));
        assert!((bare.stand(at(512.0, 512.0)).unwrap().dist2d(at(512.0, 512.0))) < 12.0, "on bare ground the stand is the point's cell");
        let beside = built.snap(at(512.0, 512.0)).expect("ground beside the factory");
        assert!(!marked.contains(&((beside.z / 16.0) as usize * 64 + (beside.x / 16.0) as usize)), "{beside:?}");
        assert!(beside.dist2d(at(512.0, 512.0)) < 80.0);
        // A wreck of no type marks one cell; one of a type, that type's footprint the longer side both ways.
        let (one, _) = structures(&terrain, &[Footprint::cell(at(200.0, 200.0), 16.0)]);
        assert_eq!(one.iter().filter(|b| !**b).count(), 1);
        let (wide, _) = structures(&terrain, &[Footprint::facing_unknown(at(200.0, 200.0), (6, 2))]);
        assert_eq!(wide.iter().filter(|b| !**b).count(), 9, "48 about 200 is 152-248: cells 9-15, three a side");
    }

    #[test]
    fn the_way_stopped_at_its_start_is_the_whole_fields_way() {
        let g = ground(96.0);
        let (from, to) = (at(200.0, 300.0), at(850.0, 300.0));
        let whole = Field::over(g.cell, g.width, g.height, &g.costs, &[g.snap(to).unwrap()]).unwrap().route(from);
        let bounded = Field::over_until(g.cell, g.width, g.height, &g.costs, &[g.snap(to).unwrap()], Some(from)).unwrap().route(from);
        assert_eq!(whole, bounded);
        assert!(whole.len() > 40, "{}", whole.len());
    }

    #[test]
    fn the_way_round_a_wall_bends_south_and_ends_at_the_goal() {
        let g = ground(96.0);
        let way = g.way(at(200.0, 300.0), at(800.0, 300.0));
        assert!(way.len() >= 2, "{way:?}");
        let goal = *way.last().unwrap();
        assert!(goal.dist2d(at(800.0, 300.0)) < 16.0, "{goal:?}");
        // Every point stands, and some point is south of the wall's end (z > 768).
        assert!(way.iter().all(|p| g.passable_at(*p)), "{way:?}");
        assert!(way.iter().any(|p| p.z > 768.0), "the way goes round the south end: {way:?}");
        // Priced by room: the turn under the wall keeps the body's room (not hugging the wall's end).
        let under = way.iter().filter(|p| p.x > 400.0 && p.x < 600.0).map(|p| p.z).fold(0.0f32, f32::max);
        assert!(under > 768.0 + 60.0, "rounded with room, not at the lip: {way:?}");
    }

    #[test]
    fn a_goal_across_the_wall_with_no_way_is_the_straight_line() {
        let terrain = {
            let mut t = walled();
            // Close the gap: the wall runs the whole height.
            for z in 0..64 {
                for x in 30..34 {
                    t.slopes[z * 64 + x] = 255;
                }
            }
            t
        };
        let mut classes = Classes::default();
        let grid = classes.passable(&terrain, tank());
        let g = Ground::new(&terrain, &[grid], 96.0).unwrap();
        assert_eq!(g.way(at(200.0, 300.0), at(800.0, 300.0)), vec![at(800.0, 300.0)]);
        assert!(g.way_found(at(200.0, 300.0), at(800.0, 300.0)).is_none(), "the ground says it has none");
        assert!(g.way_found(at(200.0, 300.0), at(400.0, 300.0)).is_some(), "and a goal this side of the wall has one");
        assert!(!g.walk_within(at(200.0, 300.0), at(800.0, 300.0), 5000.0));
    }

    #[test]
    fn a_slot_on_the_cliff_snaps_beside_it_and_one_far_in_does_not() {
        let g = ground(96.0);
        // On the wall (x 480-544): the nearest ground is at its side.
        let s = g.snap(at(500.0, 300.0)).expect("beside the wall");
        assert!(g.passable_at(s) && (s.x < 480.0 || s.x >= 544.0), "{s:?}");
        // A lake wider than the snap distance either side.
        let terrain = {
            let mut t = walled();
            for z in 0..64 {
                for x in 0..64 {
                    if (x as i32 - 20).abs() <= 10 && (z as i32 - 40).abs() <= 10 {
                        t.heights[z * 64 + x] = -200;
                    }
                }
            }
            t
        };
        let mut classes = Classes::default();
        let grid = classes.passable(&terrain, tank());
        let g = Ground::new(&terrain, &[grid], 96.0).unwrap();
        assert!(g.snap(at(20.5 * 16.0, 40.5 * 16.0)).is_none());
    }

    #[test]
    fn the_walk_round_the_wall_is_found_within_a_cap_that_allows_it_and_not_one_that_does_not() {
        let g = ground(96.0);
        let (a, b) = (at(400.0, 300.0), at(620.0, 300.0));
        // Straight 220; round the wall's south end about 1,100.
        assert!(!g.walk_within(a, b, 2.0 * 220.0));
        assert!(g.walk_within(a, b, 1600.0));
        assert!(g.walk_within(a, at(400.0, 500.0), 2.0 * 200.0));
    }

    #[test]
    fn the_room_is_least_at_a_wall_and_the_common_ground_is_the_strictest_class() {
        let g = ground(96.0);
        assert!(g.room_at(at(470.0, 300.0)) < 32.0);
        assert!(g.room_at(at(200.0, 500.0)) > 150.0);
        let terrain = walled();
        let mut classes = Classes::default();
        let bot = MoveClass { kind: MoveKind::Bot, max_slope: 1.0, depth: 20.0, slope_mod: 0.0 };
        let both = Ground::new(&terrain, &[classes.passable(&terrain, bot), classes.passable(&terrain, tank())], 96.0).unwrap();
        assert!(!both.passable_at(at(500.0, 300.0)), "the tank cannot climb the wall, so the body cannot");
        let bots = Ground::new(&terrain, &[classes.passable(&terrain, bot)], 96.0).unwrap();
        assert!(bots.passable_at(at(500.0, 300.0)));
    }
}
