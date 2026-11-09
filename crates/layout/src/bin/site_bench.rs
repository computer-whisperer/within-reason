//! `site_bench [--repeat N] [--list] <layout-fail-*.bin>...`: the builder menu's pass over a dumped world, off the game: every
//! builder of the world asks for a site for eight kinds (a solar, a wind, a converter, a turret, a storage, a plant, a
//! defence and a radar, each the dump's facts with its own footprint and purpose), as `menu.rs` asks `layout::choose`
//! for every kind on every builder's menu each pass. Prints the pass's time, the per-search median and worst, how
//! many found a site, and a digest of every answer, so a change to the search is judged on the same answers (the
//! digest equal) and its time (1794076407-human: `builder_menus` 3-9 s a pass on eight seats, the seats' hands frozen).
//!
//! A `layout-pass-*.bin` (`layout::PassDump`: a slow pass of the game with every site question it asked) is asked
//! its own questions instead, in the game's order, and the time is also said by purpose.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use layout::{Dump, Footprint, PassDump, Purpose, Request, choose};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let list = args.iter().any(|a| a == "--list");
    let repeat: usize = args.iter().position(|a| a == "--repeat").and_then(|i| args.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(1);
    let files: Vec<&String> = args.iter().enumerate().filter(|(i, a)| !a.starts_with("--") && !(*i > 0 && args[i - 1] == "--repeat")).map(|(_, a)| a).collect();
    if files.is_empty() {
        eprintln!("usage: site_bench [--repeat N] <layout-fail-N-M.bin>...");
        std::process::exit(2);
    }
    let kinds: [(&str, Footprint, Purpose, bool); 8] = [
        ("solar", Footprint { x: 4, z: 4 }, Purpose::Generator { cap: None }, false),
        ("wind", Footprint { x: 3, z: 3 }, Purpose::Generator { cap: Some(6) }, false),
        ("converter", Footprint { x: 4, z: 4 }, Purpose::Generator { cap: None }, false),
        ("turret", Footprint { x: 6, z: 6 }, Purpose::Turret, false),
        ("storage", Footprint { x: 4, z: 4 }, Purpose::Frame, false),
        ("plant", Footprint { x: 8, z: 8 }, Purpose::Plant, true),
        ("defence", Footprint { x: 2, z: 2 }, Purpose::Place, false),
        ("radar", Footprint { x: 2, z: 2 }, Purpose::Place, false),
    ];
    let mut grand = 0.0f32;
    for file in files {
        // A pass dump is read first: a failure dump ends before a pass dump's questions begin.
        let (dump, asked) = match PassDump::read(std::path::Path::new(file)) {
            Ok(pass) => (pass.dump, Some(pass.requests)),
            Err(_) => match Dump::read(std::path::Path::new(file)) {
                Ok(d) => (d, None),
                Err(e) => {
                    eprintln!("{file}: {e}");
                    continue;
                }
            },
        };
        let mut hasher = DefaultHasher::new();
        let mut times: Vec<f32> = Vec::new();
        let mut by_purpose: std::collections::BTreeMap<String, (usize, f32)> = Default::default();
        let mut found = 0usize;
        for rep in 0..repeat {
            let world = dump.world();
            // The questions: the pass's own, or every builder of the world asking for the eight kinds.
            let questions: Vec<(usize, String, Request)> = match &asked {
                Some(requests) => requests.iter().enumerate().map(|(i, r)| (i, format!("def {} {:?}", r.facts.def.0, r.purpose), r.clone())).collect(),
                None => world
                    .builders
                    .iter()
                    .enumerate()
                    .flat_map(|(b, builder)| {
                        let (dump, world) = (&dump, &world);
                        kinds.iter().enumerate().map(move |(k, (name, fp, purpose, factory))| {
                            let mut facts = dump.request.facts.clone();
                            facts.footprint = *fp;
                            facts.factory = *factory;
                            facts.product_half = if *factory { 32.0 } else { 0.0 };
                            facts.def = bot_protocol::UnitDefId(10_000 + k as i32);
                            let centre = match purpose {
                                Purpose::Turret => world.plants.iter().map(|p| p.centre).min_by(|a, b| a.dist2d(builder.at).total_cmp(&b.dist2d(builder.at))).unwrap_or(builder.at),
                                _ => builder.at,
                            };
                            (b, name.to_string(), Request { facts, purpose: *purpose, centre, radius: 200.0, builder_at: builder.at, builder: dump.request.builder.clone() })
                        })
                    })
                    .collect(),
            };
            for (b, name, req) in &questions {
                let (b, name) = (*b, name.as_str());
                let started = Instant::now();
                let answer = choose(&world, req);
                let ms = started.elapsed().as_secs_f32() * 1000.0;
                times.push(ms);
                let slot = by_purpose.entry(format!("{:?}", req.purpose)).or_default();
                *slot = (slot.0 + 1, slot.1 + ms);
                // The digest is the first pass's answers, whatever the repeat; `--list` prints each of them.
                if list && rep == 0 {
                    match &answer {
                        Ok(site) => println!("  builder {b} {name}: ({:.0}, {:.0}) facing {} | {}", site.at.x, site.at.z, site.facing, site.words),
                        Err(why) => println!("  builder {b} {name}: none: {why}"),
                    }
                }
                match &answer {
                    Ok(site) => {
                        found += 1;
                        if rep == 0 {
                            (b, name, site.at.x as i32, site.at.z as i32, site.facing).hash(&mut hasher);
                        }
                    }
                    Err(why) if rep == 0 => (b, name, why.as_str()).hash(&mut hasher),
                    Err(_) => {}
                }
            }
        }
        let total: f32 = times.iter().sum();
        grand += total / repeat as f32;
        times.sort_by(|a, b| a.total_cmp(b));
        if asked.is_some() {
            let purposes = by_purpose.iter().map(|(p, (n, ms))| format!("{p} {} in {:.0} ms", n / repeat, ms / repeat as f32)).collect::<Vec<_>>().join("; ");
            println!("{}: {} (the game's pass {:.0} ms; {} standing, {} plants, {} builders); by purpose: {purposes}", std::path::Path::new(file).file_name().unwrap().to_string_lossy(), dump.label, dump.ms, dump.standing.len(), dump.plants.len(), dump.builders.len());
        }
        println!(
            "{}: {} builders, {} questions x {repeat} = {} searches, pass {:.0} ms, a search median {:.1} ms, p90 {:.1}, worst {:.1}; {} found; digest {:016x}",
            std::path::Path::new(file).file_name().unwrap().to_string_lossy(),
            dump.builders.len(),
            times.len() / repeat,
            times.len(),
            total / repeat as f32,
            times[times.len() / 2],
            times[times.len() * 9 / 10],
            times[times.len() - 1],
            found / repeat,
            hasher.finish()
        );
    }
    println!("all dumps: {grand:.0} ms a pass");
}
