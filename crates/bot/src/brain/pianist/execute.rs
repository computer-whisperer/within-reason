//! The executor of the pass (`docs/design/2026-09-26-one-pass.md` §5): a world's states become orders through the
//! existing actuators (build sites, walking, the march, fight orders, the engine's hunt), a task is remembered,
//! and the decision goes into the record with its source (`rule` for the base world, `plan` for a pick, `list` for
//! a step of the player's list).

use bot_protocol::{Command, OwnUnit, Tick, UnitId, Vec3};
use serde_json::json;

use super::super::economy::Plan;
use super::super::roster::Kit;
use super::super::Brain;
use super::groups::LAST_HOLD;
use super::picture::{Party, Picture, clock};
use super::plan::{Kind, Response, Slot, State, World};
use super::{Group, GroupTask, Task};

/// The soldiers of a group nearest a point, for a detachment.
pub(crate) fn nearest_of<'a>(units: &[&'a OwnUnit], to: bot_protocol::Vec3, n: usize) -> Vec<&'a OwnUnit> {
    let mut sorted: Vec<&OwnUnit> = units.to_vec();
    sorted.sort_by(|a, b| a.pos.dist2d(to).total_cmp(&b.pos.dist2d(to)));
    sorted.into_iter().take(n).collect()
}

fn kind_of(actor: &str) -> &'static str {
    if actor.starts_with("group_") {
        "group"
    } else if actor.starts_with("lab_") || actor.starts_with("plant_") || actor.starts_with("factory_") {
        "lab"
    } else {
        "builder"
    }
}

impl Brain {
    /// Puts a world's states in force: a state already current stands; `Leave` ends a current hunt or engagement
    /// of that party and declines it for a while; every other state is executed. Returns what changed, for the
    /// log and the `done` lines.
    /// `held` marks the slots this world does not touch: the pass holds every open slot when a question goes out,
    /// and the pick then plays those in full, base state included.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_plan(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, slots: &[Slot], world: &World, source: &'static str, held: &[bool], commands: &mut Vec<Command>) -> Vec<String> {
        let frame = tick.frame;
        let mut done: Vec<(String, String, String)> = Vec::new();
        for (i, (slot, si)) in slots.iter().zip(world).enumerate() {
            let state = &slot.states[*si];
            if state.current || held.get(i).copied().unwrap_or(false) {
                continue;
            }
            // A slot with nothing to decide had its base put in force by the rule when the gate was asked; the
            // pick does not order it twice (onepass-smoke-2: a constructor's extractor ordered twice in one second,
            // by the rule and by w1). An open slot was held for the pick and is played here whatever it chose.
            if source == "plan" && !slot.open() && *si == slot.base() {
                continue;
            }
            match (&slot.kind, &state.response) {
                (Kind::Threat(party, _), Response::Leave) => {
                    for (actor, did) in self.leave_party(party, &tick.snapshot.own_units, frame, commands) {
                        done.push((actor, state.id.clone(), did));
                    }
                }
                (_, Response::Keep) => {}
                _ => {
                    if let Some(did) = self.execute(tick, kit, picture, slot, state, commands) {
                        done.push((state.actor.clone(), state.id.clone(), did));
                    }
                }
            }
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (actor, id, did) in &done {
            if source == "plan" {
                pianist.picked.insert(actor.clone(), frame);
            }
            pianist.done.push(format!("{} {actor}: {did} ({source})", clock(frame)));
            pianist.played.push(json!({ "actor": actor, "kind": kind_of(actor), "played": id, "did": did, "source": source }));
            self.journal.note_from(source, frame, kind_of(actor), json!({ "actor": actor, "state": id }), json!({ "did": did }));
        }
        done.into_iter().map(|(a, _, d)| format!("{a}: {d}")).collect()
    }

    /// Nobody moves for the party: a group hunting it or engaging it stops and declines it for a while.
    fn leave_party(&mut self, party: &Party, own: &[OwnUnit], frame: i32, commands: &mut Vec<Command>) -> Vec<(String, String)> {
        let mut done = Vec::new();
        let Some(pianist) = self.pianist.as_mut() else { return done };
        for group in pianist.groups.iter_mut() {
            if group.hunt.as_ref().is_some_and(|h| party.ids.contains(&h.quarry)) {
                let hunters: Vec<UnitId> = group.hunt.as_ref().map(|h| h.hunters.clone()).unwrap_or_default();
                group.hunt = None;
                group.declined.push((party.name.clone(), frame));
                let units: Vec<&OwnUnit> = own.iter().filter(|u| hunters.contains(&u.id)).collect();
                commands.extend(group.rejoin_orders(&units));
                done.push((format!("group_{}", group.name), format!("its hunt of {} called off", party.name)));
            }
            if matches!(&group.task, GroupTask::Engage { party: ids, .. } if ids.iter().any(|id| party.ids.contains(id))) {
                let units = group.units(own);
                commands.extend(group.hold_orders(&units));
                group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                group.declined.push((party.name.clone(), frame));
                done.push((format!("group_{}", group.name), format!("leaves {} and holds", party.name)));
            }
        }
        done
    }

    fn execute(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, slot: &Slot, state: &State, commands: &mut Vec<Command>) -> Option<String> {
        match &slot.kind {
            Kind::Builder(id) => self.execute_builder(tick, kit, picture, *id, &state.response, slot.queue_ahead, None, commands),
            Kind::Lab(id) => self.execute_lab(tick, *id, &state.response, commands),
            Kind::Group(name) => self.execute_group(tick, picture, name, &state.response, None, commands),
            Kind::Threat(party, _) => {
                if let Some(name) = state.actor.strip_prefix("group_") {
                    let name = name.to_string();
                    return self.execute_group(tick, picture, &name, &state.response, Some(party), commands);
                }
                // A builder's attack: the state's actor names the unit.
                let id = tick.snapshot.own_units.iter().find(|u| self.actor_name(u.id) == state.actor)?.id;
                self.execute_builder(tick, kit, picture, id, &state.response, false, None, commands)
            }
        }
    }

    /// A builder's state as orders (H-HANDS-QUEUE: `queue` orders it behind the build in progress and keeps it as
    /// the builder's next task). A list step names its words, kept so the list resumes where it was interrupted.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_builder(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, id: UnitId, response: &Response, queue: bool, list_step: Option<&str>, commands: &mut Vec<Command>) -> Option<String> {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let unit = own.iter().find(|u| u.id == id)?;
        let name = self.actor_name(id);
        let place = |named: &str| picture.places.iter().find(|p| p.name == named).cloned();
        let previous_since = self.pianist.as_ref().and_then(|p| p.tasks.get(&id)).map(Task::since);
        // H-HANDS-STARTED: the kind of building already started, ordered again, is the same build going on, not a
        // second frame. Helping the lab it already helps is the same task going on, not a new guard order.
        let (started_def, helping) = match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
            Some(Task::Build { def, started: true, .. }) if !queue => (Some(*def), None),
            Some(Task::Assist { lab, .. }) if !queue => (None, Some(*lab)),
            _ => (None, None),
        };
        match response {
            Response::Building(def) | Response::BuildingAt(def, _) if started_def == Some(*def) => return None,
            Response::Extractor(_) if started_def.is_some_and(|d| self.world.is_extractor_def(d)) => return None,
            Response::Assist(target) if helping == Some(*target) => return None,
            _ => {}
        }
        let mut task: Option<Task> = None;
        let mut did: Option<String> = None;
        let mut build = |plan: Plan, spot: Option<usize>| -> Option<String> {
            let (def, site) = self.build_site_for(&plan, unit, own, kit)?;
            let near = site.near;
            commands.push(Command::Build { unit: id, def, site: Some(site), queue });
            task = Some(Task::Build { def, near, spot, ordered: frame, started: false });
            Some(format!("build a {} at {}", self.name(def), self.place_words(&picture.places, near)))
        };
        match response {
            Response::Keep => {}
            Response::Extractor(i) => {
                did = build(Plan::Extractor(self.world.hello.metal_spots[*i]), Some(*i));
            }
            Response::Building(def) => {
                let plan = self.place_planned(*def, unit, own, kit);
                did = build(plan, None);
            }
            // A tier-2 extractor goes over one of our basic extractors; one already upgrading at the place is helped
            // to finish, not ordered again.
            Response::BuildingAt(def, place_name) if self.world.def(*def).is_some_and(|d| d.extracts_metal > 0.0) => {
                let radius = self.spot_occupied_radius();
                let asked = place(place_name).map(|p| p.at);
                let upgradable = |u: &&OwnUnit| kit.is_extractor(u.def) && u.def != *def && !u.being_built && !own.iter().any(|f| f.def == *def && f.pos.dist2d(u.pos) < radius);
                match asked.and_then(|at| own.iter().find(|u| u.def == *def && u.being_built && u.pos.dist2d(at) < radius)) {
                    Some(under_way) => {
                        commands.push(Command::Repair { unit: id, target: under_way.id, queue });
                        task = Some(Task::Repair { target: under_way.id, since: frame });
                        did = Some(format!("help finish the {} already under way at {}", self.name(*def), self.place_words(&picture.places, under_way.pos)));
                    }
                    None => {
                        let target = asked.and_then(|at| own.iter().filter(upgradable).find(|u| u.pos.dist2d(at) < radius)).or_else(|| own.iter().filter(upgradable).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))));
                        match target {
                            Some(ours) => did = build(Plan::Near(*def, ours.pos), None),
                            None => did = Some(format!("nothing to do: no extractor of ours is left to upgrade into a {}", self.name(*def))),
                        }
                    }
                }
            }
            Response::BuildingAt(def, place_name) => {
                let at = place(place_name).map_or(unit.pos, |p| p.at);
                // A building that stands on water keeps the water site: snapped to the builder's own ground, a
                // shipyard's mark became the nearest land cell and the engine had nowhere to put it (Cape Violet,
                // 2026-09-27: three builders sent to yards never moved). The engine walks the builder to the shore.
                let on_water = super::glossary::entry(self.name(*def)).is_some_and(|e| e.has_flag("on_water"));
                let site = if on_water { at } else { self.snap_for(self.walker_of(unit.def), at) };
                did = build(Plan::Near(*def, site), None);
            }
            Response::Assist(target) => {
                // Queued, the guard order is given when the build finishes (`Pianist::promote`).
                if !queue {
                    commands.push(Command::Guard { unit: id, target: *target });
                }
                task = Some(Task::Assist { lab: *target, since: frame });
                did = Some(format!("help {} build", self.actor_name(*target)));
            }
            Response::Reclaim(at) => {
                let wrecks = self.wrecks_to_take(*at, unit);
                if !wrecks.is_empty() {
                    commands.extend(wrecks.into_iter().enumerate().map(|(n, feature)| Command::ReclaimFeature { unit: id, feature, queue: queue || n > 0 }));
                    task = Some(Task::Reclaim { at: *at, since: frame });
                    did = Some(format!("reclaim wrecks at {}", self.place_words(&picture.places, *at)));
                }
            }
            Response::ReclaimUnit(target) => {
                commands.push(Command::ReclaimUnit { unit: id, target: *target, queue });
                task = Some(Task::ReclaimUnit { target: *target, since: frame });
                did = Some(format!("take apart our {}", own.iter().find(|u| u.id == *target).map_or("unit".to_string(), |u| self.handle(u))));
            }
            Response::Repair(target) => {
                commands.push(Command::Repair { unit: id, target: *target, queue });
                task = Some(Task::Repair { target: *target, since: frame });
                did = Some("repair".into());
            }
            Response::WalkTo(place_name) => {
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(self.walker_of(unit.def), p.at);
                    commands.push(Command::Move { unit: id, to, queue });
                    task = Some(Task::Walk { to, place: p.name.clone(), since: frame });
                    did = Some(format!("walk to {}", p.name));
                }
            }
            Response::Attack(party_name) => {
                if let Some(party) = picture.parties.iter().find(|p| p.name == *party_name) {
                    commands.push(Command::Fight { unit: id, to: party.at, queue });
                    task = Some(Task::Walk { to: party.at, place: party_name.clone(), since: frame });
                    did = Some(format!("attack {party_name}"));
                }
            }
            Response::RetreatHome => {
                match self.world.is_commander_def(unit.def).then(|| self.commander_waypoint_home(unit.pos)).flatten() {
                    Some(waypoint) => {
                        commands.push(Command::Move { unit: id, to: waypoint, queue });
                        commands.push(Command::Move { unit: id, to: self.home, queue: true });
                    }
                    None => commands.push(Command::Move { unit: id, to: self.home, queue }),
                }
                task = Some(Task::Walk { to: self.home, place: "home".into(), since: frame });
                did = Some("go home".into());
            }
            _ => {}
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if matches!(response, Response::RetreatHome) {
            pianist.retreated.insert(id, frame);
        }
        if let Some(task) = task {
            if let Task::Build { def, .. } = &task {
                if let Some(d) = self.world.def(*def) {
                    pianist.ordered.entry(id).or_default().push(d.name.clone());
                }
            }
            if queue {
                pianist.queued.insert(id, task);
                did = did.map(|d| format!("next, queued: {d}"));
            } else {
                pianist.tasks.insert(id, task);
            }
        }
        // The list step this builder is on, and its return to the list when the pass diverts the builder from the
        // task it became: the list resumes where it was interrupted, not one step on.
        match list_step {
            Some(step) => {
                if !queue {
                    pianist.list_steps.insert(id, (step.to_string(), frame));
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
            }
        }
        did
    }

    fn execute_lab(&mut self, tick: &Tick, id: UnitId, response: &Response, commands: &mut Vec<Command>) -> Option<String> {
        let Response::Next(def) = response else { return None };
        commands.push(Command::Build { unit: id, def: *def, site: None, queue: false });
        self.pianist.as_mut().expect("pianist mode").lab_queue.entry(id).or_default().push((*def, tick.frame));
        Some(format!("build a {}", self.name(*def)))
    }

    /// A group's state as orders: against a party (the threat family) or on its course.
    pub(super) fn execute_group(&mut self, tick: &Tick, picture: &Picture, group_name: &str, response: &Response, party: Option<&Party>, commands: &mut Vec<Command>) -> Option<String> {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let enemies = tick.snapshot.enemies.as_slice();
        let home = self.home;
        let place = |named: &str| picture.places.iter().find(|p| p.name == named).cloned();
        let Some(mut pianist) = self.pianist.take() else { return None };
        let Some(index) = pianist.groups.iter().position(|g| g.name == group_name) else {
            self.pianist = Some(pianist);
            return None;
        };
        let units: Vec<&OwnUnit> = pianist.groups[index].units(own);
        let ids: Vec<UnitId> = units.iter().map(|u| u.id).collect();
        let centre = super::groups::centre_of(&units);
        let mut did: Option<String> = None;
        match response {
            Response::Whole => {
                if let Some(party) = party {
                    let group = &mut pianist.groups[index];
                    // Every air group's engagement names a target (a fight order makes a bomber bomb the nearest
                    // thing on its line, not the party).
                    let target = (group.domain == crate::world::Domain::Air).then(|| self.party_target(party, enemies)).flatten().map(|(id, _)| id);
                    did = Some(Brain::engage_group(group, party, own, target, frame, commands));
                }
            }
            Response::Hunt(hunters) => {
                if let Some(party) = party {
                    did = Some(Brain::start_hunt(&mut pianist.groups[index], hunters.clone(), party, own, enemies, frame, commands));
                }
            }
            Response::Back(place_name) => {
                if let Some(p) = place(place_name) {
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to: p.at, queue: false }));
                    group.set_task(GroupTask::Move { to: p.at, place: p.name.clone(), fight: false, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("fall back to {}{}", p.name, party.map_or(String::new(), |p| format!(" from {}", p.name))));
                }
            }
            Response::Hold => {
                let group = &mut pianist.groups[index];
                commands.extend(group.hold_orders(&units));
                group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                did = Some("hold".into());
            }
            Response::Walk { place: place_name, fight } => {
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    let group = &mut pianist.groups[index];
                    group.hunt = None;
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| if *fight { Command::Fight { unit: *id, to, queue: false } } else { Command::Move { unit: *id, to, queue: false } }));
                    group.set_task(GroupTask::Move { to, place: p.name.clone(), fight: *fight, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("{} to {}", if *fight { "advance" } else { "walk" }, p.name));
                }
            }
            Response::Retreat => {
                let group = &mut pianist.groups[index];
                group.hunt = None;
                commands.extend(group.release_orders(&units));
                commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to: home, queue: false }));
                group.set_task(GroupTask::Move { to: home, place: "home".into(), fight: false, since: frame }, frame);
                group.last_order = frame;
                did = Some("fall back home".into());
            }
            Response::FallBack => {
                if let Some(to) = pianist.groups[index].last_hold {
                    let place = format!("{} {}", LAST_HOLD, self.place_words(&picture.places, to));
                    let group = &mut pianist.groups[index];
                    group.hunt = None;
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                    group.set_task(GroupTask::Move { to, place: place.clone(), fight: false, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("fall back to {place}"));
                }
            }
            Response::Split(n, place_name) => {
                if let Some(p) = place(place_name) {
                    let n = (*n).clamp(1, units.len().saturating_sub(1).max(1));
                    let domain = pianist.groups[index].domain;
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    let free: Vec<&OwnUnit> = pianist.groups[index].free_units(own);
                    let detached: Vec<UnitId> = nearest_of(&free, to, n).iter().map(|u| u.id).collect();
                    pianist.groups[index].members.retain(|id| !detached.contains(id));
                    commands.extend(detached.iter().flat_map(|id| [Command::MoveState { unit: *id, state: 1 }].into_iter().filter(|_| domain == crate::world::Domain::Air).chain([Command::Fight { unit: *id, to, queue: false }])));
                    let name = pianist.new_group_name();
                    did = Some(format!("send {} soldiers as group_{name} to {}", detached.len(), p.name));
                    let parent_name = pianist.groups[index].name.clone();
                    pianist.groups.push(Group::new(name, domain, detached, GroupTask::Move { to, place: p.name.clone(), fight: true, since: frame }, frame).split_from(&parent_name));
                }
            }
            Response::Scout => {
                // A scout to where the group stands looks at nothing.
                if let Some(p) = centre.and_then(|c| self.scout_target(c, picture, frame)).filter(|p| centre.is_none_or(|c| c.dist2d(p.at) > 600.0)) {
                    let domain = pianist.groups[index].domain;
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    // The fastest soldier of the group goes: a raider by the glossary's class when there is one.
                    let raider = |u: &&OwnUnit| super::glossary::entry(self.name(u.def)).is_some_and(|e| e.class.contains("raider") || e.class.contains("scout"));
                    let free: Vec<&OwnUnit> = pianist.groups[index].free_units(own);
                    let scout = free.iter().copied().filter(raider).min_by(|a, b| a.pos.dist2d(to).total_cmp(&b.pos.dist2d(to))).or_else(|| nearest_of(&free, to, 1).first().copied());
                    if let Some(scout) = scout {
                        pianist.groups[index].members.retain(|id| *id != scout.id);
                        if domain == crate::world::Domain::Air {
                            commands.push(Command::MoveState { unit: scout.id, state: 1 });
                        }
                        commands.push(Command::Move { unit: scout.id, to, queue: false });
                        let name = pianist.new_group_name();
                        did = Some(format!("send a {} as group_{name} to look at {}", self.name(scout.def), p.name));
                        let parent_name = pianist.groups[index].name.clone();
                        pianist.groups.push(Group::new(name, domain, vec![scout.id], GroupTask::Move { to, place: p.name.clone(), fight: false, since: frame }, frame).split_from(&parent_name));
                    }
                }
            }
            Response::Gather(place_name) => {
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    let group = &mut pianist.groups[index];
                    group.hunt = None;
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                    group.set_task(GroupTask::Move { to, place: p.name.clone(), fight: false, since: frame }, frame);
                    group.gathering = true;
                    group.last_order = frame;
                    did = Some(format!("gather at {}", p.name));
                }
            }
            Response::Shell(party_name) => {
                if let Some(party) = party.filter(|p| p.name == *party_name).or_else(|| picture.parties.iter().find(|p| p.name == *party_name)) {
                    let group = &mut pianist.groups[index];
                    let reach_of = |u: &&OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.reach);
                    let long: Vec<&OwnUnit> = units.iter().copied().filter(|u| reach_of(u) >= 600.0).collect();
                    let reach = long.iter().map(reach_of).fold(0.0, f32::max);
                    let from = centre.unwrap_or(party.at);
                    let dx = from.x - party.at.x;
                    let dz = from.z - party.at.z;
                    let len = dx.hypot(dz).max(1.0);
                    let point = |dist: f32| Vec3 { x: party.at.x + dx / len * dist, y: 0.0, z: party.at.z + dz / len * dist };
                    let standoff = self.snap_for(self.group_walker(group, own), point(reach * 0.85));
                    let screen = self.snap_for(self.group_walker(group, own), point(reach * 0.55));
                    group.hunt = None;
                    commands.extend(group.release_orders(&units));
                    for u in &units {
                        let to = if reach_of(u) >= 600.0 { standoff } else { screen };
                        commands.push(Command::Fight { unit: u.id, to, queue: false });
                    }
                    group.set_task(GroupTask::Move { to: standoff, place: format!("standoff from {}", party.name), fight: true, since: frame }, frame);
                    group.shelling = true;
                    group.last_order = frame;
                    did = Some(format!("shell {} from {:.0} short of it with {} long-reach soldiers, {} between", party.name, reach * 0.85, long.len(), units.len() - long.len()));
                }
            }
            Response::Join(other) => {
                if let Some(target) = pianist.groups.iter().position(|g| g.name == *other && g.domain == pianist.groups[index].domain) {
                    pianist.groups[index].hunt = None;
                    let members = std::mem::take(&mut pianist.groups[index].members);
                    let to = super::groups::centre_of(&pianist.groups[target].units(own)).or(centre).unwrap_or(home);
                    commands.extend(members.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                    pianist.groups[target].members.extend(members);
                    pianist.groups.remove(index);
                    did = Some(format!("join group_{other}"));
                }
            }
            _ => {}
        }
        self.pianist = Some(pianist);
        did
    }

    /// The unit of a party a group attacks by name: its commander when one is there, else its dearest member seen.
    pub(super) fn party_target(&self, party: &Party, enemies: &[bot_protocol::EnemyUnit]) -> Option<(UnitId, &'static str)> {
        let seen: Vec<&bot_protocol::EnemyUnit> = enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
        if let Some(c) = seen.iter().find(|e| e.def.is_some_and(|d| self.world.is_commander_def(d))) {
            return Some((c.id, "commander"));
        }
        seen.iter().filter(|e| e.def.is_some()).max_by(|a, b| {
            let metal = |e: &bot_protocol::EnemyUnit| e.def.and_then(|d| self.world.def(d)).map_or(0.0, |d| d.metal_cost);
            metal(a).total_cmp(&metal(b))
        }).map(|e| (e.id, "dearest unit"))
    }
}
