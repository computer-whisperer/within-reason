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

/// usage: bot [--player] [--pianist]
/// `--pianist`: Jev plays every unit from the player's instructions (`docs/design/2026-09-21-pianist.md`).
/// `--player`: a Claude Code session beside the brains (see `DESIGN.md`), the Opus player whose lever is the
/// pianist's instructions; one session serves every seat we play on a team (`strategist/seats.rs`), or each seat
/// has its own (`strategist::player_per_seat`).
/// Transcripts go to `$WITHIN_REASON_LOG_DIR`, else the current directory.
fn main() -> io::Result<()> {
    let mut player = false;
    let mut pianist = false;
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
    // Sessions running now. With `WITHIN_REASON_EXIT_WHEN_OVER` (`run/human_game.sh`: one game a process) the bot
    // leaves when its last session ends; otherwise it listens on for the next game.
    static LIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let leave_when_over = std::env::var_os("WITHIN_REASON_EXIT_WHEN_OVER").is_some();
    for stream in listener.incoming() {
        let stream = stream?;
        LIVE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::thread::spawn(move || {
            if let Err(e) = session(stream, player, pianist) {
                eprintln!("session ended: {e}");
            }
            if LIVE.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 && leave_when_over {
                eprintln!("every session has ended: the game is over; leaving");
                let _ = std::fs::remove_file(socket_path());
                std::process::exit(0);
            }
        });
    }
    Ok(())
}

pub(crate) fn log_dir() -> std::path::PathBuf {
    std::env::var_os("WITHIN_REASON_LOG_DIR").map_or_else(|| ".".into(), Into::into)
}

/// `player`: whether the Opus player's session is started (one per team, `strategist/seats.rs`, or one a seat).
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
        .then(|| {
            let start = || Strategist::start(&log_dir(), hello.ai_id).map(std::sync::Arc::new);
            if strategist::player_per_seat() { start() } else { board.strategist(start) }
        })
        .and_then(|started| started.inspect_err(|e| eprintln!("the player's session failed to start: {e}")).ok());
    // The side's commander, when the game is to have one (`WITHIN_REASON_COMMANDER`): held by every seat's session,
    // and each seat's player joins it. One that fails to start is not fatal: the players play by their own judgement.
    let commander = strategist.as_ref().filter(|_| strategist::command::model().is_some()).and_then(|player| {
        let commander = board.commander(|| strategist::command::Commander::start(&log_dir())).inspect_err(|e| eprintln!("the commander's session failed to start: {e}")).ok()?;
        commander.attach(hello.team, &player.shared);
        Some(commander)
    });
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
        banner += &format!(" ({}{}{})", strategist::model(), setting("WITHIN_REASON_EFFORT").map_or(String::new(), |e| format!(", effort {e}")), cli.map_or(String::new(), |v| format!(", {v}")));
    }
    if let Some(disabled) = setting("WITHIN_REASON_DISABLE") {
        banner += &format!(" | off: {disabled}");
    }
    if pianist.is_some() {
        banner += &format!(" | hands: {}", pianist.as_ref().map_or(String::new(), |p| p.model()));
    }
    let mut brain = Brain::new(World::new(hello), strategist.as_ref().map(|s| s.shared.clone()), board, banner, pianist);
    // The opening turn is asked for now, before the game's first tick (`docs/design/2026-09-30-opening-turn.md`).
    brain.before_the_game();
    write_frame(&mut stream, &Commands::default())?;
    let (mut last_frame, mut last_own) = (0, 0);
    let _commander = commander;
    loop {
        let tick = match next() {
            Ok(ToBot::Tick(tick)) => tick,
            Ok(_) => return Err(io::Error::other("unexpected second Hello")),
            Err(e) => {
                // The engine closed the connection: the game is over as far as this seat can know.
                if let Some(r) = &mut recorder {
                    r.ended(last_frame, last_own, &e.to_string());
                }
                return Err(e);
            }
        };
        (last_frame, last_own) = (tick.frame, tick.snapshot.own_units.len());
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
