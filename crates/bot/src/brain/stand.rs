//! The commander's stand (`layout::stand`; `docs/design/2026-10-29-base-layout.md` §1.1, step 1): when the
//! commander's list first plays, the opening's slots are solved once from where it stands, the spots and the terrain,
//! so that the list's leading steps go to the engine as one queue (`lists.rs` `play_lists`) at exact sites
//! (`economy.rs` `place_planned`, `Plan::Slot`) and the commander never idles between its first structures. The people
//! stand still or take one step for the opening (the study §5: one or two stands, 0.6 s from a structure finishing to
//! the next order; ours 3.5 / 1.0 / 0.8 / 5.8 s from three stands, one order at a time).

use std::collections::VecDeque;

use bot_protocol::{OwnUnit, UnitDefId, UnitId, Vec3};
use layout::{Ground, Scene, Slot, Stand, Want};

use super::pianist::lists::{step_id, step_low, step_start};
use super::pianist::picture::Picture;
use super::pianist::{Pianist, Task};
use super::roster::Kit;
use super::Brain;

/// How many leading steps of the list the stand solves for, at most.
const STAND_STEPS: usize = 8;
/// A slot is taken by a building or an order standing this close to it.
const SLOT_TAKEN: f32 = 24.0;
/// The shim refuses a site this close to a metal spot's centre (`ai-host` `SPOT_KEEPOUT`): no slot goes there.
const SPOT_KEEPOUT: f32 = 100.0;

impl Brain {
    /// Solves the stand for `unit` from the leading steps of its list, once. Said in the hands' lines.
    pub(super) fn solve_stand(&mut self, unit: &OwnUnit, pianist: &mut Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) {
        let name = self.actor_name(unit.id);
        let Some(steps) = pianist.scripts.get(&name) else { return };
        let wants = self.stand_wants(steps, unit);
        if wants.is_empty() {
            return;
        }
        let Some(builder) = self.facts_of(unit.def) else { return };
        let Some(extractor) = self.facts_of(kit.extractor) else { return };
        let facts = |def: UnitDefId| self.facts_of(def);
        let hello = &self.world.hello;
        let free: Vec<usize> = self.free_spots(unit, pianist, picture, own, frame, kit).into_iter().map(|(i, _)| i).collect();
        let spots_taken: Vec<usize> = (0..hello.metal_spots.len()).filter(|i| !free.contains(i)).collect();
        let terrain = &hello.terrain;
        // The lane's ground: what the first factory wanted makes must walk it.
        let factory = wants.iter().find_map(|w| match w {
            Want::Building(def) if self.world.is_factory_def(*def) => Some(*def),
            _ => None,
        });
        let passable = factory.and_then(|def| self.passable_for_factory(def));
        let open = |p: Vec3| match passable.as_deref() {
            Some(passable) => {
                let (i, j) = ((p.x / terrain.cell) as usize, (p.z / terrain.cell) as usize);
                i < terrain.width as usize && j < terrain.height as usize && passable[j * terrain.width as usize + i]
            }
            None => true,
        };
        let rect_of = |def: UnitDefId, at: Vec3, facing: i32| facts(def).map(|f| f.footprint.faced(facing).rect(at));
        let mut occupied: Vec<layout::Rect> = own.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| rect_of(u.def, u.pos, u.facing)).collect();
        occupied.extend(pianist.tasks.values().chain(pianist.queued_tasks().map(|(_, t)| t)).filter_map(|t| match t {
            Task::Build { def, near, facing, .. } if self.world.def(*def).is_some_and(|d| d.speed == 0.0) => rect_of(*def, *near, *facing),
            _ => None,
        }));
        let scene = Scene {
            start: unit.pos,
            builder,
            extractor,
            facts: &facts,
            spots: &hello.metal_spots,
            squares: &hello.metal_spot_squares,
            extractor_radius: hello.map.extractor_radius,
            spots_taken: &spots_taken,
            spot_keep_out: SPOT_KEEPOUT,
            ground: Ground::new(terrain, hello.map.width, hello.map.height),
            toward: self.enemy_base(unit.pos),
            open: &open,
            occupied,
            coverage: self.coverage.as_ref().map(|c| (c, self.world.def(kit.extractor).map_or(0.001, |d| d.extracts_metal))),
        };
        let Some(stand) = layout::stand(&scene, &wants) else { return };
        let words = self.stand_words(&stand, unit, &wants);
        eprintln!("[ai {}] f={frame} {name}: {words}", self.ai());
        pianist.done.push(format!("{} {name}: {words}", super::pianist::picture::clock(frame)));
        self.stand = Some((unit.id, stand));
    }

    fn stand_words(&self, stand: &Stand, unit: &OwnUnit, wants: &[Want]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for slot in &stand.slots {
            parts.push(match *slot {
                Slot::Extractor { spot, at } => {
                    let reach = self.world.def(unit.def).map_or(100.0, |d| d.build_distance);
                    let walk = (at.dist2d(stand.at) - reach - 24.0).max(0.0);
                    let where_ = match spot {
                        Some(spot) => format!("spot_{spot}"),
                        None => format!("the metal at ({:.0}, {:.0})", at.x, at.z),
                    };
                    if walk > 0.0 { format!("extractor at {where_} ({walk:.0} out of reach)") } else { format!("extractor at {where_}") }
                }
                Slot::Building { def, at, facing } if self.world.is_factory_def(def) => format!("{} at ({:.0}, {:.0}) facing {}", self.name(def), at.x, at.z, ["south", "east", "north", "west"][facing.rem_euclid(4) as usize]),
                Slot::Building { def, at, .. } => format!("{} at ({:.0}, {:.0})", self.name(def), at.x, at.z),
            });
        }
        let unplaced = if stand.unplaced > 0 { format!("; {} of the {} steps found no slot within reach and go where the layout puts them", stand.unplaced, wants.len()) } else { String::new() };
        let moved = if stand.at.dist2d(unit.pos) > 1.0 { format!(", {:.0} from where it stands", stand.at.dist2d(unit.pos)) } else { String::new() };
        format!("the stand for its opening at ({:.0}, {:.0}){moved}: {}{unplaced}; the steps go as one queue", stand.at.x, stand.at.z, parts.join(", "))
    }

    /// What the leading steps of a list want of the stand: bare `extractor` steps and buildings without a place that
    /// this builder makes, up to and including the first factory, at most `STAND_STEPS`.
    fn stand_wants(&self, steps: &VecDeque<String>, unit: &OwnUnit) -> Vec<Want> {
        let mut wants = Vec::new();
        for step in steps.iter().take(STAND_STEPS) {
            let Some(want) = self.want_of(step, unit) else { break };
            wants.push(want);
            if matches!(want, Want::Building(def) if self.world.is_factory_def(def)) {
                break;
            }
        }
        wants
    }

    /// The want a step is, if the stand can place it: `extractor` (the nearest free spot), or a building without a
    /// place this builder can make that does not stand at a place.
    fn want_of(&self, step: &str, unit: &OwnUnit) -> Option<Want> {
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        let (body, _) = step_id(step);
        let (body, _) = step_low(body);
        let (body, _) = step_start(&body);
        let mut words = body.split_whitespace();
        let kind = words.next().unwrap_or_default();
        let place = words.next();
        match kind {
            "extractor" if place.is_none_or(|p| p == "nearest") => Some(Want::Extractor),
            other if place.is_none() => {
                let def = self.world.def_named(other)?;
                (can(def) && self.world.def(def).is_some_and(|d| d.speed == 0.0) && !self.placed_at_place(def)).then_some(Want::Building(def))
            }
            _ => None,
        }
    }

    /// The stand's slots for `def` that nothing stands on, nothing is ordered at and the engine has not refused.
    pub(super) fn free_stand_slots(&self, builder: UnitId, def: UnitDefId, own: &[OwnUnit], pianist: &Pianist) -> Vec<(Vec3, i32)> {
        let Some((who, stand)) = self.stand.as_ref() else { return Vec::new() };
        if *who != builder {
            return Vec::new();
        }
        let taken = |at: Vec3| {
            own.iter().any(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0) && u.pos.dist2d(at) < SLOT_TAKEN)
                || pianist.tasks.values().chain(pianist.queued_tasks().map(|(_, t)| t)).any(|t| matches!(t, Task::Build { near, .. } if near.dist2d(at) < SLOT_TAKEN))
                || pianist.refused_sites.iter().any(|(d, near, _)| *d == def && near.dist2d(at) < SLOT_TAKEN)
        };
        stand
            .slots
            .iter()
            .filter_map(|s| match *s {
                Slot::Building { def: d, at, facing } if d == def && !taken(at) => Some((at, facing)),
                _ => None,
            })
            .collect()
    }

    /// On a metal map, the stand's extractor tiles still free for `builder`, in the stand's order: a bare `extractor`
    /// step takes them before the metal nearest the builder, as it takes the stand's spots on a spot map, so the
    /// stand's other slots stay clear (metal-step3-speedmetal-1 and -fmp-1: the second extractor, sited nearest the
    /// commander, stood on the plant's slot and the plant was refused for a minute).
    pub(super) fn stand_tiles(&self, builder: &OwnUnit, pianist: &Pianist, own: &[OwnUnit], frame: i32) -> Vec<Vec3> {
        let Some((who, stand)) = self.stand.as_ref() else { return Vec::new() };
        if *who != builder.id {
            return Vec::new();
        }
        let taken = |at: Vec3| {
            own.iter().any(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0) && u.pos.dist2d(at) < SLOT_TAKEN)
                || pianist.tasks.values().chain(pianist.queued_tasks().map(|(_, t)| t)).any(|t| matches!(t, Task::Build { near, .. } if near.dist2d(at) < SLOT_TAKEN))
                || pianist.refused_sites.iter().any(|(_, near, until)| *until > frame && near.dist2d(at) < SLOT_TAKEN)
        };
        stand.slots.iter().filter_map(|s| match *s { Slot::Extractor { spot: None, at } if !taken(at) => Some(at), _ => None }).collect()
    }

    /// The stand's extractor spots still free for `builder`, in the stand's order.
    pub(super) fn stand_spots(&self, builder: &OwnUnit, pianist: &Pianist, picture: &Picture, own: &[OwnUnit], frame: i32, kit: &Kit) -> Vec<usize> {
        let Some((who, stand)) = self.stand.as_ref() else { return Vec::new() };
        if *who != builder.id {
            return Vec::new();
        }
        let free = self.free_spots(builder, pianist, picture, own, frame, kit);
        stand
            .slots
            .iter()
            .filter_map(|s| match *s {
                Slot::Extractor { spot: Some(spot), .. } if free.iter().any(|(i, _)| *i == spot) => Some(spot),
                _ => None,
            })
            .collect()
    }

    /// Whether a step of `unit`'s list is the stand's: a bare `extractor` while the stand has a free spot left
    /// (`prefer`), or a building with a free slot beyond those already taken this tick (`batched`).
    pub(super) fn step_on_the_stand(&self, step: &str, unit: &OwnUnit, pianist: &Pianist, own: &[OwnUnit], prefer: &[usize], batched: &[UnitDefId]) -> bool {
        match self.want_of(step, unit) {
            Some(Want::Extractor) => !prefer.is_empty() || (self.coverage.is_some() && !self.stand_tiles(unit, pianist, own, i32::MAX).is_empty()),
            Some(Want::Building(def)) => self.free_stand_slots(unit.id, def, own, pianist).len() > batched.iter().filter(|d| **d == def).count(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use bot_protocol::{Command, OwnUnit, Resource, Snapshot, Terrain, Tick, UnitDefId, Vec3};

    use crate::brain::pianist::fixtures::{at, brain_of, own};
    use crate::brain::pianist::picture::{Picture, Place};
    use crate::brain::pianist::Pianist;
    use crate::brain::roster::{Kit, ROSTERS};

    /// The opening list of a commander on flat ground beside two spots: the stand is solved when the list first
    /// plays, and every leading step up to the plant goes out at once as one queue (the first order plain, the rest
    /// behind it), the solars and the plant at the stand's exact slots, the extractors on the two nearest spots;
    /// the step after the plant waits its turn as before.
    #[test]
    fn the_opening_goes_as_one_queue_at_the_stands_slots() {
        let mut brain = brain_of(&["armcom", "armmex", "armsolar", "armvp"]);
        let (com, mex, solar, vp) = (UnitDefId(1), UnitDefId(2), UnitDefId(3), UnitDefId(4));
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].build_distance = 145.0;
            defs[0].build_speed = 300.0;
            defs[0].build_options = vec![mex, solar, vp];
            defs[1].footprint = (6, 6);
            defs[1].extracts_metal = 1.8;
            defs[2].footprint = (10, 10);
            defs[3].footprint = (12, 12);
            defs[3].build_options = vec![com];
            let (w, h) = (512u32, 384u32);
            brain.world.hello.terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() };
            brain.world.hello.metal_spots = vec![at(880.0, 1000.0), at(1120.0, 900.0), at(3000.0, 3000.0)];
            brain.world.hello.metal_spot_squares = vec![Vec::new(); 3];
        }
        let commander = OwnUnit { idle: true, ..own(10, 1, at(1000.0, 1000.0)) };
        brain.known_units.insert(commander.id, (com, commander.pos));
        brain.home = commander.pos;
        let kit = Kit { commander: com, extractor: mex, converter: solar, lab: vp, turret: solar, constructor: com, vehicle_constructor: com, advanced_constructor: com, advanced_extractor: mex, raider: com, line: com, resurrector: com, roster: &ROSTERS[0] };
        let places: Vec<Place> = brain.world.hello.metal_spots.iter().enumerate().map(|(i, p)| Place { name: format!("spot_{i}"), at: *p, spot: Some(i), cells: None }).collect();
        let picture = Picture { rules: String::new(), state: serde_json::json!({}), places, parties: Vec::new() };
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        let steps = ["extractor", "extractor", "armsolar", "armsolar", "armsolar", "armvp", "armsolar"];
        pianist.scripts.insert("commander".to_string(), steps.iter().map(|s| s.to_string()).collect::<VecDeque<_>>());
        brain.pianist = Some(pianist);
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        let tick = Tick { frame: 60, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: vec![commander.clone()], allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None } };
        let mut commands = Vec::new();
        brain.play_lists(&tick, &kit, &picture, &mut commands);
        let builds: Vec<(UnitDefId, bool, Option<Vec3>, bool)> = commands
            .iter()
            .filter_map(|c| match c {
                Command::Build { def, queue, site, .. } => Some((*def, *queue, site.as_ref().map(|s| s.near), site.as_ref().is_some_and(|s| s.exact))),
                _ => None,
            })
            .collect();
        assert_eq!(builds.len(), 6, "{commands:?}");
        assert_eq!(builds.iter().map(|b| b.0).collect::<Vec<_>>(), vec![mex, mex, solar, solar, solar, vp]);
        assert_eq!(builds.iter().map(|b| b.1).collect::<Vec<_>>(), vec![false, true, true, true, true, true], "the first plain, the rest queued behind it");
        assert!(builds[2..].iter().all(|b| b.3), "the solars and the plant at exact slots: {builds:?}");
        let sites: Vec<Vec3> = builds.iter().filter_map(|b| b.2).collect();
        assert!(sites[0].dist2d(at(880.0, 1000.0)) <= 1.0 && sites[1].dist2d(at(1120.0, 900.0)) <= 1.0, "the two nearest spots: {sites:?}");
        for (i, a) in sites.iter().enumerate() {
            for b in &sites[i + 1..] {
                assert!(a.dist2d(*b) >= 40.0, "two slots on one ground: {sites:?}");
            }
            assert!(a.dist2d(commander.pos) <= 200.0, "{a:?} is out of the commander's reach");
        }
        let pianist = brain.pianist.as_ref().expect("kept");
        assert!(pianist.tasks.contains_key(&commander.id));
        assert_eq!(pianist.queue_of(commander.id).count(), 5);
        assert_eq!(pianist.scripts["commander"].iter().collect::<Vec<_>>(), vec!["armsolar"], "the step after the plant waits");
        assert!(pianist.done.iter().any(|d| d.contains("the stand for its opening")), "{:?}", pianist.done);
        assert!(brain.stand.is_some());
    }

    /// On a metal map the stand's extractor tiles and its plant's slot are kept while the list plays: a bare
    /// `extractor` step whose stand tile is gone takes no tile over them, and the world built while the pianist
    /// is out of `self` (the lists' pass) carries the reservations (1794106612-human, Full Metal Plate: a turbine
    /// took t3's third tile, the extractor went over the plant's slot, and the plant had no site).
    #[test]
    fn on_a_metal_map_the_stands_tiles_and_slots_are_kept_while_the_list_plays() {
        let mut brain = brain_of(&["armcom", "armmex", "armsolar", "armvp"]);
        let (com, mex, solar, vp) = (UnitDefId(1), UnitDefId(2), UnitDefId(3), UnitDefId(4));
        let (w, h) = (128u32, 128u32);
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].build_distance = 145.0;
            defs[0].build_speed = 300.0;
            defs[0].build_options = vec![mex, solar, vp];
            defs[1].footprint = (6, 6);
            defs[1].extracts_metal = 1.8;
            defs[2].footprint = (10, 10);
            defs[3].footprint = (12, 12);
            defs[3].build_options = vec![com];
            brain.world.hello.terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: vec![200; (w * h) as usize] };
            brain.world.hello.map.metal_map = true;
            brain.world.hello.map.width = 2048.0;
            brain.world.hello.map.height = 2048.0;
        }
        brain.coverage = layout::Coverage::new(&brain.world.hello.terrain, 1.0, 24.0);
        let commander = OwnUnit { idle: true, ..own(10, 1, at(1000.0, 1000.0)) };
        brain.known_units.insert(commander.id, (com, commander.pos));
        brain.home = commander.pos;
        let kit = Kit { commander: com, extractor: mex, converter: solar, lab: vp, turret: solar, constructor: com, vehicle_constructor: com, advanced_constructor: com, advanced_extractor: mex, raider: com, line: com, resurrector: com, roster: &ROSTERS[0] };
        brain.kit = Some(kit);
        let picture = Picture { rules: String::new(), state: serde_json::json!({}), places: Vec::new(), parties: Vec::new() };
        let mut pianist = Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist");
        pianist.scripts.insert("commander".to_string(), ["extractor", "extractor", "armsolar", "armvp"].iter().map(|s| s.to_string()).collect::<VecDeque<_>>());
        let own_units = vec![commander.clone()];
        brain.solve_stand(&commander, &mut pianist, &picture, &own_units, 60, &kit);
        let (_, stand) = brain.stand.clone().expect("a stand");
        let rects: Vec<(layout::Rect, &str)> = stand
            .slots
            .iter()
            .map(|s| match *s {
                layout::Slot::Extractor { at, .. } => (brain.facts_of(mex).unwrap().footprint.rect(at), "tile"),
                layout::Slot::Building { def, at, facing } => (brain.facts_of(def).unwrap().footprint.faced(facing).rect(at), "slot"),
            })
            .collect();
        assert_eq!(rects.iter().filter(|r| r.1 == "tile").count(), 2, "{stand:?}");
        assert_eq!(rects.len(), 4, "{stand:?}");
        // The world of the lists' pass, the pianist out of `self`.
        let world = brain.layout_world_for(&own_units, Some(&pianist));
        for (rect, what) in &rects {
            assert!(world.reservations.iter().any(|r| r.rect.overlaps(rect)), "the stand's {what} at {rect:?} is not kept: {:?}", world.reservations);
        }
        // A tile for a third extractor, asked while the list plays: off every slot of the stand.
        let tile = brain.tile_site(&commander, &own_units, &pianist, commander.pos, super::super::economy::TILE_REACH, 60).expect("a tile");
        let third = brain.facts_of(mex).unwrap().footprint.rect(tile);
        for (rect, what) in &rects {
            assert!(!third.overlaps(rect), "the third extractor at {tile:?} over the stand's {what} at {rect:?}");
        }
    }
}
