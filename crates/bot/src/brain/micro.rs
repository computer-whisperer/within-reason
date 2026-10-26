//! The brain's side of the control lane (`crates/micro`, `docs/design/2026-09-20-micro-lane.md`,
//! `docs/design/2026-09-25-formation-micro.md`): what the brain tells the lane (standing orders, commitments, the
//! footwork rules of the pianist's groups) and what the lane reads of the brain (`BrainView`).

use std::collections::HashMap;
use std::sync::Arc;

use bot_protocol::{Command, Terrain, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use combatsim::sim::Rules;
pub use micro::{Commitment, Footwork, Lane, Stats, View};

/// The enemy commander's D-gun is not in the simulator (nobody presses the button there) and it kills a Pawn a shot:
/// a party smaller than this much metal does not fight within reach of the commander (rush-smoke2: four Pawns dead to
/// it in six seconds); a second player's twelve Pawns killed BARb's.
pub(super) const COMMANDER_PARTY_METAL: f32 = 450.0;

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
    /// The brain's orders this tick become the standing orders of the soldiers they went to.
    pub(super) fn note_standing_orders(&mut self, commands: &[Command], frame: i32) {
        let Some(kit) = self.kit else { return };
        let (known, world) = (&self.known_units, &self.world);
        let soldier = |unit: UnitId| {
            known.get(&unit).is_some_and(|(def, _)| *def != kit.commander && world.def(*def).is_some_and(|d| d.speed > 0.0 && d.weapon_count > 0 && d.build_speed == 0.0) && world.domain_of(*def) != Domain::Air)
        };
        self.lane.note_standing_orders(commands, frame, soldier);
    }

    /// What each soldier is priced against in the lane: the pianist's groups' commitments and footwork, per unit; a
    /// hunting or roving group is declared to the lane as a group of that kind (design D15), and a ground group
    /// holding or walking as a formation of kind Hold, Go or FightTo (the migration design, chunk 2: `Group::declared`;
    /// its order carries the company cap and the muster points: the companies design §4). Every other group's soldiers (air,
    /// engaging, following, shelling) are loose soldiers to the lane until their chunks.
    pub(super) fn note_commitments(&mut self) {
        let mut commitment: HashMap<UnitId, Commitment> = HashMap::new();
        let mut declared: Vec<(String, Vec<UnitId>, micro::GroupOrder, Footwork, Commitment)> = Vec::new();
        // The pianist's groups (H-HANDS-GROUPS): one advancing (`fight_to`) is committed to everything, turrets and the
        // commander included, as a wave is: priced against no turret, a ball of eighty stood at the edge of the enemy
        // base's laser towers inside a Guardian's reach for five minutes while Jev said "continue" every ten seconds
        // (pianist-player-2). The hold it arrives in keeps that commitment (pianist-player-4: the ball that fought
        // into the base stood there stepping out of the towers' reach, 35 of 35 held back). A hold Jev chose stands
        // against mobile units and steps out of turret reach; an engaging group fights the commander when it is the
        // raid's size for it (pianist-player-4: 28 of 34 Maces stepped back from the lone enemy commander); one
        // walking without fighting flees everything. Each group's footwork rules come from the player (H-HANDS-LANE).
        let mut footwork: HashMap<UnitId, Footwork> = HashMap::new();
        if let Some(pianist) = &self.pianist {
            for group in &pianist.groups {
                let metal: f32 = group.members.iter().filter_map(|id| self.known_units.get(id)).filter_map(|(def, _)| self.world.def(*def)).map(|d| d.metal_cost).sum();
                let commitment_of = match &group.task {
                    super::pianist::GroupTask::Hold { committed: true, .. } | super::pianist::GroupTask::Move { fight: true, .. } => Commitment::All,
                    // A group following a ward stands as a hold does: against mobile units, out of turret reach.
                    super::pianist::GroupTask::Hold { .. } | super::pianist::GroupTask::Follow { .. } => Commitment::Priced { turrets: Vec::new(), commander: false },
                    super::pianist::GroupTask::Engage { .. } => Commitment::Priced { turrets: Vec::new(), commander: metal >= COMMANDER_PARTY_METAL },
                    super::pianist::GroupTask::Move { fight: false, .. } => Commitment::None,
                    super::pianist::GroupTask::Hunt(_) | super::pianist::GroupTask::Rove { .. } => Commitment::None,
                };
                let rules = self.footwork_of(&group.name);
                let name = group.lane_name();
                match &group.task {
                    // A roving group is the lane's whole (H-MICRO-ROVE), within the region the player marked, if one.
                    super::pianist::GroupTask::Rove { within } => {
                        let within = within.as_ref().and_then(|name| self.region_of(name));
                        declared.push((name, group.members.clone(), micro::GroupOrder::new(micro::GroupKind::Rove { within }, Vec::new()), Footwork::raw(), Commitment::All));
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
                                micro::GroupKind::Hold => self.lane.group_state(&screen_name).map_or(screen_at, |s| s.anchor),
                                _ => screen_at,
                            };
                            declared.push((name, artillery, company(micro::GroupOrder::new(kind.clone(), vec![*at])), rules, commitment_of.clone()));
                            declared.push((screen_name, rest, company(micro::GroupOrder::new(kind, vec![screen_point])), rules, commitment_of.clone()));
                        } else {
                            declared.push((name, members, company(micro::GroupOrder::new(kind, vec![*at])), rules, commitment_of.clone()));
                        }
                    }
                    // A hunt is the engine's (H-MICRO-HUNT): attack by id every tick, raw, until the quarry is dead or
                    // lost or the leash ends (docs/design/2026-09-26-threat-response.md §1). A quarry that cloaks is
                    // hidden where it stands when it drops out of sight, not gone: the hunt runs on to its last place
                    // for longer before it ends (H-HUNT-HIDDEN).
                    super::pianist::GroupTask::Hunt(h) => {
                        let lost_frames = if self.quarry_cloaks(h.quarry) { super::pianist::groups::HUNT_HIDDEN_FRAMES } else { micro::HUNT_LOST_FRAMES };
                        let order = micro::HuntOrder { quarry: h.quarry, leash: super::pianist::groups::HUNT_LEASH, lost_frames, since: h.since };
                        declared.push((name, group.members.clone(), micro::GroupOrder::new(micro::GroupKind::Hunt(order), Vec::new()), Footwork::raw(), Commitment::All));
                    }
                    _ => {
                        for id in &group.members {
                            commitment.insert(*id, commitment_of.clone());
                            footwork.insert(*id, rules);
                        }
                    }
                }
            }
        }
        for name in self.lane.group_names() {
            if !declared.iter().any(|(n, ..)| *n == name) {
                self.lane.clear_group(&name);
            }
        }
        for (name, members, order, footwork, commitment) in declared {
            self.lane.set_group(&name, members, order, footwork, commitment);
        }
        self.lane.set_commitments(commitment, footwork);
    }

    /// The player's company cap for a pianist group (`lane` tool, `company`; the companies design §4): the group's
    /// own, else `all`, else `COMPANY_CAP`; 0 lifts it.
    pub(super) fn company_of(&self, group: &str) -> usize {
        let Some(shared) = &self.strategist else { return super::pianist::groups::COMPANY_CAP };
        let company = shared.company.lock().unwrap();
        company.get(&format!("group_{group}")).or_else(|| company.get("all")).copied().unwrap_or(super::pianist::groups::COMPANY_CAP)
    }

    /// The player's footwork setting for a pianist group (H-HANDS-LANE): the group's own, else `all`, else every rule.
    pub(super) fn footwork_of(&self, group: &str) -> Footwork {
        let Some(shared) = &self.strategist else { return Footwork::default() };
        let lane = shared.lane.lock().unwrap();
        lane.get(&format!("group_{group}")).or_else(|| lane.get("all")).copied().unwrap_or_default()
    }

    /// Every tick: the lane over the brain's view, over the brain's own orders of this tick (`commands`, which
    /// it may rewrite); its firings go to the journal; its own commands are appended.
    pub(super) fn micro(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        if self.kit.is_none() {
            return;
        }
        if self.sim.defs.is_empty() {
            self.survey_sim_defs();
        }
        let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
        let mut lane = std::mem::take(&mut self.lane);
        let output = lane.tick(&BrainView { brain: self }, tick, commands, debug);
        self.lane = lane;
        for rule in output.fired {
            self.fire(rule);
        }
        self.journal.milling += output.milling;
        self.journal.groups = self.lane.group_names().into_iter().filter_map(|name| self.lane.group_state(&name).cloned().map(|state| (name, state))).collect();
        commands.extend(output.commands);
        // The hunts' word from the engine: a hunt ended (`groups.rs` `hunt_event`).
        if !output.hunts.is_empty()
            && let Some(pianist) = self.pianist.as_mut()
        {
            let own = &tick.snapshot.own_units;
            let (soldiers, buildings, world) = (&self.enemy_soldiers, &self.enemy_buildings, &self.world);
            let cloaks = |quarry: UnitId| soldiers.get(&quarry).or_else(|| buildings.get(&quarry)).and_then(|(d, ..)| world.def(*d)).is_some_and(|d| d.cloak.is_some());
            for event in &output.hunts {
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
        if !output.rove.is_empty() {
            self.rove_events(&output.rove, tick.frame);
        }
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
}

/// What the lane reads of the brain.
struct BrainView<'a> {
    brain: &'a Brain,
}

impl View for BrainView<'_> {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.brain.world.def(def)
    }

    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        let (reach, dps, speed) = self.brain.sim_stats(def)?;
        let (health, dgun) = match self.brain.sim.defs.get(&def) {
            Some(i) => {
                let unit = &self.brain.sim.rules.units.list[*i];
                (unit.health, unit.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max))
            }
            None => (super::pianist::glossary::entry(self.brain.name(def)).map_or(0.0, |e| e.health), 0.0),
        };
        Some(Stats { reach, dps, speed, health, dgun, sight: self.brain.sight_of(def), area: self.brain.world.def(def).map_or(0.0, |d| d.blast_radius) })
    }

    fn grid_spec(&self) -> (f32, usize, usize) {
        let terrain = &self.brain.world.hello.terrain;
        if terrain.width > 0 && terrain.cell > 0.0 {
            (terrain.cell, terrain.width as usize, terrain.height as usize)
        } else {
            let map = &self.brain.world.hello.map;
            (16.0, (map.width / 16.0).ceil() as usize, (map.height / 16.0).ceil() as usize)
        }
    }

    fn terrain(&self) -> Option<&Terrain> {
        let terrain = &self.brain.world.hello.terrain;
        (terrain.width > 0 && terrain.cell > 0.0).then_some(terrain)
    }

    fn home(&self) -> Vec3 {
        self.brain.home
    }

    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        // A building last seen being built is remembered, but it shoots nothing.
        self.brain.enemy_buildings.iter().filter(|(id, _)| !self.brain.enemy_unfinished.contains(id)).map(|(id, (def, pos, _))| (*id, *def, *pos)).collect()
    }

    fn building_at(&self, id: UnitId) -> Option<Vec3> {
        self.brain.enemy_buildings.get(&id).map(|b| b.1)
    }

    fn enemy_known(&self, id: UnitId) -> bool {
        self.brain.enemy_soldiers.contains_key(&id)
    }

    /// A radar contact is taken for the soldier of theirs we have seen most, as the pricing does.
    fn blip_def(&self) -> Option<UnitDefId> {
        let mut counted: HashMap<UnitDefId, usize> = HashMap::new();
        self.brain.enemy_soldiers.values().for_each(|(def, _, _)| *counted.entry(*def).or_default() += 1);
        counted.into_iter().max_by_key(|(def, n)| (*n, def.0)).map(|(def, _)| def).or(self.brain.kit.as_ref().map(|k| k.line))
    }

    fn enabled(&self, rule: &str) -> bool {
        self.brain.enabled(rule)
    }

    fn mine(&self, _unit: UnitId) -> bool {
        true
    }

    fn label(&self) -> String {
        format!("[ai {}]", self.brain.ai())
    }

    /// Every metal spot a unit of this type can reach from home, with when it was last within sight of ours
    /// (H-SCOUT-SPOTS), his when it lies in another ally team's start box or within 600 of his base; and his base
    /// when one has been found, last seen when the freshest spot round it was.
    fn rove_goals(&self, def: UnitDefId) -> Vec<micro::RoveGoal> {
        let brain = self.brain;
        let walker = brain.walker_of(def);
        let ours = brain.world.hello.ally_team;
        let base = brain.found_enemy_base();
        // His box counts only while the lobby's boxes are honoured (game 10: our own G1 seat stood in "his" box and
        // the rovers walked our own top row for it).
        let honoured = brain.boxes_honoured;
        let theirs = |at: Vec3| (honoured && brain.world.hello.start_boxes.iter().any(|b| b.ally_team != ours && b.contains(at))) || base.is_some_and(|b| b.dist2d(at) < BASE_RADIUS);
        let spots = &brain.world.hello.metal_spots;
        let mut goals: Vec<micro::RoveGoal> = spots.iter().enumerate().filter(|(_, at)| brain.reachable_for(walker, **at)).map(|(i, at)| micro::RoveGoal { name: format!("spot_{i}"), at: *at, seen: brain.spot_seen(i), theirs: theirs(*at) }).collect();
        if let Some(at) = base.filter(|b| brain.reachable_for(walker, *b)) {
            let seen = (0..spots.len()).filter(|i| spots[*i].dist2d(at) < BASE_RADIUS).filter_map(|i| brain.spot_seen(i)).max();
            goals.push(micro::RoveGoal { name: "his base".to_string(), at, seen, theirs: true });
        }
        goals
    }
}

/// A spot this close to his found base is his; the base counts as seen when one of them was.
const BASE_RADIUS: f32 = 600.0;

impl Brain {
    /// The region a roving group is told to stay within (`GroupTask::Rove { within }`): the marked place's cells as
    /// ground, north-west and south-east corners.
    fn region_of(&self, place: &str) -> Option<(Vec3, Vec3)> {
        let pianist = self.pianist.as_ref()?;
        let cells = pianist.places.iter().find(|p| p.name == place)?.cells?;
        Some(self.world.cells_rect(cells))
    }
}
