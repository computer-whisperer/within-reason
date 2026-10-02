//! The executor of the pass (`docs/design/2026-10-01-hands-rebuild.md`, law 7): a picked world's moves become
//! orders through the actuators (build sites, walking, fight orders, the engine's hunt), a task is remembered, the
//! actor's register says what it chose and what it left, and the decision goes into the record with its source
//! (`plan` for a pick, `list` for a step of the player's list). Code carries a picked move to its end and says the
//! end; it never starts, changes or stops one by itself.

use bot_protocol::{Command, OwnUnit, Tick, UnitId, Vec3};
use serde_json::json;

use super::super::Brain;
use super::super::economy::Plan;
use super::super::roster::Kit;
use super::compose::World;
use super::groups::Ward;
use super::menu::{ARTILLERY_REACH, His, Kind, Menu, Order, Register, Site};
use super::picture::{Party, Picture, clock};
use super::{Group, GroupTask, Task};

/// A defence ordered at a place stands this far toward the enemy from it, covering the approach.
const DEFENCE_FORWARD: f32 = 120.0;
/// A build's site within this of a place is the build at that place (a defence stands `DEFENCE_FORWARD` off it,
/// the engine's site search a building's width more).
pub(super) const SAME_SITE: f32 = 250.0;
/// Artillery shells from this share of its reach, and the screen stands at this share (footwork below the `shell`
/// move; never ruled).
const STANDOFF: f32 = 0.85;
const SCREEN: f32 = 0.55;

/// Whether a build (the spot it takes, the site asked for) is the one an order names: at the spot the order
/// names, or within `SAME_SITE` of its place (`Some(None)`: a place the picture does not know, which is no site);
/// an order naming neither is any build of its type.
fn at_the_orders_site(on: Option<usize>, near: Vec3, spot: Option<usize>, place: Option<Option<Vec3>>) -> bool {
    match (spot, place) {
        (Some(i), _) => on == Some(i),
        (None, Some(at)) => at.is_some_and(|at| at.dist2d(near) < SAME_SITE),
        (None, None) => true,
    }
}

/// A factory frame of the type a builder is told to build, standing unfinished within this of it, is helped up
/// instead of a second frame being started (game 10: a second advanced vehicle plant at 18:35, 224 from the first
/// at 0%, abandoned at 19:15; the user: two seats gifting metal does not help a seat without the build power). The
/// move's words say so (`menu.rs`).
pub(super) const FRAME_HELP: f32 = 900.0;

impl Brain {
    /// Puts a picked world in force: every actor's move other than `stay` is executed, and its register is
    /// written. Returns what changed, for the log and the `done` lines.
    pub(super) fn apply_world(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, menus: &[Menu], world: &World, commands: &mut Vec<Command>) -> Vec<String> {
        let frame = tick.frame;
        let mut done: Vec<(String, &'static str, String, String)> = Vec::new();
        // A pick answered after its group began to rove is not played on it, nor merges anyone into it (H-MICRO-ROVE).
        let roving: Vec<String> = self.pianist.as_ref().map(|p| p.groups.iter().filter(|g| g.roving).map(|g| format!("group_{}", g.name)).collect()).unwrap_or_default();
        for (menu, i) in menus.iter().zip(world) {
            let m = &menu.moves[*i];
            if *i == 0 || roving.contains(&menu.name) || matches!(&m.order, Order::Join(other) if roving.contains(&format!("group_{other}"))) {
                continue;
            }
            let did = match &menu.kind {
                Kind::Builder(id) => self.execute_builder(tick, kit, picture, *id, &m.order, menu.queue_ahead, None, commands),
                Kind::Factory(id) => self.execute_factory(tick, *id, &m.order, commands),
                Kind::Group(name) => self.execute_group(tick, picture, name, &m.order, commands),
            };
            if let Some(did) = did {
                let left = if menu.idle { String::new() } else { format!(", leaving its course ({})", menu.course_words) };
                self.pianist.as_mut().expect("pianist mode").registers.insert(menu.name.clone(), Register { frame, words: format!("picked {}{left}", m.said), left: menu.course_key.clone() });
                done.push((menu.name.clone(), menu.kind.word(), menu.id(m), did));
            }
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (actor, kind, id, did) in &done {
            pianist.done.push(format!("{} {actor}: {did} (plan)", clock(frame)));
            pianist.played.push(json!({ "actor": actor, "kind": kind, "played": id, "did": did, "source": "plan" }));
            self.journal.note_from("plan", frame, kind, json!({ "actor": actor, "move": id }), json!({ "did": did }));
        }
        done.into_iter().map(|(a, _, _, d)| format!("{a}: {d}")).collect()
    }

    /// A builder's move as orders (H-HANDS-QUEUE: `queue` orders it behind the build in progress and keeps it as
    /// the builder's next task). A list step names its words, kept so the list resumes where it was interrupted.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_builder(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, id: UnitId, order: &Order, queue: bool, list_step: Option<&str>, commands: &mut Vec<Command>) -> Option<String> {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let unit = own.iter().find(|u| u.id == id)?;
        let name = self.actor_name(id);
        let place = |named: &str| picture.places.iter().find(|p| p.name == named).cloned();
        let previous_since = self.pianist.as_ref().and_then(|p| p.tasks.get(&id)).map(Task::since);
        // H-HANDS-STARTED: the build already started, ordered again at the same site, is the same build going on,
        // not a second frame; helping the lab it already helps is the same task going on, not a new guard order.
        // A list step naming the type of the actor's current order, started or not, keeps that order (11.3: a
        // list's `corsolar` step re-issued a pick's solar 15 frames after it, and the first frame decayed).
        let (ordered_def, helping) = match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
            Some(Task::Build { def, started, .. }) if !queue && (*started || list_step.is_some()) => (Some(*def), None),
            Some(Task::Assist { lab, .. }) if !queue => (None, Some(*lab)),
            _ => (None, None),
        };
        // The same build is the same type at the same place. By type alone, a list's `extractor spot_62`,
        // `extractor spot_54` were both taken as the extractor the pick had the builder walking to at another spot
        // (fable-2-medium, 2:33; about half of 43 such steps in four games named another place). An order for
        // another place replaces the build.
        let same_site = |spot: Option<usize>, named: Option<&str>| match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
            Some(Task::Build { spot: on, near, .. }) => at_the_orders_site(*on, *near, spot, named.map(|n| place(n).map(|p| p.at))),
            _ => false,
        };
        match order {
            Order::Build(def, Site::Planned) if ordered_def == Some(*def) => return None,
            Order::Build(def, Site::Place(named)) if ordered_def == Some(*def) && same_site(None, Some(named)) => return None,
            Order::Build(_, Site::Spot(i)) if ordered_def.is_some_and(|d| self.world.is_extractor_def(d)) && same_site(Some(*i), None) => return None,
            Order::Help(target) if helping == Some(*target) => return None,
            _ => {}
        }
        let mut task: Option<Task> = None;
        let mut did: Option<String> = None;
        // A factory of the asked type already started by another builder of ours nearby is the build to help.
        let frame_to_help = match order {
            Order::Build(def, Site::Planned | Site::Place(_)) if self.world.is_factory_def(*def) => own.iter().filter(|u| u.def == *def && u.being_built && u.pos.dist2d(unit.pos) < FRAME_HELP).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))).map(|u| u.id),
            _ => None,
        };
        if let Some(target) = frame_to_help {
            if helping != Some(target) {
                commands.push(Command::Guard { unit: id, target });
            }
            let since = match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
                Some(Task::Assist { lab, since }) if *lab == target => *since,
                _ => frame,
            };
            task = Some(Task::Assist { lab: target, since });
            did = Some(format!("help {} build (a frame of that type already started {:.0} away)", self.actor_name(target), own.iter().find(|u| u.id == target).map_or(0.0, |u| u.pos.dist2d(unit.pos))));
        }
        let mut build = |plan: Plan, spot: Option<usize>| -> Option<String> {
            let (def, site) = self.build_site_for(&plan, unit, own, kit)?;
            let near = site.near;
            commands.push(Command::Build { unit: id, def, site: Some(site), queue });
            task = Some(Task::Build { def, near, spot, ordered: frame, started: false });
            Some(format!("build a {} at {}", self.name(def), self.place_words(&picture.places, near)))
        };
        match order {
            _ if frame_to_help.is_some() => {}
            Order::Build(_, Site::Spot(i)) => {
                did = build(Plan::Extractor(self.world.hello.metal_spots[*i]), Some(*i));
            }
            Order::Build(def, Site::Planned) => {
                let plan = self.place_planned(*def, unit, own, kit);
                did = build(plan, None);
            }
            // A tier-2 extractor goes over the extractor of ours at the place; one already upgrading there is
            // helped to finish, not ordered again.
            Order::Build(def, Site::Place(place_name)) if self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0) => {
                let radius = self.spot_occupied_radius();
                let asked = place(place_name).map(|p| p.at);
                match asked.and_then(|at| own.iter().find(|u| u.def == *def && u.being_built && u.pos.dist2d(at) < radius)) {
                    Some(under_way) => {
                        commands.push(Command::Repair { unit: id, target: under_way.id, queue });
                        task = Some(Task::Repair { target: under_way.id, since: frame });
                        did = Some(format!("help finish the {} already under way at {}", self.name(*def), self.place_words(&picture.places, under_way.pos)));
                    }
                    None => match asked.and_then(|at| own.iter().find(|u| kit.is_extractor(u.def) && u.def != *def && !u.being_built && u.pos.dist2d(at) < radius)) {
                        Some(ours) => did = build(Plan::Near(*def, ours.pos), None),
                        None => did = Some(format!("nothing to do: no extractor of ours stands at {place_name} to upgrade into a {}", self.name(*def))),
                    },
                }
            }
            Order::Build(def, Site::Place(place_name)) => {
                let at = place(place_name).map_or(unit.pos, |p| p.at);
                // A defence stands toward the enemy from the place, so its reach covers the approach rather than
                // whichever side the engine's site search found free (player-10-routes; the user, 2026-09-28: "our
                // turrets are not built in reasonable locations to defend the points of interest intended").
                let at = if self.world.def(*def).is_some_and(|d| d.speed == 0.0 && d.weapon_count > 0) {
                    let enemy = self.enemy_base(at);
                    let (dx, dz) = (enemy.x - at.x, enemy.z - at.z);
                    let len = dx.hypot(dz).max(1.0);
                    Vec3 { x: at.x + dx / len * DEFENCE_FORWARD, y: at.y, z: at.z + dz / len * DEFENCE_FORWARD }
                } else {
                    at
                };
                // A building that stands on water keeps the water site: snapped to the builder's own ground, a
                // shipyard's mark became the nearest land cell and the engine had nowhere to put it (Cape Violet,
                // 2026-09-27). The engine walks the builder to the shore.
                let on_water = super::glossary::entry(self.name(*def)).is_some_and(|e| e.has_flag("on_water"));
                let site = if on_water { at } else { self.snap_for(self.walker_of(unit.def), at) };
                did = build(Plan::Near(*def, site), None);
            }
            Order::Help(target) => {
                // Queued, the guard order is given when the build finishes (`Pianist::promote`).
                if !queue {
                    commands.push(Command::Guard { unit: id, target: *target });
                }
                task = Some(Task::Assist { lab: *target, since: frame });
                did = Some(format!("help {} build", self.actor_name(*target)));
            }
            Order::TakeApart(at) => {
                let wrecks = self.wrecks_to_take(*at, unit);
                if !wrecks.is_empty() {
                    commands.extend(wrecks.into_iter().enumerate().map(|(n, feature)| Command::ReclaimFeature { unit: id, feature, queue: queue || n > 0 }));
                    task = Some(Task::Reclaim { at: *at, since: frame });
                    did = Some(format!("reclaim wrecks at {}", self.place_words(&picture.places, *at)));
                }
            }
            Order::TakeApartUnit(target) => {
                commands.push(Command::ReclaimUnit { unit: id, target: *target, queue });
                task = Some(Task::ReclaimUnit { target: *target, since: frame });
                did = Some(format!("take apart our {}", own.iter().find(|u| u.id == *target).map_or("unit".to_string(), |u| self.handle(u))));
            }
            Order::Repair(target) => {
                commands.push(Command::Repair { unit: id, target: *target, queue });
                task = Some(Task::Repair { target: *target, since: frame });
                did = Some("repair".into());
            }
            Order::DGun(party_name) => {
                if let Some(party) = picture.parties.iter().find(|p| p.name == *party_name)
                    && let Some(target) = party.ids.iter().filter_map(|id| tick.snapshot.enemies.iter().find(|e| e.id == *id)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)))
                {
                    commands.push(Command::DGun { unit: id, target: target.id });
                    // The shot takes the place of the builder's order in the engine: its task ends with it.
                    self.pianist.as_mut().expect("pianist mode").tasks.remove(&id);
                    did = Some(format!("D-gun {}'s nearest unit", party.name));
                }
            }
            Order::Go(place_name) => {
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(self.walker_of(unit.def), p.at);
                    commands.push(Command::Move { unit: id, to, queue });
                    task = Some(Task::Walk { to, place: p.name.clone(), since: frame });
                    did = Some(format!("walk to {}", p.name));
                }
            }
            Order::Follow(Ward::Group(group)) => {
                let at = self.pianist.as_ref().and_then(|p| p.groups.iter().find(|g| g.name == *group)).and_then(|g| g.body(own, None)).map(|b| b.at);
                if let Some(at) = at {
                    let to = self.snap_for(self.walker_of(unit.def), at);
                    commands.push(Command::Move { unit: id, to, queue });
                    task = Some(Task::Follow { group: group.clone(), at: to, since: frame });
                    did = Some(format!("follow group_{group}"));
                }
            }
            Order::Attack(party_name) => {
                if let Some(party) = picture.parties.iter().find(|p| p.name == *party_name) {
                    commands.push(Command::Fight { unit: id, to: party.at, queue });
                    task = Some(Task::Attack { party: party_name.clone(), to: party.at, since: frame });
                    did = Some(format!("attack {party_name}"));
                }
            }
            _ => {}
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if let Some(task) = task {
            if queue {
                pianist.queued.insert(id, task);
                did = did.map(|d| format!("next, queued: {d}"));
            } else {
                pianist.tasks.insert(id, task);
            }
        }
        // The list step this builder is on, and its return to the list when the pass takes the builder from the
        // task it became: the list resumes where it was interrupted, not one step on.
        match list_step {
            Some(step) => {
                if !queue {
                    pianist.list_steps.insert(id, (step.to_string(), frame));
                } else {
                    // Kept with the queued task and moved to `list_steps` when it is promoted: unrecorded, a queued
                    // `assist 25` read as the step before it once the plant stood, and the next step was ordered the
                    // same second (bluegecko-3v1-comet-catcher-10 1:04-1:11, player-11 1:05-1:13).
                    pianist.queued_steps.insert(id, step.to_string());
                }
            }
            None => {
                if !queue
                    && did.is_some()
                    && let Some((step, at)) = pianist.list_steps.remove(&id)
                    && previous_since == Some(at)
                    && let Some(list) = pianist.scripts.get_mut(&name)
                {
                    list.push_front(step.clone());
                    did = did.map(|d| format!("{d}; its list step '{step}' waits for it"));
                }
                // A way out the pick gave a builder with a list holds the list while the enemy is still on the
                // builder (`Pianist::list_is_held`): the list ordered its step again the next second otherwise
                // (player-31: 83 of 92 such walks undone within three seconds).
                if !queue && did.is_some() && matches!(order, Order::Go(_) | Order::Follow(_) | Order::Attack(_)) && pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) {
                    pianist.list_held.insert(id);
                }
            }
        }
        did
    }

    pub(super) fn execute_factory(&mut self, tick: &Tick, id: UnitId, order: &Order, commands: &mut Vec<Command>) -> Option<String> {
        let Order::Make(def) = order else { return None };
        commands.push(Command::Build { unit: id, def: *def, site: None, queue: false });
        self.pianist.as_mut().expect("pianist mode").lab_queue.entry(id).or_default().push((*def, tick.frame));
        Some(format!("build a {}", self.name(*def)))
    }

    /// A group's move as orders.
    pub(super) fn execute_group(&mut self, tick: &Tick, picture: &Picture, group_name: &str, order: &Order, commands: &mut Vec<Command>) -> Option<String> {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let home = self.home;
        let place = |named: &str| picture.places.iter().find(|p| p.name == named).cloned();
        let party = |named: &str| picture.parties.iter().find(|p| p.name == named);
        let mut pianist = self.pianist.take()?;
        let Some(index) = pianist.groups.iter().position(|g| g.name == group_name) else {
            self.pianist = Some(pianist);
            return None;
        };
        let units: Vec<&OwnUnit> = pianist.groups[index].units(own);
        let ids: Vec<UnitId> = units.iter().map(|u| u.id).collect();
        let centre = super::groups::centre_of(&units);
        let walker = self.group_walker(&pianist.groups[index], own);
        let mut did: Option<String> = None;
        match order {
            Order::Hold => {
                let group = &mut pianist.groups[index];
                commands.extend(group.hold_orders(&units));
                group.set_task(GroupTask::Hold { since: frame, committed: false, picked: true }, frame);
                group.last_order = frame;
                did = Some("hold where it stands".into());
            }
            Order::Gather => {
                // The group closes up on its front: toward its goal, else toward the nearest enemy, else its centre.
                let group = &mut pianist.groups[index];
                let toward = match &group.task {
                    GroupTask::Move { to, .. } => Some(*to),
                    GroupTask::Engage { at, .. } | GroupTask::Follow { at, .. } => Some(*at),
                    GroupTask::Hunt(h) => Some(h.at),
                    GroupTask::Hold { .. } => centre.and_then(|c| enemies.iter().map(|e| e.pos).min_by(|a, b| a.dist2d(c).total_cmp(&b.dist2d(c)))),
                };
                if let Some(body) = group.body(own, toward) {
                    let to = self.snap_for(walker, body.front);
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                    group.set_task(GroupTask::Move { to, place: "its front".into(), fight: false, since: frame }, frame);
                    group.gathering = true;
                    group.last_order = frame;
                    did = Some("gather on its front".into());
                }
            }
            Order::Go(place_name) | Order::FightTo(place_name) => {
                let fight = matches!(order, Order::FightTo(_));
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(walker, p.at);
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| if fight { Command::Fight { unit: *id, to, queue: false } } else { Command::Move { unit: *id, to, queue: false } }));
                    group.set_task(GroupTask::Move { to, place: p.name.clone(), fight, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("{} to {}", if fight { "advance" } else { "walk" }, p.name));
                }
            }
            Order::Attack(party_name) => {
                if let Some(party) = party(party_name) {
                    let group = &mut pianist.groups[index];
                    // Every air group's engagement names a target (a fight order makes a bomber bomb the nearest
                    // thing on its line, not the party).
                    let target = (group.domain == crate::world::Domain::Air).then(|| self.party_target(party, enemies)).flatten().map(|(id, _)| id);
                    did = Some(Brain::engage_group(group, party, own, target, frame, commands));
                }
            }
            Order::Send(hunters, party_name) => {
                if let Some(party) = party(party_name) {
                    did = Brain::start_hunt(&mut pianist, index, hunters, party, own, enemies, frame, commands);
                }
            }
            Order::Shell(target) => {
                let aim = match target {
                    His::Party(name) => party(name).map(|p| (p.at, p.name.clone())),
                    His::Place(name) => place(name).map(|p| (p.at, format!("his buildings at {}", p.name))),
                };
                if let Some((at, what)) = aim {
                    let group = &mut pianist.groups[index];
                    let reach_of = |u: &&OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.reach);
                    let long: Vec<&OwnUnit> = units.iter().copied().filter(|u| reach_of(u) >= ARTILLERY_REACH).collect();
                    let reach = long.iter().map(reach_of).fold(0.0, f32::max);
                    let from = centre.unwrap_or(at);
                    let (dx, dz) = (from.x - at.x, from.z - at.z);
                    let len = dx.hypot(dz).max(1.0);
                    let point = |dist: f32| Vec3 { x: at.x + dx / len * dist, y: 0.0, z: at.z + dz / len * dist };
                    let standoff = self.snap_for(walker, point(reach * STANDOFF));
                    let screen = self.snap_for(walker, point(reach * SCREEN));
                    commands.extend(group.release_orders(&units));
                    for u in &units {
                        let to = if reach_of(u) >= ARTILLERY_REACH { standoff } else { screen };
                        commands.push(Command::Fight { unit: u.id, to, queue: false });
                    }
                    group.set_task(GroupTask::Move { to: standoff, place: format!("standoff from {what}"), fight: true, since: frame }, frame);
                    group.shelling = true;
                    group.last_order = frame;
                    did = Some(format!("shell {what} from their reach with {} long-reach soldiers, {} between", long.len(), units.len() - long.len()));
                }
            }
            Order::Join(other) => {
                if let Some(target) = pianist.groups.iter().position(|g| g.name == *other && g.domain == pianist.groups[index].domain) {
                    let members = std::mem::take(&mut pianist.groups[index].members);
                    let to = super::groups::centre_of(&pianist.groups[target].units(own)).or(centre).unwrap_or(home);
                    commands.extend(members.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                    pianist.groups[target].members.extend(members);
                    pianist.groups.remove(index);
                    did = Some(format!("join group_{other}"));
                }
            }
            Order::Follow(ward) => {
                let found = match ward {
                    Ward::Builder(id) => own.iter().find(|u| u.id == *id).map(|u| (u.pos, self.actor_name(*id))),
                    Ward::Group(other) => pianist.groups.iter().find(|g| g.name == *other).and_then(|g| g.body(own, None)).map(|b| (b.at, format!("group_{other}"))),
                };
                if let Some((at, ward_name)) = found {
                    let to = self.snap_for(walker, at);
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| Command::Fight { unit: *id, to, queue: false }));
                    group.set_task(GroupTask::Follow { ward: ward.clone(), name: ward_name.clone(), at: to, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("follow {ward_name}"));
                }
            }
            Order::Scout => {
                // The group's fastest soldier becomes a group of its own that roves (H-MICRO-ROVE): the lane picks
                // where it looks from the next tick, and no move of the hands' moves it (player-9-posing: three
                // scouts walked at his base by the hands' Moves, pulled home, into two Pawns, onto a hunt).
                let speed = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.speed);
                if let Some(scout) = units.iter().copied().max_by(|a, b| speed(a).total_cmp(&speed(b)))
                    && units.len() >= 2
                {
                    let domain = pianist.groups[index].domain;
                    pianist.groups[index].members.retain(|id| *id != scout.id);
                    let name = pianist.new_group_name();
                    did = Some(format!("send a {} as group_{name} to rove", self.name(scout.def)));
                    let parent_name = pianist.groups[index].name.clone();
                    let mut group = Group::new(name, domain, vec![scout.id], GroupTask::Hold { since: frame, committed: false, picked: false }, frame).split_from(&parent_name);
                    // Roving from its first tick: no menu for it meanwhile.
                    (group.scout, group.roving) = (true, true);
                    pianist.groups.push(group);
                }
            }
            _ => {}
        }
        self.pianist = Some(pianist);
        did
    }

    /// The unit of a party an air group attacks by name: its commander when one is there, else its dearest member
    /// seen.
    pub(super) fn party_target(&self, party: &Party, enemies: &[bot_protocol::EnemyUnit]) -> Option<(UnitId, &'static str)> {
        let seen: Vec<&bot_protocol::EnemyUnit> = enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
        if let Some(c) = seen.iter().find(|e| e.def.is_some_and(|d| self.world.is_commander_def(d))) {
            return Some((c.id, "commander"));
        }
        seen.iter()
            .filter(|e| e.def.is_some())
            .max_by(|a, b| {
                let metal = |e: &bot_protocol::EnemyUnit| e.def.and_then(|d| self.world.def(d)).map_or(0.0, |d| d.metal_cost);
                metal(a).total_cmp(&metal(b))
            })
            .map(|e| (e.id, "dearest unit"))
    }
}

#[cfg(test)]
mod tests {
    use super::at_the_orders_site;
    use bot_protocol::Vec3;

    /// An order is the build under way only at the same place: `extractor spot_54` is not the extractor the
    /// builder is walking to at spot_62, and `armllt spot_43` is the turret ordered beside spot_43, not one elsewhere.
    #[test]
    fn an_order_is_the_build_under_way_only_at_its_own_place() {
        let at = |x: f32, z: f32| Vec3 { x, y: 0.0, z };
        assert!(at_the_orders_site(Some(62), at(576.0, 4608.0), Some(62), None));
        assert!(!at_the_orders_site(Some(62), at(576.0, 4608.0), Some(54), None));
        assert!(!at_the_orders_site(None, at(576.0, 4608.0), Some(54), None), "a build that is no extractor's spot");
        // A turret stands up to 120 toward the enemy from the place named, the site search a little more.
        assert!(at_the_orders_site(None, at(2130.0, 3360.0), None, Some(Some(at(2032.0, 3360.0)))));
        assert!(!at_the_orders_site(None, at(1216.0, 3408.0), None, Some(Some(at(2032.0, 3360.0)))));
        assert!(!at_the_orders_site(None, at(2032.0, 3360.0), None, Some(None)), "a place the picture does not know");
        // An order naming no place (`armsolar`) is any build of its type.
        assert!(at_the_orders_site(None, at(100.0, 100.0), None, None));
    }
}
