//! T-GUARD (`docs/design/2026-10-25-territory-defence.md`, step 3b): soldiers to the size of the army he has shown,
//! made before the plan's products by the factories that make the type with the most fighting strength per metal;
//! the guard stands at the base point and fights to the raid with the most of ours at stake among those it matches.
//! The planner does not see the soldiers.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, UnitId, Vec3};
use buildorder::units::Role;

use crate::defence::{flies, toward, DEFENCE_FROM, RAID_NEAR, TURRET_WEIGHT};
use crate::hands::{exit_lane, What, FACTORY_YARD};
use crate::{Territory, MINUTE, SECOND};

/// His armed units count toward the army shown this long after they were last seen.
const SHOWN_FOR: i32 = 5 * MINUTE;
/// The base point: this far from home toward his start, moved on by BASE_STEP while it lies in a factory's yard or
/// exit lane (at most BASE_STEPS times).
const BASE_OUT: f32 = 400.0;
const BASE_STEP: f32 = 200.0;
const BASE_STEPS: usize = 4;
/// His units this near one another are one raid.
const RAID_GROUP: f32 = 400.0;
/// The guard keeps a raid's place this long after it last answered one, and after giving one up (a raid near and
/// none matched) answers nothing this long.
const RAID_OVER: i32 = 20 * SECOND;
const RETREAT: i32 = 20 * SECOND;
/// A soldier is told again when the place moved this far from where it was told, or when it stands idle this long
/// and this far from the place.
const MOVED: f32 = 200.0;
const IDLE_AGAIN: i32 = 5 * SECOND;
/// The `guard:` line every this many frames.
pub(crate) const GUARD_LOG: i32 = 30 * SECOND;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Why {
    Base,
    /// Fighting to a raid: its weight and our building metal within RAID_NEAR of it.
    Raid { weight: f32, stake: f32 },
    /// Where the last raid answered was, until RAID_OVER.
    Holding,
    /// At the base point after giving up a raid it no longer matched, until the frame.
    Retreat(i32),
}

#[derive(Default)]
pub(crate) struct Guard {
    /// His armed mobile ground units seen (the army shown): metal and the frame last seen.
    shown: BTreeMap<UnitId, (f32, i32)>,
    /// His units seen destroyed. The engine keeps a dying unit in its list while its death animation runs, and the
    /// host's enemy list does not leave it out: counted again from those sightings, his bots stood in the army shown
    /// five minutes after their death (terr280: 3,836 of 4,064 dead at 15:00 of game 02). Kept for the game: the
    /// engine gives a unit's id out again only once every other id has been used (`SimObjectIDPool::FreeID`, delayed).
    dead: BTreeSet<UnitId>,
    /// The guard's type, chosen again when the factory types standing change.
    kind: Option<(Vec<usize>, Option<usize>)>,
    /// Where the guard is sent and why, and the frame it last answered a raid.
    place: Option<(Vec3, Why)>,
    last_raid: i32,
    /// Per soldier, the place it was last told and when.
    told: BTreeMap<UnitId, (Vec3, i32)>,
    /// Soldiers ordered from the factories.
    ordered: usize,
}

/// One raid: his units near our buildings within RAID_GROUP of one another.
struct Raid {
    at: Vec3,
    weight: f32,
}

impl Territory {
    /// Every hands tick: his armed mobile units in sight remembered at their metal, the ground ones in the army shown
    /// and the aircraft in the air shown (T-AIR); a radar dot of one seen before keeps it counted, never one seen
    /// destroyed, and those not seen for SHOWN_FOR forgotten.
    pub(crate) fn observe_army(&mut self) {
        let frame = self.frame;
        let seen: Vec<(UnitId, Option<(f32, bool)>)> = self
            .enemies
            .iter()
            .filter(|e| !self.guard.dead.contains(&e.id))
            .filter_map(|e| match e.def.and_then(|d| self.def(d)) {
                Some(info) if info.speed > 0.0 && info.weapon_count > 0 && info.reach > 0.0 && info.build_options.is_empty() => Some((e.id, Some((info.metal_cost, flies(info))))),
                Some(_) => None,
                None => Some((e.id, None)),
            })
            .collect();
        for (id, seen) in seen {
            match seen {
                Some((m, false)) => {
                    self.guard.shown.insert(id, (m, frame));
                }
                Some((m, true)) => {
                    self.air.shown.insert(id, (m, frame));
                }
                None => {
                    for shown in [&mut self.guard.shown, &mut self.air.shown] {
                        if let Some(entry) = shown.get_mut(&id) {
                            entry.1 = frame;
                        }
                    }
                }
            }
        }
        self.guard.shown.retain(|_, (_, at)| frame - *at <= SHOWN_FOR);
        self.air.shown.retain(|_, (_, at)| frame - *at <= SHOWN_FOR);
    }

    /// A unit of his seen destroyed is no longer in the army or the air shown, and is not counted again.
    pub(crate) fn army_lost(&mut self, enemy: UnitId) {
        self.guard.shown.remove(&enemy);
        self.air.shown.remove(&enemy);
        self.guard.dead.insert(enemy);
    }

    /// The army he has shown, in metal.
    fn shown(&self) -> f32 {
        // A fold from 0: an empty sum of floats is -0, which the line printed as "-0".
        self.guard.shown.values().fold(0.0, |sum, (m, _)| sum + m)
    }

    /// The guard wanted: the army he has shown, from DEFENCE_FROM.
    fn guard_wanted(&self) -> f32 {
        if self.frame < DEFENCE_FROM || !self.options.guard { 0.0 } else { self.shown() }
    }

    /// The guard's metal: its soldiers standing, and making (their frames at full cost and the factories' orders not
    /// yet begun).
    fn guard_metal(&self) -> (f32, f32) {
        let units = &self.game().units;
        let (mut standing, mut making) = (0.0f32, 0.0f32);
        for u in self.own.values() {
            let Some(i) = self.unit_index(u.def) else { continue };
            if units.list[i].role == Role::Army {
                if u.being_built {
                    making += units.list[i].metal_cost as f32;
                } else {
                    standing += units.list[i].metal_cost as f32;
                }
            }
        }
        for order in self.hands.orders.values() {
            if let What::Build { unit, frame: None, .. } = &order.what
                && units.list[*unit].role == Role::Army
            {
                making += units.list[*unit].metal_cost as f32;
            }
        }
        (standing, making)
    }

    /// The guard's type: among the products of our standing factories, the mobile ground unit that builds nothing with
    /// the most ground fighting strength per metal.
    fn guard_type(&mut self) -> Option<usize> {
        let game = self.game.clone().expect("started");
        let units = &game.units;
        let mut factories: Vec<usize> = self.own.values().filter(|u| !u.being_built).filter_map(|u| self.unit_index(u.def)).filter(|i| units.list[*i].role == Role::Factory).collect();
        factories.sort_unstable();
        factories.dedup();
        if let Some((known, kind)) = &self.guard.kind
            && *known == factories
        {
            return *kind;
        }
        let products: BTreeSet<usize> = factories.iter().flat_map(|f| units.list[*f].builds.iter().copied()).collect();
        let fighters: Vec<usize> = products.into_iter().filter(|u| units.list[*u].role == Role::Army && self.hello.unit_defs[*u].move_class.is_some() && self.hello.unit_defs[*u].build_options.is_empty()).collect();
        let kind = fighters
            .into_iter()
            .filter_map(|u| self.strength(u).map(|s| (u, s)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(u, _)| u);
        self.guard.kind = Some((factories, kind));
        kind
    }

    /// A factory that makes the guard's type makes one while the guard's metal is under the army he has shown.
    pub(crate) fn guard_order(&mut self, id: UnitId, unit: usize, commands: &mut Vec<Command>) -> bool {
        let wanted = self.guard_wanted();
        if wanted <= 0.0 {
            return false;
        }
        let Some(kind) = self.guard_type() else { return false };
        if !self.game().units.list[unit].builds.contains(&kind) {
            return false;
        }
        let (standing, making) = self.guard_metal();
        if standing + making >= wanted {
            return false;
        }
        let cost = self.game().units.list[kind].build_time;
        let ordered = self.order_one(id, unit, Role::Factory, kind, cost, None, "guard", commands);
        self.guard.ordered += usize::from(ordered);
        ordered
    }

    /// The base point: BASE_OUT from home toward his start, moved on while it lies in a factory's yard or exit lane.
    fn base_point(&self) -> Vec3 {
        let plants: Vec<_> = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Factory)).collect();
        let mut at = toward(self.home, self.enemy_home, BASE_OUT);
        for _ in 0..BASE_STEPS {
            if !plants.iter().any(|f| f.pos.dist2d(at) < FACTORY_YARD || exit_lane(f).contains(at)) {
                break;
            }
            at = toward(at, self.enemy_home, BASE_STEP);
        }
        at
    }

    /// His units near our buildings (as T-COVER weighs them, aircraft apart) in groups within RAID_GROUP of one
    /// another, each at its weighted centre.
    fn raids(&self, buildings: &[(Vec3, f32)]) -> Vec<Raid> {
        let near: Vec<(Vec3, f32)> = self
            .enemies
            .iter()
            .filter(|e| e.def.and_then(|d| self.def(d)).is_none_or(|info| info.move_class.is_some()))
            .map(|e| (e.pos, self.his_weight(e)))
            .filter(|(p, w)| *w > 0.0 && buildings.iter().any(|(b, _)| b.dist2d(*p) <= RAID_NEAR))
            .collect();
        let mut group: Vec<Option<usize>> = vec![None; near.len()];
        let mut raids = Vec::new();
        for i in 0..near.len() {
            if group[i].is_some() {
                continue;
            }
            let g = raids.len();
            group[i] = Some(g);
            let mut members = vec![i];
            let mut next = 0;
            while next < members.len() {
                let m = members[next];
                next += 1;
                for j in 0..near.len() {
                    if group[j].is_none() && near[m].0.dist2d(near[j].0) <= RAID_GROUP {
                        group[j] = Some(g);
                        members.push(j);
                    }
                }
            }
            let weight: f32 = members.iter().map(|m| near[*m].1).sum();
            let (x, z) = members.iter().fold((0.0, 0.0), |(x, z), m| (x + near[*m].0.x * near[*m].1, z + near[*m].0.z * near[*m].1));
            raids.push(Raid { at: Vec3 { x: x / weight, y: 0.0, z: z / weight }, weight });
        }
        raids
    }

    /// Every hands tick: where the guard goes, and a fight order to every soldier not yet told it.
    pub(crate) fn run_guard(&mut self, commands: &mut Vec<Command>) {
        let game = self.game.clone().expect("started");
        let units = &game.units;
        let soldiers: Vec<(UnitId, Vec3, bool)> = self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Army)).map(|u| (u.id, u.pos, u.idle)).collect();
        self.guard.told.retain(|id, _| soldiers.iter().any(|(s, _, _)| s == id));
        if soldiers.is_empty() {
            return;
        }
        let mut buildings: Vec<(Vec3, f32)> = Vec::new();
        let mut turrets: Vec<(Vec3, f32, f32)> = Vec::new();
        for u in self.own.values() {
            let Some(i) = self.unit_index(u.def) else { continue };
            if units.list[i].speed > 0.0 {
                continue;
            }
            // An anti-air turret is no help against the ground (T-AIR's); it is one of the buildings at stake.
            if units.list[i].role == Role::Turret && !self.is_anti_air(i) && !u.being_built {
                turrets.push((u.pos, self.hello.unit_defs[i].reach, units.list[i].metal_cost as f32));
            } else {
                buildings.push((u.pos, units.list[i].metal_cost as f32));
            }
        }
        let (ours, _) = self.guard_metal();
        let raids = self.raids(&buildings);
        let matched = raids
            .iter()
            .filter(|r| ours + TURRET_WEIGHT * turrets.iter().filter(|(p, reach, _)| p.dist2d(r.at) <= *reach).map(|(_, _, m)| m).sum::<f32>() >= r.weight)
            .map(|r| (r, buildings.iter().filter(|(b, _)| b.dist2d(r.at) <= RAID_NEAR).map(|(_, m)| m).sum::<f32>()))
            .max_by(|a, b| a.1.total_cmp(&b.1));
        let base = self.base_point();
        let frame = self.frame;
        let answering = matches!(self.guard.place, Some((_, Why::Raid { .. })));
        let place = match (self.guard.place, matched) {
            (Some((_, Why::Retreat(until))), _) if frame < until => (base, Why::Retreat(until)),
            (_, Some((raid, stake))) => {
                self.guard.last_raid = frame;
                (raid.at, Why::Raid { weight: raid.weight, stake })
            }
            _ if answering && !raids.is_empty() => (base, Why::Retreat(frame + RETREAT)),
            (Some((at, Why::Raid { .. } | Why::Holding)), None) if frame - self.guard.last_raid < RAID_OVER => (at, Why::Holding),
            _ => (base, Why::Base),
        };
        self.guard.place = Some(place);
        let to = place.0;
        for (id, pos, idle) in soldiers {
            let again = match self.guard.told.get(&id) {
                None => true,
                Some((told, at)) => told.dist2d(to) > MOVED || (idle && frame - at >= IDLE_AGAIN && pos.dist2d(to) > MOVED),
            };
            if again {
                commands.push(Command::Fight { unit: id, to, queue: false });
                self.guard.told.insert(id, (to, frame));
            }
        }
    }

    /// `guard:` every GUARD_LOG: the army he has shown and how many units it holds, the guard wanted, the guard's metal
    /// standing and making and how many soldiers stand, the soldiers ordered so far, and where the guard is and why.
    pub(crate) fn log_guard(&mut self) {
        let (standing, making) = self.guard_metal();
        let soldiers = self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Army)).count();
        let kind = self.guard.kind.as_ref().and_then(|(_, k)| *k).map_or("-".to_string(), |k| self.game().units.list[k].name.clone());
        let place = match self.guard.place {
            None => "-".to_string(),
            Some((at, why)) => {
                let why = match why {
                    Why::Base => "base".to_string(),
                    Why::Raid { weight, stake } => format!("raid {weight:.0} with {stake:.0} at stake"),
                    Why::Holding => "holding".to_string(),
                    Why::Retreat(until) => format!("retreat to {:.0} s", until as f32 / SECOND as f32),
                };
                format!("{why} at ({:.0},{:.0})", at.x, at.z)
            }
        };
        self.log(format_args!(
            "guard: shown {:.0} ({} units), wanted {:.0} | guard {standing:.0} standing ({soldiers} {kind}), {making:.0} making, {} ordered | {place}",
            self.shown(),
            self.guard.shown.len(),
            self.guard_wanted(),
            self.guard.ordered
        ));
    }
}
