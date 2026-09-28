//! Bot process. Listens on the socket the AI shims connect to and runs one session per AI.

mod brain;
mod recorder;
mod strategist;
mod texts;
mod team;
mod world;

use std::io;
use std::os::unix::net::{UnixListener, UnixStream};

use bot_protocol::{Commands, FrameReader, ToBot, socket_path, write_frame};

use brain::Brain;
use strategist::Strategist;
use world::World;

/// usage: bot [--player] [--pianist], or bot --plan-replay <match dir> <m:ss> [...] (`brain/pianist/replay.rs`)
/// `--pianist`: Jev plays every unit from the player's instructions (`docs/design/2026-09-21-pianist.md`).
/// `--player`: a Claude Code session beside the brains (see `DESIGN.md`), the Opus player whose lever is the
/// pianist's instructions; one session serves every seat we play on a team (`strategist/seats.rs`).
/// Transcripts go to `$WITHIN_REASON_LOG_DIR`, else the current directory.
fn main() -> io::Result<()> {
    let mut player = false;
    let mut pianist = false;
    // `bot --plan-replay <match dir> <m:ss> ...`: the engagement plan on a recorded moment (`run/plan_replay.py`).
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|a| a == "--plan-replay") {
        return brain::pianist::replay::plan_replay(&arguments[1..]).map_err(io::Error::other);
    }
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--player" => player = true,
            "--pianist" => pianist = true,
            other => return Err(io::Error::other(format!("unknown argument {other}; usage: bot [--player] [--pianist]"))),
        }
    }
    if player && !pianist {
        return Err(io::Error::other("--player is the pianist's player: give --pianist too"));
    }
    let path = socket_path();
    // A previous run may have left its socket file behind; nothing can be listening on it.
    if UnixStream::connect(&path).is_err() {
        let _ = std::fs::remove_file(&path);
    }
    let listener = UnixListener::bind(&path)?;
    eprintln!("listening on {}", path.display());
    for stream in listener.incoming() {
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = session(stream, player, pianist) {
                eprintln!("session ended: {e}");
            }
        });
    }
    Ok(())
}

pub(crate) fn log_dir() -> std::path::PathBuf {
    std::env::var_os("WITHIN_REASON_LOG_DIR").map_or_else(|| ".".into(), Into::into)
}

/// `player`: whether the Opus player's session is started (one per team, `strategist/seats.rs`).
fn session(mut stream: UnixStream, player: bool, pianist: bool) -> io::Result<()> {
    let mut input = stream.try_clone()?;
    let mut reader = FrameReader::default();
    let mut next = move || reader.read::<ToBot>(&mut input).map(|m| m.expect("blocking socket"));

    let ToBot::Hello(hello) = next()? else {
        return Err(io::Error::other("expected Hello first"));
    };
    // A player session that fails to start is not fatal: the hands play alone.
    let board = team::TeamBoard::of(&hello);
    let strategist = player
        .then(|| board.strategist(|| Strategist::start(&log_dir(), hello.ai_id).map(std::sync::Arc::new)))
        .and_then(|started| started.inspect_err(|e| eprintln!("the player's session failed to start: {e}")).ok());
    let mode_name = match (player, pianist) {
        (true, _) => "player",
        (false, true) => "pianist",
        (false, false) => "none",
    };
    // No key, no pianist: the process says so and the seat is not played at all rather than by the heuristics,
    // which would pass for the pianist in the ledger.
    let pianist = match pianist {
        false => None,
        true => match brain::pianist::Pianist::new(true, &log_dir(), hello.ai_id) {
            Ok(pianist) => {
                eprintln!("[ai {}] pianist: {} plays from the player's instructions", hello.ai_id, pianist.model());
                Some(pianist)
            }
            Err(e) => return Err(io::Error::other(format!("pianist mode asked for but {e}"))),
        },
    };
    let mut recorder = recorder::Recorder::from_env(&log_dir(), &hello, mode_name, strategist.is_some(), pianist.is_some());
    let setting = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let mut banner = format!("Within Reason {} | {mode_name}", env!("WITHIN_REASON_COMMIT"));
    if strategist.is_some() {
        let cli = std::process::Command::new("claude").arg("--version").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|v| !v.is_empty());
        banner += &format!(" ({}{}{})", setting("WITHIN_REASON_MODEL").unwrap_or_else(|| "claude-opus-5".into()), setting("WITHIN_REASON_EFFORT").map_or(String::new(), |e| format!(", effort {e}")), cli.map_or(String::new(), |v| format!(", {v}")));
    }
    if let Some(disabled) = setting("WITHIN_REASON_DISABLE") {
        banner += &format!(" | off: {disabled}");
    }
    if pianist.is_some() {
        banner += &format!(" | hands: {}", pianist.as_ref().map_or(String::new(), |p| p.model()));
    }
    let mut brain = Brain::new(World::new(hello), strategist.as_ref().map(|s| s.shared.clone()), board, banner, pianist);
    write_frame(&mut stream, &Commands::default())?;
    loop {
        let ToBot::Tick(tick) = next()? else {
            return Err(io::Error::other("unexpected second Hello"));
        };
        let started = std::time::Instant::now();
        let commands = brain.decide(&tick);
        let decide_ms = started.elapsed().as_secs_f32() * 1000.0;
        let journal = brain.take_journal();
        if let Some(r) = &mut recorder
            && let Err(e) = r.tick(&tick, journal, |_| 0, &commands, decide_ms)
        {
            // A full disk must not lose the match.
            eprintln!("match record abandoned: {e}");
            recorder = None;
        }
        write_frame(&mut stream, &Commands(commands))?;
    }
}
