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
//! heading, filled from the front, narrowed to the files the ground's room allows (design D17); the contact arc is the lane's, `lib.rs`; the slots re-dealt by least travel with
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
//! for, the host takes out of the group: that is the host's decision, from the state's words.
//!
//! **The ground is the group's** (design D17, 2026-10-25, the user: "Since we are replacing a direct a->b order that
//! the engine would normally handle pathing for, we have to be responsible for running the pathing"): the lane hands
//! the group a `Ground` (the cells every member's class stands on, the room about each, the way priced by room:
//! `ground.rs`) and the group plans its way between the host's points over it (`plan`), snaps every slot to it, and
//! names a member whose slot it cannot walk to an exception (`Cause::CutOff`, which holds the anchor by nothing); a
//! set-down after contact or at a turn puts the anchor on the way, never at a centroid (a body split by a cliff has
//! its centroid on the cliff: the assault's detour arm stood forty seconds at a cliff's lip with its slots on the far
//! side, then set its Stouts' slots at the cliff's foot); the files narrow to the room at the anchor's cell and widen
//! after. Without a ground (the host sent no terrain; the unit tests) the way is the host's points and nothing is
//! snapped, as before D17. Pure over positions; the lane decides what each slot becomes as an order.

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{UnitId, Vec3};

use crate::form;
use crate::ground::Ground;

/// `WITHIN_REASON_GROUP_DEBUG=1`: the group says on stderr when it re-deals, turns, plans or its slots jump.
pub(crate) fn debug_on() -> bool {
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

/// A hunt's terms (H-MICRO-HUNT): the quarry, how far any hunter may go from where it stood when the lane first ran
/// it under this hunt, how long the quarry may be out of sight and radar before the hunt ends, and the frame the host
/// began it (so a hunt picked again on the same quarry after an end is a new order, not the silent re-declaration).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntOrder {
    pub quarry: UnitId,
    pub leash: f32,
    pub lost_frames: i32,
    pub since: i32,
}

/// The host's order to a group: the kind, the route to walk, the goal its last point (one point for a straight line;
/// the lane plans the way between the points over the group's ground, design D17), and whether to march it: `march`
/// false puts the slots at the route's end at once and every member walks there at its own speed (the duel's `ranks`
/// arm, the march's end state without the pacing).
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
    /// Its slot is on no ground it can stand on near, or its walk to it is far longer than the straight line (the
    /// ground between them does not join: a cliff, a lake); holds the anchor by nothing (design D17).
    CutOff,
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
    /// The host's route points still ahead of the anchor (0 at the end: "two corners to go").
    pub legs_left: usize,
    /// The contact target the group stands on, when engaged (the lane's arc).
    pub engaged: Option<UnitId>,
    /// The files the ranks are laid in this tick (`form::FILES` in the open, fewer in a corridor: design D17).
    pub files: usize,
    /// The room about the anchor's cell on the group's ground, in elmos; `None` without a ground.
    pub room: Option<f32>,
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
/// An Attack's end that has moved this far since the way was planned has the way planned again (the field is built
/// from the end, so a smaller move is followed without a plan).
pub const REPLAN_MOVED: f32 = 128.0;
/// A heading change at a route point of this many degrees or more re-deals the slots with the anchor set down on the
/// body (D14); a smaller bend snaps the slots about the anchor and the deal stays. Twenty: a four-rank body's rear
/// corners stand 370 from the anchor, so a snap of 23 degrees moved them 146 and put 17 of 20 out of the band
/// (the circuit9 trace), while a re-deal that keeps the trail costs the body nothing.
pub const REDEAL_TURN_DEG: f32 = 20.0;
/// A body set down after contact further than this from its way is not put back on it: the way is planned again
/// from where the body stands (a chase that ended far off the route).
pub const REJOIN_WAY: f32 = 400.0;
/// A member whose walk to its slot is more than this many times the straight line is cut off (design D17), and is
/// formed again when its walk is within `CUT_OFF_REJOIN` times (hysteresis, so the report never flickers).
pub const CUT_OFF_FACTOR: f32 = 2.0;
pub const CUT_OFF_REJOIN: f32 = 1.5;
/// The walk is only searched for a member within this of its slot (a member further away is an exception already,
/// and the search's cost grows with the distance); its last verdict stands beyond it.
const CUT_OFF_CHECK: f32 = 600.0;
/// Frames between a member's walk searches (twice a second).
const CUT_OFF_CADENCE: i32 = 15;

impl Default for Pacing {
    fn default() -> Pacing {
        Pacing { fraction: 1.0, cadence: 3, formed_within: 2.0 * form::HULL_SPACING, drag_from: 2.0 * form::HULL_SPACING, exception_at: 300.0, rejoin_within: 1.5 * form::HULL_SPACING }
    }
}

/// The room a group's way is priced for: the full ranks' half-width at this spacing plus a hull (D13: 224 at the
/// hull spacing).
pub fn wanted_room(spacing: f32) -> f32 {
    (form::FILES as f32 - 1.0) / 2.0 * spacing + form::HULL_SPACING
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

/// The point of the segment `a`-`b` nearest `p`, in the ground plane.
fn project(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let len2 = dx * dx + dz * dz;
    let t = if len2 <= 0.0 { 0.0 } else { (((p.x - a.x) * dx + (p.z - a.z) * dz) / len2).clamp(0.0, 1.0) };
    Vec3 { x: a.x + t * dx, y: p.y, z: a.z + t * dz }
}

/// One declared group.
#[derive(Clone, Debug)]
pub struct Group {
    pub name: String,
    pub members: Vec<UnitId>,
    pub order: GroupOrder,
    pub anchor: Vec3,
    pub heading: (f32, f32),
    /// The way the anchor walks: the host's route planned over the ground (design D17), the host's points among its
    /// points (`way_host`), from where the anchor stood when it was planned (`way_from`).
    way: Vec<Vec3>,
    way_host: Vec<bool>,
    way_from: Vec3,
    /// The route's end the way was planned to (an Attack's end moving `REPLAN_MOVED` plans again).
    planned_end: Option<Vec3>,
    /// The way is to be planned again before the next step.
    dirty: bool,
    /// The way point the anchor walks to next; `way.len()` at the end.
    pub leg: usize,
    /// The ground the lane gave this group, if the host has terrain.
    ground: Option<Arc<Ground>>,
    /// The files the ranks are laid in: `form::FILES`, fewer where the room is less.
    files: usize,
    /// The room about the anchor's cell, when a ground is known.
    room: Option<f32>,
    /// Each member's slot index, sticky until the membership or the route changes, the anchor turns `REDEAL_TURN_DEG`
    /// at a route point, the files change or the anchor is set down after contact.
    slot_of: HashMap<UnitId, usize>,
    /// Members out of their slot beyond the exception threshold (away), with the frame it happened.
    away: HashMap<UnitId, i32>,
    /// Members whose slot the ground does not join to them (cut off), with the frame it happened, and the frame each
    /// member's walk was last searched.
    cut_off: HashMap<UnitId, i32>,
    cut_checked: HashMap<UnitId, i32>,
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
    /// Each member's slot this tick, snapped to the ground.
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
        let way = order.route.clone();
        let mut group = Group {
            name: name.to_string(),
            members,
            way_host: vec![true; way.len()],
            way,
            way_from: centroid,
            planned_end: order.route.last().copied(),
            dirty: true,
            order,
            anchor: centroid,
            heading,
            leg: 0,
            ground: None,
            files: form::FILES,
            room: None,
            slot_of: HashMap::new(),
            away: HashMap::new(),
            cut_off: HashMap::new(),
            cut_checked: HashMap::new(),
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
        group.way_from = group.anchor;
        group
    }

    /// The ground the lane built for this group's classes and room (design D17); `None` without terrain. A new ground
    /// plans the way again.
    pub fn set_ground(&mut self, ground: Option<Arc<Ground>>) {
        self.ground = ground;
        self.dirty = true;
    }

    /// The group's ground nearest `p` (for a contact slot); `p` without a ground, or with none near.
    pub fn snap(&self, p: Vec3) -> Vec3 {
        self.ground.as_ref().and_then(|g| g.snap(p)).unwrap_or(p)
    }

    /// The way the anchor walks this order by, as planned (the tests' instrument).
    pub fn way(&self) -> &[Vec3] {
        &self.way
    }

    /// The host's order again: a new kind, a route of another length or a point moved `ROUTE_MOVED` re-aims the
    /// anchor from where it stands (no jump), plans the way again and re-deals the slots by least travel (a reverse:
    /// nobody crosses the formation); a point moved less is taken silently. A new membership re-deals the slots too.
    pub fn update(&mut self, members: Vec<UnitId>, order: GroupOrder) {
        let moved = order.route.len() != self.order.route.len() || order.route.iter().zip(&self.order.route).any(|(a, b)| a.dist2d(*b) > ROUTE_MOVED);
        if order.kind != self.order.kind || order.march != self.order.march || moved {
            self.order = order;
            self.leg = 0;
            self.arrived = false;
            self.slot_of.clear();
            self.dirty = true;
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

    /// The way from where the anchor stands through the host's points (design D17): each leg the ground's way (the
    /// priced field from the leg's point, descended, thinned and straightened), the host's point ending it; the
    /// host's points themselves without a ground, or with the march off (the slots stand at the end regardless).
    fn plan(&mut self) {
        let started = std::time::Instant::now();
        self.way_from = self.anchor;
        self.way.clear();
        self.way_host.clear();
        match &self.ground {
            Some(ground) if self.order.march => {
                let mut from = self.anchor;
                let last = self.order.route.len().saturating_sub(1);
                for (i, &point) in self.order.route.iter().enumerate() {
                    // The route's end is where the body stands (D18: off a structure, with its room); the points
                    // between are passed through.
                    let point = if i == last { ground.stand(point).unwrap_or(point) } else { point };
                    let leg = ground.way(from, point);
                    let n = leg.len();
                    for (i, q) in leg.into_iter().enumerate() {
                        self.way.push(q);
                        self.way_host.push(i + 1 == n);
                    }
                    from = self.way.last().copied().unwrap_or(point);
                }
            }
            _ => {
                self.way = self.order.route.clone();
                self.way_host = vec![true; self.way.len()];
            }
        }
        self.planned_end = self.order.route.last().copied();
        self.leg = 0;
        self.dirty = false;
        if debug_on() {
            eprintln!(
                "[group {}] PLAN from ({:.0},{:.0}): {} host points, {} way points{} in {:.1} ms",
                self.name,
                self.anchor.x,
                self.anchor.z,
                self.order.route.len(),
                self.way.len(),
                if self.ground.is_some() { "" } else { " (no ground)" },
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    /// The anchor that puts the centroid of `count` ranks' slots at `centroid` facing `heading`.
    fn anchor_for(&self, centroid: Vec3, heading: (f32, f32), count: usize) -> Vec3 {
        let at_origin = form::slots(Vec3::default(), heading, count.max(1), self.spacing, self.files);
        let c0 = form::centroid(at_origin.iter().copied()).unwrap_or_default();
        Vec3 { x: centroid.x - c0.x, y: centroid.y, z: centroid.z - c0.z }
    }

    /// The way as a polyline: from where it was planned through its points.
    fn polyline(&self) -> Vec<Vec3> {
        std::iter::once(self.way_from).chain(self.way.iter().copied()).collect()
    }

    /// The members in the body: not fleeing, not away, not cut off, and in a slot once.
    fn body(&self, walkers: &[Walker]) -> Vec<Vec3> {
        walkers
            .iter()
            .filter(|w| !w.fleeing && !self.away.contains_key(&w.id) && !self.cut_off.contains_key(&w.id) && self.formed_once.get(&w.id).copied().unwrap_or(false))
            .map(|w| w.pos)
            .collect()
    }

    /// The anchor set down again after contact (where the body stands is where the march resumes from): on the body
    /// facing along the way's nearest segment, and onto that segment when it stands off it (design D17, `on_way`: a
    /// body split by a cliff has its centroid on the cliff), the leg that segment's end; the slots re-dealt where the
    /// members stand (the arc's places are not the march's). A way the
    /// anchor had finished is taken up again when the body is set down short of its end (assault2-mix12-hlt2llt2-
    /// around-ranks: with the march off the anchor stands at the end from the first tick, so after a contact that
    /// ended with the last turret out of sight the body was set down 470 short of the goal, every member formed
    /// round it, "arrived", and six of eight duels timed out there). A body further than `REJOIN_WAY` from the way
    /// (a chase that ended far off the route) has the way planned again from where it stands.
    pub fn reanchor(&mut self, walkers: &[Walker]) {
        let body = self.body(walkers);
        let (centroid, count) = match form::centroid(body.iter().copied()) {
            Some(c) => (c, body.len()),
            None => match form::centroid(walkers.iter().map(|w| w.pos)) {
                Some(c) => (c, walkers.len()),
                None => return,
            },
        };
        self.slot_of.clear();
        let points = self.polyline();
        if points.len() < 2 {
            self.anchor = self.anchor_for(centroid, self.heading, count);
            return;
        }
        let (k, q) = (0..points.len() - 1)
            .map(|k| (k, project(centroid, points[k], points[k + 1])))
            .min_by(|a, b| a.1.dist2d(centroid).total_cmp(&b.1.dist2d(centroid)))
            .expect("two points or more");
        if centroid.dist2d(q) > REJOIN_WAY {
            self.anchor = self.anchor_for(self.snap(centroid), self.heading, count);
            self.arrived = false;
            self.dirty = true;
            return;
        }
        if points[k].dist2d(points[k + 1]) >= 1.0 {
            self.heading = form::heading(points[k], points[k + 1]);
        }
        self.anchor = self.on_way(self.anchor_for(centroid, self.heading, count), points[k], points[k + 1]);
        let end = points[points.len() - 1];
        self.leg = if k + 1 == self.way.len() && self.anchor.dist2d(end) <= ROUTE_MOVED { self.way.len() } else { k };
        self.arrived = false;
    }

    /// An anchor set down on the body, put on the way's segment when it stands off it by more than the ranks' own
    /// asymmetry (the offset rank puts a formed body's anchor half a spacing beside the way's line, and that anchor
    /// is right: D14's about-face is exact on it); further off, the body is split or beside the way and the anchor
    /// goes onto it (design D17).
    fn on_way(&self, anchor: Vec3, a: Vec3, b: Vec3) -> Vec3 {
        let q = project(anchor, a, b);
        if anchor.dist2d(q) > self.spacing { q } else { anchor }
    }

    /// The body's depth behind the anchor along the heading: the mean of its members' (in the body, not away, not
    /// fleeing), or the layout's half depth for the members when nobody is in the body yet. The carry at a corner
    /// (`target`) is measured, so the trail the members keep behind their slots (about fifty at pace, the brake zone)
    /// and the ranks they actually hold are in it: from the rank count alone the body's centroid was still a hundred
    /// short of the corner when the anchor turned (the circuit7-carry trace).
    fn body_depth(&self, walkers: &[Walker]) -> f32 {
        let h = self.heading;
        let depths: Vec<f32> = self.body(walkers).iter().map(|p| (self.anchor.x - p.x) * h.0 + (self.anchor.z - p.z) * h.1).collect();
        if depths.is_empty() {
            return (self.members.len().max(1).div_ceil(self.files) - 1) as f32 * form::RANK_GAP / 2.0;
        }
        (depths.iter().sum::<f32>() / depths.len() as f32).max(0.0)
    }

    /// The point the anchor walks to for `leg`: the way's point, carried on along its leg by `carry` (the body's
    /// depth behind the anchor) when another point follows (at most half way to the next). The way is the body's
    /// centre line and the anchor is its front: an anchor that turned at the point would leave the body's centroid
    /// half its depth short of the corner, so the ranks set down over the body at the turn (D14) would lie that far
    /// inside the new leg, the inside file in the wall the way kept its room from, and the next point, 256 on, nearly
    /// abreast of the anchor, which swung the heading a second time a tick later (the first re-deal runs: 60-85
    /// degrees, the body knocked out of slot again). Carried on, the body's centroid turns at the point and the new
    /// ranks lie on the new leg. The carry stops at the ground's edge (design D18): the way bends where the ground
    /// bends it, so a bend's incoming leg points at the wall the way turned for, and the carried target is walked from
    /// the point in half-cell steps only as far as the body's ground goes (an anchor on a cliff hung every slot off
    /// it). Not at the room's edge: tried first, that cut the carry to nothing at every bend by a wall, the anchor
    /// turned with the body's centroid short of the corner (D14's first fault again), and the set-down's re-deal cost
    /// the body a second it never recovered at a pace fraction of one (circuit15: 202 s against 191, formed 0.72
    /// against 0.93, the trail behind the anchor 120 against 60 for whole legs). The room is read for the files only
    /// off the carry (`carrying`).
    fn target(&self, leg: usize, carry: f32) -> Vec3 {
        let point = self.way[leg];
        let Some(next) = self.way.get(leg + 1) else { return point };
        if carry <= 0.0 {
            return point;
        }
        let from = if leg == 0 { self.way_from } else { self.way[leg - 1] };
        if from.dist2d(point) < 1.0 {
            return point;
        }
        let u = form::heading(from, point);
        let carry = carry.min(point.dist2d(*next) / 2.0);
        let along = |d: f32| Vec3 { x: point.x + u.0 * d, y: point.y, z: point.z + u.1 * d };
        let Some(ground) = &self.ground else { return along(carry) };
        if !ground.passable_at(point) {
            return along(carry);
        }
        let step = ground.cell() / 2.0;
        let mut fits = 0.0f32;
        let mut d = step;
        while d <= carry {
            if !ground.passable_at(along(d)) {
                return along(fits);
            }
            fits = d;
            d += step;
        }
        if ground.passable_at(along(carry)) { along(carry) } else { along(fits) }
    }

    /// Whether the anchor is on a carry: past the current leg's point along the leg, walking on to the carried target
    /// while the body comes up to the point (design D18). The anchor is then not where the body is, and the room at
    /// its cell says nothing about the body's: read there, the room of whatever lay beyond the fourth corner's bend
    /// collapsed the files to two a few seconds before every such turn (circuit14: 187 to 91; D17's first circuit,
    /// worse, before its files fix).
    fn carrying(&self) -> bool {
        if self.leg >= self.way.len() {
            return false;
        }
        let point = self.way[self.leg];
        let from = if self.leg == 0 { self.way_from } else { self.way[self.leg - 1] };
        if from.dist2d(point) < 1.0 {
            return false;
        }
        let u = form::heading(from, point);
        (self.anchor.x - point.x) * u.0 + (self.anchor.z - point.z) * u.1 > 0.0
    }

    /// The anchor set down at a turn so the body's ranks lie over the body (design D14: at a turn the slots are
    /// re-dealt where the members stand, so the turn is a re-labelling and an about-face costs nobody a step), put
    /// onto the new leg when it stands off it (D17, `on_way`: a split body's centroid is not a place). The
    /// body is the members in it, and the slot count is the body's: with the joiners' slots counted the anchor would
    /// stand a rank ahead of the body. On the members themselves, not the trail they walked behind their slots
    /// ahead of them: set down there everyone is formed at once and the body walks off at full pace, the trail
    /// re-forming in a few seconds without drag (circuit9b, a 126-degree corner: pace 70 to 87, 20 formed
    /// throughout); set down the trail ahead, the trail added to the re-deal's travel and a 130-degree corner put 19
    /// of 20 out of the band (circuit10: pace 21, formed 1). A body of nobody leaves the anchor where it is.
    fn settle_on_body(&mut self, walkers: &[Walker]) {
        let body = self.body(walkers);
        let Some(centroid) = form::centroid(body.iter().copied()) else { return };
        let anchor = self.anchor_for(centroid, self.heading, body.len());
        let points = self.polyline();
        self.anchor = if points.len() >= 2 {
            let k = self.leg.min(points.len() - 2);
            self.on_way(anchor, points[k], points[k + 1])
        } else {
            anchor
        };
    }

    /// An Attack's goal follows its target: the route's end moves to `at`; an anchor that had arrived walks on when
    /// the target has moved `ROUTE_MOVED`; the way is planned again when it has moved `REPLAN_MOVED` since the plan.
    pub fn retarget(&mut self, at: Vec3) {
        let Some(end) = self.order.route.last_mut() else { return };
        let far = end.dist2d(at) > ROUTE_MOVED;
        *end = at;
        if let Some(last) = self.way.last_mut() {
            *last = at;
        }
        if self.planned_end.is_some_and(|p| p.dist2d(at) > REPLAN_MOVED) {
            self.dirty = true;
        }
        if far && self.leg >= self.way.len() {
            self.leg = self.way.len().saturating_sub(1);
            self.arrived = false;
        }
    }

    /// Whether the anchor is at the way's end.
    pub fn at_end(&self) -> bool {
        self.leg >= self.way.len()
    }

    /// The anchor walks `distance` along the way, leg by leg, each intermediate point carried on by `carry`.
    fn advance(&mut self, distance: f32, carry: f32) {
        let mut left = distance;
        while left > 0.0 && self.leg < self.way.len() {
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

    /// The files the ranks are laid in for the room at the anchor's cell (design D17): the half-width that fits with
    /// half a hull to spare, down to one file; widened only when the room allows the next file with half a spacing
    /// to spare (hysteresis). A change re-deals the slots and moves the anchor not at all: the front keeps its place
    /// and the extra ranks file in behind it (re-placed to keep the ranks' centroid, a collapse to one file threw the
    /// anchor 768 forward onto a cliff: circuit13). An anchor with no ground within the snap distance (the target
    /// carried past a bend can lie on a cliff for a few ticks) says nothing about the room and keeps the files, and so
    /// does an anchor on a carry (D18, `carrying`).
    fn fit_files(&mut self, spacing: f32) {
        let Some(ground) = &self.ground else { return };
        // On a carry the anchor is not where the body is (design D18).
        if self.carrying() {
            return;
        }
        let Some(on_ground) = ground.snap(self.anchor) else { return };
        let room = ground.room_at(on_ground);
        self.room = Some(room);
        let fits = |room: f32| ((((room - form::HULL_SPACING / 2.0).max(0.0)) * 2.0 / spacing).floor() as usize + 1).clamp(1, form::FILES);
        let now = fits(room);
        let wider = fits(room - spacing / 2.0);
        let files = if now < self.files {
            now
        } else if wider > self.files {
            wider
        } else {
            self.files
        };
        if files == self.files {
            return;
        }
        let before = self.files;
        self.files = files;
        self.slot_of.clear();
        if debug_on() {
            eprintln!("[group {}] FILES {before} -> {files} with {room:.0} of room at ({:.0},{:.0})", self.name, self.anchor.x, self.anchor.z);
        }
    }

    /// One tick: the way planned if it must be, the pace from the walkers' types, the anchor held back by its formed
    /// members and advanced along the way (unless stopped, or the order's `march` is off, when it stands at the
    /// route's end), the files fitted to the room, the slots dealt and snapped to the ground, formed and exceptions
    /// told apart. Returns each walker's slot. `spacing` is the lane's for this tick.
    pub fn tick(&mut self, frame: i32, walkers: &[Walker], spacing: f32, pacing: &Pacing) -> &HashMap<UnitId, Vec3> {
        if self.dirty {
            self.plan();
        }
        let march = self.order.march;
        let dt = self.last_frame.map_or(0.0, |f| (frame - f).max(0) as f32 / 30.0);
        self.last_frame = Some(frame);
        self.spacing = spacing;
        let carry = self.body_depth(walkers);
        let slowest = walkers.iter().map(|w| w.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        self.full_pace = if slowest.is_finite() { slowest * pacing.fraction } else { 0.0 };
        if !march && !self.at_end() {
            // No pacing: the slots stand at the destination and every unit walks there at its own speed.
            if let Some(end) = self.way.last().copied() {
                if end.dist2d(self.anchor) >= 1.0 {
                    self.heading = form::heading(self.anchor, end);
                }
                self.anchor = Vec3 { y: end.y, ..end };
            }
            self.leg = self.way.len();
        } else if self.leg < self.way.len() {
            let next = self.target(self.leg, carry);
            // (`form::heading` says east for two points under 1.0 apart: a Hold at the body's own place spun the
            // formation east at the route's end before every guard was 1.0.)
            if next.dist2d(self.anchor) >= 1.0 {
                self.heading = form::heading(self.anchor, next);
            }
        }
        // Who is where against last tick's slots: the drag on the anchor from the body's members behind theirs,
        // however far (the anchor never outruns its body); a joiner is in the body once it has stood in its slot; a
        // cut-off member holds nothing (it cannot close by walking).
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
            if *in_body && !w.fleeing && !self.cut_off.contains_key(&w.id) {
                let behind = (slot.x - w.pos.x) * h.0 + (slot.z - w.pos.z) * h.1;
                max_lag = max_lag.max(behind);
            }
        }
        let held = if max_lag <= pacing.drag_from { 1.0 } else { ((pacing.exception_at - max_lag) / (pacing.exception_at - pacing.drag_from)).clamp(0.0, 1.0) };
        self.pace = if self.stopped || self.at_end() || !march { 0.0 } else { self.full_pace * held };
        if self.pace > 0.0 {
            let (heading_before, leg_before) = (self.heading, self.leg);
            self.advance(self.pace * dt, carry);
            // A way point passed: facing the next, and at a turn of `REDEAL_TURN_DEG` or more the anchor is set
            // down on the body and the slots re-dealt where the members stand (D14).
            if self.leg > leg_before && self.leg < self.way.len() {
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
                        let body = self.body(walkers).len();
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
        // The shape to the room (D17), on the march.
        if march && !self.stopped && !self.at_end() {
            self.fit_files(spacing);
        }
        // The slots, sticky by index; dealt by least travel when the deal is stale.
        let ids: Vec<UnitId> = walkers.iter().map(|w| w.id).collect();
        let n = ids.len();
        let slots = form::slots(self.anchor, self.heading, n.max(1), spacing, self.files);
        let stale = n > 0 && (self.slot_of.len() != n || ids.iter().any(|id| !self.slot_of.contains_key(id)));
        if stale {
            let positions: Vec<Vec3> = walkers.iter().map(|w| w.pos).collect();
            let deal = form::assign(&positions, &slots[..n]);
            self.slot_of = ids.iter().copied().zip(deal).collect();
        }
        let previous = std::mem::take(&mut self.slots);
        self.slots = ids.iter().map(|id| (*id, slots[self.slot_of.get(id).copied().unwrap_or(0).min(n.saturating_sub(1))])).collect();
        // Snapped to the ground, and cut off where the ground does not join a member to its slot (D17): no standable
        // cell near the slot, or a walk to it (searched twice a second, within `CUT_OFF_CHECK`) longer than
        // `CUT_OFF_FACTOR` times the straight line; formed again within `CUT_OFF_REJOIN` times.
        if let Some(ground) = &self.ground {
            for w in walkers {
                let raw = self.slots[&w.id];
                let was = self.cut_off.contains_key(&w.id);
                let (slot, cut) = match ground.snap(raw) {
                    None => (raw, true),
                    Some(s) => {
                        let d = w.pos.dist2d(s);
                        let due = self.cut_checked.get(&w.id).is_none_or(|f| frame - *f >= CUT_OFF_CADENCE);
                        let cut = if !due || d > CUT_OFF_CHECK {
                            was
                        } else {
                            self.cut_checked.insert(w.id, frame);
                            let cap = d * if was { CUT_OFF_REJOIN } else { CUT_OFF_FACTOR } + 2.0 * ground.cell();
                            !ground.walk_within(w.pos, s, cap)
                        };
                        (s, cut)
                    }
                };
                if cut {
                    self.cut_off.entry(w.id).or_insert(frame);
                } else {
                    self.cut_off.remove(&w.id);
                }
                self.slots.insert(w.id, slot);
            }
        }
        self.cut_off.retain(|id, _| ids.contains(id));
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
                    "[group {} f={frame}] {} n={n} files={} anchor=({:.0},{:.0}) heading=({:.2},{:.2}) leg={} carry={carry:.0} pace={:.0}; {unsettled} formed members' slots moved, the furthest {worst:.0} (the anchor's step {step:.0})",
                    self.name,
                    if stale { "RE-DEAL" } else { "SLOTS JUMPED" },
                    self.files,
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
            let cut_off = self.cut_off.contains_key(&w.id);
            let cause = if formed && !self.away.contains_key(&w.id) && !cut_off {
                None
            } else if w.fleeing {
                Some(Cause::Fled)
            } else if !*formed_once {
                Some(Cause::Joining)
            } else if cut_off {
                Some(Cause::CutOff)
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
                if !self.away.contains_key(&w.id) && !cut_off {
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
            legs_left: self.way_host.iter().skip(self.leg).filter(|host| **host).count(),
            engaged: self.engaged,
            files: self.files,
            room: self.room,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::Classes;
    use bot_protocol::{MoveClass, MoveKind, Terrain};

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

    /// A flat 128 x 64 map at 16-elmo cells (2,048 x 1,024 elmos) with a cliff wall at x 960-1,024 from the north
    /// edge down to z 768: the way round it goes south of its end.
    fn walled() -> Terrain {
        let (w, h) = (128usize, 64usize);
        let mut slopes = vec![0u8; w * h];
        for z in 0..48 {
            for x in 60..64 {
                slopes[z * w + x] = 255;
            }
        }
        Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes, metal: Vec::new() }
    }

    /// A corridor 192 wide (z 416-608) through a wall at x 960-1,024 on the same map.
    fn corridor() -> Terrain {
        let (w, h) = (128usize, 64usize);
        let mut slopes = vec![0u8; w * h];
        for z in 0..h {
            if (26..38).contains(&z) {
                continue;
            }
            for x in 60..64 {
                slopes[z * w + x] = 255;
            }
        }
        Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes, metal: Vec::new() }
    }

    fn ground_of(terrain: &Terrain) -> Arc<Ground> {
        let tank = MoveClass { kind: MoveKind::Tank, max_slope: 0.3, depth: 20.0, slope_mod: 0.0 };
        let mut classes = Classes::default();
        Arc::new(Ground::new(terrain, &[classes.passable(terrain, tank)], wanted_room(64.0)).unwrap())
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
        assert_eq!((state.files, state.room), (form::FILES, None));
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
        // East through (100, 0) to (300, 0), then south: a right angle at the second point. Two ranks: half the
        // depth is 48.
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
        assert_eq!(group.state(&w).legs_left, 1);
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
    /// The anchor is set down on the way (D17), the leg the segment the body stands beside.
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
            // Contact ended with the body 500 short of the goal, and 100 to one side of the way.
            let body = walkers(&[(1, 1500.0, 68.0), (2, 1500.0, 132.0)], 100.0);
            group.stopped = false;
            group.reanchor(&body);
            assert!(!group.at_end() && !group.arrived, "the route is taken up again (march {march})");
            assert!(group.anchor.z.abs() < 1.0, "set down on the way, not at the body's centroid: {:?}", group.anchor);
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

    /// D17: with a ground the one-point order to the far side of a cliff is walked round it: the way bends south of
    /// the wall's end, every way point stands, and the anchor follows the way rather than the crow line.
    #[test]
    fn with_a_ground_a_one_point_order_is_walked_round_the_cliff() {
        let pacing = Pacing::default();
        let terrain = walled();
        let mut w = two_ranks_of_six();
        for u in w.iter_mut() {
            u.pos = at(u.pos.x + 400.0, u.pos.z + 300.0);
        }
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        let mut group = Group::new("a", members, GroupOrder::new(GroupKind::Go, vec![at(1600.0, 300.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.set_ground(Some(ground_of(&terrain)));
        group.tick(0, &w, 64.0, &pacing);
        assert!(group.way().len() >= 2, "the way bends: {:?}", group.way());
        assert!(group.way().iter().any(|p| p.z > 768.0), "round the south end: {:?}", group.way());
        assert_eq!(group.state(&w).legs_left, 1, "one host point ahead whatever the way's bends");
        // Walk: the anchor never crosses the wall's cells.
        let ground = ground_of(&terrain);
        march(&mut group, &mut w, 900, 3, &pacing);
        assert!(group.anchor.x > 1024.0 || group.anchor.z > 768.0, "past or under the wall after thirty seconds: {:?}", group.anchor);
        assert!(ground.passable_at(group.snap(group.anchor)), "the anchor stands on the ground: {:?}", group.anchor);
    }

    /// D17: a member whose slot lies across the wall from it is cut off (its walk is many times the straight line) and
    /// holds the anchor by nothing; a member whose slot is on the wall gets the slot snapped beside it.
    #[test]
    fn a_route_ending_on_a_structure_ends_where_the_body_has_room() {
        // A flat map with a factory of 80 x 64 at (1,200, 500): a Go to the factory's centre plans its end off it with
        // the room the ranks want, not against its wall with room for one file.
        let (w, h) = (128usize, 64usize);
        let terrain = Terrain { cell: 16.0, width: w as u32, height: h as u32, heights: vec![100; w * h], slopes: vec![0u8; w * h], metal: Vec::new() };
        let tank = MoveClass { kind: MoveKind::Tank, max_slope: 0.3, depth: 20.0, slope_mod: 0.0 };
        let mut classes = Classes::default();
        let (blocked, _) = crate::ground::structures(&terrain, &[crate::ground::Footprint::of(at(1200.0, 500.0), (10, 8), 0)]);
        let ground = Arc::new(Ground::new(&terrain, &[classes.passable(&terrain, tank), blocked], wanted_room(64.0)).unwrap());
        let mut group = Group::new("a", vec![UnitId(1)], GroupOrder::new(GroupKind::Go, vec![at(1200.0, 500.0)]), &[at(200.0, 500.0)], 64.0);
        group.set_ground(Some(ground.clone()));
        group.plan();
        let end = *group.way.last().unwrap();
        assert!(ground.room_at(end) >= wanted_room(64.0), "the end has the body's room: {end:?} with {:.0}", ground.room_at(end));
        assert!(end.dist2d(at(1200.0, 500.0)) < 2.0 * wanted_room(64.0) + 64.0, "and stands near the factory: {end:?}");
        assert_eq!(group.state(&walkers(&[(1, 200.0, 500.0)], 100.0)).legs_left, 1);
    }

    #[test]
    fn a_member_across_the_cliff_from_its_slot_is_cut_off_and_holds_nothing() {
        let pacing = Pacing::default();
        let terrain = walled();
        // Two members facing east toward a goal east of the wall; the body stands just west of it.
        let mut w = walkers(&[(1, 900.0, 268.0), (2, 900.0, 332.0)], 100.0);
        let mut group = Group::new("a", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Go, vec![at(1800.0, 300.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.set_ground(Some(ground_of(&terrain)));
        group.tick(0, &w, 64.0, &pacing);
        group.tick(3, &w, 64.0, &pacing);
        assert!(group.state(&w).exceptions.is_empty(), "{:?}", group.state(&w));
        // The anchor is put across the wall with its slots there (as a crow-line route would have it), and unit 1
        // is left on the west side, 120 from its slot in a straight line and a long way round.
        group.anchor = at(1100.0, 300.0);
        group.heading = (1.0, 0.0);
        group.slot_of.clear();
        let slots = group.tick(6, &w, 64.0, &pacing).clone();
        w[1].pos = slots[&UnitId(2)];
        w[0].pos = at(940.0, slots[&UnitId(1)].z);
        group.tick(21, &w, 64.0, &pacing);
        let state = group.state(&w);
        assert_eq!(state.exceptions.len(), 1, "{state:?}");
        assert_eq!((state.exceptions[0].0, state.exceptions[0].1), (UnitId(1), Cause::CutOff));
        assert!((group.pace - group.full_pace).abs() < 0.01, "a cut-off member holds the anchor by nothing: {} of {}", group.pace, group.full_pace);
        // A slot on the wall itself snaps to the ground beside it.
        group.anchor = at(992.0, 300.0);
        group.slot_of.clear();
        let slots = group.tick(24, &w, 64.0, &pacing).clone();
        let ground = ground_of(&terrain);
        assert!(slots.values().all(|s| ground.passable_at(*s)), "{slots:?}");
    }

    /// D17: a change of files moves the anchor not at all, and an anchor off the ground (the carried target past a
    /// bend on a cliff) keeps its files instead of reading the room as none (circuit13: a collapse to one file threw
    /// the anchor 768 forward onto the cliff and cut every member off).
    #[test]
    fn the_carry_past_a_bend_stops_at_the_grounds_edge_and_runs_whole_in_the_open() {
        // A bend just south-west of the wall's end, its incoming leg pointing north-east into the wall: carried the
        // body's depth regardless the target would stand inside the wall (x 1,079, z 492); the line enters the wall
        // at x 960, 134 along.
        let (start, corner, end) = (at(700.0, 1250.0), at(900.0, 850.0), at(1700.0, 100.0));
        let mut group = Group::new("a", vec![UnitId(1)], GroupOrder::new(GroupKind::Go, vec![corner, end]), &[start], 64.0);
        group.spacing = 64.0;
        let whole = group.target(0, 400.0);
        assert!((whole.dist2d(corner) - 400.0).abs() < 0.5, "without a ground the whole carry: {whole:?}");
        let ground = ground_of(&walled());
        group.set_ground(Some(ground.clone()));
        let stopped = group.target(0, 400.0);
        let carried = stopped.dist2d(corner);
        assert!((100.0..134.0).contains(&carried), "the carry stopped at the wall: {carried:.0} ({stopped:?})");
        assert!(ground.passable_at(stopped));
        // The same bend far from any wall carries the whole depth.
        let (start, corner, end) = (at(200.0, 900.0), at(400.0, 700.0), at(800.0, 100.0));
        let mut open = Group::new("b", vec![UnitId(1)], GroupOrder::new(GroupKind::Go, vec![corner, end]), &[start], 64.0);
        open.spacing = 64.0;
        open.set_ground(Some(ground));
        let target = open.target(0, 200.0);
        assert!((target.dist2d(corner) - 200.0).abs() < 0.5, "in the open the whole carry: {target:?}");
    }

    #[test]
    fn the_files_hold_while_the_anchor_is_on_a_carry() {
        // The anchor past the way's point along the leg is carrying; short of it, or on the next leg, it is not; and
        // on a carry into the wall's room the files stay six.
        let (start, corner, end) = (at(300.0, 850.0), at(800.0, 850.0), at(800.0, 100.0));
        let mut group = Group::new("a", vec![UnitId(1)], GroupOrder::new(GroupKind::Go, vec![corner, end]), &[start], 64.0);
        group.spacing = 64.0;
        group.set_ground(Some(ground_of(&walled())));
        group.anchor = at(700.0, 850.0);
        assert!(!group.carrying());
        group.anchor = at(900.0, 850.0);
        assert!(group.carrying());
        group.fit_files(64.0);
        assert_eq!(group.files, form::FILES);
        group.leg = 1;
        group.anchor = at(800.0, 700.0);
        assert!(!group.carrying());
    }

    #[test]
    fn a_change_of_files_keeps_the_anchor_and_an_anchor_off_the_ground_keeps_its_files() {
        let pacing = Pacing::default();
        let terrain = corridor();
        let mut w = two_ranks_of_six();
        for u in w.iter_mut() {
            u.pos = at(u.pos.x + 800.0, u.pos.z + 512.0);
        }
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        let mut group = Group::new("a", members, GroupOrder::new(GroupKind::Go, vec![at(1700.0, 512.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.set_ground(Some(ground_of(&terrain)));
        group.tick(0, &w, 64.0, &pacing);
        // The anchor in the corridor's mouth, where the room is the corridor's: the files narrow, the anchor stays.
        group.anchor = at(968.0, 512.0);
        let before = group.anchor;
        group.tick(3, &w, 64.0, &pacing);
        assert!(group.state(&w).files <= 3, "{:?}", group.state(&w));
        assert!(group.anchor.dist2d(before) < 10.0, "the anchor walked one tick at most: {:?} from {before:?}", group.anchor);
        // The anchor off the ground with none within six cells (off the map here; the carried target past a bend on
        // a cliff in the game): the files stand.
        let files = group.state(&w).files;
        group.anchor = at(-500.0, 512.0);
        group.tick(6, &w, 64.0, &pacing);
        assert_eq!(group.state(&w).files, files);
    }

    /// D17: in a corridor 192 wide the ranks narrow to the files that fit (three at 64) and widen again in the open.
    #[test]
    fn the_files_narrow_to_the_corridor_and_widen_after() {
        let pacing = Pacing::default();
        let terrain = corridor();
        let mut w = two_ranks_of_six();
        for u in w.iter_mut() {
            u.pos = at(u.pos.x + 500.0, u.pos.z + 512.0);
        }
        let members: Vec<UnitId> = w.iter().map(|w| w.id).collect();
        let mut group = Group::new("a", members, GroupOrder::new(GroupKind::Go, vec![at(1700.0, 512.0)]), &w.iter().map(|w| w.pos).collect::<Vec<_>>(), 64.0);
        group.set_ground(Some(ground_of(&terrain)));
        group.tick(0, &w, 64.0, &pacing);
        assert_eq!(group.state(&w).files, form::FILES);
        let mut least = form::FILES;
        let mut frame = 0;
        while frame < 1200 {
            frame += 3;
            let slots = group.tick(frame, &w, 64.0, &pacing).clone();
            for u in w.iter_mut() {
                let s = slots[&u.id];
                u.pos = at(s.x - group.heading.0 * 20.0, s.z - group.heading.1 * 20.0);
            }
            let state = group.state(&w);
            if group.anchor.x > 940.0 && group.anchor.x < 1040.0 {
                least = least.min(state.files);
            }
            assert!(slots.values().filter(|s| s.x >= 960.0 && s.x < 1024.0).all(|s| (s.z - 512.0).abs() < 96.0 + 1.0), "between the walls every slot is within the corridor: {slots:?}");
            if group.at_end() {
                break;
            }
        }
        assert!(least <= 3, "narrowed in the corridor: {least} files");
        assert!(group.at_end(), "arrived: anchor {:?}", group.anchor);
        assert_eq!(group.state(&w).files, form::FILES, "widened again in the open");
    }
}
