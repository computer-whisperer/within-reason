//! Formation geometry for H-MICRO-FORM (`docs/design/2026-09-28-body-shape.md`, after
//! `docs/design/2026-09-25-formation-micro.md`): where a group's slots lie and who takes which slot. Pure functions
//! over positions; the group (`group.rs`) carries the anchor the ranks hang from, and the lane (`lib.rs`) decides what
//! order each slot becomes. Which soldiers make a group is the host's declaration (`Lane::set_group`), not an inference
//! here (bodies by chaining went 2026-10-24, `docs/design/2026-10-24-formation-drills.md` §2).
//!
//! A group stands in ranks across its heading, on the march and at contact alike: the contact ring round a target
//! (H-MICRO-FORM-ARC) went 2026-10-28 (`docs/design/2026-10-28-contact-cleanup.md`).
//!
//! The pros give a group one point per unit on a nearly straight line about two hulls apart and fight in bodies of
//! about six (`docs/knowledge/formations.md`, K-form-*); a ball loses to a few area units, worse the bigger and tighter
//! it is (K-units-duel-spacing-decides-area-damage).

use bot_protocol::Vec3;

/// Elmos between neighbouring slots when nothing with an area weapon is about and the hulls are small: two hulls of
/// the first tier's (the pros' median spacing between their points is 64, their soldiers stand 67-70 from the nearest
/// friend at contact).
pub const HULL_SPACING: f32 = 64.0;
/// Against area weapons the spacing is this many times the largest blast radius about (two radii: a shell that lands
/// on one unit reaches no neighbour, one that lands between two reaches both only at its edge), at least the hull
/// spacing and at most `SPACING_MAX` (H-MICRO-FORM-SPACING).
pub const AREA_SPACING: f32 = 2.0;
/// The widest spacing against area weapons: the bench's 160 against a Fatboy (radius 150) and Bulls (65), where 25
/// Stouts went from -0.55 to -0.22 and 21 against Bulls from -0.34 to +0.01 (`docs/studies/2026-09-28-body-sizes.md`,
/// main tree).
pub const SPACING_MAX: f32 = 160.0;
/// Slots in a rank on the march: the pros' body at the median (6), and their drawn lines' width (279 elmos across the
/// travel). Further ranks stand a rank gap behind, offset half a spacing.
pub const FILES: usize = 6;
/// Elmos between ranks for small hulls.
pub const RANK_GAP: f32 = 96.0;
/// H-MICRO-FORM-HULLS: neighbours in a rank stand at least this many of the formation's hull apart (`widest_pair`
/// of its members' pathing hulls, `MoveClass::footprint`: the side of the square the engine paths and collides a
/// unit by), and the ranks at least `HULL_DEEP` of it. The 64 and 96 above are the first tier's (hulls of 24 to 40); a Marauder's hull is 72 and a
/// Thor's 104, and a body sent to slots nearer than its hulls cannot stand in them: its members block each other, the
/// engine ends a blocked unit's move short of its point (K-engine-a-blocked-move-ends-short), and the anchor waits for
/// it.
pub const HULL_ACROSS: f32 = 1.5;
pub const HULL_DEEP: f32 = 1.5;

/// The distances between a formation's slots: `across` between neighbours in a rank, `deep` between ranks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pitch {
    pub across: f32,
    pub deep: f32,
}

/// The pitch of small hulls with no area weapon about.
pub const HULL_PITCH: Pitch = Pitch { across: HULL_SPACING, deep: RANK_GAP };

/// A unit vector from `from` toward `to`; east when they coincide.
pub fn heading(from: Vec3, to: Vec3) -> (f32, f32) {
    let (dx, dz) = (to.x - from.x, to.z - from.z);
    let len = dx.hypot(dz);
    if len < 1.0 { (1.0, 0.0) } else { (dx / len, dz / len) }
}

pub fn centroid(points: impl Iterator<Item = Vec3>) -> Option<Vec3> {
    let mut n = 0.0;
    let mut sum = Vec3::default();
    for p in points {
        n += 1.0;
        sum = Vec3 { x: sum.x + p.x, y: 0.0, z: sum.z + p.z };
    }
    (n > 0.0).then(|| Vec3 { x: sum.x / n, y: 0.0, z: sum.z / n })
}

/// The spacing a body keeps given the largest blast radius of anything of theirs about (0 for none):
/// H-MICRO-FORM-SPACING.
pub fn spacing_for(area: f32, factor: f32, max: f32) -> f32 {
    (area * factor).clamp(HULL_SPACING, max.max(HULL_SPACING))
}

/// The hull a formation's pitch is for, from its members' pathing hulls, largest first: the mean of the two largest,
/// which is the room the widest pair that can stand side by side takes between its centres (a lone Thor among
/// Marauders needs 88 to its neighbour, not a Thor's 104; two Thors need 104); a lone member's own; 0 for nobody.
pub fn widest_pair(hulls_descending: &[f32]) -> f32 {
    match hulls_descending {
        [] => 0.0,
        [one] => *one,
        [first, second, ..] => (first + second) / 2.0,
    }
}

/// The pitch a body keeps: across, the area spacing (`spacing_for`) or `across` of its hull, whichever is wider;
/// deep, the rank gap or `deep` of that hull (H-MICRO-FORM-HULLS). `hull` is the formation's (`widest_pair`), in
/// elmos, 0 when unknown.
pub fn pitch_for(spacing: f32, hull: f32, across: f32, deep: f32) -> Pitch {
    Pitch { across: spacing.max(hull * across), deep: RANK_GAP.max(hull * deep) }
}

/// The ranks' places across the facing: for each rank, front first, each slot's distance from the rank's centre
/// along the left of the facing, ascending (`slots` lays them; `Ground::fit_rank` fits a rank to the ground by them).
/// The ranks fill from the front and the remainder stands in the last rank, centred (design D14: a joiner adds a slot
/// behind and the body's slots stand; ranks as even as possible moved every slot when one unit joined, 12 as 6 / 6
/// to 13 as 5 / 5 / 3); a rank with the front's parity is shifted half a spacing (a checkerboard, so a rear unit
/// looks between two front ones), one with the other parity already falls on the front's gaps when centred.
pub fn rank_sides(count: usize, across: f32, files: usize) -> Vec<Vec<f32>> {
    let per_rank = files.max(1);
    let ranks = count.div_ceil(per_rank).max(1);
    let front = per_rank.min(count);
    (0..ranks)
        .map(|rank| {
            let in_rank = (count - rank * per_rank).min(per_rank);
            let offset = if rank % 2 == 1 && in_rank % 2 == front % 2 { across / 2.0 } else { 0.0 };
            (0..in_rank).map(|file| (file as f32 - (in_rank as f32 - 1.0) / 2.0) * across + offset).collect()
        })
        .collect()
}

/// `count` slots in ranks of `files` across `h`, `pitch.across` apart, the front rank centred on `anchor` and each
/// further rank `pitch.deep` behind it (`rank_sides`); within a rank left to right (across ascending), front rank
/// first. The march's shape.
pub fn slots(anchor: Vec3, h: (f32, f32), count: usize, pitch: Pitch, files: usize) -> Vec<Vec3> {
    let left = (-h.1, h.0);
    let mut out = Vec::with_capacity(count);
    for (rank, sides) in rank_sides(count, pitch.across, files).iter().enumerate() {
        let back = rank as f32 * pitch.deep;
        for side in sides {
            out.push(Vec3 { x: anchor.x + left.0 * side - h.0 * back, y: anchor.y, z: anchor.z + left.1 * side - h.1 * back });
        }
    }
    out
}

/// Each unit's slot by the least total squared travel: an exact assignment (Kuhn-Munkres with potentials, cubic in
/// the count; a few dozen units take microseconds), so a turn or a joiner costs the body the least movement and
/// nobody walks past a slot it could have had (design D14). The sort-and-swap before it left 4-11 of a turning body
/// of 13-20 within the formed band on the circuit's corners where the exact deal with the anchor on the body leaves all.
pub fn assign(units: &[Vec3], slots: &[Vec3]) -> Vec<usize> {
    let n = units.len();
    debug_assert_eq!(n, slots.len());
    if n == 0 {
        return Vec::new();
    }
    let cost = |i: usize, j: usize| -> f64 {
        let d = units[i].dist2d(slots[j]) as f64;
        d * d
    };
    // Rows the units, columns the slots, both 1-based; `p[j]` the unit holding slot `j` (0: nobody).
    let mut u = vec![0.0f64; n + 1];
    let mut v = vec![0.0f64; n + 1];
    let mut p = vec![0usize; n + 1];
    let mut way = vec![0usize; n + 1];
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0usize;
        let mut minv = vec![f64::INFINITY; n + 1];
        let mut used = vec![false; n + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = f64::INFINITY;
            let mut j1 = 0usize;
            for j in 1..=n {
                if used[j] {
                    continue;
                }
                let cur = cost(i0 - 1, j - 1) - u[i0] - v[j];
                if cur < minv[j] {
                    minv[j] = cur;
                    way[j] = j0;
                }
                if minv[j] < delta {
                    delta = minv[j];
                    j1 = j;
                }
            }
            for j in 0..=n {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut slot_of = vec![0usize; n];
    for j in 1..=n {
        if p[j] > 0 {
            slot_of[p[j] - 1] = j - 1;
        }
    }
    slot_of
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    #[test]
    fn spacing_is_the_hull_without_area_and_two_radii_with_it_up_to_the_cap() {
        assert_eq!(spacing_for(0.0, AREA_SPACING, SPACING_MAX), HULL_SPACING);
        // A Stout's shell (radius 24) and a Centurion's (5.5) leave the hull spacing.
        assert_eq!(spacing_for(24.0, AREA_SPACING, SPACING_MAX), HULL_SPACING);
        // A Bull's (65): 130; a Fatboy's (150): the cap.
        assert_eq!(spacing_for(65.0, AREA_SPACING, SPACING_MAX), 130.0);
        assert_eq!(spacing_for(150.0, AREA_SPACING, SPACING_MAX), SPACING_MAX);
    }

    /// H-MICRO-FORM-HULLS: the first tier's hulls (24 to 40) leave the pitch at 64 by 96; a Bull's or a Starlight's
    /// (56) widens the rank; a Marauder's (72) and a Thor's (104) widen and deepen it; an area spacing wider than
    /// the hulls' stands.
    #[test]
    fn the_pitch_is_the_small_hulls_until_the_largest_hull_needs_more() {
        let pitch = |spacing: f32, hull: f32| pitch_for(spacing, hull, HULL_ACROSS, HULL_DEEP);
        assert_eq!(pitch(HULL_SPACING, 0.0), HULL_PITCH);
        assert_eq!(pitch(HULL_SPACING, 40.0), HULL_PITCH);
        assert_eq!(pitch(HULL_SPACING, 56.0), Pitch { across: 84.0, deep: RANK_GAP });
        assert_eq!(pitch(HULL_SPACING, 72.0), Pitch { across: 108.0, deep: 108.0 });
        assert_eq!(pitch(HULL_SPACING, 104.0), Pitch { across: 156.0, deep: 156.0 });
        assert_eq!(pitch(160.0, 72.0), Pitch { across: 160.0, deep: 108.0 });
        // The formation's hull is its widest pair's: a lone Thor among Marauders, two Thors, one unit, nobody.
        assert_eq!(widest_pair(&[104.0, 72.0, 72.0]), 88.0);
        assert_eq!(widest_pair(&[104.0, 104.0, 72.0]), 104.0);
        assert_eq!(widest_pair(&[72.0]), 72.0);
        assert_eq!(widest_pair(&[]), 0.0);
        // The ranks stand the pitch's depth apart.
        let s = slots(at(500.0, 500.0), (1.0, 0.0), 12, pitch(HULL_SPACING, 104.0), FILES);
        assert!((s[0].x - s[6].x - 156.0).abs() < 0.01 && (s[0].dist2d(s[1]) - 156.0).abs() < 0.01, "{:?} {:?} {:?}", s[0], s[1], s[6]);
    }

    #[test]
    fn slots_lie_across_the_heading_two_hulls_apart_centred_on_the_anchor() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 3, HULL_PITCH, FILES);
        assert_eq!(s.len(), 3);
        assert!((s[1].x - 500.0).abs() < 0.01 && (s[1].z - 500.0).abs() < 0.01);
        assert!((s[0].dist2d(s[1]) - HULL_SPACING).abs() < 0.01);
        assert!((s[0].x - 500.0).abs() < 0.01, "the rank runs across the heading: {s:?}");
    }

    #[test]
    fn a_body_over_six_stands_in_ranks_the_rear_offset_between_the_front() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 11, HULL_PITCH, FILES);
        assert_eq!(s.len(), 11);
        let front: Vec<&Vec3> = s.iter().filter(|p| (p.x - 500.0).abs() < 0.01).collect();
        let rear: Vec<&Vec3> = s.iter().filter(|p| (p.x - (500.0 - RANK_GAP)).abs() < 0.01).collect();
        assert_eq!((front.len(), rear.len()), (6, 5));
        let front_z: Vec<f32> = front.iter().map(|p| p.z).collect();
        for p in rear {
            assert!(front_z.iter().all(|z| (z - p.z).abs() > HULL_SPACING / 4.0), "rear slot {p:?} sits behind a front unit");
        }
    }

    #[test]
    fn ranks_fill_from_the_front_and_a_joiner_adds_a_slot_behind() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let twelve = slots(at(500.0, 500.0), h, 12, HULL_PITCH, FILES);
        let thirteen = slots(at(500.0, 500.0), h, 13, HULL_PITCH, FILES);
        let fourteen = slots(at(500.0, 500.0), h, 14, HULL_PITCH, FILES);
        for (a, b) in twelve.iter().zip(&thirteen) {
            assert!(a.dist2d(*b) < 0.01, "a joiner moved a body slot: {a:?} to {b:?}");
        }
        assert!((thirteen[12].x - (500.0 - 2.0 * RANK_GAP)).abs() < 0.01 && (thirteen[12].z - 500.0).abs() < 0.01, "the thirteenth alone in a third rank, centred: {:?}", thirteen[12]);
        let third: Vec<&Vec3> = fourteen.iter().filter(|p| (p.x - (500.0 - 2.0 * RANK_GAP)).abs() < 0.01).collect();
        assert_eq!(third.len(), 2);
    }

    #[test]
    fn assignment_is_the_least_travel_permutation() {
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(0.0, 200.0), at(0.0, 100.0)];
        let s = slots(at(150.0, 100.0), h, 3, HULL_PITCH, FILES);
        let slot_of = assign(&units, &s);
        assert_eq!(slot_of, vec![0, 2, 1]);
        let mut seen = slot_of.clone();
        seen.sort();
        assert_eq!(seen, vec![0, 1, 2], "a permutation");
    }

    #[test]
    fn no_pair_would_be_shorter_swapped() {
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(300.0, 64.0), at(0.0, 128.0), at(300.0, 192.0)];
        let s = slots(at(400.0, 96.0), h, 4, HULL_PITCH, FILES);
        let slot_of = assign(&units, &s);
        for i in 0..4 {
            for j in i + 1..4 {
                let now = units[i].dist2d(s[slot_of[i]]) + units[j].dist2d(s[slot_of[j]]);
                let swapped = units[i].dist2d(s[slot_of[j]]) + units[j].dist2d(s[slot_of[i]]);
                assert!(swapped + 1.0 >= now, "pair {i},{j} would be shorter swapped");
            }
        }
    }

    #[test]
    fn the_deal_beats_or_matches_every_permutation() {
        // Seven units scattered about a turned formation: the exact deal's squared travel is the least of all 5,040.
        let units = [at(10.0, -150.0), at(-80.0, 40.0), at(120.0, 90.0), at(-30.0, -60.0), at(200.0, -20.0), at(60.0, 170.0), at(-140.0, 120.0)];
        let s = slots(at(0.0, 0.0), (0.0, 1.0), 7, HULL_PITCH, FILES);
        let slot_of = assign(&units, &s);
        let total = |perm: &[usize]| -> f64 { perm.iter().enumerate().map(|(i, j)| (units[i].dist2d(s[*j]) as f64).powi(2)).sum() };
        let mut seen = slot_of.clone();
        seen.sort();
        assert_eq!(seen, (0..7).collect::<Vec<_>>(), "a permutation");
        let best = total(&slot_of);
        fn permutations(k: usize, perm: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
            if k == perm.len() {
                out.push(perm.clone());
                return;
            }
            for i in k..perm.len() {
                perm.swap(k, i);
                permutations(k + 1, perm, out);
                perm.swap(k, i);
            }
        }
        let mut all = Vec::new();
        permutations(0, &mut (0..7).collect(), &mut all);
        assert_eq!(all.len(), 5040);
        for perm in &all {
            assert!(total(perm) + 1e-3 >= best, "{perm:?} travels less: {} against {best}", total(perm));
        }
    }
}
