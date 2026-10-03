//! The map sheet of a recorded game's map (`terrain::sheet`).
//!
//!     mapsheet run/matches/<batch>/<NN> [--json] [--start LABEL:X,Z ...] [--max-metal M]
//!
//! Reads the record's header (the map, the spots, the unit definitions) and `terrain-<ai>.bin` beside it. The
//! starts are ours (our first unit in the first sample) and the opponent's (the truth file's first row) unless
//! `--start` gives them (the first is ours). A record from before the header's `metal_spots_amount` carries the
//! engine's figure for a spot (the sum round its own spot point, which can miss squares of the patch): with
//! `--max-metal` (the map's `maxMetal`, from its `mapinfo.lua`) the patch's worth is computed from the terrain
//! file's metal instead.

use std::path::Path;

use bot_protocol::{MoveClass, MoveKind, Terrain, Vec3};
use serde_json::Value;
use terrain::sheet::{Input, Mover, markdown, sheet};

/// How far from a spot a square of metal still belongs to its patch (the shim's reading of the metal map).
const PATCH_REACH: f32 = 200.0;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut dir = None;
    let (mut as_json, mut starts, mut max_metal) = (false, Vec::new(), None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => as_json = true,
            "--max-metal" => max_metal = it.next().and_then(|v| v.parse::<f32>().ok()),
            "--start" => {
                let parsed = it.next().and_then(|v| {
                    let (label, at) = v.split_once(':')?;
                    let (x, z) = at.split_once(',')?;
                    Some((label.to_string(), Vec3 { x: x.parse().ok()?, y: 0.0, z: z.parse().ok()? }))
                });
                match parsed {
                    Some(s) => starts.push(s),
                    None => usage(),
                }
            }
            other if !other.starts_with("--") => dir = Some(other.to_string()),
            _ => usage(),
        }
    }
    let Some(dir) = dir else { usage() };
    match run(Path::new(&dir), starts, max_metal) {
        Ok(sheet) if as_json => println!("{}", serde_json::to_string_pretty(&sheet).unwrap_or_default()),
        Ok(sheet) => print!("{}", markdown(&sheet)),
        Err(e) => {
            eprintln!("mapsheet: {e}");
            std::process::exit(1);
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: mapsheet run/matches/<batch>/<NN> [--json] [--start LABEL:X,Z ...] [--max-metal M]");
    std::process::exit(2);
}

fn run(dir: &Path, mut starts: Vec<(String, Vec3)>, max_metal: Option<f32>) -> Result<Value, String> {
    let ai = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| n.strip_prefix("record-")?.strip_suffix(".jsonl").map(str::to_string))
        .min()
        .ok_or("no record in the directory")?;
    let record = std::fs::read_to_string(dir.join(format!("record-{ai}.jsonl"))).map_err(|e| e.to_string())?;
    let mut lines = record.lines();
    let header: Value = serde_json::from_str(lines.next().unwrap_or("")).map_err(|e| e.to_string())?;
    let grid = &header["terrain"];
    let (width, height) = (grid["width"].as_u64().ok_or("no terrain in the header")? as usize, grid["height"].as_u64().unwrap_or(0) as usize);
    let bytes = std::fs::read(dir.join(grid["file"].as_str().unwrap_or(""))).map_err(|e| format!("the terrain file: {e}"))?;
    let cells = width * height;
    if bytes.len() < cells * 3 {
        return Err("the terrain file is shorter than the header says".into());
    }
    let ground = Terrain {
        cell: grid["cell"].as_f64().unwrap_or(16.0) as f32,
        width: width as u32,
        height: height as u32,
        heights: bytes[..cells * 2].chunks(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect(),
        slopes: bytes[cells * 2..cells * 3].to_vec(),
        metal: bytes.get(cells * 3..cells * 4).map(<[u8]>::to_vec).unwrap_or_default(),
    };
    let map = &header["map"];
    let number = |v: &Value| v.as_f64().unwrap_or(0.0) as f32;
    let at = |x: f64, z: f64| Vec3 { x: x as f32, y: 0.0, z: z as f32 };
    let defs = header["unit_defs"].as_array().ok_or("no unit definitions")?;
    let movers: Vec<Mover> = defs
        .iter()
        .filter_map(|d| {
            let m = d["move"].as_array()?;
            let kind = match m[0].as_str()? {
                "tank" => MoveKind::Tank,
                "bot" => MoveKind::Bot,
                "hover" => MoveKind::Hover,
                _ => MoveKind::Ship,
            };
            let class = MoveClass { kind, max_slope: m[1].as_f64()? as f32, depth: m[2].as_f64()? as f32, slope_mod: m.get(3).and_then(Value::as_f64).unwrap_or(0.0) as f32 };
            Some(Mover { name: d["name"].as_str()?.to_string(), speed: number(&d["speed"]), class })
        })
        .collect();
    if starts.is_empty() {
        let ours = lines
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|r| r["t"] == "s" && r["own"].as_array().is_some_and(|o| !o.is_empty()))
            .and_then(|r| Some(at(r["own"][0][2].as_f64()?, r["own"][0][3].as_f64()?)))
            .ok_or("no unit of ours in the record")?;
        starts.push(("ours".into(), ours));
        let truth = std::fs::read_to_string(dir.join(format!("truth-{ai}.jsonl"))).ok();
        let theirs = truth.as_deref().and_then(|t| serde_json::from_str::<Value>(t.lines().next()?).ok()).and_then(|t| Some(at(t["enemy"][0][2].as_f64()?, t["enemy"][0][3].as_f64()?)));
        starts.extend(theirs.map(|p| ("the opponent's".to_string(), p)));
    }
    let rate = defs.iter().find(|d| d["name"] == "armmex").map_or(0.0, |d| number(&d["extracts_metal"]));
    let worth = header["metal_spots_amount"] == "worth";
    let places: Vec<(Vec3, f32)> = header["metal_spots"].as_array().map(|s| s.iter().filter_map(|s| Some((at(s[0].as_f64()?, s[1].as_f64()?), s.get(2).map_or(0.0, number)))).collect()).unwrap_or_default();
    // The patches' raw metal from the terrain file, each square to its nearest spot within reach, as the shim reads it.
    let mut raw = vec![0f32; places.len()];
    for (index, value) in ground.metal.iter().enumerate().filter(|(_, v)| **v > 0) {
        let (x, z) = (((index % width) as f32 + 0.5) * ground.cell, ((index / width) as f32 + 0.5) * ground.cell);
        let nearest = places.iter().enumerate().map(|(i, (p, _))| (i, (p.x - x).hypot(p.z - z))).min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((i, d)) = nearest
            && d < PATCH_REACH
        {
            raw[i] += f32::from(*value);
        }
    }
    let spots = places
        .iter()
        .zip(&raw)
        .map(|((p, amount), raw)| (*p, if worth { Some(amount * rate) } else { max_metal.filter(|_| *raw > 0.0).map(|m| raw * m * rate).or(Some(amount * rate)) }))
        .collect();
    let input = Input {
        name: map["name"].as_str().unwrap_or("?"),
        terrain: &ground,
        size: (number(&map["width"]), number(&map["height"])),
        grid: (header["grid"]["columns"].as_f64().unwrap_or(8.0) as f32, header["grid"]["rows"].as_f64().unwrap_or(8.0) as f32),
        wind: (number(&map["wind_min"]), number(&map["wind_max"])),
        tidal: number(&map["tidal"]),
        spots,
        movers,
        starts,
    };
    Ok(sheet(&input))
}
