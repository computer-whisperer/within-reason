//! Formation geometry for H-MICRO-FORM (`docs/design/2026-09-25-formation-micro.md`): which soldiers make a body,
//! where a body's slots lie, and who takes which slot. Pure functions over positions; the lane (`lib.rs`) decides
//! what order each slot becomes.
//!
//! The pros give a group one point per unit on a nearly straight line about two hulls apart, fight in bodies of
//! about six, and keep raiders and line units apart (`docs/knowledge/formations.md`, K-form-*).

use bot_protocol::{UnitId, Vec3};

/// Elmos between neighbouring slots: two hulls (the pros' median spacing between their points is 64).
pub const SPACING: f32 = 64.0;
/// Slots in a rank: the pros' body at the median (6), and their drawn lines' width (279 elmos across the travel).
/// A body of 11 in one rank was 700 wide and lost its ends to the enemy's ball (form-blitz-on: -0.17 against the
/// plain order); further ranks stand `RANK_GAP` behind, offset half a spacing.
pub const FILES: usize = 6;
pub const RANK_GAP: f32 = 96.0;
/// A body of more than this splits into sub-bodies side by side (ours fight 13 at the median, the pros 6).
pub const BODY_MAX: usize = 12;
/// The rank is centred this far ahead of the body's centroid, so the body keeps walking; never beyond the goal.
pub const LEAD: f32 = 150.0;
/// Units under one order this close to each other are one body (chained).
pub const BODY_LINK: f32 = 400.0;
/// Two group orders whose points are this close are the same order.
pub const GROUP_POINT: f32 = 48.0;
/// A type with a reach up to this is a raider (short-range, fast); the survey's short / long split at 250 / 300.
pub const RAIDER_REACH: f32 = 250.0;

/// The group order a soldier stands under, as far as bodies care: what kind, and where or at whom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Order {
    Fight(Vec3),
    Move(Vec3),
    Attack(UnitId),
}

impl Order {
    fn same_group(self, other: Order) -> bool {
        match (self, other) {
            (Order::Fight(a), Order::Fight(b)) | (Order::Move(a), Order::Move(b)) => a.dist2d(b) <= GROUP_POINT,
            (Order::Attack(a), Order::Attack(b)) => a == b,
            _ => false,
        }
    }
}

/// A soldier as bodies see it.
#[derive(Clone, Copy, Debug)]
pub struct Member {
    pub id: UnitId,
    pub pos: Vec3,
    pub raider: bool,
    pub order: Order,
}

/// A unit vector from `from` toward `to`; east when they coincide.
pub fn heading(from: Vec3, to: Vec3) -> (f32, f32) {
    let (dx, dz) = (to.x - from.x, to.z - from.z);
    let len = dx.hypot(dz);
    if len < 1.0 { (1.0, 0.0) } else { (dx / len, dz / len) }
}

/// The coordinate of `p` across a heading (to its left, positive).
pub fn across(p: Vec3, h: (f32, f32)) -> f32 {
    p.x * -h.1 + p.z * h.0
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

/// Bodies: members under the same group order and of the same class, chained within `BODY_LINK`; a body larger
/// than `BODY_MAX` is cut across its heading (toward its order's point, or, for an attack, across the chain's own
/// spread) into near-equal pieces of at most `BODY_MAX`. Bodies of one are not returned.
pub fn bodies(members: &[Member], target_of: impl Fn(UnitId) -> Option<Vec3>) -> Vec<Vec<Member>> {
    let n = members.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(parent: &mut Vec<usize>, i: usize) -> usize {
        let mut r = i;
        while parent[r] != r {
            r = parent[r];
        }
        let mut i = i;
        while parent[i] != r {
            let next = parent[i];
            parent[i] = r;
            i = next;
        }
        r
    }
    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = (&members[i], &members[j]);
            if a.raider == b.raider && a.order.same_group(b.order) && a.pos.dist2d(b.pos) <= BODY_LINK {
                let (ra, rb) = (root(&mut parent, i), root(&mut parent, j));
                if ra != rb {
                    parent[ra] = rb;
                }
            }
        }
    }
    let mut groups: Vec<Vec<Member>> = Vec::new();
    let mut of_root: Vec<Option<usize>> = vec![None; n];
    for i in 0..n {
        let r = root(&mut parent, i);
        match of_root[r] {
            Some(g) => groups[g].push(members[i]),
            None => {
                of_root[r] = Some(groups.len());
                groups.push(vec![members[i]]);
            }
        }
    }
    let mut out = Vec::new();
    for group in groups {
        if group.len() < 2 {
            continue;
        }
        if group.len() <= BODY_MAX {
            out.push(group);
            continue;
        }
        let centre = centroid(group.iter().map(|m| m.pos)).expect("non-empty");
        let goal = match group[0].order {
            Order::Fight(p) | Order::Move(p) => Some(p),
            Order::Attack(id) => target_of(id),
        };
        let h = match goal {
            Some(g) if g.dist2d(centre) > 1.0 => heading(centre, g),
            _ => principal_axis(&group),
        };
        let mut sorted = group;
        sorted.sort_by(|a, b| across(a.pos, h).total_cmp(&across(b.pos, h)));
        let pieces = sorted.len().div_ceil(BODY_MAX);
        let size = sorted.len().div_ceil(pieces);
        for piece in sorted.chunks(size) {
            if piece.len() >= 2 {
                out.push(piece.to_vec());
            }
        }
    }
    out
}

/// The direction along which the group is longest (its major axis), so a big body without a goal is cut across it.
fn principal_axis(group: &[Member]) -> (f32, f32) {
    let c = centroid(group.iter().map(|m| m.pos)).expect("non-empty");
    let (mut sxx, mut sxz, mut szz) = (0.0f32, 0.0f32, 0.0f32);
    for m in group {
        let (dx, dz) = (m.pos.x - c.x, m.pos.z - c.z);
        sxx += dx * dx;
        sxz += dx * dz;
        szz += dz * dz;
    }
    // The major eigenvector of the 2x2 covariance; a perpendicular heading puts the cut across the long side.
    let angle = 0.5 * (2.0 * sxz).atan2(sxx - szz);
    let major = (angle.cos(), angle.sin());
    (-major.1, major.0)
}

/// `count` slots in ranks of `files` across `h`, `spacing` apart, the front rank centred on `anchor` and each further
/// rank `RANK_GAP` behind it and offset half a spacing (a checkerboard, so a rear unit looks between two front
/// ones); within a rank left to right (across ascending), front rank first.
pub fn slots(anchor: Vec3, h: (f32, f32), count: usize, spacing: f32, files: usize) -> Vec<Vec3> {
    let left = (-h.1, h.0);
    let ranks = count.div_ceil(files.max(1)).max(1);
    let per_rank = count.div_ceil(ranks);
    let front = per_rank.min(count);
    let mut out = Vec::with_capacity(count);
    for rank in 0..ranks {
        let in_rank = (count - rank * per_rank).min(per_rank);
        // Between the front rank's units: a rank with the front's parity is shifted half a spacing, one with the
        // other parity already falls on the front's gaps when centred.
        let offset = if rank % 2 == 1 && in_rank % 2 == front % 2 { spacing / 2.0 } else { 0.0 };
        let back = rank as f32 * RANK_GAP;
        for file in 0..in_rank {
            let side = (file as f32 - (in_rank as f32 - 1.0) / 2.0) * spacing + offset;
            out.push(Vec3 { x: anchor.x + left.0 * side - h.0 * back, y: anchor.y, z: anchor.z + left.1 * side - h.1 * back });
        }
    }
    out
}

/// The rank's centre: `LEAD` ahead of the body's centroid along `h`, but never past the goal.
pub fn anchor(centre: Vec3, h: (f32, f32), goal: Option<Vec3>) -> Vec3 {
    let lead = goal.map_or(LEAD, |g| LEAD.min(g.dist2d(centre)));
    Vec3 { x: centre.x + h.0 * lead, y: centre.y, z: centre.z + h.1 * lead }
}

/// Which slot each unit takes (index into `slots`, one per unit, a permutation): units in across order take slots
/// in across order (nobody crosses anybody), then pairs are swapped while any swap shortens the total travel.
pub fn assign(units: &[Vec3], slots: &[Vec3], h: (f32, f32)) -> Vec<usize> {
    let n = units.len();
    debug_assert_eq!(n, slots.len());
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|a, b| across(units[*a], h).total_cmp(&across(units[*b], h)));
    let mut slot_of = vec![0usize; n];
    for (rank, unit) in order.into_iter().enumerate() {
        slot_of[unit] = rank;
    }
    let mut improved = true;
    let mut rounds = 0;
    while improved && rounds < 20 {
        improved = false;
        rounds += 1;
        for i in 0..n {
            for j in i + 1..n {
                let (si, sj) = (slot_of[i], slot_of[j]);
                let now = units[i].dist2d(slots[si]) + units[j].dist2d(slots[sj]);
                let swapped = units[i].dist2d(slots[sj]) + units[j].dist2d(slots[si]);
                if swapped + 1.0 < now {
                    slot_of.swap(i, j);
                    improved = true;
                }
            }
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

    fn member(id: i32, x: f32, z: f32, raider: bool, order: Order) -> Member {
        Member { id: UnitId(id), pos: at(x, z), raider, order }
    }

    #[test]
    fn raiders_and_line_units_under_one_order_are_two_bodies() {
        let go = Order::Fight(at(2000.0, 500.0));
        let members = [
            member(1, 0.0, 0.0, true, go),
            member(2, 50.0, 0.0, true, go),
            member(3, 0.0, 100.0, false, go),
            member(4, 60.0, 100.0, false, go),
            // A lone unit, and one under another order: neither is in a body.
            member(5, 900.0, 900.0, false, go),
            member(6, 30.0, 50.0, false, Order::Move(at(0.0, 0.0))),
        ];
        let mut bodies = bodies(&members, |_| None);
        bodies.sort_by_key(|b| b[0].id.0);
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0].iter().map(|m| m.id.0).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(bodies[1].iter().map(|m| m.id.0).collect::<Vec<_>>(), vec![3, 4]);
    }

    #[test]
    fn a_big_body_is_cut_across_its_heading_into_near_equal_pieces() {
        // Thirteen Stouts in a row across the way to a goal to the east: 7 and 6, the north ones together.
        let go = Order::Fight(at(3000.0, 320.0));
        let members: Vec<Member> = (0..13).map(|i| member(i, 100.0, i as f32 * 50.0, false, go)).collect();
        let bodies = bodies(&members, |_| None);
        assert_eq!(bodies.len(), 2);
        let mut sizes: Vec<usize> = bodies.iter().map(|b| b.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![6, 7]);
        for body in &bodies {
            let zs: Vec<f32> = body.iter().map(|m| m.pos.z).collect();
            let (lo, hi) = (zs.iter().cloned().fold(f32::INFINITY, f32::min), zs.iter().cloned().fold(0.0, f32::max));
            assert!(hi - lo <= 6.0 * 50.0 + 1.0, "a piece is contiguous across the heading: {zs:?}");
        }
    }

    #[test]
    fn slots_lie_across_the_heading_two_hulls_apart_centred_on_the_anchor() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 3, SPACING, FILES);
        assert_eq!(s.len(), 3);
        assert!((s[1].x - 500.0).abs() < 0.01 && (s[1].z - 500.0).abs() < 0.01);
        assert!((s[0].dist2d(s[1]) - SPACING).abs() < 0.01);
        assert!((s[0].x - 500.0).abs() < 0.01, "the rank runs across the heading: {s:?}");
    }

    #[test]
    fn a_body_over_six_stands_in_ranks_the_rear_offset_between_the_front() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 11, SPACING, FILES);
        assert_eq!(s.len(), 11);
        let front: Vec<&Vec3> = s.iter().filter(|p| (p.x - 500.0).abs() < 0.01).collect();
        let rear: Vec<&Vec3> = s.iter().filter(|p| (p.x - (500.0 - RANK_GAP)).abs() < 0.01).collect();
        assert_eq!((front.len(), rear.len()), (6, 5));
        // A rear unit stands half a spacing across from the front units: between two of them.
        let front_z: Vec<f32> = front.iter().map(|p| p.z).collect();
        for p in rear {
            assert!(front_z.iter().all(|z| (z - p.z).abs() > SPACING / 4.0), "rear slot {p:?} sits behind a front unit");
        }
    }

    #[test]
    fn the_anchor_leads_the_centroid_but_stops_at_the_goal() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let far = anchor(at(0.0, 0.0), h, Some(at(1000.0, 0.0)));
        assert!((far.x - LEAD).abs() < 0.01);
        let near = anchor(at(0.0, 0.0), h, Some(at(60.0, 0.0)));
        assert!((near.x - 60.0).abs() < 0.01);
    }

    #[test]
    fn assignment_keeps_the_across_order_and_shortens_the_total() {
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(0.0, 200.0), at(0.0, 100.0)];
        let s = slots(at(150.0, 100.0), h, 3, SPACING, FILES);
        let slot_of = assign(&units, &s, h);
        // Across ascending is z ascending here (left of east is south... across = z for h = east).
        assert_eq!(slot_of, vec![0, 2, 1]);
        let mut seen = slot_of.clone();
        seen.sort();
        assert_eq!(seen, vec![0, 1, 2], "a permutation");
    }

    #[test]
    fn a_swap_that_shortens_travel_is_taken() {
        // Two units across-ordered a, b but the slots are far off to the side such that swapping is shorter is
        // impossible on a line; check the invariant instead: no pair would be better swapped.
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(300.0, 64.0), at(0.0, 128.0), at(300.0, 192.0)];
        let s = slots(at(400.0, 96.0), h, 4, SPACING, FILES);
        let slot_of = assign(&units, &s, h);
        for i in 0..4 {
            for j in i + 1..4 {
                let now = units[i].dist2d(s[slot_of[i]]) + units[j].dist2d(s[slot_of[j]]);
                let swapped = units[i].dist2d(s[slot_of[j]]) + units[j].dist2d(s[slot_of[i]]);
                assert!(swapped + 1.0 >= now, "pair {i},{j} would be shorter swapped");
            }
        }
    }
}
