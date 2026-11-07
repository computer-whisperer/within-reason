//! Factories' exit lanes and units that cannot move (docs/design/2026-09-22-yard-and-reclaim.md). In hands-2-bulldogs
//! the player's solar collectors at its marked place sealed the Advanced Vehicle Plant's front: five Bulls stood in the
//! yard five to seven minutes each, the plant built nothing for five minutes with metal and energy full, and the
//! picture said nothing of it though the engine raised 156 move-failed events on the Bulls.

use bot_protocol::{Event, Lane, OwnUnit, UnitId, Vec3, UnitDefId};

use super::{Brain, Tick};

/// A stuck unit is free again once it has moved this far from where its move failed.
pub(super) const STUCK_FREE: f32 = 80.0;
/// A yard with units stuck in it this long wakes the player.
const YARD_WAKE_FRAMES: i32 = 30 * super::FRAMES_PER_SECOND;

/// A unit of ours that could not move: when the engine first said so, and where it stood.
pub(super) struct Stuck {
    pub since: i32,
    pub at: Vec3,
}

impl Brain {
    /// The apron a factory's finished units leave through (`layout::APRON` off its front face, as wide as the
    /// footprint plus its widest product a side; docs/design/2026-10-29-base-layout.md §1.2). None for anything
    /// else. The street beyond it is the layout layer's (`economy.rs` `street_words`). Measured from the centre, as it was until
    /// pace-1, the strip's rounded start reached behind the factory too: a constructor wedged between a solar and
    /// the plant's back wall counted as standing in the lane, and three "exit lane is blocked" wakes named a lane
    /// that was free.
    pub(super) fn lane_of(&self, factory: &OwnUnit) -> Option<Lane> {
        self.lane_at(factory.def, factory.pos, factory.facing)
    }

    /// The exit lane a factory of this type would have standing at `pos` with this facing, so a factory not yet
    /// standing has a lane too.
    pub(super) fn lane_at(&self, def_id: UnitDefId, pos: Vec3, facing: i32) -> Option<Lane> {
        if !self.world.is_factory_def(def_id) {
            return None;
        }
        let f = self.facts_of(def_id)?;
        Some(layout::lane(f.footprint, pos, facing, layout::APRON, f.product_half))
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

    /// Every tick: the lanes of our factories, standing or being built (a site chosen now must stay clear of a lane
    /// that will be), and which of our mobile units cannot move.
    pub(super) fn track_yards(&mut self, tick: &Tick) {
        let own = &tick.snapshot.own_units;
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
                let factory_unit = own.iter().find(|u| u.id == factory);
                let ground = factory_unit.and_then(|f| self.lane_ground(f, &lane));
                // The lane clear but the street beyond it shut: the street's state names the opener (1794060468-human:
                // seventeen Fatboys in a pocket past the lane, "nothing of ours stands in the lane" three times).
                let street = factory_unit.and_then(|f| self.street_words(f, own)).filter(|w| !w.starts_with("open"));
                let cause = match (ground, blockers.is_empty(), street) {
                    (Some(ground), _, _) => format!("; its exit lane {ground}: its units cannot leave it, and only removing {name} frees them"),
                    (None, true, Some(street)) => format!("; nothing of ours stands in the lane, but its street to the open is {street}"),
                    (None, true, None) => "; nothing of ours stands in the lane".to_string(),
                    (None, false, _) => format!("; in the lane: {} (the remove tool takes them away)", blockers.iter().map(|u| self.handle(u)).collect::<Vec<_>>().join(", ")),
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
        if let Some(cell) = self.world.cells.parse(at) {
            return standing.filter(|u| self.world.cells.cell_of(u.pos) == cell).min_by_key(|u| u.id.0);
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
        assert_eq!(id(brain.unit_named("armllt at E3", &own)), Some(300));
        assert_eq!(id(brain.unit_named("armllt at e3", &own)), Some(300));
        assert_eq!(id(brain.unit_named("armwar at E3", &own)), Some(302));
        assert_eq!(id(brain.unit_named("armllt at spot_40", &own)), Some(300), "the nearest within 600");
        assert_eq!(id(brain.unit_named("armwar at spot_40", &own)), Some(302));
        assert_eq!(id(brain.unit_named("armflash at E3", &own)), None);
        assert_eq!(id(brain.unit_named("armllt at Z9", &own)), None);
        assert_eq!(id(brain.unit_named("armllt at A1", &own)), Some(303));
        assert_eq!(id(brain.unit_named("armllt_301", &own)), Some(301));
        // "unfinished <type> at <cell or place>" names a frame of the type there, never a standing one.
        let with_frame: Vec<OwnUnit> = own.iter().cloned().chain([OwnUnit { being_built: true, health: 40.0, max_health: 100.0, build_progress: 0.4, ..own[0].clone() }].into_iter().map(|mut u| { u.id = UnitId(310); u })).collect();
        assert_eq!(id(brain.unit_named("unfinished armllt at E3", &with_frame)), Some(310));
        assert_eq!(id(brain.unit_named("unfinished armllt at spot_40", &with_frame)), Some(310));
        assert_eq!(id(brain.unit_named("armllt at E3", &with_frame)), Some(300), "the plain form keeps to standing units");
        assert_eq!(id(brain.unit_named("unfinished armwar at E4", &with_frame)), None);
    }
}
