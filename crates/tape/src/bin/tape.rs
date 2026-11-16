//! The tape's tool.
//!
//!     tape pack <tape.raw> [-o game.tape] [--check] [--seats seats.json]
//!                                                      the raw stream of the capture widget into a tape; with
//!                                                      `--seats`, who sat where (run/replays/startscript.py's
//!                                                      `seats`: in a replay the engine names every seat UNKNOWN)
//!     tape info <game.tape>                            what a tape holds
//!     tape check <tape.raw> <game.tape> [--every N]    the tape read back against the raw stream it was made from
//!     tape audit <game.tape> [-o audit.jsonl]         the waste audit's dump (docs/harness/audit.md) from a tape,
//!                                                      as audit_dump.lua writes it in a replay; then
//!                                                      run/audit.py <the directory>
//!     tape grow <tape.raw> [--cuts N] [-o FILE]        the tape that grows while a game is played (pack::Growing),
//!                                                      checked: the stream cut at N places (inside records, as the
//!                                                      arena's follower meets it), the tape grown through each and
//!                                                      read again by one reader that keeps its slices, as the page does
//!     tape orders <game.tape>                          what the units stood under, a second at a time: by command,
//!                                                      how many unit-seconds and what the order pointed at
//!
//! `run/tape.py` runs the engine on a demo and then `tape pack`.

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use tape::game::Game;
use tape::read::{Exact, Frame, Tape, chunk_bytes};
use tape::slice::{quantized, quantized_burst};
use tape::{LINGER_FRAMES, SLICE_FRAMES, pack};

fn fail<T>(what: impl std::fmt::Display) -> T {
    eprintln!("tape: {what}");
    std::process::exit(1)
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| fail(format!("{path}: {e}")))
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1e6)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    match args.first().map(String::as_str) {
        Some("pack") if args.len() >= 2 => {
            let raw_path = &args[1];
            let out = value("-o").unwrap_or_else(|| std::path::Path::new(raw_path).with_file_name("game.tape").to_string_lossy().into_owned());
            let started = Instant::now();
            let raw = read(raw_path);
            let mut game = Game::ingest(&raw).unwrap_or_else(fail);
            if let Some(path) = value("--seats") {
                let seats: serde_json::Value = serde_json::from_slice(&read(&path)).unwrap_or_else(|e| fail(format!("{path}: {e}")));
                name_seats(&mut game, &seats);
            }
            let read_s = started.elapsed().as_secs_f32();
            let (file, report) = pack::pack(&game, true).unwrap_or_else(fail);
            std::fs::write(&out, &file).unwrap_or_else(|e| fail(format!("{out}: {e}")));
            println!(
                "{out}: {} from {} of raw stream; {} frames ({:.1} game minutes), {} unit lives, {} projectiles, {} features; read {:.1} s, packed {:.1} s",
                megabytes(file.len() as u64), megabytes(raw.len() as u64), game.last_frame, game.last_frame as f32 / 1800.0, game.units.len(), game.projectiles.len(),
                game.features.len(), read_s, started.elapsed().as_secs_f32() - read_s
            );
            for (kind, count, raw, stored) in &report.chunks {
                println!("  {kind} x{count}: {} stored, {} unpacked", megabytes(*stored), megabytes(*raw));
            }
            for (name, raw, stored) in &report.sections {
                println!("  slices' {name}: {} packed alone, {} unpacked", megabytes(*stored), megabytes(*raw));
            }
            if !game.unknown.is_empty() {
                println!("  passed over, records of kinds this reader does not know (a widget newer than this build): {:?}", game.unknown);
            }
            let mut kinds: Vec<_> = game.path_check.iter().collect();
            kinds.sort_by(|a, b| a.0.cmp(b.0));
            for (kind, (rows, worst)) in kinds {
                println!("  path check, {kind}: {rows} places, the formula's worst miss {worst:.2} elmos");
            }
            if flag("--check") {
                check(&game, &file, 1);
            }
        }
        Some("info") if args.len() >= 2 => {
            let file = read(&args[1]);
            let tape = Tape::open(&file).unwrap_or_else(fail);
            let head: serde_json::Value = serde_json::from_str(&tape.head).unwrap_or_default();
            println!("{}: {}", args[1], megabytes(file.len() as u64));
            println!("  {} on {}, engine {}", head["game"], head["map"]["name"], head["engine"]);
            println!("  {} frames ({:.1} game minutes) in {} slices of {} frames, a unit sample every {} frames", tape.last_frame, tape.last_frame as f32 / 1800.0, tape.slice_count(), tape.slice_frames, tape.sample_frames);
            println!("  {} unit lives, {} features, {} ground patches, ground {} x {} corners", tape.units.len(), tape.features.len(), tape.patches.len(), tape.ground.columns, tape.ground.rows);
            let mut by_kind: BTreeMap<String, (u32, u64)> = BTreeMap::new();
            for c in &tape.chunks {
                let e = by_kind.entry(String::from_utf8_lossy(&c.kind).to_string()).or_default();
                e.0 += 1;
                e.1 += c.len as u64;
            }
            for (kind, (count, bytes)) in by_kind {
                println!("  {kind} x{count}: {}", megabytes(bytes));
            }
        }
        Some("orders") if args.len() >= 2 => orders(&read(&args[1])),
        Some("audit") if args.len() >= 2 => {
            // `-o -`: to standard output, the account to standard error (run/audit.py reads a tape's audit so).
            let out = value("-o").unwrap_or_else(|| std::path::Path::new(&args[1]).with_file_name("audit.jsonl").to_string_lossy().into_owned());
            let started = Instant::now();
            let file = read(&args[1]);
            let sink: Box<dyn std::io::Write> = if out == "-" { Box::new(std::io::stdout().lock()) } else { Box::new(std::fs::File::create(&out).unwrap_or_else(|e| fail(format!("{out}: {e}")))) };
            let mut writer = std::io::BufWriter::new(sink);
            let written = tape::audit::write(&file, &mut writer).unwrap_or_else(fail);
            std::io::Write::flush(&mut writer).unwrap_or_else(|e| fail(format!("{out}: {e}")));
            let named = if out == "-" { format!("the audit of {}", args[1]) } else { out.clone() };
            eprintln!("{named}: {} samples, {} unit rows, {} commands, {} unit events ({:.1} s)", written.samples, written.unit_rows, written.commands, written.events, started.elapsed().as_secs_f32());
        }
        Some("grow") if args.len() >= 2 => {
            let raw = read(&args[1]);
            let cuts: usize = value("--cuts").and_then(|v| v.parse().ok()).unwrap_or(12).max(1);
            let out = value("-o").unwrap_or_else(|| format!("{}.growing", args[1]));
            let _ = std::fs::remove_file(&out);
            let started = Instant::now();
            let mut growing = pack::Growing::new(&out);
            let mut tape: Option<Tape> = None;
            for i in 1..=cuts {
                let end = raw.len() * i / cuts;
                let game = Game::ingest(&raw[..end]).unwrap_or_else(fail);
                let added = growing.grow(&game).unwrap_or_else(fail);
                if added == 0 {
                    continue;
                }
                let file = read(&out);
                match tape.as_mut() {
                    None => tape = Some(Tape::open(&file).unwrap_or_else(fail)),
                    Some(t) => t.refresh(&file).unwrap_or_else(fail),
                }
                let t = tape.as_mut().unwrap();
                let (slices, frames, units, _) = check_slices(t, &game, &file, 1, true);
                check_tables(t, &game);
                println!("cut {i}/{cuts} ({} of {}): {added} slices added, {slices} in all, {:.1} MB; {frames} frames and {units} unit states the same as the stream so far", megabytes(end as u64), megabytes(raw.len() as u64), file.len() as f64 / 1e6);
            }
            let whole = Game::ingest(&raw).unwrap_or_else(fail);
            let (packed, _) = pack::pack(&whole, false).unwrap_or_else(fail);
            println!("grow: {} slices grown in {:.1} s, the file {} where the tape packed whole is {}", growing.slices(), started.elapsed().as_secs_f32(), megabytes(std::fs::metadata(&out).map_or(0, |m| m.len())), megabytes(packed.len() as u64));
        }
        Some("check") if args.len() >= 3 => {
            let game = Game::ingest(&read(&args[1])).unwrap_or_else(fail);
            let every = value("--every").and_then(|v| v.parse().ok()).unwrap_or(1);
            check(&game, &read(&args[2]), every);
        }
        _ => fail::<()>("usage: tape pack <tape.raw> [-o game.tape] [--check] [--seats seats.json] | tape info <game.tape> | tape check <tape.raw> <game.tape> [--every N] | tape orders <game.tape>"),
    }
}

/// Gives the header's teams the start script's word on who plays them: a person's name, or an AI's short name and
/// version; the whole controller row goes in as `controller`.
fn name_seats(game: &mut Game, seats: &serde_json::Value) {
    let Some(teams) = game.head.get_mut("teams").and_then(|t| t.as_array_mut()) else { return };
    for seat in seats.as_array().into_iter().flatten() {
        let Some(team) = teams.iter_mut().find(|t| t["team"] == seat["team"]) else { continue };
        let controller = &seat["controller"];
        let name = match (controller["person"]["name"].as_str(), controller["ai"]["short_name"].as_str().or(controller["ai"]["name"].as_str())) {
            (Some(person), _) => person.to_string(),
            (None, Some(ai)) => format!("{ai} {}", controller["ai"]["version"].as_str().unwrap_or("")).trim().to_string(),
            _ => continue,
        };
        team["name"] = name.into();
        team["controller"] = controller.clone();
    }
}

/// The reader's account of the orders, once a game second: for each command the unit-seconds stood under it and what
/// it pointed at (`Frame::doing`), and the commands given (`Frame::commands` over the same frames).
fn orders(file: &[u8]) {
    let mut tape = Tape::open(file).unwrap_or_else(fail);
    let mut out = Frame::default();
    // command -> [nowhere, a place, a unit, a feature], and the units building under it
    let mut tally: BTreeMap<i32, ([u64; 4], u64)> = BTreeMap::new();
    // command -> places off the map (or far above or below it), and the first few of them
    let head: serde_json::Value = serde_json::from_str(&tape.head).unwrap_or_default();
    let (width, height) = (head["map"]["width"].as_f64().unwrap_or(0.0) as f32, head["map"]["height"].as_f64().unwrap_or(0.0) as f32);
    let mut off: BTreeMap<i32, (u64, Vec<String>)> = BTreeMap::new();
    let (mut seconds, mut alive, mut flashes, mut history) = (0u64, 0u64, 0u64, 0u64);
    let mut scratch = Vec::new();
    let slices: Vec<_> = tape.of_kind(b"SLIC").cloned().collect();
    for (k, chunk) in slices.iter().enumerate() {
        let index = tape.load_slice(chunk_bytes(file, chunk).unwrap_or_else(fail)).unwrap_or_else(fail);
        let first = k as u32 * SLICE_FRAMES;
        for f in (first..(first + SLICE_FRAMES).min(tape.last_frame + 1)).step_by(30) {
            if !tape.frame_in(index, f as f32, &mut out) {
                fail::<()>(format!("frame {f}: no state"));
            }
            seconds += 1;
            alive += (out.units.len() / tape::read::UNIT_STRIDE) as u64;
            flashes += (out.commands.len() / tape::read::COMMAND_STRIDE) as u64;
            for d in out.doing.chunks_exact(tape::read::DOING_STRIDE) {
                // a build's type is not the command's kind: all builds are one line
                let entry = tally.entry((d[2] as i32).max(-1)).or_default();
                entry.0[d[3] as usize] += 1;
                entry.1 += (d[8] >= 0.0) as u64;
                if d[3] == 1.0 && (d[4] < -64.0 || d[6] < -64.0 || d[4] > width + 64.0 || d[6] > height + 64.0 || d[5].abs() > 5000.0) {
                    let entry = off.entry(d[2] as i32).or_default();
                    entry.0 += 1;
                    if entry.1.len() < 4 {
                        entry.1.push(format!("frame {f} row {} at ({:.0}, {:.0}, {:.0})", d[1], d[4], d[5], d[6]));
                    }
                }
            }
        }
        if let Some(row) = tape.slices[&index].orders.first().map(|o| o.unit).filter(|u| *u > 0) {
            tape.orders_of(row - 1, &mut scratch);
            history += (scratch.len() / tape::read::HISTORY_STRIDE) as u64;
        }
        // (the slice before stays while this one is read: its last orders still fade in this one's first frames)
        if let Some(before) = index.checked_sub(1) {
            tape.drop_slice(before);
        }
    }
    println!("{seconds} game seconds, {alive} unit-seconds; {flashes} commands in the fading lists; {history} history rows of one unit a slice");
    println!("{:>8} {:>10} {:>10} {:>10} {:>10} {:>10}", "command", "nowhere", "a place", "a unit", "a feature", "building");
    for (cmd, (how, builds)) in tally {
        let name = if cmd < 0 { "build".to_string() } else { cmd.to_string() };
        println!("{name:>8} {:>10} {:>10} {:>10} {:>10} {builds:>10}", how[0], how[1], how[2], how[3]);
    }
    for (cmd, (n, first)) in off {
        println!("command {cmd}: {n} unit-seconds at a place off the map ({width} x {height}), e.g. {}", first.join("; "));
    }
}

/// The tape read back against the game it was packed from: at every `every`th sample frame every unit alive, its
/// place, attitude, health, build progress, order, sight, build power and experience exactly; per slice its orders,
/// projectiles, bursts, damage taken and moving pieces; and the flat arrays of `Tape::frame` for the same units. Exits
/// 1 on the first slice with a difference.
fn check(game: &Game, file: &[u8], every: u32) {
    let started = Instant::now();
    let mut tape = Tape::open(file).unwrap_or_else(fail);
    let (slices, frames, units, seek_worst) = check_slices(&mut tape, game, file, every, false);
    check_tables(&tape, game);
    println!(
        "check: {slices} slices, {frames} frames, {units} unit states, every order, projectile and burst, the tables and the ground the same as the raw stream ({:.1} s; the slowest slice to load {:.0} ms)",
        started.elapsed().as_secs_f32(), seek_worst * 1000.0
    );
}

/// Every slice of the tape's index against the game: its orders, projectiles and bursts, and every unit at every
/// `every`th sample. A slice not loaded is loaded from `file`; with `keep` it stays (the growing tape's check, where
/// one reader keeps its slices across refreshes as the page does), else it is dropped after. A projectile's end past
/// its slice is compared as the slice's end and a frame, and its samples up to the slice's end: a growing tape's slice
/// holds the end and the samples it had when written.
/// Answers the slices, frames and unit states compared and the slowest load in seconds.
fn check_slices(tape: &mut Tape, game: &Game, file: &[u8], every: u32, keep: bool) -> (usize, u64, u64, f32) {
    let sample = game.sample_frames();
    let mut frame_out = Frame::default();
    let (mut frames, mut units, mut seek_worst) = (0u64, 0u64, 0f32);
    let mut pieces = 0u64;
    let rests = game.piece_rest();
    // Each unit's pieces with keyframes: the piece, where its keyframes lie in the unit's, and a cursor, the number of
    // them at or before the frame checked (the frames only go forward, so it only moves on).
    let mut runs: Vec<Vec<(u16, usize, usize, usize)>> = game
        .units
        .iter()
        .map(|u| {
            let mut run = Vec::new();
            let mut i = 0;
            while i < u.keys.len() {
                let piece = u.keys[i].piece;
                let j = i + u.keys[i..].partition_point(|k| k.piece == piece);
                run.push((piece, i, j, 0));
                i = j;
            }
            run
        })
        .collect();
    let slices: Vec<_> = tape.of_kind(b"SLIC").cloned().collect();
    for (k, chunk) in slices.iter().enumerate() {
        let index = if keep && tape.slices.contains_key(&(k as u32)) {
            k as u32
        } else {
            let t = Instant::now();
            let index = tape.load_slice(chunk_bytes(file, chunk).unwrap_or_else(fail)).unwrap_or_else(fail);
            seek_worst = seek_worst.max(t.elapsed().as_secs_f32());
            index
        };
        if index != k as u32 {
            fail::<()>(format!("slice {k} says it is slice {index}"));
        }
        let first = k as u32 * SLICE_FRAMES;
        let last = (first + SLICE_FRAMES).min(game.last_frame);
        let slice = &tape.slices[&index];
        let tracks: HashMap<(u32, u16), &tape::slice::PieceTrack> = slice.pieces.iter().map(|t| ((t.row, t.piece), t)).collect();
        let within = |f: u32| f > first && f <= first + SLICE_FRAMES;
        let orders: Vec<_> = game.orders.iter().filter(|o| within(o.frame)).cloned().collect();
        if orders != slice.orders {
            fail::<()>(format!("slice {k}: {} orders in the tape, {} in the game, or they differ", slice.orders.len(), orders.len()));
        }
        // (and its samples up to the slice's last frame, a sample frame: a growing tape's slice holds those it had
        // when written, and the slice is read no further)
        let past = |mut p: tape::game::Projectile| {
            p.end = p.end.min(first + SLICE_FRAMES + 1);
            let n = p.samples.partition_point(|(f, _)| *f <= first + SLICE_FRAMES);
            p.samples.truncate(n);
            p
        };
        let projectiles: Vec<_> = game.projectiles.iter().filter(|p| p.begin <= first + SLICE_FRAMES && p.end + LINGER_FRAMES > first).map(|p| past(quantized(p))).collect();
        let held: Vec<_> = slice.projectiles.iter().cloned().map(past).collect();
        if projectiles != held {
            let at = projectiles.iter().zip(&held).position(|(a, b)| a != b);
            fail::<()>(format!("slice {k}: projectiles differ ({} in the game, {} in the tape; the first difference at {:?}: {:?} against {:?})", projectiles.len(), held.len(), at, at.map(|i| &projectiles[i]), at.map(|i| &held[i])));
        }
        let mut hurt: Vec<(u32, u32, f32)> = game.units.iter().enumerate().flat_map(|(row, u)| u.hurt.iter().filter(|(f, _)| within(*f)).map(move |(f, h)| (*f, row as u32, *h))).collect();
        hurt.sort_by_key(|h| (h.0, h.1));
        if hurt != slice.hurt {
            fail::<()>(format!("slice {k}: the damage taken differs ({} rows in the game, {} in the tape)", hurt.len(), slice.hurt.len()));
        }
        let bursts: Vec<_> = game.bursts.iter().filter(|b| b.frame + LINGER_FRAMES > first && b.frame <= first + SLICE_FRAMES).map(quantized_burst).collect();
        if bursts != slice.bursts {
            fail::<()>(format!("slice {k}: bursts differ"));
        }
        let mut f = first.div_ceil(sample) * sample;
        while f <= last {
            let mut want: Vec<Exact> = game.units.iter().enumerate().take_while(|(_, u)| u.created <= f).filter(|(_, u)| u.alive(f))
                .map(|(row, u)| Exact {
                    row: row as u32, pose: u.pose(f), vital: u.vital(f), activity: u.doing(f), sight: u.sight(f, game.ally(u.team)), work: u.work_at(f), rank: u.rank_at(f),
                })
                .collect();
            let mut got = tape.exact(index, f).unwrap_or_else(|| fail(format!("frame {f}: its slice is not loaded")));
            want.sort_by_key(|e| e.row);
            got.sort_by_key(|e| e.row);
            if want != got {
                let at = want.iter().zip(&got).position(|(a, b)| a != b);
                fail::<()>(format!("frame {f}: {} units in the game, {} in the tape; the first difference: {:?} against {:?}", want.len(), got.len(), at.map(|i| &want[i]), at.map(|i| &got[i])));
            }
            // The flat arrays: the same units, each where its exact pose says.
            if !tape.frame_in(index, f as f32, &mut frame_out) {
                fail::<()>(format!("frame {f}: no state"));
            }
            if frame_out.units.len() != want.len() * tape::read::UNIT_STRIDE {
                fail::<()>(format!("frame {f}: the flat arrays hold {} units, the game {}", frame_out.units.len() / tape::read::UNIT_STRIDE, want.len()));
            }
            for row in frame_out.units.chunks_exact(tape::read::UNIT_STRIDE) {
                let e = &want[want.binary_search_by_key(&(row[13] as u32), |e| e.row).unwrap_or_else(|_| fail(format!("frame {f}: the arrays name row {} which is not alive", row[13])))];
                for i in 0..3 {
                    if (row[i] - e.pose.pos[i] as f32 / game.position_scale()).abs() > 0.01 {
                        fail::<()>(format!("frame {f}: row {} stands at {:?} in the arrays, {:?} eighths in the game", e.row, &row[..3], e.pose.pos));
                    }
                }
            }
            // The moving pieces: each piece with keyframes as its track in the slice says, or at rest without one.
            if game.has_pieces() {
                for e in &want {
                    let u = &game.units[e.row as usize];
                    let rest = rests.get(&u.def);
                    for (piece, i, j, n) in runs[e.row as usize].iter_mut() {
                        let (piece, keys) = (*piece, &u.keys[*i..*j]);
                        while *n < keys.len() && keys[*n].frame <= f {
                            *n += 1;
                        }
                        let r = rest.and_then(|r| r.get(piece as usize - 1)).copied().unwrap_or([0.0; 3]);
                        let (a, pl) = tape::pieces::state_past(keys, *n, f as f64, r);
                        match tracks.get(&(e.row, piece)) {
                            Some(track) => {
                                let (ta, tp) = tape::pieces::track_at(&track.points, f as f64);
                                if (0..3).any(|k| (a[k] - ta[k]).abs() > 1.5 || (pl[k] - tp[k]).abs() > 1.5) {
                                    let near: Vec<_> = u.keys.iter().filter(|k| k.piece == piece && k.frame + 400 > f && k.frame < f + 400).map(|k| (k.frame, k.angles)).collect();
                                    fail::<()>(format!("frame {f}: row {} piece {piece} is {a:?} {pl:?} in the game, {ta:?} {tp:?} in the tape; its keyframes near: {near:?}; the track: {:?}", e.row, track.points.iter().map(|p| (p.0, p.1)).collect::<Vec<_>>()));
                                }
                            }
                            None if !tape::pieces::is_rest(a.map(f64::round), pl.map(f64::round), r) => fail::<()>(format!("frame {f}: row {} piece {piece} moves in the game ({a:?} {pl:?}) and has no track in slice {k}", e.row)),
                            None => {}
                        }
                        pieces += 1;
                    }
                }
            }
            let flying = game.projectiles.iter().filter(|p| p.begin <= f && f < p.end).count();
            if flying != frame_out.projectiles.len() / tape::read::PROJECTILE_STRIDE {
                fail::<()>(format!("frame {f}: {flying} projectiles in flight in the game, {} in the arrays", frame_out.projectiles.len() / tape::read::PROJECTILE_STRIDE));
            }
            frames += 1;
            units += want.len() as u64;
            f += sample * every;
        }
        if !keep {
            tape.drop_slice(index);
        }
    }
    if pieces > 0 {
        println!("  pieces: {pieces} piece states the same as the stream's keyframes");
    }
    (slices.len(), frames, units, seek_worst)
}

/// The tape's tables against the game's: the unit rows, the features, the ground and its patches.
fn check_tables(tape: &Tape, game: &Game) {
    if tape.units.len() != game.units.len() {
        fail::<()>(format!("the tape has {} unit lives, the game {}", tape.units.len(), game.units.len()));
    }
    for (row, (u, t)) in game.units.iter().zip(&tape.units).enumerate() {
        if tape::tables::UnitRow::of(u) != *t {
            fail::<()>(format!("unit row {row} differs: {:?} against {:?}", tape::tables::UnitRow::of(u), t));
        }
    }
    if game.features != tape.features {
        fail::<()>("the feature tables differ");
    }
    if game.feature_metal != tape.feature_metal {
        fail::<()>(format!("the wrecks' metal differs: {} changes in the game, {} in the tape", game.feature_metal.len(), tape.feature_metal.len()));
    }
    let step = tape.ground.step;
    let worst = game.ground.2.iter().zip(&tape.ground.heights).map(|(h, q)| (h - tape.ground.height(*q)).abs()).fold(0.0f32, f32::max);
    if game.ground.2.len() != tape.ground.heights.len() || worst > step {
        fail::<()>(format!("the ground differs: {} corners against {}, the worst height off by {worst} (a step is {step})", game.ground.2.len(), tape.ground.heights.len()));
    }
    if game.patches.len() != tape.patches.len() {
        fail::<()>("the ground patches differ in number");
    }
}
