//! Where his commander starts (T-SCENARIO): the script's position when the lobby fixed the starts, else where the
//! game itself puts a seat nobody placed in its box, else our start mirrored through the map's centre.
//!
//! The game's rule is BAR's `GuessOne` (`common/lib_startpoint_guesser.lua`, called by `game_initial_spawn.lua` at
//! the game's start for every seat of a choose-in-game script with no start of its own, which is every AI's): the
//! metal spot in the box with the densest cluster round it, then a step toward its nearest neighbour. Replayed on
//! the dumps of terr103-107 it put his start within 1-4 elmos of where his commander stood in all twenty games,
//! where his box's centre (the bot's guess before) was 100-2,437 off and the mirror 3-1,195.

use bot_protocol::{Controller, Hello, StartBox, Vec3};

/// A start claims the spots within this of it, and within CLAIM_HEIGHT of its height (`claimRadius`, the D-gun's
/// 250 times 2.3, and `claimHeight`).
const CLAIM_RADIUS: f32 = 250.0 * 2.3;
const CLAIM_HEIGHT: f32 = 300.0;
/// Spots outside the box this near it count toward a spot's cluster (`walkRadius`).
const WALK_RADIUS: f32 = 250.0 * 3.0;
/// The start lies this far from the best spot toward its nearest free neighbour.
const STEP: f32 = 120.0;

/// His start and what it rests on.
pub(crate) fn his_start(hello: &Hello, home: Vec3) -> (Vec3, &'static str) {
    let ours = hello.ally_team;
    if let Some(at) = hello.teams.iter().filter(|t| t.ally_team != ours && t.controller != Controller::Gaia).find_map(|t| t.start_pos) {
        return (at, "the script's");
    }
    let mirror = Vec3 { x: hello.map.width - home.x, y: 0.0, z: hello.map.height - home.z };
    // The boxes say where a seat starts only when the script has the game choose (startpostype 2).
    let Some(his_box) = hello.start_boxes.iter().find(|b| b.ally_team != ours).filter(|_| hello.start_pos_type == Some(2)) else {
        return (mirror, "our start mirrored");
    };
    let terrain = &hello.terrain;
    let cell = |p: Vec3| {
        let (i, j) = ((p.x / terrain.cell).floor() as i64, (p.z / terrain.cell).floor() as i64);
        (i >= 0 && j >= 0 && i < i64::from(terrain.width) && j < i64::from(terrain.height)).then(|| j as usize * terrain.width as usize + i as usize)
    };
    let height = |p: Vec3| cell(p).and_then(|k| terrain.heights.get(k)).map_or(0.0, |h| f32::from(*h));
    // `IsSteep`: steeper than a commander starts on, the mover's slope limit (acos(1 - 0.41221) less 0.02), read
    // here from the cell's slope value (1 - the normal's y, times 255) where the game tests four 5-elmo steps.
    let steep_limit = (1.0 - ((1.0f32 - 0.41221).acos() - 0.02).cos()) * 255.0;
    let steep = |p: Vec3| cell(p).and_then(|k| terrain.slopes.get(k)).is_some_and(|s| f32::from(*s) > steep_limit);
    match guess(&hello.metal_spots, his_box, &[home], height, steep) {
        Some(at) => (at, "the game's guess in his box"),
        None => (his_box.centre(), "his box's centre"),
    }
}

/// BAR's `GuessOne` for a rectangular box: None where it gives up (no spot in the box or within WALK_RADIUS of it),
/// and the game then tries its second guess, which this does not (`GuessTwo`, from the metal map) before the box's
/// centre. `placed` are the starts already standing, whose spots are claimed.
pub(crate) fn guess(spots: &[Vec3], b: &StartBox, placed: &[Vec3], height: impl Fn(Vec3) -> f32, steep: impl Fn(Vec3) -> bool) -> Option<Vec3> {
    // Strictly inside, as the game's `IsInStartbox` tests a rectangle (`StartBox::contains` takes the edges in).
    let inside = |p: Vec3| b.left < p.x && p.x < b.right && b.top < p.z && p.z < b.bottom;
    let nearest_inside = |p: Vec3| Vec3 { x: p.x.clamp(b.left, b.right), y: 0.0, z: p.z.clamp(b.top, b.bottom) };
    let claimed = |p: Vec3, y: f32| placed.iter().any(|s| s.dist2d(p) <= CLAIM_RADIUS && (y - height(*s)).abs() <= CLAIM_HEIGHT);
    let (mut free, mut walkable): (Vec<(Vec3, f32)>, Vec<(Vec3, f32)>) = (Vec::new(), Vec::new());
    for s in spots {
        let p = Vec3 { y: 0.0, ..*s };
        let y = height(p);
        if claimed(p, y) {
            continue;
        }
        if inside(p) {
            free.push((p, y));
        } else if p.dist2d(nearest_inside(p)) <= WALK_RADIUS {
            walkable.push((p, y));
        }
    }
    if free.is_empty() {
        // The game keeps the last walkable spot's nearest point in the box that is not steep (its loop never lowers
        // its best distance, and every walkable spot is within it).
        return walkable.iter().map(|(p, _)| nearest_inside(*p)).filter(|q| !steep(*q)).last();
    }
    // Each free spot scores 10 for every other spot (free or walkable) it claims, else the inverse square root of
    // the distance; the last of the best wins (the game's `>=`).
    let score = |(p, y): (Vec3, f32)| -> f32 {
        free.iter().chain(&walkable).filter(|(q, _)| (q.x, q.z) != (p.x, p.z)).map(|(q, qy)| {
            let r = p.dist2d(*q);
            if r <= CLAIM_RADIUS && (y - qy).abs() <= CLAIM_HEIGHT { 10.0 } else { r.powf(-0.5) }
        }).sum()
    };
    let scores: Vec<f32> = free.iter().map(|f| score(*f)).collect();
    let best = (1..scores.len()).fold(0, |best, i| if scores[i] >= scores[best] { i } else { best });
    let b0 = free[best].0;
    // The first nearest of the other free spots; with none, or one sharing the best's x or z, the game steps toward
    // (x + 1, x + 1), its own slip kept.
    let nearest = free.iter().enumerate().filter(|(i, _)| *i != best).fold(None, |acc: Option<Vec3>, (_, (q, _))| match acc {
        Some(a) if a.dist2d(b0) <= q.dist2d(b0) => Some(a),
        _ => Some(*q),
    });
    let n = match nearest {
        Some(q) if q.x != b0.x && q.z != b0.z => q,
        _ => Vec3 { x: b0.x + 1.0, y: 0.0, z: b0.x + 1.0 },
    };
    let norm = (b0.x - n.x).hypot(b0.z - n.z).max(1e-3);
    let at = Vec3 { x: b0.x + STEP * (n.x - b0.x) / norm, y: 0.0, z: b0.z + STEP * (n.z - b0.z) / norm };
    Some(if steep(at) { b0 } else { at })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    #[test]
    fn the_start_is_the_densest_cluster_in_the_box_a_step_toward_its_neighbour() {
        // His box the east strip; a lone spot near its centre and a cluster of three at its north end, whose ends
        // lie 650 apart (past the claim) and its middle within the claim of both.
        let b = StartBox { ally_team: 1, left: 6000.0, top: 0.0, right: 8000.0, bottom: 6000.0 };
        let spots = [at(7000.0, 3000.0), at(6500.0, 700.0), at(6800.0, 600.0), at(7100.0, 450.0), at(1000.0, 3000.0)];
        let got = guess(&spots, &b, &[at(1000.0, 3000.0)], |_| 100.0, |_| false).unwrap();
        // The middle (6800, 600) scores 20 and a little; its nearest is (6500, 700) at 316: a step of 120 toward it.
        let (want_x, want_z) = (6800.0 - 120.0 * 300.0 / 316.23, 600.0 + 120.0 * 100.0 / 316.23);
        assert!((got.x - want_x).abs() < 0.5 && (got.z - want_z).abs() < 0.5, "{got:?}");
        // Steep there: the spot itself.
        let on_spot = guess(&spots, &b, &[], |_| 100.0, |p| p.x < 6790.0).unwrap();
        assert_eq!((on_spot.x, on_spot.z), (6800.0, 600.0));
    }

    #[test]
    fn a_neighbour_sharing_a_coordinate_sends_the_step_the_games_odd_way() {
        // The middle of three in a row 300 apart; its nearest shares its z, and the game steps toward (x + 1, x + 1).
        let b = StartBox { ally_team: 1, left: 6000.0, top: 0.0, right: 8000.0, bottom: 6000.0 };
        let spots = [at(6500.0, 600.0), at(6800.0, 600.0), at(7100.0, 600.0)];
        let got = guess(&spots, &b, &[], |_| 0.0, |_| false).unwrap();
        assert!((got.x - 6800.0).abs() < 0.1 && (got.z - 720.0).abs() < 0.1, "{got:?}");
    }

    #[test]
    fn a_claimed_spot_is_no_start_and_an_empty_box_takes_a_walkable_one() {
        let b = StartBox { ally_team: 1, left: 0.0, top: 0.0, right: 1000.0, bottom: 1000.0 };
        // The one spot inside is claimed by a start beside it; the one outside lies 500 off the box's east edge.
        let spots = [at(500.0, 500.0), at(1500.0, 400.0)];
        let got = guess(&spots, &b, &[at(600.0, 500.0)], |_| 0.0, |_| false).unwrap();
        assert_eq!((got.x, got.z), (1000.0, 400.0), "the nearest point of the box to the walkable spot");
        assert!(guess(&[at(3000.0, 400.0)], &b, &[], |_| 0.0, |_| false).is_none(), "nothing within the walk: the game's second guess");
    }
}
