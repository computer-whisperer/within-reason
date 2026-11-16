//! The tape's whole-game tables: the unit lives, the features, the ground and its changes.

use crate::format::{NEVER, R, Result, W, err};
use crate::game::{Feature, Patch, Pose, Unit, Vital};

/// A unit life as the tape's `UNIT` table holds it: `game::Unit` without its samples.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitRow {
    pub engine_id: u32,
    pub def: u32,
    pub team: u8,
    pub created: u32,
    pub finished: u32,
    pub destroyed: u32,
    pub killer: u32,
    pub builder: u32,
    pub mobile: bool,
    pub max_health: f32,
    pub birth: Pose,
    pub birth_vital: Vital,
    pub death: [i32; 3],
    pub given: Vec<(u32, u8)>,
}

impl UnitRow {
    pub fn of(u: &Unit) -> UnitRow {
        UnitRow {
            engine_id: u.engine_id, def: u.def, team: u.team, created: u.created, finished: u.finished, destroyed: u.destroyed, killer: u.killer, builder: u.builder,
            mobile: u.mobile, max_health: u.max_health, birth: u.birth, birth_vital: u.birth_vital, death: u.death, given: u.given.clone(),
        }
    }

    pub fn alive(&self, frame: u32) -> bool {
        self.created <= frame && frame < self.destroyed
    }

    pub fn team_at(&self, frame: u32) -> u8 {
        self.given.iter().rev().find(|(f, _)| *f <= frame).map_or(self.team, |(_, t)| *t)
    }
}

/// A frame after `base`, or never: 0 never, else the difference plus one.
fn later(w: &mut W, frame: u32, base: u32) {
    w.var(if frame == NEVER { 0 } else { (frame - base) as u64 + 1 });
}

fn read_later(r: &mut R, base: u32) -> Result<u32> {
    let v = r.var()?;
    Ok(if v == 0 { NEVER } else { base + (v - 1) as u32 })
}

pub fn encode_units(units: &[UnitRow]) -> Vec<u8> {
    let mut c: [W; 9] = Default::default();
    let mut created = 0u32;
    for u in units {
        c[0].var(u.engine_id as u64);
        c[1].var(u.def as u64);
        c[2].u8(u.team);
        c[2].u8(u.mobile as u8);
        c[3].var((u.created - created) as u64);
        created = u.created;
        later(&mut c[3], u.finished, u.created);
        later(&mut c[3], u.destroyed, u.created);
        c[4].var(u.killer as u64);
        c[4].var(u.builder as u64);
        c[5].f32(u.max_health);
        c[5].var(u.birth_vital.health as u64);
        c[5].var(u.birth_vital.build as u64);
        for v in u.birth.pos {
            c[6].zig(v as i64);
        }
        for v in u.birth.rot {
            c[7].zig(v as i64);
        }
        for i in 0..3 {
            c[6].zig((u.death[i] - u.birth.pos[i]) as i64);
        }
        c[8].var(u.given.len() as u64);
        for (frame, team) in &u.given {
            c[8].var(*frame as u64);
            c[8].u8(*team);
        }
    }
    let mut out = W::default();
    out.var(units.len() as u64);
    for col in &c {
        out.column(col);
    }
    out.0
}

pub fn decode_units(bytes: &[u8]) -> Result<Vec<UnitRow>> {
    let mut r = R::new(bytes);
    let n = r.var()? as usize;
    let mut c: Vec<R> = (0..9).map(|_| r.column()).collect::<Result<_>>()?;
    let mut units = Vec::with_capacity(n);
    let mut created = 0u32;
    for _ in 0..n {
        let engine_id = c[0].var()? as u32;
        let def = c[1].var()? as u32;
        let team = c[2].u8()?;
        let mobile = c[2].u8()? != 0;
        created += c[3].var()? as u32;
        let finished = read_later(&mut c[3], created)?;
        let destroyed = read_later(&mut c[3], created)?;
        let killer = c[4].var()? as u32;
        let builder = c[4].var()? as u32;
        let max_health = c[5].f32()?;
        let birth_vital = Vital { health: c[5].var()? as u32, build: c[5].var()? as u16, paralyze: 0, flags: 0 };
        let mut birth = Pose::default();
        for v in &mut birth.pos {
            *v = c[6].zig()? as i32;
        }
        for v in &mut birth.rot {
            *v = c[7].zig()? as i16;
        }
        let mut death = birth.pos;
        for v in &mut death {
            *v += c[6].zig()? as i32;
        }
        let mut given = Vec::new();
        for _ in 0..c[8].var()? {
            given.push((c[8].var()? as u32, c[8].u8()?));
        }
        units.push(UnitRow { engine_id, def, team, created, finished, destroyed, killer, builder, mobile, max_health, birth, birth_vital, death, given });
    }
    Ok(units)
}

pub fn encode_features(features: &[Feature]) -> Vec<u8> {
    let mut out = W::default();
    out.var(features.len() as u64);
    let mut c: [W; 4] = Default::default();
    for f in features {
        c[0].var(f.id as u64);
        c[0].var(f.def as u64);
        c[0].zig(f.team as i64);
        c[1].var(f.created as u64);
        later(&mut c[1], f.gone, f.created);
        for v in f.pos.iter().chain(&f.front).chain(&f.up) {
            c[2].f32(*v);
        }
        c[3].f32(f.metal);
        c[3].f32(f.energy);
    }
    for col in &c {
        out.column(col);
    }
    out.0
}

pub fn decode_features(bytes: &[u8]) -> Result<Vec<Feature>> {
    let mut r = R::new(bytes);
    let n = r.var()? as usize;
    let mut c: Vec<R> = (0..4).map(|_| r.column()).collect::<Result<_>>()?;
    let mut features = Vec::with_capacity(n);
    for _ in 0..n {
        let (id, def, team) = (c[0].var()? as u32, c[0].var()? as u32, c[0].zig()? as i32);
        let created = c[1].var()? as u32;
        let gone = read_later(&mut c[1], created)?;
        let mut v = [0f32; 9];
        for x in &mut v {
            *x = c[2].f32()?;
        }
        features.push(Feature { id, def, team, pos: [v[0], v[1], v[2]], front: [v[3], v[4], v[5]], up: [v[6], v[7], v[8]], metal: c[3].f32()?, energy: c[3].f32()?, created, gone });
    }
    Ok(features)
}

/// The ground's heights as the tape keeps them: `low + step * value`, a u16 a corner.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ground {
    pub columns: u32,
    pub rows: u32,
    pub low: f32,
    pub step: f32,
    pub heights: Vec<u16>,
}

impl Ground {
    /// The scale that holds every height of the game: the start's and every later patch's.
    pub fn scale(start: &[f32], patches: &[Patch]) -> (f32, f32) {
        let all = start.iter().chain(patches.iter().flat_map(|p| p.heights.iter()));
        let (low, high) = all.fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        if low > high {
            return (0.0, 1.0);
        }
        (low, ((high - low) / 65535.0).max(1.0 / 1024.0))
    }

    pub fn quantize(&self, height: f32) -> u16 {
        ((height - self.low) / self.step).round().clamp(0.0, 65535.0) as u16
    }

    pub fn height(&self, value: u16) -> f32 {
        self.low + self.step * value as f32
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = W::default();
        out.u32(self.columns);
        out.u32(self.rows);
        out.f32(self.low);
        out.f32(self.step);
        // Each height as the difference from its western neighbour: the ground is smooth, the differences small.
        let mut before = 0u16;
        for (i, h) in self.heights.iter().enumerate() {
            if i as u32 % self.columns == 0 {
                before = 0;
            }
            out.u16(h.wrapping_sub(before));
            before = *h;
        }
        out.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Ground> {
        let mut r = R::new(bytes);
        let mut ground = Ground { columns: r.u32()?, rows: r.u32()?, low: r.f32()?, step: r.f32()?, heights: Vec::new() };
        let n = (ground.columns * ground.rows) as usize;
        if r.left() != n * 2 {
            return err(format!("the ground chunk holds {} bytes for {} corners", r.left(), n));
        }
        ground.heights.reserve(n);
        let mut before = 0u16;
        for i in 0..n {
            if i as u32 % ground.columns == 0 {
                before = 0;
            }
            before = before.wrapping_add(r.u16()?);
            ground.heights.push(before);
        }
        Ok(ground)
    }
}

/// A change to the ground, in the ground's own u16 scale.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundPatch {
    pub frame: u32,
    pub x: u32,
    pub z: u32,
    pub w: u32,
    pub h: u32,
    pub heights: Vec<u16>,
}

pub fn encode_patches(patches: &[GroundPatch]) -> Vec<u8> {
    let mut out = W::default();
    out.var(patches.len() as u64);
    for p in patches {
        out.var(p.frame as u64);
        for v in [p.x, p.z, p.w, p.h] {
            out.var(v as u64);
        }
        for h in &p.heights {
            out.u16(*h);
        }
    }
    out.0
}

pub fn decode_patches(bytes: &[u8]) -> Result<Vec<GroundPatch>> {
    let mut r = R::new(bytes);
    let n = r.var()? as usize;
    let mut patches = Vec::with_capacity(n);
    for _ in 0..n {
        let frame = r.var()? as u32;
        let (x, z, w, h) = (r.var()? as u32, r.var()? as u32, r.var()? as u32, r.var()? as u32);
        let heights = (0..w * h).map(|_| r.u16()).collect::<Result<_>>()?;
        patches.push(GroundPatch { frame, x, z, w, h, heights });
    }
    Ok(patches)
}
