//! The tape's reader as the page loads it (`viewer/scene/tape.js`): one tape a page, plain C functions, no
//! wasm-bindgen. The page fetches byte ranges of `game.tape` and hands them in; it reads the answers as typed arrays
//! over this module's memory (a pointer and a length), which stay good until the next call that fills them.
//!
//! Built by `run/build_viewer.sh`: `cargo build --release --target wasm32-unknown-unknown -p tape-wasm`.

use std::cell::RefCell;
use tape::read::{Frame, Tape};

#[derive(Default)]
struct State {
    tape: Option<Tape>,
    frame: Frame,
    error: String,
    /// The unit table as the page reads it: eight u32 a row (engine id, type, team, created, finished, destroyed,
    /// killer row + 1, builder row + 1) and eight f32 (full health, where it first stood x y z, where it died x y z,
    /// 1 when its type moves).
    unit_ints: Vec<u32>,
    unit_floats: Vec<f32>,
    /// Fourteen f32 a feature: place, front, up, metal, energy, type, the frames it appeared and went (-1 never).
    features: Vec<f32>,
    /// The rectangle of the ground the last `wr_ground` changed: column, row, columns, rows.
    dirty: [u32; 4],
    /// The orders of the unit last asked about (`wr_orders_of`).
    history: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

fn bytes<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    // The page writes `len` bytes at a pointer `wr_alloc` gave it before the call.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// Runs a step that may fail: 0 and the error kept for `wr_text(9)`, or what the step answers.
fn attempt(f: impl FnOnce(&mut State) -> tape::format::Result<i32>) -> i32 {
    with(|s| match f(s) {
        Ok(v) => v,
        Err(e) => {
            s.error = e.0;
            -1
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len.max(1));
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_free(ptr: *mut u8, len: usize) {
    unsafe { drop(Vec::from_raw_parts(ptr, 0, len.max(1))) }
}

/// The index chunk. 0 when it is taken.
#[unsafe(no_mangle)]
pub extern "C" fn wr_open(ptr: *const u8, len: usize) -> i32 {
    attempt(|s| {
        s.tape = Some(Tape::new(bytes(ptr, len))?);
        Ok(0)
    })
}

/// A tape still being written: its new index (`Tape::reindex`); the slices read stay.
#[unsafe(no_mangle)]
pub extern "C" fn wr_reindex(ptr: *const u8, len: usize) -> i32 {
    attempt(|s| {
        let Some(tape) = s.tape.as_mut() else { return tape::format::err("no tape is open") };
        tape.reindex(bytes(ptr, len))?;
        Ok(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_chunk_count() -> u32 {
    with(|s| s.tape.as_ref().map_or(0, |t| t.chunks.len() as u32))
}

/// A chunk's kind (its four letters as a little-endian u32), offset, length and first frame: `what` 0 to 3.
#[unsafe(no_mangle)]
pub extern "C" fn wr_chunk(index: u32, what: u32) -> f64 {
    with(|s| {
        let Some(c) = s.tape.as_ref().and_then(|t| t.chunks.get(index as usize)) else { return -1.0 };
        match what {
            0 => u32::from_le_bytes(c.kind) as f64,
            1 => c.offset as f64,
            2 => c.len as f64,
            _ => c.first as f64,
        }
    })
}

/// Any chunk but a slice. 0 when it is taken.
#[unsafe(no_mangle)]
pub extern "C" fn wr_set_chunk(ptr: *const u8, len: usize) -> i32 {
    attempt(|s| {
        let Some(tape) = s.tape.as_mut() else { return tape::format::err("no tape is open") };
        tape.set_chunk(bytes(ptr, len))?;
        s.unit_ints = tape.units.iter().flat_map(|u| [u.engine_id, u.def, u.team as u32, u.created, u.finished, u.destroyed, u.killer, u.builder]).collect();
        let scale = 1.0 / tape.position_scale;
        s.unit_floats = tape
            .units
            .iter()
            .flat_map(|u| [u.max_health, u.birth.pos[0] as f32 * scale, u.birth.pos[1] as f32 * scale, u.birth.pos[2] as f32 * scale, u.death[0] as f32 * scale, u.death[1] as f32 * scale, u.death[2] as f32 * scale, u.mobile as u32 as f32])
            .collect();
        let frame = |f: u32| if f == u32::MAX { -1.0 } else { f as f32 };
        s.features = tape
            .features
            .iter()
            .flat_map(|f| [f.pos[0], f.pos[1], f.pos[2], f.front[0], f.front[1], f.front[2], f.up[0], f.up[1], f.up[2], f.metal, f.energy, f.def as f32, frame(f.created), frame(f.gone)])
            .collect();
        Ok(0)
    })
}

/// A slice chunk. Its index, or -1.
#[unsafe(no_mangle)]
pub extern "C" fn wr_load_slice(ptr: *const u8, len: usize) -> i32 {
    attempt(|s| {
        let Some(tape) = s.tape.as_mut() else { return tape::format::err("no tape is open") };
        Ok(tape.load_slice(bytes(ptr, len))? as i32)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_drop_slice(index: u32) {
    with(|s| {
        if let Some(tape) = s.tape.as_mut() {
            tape.drop_slice(index)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_has_slice(index: u32) -> i32 {
    with(|s| s.tape.as_ref().is_some_and(|t| t.slices.contains_key(&index)) as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_slice_index(frame: f32) -> u32 {
    with(|s| s.tape.as_ref().map_or(0, |t| t.slice_index(frame)))
}

/// Fills the state at a frame. 1 when it did; 0 when the frame's slice is not loaded (the arrays are as they were).
#[unsafe(no_mangle)]
pub extern "C" fn wr_frame(frame: f32) -> i32 {
    with(|s| {
        let State { tape, frame: out, .. } = s;
        tape.as_ref().is_some_and(|t| t.frame(frame, out)) as i32
    })
}

/// Fills array 10 with every order given to a unit (its row) in the slices now loaded; says how many.
#[unsafe(no_mangle)]
pub extern "C" fn wr_orders_of(row: u32) -> u32 {
    with(|s| {
        let State { tape, history, .. } = s;
        if let Some(t) = tape.as_ref() {
            t.orders_of(row, history);
        }
        (history.len() / tape::read::HISTORY_STRIDE) as u32
    })
}

/// An array's address: 0 units, 1 ranges (u32), 2 projectiles, 3 bursts, 4 the unit table's u32, 5 its f32, 6 the
/// features, 7 the ground now (u16), 8 what the units are doing, 9 the commands just given, 10 a unit's orders, 11
/// what each ally team sees of each unit (u32, beside 0).
#[unsafe(no_mangle)]
pub extern "C" fn wr_array(which: u32) -> *const u8 {
    with(|s| match which {
        0 => s.frame.units.as_ptr() as *const u8,
        1 => s.frame.ranges.as_ptr() as *const u8,
        2 => s.frame.projectiles.as_ptr() as *const u8,
        3 => s.frame.bursts.as_ptr() as *const u8,
        4 => s.unit_ints.as_ptr() as *const u8,
        5 => s.unit_floats.as_ptr() as *const u8,
        6 => s.features.as_ptr() as *const u8,
        8 => s.frame.doing.as_ptr() as *const u8,
        9 => s.frame.commands.as_ptr() as *const u8,
        10 => s.history.as_ptr() as *const u8,
        11 => s.frame.seen.as_ptr() as *const u8,
        12 => s.frame.pieces.as_ptr() as *const u8,
        13 => s.frame.piece_at.as_ptr() as *const u8,
        _ => s.tape.as_ref().map_or(std::ptr::null(), |t| t.ground_now.as_ptr() as *const u8),
    })
}

/// The same arrays' lengths, in elements.
#[unsafe(no_mangle)]
pub extern "C" fn wr_array_len(which: u32) -> u32 {
    with(|s| match which {
        0 => s.frame.units.len(),
        1 => s.frame.ranges.len(),
        2 => s.frame.projectiles.len(),
        3 => s.frame.bursts.len(),
        4 => s.unit_ints.len(),
        5 => s.unit_floats.len(),
        6 => s.features.len(),
        8 => s.frame.doing.len(),
        9 => s.frame.commands.len(),
        10 => s.history.len(),
        11 => s.frame.seen.len(),
        12 => s.frame.pieces.len(),
        13 => s.frame.piece_at.len(),
        _ => s.tape.as_ref().map_or(0, |t| t.ground_now.len()),
    } as u32)
}

/// A text's address: 0 the header, 1 the types, 2 the teams' resources, 3 the events (all JSON), 9 the last error.
#[unsafe(no_mangle)]
pub extern "C" fn wr_text(which: u32) -> *const u8 {
    with(|s| text(s, which).as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_text_len(which: u32) -> u32 {
    with(|s| text(s, which).len() as u32)
}

fn text(s: &State, which: u32) -> &str {
    match (which, s.tape.as_ref()) {
        (0, Some(t)) => &t.head,
        (1, Some(t)) => &t.defs,
        (2, Some(t)) => &t.teams,
        (3, Some(t)) => &t.events,
        (9, _) => &s.error,
        _ => "",
    }
}

/// The ground's shape: 0 columns, 1 rows, 2 the height a zero stands for, 3 the height a step of one adds.
/// The moving pieces' block in `Frame::pieces`: how many transforms a unit has (the most pieces a type has).
#[unsafe(no_mangle)]
pub extern "C" fn wr_piece_stride() -> u32 {
    with(|s| s.tape.as_ref().map_or(0, |t| t.piece_stride as u32))
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_ground_info(what: u32) -> f32 {
    with(|s| {
        let Some(t) = s.tape.as_ref() else { return 0.0 };
        match what {
            0 => t.ground.columns as f32,
            1 => t.ground.rows as f32,
            2 => t.ground.low,
            _ => t.ground.step,
        }
    })
}

/// Brings the ground to a frame. 1 when it changed since the last call: `wr_ground_dirty` then says where.
#[unsafe(no_mangle)]
pub extern "C" fn wr_ground(frame: u32) -> i32 {
    with(|s| match s.tape.as_mut().and_then(|t| t.ground_at(frame)) {
        Some((x, z, w, h)) => {
            s.dirty = [x, z, w, h];
            1
        }
        None => 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn wr_ground_dirty(what: u32) -> u32 {
    with(|s| s.dirty[(what as usize).min(3)])
}
