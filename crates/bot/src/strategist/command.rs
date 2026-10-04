//! The side's commander (`docs/design/2026-10-03-commander-seat.md`): a session above the players that orders no
//! unit. It reads a report of the whole side and writes a direction in prose, a plan every player is shown and a
//! paragraph for each seat; the players carry it out with their own judgement.
//!
//! Its state is a `Shared` of its own (the side's), into which every seat's brain publishes what it publishes for
//! its player; the players' own `Shared`s point at it (`Shared::side`) for the direction, and it points back at
//! them for their notes and packets.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::shared::Shared;
use super::{Launch, Session};

const FRAMES_PER_SECOND: i32 = 30;
/// Game seconds between the commander's turns unless its `wait` says otherwise, and the bounds of what it may ask.
const DEFAULT_WAIT: u32 = 90;
const WAIT_BOUNDS: (u64, u64) = (30, 300);
/// The first turn comes when the game is this old: the players' opening lists are running and their first reports
/// are in.
const FIRST_TURN_FRAME: i32 = 20 * FRAMES_PER_SECOND;
/// A turn with no answer for this long is given up and the session replaced.
const TURN_CAP: Duration = Duration::from_secs(240);
/// The longest a part of a direction may be: a plan is a few lines, a seat's paragraph a short one.
const PART_LIMIT: usize = 1500;

/// One part of a direction as written: the words, the frame of the report they answered, and the frame they come
/// into force (the report's frame plus the game time the turn's wall seconds are worth: the commander never holds
/// the game, so in lockstep its thinking is charged as if the game had run on).
#[derive(Clone, Debug, Default)]
pub struct Written {
    pub text: String,
    pub frame: i32,
    pub due: i32,
}

/// The direction in force and what is on its way: the plan for every seat, and each seat's paragraph by team.
#[derive(Debug, Default)]
pub struct Direction {
    plan: Vec<Written>,
    seats: BTreeMap<i32, Vec<Written>>,
    /// The turn in hand: its report's frame and when it began, for `due`.
    turn: Option<(i32, Instant)>,
    /// How many of each player's notes the commander has been shown, by the player's lead team.
    notes_seen: BTreeMap<i32, usize>,
    /// Each seat every `SAMPLE_FRAMES`, oldest first: the commander is shown the last `HISTORY_ROWS` of them, so a
    /// figure of this second is read against where it has been (duel-split-1: nine directions in ten turns, each
    /// answering the report's momentary energy or bank, and the economy swung between them).
    history: BTreeMap<i32, Vec<Sample>>,
}

/// A seat's economy at a frame.
#[derive(Clone, Debug)]
struct Sample {
    frame: i32,
    metal: (f32, f32, f32),
    energy: (f32, f32, f32),
    counts: super::shared::Counts,
    soldiers: usize,
    army_metal: u32,
}

const SAMPLE_FRAMES: i32 = 30 * FRAMES_PER_SECOND;
const HISTORY_ROWS: usize = 14;
/// How many of its earlier directions to a part the commander is shown beside the one in force.
const DIRECTIONS_SHOWN: usize = 4;

impl Direction {
    /// Seats side: a seat has published; its history gains a row when the last is `SAMPLE_FRAMES` old.
    pub fn sample(&mut self, team: i32, seat: &super::seats::SeatView) {
        let rows = self.history.entry(team).or_default();
        if rows.last().is_some_and(|last| seat.frame - last.frame < SAMPLE_FRAMES) {
            return;
        }
        let b = &seat.briefing;
        rows.push(Sample {
            frame: seat.frame,
            metal: (b.metal.current, b.metal.income, b.metal.usage),
            energy: (b.energy.current, b.energy.income, b.energy.usage),
            counts: b.counts.clone(),
            soldiers: seat.field.score.soldiers,
            army_metal: seat.field.score.army_metal,
        });
    }

    /// The seat's last rows as a table, with the commander's own directions marked where they came into force.
    fn history_lines(&self, team: i32) -> Vec<String> {
        let Some(rows) = self.history.get(&team).filter(|rows| rows.len() > 1) else { return Vec::new() };
        let rows = &rows[rows.len().saturating_sub(HISTORY_ROWS)..];
        let landed: Vec<i32> = self.plan.iter().chain(self.seats.get(&team).into_iter().flatten()).map(|w| w.due).collect();
        let mut lines = vec![format!("  t{team} every 30 s (clock | metal banked, income, spending | energy stored, income, spending | extractors, generators, converters, constructors, factories, turrets | soldiers, army metal); `<` marks where a direction of yours came into force:")];
        let mut previous = rows.first().map_or(0, |r| r.frame) - SAMPLE_FRAMES;
        for r in rows {
            let mark = if landed.iter().any(|due| *due > previous && *due <= r.frame) { " <" } else { "" };
            previous = r.frame;
            lines.push(format!(
                "    {} | {:.0}, +{:.0}, -{:.0} | {:.0}, +{:.0}, -{:.0} | {}, {}, {}, {}, {}, {} | {}, {}{mark}",
                clock(r.frame), r.metal.0, r.metal.1, r.metal.2, r.energy.0, r.energy.1, r.energy.2,
                r.counts.extractors, r.counts.generators, r.counts.converters, r.counts.constructors, r.counts.labs, r.counts.turrets, r.soldiers, r.army_metal
            ));
        }
        lines
    }
}

/// The newest of `parts` in force at `frame`.
fn in_force(parts: &[Written], frame: i32) -> Option<&Written> {
    parts.iter().rev().find(|w| w.due <= frame)
}

fn clock(frame: i32) -> String {
    format!("{}:{:02}", frame / FRAMES_PER_SECOND / 60, frame / FRAMES_PER_SECOND % 60)
}

impl Shared {
    /// Player side: the direction as this player's report opens with it, for the seats it plays. `shown` is the
    /// frame of the newest part it was last shown in full; a direction that has not changed since is one line.
    pub fn direction_for(&self, teams: &[i32], frame: i32, shown: &mut i32, full: bool) -> String {
        let direction = self.direction.lock().unwrap();
        let plan = in_force(&direction.plan, frame);
        let seats: Vec<(i32, &Written)> = teams.iter().filter_map(|t| direction.seats.get(t).and_then(|p| in_force(p, frame)).map(|w| (*t, w))).collect();
        let newest = plan.iter().copied().chain(seats.iter().map(|(_, w)| *w)).map(|w| w.due).max();
        let Some(newest) = newest else {
            return "the side's commander has given no direction yet: play your seat by your own judgement.\n".to_string();
        };
        if !full && newest <= *shown {
            return format!("the commander's direction of {} stands (above, in an earlier report).\n", clock(newest));
        }
        *shown = newest;
        let age = |w: &Written| format!("written from the report of {}, {} s ago", clock(w.frame), (frame - w.frame).max(0) / FRAMES_PER_SECOND);
        let mut lines = vec!["the commander's direction (the side's plan from the seat that sees every seat; carry it out with your own judgement of how):".to_string()];
        if let Some(plan) = plan {
            lines.push(format!("  plan, the same words to every seat ({}): {}", age(plan), plan.text));
        }
        for (team, w) in seats {
            lines.push(format!("  to your seat{} ({}): {}", if teams.len() > 1 { format!(" team {team}") } else { String::new() }, age(w), w.text));
        }
        lines.join("\n") + "\n"
    }

    /// Brain side, lockstep, every tick: while the commander's turn is in hand the game runs no faster than the
    /// wall, counted from the turn's report, so its direction lands as late as it would in a game with people.
    pub fn pace_with_the_commander(&self, frame: i32) {
        loop {
            let Some((began_frame, began)) = self.direction.lock().unwrap().turn else { return };
            if frame as f32 <= began_frame as f32 + began.elapsed().as_secs_f32() * FRAMES_PER_SECOND as f32 {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// The `direct` tool: a plan for every seat and paragraphs by seat (`t2` or `2`). What is not named stands.
    fn direct(&self, arguments: &Value) -> Result<String, String> {
        let teams: Vec<i32> = self.seats.lock().unwrap().keys().copied().collect();
        let plan = arguments["plan"].as_str().map(str::trim).filter(|t| !t.is_empty());
        let mut paragraphs: Vec<(i32, String)> = Vec::new();
        if let Some(seats) = arguments["seats"].as_object() {
            for (name, text) in seats {
                let team: i32 = name.trim_start_matches(['t', 'T']).parse().map_err(|_| format!("{name} is not a seat: name seats as the report does, {}", teams.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(", ")))?;
                if !teams.contains(&team) {
                    return Err(format!("t{team} is not a seat of ours: the seats are {}", teams.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(", ")));
                }
                let text = text.as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or(format!("t{team}'s paragraph is empty"))?;
                paragraphs.push((team, text.to_string()));
            }
        }
        if plan.is_none() && paragraphs.is_empty() {
            return Err("direct needs a \"plan\", or \"seats\" with a paragraph for a seat, or both".into());
        }
        if let Some(long) = plan.iter().map(|p| p.to_string()).chain(paragraphs.iter().map(|(_, p)| p.clone())).find(|p| p.chars().count() > PART_LIMIT) {
            return Err(format!("a part of {} characters; at most {PART_LIMIT} each. A direction is a few lines: the players read it every turn beside their own report", long.chars().count()));
        }
        let now = self.briefing().frame;
        let mut direction = self.direction.lock().unwrap();
        let (frame, due) = match direction.turn {
            Some((frame, began)) => (frame, frame + (began.elapsed().as_secs_f32() * FRAMES_PER_SECOND as f32) as i32),
            None => (now, now),
        };
        let written = |text: &str| Written { text: text.to_string(), frame, due };
        if let Some(plan) = plan {
            direction.plan.push(written(plan));
        }
        for (team, text) in &paragraphs {
            direction.seats.entry(*team).or_default().push(written(text));
        }
        Ok(format!(
            "direction written{}{}: the players are shown it from their next report{}",
            if plan.is_some() { " (plan)" } else { "" },
            if paragraphs.is_empty() { String::new() } else { format!(" ({})", paragraphs.iter().map(|(t, _)| format!("t{t}")).collect::<Vec<_>>().join(", ")) },
            if due > now { format!(" after {}, {} s of game from now (the time this turn has taken)", clock(due), (due - now) / FRAMES_PER_SECOND) } else { String::new() }
        ))
    }
}

/// The commander's tools: it looks as the players look and writes only the direction.
pub(super) fn tool_list() -> Value {
    let empty = json!({ "type": "object", "properties": {}, "additionalProperties": false });
    json!([
        { "name": "direct",
          "description": "Your direction to the players, in prose. `plan`: a few lines every seat's player is shown, the same words to all: what the side is doing in the next minutes, where and on what condition bodies of several seats meet and go together. `seats`: an object of seat (t0, t1, ... as the report names them) to a paragraph for that seat's player: what it makes, where its army goes, what it should stop or start. What you do not name stands: a seat keeps its last paragraph, the plan stays until you write a new one. Name places as the report does (spots, grid cells, the names in the picture). You order no unit and write no build list: each player turns your words into its own orders, and reads them every turn beside its own report, so keep each part to a few lines.",
          "inputSchema": { "type": "object", "additionalProperties": false, "properties": {
              "plan": { "type": "string" },
              "seats": { "type": "object", "additionalProperties": { "type": "string" }, "description": "Seat (t0, t1, ...) to its paragraph." } } } },
        { "name": "mark",
          "description": "Name a place for the whole side: an object of name to [x, z] map coordinates or a grid cell (\"E7\": its centre), or null to forget it. Every seat's player and hands see it under the same name from their next look, so a meeting place is one place to all of them (\"gather at meet_east\"). Names are lower-case words with underscores; home, spot_N, passage_N and group_N are taken.",
          "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, { "type": "string" }, { "type": "null" } ] } } },
        { "name": "say",
          "description": "Say something in the game's chat, to everyone playing. Chat is yours alone on our side: the players cannot speak or hear. Short lines: the game shows 127 characters a line, the bot prefixes `[WReason] ` (never write it yourself) and splits a longer text. Your lines go out under the name of the person hosting the bot (the map's `people`). When an experienced player offers advice or asks what we are doing, answer, ask what they would do, and put what you take from it into your direction.",
          "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string", "maxLength": 240 } } } },
        super::mcp::surrender_tool(),
        { "name": "situation",
          "description": "The picture the players' hands read this second, every seat's together: economy, ours, enemy, places by name, every actor with what it is doing. Your report carries a summary; call this to read the whole of it. It is large.",
          "inputSchema": empty },
        { "name": "overview",
          "description": "The side's state as numbers: time, economy, unit counts, enemies in sight, remembered enemy buildings, recent events.",
          "inputSchema": empty },
        { "name": "map",
          "description": "Static map facts: size, grid naming, the starts, the lobby's start boxes, every metal spot with its grid cell, the map sheet, who is in the game.",
          "inputSchema": empty },
        { "name": "units",
          "description": "The glossary entry for units by internal name, ours or the opponent's: what it is for, what it beats and loses to, its numbers. Several names at once.",
          "inputSchema": { "type": "object", "additionalProperties": false, "required": ["names"], "properties": { "names": { "type": "array", "items": { "type": "string" }, "minItems": 1 } } } },
        { "name": "note",
          "description": "Record your reasoning in a sentence or two. Kept with the game time for post-game analysis, and handed to you again if your session is replaced; it changes nothing in the game.",
          "inputSchema": { "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"], "additionalProperties": false } },
        { "name": "wait",
          "description": format!("Sets how many game seconds pass before your next report (default {DEFAULT_WAIT}, {} to {}); it holds until you change it. The game does not stop for you: your turn ends when you stop writing.", WAIT_BOUNDS.0, WAIT_BOUNDS.1),
          "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "max_seconds": { "type": "integer", "minimum": WAIT_BOUNDS.0, "maximum": WAIT_BOUNDS.1 } } } },
    ])
}

/// One of the commander's tool calls; `None` for a tool it shares with the player unchanged (`mcp::call_tool`).
pub(super) fn call_tool(name: &str, arguments: &Value, shared: &Arc<Shared>) -> Option<Result<String, String>> {
    match name {
        "direct" => Some(shared.direct(arguments)),
        "wait" => Some({
            let mut wake = shared.wake.lock().unwrap();
            if let Some(seconds) = arguments["max_seconds"].as_u64() {
                wake.max_seconds = seconds.clamp(WAIT_BOUNDS.0, WAIT_BOUNDS.1) as u32;
            }
            Ok(format!("your next report comes {} s of game after this one began", wake.max_seconds))
        }),
        "surrender" => Some(super::mcp::surrender(arguments, shared)),
        "situation" | "overview" | "map" | "units" | "note" | "say" | "mark" => None,
        other => Some(Err(format!("{other} is not a tool of the commander's: you direct the players in words (`direct`); the units are theirs"))),
    }
}

/// A seat's build power on a line: what it has, and how much of it stands in reach of each factory now.
fn build_power_line(team: i32, power: &super::shared::BuildPower) -> String {
    let own: f32 = power.factories.iter().map(|f| f.own).fold(0.0, |a, b| a + b);
    let mut line = format!(
        "  build power of t{team} (what turns metal into things; a factory builds at its own power plus what helps it): {:.0} in builders that walk ({} of them), {:.0} in {} construction turrets, {own:.0} in the factories themselves",
        power.mobile.1, power.mobile.0, power.turrets.1, power.turrets.0
    );
    for f in &power.factories {
        line += &format!(
            "; in reach of {}: {:.0} (itself {:.0}, {} construction turrets {:.0}, {} walking builders {:.0})",
            f.name, f.own + f.turrets.1 + f.mobile.1, f.own, f.turrets.0, f.turrets.1, f.mobile.0, f.mobile.1
        );
    }
    if !power.factories.is_empty() {
        line += &format!("; {:.0} of the walking builders' power stands in reach of no factory", power.mobile_away);
    }
    line
}

/// The commander's report: the side as one game, a line a seat, a line a group, what the players wrote, and its own
/// direction. `first`: with the map.
fn report(side: &Shared, first: bool) -> String {
    let briefing = side.briefing();
    let field = side.field();
    let frame = briefing.frame;
    let fights: Vec<String> = std::mem::take(&mut *side.fights.lock().unwrap()).into_iter().map(|(what, n)| format!("{what} x{n}")).collect();
    let mut lines = Vec::new();
    if first {
        lines.push(format!("Map: {}\n", side.map.lock().unwrap()));
    }
    lines.push(format!("[{}] the side's report", briefing.game_time));
    lines.extend(super::report::front(&briefing, &field, &fights, false));
    lines.extend(super::report::contact(&briefing, &field));
    lines.extend(super::fights::lines(&side.deaths.lock().unwrap(), frame));
    let chat: Vec<String> = std::mem::take(&mut *side.chat_in.lock().unwrap()).into_iter().map(|(at, _, text)| format!("  {} {text}", clock(at))).collect();
    if !chat.is_empty() {
        lines.push("chat since your last report (people in the game; `say` answers them, and only you hear or speak for our side):".to_string());
        lines.extend(chat);
    }
    let marks = side.marks.lock().unwrap();
    if !marks.is_empty() {
        lines.push(format!("your marks (places every seat knows by these names): {}", marks.iter().map(|(name, (x, z))| format!("{name} ({x:.0}, {z:.0})")).collect::<Vec<_>>().join(", ")));
    }
    drop(marks);
    // A line a seat, then its groups and what its builders and factories are at, from the picture its hands read.
    let seats = side.seats.lock().unwrap().clone();
    let hands = side.hands.lock().unwrap().clone();
    for (team, view) in &seats {
        let b = &view.briefing;
        let faction = view.field.factions.iter().find(|(t, _)| t == team).map(|(_, f)| f.as_str()).unwrap_or("faction not yet known");
        let stalling = b.energy.storage > 0.0 && b.energy.current / b.energy.storage < 0.05 && b.energy.usage >= b.energy.income * 0.9;
        lines.push(format!(
            "seat t{team} ({faction}{}, home {}): metal {:.0} banked ({:+.1}/-{:.1}), energy {:.0}/{:.0} ({:+.0}){} | extractors {}, free spots in its reach {} | constructors {}, factories {}, turrets {} | soldiers {} worth {} metal",
            if b.colour.is_empty() { String::new() } else { format!(", {} to the people in the game", b.colour) },
            b.home.grid, b.metal.current, b.metal.income, b.metal.usage, b.energy.current, b.energy.storage, b.energy.income - b.energy.usage,
            if stalling { " STALLING" } else { "" },
            b.counts.extractors, view.field.score.free_spots, b.counts.constructors, b.counts.labs, b.counts.turrets, view.field.score.soldiers, view.field.score.army_metal
        ));
        let Some(actors) = hands.get(team).and_then(|h| h.picture["actors"].as_object()) else { continue };
        let said = |entry: &Value, key: &str| entry[key].as_str().map(str::to_string);
        let (mut builders, mut idle, mut helping) = (0, 0, 0);
        for (name, entry) in actors {
            let doing = said(entry, "doing").unwrap_or_default();
            if name.starts_with("group_") {
                let mut parts: Vec<String> = [said(entry, "units"), said(entry, "at").map(|at| format!("at {at}")), Some(doing)].into_iter().flatten().filter(|p| !p.is_empty()).collect();
                parts.extend(["enemies_near", "under_fire"].iter().filter_map(|key| said(entry, key).map(|text| format!("{key}: {text}"))));
                lines.push(format!("  {name}: {}", parts.join("; ")));
            } else if name.starts_with("constructor_") || name.starts_with("commander") {
                builders += 1;
                idle += usize::from(doing.starts_with("idle"));
                helping += usize::from(doing.starts_with("helping"));
            } else {
                // A factory: what it is making and what it may make.
                let parts: Vec<String> = [Some(doing), said(entry, "allowed").map(|a| format!("allowed: {a}"))].into_iter().flatten().filter(|p| !p.is_empty()).collect();
                lines.push(format!("  {name}: {}", parts.join("; ")));
            }
        }
        lines.push(format!("  builders of t{team} (its commander unit and constructors): {builders}, of them {idle} idle and {helping} helping a factory or another builder"));
        lines.push(build_power_line(*team, &b.build_power));
        lines.extend(side.direction.lock().unwrap().history_lines(*team));
    }
    // What the players wrote since the last report: their notes, and the opening paragraph of each standing packet.
    let players: Vec<(i32, Arc<Shared>)> = side.players.lock().unwrap().iter().filter_map(|(team, p)| p.upgrade().map(|p| (*team, p))).collect();
    let mut direction = side.direction.lock().unwrap();
    for (team, player) in &players {
        let who = match player.live_seats().as_slice() {
            [] | [_] => format!("t{team}'s player"),
            many => format!("the player of {}", many.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(", ")),
        };
        let packet = player.instructions.lock().unwrap().clone();
        if let Some(opening) = packet.split("\n\n").next().map(str::trim).filter(|p| !p.is_empty()) {
            lines.push(format!("{who}, the head of its standing packet to its hands: {opening}"));
        }
        let notes = player.notes.lock().unwrap();
        let seen = direction.notes_seen.entry(*team).or_default();
        if notes.len() > *seen {
            lines.push(format!("{who}, its notes since your last report:"));
            lines.extend(notes[*seen..].iter().map(|n| format!("  {n}")));
        }
        *seen = notes.len();
    }
    // Its own direction, as it stands and as it is on its way.
    // Every part with the ones before it, newest last: what it has said over the game is beside what the game did.
    let part = |label: String, parts: &[Written]| -> Vec<String> {
        let shown = &parts[parts.len().saturating_sub(DIRECTIONS_SHOWN)..];
        shown
            .iter()
            .enumerate()
            .map(|(n, w)| {
                let standing = n + 1 == shown.len();
                format!("  {label}, from your report of {}{}{}: {}", clock(w.frame), if w.due > frame { format!(", in force from {}", clock(w.due)) } else { String::new() }, if standing { " (the one that stands)" } else { " (replaced)" }, w.text)
            })
            .collect()
    };
    let mut own: Vec<String> = part("plan".to_string(), &direction.plan);
    own.extend(direction.seats.iter().flat_map(|(team, parts)| part(format!("t{team}"), parts)));
    if own.is_empty() {
        lines.push("your direction: none yet. The players are playing their seats by their own judgement.".to_string());
    } else {
        lines.push(format!("your direction, with up to {} earlier ones of each part, oldest first:", DIRECTIONS_SHOWN - 1));
        lines.extend(own);
    }
    lines.push(format!("your next report comes {} s of game after this one (`wait` changes it)", side.wake.lock().unwrap().max_seconds));
    lines.join("\n")
}

/// The commander's session and its side's state, alive while a seat's session holds it.
pub struct Commander {
    pub side: Arc<Shared>,
    _server: super::mcp::McpServer,
    stop: Arc<std::sync::atomic::AtomicBool>,
}

/// `WITHIN_REASON_COMMANDER` (the arena's `--commander MODEL`): the model of the side's commander; unset or `none`,
/// no commander.
pub fn model() -> Option<String> {
    std::env::var("WITHIN_REASON_COMMANDER").ok().filter(|m| !m.is_empty() && m != "none")
}

impl Commander {
    /// Starts the commander's tool server and session. `dir` receives `commander.jsonl`.
    pub fn start(dir: &std::path::Path) -> std::io::Result<Self> {
        let model = model().ok_or_else(|| std::io::Error::other("no commander model"))?;
        let side = Arc::new(Shared::default());
        side.commander.store(true, Ordering::Relaxed);
        side.wake.lock().unwrap().max_seconds = DEFAULT_WAIT;
        let effort = std::env::var("WITHIN_REASON_COMMANDER_EFFORT").ok().filter(|e| !e.is_empty()).unwrap_or_else(|| "high".into());
        let (launch, server) = Launch::prepare(dir, "commander", side.clone(), model.clone(), effort, super::commander_prompt)?;
        let session = launch.spawn()?;
        eprintln!("[commander] the side's commander started ({model}, effort {}, MCP on port {})", launch.effort, server.port);
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (driver_side, driver_stop) = (side.clone(), stop.clone());
        std::thread::spawn(move || drive(launch, session, &driver_side, &driver_stop));
        Ok(Commander { side, _server: server, stop })
    }

    /// A seat's player joins the side: it is shown the direction, and the commander its notes and packet.
    pub fn attach(&self, team: i32, player: &Arc<Shared>) {
        let _ = player.side.set(self.side.clone());
        let mut players = self.side.players.lock().unwrap();
        if !players.iter().any(|(_, p)| p.upgrade().is_some_and(|p| Arc::ptr_eq(&p, player))) {
            players.push((team, Arc::downgrade(player)));
        }
    }
}

impl Drop for Commander {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// The commander's turns: by the game's clock, never holding the game. A turn is one prompt and the session's
/// whole response; what it directs comes into force as `Shared::direct` dates it.
fn drive(launch: Launch, mut session: Session, side: &Shared, stop: &std::sync::atomic::AtomicBool) {
    let mut last_turn: Option<i32> = None;
    let mut turns_this_session = 0;
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(250));
        let frame = side.briefing().frame;
        let due = match last_turn {
            None => FIRST_TURN_FRAME,
            Some(last) => last + side.wake.lock().unwrap().max_seconds as i32 * FRAMES_PER_SECOND,
        };
        // A person's chat wakes it at once: an answer a minute late is no answer.
        // Before its first turn too, once the game runs (human-20: "gl hf" at 0:01 answered at 0:20).
        let spoken_to = frame > 0 && !side.chat_in.lock().unwrap().is_empty();
        if frame < due && !spoken_to {
            continue;
        }
        if turns_this_session >= super::TURNS_PER_SESSION {
            session.end();
            session = match launch.spawn() {
                Ok(next) => next,
                Err(e) => {
                    eprintln!("[commander] could not restart the session: {e}; the players carry on by their own judgement");
                    return;
                }
            };
            turns_this_session = 0;
        }
        let mut prompt = String::new();
        if turns_this_session == 0 && last_turn.is_some() {
            let notes = side.notes.lock().unwrap();
            if !notes.is_empty() {
                prompt += &format!("You are taking over mid-game from an earlier session of yourself. Its notes:\n{}\n\n", notes.join("\n"));
            }
        }
        prompt += &report(side, turns_this_session == 0);
        last_turn = Some(frame);
        turns_this_session += 1;
        let started = Instant::now();
        side.direction.lock().unwrap().turn = Some((frame, started));
        launch.transcript.record(json!({ "kind": "turn", "frame": frame, "prompt": prompt }));
        let ended_by = if session.send(&prompt) {
            loop {
                match session.turn_done.recv_timeout(Duration::from_millis(250)) {
                    Ok(()) => break Some("response"),
                    Err(RecvTimeoutError::Timeout) if started.elapsed() > TURN_CAP => break Some("abandoned"),
                    Err(RecvTimeoutError::Timeout) if !stop.load(Ordering::Relaxed) => {}
                    Err(_) => break None,
                }
            }
        } else {
            None
        };
        if let Some(ended_by) = ended_by {
            launch.transcript.record(json!({ "kind": "turn_end", "wall_seconds": started.elapsed().as_secs_f32(), "ended_by": ended_by }));
        }
        if ended_by == Some("abandoned") {
            eprintln!("[commander] turn abandoned after {} s with no answer: the session is replaced", started.elapsed().as_secs());
            turns_this_session = super::TURNS_PER_SESSION;
        }
        let finished = ended_by.is_some();
        side.direction.lock().unwrap().turn = None;
        if !finished {
            if !stop.load(Ordering::Relaxed) {
                eprintln!("[commander] the session ended; the players carry on by their own judgement");
            }
            break;
        }
    }
    session.end();
}

/// The players a commander's side points back at: by the team of the seat that attached each.
pub type Players = Vec<(i32, Weak<Shared>)>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategist::shared::Briefing;

    fn side_at(frame: i32) -> Arc<Shared> {
        let side = Arc::new(Shared::default());
        for team in [0, 1] {
            side.publish_briefing(team, Default::default(), Briefing { frame, ..Briefing::default() });
        }
        side
    }

    /// A direction is charged the game time its turn's wall seconds are worth, is shown to a player only from
    /// then, in full once and as one line after, and a seat not named keeps its paragraph.
    #[test]
    fn a_direction_comes_into_force_after_its_turns_time_and_is_shown_once_in_full() {
        let side = side_at(3000);
        let mut shown = 0;
        assert!(side.direction_for(&[1], 3000, &mut shown, false).contains("no direction yet"));
        side.direction.lock().unwrap().turn = Some((3000, Instant::now() - Duration::from_secs(10)));
        let said = side.direct(&json!({ "plan": "gather at spot_9, go at 5:00", "seats": { "t1": "Stouts from every plant" } })).unwrap();
        assert!(said.contains("(plan)") && said.contains("(t1)") && said.contains("after 1:50"), "{said}");
        assert!(side.direction_for(&[1], 3100, &mut shown, false).contains("no direction yet"), "not yet in force");
        let first = side.direction_for(&[1], 3400, &mut shown, false);
        assert!(first.contains("plan, the same words to every seat (written from the report of 1:40, 13 s ago): gather at spot_9") && first.contains("to your seat (written from the report of 1:40, 13 s ago): Stouts"), "{first}");
        assert!(!side.direction_for(&[0], 3400, &mut 0, false).contains("Stouts"), "another seat's paragraph is not this seat's");
        assert_eq!(side.direction_for(&[1], 3500, &mut shown, false), "the commander's direction of 1:50 stands (above, in an earlier report).\n");
        assert!(side.direction_for(&[1], 3500, &mut shown, true).contains("Stouts"), "a fresh session is shown it in full");
        // A new plan alone: the seat's paragraph stands beside it.
        side.direction.lock().unwrap().turn = None;
        side.publish_briefing(0, Default::default(), Briefing { frame: 4000, ..Briefing::default() });
        side.direct(&json!({ "plan": "hold at spot_9" })).unwrap();
        let second = side.direction_for(&[1], 4000, &mut shown, false);
        assert!(second.contains("hold at spot_9") && second.contains("Stouts"), "{second}");
        assert!(side.direct(&json!({ "seats": { "t7": "x" } })).unwrap_err().contains("not a seat of ours"));
        assert!(side.direct(&json!({})).is_err());
    }

    #[test]
    fn the_commander_has_no_tool_that_orders_a_unit() {
        let side = side_at(0);
        // People's chat reaches the commander once however many seats heard it, and its own line is not chat.
        assert!(side.hear(900, 0, "push now", "thebluegecko: push now".into()));
        assert!(!side.hear(905, 0, "push now", "thebluegecko: push now".into()), "the second seat's copy");
        side.said.lock().unwrap().push("on our way".into());
        assert!(!side.hear(950, 0, "on our way", "computer_whisperer: on our way".into()), "its own line coming back");
        assert_eq!(side.chat_in.lock().unwrap().len(), 1);
        for tool in ["instruct", "queue", "produce", "remove", "lane", "orders"] {
            assert!(call_tool(tool, &json!({}), &side).is_some_and(|r| r.is_err()), "{tool}");
            assert!(!tool_list().as_array().unwrap().iter().any(|t| t["name"] == tool));
        }
        assert!(call_tool("situation", &json!({}), &side).is_none());
        // Giving the game up is the commander's, with a reason, and sets the side's flag for every seat.
        assert!(call_tool("surrender", &json!({}), &side).unwrap().is_err());
        assert!(!side.surrender.load(Ordering::Relaxed));
        assert!(call_tool("surrender", &json!({ "reason": "no factory left, his army in our base" }), &side).unwrap().is_ok());
        assert!(side.surrender.load(Ordering::Relaxed));
        assert!(call_tool("wait", &json!({ "max_seconds": 5 }), &side).unwrap().unwrap().contains("30 s"));
    }
}
