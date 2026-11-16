//! Reads a tape: the index, the tables, a slice at a time, and the state at a frame as flat arrays a page hands to
//! the GPU. No file access here: the caller fetches byte ranges (the page over HTTP, the tools from a file) and hands
//! them in, so the same code runs in both.

use crate::LINGER_FRAMES;
use crate::format::{CHUNK_HEADER, END, Error, FOOTER, MAGIC, R, Result, err, read_chunk};
use crate::game::{AIM_GROUND, AIM_UNIT, ANGLE_TURN, Activity, PATH_BEAM, PATH_FORMULA, Pose, Projectile, Vital};
use crate::slice::Slice;
use crate::game::Feature;
use crate::tables::{Ground, GroundPatch, UnitRow, decode_features, decode_patches, decode_units};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkRef {
    pub kind: [u8; 4],
    pub offset: u64,
    /// The chunk whole, its header included.
    pub len: u32,
    /// A slice's first frame.
    pub first: u32,
}

/// Where the index chunk begins, from the file's last sixteen bytes.
pub fn index_offset(footer: &[u8]) -> Result<u64> {
    if footer.len() != FOOTER || &footer[8..] != END {
        return err("not a finished tape: its last bytes are not the footer");
    }
    Ok(u64::from_le_bytes(footer[..8].try_into().unwrap()))
}

/// Floats a unit in `Frame::units`: place (elmos), front, up, health as a share of full, build progress 0 to 1, team,
/// flags (1 cloaked, 2 its type moves, 4 finished), row, type, paralysis as a share of full health.
pub const UNIT_STRIDE: usize = 16;
/// Floats a projectile: where it is, where its tail or a beam's far end is, weapon type, path, frames since it
/// began, frames it lives, its owner's row (1-based), team.
pub const PROJECTILE_STRIDE: usize = 12;
/// Floats a burst: place, frames since, weapon type (or the unit type for a death), kind (0 explosion and 1 muzzle
/// fire as the game reported them, 2 a projectile's end, 3 a unit's death), the row of its owner or of the dead
/// (1-based), spare.
pub const BURST_STRIDE: usize = 8;

/// The state at a frame.
#[derive(Debug, Default)]
pub struct Frame {
    /// By unit type, so that a type is one run.
    pub units: Vec<f32>,
    /// (type, first unit, count) for each type present.
    pub ranges: Vec<u32>,
    pub projectiles: Vec<f32>,
    pub bursts: Vec<f32>,
    /// Row -> its place in `units` this frame, or `u32::MAX`.
    slot: Vec<u32>,
    scratch: Vec<f32>,
}

/// A unit at a whole frame, exactly as stamped: what the checker compares.
#[derive(Debug, Clone, PartialEq)]
pub struct Exact {
    pub row: u32,
    pub pose: Pose,
    pub vital: Vital,
    pub activity: Activity,
    pub aim: Vec<(u8, [i16; 2])>,
}

#[derive(Default)]
pub struct Tape {
    pub chunks: Vec<ChunkRef>,
    /// The JSON chunks as text, for the page.
    pub head: String,
    pub defs: String,
    pub teams: String,
    pub events: String,
    pub ground: Ground,
    pub patches: Vec<GroundPatch>,
    pub units: Vec<UnitRow>,
    pub features: Vec<Feature>,
    pub slices: HashMap<u32, Slice>,
    pub slice_frames: u32,
    pub sample_frames: u32,
    pub last_frame: u32,
    pub position_scale: f32,
    /// (frame, row) of every death, in frame order.
    deaths: Vec<(u32, u32)>,
    /// The ground as it stands at `ground_frame`.
    pub ground_now: Vec<u16>,
    ground_patches: usize,
}

impl Tape {
    /// From the index chunk (header and all).
    pub fn new(index_chunk: &[u8]) -> Result<Tape> {
        let (kind, body) = read_chunk(index_chunk)?;
        if &kind != b"INDX" {
            return err("the footer does not point at an index chunk");
        }
        let mut r = R::new(&body);
        let n = r.u32()? as usize;
        let mut chunks = Vec::with_capacity(n);
        for _ in 0..n {
            chunks.push(ChunkRef { kind: r.take(4)?.try_into().unwrap(), offset: r.u64()?, len: r.u32()?, first: r.u32()? });
        }
        Ok(Tape { chunks, slice_frames: crate::SLICE_FRAMES, sample_frames: 3, position_scale: 4.0, ..Default::default() })
    }

    /// The chunks of one kind, in file order.
    pub fn of_kind<'a>(&'a self, kind: &'a [u8; 4]) -> impl Iterator<Item = &'a ChunkRef> {
        self.chunks.iter().filter(move |c| &c.kind == kind)
    }

    /// Takes any chunk but a slice: the header, the tables.
    pub fn set_chunk(&mut self, bytes: &[u8]) -> Result<()> {
        let (kind, body) = read_chunk(bytes)?;
        let text = |body: Vec<u8>| String::from_utf8(body).map_err(|e| Error(format!("a JSON chunk is not UTF-8: {e}")));
        match &kind {
            b"HEAD" => {
                let head: serde_json::Value = serde_json::from_slice(&body).map_err(|e| Error(format!("the tape's header does not parse: {e}")))?;
                let num = |key: &str, default: f64| head.get(key).and_then(|v| v.as_f64()).unwrap_or(default);
                self.slice_frames = num("slice_frames", crate::SLICE_FRAMES as f64) as u32;
                self.sample_frames = num("sample_frames", 3.0) as u32;
                self.last_frame = num("last_frame", 0.0) as u32;
                self.position_scale = num("position_scale", 4.0) as f32;
                if num("angle_scale", 0.0) as i32 != ANGLE_TURN {
                    return err(format!("this tape's angles are {}ths of a turn; this reader's are {ANGLE_TURN}ths", num("angle_scale", 0.0)));
                }
                self.head = text(body)?;
            }
            b"DEFS" => self.defs = text(body)?,
            b"TEAM" => self.teams = text(body)?,
            b"EVNT" => self.events = text(body)?,
            b"GRND" => {
                self.ground = Ground::decode(&body)?;
                self.ground_now = self.ground.heights.clone();
                self.ground_patches = 0;
            }
            b"GPAT" => {
                self.patches = decode_patches(&body)?;
                self.ground_now = self.ground.heights.clone();
                self.ground_patches = 0;
            }
            b"UNIT" => {
                self.units = decode_units(&body)?;
                self.deaths = self.units.iter().enumerate().filter(|(_, u)| u.destroyed != crate::format::NEVER).map(|(row, u)| (u.destroyed, row as u32)).collect();
                self.deaths.sort();
            }
            b"FEAT" => self.features = decode_features(&body)?,
            b"SLIC" => return err("a slice goes to load_slice"),
            other => return err(format!("a chunk of kind {:?} this reader does not know", String::from_utf8_lossy(other))),
        }
        Ok(())
    }

    /// Takes a slice chunk; says which slice it was.
    pub fn load_slice(&mut self, bytes: &[u8]) -> Result<u32> {
        let (kind, body) = read_chunk(bytes)?;
        if &kind != b"SLIC" {
            return err("not a slice chunk");
        }
        let units = &self.units;
        let slice = Slice::decode(&body, &|row| units.get(row as usize).map(|u| u.birth).unwrap_or_default())?;
        let index = slice.first / self.slice_frames.max(1);
        self.slices.insert(index, slice);
        Ok(index)
    }

    pub fn drop_slice(&mut self, index: u32) {
        self.slices.remove(&index);
    }

    pub fn slice_count(&self) -> u32 {
        self.of_kind(b"SLIC").count() as u32
    }

    /// The slice that holds a frame.
    pub fn slice_index(&self, frame: f32) -> u32 {
        ((frame.max(0.0) as u32) / self.slice_frames.max(1)).min(self.slice_count().saturating_sub(1))
    }

    /// A whole tape from memory: every table loaded, no slice. The tools' way in.
    pub fn open(file: &[u8]) -> Result<Tape> {
        if file.len() < MAGIC.len() + FOOTER || &file[..MAGIC.len()] != MAGIC {
            return err("not a tape: the file does not begin with the tape's mark");
        }
        let at = index_offset(&file[file.len() - FOOTER..])? as usize;
        let mut tape = Tape::new(&file[at..file.len() - FOOTER])?;
        // The unit table before anything that leans on it.
        let chunks = tape.chunks.clone();
        for c in chunks.iter().filter(|c| &c.kind != b"SLIC") {
            tape.set_chunk(chunk_bytes(file, c)?)?;
        }
        Ok(tape)
    }

    /// The rows that may be alive in a slice at `frame`: the keyframe's, then those created since.
    fn candidates<'a>(&'a self, slice: &'a Slice, frame: u32) -> impl Iterator<Item = u32> + 'a {
        let from = self.units.partition_point(|u| u.created <= slice.first);
        let to = self.units.partition_point(|u| u.created <= frame);
        slice.key.iter().map(|k| k.row).chain(from as u32..to as u32).filter(move |row| self.units[*row as usize].destroyed > frame)
    }

    /// Every unit alive at a whole frame, exactly as stamped, read from slice `index` (a slice's last frame is the
    /// next one's first: either holds it). None when the slice is not loaded.
    pub fn exact(&self, index: u32, frame: u32) -> Option<Vec<Exact>> {
        let slice = self.slices.get(&index)?;
        let mut out: Vec<Exact> = Vec::new();
        let mut slot: HashMap<u32, usize> = HashMap::new();
        for k in &slice.key {
            if self.units[k.row as usize].destroyed > frame {
                slot.insert(k.row, out.len());
                out.push(Exact { row: k.row, pose: k.pose.unwrap_or(self.units[k.row as usize].birth), vital: k.vital, activity: k.activity, aim: k.aim.clone() });
            }
        }
        for row in self.candidates(slice, frame).skip(out.len()) {
            let u = &self.units[row as usize];
            slot.insert(row, out.len());
            out.push(Exact { row, pose: u.birth, vital: u.birth_vital, activity: Activity::default(), aim: Vec::new() });
        }
        for track in &slice.tracks {
            let Some(&at) = slot.get(&track.row) else { continue };
            let n = track.samples.partition_point(|(f, _)| *f <= frame);
            if n > 0 {
                out[at].pose = track.samples[n - 1].1;
            }
        }
        for (f, row, vital) in &slice.vitals {
            if *f > frame {
                break;
            }
            if let Some(&at) = slot.get(row) {
                out[at].vital = *vital;
            }
        }
        for (f, row, activity) in &slice.activity {
            if *f > frame {
                break;
            }
            if let Some(&at) = slot.get(row) {
                out[at].activity = *activity;
            }
        }
        for (f, row, weapon, dir) in &slice.aim {
            if *f > frame {
                break;
            }
            if let Some(&at) = slot.get(row) {
                match out[at].aim.iter_mut().find(|(w, _)| w == weapon) {
                    Some(entry) => entry.1 = *dir,
                    None => out[at].aim.push((*weapon, *dir)),
                }
            }
        }
        for e in &mut out {
            e.aim.sort_by_key(|(w, _)| *w);
        }
        Some(out)
    }

    /// The state at a frame (a fraction of a frame is between samples) into `out`. False when the frame's slice is
    /// not loaded: `out` is left as it was.
    pub fn frame(&self, frame: f32, out: &mut Frame) -> bool {
        self.frame_in(self.slice_index(frame), frame, out)
    }

    /// The same, read from slice `index`.
    pub fn frame_in(&self, index: u32, frame: f32, out: &mut Frame) -> bool {
        let frame = frame.clamp(0.0, self.last_frame as f32);
        let Some(slice) = self.slices.get(&index) else { return false };
        let whole = frame.floor() as u32;
        let ps = 1.0 / self.position_scale;

        out.slot.clear();
        out.slot.resize(self.units.len(), u32::MAX);
        let rows = &mut out.scratch;
        rows.clear();
        // Unsorted first: the keyframe's units, then the ones created since, each a full row.
        let push = |rows: &mut Vec<f32>, slot: &mut Vec<u32>, row: u32, pose: &Pose, vital: &Vital| {
            let u = &self.units[row as usize];
            let full = u.max_health.max(1.0);
            let flags = (vital.flags & 1) as u32 | (u.mobile as u32) << 1 | ((u.finished <= whole) as u32) << 2;
            let mut nine = [0f32; 9];
            write_pose(&mut nine, &floats(pose, ps));
            rows.extend_from_slice(&nine);
            rows.extend_from_slice(&[vital.health as f32 / full, vital.build as f32 / 1000.0, u.team_at(whole) as f32, flags as f32, row as f32, u.def as f32, vital.paralyze as f32 / full]);
            slot[row as usize] = (rows.len() / UNIT_STRIDE) as u32 - 1;
        };
        for k in &slice.key {
            let u = &self.units[k.row as usize];
            if u.destroyed > whole {
                push(rows, &mut out.slot, k.row, &k.pose.unwrap_or(u.birth), &k.vital);
            }
        }
        let keyed = rows.len() / UNIT_STRIDE;
        for row in self.candidates(slice, whole).skip(keyed) {
            let u = &self.units[row as usize];
            push(rows, &mut out.slot, row, &u.birth, &u.birth_vital);
        }
        // Motion: the last sample at or before the frame, and on towards the next one over its last sample interval
        // (a unit is written only when it moved, so it stood still until then).
        let step = self.sample_frames.max(1) as f32;
        for track in &slice.tracks {
            let at = out.slot[track.row as usize];
            if at == u32::MAX {
                continue;
            }
            let base = at as usize * UNIT_STRIDE;
            let n = track.samples.partition_point(|(f, _)| *f as f32 <= frame);
            // The sample before the frame: in the track, or what the unit had when the slice began.
            let (before_frame, before) = if n > 0 {
                (track.samples[n - 1].0 as f32, track.samples[n - 1].1)
            } else {
                let key = slice.key.binary_search_by_key(&track.row, |k| k.row).ok().and_then(|k| slice.key[k].pose);
                (slice.first as f32, key.unwrap_or(self.units[track.row as usize].birth))
            };
            let mut now = floats(&before, ps);
            if let Some((next_frame, next)) = track.samples.get(n) {
                let begin = (*next_frame as f32 - step).max(before_frame);
                let t = (frame - begin) / (*next_frame as f32 - begin).max(1e-6);
                if t > 0.0 {
                    now = between(&now, &floats(next, ps), t.min(1.0));
                }
            }
            write_pose(&mut rows[base..base + 9], &now);
        }
        for (f, row, vital) in &slice.vitals {
            if *f > whole {
                break;
            }
            let at = out.slot[*row as usize];
            if at != u32::MAX {
                let base = at as usize * UNIT_STRIDE;
                let u = &self.units[*row as usize];
                let full = u.max_health.max(1.0);
                rows[base + 9] = vital.health as f32 / full;
                rows[base + 10] = vital.build as f32 / 1000.0;
                rows[base + 15] = vital.paralyze as f32 / full;
                rows[base + 12] = ((rows[base + 12] as u32 & !1) | (vital.flags & 1) as u32) as f32;
            }
        }
        // By type: a counting sort into `units`, and the slots renumbered to match.
        let mut counts: HashMap<u32, u32> = HashMap::new();
        for row in rows.chunks_exact(UNIT_STRIDE) {
            *counts.entry(row[14] as u32).or_default() += 1;
        }
        let mut types: Vec<u32> = counts.keys().copied().collect();
        types.sort();
        out.ranges.clear();
        let mut next: HashMap<u32, u32> = HashMap::new();
        let mut at = 0u32;
        for def in types {
            out.ranges.extend_from_slice(&[def, at, counts[&def]]);
            next.insert(def, at);
            at += counts[&def];
        }
        out.units.clear();
        out.units.resize(rows.len(), 0.0);
        for row in rows.chunks_exact(UNIT_STRIDE) {
            let place = next.get_mut(&(row[14] as u32)).unwrap();
            let base = *place as usize * UNIT_STRIDE;
            out.units[base..base + UNIT_STRIDE].copy_from_slice(row);
            out.slot[row[13] as usize] = *place;
            *place += 1;
        }

        // Projectiles in flight, and the ends of those lately gone.
        out.projectiles.clear();
        out.bursts.clear();
        let linger = LINGER_FRAMES as f32;
        for p in &slice.projectiles {
            if (p.begin as f32) > frame {
                break; // by first frame
            }
            let life = (p.end - p.begin) as f32;
            if frame < p.end as f32 {
                let t = frame - p.begin as f32;
                let (head, tail) = match p.path {
                    PATH_BEAM => (p.pos, self.beam_end(p, out)),
                    _ => (place_of(p, t), place_of(p, t - 1.0)),
                };
                out.projectiles.extend_from_slice(&[head[0], head[1], head[2], tail[0], tail[1], tail[2], p.wdef as f32, p.path as f32, t, life, p.owner as f32, p.team as f32]);
            } else if frame - (p.end as f32) < linger && p.wdef >= 0 {
                let at = if p.path == PATH_BEAM { self.beam_end(p, out) } else { place_of(p, life) };
                out.bursts.extend_from_slice(&[at[0], at[1], at[2], frame - p.end as f32, p.wdef as f32, 2.0, p.owner as f32, 0.0]);
            }
        }
        for b in &slice.bursts {
            if b.frame as f32 > frame {
                break;
            }
            if frame - (b.frame as f32) < linger {
                out.bursts.extend_from_slice(&[b.pos[0], b.pos[1], b.pos[2], frame - b.frame as f32, b.wdef as f32, b.kind as f32, b.owner as f32, 0.0]);
            }
        }
        let from = self.deaths.partition_point(|(f, _)| (*f as f32) <= frame - linger);
        for (f, row) in &self.deaths[from..] {
            if *f as f32 > frame {
                break;
            }
            let u = &self.units[*row as usize];
            out.bursts.extend_from_slice(&[u.death[0] as f32 * ps, u.death[1] as f32 * ps, u.death[2] as f32 * ps, frame - *f as f32, u.def as f32, 3.0, *row as f32 + 1.0, 0.0]);
        }
        true
    }

    /// Where a beam ends: at the place it was aimed, or at the unit it was aimed at where that stands this frame.
    fn beam_end(&self, p: &Projectile, out: &Frame) -> [f32; 3] {
        if p.aim == AIM_GROUND {
            return p.target;
        }
        if p.aim == AIM_UNIT && p.target[0] >= 1.0 {
            let row = p.target[0] as usize - 1;
            if let Some(&at) = out.slot.get(row) {
                if at != u32::MAX {
                    let base = at as usize * UNIT_STRIDE;
                    return [out.units[base], out.units[base + 1], out.units[base + 2]];
                }
            }
            if let Some(u) = self.units.get(row) {
                let ps = 1.0 / self.position_scale;
                return [u.death[0] as f32 * ps, u.death[1] as f32 * ps, u.death[2] as f32 * ps];
            }
        }
        // Aimed at nothing the tape names: along its velocity, as far as it would go in its life.
        let life = (p.end - p.begin).max(1) as f32;
        [p.pos[0] + p.vel[0] * life, p.pos[1] + p.vel[1] * life, p.pos[2] + p.vel[2] * life]
    }

    /// The ground as it stands at `frame`: brings `ground_now` there and says which rectangle changed since the last
    /// call, as (column, row, columns, rows), or None when nothing did.
    pub fn ground_at(&mut self, frame: u32) -> Option<(u32, u32, u32, u32)> {
        let want = self.patches.partition_point(|p| p.frame <= frame);
        if want == self.ground_patches {
            return None;
        }
        let columns = self.ground.columns;
        let mut dirty: Option<(u32, u32, u32, u32)> = None;
        let mut grow = |x: u32, z: u32, w: u32, h: u32| {
            dirty = Some(match dirty {
                None => (x, z, x + w, z + h),
                Some((a, b, c, d)) => (a.min(x), b.min(z), c.max(x + w), d.max(z + h)),
            });
        };
        let from = if want < self.ground_patches {
            // Backwards: from the start again. Everything the undone patches touched has changed.
            for p in &self.patches[want..self.ground_patches] {
                grow(p.x, p.z, p.w, p.h);
            }
            self.ground_now.copy_from_slice(&self.ground.heights);
            0
        } else {
            self.ground_patches
        };
        for p in &self.patches[from..want] {
            if from == self.ground_patches {
                grow(p.x, p.z, p.w, p.h);
            }
            for row in 0..p.h {
                let to = ((p.z + row) * columns + p.x) as usize;
                let at = (row * p.w) as usize;
                if to + p.w as usize <= self.ground_now.len() {
                    self.ground_now[to..to + p.w as usize].copy_from_slice(&p.heights[at..at + p.w as usize]);
                }
            }
        }
        self.ground_patches = want;
        dirty.map(|(a, b, c, d)| (a, b, c - a, d - b))
    }
}

/// A pose as floats: the place in elmos, then heading, pitch and roll in radians.
fn floats(pose: &Pose, ps: f32) -> [f32; 6] {
    let turn = std::f32::consts::TAU / ANGLE_TURN as f32;
    [pose.pos[0] as f32 * ps, pose.pos[1] as f32 * ps, pose.pos[2] as f32 * ps, pose.rot[0] as f32 * turn, pose.rot[1] as f32 * turn, pose.rot[2] as f32 * turn]
}

/// The share `t` of the way from one pose to another: the place straight, each angle the short way round.
fn between(a: &[f32; 6], b: &[f32; 6], t: f32) -> [f32; 6] {
    let mut out = *a;
    for i in 0..3 {
        out[i] += (b[i] - a[i]) * t;
    }
    for i in 3..6 {
        let mut d = b[i] - a[i];
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        } else if d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        out[i] += d * t;
    }
    out
}

/// The front and up directions of a heading, pitch and roll: `attitude` of tape_dump.lua turned back. The front is
/// the heading turned up by the pitch; the up is the no-roll up (the vertical less its part along the front) turned
/// about the front by the roll.
pub fn attitude(yaw: f32, pitch: f32, roll: f32) -> ([f32; 3], [f32; 3]) {
    let ((sy, cy), (sp, cp), (sr, cr)) = (yaw.sin_cos(), pitch.sin_cos(), roll.sin_cos());
    let front = [sy * cp, sp, cy * cp];
    let level = [-sy * sp, cp, -cy * sp];
    let side = [front[1] * level[2] - front[2] * level[1], front[2] * level[0] - front[0] * level[2], front[0] * level[1] - front[1] * level[0]];
    (front, [level[0] * cr + side[0] * sr, level[1] * cr + side[1] * sr, level[2] * cr + side[2] * sr])
}

/// Nine floats of a unit's row: its place, its front, its up.
fn write_pose(to: &mut [f32], pose: &[f32; 6]) {
    let (front, up) = attitude(pose[3], pose[4], pose[5]);
    to[..3].copy_from_slice(&pose[..3]);
    to[3..6].copy_from_slice(&front);
    to[6..9].copy_from_slice(&up);
}

#[cfg(test)]
mod tests {
    use super::attitude;

    /// The widget's `attitude`, in Rust: the angles of a front and an up.
    fn angles(f: [f32; 3], u: [f32; 3]) -> (f32, f32, f32) {
        let (yaw, pitch) = (f[0].atan2(f[2]), f[1].clamp(-1.0, 1.0).asin());
        let a = [-f[0] * f[1], 1.0 - f[1] * f[1], -f[2] * f[1]];
        let n = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let a = [a[0] / n, a[1] / n, a[2] / n];
        let c = [a[1] * u[2] - a[2] * u[1], a[2] * u[0] - a[0] * u[2], a[0] * u[1] - a[1] * u[0]];
        (yaw, pitch, (c[0] * f[0] + c[1] * f[1] + c[2] * f[2]).atan2(a[0] * u[0] + a[1] * u[1] + a[2] * u[2]))
    }

    #[test]
    fn an_attitude_comes_back_from_its_angles() {
        // A unit on a slope facing every way, and a banking, climbing aircraft.
        for (yaw, pitch, roll) in [(0.0f32, 0.0f32, 0.0f32), (1.2, 0.2, -0.1), (-2.9, -0.4, 0.3), (3.1, 0.9, 1.4), (0.5, -1.2, -2.5)] {
            let (front, up) = attitude(yaw, pitch, roll);
            let dot = front[0] * up[0] + front[1] * up[1] + front[2] * up[2];
            assert!(dot.abs() < 1e-5, "front and up are square: {dot}");
            let (y, p, r) = angles(front, up);
            for (got, want) in [(y, yaw), (p, pitch), (r, roll)] {
                assert!((got - want).abs() < 1e-4, "{got} for {want} at {yaw} {pitch} {roll}");
            }
        }
        // Level, facing south (+z), no roll: the front is +z and the up is +y, as the engine has a new unit.
        assert_eq!(attitude(0.0, 0.0, 0.0), ([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]));
    }
}

/// Where a projectile is `t` frames after it began: by the formula, or between its samples.
pub fn place_of(p: &Projectile, t: f32) -> [f32; 3] {
    if p.path == PATH_FORMULA || p.samples.is_empty() {
        return p.formula(t);
    }
    let frame = p.begin as f32 + t;
    let n = p.samples.partition_point(|(f, _)| (*f as f32) <= frame);
    let (f0, a) = if n == 0 { (p.begin as f32, p.pos) } else { (p.samples[n - 1].0 as f32, p.samples[n - 1].1) };
    let (f1, b) = match p.samples.get(n) {
        Some((f, b)) => (*f as f32, *b),
        None => {
            // Past the last sample: on at the pace of the last stretch.
            let (fp, before) = if n >= 2 { (p.samples[n - 2].0 as f32, p.samples[n - 2].1) } else { (p.begin as f32, p.pos) };
            let span = (f0 - fp).max(1.0);
            let k = (frame - f0) / span;
            return [a[0] + (a[0] - before[0]) * k, a[1] + (a[1] - before[1]) * k, a[2] + (a[2] - before[2]) * k];
        }
    };
    let k = (frame - f0) / (f1 - f0).max(1e-6);
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

/// A chunk's bytes out of a whole file.
pub fn chunk_bytes<'a>(file: &'a [u8], chunk: &ChunkRef) -> Result<&'a [u8]> {
    let (from, to) = (chunk.offset as usize, chunk.offset as usize + chunk.len as usize);
    if to > file.len() || (chunk.len as usize) < CHUNK_HEADER {
        return err("the index names a chunk past the file's end");
    }
    Ok(&file[from..to])
}
