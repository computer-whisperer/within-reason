//! The block (`layout::lay_block`; `docs/design/2026-10-29-base-layout.md` §1.1, step 3): the one global decision
//! after the stand. When our first advanced plant is placed (ordered or standing) the block's centre is found once, at
//! the player's `block` mark or on the flattest ground beside the plant that holds two frame rows and the gantry's
//! slot; from then on the layer (`economy.rs` `layout_world`) carries its reservations (the rows for frames, the slot
//! for the gantry) and its sub-lattice for construction turrets (96 apart while sparse, 48 once the player says
//! `dense` through the `block` tool), and the bands of a turret and a frame are centred on it. Banks standing on its
//! ground are listed in the report for reclaim-for-ground; the player decides (the `remove` tool).

use bot_protocol::{OwnUnit, UnitDefId, Vec3};
use layout::{Block, BlockWants, DENSE_PITCH, Facts, SPARSE_PITCH};

use super::pianist::glossary;
use super::pianist::picture::clock;
use super::roster::Kit;
use super::Brain;

/// How far the player's `block` mark may stand from the block's centre before the block is laid again at the mark.
const MARK_MOVED: f32 = 96.0;
/// When no ground held a block, the search runs again after this many frames (30 s).
const RETRY_FRAMES: i32 = 900;

impl Brain {
    /// Lays the block once an advanced plant of ours is placed, or lays it again at the player's `block` mark when
    /// the mark has moved; keeps its pitch at the player's word. Said in the hands' lines.
    pub(super) fn tend_block(&mut self, own: &[OwnUnit], kit: &Kit, frame: i32) {
        let dense = self.strategist.as_ref().is_some_and(|s| s.block_dense.load(std::sync::atomic::Ordering::Relaxed));
        let mark = self.strategist.as_ref().and_then(|s| s.marks_all().get("block").map(|m| Vec3 { x: m.x, y: 0.0, z: m.z }));
        if let Some(block) = self.block.as_mut() {
            block.pitch = if dense { DENSE_PITCH } else { SPARSE_PITCH };
            match mark {
                Some(m) if block.centre.dist2d(m) > MARK_MOVED => {}
                _ => return,
            }
        } else if frame < self.block_tried + RETRY_FRAMES {
            return;
        }
        let Some(plant) = self.advanced_plant(own) else { return };
        self.block_tried = frame;
        let Some(wants) = self.block_wants(plant, kit) else { return };
        let laid = {
            let world = self.layout_world_without_block(own);
            layout::lay_block(&world, &wants, mark)
        };
        match laid {
            Ok(mut block) => {
                block.pitch = if dense { DENSE_PITCH } else { SPARSE_PITCH };
                let words = self.block_laid_words(&block, &wants, own);
                eprintln!("[ai {}] f={frame} {words}", self.ai());
                if let Some(pianist) = self.pianist.as_mut() {
                    pianist.done.push(format!("{} {words}", clock(frame)));
                }
                self.block = Some(block);
                self.block_failed = None;
            }
            Err(why) => {
                let said = format!("{why} ({}); mark `block` where it should stand", wants.plant.name);
                if self.block_failed.as_deref() != Some(&said) {
                    eprintln!("[ai {}] f={frame} {said}", self.ai());
                }
                self.block_failed = Some(said);
            }
        }
    }

    /// Our first advanced plant, standing or ordered: a factory of tier 2 or above by the glossary.
    fn advanced_plant(&self, own: &[OwnUnit]) -> Option<(UnitDefId, Vec3, i32, String)> {
        let advanced = |def: UnitDefId| self.world.is_factory_def(def) && glossary::entry(self.name(def)).is_some_and(|e| e.tier >= 2);
        if let Some(u) = own.iter().filter(|u| advanced(u.def)).min_by_key(|u| u.id.0) {
            return Some((u.def, u.pos, u.facing, self.actor_name(u.id)));
        }
        let pianist = self.pianist.as_ref()?;
        pianist.tasks.values().chain(pianist.queued_tasks().map(|(_, t)| t)).find_map(|t| match t {
            super::pianist::Task::Build { def, near, facing, .. } if advanced(*def) => Some((*def, *near, *facing, format!("an ordered {}", self.name(*def)))),
            _ => None,
        })
    }

    /// What the block is laid for: the kit's turret, the largest frame the advanced constructor builds (the advanced
    /// fusion), the gantry when the advanced constructor builds a factory larger than the plant, and the plant.
    fn block_wants(&self, plant: (UnitDefId, Vec3, i32, String), kit: &Kit) -> Option<BlockWants> {
        let (def, at, facing, name) = plant;
        // The construction turret (the roster's `nano`), not the kit's laser tower (block-1: the LLT's footprint of 4
        // put the centre off the turret's grid and its reach of 128 stood for the block's).
        let turret = self.facts_of(self.world.def_named(kit.roster.role("nano")?)?)?;
        let plant_facts = self.facts_of(def)?;
        let builds = self.world.def(kit.advanced_constructor).map(|d| d.build_options.clone()).unwrap_or_default();
        let on_water = |d: UnitDefId| glossary::entry(self.name(d)).is_some_and(|e| e.has_flag("on_water"));
        let frame: Facts = builds
            .iter()
            .filter(|d| !self.world.is_factory_def(**d) && !on_water(**d))
            .filter(|d| self.world.def(**d).is_some_and(|u| u.speed == 0.0 && u.energy_make > 0.0))
            .filter_map(|d| self.facts_of(*d))
            .max_by_key(|f| (f.footprint.x * f.footprint.z, f.build_time as i64))?;
        let gantry = builds
            .iter()
            .filter(|d| self.world.is_factory_def(**d) && !on_water(**d))
            .filter_map(|d| self.facts_of(*d))
            .filter(|f| f.footprint.x.max(f.footprint.z) > plant_facts.footprint.x.max(plant_facts.footprint.z))
            .max_by_key(|f| f.footprint.x.max(f.footprint.z));
        let plant = self.plant_of(def, at, facing, true, name)?;
        Some(BlockWants { turret, frame, gantry, plant })
    }

    /// The hands' line when the block is laid.
    fn block_laid_words(&self, block: &Block, wants: &BlockWants, own: &[OwnUnit]) -> String {
        let gantry = match &block.gantry {
            Some((g, _, facing, _)) => format!(" and a {}'s slot facing {}", self.name(g.def), ["south", "east", "north", "west"][facing.rem_euclid(4) as usize]),
            None => String::new(),
        };
        let where_ = if block.marked { "at your mark".to_string() } else { format!("{:.0} from {}", block.from_plant, wants.plant.name) };
        let ground = self.on_block_ground(block, own);
        let reclaim = if ground.is_empty() { String::new() } else { format!("; its ground holds {}: reclaim them as the block's slots reach them", ground.join(", ")) };
        format!(
            "the block for the construction turrets and the frames at ({:.0}, {:.0}) {where_}: two rows of {} {} slots{gantry}; turrets {:.0} apart until you say dense{reclaim}",
            block.centre.x,
            block.centre.z,
            block.frame_slots.len() / 2,
            self.name(wants.frame.def),
            block.pitch
        )
    }

    /// Our finished banks (generators, converters) standing on the block's reserved ground, by handle.
    fn on_block_ground(&self, block: &Block, own: &[OwnUnit]) -> Vec<String> {
        own.iter()
            .filter(|u| !u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0) && matches!(self.purpose_of(u.def), layout::Purpose::Generator { .. }))
            .filter(|u| self.facts_of(u.def).is_some_and(|f| block.holds(&f.footprint.faced(u.facing).rect(u.pos))))
            .map(|u| self.handle(u))
            .collect()
    }

    /// The report's block line: where it stands, its pitch, what it holds and what stands on its ground; or why
    /// there is none when an advanced plant is placed and no ground held one.
    pub(super) fn block_line(&self, own: &[OwnUnit]) -> Option<String> {
        let Some(block) = self.block.as_ref() else {
            return self.block_failed.clone().map(|why| format!("the block (where your construction turrets and frames stand): {why}"));
        };
        let turrets = own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def) && block.lattice().holds(u.pos)).count();
        let standing: Vec<layout::Rect> = own.iter().filter_map(|u| self.facts_of(u.def).filter(|f| f.building).map(|f| f.footprint.faced(u.facing).rect(u.pos))).collect();
        let frame_fp = self.frame_footprint_of(block);
        let free = block.free_frame_slots(&standing, frame_fp);
        let pitch = if block.pitch <= DENSE_PITCH { "dense (turrets touching, 48 apart)".to_string() } else { format!("sparse (turrets {:.0} apart; the `block` tool's dense packs them at 48)", block.pitch) };
        let gantry = match &block.gantry {
            Some((g, at, _, _)) => format!(", a {}'s slot at {}", self.name(g.def), self.world.grid(*at)),
            None => String::new(),
        };
        let ground = self.on_block_ground(block, own);
        let reclaim = if ground.is_empty() { String::new() } else { format!("; on its ground: {} (reclaim them as the block's slots reach them: `remove`)", ground.join(", ")) };
        let placed = if block.marked { "at your `block` mark".to_string() } else { "beside the advanced plant; a `block` mark moves it".to_string() };
        Some(format!(
            "the block (where your construction turrets and frames stand): centre ({:.0}, {:.0}) at {}, {placed}; {pitch}; {turrets} turret{} on it, {free} of {} frame slots free{gantry}{reclaim}",
            block.centre.x,
            block.centre.z,
            self.world.grid(block.centre),
            if turrets == 1 { "" } else { "s" },
            block.frame_slots.len()
        ))
    }

    /// The footprint the block's frame slots were laid for: the row's depth less the slack, square.
    fn frame_footprint_of(&self, block: &Block) -> layout::Footprint {
        let side = ((block.frame_rows[0].x1 - block.frame_rows[0].x0).min(block.frame_rows[0].z1 - block.frame_rows[0].z0) / 8.0) as i32 - 6;
        layout::Footprint { x: side.max(2), z: side.max(2) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::pianist::fixtures::{brain_of, own};
    use crate::brain::roster::{Kit, ROSTERS};
    use bot_protocol::Terrain;

    /// An advanced plant standing lays the block beside it: its line is said, the report has it, a turret's band and a
    /// frame's band centre on it, and a turret's site is on its sub-lattice.
    #[test]
    fn the_block_is_laid_at_the_advanced_plant_and_the_bands_follow_it() {
        use crate::brain::pianist::Pianist;
        let mut brain = brain_of(&["armcom", "armnanotc", "armavp", "armack", "armafus", "armshltx", "armsolar", "armmex", "armmoho", "armckfus"]);
        let (com, nano, avp, ack, afus, gantry, solar) = (UnitDefId(1), UnitDefId(2), UnitDefId(3), UnitDefId(4), UnitDefId(5), UnitDefId(6), UnitDefId(7));
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].build_distance = 145.0;
            defs[0].build_speed = 300.0;
            defs[0].build_options = vec![nano, avp, solar];
            defs[1].footprint = (6, 6);
            defs[1].build_speed = 200.0;
            defs[1].build_distance = 400.0;
            defs[2].footprint = (18, 18);
            defs[2].build_options = vec![ack];
            defs[3].speed = 30.0;
            defs[3].build_speed = 300.0;
            defs[3].build_options = vec![afus, gantry, UnitDefId(10)];
            defs[4].footprint = (12, 12);
            defs[4].energy_make = 3000.0;
            defs[4].metal_cost = 9700.0;
            defs[5].footprint = (24, 24);
            defs[5].build_options = vec![com];
            defs[6].footprint = (10, 10);
            defs[6].energy_make = 20.0;
            // A fusion as wide as the advanced fusion but shallower: the rows are laid for the larger.
            defs[9].footprint = (12, 10);
            defs[9].energy_make = 750.0;
            defs[4].build_time = 313_000.0;
            defs[9].build_time = 65_000.0;
            let (w, h) = (512u32, 384u32);
            brain.world.hello.terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() };
        }
        brain.home = Vec3 { x: 2000.0, y: 0.0, z: 2000.0 };
        brain.pianist = Some(Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist"));
        let kit = Kit { commander: com, extractor: UnitDefId(8), converter: solar, lab: avp, turret: solar, constructor: ack, vehicle_constructor: ack, advanced_constructor: ack, advanced_extractor: UnitDefId(9), raider: com, line: com, resurrector: com, roster: &ROSTERS[0] };
        let commander = own(10, 1, Vec3 { x: 2008.0, y: 0.0, z: 2200.0 });
        let plant = OwnUnit { facing: 1, ..own(11, 3, Vec3 { x: 2000.0, y: 0.0, z: 2000.0 }) };
        let ours = vec![commander.clone(), plant.clone()];
        brain.tend_block(&ours, &kit, 100);
        let block = brain.block.clone().expect("a block");
        assert!(block.gantry.is_some(), "the gantry's slot: {block:?}");
        assert_eq!(block.pitch, SPARSE_PITCH);
        assert_eq!(block.reach, 400.0, "the construction turret's reach, not the kit's turret's");
        assert!(brain.pianist.as_ref().unwrap().done.iter().any(|d| d.contains("armafus slots")), "{:?}", brain.pianist.as_ref().unwrap().done);
        assert!(brain.pianist.as_ref().unwrap().done.iter().any(|d| d.contains("the block for the construction turrets")), "{:?}", brain.pianist.as_ref().unwrap().done);
        let line = brain.block_line(&ours).expect("a line");
        assert!(line.contains("6 of 6 frame slots free") && line.contains("sparse"), "{line}");
        // The bands: a turret's and a frame's are the block's.
        assert_eq!(brain.band_for(nano, &commander, &ours), (block.centre, block.reach));
        assert_eq!(brain.band_for(afus, &commander, &ours), (block.centre, block.reach));
        // A turret's site is on the sub-lattice, off the rows.
        let plan = brain.place_planned(nano, &commander, &ours);
        let super::super::economy::Plan::Slot(_, at, _) = plan else { panic!("a slot") };
        assert!(block.lattice().holds(at), "{at:?} off the lattice at {:?}", block.centre);
        assert!(!block.holds(&layout::Footprint { x: 6, z: 6 }.rect(at)));
        // Dense on the player's word.
        brain.strategist = Some(std::sync::Arc::new(crate::strategist::shared::Shared::default()));
        brain.strategist.as_ref().unwrap().block_dense.store(true, std::sync::atomic::Ordering::Relaxed);
        brain.tend_block(&ours, &kit, 200);
        assert_eq!(brain.block.as_ref().unwrap().pitch, DENSE_PITCH);
        // A plant of tier 1 lays no block.
        let mut fresh = brain_of(&["armcom", "armvp"]);
        fresh.world.hello.unit_defs[1].build_options = vec![com];
        let plant = own(11, 2, Vec3 { x: 2000.0, y: 0.0, z: 2000.0 });
        fresh.tend_block(std::slice::from_ref(&plant), &kit, 100);
        assert!(fresh.block.is_none());
    }
}
