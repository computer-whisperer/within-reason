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
    let mut slice = Slice { first, frames: SLICE_FRAMES, ..Default::default() };
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
    }
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
    json!({ "frames": frames, "columns": ["metal", "metal_store", "metal_pull", "metal_income", "metal_spend", "energy", "energy_store", "energy_pull", "energy_income", "energy_spend"], "teams": teams })
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
    let mut head = game.head.clone();
    if let Some(h) = head.as_object_mut() {
        h.remove("t");
        h.insert("format".into(), json!("within-reason-tape"));
        h.insert("version".into(), json!(1));
        h.insert("slice_frames".into(), json!(SLICE_FRAMES));
        h.insert("linger_frames".into(), json!(LINGER_FRAMES));
        h.insert("slices".into(), json!(slices));
        h.insert("last_frame".into(), json!(game.last_frame));
        h.insert("winners".into(), game.winners.clone());
        h.insert("units".into(), json!(game.units.len()));
        h.insert("features".into(), json!(game.features.len()));
        h.insert("projectiles".into(), json!(game.projectiles.len()));
        h.insert("capture".into(), game.stats.clone());
        h.insert("assets".into(), Value::Object(game.assets.clone()));
        h.insert("map_files".into(), game.map_files.clone());
        h.insert("path_check".into(), json!(game.path_check.iter().map(|(k, v)| (k.clone(), json!({ "rows": v.0, "worst": v.1 }))).collect::<serde_json::Map<_, _>>()));
    }
    put(&mut file, b"HEAD", serde_json::to_vec(&head).unwrap(), 0);
    put(&mut file, b"DEFS", serde_json::to_vec(&json!({ "units": game.unit_defs, "weapons": game.weapon_defs, "features": game.feature_defs })).unwrap(), 0);

    let (low, step) = Ground::scale(&game.ground.2, &game.patches);
    let mut ground = Ground { columns: game.ground.0, rows: game.ground.1, low, step, heights: Vec::new() };
    ground.heights = game.ground.2.iter().map(|h| ground.quantize(*h)).collect();
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

    let rows: Vec<UnitRow> = game.units.iter().map(UnitRow::of).collect();
    put(&mut file, b"UNIT", encode_units(&rows), 0);
    put(&mut file, b"FEAT", encode_features(&game.features), 0);
    put(&mut file, b"TEAM", serde_json::to_vec(&teams_table(game)).unwrap(), 0);
    put(&mut file, b"EVNT", serde_json::to_vec(&game.events).unwrap(), 0);
    let patches: Vec<GroundPatch> = game.patches.iter().map(|p| GroundPatch { frame: p.frame, x: p.x, z: p.z, w: p.w, h: p.h, heights: p.heights.iter().map(|h| ground.quantize(*h)).collect() }).collect();
    put(&mut file, b"GPAT", encode_patches(&patches), 0);

    let mut table = W::default();
    table.u32(index.len() as u32);
    for (kind, offset, len, first) in &index {
        table.bytes(kind);
        table.u64(*offset);
        table.u32(*len);
        table.u32(*first);
    }
    let (offset, len) = write_chunk(&mut file, b"INDX", &table.0, None);
    report.add(b"INDX", table.0.len(), len as usize);
    file.extend_from_slice(&offset.to_le_bytes());
    file.extend_from_slice(END);
    report.sections = sections;
    Ok((file, report))
}
