//! D-BLOCK: the block and its farm (`layout::lay_block`, `layout::Farm`; the flagship's since the base layout arc's
//! step 3 and the farm-holds design, `docs/design/2026-11-08-farm-holds.md` §1.3; here since 2026-11-09,
//! `docs/design/2026-11-09-distilled-layout.md`). When our first advanced plant stands or is going up the block's
//! centre is found once: the ground nearest the back field that holds a footing of two rows of the largest reactor
//! (the layer's marked laying, the back field the mark: where the bot's reactors went before, behind home). The
//! layer's unmarked laying, the flattest ground within 2,400 of the plant, has a player to judge it in the
//! flagship; here it laid the block 2,300 from home in two of four games on Flooded Valley, out of reach of every
//! turret, and the first reactors went up there alone (dist328). From then on the layer's world carries the block's sub-lattice for construction turrets (96 apart, so a
//! dead turret takes no neighbour and the reactors have room between them), and the band of a reactor, an advanced
//! converter and a ring's turret is the farm's: the block's centre out to the farthest turret's reach. Nothing is
//! reserved for the farm: it is the ground the turrets reach, a frame goes where the most build power reaches it
//! and a turret where it shares the most free ground with the turrets standing, so the farm grows as one piece
//! (`crates/layout/src/farm.rs`). Before the block is laid, and where no ground holds one, the bands are the back
//! field's and the frame's own (`siting.rs` `band_for`). `block=off` plays without it.

use bot_protocol::{OwnUnit, Tick, Vec3};
use layout::{BlockWants, FARM_EXTENT, ROOM_LOW, SPARSE_PITCH};

use crate::{Distilled, SECOND};

/// When no ground held a block, the search runs again after this long.
const RETRY: i32 = 30 * SECOND;
/// The block's plant is gone when no factory of ours stands within this of where it stood.
const PLANT_MOVED: f32 = 64.0;

impl Distilled {
    /// Lays the block at the back field once an advanced plant of ours stands or is being built (not on an order:
    /// the engine may refuse the order's slot), and again when the plant it was laid for is gone.
    pub(crate) fn tend_block(&mut self, tick: &Tick) {
        if !self.options.block {
            return;
        }
        let factory = |u: &OwnUnit| self.def(u.def).is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty());
        if let Some(block) = &self.block {
            if self.own.values().any(|u| factory(u) && u.pos.dist2d(block.plant_at) < PLANT_MOVED) {
                return;
            }
            eprintln!("[distilled ai={}] f={} block: its plant at ({:.0},{:.0}) is gone", self.hello.ai_id, self.frame, block.plant_at.x, block.plant_at.z);
            self.block = None;
        }
        if self.frame < self.block_tried + RETRY {
            return;
        }
        let Some(plant) = self.own.values().filter(|u| Some(u.def) == self.kit.advanced_plant).min_by_key(|u| u.id.0) else { return };
        self.block_tried = self.frame;
        // What it is laid for: the construction turret, the largest reactor of the kit, the plant. The bot builds
        // no factory larger than the advanced plant, so no gantry's slot.
        let wants = (|| {
            let turret = self.facts_of(self.kit.turret?)?;
            let frame = self.facts_of(self.kit.advanced_fusion.or(self.kit.fusion)?)?;
            let plant = self.plant_of(plant.def, plant.pos, plant.facing, true, format!("our {}", self.name_of(plant.def)))?;
            Some(BlockWants { turret, frame, gantry: None, plant })
        })();
        let Some(wants) = wants else { return };
        let laid = {
            let world = self.layout_world(tick, None);
            layout::lay_block(&world, &wants, Some(self.back_field()))
        };
        match laid {
            Ok(mut block) => {
                block.pitch = SPARSE_PITCH;
                eprintln!(
                    "[distilled ai={}] f={} block: laid at ({:.0},{:.0}), {:.0} from the back field, {:.0} from {} at ({:.0},{:.0}), {:.0} from home; turrets {:.0} apart, reach {:.0}",
                    self.hello.ai_id,
                    self.frame,
                    block.centre.x,
                    block.centre.z,
                    block.from_plant,
                    block.centre.dist2d(block.plant_at),
                    wants.plant.name,
                    block.plant_at.x,
                    block.plant_at.z,
                    block.centre.dist2d(self.home),
                    block.pitch,
                    block.reach
                );
                self.block = Some(block);
                self.block_refused = None;
            }
            // Said when the reason changes: the search runs every `RETRY` (dist336, Hooked: 83 lines in four games).
            Err(why) => {
                if self.block_refused.as_deref() != Some(why.as_str()) {
                    eprintln!("[distilled ai={}] f={} block: none: {why}", self.hello.ai_id, self.frame);
                }
                self.block_refused = Some(why);
            }
        }
    }

    /// The farm's construction turrets: ours finished, within FARM_EXTENT of the block's centre.
    fn farm_turrets<'a>(&'a self, block: &'a layout::Block) -> impl Iterator<Item = &'a OwnUnit> {
        self.finished(self.kit.turret).filter(|t| t.pos.dist2d(block.centre) <= FARM_EXTENT)
    }

    /// The farm's band (the flagship's `farm_band`): the block's centre, and a radius that covers the farthest of
    /// its construction turrets and that turret's reach, never under the block's own reach, so a turret can grow
    /// the farm at its edge and a frame can stand wherever the turrets reach.
    pub(crate) fn farm_band(&self, block: &layout::Block) -> (Vec3, f32) {
        let farthest = self.farm_turrets(block).map(|t| t.pos.dist2d(block.centre)).fold(0.0f32, f32::max);
        (block.centre, block.reach.max(farthest + block.reach))
    }

    /// The yard: where the reactors and advanced converters go up, the block's centre once it is laid, the back
    /// field before.
    pub(crate) fn yard(&self) -> Vec3 {
        self.block.as_ref().map_or_else(|| self.back_field(), |b| b.centre)
    }

    /// The minute line's farm: the turrets on it and the room in reach of ROOM_LOW or more of them, counted in the
    /// block's frame (the advanced fusion); `-` before the block is laid.
    pub(crate) fn farm_word(&self, tick: &Tick) -> String {
        let Some(block) = &self.block else { return "-".to_string() };
        let turrets = self.farm_turrets(block).count();
        let world = self.layout_world(tick, None);
        let room = world.farm(block.reach, &block.frame).map_or(0, |farm| layout::farm_footprints(farm.room(ROOM_LOW), block.frame.footprint));
        format!("{turrets}t/{room}")
    }
}
