//! The Claude Code session beside the brain: it watches the game through our MCP server and pulls the levers the
//! brain exposes. Never in the control loop; see `DESIGN.md`. Two ways to run it:
//!
//! - **Strategist**: Opus, a turn every 45 s of game time or on a trigger, standing directives only.
//! - **Commander**: Sonnet, turns back to back at game speed 1, with squads, the unit mix and turret requests.
//! - **Player**: Opus over the pianist (`docs/design/2026-09-21-pianist.md`): turns when woken, the game held
//!   still meanwhile, and one lever, the standing instructions in prose that Jev reads every second.

mod mcp;
mod report;
pub mod seats;
pub mod shared;
mod transcript;

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use mcp::McpServer;
use shared::Shared;
use transcript::Transcript;

/// `claude --effort` unless `WITHIN_REASON_EFFORT` (the arena's `--effort`) says otherwise. The game is held still during a
/// turn, so thinking costs wall time only; against people in real time it will want to be `low`.
const DEFAULT_EFFORT: &str = "high";

/// Sessions run on this subscription unless `WITHIN_REASON_CLAUDE_CONFIG_DIR` says otherwise: it has extra usage
/// (paid credits) disabled, so it can be blocked but never charged (`docs/harness/claude-p.md`).
const DEFAULT_CLAUDE_CONFIG_DIR: &str = ".claude2";
/// Game time between the strategist's routine turns when nothing triggers one sooner.
const ROUTINE_INTERVAL_FRAMES: i32 = 45 * 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Strategist,
    Commander,
    Player,
}

impl Mode {
    /// `WITHIN_REASON_MODEL` (the arena's `--commander-model`) overrides the role's usual model.
    fn model(self) -> String {
        std::env::var("WITHIN_REASON_MODEL").ok().filter(|m| !m.is_empty()).unwrap_or_else(|| {
            match self {
                Mode::Strategist | Mode::Player => "claude-opus-5",
                Mode::Commander => "claude-sonnet-5",
            }
            .into()
        })
    }

    /// Read from the checkout at every session start (`crate::texts`): an edit needs no rebuild.
    fn system_prompt(self) -> String {
        use crate::texts::{read, COMMANDER_BRIEF, COMMANDER_PROMPT, PLAYER_BRIEF, PLAYER_PROMPT, POLICY_PROMPT, STRATEGIST_PROMPT};
        match self {
            Mode::Strategist => read(&STRATEGIST_PROMPT),
            // The role, then what the project knows (`docs/README.md`: the brief is rewritten from the knowledge base).
            Mode::Commander => read(&COMMANDER_PROMPT) + &read(&COMMANDER_BRIEF),
            // The player's lever is the packet, or the Lua policy when the runtime is on (`bot --policy`).
            Mode::Player if policy_mode() => read(&POLICY_PROMPT) + &read(&PLAYER_BRIEF) + &objective(),
            Mode::Player => read(&PLAYER_PROMPT) + &read(&PLAYER_BRIEF) + &objective(),
        }
    }

    /// Turns are taken with the game held still (the brain asks for them, `brain/wake.rs`).
    fn lockstep(self) -> bool {
        if realtime() {
            return false;
        }
        matches!(self, Mode::Commander | Mode::Player)
    }

    /// Turns after which the session is replaced by a fresh one that is handed the notes, to bound its context.
    fn turns_per_session(self) -> usize {
        match self {
            Mode::Strategist => usize::MAX,
            Mode::Commander | Mode::Player => 40,
        }
    }
}

/// A requirement the user set for this game (`arena --objective`, `WITHIN_REASON_OBJECTIVE`): "kill the commander
/// with Thunder bombers", to prove a branch of the roster reachable. Appended to the player's role text.
fn objective() -> String {
    match std::env::var("WITHIN_REASON_OBJECTIVE") {
        Ok(text) if !text.trim().is_empty() => {
            eprintln!("[strategist] objective in the role text: {}", text.trim());
            format!("\n\n**This game's objective, set by the user (it overrides the brief's plan where they conflict; winning any other way does not count):** {}\n", text.trim())
        }
        _ => String::new(),
    }
}

pub struct Strategist {
    pub shared: Arc<Shared>,
    _server: McpServer,
    stop: Arc<AtomicBool>,
}

/// What it takes to start a session, kept so the driver can start the next one.
struct Launch {
    mode: Mode,
    cwd: PathBuf,
    config_dir: PathBuf,
    /// `claude --effort`: stated, never inherited, so a transcript can be compared with another.
    effort: String,
    mcp_config: String,
    mcp_url: String,
    transcript: Arc<Transcript>,
}

/// Which CLI runs the model: Claude Code for Claude models, the Codex CLI for OpenAI ones (`gpt-*`), chosen by the
/// model's name. Codex has no long-lived session over stdin: every turn is one `codex exec`, resumed on the thread the
/// first one started, with the role text as `AGENTS.md` in the working directory and the bot's MCP server attached by
/// URL.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Backend {
    Claude,
    Codex,
}

impl Backend {
    fn for_model(model: &str) -> Backend {
        if model.starts_with("gpt-") || model.starts_with("o3") || model.starts_with("o4") { Backend::Codex } else { Backend::Claude }
    }
}

enum SessionKind {
    Claude { child: Child, stdin: ChildStdin },
    Codex {
        model: String,
        effort: String,
        cwd: PathBuf,
        mcp_url: String,
        transcript: Arc<Transcript>,
        thread_id: Arc<std::sync::Mutex<Option<String>>>,
        child: Arc<std::sync::Mutex<Option<Child>>>,
        turn_done_tx: std::sync::mpsc::Sender<()>,
    },
}

struct Session {
    kind: SessionKind,
    turn_done: Receiver<()>,
    /// The system prompt this session was started with (`texts::digest`), to notice an edit on disk.
    prompt: u64,
}

impl Strategist {
    /// Starts the MCP server and the Claude Code session. `dir` receives `strategist-N.jsonl`.
    pub fn start(dir: &Path, ai_id: i32, mode: Mode) -> std::io::Result<Self> {
        let shared = Arc::new(Shared::default());
        shared.lockstep.store(mode.lockstep(), Ordering::Relaxed);
        shared.gated.store(matches!(mode, Mode::Commander | Mode::Player), Ordering::Relaxed);
        // How late the commander's orders land, in game seconds per wall second of thought (arena `--think-penalty`).
        *shared.think_penalty.lock().unwrap() = std::env::var("WITHIN_REASON_THINK_PENALTY").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let transcript = Arc::new(Transcript::create(&dir.join(format!("strategist-{ai_id}.jsonl")))?);
        let server = McpServer::start(shared.clone(), transcript.clone(), mode)?;
        // An empty working directory: nothing for the session to discover.
        let cwd = dir.join(format!("strategist-{ai_id}-cwd"));
        std::fs::create_dir_all(&cwd)?;
        let mcp_config = json!({ "mcpServers": { "wreason": { "type": "http", "url": format!("http://127.0.0.1:{}/mcp", server.port) } } });
        let config_dir = std::env::var_os("WITHIN_REASON_CLAUDE_CONFIG_DIR").map_or_else(
            || PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(DEFAULT_CLAUDE_CONFIG_DIR),
            Into::into,
        );
        let effort = std::env::var("WITHIN_REASON_EFFORT").ok().filter(|e| !e.is_empty()).unwrap_or_else(|| DEFAULT_EFFORT.into());
        let mcp_url = format!("http://127.0.0.1:{}/mcp", server.port);
        let launch = Launch { mode, cwd, config_dir, effort, mcp_config: mcp_config.to_string(), mcp_url, transcript };
        let session = launch.spawn()?;
        eprintln!(
            "[ai {ai_id}] {mode:?} started ({}, effort {}, {}, MCP on port {})",
            mode.model(),
            launch.effort,
            match Backend::for_model(&mode.model()) {
                Backend::Claude => format!("account {}", launch.config_dir.display()),
                Backend::Codex => "the Codex CLI on its own login".to_string(),
            },
            server.port
        );
        let stop = Arc::new(AtomicBool::new(false));
        let (driver_shared, driver_stop) = (shared.clone(), stop.clone());
        std::thread::spawn(move || drive(launch, session, &driver_shared, &driver_stop, ai_id));
        Ok(Strategist { shared, _server: server, stop })
    }
}

impl Drop for Strategist {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Launch {
    fn spawn(&self) -> std::io::Result<Session> {
        let prompt = self.mode.system_prompt();
        if Backend::for_model(&self.mode.model()) == Backend::Codex {
            std::fs::write(self.cwd.join("AGENTS.md"), &prompt)?;
            let (turn_done_tx, turn_done) = channel();
            let kind = SessionKind::Codex {
                model: self.mode.model(),
                effort: match self.effort.as_str() {
                    "xhigh" | "max" => "xhigh".to_string(),
                    other => other.to_string(),
                },
                cwd: self.cwd.clone(),
                mcp_url: self.mcp_url.clone(),
                transcript: self.transcript.clone(),
                thread_id: Arc::new(std::sync::Mutex::new(None)),
                child: Arc::new(std::sync::Mutex::new(None)),
                turn_done_tx,
            };
            return Ok(Session { kind, turn_done, prompt: crate::texts::digest(&prompt) });
        }
        let mut child = Command::new("claude")
            .current_dir(&self.cwd)
            .env("CLAUDE_CONFIG_DIR", &self.config_dir)
            .args(["-p", "--model", &self.mode.model(), "--tools", "", "--strict-mcp-config", "--mcp-config"])
            .arg(&self.mcp_config)
            .args(["--allowedTools", "mcp__wreason__*", "--permission-mode", "dontAsk", "--setting-sources", ""])
            .args(["--effort", &self.effort])
            .args(["--system-prompt", &prompt])
            .args(["--input-format", "stream-json", "--output-format", "stream-json", "--verbose"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Kept since realtime-1: a session that answers nothing for 120 s (games 10, 11, realtime-1) leaves no
            // trace without it.
            .stderr(std::fs::File::create(self.cwd.with_file_name(self.cwd.file_name().map(|n| n.to_string_lossy().replace("-cwd", "-stderr.log")).unwrap_or_default())).map_or(Stdio::null(), Stdio::from))
            .spawn()?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (turn_done_tx, turn_done) = channel();
        let transcript = self.transcript.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let Ok(message) = serde_json::from_str::<Value>(&line) else { continue };
                let kind = message["type"].as_str().unwrap_or_default();
                if matches!(kind, "assistant" | "result" | "rate_limit_event") {
                    transcript.record(json!({ "kind": kind, "message": message }));
                }
                // We spend weekly allotments only: the first sign of paid overage, or of any limit, ends the session.
                let limit = &message["rate_limit_info"];
                if kind == "rate_limit_event"
                    && (limit["isUsingOverage"].as_bool() == Some(true) || limit["status"].as_str().is_some_and(|s| s != "allowed"))
                {
                    eprintln!("[strategist] STOPPING: rate limit event {limit}");
                    transcript.record(json!({ "kind": "stopped", "reason": limit }));
                    break;
                }
                if kind == "result" && turn_done_tx.send(()).is_err() {
                    break;
                }
            }
        });
        Ok(Session { kind: SessionKind::Claude { child, stdin }, turn_done, prompt: crate::texts::digest(&prompt) })
    }
}

impl Session {
    /// One turn's prompt to the model: a line on the session's stdin (Claude), or one `codex exec` on the thread
    /// (Codex), whose events go to the transcript in the same shape and whose exit is the turn's result.
    fn send(&mut self, prompt: &str) -> bool {
        match &mut self.kind {
            SessionKind::Claude { stdin, .. } => {
                let line = json!({ "type": "user", "message": { "role": "user", "content": prompt } });
                writeln!(stdin, "{line}").and_then(|()| stdin.flush()).is_ok()
            }
            SessionKind::Codex { model, effort, cwd, mcp_url, transcript, thread_id, child, turn_done_tx } => {
                let mut command = Command::new("codex");
                command.arg("exec");
                if let Some(id) = thread_id.lock().unwrap().as_ref() {
                    command.args(["resume", id]);
                }
                // The bot's tools are MCP calls, which `codex exec` refuses under every approval policy but the
                // bypass; the working directory is empty and the sandbox has nothing to guard there. `resume` takes
                // neither `-s` nor `-C`, so the directory is the process's.
                command
                    .current_dir(&*cwd)
                    .args(["--json", "-m", model, "--skip-git-repo-check", "--dangerously-bypass-approvals-and-sandbox"])
                    .args(["-c", &format!("model_reasoning_effort=\"{effort}\"")])
                    .args(["-c", &format!("mcp_servers.wreason.url=\"{mcp_url}\"")])
                    .arg("-")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(std::fs::File::create(cwd.with_file_name(cwd.file_name().map(|n| n.to_string_lossy().replace("-cwd", "-stderr.log")).unwrap_or_default())).map_or(Stdio::null(), Stdio::from));
                let mut spawned = match command.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[strategist] codex exec failed to start: {e}");
                        return false;
                    }
                };
                let Some(mut stdin) = spawned.stdin.take() else { return false };
                let Some(stdout) = spawned.stdout.take() else { return false };
                let prompt = prompt.to_string();
                std::thread::spawn(move || {
                    let _ = stdin.write_all(prompt.as_bytes());
                    drop(stdin);
                });
                *child.lock().unwrap() = Some(spawned);
                let (transcript, thread_id, child, model, done) = (transcript.clone(), thread_id.clone(), child.clone(), model.clone(), turn_done_tx.clone());
                std::thread::spawn(move || {
                    let mut usage = Value::Null;
                    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                        let Ok(event) = serde_json::from_str::<Value>(&line) else { continue };
                        match event["type"].as_str().unwrap_or_default() {
                            "thread.started" => {
                                if let Some(id) = event["thread_id"].as_str() {
                                    *thread_id.lock().unwrap() = Some(id.to_string());
                                }
                            }
                            "item.completed" if event["item"]["type"] == "agent_message" => {
                                let text = event["item"]["text"].as_str().unwrap_or_default();
                                transcript.record(json!({ "kind": "assistant", "message": { "message": { "model": model, "content": [{ "type": "text", "text": text }] } } }));
                            }
                            "turn.completed" => usage = event["usage"].clone(),
                            "error" => transcript.record(json!({ "kind": "error", "message": event })),
                            _ => {}
                        }
                    }
                    let status = child.lock().unwrap().as_mut().map(|c| c.wait().ok());
                    *child.lock().unwrap() = None;
                    transcript.record(json!({ "kind": "result", "message": { "type": "result", "usage": usage, "modelUsage": { model.clone(): { "inputTokens": usage["input_tokens"], "outputTokens": usage["output_tokens"], "cacheReadInputTokens": usage["cached_input_tokens"] } }, "exit": format!("{status:?}") } }));
                    let _ = done.send(());
                });
                true
            }
        }
    }

    fn end(self) {
        match self.kind {
            SessionKind::Claude { mut child, .. } => {
                let _ = child.kill();
                let _ = child.wait();
            }
            SessionKind::Codex { child, .. } => {
                if let Some(mut c) = child.lock().unwrap().take() {
                    let _ = c.kill();
                    let _ = c.wait();
                }
            }
        }
    }
}

/// Sends one turn at a time. The strategist is called on a timer and on triggers while the game runs on; the
/// commander is called when the brain asks (its wake conditions), and the brain holds the game until the turn ends.
fn drive(launch: Launch, mut session: Session, shared: &Shared, stop: &AtomicBool, ai_id: i32) {
    let mode = launch.mode;
    let mut last_turn_frame = i32::MIN / 2;
    let mut turns_this_session = 0;
    let mut seen = report::Seen::default();
    let mut owed_result = false;
    while !stop.load(Ordering::Relaxed) {
        let headline = match mode {
            Mode::Commander | Mode::Player => match shared.next_turn_request(stop) {
                Some(reason) => reason,
                None => break,
            },
            Mode::Strategist => {
                std::thread::sleep(Duration::from_millis(200));
                let frame = shared.briefing().frame;
                let triggers = std::mem::take(&mut *shared.triggers.lock().unwrap());
                if frame == 0 || (triggers.is_empty() && frame - last_turn_frame < ROUTINE_INTERVAL_FRAMES) {
                    continue;
                }
                if triggers.is_empty() { "Routine check.".to_string() } else { triggers.join(" ") }
            }
        };
        // The last turn ended at its `wait`; the session may still be writing its closing words, and takes no new
        // prompt until it has reported that response finished. Before the restart below, not after: the new session
        // owes nothing, and waiting on it held the game until the arena gave up (commander game 10b, turn 40).
        if owed_result {
            owed_result = false;
            let arrived = loop {
                match session.turn_done.recv_timeout(Duration::from_millis(250)) {
                    Ok(()) => break true,
                    Err(RecvTimeoutError::Timeout) if !stop.load(Ordering::Relaxed) => {}
                    Err(_) => break false,
                }
            };
            if !arrived {
                shared.end_turn();
                break;
            }
        }
        // A fresh session at the cap, and as soon as the prompt on disk has been edited (the user, 2026-09-22: text
        // updates without a rebuild); either way the new session is handed the notes.
        let edited = crate::texts::digest(&mode.system_prompt()) != session.prompt;
        if edited {
            eprintln!("[ai {ai_id}] the prompt on disk has changed: a fresh session reads it");
        }
        if turns_this_session >= mode.turns_per_session() || edited {
            session.end();
            session = match launch.spawn() {
                Ok(next) => next,
                Err(e) => {
                    eprintln!("[ai {ai_id}] could not restart the session: {e}; heuristics carry on alone");
                    shared.close_gate();
                    return;
                }
            };
            turns_this_session = 0;
        }
        let (frame, game_time) = {
            let briefing = shared.briefing();
            (briefing.frame, briefing.game_time)
        };
        let prompt = match mode {
            Mode::Strategist => format!("Game time {game_time}. {headline}"),
            Mode::Commander => commander_prompt(&game_time, &headline, shared, &mut seen, turns_this_session == 0),
            Mode::Player => player_prompt(&game_time, &headline, shared, &mut seen, turns_this_session == 0),
        };
        last_turn_frame = frame;
        turns_this_session += 1;
        let started = Instant::now();
        launch.transcript.record(json!({ "kind": "turn", "frame": frame, "prompt": prompt }));
        shared.turn_over.store(false, Ordering::Relaxed);
        let sent = session.send(&prompt);
        let mut abandoned = false;
        let finished = sent
            && loop {
                match session.turn_done.recv_timeout(Duration::from_millis(50)) {
                    Ok(()) => break true,
                    // The commander called `wait`: the game is running again, the response's tail is owed.
                    Err(RecvTimeoutError::Timeout) if mode.lockstep() && !shared.turn_in_progress() => {
                        owed_result = true;
                        break true;
                    }
                    // A turn that has held the game this long is a hung session (pianist-player-10: the first turn
                    // never returned, and the engine's watchdog killed the game at three minutes): the game goes on
                    // and the session is replaced.
                    Err(RecvTimeoutError::Timeout) if matches!(mode, Mode::Commander | Mode::Player) && started.elapsed() > turn_cap(mode) => {
                        abandoned = true;
                        break true;
                    }
                    Err(RecvTimeoutError::Timeout) if !stop.load(Ordering::Relaxed) => {}
                    Err(_) => break false,
                }
            };
        if abandoned {
            eprintln!("[ai {ai_id}] {mode:?} turn abandoned after {} s with no answer: the session is replaced", started.elapsed().as_secs());
            launch.transcript.record(json!({ "kind": "turn_end", "wall_seconds": started.elapsed().as_secs_f32(), "ended_by": "abandoned" }));
            shared.end_turn();
            session.end();
            session = match launch.spawn() {
                Ok(next) => next,
                Err(e) => {
                    eprintln!("[ai {ai_id}] could not restart the session: {e}; heuristics carry on alone");
                    shared.close_gate();
                    return;
                }
            };
            turns_this_session = 0;
            owed_result = false;
            continue;
        }
        launch.transcript.record(json!({ "kind": "turn_end", "wall_seconds": started.elapsed().as_secs_f32(), "ended_by": if owed_result { "wait" } else { "response" } }));
        shared.end_turn();
        if !finished {
            if !stop.load(Ordering::Relaxed) {
                eprintln!("[ai {ai_id}] {mode:?} session ended; heuristics carry on alone");
            }
            break;
        }
    }
    shared.close_gate();
    session.end();
}

/// A turn is abandoned after this long with no answer: in lockstep 45 s (the engine's watchdog, HangTimeout, is 60 s by
/// default and the arena raises it to 600; turns run 1 to 11 s), in real time 120 s (the game runs on meanwhile).
fn turn_cap(mode: Mode) -> Duration {
    // A Codex turn is one process with a 3 s floor and 20-60 s of writing (docs/studies/policy-models.md): the
    // lockstep cap that catches a hung Claude session would discard most of them.
    Duration::from_secs(if mode.lockstep() && Backend::for_model(&mode.model()) == Backend::Claude { 45 } else { 120 })
}

/// `WITHIN_REASON_REALTIME`: the game is never held for a turn or an answer (a game against people; the arena's
/// `--realtime` rehearsal). The player is told; the pianist asks Jev on a thread and plays the answers when they come.
pub(crate) fn realtime() -> bool {
    std::env::var_os("WITHIN_REASON_REALTIME").is_some()
}

/// `bot --policy`: the player's lever is a Lua policy (its role text, its tool); set once at start.
static POLICY_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_policy_mode(on: bool) {
    POLICY_MODE.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn policy_mode() -> bool {
    POLICY_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The commander is shown the picture outright (a tool call to look would double its turn), in full at the start of
/// a session and as changes afterwards.
fn commander_prompt(game_time: &str, headline: &str, shared: &Shared, seen: &mut report::Seen, fresh_session: bool) -> String {
    let briefing = shared.briefing();
    let field = shared.field();
    let fights: Vec<String> =
        std::mem::take(&mut *shared.fights.lock().unwrap()).into_iter().map(|(what, n)| format!("{what} x{n}")).collect();
    let mut prompt = String::new();
    if fresh_session && realtime() {
        prompt.push_str("This game runs in real time: it does not pause while you take a turn, and your orders land when the turn ends, five to ten seconds later. Decide from the report, write states that hold, and keep turns short.\n\n");
    }
    if fresh_session {
        let notes = shared.notes.lock().unwrap();
        if !notes.is_empty() {
            prompt += &format!("You are taking over mid-game from an earlier session of yourself. Its notes:\n{}\n\n", notes.join("\n"));
        }
        prompt += &format!("Map: {}\n\n", shared.map.lock().unwrap());
    }
    let wake = serde_json::to_string(&*shared.wake.lock().unwrap()).unwrap_or_default();
    prompt += &format!(
        "[{game_time}] Woken because: {headline}\n{}\nwake conditions in force: {wake}",
        report::report(seen, &briefing, &field, &fights, fresh_session)
    );
    prompt
}

/// The player is shown the game as the commander is, then its hands: what they did since the last turn, the actors
/// as the picture has them. A fresh session is handed the notes and the instructions in force as well as the map.
fn player_prompt(game_time: &str, headline: &str, shared: &Shared, seen: &mut report::Seen, fresh_session: bool) -> String {
    let briefing = shared.briefing();
    let field = shared.field();
    let fights: Vec<String> =
        std::mem::take(&mut *shared.fights.lock().unwrap()).into_iter().map(|(what, n)| format!("{what} x{n}")).collect();
    let hands = {
        let mut hands = shared.hands.lock().unwrap();
        let snapshot = hands.clone();
        hands.done.clear();
        hands.policy_stats = Default::default();
        snapshot
    };
    let mut prompt = String::new();
    if fresh_session && realtime() {
        prompt.push_str("This game runs in real time: it does not pause while you take a turn, and your orders land when the turn ends, five to ten seconds later. Decide from the report, write states that hold, and keep turns short.\n\n");
    }
    if fresh_session {
        let notes = shared.notes.lock().unwrap();
        if !notes.is_empty() {
            prompt += &format!("You are taking over mid-game from an earlier session of yourself. Its notes:\n{}\n\n", notes.join("\n"));
        }
        let instructions = shared.instructions.lock().unwrap();
        if !instructions.trim().is_empty() {
            prompt += &format!("The instructions in force, as your earlier session last wrote them:\n{instructions}\n\n");
        }
        prompt += &format!("Map: {}\n\n", shared.map.lock().unwrap());
    }
    let wake = serde_json::to_string(&*shared.wake.lock().unwrap()).unwrap_or_default();
    let chat: Vec<String> = std::mem::take(&mut *shared.chat_in.lock().unwrap()).into_iter().map(|(frame, player, text)| format!("{} player {player}: {text}", crate::brain::pianist::clock(frame))).collect();
    let searches = std::mem::take(&mut *shared.search_results.lock().unwrap());
    if !searches.is_empty() {
        prompt += &format!("Your search finished (the simulator's answer, from the game as it stood when you asked):\n{}\n\n", searches.join("\n"));
    }
    prompt += &format!(
        "[{game_time}] Woken because: {headline}\n{}\nwake conditions in force: {wake}",
        report::player_report(seen, &briefing, &field, &fights, &hands, &chat, fresh_session)
    );
    prompt
}
