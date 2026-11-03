//! The map sheet: what a map is, in the numbers the decisions turn on, the same for every map and computed from the
//! ground (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3a). The bot computes it at the start of a
//! game for the player's first report; `mapsheet` computes it from a recorded game.

use bot_protocol::{MoveClass, Terrain, Vec3};
use serde_json::{Value, json};

use crate::{Field, costs, passable};

/// The ways of moving the sheet prices, each with the units whose speeds turn its distances into seconds (the first
/// of them that the game has gives the class).
const CLASSES: &[(&str, &[&str])] = &[
    ("bots", &["armpw", "armck", "armrock", "corak", "corck"]),
    ("vehicles", &["armfav", "armflash", "armcv", "armstump", "corfav", "corgator", "corcv"]),
    ("commander", &["armcom", "corcom"]),
    ("all-terrain bots", &["armscab", "cortermite"]),
    ("amphibious tanks", &["armcroc", "corseal"]),
    ("hovercraft", &["armsh", "armch", "corsh", "corch"]),
    ("ships", &["armpt", "armcs", "corpt", "corcs"]),
];
/// Who takes spots, for the spots within so many seconds of a start.
const TAKERS: &[&str] = &["armcom", "armck", "armcv"];
const WITHIN: &[f32] = &[30.0, 60.0, 90.0, 120.0];
/// Two walks this close to each other (the longer over the shorter) make a spot neither side's.
const CONTESTED: f32 = 1.25;
/// The ways between two starts are every cell on a route at most this many times the shortest.
const DETOUR: f32 = 1.3;
/// Depth of a band of the ways, in flat cells; the ends of the way (this share at each) are the bases, not the way.
const BAND: u32 = 4;
const ENDS: f32 = 0.15;

/// How wide the ways between two starts are where they are tightest and at the median, in elmos, with the place
/// and the share of the way of the tightest: the ways' cells in bands by distance from the first start, a band's
/// width its cells over its depth. On open ground the ways are an ellipse a good part of the distance wide; a
/// pass shows as a tightest band a fraction of the median.
fn cross_section(from: &Field, to: &Field) -> Option<(f32, f32, Vec3, f32)> {
    let both = |i: usize| (from.cost[i] != u32::MAX && to.cost[i] != u32::MAX).then(|| from.cost[i] + to.cost[i]);
    let shortest = (0..from.cost.len()).filter_map(both).min()?;
    let limit = (shortest as f32 * DETOUR) as u32;
    let depth = BAND * 10;
    let mut bands: Vec<(u32, f64, f64)> = vec![(0, 0.0, 0.0); (shortest / depth) as usize + 1];
    for i in (0..from.cost.len()).filter(|i| both(*i).is_some_and(|sum| sum <= limit)) {
        if let Some(band) = bands.get_mut((from.cost[i] / depth) as usize) {
            let at = from.centre(i);
            *band = (band.0 + 1, band.1 + f64::from(at.x), band.2 + f64::from(at.z));
        }
    }
    let count = bands.len();
    let middle: Vec<(usize, &(u32, f64, f64))> = bands.iter().enumerate().filter(|(n, b)| b.0 > 0 && (*n as f32) >= count as f32 * ENDS && (*n as f32) <= count as f32 * (1.0 - ENDS)).collect();
    let width = |cells: u32| cells as f32 / BAND as f32 * from.cell;
    let (n, tightest) = middle.iter().min_by_key(|(_, b)| b.0)?;
    let mut widths: Vec<u32> = middle.iter().map(|(_, b)| b.0).collect();
    widths.sort_unstable();
    let at = Vec3 { x: (tightest.1 / f64::from(tightest.0)) as f32, y: 0.0, z: (tightest.2 / f64::from(tightest.0)) as f32 };
    Some((width(tightest.0), *n as f32 / count as f32, at, width(widths[widths.len() / 2])))
}

/// A unit the sheet turns distances into seconds with.
pub struct Mover {
    pub name: String,
    pub speed: f32,
    pub class: MoveClass,
}

pub struct Input<'a> {
    pub name: &'a str,
    pub terrain: &'a Terrain,
    /// The map's size in elmos and the grid the cells are named on.
    pub size: (f32, f32),
    pub grid: crate::Grid,
    pub wind: (f32, f32),
    pub tidal: f32,
    /// Each spot and what a tier-1 extractor draws from it a second (`None`: not known for this record).
    pub spots: Vec<(Vec3, Option<f32>)>,
    /// A metal map (the metal-maps design §1.4): no spots; the fields the shim found and what a tier-1 extractor
    /// draws a second standing anywhere on the metal. The metal itself is read from the terrain.
    pub metal_map: bool,
    pub fields: &'a [bot_protocol::MetalField],
    pub extractor_worth: f32,
    pub movers: Vec<Mover>,
    /// The starts, ours first; every other is priced against the first.
    pub starts: Vec<(String, Vec3)>,
}

/// What a Comet Catcher spot is worth to a tier-1 extractor, the unit the metal maps' worth is said in.
const COMET_SPOT: f32 = 2.3;
/// Cells whose metal shares are all within this ratio of each other hold the metal "evenly".
const EVEN: f32 = 1.5;
/// A cell with less than this share of the map's metal is left off the by-cell list.
const CELL_SHARE_FLOOR: f32 = 0.005;

/// Each cell's share of the map's metal, by the squares' values, indexed as the grid indexes its cells; empty when
/// the terrain carries no metal.
pub fn metal_shares(terrain: &Terrain, grid: &crate::Grid) -> Vec<f32> {
    let width = terrain.width as usize;
    let mut by_cell = vec![0f64; grid.count()];
    for (i, v) in terrain.metal.iter().enumerate().filter(|(_, v)| **v > 0) {
        let at = Vec3 { x: ((i % width) as f32 + 0.5) * terrain.cell, y: 0.0, z: ((i / width) as f32 + 0.5) * terrain.cell };
        let (column, row) = grid.cell_of(at);
        by_cell[grid.index(column, row)] += f64::from(*v);
    }
    let total: f64 = by_cell.iter().sum();
    if total <= 0.0 {
        return Vec::new();
    }
    by_cell.iter().map(|v| (v / total) as f32).collect()
}

/// The cells with metal, named, with their shares, in the grid's order (`metal_shares` by name).
pub fn named_shares(shares: &[f32], grid: &crate::Grid) -> Vec<(String, f32)> {
    grid.cells().filter(|(c, r)| shares.get(grid.index(*c, *r)).is_some_and(|s| *s > 0.0)).map(|(c, r)| (crate::Grid::name_of(c, r), shares[grid.index(c, r)])).collect()
}

/// Each cell's share of the map's metal, cells with none left out, in the grid's order.
pub fn metal_by_cell(terrain: &Terrain, grid: &crate::Grid) -> Vec<(String, f32)> {
    named_shares(&metal_shares(terrain, grid), grid)
}

/// The by-cell shares in words: "evenly over every cell (0.7 % each)" when every cell has metal and the shares are
/// within `EVEN` of each other, else the cells with a share worth naming.
pub fn cell_share_words(shares: &[(String, f32)], cells: usize) -> String {
    let (low, high) = shares.iter().fold((f32::MAX, 0f32), |(lo, hi), (_, s)| (lo.min(*s), hi.max(*s)));
    if shares.len() == cells && high <= low * EVEN {
        return format!("evenly over every cell ({:.1}% each)", 100.0 / cells as f32);
    }
    let named: Vec<String> = shares.iter().filter(|(_, s)| *s >= CELL_SHARE_FLOOR).map(|(cell, s)| format!("{cell} {:.0}%", s * 100.0)).collect();
    format!("by cell: {}", named.join(", "))
}

pub fn sheet(input: &Input) -> Value {
    let ground = input.terrain;
    let cells = (ground.width * ground.height) as usize;
    let (map_w, map_h) = input.size;
    let cell_name = |p: Vec3| input.grid.name(p);
    let mover = |name: &str| input.movers.iter().find(|m| m.name == name);
    let starts = &input.starts;
    let spots = &input.spots;
    let mut values: Vec<f32> = spots.iter().filter_map(|(_, v)| *v).collect();
    values.sort_by(f32::total_cmp);
    // On a metal map the squares with metal are what the classes reach and the starts are priced to, each at its
    // value (the metal-maps design §1.4).
    let square_centre = |i: usize| Vec3 { x: ((i % ground.width as usize) as f32 + 0.5) * ground.cell, y: 0.0, z: ((i / ground.width as usize) as f32 + 0.5) * ground.cell };
    let metal: Vec<(Vec3, f32)> = if input.metal_map { ground.metal.iter().enumerate().filter(|(_, v)| **v > 0).map(|(i, v)| (square_centre(i), f32::from(*v))).collect() } else { Vec::new() };
    let metal_total: f32 = metal.iter().map(|(_, v)| v).sum::<f32>().max(1.0);
    let percent = |share: f32| format!("{:.0}%", share * 100.0);
    let water = ground.heights.iter().filter(|h| **h < 0).count() as f32 / cells.max(1) as f32;
    let (low, high) = (ground.heights.iter().min().copied().unwrap_or(0), ground.heights.iter().max().copied().unwrap_or(0));
    // A class that cannot stand at a start (a ship from a start on land) is priced from the nearest ground it
    // stands on, and the row says how far that is.
    let seed = |class: MoveClass, from: Vec3| -> Option<(Vec3, f32)> {
        let ok = passable(ground, class);
        let width = ground.width as usize;
        let centre = |i: usize| Vec3 { x: ((i % width) as f32 + 0.5) * ground.cell, y: 0.0, z: ((i / width) as f32 + 0.5) * ground.cell };
        (0..ok.len()).filter(|i| ok[*i]).map(|i| (centre(i), (centre(i).x - from.x).hypot(centre(i).z - from.z))).min_by(|a, b| a.1.total_cmp(&b.1))
    };
    let field = |class: MoveClass, from: Vec3| seed(class, from).and_then(|(at, _)| Field::from_costs(ground, &costs(ground, class), &[at]));

    let mut classes = Vec::new();
    // The spots some surface class (walking, driving, hovering, sailing) builds from some start: reachable ground
    // within about a constructor's build distance (110-150) of the spot, the field's own snap (96) from the spot or a
    // point 24 beside it. The rest are the air's alone (an air constructor's).
    let mut on_foot = vec![false; spots.len()];
    let around = |s: Vec3| [(0.0, 0.0), (24.0, 0.0), (-24.0, 0.0), (0.0, 24.0), (0.0, -24.0)].map(|(dx, dz)| Vec3 { x: s.x + dx, y: s.y, z: s.z + dz });
    for (label, units) in CLASSES {
        let Some(class) = units.iter().find_map(|u| mover(u)).map(|m| m.class) else { continue };
        let open = passable(ground, class).iter().filter(|ok| **ok).count() as f32 / cells.max(1) as f32;
        let from: Vec<Option<Field>> = starts.iter().map(|(_, p)| field(class, *p)).collect();
        let Some(home) = from.first().and_then(Option::as_ref) else {
            classes.push(json!({ "class": label, "ground_it_can_stand_on": format!("{:.0}%", open * 100.0), "from_our_start": "no ground on this map it can stand on" }));
            continue;
        };
        for f in from.iter().flatten() {
            for (i, (s, _)) in spots.iter().enumerate() {
                on_foot[i] |= around(*s).iter().any(|p| f.distance(*p).is_some());
            }
        }
        let off = starts.first().and_then(|(_, p)| seed(class, *p)).map(|(_, d)| d).filter(|d| *d > ground.cell * 6.0);
        let mut to = Vec::new();
        for ((whose, p), far) in starts.iter().zip(&from).skip(1) {
            let walk = home.distance(*p);
            let seconds: serde_json::Map<String, Value> =
                units.iter().filter_map(|u| mover(u)).filter(|m| m.speed > 0.0).map(|m| (m.name.clone(), json!(walk.map(|w| (w / m.speed).round() as i32)))).collect();
            let ways = far.as_ref().filter(|_| walk.is_some()).and_then(|far| cross_section(home, far));
            let slot_of = |s: Vec3, far: &Field| match (home.distance(s), far.distance(s)) {
                (Some(x), Some(y)) if x.max(y) <= x.min(y) * CONTESTED => 2,
                (Some(x), Some(y)) if x < y => 0,
                (Some(_), None) => 0,
                (None, None) => 3,
                _ => 1,
            };
            let mut split = [0usize; 4];
            let mut metal_split = [0f32; 4];
            if let Some(far) = far {
                for (s, _) in spots {
                    split[slot_of(*s, far)] += 1;
                }
                for (s, v) in &metal {
                    metal_split[slot_of(*s, far)] += v;
                }
            }
            let mut entry = json!({
                "to": whose, "walk": walk.map(|w| w as i32), "seconds": seconds,
                "the_ways_at_their_tightest": ways.map(|(width, along, at, median)| json!({
                    "width": width as i32, "at": cell_name(at), "share_of_the_way": (along * 100.0) as i32, "median_width_of_the_ways": median as i32,
                })),
            });
            if input.metal_map {
                entry["metal_nearer_us_nearer_him_contested_unreachable"] = json!(metal_split.map(|v| percent(v / metal_total)));
            } else {
                entry["spots_nearer_us_nearer_him_contested_unreachable"] = json!(split);
            }
            to.push(entry);
        }
        let mut entry = json!({
            "class": label, "max_slope": class.max_slope, "wades": class.depth,
            "ground_it_can_stand_on": format!("{:.0}%", open * 100.0),
            "nearest_ground_it_stands_on_from_our_start": off.map(|d| d as i32),
            "to_each_other_start": to,
        });
        if input.metal_map {
            entry["metal_it_reaches_from_our_start"] = json!(percent(metal.iter().filter(|(s, _)| home.distance(*s).is_some()).map(|(_, v)| v).sum::<f32>() / metal_total));
        } else {
            entry["spots_it_reaches_from_our_start"] = json!(spots.iter().filter(|(s, _)| home.distance(*s).is_some()).count());
        }
        classes.push(entry);
    }
    let mut expansion = Vec::new();
    for (who, start) in starts {
        for taker in TAKERS {
            let Some(m) = mover(taker).filter(|m| m.speed > 0.0) else { continue };
            let Some(f) = field(m.class, *start) else { continue };
            if input.metal_map {
                // The seconds to the nearest metal from the start: on the plate it is nothing, on a map whose starts
                // stand off the metal it is the opening's first walk.
                let nearest = metal.iter().filter_map(|(s, _)| f.distance(*s)).fold(None, |best: Option<f32>, w| Some(best.map_or(w, |b| b.min(w))));
                expansion.push(json!({ "start": who, "taker": taker, "speed": m.speed, "seconds_to_the_nearest_metal": nearest.map(|w| (w / m.speed).round() as i32) }));
                continue;
            }
            let mut walks: Vec<f32> = spots.iter().filter_map(|(s, _)| f.distance(*s)).map(|w| w / m.speed).collect();
            walks.sort_by(f32::total_cmp);
            let counts: Vec<usize> = WITHIN.iter().map(|t| walks.iter().filter(|w| *w <= t).count()).collect();
            let nth = [4, 9, 14].map(|n| walks.get(n).map(|w| w.round() as i32));
            expansion.push(json!({ "start": who, "taker": taker, "speed": m.speed, "spots_within_seconds": counts, "seconds_to_the_5th_10th_15th": nth }));
        }
    }
    let ours = starts.first().map(|(_, p)| *p);
    let metal_block = input.metal_map.then(|| {
        let shares = metal_by_cell(ground, &input.grid);
        json!({
            "extractor_worth": input.extractor_worth,
            "comet_spots_worth": (input.extractor_worth / COMET_SPOT * 10.0).round() / 10.0,
            "share_of_the_ground": percent(metal.len() as f32 / cells.max(1) as f32),
            "fields": input.fields.iter().map(|f| json!({
                "name": f.name, "cells": crate::Grid::words(input.grid.cells_touching(f.bounds[0], f.bounds[1], f.bounds[2], f.bounds[3])),
                "room": f.room, "filled_draws_a_second": (f.room as f32 * input.extractor_worth).round() as i32,
            })).collect::<Vec<_>>(),
            "by_cell": shares.iter().map(|(cell, share)| json!({ "cell": cell, "share": percent(*share) })).collect::<Vec<_>>(),
            "by_cell_words": cell_share_words(&shares, input.grid.count()),
        })
    });
    json!({
        "map": input.name,
        "size": [map_w as i32, map_h as i32],
        "wind": [input.wind.0, input.wind.1], "tidal": input.tidal,
        "under_water": format!("{:.0}%", water * 100.0), "ground_height": [low, high],
        "metal": metal_block,
        "spots": {
            "count": spots.len(), "metal_a_second_each_low_median_high": [values.first(), values.get(values.len() / 2), values.last()],
            "by_air_only": (spots.iter().zip(&on_foot)).filter(|(_, reached)| !**reached).map(|((s, _), _)| json!({ "cell": cell_name(*s), "x": s.x as i32, "z": s.z as i32 })).collect::<Vec<_>>(),
        },
        "starts": starts.iter().map(|(who, p)| json!({
            "whose": who, "x": p.x as i32, "z": p.z as i32, "cell": cell_name(*p),
            "straight_from_ours": ours.map(|o| ((p.x - o.x).hypot(p.z - o.z)) as i32),
        })).collect::<Vec<_>>(),
        "seconds_within": WITHIN,
        "the_ways_note": format!("the ways between two starts are every route at most {DETOUR} times the shortest; their width is given where it is tightest and at the median, the ends of the way left out: a pass is a tightest width a fraction of the median"),
        "classes": classes,
        "expansion": expansion,
    })
}

pub fn markdown(s: &Value) -> String {
    let text = |v: &Value| match v {
        Value::Null => "-".to_string(),
        Value::String(t) => t.clone(),
        Value::Number(n) => n.as_f64().map_or(n.to_string(), |f| if f.fract() == 0.0 { format!("{f:.0}") } else { format!("{f:.2}") }),
        other => other.to_string(),
    };
    let list = |v: &Value, sep: &str| v.as_array().map(|a| a.iter().map(text).collect::<Vec<_>>().join(sep)).unwrap_or_default();
    let metal_map = s["metal"].is_object();
    let metal_line = match &s["metal"] {
        m if m.is_object() => {
            let fields: Vec<String> = m["fields"].as_array().into_iter().flatten().map(|f| format!("{} ({}, room for about {} tier-1 extractors; filled, about {} a second)", text(&f["name"]), text(&f["cells"]), text(&f["room"]), text(&f["filled_draws_a_second"]))).collect();
            format!(
                "A metal map: no metal spots. An extractor stands anywhere on the metal and draws {} metal a second ({} Comet Catcher spots' worth); metal is not what is short here, energy and build power are. Metal covers {} of the ground in {} field{}: {}. Its share {}",
                text(&m["extractor_worth"]), text(&m["comet_spots_worth"]), text(&m["share_of_the_ground"]), fields.len(), if fields.len() == 1 { "" } else { "s" }, fields.join("; "), text(&m["by_cell_words"])
            )
        }
        _ => format!("{} metal spots; a tier-1 extractor draws {} metal a second (lowest / median / highest spot)", text(&s["spots"]["count"]), list(&s["spots"]["metal_a_second_each_low_median_high"], " / ")),
    };
    let mut out = format!("### {}\n\n", text(&s["map"]));
    out += &format!(
        "- {} elmos; wind {}; tidal {}; {} under water; ground height {}.\n- {metal_line}.\n- Starts: {}.\n\n",
        list(&s["size"], " by "), list(&s["wind"], " to "), text(&s["tidal"]), text(&s["under_water"]), list(&s["ground_height"], " to "),
        // By cell, not "ours" and "the opponent's": the brief's sheet is read from either start (the commander on Cape
        // Violet, 2026-10-18, read "ours A5" while its seat stood at H6).
        s["starts"]
            .as_array()
            .map(|a| {
                let names: Vec<String> = a.iter().map(|p| format!("{} ({}, {})", text(&p["cell"]), text(&p["x"]), text(&p["z"]))).collect();
                let apart = if a.len() == 2 { a.get(1).map(|p| format!(", {} apart", text(&p["straight_from_ours"]))).unwrap_or_default() } else { String::new() };
                let first = a.first().map(|p| text(&p["cell"])).unwrap_or_default();
                let others = if a.len() > 1 { ": from another start, read \"us\" and \"him\" the other way round" } else { "" };
                format!("{}{apart}; the table below is from {first}, which was ours in the game the sheet was made from{others}", names.join(", "))
            })
            .unwrap_or_default(),
    );
    if let Some(air) = s["spots"]["by_air_only"].as_array().filter(|a| !a.is_empty()) {
        let mut cells: Vec<String> = air.iter().map(|p| text(&p["cell"])).collect();
        cells.sort();
        cells.dedup();
        out += &format!("- {} of the spots are built by air alone: no unit that walks, drives, hovers or sails reaches within a constructor's build distance of them from a start (in {}).\n\n", air.len(), cells.join(", "));
    }
    let (reaches, whose, split_key) = if metal_map {
        ("Metal it reaches from our start", "Metal nearer us / nearer him / contested / unreachable", "metal_nearer_us_nearer_him_contested_unreachable")
    } else {
        ("Spots it reaches from our start", "Spots nearer us / nearer him / contested / unreachable", "spots_nearer_us_nearer_him_contested_unreachable")
    };
    let reach_key = if metal_map { "metal_it_reaches_from_our_start" } else { "spots_it_reaches_from_our_start" };
    out += &format!("| Moves as | Ground it can stand on | {reaches} | To | Walked | Seconds for | {whose} | The ways at their tightest |\n|---|---|---|---|---|---|---|---|\n");
    for c in s["classes"].as_array().into_iter().flatten() {
        let Some(to) = c["to_each_other_start"].as_array() else {
            out += &format!("| {} | {} | {} | | | | | |\n", text(&c["class"]), text(&c["ground_it_can_stand_on"]), text(&c["from_our_start"]));
            continue;
        };
        let ground = match &c["nearest_ground_it_stands_on_from_our_start"] {
            Value::Null => text(&c["ground_it_can_stand_on"]),
            d => format!("{} (the nearest {} from our start)", text(&c["ground_it_can_stand_on"]), text(d)),
        };
        // One start: the row without the walk to another.
        if to.is_empty() {
            out += &format!("| {} | {} | {} | | | | | |\n", text(&c["class"]), ground, text(&c[reach_key]));
        }
        for t in to {
            let seconds = t["seconds"].as_object().map(|o| o.iter().map(|(u, t)| format!("{u} {}", text(t))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
            let narrow = match &t["the_ways_at_their_tightest"] {
                Value::Null => "-".to_string(),
                w => format!("{} at {}, {}% of the way (median {})", text(&w["width"]), text(&w["at"]), text(&w["share_of_the_way"]), text(&w["median_width_of_the_ways"])),
            };
            out += &format!("| {} | {} | {} | {} | {} | {} | {} | {} |\n", text(&c["class"]), ground, text(&c[reach_key]), text(&t["to"]), text(&t["walk"]), seconds, list(&t[split_key], " / "), narrow);
        }
    }
    if metal_map {
        out += "\n| Start | Taken by (speed) | Seconds to the nearest metal |\n|---|---|---|\n";
        for e in s["expansion"].as_array().into_iter().flatten() {
            out += &format!("| {} | {} ({}) | {} |\n", text(&e["start"]), text(&e["taker"]), text(&e["speed"]), text(&e["seconds_to_the_nearest_metal"]));
        }
        return out;
    }
    out += &format!("\n| Start | Taken by (speed) | Spots within {} s | Seconds to the 5th / 10th / 15th spot |\n|---|---|---|---|\n", list(&s["seconds_within"], " / "));
    for e in s["expansion"].as_array().into_iter().flatten() {
        out += &format!("| {} | {} ({}) | {} | {} |\n", text(&e["start"]), text(&e["taker"]), text(&e["speed"]), list(&e["spots_within_seconds"], " / "), list(&e["seconds_to_the_5th_10th_15th"], " / "));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain(w: u32, h: u32, metal: impl Fn(u32, u32) -> u8) -> Terrain {
        let mut m = Vec::new();
        for z in 0..h {
            for x in 0..w {
                m.push(metal(x, z));
            }
        }
        Terrain { cell: 16.0, width: w, height: h, heights: vec![10; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: m }
    }

    /// A plate's metal is "evenly over every cell"; a map whose metal lies in the west names the cells that hold it.
    #[test]
    fn the_cells_shares_are_even_on_a_plate_and_named_where_the_metal_lies() {
        let grid = crate::Grid::of(2048.0, 1024.0);
        let plate = terrain(128, 64, |_, _| 255);
        let shares = metal_by_cell(&plate, &grid);
        assert_eq!(shares.len(), 2);
        assert_eq!(cell_share_words(&shares, grid.count()), "evenly over every cell (50.0% each)");
        let west = terrain(128, 64, |x, _| if x < 32 { 255 } else if x < 64 { 85 } else { 0 });
        let shares = metal_by_cell(&west, &grid);
        assert_eq!(shares.len(), 1);
        assert_eq!(cell_share_words(&shares, grid.count()), "by cell: A1 100%");
        let both = terrain(128, 64, |x, _| if x < 64 { 255 } else { 85 });
        let shares = metal_by_cell(&both, &grid);
        assert_eq!(cell_share_words(&shares, grid.count()), "by cell: A1 75%, B1 25%");
    }
}
