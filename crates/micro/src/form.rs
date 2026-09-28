//! Body geometry for H-MICRO-FORM (`docs/design/2026-09-28-body-shape.md`, after
//! `docs/design/2026-09-25-formation-micro.md`): which soldiers make a body, where a body's slots lie, and who takes
//! which slot. Pure functions over positions; the lane (`lib.rs`) decides what order each slot becomes.
//!
//! A shape is slots round a reference from four numbers: the spacing `s`, the width and ranks (the march), the
//! curvature and the pitch (contact). On the march a body stands in ranks across its heading. At contact it stands on
//! the near side of the set of points within `R - m` of its target (`R` the body's shortest reach, `m` a margin): a
//! point target (a turret, a tier-2 unit, a small party) makes that a concave arc centred on it, a party wider than it
//! is deep a rank along it whose ends curl round its ends. Every slot has the target in reach at once, and a body
//! whose count exceeds the near half (about `pi * (R - m) / s` on a point: nine Blitzes at 65, seventeen Stouts)
//! stands on further arcs round the target's sides, never in a queue behind the first.
//!
//! The pros give a group one point per unit on a nearly straight line about two hulls apart, fight in bodies of about
//! six, and keep raiders and line units apart (`docs/knowledge/formations.md`, K-form-*); a ball loses to a few area
//! units, worse the bigger and tighter it is (K-units-duel-spacing-decides-area-damage).

use std::f32::consts::PI;

use bot_protocol::{UnitId, Vec3};

/// Elmos between neighbouring slots when nothing with an area weapon is about: two hulls (the pros' median spacing
/// between their points is 64, their soldiers stand 67-70 from the nearest friend at contact).
pub const HULL_SPACING: f32 = 64.0;
/// Against area weapons the spacing is this many times the largest blast radius about (two radii: a shell that lands
/// on one unit reaches no neighbour, one that lands between two reaches both only at its edge), at least the hull
/// spacing and at most `SPACING_MAX` (H-MICRO-FORM-SPACING).
pub const AREA_SPACING: f32 = 2.0;
/// The widest spacing: the bench's 160 against a Fatboy (radius 150) and Bulls (65), where 25 Stouts went from -0.55
/// to -0.22 and 21 against Bulls from -0.34 to +0.01 (`docs/studies/2026-09-28-body-sizes.md`, main tree).
pub const SPACING_MAX: f32 = 160.0;
/// Slots in a rank on the march: the pros' body at the median (6), and their drawn lines' width (279 elmos across the
/// travel). Further ranks stand `RANK_GAP` behind, offset half a spacing.
pub const FILES: usize = 6;
pub const RANK_GAP: f32 = 96.0;
/// Units under one order this close to each other are one body (chained).
pub const BODY_LINK: f32 = 400.0;
/// Two group orders whose points are this close are the same order.
pub const GROUP_POINT: f32 = 48.0;
/// A type with a reach up to this is a raider (short-range, fast); the survey's short / long split at 250 / 300.
pub const RAIDER_REACH: f32 = 250.0;
/// `m`: a slot stands this far inside the body's reach of its target (the engine's range test and ours disagree
/// within 20 at the edge).
pub const REACH_MARGIN: f32 = 20.0;
/// Enemies chained this close to the target are its party, the shape's reference.
pub const PARTY_LINK: f32 = 150.0;
/// Other buildings' reach plus this is what the arc's pitch keeps its slots out of, and what makes a soldier of theirs
/// "under cover" for the choice of target (the TAS: fight the mobile units where the static ones cannot reach).
pub const STATIC_MARGIN: f32 = 40.0;
/// The pitches tried to keep the arc out of other buildings' reach, as angles about the target (radians).
const PITCHES: [f32; 9] = [0.0, 0.26, -0.26, 0.52, -0.52, 0.79, -0.79, 1.05, -1.05];

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

/// An armed enemy as a shape sees it: where, whether it walks, how far it shoots.
#[derive(Clone, Copy, Debug)]
pub struct Foe {
    pub id: UnitId,
    pub pos: Vec3,
    pub mobile: bool,
    pub reach: f32,
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

/// The spacing a body keeps given the largest blast radius of anything of theirs about (0 for none):
/// H-MICRO-FORM-SPACING.
pub fn spacing_for(area: f32, factor: f32, max: f32) -> f32 {
    (area * factor).clamp(HULL_SPACING, max.max(HULL_SPACING))
}

/// How many slots the near half of a point target's arc holds at radius `radius` and spacing `spacing`: the
/// range-against-diameter number (`pi * r / s` plus the end slot).
pub fn arc_capacity(radius: f32, spacing: f32) -> usize {
    (PI * radius / spacing.max(1.0)).floor() as usize + 1
}

/// Bodies: members under the same group order and of the same class, chained within `BODY_LINK`. Bodies of one are
/// not returned. A body is not cut by its size here: at contact its count against the arc at its reach decides how
/// many arcs it stands on (`contact_slots`).
pub fn bodies(members: &[Member]) -> Vec<Vec<Member>> {
    let n = members.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(parent: &mut [usize], i: usize) -> usize {
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
    groups.into_iter().filter(|g| g.len() >= 2).collect()
}

/// The direction along which the points lie longest (their major axis).
fn major_axis(points: &[Vec3]) -> (f32, f32) {
    let c = centroid(points.iter().copied()).unwrap_or_default();
    let (mut sxx, mut sxz, mut szz) = (0.0f32, 0.0f32, 0.0f32);
    for p in points {
        let (dx, dz) = (p.x - c.x, p.z - c.z);
        sxx += dx * dx;
        sxz += dx * dz;
        szz += dz * dz;
    }
    let angle = 0.5 * (2.0 * sxz).atan2(sxx - szz);
    (angle.cos(), angle.sin())
}

/// `count` slots in ranks of `files` across `h`, `spacing` apart, the front rank centred on `anchor` and each further
/// rank `RANK_GAP` behind it and offset half a spacing (a checkerboard, so a rear unit looks between two front
/// ones); within a rank left to right (across ascending), front rank first. The march's shape.
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

/// The boundary of the points within `r` of the segment `a`-`b` (a stadium; a circle when `a` = `b`), walked by arc
/// length: along the left side from `a` to `b`, round `b`, back along the right side, round `a`.
#[derive(Clone, Copy, Debug)]
pub struct Stadium {
    pub a: Vec3,
    pub b: Vec3,
    pub r: f32,
}

impl Stadium {
    fn frame(&self) -> ((f32, f32), (f32, f32), f32) {
        let len = self.a.dist2d(self.b);
        let u = if len < 1.0 { (1.0, 0.0) } else { ((self.b.x - self.a.x) / len, (self.b.z - self.a.z) / len) };
        let n = (-u.1, u.0);
        (u, n, if len < 1.0 { 0.0 } else { len })
    }

    pub fn perimeter(&self) -> f32 {
        let (_, _, len) = self.frame();
        2.0 * len + 2.0 * PI * self.r
    }

    /// The length of the half facing a point straight off the segment's middle: one side and two quarter caps.
    pub fn half(&self) -> f32 {
        self.perimeter() / 2.0
    }

    /// The point at arc length `t` (any real: the walk wraps).
    pub fn at(&self, t: f32) -> Vec3 {
        let (u, n, len) = self.frame();
        let p = self.perimeter();
        let t = t.rem_euclid(p.max(1e-3));
        let cap = PI * self.r;
        let off = |base: Vec3, d: (f32, f32), k: f32| Vec3 { x: base.x + d.0 * k, y: base.y, z: base.z + d.1 * k };
        if t < len {
            off(off(self.a, u, t), n, self.r)
        } else if t < len + cap {
            // Round `b` from +n through +u to -n.
            let th = (t - len) / self.r.max(1e-3);
            let d = (n.0 * th.cos() + u.0 * th.sin(), n.1 * th.cos() + u.1 * th.sin());
            off(self.b, d, self.r)
        } else if t < 2.0 * len + cap {
            off(off(self.b, u, -(t - len - cap)), n, -self.r)
        } else {
            // Round `a` from -n through -u to +n.
            let th = (t - 2.0 * len - cap) / self.r.max(1e-3);
            let d = (-n.0 * th.cos() - u.0 * th.sin(), -n.1 * th.cos() - u.1 * th.sin());
            off(self.a, d, self.r)
        }
    }

    /// The arc length of the boundary point nearest `p`.
    pub fn param_of(&self, p: Vec3) -> f32 {
        let (u, n, len) = self.frame();
        let rel = (p.x - self.a.x, p.z - self.a.z);
        let along = rel.0 * u.0 + rel.1 * u.1;
        let cap = PI * self.r;
        if len > 0.0 && (0.0..=len).contains(&along) {
            let side = rel.0 * n.0 + rel.1 * n.1;
            if side >= 0.0 { along } else { len + cap + (len - along) }
        } else if along >= len {
            let d = (p.x - self.b.x, p.z - self.b.z);
            // From +n toward +u (beyond `b` the offset has no -u part) to -n.
            let th = (d.0 * u.0 + d.1 * u.1).max(0.0).abs().atan2(d.0 * n.0 + d.1 * n.1);
            len + th * self.r
        } else {
            let d = (p.x - self.a.x, p.z - self.a.z);
            // From -n toward -u to +n.
            let th = (-(d.0 * u.0 + d.1 * u.1)).max(0.0).abs().atan2(-(d.0 * n.0 + d.1 * n.1));
            2.0 * len + cap + th * self.r
        }
    }
}

/// What a body at contact forms on: the target it chose, the stadium of its slots, the arc length facing the body
/// (before any pitch), and the party's statics it keeps out of.
#[derive(Clone, Debug)]
pub struct Contact {
    pub target: UnitId,
    /// The target and the foes chained to it.
    pub party: Vec<UnitId>,
    pub stadium: Stadium,
    pub facing: f32,
    /// Whether the target's party is a line (wider across the body's approach than the body's reach), not a point.
    pub line: bool,
    /// Armed buildings not in the target's party: their place and reach.
    pub avoid: Vec<(Vec3, f32)>,
}

/// The body's target and the shape of its party. The host's named target (an Attack order) when it is among the
/// foes; else the target it had (`keep`) while it is still among them; else the soldier of theirs nearest the body's
/// centre that no other building of theirs covers, else the nearest soldier, else (only buildings about) the nearest
/// building. Its party is
/// the foes chained within `PARTY_LINK` of it, of its kind (mobile or not); fitted with a segment along the party's
/// major axis, the slots stand `reach - REACH_MARGIN` beyond the party's own spread off that segment (at least half
/// that), so every slot has the nearest of the party in reach.
pub fn contact(centre: Vec3, named: Option<UnitId>, keep: Option<UnitId>, foes: &[Foe], reach: f32) -> Option<Contact> {
    let covered = |p: Vec3| foes.iter().any(|f| !f.mobile && f.pos.dist2d(p) < f.reach + STATIC_MARGIN);
    let nearest = |mobile: bool, open: bool| {
        foes.iter().filter(|f| f.mobile == mobile && (!open || !covered(f.pos))).min_by(|a, b| a.pos.dist2d(centre).total_cmp(&b.pos.dist2d(centre)))
    };
    let find = |id: UnitId| foes.iter().find(|f| f.id == id);
    let target = named
        .and_then(find)
        .or_else(|| keep.and_then(find))
        .or_else(|| nearest(true, true))
        .or_else(|| nearest(true, false))
        .or_else(|| nearest(false, false))?;
    let mut party = vec![*target];
    let mut i = 0;
    while i < party.len() {
        let p = party[i].pos;
        for f in foes.iter().filter(|f| f.mobile == target.mobile) {
            if !party.iter().any(|q| q.id == f.id) && f.pos.dist2d(p) <= PARTY_LINK {
                party.push(*f);
            }
        }
        i += 1;
    }
    let points: Vec<Vec3> = party.iter().map(|f| f.pos).collect();
    let c = centroid(points.iter().copied()).unwrap_or(target.pos);
    let (a, b, spread) = if party.len() < 2 {
        (target.pos, target.pos, 0.0)
    } else {
        let axis = major_axis(&points);
        let t = |p: &Vec3| (p.x - c.x) * axis.0 + (p.z - c.z) * axis.1;
        let off = |p: &Vec3| ((p.x - c.x) * -axis.1 + (p.z - c.z) * axis.0).abs();
        let lo = points.iter().map(t).fold(f32::INFINITY, f32::min);
        let hi = points.iter().map(t).fold(f32::NEG_INFINITY, f32::max);
        let spread = points.iter().map(off).fold(0.0, f32::max);
        (Vec3 { x: c.x + axis.0 * lo, y: c.y, z: c.z + axis.1 * lo }, Vec3 { x: c.x + axis.0 * hi, y: c.y, z: c.z + axis.1 * hi }, spread)
    };
    let stand = reach - REACH_MARGIN;
    let r = (stand - spread).max(stand / 2.0).max(1.0);
    let stadium = Stadium { a, b, r };
    let h = heading(centre, c);
    let width = {
        let xs: Vec<f32> = points.iter().map(|p| across(*p, h)).collect();
        xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max) - xs.iter().cloned().fold(f32::INFINITY, f32::min)
    };
    let avoid = foes.iter().filter(|f| !f.mobile && !party.iter().any(|q| q.id == f.id)).map(|f| (f.pos, f.reach)).collect();
    Some(Contact { target: target.id, party: party.iter().map(|f| f.id).collect(), stadium, facing: stadium.param_of(centre), line: width > reach, avoid })
}

/// A body's slots at contact: `count` slots `spacing` apart along the stadium, centred on the point facing the body
/// turned by `pitch` (an angle about the target), the near half first (the first arc) and the rest continuing round
/// both sides (the further arcs). A count the full ring does not hold at `spacing` closes the spacing to fit it, down
/// to the hull's (a ring inside would stand among the enemy: the first smoke put 10 of 25 Stouts there); beyond that
/// a second ring `RANK_GAP` outside, offset half a spacing, looks between the first. Returns the slots, the pitch taken
/// (the one of `PITCHES` that leaves the fewest slots inside the reach of a building to avoid, the smallest turn among
/// equals), and how many arcs the body stands on (its count over the near half's).
pub fn contact_slots(contact: &Contact, count: usize, spacing: f32) -> (Vec<Vec3>, f32, usize) {
    let st = contact.stadium;
    let near = ((st.half() / spacing).floor() as usize + 1).max(1);
    let arcs = count.div_ceil(near).max(1);
    let lay = |pitch: f32| -> Vec<Vec3> {
        let mut out = Vec::with_capacity(count);
        let mut ring = st;
        let mut left = count;
        let mut centre = contact.facing + pitch * st.r;
        while left > 0 {
            let spacing = spacing.min(ring.perimeter() / left as f32).max(HULL_SPACING);
            let fits = ((ring.perimeter() / spacing).floor() as usize).max(1);
            let here = left.min(fits);
            for i in 0..here {
                // Out from the centre, alternating sides: 0, +1, -1, +2, ... for an odd count, +1/2, -1/2, +3/2, ...
                // for an even one.
                let step = if here % 2 == 1 {
                    let j = i.div_ceil(2) as f32;
                    if i % 2 == 1 { j } else { -j }
                } else {
                    let j = (i / 2) as f32 + 0.5;
                    if i % 2 == 0 { j } else { -j }
                };
                out.push(ring.at(centre + step * spacing));
            }
            left -= here;
            let outer = Stadium { r: ring.r + RANK_GAP, ..ring };
            centre = outer.param_of(ring.at(centre)) + spacing / 2.0;
            ring = outer;
        }
        out
    };
    let inside = |slots: &[Vec3]| slots.iter().filter(|p| contact.avoid.iter().any(|(at, reach)| p.dist2d(*at) < reach + STATIC_MARGIN)).count();
    let mut best = (usize::MAX, 0.0f32, Vec::new());
    for pitch in PITCHES {
        let slots = lay(pitch);
        let n = if contact.avoid.is_empty() { 0 } else { inside(&slots) };
        if n < best.0 {
            best = (n, pitch, slots);
        }
        if n == 0 {
            break;
        }
    }
    // What no pitch clears stays where it is: a slot pushed out of a turret's reach left its unit out of reach of the
    // target and inside the target's own (the first E3 build: -0.14 against the old lane's +0.11 on the 900 cut).
    let (_, pitch, slots) = best;
    (slots, pitch, arcs)
}

/// Which slot each unit takes (index into `slots`, one per unit, a permutation): units in across order take slots
/// in across order (nobody crosses anybody), then pairs are swapped while any swap shortens the total travel.
pub fn assign(units: &[Vec3], slots: &[Vec3], h: (f32, f32)) -> Vec<usize> {
    let n = units.len();
    debug_assert_eq!(n, slots.len());
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|a, b| across(units[*a], h).total_cmp(&across(units[*b], h)));
    let mut by_slot: Vec<usize> = (0..n).collect();
    by_slot.sort_by(|a, b| across(slots[*a], h).total_cmp(&across(slots[*b], h)));
    let mut slot_of = vec![0usize; n];
    for (rank, unit) in order.into_iter().enumerate() {
        slot_of[unit] = by_slot[rank];
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

    fn foe(id: i32, x: f32, z: f32, mobile: bool, reach: f32) -> Foe {
        Foe { id: UnitId(id), pos: at(x, z), mobile, reach }
    }

    const STOUT: f32 = 350.0;

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
        let mut bodies = bodies(&members);
        bodies.sort_by_key(|b| b[0].id.0);
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0].iter().map(|m| m.id.0).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(bodies[1].iter().map(|m| m.id.0).collect::<Vec<_>>(), vec![3, 4]);
    }

    #[test]
    fn a_big_body_is_not_cut_by_its_size() {
        let go = Order::Fight(at(3000.0, 320.0));
        let members: Vec<Member> = (0..25).map(|i| member(i, 100.0, i as f32 * 50.0, false, go)).collect();
        assert_eq!(bodies(&members).len(), 1);
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

    #[test]
    fn the_arc_holds_nine_blitzes_at_65_and_seventeen_stouts_at_the_hull() {
        assert_eq!(arc_capacity(180.0 - REACH_MARGIN, 65.0), 8);
        assert_eq!(arc_capacity(STOUT - REACH_MARGIN, HULL_SPACING), 17);
    }

    #[test]
    fn the_stadium_walk_and_its_inverse_agree() {
        for st in [Stadium { a: at(0.0, 0.0), b: at(0.0, 0.0), r: 300.0 }, Stadium { a: at(0.0, 0.0), b: at(400.0, 100.0), r: 200.0 }] {
            let p = st.perimeter();
            for k in 0..40 {
                let t = p * k as f32 / 40.0;
                let q = st.at(t);
                let back = st.param_of(Vec3 { x: q.x, y: 0.0, z: q.z });
                let err = (back - t).abs().min(p - (back - t).abs());
                assert!(err < 1.0, "t {t} -> {q:?} -> {back}");
            }
        }
    }

    #[test]
    fn seventeen_stouts_on_a_point_all_have_it_in_reach_on_one_arc_at_the_hull_spacing() {
        let foes = [foe(100, 1000.0, 0.0, true, 700.0)];
        let contact = contact(at(0.0, 0.0), None, None, &foes, STOUT).expect("a target");
        assert_eq!(contact.target, UnitId(100));
        let (slots, pitch, arcs) = contact_slots(&contact, 17, HULL_SPACING);
        assert_eq!((slots.len(), pitch, arcs), (17, 0.0, 1));
        for s in &slots {
            let d = s.dist2d(foes[0].pos);
            assert!((d - (STOUT - REACH_MARGIN)).abs() < 1.0, "slot {s:?} at {d} from the target");
            // The near half: no slot beyond the target from where the body comes.
            assert!(s.x <= 1000.0 + 1.0, "slot {s:?} behind the target");
        }
        // Neighbours a spacing apart along the arc (the chord is a little shorter).
        let mut by_z: Vec<&Vec3> = slots.iter().collect();
        by_z.sort_by(|a, b| a.z.total_cmp(&b.z));
        for w in by_z.windows(2) {
            let d = w[0].dist2d(*w[1]);
            assert!(d > HULL_SPACING - 2.0 && d <= HULL_SPACING + 0.5, "neighbours {d} apart");
        }
    }

    #[test]
    fn twenty_five_stouts_on_a_point_stand_on_two_arcs_all_in_reach() {
        let foes = [foe(100, 1000.0, 0.0, true, 700.0)];
        let contact = contact(at(0.0, 0.0), None, None, &foes, STOUT).expect("a target");
        let (slots, _, arcs) = contact_slots(&contact, 25, HULL_SPACING);
        assert_eq!((slots.len(), arcs), (25, 2));
        assert!(slots.iter().all(|s| s.dist2d(foes[0].pos) <= STOUT - REACH_MARGIN + 1.0));
        // Eight of them round the far side: beyond the target from the body.
        assert_eq!(slots.iter().filter(|s| s.x > 1000.0 + 1.0).count(), 8);
    }

    #[test]
    fn a_count_the_ring_does_not_hold_closes_the_spacing_and_never_stands_inside() {
        // Twenty-five Stouts at the Fatboys' 160 round four Fatboys in a row: 15 would fit; all 25 go on the one
        // ring at its perimeter over 25, none nearer the party than the ring.
        let foes: Vec<Foe> = (0..4).map(|i| foe(100 + i, 1000.0, i as f32 * 56.0, true, 700.0)).collect();
        let contact = contact(at(0.0, 84.0), None, None, &foes, STOUT).expect("a target");
        let (slots, _, arcs) = contact_slots(&contact, 25, SPACING_MAX);
        assert_eq!(slots.len(), 25);
        assert!(arcs >= 2);
        let ring = STOUT - REACH_MARGIN;
        for s in &slots {
            let d = foes.iter().map(|f| f.pos.dist2d(*s)).fold(f32::INFINITY, f32::min);
            assert!(d >= ring - 1.0 && d <= ring + 30.0, "slot {s:?} at {d} from the nearest Fatboy");
        }
        for (i, a) in slots.iter().enumerate() {
            for b in &slots[i + 1..] {
                assert!(a.dist2d(*b) >= HULL_SPACING - 2.0, "{a:?} and {b:?} closer than two hulls");
            }
        }
    }

    #[test]
    fn a_small_party_is_a_point_and_every_slot_reaches_all_of_it() {
        // Four Fatboys in a row 56 apart across the approach.
        let foes: Vec<Foe> = (0..4).map(|i| foe(100 + i, 1000.0, i as f32 * 56.0, true, 700.0)).collect();
        let contact = contact(at(0.0, 84.0), None, None, &foes, STOUT).expect("a target");
        assert!(!contact.line);
        let (slots, _, _) = contact_slots(&contact, 6, SPACING_MAX);
        for s in &slots {
            assert!(foes.iter().any(|f| f.pos.dist2d(*s) <= STOUT - REACH_MARGIN + 1.0), "slot {s:?} reaches none");
        }
    }

    #[test]
    fn a_wide_party_is_a_line_and_the_rank_runs_along_it() {
        // Twelve Stouts in a row 64 apart across the approach: 704 wide.
        let foes: Vec<Foe> = (0..12).map(|i| foe(100 + i, 1000.0, i as f32 * 64.0, true, STOUT)).collect();
        let contact = contact(at(0.0, 352.0), None, None, &foes, STOUT).expect("a target");
        assert!(contact.line);
        let (slots, _, arcs) = contact_slots(&contact, 12, HULL_SPACING);
        assert_eq!(arcs, 1);
        // The middle ten stand on a straight rank at x = 1000 - 330.
        let straight = slots.iter().filter(|s| (s.x - (1000.0 - (STOUT - REACH_MARGIN))).abs() < 1.0).count();
        assert!(straight >= 10, "{slots:?}");
    }

    #[test]
    fn the_arc_turns_away_from_a_turret_that_is_not_its_target() {
        // A Centurion at the origin, a light turret 450 off to one side of the approach; Blitzes come from +z.
        let foes = [foe(1, 0.0, 0.0, true, 325.0), foe(2, -318.0, 318.0, false, 430.0)];
        let contact = contact(at(0.0, 800.0), None, None, &foes, 180.0).expect("a target");
        assert_eq!(contact.target, UnitId(1));
        let (slots, pitch, _) = contact_slots(&contact, 6, 65.0);
        assert!(pitch != 0.0);
        let under = |slots: &[Vec3]| slots.iter().filter(|s| s.dist2d(foes[1].pos) < 430.0 + STATIC_MARGIN).count();
        let (straight, _, _) = contact_slots(&Contact { avoid: Vec::new(), ..contact.clone() }, 6, 65.0);
        assert!(under(&slots) < under(&straight), "the turn leaves fewer under the turret");
        // Every slot still has the target in reach.
        assert!(slots.iter().all(|s| s.dist2d(foes[0].pos) <= 180.0 - REACH_MARGIN + 1.0));
    }

    #[test]
    fn a_named_target_is_taken_and_mobiles_come_before_buildings() {
        let foes = [foe(1, 100.0, 0.0, false, 430.0), foe(2, 600.0, 0.0, true, 325.0), foe(3, 900.0, 0.0, true, 325.0)];
        assert_eq!(contact(at(0.0, 0.0), None, None, &foes, 180.0).unwrap().target, UnitId(2));
        assert_eq!(contact(at(0.0, 0.0), Some(UnitId(1)), None, &foes, 180.0).unwrap().target, UnitId(1));
        assert_eq!(contact(at(0.0, 0.0), None, None, &foes[..1], 180.0).unwrap().target, UnitId(1));
        // The target it had is kept while it is there, though another is nearer now.
        assert_eq!(contact(at(0.0, 0.0), None, Some(UnitId(3)), &foes, 180.0).unwrap().target, UnitId(3));
    }

    #[test]
    fn a_soldier_under_a_turret_is_taken_after_one_in_the_open() {
        // A Centurion 100 from a turret and nearer, another 700 from it.
        let foes = [foe(1, 0.0, 0.0, false, 430.0), foe(2, 100.0, 0.0, true, 325.0), foe(3, 700.0, 0.0, true, 325.0)];
        assert_eq!(contact(at(300.0, 300.0), None, None, &foes, 180.0).unwrap().target, UnitId(3));
    }

    #[test]
    fn slots_lie_across_the_heading_two_hulls_apart_centred_on_the_anchor() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 3, HULL_SPACING, FILES);
        assert_eq!(s.len(), 3);
        assert!((s[1].x - 500.0).abs() < 0.01 && (s[1].z - 500.0).abs() < 0.01);
        assert!((s[0].dist2d(s[1]) - HULL_SPACING).abs() < 0.01);
        assert!((s[0].x - 500.0).abs() < 0.01, "the rank runs across the heading: {s:?}");
    }

    #[test]
    fn a_body_over_six_stands_in_ranks_the_rear_offset_between_the_front() {
        let h = heading(at(0.0, 0.0), at(1000.0, 0.0));
        let s = slots(at(500.0, 500.0), h, 11, HULL_SPACING, FILES);
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
    fn assignment_keeps_the_across_order_and_is_a_permutation() {
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(0.0, 200.0), at(0.0, 100.0)];
        let s = slots(at(150.0, 100.0), h, 3, HULL_SPACING, FILES);
        let slot_of = assign(&units, &s, h);
        assert_eq!(slot_of, vec![0, 2, 1]);
        let mut seen = slot_of.clone();
        seen.sort();
        assert_eq!(seen, vec![0, 1, 2], "a permutation");
    }

    #[test]
    fn no_pair_would_be_shorter_swapped() {
        let h = (1.0, 0.0);
        let units = [at(0.0, 0.0), at(300.0, 64.0), at(0.0, 128.0), at(300.0, 192.0)];
        let s = slots(at(400.0, 96.0), h, 4, HULL_SPACING, FILES);
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
