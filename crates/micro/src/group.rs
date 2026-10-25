//! The group as one object (`docs/design/2026-10-24-formation-drills.md` §2, ruled 2026-10-24 night): an anchor with
//! a place, a heading, a pace and a shape, walking the host's route; slots as offsets from it; every member's job to be
//! in its slot. The host declares a group once per decision (`Lane::set_group`) and never orders a member. The lane
//! sends each member to its slot every tick, and the engine's own approach to a goal does the following: a unit sent
//! every tick to a point moving at a pace below its speed settles at that pace, trailing the point inside its brake
//! zone (K-engine-a-slot-moving-at-a-pace-paces-the-unit, measured on the drill bench: a Blitz holds 52-56 behind a
//! slot at 54; the mixed column translated rigidly at 0.9 of the Janus's speed arrived as the formation it set out
//! in, 98 x 320 against the ranks' nominal 96 x 320, where the same column under one order was 923 deep).
//!
//! What the loop holds: the pace (the slowest type's speed in the group times `Pacing::fraction`, 1.0: the anchor is
//! held back by its formed members, so the slowest type regains a lost slot by the anchor waiting, not by a margin on
//! the pace; the bench's 0.9 cost a pure column 4 s for nothing and met no bar the 1.0 missed), the shape (ranks of `form::FILES` across the
//! heading; the contact arc is the lane's, `lib.rs`), coherence (a member within `formed_within` of its slot is
//! formed; one behind it by more than `drag_from` slows the anchor, to a stop at `exception_at` behind; one further
//! than `exception_at` from its slot is an exception reported as away, and is formed again within `rejoin_within`:
//! two thresholds with hysteresis, so the report never flickers), and arrival (the anchor at the route's end and
//! every member that is not an exception in its slot). **The anchor never outruns its body** (2026-10-24 night, the
//! first circuit on Quicksilver: the anchor walked the crow line through ground the Pawns went round, every member
//! fell further than the threshold, and under the first law, which released them, the anchor walked the whole lap
//! alone at full pace with twelve members a map behind it, which no same-speed member can ever close): every member
//! that has been in the body once (within `exception_at` of its slot) holds the anchor by its lag however far behind
//! it is, a fleeing member excepted; a joiner holds nothing until it has come that near. What the group will not wait
//! for, the host takes out of the group: that is the host's decision, from the state's words. Pure over positions;
//! the lane decides what each slot becomes as an order.

use std::collections::HashMap;

use bot_protocol::{UnitId, Vec3};

use crate::form;

/// What a group is to do at its route's end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupKind {
    /// Walk there and stand; nothing on the way is fought (the hands' walk, a Move).
    Go,
    /// Fight to it: the contact arc on anything armed met within the horizon (the hands' advance).
    FightTo,
    /// Attack this unit: the route's end follows its sightings, the arc is on it.
    Attack(UnitId),
    /// Stand at the route's end in formation and fight what comes.
    Hold,
}

/// The host's order to a group: the kind and the route to walk, the goal its last point (one point for a straight
/// line; the host's field gives the points round terrain).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupOrder {
    pub kind: GroupKind,
    pub route: Vec<Vec3>,
}

/// Why a member is out of its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// Added after the group formed and not yet in a slot.
    Joining,
    /// Under the lane's flee this tick.
    Fled,
    /// Not moving while out of its slot.
    Stuck,
    /// Behind its slot within the exception threshold: slows the anchor.
    Behind,
    /// Further than the exception threshold from its slot; once in the body, it holds the anchor like one behind.
    Away,
}

/// What the lane reports of a group this tick (the hands' words; the director's instrument).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupState {
    /// The formation's place, way and speed this tick (elmos a second; 0 when stopped: arrived, at contact, held
    /// back entirely).
    pub anchor: Vec3,
    pub heading: (f32, f32),
    pub pace: f32,
    /// The pace the group would walk at unheld: the slowest type's speed times the fraction.
    pub full_pace: f32,
    /// Members within `formed_within` of their slot.
    pub formed: Vec<UnitId>,
    /// Members out of their slot: why, and seconds to it at the unit's own speed.
    pub exceptions: Vec<(UnitId, Cause, f32)>,
    /// The anchor at the route's end and every member not an exception in its slot.
    pub arrived: bool,
    /// Route points still ahead of the anchor (0 at the end: "two corners to go").
    pub legs_left: usize,
    /// The contact target the group stands on, when engaged (the lane's arc).
    pub engaged: Option<UnitId>,
}

/// The loop's numbers, each with a drill on the bench that sets it (design §2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pacing {
    /// The pace as a share of the slowest type's speed. 1.0: the drag below holds the anchor to a lagging member
    /// (the bench, 2026-10-24 night: at 1.0 the mix arrived 33 / 35 / 37 with twelve formed every second and the
    /// Blitz column 20 / 20 / 21; at 0.9, 36 / 38 / 40 and 22 / 22 / 23, the fraction's price with nothing bought).
    pub fraction: f32,
    /// Frames between a member's slot orders (3: every tick; the bench's 1 s arm held the per-second mean too).
    pub cadence: i32,
    /// A member this near its slot is formed (two hulls).
    pub formed_within: f32,
    /// A member of the body further behind its slot than this slows the anchor, to a stop at `exception_at`.
    pub drag_from: f32,
    /// A member further than this from its slot is an exception (away); a joiner this near is in the body from then on.
    pub exception_at: f32,
    /// An away member this near its slot is formed again.
    pub rejoin_within: f32,
}

/// A route point that moved less than this since the host's last order is the same point (an enemy's centre
/// creeping; the host re-declaring every tick): the anchor's leg and the slots' deal stand.
pub const ROUTE_MOVED: f32 = 48.0;

impl Default for Pacing {
    fn default() -> Pacing {
        Pacing { fraction: 1.0, cadence: 3, formed_within: 2.0 * form::HULL_SPACING, drag_from: form::HULL_SPACING, exception_at: 300.0, rejoin_within: 1.5 * form::HULL_SPACING }
    }
}

/// A member as the group sees it this tick.
#[derive(Clone, Copy, Debug)]
pub struct Walker {
    pub id: UnitId,
    pub pos: Vec3,
    /// Its type's speed, elmos a second.
    pub speed: f32,
    /// Its speed this tick, elmos a second.
    pub moving: f32,
    pub fleeing: bool,
}

/// One declared group.
#[derive(Clone, Debug)]
pub struct Group {
    pub name: String,
    pub members: Vec<UnitId>,
    pub order: GroupOrder,
    pub anchor: Vec3,
    pub heading: (f32, f32),
    /// The route point the anchor walks to next; `route.len()` at the end.
    pub leg: usize,
    /// Each member's slot index, sticky until the membership or the route changes.
    slot_of: HashMap<UnitId, usize>,
    /// Members out of their slot beyond the exception threshold (away), with the frame it happened.
    away: HashMap<UnitId, i32>,
    /// Members that have been in a slot at least once since joining.
    formed_once: HashMap<UnitId, bool>,
    /// Members that have been within the exception threshold of their slot at least once: the body, whose lag holds
    /// the anchor.
    in_body: HashMap<UnitId, bool>,
    /// Stopped by the lane: at contact the anchor stands.
    pub stopped: bool,
    pub arrived: bool,
    pub engaged: Option<UnitId>,
    last_frame: Option<i32>,
    /// The pace this tick (held back or not) and the full pace.
    pub pace: f32,
    pub full_pace: f32,
    /// The spacing the slots were last laid at.
    spacing: f32,
    /// Each member's slot this tick.
    slots: HashMap<UnitId, Vec3>,
    /// The exceptions this tick, as reported.
    exceptions: Vec<(UnitId, Cause, f32)>,
}

impl Group {
    /// A new group: the anchor placed so the ranks' centroid is the members' (`positions`), facing the route's first
    /// point.
    pub fn new(name: &str, members: Vec<UnitId>, order: GroupOrder, positions: &[Vec3], spacing: f32) -> Group {
        let centroid = form::centroid(positions.iter().copied()).unwrap_or_else(|| order.route.first().copied().unwrap_or_default());
        let first = order.route.first().copied().unwrap_or(centroid);
        let heading = if first.dist2d(centroid) > 1.0 { form::heading(centroid, first) } else { (1.0, 0.0) };
        let mut group = Group {
            name: name.to_string(),
            members,
            order,
            anchor: centroid,
            heading,
            leg: 0,
            slot_of: HashMap::new(),
            away: HashMap::new(),
            formed_once: HashMap::new(),
            in_body: HashMap::new(),
            stopped: false,
            arrived: false,
            engaged: None,
            last_frame: None,
            pace: 0.0,
            full_pace: 0.0,
            spacing,
            slots: HashMap::new(),
            exceptions: Vec::new(),
        };
        group.anchor = group.anchor_for(centroid, heading);
        group
    }

    /// The host's order again: a new kind, a route of another length or a point moved `ROUTE_MOVED` re-aims the
    /// anchor from where it stands (no jump) and re-deals the slots by least travel (a reverse: nobody crosses the
    /// formation); a point moved less is taken silently. A new membership re-deals the slots too.
    pub fn update(&mut self, members: Vec<UnitId>, order: GroupOrder) {
        let moved = order.route.len() != self.order.route.len() || order.route.iter().zip(&self.order.route).any(|(a, b)| a.dist2d(*b) > ROUTE_MOVED);
        if order.kind != self.order.kind || moved {
            self.order = order;
            self.leg = 0;
            self.arrived = false;
            self.slot_of.clear();
        } else {
            self.order.route = order.route;
        }
        if members != self.members {
            for id in &members {
                if !self.members.contains(id) {
                    self.formed_once.entry(*id).or_insert(false);
                }
            }
            self.members = members;
            self.slot_of.clear();
        }
    }

    /// The anchor that puts the ranks' centroid at `centroid` facing `heading`.
    fn anchor_for(&self, centroid: Vec3, heading: (f32, f32)) -> Vec3 {
        let n = self.members.len().max(1);
        let at_origin = form::slots(Vec3::default(), heading, n, self.spacing, form::FILES);
        let c0 = form::centroid(at_origin.iter().copied()).unwrap_or_default();
        Vec3 { x: centroid.x - c0.x, y: centroid.y, z: centroid.z - c0.z }
    }

    /// The anchor set down again at the members' centroid (after contact, where the body stands is where the march
    /// resumes from), facing the next route point.
    pub fn reanchor(&mut self, centroid: Vec3) {
        if let Some(next) = self.order.route.get(self.leg)
            && next.dist2d(centroid) > 1.0
        {
            self.heading = form::heading(centroid, *next);
        }
        self.anchor = self.anchor_for(centroid, self.heading);
    }

    /// An Attack's goal follows its target: the route's end moves to `at`; an anchor that had arrived walks on when
    /// the target has moved `ROUTE_MOVED`.
    pub fn retarget(&mut self, at: Vec3) {
        let len = self.order.route.len();
        let Some(end) = self.order.route.last_mut() else { return };
        let far = end.dist2d(at) > ROUTE_MOVED;
        *end = at;
        if far && self.leg >= len {
            self.leg = len - 1;
            self.arrived = false;
        }
    }

    /// Whether the anchor is at the route's end.
    pub fn at_end(&self) -> bool {
        self.leg >= self.order.route.len()
    }

    /// The anchor walks `distance` along the route, leg by leg.
    fn advance(&mut self, distance: f32) {
        let mut left = distance;
        while left > 0.0 && self.leg < self.order.route.len() {
            let target = self.order.route[self.leg];
            let d = self.anchor.dist2d(target);
            if d <= left {
                if d > 0.5 {
                    self.heading = form::heading(self.anchor, target);
                }
                self.anchor = Vec3 { y: target.y, ..target };
                left -= d;
                self.leg += 1;
            } else {
                let h = form::heading(self.anchor, target);
                self.heading = h;
                self.anchor = Vec3 { x: self.anchor.x + h.0 * left, y: self.anchor.y, z: self.anchor.z + h.1 * left };
                left = 0.0;
            }
        }
    }

    /// One tick: the pace from the walkers' types, the anchor held back by its formed members and advanced along the
    /// route (unless stopped, or `march` is off, when it stands at the route's end), the slots dealt, formed and
    /// exceptions told apart. Returns each walker's slot. `spacing` is the lane's for this tick.
    pub fn tick(&mut self, frame: i32, walkers: &[Walker], spacing: f32, pacing: &Pacing, march: bool) -> &HashMap<UnitId, Vec3> {
        let dt = self.last_frame.map_or(0.0, |f| (frame - f).max(0) as f32 / 30.0);
        self.last_frame = Some(frame);
        self.spacing = spacing;
        let slowest = walkers.iter().map(|w| w.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        self.full_pace = if slowest.is_finite() { slowest * pacing.fraction } else { 0.0 };
        if !march && !self.at_end() {
            // No pacing: the slots stand at the destination and every unit walks there at its own speed.
            if let Some(end) = self.order.route.last().copied() {
                if end.dist2d(self.anchor) > 1.0 {
                    self.heading = form::heading(self.anchor, end);
                }
                self.anchor = Vec3 { y: end.y, ..end };
            }
            self.leg = self.order.route.len();
        } else if self.leg < self.order.route.len() {
            let next = self.order.route[self.leg];
            if next.dist2d(self.anchor) > 0.5 {
                self.heading = form::heading(self.anchor, next);
            }
        }
        // Who is where against last tick's slots: the drag on the anchor from the body's members behind theirs,
        // however far (the anchor never outruns its body); a joiner is in the body once within the threshold.
        let h = self.heading;
        let mut max_lag = 0.0f32;
        for w in walkers {
            let Some(slot) = self.slots.get(&w.id) else { continue };
            let off = w.pos.dist2d(*slot);
            let away = self.away.contains_key(&w.id);
            if away {
                if off <= pacing.rejoin_within {
                    self.away.remove(&w.id);
                }
            } else if off > pacing.exception_at {
                self.away.insert(w.id, frame);
            }
            let in_body = self.in_body.entry(w.id).or_insert(false);
            if off <= pacing.exception_at {
                *in_body = true;
            }
            if *in_body && !w.fleeing {
                let behind = (slot.x - w.pos.x) * h.0 + (slot.z - w.pos.z) * h.1;
                max_lag = max_lag.max(behind);
            }
        }
        let held = if max_lag <= pacing.drag_from { 1.0 } else { ((pacing.exception_at - max_lag) / (pacing.exception_at - pacing.drag_from)).clamp(0.0, 1.0) };
        self.pace = if self.stopped || self.at_end() || !march { 0.0 } else { self.full_pace * held };
        if self.pace > 0.0 {
            self.advance(self.pace * dt);
        }
        // The slots, sticky by index; dealt by least travel when the deal is stale.
        let ids: Vec<UnitId> = walkers.iter().map(|w| w.id).collect();
        let n = ids.len();
        let slots = form::slots(self.anchor, self.heading, n.max(1), spacing, form::FILES);
        if n > 0 && (self.slot_of.len() != n || ids.iter().any(|id| !self.slot_of.contains_key(id))) {
            let positions: Vec<Vec3> = walkers.iter().map(|w| w.pos).collect();
            let deal = form::assign(&positions, &slots[..n], self.heading);
            self.slot_of = ids.iter().copied().zip(deal).collect();
        }
        self.slots = ids.iter().map(|id| (*id, slots[self.slot_of.get(id).copied().unwrap_or(0).min(n.saturating_sub(1))])).collect();
        // Formed and exceptions, from this tick's slots.
        self.exceptions.clear();
        let mut all_formed = true;
        for w in walkers {
            let slot = self.slots[&w.id];
            let off = w.pos.dist2d(slot);
            let formed = off <= pacing.formed_within;
            let formed_once = self.formed_once.entry(w.id).or_insert(false);
            if formed {
                *formed_once = true;
            }
            let cause = if formed && !self.away.contains_key(&w.id) {
                None
            } else if w.fleeing {
                Some(Cause::Fled)
            } else if !*formed_once {
                Some(Cause::Joining)
            } else if w.moving < 5.0 {
                // Said before away: a unit standing still a map from its slot is stuck first (the circuit's two
                // Pawns given on bad ground never moved and were reported away for nine minutes).
                Some(Cause::Stuck)
            } else if self.away.contains_key(&w.id) {
                Some(Cause::Away)
            } else {
                Some(Cause::Behind)
            };
            if let Some(cause) = cause {
                let seconds = if w.speed > 0.0 { off / w.speed } else { f32::INFINITY };
                self.exceptions.push((w.id, cause, seconds));
                if !self.away.contains_key(&w.id) {
                    all_formed = false;
                }
            }
        }
        self.arrived = self.at_end() && all_formed && !walkers.is_empty();
        &self.slots
    }

    /// The report.
    pub fn state(&self, walkers: &[Walker]) -> GroupState {
        let formed = walkers.iter().filter(|w| !self.exceptions.iter().any(|(id, _, _)| *id == w.id)).map(|w| w.id).collect();
        GroupState {
            anchor: self.anchor,
            heading: self.heading,
            pace: self.pace,
            full_pace: self.full_pace,
            formed,
            exceptions: self.exceptions.clone(),
            arrived: self.arrived,
            legs_left: self.order.route.len().saturating_sub(self.leg),
            engaged: self.engaged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn walkers(positions: &[(i32, f32, f32)], speed: f32) -> Vec<Walker> {
        positions.iter().map(|(id, x, z)| Walker { id: UnitId(*id), pos: at(*x, *z), speed, moving: speed, fleeing: false }).collect()
    }

    fn march(group: &mut Group, walkers: &mut [Walker], frames: i32, step: i32, pacing: &Pacing) {
        // Each tick every walker stands 20 behind its slot (the brake zone's trail), as the engine would have it.
        let mut frame = 0;
        while frame < frames {
            frame += step;
            let slots = group.tick(frame, walkers, 64.0, pacing, true).clone();
            for w in walkers.iter_mut() {
                let s = slots[&w.id];
                w.pos = at(s.x - group.heading.0 * 20.0, s.z - group.heading.1 * 20.0);
            }
        }
    }

    #[test]
    fn the_anchor_walks_the_route_at_the_slowest_type_times_the_fraction() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        w[1].speed = 54.0;
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder { kind: GroupKind::Go, route: vec![at(3000.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing, true);
        let start = group.anchor;
        march(&mut group, &mut w, 300, 3, &pacing);
        // Ten seconds at the slowest type's 54 = 540 elmos east, on the axis.
        assert!((group.anchor.x - start.x - 540.0).abs() < 3.0, "{} from {}", group.anchor.x, start.x);
        assert!(group.anchor.z.abs() < 1.0);
        assert_eq!(group.heading, (1.0, 0.0));
        assert!((group.full_pace - 54.0).abs() < 0.1);
        assert!(!group.arrived);
        let state = group.state(&w);
        assert_eq!(state.formed.len(), 2);
        assert!(state.exceptions.is_empty());
    }

    #[test]
    fn the_anchor_arrives_and_stops_at_the_end_with_everyone_in_slot() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder { kind: GroupKind::Go, route: vec![at(400.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing, true);
        march(&mut group, &mut w, 600, 3, &pacing);
        assert!(group.at_end());
        assert_eq!(group.pace, 0.0);
        assert!(group.arrived, "{:?}", group.state(&w));
        assert!((group.anchor.x - 400.0).abs() < 0.5);
    }

    #[test]
    fn a_member_behind_its_slot_slows_the_anchor_and_far_behind_stops_it() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder { kind: GroupKind::Go, route: vec![at(5000.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing, true);
        // Unit 2 lags 150 behind its slot: the anchor walks at less than the pace.
        let slots = group.tick(3, &w, 64.0, &pacing, true).clone();
        w[0].pos = at(slots[&UnitId(1)].x - 20.0, -32.0);
        w[1].pos = at(slots[&UnitId(2)].x - 150.0, 32.0);
        group.tick(6, &w, 64.0, &pacing, true);
        assert!(group.pace > 0.0 && group.pace < group.full_pace * 0.7, "pace {} of {}", group.pace, group.full_pace);
        let state = group.state(&w);
        assert_eq!(state.exceptions.len(), 1);
        assert_eq!(state.exceptions[0].1, Cause::Behind);
        // 400 behind: the cause Away, and the anchor stops for it (it never outruns its body); within 96 again it
        // is formed again.
        w[1].pos = at(slots[&UnitId(2)].x - 400.0, 32.0);
        group.tick(9, &w, 64.0, &pacing, true);
        assert_eq!(group.pace, 0.0, "the anchor waits for a member of its body a map behind");
        assert_eq!(group.state(&w).exceptions[0].1, Cause::Away);
        let slots = group.tick(12, &w, 64.0, &pacing, true).clone();
        w[1].pos = at(slots[&UnitId(2)].x - 150.0, 32.0);
        group.tick(15, &w, 64.0, &pacing, true);
        assert!(group.pace > 0.0 && group.pace < group.full_pace * 0.7, "pace {} of {}", group.pace, group.full_pace);
        assert_eq!(group.state(&w).exceptions[0].1, Cause::Away, "still away until within the rejoin distance");
        let slots = group.tick(18, &w, 64.0, &pacing, true).clone();
        w[1].pos = at(slots[&UnitId(2)].x - 20.0, 32.0);
        group.tick(21, &w, 64.0, &pacing, true);
        assert!(group.state(&w).exceptions.is_empty());
    }

    #[test]
    fn a_new_route_turns_the_anchor_where_it_stands_and_redeals_the_slots() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0), (3, -96.0, 0.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2), UnitId(3)], GroupOrder { kind: GroupKind::Go, route: vec![at(3000.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing, true);
        march(&mut group, &mut w, 150, 3, &pacing);
        let before = group.anchor;
        group.update(group.members.clone(), GroupOrder { kind: GroupKind::Go, route: vec![at(-3000.0, 0.0)] });
        let slots = group.tick(153, &w, 64.0, &pacing, true).clone();
        assert!(group.anchor.dist2d(before) < 15.0, "no jump: one tick's walk at most");
        assert!(group.heading.0 < -0.99);
        // The rear unit of the march east is now the front rank's: nobody crosses the formation.
        let rear_before = w.iter().min_by(|a, b| a.pos.x.total_cmp(&b.pos.x)).unwrap().id;
        let front_slot_x = slots.values().map(|s| s.x).fold(f32::INFINITY, f32::min);
        assert!((slots[&rear_before].x - front_slot_x).abs() < 1.0);
    }

    #[test]
    fn march_off_puts_the_slots_at_the_destination_at_once() {
        let pacing = Pacing::default();
        let w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder { kind: GroupKind::Go, route: vec![at(2000.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        let slots = group.tick(0, &w, 64.0, &pacing, false).clone();
        assert!(group.at_end());
        assert!((slots[&UnitId(1)].x - 2000.0).abs() < 1.0);
        assert_eq!(group.pace, 0.0);
        assert!(!group.arrived);
    }

    #[test]
    fn a_joiner_is_an_exception_until_it_first_reaches_a_slot() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder { kind: GroupKind::Go, route: vec![at(3000.0, 0.0)] }, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing, true);
        march(&mut group, &mut w, 60, 3, &pacing);
        group.update(vec![UnitId(1), UnitId(2), UnitId(3)], group.order.clone());
        w.push(Walker { id: UnitId(3), pos: at(-900.0, 0.0), speed: 100.0, moving: 100.0, fleeing: false });
        group.tick(63, &w, 64.0, &pacing, true);
        let state = group.state(&w);
        assert_eq!(state.exceptions.len(), 1);
        assert_eq!(state.exceptions[0].0, UnitId(3));
        assert_eq!(state.exceptions[0].1, Cause::Joining);
        assert!(state.exceptions[0].2 > 8.0, "seconds to its slot at its speed");
        assert!((group.pace - group.full_pace).abs() < 0.01, "a joiner does not hold the anchor");
    }
}
