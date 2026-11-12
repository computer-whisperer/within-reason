//! The deal replayed over a seat's record (`docs/design/2026-11-11-groups-as-slots.md` §10 stage 0): every armed
//! ground soldier of the seat each second, dealt by `micro::deal` to groups a stand-in for the player declares (a
//! group of `--slots` wherever `--found` soldiers no group has room for stand inside one gate), each group's place
//! following the soldiers with it. The recording is not steered by the deal, so nobody comes when it says coming:
//! this sets the gates and the margin and can only refute them.
//!
//!     cargo run --release -p micro --example deal_replay -- <record-N.jsonl> [--from MIN] [--gate S] [--leave S] [--margin S] [--slots N] [--found N] [--quiet]
//!
//! Printed: every two minutes each group's place with how many are with it and coming; at the end, per unit-minute,
//! how often a soldier's group changed (a re-deal) and how often it fell from with to coming of the same group (the
//! gate's edge), and the shares of soldier-seconds with a group, coming and surplus.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use bot_protocol::{UnitDefId, UnitId, Vec3};
use micro::deal::{Deal, Demand, Gates, Place, Slots, Soldier, Standing, fills, straight};
use serde_json::Value;

fn cell(at: Vec3) -> String {
    let col = (b'A' + ((at.x / 1024.0) as u8).min(15)) as char;
    format!("{col}{}", (at.z / 1024.0) as u32 + 1)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let record = args.first().expect("a record file");
    let number = |flag: &str, default: f32| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<f32>().ok()).unwrap_or(default);
    let from = number("--from", 0.0) as i64 * 60;
    let gate = number("--gate", micro::deal::GATE_SECONDS);
    let gates = Gates { gate, leave: number("--leave", 2.0 * gate), margin: number("--margin", micro::deal::DEAL_MARGIN_SECONDS) };
    let slots = number("--slots", 96.0) as usize;
    let found = number("--found", 24.0) as usize;
    let quiet = args.iter().any(|a| a == "--quiet");

    let mut lines = BufReader::new(std::fs::File::open(record).expect("the record opens")).lines();
    let header: Value = serde_json::from_str(&lines.next().expect("a header").expect("a line")).expect("the header parses");
    // A soldier: the record's `army` and `hover` classes, armed and mobile. `own` names a type by its place in the list.
    let speeds: Vec<Option<f32>> = header["unit_defs"]
        .as_array()
        .expect("unit_defs")
        .iter()
        .map(|d| {
            let class = d["class"].as_str().unwrap_or("");
            let speed = d["speed"].as_f64().unwrap_or(0.0) as f32;
            ((class == "army" || class == "hover") && speed > 0.0 && d["weapons"].as_i64().unwrap_or(0) > 0).then_some(speed)
        })
        .collect();

    let mut deal = Deal::default();
    let mut demands: Vec<Demand> = Vec::new();
    let mut last: HashMap<UnitId, Standing> = HashMap::new();
    let (mut seconds, mut with_s, mut coming_s, mut surplus_s) = (0u64, 0u64, 0u64, 0u64);
    let (mut redeals, mut edges) = (0u64, 0u64);
    for line in lines {
        let line = line.expect("a line");
        if !line[..line.len().min(30)].contains("\"t\":\"s\"") {
            continue;
        }
        let sample: Value = serde_json::from_str(&line).expect("a sample parses");
        let second = sample["f"].as_i64().unwrap_or(0) / 30;
        if second < from {
            continue;
        }
        let soldiers: Vec<Soldier> = sample["own"]
            .as_array()
            .map(|own| {
                own.iter()
                    .filter_map(|u| {
                        let def = u[1].as_i64()? as usize;
                        let speed = (*speeds.get(def)?)?;
                        Some(Soldier { id: UnitId(u[0].as_i64()? as i32), def: UnitDefId(def as i32), pos: Vec3 { x: u[2].as_f64()? as f32, y: 0.0, z: u[3].as_f64()? as f32 }, speed })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut dealt = deal.deal(&soldiers, &demands, &[], &straight, gates);
        // The stand-in player: a group where `found` surplus soldiers stand inside one gate of one of them.
        loop {
            let surplus: Vec<&Soldier> = soldiers.iter().filter(|s| dealt[&s.id] == Standing::Surplus).collect();
            let best = surplus.iter().map(|c| surplus.iter().filter(|s| straight(s, c.pos) <= gates.gate).collect::<Vec<_>>()).max_by_key(Vec::len);
            let Some(mass) = best.filter(|m| m.len() >= found) else { break };
            let n = mass.len() as f32;
            let at = Vec3 { x: mass.iter().map(|s| s.pos.x).sum::<f32>() / n, y: 0.0, z: mass.iter().map(|s| s.pos.z).sum::<f32>() / n };
            demands.push(Demand { name: format!("g{}", demands.len() + 1), slots: Slots::any(slots), place: Place::At(at) });
            dealt = deal.deal(&soldiers, &demands, &[], &straight, gates);
        }
        // Each group's place follows the soldiers with it.
        let fill = fills(&dealt);
        let at_of: HashMap<UnitId, Vec3> = soldiers.iter().map(|s| (s.id, s.pos)).collect();
        for d in &mut demands {
            if let Some(f) = fill.get(&d.name).filter(|f| !f.with.is_empty()) {
                let n = f.with.len() as f32;
                d.place = Place::At(Vec3 { x: f.with.iter().map(|id| at_of[id].x).sum::<f32>() / n, y: 0.0, z: f.with.iter().map(|id| at_of[id].z).sum::<f32>() / n });
            }
        }
        for (id, now) in &dealt {
            seconds += 1;
            match now {
                Standing::With(_) => with_s += 1,
                Standing::Coming(..) => coming_s += 1,
                _ => surplus_s += 1,
            }
            if let Some(then) = last.get(id) {
                match (then.group(), now.group()) {
                    (Some(a), Some(b)) if a != b => redeals += 1,
                    (Some(_), Some(_)) if matches!(then, Standing::With(_)) && matches!(now, Standing::Coming(..)) => edges += 1,
                    _ => {}
                }
            }
        }
        last = dealt;
        if !quiet && second % 120 == 0 {
            let groups: Vec<String> = demands
                .iter()
                .map(|d| {
                    let f = fill.get(&d.name).cloned().unwrap_or_default();
                    let Place::At(at) = d.place else { unreachable!() };
                    let tail = f.coming.last().map_or(String::new(), |(_, t)| format!(" (last {t:.0}s)"));
                    format!("{}@{} {}+{}{tail}", d.name, cell(at), f.with.len(), f.coming.len())
                })
                .collect();
            println!("{:2}:{:02} soldiers {:3} | {} | surplus {}", second / 60, second % 60, soldiers.len(), groups.join("  "), fill.get("").map_or(0, |f| f.with.len()));
        }
    }
    let minutes = seconds as f64 / 60.0;
    println!(
        "gate {} leave {} margin {} slots {slots} found {found}: groups {}; re-deals {:.2} a unit-minute, with-to-coming {:.2}; soldier-seconds with {:.0} % coming {:.0} % surplus {:.0} % of {seconds}",
        gates.gate,
        gates.leave,
        gates.margin,
        demands.len(),
        redeals as f64 / minutes.max(1.0),
        edges as f64 / minutes.max(1.0),
        100.0 * with_s as f64 / seconds.max(1) as f64,
        100.0 * coming_s as f64 / seconds.max(1) as f64,
        100.0 * surplus_s as f64 / seconds.max(1) as f64
    );
}
