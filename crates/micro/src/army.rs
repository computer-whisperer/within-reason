//! The army of companies (`docs/design/2026-10-25-companies.md` §2): a declared group with a cap is partitioned
//! into companies of at most the cap, each a formation of its own (`group::Group`). A **marching** company is a whole
//! group with the army's order: its own way from where it stands, its own ground reading, pacing, contact and
//! arrival; nothing of another company moves it. A **mustering** company holds beside a muster point (a plant) until
//! it has `release` members or `MUSTER_PATIENCE` has passed since its first, then marches. Two marching companies near
//! each other that fit under the cap together merge. With a cap of 0 everyone is one company: the group as it was
//! before the cap.
//!
//! The cap is there so that no group inherits one movement domain (the user, after companies-1): the Glitters lobby
//! game's mass of 551 had its head reach a choke and its tail of thousands of elmos walk to align to the column. The
//! first build slaved the companies to a lead's anchor at offsets and rebuilt that coupling one level up (companies-1:
//! slaved anchors swung up to 4,229 at the lead's turns); travelling together is a later problem.

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{UnitId, Vec3};

use crate::ground::Ground;
use crate::group::{Cause, Group, GroupKind, GroupOrder, GroupState, Walker};

/// A newcomer this near a company's anchor (or a muster point) joins it rather than founding one; two marching
/// companies this near each other merge when they fit under the cap.
pub const JOIN_NEAR: f32 = 800.0;
/// A muster of `MUSTER_MIN` or more releases this long after its first member came, full or not; one of fewer at
/// twice it (a trickle goes in fours, not never, and a lone straggler not for three minutes). The fed circuit at a
/// cap of 12: the lab made 11 in two minutes, so a muster full at the cap would have waited the whole lap.
pub const MUSTER_PATIENCE: i32 = 90 * 30;
pub const MUSTER_MIN: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Marching,
    Mustering,
}

impl Role {
    pub fn word(self) -> &'static str {
        match self {
            Role::Marching => "marching",
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
    /// A muster's point; none for a marching company.
    pub at: Option<Vec3>,
}

pub struct Company {
    pub formation: Group,
    pub members: Vec<UnitId>,
    /// The muster point and the frame its first member came; none once it marches.
    pub muster: Option<(Vec3, i32)>,
    pub state: Option<GroupState>,
}

impl Company {
    pub fn role(&self) -> Role {
        if self.muster.is_some() { Role::Mustering } else { Role::Marching }
    }

    /// The order its formation walks this tick: the army's, or a hold at its muster.
    pub fn order(&self, army: &GroupOrder) -> GroupOrder {
        own_order(self.muster, army)
    }
}

fn own_order(muster: Option<(Vec3, i32)>, army: &GroupOrder) -> GroupOrder {
    match muster {
        Some((at, _)) => GroupOrder::new(GroupKind::Hold, vec![at]),
        None => army.clone(),
    }
}

#[derive(Default)]
pub struct Army {
    pub companies: Vec<Company>,
    next: usize,
    /// The cap the companies were made under; a new one partitions every member afresh.
    cap: Option<usize>,
}

impl Army {
    /// Which company has this member.
    pub fn company_of(&self, id: UnitId) -> Option<usize> {
        self.companies.iter().position(|c| c.members.contains(&id))
    }

    /// The membership this tick (§2, the partition): a new cap dissolves the companies, members gone leave theirs,
    /// newcomers are placed. The muster is for newcomers to an army that marches: the declaration's members (the
    /// first partition, or the first after a new cap) found marching companies wherever they stand (the circuit drill
    /// at a cap of 24: the sixty spawned by the lab mustered there, and the last twelve waited the muster's patience). A company is founded with every pending newcomer near it, so its anchor is set on all of
    /// them (`Group::new`'s dense cluster), not on the first. Returns how many companies were founded.
    #[allow(clippy::too_many_arguments)]
    pub fn partition(&mut self, name: &str, members: &[UnitId], walkers: &HashMap<UnitId, Walker>, order: &GroupOrder, frame: i32, spacing: f32, ground: &Option<Arc<Ground>>) -> usize {
        let cap = order.cap;
        if self.cap != Some(cap) {
            self.companies.clear();
            self.cap = Some(cap);
        }
        for c in &mut self.companies {
            c.members.retain(|id| members.contains(id) && walkers.contains_key(id));
        }
        self.companies.retain(|c| !c.members.is_empty());
        let musters: &[Vec3] = if self.companies.is_empty() { &[] } else { &order.musters };
        let mut founded = 0;
        let mut pending: Vec<UnitId> = members.iter().copied().filter(|id| walkers.contains_key(id) && self.company_of(*id).is_none()).collect();
        // The companions a founding takes from the pending: those within `JOIN_NEAR` of `about`, up to `room`.
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
                match self.companies.first_mut() {
                    Some(c) => c.members.push(id),
                    None => {
                        let mut all = vec![id];
                        all.append(&mut pending);
                        self.found(name, all, walkers, order, None, spacing, ground);
                        founded += 1;
                    }
                }
                continue;
            }
            // (1) A marching company near enough and under the cap.
            let near = (self.companies.iter().enumerate())
                .filter(|(_, c)| c.role() == Role::Marching && c.members.len() < cap)
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
            let muster = musters.iter().copied().map(|m| (m, m.dist2d(pos))).filter(|(_, d)| *d <= JOIN_NEAR).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(m, _)| m);
            if let Some(m) = muster {
                match self.companies.iter().position(|c| c.muster.is_some_and(|(at, _)| at.dist2d(m) < 1.0) && c.members.len() < cap) {
                    Some(i) => self.companies[i].members.push(id),
                    None => {
                        let mut all = vec![id];
                        all.extend(take_near(&mut pending, m, cap - 1));
                        self.found(name, all, walkers, order, Some((m, frame)), spacing, ground);
                        founded += 1;
                    }
                }
                continue;
            }
            // (3) A marching company of its own where it stands, with the pending newcomers near it.
            let mut all = vec![id];
            all.extend(take_near(&mut pending, pos, cap - 1));
            self.found(name, all, walkers, order, None, spacing, ground);
            founded += 1;
        }
        founded
    }

    #[allow(clippy::too_many_arguments)]
    fn found(&mut self, name: &str, members: Vec<UnitId>, walkers: &HashMap<UnitId, Walker>, army: &GroupOrder, muster: Option<(Vec3, i32)>, spacing: f32, ground: &Option<Arc<Ground>>) {
        self.next += 1;
        let positions: Vec<Vec3> = members.iter().map(|id| walkers[id].pos).collect();
        let mut formation = Group::new(&format!("{name}.{}", self.next), members.clone(), own_order(muster, army), &positions, spacing);
        if ground.is_some() {
            formation.set_ground(ground.clone());
        }
        self.companies.push(Company { formation, members, muster, state: None });
    }

    /// The merge (§2): two marching companies whose anchors stand within `JOIN_NEAR` and whose members fit under the
    /// cap together merge into the larger, once a tick, on the march or at a halt. Returns the pair's sizes.
    pub fn merge(&mut self, cap: usize) -> Option<(usize, usize)> {
        if cap == 0 {
            return None;
        }
        let marching: Vec<usize> = (0..self.companies.len()).filter(|&i| self.companies[i].role() == Role::Marching).collect();
        let mut best: Option<(f32, usize, usize)> = None;
        for (n, &a) in marching.iter().enumerate() {
            for &b in &marching[n + 1..] {
                let (ca, cb) = (&self.companies[a], &self.companies[b]);
                let d = ca.formation.anchor.dist2d(cb.formation.anchor);
                if d <= JOIN_NEAR && ca.members.len() + cb.members.len() <= cap && best.is_none_or(|(x, ..)| d < x) {
                    best = Some((d, a, b));
                }
            }
        }
        let (_, mut a, mut b) = best?;
        if self.companies[a].members.len() < self.companies[b].members.len() {
            std::mem::swap(&mut a, &mut b);
        }
        // `a` is the larger: it takes `b`'s members.
        let gone = self.companies.remove(b);
        let a = if b < a { a - 1 } else { a };
        let sizes = (self.companies[a].members.len(), gone.members.len());
        self.companies[a].members.extend(gone.members);
        Some(sizes)
    }

    /// The army's reading: the largest marching company's numbers at the top (the main body), every company's formed
    /// and exceptions together, engaged when any company is, arrived when every marching company is (a muster is not
    /// of the march yet). None while nobody marches.
    pub fn compose(&self) -> Option<GroupState> {
        let main = (self.companies.iter()).filter(|c| c.role() == Role::Marching && c.state.is_some()).max_by_key(|c| c.members.len())?;
        let top = main.state.clone()?;
        let mut formed = Vec::new();
        let mut exceptions = Vec::new();
        let mut companies = Vec::new();
        let mut arrived = true;
        let mut engaged = top.engaged;
        for c in &self.companies {
            let Some(s) = &c.state else { continue };
            formed.extend(s.formed.iter().copied());
            exceptions.extend(s.exceptions.iter().cloned());
            engaged = engaged.or(s.engaged);
            if c.role() == Role::Marching {
                arrived &= s.arrived;
            }
            companies.push(CompanyState { role: c.role(), anchor: s.anchor, members: c.members.len(), formed: s.formed.len(), exceptions: s.exceptions.clone(), arrived: s.arrived, at: c.muster.map(|(at, _)| at) });
        }
        Some(GroupState { formed, exceptions, arrived, engaged, companies, ..top })
    }
}

/// Whether a muster is due to release: `release` members stand (the cap when none was given), or its first member
/// has waited `MUSTER_PATIENCE` with `MUSTER_MIN` standing, or twice it.
pub fn muster_due(members: usize, since: i32, frame: i32, order: &GroupOrder) -> bool {
    let wanted = order.release.unwrap_or(order.cap).max(1);
    let waited = frame - since;
    members >= wanted || (members >= MUSTER_MIN && waited >= MUSTER_PATIENCE) || waited >= 2 * MUSTER_PATIENCE
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn sizes(army: &Army) -> Vec<(Role, usize)> {
        army.companies.iter().map(|c| (c.role(), c.members.len())).collect()
    }

    /// With no cap everyone is one company; with a cap of 24, forty-eight in a block are two marching companies of
    /// 24. A member gone leaves; a newcomer near a company with room joins it; a new cap partitions afresh.
    #[test]
    fn the_partition_founds_companies_of_the_cap() {
        let walkers = block(1, 48, 0.0, 0.0);
        let members = ids(&walkers);
        let go = GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]);
        let mut whole = Army::default();
        whole.partition("a", &members, &walkers, &go, 0, 64.0, &None);
        assert_eq!(sizes(&whole), vec![(Role::Marching, 48)]);
        let mut capped = Army::default();
        let order = go.clone().with_cap(24);
        assert_eq!(capped.partition("a", &members, &walkers, &order, 0, 64.0, &None), 2);
        assert_eq!(sizes(&capped), vec![(Role::Marching, 24), (Role::Marching, 24)]);
        let mut fewer = members.clone();
        fewer.retain(|id| id.0 != 3);
        capped.partition("a", &fewer, &walkers, &order, 3, 64.0, &None);
        assert_eq!(capped.companies[0].members.len(), 23);
        let mut more = walkers.clone();
        more.insert(UnitId(99), Walker { id: UnitId(99), pos: at(100.0, 100.0), speed: 80.0, moving: 0.0, fleeing: false });
        let mut all = ids(&more);
        all.retain(|id| id.0 != 3);
        capped.partition("a", &all, &more, &order, 6, 64.0, &None);
        assert_eq!(capped.company_of(UnitId(99)), Some(0), "the company with room: {:?}", sizes(&capped));
        capped.partition("a", &all, &more, &go, 9, 64.0, &None);
        assert_eq!(sizes(&capped), vec![(Role::Marching, 48)], "the cap lifted: one company");
    }

    /// A newcomer far from the army, near a muster point, musters there; the muster releases when it holds the
    /// release count, or after `MUSTER_PATIENCE` with four standing, or twice it.
    #[test]
    fn a_far_newcomer_musters_and_the_muster_releases_full_or_late() {
        let mut walkers = block(1, 12, 0.0, 0.0);
        let muster = at(6000.0, 0.0);
        let mut order = GroupOrder::new(GroupKind::FightTo, vec![at(9000.0, 0.0)]).with_cap(12);
        order.musters = vec![muster];
        order.release = Some(6);
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, 0, 64.0, &None);
        assert_eq!(army.companies.len(), 1);
        for (i, id) in [101, 102, 103].iter().enumerate() {
            walkers.insert(UnitId(*id), Walker { id: UnitId(*id), pos: at(6000.0 + 40.0 * i as f32, 100.0), speed: 80.0, moving: 0.0, fleeing: false });
        }
        army.partition("a", &ids(&walkers), &walkers, &order, 30, 64.0, &None);
        assert_eq!(sizes(&army), vec![(Role::Marching, 12), (Role::Mustering, 3)]);
        let m = &army.companies[1];
        assert_eq!(m.order(&order).kind, GroupKind::Hold);
        assert!(!muster_due(3, 30, 60, &order), "three of six, a second in: not due");
        assert!(muster_due(6, 30, 60, &order), "six: due");
        assert!(!muster_due(3, 30, 30 + MUSTER_PATIENCE, &order), "three, ninety seconds in: not yet");
        assert!(muster_due(4, 30, 30 + MUSTER_PATIENCE, &order), "four, ninety seconds in: due");
        assert!(muster_due(1, 30, 30 + 2 * MUSTER_PATIENCE, &order), "one, three minutes in: due");
        // The declaration's members beside the muster march: the muster is for newcomers.
        let mut fresh = Army::default();
        let by_plant = block(301, 30, 6000.0, 0.0);
        fresh.partition("b", &ids(&by_plant), &by_plant, &order, 0, 64.0, &None);
        assert_eq!(sizes(&fresh), vec![(Role::Marching, 12), (Role::Marching, 12), (Role::Marching, 6)]);
    }

    /// The merge: two marching companies near each other that fit under the cap merge into the larger, on the march;
    /// two that would overflow it, or that stand far apart, stay apart.
    #[test]
    fn near_companies_that_fit_merge_on_the_march() {
        let order = GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]).with_cap(24);
        let mut walkers = block(1, 10, 0.0, 0.0);
        walkers.extend(block(101, 6, 0.0, 2000.0));
        walkers.extend(block(201, 20, 6000.0, 6000.0));
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, 0, 64.0, &None);
        assert_eq!(army.companies.len(), 3, "{:?}", sizes(&army));
        assert_eq!(army.merge(24), None, "all far apart");
        // The six walk up beside the ten.
        let six = army.companies.iter().position(|c| c.members.len() == 6).unwrap();
        army.companies[six].formation.anchor = at(300.0, 0.0);
        assert_eq!(army.merge(24), Some((10, 6)));
        assert_eq!(sizes(&army), vec![(Role::Marching, 16), (Role::Marching, 20)]);
        // Sixteen and twenty would overflow 24: no merge however near.
        army.companies[1].formation.anchor = at(200.0, 0.0);
        assert_eq!(army.merge(24), None);
    }
}
