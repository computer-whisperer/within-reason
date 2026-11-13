//! Soldier groups: the pianist's unit of command. A group is a name, its slots, a task and a place, and only the
//! player makes, resizes or ends one (`docs/design/2026-11-11-groups-as-slots.md` §3; H-HANDS-SLOTS). Once a second
//! every soldier of the seat is dealt to a group's slot from its position and type alone (`micro::deal`): inside the
//! group's gate it is with it, of its body; outside, it is coming to it on one plain order. Nothing of a soldier's
//! past enters: not the factory it came from, not the group it hunted for (the user, 2026-11-10: "almost no hidden
//! state to our unit control"). A group carries one task at a time; between calls the standing orders are kept up (an
//! engagement follows its party, an arrival becomes a hold). Until 2026-11-11 a soldier joined the group the player
//! named for its factory and stayed in it wherever it stood (H-HANDS-GROUPS), which in the lobby 10v8 on Erebos Lakes
//! made one group a string of 10 to 23 companies from its plants to the front.

use std::collections::{BTreeMap, HashMap};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::Party;
use crate::world::Domain;

/// A moving group has arrived when its centre is this close to its destination; a declared group's front that stops
/// further than this from the place it walked to has "stopped short" in the news and the idle words.
pub(super) const ARRIVED: f32 = 300.0;
/// H-HANDS-ARRIVAL: a declared group's walk has arrived when a company of this many or more stands arrived at the
/// way's end (the lane's own flag, per company). The lane's composed `arrived` waits for every company, and an army
/// nearly always has one on the road: apex-v0409-comet-sym0-opus-1, the lead company of 18 stood arrived at spot_23
/// from 10:40 to 11:57 and two companies of 35 at spot_46 from 13:58 to 14:14 while the group read "walking to" and
/// Jev was asked to change a course that was over for the front. A company under this size is a straggler.
const FRONT_MIN: usize = 4;
/// The companies a group holds at most (the user, 2026-11-10: "groups should never grow beyond a certain size --
/// perhaps just 4 companies"), and so the slots of the group a seat has when its player has declared none.
pub(crate) const GROUP_COMPANIES: usize = 4;
/// Enemies this close to a group's body are what it met since its last stop (the route's `met` fact).
const MET_REACH: f32 = 1200.0;
/// Places reached kept per group, the oldest dropped: the words stay short.
const REACHED_KEPT: usize = 6;

/// H-HANDS-ARRIVAL: where a declared group's walk to `to` has arrived, if it has. A group of companies: the anchor of
/// the arrived marching company of `FRONT_MIN` or more nearest the way's end (a muster at its point is not of the
/// march; a smaller company is a straggler). A group without companies: its own anchor when its flag says so.
fn front_arrived(state: &micro::GroupState, to: Vec3) -> Option<Vec3> {
    if state.companies.is_empty() {
        return state.arrived.then_some(state.anchor);
    }
    (state.companies.iter())
        .filter(|c| c.arrived && c.members >= FRONT_MIN)
        .min_by(|a, b| a.anchor.dist2d(to).total_cmp(&b.anchor.dist2d(to)))
        .map(|c| c.anchor)
}
/// A party nobody has seen for this long is gone; a ground group holds where it stands, an air group searches its
/// target's last position once and holds only after `AIR_LOST_FRAMES` (H-HANDS-AIR-TARGET: evidence-1-bombers, where
/// the hold after six seconds out of sight cancelled every strike).
const LOST_FRAMES: i32 = 6 * FRAMES_PER_SECOND;
const AIR_LOST_FRAMES: i32 = 60 * FRAMES_PER_SECOND;
/// H-HUNT-HIDDEN: a hunt on a quarry that cloaks (a spy, a sniper, a pop-up turret) runs this long out of sight and
/// radar before it ends, the hunters walking to its last place meanwhile (within its decloak distance it shows
/// again); the end's word is "hidden", not "lost" (the user, 2026-10-18: "a cloakable quarry out of sight is hidden,
/// not lost"; docs/code-decisions.md §2).
pub(crate) const HUNT_HIDDEN_FRAMES: i32 = 30 * FRAMES_PER_SECOND;
/// H-HANDS-LOSS-WAKE: a group wakes the player once per turn when, since the player's last orders, it has lost this
/// many soldiers or this share of its metal (upgrade-2: a unit loss woke nothing, and the ball shed pairs against the
/// Hounds for 7 to 37 s until the timer; Jev's `needs_player` Noul was a constant 0.77 with noise, retired).
const LOSS_WAKE_COUNT: usize = 2;
const LOSS_WAKE_SHARE: f32 = 0.25;
/// A hunt (`docs/design/2026-09-26-threat-response.md` §1; the user, 2026-09-29: a hunt is a group of its own, never
/// part of the mass it left): the group's members attack one unit of a party by id, raw (no formation, no flee, no
/// march), until the quarry dies, is out of sight and radar for `HUNT_LOST_FRAMES`, or the quarry has run past the
/// leash from where it stood when the hunt began (the chase is bounded, the approach is not). Whether the
/// hunters can catch or survive the quarry is said in the state's words for the pick; the engine drops nobody. The
/// group holds where the hunt ends; the merge back is a state Jev picks or the player asks for.
#[derive(Clone, Debug)]
pub(crate) struct Hunt {
    pub quarry: UnitId,
    pub party: String,
    /// Where the quarry stood when the hunt began: the leash is its run from there (the engine's `quarry_from`; the
    /// chase words read it). Until 2026-10-25 evening the leash was the hunters' run from where each stood, so a
    /// hunt begun farther than the leash from its party ended 900 into the approach (migration-3-1: every hunt of
    /// the first ten minutes, the sends repeated after each); before that the hunters' centre (the impedance tally,
    /// 2026-10-16: 174 of 395 leash ends "after 0 s").
    pub from: Vec3,
    pub since: i32,
    pub last_seen: i32,
    /// Where the quarry was last seen: the attack goes there when it is on radar only.
    pub at: Vec3,
}

/// A hunt whose quarry has run this far from where it stood when the hunt began ends (threats-smoke-1: anchored on
/// the group's station, a hunt begun 900 from the station ended on its first tick, 12 of 16 hunts one order long).
/// The engine judges it (H-MICRO-HUNT).
pub(crate) const HUNT_LEASH: f32 = 900.0;
/// Members a company of a declared group holds at most (the companies design, `docs/design/2026-10-25-companies.md`
/// §4), unless the player sets the group's `company` with the `lane` tool; six files by four ranks, under 300 deep.
pub(crate) const COMPANY_CAP: usize = 24;
/// A follower is sent after its ward again when the ward has moved this far from where the follower was last sent.
const FOLLOW_STEP: f32 = 150.0;
/// Where a follower is sent this second (H-HANDS-FOLLOW), or nowhere while its last order still fits: after its
/// ward when the ward has moved `FOLLOW_STEP` from where the follower was last sent (`at`).
pub(super) fn follow_step(ward_at: Vec3, at: Vec3) -> Option<Vec3> {
    (ward_at.dist2d(at) > FOLLOW_STEP).then_some(ward_at)
}

/// Whom a group or a builder follows: another group of ours, or a builder.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Ward {
    Group(String),
    Builder(UnitId),
}
/// A roving group's log keeps this many lines.
const ROVE_LOG: usize = 10;

#[derive(Clone, Debug)]
pub(crate) enum GroupTask {
    /// `picked`: a hold Jev picked, which is a course; a hold an order ended in is none, and the group is idle.
    /// `at`: where the lane's anchor stands for the hold (its Hold route; the ranks hang from it): the anchor as it is
    /// when an arrival ends in a hold or a hold is picked, the body's place when the group forms (the migration
    /// design, chunk 2).
    Hold { picked: bool, at: Vec3 },
    Move { to: Vec3, place: String, fight: bool },
    /// `target`: one unit of the party every member attacks directly (`attack_unit`, and every air group's
    /// engagement): re-issued while it is seen, the group holds when it is lost or dead.
    /// `searched`: an air group has been sent to its lost target's last position.
    /// `at`: where the party stands now; `sent`: where the fight order in force is aimed, which follows `at`
    /// (H-HANDS-ATTACK-FOLLOWS).
    Engage { party: Vec<UnitId>, at: Vec3, sent: Vec3, last_seen: i32, target: Option<UnitId>, searched: bool },
    /// A hunt (H-MICRO-HUNT): the whole group after one unit of a party, the micro engine's to run.
    Hunt(Hunt),
    /// Following (H-HANDS-FOLLOW): the group stays beside a builder or another group of ours wherever it goes.
    /// `at`: where the ward stood at the group's last order to follow. Whom it fights is the pick's and the
    /// engine's on its fight orders, never this task's.
    Follow { ward: Ward, name: String, at: Vec3, since: i32 },
    /// Roving (H-MICRO-ROVE): every soldier is the lane's, looking at what we know least, killing what it finds
    /// unguarded and keeping out of every reach. A pick like every task (the `rove` move); the way back is any other pick (docs/design/2026-10-07-rove-approval.md, the rework).
    /// `within`: a region the player marked, outside which the rovers look at nothing and attack nothing (the
    /// `rove_<region>` move; docs/design/2026-10-17-rove-destinations.md).
    Rove { within: Option<String> },
}

/// One soldier lost: the frame, its type, and the killer with its type when the attacker was known and identified.
pub(crate) type Loss = (i32, UnitDefId, Option<(UnitId, UnitDefId)>);

#[derive(Debug)]
pub(crate) struct Group {
    pub name: String,
    /// Its members' movement domain (docs/design/2026-09-22-domains.md): a group never mixes air and ground.
    pub domain: Domain,
    /// Its slots, the player's (the slots design §3.1): the strength it wants, by type where the player named types.
    /// An answer has none: its soldiers are lent.
    pub slots: micro::deal::Slots,
    /// Where its slots are: the point a holding group holds; the centre of the soldiers with a moving one, or where
    /// it last had any. A roving group's is not read.
    pub place: Vec3,
    /// The soldiers with it, its body, as the deal read them a second ago: what the lane forms, what the odds are
    /// priced on, what an order moves.
    pub members: Vec<UnitId>,
    /// The soldiers coming to it, soonest first, each with its seconds to the place: on one plain order
    /// (`micro::deal::Comers`), not of the body until they are inside its gate.
    pub coming: Vec<(UnitId, f32)>,
    /// The party this group is the answer to (§5), when it is one: made and ended by the worlds, never named by the
    /// player. Its members are the soldiers lent to it; `lent` is how many each lender gave.
    pub answer: Option<String>,
    pub lent: Vec<(String, usize)>,
    pub task: GroupTask,
    pub last_order: i32,
    /// Soldiers lost in the last minute (frame, type); the turn frame the wake counts from, and whether the player has
    /// been woken for them this turn (H-HANDS-LOSS-WAKE). The picture says the last 30 s.
    /// (frame, the type lost, the killer and its type when known): the picture's `losses` line says to whom.
    pub losses: Vec<Loss>,
    /// The soldiers lost in the last minute on their way to it, each with where it died: no loss of the body's, and
    /// said in the `reinforcements` line (a comer walks alone and nothing turns it back, so his guns on its way
    /// show here first).
    pub lost_coming: Vec<(Loss, Vec3)>,
    pub losses_since: i32,
    pub loss_warned: bool,
    /// Shelling a party from a standoff: the long-reach members at the standoff, the rest between (`shell`); `screen`
    /// is where the rest stand, the lane's second formation of the group (the migration design, chunk 5).
    pub shelling: bool,
    pub screen: Option<Vec3>,
    /// The named places its body has come within `ARRIVED` of, in order, with the frame: facts said in the picture
    /// and in `recent` and never acted on by code (the registers; docs/design/2026-09-28-routes-in-prose.md §4.2).
    pub reached: Vec<(String, i32)>,
    /// The first enemies in sight near the body since the last place reached: the frame and the words.
    pub met: Option<(i32, String)>,
    /// What its rovers found, attacked, ran from and gave up, newest last, for the picture: (frame, words).
    pub rove_log: Vec<(i32, String)>,
    /// The place whose walk ended in the hold the group is in (H-HANDS-ARRIVAL), until its next task, and how far
    /// from it the front stopped: the idle words say "a stop it has reached (spot_15)" within `ARRIVED` and "a stop
    /// 480 short of spot_63" past it, and while it is set the nearness register above is quiet (apex-v0409-comet-
    /// sym0-opus-2 13:16-13:46: the front company arrived 300-odd from spot_15, the idle words said nothing of a stop
    /// reached, and the pick sent the group to spot_15 again every second or two, twenty times; barb-ladder3-20-1
    /// 14:23-14:51: the front stopped 480 from spot_63 beside spot_59, the arrival wrote "reached spot_63" and the
    /// nearness register "reached spot_59" in turn every second, each a piece of news, and the words "a stop it has
    /// reached (spot_63)" stood against a packet asking for spot_63 while the pick walked the group there 23 times).
    pub arrived_at: Option<(String, f32)>,
}

/// A group's shape on the ground this second (H-HANDS-GROUP-BODY): the core (members not still joining), its front
/// (the member nearest `toward`: the goal, else the nearest enemy) and tail (the farthest), the body's position
/// (the `BODY_SHARE`-th nearest member, as the march measures it), and how many of the core stand within `ARRIVED`
/// of the goal. A group was a point before: its centroid stood in deep water when the group straddled a cliff
/// (Cape Violet 9:40-11:17), "advancing, 934 to go" was said of a column strung 2,000-2,900 long whose head was
/// already dying under turrets (game 3 8:35), and the odds were priced on all 48 when 17 were in the fight.
pub(crate) struct Body<'a> {
    pub core: Vec<&'a OwnUnit>,
    /// Under the `stragglers` layer (H-HANDS-STRAGGLERS): the members of the farthest fifth that stand over
    /// `STRUNG_OUT` from the body's place. Out of the core, as the joiners are: four fifths of a large body stood
    /// within 330 to 520 of its middle while its front-to-tail length was over `STRUNG_OUT` in 84 to 100% of seconds
    /// (`run/group_shape.py`), so every large group was said strung out and priced in part.
    pub stragglers: Vec<&'a OwnUnit>,
    pub front: Vec3,
    pub tail: Vec3,
    /// From the front to the tail.
    pub length: f32,
    /// Where the body stands: the `BODY_SHARE`-th member from the front.
    pub at: Vec3,
    pub arrived: usize,
}

/// The body is measured at this share of the core, counted from the front (the march's own measure).
const BODY_SHARE: f32 = 0.6;
/// One member in this many, the farthest from the body's place, can be a straggler.
const STRAGGLER_SHARE: usize = 5;

impl Body<'_> {
    /// The core members within `reach` of `at`: the part of the group that is in a fight there.
    pub(crate) fn within(&self, at: Vec3, reach: f32) -> Vec<&OwnUnit> {
        self.core.iter().copied().filter(|u| u.pos.dist2d(at) <= reach).collect()
    }

    /// Strung out: the tail is farther behind the front than a group fights across.
    pub(crate) fn strung_out(&self) -> bool {
        self.length > STRUNG_OUT
    }
}

/// A group longer than this from front to tail fights in pieces: the words and the odds say which piece.
pub(crate) const STRUNG_OUT: f32 = 600.0;

impl Group {
    /// What the group is doing, in a word or two: for the loss wake and the record's `lane` row.
    pub(crate) fn course_word(&self) -> String {
        match &self.task {
            GroupTask::Rove { within, .. } => within.as_ref().map_or("roving".to_string(), |w| format!("roving within {w}")),
            GroupTask::Hold { .. } => "holding".to_string(),
            GroupTask::Move { place, fight, .. } => format!("{} to {place}", if *fight { "advancing" } else { "walking" }),
            GroupTask::Engage { .. } => "attacking a party".to_string(),
            GroupTask::Hunt(h) => format!("hunting {}", h.party),
            GroupTask::Follow { name, .. } => format!("following {name}"),
        }
    }

    /// A group of these slots at this place, with nobody in it yet: the deal fills it.
    pub(crate) fn new(name: String, domain: Domain, slots: micro::deal::Slots, place: Vec3, task: GroupTask, frame: i32) -> Group {
        Group { name, domain, slots, place, members: Vec::new(), coming: Vec::new(), answer: None, lent: Vec::new(), task, last_order: frame, losses: Vec::new(), lost_coming: Vec::new(), losses_since: frame, loss_warned: false, shelling: false, screen: None, reached: Vec::new(), met: None, rove_log: Vec::new(), arrived_at: None }
    }

    /// A group whose body is these soldiers, with a slot for each: what a test stands a group up as.
    #[cfg(test)]
    pub(crate) fn of(name: String, domain: Domain, members: Vec<UnitId>, task: GroupTask, frame: i32) -> Group {
        let mut group = Group::new(name, domain, micro::deal::Slots::any(members.len()), Vec3::default(), task, frame);
        group.members = members;
        group
    }

    /// The group's soldiers in every state: with it and coming to it.
    pub(crate) fn strength(&self) -> usize {
        self.members.len() + self.coming.len()
    }

    /// Whether the group is the lane's as a formation (the migration design, chunks 2-5): a ground group holding,
    /// walking, attacking, following or shelling (two formations then). Its members get their orders from the lane's
    /// group, never from the hands: no per-unit order, no joiners' walk, arrival from the lane's state.
    pub(crate) fn declared(&self) -> bool {
        self.domain == Domain::Ground && matches!(self.task, GroupTask::Hold { .. } | GroupTask::Move { .. } | GroupTask::Engage { .. } | GroupTask::Follow { .. })
    }

    /// The group's name in the lane.
    pub(crate) fn lane_name(&self) -> String {
        format!("group_{}", self.name)
    }

    /// The lane's name for the shell's screen, the second formation of a shelling group.
    pub(crate) fn screen_lane_name(&self) -> String {
        format!("group_{}_screen", self.name)
    }

    /// The group's body toward `toward` (the goal of a walk, the nearest enemy, or nothing: then the front is the
    /// centre and the tail the member farthest from it). `None` when no member stands.
    pub(crate) fn body<'a>(&self, own: &'a [OwnUnit], toward: Option<Vec3>) -> Option<Body<'a>> {
        self.body_with(own, toward, super::layers::stragglers())
    }

    /// `body`, with the `stragglers` layer on or off.
    pub(crate) fn body_with<'a>(&self, own: &'a [OwnUnit], toward: Option<Vec3>, stragglers: bool) -> Option<Body<'a>> {
        let mut core = self.units(own);
        let centre = centre_of(&core)?;
        let toward = toward.unwrap_or(centre);
        core.sort_by(|a, b| a.pos.dist2d(toward).total_cmp(&b.pos.dist2d(toward)));
        let place = |core: &[&OwnUnit]| core.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len() - 1)).map_or(centre, |u| u.pos);
        let mut apart: Vec<&OwnUnit> = Vec::new();
        if stragglers {
            let at = place(&core);
            let mut farthest: Vec<&OwnUnit> = core.clone();
            farthest.sort_by(|a, b| b.pos.dist2d(at).total_cmp(&a.pos.dist2d(at)));
            apart = farthest.into_iter().take(core.len() / STRAGGLER_SHARE).filter(|u| u.pos.dist2d(at) > STRUNG_OUT).collect();
            core.retain(|u| !apart.iter().any(|a| a.id == u.id));
        }
        let front = core.first().map_or(centre, |u| u.pos);
        let tail = core.last().map_or(centre, |u| u.pos);
        let at = place(&core);
        let arrived = core.iter().filter(|u| u.pos.dist2d(toward) < ARRIVED).count();
        Some(Body { length: front.dist2d(tail), core, stragglers: apart, front, tail, at, arrived })
    }

    /// Orders for a soldier that has come inside the gate of a group the hands order member by member (aircraft): what
    /// the task would give it now. A declared group's newcomer is the lane's.
    pub(crate) fn rejoin_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
        match &self.task {
            GroupTask::Hold { .. } => self.hold_orders(units),
            GroupTask::Move { to, fight, .. } => units.iter().map(|u| if *fight { Command::Fight { unit: u.id, to: *to, queue: false } } else { Command::Move { unit: u.id, to: *to, queue: false } }).collect(),
            GroupTask::Engage { at, target, .. } => units.iter().map(|u| match target { Some(t) => Command::Attack { unit: u.id, target: *t, queue: false }, None => Command::Fight { unit: u.id, to: *at, queue: false } }).collect(),
            GroupTask::Hunt(h) => units.iter().map(|u| Command::Attack { unit: u.id, target: h.quarry, queue: false }).collect(),
            GroupTask::Follow { at, .. } => units.iter().map(|u| Command::Fight { unit: u.id, to: *at, queue: false }).collect(),
            // The lane takes a roving group's soldiers from the next tick: no order of the hands'.
            GroupTask::Rove { .. } => Vec::new(),
        }
    }

    /// Whether the lane runs its soldiers (H-MICRO-ROVE).
    pub(crate) fn roving(&self) -> bool {
        matches!(self.task, GroupTask::Rove { .. })
    }

    /// The soldiers of the body lost since `since`, and their metal.
    pub(crate) fn lost_since(&self, since: i32, world: &crate::world::World) -> (usize, f32) {
        let lost: Vec<&Loss> = self.losses.iter().filter(|(f, ..)| *f >= since).collect();
        (lost.len(), lost.iter().filter_map(|(_, def, _)| world.def(*def)).map(|d| d.metal_cost).sum())
    }

    /// The soldiers lost since `since` on their way to it, and their metal.
    pub(crate) fn lost_coming_since(&self, since: i32, world: &crate::world::World) -> (usize, f32) {
        let lost: Vec<&Loss> = self.lost_coming.iter().map(|(l, _)| l).filter(|(f, ..)| *f >= since).collect();
        (lost.len(), lost.iter().filter_map(|(_, def, _)| world.def(*def)).map(|d| d.metal_cost).sum())
    }

    /// The orders that make this group stand still: a stop for ground units; for aircraft, whose empty queue is a
    /// licence to hunt, hold position and a move to the centre (the domains design, decision 4).
    pub(crate) fn hold_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
        if self.declared() {
            // The lane's Hold keeps the ranks where they stand (the migration design, chunk 2).
            return Vec::new();
        }
        match self.domain {
            Domain::Air => {
                let centre = centre_of(units).unwrap_or_default();
                units.iter().flat_map(|u| [Command::MoveState { unit: u.id, state: 0 }, Command::Move { unit: u.id, to: centre, queue: false }]).collect()
            }
            _ => units.iter().map(|u| Command::Stop { unit: u.id }).collect(),
        }
    }

    /// Before an order to an air group: manoeuvre again, so a fight order picks targets on its way.
    pub(crate) fn release_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
        match self.domain {
            Domain::Air => units.iter().map(|u| Command::MoveState { unit: u.id, state: 1 }).collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn units<'a>(&self, own: &'a [OwnUnit]) -> Vec<&'a OwnUnit> {
        own.iter().filter(|u| self.members.contains(&u.id)).collect()
    }
}

impl Group {
    /// A new task; a shell's standoff placement ends with it.
    pub(crate) fn set_task(&mut self, task: GroupTask, frame: i32) {
        self.task = task;
        self.last_order = frame;
        self.shelling = false;
        self.screen = None;
        self.arrived_at = None;
    }
}

impl GroupTask {
    /// The hold an order ends in, or a group forms in: nobody picked it.
    pub(crate) fn idle_hold(at: Vec3) -> GroupTask {
        GroupTask::Hold { picked: false, at }
    }
}

pub(crate) fn centre_of(units: &[&OwnUnit]) -> Option<Vec3> {
    if units.is_empty() {
        return None;
    }
    let n = units.len() as f32;
    Some(units.iter().fold(Vec3::default(), |sum, u| Vec3 { x: sum.x + u.pos.x / n, y: 0.0, z: sum.z + u.pos.z / n }))
}

/// A group's name as the hands keep it: the player's `group_X` without the prefix, with the seat's tag on it.
pub(crate) fn group_name(declared: &str, tag: &str) -> String {
    let base = declared.trim_start_matches("group_");
    if tag.is_empty() || base.ends_with(tag) { base.to_string() } else { format!("{base}{tag}") }
}

/// The answer to a party as a group of the hands' (the slots design §5): one a party, named for it.
pub(crate) fn answer_name(party: &str, tag: &str) -> String {
    format!("answer_{party}{tag}")
}

impl Brain {
    /// The groups are the player's (the slots design §3.1; H-HANDS-SLOTS): those its `groups` tool declared, each
    /// made where it said (home when it said nowhere) holding with no pick, resized when its slots change, and ended
    /// when it is no longer declared, its soldiers then dealt elsewhere. A seat whose player has declared none has
    /// one, `group_A`, of `GROUP_COMPANIES` companies of any type, holding at home. An answer is not the player's and
    /// is left alone here.
    fn sync_groups(&self, pianist: &mut super::Pianist, frame: i32) {
        let declared = self.strategist.as_ref().map(|s| s.groups.lock().unwrap().clone()).unwrap_or_default();
        let tag = pianist.seat_tag.clone();
        let mut wanted: Vec<(String, micro::deal::Slots, Domain, Vec3)> = declared
            .iter()
            .map(|(name, d)| {
                let typed = d.typed.iter().filter_map(|(unit, n)| Some((self.world.def_named(unit)?, *n))).collect();
                let at = d.at.map_or(self.home, |[x, z]| Vec3 { x, y: 0.0, z });
                (group_name(name, &tag), micro::deal::Slots { any: d.any, typed }, if d.air { Domain::Air } else { Domain::Ground }, at)
            })
            .collect();
        if wanted.is_empty() {
            wanted.push((group_name("group_A", &tag), micro::deal::Slots::any(GROUP_COMPANIES * COMPANY_CAP), Domain::Ground, self.home));
        }
        pianist.groups.retain(|g| g.answer.is_some() || wanted.iter().any(|(name, ..)| *name == g.name));
        for (name, slots, domain, at) in wanted {
            match pianist.groups.iter_mut().find(|g| g.name == name) {
                Some(group) => group.slots = slots,
                None => pianist.groups.push(Group::new(name, domain, slots, at, GroupTask::idle_hold(at), frame)),
            }
        }
    }

    /// The deal (the slots design §3.3; H-MICRO-DEAL), every think: each soldier is with a group (inside its gate: of
    /// its body, the lane's or the task's from now), coming to one (on one plain order to its place), lent to an
    /// answer, or surplus. Air and ground are dealt apart: a group never mixes them. The travel time is the straight
    /// line over the soldier's speed (a group's place moves every second; a field a place would not keep up).
    fn deal_groups(&self, pianist: &mut super::Pianist, soldiers: &[&OwnUnit], commands: &mut Vec<Command>) {
        use micro::deal::{Comer, Demand, Gates, Place, Soldier, Standing};
        let region = |name: &str| pianist.places.iter().find(|p| p.name == name).and_then(|p| p.cells).map(|cells| self.world.cells.rect(cells));
        let mut domains: Vec<Domain> = Vec::new();
        for d in soldiers.iter().map(|u| self.world.domain_of(u.def)).chain(pianist.groups.iter().map(|g| g.domain)) {
            if !domains.contains(&d) {
                domains.push(d);
            }
        }
        // Where each group's slots are now: the hold's point, or the centre of the soldiers that were with it.
        let unit_of: HashMap<UnitId, &OwnUnit> = soldiers.iter().map(|u| (u.id, *u)).collect();
        for group in pianist.groups.iter_mut().filter(|g| g.answer.is_none()) {
            group.place = match &group.task {
                GroupTask::Hold { at, .. } => *at,
                _ => micro::deal::centre(group.members.iter().filter_map(|id| unit_of.get(id)).map(|u| u.pos)).unwrap_or(group.place),
            };
        }
        let mut dealt: HashMap<UnitId, Standing> = HashMap::new();
        for domain in domains {
            let mine: Vec<Soldier> = soldiers.iter().filter(|u| self.world.domain_of(u.def) == domain).map(|u| Soldier { id: u.id, def: u.def, pos: u.pos, speed: self.world.def(u.def).map_or(1.0, |d| d.speed) }).collect();
            let demands: Vec<Demand> = (pianist.groups.iter())
                .filter(|g| g.answer.is_none() && g.domain == domain)
                .map(|g| {
                    let place = match &g.task {
                        GroupTask::Rove { within } => Place::Roving { about: within.as_deref().and_then(region).map(|(a, b)| Vec3 { x: (a.x + b.x) / 2.0, y: 0.0, z: (a.z + b.z) / 2.0 }) },
                        _ => Place::At(g.place),
                    };
                    Demand { name: g.name.clone(), slots: g.slots.clone(), place }
                })
                .collect();
            let answers: Vec<&str> = pianist.groups.iter().filter(|g| g.answer.is_some() && g.domain == domain).map(|g| g.name.as_str()).collect();
            dealt.extend(pianist.deals.entry(domain).or_default().deal(&mine, &demands, &answers, &micro::deal::straight, Gates::default()));
        }
        let fill = micro::deal::fills(&dealt);
        let mut comers: Vec<Comer> = Vec::new();
        for group in pianist.groups.iter_mut() {
            if group.answer.is_some() {
                group.members = dealt.iter().filter(|(_, s)| matches!(s, Standing::Lent(a) if *a == group.name)).map(|(id, _)| *id).collect();
                group.members.sort_by_key(|id| id.0);
                continue;
            }
            let f = fill.get(&group.name).cloned().unwrap_or_default();
            // A soldier that has come inside the gate of a group the hands order member by member takes its task's
            // order; a declared group's is the lane's from the next tick, a rover the lane's whole.
            if !group.declared() && !group.roving() {
                let fresh: Vec<&OwnUnit> = f.with.iter().filter(|id| !group.members.contains(id)).filter_map(|id| unit_of.get(id).copied()).collect();
                commands.extend(group.rejoin_orders(&fresh));
            }
            group.members = f.with;
            group.coming = f.coming;
            // The comers walk to the place, fighting on the way unless the group only walks.
            let fight = !matches!(group.task, GroupTask::Move { fight: false, .. });
            if let Some(first) = group.coming.first().and_then(|(id, _)| unit_of.get(id)) {
                let place = self.snap_for(self.walker_of(first.def), group.place);
                comers.extend(group.coming.iter().filter_map(|(id, _)| unit_of.get(id)).map(|u| Comer { id: u.id, place, fight, idle: u.idle }));
            }
        }
        pianist.comers.orders(&comers, commands);
        pianist.surplus = fill.get("").map(|f| f.with.clone()).unwrap_or_default();
    }
}

impl Brain {
    /// Every think: membership against what stands, new soldiers adopted, standing orders kept up.
    pub(super) fn keep_groups(&mut self, tick: &Tick, kit: &Kit, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, kit)).collect();
        let enemies = &tick.snapshot.enemies;
        let frame = tick.frame;
        let Some(mut pianist) = self.pianist.take() else { return };
        // Losses since the player's last orders, per group; enough of them wake the player (H-HANDS-LOSS-WAKE).
        let turn_frame = self.strategist.as_ref().map_or(0, |s| s.last_turn_frame.load(std::sync::atomic::Ordering::Relaxed));
        let mut loss_wakes: Vec<String> = Vec::new();
        for group in &mut pianist.groups {
            if group.losses_since != turn_frame {
                (group.losses_since, group.loss_warned) = (turn_frame, false);
            }
            group.losses.retain(|(f, ..)| frame - f <= 60 * FRAMES_PER_SECOND);
            group.lost_coming.retain(|((f, ..), _)| frame - f <= 60 * FRAMES_PER_SECOND);
            // A soldier of the group at the last deal: the body's loss when it was with it, its own when it was
            // coming.
            for (f, id, def, killer, at) in self.unit_losses.iter().rev().take_while(|(f, ..)| *f == frame) {
                if group.members.contains(id) {
                    group.losses.push((*f, *def, *killer));
                } else if group.coming.iter().any(|(c, _)| c == id) {
                    group.lost_coming.push(((*f, *def, *killer), *at));
                }
            }
            group.members.retain(|id| soldiers.iter().any(|u| u.id == *id));
            group.coming.retain(|(id, _)| soldiers.iter().any(|u| u.id == *id));
            let standing: f32 = group.units(own).iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            // The wake counts both: soldiers dying on their way to the group are the player's to hear of.
            let (with, on_the_way) = (group.lost_since(turn_frame, &self.world), group.lost_coming_since(turn_frame, &self.world));
            let (count, lost) = (with.0 + on_the_way.0, with.1 + on_the_way.1);
            if !group.loss_warned && !group.members.is_empty() && (count >= LOSS_WAKE_COUNT || lost >= LOSS_WAKE_SHARE * (standing + lost)) {
                group.loss_warned = true;
                let doing = group.course_word();
                let coming = if on_the_way.0 > 0 { format!(", {} of them on their way to it", on_the_way.0) } else { String::new() };
                loss_wakes.push(format!(
                    "group_{} has lost {} of its {} soldiers ({lost:.0} metal) since your last orders{coming}, {doing} at {}",
                    group.name,
                    count,
                    group.strength() + count,
                    self.world.grid(centre_of(&group.units(own)).unwrap_or_default())
                ));
            }
        }
        if let Some(shared) = &self.strategist {
            for text in loss_wakes {
                shared.trigger(text);
            }
        }
        // The player's groups, then the deal over them (the slots design §3): who is with each group, who is coming,
        // who is lent to an answer and who has no slot.
        self.sync_groups(&mut pianist, frame);
        self.deal_groups(&mut pianist, &soldiers, commands);
        // Standing orders, under each group's footwork rules (H-HANDS-LANE); a declared group's are the lane's.
        let mut hunt_ends: Vec<String> = Vec::new();
        let mut route_news: Vec<String> = Vec::new();
        let mut follow_ends: Vec<String> = Vec::new();
        // Where each group stands, for the groups that follow another: its formation's anchor when the lane has one,
        // else its body. (Until 2026-10-31 the body's place for every group: the member at the 60th percentile from
        // the centroid, which in barb-ladder4-20-1 was a joiner crossing the base or a Bull standing away, and seven
        // followers were sent 400-600 elmo to another member of the ward every second.)
        let bodies: Vec<(String, Vec3)> = pianist.groups.iter().filter_map(|g| self.lane_group(&g.lane_name()).map(|s| s.anchor).or_else(|| g.body(own, None).map(|b| b.at)).map(|at| (g.name.clone(), at))).collect();
        for group in pianist.groups.iter_mut() {
            // A roving group is the lane's (H-MICRO-ROVE): no hunt, no orders kept up, every member its own from the
            // tick it joins.
            if group.roving() {
                continue;
            }
            let units = group.units(own);
            let goal = match &group.task { GroupTask::Move { to, .. } => Some(*to), GroupTask::Engage { at, .. } | GroupTask::Follow { at, .. } => Some(*at), GroupTask::Hunt(h) => Some(h.at), GroupTask::Hold { .. } | GroupTask::Rove { .. } => None };
            // Where the body stands: a declared group's anchor, the lane's; else the body's place. A group with nobody
            // with it has nothing to keep up.
            let lane_state = if group.declared() { self.lane_group(&group.lane_name()).cloned() } else { None };
            let Some(centre) = lane_state.as_ref().map(|s| s.anchor).or_else(|| group.body(own, goal).map(|b| b.at)) else { continue };
            // The registers' facts: a named place (`menu::named_places`) the body comes within `ARRIVED` of is
            // reached, said in `recent` and asked about at once; the first enemies in sight since the last stop are
            // kept until the next. Code records; Jev picks the next move.
            // One register writes at a time: a group holding where its walk ended (`arrived_at`) is where that walk
            // says, and the nearest named place is not news until its next task.
            let here = pianist.places.iter().find(|p| pianist.named.contains(&p.name) && p.at.dist2d(centre) < ARRIVED).map(|p| p.name.clone());
            if let Some(place) = here
                && group.arrived_at.is_none()
                && group.reached.last().is_none_or(|(last, _)| *last != place)
            {
                if group.reached.len() >= REACHED_KEPT {
                    group.reached.remove(0);
                }
                group.reached.push((place.clone(), frame));
                group.met = None;
                route_news.push(format!("group_{} reached {place}", group.name));
            }
            if group.met.is_none() {
                let mut counts: BTreeMap<String, usize> = BTreeMap::new();
                let mut at = None;
                for e in enemies.iter().filter(|e| e.pos.dist2d(centre) < MET_REACH) {
                    *counts.entry(e.def.map_or("unidentified".to_string(), |d| self.name(d).to_string())).or_default() += 1;
                    at.get_or_insert(e.pos);
                }
                if let Some(at) = at {
                    let what = counts.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", ");
                    let words = format!("{what} at {}", self.world.grid(at));
                    route_news.push(format!("group_{} met {words}", group.name));
                    group.met = Some((frame, words));
                }
            }
            let declared = group.declared();
            match &mut group.task {
                GroupTask::Hold { .. } | GroupTask::Rove { .. } => {}
                GroupTask::Move { to, place, .. } if declared => {
                    // The front's arrival (H-HANDS-ARRIVAL: an arrived marching company of `FRONT_MIN` or more, the
                    // one nearest the way's end; the formation itself when the group has no companies), read a
                    // second after the order at the earliest: the state before it is the last task's.
                    if frame - group.last_order >= FRAMES_PER_SECOND
                        && let Some(at) = lane_state.as_ref().and_then(|s| front_arrived(s, *to))
                    {
                        // An arrival ends the order in a hold nobody picked, the front's anchor where it stands:
                        // the group is idle and is asked; the companies still on the road walk to the hold. A
                        // shell's hold keeps shelling: the screen stays between (chunk 5). Within `ARRIVED` of the
                        // spot the place is reached, in the register and the idle words; further off the news and
                        // the words say how far short it stopped, and the register is not written.
                        let place = place.clone();
                        let short = at.dist2d(*to);
                        let (shelling, screen) = (group.shelling, group.screen);
                        group.set_task(GroupTask::Hold { picked: false, at }, frame);
                        if short > ARRIVED {
                            route_news.push(format!("group_{} stopped {short:.0} short of {place}", group.name));
                        } else if group.reached.last().is_none_or(|(last, _)| *last != place) {
                            if group.reached.len() >= REACHED_KEPT {
                                group.reached.remove(0);
                            }
                            group.reached.push((place.clone(), frame));
                            route_news.push(format!("group_{} reached {place}", group.name));
                        }
                        group.arrived_at = Some((place, short));
                        if shelling {
                            group.shelling = true;
                            group.screen = screen;
                        }
                    }
                }
                GroupTask::Move { to, .. } => {
                    // An air group, or a shelling one, until their chunks: arrival by the members, not the centre (a
                    // group split by a cliff never got its centre within 300 of its goal, so a fall-back walk stayed
                    // the base world for two minutes and the player's new stations were shown as the default and
                    // never played: Cape Violet 9:40-11:17).
                    let moving: Vec<&OwnUnit> = units.iter().copied().filter(|u| !self.stuck.contains_key(&u.id)).collect();
                    let core = if moving.is_empty() { units.clone() } else { moving };
                    let mut distances: Vec<f32> = core.iter().map(|u| u.pos.dist2d(*to)).collect();
                    distances.sort_by(f32::total_cmp);
                    let to_go = distances.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len().saturating_sub(1))).copied().unwrap_or(0.0);
                    if to_go < ARRIVED {
                        let at = *to;
                        group.set_task(GroupTask::Hold { picked: false, at }, frame);
                    }
                }
                GroupTask::Engage { party, at, sent, last_seen, target, searched, .. } => {
                    let seen: Vec<&bot_protocol::EnemyUnit> = enemies.iter().filter(|e| party.contains(&e.id)).collect();
                    let air = group.domain == Domain::Air;
                    // A named target: the attack stands while the target is seen. Lost, a ground group holds; an air
                    // group searches the last position once and holds only much later (H-HANDS-AIR-TARGET).
                    if let Some(t) = *target {
                        if let Some(e) = seen.iter().find(|e| e.id == t) {
                            let found_again = *searched;
                            *last_seen = frame;
                            *searched = false;
                            *at = e.pos;
                            if found_again && !declared {
                                group.last_order = frame;
                                commands.extend(units.iter().map(|u| Command::Attack { unit: u.id, target: t, queue: false }));
                            }
                        } else if air && frame - *last_seen > AIR_LOST_FRAMES {
                            commands.extend(group.hold_orders(&units));
                            group.task = GroupTask::idle_hold(centre);
                        } else if air && frame - *last_seen > LOST_FRAMES && !*searched {
                            *searched = true;
                            group.last_order = frame;
                            let to = *at;
                            commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to, queue: false }));
                        } else if !air && frame - *last_seen > LOST_FRAMES {
                            commands.extend(group.hold_orders(&units));
                            group.task = GroupTask::idle_hold(centre);
                        }
                    } else if let Some(now) = centre_of_enemies(&seen) {
                        *last_seen = frame;
                        *at = now;
                        // The attack follows the party (H-HANDS-ATTACK-FOLLOWS): the fight order was to where it
                        // stood at the pick, and a party that moves on left the group standing there with no order
                        // while its task still read "attacking" (player-35 7:53-8:28).
                        // A declared group's route carries the party's centre each tick (the lane's, chunk 3).
                        if !declared && let Some(to) = follow_step(now, *sent) {
                            *sent = to;
                            group.last_order = frame;
                            commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to, queue: false }));
                        }
                    } else if frame - *last_seen > LOST_FRAMES {
                        commands.extend(group.hold_orders(&units));
                        group.task = GroupTask::idle_hold(centre);
                    }
                }
                // The hunt's book: the quarry's last sighting for the words; the chase and its ends are the engine's
                // (H-MICRO-HUNT). Every hunter dead ends it, and the answer with it (below).
                GroupTask::Hunt(hunt) => {
                    if let Some(q) = enemies.iter().find(|e| e.id == hunt.quarry) {
                        hunt.last_seen = frame;
                        hunt.at = q.pos;
                    }
                    if units.is_empty() {
                        hunt_ends.push(format!("the answer to {} ended after {} s: every hunter dead", hunt.party, (frame - hunt.since) / FRAMES_PER_SECOND));
                    }
                }
                // Following (H-HANDS-FOLLOW): the group goes after its ward when it has moved on. A ward that is
                // gone (a builder dead, a group dead or merged away) ends it in a hold, said, and the group is asked
                // at once.
                GroupTask::Follow { ward, name: ward_name, at, since } => {
                    let ward_at = match ward {
                        Ward::Builder(id) => own.iter().find(|u| u.id == *id).map(|u| u.pos),
                        Ward::Group(other) => bodies.iter().find(|(name, _)| name == other).map(|(_, at)| *at),
                    };
                    match ward_at {
                        Some(ward_at) => {
                            if let Some(to) = follow_step(ward_at, *at) {
                                *at = to;
                                group.last_order = frame;
                                // A declared group's route carries the ward's place (the lane's, chunk 4).
                                if !declared {
                                    commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to, queue: false }));
                                }
                            }
                        }
                        None => {
                            follow_ends.push(format!("group_{} followed {ward_name} for {} s: {ward_name} is gone; the group holds where it is", group.name, (frame - *since) / FRAMES_PER_SECOND));
                            commands.extend(group.hold_orders(&units));
                            group.task = GroupTask::idle_hold(centre);
                        }
                    }
                }
            }
        }
        for text in follow_ends {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            pianist.note(frame, text.clone());
            pianist.events.insert(text);
        }
        for text in hunt_ends {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            pianist.hunt_events.push(text);
        }
        // An answer with nobody left is over.
        let over: Vec<String> = pianist.groups.iter().filter(|g| g.answer.is_some() && g.members.is_empty()).map(|g| g.name.clone()).collect();
        for name in over {
            pianist.end_answer(&name);
        }
        for text in route_news {
            pianist.note(frame, text.clone());
            pianist.events.insert(text);
        }
        self.pianist = Some(pianist);
    }
}

impl Brain {
    /// The lane's word from the rovers (H-MICRO-ROVE), into their groups' logs (the picture's `rove` entry), one
    /// line a rover, a tick and a kind: what it found, by place; what it attacks; what it ran from; what it gave up.
    /// A building or commander of his found, and every attack, is also what the hands did for the player's report.
    pub(crate) fn rove_events(&mut self, events: &[micro::RoveEvent], frame: i32) {
        let Some(mut pianist) = self.pianist.take() else { return };
        let clock = super::picture::clock(frame);
        for group in pianist.groups.iter_mut() {
            let mine: Vec<&micro::RoveEvent> = events.iter().filter(|e| group.members.contains(&e.rover)).collect();
            if mine.is_empty() {
                continue;
            }
            let mut found: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
            let mut news = false;
            for event in &mine {
                let line = match &event.what {
                    micro::RoveWhat::Found { def, at, .. } => {
                        let d = self.world.def(*def);
                        news |= d.is_some_and(|d| d.speed == 0.0 || (d.build_speed > 0.0 && d.weapon_count > 0));
                        *found.entry(self.place_words(&pianist.places, *at)).or_default().entry(self.short_words(*def)).or_default() += 1;
                        continue;
                    }
                    micro::RoveWhat::Attacks { def, at, from, .. } => {
                        let how = match from {
                            micro::StandKind::Open => "unguarded ".to_string(),
                            micro::StandKind::Rim | micro::StandKind::Shadow => String::new(),
                        };
                        let rim = match from {
                            micro::StandKind::Open => "",
                            micro::StandKind::Rim => " from outside the reach of what guards it",
                            micro::StandKind::Shadow => " from behind it, where its guard cannot shoot through it",
                        };
                        let text = format!("{clock} group_{}'s rover attacks his {how}{} at {}{rim}", group.name, self.short_words(*def), self.place_words(&pianist.places, *at));
                        pianist.done.push(text.clone());
                        text
                    }
                    micro::RoveWhat::Evades { from, at } => format!("{clock} ran from {} at {}", from.map_or("a radar contact".to_string(), |d| format!("his {}", self.short_words(d))), self.place_words(&pianist.places, *at)),
                    micro::RoveWhat::GivesUp { goal, why } => format!("{clock} gave up looking at {goal}: {why}"),
                    micro::RoveWhat::Abandons { def, at, why, .. } => format!("{clock} gave up attacking his {} at {}: {why}", self.short_words(*def), self.place_words(&pianist.places, *at)),
                    micro::RoveWhat::Heals { def, at, .. } => format!("{clock} group_{}'s rover, hurt, goes to our {} at {} to be healed", group.name, self.short_words(*def), self.place_words(&pianist.places, *at)),
                    micro::RoveWhat::Pushes { def, at, against, pack, .. } => {
                        let text = format!("{clock} group_{}'s {pack} rovers push through {against} shooter{} at his {} at {} (the odds allow it)", group.name, if *against == 1 { "" } else { "s" }, self.short_words(*def), self.place_words(&pianist.places, *at));
                        pianist.done.push(text.clone());
                        text
                    }
                };
                pianist.rove_events.push(format!("group_{} {line}", group.name));
                group.rove_log.push((frame, line));
            }
            for (place, kinds) in found {
                let line = format!("{clock} found his {} at {place}", kinds.iter().map(|(k, n)| if *n == 1 { k.clone() } else { format!("{n} {k}") }).collect::<Vec<_>>().join(", "));
                if news {
                    pianist.done.push(format!("{clock} group_{}'s rover {}", group.name, line.trim_start_matches(&clock).trim_start()));
                }
                pianist.rove_events.push(format!("group_{} {line}", group.name));
                group.rove_log.push((frame, line));
            }
            let excess = group.rove_log.len().saturating_sub(ROVE_LOG);
            group.rove_log.drain(..excess);
        }
        self.pianist = Some(pianist);
    }

    /// The micro engine's word on a hunt (`micro::HuntEvent`): its end (the quarry dead or lost, the leash reached)
    /// ends the answer, whose soldiers the next deal places like any others (the slots design §5). Returns the end's
    /// words and, for a hunt that ended without a kill, the failure to remember (the party and why). The caller ends
    /// the answer (`Pianist::end_answer`).
    pub(crate) fn hunt_event(group: &Group, event: &micro::HuntEvent, frame: i32, cloaks: bool) -> Option<(String, Option<(String, String)>)> {
        let GroupTask::Hunt(hunt) = &group.task else { return None };
        // The event names the lane's group, `group_<name>` (D15).
        if hunt.quarry != event.quarry || event.group != format!("group_{}", group.name) {
            return None;
        }
        let (party, since) = (hunt.party.clone(), hunt.since);
        let why = match event.why {
            "dead" => "the quarry is dead".to_string(),
            "lost" if cloaks => format!("{party} hidden: it cloaks, and nothing of ours has seen it for {} s, its last place walked to", HUNT_HIDDEN_FRAMES / FRAMES_PER_SECOND),
            "lost" => format!("{party} out of sight for 6 s"),
            "leash" => format!("its quarry ran {HUNT_LEASH:.0} from where it stood when the hunt began, and was let go"),
            other => other.to_string(),
        };
        let failed = (event.why != "dead").then(|| (party.clone(), why.clone()));
        Some((format!("the answer to {party} ended after {} s: {why}; its {} soldiers go back to the groups with room for them", (frame - since) / FRAMES_PER_SECOND, group.members.len()), failed))
    }

    /// A lend (the slots design §5; H-HANDS-ANSWER): `count` of the lender's soldiers, those that reach the party
    /// soonest, with it or coming to it (the deal's choice), go to the answer to the party. One answer a party: a
    /// hunt of the lane's under the party's name, which the worlds make and end and the player never names; a second
    /// lend at the party joins it. The lender's slots are open from the next deal. Returns what was done. Until
    /// 2026-11-11 the hunters left as a group of their own under a new name (the user, 2026-09-29), kept the lender's
    /// name when half of it went, and came back by a later pick.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lend(pianist: &mut super::Pianist, index: usize, count: usize, party: &Party, own: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit], speed: &dyn Fn(UnitDefId) -> f32, frame: i32, commands: &mut Vec<Command>) -> Option<String> {
        let (lender, domain) = (pianist.groups[index].name.clone(), pianist.groups[index].domain);
        let answer = answer_name(&party.name, &pianist.seat_tag);
        let soldiers: Vec<micro::deal::Soldier> = own.iter().map(|u| micro::deal::Soldier { id: u.id, def: u.def, pos: u.pos, speed: speed(u.def) }).collect();
        let went = pianist.deals.get_mut(&domain)?.lend(&answer, &lender, count, party.at, &soldiers, &micro::deal::straight);
        if went.is_empty() {
            return None;
        }
        let hunters: Vec<&OwnUnit> = own.iter().filter(|u| went.contains(&u.id)).collect();
        let group = &mut pianist.groups[index];
        commands.extend(group.release_orders(&hunters));
        group.members.retain(|id| !went.contains(id));
        group.coming.retain(|(id, _)| !went.contains(id));
        let did = format!("{} of group_{lender} answer {} ({})", went.len(), party.name, party.composition);
        match pianist.groups.iter_mut().find(|g| g.name == answer) {
            Some(group) => {
                group.members.extend(went.iter().copied());
                match group.lent.iter_mut().find(|(name, _)| *name == lender) {
                    Some((_, n)) => *n += went.len(),
                    None => group.lent.push((lender, went.len())),
                }
            }
            None => {
                let mut group = Group::new(answer, domain, micro::deal::Slots::default(), party.at, GroupTask::Hunt(Brain::hunt_of(party, &hunters, enemies, frame)), frame);
                group.answer = Some(party.name.clone());
                group.lent = vec![(lender, went.len())];
                group.members = went;
                pianist.groups.push(group);
            }
        }
        Some(did)
    }

    /// A hunt of the party's unit nearest these hunters, begun where they stand.
    pub(super) fn hunt_of(party: &Party, hunters: &[&OwnUnit], enemies: &[bot_protocol::EnemyUnit], frame: i32) -> Hunt {
        let centre = centre_of(hunters).unwrap_or(party.at);
        let quarry = party
            .ids
            .iter()
            .filter_map(|id| enemies.iter().find(|e| e.id == *id))
            .min_by(|a, b| a.pos.dist2d(centre).total_cmp(&b.pos.dist2d(centre)))
            .map(|e| (e.id, e.pos))
            .unwrap_or((party.ids[0], party.at));
        Hunt { quarry: quarry.0, party: party.name.clone(), from: quarry.1, since: frame, last_seen: frame, at: quarry.1 }
    }

    /// The whole group engages the party: its task the engagement; an undeclared group (air) released and sent to
    /// fight at it, a declared one left to the lane's Attack or FightTo (the migration design, chunk 3).
    pub(super) fn engage_group(group: &mut Group, party: &Party, own: &[OwnUnit], target: Option<UnitId>, frame: i32, commands: &mut Vec<Command>) -> String {
        let units = group.units(own);
        group.set_task(GroupTask::Engage { party: party.ids.clone(), at: party.at, sent: party.at, last_seen: frame, target, searched: false }, frame);
        if !group.declared() {
            commands.extend(group.release_orders(&units));
            match target {
                Some(t) => commands.extend(units.iter().map(|u| Command::Attack { unit: u.id, target: t, queue: false })),
                None => commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to: party.at, queue: false })),
            }
        }
        format!("attack {} ({}) with the whole group", party.name, party.composition)
    }
}

pub(crate) fn centre_of_enemies(units: &[&bot_protocol::EnemyUnit]) -> Option<Vec3> {
    if units.is_empty() {
        return None;
    }
    let n = units.len() as f32;
    Some(units.iter().fold(Vec3::default(), |sum, u| Vec3 { x: sum.x + u.pos.x / n, y: 0.0, z: sum.z + u.pos.z / n }))
}

#[cfg(test)]
mod tests {
    /// A hunt's leash is anchored on the quarry where it stood when the hunt began, not on the hunters: a hunt begun
    /// far from its party is allowed to arrive.
    #[test]
    fn a_hunt_anchors_its_leash_on_the_quarry() {
        let (_, ours, enemies, parties) = e3();
        let hunters: Vec<&bot_protocol::OwnUnit> = ours.iter().take(3).collect();
        let hunt = Brain::hunt_of(&parties[0], &hunters, &enemies, 30);
        let quarry = enemies.iter().find(|e| e.id == hunt.quarry).expect("the quarry is in sight");
        assert_eq!(hunt.from, quarry.pos);
        assert!(parties[0].ids.contains(&hunt.quarry));
    }

    use super::super::fixtures::{e3, own};
    use super::*;

    /// Sixteen soldiers together and four far behind: by the old measure the group is strung out; under the
    /// `stragglers` layer the four are stragglers, out of the core, and the body is not strung out, though it was
    /// without the layer. A column with every soldier 100 behind the last stays strung out.
    #[test]
    fn a_few_far_behind_are_stragglers_and_the_body_is_not_strung_out() {
        let spot = |x: f32| Vec3 { x, y: 0.0, z: 1000.0 };
        let mut ours: Vec<OwnUnit> = (0..16).map(|i| own(i, 1, spot(3000.0 + 20.0 * i as f32))).collect();
        ours.extend((0..4).map(|i| own(16 + i, 1, spot(2000.0 - 300.0 * i as f32))));
        let group = Group::of("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        let toward = Some(spot(4000.0));
        let old = group.body_with(&ours, toward, false).expect("a body");
        assert!(old.strung_out() && old.stragglers.is_empty() && old.core.len() == 20);
        let body = group.body_with(&ours, toward, true).expect("a body");
        assert_eq!((body.stragglers.len(), body.core.len()), (4, 16));
        assert!(!body.strung_out() && old.length > STRUNG_OUT, "{} {}", body.length, old.length);
        let column: Vec<OwnUnit> = (0..20).map(|i| own(i, 1, spot(3000.0 - 100.0 * i as f32))).collect();
        let body = group.body_with(&column, toward, true).expect("a body");
        assert!(body.strung_out() && body.stragglers.len() == 4, "{} {}", body.length, body.stragglers.len());
    }

    /// A follower is sent after its ward when the ward has moved on, and nowhere while its last order still fits; a
    /// member rejoining gets the same order.
    #[test]
    fn a_follower_goes_after_its_ward() {
        let at = |x: f32| Vec3 { x, y: 0.0, z: 0.0 };
        assert_eq!(follow_step(at(100.0), at(0.0)), None);
        assert_eq!(follow_step(at(200.0), at(0.0)), Some(at(200.0)));
        let soldier = own(1, 1, at(0.0));
        let group = Group::of("A".into(), Domain::Air, vec![soldier.id], GroupTask::Follow { ward: Ward::Builder(UnitId(9)), name: "constructor_9".into(), at: at(300.0), since: 0 }, 0);
        assert!(matches!(group.rejoin_orders(&[&soldier])[..], [Command::Fight { to, .. }] if to.x == 300.0));
        // A ground follower is the lane's (chunk 4): declared, a FightTo at the ward's place.
        let ground = Group::of("B".into(), Domain::Ground, vec![soldier.id], GroupTask::Follow { ward: Ward::Builder(UnitId(9)), name: "constructor_9".into(), at: at(300.0), since: 0 }, 0);
        assert!(ground.declared());
    }

    /// A lend (the slots design §5): three of fourteen go to the answer to the party, the lender's soonest to it; a
    /// second lend at the same party joins the same answer; the answer is no group of the player's, and its end
    /// frees its soldiers for the next deal, which finds them the lender's open slots.
    #[test]
    fn a_lend_goes_to_one_answer_a_party_and_its_end_frees_its_soldiers() {
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let all: Vec<&OwnUnit> = ours.iter().collect();
        let centre = centre_of(&all).unwrap();
        pianist.groups.push(Group::of("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(centre), 0));
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None, conversion: 0.0 } };
        let mut commands = Vec::new();
        brain.keep_groups(&tick(30, &ours, &enemies), &kit, &mut commands);
        let mut pianist = brain.pianist.take().unwrap();
        assert_eq!((pianist.groups.len(), pianist.groups[0].members.len(), pianist.groups[0].slots.total()), (1, 14, GROUP_COMPANIES * COMPANY_CAP), "the seat's one group, every soldier with it");
        let speed = |_: UnitDefId| 60.0;
        let did = Brain::lend(&mut pianist, 0, 3, &parties[0], &ours, &enemies, &speed, 30, &mut commands).expect("a lend");
        assert!(did.starts_with("3 of group_A answer party_12"), "{did}");
        assert_eq!(pianist.groups.len(), 2);
        assert_eq!(pianist.groups[0].members.len(), 11);
        let answer = &pianist.groups[1];
        assert_eq!((answer.name.as_str(), answer.answer.as_deref(), answer.members.len()), ("answer_party_12", Some("party_12"), 3));
        // The soonest to the party: nobody left with the lender stands nearer it than a hunter does.
        let far = |id: &UnitId| ours.iter().find(|u| u.id == *id).unwrap().pos.dist2d(parties[0].at);
        let worst = answer.members.iter().map(far).fold(0.0, f32::max);
        assert!(pianist.groups[0].members.iter().all(|id| far(id) >= worst - 1.0));
        let GroupTask::Hunt(hunt) = &answer.task else { panic!("the answer's task is the hunt: {:?}", answer.task) };
        let quarry = hunt.quarry;
        assert!(parties[0].ids.contains(&quarry));
        // A second lend at the party joins the answer.
        Brain::lend(&mut pianist, 0, 2, &parties[0], &ours, &enemies, &speed, 45, &mut commands).expect("a second lend");
        assert_eq!((pianist.groups.len(), pianist.groups[1].members.len(), pianist.groups[1].lent.clone()), (2, 5, vec![("A".to_string(), 5)]));
        // The deal keeps the lent with the answer while it lives.
        brain.pianist = Some(pianist);
        brain.keep_groups(&tick(60, &ours, &enemies), &kit, &mut commands);
        let mut pianist = brain.pianist.take().unwrap();
        assert_eq!((pianist.groups[0].members.len(), pianist.groups[1].members.len()), (9, 5));
        // The engine says the quarry is lost: the answer ends, the failure is returned, and the next deal has all
        // fourteen with the lender again. An event for another group's hunt is not this one's.
        let other = micro::HuntEvent { group: "group_A".into(), quarry, why: "lost" };
        assert!(Brain::hunt_event(&pianist.groups[1], &other, 200, false).is_none());
        let event = micro::HuntEvent { group: "group_answer_party_12".into(), quarry, why: "lost" };
        let (text, failed) = Brain::hunt_event(&pianist.groups[1], &event, 200, false).expect("the end");
        assert!(text.starts_with("the answer to party_12 ended after") && text.contains("go back"), "{text}");
        assert_eq!(failed.map(|(p, _)| p).as_deref(), Some("party_12"));
        pianist.end_answer("answer_party_12");
        brain.pianist = Some(pianist);
        brain.keep_groups(&tick(210, &ours, &enemies), &kit, &mut commands);
        let pianist = brain.pianist.as_ref().unwrap();
        assert_eq!((pianist.groups.len(), pianist.groups[0].members.len()), (1, 14));
    }

    /// A soldier far from its group's place is coming to it, on one plain order to the place that is not sent again
    /// while the place stands; inside the gate it is of the body, and the hands order it nothing: a declared group's
    /// members are the lane's. Nothing of where it came from enters (the slots design §3).
    #[test]
    fn a_soldier_far_from_its_group_comes_on_one_order_and_is_of_it_inside_the_gate() {
        use super::super::fixtures::at;
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, mut ours, enemies, _parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let body: Vec<&OwnUnit> = ours[..12].iter().collect();
        let centre = centre_of(&body).unwrap();
        pianist.groups.push(Group::of("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(centre), 0));
        brain.pianist = Some(pianist);
        // Two stand far off, wherever they came from.
        let far = [ours[12].id, ours[13].id];
        for unit in &mut ours[12..] {
            unit.pos = at(centre.x - 4000.0, centre.z);
        }
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None, conversion: 0.0 } };
        let mut commands = Vec::new();
        brain.keep_groups(&tick(30, &ours, &enemies), &kit, &mut commands);
        let sent: Vec<UnitId> = commands.iter().filter_map(|c| if let Command::Fight { unit, to, .. } = c { (to.dist2d(centre) < 100.0).then_some(*unit) } else { None }).collect();
        assert_eq!(sent, far, "{commands:?}");
        let group = &brain.pianist.as_ref().unwrap().groups[0];
        assert_eq!((group.members.len(), group.coming.iter().map(|(id, _)| *id).collect::<Vec<_>>()), (12, far.to_vec()));
        // The lane is told who they are: it holds none of them back (`session.rs`).
        assert_eq!(brain.coming(), far.to_vec());
        // The place stands: nothing is sent again.
        let mut commands = Vec::new();
        brain.keep_groups(&tick(60, &ours, &enemies), &kit, &mut commands);
        assert!(commands.is_empty(), "{commands:?}");
        // One comes inside the gate: of the body, with no order of the hands.
        ours[12].pos = at(centre.x - 300.0, centre.z);
        let mut commands = Vec::new();
        brain.keep_groups(&tick(90, &ours, &enemies), &kit, &mut commands);
        assert!(commands.is_empty(), "{commands:?}");
        let group = &brain.pianist.as_ref().unwrap().groups[0];
        assert_eq!((group.members.len(), group.coming.len()), (13, 1));
        assert!(group.members.contains(&far[0]));
        assert_eq!(brain.coming(), vec![far[1]]);
    }

    /// The groups are the player's (the slots design §3.1): a seat with none declared has `group_A`; a declared
    /// group is made where the player said with its slots and holds there empty until the deal fills it, is resized
    /// by a new declaration, and ends when it is no longer declared, its soldiers dealt to what is left.
    #[test]
    fn the_player_declares_the_groups_and_a_seat_without_any_has_group_a() {
        use crate::strategist::shared::{GroupDecl, Shared};
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, enemies, _parties) = e3();
        let shared = std::sync::Arc::new(Shared::default());
        brain.strategist = Some(shared.clone());
        brain.pianist = Some(super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist"));
        let all: Vec<&OwnUnit> = ours.iter().collect();
        let centre = centre_of(&all).unwrap();
        brain.home = centre;
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None, conversion: 0.0 } };
        let names = |brain: &Brain| brain.pianist.as_ref().unwrap().groups.iter().map(|g| (g.name.clone(), g.slots.total(), g.members.len())).collect::<Vec<_>>();
        let mut commands = Vec::new();
        brain.keep_groups(&tick(30, &ours, &enemies), &kit, &mut commands);
        assert_eq!(names(&brain), [("A".to_string(), GROUP_COMPANIES * COMPANY_CAP, 14)]);
        // Two groups declared, a station of four far away among them: the nearest fill the army, the rest come to
        // the station, which holds where the player made it.
        let station = [centre.x + 5000.0, centre.z];
        *shared.groups.lock().unwrap() = [("group_army".to_string(), GroupDecl { any: 10, ..GroupDecl::default() }), ("group_guard".to_string(), GroupDecl { any: 4, at: Some(station), ..GroupDecl::default() })].into_iter().collect();
        brain.keep_groups(&tick(60, &ours, &enemies), &kit, &mut commands);
        assert_eq!(names(&brain), [("army".to_string(), 10, 10), ("guard".to_string(), 4, 0)]);
        let guard = &brain.pianist.as_ref().unwrap().groups[1];
        assert_eq!((guard.coming.len(), guard.place.x), (4, station[0]));
        assert!(matches!(guard.task, GroupTask::Hold { picked: false, .. }));
        // The station is resized to two: two of the four have no slot. Then it is ended: nobody comes to it.
        shared.groups.lock().unwrap().get_mut("group_guard").unwrap().any = 2;
        brain.keep_groups(&tick(90, &ours, &enemies), &kit, &mut commands);
        assert_eq!((brain.pianist.as_ref().unwrap().groups[1].coming.len(), brain.pianist.as_ref().unwrap().surplus.len()), (2, 2));
        shared.groups.lock().unwrap().remove("group_guard");
        brain.keep_groups(&tick(120, &ours, &enemies), &kit, &mut commands);
        assert_eq!(names(&brain), [("army".to_string(), 10, 10)]);
        assert_eq!(brain.pianist.as_ref().unwrap().surplus.len(), 4);
    }

    /// A ground group's attack is the lane's (chunk 3): the hands set the task and order no member, the party's
    /// sightings move the task's `at`, and a party out of sight for six seconds ends it in a hold.
    #[test]
    fn a_ground_attack_is_declared_and_the_hands_order_nobody() {
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, mut enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::of("A".into(), Domain::Ground, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        let mut commands = Vec::new();
        Brain::engage_group(&mut group, &parties[0], &ours[..12], None, 30, &mut commands);
        assert!(commands.is_empty(), "no order of the hands' for a declared attack: {commands:?}");
        assert!(group.declared() && matches!(group.task, GroupTask::Engage { .. }));
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None, conversion: 0.0 } };
        // The party runs 600 east: the task's `at` follows, no Fight is issued.
        for e in enemies.iter_mut().take(3) {
            e.pos.x += 600.0;
        }
        let mut commands = Vec::new();
        brain.keep_groups(&tick(90, &ours, &enemies), &kit, &mut commands);
        assert!(!commands.iter().any(|c| matches!(c, Command::Fight { .. } | Command::Attack { .. })), "{commands:?}");
        let at = match &brain.pianist.as_ref().unwrap().groups[0].task { GroupTask::Engage { at, .. } => *at, t => panic!("{t:?}") };
        assert!((at.x - 5102.0).abs() < 2.0, "the task follows the party: {at:?}");
        // Out of sight for six seconds: a hold.
        let gone: Vec<bot_protocol::EnemyUnit> = enemies.iter().skip(3).cloned().collect();
        brain.keep_groups(&tick(120, &ours, &gone), &kit, &mut Vec::new());
        brain.keep_groups(&tick(300, &ours, &gone), &kit, &mut Vec::new());
        assert!(matches!(brain.pianist.as_ref().unwrap().groups[0].task, GroupTask::Hold { .. }), "{:?}", brain.pianist.as_ref().unwrap().groups[0].task);
    }

    /// A shelling group is two formations in the lane (chunk 5): its artillery at the standoff under the group's name,
    /// the rest at the screen point under `_screen`; a plain walk is one.
    #[test]
    fn a_shelling_group_is_two_formations_in_the_lane() {
        use super::super::fixtures::{at, brain_of};
        let mut brain = brain_of(&["armart", "armflash"]);
        assert!(brain.artillery_def(UnitDefId(1)) && !brain.artillery_def(UnitDefId(2)), "the Shellshocker is artillery, the Blitz is not");
        for (id, def) in [(1, 1), (2, 1), (3, 2), (4, 2)] {
            brain.known_units.insert(UnitId(id), (UnitDefId(def), at(1000.0 + 50.0 * id as f32, 1000.0)));
        }
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::of("S".into(), Domain::Ground, (1..=4).map(UnitId).collect(), GroupTask::Move { to: at(3000.0, 1000.0), place: "standoff from party_1".into(), fight: true }, 0);
        group.shelling = true;
        group.screen = Some(at(3400.0, 1000.0));
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let declared = brain.declare_groups();
        let mut names: Vec<&str> = declared.iter().map(|(n, ..)| n.as_str()).collect();
        names.sort();
        assert_eq!(names, ["group_S", "group_S_screen"]);
        let members = |declared: &[crate::session::Declared], name: &str| declared.iter().find(|(n, ..)| n == name).map(|(_, m, _)| m.clone());
        assert_eq!(members(&declared, "group_S"), Some(vec![UnitId(1), UnitId(2)]));
        assert_eq!(members(&declared, "group_S_screen"), Some(vec![UnitId(3), UnitId(4)]));
        // The shell ends (a walk picked): one formation of all four, the screen gone.
        brain.pianist.as_mut().unwrap().groups[0].set_task(GroupTask::Move { to: at(3000.0, 1000.0), place: "spot_1".into(), fight: false }, 30);
        let declared = brain.declare_groups();
        assert_eq!(declared.iter().map(|(n, ..)| n.as_str()).collect::<Vec<_>>(), ["group_S"]);
        assert_eq!(members(&declared, "group_S").map(|m| m.len()), Some(4));
    }

    /// A group following a group is declared to the lane with its ward (the contact cleanup's §5.3): the lane keeps
    /// its end at the ward's anchor and off the ward's footprint. A follower of a builder names none: its end is the
    /// hands' point, the builder's place.
    #[test]
    fn a_following_group_names_its_ward_to_the_lane() {
        use super::super::fixtures::{at, brain_of};
        let mut brain = brain_of(&["armflash"]);
        for id in 1..=5 {
            brain.known_units.insert(UnitId(id), (UnitDefId(1), at(1000.0 + 50.0 * id as f32, 1000.0)));
        }
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.groups.push(Group::of("A".into(), Domain::Ground, vec![UnitId(1), UnitId(2)], GroupTask::Move { to: at(3000.0, 1000.0), place: "spot_1".into(), fight: true }, 0));
        pianist.groups.push(Group::of("B".into(), Domain::Ground, vec![UnitId(3), UnitId(4)], GroupTask::Follow { ward: Ward::Group("A".into()), name: "group_A".into(), at: at(1100.0, 1000.0), since: 0 }, 0));
        pianist.groups.push(Group::of("C".into(), Domain::Ground, vec![UnitId(5)], GroupTask::Follow { ward: Ward::Builder(UnitId(9)), name: "constructor_9".into(), at: at(1100.0, 1000.0), since: 0 }, 0));
        brain.pianist = Some(pianist);
        let declared = brain.declare_groups();
        let ward = |name: &str| declared.iter().find(|(n, ..)| n == name).and_then(|(_, _, order)| order.ward.clone());
        assert_eq!(ward("group_B").as_deref(), Some("group_A"));
        assert_eq!(ward("group_C"), None);
        assert_eq!(ward("group_A"), None);
    }

    /// A roving group is the lane's: the hands keep up no order for it, and its course word
    /// is roving (the rework of docs/design/2026-10-07-rove-approval.md: roving is a task, never a flag).
    #[test]
    fn a_roving_group_gets_no_order_from_the_hands() {
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, enemies, _parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::of("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        group.set_task(GroupTask::Rove { within: None }, 0);
        assert_eq!(group.course_word(), "roving");
        assert!(group.rejoin_orders(&[&ours[0]]).is_empty());
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, vehicle_constructor: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = Tick { frame: 60, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.clone(), allies: Vec::new(), enemies: enemies.clone(), wrecks: None, sensors: None, conversion: 0.0 } };
        let mut commands = Vec::new();
        brain.keep_groups(&tick, &kit, &mut commands);
        let group = &brain.pianist.as_ref().unwrap().groups[0];
        assert!(group.roving());
        assert!(commands.is_empty(), "{commands:?}");
        assert!(brain.journal.notes.iter().all(|n| n.source != "lane"));
    }

    /// H-HANDS-ARRIVAL: a walk has arrived when a company of `FRONT_MIN` or more stands arrived at the way's end,
    /// whatever the companies behind it do; the hold is at the arrived company nearest the end. A group without
    /// companies is read by its own flag. A straggler company arrived alone is nothing.
    #[test]
    fn a_walk_arrives_with_its_front_company_and_not_with_a_straggler() {
        use super::super::fixtures::at;
        use micro::{CompanyState, GroupState};
        let to = at(4352.0, 1904.0);
        let company = |anchor: Vec3, members: usize, arrived: bool| CompanyState { anchor, members, formed: members, exceptions: Vec::new(), arrived };
        let state = |companies: Vec<CompanyState>, arrived: bool| GroupState { anchor: at(4664.0, 2024.0), heading: (1.0, 0.0), pace: 0.0, full_pace: 75.0, formed: Vec::new(), exceptions: Vec::new(), arrived, legs_left: 0, engaged: None, way: Some(0.0), companies, passing: None };
        // The game's 10:40: the lead company of 18 arrived 334 past the spot, a company of 4 marching far behind;
        // the composed flag false.
        let front = company(at(4664.0, 2024.0), 18, true);
        let s = state(vec![company(at(1500.0, 3000.0), 4, false), front.clone()], false);
        assert_eq!(front_arrived(&s, to), Some(front.anchor));
        // Two arrived companies: the hold is at the one nearer the end.
        let nearer = company(at(4400.0, 1950.0), 14, true);
        let s = state(vec![front.clone(), nearer.clone(), company(at(2500.0, 5000.0), 22, false)], false);
        assert_eq!(front_arrived(&s, to), Some(nearer.anchor));
        // A straggler of three arrived alone: no arrival.
        let s = state(vec![company(at(4400.0, 1950.0), 3, true), company(at(2500.0, 5000.0), 22, false)], false);
        assert_eq!(front_arrived(&s, to), None);
        // No companies (a group with no cap): the formation's own flag.
        assert_eq!(front_arrived(&state(Vec::new(), true), to), Some(at(4664.0, 2024.0)));
        assert_eq!(front_arrived(&state(Vec::new(), false), to), None);
    }
}
