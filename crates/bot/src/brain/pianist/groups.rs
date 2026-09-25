//! Soldier groups: the pianist's unit of command. A new soldier joins the group the player named for its factory's
//! output, else its factory's own group; nothing merges by proximity (H-HANDS-GROUPS, since 2026-09-25: groups are
//! the player's, `docs/design/2026-09-25-one-decider.md` §3). A group carries one task at a time; between calls the
//! standing orders are kept up (a march arrives together, an engagement follows its party, an arrival becomes a hold).

use std::collections::{BTreeMap, HashSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::Party;
use super::standing::NEVER_REACH;
use crate::world::Domain;

/// H-HANDS-GROUPS: a newcomer whose group stands farther than this walks to it (fresh Grunts at the lab formed groups of
/// one and two while the ball held 400 to 2,700 away, and died in ones and twos: human-8, eight Grunts lost by 3:14,
/// every one in a group of five or fewer, 380 to 1,231 from the commander).
const ADOPT_RADIUS: f32 = 400.0;
/// A moving group has arrived when its centre is this close to its destination.
const ARRIVED: f32 = 300.0;
/// An engaged group is sent on when its party has moved this far, and no more often than this.
const FOLLOW_DISTANCE: f32 = 150.0;
const FOLLOW_FRAMES: i32 = 2 * FRAMES_PER_SECOND;
/// A party that has drawn an engaging group this far from where the engagement began is running, not fighting:
/// the group holds and the player is told (2v1-hard_aggressive 9:21-9:46 and 11:04-11:32: groups followed
/// retreating parties 2,000 and 3,500 elmos across the map, re-sent every 2 s while the party stayed in sight
/// 300-600 ahead, and died to what waited there; K-hands-follow-had-no-leash).
const FOLLOW_LEASH: f32 = 900.0;
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
/// An advance that has not got nearer its goal for this long is given up: the group holds and the player is told
/// (pianist-player-7: the ball answered `continue` for 151 s short of an islet spot, then for 103 s short of a mark
/// on the shore, while the player rewrote the packet four times).
const GIVE_UP_FRAMES: i32 = 90 * FRAMES_PER_SECOND;
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

/// A hunt (`docs/design/2026-09-26-threat-response.md` §1-2): these members of the group attack one unit of a party by
/// id, raw (no formation, no flee, no march), re-issued from the latest sighting, until the quarry dies, is out of
/// sight and radar for `HUNT_LOST_FRAMES`, or the leash ends; a hunter under a third of its health drops out. The
/// rest of the group keeps its task; the hunters rejoin it when the hunt ends. The micro engine's own primitive
/// replaces the brain's re-issued attack when it lands.
#[derive(Clone, Debug)]
pub(crate) struct Hunt {
    pub quarry: UnitId,
    pub party: String,
    pub hunters: Vec<UnitId>,
    pub from: Vec3,
    pub since: i32,
    pub last_seen: i32,
    /// Where the quarry was last seen: the attack goes there when it is on radar only.
    pub at: Vec3,
}

/// A hunt this far from where it began ends (threats-smoke-1: anchored on the group's station, a hunt begun 900
/// from the station ended on its first tick, 12 of 16 hunts one order long). The engine judges it (H-MICRO-HUNT).
pub(crate) const HUNT_LEASH: f32 = 900.0;
/// A party a group was told to leave is not answered by its rule's default for this long.
pub(super) const DECLINE_FRAMES: i32 = 30 * FRAMES_PER_SECOND;

#[derive(Clone, Debug)]
pub(crate) enum GroupTask {
    /// `committed`: the hold an advance arrives in, still committed to everything there (H-HANDS-GROUPS).
    Hold { since: i32, committed: bool },
    Move { to: Vec3, place: String, fight: bool, since: i32 },
    /// `target`: one unit of the party every member attacks directly (`attack_unit`, and every air group's
    /// engagement): re-issued while it is seen, the group holds when it is lost or dead.
    /// `searched`: an air group has been sent to its lost target's last position. `from`: where the group stood
    /// when the engagement began, the follow leash's anchor.
    Engage { party: Vec<UnitId>, at: Vec3, since: i32, last_seen: i32, target: Option<UnitId>, searched: bool, from: Vec3 },
}

impl GroupTask {
    pub(crate) fn busy(&self) -> bool {
        !matches!(self, GroupTask::Hold { .. })
    }

    /// The place a walk is bound for, by name.
    pub(crate) fn place(&self) -> Option<&str> {
        match self {
            GroupTask::Move { place, .. } => Some(place.as_str()),
            _ => None,
        }
    }

    /// The frame the task was set.
    pub(crate) fn since(&self) -> i32 {
        match self {
            GroupTask::Hold { since, .. } | GroupTask::Move { since, .. } | GroupTask::Engage { since, .. } => *since,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Group {
    pub name: String,
    /// Its members' movement domain (docs/design/2026-09-22-domains.md): a group never mixes air and ground.
    pub domain: Domain,
    pub members: Vec<UnitId>,
    pub task: GroupTask,
    /// H-ARMY-MARCH's memory: who is waiting for the body.
    pub held: HashSet<UnitId>,
    pub last_order: i32,
    /// Whether an enemy party stood within reach at the last look: a new one is a reason to ask at once.
    pub enemies_near: bool,
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
    /// The hunt some or all of its members are on (`docs/design/2026-09-26-threat-response.md` §1-2).
    pub hunt: Option<Hunt>,
    /// Parties this group was told to leave (a pick, or a hunt that ended at its leash), and when: no default and
    /// no hunt against them for `DECLINE_FRAMES`, so a plan the pick ended is not restarted by the rule a second later.
    pub declined: Vec<(String, i32)>,
}

impl Group {
    pub(crate) fn new(name: String, domain: Domain, members: Vec<UnitId>, task: GroupTask, frame: i32) -> Group {
        Group { name, domain, members, task, held: HashSet::new(), last_order: frame, enemies_near: false, best_to_go: f32::INFINITY, progressed: frame, stall_warned: false, parent: None, born: frame, losses: Vec::new(), losses_since: frame, loss_warned: false, last_hold: None, hunt: None, declined: Vec::new() }
    }

    /// The members not on a hunt: the ones the group's task orders.
    pub(crate) fn free_units<'a>(&self, own: &'a [OwnUnit]) -> Vec<&'a OwnUnit> {
        let hunting: Vec<UnitId> = self.hunt.as_ref().map(|h| h.hunters.clone()).unwrap_or_default();
        self.units(own).into_iter().filter(|u| !hunting.contains(&u.id)).collect()
    }

    /// Orders for members rejoining the task: what the task would give them now.
    pub(crate) fn rejoin_orders(&self, units: &[&OwnUnit]) -> Vec<Command> {
        match &self.task {
            GroupTask::Hold { .. } => self.hold_orders(units),
            GroupTask::Move { to, fight, .. } => units.iter().map(|u| if *fight { Command::Fight { unit: u.id, to: *to, queue: false } } else { Command::Move { unit: u.id, to: *to, queue: false } }).collect(),
            GroupTask::Engage { at, target, .. } => units.iter().map(|u| match target { Some(t) => Command::Attack { unit: u.id, target: *t, queue: false }, None => Command::Fight { unit: u.id, to: *at, queue: false } }).collect(),
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
        self.held.clear();
        self.best_to_go = f32::INFINITY;
        self.progressed = frame;
        self.stall_warned = false;
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
    pub(super) fn keep_groups(&mut self, tick: &Tick, _kit: &Kit, commands: &mut Vec<Command>) {
        let Some(kit) = self.kit else { return };
        let own = &tick.snapshot.own_units;
        let soldiers: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_army(u, &kit)).collect();
        let enemies = &tick.snapshot.enemies;
        let frame = tick.frame;
        let home = self.home;
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
                    GroupTask::Hold { .. } => "holding".to_string(),
                    GroupTask::Move { place, fight, .. } => format!("{} to {place}", if *fight { "advancing" } else { "walking" }),
                    GroupTask::Engage { .. } => "attacking a party".to_string(),
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
        for unit in loose {
            let domain = self.world.domain_of(unit.def);
            let factory = pianist.produced_by.get(&unit.id).copied();
            let named = factory.and_then(|f| self.allowed_units(&self.actor_name(f))).and_then(|a| a.group);
            let wanted = group_for_newcomer(named.as_deref(), factory.and_then(|f| pianist.rally.get(&f)).map(String::as_str));
            let target = wanted.and_then(|name| pianist.groups.iter().position(|g| g.name == name && g.domain == domain));
            match target {
                Some(i) => {
                    let centre = centre_of(&pianist.groups[i].units(own));
                    pianist.groups[i].members.push(unit.id);
                    if let Some(c) = centre
                        && c.dist2d(unit.pos) > ADOPT_RADIUS
                    {
                        commands.push(Command::Move { unit: unit.id, to: c, queue: false });
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
        let footwork: Vec<crate::strategist::shared::Footwork> = pianist.groups.iter().map(|g| self.footwork_of(&g.name)).collect();
        let marks: Vec<String> = self.strategist.as_ref().map(|s| s.marks.lock().unwrap().keys().cloned().collect()).unwrap_or_default();
        let is_mark_name = |place: &str| place != "home" && !place.starts_with("spot_") && !place.starts_with("passage_") && !place.starts_with(LAST_HOLD);
        let mut marches: Vec<(usize, Vec3)> = Vec::new();
        let mut stalled: Vec<String> = Vec::new();
        // Per group, the places it never goes (the player's `never` rule), for the chase that would reach one.
        let never_at: BTreeMap<String, Vec<(String, Vec3)>> = pianist
            .groups
            .iter()
            .map(|g| {
                let places = pianist.standing.never_places(&format!("group_{}", g.name)).into_iter().filter_map(|n| pianist.places.iter().find(|p| p.name == n).map(|p| (n.clone(), p.at))).collect();
                (g.name.clone(), places)
            })
            .collect();
        let mut hunt_ends: Vec<String> = Vec::new();
        for (index, group) in pianist.groups.iter_mut().enumerate() {
            // The hunt first: its members are not the task's this tick.
            if let Some(end) = Brain::tick_hunt(group, own, enemies, frame, commands) {
                hunt_ends.push(end);
            }
            let units = group.free_units(own);
            let Some(centre) = centre_of(&units) else { continue };
            match &mut group.task {
                GroupTask::Hold { since, .. } => {
                    if frame - *since >= STATION_FRAMES {
                        group.last_hold = Some(centre);
                    }
                }
                GroupTask::Move { to, fight, place, .. } => {
                    let to_go = centre.dist2d(*to);
                    if to_go < group.best_to_go - PROGRESS_STEP {
                        (group.best_to_go, group.progressed) = (to_go, frame);
                    }
                    let forgotten = is_mark_name(place) && !marks.contains(place);
                    let given_up = frame - group.progressed >= GIVE_UP_FRAMES;
                    if to_go < ARRIVED {
                        group.task = GroupTask::Hold { since: frame, committed: *fight };
                        group.held.clear();
                    } else if forgotten || given_up {
                        stalled.push(if forgotten {
                            format!("group_{} was walking to {place}, which is no longer a marked place: it holds where it is", group.name)
                        } else {
                            format!("group_{} gave up its {} to {place}: it has not got nearer for {} s, {to_go:.0} short of it, and holds where it is", group.name, if *fight { "advance" } else { "walk" }, (frame - group.progressed) / FRAMES_PER_SECOND)
                        });
                        commands.extend(group.hold_orders(&units));
                        group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                    } else if *fight {
                        if frame - group.progressed >= STALL_FRAMES && !group.stall_warned {
                            group.stall_warned = true;
                            stalled.push(format!("group_{} was told to advance to {place} and has not got nearer for {} s, {to_go:.0} short of it", group.name, (frame - group.progressed) / FRAMES_PER_SECOND));
                        }
                        if footwork[index].march && group.domain != Domain::Air {
                            marches.push((index, *to));
                        }
                    }
                }
                GroupTask::Engage { party, at, last_seen, target, searched, from, .. } => {
                    let seen: Vec<&bot_protocol::EnemyUnit> = enemies.iter().filter(|e| party.contains(&e.id)).collect();
                    let air = group.domain == Domain::Air;
                    // The leash: a ground group drawn this far from where it engaged holds where it is.
                    if !air && footwork[index].follow && centre.dist2d(*from) > FOLLOW_LEASH {
                        stalled.push(format!("group_{} was drawn {:.0} from where it engaged its party, which is running, not fighting: it holds where it is", group.name, centre.dist2d(*from)));
                        commands.extend(group.hold_orders(&units));
                        group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                        continue;
                    }
                    // A quarry that has run to a place this group never goes is left there (worlds-2, 18:28-18:43:
                    // `never: spot_20 ...` set while group_C chased a Centurion into the E3 nest, and the chase went
                    // on to the nest's turrets).
                    let quarry_at = centre_of_enemies(&seen).unwrap_or(*at);
                    if let Some((place, _)) = never_at.get(&group.name).and_then(|ps| ps.iter().find(|(_, p)| p.dist2d(quarry_at) < NEVER_REACH)) {
                        stalled.push(format!("group_{} was chasing its party to {place}, where it never goes: it holds where it is", group.name));
                        commands.extend(group.hold_orders(&units));
                        group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                        continue;
                    }
                    // A named target: the attack stands while the target is seen. Lost, a ground group holds; an air
                    // group searches the last position once and holds only much later (H-HANDS-AIR-TARGET).
                    if let Some(t) = *target {
                        if let Some(e) = seen.iter().find(|e| e.id == t) {
                            let found_again = *searched;
                            *last_seen = frame;
                            *searched = false;
                            if found_again || (e.pos.dist2d(*at) > FOLLOW_DISTANCE && frame - group.last_order >= FOLLOW_FRAMES) {
                                *at = e.pos;
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
                        if footwork[index].follow && now.dist2d(*at) > FOLLOW_DISTANCE && frame - group.last_order >= FOLLOW_FRAMES {
                            *at = now;
                            group.last_order = frame;
                            commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to: now, queue: false }));
                        }
                    } else if frame - *last_seen > LOST_FRAMES {
                        commands.extend(group.hold_orders(&units));
                        group.task = GroupTask::Hold { since: frame, committed: false };
                    }
                }
            }
            let _ = home;
        }
        for text in hunt_ends {
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            pianist.hunt_events.push(text);
        }
        for text in stalled {
            pianist.note(frame, text.clone());
            pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
            if let Some(shared) = &self.strategist {
                shared.trigger(text);
            }
        }
        self.pianist = Some(pianist);
        for (index, to) in marches {
            let mut held = std::mem::take(&mut self.pianist.as_mut().expect("pianist mode").groups[index].held);
            let members = self.pianist.as_ref().expect("pianist mode").groups[index].members.clone();
            // A straggler that cannot move (`yards.rs` `stuck`) holds nobody up (worlds-1, 16:14: the player turned
            // the march off for group_C, held by a soldier stuck at B5).
            let group: Vec<&OwnUnit> = own.iter().filter(|u| members.contains(&u.id) && !self.stuck.contains_key(&u.id)).collect();
            commands.extend(self.march(&mut held, &group, to, enemies.as_slice()));
            self.pianist.as_mut().expect("pianist mode").groups[index].held = held;
        }
    }
}

impl Brain {
    /// One tick of a group's hunt: the book kept (the quarry's last sighting for the words), hunters that died
    /// dropped, and the hunt ended when none is left. The chasing and the other ends are the micro engine's
    /// (H-MICRO-HUNT: `Commitment::Hunt` set in `micro.rs` `note_commitments`, the ends read from its output in
    /// `micro.rs` `micro`). Returns the end's words.
    fn tick_hunt(group: &mut Group, own: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit], frame: i32, commands: &mut Vec<Command>) -> Option<String> {
        let hunt = group.hunt.as_mut()?;
        hunt.hunters.retain(|id| own.iter().any(|u| u.id == *id));
        if let Some(q) = enemies.iter().find(|e| e.id == hunt.quarry) {
            hunt.last_seen = frame;
            hunt.at = q.pos;
        }
        if !hunt.hunters.is_empty() {
            return None;
        }
        let (party, since) = (hunt.party.clone(), hunt.since);
        group.hunt = None;
        let _ = commands;
        Some(format!("group_{}'s hunt of {party} ended after {} s: every hunter dead", group.name, (frame - since) / FRAMES_PER_SECOND))
    }

    /// The micro engine's word on a hunt (`micro::HuntEvent`): a hunter dropped (hurt, or slower than the quarry)
    /// rejoins the group's task; the hunt's end (the quarry dead or lost, the leash reached, no hunter left) releases
    /// every hunter and declines the party for `DECLINE_FRAMES`. Returns the end's words.
    pub(crate) fn hunt_event(group: &mut Group, event: &micro::HuntEvent, own: &[OwnUnit], frame: i32, commands: &mut Vec<Command>) -> Option<String> {
        let hunt = group.hunt.as_mut().filter(|h| h.quarry == event.quarry)?;
        match event.hunter {
            Some(hunter) => {
                hunt.hunters.retain(|id| *id != hunter);
                let dropped: Vec<&OwnUnit> = own.iter().filter(|u| u.id == hunter).collect();
                commands.extend(group.rejoin_orders(&dropped));
                None
            }
            None => {
                let (party, since, hunters) = (hunt.party.clone(), hunt.since, hunt.hunters.clone());
                group.hunt = None;
                group.declined.retain(|(_, f)| frame - f < DECLINE_FRAMES);
                group.declined.push((party.clone(), frame));
                let rejoining: Vec<&OwnUnit> = own.iter().filter(|u| hunters.contains(&u.id)).collect();
                commands.extend(group.rejoin_orders(&rejoining));
                let why = match event.why {
                    "dead" => "the quarry is dead".to_string(),
                    "lost" => format!("{party} out of sight for 6 s"),
                    "leash" => format!("the leash of {HUNT_LEASH:.0} from where it began reached"),
                    other => other.to_string(),
                };
                Some(format!("group_{}'s hunt of {party} ended after {} s: {why}; {} rejoin the group", group.name, (frame - since) / FRAMES_PER_SECOND, rejoining.len()))
            }
        }
    }

    /// Starts a hunt of the party's nearest unit by these members of the group (the previous hunt, if any, ends).
    pub(super) fn start_hunt(group: &mut Group, hunters: Vec<UnitId>, party: &Party, own: &[OwnUnit], enemies: &[bot_protocol::EnemyUnit], frame: i32, commands: &mut Vec<Command>) -> String {
        let units: Vec<&OwnUnit> = own.iter().filter(|u| hunters.contains(&u.id)).collect();
        let centre = centre_of(&units).unwrap_or(party.at);
        let quarry = party
            .ids
            .iter()
            .filter_map(|id| enemies.iter().find(|e| e.id == *id))
            .min_by(|a, b| a.pos.dist2d(centre).total_cmp(&b.pos.dist2d(centre)))
            .map(|e| (e.id, e.pos))
            .unwrap_or((party.ids[0], party.at));
        // Released; the engine's hunt commitment (set at the next `note_commitments`) does the chasing.
        commands.extend(group.release_orders(&units));
        let n = units.len();
        group.hunt = Some(Hunt { quarry: quarry.0, party: party.name.clone(), hunters, from: centre, since: frame, last_seen: frame, at: quarry.1 });
        format!("{n} of group_{} hunt {} ({})", group.name, party.name, party.composition)
    }

    /// The whole group engages the party: released, sent to fight at it, its task the engagement.
    pub(super) fn engage_group(group: &mut Group, party: &Party, own: &[OwnUnit], target: Option<UnitId>, frame: i32, commands: &mut Vec<Command>) -> String {
        group.hunt = None;
        let units = group.units(own);
        commands.extend(group.release_orders(&units));
        match target {
            Some(t) => commands.extend(units.iter().map(|u| Command::Attack { unit: u.id, target: t, queue: false })),
            None => commands.extend(units.iter().map(|u| Command::Fight { unit: u.id, to: party.at, queue: false })),
        }
        let from = centre_of(&units).unwrap_or(party.at);
        group.set_task(GroupTask::Engage { party: party.ids.clone(), at: party.at, since: frame, last_seen: frame, target, searched: false, from }, frame);
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
    use super::group_for_newcomer;

    #[test]
    fn a_newcomer_joins_the_named_group_else_its_factorys_own_and_new_starts_a_fresh_one() {
        assert_eq!(group_for_newcomer(Some("group_A"), Some("B")), Some("A".to_string()));
        assert_eq!(group_for_newcomer(None, Some("B")), Some("B".to_string()));
        assert_eq!(group_for_newcomer(Some("new"), Some("B")), None);
        assert_eq!(group_for_newcomer(None, None), None);
    }
}
