//! The game's builder-priority flag (K-mech-builder-priority-is-a-builder-flag): a low builder draws only the team's
//! surplus, full speed or nothing; high is the default and changes nothing. Two rules, both the user's word of
//! 2026-10-13 (docs/code-decisions.md §1): every plant and construction turret is low from birth, as every human's
//! are through the stock widget (H-HANDS-PRIORITY-BIRTH), and a mobile builder is low while it is on a list step
//! carrying the word `low`, high otherwise (H-HANDS-LIST-LOW): the pros' commander guards the plant low in the
//! opening stall so the constructors' extractors take the metal first (K-econ-commander-low-while-assisting). The
//! flag is read back from the engine each tick (`OwnUnit::priority_low`), so an order goes only when it differs from
//! what is wanted, and a bot restarted mid-game or a unit it never touched comes right on the next tick. The engine
//! applies an order a frame or two after the tick that sent it, so the same order is not sent again within a second
//! (prio-player-1: every toggle went twice, one tick apart, before this).

use std::collections::HashMap;

use bot_protocol::{Command, Tick, UnitId};

use super::super::Brain;
use super::lists::step_is_low;

/// Frames within which an order already sent is not sent again, while the flag has not read back yet.
const RESEND_FRAMES: i32 = 30;

impl Brain {
    pub(super) fn tend_priorities(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(pianist) = self.pianist.as_mut() else { return };
        let world = &self.world;
        let mut sent: HashMap<UnitId, (bool, i32)> = std::mem::take(&mut pianist.priority_sent);
        sent.retain(|_, (_, frame)| tick.frame - *frame < RESEND_FRAMES);
        for unit in &tick.snapshot.own_units {
            let low = if world.is_factory_def(unit.def) || world.def(unit.def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0) {
                // A plant or a construction turret (H-HANDS-PRIORITY-BIRTH).
                true
            } else if world.is_mobile_builder(unit.def) {
                // The step it is on (H-HANDS-LIST-LOW); off a list, high.
                pianist.list_steps.get(&unit.id).is_some_and(|(step, _)| step_is_low(step))
            } else {
                // A unit the game gives no flag (a resurrection bot, a soldier).
                continue;
            };
            if unit.priority_low != low && sent.get(&unit.id).is_none_or(|(l, _)| *l != low) {
                commands.push(Command::Priority { unit: unit.id, low });
                sent.insert(unit.id, (low, tick.frame));
            }
        }
        pianist.priority_sent = sent;
    }
}

/// The flag a unit should carry, for tests: a plant or a turret low, a mobile builder as its step says, else none.
#[cfg(test)]
pub(super) fn wanted_low(factory: bool, static_builder: bool, mobile_builder: bool, step: Option<&str>) -> Option<bool> {
    if factory || static_builder {
        return Some(true);
    }
    if mobile_builder {
        return Some(step.is_some_and(step_is_low));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::wanted_low;

    /// Plants and turrets are low whatever they do; a constructor follows the word on its step; a soldier has no flag.
    #[test]
    fn plants_and_turrets_are_low_and_a_builder_follows_its_step() {
        assert_eq!(wanted_low(true, false, false, None), Some(true));
        assert_eq!(wanted_low(false, true, false, Some("extractor")), Some(true));
        assert_eq!(wanted_low(false, false, true, Some("assist 90 low")), Some(true));
        assert_eq!(wanted_low(false, false, true, Some("armsolar #s1")), Some(false));
        assert_eq!(wanted_low(false, false, true, None), Some(false));
        assert_eq!(wanted_low(false, false, false, Some("low")), None);
    }
}
