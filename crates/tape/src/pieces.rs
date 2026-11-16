//! The moving pieces (docs/design/2026-11-15-game-tape.md section 9): a piece's state between its keyframes, and a
//! unit's pieces composed into the transforms a viewer draws its model with.
//!
//! A piece's state is its turn against its parent, the script's own angles about x, y and z (pitch, yaw, roll), and its
//! place against its parent (its offset plus the script's move). The engine builds the rotation as
//! `CQuaternion::FromEulerYPR` of the angles (`S3DModelPiece::ComposeTransform`); the capture widget reads them back from
//! the parent-relative matrix R as pitch = asin(-R12), yaw = atan2(R02, R22), roll = atan2(R10, R11), which
//! `rotation` here undoes (a test holds the two together).

use crate::game::PieceKey;

/// The numbers of a piece at a frame, in the tape's units (angles in `angle_scale`ths of a radian, the place in
/// `place_scale`ths of an elmo): straight between two keyframes, and held past the ends: before the first as it was
/// then (a unit's creation script poses its pieces at once, and the widget's first look at the unit comes up to a few
/// frames later; a piece that moved only later has its rest written as its first keyframe), after the last where it
/// left it. A piece with no keyframes is at `rest` (no turn, its offset).
pub fn state_at(keys: &[PieceKey], frame: f64, rest: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    state_past(keys, keys.partition_point(|k| k.frame as f64 <= frame), frame, rest)
}

/// `state_at` where the number of keyframes at or before `frame` is known (`n`): a reader walking forward through a
/// piece's keyframes keeps it as a cursor instead of searching them at every frame.
pub fn state_past(keys: &[PieceKey], n: usize, frame: f64, rest: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    if keys.is_empty() {
        return ([0.0; 3], rest);
    }
    if n == 0 {
        let k = &keys[0];
        return (k.angles.map(|x| x as f64), k.place.map(|x| x as f64));
    }
    let a = &keys[n - 1];
    let fa = |v: [i32; 3]| v.map(|x| x as f64);
    let fp = |v: [i16; 3]| v.map(|x| x as f64);
    if n == keys.len() {
        return (fa(a.angles), fp(a.place));
    }
    let b = &keys[n];
    let u = (frame - a.frame as f64) / (b.frame as f64 - a.frame as f64);
    let (aa, ba, ap, bp) = (fa(a.angles), fa(b.angles), fp(a.place), fp(b.place));
    ([0, 1, 2].map(|i| aa[i] + u * (ba[i] - aa[i])), [0, 1, 2].map(|i| ap[i] + u * (bp[i] - ap[i])))
}

/// A slice track's numbers at a frame (`crate::slice::PieceTrack::points`): straight between two points, held past
/// the ends.
pub fn track_at(points: &[(u32, [i32; 3], [i32; 3])], frame: f64) -> ([f64; 3], [f64; 3]) {
    let n = points.partition_point(|(f, _, _)| *f as f64 <= frame);
    let f64s = |v: [i32; 3]| v.map(|x| x as f64);
    if n == 0 {
        let (_, a, p) = points[0];
        return (f64s(a), f64s(p));
    }
    let (fa, aa, pa) = points[n - 1];
    if n == points.len() {
        return (f64s(aa), f64s(pa));
    }
    let (fb, ab, pb) = points[n];
    let u = (frame - fa as f64) / (fb as f64 - fa as f64);
    ([0, 1, 2].map(|i| aa[i] as f64 + u * (ab[i] - aa[i]) as f64), [0, 1, 2].map(|i| pa[i] as f64 + u * (pb[i] - pa[i]) as f64))
}

/// Whether a piece's numbers are its rest, give or take the rounding of the capture (two steps of an angle, one of the
/// place).
pub fn is_rest(angles: [f64; 3], place: [f64; 3], rest: [f64; 3]) -> bool {
    angles.iter().all(|a| a.abs() <= 2.0) && place.iter().zip(rest).all(|(p, r)| (p - r).abs() <= 1.0)
}

/// The rotation of the script's angles (radians: pitch, yaw, roll) as the engine makes it, `CQuaternion::FromEulerYPR`
/// turned into a matrix, by columns: [c0, c1, c2], each the image of an axis.
pub fn rotation([p, y, r]: [f64; 3]) -> [[f64; 3]; 3] {
    let (sp, cp) = (p * 0.5).sin_cos();
    let (sy, cy) = (y * 0.5).sin_cos();
    let (sr, cr) = (r * 0.5).sin_cos();
    let x = cr * cy * sp + cp * sr * sy;
    let qy = cp * cr * sy - cy * sp * sr;
    let z = cp * cy * sr - cr * sp * sy;
    let w = cp * cr * cy + sp * sr * sy;
    // R[row][col] of a unit quaternion, then by columns.
    let m = [
        [1.0 - 2.0 * (qy * qy + z * z), 2.0 * (x * qy - z * w), 2.0 * (x * z + qy * w)],
        [2.0 * (x * qy + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (qy * z - x * w)],
        [2.0 * (x * z - qy * w), 2.0 * (qy * z + x * w), 1.0 - 2.0 * (x * x + qy * qy)],
    ];
    [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]]
}

/// The angles the capture widget reads from a parent-relative rotation (by columns), as `tape_dump.lua` does.
pub fn angles_of(c: &[[f64; 3]; 3]) -> [f64; 3] {
    // R[row][col] = c[col][row]
    [(-c[2][1]).clamp(-1.0, 1.0).asin(), c[2][0].atan2(c[2][2]), c[0][1].atan2(c[1][1])]
}

/// A unit type's pieces: each one's parent (its index from 1, 0 the root's) and offset from it at rest (elmos), and
/// where each stands at rest in the model (its offsets summed up to the root).
#[derive(Debug, Clone, Default)]
pub struct PieceModel {
    pub parents: Vec<u16>,
    pub offsets: Vec<[f64; 3]>,
    pub origins: Vec<[f64; 3]>,
}

impl PieceModel {
    pub fn new(parents: Vec<u16>, offsets: Vec<[f64; 3]>) -> PieceModel {
        let mut origins = vec![[0.0; 3]; parents.len()];
        for i in 0..parents.len() {
            let p = parents[i] as usize;
            let base = if p > 0 && p - 1 < i { origins[p - 1] } else { [0.0; 3] };
            origins[i] = [0, 1, 2].map(|k| base[k] + offsets[i][k]);
        }
        PieceModel { parents, offsets, origins }
    }
}

/// A unit's pieces at a frame as the transforms a viewer applies to its model's vertices at rest (which stand at the
/// pieces' rest places): for each piece, twelve floats, the 3x3 rotation by columns then the translation, so that a
/// vertex v of the piece is drawn at R v + t. `local(piece)` gives a piece's angles (radians) and place (elmos) against
/// its parent, or None for rest. Parents come before their children (the engine's piece order).
pub fn compose(model: &PieceModel, local: &dyn Fn(usize) -> Option<([f64; 3], [f64; 3])>, out: &mut Vec<f32>) {
    let n = model.parents.len();
    // model-space rotation (by columns) and place of each piece's own frame
    let mut rot: Vec<[[f64; 3]; 3]> = Vec::with_capacity(n);
    let mut at: Vec<[f64; 3]> = Vec::with_capacity(n);
    for i in 0..n {
        let (r, t) = match local(i) {
            Some((angles, place)) => (rotation(angles), place),
            None => ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], model.offsets[i]),
        };
        let p = model.parents[i] as usize;
        if p > 0 && p - 1 < i {
            let (pr, pt) = (rot[p - 1], at[p - 1]);
            let apply = |v: [f64; 3]| [0, 1, 2].map(|k| pr[0][k] * v[0] + pr[1][k] * v[1] + pr[2][k] * v[2]);
            rot.push([apply(r[0]), apply(r[1]), apply(r[2])]);
            let moved = apply(t);
            at.push([0, 1, 2].map(|k| pt[k] + moved[k]));
        } else {
            rot.push(r);
            at.push(t);
        }
        // v at rest stands at origin + local; drawn at R (v - origin) + place.
        let (r, o) = (rot[i], model.origins[i]);
        let ro = [0, 1, 2].map(|k| r[0][k] * o[0] + r[1][k] * o[1] + r[2][k] * o[2]);
        out.extend_from_slice(&[
            r[0][0] as f32, r[0][1] as f32, r[0][2] as f32, r[1][0] as f32, r[1][1] as f32, r[1][2] as f32, r[2][0] as f32, r[2][1] as f32, r[2][2] as f32,
            (at[i][0] - ro[0]) as f32, (at[i][1] - ro[1]) as f32, (at[i][2] - ro[2]) as f32,
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_widgets_angles_give_back_the_engines_rotation() {
        let mut worst = 0.0f64;
        for i in 0..2000 {
            let f = i as f64;
            // pitch within the asin's half turn, yaw and roll all round
            let a = [((f * 0.37).sin()) * 1.5, (f * 0.91).sin() * 3.1, (f * 1.33).cos() * 3.1];
            let back = angles_of(&rotation(a));
            for k in 0..3 {
                worst = worst.max((back[k] - a[k]).abs());
            }
        }
        assert!(worst < 1e-9, "the angles read back differ by {worst}");
    }

    #[test]
    fn at_rest_a_piece_is_drawn_where_it_stands() {
        let model = PieceModel::new(vec![0, 1, 2], vec![[1.0, 2.0, 3.0], [0.0, 5.0, 0.0], [2.0, 0.0, -1.0]]);
        let mut out = Vec::new();
        compose(&model, &|_| None, &mut out);
        for piece in out.chunks_exact(12) {
            assert_eq!(&piece[..9], &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            assert!(piece[9..].iter().all(|v| v.abs() < 1e-6), "{piece:?}");
        }
        // A quarter turn of the middle piece about y carries the last one round with it.
        let mut out = Vec::new();
        compose(&model, &|i| (i == 1).then_some(([0.0, std::f64::consts::FRAC_PI_2, 0.0], [0.0, 5.0, 0.0])), &mut out);
        let last = &out[24..36];
        // the last piece's rest place (3, 7, 2) turns about the middle one's (1, 7, 3)
        let v = [3.0f32, 7.0, 2.0];
        let drawn = [0, 1, 2].map(|k| last[k] * v[0] + last[3 + k] * v[1] + last[6 + k] * v[2] + last[9 + k]);
        let r = rotation([0.0, std::f64::consts::FRAC_PI_2, 0.0]);
        let d = [2.0, 0.0, -1.0];
        let want = [0, 1, 2].map(|k| [1.0, 7.0, 3.0][k] + r[0][k] * d[0] + r[1][k] * d[1] + r[2][k] * d[2]);
        for k in 0..3 {
            assert!((drawn[k] as f64 - want[k]).abs() < 1e-5, "{drawn:?} against {want:?}");
        }
    }
}
