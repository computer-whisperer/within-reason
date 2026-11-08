//! The block (the design §1.1, step 3): the one global decision after the stand. When the first advanced plant is
//! placed the block's centre is found once, at the player's `block` mark or on the flattest ground beside the plant
//! that holds two frame rows and, when the roster has one, the gantry's slot with its apron; from then on every slot in
//! it is chosen by the levels like any other (`choose`), under two things the block adds to the world: its ground is
//! reserved (the frame rows for frames, the gantry's slot for the gantry), and construction turrets stand on its
//! sub-lattice, 96 apart while the block is sparse (no turret within another's blast, 68 in reach of a frame) and 48
//! once the player says `dense` (the people's end state). Banks standing on its ground are listed for
//! reclaim-for-ground, the way thebluegecko's seventeen solars went.
//!
//! The geometry, from the people's blocks (the study §4: bars of turrets with rows of frames between them, every
//! frame in reach of dozens of turrets): with the centre C and the rows running along the plant's front, a bar of
//! turrets through C and one 192 either side of it, and between the bars two rows 144 deep (an advanced fusion's 96
//! and a fus's 80 both fit, with the slack the grid's snap needs), three frames long. The gantry's slot (192 square)
//! stands beyond the rows' end past the outer turret column, its apron toward the enemy.

use bot_protocol::Vec3;

use crate::choose::{Reservation, Reserved, World};
use crate::street::{Flood, Plant};
use crate::{Facts, Footprint, GRID, Rect, facing_dir, facings_toward};

/// The sub-lattice's pitch while the block is sparse: a turret's blast (64) reaches no neighbour.
pub const SPARSE_PITCH: f32 = 96.0;
/// After the player's `dense`: turrets touching, the people's pitch.
pub const DENSE_PITCH: f32 = 48.0;
/// How far from the advanced plant the block's centre is looked for before the search widens (twice, doubling).
const BLOCK_SEARCH: f32 = 600.0;
/// Slack over a turret's build distance for a plant's own size (the engine adds the target's radius): an unmarked
/// block's centre keeps the turret's reach plus this from every plant, so the block's turrets do not see a plant,
/// whose product under way is always the nearest thing to help (econ-flagship-8 and -9: blocks 499 and 304 from the
/// advanced plant, their turrets on the plant while the fusion frames crawled). A marked block is the player's word.
const PLANT_CLEARANCE_SLACK: f32 = 100.0;
/// The candidate centres' step: three build squares (a street check runs on each).
const SEARCH_STEP: f32 = 48.0;
const WIDENINGS: u32 = 2;
/// Frames a row.
const ROW_SLOTS: i32 = 3;
/// A row's depth: a frame of 96 with a cell's slack either side, between two bars 192 apart.
const ROW_DEPTH: f32 = 144.0;
/// Where a row begins, off the centre bar: half a turret's footprint.
const ROW_FROM: f32 = 24.0;
/// A turret's footprint, for the rows' offsets and the gantry's.
const TURRET_HALF: f32 = 24.0;
/// The block's period across the rows: a row and the bar of turrets beyond it, so the bars stand at 0 and ±192.
const PERIOD: f32 = 2.0 * ROW_FROM + ROW_DEPTH;
/// How many tiers of rows the block grows to (`Block::grow`): the first either side of the centre bar, the second
/// outside the next bars, the third the first's rows carried along the axis away from the gantry, which beside a
/// plant is the plant's own ground, so the third tier holds once the plant is eaten (his way). A fourth would stand
/// past the turrets' reach of the centre (the frame band), so it is never laid; a second block is the player's
/// `block` mark.
pub const MAX_TIERS: u32 = 3;
/// Sites whose height spread is within this of the flattest are equally flat.
const FLAT_BAND: f32 = 4.0;
/// Nothing but an extractor stands within this of a spot's centre (the shim's keep-out).
const SPOT_BOX_HALF: f32 = 32.0;
/// The margin round an ally's footprint.
const ALLY_MARGIN: f32 = 8.0;

/// The turrets' sub-lattice: the points `centre + pitch * (i, j)`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Lattice {
    pub centre: Vec3,
    pub pitch: f32,
}

impl Lattice {
    pub fn holds(&self, at: Vec3) -> bool {
        let off = |v: f32| ((v / self.pitch).round() * self.pitch - v).abs() < 1.0;
        off(at.x - self.centre.x) && off(at.z - self.centre.z)
    }
}

/// What the block is laid for.
pub struct Wants {
    /// The construction turret: its reach is the block's.
    pub turret: Facts,
    /// The largest frame the rows hold (the advanced fusion).
    pub frame: Facts,
    /// The gantry, with its street's half-width set, when the roster has one.
    pub gantry: Option<Facts>,
    /// The advanced plant the block is laid beside.
    pub plant: Plant,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub centre: Vec3,
    pub pitch: f32,
    /// How far its turrets reach: a frame goes no farther from the centre than this.
    pub reach: f32,
    /// The frame rows, two a tier (`grow`): the first pair either side of the centre bar.
    pub frame_rows: Vec<Rect>,
    /// The frame slots the rows hold.
    pub frame_slots: Vec<Vec3>,
    /// The gantry's slot (its definition, centre, facing) and its apron.
    pub gantry: Option<(Facts, Vec3, i32, Rect)>,
    /// Whether the player's mark placed it.
    pub marked: bool,
    /// The advanced plant it was laid beside, and how far from it the centre stands.
    pub plant_at: Vec3,
    pub from_plant: f32,
    /// Whether the rows run along x (the plant faces north or south), and the frame the rows are laid for.
    pub along_x: bool,
    pub frame: Facts,
    /// How many tiers of rows stand (1 to MAX_TIERS).
    pub tiers: u32,
}

impl Block {
    pub fn lattice(&self) -> Lattice {
        Lattice { centre: self.centre, pitch: self.pitch }
    }

    /// The ground it keeps: the rows for frames, the gantry's slot and apron for the gantry.
    pub fn reservations(&self) -> Vec<Reservation> {
        let mut out: Vec<Reservation> = self.frame_rows.iter().map(|r| Reservation { rect: *r, for_: Reserved::Frames }).collect();
        if let Some((g, at, facing, apron)) = &self.gantry {
            out.push(Reservation { rect: g.footprint.faced(*facing).rect(*at), for_: Reserved::Kind(g.def) });
            out.push(Reservation { rect: *apron, for_: Reserved::Nothing });
        }
        out
    }

    /// Whether `rect` stands on the block's reserved ground.
    pub fn holds(&self, rect: &Rect) -> bool {
        self.reservations().iter().any(|r| r.rect.overlaps(rect))
    }

    /// The frame slots not covered by anything standing or ordered.
    pub fn free_frame_slots(&self, standing: &[Rect], frame: Footprint) -> usize {
        self.frame_slots.iter().filter(|at| !standing.iter().any(|s| s.overlaps(&frame.rect(**at)))).count()
    }

    /// Lays the next tier of rows (two rows, 2 × ROW_SLOTS slots) when its ground holds: clear of everything but our
    /// banks, off the spots and aprons, closing no street. `world` is the world without this block's own
    /// reservations. Err says why not (econ-flagship-5: the six slots were full by 11:28 and the twelve later large
    /// frames stood 376 to 608 from the centre with a third of the turrets in reach of each).
    pub fn grow(&mut self, world: &World) -> Result<usize, String> {
        if self.tiers >= MAX_TIERS {
            return Err(format!("the block has its {MAX_TIERS} tiers of rows; a further one would stand past the turrets' reach, so a second block wants a `block` mark"));
        }
        let tier = self.tiers + 1;
        let (rows, slots) = tier_rows(self.centre, self.along_x, &self.frame, tier);
        // Each row on its own: a tier is whichever of its two rows hold (econ-flagship-6: the block laid 496 west of
        // the advanced plant, its second tier's east row on the plant's footprint, and a two-rows-or-none tier left
        // it at six slots all game).
        let mut held: Vec<(Rect, Vec<Vec3>)> = Vec::new();
        for row in &rows {
            let in_row: Vec<Vec3> = slots.iter().copied().filter(|s| row.overlaps(&self.frame.footprint.rect(*s))).collect();
            if in_row.len() == ROW_SLOTS as usize && ground_holds(world, &self.frame, &[*row], &[*row], &in_row).is_some() {
                held.push((*row, in_row));
            }
        }
        // Both rows together may close a street one alone does not: then the first alone.
        if held.len() == 2 {
            let rects: Vec<Rect> = held.iter().map(|(r, _)| *r).collect();
            let all: Vec<Vec3> = held.iter().flat_map(|(_, s)| s.iter().copied()).collect();
            if ground_holds(world, &self.frame, &rects, &rects, &all).is_none() {
                held.truncate(1);
            }
        }
        if held.is_empty() {
            return Err(format!("no clear ground for the block's tier {tier} of rows"));
        }
        let added: usize = held.iter().map(|(_, s)| s.len()).sum();
        for (row, in_row) in held {
            self.frame_rows.push(row);
            self.frame_slots.extend(in_row);
        }
        self.tiers = tier;
        Ok(added)
    }
}

/// The rows and slots of one tier: tier 1 either side of the centre bar, tier 2 beyond the next bars (a period
/// farther out), tier 3 tier 1's rows carried one row-length along the axis away from the gantry.
fn tier_rows(centre: Vec3, along_x: bool, frame: &Facts, tier: u32) -> (Vec<Rect>, Vec<Vec3>) {
    let rect = |a0: f32, a1: f32, d0: f32, d1: f32| {
        if along_x {
            Rect { x0: centre.x + a0, x1: centre.x + a1, z0: centre.z + d0, z1: centre.z + d1 }
        } else {
            Rect { x0: centre.x + d0, x1: centre.x + d1, z0: centre.z + a0, z1: centre.z + a1 }
        }
    };
    let frame_side = frame.footprint.x.max(frame.footprint.z) as f32 * crate::SQUARE;
    let half_len = ROW_SLOTS as f32 * frame_side / 2.0;
    let (off, shift) = match tier {
        2 => (PERIOD, 0.0),
        3 => (0.0, -2.0 * half_len),
        _ => (0.0, 0.0),
    };
    let (a0, a1) = (shift - half_len, shift + half_len);
    let rows = vec![rect(a0, a1, off + ROW_FROM, off + ROW_FROM + ROW_DEPTH), rect(a0, a1, -off - ROW_FROM - ROW_DEPTH, -off - ROW_FROM)];
    // The slots: a frame's centre on the build grid, inside its row.
    let mut slots = Vec::new();
    for (k, row) in rows.iter().enumerate() {
        for i in 0..ROW_SLOTS {
            let a = shift - half_len + (i as f32 + 0.5) * frame_side;
            let d = if k == 0 { off + ROW_FROM + ROW_DEPTH / 2.0 } else { -off - ROW_FROM - ROW_DEPTH / 2.0 };
            let p = if along_x { Vec3 { x: centre.x + a, y: centre.y, z: centre.z + d } } else { Vec3 { x: centre.x + d, y: centre.y, z: centre.z + a } };
            let p = frame.footprint.snap(p);
            if row.overlaps(&frame.footprint.rect(p)) {
                slots.push(p);
            }
        }
    }
    (rows, slots)
}

/// The block's geometry at `centre` with the rows along `along` (a unit axis) and `deep` across them: the first tier
/// of rows and the gantry's slot.
struct Shape {
    rows: Vec<Rect>,
    slots: Vec<Vec3>,
    gantry: Option<(Vec3, i32, Rect)>,
}

fn shape(centre: Vec3, along_x: bool, wants: &Wants, toward: Vec3) -> Shape {
    let frame_side = wants.frame.footprint.x.max(wants.frame.footprint.z) as f32 * crate::SQUARE;
    let half_len = ROW_SLOTS as f32 * frame_side / 2.0;
    let (rows, slots) = tier_rows(centre, along_x, &wants.frame, 1);
    let gantry = wants.gantry.as_ref().map(|g| {
        let side = g.footprint.x.max(g.footprint.z) as f32 * crate::SQUARE;
        // Past the rows' end and the outer turret column there.
        let a = half_len + TURRET_HALF * 2.0 + side / 2.0;
        let p = if along_x { Vec3 { x: centre.x + a, y: centre.y, z: centre.z } } else { Vec3 { x: centre.x, y: centre.y, z: centre.z + a } };
        let p = g.footprint.snap(p);
        let facing = facings_toward(p, toward)[0];
        let plant = Plant { centre: p, footprint: g.footprint, facing, product_half: g.product_half, ours: true, name: String::new(), widest: String::new(), passable: g.product_passable.clone() };
        (p, facing, plant.apron())
    });
    Shape { rows, slots, gantry }
}

/// Lays the block beside `wants.plant` in `world`: at `mark` when the player gave one, else on the flattest ground
/// within reach of the plant that holds the rows and the gantry's slot clear of everything standing but our own banks
/// (which the report lists for reclaim), off the spots and every apron, closing no street. Among sites equally flat
/// the one with the least of ours on it, then the nearest the plant.
pub fn lay(world: &World, wants: &Wants, mark: Option<Vec3>) -> Result<Block, String> {
    let turret_fp = wants.turret.footprint;
    let (fx, _) = facing_dir(wants.plant.facing);
    // The rows run along the plant's front: across its facing.
    let along_x = fx == 0.0;
    let plant_at = wants.plant.centre;
    // The candidates, nearest first: turret sites (the centre is one) three squares apart.
    let search = |centre: Vec3, radius: f32| -> Vec<(Vec3, f32)> {
        let mut out = Vec::new();
        let first = turret_fp.snap(Vec3 { x: centre.x - radius, y: centre.y, z: centre.z - radius });
        let steps = (2.0 * radius / SEARCH_STEP) as i32 + 1;
        for i in 0..=steps {
            for j in 0..=steps {
                let p = Vec3 { x: first.x + i as f32 * SEARCH_STEP, y: centre.y, z: first.z + j as f32 * SEARCH_STEP };
                if p.dist2d(centre) <= radius {
                    out.push((p, p.dist2d(centre)));
                }
            }
        }
        out.sort_by(|a, b| a.1.total_cmp(&b.1));
        out
    };
    let candidates: Vec<(Vec3, f32)> = match mark {
        // The mark: the nearest point to it that holds the block, within a block's search of it.
        Some(m) => search(m, BLOCK_SEARCH),
        None => search(plant_at, BLOCK_SEARCH * 2f32.powi(WIDENINGS as i32)),
    };
    let mut best: Option<(Vec3, Shape, f32, f32, f32)> = None;
    let mut why = format!("no ground for a block within {:.0} of the advanced plant", BLOCK_SEARCH * 2f32.powi(WIDENINGS as i32));
    let clearance = wants.turret.build_distance + PLANT_CLEARANCE_SLACK;
    for (centre, distance) in candidates {
        if mark.is_none() && world.plants.iter().any(|p| crate::choose::distance_to(&p.rect(), centre) < clearance) {
            continue;
        }
        let s = shape(centre, along_x, wants, world.toward);
        let Some((spread, metal)) = holds(world, wants, &s) else { continue };
        // Marked: the nearest point to the mark that holds, banks on it accepted (the mark is the player's word).
        // Unmarked: the flattest, then the least of ours on it, then the nearest the plant.
        let better = match &best {
            None => true,
            Some((_, _, bs, bm, bd)) if mark.is_some() => distance < *bd || (distance == *bd && (spread < *bs || (spread == *bs && metal < *bm))),
            Some((_, _, bs, bm, bd)) => {
                if (spread - bs).abs() > FLAT_BAND {
                    spread < *bs
                } else if metal != *bm {
                    metal < *bm
                } else {
                    distance < *bd
                }
            }
        };
        if better {
            best = Some((centre, s, spread, metal, distance));
        }
        // Nearest first: a marked block ends at the first point that holds; an unmarked one at the first flat and
        // clear, which nothing farther can beat.
        if mark.is_some() || (spread <= FLAT_BAND && metal == 0.0) {
            break;
        }
    }
    if mark.is_some() && best.is_none() {
        why = "no ground for a block at the mark".to_string();
    }
    let (centre, s, _, _, distance) = best.ok_or(why)?;
    Ok(Block {
        centre,
        pitch: SPARSE_PITCH,
        reach: wants.turret.build_distance,
        frame_rows: s.rows,
        frame_slots: s.slots,
        gantry: s.gantry.map(|(at, facing, apron)| (wants.gantry.clone().expect("a gantry"), at, facing, apron)),
        marked: mark.is_some(),
        plant_at,
        from_plant: distance,
        along_x,
        frame: wants.frame.clone(),
        tiers: 1,
    })
}

/// Whether the block of this shape stands: every frame slot and the gantry's slot on ground that takes them, nothing
/// of ours but banks (generators, converters) on its ground and nothing of anyone else's, off the spots and every
/// plant's apron and the reservations, no street closed. Some((the height spread over it, the metal of ours on it)).
fn holds(world: &World, wants: &Wants, s: &Shape) -> Option<(f32, f32)> {
    // The ground it reserves, and what of it will be solid (the rows filled, the gantry): its apron stays open.
    let mut rects: Vec<Rect> = s.rows.clone();
    let mut solid: Vec<Rect> = s.rows.clone();
    if let Some((at, facing, apron)) = &s.gantry {
        let g = wants.gantry.as_ref()?;
        if !world.ground.stands(g.footprint.faced(*facing), *at, g) || !world.ground.in_map(apron) {
            return None;
        }
        rects.push(g.footprint.faced(*facing).rect(*at));
        solid.push(g.footprint.faced(*facing).rect(*at));
        rects.push(*apron);
    }
    if s.slots.len() < 2 * ROW_SLOTS as usize {
        return None;
    }
    let metal = ground_holds(world, &wants.frame, &rects, &solid, &s.slots)?;
    if let Some((at, facing, _)) = &s.gantry {
        let g = wants.gantry.as_ref()?;
        let own = Plant { centre: *at, footprint: g.footprint, facing: *facing, product_half: g.product_half, ours: true, name: String::new(), widest: String::new(), passable: g.product_passable.clone() };
        if !Flood::of_plant(&world.ground, &own, &world.blocking_rects(), &*world.open).reaches_the_open_round(&solid) {
            return None;
        }
    }
    // Flatness: the height spread over the rows' cells.
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for r in &rects {
        let mut z = r.z0 + GRID / 2.0;
        while z < r.z1 {
            let mut x = r.x0 + GRID / 2.0;
            while x < r.x1 {
                let h = world.ground.height_at(x, z)?;
                lo = lo.min(h);
                hi = hi.max(h);
                x += GRID;
            }
            z += GRID;
        }
    }
    Some((hi - lo, metal))
}

/// Whether ground for frames holds: every slot stands for the frame and is off the spots; every rect in the map,
/// off the spots' boxes, every plant's pad and apron and every reservation, with nothing of ours but banks on it
/// (their metal summed, for the report's reclaim list) and nothing of anyone else's; and no plant's street closed
/// with `solid` added to what stands.
fn ground_holds(world: &World, frame: &Facts, rects: &[Rect], solid: &[Rect], slots: &[Vec3]) -> Option<f32> {
    for slot in slots {
        if !world.ground.stands(frame.footprint, *slot, frame) {
            return None;
        }
        if world.spots.iter().any(|p| p.dist2d(*slot) < world.spot_keep_out) {
            return None;
        }
    }
    let mut metal = 0.0;
    for r in rects {
        if !world.ground.in_map(r) {
            return None;
        }
        if world.spots.iter().any(|p| Rect { x0: p.x - SPOT_BOX_HALF, z0: p.z - SPOT_BOX_HALF, x1: p.x + SPOT_BOX_HALF, z1: p.z + SPOT_BOX_HALF }.overlaps(r)) {
            return None;
        }
        if world.plants.iter().any(|p| p.apron().overlaps(r) || p.rect().overlaps(r)) {
            return None;
        }
        if world.reservations.iter().any(|res| res.rect.overlaps(r)) {
            return None;
        }
        for st in &world.standing {
            let their = if st.ours { st.rect } else { st.rect.grown(ALLY_MARGIN) };
            if !their.overlaps(r) {
                continue;
            }
            // A bank of ours is listed for reclaim; anything else on the ground rules the site out.
            if st.ours && st.bank {
                metal += st.metal;
            } else {
                return None;
            }
        }
    }
    // The solid ground as blocks, the rows filled: no plant's street may close.
    if world.plants.iter().enumerate().filter(|(_, p)| rects.iter().any(|r| p.concerns(r))).any(|(i, _)| !world.plant_street(i).reaches_the_open_round(solid)) {
        return None;
    }
    Some(metal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choose::{Purpose, Request, Standing, choose};
    use crate::Ground;
    use bot_protocol::{Terrain, UnitDefId};

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn flat() -> Terrain {
        let (w, h) = (256u32, 256u32);
        Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() }
    }

    fn facts(def: i32, fp: i32, metal: f32, health: f32) -> Facts {
        Facts { def: UnitDefId(def), footprint: Footprint { x: fp, z: fp }, build_distance: 0.0, build_speed: 0.0, build_time: 1000.0, metal, health, blast: None, height_dif: 10.7, extractor: false, factory: false, building: true, product_half: 0.0, product_passable: None, min_depth: crate::NO_DEPTH_FLOOR, max_depth: 0.0, floater: false }
    }

    fn world<'a>(t: &'a Terrain, open: &'a dyn Fn(Vec3) -> bool) -> World<'a> {
        World { ground: Ground::new(t, 4096.0, 4096.0), spots: &[], spot_keep_out: 100.0, standing: Vec::new(), plants: Vec::new(), builders: Vec::new(), workers: Vec::new(), frames: Vec::new(), frame_slots: Vec::new(), reservations: Vec::new(), refused: Vec::new(), home: at(2048.0, 2048.0), toward: at(4000.0, 2048.0), open: Box::new(open), lattice: None, walks: Default::default(), standing_near: Default::default(), spots_near: Default::default(), streets: Default::default(), blasts_near: Default::default() }
    }

    fn wants() -> Wants {
        let turret = Facts { build_distance: 400.0, build_speed: 200.0, ..facts(5, 6, 230.0, 560.0) };
        let frame = Facts { build_time: 312_500.0, blast: Some((640.0, 10_600.0)), ..facts(7, 12, 9700.0, 8300.0) };
        let gantry = Facts { factory: true, product_half: 64.0, ..facts(8, 24, 7900.0, 16000.0) };
        let plant = Plant { centre: at(2000.0, 2000.0), footprint: Footprint { x: 18, z: 18 }, facing: 1, product_half: 32.0, ours: true, name: "plant_1".into(), widest: "Bull".into(), passable: None };
        Wants { turret, frame, gantry: Some(gantry), plant }
    }

    fn with_plant(w: &mut World, wants: &Wants) {
        w.plants.push(wants.plant.clone());
        w.standing.push(Standing { rect: wants.plant.rect(), def: Some(UnitDefId(9)), kind: "armavp".into(), ours: true, bank: false, metal: 2600.0, health: 4750.0, blast: None, name: "plant_1".into() });
    }

    /// Beside an east-facing advanced plant the block is laid on clear flat ground with its rows along the plant's
    /// front, off the plant and its apron; the gantry's slot and apron lie on the map; six frame slots stand.
    #[test]
    fn the_block_is_laid_beside_the_advanced_plant_off_its_apron() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let wants = wants();
        with_plant(&mut w, &wants);
        let block = lay(&w, &wants, None).expect("a block");
        assert!(block.from_plant <= 2.0 * BLOCK_SEARCH, "{block:?}");
        // Out of the turrets' reach of the plant: the block's turrets never see it (econ-flagship-9).
        assert!(crate::choose::distance_to(&wants.plant.rect(), block.centre) >= wants.turret.build_distance + PLANT_CLEARANCE_SLACK, "{block:?}");
        assert_eq!(block.frame_slots.len(), 6);
        let apron = wants.plant.apron();
        for r in block.reservations() {
            assert!(!r.rect.overlaps(&apron) && !r.rect.overlaps(&wants.plant.rect()), "{r:?} on the plant or its apron");
        }
        // The rows run along z (the plant faces east), 144 deep across x.
        assert!((block.frame_rows[0].x1 - block.frame_rows[0].x0 - ROW_DEPTH).abs() < 1.0, "{:?}", block.frame_rows);
        assert_eq!(block.pitch, SPARSE_PITCH);
        assert!(block.gantry.is_some());
        // The block centre is a turret site: the sub-lattice holds it.
        assert!(block.lattice().holds(block.centre));
    }

    /// With the block laid: a turret goes on the sub-lattice, off the rows; a frame takes a row slot; a solar never
    /// stands on the rows; after `dense` the 48 points open.
    #[test]
    fn turrets_take_the_sublattice_frames_the_rows_and_banks_keep_off() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let wants = wants();
        with_plant(&mut w, &wants);
        let mut block = lay(&w, &wants, None).expect("a block");
        w.reservations = block.reservations();
        w.lattice = Some(block.lattice());
        w.frame_slots = block.frame_slots.clone();
        let builder = Facts { build_distance: 145.0, build_speed: 300.0, ..facts(1, 4, 0.0, 0.0) };
        // A turret near the plant: on the lattice, outside the rows.
        let req = Request { facts: wants.turret.clone(), purpose: Purpose::Turret, centre: block.centre, radius: block.reach, builder_at: block.centre, builder: builder.clone() };
        let site = choose(&w, &req).expect("a turret site");
        assert!(block.lattice().holds(site.at), "{site:?} off the lattice at {:?}", block.centre);
        assert!(!block.holds(&wants.turret.footprint.rect(site.at)), "{site:?} on the rows");
        assert!(site.words.starts_with("6 frame slots"), "with the block laid a turret is scored by the slots it reaches, not the pad: {site:?}");
        // A frame goes into a row, not merely beside one: a turret at the centre reaches the whole band, and the
        // builder stands 700 west, so ground at the band's west edge is 300 nearer its walk and as quick to stand
        // (block-5: both frames went beside the rows that way).
        w.workers.push(crate::Worker { at: block.centre, reach: 400.0, power: 200.0 });
        let far_west = Vec3 { x: block.centre.x - 700.0, y: 0.0, z: block.centre.z };
        let req = Request { facts: wants.frame.clone(), purpose: Purpose::Frame, centre: block.centre, radius: block.reach, builder_at: far_west, builder: builder.clone() };
        let site = choose(&w, &req).expect("a frame site");
        assert!(site.words.contains("1 turret"), "{site:?}");
        assert!(site.words.contains("in a frame row"), "{site:?}");
        assert!(block.frame_rows.iter().any(|r| r.overlaps(&wants.frame.footprint.rect(site.at))), "{site:?} outside the rows {:?}", block.frame_rows);
        // A solar's band at the block centre finds ground, never on the rows or the gantry's slot.
        let solar = facts(3, 10, 155.0, 340.0);
        let req = Request { facts: solar.clone(), purpose: Purpose::Generator { cap: None }, centre: block.centre, radius: 200.0, builder_at: block.centre, builder: builder.clone() };
        let site = choose(&w, &req).expect("a solar site");
        assert!(!block.holds(&solar.footprint.rect(site.at)), "{site:?} on the block's ground");
        // Dense: a point 48 off the centre is on the lattice; sparse it was not.
        let off = Vec3 { x: block.centre.x + 48.0, y: 0.0, z: block.centre.z };
        assert!(!block.lattice().holds(off));
        block.pitch = DENSE_PITCH;
        assert!(block.lattice().holds(off));
    }

    /// The player's mark places the block there; a bank of ours on the ground is accepted and its metal counted,
    /// a plant of ours on it is not.
    #[test]
    fn the_mark_places_the_block_and_banks_on_its_ground_are_accepted() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let wants = wants();
        with_plant(&mut w, &wants);
        let mark = at(2600.0, 2600.0);
        let block = lay(&w, &wants, Some(mark)).expect("a block");
        assert!(block.marked && block.centre.dist2d(mark) < 24.0, "{block:?}");
        // Solars over the block's ground: still laid there, the ground listed.
        for k in 0..3 {
            let p = at(2600.0, 2500.0 + 80.0 * k as f32);
            w.standing.push(Standing { rect: Footprint { x: 10, z: 10 }.rect(p), def: Some(UnitDefId(3)), kind: "armsolar".into(), ours: true, bank: true, metal: 155.0, health: 340.0, blast: None, name: format!("armsolar_{k}") });
        }
        let block = lay(&w, &wants, Some(mark)).expect("a block over banks");
        assert!(block.centre.dist2d(mark) < 24.0, "{block:?}");
        assert!(w.standing.iter().filter(|s| s.bank).any(|s| block.holds(&s.rect)));
        // A plant of ours there instead: the block moves off it.
        w.standing.retain(|s| !s.bank);
        w.standing.push(Standing { rect: Footprint { x: 12, z: 12 }.rect(mark), def: Some(UnitDefId(9)), kind: "armvp".into(), ours: true, bank: false, metal: 700.0, health: 3000.0, blast: None, name: "plant_2".into() });
        let block = lay(&w, &wants, Some(mark)).expect("a block beside the plant");
        assert!(!block.holds(&Footprint { x: 12, z: 12 }.rect(mark)), "{block:?}");
    }

    /// The block grows a tier at a time on clear ground, two rows and six slots each, every slot within the turrets'
    /// reach of the centre. A building of ours (not a bank) on the third tier's ground refuses it while it stands
    /// and the tier holds once it is eaten (his way: each plant reclaimed for the next tier's frames; before
    /// 2026-11-08 the block lay close enough for the plant itself to be that building); a fourth is never laid; a
    /// turret of ours standing on the next tier's ground stops the growth there (econ-flagship-5: six slots full by
    /// 11:28, the later frames outside).
    #[test]
    fn the_block_grows_by_tiers_within_reach_and_stops_at_what_stands() {
        let t = flat();
        let open = |_: Vec3| true;
        let mut w = world(&t, &open);
        let wants = wants();
        with_plant(&mut w, &wants);
        let mut block = lay(&w, &wants, None).expect("a block");
        assert_eq!((block.tiers, block.frame_slots.len(), block.frame_rows.len()), (1, 6, 2));
        assert_eq!(block.grow(&w), Ok(6));
        let (third, _) = tier_rows(block.centre, block.along_x, &block.frame, 3);
        let in_the_way = |r: &Rect| Standing { rect: wants.turret.footprint.rect(Vec3 { x: (r.x0 + r.x1) / 2.0, y: 0.0, z: (r.z0 + r.z1) / 2.0 }), def: Some(UnitDefId(6)), kind: "armrad".into(), ours: true, bank: false, metal: 60.0, health: 500.0, blast: None, name: "radar_1".into() };
        let before = w.standing.len();
        w.standing.extend(third.iter().map(in_the_way));
        assert!(block.grow(&w).unwrap_err().contains("no clear ground"), "the third tier's ground is taken: {block:?}");
        assert_eq!((block.tiers, block.frame_slots.len()), (2, 12));
        // What stood there eaten: the third tier holds.
        w.standing.truncate(before);
        assert_eq!(block.grow(&w), Ok(6));
        assert_eq!((block.tiers, block.frame_slots.len(), block.frame_rows.len()), (3, 18, 6));
        for s in &block.frame_slots {
            assert!(s.dist2d(block.centre) <= block.reach, "slot {s:?} past the reach of {:?}", block.centre);
        }
        for (i, a) in block.frame_rows.iter().enumerate() {
            for b in &block.frame_rows[i + 1..] {
                assert!(!a.overlaps(b), "rows overlap: {a:?} {b:?}");
            }
        }
        assert!(block.grow(&w).unwrap_err().contains("3 tiers"));
        // A turret of ours on one row of the second tier's ground: the tier is the other row alone, three slots.
        let mut w2 = world(&t, &open);
        with_plant(&mut w2, &wants);
        let mut block = lay(&w2, &wants, None).expect("a block");
        let (rows, _) = tier_rows(block.centre, block.along_x, &block.frame, 2);
        let on_it = Vec3 { x: (rows[0].x0 + rows[0].x1) / 2.0, y: 0.0, z: (rows[0].z0 + rows[0].z1) / 2.0 };
        w2.standing.push(Standing { rect: wants.turret.footprint.rect(on_it), def: Some(UnitDefId(5)), kind: "armnanotc".into(), ours: true, bank: false, metal: 230.0, health: 560.0, blast: None, name: "nano_1".into() });
        assert_eq!(block.grow(&w2), Ok(3), "{block:?}");
        assert_eq!((block.tiers, block.frame_slots.len(), block.frame_rows.len()), (2, 9, 3));
        assert!(block.frame_rows.iter().all(|r| !r.overlaps(&wants.turret.footprint.rect(on_it))), "the blocked row was laid: {block:?}");
        // Both rows blocked: the tier is refused.
        let on_other = Vec3 { x: (rows[1].x0 + rows[1].x1) / 2.0, y: 0.0, z: (rows[1].z0 + rows[1].z1) / 2.0 };
        let mut block = lay(&w2, &wants, None).expect("a block");
        w2.standing.push(Standing { rect: wants.turret.footprint.rect(on_other), def: Some(UnitDefId(5)), kind: "armnanotc".into(), ours: true, bank: false, metal: 230.0, health: 560.0, blast: None, name: "nano_2".into() });
        assert!(block.grow(&w2).unwrap_err().contains("no clear ground"), "{block:?}");
        assert_eq!(block.tiers, 1);
    }
}
