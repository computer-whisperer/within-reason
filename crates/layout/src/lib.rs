//! Where a building goes: the layout layer shared by the flagship (`crates/bot`) and the heuristic bots
//! (`docs/design/2026-10-29-base-layout.md`). It knows the build grid and footprints, the ground a footprint may
//! stand on (F-map-build), an extractor's site inside its patch, a factory's exit lane, and the commander's stand:
//! the opening's slots solved once at the start, from the start position, the spots and the terrain, so that the
//! opening list goes to the engine as one queue and the commander never idles between its first structures (the
//! study, `docs/studies/2026-10-29-base-layout.md` §3 and §5: the people's 0.6 s from a structure finishing to the
//! next order and one or two stands up to the plant, against our 3.5 / 1.0 / 0.8 / 5.8 s and three). No engine, no
//! Jev: the ground is the terrain the host sends, the facts are the definitions.

mod block;
mod choose;
mod coverage;
mod dump;
mod farm;
mod street;

use std::collections::HashMap;

use bot_protocol::{Lane, Terrain, UnitDefId, UnitDefInfo, Vec3};
use serde::{Deserialize, Serialize};

pub use block::{Block, DENSE_PITCH, Lattice, SPARSE_PITCH, Wants as BlockWants, lay as lay_block};
pub use farm::{FARM_EXTENT, Farm, ROOM_HIGH, ROOM_LOW, cells_of as farm_cells_of, footprints as farm_footprints};
pub use choose::{Frame, Purpose, Request, Reservation, Reserved, Site, Standing, Worker, World, choose, choose_traced, distance_to};
pub use coverage::Coverage;
pub use dump::{Dump, PassDump};
pub use street::{APRON, Blocked, Builder, Flood, Plant, STREET_REACH, StreetState, WayOut, street_open, street_state, way_out, way_out_state};

/// A build square (`SQUARE_SIZE`): footprints count them.
pub const SQUARE: f32 = 8.0;
/// The build grid a building's centre snaps to (`CGameHelper::Pos2BuildPos`).
pub const GRID: f32 = 16.0;
/// The engine's height tolerance for a building whose definition does not say: `maxslope` 10 in 276 of BAR's 543
/// definitions, times the game's 1.5, so 40 tan 15° (F-map-build).
const DEFAULT_HEIGHT_DIF: f32 = 10.7;
/// The engine's default `minWaterDepth`: no floor to the water a kind stands in (`UnitDef.cpp`).
pub const NO_DEPTH_FLOOR: f32 = -10e6;
/// A metal-map square is 16 elmos; an extractor's offset from its spot is tried in steps of `MEX_STEP`.
const MEX_SQUARE: f32 = 16.0;
const MEX_STEP: f32 = 8.0;
/// How far from the start the stand is looked for: the people stand still or take one step (the study §5).
const STAND_RADIUS: f32 = 128.0;
const STAND_PITCH: f32 = 32.0;
/// The ground the commander itself needs at the stand: nothing is placed over it.
const COMMANDER_HALF: f32 = 24.0;
/// The margin round a metal spot's centre no building covers, so the extractor's own site stays free: the extractor's
/// footprint (48) and a build square.
const SPOT_BOX_HALF: f32 = 32.0;
/// How many touching neighbours of its kind a slot is credited: a bank grows along a row or round a corner.
const TOUCHING_CAP: usize = 2;
/// The cells the escape check walks, round the stand: a building fenced the commander in when no cell this far out is
/// reached from where it stands.
const ESCAPE_MARGIN: f32 = 96.0;
/// No two of the stand's rects leave a channel narrower than the commander between them (touching is no channel):
/// the engine's pathing sent it through one and it jammed there. The east seat of Comet Catcher, 2026-11-03 night
/// (dist102, dist103): the plant's south face and the first extractor's north face 32 apart, the commander wedged
/// between them at (6843, 705) on its way west in every east game, holding an extractor order five minutes at a time.
const CHANNEL: f32 = 2.0 * COMMANDER_HALF;

/// A building's footprint in build squares (x across its front, z front to back), as the definitions give it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    pub x: i32,
    pub z: i32,
}

impl Footprint {
    /// The footprint as it lies on the ground with this facing: an odd facing (east, west) swaps the sides.
    pub fn faced(self, facing: i32) -> Footprint {
        if facing.rem_euclid(2) == 1 { Footprint { x: self.z, z: self.x } } else { self }
    }

    /// Half the sides, in elmos.
    pub fn half(self) -> (f32, f32) {
        (self.x as f32 * SQUARE / 2.0, self.z as f32 * SQUARE / 2.0)
    }

    /// Where the engine stands a building of this footprint ordered at `pos` (`Pos2BuildPos`): the 16-elmo grid,
    /// offset by 8 on an axis whose size in squares is 2 mod 4.
    pub fn snap(self, pos: Vec3) -> Vec3 {
        Vec3 { x: snap_axis(pos.x, self.x), y: pos.y, z: snap_axis(pos.z, self.z) }
    }

    /// The ground it covers standing at `centre`.
    pub fn rect(self, centre: Vec3) -> Rect {
        let (hx, hz) = self.half();
        Rect { x0: centre.x - hx, z0: centre.z - hz, x1: centre.x + hx, z1: centre.z + hz }
    }
}

fn snap_axis(v: f32, size: i32) -> f32 {
    if size & 2 != 0 { (v / GRID).floor() * GRID + GRID / 2.0 } else { ((v + GRID / 2.0) / GRID).floor() * GRID }
}

/// An axis-aligned piece of ground, in elmos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x0: f32,
    pub z0: f32,
    pub x1: f32,
    pub z1: f32,
}

impl Rect {
    pub fn centre(&self) -> Vec3 {
        Vec3 { x: (self.x0 + self.x1) / 2.0, y: 0.0, z: (self.z0 + self.z1) / 2.0 }
    }

    /// Whether the two share ground; two that only touch along an edge do not.
    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.z0 < o.z1 && o.z0 < self.z1
    }

    /// Whether the two stand side by side: an edge in common, over a length, with no ground shared.
    pub fn touches(&self, o: &Rect) -> bool {
        let along_x = (self.x1 == o.x0 || o.x1 == self.x0) && self.z0 < o.z1 && o.z0 < self.z1;
        let along_z = (self.z1 == o.z0 || o.z1 == self.z0) && self.x0 < o.x1 && o.x0 < self.x1;
        along_x || along_z
    }

    pub fn contains(&self, p: Vec3) -> bool {
        p.x >= self.x0 && p.x < self.x1 && p.z >= self.z0 && p.z < self.z1
    }

    pub fn grown(&self, by: f32) -> Rect {
        Rect { x0: self.x0 - by, z0: self.z0 - by, x1: self.x1 + by, z1: self.z1 + by }
    }
}

/// Where a kind of unit stands, one flag per terrain cell in the terrain's row-major order (`terrain::passable` of its
/// movement class): a plant's products' ground, the strictest among them, which its apron and street are judged on.
/// Shared, since a world is built every pass and the set is the map's size.
pub type Passable = std::sync::Arc<[bool]>;

/// What the layer needs to know of a definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Facts {
    pub def: UnitDefId,
    pub footprint: Footprint,
    /// How far from itself a builder of this kind builds, measured to the target's edge.
    pub build_distance: f32,
    /// Its build power, for a builder; 0 otherwise.
    pub build_speed: f32,
    /// Work one unit of build power does on it for one second adds `1 / build_time` of it.
    pub build_time: f32,
    pub metal: f32,
    pub health: f32,
    /// What its death does to what stands around it: (radius, damage at the centre); None for a quiet death.
    pub blast: Option<(f32, f32)>,
    /// The height difference across its footprint the engine allows (F-map-build).
    pub height_dif: f32,
    /// The water it stands in: every square's ground at a height in `[-max_depth, -min_depth]` (F-map-build). A land
    /// building has no floor and a ceiling of 0; a sea kind a floor (a shipyard's 30) and no ceiling; a tier-1
    /// extractor neither.
    pub min_depth: f32,
    pub max_depth: f32,
    /// Whether it floats: in water its squares skip the height-difference check.
    pub floater: bool,
    pub extractor: bool,
    pub factory: bool,
    /// A structure: it does not move.
    pub building: bool,
    /// For a factory, half the pathing width of the widest unit it makes, in elmos: its street's half-width. 0
    /// otherwise; the caller sets it (`with_products`), since it needs the products' definitions.
    pub product_half: f32,
    /// For a factory, the ground its products stand on, the strictest of their movement classes (a vehicle plant's
    /// Stumpy on slopes to 28, a shipyard's Corsair in water 30 deep): its apron and street are judged on it, and on
    /// the world's `open` when the caller has none (apex-v0409-comet-sym0-opus-2: the three vehicle plants' streets
    /// were judged on the Pawn's ground, which climbs a crater's rim a Stumpy cannot, and all three were penned by
    /// 12:00 with the street line saying open). None for every other kind.
    #[serde(with = "dump::walk_bits")]
    pub product_passable: Option<Passable>,
}

impl Facts {
    pub fn of(d: &UnitDefInfo) -> Facts {
        Facts {
            def: d.id,
            footprint: Footprint { x: d.footprint.0, z: d.footprint.1 },
            build_distance: d.build_distance,
            build_speed: d.build_speed,
            build_time: d.build_time,
            metal: d.metal_cost,
            health: d.health,
            blast: d.death_blast.map(|b| (b.radius, b.damage)),
            height_dif: if d.max_height_dif > 0.0 { d.max_height_dif } else { DEFAULT_HEIGHT_DIF },
            // A record or shim from before the depths were sent says 0 and 0: land alone.
            min_depth: if d.min_water_depth == 0.0 && d.max_water_depth == 0.0 { NO_DEPTH_FLOOR } else { d.min_water_depth },
            max_depth: d.max_water_depth,
            floater: d.floater,
            extractor: d.extracts_metal > 0.0,
            factory: d.speed == 0.0 && !d.build_options.is_empty(),
            building: d.speed == 0.0,
            product_half: 0.0,
            product_passable: None,
        }
    }

    /// A factory's street: its widest product's half-width, and the ground its products stand on.
    pub fn with_products(self, product_half: f32, product_passable: Option<Passable>) -> Facts {
        Facts { product_half, product_passable, ..self }
    }

    /// The width of a factory's street, as the design has it: as wide as its widest product, half of it. `widths`
    /// are the products' pathing widths in elmos (`UnitDefInfo::path_width`: the move class's footprint, which the
    /// engine's block test spans, not the unit's; 1794060468-human: the layer took a Fatboy as 48 and left it two
    /// 48-elmo corridors, and the engine, which paths a Fatboy at 56, kept seventeen of them in the pocket).
    pub fn product_half_of(widths: impl Iterator<Item = f32>) -> f32 {
        widths.fold(0.0, f32::max) / 2.0
    }

    /// How far from `builder`'s position this building's centre may be for the builder to raise it without a step:
    /// the build distance (to the target's edge) plus the building's radius, taken as half its narrower side. The
    /// engine takes the model's radius, which is larger; so a slot within this is within reach for certain.
    pub fn reach_from(&self, builder: &Facts) -> f32 {
        let (hx, hz) = self.footprint.half();
        builder.build_distance + hx.min(hz)
    }
}

/// The ground, for whether a footprint stands.
pub struct Ground<'a> {
    terrain: &'a Terrain,
    map_width: f32,
    map_height: f32,
    /// The least and greatest height on the map, found once when first asked.
    bounds: std::cell::OnceCell<(f32, f32)>,
    /// Per cell, the least and greatest height of it and its four neighbours, found once when first asked: the
    /// range an engine square's height between our cells lies in (`stands` read five cells a square before).
    ranges: std::cell::OnceCell<Vec<(i16, i16)>>,
}

impl<'a> Ground<'a> {
    pub fn new(terrain: &'a Terrain, map_width: f32, map_height: f32) -> Ground<'a> {
        Ground { terrain, map_width, map_height, bounds: std::cell::OnceCell::new(), ranges: std::cell::OnceCell::new() }
    }

    /// The least and greatest height anywhere on the map.
    pub fn height_bounds(&self) -> (f32, f32) {
        *self.bounds.get_or_init(|| {
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for h in &self.terrain.heights {
                lo = lo.min(f32::from(*h));
                hi = hi.max(f32::from(*h));
            }
            (lo, hi)
        })
    }

    /// Whether any ground on the map is at a height in the kind's depths, `[-f.max_depth, -f.min_depth]`: a sea
    /// kind on a map without water stands nowhere, and the search for its site is not made (econ-flagship-1: a
    /// shipyard's search flooded its whole band three times for every constructor every second, about a second each,
    /// 9,704 times, and the game took 2.5 hours of wall under the lockstep).
    pub fn depths_on_map(&self, f: &Facts) -> bool {
        let (lo, hi) = self.height_bounds();
        lo <= -f.min_depth && hi >= -f.max_depth
    }

    pub fn height_at(&self, x: f32, z: f32) -> Option<f32> {
        let t = self.terrain;
        if t.cell <= 0.0 || x < 0.0 || z < 0.0 {
            return None;
        }
        let (i, j) = ((x / t.cell) as usize, (z / t.cell) as usize);
        (i < t.width as usize && j < t.height as usize).then(|| f32::from(t.heights[j * t.width as usize + i]))
    }

    /// Whether `walk` (one flag a terrain cell) says the ground at (x, z) is stood on; off the map it is not.
    pub fn walks(&self, walk: &[bool], x: f32, z: f32) -> bool {
        let t = self.terrain;
        if t.cell <= 0.0 || x < 0.0 || z < 0.0 {
            return false;
        }
        let (i, j) = ((x / t.cell) as usize, (z / t.cell) as usize);
        i < t.width as usize && j < t.height as usize && walk.get(j * t.width as usize + i).copied().unwrap_or(false)
    }

    /// The terrain's cell and its count of cells each way: what a walk set is indexed by (`walks`).
    pub fn cells(&self) -> (f32, usize, usize) {
        (self.terrain.cell, self.terrain.width as usize, self.terrain.height as usize)
    }

    /// The map's extent in elmos.
    pub fn extent(&self) -> (f32, f32) {
        (self.map_width, self.map_height)
    }

    pub fn in_map(&self, r: &Rect) -> bool {
        r.x0 >= 0.0 && r.z0 >= 0.0 && r.x1 <= self.map_width && r.z1 <= self.map_height
    }

    /// Whether a building of `f` with this footprint (as faced) stands at `centre`, as the engine tests a build square
    /// (`GameHelper.cpp` `CheckTerrainConstraints`, F-map-build): inside the map, every square's ground at a height
    /// in `[-f.max_depth, -f.min_depth]` (land alone for a land building, water deep enough for a sea kind, anywhere
    /// for a tier-1 extractor), and within `f.height_dif` of the centre square's height unless the kind floats and
    /// the square is under water.
    pub fn stands(&self, footprint: Footprint, centre: Vec3, f: &Facts) -> bool {
        let rect = footprint.rect(centre);
        if !self.in_map(&rect) {
            return false;
        }
        let Some(floor) = self.height_at(centre.x, centre.z) else { return false };
        let t = self.terrain;
        if t.cell <= 0.0 {
            return false;
        }
        // Every engine square of the footprint lies in one of our cells; the test is per cell (a 12-square side
        // read 144 squares, 720 cells, where its 36 cells say the same).
        let (w, h) = (t.width as usize, t.height as usize);
        let (i0, i1) = ((rect.x0 / t.cell).floor().max(0.0) as usize, ((rect.x1 / t.cell).ceil() as usize).min(w));
        let (j0, j1) = ((rect.z0 / t.cell).floor().max(0.0) as usize, ((rect.z1 / t.cell).ceil() as usize).min(h));
        let ranges = self.ranges();
        for j in j0..j1 {
            for i in i0..i1 {
                let k = j * w + i;
                let cell = f32::from(t.heights[k]);
                // The engine reads the height at the square itself, between our cells, so it lies between the cell's
                // and its neighbours': the whole range must be in the kind's depths (water-2: a shipyard's site with
                // every cell at -30 to -119 was refused by the engine for a floor of 30).
                let (low, high) = (f32::from(ranges[k].0), f32::from(ranges[k].1));
                if low < -f.max_depth || high > -f.min_depth {
                    return false;
                }
                if (low - floor).abs().max((high - floor).abs()) > f.height_dif && !(f.floater && cell <= 0.0) {
                    return false;
                }
            }
        }
        true
    }

    /// Per cell, the least and greatest height of it and its four neighbours.
    fn ranges(&self) -> &[(i16, i16)] {
        self.ranges.get_or_init(|| {
            let t = self.terrain;
            let (w, h) = (t.width as usize, t.height as usize);
            let mut out = Vec::with_capacity(w * h);
            for j in 0..h {
                for i in 0..w {
                    let mut low = t.heights[j * w + i];
                    let mut high = low;
                    for (di, dj) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                        let (ni, nj) = (i as i64 + di, j as i64 + dj);
                        if ni >= 0 && nj >= 0 && (ni as usize) < w && (nj as usize) < h {
                            let n = t.heights[nj as usize * w + ni as usize];
                            low = low.min(n);
                            high = high.max(n);
                        }
                    }
                    out.push((low, high));
                }
            }
            out
        })
    }
}

/// Where an extractor on `spot` stands for a builder at `from` with `reach` to the extractor's centre: inside the
/// patch, as near the builder as the patch allows, so the builder takes no step for it when it can be helped. The
/// engine draws only the squares whose centres lie within the extractor radius of the extractor
/// (`ExtractorBuilding.cpp`), stricter than what the game allows to be built (`cmd_mex_denier.lua`: the radius plus
/// one metal square of every square of the patch), so every square of the patch stays within the radius less a
/// square for the grid's snap: player-52's extractors stood 101 off centre at the median under the game's rule and
/// drew 1.9 a second from patches worth 2.3. Tried from the offset wanted down to nothing; the spot's centre when the
/// patch is unknown.
pub fn extractor_site(spot: Vec3, squares: &[(f32, f32)], extractor_radius: f32, from: Vec3, reach: f32) -> Vec3 {
    if squares.is_empty() {
        return spot;
    }
    let d = spot.dist2d(from);
    let wanted = (d - reach + SQUARE).max(0.0);
    if wanted <= 0.0 || d <= 0.0 {
        return spot;
    }
    let allowed = extractor_radius - MEX_SQUARE;
    let valid = |p: Vec3| squares.iter().all(|(sx, sz)| (p.x - sx).hypot(p.z - sz) < allowed);
    let mut offset = wanted.min(extractor_radius);
    while offset > 0.0 {
        let p = Vec3 { x: spot.x + (from.x - spot.x) / d * offset, y: spot.y, z: spot.z + (from.z - spot.z) / d * offset };
        if valid(p) {
            return p;
        }
        offset -= MEX_STEP;
    }
    spot
}

/// The engine's facing as a direction: 0 south (+z), 1 east (+x), 2 north (-z), 3 west (-x).
pub fn facing_dir(facing: i32) -> (f32, f32) {
    match facing.rem_euclid(4) {
        0 => (0.0, 1.0),
        1 => (1.0, 0.0),
        2 => (0.0, -1.0),
        _ => (-1.0, 0.0),
    }
}

/// The facings in order of how much each points toward `toward` from `at`: a factory's units leave through its front.
pub fn facings_toward(at: Vec3, toward: Vec3) -> [i32; 4] {
    let (dx, dz) = (toward.x - at.x, toward.z - at.z);
    let mut facings = [0, 1, 2, 3];
    facings.sort_by(|a, b| {
        let along = |f: &i32| {
            let (fx, fz) = facing_dir(*f);
            fx * dx + fz * dz
        };
        along(b).total_cmp(&along(a))
    });
    facings
}

/// A factory's exit lane standing at `centre` with this facing: from its front face out by `depth`, as wide as the
/// footprint plus `margin` a side (the engine's empty-spot search walks the front; `Factory.cpp` `SendToEmptySpot`).
pub fn lane(footprint: Footprint, centre: Vec3, facing: i32, depth: f32, margin: f32) -> Lane {
    let (half_width, half_depth) = footprint.half();
    let (fx, fz) = facing_dir(facing);
    let front = Vec3 { x: centre.x + fx * half_depth, y: centre.y, z: centre.z + fz * half_depth };
    let far = half_depth + depth;
    Lane { from: front, to: Vec3 { x: centre.x + fx * far, y: centre.y, z: centre.z + fz * far }, half_width: half_width + margin }
}

/// The same lane as the ground it covers.
pub fn lane_rect(footprint: Footprint, centre: Vec3, facing: i32, depth: f32, margin: f32) -> Rect {
    let l = lane(footprint, centre, facing, depth, margin);
    let (x0, x1) = (l.from.x.min(l.to.x), l.from.x.max(l.to.x));
    let (z0, z1) = (l.from.z.min(l.to.z), l.from.z.max(l.to.z));
    match facing.rem_euclid(2) {
        0 => Rect { x0: x0 - l.half_width, z0, x1: x1 + l.half_width, z1 },
        _ => Rect { x0, z0: z0 - l.half_width, x1, z1: z1 + l.half_width },
    }
}

/// One thing the opening wants built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Want {
    /// The nearest free spot.
    Extractor,
    Building(UnitDefId),
}

/// One solved slot of the stand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    /// An extractor: on the spot with this index, or (`None`) on a tile of a metal map's coverage.
    Extractor { spot: Option<usize>, at: Vec3 },
    Building { def: UnitDefId, at: Vec3, facing: i32 },
}

impl Slot {
    pub fn at(&self) -> Vec3 {
        match *self {
            Slot::Extractor { at, .. } | Slot::Building { at, .. } => at,
        }
    }
}

/// The commander's stand: where it stands, and its opening slots in the wants' order.
#[derive(Clone, Debug)]
pub struct Stand {
    pub at: Vec3,
    pub slots: Vec<Slot>,
    /// The walking the stand costs the builder: from the start to the stand, and out and back to every extractor
    /// beyond its reach.
    pub walk: f32,
    /// The wants that found no slot within reach: placed by other means.
    pub unplaced: usize,
}

/// What the stand is solved over.
pub struct Scene<'a> {
    pub start: Vec3,
    /// The builder that stands: its reach.
    pub builder: Facts,
    /// The extractor it builds.
    pub extractor: Facts,
    pub facts: &'a dyn Fn(UnitDefId) -> Option<Facts>,
    pub spots: &'a [Vec3],
    /// Per spot, the centres of its metal squares (`Hello::metal_spot_squares`); empty when unknown.
    pub squares: &'a [Vec<(f32, f32)>],
    pub extractor_radius: f32,
    /// Spots no extractor of ours goes on: held, claimed by another builder, his.
    pub spots_taken: &'a [usize],
    /// No building but an extractor has its centre this close to a spot's centre (the shim refuses such a site, so
    /// the base's own generators never bury an extractor site).
    pub spot_keep_out: f32,
    pub ground: Ground<'a>,
    /// Where the enemy is: a factory's front points there when it can.
    pub toward: Vec3,
    /// Whether a factory's units can stand on the ground here (its strictest movement class).
    pub open: &'a dyn Fn(Vec3) -> bool,
    /// The ground already taken: what stands or is ordered.
    pub occupied: Vec<Rect>,
    /// On a metal map (no spots published): the metal's coverage and the extractor's depth (`extractsMetal`), so
    /// the stand's extractor slots are the tiles of full worth within the builder's reach of the stand.
    pub coverage: Option<(&'a Coverage, f32)>,
}

/// How many of a factory's sites the stand tests for an open street, the best first.
const STREET_TRIES: usize = 32;

/// The stand for these wants: the point near the start from which the most of them are within the builder's reach,
/// with the least walking, and the slots there. Solved in the wants' kinds' order of need (extractors on their
/// spots, then factories with their lanes, then the rest) and returned in the wants' order. `None` when there is no
/// ground at all (no terrain).
pub fn stand(scene: &Scene, wants: &[Want]) -> Option<Stand> {
    if scene.ground.terrain.cell <= 0.0 || scene.ground.terrain.heights.is_empty() {
        return None;
    }
    let mut memo: HashMap<(i32, i32, i32, i32, i32), bool> = HashMap::new();
    let mut candidates: Vec<Vec3> = vec![scene.start];
    let n = (STAND_RADIUS / STAND_PITCH) as i32;
    for i in -n..=n {
        for j in -n..=n {
            let p = Vec3 { x: scene.start.x + i as f32 * STAND_PITCH, y: scene.start.y, z: scene.start.z + j as f32 * STAND_PITCH };
            if p.dist2d(scene.start) <= STAND_RADIUS && (i, j) != (0, 0) {
                candidates.push(p);
            }
        }
    }
    candidates.sort_by(|a, b| a.dist2d(scene.start).total_cmp(&b.dist2d(scene.start)));
    let mut best: Option<Stand> = None;
    for s in candidates {
        // Nowhere to stand: in the water, off the map, under something.
        if !scene.ground.height_at(s.x, s.z).is_some_and(|h| h >= 0.0) || scene.occupied.iter().any(|r| r.contains(s)) {
            continue;
        }
        let mut found = solve_at(scene, s, wants, &mut memo);
        found.walk += s.dist2d(scene.start);
        let better = match &best {
            None => true,
            Some(b) => (found.unplaced, found.walk) < (b.unplaced, b.walk),
        };
        if better {
            let done = found.unplaced == 0 && found.walk == 0.0;
            best = Some(found);
            if done {
                break;
            }
        }
    }
    best
}

fn solve_at(scene: &Scene, s: Vec3, wants: &[Want], memo: &mut HashMap<(i32, i32, i32, i32, i32), bool>) -> Stand {
    let mut occupied: Vec<Rect> = scene.occupied.clone();
    let own_ground = Rect { x0: s.x - COMMANDER_HALF, z0: s.z - COMMANDER_HALF, x1: s.x + COMMANDER_HALF, z1: s.z + COMMANDER_HALF };
    occupied.push(own_ground);
    let mut spot_boxes: Vec<(usize, Rect)> = scene
        .spots
        .iter()
        .enumerate()
        .filter(|(i, _)| !scene.spots_taken.contains(i))
        .map(|(i, p)| (i, Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }))
        .collect();
    let mut aprons: Vec<Rect> = Vec::new();
    let mut plants: Vec<Plant> = Vec::new();
    let mut slots: Vec<Option<Slot>> = vec![None; wants.len()];
    let mut walk = 0.0;
    // A rect that would leave a channel narrower than the commander to anything standing, ordered or kept for an
    // extractor (the commander's own ground excepted: it walks away from it).
    let channels = |rect: &Rect, occupied: &[Rect], spot_boxes: &[(usize, Rect)]| {
        let narrow = |o: &Rect| !o.overlaps(rect) && !o.touches(rect) && o.grown(CHANNEL).overlaps(rect);
        occupied.iter().filter(|o| !o.contains(s)).any(narrow) || spot_boxes.iter().any(|(_, o)| narrow(o))
    };
    let mut stands = |footprint: Footprint, centre: Vec3, f: &Facts| -> bool {
        let key = (f.def.0, footprint.x, footprint.z, centre.x.round() as i32, centre.z.round() as i32);
        *memo.entry(key).or_insert_with(|| scene.ground.stands(footprint, centre, f))
    };
    // Extractors first: their spots are given, and everything else keeps off them. On a metal map (no spots) each
    // takes the tile of full worth nearest the stand within the builder's reach, the ground standing and free.
    let mex_reach = scene.extractor.reach_from(&scene.builder);
    for (k, want) in wants.iter().enumerate() {
        if *want != Want::Extractor {
            continue;
        }
        if scene.spots.is_empty() {
            let Some((coverage, depth)) = scene.coverage else { continue };
            let footprint = scene.extractor.footprint;
            // A commander's width off the commander, as between any two of the stand's rects: the tile nearest the
            // stand is otherwise 7 elmos off its side, and two of them with the plant and a turbine are a pen
            // (1794106612-human on Full Metal Plate, all three commanders).
            let ok = |p: Vec3, rect: &Rect| scene.ground.stands(footprint, p, &scene.extractor) && !occupied.iter().any(|o| o.overlaps(rect)) && !own_ground.grown(CHANNEL).overlaps(rect);
            let Some((at, _)) = coverage.best_site(footprint, s, s, mex_reach, depth, &ok) else { continue };
            occupied.push(footprint.rect(at));
            walk += 2.0 * (at.dist2d(s) - mex_reach).max(0.0);
            slots[k] = Some(Slot::Extractor { spot: None, at });
            continue;
        }
        let nearest = spot_boxes.iter().enumerate().min_by(|(_, (i, _)), (_, (j, _))| scene.spots[*i].dist2d(s).total_cmp(&scene.spots[*j].dist2d(s)));
        let Some((n, (i, _))) = nearest.map(|(n, (i, r))| (n, (*i, *r))) else { continue };
        let squares = scene.squares.get(i).map_or(&[][..], Vec::as_slice);
        // Sited as the builder will site it, from the stand with its build distance.
        let at = extractor_site(scene.spots[i], squares, scene.extractor_radius, s, scene.builder.build_distance);
        spot_boxes.remove(n);
        occupied.push(scene.extractor.footprint.rect(at));
        walk += 2.0 * (at.dist2d(s) - mex_reach).max(0.0);
        slots[k] = Some(Slot::Extractor { spot: Some(i), at });
    }
    // The factories: the most ground, an apron the rest keeps out of, and a street to the open.
    let facings = facings_toward(s, scene.toward);
    for (k, want) in wants.iter().enumerate() {
        let Want::Building(def) = *want else { continue };
        let Some(f) = (scene.facts)(def) else { continue };
        if !f.factory {
            continue;
        }
        let reach = f.reach_from(&scene.builder);
        let mut found: Vec<(usize, f32, Vec3, i32, Rect, Rect, Plant)> = Vec::new();
        for (rank, facing) in facings.iter().enumerate() {
            let fp = f.footprint.faced(*facing);
            for centre in lattice(fp, s, reach) {
                let rect = fp.rect(centre);
                if occupied.iter().any(|r| r.overlaps(&rect)) || spot_boxes.iter().any(|(_, r)| r.overlaps(&rect)) || aprons.iter().any(|r| r.overlaps(&rect)) || near_a_spot(scene, centre) || channels(&rect, &occupied, &spot_boxes) {
                    continue;
                }
                if !stands(fp, centre, &f) {
                    continue;
                }
                let plant = Plant { centre, footprint: f.footprint, facing: *facing, product_half: f.product_half, ours: true, name: String::new(), widest: String::new(), passable: f.product_passable.clone() };
                let apron = plant.apron();
                if !scene.ground.in_map(&apron) || occupied.iter().any(|r| r.overlaps(&apron)) || spot_boxes.iter().any(|(_, r)| r.overlaps(&apron)) || !lane_open(&apron, scene.open) {
                    continue;
                }
                found.push((rank, centre.dist2d(s), centre, *facing, rect, apron, plant));
            }
        }
        // The best by facing and nearness whose street reaches the open, over the ground its products stand on
        // (2026-11-06: the street was tested for the single best site alone, by the caller's `open` and never by the
        // products' ground, so on Hooked the distilled bot's plant stood with its door 16 elmos from a cliff and its
        // constructors jittered there from 1:20, the plant blocked from 2:52: dist155, dist167). No more than
        // `STREET_TRIES` floods a factory.
        found.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap_or(std::cmp::Ordering::Equal));
        for (_, _, centre, facing, rect, apron, plant) in found.into_iter().take(STREET_TRIES) {
            occupied.push(rect);
            if street_open(&scene.ground, &plant, &occupied, scene.open) {
                aprons.push(apron);
                plants.push(plant);
                slots[k] = Some(Slot::Building { def, at: centre, facing });
                break;
            }
            occupied.pop();
        }
    }
    // The rest: packed, touching their own kind, out of every lane, leaving the commander a way out.
    let mut placed_of: HashMap<UnitDefId, Vec<Rect>> = HashMap::new();
    for (k, want) in wants.iter().enumerate() {
        let Want::Building(def) = *want else { continue };
        let Some(f) = (scene.facts)(def) else { continue };
        if f.factory || !f.building {
            continue;
        }
        let reach = f.reach_from(&scene.builder);
        let fp = f.footprint;
        let mut ranked: Vec<((usize, f32), Vec3, Rect)> = Vec::new();
        for centre in lattice(fp, s, reach) {
            let rect = fp.rect(centre);
            if occupied.iter().any(|r| r.overlaps(&rect)) || spot_boxes.iter().any(|(_, r)| r.overlaps(&rect)) || aprons.iter().any(|r| r.overlaps(&rect)) || near_a_spot(scene, centre) || channels(&rect, &occupied, &spot_boxes) {
                continue;
            }
            if !stands(fp, centre, &f) {
                continue;
            }
            let touching = placed_of.get(&def).map_or(0, |rs| rs.iter().filter(|r| r.touches(&rect)).count()).min(TOUCHING_CAP);
            ranked.push(((TOUCHING_CAP - touching, centre.dist2d(s)), centre, rect));
        }
        ranked.sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.total_cmp(&b.0.1)));
        for (_, centre, rect) in ranked {
            let mut with = occupied.clone();
            with.push(rect);
            if !escapes(s, &with, reach + ESCAPE_MARGIN) || plants.iter().any(|p| !street_open(&scene.ground, p, &with, scene.open)) {
                continue;
            }
            occupied.push(rect);
            placed_of.entry(def).or_default().push(rect);
            slots[k] = Some(Slot::Building { def, at: centre, facing: 0 });
            break;
        }
    }
    let unplaced = slots.iter().filter(|s| s.is_none()).count();
    Stand { at: s, slots: slots.into_iter().flatten().collect(), walk, unplaced }
}

/// Whether a building's centre here would be refused for a metal spot's sake.
fn near_a_spot(scene: &Scene, centre: Vec3) -> bool {
    scene.spots.iter().any(|p| p.dist2d(centre) < scene.spot_keep_out)
}

/// The centres on the build grid for this footprint within `reach` of `s`.
fn lattice(fp: Footprint, s: Vec3, reach: f32) -> Vec<Vec3> {
    let mut out = Vec::new();
    let first = fp.snap(Vec3 { x: s.x - reach, y: s.y, z: s.z - reach });
    let steps = (2.0 * reach / GRID) as i32 + 1;
    for i in 0..=steps {
        for j in 0..=steps {
            let p = Vec3 { x: first.x + i as f32 * GRID, y: s.y, z: first.z + j as f32 * GRID };
            if p.dist2d(s) <= reach {
                out.push(p);
            }
        }
    }
    out
}

/// Whether every cell of the strip is ground the factory's units can stand on.
fn lane_open(lane: &Rect, open: &dyn Fn(Vec3) -> bool) -> bool {
    let mut z = lane.z0 + GRID / 2.0;
    while z < lane.z1 {
        let mut x = lane.x0 + GRID / 2.0;
        while x < lane.x1 {
            if !open(Vec3 { x, y: 0.0, z }) {
                return false;
            }
            x += GRID;
        }
        z += GRID;
    }
    true
}

/// Whether a unit the commander's size at `from` reaches ground `radius` away through the cells no `occupied` rect
/// covers (a flood over the 16-elmo grid; a cell is blocked when its centre lies within a rect grown by the
/// commander's half-width).
fn escapes(from: Vec3, occupied: &[Rect], radius: f32) -> bool {
    let n = (radius / GRID).ceil() as i32;
    // The ground kept for the commander itself is no wall to it.
    let walls: Vec<Rect> = occupied.iter().filter(|r| !r.contains(from)).map(|r| r.grown(COMMANDER_HALF)).collect();
    let blocked = |i: i32, j: i32| {
        let p = Vec3 { x: from.x + i as f32 * GRID, y: 0.0, z: from.z + j as f32 * GRID };
        walls.iter().any(|r| r.contains(p))
    };
    let mut seen = vec![false; ((2 * n + 1) * (2 * n + 1)) as usize];
    let index = |i: i32, j: i32| ((j + n) * (2 * n + 1) + (i + n)) as usize;
    let mut stack: Vec<(i32, i32)> = vec![(0, 0)];
    seen[index(0, 0)] = true;
    while let Some((i, j)) = stack.pop() {
        if i.abs() == n || j.abs() == n {
            return true;
        }
        for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (ni, nj) = (i + di, j + dj);
            if ni.abs() > n || nj.abs() > n || seen[index(ni, nj)] {
                continue;
            }
            seen[index(ni, nj)] = true;
            // The commander's own square is never blocked, whatever stands over it.
            if (ni, nj) != (0, 0) && blocked(ni, nj) {
                continue;
            }
            stack.push((ni, nj));
        }
    }
    false
}

/// For the tests: where a ship of `depth` floats, from the terrain's heights (the bot's `terrain::passable` of a Ship
/// class says the same).
#[cfg(test)]
pub(crate) fn sea_walk(t: &Terrain, depth: f32) -> Passable {
    t.heights.iter().map(|h| f32::from(*h) <= -depth).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A flat map of 64 by 64 cells of 16 elmos, with a cliff where `cliff` says.
    fn terrain(cliff: impl Fn(f32, f32) -> bool) -> Terrain {
        let (w, h) = (64u32, 64u32);
        let mut heights = Vec::new();
        for j in 0..h {
            for i in 0..w {
                let (x, z) = (i as f32 * 16.0 + 8.0, j as f32 * 16.0 + 8.0);
                heights.push(if cliff(x, z) { 200 } else { 50 });
            }
        }
        Terrain { cell: 16.0, width: w, height: h, heights, slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    const COM: UnitDefId = UnitDefId(1);
    const MEX: UnitDefId = UnitDefId(2);
    const SOLAR: UnitDefId = UnitDefId(3);
    const LAB: UnitDefId = UnitDefId(4);

    fn facts(def: UnitDefId) -> Option<Facts> {
        let f = |footprint: (i32, i32), extractor: bool, factory: bool, build_distance: f32| Facts {
            def,
            footprint: Footprint { x: footprint.0, z: footprint.1 },
            build_distance,
            height_dif: 10.7,
            min_depth: NO_DEPTH_FLOOR,
            max_depth: 0.0,
            floater: false,
            extractor,
            factory,
            building: true,
            build_speed: 0.0,
            build_time: 1.0,
            metal: 0.0,
            health: 0.0,
            blast: None,
            product_half: 0.0,
            product_passable: None,
        };
        match def {
            COM => Some(f((4, 4), false, false, 145.0)),
            MEX => Some(f((6, 6), true, false, 0.0)),
            SOLAR => Some(f((10, 10), false, false, 0.0)),
            LAB => Some(Facts { product_half: 32.0, ..f((12, 12), false, true, 0.0) }),
            _ => None,
        }
    }

    fn scene<'a>(t: &'a Terrain, spots: &'a [Vec3], squares: &'a [Vec<(f32, f32)>], open: &'a dyn Fn(Vec3) -> bool) -> Scene<'a> {
        Scene {
            start: at(512.0, 512.0),
            builder: facts(COM).unwrap(),
            extractor: facts(MEX).unwrap(),
            facts: &facts,
            spots,
            squares,
            extractor_radius: 120.0,
            spots_taken: &[],
            spot_keep_out: 100.0,
            ground: Ground::new(t, 1024.0, 1024.0),
            toward: at(1000.0, 512.0),
            open,
            occupied: Vec::new(),
            coverage: None,
        }
    }

    /// On a metal map the stand's extractor tiles keep a commander's width off the commander, as any two of the
    /// stand's rects keep between them: the nearest full tile is otherwise 7 elmos off its side, and two of them
    /// with the plant and a turbine were a pen (1794106612-human on Full Metal Plate, all three commanders).
    #[test]
    fn on_a_metal_map_the_extractor_tiles_keep_off_the_commander() {
        let (w, h) = (64u32, 64u32);
        let t = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: vec![200; (w * h) as usize] };
        let coverage = crate::Coverage::new(&t, 1.0, 24.0).expect("a metal map");
        let open = |_: Vec3| true;
        let mut sc = scene(&t, &[], &[], &open);
        sc.coverage = Some((&coverage, 1.8));
        let wants = [Want::Extractor, Want::Extractor, Want::Building(SOLAR), Want::Building(SOLAR), Want::Building(LAB)];
        let stand = stand(&sc, &wants).expect("a stand");
        let own = Rect { x0: stand.at.x - COMMANDER_HALF, z0: stand.at.z - COMMANDER_HALF, x1: stand.at.x + COMMANDER_HALF, z1: stand.at.z + COMMANDER_HALF };
        let tiles: Vec<Vec3> = stand.slots.iter().filter_map(|s| match *s { Slot::Extractor { spot: None, at } => Some(at), _ => None }).collect();
        assert_eq!(tiles.len(), 2, "{stand:?}");
        for at in tiles {
            let rect = Footprint { x: 6, z: 6 }.rect(at);
            assert!(!own.grown(CHANNEL).overlaps(&rect), "tile at ({}, {}) within a commander's width of the stand at ({}, {})", at.x, at.z, stand.at.x, stand.at.z);
            assert!(at.dist2d(stand.at) <= 145.0, "{at:?} beyond reach of the stand");
        }
        assert_eq!(stand.unplaced, 0, "{stand:?}");
    }

    #[test]
    fn the_grid_snaps_as_the_engine_does() {
        // A solar (10 squares, 2 mod 4) sits at an offset of 8; a lab (12 squares) on the grid itself.
        assert_eq!(Footprint { x: 10, z: 10 }.snap(at(100.0, 100.0)), at(104.0, 104.0));
        assert_eq!(Footprint { x: 12, z: 12 }.snap(at(100.0, 100.0)), at(96.0, 96.0));
        assert_eq!(Footprint { x: 12, z: 12 }.snap(at(105.0, 105.0)), at(112.0, 112.0));
        assert_eq!(Footprint { x: 12, z: 6 }.faced(1), Footprint { x: 6, z: 12 });
    }

    #[test]
    fn an_extractor_is_offset_toward_the_builder_inside_its_patch() {
        let spot = at(500.0, 500.0);
        let squares: Vec<(f32, f32)> = vec![(488.0, 488.0), (504.0, 488.0), (488.0, 504.0), (504.0, 504.0)];
        // A builder 300 away with 169 of reach: the site moves toward it, every square still within 104 of it.
        let site = extractor_site(spot, &squares, 120.0, at(800.0, 500.0), 169.0);
        assert!(site.x > spot.x && site.z == spot.z, "{site:?}");
        assert!(squares.iter().all(|(sx, sz)| (site.x - sx).hypot(site.z - sz) < 104.0));
        // In reach already: the centre.
        assert_eq!(extractor_site(spot, &squares, 120.0, at(600.0, 500.0), 169.0), spot);
    }

    #[test]
    fn a_lane_lies_off_the_front_face() {
        let lab = Footprint { x: 12, z: 12 };
        // Facing east from (512, 512): the front face at x = 560, the lane to 880, 96 wide plus 48 a side.
        let r = lane_rect(lab, at(512.0, 512.0), 1, 320.0, 48.0);
        assert_eq!(r, Rect { x0: 560.0, z0: 416.0, x1: 880.0, z1: 608.0 });
        let r = lane_rect(lab, at(512.0, 512.0), 2, 320.0, 48.0);
        assert_eq!(r, Rect { x0: 416.0, z0: 144.0, x1: 608.0, z1: 464.0 });
    }

    /// On a metal map (no spots) the stand's extractors are tiles of the coverage: full worth, within the builder's
    /// reach of the stand, the ground standing, nothing overlapping; a third beyond the reach is unplaced.
    #[test]
    fn the_stand_lays_its_extractors_on_the_coverage_when_the_map_has_no_spots() {
        let mut t = terrain(|_, _| false);
        t.metal = vec![255; (t.width * t.height) as usize];
        let coverage = Coverage::new(&t, 10.0, 30.0).expect("a coverage");
        let open = |_: Vec3| true;
        let mut sc = scene(&t, &[], &[], &open);
        sc.coverage = Some((&coverage, 0.001));
        let wants = [Want::Extractor, Want::Extractor, Want::Building(SOLAR), Want::Building(LAB)];
        let stand = super::stand(&sc, &wants).expect("a stand");
        let mexes: Vec<Vec3> = stand.slots.iter().filter_map(|s| if let Slot::Extractor { spot: None, at } = s { Some(*at) } else { None }).collect();
        assert_eq!(mexes.len(), 2, "{:?}", stand.slots);
        let reach = facts(MEX).unwrap().reach_from(&sc.builder);
        for at in &mexes {
            assert!(at.dist2d(stand.at) <= reach + 1.0, "within reach of the stand: {at:?} from {:?}", stand.at);
            // Unclaimed metal under every square it reaches (the test's 6 by 6 footprint centres off a square corner,
            // so the count is not SpeedMetal's twelve; the plateau is even, so both tiles are worth the same).
            assert!(coverage.worth_at(*at, 0.001) > 15.0 && (coverage.worth_at(*at, 0.001) - coverage.worth_at(mexes[0], 0.001)).abs() < 0.01, "{}", coverage.worth_at(*at, 0.001));
        }
        let rects: Vec<Rect> = stand.slots.iter().map(|s| match *s { Slot::Extractor { at, .. } => facts(MEX).unwrap().footprint.rect(at), Slot::Building { def, at, facing } => facts(def).unwrap().footprint.faced(facing).rect(at) }).collect();
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(!a.overlaps(b), "{a:?} overlaps {b:?}");
            }
        }
        assert_eq!(stand.unplaced, 0);
        // Without a coverage and without spots, the extractor wants find no slot.
        sc.coverage = None;
        let stand = super::stand(&sc, &wants).expect("a stand");
        assert!(stand.slots.iter().all(|s| !matches!(s, Slot::Extractor { .. })));
    }

    /// The opening's stand on flat ground: two extractors on their spots, the lab facing the enemy with its lane
    /// free, three solars touching, every slot within reach of the stand, nothing overlapping, the commander free to
    /// leave.
    #[test]
    fn the_stand_puts_the_opening_within_reach_of_one_point() {
        let t = terrain(|_, _| false);
        let spots = [at(400.0, 512.0), at(620.0, 400.0), at(100.0, 100.0)];
        let squares: Vec<Vec<(f32, f32)>> = spots.iter().map(|s| vec![(s.x - 8.0, s.z - 8.0), (s.x + 8.0, s.z - 8.0), (s.x - 8.0, s.z + 8.0), (s.x + 8.0, s.z + 8.0)]).collect();
        let open = |_: Vec3| true;
        let sc = scene(&t, &spots, &squares, &open);
        let wants = [Want::Extractor, Want::Extractor, Want::Building(SOLAR), Want::Building(SOLAR), Want::Building(SOLAR), Want::Building(LAB)];
        let stand = stand(&sc, &wants).expect("a stand");
        assert_eq!(stand.unplaced, 0, "{stand:?}");
        assert_eq!(stand.slots.len(), 6);
        let spots_taken: Vec<usize> = stand.slots.iter().filter_map(|s| if let Slot::Extractor { spot, .. } = s { *spot } else { None }).collect();
        assert_eq!(spots_taken, vec![0, 1], "the two nearest spots, in the wants' order");
        let rects: Vec<Rect> = stand
            .slots
            .iter()
            .map(|s| match *s {
                Slot::Extractor { at, .. } => facts(MEX).unwrap().footprint.rect(at),
                Slot::Building { def, at, facing } => facts(def).unwrap().footprint.faced(facing).rect(at),
            })
            .collect();
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(!a.overlaps(b), "{a:?} overlaps {b:?}");
            }
        }
        let lab = stand.slots.iter().find_map(|s| if let Slot::Building { def: LAB, at, facing } = *s { Some((at, facing)) } else { None }).expect("the lab");
        assert_eq!(lab.1, 1, "the lab faces east, toward the enemy");
        let plant = Plant { centre: lab.0, footprint: facts(LAB).unwrap().footprint, facing: lab.1, product_half: 32.0, ours: true, name: String::new(), widest: String::new(), passable: None };
        assert!(rects.iter().all(|r| !r.overlaps(&plant.apron())), "nothing in the lab's apron");
        assert!(street_open(&sc.ground, &plant, &rects, &open), "the lab's street is open");
        for s in &stand.slots {
            let reach = match *s {
                Slot::Extractor { .. } => facts(MEX).unwrap().reach_from(&sc.builder),
                Slot::Building { def, .. } => facts(def).unwrap().reach_from(&sc.builder),
            };
            assert!(s.at().dist2d(stand.at) <= reach + 1.0, "{s:?} is out of reach of {:?}", stand.at);
        }
        let solars: Vec<Rect> = stand.slots.iter().filter_map(|s| if let Slot::Building { def: SOLAR, at, .. } = *s { Some(facts(SOLAR).unwrap().footprint.rect(at)) } else { None }).collect();
        assert_eq!(solars.len(), 3);
        assert!(solars.iter().any(|a| solars.iter().any(|b| a != b && a.touches(b))), "the solars make a bank: {solars:?}");
        assert!(escapes(stand.at, &rects, 300.0));
    }

    /// A cliff east of the start: nothing stands on it, the lab's apron lies on open ground, and its street reaches
    /// the open (the street rule lets it face the cliff when its units can leave round the sides).
    #[test]
    fn the_stand_keeps_off_a_cliff() {
        let t = terrain(|x, _| x > 600.0);
        let spots = [at(400.0, 512.0)];
        let squares: Vec<Vec<(f32, f32)>> = vec![vec![]];
        let open = |p: Vec3| p.x <= 600.0;
        let sc = scene(&t, &spots, &squares, &open);
        let wants = [Want::Extractor, Want::Building(LAB), Want::Building(SOLAR), Want::Building(SOLAR)];
        let stand = stand(&sc, &wants).expect("a stand");
        assert_eq!(stand.unplaced, 0, "{stand:?}");
        let lab = stand.slots.iter().find_map(|s| if let Slot::Building { def: LAB, at, facing } = *s { Some((at, facing)) } else { None }).expect("the lab");
        let plant = Plant { centre: lab.0, footprint: facts(LAB).unwrap().footprint, facing: lab.1, product_half: 32.0, ours: true, name: String::new(), widest: String::new(), passable: None };
        assert!(plant.apron().x1 <= 600.0 + 8.0, "the apron is off the cliff: {stand:?}");
        let rects: Vec<Rect> = stand.slots.iter().map(|s| match *s { Slot::Extractor { at, .. } => facts(MEX).unwrap().footprint.rect(at), Slot::Building { def, at, facing } => facts(def).unwrap().footprint.faced(facing).rect(at) }).collect();
        assert!(street_open(&sc.ground, &plant, &rects, &open));
        for s in &stand.slots {
            let fp = match *s {
                Slot::Extractor { .. } => facts(MEX).unwrap().footprint,
                Slot::Building { def, facing, .. } => facts(def).unwrap().footprint.faced(facing),
            };
            assert!(fp.rect(s.at()).x1 <= 600.0 + 8.0, "{s:?} stands on the cliff");
        }
    }
}

#[cfg(test)]
mod sea_tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A shore: a cliff top at 80 west of x 128, land at 50 to x 512, a shallow strip of -10 to x 576, water of -40
    /// to x 896 and -60 beyond.
    fn shore() -> Terrain {
        let (w, h) = (64u32, 64u32);
        let mut heights = Vec::new();
        for _ in 0..h {
            for i in 0..w {
                let x = i as f32 * 16.0 + 8.0;
                heights.push(if x < 128.0 { 80 } else if x < 512.0 { 50 } else if x < 576.0 { -10 } else if x < 896.0 { -40 } else { -60 });
            }
        }
        Terrain { cell: 16.0, width: w, height: h, heights, slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn kind(min_depth: f32, max_depth: f32, floater: bool) -> Facts {
        Facts { def: UnitDefId(1), footprint: Footprint { x: 4, z: 4 }, build_distance: 0.0, build_speed: 0.0, build_time: 1.0, metal: 0.0, health: 0.0, blast: None, height_dif: 10.7, min_depth, max_depth, floater, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None }
    }

    /// The engine's depth rule (F-map-build): a land building stands on land alone, a sea kind in water at least its
    /// floor deep, a tier-1 extractor anywhere; a floater does not mind the sea floor's slope, a submerged kind does.
    #[test]
    fn a_kind_stands_where_its_depths_say_and_a_floater_ignores_the_sea_floor() {
        let t = shore();
        let g = Ground::new(&t, 1024.0, 1024.0);
        let fp = Footprint { x: 4, z: 4 };
        let land = kind(NO_DEPTH_FLOOR, 0.0, false);
        let tidal = kind(20.0, 1e7, true);
        let shipyard = kind(30.0, 1e7, true);
        let extractor = kind(NO_DEPTH_FLOOR, 1e7, false);
        let sunk_fusion = kind(25.0, 1e7, false);
        let (dry, shallow, wet) = (at(256.0, 512.0), at(544.0, 512.0), at(768.0, 512.0));
        assert!(g.stands(fp, dry, &land) && !g.stands(fp, shallow, &land) && !g.stands(fp, wet, &land));
        assert!(!g.stands(fp, dry, &tidal) && !g.stands(fp, shallow, &tidal) && g.stands(fp, wet, &tidal));
        assert!(g.stands(fp, wet, &shipyard) && !g.stands(fp, at(544.0, 512.0), &shipyard));
        // At the water's edge the next cell is the shallow strip of -10: the engine's squares between the cells may be
        // shallower than the floor, so a shipyard keeps a cell off the edge (water-2).
        assert!(!g.stands(fp, at(592.0, 512.0), &shipyard), "a shipyard with the shallow strip in the next cell");
        assert!(g.stands(fp, at(624.0, 512.0), &shipyard), "a shipyard a cell further out");
        assert!(g.stands(fp, dry, &extractor) && g.stands(fp, shallow, &extractor) && g.stands(fp, wet, &extractor));
        // Over the step from -40 to -60 (a footprint of 32 across x 896): the floater stands, the submerged kind not.
        let step = at(896.0, 512.0);
        assert!(g.stands(fp, step, &tidal), "a floater over a 20-elmo step in the sea floor");
        assert!(!g.stands(fp, step, &sunk_fusion), "a submerged kind over the same step");
        assert!(g.stands(fp, at(960.0, 512.0), &sunk_fusion), "the submerged kind on the flat floor beyond");
        // The engine's squares lie between our cells: a land kind whose footprint's next cell is the cliff top (30
        // higher) is refused, one a cell further off the cliff stands (water-2: a land defence the engine refused had
        // every cell within its tolerance and a neighbour cell outside it).
        assert!(!g.stands(fp, at(144.0, 512.0), &land), "a land kind beside the cliff");
        assert!(g.stands(fp, at(176.0, 512.0), &land), "a land kind a cell off the cliff");
    }
}
