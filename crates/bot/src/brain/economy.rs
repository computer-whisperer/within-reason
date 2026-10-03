//! Where a building goes: the site for what the pianist's builders are told to build (`place_planned`,
//! `build_site_for`, the extractor's exact site). The rules that chose what to build went with the heuristic bot
//! (2026-09-25, `docs/design/2026-09-25-one-decider.md`).

use bot_protocol::{BuildSite, Lane, OwnUnit, Placement, UnitDefId, Vec3};

use super::roster::Kit;
use super::Brain;

pub(super) const EARLY_COMMANDER_LEASH: f32 = 1500.0;
const NANO_REACH: f32 = 260.0;
/// A metal extractor this close to a spot occupies it.
/// An extractor this close to a spot's centre stands on that spot, at least; the map's extractor radius when larger,
/// since an extractor may be built anywhere within it (`extractor_site`; rush-20: offset extractors were not seen as
/// occupying their spot, which was claimed and ordered again and again, 3 extractors built by minute 3 for 12 orders).
/// Gaps between base buildings, in 8-elmo build squares. Three squares made a maze the army could not leave.
const BUILDING_GAP: i32 = 5;
const LAB_GAP: i32 = 8;
/// Distances from the start point along the line to the enemy: generators behind, labs ahead, turrets beyond them.
const BACK_FIELD: f32 = 150.0;
/// A metal-map square is 16 elmos; an extractor offset is tried in steps of `MEX_STEP`.
const MEX_SQUARE: f32 = 16.0;
const MEX_STEP: f32 = 8.0;
/// How far off its centre an extractor still counts as refused for that spot (the dropped-order fallback).
pub(super) const MEX_PATCH: f32 = 130.0;
/// How far around a site the engine refused for a building the next search for that type keeps away.
const REFUSED_SITE_RADIUS: f32 = 64.0;
const LAB_YARD: f32 = 350.0;
/// How far a building's anchor keeps from anything of ours standing or started, so the engine's closest free site
/// to it stays within the builder's reach: the lab's gap (LAB_GAP squares) plus half of it and a neighbour, and a
/// small building's gap plus a little.
const LAB_CLEARANCE: f32 = 230.0;
const BUILDING_CLEARANCE: f32 = 90.0;
/// No building goes up this close to a metal spot's centre: a lab anchored 102 from a spot left the engine no site for
/// the extractor (human-1, the user's "it builds the lab right on top of it"; a lab is 96 wide, an extractor 48). The
/// lab keeps farther: the engine's site search moved one 80 from an anchor that was 156 from the spot (human-1, second
/// game: the lab landed 98 from spot_10 again).
const SPOT_CLEARANCE: f32 = 150.0;
const LAB_SPOT_CLEARANCE: f32 = 260.0;
const TURRET_LINE: f32 = 650.0;
/// A builder farther than this from home puts a generator in the back field at home, not beside itself.
const GENERATOR_HOME: f32 = 1000.0;
/// No orders before this frame: the engine loses them.
pub(super) const FIRST_ORDER_FRAME: i32 = 60;
/// A builder is not judged idle for this long after an order: the order has to reach it first.
pub(super) const ORDER_GRACE_FRAMES: i32 = 45;

/// What one builder should do next.
pub(super) enum Plan {
    Extractor(Vec3),
    /// A building placed near `anchor`.
    Near(UnitDefId, Vec3),
    /// A building placed right beside `anchor`: a construction turret has to reach the factory it helps.
    Beside(UnitDefId, Vec3),
}

impl Brain {
    pub(super) fn place_planned(&self, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit], kit: &Kit) -> Plan {
        let lab = own.iter().filter(|u| kit.is_factory(u.def)).min_by(|a, b| a.pos.dist2d(builder.pos).total_cmp(&b.pos.dist2d(builder.pos)));
        // By what the definition is (a factory, a defence, a construction turret), so every building the game has
        // places (docs/design/2026-09-22-full-roster.md, decision 3).
        let info = self.world.def(def);
        let static_builder = info.is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty());
        let on_water = super::pianist::glossary::entry(self.name(def)).is_some_and(|e| e.has_flag("on_water"));
        match def {
            // A sea building stands on the water nearest the builder (the menu offers it only with water in reach).
            d if on_water => match self.world.nearest_water(builder.pos, info.map_or(100.0, |b| b.build_distance) + 300.0) {
                Some(water) => Plan::Near(d, water),
                None => Plan::Near(d, builder.pos),
            },
            // The yard, but no farther from the builder than its reach: the commander built the lab at its feet in
            // both experienced players' replays and never took a step for it (rush-2-ab: ours walked 280 for it).
            d if self.world.is_factory_def(d) => Plan::Near(d, self.beside_builder(builder, self.forward_of_home(LAB_YARD), own, LAB_CLEARANCE, LAB_SPOT_CLEARANCE)),
            d if info.is_some_and(|i| i.speed == 0.0 && i.weapon_count > 0) => Plan::Near(d, self.forward_of_home(TURRET_LINE)),
            d if static_builder && lab.is_some() => Plan::Beside(d, lab.unwrap().pos),
            d if static_builder => Plan::Near(d, self.forward_of_home(-BACK_FIELD)),
            // A generator goes home, to the back field behind the plants, when its builder is out on the map: beside
            // the builder it stood on the frontier and died there (player-10-routes; the user, 2026-09-28: "we tend
            // to build solars etc on the front lines rather than back at base"). At home, beside the builder as before.
            d if info.is_some_and(|i| i.energy_make > 0.0 || i.energy_upkeep < 0.0 || i.wind_cap > 0.0) && builder.pos.dist2d(self.home) > GENERATOR_HOME => Plan::Near(d, self.forward_of_home(-BACK_FIELD)),
            // As the simulator places them: beside the builder wherever it stands, no walking (queue-smoke: a planned
            // solar went to the back field 497 elmos from a commander out at a far extractor, and the plan's timing
            // with it).
            d => Plan::Beside(d, self.beside_builder(builder, self.enemy_base(builder.pos), own, BUILDING_CLEARANCE, SPOT_CLEARANCE)),
        }
    }

    /// Where a building goes up beside `builder` without it taking a step: the engine's site search does not count
    /// units as in the way, so a site asked for at the builder's own position is its own position, and the game then
    /// has the builder walk off, turn round and come back (five seconds a building, rush-2-ab). A point one reach
    /// away toward `toward`, so the base grows that way, turned round the builder in 45-degree steps until nothing
    /// of ours stands or is started within `clearance` of it: the engine's closest free site to an anchor on the first
    /// extractor's nanoframe lay beyond the extractor, out of reach, and the commander walked for the lab
    /// (rush-7-comet-std-noplace 09).
    fn beside_builder(&self, builder: &OwnUnit, toward: Vec3, own: &[OwnUnit], clearance: f32, spot_clearance: f32) -> Vec3 {
        let reach = self.world.def(builder.def).map_or(100.0, |d| d.build_distance.max(60.0));
        let heading = (toward.z - builder.pos.z).atan2(toward.x - builder.pos.x);
        let at = |turn: f32| {
            let angle = heading + turn.to_radians();
            Vec3 { x: builder.pos.x + angle.cos() * reach, y: 0.0, z: builder.pos.z + angle.sin() * reach }
        };
        let standing = |p: Vec3| own.iter().any(|u| u.id != builder.id && self.world.def(u.def).is_some_and(|d| d.speed == 0.0) && u.pos.dist2d(p) < clearance);
        let on_spot = |p: Vec3| self.world.hello.metal_spots.iter().any(|s| s.dist2d(p) < spot_clearance);
        let started = |p: Vec3| {
            self.pianist.as_ref().is_some_and(|pianist| {
                pianist.tasks.iter().any(|(id, task)| *id != builder.id && matches!(task, super::pianist::Task::Build { near, .. } if near.dist2d(p) < clearance))
            })
        };
        [0.0, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0]
            .into_iter()
            .map(at)
            .find(|p| !standing(*p) && !started(*p) && !on_spot(*p))
            .unwrap_or_else(|| at(0.0))
    }

    pub(super) fn extractor_site(&self, spot: Vec3, builder: &OwnUnit) -> Vec3 {
        let hello = &self.world.hello;
        let Some(index) = hello.metal_spots.iter().position(|s| s.dist2d(spot) < 1.0) else { return spot };
        let squares = hello.metal_spot_squares.get(index).map_or(&[][..], |s| s.as_slice());
        if self.centre_only.contains(&index) || squares.is_empty() {
            return spot;
        }
        let reach = self.world.def(builder.def).map_or(100.0, |d| d.build_distance.max(60.0));
        let d = spot.dist2d(builder.pos);
        let wanted = (d - reach + 8.0).max(0.0);
        if wanted <= 0.0 || d <= 0.0 {
            return spot;
        }
        // The engine draws only the squares whose centres lie within the extractor radius of the extractor
        // (ExtractorBuilding.cpp), which is stricter than what the game allows to be built (cmd_mex_denier.lua: the
        // radius plus one metal square of every square of the patch). So every square of the patch stays within the
        // radius, less a square for the build grid's snap: player-52's extractors stood 101 off centre at the median
        // under the game's rule and drew 1.9 a second from patches worth 2.3. Tried from the offset wanted down to
        // nothing.
        let allowed = hello.map.extractor_radius - MEX_SQUARE;
        let valid = |p: Vec3| squares.iter().all(|(sx, sz)| (p.x - sx).hypot(p.z - sz) < allowed);
        let mut offset = wanted.min(hello.map.extractor_radius);
        while offset > 0.0 {
            let p = Vec3 { x: spot.x + (builder.pos.x - spot.x) / d * offset, y: spot.y, z: spot.z + (builder.pos.z - spot.z) / d * offset };
            if valid(p) {
                return p;
            }
            offset -= MEX_STEP;
        }
        spot
    }

    /// What a plan builds and where the engine is asked to put it. None when the builder cannot build it.
    pub(super) fn build_site_for(&self, plan: &Plan, unit: &OwnUnit, own: &[OwnUnit], kit: &Kit) -> Option<(UnitDefId, BuildSite)> {
        let planned_def = match plan {
            Plan::Extractor(_) => kit.extractor,
            Plan::Near(def_id, _) | Plan::Beside(def_id, _) => *def_id,
        };
        if !self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&planned_def)) {
            return None;
        }
        // Nothing but an extractor goes in a factory's exit lane (yards.rs), and a site the engine refused for this
        // type is not asked again while the refusal stands (a point lane).
        let mut keep_out = self.lanes.clone();
        if let Some(pianist) = self.pianist.as_ref() {
            keep_out.extend(pianist.refused_sites.iter().filter(|(def, _, _)| *def == planned_def).map(|(_, at, _)| Lane { from: *at, to: *at, half_width: REFUSED_SITE_RADIUS }));
        }
        // A factory is offered every facing, the one toward the enemy first; each facing's own exit lane must be
        // clear of what stands or is ordered (the mirrored lanes), of ground its units cannot walk and of the map's
        // edge. Anything else stands the engine's default way.
        let placements = |anchor: Vec3| -> Vec<Placement> {
            if !self.world.is_factory_def(planned_def) {
                return vec![Placement { facing: 0, keep_out: keep_out.clone() }];
            }
            self.facings_toward_the_enemy(anchor)
                .into_iter()
                .map(|facing| {
                    let mut keep_out = keep_out.clone();
                    keep_out.extend(self.own_lane_keep_out(planned_def, own, facing));
                    keep_out.extend(self.blocked_lane_keep_out(planned_def, anchor, 1000.0 + 2.0 * super::yards::LANE_DEPTH, facing));
                    Placement { facing, keep_out }
                })
                .collect()
        };
        Some(match *plan {
            // The game rejects an extractor that is not exactly on its spot (cmd_mex_denier.lua), and the shim
            // places extractors exactly at `near`, searching nowhere.
            Plan::Extractor(spot) => (kit.extractor, BuildSite { near: self.extractor_site(spot, unit), search_radius: 0.0, min_dist: 0, placements: Vec::new() }),
            // A tier-2 extractor over one of ours goes exactly where ours stands: the game upgrades in place and
            // refuses any other position on a held spot (low-1: three `armmoho spot_N` orders at the spots' centres,
            // 97-104 elmos from our extractors placed off centre, all "site bad"; the player: "upgrade path not working").
            Plan::Near(def_id, anchor) | Plan::Beside(def_id, anchor)
                if self.world.def(def_id).is_some_and(|d| d.extracts_metal > 0.0)
                    && let Some(ours) = own.iter().filter(|u| kit.is_extractor(u.def) && u.def != def_id && u.pos.dist2d(anchor) < self.spot_occupied_radius()).min_by(|a, b| a.pos.dist2d(anchor).total_cmp(&b.pos.dist2d(anchor))) =>
            {
                (def_id, BuildSite { near: ours.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new() })
            }
            Plan::Near(def_id, anchor) => (def_id, BuildSite { near: anchor, search_radius: 1000.0, min_dist: self.gap_around(def_id, kit), placements: placements(anchor) }),
            Plan::Beside(def_id, anchor) => (def_id, BuildSite { near: anchor, search_radius: NANO_REACH, min_dist: 2, placements: placements(anchor) }),
        })
    }

    /// The gap, in build squares, a new building keeps from its neighbours: wide enough for units to walk through.
    fn gap_around(&self, def: UnitDefId, _kit: &Kit) -> i32 {
        // Any factory: the hover platform and the aircraft plant have exit lanes like the lab's.
        if self.world.is_factory_def(def) { LAB_GAP } else { BUILDING_GAP }
    }

    pub(super) fn spot_occupied_radius(&self) -> f32 {
        self.world.hello.map.spot_radius()
    }

}
