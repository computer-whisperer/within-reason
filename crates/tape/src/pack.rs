//! Cuts a game into the tape's chunks.

use crate::format::{END, MAGIC, Result, W, write_chunk};
use crate::game::Game;
use crate::slice::{KeyUnit, Slice, Track};
use crate::tables::{Ground, GroundPatch, UnitRow, encode_features, encode_patches, encode_units};
use crate::{LINGER_FRAMES, SLICE_FRAMES};
use serde_json::{Value, json};

const LEVEL: i32 = 12;

fn compress(raw: &[u8]) -> Option<Vec<u8>> {
    zstd::bulk::compress(raw, LEVEL).ok()
}

/// What `pack` wrote: a chunk kind with its count, raw bytes and stored bytes; and what each section of the slices
/// weighs, unpacked and packed alone (the sections are packed together in a slice, so the second is a guide).
#[derive(Debug, Default)]
pub struct Report {
    pub chunks: Vec<(String, u32, u64, u64)>,
    pub sections: Vec<(&'static str, u64, u64)>,
}

impl Report {
    fn add(&mut self, kind: &[u8; 4], raw: usize, stored: usize) {
        let name = String::from_utf8_lossy(kind).to_string();
        match self.chunks.iter_mut().find(|c| c.0 == name) {
            Some(c) => {
                c.1 += 1;
                c.2 += raw as u64;
                c.3 += stored as u64;
            }
            None => self.chunks.push((name, 1, raw as u64, stored as u64)),
        }
    }
}

/// The slice that begins at `first`: the state then, and what is stamped after it up to `first + SLICE_FRAMES`.
pub fn cut(game: &Game, first: u32) -> Slice {
    let last = first + SLICE_FRAMES;
    let within = |frame: u32| frame > first && frame <= last;
    let mut slice = Slice { first, frames: SLICE_FRAMES, has_sight: game.has_sight(), has_audit: game.has_audit(), has_pieces: game.has_pieces(), ..Default::default() };
    let rests = if slice.has_pieces { game.piece_rest() } else { Default::default() };
    for (row, u) in game.units.iter().enumerate() {
        let row = row as u32;
        if u.created > last {
            break; // the units are in the order they were created
        }
        if u.destroyed <= first {
            continue;
        }
        if u.alive(first) {
            slice.key.push(KeyUnit { row, vital: u.vital(first), activity: u.doing(first), pose: u.mobile.then(|| u.pose(first)), aim: u.aims(first) });
        }
        let from = u.samples.partition_point(|(f, _)| *f <= first);
        let to = u.samples.partition_point(|(f, _)| *f <= last);
        if to > from {
            slice.tracks.push(Track { row, samples: u.samples[from..to].to_vec() });
        }
        slice.vitals.extend(u.vitals.iter().filter(|(f, _)| within(*f)).map(|(f, v)| (*f, row, *v)));
        slice.activity.extend(u.activity.iter().filter(|(f, _)| within(*f)).map(|(f, a)| (*f, row, *a)));
        slice.aim.extend(u.aim.iter().filter(|(f, ..)| within(*f)).map(|(f, w, d)| (*f, row, *w, *d)));
        if slice.has_sight {
            if u.alive(first) {
                slice.key_sight.push(u.sight(first, game.ally(u.team)));
            }
            slice.sight.extend(u.sight.iter().filter(|(f, _)| within(*f)).map(|(f, s)| (*f, row, *s)));
        }
        if slice.has_audit {
            if u.alive(first) {
                slice.key_work.push(u.work_at(first));
                slice.key_rank.push(u.rank_at(first));
            }
            slice.work.extend(u.work.iter().filter(|(f, _)| within(*f)).map(|(f, w)| (*f, row, *w)));
            slice.rank.extend(u.rank.iter().filter(|(f, _)| within(*f)).map(|(f, k)| (*f, row, *k)));
            slice.hurt.extend(u.hurt.iter().filter(|(f, _)| within(*f)).map(|(f, h)| (*f, row, *h)));
        }
        if slice.has_pieces && !u.keys.is_empty() {
            // Each piece's path through the slice, from its numbers at the slice's (or the unit's) first frame to
            // those at its last (or the unit's death).
            let (start, end) = (first.max(u.created), last.min(u.destroyed));
            let rest = rests.get(&u.def);
            let mut i = 0;
            while i < u.keys.len() {
                let piece = u.keys[i].piece;
                let j = i + u.keys[i..].partition_point(|k| k.piece == piece);
                let keys = &u.keys[i..j];
                i = j;
                let r = rest.and_then(|r| r.get(piece as usize - 1)).copied().unwrap_or([0.0; 3]);
                let round = |v: [f64; 3]| v.map(|x| x.round() as i32);
                let mut points = Vec::new();
                let (a, p) = crate::pieces::state_at(keys, start as f64, r);
                points.push((start, round(a), round(p)));
                points.extend(keys.iter().filter(|k| k.frame > start && k.frame < end).map(|k| (k.frame, k.angles, k.place.map(|v| v as i32))));
                if end > start {
                    let (a, p) = crate::pieces::state_at(keys, end as f64, r);
                    points.push((end, round(a), round(p)));
                }
                if points.iter().all(|(_, a, p)| crate::pieces::is_rest(a.map(|v| v as f64), p.map(|v| v as f64), r)) {
                    continue;
                }
                slice.pieces.push(crate::slice::PieceTrack { row, piece, points });
            }
        }
    }
    slice.work.sort_by_key(|w| (w.0, w.1));
    slice.rank.sort_by_key(|k| (k.0, k.1));
    slice.hurt.sort_by_key(|h| (h.0, h.1));
    slice.sight.sort_by_key(|s| (s.0, s.1));
    slice.vitals.sort_by_key(|v| (v.0, v.1));
    slice.activity.sort_by_key(|a| (a.0, a.1));
    slice.aim.sort_by_key(|a| (a.0, a.1, a.2));
    slice.orders = game.orders.iter().filter(|o| within(o.frame)).cloned().collect();
    slice.projectiles = game.projectiles.iter().filter(|p| p.begin <= last && p.end + LINGER_FRAMES > first).cloned().collect();
    slice.bursts = game.bursts.iter().filter(|b| b.frame + LINGER_FRAMES > first && b.frame <= last).cloned().collect();
    slice
}

/// The resources table: each team's ten numbers a second, as columns.
fn teams_table(game: &Game) -> Value {
    let mut frames = Vec::new();
    let mut teams: serde_json::Map<String, Value> = Default::default();
    for (frame, rows) in &game.resources {
        frames.push(json!(frame));
        for row in rows.as_array().into_iter().flatten() {
            let Some(row) = row.as_array() else { continue };
            let Some(team) = row.first().and_then(Value::as_i64) else { continue };
            let list = teams.entry(team.to_string()).or_insert_with(|| json!([]));
            let list = list.as_array_mut().unwrap();
            // a team that appears late is padded so that every team's list runs beside `frames`
            while list.len() + 1 < frames.len() {
                list.push(Value::Null);
            }
            list.push(Value::Array(row[1..].to_vec()));
        }
    }
    let mut columns = vec!["metal", "metal_store", "metal_pull", "metal_income", "metal_spend", "energy", "energy_store", "energy_pull", "energy_income", "energy_spend"];
    if game.raw_version >= 3 {
        columns.extend(["metal_excess", "metal_sent", "metal_received", "energy_excess", "energy_sent", "energy_received"]);
    }
    // The engine's statistics of each team, a row as it added an entry: [frame, the numbers].
    let mut stats: serde_json::Map<String, Value> = Default::default();
    for (frame, rows) in &game.team_stats {
        for row in rows.as_array().into_iter().flatten() {
            let Some(row) = row.as_array() else { continue };
            let Some(team) = row.first().and_then(Value::as_i64) else { continue };
            let list = stats.entry(team.to_string()).or_insert_with(|| json!([]));
            let mut entry = vec![json!(frame)];
            entry.extend(row[1..].iter().cloned());
            list.as_array_mut().unwrap().push(Value::Array(entry));
        }
    }
    let stat_columns = ["metal_used", "metal_produced", "metal_excess", "energy_used", "energy_produced", "energy_excess", "damage_dealt", "damage_received", "units_produced", "units_died", "units_killed"];
    json!({ "frames": frames, "columns": columns, "teams": teams, "stat_columns": stat_columns, "stats": stats })
}

/// The header: the raw stream's, with what the tape adds. `slices` and `last_frame` are the tape's as written (a
/// growing tape's are those of the slices it has so far).
fn head_chunk(game: &Game, slices: u32, last_frame: u32, growing: bool) -> Vec<u8> {
    let mut head = game.head.clone();
    if let Some(h) = head.as_object_mut() {
        h.remove("t");
        h.insert("format".into(), json!("within-reason-tape"));
        // 2: a slice may end with what each ally team saw (when the header has `ally_teams`); 3: and with what the
        // waste audit reads, and the tables have its columns (from a raw stream of version 3).
        h.insert("version".into(), json!(if game.has_pieces() { 4 } else if game.has_audit() { 3 } else { 2 }));
        h.insert("raw_version".into(), json!(game.raw_version.max(1)));
        h.insert("slice_frames".into(), json!(SLICE_FRAMES));
        h.insert("linger_frames".into(), json!(LINGER_FRAMES));
        h.insert("slices".into(), json!(slices));
        h.insert("last_frame".into(), json!(last_frame));
        h.insert("winners".into(), game.winners.clone());
        h.insert("units".into(), json!(game.units.len()));
        h.insert("features".into(), json!(game.features.len()));
        h.insert("projectiles".into(), json!(game.projectiles.len()));
        h.insert("capture".into(), game.stats.clone());
        h.insert("assets".into(), Value::Object(game.assets.clone()));
        h.insert("map_files".into(), game.map_files.clone());
        h.insert("path_check".into(), json!(game.path_check.iter().map(|(k, v)| (k.clone(), json!({ "rows": v.0, "worst": v.1 }))).collect::<serde_json::Map<_, _>>()));
        if growing {
            // (still being written: a reader reads the footer again for more)
            h.insert("growing".into(), json!(true));
        }
    }
    serde_json::to_vec(&head).unwrap()
}

fn defs_chunk(game: &Game) -> Vec<u8> {
    serde_json::to_vec(&json!({ "units": game.unit_defs, "weapons": game.weapon_defs, "features": game.feature_defs })).unwrap()
}

/// The ground at the start, its heights quantized to a scale that holds every height the game reaches (with
/// `margin` elmos more each way for a game still being played, whose craters are not all made yet).
fn ground_of(game: &Game, margin: f32) -> Ground {
    let (low, step) = Ground::scale(&game.ground.2, &game.patches);
    let (low, step) = if margin > 0.0 { (low - margin, (step * 65535.0 + 2.0 * margin) / 65535.0) } else { (low, step) };
    let mut ground = Ground { columns: game.ground.0, rows: game.ground.1, low, step, heights: Vec::new() };
    ground.heights = game.ground.2.iter().map(|h| ground.quantize(*h)).collect();
    ground
}

/// The whole-game tables, after the slices. `FMET` (format 3): each wreck's metal as it changed, [frame, the
/// feature's index in `FEAT`, whole metal].
fn tables(game: &Game, ground: &Ground) -> Vec<(&'static [u8; 4], Vec<u8>)> {
    let rows: Vec<UnitRow> = game.units.iter().map(UnitRow::of).collect();
    let patches: Vec<GroundPatch> = game.patches.iter().map(|p| GroundPatch { frame: p.frame, x: p.x, z: p.z, w: p.w, h: p.h, heights: p.heights.iter().map(|h| ground.quantize(*h)).collect() }).collect();
    let mut tables: Vec<(&'static [u8; 4], Vec<u8>)> = vec![
        (b"UNIT", encode_units(&rows)),
        (b"FEAT", encode_features(&game.features)),
        (b"TEAM", serde_json::to_vec(&teams_table(game)).unwrap()),
        (b"EVNT", serde_json::to_vec(&game.events).unwrap()),
        (b"GPAT", encode_patches(&patches)),
    ];
    if game.has_audit() {
        tables.push((b"FMET", serde_json::to_vec(&game.feature_metal).unwrap()));
    }
    tables
}

/// The index and the footer after it, appended to `file`; `base` is the file's length before `file` (whose offsets
/// start at 0). Returns the index chunk's stored length.
fn close(file: &mut Vec<u8>, base: u64, index: &[([u8; 4], u64, u32, u32)]) -> u32 {
    let mut table = W::default();
    table.u32(index.len() as u32);
    for (kind, offset, len, first) in index {
        table.bytes(kind);
        table.u64(*offset);
        table.u32(*len);
        table.u32(*first);
    }
    let (offset, len) = write_chunk(file, b"INDX", &table.0, None);
    file.extend_from_slice(&(base + offset).to_le_bytes());
    file.extend_from_slice(END);
    len
}

pub fn pack(game: &Game) -> Result<(Vec<u8>, Report)> {
    let mut file = MAGIC.to_vec();
    let mut report = Report::default();
    // (kind, offset, length, first frame)
    let mut index: Vec<([u8; 4], u64, u32, u32)> = Vec::new();
    let mut put = |file: &mut Vec<u8>, kind: &[u8; 4], raw: Vec<u8>, first: u32| {
        let (offset, len) = write_chunk(file, kind, &raw, compress(&raw));
        report.add(kind, raw.len(), len as usize);
        index.push((*kind, offset, len, first));
    };

    let slices = game.last_frame / SLICE_FRAMES + 1;
    put(&mut file, b"HEAD", head_chunk(game, slices, game.last_frame, false), 0);
    put(&mut file, b"DEFS", defs_chunk(game), 0);
    let ground = ground_of(game, 0.0);
    put(&mut file, b"GRND", ground.encode(), 0);

    let birth = |row: u32| game.units[row as usize].birth;
    let mut sections: Vec<(&'static str, u64, u64)> = crate::slice::SECTIONS.iter().map(|name| (*name, 0, 0)).collect();
    for k in 0..slices {
        let slice = cut(game, k * SLICE_FRAMES);
        let (bytes, ends) = slice.encode_sections(&birth);
        let mut from = 8; // after the slice's first frame and length
        for (section, end) in sections.iter_mut().zip(&ends) {
            section.1 += (end - from) as u64;
            section.2 += compress(&bytes[from..*end]).map_or(0, |c| c.len() as u64);
            from = *end;
        }
        put(&mut file, b"SLIC", bytes, k * SLICE_FRAMES);
    }
    for (kind, raw) in tables(game, &ground) {
        put(&mut file, kind, raw, 0);
    }
    let table_len = 4 + index.len() * 20;
    let len = close(&mut file, 0, &index);
    report.add(b"INDX", table_len, len as usize);
    report.sections = sections;
    Ok((file, report))
}

/// Elmos the ground's height scale reaches past the heights a growing tape has seen, for the craters still to come.
const GROWING_MARGIN: f32 = 400.0;

/// A tape written while its game is played (docs/design/2026-11-15-game-tape.md §8, 4b). The ground at the start goes
/// in once; then each time `grow` finds slices the raw stream has passed, it appends them and after them the header,
/// the types, the tables, the index and the footer. The file only grows, so a reader holding an older footer still
/// reads whole chunks; the superseded header and tables are dead bytes until `pack` writes the game's tape compact
/// at its end. The index lists the chunks in a packed tape's order (header, types, ground, slices, tables), the order
/// a reader hands them in.
pub struct Growing {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
    len: u64,
    ground: Option<(Ground, u64, u32)>,
    /// Each slice written: offset, stored length, first frame.
    slices: Vec<(u64, u32, u32)>,
}

impl Growing {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Growing {
        Growing { path: path.into(), file: None, len: 0, ground: None, slices: Vec::new() }
    }

    /// Slices written so far.
    pub fn slices(&self) -> u32 {
        self.slices.len() as u32
    }

    /// Appends the slices `game` (the raw stream as far as it is written) has whole, a slice being whole once the
    /// stream has passed its last frame, and after them the rest. Answers how many slices it added (none: nothing
    /// written).
    pub fn grow(&mut self, game: &Game) -> Result<u32> {
        use std::io::Write;
        let whole = game.last_frame.saturating_sub(1) / SLICE_FRAMES;
        let before = self.slices.len() as u32;
        if whole <= before || game.head.is_null() {
            return Ok(0);
        }
        let io = |e: std::io::Error| crate::format::Error(format!("{}: {e}", self.path.display()));
        let mut out = Vec::new();
        let base = self.len;
        let put = |out: &mut Vec<u8>, kind: &[u8; 4], raw: Vec<u8>| {
            let (offset, len) = write_chunk(out, kind, &raw, compress(&raw));
            (base + offset, len)
        };
        if self.ground.is_none() {
            out.extend_from_slice(MAGIC);
            let ground = ground_of(game, GROWING_MARGIN);
            let (offset, len) = put(&mut out, b"GRND", ground.encode());
            self.ground = Some((ground, offset, len));
        }
        let birth = |row: u32| game.units[row as usize].birth;
        for k in before..whole {
            let (bytes, _) = cut(game, k * SLICE_FRAMES).encode_sections(&birth);
            let (offset, len) = put(&mut out, b"SLIC", bytes);
            self.slices.push((offset, len, k * SLICE_FRAMES));
        }
        let (ground, ground_at, ground_len) = self.ground.as_ref().unwrap();
        let head = put(&mut out, b"HEAD", head_chunk(game, whole, whole * SLICE_FRAMES, true));
        let defs = put(&mut out, b"DEFS", defs_chunk(game));
        let mut index = vec![(*b"HEAD", head.0, head.1, 0), (*b"DEFS", defs.0, defs.1, 0), (*b"GRND", *ground_at, *ground_len, 0)];
        index.extend(self.slices.iter().map(|(offset, len, first)| (*b"SLIC", *offset, *len, *first)));
        for (kind, raw) in tables(game, ground) {
            let (offset, len) = put(&mut out, kind, raw);
            index.push((*kind, offset, len, 0));
        }
        close(&mut out, base, &index);
        if self.file.is_none() {
            self.file = Some(std::fs::File::create(&self.path).map_err(io)?);
        }
        let file = self.file.as_mut().unwrap();
        file.write_all(&out).map_err(io)?;
        self.len += out.len() as u64;
        Ok(whole - before)
    }
}
