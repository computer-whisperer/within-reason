//! The waste audit's dump, `audit.jsonl` (docs/harness/audit.md), written from a tape row for row as
//! `audit_dump.lua` writes it while a replay plays: so `run/audit.py`, `run/audit_side.py` and the review skill read a
//! game's tape as they read a replay's dump, and an arena game needs no replay for its audit
//! (docs/design/2026-11-15-game-tape.md §8, 4c).
//!
//! Where the two can differ: a place is the tape's (quarters of an elmo) floored, where the widget floors the engine's;
//! health and build progress come from the tape's whole health points and thousandths; a repair, reclaim, resurrect or
//! capture of one target inside an area names its target, where the widget wrote the target's id and the area's z as
//! a place; the teams' numbers are the tape's tenths.

use std::collections::BTreeMap;
use std::io::Write;

use serde_json::{Value, json};

use crate::format::{NEVER, Result, err};
use crate::read::{Tape, chunk_bytes};

/// Frames between the dump's samples of every unit (the widget's `SAMPLE`).
const SAMPLE: u32 = 30;
/// Frames between its looks at the wrecks (the widget's `FEATURES`).
const WRECKS: u32 = 450;

/// What `write` wrote.
pub struct Written {
    pub samples: u32,
    pub unit_rows: u64,
    pub commands: u64,
    pub events: u64,
}

fn parse(text: &str, what: &str) -> Result<Value> {
    if text.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(text).map_err(|e| crate::format::Error(format!("the tape's {what} does not parse: {e}")))
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

/// A number as the widget writes one: whole numbers whole, the rest to six figures.
fn figure(v: f64) -> Value {
    if v == v.trunc() && v.abs() < 1e15 {
        json!(v as i64)
    } else {
        let digits = 6 - (v.abs().log10().floor() as i32 + 1);
        let scale = 10f64.powi(digits.clamp(0, 12));
        json!((v * scale).round() / scale)
    }
}

/// The value of a list of `[frame, ...]` rows at or before a frame.
fn at_or_before(rows: &[Value], frame: u32) -> Option<&Value> {
    let n = rows.partition_point(|r| r.get(0).and_then(Value::as_f64).unwrap_or(0.0) as u32 <= frame);
    n.checked_sub(1).map(|i| &rows[i])
}

pub fn write(file: &[u8], out: &mut impl Write) -> Result<Written> {
    let mut tape = Tape::open(file)?;
    let head = parse(&tape.head, "header")?;
    let defs = parse(&tape.defs, "types")?;
    let teams_table = parse(&tape.teams, "teams table")?;
    let events = parse(&tape.events, "events")?;
    let io = |e: std::io::Error| crate::format::Error(format!("writing the audit: {e}"));
    let put = |out: &mut dyn Write, row: &Value| -> Result<()> {
        serde_json::to_writer(&mut *out, row).map_err(|e| crate::format::Error(e.to_string()))?;
        out.write_all(b"\n").map_err(io)
    };

    // The seats, every team but gaia's in the engine's order, and each team's enemy as the widget picks it: the ally
    // team of the last other team not its ally (a two-sided game's other side).
    let gaia = |t: &&Value| num(t, "gaia") > 0.0;
    let mut seats: Vec<&Value> = head.get("teams").and_then(Value::as_array).into_iter().flatten().filter(|t| !gaia(t)).collect();
    seats.sort_by_key(|t| num(t, "team") as i64);
    let teams: Vec<i64> = seats.iter().map(|t| num(t, "team") as i64).collect();
    let ally: BTreeMap<i64, i64> = seats.iter().map(|t| (num(t, "team") as i64, num(t, "ally") as i64)).collect();
    let mut enemy: BTreeMap<i64, i64> = BTreeMap::new();
    for team in &teams {
        for other in &teams {
            if ally[other] != ally[team] {
                enemy.insert(*team, ally[other]);
            }
        }
    }
    let start = |t: &Value, i: usize| t.get("start").and_then(|s| s.get(i)).and_then(Value::as_f64).unwrap_or(0.0).floor() as i64;
    let seat_rows: Vec<Value> = seats
        .iter()
        .map(|t| json!({ "team": num(t, "team") as i64, "ally": num(t, "ally") as i64, "side": t.get("side").cloned().unwrap_or(json!("")), "name": t.get("name").cloned().unwrap_or(json!("")), "ai": num(t, "ai") as i64, "start": [start(t, 0), start(t, 2)] }))
        .collect();
    let map = head.get("map").cloned().unwrap_or(Value::Null);
    put(out, &json!({ "t": "header", "format": "within-reason-audit", "version": 1, "sample_frames": SAMPLE, "made_from": "game.tape",
        "map": { "name": map.get("name").cloned().unwrap_or(json!("")), "width": num(&map, "width"), "height": num(&map, "height") },
        "seats": seat_rows, "spots": head.get("spots").cloned().unwrap_or(json!([])) }))?;

    // The types, as the widget's `defRow` has them.
    let unit_defs: Vec<Value> = defs.get("units").and_then(Value::as_array).cloned().unwrap_or_default();
    for d in &unit_defs {
        let field = |key: &str| d.get(key).cloned().unwrap_or(json!(0));
        let weapons = d.get("weapons").and_then(Value::as_array).map_or(0, Vec::len);
        put(out, &json!({ "t": "def", "d": {
            "id": field("id"), "name": field("name"), "power": field("power"), "human": field("human"), "metal": field("metal"), "energy": field("energy"),
            "build_time": field("build_time"), "build_speed": field("build_speed"), "build_distance": field("build_distance"), "speed": field("speed"),
            "health": field("health"), "reach": field("reach"), "weapons": weapons, "building": field("building"), "factory": field("factory"), "air": field("air"),
            "extracts": field("extracts"), "energy_make": field("energy_make"), "energy_upkeep": field("energy_upkeep"), "wind": field("wind"), "tidal": field("tidal"),
            "metal_store": field("metal_store"), "energy_store": field("energy_store"), "can_assist": field("can_assist"), "can_reclaim": field("can_reclaim"),
            "tech": field("tech"), "sight": field("sight"), "radar": field("radar"), "xsize": field("xsize"), "zsize": field("zsize"),
            "options": d.get("options").cloned().unwrap_or(json!([])), "converter": field("converter"),
        } }))?;
    }

    // The rows that are not samples, by (frame, the order within a frame, a count): a unit's events, then the
    // commands, as the engine calls a widget in a frame before its `GameFrame`.
    let mut pending: BTreeMap<(u32, u8, u64), Value> = BTreeMap::new();
    let mut seq = 0u64;
    let mut written = Written { samples: 0, unit_rows: 0, commands: 0, events: 0 };
    let units = tape.units.clone();
    let engine = |row: u32| if row == 0 { 0 } else { units[row as usize - 1].engine_id as i64 };
    let team_at = |row: usize, frame: u32| units[row].given.iter().filter(|(f, _)| *f <= frame).last().map_or(units[row].team, |(_, t)| *t) as i64;
    let scale = num(&head, "position_scale").max(1.0);
    let place = |p: [i32; 3]| ((p[0] as f64 / scale).floor() as i64, (p[2] as f64 / scale).floor() as i64);
    for (row, u) in units.iter().enumerate() {
        let (x, z) = place(u.birth.pos);
        let mut add = |frame: u32, value: Value| {
            seq += 1;
            pending.insert((frame, 0, seq), value);
        };
        add(u.created, json!({ "t": "ev", "f": u.created, "k": "created", "u": u.engine_id, "d": u.def, "team": u.team, "x": x, "z": z, "by": engine(u.builder) }));
        let mut team = u.team as i64;
        for (frame, to) in &u.given {
            add(*frame, json!({ "t": "ev", "f": frame, "k": "given", "u": u.engine_id, "d": u.def, "team": *to as i64, "from": team }));
            team = *to as i64;
        }
        if u.finished != NEVER {
            add(u.finished, json!({ "t": "ev", "f": u.finished, "k": "finished", "u": u.engine_id, "d": u.def, "team": team_at(row, u.finished), "x": x, "z": z }));
        }
        if u.destroyed != NEVER {
            let (dx, dz) = place(u.death);
            let killer = (u.killer > 0).then(|| &units[u.killer as usize - 1]);
            // (the build progress at its death is its last vitals' in the slice it died in: filled in below)
            add(u.destroyed, json!({ "t": "ev", "f": u.destroyed, "k": "destroyed", "u": u.engine_id, "d": u.def, "team": team_at(row, u.destroyed), "x": dx, "z": dz,
                "by": killer.map_or(0, |k| k.engine_id as i64), "by_d": killer.map_or(0, |k| k.def as i64), "by_team": killer.map_or(-1, |k| k.team as i64), "progress": 100, "row": row }));
        }
    }
    written.events = pending.len() as u64;

    // The teams' numbers each second, by name, and the engine's statistics as it added them.
    let columns: Vec<String> = teams_table.get("columns").and_then(Value::as_array).into_iter().flatten().filter_map(|c| c.as_str().map(String::from)).collect();
    let column = |name: &str| columns.iter().position(|c| c == name);
    let frames: Vec<u32> = teams_table.get("frames").and_then(Value::as_array).into_iter().flatten().map(|f| f.as_f64().unwrap_or(0.0) as u32).collect();
    let stat_columns: Vec<String> = teams_table.get("stat_columns").and_then(Value::as_array).into_iter().flatten().filter_map(|c| c.as_str().map(String::from)).collect();
    let side = |team: i64, frame: u32| -> Value {
        let at = frames.partition_point(|f| *f <= frame).checked_sub(1);
        let row = at.and_then(|i| teams_table.get("teams")?.get(team.to_string())?.get(i)).and_then(Value::as_array);
        let get = |name: &str| row.and_then(|r| column(name).and_then(|c| r.get(c))).and_then(Value::as_f64).unwrap_or(0.0);
        let of = |kind: &str| -> Value {
            ["", "_store", "_pull", "_income", "_spend", "_excess", "_sent", "_received"].iter().map(|s| figure(get(&format!("{kind}{s}")))).collect::<Vec<_>>().into()
        };
        let stats = teams_table.get("stats").and_then(|s| s.get(team.to_string())).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let st: serde_json::Map<String, Value> = at_or_before(stats, frame)
            .and_then(Value::as_array)
            .map(|r| stat_columns.iter().enumerate().map(|(i, name)| (name.clone(), figure(r.get(i + 1).and_then(Value::as_f64).unwrap_or(0.0)))).collect())
            .unwrap_or_default();
        json!({ "team": team, "m": of("metal"), "e": of("energy"), "st": st })
    };

    // The wrecks' metal: each feature's changes in frame order.
    let feature_defs: BTreeMap<u32, String> = defs.get("features").and_then(Value::as_array).into_iter().flatten().map(|d| (num(d, "id") as u32, d.get("name").and_then(Value::as_str).unwrap_or("").to_string())).collect();
    let mut metal_of: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    for (frame, at, metal) in &tape.feature_metal {
        metal_of.entry(*at).or_default().push((*frame, *metal));
    }
    let features = tape.features.clone();
    let wrecks = |frame: u32| -> Value {
        let mut rows = Vec::new();
        for (at, f) in features.iter().enumerate() {
            if f.created > frame || (f.gone != NEVER && f.gone <= frame) {
                continue;
            }
            let changes = metal_of.get(&(at as u32)).map(Vec::as_slice).unwrap_or(&[]);
            let n = changes.partition_point(|(c, _)| *c <= frame);
            let metal = if n > 0 { changes[n - 1].1 as i64 } else { f.metal.floor() as i64 };
            if metal >= 1 {
                rows.push(json!([f.id, feature_defs.get(&f.def).cloned().unwrap_or_default(), f.pos[0].floor() as i64, f.pos[2].floor() as i64, metal]));
            }
        }
        json!({ "t": "wrecks", "f": frame, "w": rows })
    };

    // The slices in order: each one's commands into the pending rows, then a sample every SAMPLE frames. The damage
    // taken goes to the sample that ends the second it was taken in (a dying unit's row is at its death, the others'
    // at the samples), which may be the next slice's.
    let mut hurt: BTreeMap<u32, Vec<(u32, f32)>> = BTreeMap::new();
    // A unit that died is listed once more, at the first sample at or after its death, as the widget lists a dying unit
    // (the engine keeps one for a while after it is told of the death): where it died, its last numbers, and the
    // experience it died with (what it dealt in its last second). Its team and its row, by the sample's frame.
    let mut dying: BTreeMap<u32, Vec<(i64, Value)>> = BTreeMap::new();
    let slices: Vec<_> = tape.of_kind(b"SLIC").cloned().collect();
    let slice_frames = tape.slice_frames.max(1);
    let last = tape.last_frame;
    for (k, chunk) in slices.iter().enumerate() {
        let index = tape.load_slice(chunk_bytes(file, chunk)?)?;
        if index as usize != k {
            return err(format!("slice {k} says it is slice {index}"));
        }
        let slice = &tape.slices[&index];
        for o in &slice.orders {
            if o.unit == 0 {
                continue;
            }
            let row = o.unit as usize - 1;
            let mut c = json!({ "t": "cmd", "f": o.frame, "u": units[row].engine_id, "team": team_at(row, o.frame), "c": o.cmd, "shift": o.shift as u8 });
            if o.params.len() >= 3 {
                c["x"] = json!(o.params[0]);
                c["z"] = json!(o.params[2]);
            } else if let Some(target) = o.params.first() {
                c["target"] = json!(target);
            }
            seq += 1;
            pending.insert((o.frame, 1, seq), c);
            written.commands += 1;
        }
        // The build progress of the units that die in this slice, from their last vitals.
        for value in pending.range_mut((slice.first, 0, 0)..(slice.first + slice_frames + 1, 0, 0)).map(|(_, v)| v) {
            // (a death on a slice's last frame is met again at the next slice's start: done once)
            let Some(row) = value.get("row").and_then(Value::as_u64).map(|r| r as u32) else { continue };
            let frame = value["f"].as_u64().unwrap_or(0) as u32;
            let build = slice.vitals.iter().filter(|(f, r, _)| *r == row && *f <= frame).last().map(|(_, _, v)| v.build)
                .or_else(|| slice.key.iter().find(|k| k.row == row).map(|k| k.vital.build))
                .unwrap_or(units[row as usize].birth_vital.build);
            value["progress"] = json!(build as i64 / 10);
            value.as_object_mut().unwrap().remove("row");
        }
        for (f, row, amount) in &slice.hurt {
            hurt.entry(f.div_ceil(SAMPLE) * SAMPLE).or_default().push((*row, *amount));
        }
        for (row, u) in units.iter().enumerate() {
            let (died, r) = (u.destroyed, row as u32);
            if died == NEVER || died <= slice.first || died > slice.first + slice_frames || died.div_ceil(SAMPLE) * SAMPLE > last {
                continue;
            }
            fn last_of<T: Copy>(list: &[(u32, u32, T)], row: u32, frame: u32) -> Option<T> {
                list.iter().filter(|(f, r, _)| *r == row && *f <= frame).last().map(|x| x.2)
            }
            let key = slice.key.iter().position(|k| k.row == r);
            let vital = last_of(&slice.vitals, r, died).or(key.map(|i| slice.key[i].vital)).unwrap_or(u.birth_vital);
            let a = last_of(&slice.activity, r, died).or(key.map(|i| slice.key[i].activity)).unwrap_or_default();
            let work = last_of(&slice.work, r, died).or(key.and_then(|i| slice.key_work.get(i).copied())).unwrap_or_default();
            let rank = last_of(&slice.rank, r, died).or(key.and_then(|i| slice.key_rank.get(i).copied())).unwrap_or([0.0, u.max_health]);
            let sight = last_of(&slice.sight, r, died).or(key.and_then(|i| slice.key_sight.get(i).copied())).unwrap_or([0, 0]);
            let team = team_at(row, died);
            let (x, z) = place(u.death);
            let (tx, tz, target) = match a.how {
                3 => (a.a as i64, a.c as i64, 0),
                1 => (0, 0, a.a as i64),
                _ => (0, 0, 0),
            };
            let seen = enemy.get(&team).map_or(0, |&enemy| ((sight[0] >> enemy) & 1) + 2 * ((sight[1] >> enemy) & 1)) as i64;
            dying.entry(died.div_ceil(SAMPLE) * SAMPLE).or_default().push((team, json!([
                u.engine_id, u.def, team, x, z, 0, vital.build as i64 / 10, a.queue, a.cmd, tx, tz, target, a.builds, work[0], work[1], seen, figure(rank[0] as f64)
            ])));
        }
        let mut frame = slice.first.div_ceil(SAMPLE) * SAMPLE;
        while frame < slice.first + slice_frames && frame <= last {
            // Everything given or done up to this frame, then its sample.
            let now: Vec<(u32, u8, u64)> = pending.range(..(frame + 1, 0, 0)).map(|(key, _)| *key).collect();
            for key in now {
                let row = pending.remove(&key).unwrap();
                put(out, &row)?;
            }
            let exact = tape.exact(index, frame).ok_or_else(|| crate::format::Error(format!("frame {frame}: its slice is not loaded")))?;
            let mut by_team: BTreeMap<i64, Vec<Value>> = BTreeMap::new();
            for e in &exact {
                let u = &units[e.row as usize];
                let team = team_at(e.row as usize, frame);
                let (x, z) = place(e.pose.pos);
                let full = e.rank[1] as f64;
                let health = if full > 0.0 { (e.vital.health as f64 / full * 100.0).floor() as i64 } else { 0 };
                let a = &e.activity;
                let (tx, tz, target) = match a.how {
                    3 => (a.a as i64, a.c as i64, 0),
                    1 => (0, 0, a.a as i64),
                    _ => (0, 0, 0),
                };
                let seen = enemy.get(&team).map_or(0, |&enemy| ((e.sight[0] >> enemy) & 1) + 2 * ((e.sight[1] >> enemy) & 1)) as i64;
                by_team.entry(team).or_default().push(json!([
                    u.engine_id, u.def, team, x, z, health, e.vital.build as i64 / 10, a.queue, a.cmd, tx, tz, target, a.builds, e.work[0], e.work[1], seen, figure(e.rank[0] as f64)
                ]));
            }
            for (team, row) in dying.remove(&frame).unwrap_or_default() {
                by_team.entry(team).or_default().push(row);
            }
            let mut rows = Vec::new();
            let mut sides = Vec::new();
            for team in &teams {
                sides.push(side(*team, frame));
                rows.extend(by_team.remove(team).unwrap_or_default());
            }
            written.unit_rows += rows.len() as u64;
            // (one entry a unit, as the widget sums them: a unit that died on this frame has its death's row and the
            // sample's)
            let mut summed: BTreeMap<u32, f64> = BTreeMap::new();
            for (row, amount) in hurt.remove(&frame).unwrap_or_default() {
                *summed.entry(row).or_default() += amount as f64;
            }
            let hits: Vec<Value> = summed.iter().map(|(row, amount)| json!([units[*row as usize].engine_id, amount.round() as i64])).collect();
            put(out, &json!({ "t": "s", "f": frame, "sides": sides, "u": rows, "hits": hits }))?;
            written.samples += 1;
            if frame % WRECKS == 0 {
                put(out, &wrecks(frame))?;
            }
            frame += SAMPLE;
        }
        tape.drop_slice(index);
    }
    for (_, mut row) in std::mem::take(&mut pending) {
        if let Some(fields) = row.as_object_mut() {
            fields.remove("row");
        }
        put(out, &row)?;
    }
    for e in events.as_array().into_iter().flatten().filter(|e| e.get("t").and_then(Value::as_str) == Some("over")) {
        put(out, &json!({ "t": "over", "f": num(e, "f") as i64, "winners": e.get("winners").cloned().unwrap_or(json!([])) }))?;
    }
    Ok(written)
}
