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

    /// The exit lane a factory of this type would have standing at `pos` with this facing, so a factory not yet
    /// standing has a lane too.
    pub(super) fn lane_at(&self, def_id: UnitDefId, pos: Vec3, facing: i32) -> Option<Lane> {
        let (half_width, half_depth) = self.factory_half_size(def_id)?;
        let (fx, fz) = facing_dir(facing);
        let reach = half_depth + LANE_DEPTH;
        let front = Vec3 { x: pos.x + fx * half_depth, y: pos.y, z: pos.z + fz * half_depth };
        Some(Lane { from: front, to: Vec3 { x: pos.x + fx * reach, y: pos.y, z: pos.z + fz * reach }, half_width: half_width + LANE_MARGIN })
    }

    /// Half the footprint of a factory type across its front and from front to back, in elmos; None for anything
    /// else. The footprint's z is its depth in its own frame whichever way it faces.
    fn factory_half_size(&self, def_id: UnitDefId) -> Option<(f32, f32)> {
        if !self.world.is_factory_def(def_id) {
            return None;
        }
        let def = self.world.def(def_id)?;
        Some((def.footprint.0 as f32 * SQUARE / 2.0, def.footprint.1 as f32 * SQUARE / 2.0))
    }

    /// The facings to try for a factory at `anchor`, the one whose lane points most toward the enemy's start first:
    /// its units leave toward where they are going. Every facing is offered; the shim takes the first with a site.
    pub(super) fn facings_toward_the_enemy(&self, anchor: Vec3) -> [i32; 4] {
        let enemy = self.enemy_base(self.home);
        let (dx, dz) = (enemy.x - anchor.x, enemy.z - anchor.z);
        let mut facings = [0, 1, 2, 3];
        facings.sort_by(|a, b| {
            let along = |f: &i32| { let (fx, fz) = facing_dir(*f); fx * dx + fz * dz };
            along(b).total_cmp(&along(a))
        });
        facings
    }

    /// Where a new factory of this type must not stand for its exit lane, with this facing, to be clear of what
    /// stands, is ordered or will be built: for every building of ours and every metal spot of the map, the lane
    /// mirrored behind it. A site in one of these would have that building or spot in its lane (the yard design's
    /// open case, decided 2026-09-23). The spots count whether or not an extractor stands on them: an extractor is
    /// placed exactly on its spot and no lane keeps it out, so a factory placed with a spot in its lane got an
    /// extractor there as a matter of course and the player had to reclaim it (apex-v022-carrot-sym50-opus-1 4:48,
    /// the user's repeated pattern; K-eco-a-factory-with-a-spot-in-its-lane-gets-an-extractor-there).
    pub(super) fn own_lane_keep_out(&self, def_id: UnitDefId, own: &[OwnUnit], facing: i32) -> Vec<Lane> {
        let Some((half_width, half_depth)) = self.factory_half_size(def_id) else { return Vec::new() };
        let standing = own.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos);
        let ordered = self.pianist.as_ref().map(|p| p.tasks.values()).into_iter().flatten().filter_map(|t| match t {
            super::pianist::Task::Build { def, near, .. } if self.world.def(*def).is_some_and(|d| d.speed == 0.0) => Some(*near),
            _ => None,
        });
        let spots = self.world.hello.metal_spots.iter().copied();
        lanes_behind_points(standing.chain(ordered).chain(spots), facing, half_width, half_depth)
    }

    /// Where a new factory of this type must not stand for its exit lane, with this facing, to run over ground its
    /// units cannot walk or off the map: the lane mirrored behind every unwalkable cell within `radius` of `anchor`
    /// and behind every cell past the map's edge (bluegecko-3v1-comet-catcher-4: three of five factories had their
    /// lane on cliffs, the user saw the units choking in one; player-29: an Advanced Vehicle Plant with its front
    /// five elmos from the south edge, seven units stuck in it from 16:51 on).
    pub(super) fn blocked_lane_keep_out(&self, def_id: UnitDefId, anchor: Vec3, radius: f32, facing: i32) -> Vec<Lane> {
        let Some((half_width, half_depth)) = self.factory_half_size(def_id) else { return Vec::new() };
        let def = self.world.def(def_id).expect("a factory type has a definition");
        // A shipyard's units leave into the water the engine placed it on: its lane is the engine's business.
        if def.build_options.iter().any(|b| self.world.def(*b).is_some_and(|d| d.move_class.is_some_and(|mc| mc.kind == bot_protocol::MoveKind::Ship))) {
            return Vec::new();
        }
        let Some(passable) = self.passable_for_factory(def_id) else { return Vec::new() };
        let terrain = &self.world.hello.terrain;
        lanes_behind_blocked(passable, terrain.width as usize, terrain.height as usize, terrain.cell, half_width, half_depth, anchor, radius, facing)
    }

    /// Whether a factory's lane leaves the ground its units can walk: off the map, or over cells the factory's
    /// strictest movement class cannot cross. The wake names it (player-29, 17:22: "exit lane is blocked ... nothing
    /// of ours stands in the lane" for a lane off the map's edge, and the player reclaimed the two stuck Lugers).
    pub(super) fn lane_ground(&self, factory: &OwnUnit, lane: &Lane) -> Option<&'static str> {
        let map = &self.world.hello.map;
        let off_map = |p: Vec3| p.x < 0.0 || p.z < 0.0 || p.x > map.width || p.z > map.height;
        // The lane's far end and its middle: a lane that reaches the edge is as good as one across it.
        let middle = Vec3 { x: (lane.from.x + lane.to.x) / 2.0, y: 0.0, z: (lane.from.z + lane.to.z) / 2.0 };
        if off_map(lane.to) || off_map(middle) {
            return Some("runs off the map's edge");
        }
        let terrain = &self.world.hello.terrain;
        let passable = self.passable_for_factory(factory.def)?;
        let cell_at = |p: Vec3| {
            let (i, j) = ((p.x / terrain.cell) as usize, (p.z / terrain.cell) as usize);
            (i < terrain.width as usize && j < terrain.height as usize).then(|| passable[j * terrain.width as usize + i])
        };
        if cell_at(lane.to) == Some(false) || cell_at(middle) == Some(false) {
            return Some("runs over ground its units cannot walk");
        }
        None
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

    /// Every tick: the lanes of our factories, standing or being built (a site chosen now must stay clear of a lane
    /// that will be), and which of our mobile units cannot move.
    pub(super) fn track_yards(&mut self, tick: &Tick) {
        let own = &tick.snapshot.own_units;
        self.lanes = own.iter().filter_map(|u| self.lane_of(u)).collect();
        // A factory not yet placed faces the way the shim tries first (`facings_toward_the_enemy`).
        let pending: Vec<Lane> = self.pending_factories().into_iter().filter_map(|(def, at)| self.lane_at(def, at, self.facings_toward_the_enemy(at)[0])).collect();
        self.lanes.extend(pending);
        self.stuck.retain(|id, s| own.iter().any(|u| u.id == *id && u.pos.dist2d(s.at) < STUCK_FREE));
        // Mobile units standing still inside a standing factory's lane (a builder working from the pad blocks the plant
        // as surely as a stuck unit; wake-3: nothing left the plant for three minutes and nobody was told).
        let standing_lanes: Vec<(UnitId, Lane)> = own.iter().filter_map(|u| (!u.being_built).then(|| self.lane_of(u).map(|l| (u.id, l))).flatten()).collect();
        let before = std::mem::take(&mut self.lane_standers);
        // A unit still being built on the pad is not standing in the lane (Cape Violet, 1:31: a constructor at 99 %
        // woke the player as blocking its own plant).
        for u in own.iter().filter(|u| !u.being_built && self.world.def(u.def).is_some_and(|d| d.speed > 0.0)) {
            if let Some((factory, _)) = standing_lanes.iter().find(|(_, l)| l.contains(u.pos)) {
                let since = match before.get(&u.id) {
                    Some((s, at, _)) if at.dist2d(u.pos) < 8.0 => *s,
                    _ => tick.frame,
                };
                let at = before.get(&u.id).filter(|(_, at, _)| at.dist2d(u.pos) < 8.0).map_or(u.pos, |(_, at, _)| *at);
                self.lane_standers.insert(u.id, (since, at, *factory));
            }
        }
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
            let mut stuck = self.stuck_in_lane(&lane, own);
            let standers: Vec<(UnitId, i32)> = self.lane_standers.iter().filter(|(id, (since, _, f))| *f == factory && tick.frame - since >= YARD_WAKE_FRAMES && !stuck.iter().any(|(s, _)| s == *id)).map(|(id, (since, _, _))| (*id, *since)).collect();
            stuck.extend(standers);
            let longest = stuck.iter().map(|(_, since)| tick.frame - since).max().unwrap_or(0);
            if longest < YARD_WAKE_FRAMES {
                self.yard_warned.remove(&factory);
                continue;
            }
            if self.yard_warned.insert(factory) {
                let name = self.actor_name(factory);
                let blockers = self.lane_blockers(&lane, own, factory);
                let names: Vec<String> = stuck.iter().filter_map(|(id, _)| own.iter().find(|u| u.id == *id)).map(|u| self.handle(u)).collect();
                let ground = own.iter().find(|u| u.id == factory).and_then(|f| self.lane_ground(f, &lane));
                let cause = match (ground, blockers.is_empty()) {
                    (Some(ground), _) => format!("; its exit lane {ground}: its units cannot leave it, and only removing {name} frees them"),
                    (None, true) => "; nothing of ours stands in the lane".to_string(),
                    (None, false) => format!("; in the lane: {} (the remove tool takes them away)", blockers.iter().map(|u| self.handle(u)).collect::<Vec<_>>().join(", ")),
                };
                let text = format!(
                    "{name}'s exit lane is blocked: {} of ours ({}) have stood in it for {} (stuck, or standing there working){cause}",
                    stuck.len(),
                    names.join(", "),
                    super::pianist::clock(longest),
                );
                self.trigger("yard", tick.frame, text);
            }
        }
    }

    /// A mobile unit standing still in a factory's exit lane: since when, and which factory (H-ECO-YARD-LANE).
    pub(super) fn lane_stander(&self, unit: UnitId, frame: i32) -> Option<(i32, UnitId)> {
        self.lane_standers.get(&unit).filter(|(since, _, _)| frame - since >= 10 * super::FRAMES_PER_SECOND).map(|(since, _, f)| (*since, *f))
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

    /// The unit of ours a `remove` target or a `reclaim` step names: a handle (`unit_by_handle`), or "<type> at
    /// <place or cell>" (`armwin at spot_64`, `armsolar at E4`): the nearest standing unit of that type to the place
    /// within `AT_PLACE`, or any in the cell (the impedance tally, 2026-10-16: a wind turbine penned a commander
    /// for nineteen minutes, and the picture names a building's handle only in a factory's `yard` line).
    pub(super) fn unit_named<'a>(&self, name: &str, own: &'a [OwnUnit]) -> Option<&'a OwnUnit> {
        if let Some(unit) = self.unit_by_handle(name, own) {
            return Some(unit);
        }
        // "unfinished <type> at <place or cell>": a frame of the type there, not a standing one.
        let (name, unfinished) = match name.strip_prefix("unfinished ") {
            Some(rest) => (rest.trim(), true),
            None => (name, false),
        };
        let (kind, at) = name.split_once(" at ")?;
        let def = self.world.def_named(kind.trim())?;
        let at = at.trim();
        let standing = own.iter().filter(|u| u.def == def && u.being_built == unfinished);
        let cell = at.to_ascii_uppercase();
        if cell.len() == 2 && cell.starts_with(|c: char| ('A'..='H').contains(&c)) && cell.ends_with(|c: char| ('1'..='8').contains(&c)) {
            return standing.filter(|u| self.world.grid(u.pos) == cell).min_by_key(|u| u.id.0);
        }
        let place = self.pianist.as_ref()?.places.iter().find(|p| p.name == at)?.at;
        standing.filter(|u| u.pos.dist2d(place) <= AT_PLACE).min_by(|a, b| a.pos.dist2d(place).total_cmp(&b.pos.dist2d(place)))
    }

    /// The unit of ours a handle names: `armsolar_31002`, or an actor's name (commander, constructor_N, plant_N).
    pub(super) fn unit_by_handle<'a>(&self, handle: &str, own: &'a [OwnUnit]) -> Option<&'a OwnUnit> {
        // `commander` has no underscore: the split must not end the lookup (2v1-hard_aggressive: every lookup of the
        // commander by handle returned None, so its first list never skipped the default's builds and a `stop` for it did nothing).
        if handle == self.commander_handle() {
            return own.iter().find(|u| self.world.is_commander_def(u.def));
        }
        // `constructor_N`, `lab_N`, `plant_N`: the unit's id; a name with another seat's tag or a bare word is nobody's.
        let id = handle.rsplit_once('_').and_then(|(_, n)| n.parse::<i32>().ok());
        id.and_then(|id| own.iter().find(|u| u.id.0 == id))
    }
}

/// A "<type> at <place>" name reaches this far from the place.
const AT_PLACE: f32 = 600.0;

/// The engine's facing as a unit vector in the ground plane: 0 south (+z), 1 east (+x), 2 north (-z), 3 west (-x).
pub(super) fn facing_dir(facing: i32) -> (f32, f32) {
    match facing.rem_euclid(4) {
        0 => (0.0, 1.0),
        1 => (1.0, 0.0),
        2 => (0.0, -1.0),
        _ => (-1.0, 0.0),
    }
}

/// The strip of sites from which a factory of this size, with this facing, would have `at` in its exit lane: the
/// lane mirrored behind the point.
fn lane_behind(at: Vec3, facing: i32, half_width: f32, half_depth: f32) -> Lane {
    let (fx, fz) = facing_dir(facing);
    let reach = half_depth + LANE_DEPTH;
    Lane {
        from: Vec3 { x: at.x - fx * half_depth, y: 0.0, z: at.z - fz * half_depth },
        to: Vec3 { x: at.x - fx * reach, y: 0.0, z: at.z - fz * reach },
        half_width: half_width + LANE_MARGIN,
    }
}

/// The keep-out strips for a factory with this facing whose lane must cover none of `points`: each point's lane
/// mirrored behind it.
pub(super) fn lanes_behind_points(points: impl Iterator<Item = Vec3>, facing: i32, half_width: f32, half_depth: f32) -> Vec<Lane> {
    points.map(|at| lane_behind(at, facing, half_width, half_depth)).collect()
}

/// The keep-out strips for a factory with this facing whose lane must not cross a blocked cell: for each blocked
/// cell within `radius` of `anchor` (sampled every other cell each way, a cell being 16 elmos), the strip of sites
/// whose lane would cover it, which is the lane mirrored behind the cell. Cells past the map's edge are blocked:
/// a unit sent off the map stands at the edge.
pub(super) fn lanes_behind_blocked(passable: &[bool], width: usize, height: usize, cell: f32, half_width: f32, half_depth: f32, anchor: Vec3, radius: f32, facing: i32) -> Vec<Lane> {
    let stride = 2i64;
    let (x0, x1) = (((anchor.x - radius) / cell).floor() as i64, ((anchor.x + radius) / cell).ceil() as i64);
    let (z0, z1) = (((anchor.z - radius) / cell).floor() as i64, ((anchor.z + radius) / cell).ceil() as i64);
    let blocked_cell = |i: i64, j: i64| i < 0 || j < 0 || i >= width as i64 || j >= height as i64 || !passable[j as usize * width + i as usize];
    let mut lanes = Vec::new();
    let mut j = z0;
    while j < z1 {
        let mut i = x0;
        while i < x1 {
            let blocked = (0..stride).any(|dj| (0..stride).any(|di| blocked_cell(i + di, j + dj)));
            if blocked {
                let at = Vec3 { x: (i as f32 + 1.0) * cell, y: 0.0, z: (j as f32 + 1.0) * cell };
                if at.dist2d(anchor) <= radius {
                    lanes.push(lane_behind(at, facing, half_width, half_depth));
                }
            }
            i += stride;
        }
        j += stride;
    }
    lanes
}

#[cfg(test)]
mod lane_ground_tests {
    use super::*;

    #[test]
    fn a_cliff_south_of_a_site_keeps_the_factory_off_it() {
        // A 64-cell square map, cell 16: a cliff band across rows 40-41 (z 640-672).
        let (w, h, cell) = (64usize, 64usize, 16.0);
        let passable: Vec<bool> = (0..w * h).map(|k| !(40..42).contains(&(k / w))).collect();
        let anchor = Vec3 { x: 512.0, y: 0.0, z: 400.0 };
        let lanes = lanes_behind_blocked(&passable, w, h, cell, 48.0, 48.0, anchor, 600.0, 0);
        assert!(!lanes.is_empty());
        let banned = |x: f32, z: f32| lanes.iter().any(|l| l.contains(Vec3 { x, y: 0.0, z }));
        // A site whose lane (48 deep to the front, 320 past it) reaches the cliff at z 640: from z 272 up to 592.
        assert!(banned(512.0, 400.0), "a site 240 north of the cliff has its lane on it");
        assert!(banned(512.0, 580.0));
        assert!(!banned(512.0, 150.0), "a site 490 north of the cliff is clear (the strip ends are rounded by the half width)");
        assert!(!banned(512.0, 60.0));
        // Facing north, the same cliff bans the sites south of it and frees the ones north of it.
        let lanes = lanes_behind_blocked(&passable, w, h, cell, 48.0, 48.0, Vec3 { x: 512.0, y: 0.0, z: 900.0 }, 600.0, 2);
        let banned = |x: f32, z: f32| lanes.iter().any(|l| l.contains(Vec3 { x, y: 0.0, z }));
        assert!(banned(512.0, 900.0), "a site 228 south of the cliff, facing north, has its lane on it");
        assert!(!banned(512.0, 400.0), "a site north of the cliff, facing north, is clear of it");
    }

    #[test]
    fn the_map_edge_keeps_a_factory_from_facing_it() {
        // Player-29: an 18x18 Advanced Vehicle Plant (half size 72) at z 6067 on a 6144-deep map, facing south, its
        // front five elmos from the edge. A 64-cell map here, cell 16, 1024 deep, all walkable.
        let (w, h, cell) = (64usize, 64usize, 16.0);
        let passable = vec![true; w * h];
        let anchor = Vec3 { x: 512.0, y: 0.0, z: 947.0 };
        let banned_facing = |facing: i32| {
            let lanes = lanes_behind_blocked(&passable, w, h, cell, 72.0, 72.0, anchor, 1000.0, facing);
            lanes.iter().any(|l| l.contains(anchor))
        };
        assert!(banned_facing(0), "facing the south edge from 77 elmos: the lane runs off the map");
        assert!(banned_facing(1) && banned_facing(3), "facing along the edge from 77 elmos: the lane's margin hangs over it, as beside a cliff");
        assert!(!banned_facing(2), "facing north: the lane runs up the map");
        // Further in, the lanes along the edge clear it: the margin is 120 (half the width plus 48). The site sits
        // well west too: a lane's rounded far end (392 out, 120 round) would touch the east edge from x 512.
        let inner = Vec3 { x: 400.0, y: 0.0, z: 880.0 };
        let lanes = lanes_behind_blocked(&passable, w, h, cell, 72.0, 72.0, inner, 1000.0, 1);
        assert!(!lanes.iter().any(|l| l.contains(inner)), "facing east 144 elmos from the edge is clear");
    }
}

#[cfg(test)]
mod named_tests {
    use super::*;
    use crate::brain::pianist::fixtures::{at, brain_of, own};

    /// "<type> at <cell or place>" names a standing unit of that type: any in the cell (the lowest id), the nearest
    /// within 600 of a place; a handle still names its unit.
    #[test]
    fn a_unit_is_named_by_type_at_a_cell_or_a_place() {
        let mut brain = brain_of(&["armflash", "armwar", "armllt"]);
        let own = vec![own(300, 3, at(4500.0, 2600.0)), own(301, 3, at(4600.0, 2650.0)), own(302, 2, at(4550.0, 2620.0)), own(303, 3, at(100.0, 100.0))];
        let mut pianist = crate::brain::pianist::Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.places.push(crate::brain::pianist::picture::Place { name: "spot_40".into(), at: at(4000.0, 2600.0), spot: Some(40), cells: None });
        brain.pianist = Some(pianist);
        let id = |u: Option<&OwnUnit>| u.map(|u| u.id.0);
        assert_eq!(id(brain.unit_named("armllt at E4", &own)), Some(300));
        assert_eq!(id(brain.unit_named("armllt at e4", &own)), Some(300));
        assert_eq!(id(brain.unit_named("armwar at E4", &own)), Some(302));
        assert_eq!(id(brain.unit_named("armllt at spot_40", &own)), Some(300), "the nearest within 600");
        assert_eq!(id(brain.unit_named("armwar at spot_40", &own)), Some(302));
        assert_eq!(id(brain.unit_named("armflash at E4", &own)), None);
        assert_eq!(id(brain.unit_named("armllt at Z9", &own)), None);
        assert_eq!(id(brain.unit_named("armllt at A1", &own)), Some(303));
        assert_eq!(id(brain.unit_named("armllt_301", &own)), Some(301));
        // "unfinished <type> at <cell or place>" names a frame of the type there, never a standing one.
        let with_frame: Vec<OwnUnit> = own.iter().cloned().chain([OwnUnit { being_built: true, health: 40.0, max_health: 100.0, build_progress: 0.4, ..own[0].clone() }].into_iter().map(|mut u| { u.id = UnitId(310); u })).collect();
        assert_eq!(id(brain.unit_named("unfinished armllt at E4", &with_frame)), Some(310));
        assert_eq!(id(brain.unit_named("unfinished armllt at spot_40", &with_frame)), Some(310));
        assert_eq!(id(brain.unit_named("armllt at E4", &with_frame)), Some(300), "the plain form keeps to standing units");
        assert_eq!(id(brain.unit_named("unfinished armwar at E4", &with_frame)), None);
    }
}

#[cfg(test)]
mod spot_lane_tests {
    use super::*;

    #[test]
    fn a_metal_spot_in_front_of_a_site_keeps_the_factory_from_facing_it() {
        // apex-v022-carrot-sym50-opus-1: the first vehicle plant at (2542, 7360) facing east, a spot at (2672, 7320)
        // 130 east of its centre; the extractor built there sealed the plant and the player reclaimed it at 4:48.
        let site = Vec3 { x: 2542.0, y: 0.0, z: 7360.0 };
        let spot = Vec3 { x: 2672.0, y: 0.0, z: 7320.0 };
        let banned = |at: Vec3, facing: i32| lanes_behind_points([spot].into_iter(), facing, 28.0, 28.0).iter().any(|l| l.contains(at));
        assert!(banned(site, 1), "facing east, the spot is in the lane");
        assert!(!banned(site, 0) && !banned(site, 2) && !banned(site, 3), "the other three facings keep the lane off the spot");
        // A site 240 north of the spot's line faces east freely: the lane's half width is 28 plus the 48 margin.
        assert!(!banned(Vec3 { x: 2542.0, y: 0.0, z: 7120.0 }, 1));
        // And one 530 west of the spot: the lane runs 320 past the front and its far end is rounded by the 76 half
        // width, so a site 400 west still has the spot at the lane's end; 530 west is beyond it.
        assert!(banned(Vec3 { x: 2272.0, y: 0.0, z: 7320.0 }, 1));
        assert!(!banned(Vec3 { x: 2142.0, y: 0.0, z: 7320.0 }, 1));
    }
}
