//! The metal map as the shim reads it, and what it publishes of it: the spots' patches on a spot map, the fields
//! and an extractor's worth on a metal map (`docs/design/2026-11-01-metal-maps.md` §1.1). Pure functions over the
//! engine's raw metal map, so they are tested here; `engine.rs` only reads the map and calls them.

use std::collections::VecDeque;

use bot_protocol::{MetalField, Vec3};

/// One metal square is 2 by 2 heightmap squares.
pub const SQUARE: f32 = 16.0;
/// The engine's spot analyzer stops at this many spots (`ResourceMapAnalyzer.cpp` `maxSpots`): a list this long is
/// its packing of a metal map, not a map's spots.
pub const ENGINE_SPOT_CAP: usize = 10_000;
/// A tier-1 extractor's footprint is 4 by 4 heightmap squares: 2 by 2 metal squares in each direction, so 16 of them
/// under it. The room of a field is counted in these.
const FOOTPRINT_SQUARES: u32 = 16;

/// The raw metal map: one value per square, row-major from the north-west corner, values 0..=255 (the engine hands
/// them as `i16`).
pub struct MetalMap<'a> {
    pub values: &'a [i16],
    pub width: usize,
    pub height: usize,
}

impl MetalMap<'_> {
    pub fn value(&self, x: usize, z: usize) -> i16 {
        self.values.get(z * self.width + x).copied().unwrap_or(0).max(0)
    }

    /// The centre of square `(x, z)` in elmos.
    pub fn centre(x: usize, z: usize) -> (f32, f32) {
        ((x as f32 + 0.5) * SQUARE, (z as f32 + 0.5) * SQUARE)
    }

    pub fn squares(&self) -> usize {
        self.width * self.height
    }

    /// How many squares hold metal.
    pub fn metal_squares(&self) -> usize {
        (0..self.squares()).filter(|&i| self.values.get(i).copied().unwrap_or(0) > 0).count()
    }
}

/// Whether the map is a metal map: the game's finder said so (`mex_count` -1), or, without the finder's word, the
/// engine's list hit its cap or metal covers more than a tenth of the squares.
pub fn is_metal_map(mex_count: f32, engine_spots: usize, map: &MetalMap) -> bool {
    mex_count < 0.0 || engine_spots >= ENGINE_SPOT_CAP || map.metal_squares() * 10 > map.squares()
}

/// The spots' patches: per spot the centres (elmos) of its squares, and the sums of x times metal, z times metal,
/// and metal, for the centroid and the worth.
pub struct Patches {
    pub squares: Vec<Vec<(f32, f32)>>,
    pub weighted: Vec<(f64, f64, f64)>,
}

/// Every metal square to the nearest spot within `reach` of it. Spots are bucketed by `reach` so the work is the
/// squares, not the squares times the spots (10,000 spots on SpeedMetal made the old scan 7.4 s).
pub fn assign_squares(spots: &[Vec3], map: &MetalMap, reach: f32) -> Patches {
    let mut squares: Vec<Vec<(f32, f32)>> = vec![Vec::new(); spots.len()];
    let mut weighted: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, 0.0); spots.len()];
    if spots.is_empty() || map.width == 0 {
        return Patches { squares, weighted };
    }
    let bucket_of = |x: f32, z: f32| ((x / reach).floor() as i64, (z / reach).floor() as i64);
    let mut buckets: std::collections::HashMap<(i64, i64), Vec<usize>> = std::collections::HashMap::new();
    for (i, spot) in spots.iter().enumerate() {
        buckets.entry(bucket_of(spot.x, spot.z)).or_default().push(i);
    }
    for z in 0..map.height {
        for x in 0..map.width {
            let value = map.value(x, z);
            if value <= 0 {
                continue;
            }
            let (cx, cz) = MetalMap::centre(x, z);
            let (bx, bz) = bucket_of(cx, cz);
            let mut nearest: Option<(usize, f32)> = None;
            for dz in -1..=1 {
                for dx in -1..=1 {
                    for &i in buckets.get(&(bx + dx, bz + dz)).map_or(&[][..], |b| b.as_slice()) {
                        let d = (spots[i].x - cx).hypot(spots[i].z - cz);
                        if d < reach && nearest.is_none_or(|(_, best)| d < best) {
                            nearest = Some((i, d));
                        }
                    }
                }
            }
            if let Some((i, _)) = nearest {
                squares[i].push((cx, cz));
                let w = f64::from(value);
                weighted[i] = (weighted[i].0 + f64::from(cx) * w, weighted[i].1 + f64::from(cz) * w, weighted[i].2 + w);
            }
        }
    }
    Patches { squares, weighted }
}

/// The metal fields of a metal map: connected metal squares, where two squares are connected when they lie within an
/// extractor's diameter of each other (so a road or a crack narrower than a mex does not split a field), named
/// `field_N` west to east. Fields with less than one footprint of squares are dropped: nothing stands on them.
pub fn fields(map: &MetalMap, scale: f32, radius: f32) -> Vec<MetalField> {
    if map.width == 0 || map.height == 0 {
        return Vec::new();
    }
    // The gap bridged, in squares: an extractor's diameter, at least one square.
    let gap = ((2.0 * radius / SQUARE).ceil() as i64).max(1);
    let mut field_of: Vec<u32> = vec![0; map.squares()];
    let mut found: Vec<MetalField> = Vec::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    for z0 in 0..map.height {
        for x0 in 0..map.width {
            if map.value(x0, z0) <= 0 || field_of[z0 * map.width + x0] != 0 {
                continue;
            }
            let id = found.len() as u32 + 1;
            field_of[z0 * map.width + x0] = id;
            queue.push_back((x0, z0));
            let (mut sx, mut sz, mut sw, mut n) = (0.0f64, 0.0f64, 0.0f64, 0u32);
            let (mut west, mut north, mut east, mut south) = (usize::MAX, usize::MAX, 0usize, 0usize);
            while let Some((x, z)) = queue.pop_front() {
                let w = f64::from(map.value(x, z));
                let (cx, cz) = MetalMap::centre(x, z);
                sx += f64::from(cx) * w;
                sz += f64::from(cz) * w;
                sw += w;
                n += 1;
                (west, north, east, south) = (west.min(x), north.min(z), east.max(x), south.max(z));
                for dz in -gap..=gap {
                    for dx in -gap..=gap {
                        let (nx, nz) = (x as i64 + dx, z as i64 + dz);
                        if nx < 0 || nz < 0 || nx >= map.width as i64 || nz >= map.height as i64 {
                            continue;
                        }
                        let (nx, nz) = (nx as usize, nz as usize);
                        if map.value(nx, nz) > 0 && field_of[nz * map.width + nx] == 0 {
                            field_of[nz * map.width + nx] = id;
                            queue.push_back((nx, nz));
                        }
                    }
                }
            }
            found.push(MetalField {
                name: String::new(),
                centre: Vec3 { x: (sx / sw) as f32, y: 0.0, z: (sz / sw) as f32 },
                bounds: [west as f32 * SQUARE, north as f32 * SQUARE, (east + 1) as f32 * SQUARE, (south + 1) as f32 * SQUARE],
                worth: (sw * f64::from(scale)) as f32,
                room: n / FOOTPRINT_SQUARES,
                squares: n,
            });
        }
    }
    found.retain(|f| f.room >= 1);
    found.sort_by(|a, b| a.centre.x.total_cmp(&b.centre.x).then(a.centre.z.total_cmp(&b.centre.z)));
    for (i, f) in found.iter_mut().enumerate() {
        f.name = format!("field_{i}");
    }
    found
}

/// How many squares an extractor claims: those whose centres lie within `radius` of its centre, which for a 4 by 4
/// footprint sits on a square corner. Comet's 120 claims a patch's dozen and more; SpeedMetal's 30 claims 12, Full
/// Metal Plate's 24 claims 4.
pub fn squares_claimed(radius: f32) -> u32 {
    let steps = (radius / SQUARE).ceil() as i64 + 1;
    let mut n = 0;
    for i in -steps..steps {
        for j in -steps..steps {
            let (cx, cz) = ((i as f32 + 0.5) * SQUARE, (j as f32 + 0.5) * SQUARE);
            if cx.hypot(cz) < radius {
                n += 1;
            }
        }
    }
    n
}

/// The metal a second a tier-1 extractor draws standing on the map's typical metal (the median value among the
/// squares that hold any), the engine's rule: squares claimed times the value times the map's scale times
/// `extracts_metal` (SpeedMetal 12 x 255 x 10 x 0.001 = 30.6; Full Metal Plate 4 x 79 x 7.5 x 0.001 = 2.37).
pub fn extractor_worth(map: &MetalMap, radius: f32, scale: f32, extracts_metal: f32) -> f32 {
    let mut values: Vec<i16> = (0..map.squares()).map(|i| map.values.get(i).copied().unwrap_or(0)).filter(|v| *v > 0).collect();
    if values.is_empty() {
        return 0.0;
    }
    values.sort_unstable();
    let median = f32::from(values[values.len() / 2]);
    squares_claimed(radius) as f32 * median * scale * extracts_metal
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_of(width: usize, height: usize, metal: impl Fn(usize, usize) -> i16) -> Vec<i16> {
        (0..height).flat_map(|z| (0..width).map(move |x| (x, z))).map(|(x, z)| metal(x, z)).collect()
    }

    #[test]
    fn an_extractor_claims_twelve_squares_at_speedmetals_radius_and_four_at_full_metal_plates() {
        assert_eq!(squares_claimed(30.0), 12);
        assert_eq!(squares_claimed(24.0), 4);
    }

    #[test]
    fn the_worth_is_the_engines_income_on_the_two_metal_maps() {
        let speed = map_of(8, 8, |_, _| 255);
        let worth = extractor_worth(&MetalMap { values: &speed, width: 8, height: 8 }, 30.0, 10.0, 0.001);
        assert!((worth - 30.6).abs() < 0.01, "{worth}");
        let plate = map_of(8, 8, |_, _| 79);
        let worth = extractor_worth(&MetalMap { values: &plate, width: 8, height: 8 }, 24.0, 7.5, 0.001);
        assert!((worth - 2.37).abs() < 0.01, "{worth}");
    }

    #[test]
    fn a_metal_map_is_told_by_the_finder_the_cap_or_the_cover() {
        let bare = map_of(10, 10, |x, z| if (x, z) == (3, 3) { 255 } else { 0 });
        let bare = MetalMap { values: &bare, width: 10, height: 10 };
        assert!(!is_metal_map(1.0, 1, &bare));
        assert!(is_metal_map(-1.0, 1, &bare));
        assert!(is_metal_map(0.0, ENGINE_SPOT_CAP, &bare));
        let covered = map_of(10, 10, |x, _| if x < 2 { 255 } else { 0 });
        assert!(is_metal_map(0.0, 1, &MetalMap { values: &covered, width: 10, height: 10 }));
    }

    #[test]
    fn squares_go_to_the_nearest_spot_within_reach_and_the_centroid_is_metal_weighted() {
        // Two patches: a plus round (5, 5) and a single square at (20, 5); a stray square far from both.
        let values = map_of(32, 12, |x, z| match (x, z) {
            (5, 5) | (4, 5) | (6, 5) | (5, 4) | (5, 6) => 255,
            (7, 5) => 127,
            (20, 5) => 255,
            (30, 11) => 255,
            _ => 0,
        });
        let map = MetalMap { values: &values, width: 32, height: 12 };
        let spots = [Vec3 { x: 5.5 * SQUARE, y: 0.0, z: 5.5 * SQUARE }, Vec3 { x: 20.5 * SQUARE, y: 0.0, z: 5.5 * SQUARE }];
        // Reach 150: the stray at (30, 11) is 186 from the second spot.
        let Patches { squares, weighted } = assign_squares(&spots, &map, 150.0);
        assert_eq!(squares[0].len(), 6);
        assert_eq!(squares[1].len(), 1);
        let (sx, _, sw) = weighted[0];
        assert!(sx / sw > 5.5 * f64::from(SQUARE), "the 127 square at x=7 pulls the centroid east: {}", sx / sw);
        assert_eq!(weighted[1].2, 255.0);
    }

    #[test]
    fn fields_are_connected_across_a_crack_narrower_than_an_extractor_and_named_west_to_east() {
        // Two plateaus of 8x8 squares (64 squares, room 4 each): one at x 0-7 split by a one-square crack at x=3,
        // one at x 40-47; a speck of one column (8 squares, under a footprint) between them.
        let values = map_of(48, 8, |x, _| if (x < 8 && x != 3) || x >= 40 || x == 20 { 255 } else { 0 });
        let map = MetalMap { values: &values, width: 48, height: 8 };
        let found = fields(&map, 10.0, 30.0);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].name, "field_0");
        assert_eq!(found[0].squares, 56);
        assert_eq!(found[0].room, 3);
        assert_eq!(found[0].bounds, [0.0, 0.0, 8.0 * SQUARE, 8.0 * SQUARE]);
        assert!((found[0].worth - 56.0 * 255.0 * 10.0).abs() < 1.0);
        assert_eq!(found[1].name, "field_1");
        assert_eq!(found[1].squares, 64);
        assert!(found[1].centre.x > found[0].centre.x);
    }

    #[test]
    fn a_uniform_map_is_one_field() {
        let values = map_of(64, 64, |_, _| 79);
        let found = fields(&MetalMap { values: &values, width: 64, height: 64 }, 7.5, 24.0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].room, 64 * 64 / 16);
    }
}
