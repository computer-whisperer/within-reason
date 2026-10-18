//! Fire from out of sight as standing places (H-GUNS), and what our own shots land (H-HANDS-FIRING-LINE).
//!
//! `shelling.rs` reads the last ninety seconds of hits from out of sight into one live estimate per group, the
//! picture's `shelling` place, which moves with each hit and goes when the hits stop. What it never kept was the
//! fact behind six minutes of the same weapon from the same bearing: his guns stand there. human-8v8-1: Nightmare's
//! player wrote "a Bastion we can't see, likely at G2" turn after turn while the hands forgot each estimate after
//! ninety seconds, the report said "nothing of his, in sight now" over the cell, and the commander planned on "radar
//! sight" into it. The user, 2026-10-18: "the fact that our units were constantly shelled from those locations should
//! have been a dead giveaway ... both should have been knowable from our seats."
//!
//! A gun is a weapon type and an estimate of where it stands, grown from every hit of that type whose estimate falls
//! near it; it is named `guns_N` for good, is a place in the picture (so a packet can name it: never there, shell it,
//! walk onto it), a line in both readers' reports, and an entry in its cell's scouting line. It is kept ten minutes
//! past its last hit, and dropped sooner when a unit of ours has stood on the estimate fifteen seconds with no hit.
//!
//! The firing line is the other half: a group that fires and lands nothing on anything in sight is shooting at what it
//! cannot see, or into a shield. The engine reports a hit on his unit only while that unit is in sight or radar, so
//! the count is of what was seen to land.

use bot_protocol::{Event, Tick, UnitId, Vec3};

use super::{Brain, FRAMES_PER_SECOND};

/// A gun is forgotten this long after its last hit.
const GUN_MEMORY: i32 = 10 * 60 * FRAMES_PER_SECOND;
/// A fresh estimate within this of a remembered gun of the same weapon is that gun.
const GUN_MERGE: f32 = 600.0;
/// A unit of ours standing this close to the estimate, with no hit of the gun's for `GUN_QUIET`, means the gun is
/// gone or the estimate was wrong: the place goes.
const GUN_STOOD_ON: f32 = 300.0;
const GUN_QUIET: i32 = 15 * FRAMES_PER_SECOND;
/// Shots and landed hits are counted over this window for the group's `firing` words.
pub(super) const FIRING_WINDOW: i32 = 30 * FRAMES_PER_SECOND;
/// Fewer shots than this in the window say nothing.
const FIRING_MIN_SHOTS: usize = 10;

/// A weapon of his firing from out of sight, remembered where the hits' bearings put it.
#[derive(Clone, Debug)]
pub(super) struct Gun {
    /// `guns_N`, kept for the gun's life.
    pub name: String,
    /// The engine's weapon name (`shelling.rs` reads the unit type off its prefix).
    pub weapon: String,
    pub range: f32,
    /// The estimate: the hits' least-squares point, moved toward each new estimate by its share of the hits; a seen
    /// unit of the weapon's type within reach replaces it.
    pub at: Vec3,
    pub since: i32,
    pub last: i32,
    pub hits: usize,
    pub damage: f32,
    /// The type a sighting laid the hits to, when one did.
    pub attributed: Option<String>,
}

impl Brain {
    /// Grows the guns from the hits since the last think, and forgets the quiet and the stood-on.
    pub(super) fn track_guns(&mut self, tick: &Tick) {
        let frame = tick.frame;
        let standing: Vec<Vec3> = tick.snapshot.own_units.iter().filter(|u| !u.being_built).map(|u| u.pos).collect();
        self.guns.retain(|g| frame - g.last <= GUN_MEMORY && !(frame - g.last > GUN_QUIET && standing.iter().any(|p| p.dist2d(g.at) <= GUN_STOOD_ON)));
        let fresh: Vec<&super::shelling::Shell> = self.shelling.iter().filter(|s| s.frame > self.guns_until).collect();
        self.guns_until = frame;
        if fresh.is_empty() {
            return;
        }
        let mut weapons: Vec<String> = fresh.iter().map(|s| s.weapon.clone()).collect();
        weapons.sort();
        weapons.dedup();
        // One nameless candidate per weapon: the estimate from every recent hit of the weapon (the live one), the
        // count and damage from the new hits only.
        let mut candidates: Vec<Gun> = Vec::new();
        for weapon in weapons {
            let all: Vec<&super::shelling::Shell> = self.shelling.iter().filter(|s| s.weapon == weapon).collect();
            let Some(estimate) = self.shelling_over(&all) else { continue };
            let new: Vec<&&super::shelling::Shell> = fresh.iter().filter(|s| s.weapon == weapon).collect();
            let (since, last) = (new.iter().map(|s| s.frame).min().unwrap_or(frame), new.iter().map(|s| s.frame).max().unwrap_or(frame));
            candidates.push(Gun { name: String::new(), weapon, range: estimate.range, at: estimate.at, since, last, hits: new.len(), damage: new.iter().map(|s| s.damage).sum(), attributed: estimate.attributed.map(|a| a.name) });
        }
        for new in candidates {
            match self.guns.iter_mut().filter(|g| g.weapon == new.weapon).min_by(|a, b| a.at.dist2d(new.at).total_cmp(&b.at.dist2d(new.at))).filter(|g| g.at.dist2d(new.at) <= GUN_MERGE) {
                Some(gun) => {
                    // A sighting pins the place; else the estimate drifts toward the new one by its share of the hits.
                    let share = new.hits as f32 / (gun.hits + new.hits).max(1) as f32;
                    gun.at = if new.attributed.is_some() { new.at } else { Vec3 { x: gun.at.x + (new.at.x - gun.at.x) * share, y: 0.0, z: gun.at.z + (new.at.z - gun.at.z) * share } };
                    gun.last = gun.last.max(new.last);
                    gun.hits += new.hits;
                    gun.damage += new.damage;
                    gun.range = gun.range.max(new.range);
                    if new.attributed.is_some() {
                        gun.attributed = new.attributed;
                    }
                }
                None => {
                    self.guns_named += 1;
                    self.guns.push(Gun { name: format!("guns_{}", self.guns_named), ..new });
                }
            }
        }
    }

    /// One line per gun for the reports, the most recent fire first.
    pub(super) fn gun_lines(&self, frame: i32) -> Vec<String> {
        let mut guns: Vec<&Gun> = self.guns.iter().collect();
        guns.sort_by_key(|g| std::cmp::Reverse(g.last));
        guns.iter()
            .map(|g| {
                let laid = g.attributed.as_ref().map_or(String::new(), |a| format!("; {a} seen standing there"));
                format!(
                    "{}: {} (reach {:.0}) about {} ({}, {}) by the bearings of {} hit{} ({:.0} damage) since {}, last {} ago{laid}",
                    g.name,
                    self.weapon_words(&g.weapon),
                    g.range,
                    self.world.grid(g.at),
                    g.at.x as i32,
                    g.at.z as i32,
                    g.hits,
                    if g.hits == 1 { "" } else { "s" },
                    g.damage,
                    super::pianist::picture::clock(g.since),
                    super::pianist::picture::clock(frame - g.last)
                )
            })
            .collect()
    }

    /// The `guns_N` places laid to a cell, each with its weapon's unit type, for the scouting block.
    pub(super) fn guns_in(&self, cell: &str) -> Vec<String> {
        self.guns.iter().filter(|g| self.world.grid(g.at) == cell).map(|g| format!("{} ({})", g.name, g.weapon.split('_').next().unwrap_or(&g.weapon))).collect()
    }

    /// Remembers this tick's shots of ours and the hits of ours the engine reported landing on his units.
    pub(super) fn track_fire(&mut self, tick: &Tick) {
        let frame = tick.frame;
        self.fired_log.retain(|(f, _)| frame - f <= FIRING_WINDOW);
        self.landed_log.retain(|(f, _)| frame - f <= FIRING_WINDOW);
        for event in &tick.events {
            match event {
                Event::WeaponFired { unit, .. } => self.fired_log.push((frame, *unit)),
                Event::EnemyDamaged { attacker: Some(attacker), .. } => self.landed_log.push((frame, *attacker)),
                _ => {}
            }
        }
    }

    /// What a group's members fired in the last thirty seconds and what of it was seen to land; None under ten shots.
    pub(super) fn firing_words(&self, members: &[UnitId]) -> Option<String> {
        let shots = self.fired_log.iter().filter(|(_, u)| members.contains(u)).count();
        if shots < FIRING_MIN_SHOTS {
            return None;
        }
        let landed = self.landed_log.iter().filter(|(_, u)| members.contains(u)).count();
        Some(if landed == 0 {
            format!("{shots} shots in the last {} s and nothing of his seen hurt by them: the targets are out of sight (a hit on what we cannot see is not reported), or a shield takes them", FIRING_WINDOW / FRAMES_PER_SECOND)
        } else {
            format!("{shots} shots in the last {} s, {landed} hits seen landing on his units", FIRING_WINDOW / FRAMES_PER_SECOND)
        })
    }
}

#[cfg(test)]
mod tests {
    use bot_protocol::{Event, OwnUnit, Resource, Snapshot, Tick, UnitId, Vec3, Weapon};

    use super::super::pianist::fixtures::{at, brain_of, own};
    use super::super::shelling::Shell;

    fn tick(frame: i32, ours: Vec<OwnUnit>, events: Vec<Event>) -> Tick {
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        Tick { frame, late: 0, events, snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: ours, allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None } }
    }

    /// Hits of one weapon from one bearing become a gun with a stable name and a place; a second weapon is another
    /// gun; the gun outlives the ninety-second hit memory, and goes when ours stand on its place with the fire stopped.
    #[test]
    fn hits_from_out_of_sight_become_a_standing_gun() {
        let mut brain = brain_of(&["armcom", "armsolar"]);
        let shell = |frame: i32, at: Vec3, dir: Vec3, weapon: &str, range: f32| Shell { at, dir, range, weapon: weapon.into(), damage: 100.0, unit: UnitId(10), frame };
        let north = Vec3 { x: 0.0, y: 0.0, z: -1.0 };
        let quiet = |frame: i32, ours: Vec<OwnUnit>| tick(frame, ours, Vec::new());
        // Four hits from the north on units at z 3000 and 3200: one gun, the estimate up the ray.
        brain.shelling = (0..4).map(|i| shell(30 + i, at(5000.0 + i as f32 * 20.0, 3000.0 + (i % 2) as f32 * 200.0), north, "legbastion_heat_ray", 1100.0)).collect();
        brain.track_guns(&quiet(60, vec![own(10, 1, at(5000.0, 3000.0))]));
        assert_eq!(brain.guns.len(), 1);
        let gun = brain.guns[0].clone();
        assert_eq!((gun.name.as_str(), gun.hits, gun.damage), ("guns_1", 4, 400.0));
        assert!(gun.at.z < 2600.0 && gun.at.z > 1800.0, "up the ray toward the north: {:?}", gun.at);
        // Later hits of the same weapon near the same estimate grow the gun; another weapon is another gun.
        brain.shelling.push(shell(300, at(5100.0, 3000.0), north, "legbastion_heat_ray", 1100.0));
        brain.shelling.push(shell(300, at(5100.0, 3000.0), Vec3 { x: 1.0, y: 0.0, z: 0.0 }, "legelrpcmech_cannon", 3100.0));
        brain.track_guns(&quiet(330, vec![own(10, 1, at(5000.0, 3000.0))]));
        assert_eq!(brain.guns.len(), 2, "{:?}", brain.guns);
        assert_eq!((brain.guns[0].hits, brain.guns[1].name.as_str()), (5, "guns_2"));
        let lines = brain.gun_lines(330);
        assert!(lines[0].starts_with("guns_"), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("by the bearings of 5 hits (500 damage)")), "{lines:?}");
        assert!(brain.guns_in(&brain.world.grid(brain.guns[0].at)).iter().any(|g| g.starts_with("guns_1 (legbastion)")));
        // The hit memory forgets the shells; the gun stands ten minutes.
        brain.shelling.clear();
        brain.track_guns(&quiet(330 + 5 * 60 * 30, vec![own(10, 1, at(5000.0, 3000.0))]));
        assert_eq!(brain.guns.len(), 2);
        // Ours stand on the first gun's place with the fire long stopped: it goes; the other stays.
        let stood = brain.guns[0].at;
        brain.track_guns(&quiet(330 + 6 * 60 * 30, vec![own(10, 1, stood)]));
        assert_eq!(brain.guns.len(), 1);
        assert_eq!(brain.guns[0].name, "guns_2");
        // The firing words: shots with nothing landing, then with hits.
        let shots: Vec<Event> = (0..12).map(|_| Event::WeaponFired { unit: UnitId(10), weapon: Some(Weapon { name: "x".into(), range: 300.0 }) }).collect();
        brain.track_fire(&tick(1000, vec![own(10, 1, at(5000.0, 3000.0))], shots));
        assert!(brain.firing_words(&[UnitId(10)]).unwrap().starts_with("12 shots in the last 30 s and nothing of his seen hurt"));
        assert!(brain.firing_words(&[UnitId(11)]).is_none());
        brain.track_fire(&tick(1030, vec![own(10, 1, at(5000.0, 3000.0))], vec![Event::EnemyDamaged { enemy: UnitId(70), attacker: Some(UnitId(10)), damage: 50.0, weapon: None }]));
        assert!(brain.firing_words(&[UnitId(10)]).unwrap().contains(", 1 hits seen landing"));
    }
}
