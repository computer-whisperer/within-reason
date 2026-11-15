//! A recorded group walked through the lane again, open loop: the units of a match record (`docs/harness/
//! record-format.md`) where they stood each second, the group declared with the members the record names and the
//! order given here, and what the lane made of it second by second: every company's anchor, its way's end, whether it
//! stands, forms up or closes, beside the anchor the record holds for that second. For the question a record cannot
//! answer, why a formation went where it went. By hand:
//!
//! `WR_RECORD=<record-N.jsonl> WR_GROUP=group_A_t3 WR_TO=2600,4568 WR_FROM=21:15 WR_UNTIL=23:10 [WR_CAP=24]
//!  [WR_KIND=fight|go|hold] [WITHIN_REASON_GROUP_DEBUG=1] cargo test -p micro --release a_recorded_group_again -- --ignored --nocapture`
//!
//! What it is not: the game. The record holds places once a second (walked between here at the lane's ten ticks), no
//! wrecks, no buildings of theirs but those in sight, and the lane begins with no books, so its companies are founded
//! afresh at `WR_FROM`. The printed distance between each replayed company and the nearest recorded one says how far
//! the two have parted. The types' numbers are the record's (reach, speed, footprint, class); a type with a weapon
//! deals 10 a second.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, EnemyUnit, MoveClass, MoveKind, OwnUnit, Snapshot, Terrain, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use serde_json::Value;

use crate::{GroupKind, GroupOrder, Lane, Stats, View};

struct Recorded {
    defs: Vec<UnitDefInfo>,
    stats: Vec<Option<Stats>>,
    terrain: Terrain,
}

impl View for Recorded {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.defs.get(def.0 as usize)
    }
    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        self.stats.get(def.0 as usize).copied().flatten()
    }
    fn grid_spec(&self) -> (f32, usize, usize) {
        let cell = 128.0;
        (cell, (self.terrain.width as f32 * self.terrain.cell / cell).ceil() as usize, (self.terrain.height as f32 * self.terrain.cell / cell).ceil() as usize)
    }
    fn terrain(&self) -> Option<&Terrain> {
        Some(&self.terrain)
    }
    fn home(&self) -> Vec3 {
        Vec3::default()
    }
    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        Vec::new()
    }
    fn building_at(&self, _id: UnitId) -> Option<Vec3> {
        None
    }
    fn enemy_known(&self, _id: UnitId) -> bool {
        true
    }
    fn blip_def(&self) -> Option<UnitDefId> {
        None
    }
    fn enabled(&self, _rule: &str) -> bool {
        true
    }
    fn mine(&self, _unit: UnitId) -> bool {
        true
    }
    fn label(&self) -> String {
        "replay".to_string()
    }
}

fn clock(text: &str) -> i32 {
    let (m, s) = text.split_once(':').expect("M:SS");
    m.parse::<i32>().unwrap() * 60 + s.parse::<i32>().unwrap()
}

fn num(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.0) as f32
}

/// One second of the record: our units (id, type, place, health in percent, being built), theirs in sight, and the
/// group's members and companies as the lane of the game had them.
struct Second {
    own: HashMap<u32, (u32, Vec3, f32, bool)>,
    enemies: Vec<(u32, Option<u32>, Vec3, f32, bool)>,
    members: Vec<u32>,
    companies: Vec<(Vec3, usize, usize, bool)>,
}

#[test]
#[ignore]
fn a_recorded_group_again() {
    let path = std::env::var("WR_RECORD").expect("WR_RECORD");
    let group = std::env::var("WR_GROUP").expect("WR_GROUP");
    let to = {
        let v = std::env::var("WR_TO").expect("WR_TO");
        let (x, z) = v.split_once(',').expect("x,z");
        Vec3 { x: x.parse().unwrap(), y: 0.0, z: z.parse().unwrap() }
    };
    let (from, until) = (clock(&std::env::var("WR_FROM").expect("WR_FROM")), clock(&std::env::var("WR_UNTIL").expect("WR_UNTIL")));
    let cap: usize = std::env::var("WR_CAP").ok().map_or(24, |v| v.parse().unwrap());
    let kind = match std::env::var("WR_KIND").as_deref() {
        Ok("go") => GroupKind::Go,
        Ok("hold") => GroupKind::Hold,
        _ => GroupKind::FightTo,
    };
    let text = std::fs::read_to_string(&path).expect("the record");
    let mut lines = text.lines();
    let header: Value = serde_json::from_str(lines.next().expect("a header")).expect("the header");
    // The types, by their place in the header's list (the record's `d`).
    let mut defs = Vec::new();
    let mut stats = Vec::new();
    for (i, d) in header["unit_defs"].as_array().expect("unit_defs").iter().enumerate() {
        let weapons = d["weapons"].as_i64().unwrap_or(0) as i32;
        let mut def = crate::test_host::def(UnitDefId(i as i32), d["name"].as_str().unwrap_or(""), num(&d["speed"]), weapons, num(&d["build_speed"]), num(&d["extracts_metal"]));
        def.footprint = (d["footprint"][0].as_i64().unwrap_or(2) as i32, d["footprint"][1].as_i64().unwrap_or(2) as i32);
        def.max_height_dif = num(&d["max_height_dif"]);
        def.move_class = d["move"].as_array().map(|m| MoveClass {
            kind: match m[0].as_str().unwrap_or("") {
                "bot" => MoveKind::Bot,
                "hover" => MoveKind::Hover,
                "ship" => MoveKind::Ship,
                _ => MoveKind::Tank,
            },
            max_slope: num(&m[1]),
            depth: num(&m[2]),
            slope_mod: num(&m[3]),
            footprint: m[4].as_i64().unwrap_or(0) as i32,
        });
        stats.push(Some(Stats { reach: num(&d["reach"]), dps: if weapons > 0 { 10.0 } else { 0.0 }, speed: num(&d["speed"]), health: num(&d["health"]), sight: num(&d["sight"]), ..Stats::default() }));
        defs.push(def);
    }
    let meta = &header["terrain"];
    let (width, height) = (meta["width"].as_u64().unwrap() as usize, meta["height"].as_u64().unwrap() as usize);
    let bytes = std::fs::read(std::path::Path::new(&path).with_file_name(meta["file"].as_str().unwrap())).expect("the terrain file");
    let n = width * height;
    let terrain = Terrain { cell: num(&meta["cell"]), width: width as u32, height: height as u32, heights: (0..n).map(|i| i16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]])).collect(), slopes: bytes[2 * n..3 * n].to_vec(), metal: Vec::new() };
    let view = Recorded { defs, stats, terrain };
    // The seconds asked for, and one more to walk to.
    let mut seconds: HashMap<i32, Second> = HashMap::new();
    for line in lines {
        if !line.contains("\"t\":\"s\"") {
            continue;
        }
        let Some(at) = line.find("\"f\":") else { continue };
        let frame: i32 = line[at + 4..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(-1);
        if frame % 30 != 0 || frame / 30 < from || frame / 30 > until + 1 {
            continue;
        }
        let row: Value = serde_json::from_str(line).expect("a sample");
        let own = row["own"].as_array().map_or(HashMap::new(), |units| {
            units.iter().map(|u| (u[0].as_u64().unwrap() as u32, (u[1].as_u64().unwrap() as u32, Vec3 { x: num(&u[2]), y: 0.0, z: num(&u[3]) }, num(&u[4]), u[5].as_i64().unwrap_or(0) & 1 == 1))).collect()
        });
        let enemies = row["en"].as_array().map_or(Vec::new(), |units| {
            units.iter().map(|e| (e[0].as_u64().unwrap() as u32, e[1].as_i64().filter(|d| *d >= 0).map(|d| d as u32), Vec3 { x: num(&e[2]), y: 0.0, z: num(&e[3]) }, num(&e[4]), e[5].as_i64().unwrap_or(0) == 1)).collect()
        });
        let g = &row["groups"][&group];
        let mut members: Vec<u32> = g["formed"].as_array().map_or(Vec::new(), |f| f.iter().map(|u| u.as_u64().unwrap() as u32).collect());
        members.extend(g["exceptions"].as_array().map_or(Vec::new(), |x| x.iter().map(|e| e[0].as_u64().unwrap() as u32).collect::<Vec<_>>()));
        let companies = g["companies"].as_array().map_or(Vec::new(), |cs| {
            cs.iter().map(|c| (Vec3 { x: num(&c["anchor"][0]), y: 0.0, z: num(&c["anchor"][1]) }, c["members"].as_u64().unwrap_or(0) as usize, c["formed"].as_u64().unwrap_or(0) as usize, c["arrived"].as_bool().unwrap_or(false))).collect()
        });
        seconds.insert(frame / 30, Second { own, enemies, members, companies });
    }
    let mut lane = Lane::default();
    let order = GroupOrder::new(kind, vec![to]).with_cap(cap);
    let name = group.strip_prefix("group_").unwrap_or(&group).to_string();
    for second in from..until {
        let (Some(now), Some(next)) = (seconds.get(&second), seconds.get(&(second + 1))) else { continue };
        let members: Vec<UnitId> = now.members.iter().map(|u| UnitId(*u as i32)).collect();
        lane.set_group(&name, members.clone(), order.clone());
        let mine: HashSet<u32> = now.members.iter().copied().collect();
        for k in 0..10 {
            let share = k as f32 / 10.0;
            let own_units: Vec<OwnUnit> = (now.own.iter())
                .filter(|(id, (def, ..))| mine.contains(id) || view.defs[*def as usize].speed <= 0.0)
                .map(|(id, (def, at, health, building))| {
                    let then = next.own.get(id).map_or(*at, |(_, p, ..)| *p);
                    let pos = Vec3 { x: at.x + (then.x - at.x) * share, y: 0.0, z: at.z + (then.z - at.z) * share };
                    let max = view.stats[*def as usize].map_or(100.0, |s| s.health.max(1.0));
                    serde_json::from_value(serde_json::json!({
                        "id": id, "def": def, "pos": pos, "vel": Vec3 { x: (then.x - at.x) / 30.0, y: 0.0, z: (then.z - at.z) / 30.0 },
                        "health": max * health / 100.0, "max_health": max, "being_built": building, "idle": false, "reload_frame": 0, "facing": 0,
                    }))
                    .expect("an own unit")
                })
                .collect();
            let enemies: Vec<EnemyUnit> = (now.enemies.iter())
                .map(|(id, def, at, health, building)| {
                    serde_json::from_value(serde_json::json!({ "id": id, "def": def, "pos": at, "vel": Vec3::default(), "health": health, "team": null, "being_built": building })).expect("an enemy")
                })
                .collect();
            let tick = Tick { frame: second * 30 + k * 3, late: 0, events: Vec::new(), snapshot: Snapshot { own_units, enemies, ..Snapshot::default() } };
            let mut host: Vec<Command> = Vec::new();
            lane.tick(&view, &tick, &mut host, false);
        }
        let Some(declared) = lane.groups.get(&name) else { continue };
        println!("{}:{:02}  the record: {}", second / 60, second % 60, now.companies.iter().map(|(a, m, f, arrived)| format!("({:.0},{:.0}) {f}/{m}{}", a.x, a.z, if *arrived { " ARRIVED" } else { "" })).collect::<Vec<_>>().join(" | "));
        for c in &declared.army.companies {
            let f = &c.formation;
            let state = c.state.as_ref();
            let near = now.companies.iter().map(|(a, ..)| a.dist2d(f.anchor)).fold(f32::INFINITY, f32::min);
            println!(
                "    {:14} ({:.0},{:.0}) {:4.0} from the place, {:4.0} from the record's nearest; {}/{}{}{}{} pace {:.0}; {}",
                f.name,
                f.anchor.x,
                f.anchor.z,
                f.anchor.dist2d(to),
                near,
                state.map_or(0, |s| s.formed.len()),
                c.members.len(),
                if state.is_some_and(|s| s.arrived) { " ARRIVED" } else { "" },
                if f.passing() { " forming up" } else { "" },
                if f.footprint().standing { " standing" } else { "" },
                state.map_or(0.0, |s| s.pace),
                f.probe()
            );
        }
    }
}
