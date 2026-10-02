//! Sight over the metal spots (H-SCOUT-SPOTS): when each was last within a unit of ours' sight, for the picture's
//! staleness words and the rove goals. And sight over the ground (H-SCOUT-GLANCE,
//! `docs/design/2026-10-01-scouting-glance.md`): when each tile of the map was last within a unit of ours' sight,
//! rolled up to the named cells for the player's report: how much of a cell was seen, how long ago, and what of
//! his stood there. Sight here is a circle; the engine's is cut by terrain, and another seat's units are not counted. The route-making raider (H-SCOUT-ROUTE) went with the heuristic
//! bot (`docs/design/2026-09-25-one-decider.md`): scouting is the player's to order.

use bot_protocol::{Tick, UnitDefId, Vec3};

use super::{Brain, FRAMES_PER_SECOND};
use crate::strategist::shared::CellLook;

/// Sight of a unit type the simulator's table does not know.
const DEFAULT_SIGHT: f32 = 350.0;
/// Spots are surveyed against every unit's sight this often.
const SURVEY_FRAMES: i32 = FRAMES_PER_SECOND / 2;

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
        let sights: Vec<(Vec3, f32)> = tick.snapshot.own_units.iter().map(|u| (u.pos, self.sight_of(u.def))).collect();
        for (index, spot) in spots.iter().enumerate() {
            if sights.iter().any(|(pos, sight)| pos.dist2d(*spot) <= *sight) {
                self.spots.seen[index] = Some(tick.frame);
            }
        }
        if self.spots.ground.seen.is_empty() {
            self.spots.ground = Ground::sized(self.world.hello.map.width, self.world.hello.map.height);
        }
        for (pos, sight) in &sights {
            self.spots.ground.mark(*pos, *sight, tick.frame);
        }
    }

    /// H-SCOUT-GLANCE: every named cell of the map as units of ours last saw it, columns A to H and rows 1 to 8
    /// in order: how much of it, how long ago, his buildings remembered there, and its metal spots.
    pub(super) fn scouting(&self, frame: i32) -> Vec<CellLook> {
        let boxes: Vec<&bot_protocol::StartBox> = self.world.hello.start_boxes.iter().filter(|b| b.ally_team != self.world.hello.ally_team).collect();
        let (width, height) = (self.world.hello.map.width, self.world.hello.map.height);
        let mut cells = Vec::with_capacity(CELLS * CELLS);
        for column in 0..CELLS {
            for row in 0..CELLS {
                let centre = Vec3 { x: (column as f32 + 0.5) * width / CELLS as f32, y: 0.0, z: (row as f32 + 0.5) * height / CELLS as f32 };
                let cell = self.world.grid(centre);
                let (seen_share, at) = self.spots.ground.cell(column, row);
                let mut his: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                for (def, pos, _) in self.enemy_buildings.values() {
                    if self.world.grid(*pos) == cell {
                        *his.entry(self.name(*def).to_string()).or_default() += 1;
                    }
                }
                cells.push(CellLook {
                    seen_share,
                    ago: at.map(|at| (frame - at) / FRAMES_PER_SECOND),
                    his: his.into_iter().collect(),
                    his_commander: self.enemy_commander_seen.is_some_and(|(pos, _)| self.world.grid(pos) == cell),
                    in_his_box: boxes.iter().any(|b| b.contains(centre)),
                    spots: self.world.hello.metal_spots.iter().enumerate().filter(|(_, s)| self.world.grid(**s) == cell).map(|(i, _)| i).collect(),
                    cell,
                });
            }
        }
        cells
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
}
