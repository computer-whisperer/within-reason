//! D-GROUND (2026-11-06): whether a place can be walked to, and how far the walk is. Until the map bench of that day
//! (`README.md`, "The maps") every distance in the bot was a straight line and no rule asked whether its place could
//! be reached: on Flooded Valley six constructors stood at the shore with extractor orders all game (dist156), on
//! Quicksilver a body of 160 growing to 375 stood eleven minutes 750 from a target its formation had no route to
//! (dist149 game 05), and on Hooked the commander walked 75 s to its second extractor before a solar (dist155).
//! The shim's terrain and the shared `terrain` crate had the answer all along (the flagship's travel times).

use std::sync::Arc;

use bot_protocol::{MoveKind, UnitDefId, Vec3};
use terrain::Field;

use crate::Distilled;

/// A movement class as a key: its kind, steepest slope, depth and slope law.
type Class = (MoveKind, i32, i32, i32);

/// The walking fields: one from home a movement class over the shim's cells (16 elmos), found when first asked (a
/// search over the whole map, tens of milliseconds once), and each class's ground at a quarter of that resolution
/// for the fields a builder asks from where it stands every time it chooses a site (a millisecond or two each).
#[derive(Default)]
pub(crate) struct Ground {
    costs: std::cell::RefCell<Vec<(Class, Arc<Vec<u32>>)>>,
    home: std::cell::RefCell<Vec<(Class, Option<Arc<Field>>)>>,
    coarse: std::cell::RefCell<Vec<(Class, Arc<Vec<u32>>)>>,
}

/// The coarse ground's cell in the shim's cells: one of its cells is passable when half of these squared are, at
/// their mean cost. A wall thinner than half a coarse cell leaks, which prices a walk as the straight line did;
/// whether a place can be reached at all is always the fine field from home's answer.
const COARSE: usize = 4;

impl Distilled {
    /// A type's movement class; none under `ground=off`, so every question below gets the straight line's answer.
    fn class_of(&self, def: UnitDefId) -> Option<(Class, bot_protocol::MoveClass)> {
        if !self.options.ground {
            return None;
        }
        let class = self.def(def)?.move_class?;
        Some(((class.kind, (class.max_slope * 255.0).round() as i32, class.depth as i32, (class.slope_mod * 16.0).round() as i32), class))
    }

    /// The cost of crossing each cell for a class (`terrain::costs`), found once a class. `None` without terrain data.
    fn class_costs(&self, key: Class, class: bot_protocol::MoveClass) -> Option<Arc<Vec<u32>>> {
        let terrain = &self.hello.terrain;
        if terrain.cell <= 0.0 || terrain.heights.is_empty() {
            return None;
        }
        if let Some((_, costs)) = self.ground.costs.borrow().iter().find(|(k, _)| *k == key) {
            return Some(costs.clone());
        }
        let costs = Arc::new(terrain::costs(terrain, class));
        self.ground.costs.borrow_mut().push((key, costs.clone()));
        Some(costs)
    }

    /// The field of walking distances from home for this type's movement class, in elmos at full speed (a slope
    /// costs by the engine's law). `None` for a type that does not walk, or without terrain data.
    fn home_field(&self, def: UnitDefId) -> Option<Arc<Field>> {
        let (key, class) = self.class_of(def)?;
        if let Some((_, field)) = self.ground.home.borrow().iter().find(|(k, _)| *k == key) {
            return field.clone();
        }
        let began = std::time::Instant::now();
        let field = self.class_costs(key, class).and_then(|costs| Field::from_costs(&self.hello.terrain, &costs, &[self.home])).map(Arc::new);
        let spots = &self.hello.metal_spots;
        let reached = field.as_ref().map_or(spots.len(), |f| spots.iter().filter(|s| f.distance(**s).is_some()).count());
        let his_start = field.as_ref().is_none_or(|f| f.distance(self.enemy_home).is_some());
        eprintln!(
            "[distilled ai={}] f={} ground: {:?} (slope {}, depth {}) walks from home to {reached} of {} spots, his start {}; {} ms{}",
            self.hello.ai_id,
            self.frame,
            key.0,
            key.1,
            key.2,
            spots.len(),
            if his_start { "reached" } else { "not reached" },
            began.elapsed().as_millis(),
            if field.is_none() { " (no field)" } else { "" }
        );
        // `WITHIN_REASON_DISTILLED_SKETCH=N`: the class's ground as N columns of text (`terrain::sketch`), for a reader
        // of the log who cannot see the map.
        if let (Some(f), Some(size)) = (&field, std::env::var("WITHIN_REASON_DISTILLED_SKETCH").ok().and_then(|n| n.parse::<usize>().ok())) {
            for row in terrain::sketch(&self.hello.terrain, &terrain::passable(&self.hello.terrain, class), f, size) {
                eprintln!("[distilled ai={}] sketch {:?} {row}", self.hello.ai_id, key.0);
            }
        }
        self.ground.home.borrow_mut().push((key, field.clone()));
        field
    }

    /// The walking distances from `from` for this type's movement class over the coarse ground: what a builder
    /// choosing among sites asks, for the order of them. `None` for a type that does not walk, without terrain data,
    /// or from a place off its ground.
    pub(crate) fn field_from(&self, def: UnitDefId, from: Vec3) -> Option<Field> {
        let (key, class) = self.class_of(def)?;
        let terrain = &self.hello.terrain;
        let (width, height) = (terrain.width as usize, terrain.height as usize);
        let (coarse_width, coarse_height) = (width.div_ceil(COARSE), height.div_ceil(COARSE));
        let known = self.ground.coarse.borrow().iter().find(|(k, _)| *k == key).map(|(_, costs)| costs.clone());
        let coarse = match known {
            Some(costs) => costs,
            None => {
                let fine = self.class_costs(key, class)?;
                let mut costs = vec![terrain::IMPASSABLE; coarse_width * coarse_height];
                for (index, cost) in costs.iter_mut().enumerate() {
                    let (cx, cz) = (index % coarse_width, index / coarse_width);
                    let (mut open, mut sum, mut cells) = (0u64, 0u64, 0u64);
                    for z in cz * COARSE..((cz + 1) * COARSE).min(height) {
                        for x in cx * COARSE..((cx + 1) * COARSE).min(width) {
                            cells += 1;
                            let c = fine[z * width + x];
                            if c != terrain::IMPASSABLE {
                                open += 1;
                                sum += u64::from(c);
                            }
                        }
                    }
                    if open > 0 && 2 * open >= cells {
                        *cost = (sum / open) as u32;
                    }
                }
                let costs = Arc::new(costs);
                self.ground.coarse.borrow_mut().push((key, costs.clone()));
                costs
            }
        };
        Field::over(terrain.cell * COARSE as f32, coarse_width, coarse_height, &coarse, &[from])
    }

    /// The walk from home to `at` for a unit of this type (to the ground it can stand on nearest `at`, a few cells
    /// off at most: spots and buildings sit on shores and ledges): `None` when it cannot get there. Without terrain
    /// data, or for a type with no movement class, the straight line.
    pub(crate) fn walk_from_home(&self, def: Option<UnitDefId>, at: Vec3) -> Option<f32> {
        match def.and_then(|d| self.home_field(d)) {
            Some(field) => field.distance(at),
            None => Some(self.home.dist2d(at)),
        }
    }

    /// Whether the army's line unit can get to `at` from home.
    pub(crate) fn army_reaches(&self, at: Vec3) -> bool {
        self.walk_from_home(self.kit.tank.or(self.kit.raider), at).is_some()
    }
}

/// D-KIT (2026-11-06): the plant's product that takes a place in the kit is one that can walk the map. A constructor
/// or line unit of the kit that reaches under this share of the metal spots the plant's best product of its kind
/// reaches gives its place to that product: on Flooded Valley the vehicle plant's constructor and tank reach 2 of
/// 16 spots and not his start, and the same plant makes the amphibious Beaver and Pincer (Muskrat and Garpike),
/// which reach all of them (dist156, dist168: two extractors and 7 metal a second all game, 0-11-5). A scout,
/// raider or artillery piece that cannot follow the line unit is left out. At 0.9 the kit changed on Quicksilver
/// too, where the vehicles reach 38 of 44 spots and his start: the amphibious pair is slower and dearer, and the
/// scouts, raiders and artillery went with it.
const KIT_REACH: f32 = 0.6;

impl Distilled {
    /// How many of the map's metal spots this type walks to from home.
    fn spots_reached(&self, def: UnitDefId) -> usize {
        self.hello.metal_spots.iter().filter(|s| self.walk_from_home(Some(def), **s).is_some()).count()
    }

    /// D-KIT: the kit's constructor and soldiers fitted to the ground, once at the start.
    pub(crate) fn fit_kit(&mut self) {
        if !self.options.ground || self.hello.metal_spots.is_empty() {
            return;
        }
        let Some(plant) = self.kit.plant.and_then(|p| self.def(p)) else { return };
        let products: Vec<(UnitDefId, usize, f32, bool, bool)> = plant
            .build_options
            .iter()
            .filter_map(|id| self.def(*id))
            .filter(|d| d.speed > 0.0 && d.move_class.is_some())
            .map(|d| (d.id, self.spots_reached(d.id), d.metal_cost, !d.build_options.is_empty(), d.weapon_count > 0))
            .collect();
        let short = |have: usize, best: usize| (have as f32) < KIT_REACH * best as f32;
        let mut changes: Vec<String> = Vec::new();
        // The constructor: the cheapest of the plant's builders that reach the most.
        if let Some(current) = self.kit.constructor {
            let have = self.spots_reached(current);
            let best = products.iter().filter(|p| p.3).max_by(|a, b| a.1.cmp(&b.1).then(b.2.total_cmp(&a.2)));
            if let Some(&(id, reached, ..)) = best.filter(|p| p.0 != current && short(have, p.1)) {
                changes.push(format!("constructor {} ({have} spots) -> {} ({reached})", self.name_of(current), self.name_of(id)));
                self.kit.constructor = Some(id);
            }
        }
        // The line unit: of the plant's soldiers that reach the most, the one nearest the tank's price.
        if let Some(current) = self.kit.tank {
            let (have, price) = (self.spots_reached(current), self.def(current).map_or(0.0, |d| d.metal_cost));
            let most = products.iter().filter(|p| p.4 && !p.3).map(|p| p.1).max().unwrap_or(0);
            let best = products.iter().filter(|p| p.4 && !p.3 && p.1 == most).min_by(|a, b| (a.2 - price).abs().total_cmp(&(b.2 - price).abs()));
            if let Some(&(id, reached, ..)) = best.filter(|p| p.0 != current && short(have, p.1)) {
                changes.push(format!("line unit {} ({have} spots) -> {} ({reached})", self.name_of(current), self.name_of(id)));
                self.kit.tank = Some(id);
            }
        }
        // What cannot follow the line unit is left out.
        let line = self.kit.tank.map_or(0, |t| self.spots_reached(t));
        for (name, slot) in [("scout", self.kit.scout), ("raider", self.kit.raider), ("artillery", self.kit.artillery)] {
            if let Some(def) = slot.filter(|d| short(self.spots_reached(*d), line)) {
                changes.push(format!("no {name} ({} reaches {} spots)", self.name_of(def), self.spots_reached(def)));
                match name {
                    "scout" => self.kit.scout = None,
                    "raider" => self.kit.raider = None,
                    _ => self.kit.artillery = None,
                }
            }
        }
        if !changes.is_empty() {
            eprintln!("[distilled ai={}] f={} kit by the ground: {}", self.hello.ai_id, self.frame, changes.join("; "));
        }
    }
}
