//! The army of companies (`docs/design/2026-10-25-companies.md` §2): a declared group with a cap is partitioned
//! into companies of at most the cap, each a formation of its own (`group::Group`) with the army's order: its own way
//! from where it stands, its own ground reading, pacing, contact and arrival; nothing of another company moves it.
//! Two companies near each other that fit under the cap together merge. With a cap of 0 everyone is one company: the
//! group as it was before the cap.
//!
//! The cap is there so that no group inherits one movement domain (the user, after companies-1): the Glitters lobby
//! game's mass of 551 had its head reach a choke and its tail of thousands of elmos walk to align to the column. The
//! first build slaved the companies to a lead's anchor at offsets and rebuilt that coupling one level up (companies-1:
//! slaved anchors swung up to 4,229 at the lead's turns); travelling together is a later problem.
//!
//! There is no muster (`docs/design/2026-11-11-groups-as-slots.md` §7, the user's ruling on D1: nothing holds a
//! unit back). Until 2026-11-11 a newcomer far from the army gathered at a plant's point until a company's worth stood
//! there or 90 s had passed. The host now declares only the soldiers with the group (`deal.rs`), and one still coming
//! walks on its own order until it is.

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{UnitId, Vec3};

use crate::form;
use crate::ground::Ground;
use crate::group::{Cause, Group, GroupOrder, GroupState, Walker};

/// A newcomer this near a company's anchor joins it rather than founding one; two companies this near each other
/// merge when they fit under the cap.
pub const JOIN_NEAR: f32 = 800.0;

/// One company's reading, in the army's state.
#[derive(Clone, Debug, PartialEq)]
pub struct CompanyState {
    pub anchor: Vec3,
    pub members: usize,
    pub formed: usize,
    pub exceptions: Vec<(UnitId, Cause, f32)>,
    pub arrived: bool,
}

pub struct Company {
    pub formation: Group,
    pub members: Vec<UnitId>,
    pub state: Option<GroupState>,
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
    /// newcomers are placed: in the nearest company within `JOIN_NEAR` that has room, else in a company founded where
    /// they stand. A company is founded with every pending newcomer near it, so its anchor is set on all of them
    /// (`Group::new`'s dense cluster), not on the first. Returns how many companies were founded.
    pub fn partition(&mut self, name: &str, members: &[UnitId], walkers: &HashMap<UnitId, Walker>, order: &GroupOrder, pitch: form::Pitch, ground: &Option<Arc<Ground>>) -> usize {
        let cap = order.cap;
        if self.cap != Some(cap) {
            self.companies.clear();
            self.cap = Some(cap);
        }
        for c in &mut self.companies {
            c.members.retain(|id| members.contains(id) && walkers.contains_key(id));
        }
        self.companies.retain(|c| !c.members.is_empty());
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
                        self.found(name, all, walkers, order, pitch, ground);
                        founded += 1;
                    }
                }
                continue;
            }
            // (1) A company near enough and under the cap.
            let near = (self.companies.iter().enumerate())
                .filter(|(_, c)| c.members.len() < cap)
                .map(|(i, c)| (i, c.formation.anchor.dist2d(pos)))
                .filter(|(_, d)| *d <= JOIN_NEAR)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i);
            if let Some(i) = near {
                self.companies[i].members.push(id);
                continue;
            }
            // (2) A company of its own where it stands, with the pending newcomers near it.
            let mut all = vec![id];
            all.extend(take_near(&mut pending, pos, cap - 1));
            self.found(name, all, walkers, order, pitch, ground);
            founded += 1;
        }
        founded
    }

    fn found(&mut self, name: &str, members: Vec<UnitId>, walkers: &HashMap<UnitId, Walker>, army: &GroupOrder, pitch: form::Pitch, ground: &Option<Arc<Ground>>) {
        self.next += 1;
        let positions: Vec<Vec3> = members.iter().map(|id| walkers[id].pos).collect();
        let mut formation = Group::new(&format!("{name}.{}", self.next), members.clone(), army.clone(), &positions, pitch);
        if ground.is_some() {
            formation.set_ground(ground.clone());
        }
        // A company founded beside an army already on a route of several points walks on from where the nearest
        // company is, not from the route's first point (slots1-circ-pawn24-feed36-cap24, 2026-11-11: soldiers that
        // came up with the body half way round a circuit founded a company that began the lap again; before the
        // deal a newcomer mustered at the lab, where the route begins).
        if let Some(near) = positions.first().and_then(|at| self.companies.iter().min_by(|a, b| a.formation.anchor.dist2d(*at).total_cmp(&b.formation.anchor.dist2d(*at)))) {
            formation.begin_at(near.formation.route_passed());
        }
        self.companies.push(Company { formation, members, state: None });
    }

    /// The merge (§2): two companies whose anchors stand within `JOIN_NEAR` and whose members fit under the cap
    /// together merge into the larger, once a tick, on the march or at a halt. Returns the pair's sizes.
    pub fn merge(&mut self, cap: usize) -> Option<(usize, usize)> {
        if cap == 0 {
            return None;
        }
        let mut best: Option<(f32, usize, usize)> = None;
        for a in 0..self.companies.len() {
            for b in a + 1..self.companies.len() {
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

    /// The army's reading: the largest company's numbers at the top (the main body), every company's formed and
    /// exceptions together, engaged when any company is, arrived when every company is, passing when any is. None
    /// while no company has been ticked.
    pub fn compose(&self) -> Option<GroupState> {
        let main = self.companies.iter().filter(|c| c.state.is_some()).max_by_key(|c| c.members.len())?;
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
            arrived &= s.arrived;
            companies.push(CompanyState { anchor: s.anchor, members: c.members.len(), formed: s.formed.len(), exceptions: s.exceptions.clone(), arrived: s.arrived });
        }
        // Passing (the restricted-ground design §1.4): the members through of every company in a pass.
        let passing = self.companies.iter().filter_map(|c| c.state.as_ref()?.passing).fold(None, |sum: Option<usize>, through| Some(sum.unwrap_or(0) + through));
        Some(GroupState { formed, exceptions, arrived, engaged, companies, passing, ..top })
    }
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
                (id, Walker { id, pos, speed: 80.0, moving: 0.0 })
            })
            .collect()
    }

    fn ids(walkers: &HashMap<UnitId, Walker>) -> Vec<UnitId> {
        let mut v: Vec<UnitId> = walkers.keys().copied().collect();
        v.sort_by_key(|id| id.0);
        v
    }

    fn sizes(army: &Army) -> Vec<usize> {
        army.companies.iter().map(|c| c.members.len()).collect()
    }

    /// With no cap everyone is one company; with a cap of 24, forty-eight in a block are two companies of 24. A member gone leaves; a newcomer near a company with room joins it; a new cap partitions afresh.
    #[test]
    fn the_partition_founds_companies_of_the_cap() {
        let walkers = block(1, 48, 0.0, 0.0);
        let members = ids(&walkers);
        let go = GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]);
        let mut whole = Army::default();
        whole.partition("a", &members, &walkers, &go, form::HULL_PITCH, &None);
        assert_eq!(sizes(&whole), vec![48]);
        let mut capped = Army::default();
        let order = go.clone().with_cap(24);
        assert_eq!(capped.partition("a", &members, &walkers, &order, form::HULL_PITCH, &None), 2);
        assert_eq!(sizes(&capped), vec![24, 24]);
        let mut fewer = members.clone();
        fewer.retain(|id| id.0 != 3);
        capped.partition("a", &fewer, &walkers, &order, form::HULL_PITCH, &None);
        assert_eq!(capped.companies[0].members.len(), 23);
        let mut more = walkers.clone();
        more.insert(UnitId(99), Walker { id: UnitId(99), pos: at(100.0, 100.0), speed: 80.0, moving: 0.0 });
        let mut all = ids(&more);
        all.retain(|id| id.0 != 3);
        capped.partition("a", &all, &more, &order, form::HULL_PITCH, &None);
        assert_eq!(capped.company_of(UnitId(99)), Some(0), "the company with room: {:?}", sizes(&capped));
        capped.partition("a", &all, &more, &go, form::HULL_PITCH, &None);
        assert_eq!(sizes(&capped), vec![48], "the cap lifted: one company");
    }

    /// A newcomer far from every company founds one where it stands, with the others beside it: the host is to
    /// declare only the soldiers with the group, and what it declares the army places.
    #[test]
    fn a_far_newcomer_founds_a_company_where_it_stands() {
        let mut walkers = block(1, 12, 0.0, 0.0);
        let order = GroupOrder::new(GroupKind::FightTo, vec![at(9000.0, 0.0)]).with_cap(12);
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, form::HULL_PITCH, &None);
        assert_eq!(army.companies.len(), 1);
        for (i, id) in [101, 102, 103].iter().enumerate() {
            walkers.insert(UnitId(*id), Walker { id: UnitId(*id), pos: at(6000.0 + 40.0 * i as f32, 100.0), speed: 80.0, moving: 0.0 });
        }
        assert_eq!(army.partition("a", &ids(&walkers), &walkers, &order, form::HULL_PITCH, &None), 1);
        assert_eq!(sizes(&army), vec![12, 3]);
        assert_eq!(army.companies[1].formation.order.kind, GroupKind::FightTo, "it marches on the army's order");
    }

    /// A company founded beside one that has passed a point of the route begins after that point too.
    #[test]
    fn a_company_founded_on_the_route_begins_where_its_neighbour_is() {
        let route = vec![at(2000.0, 0.0), at(2000.0, 2000.0), at(0.0, 2000.0)];
        let order = GroupOrder::new(GroupKind::Go, route).with_cap(6);
        let mut walkers = block(1, 6, 0.0, 0.0);
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, form::HULL_PITCH, &None);
        army.companies[0].formation.begin_at(1);
        army.companies[0].formation.anchor = at(2000.0, 900.0);
        walkers.insert(UnitId(50), Walker { id: UnitId(50), pos: at(2000.0, 100.0), speed: 80.0, moving: 0.0 });
        assert_eq!(army.partition("a", &ids(&walkers), &walkers, &order, form::HULL_PITCH, &None), 1);
        assert_eq!(army.companies[1].formation.route_passed(), 1);
    }

    /// The merge: two companies near each other that fit under the cap merge into the larger, on the march;
    /// two that would overflow it, or that stand far apart, stay apart.
    #[test]
    fn near_companies_that_fit_merge_on_the_march() {
        let order = GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]).with_cap(24);
        let mut walkers = block(1, 10, 0.0, 0.0);
        walkers.extend(block(101, 6, 0.0, 2000.0));
        walkers.extend(block(201, 20, 6000.0, 6000.0));
        let mut army = Army::default();
        army.partition("a", &ids(&walkers), &walkers, &order, form::HULL_PITCH, &None);
        assert_eq!(army.companies.len(), 3, "{:?}", sizes(&army));
        assert_eq!(army.merge(24), None, "all far apart");
        // The six walk up beside the ten.
        let six = army.companies.iter().position(|c| c.members.len() == 6).unwrap();
        army.companies[six].formation.anchor = at(300.0, 0.0);
        assert_eq!(army.merge(24), Some((10, 6)));
        assert_eq!(sizes(&army), vec![16, 20]);
        // Sixteen and twenty would overflow 24: no merge however near.
        army.companies[1].formation.anchor = at(200.0, 0.0);
        assert_eq!(army.merge(24), None);
    }
}
