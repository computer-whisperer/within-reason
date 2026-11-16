//! The game's tape, captured during an arena game (docs/design/2026-11-15-game-tape.md §8, 4a and 4b). The tape widget
//! (its only copy is the replays' template's) rides in the engine's spectating host as it does in a replay. While the
//! game runs, a follower appends each slice the raw stream has passed to `game.tape`, so the viewer can watch the game
//! in 3D as it is played; after it, the raw stream is packed into a compact `game.tape` in its place and removed. The
//! models, textures and map files the widget copies out of the engine's archives go into the cache the viewer serves
//! them from as they come.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Where a widget may write under the engine's write directory (the game's `userWritableFolders`).
const WRITTEN: &str = "LuaUI/Config/within-reason";
const WIDGET: &str = "run/replay-template/LuaUI/Widgets/tape_dump.lua";
/// The asset cache, shared by every tape (`run/view_match.py` serves it as `/assets/`).
const CACHE: &str = "run/data/tape-assets";
/// The follower's shortest wait between two reads of the raw stream.
const FOLLOW_EVERY: Duration = Duration::from_secs(2);

/// Puts the tape widget and its marker into a match directory, with the names the cache already holds (the widget
/// copies only the others).
pub fn begin(repo: &Path, dir: &Path) -> io::Result<()> {
    fs::copy(repo.join(WIDGET), dir.join("LuaUI/Widgets/tape_dump.lua"))?;
    let written = dir.join(WRITTEN);
    fs::create_dir_all(written.join("assets"))?;
    fs::write(written.join("tape.on"), "")?;
    let cache = repo.join(CACHE);
    fs::create_dir_all(&cache)?;
    let mut names: Vec<String> = fs::read_dir(&cache)?.flatten().filter_map(|e| e.file_name().into_string().ok()).collect();
    names.sort();
    fs::write(written.join("assets.have"), names.iter().map(|n| format!("{n}\n")).collect::<String>())
}

/// The widget's asset copies into the cache (a name the cache has is the same file: they are named by content).
/// Says how many were new. A copy is moved while the widget may still be writing it: the rename keeps the file the
/// widget writes to, and the tape names it only after it is closed.
fn move_assets(repo: &Path, dir: &Path) -> io::Result<u32> {
    let copies = dir.join(WRITTEN).join("assets");
    let cache = repo.join(CACHE);
    let mut new = 0;
    if copies.is_dir() {
        for entry in fs::read_dir(&copies)?.flatten() {
            let to = cache.join(entry.file_name());
            if to.exists() {
                fs::remove_file(entry.path())?;
            } else {
                fs::rename(entry.path(), to)?;
                new += 1;
            }
        }
    }
    Ok(new)
}

/// The follower of a game being played: stopped (and waited for) before `finish`.
pub struct Follower {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<String>>,
}

impl Follower {
    /// Stops it and says what it did.
    pub fn stop(mut self) -> String {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.take().and_then(|t| t.join().ok()).unwrap_or_default()
    }
}

/// Follows the raw stream while the game runs: the new asset copies into the cache, then the stream read again and
/// the slices it has passed appended to `game.tape` (`tape::pack::Growing`). It waits at least `FOLLOW_EVERY` between
/// reads and at least as long as the last read took, so it never takes more than half a core from the game.
pub fn follow(repo: &Path, dir: &Path) -> Follower {
    let stop = Arc::new(AtomicBool::new(false));
    let (repo, dir): (PathBuf, PathBuf) = (repo.into(), dir.into());
    let flag = stop.clone();
    let thread = std::thread::spawn(move || {
        let raw_path = dir.join(WRITTEN).join("tape.raw");
        let mut growing = tape::pack::Growing::new(dir.join("game.tape"));
        let (mut reads, mut busy, mut last_size, mut failed) = (0u32, Duration::ZERO, 0u64, None::<String>);
        let mut wait = FOLLOW_EVERY;
        let mut next = Instant::now() + wait;
        while !flag.load(Ordering::Relaxed) {
            if Instant::now() < next {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            let started = Instant::now();
            let _ = move_assets(&repo, &dir);
            let size = fs::metadata(&raw_path).map_or(0, |m| m.len());
            if size > last_size && failed.is_none() {
                last_size = size;
                let grown = fs::read(&raw_path).map_err(|e| e.to_string()).and_then(|raw| tape::game::Game::ingest(&raw).map_err(|e| e.to_string())).and_then(|game| growing.grow(&game).map_err(|e| e.to_string()));
                if let Err(e) = grown {
                    failed = Some(e);
                }
                reads += 1;
            }
            let took = started.elapsed();
            busy += took;
            wait = FOLLOW_EVERY.max(took);
            next = Instant::now() + wait;
        }
        match failed {
            Some(e) => format!("the tape followed the game for {} slices, then stopped: {e}", growing.slices()),
            None => format!("the tape followed the game: {} slices in {reads} reads, {:.1} s of work", growing.slices(), busy.as_secs_f32()),
        }
    });
    Follower { stop, thread: Some(thread) }
}

/// After the game: the widget's last asset copies into the cache, and the raw stream packed into a compact
/// `game.tape` that replaces the growing one in one rename (a reader following it reads the new footer), then
/// removed (kept when it does not pack). Says what was made.
pub fn finish(repo: &Path, dir: &Path) -> io::Result<String> {
    let new_assets = move_assets(repo, dir)?;
    let raw_path = dir.join(WRITTEN).join("tape.raw");
    let raw = fs::read(&raw_path).map_err(|e| io::Error::other(format!("no raw stream ({e}): see engine.log for the tape dump's lines")))?;
    let started = Instant::now();
    let kept = || format!("the raw stream is kept: target/release/tape pack {}", raw_path.display());
    let game = tape::game::Game::ingest(&raw).map_err(|e| io::Error::other(format!("the raw stream does not read: {e}; {}", kept())))?;
    let (file, _) = tape::pack::pack(&game).map_err(|e| io::Error::other(format!("the tape does not pack: {e}; {}", kept())))?;
    let unknown = if game.unknown.is_empty() { String::new() } else { format!("; passed over, records this build does not know (the widget is newer: rebuild and pack the raw stream again): {:?}", game.unknown) };
    let temp = dir.join("game.tape.new");
    fs::write(&temp, &file)?;
    fs::rename(&temp, dir.join("game.tape"))?;
    // (a stream with records this build passed over is kept, for a newer build to pack whole)
    if unknown.is_empty() {
        fs::remove_file(&raw_path)?;
    }
    Ok(format!(
        "game.tape {:.1} MB from {:.1} MB of raw stream, {:.1} game minutes, packed in {:.1} s; {new_assets} new assets at the end{unknown}",
        file.len() as f64 / 1e6,
        raw.len() as f64 / 1e6,
        game.last_frame as f64 / 1800.0,
        started.elapsed().as_secs_f64()
    ))
}
