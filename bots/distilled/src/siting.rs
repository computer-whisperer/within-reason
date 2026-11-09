//! D-LAYOUT: where a building goes, by the shared base layout engine (`crates/layout`, the flagship's layer since
//! the base layout arc of 2026-10-29: constraints, then bands; `docs/design/2026-10-29-base-layout.md` §1.1), fed as
//! the flagship feeds it (`docs/design/2026-11-09-distilled-layout.md`). The world is what stands (ours finished and
//! framed, every order of ours with a site chosen here, allies' buildings with a square of margin, his known
//! buildings), the plants with their aprons and streets (so nothing of ours closes a street), our construction
//! turrets as build power and our frames, our mobile builders each on its own ground (so none is walled in), the
//! slots the engine refused lately, the stand's unbuilt slots, and the block's sub-lattice once it is laid
//! (`block.rs`). A kind is asked for within its own band: a generator a short walk from its builder, a plant's turret
//! within its reach of the pad, a plant within a few seconds of the yard's point, a frame (a reactor, an advanced
//! converter) and a ring's turret on the farm, a silo in the back field, a defence at its rule's point. The layer's
//! answer is asked of the engine exactly (`slot_site`, the slot or none). It is the bot's one way of siting: when
//! the layer finds no ground the engine searches round the band's centre, off every plant's lane
//! (`fallback_site`, the flagship's fallback). On a metal map the extractors are the coverage's (`economy.rs`
//! `free_site`), not this.
//!
//! 2026-11-03, the user: "update this to use the new base layout engine if we haven't already"; 2026-11-09: "Let's
//! update the distilled bot to use the new base layout engine". Until the second the bot kept its earlier siting
//! beside the layer (each job's own point and the engine's search, the `layout=off` arm and the fallback), sorted
//! kinds by unit data (the advanced converter a generator in banks, econ-flagship-15's fault), and gave the layer
//! no refused slots, no ground for the builders and no block.

use bot_protocol::{BuildSite, MoveClass, MoveKind, OwnUnit, Placement, Tick, UnitDefId, Vec3};
use layout::{Facts, Frame, Ground, Plant, Purpose, Request, Reservation, Reserved, Scene, Slot, Standing, Want, Worker, World};

use crate::Distilled;
use crate::economy::Job;

/// The shim refuses a site this close to a metal spot's centre (`ai-host` `SPOT_KEEPOUT`): no slot goes there.
const SPOT_KEEPOUT: f32 = 100.0;
/// The bands of distance (the flagship's, `crates/bot/src/brain/economy.rs`): how far from its anchor a kind's site is
/// looked for before the band widens (twice, doubling).
const BAND_GENERATOR: f32 = 300.0;
const BAND_TURRET: f32 = 400.0;
const BAND_PLANT: f32 = 600.0;
const BAND_FRAME: f32 = 600.0;
const BAND_PLACE: f32 = 200.0;
/// The back field: this far behind home, away from the enemy (the silos; the reactors before the block is laid).
const BACK_FIELD: f32 = 350.0;
/// The yard's point: a plant is looked for round the point this far toward the enemy from home.
const YARD: f32 = 350.0;
/// The fallback: the engine's search this far round the band's centre, two squares from anything, off every
/// plant's lane this far out from its front (the flagship's figures, `crates/bot/src/brain/economy.rs`).
const FALLBACK_SEARCH: f32 = 600.0;
const FALLBACK_LANE: f32 = 300.0;
/// Banks of converters and wind turbines stop at this many, solars and the other generators at 8 (the flagship's
/// caps: the people's clusters of 7 to 10; a longer bank is a wall the builders walk round).
const BANK_CAP: usize = 6;
const BANK_CAP_GENERATOR: usize = 8;
/// The engine is asked for the slot itself: this much search round it, no gap.
const SLOT_SEARCH: f32 = 24.0;

impl Distilled {
    /// D-STAND: the commander's stand (`layout::stand`, the flagship's H-ECO-STAND since 2026-10-29), solved once at
    /// the start from where the commander stands: D-OPENING's two extractors, two solars and the plant as slots within
    /// its reach, the plant with its street clear and facing the enemy, nothing fencing the commander in. `order`
    /// sends the slots as one queue (`Job::Stand`), so the commander never idles or walks between its first
    /// structures; a slot the engine refuses falls to the opening's rungs. `None` when the kit lacks a piece or the
    /// map has no terrain.
    pub(crate) fn solve_stand(&self, commander: &OwnUnit) -> Option<layout::Stand> {
        let (solar, plant) = (self.kit.solar?, self.kit.plant?);
        let wants = [Want::Extractor, Want::Extractor, Want::Building(solar), Want::Building(solar), Want::Building(plant)];
        let builder = self.facts_of(commander.def)?;
        let extractor_def = self.kit.extractor?;
        let extractor = self.facts_of(extractor_def)?;
        let facts = |def: UnitDefId| self.facts_of(def);
        let hello = &self.hello;
        // The apron's ground: what the plant makes must stand on it (the flagship's stand has judged it so).
        let terrain = &hello.terrain;
        let walk = self.product_walk(plant);
        let open = |p: Vec3| match &walk {
            Some(walk) => {
                let (i, j) = ((p.x / terrain.cell) as usize, (p.z / terrain.cell) as usize);
                i < terrain.width as usize && j < terrain.height as usize && walk[j * terrain.width as usize + i]
            }
            None => true,
        };
        let occupied: Vec<layout::Rect> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| self.facts_of(u.def).map(|f| f.footprint.faced(u.facing).rect(u.pos))).collect();
        let scene = Scene {
            start: commander.pos,
            builder,
            extractor,
            facts: &facts,
            spots: &hello.metal_spots,
            squares: &hello.metal_spot_squares,
            extractor_radius: hello.map.extractor_radius,
            spots_taken: &[],
            spot_keep_out: SPOT_KEEPOUT,
            ground: Ground::new(&hello.terrain, hello.map.width, hello.map.height),
            toward: self.enemy_home,
            open: &open,
            occupied,
            coverage: self.coverage.as_ref().map(|c| (c, self.def(extractor_def).map_or(0.001, |d| d.extracts_metal))),
        };
        let stand = layout::stand(&scene, &wants)?;
        let slots: Vec<String> = stand
            .slots
            .iter()
            .map(|s| match *s {
                Slot::Extractor { spot, at } => format!("extractor at {} ({:.0},{:.0})", spot.map_or("a tile".to_string(), |i| format!("spot_{i}")), at.x, at.z),
                Slot::Building { def, at, facing } => format!("{} at ({:.0},{:.0}) facing {facing}", self.name_of(def), at.x, at.z),
            })
            .collect();
        eprintln!(
            "[distilled ai={}] f={} stand: at ({:.0},{:.0}), {:.0} from the start, walk {:.0}: {}; {} of {} unplaced",
            hello.ai_id,
            self.frame,
            stand.at.x,
            stand.at.z,
            stand.at.dist2d(commander.pos),
            stand.walk,
            slots.join(", "),
            stand.unplaced,
            wants.len()
        );
        Some(stand)
    }

    /// What the layer knows of a kind: its footprint and, for a factory, its street's half-width (its widest product's)
    /// and the ground its products stand on (`product_walk`), as the flagship's layer has them.
    pub(crate) fn facts_of(&self, def: UnitDefId) -> Option<Facts> {
        let d = self.def(def)?;
        let facts = Facts::of(d);
        if !facts.factory {
            return Some(facts);
        }
        let products = d.build_options.iter().filter_map(|p| self.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some());
        Some(facts.with_products(Facts::product_half_of(products.map(|p| p.path_width())), self.product_walk(def)))
    }

    /// The ground a movement class stands on over the hello's terrain (`terrain::passable`), found once a class: a
    /// factory's products' and a builder's own. `None` without terrain data.
    fn class_walk(&self, class: MoveClass) -> Option<layout::Passable> {
        let terrain = &self.hello.terrain;
        if terrain.cell <= 0.0 || terrain.heights.is_empty() {
            return None;
        }
        let key: ClassKey = (class.kind, (class.max_slope * 255.0).round() as i32, class.depth as i32);
        if let Some((_, walk)) = self.class_walks.borrow().iter().find(|(k, _)| *k == key) {
            return Some(walk.clone());
        }
        let walk: layout::Passable = terrain::passable(terrain, class).into();
        self.class_walks.borrow_mut().push((key, walk.clone()));
        Some(walk)
    }

    /// The ground a factory's products stand on: the strictest movement class among them (the fewest cells
    /// passable), found once a kind. Without it the street rule was judged on the world's `open`, which is
    /// everywhere, and on 2026-11-04 (dist119 game 02) the advanced vehicle plant stood facing Comet's crater wall
    /// 60 elmos before its door with the street called open; the solars the layer then placed round it closed the
    /// real ways out and every advanced constructor sat in the yard to the end. The flagship's layer has had the
    /// products' ground since 2026-10-31 (apex-v0409-comet-sym0-opus-2's three plants penned by the rim).
    fn product_walk(&self, def: UnitDefId) -> Option<layout::Passable> {
        if let Some(walk) = self.product_walks.borrow().get(&def) {
            return walk.clone();
        }
        let classes = self.def(def).into_iter().flat_map(|d| d.build_options.iter()).filter_map(|p| self.def(*p)).filter(|p| p.speed > 0.0).filter_map(|p| p.move_class);
        let open = |walk: &layout::Passable| walk.iter().filter(|p| **p).count();
        let walk = classes.filter_map(|class| self.class_walk(class)).min_by_key(open);
        self.product_walks.borrow_mut().insert(def, walk.clone());
        walk
    }

    /// What a kind is placed for (the design's level 1), by its part in the bot's kit: a plant has a street, a
    /// construction turret a reach; the reactors, the advanced converter (the flagship's default `holds`) and the
    /// silo are frames, raised where the build power reaches; the solars and the basic converter stand in banks
    /// beside their builder; a defence or a radar stands at its rule's point. Until 2026-11-09 unit data sorted the
    /// kinds, and the advanced converter (380 metal, under the sorter's 500) was a generator in banks of six: the
    /// flagship's fault of econ-flagship-15, where 54 of them decayed beside their builders.
    pub(crate) fn purpose_of(&self, def: UnitDefId) -> Purpose {
        let (kit, is) = (&self.kit, |kind: Option<UnitDefId>| kind == Some(def));
        let d = self.def(def);
        if d.is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty()) {
            Purpose::Plant
        } else if d.is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0) {
            Purpose::Turret
        } else if is(kit.fusion) || is(kit.advanced_fusion) || is(kit.advanced_converter) || is(kit.silo) {
            Purpose::Frame
        } else if is(kit.converter) {
            Purpose::Generator { cap: Some(BANK_CAP) }
        } else if is(kit.solar) || is(kit.advanced_solar) {
            Purpose::Generator { cap: Some(BANK_CAP_GENERATOR) }
        } else {
            Purpose::Place
        }
    }

    /// A plant for the street rule: its footprint, facing and its widest product's half-width.
    pub(crate) fn plant_of(&self, def: UnitDefId, at: Vec3, facing: i32, ours: bool, name: String) -> Option<Plant> {
        let f = self.facts_of(def)?;
        let d = self.def(def)?;
        let widest = d.build_options.iter().filter_map(|p| self.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some()).max_by(|a, b| a.path_width().total_cmp(&b.path_width()));
        Some(Plant { centre: at, footprint: f.footprint, facing, product_half: f.product_half, ours, name, widest: widest.map_or(String::new(), |p| p.name.clone()), passable: f.product_passable.clone() })
    }

    /// The world the layer chooses over this tick for `asker` (none: the block's laying): what stands (ours,
    /// finished or framed; our orders with a site chosen here and not yet begun; allies' buildings; his known
    /// buildings), the plants (ours, ordered and allies'), our construction turrets as build power, our frames, our
    /// mobile builders each on its class's ground, the slots the engine refused in the last minute, the stand's
    /// unbuilt slots while the commander holds the stand's queue, and the block's sub-lattice and frame. The
    /// asker's walk to a site is priced on its own ground.
    pub(crate) fn layout_world(&self, tick: &Tick, asker: Option<&OwnUnit>) -> World<'_> {
        let hello = &self.hello;
        let factory = |def: UnitDefId| self.def(def).is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty());
        let rect_of = |def: UnitDefId, at: Vec3, facing: i32| self.facts_of(def).map(|f| f.footprint.faced(facing).rect(at));
        let standing_of = |def: UnitDefId, at: Vec3, facing: i32, ours: bool, name: String| {
            let d = self.def(def)?;
            let bank = matches!(self.purpose_of(def), Purpose::Generator { .. });
            Some(Standing { rect: rect_of(def, at, facing)?, def: Some(def), kind: d.name.clone(), ours, bank, metal: d.metal_cost, health: d.health, blast: d.death_blast.map(|b| (b.radius, b.damage)), name })
        };
        let mut standing: Vec<Standing> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| standing_of(u.def, u.pos, u.facing, true, format!("our {}", self.name_of(u.def)))).collect();
        let mut plants: Vec<Plant> = self.own.values().filter(|u| factory(u.def)).filter_map(|u| self.plant_of(u.def, u.pos, u.facing, true, format!("our {}", self.name_of(u.def)))).collect();
        // Orders with a site chosen here whose frame has not begun: the ground is taken.
        for order in self.economy.orders.values() {
            let (Some((at, facing)), Some(def)) = (order.site, order.job.def(&self.kit)) else {
                continue;
            };
            if self.own.values().any(|u| u.def == def && u.pos.dist2d(at) < 1.0) {
                continue;
            }
            standing.extend(standing_of(def, at, facing, true, format!("an ordered {}", self.name_of(def))));
            if factory(def) {
                plants.extend(self.plant_of(def, at, facing, true, format!("an ordered {}", self.name_of(def))));
            }
        }
        // Allies' buildings; their units are no walls (the flagship's fault of 1793769060-human).
        for a in tick.snapshot.allies.iter().filter(|a| self.def(a.def).is_some_and(|d| d.speed == 0.0)) {
            standing.extend(standing_of(a.def, a.pos, a.facing, false, format!("an allied {}", self.name_of(a.def))));
            if factory(a.def) {
                plants.extend(self.plant_of(a.def, a.pos, a.facing, false, format!("an allied {}", self.name_of(a.def))));
            }
        }
        for seen in self.army.his_buildings.values() {
            if let Some(def) = seen.def {
                standing.extend(standing_of(def, seen.pos, 0, false, format!("his {}", self.name_of(def))));
            }
        }
        let workers: Vec<Worker> = self
            .own
            .values()
            .filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty()))
            .filter_map(|u| self.def(u.def).map(|d| Worker { at: u.pos, reach: d.build_distance, power: d.build_speed }))
            .collect();
        let frames: Vec<Frame> =
            self.own.values().filter(|u| u.being_built && self.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| Some(Frame { rect: rect_of(u.def, u.pos, u.facing)?, metal: self.def(u.def)?.metal_cost })).collect();
        // Our mobile builders, each kept a way out of what the layer places round it, on its own ground.
        let builders: Vec<layout::Builder> = self
            .own
            .values()
            .filter(|u| !u.being_built)
            .filter_map(|u| self.def(u.def).filter(|d| d.speed > 0.0 && d.build_speed > 0.0).map(|d| layout::Builder { at: u.pos, half: d.path_width() / 2.0, passable: d.move_class.and_then(|c| self.class_walk(c)) }))
            .collect();
        // The stand's slots not yet built, while the commander holds its queue: nothing else takes their ground.
        let mut reservations: Vec<Reservation> = Vec::new();
        if let Some(stand) = self.stand.as_ref().filter(|_| self.economy.orders.values().any(|o| o.job == Job::Stand)) {
            let built = |def: Option<UnitDefId>, at: Vec3| self.own.values().any(|u| Some(u.def) == def && u.pos.dist2d(at) < 1.0);
            reservations.extend(stand.slots.iter().filter_map(|slot| match *slot {
                Slot::Building { def, at, facing } if !built(Some(def), at) => rect_of(def, at, facing),
                Slot::Extractor { spot: None, at } if !built(self.kit.extractor, at) => self.kit.extractor.and_then(|e| rect_of(e, at, 0)),
                _ => None,
            }).map(|rect| Reservation { rect, for_: Reserved::Nothing }));
        }
        let (mut lattice, mut farm_frame) = (None, None);
        if let Some(block) = &self.block {
            reservations.extend(block.reservations());
            lattice = Some(block.lattice());
            farm_frame = Some(block.frame.footprint);
        }
        let refused: Vec<(UnitDefId, Vec3)> = self.economy.sites_refused.iter().filter(|(_, _, until)| self.frame < *until).map(|(def, at, _)| (*def, *at)).collect();
        // The asker's ground: its walk to a site is a walk on it.
        let terrain = &hello.terrain;
        let walk = asker.and_then(|u| self.def(u.def)).and_then(|d| d.move_class).and_then(|c| self.class_walk(c));
        let open = move |p: Vec3| match &walk {
            Some(walk) => {
                let (i, j) = ((p.x / terrain.cell) as usize, (p.z / terrain.cell) as usize);
                i < terrain.width as usize && j < terrain.height as usize && walk[j * terrain.width as usize + i]
            }
            None => true,
        };
        World {
            ground: Ground::new(terrain, hello.map.width, hello.map.height),
            spots: &hello.metal_spots,
            spot_keep_out: SPOT_KEEPOUT,
            standing,
            plants,
            builders,
            workers,
            frames,
            reservations,
            lattice,
            farm_frame,
            refused,
            home: self.home,
            toward: self.enemy_home,
            open: Box::new(open),
            walks: Default::default(),
            standing_near: Default::default(),
            spots_near: Default::default(),
            streets: Default::default(),
            blasts_near: Default::default(),
            farm: Default::default(),
            fixed: Default::default(),
            walk_sums: Default::default(),
        }
    }

    /// The back field: behind home, away from the enemy.
    pub(crate) fn back_field(&self) -> Vec3 {
        self.toward_enemy(self.home, -BACK_FIELD)
    }

    /// The kind a job places and its band: where its site is looked for and how far round it. With the block laid
    /// (`block.rs`) a frame's and a ring turret's band is the farm's; a plant's turret stays at its plant.
    pub(crate) fn band_for(&self, job: Job, unit: &OwnUnit) -> Option<(UnitDefId, Vec3, f32)> {
        let def = job.def(&self.kit)?;
        let is_plant = |u: &OwnUnit| Some(u.def) == self.kit.plant || Some(u.def) == self.kit.advanced_plant || Some(u.def) == self.kit.air_plant;
        let nearest_plant = || self.own.values().filter(|u| is_plant(u)).min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos))).map_or(unit.pos, |p| p.pos);
        let farm = self.block.as_ref().map(|block| self.farm_band(block));
        Some(match job {
            // Beside its builder: a solar, and since 2026-11-04 a converter (at the back field every converter cost
            // its builder the walk there, eleven a minute standing of forty constructors on the job, dist122).
            Job::Solar | Job::AdvancedSolar | Job::Converter => (def, unit.pos, BAND_GENERATOR),
            Job::Turret(target) => match self.own.get(&target) {
                Some(plant) if is_plant(plant) => (def, plant.pos, BAND_TURRET),
                // The ring's: on the farm, or round its frame before the block is laid.
                Some(frame) => farm.map_or((def, frame.pos, BAND_TURRET), |(centre, radius)| (def, centre, radius)),
                None => (def, nearest_plant(), BAND_TURRET),
            },
            // Not the first plant inside the commander's arm (tried 2026-11-03 for the pool's 0:34: it began at 0:43
            // instead of 0:47, and the commander, hemmed in by its own plant and field, walked one solar order for
            // 250-360 s in two to four games of sixteen against one: dist88, dist90, dist91 against dist86).
            Job::Plant | Job::AdvancedPlant => (def, self.toward_enemy(self.home, YARD), BAND_PLANT),
            Job::AirPlant => (def, self.air_plant_site(), BAND_PLANT),
            Job::Fusion | Job::AdvancedFusion | Job::AdvancedConverter => farm.map_or((def, self.back_field(), BAND_FRAME), |(centre, radius)| (def, centre, radius)),
            Job::Silo => (def, self.back_field(), BAND_FRAME),
            Job::AntiAir => (def, self.anti_air_site(), BAND_PLACE),
            Job::Radar(at) => (def, at, BAND_PLACE),
            Job::Extractor(_) | Job::Moho(_) | Job::Assist | Job::AirAssist | Job::Help(_) | Job::Reclaim(_) | Job::Stand => {
                return None;
            }
        })
    }

    /// The layer's site for this job and builder: the slot, its facing and the layer's words for it, or why there
    /// is none.
    pub(crate) fn layout_site(&self, world: &World, job: Job, unit: &OwnUnit) -> Result<layout::Site, String> {
        let (def, centre, radius) = self.band_for(job, unit).ok_or_else(|| "not a layout kind".to_string())?;
        let (facts, builder) = (self.facts_of(def).ok_or("no facts")?, self.facts_of(unit.def).ok_or("no builder facts")?);
        let req = Request { facts, purpose: self.purpose_of(def), centre, radius, builder_at: unit.pos, builder };
        layout::choose(world, &req)
    }

    /// The engine asked for the layer's slot itself: that site or none (the layer kept it off every apron, street
    /// and spot).
    pub(crate) fn slot_site(at: Vec3, facing: i32) -> BuildSite {
        BuildSite { near: at, search_radius: SLOT_SEARCH, min_dist: 0, placements: vec![Placement { facing, keep_out: Vec::new() }], exact: true }
    }

    /// The engine's search for a site the layer found none for (the flagship's fallback, `fallback_placements`):
    /// round the band's centre, two squares from anything, off every plant's lane (ours standing or framed,
    /// allies') `FALLBACK_LANE` out from its front, and for a plant the facings toward the enemy in order. The
    /// apron the layer keeps empty is the nearest free ground to a bare search, which fills a plant's door first
    /// (1793769060-human).
    pub(crate) fn fallback_site(&self, def: UnitDefId, centre: Vec3, tick: &Tick) -> BuildSite {
        let lane = |d: UnitDefId, at: Vec3, facing: i32| self.facts_of(d).filter(|f| f.factory).map(|f| layout::lane(f.footprint, at, facing, FALLBACK_LANE, f.product_half));
        let keep_out: Vec<bot_protocol::Lane> = self.own.values().filter_map(|u| lane(u.def, u.pos, u.facing)).chain(tick.snapshot.allies.iter().filter_map(|a| lane(a.def, a.pos, a.facing))).collect();
        let factory = self.def(def).is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty());
        let facings: Vec<i32> = if factory { layout::facings_toward(centre, self.enemy_home).to_vec() } else { vec![0] };
        BuildSite { near: centre, search_radius: FALLBACK_SEARCH, min_dist: 2, placements: facings.into_iter().map(|facing| Placement { facing, keep_out: keep_out.clone() }).collect(), exact: false }
    }

    pub(crate) fn name_of(&self, def: UnitDefId) -> String {
        self.def(def).map_or_else(|| format!("def {}", def.0), |d| d.name.clone())
    }
}

/// A movement class as the key of its ground: its kind, steepest slope and depth.
pub(crate) type ClassKey = (MoveKind, i32, i32);
