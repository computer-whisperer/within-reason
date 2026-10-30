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

use crate::block::Lattice;
use crate::street::{Plant, street_open};
use crate::{Facts, Footprint, GRID, Ground, Rect, facings_toward};

/// The margin round an ally's footprint: one build square, so nothing of ours touches theirs.
const ALLY_MARGIN: f32 = 8.0;
/// The margin round a metal spot's centre no building covers (the extractor's footprint and a square).
const SPOT_BOX_HALF: f32 = 32.0;
/// A refused site keeps candidates of the type this far off.
const REFUSED_RADIUS: f32 = 32.0;
/// Banks of one kind stand touching or a vehicle street apart: three build squares.
const BANK_STREET: f32 = 24.0;
/// How many touching neighbours of its kind a generator is credited at most.
const TOUCHING_CAP: f32 = 2.0;
/// How many candidates, best first, the street check tries before the band is widened.
const STREET_TRIES: usize = 12;
/// The band of distance is widened this many times by doubling when nothing in it passes.
const WIDENINGS: u32 = 2;
/// The bands: a candidate within the band of the best at a level goes to the next level.
const BAND_GENERATOR: f32 = 0.5;
const BAND_TURRET: f32 = 0.5;
const BAND_FRAME_SECONDS: f32 = 3.0;
const BAND_CHAIN_METAL: f32 = 100.0;
const BAND_WALK: f32 = 48.0;

/// A footprint standing on the ground: ours or an ally's, finished or ordered.
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub struct Worker {
    pub at: Vec3,
    pub reach: f32,
    pub power: f32,
}

/// An unfinished building a turret could help.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub rect: Rect,
    pub metal: f32,
}

/// Ground kept for something not yet ordered, and for what.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reservation {
    pub rect: Rect,
    pub for_: Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reserved {
    /// Nothing goes here (the stand's slots, which their kinds take directly; a gantry's apron).
    Nothing,
    /// The block's frame rows: a frame may stand here.
    Frames,
    /// One kind's slot (the gantry's).
    Kind(UnitDefId),
}

impl Reservation {
    /// Whether a building of `facts` placed for `purpose` may stand on this ground.
    fn admits(&self, facts: &Facts, purpose: Purpose) -> bool {
        match self.for_ {
            Reserved::Nothing => false,
            Reserved::Frames => purpose == Purpose::Frame,
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
    pub workers: Vec<Worker>,
    pub frames: Vec<Frame>,
    /// Ground kept for something not yet ordered: the stand's free slots, the block's rows and gantry slot.
    pub reservations: Vec<Reservation>,
    /// The block's sub-lattice for construction turrets, once the block is laid.
    pub lattice: Option<Lattice>,
    /// Sites the engine refused lately, by type.
    pub refused: Vec<(UnitDefId, Vec3)>,
    pub home: Vec3,
    /// Where the enemy is: a plant's front points there when it can.
    pub toward: Vec3,
    /// Whether a plant's units can stand on the ground here.
    pub open: Box<dyn Fn(Vec3) -> bool + 'a>,
}

/// What a building is for: the score of level 1.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug)]
pub struct Site {
    pub at: Vec3,
    pub facing: i32,
    pub words: String,
}

struct Candidate {
    at: Vec3,
    facing: i32,
    rect: Rect,
    /// Level 1 (two parts), level 2, level 3, level 4; lower is better.
    scores: [f32; 5],
    words: String,
}

/// The site for `req` in `world`, or why there is none.
pub fn choose(world: &World, req: &Request) -> Result<Site, String> {
    let f = &req.facts;
    let facings: Vec<i32> = if f.factory { facings_toward(req.centre, world.toward).to_vec() } else { vec![0] };
    let bands = match req.purpose {
        Purpose::Generator { .. } => [BAND_GENERATOR, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Turret => [BAND_TURRET, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Plant => [BAND_TURRET, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Frame => [BAND_FRAME_SECONDS, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
        Purpose::Place => [GRID, 0.0, BAND_CHAIN_METAL, BAND_WALK, 0.0],
    };
    let banks = bank_sizes(world, f.def);
    let mut why = String::new();
    for widening in 0..=WIDENINGS {
        let radius = req.radius * 2f32.powi(widening as i32);
        let mut candidates: Vec<Candidate> = Vec::new();
        for (rank, facing) in facings.iter().enumerate() {
            let fp = f.footprint.faced(*facing);
            for at in lattice(fp, req.centre, radius) {
                if let Some(c) = candidate(world, req, &banks, fp, at, *facing, rank) {
                    candidates.push(c);
                }
            }
        }
        if candidates.is_empty() {
            why = format!("no ground within {radius:.0} of ({:.0}, {:.0}) takes a {} there", req.centre.x, req.centre.z, f.footprint.x * 8);
            continue;
        }
        let ordered = by_bands(candidates, &bands);
        let plants_own: Vec<Plant> = Vec::new();
        for c in ordered.iter().take(STREET_TRIES) {
            let mut blocks: Vec<Rect> = world.standing.iter().map(|s| if s.ours { s.rect } else { s.rect.grown(ALLY_MARGIN) }).collect();
            blocks.push(c.rect);
            let plants = world.plants.iter().chain(plants_own.iter());
            let mut closed = plants.filter(|p| p.concerns(&c.rect)).filter(|p| !street_open(&world.ground, p, &blocks, &*world.open));
            if let Some(p) = closed.next() {
                why = format!("the best site at ({:.0}, {:.0}) would close {}'s street for its {}", c.at.x, c.at.z, p.name, p.widest);
                continue;
            }
            // A plant's own street, from the site it would stand at.
            if f.factory {
                let own = Plant { centre: c.at, footprint: f.footprint, facing: c.facing, product_half: f.product_half, ours: true, name: "it".into(), widest: String::new() };
                if !street_open(&world.ground, &own, &blocks, &*world.open) {
                    why = format!("the best site at ({:.0}, {:.0}) has no street to the open", c.at.x, c.at.z);
                    continue;
                }
            }
            return Ok(Site { at: c.at, facing: c.facing, words: c.words.clone() });
        }
    }
    Err(why)
}

/// One lattice point as a candidate, or None when a constraint fails.
fn candidate(world: &World, req: &Request, banks: &HashMap<usize, usize>, fp: Footprint, at: Vec3, facing: i32, facing_rank: usize) -> Option<Candidate> {
    let f = &req.facts;
    let rect = fp.rect(at);
    if !world.ground.stands(fp, at, f.height_dif, false) {
        return None;
    }
    if world.standing.iter().any(|s| (if s.ours { s.rect } else { s.rect.grown(ALLY_MARGIN) }).overlaps(&rect)) {
        return None;
    }
    if world.reservations.iter().any(|r| r.rect.overlaps(&rect) && !r.admits(f, req.purpose)) {
        return None;
    }
    if req.purpose == Purpose::Turret && world.lattice.is_some_and(|l| !l.holds(at)) {
        return None;
    }
    if world.spots.iter().any(|p| p.dist2d(at) < world.spot_keep_out || Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }.overlaps(&rect)) {
        return None;
    }
    if world.refused.iter().any(|(d, p)| *d == f.def && p.dist2d(at) < REFUSED_RADIUS) {
        return None;
    }
    if world.plants.iter().any(|p| p.apron().overlaps(&rect)) {
        return None;
    }
    if f.factory {
        let own = Plant { centre: at, footprint: f.footprint, facing, product_half: f.product_half, ours: true, name: String::new(), widest: String::new() };
        let apron = own.apron();
        if !world.ground.in_map(&apron) || world.standing.iter().any(|s| s.rect.overlaps(&apron)) || world.reservations.iter().any(|r| r.rect.overlaps(&apron) && !r.admits(f, req.purpose)) {
            return None;
        }
        if world.spots.iter().any(|p| Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }.overlaps(&apron)) {
            return None;
        }
        if !apron_open(&apron, &*world.open) {
            return None;
        }
    }
    // Level 1.
    let mut words = String::new();
    let l1 = match req.purpose {
        Purpose::Generator { cap } => {
            let same: Vec<(usize, &Standing)> = world.standing.iter().enumerate().filter(|(_, s)| s.ours && s.def == Some(f.def)).collect();
            let touching: Vec<usize> = same.iter().filter(|(_, s)| s.rect.touches(&rect)).map(|(i, _)| *i).collect();
            // Touching its kind, or a street apart from it: a gap under the street's width is neither.
            if same.iter().any(|(_, s)| !s.rect.touches(&rect) && gap(&s.rect, &rect) < BANK_STREET) {
                return None;
            }
            if let Some(cap) = cap {
                let bank = touching.iter().map(|i| banks.get(i).copied().unwrap_or(1)).max().unwrap_or(0);
                if bank + 1 > cap {
                    return None;
                }
            }
            let n = touching.len();
            words = match n {
                0 => "founding a bank".to_string(),
                _ => format!("touching {n} of its kind"),
            };
            [-(n as f32).min(TOUCHING_CAP), 0.0]
        }
        Purpose::Turret => {
            let reach = f.build_distance;
            let pads = world.plants.iter().filter(|p| distance_to(&p.rect(), at) <= reach).count();
            let frames = world.frames.iter().filter(|fr| distance_to(&fr.rect, at) <= reach).count();
            if pads + frames == 0 && !(world.plants.is_empty() && world.frames.is_empty()) {
                return None;
            }
            words = format!("{pads} pad{} and {frames} frame{} in reach", if pads == 1 { "" } else { "s" }, if frames == 1 { "" } else { "s" });
            [-((pads + frames) as f32), 0.0]
        }
        Purpose::Plant => {
            let turrets = world.workers.iter().filter(|w| distance_to(&rect, w.at) <= w.reach).count();
            words = format!("{turrets} turret{} in reach, facing {}", if turrets == 1 { "" } else { "s" }, ["south", "east", "north", "west"][facing.rem_euclid(4) as usize]);
            [-(turrets as f32), facing_rank as f32]
        }
        Purpose::Frame => {
            let in_reach: Vec<&Worker> = world.workers.iter().filter(|w| distance_to(&rect, w.at) <= w.reach).collect();
            let power: f32 = in_reach.iter().map(|w| w.power).sum::<f32>() + req.builder.build_speed;
            let seconds = if power > 0.0 { f.build_time / power } else { f32::INFINITY };
            words = format!("{} turret{} with {:.0} build power in reach, {seconds:.0} s to stand", in_reach.len(), if in_reach.len() == 1 { "" } else { "s" }, power);
            [seconds, 0.0]
        }
        Purpose::Place => [at.dist2d(req.centre), 0.0],
    };
    // Level 2: the chain sum.
    let own_blast = f.blast;
    let taken: f32 = world
        .standing
        .iter()
        .filter_map(|s| s.blast.map(|b| blast_at(b, s.rect.centre(), &rect)))
        .sum();
    let killed: Vec<&Standing> = match own_blast {
        Some(b) => world.standing.iter().filter(|s| s.ours && s.health > 0.0 && blast_at(b, at, &s.rect) >= s.health).collect(),
        None => Vec::new(),
    };
    let kills: f32 = killed.iter().map(|s| s.metal).sum();
    let l2 = kills + if f.health > 0.0 && taken >= f.health { f.metal } else { 0.0 };
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
        words = format!("{words}; its blast holds {}, {kills:.0} metal of ours", kinds.join(" and "));
    }
    let l3 = (req.builder_at.dist2d(at) - f.reach_from(&req.builder)).max(0.0);
    let l4 = world.home.dist2d(at);
    Some(Candidate { at, facing, rect, scores: [l1[0], l1[1], l2, l3, l4], words })
}

/// The candidates in the order of the bands: at each level those within the band of the best go on to the next
/// level's order; the rest follow them in this level's order, so the street check has them to fall back to.
fn by_bands(candidates: Vec<Candidate>, bands: &[f32; 5]) -> Vec<Candidate> {
    by_bands_from(candidates, bands, 0)
}

fn by_bands_from(candidates: Vec<Candidate>, bands: &[f32; 5], level: usize) -> Vec<Candidate> {
    if level >= 5 || candidates.len() <= 1 {
        return candidates;
    }
    let best = candidates.iter().map(|c| c.scores[level]).fold(f32::INFINITY, f32::min);
    let (within, mut rest): (Vec<Candidate>, Vec<Candidate>) = candidates.into_iter().partition(|c| c.scores[level] <= best + bands[level]);
    rest.sort_by(|a, b| a.scores[level].total_cmp(&b.scores[level]));
    let mut out = by_bands_from(within, bands, level + 1);
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

fn apron_open(apron: &Rect, open: &dyn Fn(Vec3) -> bool) -> bool {
    let mut z = apron.z0 + GRID / 2.0;
    while z < apron.z1 {
        let mut x = apron.x0 + GRID / 2.0;
        while x < apron.x1 {
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
fn distance_to(r: &Rect, p: Vec3) -> f32 {
    let dx = (r.x0 - p.x).max(0.0).max(p.x - r.x1);
    let dz = (r.z0 - p.z).max(0.0).max(p.z - r.z1);
    dx.hypot(dz)
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
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn flat() -> Terrain {
        let (w, h) = (256u32, 256u32);
        Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn facts(def: i32, fp: i32, metal: f32, health: f32, blast: Option<(f32, f32)>) -> Facts {
        Facts { def: UnitDefId(def), footprint: Footprint { x: fp, z: fp }, build_distance: 0.0, build_speed: 0.0, build_time: 1000.0, metal, health, blast, height_dif: 10.7, extractor: false, factory: false, building: true, product_half: 0.0 }
    }

    fn builder() -> Facts {
        Facts { build_distance: 145.0, build_speed: 300.0, ..facts(1, 4, 0.0, 0.0, None) }
    }

    fn standing(def: i32, fp: i32, p: Vec3, metal: f32, health: f32, blast: Option<(f32, f32)>) -> Standing {
        Standing { rect: Footprint { x: fp, z: fp }.rect(p), def: Some(UnitDefId(def)), kind: format!("u{def}"), ours: true, bank: true, metal, health, blast, name: format!("u{def}") }
    }

    fn world<'a>(t: &'a Terrain, open: &'a dyn Fn(Vec3) -> bool) -> World<'a> {
        World { ground: Ground::new(t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), workers: Vec::new(), frames: Vec::new(), reservations: Vec::new(), lattice: None, refused: Vec::new(), home: at(2048.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open) }
    }

    /// A solar goes touching the solars that stand (a bank grows), never in a gap under a street's width; a
    /// converter's bank stops at its cap and the next founds a bank a street away.
    #[test]
    fn generators_bank_touching_their_kind_and_converters_stop_at_the_cap() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        w.standing.push(standing(3, 10, at(2008.0, 2008.0), 155.0, 340.0, Some((50.0, 89.0))));
        let req = Request { facts: solar, purpose: Purpose::Generator { cap: None }, centre: at(2100.0, 2100.0), radius: 300.0, builder_at: at(2100.0, 2100.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(Footprint { x: 10, z: 10 }.rect(site.at).touches(&w.standing[0].rect), "{site:?}");
        assert!(site.words.contains("touching 1"));
        // A converter (cap 6) beside a bank of six founds a new bank at least a street off.
        let conv = facts(4, 6, 1.0, 167.0, Some((64.0, 200.0)));
        for k in 0..6 {
            w.standing.push(standing(4, 6, at(2408.0 + 48.0 * k as f32, 2408.0), 1.0, 167.0, Some((64.0, 200.0))));
        }
        let req = Request { facts: conv, purpose: Purpose::Generator { cap: Some(6) }, centre: at(2520.0, 2400.0), radius: 200.0, builder_at: at(2520.0, 2400.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let rect = Footprint { x: 6, z: 6 }.rect(site.at);
        assert!(w.standing[1..].iter().all(|s| !s.rect.touches(&rect) && gap(&s.rect, &rect) >= BANK_STREET), "{site:?}");
        assert!(site.words.contains("founding"));
    }

    /// A turret goes where it reaches the most pads; a plant goes where the most turrets reach it and keeps its
    /// street; a frame goes where the build power is.
    #[test]
    fn turrets_reach_pads_plants_sit_by_turrets_and_frames_by_power() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let plant = |p: Vec3| Plant { centre: p, footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant".into(), widest: "Bull".into() };
        w.plants.push(plant(at(2000.0, 2000.0)));
        w.plants.push(plant(at(2000.0, 2300.0)));
        for p in [at(2000.0, 2000.0), at(2000.0, 2300.0)] {
            w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(p), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant".into() });
        }
        let nano = Facts { build_distance: 240.0, build_speed: 200.0, ..facts(5, 6, 300.0, 560.0, Some((100.0, 530.0))) };
        let req = Request { facts: nano, purpose: Purpose::Turret, centre: at(2000.0, 2150.0), radius: 400.0, builder_at: at(2000.0, 2150.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.words.starts_with("2 pads"), "{site:?}");
        // Its site is not in either plant's apron (east of them).
        assert!(w.plants.iter().all(|p| !p.apron().overlaps(&Footprint { x: 6, z: 6 }.rect(site.at))));
        // A second plant beside the first, within the turret's reach, facing east like it.
        w.workers.push(Worker { at: site.at, reach: 240.0, power: 200.0 });
        let vp = Facts { factory: true, product_half: 32.0, ..facts(9, 12, 700.0, 3000.0, None) };
        let req = Request { facts: vp, purpose: Purpose::Plant, centre: at(2000.0, 2150.0), radius: 500.0, builder_at: at(2000.0, 2150.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert!(site.words.starts_with("1 turret"), "{site:?}");
        assert_eq!(site.facing, 1, "{site:?}");
        // A fusion goes within the worker's reach: fewer seconds to stand.
        let fus = Facts { build_time: 100_000.0, ..facts(7, 8, 4300.0, 3800.0, Some((300.0, 3000.0))) };
        let req = Request { facts: fus, purpose: Purpose::Frame, centre: at(2000.0, 2150.0), radius: 600.0, builder_at: at(2600.0, 2150.0), builder: builder() };
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
        let req = Request { facts: vp, purpose: Purpose::Plant, centre: builder_at, radius: 60.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        assert_ne!(site.facing, 1, "not east, off the map: {site:?}");
        let spots = [at(2300.0, 2048.0)];
        let mut w = world(&t, &open);
        w.spots = &spots;
        let builder_at = at(2100.0, 2048.0);
        let req = Request { facts: vp, purpose: Purpose::Plant, centre: builder_at, radius: 60.0, builder_at, builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let own = Plant { centre: site.at, footprint: vp.footprint, facing: site.facing, product_half: 32.0, ours: true, name: String::new(), widest: String::new() };
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
        w.plants.push(Plant { centre: at(2000.0, 2000.0), footprint: Footprint { x: 12, z: 12 }, facing: 1, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into() });
        w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(at(2000.0, 2000.0)), def: Some(UnitDefId(9)), kind: "plant".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant_1".into() });
        for k in -12..=12 {
            if k == 0 {
                continue;
            }
            w.standing.push(standing(3, 10, at(2200.0, 2000.0 + 80.0 * k as f32), 155.0, 340.0, None));
        }
        let solar = facts(3, 10, 155.0, 340.0, Some((50.0, 89.0)));
        let req = Request { facts: solar, purpose: Purpose::Generator { cap: None }, centre: at(2200.0, 2000.0), radius: 120.0, builder_at: at(2200.0, 2000.0), builder: builder() };
        let site = choose(&w, &req).expect("a site");
        let rect = Footprint { x: 10, z: 10 }.rect(site.at);
        assert!(!rect.overlaps(&Rect { x0: 2160.0, z0: 1960.0, x1: 2240.0, z1: 2040.0 }), "placed in the gap: {site:?}");
        assert!(!w.plants[0].apron().overlaps(&rect));
    }
}
