//! Where duels are fought: flat, dry rectangles every listed unit can cross, far from the commanders and from
//! each other. The long side runs west-east: spawned units always face south (the engine's cheat gives no choice),
//! so on a west-east axis both sides start side-on to the enemy and neither has its back turned.

use bot_protocol::{Terrain, Vec3};

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
const COMMANDER_GAP: f32 = 900.0;
/// Most the ground may rise or fall within a site, in elmos (height lends range to ballistic weapons).
const MAX_RELIEF: i32 = 16;
/// Candidate rectangles are tried every this many cells.
const STRIDE: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub centre: Vec3,
}

/// The rectangle a site must offer: `length` along the west-east axis, `width` across it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub length: f32,
    pub width: f32,
}

impl Footprint {
    /// The least rectangle that holds both armies of every duel named: `(formation, count)` per army.
    pub fn holding(armies: impl IntoIterator<Item = (Formation, u32)>) -> Footprint {
        let mut footprint = Footprint { length: LENGTH, width: WIDTH };
        for (formation, count) in armies {
            let (across, depth) = formation.extent(count);
            footprint.length = footprint.length.max(SEPARATION + 2.0 * (depth + MARGIN));
            footprint.width = footprint.width.max(across + 2.0 * MARGIN);
        }
        footprint
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
        let per_rank = self.per_rank(count);
        let spacing = self.spacing;
        let back = if faces_east { -spacing } else { spacing };
        (0..count)
            .map(|i| {
                let (rank, file) = (i / per_rank, i % per_rank);
                let in_rank = (count - rank * per_rank).min(per_rank);
                Vec3 {
                    x: front.x + back * rank as f32,
                    y: front.y,
                    z: front.z + (file as f32 - (in_rank - 1) as f32 / 2.0) * spacing,
                }
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
    let Footprint { length, width: across } = footprint;
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
                    if high - low > MAX_RELIEF {
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
                sites.push(Site { centre: *centre });
            }
        }
        if sites.len() > best.len() {
            best = sites;
        }
    }
    best
}

/// Distance from `point` to the site rectangle of `footprint` centred on `centre`.
fn rect_distance(centre: Vec3, point: Vec3, footprint: Footprint) -> f32 {
    let dx = ((point.x - centre.x).abs() - footprint.length / 2.0).max(0.0);
    let dz = ((point.z - centre.z).abs() - footprint.width / 2.0).max(0.0);
    dx.hypot(dz)
}
