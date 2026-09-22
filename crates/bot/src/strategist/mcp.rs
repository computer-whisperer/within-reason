//! Minimal MCP server over streamable HTTP: the five methods Claude Code was observed to use
//! (`docs/harness/claude-p.md`), answered with plain JSON.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bot_protocol::Vec3;
use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server};

use super::Mode;
use super::shared::{Focus, OrderKind, OutpostTurrets, PlanContext, Post, Shared, Stance, Timed};
use super::transcript::Transcript;

const DEFAULT_TTL_SECONDS: i64 = 120;
const MAX_TTL_SECONDS: i64 = 600;
/// The player's packet at most: Jev reads it every second beside a picture of about 4k tokens, within 64k.
const INSTRUCTIONS_LIMIT: usize = 8000;

pub struct McpServer {
    pub port: u16,
    stop: Arc<AtomicBool>,
}

impl McpServer {
    /// Serves on a free localhost port until dropped.
    pub fn start(shared: Arc<Shared>, transcript: Arc<Transcript>, mode: Mode) -> std::io::Result<Self> {
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
                let reply = serde_json::from_str::<Value>(&body).ok().and_then(|call| handle(&call, &shared, &transcript, mode));
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

fn handle(call: &Value, shared: &Shared, transcript: &Transcript, mode: Mode) -> Option<Value> {
    let id = call.get("id")?.clone();
    let method = call["method"].as_str().unwrap_or_default();
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": call["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "within-reason", "version": env!("CARGO_PKG_VERSION") },
        })),
        "tools/list" => Ok(json!({ "tools": tool_list(mode) })),
        "tools/call" => {
            let name = call["params"]["name"].as_str().unwrap_or_default();
            let arguments = &call["params"]["arguments"];
            let outcome = if shared.turn_over.load(Ordering::Relaxed) {
                Err("Your turn is over and the game is running. Stop now: call nothing more and write nothing more. You will be woken with a new report.".to_string())
            } else if name == "orders" {
                orders(arguments, shared, mode)
            } else {
                call_tool(name, arguments, shared, mode)
            };
            // Full results, briefings included: they are what the strategist decided on, and the
            // labelled state for evaluating faster models against its decisions.
            let recorded = outcome.as_ref().map_or_else(
                |problem| Value::String(problem.clone()),
                |text| serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.clone())),
            );
            transcript.record(json!({ "kind": "tool_call", "tool": name, "arguments": arguments, "result": recorded }));
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

/// The tools a mode offers. The commander's levers steer the heuristics; the player has none of them: its lever is
/// `instruct`, and its `situation` is the picture its hands read.
fn tool_list(mode: Mode) -> Value {
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
        "description": "Ends your turn: the game resumes the moment this is called, so call it last and write nothing after it. Sets when you are next woken; the settings hold until you change them. The game is paused during your turn and runs fast between turns, so a long quiet wait costs nothing and a raid still wakes you at once. You are always woken for a base attack, the commander under fire, or a wiped-out wave.",
        "inputSchema": { "type": "object", "additionalProperties": false, "properties": {
            "max_seconds": { "type": "integer", "minimum": 5, "maximum": 180, "description": "Game seconds after which you are woken whatever happens (default 30)." },
            "enemy_near_extractor": { "type": "boolean", "description": "Enemies appear within 600 of an extractor that had none near." },
            "squad_engaged": { "type": "boolean", "description": engaged },
            "extractor_lost": { "type": "boolean" },
            "pool_reaches": { "type": "object", "additionalProperties": { "type": "integer", "minimum": 1 }, "description": pool },
            "chat": { "type": "boolean", "description": "A person in the game says something (default on)." } } } });
    let orders = |tools: &[&str], what: &str| json!({ "name": "orders",
        "description": format!("Your whole turn in one call, and it ENDS the turn: {what}, carried out in the order listed, then the game resumes. Each entry names one of the other tools and its arguments, exactly as you would call it alone. Include a `wait` entry to change when you are next woken; without one the wake settings in force stand. Call nothing and write nothing after it."),
        "inputSchema": { "type": "object", "additionalProperties": false, "required": ["calls"], "properties": {
            "calls": { "type": "array", "minItems": 1, "items": { "type": "object", "additionalProperties": false, "required": ["tool"], "properties": {
                "tool": { "type": "string", "enum": tools },
                "arguments": { "type": "object", "description": "That tool's arguments, e.g. {\"text\": \"...\"} for note." } } } } } } });
    match mode {
        Mode::Player => json!([
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
              "description": "Search build orders in the simulator from the game as it stands now, without ordering anything: an objective in words (`income`, `army`, `mix` (army plus two minutes of income), or `target UNIT by M:SS`, e.g. `target armbull by 9:00`), a horizon in minutes and a time budget in seconds (default 10, at most 20; the game holds while you wait). The search may use the whole roster: every generator, storage, converter, extractor, factory, nano and unit the faction reaches. Returns the best order found in the `plan` tool's words with its curves and score. Give `queues` to start the search from your own order.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["objective"], "properties": { "objective": { "type": "string" }, "minutes": { "type": "number" }, "seconds": { "type": "number" }, "queues": { "type": "object" } } } },
            { "name": "instruct",
              "description": format!("Your standing instructions to your hands: the whole packet, replacing the last one. Jev reads it every second beside the picture and picks each actor's next action from a menu, so write it as standing orders in plain words: the build order per builder as a sequence, what the lab makes and when that changes, where each group stands, when it engages, scouts and attacks, what to do about raids. Name places as the picture does (home, spot_N, passage_N, and any place you marked with `mark`; a spot or passage you name here is always on your hands' menu, however far) and groups as group_A, group_B. No arithmetic for the hands to do: say \"when we have about ten soldiers\", not a formula. At most {INSTRUCTIONS_LIMIT} characters."),
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string" } } } },
            { "name": "queue",
              "description": "A builder's build list, done exactly and in order by the bot itself without asking your hands: an object of builder name (commander, constructor_N) to a list of steps, or null (or an empty list) to cancel its list. Steps: \"extractor spot_N\" (or \"extractor\" for the nearest free spot), any building by its internal name as the roster lists it (\"armsolar\", \"armvp\", \"armnanotc\", \"armfus\"), one that stands at a place with the place after it (\"armllt spot_3\", \"armrad home\", \"armmoho spot_3\" over our extractor there), \"assist\" (help the nearest factory, standing or being built: the last step of an opening). Each step is ordered when the one before is 60% built, so nothing idles; a step that cannot be done (the spot taken, a place unknown, a building this builder cannot make) is skipped and said in the hands' report. While a list runs the builder is off your hands' menu unless an enemy is on it; your instructions take over when the list is done. This is how an opening is made to happen as written: the hands do not follow a sequence (comet-1, comet-2: 'three solars, then the plant' got extractors and the plant at 1:45).",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" } ] }, "description": "Builder name to steps, or null." } },
            { "name": "lane",
              "description": "Which footwork rules your hands' code applies to a group's soldiers between your hands' orders, per group name or for \"all\": \"raw\" (none: the group's orders reach the engine exactly as given), \"on\" (all of them, the default), or a list of the rules to keep. The rules: flee (a soldier steps out of the reach of a turret or a fight it was not sent against, or one it would die in), fan (spreads out under a commander's D-gun), focus (soldiers standing together shoot one target at a time), kite (a soldier that outranges its enemy steps back while reloading), march (an advancing group waits for its stragglers so it arrives together), follow (an engaging group is re-sent after its party as it moves). A setting stands until you change it; the group's picture entry shows it when it is not the default.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "string", "enum": ["raw", "on"] }, { "type": "array", "items": { "type": "string", "enum": ["flee", "fan", "focus", "kite", "march", "follow"] } } ] }, "description": "Group name (group_A) or \"all\" to its setting." } },
            { "name": "mark",
              "description": "Name a place of your own for your hands: an object of name to [x, z] map coordinates or a grid cell (\"E7\": its centre), or null to forget it. A marked place joins the picture's places at once, so instructions can send groups and builders there (\"group_B: advance to south_gate\"), and its entry says whose ground it is and what enemy is near. Names are lower-case words with underscores; home, spot_N, passage_N and group_N are taken. Any spot or passage you name in the packet is on your hands' menu already, however far; mark is for places that are not spots.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, { "type": "string" }, { "type": "null" } ] }, "description": "Place name to [x, z], a grid cell, or null." } },
            { "name": "produce",
              "description": "What each factory or builder may build: an object of actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" (every commander and constructor) or \"all\" (everyone) to a list of unit names (as the roster writes them: armpw, armham, armck, armfus), or null to lift the restriction. A name with a count after a colon (armck:1) is allowed that many more times from now and then drops off the list by itself: the way to say 'one constructor, then raiders' to hands that cannot count. A lab with a list is offered only those units and nothing else, every time it is asked; your instructions still say which of them and when. A builder without a list is offered the usual buildings (generators, factories, light and heavy turrets, radar, storage, the tier-2 lab and extractor, fusion); a list replaces that, so a fusion reactor, an aircraft plant or a jammer from a constructor is asked for here. Use it when the packet's words are not getting the mix you want. A list naming nothing the actor can build leaves it unrestricted; the actor's entry in the picture shows its list. Your policy is not bound by lists: it may order anything a builder can build.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" } ] }, "description": "Actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\" to unit names, or null." } },
            { "name": "policy",
              "description": "Your Lua policy (with `bot --policy`): the script your hands run once a game second over the picture, in place of, or beside, Jev's reading of your packet. {\"set\": script} replaces the whole policy and starts a fresh Lua state; {\"amend\": chunk} runs the chunk in the living state, so each top-level function or table it defines replaces the one of that name in place and the rest stands (globals persist); {} returns the policy in force. A parse error is answered at once; runtime errors and the orders given come in your report. The script defines decide(S) and returns { [actor] = { [\"do\"] = option, where = place, whom = party, how_many = \"2\"|\"4\"|\"8\"|\"half\", where_scout = place } }; only an option in that actor's S.actors[name].options can be ordered; an actor left out keeps its course.",
              "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "set": { "type": "string" }, "amend": { "type": "string" } } } },
            { "name": "say",
              "description": "Say something in the game's chat, to everyone playing. Short lines. The report shows what people say to you; when an experienced player offers advice or asks what you are doing, answer, and ask them what they would do: their feedback is what this project learns from.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string", "maxLength": 240 } } } },
            orders(&["instruct", "queue", "policy", "lane", "mark", "produce", "say", "note", "wait"], "your instructions or policy, build lists, footwork settings, marked places, what labs may build, a chat line, a note and when to be woken"),
            wait("A group of ours starts fighting an enemy party.", "Woken when this many soldiers of each named unit type are alive, e.g. {\"armham\": 6}. {} clears it."),
            note,
        ]),
        Mode::Strategist | Mode::Commander => json!([
            overview, map,
            { "name": "set_directives",
              "description": "Set standing orders for the bot's heuristics. Give only the fields you want to change. Every directive expires after ttl_seconds of game time (default 120, max 600) and the heuristic's own default takes over, so renew what should persist. Pass null for a field to clear it now.",
              "inputSchema": { "type": "object", "additionalProperties": false, "properties": {
                  "army_stance": { "enum": ["defend", "gather", "attack", null],
                      "description": "defend: army stays home. gather: keep massing, launch nothing. attack: commit the home group now regardless of size." },
                  "attack_target": { "type": ["object", "null"], "properties": { "x": { "type": "number" }, "z": { "type": "number" } },
                      "required": ["x", "z"], "description": "Where attackers go, in map coordinates." },
                  "wave_size": { "type": ["integer", "null"], "minimum": 1, "maximum": 200,
                      "description": "Home-group size at which the bot launches a wave on its own." },
                  "army_station": { "type": ["object", "null"], "properties": { "x": { "type": "number" }, "z": { "type": "number" } },
                      "required": ["x", "z"], "description": "Where the home group waits and gathers. By default it stands just ahead of our most exposed extractors; it still turns on raiders near any of our extractors and on intruders at the base." },
                  "min_constructors": { "type": ["integer", "null"], "minimum": 1, "maximum": 10,
                      "description": "Factories keep at least this many constructors alive (the bot's own floor is 3)." },
                  "min_converters": { "type": ["integer", "null"], "minimum": 0, "maximum": 40,
                      "description": "Constructors build energy-to-metal converters up to this count before expanding further, energy permitting." },
                  "max_converters": { "type": ["integer", "null"], "minimum": 0, "maximum": 40,
                      "description": "No more converters than this, whatever energy is banked (each takes 70 energy a second to run and costs 1150 to build; the bot's own rule builds one only when energy income exceeds usage by that much and stops at 40). 0 stops them." },
                  "base_turrets": { "type": ["integer", "null"], "minimum": 0, "maximum": 6,
                      "description": "How many light turrets (85 metal each) the bot puts up on its own 450 forward of home: unset it builds 2 after the lab and up to 6 when constructors are idle; 0 stops them. Turrets are for Ticks: one at an extractor repels them where no army stands; Hammers and Pawns are the army's to counter." },
                  "outpost_turrets": { "type": ["string", "array", "null"], "items": { "type": "integer", "minimum": 0 },
                      "description": "Which of our extractors get a light turret of the bot's own accord: \"all\" (unset: one at every extractor more than 500 from home), \"none\" (only where request_turret asks), or a list of metal spot numbers n from the map (a turret at each of those extractors as they stand, none elsewhere). Give the spots your squads do not cover." },
                  "expansion_radius": { "type": ["integer", "null"], "minimum": 500, "maximum": 20000,
                      "description": "Constructors build extractors only on metal spots within this walking distance of home (see walk_from_home in the map). Use it to stop expansion into places you cannot defend." },
                  "tier2": { "type": ["boolean", "null"],
                      "description": "The advanced bot lab (2600 metal, then advanced constructors that upgrade our extractors to four times the yield, and tier-2 units). Unset, the bot starts it when metal income reaches 22 and energy income 450 with nothing dying at home. true starts it now; false holds it back." },
                  "resurrect": { "type": ["boolean", "null"],
                      "description": "Resurrection bots (the bot builds one per 600 metal of wrecks lying on held ground, at most 6) raise wrecked soldiers worth 100 metal or more when stored energy is above half and 100 metal is banked, and take everything else apart for its metal. false: they raise nothing and reclaim everything." },
                  "commander_station": { "type": ["object", "null"], "properties": { "x": { "type": "number" }, "z": { "type": "number" } },
                      "required": ["x", "z"], "description": "The commander walks here and builds only near here (it is a strong builder and fighter, and the game is lost if it dies). Without this it builds within 24 seconds of its own walking from home." },
                  "economy_focus": { "enum": ["expand", "energy", "production", "defence", null],
                      "description": "What constructors prefer once the opening is done." },
                  "pressure": { "type": ["boolean", "null"],
                      "description": "The early raider pressure: from the first Pawn the bot sends raiders at the opponent's extractors and base, priced by its fight simulator, retreating from the commander and turrets and harassing elsewhere. false keeps them home (they join the home group); unset or true lets it run." },
                  "scout_at": { "type": ["object", "null"], "properties": { "x": { "type": "number" }, "z": { "type": "number" } },
                      "required": ["x", "z"], "description": "Send the next scout (one raider, a route of metal spots) round this point first. The bot scouts by itself: the enemy base's spots when stale, the rest of its box, then the map." },
                  "ttl_seconds": { "type": "integer", "minimum": 10, "maximum": MAX_TTL_SECONDS } } } },
            { "name": "situation",
              "description": "The field picture: unassigned soldiers by type, your squads (composition, health, where, post, whether engaged), every extractor with enemies near it and whether a turret covers it, turrets, what the factories can build with metal cost, the production mix in force. You are sent this at the start of every turn; call it only to look again mid-turn.",
              "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false } },
            { "name": "squad",
              "description": "Create or change a squad. Soldiers in a squad are yours; all others follow the bot's heuristics (home group, attack waves). take draws that many more of each unit type from the unassigned soldiers, nearest to `near` (else to the post); units not yet built are added as they appear. post is a standing defensive position: the squad stands there, engages any enemy that comes within radius of it, and returns. order is a one-off move or fight (attack-move) to a position and cancels the post. release hands the squad back to the heuristics.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["name"], "properties": {
                  "name": { "type": "string" },
                  "take": { "type": "object", "additionalProperties": { "type": "integer", "minimum": 0, "maximum": 50 },
                      "description": "Unit name to how many more to draw, e.g. {\"armpw\": 3, \"armham\": 2}." },
                  "near": { "type": "object", "properties": { "x": { "type": "number" }, "z": { "type": "number" } }, "required": ["x", "z"] },
                  "post": { "type": "object", "properties": { "x": { "type": "number" }, "z": { "type": "number" }, "radius": { "type": "number", "minimum": 100, "maximum": 1500 } },
                      "required": ["x", "z", "radius"] },
                  "order": { "type": "object", "properties": { "kind": { "enum": ["move", "fight"] }, "x": { "type": "number" }, "z": { "type": "number" } },
                      "required": ["kind", "x", "z"] },
                  "release": { "type": "boolean" } } } },
            { "name": "set_production",
              "description": "The unit mix the factories build, as unit name to weight (names from `buildable` in the situation). Factories build whichever type is furthest below its share of what is alive. The bot keeps its own floor of constructors. An empty object returns production to the bot's default batch.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["weights"], "properties": {
                  "weights": { "type": "object", "additionalProperties": { "type": "integer", "minimum": 0, "maximum": 100 } } } } },
            { "name": "request_turret",
              "description": "Ask for a light defence turret at a position; the next free constructor builds it near there.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["x", "z"],
                  "properties": { "x": { "type": "number" }, "z": { "type": "number" } } } },
            { "name": "expansion",
              "description": "Which metal spots the constructors take. Spots are numbered as in the map's metal_spots list (`n`). `take_first`: spots taken before any other, in this order, wherever they lie and even if they were raided before (this is also how you order a lost extractor rebuilt, or leave it lost by not listing it). `leave_alone`: spots never taken, e.g. ones you cannot hold. Other spots follow the bot's rule (nearest first, within expansion_radius, skipping recently raided ones without cover). Each call replaces the whole plan; {} clears it.",
              "inputSchema": { "type": "object", "additionalProperties": false, "properties": {
                  "take_first": { "type": "array", "items": { "type": "integer", "minimum": 0 } },
                  "leave_alone": { "type": "array", "items": { "type": "integer", "minimum": 0 } } } } },
            orders(&["squad", "set_directives", "set_production", "request_turret", "expansion", "note", "wait"], "every order you want to give"),
            wait("A posted squad starts fighting.", "Woken when this many of each named unit type stand unassigned, e.g. {\"armham\": 4}. {} clears it."),
            note,
        ]),
    }
}

/// What `orders` may batch in a mode.
fn batchable(mode: Mode) -> &'static [&'static str] {
    match mode {
        Mode::Player => &["instruct", "queue", "policy", "lane", "mark", "produce", "say", "note", "wait"],
        Mode::Strategist | Mode::Commander => &["squad", "set_directives", "set_production", "request_turret", "expansion", "note", "wait"],
    }
}

/// The `orders` tool: several tool calls in one request. `wait` goes last wherever it was listed, since it ends the turn.
fn orders(arguments: &Value, shared: &Shared, mode: Mode) -> Result<String, String> {
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
        if !batchable(mode).contains(&tool) {
            lines.push(format!("{tool}: not a tool that can be batched"));
            continue;
        }
        let mut beside = call.clone();
        if let Some(fields) = beside.as_object_mut() {
            fields.remove("tool");
            fields.remove("arguments");
        }
        let arguments = call.get("arguments").filter(|a| a.is_object()).unwrap_or(if beside.as_object().is_some_and(|f| !f.is_empty()) { &beside } else { &empty });
        match call_tool(tool, arguments, shared, mode) {
            Ok(text) => lines.push(format!("{tool}: {text}")),
            Err(problem) => lines.push(format!("{tool}: REFUSED: {problem}")),
        }
    }
    Ok(lines.join("\n"))
}

fn call_tool(name: &str, arguments: &Value, shared: &Shared, mode: Mode) -> Result<String, String> {
    if !tool_list(mode).as_array().is_some_and(|tools| tools.iter().any(|t| t["name"] == name)) {
        return Err(format!("{name} is not a tool in this mode"));
    }
    match name {
        "overview" => serde_json::to_string(&shared.briefing()).map_err(|e| e.to_string()),
        "map" => {
            let mut map = shared.map.lock().unwrap().clone();
            map["ground"] = serde_json::json!({
                "legend": "whose ground is whose right now, one character per 256 elmos, north up: + held by us, ? contested, - theirs",
                "rows": *shared.ground_sketch.lock().unwrap(),
            });
            Ok(map.to_string())
        }
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
        "situation" if mode == Mode::Player => Ok(shared.hands.lock().unwrap().picture.to_string()),
        "situation" => serde_json::to_string(&shared.field()).map_err(|e| e.to_string()),
        "instruct" => {
            let text = arguments["text"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("instruct needs the packet under \"text\"")?;
            if text.chars().count() > INSTRUCTIONS_LIMIT {
                return Err(format!("the packet is {} characters; at most {INSTRUCTIONS_LIMIT}. Cut it: your hands read it every second", text.chars().count()));
            }
            *shared.instructions.lock().unwrap() = text.to_string();
            Ok(format!("instructions replaced ({} characters); your hands read them from their next look, once your turn ends", text.chars().count()))
        }
        "policy" => {
            use super::shared::PolicyChange;
            let set = arguments["set"].as_str().map(str::trim).filter(|t| !t.is_empty());
            let amend = arguments["amend"].as_str().map(str::trim).filter(|t| !t.is_empty());
            match (set, amend) {
                (Some(_), Some(_)) => Err("policy takes \"set\" or \"amend\", not both".into()),
                (None, None) => {
                    let hands = shared.hands.lock().unwrap();
                    if !hands.policy_on {
                        return Err("the policy runtime is off in this game (bot --policy); your lever is `instruct`".into());
                    }
                    Ok(if hands.policy_text.is_empty() { "no policy is in force".to_string() } else { format!("policy in force (version {}):\n{}", hands.policy_version, hands.policy_text) })
                }
                (Some(script), None) => {
                    if !shared.hands.lock().unwrap().policy_on {
                        return Err("the policy runtime is off in this game (bot --policy); your lever is `instruct`".into());
                    }
                    crate::brain::pianist::Policy::check(script).map_err(|e| format!("the script does not parse: {e}"))?;
                    if !script.contains("function decide") && !script.contains("decide =") {
                        return Err("the script defines no `decide` function".into());
                    }
                    shared.policy.lock().unwrap().push(PolicyChange::Set(script.to_string()));
                    Ok(format!("policy replaced ({} lines): it runs from the next game second, once your turn ends", script.lines().count()))
                }
                (None, Some(chunk)) => {
                    if !shared.hands.lock().unwrap().policy_on {
                        return Err("the policy runtime is off in this game (bot --policy); your lever is `instruct`".into());
                    }
                    if chunk.trim() == "-- unchanged" {
                        return Ok("policy unchanged".into());
                    }
                    if shared.hands.lock().unwrap().policy_text.is_empty() && shared.policy.lock().unwrap().iter().all(|c| !matches!(c, PolicyChange::Set(_))) {
                        return Err("no policy is in force to amend: set one first".into());
                    }
                    crate::brain::pianist::Policy::check(chunk).map_err(|e| format!("the amendment does not parse: {e}"))?;
                    shared.policy.lock().unwrap().push(PolicyChange::Amend(chunk.to_string()));
                    Ok(format!("amendment accepted ({} lines): what it defines replaces the same names in place from the next game second", chunk.lines().count()))
                }
            }
        }
        "plan" | "search" => {
            let context = shared.plan_context.lock().unwrap().clone().ok_or("the simulator has no picture of the game yet: try again in a few seconds")?;
            let marks = shared.marks.lock().unwrap().clone();
            planning::run(name, arguments, &context, &marks)
        }
        "units" => {
            let names = arguments["names"].as_array().ok_or("units takes {\"names\": [\"armpw\", ...]}")?;
            let mut lines: Vec<String> = Vec::new();
            for name in names {
                let name = name.as_str().ok_or("names are strings")?;
                lines.push(match crate::brain::pianist::glossary::entry(name) {
                    Some(e) => format!(
                        "{} ({name}): {}, tier {}{}. {}. {}",
                        e.name, e.class, e.tier,
                        if e.made_by.is_empty() { String::new() } else { format!(", made by {}", e.made_by.join(", ")) },
                        e.numbers(),
                        if e.prose.is_empty() { e.gloss.clone() } else { e.prose.clone() }
                    ),
                    None => format!("{name}: not in the glossary"),
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
                if name != "commander" && !name.starts_with("constructor_") {
                    return Err(format!("{name}: lists are by builder name (commander or constructor_N)"));
                }
                // As the model writes them (comet-5, 24:11: "cannot cancel the commander's old solar list, the queue
                // tool rejects every form I try"): the list as a JSON string, "null" as a string, one step as a bare
                // string, an empty list to cancel.
                let value = match value {
                    Value::String(s) if s.trim() == "null" => Value::Null,
                    Value::String(s) if s.trim_start().starts_with('[') => serde_json::from_str(s).map_err(|e| format!("{name}: {e}"))?,
                    Value::String(s) => Value::Array(vec![Value::String(s.clone())]),
                    other => other.clone(),
                };
                let list = match &value {
                    Value::Null => None,
                    Value::Array(items) if items.is_empty() => None,
                    Value::Array(items) => {
                        let steps: Vec<String> = items.iter().map(|v| v.as_str().map(|s| s.trim().to_string()).ok_or_else(|| format!("{name}: steps are strings"))).collect::<Result<_, _>>()?;
                        for step in &steps {
                            let mut words = step.split_whitespace();
                            let kind = words.next().unwrap_or_default();
                            if !matches!(kind, "extractor" | "assist") && !roster.is_empty() && !roster.contains(&kind.to_string()) {
                                return Err(format!("{name}: '{step}' is not a step; a step is extractor, assist, or a unit's internal name from the roster (armsolar, armllt spot_3)"));
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
                    Some(steps) => format!("{name}: {} steps, done in order from its next look", steps.len()),
                    None => format!("{name}: list cancelled"),
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
                    Value::Array(items) => {
                        let names: Vec<String> = items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{name}: rule names are strings"))).collect::<Result<_, _>>()?;
                        super::shared::Footwork::keeping(&names)?
                    }
                    _ => return Err(format!("{name}: \"raw\", \"on\" or a list of rules to keep")),
                };
                parsed.push((name.clone(), footwork));
            }
            let mut lane = shared.lane.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, footwork) in parsed {
                said.push(format!("{name}: {}", footwork.words().unwrap_or_else(|| "all footwork rules (the default)".to_string())));
                if footwork == super::shared::Footwork::default() && name != "all" {
                    lane.remove(&name);
                } else {
                    lane.insert(name, footwork);
                }
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
            let lists = arguments.as_object().filter(|o| !o.is_empty()).ok_or("produce takes an object: factory name (lab_N or plant_N) or \"all\" to a list of unit names (a name with :N caps it at N more), or null to lift it")?;
            let known: Vec<String> = shared.field().roster.iter().map(|(name, _)| name.clone()).collect();
            let mut parsed: Vec<(String, Option<Vec<String>>)> = Vec::new();
            for (name, value) in lists {
                if !matches!(name.as_str(), "all" | "all_builders" | "commander") && !["lab_", "plant_", "factory_", "constructor_"].iter().any(|p| name.starts_with(p)) {
                    return Err(format!("{name}: lists are by actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\""));
                }
                let list = match value {
                    Value::Null => None,
                    Value::Array(items) => {
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
                        Some(units)
                    }
                    _ => return Err(format!("{name}: a list of unit names, or null")),
                };
                parsed.push((name.clone(), list));
            }
            let mut allowed = shared.allowed.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, list) in parsed {
                match list {
                    Some(units) => {
                        said.push(format!("{name} may build only {}", if units.is_empty() { "nothing".to_string() } else { units.join(", ") }));
                        allowed.insert(name, units);
                    }
                    None => {
                        allowed.remove(&name);
                        said.push(format!("{name} may build anything"));
                    }
                }
            }
            Ok(format!("{}; they see it from their next look", said.join("; ")))
        }
        "squad" => squad(arguments, shared),
        "set_production" => {
            let weights = arguments["weights"].as_object().ok_or("weights must be an object")?;
            let known: Vec<String> = shared.field().buildable.iter().map(|(name, _)| name.clone()).collect();
            if let Some(unknown) = weights.keys().find(|name| !known.contains(name)) {
                return Err(format!("{unknown} is not something our factories build; see `buildable`"));
            }
            let mix = weights.iter().map(|(name, w)| (name.clone(), w.as_u64().unwrap_or(0) as u32)).collect();
            shared.field_orders.lock().unwrap().production = mix;
            Ok("production mix set".into())
        }
        "request_turret" => {
            let at = position(arguments, "request_turret")?.ok_or("needs x and z")?;
            // Anywhere we already stand: a turret asked for on ground we hold nothing near is a constructor sent to die.
            let field = shared.field();
            let held = field.extractors.iter().map(|x| &x.at).chain(field.squads.iter().filter_map(|q| q.centre.as_ref())).chain(field.turrets.iter());
            let near = held.map(|p| (p.x as f32 - at.x).hypot(p.z as f32 - at.z)).fold(f32::INFINITY, f32::min);
            if near > 1000.0 {
                return Err(format!("nothing of ours (extractor, turret or squad) within 1000 of there (nearest is {near:.0} away); a constructor would walk there alone. Move a squad there first"));
            }
            let mut orders = shared.field_orders.lock().unwrap();
            orders.turret_requests.push(at);
            Ok("turret requested".into())
        }
        "expansion" => {
            let list = |key: &str| -> Vec<usize> {
                arguments.get(key).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_u64).map(|n| n as usize).collect()).unwrap_or_default()
            };
            let mut orders = shared.field_orders.lock().unwrap();
            orders.spot_priority = list("take_first");
            orders.spot_avoid = list("leave_alone");
            Ok(format!("expansion plan set: {} to take first, {} left alone", orders.spot_priority.len(), orders.spot_avoid.len()))
        }
        "set_directives" => set_directives(arguments, shared),
        _ => Err(format!("unknown tool {name}")),
    }
}

fn squad(arguments: &Value, shared: &Shared) -> Result<String, String> {
    let name = arguments["name"].as_str().filter(|n| !n.is_empty()).ok_or("squad needs a name")?;
    let known: Vec<String> = shared.field().buildable.iter().map(|(name, _)| name.clone()).collect();
    let mut orders = shared.field_orders.lock().unwrap();
    let request = orders.squads.entry(name.to_string()).or_default();
    if let Some(take) = arguments["take"].as_object() {
        if let Some(unknown) = take.keys().find(|unit| !known.contains(unit)) {
            return Err(format!("{unknown} is not a unit our factories build; see `buildable`"));
        }
        for (unit, count) in take {
            *request.take.entry(unit.clone()).or_default() += count.as_u64().unwrap_or(0) as usize;
        }
    }
    if let Some(near) = position(&arguments["near"], "near")? {
        request.near = Some(near);
    }
    if let Some(at) = position(&arguments["post"], "post")? {
        let radius = arguments["post"]["radius"].as_f64().ok_or("post needs a radius")? as f32;
        request.post = Some(Post { at, radius: radius.clamp(100.0, 1500.0) });
    }
    if let Some(to) = position(&arguments["order"], "order")? {
        let kind = parse::<OrderKind>(&arguments["order"]["kind"])?.ok_or("order needs a kind")?;
        request.post = None;
        request.order = Some((kind, to));
        request.seen_by.clear();
    }
    if arguments["release"].as_bool() == Some(true) {
        request.release = true;
        request.seen_by.clear();
    }
    Ok(format!("squad {name} updated. The game is paused during your turn, so members are drawn and orders carried out when the turn ends; your next report shows the result"))
}

fn set_directives(arguments: &Value, shared: &Shared) -> Result<String, String> {
    let frame = shared.briefing().frame;
    let ttl = arguments["ttl_seconds"].as_i64().unwrap_or(DEFAULT_TTL_SECONDS).clamp(10, MAX_TTL_SECONDS);
    let expires_frame = frame + ttl as i32 * 30;
    let fields = arguments.as_object().ok_or("arguments must be an object")?;
    let mut directives = shared.directives.lock().unwrap();
    for (field, value) in fields {
        match field.as_str() {
            "ttl_seconds" => {}
            "army_stance" => {
                directives.army_stance = parse::<Stance>(value)?.map(|value| Timed { value, expires_frame });
            }
            "economy_focus" => {
                directives.economy_focus = parse::<Focus>(value)?.map(|value| Timed { value, expires_frame });
            }
            "wave_size" => {
                directives.wave_size = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "min_constructors" => {
                directives.min_constructors = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "min_converters" => {
                directives.min_converters = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "max_converters" => {
                directives.max_converters = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "expansion_radius" => {
                directives.expansion_radius = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "base_turrets" => {
                directives.base_turrets = parse::<usize>(value)?.map(|value| Timed { value, expires_frame });
            }
            "outpost_turrets" => {
                directives.outpost_turrets = parse::<OutpostTurrets>(value).map_err(|e| format!("outpost_turrets is \"all\", \"none\" or a list of spot numbers: {e}"))?.map(|value| Timed { value, expires_frame });
            }
            "resurrect" => directives.resurrect = parse::<bool>(value)?.map(|value| Timed { value, expires_frame }),
            "tier2" => directives.tier2 = parse::<bool>(value)?.map(|value| Timed { value, expires_frame }),
            "commander_station" => directives.commander_station = position(value, field)?.map(|value| Timed { value, expires_frame }),
            "pressure" => directives.pressure = parse::<bool>(value)?.map(|value| Timed { value, expires_frame }),
            "scout_at" => directives.scout_at = position(value, field)?.map(|value| Timed { value, expires_frame }),
            "attack_target" => directives.attack_target = position(value, field)?.map(|value| Timed { value, expires_frame }),
            "army_station" => directives.army_station = position(value, field)?.map(|value| Timed { value, expires_frame }),
            other => return Err(format!("unknown directive {other}")),
        }
    }
    Ok(format!("in force: {}", directives.describe(frame).join("; ")))
}

fn position(value: &Value, field: &str) -> Result<Option<Vec3>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let coordinate = |axis: &str| value[axis].as_f64().ok_or(format!("{field} needs a number {axis}"));
    Ok(Some(Vec3 { x: coordinate("x")? as f32, y: 0.0, z: coordinate("z")? as f32 }))
}

fn parse<T: serde::de::DeserializeOwned>(value: &Value) -> Result<Option<T>, String> {
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value(value.clone()).map(Some).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_player_has_its_lever_and_none_of_the_commanders() {
        let names = |mode: Mode| tool_list(mode).as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        let player = names(Mode::Player);
        assert_eq!(player, ["overview", "map", "situation", "units", "plan", "search", "instruct", "queue", "lane", "mark", "produce", "policy", "say", "orders", "wait", "note"]);
        let commander = names(Mode::Commander);
        assert!(commander.contains(&"squad".to_string()) && !commander.contains(&"instruct".to_string()));
        for tool in batchable(Mode::Player) {
            assert!(player.contains(&tool.to_string()));
        }
        let shared = Shared::default();
        assert!(call_tool("squad", &json!({ "name": "a" }), &shared, Mode::Player).is_err());
        assert!(call_tool("instruct", &json!({ "text": "commander: build the lab first." }), &shared, Mode::Player).is_ok());
        assert_eq!(*shared.instructions.lock().unwrap(), "commander: build the lab first.");
        assert!(call_tool("instruct", &json!({ "text": "x".repeat(INSTRUCTIONS_LIMIT + 1) }), &shared, Mode::Player).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["extractor spot_45", "armsolar", "armvp", "assist"], "constructor_7": null }), &shared, Mode::Player).is_ok());
        assert_eq!(shared.queues.lock().unwrap().get("commander").cloned().flatten().map(|s| s.len()), Some(4));
        assert!(shared.queues.lock().unwrap().contains_key("constructor_7"));
        // With a roster published, a step must be a unit of it; without one any word passes and the bot skips it.
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armllt".into(), 85)], ..Default::default() });
        assert!(call_tool("queue", &json!({ "commander": ["turret spot_3"] }), &shared, Mode::Player).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["armllt spot_3", "armsolar"] }), &shared, Mode::Player).is_ok());
        // Every player tool but the readers and `orders` itself can be batched (comet-3: `queue` was refused by the
        // batch as "not a tool that can be batched" and the game ran without the list).
        for tool in ["instruct", "queue", "policy", "lane", "mark", "produce", "say", "note", "wait"] {
            assert!(batchable(Mode::Player).contains(&tool), "{tool}");
        }
        assert!(orders(&json!({ "calls": [{ "tool": "queue", "arguments": { "commander": ["armsolar"] } }] }), &shared, Mode::Player).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "group_A": ["armsolar"] }), &shared, Mode::Player).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["windmill"] }), &shared, Mode::Player).is_err());
        // The shapes the model actually sent to cancel or replace a list (comet-5).
        for form in [json!({ "commander": "null" }), json!({ "commander": [] }), json!({ "commander": null })] {
            assert!(call_tool("queue", &form, &shared, Mode::Player).unwrap().contains("cancelled"), "{form}");
            assert!(shared.queues.lock().unwrap().get("commander").cloned().flatten().is_none());
        }
        assert!(call_tool("queue", &json!({ "commander": "[\"assist\"]" }), &shared, Mode::Player).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "commander": "assist" }), &shared, Mode::Player).unwrap().contains("1 steps"));
    }

    #[test]
    fn the_lane_tool_sets_footwork_by_group() {
        use super::super::shared::Footwork;
        let shared = Shared::default();
        assert!(call_tool("lane", &json!({ "group_A": "raw", "all": ["fan", "focus"] }), &shared, Mode::Player).is_ok());
        let lane = shared.lane.lock().unwrap().clone();
        assert_eq!(lane["group_A"], Footwork::raw());
        assert_eq!(lane["all"], Footwork { flee: false, fan: true, focus: true, kite: false, march: false, follow: false });
        assert!(call_tool("lane", &json!({ "group_A": ["dance"] }), &shared, Mode::Player).is_err());
        assert!(call_tool("lane", &json!({ "raiders": "raw" }), &shared, Mode::Player).is_err());
        assert!(call_tool("lane", &json!({}), &shared, Mode::Player).is_err());
        assert!(call_tool("lane", &json!({ "group_A": "on" }), &shared, Mode::Player).is_ok());
        assert!(!shared.lane.lock().unwrap().contains_key("group_A"));
        assert!(call_tool("lane", &json!({ "group_A": "raw" }), &shared, Mode::Commander).is_err());
    }

    #[test]
    fn marks_are_named_places_by_coordinates_or_cell() {
        let shared = Shared::default();
        *shared.map.lock().unwrap() = json!({ "width": 8000.0, "height": 8000.0 });
        assert!(call_tool("mark", &json!({ "south_gate": [3600, 5400], "far_east": "H4" }), &shared, Mode::Player).is_ok());
        let marks = shared.marks.lock().unwrap().clone();
        assert_eq!(marks["south_gate"], (3600.0, 5400.0));
        assert_eq!(marks["far_east"], (7500.0, 3500.0));
        assert!(call_tool("mark", &json!({ "spot_3": [1.0, 1.0] }), &shared, Mode::Player).is_err());
        assert!(call_tool("mark", &json!({ "Gate": [1.0, 1.0] }), &shared, Mode::Player).is_err());
        assert!(call_tool("mark", &json!({ "x": [9000.0, 1.0] }), &shared, Mode::Player).is_err());
        assert!(call_tool("mark", &json!({ "x": "Z9" }), &shared, Mode::Player).is_err());
        assert!(call_tool("mark", &json!({ "south_gate": null }), &shared, Mode::Player).is_ok());
        assert!(!shared.marks.lock().unwrap().contains_key("south_gate"));
    }

    #[test]
    fn produce_whitelists_a_lab_or_all() {
        let shared = Shared::default();
        assert!(call_tool("produce", &json!({ "all": ["armpw", "armham"], "lab_7": [] }), &shared, Mode::Player).is_ok());
        let allowed = shared.allowed.lock().unwrap().clone();
        assert_eq!(allowed["all"], vec!["armpw".to_string(), "armham".to_string()]);
        assert!(allowed["lab_7"].is_empty());
        assert!(call_tool("produce", &json!({ "all": ["armck:1", "armpw"] }), &shared, Mode::Player).is_ok());
        assert!(call_tool("produce", &json!({ "all": ["armck:x"] }), &shared, Mode::Player).is_err());
        assert_eq!(crate::brain::pianist::allowance("armck:2"), ("armck", Some(2)));
        assert_eq!(crate::brain::pianist::allowance("armpw"), ("armpw", None));
        assert!(call_tool("produce", &json!({ "group_A": ["armpw"] }), &shared, Mode::Player).is_err());
        assert!(call_tool("produce", &json!({ "commander": ["armfus:1"], "all_builders": ["armllt", "armsolar"] }), &shared, Mode::Player).is_ok());
        assert_eq!(shared.allowed.lock().unwrap()["all_builders"], vec!["armllt".to_string(), "armsolar".to_string()]);
        assert!(call_tool("units", &json!({ "names": ["armpw"] }), &shared, Mode::Player).is_ok());
        assert!(call_tool("units", &json!({}), &shared, Mode::Player).is_err());
        assert!(call_tool("produce", &json!({ "all": "armpw" }), &shared, Mode::Player).is_err());
        assert!(call_tool("produce", &json!({ "lab_7": null }), &shared, Mode::Player).is_ok());
        assert!(!shared.allowed.lock().unwrap().contains_key("lab_7"));
    }
}

/// The `plan` and `search` tools (docs/design/2026-09-22-plan-search.md, decisions 5 and 6): the player's queue
/// vocabulary in and out of the simulator's plans.
mod planning {
    use std::collections::BTreeMap;

    use buildorder::anneal::{anneal_within, Objective, Palette, Search};
    use buildorder::plan::{Item, Plan, Step};
    use buildorder::sim::{simulate, Outcome};
    use serde_json::{Value, json};

    use super::PlanContext;

    const DEFAULT_MINUTES: f64 = 8.0;
    const MAX_MINUTES: f64 = 20.0;
    const DEFAULT_SECONDS: f64 = 10.0;
    const MAX_SECONDS: f64 = 20.0;
    const SEARCH_THREADS: usize = 4;
    const TARGET_COUNT: usize = 6;

    fn mmss(seconds: f64) -> String {
        let s = seconds.max(0.0).round() as i64;
        format!("{}:{:02}", s / 60, s % 60)
    }

    pub(super) fn run(name: &str, arguments: &Value, ctx: &PlanContext, marks: &BTreeMap<String, (f32, f32)>) -> Result<String, String> {
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
                let objective = if text.starts_with("target") { Objective::parse_target(&text, units)? } else { Objective::parse(&text).ok_or_else(|| format!("{text}: not an objective (income, army, mix, or target UNIT by M:SS)"))? };
                let seconds = arguments["seconds"].as_f64().unwrap_or(DEFAULT_SECONDS).clamp(1.0, MAX_SECONDS);
                let palette = Palette::roster(units, ctx.game.commander, ctx.turret, ctx.water);
                let (factories, constructors) = (ctx.standing_factories + 2, ctx.standing_constructors + 3);
                let start = given.or_else(|| match objective {
                    Objective::Target { unit, .. } => Some(palette.chain_seed(units, ctx.game.commander, unit, TARGET_COUNT, factories, constructors, ctx.wind)),
                    _ => None,
                });
                let search = Search { objective, horizon, iterations: 0, seed: ctx.frame as u64 + 1, factories, constructors, hot: 0.02, start };
                let found = anneal_within(units, &ctx.scenario, &ctx.state, &palette, &search, std::time::Duration::from_secs_f64(seconds), SEARCH_THREADS);
                Ok(report(ctx, &found.plan, &found.outcome, Some(found.score)))
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
            Some((x, z)) => match ctx.spots.iter().enumerate().find(|(_, s)| ((s.0 - x).powi(2) + (s.1 - z).powi(2)).sqrt() < 100.0) {
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
