//! The tape's tool.
//!
//!     tape pack <tape.raw> [-o game.tape] [--check]    the raw stream of the capture widget into a tape
//!     tape info <game.tape>                            what a tape holds
//!     tape check <tape.raw> <game.tape> [--every N]    the tape read back against the raw stream it was made from
//!
//! `run/tape.py` runs the engine on a demo and then `tape pack`.

use std::collections::BTreeMap;
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
            let game = Game::ingest(&raw).unwrap_or_else(fail);
            let read_s = started.elapsed().as_secs_f32();
            let (file, report) = pack::pack(&game).unwrap_or_else(fail);
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
        Some("check") if args.len() >= 3 => {
            let game = Game::ingest(&read(&args[1])).unwrap_or_else(fail);
            let every = value("--every").and_then(|v| v.parse().ok()).unwrap_or(1);
            check(&game, &read(&args[2]), every);
        }
        _ => fail::<()>("usage: tape pack <tape.raw> [-o game.tape] [--check] | tape info <game.tape> | tape check <tape.raw> <game.tape> [--every N]"),
    }
}

/// The tape read back against the game it was packed from: at every `every`th sample frame every unit alive, its
/// place, attitude, health, build progress, order and aim exactly; per slice its orders, projectiles and bursts; and
/// the flat arrays of `Tape::frame` for the same units. Exits 1 on the first slice with a difference.
fn check(game: &Game, file: &[u8], every: u32) {
    let started = Instant::now();
    let mut tape = Tape::open(file).unwrap_or_else(fail);
    let sample = game.sample_frames();
    let mut frame_out = Frame::default();
    let (mut frames, mut units, mut seek_worst) = (0u64, 0u64, 0f32);
    let slices: Vec<_> = tape.of_kind(b"SLIC").cloned().collect();
    if tape.units.len() != game.units.len() {
        fail::<()>(format!("the tape has {} unit lives, the game {}", tape.units.len(), game.units.len()));
    }
    for (k, chunk) in slices.iter().enumerate() {
        let t = Instant::now();
        let index = tape.load_slice(chunk_bytes(file, chunk).unwrap_or_else(fail)).unwrap_or_else(fail);
        seek_worst = seek_worst.max(t.elapsed().as_secs_f32());
        if index != k as u32 {
            fail::<()>(format!("slice {k} says it is slice {index}"));
        }
        let first = k as u32 * SLICE_FRAMES;
        let last = (first + SLICE_FRAMES).min(game.last_frame);
        let slice = &tape.slices[&index];
        let within = |f: u32| f > first && f <= first + SLICE_FRAMES;
        let orders: Vec<_> = game.orders.iter().filter(|o| within(o.frame)).cloned().collect();
        if orders != slice.orders {
            fail::<()>(format!("slice {k}: {} orders in the tape, {} in the game, or they differ", slice.orders.len(), orders.len()));
        }
        let projectiles: Vec<_> = game.projectiles.iter().filter(|p| p.begin <= first + SLICE_FRAMES && p.end + LINGER_FRAMES > first).map(quantized).collect();
        if projectiles != slice.projectiles {
            let at = projectiles.iter().zip(&slice.projectiles).position(|(a, b)| a != b);
            fail::<()>(format!("slice {k}: projectiles differ ({} in the game, {} in the tape; the first difference at {:?}: {:?} against {:?})", projectiles.len(), slice.projectiles.len(), at, at.map(|i| &projectiles[i]), at.map(|i| &slice.projectiles[i])));
        }
        let bursts: Vec<_> = game.bursts.iter().filter(|b| b.frame + LINGER_FRAMES > first && b.frame <= first + SLICE_FRAMES).map(quantized_burst).collect();
        if bursts != slice.bursts {
            fail::<()>(format!("slice {k}: bursts differ"));
        }
        let mut f = first.div_ceil(sample) * sample;
        while f <= last {
            let mut want: Vec<Exact> = game.units.iter().enumerate().take_while(|(_, u)| u.created <= f).filter(|(_, u)| u.alive(f))
                .map(|(row, u)| Exact { row: row as u32, pose: u.pose(f), vital: u.vital(f), activity: u.doing(f), aim: u.aims(f) })
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
            let flying = game.projectiles.iter().filter(|p| p.begin <= f && f < p.end).count();
            if flying != frame_out.projectiles.len() / tape::read::PROJECTILE_STRIDE {
                fail::<()>(format!("frame {f}: {flying} projectiles in flight in the game, {} in the arrays", frame_out.projectiles.len() / tape::read::PROJECTILE_STRIDE));
            }
            frames += 1;
            units += want.len() as u64;
            f += sample * every;
        }
        tape.drop_slice(index);
    }
    // The tables.
    for (row, (u, t)) in game.units.iter().zip(&tape.units).enumerate() {
        if tape::tables::UnitRow::of(u) != *t {
            fail::<()>(format!("unit row {row} differs: {:?} against {:?}", tape::tables::UnitRow::of(u), t));
        }
    }
    if game.features != tape.features {
        fail::<()>("the feature tables differ");
    }
    let step = tape.ground.step;
    let worst = game.ground.2.iter().zip(&tape.ground.heights).map(|(h, q)| (h - tape.ground.height(*q)).abs()).fold(0.0f32, f32::max);
    if game.ground.2.len() != tape.ground.heights.len() || worst > step {
        fail::<()>(format!("the ground differs: {} corners against {}, the worst height off by {worst} (a step is {step})", game.ground.2.len(), tape.ground.heights.len()));
    }
    if game.patches.len() != tape.patches.len() {
        fail::<()>("the ground patches differ in number");
    }
    println!(
        "check: {} slices, {frames} frames, {units} unit states, every order, projectile and burst, the tables and the ground the same as the raw stream ({:.1} s; the slowest slice to load {:.0} ms)",
        slices.len(), started.elapsed().as_secs_f32(), seek_worst * 1000.0
    );
}
