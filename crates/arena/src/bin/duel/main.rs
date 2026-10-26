//! Unit-against-unit duels: spawns two armies on flat ground in a headless match where both teams are our AI,
//! sends them at each other and records who is left. Many duels run per engine start, at several sites at once.
//! This process is the bot for both teams: the shims connect to it instead of `bot`. See `docs/harness/duels.md`.
//!
//! usage: duel (--units a,b,c | --ours a,b --theirs c,d | --pairs a:b,c:d) [--reps N] [--budget METAL | --count N]
//!             [--parallel N] [--sites N] [--duels-per-match N] [--time-limit SECONDS] [--sweep-waves N] [--spacing ELMOS] [--formation X[/Y],...] [--lane X[/Y]] [--speed N] [--map NAME]
//!             [--label TEXT] [--base-port N]
//!        duel --scenario FILE [--reps N] [--parallel N] [--time-limit SECONDS] [--lane X[/Y]] [--speed N] [--label TEXT] [--base-port N]
//!             [--script FILE [--script-delay S]] [--theirs director|track] [--economy bare|fed]
//!             (a recorded engagement, `scenario.rs`, fought `--reps` times on its own map and place; side 0 under a
//!             timed script, side 1 on its recorded track)
//!        --seeds a,b,c: every duel `--reps` times under each engine seed (`FixedRNGSeed`), not only the match's own.
//!        A side of a pairing is a unit name (the count from --budget/--count) or a mixed force `name*count+name*count`.
//!        --lane off|old|on|ranks per side: whether the control lane (`crates/micro`) drives the army as a declared group over the director's orders (`ranks`: the group without pacing).
//!        duel --report DIR [duels.csv ...]   (rebuild the tables in DIR, from its own duels.csv or the files named)

mod director;
mod drill;
mod fire;
mod plan;
mod raid;
mod report;
mod scenario;
mod script;
mod sites;

use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arena::autohost::{Autohost, Event};
use arena::harness::{
    ENGINE_MEMORY_GB, GAME_TAG, LOAD_ALLOWANCE, REPO, available_memory_gb, copy_tree, git_commit, refresh_cache_template,
    resolve_game, stop,
};
use bot_protocol::{Commands, FrameReader, ToBot, write_frame};

use director::{Batch, Control, Director};
use plan::{Job, LaneMode, Sizing};
use scenario::{Scenario, Script};
use sites::Formation;

/// A match whose director hears nothing for this long has hung.
const STALL_ALLOWANCE: Duration = Duration::from_secs(120);
/// A duel is given up after its match aborted this many times under it.
const MAX_ATTEMPTS: u32 = 2;

struct Options {
    pairs: Vec<(String, String)>,
    reps: u32,
    sizing: Sizing,
    parallel: usize,
    sites: usize,
    duels_per_match: u32,
    time_limit: i32,
    sweep_waves: u32,
    /// Elmos between neighbours for a formation that names none.
    spacing: f32,
    /// How the two armies stand, one entry per shape the batch fights every pairing in (`--formation`).
    shapes: Vec<[Formation; 2]>,
    /// Whether the control lane drives each side (`x`, `y`).
    lanes: [LaneMode; 2],
    speed: u32,
    map: String,
    label: String,
    base_port: u16,
    /// `--scenario`: this engagement instead of pairings, and the start boxes that keep the commanders away from it.
    scenario: Option<Scenario>,
    /// `--chase N`: the chase instrument (`director.rs` `chase_orders`).
    chase: Option<i32>,
    /// `--scenario raid`: the raid scenario (`raid.rs`).
    raid: bool,
    boxes: [script::StartBox; 2],
    /// `--script FILE`: timed orders for side 0 of the scenario; `--script-delay`.
    script: Option<Script>,
    script_delay: f32,
    /// `--theirs director|track`: who orders side 1 of the scenario.
    theirs: Control,
    /// `--seeds`: the engine seeds each duel is fought under.
    seeds: Vec<u32>,
    /// `--economy fed`: a free fusion for each team (`director.rs` `FEEDER`).
    fed: bool,
    /// `--drill`: every job a drill (`drill.rs`), `--ours` the army and nobody on the other side.
    drill: Option<drill::Options>,
}

fn main() -> io::Result<()> {
    let options = parse_args()?;
    let repo = fs::canonicalize(REPO)?;
    let status = Command::new(repo.join("run/install_ai.sh")).stdout(Stdio::null()).status()?;
    if !status.success() {
        return Err(io::Error::other("install_ai.sh failed"));
    }
    let needed_gb = options.parallel as u64 * ENGINE_MEMORY_GB;
    let available_gb = available_memory_gb()?;
    if available_gb < needed_gb {
        return Err(io::Error::other(format!("{} engines need about {needed_gb} GB, only {available_gb} GB available", options.parallel)));
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let batch_dir = repo.join(format!("run/matches/{stamp}-duel-{}", options.label));
    fs::create_dir_all(&batch_dir)?;
    let jobs = plan::jobs(&options.pairs, options.reps, options.shapes.len(), &options.seeds);
    let shapes: Vec<String> = options.shapes.iter().map(|[x, y]| format!("{}/{}", x.label(), y.label())).collect();
    println!(
        "{} duels ({} pairings x {} shapes x {}) on {} -> {}",
        jobs.len(), options.pairs.len(), options.shapes.len(), options.reps, options.map, batch_dir.display()
    );
    fs::write(
        batch_dir.join("batch.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "label": options.label, "commit": git_commit(&repo), "map": options.map, "pairings": options.pairs.len(),
            "reps": options.reps, "sizing": format!("{:?}", options.sizing), "time_limit": options.time_limit,
            "speed": options.speed, "sweep_waves": options.sweep_waves, "spacing": options.spacing, "formations": shapes, "lanes": [options.lanes[0].label(), options.lanes[1].label()], "sites": options.sites, "duels_per_match": options.duels_per_match,
            "scenario": options.scenario.as_ref().map(|s| s.centre), "chase": options.chase, "raid": options.raid,
            "script": options.script.as_ref().map(|s| s.label.clone()), "script_delay": options.script_delay,
            "theirs": options.theirs.label(), "seeds": options.seeds, "economy": if options.fed { "fed" } else { "bare" },
            "drill": options.drill.as_ref().map(|d| serde_json::json!({ "kind": d.kind.label(), "distance": d.distance, "order": if d.fight { "fight" } else { "move" }, "reissue_s": d.reissue.map(|f| f as f32 / drill::FPS as f32), "lead": d.lead, "pace": d.pace, "turn_at_s": d.turn_at / drill::FPS, "lab": d.lab, "target": d.target, "from": d.from, "to": d.to, "around": d.around, "feed": d.feed.iter().map(|(n, c)| format!("{n}*{c}")).collect::<Vec<_>>().join("+"), "laps": d.laps, "circuit": d.circuit })),
            "argv": std::env::args().collect::<Vec<_>>(),
            "disabled": std::env::var("WITHIN_REASON_DISABLE").ok(), "form_spacing": std::env::var("WITHIN_REASON_FORM_SPACING").ok(),
        }))?,
    )?;

    let csv_path = batch_dir.join("duels.csv");
    fs::write(&csv_path, format!("{}\n", report::HEADER))?;
    let csv = Mutex::new(OpenOptions::new().append(true).open(&csv_path)?);
    let log_dir = (options.scenario.is_some() || options.drill.is_some()).then(|| batch_dir.join("seconds"));
    if let Some(dir) = &log_dir {
        fs::create_dir_all(dir)?;
    }
    let total = jobs.len();
    let done = AtomicUsize::new(0);
    let batch = Arc::new(Batch {
        queue: Mutex::new(VecDeque::from(jobs)),
        results: Mutex::new(Vec::new()),
        sizing: options.sizing,
        time_limit: options.time_limit,
        sweep_waves: options.sweep_waves,
        shapes: options.shapes.clone(),
        lanes: options.lanes,
        units: combatsim::units::Units::default(),
        scenario: options.scenario.clone(),
        chase: options.chase,
        raid: options.raid,
        script: options.script.clone(),
        script_delay: options.script_delay,
        theirs: options.theirs,
        fed: options.fed,
        drill: options.drill.clone(),
        log_dir,
        on_result: Box::new(move |result| {
            // Written as they finish, so an interrupted batch keeps what it has.
            let _ = writeln!(csv.lock().unwrap(), "{}", report::row(result));
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if !result.drill.kind.is_empty() {
                let d = &result.drill;
                let s = |v: Option<f32>| v.map_or("-".to_string(), |v| format!("{v:.0}"));
                println!(
                    "[{n}/{total}] drill {}: {}x{} ({}, lane {}){}: {} {:.0}s; arrived first/median/last {}/{}/{} s, spread {}, nearest friend {}, speed share {}, spin-up {} s, at contact {} deep x {} wide, ramp {}/{} s{}{}{}",
                    d.kind, result.count[0], result.job.x, result.formation[0], result.lane[0],
                    result.seed.map_or(String::new(), |s| format!(" seed {s}")), result.reason, result.seconds,
                    s(d.arrive[0]), s(d.arrive[1]), s(d.arrive[2]), s(d.spread_arrival), s(d.nn_arrival),
                    d.speed_share.map_or("-".to_string(), |v| format!("{v:.2}")), s(d.spin_up_s),
                    s(d.depth_contact), s(d.width_contact), d.ramp_half_s.map_or("-".to_string(), |v| format!("{v:.1}")), d.ramp_all_s.map_or("-".to_string(), |v| format!("{v:.1}")),
                    d.recover_s.map_or(String::new(), |v| format!(", recovered {v:.0} s after the turn")),
                    if d.kind == "gather" { format!(", produced {}, join median {} s", d.produced, s(d.join_median_s)) } else { String::new() }
                        + &if d.kind == "circuit" { format!(", laps {}, formed share {}, produced {}, joined {} (median {} s), left behind {}", d.laps, d.formed_share.map_or("-".to_string(), |v| format!("{v:.2}")), d.produced, d.joined, s(d.join_median_s), d.left_behind) } else { String::new() }
                        + &if d.companies > 1 { format!(", {} companies (their formed share {})", d.companies, d.company_formed.map_or("-".to_string(), |v| format!("{v:.2}"))) } else { String::new() },
                    if d.kind == "into" { format!(", first shot {} s, shot ramp {}/{} s, target died {}x", s(d.first_shot_s), s(d.shot_ramp_half_s), s(d.shot_ramp_all_s), d.target_deaths) } else { String::new() }
                );
                return;
            }
            let muzzled = |i: usize| 100.0 * result.fire[i].muzzled_seconds() as f32 / result.fire[i].reach_seconds.max(1) as f32;
            println!(
                "[{n}/{total}] {}x{} ({}, lane {}) vs {}x{} ({}, lane {}){}: {} ({}) {:.0}s, left {:.0}% / {:.0}% (at 30 s {:.0}% / {:.0}%), muzzled {:.0}% / {:.0}%, nearest friend {:.0} / {:.0}",
                result.count[0], result.job.x, result.formation[0], result.lane[0], result.count[1], result.job.y, result.formation[1], result.lane[1],
                result.seed.map_or(String::new(), |s| format!(" seed {s}")),
                result.winner, result.reason, result.seconds, 100.0 * result.value_left[0], 100.0 * result.value_left[1],
                100.0 * result.left_at[0][0], 100.0 * result.left_at[0][1],
                muzzled(0), muzzled(1), result.shape[0].median_nearest(), result.shape[1].median_nearest()
            );
        }),
    });

    let started = Instant::now();
    let options = Arc::new(options);
    let next_match = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..options.parallel)
        .map(|_| {
            let (batch, options, repo, batch_dir, next_match) =
                (batch.clone(), options.clone(), repo.clone(), batch_dir.clone(), next_match.clone());
            std::thread::spawn(move || {
                let mut failures = 0;
                while !batch.queue.lock().unwrap().is_empty() && failures < 3 {
                    let index = next_match.fetch_add(1, Ordering::Relaxed);
                    match run_match(&repo, &batch_dir, &options, &batch, index) {
                        Ok(()) => failures = 0,
                        Err(e) => {
                            eprintln!("match {index}: {e}");
                            failures += 1;
                        }
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    refresh_cache_template(&repo, &batch_dir);

    let left = batch.queue.lock().unwrap().len();
    let ran = batch.results.lock().unwrap().len();
    println!("== {ran} duels in {:.0}s over {} matches; {left} not run ==", started.elapsed().as_secs_f32(), next_match.load(Ordering::Relaxed));
    report::write(&[&csv_path], &batch_dir)?;
    println!("{}\n{}", csv_path.display(), batch_dir.join("matrix.md").display());
    Ok(())
}

/// One engine start: duels run until the queue is empty or the match has had its share.
fn run_match(repo: &Path, batch_dir: &Path, options: &Options, batch: &Arc<Batch>, index: usize) -> io::Result<()> {
    // Under `--seeds` the match takes the seed of the first job waiting, and only that seed's jobs; else its own.
    let seed = if options.seeds.is_empty() {
        None
    } else {
        match batch.queue.lock().unwrap().front() {
            Some(job) => job.seed,
            None => return Ok(()),
        }
    };
    let dir = batch_dir.join(format!("{index:02}"));
    fs::create_dir_all(&dir)?;
    let host_port = options.base_port + 2 * index as u16;
    copy_tree(&repo.join("run/match-template"), &dir)?;
    let cache_template = repo.join("run/cache-template");
    if cache_template.is_dir() {
        copy_tree(&cache_template, &dir.join("cache"))?;
    }
    let script_path = dir.join("script.txt");
    let engine_seed = seed.unwrap_or(index as u32 + 1);
    fs::write(&script_path, script::render(&resolve_game(repo, GAME_TAG)?, &options.map, host_port, host_port + 1, engine_seed, options.boxes))?;

    let mut autohost = Autohost::bind(host_port + 1)?;
    // Unix socket paths are limited to ~108 bytes, so the socket cannot live in the match directory.
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, Into::into);
    let socket = runtime_dir.join(format!("wreason-duel-{}-{index}.sock", std::process::id()));
    let _ = fs::remove_file(&socket);
    let director = Arc::new(Mutex::new(Director::new(batch.clone(), index, options.sites, options.duels_per_match, seed)));
    let closing = Arc::new(AtomicBool::new(false));
    let listener = serve(&socket, director.clone(), closing.clone())?;

    let log = File::create(dir.join("engine.log"))?;
    let mut engine = Command::new(repo.join("run/engine/spring-headless"))
        .args(["--isolation", "--write-dir"])
        .arg(&dir)
        .arg(&script_path)
        .env("SPRING_DATADIR", repo.join("run/data"))
        .env("WITHIN_REASON_SOCKET", &socket)
        // The shim waits for the director's answer to every tick, so orders land on the frame they were decided
        // for however fast the game runs.
        .env("WITHIN_REASON_LOCKSTEP", "1")
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;

    let mut playing = false;
    let load_deadline = Instant::now() + LOAD_ALLOWANCE;
    let outcome = loop {
        if engine.try_wait()?.is_some() {
            break Err(io::Error::other("engine exited; see engine.log"));
        }
        {
            let director = director.lock().unwrap();
            if let Some(problem) = &director.fatal {
                // Nothing a retry would cure: drop the plan so the workers stop.
                batch.queue.lock().unwrap().clear();
                break Err(io::Error::other(problem.clone()));
            }
            if director.finished {
                break Ok(());
            }
            if playing && director.last_tick.elapsed() > STALL_ALLOWANCE {
                break Err(io::Error::other("no tick from the engine for two minutes"));
            }
        }
        if !playing && Instant::now() > load_deadline {
            break Err(io::Error::other("engine did not start playing"));
        }
        match autohost.receive(Duration::from_millis(200))? {
            Some(Event::StartPlaying) => {
                // Raising the minimum forces the server's speed up; the maximum has to allow it first.
                autohost.send(&format!("/setmaxspeed {}", options.speed))?;
                autohost.send(&format!("/setminspeed {}", options.speed))?;
                playing = true;
                director.lock().unwrap().last_tick = Instant::now();
            }
            Some(Event::GameOver { .. }) => break Err(io::Error::other("the game ended under the duels (a commander died?)")),
            Some(Event::Other) | None => {}
        }
    };
    stop(&mut engine, &mut autohost);
    closing.store(true, Ordering::Relaxed);
    let _ = listener.join();
    let _ = fs::remove_file(&socket);
    let mut queue = batch.queue.lock().unwrap();
    for job in director.lock().unwrap().unfinished() {
        if job.attempts + 1 < MAX_ATTEMPTS {
            queue.push_back(Job { attempts: job.attempts + 1, ..job });
        } else {
            eprintln!("gave up on {} vs {} (rep {})", job.x, job.y, job.rep);
        }
    }
    outcome
}

/// Accepts the two shims' connections and answers their ticks from the director until `closing` is set.
fn serve(socket: &Path, director: Arc<Mutex<Director>>, closing: Arc<AtomicBool>) -> io::Result<std::thread::JoinHandle<()>> {
    let listener = UnixListener::bind(socket)?;
    listener.set_nonblocking(true)?;
    Ok(std::thread::spawn(move || {
        while !closing.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let director = director.clone();
                    std::thread::spawn(move || {
                        // Ends with an error when the engine goes away, which is how every session ends.
                        let _ = session(stream, &director);
                    });
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(50)),
                Err(e) => return eprintln!("duel socket: {e}"),
            }
        }
    }))
}

fn session(mut stream: UnixStream, director: &Mutex<Director>) -> io::Result<()> {
    stream.set_nonblocking(false)?;
    let mut input = stream.try_clone()?;
    let mut reader = FrameReader::default();
    let mut next = move || reader.read::<ToBot>(&mut input).map(|m| m.expect("blocking socket"));
    let ToBot::Hello(hello) = next()? else {
        return Err(io::Error::other("expected Hello first"));
    };
    let team = hello.team;
    director.lock().unwrap().hello(hello);
    write_frame(&mut stream, &Commands::default())?;
    loop {
        let ToBot::Tick(tick) = next()? else {
            return Err(io::Error::other("unexpected second Hello"));
        };
        let commands = director.lock().unwrap().tick(team, &tick);
        write_frame(&mut stream, &Commands(commands))?;
    }
}

fn parse_args() -> io::Result<Options> {
    let mut options = Options {
        pairs: Vec::new(),
        reps: 4,
        sizing: Sizing::EqualMetal(1200.0),
        parallel: 2,
        sites: 3,
        duels_per_match: 45,
        time_limit: 240,
        sweep_waves: 3,
        spacing: 56.0,
        shapes: Vec::new(),
        lanes: [LaneMode::Off; 2],
        speed: 50,
        map: "Quicksilver Remake 1.24".into(),
        label: "batch".into(),
        base_port: 9500,
        scenario: None,
        chase: None,
        raid: false,
        boxes: script::DUEL_BOXES,
        script: None,
        script_delay: 0.0,
        theirs: Control::Director,
        seeds: Vec::new(),
        fed: false,
        drill: None,
    };
    let mut drill_kind: Option<drill::Kind> = None;
    let mut drill_opts = drill::Options { kind: drill::Kind::Walk, distance: 2000.0, fight: true, reissue: None, lead: None, pace: None, turn_at: 12 * drill::FPS, lab: "armvp".into(), target: "armshltx".into(), from: None, to: None, around: None, feed: Vec::new(), laps: 1, circuit: None, advance: false, behind: Vec::new(), behind_distance: 0.0, scatter: 0.0, churn: None, churn_def: "armsolar".into(), stall: None, crosser: None, nudge: None, late: None, cap: 0 };
    let mut lab_given = false;
    let mut script_path: Option<PathBuf> = None;
    let mut time_limit_given = false;
    let mut economy_given = false;
    let mut map_given = false;
    let list = |text: String| text.split(',').map(str::to_string).collect::<Vec<_>>();
    let point = |text: &str| -> (f32, f32) {
        let parts: Vec<f32> = text.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() != 2 {
            usage("a point is X,Z in elmos (--from, --to, --circuit)");
        }
        (parts[0], parts[1])
    };
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    let mut formations = String::from("ranks8");
    let mut lanes = String::from("off");
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        if flag == "--report" {
            let dir = PathBuf::from(args.next().unwrap_or_else(|| usage("--report needs a directory")));
            let named: Vec<PathBuf> = args.map(PathBuf::from).collect();
            let inputs = if named.is_empty() { vec![dir.join("duels.csv")] } else { named };
            report::write(&inputs.iter().map(PathBuf::as_path).collect::<Vec<_>>(), &dir)?;
            std::process::exit(0);
        }
        let mut value = || args.next().unwrap_or_else(|| usage(&format!("{flag} needs a value")));
        let number = |text: String| text.parse::<u32>().unwrap_or_else(|_| usage(&format!("{flag} takes a number")));
        match flag.as_str() {
            "--units" => options.pairs.extend(plan::all_pairs(&list(value()))),
            "--ours" => ours = list(value()),
            // Under a scenario `--theirs` names who orders side 1; elsewhere the other side's units.
            "--theirs" => match value().as_str() {
                "director" => options.theirs = Control::Director,
                "track" => options.theirs = Control::Track,
                "ai" => usage("--theirs ai is not built yet (phase 3 of docs/design/2026-09-28-tas-micro.md)"),
                other => theirs = list(other.to_string()),
            },
            "--script" => script_path = Some(PathBuf::from(value())),
            "--script-delay" => options.script_delay = value().parse().unwrap_or_else(|_| usage("--script-delay takes seconds")),
            "--seeds" => {
                options.seeds = list(value()).iter().map(|s| s.parse().unwrap_or_else(|_| usage("--seeds takes numbers"))).collect();
            }
            "--economy" => {
                economy_given = true;
                match value().as_str() {
                    "fed" => options.fed = true,
                    "bare" => options.fed = false,
                    _ => usage("--economy is bare or fed"),
                }
            }
            "--pairs" => {
                for pair in list(value()) {
                    let (x, y) = pair.split_once(':').unwrap_or_else(|| usage("--pairs takes x:y,x:y"));
                    options.pairs.push((x.to_string(), y.to_string()));
                }
            }
            "--reps" => options.reps = number(value()),
            "--budget" => options.sizing = Sizing::EqualMetal(number(value()) as f32),
            "--count" => options.sizing = Sizing::EqualCount(number(value())),
            "--parallel" => options.parallel = number(value()) as usize,
            "--sites" => options.sites = number(value()) as usize,
            "--duels-per-match" => options.duels_per_match = number(value()),
            "--time-limit" => {
                options.time_limit = number(value()) as i32;
                time_limit_given = true;
            }
            "--sweep-waves" => options.sweep_waves = number(value()),
            "--spacing" => options.spacing = number(value()) as f32,
            "--formation" => formations = value(),
            "--lane" => lanes = value(),
            "--chase" => options.chase = Some(number(value()) as i32),
            "--speed" => options.speed = number(value()),
            "--base-port" => options.base_port = number(value()) as u16,
            "--map" => {
                options.map = value();
                map_given = true;
            }
            "--scenario" => {
                let what = value();
                if what == "raid" {
                    options.raid = true;
                    continue;
                }
                let scenario = Scenario::load(std::path::Path::new(&what)).unwrap_or_else(|e| usage(&e));
                let [x, z] = scenario.centre;
                let [w, h] = scenario.map_size;
                options.boxes = script::farthest_corners(x / w.max(1.0), z / h.max(1.0));
                options.map = scenario.map.clone();
                options.pairs.push((scenario.sides[0].name.clone(), scenario.sides[1].name.clone()));
                options.scenario = Some(scenario);
            }
            "--drill" => drill_kind = Some(drill::Kind::parse(&value()).unwrap_or_else(|| usage("--drill is walk, reverse, gather, into, circuit or assault"))),
            "--feed" => {
                let force = plan::Force::parse(&value());
                if force.parts.iter().any(|(_, c)| *c == 0) {
                    usage("--feed takes name*count[+name*count]");
                }
                drill_opts.feed = force.parts;
            }
            "--laps" => drill_opts.laps = number(value()).max(1),
            "--circuit" => drill_opts.circuit = Some(value().split(';').map(point).collect()),
            "--target" => drill_opts.target = value(),
            "--from" => drill_opts.from = Some(point(&value())),
            "--to" => drill_opts.to = Some(point(&value())),
            "--around" => drill_opts.around = Some(value().parse::<f32>().unwrap_or_else(|_| usage("--around takes a ratio, such as 1.3"))),
            "--advance" => drill_opts.advance = true,
            // D21's arms (`docs/design/2026-10-24-formation-drills.md`).
            "--behind" => {
                let text = value();
                let (force, distance) = text.split_once('@').unwrap_or_else(|| usage("--behind takes name*count[+...]@DISTANCE, such as armpw*4@1600"));
                let force = plan::Force::parse(force);
                if force.parts.iter().any(|(_, c)| *c == 0) {
                    usage("--behind takes name*count[+...]@DISTANCE");
                }
                drill_opts.behind = force.parts;
                drill_opts.behind_distance = distance.parse::<f32>().unwrap_or_else(|_| usage("--behind's distance is elmos, such as 1600"));
            }
            "--scatter" => drill_opts.scatter = value().parse::<f32>().unwrap_or_else(|_| usage("--scatter takes elmos, such as 1200")),
            "--churn" => {
                let seconds = value().parse::<f32>().unwrap_or_else(|_| usage("--churn takes seconds, such as 5"));
                drill_opts.churn = Some((seconds * drill::FPS as f32).round() as i32).filter(|f| *f > 0);
            }
            "--churn-def" => drill_opts.churn_def = value(),
            "--cap" => drill_opts.cap = number(value()) as usize,
            "--late" => {
                let seconds = value().parse::<f32>().unwrap_or_else(|_| usage("--late takes seconds, such as 4"));
                drill_opts.late = Some((seconds * drill::FPS as f32).round() as i32).filter(|f| *f >= 0);
            }
            "--nudge" => {
                let seconds = value().parse::<f32>().unwrap_or_else(|_| usage("--nudge takes seconds, such as 5"));
                drill_opts.nudge = Some((seconds * drill::FPS as f32).round() as i32).filter(|f| *f > 0);
            }
            "--stall" => {
                let text = value();
                let (at, for_s) = text.split_once(',').unwrap_or_else(|| usage("--stall takes AT,FOR in seconds, such as 8,6"));
                let parse = |t: &str| t.trim().parse::<f32>().unwrap_or_else(|_| usage("--stall takes AT,FOR in seconds, such as 8,6"));
                drill_opts.stall = Some(((parse(at) * drill::FPS as f32).round() as i32, (parse(for_s) * drill::FPS as f32).round() as i32));
            }
            "--crosser" => {
                let text = value();
                let (name, rest) = text.split_once('@').unwrap_or_else(|| usage("--crosser takes NAME@ALONG,ACROSS, such as armfav@700,500"));
                let (along, across) = rest.split_once(',').unwrap_or_else(|| usage("--crosser takes NAME@ALONG,ACROSS, such as armfav@700,500"));
                let parse = |t: &str| t.trim().parse::<f32>().unwrap_or_else(|_| usage("--crosser's places are elmos, such as 700,500"));
                drill_opts.crosser = Some((name.to_string(), parse(along), parse(across)));
            }
            "--distance" => drill_opts.distance = number(value()) as f32,
            "--order" => {
                drill_opts.fight = match value().as_str() {
                    "fight" => true,
                    "move" => false,
                    _ => usage("--order is fight or move"),
                }
            }
            "--reissue" => {
                let seconds = value().parse::<f32>().unwrap_or_else(|_| usage("--reissue takes seconds, such as 2 or 0.1"));
                drill_opts.reissue = Some((seconds * drill::FPS as f32).round() as i32).filter(|f| *f > 0);
            }
            "--lead" => drill_opts.lead = Some(value().parse::<f32>().unwrap_or_else(|_| usage("--lead takes elmos, such as 100"))).filter(|l| *l > 0.0),
            "--pace" => drill_opts.pace = Some(value().parse::<f32>().unwrap_or_else(|_| usage("--pace takes elmos a second, such as 54"))).filter(|p| *p > 0.0),
            "--turn-at" => drill_opts.turn_at = number(value()) as i32 * drill::FPS,
            "--lab" => {
                drill_opts.lab = value();
                lab_given = true;
            }
            "--label" => options.label = value(),
            _ => usage(&format!("unknown argument {flag}")),
        }
    }
    if let Some(kind) = drill_kind {
        // A drill (`drill.rs`): `--ours` names the army (a gather: the list its lab builds), nobody on the other side,
        // on the duel's map (Quicksilver offers a 3,200 strip, Comet Catcher 1,920); a gather gets the time its list
        // takes to build and the feeder's energy (the resource cheat gives metal only; `director.rs` `drill_orders`).
        if ours.is_empty() {
            usage("--drill needs --ours (the army, or the list a gather's lab builds)");
        }
        let assault = kind == drill::Kind::Assault;
        if !assault && !theirs.is_empty() {
            usage("--drill fights nobody: no --theirs");
        }
        if assault && theirs.is_empty() {
            usage("--drill assault needs --theirs DEFENCE (armed buildings with counts: armllt*2+armbeamer*1)");
        }
        if drill_opts.advance && !assault {
            usage("--advance is the assault's: theirs march on the army's start");
        }
        if assault && ours.iter().chain(&theirs).any(|f| !plan::Force::parse(f).counted) {
            usage("--drill assault takes both forces with counts (armflash*12 against armhlt*1): nothing is sized against the other side");
        }
        if (drill_opts.lead.is_some() || drill_opts.pace.is_some()) && drill_opts.reissue.is_none() {
            usage("--lead and --pace need --reissue (the cadence at which the point is re-placed)");
        }
        if drill_opts.from.is_some() != drill_opts.to.is_some() {
            usage("--from and --to go together");
        }
        if drill_opts.from.is_some() && drill_opts.around.is_some() {
            usage("--from/--to or --around, not both");
        }
        if drill_opts.from.is_some() || kind == drill::Kind::Circuit {
            options.sites = 1;
        }
        if kind == drill::Kind::Circuit && !lab_given {
            // A circuit's default army is Pawns, so its lab is the bot lab.
            drill_opts.lab = "armlab".into();
        }
        if kind != drill::Kind::Circuit && !drill_opts.feed.is_empty() {
            usage("--feed, --laps and --circuit belong to --drill circuit");
        }
        let has_lab = kind == drill::Kind::Gather || (kind == drill::Kind::Circuit && !drill_opts.feed.is_empty());
        options.drill = Some(drill::Options { kind, ..drill_opts });
        options.pairs = if assault { plan::cross(&ours, &theirs) } else { ours.drain(..).map(|o| (o, plan::NOBODY.to_string())).collect() };
        if assault {
            ours.clear();
            theirs.clear();
        }
        options.boxes = script::DRILL_BOXES;
        if !time_limit_given {
            options.time_limit = match kind {
                drill::Kind::Gather => 480,
                drill::Kind::Circuit => 600,
                drill::Kind::Assault => 300,
                _ => 240,
            };
        }
        if has_lab && !economy_given {
            options.fed = true;
        }
    }
    if options.raid {
        // The raid's default sides (`raid.rs`): four Rovers and two Blitzes at the station, a Tick and a Pawn raiding;
        // three minutes on the flat map.
        if ours.is_empty() {
            ours = vec!["armfav*4+armflash*2".to_string()];
        }
        if theirs.is_empty() {
            theirs = vec!["armflea*1+armpw*1".to_string()];
        }
        if !time_limit_given {
            options.time_limit = 180;
        }
        if !map_given {
            options.map = "Comet Catcher Remake 1.8".into();
        }
    }
    options.pairs.extend(plan::cross(&ours, &theirs));
    // `X` or `X/Y` per shape, comma-separated: X for the first army of each pairing, Y (default ranks of eight) for
    // the second; each at `--spacing` unless it names its own with `@`.
    for shape in formations.split(',') {
        let (x, y) = shape.split_once('/').unwrap_or((shape, "ranks8"));
        let parse = |text: &str| {
            Formation::parse(text, options.spacing).unwrap_or_else(|| usage(&format!("--formation: {text} is not ranksN, line, or either @SPACING")))
        };
        options.shapes.push([parse(x), parse(y)]);
    }
    // `X` or `X/Y`: the lane for the first army of each pairing, and for the second (default off).
    let (lane_x, lane_y) = lanes.split_once('/').unwrap_or((lanes.as_str(), "off"));
    let parse_lane = |text: &str| LaneMode::parse(text).unwrap_or_else(|| usage(&format!("--lane: {text} is not off, old, on or ranks")));
    options.lanes = [parse_lane(lane_x), parse_lane(lane_y)];
    if options.pairs.is_empty() {
        usage("no pairings: give --units, --ours with --theirs, --pairs, or --scenario");
    }
    if options.scenario.is_some() && (options.pairs.len() != 1 || options.shapes.len() != 1) {
        usage("--scenario fights its own sides: no pairings and no formations beside it");
    }
    if let Some(path) = script_path {
        let Some(scenario) = &options.scenario else { usage("--script needs --scenario FILE") };
        options.script = Some(Script::load(&path, scenario).unwrap_or_else(|e| usage(&e)));
        if options.lanes[0] != LaneMode::Off {
            eprintln!("--script: side 0 is the script's alone; its lane is off");
            options.lanes[0] = LaneMode::Off;
        }
    }
    if options.theirs == Control::Track && options.scenario.as_ref().is_none_or(|s| s.sides[1].track.is_empty()) {
        usage("--theirs track needs a scenario whose side 1 has a track (run/engagement.py --track)");
    }
    Ok(options)
}

fn usage(problem: &str) -> ! {
    eprintln!("{problem}\nusage: duel (--units a,b,c | --ours a,b --theirs c,d | --pairs a:b,c:d) [--reps N] [--budget METAL | --count N] [--parallel N] [--sites N] [--duels-per-match N] [--time-limit SECONDS] [--sweep-waves N] [--spacing ELMOS] [--formation X[/Y],...] [--lane off|old|on|ranks[/...]] [--speed N] [--map NAME] [--label TEXT] [--base-port N]\n       duel --scenario FILE [--reps N] [--parallel N] [--time-limit SECONDS] [--lane X[/Y]] [--speed N] [--label TEXT] [--base-port N]
            [--script FILE [--script-delay S]] [--theirs director|track] [--economy bare|fed] [--seeds a,b,c]
       duel --drill walk|reverse|gather|into --ours FORCE[,FORCE] [--distance 2000] [--order fight|move] [--reissue S] [--lead ELMOS] [--pace ELMOS/S] [--turn-at 12] [--lab armvp] [--target armshltx] [--from X,Z --to X,Z | --around RATIO] [--count N] [--formation X] [--lane off|old|on|ranks] [--cap N] [--reps N] [--seeds a,b] [--time-limit S] [--map NAME] [--label TEXT]
            (one army walks the strip, no enemy: docs/design/2026-10-24-formation-drills.md)
       duel --drill circuit --ours FORCE [--feed name*count] [--laps 1] [--circuit X,Z;X,Z;...] [--lab armlab] [--order fight|move] [--lane off|on] ...
            (a deployed army round the map while a lab feeds it: design D12)
       duel --drill assault --ours FORCE --theirs DEFENCE [--distance 2500] [--order fight|move] [--lane off|old|on|ranks[/off|on]] [--formation X] [--from X,Z --to X,Z | --around RATIO] [--advance] ...
            (the walk into a defence that shoots back, judged as a duel: design D16; both forces with counts, armflash*12 against armhlt*1;
             --advance: theirs march on the army's start under the second lane mode, design D18)
       duel --report DIR [duels.csv ...]");
    std::process::exit(2)
}
