//! Construction turrets (H-ECO-NANO-GUARD). The engine gives a static builder no order of its own (escalate-7: five
//! nano turrets beside the plant, no command to any of them, idle in every sample; the user, watching: "the
//! construction turrets don't actually appear to be configured to assist the lab they were placed near"). The game's
//! UI does: BAR's ImmobileBuilder widget, on by default, sets every immobile builder to the maneuver move state and
//! gives it a fight order with the META bit at a point 50 elmos off its position toward the map's centre, which an
//! immobile unit never reaches (a fight finishes within 64), so every slow update the engine's builder fight loop
//! repairs or helps build whatever of ours stands within its build distance, stationary targets first and the
//! nearest of them, else takes apart wrecks and enemy units there (`CBuilderCAI::ExecuteFight`). The maneuver state
//! matters: under hold position the loop skips mobile units being built, a factory's products among them. A second
//! default widget, Priority Construction Turrets, puts every such turret in reach of a newly started turret frame
//! onto that frame, so a row of turrets builds itself.
//!
//! The brain replicates both (the user, 2026-10-30, watching econ-flagship-1: "nano turrets are currently ordered to
//! focus on a single target rather than being free to assist generally. The default configuration for a nano turret
//! from the player ui is opportunistically helpful, and we should probably figure out how to replicate that rather
//! than binding it to a facility"). From 2026-09-23 to 2026-10-30 every idle turret guarded the nearest finished
//! factory in its reach instead, which fed that factory's product and nothing else in reach.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::{Brain, FRAMES_PER_SECOND};

/// How long an order given to a turret is trusted before an idle turret is told again (the snapshot shows the
/// order a tick late; an order the engine refused leaves the turret idle).
const RETELL_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// Slack over the turret's build distance for the target's own size (the engine adds the target's radius).
const FACTORY_REACH_SLACK: f32 = 100.0;
/// The widget's offset of the fight point from the turret, toward the map's centre: past the 64 elmos within
/// which a fight order finishes, inside the map.
const BESIDE: f32 = 50.0;
/// The engine's maneuver move state.
const MANEUVER: i32 = 1;

/// The turrets' state: when each was last told (its standing order or a frame), which have their move state set, and
/// the turret frames already seen (each pulls the turrets in reach once, when it appears).
#[derive(Default)]
pub(super) struct Nanos {
    told: HashMap<UnitId, i32>,
    set_up: HashSet<UnitId>,
    frames_seen: HashSet<UnitId>,
}

impl Brain {
    /// A builder that cannot move and makes no units: a construction turret (nano).
    pub(super) fn is_static_builder(&self, def: UnitDefId) -> bool {
        self.world.def(def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty())
    }

    /// Whether a turret's build distance (with the slack for the target's size) covers `target`.
    pub(super) fn turret_reaches(&self, nano: &OwnUnit, target: &OwnUnit) -> bool {
        nano.pos.dist2d(target.pos) <= self.world.def(nano.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK
    }

    /// The widget's fight point: 50 elmos off the turret toward the map's centre on each axis.
    fn beside(&self, at: Vec3) -> Vec3 {
        let map = &self.world.hello.map;
        let toward = |p: f32, size: f32| if p > size / 2.0 { p - BESIDE } else { p + BESIDE };
        Vec3 { x: toward(at.x, map.width), y: at.y, z: toward(at.z, map.height) }
    }

    /// Every finished construction turret holds the game's default order: a new turret frame pulls the turrets in
    /// reach onto it; an idle turret is set to maneuver once and given the fight-in-place beside itself.
    pub(super) fn tend_nanos(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let turrets: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def)).collect();
        let mut told: Vec<UnitId> = Vec::new();
        let frames: Vec<&OwnUnit> = own.iter().filter(|u| u.being_built && self.is_static_builder(u.def)).collect();
        for frame in frames {
            if !self.nanos.frames_seen.insert(frame.id) {
                continue;
            }
            for nano in turrets.iter().filter(|t| self.turret_reaches(t, frame)) {
                commands.push(Command::Repair { unit: nano.id, target: frame.id, queue: false });
                told.push(nano.id);
            }
        }
        let idle: Vec<&OwnUnit> = turrets.iter().copied().filter(|u| u.idle && !told.contains(&u.id)).collect();
        for nano in idle {
            if self.nanos.told.get(&nano.id).is_some_and(|at| tick.frame - at < RETELL_FRAMES) {
                continue;
            }
            if !self.nanos.set_up.contains(&nano.id) {
                commands.push(Command::MoveState { unit: nano.id, state: MANEUVER });
            }
            commands.push(Command::Tend { unit: nano.id, to: self.beside(nano.pos) });
            told.push(nano.id);
        }
        for nano in told {
            self.nanos.told.insert(nano, tick.frame);
            self.nanos.set_up.insert(nano);
        }
        self.nanos.told.retain(|nano, _| own.iter().any(|u| u.id == *nano));
        self.nanos.set_up.retain(|nano| own.iter().any(|u| u.id == *nano));
        self.nanos.frames_seen.retain(|frame| own.iter().any(|u| u.id == *frame));
    }

    /// The seat's build power by kind, and what of it stands in reach of each factory now (a builder is in reach
    /// when its build range covers the factory, with the slack `nanos_on` allows).
    pub(super) fn build_power(&self, own: &[OwnUnit]) -> crate::strategist::shared::BuildPower {
        use crate::strategist::shared::{BuildPower, FactoryPower};
        let power = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.build_speed);
        let reaches = |builder: &OwnUnit, factory: &OwnUnit| builder.pos.dist2d(factory.pos) <= self.world.def(builder.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK;
        let standing: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && power(u) > 0.0).collect();
        let factories: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.world.is_factory_def(u.def)).collect();
        let turrets: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.is_static_builder(u.def)).collect();
        let mobile: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).collect();
        let sum = |units: &[&OwnUnit]| (units.len(), units.iter().map(|u| power(u)).fold(0.0, |a, b| a + b));
        BuildPower {
            mobile: sum(&mobile),
            turrets: sum(&turrets),
            mobile_away: sum(&mobile.iter().copied().filter(|b| !factories.iter().any(|f| reaches(b, f))).collect::<Vec<_>>()).1,
            factories: factories
                .iter()
                .map(|f| FactoryPower {
                    name: format!("{}_{}", self.world.def(f.def).map_or("factory", |d| d.name.as_str()), f.id.0),
                    own: power(f),
                    turrets: sum(&turrets.iter().copied().filter(|b| reaches(b, f)).collect::<Vec<_>>()),
                    mobile: sum(&mobile.iter().copied().filter(|b| reaches(b, f)).collect::<Vec<_>>()),
                })
                .collect(),
        }
    }

    /// (turrets within reach of this factory, of them idle) for the picture's factory entry.
    pub(super) fn nanos_on(&self, factory: &OwnUnit, own: &[OwnUnit]) -> (usize, usize) {
        let mut near = 0;
        let mut idle = 0;
        for nano in own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def)) {
            if self.turret_reaches(nano, factory) {
                near += 1;
                idle += usize::from(nano.idle);
            }
        }
        (near, idle)
    }
}
