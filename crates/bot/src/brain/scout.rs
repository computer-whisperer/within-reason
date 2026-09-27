//! Sight over the metal spots (H-SCOUT-SPOTS): when each was last within a unit of ours' sight, for the picture's
//! staleness words and the player's scout targets. The route-making raider (H-SCOUT-ROUTE) went with the heuristic
//! bot (`docs/design/2026-09-25-one-decider.md`): scouting is the player's to order.

use bot_protocol::{Tick, UnitDefId};

use super::{Brain, FRAMES_PER_SECOND};

/// Sight of a unit type the simulator's table does not know.
const DEFAULT_SIGHT: f32 = 350.0;
/// Spots are surveyed against every unit's sight this often.
const SURVEY_FRAMES: i32 = FRAMES_PER_SECOND / 2;

#[derive(Default)]
pub struct Spots {
    /// Per metal spot (the hello's order), the last frame it was within an own unit's sight.
    seen: Vec<Option<i32>>,
    last_survey: i32,
}

impl Brain {
    /// Sight of a unit type: the simulator's table, else a default.
    pub(super) fn sight_of(&self, def: UnitDefId) -> f32 {
        self.sim.defs.get(&def).map_or(DEFAULT_SIGHT, |&index| self.sim.rules.units.list[index].sight)
    }

    /// H-SCOUT-SPOTS: marks every spot within some unit of ours' sight as seen now.
    pub(super) fn survey_spots(&mut self, tick: &Tick) {
        let count = self.world.hello.metal_spots.len();
        if self.spots.seen.len() != count {
            self.spots.seen = vec![None; count];
        }
        if tick.frame - self.spots.last_survey < SURVEY_FRAMES {
            return;
        }
        self.spots.last_survey = tick.frame;
        if self.sim.defs.is_empty() {
            self.survey_sim_defs();
        }
        let spots = &self.world.hello.metal_spots;
        let sights: Vec<(bot_protocol::Vec3, f32)> = tick.snapshot.own_units.iter().map(|u| (u.pos, self.sight_of(u.def))).collect();
        for (index, spot) in spots.iter().enumerate() {
            if sights.iter().any(|(pos, sight)| pos.dist2d(*spot) <= *sight) {
                self.spots.seen[index] = Some(tick.frame);
            }
        }
    }

    /// The frame a spot was last within an own unit's sight; `None` when never.
    pub(super) fn spot_seen(&self, index: usize) -> Option<i32> {
        self.spots.seen.get(index).copied().flatten()
    }
}
