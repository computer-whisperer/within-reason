//! Soldier groups: the pianist's unit of command. A new soldier joins the group the player named for its factory's
//! output, else its factory's own group; nothing merges by proximity (H-HANDS-GROUPS, since 2026-09-25: groups are
//! the player's, `docs/design/2026-09-25-one-decider.md` §3). A group carries one task at a time; between calls the
//! standing orders are kept up (a march arrives together, an engagement follows its party, an arrival becomes a hold).

use std::collections::{BTreeMap, HashSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

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
const REACHED_KEPT: usize = 12;
/// A party nobody has seen for this long is gone; a ground group holds where it stands, an air group searches its
/// target's last position once and holds only after `AIR_LOST_FRAMES` (H-HANDS-AIR-TARGET: evidence-1-bombers, where
/// the hold after six seconds out of sight cancelled every strike).
const LOST_FRAMES: i32 = 6 * FRAMES_PER_SECOND;
const AIR_LOST_FRAMES: i32 = 60 * FRAMES_PER_SECOND;
/// H-HANDS-STALL: a moving group whose centre has not come this much closer to its goal in this long is stalled; the
/// picture says so, and an advancing one wakes the player once (pianist-player-2: the ball stood five minutes short
/// of the enemy base with the player reading "under fire" and nothing about the hands).
const PROGRESS_STEP: f32 = 60.0;
const STALL_FRAMES: i32 = 45 * FRAMES_PER_SECOND;
/// H-HANDS-LOSS-WAKE: a group wakes the player once per turn when, since the player's last orders, it has lost this
/// many soldiers or this share of its metal (upgrade-2: a unit loss woke nothing, and the ball shed pairs against the
/// Hounds for 7 to 37 s until the timer; Jev's `needs_player` Noul was a constant 0.77 with noise, retired).
const LOSS_WAKE_COUNT: usize = 2;
const LOSS_WAKE_SHARE: f32 = 0.25;
/// How a `fall_back` walk names its destination in the group's task (H-HANDS-FALL-BACK); not a mark's name.
pub(super) const LAST_HOLD: &str = "where it last held,";
/// A hold this long is a station worth falling back to (wake-2: a one-second pause at the front became "where it
/// last held", and group_L fell back to where it stood, held, retreated, and fell back again, eleven times in 30 s).
pub(super) const STATION_FRAMES: i32 = 15 * FRAMES_PER_SECOND;

/// A hunt (`docs/design/2026-09-26-threat-response.md` §1; the user, 2026-09-29: a hunt is a group of its own, never
/// part of the mass it left): the group's members attack one unit of a party by id, raw (no formation, no flee, no
/// march), until the quarry dies, is out of sight and radar for `HUNT_LOST_FRAMES`, or the leash ends; a hunter under
/// a third of its health drops out (`dropped`) and holds. The group holds where the hunt ends; the merge back is a
/// state Jev picks or the player asks for.
#[derive(Clone, Debug)]
pub(crate) struct Hunt {
    pub quarry: UnitId,
    pub party: String,
    /// Members the engine let go (hurt, or slower than the quarry): they hold and are not committed again.
    pub dropped: Vec<UnitId>,
    pub from: Vec3,
    pub since: i32,
    pub last_seen: i32,
    /// Where the quarry was last seen: the attack goes there when it is on radar only.
    pub at: Vec3,
}

/// A hunt this far from where it began ends (threats-smoke-1: anchored on the group's station, a hunt begun 900
/// from the station ended on its first tick, 12 of 16 hunts one order long). The engine judges it (H-MICRO-HUNT).
pub(crate) const HUNT_LEASH: f32 = 900.0;
/// A roving group's log keeps this many lines.
const ROVE_LOG: usize = 10;

#[derive(Clone, Debug)]
pub(crate) enum GroupTask {
    /// `committed`: the hold an advance arrives in, still committed to everything there (H-HANDS-GROUPS).
    Hold { since: i32, committed: bool },
    Move { to: Vec3, place: String, fight: bool, since: i32 },
    /// `target`: one unit of the party every member attacks directly (`attack_unit`, and every air group's
    /// engagement): re-issued while it is seen, the group holds when it is lost or dead.
    /// `searched`: an air group has been sent to its lost target's last position.
    Engage { party: Vec<UnitId>, at: Vec3, since: i32, last_seen: i32, target: Option<UnitId>, searched: bool },
    /// A hunt (H-MICRO-HUNT): the whole group after one unit of a party, the micro engine's to run.
    Hunt(Hunt),
}

impl GroupTask {
    pub(crate) fn busy(&self) -> bool {
        !matches!(self, GroupTask::Hold { .. })
    }
}

#[derive(Debug)]
pub(crate) struct Group {
    pub name: String,
    /// Its members' movement domain (docs/design/2026-09-22-domains.md): a group never mixes air and ground.
    pub domain: Domain,
    pub members: Vec<UnitId>,
    pub task: GroupTask,
    pub last_order: i32,
    /// The nearest the centre has been to a moving task's goal, and when it last got nearer (H-HANDS-STALL).
    pub best_to_go: f32,
    pub progressed: i32,
    pub stall_warned: bool,
    /// The group this one was split from (a detachment, a split, a scout), and when: said in the picture on both
    /// sides, so the player sees an army in pieces as it happens (escalate-6: seven detachments died one by one).
    pub parent: Option<String>,
    pub born: i32,
    /// Soldiers lost in the last minute (frame, type); the turn frame the wake counts from, and whether the player has
    /// been woken for them this turn (H-HANDS-LOSS-WAKE). The picture says the last 30 s.
    pub losses: Vec<(i32, UnitDefId)>,
    pub losses_since: i32,
    pub loss_warned: bool,
    /// Where the group last stood holding: the `fall_back` option's destination (H-HANDS-FALL-BACK).
    pub last_hold: Option<Vec3>,
    /// Members still on their way to join (a plant's output walking to the body): not the front, not the tail,
    /// not counted as arrived; said in the picture as reinforcements on the way (H-HANDS-GROUP-BODY).
    pub joining: HashSet<UnitId>,
    /// Gathering: holding where it arrived until its tail is up (the `gather` state, H-HANDS-GROUP-STATES).
    pub gathering: bool,
    /// Shelling a party from a standoff: the long-reach members at the standoff, the rest between (`shell`).
    pub shelling: bool,
    /// The places named for this group in the packet that its body has come within `ARRIVED` of, in order, with
    /// the frame: the facts of a route the packet wrote in prose, said in the picture and in `recent` and never
    /// acted on by code (docs/design/2026-09-28-routes-in-prose.md §4.2; the station list they replace was a
    /// mechanic Jev only said yes or no to).
    pub reached: Vec<(String, i32)>,
    /// The first enemies in sight near the body since the last place reached: the frame and the words.
    pub met: Option<(i32, String)>,
    /// The places its paragraph names, in the packet's order, as of the last look: the route `reached` and `met`
    /// are the facts of. A different set of places is a new route and the facts begin again (player-10 18:20: a
    /// group's `route_seen` listed every arrival since 3:09 and none of the route it was on).
    pub route: Vec<String>,
    /// Made by the hands' `scout` state: it roves unless the player's `lane` says otherwise for it (H-MICRO-ROVE).
    pub scout: bool,
    /// Roving as of the last look (`keep_groups`): a change hands the group to the lane or takes it back.
    pub roving: bool,
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
    pub(crate) fn new(name: String, domain: Domain, members: Vec<UnitId>, task: GroupTask, frame: i32) -> Group {
        Group { name, domain, members, task, last_order: frame, best_to_go: f32::INFINITY, progressed: frame, stall_warned: false, parent: None, born: frame, losses: Vec::new(), losses_since: frame, loss_warned: false, last_hold: None, joining: HashSet::new(), gathering: false, shelling: false, reached: Vec::new(), met: None, route: Vec::new(), scout: false, roving: false, rove_log: Vec::new() }
    }

    /// The group's body toward `toward` (the goal of a walk, the nearest enemy, or nothing: then the front is the
    /// centre and the tail the member farthest from it). `None` when no member stands.
    pub(crate) fn body<'a>(&self, own: &'a [OwnUnit], toward: Option<Vec3>) -> Option<Body<'a>> {
        let units = self.units(own);
        let (joining, mut core): (Vec<&OwnUnit>, Vec<&OwnUnit>) = units.iter().copied().partition(|u| self.joining.contains(&u.id));
        if core.is_empty() {
            // Everybody is still on the way: the body is wherever they are.
            core = joining.clone();
        }
        let centre = centre_of(&core)?;
        let toward = toward.unwrap_or(centre);
        core.sort_by(|a, b| a.pos.dist2d(toward).total_cmp(&b.pos.dist2d(toward)));
        let front = core.first().map_or(centre, |u| u.pos);
        let tail = core.last().map_or(centre, |u| u.pos);
        let at = core.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len() - 1)).map_or(centre, |u| u.pos);
        let arrived = core.iter().filter(|u| u.pos.dist2d(toward) < ARRIVED).count();
        let joining: Vec<&OwnUnit> = joining.into_iter().filter(|u| !core.iter().any(|c| c.id == u.id)).collect();
        Some(Body { length: front.dist2d(tail), core, joining, front, tail, at, arrived })
    }

    /// Orders for members rejoining the task: what the task would give them now. A holding group's members walk back
    /// to where it holds (onepass-norules-hard-3, 4:47: a hunter released 850 from its holding group was told to
    /// stand where it was, and stood there for the rest of the game).
    pub(crate) fn rejoin_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
        match &self.task {
            GroupTask::Hold { .. } => match self.last_hold {
                Some(to) => units.iter().map(|u| Command::Move { unit: u.id, to, queue: false }).collect(),
                None => self.hold_orders(units),
            },
            GroupTask::Move { to, fight, .. } => units.iter().map(|u| if *fight { Command::Fight { unit: u.id, to: *to, queue: false } } else { Command::Move { unit: u.id, to: *to, queue: false } }).collect(),
            GroupTask::Engage { at, target, .. } => units.iter().map(|u| match target { Some(t) => Command::Attack { unit: u.id, target: *t, queue: false }, None => Command::Fight { unit: u.id, to: *at, queue: false } }).collect(),
            GroupTask::Hunt(h) => units.iter().map(|u| Command::Attack { unit: u.id, target: h.quarry, queue: false }).collect(),
        }
    }

    /// The soldiers lost since `since`, and their metal.
    pub(crate) fn lost_since(&self, since: i32, world: &crate::world::World) -> (usize, f32) {
        let lost: Vec<&(i32, UnitDefId)> = self.losses.iter().filter(|(f, _)| *f >= since).collect();
        (lost.len(), lost.iter().filter_map(|(_, def)| world.def(*def)).map(|d| d.metal_cost).sum())
    }

    /// A group split from another.
    pub(crate) fn split_from(mut self, parent: &str) -> Group {
        self.parent = Some(parent.to_string());
        self
    }

    /// The orders that make this group stand still: a stop for ground units; for aircraft, whose empty queue is a
    /// licence to hunt, hold position and a move to the centre (the domains design, decision 4).
    pub(crate) fn hold_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
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

    /// Game seconds since a moving group last got nearer its goal; `None` when it is not moving.
    pub(crate) fn stalled_seconds(&self, frame: i32) -> Option<i32> {
        matches!(self.task, GroupTask::Move { .. }).then(|| (frame - self.progressed) / FRAMES_PER_SECOND)
    }

    pub(crate) fn units<'a>(&self, own: &'a [OwnUnit]) -> Vec<&'a OwnUnit> {
        own.iter().filter(|u| self.members.contains(&u.id)).collect()
    }
}

impl Group {
    /// A new task starts the progress clock afresh.
    pub(crate) fn set_task(&mut self, task: GroupTask, frame: i32) {
        self.task = task;
        self.best_to_go = f32::INFINITY;
        self.progressed = frame;
        self.stall_warned = false;
        self.gathering = false;
        self.shelling = false;
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
            group.losses.retain(|(f, _)| frame - f <= 60 * FRAMES_PER_SECOND);
            for (f, id, def) in self.unit_losses.iter().rev().take_while(|(f, ..)| *f == frame) {
                if group.members.contains(id) {
                    group.losses.push((*f, *def));
                }
            }
            group.members.retain(|id| soldiers.iter().any(|u| u.id == *id));
            let standing: f32 = group.units(own).iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            let (count, lost) = group.lost_since(turn_frame, &self.world);
            if !group.loss_warned && !group.members.is_empty() && (count >= LOSS_WAKE_COUNT || lost >= LOSS_WAKE_SHARE * (standing + lost)) {
                group.loss_warned = true;
                let doing = match &group.task {
                    _ if group.roving => "roving".to_string(),
                    GroupTask::Hold { .. } => "holding".to_string(),
                    GroupTask::Move { place, fight, .. } => format!("{} to {place}", if *fight { "advancing" } else { "walking" }),
                    GroupTask::Engage { .. } => "attacking a party".to_string(),
                    GroupTask::Hunt(h) => format!("hunting {}", h.party),
                };
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
        // start, a resurrection) forms a group of its own. Nothing merges by proximity: a merge is the player's order
        // (`join_group_X`, the standing rule `join`). Standing-2: 22 groups lived a median 1.3 minutes under the old
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
                let group = Group::new(name.clone(), domain, members, GroupTask::Hold { since: frame, committed: false }, frame);
                commands.extend(group.hold_orders(&units));
                pianist.groups.push(group);
                pianist.done.push(format!("{} group_{name} formed from the {} soldiers given to this seat", super::picture::clock(frame), units.len()));
            }
            loose.retain(|u| !given.iter().any(|g| g.id == u.id));
        }
        for unit in loose {
            let domain = self.world.domain_of(unit.def);
            let factory = pianist.produced_by.get(&unit.id).copied();
            let named = factory.and_then(|f| self.allowed_units(&self.actor_name(f))).and_then(|a| a.group);
            let wanted = group_for_newcomer(named.as_deref(), factory.and_then(|f| pianist.rally.get(&f)).map(String::as_str));
            let target = wanted.and_then(|name| pianist.groups.iter().position(|g| g.name == name && g.domain == domain));
            match target {
                Some(i) => {
                    // To the nearest member standing, on ground the newcomer reaches, never the arithmetic centre
                    // (Cape Violet: the centre of a group split between a plateau and the shore below lay in deep
                    // water, and every newcomer was sent into it).
                    let nearest = pianist.groups[i].units(own).iter().filter(|m| !pianist.groups[i].joining.contains(&m.id)).map(|m| m.pos).min_by(|a, b| a.dist2d(unit.pos).total_cmp(&b.dist2d(unit.pos)));
                    // Of the body when it stands within the adopt radius of the body's place, else a reinforcement
                    // on its way: by the nearest member a plant's stream at the yard adopted itself one unit at a
                    // time and the body's tail never left home (player-10 18:20: "its tail is 4649 behind at yard2").
                    let body_at = pianist.groups[i].body(own, None).map(|b| b.at);
                    pianist.groups[i].members.push(unit.id);
                    if let Some(to) = nearest
                        && body_at.is_none_or(|at| at.dist2d(unit.pos) > ADOPT_RADIUS)
                    {
                        let to = self.snap_for(self.walker_of(unit.def), to);
                        pianist.groups[i].joining.insert(unit.id);
                        commands.push(Command::Move { unit: unit.id, to, queue: false });
                    }
                }
                None => {
                    let name = pianist.new_group_name();
                    if let Some(f) = factory {
                        pianist.rally.insert(f, name.clone());
                        // `new` asked for one fresh group, not one per soldier: from now the factory's own.
                        if named.as_deref() == Some("new")
                            && let Some(shared) = &self.strategist
                            && let Some(a) = shared.allowed.lock().unwrap().get_mut(&self.actor_name(f))
                        {
                            a.group = None;
                        }
                    }
                    let group = Group::new(name, domain, vec![unit.id], GroupTask::Hold { since: frame, committed: false }, frame);
                    commands.extend(group.hold_orders(&[unit]));
                    pianist.groups.push(group);
                }
            }
        }
        // Standing orders, under each group's footwork rules (H-HANDS-LANE).
        let mut stalled: Vec<String> = Vec::new();
        let mut hunt_ends: Vec<String> = Vec::new();
        let mut route_news: Vec<String> = Vec::new();
        let roving: Vec<bool> = pianist.groups.iter().map(|g| self.roves(g)).collect();
        let mut rove_changes: Vec<String> = Vec::new();
        for (index, group) in pianist.groups.iter_mut().enumerate() {
            // A roving group is the lane's (H-MICRO-ROVE): no task, no hunt, no orders kept up. Switched either way,
            // it starts from a hold; switched off, its soldiers hold where each stands.
            if roving[index] != group.roving {
                group.roving = roving[index];
                (group.gathering, group.shelling) = (false, false);
                group.joining.clear();
                group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                if group.roving {
                    rove_changes.push(format!("group_{} roves: its soldiers look at what we know least, kill what they find unguarded and keep out of every reach, in code, until the lane is set otherwise", group.name));
                } else {
                    commands.extend(group.hold_orders(&group.units(own)));
                    rove_changes.push(format!("group_{} stops roving and holds where each of its soldiers stands", group.name));
                }
            }
            if group.roving {
                group.joining.clear();
                continue;
            }
            let units = group.units(own);
            group.joining.retain(|id| group.members.contains(id));
            // A newcomer that has reached the body is of it (or was sent nowhere: nothing stands to reach).
            let goal = match &group.task { GroupTask::Move { to, .. } => Some(*to), GroupTask::Engage { at, .. } => Some(*at), GroupTask::Hunt(h) => Some(h.at), GroupTask::Hold { .. } => None };
            if let Some(body) = group.body(own, goal) {
                let core_at = body.at;
                let joined: Vec<UnitId> = body.joining.iter().filter(|u| u.pos.dist2d(core_at) <= ADOPT_RADIUS || self.stuck.contains_key(&u.id)).map(|u| u.id).collect();
                for id in joined {
                    group.joining.remove(&id);
                    // Onto the task with the rest.
                    if let Some(u) = units.iter().find(|u| u.id == id) {
                        commands.extend(group.rejoin_orders(&[u]));
                    }
                }
            }
            // The body's place, not the centre of every member: the stream still joining from the plants would drag
            // the centre home (player-10 18:20: a front at spot_40 with 34 waiting at the yard read "0 of 59 arrived").
            let Some(centre) = group.body(own, goal).map(|b| b.at) else { continue };
            // The route's facts (routes-in-prose §4.2-4.3): a place the packet names for this group that the body
            // comes within `ARRIVED` of is reached, said in `recent` and asked about at once; the first enemies in
            // sight since the last stop are kept until the next. Code records; Jev picks the next leg. The facts
            // are the current route's: a paragraph naming a different set of places begins them again.
            let text = super::diet::paragraph(&pianist.packet_seen, &format!("group_{}", group.name)).unwrap_or(&pianist.packet_seen);
            let route: Vec<String> = super::diet::named_in_order(text, pianist.places.iter().map(|p| p.name.as_str()).filter(|n| *n != "home")).into_iter().map(str::to_string).collect();
            if route != group.route {
                group.route = route;
                group.reached.clear();
                group.met = None;
            }
            let here = pianist.places.iter().find(|p| p.name != "home" && super::diet::names(text, &p.name) && p.at.dist2d(centre) < ARRIVED).map(|p| p.name.clone());
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
            // A gather ends when the tail is up: the group is a body again and holds where it gathered.
            if group.gathering
                && let Some(body) = group.body(own, None)
                && !body.strung_out()
            {
                group.gathering = false;
            }
            match &mut group.task {
                GroupTask::Hold { since, .. } => {
                    if frame - *since >= STATION_FRAMES {
                        group.last_hold = Some(centre);
                    }
                }
                GroupTask::Move { to, fight, place, .. } => {
                    // Arrival and "to go" by the members, not the centre: a group split by a cliff never got its
                    // centre within 300 of its goal, so a fall-back walk stayed the base world for two minutes and
                    // the player's new stations were shown as the default and never played (Cape Violet 9:40-11:17).
                    let joining = group.joining.clone();
                    let moving: Vec<&OwnUnit> = units.iter().copied().filter(|u| !joining.contains(&u.id) && !self.stuck.contains_key(&u.id)).collect();
                    let core = if moving.is_empty() { units.clone() } else { moving };
                    let mut distances: Vec<f32> = core.iter().map(|u| u.pos.dist2d(*to)).collect();
                    distances.sort_by(f32::total_cmp);
                    let to_go = distances.get(((core.len() as f32 * BODY_SHARE) as usize).min(core.len().saturating_sub(1))).copied().unwrap_or(0.0);
                    if to_go < group.best_to_go - PROGRESS_STEP {
                        (group.best_to_go, group.progressed) = (to_go, frame);
                    }
                    if to_go < ARRIVED {
                        let gathering = group.gathering;
                        let shelling = group.shelling;
                        group.task = GroupTask::Hold { since: frame, committed: *fight || shelling };
                        group.gathering = gathering;
                        group.shelling = shelling;
                    } else if *fight {
                        if frame - group.progressed >= STALL_FRAMES && !group.stall_warned {
                            group.stall_warned = true;
                            stalled.push(format!("group_{} was told to advance to {place} and has not got nearer for {} s, {to_go:.0} short of it", group.name, (frame - group.progressed) / FRAMES_PER_SECOND));
                        }
                    }
                }
                GroupTask::Engage { party, at, last_seen, target, searched, .. } => {
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
                            if found_again {
                                group.last_order = frame;
                                commands.extend(units.iter().map(|u| Command::Attack { unit: u.id, target: t, queue: false }));
                            }
                        } else if air && frame - *last_seen > AIR_LOST_FRAMES {
                            commands.extend(group.hold_orders(&units));
                            group.task = GroupTask::Hold { since: frame, committed: false };
                        } else if air && frame - *last_seen > LOST_FRAMES && !*searched {
                            *searched = true;
                            group.last_order = frame;
                            let to = *at;
                            commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to, queue: false }));
                        } else if !air && frame - *last_seen > LOST_FRAMES {
                            commands.extend(group.hold_orders(&units));
                            group.task = GroupTask::Hold { since: frame, committed: false };
                        }
                    } else if let Some(now) = centre_of_enemies(&seen) {
                        *last_seen = frame;
                        *at = now;
                    } else if frame - *last_seen > LOST_FRAMES {
                        commands.extend(group.hold_orders(&units));
                        group.task = GroupTask::Hold { since: frame, committed: false };
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
            }
        }
        for text in rove_changes {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
        }
        for text in hunt_ends {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            pianist.hunt_events.push(text);
        }
        for text in route_news {
            pianist.note(frame, text.clone());
            pianist.events.insert(text);
        }
        for text in stalled {
            pianist.note(frame, text.clone());
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            if let Some(shared) = &self.strategist {
                shared.trigger(text);
            }
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

    /// The micro engine's word on a hunt (`micro::HuntEvent`): a hunter dropped (hurt, or slower than the quarry)
    /// holds where it is; the hunt's end (the quarry dead or lost, the leash reached, no hunter left) leaves the
    /// group holding where it stands, for Jev or the player to send on or merge. Returns the end's words and, for a
    /// hunt that ended without a kill, the failure to remember (the party and why).
    pub(crate) fn hunt_event(group: &mut Group, event: &micro::HuntEvent, own: &[OwnUnit], frame: i32, commands: &mut Vec<Command>) -> Option<(String, Option<(String, String)>)> {
        let GroupTask::Hunt(hunt) = &mut group.task else { return None };
        if hunt.quarry != event.quarry {
            return None;
        }
        match event.hunter {
            Some(hunter) => {
                hunt.dropped.push(hunter);
                commands.push(Command::Stop { unit: hunter });
                None
            }
            None => {
                let (party, since) = (hunt.party.clone(), hunt.since);
                let units = group.units(own);
                commands.extend(group.hold_orders(&units));
                group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                let why = match event.why {
                    "dead" => "the quarry is dead".to_string(),
                    "lost" => format!("{party} out of sight for 6 s"),
                    "leash" => format!("the leash of {HUNT_LEASH:.0} from where it began reached"),
                    other => other.to_string(),
                };
                let failed = (event.why != "dead").then(|| (party.clone(), why.clone()));
                Some((format!("group_{}'s hunt of {party} ended after {} s: {why}; it holds where it is ({} soldiers)", group.name, (frame - since) / FRAMES_PER_SECOND, units.len()), failed))
            }
        }
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
        let group = &mut pianist.groups[index];
        commands.extend(group.release_orders(&chosen));
        if chosen.len() == all.len() {
            group.set_task(GroupTask::Hunt(hunt), frame);
            return Some(format!("hunt {} ({}) with the whole group", party.name, party.composition));
        }
        let ids: Vec<UnitId> = chosen.iter().map(|u| u.id).collect();
        group.members.retain(|id| !ids.contains(id));
        let (domain, parent_name) = (group.domain, group.name.clone());
        let name = pianist.new_group_name();
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
        Hunt { quarry: quarry.0, party: party.name.clone(), dropped: Vec::new(), from: centre, since: frame, last_seen: frame, at: quarry.1 }
    }

    /// The whole group engages the party: released, sent to fight at it, its task the engagement.
    pub(super) fn engage_group(group: &mut Group, party: &Party, own: &[OwnUnit], target: Option<UnitId>, frame: i32, commands: &mut Vec<Command>) -> String {
        let units = group.units(own);
        commands.extend(group.release_orders(&units));
        match target {
            Some(t) => commands.extend(units.iter().map(|u| Command::Attack { unit: u.id, target: t, queue: false })),
            None => commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to: party.at, queue: false })),
        }
        // The leash's anchor is the body's place, as the leash measures it (`tick_groups`: "the body's place, not
        group.set_task(GroupTask::Engage { party: party.ids.clone(), at: party.at, since: frame, last_seen: frame, target, searched: false }, frame);
        group.last_order = frame;
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

/// The group a newcomer wants (H-HANDS-GROUPS): the one the player named for its factory, none for `new` (a fresh
/// group), else the factory's own; a soldier of no factory forms its own.
fn group_for_newcomer(named: Option<&str>, factory_group: Option<&str>) -> Option<String> {
    match named {
        Some("new") => None,
        Some(name) => Some(name.trim_start_matches("group_").to_string()),
        None => factory_group.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{e3, own};
    use super::*;

    /// A hunt is a group of its own: three of fourteen sent after a party split off under a new name with the hunt
    /// as their task, the parent keeps eleven; the whole of a group hunting keeps its name; the hunt's end is a hold.
    #[test]
    fn a_hunt_splits_its_hunters_into_a_group_of_their_own_and_ends_in_a_hold() {
        let (_, ours, enemies, parties) = e3();
        let mut pianist = super::super::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.groups.push(Group::new("A".into(), Domain::Ground, ours.iter().map(|u| u.id).collect(), GroupTask::Hold { since: 0, committed: false }, 0));
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
        let event = micro::HuntEvent { quarry, hunter: None, why: "lost" };
        let (text, failed) = Brain::hunt_event(&mut pianist.groups[1], &event, &ours, 200, &mut commands).expect("the end");
        assert!(text.contains("holds where it is"), "{text}");
        assert_eq!(failed.map(|(p, _)| p).as_deref(), Some("party_12"));
        assert!(matches!(pianist.groups[1].task, GroupTask::Hold { .. }));
        let _ = own;
    }

    #[test]
    fn a_newcomer_joins_the_named_group_else_its_factorys_own_and_new_starts_a_fresh_one() {
        assert_eq!(group_for_newcomer(Some("group_A"), Some("B")), Some("A".to_string()));
        assert_eq!(group_for_newcomer(None, Some("B")), Some("B".to_string()));
        assert_eq!(group_for_newcomer(Some("new"), Some("B")), None);
        assert_eq!(group_for_newcomer(None, None), None);
    }
}
