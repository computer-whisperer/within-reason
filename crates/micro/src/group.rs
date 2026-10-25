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
//! heading, filled from the front; the contact arc is the lane's, `lib.rs`; the slots re-dealt by least travel with
//! the anchor set down on the body when the anchor turns `REDEAL_TURN_DEG` or more at a route point, design D14: a
//! turn is a re-labelling of who stands where and an about-face is free, where the slots rotating about the anchor
//! with each member chasing its own knocked 13 of 14 out of slot at every corner of the circuit), coherence (a member within `formed_within` of its slot is
//! formed; one behind it by more than `drag_from` slows the anchor, to a stop at `exception_at` behind; one further
//! than `exception_at` from its slot is an exception reported as away, and is formed again within `rejoin_within`:
//! two thresholds with hysteresis, so the report never flickers), and arrival (the anchor at the route's end and
//! every member that is not an exception in its slot). **The anchor never outruns its body** (2026-10-24 night, the
//! first circuit on Quicksilver: the anchor walked the crow line through ground the Pawns went round, every member
//! fell further than the threshold, and under the first law, which released them, the anchor walked the whole lap
//! alone at full pace with twelve members a map behind it, which no same-speed member can ever close): every member
//! that has been in the body once (stood within `formed_within` of its slot) holds the anchor by its lag however far
//! behind it is, a fleeing member excepted; a joiner holds nothing until it has stood in its slot. What the group will not wait
//! for, the host takes out of the group: that is the host's decision, from the state's words. Pure over positions;
//! the lane decides what each slot becomes as an order.

use std::collections::HashMap;

use bot_protocol::{UnitId, Vec3};

use crate::form;

/// `WITHIN_REASON_GROUP_DEBUG=1`: the group says on stderr when it re-deals, turns, or its slots jump.
fn debug_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WITHIN_REASON_GROUP_DEBUG").is_some_and(|v| v != "0" && !v.is_empty()))
}

/// What a group is to do (design D15: every way the lane owns a set of units is a kind of group). The first four
/// walk the order's route as a formation; a hunt and a rove have no anchor and no slots, and are run raw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GroupKind {
    /// Walk there and stand; nothing on the way is fought (the hands' walk, a Move).
    Go,
    /// Fight to it: the contact arc on anything armed met within the horizon (the hands' advance).
    FightTo,
    /// Attack this unit: the route's end follows its sightings, the arc is on it.
    Attack(UnitId),
    /// Stand at the route's end in formation and fight what comes.
    Hold,
    /// Every member attacks the quarry by id each tick until it is dead, lost or the leash is reached (H-MICRO-HUNT;
    /// `lib.rs` `hunt`). The route is ignored.
    Hunt(HuntOrder),
    /// Every member is a rover (H-MICRO-ROVE, `rove.rs`): it evades, kills what it finds unguarded and looks at what
    /// the host knows least, within the region when one is given (north-west and south-east corners). The route is
    /// ignored.
    Rove { within: Option<(Vec3, Vec3)> },
}

/// A hunt: the unit to kill, the leash beyond which the hunt ends (measured for each hunter from where it stood when
/// the lane first ran it under this hunt), how long the quarry may be out of sight and radar before the hunt ends as
/// "lost" (`HUNT_LOST_FRAMES` unless the host says longer: a quarry that cloaks is hidden where it stands, not gone),
/// and the frame the host began it: a hunt picked again on the same quarry after an end is a new order, not the
/// silent re-declaration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntOrder {
    pub quarry: UnitId,
    pub leash: f32,
    pub lost_frames: i32,
    pub since: i32,
}

/// The host's order to a group: the kind, the route to walk, the goal its last point (one point for a straight line;
/// the host's field gives the points round terrain), and whether to march it: `march` false puts the slots at the
/// route's end at once and every member walks there at its own speed (the duel's `ranks` arm, the march's end state
/// without its pacing).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupOrder {
    pub kind: GroupKind,
    pub route: Vec<Vec3>,
    pub march: bool,
}

impl GroupOrder {
    /// A marching order of this kind along this route.
    pub fn new(kind: GroupKind, route: Vec<Vec3>) -> GroupOrder {
        GroupOrder { kind, route, march: true }
    }
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
    /// A member of the body further behind its slot than this slows the anchor, to a stop at `exception_at`. The
    /// formed band (128): a unit sent every tick to a point moving at the pace settles behind it by a distance of
    /// the engine's (72-86 for a Pawn at 80, about twice that for a Stout), so a drag from 64 held a body of twenty
    /// Pawns at 0.90 of full pace on a straight walk with every rank the same distance behind (walk-redeal-pawn20),
    /// a body of twelve at 0.98; the formed band holds nothing it calls formed.
    pub drag_from: f32,
    /// A member further than this from its slot is an exception (away).
    pub exception_at: f32,
    /// An away member this near its slot is formed again.
    pub rejoin_within: f32,
}

/// A route point that moved less than this since the host's last order is the same point (an enemy's centre
/// creeping; the host re-declaring every tick): the anchor's leg and the slots' deal stand.
pub const ROUTE_MOVED: f32 = 48.0;
/// A heading change at a route point of this many degrees or more re-deals the slots with the anchor set down on the
/// body (D14); a smaller bend snaps the slots about the anchor and the deal stays. Twenty: a four-rank body's rear
/// corners stand 370 from the anchor, so a snap of 23 degrees moved them 146 and put 17 of 20 out of the band
/// (the circuit9 trace), while a re-deal that keeps the trail costs the body nothing.
pub const REDEAL_TURN_DEG: f32 = 20.0;

impl Default for Pacing {
    fn default() -> Pacing {
        Pacing { fraction: 1.0, cadence: 3, formed_within: 2.0 * form::HULL_SPACING, drag_from: 2.0 * form::HULL_SPACING, exception_at: 300.0, rejoin_within: 1.5 * form::HULL_SPACING }
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
    /// Each member's slot index, sticky until the membership or the route changes, the anchor turns `REDEAL_TURN_DEG`
    /// at a route point or is set down after contact.
    slot_of: HashMap<UnitId, usize>,
    /// Members out of their slot beyond the exception threshold (away), with the frame it happened.
    away: HashMap<UnitId, i32>,
    /// Members that have stood in a slot at least once since joining: the body, whose lag holds the anchor (a joiner
    /// holds nothing until then; when it held from within the exception threshold, a joiner arriving at 280 behind
    /// its slot stalled the whole body to a pace of 16, twice a lap: circuit9).
    formed_once: HashMap<UnitId, bool>,
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
        group.anchor = group.anchor_for(centroid, heading, group.members.len());
        group
    }

    /// The host's order again: a new kind, a route of another length or a point moved `ROUTE_MOVED` re-aims the
    /// anchor from where it stands (no jump) and re-deals the slots by least travel (a reverse: nobody crosses the
    /// formation); a point moved less is taken silently. A new membership re-deals the slots too.
    pub fn update(&mut self, members: Vec<UnitId>, order: GroupOrder) {
        let moved = order.route.len() != self.order.route.len() || order.route.iter().zip(&self.order.route).any(|(a, b)| a.dist2d(*b) > ROUTE_MOVED);
        if order.kind != self.order.kind || order.march != self.order.march || moved {
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

    /// The anchor that puts the centroid of `count` ranks' slots at `centroid` facing `heading`.
    fn anchor_for(&self, centroid: Vec3, heading: (f32, f32), count: usize) -> Vec3 {
        let at_origin = form::slots(Vec3::default(), heading, count.max(1), self.spacing, form::FILES);
        let c0 = form::centroid(at_origin.iter().copied()).unwrap_or_default();
        Vec3 { x: centroid.x - c0.x, y: centroid.y, z: centroid.z - c0.z }
    }

    /// The anchor set down again at the members' centroid (after contact, where the body stands is where the march
    /// resumes from), facing the next route point; the slots re-dealt where the members stand (the arc's places are
    /// not the march's). A route the anchor had finished is taken up again when the body is set down short of its
    /// end (assault2-mix12-hlt2llt2-around-ranks: with the march off the anchor stands at the end from the first tick,
    /// so after a contact that ended with the last turret out of sight the body was set down 470 short of the goal,
    /// every member formed round it, "arrived", and six of eight duels timed out there).
    pub fn reanchor(&mut self, centroid: Vec3) {
        let len = self.order.route.len();
        let anchor = self.anchor_for(centroid, self.heading, self.members.len());
        if self.leg >= len
            && let Some(end) = self.order.route.last()
            && end.dist2d(anchor) > ROUTE_MOVED
        {
            self.leg = len - 1;
            self.arrived = false;
        }
        if let Some(next) = self.order.route.get(self.leg)
            && next.dist2d(centroid) > 1.0
        {
            self.heading = form::heading(centroid, *next);
        }
        self.anchor = self.anchor_for(centroid, self.heading, self.members.len());
        self.slot_of.clear();
    }

    /// The body's depth behind the anchor along the heading: the mean of its members' (in the body, not away, not
    /// fleeing), or the layout's half depth for the members when nobody is in the body yet. The carry at a corner
    /// (`target`) is measured, so the trail the members keep behind their slots (about fifty at pace, the brake zone)
    /// and the ranks they actually hold are in it: from the rank count alone the body's centroid was still a hundred
    /// short of the corner when the anchor turned (the circuit7-carry trace).
    fn body_depth(&self, walkers: &[Walker]) -> f32 {
        let h = self.heading;
        let depths: Vec<f32> = walkers
            .iter()
            .filter(|w| !w.fleeing && !self.away.contains_key(&w.id) && self.formed_once.get(&w.id).copied().unwrap_or(false))
            .map(|w| (self.anchor.x - w.pos.x) * h.0 + (self.anchor.z - w.pos.z) * h.1)
            .collect();
        if depths.is_empty() {
            return (self.members.len().max(1).div_ceil(form::FILES) - 1) as f32 * form::RANK_GAP / 2.0;
        }
        (depths.iter().sum::<f32>() / depths.len() as f32).max(0.0)
    }

    /// The point the anchor walks to for `leg`: the route's point, carried on along its leg by `carry` (the body's
    /// depth behind the anchor) when a point precedes it and another follows (at most half way to the next). The route is the body's centre line and
    /// the anchor is its front: an anchor that turned at the point would leave the body's centroid half its depth
    /// short of the corner, so the ranks set down over the body at the turn (D14) would lie that far inside the new
    /// leg, the inside file in the wall the route kept its room from, and the next point, 256 on, nearly abreast of
    /// the anchor, which swung the heading a second time a tick later (the first re-deal runs: 60-85 degrees, the
    /// body knocked out of slot again). Carried on, the body's centroid turns at the point and the new ranks lie on
    /// the new leg. The first point has no leg into it, so no carry.
    fn target(&self, leg: usize, carry: f32) -> Vec3 {
        let route = &self.order.route;
        let point = route[leg];
        let Some(next) = route.get(leg + 1) else { return point };
        if leg == 0 || carry <= 0.0 {
            return point;
        }
        let from = route[leg - 1];
        if from.dist2d(point) < 1.0 {
            return point;
        }
        let u = form::heading(from, point);
        let carry = carry.min(point.dist2d(*next) / 2.0);
        Vec3 { x: point.x + u.0 * carry, y: point.y, z: point.z + u.1 * carry }
    }

    /// The anchor set down so the body's ranks lie over the body (design D14: at a turn the slots are re-dealt where
    /// the members stand, so the turn is a re-labelling and an about-face costs nobody a step). The body is the
    /// members in it, not away and not fleeing, and the slot count is the body's: with the joiners' slots counted the
    /// anchor would stand a rank ahead of the body. On the members themselves, not the trail they walked behind their
    /// slots ahead of them: set down there everyone is formed at once and the body walks off at full pace, the trail
    /// re-forming in a few seconds without drag (circuit9b, a 126-degree corner: pace 70 to 87, 20 formed throughout);
    /// set down the trail ahead, the trail added to the re-deal's travel and a 130-degree corner put 19 of 20 out of
    /// the band (circuit10: pace 21, formed 1). A body of nobody leaves the anchor where it is.
    fn settle_on_body(&mut self, walkers: &[Walker]) {
        let body: Vec<Vec3> = walkers
            .iter()
            .filter(|w| !w.fleeing && !self.away.contains_key(&w.id) && self.formed_once.get(&w.id).copied().unwrap_or(false))
            .map(|w| w.pos)
            .collect();
        if let Some(centroid) = form::centroid(body.iter().copied()) {
            self.anchor = self.anchor_for(centroid, self.heading, body.len());
        }
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

    /// The anchor walks `distance` along the route, leg by leg, each intermediate point carried on by `carry`.
    fn advance(&mut self, distance: f32, carry: f32) {
        let mut left = distance;
        while left > 0.0 && self.leg < self.order.route.len() {
            let target = self.target(self.leg, carry);
            let d = self.anchor.dist2d(target);
            if d <= left {
                if d >= 1.0 {
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
    /// route (unless stopped, or the order's `march` is off, when it stands at the route's end), the slots dealt, formed and
    /// exceptions told apart. Returns each walker's slot. `spacing` is the lane's for this tick.
    pub fn tick(&mut self, frame: i32, walkers: &[Walker], spacing: f32, pacing: &Pacing) -> &HashMap<UnitId, Vec3> {
        let march = self.order.march;
        let dt = self.last_frame.map_or(0.0, |f| (frame - f).max(0) as f32 / 30.0);
        self.last_frame = Some(frame);
        self.spacing = spacing;
        let carry = self.body_depth(walkers);
        let slowest = walkers.iter().map(|w| w.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        self.full_pace = if slowest.is_finite() { slowest * pacing.fraction } else { 0.0 };
        if !march && !self.at_end() {
            // No pacing: the slots stand at the destination and every unit walks there at its own speed.
            if let Some(end) = self.order.route.last().copied() {
                if end.dist2d(self.anchor) >= 1.0 {
                    self.heading = form::heading(self.anchor, end);
                }
                self.anchor = Vec3 { y: end.y, ..end };
            }
            self.leg = self.order.route.len();
        } else if self.leg < self.order.route.len() {
            let next = self.target(self.leg, carry);
            // (`form::heading` says east for two points under 1.0 apart: a Hold at the body's own place spun the
            // formation east at the route's end before every guard was 1.0.)
            if next.dist2d(self.anchor) >= 1.0 {
                self.heading = form::heading(self.anchor, next);
            }
        }
        // Who is where against last tick's slots: the drag on the anchor from the body's members behind theirs,
        // however far (the anchor never outruns its body); a joiner is in the body once it has stood in its slot.
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
            let in_body = self.formed_once.entry(w.id).or_insert(false);
            if off <= pacing.formed_within {
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
            let (heading_before, leg_before) = (self.heading, self.leg);
            self.advance(self.pace * dt, carry);
            // A route point passed: facing the next, and at a turn of `REDEAL_TURN_DEG` or more the anchor is set
            // down on the body and the slots re-dealt where the members stand (D14).
            if self.leg > leg_before && self.leg < self.order.route.len() {
                let next = self.target(self.leg, carry);
                if next.dist2d(self.anchor) >= 1.0 {
                    self.heading = form::heading(self.anchor, next);
                }
                let turn = heading_before.0 * self.heading.0 + heading_before.1 * self.heading.1;
                if turn < REDEAL_TURN_DEG.to_radians().cos() {
                    let before = self.anchor;
                    // Set down on the body facing the next target from there, twice: the setting-down moves the
                    // anchor, and the ranks are laid at the heading from where it stands (laid at the heading from
                    // where it turned, they swung 25-35 degrees more a tick later: the circuit7-carry trace).
                    for _ in 0..2 {
                        self.settle_on_body(walkers);
                        if next.dist2d(self.anchor) >= 1.0 {
                            self.heading = form::heading(self.anchor, next);
                        }
                    }
                    self.slot_of.clear();
                    if debug_on() {
                        let body = walkers.iter().filter(|w| !w.fleeing && !self.away.contains_key(&w.id) && self.formed_once.get(&w.id).copied().unwrap_or(false)).count();
                        eprintln!(
                            "[group {} f={frame}] TURN {:.0} deg at leg {}: anchor ({:.0},{:.0}) set down at ({:.0},{:.0}) on a body of {body} of {} (carry {carry:.0})",
                            self.name,
                            turn.clamp(-1.0, 1.0).acos().to_degrees(),
                            self.leg,
                            before.x,
                            before.z,
                            self.anchor.x,
                            self.anchor.z,
                            walkers.len()
                        );
                    }
                }
            }
        }
        // The slots, sticky by index; dealt by least travel when the deal is stale.
        let ids: Vec<UnitId> = walkers.iter().map(|w| w.id).collect();
        let n = ids.len();
        let slots = form::slots(self.anchor, self.heading, n.max(1), spacing, form::FILES);
        let stale = n > 0 && (self.slot_of.len() != n || ids.iter().any(|id| !self.slot_of.contains_key(id)));
        if stale {
            let positions: Vec<Vec3> = walkers.iter().map(|w| w.pos).collect();
            let deal = form::assign(&positions, &slots[..n]);
            self.slot_of = ids.iter().copied().zip(deal).collect();
        }
        let previous = std::mem::take(&mut self.slots);
        self.slots = ids.iter().map(|id| (*id, slots[self.slot_of.get(id).copied().unwrap_or(0).min(n.saturating_sub(1))])).collect();
        if debug_on() {
            // Of the members that stood within the band of their slot last tick, how many have a slot elsewhere now
            // (further than the anchor's own step and a half spacing), and the furthest.
            let step = self.pace * dt;
            let (mut unsettled, mut worst) = (0usize, 0.0f32);
            for w in walkers {
                if let (Some(p), Some(s)) = (previous.get(&w.id), self.slots.get(&w.id))
                    && w.pos.dist2d(*p) <= pacing.formed_within
                {
                    let d = p.dist2d(*s);
                    if d > step + 32.0 {
                        unsettled += 1;
                        worst = worst.max(d);
                    }
                }
            }
            if stale || unsettled > 0 {
                eprintln!(
                    "[group {} f={frame}] {} n={n} anchor=({:.0},{:.0}) heading=({:.2},{:.2}) leg={} carry={carry:.0} pace={:.0}; {unsettled} formed members' slots moved, the furthest {worst:.0} (the anchor's step {step:.0})",
                    self.name,
                    if stale { "RE-DEAL" } else { "SLOTS JUMPED" },
                    self.anchor.x,
                    self.anchor.z,
                    self.heading.0,
                    self.heading.1,
                    self.leg,
                    self.pace
                );
            }
        }
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
            let slots = group.tick(frame, walkers, 64.0, pacing).clone();
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
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Go, vec![at(3000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
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
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Go, vec![at(400.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
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
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
        // Unit 2 lags 200 behind its slot: the anchor walks at less than the pace.
        let slots = group.tick(3, &w, 64.0, &pacing).clone();
        w[0].pos = at(slots[&UnitId(1)].x - 20.0, -32.0);
        w[1].pos = at(slots[&UnitId(2)].x - 200.0, 32.0);
        group.tick(6, &w, 64.0, &pacing);
        assert!(group.pace > 0.0 && group.pace < group.full_pace * 0.7, "pace {} of {}", group.pace, group.full_pace);
        let state = group.state(&w);
        assert_eq!(state.exceptions.len(), 1);
        assert_eq!(state.exceptions[0].1, Cause::Behind);
        // 400 behind: the cause Away, and the anchor stops for it (it never outruns its body); within 96 again it
        // is formed again.
        w[1].pos = at(slots[&UnitId(2)].x - 400.0, 32.0);
        group.tick(9, &w, 64.0, &pacing);
        assert_eq!(group.pace, 0.0, "the anchor waits for a member of its body a map behind");
        assert_eq!(group.state(&w).exceptions[0].1, Cause::Away);
        let slots = group.tick(12, &w, 64.0, &pacing).clone();
        w[1].pos = at(slots[&UnitId(2)].x - 200.0, 32.0);
        group.tick(15, &w, 64.0, &pacing);
        assert!(group.pace > 0.0 && group.pace < group.full_pace * 0.7, "pace {} of {}", group.pace, group.full_pace);
        assert_eq!(group.state(&w).exceptions[0].1, Cause::Away, "still away until within the rejoin distance");
        let slots = group.tick(18, &w, 64.0, &pacing).clone();
        w[1].pos = at(slots[&UnitId(2)].x - 20.0, 32.0);
        group.tick(21, &w, 64.0, &pacing);
        assert!(group.state(&w).exceptions.is_empty());
    }

    #[test]
    fn a_new_route_turns_the_anchor_where_it_stands_and_redeals_the_slots() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0), (3, -96.0, 0.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2), UnitId(3)], GroupOrder::new(GroupKind::Go, vec![at(3000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
        march(&mut group, &mut w, 150, 3, &pacing);
        let before = group.anchor;
        group.update(group.members.clone(), GroupOrder::new(GroupKind::Go, vec![at(-3000.0, 0.0)]));
        let slots = group.tick(153, &w, 64.0, &pacing).clone();
        assert!(group.anchor.dist2d(before) < 15.0, "no jump: one tick's walk at most");
        assert!(group.heading.0 < -0.99);
        // The rear unit of the march east is now the front rank's: nobody crosses the formation.
        let rear_before = w.iter().min_by(|a, b| a.pos.x.total_cmp(&b.pos.x)).unwrap().id;
        let front_slot_x = slots.values().map(|s| s.x).fold(f32::INFINITY, f32::min);
        assert!((slots[&rear_before].x - front_slot_x).abs() < 1.0);
    }

    /// Twelve in two ranks of six facing east, each at its slot: the front rank on x = 0, the rear 96 behind and offset.
    fn two_ranks_of_six() -> Vec<Walker> {
        let places: Vec<(i32, f32, f32)> = (0..12).map(|i| (i as i32 + 1, if i < 6 { 0.0 } else { -96.0 }, ((i % 6) as f32 - 2.5) * 64.0 + if i < 6 { 0.0 } else { 32.0 })).collect();
        walkers(&places, 100.0)
    }

    #[test]
    fn an_about_face_at_a_route_point_costs_the_body_no_step() {
        let pacing = Pacing::default();
        let mut w = two_ranks_of_six();
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        let mut group = Group::new("a", members, GroupOrder::new(GroupKind::Go, vec![at(200.0, 0.0), at(-3000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        // To the point with everyone in its slot each tick (the engine's following, without the trail), until the
        // anchor passes it and turns back west.
        let mut frame = 0;
        let mut turned = None;
        while frame < 150 {
            let slots = group.tick(frame, &w, 64.0, &pacing).clone();
            if group.heading.0 < 0.0 {
                turned = Some(slots);
                break;
            }
            for u in w.iter_mut() {
                u.pos = slots[&u.id];
            }
            frame += 3;
        }
        let slots = turned.expect("the anchor turned back within five seconds");
        assert!(group.heading.0 < -0.99, "{:?}", group.heading);
        // The walkers stand where last tick's slots were; the new slots lie on them (a degree of the approach's tilt).
        for u in &w {
            assert!(u.pos.dist2d(slots[&u.id]) < 5.0, "unit {:?} at {:?} would walk to {:?}", u.id, u.pos, slots[&u.id]);
        }
        assert!((group.anchor.x - w[6].pos.x).abs() < 5.0, "the old rear rank is the new front: anchor {:?}, rear at {:?}", group.anchor, w[6].pos);
        assert!(group.state(&w).exceptions.is_empty());
        group.tick(frame + 3, &w, 64.0, &pacing);
        assert!((group.pace - group.full_pace).abs() < 0.01, "nobody lags, so the anchor walks on at full pace: {} of {}", group.pace, group.full_pace);
        // Ten behind the slots after that tick's walk: within the drag band, all formed.
        for u in w.iter_mut() {
            u.pos = at(u.pos.x - 10.0, u.pos.z);
        }
        group.tick(frame + 6, &w, 64.0, &pacing);
        assert_eq!(group.state(&w).formed.len(), 12);
    }

    #[test]
    fn the_anchor_carries_on_past_a_corner_by_half_the_depth_so_the_body_turns_on_the_route() {
        let pacing = Pacing::default();
        let mut w = two_ranks_of_six();
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        // East through (100, 0) to (300, 0), then south: a right angle at the second point (the first point has no
        // leg into it and is not carried). Two ranks: half the depth is 48.
        let mut group = Group::new("a", members, GroupOrder::new(GroupKind::Go, vec![at(100.0, 0.0), at(300.0, 0.0), at(300.0, 3000.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        let mut frame = 0;
        let mut turned = None;
        while frame < 300 {
            let before: Vec<Vec3> = w.iter().map(|u| u.pos).collect();
            let slots = group.tick(frame, &w, 64.0, &pacing).clone();
            if group.heading.1 > 0.5 {
                turned = Some((before, slots));
                break;
            }
            for u in w.iter_mut() {
                u.pos = slots[&u.id];
            }
            frame += 3;
        }
        let (before, slots) = turned.expect("the anchor turned south");
        // It turned 48 past the point, where the body's centroid stood on the corner.
        let centroid = form::centroid(before.iter().copied()).unwrap();
        assert!((centroid.x - 300.0).abs() < 15.0 && centroid.z.abs() < 20.0, "the body's centroid at the turn: {centroid:?}");
        assert!((group.anchor.x - 300.0).abs() < 25.0, "the new ranks lie on the new leg: anchor {:?}", group.anchor);
        // A right angle re-dealt where the body stands: a 320 x 96 body turned into a 96 x 320 one, so the flanks walk
        // 130-160 (the least total squared travel, not the least worst) and two thirds of the body less than the
        // formed band; the slots rotating about the anchor moved the outer flank's slot 300-670 on the circuit's
        // corners. An about-face is free (the test above); a right angle is cheap in proportion to the body's depth.
        let travel: Vec<f32> = w.iter().map(|u| u.pos.dist2d(slots[&u.id])).collect();
        let worst = travel.iter().cloned().fold(0.0, f32::max);
        assert!(worst < 170.0, "the worst walk {worst}: {travel:?}");
        assert!(travel.iter().filter(|d| **d <= pacing.formed_within).count() >= 8, "{travel:?}");
        // The next point is straight ahead: no second swing a tick later.
        group.tick(frame + 3, &w, 64.0, &pacing);
        assert!(group.heading.1 > 0.98, "{:?}", group.heading);
    }

    #[test]
    fn a_joiner_takes_a_slot_behind_and_moves_no_body_slot() {
        let pacing = Pacing::default();
        let mut w = two_ranks_of_six();
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        let mut group = Group::new("a", members.clone(), GroupOrder::new(GroupKind::Go, vec![at(5000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
        let before = group.tick(3, &w, 64.0, &pacing).clone();
        let anchor_before = group.anchor;
        let mut with_joiner = members.clone();
        with_joiner.push(UnitId(13));
        group.update(with_joiner, group.order.clone());
        w.push(Walker { id: UnitId(13), pos: at(-900.0, 0.0), speed: 100.0, moving: 100.0, fleeing: false });
        let after = group.tick(6, &w, 64.0, &pacing).clone();
        let walked = group.anchor.x - anchor_before.x;
        for id in &members {
            let (a, b) = (before[id], after[id]);
            assert!((b.x - a.x - walked).abs() < 0.01 && (b.z - a.z).abs() < 0.01, "the joiner moved {id:?}'s slot from {a:?} to {b:?}");
        }
        assert!(after[&UnitId(13)].x < group.anchor.x - 2.0 * form::RANK_GAP + 0.01, "the joiner's slot is a third rank behind: {:?} with the anchor at {:?}", after[&UnitId(13)], group.anchor);
    }

    #[test]
    fn march_off_puts_the_slots_at_the_destination_at_once() {
        let pacing = Pacing::default();
        let w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let order = GroupOrder { march: false, ..GroupOrder::new(GroupKind::Go, vec![at(2000.0, 0.0)]) };
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], order, &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        let slots = group.tick(0, &w, 64.0, &pacing).clone();
        assert!(group.at_end());
        assert!((slots[&UnitId(1)].x - 2000.0).abs() < 1.0);
        assert_eq!(group.pace, 0.0);
        assert!(!group.arrived);
    }

    /// After a contact that ended short of the goal the march resumes to it, with the march off too: the slots stand
    /// at the destination again, not round the body where contact ended (the assault's detour arm, six timeouts).
    #[test]
    fn a_body_set_down_short_of_the_end_after_contact_walks_on() {
        let pacing = Pacing::default();
        let w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let positions: Vec<Vec3> = w.iter().map(|w| w.pos).collect();
        for march in [false, true] {
            let order = GroupOrder { march, ..GroupOrder::new(GroupKind::FightTo, vec![at(2000.0, 0.0)]) };
            let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], order, &positions, 64.0);
            group.tick(0, &w, 64.0, &pacing);
            if !march {
                assert!(group.at_end());
            } else {
                // Walk the anchor to the end.
                group.leg = 1;
                group.anchor = at(2000.0, 0.0);
            }
            group.stopped = true;
            // Contact ended with the body 500 short of the goal.
            let body = walkers(&[(1, 1500.0, -32.0), (2, 1500.0, 32.0)], 100.0);
            group.stopped = false;
            group.reanchor(form::centroid(body.iter().map(|w| w.pos)).unwrap());
            assert!(!group.at_end() && !group.arrived, "the route is taken up again (march {march})");
            let slots = group.tick(3, &body, 64.0, &pacing).clone();
            let slot_x = slots[&UnitId(1)].x;
            if march {
                assert!(slot_x > 1500.0 && slot_x < 1700.0, "the anchor walks on from the body: {slot_x}");
            } else {
                assert!((slot_x - 2000.0).abs() < 1.0, "the slots stand at the destination again: {slot_x}");
            }
        }
    }

    #[test]
    fn a_joiner_is_an_exception_until_it_first_reaches_a_slot() {
        let pacing = Pacing::default();
        let mut w = walkers(&[(1, 0.0, -32.0), (2, 0.0, 32.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Go, vec![at(3000.0, 0.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.tick(0, &w, 64.0, &pacing);
        march(&mut group, &mut w, 60, 3, &pacing);
        group.update(vec![UnitId(1), UnitId(2), UnitId(3)], group.order.clone());
        w.push(Walker { id: UnitId(3), pos: at(-900.0, 0.0), speed: 100.0, moving: 100.0, fleeing: false });
        group.tick(63, &w, 64.0, &pacing);
        let state = group.state(&w);
        assert_eq!(state.exceptions.len(), 1);
        assert_eq!(state.exceptions[0].0, UnitId(3));
        assert_eq!(state.exceptions[0].1, Cause::Joining);
        assert!(state.exceptions[0].2 > 8.0, "seconds to its slot at its speed");
        assert!((group.pace - group.full_pace).abs() < 0.01, "a joiner does not hold the anchor");
    }
}
