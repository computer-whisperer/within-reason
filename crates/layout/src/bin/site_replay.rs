//! `site_replay [--without-asker] [--without-kind NAME] [--near] <layout-fail-N-M.bin>...`: a site search the bot
//! dumped when it failed (`dump.rs`), run again off the game over the same world and traced: each widening's points,
//! candidates and time, and each tried candidate's verdict. `--without-asker` takes the asking builder out of the
//! world's builders before the search, to show whether the site was refused for walling in the builder that asked
//! for it; `--without-kind NAME` takes every standing building of that kind (its internal name, `armnanotc`) out, to
//! show which buildings close a street (econ-flagship-8: every advanced fusion site within 800 of the block refused
//! for the plant's street); `--near` lists what stands within the request's band of its centre.

use std::time::Instant;

use layout::{Dump, choose_traced};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let without_asker = args.iter().any(|a| a == "--without-asker");
    let near = args.iter().any(|a| a == "--near");
    let without_kinds: Vec<&String> = args.windows(2).filter(|w| w[0] == "--without-kind").map(|w| &w[1]).collect();
    let files: Vec<&String> = args.iter().enumerate().filter(|(i, a)| !a.starts_with("--") && !(*i > 0 && args[i - 1] == "--without-kind")).map(|(_, a)| a).collect();
    if files.is_empty() {
        eprintln!("usage: site_replay [--without-asker] [--without-kind NAME] [--near] <layout-fail-N-M.bin>...");
        std::process::exit(2);
    }
    for file in files {
        let dump = match Dump::read(std::path::Path::new(file)) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{file}: {e}");
                continue;
            }
        };
        let req = &dump.request;
        println!("== {file}: {}", dump.label);
        println!("the game: {} ({:.0} ms, {} trace lines)", dump.why, dump.ms, dump.trace.len());
        println!(
            "world: map {:.0} x {:.0}, open {}, {} spots, {} standing, {} plants, {} builders, {} workers, {} frames, {} reservations, {} refused, lattice {}",
            dump.extent.0,
            dump.extent.1,
            dump.open.as_ref().map_or("everywhere".to_string(), |w| format!("{} of {} cells", w.iter().filter(|p| **p).count(), w.len())),
            dump.spots.len(),
            dump.standing.len(),
            dump.plants.len(),
            dump.builders.len(),
            dump.workers.len(),
            dump.frames.len(),
            dump.reservations.len(),
            dump.refused.len(),
            dump.lattice.map_or("none".to_string(), |l| format!("pitch {:.0} at ({:.0}, {:.0})", l.pitch, l.centre.x, l.centre.z)),
        );
        println!(
            "request: def {} footprint {}x{} {:?}, band {:.0} round ({:.0}, {:.0}), builder def {} ({}x{}) at ({:.0}, {:.0}), product half {:.0}, products' walk {}",
            req.facts.def.0,
            req.facts.footprint.x,
            req.facts.footprint.z,
            req.purpose,
            req.radius,
            req.centre.x,
            req.centre.z,
            req.builder.def.0,
            req.builder.footprint.x,
            req.builder.footprint.z,
            req.builder_at.x,
            req.builder_at.z,
            req.facts.product_half,
            req.facts.product_passable.as_ref().map_or("none".to_string(), |w| format!("{} cells", w.iter().filter(|p| **p).count())),
        );
        for (i, b) in dump.builders.iter().enumerate() {
            let under = dump.standing.iter().find(|s| s.rect.contains(b.at));
            println!(
                "  builder {i}: at ({:.0}, {:.0}) half {:.0}, walk {}, {:.0} from the asker{}{}",
                b.at.x,
                b.at.z,
                b.half,
                b.passable.as_ref().map_or("the world's open".to_string(), |w| format!("{} cells", w.iter().filter(|p| **p).count())),
                b.at.dist2d(req.builder_at),
                if b.at.dist2d(req.builder_at) < 1.0 { " (the asker)" } else { "" },
                under.map_or(String::new(), |s| format!(", standing on {}", s.name)),
            );
        }
        if near {
            let mut close: Vec<&layout::Standing> = dump.standing.iter().filter(|s| layout::distance_to(&s.rect, req.centre) <= req.radius).collect();
            close.sort_by(|a, b| layout::distance_to(&a.rect, req.centre).total_cmp(&layout::distance_to(&b.rect, req.centre)));
            println!("standing within {:.0} of the centre: {}", req.radius, close.len());
            for s in close {
                println!("  {} ({:.0}..{:.0}, {:.0}..{:.0}) {:.0} from the centre{}", s.kind, s.rect.x0, s.rect.x1, s.rect.z0, s.rect.z1, layout::distance_to(&s.rect, req.centre), if s.bank { ", a bank" } else { "" });
            }
        }
        let mut world = dump.world();
        if !without_kinds.is_empty() {
            let before = world.standing.len();
            world.standing.retain(|s| !without_kinds.iter().any(|k| **k == s.kind));
            println!("without {}: {} of {} standing kept", without_kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", "), world.standing.len(), before);
        }
        if without_asker {
            let before = world.builders.len();
            world.builders.retain(|b| b.at.dist2d(req.builder_at) >= 1.0);
            println!("without the asker: {} of {} builders kept", world.builders.len(), before);
        }
        let mut trace = Vec::new();
        let started = Instant::now();
        let answer = choose_traced(&world, req, Some(&mut trace));
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        match &answer {
            Ok(site) => println!("replay: site at ({:.0}, {:.0}) facing {} ({ms:.0} ms): {}", site.at.x, site.at.z, site.facing, site.words),
            Err(why) => println!("replay: {why} ({ms:.0} ms)"),
        }
        for line in &trace {
            println!("  {line}");
        }
        // The same world gives the same verdicts; only the widenings' times differ between the game and here.
        let verdicts = |t: &[String]| t.iter().filter(|l| !l.starts_with("widening")).cloned().collect::<Vec<String>>();
        if !without_asker && without_kinds.is_empty() && verdicts(&trace) != verdicts(&dump.trace) {
            println!("the game's trace differed:");
            for line in &dump.trace {
                println!("  {line}");
            }
        }
    }
}
