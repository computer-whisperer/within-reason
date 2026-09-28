//! The engagement plan on a recorded moment (`docs/design/2026-09-28-engagement-plan.md`, validation): the brain is
//! rebuilt from the record's header and terrain, the enemy's buildings are remembered as the brain remembers them
//! (every static seen up to the clock, until its death), our units and his in sight are the sample at the clock, the
//! groups, places, party names and the player's packet are the Jev log's last call before it; then the same code as
//! the live pass finds the position, words the battlefield, enumerates the candidates and asks Jev.
//!
//! usage: bot --plan-replay <match dir> <m:ss> [--body group_X] [--dump] [--no-examples] [--both] [--repeat N]
//!   --dump prints the request and asks nothing; --both asks with and without the examples block. The key is read
//!   by `jev::Client::from_env` and never printed.

use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use bot_protocol::{EnemyUnit, OwnUnit, UnitDefId, UnitId, Vec3};
use serde_json::Value;

use super::super::{Brain, FRAMES_PER_SECOND};
use super::engagement::{self, Candidate};
use super::picture::{Party, Place};
use super::{Group, GroupTask, Pianist};

struct Args {
    dir: PathBuf,
    frame: i32,
    body: Option<String>,
    dump: bool,
    examples: Vec<bool>,
    repeat: usize,
}

fn args(raw: &[String]) -> Result<Args, String> {
    let usage = "usage: bot --plan-replay <match dir> <m:ss> [--body group_X] [--dump] [--no-examples] [--both] [--repeat N]";
    let mut positional = Vec::new();
    let (mut body, mut dump, mut examples, mut repeat) = (None, false, vec![true], 1);
    let mut it = raw.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--body" => body = Some(it.next().ok_or(usage)?.clone()),
            "--dump" => dump = true,
            "--no-examples" => examples = vec![false],
            "--both" => examples = vec![true, false],
            "--repeat" => repeat = it.next().and_then(|n| n.parse().ok()).ok_or(usage)?,
            other => positional.push(other.to_string()),
        }
    }
    let [dir, clock] = positional.as_slice() else { return Err(usage.into()) };
    let (m, s) = clock.split_once(':').ok_or(usage)?;
    let frame = (m.parse::<i32>().map_err(|_| usage)? * 60 + s.parse::<i32>().map_err(|_| usage)?) * FRAMES_PER_SECOND;
    Ok(Args { dir: dir.into(), frame, body, dump, examples, repeat })
}

fn lines(path: &Path) -> Result<impl Iterator<Item = String>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(BufReader::new(file).lines().map_while(Result::ok))
}

fn at(x: &Value, z: &Value) -> Vec3 {
    Vec3 { x: x.as_f64().unwrap_or(0.0) as f32, y: 0.0, z: z.as_f64().unwrap_or(0.0) as f32 }
}

/// The Jev log's last call at or before the frame, with the packet carried forward to it.
fn last_call(dir: &Path, ai: i64, frame: i32) -> Result<(Value, String), String> {
    let mut last = None;
    let mut instructions = String::new();
    for line in lines(&dir.join(format!("jev-{ai}.jsonl")))? {
        if !line.contains("\"t\":\"call\"") {
            continue;
        }
        let Ok(call) = serde_json::from_str::<Value>(&line) else { continue };
        if call["f"].as_i64().unwrap_or(0) > frame as i64 {
            break;
        }
        if let Some(text) = call["instructions"].as_str() {
            instructions = text.to_string();
        }
        last = Some(call);
    }
    Ok((last.ok_or("the Jev log has no call before the clock")?, instructions))
}

pub fn plan_replay(raw: &[String]) -> Result<(), String> {
    let a = args(raw)?;
    let record = std::fs::read_dir(&a.dir).map_err(|e| e.to_string())?.filter_map(Result::ok).map(|e| e.path()).find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("record-") && n.ends_with(".jsonl"))).ok_or("no record-<ai>.jsonl in the match directory")?;
    let mut records = lines(&record)?;
    let header: Value = serde_json::from_str(&records.next().ok_or("an empty record")?).map_err(|e| e.to_string())?;
    let ai = header["ai_id"].as_i64().unwrap_or(0);
    let (hello, ids) = crate::recorder::hello_from_record(&a.dir, &header)?;
    let def_of = |index: &Value| -> Option<UnitDefId> { index.as_i64().filter(|i| *i >= 0).and_then(|i| ids.get(i as usize).copied()) };
    let scratch = std::env::temp_dir();
    let pianist = Pianist::new(false, &scratch, ai as i32)?;
    let mut brain = Brain::new(crate::world::World::new(hello), None, std::sync::Arc::new(crate::team::TeamBoard::default()), String::new(), Some(pianist));
    brain.survey_sim_defs();
    // The record up to the clock: the faction and home from the first sample, his buildings as the brain keeps them
    // (`track_enemy_buildings`: every static seen, dropped at its death), and the last sample's units.
    let mut sample: Option<Value> = None;
    let mut adopted = false;
    for line in records {
        let Ok(r) = serde_json::from_str::<Value>(&line) else { continue };
        let f = r["f"].as_i64().unwrap_or(0) as i32;
        if f > a.frame {
            break;
        }
        match r["t"].as_str() {
            Some("s") => {
                if !adopted {
                    adopted = true;
                    let own = own_units(&r, &def_of);
                    brain.adopt_faction(&own);
                }
                for e in r["en"].as_array().into_iter().flatten() {
                    let (Some(def), Some(id)) = (def_of(&e[1]), e[0].as_i64()) else { continue };
                    if brain.world.def(def).is_some_and(|d| d.speed == 0.0) {
                        let id = UnitId(id as i32);
                        brain.enemy_buildings.insert(id, (def, at(&e[2], &e[3]), f));
                        if e[5].as_i64() == Some(1) {
                            brain.enemy_unfinished.insert(id);
                        } else {
                            brain.enemy_unfinished.remove(&id);
                        }
                    }
                }
                sample = Some(r);
            }
            Some("ev") if r["k"] == "enemy_destroyed" => {
                if let Some(id) = r["u"].as_i64() {
                    brain.enemy_buildings.remove(&UnitId(id as i32));
                    brain.enemy_unfinished.remove(&UnitId(id as i32));
                }
            }
            _ => {}
        }
    }
    let sample = sample.ok_or("no sample before the clock")?;
    let own = own_units(&sample, &def_of);
    let enemies: Vec<EnemyUnit> = sample["en"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|e| EnemyUnit { id: UnitId(e[0].as_i64().unwrap_or(0) as i32), def: def_of(&e[1]), pos: at(&e[2], &e[3]), vel: Vec3::default(), health: e[4].as_f64().unwrap_or(0.0) as f32, team: None, being_built: e[5].as_i64() == Some(1) })
        .collect();
    let (call, instructions) = last_call(&a.dir, ai, a.frame)?;
    let places: Vec<Place> = call["places"].as_array().into_iter().flatten().map(|p| Place { name: p["name"].as_str().unwrap_or_default().to_string(), at: at(&p["x"], &p["z"]), spot: p["spot"].as_u64().map(|s| s as usize) }).collect();
    let previous: Vec<Party> = call["parties"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| Party { name: p["name"].as_str().unwrap_or_default().to_string(), ids: p["ids"].as_array().into_iter().flatten().filter_map(Value::as_i64).map(|i| UnitId(i as i32)).collect(), at: at(&p["x"], &p["z"]), metal: 0.0, composition: String::new(), has_commander: false, killing: None, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() })
        .collect();
    let next = std::cell::Cell::new(900);
    let parties = brain.enemy_parties(&enemies, &previous, &next);
    let groups: Vec<Group> = call["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|g| {
            let members: Vec<UnitId> = g["members"].as_array().into_iter().flatten().filter_map(Value::as_i64).map(|i| UnitId(i as i32)).collect();
            let domain = members.iter().find_map(|id| own.iter().find(|u| u.id == *id)).map_or(crate::world::Domain::Ground, |u| brain.world.domain_of(u.def));
            let to = at(&g["task"]["to"][0], &g["task"]["to"][1]);
            let task = match g["task"]["kind"].as_str() {
                Some("move_to") | Some("fight_to") => GroupTask::Move { to, place: g["task"]["place"].as_str().unwrap_or_default().to_string(), fight: g["task"]["kind"] == "fight_to", since: a.frame },
                Some("engage") | Some("attack_unit") => GroupTask::Engage { party: Vec::new(), at: to, since: a.frame, last_seen: a.frame, target: None, searched: false, from: to },
                _ => GroupTask::Hold { since: a.frame, committed: false },
            };
            Group::new(g["name"].as_str().unwrap_or_default().to_string(), domain, members, task, a.frame)
        })
        .collect();
    let standing: HashMap<String, String> = call["state"]["actors"].as_object().into_iter().flatten().filter_map(|(k, v)| v["standing"].as_str().map(|s| (k.clone(), s.to_string()))).collect();
    {
        let p = brain.pianist.as_mut().expect("built with one");
        p.groups = groups;
        p.places = places.clone();
        p.parties = parties.clone();
    }
    println!("{} at {} (frame {}; the Jev log's call at {}): {} of ours, {} of his in sight, {} of his buildings remembered", a.dir.display(), super::picture::clock(a.frame), a.frame, super::picture::clock(call["f"].as_i64().unwrap_or(0) as i32), own.len(), enemies.len(), brain.enemy_buildings.len());
    let pianist = brain.pianist.as_ref().expect("built with one");
    let wanted: Vec<&Group> = pianist.groups.iter().filter(|g| a.body.as_ref().is_none_or(|b| *b == format!("group_{}", g.name))).collect();
    if wanted.is_empty() {
        return Err(format!("no group {} in the log's call", a.body.unwrap_or_default()));
    }
    let mut asked = Vec::new();
    for group in wanted {
        let name = format!("group_{}", group.name);
        let task = match &group.task {
            GroupTask::Hold { .. } => "holding".to_string(),
            GroupTask::Move { place, fight, .. } => format!("{} to {place}", if *fight { "advancing" } else { "walking" }),
            GroupTask::Engage { .. } => "attacking a party".to_string(),
        };
        match brain.engagement_of(group, &own, &parties, &enemies, &places, false) {
            Err(why) => println!("\n{name} ({task}): not planned: {why}"),
            Ok((bf, candidates)) => {
                let live = if group.task.busy() { String::new() } else { " (holding: the log does not say whether the hold is an advance's arrival, which the live gate plans, or a hold it does not; planned here)".to_string() };
                println!("\n{name} ({task}){live}: position {} at {:.0} from its front", bf.position.signature(), bf.position.distance_to(bf.ours.front));
                let words = brain.battlefield_words(&bf, &places);
                let paragraph = super::diet::paragraph(&instructions, &name).unwrap_or_default().to_string();
                asked.push((name, words, candidates, paragraph, standing.get(&format!("group_{}", group.name)).cloned().unwrap_or_default()));
            }
        }
    }
    for (name, words, candidates, paragraph, standing) in &asked {
        println!("\nbattlefield of {name}:\n{}", serde_json::to_string_pretty(words).unwrap_or_default());
        println!("\ncandidates:");
        for c in candidates.iter() {
            let phases: Vec<String> = c.phases.iter().map(|p| format!("{:?} {}", p.kind, p.elements.join("+"))).collect();
            println!("  {} [{}]: {}", c.key, phases.join(" > "), c.words);
        }
        if a.dump {
            let request = engagement::request(name, words.clone(), candidates, paragraph, standing, a.examples[0]);
            println!("\nrequest:\n{}", serde_json::to_string_pretty(&serde_json::json!({ "state": request.state, "questions": request.questions })).unwrap_or_default());
            continue;
        }
        let client = jev::Client::from_env().map_err(|e| e.to_string())?;
        for examples in &a.examples {
            for rep in 0..a.repeat {
                let request = engagement::request(name, words.clone(), candidates, paragraph, standing, *examples);
                let response = client.ask(&request).map_err(|e| e.to_string())?;
                let ranking = ranking(&response.answers, candidates);
                let taken = engagement::choose(&response.answers, candidates).map(|(i, _)| candidates[i].key).unwrap_or("none");
                println!("\n{name} {} examples, ask {} ({}, {} ms): {ranking}; taken: {taken}", if *examples { "with" } else { "without" }, rep + 1, response.model, response.latency.as_millis());
            }
        }
    }
    Ok(())
}

fn own_units(sample: &Value, def_of: &dyn Fn(&Value) -> Option<UnitDefId>) -> Vec<OwnUnit> {
    sample["own"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|u| {
            let flags = u[5].as_i64().unwrap_or(0);
            Some(OwnUnit { id: UnitId(u[0].as_i64()? as i32), def: def_of(&u[1])?, pos: at(&u[2], &u[3]), vel: Vec3::default(), health: (u[4].as_f64().unwrap_or(100.0) as f32 / 100.0).max(0.0), max_health: 1.0, being_built: flags & 1 != 0, idle: flags & 2 != 0, reload_frame: 0, facing: 0 })
        })
        .filter(|u| u.health > 0.0)
        .collect()
}

/// The candidates by Jev's probability, highest first: "screen_first 0.52, nearest_first 0.21, ...".
fn ranking(answers: &BTreeMap<String, jev::Answer>, candidates: &[Candidate]) -> String {
    let Some(jev::Answer::Choice { probabilities, .. }) = answers.get(engagement::QUESTION) else { return "no answer".into() };
    let mut ranked: Vec<(&str, f64)> = candidates.iter().map(|c| (c.key, probabilities.get(c.key).copied().unwrap_or(0.0))).collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.iter().map(|(k, p)| format!("{k} {p:.2}")).collect::<Vec<_>>().join(", ")
}
