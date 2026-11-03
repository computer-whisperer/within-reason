//! D-LAYOUT: where a building goes, by the shared base layout engine (`crates/layout`, the flagship's layer since
//! the base layout arc of 2026-10-29: constraints, then bands; `docs/design/2026-10-29-base-layout.md` §1.1). The
//! world is what stands (ours finished and framed, every order of ours with a site chosen here, allies' with a
//! square of margin, his known buildings), the plants with their aprons and streets (so nothing of ours closes a
//! street), our construction turrets as build power and our frames; a kind is asked for within its own band (a
//! generator a short walk from its builder, a turret within its reach of a pad, a plant and a frame within a few
//! seconds of the builder, the back field for the reactors, converters and silos). The layer's answer is asked of
//! the engine exactly (`BuildSite::exact`, the slot or none); when it finds no ground the old site search stands
//! (`economy.rs` `order`). On a metal map the extractors are the coverage's (`economy.rs` `free_site`), not this.
//!
//! 2026-11-03, the user: "update this to use the new base layout engine if we haven't already". Before it the bot's
//! generators went to the engine's search round the back field and crowded the yard (distilled-t2b-30-hard), its
//! turrets to one of four points round a plant, its plants to a point 350 toward the enemy with a 900 search.

use bot_protocol::{OwnUnit, Tick, UnitDefId, Vec3};
use layout::{Facts, Footprint, Frame, Ground, Plant, Purpose, Request, Standing, Worker, World};

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
/// Banks of converters and wind turbines stop at this many, solars and the other generators at 8 (the flagship's
/// caps: the people's clusters of 7 to 10; a longer bank is a wall the builders walk round).
const BANK_CAP: usize = 6;
const BANK_CAP_GENERATOR: usize = 8;
/// The engine is asked for the slot itself: this much search round it, no gap.
const SLOT_SEARCH: f32 = 24.0;

impl Distilled {
    /// What the layer knows of a kind: its footprint and, for a factory, its street's half-width (its widest product's).
    /// Its products' ground is not known here (the bot keeps no walk sets), so the street is judged on the world's
    /// `open`, which is everywhere.
    pub(crate) fn facts_of(&self, def: UnitDefId) -> Option<Facts> {
        let d = self.def(def)?;
        let facts = Facts::of(d);
        if !facts.factory {
            return Some(facts);
        }
        let products = d.build_options.iter().filter_map(|p| self.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some());
        Some(facts.with_products(Facts::product_half_of(products.map(|p| Footprint { x: p.footprint.0, z: p.footprint.1 })), None))
    }

    /// What a kind is placed for (the design's level 1), as the flagship reads it.
    pub(crate) fn purpose_of(&self, def: UnitDefId) -> Purpose {
        let Some(d) = self.def(def) else {
            return Purpose::Frame;
        };
        let factory = d.speed == 0.0 && !d.build_options.is_empty();
        let static_builder = d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty();
        if factory {
            Purpose::Plant
        } else if static_builder {
            Purpose::Turret
        } else if (d.converter.is_some() && d.metal_cost < 500.0) || d.wind_cap > 0.0 {
            Purpose::Generator { cap: Some(BANK_CAP) }
        } else if (d.energy_make > 0.0 || d.energy_upkeep < 0.0 || d.tidal_make > 0.0) && d.metal_cost < 500.0 {
            Purpose::Generator { cap: Some(BANK_CAP_GENERATOR) }
        } else if Some(def) == self.kit.aa_turret || Some(def) == self.kit.sentry {
            Purpose::Place
        } else {
            Purpose::Frame
        }
    }

    /// A plant for the street rule: its footprint, facing and its widest product's half-width.
    fn plant_of(&self, def: UnitDefId, at: Vec3, facing: i32, ours: bool, name: String) -> Option<Plant> {
        let f = self.facts_of(def)?;
        let d = self.def(def)?;
        let widest = d.build_options.iter().filter_map(|p| self.def(*p)).filter(|p| p.speed > 0.0 && p.move_class.is_some()).max_by_key(|p| p.footprint.0.max(p.footprint.1));
        Some(Plant { centre: at, footprint: f.footprint, facing, product_half: f.product_half, ours, name, widest: widest.map_or(String::new(), |p| p.name.clone()), passable: None })
    }

    /// The world the layer chooses over this tick: what stands (ours, finished or framed; our orders with a site
    /// chosen here and not yet begun; allies'; his known buildings), the plants (ours, ordered and allies'), our
    /// construction turrets as build power, our frames.
    pub(crate) fn layout_world(&self, tick: &Tick) -> World<'_> {
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
        World {
            ground: Ground::new(&hello.terrain, hello.map.width, hello.map.height),
            spots: &hello.metal_spots,
            spot_keep_out: SPOT_KEEPOUT,
            standing,
            plants,
            workers,
            frames,
            frame_slots: Vec::new(),
            reservations: Vec::new(),
            lattice: None,
            refused: Vec::new(),
            home: self.home,
            toward: self.enemy_home,
            open: Box::new(|_| true),
            walks: Default::default(),
            standing_near: Default::default(),
            spots_near: Default::default(),
        }
    }

    /// The kind a job places and its band: where its site is looked for and how far round it.
    pub(crate) fn band_for(&self, job: Job, unit: &OwnUnit) -> Option<(UnitDefId, Vec3, f32)> {
        let def = job.def(&self.kit)?;
        let back = self.toward_enemy(self.home, -350.0);
        let nearest_plant = || {
            self.own
                .values()
                .filter(|u| Some(u.def) == self.kit.plant || Some(u.def) == self.kit.advanced_plant || Some(u.def) == self.kit.air_plant)
                .min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)))
                .map_or(unit.pos, |p| p.pos)
        };
        Some(match job {
            Job::Solar | Job::AdvancedSolar => (def, unit.pos, BAND_GENERATOR),
            Job::Turret(target) => (def, self.own.get(&target).map_or_else(nearest_plant, |t| t.pos), BAND_TURRET),
            Job::Plant | Job::AdvancedPlant => (def, self.toward_enemy(self.home, 350.0), BAND_PLANT),
            Job::AirPlant => (def, self.air_plant_site(), BAND_PLANT),
            Job::Fusion | Job::AdvancedFusion | Job::Silo | Job::AdvancedConverter => (def, back, BAND_FRAME),
            Job::Converter => (def, back, BAND_GENERATOR),
            Job::AntiAir => (def, self.anti_air_site(), BAND_PLACE),
            Job::Extractor(_) | Job::Moho(_) | Job::Assist | Job::AirAssist | Job::Help(_) => {
                return None;
            }
        })
    }

    /// The layer's site for this job and builder: the slot and its facing, or why there is none.
    pub(crate) fn layout_site(&self, world: &World, job: Job, unit: &OwnUnit) -> Result<(Vec3, i32), String> {
        let (def, centre, radius) = self.band_for(job, unit).ok_or_else(|| "not a layout kind".to_string())?;
        let (facts, builder) = (self.facts_of(def).ok_or("no facts")?, self.facts_of(unit.def).ok_or("no builder facts")?);
        let req = Request { facts, purpose: self.purpose_of(def), centre, radius, builder_at: unit.pos, builder };
        layout::choose(world, &req).map(|site| (site.at, site.facing))
    }

    /// The engine asked for the layer's slot itself: that site or none (the layer kept it off every apron, street
    /// and spot).
    pub(crate) fn slot_site(at: Vec3, facing: i32) -> bot_protocol::BuildSite {
        bot_protocol::BuildSite { near: at, search_radius: SLOT_SEARCH, min_dist: 0, placements: vec![bot_protocol::Placement { facing, keep_out: Vec::new() }], exact: true }
    }

    fn name_of(&self, def: UnitDefId) -> String {
        self.def(def).map_or_else(|| format!("def {}", def.0), |d| d.name.clone())
    }
}
