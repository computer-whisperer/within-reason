//! Construction turrets (H-ECO-NANO-GUARD). A static builder stands idle until it is told what to help: the game
//! gives it no order of its own (escalate-7: five nano turrets beside the plant, no command to any of them, idle in
//! every sample of the game; the user, watching: "the construction turrets don't actually appear to be configured
//! to assist the lab they were placed near"). The brain gives every idle one a guard order on the nearest finished
//! factory in its reach, so it adds its build power to whatever that factory makes, and says in the picture how
//! many stand on each factory.

use std::collections::HashMap;

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId};

use super::{Brain, FRAMES_PER_SECOND};

/// How often an idle turret is told again (its target may have died or been rebuilt).
const RETELL_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// Slack over the turret's build distance for the factory's own size.
const FACTORY_REACH_SLACK: f32 = 100.0;

impl Brain {
    /// A builder that cannot move and makes no units: a construction turret (nano).
    pub(super) fn is_static_builder(&self, def: UnitDefId) -> bool {
        self.world.def(def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty())
    }

    /// The finished factory nearest a turret and within its reach, if any.
    fn factory_in_reach<'a>(&self, nano: &OwnUnit, own: &'a [OwnUnit]) -> Option<&'a OwnUnit> {
        let reach = self.world.def(nano.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK;
        own.iter()
            .filter(|u| !u.being_built && self.world.is_factory_def(u.def) && u.pos.dist2d(nano.pos) <= reach)
            .min_by(|a, b| a.pos.dist2d(nano.pos).total_cmp(&b.pos.dist2d(nano.pos)))
    }

    /// Every idle finished construction turret guards the nearest factory in its reach.
    pub(super) fn tend_nanos(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let mut told: Vec<(UnitId, UnitId)> = Vec::new();
        for nano in own.iter().filter(|u| !u.being_built && u.idle && self.is_static_builder(u.def)) {
            if self.nano_guards.get(&nano.id).is_some_and(|(_, at)| tick.frame - at < RETELL_FRAMES) {
                continue;
            }
            if let Some(factory) = self.factory_in_reach(nano, own) {
                commands.push(Command::Guard { unit: nano.id, target: factory.id });
                told.push((nano.id, factory.id));
            }
        }
        for (nano, factory) in told {
            self.nano_guards.insert(nano, (factory, tick.frame));
        }
        self.nano_guards.retain(|nano, _| own.iter().any(|u| u.id == *nano));
    }

    /// (turrets within reach of this factory, of them idle) for the picture's factory entry.
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

    pub(super) fn nanos_on(&self, factory: &OwnUnit, own: &[OwnUnit]) -> (usize, usize) {
        let mut near = 0;
        let mut idle = 0;
        for nano in own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def)) {
            let reach = self.world.def(nano.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK;
            if nano.pos.dist2d(factory.pos) <= reach {
                near += 1;
                idle += usize::from(nano.idle);
            }
        }
        (near, idle)
    }
}

/// The type the map holds: turret to (the factory it guards, when it was told).
pub(super) type NanoGuards = HashMap<UnitId, (UnitId, i32)>;
