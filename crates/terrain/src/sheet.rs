//! The map sheet: what a map is, in the numbers the decisions turn on, the same for every map and computed from the
//! ground (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3a). The bot computes it at the start of a
//! game for the player's first report; `mapsheet` computes it from a recorded game.

use bot_protocol::{MoveClass, Terrain, Vec3};
use serde_json::{Value, json};

use crate::{Field, costs, passable, passages};

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
/// A narrow place wider than this is open ground, not a passage: a body of twenty in ranks of six stands about 500
/// wide (`crates/micro`, RANK_GAP 96).
const PASSAGE: f32 = 600.0;

/// A unit the sheet turns distances into seconds with.
pub struct Mover {
    pub name: String,
    pub speed: f32,
    pub class: MoveClass,
}

pub struct Input<'a> {
    pub name: &'a str,
    pub terrain: &'a Terrain,
    /// The map's size in elmos and the grid the cells are named on (columns, rows).
    pub size: (f32, f32),
    pub grid: (f32, f32),
    pub wind: (f32, f32),
    pub tidal: f32,
    /// Each spot and what a tier-1 extractor draws from it a second (`None`: not known for this record).
    pub spots: Vec<(Vec3, Option<f32>)>,
    pub movers: Vec<Mover>,
    /// The starts, ours first; every other is priced against the first.
    pub starts: Vec<(String, Vec3)>,
}

pub fn sheet(input: &Input) -> Value {
    let ground = input.terrain;
    let cells = (ground.width * ground.height) as usize;
    let (map_w, map_h) = input.size;
    let (columns, rows) = input.grid;
    let cell_name = |p: Vec3| {
        let c = ((p.x / map_w * columns) as u8).min(columns as u8 - 1);
        let r = ((p.z / map_h * rows) as u32).min(rows as u32 - 1);
        format!("{}{}", (b'A' + c) as char, r + 1)
    };
    let mover = |name: &str| input.movers.iter().find(|m| m.name == name);
    let starts = &input.starts;
    let spots = &input.spots;
    let mut values: Vec<f32> = spots.iter().filter_map(|(_, v)| *v).collect();
    values.sort_by(f32::total_cmp);
    let water = ground.heights.iter().filter(|h| **h < 0).count() as f32 / cells.max(1) as f32;
    let (low, high) = (ground.heights.iter().min().copied().unwrap_or(0), ground.heights.iter().max().copied().unwrap_or(0));
    let field = |class: MoveClass, from: Vec3| Field::from_costs(ground, &costs(ground, class), &[from]);

    let mut classes = Vec::new();
    for (label, units) in CLASSES {
        let Some(class) = units.iter().find_map(|u| mover(u)).map(|m| m.class) else { continue };
        let open = passable(ground, class).iter().filter(|ok| **ok).count() as f32 / cells.max(1) as f32;
        let from: Vec<Option<Field>> = starts.iter().map(|(_, p)| field(class, *p)).collect();
        let Some(home) = from.first().and_then(Option::as_ref) else {
            classes.push(json!({ "class": label, "ground_it_can_stand_on": format!("{:.0}%", open * 100.0), "from_our_start": "it cannot stand at our start" }));
            continue;
        };
        let mut to = Vec::new();
        for ((whose, p), far) in starts.iter().zip(&from).skip(1) {
            let walk = home.distance(*p);
            let seconds: serde_json::Map<String, Value> =
                units.iter().filter_map(|u| mover(u)).filter(|m| m.speed > 0.0).map(|m| (m.name.clone(), json!(walk.map(|w| (w / m.speed).round() as i32)))).collect();
            let found = match far {
                Some(far) if walk.is_some() => passages(home, far),
                _ => Vec::new(),
            };
            let narrowest = found.iter().map(|p| p.width).min_by(f32::total_cmp);
            let tight: Vec<Value> = found.iter().filter(|p| p.width <= PASSAGE).take(4).map(|p| json!({ "at": cell_name(p.at), "width": p.width as i32, "share_of_the_way": (p.along * 100.0) as i32 })).collect();
            let mut split = [0usize; 4];
            if let Some(far) = far {
                for (s, _) in spots {
                    let slot = match (home.distance(*s), far.distance(*s)) {
                        (Some(x), Some(y)) if x.max(y) <= x.min(y) * CONTESTED => 2,
                        (Some(x), Some(y)) if x < y => 0,
                        (Some(_), None) => 0,
                        (None, None) => 3,
                        _ => 1,
                    };
                    split[slot] += 1;
                }
            }
            to.push(json!({
                "to": whose, "walk": walk.map(|w| w as i32), "seconds": seconds,
                "spots_nearer_us_nearer_him_contested_unreachable": split,
                "passages": tight, "narrowest_place_on_the_way": narrowest.map(|w| w as i32),
            }));
        }
        classes.push(json!({
            "class": label, "max_slope": class.max_slope, "wades": class.depth,
            "ground_it_can_stand_on": format!("{:.0}%", open * 100.0),
            "spots_it_reaches_from_our_start": spots.iter().filter(|(s, _)| home.distance(*s).is_some()).count(),
            "to_each_other_start": to,
        }));
    }
    let mut expansion = Vec::new();
    for (who, start) in starts {
        for taker in TAKERS {
            let Some(m) = mover(taker).filter(|m| m.speed > 0.0) else { continue };
            let Some(f) = field(m.class, *start) else { continue };
            let mut walks: Vec<f32> = spots.iter().filter_map(|(s, _)| f.distance(*s)).map(|w| w / m.speed).collect();
            walks.sort_by(f32::total_cmp);
            let counts: Vec<usize> = WITHIN.iter().map(|t| walks.iter().filter(|w| *w <= t).count()).collect();
            let nth = [4, 9, 14].map(|n| walks.get(n).map(|w| w.round() as i32));
            expansion.push(json!({ "start": who, "taker": taker, "speed": m.speed, "spots_within_seconds": counts, "seconds_to_the_5th_10th_15th": nth }));
        }
    }
    let ours = starts.first().map(|(_, p)| *p);
    json!({
        "map": input.name,
        "size": [map_w as i32, map_h as i32],
        "wind": [input.wind.0, input.wind.1], "tidal": input.tidal,
        "under_water": format!("{:.0}%", water * 100.0), "ground_height": [low, high],
        "spots": { "count": spots.len(), "metal_a_second_each_low_median_high": [values.first(), values.get(values.len() / 2), values.last()] },
        "starts": starts.iter().map(|(who, p)| json!({
            "whose": who, "x": p.x as i32, "z": p.z as i32, "cell": cell_name(*p),
            "straight_from_ours": ours.map(|o| ((p.x - o.x).hypot(p.z - o.z)) as i32),
        })).collect::<Vec<_>>(),
        "seconds_within": WITHIN,
        "a_passage_is_narrower_than": PASSAGE,
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
    let mut out = format!("### {}\n\n", text(&s["map"]));
    out += &format!(
        "- {} elmos; wind {}; tidal {}; {} under water; ground height {}.\n- {} metal spots; a tier-1 extractor draws {} metal a second (lowest / median / highest spot).\n- Starts: {}.\n\n",
        list(&s["size"], " by "), list(&s["wind"], " to "), text(&s["tidal"]), text(&s["under_water"]), list(&s["ground_height"], " to "),
        text(&s["spots"]["count"]), list(&s["spots"]["metal_a_second_each_low_median_high"], " / "),
        s["starts"].as_array().map(|a| a.iter().map(|p| format!("{} {} ({}, {}; {} from ours)", text(&p["whose"]), text(&p["cell"]), text(&p["x"]), text(&p["z"]), text(&p["straight_from_ours"]))).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
    );
    out += "| Moves as | Ground it can stand on | Spots it reaches from our start | To | Walked | Seconds for | Spots nearer us / nearer him / contested / unreachable | Passages (cell, width, % of the way) |\n|---|---|---|---|---|---|---|---|\n";
    for c in s["classes"].as_array().into_iter().flatten() {
        let Some(to) = c["to_each_other_start"].as_array() else {
            out += &format!("| {} | {} | {} | | | | | |\n", text(&c["class"]), text(&c["ground_it_can_stand_on"]), text(&c["from_our_start"]));
            continue;
        };
        for t in to {
            let seconds = t["seconds"].as_object().map(|o| o.iter().map(|(u, t)| format!("{u} {}", text(t))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
            let narrow = t["passages"].as_array().map(|a| a.iter().map(|p| format!("{} {} {}%", text(&p["at"]), text(&p["width"]), text(&p["share_of_the_way"]))).collect::<Vec<_>>().join("; ")).filter(|x| !x.is_empty());
            let narrow = narrow.unwrap_or_else(|| match &t["narrowest_place_on_the_way"] {
                Value::Null => "-".into(),
                w => format!("open (narrowest {})", text(w)),
            });
            out += &format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                text(&c["class"]), text(&c["ground_it_can_stand_on"]), text(&c["spots_it_reaches_from_our_start"]), text(&t["to"]), text(&t["walk"]), seconds,
                list(&t["spots_nearer_us_nearer_him_contested_unreachable"], " / "), narrow
            );
        }
    }
    out += &format!("\n| Start | Taken by (speed) | Spots within {} s | Seconds to the 5th / 10th / 15th spot |\n|---|---|---|---|\n", list(&s["seconds_within"], " / "));
    for e in s["expansion"].as_array().into_iter().flatten() {
        out += &format!("| {} | {} ({}) | {} | {} |\n", text(&e["start"]), text(&e["taker"]), text(&e["speed"]), list(&e["spots_within_seconds"], " / "), list(&e["seconds_to_the_5th_10th_15th"], " / "));
    }
    out
}
