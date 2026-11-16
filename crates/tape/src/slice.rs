//! One slice of the tape: the state at its first frame and everything stamped after it up to and including its last.
//! Columns of varints, each value the difference from the one before where that makes it small.

use crate::format::{R, Result, W, err};
use crate::game::{Activity, Burst, Order, PATH_SAMPLED, Pose, Projectile, Vital, wrap};

/// A unit at the slice's first frame.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyUnit {
    pub row: u32,
    pub vital: Vital,
    pub activity: Activity,
    /// Where it stands, for a unit whose type moves (the others stand where their table row says).
    pub pose: Option<Pose>,
    pub aim: Vec<(u8, [i16; 2])>,
}

/// A unit's samples stamped inside the slice, in frame order.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub row: u32,
    pub samples: Vec<(u32, Pose)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Slice {
    pub first: u32,
    pub frames: u32,
    /// By row, ascending.
    pub key: Vec<KeyUnit>,
    /// By row, ascending.
    pub tracks: Vec<Track>,
    /// (frame, row, what changed to), in frame order; the same for the next two.
    pub vitals: Vec<(u32, u32, Vital)>,
    pub activity: Vec<(u32, u32, Activity)>,
    pub aim: Vec<(u32, u32, u8, [i16; 2])>,
    pub orders: Vec<Order>,
    /// Every projectile in flight at some frame of the slice, by its first frame; its samples whole.
    pub projectiles: Vec<Projectile>,
    pub bursts: Vec<Burst>,
    /// Whether the slice says what each ally team saw (a tape of format 2 made from a stream that does): then
    /// `key_sight` is beside `key`, and `sight` is (frame, row, what changed to) in frame order. [in sight, on radar],
    /// bit `a` for ally team a.
    pub has_sight: bool,
    pub key_sight: Vec<[u16; 2]>,
    pub sight: Vec<(u32, u32, [u16; 2])>,
    /// Whether the slice has what the waste audit reads (format 3, from a raw stream of version 3, which has sight
    /// too): `key_work` and `key_rank` beside `key`, and (frame, row, what changed to) in frame order of [build power
    /// in use in hundredths, a factory's line] and [experience, full health], and the damage each unit took.
    pub has_audit: bool,
    pub key_work: Vec<[u16; 2]>,
    pub work: Vec<(u32, u32, [u16; 2])>,
    pub key_rank: Vec<[f32; 2]>,
    pub rank: Vec<(u32, u32, [f32; 2])>,
    pub hurt: Vec<(u32, u32, f32)>,
    /// Whether the slice has the units' moving pieces (format 4, from a raw stream of version 4 captured with them):
    /// a track for each piece not at rest somewhere in the slice, by row and piece.
    pub has_pieces: bool,
    pub pieces: Vec<PieceTrack>,
}

/// One piece's path through a slice: its numbers at the slice's first frame (or its unit's first), at each keyframe
/// inside, and at the slice's last frame, so that a slice is read alone: (frame, angles, place) in the tape's units
/// (`crate::game::PieceKey`), straight between two points.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PieceTrack {
    pub row: u32,
    pub piece: u16,
    pub points: Vec<(u32, [i32; 3], [i32; 3])>,
}

/// A slice's sections, in the order `encode_sections` ends them. Sight is there only when the slice has sight (a slice
/// of format 1 ends with its bursts), and the three after it only when it has the audit's numbers (format 3).
pub const SECTIONS: [&str; 13] = ["keyframe", "motion", "vitals", "activity", "aim", "orders", "projectiles", "bursts", "sight", "work", "rank", "hurt", "pieces"];

/// Fixed-point scales of a projectile's numbers: eighths of an elmo, 1024ths of an elmo a frame, 65536ths for gravity.
const P_POS: f32 = 8.0;
const P_VEL: f32 = 1024.0;
const P_GRAVITY: f32 = 65536.0;

fn fixed(v: f32, scale: f32) -> i64 {
    (v * scale).round() as i64
}

fn vital_columns(vitals: impl Iterator<Item = Vital>) -> [W; 4] {
    let mut c: [W; 4] = Default::default();
    for v in vitals {
        c[0].var(v.health as u64);
        c[1].var(v.build as u64);
        c[2].var(v.paralyze as u64);
        c[3].u8(v.flags);
    }
    c
}

fn activity_columns(rows: impl Iterator<Item = Activity>) -> [W; 7] {
    let mut c: [W; 7] = Default::default();
    for a in rows {
        c[0].zig(a.cmd as i64);
        c[1].u8(a.how);
        c[2].zig(a.a as i64);
        c[3].zig(a.b as i64);
        c[4].zig(a.c as i64);
        c[5].var(a.builds as u64);
        c[6].var(a.queue as u64);
    }
    c
}

fn read_vital(c: &mut [R; 4]) -> Result<Vital> {
    Ok(Vital { health: c[0].var()? as u32, build: c[1].var()? as u16, paralyze: c[2].var()? as u32, flags: c[3].u8()? })
}

fn read_activity(c: &mut [R; 7]) -> Result<Activity> {
    Ok(Activity { cmd: c[0].zig()? as i32, how: c[1].u8()?, a: c[2].zig()? as i32, b: c[3].zig()? as i32, c: c[4].zig()? as i32, builds: c[5].var()? as u32, queue: c[6].var()? as u16 })
}

/// Reads `N` columns in a row.
fn take_columns<'a, const N: usize>(r: &mut R<'a>) -> Result<[R<'a>; N]> {
    let mut out: Vec<R<'a>> = Vec::with_capacity(N);
    for _ in 0..N {
        out.push(r.column()?);
    }
    Ok(out.try_into().unwrap_or_else(|_| unreachable!()))
}

impl Slice {
    /// `birth(row)` is where the tape's unit table says the unit first stood: what a track's first sample is the
    /// difference from when the unit is not in the keyframe.
    pub fn encode(&self, birth: &dyn Fn(u32) -> Pose) -> Vec<u8> {
        self.encode_sections(birth).0
    }

    /// The slice's bytes, and where each of its sections ends in them (`SECTIONS` names them): for the tool's account
    /// of what a tape's bytes are.
    pub fn encode_sections(&self, birth: &dyn Fn(u32) -> Pose) -> (Vec<u8>, Vec<usize>) {
        let mut ends = Vec::new();
        let mut out = W::default();
        out.u32(self.first);
        out.u32(self.frames);

        // The keyframe.
        out.var(self.key.len() as u64);
        let (mut rows, mut poses, mut has_pose, mut aims) = (W::default(), W::default(), W::default(), W::default());
        let mut last_row = 0u32;
        for k in &self.key {
            rows.var((k.row - last_row) as u64);
            last_row = k.row;
            has_pose.u8(k.pose.is_some() as u8);
            if let Some(p) = &k.pose {
                for v in p.pos {
                    poses.zig(v as i64);
                }
                for v in p.rot {
                    poses.zig(v as i64);
                }
            }
            aims.var(k.aim.len() as u64);
            for (weapon, dir) in &k.aim {
                aims.u8(*weapon);
                for v in dir {
                    aims.zig(*v as i64);
                }
            }
        }
        for c in [&rows, &has_pose, &poses, &aims] {
            out.column(c);
        }
        for c in &vital_columns(self.key.iter().map(|k| k.vital)) {
            out.column(c);
        }
        for c in &activity_columns(self.key.iter().map(|k| k.activity)) {
            out.column(c);
        }

        ends.push(out.0.len());
        // Motion: per track its row and count, then every sample's frame step and its place and attitude as the
        // difference from the sample before (the place a second difference: a unit at a steady pace writes zeros; an
        // angle's difference the short way round).
        out.var(self.tracks.len() as u64);
        let (mut rows, mut counts, mut steps) = (W::default(), W::default(), W::default());
        let mut pos: [W; 3] = Default::default();
        let mut rot: [W; 3] = Default::default();
        let mut last_row = 0u32;
        for track in &self.tracks {
            rows.var((track.row - last_row) as u64);
            last_row = track.row;
            counts.var(track.samples.len() as u64);
            let mut before = match self.key.binary_search_by_key(&track.row, |k| k.row) {
                Ok(at) => self.key[at].pose.unwrap_or_else(|| birth(track.row)),
                Err(_) => birth(track.row),
            };
            let mut frame = self.first;
            let mut pace = [0i64; 3];
            for (f, pose) in &track.samples {
                steps.var((f - frame) as u64);
                frame = *f;
                for i in 0..3 {
                    let step = (pose.pos[i] - before.pos[i]) as i64;
                    pos[i].zig(step - pace[i]);
                    pace[i] = step;
                }
                for i in 0..3 {
                    rot[i].zig(wrap(pose.rot[i] as i32 - before.rot[i] as i32) as i64);
                }
                before = *pose;
            }
        }
        for c in [&rows, &counts, &steps].into_iter().chain(pos.iter()).chain(rot.iter()) {
            out.column(c);
        }

        ends.push(out.0.len());
        // Vitals, activity and aim as they changed.
        out.var(self.vitals.len() as u64);
        let (mut frames, mut rows) = (W::default(), W::default());
        let mut frame = self.first;
        for (f, row, _) in &self.vitals {
            frames.var((f - frame) as u64);
            frame = *f;
            rows.var(*row as u64);
        }
        out.column(&frames);
        out.column(&rows);
        for c in &vital_columns(self.vitals.iter().map(|v| v.2)) {
            out.column(c);
        }

        ends.push(out.0.len());
        out.var(self.activity.len() as u64);
        let (mut frames, mut rows) = (W::default(), W::default());
        let mut frame = self.first;
        for (f, row, _) in &self.activity {
            frames.var((f - frame) as u64);
            frame = *f;
            rows.var(*row as u64);
        }
        out.column(&frames);
        out.column(&rows);
        for c in &activity_columns(self.activity.iter().map(|a| a.2)) {
            out.column(c);
        }

        ends.push(out.0.len());
        out.var(self.aim.len() as u64);
        let mut c: [W; 5] = Default::default();
        let mut frame = self.first;
        for (f, row, weapon, d) in &self.aim {
            c[0].var((f - frame) as u64);
            frame = *f;
            c[1].var(*row as u64);
            c[2].u8(*weapon);
            for i in 0..2 {
                c[3 + i].zig(d[i] as i64);
            }
        }
        for col in &c {
            out.column(col);
        }

        ends.push(out.0.len());
        // Orders.
        out.var(self.orders.len() as u64);
        let mut c: [W; 6] = Default::default();
        let mut frame = self.first;
        for o in &self.orders {
            c[0].var((o.frame - frame) as u64);
            frame = o.frame;
            c[1].var(o.unit as u64);
            c[2].zig(o.cmd as i64);
            c[3].u8(o.shift as u8);
            c[4].var(o.params.len() as u64);
            for p in &o.params {
                c[5].zig(*p as i64);
            }
        }
        for col in &c {
            out.column(col);
        }

        ends.push(out.0.len());
        // Projectiles.
        out.var(self.projectiles.len() as u64);
        let mut c: [W; 13] = Default::default();
        let mut begin = 0i64;
        for p in &self.projectiles {
            c[0].zig(p.begin as i64 - begin);
            begin = p.begin as i64;
            c[1].var((p.end - p.begin) as u64);
            c[2].var(p.id as u64);
            c[3].zig(p.wdef as i64);
            c[4].var(p.owner as u64);
            c[5].zig(p.team as i64);
            c[6].u8(p.path | (p.aim << 4));
            for i in 0..3 {
                c[7].zig(fixed(p.pos[i], P_POS));
                c[8].zig(fixed(p.vel[i], P_VEL));
            }
            c[9].zig(fixed(p.gravity, P_GRAVITY));
            if p.aim == crate::game::AIM_GROUND {
                for i in 0..3 {
                    c[10].zig(fixed(p.target[i], P_POS));
                }
            } else {
                c[10].zig(p.target[0] as i64);
            }
            if p.path == PATH_SAMPLED {
                c[11].var(p.samples.len() as u64);
                let mut frame = p.begin;
                let mut before = [fixed(p.pos[0], P_POS), fixed(p.pos[1], P_POS), fixed(p.pos[2], P_POS)];
                for (f, at) in &p.samples {
                    c[11].var((f - frame) as u64);
                    frame = *f;
                    for i in 0..3 {
                        let v = fixed(at[i], P_POS);
                        c[12].zig(v - before[i]);
                        before[i] = v;
                    }
                }
            }
        }
        for col in &c {
            out.column(col);
        }

        ends.push(out.0.len());
        // Bursts.
        out.var(self.bursts.len() as u64);
        let mut c: [W; 5] = Default::default();
        let mut frame = self.first;
        for b in &self.bursts {
            c[0].var((b.frame - frame) as u64);
            frame = b.frame;
            c[1].u8(b.kind);
            for i in 0..3 {
                c[2].zig(fixed(b.pos[i], P_POS));
            }
            c[3].zig(b.wdef as i64);
            c[4].var(b.owner as u64);
        }
        for col in &c {
            out.column(col);
        }
        ends.push(out.0.len());

        if self.has_sight {
            // Sight: the keyframe's units' in the keyframe's order, then the changes.
            let mut c: [W; 2] = Default::default();
            for s in &self.key_sight {
                c[0].var(s[0] as u64);
                c[1].var(s[1] as u64);
            }
            for col in &c {
                out.column(col);
            }
            out.var(self.sight.len() as u64);
            let mut c: [W; 4] = Default::default();
            let mut frame = self.first;
            for (f, row, s) in &self.sight {
                c[0].var((f - frame) as u64);
                frame = *f;
                c[1].var(*row as u64);
                c[2].var(s[0] as u64);
                c[3].var(s[1] as u64);
            }
            for col in &c {
                out.column(col);
            }
            ends.push(out.0.len());
        }
        if self.has_audit {
            // Work and rank: the keyframe's units' in the keyframe's order, then the changes; then the damage taken.
            let mut c: [W; 2] = Default::default();
            for w in &self.key_work {
                c[0].var(w[0] as u64);
                c[1].var(w[1] as u64);
            }
            for col in &c {
                out.column(col);
            }
            out.var(self.work.len() as u64);
            let mut c: [W; 4] = Default::default();
            let mut frame = self.first;
            for (f, row, w) in &self.work {
                c[0].var((f - frame) as u64);
                frame = *f;
                c[1].var(*row as u64);
                c[2].var(w[0] as u64);
                c[3].var(w[1] as u64);
            }
            for col in &c {
                out.column(col);
            }
            ends.push(out.0.len());
            let mut c: [W; 2] = Default::default();
            for k in &self.key_rank {
                c[0].f32(k[0]);
                c[1].var(k[1].max(0.0).round() as u64);
            }
            for col in &c {
                out.column(col);
            }
            out.var(self.rank.len() as u64);
            let mut c: [W; 4] = Default::default();
            let mut frame = self.first;
            for (f, row, k) in &self.rank {
                c[0].var((f - frame) as u64);
                frame = *f;
                c[1].var(*row as u64);
                c[2].f32(k[0]);
                c[3].var(k[1].max(0.0).round() as u64);
            }
            for col in &c {
                out.column(col);
            }
            ends.push(out.0.len());
            out.var(self.hurt.len() as u64);
            let mut c: [W; 3] = Default::default();
            let mut frame = self.first;
            for (f, row, amount) in &self.hurt {
                c[0].var((f - frame) as u64);
                frame = *f;
                c[1].var(*row as u64);
                c[2].f32(*amount);
            }
            for col in &c {
                out.column(col);
            }
            ends.push(out.0.len());
        }
        if self.has_pieces {
            // The tracks (row step, piece, points), then every point: the frame step from the last (from the slice's
            // first for a track's first), the angles and the place, each the difference from the track's last point.
            out.var(self.pieces.len() as u64);
            let mut c: [W; 3] = Default::default();
            let mut last_row = 0u32;
            for t in &self.pieces {
                c[0].var((t.row - last_row) as u64);
                last_row = t.row;
                c[1].var(t.piece as u64);
                c[2].var(t.points.len() as u64);
            }
            for col in &c {
                out.column(col);
            }
            let mut c: [W; 7] = Default::default();
            for t in &self.pieces {
                let (mut frame, mut angles, mut place) = (self.first, [0i64; 3], [0i64; 3]);
                for (f, a, p) in &t.points {
                    c[0].var((f - frame) as u64);
                    frame = *f;
                    for k in 0..3 {
                        c[1 + k].zig(a[k] as i64 - angles[k]);
                        c[4 + k].zig(p[k] as i64 - place[k]);
                    }
                    angles = a.map(|v| v as i64);
                    place = p.map(|v| v as i64);
                }
            }
            for col in &c {
                out.column(col);
            }
            ends.push(out.0.len());
        }
        (out.0, ends)
    }

    pub fn decode(bytes: &[u8], birth: &dyn Fn(u32) -> Pose) -> Result<Slice> {
        let mut r = R::new(bytes);
        let mut slice = Slice { first: r.u32()?, frames: r.u32()?, ..Default::default() };

        let n = r.var()? as usize;
        let [mut rows, mut has_pose, mut poses, mut aims] = take_columns::<4>(&mut r)?;
        let mut vital = take_columns::<4>(&mut r)?;
        let mut activity = take_columns::<7>(&mut r)?;
        let mut row = 0u32;
        for _ in 0..n {
            row += rows.var()? as u32;
            let pose = if has_pose.u8()? != 0 {
                let mut p = Pose::default();
                for v in &mut p.pos {
                    *v = poses.zig()? as i32;
                }
                for v in &mut p.rot {
                    *v = poses.zig()? as i16;
                }
                Some(p)
            } else {
                None
            };
            let mut aim = Vec::new();
            for _ in 0..aims.var()? {
                let weapon = aims.u8()?;
                aim.push((weapon, [aims.zig()? as i16, aims.zig()? as i16]));
            }
            slice.key.push(KeyUnit { row, vital: read_vital(&mut vital)?, activity: read_activity(&mut activity)?, pose, aim });
        }

        let n = r.var()? as usize;
        let [mut rows, mut counts, mut steps] = take_columns::<3>(&mut r)?;
        let mut pos = take_columns::<3>(&mut r)?;
        let mut rot = take_columns::<3>(&mut r)?;
        let mut row = 0u32;
        for _ in 0..n {
            row += rows.var()? as u32;
            let count = counts.var()? as usize;
            let mut before = match slice.key.binary_search_by_key(&row, |k| k.row) {
                Ok(at) => slice.key[at].pose.unwrap_or_else(|| birth(row)),
                Err(_) => birth(row),
            };
            let mut frame = slice.first;
            let mut pace = [0i64; 3];
            let mut samples = Vec::with_capacity(count);
            for _ in 0..count {
                frame += steps.var()? as u32;
                let mut pose = before;
                for i in 0..3 {
                    pace[i] += pos[i].zig()?;
                    pose.pos[i] = (before.pos[i] as i64 + pace[i]) as i32;
                }
                for i in 0..3 {
                    pose.rot[i] = wrap(before.rot[i] as i32 + rot[i].zig()? as i32) as i16;
                }
                samples.push((frame, pose));
                before = pose;
            }
            slice.tracks.push(Track { row, samples });
        }

        let n = r.var()? as usize;
        let [mut frames, mut rows] = take_columns::<2>(&mut r)?;
        let mut vital = take_columns::<4>(&mut r)?;
        let mut frame = slice.first;
        for _ in 0..n {
            frame += frames.var()? as u32;
            slice.vitals.push((frame, rows.var()? as u32, read_vital(&mut vital)?));
        }

        let n = r.var()? as usize;
        let [mut frames, mut rows] = take_columns::<2>(&mut r)?;
        let mut activity = take_columns::<7>(&mut r)?;
        let mut frame = slice.first;
        for _ in 0..n {
            frame += frames.var()? as u32;
            slice.activity.push((frame, rows.var()? as u32, read_activity(&mut activity)?));
        }

        let n = r.var()? as usize;
        let mut c = take_columns::<5>(&mut r)?;
        let mut frame = slice.first;
        for _ in 0..n {
            frame += c[0].var()? as u32;
            let row = c[1].var()? as u32;
            let weapon = c[2].u8()?;
            slice.aim.push((frame, row, weapon, [c[3].zig()? as i16, c[4].zig()? as i16]));
        }

        let n = r.var()? as usize;
        let mut c = take_columns::<6>(&mut r)?;
        let mut frame = slice.first;
        for _ in 0..n {
            frame += c[0].var()? as u32;
            let unit = c[1].var()? as u32;
            let cmd = c[2].zig()? as i32;
            let shift = c[3].u8()? != 0;
            let mut params = Vec::new();
            for _ in 0..c[4].var()? {
                params.push(c[5].zig()? as i32);
            }
            slice.orders.push(Order { frame, unit, cmd, shift, params });
        }

        let n = r.var()? as usize;
        let mut c = take_columns::<13>(&mut r)?;
        let mut begin = 0i64;
        for _ in 0..n {
            begin += c[0].zig()?;
            let end = begin as u32 + c[1].var()? as u32;
            let id = c[2].var()? as u32;
            let wdef = c[3].zig()? as i32;
            let owner = c[4].var()? as u32;
            let team = c[5].zig()? as i32;
            let packed = c[6].u8()?;
            let (path, aim) = (packed & 15, packed >> 4);
            let (mut at, mut vel) = ([0i64; 3], [0f32; 3]);
            for i in 0..3 {
                at[i] = c[7].zig()?;
                vel[i] = c[8].zig()? as f32 / P_VEL;
            }
            let gravity = c[9].zig()? as f32 / P_GRAVITY;
            let target = if aim == crate::game::AIM_GROUND {
                [c[10].zig()? as f32 / P_POS, c[10].zig()? as f32 / P_POS, c[10].zig()? as f32 / P_POS]
            } else {
                [c[10].zig()? as f32, 0.0, 0.0]
            };
            let mut samples = Vec::new();
            if path == PATH_SAMPLED {
                let mut frame = begin as u32;
                let mut before = at;
                for _ in 0..c[11].var()? {
                    frame += c[11].var()? as u32;
                    for v in &mut before {
                        *v += c[12].zig()?;
                    }
                    samples.push((frame, [before[0] as f32 / P_POS, before[1] as f32 / P_POS, before[2] as f32 / P_POS]));
                }
            }
            slice.projectiles.push(Projectile {
                id, wdef, owner, team, begin: begin as u32, end, pos: [at[0] as f32 / P_POS, at[1] as f32 / P_POS, at[2] as f32 / P_POS], vel, gravity, path, aim, target, samples,
            });
        }

        let n = r.var()? as usize;
        let mut c = take_columns::<5>(&mut r)?;
        let mut frame = slice.first;
        for _ in 0..n {
            frame += c[0].var()? as u32;
            let kind = c[1].u8()?;
            let pos = [c[2].zig()? as f32 / P_POS, c[2].zig()? as f32 / P_POS, c[2].zig()? as f32 / P_POS];
            slice.bursts.push(Burst { frame, kind, pos, wdef: c[3].zig()? as i32, owner: c[4].var()? as u32 });
        }

        if r.left() > 0 {
            slice.has_sight = true;
            let mut c = take_columns::<2>(&mut r)?;
            for _ in 0..slice.key.len() {
                slice.key_sight.push([c[0].var()? as u16, c[1].var()? as u16]);
            }
            let n = r.var()? as usize;
            let mut c = take_columns::<4>(&mut r)?;
            let mut frame = slice.first;
            for _ in 0..n {
                frame += c[0].var()? as u32;
                slice.sight.push((frame, c[1].var()? as u32, [c[2].var()? as u16, c[3].var()? as u16]));
            }
        }
        if r.left() > 0 {
            slice.has_audit = true;
            let mut c = take_columns::<2>(&mut r)?;
            for _ in 0..slice.key.len() {
                slice.key_work.push([c[0].var()? as u16, c[1].var()? as u16]);
            }
            let n = r.var()? as usize;
            let mut c = take_columns::<4>(&mut r)?;
            let mut frame = slice.first;
            for _ in 0..n {
                frame += c[0].var()? as u32;
                slice.work.push((frame, c[1].var()? as u32, [c[2].var()? as u16, c[3].var()? as u16]));
            }
            let mut c = take_columns::<2>(&mut r)?;
            for _ in 0..slice.key.len() {
                slice.key_rank.push([c[0].f32()?, c[1].var()? as f32]);
            }
            let n = r.var()? as usize;
            let mut c = take_columns::<4>(&mut r)?;
            let mut frame = slice.first;
            for _ in 0..n {
                frame += c[0].var()? as u32;
                slice.rank.push((frame, c[1].var()? as u32, [c[2].f32()?, c[3].var()? as f32]));
            }
            let n = r.var()? as usize;
            let mut c = take_columns::<3>(&mut r)?;
            let mut frame = slice.first;
            for _ in 0..n {
                frame += c[0].var()? as u32;
                slice.hurt.push((frame, c[1].var()? as u32, c[2].f32()?));
            }
        }
        if r.left() > 0 {
            slice.has_pieces = true;
            let n = r.var()? as usize;
            let mut c = take_columns::<3>(&mut r)?;
            let mut row = 0u32;
            let mut counts = Vec::with_capacity(n);
            for _ in 0..n {
                row += c[0].var()? as u32;
                slice.pieces.push(PieceTrack { row, piece: c[1].var()? as u16, points: Vec::new() });
                counts.push(c[2].var()? as usize);
            }
            let mut c = take_columns::<7>(&mut r)?;
            for (t, count) in slice.pieces.iter_mut().zip(counts) {
                let (mut frame, mut angles, mut place) = (slice.first, [0i64; 3], [0i64; 3]);
                for _ in 0..count {
                    frame += c[0].var()? as u32;
                    for k in 0..3 {
                        angles[k] += c[1 + k].zig()?;
                        place[k] += c[4 + k].zig()?;
                    }
                    t.points.push((frame, angles.map(|v| v as i32), place.map(|v| v as i32)));
                }
            }
        }
        if r.left() != 0 {
            return err(format!("a slice has {} bytes after its last column", r.left()));
        }
        Ok(slice)
    }
}

/// A projectile's numbers as the tape keeps them: what `decode` gives back for what `encode` was given.
pub fn quantized(p: &Projectile) -> Projectile {
    let q = |v: f32, scale: f32| fixed(v, scale) as f32 / scale;
    let mut out = p.clone();
    for i in 0..3 {
        out.pos[i] = q(p.pos[i], P_POS);
        out.vel[i] = q(p.vel[i], P_VEL);
    }
    out.gravity = q(p.gravity, P_GRAVITY);
    out.target = if p.aim == crate::game::AIM_GROUND { [q(p.target[0], P_POS), q(p.target[1], P_POS), q(p.target[2], P_POS)] } else { [p.target[0], 0.0, 0.0] };
    for (_, at) in &mut out.samples {
        for v in at {
            *v = q(*v, P_POS);
        }
    }
    if out.path != PATH_SAMPLED {
        out.samples.clear();
    }
    out
}

pub fn quantized_burst(b: &Burst) -> Burst {
    let mut out = b.clone();
    for v in &mut out.pos {
        *v = fixed(*v, P_POS) as f32 / P_POS;
    }
    out
}
