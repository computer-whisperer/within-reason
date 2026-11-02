//! The executor of the pass (`docs/design/2026-10-01-hands-rebuild.md`, law 7): a picked world's moves become
//! orders through the actuators (build sites, walking, fight orders, the engine's hunt), a task is remembered, and
//! the decision goes into the record with its source (`plan` for a pick, `list` for a step of the player's list);
//! nothing of the pick is kept for Jev's words (the last-pick register went 2026-10-24: the policy of t-1 does not
//! enter the words). Code carries a picked move to its end and says the
//! end; it never starts, changes or stops one by itself.

use bot_protocol::{Command, OwnUnit, Tick, UnitId, Vec3};
use serde_json::json;

use super::super::Brain;
use super::super::economy::Plan;
use super::super::roster::Kit;
use super::compose::World;
use super::groups::Ward;
use super::menu::{His, Kind, Menu, Order, Site};
use super::picture::{Party, Picture, clock};
use super::{Group, GroupTask, Task};

/// A defence ordered at a place stands this far toward the enemy from it, covering the approach.
const DEFENCE_FORWARD: f32 = 120.0;
/// A build's site within this of a place is the build at that place (a defence stands `DEFENCE_FORWARD` off it,
/// the engine's site search a building's width more).
pub(super) const SAME_SITE: f32 = 250.0;
/// Artillery shells from this share of its reach, and the screen stands at this share, both on the line from the
/// target to the guns (H-HANDS-SHELL, `shell_points`; the user's word 2026-10-31).
const STANDOFF: f32 = 0.85;
const SCREEN: f32 = 0.55;

/// Where artillery shells a target from, and where its screen stands (H-HANDS-SHELL). Guns that already have the
/// target inside their reach and stand outside its reach (`target_reach`, the longest of the target's) stay where
/// they are: the standoff is their centroid. Otherwise the standoff is `STANDOFF` of their reach from the target on
/// the line from the target toward the guns' centroid (`fallback`, the group's place, when it has no guns). The screen stands
/// `SCREEN` of the reach from the target on the line to the standoff. Until 2026-10-31 the line ran to the whole
/// group's centroid, joiners walking out from home included, and the standoff was walked to whatever the guns had in
/// reach: barb-ladder3-20-1 30:32, two Starlights (950) standing 863 from a Fatboy (700) were walked 520 east to a
/// point beside the nano pad, 698 round the crater, out of a shot they already had, and the words of that walk
/// ("nothing of it has the party in reach, about 20 s of walking") flipped the pick to attack the next second.
pub(super) fn shell_points(guns: &[Vec3], fallback: Vec3, target: Vec3, reach: f32, target_reach: f32) -> (Vec3, Vec3) {
    let n = guns.len() as f32;
    let from = if guns.is_empty() { fallback } else { Vec3 { x: guns.iter().map(|g| g.x).sum::<f32>() / n, y: fallback.y, z: guns.iter().map(|g| g.z).sum::<f32>() / n } };
    let line = |toward: Vec3, dist: f32| {
        let (dx, dz) = (toward.x - target.x, toward.z - target.z);
        let len = dx.hypot(dz).max(1.0);
        Vec3 { x: target.x + dx / len * dist, y: toward.y, z: target.z + dz / len * dist }
    };
    let in_place = !guns.is_empty()
        && guns.iter().all(|g| {
            let d = g.dist2d(target);
            d <= reach && d > target_reach
        });
    let standoff = if in_place { from } else { line(from, reach * STANDOFF) };
    let screen = line(standoff, reach * SCREEN);
    (standoff, screen)
}

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


impl Brain {
    /// Puts a picked world in force: every actor's move other than `stay` is executed. Returns what changed, for
    /// the log and the `done` lines.
    pub(super) fn apply_world(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, menus: &[Menu], world: &World, commands: &mut Vec<Command>) -> Vec<String> {
        let frame = tick.frame;
        let mut done: Vec<(String, &'static str, String, String)> = Vec::new();
        for (menu, i) in menus.iter().zip(world) {
            let m = &menu.moves[*i];
            if *i == 0 {
                continue;
            }
            let did = match &menu.kind {
                Kind::Builder(id) => self.execute_builder(tick, kit, picture, *id, &m.order, menu.queue_ahead, None, commands),
                Kind::Factory(id) => self.execute_factory(tick, *id, &m.order, commands),
                Kind::Group(name) => self.execute_group(tick, picture, name, &m.order, commands),
            };
            if let Some(did) = did {
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
            // The frame this builder is finishing already is the same build going on.
            Order::Finish(target) if !queue && matches!(self.pianist.as_ref().and_then(|p| p.tasks.get(&id)), Some(Task::Build { frame: Some(f), .. }) if f == target) => return None,
            _ => {}
        }
        let mut task: Option<Task> = None;
        let mut did: Option<String> = None;
        let mut build = |plan: Plan, spot: Option<usize>| -> Option<String> {
            let (def, site) = self.build_site_for(&plan, unit, own, kit)?;
            let near = site.near;
            let facing = match plan {
                Plan::Slot(_, _, facing) => facing,
                _ => 0,
            };
            // The layer's reasons for the site, so the player reads why it stands there.
            let why = match plan {
                Plan::Slot(..) => self.site_words(def, unit, own, near).map(|w| format!(" ({w})")).unwrap_or_default(),
                _ => String::new(),
            };
            commands.push(Command::Build { unit: id, def, site: Some(site), queue });
            task = Some(Task::Build { def, near, spot, ordered: frame, started: false, frame: None, facing });
            Some(format!("build a {} at {}{why}", self.name(def), self.place_words(&picture.places, near)))
        };
        match order {
            Order::Build(_, Site::Spot(i)) => {
                did = build(Plan::Extractor(self.world.hello.metal_spots[*i]), Some(*i));
            }
            Order::Build(def, Site::Planned) => {
                let plan = self.place_planned(*def, unit, own);
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
            // The place is the step's band of distance (the design §1.4): the site is chosen by the levels within it.
            Order::Build(def, Site::Place(place_name)) => {
                let at = place(place_name).map_or(unit.pos, |p| p.at);
                // A defence stands toward the enemy from the place, so its reach covers the approach rather than
                // whichever side the ground allows (player-10-routes; the user, 2026-09-28: "our turrets are not
                // built in reasonable locations to defend the points of interest intended").
                let defence = self.world.def(*def).is_some_and(|d| d.speed == 0.0 && d.weapon_count > 0);
                let (at, band) = if defence {
                    let enemy = self.enemy_base(at);
                    let (dx, dz) = (enemy.x - at.x, enemy.z - at.z);
                    let len = dx.hypot(dz).max(1.0);
                    (Vec3 { x: at.x + dx / len * DEFENCE_FORWARD, y: at.y, z: at.z + dz / len * DEFENCE_FORWARD }, super::super::economy::BAND_DEFENCE)
                } else {
                    (at, super::super::economy::BAND_PLACE)
                };
                let plan = self.place_in(*def, unit, own, at, band);
                did = build(plan, None);
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
            Order::TakeApartHis(target) => {
                if let Some(def) = tick.snapshot.enemies.iter().find(|e| e.id == *target).and_then(|e| e.def) {
                    commands.push(Command::ReclaimUnit { unit: id, target: *target, queue });
                    task = Some(Task::TakeApartHis { target: *target, def, since: frame });
                    did = Some(format!("take apart his {}_{}", self.name(def), target.0));
                }
            }
            Order::Repair(target) => {
                commands.push(Command::Repair { unit: id, target: *target, queue });
                task = Some(Task::Repair { target: *target, since: frame });
                did = Some("repair".into());
            }
            // An unfinished frame of ours as this builder's own build: the engine's repair command builds it, and
            // the task is a started build on that frame (the finish-first law and the 60 % rule hold, and the
            // finished event frees the builder or begins what it queued).
            Order::Finish(target) => {
                if let Some(frame_unit) = own.iter().find(|u| u.id == *target && u.being_built) {
                    commands.push(Command::Repair { unit: id, target: *target, queue });
                    let spot = self.world.is_extractor_def(frame_unit.def).then(|| self.world.hello.metal_spots.iter().position(|s| s.dist2d(frame_unit.pos) < self.spot_occupied_radius())).flatten();
                    task = Some(Task::Build { def: frame_unit.def, near: frame_unit.pos, spot, ordered: frame, started: true, frame: Some(*target), facing: frame_unit.facing });
                    did = Some(format!("finish our {} at {} ({:.0}% built)", self.short_words(frame_unit.def), self.place_words(&picture.places, frame_unit.pos), frame_unit.build_progress * 100.0));
                }
            }
            Order::DGun(party_name) => {
                if let Some(party) = picture.parties.iter().find(|p| p.name == *party_name)
                    && let Some(target) = self.dgun_targets(party, unit, &tick.snapshot.enemies).into_iter().next()
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
                    // In the engine's enemy-only mode: a plain fight makes a builder stop to repair, help build and
                    // take wrecks apart beside its line (companies-1 8:15-8:32: the commander moved 110 in 17 s
                    // toward a raid 1,300 away; the enemy-only drill, docs/data/scenarios-2026-10-27/: the plain
                    // fight stood 8 s repairing a turret beside the line, the enemy-only one walked past it and took
                    // the Pawn apart). It takes apart any of his it passes within its build distance.
                    commands.push(Command::FightReclaimingEnemies { unit: id, to: party.at, queue });
                    task = Some(Task::Attack { party: party_name.clone(), to: party.at, since: frame });
                    did = Some(format!("attack {party_name}"));
                }
            }
            _ => {}
        }
        let pianist = self.pianist.as_mut().expect("pianist mode");
        if let Some(task) = task {
            if queue {
                // With the list step it came from, moved to `list_steps` when the task is promoted (`Queued`).
                pianist.queue_push(id, task, list_step.map(str::to_string));
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
                // The ranks stand about the lane's anchor when the group is the lane's, else the body's place.
                let anchor = self.lane_group(&pianist.groups[index].lane_name()).map(|s| s.anchor);
                let group = &mut pianist.groups[index];
                let at = anchor.or_else(|| group.body(own, None).map(|b| b.at)).or(centre).unwrap_or(home);
                group.set_task(GroupTask::Hold { picked: true, at }, frame);
                commands.extend(group.hold_orders(&units));
                did = Some("hold where it stands".into());
            }
            Order::Go(place_name) | Order::FightTo(place_name) | Order::Raid(place_name) => {
                let fight = !matches!(order, Order::Go(_));
                if let Some(p) = place(place_name) {
                    let to = self.snap_for(walker, p.at);
                    let group = &mut pianist.groups[index];
                    commands.extend(group.release_orders(&units));
                    group.set_task(GroupTask::Move { to, place: p.name.clone(), fight }, frame);
                    // A declared group's walk is the lane's (its Go or FightTo); the hands order no member.
                    if !group.declared() {
                        commands.extend(ids.iter().map(|id| if fight { Command::Fight { unit: *id, to, queue: false } } else { Command::Move { unit: *id, to, queue: false } }));
                    }
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
            Order::Stun(party_name) => {
                if let Some(party) = party(party_name) {
                    let group = &mut pianist.groups[index];
                    Brain::engage_group(group, party, own, None, frame, commands);
                    did = Some(format!("stun {} ({})", party.name, party.composition));
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
                    let long: Vec<&OwnUnit> = units.iter().copied().filter(|u| self.artillery(u)).collect();
                    let reach = long.iter().map(reach_of).fold(0.0, f32::max);
                    let guns: Vec<Vec3> = long.iter().map(|u| u.pos).collect();
                    // The target's own reach: its party's members, or whatever of his stands at the place.
                    let target_reach = (enemies.iter())
                        .filter(|e| match target {
                            His::Party(name) => party(name).is_some_and(|p| p.ids.contains(&e.id)),
                            His::Place(_) => e.pos.dist2d(at) <= SAME_SITE,
                        })
                        .filter_map(|e| e.def.and_then(|d| self.world.def(d)))
                        .map(|d| d.reach)
                        .fold(0.0, f32::max);
                    let (standoff, screen) = shell_points(&guns, centre.unwrap_or(at), at, reach, target_reach);
                    let standoff = self.snap_for(walker, standoff);
                    let screen = self.snap_for(walker, screen);
                    commands.extend(group.release_orders(&units));
                    group.set_task(GroupTask::Move { to: standoff, place: format!("standoff from {what}"), fight: true }, frame);
                    group.shelling = true;
                    group.screen = Some(screen);
                    // A declared group's shell is the lane's two formations (chunk 5); an air group's the hands' orders.
                    if !group.declared() {
                        for u in &units {
                            let to = if self.artillery(u) { standoff } else { screen };
                            commands.push(Command::Fight { unit: u.id, to, queue: false });
                        }
                    }
                    did = Some(format!("shell {what} from their reach with {} long-reach soldiers, {} between", long.len(), units.len() - long.len()));
                }
            }
            Order::Join(other) | Order::TakeIn(other) => {
                if let Some(target) = pianist.groups.iter().position(|g| g.name == *other && g.domain == pianist.groups[index].domain) {
                    // The two become one body, and the direction is the pick's: a `join` walks this group's soldiers
                    // to the other's body, which keeps its name, its order and its place; a `take in` walks the
                    // other's to this one's. (From 2026-10-03 to 2026-10-31 the smaller walked to the larger
                    // whichever picked it, after human-22, where a body of 33 joined its own detachment of 8 and
                    // took its name; in barb-ladder4-20-1 that rule left two Bulls named as the army's leader
                    // unable to be joined by anyone without losing their name, and the player built the army of
                    // follows instead; now both directions are moves with their own words, read against the
                    // packet.) The walkers take the body's order when they reach it (H-HANDS-JOINERS): until then
                    // they are on their way, not of its front or its odds.
                    let take_in = matches!(order, Order::TakeIn(_));
                    let (keep, gone) = if take_in { (index, target) } else { (target, index) };
                    let to = pianist.groups[keep].body(own, None).map(|b| b.at).or(centre).unwrap_or(home);
                    let absorbed = pianist.groups.remove(gone);
                    let keep = if gone < keep { keep - 1 } else { keep };
                    let body = &mut pianist.groups[keep];
                    // Into a declared body they are members with slots; into any other, joiners sent to its place.
                    if !body.declared() {
                        commands.extend(absorbed.members.iter().map(|id| Command::Move { unit: *id, to, queue: false }));
                        body.joining.extend(absorbed.members.iter().map(|id| (*id, to)));
                    }
                    body.members.extend(absorbed.members);
                    did = Some(if take_in { format!("take in group_{other}: its soldiers walk to this body, which keeps its name and its course") } else { format!("join group_{other}: one body under group_{other}'s name and course") });
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
                    group.set_task(GroupTask::Follow { ward: ward.clone(), name: ward_name.clone(), at: to, since: frame }, frame);
                    // A declared group's follow is the lane's FightTo at the ward's place (chunk 4).
                    if !group.declared() {
                        commands.extend(ids.iter().map(|id| Command::Fight { unit: *id, to, queue: false }));
                    }
                    group.last_order = frame;
                    did = Some(format!("follow {ward_name}"));
                }
            }
            Order::RoveOne | Order::RoveOneWithin(_) => {
                // The group's fastest soldier leaves as a group of its own that roves (H-MICRO-ROVE): a detachment
                // like the hunt's, the lane running it from the next tick; within a region the player marked when
                // the move names one.
                let within = match order {
                    Order::RoveOneWithin(region) => Some(region.clone()),
                    _ => None,
                };
                let speed = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.speed);
                if let Some(scout) = units.iter().copied().filter(|u| self.can_rove(u.def)).max_by(|a, b| speed(a).total_cmp(&speed(b)))
                    && units.len() >= 2
                {
                    let domain = pianist.groups[index].domain;
                    pianist.groups[index].members.retain(|id| *id != scout.id);
                    let name = pianist.new_group_name();
                    did = Some(format!("a {} leaves as group_{name} and roves{}", self.name(scout.def), within.as_ref().map_or(String::new(), |w| format!(" within {w}"))));
                    let parent_name = pianist.groups[index].name.clone();
                    let group = Group::new(name, domain, vec![scout.id], GroupTask::Rove { within }, frame).split_from(&parent_name);
                    pianist.groups.push(group);
                }
            }
            Order::Rove | Order::RoveWithin(_) => {
                // The whole group roves (H-MICRO-ROVE): a task like any other, the lane taking every soldier from
                // the next tick; any later pick is the way back (docs/design/2026-10-07-rove-approval.md, the
                // rework). The player's own `rove` setting, the one unit-control order that bypassed the pick, is
                // gone since 2026-10-07. Within a region the player marked when the move names one; the plain
                // `rove` frees a group of its region.
                let within = match order {
                    Order::RoveWithin(region) => Some(region.clone()),
                    _ => None,
                };
                let group = &mut pianist.groups[index];
                group.joining.clear();
                did = Some(match &within {
                    Some(region) => format!("roves within {region}: its soldiers look only at places inside it, kill what they find unguarded there and keep out of every reach"),
                    None => "roves: its soldiers look at what we know least, kill what they find unguarded and keep out of every reach".to_string(),
                });
                group.set_task(GroupTask::Rove { within }, frame);
                group.last_order = frame;
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
    /// H-HANDS-SHELL: guns that already have the target in reach and stand out of its reach shell from where they
    /// are; guns too far walk to the standoff on their own line to the target, not the group's; guns inside the
    /// target's reach step out to the standoff.
    #[test]
    fn the_standoff_is_where_the_guns_already_have_the_shot_else_on_their_own_line() {
        let at = |x: f32, z: f32| Vec3 { x, y: 0.0, z };
        let fatboy = at(755.0, 4535.0);
        // Two Starlights (950) 863 from a Fatboy (700), the group's centroid dragged toward home by joiners.
        let guns = [at(640.0, 5380.0), at(672.0, 5404.0)];
        let home_ward = at(1467.0, 5451.0);
        let (standoff, screen) = super::shell_points(&guns, home_ward, fatboy, 950.0, 700.0);
        assert!(standoff.dist2d(at(656.0, 5392.0)) < 1.0, "the guns stay: {standoff:?}");
        let on_line = |p: Vec3| ((p.x - fatboy.x) * (standoff.z - fatboy.z) - (p.z - fatboy.z) * (standoff.x - fatboy.x)).abs() / standoff.dist2d(fatboy);
        assert!((screen.dist2d(fatboy) - 0.55 * 950.0).abs() < 1.0 && on_line(screen) < 1.0, "the screen between the guns and the target: {screen:?}");
        // The same guns 1,500 off: the standoff at 0.85 of their reach on their own line, not toward the group's centroid.
        let far = [at(755.0, 6035.0), at(755.0, 6065.0)];
        let (standoff, _) = super::shell_points(&far, home_ward, fatboy, 950.0, 700.0);
        assert!((standoff.dist2d(fatboy) - 0.85 * 950.0).abs() < 1.0 && (standoff.x - 755.0).abs() < 1.0, "{standoff:?}");
        // Guns inside the Fatboy's reach step out to the standoff.
        let close = [at(755.0, 5035.0)];
        let (standoff, _) = super::shell_points(&close, home_ward, fatboy, 950.0, 700.0);
        assert!((standoff.dist2d(fatboy) - 0.85 * 950.0).abs() < 1.0, "{standoff:?}");
        // No guns at all: the group's line.
        let (standoff, _) = super::shell_points(&[], home_ward, fatboy, 950.0, 700.0);
        assert!((standoff.dist2d(fatboy) - 0.85 * 950.0).abs() < 1.0 && standoff.x > fatboy.x && standoff.z > fatboy.z, "{standoff:?}");
    }

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
