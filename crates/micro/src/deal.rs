//! The deal (`docs/design/2026-11-11-groups-as-slots.md` §3): a group is slots at a place, and once a second every
//! soldier of the seat is dealt to one from its position and type alone. Nothing of a soldier's past enters but the
//! slot it held a second ago, which damps the deal (the user, 2026-11-10: "almost no hidden state to our unit
//! control"; "Two units with the same position, velocity, etc that are not fully interchangable is a gauge orbit we
//! can take advantage of removing"). The order of the deal:
//!
//! 1. **Answers.** A soldier lent to an answer that still lives stays lent. A lend is an event (`Deal::lend`): the
//!    lender's soonest to the party at that moment. Its slot in the lender is open from then on, and when the answer
//!    ends (`Deal::release`) it is dealt like any other soldier: where it came from is nothing to the deal.
//! 2. **In place.** A soldier inside a group's gate takes a slot of it while one of its type is open: `gate` seconds
//!    of its own travel to the group's place to enter, `leave` to stay, so that a body some companies across does not
//!    shed its edges. A group with more soldiers in its gate than slots keeps those with it a second ago, then the
//!    nearest; the ones let go are those best placed for another group.
//! 3. **Rove slots.** A roving group short of strength keeps its rovers and takes the soldiers of its types nearest
//!    them (or nearest its region) that nothing above placed. A rover has no place to come to: it roves from where it
//!    stands.
//! 4. **The rest by time.** Every other soldier takes the open slot it reaches soonest, the pairs in order of time. A
//!    soldier already coming to a slot is dealt elsewhere only when that saves `margin` seconds.
//! 5. **Surplus.** A soldier no slot is open for holds none.
//!
//! The host says the groups (the player's slots, each with its place) and how long a soldier needs to a point, and
//! reads who is with each group (the body it hands to the lane), who is coming and when, who is lent and who is
//! surplus. Code decides which soldier fills which slot and nothing else: the slots are the player's, an answer's
//! strength is a pick's.

use std::collections::{BTreeMap, HashMap};

use bot_protocol::{UnitDefId, UnitId, Vec3};

/// Seconds of a soldier's own travel to a group's place within which it is with the group.
pub const GATE_SECONDS: f32 = 15.0;
/// Seconds within which a soldier with a group stays with it.
pub const GATE_LEAVE_SECONDS: f32 = 30.0;
/// A soldier coming to a slot is dealt to another only when that saves this many seconds.
pub const DEAL_MARGIN_SECONDS: f32 = 5.0;

/// The deal's constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gates {
    pub gate: f32,
    pub leave: f32,
    pub margin: f32,
}

impl Default for Gates {
    fn default() -> Self {
        Gates { gate: GATE_SECONDS, leave: GATE_LEAVE_SECONDS, margin: DEAL_MARGIN_SECONDS }
    }
}

/// A soldier as the deal sees it: where it is, what it is, how fast it goes.
#[derive(Clone, Copy, Debug)]
pub struct Soldier {
    pub id: UnitId,
    pub def: UnitDefId,
    pub pos: Vec3,
    /// Elmos a second.
    pub speed: f32,
}

/// A group's wanted strength: slots of a named type, and slots any type fills.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Slots {
    pub any: usize,
    pub typed: Vec<(UnitDefId, usize)>,
}

impl Slots {
    pub fn any(count: usize) -> Slots {
        Slots { any: count, typed: Vec::new() }
    }

    pub fn total(&self) -> usize {
        self.any + self.typed.iter().map(|(_, n)| n).sum::<usize>()
    }
}

/// Where a group's slots are: a point (the hold's, or the centre of the body on the move), or nowhere for a roving
/// group, whose newcomers are taken nearest `about` (its region's centre) when one is given.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    At(Vec3),
    Roving { about: Option<Vec3> },
}

/// One group's demand.
#[derive(Clone, Debug)]
pub struct Demand {
    pub name: String,
    pub slots: Slots,
    pub place: Place,
}

/// What the deal gave a soldier.
#[derive(Clone, Debug, PartialEq)]
pub enum Standing {
    /// In a slot of this group and inside its gate (or roving for it): of its body.
    With(String),
    /// In a slot of this group and this many seconds from its place.
    Coming(String, f32),
    /// Lent to this answer.
    Lent(String),
    /// No slot is open for it.
    Surplus,
}

impl Standing {
    /// The group whose slot it holds.
    pub fn group(&self) -> Option<&str> {
        match self {
            Standing::With(g) | Standing::Coming(g, _) => Some(g),
            _ => None,
        }
    }
}

/// The open slots of one group during a deal.
struct Open {
    any: usize,
    typed: HashMap<UnitDefId, usize>,
}

impl Open {
    fn of(slots: &Slots) -> Open {
        Open { any: slots.any, typed: slots.typed.iter().copied().collect() }
    }

    fn has(&self, def: UnitDefId) -> bool {
        self.any > 0 || self.typed.get(&def).is_some_and(|n| *n > 0)
    }

    /// Takes a slot for this type: one named for it first.
    fn take(&mut self, def: UnitDefId) -> bool {
        if let Some(n) = self.typed.get_mut(&def).filter(|n| **n > 0) {
            *n -= 1;
            return true;
        }
        if self.any > 0 {
            self.any -= 1;
            return true;
        }
        false
    }

    fn left(&self) -> usize {
        self.any + self.typed.values().sum::<usize>()
    }
}

/// The deal's one memory: what each soldier held a second ago.
#[derive(Default)]
pub struct Deal {
    held: HashMap<UnitId, Standing>,
}

impl Deal {
    /// What this soldier held at the last deal.
    pub fn held(&self, id: UnitId) -> Option<&Standing> {
        self.held.get(&id)
    }

    /// Lends `count` soldiers of `lender` to `answer`: those holding a slot of the lender (with it or coming to it)
    /// that reach `at` soonest. Returns who went, soonest first. Their slots are open from the next deal.
    pub fn lend(&mut self, answer: &str, lender: &str, count: usize, at: Vec3, soldiers: &[Soldier], travel: &dyn Fn(&Soldier, Vec3) -> f32) -> Vec<UnitId> {
        let mut theirs: Vec<(f32, UnitId)> = soldiers.iter().filter(|s| self.held.get(&s.id).and_then(Standing::group) == Some(lender)).map(|s| (travel(s, at), s.id)).collect();
        theirs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.0.cmp(&b.1.0)));
        theirs.truncate(count);
        for (_, id) in &theirs {
            self.held.insert(*id, Standing::Lent(answer.to_string()));
        }
        theirs.into_iter().map(|(_, id)| id).collect()
    }

    /// Ends an answer: its soldiers hold nothing, and the next deal places them like any others.
    pub fn release(&mut self, answer: &str) {
        self.held.retain(|_, s| !matches!(s, Standing::Lent(a) if a == answer));
    }

    /// One soldier leaves its answer (a hunter withdrawn alone).
    pub fn release_one(&mut self, id: UnitId) {
        if matches!(self.held.get(&id), Some(Standing::Lent(_))) {
            self.held.remove(&id);
        }
    }

    /// The deal over these soldiers and groups. `answers` are the names of the answers that live; `travel` is the
    /// seconds a soldier needs to a point.
    pub fn deal(&mut self, soldiers: &[Soldier], demands: &[Demand], answers: &[&str], travel: &dyn Fn(&Soldier, Vec3) -> f32, gates: Gates) -> HashMap<UnitId, Standing> {
        let mut out: HashMap<UnitId, Standing> = HashMap::new();
        let mut open: Vec<Open> = demands.iter().map(|d| Open::of(&d.slots)).collect();
        let held = |id: UnitId| self.held.get(&id);
        // Seconds from each soldier to each placed group.
        let seconds: Vec<Vec<f32>> = soldiers
            .iter()
            .map(|s| {
                demands
                    .iter()
                    .map(|d| match d.place {
                        Place::At(at) => travel(s, at),
                        Place::Roving { .. } => f32::INFINITY,
                    })
                    .collect()
            })
            .collect();
        // 1. Answers.
        for s in soldiers {
            if let Some(Standing::Lent(a)) = held(s.id)
                && answers.contains(&a.as_str())
            {
                out.insert(s.id, Standing::Lent(a.clone()));
            }
        }
        // 2. In place: per group, those inside its gate, the ones with it a second ago first, then the nearest.
        let mut inside: Vec<Vec<(bool, f32, usize)>> = vec![Vec::new(); demands.len()];
        for (si, s) in soldiers.iter().enumerate() {
            if out.contains_key(&s.id) {
                continue;
            }
            // Its own group while it is inside the leaving gate, else the nearest gate it is inside.
            let own = demands.iter().position(|d| matches!(held(s.id), Some(Standing::With(g)) if *g == d.name)).filter(|di| seconds[si][*di] <= gates.leave);
            let near = (0..demands.len()).filter(|di| seconds[si][*di] <= gates.gate).min_by(|a, b| seconds[si][*a].total_cmp(&seconds[si][*b]));
            if let Some(di) = own.or(near) {
                inside[di].push((own == Some(di), seconds[si][di], si));
            }
        }
        for (di, mut mine) in inside.into_iter().enumerate() {
            let room = open[di].left();
            if mine.len() > room {
                // More inside than slots: let go those best placed for another group (the least seconds to another
                // group's place), among them the newcomers before the ones already with it.
                let elsewhere = |si: usize| (0..demands.len()).filter(|other| *other != di).map(|other| seconds[si][other]).fold(f32::INFINITY, f32::min);
                mine.sort_by(|a, b| b.0.cmp(&a.0).then(elsewhere(b.2).total_cmp(&elsewhere(a.2))).then(a.1.total_cmp(&b.1)));
            } else {
                mine.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.total_cmp(&b.1)));
            }
            for (_, _, si) in mine {
                let s = &soldiers[si];
                if open[di].take(s.def) {
                    out.insert(s.id, Standing::With(demands[di].name.clone()));
                }
            }
        }
        // 3. Rove slots: its rovers stay, then the nearest of the rest.
        for (di, d) in demands.iter().enumerate() {
            let Place::Roving { about } = d.place else { continue };
            let mut mine: Vec<&Soldier> = soldiers.iter().filter(|s| !out.contains_key(&s.id) && matches!(held(s.id), Some(Standing::With(g)) if *g == d.name)).collect();
            mine.sort_by_key(|s| s.id.0);
            let mut centre = about;
            if centre.is_none() && !mine.is_empty() {
                let n = mine.len() as f32;
                centre = Some(Vec3 { x: mine.iter().map(|s| s.pos.x).sum::<f32>() / n, y: 0.0, z: mine.iter().map(|s| s.pos.z).sum::<f32>() / n });
            }
            for s in mine {
                if open[di].take(s.def) {
                    out.insert(s.id, Standing::With(d.name.clone()));
                }
            }
            if open[di].left() == 0 {
                continue;
            }
            let mut rest: Vec<(f32, &Soldier)> = soldiers.iter().filter(|s| !out.contains_key(&s.id) && open[di].has(s.def)).map(|s| (centre.map_or(0.0, |c| travel(s, c)), s)).collect();
            rest.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.id.0.cmp(&b.1.id.0)));
            for (_, s) in rest {
                if open[di].take(s.def) {
                    out.insert(s.id, Standing::With(d.name.clone()));
                }
            }
        }
        // 4. The rest by time, a soldier's own slot of a second ago `margin` seconds nearer than it is.
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for (si, s) in soldiers.iter().enumerate() {
            if out.contains_key(&s.id) {
                continue;
            }
            let own = held(s.id).and_then(Standing::group);
            for (di, d) in demands.iter().enumerate() {
                if seconds[si][di].is_finite() && open[di].has(s.def) {
                    let bias = if own == Some(d.name.as_str()) { gates.margin } else { 0.0 };
                    pairs.push((seconds[si][di] - bias, si, di));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(soldiers[a.1].id.0.cmp(&soldiers[b.1].id.0)).then(a.2.cmp(&b.2)));
        for (_, si, di) in pairs {
            let s = &soldiers[si];
            if out.contains_key(&s.id) {
                continue;
            }
            if open[di].take(s.def) {
                let t = seconds[si][di];
                // Inside the gate with no slot in step 2 only when the slot opened since: it is with the group.
                out.insert(s.id, if t <= gates.gate { Standing::With(demands[di].name.clone()) } else { Standing::Coming(demands[di].name.clone(), t) });
            }
        }
        // 5. Surplus.
        for s in soldiers {
            out.entry(s.id).or_insert(Standing::Surplus);
        }
        self.held = out.clone();
        out
    }
}

/// A group's reading from a deal: who is with it, who is coming and in how many seconds, soonest first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fill {
    pub with: Vec<UnitId>,
    pub coming: Vec<(UnitId, f32)>,
}

/// Every group's fill from a deal's result, by group name; the surplus under the empty name.
pub fn fills(dealt: &HashMap<UnitId, Standing>) -> BTreeMap<String, Fill> {
    let mut out: BTreeMap<String, Fill> = BTreeMap::new();
    for (id, standing) in dealt {
        match standing {
            Standing::With(g) => out.entry(g.clone()).or_default().with.push(*id),
            Standing::Coming(g, t) => out.entry(g.clone()).or_default().coming.push((*id, *t)),
            Standing::Surplus => out.entry(String::new()).or_default().with.push(*id),
            Standing::Lent(_) => {}
        }
    }
    for fill in out.values_mut() {
        fill.with.sort_by_key(|id| id.0);
        fill.coming.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.0.cmp(&b.0.0)));
    }
    out
}

/// The straight line over the soldier's speed: the travel time where the host holds no field.
pub fn straight(s: &Soldier, to: Vec3) -> f32 {
    s.pos.dist2d(to) / s.speed.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAWN: UnitDefId = UnitDefId(1);
    const BULL: UnitDefId = UnitDefId(2);

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A Pawn-like soldier at 100 a second, so 100 elmos are a second.
    fn pawn(id: i32, x: f32, z: f32) -> Soldier {
        Soldier { id: UnitId(id), def: PAWN, pos: at(x, z), speed: 100.0 }
    }

    fn bull(id: i32, x: f32, z: f32) -> Soldier {
        Soldier { id: UnitId(id), def: BULL, pos: at(x, z), speed: 50.0 }
    }

    fn group(name: &str, slots: usize, x: f32, z: f32) -> Demand {
        Demand { name: name.to_string(), slots: Slots::any(slots), place: Place::At(at(x, z)) }
    }

    fn with(g: &str) -> Standing {
        Standing::With(g.to_string())
    }

    #[test]
    fn a_soldier_is_with_the_group_inside_its_gate_and_coming_outside_it() {
        let mut deal = Deal::default();
        let soldiers = [pawn(1, 0.0, 0.0), pawn(2, 1400.0, 0.0), pawn(3, 4000.0, 0.0), bull(4, 1000.0, 0.0)];
        let dealt = deal.deal(&soldiers, &[group("A", 8, 0.0, 0.0)], &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)], with("A"));
        assert_eq!(dealt[&UnitId(2)], with("A"), "14 s away at its own speed");
        assert_eq!(dealt[&UnitId(3)], Standing::Coming("A".into(), 40.0));
        assert_eq!(dealt[&UnitId(4)], Standing::Coming("A".into(), 20.0), "the same 1,000 is 20 s for a Bull");
        let fill = &fills(&dealt)["A"];
        assert_eq!(fill.with, [UnitId(1), UnitId(2)]);
        assert_eq!(fill.coming, [(UnitId(4), 20.0), (UnitId(3), 40.0)]);
    }

    #[test]
    fn two_soldiers_in_one_place_are_dealt_alike_whatever_they_held() {
        // The gauge: a soldier that was coming to B and one that held nothing, standing together beside A with A's
        // slots open, are both A's. B is 60 s off, so the margin does not hold the first.
        let demands = [group("A", 4, 0.0, 0.0), group("B", 4, 6000.0, 0.0)];
        let mut deal = Deal::default();
        deal.held.insert(UnitId(1), Standing::Coming("B".into(), 60.0));
        let dealt = deal.deal(&[pawn(1, 100.0, 0.0), pawn(2, 100.0, 10.0)], &demands, &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)], with("A"));
        assert_eq!(dealt[&UnitId(2)], with("A"));
    }

    #[test]
    fn a_full_group_sends_the_rest_to_the_next_and_the_last_are_surplus() {
        let demands = [group("A", 2, 0.0, 0.0), group("B", 1, 3000.0, 0.0)];
        let soldiers = [pawn(1, 0.0, 0.0), pawn(2, 50.0, 0.0), pawn(3, 100.0, 0.0), pawn(4, 150.0, 0.0)];
        let mut deal = Deal::default();
        let dealt = deal.deal(&soldiers, &demands, &[], &straight, Gates::default());
        // Of four beside A, the two best placed for B (the nearest to it) are let go; one takes B's slot.
        assert_eq!(dealt[&UnitId(1)], with("A"));
        assert_eq!(dealt[&UnitId(2)], with("A"));
        assert_eq!(dealt[&UnitId(4)], Standing::Coming("B".into(), 28.5));
        assert_eq!(dealt[&UnitId(3)], Standing::Surplus);
    }

    #[test]
    fn a_group_resized_down_lets_go_those_best_placed_for_the_group_that_grew() {
        // Six with A; A goes to four and a station of two is declared east: the two easternmost go.
        let soldiers: Vec<Soldier> = (0..6).map(|i| pawn(i, i as f32 * 100.0, 0.0)).collect();
        let mut deal = Deal::default();
        deal.deal(&soldiers, &[group("A", 6, 250.0, 0.0)], &[], &straight, Gates::default());
        let dealt = deal.deal(&soldiers, &[group("A", 4, 250.0, 0.0), group("station", 2, 3000.0, 0.0)], &[], &straight, Gates::default());
        let fill = fills(&dealt);
        assert_eq!(fill["A"].with, [UnitId(0), UnitId(1), UnitId(2), UnitId(3)]);
        assert_eq!(fill["station"].coming.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [UnitId(5), UnitId(4)]);
    }

    #[test]
    fn a_soldier_with_a_group_stays_to_the_leaving_gate() {
        let demands = [group("A", 4, 0.0, 0.0)];
        let mut deal = Deal::default();
        deal.deal(&[pawn(1, 1000.0, 0.0)], &demands, &[], &straight, Gates::default());
        // 25 s out: past the gate, inside the leaving gate.
        let dealt = deal.deal(&[pawn(1, 2500.0, 0.0)], &demands, &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)], with("A"));
        let dealt = deal.deal(&[pawn(1, 3100.0, 0.0)], &demands, &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)], Standing::Coming("A".into(), 31.0));
        // And one that was never with it is coming at the same 25 s.
        let mut fresh = Deal::default();
        assert_eq!(fresh.deal(&[pawn(2, 2500.0, 0.0)], &demands, &[], &straight, Gates::default())[&UnitId(2)], Standing::Coming("A".into(), 25.0));
    }

    #[test]
    fn a_soldier_coming_to_a_slot_is_dealt_elsewhere_only_for_the_margin() {
        // Coming to A, 30 s out; B opens 27 s away: it keeps A. B 24 s away: it goes to B.
        let mut deal = Deal::default();
        let soldier = [pawn(1, 3000.0, 0.0)];
        deal.deal(&soldier, &[group("A", 1, 0.0, 0.0)], &[], &straight, Gates::default());
        let dealt = deal.deal(&soldier, &[group("A", 1, 0.0, 0.0), group("B", 1, 5700.0, 0.0)], &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)].group(), Some("A"));
        let dealt = deal.deal(&soldier, &[group("A", 1, 0.0, 0.0), group("B", 1, 5400.0, 0.0)], &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(1)].group(), Some("B"));
    }

    #[test]
    fn typed_slots_take_their_type_and_the_rest_take_any() {
        let demand = Demand { name: "A".into(), slots: Slots { any: 1, typed: vec![(BULL, 1)] }, place: Place::At(at(0.0, 0.0)) };
        let soldiers = [pawn(1, 0.0, 0.0), pawn(2, 10.0, 0.0), bull(3, 20.0, 0.0), bull(4, 30.0, 0.0)];
        let dealt = Deal::default().deal(&soldiers, &[demand], &[], &straight, Gates::default());
        // The Bull's slot goes to a Bull; the one open slot to the nearest of the rest.
        assert_eq!(dealt[&UnitId(3)], with("A"));
        assert_eq!(dealt[&UnitId(1)], with("A"));
        assert_eq!(dealt[&UnitId(2)], Standing::Surplus);
        assert_eq!(dealt[&UnitId(4)], Standing::Surplus);
    }

    #[test]
    fn a_lend_takes_the_lenders_soonest_and_the_end_deals_them_like_any_other() {
        let demands = [group("A", 4, 0.0, 0.0), group("station", 2, 2000.0, 0.0)];
        // Four with A; a fresh one walking to A stands beside the party at (1900, 0).
        let soldiers = [pawn(1, 0.0, 0.0), pawn(2, 100.0, 0.0), pawn(3, 200.0, 0.0), pawn(5, 1800.0, 600.0)];
        let mut deal = Deal::default();
        let dealt = deal.deal(&soldiers, &demands, &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(5)], with("station"), "the nearer gate");
        assert_eq!(fills(&dealt)["A"].with.len(), 3);
        // A lends two against a party at (900, 0): its two nearest that.
        let went = deal.lend("party_12", "A", 2, at(900.0, 0.0), &soldiers, &straight);
        assert_eq!(went, [UnitId(3), UnitId(2)]);
        let dealt = deal.deal(&soldiers, &demands, &["party_12"], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(3)], Standing::Lent("party_12".into()));
        assert_eq!(fills(&dealt)["A"].with, [UnitId(1)], "the lent slots are open");
        // The answer ends with its soldiers at the party: A's place is 9 s off and its slots are open, so they are A's.
        deal.release("party_12");
        let moved = [pawn(1, 0.0, 0.0), pawn(2, 900.0, 0.0), pawn(3, 900.0, 10.0), pawn(5, 1800.0, 600.0)];
        let dealt = deal.deal(&moved, &demands, &[], &straight, Gates::default());
        assert_eq!(dealt[&UnitId(2)], with("A"));
        assert_eq!(dealt[&UnitId(3)], with("A"));
        // An answer that is no longer among the living frees its soldiers without a release, too.
        deal.lend("party_13", "A", 1, at(900.0, 0.0), &moved, &straight);
        let dealt = deal.deal(&moved, &demands, &[], &straight, Gates::default());
        assert!(dealt.values().all(|s| !matches!(s, Standing::Lent(_))));
    }

    #[test]
    fn a_roving_group_keeps_its_rovers_and_fills_from_the_unplaced() {
        let rovers = Demand { name: "R".into(), slots: Slots { any: 0, typed: vec![(PAWN, 2)] }, place: Place::Roving { about: Some(at(5000.0, 0.0)) } };
        let demands = [group("A", 2, 0.0, 0.0), rovers];
        // Two Pawns with A, two more beside them and a Bull: the two Pawns A has no slot for rove; the Bull is surplus.
        let soldiers = [pawn(1, 0.0, 0.0), pawn(2, 10.0, 0.0), pawn(3, 20.0, 0.0), pawn(4, 30.0, 0.0), bull(5, 40.0, 0.0)];
        let mut deal = Deal::default();
        let dealt = deal.deal(&soldiers, &demands, &[], &straight, Gates::default());
        assert_eq!(fills(&dealt)["R"].with, [UnitId(3), UnitId(4)], "the ones best placed for elsewhere are let go of A");
        assert_eq!(dealt[&UnitId(5)], Standing::Surplus);
        // A rover far across the map is still its group's: it has no place to come to.
        let far = [pawn(1, 0.0, 0.0), pawn(2, 10.0, 0.0), pawn(3, 9000.0, 0.0), pawn(4, 9000.0, 50.0), bull(5, 40.0, 0.0)];
        let dealt = deal.deal(&far, &demands, &[], &straight, Gates::default());
        assert_eq!(fills(&dealt)["R"].with, [UnitId(3), UnitId(4)]);
    }
}
