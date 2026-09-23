//! The hands: an answer becomes orders through the existing actuators (build sites, walking, the march, fight
//! orders), a task is remembered, and the decision goes into the record.

use std::collections::BTreeMap;

use bot_protocol::{Command, OwnUnit, Tick, UnitId};
use jev::Answer;
use serde_json::json;

use super::super::economy::Plan;
use super::super::roster::Kit;
use super::super::Brain;
use super::menu::{Actor, Menu, Pick, nearest_of};
use super::picture::Picture;
use super::{Group, GroupTask, SWITCH_MARGIN, Task};

impl Brain {
    /// Plays every menu's answer.
    pub(super) fn play(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, menus: Vec<Menu>, answers: &BTreeMap<String, Answer>, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        for menu in menus {
            let Actor::Global = menu.actor else {
                self.play_one(tick, kit, picture, menu, answers, commands);
                continue;
            };
            let outputs: BTreeMap<&str, f64> = menu.questions.iter().filter_map(|(id, _)| answers.get(id).map(|a| (id.as_str(), a.probability_of("yes")))).collect();
            self.journal.note_from("jev", frame, "global", serde_json::Value::Null, json!(outputs));
        }
    }

    fn play_one(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, menu: Menu, answers: &BTreeMap<String, Answer>, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let name = menu.name.clone();
        let question = if matches!(menu.actor, Actor::Lab(_)) { format!("{name}.next") } else { format!("{name}.do") };
        // A step from the player's list plays itself (H-HANDS-SCRIPT); anything else is the answer's choice.
        let scripted = menu.scripted.clone();
        let (choice, probabilities, confidence): (String, BTreeMap<String, f64>, f64) = match (&scripted, answers.get(&question)) {
            (Some((key, _)), _) => (key.clone(), BTreeMap::new(), 1.0),
            (None, Some(Answer::Choice { choice, probabilities, confidence })) => (choice.clone(), probabilities.clone(), *confidence),
            _ => return,
        };
        let mut chosen = choice.clone();
        let p = |option: &str| probabilities.get(option).copied().unwrap_or(0.0);
        // H-HANDS-SWITCH: a busy actor changes course only for a clear winner.
        let mut kept = false;
        if scripted.is_none() && menu.busy && chosen != "continue" && p(&chosen) - p("continue") < SWITCH_MARGIN {
            chosen = "continue".into();
            kept = true;
        }
        let answered = |q: &str| answers.get(&format!("{name}.{q}")).and_then(|a| if let Answer::Choice { choice, .. } = a { Some(choice.clone()) } else { None });
        let listed = scripted.as_ref().and_then(|(_, place)| place.clone());
        let where_ = listed.clone().or_else(|| answered("where"));
        let where_extractor = listed.or_else(|| answered("where_extractor"));
        let whom = answers.get(&format!("{name}.whom")).and_then(|a| if let Answer::Choice { choice, .. } = a { Some(choice.clone()) } else { None });
        let how_many = answers.get(&format!("{name}.how_many")).and_then(|a| if let Answer::Choice { choice, .. } = a { Some(choice.clone()) } else { None });
        let place = |named: &Option<String>| named.as_ref().and_then(|n| picture.places.iter().find(|p| p.name == *n)).cloned();
        let Some(pick) = menu.options.get(&chosen).cloned() else { return };
        let mut did: Option<String> = None;
        match menu.actor {
            Actor::Builder(id) => {
                let Some(unit) = own.iter().find(|u| u.id == id) else { return };
                // H-HANDS-QUEUE: asked ahead, the answer is ordered behind the build in progress and kept as the
                // builder's next task.
                let queue = menu.queue_ahead;
                let mut task: Option<Task> = None;
                let mut build = |plan: Plan, spot: Option<usize>| -> Option<String> {
                    let (def, site) = self.build_site_for(&plan, unit, own, kit)?;
                    let near = site.near;
                    commands.push(Command::Build { unit: id, def, site: Some(site), queue });
                    task = Some(Task::Build { def, near, spot, ordered: frame, started: false });
                    Some(format!("build a {} at {}", self.name(def), self.place_words(&picture.places, near)))
                };
                // H-HANDS-STARTED: the kind of building already started, answered again, is the same build going
                // on, not a second frame (pianist-player-6: "generator" three asks running, three frames, two decayed).
                // Asked ahead, the same kind again is the next one, queued.
                let started_def = match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
                    Some(Task::Build { def, started: true, .. }) if !queue => Some(*def),
                    _ => None,
                };
                // Helping the lab it already helps is the same task going on, not a new guard order: re-issued, the
                // order reset the task's clock every ask and the picture said "since 2 s ago" of a minute's helping.
                let helping = match self.pianist.as_ref().and_then(|p| p.tasks.get(&id)) {
                    Some(Task::Assist { lab, .. }) if !queue => Some(*lab),
                    _ => None,
                };
                let pick = match pick {
                    Pick::Building(def) | Pick::BuildingAt(def) if started_def == Some(def) => Pick::Continue,
                    Pick::Extractor if started_def.is_some_and(|d| self.world.is_extractor_def(d)) => Pick::Continue,
                    Pick::AssistLab(lab) if helping == Some(lab) => Pick::Continue,
                    other => other,
                };
                match pick {
                    Pick::Continue | Pick::Wait => {}
                    Pick::Extractor => {
                        // The spot answered in `where` when it is one of the free ones offered, else the nearest.
                        let asked = place(&where_extractor).and_then(|p| p.spot).filter(|i| menu.spots.contains(i));
                        if let Some(i) = asked.or_else(|| menu.spots.first().copied()) {
                            did = build(Plan::Extractor(self.world.hello.metal_spots[i]), Some(i));
                        }
                    }
                    Pick::Building(def) => {
                        let plan = self.place_planned(def, unit, own, kit);
                        did = build(plan, None);
                    }
                    Pick::BuildingAt(def) => {
                        let at = place(&where_).map_or(unit.pos, |p| p.at);
                        did = build(Plan::Near(def, self.snap_for(self.walker_of(unit.def), at)), None);
                    }
                    Pick::AssistLab(lab) => {
                        // Queued, the guard order is given when the build finishes (`Pianist::promote`).
                        if !queue {
                            commands.push(Command::Guard { unit: id, target: lab });
                        }
                        task = Some(Task::Assist { lab, since: frame });
                        did = Some("help the factory".into());
                    }
                    Pick::Reclaim(at) => {
                        let wrecks = self.wrecks_to_take(at, unit);
                        if !wrecks.is_empty() {
                            commands.extend(wrecks.into_iter().enumerate().map(|(n, feature)| Command::ReclaimFeature { unit: id, feature, queue: queue || n > 0 }));
                            task = Some(Task::Reclaim { at, since: frame });
                            did = Some(format!("reclaim wrecks at {}", self.place_words(&picture.places, at)));
                        }
                    }
                    Pick::ReclaimUnit(target) => {
                        commands.push(Command::ReclaimUnit { unit: id, target, queue });
                        task = Some(Task::ReclaimUnit { target, since: frame });
                        did = Some(format!("take apart our {}", own.iter().find(|u| u.id == target).map_or("unit".to_string(), |u| self.handle(u))));
                    }
                    Pick::Repair(target) => {
                        commands.push(Command::Repair { unit: id, target, queue });
                        task = Some(Task::Repair { target, since: frame });
                        did = Some("repair".into());
                    }
                    Pick::WalkTo => {
                        if let Some(p) = place(&where_) {
                            let to = self.snap_for(self.walker_of(unit.def), p.at);
                            commands.push(Command::Move { unit: id, to, queue });
                            task = Some(Task::Walk { to, place: p.name.clone(), since: frame });
                            did = Some(format!("walk to {}", p.name));
                        }
                    }
                    Pick::Attack(at, ref party_name) => {
                        commands.push(Command::Fight { unit: id, to: at, queue });
                        task = Some(Task::Walk { to: at, place: party_name.clone(), since: frame });
                        did = Some(format!("attack {party_name}"));
                    }
                    Pick::RetreatHome => {
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
                if let Some(task) = task {
                    let pianist = self.pianist.as_mut().expect("pianist mode");
                    if queue {
                        pianist.queued.insert(id, task);
                        did = did.map(|d| format!("next, queued: {d}"));
                    } else {
                        pianist.tasks.insert(id, task);
                    }
                } else if matches!(pick, Pick::Wait) {
                    self.pianist.as_mut().expect("pianist mode").tasks.remove(&id);
                }
            }
            Actor::Lab(id) => {
                if let (Some(key), None) = (&menu.replay_key, &menu.replay)
                    && let Some(pianist) = self.pianist.as_mut()
                {
                    pianist.replays.insert(name.clone(), super::Replay { key: key.clone(), probabilities: probabilities.clone(), confidence, since: frame });
                }
                if let Pick::Unit(def) = pick {
                    commands.push(Command::Build { unit: id, def, site: None, queue: false });
                    self.pianist.as_mut().expect("pianist mode").lab_queue.entry(id).or_default().push((def, frame));
                    did = Some(format!("build a {}", self.name(def)));
                }
            }
            Actor::Group(ref group_name) => {
                did = self.play_group(tick, picture, group_name, pick, &where_, &whom, &how_many, commands);
            }
            Actor::Global => {}
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if kept {
            pianist.stats.kept += 1;
        } else if menu.busy && chosen != "continue" {
            pianist.stats.switches += 1;
        }
        if let Some(did) = &did {
            let from = if scripted.is_some() { " (from its list)" } else if menu.policy { " (policy)" } else { "" };
            pianist.done.push(format!("{} {name}: {did}{from}", super::picture::clock(frame)));
        }
        let kind: &'static str = match menu.actor {
            Actor::Builder(_) => "builder",
            Actor::Lab(_) => "lab",
            Actor::Group(_) => "group",
            Actor::Global => "global",
        };
        let inputs = json!({ "actor": name, "options": menu.options.keys().collect::<Vec<_>>(), "busy": menu.busy });
        let outputs = json!({ "choice": choice, "played": chosen, "probability": p(&choice), "confidence": confidence, "scripted": scripted.is_some(), "where": where_, "where_extractor": where_extractor, "whom": whom, "how_many": how_many, "did": did });
        let source = if menu.policy { "policy" } else if scripted.is_some() { "list" } else { "jev" };
        self.pianist.as_mut().expect("pianist mode").played.push(json!({ "actor": inputs["actor"], "kind": kind, "busy": menu.busy, "options": inputs["options"], "choice": choice, "played": chosen, "kept": kept, "probability": p(&choice), "confidence": confidence, "did": did, "source": source }));
        self.journal.note_from(source, frame, kind, inputs, outputs);
    }

    #[allow(clippy::too_many_arguments)]
    fn play_group(&mut self, tick: &Tick, picture: &Picture, group_name: &str, pick: Pick, where_: &Option<String>, whom: &Option<String>, how_many: &Option<String>, commands: &mut Vec<Command>) -> Option<String> {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let home = self.home;
        let place = |named: &Option<String>| named.as_ref().and_then(|n| picture.places.iter().find(|p| p.name == *n)).cloned();
        let Some(mut pianist) = self.pianist.take() else { return None };
        let Some(index) = pianist.groups.iter().position(|g| g.name == group_name) else {
            self.pianist = Some(pianist);
            return None;
        };
        let units: Vec<&OwnUnit> = pianist.groups[index].units(own);
        let ids: Vec<UnitId> = units.iter().map(|u| u.id).collect();
        let centre = super::groups::centre_of(&units);
        let mut did: Option<String> = None;
        match pick {
            Pick::Continue => {}
            Pick::Hold => {
                // Orders only on entering the hold (H-HANDS-AIR-TARGET, decision 12): a holding group picked `hold`
                // again gets none, where an air group used to get a move and a move state per member every ask.
                let group = &mut pianist.groups[index];
                if group.task.busy() {
                    commands.extend(group.hold_orders(&units));
                }
                group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                did = Some("hold".into());
            }
            Pick::MoveTo { fight } => {
                if let Some(p) = place(where_) {
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    commands.extend(ids.iter().map(|id| if fight { Command::Fight { unit: *id, to, queue: false } } else { Command::Move { unit: *id, to, queue: false } }));
                    group.set_task(GroupTask::Move { to, place: p.name.clone(), fight, since: frame }, frame);
                    group.last_order = frame;
                    did = Some(format!("{} to {}", if fight { "advance" } else { "walk" }, p.name));
                }
            }
            Pick::Engage | Pick::AttackUnit => {
                if let Some(party) = whom.as_ref().and_then(|n| picture.parties.iter().find(|p| p.name == *n)) {
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    // A named target: `attack_unit` always, and every air group's engagement (a fight order makes a
                    // bomber bomb the nearest thing on its line, not the party).
                    let target = if matches!(pick, Pick::AttackUnit) || group.domain == crate::world::Domain::Air { self.party_target(party, &tick.snapshot.enemies) } else { None };
                    match target {
                        Some((t, what)) => {
                            commands.extend(ids.iter().map(|id| Command::Attack { unit: *id, target: t, queue: false }));
                            did = Some(format!("attack the {what} of {} ({})", party.name, party.composition));
                        }
                        None => {
                            commands.extend(ids.iter().map(|id| Command::Fight { unit: *id, to: party.at, queue: false }));
                            did = Some(format!("attack {} ({})", party.name, party.composition));
                        }
                    }
                    group.set_task(GroupTask::Engage { party: party.ids.clone(), at: party.at, since: frame, last_seen: frame, target: target.map(|(t, _)| t), searched: false }, frame);
                    group.last_order = frame;
                }
            }
            Pick::Retreat => {
                let group = &mut pianist.groups[index];
                commands.extend(group.release_orders(&units));
                commands.extend(ids.iter().map(|id| Command::Move { unit: *id, to: home, queue: false }));
                group.set_task(GroupTask::Move { to: home, place: "home".into(), fight: false, since: frame }, frame);
                group.last_order = frame;
                did = Some("fall back home".into());
            }
            Pick::Split => {
                if let Some(p) = place(where_) {
                    let n = match how_many.as_deref() {
                        Some("1") => 1,
                        Some("2") => 2,
                        Some("4") => 4,
                        Some("8") => 8,
                        _ => units.len() / 2,
                    }
                    .clamp(1, units.len().saturating_sub(1).max(1));
                    let domain = pianist.groups[index].domain;
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    let detached: Vec<UnitId> = nearest_of(&units, to, n).iter().map(|u| u.id).collect();
                    pianist.groups[index].members.retain(|id| !detached.contains(id));
                    commands.extend(detached.iter().flat_map(|id| [Command::MoveState { unit: *id, state: 1 }].into_iter().filter(|_| domain == crate::world::Domain::Air).chain([Command::Fight { unit: *id, to, queue: false }])));
                    let name = pianist.new_group_name();
                    pianist.last_asked.insert(format!("group_{name}"), frame);
                    did = Some(format!("send {} soldiers as group_{name} to {}", detached.len(), p.name));
                    let parent_name = pianist.groups[index].name.clone();
                    pianist.groups.push(Group::new(name, domain, detached, GroupTask::Move { to, place: p.name.clone(), fight: true, since: frame }, frame).split_from(&parent_name));
                }
            }
            Pick::Detach => {
                // The party in `whom` unless a group already engages it; then the nearest party nobody engages.
                let engaged: Vec<UnitId> = pianist.groups.iter().filter_map(|g| if let GroupTask::Engage { party, .. } = &g.task { Some(party.clone()) } else { None }).flatten().collect();
                let free = |p: &&super::picture::Party| p.ids.len() <= super::menu::DETACH_PARTY_MAX && !p.ids.iter().any(|id| engaged.contains(id));
                let party = whom.as_ref().and_then(|n| picture.parties.iter().find(|p| p.name == *n)).filter(free).or_else(|| {
                    picture.parties.iter().filter(free).min_by(|a, b| centre.map_or(0.0, |c| a.at.dist2d(c)).total_cmp(&centre.map_or(0.0, |c| b.at.dist2d(c))))
                });
                if let Some(party) = party {
                    let n = match how_many.as_deref() {
                        Some("1") => 1,
                        Some("2") => 2,
                        Some("4") => 4,
                        Some("8") => 8,
                        _ => units.len() / 2,
                    }
                    .clamp(1, units.len().saturating_sub(1).max(1));
                    let domain = pianist.groups[index].domain;
                    let detached: Vec<UnitId> = nearest_of(&units, party.at, n).iter().map(|u| u.id).collect();
                    pianist.groups[index].members.retain(|id| !detached.contains(id));
                    let target = if domain == crate::world::Domain::Air { self.party_target(party, &tick.snapshot.enemies) } else { None };
                    commands.extend(detached.iter().flat_map(|id| [Command::MoveState { unit: *id, state: 1 }].into_iter().filter(|_| domain == crate::world::Domain::Air).chain([match target { Some((t, _)) => Command::Attack { unit: *id, target: t, queue: false }, None => Command::Fight { unit: *id, to: party.at, queue: false } }])));
                    let name = pianist.new_group_name();
                    pianist.last_asked.insert(format!("group_{name}"), frame);
                    did = Some(format!("send {} soldiers as group_{name} against {} ({})", detached.len(), party.name, party.composition));
                    let parent_name = pianist.groups[index].name.clone();
                    pianist.groups.push(Group::new(name, domain, detached, GroupTask::Engage { party: party.ids.clone(), at: party.at, since: frame, last_seen: frame, target: target.map(|(t, _)| t), searched: false }, frame).split_from(&parent_name));
                }
            }
            Pick::Scout => {
                // The place is the bot's (H-HANDS-DIET, decision 1). A scout to where the group stands looks at nothing.
                if let Some(p) = centre.and_then(|c| self.scout_target(c, picture, frame)).filter(|p| centre.is_none_or(|c| c.dist2d(p.at) > 600.0)) {
                    let domain = pianist.groups[index].domain;
                    let to = self.snap_for(self.group_walker(&pianist.groups[index], own), p.at);
                    // The fastest soldier of the group goes: a raider by the glossary's class when there is one.
                    let raider = |u: &&OwnUnit| super::glossary::entry(self.name(u.def)).is_some_and(|e| e.class.contains("raider") || e.class.contains("scout"));
                    let scout = units.iter().copied().filter(raider).min_by(|a, b| a.pos.dist2d(to).total_cmp(&b.pos.dist2d(to))).or_else(|| nearest_of(&units, to, 1).first().copied());
                    if let Some(scout) = scout {
                        pianist.groups[index].members.retain(|id| *id != scout.id);
                        if domain == crate::world::Domain::Air {
                            commands.push(Command::MoveState { unit: scout.id, state: 1 });
                        }
                        commands.push(Command::Move { unit: scout.id, to, queue: false });
                        let name = pianist.new_group_name();
                        pianist.last_asked.insert(format!("group_{name}"), frame);
                        did = Some(format!("send a {} as group_{name} to look at {}", self.name(scout.def), p.name));
                        let parent_name = pianist.groups[index].name.clone();
                        pianist.groups.push(Group::new(name, domain, vec![scout.id], GroupTask::Move { to, place: p.name.clone(), fight: false, since: frame }, frame).split_from(&parent_name));
                    }
                }
            }
            Pick::Join(other) => {
                if let Some(target) = pianist.groups.iter().position(|g| g.name == other && g.domain == pianist.groups[index].domain) {
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
}

impl Brain {
    /// The unit of a party a group attacks by name: its commander when one is there, else its dearest member seen.
    pub(super) fn party_target(&self, party: &super::picture::Party, enemies: &[bot_protocol::EnemyUnit]) -> Option<(UnitId, &'static str)> {
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
