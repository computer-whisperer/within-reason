//! What a host says to the lane about a group and what the lane says back
//! (`docs/design/2026-11-15-army-control-loop.md`). The host declares a group once per decision (`Lane::set_group`):
//! its members and one order. The lane's army (`army.rs`) decides where its companies stand and walk, and reports what
//! the units are doing. Until 2026-11-15 this file was the formation itself, an anchor walked along a route with nine
//! books a member; the design's §2 says why that went and §8 what went with it.

use bot_protocol::{UnitId, Vec3};

use crate::form;

/// `WITHIN_REASON_GROUP_DEBUG=1`: the army says its decisions on stderr (a re-form, an order, a way, a jump over a
/// tight stretch, a deal taken on gain).
pub(crate) fn debug_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WITHIN_REASON_GROUP_DEBUG").is_some_and(|v| v != "0" && !v.is_empty()))
}

/// What a group is to do. The first four walk to the order's place as an army of companies; a hunt and a rove have no
/// formation and are run raw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GroupKind {
    /// Walk there and stand; nothing on the way is fought (the hands' walk, a Move).
    Go,
    /// Fight to it: a company stops while every slot of it has an armed enemy in reach, and no nearer an enemy than
    /// the front's stop (the hands' advance).
    FightTo,
    /// Attack this unit: the place follows its sightings; otherwise an advance.
    Attack(UnitId),
    /// Stand at the place in formation and fight what comes.
    Hold,
    /// Every member attacks the quarry by id each tick until it is dead, lost or the leash is reached (H-MICRO-HUNT;
    /// `lib.rs` `hunt`). The route is ignored.
    Hunt(HuntOrder),
    /// Every member is a rover (H-MICRO-ROVE, `rove.rs`), within the region when one is given (north-west and
    /// south-east corners). The route is ignored.
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

/// The host's order to a group: the kind, the route (the place its last point; the points before it are passed
/// through), and whether to march it: `march` false sets every company on its place at once and each member walks
/// there at its own speed.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupOrder {
    pub kind: GroupKind,
    pub route: Vec<Vec3>,
    pub march: bool,
    /// Members a company holds at most; 0: no cap, one company.
    pub cap: usize,
    /// The enemy units an attack on a party is on: the place is kept on the member nearest the army, the host's route
    /// (the party's centre) only while none is seen. Empty: the route is the route.
    pub party: Vec<UnitId>,
    /// The group this one follows (the lane's name of it): its place is the row behind its ward's. None: the route is
    /// the route.
    pub ward: Option<String>,
    /// A roving group's hurt rovers go to a builder of ours to be healed (H-MICRO-HEAL); on by default.
    pub heal: bool,
}

impl GroupOrder {
    /// A marching order of this kind along this route.
    pub fn new(kind: GroupKind, route: Vec<Vec3>) -> GroupOrder {
        GroupOrder { kind, route, march: true, cap: 0, party: Vec::new(), ward: None, heal: true }
    }

    /// The same order with the rovers' healing on or off.
    pub fn with_heal(self, heal: bool) -> GroupOrder {
        GroupOrder { heal, ..self }
    }

    /// The same order with a company cap.
    pub fn with_cap(self, cap: usize) -> GroupOrder {
        GroupOrder { cap, ..self }
    }
}

/// Why a member is out of its slot, read this tick from what it is doing (nothing is remembered about a member).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// Moving: on its way to its slot.
    Coming,
    /// Not moving and not in its slot.
    Standing,
    /// Under one of the lane's reflexes (flee, fan, kite), which took it out of its slot.
    Reflex,
}

/// One company's reading in the army's state: where its members are (the median by coordinate), its block at the
/// place, how many it has, how many are within the band of their slots, how many stand in its block at the place,
/// the way left to it, and whether it is stopped by the enemy or waiting beyond a tight stretch.
#[derive(Clone, Debug, PartialEq)]
pub struct CompanyState {
    pub centre: Vec3,
    pub block: Vec3,
    pub members: usize,
    pub formed: usize,
    pub at_place: usize,
    pub to_go: f32,
    pub halted: bool,
    pub passing: bool,
}

/// What the lane reports of a group this tick. Every field is read from the units, the order and the army's blocks
/// at the place; none is a formation's own bookkeeping (the design's §5.10), so no re-plan, re-deal or restart
/// changes what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupState {
    /// The place the army's blocks are laid at (the order's, an attack's target as last decided); `None` before the
    /// first order.
    pub place: Option<Vec3>,
    /// Where the group is: the median by coordinate of its members' positions.
    pub centre: Vec3,
    /// The way the ranks face.
    pub facing: (f32, f32),
    pub members: usize,
    /// Members standing inside the group's blocks at the place.
    pub at_place: usize,
    /// `at_place` is the share (`Pacing::share`) of the members or more.
    pub arrived: bool,
    /// The median over the members of the way left to their own slot at the place, in elmos; `None` when a company
    /// has no way to its block on the group's ground (the hands' "no way on foot").
    pub to_go: Option<f32>,
    /// The median of the members' speeds this tick, and the slowest company's full pace (elmos a second).
    pub speed: f32,
    pub full_pace: f32,
    /// Members within the band of their slot.
    pub formed: Vec<UnitId>,
    /// Members out of their slot: why, and seconds to it at the unit's own speed.
    pub exceptions: Vec<(UnitId, Cause, f32)>,
    /// The nearest armed enemy some member has in reach.
    pub engaged: Option<UnitId>,
    /// Members with an armed enemy in reach, and companies stopped because every slot of theirs has one.
    pub in_reach: usize,
    pub halted: usize,
    /// The host's route points still ahead of the army (0 on the last leg).
    pub legs_left: usize,
    /// The members through, of the companies waiting beyond a tight stretch; `None` when none is.
    pub passing: Option<usize>,
    pub companies: Vec<CompanyState>,
}

/// The loop's numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pacing {
    /// The pace as a share of the slowest type's speed. 1.0: a pace-type member behind its slot closes only when the
    /// frame slows or stops (the design's §5.6, a stated limit; the straggler drill measures it).
    pub fraction: f32,
    /// The fewest frames between a member's slot orders.
    pub cadence: i32,
    /// A member this near its slot is formed (two hulls).
    pub formed_within: f32,
    /// The hold: the frame walks at full pace while the member `share` of the way up the list of live members' lags
    /// is no further behind its slot than `drag_from`, and stands when it is `exception_at` behind (times the pitch
    /// over the small hulls', for large ones).
    pub drag_from: f32,
    pub exception_at: f32,
    /// The share of a company the frame waits for, the share of a block's slots that must stand for ranks, and the
    /// share of the members in the blocks that is arrival (the design's D5).
    pub share: f32,
    /// The slowest type's goal is its slot carried this far ahead along the travel, times the pace's share.
    pub lead: f32,
    /// The front's stop: no slot comes nearer an armed enemy in sight than this share of the company's shortest reach.
    pub close_to: f32,
    /// A member with no order further than the band from its goal is sent again once its last order is this many
    /// frames old, at a cadence that doubles while it does not move. `i32::MAX`: never.
    pub idle_after: i32,
    /// H-MICRO-NEAR: a marching member's slot order goes out again only once its point has moved this share of the
    /// pitch across since the last.
    pub near: f32,
}

/// The share the frame waits for (four fifths).
pub const SHARE: f32 = 0.8;
/// The slowest type's lead.
pub const LEAD: f32 = 300.0;
/// `Pacing::idle_after`: a second; and the longest a member that does not move waits to be sent again.
pub const IDLE_AFTER: i32 = 30;
pub const IDLE_LONGEST: i32 = 240;
/// `Pacing::near`: a quarter of the pitch across (`docs/design/2026-11-10-order-budget.md`, rule 1).
pub const NEAR_SHARE: f32 = 0.25;
/// A place that moved less than this since the order in force is the same place.
pub const ROUTE_MOVED: f32 = 48.0;
/// An attack's target that has moved this far since the place was decided moves the place.
pub const REPLAN_MOVED: f32 = 128.0;
/// How much nearer the army another member of the party must be before the place leaves the one it is on.
pub const PARTY_SWITCH: f32 = 150.0;
/// The room kept between two blocks.
pub const BLOCK_MARGIN: f32 = 48.0;
/// The facing changes only for an approach more than this off its axis, either way.
pub const REFACE_DEG: f32 = 60.0;
/// What a decision stands for at the least (frames): the facing, a re-form, a pitch. Longer than a re-dealt company
/// takes to stand in its slots again (a slot moved 480 at 54 a second is nine seconds).
pub const DWELL: i32 = 300;
/// The frames between two deals taken on gain in one company, and the gain that takes one: this share of the total
/// squared travel and this much a member.
pub const DEAL_DWELL: i32 = 150;
pub const DEAL_GAIN: f32 = 0.25;
pub const DEAL_GAIN_EACH: f32 = 32.0;
/// A way in force gives way to a fresh plan only when that is shorter by this share and this many elmos.
pub const WAY_GAIN: f32 = 0.2;
pub const WAY_GAIN_ELMOS: f32 = 200.0;
/// The planned way is sampled every `RELAX_STEP` for stretches where the block does not stand; a group that fights
/// passes one relaxed only with nothing hostile within `RELAX_CLEAR` of it.
pub const RELAX_STEP: f32 = 32.0;
pub const RELAX_CLEAR: f32 = 1000.0;
/// How far past a stretch a forming-up ground is looked for along the way.
pub const FORM_AHEAD: f32 = 3000.0;
/// How far behind its ward's centre a follower's frame stays along its own travel while the ward marches.
pub const FOLLOW_KEEP: f32 = 400.0;
/// A member moving slower than this stands.
pub const MOVING: f32 = 5.0;

/// Whether the relaxed posture is on (the bench's arm: `WITHIN_REASON_RELAX=0` keeps the ranks on any ground).
pub fn relax_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("WITHIN_REASON_RELAX").map_or(true, |v| v != "0"))
}

/// The bench's arm for a faster type's stride (`WITHIN_REASON_STRIDE=64`): a member faster than its company's pace
/// is sent its slot carried this far ahead along the travel, and sent again once that point has moved this far. 0,
/// the default, sends it its slot itself, again each `form::near_tolerance` the slot has moved
/// (K-group-a-faster-type-costs-an-order-a-cell).
pub fn stride() -> f32 {
    static ARM: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *ARM.get_or_init(|| std::env::var("WITHIN_REASON_STRIDE").ok().and_then(|v| v.parse().ok()).filter(|v: &f32| *v > 0.0).unwrap_or(0.0))
}

impl Default for Pacing {
    fn default() -> Pacing {
        Pacing {
            fraction: 1.0,
            cadence: 3,
            formed_within: 2.0 * form::HULL_SPACING,
            drag_from: 2.0 * form::HULL_SPACING,
            exception_at: 300.0,
            share: SHARE,
            lead: LEAD,
            close_to: 0.5,
            idle_after: IDLE_AFTER,
            near: NEAR_SHARE,
        }
    }
}

/// The room a group's way is priced for: a block's half-width at this spacing plus a hull.
pub fn wanted_room(spacing: f32) -> f32 {
    (form::FILES as f32 - 1.0) / 2.0 * spacing + form::HULL_SPACING
}

/// A member as the army sees it this tick.
#[derive(Clone, Copy, Debug)]
pub struct Walker {
    pub id: UnitId,
    pub pos: Vec3,
    /// Its type's speed, elmos a second.
    pub speed: f32,
    /// Its speed this tick, elmos a second.
    pub moving: f32,
    /// Its weapons' reach; 0 for none.
    pub reach: f32,
    /// Its pathing hull's side, for its company's pitch.
    pub hull: f32,
    /// Under one of the lane's reflexes as of the last tick.
    pub reflex: bool,
}
