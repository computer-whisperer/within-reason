//! Fire from out of sight (H-HANDS-SHELLED): the engine's damage event carries the weapon and a direction toward the
//! attacker whether or not it is in sight, so a group shelled by something it cannot see still knows what is shelling
//! it, how far that weapon reaches and which way it stands. The likeliest source goes into the picture as a place the
//! hands can advance on, and the player is woken when a group stands under it (the user, watching pianist-player-6:
//! the army "is getting shelled from just out of frame but has no way to push out to try and kill it").
//! Each group reads the estimate from the hits on its own members (`shelling_for`), and the weapon is the one that did
//! the most damage, not the one that hit most often: player-9 21:15, the one side-wide estimate, voted by hit count,
//! named a Beamer at E4 (490) to every group on the map, group_M at B2 included, while group_G stood inside a seen
//! Gauntlet's 1,220 and was offered a pull-out of 42 elmos.

use std::collections::BTreeMap;

use bot_protocol::{Event, Tick, UnitId, Vec3};

use super::pianist::GroupTask;
use super::{Brain, FRAMES_PER_SECOND};

/// Hits older than this are forgotten (20 s until realtime-1: the place flickered with each hit and the hands re-sent
/// the ball to it, pianist-player-12 and -13; 45 s until 2026-09-23, when the place vanishing mid-walk was the player's
/// complaint in three games).
pub(super) const SHELL_MEMORY: i32 = 90 * FRAMES_PER_SECOND;
/// A shelling with no hit for this long, and a unit of ours standing within `STOOD_ON` of its place, is over: the
/// shooter is dead or gone, and the place goes.
const QUIET_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
const STOOD_ON: f32 = 300.0;
/// The player is woken when this many hits have come from out of sight over this long, once a minute.
const WAKE_HITS: usize = 5;
const WAKE_SECONDS: i32 = 15;
const WAKE_COOLDOWN: i32 = 60 * FRAMES_PER_SECOND;
/// With one ray, the source is presumed this far along it, as a share of the weapon's range.
const ALONG_THE_RAY: f32 = 0.8;

/// One hit from out of sight: where it landed, which way the shooter was, and what fired.
#[derive(Clone, Debug)]
pub(super) struct Shell {
    pub at: Vec3,
    pub dir: Vec3,
    pub range: f32,
    pub weapon: String,
    pub damage: f32,
    pub unit: UnitId,
    pub frame: i32,
}

/// What the recent hits say together.
pub(super) struct Shelling {
    /// The likeliest place of the shooter.
    pub at: Vec3,
    /// The mean direction of the hits, from us toward it.
    pub dir: Vec3,
    pub weapon: String,
    pub range: f32,
    pub hits: usize,
    pub since: i32,
    /// The frame of the latest hit.
    pub last: i32,
    pub units: Vec<UnitId>,
    /// The unit the hits are laid to when one of the weapon's type was seen within its reach of them lately: `at`
    /// is then where it was seen (quick-1, 4:56: the enemy commander's hits on group_A, seen seven seconds before at
    /// the same place, were said as shelling from an unknown; escalate-2: two alarms of the same kind).
    pub attributed: Option<Attribution>,
}

/// A remembered enemy that explains the hits.
pub(super) struct Attribution {
    pub name: String,
    pub seen: i32,
}

impl Brain {
    /// A weapon in words: the unit its definition name is prefixed with ("armart_tawf113_weapon" is the Shellshock's),
    /// else the raw name.
    pub(super) fn weapon_words(&self, weapon: &str) -> String {
        let prefix = weapon.split('_').next().unwrap_or(weapon);
        match self.world.def_named(prefix) {
            Some(def) if self.kit.is_some() => {
                let d = self.world.def(def).expect("a named def exists");
                format!("{} ({}{})", self.name(def), d.name, if d.speed == 0.0 { ", a turret" } else { "" })
            }
            _ => weapon.to_string(),
        }
    }
}

/// Eight compass words for a direction; north is toward smaller z, as the grid's rows run.
pub(super) fn compass(dir: Vec3) -> &'static str {
    let angle = dir.x.atan2(-dir.z).to_degrees().rem_euclid(360.0);
    const NAMES: [&str; 8] = ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"];
    NAMES[(((angle + 22.5) / 45.0) as usize) % 8]
}

impl Brain {
    /// Remembers this tick's hits from out of sight and wakes the player when a group has stood under them.
    pub(super) fn track_shelling(&mut self, tick: &Tick) {
        let frame = tick.frame;
        self.shelling.retain(|s| frame - s.frame <= SHELL_MEMORY);
        if let Some(s) = self.shelling()
            && self.shelling.iter().all(|h| frame - h.frame > QUIET_FRAMES)
            && tick.snapshot.own_units.iter().any(|u| !u.being_built && u.pos.dist2d(s.at) < STOOD_ON)
        {
            self.shelling.clear();
        }
        for event in &tick.events {
            let Event::UnitDamaged { unit, attacker: None, damage, from: Some(dir), weapon: Some(weapon) } = event else { continue };
            if weapon.range <= 0.0 {
                continue;
            }
            let Some(hit) = tick.snapshot.own_units.iter().find(|u| u.id == *unit) else { continue };
            let len = dir.x.hypot(dir.z);
            if len < 0.1 {
                continue;
            }
            self.shelling.push(Shell { at: hit.pos, dir: Vec3 { x: dir.x / len, y: 0.0, z: dir.z / len }, range: weapon.range, weapon: weapon.name.clone(), damage: *damage, unit: *unit, frame });
        }
        let Some(shared) = &self.strategist else { return };
        let Some(s) = self.shelling() else { return };
        if s.hits < WAKE_HITS || frame - s.since < WAKE_SECONDS * FRAMES_PER_SECOND || frame - self.shelling_warned < WAKE_COOLDOWN {
            return;
        }
        // A group already advancing on it needs no waking.
        let advancing = self.pianist.as_ref().is_some_and(|p| p.groups.iter().any(|g| matches!(g.task, GroupTask::Move { fight: true, .. }) && g.members.iter().any(|m| s.units.contains(m))));
        if advancing {
            return;
        }
        self.shelling_warned = frame;
        let laid_to = match &s.attributed {
            Some(a) => format!(": likely the {} seen {} s ago there, and the picture's place `shelling` is where it was seen", a.name, (frame - a.seen) / FRAMES_PER_SECOND),
            None => ": the picture names the place `shelling` where it likeliest stands".to_string(),
        };
        shared.trigger(format!(
            "our soldiers are being shelled from out of sight by a {} (range {:.0}) from the {}, {} hits in {} s{laid_to}",
            self.weapon_words(&s.weapon), s.range, compass(s.dir), s.hits, (frame - s.since) / FRAMES_PER_SECOND
        ));
    }

    /// The recent hits from out of sight on the whole side, read together (the wake and the picture's `shelling` place).
    pub(super) fn shelling(&self) -> Option<Shelling> {
        self.shelling_over(&self.shelling.iter().collect::<Vec<_>>())
    }

    /// The recent hits on these units alone: a group's own shooter, which is not the side's.
    pub(super) fn shelling_for(&self, members: &[UnitId]) -> Option<Shelling> {
        self.shelling_over(&self.shelling.iter().filter(|s| members.contains(&s.unit)).collect::<Vec<_>>())
    }

    /// Hits read together: the weapon that did the most damage, and its likeliest place: the point nearest every
    /// hit's ray when two or more rays cross, else four fifths of the range along the one direction.
    fn shelling_over(&self, all: &[&Shell]) -> Option<Shelling> {
        if all.is_empty() {
            return None;
        }
        let mut damage: BTreeMap<&str, f32> = BTreeMap::new();
        for s in all {
            *damage.entry(s.weapon.as_str()).or_default() += s.damage;
        }
        let weapon = damage.iter().max_by(|a, b| a.1.total_cmp(b.1)).map(|(w, _)| w.to_string())?;
        let shells: Vec<&Shell> = all.iter().copied().filter(|s| s.weapon == weapon).collect();
        let n = shells.len() as f32;
        let range = shells.iter().map(|s| s.range).fold(0.0, f32::max);
        let origin = shells.iter().fold(Vec3::default(), |sum, s| Vec3 { x: sum.x + s.at.x / n, y: 0.0, z: sum.z + s.at.z / n });
        let sum_dir = shells.iter().fold((0.0f32, 0.0f32), |sum, s| (sum.0 + s.dir.x, sum.1 + s.dir.z));
        let len = sum_dir.0.hypot(sum_dir.1).max(1e-3);
        let dir = Vec3 { x: sum_dir.0 / len, y: 0.0, z: sum_dir.1 / len };
        let along = Vec3 { x: origin.x + dir.x * range * ALONG_THE_RAY, y: 0.0, z: origin.z + dir.z * range * ALONG_THE_RAY };
        // Least squares over the rays: sum over hits of (I - d d^T) p = sum of (I - d d^T) a.
        let (mut a11, mut a12, mut a22, mut b1, mut b2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for s in &shells {
            let (p11, p12, p22) = (1.0 - s.dir.x * s.dir.x, -s.dir.x * s.dir.z, 1.0 - s.dir.z * s.dir.z);
            a11 += p11;
            a12 += p12;
            a22 += p22;
            b1 += p11 * s.at.x + p12 * s.at.z;
            b2 += p12 * s.at.x + p22 * s.at.z;
        }
        let det = a11 * a22 - a12 * a12;
        let at = if shells.len() >= 2 && det.abs() > 1e-2 {
            let p = Vec3 { x: (b1 * a22 - b2 * a12) / det, y: 0.0, z: (a11 * b2 - a12 * b1) / det };
            let ahead = (p.x - origin.x) * dir.x + (p.z - origin.z) * dir.z;
            if ahead > 0.0 && p.dist2d(origin) <= range * 1.1 { p } else { along }
        } else {
            along
        };
        let mut units: Vec<UnitId> = shells.iter().map(|s| s.unit).collect();
        units.sort();
        units.dedup();
        let last = shells.iter().map(|s| s.frame).max().unwrap_or(0);
        // A remembered enemy of the weapon's type within its reach of the hits, seen lately: the nearest to the
        // estimate explains the hits, and its last seen place replaces the estimate.
        let shooter = weapon.split('_').next().and_then(|prefix| self.world.def_named(prefix));
        let attributed = shooter.and_then(|def| {
            let reach = range * 1.1;
            let mut seen: Vec<(Vec3, i32)> = self.enemy_buildings.values().filter(|(d, _, _)| *d == def).map(|(_, p, f)| (*p, *f)).collect();
            seen.extend(self.enemy_soldiers.values().filter(|(d, _, f)| *d == def && last - f <= SHELL_MEMORY).map(|(_, p, f)| (*p, *f)));
            if self.is_commander_def(def) {
                seen.extend(self.enemy_commander_seen.filter(|(_, f)| last - f <= SHELL_MEMORY));
            }
            seen.into_iter().filter(|(p, _)| p.dist2d(origin) <= reach).min_by(|a, b| a.0.dist2d(at).total_cmp(&b.0.dist2d(at))).map(|(p, f)| (p, Attribution { name: self.name(def).to_string(), seen: f }))
        });
        let (at, attributed) = match attributed {
            Some((p, a)) => (p, Some(a)),
            None => (at, None),
        };
        Some(Shelling { at, dir, weapon, range, hits: shells.len(), since: shells.iter().map(|s| s.frame).min().unwrap_or(0), last, units, attributed })
    }
}
