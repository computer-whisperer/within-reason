//! The brain's side of the control lane (`crates/micro`, `docs/design/2026-09-20-micro-lane.md`,
//! `docs/design/2026-09-25-formation-micro.md`). The lane runs on the session's fast thread (`session.rs`,
//! `docs/design/2026-10-03-lane-off-the-brain-thread.md`); what the brain tells it goes with each tick's answer (the
//! groups it declares, the view's fixed and changing parts, `LaneFixed` and `LaneScene`), and what the lane tells the
//! brain comes with the next tick (`LaneReport`: hunts ended, rove events, the groups' states, the rovers' reports).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bot_protocol::{Command, Tick, UnitDefId, UnitId, Vec3};
use combatsim::sim::Rules;
pub use micro::{GroupState, RoveReport, Stats};

use crate::session::{Declared, LaneFixed, LaneReport, LaneScene};

/// The combat simulator's tables, and its unit type for each of the game's (filled on first use).
pub(super) struct Sim {
    pub(super) rules: Arc<Rules>,
    pub(super) defs: HashMap<UnitDefId, usize>,
}

impl Default for Sim {
    fn default() -> Self {
        // Their commander presses its D-gun (K-barb-commander-dgun-beats-a-pawn-party); ours never does in a scene.
        let tuning = combatsim::sim::Tuning { dgun: true, ..Default::default() };
        Sim { rules: Arc::new(Rules::new(combatsim::units::Units::default(), tuning)), defs: HashMap::new() }
    }
}

use super::Brain;
use crate::world::Domain;

impl Brain {
    /// A declared group's state as the lane last reported it.
    pub(super) fn lane_group(&self, name: &str) -> Option<&GroupState> {
        self.lane_groups.get(name)
    }

    /// The pianist's groups for the lane: a hunting or roving group declared as a group of that kind (design D15), a
    /// ground group holding, walking, advancing, attacking, following or shelling as a formation of kind Hold, Go,
    /// FightTo or Attack (the migration design, chunks 2-5: `Group::declared`; its order carries the company cap and
    /// the muster points: the companies design §4); a declared group's members are under no reflex
    /// (`docs/design/2026-10-28-contact-cleanup.md` §5). An air group is nothing to the lane but a rove (its aircraft
    /// are no lane soldiers otherwise). The fast thread sets these on the lane and clears every group not among them.
    pub(super) fn declare_groups(&self) -> Vec<Declared> {
        let mut declared: Vec<Declared> = Vec::new();
        let Some(pianist) = &self.pianist else { return declared };
        for group in &pianist.groups {
            let name = group.lane_name();
            match &group.task {
                // A roving group is the lane's whole (H-MICRO-ROVE), within the region the player marked, if one.
                super::pianist::GroupTask::Rove { within } => {
                    let within = within.as_ref().and_then(|name| self.region_of(name));
                    let heal = self.heal_of(&name);
                    declared.push((name, group.members.clone(), micro::GroupOrder::new(micro::GroupKind::Rove { within }, Vec::new()).with_heal(heal)));
                }
                // A formation (chunks 2 and 3): the hold about its point, the walk or the advance to its place, the
                // attack on a party (its named unit, or the fight to its centre as the sightings move it); the lane
                // paths the anchor there and places every member (H-MICRO-GROUP).
                super::pianist::GroupTask::Hold { at, .. } | super::pianist::GroupTask::Move { to: at, .. } | super::pianist::GroupTask::Engage { at, .. } | super::pianist::GroupTask::Follow { at, .. } if group.declared() => {
                    let kind = match &group.task {
                        // A follower fights at the ward's place (chunk 4): a Go would walk past what attacks it.
                        super::pianist::GroupTask::Move { fight: true, .. } | super::pianist::GroupTask::Follow { .. } => micro::GroupKind::FightTo,
                        super::pianist::GroupTask::Move { .. } => micro::GroupKind::Go,
                        super::pianist::GroupTask::Engage { target: Some(t), .. } => micro::GroupKind::Attack(*t),
                        super::pianist::GroupTask::Engage { .. } => micro::GroupKind::FightTo,
                        _ => micro::GroupKind::Hold,
                    };
                    let members: Vec<UnitId> = group.members.clone();
                    // The companies (the companies design §4): the player's cap for the group, the muster points
                    // of the plants feeding it and their `join_in` as the count that walks together.
                    let cap = self.company_of(&group.name);
                    let (musters, release) = pianist.musters.get(&group.name).cloned().unwrap_or_default();
                    let company = |order: micro::GroupOrder| micro::GroupOrder { cap, musters: musters.clone(), release, ..order };
                    // A shelling group is two formations (chunk 5): its artillery at the standoff, the rest at the
                    // screen; the hold its walk arrives in keeps both where their anchors stand.
                    let screen = group.screen.filter(|_| group.shelling);
                    let (artillery, rest): (Vec<UnitId>, Vec<UnitId>) = members.iter().partition(|id| self.known_units.get(id).is_some_and(|(def, _)| self.artillery_def(*def)));
                    if let Some(screen_at) = screen
                        && !artillery.is_empty()
                        && !rest.is_empty()
                    {
                        let screen_name = group.screen_lane_name();
                        let screen_point = match kind {
                            micro::GroupKind::Hold => self.lane_group(&screen_name).map_or(screen_at, |s| s.anchor),
                            _ => screen_at,
                        };
                        declared.push((name, artillery, company(micro::GroupOrder::new(kind.clone(), vec![*at]))));
                        declared.push((screen_name, rest, company(micro::GroupOrder::new(kind, vec![screen_point]))));
                    } else {
                        // An attack on a party carries the party (§5.2): each company's route ends at the nearest
                        // member of it in sight, the centre only while none is.
                        let party = match &group.task {
                            super::pianist::GroupTask::Engage { party, .. } => party.clone(),
                            _ => Vec::new(),
                        };
                        // A follower of a group carries its ward (§5.3): the lane keeps its end at the ward's
                        // anchor and off its footprint; the hands' point is where the ward stood at the last step.
                        let ward = match &group.task {
                            super::pianist::GroupTask::Follow { ward: super::pianist::groups::Ward::Group(other), .. } => Some(format!("group_{other}")),
                            _ => None,
                        };
                        declared.push((name, members, company(micro::GroupOrder { party, ward, ..micro::GroupOrder::new(kind, vec![*at]) })));
                    }
                }
                // A hunt is the engine's (H-MICRO-HUNT): attack by id every tick, raw, until the quarry is dead or
                // lost or the leash ends (docs/design/2026-09-26-threat-response.md §1). A quarry that cloaks is
                // hidden where it stands when it drops out of sight, not gone: the hunt runs on to its last place
                // for longer before it ends (H-HUNT-HIDDEN).
                super::pianist::GroupTask::Hunt(h) => {
                    let lost_frames = if self.quarry_cloaks(h.quarry) { super::pianist::groups::HUNT_HIDDEN_FRAMES } else { micro::HUNT_LOST_FRAMES };
                    let order = micro::HuntOrder { quarry: h.quarry, leash: super::pianist::groups::HUNT_LEASH, lost_frames, since: h.since };
                    declared.push((name, group.members.clone(), micro::GroupOrder::new(micro::GroupKind::Hunt(order), Vec::new())));
                }
                _ => {}
            }
        }
        declared
    }

    /// The player's company cap for a pianist group (`lane` tool, `company`; the companies design §4): the group's
    /// own, else `all`, else `COMPANY_CAP`; 0 lifts it.
    pub(super) fn company_of(&self, group: &str) -> usize {
        let Some(shared) = &self.strategist else { return super::pianist::groups::COMPANY_CAP };
        let company = shared.company.lock().unwrap();
        company.get(&format!("group_{group}")).or_else(|| company.get("all")).copied().unwrap_or(super::pianist::groups::COMPANY_CAP)
    }

    /// The player's word on a roving group's healing (`lane` tool, `heal`; H-MICRO-HEAL): the group's own, else
    /// `all`, else on.
    pub(super) fn heal_of(&self, group: &str) -> bool {
        let Some(shared) = &self.strategist else { return true };
        let heal = shared.heal.lock().unwrap();
        heal.get(&format!("group_{group}")).or_else(|| heal.get("all")).copied().unwrap_or(true)
    }

    /// The lane's word of the previous tick: its firings for the per-minute line (the record counts them on the
    /// fast thread), the groups' states and the rovers' reports for the picture, the hunts' ends (`groups.rs`
    /// `hunt_event`) and the rovers' events.
    pub(super) fn take_lane_report(&mut self, report: LaneReport, tick: &Tick, commands: &mut Vec<Command>) {
        for rule in report.fired {
            *self.fired.entry(rule).or_default() += 1;
        }
        self.lane_groups = report.groups;
        self.lane_roving = report.roving;
        if !report.hunts.is_empty()
            && let Some(pianist) = self.pianist.as_mut()
        {
            let own = &tick.snapshot.own_units;
            let (soldiers, buildings, world) = (&self.enemy_soldiers, &self.enemy_buildings, &self.world);
            let cloaks = |quarry: UnitId| soldiers.get(&quarry).or_else(|| buildings.get(&quarry)).and_then(|(d, ..)| world.def(*d)).is_some_and(|d| d.cloak.is_some());
            for event in &report.hunts {
                let mut ended = Vec::new();
                for group in pianist.groups.iter_mut() {
                    if let Some(end) = Brain::hunt_event(group, event, own, tick.frame, cloaks(event.quarry), commands) {
                        ended.push(end);
                    }
                }
                for (text, failed) in ended {
                    pianist.done.push(format!("{} {text}", super::pianist::picture::clock(tick.frame)));
                    pianist.hunt_events.push(text);
                    if let Some((party, why)) = failed {
                        pianist.hunts_failed.retain(|(_, f, _)| tick.frame - f < 3 * 60 * super::FRAMES_PER_SECOND);
                        pianist.hunts_failed.push((party, tick.frame, why));
                    }
                }
            }
        }
        if !report.rove.is_empty() {
            self.rove_events(&report.rove, tick.frame);
        }
    }

    /// The lane's view of what never changes, once the faction is adopted.
    pub(super) fn lane_fixed(&mut self) -> Arc<LaneFixed> {
        if self.sim.defs.is_empty() {
            self.survey_sim_defs();
        }
        let kit = self.kit.expect("the fixed view follows the faction");
        let unit_defs = &self.world.hello.unit_defs;
        let defs = unit_defs.iter().map(|d| (d.id, d.clone())).collect();
        let stats = unit_defs.iter().filter_map(|d| self.stats_of(d.id).map(|s| (d.id, s))).collect();
        let soldier_defs = unit_defs
            .iter()
            .filter(|d| d.id != kit.commander && d.speed > 0.0 && d.weapon_count > 0 && d.build_speed == 0.0 && self.world.domain_of(d.id) != Domain::Air)
            .map(|d| d.id)
            .collect();
        let terrain = &self.world.hello.terrain;
        let (terrain, grid_spec) = if terrain.width > 0 && terrain.cell > 0.0 {
            (Some(terrain.clone()), (terrain.cell, terrain.width as usize, terrain.height as usize))
        } else {
            let map = &self.world.hello.map;
            (None, (16.0, (map.width / 16.0).ceil() as usize, (map.height / 16.0).ceil() as usize))
        };
        let sim = (!self.sim.defs.is_empty()).then(|| (self.sim.rules.clone(), self.sim.defs.clone()));
        Arc::new(LaneFixed { defs, stats, terrain, grid_spec, home: self.home, disabled: self.disabled.clone(), label: format!("[ai {}]", self.ai()), soldier_defs, sim })
    }

    /// The lane's view of what the brain's trackers hold, as of this think.
    pub(super) fn lane_scene(&self, frame: i32) -> Arc<LaneScene> {
        // A building last seen being built is remembered, but it shoots nothing.
        let remembered = self.enemy_buildings.iter().filter(|(id, _)| !self.enemy_unfinished.contains(id)).map(|(id, (def, pos, _))| (*id, *def, *pos)).collect();
        let buildings = self.enemy_buildings.iter().map(|(id, b)| (*id, b.1)).collect();
        let known_soldiers = self.enemy_soldiers.keys().copied().collect();
        // A radar contact is taken for the soldier of theirs we have seen most, as the pricing does.
        let mut counted: HashMap<UnitDefId, usize> = HashMap::new();
        self.enemy_soldiers.values().for_each(|(def, _, _)| *counted.entry(*def).or_default() += 1);
        let blip = counted.into_iter().max_by_key(|(def, n)| (*n, def.0)).map(|(def, _)| def).or(self.kit.as_ref().map(|k| k.line));
        let rovers: HashSet<UnitDefId> = self.known_units.values().map(|(def, _)| *def).filter(|def| self.world.def(*def).is_some_and(|d| d.speed > 0.0 && d.weapon_count > 0) && self.can_rove(*def)).collect();
        let rove_goals = rovers.into_iter().map(|def| (def, self.rove_goals_of(def))).collect();
        // The commander heals no rover while it is under fire or on an attack (H-HANDS-HEAL).
        let no_heal: HashSet<UnitId> = self
            .pianist
            .as_ref()
            .map(|p| {
                let mut set: HashSet<UnitId> = p.under_fire(frame).collect();
                set.extend(p.tasks.iter().filter(|(_, t)| matches!(t, super::pianist::Task::Attack { .. })).map(|(id, _)| *id));
                set
            })
            .unwrap_or_default();
        Arc::new(LaneScene { remembered, buildings, known_soldiers, blip, stunned: self.enemy_stunned.clone(), rove_goals, no_heal })
    }

    pub(super) fn survey_sim_defs(&mut self) {
        let rules = self.sim.rules.clone();
        // Every definition the table has, aircraft included; a type it lacks gets the glossary's numbers from
        // `sim_stats` rather than a stand-in by metal (docs/design/2026-09-22-domains.md, decision 7).
        for def in &self.world.hello.unit_defs {
            if let Some(index) = rules.units.index(&def.name) {
                self.sim.defs.insert(def.id, index);
            }
        }
    }

    /// Whether a soldier of this type can rove (H-MICRO-ROVE): any on the ground; an aircraft with a weapon that hits
    /// the ground (a gunship, a bomber), not a fighter, a stun drone or a scout, which would kill nothing.
    pub(super) fn can_rove(&self, def: UnitDefId) -> bool {
        self.world.domain_of(def) != Domain::Air || self.sim_stats(def).is_some_and(|(_, dps, _)| dps > 0.0)
    }

    /// Whether a type's weapons only stun (a Shuriken, a Stiletto): it kills nothing, and the stun is what it is for.
    pub(super) fn stuns_only(&self, def: UnitDefId) -> bool {
        self.world.def(def).is_some_and(|d| d.stun.is_some()) && self.sim_stats(def).is_none_or(|(_, dps, _)| dps <= 0.0)
    }

    /// Whether every member of a group stuns only.
    pub(super) fn stun_group(&self, group: &super::pianist::Group) -> bool {
        !group.members.is_empty() && group.members.iter().all(|id| self.known_units.get(id).is_some_and(|(def, _)| self.stuns_only(*def)))
    }

    /// A type's reach, damage a second and speed against ground: the simulator's table, else the glossary's numbers
    /// for a type the table lacks (never a stand-in by metal).
    pub(super) fn sim_stats(&self, def: UnitDefId) -> Option<(f32, f32, f32)> {
        if let Some(i) = self.sim.defs.get(&def) {
            let unit = &self.sim.rules.units.list[*i];
            return Some((unit.reach(), unit.dps(), unit.speed));
        }
        let entry = super::pianist::glossary::entry(self.name(def))?;
        Some((entry.range, entry.dps.unwrap_or(0.0), entry.speed))
    }

    /// The lane's numbers for a type: the simulator's where the table has it, the glossary's otherwise.
    fn stats_of(&self, def: UnitDefId) -> Option<Stats> {
        let (reach, dps, speed) = self.sim_stats(def)?;
        let (health, dgun, air_reach, air_dps, falloff, heights) = match self.sim.defs.get(&def) {
            Some(i) => {
                let unit = &self.sim.rules.units.list[*i];
                (unit.health, unit.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max), unit.reach_air(), unit.dps_air(), unit.falloff(), (unit.mid, unit.top, unit.radius))
            }
            None => match super::pianist::glossary::entry(self.name(def)) {
                // The glossary has one reach for all of a type's weapons: its anti-air's when it has some, and no
                // weapon kind, so no falloff; and no heights, so no shadow stand on or behind it.
                Some(e) => (e.health, 0.0, e.dps_air.filter(|d| *d > 0.0).map_or(0.0, |_| e.range), e.dps_air.unwrap_or(0.0), 0.0, (0.0, 0.0, 0.0)),
                None => (0.0, 0.0, 0.0, 0.0, 0.0, (0.0, 0.0, 0.0)),
            },
        };
        Some(Stats { reach, dps, speed, health, dgun, sight: self.sight_of(def), area: self.world.def(def).map_or(0.0, |d| d.blast_radius), air_reach, air_dps, falloff, aim: heights.0, top: heights.1, radius: heights.2 })
    }

    /// Every metal spot a unit of this type can reach from home, with when it was last within sight of ours
    /// (H-SCOUT-SPOTS), his when it lies in another ally team's start box or within 600 of his base; and his base
    /// when one has been found, last seen when the freshest spot round it was.
    fn rove_goals_of(&self, def: UnitDefId) -> Vec<micro::RoveGoal> {
        let walker = self.walker_of(def);
        let ours = self.world.hello.ally_team;
        let base = self.found_enemy_base();
        // His box counts only while the lobby's boxes are honoured (game 10: our own G1 seat stood in "his" box and
        // the rovers walked our own top row for it).
        let honoured = self.boxes_honoured;
        let theirs = |at: Vec3| (honoured && self.world.hello.start_boxes.iter().any(|b| b.ally_team != ours && b.contains(at))) || base.is_some_and(|b| b.dist2d(at) < BASE_RADIUS);
        let spots = &self.world.hello.metal_spots;
        let mut goals: Vec<micro::RoveGoal> = spots.iter().enumerate().filter(|(_, at)| self.reachable_for(walker, **at)).map(|(i, at)| micro::RoveGoal { name: format!("spot_{i}"), at: *at, seen: self.spot_seen(i), theirs: theirs(*at) }).collect();
        // On a map without spots (a metal map) the places are the metal fields' centres and the cells of the grid,
        // each seen when the scout's look at its cell says (the metal-maps design §1.2).
        let mut cell_looks = Vec::new();
        if spots.is_empty() {
            let grid = self.world.cells;
            cell_looks = self.cell_looks();
            let seen_at = |at: Vec3| {
                let (c, r) = grid.cell_of(at);
                cell_looks[grid.index(c, r)].1
            };
            goals.extend(self.world.hello.metal_fields.iter().filter(|f| self.reachable_for(walker, f.centre)).map(|f| micro::RoveGoal { name: f.name.clone(), at: f.centre, seen: seen_at(f.centre), theirs: theirs(f.centre) }));
            goals.extend(grid.cells().map(|(c, r)| grid.centre(c, r)).filter(|at| self.reachable_for(walker, *at)).map(|at| micro::RoveGoal { name: grid.name(at), at, seen: seen_at(at), theirs: theirs(at) }));
        }
        if let Some(at) = base.filter(|b| self.reachable_for(walker, *b)) {
            let seen = (0..spots.len()).filter(|i| spots[*i].dist2d(at) < BASE_RADIUS).filter_map(|i| self.spot_seen(i)).max().or_else(|| {
                let (c, r) = self.world.cells.cell_of(at);
                cell_looks.get(self.world.cells.index(c, r)).and_then(|l| l.1)
            });
            goals.push(micro::RoveGoal { name: "his base".to_string(), at, seen, theirs: true });
        }
        goals
    }

    /// The region a roving group is told to stay within (`GroupTask::Rove { within }`): the marked place's cells as
    /// ground, north-west and south-east corners.
    fn region_of(&self, place: &str) -> Option<(Vec3, Vec3)> {
        let pianist = self.pianist.as_ref()?;
        let cells = pianist.places.iter().find(|p| p.name == place)?.cells?;
        Some(self.world.cells.rect(cells))
    }
}

/// A spot this close to his found base is his; the base counts as seen when one of them was.
const BASE_RADIUS: f32 = 600.0;
