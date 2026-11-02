//! The metal under the ground and who draws it (`docs/design/2026-11-01-metal-maps.md` §1.3). An extractor claims
//! every metal square whose centre lies within the map's extractor radius, at its depth (`extractsMetal`), and a
//! square already claimed yields a later extractor only the depth above the claim (`MetalMap.cpp`
//! `RequestExtraction`; K-mech-a-metal-map-publishes-no-spots). A tile's worth to a new extractor is the unclaimed
//! metal within its radius: a spot is a local maximum of this field, a metal map a plateau, and one siting rule
//! serves both. On today's metal maps (radius 24 and 30) touching extractors share nothing, so the rule reduces to
//! "the next free tile of metal"; on a map with a large radius it keeps the second extractor off the first's squares.

use bot_protocol::{Terrain, Vec3};

use crate::{Footprint, GRID, Rect};

/// The rings the site search grows by: a lattice within this much of the builder is tried before a wider one.
const RING: f32 = 256.0;

/// The metal map with the extractors' claims over it.
#[derive(Clone, Debug)]
pub struct Coverage {
    width: usize,
    height: usize,
    /// Edge of a metal square in elmos (the terrain's cell).
    cell: f32,
    values: Vec<u8>,
    /// The map's metal scale (`maxMetal`): a square's metal is its value times this.
    scale: f32,
    /// The map's extractor radius.
    radius: f32,
    /// Per square, the depth claimed: the greatest `extracts_metal` of the extractors whose radius covers it.
    claimed: Vec<f32>,
}

impl Coverage {
    /// From the terrain's raw metal map; `None` when the terrain carries none.
    pub fn new(terrain: &Terrain, scale: f32, radius: f32) -> Option<Coverage> {
        let (width, height) = (terrain.width as usize, terrain.height as usize);
        if terrain.metal.len() != width * height || width == 0 || height == 0 || terrain.cell <= 0.0 {
            return None;
        }
        Some(Coverage { width, height, cell: terrain.cell, values: terrain.metal.clone(), scale, radius, claimed: vec![0.0; width * height] })
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Forget every claim: the claims are rebuilt from what stands.
    pub fn clear_claims(&mut self) {
        self.claimed.iter_mut().for_each(|c| *c = 0.0);
    }

    /// The squares whose centres lie within the radius of `centre`: what an extractor there claims.
    fn squares_within(&self, centre: Vec3) -> impl Iterator<Item = usize> + '_ {
        let r = self.radius;
        let (i0, i1) = (((centre.x - r) / self.cell).floor().max(0.0) as usize, (((centre.x + r) / self.cell).ceil().max(0.0) as usize).min(self.width));
        let (j0, j1) = (((centre.z - r) / self.cell).floor().max(0.0) as usize, (((centre.z + r) / self.cell).ceil().max(0.0) as usize).min(self.height));
        let cell = self.cell;
        (j0..j1).flat_map(move |j| (i0..i1).map(move |i| (i, j))).filter_map(move |(i, j)| {
            let (cx, cz) = ((i as f32 + 0.5) * cell, (j as f32 + 0.5) * cell);
            ((cx - centre.x).hypot(cz - centre.z) < r).then_some(j * self.width + i)
        })
    }

    /// An extractor at `centre` drawing at `depth` claims its squares (the engine's rule: to the greater depth).
    pub fn claim(&mut self, centre: Vec3, depth: f32) {
        let squares: Vec<usize> = self.squares_within(centre).collect();
        for k in squares {
            self.claimed[k] = self.claimed[k].max(depth);
        }
    }

    /// The metal a second an extractor at `centre` drawing at `depth` would get: the unclaimed depth of each square
    /// within its radius times the square's metal.
    pub fn worth_at(&self, centre: Vec3, depth: f32) -> f32 {
        self.squares_within(centre).map(|k| (depth - self.claimed[k]).max(0.0) * f32::from(self.values[k]) * self.scale).sum()
    }

    /// Whether any metal lies within the radius of `centre`, claimed or not.
    pub fn on_metal(&self, centre: Vec3) -> bool {
        self.squares_within(centre).any(|k| self.values[k] > 0)
    }

    /// What an unclaimed extractor draws at `depth` on the map's typical metal (the median value of the squares that
    /// hold any): the bar a candidate is measured against.
    pub fn full_worth(&self, depth: f32) -> f32 {
        let mut values: Vec<u8> = self.values.iter().copied().filter(|v| *v > 0).collect();
        if values.is_empty() {
            return 0.0;
        }
        values.sort_unstable();
        let median = f32::from(values[values.len() / 2]);
        // The squares an extractor claims from a footprint centre on the build grid: counted from a square corner.
        let corner = Vec3 { x: (self.width / 2) as f32 * self.cell, y: 0.0, z: (self.height / 2) as f32 * self.cell };
        self.squares_within(corner).count() as f32 * median * self.scale * depth
    }

    /// The best tile for an extractor of `footprint` drawing at `depth`, for a builder at `from` taking tiles within
    /// `reach` of `anchor`: the most unclaimed metal, ties to the nearest the builder. `ok` is the caller's say on
    /// a tile (the ground stands, nothing in the way, no enemy near). Searched in rings round the builder, and a
    /// ring that holds a tile worth the full amount ends the search: the nearest tile of full metal, however large
    /// the map. Returns the tile and its worth.
    pub fn best_site(&self, footprint: Footprint, from: Vec3, anchor: Vec3, reach: f32, depth: f32, ok: &dyn Fn(Vec3, &Rect) -> bool) -> Option<(Vec3, f32)> {
        let full = self.full_worth(depth);
        let (width, height) = (self.width as f32 * self.cell, self.height as f32 * self.cell);
        let farthest = [from.dist2d(Vec3 { x: 0.0, y: 0.0, z: 0.0 }), from.dist2d(Vec3 { x: width, y: 0.0, z: 0.0 }), from.dist2d(Vec3 { x: 0.0, y: 0.0, z: height }), from.dist2d(Vec3 { x: width, y: 0.0, z: height })].into_iter().fold(0.0f32, f32::max);
        let mut best: Option<(Vec3, f32, f32)> = None;
        let mut inner = 0.0f32;
        while inner < farthest {
            let outer = inner + RING;
            let first = footprint.snap(Vec3 { x: from.x - outer, y: from.y, z: from.z - outer });
            let steps = (2.0 * outer / GRID) as i32 + 1;
            for i in 0..=steps {
                for j in 0..=steps {
                    let p = Vec3 { x: first.x + i as f32 * GRID, y: from.y, z: first.z + j as f32 * GRID };
                    let d = p.dist2d(from);
                    if d < inner || d >= outer || p.x < 0.0 || p.z < 0.0 || p.x > width || p.z > height || p.dist2d(anchor) > reach {
                        continue;
                    }
                    let worth = self.worth_at(p, depth);
                    if worth <= 0.0 || best.is_some_and(|(_, w, bd)| worth < w || (worth == w && d >= bd)) {
                        continue;
                    }
                    let rect = footprint.rect(p);
                    if !ok(p, &rect) {
                        continue;
                    }
                    best = Some((p, worth, d));
                }
            }
            if best.is_some_and(|(_, w, _)| w >= full) {
                break;
            }
            inner = outer;
        }
        best.map(|(p, w, _)| (p, w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A metal map of `w` by `h` squares with `metal(x, z)` in each.
    fn terrain(w: u32, h: u32, metal: impl Fn(u32, u32) -> u8) -> Terrain {
        let n = (w * h) as usize;
        Terrain { cell: 16.0, width: w, height: h, heights: vec![0; n], slopes: vec![0; n], metal: (0..h).flat_map(|z| (0..w).map(move |x| (x, z))).map(|(x, z)| metal(x, z)).collect() }
    }

    const MEX: Footprint = Footprint { x: 4, z: 4 };

    #[test]
    fn an_unclaimed_tile_on_speedmetal_is_worth_the_engines_income_and_a_touching_neighbour_loses_nothing() {
        let t = terrain(64, 64, |_, _| 255);
        let mut c = Coverage::new(&t, 10.0, 30.0).unwrap();
        let first = MEX.snap(at(512.0, 512.0));
        assert!((c.worth_at(first, 0.001) - 30.6).abs() < 0.01, "{}", c.worth_at(first, 0.001));
        assert!((c.full_worth(0.001) - 30.6).abs() < 0.01);
        c.claim(first, 0.001);
        assert_eq!(c.worth_at(first, 0.001), 0.0);
        // The neighbour 64 east, footprints touching: its twelve squares are its own.
        let next = at(first.x + 64.0, first.z);
        assert!((c.worth_at(next, 0.001) - 30.6).abs() < 0.01, "{}", c.worth_at(next, 0.001));
        // A tier-2 over the first draws the depth above the claim.
        assert!((c.worth_at(first, 0.004) - 3.0 * 30.6).abs() < 0.05, "{}", c.worth_at(first, 0.004));
    }

    #[test]
    fn with_a_large_radius_an_overlapping_neighbour_is_worth_less() {
        let t = terrain(64, 64, |_, _| 255);
        let mut c = Coverage::new(&t, 1.0, 120.0).unwrap();
        let first = MEX.snap(at(512.0, 512.0));
        let alone = c.worth_at(first, 0.001);
        c.claim(first, 0.001);
        let near = c.worth_at(at(first.x + 64.0, first.z), 0.001);
        let far = c.worth_at(at(first.x + 256.0, first.z), 0.001);
        assert!(near < 0.5 * alone, "near {near} alone {alone}");
        assert!((far - alone).abs() < 0.01);
    }

    #[test]
    fn the_best_site_is_the_nearest_tile_of_full_metal_the_caller_allows() {
        // Metal everywhere but a bare strip at x 400-600; the builder stands in the strip.
        let t = terrain(64, 64, |x, _| if (25..38).contains(&x) { 0 } else { 255 });
        let mut c = Coverage::new(&t, 10.0, 30.0).unwrap();
        let from = at(512.0, 512.0);
        let (site, worth) = c.best_site(MEX, from, from, f32::MAX, 0.001, &|_, _| true).unwrap();
        assert!((worth - 30.6).abs() < 0.01);
        assert!(site.x < 400.0 || site.x > 608.0, "{site:?}");
        assert!(site.dist2d(from) < 200.0, "{site:?}");
        // The caller forbids the west: the site goes east.
        let (east, _) = c.best_site(MEX, from, from, f32::MAX, 0.001, &|p, _| p.x > 512.0).unwrap();
        assert!(east.x > 608.0, "{east:?}");
        // Claimed tiles are passed over for the next free one.
        c.claim(site, 0.001);
        let (second, w2) = c.best_site(MEX, from, from, f32::MAX, 0.001, &|_, _| true).unwrap();
        assert_ne!(second, site);
        assert!((w2 - 30.6).abs() < 0.01);
        // Within reach of the anchor only.
        assert!(c.best_site(MEX, from, from, 50.0, 0.001, &|_, _| true).is_none());
    }

    #[test]
    fn a_spot_map_is_a_local_maximum() {
        // One plus-shaped patch round (520, 520); nothing else.
        let t = terrain(64, 64, |x, z| if (x as i32 - 32).abs() + (z as i32 - 32).abs() <= 1 { 255 } else { 0 });
        let c = Coverage::new(&t, 0.75, 120.0).unwrap();
        let on = c.worth_at(at(520.0, 520.0), 0.001);
        let off = c.worth_at(at(720.0, 520.0), 0.001);
        assert!(on > 0.0 && off == 0.0, "{on} {off}");
        assert!(c.on_metal(at(520.0, 520.0)));
        assert!(!c.on_metal(at(900.0, 900.0)));
        assert!(Coverage::new(&Terrain::default(), 1.0, 120.0).is_none());
    }
}
