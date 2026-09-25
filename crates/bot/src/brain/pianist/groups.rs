//! Soldier groups: the pianist's unit of command. A new soldier joins the group the player named for its factory's
//! output, else its factory's own group; nothing merges by proximity (H-HANDS-GROUPS, since 2026-09-25: groups are
//! the player's, `docs/design/2026-09-25-one-decider.md` §3). A group carries one task at a time; between calls the
//! standing orders are kept up (a march arrives together, an engagement follows its party, an arrival becomes a hold).

use std::collections::HashSet;

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
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
}

impl Group {
    pub(crate) fn new(name: String, domain: Domain, members: Vec<UnitId>, task: GroupTask, frame: i32) -> Group {
        Group { name, domain, members, task, held: HashSet::new(), last_order: frame, enemies_near: false, best_to_go: f32::INFINITY, progressed: frame, stall_warned: false, parent: None, born: frame, losses: Vec::new(), losses_since: frame, loss_warned: false, last_hold: None }
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
        for (index, group) in pianist.groups.iter_mut().enumerate() {
            let units = group.units(own);
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
            let group: Vec<&OwnUnit> = own.iter().filter(|u| members.contains(&u.id)).collect();
            commands.extend(self.march(&mut held, &group, to, enemies.as_slice()));
            self.pianist.as_mut().expect("pianist mode").groups[index].held = held;
        }
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
