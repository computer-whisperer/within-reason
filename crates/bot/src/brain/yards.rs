//! Factories' exit lanes and units that cannot move (docs/design/2026-09-22-yard-and-reclaim.md). In hands-2-bulldogs
//! the player's solar collectors at its marked place sealed the Advanced Vehicle Plant's front: five Bulls stood in the
//! yard five to seven minutes each, the plant built nothing for five minutes with metal and energy full, and the
//! picture said nothing of it though the engine raised 156 move-failed events on the Bulls.

use bot_protocol::{Event, Lane, OwnUnit, UnitId, Vec3, UnitDefId};

use super::{Brain, Tick};

/// How far a factory's exit lane runs past its front edge: the engine's empty-spot search walks the front, and a
/// finished unit is sent to `pos + frontdir * (radius + unit radius)` (`Factory.cpp` `SendToEmptySpot`).
pub(super) const LANE_DEPTH: f32 = 320.0;
/// The lane is wider than the factory by this on each side: a Bull is about 40 elmos across.
pub(super) const LANE_MARGIN: f32 = 48.0;
/// A stuck unit is free again once it has moved this far from where its move failed.
pub(super) const STUCK_FREE: f32 = 80.0;
/// A yard with units stuck in it this long wakes the player.
const YARD_WAKE_FRAMES: i32 = 30 * super::FRAMES_PER_SECOND;
/// Build squares are 8 elmos (`SQUARE_SIZE`); a definition's footprint counts them.
const SQUARE: f32 = 8.0;

/// A unit of ours that could not move: when the engine first said so, and where it stood.
pub(super) struct Stuck {
    pub since: i32,
    pub at: Vec3,
}

impl Brain {
    /// The lane a factory's finished units leave through: from its front face out by `LANE_DEPTH`, as wide as the
    /// footprint plus `LANE_MARGIN` a side. None for anything else. Measured from the centre, as it was until
    /// pace-1, the strip's rounded start reached behind the factory too: a constructor wedged between a solar and
    /// the plant's back wall counted as standing in the lane, and three "exit lane is blocked" wakes named a lane
    /// that was free.
    pub(super) fn lane_of(&self, factory: &OwnUnit) -> Option<Lane> {
        self.lane_at(factory.def, factory.pos, factory.facing)
    }

    /// The exit lane a factory of this type would have standing at `pos` with this facing (0 south, the engine's
    /// facing for every building of ours in the recorded games), so a factory not yet standing has a lane too.
    pub(super) fn lane_at(&self, def_id: UnitDefId, pos: Vec3, facing: i32) -> Option<Lane> {
        if !self.world.is_factory_def(def_id) {
            return None;
        }
        let def = self.world.def(def_id)?;
        let (half_width, half_depth) = (def.footprint.0 as f32 * SQUARE / 2.0, def.footprint.1 as f32 * SQUARE / 2.0);
        // The engine's facing: 0 south (+z), 1 east (+x), 2 north (-z), 3 west (-x). The footprint's z is its depth
        // in its own frame whichever way it faces.
        let (fx, fz) = match facing.rem_euclid(4) {
            0 => (0.0, 1.0),
            1 => (1.0, 0.0),
            2 => (0.0, -1.0),
            _ => (-1.0, 0.0),
        };
        let reach = half_depth + LANE_DEPTH;
        let front = Vec3 { x: pos.x + fx * half_depth, y: pos.y, z: pos.z + fz * half_depth };
        Some(Lane { from: front, to: Vec3 { x: pos.x + fx * reach, y: pos.y, z: pos.z + fz * reach }, half_width: half_width + LANE_MARGIN })
    }

    /// Factory build orders the engine has not started yet: (type, site). A site chosen now must stay clear of their
    /// lanes too (escalate-4, fixes-1: two and three factories ordered within a minute, one in another's lane).
    pub(super) fn pending_factories(&self) -> Vec<(UnitDefId, Vec3)> {
        let Some(pianist) = self.pianist.as_ref() else { return Vec::new() };
        pianist
            .tasks
            .values()
            .filter_map(|t| match t {
                super::pianist::Task::Build { def, near, started: false, .. } if self.world.is_factory_def(*def) => Some((*def, *near)),
                _ => None,
            })
            .collect()
    }

    /// Where a new factory of this type must not stand for its own exit lane (facing south) to be clear of what
    /// stands or is ordered: for every building of ours, the mirror of the lane, pointing north from it. A site in
    /// one of these would have that building in its lane (the yard design's open case, decided 2026-09-23).
    pub(super) fn own_lane_keep_out(&self, def_id: UnitDefId, own: &[OwnUnit]) -> Vec<Lane> {
        let Some(def) = self.world.def(def_id).filter(|_| self.world.is_factory_def(def_id)) else { return Vec::new() };
        let (half_width, half_depth) = (def.footprint.0 as f32 * SQUARE / 2.0, def.footprint.1 as f32 * SQUARE / 2.0);
        let reach = half_depth + LANE_DEPTH;
        let mirror = |at: Vec3| Lane { from: Vec3 { x: at.x, y: at.y, z: at.z - half_depth }, to: Vec3 { x: at.x, y: at.y, z: at.z - reach }, half_width: half_width + LANE_MARGIN };
        let standing = own.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && d.extracts_metal == 0.0)).map(|u| mirror(u.pos));
        let ordered = self.pianist.as_ref().map(|p| p.tasks.values()).into_iter().flatten().filter_map(|t| match t {
            super::pianist::Task::Build { def, near, .. } if self.world.def(*def).is_some_and(|d| d.speed == 0.0 && d.extracts_metal == 0.0) => Some(mirror(*near)),
            _ => None,
        });
        standing.chain(ordered).collect()
    }

    /// Every tick: the lanes of our factories, standing or being built (a site chosen now must stay clear of a lane
    /// that will be), and which of our mobile units cannot move.
    pub(super) fn track_yards(&mut self, tick: &Tick) {
        let own = &tick.snapshot.own_units;
        self.lanes = own.iter().filter_map(|u| self.lane_of(u)).collect();
        let pending: Vec<Lane> = self.pending_factories().into_iter().filter_map(|(def, at)| self.lane_at(def, at, 0)).collect();
        self.lanes.extend(pending);
        self.stuck.retain(|id, s| own.iter().any(|u| u.id == *id && u.pos.dist2d(s.at) < STUCK_FREE));
        for event in &tick.events {
            let Event::UnitMoveFailed { unit } = event else { continue };
            if let Some(u) = own.iter().find(|u| u.id == *unit)
                && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)
            {
                self.stuck.entry(*unit).or_insert(Stuck { since: tick.frame, at: u.pos });
            }
        }
        // A yard blocked for a while wakes the player once; a yard that frees itself may wake it again later.
        let factories: Vec<(UnitId, Lane)> = own.iter().filter_map(|u| self.lane_of(u).map(|l| (u.id, l))).collect();
        for (factory, lane) in factories {
            let stuck = self.stuck_in_lane(&lane, own);
            let longest = stuck.iter().map(|(_, since)| tick.frame - since).max().unwrap_or(0);
            if longest < YARD_WAKE_FRAMES {
                self.yard_warned.remove(&factory);
                continue;
            }
            if self.yard_warned.insert(factory) {
                let name = self.actor_name(factory);
                let blockers = self.lane_blockers(&lane, own, factory);
                let names: Vec<String> = stuck.iter().filter_map(|(id, _)| own.iter().find(|u| u.id == *id)).map(|u| self.handle(u)).collect();
                let text = format!(
                    "{name}'s exit lane is blocked: {} of ours ({}) have stood in it unable to move for {}{}",
                    stuck.len(),
                    names.join(", "),
                    super::pianist::clock(longest),
                    if blockers.is_empty() { "; nothing of ours stands in the lane".to_string() } else { format!("; in the lane: {} (the remove tool takes them away)", blockers.iter().map(|u| self.handle(u)).collect::<Vec<_>>().join(", ")) }
                );
                self.trigger("yard", tick.frame, text);
            }
        }
    }

    /// Our buildings whose centre lies in `lane`, the factory itself aside: what the player would remove.
    pub(super) fn lane_blockers<'a>(&self, lane: &Lane, own: &'a [OwnUnit], factory: UnitId) -> Vec<&'a OwnUnit> {
        own.iter().filter(|u| u.id != factory && self.world.def(u.def).is_some_and(|d| d.speed == 0.0) && lane.contains(u.pos)).collect()
    }

    /// Stuck units of ours standing in `lane`, with when each got stuck.
    pub(super) fn stuck_in_lane(&self, lane: &Lane, own: &[OwnUnit]) -> Vec<(UnitId, i32)> {
        own.iter().filter_map(|u| self.stuck.get(&u.id).filter(|_| lane.contains(u.pos)).map(|s| (u.id, s.since))).collect()
    }

    /// The name the player uses for one unit of ours that is not an actor: its type and id (`armsolar_31002`).
    pub(super) fn handle(&self, unit: &OwnUnit) -> String {
        format!("{}_{}", self.name(unit.def), unit.id.0)
    }

    /// The unit of ours a handle names: `armsolar_31002`, or an actor's name (commander, constructor_N, plant_N).
    pub(super) fn unit_by_handle<'a>(&self, handle: &str, own: &'a [OwnUnit]) -> Option<&'a OwnUnit> {
        let id = handle.rsplit_once('_')?.1.parse::<i32>().ok();
        match (handle, id) {
            ("commander", _) => own.iter().find(|u| self.world.is_commander_def(u.def)),
            (_, Some(id)) => own.iter().find(|u| u.id.0 == id),
            _ => None,
        }
    }
}
