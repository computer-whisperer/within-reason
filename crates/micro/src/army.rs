//! The army of companies (`docs/design/2026-10-25-companies.md` §2): a declared group with a cap is partitioned
//! into companies of at most the cap, each a formation of its own (`group::Group`) with a role. The **lead** walks
//! the army's way (the order's route, paced); a **slaved** company hangs its anchor off the lead's at an offset, every
//! tick, and its members drag the lead's pace as the lead's own do; a **joining** company marches on its own way to
//! its slaved point and is slaved within `SLAVE_WITHIN` of it; a **mustering** company holds beside a muster point
//! (a plant) until it has `release` members or `MUSTER_PATIENCE` has passed since its first, then joins. With a cap
//! of 0 everyone is one company, the lead: the group as it was before the cap.
//!
//! The Glitters lobby game (2026-10-25): one group of 551 members over thirty minutes, fed by every plant, the median
//! member 6,931 from the anchor when it appeared; the joiners held nothing, the anchor marched at full pace with one
//! to twenty formed, and "arrived" fired on the few at the front eight times.

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{UnitId, Vec3};

use crate::ground::Ground;
use crate::group::{Cause, Group, GroupKind, GroupOrder, GroupState, Walker};

/// A newcomer this near a company's anchor (or a muster point) joins it rather than founding one.
pub const JOIN_NEAR: f32 = 800.0;
/// A joining company this near its slaved point is slaved: its anchor becomes the point and its members walk in.
pub const SLAVE_WITHIN: f32 = 600.0;
/// A joining company's route is planned again when its slaved point has moved this far since it was set.
pub const FOLLOW_STEP: f32 = 150.0;
/// A muster of `MUSTER_MIN` or more releases this long after its first member came, full or not; one of fewer at
/// twice it (a trickle goes in fours, not never, and a lone straggler not for three minutes). The fed circuit at a
/// cap of 12: the lab made 11 in two minutes, so a muster full at the cap would have waited the whole lap.
pub const MUSTER_PATIENCE: i32 = 90 * 30;
pub const MUSTER_MIN: usize = 4;
/// Between companies, abreast and in line.
pub const COMPANY_GAP: f32 = 128.0;
/// Companies abreast at most, the lead counted; the rest fall into the lines behind.
pub const ABREAST: usize = 3;
/// A joining company this near its slaved point is waited for: its median lag drags the lead as a slaved company's
/// does (a company of the same speed never catches a marching army; the army waits the few seconds it is from it).
pub const WAIT_FOR: f32 = 1600.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Lead,
    Slaved,
    Joining,
    Mustering,
}

impl Role {
    pub fn word(self) -> &'static str {
        match self {
            Role::Lead => "lead",
            Role::Slaved => "slaved",
            Role::Joining => "joining",
            Role::Mustering => "mustering",
        }
    }
}

/// One company's reading, in the army's state.
#[derive(Clone, Debug, PartialEq)]
pub struct CompanyState {
    pub role: Role,
    pub anchor: Vec3,
    pub members: usize,
    pub formed: usize,
    pub exceptions: Vec<(UnitId, Cause, f32)>,
    pub arrived: bool,
    /// Where it is bound: a joining company's slaved point, a muster's point; none for the lead and the slaved.
    pub at: Option<Vec3>,
}

pub struct Company {
    pub formation: Group,
    pub members: Vec<UnitId>,
    pub role: Role,
    /// Its own order: a muster's hold, a joining company's march to its slaved point.
    pub order: GroupOrder,
    /// The muster point and the frame its first member came.
    pub muster: Option<(Vec3, i32)>,
    /// The slaved point its joining route was last planned to.
    pub route_at: Option<Vec3>,
    /// Its place in the army's shape (the lead's is 0; the slaved and joining count from 1).
    pub slot: usize,
    pub state: Option<GroupState>,
}

#[derive(Default)]
pub struct Army {
    pub companies: Vec<Company>,
    next: usize,
    /// Whether the army read arrived at its last tick (the floor merges at a halt).
    pub arrived: bool,
}

impl Army {
    /// Which company has this member.
    pub fn company_of(&self, id: UnitId) -> Option<usize> {
        self.companies.iter().position(|c| c.members.contains(&id))
    }

    pub fn lead(&self) -> Option<usize> {
        self.companies.iter().position(|c| c.role == Role::Lead)
    }

    /// The membership this tick: members gone leave their company, newcomers are placed (§2, the partition). A
    /// company is founded with every newcomer that would pick it, so its anchor is set on all of them (`Group::new`'s
    /// dense cluster), not on the first. Returns how many companies were founded.
    pub fn partition(&mut self, name: &str, members: &[UnitId], walkers: &HashMap<UnitId, Walker>, order: &GroupOrder, frame: i32, spacing: f32, ground: &Option<Arc<Ground>>) -> usize {
        for c in &mut self.companies {
            c.members.retain(|id| members.contains(id) && walkers.contains_key(id));
        }
        self.companies.retain(|c| !c.members.is_empty());
        let mut founded = 0;
        let mut pending: Vec<UnitId> = members.iter().copied().filter(|id| walkers.contains_key(id) && self.company_of(*id).is_none()).collect();
        let cap = order.cap;
        // The companions a founding takes from the pending: those within `JOIN_NEAR` of `about`, up to the cap.
        let take_near = |pending: &mut Vec<UnitId>, about: Vec3, room: usize| -> Vec<UnitId> {
            let mut taken = Vec::new();
            pending.retain(|id| {
                if taken.len() < room && walkers[id].pos.dist2d(about) <= JOIN_NEAR {
                    taken.push(*id);
                    false
                } else {
                    true
                }
            });
            taken
        };
        while !pending.is_empty() {
            let id = pending.remove(0);
            let pos = walkers[&id].pos;
            if cap == 0 {
                match self.companies.iter().position(|c| c.role != Role::Mustering) {
                    Some(i) => self.companies[i].members.push(id),
                    None => {
                        let mut all = vec![id];
                        all.append(&mut pending);
                        let positions: Vec<Vec3> = all.iter().map(|id| walkers[id].pos).collect();
                        self.found(name, Role::Lead, all, &positions, order.clone(), None, spacing, ground);
                        founded += 1;
                    }
                }
                continue;
            }
            // (1) A marching company near enough and under the cap.
            let near = (self.companies.iter().enumerate())
                .filter(|(_, c)| c.role != Role::Mustering && c.members.len() < cap)
                .map(|(i, c)| (i, c.formation.anchor.dist2d(pos)))
                .filter(|(_, d)| *d <= JOIN_NEAR)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i);
            if let Some(i) = near {
                self.companies[i].members.push(id);
                continue;
            }
            // (2) The muster point nearest it, when one is near: its mustering company, founded on the first with
            // every pending newcomer near the same point.
            let muster = order.musters.iter().copied().map(|m| (m, m.dist2d(pos))).filter(|(_, d)| *d <= JOIN_NEAR).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(m, _)| m);
            if let Some(m) = muster {
                match self.companies.iter().position(|c| c.role == Role::Mustering && c.muster.is_some_and(|(at, _)| at.dist2d(m) < 1.0) && c.members.len() < cap) {
                    Some(i) => self.companies[i].members.push(id),
                    None => {
                        let mut all = vec![id];
                        all.extend(take_near(&mut pending, m, cap - 1));
                        let positions: Vec<Vec3> = all.iter().map(|id| walkers[id].pos).collect();
                        self.found(name, Role::Mustering, all, &positions, GroupOrder::new(GroupKind::Hold, vec![m]), Some((m, frame)), spacing, ground);
                        founded += 1;
                    }
                }
                continue;
            }
            // (3) No muster near: the smallest marching company under the cap within twice `JOIN_NEAR` when no muster
            // was given at all, else a company of its own where it stands, which joins as a body (a lone far
            // newcomer still walks): slaved at once when it stands by the lead, joining from further off.
            if order.musters.is_empty()
                && let Some(i) = (self.companies.iter().enumerate())
                    .filter(|(_, c)| c.role != Role::Mustering && c.members.len() < cap && c.formation.anchor.dist2d(pos) <= 2.0 * JOIN_NEAR)
                    .min_by_key(|(_, c)| c.members.len())
                    .map(|(i, _)| i)
            {
                self.companies[i].members.push(id);
                continue;
            }
            let joining = (self.companies.iter().enumerate())
                .filter(|(_, c)| c.role == Role::Joining && c.members.len() < cap)
                .map(|(i, c)| (i, c.formation.anchor.dist2d(pos)))
                .filter(|(_, d)| *d <= JOIN_NEAR)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i);
            match joining {
                Some(i) => self.companies[i].members.push(id),
                None => {
                    let mut all = vec![id];
                    all.extend(take_near(&mut pending, pos, cap - 1));
                    let positions: Vec<Vec3> = all.iter().map(|id| walkers[id].pos).collect();
                    let to = order.route.first().copied().unwrap_or(pos);
                    let lead_at = self.lead().map(|i| self.companies[i].formation.anchor);
                    let role = if self.lead().is_none() && self.companies.iter().all(|c| c.role == Role::Mustering) {
                        Role::Lead
                    } else if lead_at.is_some_and(|a| a.dist2d(pos) <= JOIN_NEAR) {
                        Role::Slaved
                    } else {
                        Role::Joining
                    };
                    let own = if role == Role::Lead { order.clone() } else { GroupOrder::new(GroupKind::FightTo, vec![to]) };
                    self.found(name, role, all, &positions, own, None, spacing, ground);
                    founded += 1;
                }
            }
        }
        founded
    }

    #[allow(clippy::too_many_arguments)]
    fn found(&mut self, name: &str, role: Role, members: Vec<UnitId>, positions: &[Vec3], order: GroupOrder, muster: Option<(Vec3, i32)>, spacing: f32, ground: &Option<Arc<Ground>>) {
        self.next += 1;
        let mut formation = Group::new(&format!("{name}.{}", self.next), members.clone(), order.clone(), positions, spacing);
        if ground.is_some() {
            formation.set_ground(ground.clone());
        }
        self.companies.push(Company { formation, members, role, order, muster, route_at: None, slot: 0, state: None });
    }

    /// One lead: the largest of the slaved when the lead is gone, else the largest joining company (an army born of a
    /// muster alone gets its lead from the first release). The new lead takes the army's order.
    pub fn ensure_lead(&mut self, order: &GroupOrder) {
        if self.lead().is_some() {
            return;
        }
        let pick = |role: Role, companies: &[Company]| (companies.iter().enumerate()).filter(|(_, c)| c.role == role).max_by_key(|(_, c)| c.members.len()).map(|(i, _)| i);
        let Some(i) = pick(Role::Slaved, &self.companies).or_else(|| pick(Role::Joining, &self.companies)) else { return };
        let c = &mut self.companies[i];
        c.role = Role::Lead;
        c.order = order.clone();
        c.route_at = None;
        c.formation.unslave();
        c.formation.update(c.members.clone(), order.clone());
    }

    /// The slaved and joining companies' places in the shape, 1 and up: each takes the free slot whose point is
    /// nearest where it stands, the nearest company first (a block split front and back falls into column, not
    /// across itself).
    pub fn assign_slots(&mut self, lead_anchor: Vec3, heading: (f32, f32), width: f32, depth: f32, abreast: usize) {
        let movers: Vec<usize> = (0..self.companies.len()).filter(|&i| matches!(self.companies[i].role, Role::Slaved | Role::Joining)).collect();
        for &i in &movers {
            self.companies[i].slot = 0;
        }
        for c in &mut self.companies {
            if matches!(c.role, Role::Lead | Role::Mustering) {
                c.slot = 0;
            }
        }
        let n = movers.len();
        let point = |k: usize| {
            let (dx, dz) = Army::offset(k, heading, width, depth, abreast);
            Vec3 { x: lead_anchor.x + dx, y: lead_anchor.y, z: lead_anchor.z + dz }
        };
        let mut free: Vec<usize> = (1..=n).collect();
        let mut left: Vec<usize> = movers;
        while !left.is_empty() && !free.is_empty() {
            // The (company, slot) pair nearest of all.
            let mut best: Option<(f32, usize, usize)> = None;
            for &i in &left {
                let at = self.companies[i].formation.anchor;
                for &k in &free {
                    let d = at.dist2d(point(k));
                    if best.is_none_or(|(b, ..)| d < b) {
                        best = Some((d, i, k));
                    }
                }
            }
            let (_, i, k) = best.expect("a pair");
            self.companies[i].slot = k;
            left.retain(|&j| j != i);
            free.retain(|&f| f != k);
        }
    }

    /// The floor (§2): at a halt, two lead or slaved companies both under half the cap merge into the larger, once a
    /// tick; never on the march, never above the cap.
    pub fn merge_floor(&mut self, cap: usize, halted: bool) -> bool {
        if cap == 0 || !halted {
            return false;
        }
        let small: Vec<usize> = (self.companies.iter().enumerate()).filter(|(_, c)| matches!(c.role, Role::Lead | Role::Slaved) && c.members.len() * 2 < cap).map(|(i, _)| i).collect();
        if small.len() < 2 {
            return false;
        }
        let (mut a, mut b) = (small[0], small[1]);
        if self.companies[a].members.len() < self.companies[b].members.len() || (self.companies[a].members.len() == self.companies[b].members.len() && self.companies[b].role == Role::Lead) {
            std::mem::swap(&mut a, &mut b);
        }
        // `a` is the larger (or the lead): it takes `b`'s members. A lead merged away leaves the survivor to be made
        // the lead (`ensure_lead`, which gives it the army's order).
        let gone = self.companies.remove(b);
        let a = if b < a { a - 1 } else { a };
        self.companies[a].members.extend(gone.members);
        if gone.role == Role::Lead {
            self.companies[a].role = Role::Slaved;
        }
        true
    }

    /// Where slot `k` of the shape stands relative to the lead's anchor: companies abreast first (left, then right,
    /// then further out), `abreast` to a line counting the lead, the lines behind by the company's depth and the gap.
    pub fn offset(k: usize, heading: (f32, f32), width: f32, depth: f32, abreast: usize) -> (f32, f32) {
        let abreast = abreast.max(1);
        let line = k / abreast;
        let col = k % abreast;
        // Column 0 is the lead's place; the others alternate left and right of it.
        let lateral = if col == 0 { 0.0 } else { ((col as f32 + 1.0) / 2.0).floor() * if col % 2 == 1 { 1.0 } else { -1.0 } };
        let left = (-heading.1, heading.0);
        let side = lateral * (width + COMPANY_GAP);
        let back = line as f32 * (depth + COMPANY_GAP);
        (left.0 * side - heading.0 * back, left.1 * side - heading.1 * back)
    }

    /// How many companies stand abreast for the lead's room: the room is elmos about the anchor's cell, so twice it is
    /// the width available.
    pub fn abreast_for(room: Option<f32>, width: f32) -> usize {
        match room {
            Some(r) => (((2.0 * r) / (width + COMPANY_GAP)).floor() as usize).clamp(1, ABREAST),
            None => ABREAST,
        }
    }

    /// The army's reading: the lead's numbers at the top, every company's formed and exceptions together, arrived
    /// when the lead and every slaved company are and nothing is still joining (a muster is not of the march yet).
    pub fn compose(&mut self) -> Option<GroupState> {
        let lead = self.companies.iter().find(|c| c.role == Role::Lead)?;
        let lead_state = lead.state.clone()?;
        let mut formed = Vec::new();
        let mut exceptions = Vec::new();
        let mut companies = Vec::new();
        let mut arrived = lead_state.arrived;
        let mut engaged = lead_state.engaged;
        for c in &self.companies {
            let Some(s) = &c.state else { continue };
            formed.extend(s.formed.iter().copied());
            exceptions.extend(s.exceptions.iter().cloned());
            engaged = engaged.or(s.engaged);
            match c.role {
                Role::Slaved => arrived &= s.arrived,
                Role::Joining => arrived = false,
                Role::Lead | Role::Mustering => {}
            }
            companies.push(CompanyState {
                role: c.role,
                anchor: s.anchor,
                members: c.members.len(),
                formed: s.formed.len(),
                exceptions: s.exceptions.clone(),
                arrived: s.arrived,
                at: match c.role {
                    Role::Joining => c.route_at,
                    Role::Mustering => c.muster.map(|(at, _)| at),
                    Role::Lead | Role::Slaved => None,
                },
            });
        }
        self.arrived = arrived;
        Some(GroupState { formed, exceptions, arrived, engaged, companies, ..lead_state })
    }

    /// The width of a company's ranks and the depth of its layout, from the lead's.
    pub fn shape(lead: &Group, spacing: f32) -> (f32, f32) {
        (lead.files() as f32 * spacing, lead.layout_depth() + spacing)
    }

    /// Where slaved company `k` stands this tick: the lead's anchor at the shape's offset, snapped to the company's
    /// ground.
    pub fn slaved_point(company: &Group, lead_anchor: Vec3, heading: (f32, f32), k: usize, width: f32, depth: f32, abreast: usize) -> Vec3 {
        let (dx, dz) = Army::offset(k, heading, width, depth, abreast);
        company.snap(Vec3 { x: lead_anchor.x + dx, y: lead_anchor.y, z: lead_anchor.z + dz })
    }
}

/// Whether a muster is due to release: `release` members stand (the cap when none was given), or its first member
/// has waited `MUSTER_PATIENCE`.
pub fn muster_due(members: usize, since: i32, frame: i32, order: &GroupOrder) -> bool {
    let wanted = order.release.unwrap_or(order.cap).max(1);
    let waited = frame - since;
    members >= wanted || (members >= MUSTER_MIN && waited >= MUSTER_PATIENCE) || waited >= 2 * MUSTER_PATIENCE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::GroupKind;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A block of `n` walkers about `(x, z)`, six to a rank at 64, ids from `first`.
    fn block(first: i32, n: usize, x: f32, z: f32) -> HashMap<UnitId, Walker> {
        (0..n)
            .map(|i| {
                let id = UnitId(first + i as i32);
                let pos = at(x - (i / 6) as f32 * 96.0, z + (i % 6) as f32 * 64.0);
                (id, Walker { id, pos, speed: 80.0, moving: 0.0, fleeing: false })
            })
            .collect()
    }

    fn ids(walkers: &HashMap<UnitId, Walker>) -> Vec<UnitId> {
        let mut v: Vec<UnitId> = walkers.keys().copied().collect();
        v.sort_by_key(|id| id.0);
        v
    }

    /// With no cap everyone is one company, the lead; with a cap of 24, forty-eight in a block are two companies of
    /// 24, the first the lead and the second joining it (slaved once it stands at its offset).
    #[test]
    fn the_partition_founds_companies_of_the_cap() {
        let walkers = block(1, 48, 0.0, 0.0);
        let members = ids(&walkers);
        let mut whole = Army::default();
        whole.partition("a", &members, &walkers, &GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]), 0, 64.0, &None);
        assert_eq!(whole.companies.len(), 1);
        assert_eq!((whole.companies[0].role, whole.companies[0].members.len()), (Role::Lead, 48));
        let mut capped = Army::default();
        let order = GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]).with_cap(24);
        let founded = capped.partition("a", &members, &walkers, &order, 0, 64.0, &None);
        assert_eq!(founded, 2);
        let sizes: Vec<(Role, usize)> = capped.companies.iter().map(|c| (c.role, c.members.len())).collect();
        assert_eq!(sizes, vec![(Role::Lead, 24), (Role::Slaved, 24)], "by the lead, so slaved at once: {sizes:?}");
        capped.ensure_lead(&order);
        capped.assign_slots(capped.companies[0].formation.anchor, (1.0, 0.0), 384.0, 400.0, 1);
        assert_eq!(capped.companies[1].slot, 1);
        // A member gone leaves; a newcomer near the lead joins the company with room, else founds one.
        let mut fewer = members.clone();
        fewer.retain(|id| id.0 != 3);
        capped.partition("a", &fewer, &walkers, &order, 3, 64.0, &None);
        assert_eq!(capped.companies[0].members.len(), 23);
        let mut more = walkers.clone();
        more.insert(UnitId(99), Walker { id: UnitId(99), pos: at(100.0, 100.0), speed: 80.0, moving: 0.0, fleeing: false });
        let mut all = ids(&more);
        all.retain(|id| id.0 != 3);
        capped.partition("a", &all, &more, &order, 6, 64.0, &None);
        assert!(capped.company_of(UnitId(99)).is_some_and(|i| capped.companies[i].members.len() == 24), "{:?}", capped.companies.iter().map(|c| c.members.len()).collect::<Vec<_>>());
    }

    /// A newcomer far from the army, near a muster point, musters there; the muster releases when it holds the
    /// release count, or after `MUSTER_PATIENCE`.
    #[test]
    fn a_far_newcomer_musters_and_the_muster_releases_full_or_late() {
        let mut walkers = block(1, 12, 0.0, 0.0);
        let muster = at(6000.0, 0.0);
        let mut order = GroupOrder::new(GroupKind::FightTo, vec![at(9000.0, 0.0)]).with_cap(12);
        order.musters = vec![muster];
        order.release = Some(4);
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, 0, 64.0, &None);
        assert_eq!(army.companies.len(), 1);
        for (i, id) in [101, 102, 103].iter().enumerate() {
            walkers.insert(UnitId(*id), Walker { id: UnitId(*id), pos: at(6000.0 + 40.0 * i as f32, 100.0), speed: 80.0, moving: 0.0, fleeing: false });
        }
        army.partition("a", &ids(&walkers), &walkers, &order, 30, 64.0, &None);
        assert_eq!(army.companies.len(), 2);
        let m = &army.companies[1];
        assert_eq!((m.role, m.members.len(), m.muster.map(|(at, _)| at.x)), (Role::Mustering, 3, Some(6000.0)));
        assert!(!muster_due(3, 30, 60, &order), "three of four, a second in: not due");
        assert!(muster_due(4, 30, 60, &order), "four: due");
        assert!(!muster_due(1, 30, 30 + MUSTER_PATIENCE, &order), "one alone, ninety seconds in: not yet");
        assert!(muster_due(4, 30, 30 + MUSTER_PATIENCE, &order) || order.release != Some(4), "four is the release here anyway");
        assert!(muster_due(2, 30, 30 + 2 * MUSTER_PATIENCE, &order), "two, three minutes in: due");
    }

    /// The shape: slot 1 left of the lead by a company's width and the gap, slot 2 right, slot 3 behind the lead by
    /// the depth and the gap when three stand abreast; everyone in column when the room allows one.
    #[test]
    fn the_offsets_lay_companies_abreast_then_behind() {
        let east = (1.0, 0.0);
        let (width, depth) = (384.0, 400.0);
        let step = width + COMPANY_GAP;
        let left = Army::offset(1, east, width, depth, 3);
        let right = Army::offset(2, east, width, depth, 3);
        let behind = Army::offset(3, east, width, depth, 3);
        // Heading east, the shape's left is `(-h.1, h.0)`: positive z.
        assert!((left.0).abs() < 0.01 && (left.1 - step).abs() < 0.01, "{left:?}");
        assert!((right.0).abs() < 0.01 && (right.1 + step).abs() < 0.01, "{right:?}");
        assert!((behind.0 + depth + COMPANY_GAP).abs() < 0.01 && behind.1.abs() < 0.01, "{behind:?}");
        let column = Army::offset(2, east, width, depth, 1);
        assert!((column.0 + 2.0 * (depth + COMPANY_GAP)).abs() < 0.01 && column.1.abs() < 0.01, "{column:?}");
        assert_eq!(Army::abreast_for(Some(1000.0), width), 3);
        assert_eq!(Army::abreast_for(Some(300.0), width), 1);
        assert_eq!(Army::abreast_for(None, width), ABREAST);
    }

    /// The floor: two companies under half the cap merge at a halt, never on the march.
    #[test]
    fn two_small_companies_merge_at_a_halt() {
        let a = block(1, 5, 0.0, 0.0);
        let b = block(101, 6, 0.0, 2000.0);
        let mut walkers = a.clone();
        walkers.extend(b.iter().map(|(k, v)| (*k, *v)));
        let order = GroupOrder::new(GroupKind::Hold, vec![at(0.0, 0.0)]).with_cap(24);
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, 0, 64.0, &None);
        assert_eq!(army.companies.len(), 2, "two blocks 2,000 apart: two companies");
        army.ensure_lead(&order);
        for c in &mut army.companies {
            if c.role == Role::Joining {
                c.role = Role::Slaved;
            }
        }
        assert!(!army.merge_floor(24, false), "on the march: no merge");
        assert!(army.merge_floor(24, true), "at a halt: merged");
        army.ensure_lead(&order);
        assert_eq!(army.companies.len(), 1);
        assert_eq!((army.companies[0].role, army.companies[0].members.len()), (Role::Lead, 11));
    }
}
