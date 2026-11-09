//! Where a building goes: constraints, then bands (the design §1.1). Nothing is drawn in advance but reservations;
//! a site is the free lattice point best in order, each level a band, the first level that separates the candidates
//! deciding. Level 0 the constraints (the footprint stands; no apron and no street closed, ours or an ally's; the
//! reservations kept; off the spots; the bank rule for the kinds that bank; the kind's band of distance). Level 1 the
//! purpose: a generator's neighbours of its kind, a turret's pads and frames in reach, a plant's turrets in reach and
//! its facing, a frame's stand-up time under the build power in reach. Level 2 the chain sum (§1.3): what its death
//! takes and what its neighbours' deaths would do to it. Level 3 the builder's walk, level 4 the distance to home.
//! The street check (`street.rs`) runs on the candidates in that order and the first that keeps every street open is
//! the site.

use std::collections::HashMap;

use bot_protocol::{UnitDefId, Vec3};
use serde::{Deserialize, Serialize};

use crate::block::Lattice;
use crate::farm::Farm;
use crate::street::{Builder, Flood, Plant};
use crate::{Facts, Footprint, GRID, Ground, Rect, facings_toward};

/// The margin round an ally's footprint: one build square, so nothing of ours touches theirs.
const ALLY_MARGIN: f32 = 8.0;
/// The margin round a metal spot's centre no building covers (the extractor's footprint and a square).
const SPOT_BOX_HALF: f32 = 32.0;
/// A refused site keeps candidates of the type this far off.
const REFUSED_RADIUS: f32 = 32.0;
/// Banks of one kind stand touching or a vehicle street apart: three cells, which the commander and the tier-1
/// vehicles pass (the user, 2026-10-30, after block-2's commander rounded a wall of solars 700 long).
const BANK_STREET: f32 = 48.0;
/// How many touching neighbours of its kind a generator is credited at most.
const TOUCHING_CAP: f32 = 2.0;
/// A site that founds a bank scores this at level 1: worse than touching one by more than the band, so a bank grows
/// before a new one is founded, while touching one near the builder and touching two far away both pass to the walk.
const FOUNDING: f32 = 1.0;
/// How many candidates, best first, the street check tries before the band is widened. A candidate refused once is
/// not tried again: until 2026-11-07 the widenings re-tried the same twelve (the ranking is the distance's, so the
/// twelve best of the wider band were the twelve nearest, already refused), and 31 of 34 dumped searches ran all
/// three widenings over one set of sites.
const STREET_TRIES: usize = 12;
/// A builder of ours this close to a candidate's footprint is asked whether the site walls it in.
const BUILDER_NEAR: f32 = 200.0;
/// The band of distance is widened this many times by doubling when nothing in it passes.
const WIDENINGS: u32 = 2;
/// The bands: a candidate within the band of the best at a level goes to the next level.
const BAND_GENERATOR: f32 = 1.0;
const BAND_TURRET: f32 = 0.5;
const BAND_FRAME_SECONDS: f32 = 3.0;
const BAND_CHAIN_METAL: f32 = 100.0;
const BAND_WALK: f32 = 48.0;

/// A footprint standing on the ground: ours or an ally's, finished or ordered.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Standing {
    pub rect: Rect,
    pub def: Option<UnitDefId>,
    /// Its kind's internal name, for the bill's words ("31 armnanotc").
    pub kind: String,
    pub ours: bool,
    /// A generator or converter: a bank the block may be laid over, listed for reclaim-for-ground.
    pub bank: bool,
    pub metal: f32,
    pub health: f32,
    /// Its death's blast: (radius, damage at the centre).
    pub blast: Option<(f32, f32)>,
    /// How the report names it.
    pub name: String,
}

/// A standing construction turret: build power that reaches a site.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Worker {
    pub at: Vec3,
    pub reach: f32,
    pub power: f32,
}

/// An unfinished building a turret could help.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub rect: Rect,
    pub metal: f32,
}

/// Ground kept for something not yet ordered, and for what.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reservation {
    pub rect: Rect,
    pub for_: Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Reserved {
    /// Nothing goes here (the stand's slots, which their kinds take directly; a gantry's apron).
    Nothing,
    /// One kind's slot (the gantry's).
    Kind(UnitDefId),
}

impl Reservation {
    /// Whether a building of `facts` placed for `purpose` may stand on this ground.
    fn admits(&self, facts: &Facts) -> bool {
        match self.for_ {
            Reserved::Nothing => false,
            Reserved::Kind(def) => facts.def == def,
        }
    }
}

/// What a site is chosen over.
pub struct World<'a> {
    pub ground: Ground<'a>,
    pub spots: &'a [Vec3],
    /// No building but an extractor has its centre this close to a spot's centre (the shim's keep-out).
    pub spot_keep_out: f32,
    pub standing: Vec<Standing>,
    pub plants: Vec<Plant>,
    /// Our mobile builders: a site that walls one in is passed over (`way_out`).
    pub builders: Vec<Builder>,
    pub workers: Vec<Worker>,
    pub frames: Vec<Frame>,
    /// Ground kept for something not yet ordered: the stand's free slots, the block's gantry slot and apron.
    pub reservations: Vec<Reservation>,
    /// The block's sub-lattice for construction turrets, once the block is laid; its centre is the farm's.
    pub lattice: Option<Lattice>,
    /// The frame the farm is for (the advanced fusion's footprint), once the block is laid: the turret score's
    /// words count free farm ground in it.
    pub farm_frame: Option<Footprint>,
    /// Sites the engine refused lately, by type.
    pub refused: Vec<(UnitDefId, Vec3)>,
    pub home: Vec3,
    /// Where the enemy is: a plant's front points there when it can.
    pub toward: Vec3,
    /// Whether a plant's units can stand on the ground here.
    pub open: Box<dyn Fn(Vec3) -> bool + 'a>,
    /// The walk floods made over this world, by start cell, centre cell and radius: a builder's menu asks for a site
    /// for a dozen kinds a second, most from the same place with the same band, and the flood is the same.
    pub walks: std::cell::RefCell<HashMap<(i32, i32, i32, i32, i32), std::rc::Rc<Walk>>>,
    /// What stands, and the spots, by the buckets of ground they touch, made once when first asked: a candidate is
    /// tested against what stands near it, not against every building of a base of hundreds.
    pub standing_near: std::cell::RefCell<Option<(usize, Buckets)>>,
    pub spots_near: std::cell::RefCell<Option<(usize, Buckets)>>,
    /// Each plant's street flood (true, i) and each builder's way-out flood (false, i) over what stands, made when
    /// first asked: a try adds its own rect on top (`Flood::reaches_the_open_round`).
    pub streets: std::cell::RefCell<HashMap<(bool, usize), std::rc::Rc<Flood>>>,
    /// What stands with a death blast, by the buckets its blast reaches (`blasts_near`).
    pub blasts_near: std::cell::RefCell<Option<(usize, Buckets)>>,
    /// The farm round the block, made once when first asked (`farm`).
    pub farm: std::cell::RefCell<Option<std::rc::Rc<Farm>>>,
    /// The candidates' fixed parts, made as the questions come to them (`fixed_at`).
    pub fixed: std::cell::RefCell<FixedMemo>,
    /// The sums of the plants' walk sets asked so far (`walk_sums`), each beside its set.
    pub walk_sums: std::cell::RefCell<Vec<(crate::Passable, std::rc::Rc<Vec<u32>>)>>,
}

/// The edge of a bucket of ground.
const BUCKET: f32 = 256.0;

/// A grid of buckets over the map, each holding the indices of the rects that touch it.
pub struct Buckets {
    nx: usize,
    nz: usize,
    cells: Vec<Vec<u32>>,
}

impl Buckets {
    fn new(extent: (f32, f32), rects: impl Iterator<Item = Rect>) -> Buckets {
        let nx = (extent.0 / BUCKET).ceil().max(1.0) as usize;
        let nz = (extent.1 / BUCKET).ceil().max(1.0) as usize;
        let mut cells = vec![Vec::new(); nx * nz];
        for (i, r) in rects.enumerate() {
            let (i0, i1, j0, j1) = Buckets::span(nx, nz, &r);
            for j in j0..j1 {
                for x in i0..i1 {
                    cells[j * nx + x].push(i as u32);
                }
            }
        }
        Buckets { nx, nz, cells }
    }

    fn span(nx: usize, nz: usize, r: &Rect) -> (usize, usize, usize, usize) {
        let i0 = ((r.x0 / BUCKET).floor().max(0.0) as usize).min(nx);
        let i1 = (((r.x1 / BUCKET).floor().max(0.0) as usize) + 1).min(nx);
        let j0 = ((r.z0 / BUCKET).floor().max(0.0) as usize).min(nz);
        let j1 = (((r.z1 / BUCKET).floor().max(0.0) as usize) + 1).min(nz);
        (i0, i1, j0, j1)
    }

    /// Whether `hit` holds for any rect in the buckets `r` touches: no list is made, and a rect in two of the
    /// buckets may be asked twice.
    fn any(&self, r: &Rect, mut hit: impl FnMut(usize) -> bool) -> bool {
        let (i0, i1, j0, j1) = Buckets::span(self.nx, self.nz, r);
        (j0..j1).any(|j| (i0..i1).any(|x| self.cells[j * self.nx + x].iter().any(|i| hit(*i as usize))))
    }

    /// The indices of the rects in the buckets `r` touches, each once.
    fn near(&self, r: &Rect) -> Vec<usize> {
        let (i0, i1, j0, j1) = Buckets::span(self.nx, self.nz, r);
        let mut out: Vec<usize> = Vec::new();
        for j in j0..j1 {
            for x in i0..i1 {
                out.extend(self.cells[j * self.nx + x].iter().map(|i| *i as usize));
            }
        }
        if (i1 - i0) * (j1 - j0) > 1 {
            out.sort_unstable();
            out.dedup();
        }
        out
    }
}

impl World<'_> {
    /// A standing building's rect as the tests take it: an ally's with its margin.
    fn blocking_rect(s: &Standing) -> Rect {
        if s.ours { s.rect } else { s.rect.grown(ALLY_MARGIN) }
    }

    /// What stands, as the street rule's blocks.
    pub fn blocking_rects(&self) -> Vec<Rect> {
        self.standing.iter().map(World::blocking_rect).collect()
    }

    /// The farm round the block's centre (`lattice`), made once: the free ground every construction turret within
    /// FARM_EXTENT reaches. `reach` is a turret's, `facts` the ground test's (the first asker's; a turret's and a
    /// frame's agree on land). None before the block is laid.
    pub fn farm(&self, reach: f32, facts: &Facts) -> Option<std::rc::Rc<Farm>> {
        let lattice = self.lattice?;
        Some(self.farm.borrow_mut().get_or_insert_with(|| std::rc::Rc::new(Farm::over(self, lattice.centre, reach, facts))).clone())
    }

    /// This question's kind among the kinds asked of this world so far, for `fixed_at`.
    fn fixed_kind(&self, f: &Facts, purpose: Purpose) -> u32 {
        let lens = [self.standing.len(), self.plants.len(), self.builders.len(), self.workers.len(), self.frames.len(), self.reservations.len(), self.refused.len(), usize::from(self.lattice.is_some())];
        let mut memo = self.fixed.borrow_mut();
        if memo.lens != lens {
            *memo = FixedMemo { lens, ..Default::default() };
        }
        match memo.kinds.iter().position(|(known, p)| *p == purpose && same_kind(known, f)) {
            Some(i) => i as u32,
            None => {
                memo.kinds.push((f.clone(), purpose));
                memo.kinds.len() as u32 - 1
            }
        }
    }

    /// The fixed part of a candidate of `kind` at `at` with `facing`, made by `make` when no question has come to the
    /// point before.
    fn fixed_at(&self, kind: u32, facing: i32, at: Vec3, make: impl FnOnce() -> Option<Fixed>) -> Option<u32> {
        let key = (kind, facing, at.x.to_bits(), at.z.to_bits());
        if let Some(known) = self.fixed.borrow().sites.get(&key) {
            return (*known != NO_SITE).then_some(*known);
        }
        let made = make();
        let mut memo = self.fixed.borrow_mut();
        let index = match made {
            Some(fixed) => {
                memo.made.push(fixed);
                memo.made.len() as u32 - 1
            }
            None => NO_SITE,
        };
        memo.sites.insert(key, index);
        (index != NO_SITE).then_some(index)
    }

    /// Whether a kind's footprint, as faced, stands at `at` (`footprint_stands`), asked by `make` once a point.
    fn footprint_at(&self, kind: u32, fp: Footprint, at: Vec3, make: impl FnOnce() -> bool) -> bool {
        let key = (kind, fp.x * 1_000 + fp.z, at.x.to_bits(), at.z.to_bits());
        if let Some(known) = self.fixed.borrow().footprints.get(&key) {
            return *known;
        }
        let stands = make();
        self.fixed.borrow_mut().footprints.insert(key, stands);
        stands
    }

    /// Whether a plant's products stand on every cell of `area` (`cells_open` over `Plant::stands`): by its own walk
    /// set counted from the set's sums, made once a set, where the area's cells are a block of the set's (they are,
    /// the terrain's cell being the grid's); cell by cell otherwise, and by `open` for a plant with no set.
    fn products_stand(&self, walk: Option<&crate::Passable>, area: &Rect) -> bool {
        let (cell, width, height) = self.ground.cells();
        let cell_by_cell = || {
            cells_open(area, &|p: Vec3| match walk {
                Some(walk) => self.ground.walks(walk, p.x, p.z),
                None => (self.open)(p),
            })
        };
        let Some(walk) = walk.filter(|_| cell > 0.0) else { return cell_by_cell() };
        // The cells `cells_open` would ask along a side, as it steps them: the first and how many, when they follow
        // one another.
        enum Side {
            Empty,
            OffTheMap,
            Ragged,
            Block(usize, usize),
        }
        let along = |from: f32, to: f32, count: usize| {
            let (mut first, mut n) = (0usize, 0usize);
            let mut v = from + GRID / 2.0;
            while v < to {
                // Off the map on the low side or the high: `Ground::walks` has no ground there.
                if v < 0.0 || (v / cell) as usize >= count {
                    return Side::OffTheMap;
                }
                let k = (v / cell) as usize;
                if n == 0 {
                    first = k;
                } else if k != first + n {
                    return Side::Ragged;
                }
                n += 1;
                v += GRID;
            }
            if n == 0 { Side::Empty } else { Side::Block(first, n) }
        };
        match (along(area.x0, area.x1, width), along(area.z0, area.z1, height)) {
            // No cell on a side is no cell at all; a cell off the map is one the products do not stand on.
            (Side::Empty, _) => true,
            (Side::OffTheMap | Side::Ragged | Side::Block(..), Side::Empty) => true,
            (Side::OffTheMap, _) | (_, Side::OffTheMap) => false,
            (Side::Ragged, _) | (_, Side::Ragged) => cell_by_cell(),
            (Side::Block(i0, nx), Side::Block(j0, nz)) => {
                let sums = self.walk_sums(walk);
                let at = |i: usize, j: usize| sums[j * (width + 1) + i];
                (at(i0 + nx, j0 + nz) + at(i0, j0)).wrapping_sub(at(i0 + nx, j0) + at(i0, j0 + nz)) as usize == nx * nz
            }
        }
    }

    /// A walk set's sums: at (i, j) the count of its cells before column i and row j, a row of `width + 1`.
    fn walk_sums(&self, walk: &crate::Passable) -> std::rc::Rc<Vec<u32>> {
        if let Some((_, sums)) = self.walk_sums.borrow().iter().find(|(known, _)| std::sync::Arc::ptr_eq(known, walk)) {
            return sums.clone();
        }
        let (_, width, height) = self.ground.cells();
        let mut sums = vec![0u32; (width + 1) * (height + 1)];
        for j in 0..height {
            let mut row = 0u32;
            for i in 0..width {
                row += u32::from(walk.get(j * width + i).copied().unwrap_or(false));
                sums[(j + 1) * (width + 1) + i + 1] = sums[j * (width + 1) + i + 1] + row;
            }
        }
        let sums = std::rc::Rc::new(sums);
        self.walk_sums.borrow_mut().push((walk.clone(), sums.clone()));
        sums
    }

    /// Plant `i`'s street flood over what stands, made once.
    pub fn plant_street(&self, i: usize) -> std::rc::Rc<Flood> {
        self.streets.borrow_mut().entry((true, i)).or_insert_with(|| std::rc::Rc::new(Flood::of_plant(&self.ground, &self.plants[i], &self.blocking_rects(), &*self.open))).clone()
    }

    /// Builder `i`'s way-out flood over what stands, made once.
    pub fn builder_way(&self, i: usize) -> std::rc::Rc<Flood> {
        self.streets.borrow_mut().entry((false, i)).or_insert_with(|| std::rc::Rc::new(Flood::of_builder(&self.ground, &self.builders[i], &self.blocking_rects(), &*self.open))).clone()
    }

    /// The indices in the buckets `r` touches, the buckets made for the list's present length (a world's lists are
    /// filled before it is asked, and a test that pushes to one between asks gets fresh buckets).
    fn near(cache: &std::cell::RefCell<Option<(usize, Buckets)>>, len: usize, make: impl FnOnce() -> Buckets, r: &Rect) -> Vec<usize> {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(n, _)| *n != len) {
            *cache = Some((len, make()));
        }
        cache.as_ref().map_or_else(Vec::new, |(_, b)| b.near(r))
    }

    /// What stands with a death blast that can reach `r`, with each one's index in `standing`, in index order: a blast
    /// is nothing past its radius, so a sum of `blast_at` over these is the sum over everything that stands, to the
    /// bit (until 2026-11-07 every candidate summed over every building of the base: a third of a builder menu's pass
    /// after the floods were cached, `site_bench`).
    fn blasts_near(&self, r: &Rect) -> impl Iterator<Item = (usize, &Standing)> {
        let reach = |s: &Standing| match s.blast {
            Some((radius, _)) if radius > 0.0 => {
                let c = s.rect.centre();
                Rect { x0: c.x - radius, z0: c.z - radius, x1: c.x + radius, z1: c.z + radius }
            }
            // No blast: a rect in no bucket.
            _ => Rect { x0: f32::MAX, z0: f32::MAX, x1: f32::MIN, z1: f32::MIN },
        };
        World::near(&self.blasts_near, self.standing.len(), || Buckets::new(self.ground.extent(), self.standing.iter().map(reach)), r).into_iter().map(|i| (i, &self.standing[i]))
    }

    /// `any` over a bucketed list, the buckets made as `near` makes them.
    fn any_near(cache: &std::cell::RefCell<Option<(usize, Buckets)>>, len: usize, make: impl FnOnce() -> Buckets, r: &Rect, hit: impl FnMut(usize) -> bool) -> bool {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(n, _)| *n != len) {
            *cache = Some((len, make()));
        }
        cache.as_ref().is_some_and(|(_, b)| b.any(r, hit))
    }

    /// Whether `hit` holds for anything standing in the buckets `r` touches (`standing_near(r).any(hit)`, with no
    /// list made: a candidate's first tests, asked at every lattice point).
    pub(crate) fn standing_any(&self, r: &Rect, mut hit: impl FnMut(&Standing) -> bool) -> bool {
        World::any_near(&self.standing_near, self.standing.len(), || Buckets::new(self.ground.extent(), self.standing.iter().map(World::blocking_rect)), r, |i| hit(&self.standing[i]))
    }

    /// Whether `hit` holds for any spot whose keep-out touches the buckets `r` touches.
    pub(crate) fn spots_any(&self, r: &Rect, mut hit: impl FnMut(&Vec3) -> bool) -> bool {
        let half = self.spot_keep_out.max(SPOT_BOX_HALF);
        World::any_near(&self.spots_near, self.spots.len(), || Buckets::new(self.ground.extent(), self.spots.iter().map(|p| Rect { x0: p.x - half, z0: p.z - half, x1: p.x + half, z1: p.z + half })), r, |i| hit(&self.spots[i]))
    }

    /// What stands in the buckets `r` touches, with each one's index in `standing`.
    pub(crate) fn standing_near(&self, r: &Rect) -> impl Iterator<Item = (usize, &Standing)> {
        World::near(&self.standing_near, self.standing.len(), || Buckets::new(self.ground.extent(), self.standing.iter().map(World::blocking_rect)), r).into_iter().map(|i| (i, &self.standing[i]))
    }

}

/// What a building is for: the score of level 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Purpose {
    /// A generator or basic converter: beside its own kind, in a bank of at most `cap` (None: unbounded).
    Generator { cap: Option<usize> },
    /// A construction turret: the pads and frames in its reach.
    Turret,
    /// A plant: the turrets in reach of its pad, then its facing toward the enemy.
    Plant,
    /// A frame (a reactor, a storage, an advanced converter, a silo): the build power in reach.
    Frame,
    /// A defence or a radar at the player's place: as near the place as the ground allows.
    Place,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub facts: Facts,
    pub purpose: Purpose,
    /// The band of distance: within `radius` of `centre`.
    pub centre: Vec3,
    pub radius: f32,
    pub builder_at: Vec3,
    pub builder: Facts,
}

/// A chosen site, with its reasons for the hands' line.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub at: Vec3,
    pub facing: i32,
    pub words: String,
}

struct Candidate {
    at: Vec3,
    facing: i32,
    rect: Rect,
    /// Level 1 (two parts), level 2, level 3, level 4; lower is better. Level 3, the builder's walk, is not a number
    /// until the bands come to it (`by_bands_from`): most candidates fall behind at the levels before it and are
    /// ordered there, and the walk is a look over every cell within the builder's reach of the site.
    scores: [f32; 5],
    /// Its fixed part, in the world's memo.
    fixed: u32,
}

/// A candidate's part that is the same whoever asks: a pass of the builder menus asks for one kind from every open
/// builder, a frame's and a turret's over one band (the farm's) and the rest over bands that overlap wherever two
/// builders stand near each other, and the ground, what stands, the aprons, the build power in reach and the
/// blasts at a point are the world's, not the asker's (1794201690-econ-flagship-22 at 9:22: a pass of 51 questions
/// tested 15 frames' and 15 plants' lattices from three builders, 0.8 s). Kept by the world (`World::fixed_at`).
pub struct Fixed {
    rect: Rect,
    level1: Level1,
    l2: f32,
    l4: f32,
    /// What its blast holds, to follow the level-1 words; empty when it holds nothing of ours.
    blast_words: String,
}

/// Level 1 as far as the world settles it.
enum Level1 {
    Generator { touching: usize },
    /// What it assists, and the words for that; with nothing to assist the asked place decides.
    Turret { assisted: usize, said: String },
    Plant { turrets: usize },
    /// The turrets in reach of the frame and their build power; the asker adds its own.
    Frame { turrets: usize, power: f32, build_time: f32 },
    Place,
}

impl Fixed {
    /// Level 1's two parts for this asker.
    fn level1(&self, req: &Request, at: Vec3, facing_rank: usize) -> [f32; 2] {
        match &self.level1 {
            Level1::Generator { touching } => [if *touching == 0 { FOUNDING } else { -(*touching as f32).min(TOUCHING_CAP) }, 0.0],
            // Nothing to assist yet: the turret prepares the ground for what comes (a nano before the fusions, the
            // people's order of a farm) and stands nearest the asked place. Until 2026-11-07 this was a refusal,
            // and a turret asked for at a place with nothing in reach went to the engine's search.
            Level1::Turret { assisted: 0, .. } => [at.dist2d(req.centre) / GRID, 0.0],
            Level1::Turret { assisted, .. } => [-(*assisted as f32), 0.0],
            Level1::Plant { turrets } => [-(*turrets as f32), facing_rank as f32],
            // The score is the seconds to stand at that power; the words say the power alone, what is measured,
            // not the clock (the user, 2026-11-08: the scripted reports do not predict the game's state).
            Level1::Frame { power, build_time, .. } => {
                let power = power + req.builder.build_speed;
                [if power > 0.0 { build_time / power } else { f32::INFINITY }, 0.0]
            }
            Level1::Place => [at.dist2d(req.centre), 0.0],
        }
    }

    /// The site's reasons for the hands' line.
    fn words(&self, req: &Request, at: Vec3, facing: i32) -> String {
        let plural = |n: usize| if n == 1 { "" } else { "s" };
        let words = match &self.level1 {
            Level1::Generator { touching: 0 } => "founding a bank".to_string(),
            Level1::Generator { touching } => format!("touching {touching} of its kind"),
            Level1::Turret { assisted: 0, .. } => format!("nothing in reach yet, {:.0} from the asked place", at.dist2d(req.centre)),
            Level1::Turret { said, .. } => said.clone(),
            Level1::Plant { turrets } => format!("{turrets} turret{} in reach, facing {}", plural(*turrets), ["south", "east", "north", "west"][facing.rem_euclid(4) as usize]),
            Level1::Frame { turrets, power, .. } => format!("{turrets} turret{} with {:.0} build power in reach", plural(*turrets), power + req.builder.build_speed),
            Level1::Place => String::new(),
        };
        format!("{words}{}", self.blast_words)
    }
}

/// The fixed parts made over a world: the kinds asked so far, each with every fact the tests read, and each tested
/// point by kind, facing and place. Emptied when the world's lists change length (a test that pushes to one between
/// asks, as the buckets have it).
#[derive(Default)]
pub struct FixedMemo {
    lens: [usize; 8],
    kinds: Vec<(Facts, Purpose)>,
    /// The fixed parts made, and each tested point's index among them (`NO_SITE` where a constraint failed).
    made: Vec<Fixed>,
    sites: HashMap<(u32, i32, u32, u32), u32, std::hash::BuildHasherDefault<PointHasher>>,
    /// Whether a plant's footprint stands at a point, by kind and the footprint as faced (`footprint_at`).
    footprints: HashMap<(u32, i32, u32, u32), bool, std::hash::BuildHasherDefault<PointHasher>>,
}

/// A tested point where a constraint failed.
const NO_SITE: u32 = u32::MAX;

/// The memo's hash: a point's key is four words, asked for every lattice point of every question, and the
/// standard hash was a fifth of a pass (`site_bench`). A multiply a word and a mix at the end.
#[derive(Default)]
pub struct PointHasher(u64);

impl std::hash::Hasher for PointHasher {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u32(u32::from(*b));
        }
    }

    fn write_u32(&mut self, n: u32) {
        self.0 = (self.0.rotate_left(5) ^ u64::from(n)).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_i32(&mut self, n: i32) {
        self.write_u32(n as u32);
    }

    fn finish(&self) -> u64 {
        let h = (self.0 ^ (self.0 >> 33)).wrapping_mul(0xff51_afd7_ed55_8ccd);
        h ^ (h >> 33)
    }
}

/// Whether two questions' kinds are one to every test of `fixed`: every fact to the bit, the products' ground the
/// same set (the memo holds the first asker's facts, so the set outlives the comparison).
fn same_kind(a: &Facts, b: &Facts) -> bool {
    let bits = |f: &Facts| [f.build_distance, f.build_speed, f.build_time, f.metal, f.health, f.height_dif, f.min_depth, f.max_depth, f.product_half].map(f32::to_bits);
    a.def == b.def
        && a.footprint == b.footprint
        && bits(a) == bits(b)
        && a.blast.map(|(r, d)| (r.to_bits(), d.to_bits())) == b.blast.map(|(r, d)| (r.to_bits(), d.to_bits()))
        && (a.floater, a.extractor, a.factory, a.building) == (b.floater, b.extractor, b.factory, b.building)
        && match (&a.product_passable, &b.product_passable) {
            (None, None) => true,
            (Some(x), Some(y)) => std::sync::Arc::ptr_eq(x, y),
            _ => false,
        }
}

/// The site for `req` in `world`, or why there is none.
pub fn choose(world: &World, req: &Request) -> Result<Site, String> {
    choose_traced(world, req, None)
}

/// `choose`, with every step written into `trace` when one is given: each widening's points, candidates and time,
/// and each tried candidate's verdict, with the walled builder's position against the site (the dump's replay,
/// `dump.rs`; site-probe-1 left open why a search that finds a site on flat ground fails in the game).
pub fn choose_traced(world: &World, req: &Request, mut trace: Option<&mut Vec<String>>) -> Result<Site, String> {
    let f = &req.facts;
    let facings: Vec<i32> = if f.factory { facings_toward(req.centre, world.toward).to_vec() } else { vec![0] };
    let bands = match req.purpose {
        Purpose::Generator { .. } => [BAND_GENERATOR, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Turret => [BAND_TURRET, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Plant => [BAND_TURRET, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Frame => [BAND_FRAME_SECONDS, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Place => [GRID, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
    };
    if !world.ground.depths_on_map(f) {
        let (lo, hi) = world.ground.height_bounds();
        let wants = if f.min_depth > 0.0 {
            format!("water at least {:.0} deep", f.min_depth)
        } else if f.max_depth < 1e6 {
            format!("ground above {:.0}", -f.max_depth)
        } else {
            "any ground".to_string()
        };
        return Err(format!("no ground on this map takes it: it wants {wants} and the map's heights run {lo:.0} to {hi:.0}"));
    }
    // The banks are a generator's alone.
    let banks = if matches!(req.purpose, Purpose::Generator { .. }) { bank_sizes(world, f.def) } else { HashMap::new() };
    let kind = world.fixed_kind(f, req.purpose);
    let mut why = String::new();
    // The candidates refused in any widening, by position and facing: the world is the same, so a street closed or
    // a builder walled in at the first widening is so at the next, and the next widening's tries are new sites.
    let mut refused: Vec<(Vec3, i32)> = Vec::new();
    // The blocks of the street rule: what stands; a candidate under test goes on top of a plant's or builder's flood.
    let blocks: Vec<Rect> = world.blocking_rects();
    // A street closed before the candidate is not the candidate's to close, nor a builder walled in already: until
    // 2026-11-04 a street closed by anything (an ally's unit in its own lab's door, a door walled by earlier
    // fallbacks) read as closed by every site within 1,200 of the plant, and the whole base went to the engine's
    // search (1793769060-human, seats 2 and 3 from 1:04 and 5:21). Found once per plant or builder, as it comes
    // into question.
    let mut open_before: Vec<Option<bool>> = vec![None; world.plants.len()];
    let mut out_before: Vec<Option<bool>> = vec![None; world.builders.len()];
    for widening in 0..=WIDENINGS {
        let started = std::time::Instant::now();
        let radius = req.radius * 2f32.powi(widening as i32);
        // The flood for this widening alone: most searches end at the first, and a flood over four times the band
        // is sixteen times the cells. Made when the bands first ask a walk of it.
        let walk: std::cell::OnceCell<std::rc::Rc<Walk>> = std::cell::OnceCell::new();
        let reach = f.reach_from(&req.builder);
        // The cells within the builder's reach of a site lie the same way round every point of one lattice: found
        // at the first candidate asked, read by the rest.
        let disc: std::cell::OnceCell<Option<Disc>> = std::cell::OnceCell::new();
        let walked = |c: &Candidate| {
            let walk = walk.get_or_init(|| Walk::cached(world, req.builder_at, req.centre, radius));
            let disc = disc.get_or_init(|| walk.disc(c.at, reach));
            disc.as_ref().and_then(|d| walk.to_within_disc(d, c.at)).unwrap_or_else(|| walk.to_within(c.at, reach))
        };
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut points = 0usize;
        for (rank, facing) in facings.iter().enumerate() {
            let fp = f.footprint.faced(*facing);
            for at in lattice(fp, req.centre, radius) {
                points += 1;
                if let Some(c) = candidate(world, req, kind, &banks, fp, at, *facing, rank) {
                    candidates.push(c);
                }
            }
        }
        if let Some(t) = trace.as_deref_mut() {
            t.push(format!("widening {widening}: radius {radius:.0} round ({:.0}, {:.0}), {points} points, {} candidates, {:.1} ms", req.centre.x, req.centre.z, candidates.len(), started.elapsed().as_secs_f32() * 1000.0));
        }
        if candidates.is_empty() {
            why = format!("no ground within {radius:.0} of ({:.0}, {:.0}) takes a {} there", req.centre.x, req.centre.z, f.footprint.x * 8);
            continue;
        }
        let ordered = by_bands(candidates, &bands, &walked);
        let tries: Vec<&Candidate> = ordered.iter().filter(|c| !refused.iter().any(|(p, facing)| *facing == c.facing && p.dist2d(c.at) < 1.0)).take(STREET_TRIES).collect();
        for (n, c) in tries.into_iter().enumerate() {
            let mut verdict = |what: String| {
                if let Some(t) = trace.as_deref_mut() {
                    // A try from behind the bands was never asked its walk.
                    let mut scores = c.scores;
                    if scores[3].is_nan() {
                        scores[3] = walked(c);
                    }
                    t.push(format!("  try {n}: ({:.0}, {:.0}) facing {} scores {scores:?}: {what}", c.at.x, c.at.z, c.facing));
                }
            };
            let closed = world.plants.iter().enumerate().filter(|(_, p)| p.concerns(&c.rect)).find(|(i, _)| {
                let street = world.plant_street(*i);
                let was_open = *open_before[*i].get_or_insert_with(|| street.reaches_the_open());
                was_open && !street.reaches_the_open_round(&[c.rect])
            });
            if let Some((_, p)) = closed {
                why = format!("the best site at ({:.0}, {:.0}) would close {}'s street for its {}", c.at.x, c.at.z, p.name, p.widest);
                verdict(format!("closes {}'s street", p.name));
                refused.push((c.at, c.facing));
                continue;
            }
            // The asker is among them when the site is within its reach from where it stands, since then it builds
            // without moving: beyond its reach it walks to the site and stands beside what it builds, and the flood
            // from where it stands now says nothing (six of the eight walled-in dumps of 2026-11-07 named such an
            // asker, 46 to 95 from the site's edge). Left out altogether, as it was from that review until
            // 1794106612-human on Full Metal Plate, all three commanders shut the last way out of their own stands
            // with a turbine or an extractor built from where they stood.
            let checked = |b: &Builder| b.at.dist2d(req.builder_at) >= 1.0 || distance_to(&c.rect, b.at) <= req.builder.build_distance;
            let walled = world.builders.iter().enumerate().filter(|(_, b)| checked(b) && distance_to(&c.rect, b.at) <= BUILDER_NEAR && b.concerns(&c.rect)).find(|(i, _)| {
                let way = world.builder_way(*i);
                let had_way = *out_before[*i].get_or_insert_with(|| way.reaches_the_open());
                had_way && !way.reaches_the_open_round(&[c.rect])
            });
            if let Some((i, b)) = walled {
                why = format!("the best site at ({:.0}, {:.0}) would wall in our builder at ({:.0}, {:.0})", c.at.x, c.at.z, b.at.x, b.at.z);
                verdict(format!("walls in builder {i} at ({:.0}, {:.0}), {:.0} from the site's edge, {}", b.at.x, b.at.z, distance_to(&c.rect, b.at), if c.rect.contains(b.at) { "standing under the site" } else { "beside it" }));
                refused.push((c.at, c.facing));
                continue;
            }
            // A plant's own street, from the site it would stand at.
            if f.factory {
                let own = Plant { centre: c.at, footprint: f.footprint, facing: c.facing, product_half: f.product_half, ours: true, name: "it".into(), widest: String::new(), passable: f.product_passable.clone() };
                if !Flood::of_plant(&world.ground, &own, &blocks, &*world.open).reaches_the_open_round(&[c.rect]) {
                    why = format!("the best site at ({:.0}, {:.0}) has no street to the open", c.at.x, c.at.z);
                    verdict("no street of its own to the open".to_string());
                    refused.push((c.at, c.facing));
                    continue;
                }
            }
            let words = world.fixed.borrow().made[c.fixed as usize].words(req, c.at, c.facing);
            verdict(format!("chosen: {words}"));
            return Ok(Site { at: c.at, facing: c.facing, words });
        }
    }
    if let Some(t) = trace {
        t.push(format!("no site: {why}"));
    }
    Err(why)
}

/// One lattice point as a candidate, or None when a constraint fails: its fixed part from the world's memo (made
/// here when this is the first question to come to the point), and the asker's own on top.
fn candidate(world: &World, req: &Request, kind: u32, banks: &HashMap<usize, usize>, fp: Footprint, at: Vec3, facing: i32, facing_rank: usize) -> Option<Candidate> {
    let index = world.fixed_at(kind, facing, at, || fixed(world, &req.facts, req.purpose, kind, banks, fp, at, facing))?;
    let memo = world.fixed.borrow();
    let fixed = &memo.made[index as usize];
    let l1 = fixed.level1(req, at, facing_rank);
    Some(Candidate { at, facing, rect: fixed.rect, scores: [l1[0], l1[1], fixed.l2, f32::NAN, fixed.l4], fixed: index })
}

/// Whether a kind's footprint stands at a lattice point, whichever way it faces: nothing of ours or theirs under it,
/// off the spots, the aprons, the reservations and the refused sites, on ground that takes it, and for a plant on
/// ground its products stand on. The tests in the order of their cost: each is a refusal of its own, and the ground
/// under a plant's footprint is sixty-four cells to read.
fn footprint_stands(world: &World, f: &Facts, purpose: Purpose, fp: Footprint, at: Vec3) -> bool {
    let rect = fp.rect(at);
    // No site over a mobile builder of ours: the sites under the asking builder are the best by every band (its
    // walk to them is nothing) and were the twelve tries of every widening, each refused as walling it in, so no
    // search anchored at a builder reached a site beside it (site-dump-1, 2026-11-05; 6,814 of an 8v8's 6,910
    // refusals).
    if world.builders.iter().any(|b| b.body().overlaps(&rect)) {
        return false;
    }
    if world.reservations.iter().any(|r| r.rect.overlaps(&rect) && !r.admits(f)) {
        return false;
    }
    if purpose == Purpose::Turret && world.lattice.is_some_and(|l| !l.holds(at)) {
        return false;
    }
    if world.refused.iter().any(|(d, p)| *d == f.def && p.dist2d(at) < REFUSED_RADIUS) {
        return false;
    }
    if world.plants.iter().any(|p| p.apron().overlaps(&rect)) {
        return false;
    }
    if world.standing_any(&rect, |s| World::blocking_rect(s).overlaps(&rect)) {
        return false;
    }
    if world.spots_any(&rect, |p| p.dist2d(at) < world.spot_keep_out || Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }.overlaps(&rect)) {
        return false;
    }
    if !world.ground.stands(fp, at, f) {
        return false;
    }
    // A plant's footprint is its products' own ground: a lab may stand on 17 of height difference, a tank climbs
    // less, and a plant over a knoll holds its products inside (econ-flagship-20: the advanced plant at (920, 2952)
    // on a knoll with slope bytes to 60 against the tank's 28, two advanced constructors penned from birth, the
    // plant the only thing whose removal freed them).
    !f.factory || world.products_stand(f.product_passable.as_ref(), &rect)
}

/// What a lattice point is for a kind whoever asks, from wherever, for whatever band: every constraint, level 1 as
/// far as the world settles it, levels 2 and 4. None when a constraint fails.
fn fixed(world: &World, f: &Facts, purpose: Purpose, kind: u32, banks: &HashMap<usize, usize>, fp: Footprint, at: Vec3, facing: i32) -> Option<Fixed> {
    let rect = fp.rect(at);
    // A plant is asked at every point once a facing, and what its footprint needs is the same for each: asked of
    // the memo, by the footprint as faced.
    let footprint_stands = if f.factory { world.footprint_at(kind, fp, at, || footprint_stands(world, f, purpose, fp, at)) } else { footprint_stands(world, f, purpose, fp, at) };
    if !footprint_stands {
        return None;
    }
    if f.factory {
        let own = Plant { centre: at, footprint: f.footprint, facing, product_half: f.product_half, ours: true, name: String::new(), widest: String::new(), passable: f.product_passable.clone() };
        let apron = own.apron();
        if !world.ground.in_map(&apron) || world.standing_any(&apron, |s| s.rect.overlaps(&apron)) || world.reservations.iter().any(|r| r.rect.overlaps(&apron) && !r.admits(f)) {
            return None;
        }
        if world.spots_any(&apron, |p| Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }.overlaps(&apron)) {
            return None;
        }
        // The apron is the products' own ground (water-1: every shipyard site was refused by the land's apron check).
        if !world.products_stand(f.product_passable.as_ref(), &apron) {
            return None;
        }
    }
    // Level 1, as far as the world settles it (`Fixed::level1` and `Fixed::words` finish it for an asker).
    let level1 = match purpose {
        Purpose::Generator { cap } => {
            // Its kind within a street's width: the touching and gap tests reach no farther.
            let same: Vec<(usize, &Standing)> = world.standing_near(&rect.grown(BANK_STREET)).filter(|(_, s)| s.ours && s.def == Some(f.def)).collect();
            let touching: Vec<usize> = same.iter().filter(|(_, s)| s.rect.touches(&rect)).map(|(i, _)| *i).collect();
            // Touching its kind, or a street apart from it: a gap under the street's width is neither. Meeting at a
            // corner (a gap of nothing) is packing, not a gap: before this, the only legal touching site on a bank's
            // flat side was a notch touching two, and the banks grew as staircases (block-2).
            if same.iter().any(|(_, s)| !s.rect.touches(&rect) && { let g = gap(&s.rect, &rect); g > 0.0 && g < BANK_STREET }) {
                return None;
            }
            if let Some(cap) = cap {
                let bank = touching.iter().map(|i| banks.get(i).copied().unwrap_or(1)).max().unwrap_or(0);
                if bank + 1 > cap {
                    return None;
                }
            }
            Level1::Generator { touching: touching.len() }
        }
        Purpose::Turret => {
            let reach = f.build_distance;
            let frames = world.frames.iter().filter(|fr| distance_to(&fr.rect, at) <= reach).count();
            let plural = |n: usize| if n == 1 { "" } else { "s" };
            let (assisted, said) = if let Some(farm) = world.farm(reach, f) {
                // The block is laid: a turret is for the free farm ground it shares with the turrets standing (the
                // farm grows as one piece and the power stacks where the frames will go) and the frames it
                // reaches, and a plant's pad counts for nothing (econ-flagship-5: turrets scored by pads stood by
                // the running advanced plant, whose product took their power by the engine's nearest rule, while
                // the frames stood at the block's edge with a third of the turrets in reach). Until 2026-11-08 the
                // block's reserved frame slots stood here; the farm-holds design §1.3.
                let gain = farm.gain(at, reach);
                let frame_cells = world.farm_frame.map_or(1, crate::farm::cells_of).max(1);
                let worth = gain / frame_cells;
                (worth + frames, format!("{worth} frame{}' worth of free farm ground and {frames} frame{} in reach", if worth == 1 { "" } else { "s" }, plural(frames)))
            } else {
                let pads = world.plants.iter().filter(|p| distance_to(&p.rect(), at) <= reach).count();
                (pads + frames, format!("{pads} pad{} and {frames} frame{} in reach", plural(pads), plural(frames)))
            };
            Level1::Turret { assisted, said }
        }
        Purpose::Plant => {
            Level1::Plant { turrets: world.workers.iter().filter(|w| distance_to(&rect, w.at) <= w.reach).count() }
        }
        Purpose::Frame => {
            let in_reach: Vec<&Worker> = world.workers.iter().filter(|w| distance_to(&rect, w.at) <= w.reach).collect();
            Level1::Frame { turrets: in_reach.len(), power: in_reach.iter().map(|w| w.power).sum::<f32>(), build_time: f.build_time }
        }
        Purpose::Place => Level1::Place,
    };
    // Level 2: the chain sum.
    let own_blast = f.blast;
    let taken: f32 = world.blasts_near(&rect).filter_map(|(_, s)| s.blast.map(|b| blast_at(b, s.rect.centre(), &rect))).sum();
    let killed: Vec<&Standing> = match own_blast {
        Some(b) => world.standing_near(&Rect { x0: at.x - b.0, z0: at.z - b.0, x1: at.x + b.0, z1: at.z + b.0 }).map(|(_, s)| s).filter(|s| s.ours && s.health > 0.0 && blast_at(b, at, &s.rect) >= s.health).collect(),
        None => Vec::new(),
    };
    let kills: f32 = killed.iter().map(|s| s.metal).sum();
    let l2 = kills + if f.health > 0.0 && taken >= f.health { f.metal } else { 0.0 };
    let mut blast_words = String::new();
    if kills > 0.0 {
        // The bill, reported not bounded (§1.3): what its death would take, by kind.
        let mut by_kind: Vec<(String, usize)> = Vec::new();
        for s in &killed {
            match by_kind.iter_mut().find(|(k, _)| *k == s.kind) {
                Some((_, n)) => *n += 1,
                None => by_kind.push((s.kind.clone(), 1)),
            }
        }
        by_kind.sort_by(|a, b| b.1.cmp(&a.1));
        let kinds: Vec<String> = by_kind.iter().map(|(k, n)| format!("{n} {k}")).collect();
        blast_words = format!("; its blast holds {}, {kills:.0} metal of ours", kinds.join(" and "));
    }
    // Level 3, the builder's walk to within reach of the site, round what stands, is asked by the bands.
    let l4 = world.home.dist2d(at);
    Some(Fixed { rect, level1, l2, l4, blast_words })
}

/// The builder's walk: a flood over the cells from where it stands, round what stands (ours, allies' with their
/// margin) and over ground its units take, so a site across a bank reads as far as the way round it (block-2: the
/// commander rounded a wall of solars 700 long for a site that touched two of them, 560 off in a line). One flood a
/// choice, over a window that holds the builder and the widest band.
pub struct Walk {
    x0: f32,
    z0: f32,
    nx: usize,
    nz: usize,
    dist: Vec<f32>,
}

impl Walk {
    /// The flood from `from` for a search of `radius` about `centre`, made once per world.
    fn cached(world: &World, from: Vec3, centre: Vec3, radius: f32) -> std::rc::Rc<Walk> {
        let cell = |v: f32| (v / GRID).floor() as i32;
        let key = (cell(from.x), cell(from.z), cell(centre.x), cell(centre.z), radius.round() as i32);
        world.walks.borrow_mut().entry(key).or_insert_with(|| std::rc::Rc::new(Walk::new(world, from, centre, radius))).clone()
    }

    fn new(world: &World, from: Vec3, centre: Vec3, radius: f32) -> Walk {
        let (w, h) = world.ground.extent();
        let half = radius + from.dist2d(centre) + 2.0 * GRID;
        let x0 = ((from.x - half) / GRID).floor().max(0.0) * GRID;
        let z0 = ((from.z - half) / GRID).floor().max(0.0) * GRID;
        let x1 = ((from.x + half) / GRID).ceil() * GRID;
        let z1 = ((from.z + half) / GRID).ceil() * GRID;
        let (x1, z1) = (x1.min((w / GRID).floor() * GRID), z1.min((h / GRID).floor() * GRID));
        let nx = ((x1 - x0) / GRID).max(0.0) as usize;
        let nz = ((z1 - z0) / GRID).max(0.0) as usize;
        let mut blocked = vec![false; nx * nz];
        for s in &world.standing {
            let r = if s.ours { s.rect } else { s.rect.grown(ALLY_MARGIN) };
            let (i0, i1) = (((r.x0 - x0) / GRID).floor().max(0.0) as usize, (((r.x1 - x0) / GRID).ceil().max(0.0) as usize).min(nx));
            let (j0, j1) = (((r.z0 - z0) / GRID).floor().max(0.0) as usize, (((r.z1 - z0) / GRID).ceil().max(0.0) as usize).min(nz));
            for j in j0..j1 {
                for i in i0..i1 {
                    blocked[j * nx + i] = true;
                }
            }
        }
        let start = (((from.x - x0) / GRID) as usize, ((from.z - z0) / GRID) as usize);
        if nx == 0 || nz == 0 || start.0 >= nx || start.1 >= nz {
            return Walk { x0, z0, nx, nz, dist: vec![f32::INFINITY; nx * nz] };
        }
        // The least walk to every cell, a straight step 16 and a diagonal 23. The steps are whole and at most 23, so
        // the cells still to settle lie within 24 of the nearest of them and a ring of 24 buckets holds them in
        // order with no heap; a cell's ground is asked once, when a walk first comes to it (until 2026-11-09 a heap,
        // and the ground asked of every neighbour of every settled cell: half of a builder menu's pass, `site_bench`).
        const RING: usize = 24;
        const UNSEEN: u8 = 0;
        const OPEN: u8 = 1;
        const SHUT: u8 = 2;
        let mut ground: Vec<u8> = blocked.iter().map(|b| if *b { SHUT } else { UNSEEN }).collect();
        let mut best = vec![u32::MAX; nx * nz];
        let mut ring: [Vec<u32>; RING] = Default::default();
        let s = start.1 * nx + start.0;
        best[s] = 0;
        ring[0].push(s as u32);
        let (mut waiting, mut d) = (1usize, 0u32);
        let mut settled: Vec<u32> = Vec::new();
        while waiting > 0 {
            std::mem::swap(&mut settled, &mut ring[d as usize % RING]);
            waiting -= settled.len();
            for &k in &settled {
                let k = k as usize;
                // Reached sooner since it was put here.
                if best[k] != d {
                    continue;
                }
                let (i, j) = (k % nx, k / nx);
                for (di, dj) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    let (ni, nj) = (i as i32 + di, j as i32 + dj);
                    if ni < 0 || nj < 0 || ni >= nx as i32 || nj >= nz as i32 {
                        continue;
                    }
                    let (ni, nj) = (ni as usize, nj as usize);
                    let n = nj * nx + ni;
                    if ground[n] == UNSEEN {
                        ground[n] = if (world.open)(Vec3 { x: x0 + (ni as f32 + 0.5) * GRID, y: 0.0, z: z0 + (nj as f32 + 0.5) * GRID }) { OPEN } else { SHUT };
                    }
                    if ground[n] == SHUT {
                        continue;
                    }
                    let nd = d + if di != 0 && dj != 0 { 23 } else { 16 };
                    if nd < best[n] {
                        best[n] = nd;
                        ring[nd as usize % RING].push(n as u32);
                        waiting += 1;
                    }
                }
            }
            settled.clear();
            d += 1;
        }
        Walk { x0, z0, nx, nz, dist: best.into_iter().map(|d| if d == u32::MAX { f32::INFINITY } else { d as f32 }).collect() }
    }

    /// The cells `to_within` looks over round `p`, as rows: for each row of cells the first and last column whose
    /// centre lies within `reach`. The test is `to_within`'s own, on the same numbers: with the point and the
    /// window's corner on half elmos (a lattice point is on the build grid, the window on the walk's) every
    /// difference is exact, so the rows are the same round any point that lies the same way in its cell. None for a
    /// point off the half elmos.
    fn disc(&self, p: Vec3, reach: f32) -> Option<Disc> {
        if ![p.x, p.z, self.x0, self.z0].into_iter().all(on_half_elmos) {
            return None;
        }
        let cells = (reach / GRID).ceil() as i32;
        let (ci, cj) = (((p.x - self.x0) / GRID).floor() as i32, ((p.z - self.z0) / GRID).floor() as i32);
        let (fx, fz) = (p.x - (self.x0 + ci as f32 * GRID), p.z - (self.z0 + cj as f32 * GRID));
        let mut rows = Vec::new();
        for dj in -cells..=cells {
            let dz = (dj as f32 + 0.5) * GRID - fz;
            let within: Vec<i32> = (-cells..=cells).filter(|di| { let dx = (*di as f32 + 0.5) * GRID - fx; dx * dx + dz * dz <= reach * reach }).collect();
            if let (Some(first), Some(last)) = (within.first(), within.last()) {
                rows.push((dj, *first, *last));
            }
        }
        Some(Disc { fx, fz, rows })
    }

    /// `to_within` by a disc's rows; None when `p` does not lie in its cell as the disc's point did.
    fn to_within_disc(&self, disc: &Disc, p: Vec3) -> Option<f32> {
        if !on_half_elmos(p.x) || !on_half_elmos(p.z) {
            return None;
        }
        let (ci, cj) = (((p.x - self.x0) / GRID).floor() as i32, ((p.z - self.z0) / GRID).floor() as i32);
        if p.x - (self.x0 + ci as f32 * GRID) != disc.fx || p.z - (self.z0 + cj as f32 * GRID) != disc.fz {
            return None;
        }
        let mut best = f32::INFINITY;
        for (dj, first, last) in &disc.rows {
            let j = cj + dj;
            let (i0, i1) = ((ci + first).max(0), (ci + last).min(self.nx as i32 - 1));
            if j < 0 || j >= self.nz as i32 || i0 > i1 {
                continue;
            }
            let row = j as usize * self.nx;
            best = self.dist[row + i0 as usize..=row + i1 as usize].iter().fold(best, |least, d| least.min(*d));
        }
        Some(best)
    }

    /// The walk to within `reach` of `p`: the least distance to a cell whose centre lies within `reach` of it, so a
    /// site the builder cannot stand on (one on the water, for a builder on land) is reached from the shore.
    /// Infinite when the flood reached no such cell.
    fn to_within(&self, p: Vec3, reach: f32) -> f32 {
        let cells = (reach / GRID).ceil() as i32;
        let (ci, cj) = (((p.x - self.x0) / GRID).floor() as i32, ((p.z - self.z0) / GRID).floor() as i32);
        let mut best = f32::INFINITY;
        for j in (cj - cells).max(0)..=(cj + cells).min(self.nz as i32 - 1) {
            for i in (ci - cells).max(0)..=(ci + cells).min(self.nx as i32 - 1) {
                let (dx, dz) = (self.x0 + (i as f32 + 0.5) * GRID - p.x, self.z0 + (j as f32 + 0.5) * GRID - p.z);
                if dx * dx + dz * dz <= reach * reach {
                    best = best.min(self.dist[j as usize * self.nx + i as usize]);
                }
            }
        }
        best
    }
}

/// The rows of cells within a reach of a point (`Walk::disc`), and where the point lay in its cell.
struct Disc {
    fx: f32,
    fz: f32,
    /// (row from the point's, first column, last column from the point's).
    rows: Vec<(i32, i32, i32)>,
}

/// A whole number of half elmos, small enough that sums and differences of such numbers are exact.
fn on_half_elmos(v: f32) -> bool {
    v.abs() < 1_048_576.0 && (v * 2.0).fract() == 0.0
}

/// The candidates in the order of the bands: at each level those within the band of the best go on to the next
/// level's order; the rest follow them in this level's order, so the street check has them to fall back to.
/// `walked` is a candidate's level 3, asked of those the earlier levels' bands let through.
fn by_bands(candidates: Vec<Candidate>, bands: &[f32; 5], walked: &dyn Fn(&Candidate) -> f32) -> Vec<Candidate> {
    by_bands_from(candidates, bands, 0, walked)
}

fn by_bands_from(mut candidates: Vec<Candidate>, bands: &[f32; 5], level: usize, walked: &dyn Fn(&Candidate) -> f32) -> Vec<Candidate> {
    if level >= 5 || candidates.len() <= 1 {
        return candidates;
    }
    if level == 3 {
        for c in candidates.iter_mut() {
            c.scores[3] = walked(c);
        }
    }
    let best = candidates.iter().map(|c| c.scores[level]).fold(f32::INFINITY, f32::min);
    let (within, mut rest): (Vec<Candidate>, Vec<Candidate>) = candidates.into_iter().partition(|c| c.scores[level] <= best + bands[level]);
    rest.sort_by(|a, b| a.scores[level].total_cmp(&b.scores[level]));
    let mut out = by_bands_from(within, bands, level + 1, walked);
    out.extend(rest);
    out
}

/// The size of the bank each of our standing buildings of `def` belongs to (touching ones), by index.
fn bank_sizes(world: &World, def: UnitDefId) -> HashMap<usize, usize> {
    let same: Vec<usize> = world.standing.iter().enumerate().filter(|(_, s)| s.ours && s.def == Some(def)).map(|(i, _)| i).collect();
    let mut bank_of: HashMap<usize, usize> = HashMap::new();
    let mut sizes: HashMap<usize, usize> = HashMap::new();
    for &i in &same {
        if bank_of.contains_key(&i) {
            continue;
        }
        let mut members = vec![i];
        let mut stack = vec![i];
        bank_of.insert(i, i);
        while let Some(j) = stack.pop() {
            for &k in &same {
                if !bank_of.contains_key(&k) && world.standing[j].rect.touches(&world.standing[k].rect) {
                    bank_of.insert(k, i);
                    members.push(k);
                    stack.push(k);
                }
            }
        }
        for m in &members {
            sizes.insert(*m, members.len());
        }
    }
    sizes
}

/// The centres on the build grid for this footprint within `radius` of `centre`.
fn lattice(fp: Footprint, centre: Vec3, radius: f32) -> Vec<Vec3> {
    let mut out = Vec::new();
    let first = fp.snap(Vec3 { x: centre.x - radius, y: centre.y, z: centre.z - radius });
    let steps = (2.0 * radius / GRID) as i32 + 1;
    for i in 0..=steps {
        for j in 0..=steps {
            let p = Vec3 { x: first.x + i as f32 * GRID, y: centre.y, z: first.z + j as f32 * GRID };
            if p.dist2d(centre) <= radius {
                out.push(p);
            }
        }
    }
    out
}

/// Whether every cell of `area` is passable by `open`: a plant's apron and its own footprint for its products.
fn cells_open(area: &Rect, open: &dyn Fn(Vec3) -> bool) -> bool {
    let mut z = area.z0 + GRID / 2.0;
    while z < area.z1 {
        let mut x = area.x0 + GRID / 2.0;
        while x < area.x1 {
            if !open(Vec3 { x, y: 0.0, z }) {
                return false;
            }
            x += GRID;
        }
        z += GRID;
    }
    true
}

/// The distance from a point to a rect's nearest ground; 0 inside.
/// The distance from a point to a rectangle, 0 inside it.
pub fn distance_to(r: &Rect, p: Vec3) -> f32 {
    let dx = (r.x0 - p.x).max(0.0).max(p.x - r.x1);
    let dz = (r.z0 - p.z).max(0.0).max(p.z - r.z1);
    (dx * dx + dz * dz).sqrt()
}

/// The gap between two rects along the axis they are farthest apart on; negative when they overlap.
fn gap(a: &Rect, b: &Rect) -> f32 {
    let dx = (b.x0 - a.x1).max(a.x0 - b.x1);
    let dz = (b.z0 - a.z1).max(a.z0 - b.z1);
    dx.max(dz)
}

/// A blast (radius, damage) from `from` on a victim covering `victim`: the damage falls linearly to the radius,
/// measured to the victim's surface (F-fight-death).
fn blast_at((radius, damage): (f32, f32), from: Vec3, victim: &Rect) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    let d = distance_to(victim, from);
    (damage * (1.0 - d / radius)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::street::{street_open, way_out};
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// The walk's flood against the plain search it replaced (a heap, the ground asked at every step): the same
    /// walk to every cell, round buildings, over closed ground, from a start in the open and from one shut in.
    #[test]
    fn the_walks_flood_is_the_least_walk_to_every_cell() {
        let t = flat();
        // Closed ground in stripes with gaps, so the walks bend.
        let open = |p: Vec3| !((p.x / 16.0) as i32 % 7 == 3 && (p.z / 16.0) as i32 % 5 != 0) && !(p.z > 2300.0 && p.z < 2330.0 && p.x < 2200.0);
        let mut w = world(&t, &open);
        let mut seed = 12345u32;
        let mut next = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        for _ in 0..120 {
            let fp = [2, 4, 6, 10][(next() * 4.0) as usize % 4];
            w.standing.push(standing(3, fp, Footprint { x: fp, z: fp }.snap(at(1500.0 + next() * 1100.0, 1500.0 + next() * 1100.0)), 100.0, 100.0, None));
        }
        w.standing[7].ours = false;
        let plain = |from: Vec3, centre: Vec3, radius: f32| -> Vec<f32> {
            let made = Walk::new(&w, from, centre, radius);
            let (nx, nz, x0, z0) = (made.nx, made.nz, made.x0, made.z0);
            let mut blocked = vec![false; nx * nz];
            for s in &w.standing {
                let r = if s.ours { s.rect } else { s.rect.grown(ALLY_MARGIN) };
                for j in 0..nz {
                    for i in 0..nx {
                        let (cx0, cz0) = (x0 + i as f32 * GRID, z0 + j as f32 * GRID);
                        if cx0 < r.x1 && cx0 + GRID > r.x0 && cz0 < r.z1 && cz0 + GRID > r.z0 {
                            blocked[j * nx + i] = true;
                        }
                    }
                }
            }
            let mut dist = vec![f32::INFINITY; nx * nz];
            let start = (((from.x - x0) / GRID) as usize, ((from.z - z0) / GRID) as usize);
            let mut heap: std::collections::BinaryHeap<(std::cmp::Reverse<u32>, usize)> = std::collections::BinaryHeap::new();
            dist[start.1 * nx + start.0] = 0.0;
            heap.push((std::cmp::Reverse(0), start.1 * nx + start.0));
            while let Some((std::cmp::Reverse(d), k)) = heap.pop() {
                if d as f32 > dist[k] {
                    continue;
                }
                let (i, j) = (k % nx, k / nx);
                for (di, dj) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    let (ni, nj) = (i as i32 + di, j as i32 + dj);
                    if ni < 0 || nj < 0 || ni >= nx as i32 || nj >= nz as i32 {
                        continue;
                    }
                    let n = nj as usize * nx + ni as usize;
                    if blocked[n] || !open(Vec3 { x: x0 + (ni as f32 + 0.5) * GRID, y: 0.0, z: z0 + (nj as f32 + 0.5) * GRID }) {
                        continue;
                    }
                    let nd = d + if di != 0 && dj != 0 { 23 } else { 16 };
                    if (nd as f32) < dist[n] {
                        dist[n] = nd as f32;
                        heap.push((std::cmp::Reverse(nd), n));
                    }
                }
            }
            dist
        };
        let mut reached = 0;
        for (from, centre, radius) in [(at(2000.0, 2000.0), at(2000.0, 2000.0), 300.0), (at(1400.0, 1400.0), at(2100.0, 2050.0), 600.0), (at(2650.0, 2700.0), at(1800.0, 1900.0), 400.0), (w.standing[0].rect.centre(), at(2000.0, 2000.0), 500.0), (at(40.0, 40.0), at(300.0, 200.0), 1200.0)] {
            let made = Walk::new(&w, from, centre, radius);
            let want = plain(from, centre, radius);
            assert_eq!(made.dist.len(), want.len());
            assert!(made.dist.iter().zip(&want).all(|(a, b)| a == b), "the flood from {from:?} for {radius} round {centre:?} differs");
            reached += want.iter().filter(|d| d.is_finite() && **d > 0.0).count();
        }
        assert!(reached > 10_000, "the floods reached {reached} cells");
    }

    /// One world asked by many builders for many kinds answers each as a world asked that question alone: the memo of
    /// fixed parts, the footprints' and the walk sets' sums change no answer and no word.
    #[test]
    fn one_world_answers_every_asker_as_a_fresh_world_does() {
        let t = flat();
        let open = |p: Vec3| !(p.x > 2500.0 && p.x < 2540.0 && p.z < 2300.0);
        // The products' ground: all of it but a strip, so some plant sites fail on their footprint or apron.
        let walk: crate::Passable = (0..256 * 256).map(|k| { let (i, j) = (k % 256, k / 256); !(i > 118 && i < 122 && j > 100) }).collect::<Vec<bool>>().into();
        let make = || {
            let mut w = world(&t, &open);
            let mut seed = 99u32;
            let mut next = move || {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed >> 8) as f32 / (1u32 << 24) as f32
            };
            for k in 0..60 {
                let (def, fp) = [(3, 10), (4, 6), (5, 4)][k % 3];
                w.standing.push(standing(def, fp, Footprint { x: fp, z: fp }.snap(at(1700.0 + next() * 700.0, 1700.0 + next() * 700.0)), 150.0, 300.0, (k % 4 == 0).then_some((120.0, 400.0))));
            }
            for k in 0..8 {
                w.workers.push(Worker { at: at(2050.0 + 40.0 * k as f32, 2420.0), reach: 400.0, power: 200.0 });
            }
            w.frames.push(Frame { rect: Footprint { x: 12, z: 12 }.rect(at(2200.0, 2500.0)), metal: 4000.0 });
            w.builders = [(1800.0, 1650.0), (2300.0, 2450.0), (1650.0, 2300.0), (2420.0, 1700.0)].iter().map(|(x, z)| Builder { at: at(*x, *z), half: 8.0, passable: None }).collect();
            w.refused.push((UnitDefId(21), at(2296.0, 2440.0)));
            w
        };
        let plant = |def: i32| Facts { factory: true, product_half: 24.0, product_passable: Some(walk.clone()), footprint: Footprint { x: 12, z: 12 }, ..facts(def, 12, 700.0, 3000.0, None) };
        let kinds: Vec<(Facts, Purpose, Option<Vec3>, f32)> = vec![
            (facts(3, 10, 155.0, 340.0, Some((50.0, 89.0))), Purpose::Generator { cap: None }, None, 300.0),
            (facts(5, 4, 40.0, 200.0, None), Purpose::Generator { cap: Some(4) }, None, 300.0),
            (Facts { build_distance: 400.0, ..facts(20, 6, 230.0, 560.0, None) }, Purpose::Turret, Some(at(2100.0, 2400.0)), 500.0),
            (facts(21, 10, 4000.0, 5000.0, Some((300.0, 3000.0))), Purpose::Frame, Some(at(2100.0, 2400.0)), 600.0),
            (plant(30), Purpose::Plant, None, 600.0),
            (plant(31), Purpose::Plant, None, 600.0),
            (facts(22, 4, 90.0, 600.0, None), Purpose::Place, None, 200.0),
        ];
        let shared = make();
        let questions: Vec<Request> = shared
            .builders
            .iter()
            .flat_map(|b| kinds.iter().map(|(facts, purpose, centre, radius)| Request { facts: facts.clone(), purpose: *purpose, centre: centre.unwrap_or(b.at), radius: *radius, builder_at: b.at, builder: Facts { build_distance: 145.0, build_speed: 300.0, ..builder() } }))
            .collect();
        let (mut found, mut refused) = (0, 0);
        // Twice over the shared world: the second time every point is the memo's.
        for req in questions.iter().chain(questions.iter()) {
            let (mut shared_trace, mut alone_trace) = (Vec::new(), Vec::new());
            let together = choose_traced(&shared, req, Some(&mut shared_trace));
            let alone = choose_traced(&make(), req, Some(&mut alone_trace));
            match (&together, &alone) {
                (Ok(a), Ok(b)) => {
                    assert!(a.at.x == b.at.x && a.at.z == b.at.z && a.facing == b.facing && a.words == b.words, "{:?} for a builder at {:?}: {a:?} against {b:?}", req.purpose, req.builder_at);
                    found += 1;
                }
                (Err(a), Err(b)) => {
                    assert_eq!(a, b);
                    refused += 1;
                }
                _ => panic!("{:?} for a builder at {:?}: {together:?} against {alone:?}", req.purpose, req.builder_at),
            }
            // The traces but for their times.
            let timeless = |t: &[String]| t.iter().map(|l| l.rsplit_once(", ").filter(|(_, ms)| ms.ends_with(" ms")).map_or(l.clone(), |(head, _)| head.to_string())).collect::<Vec<_>>();
            assert_eq!(timeless(&shared_trace), timeless(&alone_trace));
        }
        assert!(found >= 40, "{found} found, {refused} refused");
    }

    /// The products' ground under an area by the walk set's sums is `cells_open` over the set: on the map and
    /// hanging off each edge, on the grid and half a cell off it, empty areas, areas over the closed strip.
    #[test]
    fn the_products_ground_by_sums_is_the_ground_cell_by_cell() {
        let t = flat();
        let open = |_: Vec3| true;
        let w = world(&t, &open);
        let walk: crate::Passable = (0..256 * 256).map(|k| { let (i, j) = (k % 256, k / 256); !(i % 23 == 7 && j % 5 != 0) && !(j == 190 && i > 40) }).collect::<Vec<bool>>().into();
        let mut seed = 7u32;
        let mut next = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let (mut stood, mut failed) = (0, 0);
        for k in 0..20_000 {
            let step = [16.0, 8.0, 4.0, 0.5][k % 4];
            let snap = |v: f32| (v / step).round() * step;
            let (x0, z0) = (snap(next() * 4300.0 - 100.0), snap(next() * 4300.0 - 100.0));
            let area = Rect { x0, z0, x1: x0 + snap(next() * 200.0), z1: z0 + snap(next() * 200.0) };
            let cell_by_cell = cells_open(&area, &|p: Vec3| w.ground.walks(&walk, p.x, p.z));
            assert_eq!(w.products_stand(Some(&walk), &area), cell_by_cell, "{area:?}");
            if cell_by_cell { stood += 1 } else { failed += 1 }
        }
        assert!(stood > 2_000 && failed > 2_000, "{stood} stood, {failed} did not");
    }

    /// The walk to within a reach of a point read by the disc's rows is the look over every cell: on lattices of
    /// either parity, at the window's edges and beyond them, for several reaches.
    #[test]
    fn the_walk_within_reach_by_rows_is_the_look_over_every_cell() {
        let t = flat();
        let open = |p: Vec3| (p.x / 16.0) as i32 % 9 != 4 || (p.z / 16.0) as i32 % 6 == 0;
        let mut w = world(&t, &open);
        for k in 0..40 {
            w.standing.push(standing(3, 10, Footprint { x: 10, z: 10 }.snap(at(1700.0 + (k % 8) as f32 * 90.0, 1700.0 + (k / 8) as f32 * 130.0)), 100.0, 100.0, None));
        }
        let walk = Walk::new(&w, at(1650.0, 1650.0), at(2000.0, 2000.0), 500.0);
        let mut asked = 0;
        for reach in [40.0f32, 145.0, 168.0, 209.0, 400.0] {
            for fp in [Footprint { x: 4, z: 4 }, Footprint { x: 3, z: 3 }, Footprint { x: 6, z: 3 }] {
                let points = lattice(fp, at(2000.0, 2000.0), 700.0);
                let disc = walk.disc(points[0], reach).expect("a lattice point is on the half elmos");
                for p in &points {
                    assert_eq!(walk.to_within_disc(&disc, *p), Some(walk.to_within(*p, reach)), "{p:?} within {reach}");
                    asked += 1;
                }
            }
        }
        assert!(asked > 50_000);
        // A point that lies otherwise in its cell is not the disc's to answer, nor one off the half elmos.
        let disc = walk.disc(at(2000.0, 2000.0), 145.0).unwrap();
        assert_eq!(walk.to_within_disc(&disc, at(2008.0, 2000.0)), None);
        assert_eq!(walk.to_within_disc(&disc, at(2000.3, 2000.0)), None);
        assert!(walk.disc(at(2000.3, 2000.0), 145.0).is_none());
    }

    fn flat() -> Terrain {
        let (w, h) = (256u32, 256u32);
        Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn facts(def: i32, fp: i32, metal: f32, health: f32, blast: Option<(f32, f32)>) -> Facts {
        Facts { def: UnitDefId(def), footprint: Footprint { x: fp, z: fp }, build_distance: 0.0, build_speed: 0.0, build_time: 1000.0, metal, health, blast, height_dif: 10.7, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false }
    }

    fn builder() -> Facts {
        Facts { build_distance: 145.0, build_speed: 300.0, ..facts(1, 4, 0.0, 0.0, None) }
    }

    fn standing(def: i32, fp: i32, p: Vec3, metal: f32, health: f32, blast: Option<(f32, f32)>) -> Standing {
        Standing { rect: Footprint { x: fp, z: fp }.rect(p), def: Some(UnitDefId(def)), kind: format!("u{def}"), ours: true, bank: true, metal, health, blast, name: format!("u{def}") }
    }

    fn world<'a>(t: &'a Terrain, open: &'a dyn Fn(Vec3) -> bool) -> World<'a> {
        World { ground: Ground::new(t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), lattice: None, refused: Vec::new(), home: at(2048.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open), walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(), farm_frame: None, farm: Default::default(), fixed: Default::default(), walk_sums: Default::default() }
    }

    /// A solar goes touching the solars that stand (a bank grows), never in a gap under a street's width; a
    /// converter's bank stops at its cap and the next founds a bank a street away.
    /// A plant's footprint is its products' ground too: with the ground under the asked centre impassable to them
    /// (a knoll a lab may stand on and a tank cannot climb), the site moves off the knoll, apron and footprint both
    /// clear of it (econ-flagship-20: two advanced constructors penned inside their plant from birth).
    #[test]
    fn a_plant_never_stands_on_ground_its_products_cannot_leave() {
        let t = flat();
        // Cell-aligned (GRID 16), so the cells' centres and the rect agree on where the knoll ends.
        let knoll = Rect { x0: 1984.0, z0: 1952.0, x1: 2096.0, z1: 2048.0 };
        let open = move |p: Vec3| !(p.x >= knoll.x0 && p.x < knoll.x1 && p.z >= knoll.z0 && p.z < knoll.z1);
        let w = world(&t, &open);
        let plant = Facts { factory: true, product_half: 20.0, ..facts(9, 18, 2600.0, 4750.0, None) };
        let req = Request { facts: plant.clone(), purpose: Purpose::Plant, centre: at(2040.0, 2000.0), radius: 300.0, builder_at: at(2040.0, 2000.0), builder: builder() };
        let site = choose(&w, &req).expect("a site off the knoll");
        let rect = plant.footprint.faced(site.facing).rect(site.at);
        assert!(!rect.overlaps(&knoll), "the plant stands over the knoll: {site:?}");
        // Without the knoll the plant stands at the asked place.
        let flat_open = |_: Vec3| true;
        let w2 = world(&t, &flat_open);
        let site2 = choose(&w2, &req).expect("a site");
        assert!(site2.at.dist2d(at(2040.0, 2000.0)) < 48.0, "{site2:?}");
    }

    #[test]
    fn generators_bank_touching_their_kind_and_converters_stop_at_the_cap() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        w.standing.push(standing(3, 10, at(2008.0, 2008.0), 155.0, 340.0, Some((50.0, 89.0))));
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: at(2100.0, 2100.0), radius: 300.0, builder_at: at(2100.0, 2100.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(Footprint { x: 10, z: 10 }.rect(site.at).touches(&w.standing[0].rect), "{site:?}");
        assert!(site.words.contains("touching 1"));
        // A converter (cap 6) beside a bank of six founds a new bank at least a street off.
        let conv = facts(4, 6, 1.0, 167.0, Some((64.0, 200.0)));
        for k in 0..6 {
            w.standing.push(standing(4, 6, at(2408.0 + 48.0 * k as f32, 2408.0), 1.0, 167.0, Some((64.0, 200.0))));
        }
        let req = Request { facts: conv.clone(), purpose: Purpose::Generator { cap: Some(6) }, centre: at(2520.0, 2400.0), radius: 200.0, builder_at: at(2520.0, 2400.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let rect = Footprint { x: 6, z: 6 }.rect(site.at);
        assert!(w.standing[1..].iter().all(|s| !s.rect.touches(&rect) && gap(&s.rect, &rect) >= BANK_STREET), "{site:?}");
        assert!(site.words.contains("founding"));
    }

    /// The walk is the way round, not the line: with a wall of solars between the builder and a site touching two of
    /// them, the solar goes touching one on the builder's side (block-2's commander rounded a wall 700 long).
    #[test]
    fn the_walk_goes_round_what_stands_so_the_near_side_of_a_wall_wins() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        // A wall along x at z 2000, 11 solars from 1600 to 2400, with a notch at its middle on the far (north) side
        // where a solar would touch two.
        for k in 0..11 {
            w.standing.push(standing(3, 10, at(1608.0 + 80.0 * k as f32, 2008.0), 155.0, 340.0, None));
        }
        w.standing.push(standing(3, 10, at(1928.0, 1928.0), 155.0, 340.0, None));
        w.standing.push(standing(3, 10, at(2088.0, 1928.0), 155.0, 340.0, None));
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        // The builder south of the wall, under the notch.
        let builder_at = at(2008.0, 2150.0);
        // No cap here: the walk alone decides between touching two across the wall and touching one on this side.
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: builder_at, radius: 300.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.at.z > 2048.0, "across the wall: {site:?}");
        assert!(site.words.contains("touching 1"), "{site:?}");
        // The cap: with the wall's bank of thirteen over a cap of 8, nothing touching it is a candidate.
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: Some(8) }, centre: builder_at, radius: 300.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.words.contains("founding"), "{site:?}");
        let rect = Footprint { x: 10, z: 10 }.rect(site.at);
        assert!(w.standing.iter().all(|s| !s.rect.touches(&rect) && gap(&s.rect, &rect) >= BANK_STREET), "{site:?}");
    }

    /// A turret goes where it reaches the most pads; a plant goes where the most turrets reach it and keeps its
    /// street; a frame goes where the build power is.
    #[test]
    fn turrets_reach_pads_plants_sit_by_turrets_and_frames_by_power() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let plant = |p: Vec3| Plant { centre: p, footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant".into(), widest: "Bull".into(), passable: None };
        w.plants.push(plant(at(2000.0, 2000.0)));
        w.plants.push(plant(at(2000.0, 2300.0)));
        for p in [at(2000.0, 2000.0), at(2000.0, 2300.0)] {
            w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(p), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant".into() });
        }
        let nano = Facts { build_distance: 240.0, build_speed: 200.0, ..facts(5, 6, 300.0, 560.0, Some((100.0, 530.0))) };
        let req = Request { facts: nano.clone(), purpose: Purpose::Turret, centre: at(2000.0, 2150.0), radius: 400.0, builder_at: at(2000.0, 2150.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.words.starts_with("2 pads"), "{site:?}");
        // Its site is not in either plant's apron (east of them).
        assert!(w.plants.iter().all(|p| !p.apron().overlaps(&Footprint { x: 6, z: 6 }.rect(site.at))));
        // A second plant beside the first, within the turret's reach, facing east like it.
        w.workers.push(Worker { at: site.at, reach: 240.0, power: 200.0 });
        let vp = Facts { factory: true, product_half: 32.0, ..facts(9, 12, 700.0, 3000.0, None) };
        let req = Request { facts: vp.clone(), purpose: Purpose::Plant, centre: at(2000.0, 2150.0), radius: 500.0, builder_at: at(2000.0, 2150.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.words.starts_with("1 turret"), "{site:?}");
        assert_eq!(site.facing, 1, "{site:?}");
        // A fusion goes within the worker's reach: fewer seconds to stand.
        let fus = Facts { build_time: 100_000.0, ..facts(7, 8, 4300.0, 3800.0, Some((300.0, 3000.0))) };
        let req = Request { facts: fus.clone(), purpose: Purpose::Frame, centre: at(2000.0, 2150.0), radius: 600.0, builder_at: at(2600.0, 2150.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(distance_to(&Footprint { x: 8, z: 8 }.rect(site.at), w.workers[0].at) <= 240.0, "{site:?}");
    }

    /// A plant's apron must lie on the map, off every spot and on open ground: by the east edge with the enemy east it
    /// faces another way; with a spot before its east face likewise.
    #[test]
    fn a_plants_apron_keeps_off_the_map_edge_and_the_spots() {
        let t = flat();
        let open = |_: Vec3| true;
        let vp = Facts { factory: true, product_half: 32.0, ..facts(9, 12, 700.0, 3000.0, None) };
        let builder_at = at(4040.0, 2048.0);
        let w = world(&t, &open);
        let req = Request { facts: vp.clone(), purpose: Purpose::Plant, centre: builder_at, radius: 60.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert_ne!(site.facing, 1, "not east, off the map: {site:?}");
        let spots = [at(2300.0, 2048.0)];
        let mut w = world(&t, &open);
        w.spots = &spots;
        let builder_at = at(2100.0, 2048.0);
        let req = Request { facts: vp.clone(), purpose: Purpose::Plant, centre: builder_at, radius: 60.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let own = Plant { centre: site.at, footprint: vp.footprint, facing: site.facing, product_half: 32.0, ours: true, name: String::new(), widest: String::new(), passable: None };
        assert!(!own.apron().overlaps(&Rect { x0: 2268.0, z0: 2016.0, x1: 2332.0, z1: 2080.0 }), "the apron over the spot: {site:?}");
        assert!(site.at.dist2d(spots[0]) >= 100.0);
    }

    /// The street rule: with a plant facing east and a wall of solars east of it leaving one gap, a solar is never
    /// placed in the gap (it would close the street), though the gap is the nearest free ground to the builder.
    #[test]
    fn a_site_that_would_close_a_street_is_passed_over() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        w.plants.push(Plant { centre: at(2000.0, 2000.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into(), passable: None });
        w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(at(2000.0, 2000.0)), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant_1".into() });
        for k in -12..=12 {
            if k == 0 {
                continue;
            }
            w.standing.push(standing(3, 10, at(2200.0, 2000.0 + 80.0 * k as f32), 155.0, 340.0, None));
        }
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: at(2200.0, 2000.0), radius: 120.0, builder_at: at(2200.0, 2000.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let rect = Footprint { x: 10, z: 10 }.rect(site.at);
        assert!(!rect.overlaps(&Rect { x0: 2160.0, z0: 1960.0, x1: 2240.0, z1: 2040.0 }), "placed in the gap: {site:?}");
        assert!(!w.plants[0].apron().overlaps(&rect));
    }

    /// A street closed already is nobody's to close: with the plant ringed by solars (closed before any candidate),
    /// a solar asked for 700 away, inside the plant's window, gets its site; until 2026-11-04 every site in the
    /// window was refused as closing the street and the engine's search took over (1793769060-human).
    #[test]
    fn an_already_closed_street_does_not_veto_a_site_in_its_window() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        w.plants.push(Plant { centre: at(2000.0, 2000.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into(), passable: None });
        w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(at(2000.0, 2000.0)), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant_1".into() });
        for k in -3..=3 {
            let o = 80.0 * k as f32;
            for (x, z) in [(2240.0, 2000.0 + o), (1760.0, 2000.0 + o), (2000.0 + o, 1760.0), (2000.0 + o, 2240.0)] {
                w.standing.push(standing(3, 10, at(x, z), 155.0, 340.0, None));
            }
        }
        assert!(!street_open(&w.ground, &w.plants[0], &w.standing.iter().map(|s| s.rect).collect::<Vec<_>>(), &*w.open), "the ring closes the street");
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: at(2700.0, 2000.0), radius: 120.0, builder_at: at(2700.0, 2000.0), builder: builder() };
        let site = choose(&w, &req).expect("a site, the street being closed already");
        assert!(site.at.dist2d(at(2700.0, 2000.0)) < 200.0, "{site:?}");
    }

    /// A working builder keeps its way out: in a bay of turbines open to the west, the turbine that would shut the
    /// bay is passed over for one that leaves it open (t16's advanced constructor, 1793769060-human 4:55).
    #[test]
    fn a_site_that_would_wall_in_a_working_builder_is_passed_over() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let builder_at = at(2048.0, 2048.0);
        w.builders.push(Builder { at: builder_at, half: 24.0, passable: None });
        for k in -2..=2 {
            let o = 48.0 * k as f32;
            for (x, z) in [(2048.0 + o, 1960.0), (2048.0 + o, 2136.0), (2136.0, 2048.0 + o)] {
                w.standing.push(standing(5, 6, at(x, z), 40.0, 200.0, None));
            }
        }
        let wind = facts(5, 6, 40.0, 200.0, None);
        // Asked for at the bay's mouth: touching the bank there would shut it.
        let req = Request { facts: wind.clone(), purpose: Purpose::Generator { cap: None }, centre: at(1960.0, 2048.0), radius: 100.0, builder_at: at(1900.0, 2048.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let mut with: Vec<Rect> = w.standing.iter().map(|s| s.rect).collect();
        with.push(Footprint { x: 6, z: 6 }.rect(site.at));
        assert!(way_out(&w.ground, &w.builders[0], &with, &*w.open), "walled in by the site: {site:?}");
    }

    /// A builder alone on open ground asking for a solar in its own band gets one beside itself, in the first
    /// widening at the first try: the sites under its body are no candidates (site-dump-1, 2026-11-05: they were the
    /// twelve tries of every widening, each refused as walling the builder in, so every search anchored at a builder
    /// ran all three widenings and ended at the engine).
    #[test]
    fn no_site_goes_over_the_asking_builder() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let builder_at = at(2054.0, 2061.0);
        w.builders.push(Builder { at: builder_at, half: 24.0, passable: None });
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: builder_at, radius: 300.0, builder_at, builder: builder() };
        let mut trace = Vec::new();
        let site = choose_traced(&w, &req, Some(&mut trace)).expect("a site");
        assert!(!Footprint { x: 10, z: 10 }.rect(site.at).overlaps(&w.builders[0].body()), "over the builder: {site:?}");
        assert!(site.at.dist2d(builder_at) < 120.0, "beside it: {site:?}");
        assert_eq!(trace.len(), 2, "one widening, one try: {trace:?}");
        assert!(trace[1].contains("try 0") && trace[1].contains("chosen"), "{trace:?}");
    }

    /// A widening tries new sites: a solar asked for in a plant's street (the band so narrow that every candidate
    /// of the first widening closes it) is found farther out, and no position is tried twice. Until 2026-11-07 the
    /// widenings re-tried the first widening's twelve (the twelve best of the wider band were the twelve nearest),
    /// and 31 of 34 dumped searches ran all three over one set of sites.
    #[test]
    fn a_widening_tries_sites_the_last_one_did_not() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        w.plants.push(Plant { centre: at(2000.0, 2000.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into(), passable: None });
        w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(at(2000.0, 2000.0)), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant_1".into() });
        for k in -12..=12 {
            if k == 0 {
                continue;
            }
            w.standing.push(standing(3, 10, at(2200.0, 2000.0 + 80.0 * k as f32), 155.0, 340.0, None));
        }
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: at(2200.0, 2000.0), radius: 40.0, builder_at: at(2100.0, 2000.0), builder: builder() };
        let mut trace = Vec::new();
        let site = choose_traced(&w, &req, Some(&mut trace)).expect("a site");
        assert!(!Footprint { x: 10, z: 10 }.rect(site.at).overlaps(&Rect { x0: 2160.0, z0: 1960.0, x1: 2240.0, z1: 2040.0 }), "placed in the gap: {site:?}");
        let widenings = trace.iter().filter(|l| l.starts_with("widening")).count();
        assert!(widenings >= 2, "found in the first widening, the test proves nothing: {trace:?}");
        let tried: Vec<&str> = trace.iter().filter(|l| l.trim_start().starts_with("try ")).map(|l| l.split(" scores").next().unwrap().split(": ").nth(1).unwrap()).collect();
        let mut seen = std::collections::HashSet::new();
        for t in &tried {
            assert!(seen.insert(*t), "tried twice: {t} in {trace:?}");
        }
    }

    /// The asker keeps its own way out when the site is within its reach, since it builds from where it stands:
    /// inside a bay of turbines whose mouth is 56 wide, asking for a turbine at the mouth, whose nearest sites are
    /// 32 from it, with a reach of 145, every site in the mouth is refused for walling it in and the site found
    /// leaves it a way out (1794106612-human on Full Metal Plate: three commanders penned by their own turbines and
    /// extractors, built within reach from their stands). With a reach of 20 the same asker walks to the site and
    /// stands beside it, so it is not asked: the site is in the mouth (the eight walled-in dumps of 2026-11-07, six
    /// naming the asker 46 to 95 from the site's edge, had the asker left out altogether).
    #[test]
    fn the_asker_keeps_its_way_out_within_reach_and_walks_beyond_it() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let asker = at(2048.0, 2012.0);
        w.builders.push(Builder { at: asker, half: 24.0, passable: None });
        // Rows at z 1960 and 2064 (edges 1984 and 2040: a mouth of 56), the east end closed.
        for k in -2..=2 {
            let o = 48.0 * k as f32;
            w.standing.push(standing(5, 6, at(2048.0 + o, 1960.0), 40.0, 200.0, None));
            w.standing.push(standing(5, 6, at(2048.0 + o, 2064.0), 40.0, 200.0, None));
        }
        w.standing.push(standing(5, 6, at(2136.0, 2012.0), 40.0, 200.0, None));
        // Another kind than the bay's turbines, so the bank gap rule leaves the mouth's sites as candidates.
        let wind = facts(7, 6, 40.0, 200.0, None);
        let mouth = at(1952.0, 2012.0);
        let req = Request { facts: wind.clone(), purpose: Purpose::Generator { cap: None }, centre: mouth, radius: 100.0, builder_at: asker, builder: builder() };
        let mut trace = Vec::new();
        let site = choose_traced(&w, &req, Some(&mut trace)).expect("a site");
        assert!(trace.iter().any(|l| l.contains("walls in")), "{trace:?}");
        let mut with: Vec<Rect> = w.standing.iter().map(|s| s.rect).collect();
        with.push(Footprint { x: 6, z: 6 }.rect(site.at));
        assert!(way_out(&w.ground, &w.builders[0], &with, &*w.open), "walled in by its own site: {site:?} {trace:#?}");
        let walker = Request { builder: Facts { build_distance: 20.0, ..builder() }, ..req };
        let mut trace = Vec::new();
        let site = choose_traced(&w, &walker, Some(&mut trace)).expect("a site");
        assert!(trace.iter().all(|l| !l.contains("walls in")), "{trace:?}");
        with.pop();
        with.push(Footprint { x: 6, z: 6 }.rect(site.at));
        assert!(!way_out(&w.ground, &w.builders[0], &with, &*w.open), "the walker's site is in the mouth: {site:?} {trace:#?}");
    }

    /// A construction turret with nothing to assist is refused in its own words before any search: a plant 3,500
    /// away is out of reach of the widest band. With no plant or frame anywhere a turret is placed like anything
    /// else (the first one). Three dumps of 2026-11-07 were turrets asked for at hold points, refused as "no
    /// ground takes it" after three widenings over nothing.
    #[test]
    fn a_turret_with_nothing_to_assist_stands_at_the_asked_place() {
        // A nano before the fusions: nothing of ours within reach of the asked place, a plant far off. The turret is
        // sited nearest the place, not refused (2026-11-07: three dumped searches of hold points said "no ground").
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        w.plants.push(Plant { centre: at(500.0, 500.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant".into(), widest: "Bull".into(), passable: None });
        let nano = Facts { build_distance: 240.0, build_speed: 200.0, ..facts(5, 6, 300.0, 560.0, Some((100.0, 530.0))) };
        let req = Request { facts: nano.clone(), purpose: Purpose::Turret, centre: at(3500.0, 3500.0), radius: 200.0, builder_at: at(3300.0, 3500.0), builder: builder() };
        let mut trace = Vec::new();
        let site = choose_traced(&w, &req, Some(&mut trace)).expect("a site at the asked place");
        assert!(site.at.dist2d(req.centre) <= 48.0, "{site:?} {trace:#?}");
        assert!(site.words.contains("nothing in reach yet"), "{site:?}");
    }
}

#[cfg(test)]
mod sea_tests {

    #[test]
    fn a_sea_kind_on_a_map_without_water_is_refused_without_a_search() {
        // Land everywhere at 50: a shipyard wanting 30 of water stands nowhere, and the answer says so at once.
        let t = Terrain { cell: 16.0, width: 256, height: 256, heights: vec![50; 256 * 256], slopes: vec![0; 256 * 256], metal: Vec::new() };
        let open = |_: Vec3| true;
        let w = World { ground: Ground::new(&t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), lattice: None, refused: Vec::new(), home: at(2048.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open), walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(), farm_frame: None, farm: Default::default(), fixed: Default::default(), walk_sums: Default::default() };
        let mut shipyard = Facts { def: UnitDefId(9), footprint: Footprint { x: 12, z: 12 }, build_distance: 0.0, build_speed: 0.0, build_time: 5000.0, metal: 450.0, health: 2000.0, blast: None, height_dif: 10.7, min_depth: 30.0, max_depth: 1e7, floater: true, extractor: false, factory: true, building: true, product_half: 24.0, product_passable: Some(crate::sea_walk(&t, 15.0)) };
        let builder = Facts { def: UnitDefId(1), footprint: Footprint { x: 4, z: 4 }, build_distance: 145.0, build_speed: 300.0, build_time: 1.0, metal: 0.0, health: 0.0, blast: None, height_dif: 10.7, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false, extractor: false, factory: false, building: false, product_half: 0.0, product_passable: None };
        assert!(!w.ground.depths_on_map(&shipyard));
        let req = Request { facts: shipyard.clone(), purpose: Purpose::Plant, centre: at(2048.0, 2048.0), radius: 600.0, builder_at: at(2048.0, 2048.0), builder: builder.clone() };
        let why = choose(&w, &req).unwrap_err();
        assert!(why.contains("no ground on this map takes it") && why.contains("water at least 30 deep") && why.contains("50 to 50"), "{why}");
        assert!(w.walks.borrow().is_empty(), "no flood was made");
        // A land kind's search makes one flood and a second kind from the same place with the same band reuses it.
        shipyard.min_depth = crate::NO_DEPTH_FLOOR;
        shipyard.max_depth = 0.0;
        shipyard.product_passable = None;
        assert!(w.ground.depths_on_map(&shipyard));
        let req = Request { facts: shipyard.clone(), purpose: Purpose::Plant, centre: at(2048.0, 2048.0), radius: 600.0, builder_at: at(2048.0, 2048.0), builder: builder.clone() };
        assert!(choose(&w, &req).is_ok());
        assert_eq!(w.walks.borrow().len(), 1);
        let solar = Facts { def: UnitDefId(3), footprint: Footprint { x: 4, z: 4 }, build_distance: 0.0, build_speed: 0.0, build_time: 5000.0, metal: 450.0, health: 2000.0, blast: None, height_dif: 10.7, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: true, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None};
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: at(2048.0, 2048.0), radius: 600.0, builder_at: at(2048.0, 2048.0), builder };
        assert!(choose(&w, &req).is_ok());
        assert_eq!(w.walks.borrow().len(), 1, "the same start, centre and radius: the flood is shared");
    }
    use super::*;
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// Land at 50 west of x 2048, water of -40 east of it.
    fn shore() -> Terrain {
        let (w, h) = (256u32, 256u32);
        let mut heights = Vec::new();
        for _ in 0..h {
            for i in 0..w {
                heights.push(if (i as f32 * 16.0 + 8.0) < 2048.0 { 50 } else { -40 });
            }
        }
        Terrain { cell: 16.0, width: w, height: h, heights, slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    /// A tidal generator for a commander on the shore goes on the water nearest the commander's walk, though the
    /// commander cannot stand on any water site and home lies far out to sea.
    #[test]
    fn a_sea_kind_goes_on_the_water_nearest_the_builders_walk() {
        let t = shore();
        let ground = Ground::new(&t, 4096.0, 4096.0);
        let open = |p: Vec3| ground.height_at(p.x, p.z).is_some_and(|h| h >= 0.0);
        let w = World { ground: Ground::new(&t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), lattice: None, refused: Vec::new(), home: at(3500.0, 3500.0), toward: at(4000.0, 2048.0), open: Box::new(open), walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(), farm_frame: None, farm: Default::default(), fixed: Default::default(), walk_sums: Default::default() };
        let tidal = Facts { def: UnitDefId(7), footprint: Footprint { x: 3, z: 3 }, build_distance: 0.0, build_speed: 0.0, build_time: 1000.0, metal: 100.0, health: 300.0, blast: None, height_dif: 10.7, min_depth: 20.0, max_depth: 1e7, floater: true, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None };
        let commander = Facts { def: UnitDefId(1), footprint: Footprint { x: 4, z: 4 }, build_distance: 145.0, build_speed: 300.0, build_time: 1.0, metal: 0.0, health: 0.0, blast: None, height_dif: 10.7, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false, extractor: false, factory: false, building: false, product_half: 0.0, product_passable: None };
        let builder_at = at(1900.0, 2048.0);
        let req = Request { facts: tidal.clone(), purpose: Purpose::Generator { cap: None }, centre: builder_at, radius: 400.0, builder_at, builder: commander };
        let site = choose(&w, &req).expect("a site on the water");
        assert!(ground.height_at(site.at.x, site.at.z).is_some_and(|h| h < 0.0), "{site:?} is not on the water");
        assert!(site.at.x < 2150.0 && site.at.dist2d(builder_at) < 300.0, "{site:?} is not the water nearest the commander (the walk band of 48 lets the shore slide)");
    }

    /// A shipyard for a constructor on the shore goes on the water: its apron and street are judged by the water its
    /// ships float in, not by the land's passable set (water-1: the land's apron check refused every shipyard site).
    #[test]
    fn a_shipyard_goes_on_the_water_with_its_apron_and_street_at_sea() {
        let t = shore();
        let ground = Ground::new(&t, 4096.0, 4096.0);
        let open = |p: Vec3| ground.height_at(p.x, p.z).is_some_and(|h| h >= 0.0);
        let w = World { ground: Ground::new(&t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), lattice: None, refused: Vec::new(), home: at(1800.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open), walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default(), farm_frame: None, farm: Default::default(), fixed: Default::default(), walk_sums: Default::default() };
        let shipyard = Facts { def: UnitDefId(9), footprint: Footprint { x: 12, z: 12 }, build_distance: 0.0, build_speed: 0.0, build_time: 5000.0, metal: 450.0, health: 2000.0, blast: None, height_dif: 10.7, min_depth: 30.0, max_depth: 1e7, floater: true, extractor: false, factory: true, building: true, product_half: 24.0, product_passable: Some(crate::sea_walk(&t, 15.0)) };
        let commander = Facts { def: UnitDefId(1), footprint: Footprint { x: 4, z: 4 }, build_distance: 145.0, build_speed: 300.0, build_time: 1.0, metal: 0.0, health: 0.0, blast: None, height_dif: 10.7, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false, extractor: false, factory: false, building: false, product_half: 0.0, product_passable: None };
        let builder_at = at(1900.0, 2048.0);
        let req = Request { facts: shipyard.clone(), purpose: Purpose::Plant, centre: builder_at, radius: 600.0, builder_at, builder: commander };
        let site = choose(&w, &req).expect("a shipyard site on the water");
        assert!(ground.height_at(site.at.x, site.at.z).is_some_and(|h| h <= -30.0), "{site:?} is not in water 30 deep");
        assert!(site.at.x < 2400.0, "{site:?} is far out to sea");
    }
}
