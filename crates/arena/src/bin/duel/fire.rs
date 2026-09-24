//! The fire instrument in a duel: how much of its time in reach an army spent shooting, how much muzzled and why,
//! what it did to itself. The definitions are `run/fire.py`'s, which reads the same thing out of a live game's
//! record (`docs/harness/record-format.md`, the `s` line), so a duel's numbers and a game's can be set side by side.

use std::collections::HashMap;

use bot_protocol::{UnitId, Vec3};

/// The micro's FOCUS_SLACK: a target this far beyond the reach still counts as in it.
pub const SLACK: f32 = 20.0;
/// A friend this close to the line of fire blocks it (a tank's collision radius is about 20).
pub const HULL: f32 = 24.0;
/// A target within this of the reach's edge: the engine's range test and ours may disagree.
pub const EDGE: f32 = 40.0;
/// Seconds in reach without a shot before a soldier counts muzzled, or twice its reload when that is longer.
pub const QUIET_FLOOR: f32 = 3.0;

/// Why a muzzled second was muzzled, in `run/fire.py`'s order of tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// A friend within `HULL` of the line to the nearest enemy in reach, and nearer than it.
    FriendOnLine = 0,
    /// The nearest enemy in reach stands within `EDGE` of the reach's edge.
    Edge = 1,
    /// Neither: terrain, turning, a target it will not shoot.
    Clear = 2,
}

/// One army's fire over a duel.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    /// Soldier-seconds with an enemy inside the soldier's reach (plus `SLACK`).
    pub reach_seconds: u32,
    /// Shots fired in those seconds.
    pub shots: u32,
    /// Muzzled seconds by `Cause`.
    pub muzzled: [u32; 3],
    /// Hit points this army did to itself.
    pub friendly_fire: f32,
    /// Hit points this army did to the enemy army.
    pub dealt: f32,
}

impl Tally {
    pub fn muzzled_seconds(&self) -> u32 {
        self.muzzled.iter().sum()
    }
}

/// What an army accumulates between samples.
#[derive(Debug, Default)]
pub struct Fire {
    pub tally: Tally,
    /// Shots each unit fired since the last sample.
    shots: HashMap<UnitId, u32>,
    /// Consecutive in-reach seconds each unit has gone without a shot.
    quiet: HashMap<UnitId, u32>,
}

impl Fire {
    pub fn shot(&mut self, unit: UnitId) {
        *self.shots.entry(unit).or_default() += 1;
    }

    /// One second's sample: `own` are this army's living units with their type's reach and reload (`UnitDefInfo`),
    /// `enemies` where the other army's stand. An unarmed unit samples nothing, and blocks nobody's line either
    /// (`run/fire.py` counts soldiers only).
    pub fn sample(&mut self, own: &[(UnitId, Vec3, f32, f32)], enemies: &[Vec3]) {
        let shots = std::mem::take(&mut self.shots);
        for &(unit, at, reach, reload) in own {
            if reach <= 0.0 {
                continue;
            }
            let reach = reach + SLACK;
            let patience = QUIET_FLOOR.max(2.0 * reload);
            let nearest = (enemies.iter().copied())
                .filter(|e| e.dist2d(at) < reach)
                .min_by(|a, b| a.dist2d(at).total_cmp(&b.dist2d(at)));
            let Some(target) = nearest else {
                self.quiet.insert(unit, 0);
                continue;
            };
            self.tally.reach_seconds += 1;
            let fired = shots.get(&unit).copied().unwrap_or(0);
            self.tally.shots += fired;
            let quiet = self.quiet.entry(unit).or_default();
            if fired > 0 {
                *quiet = 0;
                continue;
            }
            *quiet += 1;
            if (*quiet as f32) < patience {
                continue;
            }
            let range = at.dist2d(target);
            let blocked = (own.iter())
                .any(|&(other, f, other_reach, _)| other != unit && other_reach > 0.0 && near_segment(f, at, target) < HULL && f.dist2d(at) < range);
            let cause = if blocked {
                Cause::FriendOnLine
            } else if range > reach - EDGE {
                Cause::Edge
            } else {
                Cause::Clear
            };
            self.tally.muzzled[cause as usize] += 1;
        }
    }
}

/// Distance from `p` to the segment `a`-`b`, in the ground plane.
fn near_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let l2 = dx * dx + dz * dz;
    if l2 <= 0.0 {
        return p.dist2d(a);
    }
    let t = (((p.x - a.x) * dx + (p.z - a.z) * dz) / l2).clamp(0.0, 1.0);
    (p.x - (a.x + t * dx)).hypot(p.z - (a.z + t * dz))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    #[test]
    fn a_rear_rank_behind_a_friend_is_muzzled_by_the_friend() {
        let mut fire = Fire::default();
        let own = [(UnitId(1), at(0.0, 0.0), 350.0, 1.2), (UnitId(2), at(50.0, 0.0), 350.0, 1.2)];
        let enemies = [at(300.0, 0.0)];
        for _ in 0..3 {
            fire.shot(UnitId(2));
            fire.sample(&own, &enemies);
        }
        assert_eq!(fire.tally.reach_seconds, 6);
        assert_eq!(fire.tally.shots, 3);
        // The rear unit reaches three quiet seconds (the floor; 2 x 1.2 is less) on the third sample.
        assert_eq!(fire.tally.muzzled, [1, 0, 0]);
    }

    #[test]
    fn a_target_at_the_edge_and_a_clear_line_are_told_apart() {
        let mut fire = Fire::default();
        let own = [(UnitId(1), at(0.0, -300.0), 350.0, 1.2), (UnitId(2), at(0.0, 300.0), 350.0, 1.2)];
        let enemies = [at(340.0, -300.0), at(200.0, 300.0)];
        for _ in 0..3 {
            fire.sample(&own, &enemies);
        }
        assert_eq!(fire.tally.muzzled, [0, 1, 1]);
    }

    #[test]
    fn a_shot_resets_the_quiet_count_and_out_of_reach_is_not_counted() {
        let mut fire = Fire::default();
        let own = [(UnitId(1), at(0.0, 0.0), 350.0, 1.2)];
        fire.sample(&own, &[at(1000.0, 0.0)]);
        assert_eq!(fire.tally.reach_seconds, 0);
        for second in 0..5 {
            if second == 2 {
                fire.shot(UnitId(1));
            }
            fire.sample(&own, &[at(200.0, 0.0)]);
        }
        assert_eq!(fire.tally.muzzled_seconds(), 0);
    }
}
