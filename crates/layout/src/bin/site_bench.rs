//! `site_bench [--repeat N] [--list] <layout-fail-*.bin>...`: the builder menu's pass over a dumped world, off the game: every
//! builder of the world asks for a site for eight kinds (a solar, a wind, a converter, a turret, a storage, a plant, a
//! defence and a radar, each the dump's facts with its own footprint and purpose), as `menu.rs` asks `layout::choose`
//! for every kind on every builder's menu each pass. Prints the pass's time, the per-search median and worst, how
//! many found a site, and a digest of every answer, so a change to the search is judged on the same answers (the
//! digest equal) and its time (1794076407-human: `builder_menus` 3-9 s a pass on eight seats, the seats' hands frozen).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use layout::{Dump, Footprint, Purpose, Request, choose};

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
        let dump = match Dump::read(std::path::Path::new(file)) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{file}: {e}");
                continue;
            }
        };
        let mut hasher = DefaultHasher::new();
        let mut times: Vec<f32> = Vec::new();
        let mut found = 0usize;
        for rep in 0..repeat {
            let world = dump.world();
            for (b, builder) in world.builders.iter().enumerate() {
                for (k, (name, fp, purpose, factory)) in kinds.iter().enumerate() {
                    let mut facts = dump.request.facts.clone();
                    facts.footprint = *fp;
                    facts.factory = *factory;
                    facts.product_half = if *factory { 32.0 } else { 0.0 };
                    facts.def = bot_protocol::UnitDefId(10_000 + k as i32);
                    let centre = match purpose {
                        Purpose::Turret => world.plants.iter().map(|p| p.centre).min_by(|a, b| a.dist2d(builder.at).total_cmp(&b.dist2d(builder.at))).unwrap_or(builder.at),
                        _ => builder.at,
                    };
                    let req = Request { facts, purpose: *purpose, centre, radius: 200.0, builder_at: builder.at, builder: dump.request.builder.clone() };
                    let started = Instant::now();
                    let answer = choose(&world, &req);
                    times.push(started.elapsed().as_secs_f32() * 1000.0);
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
        }
        let total: f32 = times.iter().sum();
        grand += total / repeat as f32;
        times.sort_by(|a, b| a.total_cmp(b));
        println!(
            "{}: {} builders x {} kinds x {repeat} = {} searches, pass {:.0} ms, a search median {:.1} ms, p90 {:.1}, worst {:.1}; {} found; digest {:016x}",
            std::path::Path::new(file).file_name().unwrap().to_string_lossy(),
            dump.builders.len(),
            kinds.len(),
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
