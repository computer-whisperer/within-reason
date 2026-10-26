//! The map's metal spots in regions (docs/design/2026-10-25-territory-regions.md, part 1): the spots one builder
//! works as one trip, a walk out and then a step from spot to spot. Made once from the game's start on the map's own
//! ground: a region's spots lie within a walking radius of its first, and each of our builders' movement classes
//! reaches all of a region's spots from home or none of them.

use bot_protocol::{MoveClass, Vec3};
use terrain::Field;

use crate::game::{distance, Game};

/// A region's spots lie within this many seconds' walk of its first, at the line's first constructor's speed (the
/// proposal: about 20 s, some 1,100 elmos at a vehicle constructor's 54 a second; a radius of 1,200 in straight lines
/// gave the bench's five maps 5-30 regions of a median 2-5 spots, at most 10).
pub const REGION_SECONDS: f64 = 20.0;

/// Where no way on foot is found between two spots of a region, the walk is charged this many straight lines (as
/// `game::Walked` charges it).
const NO_WAY_DETOUR: f64 = 3.0;

#[derive(Clone, Debug)]
pub struct Region {
    /// Indices into `Game::spots` in the order one builder takes them: the one nearest home first, then each time
    /// the nearest of the rest on foot.
    pub spots: Vec<usize>,
    /// The spot whose walks to the others sum least.
    pub centre: usize,
    /// Effective elmos along `spots` in that order, on the commander's ground.
    pub tour: f64,
    /// Per class of `Regions::classes`: effective elmos from home to the first spot, None where it cannot get there.
    pub trip: Vec<Option<f64>>,
}

pub struct Regions {
    pub list: Vec<Region>,
    /// Each spot's region, by `Game::spots` index.
    pub of_spot: Vec<usize>,
    /// Our ground builders' movement classes, the commander's first.
    pub classes: Vec<MoveClass>,
    /// The walking radius, in effective elmos.
    pub radius: f64,
}

fn same(a: &MoveClass, b: &MoveClass) -> bool {
    a.kind == b.kind && a.max_slope == b.max_slope && a.depth == b.depth && a.slope_mod == b.slope_mod
}

fn at(p: (f64, f64)) -> Vec3 {
    Vec3 { x: p.0 as f32, y: 0.0, z: p.1 as f32 }
}

impl Regions {
    /// The regions of `game`'s spots for the builder types in `builders` (the commander is taken whether listed or
    /// not), each region's spots within `radius` effective elmos' walk of its first. Take the unplaced spot nearest
    /// home on foot, gather the unplaced spots within the radius of it that the same classes reach, and repeat.
    pub fn new(game: &Game, builders: &[usize], radius: f64) -> Regions {
        let units = &game.units;
        let terrain = &game.terrain;
        let mut classes: Vec<MoveClass> = Vec::new();
        for b in std::iter::once(game.commander).chain(builders.iter().copied()) {
            if let Some(c) = units.list[b].move_class
                && !classes.iter().any(|k| same(k, &c))
            {
                classes.push(c);
            }
        }
        let spot = |s: usize| game.spots[s].0;
        let costs: Vec<Vec<u32>> = classes.iter().map(|c| terrain::costs(terrain, *c)).collect();
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, 4);
        // Each class's walk from home to every spot, a thread a class; without terrain data every walk is None alike.
        let from_home: Vec<Vec<Option<f64>>> = std::thread::scope(|scope| {
            let walks: Vec<_> = costs
                .iter()
                .map(|c| {
                    scope.spawn(move || {
                        let field = Field::from_costs(terrain, c, &[at(game.home)]);
                        (0..game.spots.len()).map(|s| field.as_ref().and_then(|f| f.distance(at(spot(s)))).map(f64::from)).collect()
                    })
                })
                .collect();
            walks.into_iter().map(|w| w.join().expect("a class's walks from home")).collect()
        });
        let reached_by = |s: usize| -> Vec<bool> { from_home.iter().map(|w| w[s].is_some()).collect() };
        let home_walk = |s: usize| from_home.first().and_then(|w| w[s]).unwrap_or_else(|| distance(game.home, spot(s)) * NO_WAY_DETOUR);
        let mut order: Vec<usize> = (0..game.spots.len()).collect();
        order.sort_by(|a, b| home_walk(*a).total_cmp(&home_walk(*b)));
        // A field on the commander's ground out to `limit` from a spot; None without terrain data.
        let near = |s: usize, limit: f64| costs.first().and_then(|c| Field::from_costs_within(terrain, c, &[at(spot(s))], limit as f32));
        let walk = |field: &Option<Field>, from: usize, to: usize| match field {
            Some(f) => f.distance(at(spot(to))).map(f64::from),
            None => Some(distance(spot(from), spot(to))),
        };
        // The partition: each seed with its members (the seed first) and their walks to it.
        let mut of_spot = vec![usize::MAX; game.spots.len()];
        let mut groups: Vec<(Vec<usize>, Vec<f64>)> = Vec::new();
        for &seed in &order {
            if of_spot[seed] != usize::MAX {
                continue;
            }
            let classes_reaching = reached_by(seed);
            let field = near(seed, radius);
            let members: Vec<usize> = order
                .iter()
                .copied()
                .filter(|&s| s == seed || (of_spot[s] == usize::MAX && reached_by(s) == classes_reaching && walk(&field, seed, s).is_some_and(|d| d <= radius)))
                .collect();
            let to_seed: Vec<f64> = members.iter().map(|&m| walk(&field, seed, m).unwrap_or(radius)).collect();
            for &m in &members {
                of_spot[m] = groups.len();
            }
            groups.push((members, to_seed));
        }
        // Each region's order, centre and tour from the walks between its spots, each searched no farther than the
        // way through the seed to the farthest; the regions shared over the threads.
        let shape = |(members, to_seed): &(Vec<usize>, Vec<f64>)| -> (Vec<usize>, usize, f64) {
            let k = members.len();
            let mut d = vec![vec![0.0f64; k]; k];
            if k > 1 {
                let farthest = to_seed.iter().copied().fold(0.0, f64::max);
                let fields: Vec<Option<Field>> = members.iter().zip(to_seed).map(|(&m, r)| near(m, r + farthest + 1.0)).collect();
                for i in 0..k {
                    for j in 0..k {
                        if i != j {
                            d[i][j] = walk(&fields[i], members[i], members[j]).unwrap_or_else(|| distance(spot(members[i]), spot(members[j])) * NO_WAY_DETOUR);
                        }
                    }
                }
            }
            let (mut tour, mut path, mut left) = (0.0, vec![0usize], (1..k).collect::<Vec<usize>>());
            while let Some((pos, &next)) = left.iter().enumerate().min_by(|a, b| d[*path.last().unwrap()][*a.1].total_cmp(&d[*path.last().unwrap()][*b.1])) {
                tour += d[*path.last().unwrap()][next];
                path.push(next);
                left.remove(pos);
            }
            let centre = (0..k).min_by(|a, b| d[*a].iter().sum::<f64>().total_cmp(&d[*b].iter().sum::<f64>())).unwrap_or(0);
            (path.iter().map(|&i| members[i]).collect(), members[centre], tour)
        };
        let mut shaped: Vec<Option<(Vec<usize>, usize, f64)>> = vec![None; groups.len()];
        std::thread::scope(|scope| {
            let parts: Vec<_> = (0..threads)
                .map(|t| {
                    let (groups, shape) = (&groups, &shape);
                    scope.spawn(move || groups.iter().enumerate().skip(t).step_by(threads).map(|(i, g)| (i, shape(g))).collect::<Vec<_>>())
                })
                .collect();
            for part in parts {
                for (i, region) in part.join().expect("a region's walks") {
                    shaped[i] = Some(region);
                }
            }
        });
        let list = groups
            .iter()
            .zip(shaped)
            .map(|((members, _), region)| {
                let (spots, centre, tour) = region.expect("every region shaped");
                Region { spots, centre, tour, trip: from_home.iter().map(|w| w[members[0]]).collect() }
            })
            .collect();
        Regions { list, of_spot, classes, radius }
    }

    /// The seconds a builder of type `unit` loses per extractor in region `r` when it clears the region from home:
    /// the trip to its first spot and the tour through the rest, shared over its spots. None where its class cannot
    /// get there. The tour is walked on the commander's ground for every class (a vehicle's steeper detours inside a
    /// region are not counted), and a builder that flies is charged the straight trip and the same tour.
    pub fn walk_seconds(&self, game: &Game, r: usize, unit: usize) -> Option<f64> {
        let region = &self.list[r];
        let u = &game.units.list[unit];
        let trip = match u.move_class {
            None => distance(game.home, game.spots[region.spots[0]].0),
            Some(c) => region.trip[self.classes.iter().position(|k| same(k, &c))?]?,
        };
        Some((trip + region.tour) / region.spots.len() as f64 / u.speed.max(1.0))
    }

    /// The metal a second region `r`'s spots give our tier-1 extractors.
    pub fn metal(&self, game: &Game, r: usize) -> f64 {
        self.list[r].spots.iter().map(|s| game.spot_metal(game.spots[*s].1)).sum()
    }

    /// One entry a region for the log: its spots (and how many of them are in `in_play`), their metal a second,
    /// its centre, the tour, and for each builder type the seconds an extractor there costs it ("-": cannot get there).
    pub fn describe(&self, game: &Game, builders: &[usize], in_play: &[usize]) -> Vec<String> {
        (0..self.list.len())
            .map(|r| {
                let region = &self.list[r];
                let playing = region.spots.iter().filter(|s| in_play.contains(s)).count();
                let (cx, cz) = game.spots[region.centre].0;
                let prices: Vec<String> = builders
                    .iter()
                    .map(|b| format!("{} {}", game.units.list[*b].name, self.walk_seconds(game, r, *b).map_or("-".to_string(), |s| format!("{s:.0}"))))
                    .collect();
                let play = if playing == region.spots.len() { String::new() } else { format!(" ({playing} in play)") };
                format!("r{r} {} spots{play} {:.1}/s at ({cx:.0},{cz:.0}) tour {:.0}: s an extractor {}", region.spots.len(), self.metal(game, r), region.tour, prices.join(" "))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_protocol::{MoveKind, Terrain};

    const BOT: MoveClass = MoveClass { kind: MoveKind::Bot, max_slope: 0.412, depth: 5000.0, slope_mod: 4.0 };
    const TANK: MoveClass = MoveClass { kind: MoveKind::Tank, max_slope: 0.109, depth: 20.0, slope_mod: 18.0 };

    /// The Quicksilver fixture's units on a flat 4,096 square map with home at (400, 400), the commander walking as a
    /// bot and the vehicle constructor as a tank; `ground` shapes the slopes.
    fn game(spots: &[(f64, f64)], ground: impl Fn(usize, usize) -> u8) -> (Game, usize) {
        let mut game = crate::record::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/quicksilver-nw.jsonl"), 60.0).unwrap().game;
        let n = 256usize;
        game.terrain = Terrain { cell: 16.0, width: n as u32, height: n as u32, heights: vec![50; n * n], slopes: (0..n * n).map(|i| ground(i % n, i / n)).collect(), metal: Vec::new() };
        game.home = (400.0, 400.0);
        game.spots = spots.iter().map(|p| (*p, 2.0)).collect();
        let cv = game.units.index("armcv").unwrap();
        game.units.list[game.commander].move_class = Some(BOT);
        game.units.list[cv].move_class = Some(TANK);
        (game, cv)
    }

    #[test]
    fn spots_a_walk_apart_are_two_regions_taken_nearest_first() {
        // Three spots by home and two 2,600 off; a radius of 600.
        let (game, cv) = game(&[(2900.0, 400.0), (500.0, 400.0), (800.0, 400.0), (600.0, 450.0), (3100.0, 400.0)], |_, _| 0);
        let regions = Regions::new(&game, &[cv], 600.0);
        assert_eq!(regions.list.len(), 2);
        assert_eq!(regions.list[0].spots, vec![1, 3, 2], "from the spot nearest home, each time the nearest of the rest");
        assert_eq!(regions.list[1].spots, vec![0, 4]);
        assert_eq!(regions.of_spot, vec![1, 0, 0, 0, 1]);
        let far = &regions.list[1];
        assert!((far.tour - 200.0).abs() < 20.0, "{}", far.tour);
        let trip = far.trip[1].unwrap();
        assert!((trip - 2500.0).abs() < 40.0, "{trip}");
        // The vehicle's price there: the trip and the tour shared over the two spots, at its speed.
        let speed = game.units.list[cv].speed;
        let want = (trip + far.tour) / 2.0 / speed;
        assert!((regions.walk_seconds(&game, 1, cv).unwrap() - want).abs() < 1e-9);
        assert_eq!(regions.list[0].centre, 3, "the middle spot's walks to the others sum least");
    }

    #[test]
    fn a_wall_parts_spots_a_straight_line_would_join() {
        // Two spots 320 apart either side of a wall down x = 1,600 that ends at z = 3,200: the walk round is 5,000.
        let (game, cv) = game(&[(1440.0, 800.0), (1760.0, 800.0)], |x, z| if (98..=102).contains(&x) && z < 200 { 255 } else { 0 });
        let regions = Regions::new(&game, &[cv], 1000.0);
        assert_eq!(regions.list.len(), 2);
    }

    #[test]
    fn a_spot_one_class_cannot_reach_is_a_region_of_its_own() {
        // A spot beside home in the middle of a mound 500 across whose sides only a bot climbs (a spot within the
        // field's snap of ground a vehicle stands on is reached: it builds from there), and one on the flat beside it.
        let mound = |x: usize, z: usize| if (30..=60).contains(&x) && (30..=60).contains(&z) { 60 } else { 0 };
        let (game, cv) = game(&[(728.0, 728.0), (300.0, 728.0)], mound);
        let regions = Regions::new(&game, &[cv], 1000.0);
        assert_eq!(regions.list.len(), 2);
        let on_mound = regions.of_spot[0];
        assert!(regions.walk_seconds(&game, on_mound, cv).is_none(), "the vehicle cannot get there");
        assert!(regions.walk_seconds(&game, on_mound, game.commander).is_some());
        assert!(regions.walk_seconds(&game, regions.of_spot[1], cv).is_some());
    }
}
