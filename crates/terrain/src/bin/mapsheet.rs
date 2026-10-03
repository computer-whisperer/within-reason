//! The map sheet: what a map is, in the numbers the decisions turn on, the same for every map and computed from the
//! ground a recorded game saw (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3a).
//!
//!     mapsheet run/matches/<batch>/<NN> [--json]
//!
//! Reads the record's header (the map, the spots, the unit definitions), `terrain-<ai>.bin` beside it, our start
//! (our first unit in the first sample) and the opponent's (the truth file's first row).

use std::path::Path;

use bot_protocol::{MoveClass, MoveKind, Terrain, Vec3};
use serde_json::{Value, json};
use terrain::{Field, costs, passable, passages};

/// The ways of moving the sheet prices, each with the units whose speeds turn its distances into seconds.
const CLASSES: &[(&str, &[&str])] = &[
    ("bots", &["armpw", "armck", "armrock"]),
    ("vehicles", &["armfav", "armflash", "armcv", "armstump"]),
    ("commander", &["armcom"]),
    ("all-terrain bots", &["armscab"]),
    ("amphibious tanks", &["armcroc"]),
    ("hovercraft", &["armsh", "armch"]),
    ("ships", &["armpt", "armcs"]),
];
/// Who takes spots, for the spots within so many seconds of a start.
const TAKERS: &[&str] = &["armcom", "armck", "armcv"];
const WITHIN: &[f32] = &[30.0, 60.0, 90.0, 120.0];
/// Two walks this close to each other (the longer over the shorter) make a spot neither side's.
const CONTESTED: f32 = 1.25;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(dir) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: mapsheet run/matches/<batch>/<NN> [--json]");
        std::process::exit(2);
    };
    let sheet = sheet(Path::new(dir)).unwrap_or_else(|e| {
        eprintln!("mapsheet: {e}");
        std::process::exit(1);
    });
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&sheet).unwrap());
    } else {
        print!("{}", markdown(&sheet));
    }
}

fn first_line(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(text.lines().next().unwrap_or("")).map_err(|e| format!("{}: {e}", path.display()))
}

fn sheet(dir: &Path) -> Result<Value, String> {
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
        metal: Vec::new(),
    };
    let map = &header["map"];
    let (map_w, map_h) = (map["width"].as_f64().unwrap_or(0.0) as f32, map["height"].as_f64().unwrap_or(0.0) as f32);
    let (columns, rows) = (header["grid"]["columns"].as_f64().unwrap_or(8.0) as f32, header["grid"]["rows"].as_f64().unwrap_or(8.0) as f32);
    let cell_name = |p: Vec3| {
        let c = ((p.x / map_w * columns) as u8).min(columns as u8 - 1);
        let r = ((p.z / map_h * rows) as u32).min(rows as u32 - 1);
        format!("{}{}", (b'A' + c) as char, r + 1)
    };
    let at = |x: f64, z: f64| Vec3 { x: x as f32, y: 0.0, z: z as f32 };
    let defs = header["unit_defs"].as_array().ok_or("no unit definitions")?;
    let def = |name: &str| defs.iter().find(|d| d["name"] == name);
    let class_of = |d: &Value| -> Option<MoveClass> {
        let m = d["move"].as_array()?;
        let kind = match m[0].as_str()? {
            "tank" => MoveKind::Tank,
            "bot" => MoveKind::Bot,
            "hover" => MoveKind::Hover,
            _ => MoveKind::Ship,
        };
        Some(MoveClass { kind, max_slope: m[1].as_f64()? as f32, depth: m[2].as_f64()? as f32, slope_mod: m.get(3).and_then(Value::as_f64).unwrap_or(0.0) as f32 })
    };
    let ours = lines
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|r| r["t"] == "s" && r["own"].as_array().is_some_and(|o| !o.is_empty()))
        .and_then(|r| Some(at(r["own"][0][2].as_f64()?, r["own"][0][3].as_f64()?)))
        .ok_or("no unit of ours in the record")?;
    let theirs = first_line(&dir.join(format!("truth-{ai}.jsonl"))).ok().and_then(|t| Some(at(t["enemy"][0][2].as_f64()?, t["enemy"][0][3].as_f64()?)));
    let mut starts = vec![("ours", ours)];
    starts.extend(theirs.map(|p| ("the opponent's", p)));
    let spots: Vec<(Vec3, f32)> = header["metal_spots"]
        .as_array()
        .map(|s| s.iter().filter_map(|s| Some((at(s[0].as_f64()?, s[1].as_f64()?), s.get(2).and_then(Value::as_f64).unwrap_or(0.0) as f32))).collect())
        .unwrap_or_default();
    let rate = def("armmex").and_then(|d| d["extracts_metal"].as_f64()).unwrap_or(0.0) as f32;
    let mut values: Vec<f32> = spots.iter().map(|(_, amount)| amount * rate).collect();
    values.sort_by(f32::total_cmp);
    let water = ground.heights.iter().filter(|h| **h < 0).count() as f32 / cells as f32;
    let (low, high) = (ground.heights.iter().min().copied().unwrap_or(0), ground.heights.iter().max().copied().unwrap_or(0));

    let field = |class: MoveClass, from: Vec3| Field::from_costs(&ground, &costs(&ground, class), &[from]);
    let mut classes = Vec::new();
    for (label, units) in CLASSES {
        let Some((class, _)) = units.iter().find_map(|u| def(u).and_then(|d| Some((class_of(d)?, d)))) else { continue };
        let open = passable(&ground, class).iter().filter(|ok| **ok).count() as f32 / cells as f32;
        let from: Vec<Option<Field>> = starts.iter().map(|(_, p)| field(class, *p)).collect();
        let walk = match (&from[0], starts.get(1)) {
            (Some(f), Some((_, p))) => f.distance(*p),
            _ => None,
        };
        let seconds: serde_json::Map<String, Value> = units
            .iter()
            .filter_map(|u| {
                let speed = def(u)?["speed"].as_f64()? as f32;
                Some((u.to_string(), json!(walk.filter(|_| speed > 0.0).map(|w| (w / speed).round() as i32))))
            })
            .collect();
        let reach: Vec<Value> = from.iter().map(|f| json!(f.as_ref().map(|f| spots.iter().filter(|(s, _)| f.distance(*s).is_some()).count()))).collect();
        let narrow: Vec<Value> = match (&from[0], from.get(1).and_then(Option::as_ref)) {
            (Some(a), Some(b)) if walk.is_some() => passages(a, b).iter().take(4).map(|p| json!({ "at": cell_name(p.at), "width": p.width as i32, "share_of_the_way": (p.along * 100.0) as i32 })).collect(),
            _ => Vec::new(),
        };
        let mut split = [0usize; 4];
        if let (Some(a), Some(b)) = (&from[0], from.get(1).and_then(Option::as_ref)) {
            for (s, _) in &spots {
                let slot = match (a.distance(*s), b.distance(*s)) {
                    (Some(x), Some(y)) if x.max(y) <= x.min(y) * CONTESTED => 2,
                    (Some(x), Some(y)) if x < y => 0,
                    (Some(_), None) => 0,
                    (None, None) => 3,
                    _ => 1,
                };
                split[slot] += 1;
            }
        }
        classes.push(json!({
            "class": label, "max_slope": class.max_slope, "wades": class.depth,
            "ground_it_can_stand_on": format!("{:.0}%", open * 100.0),
            "start_to_start_walk": walk.map(|w| w as i32),
            "start_to_start_seconds": seconds,
            "spots_reachable_from_each_start": reach,
            "spots_nearer_ours_nearer_his_contested_unreachable": split,
            "passages": narrow,
        }));
    }
    let mut expansion = Vec::new();
    for (who, start) in &starts {
        for taker in TAKERS {
            let Some(d) = def(taker) else { continue };
            let (Some(class), Some(speed)) = (class_of(d), d["speed"].as_f64()) else { continue };
            let Some(f) = field(class, *start) else { continue };
            let mut walks: Vec<f32> = spots.iter().filter_map(|(s, _)| f.distance(*s)).map(|w| w / speed as f32).collect();
            walks.sort_by(f32::total_cmp);
            let counts: Vec<usize> = WITHIN.iter().map(|t| walks.iter().filter(|w| *w <= t).count()).collect();
            let nth = [4, 9, 14].map(|n| walks.get(n).map(|w| w.round() as i32));
            expansion.push(json!({ "start": who, "taker": taker, "speed": speed, "spots_within_seconds": counts, "seconds_to_the_5th_10th_15th": nth }));
        }
    }
    let straight = starts.get(1).map(|(_, p)| (((p.x - ours.x).powi(2) + (p.z - ours.z).powi(2)).sqrt()) as i32);
    Ok(json!({
        "map": map["name"], "from": dir.display().to_string(),
        "size": [map_w as i32, map_h as i32],
        "wind": [map["wind_min"], map["wind_max"]], "tidal": map["tidal"],
        "under_water": format!("{:.0}%", water * 100.0), "ground_height": [low, high],
        "spots": { "count": spots.len(), "metal_a_second_each_low_median_high": [values.first(), values.get(values.len() / 2), values.last()] },
        "starts": starts.iter().map(|(who, p)| json!({ "whose": who, "x": p.x as i32, "z": p.z as i32, "cell": cell_name(*p) })).collect::<Vec<_>>(),
        "start_to_start_straight": straight,
        "seconds_within": WITHIN,
        "classes": classes,
        "expansion": expansion,
    }))
}

fn markdown(s: &Value) -> String {
    let text = |v: &Value| match v {
        Value::Null => "-".to_string(),
        Value::String(t) => t.clone(),
        Value::Number(n) => n.as_f64().map_or(n.to_string(), |f| if f.fract() == 0.0 { format!("{f:.0}") } else { format!("{f:.2}") }),
        other => other.to_string(),
    };
    let list = |v: &Value, sep: &str| v.as_array().map(|a| a.iter().map(text).collect::<Vec<_>>().join(sep)).unwrap_or_default();
    let mut out = format!("### {}\n\n", text(&s["map"]));
    out += &format!(
        "- {} elmos; wind {}; tidal {}; {} under water; ground height {}.\n- {} metal spots, each {} metal a second (lowest / median / highest).\n- Starts: {}; {} apart in a straight line.\n\n",
        list(&s["size"], " by "), list(&s["wind"], " to "), text(&s["tidal"]), text(&s["under_water"]), list(&s["ground_height"], " to "),
        text(&s["spots"]["count"]), list(&s["spots"]["metal_a_second_each_low_median_high"], " / "),
        s["starts"].as_array().map(|a| a.iter().map(|p| format!("{} {} ({}, {})", text(&p["whose"]), text(&p["cell"]), text(&p["x"]), text(&p["z"]))).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
        text(&s["start_to_start_straight"]),
    );
    out += "| Moves as | Ground it can stand on | Start to start, walked | Seconds for | Spots reachable (ours / his start) | Spots nearer us / nearer him / contested / unreachable | Passages (cell, width, % of the way) |\n|---|---|---|---|---|---|---|\n";
    for c in s["classes"].as_array().into_iter().flatten() {
        let seconds = c["start_to_start_seconds"].as_object().map(|o| o.iter().map(|(u, t)| format!("{u} {}", text(t))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
        let narrow = c["passages"].as_array().map(|a| a.iter().map(|p| format!("{} {} {}%", text(&p["at"]), text(&p["width"]), text(&p["share_of_the_way"]))).collect::<Vec<_>>().join("; ")).filter(|t| !t.is_empty()).unwrap_or_else(|| "none".into());
        out += &format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            text(&c["class"]), text(&c["ground_it_can_stand_on"]), text(&c["start_to_start_walk"]), seconds,
            list(&c["spots_reachable_from_each_start"], " / "), list(&c["spots_nearer_ours_nearer_his_contested_unreachable"], " / "), narrow
        );
    }
    out += &format!("\n| Start | Taken by (speed) | Spots within {} s | Seconds to the 5th / 10th / 15th spot |\n|---|---|---|---|\n", list(&s["seconds_within"], " / "));
    for e in s["expansion"].as_array().into_iter().flatten() {
        out += &format!("| {} | {} ({}) | {} | {} |\n", text(&e["start"]), text(&e["taker"]), text(&e["speed"]), list(&e["spots_within_seconds"], " / "), list(&e["seconds_to_the_5th_10th_15th"], " / "));
    }
    out
}
