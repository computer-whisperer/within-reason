//! Minimal MCP server over streamable HTTP: the five methods Claude Code was observed to use
//! (`docs/harness/claude-p.md`), answered with plain JSON.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server};

use super::shared::{Allowance, PlanContext, Removal, Shared};
use super::transcript::Transcript;

/// The player's packet at most: Jev reads it every second beside a picture of about 4k tokens, within 64k.
const INSTRUCTIONS_LIMIT: usize = 8000;

pub struct McpServer {
    pub port: u16,
    stop: Arc<AtomicBool>,
}

impl McpServer {
    /// Serves on a free localhost port until dropped.
    pub fn start(shared: Arc<Shared>, transcript: Arc<Transcript>) -> std::io::Result<Self> {
        let server = Server::http("127.0.0.1:0").map_err(std::io::Error::other)?;
        let port = server.server_addr().to_ip().map_or(0, |addr| addr.port());
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        std::thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                let Ok(Some(mut request)) = server.recv_timeout(std::time::Duration::from_millis(250)) else { continue };
                if *request.method() != Method::Post {
                    // No server-initiated stream.
                    let _ = request.respond(Response::empty(405));
                    continue;
                }
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let reply = serde_json::from_str::<Value>(&body).ok().and_then(|call| handle(&call, &shared, &transcript));
                let _ = match reply {
                    Some(reply) => request.respond(
                        Response::from_string(reply.to_string())
                            .with_header(Header::from_bytes("Content-Type", "application/json").unwrap()),
                    ),
                    // Notifications carry no id and get no body.
                    None => request.respond(Response::empty(202)),
                };
            }
        });
        Ok(McpServer { port, stop })
    }
}

impl Drop for McpServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn handle(call: &Value, shared: &Arc<Shared>, transcript: &Transcript) -> Option<Value> {
    let id = call.get("id")?.clone();
    let method = call["method"].as_str().unwrap_or_default();
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": call["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "within-reason", "version": env!("CARGO_PKG_VERSION") },
        })),
        "tools/list" => Ok(json!({ "tools": tool_list() })),
        "tools/call" => {
            let name = call["params"]["name"].as_str().unwrap_or_default();
            let arguments = &call["params"]["arguments"];
            let outcome = call_recorded(name, arguments, shared, transcript);
            Ok(match outcome {
                Ok(text) => json!({ "content": [{ "type": "text", "text": text }] }),
                Err(problem) => json!({ "content": [{ "type": "text", "text": problem }], "isError": true }),
            })
        }
        _ => Err(json!({ "code": -32601, "message": format!("method not found: {method}") })),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

/// One tool call from the player, by the MCP server or the API backend (`api.rs`): refused once the turn's `wait`
/// has passed, the `orders` batch or a single tool, and recorded in the transcript in full (results, briefings
/// included: they are what the strategist decided on, and the labelled state for evaluating faster models against
/// its decisions).
pub(super) fn call_recorded(name: &str, arguments: &Value, shared: &Arc<Shared>, transcript: &Transcript) -> Result<String, String> {
    let outcome = if shared.turn_over.load(Ordering::Relaxed) {
        Err("Your turn is over and the game is running. Stop now: call nothing more and write nothing more. You will be woken with a new report.".to_string())
    } else if name == "orders" {
        orders(arguments, shared)
    } else {
        call_tool(name, arguments, shared)
    };
    let recorded = outcome.as_ref().map_or_else(
        |problem| Value::String(problem.clone()),
        |text| serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.clone())),
    );
    transcript.record(json!({ "kind": "tool_call", "tool": name, "arguments": arguments, "result": recorded }));
    outcome
}

/// The same tools in OpenAI's shape for the API backend: `function` with the MCP `inputSchema` as `parameters`.
pub(super) fn openai_tools() -> Vec<Value> {
    tool_list()
        .as_array()
        .map(|tools| tools.iter().map(|t| json!({ "type": "function", "function": { "name": t["name"], "description": t["description"], "parameters": t["inputSchema"] } })).collect())
        .unwrap_or_default()
}

/// The player's tools: its levers are the packet (`instruct`), the lists, the standing rules and the production
/// whitelist; its `situation` is the picture its hands read.
fn tool_list() -> Value {
    let overview = json!({ "name": "overview",
        "description": "Current state of the game as the bot sees it: time, economy, unit counts, our army groups, enemies in sight, remembered enemy buildings, recent events, and the directives in force.",
        "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false } });
    let map = json!({ "name": "map",
        "description": "Static map facts: size, grid naming, our start, the lobby's start boxes, and every metal spot with its grid cell.",
        "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false } });
    let note = json!({ "name": "note",
        "description": "Record your reasoning in a sentence or two. Kept with the game time for post-game analysis; it changes nothing in the game.",
        "inputSchema": { "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"], "additionalProperties": false } });
    let wait = |engaged: &str, pool: &str| json!({ "name": "wait",
        "description": "Ends your turn: the game resumes the moment this is called, so call it last and write nothing after it. Sets when you are next woken; the settings hold until you change them. The game is paused during your turn and runs fast between turns, so a long quiet wait costs nothing and a raid still wakes you at once. You are always woken for a base attack, the commander under fire, a wiped-out wave, a group losing soldiers or meeting a party it does not outweigh, and losses while your orders were on their way; while a fight is on the quiet time is capped at 10 s.",
        "inputSchema": { "type": "object", "additionalProperties": false, "properties": {
            "max_seconds": { "type": "integer", "minimum": 5, "maximum": 180, "description": "Game seconds after which you are woken whatever happens (default 30)." },
            "enemy_near_extractor": { "type": "boolean", "description": "Enemies appear within 600 of an extractor that had none near." },
            "squad_engaged": { "type": "boolean", "description": engaged },
            "extractor_lost": { "type": "boolean" },
            "pool_reaches": { "type": "object", "additionalProperties": { "type": "integer", "minimum": 1 }, "description": pool },
            "chat": { "type": "boolean", "description": "A person in the game says something (default on)." } } } });
    let orders = |tools: &[&str], what: &str| json!({ "name": "orders",
        "description": format!("Your whole turn in one call, and it ENDS the turn: {what}, carried out in the order listed, then the game resumes. Each entry names one of the other tools and its arguments, exactly as you would call it alone. Include a `wait` entry, LAST, to change when you are next woken; without one the wake settings in force stand. Never call the other tools on their own beside this one: anything sent after the turn ends is refused. Call nothing and write nothing after it."),
        "inputSchema": { "type": "object", "additionalProperties": false, "required": ["calls"], "properties": {
            "calls": { "type": "array", "minItems": 1, "items": { "type": "object", "additionalProperties": false, "required": ["tool"], "properties": {
                "tool": { "type": "string", "enum": tools },
                "arguments": { "type": "object", "description": "That tool's arguments, e.g. {\"text\": \"...\"} for note." } } } } } } });
    json!([
            overview, map,
            { "name": "situation",
              "description": "The picture your hands read this second, exactly as Jev sees it (without your instructions and the standing rules): economy, ours, enemy, places by name, every actor with what it is doing, recent events. You are sent a summary of it at the start of every turn; call this to read the whole picture, or a place's entry by name.",
              "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false } },
            { "name": "units",
              "description": "The glossary entry for units by internal name, ours or the opponent's (the roster on your first report lists ours; sightings and losses name theirs): what it is for, what it beats and loses to, when to build it, its numbers. Several names at once.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["names"], "properties": { "names": { "type": "array", "items": { "type": "string" }, "minItems": 1 } } } },
            { "name": "plan",
              "description": "Simulate a build order from the game as it stands now, without ordering anything: lists of steps per builder in the `queue` tool's words (`extractor spot_N`, `assist`, any unit's internal name with an optional spot or marked place; a factory's list is unit names), keyed by actor name (commander, lab_N, plant_N, factory_N, constructor_N) or next_factory_1, next_constructor_1 for what is not standing yet. Returns the curves by minute (extractors, income, stall, army metal, build power), the minute each unit type first finishes, and what each builder actually did with its list (steps it could not do are named). The simulator knows the economy and building; it knows nothing of the enemy, losses or terrain jams, and runs about ten percent optimistic.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["queues"], "properties": { "queues": { "type": "object" }, "minutes": { "type": "number", "description": "How far ahead to simulate (default 8, at most 20)." } } } },
            { "name": "search",
              "description": "Search build orders in the simulator from the game as it stands now, without ordering anything: an objective in words (`income`, `army`, `mix` (army plus two minutes of income), or `target`: goals after commas, each a unit with an optional count and time, or `income:N by M:SS` for metal a second by then, e.g. `target armflash:4 by 3:30, armbull by 9:00, income:40 by 10:00`; only what the goals name counts, so name the raiders and the income you want beside the heavy units, or the search leaves them out: a target order is a chain to the thing asked and nothing else), a horizon in minutes and a time budget in seconds (default 10). The search runs beside the game and its answer comes with your next report, for which you are woken; `wait: true` holds your turn for it instead (the cap is 3 s: every second of your turn is a second your orders land late, in a live game and under the arena's think penalty alike; 20 s only in an arena run with the penalty off). The search may use the whole roster: every generator, storage, converter, extractor, factory, nano and unit the faction reaches. Returns the best order found in the `plan` tool's words with its curves and score. Give `queues` to start the search from your own order.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["objective"], "properties": { "objective": { "type": "string" }, "minutes": { "type": "number" }, "seconds": { "type": "number" }, "wait": { "type": "boolean" }, "queues": { "type": "object" } } } },
            { "name": "instruct",
              "description": format!("Your standing instructions to your hands: the whole packet, replacing the last one. Jev reads it every second beside the picture and picks each actor's next action from a menu, so write it as standing orders in plain words: the build order per builder as a sequence, what the lab makes and when that changes, where each group stands, when it engages, scouts and attacks, what to do about raids. Name places as the picture does (home, spot_N, passage_N, and any place you marked with `mark`; a spot or passage you name here is always on your hands' menu, however far) and groups as group_A, group_B. No arithmetic for the hands to do: say \"when we have about ten soldiers\", not a formula. At most {INSTRUCTIONS_LIMIT} characters."),
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string" } } } },
            { "name": "queue",
              "description": "A builder's build list, done exactly and in order by the bot itself without asking your hands: an object of builder name (commander, constructor_N) to a list of steps, or null (or an empty list) to cancel its list; null leaves a build in progress to finish, a list whose first step is \"stop\" (or the bare word \"stop\") drops that build first, its frame decaying, and the rest of the list follows. Steps: \"extractor spot_N\" (or \"extractor\" for the nearest free spot), any building by its internal name as the roster lists it (\"armsolar\", \"armvp\", \"armnanotc\", \"armfus\"), one that stands at a place with the place after it (\"armllt spot_3\", \"armrad home\", \"armmoho spot_3\" over our extractor there); any other building may take a place too (\"armvp spot_39\", \"armnanotc home\") and without one a factory stands beside its builder toward the yard, a generator beside the builder when it stands at home, else in the back field at home (the builder walks: the pros' solars are at base), a construction turret beside the nearest factory (it guards that factory once it stands), \"assist N\" (help the nearest factory for N seconds, then the next step: the pros guard the plant between their own builds), \"assist\" (help the nearest factory, standing or being built: the last step of a list, and only the last; the tool refuses steps after it). Each step is ordered when the one before is 60% built, so nothing idles; a new list takes over a builder that is helping a factory, walking, reclaiming or repairing at once; a step that cannot be done (the spot taken, a place unknown, a building this builder cannot make) is skipped and said in the hands' report. While a list runs the builder is off your hands' menu unless an enemy is on it; your instructions take over when the list is done. This is how an opening is made to happen as written: the hands do not follow a sequence (comet-1, comet-2: 'three solars, then the plant' got extractors and the plant at 1:45).",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" }, { "type": "string", "enum": ["stop"] } ] }, "description": "Builder name to steps, null, or \"stop\"." } },
            { "name": "lane",
              "description": "Which footwork rules your hands' code applies to a group's soldiers between your hands' orders, per group name or for \"all\": \"raw\" (none: the group's orders reach the engine exactly as given), \"on\" (all of them, the default), or a list of the rules to keep. The rules: flee (a soldier steps out of the reach of a turret or a fight it was not sent against, or one it would die in), fan (spreads out under a commander's D-gun), kite (a soldier that outranges its enemy steps back while reloading), form (a group's soldiers walk to their own slots in ranks of six across the group's heading, two hulls apart, and stand at contact), march (an advancing group waits for its stragglers so it arrives together), follow (an engaging group is re-sent after its party as it moves). \"rove\" (a group, never \"all\") hands the group's soldiers to the code whole, ten times a second, for fast units (Rover, Tick, Blitz, scout cars): each drives to look at what we know least (his start box and base first, then spots never seen, then the stalest), kills what it finds unguarded (a constructor, an extractor, a radar: nothing armed within reach of it), and never stands inside the reach of anything that can shoot it, stepping off before it is; your hands never move a roving group (no hunt, retreat, join or fall-back) and its new soldiers rove too. The group's entry says what each rover is doing and what it has found (`rove`); \"on\" or any other setting takes it back, holding where each soldier stands. A setting stands until you change it; the group's picture entry shows it when it is not the default.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "string", "enum": ["raw", "on", "rove"] }, { "type": "array", "items": { "type": "string", "enum": ["flee", "fan", "kite", "form", "march", "follow"] } } ] }, "description": "Group name (group_A) or \"all\" to its setting." } },
            { "name": "mark",
              "description": "Name a place of your own for your hands: an object of name to [x, z] map coordinates or a grid cell (\"E7\": its centre), or null to forget it. A marked place joins the picture's places at once, so instructions can send groups and builders there (\"group_B: advance to south_gate\"), and its entry says what enemy is near. Names are lower-case words with underscores; home, spot_N, passage_N and group_N are taken. Any spot or passage you name in the packet is on your hands' menu already, however far; mark is for places that are not spots.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, { "type": "string" }, { "type": "null" } ] }, "description": "Place name to [x, z], a grid cell, or null." } },
            { "name": "produce",
              "description": "What each factory or builder may build, and for a factory which group its new soldiers join ({\"plant_7\": {\"units\": [\"armflash\"], \"group\": \"group_A\"}}, or \"new\" for a fresh group of that factory's own; by default a factory's soldiers gather in one group of its own and nothing merges by itself): an object of actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" (every commander and constructor) or \"all\" (everyone) to a list of unit names (as the roster writes them: armpw, armham, armck, armfus), or null to lift the restriction. A name with a count after a colon (armck:1) is allowed that many more times from now and then drops off the list by itself; saying the same list again restarts the count: the way to say 'one constructor, then raiders' to hands that cannot count. A lab with a list is offered only those units and nothing else, every time it is asked; your instructions still say which of them and when. A builder without a list is offered the usual buildings (generators, factories, light and heavy turrets, radar, storage, the tier-2 lab and extractor, fusion); a list replaces that, so a fusion reactor, an aircraft plant or a jammer from a constructor is asked for here. Use it when the packet's words are not getting the mix you want. A list naming nothing the actor can build leaves it unrestricted; the actor's entry in the picture shows its list. Your policy is not bound by lists: it may order anything a builder can build.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" } ] }, "description": "Actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\" to unit names, or null." } },
            { "name": "transfer",
              "description": "Move metal, energy or units between seats of ours (a game with several seats: the `seats:` line): {\"metal\": 800, \"energy\": 0, \"from\": \"t2\", \"to\": \"t1\"} sends what the receiving seat's store can hold (the game caps it there; what does not fit stays), or {\"units\": [\"group_A_t2\", \"constructor_31002\", \"plant_4410\"], \"to\": \"t1\"} gives those units (a group's members, a builder, a plant, by actor name or <unit>_<id> handle) to that seat, whose hands take them into a group of their own and whose builders they become. What it makes possible: one army under one seat, the advanced plant's seat fed metal by the others, a tier-2 constructor made by one seat and given to each, a dead seat's plants and constructors given to a live one. Your hands carry it out from their next look and say what went.",
              "inputSchema": { "type": "object", "properties": { "metal": { "type": "number" }, "energy": { "type": "number" }, "units": { "type": "array", "items": { "type": "string" } }, "from": { "type": "string" }, "to": { "type": "string" } }, "required": ["to"] } },
            { "name": "remove",
              "description": "Take apart or blow up what we own: {\"reclaim\": [handles], \"by\": \"constructor_N\" (optional; else the nearest builder without a list)} puts `reclaim <handle>` steps at the front of that builder's list, and most of the metal comes back; {\"destruct\": [handles]} sends the engine's self-destruct, and nothing comes back. A handle is a unit's name and id as the picture writes it (armsolar_31002: a factory's `yard` entry names the buildings in its exit lane) or an actor's name (constructor_N, plant_N, commander). Every unit that self-destructs blows up: the answer says the blast's radius and damage and what of ours stands inside it, and a destruct that would kill something of ours is refused unless \"accept_losses\": true. The commander's blast is the game's largest; a reclaim is the safe way beside anything that matters.",
              "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "reclaim": { "type": "array", "items": { "type": "string" } }, "by": { "type": "string" }, "destruct": { "type": "array", "items": { "type": "string" } }, "accept_losses": { "type": "boolean" } } } },
            { "name": "say",
              "description": "Say something in the game's chat, to everyone playing. Short lines: the game shows 127 characters a line, the bot prefixes `[WReason] ` to each (never write it yourself) and splits a longer text into lines that fit. The report shows what people say to you; when an experienced player offers advice or asks what you are doing, answer, and ask them what they would do: their feedback is what this project learns from.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string", "maxLength": 240 } } } },
            orders(&["instruct", "queue", "lane", "mark", "produce", "remove", "transfer", "say", "note", "wait"], "your instructions or policy, build lists, footwork settings, marked places, what labs may build, removals, a chat line, a note and when to be woken"),
            wait("A group of ours starts fighting an enemy party.", "Woken when this many soldiers of each named unit type are alive, e.g. {\"armham\": 6}. {} clears it."),
            note,
    ])
}

/// What `orders` may batch.
fn batchable() -> &'static [&'static str] {
    &["instruct", "queue", "lane", "mark", "produce", "remove", "transfer", "say", "note", "wait"]
}

/// The `orders` tool: several tool calls in one request. `wait` goes last wherever it was listed, since it ends the turn.
fn orders(arguments: &Value, shared: &Arc<Shared>) -> Result<String, String> {
    let calls = arguments["calls"].as_array().ok_or("calls must be a list")?;
    let (mut waits, others): (Vec<&Value>, Vec<&Value>) = calls.iter().partition(|c| c["tool"] == "wait");
    // `orders` is the whole turn: with no `wait` listed the wake settings stand and the turn ends all the same. (In
    // commander game 11 half the turns were `orders` and then a separate `wait`: a second request, 2 s against 5.)
    let keep = json!({ "tool": "wait" });
    if waits.is_empty() {
        waits.push(&keep);
    }
    let empty = json!({});
    let mut lines = Vec::new();
    for call in others.into_iter().chain(waits) {
        // As the model writes them: sometimes with the server's prefix on the name, and often with a small tool's
        // arguments beside `tool` instead of under `arguments` (40 of 46 notes in commander game 11, all recorded empty,
        // so the sessions that took over mid-game inherited blank notes).
        let tool = call["tool"].as_str().unwrap_or_default();
        let tool = tool.rsplit("__").next().unwrap_or(tool);
        if !batchable().contains(&tool) {
            lines.push(format!("{tool}: not a tool that can be batched"));
            continue;
        }
        let mut beside = call.clone();
        if let Some(fields) = beside.as_object_mut() {
            fields.remove("tool");
            fields.remove("arguments");
        }
        let arguments = call.get("arguments").filter(|a| a.is_object()).unwrap_or(if beside.as_object().is_some_and(|f| !f.is_empty()) { &beside } else { &empty });
        match call_tool(tool, arguments, shared) {
            Ok(text) => lines.push(format!("{tool}: {text}")),
            Err(problem) => lines.push(format!("{tool}: REFUSED: {problem}")),
        }
    }
    Ok(lines.join("\n"))
}

fn call_tool(name: &str, arguments: &Value, shared: &Arc<Shared>) -> Result<String, String> {
    if !tool_list().as_array().is_some_and(|tools| tools.iter().any(|t| t["name"] == name)) {
        return Err(format!("{name} is not a tool in this mode"));
    }
    match name {
        "overview" => serde_json::to_string(&shared.briefing()).map_err(|e| e.to_string()),
        "map" => Ok(shared.map.lock().unwrap().to_string()),
        "note" => {
            let time = shared.briefing().game_time;
            let text = arguments["text"].as_str().filter(|t| !t.trim().is_empty()).ok_or("a note needs its words under \"text\"")?;
            shared.notes.lock().unwrap().push(format!("[{time}] {text}"));
            Ok("noted".into())
        }
        "wait" => {
            let mut wake = shared.wake.lock().unwrap();
            if let Some(seconds) = arguments["max_seconds"].as_u64() {
                wake.max_seconds = seconds.clamp(5, 180) as u32;
            }
            let flag = |field: &str, slot: &mut bool| {
                if let Some(on) = arguments[field].as_bool() {
                    *slot = on;
                }
            };
            flag("enemy_near_extractor", &mut wake.enemy_near_extractor);
            flag("squad_engaged", &mut wake.squad_engaged);
            flag("extractor_lost", &mut wake.extractor_lost);
            flag("chat", &mut wake.chat);
            if let Some(pool) = arguments["pool_reaches"].as_object() {
                wake.pool_reaches = pool.iter().map(|(name, n)| (name.clone(), n.as_u64().unwrap_or(1) as usize)).collect();
            }
            let reply = format!("Turn over: the game is running and you will be woken with a new report. Stop now, call nothing more and write nothing more. Wake conditions in force until you change them: {}", serde_json::to_string(&*wake).map_err(|e| e.to_string())?);
            drop(wake);
            // The turn is over here, not when the model has finished its closing sentence: that sentence is one more
            // request to the model, a second or two with the game held (a third of a median turn, commander game 9).
            shared.end_turn_at_wait();
            Ok(reply)
        }
        "situation" => Ok(shared.hands_merged(false).picture.to_string()),
        "instruct" => {
            let text = arguments["text"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("instruct needs the packet under \"text\"")?;
            if text.chars().count() > INSTRUCTIONS_LIMIT {
                return Err(format!("the packet is {} characters; at most {INSTRUCTIONS_LIMIT}. Cut it: your hands read it every second", text.chars().count()));
            }
            *shared.instructions.lock().unwrap() = text.to_string();
            Ok(format!("instructions replaced ({} characters); your hands read them from their next look, once your turn ends", text.chars().count()))
        }
        "plan" => {
            let context = shared.plan_context.lock().unwrap().clone().ok_or("the simulator has no picture of the game yet: try again in a few seconds")?;
            let marks = shared.marks.lock().unwrap().clone();
            planning::run(name, arguments, 0.0, &context, &marks)
        }
        "search" => {
            // Beside the game by default, the answer with the next report; `wait` holds the turn for it. The budget's
            // cap is the mode's: lockstep holds the game anyway, realtime does not (the user, 2026-09-22: in-game
            // thinking time can cost games).
            let context = shared.plan_context.lock().unwrap().clone().ok_or("the simulator has no picture of the game yet: try again in a few seconds")?;
            let marks = shared.marks.lock().unwrap().clone();
            // The objective is checked now, not on the thread, so a wrong one is refused at once.
            let text = arguments["objective"].as_str().ok_or("search takes an objective in words")?.trim();
            if text.starts_with("target") {
                buildorder::anneal::Objective::parse_target(text, &context.game.units)?;
            } else if buildorder::anneal::Objective::parse(text).is_none() {
                return Err(format!("{text}: not an objective ({})", planning::OBJECTIVES));
            }
            // The long cap only where the world truly waits: in lockstep with no think penalty. Under the penalty a
            // held turn's seconds are game seconds the orders land late, as in a live game.
            let world_waits = shared.lockstep.load(Ordering::Relaxed) && *shared.think_penalty.lock().unwrap() <= 0.0;
            let cap = if world_waits { planning::MAX_SECONDS } else { planning::REALTIME_MAX_SECONDS };
            let seconds = arguments["seconds"].as_f64().unwrap_or(planning::DEFAULT_SECONDS).clamp(1.0, cap);
            if arguments["wait"].as_bool().unwrap_or(false) {
                return planning::run(name, arguments, seconds, &context, &marks);
            }
            let (board, arguments) = (Arc::clone(shared), arguments.clone());
            std::thread::spawn(move || {
                let answer = planning::run("search", &arguments, seconds, &context, &marks).unwrap_or_else(|e| format!("the search failed: {e}"));
                board.search_results.lock().unwrap().push(answer);
                board.trigger("your search has finished: its answer is at the top of this report".into());
            });
            Ok(format!(
                "searching for {seconds:.0} s beside the game; the answer comes with your next report, and you will be woken for it{}. Pass \"wait\": true to hold the game and have it now (at most {cap:.0} s in this mode).",
                if world_waits { "" } else { " while the game runs on" }
            ))
        }
        "units" => {
            let names = arguments["names"].as_array().ok_or("units takes {\"names\": [\"armpw\", ...]}")?;
            let mut lines: Vec<String> = Vec::new();
            for name in names {
                let name = name.as_str().ok_or("names are strings")?;
                let blast_words = |b: Option<(f32, f32)>| b.map_or("none".to_string(), |(r, d)| format!("radius {r:.0}, damage {d:.0}"));
                let blasts = shared.blasts.lock().unwrap().get(name).map(|(death, selfd, secs)| format!(" Blasts: dying {}; self-destruct {} after {secs:.0} s.", blast_words(*death), blast_words(*selfd))).unwrap_or_default();
                lines.push(match crate::brain::pianist::glossary::entry(name) {
                    Some(e) => format!(
                        "{} ({name}): {}, tier {}{}. {}. {}{}{blasts}",
                        e.name, e.class, e.tier,
                        if e.made_by.is_empty() { String::new() } else { format!(", made by {}", e.made_by.join(", ")) },
                        e.numbers(),
                        if e.prose.is_empty() { e.gloss.clone() } else { e.prose.clone() },
                        if e.air.is_empty() { String::new() } else { format!(" Against aircraft: {}.", e.air) }
                    ),
                    None => format!("{name}: not in the glossary{blasts}"),
                });
            }
            Ok(lines.join("\n"))
        }
        "queue" => {
            // A step is an extractor, help, or any unit the roster knows (the field's roster, when the bot has
            // published one; before that any word, and the bot skips what it cannot do and says so).
            let roster: Vec<String> = shared.field().roster.iter().map(|(name, _)| name.clone()).collect();
            let lists = arguments.as_object().filter(|o| !o.is_empty()).ok_or("queue takes an object: builder name (commander or constructor_N) to a list of steps, or null to cancel")?;
            let mut parsed: Vec<(String, Option<Vec<String>>)> = Vec::new();
            for (name, value) in lists {
                if !name.starts_with("commander") && !name.starts_with("constructor_") {
                    return Err(format!("{name}: lists are by builder name (commander, or commander_tN beside other seats of ours, or constructor_N)"));
                }
                // As the model writes them (comet-5, 24:11: "cannot cancel the commander's old solar list, the queue
                // tool rejects every form I try"): the list as a JSON string, "null" as a string, one step as a bare
                // string, an empty list to cancel.
                let value = match value {
                    Value::String(s) if s.trim() == "null" => Value::Null,
                    Value::String(s) if s.trim() == "stop" => Value::Array(vec![Value::String("stop".into())]),
                    Value::String(s) if s.trim_start().starts_with('[') => serde_json::from_str(s).map_err(|e| format!("{name}: {e}"))?,
                    Value::String(s) => Value::Array(vec![Value::String(s.clone())]),
                    other => other.clone(),
                };
                let list = match &value {
                    Value::Null => None,
                    Value::Array(items) if items.is_empty() => None,
                    Value::Array(items) => {
                        let steps: Vec<String> = items.iter().map(|v| v.as_str().map(|s| s.trim().to_string()).ok_or_else(|| format!("{name}: steps are strings"))).collect::<Result<_, _>>()?;
                        for (i, step) in steps.iter().enumerate() {
                            let mut words = step.split_whitespace();
                            let kind = words.next().unwrap_or_default();
                            if kind == "stop" && i > 0 {
                                return Err(format!("{name}: 'stop' only begins a list (it drops the build in progress, then the list follows)"));
                            }
                            if !matches!(kind, "extractor" | "assist" | "stop") && !roster.is_empty() && !roster.contains(&kind.to_string()) {
                                return Err(format!("{name}: '{step}' is not a step; a step is extractor, assist, or a unit's internal name from the roster (armsolar, armllt spot_3)"));
                            }
                            if kind == "assist" && i + 1 < steps.len() {
                                // `assist N` (seconds) sits anywhere: the pros' rhythm after the plant is guard it,
                                // one solar, guard it (K-map-comet-catcher-remake-1-8-commander-assists-the-plant).
                                let seconds = words.next().and_then(|w| w.parse::<u32>().ok());
                                if !seconds.is_some_and(|n| (5..=600).contains(&n)) {
                                    return Err(format!("{name}: a bare 'assist' ends a list (the builder helps the nearest factory until you give it a new list); in the middle of one write 'assist N' with N seconds (5 to 600), and the list goes on after them"));
                                }
                            }
                        }
                        Some(steps)
                    }
                    _ => return Err(format!("{name}: a list of steps, or null")),
                };
                parsed.push((name.clone(), list));
            }
            let mut queues = shared.queues.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, list) in parsed {
                said.push(match &list {
                    Some(steps) if steps.len() == 1 && steps[0] == "stop" => format!("{name}: list cancelled and its build in progress dropped"),
                    Some(steps) if steps[0] == "stop" => format!("{name}: its build in progress dropped, then {} steps in order from its next look", steps.len() - 1),
                    Some(steps) => format!("{name}: {} steps, done in order from its next look", steps.len()),
                    None => format!("{name}: list cancelled; a build in progress finishes (begin the list with \"stop\" to drop it)"),
                });
                queues.insert(name, list);
            }
            Ok(said.join("; "))
        }
        "lane" => {
            let settings = arguments.as_object().filter(|o| !o.is_empty()).ok_or("lane takes an object: group name (or \"all\") to \"raw\", \"on\" or a list of the rules to keep")?;
            let mut parsed: Vec<(String, super::shared::Footwork)> = Vec::new();
            for (name, value) in settings {
                if name != "all" && !name.starts_with("group_") {
                    return Err(format!("{name}: settings are by group name (group_A) or \"all\""));
                }
                let footwork = match value {
                    Value::String(s) if s == "raw" => super::shared::Footwork::raw(),
                    Value::String(s) if s == "on" => super::shared::Footwork::default(),
                    Value::String(s) if s == "rove" && name == "all" => return Err("rove is per group (group_A): a whole army that roves has nobody left to fight".into()),
                    Value::String(s) if s == "rove" => super::shared::Footwork::rove(),
                    Value::Array(items) => {
                        let names: Vec<String> = items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{name}: rule names are strings"))).collect::<Result<_, _>>()?;
                        super::shared::Footwork::keeping(&names)?
                    }
                    _ => return Err(format!("{name}: \"raw\", \"on\", \"rove\" or a list of rules to keep")),
                };
                parsed.push((name.clone(), footwork));
            }
            let mut lane = shared.lane.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, footwork) in parsed {
                said.push(format!("{name}: {}", footwork.words().unwrap_or_else(|| "all footwork rules (the default)".to_string())));
                lane.insert(name, footwork);
            }
            Ok(format!("footwork set; {}", said.join("; ")))
        }
        "mark" => {
            let marks = arguments.as_object().filter(|o| !o.is_empty()).ok_or("mark takes an object: place name to [x, z], a grid cell such as \"E7\", or null to forget it")?;
            let map = shared.map.lock().unwrap().clone();
            let (width, height) = (map["width"].as_f64().unwrap_or(0.0) as f32, map["height"].as_f64().unwrap_or(0.0) as f32);
            let mut parsed: Vec<(String, Option<(f32, f32)>)> = Vec::new();
            for (name, value) in marks {
                let word = !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') && name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
                let taken = name == "home" || name == "all" || ["spot_", "passage_", "group_"].iter().any(|p| name.starts_with(p));
                if !word || taken {
                    return Err(format!("{name}: a mark's name is lower-case words with underscores, and home, spot_N, passage_N and group_N are taken"));
                }
                let at = match value {
                    Value::Null => None,
                    Value::Array(xz) if xz.len() == 2 => {
                        let (x, z) = (xz[0].as_f64().ok_or(format!("{name}: [x, z] are numbers"))? as f32, xz[1].as_f64().ok_or(format!("{name}: [x, z] are numbers"))? as f32);
                        if width > 0.0 && (x < 0.0 || z < 0.0 || x > width || z > height) {
                            return Err(format!("{name}: ({x:.0}, {z:.0}) is off the map, which is {width:.0} by {height:.0}"));
                        }
                        Some((x, z))
                    }
                    Value::String(cell) => {
                        let mut chars = cell.chars();
                        let column = chars.next().map(|c| c.to_ascii_uppercase()).filter(|c| ('A'..='H').contains(c)).ok_or(format!("{name}: a grid cell is A1 to H8"))?;
                        let row: u32 = chars.as_str().parse().ok().filter(|r| (1..=8).contains(r)).ok_or(format!("{name}: a grid cell is A1 to H8"))?;
                        if width <= 0.0 {
                            return Err("the map is not known yet; give [x, z]".to_string());
                        }
                        Some(((column as u32 - 'A' as u32) as f32 * width / 8.0 + width / 16.0, (row - 1) as f32 * height / 8.0 + height / 16.0))
                    }
                    _ => return Err(format!("{name}: [x, z], a grid cell such as \"E7\", or null")),
                };
                parsed.push((name.clone(), at));
            }
            let mut marks = shared.marks.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, at) in parsed {
                match at {
                    Some((x, z)) => {
                        marks.insert(name.clone(), (x, z));
                        said.push(format!("{name} at ({x:.0}, {z:.0})"));
                    }
                    None => {
                        marks.remove(&name);
                        said.push(format!("{name} forgotten"));
                    }
                }
            }
            Ok(format!("marked: {}; your hands see the place from their next look", said.join("; ")))
        }
        "say" => {
            let text = arguments["text"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("say needs text")?;
            let text: String = text.chars().take(240).collect();
            shared.chat_out.lock().unwrap().push(text.clone());
            Ok(format!("said: {text}"))
        }
        "produce" => {
            let lists = arguments.as_object().filter(|o| !o.is_empty()).ok_or("produce takes an object: factory name (lab_N or plant_N) or \"all\" to a list of unit names, or to {\"units\": [...], \"group\": \"group_A\" | \"new\"} for a factory")?;
            let known: Vec<String> = shared.field().roster.iter().map(|(name, _)| name.clone()).collect();
            let groups: Vec<String> = shared.hands_merged(false).picture["actors"].as_object().map(|a| a.keys().filter(|k| k.starts_with("group_")).cloned().collect()).unwrap_or_default();
            let check_units = |name: &str, items: &[Value]| -> Result<Vec<String>, String> {
                let units: Vec<String> = items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{name}: unit names are strings"))).collect::<Result<_, _>>()?;
                // "corck:1": at most one of it from now, then the rest of the list (human-7: told "one
                // constructor first, then raiders", the hands, who cannot count, made three).
                for unit in &units {
                    let (unit_name, cap) = crate::brain::pianist::allowance(unit);
                    if !known.is_empty() && !known.contains(&unit_name.to_string()) {
                        return Err(format!("{unit_name} is not a unit of our roster (internal names as the roster lists them)"));
                    }
                    if unit.contains(':') && cap.is_none() {
                        return Err(format!("{unit}: a count after the colon is a whole number of one or more (corck:1)"));
                    }
                }
                Ok(units)
            };
            // (actor, units or None to clear, the group its soldiers join: Some(Some(name)) sets, Some(None) clears, None leaves)
            let mut parsed: Vec<(String, Option<Option<Vec<String>>>, Option<Option<String>>)> = Vec::new();
            let mut factories: Vec<String> = shared.hands_merged(false).picture["actors"].as_object().map(|a| a.keys().filter(|k| ["lab_", "plant_", "factory_"].iter().any(|p| k.starts_with(p))).cloned().collect()).unwrap_or_default();
            // ... and a factory still going up, which the picture does not list yet (bluegecko-3v1-comet-catcher-8,
            // 1:05: the plants' limits refused whole, "not a factory in the picture", the plants standing by 1:17).
            for card in shared.own_cards.lock().unwrap().values().flatten() {
                if let Some(actor) = &card.actor
                    && ["lab_", "plant_", "factory_"].iter().any(|p| actor.starts_with(p))
                    && !factories.contains(actor)
                {
                    factories.push(actor.clone());
                }
            }
            for (name, value) in lists {
                let factory = ["lab_", "plant_", "factory_"].iter().any(|p| name.starts_with(p));
                if !matches!(name.as_str(), "all" | "all_builders" | "commander") && !factory && !name.starts_with("constructor_") {
                    return Err(format!("{name}: lists are by actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\""));
                }
                // A factory the picture does not name gets nothing (models-medium-gpt56-terra: five lists for "lab_1",
                // a name the player made up, accepted and never reaching a lab).
                if factory && !factories.contains(name) {
                    return Err(format!("{name}: not a factory in the picture ({}); use its name as the report gives it, or \"all\"", if factories.is_empty() { "none named yet".to_string() } else { factories.join(", ") }));
                }
                // A list written as a JSON string, as `queue` takes it (models-medium-sonnet5: eight refusals).
                let unpacked;
                let value = match value {
                    Value::String(s) => {
                        unpacked = serde_json::from_str::<Value>(s).ok().filter(Value::is_array).unwrap_or_else(|| Value::Array(s.split(',').map(|u| Value::String(u.trim().to_string())).filter(|u| u != "").collect()));
                        &unpacked
                    }
                    other => other,
                };
                match value {
                    Value::Null => parsed.push((name.clone(), Some(None), None)),
                    Value::Array(items) => parsed.push((name.clone(), Some(Some(check_units(name, items)?)), None)),
                    Value::Object(fields) => {
                        if !factory {
                            return Err(format!("{name}: the object form (units and group) is for a factory"));
                        }
                        let units = match fields.get("units") {
                            None => None,
                            Some(Value::Array(items)) => Some(Some(check_units(name, items)?)),
                            Some(Value::Null) => Some(None),
                            Some(_) => return Err(format!("{name}: units is a list of unit names")),
                        };
                        let group = match fields.get("group") {
                            None => None,
                            Some(Value::Null) => Some(None),
                            Some(Value::String(g)) if g == "new" => Some(Some("new".to_string())),
                            Some(Value::String(g)) if groups.contains(g) => Some(Some(g.clone())),
                            Some(Value::String(g)) => return Err(format!("{name}: {g} is not a group in the picture ({}); \"new\" makes one", if groups.is_empty() { "none yet".to_string() } else { groups.join(", ") })),
                            Some(_) => return Err(format!("{name}: group is a group name or \"new\"")),
                        };
                        if units.is_none() && group.is_none() {
                            return Err(format!("{name}: the object form takes units and/or group"));
                        }
                        parsed.push((name.clone(), units, group));
                    }
                    _ => return Err(format!("{name}: a list of unit names, null, or {{\"units\": [...], \"group\": ...}}")),
                }
            }
            let mut allowed = shared.allowed.lock().unwrap();
            let call = shared.produce_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let mut said: Vec<String> = Vec::new();
            for (name, units, group) in parsed {
                let before = allowed.get(&name).cloned();
                let units_now = match units {
                    Some(Some(units)) => {
                        said.push(format!("{name} may build only {}", if units.is_empty() { "nothing".to_string() } else { units.join(", ") }));
                        Some(units)
                    }
                    Some(None) => {
                        said.push(format!("{name} may build anything"));
                        None
                    }
                    None => before.as_ref().map(|a| a.units.clone()),
                };
                let group_now = match group {
                    Some(Some(g)) => {
                        said.push(if g == "new" { format!("{name}'s new soldiers form a group of their own from now") } else { format!("{name}'s new soldiers join {g}") });
                        Some(g)
                    }
                    Some(None) => {
                        said.push(format!("{name}'s new soldiers gather in its own group again"));
                        None
                    }
                    None => before.as_ref().and_then(|a| a.group.clone()),
                };
                match (units_now, group_now) {
                    (None, None) => {
                        allowed.remove(&name);
                    }
                    (units, group) => {
                        allowed.insert(name, Allowance { call: if units.is_some() { call } else { before.map_or(call, |a| a.call) }, units: units.unwrap_or_default(), group });
                    }
                }
            }
            Ok(format!("{}; they see it from their next look", said.join("; ")))
        }
        "transfer" => {
            use super::shared::Transfer;
            let team_of = |v: &Value| -> Result<i32, String> {
                let text = v.as_str().ok_or("from and to are seat tags such as \"t2\"")?;
                text.trim().trim_start_matches('t').parse::<i32>().map_err(|_| format!("{text}: a seat tag is t<team>, as the seats line writes it"))
            };
            let to = team_of(&arguments["to"])?;
            let live = shared.live_seats();
            if !live.contains(&to) {
                return Err(format!("t{to} is not a seat of ours in this game (ours: {})", live.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(", ")));
            }
            let metal = arguments["metal"].as_f64().unwrap_or(0.0) as f32;
            let energy = arguments["energy"].as_f64().unwrap_or(0.0) as f32;
            let units: Vec<String> = arguments["units"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.trim().to_string())).collect()).unwrap_or_default();
            if metal <= 0.0 && energy <= 0.0 && units.is_empty() {
                return Err("transfer takes metal and/or energy with from and to, or units with to".into());
            }
            let mut said: Vec<String> = Vec::new();
            if metal > 0.0 || energy > 0.0 {
                let from = team_of(&arguments["from"])?;
                if from == to {
                    return Err("from and to are the same seat".into());
                }
                if !live.contains(&from) {
                    return Err(format!("t{from} is not a seat of ours in this game"));
                }
                shared.transfers.lock().unwrap().push(Transfer::Resources { from_team: from, to_team: to, metal, energy });
                said.push(format!("t{from} sends {metal:.0} metal and {energy:.0} energy to t{to}; what t{to}'s store cannot hold stays with t{from}"));
            }
            if !units.is_empty() {
                let cards: Vec<super::shared::UnitCard> = shared.own_cards.lock().unwrap().values().flatten().cloned().collect();
                let hands = shared.hands_merged(false);
                let groups: Vec<String> = hands.picture["actors"].as_object().map(|o| o.keys().filter(|k| k.starts_with("group_")).cloned().collect()).unwrap_or_default();
                for h in &units {
                    if !groups.contains(h) && !cards.iter().any(|c| c.handle == *h || c.actor.as_deref() == Some(h.as_str())) {
                        return Err(format!("{h}: nothing of ours has that name (a group_X, an actor name, or a <unit>_<id> handle as the picture writes it)"));
                    }
                }
                shared.transfers.lock().unwrap().push(Transfer::Units { handles: units.clone(), to_team: to });
                said.push(format!("{} go to t{to}: its hands take them into a group of their own, its builders as its own", units.join(", ")));
            }
            Ok(format!("{}; your hands carry it out from their next look", said.join("; ")))
        }
        "remove" => {
            let cards: Vec<super::shared::UnitCard> = shared.own_cards.lock().unwrap().values().flatten().cloned().collect();
            if cards.is_empty() {
                return Err("your hands have not published our units yet: try again in a few seconds".into());
            }
            let list = |key: &str| -> Result<Vec<String>, String> {
                match &arguments[key] {
                    Value::Null => Ok(Vec::new()),
                    Value::String(s) => Ok(vec![s.trim().to_string()]),
                    Value::Array(items) => items.iter().map(|v| v.as_str().map(|s| s.trim().to_string()).ok_or_else(|| format!("{key}: handles are strings"))).collect(),
                    _ => Err(format!("{key}: a list of handles")),
                }
            };
            let (reclaim, destruct) = (list("reclaim")?, list("destruct")?);
            if reclaim.is_empty() && destruct.is_empty() {
                return Err("remove takes {\"reclaim\": [handles]} and/or {\"destruct\": [handles]}; a handle is <unit>_<id> as the picture writes it (armsolar_31002) or an actor name".into());
            }
            let find = |h: &str| cards.iter().find(|c| c.handle == h || c.actor.as_deref() == Some(h));
            for h in reclaim.iter().chain(&destruct) {
                find(h).ok_or_else(|| format!("{h}: nothing of ours has that name (handles are <unit>_<id> as the picture writes them, e.g. armsolar_31002, or an actor name)"))?;
            }
            let by = arguments["by"].as_str().map(str::to_string);
            if let Some(b) = &by
                && !cards.iter().any(|c| c.actor.as_deref() == Some(b) && c.handle != *b)
            {
                return Err(format!("{b}: not a builder standing now"));
            }
            let accept = arguments["accept_losses"].as_bool().unwrap_or(false);
            let mut said: Vec<String> = Vec::new();
            let mut refused: Vec<String> = Vec::new();
            for h in &destruct {
                let c = find(h).expect("checked above");
                match c.self_destruct {
                    None => said.push(format!("{h} self-destructs in {:.0} s with no blast", c.self_destruct_seconds)),
                    Some((radius, damage)) => {
                        let inside: Vec<&_> = cards.iter().filter(|o| o.handle != c.handle && (o.at.0 - c.at.0).hypot(o.at.1 - c.at.1) <= radius).collect();
                        let killed: Vec<String> = inside.iter().filter(|o| o.health <= damage).map(|o| format!("{} ({:.0} health)", o.actor.clone().unwrap_or_else(|| o.handle.clone()), o.health)).collect();
                        if !killed.is_empty() && !accept {
                            refused.push(format!("{h}: its blast (radius {radius:.0}, damage {damage:.0}) would kill {} of ours: {}; pass \"accept_losses\": true to do it anyway, or reclaim it instead", killed.len(), killed.join(", ")));
                            continue;
                        }
                        said.push(format!(
                            "{h} self-destructs in {:.0} s: blast radius {radius:.0}, damage {damage:.0}; of ours within it: {}",
                            c.self_destruct_seconds,
                            if inside.is_empty() { "nothing".to_string() } else { inside.iter().map(|o| format!("{} ({:.0} health{})", o.actor.clone().unwrap_or_else(|| o.handle.clone()), o.health, if o.health <= damage { ", dies" } else { "" })).collect::<Vec<_>>().join(", ") }
                        ));
                    }
                }
            }
            if !refused.is_empty() {
                return Err(refused.join("; "));
            }
            for h in &reclaim {
                let c = find(h).expect("checked above");
                said.push(format!("{h} ({}, {:.0} metal) is taken apart by {}; most of its metal comes back", c.unit, c.metal, by.as_deref().unwrap_or("the nearest builder without a list")));
            }
            let mut removals = shared.removals.lock().unwrap();
            if !destruct.is_empty() {
                removals.push(Removal::Destruct { targets: destruct });
            }
            if !reclaim.is_empty() {
                removals.push(Removal::Reclaim { targets: reclaim, by });
            }
            Ok(format!("{}; your hands carry it out from their next look", said.join("; ")))
        }
        _ => Err(format!("unknown tool {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The simulator's view of a recorded game, as the brain would publish it, for the tools' tests.
    fn plan_context() -> Arc<PlanContext> {
        let game = buildorder::record::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../buildorder/tests/fixtures/quicksilver-nw.jsonl"), 60.0).unwrap().game;
        let ground = game.ground();
        let scenario = game.scenario(game.own_half(), ground);
        let state = buildorder::sim::State::start(&scenario);
        let wind = game.mean_wind();
        let spots = game.spots.iter().map(|(at, _)| *at).collect();
        Arc::new(PlanContext { game: Arc::new(game), scenario, state, actors: vec![("commander".into(), 0)], standing_factories: 0, standing_constructors: 0, spots, spot_radius: 120.0, turret: None, water: true, wind, frame: 0 })
    }

    #[test]
    fn plan_simulates_the_players_words_and_search_answers_beside_the_game() {
        let shared = Arc::new(Shared::default());
        assert!(call_tool("plan", &json!({ "queues": { "commander": ["extractor", "extractor", "armwin", "armlab"] } }), &shared).is_err(), "no picture yet");
        *shared.plan_context.lock().unwrap() = Some(plan_context());
        let planned = call_tool("plan", &json!({ "queues": { "commander": ["extractor", "extractor", "armwin", "armlab", "assist"], "next_factory_1": ["armck", "armpw"] }, "minutes": 4 }), &shared).unwrap();
        assert!(planned.contains("curves") && planned.contains("armlab at") && planned.contains("next_factory_1"), "{planned}");
        assert!(call_tool("plan", &json!({ "queues": { "commander": ["armnothing"] } }), &shared).unwrap_err().contains("not a unit"));
        // Beside the game: an answer at once, the result and a wake within the budget and a little.
        let started = call_tool("search", &json!({ "objective": "income", "minutes": 3, "seconds": 1 }), &shared).unwrap();
        assert!(started.starts_with("searching for 1 s beside the game"), "{started}");
        let mut waited = 0;
        while shared.search_results.lock().unwrap().is_empty() && waited < 100 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            waited += 1;
        }
        let results = shared.search_results.lock().unwrap().clone();
        assert_eq!(results.len(), 1, "the answer is posted for the next report");
        assert!(results[0].contains("score") && results[0].contains("curves"), "{}", results[0]);
        assert!(shared.triggers.lock().unwrap().iter().any(|t| t.contains("search has finished")));
        // Held: the answer now, within the mode's cap.
        let held = call_tool("search", &json!({ "objective": "target armpw:4 by 2:30, armck by 3:00, income:5 by 3:00", "minutes": 3, "seconds": 1, "wait": true }), &shared).unwrap();
        assert!(held.contains("score") && held.contains("armpw"), "{held}");
        assert!(held.contains("goals: income: 5 a second asked for by 3:00, ") && held.contains("; armpw: 4 asked, ") && held.contains("; armck: "), "each goal is answered: {held}");
        assert!(call_tool("search", &json!({ "objective": "target income:40" }), &shared).unwrap_err().contains("needs its time"));
        assert!(call_tool("search", &json!({ "objective": "glory" }), &shared).unwrap_err().contains("not an objective"));
        assert!(call_tool("search", &json!({ "objective": "target armpw, glory" }), &shared).unwrap_err().contains("no such unit"));
    }

    #[test]
    fn remove_names_the_blast_and_refuses_to_kill_our_own() {
        use super::super::shared::{Removal, UnitCard};
        let shared = Arc::new(Shared::default());
        assert!(call_tool("remove", &json!({ "destruct": ["armsolar_7"] }), &shared).unwrap_err().contains("not published"));
        let card = |handle: &str, actor: Option<&str>, at: (f32, f32), health: f32, blast: Option<(f32, f32)>| UnitCard { handle: handle.into(), actor: actor.map(str::to_string), unit: handle.split('_').next().unwrap().into(), at, health, metal: 155.0, self_destruct: blast, self_destruct_seconds: 5.0 };
        shared.own_cards.lock().unwrap().insert(0, vec![
            card("armsolar_7", None, (100.0, 100.0), 400.0, Some((120.0, 500.0))),
            card("armavp_3", Some("plant_3"), (150.0, 100.0), 3000.0, Some((300.0, 2000.0))),
            card("armck_9", Some("constructor_9"), (160.0, 120.0), 300.0, None),
            card("armmex_4", None, (900.0, 900.0), 300.0, Some((50.0, 100.0))),
        ]);
        assert!(call_tool("remove", &json!({}), &shared).unwrap_err().contains("remove takes"));
        assert!(call_tool("remove", &json!({ "reclaim": ["armsolar_99"] }), &shared).unwrap_err().contains("nothing of ours has that name"));
        let refused = call_tool("remove", &json!({ "destruct": ["armsolar_7"] }), &shared).unwrap_err();
        assert!(refused.contains("would kill 1 of ours: constructor_9 (300 health)") && refused.contains("accept_losses"), "{refused}");
        assert!(shared.removals.lock().unwrap().is_empty(), "a refused destruct orders nothing");
        let done = call_tool("remove", &json!({ "destruct": ["armmex_4"], "reclaim": ["armsolar_7"], "by": "constructor_9" }), &shared).unwrap();
        assert!(done.contains("armmex_4 self-destructs in 5 s: blast radius 50, damage 100; of ours within it: nothing") && done.contains("armsolar_7 (armsolar, 155 metal) is taken apart by constructor_9"), "{done}");
        let removals = shared.removals.lock().unwrap().clone();
        assert!(matches!(&removals[0], Removal::Destruct { targets } if targets == &["armmex_4".to_string()]));
        assert!(matches!(&removals[1], Removal::Reclaim { targets, by: Some(b) } if targets == &["armsolar_7".to_string()] && b == "constructor_9"));
        let accepted = call_tool("remove", &json!({ "destruct": ["armsolar_7"], "accept_losses": true }), &shared).unwrap();
        assert!(accepted.contains("constructor_9 (300 health, dies)") && accepted.contains("plant_3 (3000 health)"), "{accepted}");
        assert!(call_tool("remove", &json!({ "reclaim": ["armsolar_7"], "by": "constructor_77" }), &shared).unwrap_err().contains("not a builder standing now"));
    }

    #[test]
    fn the_player_has_its_lever_and_none_of_the_commanders() {
        let player: Vec<String> = tool_list().as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
        assert_eq!(player, ["overview", "map", "situation", "units", "plan", "search", "instruct", "queue", "lane", "mark", "produce", "transfer", "remove", "say", "orders", "wait", "note"]);
        for tool in batchable() {
            assert!(player.contains(&tool.to_string()));
        }
        let shared = Arc::new(Shared::default());
        assert!(call_tool("instruct", &json!({ "text": "commander: build the lab first." }), &shared).is_ok());
        assert_eq!(*shared.instructions.lock().unwrap(), "commander: build the lab first.");
        assert!(call_tool("instruct", &json!({ "text": "x".repeat(INSTRUCTIONS_LIMIT + 1) }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["extractor spot_45", "armsolar", "armvp", "assist"], "constructor_7": null }), &shared).is_ok());
        assert_eq!(shared.queues.lock().unwrap().get("commander").cloned().flatten().map(|s| s.len()), Some(4));
        assert!(shared.queues.lock().unwrap().contains_key("constructor_7"));
        // `assist N` sits in the middle of a list; a bare `assist` only ends one.
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist 20", "armsolar", "assist 30", "extractor spot_36", "assist"] }), &shared).is_ok());
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist", "armsolar"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist 2", "armsolar"] }), &shared).is_err());
        // With a roster published, a step must be a unit of it; without one any word passes and the bot skips it.
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armllt".into(), 85)], ..Default::default() });
        assert!(call_tool("queue", &json!({ "commander": ["turret spot_3"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["armllt spot_3", "armsolar"] }), &shared).is_ok());
        // Every player tool but the readers and `orders` itself can be batched (comet-3: `queue` was refused by the
        // batch as "not a tool that can be batched" and the game ran without the list).
        for tool in ["instruct", "queue", "lane", "mark", "produce", "say", "note", "wait"] {
            assert!(batchable().contains(&tool), "{tool}");
        }
        assert!(orders(&json!({ "calls": [{ "tool": "queue", "arguments": { "commander": ["armsolar"] } }] }), &shared).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "group_A": ["armsolar"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["windmill"] }), &shared).is_err());
        // The shapes the model actually sent to cancel or replace a list (comet-5).
        for form in [json!({ "commander": "null" }), json!({ "commander": [] }), json!({ "commander": null })] {
            assert!(call_tool("queue", &form, &shared).unwrap().contains("cancelled"), "{form}");
            assert!(shared.queues.lock().unwrap().get("commander").cloned().flatten().is_none());
            let said = call_tool("queue", &json!({ "constructor_7": "stop", "commander": ["stop", "armsolar"] }), &shared).unwrap();
            assert!(said.contains("constructor_7: list cancelled and its build in progress dropped") && said.contains("commander: its build in progress dropped, then 1 steps"), "{said}");
            assert_eq!(shared.queues.lock().unwrap().get("constructor_7").cloned().flatten(), Some(vec!["stop".to_string()]));
            assert!(call_tool("queue", &json!({ "commander": ["armsolar", "stop"] }), &shared).is_err());
        }
        assert!(call_tool("queue", &json!({ "commander": "[\"assist\"]" }), &shared).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "commander": "assist" }), &shared).unwrap().contains("1 steps"));
    }

    #[test]
    fn the_lane_tool_sets_footwork_by_group() {
        use super::super::shared::Footwork;
        let shared = Arc::new(Shared::default());
        assert!(call_tool("lane", &json!({ "group_A": "raw", "all": ["fan", "form"] }), &shared).is_ok());
        let lane = shared.lane.lock().unwrap().clone();
        assert_eq!(lane["group_A"], Footwork::raw());
        assert_eq!(lane["all"], Footwork { flee: false, fan: true, kite: false, form: true, march: false, follow: false, rove: false });
        assert!(call_tool("lane", &json!({ "group_A": ["dance"] }), &shared).is_err());
        assert!(call_tool("lane", &json!({ "raiders": "raw" }), &shared).is_err());
        assert!(call_tool("lane", &json!({}), &shared).is_err());
        // "on" for a group is said explicitly: it takes a hands' scout off roving too.
        assert!(call_tool("lane", &json!({ "group_A": "on" }), &shared).is_ok());
        assert_eq!(shared.lane.lock().unwrap()["group_A"], Footwork::default());
        // Roving is per group, never for all.
        assert!(call_tool("lane", &json!({ "group_B": "rove" }), &shared).unwrap().contains("roving"));
        assert_eq!(shared.lane.lock().unwrap()["group_B"], Footwork::rove());
        assert!(call_tool("lane", &json!({ "all": "rove" }), &shared).is_err());
    }

    #[test]
    fn marks_are_named_places_by_coordinates_or_cell() {
        let shared = Arc::new(Shared::default());
        *shared.map.lock().unwrap() = json!({ "width": 8000.0, "height": 8000.0 });
        assert!(call_tool("mark", &json!({ "south_gate": [3600, 5400], "far_east": "H4" }), &shared).is_ok());
        let marks = shared.marks.lock().unwrap().clone();
        assert_eq!(marks["south_gate"], (3600.0, 5400.0));
        assert_eq!(marks["far_east"], (7500.0, 3500.0));
        assert!(call_tool("mark", &json!({ "spot_3": [1.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "Gate": [1.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "x": [9000.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "x": "Z9" }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "south_gate": null }), &shared).is_ok());
        assert!(!shared.marks.lock().unwrap().contains_key("south_gate"));
    }

    #[test]
    fn produce_whitelists_a_lab_or_all() {
        let shared = Arc::new(Shared::default());
        shared.hands.lock().unwrap().entry(0).or_default().picture = json!({ "actors": { "lab_7": {} } });
        // A factory the picture does not name is refused, with the names it does (models-medium-gpt56-terra: "lab_1").
        assert!(call_tool("produce", &json!({ "lab_1": ["armpw"] }), &shared).unwrap_err().contains("lab_7"));
        assert!(call_tool("produce", &json!({ "all": ["armpw", "armham"], "lab_7": [] }), &shared).is_ok());
        let allowed = shared.allowed.lock().unwrap().clone();
        assert_eq!(allowed["all"].units, vec!["armpw".to_string(), "armham".to_string()]);
        assert!(allowed["lab_7"].units.is_empty());
        assert!(call_tool("produce", &json!({ "all": ["armck:1", "armpw"] }), &shared).is_ok());
        assert!(call_tool("produce", &json!({ "all": ["armck:x"] }), &shared).is_err());
        assert_eq!(crate::brain::pianist::allowance("armck:2"), ("armck", Some(2)));
        assert_eq!(crate::brain::pianist::allowance("armpw"), ("armpw", None));
        assert!(call_tool("produce", &json!({ "group_A": ["armpw"] }), &shared).is_err());
        assert!(call_tool("produce", &json!({ "commander": ["armfus:1"], "all_builders": ["armllt", "armsolar"] }), &shared).is_ok());
        assert_eq!(shared.allowed.lock().unwrap()["all_builders"].units, vec!["armllt".to_string(), "armsolar".to_string()]);
        let first = shared.allowed.lock().unwrap()["all_builders"].call;
        assert!(call_tool("produce", &json!({ "all_builders": ["armllt", "armsolar"] }), &shared).is_ok());
        assert!(shared.allowed.lock().unwrap()["all_builders"].call > first, "the same list again is a fresh allowance");
        assert!(call_tool("units", &json!({ "names": ["armpw"] }), &shared).is_ok());
        assert!(call_tool("units", &json!({}), &shared).is_err());
        // A list as a string, JSON or comma-separated, is read as the list (models-medium-sonnet5: eight refusals).
        assert!(call_tool("produce", &json!({ "all": "armpw" }), &shared).is_ok());
        assert_eq!(shared.allowed.lock().unwrap()["all"].units, vec!["armpw".to_string()]);
        assert!(call_tool("produce", &json!({ "all": "[\"armpw\", \"armham:2\"]" }), &shared).is_ok());
        assert_eq!(shared.allowed.lock().unwrap()["all"].units, vec!["armpw".to_string(), "armham:2".to_string()]);
        assert!(call_tool("produce", &json!({ "lab_7": null }), &shared).is_ok());
        assert!(!shared.allowed.lock().unwrap().contains_key("lab_7"));
    }
}

/// The `plan` and `search` tools (docs/design/2026-09-22-plan-search.md, decisions 5 and 6): the player's queue
/// vocabulary in and out of the simulator's plans.
mod planning {
    use std::collections::BTreeMap;

    use buildorder::anneal::{anneal_within, Goal, Objective, Palette, Search};
    use buildorder::plan::{Item, Plan, Step};
    use buildorder::sim::{simulate, Outcome};
    use serde_json::{Value, json};

    use super::PlanContext;

    const DEFAULT_MINUTES: f64 = 8.0;
    const MAX_MINUTES: f64 = 20.0;
    pub(super) const DEFAULT_SECONDS: f64 = 10.0;
    /// The cap in lockstep, where the game holds during a turn; and in realtime, where it does not.
    pub(super) const MAX_SECONDS: f64 = 20.0;
    pub(super) const REALTIME_MAX_SECONDS: f64 = 3.0;
    const SEARCH_THREADS: usize = 4;
    const TARGET_COUNT: usize = 6;
    pub(super) const OBJECTIVES: &str = "income, army, mix, or target UNIT[:COUNT] [by M:SS], more goals after commas, income:N by M:SS among them";

    /// The `target` objective's goals against the found order, one line each: how many finished, and when the
    /// asked-for count (or the first) did against the deadline. None for the other objectives.
    /// "the first", "the 2nd", "the 3rd", "the 4th" ...
    fn nth(n: Option<usize>) -> String {
        match n {
            None | Some(1) => "the first".to_string(),
            Some(n) => format!("the {n}{}", match (n % 10, n % 100) { (1, x) if x != 11 => "st", (2, x) if x != 12 => "nd", (3, x) if x != 13 => "rd", _ => "th" }),
        }
    }

    fn goals_words(ctx: &PlanContext, goals: &[Goal], income: &[(f64, f64)], outcome: &Outcome) -> Option<String> {
        if goals.is_empty() && income.is_empty() {
            return None;
        }
        let units = &ctx.game.units;
        let mut lines: Vec<String> = income
            .iter()
            .map(|&(rate, by)| {
                let got = outcome.metal_income_at(by);
                format!("income: {rate:.0} a second asked for by {}, {got:.1} in the order{}", mmss(by), if got >= rate { "" } else { " (short)" })
            })
            .collect();
        lines.extend(goals
            .iter()
            .map(|g| {
                let name = &units.list[g.unit].name;
                let finished: Vec<f64> = outcome.finished.iter().filter(|f| f.unit == g.unit).map(|f| f.t).collect();
                let asked = g.count.map_or(String::new(), |n| format!("{n} asked, "));
                let done_at = g.count.map_or(finished.first().copied(), |n| finished.get(n - 1).copied());
                let which = nth(g.count);
                let when = match (done_at, g.by) {
                    (Some(t), Some(by)) if t <= by => format!("{which} finished at {} ({} asked for)", mmss(t), mmss(by)),
                    (Some(t), Some(by)) => format!("{which} finished at {}, {} late of {}", mmss(t), mmss(t - by), mmss(by)),
                    (Some(t), None) => format!("{which} finished at {}", mmss(t)),
                    (None, Some(by)) => format!("{which} not finished by the horizon ({} asked for)", mmss(by)),
                    (None, None) => "none finished by the horizon".to_string(),
                };
                format!("{name}: {asked}{} finished by the horizon; {when}", finished.len())
            }));
        Some(format!("\n\ngoals: {}", lines.join("; ")))
    }

    fn mmss(seconds: f64) -> String {
        let s = seconds.max(0.0).round() as i64;
        format!("{}:{:02}", s / 60, s % 60)
    }

    pub(super) fn run(name: &str, arguments: &Value, seconds: f64, ctx: &PlanContext, marks: &BTreeMap<String, (f32, f32)>) -> Result<String, String> {
        let units = &ctx.game.units;
        let minutes = arguments["minutes"].as_f64().unwrap_or(DEFAULT_MINUTES).clamp(1.0, MAX_MINUTES);
        let horizon = minutes * 60.0;
        let given = match arguments.get("queues") {
            Some(q) if q.is_object() => Some(plan_from_queues(ctx, q, marks)?),
            Some(Value::Null) | None => None,
            Some(_) => return Err("queues is an object: actor name to a list of steps".into()),
        };
        match name {
            "plan" => {
                let plan = given.ok_or("plan takes {\"queues\": {...}}")?;
                let outcome = simulate(units, &ctx.scenario, &ctx.state, &plan, horizon);
                Ok(report(ctx, &plan, &outcome, None))
            }
            _ => {
                let text = arguments["objective"].as_str().ok_or("search takes an objective in words")?.trim().to_string();
                let objective = if text.starts_with("target") { Objective::parse_target(&text, units)? } else { Objective::parse(&text).ok_or_else(|| format!("{text}: not an objective ({OBJECTIVES})"))? };
                let palette = Palette::roster(units, ctx.game.commander, ctx.turret, ctx.water);
                let (factories, constructors) = (ctx.standing_factories + 2, ctx.standing_constructors + 3);
                let start = given.or_else(|| match &objective {
                    Objective::Target { goals, .. } => {
                        let goals: Vec<(usize, usize)> = goals.iter().map(|g| (g.unit, g.count.unwrap_or(TARGET_COUNT))).collect();
                        Some(palette.chain_seed(units, ctx.game.commander, &goals, factories, constructors, ctx.wind))
                    }
                    _ => None,
                });
                let search = Search { objective, horizon, iterations: 0, seed: ctx.frame as u64 + 1, factories, constructors, hot: 0.02, start };
                let found = anneal_within(units, &ctx.scenario, &ctx.state, &palette, &search, std::time::Duration::from_secs_f64(seconds), SEARCH_THREADS);
                let mut out = report(ctx, &found.plan, &found.outcome, Some(found.score));
                if let Some(goals) = goals_words(ctx, search.objective.goals(), search.objective.income_goals(), &found.outcome) {
                    out.push_str(&goals);
                }
                Ok(out)
            }
        }
    }

    /// The name the player knows for a plan queue.
    fn queue_name(ctx: &PlanContext, queue: usize, factories: usize) -> String {
        if let Some((name, _)) = ctx.actors.iter().find(|(_, q)| *q == queue) {
            return name.clone();
        }
        if queue == 0 {
            "commander".into()
        } else if queue <= factories {
            format!("next_factory_{}", queue - ctx.standing_factories)
        } else {
            format!("next_constructor_{}", queue - factories - ctx.standing_constructors)
        }
    }

    fn plan_from_queues(ctx: &PlanContext, queues: &Value, marks: &BTreeMap<String, (f32, f32)>) -> Result<Plan, String> {
        let units = &ctx.game.units;
        let object = queues.as_object().ok_or("queues is an object")?;
        let next_of = |name: &str, prefix: &str| name.strip_prefix(prefix).and_then(|n| n.parse::<usize>().ok()).filter(|n| *n >= 1);
        let extra_factories = object.keys().filter_map(|k| next_of(k, "next_factory_")).max().unwrap_or(0);
        let extra_constructors = object.keys().filter_map(|k| next_of(k, "next_constructor_")).max().unwrap_or(0);
        let factories = ctx.standing_factories + extra_factories;
        let mut plan = Plan::empty(factories, ctx.standing_constructors + extra_constructors);
        let basic = units.extractor(ctx.game.commander).ok_or("the commander builds no extractor")?;
        for (name, list) in object {
            let queue = if let Some((_, q)) = ctx.actors.iter().find(|(n, _)| n == name) {
                *q
            } else if name == "commander" {
                0
            } else if let Some(n) = next_of(name, "next_factory_") {
                ctx.standing_factories + n
            } else if let Some(n) = next_of(name, "next_constructor_") {
                factories + ctx.standing_constructors + n
            } else {
                return Err(format!("{name}: not a builder standing now ({}) nor next_factory_N / next_constructor_N", ctx.actors.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ")));
            };
            let items = match list {
                Value::Array(items) => items.iter().map(|v| v.as_str().map(|s| s.trim().to_string()).ok_or_else(|| format!("{name}: steps are strings"))).collect::<Result<Vec<_>, _>>()?,
                Value::String(s) => s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                _ => return Err(format!("{name}: a list of steps")),
            };
            let mut steps = Vec::new();
            for text in items {
                let mut words = text.split_whitespace();
                let kind = words.next().unwrap_or_default();
                let place = words.next();
                let site = match place {
                    None => None,
                    Some(p) => Some(if let Some(n) = p.strip_prefix("spot_").and_then(|n| n.parse::<usize>().ok()) {
                        *ctx.spots.get(n).ok_or_else(|| format!("{name}: {p} is not a spot"))?
                    } else if let Some((x, z)) = marks.get(p) {
                        (*x as f64, *z as f64)
                    } else {
                        return Err(format!("{name}: {p} is neither a spot nor a marked place"));
                    }),
                };
                let item = match kind {
                    "assist" => Item::Assist,
                    "extractor" => Item::Build(basic),
                    other => Item::Build(units.index(other).ok_or_else(|| format!("{name}: {other} is not a unit of this game"))?),
                };
                steps.push(Step { item, site });
            }
            *plan.queue_mut(queue) = steps;
        }
        Ok(plan)
    }

    fn step_words(ctx: &PlanContext, step: &Step) -> String {
        let units = &ctx.game.units;
        let name = match step.item {
            Item::Assist => return "assist".into(),
            Item::Build(u) if units.list[u].extracts_metal > 0.0 && units.list[u].extracts_metal <= units.basic_extraction() * 1.5 => "extractor".to_string(),
            Item::Build(u) => units.list[u].name.clone(),
        };
        match step.site {
            None => name,
            Some((x, z)) => match ctx.spots.iter().enumerate().find(|(_, s)| ((s.0 - x).powi(2) + (s.1 - z).powi(2)).sqrt() < ctx.spot_radius) {
                Some((n, _)) => format!("{name} spot_{n}"),
                None => format!("{name} @{x:.0},{z:.0}"),
            },
        }
    }

    fn report(ctx: &PlanContext, plan: &Plan, outcome: &Outcome, score: Option<f64>) -> String {
        let units = &ctx.game.units;
        let factories = plan.factories.len();
        let mut queues: BTreeMap<String, Value> = BTreeMap::new();
        let mut passed: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for q in 0..plan.queue_count() {
            let name = queue_name(ctx, q, factories);
            let given: Vec<String> = plan.queue(q).iter().map(|s| step_words(ctx, s)).collect();
            let done: Vec<String> = outcome.effective.get(q).map(|e| e.iter().map(|s| step_words(ctx, s)).collect()).unwrap_or_default();
            if given.is_empty() && done.is_empty() {
                continue;
            }
            let mut left = done.clone();
            let mut missing = Vec::new();
            for step in &given {
                match left.iter().position(|d| d == step) {
                    Some(i) => {
                        left.remove(i);
                    }
                    None => missing.push(step.clone()),
                }
            }
            if !missing.is_empty() {
                passed.insert(name.clone(), missing);
            }
            queues.insert(name, json!(if score.is_some() { done } else { given }));
        }
        let t0 = ctx.state.t0;
        let mut curves: Vec<String> = Vec::new();
        let mut minute = 1.0;
        while t0 + minute * 60.0 <= outcome.last().t + 0.5 {
            let t = t0 + minute * 60.0;
            if let Some(s) = outcome.samples.iter().find(|s| (s.t - t).abs() < 0.51) {
                curves.push(format!("{}: extractors {}, metal +{:.1}/s, energy +{:.0}/s, stalled {:.0}%, army {:.0} metal ({} units), build power {:.0}, factories {}, constructors {}, nanos {}", mmss(t), s.extractors, s.metal_income, s.energy_income, (1.0 - s.stall) * 100.0, s.army_value, s.army_count, s.build_power, s.factories, s.constructors, s.nanos));
            }
            minute += 1.0;
        }
        let mut firsts: BTreeMap<String, f64> = BTreeMap::new();
        for f in &outcome.finished {
            firsts.entry(units.list[f.unit].name.clone()).or_insert(f.t);
        }
        let mut firsts: Vec<(f64, String)> = firsts.into_iter().map(|(n, t)| (t, n)).collect();
        firsts.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut out = json!({
            "from": format!("the game at {} (the simulator's picture is refreshed every ten seconds)", mmss(t0)),
            "queues": queues,
            "curves": curves,
            "first_finished": firsts.iter().map(|(t, n)| format!("{n} at {}", mmss(*t))).collect::<Vec<_>>(),
            "note": "the simulator knows the economy and building, not the enemy, losses or terrain; about ten percent optimistic on quiet openings",
        });
        if !passed.is_empty() {
            out["passed_over"] = json!(passed.iter().map(|(n, s)| format!("{n}: {} (this builder cannot build it, no free spot, or the list was not reached in time)", s.join(", "))).collect::<Vec<_>>());
        }
        if let Some(score) = score {
            out["score"] = json!(format!("{score:.0}"));
            out["queues_note"] = json!("the order as the builders would carry it out; give it to `queue` (builders) and `produce` (factories) yourself if you want it");
        }
        Ok::<String, String>(out.to_string()).unwrap_or_default()
    }
}
