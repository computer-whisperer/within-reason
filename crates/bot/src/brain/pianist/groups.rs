//! Soldier groups: the pianist's unit of command. A new soldier joins the group the player named for its factory's
//! output, else its factory's own group; nothing merges by proximity (H-HANDS-GROUPS, since 2026-09-25: groups are
//! the player's, `docs/design/2026-09-25-one-decider.md` §3). A group carries one task at a time; between calls the
//! standing orders are kept up (a march arrives together, an engagement follows its party, an arrival becomes a hold).

use std::collections::{BTreeMap, HashMap};

use bot_protocol::{Command, Lane, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::Party;
use crate::world::Domain;

/// H-HANDS-GROUPS: a newcomer whose group stands farther than this walks to it (fresh Grunts at the lab formed groups of
/// one and two while the ball held 400 to 2,700 away, and died in ones and twos: human-8, eight Grunts lost by 3:14,
/// every one in a group of five or fewer, 380 to 1,231 from the commander).
const ADOPT_RADIUS: f32 = 400.0;
/// A moving group has arrived when its centre is this close to its destination.
const ARRIVED: f32 = 300.0;
/// Enemies this close to a group's body are what it met since its last stop (the route's `met` fact).
const MET_REACH: f32 = 1200.0;
/// Places reached kept per group, the oldest dropped: the words stay short.
const REACHED_KEPT: usize = 6;
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
/// march), until the quarry dies, is out of sight and radar for `HUNT_LOST_FRAMES`, or the leash ends. Whether the
/// hunters can catch or survive the quarry is said in the state's words for the pick; the engine drops nobody. The
/// group holds where the hunt ends; the merge back is a state Jev picks or the player asks for.
#[derive(Clone, Debug)]
pub(crate) struct Hunt {
    pub quarry: UnitId,
    pub party: String,
    /// Where each hunter stood when the hunt began: the leash is measured from there, hunter by hunter (the
    /// impedance tally, 2026-10-16: measured from the hunters' centre, a detachment cut from a strung-out body or
    /// sent from afar tripped it on its first tick, 174 of 395 leash ends "after 0 s" in 23 games).
    pub anchors: Vec<(UnitId, Vec3)>,
    pub since: i32,
    pub last_seen: i32,
    /// Where the quarry was last seen: the attack goes there when it is on radar only.
    pub at: Vec3,
}

/// A hunt this far from where it began ends (threats-smoke-1: anchored on the group's station, a hunt begun 900
/// from the station ended on its first tick, 12 of 16 hunts one order long). The engine judges it (H-MICRO-HUNT).
pub(crate) const HUNT_LEASH: f32 = 900.0;
/// A follower is sent after its ward again when the ward has moved this far from where the follower was last sent.
const FOLLOW_STEP: f32 = 150.0;
/// A newcomer waiting for its fellows (`produce` ... `join_in`) stands this far past its plant's lane edge.
const WAIT_BESIDE: f32 = 80.0;

/// Where a newcomer waits for its fellows: beside its plant's exit lane, out of it, on the side toward where it will
/// walk (`toward`: its group's body).
pub(super) fn beside_lane(lane: &Lane, toward: Vec3) -> Vec3 {
    let (dx, dz) = (lane.to.x - lane.from.x, lane.to.z - lane.from.z);
    let len = dx.hypot(dz).max(1.0);
    let (px, pz) = (-dz / len, dx / len);
    let side = if (toward.x - lane.from.x) * px + (toward.z - lane.from.z) * pz >= 0.0 { 1.0 } else { -1.0 };
    let off = (lane.half_width + WAIT_BESIDE) * side;
    Vec3 { x: lane.from.x + px * off, y: lane.from.y, z: lane.from.z + pz * off }
}

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
    /// `committed`: the hold an advance arrives in, still committed to everything there (H-HANDS-GROUPS).
    /// `picked`: a hold Jev picked, which is a course; a hold an order ended in is none, and the group is idle.
    /// `at`: where the lane's anchor stands for the hold (its Hold route; the ranks hang from it): the anchor as it is
    /// when an arrival ends in a hold or a hold is picked, the body's place when the group forms (the migration
    /// design, chunk 2).
    Hold { committed: bool, picked: bool, at: Vec3 },
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
    /// unguarded and keeping out of every reach. A pick like every task (the `rove` move, or `rove_1` for the
    /// detachment it makes); the way back is any other pick (docs/design/2026-10-07-rove-approval.md, the rework).
    /// `within`: a region the player marked, outside which the rovers look at nothing and attack nothing (the
    /// `rove_<region>` move; docs/design/2026-10-17-rove-destinations.md).
    Rove { within: Option<String> },
}

#[derive(Debug)]
pub(crate) struct Group {
    pub name: String,
    /// Its members' movement domain (docs/design/2026-09-22-domains.md): a group never mixes air and ground.
    pub domain: Domain,
    pub members: Vec<UnitId>,
    pub task: GroupTask,
    pub last_order: i32,
    /// The group this one was split from (a detachment, a split, a scout): said in the picture on both sides, so
    /// the player sees an army in pieces as it happens (escalate-6: seven detachments died one by one).
    pub parent: Option<String>,
    /// Soldiers lost in the last minute (frame, type); the turn frame the wake counts from, and whether the player has
    /// been woken for them this turn (H-HANDS-LOSS-WAKE). The picture says the last 30 s.
    /// (frame, the type lost, the killer and its type when known): the picture's `losses` line says to whom.
    pub losses: Vec<(i32, UnitDefId, Option<(UnitId, UnitDefId)>)>,
    pub losses_since: i32,
    pub loss_warned: bool,
    /// Members still on their way to join (a plant's output walking to the body, the soldiers of a group that
    /// joined this one), each with where it was last sent: not the front, not the tail, not counted as arrived;
    /// said in the picture as reinforcements on the way (H-HANDS-GROUP-BODY). They are sent after the body while it
    /// moves and take the group's order when they reach it (H-HANDS-JOINERS).
    pub joining: HashMap<UnitId, Vec3>,
    /// Of the joiners, the newcomers waiting beside their plant for enough fellows to walk together (`produce` ...
    /// `join_in`): (the plant, the soldier). Not sent after the body and not given the group's orders until
    /// `release_waiting` sends them.
    pub waiting: Vec<(UnitId, UnitId)>,
    /// Shelling a party from a standoff: the long-reach members at the standoff, the rest between (`shell`).
    pub shelling: bool,
    /// The named places its body has come within `ARRIVED` of, in order, with the frame: facts said in the picture
    /// and in `recent` and never acted on by code (the registers; docs/design/2026-09-28-routes-in-prose.md §4.2).
    pub reached: Vec<(String, i32)>,
    /// The first enemies in sight near the body since the last place reached: the frame and the words.
    pub met: Option<(i32, String)>,
    /// What its rovers found, attacked, ran from and gave up, newest last, for the picture: (frame, words).
    pub rove_log: Vec<(i32, String)>,
}

/// A group's shape on the ground this second (H-HANDS-GROUP-BODY): the core (members not still joining), its front
/// (the member nearest `toward`: the goal, else the nearest enemy) and tail (the farthest), the body's position
/// (the `BODY_SHARE`-th nearest member, as the march measures it), and how many of the core stand within `ARRIVED`
/// of the goal. A group was a point before: its centroid stood in deep water when the group straddled a cliff
/// (Cape Violet 9:40-11:17), "advancing, 934 to go" was said of a column strung 2,000-2,900 long whose head was
/// already dying under turrets (game 3 8:35), and the odds were priced on all 48 when 17 were in the fight.
pub(crate) struct Body<'a> {
    pub core: Vec<&'a OwnUnit>,
    pub joining: Vec<&'a OwnUnit>,
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

    pub(crate) fn new(name: String, domain: Domain, members: Vec<UnitId>, task: GroupTask, frame: i32) -> Group {
        Group { name, domain, members, task, last_order: frame, parent: None, losses: Vec::new(), losses_since: frame, loss_warned: false, joining: HashMap::new(), waiting: Vec::new(), shelling: false, reached: Vec::new(), met: None, rove_log: Vec::new() }
    }

    /// Whether the group is the lane's as a formation (the migration design, chunk 2): a ground group holding or
    /// walking, not shelling (shell is the hands' until its chunk). Its members get their orders from the lane's
    /// group, never from the hands: no per-unit order, no joiners' walk, arrival from the lane's state.
    pub(crate) fn declared(&self) -> bool {
        self.domain == Domain::Ground && matches!(self.task, GroupTask::Hold { .. } | GroupTask::Move { .. } | GroupTask::Engage { .. }) && !self.shelling
    }

    /// The group's name in the lane.
    pub(crate) fn lane_name(&self) -> String {
        format!("group_{}", self.name)
    }

    /// The group's body toward `toward` (the goal of a walk, the nearest enemy, or nothing: then the front is the
    /// centre and the tail the member farthest from it). `None` when no member stands.
    pub(crate) fn body<'a>(&self, own: &'a [OwnUnit], toward: Option<Vec3>) -> Option<Body<'a>> {
        self.body_with(own, toward, super::layers::stragglers())
    }

    /// `body`, with the `stragglers` layer on or off.
    pub(crate) fn body_with<'a>(&self, own: &'a [OwnUnit], toward: Option<Vec3>, stragglers: bool) -> Option<Body<'a>> {
        let units = self.units(own);
        let (joining, mut core): (Vec<&OwnUnit>, Vec<&OwnUnit>) = units.iter().copied().partition(|u| self.joining.contains_key(&u.id));
        if core.is_empty() {
            // Everybody is still on the way: the body is wherever they are.
            core = joining.clone();
        }
        let centre = centre_of(&core)?;
        let toward = toward.unwrap_or(centre);
        core.sort_by(|a, b| a.pos.dist2d(toward).total_cmp(&b.pos.dist2d(toward)));
        let place = |core: &[&OwnUnit]| core.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len() - 1)).map_or(centre, |u| u.pos);
        let joining: Vec<&OwnUnit> = joining.into_iter().filter(|u| !core.iter().any(|c| c.id == u.id)).collect();
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
        Some(Body { length: front.dist2d(tail), core, joining, stragglers: apart, front, tail, at, arrived })
    }

    /// A newcomer (a plant's output) joins: of the body when it stands within the adopt radius of the body's place,
    /// with the task's order at once; else a reinforcement on its way to the nearest member of the body's core, on
    /// ground it reaches (`snap`), never the arithmetic centre (Cape Violet: the centre of a group split between a
    /// plateau and the shore below lay in deep water, and every newcomer was sent into it). By the nearest member
    /// alone a plant's stream at the yard adopted itself one unit at a time and the body's tail never left home
    /// (player-10 18:20: "its tail is 4649 behind at yard2"); adopted with no order, a Rascal stood idle at the yard
    /// 47 s while its group walked on, and the next newcomer was sent to it as the nearest member (human-27 1:47-2:34).
    /// Under `wait` (the plant, how many walk together, where to stand) a newcomer far from the body waits beside
    /// its plant instead, until `release_waiting` sends it with its fellows.
    ///
    /// A declared group (the lane's formation) walks no joiner: the newcomer is a member, the lane gives it a slot
    /// from the next tick and says `joining` until it is formed; under `wait` it waits as any newcomer does, out of
    /// the lane until released.
    pub(crate) fn admit(&mut self, unit: &OwnUnit, own: &[OwnUnit], snap: impl Fn(Vec3) -> Vec3, wait: Option<(UnitId, usize, Vec3)>) -> Vec<Command> {
        let declared = self.declared();
        let body = self.body(own, None);
        let adopted = !declared && body.as_ref().is_some_and(|b| b.at.dist2d(unit.pos) <= ADOPT_RADIUS);
        self.members.push(unit.id);
        if adopted {
            return self.rejoin_orders(&[unit]);
        }
        if let Some((plant, n, beside)) = wait
            && n >= 2
            && (declared || body.as_ref().is_none_or(|b| b.at.dist2d(unit.pos) > ADOPT_RADIUS))
        {
            let to = snap(beside);
            if !declared {
                self.joining.insert(unit.id, to);
            }
            self.waiting.push((plant, unit.id));
            return vec![Command::Move { unit: unit.id, to, queue: false }];
        }
        if declared {
            return Vec::new();
        }
        match self.nearest_of_body(unit, own) {
            Some(to) => {
                let to = snap(to);
                self.joining.insert(unit.id, to);
                vec![Command::Move { unit: unit.id, to, queue: false }]
            }
            None => Vec::new(),
        }
    }

    /// Where a joiner is sent: the nearest other member of the body's core, else the nearest other member not
    /// itself joining.
    fn nearest_of_body(&self, joiner: &OwnUnit, own: &[OwnUnit]) -> Option<Vec3> {
        let from = joiner.pos;
        let body = self.body(own, None);
        let core_nearest = body.as_ref().and_then(|b| b.core.iter().filter(|m| m.id != joiner.id).map(|m| m.pos).min_by(|a, b| a.dist2d(from).total_cmp(&b.dist2d(from))));
        core_nearest.or_else(|| self.units(own).iter().filter(|m| m.id != joiner.id && !self.joining.contains_key(&m.id)).map(|m| m.pos).min_by(|a, b| a.dist2d(from).total_cmp(&b.dist2d(from))))
    }

    /// The newcomers waiting beside `plant` walk to the body together (`join_in`), as joiners from here.
    pub(crate) fn release_waiting(&mut self, plant: UnitId, own: &[OwnUnit], snap: impl Fn(&OwnUnit, Vec3) -> Vec3) -> Vec<Command> {
        let mine: Vec<UnitId> = self.waiting.iter().filter(|(p, _)| *p == plant).map(|(_, id)| *id).collect();
        self.waiting.retain(|(p, _)| *p != plant);
        let mut commands = Vec::new();
        if self.declared() {
            // Members of the lane's group from the next tick: a slot each, no walk of the hands'.
            return commands;
        }
        for id in mine {
            let Some(unit) = own.iter().find(|u| u.id == id) else { continue };
            let Some(to) = self.nearest_of_body(unit, own) else { continue };
            let to = snap(unit, to);
            self.joining.insert(id, to);
            commands.push(Command::Move { unit: id, to, queue: false });
        }
        commands
    }

    /// Orders for members rejoining the task (a newcomer that has reached the body): what the task would give
    /// them now.
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

    /// The soldiers lost since `since`, and their metal.
    pub(crate) fn lost_since(&self, since: i32, world: &crate::world::World) -> (usize, f32) {
        let lost: Vec<&(i32, UnitDefId, Option<(UnitId, UnitDefId)>)> = self.losses.iter().filter(|(f, ..)| *f >= since).collect();
        (lost.len(), lost.iter().filter_map(|(_, def, _)| world.def(*def)).map(|d| d.metal_cost).sum())
    }

    /// A group split from another.
    pub(crate) fn split_from(mut self, parent: &str) -> Group {
        self.parent = Some(parent.to_string());
        self
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
    }
}

impl GroupTask {
    /// The hold an order ends in, or a group forms in: nobody picked it, nothing is committed.
    pub(crate) fn idle_hold(at: Vec3) -> GroupTask {
        GroupTask::Hold { committed: false, picked: false, at }
    }
}

pub(crate) fn centre_of(units: &[&OwnUnit]) -> Option<Vec3> {
    if units.is_empty() {
        return None;
    }
    let n = units.len() as f32;
    Some(units.iter().fold(Vec3::default(), |sum, u| Vec3 { x: sum.x + u.pos.x / n, y: 0.0, z: sum.z + u.pos.z / n }))
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
            for (f, id, def, killer) in self.unit_losses.iter().rev().take_while(|(f, ..)| *f == frame) {
                if group.members.contains(id) {
                    group.losses.push((*f, *def, *killer));
                }
            }
            group.members.retain(|id| soldiers.iter().any(|u| u.id == *id));
            let standing: f32 = group.units(own).iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            let (count, lost) = group.lost_since(turn_frame, &self.world);
            if !group.loss_warned && !group.members.is_empty() && (count >= LOSS_WAKE_COUNT || lost >= LOSS_WAKE_SHARE * (standing + lost)) {
                group.loss_warned = true;
                let doing = group.course_word();
                loss_wakes.push(format!(
                    "group_{} has lost {} of its {} soldiers ({lost:.0} metal) since your last orders, {doing} at {}",
                    group.name,
                    count,
                    group.members.len() + count,
                    self.world.grid(centre_of(&group.units(own)).unwrap_or_default())
                ));
            }
        }
        pianist.groups.retain(|g| !g.members.is_empty());
        if let Some(shared) = &self.strategist {
            for text in loss_wakes {
                shared.trigger(text);
            }
        }
        // H-HANDS-GROUPS: a newcomer joins the group the player named for its factory (`produce` ... `group`), else its
        // factory's own group (made on its first soldier, remade when it has died out); a soldier of no factory (the
        // start, a resurrection) forms a group of its own. Nothing merges by proximity: a merge is a move Jev picks
        // (`join`). Standing-2: 22 groups lived a median 1.3 minutes under the old
        // adoption and merge, and the tool's rules died with them.
        let mut loose: Vec<&OwnUnit> = soldiers.iter().copied().filter(|u| !pianist.groups.iter().any(|g| g.members.contains(&u.id))).collect();
        loose.sort_by_key(|u| u.id.0);
        // Soldiers given by another seat this tick form one group of their own (the `transfer` tool), by domain.
        let given: Vec<&OwnUnit> = Brain::given_soldiers(tick, &loose);
        if !given.is_empty() {
            let mut by_domain: Vec<(Domain, Vec<UnitId>)> = Vec::new();
            for u in &given {
                let domain = self.world.domain_of(u.def);
                match by_domain.iter_mut().find(|(d, _)| *d == domain) {
                    Some((_, members)) => members.push(u.id),
                    None => by_domain.push((domain, vec![u.id])),
                }
            }
            for (domain, members) in by_domain {
                let name = pianist.new_group_name();
                let units: Vec<&OwnUnit> = given.iter().copied().filter(|u| members.contains(&u.id)).collect();
                let group = Group::new(name.clone(), domain, members, GroupTask::idle_hold(centre_of(&units).unwrap_or_default()), frame);
                commands.extend(group.hold_orders(&units));
                pianist.groups.push(group);
                pianist.done.push(format!("{} group_{name} formed from the {} soldiers given to this seat", super::picture::clock(frame), units.len()));
            }
            loose.retain(|u| !given.iter().any(|g| g.id == u.id));
        }
        let mut formed: Vec<String> = Vec::new();
        for unit in loose {
            let domain = self.world.domain_of(unit.def);
            let factory = pianist.produced_by.get(&unit.id).copied();
            let allowance = factory.and_then(|f| self.allowed_units(&self.actor_name(f)));
            let named = allowance.as_ref().and_then(|a| a.group.clone());
            let call = allowance.as_ref().map_or(0, |a| a.call);
            // `by_type`: the factory's group for this unit type, kept apart from its other types'.
            let by_type = named.as_deref() == Some("by_type");
            let rally_key = factory.map(|f| (f, by_type.then_some(unit.def)));
            let wanted = group_for_newcomer(named.as_deref(), call, rally_key.and_then(|k| pianist.rally.get(&k)).map(|(c, g)| (*c, g.as_str())));
            let target = wanted.and_then(|name| pianist.groups.iter().position(|g| g.name == name && g.domain == domain));
            match target {
                Some(i) => {
                    let walker = self.walker_of(unit.def);
                    // `join_in`: it waits beside its plant, out of the lane, for its fellows.
                    let wait = allowance.as_ref().and_then(|a| a.join_in).zip(factory).and_then(|(n, f)| {
                        let plant = own.iter().find(|u| u.id == f)?;
                        let lane = self.lane_of(plant)?;
                        let toward = pianist.groups[i].body(own, None).map_or(plant.pos, |b| b.at);
                        Some((f, n, beside_lane(&lane, toward)))
                    });
                    commands.extend(pianist.groups[i].admit(unit, own, |to| self.snap_for(walker, to), wait));
                }
                None => {
                    let name = pianist.new_group_name();
                    if let Some(key) = rally_key {
                        // `new` asked for one fresh group, not one per soldier: the call's later soldiers join this one.
                        pianist.rally.insert(key, (call, name.clone()));
                        formed.push(format!(
                            "group_{name} formed: {}'s first {}{}",
                            self.actor_name(key.0),
                            self.name(unit.def),
                            if by_type { " (one group per unit type)" } else { "" }
                        ));
                    }
                    let group = Group::new(name, domain, vec![unit.id], GroupTask::idle_hold(unit.pos), frame);
                    commands.extend(group.hold_orders(&[unit]));
                    pianist.groups.push(group);
                }
            }
        }
        for text in formed {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            pianist.note(frame, text);
        }
        // Standing orders, under each group's footwork rules (H-HANDS-LANE); a declared group's are the lane's.
        let mut hunt_ends: Vec<String> = Vec::new();
        let mut route_news: Vec<String> = Vec::new();
        let mut follow_ends: Vec<String> = Vec::new();
        // Where each group's body stands, for the groups that follow another.
        let bodies: Vec<(String, Vec3)> = pianist.groups.iter().filter_map(|g| g.body(own, None).map(|b| (g.name.clone(), b.at))).collect();
        for group in pianist.groups.iter_mut() {
            // A roving group is the lane's (H-MICRO-ROVE): no hunt, no orders kept up, every member its own from the
            // tick it joins.
            if group.roving() {
                group.joining.clear();
                continue;
            }
            let units = group.units(own);
            group.joining.retain(|id, _| group.members.contains(id));
            group.waiting.retain(|(_, id)| group.members.contains(id));
            // The newcomers waiting beside a plant (`join_in`) walk together once enough of them stand, or at once
            // when the plant's allowance no longer asks it.
            let plants: Vec<UnitId> = group.waiting.iter().map(|(p, _)| *p).fold(Vec::new(), |mut v, p| {
                if !v.contains(&p) {
                    v.push(p);
                }
                v
            });
            for plant in plants {
                let asked = self.allowed_units(&self.actor_name(plant)).and_then(|a| a.join_in);
                let standing = group.waiting.iter().filter(|(p, _)| *p == plant).count();
                if asked.is_none_or(|n| standing >= n) {
                    commands.extend(group.release_waiting(plant, own, |u, to| self.snap_for(self.walker_of(u.def), to)));
                    route_news.push(format!("group_{}: {standing} new soldiers walk to it together from {}", group.name, self.actor_name(plant)));
                }
            }
            // A newcomer that has reached the body is of it (or was sent nowhere: nothing stands to reach).
            let goal = match &group.task { GroupTask::Move { to, .. } => Some(*to), GroupTask::Engage { at, .. } | GroupTask::Follow { at, .. } => Some(*at), GroupTask::Hunt(h) => Some(h.at), GroupTask::Hold { .. } | GroupTask::Rove { .. } => None };
            if let Some(body) = group.body(own, goal) {
                let core_at = body.at;
                let joined: Vec<UnitId> = body.joining.iter().filter(|u| u.pos.dist2d(core_at) <= ADOPT_RADIUS || self.stuck.contains_key(&u.id)).map(|u| u.id).collect();
                for id in joined {
                    group.joining.remove(&id);
                    group.waiting.retain(|(_, w)| *w != id);
                    // Onto the task with the rest.
                    if let Some(u) = units.iter().find(|u| u.id == id) {
                        commands.extend(group.rejoin_orders(&[u]));
                    }
                }
                // The rest go after the body: each was sent to where a member stood, and a body that has moved on
                // since left them standing at an empty place (player-35 7:46-8:28: seven Blitzes that joined a
                // group of two walked to where the two had stood and waited there with no order for 35 s).
                let core: Vec<Vec3> = body.core.iter().filter(|u| !group.joining.contains_key(&u.id)).map(|u| u.pos).collect();
                for unit in &units {
                    if group.waiting.iter().any(|(_, w)| *w == unit.id) {
                        continue;
                    }
                    let Some(sent) = group.joining.get_mut(&unit.id) else { continue };
                    let Some(nearest) = core.iter().copied().min_by(|a, b| a.dist2d(unit.pos).total_cmp(&b.dist2d(unit.pos))) else { continue };
                    if let Some(to) = follow_step(nearest, *sent) {
                        *sent = to;
                        commands.push(Command::Move { unit: unit.id, to, queue: false });
                    }
                }
            }
            // The newcomers still waiting beside their plant (`join_in`) take none of the task's orders below.
            let units: Vec<&OwnUnit> = units.into_iter().filter(|u| !group.waiting.iter().any(|(_, w)| *w == u.id)).collect();
            // The body's place, not the centre of every member: the stream still joining from the plants would drag
            // the centre home (player-10 18:20: a front at spot_40 with 34 waiting at the yard read "0 of 59 arrived").
            // A declared group's place is its anchor, the lane's.
            let lane_state = if group.declared() { self.lane.group_state(&group.lane_name()).cloned() } else { None };
            let Some(centre) = lane_state.as_ref().map(|s| s.anchor).or_else(|| group.body(own, goal).map(|b| b.at)) else { continue };
            // The registers' facts: a named place (`menu::named_places`) the body comes within `ARRIVED` of is
            // reached, said in `recent` and asked about at once; the first enemies in sight since the last stop are
            // kept until the next. Code records; Jev picks the next move.
            let here = pianist.places.iter().find(|p| pianist.named.contains(&p.name) && p.at.dist2d(centre) < ARRIVED).map(|p| p.name.clone());
            if let Some(place) = here
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
                GroupTask::Move { to, fight, .. } if declared => {
                    // The lane's arrival (the anchor at the way's end, every member not an exception in its slot),
                    // read a second after the order at the earliest: the state before it is the last task's.
                    if frame - group.last_order >= FRAMES_PER_SECOND && let Some(state) = lane_state.as_ref().filter(|s| s.arrived) {
                        // An arrival ends the order in a hold nobody picked, the anchor where it stands: the group
                        // is idle and is asked.
                        let (committed, at) = (*fight, state.anchor);
                        group.set_task(GroupTask::Hold { committed, picked: false, at }, frame);
                    }
                }
                GroupTask::Move { to, fight, .. } => {
                    // An air group, or a shelling one, until their chunks: arrival by the members, not the centre (a
                    // group split by a cliff never got its centre within 300 of its goal, so a fall-back walk stayed
                    // the base world for two minutes and the player's new stations were shown as the default and
                    // never played: Cape Violet 9:40-11:17).
                    let joining = group.joining.clone();
                    let moving: Vec<&OwnUnit> = units.iter().copied().filter(|u| !joining.contains_key(&u.id) && !self.stuck.contains_key(&u.id)).collect();
                    let core = if moving.is_empty() { units.clone() } else { moving };
                    let mut distances: Vec<f32> = core.iter().map(|u| u.pos.dist2d(*to)).collect();
                    distances.sort_by(f32::total_cmp);
                    let to_go = distances.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len().saturating_sub(1))).copied().unwrap_or(0.0);
                    if to_go < ARRIVED {
                        let (committed, at) = (*fight, *to);
                        group.set_task(GroupTask::Hold { committed, picked: false, at }, frame);
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
                            commands.extend(units.iter().filter(|u| !group.joining.contains_key(&u.id)).map(|u| Command::Fight { unit: u.id, to, queue: false }));
                        }
                    } else if frame - *last_seen > LOST_FRAMES {
                        commands.extend(group.hold_orders(&units));
                        group.task = GroupTask::idle_hold(centre);
                    }
                }
                // The hunt's book: the quarry's last sighting for the words; the chase and its ends are the engine's
                // (H-MICRO-HUNT). Every hunter dead ends it; the empty group goes at the next look.
                GroupTask::Hunt(hunt) => {
                    if let Some(q) = enemies.iter().find(|e| e.id == hunt.quarry) {
                        hunt.last_seen = frame;
                        hunt.at = q.pos;
                    }
                    if units.is_empty() {
                        hunt_ends.push(format!("group_{}'s hunt of {} ended after {} s: every hunter dead", group.name, hunt.party, (frame - hunt.since) / FRAMES_PER_SECOND));
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
                                commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to, queue: false }));
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
                    micro::RoveWhat::Attacks { def, at, .. } => {
                        let text = format!("{clock} group_{}'s rover attacks his unguarded {} at {}", group.name, self.short_words(*def), self.place_words(&pianist.places, *at));
                        pianist.done.push(text.clone());
                        text
                    }
                    micro::RoveWhat::Evades { from, at } => format!("{clock} ran from {} at {}", from.map_or("a radar contact".to_string(), |d| format!("his {}", self.short_words(d))), self.place_words(&pianist.places, *at)),
                    micro::RoveWhat::GivesUp { goal, why } => format!("{clock} gave up looking at {goal}: {why}"),
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
    /// leaves the group holding where it stands, for Jev or the player to send on or merge. Returns the end's words
    /// and, for a hunt that ended without a kill, the failure to remember (the party and why).
    pub(crate) fn hunt_event(group: &mut Group, event: &micro::HuntEvent, own: &[OwnUnit], frame: i32, cloaks: bool, commands: &mut Vec<Command>) -> Option<(String, Option<(String, String)>)> {
        let GroupTask::Hunt(hunt) = &mut group.task else { return None };
        // The event names the lane's group, `group_<name>` (D15): two groups hunting one quarry end one at a time.
        if hunt.quarry != event.quarry || event.group != format!("group_{}", group.name) {
            return None;
        }
        let (party, since) = (hunt.party.clone(), hunt.since);
        let units = group.units(own);
        commands.extend(group.hold_orders(&units));
        group.set_task(GroupTask::idle_hold(centre_of(&units).unwrap_or_default()), frame);
        let why = match event.why {
            "dead" => "the quarry is dead".to_string(),
            "lost" if cloaks => format!("{party} hidden: it cloaks, and nothing of ours has seen it for {} s, its last place walked to", HUNT_HIDDEN_FRAMES / FRAMES_PER_SECOND),
            "lost" => format!("{party} out of sight for 6 s"),
            "leash" => format!("the leash of {HUNT_LEASH:.0} from where it began reached"),
            other => other.to_string(),
        };
        let failed = (event.why != "dead").then(|| (party.clone(), why.clone()));
        Some((format!("group_{}'s hunt of {party} ended after {} s: {why}; it holds where it is ({} soldiers)", group.name, (frame - since) / FRAMES_PER_SECOND, units.len()), failed))
    }

    /// A hunt is a group of its own (the user, 2026-09-29): the hunters split off from `pianist.groups[index]` under
    /// a new name with the hunt as their task, unless the whole group hunts; the engine's commitment does the
    /// chasing (`micro.rs`). Returns what was done.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_hunt(pianist: &mut super::Pianist, index: usize, hunters: &[UnitId], party: &Party, own: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit], frame: i32, commands: &mut Vec<Command>) -> Option<String> {
        let all: Vec<&OwnUnit> = pianist.groups[index].units(own);
        let chosen: Vec<&OwnUnit> = all.iter().copied().filter(|u| hunters.contains(&u.id)).collect();
        if chosen.is_empty() {
            return None;
        }
        let hunt = Brain::hunt_of(party, &chosen, enemies, frame);
        if chosen.len() == all.len() {
            let group = &mut pianist.groups[index];
            commands.extend(group.release_orders(&chosen));
            group.set_task(GroupTask::Hunt(hunt), frame);
            return Some(format!("hunt {} ({}) with the whole group", party.name, party.composition));
        }
        let ids: Vec<UnitId> = chosen.iter().map(|u| u.id).collect();
        let name = pianist.new_group_name();
        let group = &mut pianist.groups[index];
        commands.extend(group.release_orders(&chosen));
        let (domain, parent_name) = (group.domain, group.name.clone());
        if ids.len() * 2 >= all.len() {
            // At least half the group goes: the hunters keep its name and the rest are the new group, so the name
            // the player's packet uses stays with the larger body; the soldiers on their way to join it go on to the
            // rest, which keeps the course (the user, 2026-10-17; predict-sweep-1 seed 8 at 6:53: 8 of 11 left as
            // group_F, and the packet's "group_E, the army" named the one Lasher left behind).
            let rest: Vec<UnitId> = group.members.iter().copied().filter(|id| !ids.contains(id)).collect();
            let old_task = std::mem::replace(&mut group.task, GroupTask::Hunt(hunt));
            let joining = std::mem::take(&mut group.joining);
            let waiting = std::mem::take(&mut group.waiting);
            group.members = ids;
            group.shelling = false;
            let did = format!("{} of group_{parent_name} hunt {} ({}) under its name; the {} left behind are group_{name}", group.members.len(), party.name, party.composition, rest.len());
            let mut left = Group::new(name, domain, rest, old_task, frame).split_from(&parent_name);
            left.joining = joining;
            left.waiting = waiting;
            pianist.groups.push(left);
            return Some(did);
        }
        group.members.retain(|id| !ids.contains(id));
        let did = format!("{} of group_{parent_name} hunt {} ({}) as group_{name}", ids.len(), party.name, party.composition);
        pianist.groups.push(Group::new(name, domain, ids, GroupTask::Hunt(hunt), frame).split_from(&parent_name));
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
        Hunt { quarry: quarry.0, party: party.name.clone(), anchors: hunters.iter().map(|u| (u.id, u.pos)).collect(), since: frame, last_seen: frame, at: quarry.1 }
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

/// The group a newcomer wants (H-HANDS-GROUPS): the one the player named for its factory; for `new`, the factory's
/// own group when that was founded under this `produce` call, else none (a fresh group, which founds it); for
/// `by_type`, the factory's group for this unit type (`factory_group` is then looked up by the type), kept like the
/// factory's own; else the factory's own; a soldier of no factory forms its own.
fn group_for_newcomer(named: Option<&str>, call: u64, factory_group: Option<(u64, &str)>) -> Option<String> {
    match named {
        Some("new") => factory_group.filter(|(founded, _)| *founded == call).map(|(_, g)| g.to_string()),
        Some("by_type") | None => factory_group.map(|(_, g)| g.to_string()),
        Some(name) => Some(name.trim_start_matches("group_").to_string()),
    }
}

#[cfg(test)]
mod tests {
    /// A hunt's leash is anchored hunter by hunter, where each stood when it began: a hunter far from the others'
    /// centre does not end the hunt on its first tick.
    #[test]
    fn a_hunt_anchors_its_leash_on_each_hunter() {
        let (_, ours, enemies, parties) = e3();
        let hunters: Vec<&bot_protocol::OwnUnit> = ours.iter().take(3).collect();
        let hunt = Brain::hunt_of(&parties[0], &hunters, &enemies, 30);
        assert_eq!(hunt.anchors, hunters.iter().map(|u| (u.id, u.pos)).collect::<Vec<_>>());
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
        let group = Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
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

    /// A hunt is a group of its own: three of fourteen sent after a party split off under a new name with the hunt
    /// as their task, the parent keeps eleven; the whole of a group hunting keeps its name; the hunt's end is a hold.
    #[test]
    fn a_hunt_splits_its_hunters_into_a_group_of_their_own_and_ends_in_a_hold() {
        let (_, ours, enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.groups.push(Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0));
        // The fixture named A by hand; the next name the pianist gives is B (two groups named A matched one event).
        pianist.next_group = 1;
        let hunters: Vec<UnitId> = ours.iter().take(3).map(|u| u.id).collect();
        let mut commands = Vec::new();
        let did = Brain::start_hunt(&mut pianist, 0, &hunters, &parties[0], &ours, &enemies, 30, &mut commands).expect("a hunt");
        assert!(did.starts_with("3 of group_A hunt party_12"), "{did}");
        assert_eq!(pianist.groups.len(), 2);
        assert_eq!(pianist.groups[0].members.len(), 11);
        let hunt_group = &pianist.groups[1];
        assert_eq!(hunt_group.members, hunters);
        assert_eq!(hunt_group.parent.as_deref(), Some("A"));
        let GroupTask::Hunt(hunt) = &hunt_group.task else { panic!("the new group's task is the hunt: {:?}", hunt_group.task) };
        assert!(parties[0].ids.contains(&hunt.quarry));
        let quarry = hunt.quarry;
        // The parent, sent whole: no new group, its own task the hunt.
        let rest: Vec<UnitId> = pianist.groups[0].members.clone();
        let did = Brain::start_hunt(&mut pianist, 0, &rest, &parties[0], &ours, &enemies, 60, &mut commands).expect("a hunt");
        assert!(did.starts_with("hunt party_12") && did.contains("whole group"), "{did}");
        assert_eq!(pianist.groups.len(), 2);
        assert!(matches!(pianist.groups[0].task, GroupTask::Hunt(_)));
        // The engine says the quarry is lost: the hunting group holds where it stands, and the failure is returned.
        // The event names the lane's group (D15), so the parent, hunting the same quarry, is not ended by it.
        let event = micro::HuntEvent { group: format!("group_{}", pianist.groups[1].name), quarry, why: "lost" };
        assert!(Brain::hunt_event(&mut pianist.groups[0], &event, &ours, 200, false, &mut commands).is_none());
        let (text, failed) = Brain::hunt_event(&mut pianist.groups[1], &event, &ours, 200, false, &mut commands).expect("the end");
        assert!(text.contains("holds where it is"), "{text}");
        assert_eq!(failed.map(|(p, _)| p).as_deref(), Some("party_12"));
        assert!(matches!(pianist.groups[1].task, GroupTask::Hold { .. }));
        let _ = own;
    }

    /// At least half the group sent: the hunters keep the group's name and its task becomes the hunt; the rest are
    /// the new group with the old task, marked split from it, and the soldiers on their way to join go on to the rest.
    #[test]
    fn the_hunters_keep_the_name_when_at_least_half_the_group_goes() {
        let (_, ours, enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::Move { to: own(0, 1, super::super::fixtures::at(1.0, 1.0)).pos, place: "spot_3".into(), fight: true }, 0);
        group.joining.insert(UnitId(999), super::super::fixtures::at(0.0, 0.0));
        pianist.groups.push(group);
        let hunters: Vec<UnitId> = ours.iter().take(7).map(|u| u.id).collect();
        let mut commands = Vec::new();
        let did = Brain::start_hunt(&mut pianist, 0, &hunters, &parties[0], &ours, &enemies, 30, &mut commands).expect("a hunt");
        assert!(did.starts_with("7 of group_A hunt party_12") && did.contains("under its name") && did.contains("the 7 left behind are group_"), "{did}");
        assert_eq!(pianist.groups.len(), 2);
        let (a, rest) = (&pianist.groups[0], &pianist.groups[1]);
        assert_eq!((a.name.as_str(), a.members.clone()), ("A", hunters));
        assert!(matches!(a.task, GroupTask::Hunt(_)) && a.joining.is_empty());
        assert_eq!(rest.members.len(), 7);
        assert!(matches!(&rest.task, GroupTask::Move { place, .. } if place == "spot_3"), "{:?}", rest.task);
        assert_eq!(rest.parent.as_deref(), Some("A"));
        assert!(rest.joining.contains_key(&UnitId(999)));
    }

    /// A follower is sent after its ward when the ward has moved on, and nowhere while its last order still fits; a
    /// member rejoining gets the same order.
    #[test]
    fn a_follower_goes_after_its_ward() {
        let at = |x: f32| Vec3 { x, y: 0.0, z: 0.0 };
        assert_eq!(follow_step(at(100.0), at(0.0)), None);
        assert_eq!(follow_step(at(200.0), at(0.0)), Some(at(200.0)));
        let soldier = own(1, 1, at(0.0));
        let group = Group::new("A".into(), Domain::Ground, vec![soldier.id], GroupTask::Follow { ward: Ward::Builder(UnitId(9)), name: "constructor_9".into(), at: at(300.0), since: 0 }, 0);
        assert!(matches!(group.rejoin_orders(&[&soldier])[..], [Command::Fight { to, .. }] if to.x == 300.0));
    }

    /// An attack on a party follows it, and a soldier on its way to join goes after the body and takes the group's
    /// order when it gets there (player-35 7:23-8:28: the fight order stayed at where the Tick stood at the pick,
    /// and seven Blitzes that joined walked to where the group had been; all nine stood with no order for 35 s
    /// under a task that read "attacking").
    #[test]
    fn an_attack_follows_the_party_and_a_joiner_takes_the_groups_order() {
        use super::super::fixtures::at;
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, mut ours, mut enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        // An air group: undeclared, the hands' path (a ground attack is the lane's since chunk 3: the test below).
        let mut group = Group::new("A".into(), Domain::Air, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        let mut commands = Vec::new();
        Brain::engage_group(&mut group, &parties[0], &ours[..12], None, 30, &mut commands);
        // The last two are still on their way, far behind, sent to where a member stood then.
        let joiners = [ours[12].id, ours[13].id];
        for (id, unit) in joiners.iter().zip(&mut ours[12..]) {
            unit.pos = at(1000.0, 4000.0);
            group.joining.insert(*id, at(2000.0, 3000.0));
        }
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, plant: none, vehicle_constructor: none, advanced_lab: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None } };
        // The party has not moved: no order. The joiners are sent after the body, which is not where they were sent.
        let mut commands = Vec::new();
        brain.keep_groups(&tick(60, &ours, &enemies), &kit, &mut commands);
        assert!(!commands.iter().any(|c| matches!(c, Command::Fight { .. })), "{commands:?}");
        let moved: Vec<UnitId> = commands.iter().filter_map(|c| if let Command::Move { unit, to, .. } = c { (to.x > 4000.0).then_some(*unit) } else { None }).collect();
        assert_eq!(moved, joiners, "{commands:?}");
        // The party runs 600 east: the twelve of the body are sent to fight where it is now, the joiners are not.
        for e in enemies.iter_mut().take(3) {
            e.pos.x += 600.0;
        }
        let mut commands = Vec::new();
        brain.keep_groups(&tick(90, &ours, &enemies), &kit, &mut commands);
        let fights: Vec<(UnitId, f32)> = commands.iter().filter_map(|c| if let Command::Fight { unit, to, .. } = c { Some((*unit, to.x)) } else { None }).collect();
        assert_eq!(fights.len(), 12, "{commands:?}");
        assert!(fights.iter().all(|(unit, x)| !joiners.contains(unit) && (*x - 5102.0).abs() < 2.0), "{fights:?}");
        // It stands: no order again.
        let mut commands = Vec::new();
        brain.keep_groups(&tick(120, &ours, &enemies), &kit, &mut commands);
        assert!(commands.is_empty(), "{commands:?}");
        // A joiner reaches the body: it is of it, with the group's order.
        ours[12].pos = at(4200.0, 1950.0);
        let mut commands = Vec::new();
        brain.keep_groups(&tick(150, &ours, &enemies), &kit, &mut commands);
        assert!(matches!(commands[..], [Command::Fight { unit, to, .. }] if unit == joiners[0] && (to.x - 5102.0).abs() < 2.0), "{commands:?}");
        assert!(!brain.pianist.as_ref().unwrap().groups[0].joining.contains_key(&joiners[0]));
    }

    /// A ground group's attack is the lane's (chunk 3): the hands set the task and order no member, the party's
    /// sightings move the task's `at`, and a party out of sight for six seconds ends it in a hold.
    #[test]
    fn a_ground_attack_is_declared_and_the_hands_order_nobody() {
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, mut enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::new("A".into(), Domain::Ground, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        let mut commands = Vec::new();
        Brain::engage_group(&mut group, &parties[0], &ours[..12], None, 30, &mut commands);
        assert!(commands.is_empty(), "no order of the hands' for a declared attack: {commands:?}");
        assert!(group.declared() && matches!(group.task, GroupTask::Engage { .. }));
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, plant: none, vehicle_constructor: none, advanced_lab: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = |frame: i32, ours: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit]| Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.to_vec(), allies: Vec::new(), enemies: enemies.to_vec(), wrecks: None, sensors: None } };
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

    /// A roving group is the lane's: the hands keep up no order for it, its joiners are its own, and its course word
    /// is roving (the rework of docs/design/2026-10-07-rove-approval.md: roving is a task, never a flag).
    #[test]
    fn a_roving_group_gets_no_order_from_the_hands() {
        use bot_protocol::{Resource, Snapshot};
        let (mut brain, ours, enemies, _parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let mut group = Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::idle_hold(Vec3::default()), 0);
        group.joining.insert(ours[0].id, ours[0].pos);
        group.set_task(GroupTask::Rove { within: None }, 0);
        assert_eq!(group.course_word(), "roving");
        assert!(group.rejoin_orders(&[&ours[0]]).is_empty());
        pianist.groups.push(group);
        brain.pianist = Some(pianist);
        let none = UnitDefId(99);
        let kit = Kit { commander: none, extractor: none, converter: none, lab: none, turret: none, constructor: none, plant: none, vehicle_constructor: none, advanced_lab: none, advanced_constructor: none, advanced_extractor: none, raider: none, line: none, resurrector: none, roster: &crate::brain::roster::ROSTERS[0] };
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = Tick { frame: 60, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours.clone(), allies: Vec::new(), enemies: enemies.clone(), wrecks: None, sensors: None } };
        let mut commands = Vec::new();
        brain.keep_groups(&tick, &kit, &mut commands);
        let group = &brain.pianist.as_ref().unwrap().groups[0];
        assert!(group.roving() && group.joining.is_empty());
        assert!(commands.is_empty(), "{commands:?}");
        assert!(brain.journal.notes.iter().all(|n| n.source != "lane"));
    }

    /// A declared group (a ground walk) walks no joiner: a newcomer near or far is a member with no order of the
    /// hands', the lane's slot its walk (the migration design, chunk 2).
    #[test]
    fn a_declared_groups_newcomer_is_a_member_with_no_order_of_the_hands() {
        use super::super::fixtures::{at, e3, own};
        let (_brain, ours, _enemies, _parties) = e3();
        let mut group = Group::new("A".into(), Domain::Ground, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::Move { to: at(6000.0, 1700.0), place: "spot_18".into(), fight: false }, 0);
        assert!(group.declared());
        let mut all = ours.clone();
        for (id, pos) in [(700, at(ours[0].pos.x + 100.0, ours[0].pos.z)), (701, at(1000.0, 4000.0))] {
            let newcomer = own(id, 1, pos);
            all.push(newcomer.clone());
            assert!(group.admit(&newcomer, &all, |p| p, None).is_empty(), "no order of the hands'");
            assert!(group.members.contains(&newcomer.id) && !group.joining.contains_key(&newcomer.id));
        }
        assert!(group.hold_orders(&group.units(&all)).is_empty(), "the lane's Hold keeps the ranks");
    }

    /// An undeclared group (an air group here, until its chunk): a newcomer admitted beside the body takes the task's
    /// order at once; one admitted far from it is sent to the nearest member of the body's core, not to a member
    /// still joining (human-27 1:47-2:34: a Rascal adopted at the yard with no order stood there 47 s while its
    /// group walked to spot_18, and the first Incisor was sent to it).
    #[test]
    fn a_newcomer_beside_the_body_takes_its_order_and_one_far_from_it_walks_to_the_core() {
        use super::super::fixtures::{at, e3, own};
        let (_brain, ours, _enemies, _parties) = e3();
        let mut group = Group::new("A".into(), Domain::Air, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::Move { to: at(6000.0, 1700.0), place: "spot_18".into(), fight: false }, 0);
        assert!(!group.declared());
        let mut all = ours.clone();
        let beside = own(700, 1, at(ours[0].pos.x + 100.0, ours[0].pos.z));
        all.push(beside.clone());
        let orders = group.admit(&beside, &all, |p| p, None);
        assert!(matches!(orders[..], [Command::Move { unit, to, .. }] if unit == beside.id && to.x == 6000.0), "{orders:?}");
        assert!(!group.joining.contains_key(&beside.id) && group.members.contains(&beside.id));
        // A member still joining stands far off; the newcomer far from the body goes to the core, not to it.
        let stray = own(701, 1, at(1000.0, 4000.0));
        all.push(stray.clone());
        group.members.push(stray.id);
        group.joining.insert(stray.id, stray.pos);
        let far = own(702, 1, at(1100.0, 4000.0));
        all.push(far.clone());
        let orders = group.admit(&far, &all, |p| p, None);
        assert!(matches!(orders[..], [Command::Move { unit, to, .. }] if unit == far.id && to.dist2d(stray.pos) > 1000.0), "{orders:?}");
        assert!(group.joining.contains_key(&far.id));
    }

    /// `join_in`: a newcomer far from the body waits beside its plant, out of the exit lane on the side toward the
    /// body, and is not sent after the body; `release_waiting` sends the plant's waiting newcomers together (an
    /// undeclared group, an air one here) or, for a declared group, makes them members for the lane to place. One
    /// beside an undeclared body is of it at once, `join_in` or not.
    #[test]
    fn newcomers_under_join_in_wait_beside_their_plant_and_walk_together() {
        use super::super::fixtures::{at, e3, own};
        use bot_protocol::Lane;
        let lane = Lane { from: at(1000.0, 1000.0), to: at(1000.0, 1400.0), half_width: 100.0 };
        assert_eq!(beside_lane(&lane, at(0.0, 2000.0)), at(820.0, 1000.0));
        assert_eq!(beside_lane(&lane, at(3000.0, 2000.0)), at(1180.0, 1000.0));
        let (_brain, ours, _enemies, _parties) = e3();
        // A declared group: the wait as any newcomer's, the release a membership (no walk of the hands').
        let mut declared = Group::new("D".into(), Domain::Ground, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::Move { to: at(6000.0, 1700.0), place: "spot_18".into(), fight: false }, 0);
        let mut all = ours.clone();
        let plant = UnitId(5);
        let beside = at(900.0, 4000.0);
        let newcomer = own(710, 1, at(1100.0, 4000.0));
        all.push(newcomer.clone());
        let orders = declared.admit(&newcomer, &all, |p| p, Some((plant, 3, beside)));
        assert!(matches!(orders[..], [Command::Move { unit, to, .. }] if unit == newcomer.id && to == beside), "{orders:?}");
        assert_eq!(declared.waiting, vec![(plant, UnitId(710))]);
        assert!(declared.joining.is_empty());
        assert!(declared.release_waiting(plant, &all, |_, to| to).is_empty() && declared.waiting.is_empty());
        let mut group = Group::new("A".into(), Domain::Air, ours.iter().take(12).map(|u| u.id).collect(), GroupTask::Move { to: at(6000.0, 1700.0), place: "spot_18".into(), fight: false }, 0);
        let mut all = ours.clone();
        let plant = UnitId(5);
        let beside = at(900.0, 4000.0);
        for id in [700, 701] {
            let newcomer = own(id, 1, at(1100.0, 4000.0));
            all.push(newcomer.clone());
            let orders = group.admit(&newcomer, &all, |p| p, Some((plant, 3, beside)));
            assert!(matches!(orders[..], [Command::Move { unit, to, .. }] if unit == newcomer.id && to == beside), "{orders:?}");
        }
        assert_eq!(group.waiting, vec![(plant, UnitId(700)), (plant, UnitId(701))]);
        assert!(group.joining.contains_key(&UnitId(700)) && group.joining.contains_key(&UnitId(701)));
        let orders = group.release_waiting(plant, &all, |_, to| to);
        assert_eq!(orders.len(), 2);
        assert!(orders.iter().all(|o| matches!(o, Command::Move { to, .. } if to.dist2d(at(4150.0, 1900.0)) < 200.0)), "to the nearest of the body: {orders:?}");
        assert!(group.waiting.is_empty() && group.joining.len() == 2);
        let near = own(702, 1, at(ours[0].pos.x + 50.0, ours[0].pos.z));
        all.push(near.clone());
        group.admit(&near, &all, |p| p, Some((plant, 3, beside)));
        assert!(!group.joining.contains_key(&near.id) && group.waiting.is_empty());
    }

    /// `produce` ... `by_type`: a factory's newcomers form one group per unit type, each said when it forms, and a
    /// later soldier of a type joins that type's group.
    #[test]
    fn a_factorys_newcomers_form_one_group_per_unit_type_under_by_type() {
        use super::super::fixtures::{at, brain_of, own};
        use crate::strategist::shared::{Allowance, Shared};
        use bot_protocol::{Resource, Snapshot};
        let mut brain = brain_of(&["armflash", "armwar", "armllt"]);
        let shared = std::sync::Arc::new(Shared::default());
        // The plant is unknown to the brain's units, so its name is constructor_50; the allowance is keyed by it.
        shared.allowed.lock().unwrap().insert("constructor_50".into(), Allowance { call: 1, units: Vec::new(), group: Some("by_type".into()), join_in: None });
        brain.strategist = Some(shared);
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        for id in [1, 2, 3] {
            pianist.produced_by.insert(UnitId(id), UnitId(50));
        }
        brain.pianist = Some(pianist);
        let own = vec![own(1, 1, at(1000.0, 1000.0)), own(2, 2, at(1050.0, 1000.0)), own(3, 1, at(1100.0, 1000.0))];
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = Tick { frame: 300, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: own, allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None } };
        let id = UnitDefId(3);
        let kit = Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, plant: id, vehicle_constructor: id, advanced_lab: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &crate::brain::roster::ROSTERS[0] };
        let mut commands = Vec::new();
        brain.keep_groups(&tick, &kit, &mut commands);
        let pianist = brain.pianist.as_ref().unwrap();
        let groups: Vec<(String, Vec<UnitId>)> = pianist.groups.iter().map(|g| (g.name.clone(), g.members.clone())).collect();
        assert_eq!(groups.len(), 2, "{groups:?}");
        assert!(pianist.groups.iter().any(|g| g.members == [UnitId(1), UnitId(3)]) && pianist.groups.iter().any(|g| g.members == [UnitId(2)]), "the Blitzes together, the Centurion apart: {groups:?}");
        assert_eq!(pianist.done.iter().filter(|d| d.contains("formed: constructor_50's first")).count(), 2, "{:?}", pianist.done);
        assert!(pianist.done.iter().any(|d| d.contains("armflash (one group per unit type)")), "{:?}", pianist.done);
    }

    #[test]
    fn a_newcomer_joins_the_named_group_else_its_factorys_own_and_new_starts_a_fresh_one() {
        assert_eq!(group_for_newcomer(Some("group_A"), 3, Some((3, "B"))), Some("A".to_string()));
        assert_eq!(group_for_newcomer(None, 3, Some((2, "B"))), Some("B".to_string()));
        // `new` founds one group per `produce` call: the first soldier of call 3 starts afresh, the next join it,
        // and a later call's `new` starts afresh again.
        assert_eq!(group_for_newcomer(Some("new"), 3, Some((2, "B"))), None);
        assert_eq!(group_for_newcomer(Some("new"), 3, Some((3, "C"))), Some("C".to_string()));
        assert_eq!(group_for_newcomer(Some("new"), 4, Some((3, "C"))), None);
        assert_eq!(group_for_newcomer(None, 1, None), None);
        // `by_type`: the type's group of the factory, whatever call founded it; none founds one.
        assert_eq!(group_for_newcomer(Some("by_type"), 4, Some((2, "D"))), Some("D".to_string()));
        assert_eq!(group_for_newcomer(Some("by_type"), 4, None), None);
    }
}
