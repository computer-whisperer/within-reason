//! The ground as units experience it: where a movement class can go, and how far places are by foot rather than
//! as the crow flies. Quicksilver has cliffs and water between points a straight line joins; every rule that says
//! "nearer", "forward" or "our half" means walking distance (K-maps-terrain-not-straight-lines).

pub mod sheet;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use bot_protocol::{MoveClass, MoveKind, Terrain, Vec3};

/// Walking distances from the nearest of its origins for one movement class, over the terrain grid.
pub struct Field {
    cell: f32,
    width: usize,
    height: usize,
    /// Tenths of a cell; `u32::MAX` where the class cannot get to.
    cost: Vec<u32>,
}

const UNREACHABLE: u32 = u32::MAX;
/// A position is judged by the best cell this close to it: buildings and spots sit on ledges' edges and shorelines.
const SNAP_CELLS: i32 = 6;

/// A cell's cost of crossing for a class, in tenths of the flat rate: 10 on flat ground, more on a slope by the
/// engine's own law (`1 / (1 + slope * slope_mod)` of the speed, `GroundMoveMath.cpp`), [`IMPASSABLE`] where the
/// class cannot go. The unit of a field built on these is effective elmos: elmos at full speed.
pub const IMPASSABLE: u32 = u32::MAX;

pub fn costs(terrain: &Terrain, class: MoveClass) -> Vec<u32> {
    passable(terrain, class)
        .iter()
        .zip(&terrain.slopes)
        .map(|(&ok, &slope)| {
            if !ok {
                return IMPASSABLE;
            }
            let slope = f32::from(slope) / 255.0;
            let slower = 1.0 + slope * class.slope_mod.max(0.0);
            (10.0 * slower).round().clamp(10.0, 1_000_000.0) as u32
        })
        .collect()
}

/// Each cell's distance to the nearest impassable cell or the map's edge, in elmos (an exact Euclidean distance
/// transform, Felzenszwalb and Huttenlocher's, two separable passes over squared distances): how much room a body has
/// about a point. A formation of half-width `w` fits where the clearance is `w` or more.
pub fn clearance(terrain: &Terrain, passable: &[bool]) -> Vec<f32> {
    let (width, height) = (terrain.width as usize, terrain.height as usize);
    if width == 0 || passable.len() != width * height {
        return Vec::new();
    }
    const FAR: f32 = 1.0e12;
    // Columns first: for each cell the squared distance along its column to the nearest blocked cell.
    let mut sq = vec![FAR; width * height];
    let mut f = vec![0.0f32; height.max(width)];
    let mut d = vec![0.0f32; height.max(width)];
    for x in 0..width {
        for z in 0..height {
            f[z] = if passable[z * width + x] { FAR } else { 0.0 };
        }
        distance_transform_1d(&f[..height], &mut d[..height]);
        for z in 0..height {
            sq[z * width + x] = d[z];
        }
    }
    // Then rows, over the columns' result.
    for z in 0..height {
        f[..width].copy_from_slice(&sq[z * width..(z + 1) * width]);
        distance_transform_1d(&f[..width], &mut d[..width]);
        for x in 0..width {
            // The map's edge is a wall too.
            let to_edge = (x as f32 + 0.5).min(width as f32 - x as f32 - 0.5).min(z as f32 + 0.5).min(height as f32 - z as f32 - 0.5);
            sq[z * width + x] = d[x].min(to_edge * to_edge);
        }
    }
    sq.into_iter().map(|s| s.sqrt() * terrain.cell).collect()
}

/// One pass of the squared distance transform: `d[q] = min over p of (q - p)^2 + f[p]` (the lower envelope of the
/// parabolas rooted at each `p`).
fn distance_transform_1d(f: &[f32], d: &mut [f32]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut v = vec![0usize; n];
    let mut z = vec![0.0f32; n + 1];
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * (q as f32 - p as f32));
            if s <= z[k] && k > 0 {
                k -= 1;
                continue;
            }
            if s <= z[k] {
                // k == 0 and the new parabola is below everywhere left of it: it replaces the first.
                v[0] = q;
                z[0] = f32::NEG_INFINITY;
                z[1] = f32::INFINITY;
                break;
            }
            k += 1;
            v[k] = q;
            z[k] = s;
            z[k + 1] = f32::INFINITY;
            break;
        }
    }
    k = 0;
    for q in 0..n {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let p = v[k];
        d[q] = (q as f32 - p as f32).powi(2) + f[p];
    }
}

/// The flat rate where a cell's clearance is `wanted` or more, rising to `WALL_COST` at a wall, [`IMPASSABLE`] where
/// the class cannot go: a field over these keeps a body's route `wanted` from the walls where there is room (a corner
/// is rounded at that distance) and down the middle of a corridor narrower than that.
pub const WALL_COST: f32 = 60.0;
pub fn clearance_costs(passable: &[bool], clearance: &[f32], wanted: f32) -> Vec<u32> {
    passable
        .iter()
        .zip(clearance)
        .map(|(&ok, &room)| {
            if !ok {
                return IMPASSABLE;
            }
            if wanted <= 0.0 || room >= wanted {
                return 10;
            }
            (10.0 + (WALL_COST - 10.0) * (1.0 - room / wanted)).round() as u32
        })
        .collect()
}

pub fn passable(terrain: &Terrain, class: MoveClass) -> Vec<bool> {
    let max_slope = (class.max_slope * 255.0).round() as i32;
    terrain
        .heights
        .iter()
        .zip(&terrain.slopes)
        .map(|(&height, &slope)| {
            let height = f32::from(height);
            match class.kind {
                MoveKind::Tank | MoveKind::Bot => i32::from(slope) <= max_slope && height >= -class.depth,
                MoveKind::Hover => i32::from(slope) <= max_slope || height < 0.0,
                MoveKind::Ship => height <= -class.depth,
            }
        })
        .collect()
}

impl Field {
    /// Distances from `origin` for a class that can stand where `passable` says. `None` without terrain data.
    pub fn from(terrain: &Terrain, passable: &[bool], origin: Vec3) -> Option<Field> {
        Field::from_many(terrain, passable, &[origin])
    }

    /// Distances from whichever of `origins` is nearest on foot, flat ground at one rate everywhere. `None` when
    /// none of them is near passable ground.
    pub fn from_many(terrain: &Terrain, passable: &[bool], origins: &[Vec3]) -> Option<Field> {
        let costs: Vec<u32> = passable.iter().map(|&ok| if ok { 10 } else { IMPASSABLE }).collect();
        Field::from_costs(terrain, &costs, origins)
    }

    /// Effective elmos from whichever of `origins` is nearest, over a cost layer (`costs`: tenths of the flat rate
    /// a cell, [`IMPASSABLE`] where the class cannot go; a threat penalty is the caller's to add). A step costs its
    /// length times the mean of the two cells' rates. `None` when no origin is near passable ground.
    pub fn from_costs(terrain: &Terrain, costs: &[u32], origins: &[Vec3]) -> Option<Field> {
        Field::over(terrain.cell, terrain.width as usize, terrain.height as usize, costs, origins)
    }

    /// As `from_costs`, searched no farther than `limit` effective elmos: the ground beyond it is left unreached,
    /// the cheap form when only the near ground is asked about (a region's spots, each a field of its own).
    pub fn from_costs_within(terrain: &Terrain, costs: &[u32], origins: &[Vec3], limit: f32) -> Option<Field> {
        Self::search(terrain.cell, terrain.width as usize, terrain.height as usize, costs, origins, None, limit)
    }

    /// `from_costs` over a grid given by its dimensions (a caller that holds cost layers and no `Terrain`: the lane's
    /// group ground).
    pub fn over(cell: f32, width: usize, height: usize, costs: &[u32], origins: &[Vec3]) -> Option<Field> {
        Self::search(cell, width, height, costs, origins, None, f32::INFINITY)
    }

    /// `over`, stopped once the cell at `until` is settled (design D18): the route from there descends through cells
    /// settled before it (Dijkstra settles in cost order), so the field is exact for that route and the search is
    /// bounded by the way's cost rather than the map (a plan of five legs over All That Glitters took 92 ms whole).
    /// Cells beyond are unreachable to `distance` and `route`.
    pub fn over_until(cell: f32, width: usize, height: usize, costs: &[u32], origins: &[Vec3], until: Option<Vec3>) -> Option<Field> {
        Self::search(cell, width, height, costs, origins, until, f32::INFINITY)
    }

    /// The one search: Dijkstra from the origins over the cost layer, stopped once the cell at `until` is settled
    /// and never past `limit` effective elmos.
    fn search(cell: f32, width: usize, height: usize, costs: &[u32], origins: &[Vec3], until: Option<Vec3>, limit: f32) -> Option<Field> {
        if width == 0 || cell <= 0.0 || costs.len() != width * height {
            return None;
        }
        let passable = |index: usize| costs[index] != IMPASSABLE;
        let mut field = Field { cell, width, height, cost: vec![UNREACHABLE; width * height] };
        let bound = (f64::from(limit) / f64::from(cell) * 10.0).min(f64::from(UNREACHABLE - 1)) as u32;
        let mut queue = BinaryHeap::new();
        for origin in origins {
            if let Some(start) = field.nearest(*origin, passable) {
                field.cost[start] = 0;
                queue.push(Reverse((0u32, start)));
            }
        }
        if queue.is_empty() {
            return None;
        }
        let stop = until.and_then(|p| field.nearest(p, passable));
        while let Some(Reverse((cost, index))) = queue.pop() {
            if cost > field.cost[index] {
                continue;
            }
            if stop == Some(index) {
                break;
            }
            let (x, z) = ((index % width) as i32, (index / width) as i32);
            for (dx, dz, step) in [(1, 0, 10u64), (-1, 0, 10), (0, 1, 10), (0, -1, 10), (1, 1, 14), (1, -1, 14), (-1, 1, 14), (-1, -1, 14)] {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= width as i32 || nz >= height as i32 {
                    continue;
                }
                let next = nz as usize * width + nx as usize;
                if !passable(next) {
                    continue;
                }
                // No cutting corners between two blocked cells.
                let corner_clear = dx == 0 || dz == 0 || (passable(z as usize * width + nx as usize) && passable(nz as usize * width + x as usize));
                if !corner_clear {
                    continue;
                }
                let rate = (u64::from(costs[index]) + u64::from(costs[next])) / 2;
                let total = (u64::from(cost) + step * rate / 10).min(u64::from(UNREACHABLE - 1)) as u32;
                if total < field.cost[next] && total <= bound {
                    field.cost[next] = total;
                    queue.push(Reverse((total, next)));
                }
            }
        }
        Some(field)
    }

    fn cell_of(&self, pos: Vec3) -> (i32, i32) {
        ((pos.x / self.cell) as i32, (pos.z / self.cell) as i32)
    }

    fn centre(&self, index: usize) -> Vec3 {
        Vec3 { x: ((index % self.width) as f32 + 0.5) * self.cell, y: 0.0, z: ((index / self.width) as f32 + 0.5) * self.cell }
    }

    /// The cell nearest `pos`, within the snap distance, that `accept`s.
    fn nearest(&self, pos: Vec3, accept: impl Fn(usize) -> bool) -> Option<usize> {
        let (cx, cz) = self.cell_of(pos);
        let mut best: Option<(i32, usize)> = None;
        for dz in -SNAP_CELLS..=SNAP_CELLS {
            for dx in -SNAP_CELLS..=SNAP_CELLS {
                let (x, z) = (cx + dx, cz + dz);
                if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
                    continue;
                }
                let index = z as usize * self.width + x as usize;
                let apart = dx * dx + dz * dz;
                if accept(index) && best.is_none_or(|(d, _)| apart < d) {
                    best = Some((apart, index));
                }
            }
        }
        best.map(|(_, index)| index)
    }

    /// Whether the cell under `pos` itself can be reached from the origin, with no snap to ground nearby: a plateau
    /// the class cannot climb is not, though `distance` finds the ground at its foot (terr105: an advanced plant's
    /// only site on All That Glitters was a plateau 340 above the base, its builder held 235 off for fifteen minutes).
    pub fn reaches(&self, pos: Vec3) -> bool {
        let (x, z) = ((pos.x / self.cell).floor() as i32, (pos.z / self.cell).floor() as i32);
        x >= 0 && z >= 0 && (x as usize) < self.width && (z as usize) < self.height && self.cost[z as usize * self.width + x as usize] != UNREACHABLE
    }

    /// Walking distance from the origin to (the reachable ground nearest) `pos`, in elmos.
    pub fn distance(&self, pos: Vec3) -> Option<f32> {
        let index = self.nearest(pos, |i| self.cost[i] != UNREACHABLE)?;
        Some(self.cost[index] as f32 / 10.0 * self.cell)
    }

    /// The reachable ground nearest `pos`, if any is close.
    pub fn snap(&self, pos: Vec3) -> Option<Vec3> {
        self.nearest(pos, |i| self.cost[i] != UNREACHABLE).map(|index| self.centre(index))
    }

    /// The way from `from` down to the origin: the cells' centres, `from`'s end first, the origin last. Empty when
    /// `from` cannot reach the origin. This is the route a unit would walk at the field's rates; read by descending
    /// the field, no search.
    pub fn route(&self, from: Vec3) -> Vec<Vec3> {
        let Some(mut index) = self.nearest(from, |i| self.cost[i] != UNREACHABLE) else { return Vec::new() };
        let mut route = vec![self.centre(index)];
        while self.cost[index] > 0 {
            let (x, z) = ((index % self.width) as i32, (index / self.width) as i32);
            let Some(downhill) = (-1..=1)
                .flat_map(|dz| (-1..=1).map(move |dx| (x + dx, z + dz)))
                .filter(|&(nx, nz)| nx >= 0 && nz >= 0 && nx < self.width as i32 && nz < self.height as i32)
                .map(|(nx, nz)| nz as usize * self.width + nx as usize)
                .min_by_key(|&next| self.cost[next])
            else {
                break;
            };
            if self.cost[downhill] >= self.cost[index] {
                break;
            }
            index = downhill;
            route.push(self.centre(index));
        }
        route
    }

    /// Whether the way from `from` to the origin passes a point `bad` says is dangerous.
    pub fn route_crosses(&self, from: Vec3, bad: impl Fn(Vec3) -> bool) -> bool {
        self.route(from).into_iter().any(bad)
    }

    /// The point `along` elmos from the origin on the way to `goal`; the goal itself if it is nearer than that.
    pub fn towards(&self, goal: Vec3, along: f32) -> Option<Vec3> {
        let mut index = self.nearest(goal, |i| self.cost[i] != UNREACHABLE)?;
        let wanted = (along / self.cell * 10.0) as u32;
        while self.cost[index] > wanted {
            let (x, z) = ((index % self.width) as i32, (index / self.width) as i32);
            let downhill = (-1..=1)
                .flat_map(|dz| (-1..=1).map(move |dx| (x + dx, z + dz)))
                .filter(|&(nx, nz)| nx >= 0 && nz >= 0 && nx < self.width as i32 && nz < self.height as i32)
                .map(|(nx, nz)| nz as usize * self.width + nx as usize)
                .min_by_key(|&next| self.cost[next])?;
            if self.cost[downhill] >= self.cost[index] {
                break;
            }
            index = downhill;
        }
        Some(self.centre(index))
    }
}

/// A passage on the way between two places: where it is and how wide.
#[derive(Clone, Debug)]
pub struct Passage {
    pub at: Vec3,
    pub width: f32,
    /// How far along the way from the first place it lies, 0 to 1.
    pub along: f32,
}

/// The narrow places on the ways between the origins of `from` and `to`: what a defender holds and an attacker must
/// force. The corridor is every cell on a route at most [`DETOUR`] times the shortest; it is cut into bands by distance
/// from `from`, each band falls into connected pieces (one per parallel route), and a piece much narrower than the
/// corridor's usual width is a passage. Each route's narrowest piece is reported, the narrowest first.
pub fn passages(from: &Field, to: &Field) -> Vec<Passage> {
    const DETOUR: f32 = 1.3;
    /// Depth of a band, in cells.
    const BAND: u32 = 4;
    let cells = from.cost.len();
    let sum = |i: usize| (from.cost[i] != UNREACHABLE && to.cost[i] != UNREACHABLE).then(|| from.cost[i] + to.cost[i]);
    let Some(shortest) = (0..cells).filter_map(sum).min() else { return Vec::new() };
    let limit = (shortest as f32 * DETOUR) as u32;
    let in_corridor = |i: usize| sum(i).is_some_and(|s| s <= limit);
    let band_of = |i: usize| from.cost[i] / (BAND * 10);
    let mut seen = vec![false; cells];
    // (band, cells in the piece, sum x, sum z, the cells)
    let mut pieces: Vec<(u32, usize, f32, f32, Vec<usize>)> = Vec::new();
    for start in 0..cells {
        if seen[start] || !in_corridor(start) {
            continue;
        }
        let band = band_of(start);
        let (mut count, mut x, mut z) = (0usize, 0.0, 0.0);
        let mut members: Vec<usize> = Vec::new();
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            let centre = from.centre(i);
            (count, x, z) = (count + 1, x + centre.x, z + centre.z);
            members.push(i);
            let (cx, cz) = ((i % from.width) as i32, (i / from.width) as i32);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if nx < 0 || nz < 0 || nx >= from.width as i32 || nz >= from.height as i32 {
                    continue;
                }
                let next = nz as usize * from.width + nx as usize;
                if !seen[next] && in_corridor(next) && band_of(next) == band {
                    seen[next] = true;
                    stack.push(next);
                }
            }
        }
        pieces.push((band, count, x, z, members));
    }
    // Slivers where a band (a ring round the first place) meets the corridor's edge are not routes. A piece counts when
    // it is a fifth of its band or more. The price: a narrow side pass level with a wide one is not reported (tried
    // without the share rule on three maps: 50-100 wide slivers beside every real pass).
    let mut band_cells: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for piece in &pieces {
        *band_cells.entry(piece.0).or_default() += piece.1;
    }
    pieces.retain(|p| p.1 >= 3 * BAND as usize && p.1 * 5 >= band_cells[&p.0]);
    // Away from both ends, where the corridor narrows to a point by construction.
    let bands = shortest / (BAND * 10);
    let middle: Vec<&(u32, usize, f32, f32, Vec<usize>)> = pieces.iter().filter(|p| p.0 * 5 >= bands && p.0 * 5 <= bands * 4).collect();
    // A narrow piece is a way only when the route goes on from it: the ground beyond it (later bands, flooded from
    // its cells) comes nearer to the far end than the piece itself. The neck of a dead-end pocket beside the route
    // is narrow too, and everything beyond it is farther from the end (Great Divide from the south: the finder
    // named the two cut-off pockets' necks at C5 and G5 and not the one pass at E5, and the player's first packet
    // was built on them). Several ways may lead on side by side (Quicksilver names three).
    let leads_on = |piece: &(u32, usize, f32, f32, Vec<usize>)| -> bool {
        let nearest = piece.4.iter().map(|i| to.cost[*i]).min().unwrap_or(UNREACHABLE);
        let mut reached = vec![false; cells];
        let mut stack: Vec<usize> = piece.4.clone();
        for i in &stack {
            reached[*i] = true;
        }
        while let Some(i) = stack.pop() {
            // Nearer by a band's depth at least: a pocket four cells deep is no way either.
            if to.cost[i] + BAND * 10 <= nearest {
                return true;
            }
            let (cx, cz) = ((i % from.width) as i32, (i / from.width) as i32);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if nx < 0 || nz < 0 || nx >= from.width as i32 || nz >= from.height as i32 {
                    continue;
                }
                let next = nz as usize * from.width + nx as usize;
                if !reached[next] && in_corridor(next) && band_of(next) >= piece.0 && !piece.4.contains(&next) {
                    reached[next] = true;
                    stack.push(next);
                }
            }
        }
        false
    };
    let mut widths: Vec<usize> = middle.iter().map(|p| p.1).collect();
    widths.sort_unstable();
    let Some(usual) = widths.get(widths.len() / 2).copied() else { return Vec::new() };
    #[cfg(test)]
    if std::env::var_os("WR_PASSAGE_DEBUG").is_some() {
        eprintln!("usual {usual}");
        for p in &middle {
            eprintln!("piece band {} cells {} at ({:.0},{:.0}) leads on {}", p.0, p.1, p.2 / p.1 as f32 / from.cell, p.3 / p.1 as f32 / from.cell, leads_on(p));
        }
    }
    let mut narrow: Vec<Passage> = middle
        .iter()
        .filter(|p| p.1 * 2 <= usual && leads_on(p))
        .map(|p| Passage {
            at: Vec3 { x: p.2 / p.1 as f32, y: 0.0, z: p.3 / p.1 as f32 },
            width: p.1 as f32 / BAND as f32 * from.cell,
            along: p.0 as f32 / bands.max(1) as f32,
        })
        .collect();
    narrow.sort_by(|a, b| a.width.total_cmp(&b.width));
    // One per place: neighbouring bands of the same gap say the same thing.
    let mut chosen: Vec<Passage> = Vec::new();
    for passage in narrow {
        if chosen.iter().all(|c| c.at.dist2d(passage.at) > 600.0) {
            chosen.push(passage);
        }
    }
    chosen.truncate(4);
    chosen
}

/// A coarse picture of the map in text, `size` characters square, for a reader that cannot see it: `~` water,
/// `#` ground this class cannot stand on (cliffs), `x` ground it could stand on but cannot walk to from the field's
/// origin, and `.` `o` `O` walkable ground by height (low, middle, high thirds).
pub fn sketch(terrain: &Terrain, passable: &[bool], field: &Field, size: usize) -> Vec<String> {
    let (width, height) = (terrain.width as usize, terrain.height as usize);
    let top = terrain.heights.iter().copied().max().unwrap_or(1).max(1) as f32;
    let (step_x, step_z) = (width.div_ceil(size), height.div_ceil(size));
    (0..size)
        .map(|row| {
            (0..size)
                .map(|column| {
                    let (mut water, mut blocked, mut cut_off, mut walkable, mut height_sum) = (0, 0, 0, 0, 0.0);
                    for z in (row * step_z)..((row + 1) * step_z).min(height) {
                        for x in (column * step_x)..((column + 1) * step_x).min(width) {
                            let index = z * width + x;
                            if terrain.heights[index] < 0 {
                                water += 1;
                            } else if !passable[index] {
                                blocked += 1;
                            } else if field.cost[index] == UNREACHABLE {
                                cut_off += 1;
                            } else {
                                walkable += 1;
                                height_sum += f32::from(terrain.heights[index]);
                            }
                        }
                    }
                    // A cell is what most of it is, except that any real share of cliff shows: cliffs are thin.
                    let land = blocked + cut_off + walkable;
                    if water > land {
                        '~'
                    } else if blocked * 3 >= land {
                        '#'
                    } else if cut_off > walkable {
                        'x'
                    } else {
                        match height_sum / walkable.max(1) as f32 / top {
                            t if t < 0.33 => '.',
                            t if t < 0.66 => 'o',
                            _ => 'O',
                        }
                    }
                })
                .collect()
        })
        .collect()
}


#[cfg(test)]
mod weighted {
    use super::*;

    /// The field stopped at a cell gives that cell the whole field's route and distance: a settled cell's route
    /// descends through cells settled before it (design D18). Pseudo-random walls and prices, many pairs.
    #[test]
    fn the_field_stopped_at_its_start_routes_as_the_whole_field_does() {
        let (w, h) = (96usize, 64usize);
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut compared = 0;
        for _ in 0..6 {
            let costs: Vec<u32> = (0..w * h)
                .map(|_| {
                    let r = next() % 100;
                    if r < 18 { IMPASSABLE } else if r < 60 { 10 } else { 10 + (r as u32 - 60) * 2 }
                })
                .collect();
            for _ in 0..40 {
                let p = |n: u64| Vec3 { x: ((n % w as u64) as f32 + 0.5) * 16.0, y: 0.0, z: (((n / w as u64) % h as u64) as f32 + 0.5) * 16.0 };
                let (from, to) = (p(next()), p(next()));
                let Some(whole) = Field::over(16.0, w, h, &costs, &[to]) else { continue };
                let bounded = Field::over_until(16.0, w, h, &costs, &[to], Some(from)).unwrap();
                let (a, b) = (whole.route(from), bounded.route(from));
                if a.is_empty() {
                    continue;
                }
                assert_eq!(a, b, "from {from:?} to {to:?}");
                assert_eq!(whole.distance(from), bounded.distance(from));
                compared += 1;
            }
        }
        assert!(compared > 100, "{compared}");
    }

    fn flat(width: u32, height: u32) -> Terrain {
        Terrain { cell: 16.0, width, height, heights: vec![50; (width * height) as usize], metal: Vec::new(), slopes: vec![0; (width * height) as usize] }
    }

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// The clearance is the distance to the nearest wall or edge; a route priced by it keeps off a corner.
    #[test]
    fn the_clearance_prices_the_walls_and_a_route_rounds_a_corner_at_a_distance() {
        // A 40 x 40 flat map (16-elmo cells) with a wall across the middle from the west edge to column 24.
        let terrain = flat(40, 40);
        let mut passable = vec![true; 1600];
        for x in 0..24 {
            passable[20 * 40 + x] = false;
        }
        let room = clearance(&terrain, &passable);
        // Beside the wall, one cell: 16; three cells up, 48; the far corner of the map, half a cell from two edges: 8.
        assert!((room[19 * 40 + 10] - 16.0).abs() < 0.01, "{}", room[19 * 40 + 10]);
        assert!((room[17 * 40 + 10] - 48.0).abs() < 0.01, "{}", room[17 * 40 + 10]);
        assert!((room[39 * 40 + 39] - 8.0).abs() < 0.01, "{}", room[39 * 40 + 39]);
        // The wall's end at column 24: the cell diagonal from it has root-two cells of room.
        assert!((room[19 * 40 + 24] - 16.0 * 2f32.sqrt()).abs() < 0.01, "{}", room[19 * 40 + 24]);
        // Costs: flat where the room is 64 or more, 60 at the wall itself's neighbour (room 16: 10 + 50 * 0.75).
        let costs = clearance_costs(&passable, &room, 64.0);
        assert_eq!(costs[10 * 40 + 10], 10);
        assert_eq!(costs[19 * 40 + 10], 48);
        assert_eq!(costs[20 * 40 + 10], IMPASSABLE);
        // From south of the wall to north of it round its end: the plain route hugs the end (passes within a cell of
        // it), the priced route keeps about the wanted room from it.
        let (from, to) = (at(8.0 * 16.0 + 8.0, 30.0 * 16.0 + 8.0), at(8.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0));
        let plain = Field::from_many(&terrain, &passable, &[to]).unwrap().route(from);
        let priced = Field::from_costs(&terrain, &costs, &[to]).unwrap().route(from);
        let end = at(23.0 * 16.0 + 8.0, 20.0 * 16.0 + 8.0);
        let nearest = |route: &[Vec3]| route.iter().map(|p| p.dist2d(end)).fold(f32::INFINITY, f32::min);
        assert!(nearest(&plain) < 24.0, "{}", nearest(&plain));
        assert!(nearest(&priced) > 48.0, "{}", nearest(&priced));
        assert!(!priced.is_empty() && priced.last().unwrap().dist2d(to) < 12.0);
    }

    #[test]
    fn a_slope_costs_what_the_engine_slows_a_unit_by() {
        let mut terrain = flat(32, 4);
        // A band of slope 0.2 across the middle columns; a class slowed by slope_mod 4: 1 + 0.2 * 4 = 1.8 times.
        for z in 0..4 {
            for x in 10..20 {
                terrain.slopes[z * 32 + x] = 51;
            }
        }
        let class = MoveClass { kind: MoveKind::Bot, max_slope: 0.5, depth: 20.0, slope_mod: 4.0 };
        let costs = costs(&terrain, class);
        assert_eq!(costs[5], 10);
        assert_eq!(costs[15], 18);
        let field = Field::from_costs(&terrain, &costs, &[at(8.0, 24.0)]).unwrap();
        let flat_distance = field.distance(at(8.0 + 9.0 * 16.0, 24.0)).unwrap();
        let over_the_band = field.distance(at(8.0 + 20.0 * 16.0, 24.0)).unwrap();
        assert!((flat_distance - 144.0).abs() < 1.0, "{flat_distance}");
        // Twenty steps, ten of them through the band at 1.8 (the two edge steps at the mean rate, 1.4): 448.
        assert!((over_the_band - 448.0).abs() < 1.0, "{over_the_band}");
    }

    #[test]
    fn a_route_goes_round_a_wall_and_says_so() {
        let terrain = flat(32, 32);
        let class = MoveClass { kind: MoveKind::Bot, max_slope: 0.5, depth: 20.0, slope_mod: 0.0 };
        let mut costs = costs(&terrain, class);
        // A wall down column 16 with a gap at the bottom.
        for z in 0..28 {
            costs[z * 32 + 16] = IMPASSABLE;
        }
        let field = Field::from_costs(&terrain, &costs, &[at(24.0, 24.0)]).unwrap();
        let route = field.route(at(24.0 + 30.0 * 16.0, 24.0));
        assert!(route.len() > 40, "goes round: {} cells", route.len());
        assert!(route.iter().any(|p| p.z > 28.0 * 16.0), "through the gap at the bottom");
        assert_eq!(route.last().map(|p| (p.x, p.z)), Some((24.0, 24.0)));
        assert!(field.route_crosses(at(24.0 + 30.0 * 16.0, 24.0), |p| p.z > 28.0 * 16.0));
        assert!(!field.route_crosses(at(24.0 + 30.0 * 16.0, 24.0), |p| p.z < 0.0));
    }

    #[test]
    fn a_walled_pocket_is_not_reached_though_its_distance_snaps_to_the_foot() {
        let terrain = flat(16, 16);
        let mut passable = vec![true; 16 * 16];
        // A ring of cliff round cell (10, 10): the pocket inside is flat ground nothing walks to.
        for (x, z) in (9..=11).flat_map(|x| (9..=11).map(move |z| (x, z))).filter(|&(x, z)| (x, z) != (10, 10)) {
            passable[z * 16 + x] = false;
        }
        let field = Field::from(&terrain, &passable, at(24.0, 24.0)).unwrap();
        let pocket = at(10.5 * 16.0, 10.5 * 16.0);
        assert!(!field.reaches(pocket));
        assert!(field.distance(pocket).is_some(), "distance takes the reachable ground beside it");
        assert!(field.reaches(at(5.5 * 16.0, 10.5 * 16.0)));
        assert!(!field.reaches(at(-8.0, 24.0)), "off the map");
    }

    #[test]
    fn a_bounded_field_stops_at_its_limit_and_agrees_inside_it() {
        let terrain = flat(64, 4);
        let costs = vec![10u32; 64 * 4];
        let origin = at(8.0, 8.0);
        let full = Field::from_costs(&terrain, &costs, &[origin]).unwrap();
        let near = Field::from_costs_within(&terrain, &costs, &[origin], 300.0).unwrap();
        let (inside, outside) = (at(16.0 * 15.5, 8.0), at(16.0 * 40.5, 8.0));
        assert_eq!(near.distance(inside), full.distance(inside));
        assert!(full.reaches(outside) && !near.reaches(outside) && near.distance(outside).is_none());
    }
}

#[cfg(test)]
mod field_timing {
    use super::*;

    /// How long a field over a Quicksilver-sized grid takes: one per metal spot is the plan
    /// (`docs/design/2026-09-20-micro-lane.md`, section 4).
    #[test]
    #[ignore]
    fn a_field_over_a_quicksilver_sized_grid() {
        let (width, height) = (448u32, 448u32);
        let terrain = Terrain { cell: 16.0, width, height, heights: vec![50; (width * height) as usize], metal: Vec::new(), slopes: vec![0; (width * height) as usize] };
        let passable = vec![true; (width * height) as usize];
        let started = std::time::Instant::now();
        for i in 0..44 {
            let origin = Vec3 { x: (i * 150) as f32, y: 0.0, z: (i * 100) as f32 };
            assert!(Field::from(&terrain, &passable, origin).is_some());
        }
        eprintln!("44 fields: {:.0} ms", started.elapsed().as_secs_f64() * 1000.0);
        let costs = vec![10u32; (width * height) as usize];
        let started = std::time::Instant::now();
        for i in 0..44 {
            let origin = Vec3 { x: (i * 150) as f32, y: 0.0, z: (i * 100) as f32 };
            assert!(Field::from_costs_within(&terrain, &costs, &[origin], 1100.0).is_some());
        }
        eprintln!("44 fields within 1,100: {:.0} ms", started.elapsed().as_secs_f64() * 1000.0);
    }
}

#[cfg(test)]
mod passage_tests {
    use super::*;

    /// A 60 by 40 map: a wall five cells thick down columns 28-32 with one gap four cells tall, and a dead-end
    /// pocket south of the route on the near side, entered through a neck three cells wide and four deep. The gap is
    /// the passage; the neck is as narrow, and is not.
    fn map() -> (bot_protocol::Terrain, Vec<u32>) {
        let (w, h) = (60usize, 40usize);
        let mut costs = vec![10u32; w * h];
        let mut block = |x: usize, z: usize| costs[z * w + x] = IMPASSABLE;
        for x in 28..33 {
            for z in 0..h {
                if !(18..22).contains(&z) {
                    block(x, z);
                }
            }
        }
        // The neck rows 26-29 at columns 12-14; the pocket rows 30-39, columns 8-21; walls round both.
        for x in 0..28 {
            for z in 26..30 {
                if !(12..15).contains(&x) {
                    block(x, z);
                }
            }
        }
        for z in 30..h {
            for x in 0..8 {
                block(x, z);
            }
            for x in 22..w {
                block(x, z);
            }
        }
        let terrain = bot_protocol::Terrain { cell: 32.0, width: w as u32, height: h as u32, heights: vec![10; w * h], slopes: vec![0; w * h], metal: vec![0; w * h] };
        (terrain, costs)
    }

    /// The finder on a recorded game's terrain, by hand: `WR_TERRAIN=<terrain-N.bin> WR_SIZE=192x256 WR_HOME=x,z
    /// WR_ENEMY=x,z [WR_KIND=bot|tank] cargo test -p terrain passages_of_a_recorded_map -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn passages_of_a_recorded_map() {
        let path = std::env::var("WR_TERRAIN").expect("WR_TERRAIN");
        let size = std::env::var("WR_SIZE").unwrap_or_else(|_| "192x256".into());
        let (w, h): (usize, usize) = { let mut it = size.split('x').map(|v| v.parse::<usize>().unwrap()); (it.next().unwrap(), it.next().unwrap()) };
        let xz = |var: &str| { let v = std::env::var(var).unwrap(); let mut it = v.split(',').map(|n| n.parse::<f32>().unwrap()); Vec3 { x: it.next().unwrap(), y: 0.0, z: it.next().unwrap() } };
        let bytes = std::fs::read(&path).unwrap();
        let n = w * h;
        let heights: Vec<i16> = (0..n).map(|i| i16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]])).collect();
        let slopes: Vec<u8> = bytes[2 * n..3 * n].to_vec();
        let terrain = bot_protocol::Terrain { cell: 16.0, width: w as u32, height: h as u32, heights, slopes, metal: vec![0; n] };
        let kind = if std::env::var("WR_KIND").as_deref() == Ok("tank") { bot_protocol::MoveKind::Tank } else { bot_protocol::MoveKind::Bot };
        let class = bot_protocol::MoveClass { kind, max_slope: if matches!(kind, bot_protocol::MoveKind::Tank) { 0.2 } else { 0.4122 }, depth: 20.0, slope_mod: 1.0 };
        let passable = passable(&terrain, class);
        let from = Field::from(&terrain, &passable, xz("WR_HOME")).unwrap();
        let to = Field::from(&terrain, &passable, xz("WR_ENEMY")).unwrap();
        for p in passages(&from, &to) {
            eprintln!("passage at ({:.0}, {:.0}) width {:.0} along {:.2}", p.at.x, p.at.z, p.width, p.along);
        }
    }

    #[test]
    fn a_passage_cuts_the_corridor_and_a_pockets_neck_does_not() {
        let (terrain, costs) = map();
        let at = |x: f32, z: f32| Vec3 { x: x * 32.0, y: 0.0, z: z * 32.0 };
        let from = Field::from_costs(&terrain, &costs, &[at(2.5, 20.5)]).unwrap();
        let to = Field::from_costs(&terrain, &costs, &[at(57.5, 20.5)]).unwrap();
        let found = passages(&from, &to);
        assert_eq!(found.len(), 1, "{found:?}");
        let gap = &found[0];
        assert!((gap.at.x - 30.5 * 32.0).abs() < 3.0 * 32.0 && (gap.at.z - 20.0 * 32.0).abs() < 3.0 * 32.0, "{gap:?}");
        // From the far side the same gap, and nothing else.
        let back = passages(&to, &from);
        assert_eq!(back.len(), 1, "{back:?}");
    }
}
