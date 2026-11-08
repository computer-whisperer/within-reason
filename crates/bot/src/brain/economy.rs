//! Where a building goes: the site for what the pianist's builders are told to build (`place_planned`, `place_in`,
//! `build_site_for`, the extractor's exact site). The layout layer (`crates/layout`, docs/design/2026-10-29-base-layout.md
//! §1.1) chooses every site but an extractor's and a sea building's: constraints, then bands, over a world of what
//! stands (ours, allies', his), the plants with their aprons and streets, the turrets' reach and the frames, the
//! stand's reservations and the sites the engine refused. The commander's opening takes the stand's slots first
//! (`stand.rs`). The rules that chose what to build went with the heuristic bot (2026-09-25,
//! `docs/design/2026-09-25-one-decider.md`).

use bot_protocol::{BuildSite, OwnUnit, Placement, UnitDefId, Vec3};
use layout::{Facts, Frame, Ground, Plant, Purpose, Request, Reservation, Reserved, Standing, Worker, World};

use super::roster::Kit;
use super::Brain;

pub(super) const EARLY_COMMANDER_LEASH: f32 = 1500.0;
/// How far from a layout slot the engine's site may be: a grid step, so the slot is the site or nothing.
const SLOT_SEARCH: f32 = 24.0;
/// How far an extractor's centre may be from the spot's centre and still be the spot's (the dropped-order fallback).
pub(super) const MEX_PATCH: f32 = 130.0;
/// The shim refuses a site this close to a metal spot's centre (`ai-host` `SPOT_KEEPOUT`): no slot goes there.
pub(super) const SPOT_KEEPOUT: f32 = 100.0;
/// The bands of distance (level 0): how far from its anchor a kind's site is looked for before the band widens.
/// A generator or converter stays a short walk from its builder (the people's banks are where the constructor was
/// working); a turret within its own reach of a pad; a plant and a frame within a few seconds of the builder.
const BAND_GENERATOR: f32 = 300.0;
const BAND_TURRET: f32 = 400.0;
const BAND_PLANT: f32 = 600.0;
const BAND_FRAME: f32 = 600.0;
/// A stockpiler (a silo, an anti-nuke) stands behind home, away from the enemy (the user, 2026-10-09: "putting it
/// in the back field for now").
const BACK_FIELD: f32 = 300.0;
const BAND_BACK_FIELD: f32 = 500.0;
/// A step with a place: the site is chosen within this of the place (a defence a little toward the enemy from it).
pub(super) const BAND_PLACE: f32 = 200.0;
/// How many failing site searches a game dumps for replay, and how many of one kind (`dump_layout_failure`).
const LAYOUT_DUMPS: usize = 6;
const LAYOUT_DUMPS_PER_KIND: usize = 2;
/// On a metal map, how far from the builder its "nearest" extractor tile may lie (`tile_site`).
pub(super) const TILE_REACH: f32 = 1_200.0;
/// A tile the engine refused within this distance is left alone while the refusal stands.
const TILE_REFUSED: f32 = 40.0;
/// No extractor tile within this distance of a building of his (he holds that ground).
const HIS_BUILDING_OFF: f32 = 200.0;
pub(super) const BAND_DEFENCE: f32 = 150.0;
/// Banks of converters and wind turbines stop at this many (§1.3: the chain sum); solars and the other generators
/// at 8 (the people's clusters of 7 to 10; a longer bank is a wall the builders walk round: block-2, the user's word).
const BANK_CAP: usize = 6;
const BANK_CAP_GENERATOR: usize = 8;
/// A sea building's site: the engine's search this far round the nearest water.
const WATER_SEARCH: f32 = 600.0;
/// The layer's fallback keeps the engine's search off every plant's lane this far out from its front: the apron and
/// the street's first stretch, where the search put a solar, a storage, a turret and an advanced plant in front of
/// one door (1793769060-human, plant_28133 8:01-10:48).
const FALLBACK_LANE: f32 = 300.0;
/// No orders before this frame: the engine loses them.
pub(super) const FIRST_ORDER_FRAME: i32 = 60;
/// A builder is not judged idle for this long after an order: the order has to reach it first.
pub(super) const ORDER_GRACE_FRAMES: i32 = 45;

/// What one builder should do next.
pub(super) enum Plan {
    Extractor(Vec3),
    /// A building the engine sites near `anchor`: a sea building on the water (the layout layer's ground is land
    /// until its water step), a tier-2 extractor over ours, or the layer's fallback when it found no ground.
    Near(UnitDefId, Vec3),
    /// A building at a slot the layout layer chose, with its facing: the site exactly, or nothing.
    Slot(UnitDefId, Vec3, i32),
}

impl Plan {
    pub(super) fn anchor(&self) -> Vec3 {
        match *self {
            Plan::Extractor(at) | Plan::Near(_, at) | Plan::Slot(_, at, _) => at,
        }
    }
}

impl Brain {
    /// What the layout layer knows of a definition: its footprint and, for a factory, its street's half-width (its
    /// widest product's).
    pub(super) fn facts_of(&self, def: UnitDefId) -> Option<Facts> {
        let d = self.world.def(def)?;
        let facts = Facts::of(d);
        if !facts.factory {
            return Some(facts);
        }
        let products: Vec<_> = d.build_options.iter().filter_map(|p| self.world.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some()).collect();
        // The ground its products stand on: the strictest class among them (a vehicle plant's on slopes to 28 where
        // the route survey's Pawn climbs 105: apex-v0409-comet-sym0-opus-2's three plants penned by a crater's rim).
        let walk = self.passable_for_factory(def);
        Some(facts.with_products(Facts::product_half_of(products.iter().map(|p| p.path_width())), walk))
    }

    /// What a kind is placed for (the design's level 1).
    pub(super) fn purpose_of(&self, def: UnitDefId) -> Purpose {
        let Some(d) = self.world.def(def) else { return Purpose::Place };
        let static_builder = d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty();
        if self.world.is_factory_def(def) {
            Purpose::Plant
        } else if static_builder {
            Purpose::Turret
        } else if (d.converter.is_some() && d.metal_cost < 500.0) || d.wind_cap > 0.0 {
            Purpose::Generator { cap: Some(BANK_CAP) }
        } else if (d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.tidal_make > 0.0) && d.metal_cost < 500.0 {
            Purpose::Generator { cap: Some(BANK_CAP_GENERATOR) }
        } else if self.placed_at_place(def) {
            Purpose::Place
        } else {
            Purpose::Frame
        }
    }

    /// The kind's own band of distance without a place from the player: its centre and radius. Once the block is
    /// laid a turret's and a frame's bands are the block's reach from its centre (§1.1: a frame no farther from the
    /// block's centre than the block's reach; the turret pile stands where the advanced plant stands).
    pub(super) fn band_for(&self, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit]) -> (Vec3, f32) {
        let stockpiler = self.world.def(def).is_some_and(|d| d.stockpile_seconds > 0.0);
        if stockpiler {
            return (self.forward_of_home(-BACK_FIELD), BAND_BACK_FIELD);
        }
        let block = self.block.as_ref().map(|b| (b.centre, b.reach));
        match self.purpose_of(def) {
            Purpose::Generator { .. } => (builder.pos, BAND_GENERATOR),
            Purpose::Turret => block.unwrap_or_else(|| {
                let pad = own.iter().filter(|u| self.world.is_factory_def(u.def)).min_by(|a, b| a.pos.dist2d(builder.pos).total_cmp(&b.pos.dist2d(builder.pos)));
                (pad.map_or(builder.pos, |p| p.pos), BAND_TURRET)
            }),
            Purpose::Plant => (builder.pos, BAND_PLANT),
            Purpose::Frame => block.unwrap_or((builder.pos, BAND_FRAME)),
            Purpose::Place => (builder.pos, BAND_PLACE),
        }
    }

    /// The world the layout layer chooses over, from this tick's units: what stands (ours finished and framed, every
    /// unstarted build of ours, allies' buildings with a square of margin, his known buildings), the plants with their aprons
    /// and streets (ours and allies'), our construction turrets as build power, our frames, the stand's free slots as
    /// reservations, the sites the engine refused lately, and the block's rows, gantry slot and sub-lattice.
    pub(super) fn layout_world<'a>(&'a self, own: &[OwnUnit]) -> World<'a> {
        self.layout_world_for(own, self.pianist.as_ref())
    }

    /// `layout_world` with the pianist named: while the lists play the pianist is out of `self` (`lists.rs`
    /// `play_lists`), and a world built then from `self.pianist` knew neither the stand's slots nor the builds the
    /// list had ordered (1794106612-human on Full Metal Plate: t3's third extractor went over its plant's slot).
    pub(super) fn layout_world_for<'a>(&'a self, own: &[OwnUnit], pianist: Option<&super::pianist::Pianist>) -> World<'a> {
        let mut world = self.layout_world_without_block_for(own, pianist);
        if let Some(block) = self.block.as_ref() {
            world.reservations.extend(block.reservations());
            world.lattice = Some(block.lattice());
            world.frame_slots = block.frame_slots.clone();
        }
        world
    }

    /// The world before the block is laid: what `lay_block` chooses the block's ground over.
    pub(super) fn layout_world_without_block<'a>(&'a self, own: &[OwnUnit]) -> World<'a> {
        self.layout_world_without_block_for(own, self.pianist.as_ref())
    }

    fn layout_world_without_block_for<'a>(&'a self, own: &[OwnUnit], pianist: Option<&super::pianist::Pianist>) -> World<'a> {
        let hello = &self.world.hello;
        let rect_of = |def: UnitDefId, at: Vec3, facing: i32| self.facts_of(def).map(|f| f.footprint.faced(facing).rect(at));
        let standing_of = |def: UnitDefId, at: Vec3, facing: i32, ours: bool, name: String| {
            let d = self.world.def(def)?;
            let bank = matches!(self.purpose_of(def), Purpose::Generator { .. });
            Some(Standing { rect: rect_of(def, at, facing)?, def: Some(def), kind: d.name.clone(), ours, bank, metal: d.metal_cost, health: d.health, blast: d.death_blast.map(|b| (b.radius, b.damage)), name })
        };
        let mut standing: Vec<Standing> = own.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| standing_of(u.def, u.pos, u.facing, true, self.handle(u))).collect();
        let mut plants: Vec<Plant> = own.iter().filter(|u| self.world.is_factory_def(u.def)).filter_map(|u| self.plant_of(u.def, u.pos, u.facing, true, self.actor_name(u.id))).collect();
        if let Some(pianist) = pianist {
            for task in pianist.tasks.values().chain(pianist.queued_tasks().map(|(_, t)| t)) {
                if let super::pianist::Task::Build { def, near, started: false, facing, .. } = task
                    && self.world.def(*def).is_some_and(|d| d.speed == 0.0 && d.extracts_metal == 0.0)
                {
                    standing.extend(standing_of(*def, *near, *facing, true, format!("an ordered {}", self.name(*def))));
                    if self.world.is_factory_def(*def) {
                        plants.extend(self.plant_of(*def, *near, *facing, true, format!("an ordered {}", self.name(*def))));
                    }
                }
            }
        }
        // Allies' buildings; their units are not walls (until 2026-11-04 every allied unit stood as one, and an Apex
        // soldier or constructor in its own lab's door closed that lab's street in our world: 1793769060-human).
        for a in self.allies.iter().filter(|a| self.world.def(a.def).is_some_and(|d| d.speed == 0.0)) {
            let name = format!("an allied {} at {}", self.name(a.def), self.world.grid(a.pos));
            standing.extend(standing_of(a.def, a.pos, a.facing, false, name.clone()));
            if self.world.is_factory_def(a.def) {
                plants.extend(self.plant_of(a.def, a.pos, a.facing, false, name));
            }
        }
        for (def, pos, _) in self.enemy_buildings.values() {
            standing.extend(standing_of(*def, *pos, 0, false, format!("his {}", self.name(*def))));
        }
        let workers: Vec<Worker> = own
            .iter()
            .filter(|u| !u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty()))
            .filter_map(|u| self.world.def(u.def).map(|d| Worker { at: u.pos, reach: d.build_distance, power: d.build_speed }))
            .collect();
        let frames: Vec<Frame> = own.iter().filter(|u| u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0)).filter_map(|u| Some(Frame { rect: rect_of(u.def, u.pos, u.facing)?, metal: self.world.def(u.def)?.metal_cost })).collect();
        // Our mobile builders, each kept a way out of what the layer places round it.
        let builders: Vec<layout::Builder> = own
            .iter()
            .filter(|u| !u.being_built)
            .filter_map(|u| {
                let d = self.world.def(u.def)?;
                (d.speed > 0.0 && d.build_speed > 0.0).then(|| layout::Builder { at: u.pos, half: d.path_width() / 2.0, passable: self.passable_for_class(d.move_class) })
            })
            .collect();
        // The stand's free slots and, on a metal map, its free extractor tiles: a turbine the stand could not place
        // took the third extractor's tile in 1794106612-human (t3), and the extractor went over the plant's slot.
        let reservations: Vec<Reservation> = match (&self.stand, pianist) {
            (Some((who, stand)), Some(pianist)) => {
                let commander = own.iter().find(|u| u.id == *who);
                let tiles: Vec<Vec3> = commander.map(|c| self.stand_tiles(c, pianist, own, i32::MAX)).unwrap_or_default();
                stand
                    .slots
                    .iter()
                    .filter_map(|s| match *s {
                        layout::Slot::Building { def, at, facing } if self.free_stand_slots(*who, def, own, pianist).iter().any(|(a, _)| *a == at) => rect_of(def, at, facing),
                        layout::Slot::Extractor { spot: None, at } if tiles.contains(&at) => self.kit.and_then(|k| rect_of(k.extractor, at, 0)),
                        _ => None,
                    })
                    .map(|rect| Reservation { rect, for_: Reserved::Nothing })
                    .collect()
            }
            _ => Vec::new(),
        };
        let refused: Vec<(UnitDefId, Vec3)> = self.pianist.as_ref().map(|p| p.refused_sites.iter().map(|(d, at, _)| (*d, *at)).collect()).unwrap_or_default();
        let terrain = &hello.terrain;
        let passable = self.routes_passable();
        let open = move |p: Vec3| match passable {
            Some(passable) => {
                let (i, j) = ((p.x / terrain.cell) as usize, (p.z / terrain.cell) as usize);
                i < terrain.width as usize && j < terrain.height as usize && passable[j * terrain.width as usize + i]
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
            frame_slots: Vec::new(),
            reservations,
            lattice: None,
            refused,
            home: self.home,
            toward: self.enemy_base(self.home),
            open: Box::new(open),
            walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(),
        }
    }

    /// A plant for the street rule: its footprint, facing, its widest product's half-width and its products' ground.
    pub(super) fn plant_of(&self, def: UnitDefId, at: Vec3, facing: i32, ours: bool, name: String) -> Option<Plant> {
        let f = self.facts_of(def)?;
        let d = self.world.def(def)?;
        let widest = d.build_options.iter().filter_map(|p| self.world.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some()).max_by(|a, b| a.path_width().total_cmp(&b.path_width()));
        Some(Plant { centre: at, footprint: f.footprint, facing, product_half: f.product_half, ours, name, widest: widest.map_or(String::new(), |p| p.name.clone()), passable: f.product_passable.clone() })
    }

    /// Where a building of `def` goes for `builder` without a place from the player: the stand's slot for its kind
    /// while one is free; the water for a sea building; else the layout layer's choice within the kind's own band.
    pub(super) fn place_planned(&self, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit]) -> Plan {
        self.place_planned_in(&self.layout_world(own), def, builder, own)
    }

    /// `place_planned` over a world built already: the menu asks for every kind of every builder each pass over one
    /// world (econ-flagship-1: the world was rebuilt for each of some 240 asks a second).
    pub(super) fn place_planned_in(&self, world: &World, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit]) -> Plan {
        if let Some(pianist) = self.pianist.as_ref()
            && let Some((at, facing)) = self.free_stand_slots(builder.id, def, own, pianist).into_iter().next()
        {
            return Plan::Slot(def, at, facing);
        }
        let (centre, radius) = self.band_for(def, builder, own);
        self.place_in_world(world, def, builder, centre, radius)
    }

    /// Where a building of `def` goes within `radius` of `centre` (the player's place, or the kind's band): the
    /// layer's site, on land or on water as the kind's depths have it (§1.0: water is ground with its own
    /// footprints). When the layer finds no ground the engine is asked at the band's centre and the log says why.
    pub(super) fn place_in(&self, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit], centre: Vec3, radius: f32) -> Plan {
        self.place_in_world(&self.layout_world(own), def, builder, centre, radius)
    }

    fn place_in_world(&self, world: &World, def: UnitDefId, builder: &OwnUnit, centre: Vec3, radius: f32) -> Plan {
        let (Some(facts), Some(builder_facts)) = (self.facts_of(def), self.facts_of(builder.def)) else { return Plan::Near(def, centre) };
        let req = Request { facts, purpose: self.purpose_of(def), centre, radius, builder_at: builder.pos, builder: builder_facts };
        let started = std::time::Instant::now();
        match layout::choose(world, &req) {
            Ok(site) => Plan::Slot(def, site.at, site.facing),
            Err(why) => {
                let ms = started.elapsed().as_secs_f32() * 1000.0;
                *self.layout_refused.borrow_mut().entry(def).or_insert(0) += 1;
                let said = format!("no layout site for a {} for {}: {why}; the engine is asked at ({:.0}, {:.0})", self.name(def), self.actor_name(builder.id), centre.x, centre.z);
                if self.layout_refusal_said.borrow_mut().insert(def, said.clone()) != Some(said.clone()) {
                    eprintln!("[ai {}] {said}", self.ai());
                }
                self.dump_layout_failure(world, &req, builder, &why, ms);
                Plan::Near(def, centre)
            }
        }
    }

    /// A failing search saved whole for replay (`layout::Dump`, `site_replay`): the first `LAYOUT_DUMPS` of a game,
    /// at most `LAYOUT_DUMPS_PER_KIND` a kind, into `layout-fail-<ai>-<n>.bin` beside the record, when the record
    /// is on. The search is run once more, traced, for the file.
    fn dump_layout_failure(&self, world: &World, req: &Request, builder: &OwnUnit, why: &str, ms: f32) {
        if !crate::recorder::enabled() {
            return;
        }
        let n = {
            let mut dumped = self.layout_dumped.borrow_mut();
            let (total, a_kind) = &mut *dumped;
            let a_kind = a_kind.entry(req.facts.def).or_insert(0);
            if *total >= LAYOUT_DUMPS || *a_kind >= LAYOUT_DUMPS_PER_KIND {
                return;
            }
            *a_kind += 1;
            *total += 1;
            *total
        };
        let mut trace = Vec::new();
        let _ = layout::choose_traced(world, req, Some(&mut trace));
        // The last tick's frame: `stun_frame` is set by every tick's enemy reading (`briefing.rs`).
        let seconds = self.stun_frame / super::FRAMES_PER_SECOND;
        let label = format!("{} for {} at {}:{:02}", self.name(req.facts.def), self.actor_name(builder.id), seconds / 60, seconds % 60);
        let dump = layout::Dump::of(world, req, &self.world.hello.terrain, self.routes_passable(), label, why.to_string(), ms, trace);
        let path = crate::log_dir().join(format!("layout-fail-{}-{}.bin", self.ai(), n));
        match dump.write(&path) {
            Ok(()) => eprintln!("[ai {}] layout failure {n} dumped to {} ({ms:.0} ms)", self.ai(), path.display()),
            Err(e) => eprintln!("[ai {}] layout failure {n} not dumped: {e}", self.ai()),
        }
    }

    /// The site's reasons in the layer's words, for the hands' line: the choice run again at the chosen site.
    pub(super) fn site_words(&self, def: UnitDefId, builder: &OwnUnit, own: &[OwnUnit], at: Vec3) -> Option<String> {
        let (facts, builder_facts) = (self.facts_of(def)?, self.facts_of(builder.def)?);
        let world = self.layout_world(own);
        let req = Request { facts, purpose: self.purpose_of(def), centre: at, radius: 8.0, builder_at: builder.pos, builder: builder_facts };
        layout::choose(&world, &req).ok().map(|s| s.words).filter(|w| !w.is_empty())
    }

    /// An extractor's site on `spot` for `builder`: inside the patch, toward the builder (`layout::extractor_site`);
    /// the centre where an offset was refused before. The stand's builder is sited from the stand, wherever it is
    /// now: the opening's extractors are ordered as one queue before it has moved.
    pub(super) fn extractor_site(&self, spot: Vec3, builder: &OwnUnit) -> Vec3 {
        let hello = &self.world.hello;
        let Some(index) = hello.metal_spots.iter().position(|s| s.dist2d(spot) < 1.0) else { return spot };
        if self.centre_only.contains(&index) {
            return spot;
        }
        let squares = hello.metal_spot_squares.get(index).map_or(&[][..], |s| s.as_slice());
        let reach = self.world.def(builder.def).map_or(100.0, |d| d.build_distance.max(60.0));
        let from = self.stand.as_ref().filter(|(id, _)| *id == builder.id).map_or(builder.pos, |(_, s)| s.at);
        layout::extractor_site(spot, squares, hello.map.extractor_radius, from, reach)
    }

    /// What a plan builds and where the engine is asked to put it. None when the builder cannot build it.
    pub(super) fn build_site_for(&self, plan: &Plan, unit: &OwnUnit, own: &[OwnUnit], kit: &Kit) -> Option<(UnitDefId, BuildSite)> {
        // A spot takes the builder's own extractor: an advanced constructor builds no tier-1 one, so its is the
        // tier-2 (player-50-1 22:27: `extractor spot_3` refused to an advanced constructor rebuilding lost spots).
        let can = |def: UnitDefId| self.world.def(unit.def).is_some_and(|d| d.build_options.contains(&def));
        let planned_def = match plan {
            Plan::Extractor(_) if !can(kit.extractor) && can(kit.advanced_extractor) => kit.advanced_extractor,
            Plan::Extractor(_) => kit.extractor,
            Plan::Near(def_id, _) | Plan::Slot(def_id, ..) => *def_id,
        };
        if !can(planned_def) {
            return None;
        }
        Some(match *plan {
            // On a metal map the tile was chosen over the coverage and the ground: the shim searches nowhere and
            // the engine's answer is that site or none.
            Plan::Extractor(tile) if self.coverage.is_some() => (planned_def, BuildSite { near: tile, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: true }),
            // The game rejects an extractor that is not exactly on its spot (cmd_mex_denier.lua), and the shim
            // places extractors exactly at `near`, searching nowhere.
            Plan::Extractor(spot) => (planned_def, BuildSite { near: self.extractor_site(spot, unit), search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false }),
            // A tier-2 extractor over one of ours goes exactly where ours stands: the game upgrades in place and
            // refuses any other position on a held spot (low-1: three `armmoho spot_N` orders at the spots' centres,
            // 97-104 elmos from our extractors placed off centre, all "site bad"; the player: "upgrade path not working").
            Plan::Near(def_id, anchor)
                if self.world.def(def_id).is_some_and(|d| d.extracts_metal > 0.0)
                    && let Some(ours) = own.iter().filter(|u| kit.is_extractor(u.def) && u.def != def_id && u.pos.dist2d(anchor) < self.spot_occupied_radius()).min_by(|a, b| a.pos.dist2d(anchor).total_cmp(&b.pos.dist2d(anchor))) =>
            {
                (def_id, BuildSite { near: ours.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false })
            }
            // The engine's search with a gap of two squares, off every plant's lane, a plant facing the enemy: the
            // water, or the layer's fallback.
            Plan::Near(def_id, anchor) => (def_id, BuildSite { near: anchor, search_radius: WATER_SEARCH, min_dist: 2, placements: self.fallback_placements(def_id, anchor, own), exact: false }),
            // The layout's slot, exactly: the layer kept it off every apron, street and spot itself, and the engine's
            // answer is that site or none (then the refusal is remembered and the slot is passed over).
            Plan::Slot(def_id, at, facing) => (def_id, BuildSite { near: at, search_radius: SLOT_SEARCH, min_dist: 0, placements: vec![Placement { facing, keep_out: Vec::new() }], exact: true }),
        })
    }

    /// The engine's search for a site the layer found none for: kept off every plant's lane (ours standing or
    /// framed, allies'), `FALLBACK_LANE` out from the front, and for a plant the facings toward the enemy in order.
    /// Until 2026-11-04 the search went bare, and the apron the layer had kept empty was its nearest free ground:
    /// plant_28133's door filled with a solar, a storage, a turret and the advanced plant, and plant_5506 went up
    /// facing south into the turbine field (1793769060-human).
    fn fallback_placements(&self, def: UnitDefId, anchor: Vec3, own: &[OwnUnit]) -> Vec<Placement> {
        let lane = |d: UnitDefId, at: Vec3, facing: i32| self.facts_of(d).filter(|f| f.factory).map(|f| layout::lane(f.footprint, at, facing, FALLBACK_LANE, f.product_half));
        let keep_out: Vec<bot_protocol::Lane> = own
            .iter()
            .filter_map(|u| lane(u.def, u.pos, u.facing))
            .chain(self.allies.iter().filter_map(|a| lane(a.def, a.pos, a.facing)))
            .collect();
        let facings: Vec<i32> = if self.world.is_factory_def(def) { layout::facings_toward(anchor, self.enemy_base(self.home)).to_vec() } else { vec![0] };
        facings.into_iter().map(|facing| Placement { facing, keep_out: keep_out.clone() }).collect()
    }

    pub(super) fn spot_occupied_radius(&self) -> f32 {
        self.world.hello.map.spot_radius()
    }

    /// On a metal map, the extractors' claims as the engine holds them: every extractor standing or going up, ours,
    /// an ally's or his as far as seen, claims the squares within the radius at its depth (the design §1.3).
    pub(super) fn refresh_coverage(&mut self, tick: &bot_protocol::Tick) {
        let Some(mut coverage) = self.coverage.take() else { return };
        coverage.clear_claims();
        let depth = |def: UnitDefId| self.world.def(def).map(|d| d.extracts_metal).filter(|e| *e > 0.0);
        for u in &tick.snapshot.own_units {
            if let Some(d) = depth(u.def) {
                coverage.claim(u.pos, d);
            }
        }
        for a in &self.allies {
            if let Some(d) = depth(a.def) {
                coverage.claim(a.pos, d);
            }
        }
        for (def, pos, _) in self.enemy_buildings.values() {
            if let Some(d) = depth(*def) {
                coverage.claim(*pos, d);
            }
        }
        self.coverage = Some(coverage);
    }

    /// On a metal map, the tile for a new extractor of `builder`'s: the most unclaimed metal nearest the builder
    /// within `reach` of `anchor` (`Coverage::best_site`), where the ground stands, nothing standing or ordered
    /// covers it, no plant's apron or reservation holds it, the engine has not refused it lately, and no building
    /// of his stands within 200. None off a metal map or when nothing fits.
    pub(super) fn tile_site(&self, builder: &OwnUnit, own: &[OwnUnit], pianist: &super::pianist::Pianist, anchor: Vec3, reach: f32, frame: i32) -> Option<Vec3> {
        let coverage = self.coverage.as_ref()?;
        let kit = self.kit?;
        let def = if self.world.def(builder.def).is_some_and(|d| d.build_options.contains(&kit.extractor)) { kit.extractor } else { kit.advanced_extractor };
        let d = self.world.def(def)?;
        let facts = Facts::of(d);
        let world = self.layout_world_for(own, Some(pianist));
        let ordered: Vec<layout::Rect> = pianist
            .tasks
            .values()
            .chain(pianist.queued_tasks().map(|(_, t)| t))
            .filter_map(|t| match t {
                super::pianist::Task::Build { def: td, near, .. } if self.world.def(*td).is_some_and(|d| d.extracts_metal > 0.0) => Some(facts.footprint.rect(*near)),
                _ => None,
            })
            .collect();
        let refused = |p: Vec3| pianist.refused_sites.iter().any(|(rd, at, until)| *rd == def && *until > frame && at.dist2d(p) < TILE_REFUSED);
        let ok = |p: Vec3, rect: &layout::Rect| {
            !refused(p)
                && world.ground.stands(facts.footprint, p, &facts)
                && !world.standing.iter().any(|s| s.rect.overlaps(rect))
                && !world.plants.iter().any(|pl| pl.apron().overlaps(rect))
                && !world.reservations.iter().any(|r| r.rect.overlaps(rect))
                && !ordered.iter().any(|o| o.overlaps(rect))
                && !self.enemy_buildings.values().any(|(_, pos, _)| pos.dist2d(p) < HIS_BUILDING_OFF)
        };
        coverage.best_site(facts.footprint, builder.pos, anchor, reach, d.extracts_metal, &ok).map(|(at, _)| at)
    }

    /// The reach round a named place an `extractor <place>` step or a `build_<extractor>_<place>` move sites within:
    /// a cell's half-diagonal, a metal field's extent from its centre, else `BAND_PLACE` round a point.
    pub(super) fn place_reach(&self, place: &super::pianist::picture::Place) -> f32 {
        if place.is_cell() {
            let (w, h) = self.world.cells.cell_size();
            return (w * w + h * h).sqrt() / 2.0;
        }
        if let Some(field) = self.world.hello.metal_fields.iter().find(|f| f.name == place.name) {
            let [west, north, east, south] = field.bounds;
            return (field.centre.x - west).max(east - field.centre.x).hypot((field.centre.z - north).max(south - field.centre.z));
        }
        BAND_PLACE
    }

    /// The street of a plant of ours, for the report: open, or closed for its widest product and by what.
    pub(super) fn street_words(&self, plant: &OwnUnit, own: &[OwnUnit]) -> Option<String> {
        let world = self.layout_world(own);
        let p = world.plants.iter().find(|p| p.ours && p.centre.dist2d(plant.pos) < 1.0)?;
        let blocks: Vec<layout::Rect> = world.standing.iter().map(|s| if s.ours { s.rect } else { s.rect.grown(8.0) }).collect();
        let ours: Vec<usize> = world.standing.iter().enumerate().filter(|(_, s)| s.ours && s.rect != p.rect()).map(|(i, _)| i).collect();
        let widest = match self.world.def_named(&p.widest) {
            Some(def) => format!("its {}", self.short_words(def)),
            None => "its units".to_string(),
        };
        Some(match layout::street_state(&world.ground, p, &blocks, &ours, &*world.open) {
            layout::StreetState::Open => "open for everything it makes".to_string(),
            layout::StreetState::Closed { opener: Some(i) } => {
                let s = &world.standing[i];
                if s.ours { format!("closed for {widest} by {}; reclaim it to open", s.name) } else { format!("closed for {widest} by {}", s.name) }
            }
            layout::StreetState::Closed { opener: None } => {
                let allied = world.standing.iter().filter(|s| !s.ours && p.concerns(&s.rect)).map(|s| s.name.clone()).next();
                match allied {
                    Some(name) => format!("closed for {widest}: no one building of ours opens it; {name} stands in the way"),
                    None => format!("closed for {widest} by the ground and what stands: no one building of ours opens it"),
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::pianist::fixtures::{brain_of, own};
    use crate::brain::roster::{Kit, ROSTERS};

    /// A spot takes the builder's own extractor: the tier-2 one for an advanced constructor, which builds no other.
    #[test]
    fn an_advanced_constructor_builds_its_own_extractor_at_a_spot() {
        let mut brain = brain_of(&["armmex", "armmoho", "armck", "armack"]);
        brain.world.hello.unit_defs[2].build_options = vec![UnitDefId(1)];
        brain.world.hello.unit_defs[3].build_options = vec![UnitDefId(2)];
        let (mex, moho) = (UnitDefId(1), UnitDefId(2));
        let kit = Kit { commander: mex, extractor: mex, converter: mex, lab: mex, turret: mex, constructor: mex, vehicle_constructor: mex, advanced_constructor: mex, advanced_extractor: moho, raider: mex, line: mex, resurrector: mex, roster: &ROSTERS[0] };
        let spot = Plan::Extractor(Vec3 { x: 1000.0, y: 0.0, z: 1000.0 });
        let at = Vec3 { x: 900.0, y: 0.0, z: 1000.0 };
        assert_eq!(brain.build_site_for(&spot, &own(1, 3, at), &[], &kit).map(|(def, _)| def), Some(mex), "a constructor");
        assert_eq!(brain.build_site_for(&spot, &own(2, 4, at), &[], &kit).map(|(def, _)| def), Some(moho), "an advanced constructor");
    }

    /// The layer places for the hands: on flat ground with a plant facing east and one solar standing, a second
    /// solar for the commander goes touching the first, out of the plant's apron, as an exact slot; the plant's
    /// street reads open, and reads closed by a solar once solars ring it.
    #[test]
    fn a_solar_joins_its_bank_out_of_the_apron_and_the_street_is_read() {
        use crate::brain::pianist::Pianist;
        use bot_protocol::Terrain;
        let mut brain = brain_of(&["armcom", "armsolar", "armvp"]);
        let (com, solar, vp) = (UnitDefId(1), UnitDefId(2), UnitDefId(3));
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].build_distance = 145.0;
            defs[0].build_speed = 300.0;
            defs[0].build_options = vec![solar, vp];
            defs[1].footprint = (10, 10);
            defs[1].energy_make = 20.0;
            defs[2].footprint = (12, 12);
            defs[2].build_options = vec![com];
            let (w, h) = (512u32, 384u32);
            brain.world.hello.terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() };
        }
        brain.home = Vec3 { x: 2000.0, y: 0.0, z: 2000.0 };
        brain.pianist = Some(Pianist::new(false, &std::env::temp_dir(), 0).expect("a pianist"));
        let commander = own(10, 1, Vec3 { x: 2008.0, y: 0.0, z: 2104.0 });
        let plant = OwnUnit { facing: 1, ..own(11, 3, Vec3 { x: 2000.0, y: 0.0, z: 2000.0 }) };
        let first = own(12, 2, Vec3 { x: 2008.0, y: 0.0, z: 2168.0 });
        let ours = vec![commander.clone(), plant.clone(), first.clone()];
        let plan = brain.place_planned(solar, &commander, &ours);
        let Plan::Slot(def, at, _) = plan else { panic!("the layer's slot") };
        assert_eq!(def, solar);
        let rect = layout::Footprint { x: 10, z: 10 }.rect(at);
        assert!(rect.touches(&layout::Footprint { x: 10, z: 10 }.rect(first.pos)), "touching the first solar: {at:?}");
        let apron = brain.plant_of(vp, plant.pos, 1, true, String::new()).unwrap().apron();
        assert!(!apron.overlaps(&rect), "in the apron: {at:?}");
        assert_eq!(brain.street_words(&plant, &ours).as_deref(), Some("open for everything it makes"));
        // Solars ringing the plant close its street; the one before the door opens it.
        let mut ringed = ours.clone();
        let mut id = 20;
        for k in -3..=3 {
            let o = 80.0 * k as f32;
            for p in [(2248.0, 2000.0 + o), (1752.0, 2000.0 + o), (2000.0 + o, 1752.0), (2000.0 + o, 2248.0)] {
                ringed.push(own(id, 2, Vec3 { x: p.0, y: 0.0, z: p.1 }));
                id += 1;
            }
        }
        let words = brain.street_words(&plant, &ringed).unwrap();
        assert!(words.starts_with("closed for") && words.contains("armsolar_") && words.contains("reclaim it to open"), "{words}");
    }

    /// Allies' units are not walls, their buildings are; and the layer's fallback asks the engine off every plant's
    /// lane, a plant facing the enemy first (1793769060-human: an Apex soldier in its lab's door read as a wall,
    /// and the bare search filled our own plant's door).
    #[test]
    fn allied_units_are_not_walls_and_the_fallback_keeps_off_the_lanes() {
        use bot_protocol::{AllyUnit, Terrain, UnitId};
        let mut brain = brain_of(&["armcom", "armsolar", "armvp", "armflash"]);
        let (com, solar, vp, flash) = (UnitDefId(1), UnitDefId(2), UnitDefId(3), UnitDefId(4));
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].build_distance = 145.0;
            defs[0].build_speed = 300.0;
            defs[0].build_options = vec![solar, vp];
            defs[1].footprint = (10, 10);
            defs[1].energy_make = 20.0;
            defs[2].footprint = (12, 12);
            defs[2].build_options = vec![flash];
            let (w, h) = (512u32, 384u32);
            brain.world.hello.terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() };
        }
        brain.home = Vec3 { x: 2000.0, y: 0.0, z: 2000.0 };
        let commander = own(10, 1, Vec3 { x: 2008.0, y: 0.0, z: 2104.0 });
        let plant = OwnUnit { facing: 1, ..own(11, 3, Vec3 { x: 2000.0, y: 0.0, z: 2000.0 }) };
        let ours = vec![commander.clone(), plant.clone()];
        let ally = |id: i32, def: UnitDefId, x: f32, z: f32| AllyUnit { id: UnitId(id), def, pos: Vec3 { x, y: 0.0, z }, team: 7, being_built: false, facing: 0 };
        brain.allies = vec![ally(50, flash, 2600.0, 2000.0), ally(51, solar, 2600.0, 2400.0)];
        let world = brain.layout_world(&ours);
        let names: Vec<&str> = world.standing.iter().map(|s| s.name.as_str()).collect();
        assert!(names.iter().any(|n| n.starts_with("an allied armsolar")), "{names:?}");
        assert!(!names.iter().any(|n| n.starts_with("an allied armflash")), "a unit is no wall: {names:?}");
        assert_eq!(world.builders.len(), 1, "the commander keeps a way out");
        // The fallback: a solar's site off the plant's lane; a plant's with the facings toward the enemy first.
        let kit = Kit { commander: com, extractor: com, converter: com, lab: vp, turret: com, constructor: com, vehicle_constructor: com, advanced_constructor: com, advanced_extractor: com, raider: flash, line: flash, resurrector: com, roster: &ROSTERS[0] };
        let anchor = Vec3 { x: 2200.0, y: 0.0, z: 2000.0 };
        let (_, site) = brain.build_site_for(&Plan::Near(solar, anchor), &commander, &ours, &kit).expect("a site");
        assert_eq!(site.placements.len(), 1);
        assert_eq!(site.placements[0].facing, 0);
        assert!(site.placements[0].keep_out.iter().any(|l| l.contains(Vec3 { x: 2200.0, y: 0.0, z: 2000.0 })), "the plant's lane east of its front: {:?}", site.placements[0].keep_out);
        assert!(!site.placements[0].keep_out.iter().any(|l| l.contains(Vec3 { x: 1800.0, y: 0.0, z: 2000.0 })), "behind the plant is free");
        let (_, site) = brain.build_site_for(&Plan::Near(vp, anchor), &commander, &ours, &kit).expect("a site");
        assert_eq!(site.placements.len(), 4);
        assert_eq!(site.placements[0].facing, layout::facings_toward(anchor, brain.enemy_base(brain.home))[0]);
    }

    /// The kinds' purposes and bands: a solar banks without a cap, a converter and a wind turbine to six, a
    /// construction turret reaches pads, a plant is a plant, a fusion a frame, a turret stands at a place; a silo's
    /// band is behind home, a solar's beside its builder.
    #[test]
    fn kinds_have_their_purposes_and_bands() {
        let mut brain = brain_of(&["armsolar", "armmakr", "armwin", "armnanotc", "armvp", "armfus", "armllt", "armsilo", "armcom"]);
        {
            let defs = &mut brain.world.hello.unit_defs;
            defs[0].energy_make = 20.0;
            defs[1].converter = Some(bot_protocol::Converter { capacity: 70.0, efficiency: 0.014 });
            defs[2].wind_cap = 25.0;
            defs[3].build_speed = 200.0;
            defs[4].build_options = vec![UnitDefId(9)];
            defs[5].energy_make = 1000.0;
            defs[8].build_speed = 300.0;
            defs[8].build_options = vec![UnitDefId(1)];
        }
        brain.home = Vec3 { x: 1000.0, y: 0.0, z: 1000.0 };
        let p = |i: i32| brain.purpose_of(UnitDefId(i));
        assert_eq!(p(1), Purpose::Generator { cap: Some(BANK_CAP_GENERATOR) });
        assert_eq!(p(2), Purpose::Generator { cap: Some(BANK_CAP) });
        assert_eq!(p(3), Purpose::Generator { cap: Some(BANK_CAP) });
        assert_eq!(p(4), Purpose::Turret);
        assert_eq!(p(5), Purpose::Plant);
        assert_eq!(p(6), Purpose::Frame);
        assert_eq!(p(7), Purpose::Place);
        let builder = own(1, 9, Vec3 { x: 1200.0, y: 0.0, z: 1000.0 });
        let (centre, radius) = brain.band_for(UnitDefId(8), &builder, std::slice::from_ref(&builder));
        let enemy = brain.enemy_base(brain.home);
        assert!(centre.dist2d(enemy) > brain.home.dist2d(enemy), "the silo's band is behind home: {centre:?}");
        assert_eq!(radius, BAND_BACK_FIELD);
        let (centre, radius) = brain.band_for(UnitDefId(1), &builder, std::slice::from_ref(&builder));
        assert_eq!((centre, radius), (builder.pos, BAND_GENERATOR));
    }
}
