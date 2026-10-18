//! Sight over the metal spots (H-SCOUT-SPOTS): when each was last within a unit of ours' sight, for the picture's
//! staleness words and the rove goals. And sight over the ground (H-SCOUT-GLANCE,
//! `docs/design/2026-10-01-scouting-glance.md`): when each tile of the map was last within a unit of ours' sight,
//! rolled up to the named cells for the player's report: how much of a cell was seen, how long ago, and what of
//! his stood there. Sight is the engine's own coverage grid when the shim sends it (`Snapshot::sensors`, once a
//! second: cut by terrain, the whole side's eyes; `docs/design/2026-10-18-sight-jammers-cloak-shields.md`), else a
//! circle round each unit of ours (a shim before 2026-10-18), which a hill does not cut. The radar grid says which
//! quiet cells our radar covers and which are dark (H-REPORT-RADAR-WORDS). The route-making raider (H-SCOUT-ROUTE) went with the heuristic
//! bot (`docs/design/2026-09-25-one-decider.md`): scouting is the player's to order.

use bot_protocol::{Grid, Tick, UnitDefId, Vec3};

use super::{Brain, FRAMES_PER_SECOND};
use crate::strategist::shared::CellLook;

/// Sight of a unit type the simulator's table does not know.
const DEFAULT_SIGHT: f32 = 350.0;
/// Spots are surveyed against every unit's sight this often.
const SURVEY_FRAMES: i32 = FRAMES_PER_SECOND / 2;
/// The engine's grids count as current for this long after the shim sent them (it sends every second).
const SENSORS_FRESH_FRAMES: i32 = 3 * FRAMES_PER_SECOND;

/// The side of a tile of the ground record, in elmos.
const TILE: f32 = 128.0;
/// The named cells across and down the map (`World::grid`).
const CELLS: usize = 8;

#[derive(Default)]
pub struct Spots {
    /// Per metal spot (the hello's order), the last frame it was within an own unit's sight.
    seen: Vec<Option<i32>>,
    last_survey: i32,
    ground: Ground,
}

/// The map in tiles, each with the last frame its centre was within sight of a unit of ours.
#[derive(Default)]
pub struct Ground {
    width: f32,
    height: f32,
    cols: usize,
    seen: Vec<Option<i32>>,
}

impl Ground {
    pub fn sized(width: f32, height: f32) -> Ground {
        let (cols, rows) = ((width / TILE).ceil().max(1.0) as usize, (height / TILE).ceil().max(1.0) as usize);
        Ground { width, height, cols, seen: vec![None; cols * rows] }
    }

    fn centre(&self, tile: usize) -> Vec3 {
        Vec3 { x: ((tile % self.cols) as f32 + 0.5) * TILE, y: 0.0, z: ((tile / self.cols) as f32 + 0.5) * TILE }
    }

    /// Every tile whose centre is within `sight` of `pos` is seen at `frame`.
    pub fn mark(&mut self, pos: Vec3, sight: f32, frame: i32) {
        let rows = self.seen.len() / self.cols.max(1);
        let span = |value: f32, count: usize| (((value - sight) / TILE).floor().max(0.0) as usize, (((value + sight) / TILE).ceil() as usize).min(count));
        let ((c0, c1), (r0, r1)) = (span(pos.x, self.cols), span(pos.z, rows));
        for row in r0..r1 {
            for col in c0..c1 {
                let tile = row * self.cols + col;
                if self.centre(tile).dist2d(pos) <= sight {
                    self.seen[tile] = Some(frame);
                }
            }
        }
    }

    /// Every tile whose centre the engine's coverage grid covers is seen at `frame`: the side's true sight, cut by
    /// terrain.
    pub fn mark_grid(&mut self, grid: &Grid, frame: i32) {
        for tile in 0..self.seen.len() {
            let centre = self.centre(tile);
            if grid.at(centre.x, centre.z) {
                self.seen[tile] = Some(frame);
            }
        }
    }

    /// The share of one named cell's tiles whose centre `covered` holds for (a coverage grid, a jammer's ring).
    pub fn cell_share(&self, column: usize, row: usize, covered: impl Fn(Vec3) -> bool) -> f32 {
        let of = |value: f32, extent: f32| ((value / extent * CELLS as f32) as usize).min(CELLS - 1);
        let (mut tiles, mut hits) = (0usize, 0usize);
        for tile in 0..self.seen.len() {
            let centre = self.centre(tile);
            if of(centre.x, self.width) == column && of(centre.z, self.height) == row {
                tiles += 1;
                hits += usize::from(covered(centre));
            }
        }
        hits as f32 / tiles.max(1) as f32
    }

    /// One named cell (column and row from 0): the share of its tiles ever seen, and the frame the middle one of
    /// those was last in sight (the age of the look: most of what was seen was seen about then).
    pub fn cell(&self, column: usize, row: usize) -> (f32, Option<i32>) {
        let of = |value: f32, extent: f32| ((value / extent * CELLS as f32) as usize).min(CELLS - 1);
        let mut frames: Vec<i32> = Vec::new();
        let mut tiles = 0;
        for tile in 0..self.seen.len() {
            let centre = self.centre(tile);
            if of(centre.x, self.width) == column && of(centre.z, self.height) == row {
                tiles += 1;
                frames.extend(self.seen[tile]);
            }
        }
        frames.sort_unstable();
        (frames.len() as f32 / tiles.max(1) as f32, frames.get(frames.len() / 2).copied())
    }
}

impl Brain {
    /// Sight of a unit type over flat ground: the engine's figure when the shim sends it, else the simulator's table,
    /// else a default.
    pub(super) fn sight_of(&self, def: UnitDefId) -> f32 {
        if let Some(sight) = self.world.def(def).map(|d| d.sight).filter(|s| *s > 0.0) {
            return sight;
        }
        self.sim.defs.get(&def).map_or(DEFAULT_SIGHT, |&index| self.sim.rules.units.list[index].sight)
    }

    /// The engine's coverage grids when the shim sent them within the last three seconds; None from an old shim or
    /// before the first second, when sight is the circle.
    pub(super) fn sensors_fresh(&self, frame: i32) -> Option<&bot_protocol::Sensors> {
        self.sensors.as_ref().filter(|(_, at)| frame - at <= SENSORS_FRESH_FRAMES).map(|(s, _)| s)
    }

    /// Whether a unit of his, by id, is of a type that cloaks (hidden where it stands when out of sight, not gone).
    pub(super) fn quarry_cloaks(&self, id: bot_protocol::UnitId) -> bool {
        self.enemy_soldiers.get(&id).or_else(|| self.enemy_buildings.get(&id)).and_then(|(d, ..)| self.world.def(*d)).is_some_and(|d| d.cloak.is_some())
    }

    /// His jammer towers seen standing (H-REPORT-JAMMERS): the newest sighting first, each with where it stands, the
    /// radius it hides from our radar and when it was last seen.
    pub(super) fn jammer_lines(&self, frame: i32) -> Vec<String> {
        let mut jammers: Vec<(i32, String)> = self
            .enemy_buildings
            .iter()
            .filter(|(id, (def, _, _))| !self.enemy_unfinished.contains(id) && self.world.def(*def).is_some_and(|d| d.jam_range > 0.0))
            .map(|(_, (def, pos, seen))| {
                let jam = self.world.def(*def).map_or(0.0, |d| d.jam_range);
                (*seen, format!("{} at {} ({}, {}) hides {jam:.0} around it from radar (seen {} ago)", self.name(*def), self.world.grid(*pos), pos.x as i32, pos.z as i32, super::pianist::picture::clock(frame - seen)))
            })
            .collect();
        jammers.sort_by_key(|(seen, _)| std::cmp::Reverse(*seen));
        jammers.into_iter().map(|(_, line)| line).collect()
    }

    /// His jammer rings: each standing jammer's place and the radius it hides from radar.
    fn jammer_rings(&self) -> Vec<(Vec3, f32)> {
        self.enemy_buildings
            .iter()
            .filter(|(id, _)| !self.enemy_unfinished.contains(id))
            .filter_map(|(_, (def, pos, _))| self.world.def(*def).filter(|d| d.jam_range > 0.0).map(|d| (*pos, d.jam_range)))
            .collect()
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
        if self.spots.ground.seen.is_empty() {
            self.spots.ground = Ground::sized(self.world.hello.map.width, self.world.hello.map.height);
        }
        // The engine's sight grid when it is current: the side's eyes, cut by terrain. Else the circle round each
        // unit of ours, as before 2026-10-18.
        if let Some(sensors) = self.sensors_fresh(tick.frame) {
            let los = sensors.los.clone();
            for (index, spot) in spots.iter().enumerate() {
                if los.at(spot.x, spot.z) {
                    self.spots.seen[index] = Some(tick.frame);
                }
            }
            self.spots.ground.mark_grid(&los, tick.frame);
            return;
        }
        let sights: Vec<(Vec3, f32)> = tick.snapshot.own_units.iter().map(|u| (u.pos, self.sight_of(u.def))).collect();
        for (index, spot) in spots.iter().enumerate() {
            if sights.iter().any(|(pos, sight)| pos.dist2d(*spot) <= *sight) {
                self.spots.seen[index] = Some(tick.frame);
            }
        }
        for (pos, sight) in &sights {
            self.spots.ground.mark(*pos, *sight, tick.frame);
        }
    }

    /// H-SCOUT-GLANCE: every named cell of the map as units of ours last saw it, columns A to H and rows 1 to 8
    /// in order: how much of it, how long ago, his buildings remembered there (a type that cloaks marked so), its
    /// metal spots; and, with the engine's grids, how much of it our radar covers now, the radar contacts on it,
    /// whether a seen jammer of his hides it, and the fire from out of sight laid to it.
    pub(super) fn scouting(&self, tick: &Tick) -> Vec<CellLook> {
        let frame = tick.frame;
        let boxes: Vec<&bot_protocol::StartBox> = self.world.hello.start_boxes.iter().filter(|b| b.ally_team != self.world.hello.ally_team).collect();
        let (width, height) = (self.world.hello.map.width, self.world.hello.map.height);
        let sensors = self.sensors_fresh(frame);
        let rings = self.jammer_rings();
        let contacts: Vec<Vec3> = tick.snapshot.enemies.iter().filter(|e| e.def.is_none()).map(|e| e.pos).collect();
        let mut cells = Vec::with_capacity(CELLS * CELLS);
        for column in 0..CELLS {
            for row in 0..CELLS {
                let centre = Vec3 { x: (column as f32 + 0.5) * width / CELLS as f32, y: 0.0, z: (row as f32 + 0.5) * height / CELLS as f32 };
                let cell = self.world.grid(centre);
                let (seen_share, at) = self.spots.ground.cell(column, row);
                let mut his: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                for (def, pos, _) in self.enemy_buildings.values() {
                    if self.world.grid(*pos) == cell {
                        let cloaks = if self.world.def(*def).is_some_and(|d| d.cloak.is_some()) { " (cloaks)" } else { "" };
                        *his.entry(format!("{}{cloaks}", self.name(*def))).or_default() += 1;
                    }
                }
                cells.push(CellLook {
                    seen_share,
                    ago: at.map(|at| (frame - at) / FRAMES_PER_SECOND),
                    his: his.into_iter().collect(),
                    his_commander: self.enemy_commander_seen.is_some_and(|(pos, _)| self.world.grid(pos) == cell),
                    in_his_box: boxes.iter().any(|b| b.contains(centre)),
                    spots: self.world.hello.metal_spots.iter().enumerate().filter(|(_, s)| self.world.grid(**s) == cell).map(|(i, _)| i).collect(),
                    radar_share: sensors.map(|s| self.spots.ground.cell_share(column, row, |p| s.radar.at(p.x, p.z))),
                    contacts: contacts.iter().filter(|p| self.world.grid(**p) == cell).count(),
                    jammed: rings.iter().any(|(at, jam)| at.dist2d(centre) <= *jam),
                    guns: self.guns_in(&cell),
                    cell,
                });
            }
        }
        cells
    }

    /// The `guns_N` places laid to a cell (H-GUNS): filled by `guns.rs`; none until it is built.
    fn guns_in(&self, _cell: &str) -> Vec<String> {
        Vec::new()
    }

    /// The frame a spot was last within an own unit's sight; `None` when never.
    pub(super) fn spot_seen(&self, index: usize) -> Option<i32> {
        self.spots.seen.get(index).copied().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::Ground;
    use bot_protocol::Vec3;

    /// A unit's sight marks the tiles around it; a cell's look is the share of its tiles seen and the frame of the
    /// middle one. On Comet Catcher (8192 by 6144) a cell is 8 by 6 tiles.
    #[test]
    fn a_units_sight_marks_the_ground_and_a_cell_rolls_it_up() {
        let mut ground = Ground::sized(8192.0, 6144.0);
        assert_eq!(ground.cell(6, 0), (0.0, None));
        // A Rover (sight 585) in the middle of G1 (6144..7168, 0..768) at frame 900: most of the cell, and part of
        // the cells beside it.
        ground.mark(Vec3 { x: 6656.0, y: 0.0, z: 384.0 }, 585.0, 900);
        let (share, at) = ground.cell(6, 0);
        assert!(share > 0.9, "{share}");
        assert_eq!(at, Some(900));
        let (beside, _) = ground.cell(5, 0);
        assert!(beside > 0.0 && beside < 0.2, "{beside}");
        assert_eq!(ground.cell(0, 7), (0.0, None));
        // A later look at one corner of the cell leaves the look's age with the middle tile.
        ground.mark(Vec3 { x: 6200.0, y: 0.0, z: 60.0 }, 200.0, 5000);
        assert_eq!(ground.cell(6, 0).1, Some(900));
        // A point off the map's edge marks nothing out of bounds.
        ground.mark(Vec3 { x: 8190.0, y: 0.0, z: 6140.0 }, 400.0, 100);
        assert!(ground.cell(7, 7).0 > 0.0);
    }

    /// The engine's grid marks the ground where it says we see, hills and all: a grid covering the west half of row
    /// 1 sees A1 to D1 whole and nothing of E1 to H1, whatever units stand where; and a cell's radar share is the
    /// grid's share of its tiles.
    #[test]
    fn the_engines_grid_marks_the_ground_and_a_cells_share_of_it() {
        let mut ground = Ground::sized(8192.0, 8192.0);
        // 64-elmo squares, 128 across: the west half of the first 16 rows (z < 1024, x < 4096) covered.
        let grid = bot_protocol::Grid::pack(128, 128, 64.0, (0..128 * 128).map(|i| i / 128 < 16 && i % 128 < 64));
        ground.mark_grid(&grid, 3000);
        assert_eq!(ground.cell(0, 0), (1.0, Some(3000)));
        assert_eq!(ground.cell(3, 0), (1.0, Some(3000)));
        assert_eq!(ground.cell(4, 0), (0.0, None), "E1 is behind the hill");
        assert_eq!(ground.cell(0, 1), (0.0, None));
        assert_eq!(ground.cell_share(2, 0, |p| grid.at(p.x, p.z)), 1.0);
        assert_eq!(ground.cell_share(5, 3, |p| grid.at(p.x, p.z)), 0.0);
    }
}
