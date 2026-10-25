//! Where duels are fought: flat, dry rectangles every listed unit can cross, far from the commanders and from
//! each other. The long side runs west-east: spawned units always face south (the engine's cheat gives no choice),
//! so on a west-east axis both sides start side-on to the enemy and neither has its back turned.

use bot_protocol::{Terrain, Vec3};
use terrain::Field;

/// Distance between the two armies' front ranks' centres; beyond every tier-1 weapon's range (artillery: ~710).
pub const SEPARATION: f32 = 1100.0;
/// Least rectangle checked for flatness: the separation plus 170 behind each front rank, and 640 across. A batch whose
/// formations are deeper or wider gets a rectangle that holds them (`Footprint::holding`).
const LENGTH: f32 = 1440.0;
const WIDTH: f32 = 640.0;
/// Ground checked beyond a formation's last rank and outside its outer files.
const MARGIN: f32 = 120.0;
/// Two sites' rectangles stay this far apart: a team shares sight between its duels, and artillery reaches ~710.
const SITE_GAP: f32 = 900.0;
/// No site this close to a commander, who shoots at what comes near and whose death ends the game; tier-1 units
/// see about 500 elmos.
pub const COMMANDER_GAP: f32 = 900.0;
/// Most the ground may rise or fall within a site, in elmos (height lends range to ballistic weapons).
pub const MAX_RELIEF: i32 = 16;
/// A drill's strip may rise or fall this much: a walk needs ground every listed unit can cross (the slope check
/// stands), not a firing range's flatness; a flat map's 3,840-elmo strip at 16 found no site (drill-smoke, 2026-10-24).
pub const DRILL_RELIEF: i32 = 48;
/// Candidate rectangles are tried every this many cells.
const STRIDE: usize = 4;

#[derive(Clone, Debug)]
pub struct Site {
    pub centre: Vec3,
    /// A drill's chosen ends (start, goal) when the site is not a strip through `centre`: `--from/--to`, or a
    /// detour found by `choose_detour` (design D10). The walk between them on foot is `walk`.
    pub ends: Option<(Vec3, Vec3)>,
    pub walk: Option<f32>,
    /// A circuit drill's corners in order, the lab's first, and the lap walked on foot between them (`circuit`,
    /// design D12: the field's route from each corner to the next, thinned to every `ROUTE_STEP`, the corners
    /// themselves in it, the lab's corner last); `walk` is then one lap on foot.
    pub circuit: Vec<Vec3>,
    pub circuit_route: Vec<Vec3>,
}

/// The rectangle a site must offer: `length` along the west-east axis, `width` across it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub length: f32,
    pub width: f32,
    /// Most the ground may rise or fall within it, in elmos.
    pub relief: i32,
}

impl Footprint {
    /// The least rectangle that holds both armies of every duel named: `(formation, count)` per army.
    pub fn holding(armies: impl IntoIterator<Item = (Formation, u32)>) -> Footprint {
        let mut footprint = Footprint { length: LENGTH, width: WIDTH, relief: MAX_RELIEF };
        for (formation, count) in armies {
            let (across, depth) = formation.extent(count);
            footprint.length = footprint.length.max(SEPARATION + 2.0 * (depth + MARGIN));
            footprint.width = footprint.width.max(across + 2.0 * MARGIN);
        }
        footprint
    }
}

impl Footprint {
    /// The side of the flat square a drill's end needs off the strip (design D10): the formation's extent either way
    /// plus the margin on both sides.
    pub fn drill_square(formation: Formation, count: u32) -> f32 {
        let (across, depth) = formation.extent(count);
        across.max(depth) + 2.0 * MARGIN
    }

    /// The strip a drill walks (`drill.rs`): `distance` from the start to the goal, the formation's depth and a margin
    /// behind the start, the arrival radius and a margin beyond the goal, and the formation's width across. Comet
    /// Catcher's longest dry passable strip of 640 is 1,920 and Quicksilver's 3,200 (drill-smoke, 2026-10-24), so the
    /// standard walk is 2,000 on Quicksilver.
    pub fn strip(distance: f32, formation: Formation, count: u32) -> Footprint {
        let (across, depth) = formation.extent(count);
        Footprint { length: (distance + depth + 2.0 * MARGIN + 300.0).max(LENGTH), width: (across + 2.0 * MARGIN).max(WIDTH), relief: DRILL_RELIEF }
    }
}

/// How an army is spawned and led: its shape, and the elmos between neighbours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Formation {
    pub kind: FormationKind,
    pub spacing: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormationKind {
    /// Ranks of this many across the approach, further ranks behind; the whole army attack-moves at the enemy's
    /// centre, so it closes into a ball (what the tables before 2026-09-24 were made with, at eight).
    Ranks(u32),
    /// One rank across the approach; each unit attack-moves at the enemy's centre shifted by its own place in the
    /// rank, so the line advances as a line.
    Line,
}

impl Formation {
    /// `ranks8`, `ranks:8`, `line`, each optionally with `@SPACING`; `spacing` when none is given.
    pub fn parse(text: &str, spacing: f32) -> Option<Formation> {
        let (shape, spacing) = match text.split_once('@') {
            Some((shape, s)) => (shape, s.parse().ok()?),
            None => (text, spacing),
        };
        let kind = match shape {
            "line" => FormationKind::Line,
            _ => FormationKind::Ranks(shape.strip_prefix("ranks")?.trim_start_matches(':').parse().ok().filter(|&n| n > 0)?),
        };
        Some(Formation { kind, spacing })
    }

    pub fn label(&self) -> String {
        match self.kind {
            FormationKind::Ranks(n) => format!("ranks{n}@{:.0}", self.spacing),
            FormationKind::Line => format!("line@{:.0}", self.spacing),
        }
    }

    fn per_rank(&self, count: u32) -> u32 {
        match self.kind {
            FormationKind::Ranks(n) => n,
            FormationKind::Line => count.max(1),
        }
    }

    /// Width across the approach and depth behind the front rank, centre to centre.
    pub fn extent(&self, count: u32) -> (f32, f32) {
        let per_rank = self.per_rank(count);
        let ranks = count.div_ceil(per_rank).max(1);
        ((count.min(per_rank).max(1) - 1) as f32 * self.spacing, (ranks - 1) as f32 * self.spacing)
    }

    /// Positions for `count` units: ranks run north-south through `front`, further ranks stack away from the enemy.
    pub fn places(&self, front: Vec3, faces_east: bool, count: u32) -> Vec<Vec3> {
        self.places_along(front, if faces_east { (1.0, 0.0) } else { (-1.0, 0.0) }, count)
    }

    /// Positions for `count` units facing `heading` (any non-zero (x, z)): ranks run across it through `front`, further
    /// ranks stack behind.
    pub fn places_along(&self, front: Vec3, heading: (f32, f32), count: u32) -> Vec<Vec3> {
        let per_rank = self.per_rank(count);
        let spacing = self.spacing;
        let norm = (heading.0 * heading.0 + heading.1 * heading.1).sqrt().max(1e-6);
        let (hx, hz) = (heading.0 / norm, heading.1 / norm);
        // Across the heading, to the right of it on the map's axes.
        let (ax, az) = (-hz, hx);
        (0..count)
            .map(|i| {
                let (rank, file) = (i / per_rank, i % per_rank);
                let in_rank = (count - rank * per_rank).min(per_rank);
                let back = -(rank as f32) * spacing;
                let side = (file as f32 - (in_rank - 1) as f32 / 2.0) * spacing;
                Vec3 { x: front.x + hx * back + ax * side, y: front.y, z: front.z + hz * back + az * side }
            })
            .collect()
    }
}

impl Site {
    /// Centre of the front rank at the west (`true`) or east end.
    pub fn end(&self, west: bool) -> Vec3 {
        let dx = if west { -SEPARATION / 2.0 } else { SEPARATION / 2.0 };
        Vec3 { x: self.centre.x + dx, ..self.centre }
    }
}

/// Up to `wanted` sites of `footprint`. `max_slope` is the engine slope value (0-1) the least agile listed unit manages.
pub fn choose(terrain: &Terrain, max_slope: f32, commanders: &[Vec3], wanted: usize, footprint: Footprint) -> Vec<Site> {
    let Footprint { length, width: across, relief } = footprint;
    let (width, height) = (terrain.width as usize, terrain.height as usize);
    if width == 0 || terrain.heights.len() != width * height {
        return Vec::new();
    }
    let (length_cells, width_cells) = ((length / terrain.cell) as usize, (across / terrain.cell) as usize);
    let slope_limit = (max_slope * 255.0) as u8;
    let mut candidates = Vec::new();
    for z0 in (0..height.saturating_sub(width_cells)).step_by(STRIDE) {
        'candidate: for x0 in (0..width.saturating_sub(length_cells)).step_by(STRIDE) {
            let (mut low, mut high) = (i32::MAX, i32::MIN);
            for z in z0..z0 + width_cells {
                for x in x0..x0 + length_cells {
                    let index = z * width + x;
                    let h = i32::from(terrain.heights[index]);
                    if terrain.slopes[index] > slope_limit || h < 5 {
                        continue 'candidate;
                    }
                    (low, high) = (low.min(h), high.max(h));
                    if high - low > relief {
                        continue 'candidate;
                    }
                }
            }
            let centre = Vec3 {
                x: (x0 as f32 + length_cells as f32 / 2.0) * terrain.cell,
                y: (low + high) as f32 / 2.0,
                z: (z0 as f32 + width_cells as f32 / 2.0) * terrain.cell,
            };
            if commanders.iter().all(|c| rect_distance(centre, *c, footprint) >= COMMANDER_GAP) {
                candidates.push((high - low, centre));
            }
        }
    }
    // Greedy packing finds a different number of sites depending on where it starts, so try a few orders
    // (flattest first, then sweeps across the map) and keep the fullest result.
    let orders: [fn(&(i32, Vec3)) -> f32; 7] = [
        |c| c.0 as f32,
        |c| c.1.x,
        |c| c.1.z,
        |c| -c.1.x,
        |c| -c.1.z,
        |c| c.1.x + c.1.z,
        |c| c.1.x - c.1.z,
    ];
    let mut best: Vec<Site> = Vec::new();
    for key in orders {
        candidates.sort_by(|a, b| key(a).total_cmp(&key(b)));
        let mut sites: Vec<Site> = Vec::new();
        for (_, centre) in &candidates {
            let clear = sites.iter().all(|s| {
                (s.centre.x - centre.x).abs() >= length + SITE_GAP || (s.centre.z - centre.z).abs() >= across + SITE_GAP
            });
            if clear && sites.len() < wanted {
                sites.push(Site { centre: *centre, ends: None, walk: None, circuit: Vec::new(), circuit_route: Vec::new() });
            }
        }
        if sites.len() > best.len() {
            best = sites;
        }
    }
    best
}

/// Ground a unit of the batch can stand on: dry (5 or more above the water) and no steeper than `max_slope`, per
/// cell, as `choose` judges a site's cells. The walking-distance fields of a detour are built over it.
pub fn standable(terrain: &Terrain, max_slope: f32) -> Vec<bool> {
    let slope_limit = (max_slope * 255.0) as u8;
    terrain.heights.iter().zip(&terrain.slopes).map(|(&h, &s)| s <= slope_limit && i32::from(h) >= 5).collect()
}

/// Whether the square of `side` elmos centred on cell (`cx`, `cz`) is standable throughout and rises no more than
/// `relief`.
fn flat_square(terrain: &Terrain, standable: &[bool], cx: i32, cz: i32, side: f32, relief: i32) -> bool {
    let half = (side / 2.0 / terrain.cell).ceil() as i32;
    let (width, height) = (terrain.width as i32, terrain.height as i32);
    if cx - half < 0 || cz - half < 0 || cx + half >= width || cz + half >= height {
        return false;
    }
    let (mut low, mut high) = (i32::MAX, i32::MIN);
    for z in cz - half..=cz + half {
        for x in cx - half..=cx + half {
            let index = (z * width + x) as usize;
            if !standable[index] {
                return false;
            }
            let h = i32::from(terrain.heights[index]);
            (low, high) = (low.min(h), high.max(h));
            if high - low > relief {
                return false;
            }
        }
    }
    true
}

/// Goals are tried on a lattice this far apart (one walking-distance field each); starts on a circle of `distance`
/// round each, every `DETOUR_STEP_DEG` degrees.
const DETOUR_LATTICE: f32 = 640.0;
const DETOUR_STEP_DEG: i32 = 5;

/// What a detour must be (design D10): `distance` apart as the crow flies, a walk on foot at least `ratio` times
/// that, each end on a flat square of `side` within `relief`.
#[derive(Clone, Copy, Debug)]
pub struct Detour {
    pub distance: f32,
    pub ratio: f32,
    pub side: f32,
    pub relief: i32,
}

/// A detour's chooser (design D10): pairs `distance` apart as the crow flies whose walk on foot (`terrain::Field` over
/// `standable`) is at least `ratio` times that, both ends on a flat square of `side` within `relief`, clear of the
/// commanders; the longest detours first, `wanted` of them with their ends `SITE_GAP` apart. Returns the sites and the
/// best ratio seen at all (for the message when none passes the bar).
pub fn choose_detour(terrain: &Terrain, max_slope: f32, commanders: &[Vec3], wanted: usize, detour: Detour) -> (Vec<Site>, Option<f32>) {
    let Detour { distance, ratio, side, relief } = detour;
    let (width, height) = (terrain.width as i32, terrain.height as i32);
    if width == 0 || terrain.heights.len() != (width * height) as usize || terrain.cell <= 0.0 {
        return (Vec::new(), None);
    }
    let standable = standable(terrain, max_slope);
    let cell = terrain.cell;
    let centre = |cx: i32, cz: i32| Vec3 { x: (cx as f32 + 0.5) * cell, y: 0.0, z: (cz as f32 + 0.5) * cell };
    let clear = |at: Vec3| commanders.iter().all(|c| c.dist2d(at) >= COMMANDER_GAP);
    let step = (DETOUR_LATTICE / cell).max(1.0) as usize;
    let mut found: Vec<(f32, Vec3, Vec3, f32)> = Vec::new();
    let mut best_ratio: Option<f32> = None;
    for gz in (0..height).step_by(step) {
        for gx in (0..width).step_by(step) {
            if !flat_square(terrain, &standable, gx, gz, side, relief) {
                continue;
            }
            let goal = centre(gx, gz);
            if !clear(goal) {
                continue;
            }
            let Some(field) = terrain::Field::from_many(terrain, &standable, &[goal]) else { continue };
            for k in 0..(360 / DETOUR_STEP_DEG) {
                let angle = (k * DETOUR_STEP_DEG) as f32 * std::f32::consts::PI / 180.0;
                let start = Vec3 { x: goal.x + distance * angle.cos(), y: 0.0, z: goal.z + distance * angle.sin() };
                let (sx, sz) = ((start.x / cell) as i32, (start.z / cell) as i32);
                if !flat_square(terrain, &standable, sx, sz, side, relief) || !clear(start) {
                    continue;
                }
                let Some(walk) = field.distance(start) else { continue };
                let r = walk / distance;
                best_ratio = Some(best_ratio.map_or(r, |b: f32| b.max(r)));
                if r >= ratio {
                    found.push((r, start, goal, walk));
                }
            }
        }
    }
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut sites: Vec<Site> = Vec::new();
    for (_, start, goal, walk) in found {
        if sites.len() >= wanted {
            break;
        }
        let apart = sites.iter().all(|s| {
            let (a, b) = s.ends.expect("a detour site has ends");
            [a, b].iter().all(|e| e.dist2d(start) >= SITE_GAP && e.dist2d(goal) >= SITE_GAP)
        });
        if apart {
            let mid = Vec3 { x: (start.x + goal.x) / 2.0, y: 0.0, z: (start.z + goal.z) / 2.0 };
            sites.push(Site { centre: mid, ends: Some((start, goal)), walk: Some(walk), circuit: Vec::new(), circuit_route: Vec::new() });
        }
    }
    (sites, best_ratio)
}

/// A circuit's default corners as fractions of the map (design D12): south-west (the lab's), south-east, north-east,
/// north-west, a quarter in from each edge; the commanders stand along the north edge for a drill.
pub const CIRCUIT_CORNERS: [(f32, f32); 4] = [(0.25, 0.75), (0.75, 0.75), (0.75, 0.25), (0.25, 0.25)];
/// The square a circuit's lab stands on, standable throughout within `DRILL_RELIEF`: the lab (48 by 48) and the deployed
/// army spawned `drill::CIRCUIT_SPAWN` from it in ranks of six at 64 (320 across; two Pawns of a formation spawned on
/// Quicksilver's broken ground off the square never moved).
const LAB_SQUARE: f32 = 640.0;
/// A circuit's walked route keeps a point of the field's every this far: the group's anchor walks it leg by leg
/// (the field's cells are 8 elmos; the route is the host's, round the terrain).
pub const ROUTE_STEP: f32 = 256.0;

/// What `circuit` finds: the corners, the lap walked on foot between them (`Site::circuit_route`), and each leg's
/// walk on foot.
pub struct Circuit {
    pub corners: Vec<Vec3>,
    pub route: Vec<Vec3>,
    pub legs: Vec<Option<f32>>,
}

/// A circuit's points (design D12): `wanted` (elmos, the first the lab's) or the map's inset corners, each pushed
/// `COMMANDER_GAP` from any commander nearer than that, the lab's moved to the nearest cell holding a flat square, the
/// rest to the nearest cell standable and reachable on foot from the lab's (one field from it); each leg's walk on
/// foot, the last leg closing the loop; and the lap's route on foot, the field's way from each corner to the next
/// thinned to `ROUTE_STEP`. `Err` says what was not found.
pub fn circuit(terrain: &Terrain, standable: &[bool], commanders: &[Vec3], wanted: Option<&[(f32, f32)]>) -> Result<Circuit, String> {
    let (width, height) = (terrain.width as usize, terrain.height as usize);
    if width == 0 || terrain.heights.len() != width * height || terrain.cell <= 0.0 {
        return Err("no terrain".into());
    }
    let (w, h) = (width as f32 * terrain.cell, height as f32 * terrain.cell);
    let mut points: Vec<Vec3> = match wanted {
        Some(list) => list.iter().map(|&(x, z)| Vec3 { x, y: 0.0, z }).collect(),
        None => CIRCUIT_CORNERS.iter().map(|&(fx, fz)| Vec3 { x: fx * w, y: 0.0, z: fz * h }).collect(),
    };
    if points.len() < 2 {
        return Err("a circuit needs two points or more".into());
    }
    for p in &mut points {
        for c in commanders {
            let d = p.dist2d(*c);
            if d < COMMANDER_GAP {
                let (dx, dz) = if d > 1.0 { ((p.x - c.x) / d, (p.z - c.z) / d) } else { (0.0, 1.0) };
                *p = Vec3 { x: c.x + dx * COMMANDER_GAP, y: 0.0, z: c.z + dz * COMMANDER_GAP };
            }
        }
    }
    let centre = |cx: usize, cz: usize| Vec3 { x: (cx as f32 + 0.5) * terrain.cell, y: 0.0, z: (cz as f32 + 0.5) * terrain.cell };
    // The cell nearest `want` that `accept`s: rings of the lattice outward from it, the nearest of the first ring
    // with one.
    let nearest = |want: Vec3, accept: &dyn Fn(usize, usize) -> bool| -> Option<Vec3> {
        let (wx, wz) = ((want.x / terrain.cell) as i64, (want.z / terrain.cell) as i64);
        let stride = STRIDE as i64;
        let mut r = 0i64;
        while r <= width.max(height) as i64 {
            let mut best: Option<(f32, Vec3)> = None;
            let mut dz = -r;
            while dz <= r {
                let mut dx = -r;
                while dx <= r {
                    if dx.abs() == r || dz.abs() == r {
                        let (cx, cz) = (wx + dx, wz + dz);
                        if cx >= 0 && cz >= 0 && cx < width as i64 && cz < height as i64 {
                            let at = centre(cx as usize, cz as usize);
                            let d = at.dist2d(want);
                            if best.is_none_or(|(b, _)| d < b) && accept(cx as usize, cz as usize) {
                                best = Some((d, at));
                            }
                        }
                    }
                    dx += stride;
                }
                dz += stride;
            }
            if let Some((_, at)) = best {
                return Some(at);
            }
            r += stride;
        }
        None
    };
    let lab = nearest(points[0], &|cx, cz| flat_square(terrain, standable, cx as i32, cz as i32, LAB_SQUARE, DRILL_RELIEF))
        .ok_or_else(|| format!("no flat square of {LAB_SQUARE:.0} for the lab anywhere near ({:.0},{:.0})", points[0].x, points[0].z))?;
    let field = Field::from_many(terrain, standable, &[lab]).ok_or("the lab's corner is on no passable ground")?;
    let mut out = vec![lab];
    for want in &points[1..] {
        let at = nearest(*want, &|cx, cz| standable[cz * width + cx] && field.distance(centre(cx, cz)).is_some())
            .ok_or_else(|| format!("no standable ground reachable from the lab near ({:.0},{:.0})", want.x, want.z))?;
        out.push(at);
    }
    let mut legs = Vec::new();
    let mut route = Vec::new();
    for i in 0..out.len() {
        let (from, to) = (out[i], out[(i + 1) % out.len()]);
        let field = Field::from_many(terrain, standable, &[to]);
        legs.push(field.as_ref().and_then(|f| f.distance(from)));
        // The field's way from this corner down to the next, thinned; the next corner itself closes the leg.
        let way = field.as_ref().map(|f| f.route(from)).unwrap_or_default();
        let mut last = from;
        for p in way {
            if p.dist2d(last) >= ROUTE_STEP && p.dist2d(to) >= ROUTE_STEP {
                route.push(p);
                last = p;
            }
        }
        route.push(to);
    }
    Ok(Circuit { corners: out, route, legs })
}

/// The least relief of any dry rectangle of `footprint` every listed unit can cross, whatever `footprint.relief` says,
/// and that rectangle's centre: what the map offers, for the message when no site is found (the commander gap is not
/// applied here, so a strip found here and not by `choose` fails on it). `None` when no rectangle is dry and passable.
pub fn least_relief(terrain: &Terrain, max_slope: f32, footprint: Footprint) -> Option<(i32, Vec3)> {
    let (width, height) = (terrain.width as usize, terrain.height as usize);
    if width == 0 || terrain.heights.len() != width * height {
        return None;
    }
    let (length_cells, width_cells) = ((footprint.length / terrain.cell) as usize, (footprint.width / terrain.cell) as usize);
    let slope_limit = (max_slope * 255.0) as u8;
    let mut least: Option<(i32, Vec3)> = None;
    for z0 in (0..height.saturating_sub(width_cells)).step_by(STRIDE) {
        'candidate: for x0 in (0..width.saturating_sub(length_cells)).step_by(STRIDE) {
            let (mut low, mut high) = (i32::MAX, i32::MIN);
            for z in z0..z0 + width_cells {
                for x in x0..x0 + length_cells {
                    let index = z * width + x;
                    let h = i32::from(terrain.heights[index]);
                    if terrain.slopes[index] > slope_limit || h < 5 {
                        continue 'candidate;
                    }
                    (low, high) = (low.min(h), high.max(h));
                    if least.is_some_and(|(l, _)| high - low >= l) {
                        continue 'candidate;
                    }
                }
            }
            let centre = Vec3 { x: (x0 as f32 + length_cells as f32 / 2.0) * terrain.cell, y: (low + high) as f32 / 2.0, z: (z0 as f32 + width_cells as f32 / 2.0) * terrain.cell };
            least = Some((high - low, centre));
        }
    }
    least
}

/// Distance from `point` to the site rectangle of `footprint` centred on `centre`.
fn rect_distance(centre: Vec3, point: Vec3, footprint: Footprint) -> f32 {
    let dx = ((point.x - centre.x).abs() - footprint.length / 2.0).max(0.0);
    let dz = ((point.z - centre.z).abs() - footprint.width / 2.0).max(0.0);
    dx.hypot(dz)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ranks along a heading: east is the old placement; north turns the ranks to run east-west with the second rank
    /// to the south (behind a body facing north).
    #[test]
    fn ranks_are_laid_across_the_heading() {
        let formation = Formation::parse("ranks2@64", 64.0).unwrap();
        let front = Vec3 { x: 1000.0, y: 0.0, z: 1000.0 };
        let east = formation.places(front, true, 4);
        let along = formation.places_along(front, (1.0, 0.0), 4);
        for (a, b) in east.iter().zip(&along) {
            assert!((a.x - b.x).abs() < 0.01 && (a.z - b.z).abs() < 0.01, "{a:?} {b:?}");
        }
        assert_eq!((east[0].x, east[2].x), (1000.0, 936.0));
        let north = formation.places_along(front, (0.0, -1.0), 4);
        assert!(north.iter().take(2).all(|p| (p.z - 1000.0).abs() < 0.01), "{north:?}");
        assert!(north.iter().skip(2).all(|p| (p.z - 1064.0).abs() < 0.01), "{north:?}");
        assert!((north[0].x - north[1].x).abs() > 63.0);
    }
}
