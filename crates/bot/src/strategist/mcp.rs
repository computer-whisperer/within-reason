//! Minimal MCP server over streamable HTTP: the five methods Claude Code was observed to use
//! (`docs/harness/claude-p.md`), answered with plain JSON.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server};

use super::shared::{Allowance, Firing, Mark, PlanContext, Removal, Shared};
use crate::brain::pianist::step_id;
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
                // A panic in a tool is the tool's error, not the server's end (player-43: the `plan` tool panicked
                // at 1:00 and no call of the player's reached the bot for the rest of the game).
                let reply = serde_json::from_str::<Value>(&body).ok().and_then(|call| {
                    let id = call.get("id").cloned();
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(&call, &shared, &transcript))) {
                        Ok(reply) => reply,
                        Err(payload) => {
                            let what = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned()).unwrap_or_else(|| "a panic".to_string());
                            eprintln!("[mcp] a tool panicked: {what}");
                            id.map(|id| json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": format!("the tool failed inside the bot ({what}); the bot is still running, your other tools work") }], "isError": true } }))
                        }
                    }
                });
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
        "tools/list" => Ok(json!({ "tools": if shared.commander.load(Ordering::Relaxed) {
            super::command::tool_list()
        } else if shared.side.get().is_some() {
            // Under a commander chat is the commander's.
            Value::Array(tool_list().as_array().into_iter().flatten().filter(|t| t["name"] != "say" && t["name"] != "surrender").cloned().collect())
        } else {
            tool_list()
        } })),
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
pub(super) fn openai_tools(commander: bool) -> Vec<Value> {
    if commander { super::command::tool_list() } else { tool_list() }
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
            "chat": { "type": "boolean", "description": "A person in the game says something (default on)." },
            "missile_ready": { "type": "boolean", "description": "A silo of ours has a missile come ready (default on); the report's `missiles` line says what each silo holds." } } } });
    let orders = |tools: &[&str], what: &str| json!({ "name": "orders",
        "description": format!("Your whole turn in one call, and it ENDS the turn: {what}, carried out in the order listed, then the game resumes. Each entry names one of the other tools and its arguments, exactly as you would call it alone. Include a `wait` entry, LAST, to change when you are next woken; without one the wake settings in force stand. Never call the other tools on their own beside this one: anything sent after the turn ends is refused. Call nothing and write nothing after it."),
        "inputSchema": { "type": "object", "additionalProperties": false, "required": ["calls"], "properties": {
            "calls": { "type": "array", "minItems": 1, "items": { "type": "object", "additionalProperties": false, "required": ["tool"], "properties": {
                "tool": { "type": "string", "enum": tools },
                "arguments": { "type": "object", "description": "That tool's arguments, e.g. {\"text\": \"...\"} for note." } } } } } } });
    json!([
            overview, map,
            { "name": "situation",
              "description": "The picture your hands read this second (without your instructions and the standing rules; Jev is sent it without the lines only you can act on: his buildings by place and what was looked at when): economy, ours, enemy, places by name, every actor with what it is doing, recent events. You are sent a summary of it at the start of every turn; call this to read the whole picture, or a place's entry by name.",
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
              "description": "A builder's build list, done exactly and in order by the bot itself without asking your hands: an object of builder name (commander, constructor_N; a builder still being built may be named too, by the name its lab's or its builder's line gives it, and its list waits for it to stand) to a list of steps, or null (or an empty list) to cancel its list; null leaves a build in progress to finish, a list whose first step is \"stop\" (or the bare word \"stop\") drops that build first, its frame decaying, and the rest of the list follows (a build whose frame began after the report you read is finished first even so, since your stop was written before it stood; the hands' line says which). Steps: \"finish <handle>\" (resume an unfinished structure of ours by the handle the report's `unfinished structures` line gives, whoever began it: it is this builder's own build from then on, finished first before any later list), \"extractor spot_N\" (or \"extractor\" for the nearest free spot), any building by its internal name as the roster lists it (\"armsolar\", \"armvp\", \"armnanotc\", \"armfus\") or by a role word the seat's faction fills in (\"solar\", \"wind\", \"lab\", \"plant\", \"air_plant\", \"turret\", \"radar\", \"nano\", \"metal_storage\", \"energy_storage\", \"converter\", \"advanced_lab\", \"advanced_extractor\": the way to write a list before the faction is known), one that stands at a place with the place after it (\"armllt spot_3\", \"armrad home\", \"armmoho spot_3\" over our extractor there); any other building may take a place too (\"armvp spot_39\", \"armnanotc home\") and without one a factory stands beside its builder toward the yard, a generator beside the builder when it stands at home, else in the back field at home, out of every factory's exit lane either way (the builder walks: the pros' solars are at base; a generator at a place you mark is walked to and back from, so a mark far from the plant costs the commander that walk twice at every solar: migration-3-1 marked one 970 from home and the commander left the plant it was helping for 70 s per pair), a construction turret beside the nearest factory (it guards that factory once it stands), \"assist N\" (help the nearest factory for N seconds, then the next step: the pros guard the plant between their own builds), \"assist\" (help the nearest factory, standing or being built: the last step of a list, and only the last; the tool refuses steps after it). A step may carry the word \"low\" (\"assist 90 low\", \"armsolar low\"): on that step the builder draws only the team's surplus and stands still in a metal stall, so the other builders' work comes first; the pros set their commander low while it guards the plant or builds a solar in the opening while the constructors claim spots, and high again for a build of its own. Plants and construction turrets are low always, as every human's are; a step without the word is high. Each step is ordered when the one before is 60% built, so nothing idles; a step may end with an id of your choosing (\"armavp avp_yard #avp1\", \"armsolar #s3\"): steps with the same id, on any builders' lists and in a list sent again, are one building: the first builder to reach the step starts it, any other helps build it, and once it stands the step is done, so several constructors raise one factory and a plan sent again builds nothing twice (without ids each step is its own building: the same factory step on three lists is three factories); a new list replaces the old one whole, starting from its own first step: it takes over a builder that is helping a factory, walking, reclaiming or repairing at once, a build in progress is finished first, and a step the old list had queued behind that build is dropped; a step naming a building this builder cannot make, or a turret, radar or jammer with no place, is refused here; a step that cannot be done when its turn comes (the spot taken, a place unknown) is skipped and said in the hands' report. While a list runs the builder is off your hands' menu unless an enemy is on it, and a builder your hands sent away from an enemy takes no list step while that enemy is still on it; your instructions take over when the list is done. This is how an opening is made to happen as written: the hands do not follow a sequence (comet-1, comet-2: 'three solars, then the plant' got extractors and the plant at 1:45).",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" }, { "type": "string", "enum": ["stop"] } ] }, "description": "Builder name to steps, null, or \"stop\"." } },
            { "name": "lane",
              "description": "Which footwork reflexes your hands' code applies to a group's soldiers between your hands' orders, per group name or for \"all\": \"raw\" (none: the group's orders reach the engine exactly as given), \"on\" (all three, the default), or a list of the reflexes to keep. The reflexes: flee (a soldier steps out of the reach of a turret or a fight it was not sent against, or one it would die in), fan (spreads out under a commander's D-gun), kite (a soldier that outranges its enemy steps back while reloading). Roving (fast units looking at what we know least, killing what they find unguarded and keeping out of every reach, in code, ten times a second, beyond your hands' reach) is a move of your hands, not a setting: `rove` puts a whole group to it and `rove_1` its fastest soldier as a group of its own, on a pick, so say in the packet which group roves and when; the group's entry says what each rover is doing and what it has found (`rove`). A setting here never takes a group off its rove: the way back is a pick too, so name the group's new job in the packet. A setting stands until you change it; the group's picture entry shows it when it is not the default.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "string", "enum": ["raw", "on"] }, { "type": "array", "items": { "type": "string", "enum": ["flee", "fan", "kite"] } } ] }, "description": "Group name (group_A) or \"all\" to its setting." } },
            { "name": "mark",
              "description": "Name a place of your own for your hands: an object of name to [x, z] map coordinates, a grid cell (\"E7\": its centre), or a range of cells (\"E1-H8\": the rectangle from E1 to H8, a region whose point is its centre), or null to forget it. A marked place joins the picture's places at once, so instructions can send groups and builders there (\"group_B: advance to south_gate\"), and its entry says what enemy is near. A region is also where a group can be told to rove: a packet that names it for a group puts `rove_<region>` and `rove_1_<region>` on that group's menu (its soldiers look only at places inside the region), so \"the Rovers rove in his half\" is a mark of his half and a packet naming it. Names are lower-case words with underscores; home, spot_N, passage_N and group_N are taken. Any spot or passage you name in the packet is on your hands' menu already, however far; mark is for places that are not spots.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, { "type": "string" }, { "type": "null" } ] }, "description": "Place name to [x, z], a grid cell, or null." } },
            { "name": "produce",
              "description": "What each factory or builder may build, and for a factory which group its new soldiers join ({\"plant_7\": {\"units\": [\"armflash\"], \"group\": \"group_A\"}}, or \"new\" for a fresh group of that factory's own, or \"by_type\" for one group of that factory's per unit type, so its Rovers and its Blitzes never share a group; by default a factory's soldiers gather in one group of its own and nothing merges by itself) and how they walk to it (\"join_in\": 4: its new soldiers wait beside the plant, out of its lane, and walk to their group four together, never alone; null lifts it): an object of actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" (every commander and constructor) or \"all\" (everyone) to a list of unit names (as the roster writes them: armpw, armham, armck, armfus; or role words the seat's faction fills in: constructor, vehicle_constructor, raider, line, rez, advanced_constructor, and the buildings' role words), or null to lift the restriction. A name with a count after a colon (armck:1) is made that many more times from now and then drops off the list by itself; a factory makes its counted entries in the order written, each to its count, without asking (armcv:1, armfav:3, armcv:1, armflash is one constructor, three Rovers, one constructor, then Blitzes); saying the same list again restarts it from the first entry: the way to say 'one constructor, then raiders' to hands that cannot count. With no counted entry left a factory is offered only the entries without a count, every time it is asked; your instructions say which of them and when. A builder without a list is offered the usual buildings (generators, factories, light and heavy turrets, radar, storage, the tier-2 lab and extractor, fusion); a list replaces that, so a fusion reactor, an aircraft plant or a jammer from a constructor is asked for here. Use it when the packet's words are not getting the mix you want. A list naming nothing the actor can build leaves it unrestricted; the actor's entry in the picture shows its list. Your policy is not bound by lists: it may order anything a builder can build.",
              "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "array", "items": { "type": "string" } }, { "type": "null" } ] }, "description": "Actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\" to unit names, or null." } },
            { "name": "transfer",
              "description": "Move metal or energy to another seat of our side, ours or an ally's (seats of ours by tag in the `seats:` or `our other seats` line, allies in the `allies` line), and units between seats of ours: {\"metal\": 800, \"energy\": 0, \"from\": \"t2\", \"to\": \"t1\"} sends what the receiving seat's store can hold (the game caps it there; what does not fit stays), or {\"units\": [\"group_A_t2\", \"constructor_31002\", \"plant_4410\"], \"to\": \"t1\"} gives those units (a group's members, a builder, a plant, by actor name or <unit>_<id> handle) to that seat of ours, whose hands take them into a group of their own and whose builders they become; a constructor or factory of another faction builds its own faction's things there (write them by their own names or by role words). A standing transfer, {\"from\": \"t2\", \"to\": \"t5\", \"keep_metal\": 100, \"keep_energy\": 2000}, has t2's hands send t5 everything above those amounts in t2's store every second until {\"from\": \"t2\", \"standing\": false} lifts it; one keep or both. This is how a seat feeds a sprint: a one-shot send moves a surplus, and a seat spending as it earns never has one (Center Command 2v4: 450 metal fed in nineteen minutes against three asks). What it makes possible: the sprint to tier 2 (the seat whose advanced plant is placed fed by the others until it stands, its advanced constructors given to each seat and passed on), a sprinting ally fed the same way, one army under one seat, a dead seat's plants and constructors given to a live one. Your hands carry it out from their next look and say what went.",
              "inputSchema": { "type": "object", "properties": { "metal": { "type": "number" }, "energy": { "type": "number" }, "units": { "type": "array", "items": { "type": "string" } }, "from": { "type": "string" }, "to": { "type": "string" }, "keep_metal": { "type": "number" }, "keep_energy": { "type": "number" }, "standing": { "type": "boolean" } } } },
            { "name": "remove",
              "description": "Take apart or blow up what we own: {\"reclaim\": [handles], \"by\": \"constructor_N\" (optional; else the nearest builder without a list)} puts `reclaim <handle>` steps at the front of that builder's list, and most of the metal comes back; {\"destruct\": [handles]} sends the engine's self-destruct, and nothing comes back. A handle is a unit's name and id as the picture writes it (armsolar_31002: a factory's `yard` entry names the buildings in its exit lane) or an actor's name (constructor_N, plant_N, commander); for a reclaim, \"<type> at <place or grid cell>\" (\"armwin at spot_64\", \"armsolar at E4\") names the nearest standing unit of that type to the place, for a building the picture gives no handle for, and \"unfinished <type> at <place or grid cell>\" the nearest unfinished frame of the type there (the report's `unfinished structures` line gives frames' handles too). Every unit that self-destructs blows up: the answer says the blast's radius and damage and what of ours stands inside it, and a destruct that would kill something of ours is refused unless \"accept_losses\": true. The commander's blast is the game's largest; a reclaim is the safe way beside anything that matters.",
              "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "reclaim": { "type": "array", "items": { "type": "string" } }, "by": { "type": "string" }, "destruct": { "type": "array", "items": { "type": "string" } }, "accept_losses": { "type": "boolean" } } } },
            { "name": "fire",
              "description": "Fire a nuclear silo's ready missile at a place: {\"silo\": \"armsilo_31002\" (its handle as the picture's `stockpilers` entry writes it), \"at\": [x, z] or a grid cell (\"E7\": its centre)}. The silo launches about nine seconds after the order and the warhead lands thirteen seconds later: 12,500 damage at the point, nothing 640 away, so aim at the point itself (his advanced plant, his fusions, his commander standing still), not at a cell. The game keeps a silo stockpiling to ten missiles on its own, each two minutes and about 1,042 energy a second; `fire` spends one. His anti-nuke takes any missile aimed within 2,000 of it once it has an interceptor (90 s after it stands); the enemy's buildings by place name his anti-nukes with that coverage. Refused when no missile is ready.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["silo", "at"], "properties": { "silo": { "type": "string" }, "at": { "oneOf": [ { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, { "type": "string" } ] } } } },
            { "name": "say",
              "description": "Say something in the game's chat, to everyone playing. Short lines: the game shows 127 characters a line, the bot prefixes `[WReason] ` to each (never write it yourself) and splits a longer text into lines that fit. The report shows what people say to you; when an experienced player offers advice or asks what you are doing, answer, and ask them what they would do: their feedback is what this project learns from.",
              "inputSchema": { "type": "object", "additionalProperties": false, "required": ["text"], "properties": { "text": { "type": "string", "maxLength": 240 } } } },
            signal_tool(),
            surrender_tool(),
            orders(&["instruct", "queue", "lane", "mark", "produce", "remove", "fire", "transfer", "signal", "say", "note", "predict", "wait"], "your instructions or policy, build lists, footwork settings, marked places, what labs may build, removals, a missile, a signal, a chat line, a note, a prediction of his state and when to be woken"),
            wait("A group of ours starts fighting an enemy party.", "Woken when this many soldiers of each named unit type are alive, e.g. {\"armham\": 6}. {} clears it."),
            note,
            predict_tool(),
    ])
}

/// The `signal` tool, the player's and the commander's alike.
pub(super) fn signal_tool() -> Value {
    json!({ "name": "signal",
        "description": "A named signal: the moment a waiting group goes, given as a fact the hands can see. An object of name to when it is given: \"now\", a game clock (\"6:00\"), or null to forget it. The hands' picture states every signal's status each second (\"go_west: not yet given (set for 6:00)\", then \"go_west: given, at 6:00\"), and they act on that where they do not act on a time written in a packet. So the packet says \"holds at spot_24 until go_west is given, then attacks his_base\", and the signal is set here. Setting a given signal's time again un-gives it. Names are lower-case words with underscores.",
        "inputSchema": { "type": "object", "additionalProperties": { "oneOf": [ { "type": "string" }, { "type": "null" } ] }, "description": "Signal name to \"now\", a clock such as \"6:00\", or null." } })
}

/// The `signal` tool's work, on the caller's own state.
pub(super) fn signal(arguments: &Value, shared: &Shared) -> Result<String, String> {
    let entries = arguments.as_object().filter(|o| !o.is_empty()).ok_or("signal takes an object: signal name to \"now\", a game clock such as \"6:00\", or null to forget it")?;
    let frame = shared.briefing().frame;
    let mut parsed: Vec<(String, Option<i32>)> = Vec::new();
    for (name, value) in entries {
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || !name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            return Err(format!("{name}: a signal's name is lower-case words with underscores"));
        }
        let at = match value {
            Value::Null => None,
            // null as the model sometimes writes it, in a string (signal-rehearsal 3:20).
            Value::String(s) if s.trim() == "null" => None,
            Value::String(s) if s.trim() == "now" => Some(frame),
            Value::String(s) => {
                let (m, sec) = s.trim().split_once(':').ok_or(format!("{name}: \"now\" or a game clock such as \"6:00\""))?;
                let (m, sec): (i32, i32) = (m.parse().map_err(|_| format!("{name}: {s} is not a clock"))?, sec.parse().map_err(|_| format!("{name}: {s} is not a clock"))?);
                Some((m * 60 + sec) * 30)
            }
            _ => return Err(format!("{name}: \"now\", a game clock such as \"6:00\", or null")),
        };
        parsed.push((name.clone(), at));
    }
    let clock = |f: i32| format!("{}:{:02}", f / 30 / 60, f / 30 % 60);
    let mut signals = shared.signals.lock().unwrap();
    let mut said = Vec::new();
    for (name, at) in parsed {
        match at {
            None => {
                signals.remove(&name);
                said.push(format!("{name} forgotten"));
            }
            Some(at) => {
                signals.insert(name.clone(), at);
                said.push(if at <= frame { format!("{name} is given now ({}): the hands see it from their next look", clock(frame)) } else { format!("{name} will be given at {} ({} s from now); until then the hands read it as not yet given", clock(at), (at - frame) / 30) });
            }
        }
    }
    Ok(said.join("; "))
}

/// The `surrender` tool, the player's and the commander's alike.
pub(super) fn surrender_tool() -> Value {
    json!({ "name": "surrender",
        "description": "Give the game up: every unit of ours self-destructs within seconds and the game ends as a loss. It cannot be taken back. Only when the game is lost beyond any recovery (no factory and no means to build one, or an army and economy a small fraction of his with his army in our base), or when the person hosting us asks for it in chat. Say a closing line in chat first (`say`), then call this with the reason.",
        "inputSchema": { "type": "object", "additionalProperties": false, "required": ["reason"], "properties": { "reason": { "type": "string" } } } })
}

/// The `surrender` tool's work, on the caller's own state.
pub(super) fn surrender(arguments: &Value, shared: &Shared) -> Result<String, String> {
    let reason = arguments["reason"].as_str().map(str::trim).filter(|r| !r.is_empty()).ok_or("surrender needs its reason under \"reason\"")?;
    let time = shared.briefing().game_time;
    shared.notes.lock().unwrap().push(format!("[{time}] surrendered: {reason}"));
    shared.surrender.store(true, Ordering::Relaxed);
    eprintln!("[strategist] surrender at {time}: {reason}");
    Ok("surrendered: every unit of ours is self-destructing and the game will end. Nothing more is needed from you".into())
}

/// A grid cell's column and row, 0-based ("E7" gives (4, 6)).
fn cell_of(cell: &str, label: &str) -> Result<(u8, u8), String> {
    let mut chars = cell.trim().chars();
    let column = chars.next().map(|c| c.to_ascii_uppercase()).filter(|c| ('A'..='H').contains(c)).ok_or(format!("{label}: a grid cell is A1 to H8"))?;
    let row: u32 = chars.as_str().parse().ok().filter(|r| (1..=8).contains(r)).ok_or(format!("{label}: a grid cell is A1 to H8"))?;
    Ok((column as u8 - b'A', row as u8 - 1))
}

/// A region the player marks: two cells with a dash between ("E1-H8", "H8-E1" the same), the rectangle they span;
/// its point is the centre.
fn region_of(range: &str, width: f32, height: f32, label: &str) -> Result<Mark, String> {
    let (a, b) = range.split_once('-').ok_or(format!("{label}: a region is two cells with a dash between (E1-H8)"))?;
    let ((ca, ra), (cb, rb)) = (cell_of(a, label)?, cell_of(b, label)?);
    let cells = (ca.min(cb), ra.min(rb), ca.max(cb), ra.max(rb));
    if width <= 0.0 {
        return Err("the map is not known yet; give [x, z]".to_string());
    }
    let (c0, r0, c1, r1) = cells;
    let x = (c0 as f32 + c1 as f32 + 1.0) / 2.0 * width / 8.0;
    let z = (r0 as f32 + r1 as f32 + 1.0) / 2.0 * height / 8.0;
    Ok(Mark { x, z, cells: Some(cells) })
}

/// A place the player gives: `[x, z]` as numbers, the pair in a string as the model sometimes writes it ("[1264, 2832]":
/// human-19), or a grid cell ("E7": its centre). `label` names the argument in the error. `mark` and `fire` read it.
fn point_of(value: &Value, width: f32, height: f32, label: &str) -> Result<(f32, f32), String> {
    let on_map = |x: f32, z: f32| -> Result<(f32, f32), String> {
        if width > 0.0 && (x < 0.0 || z < 0.0 || x > width || z > height) {
            return Err(format!("{label}: ({x:.0}, {z:.0}) is off the map, which is {width:.0} by {height:.0}"));
        }
        Ok((x, z))
    };
    match value {
        Value::Array(xz) if xz.len() == 2 => {
            let (x, z) = (xz[0].as_f64().ok_or(format!("{label}: [x, z] are numbers"))? as f32, xz[1].as_f64().ok_or(format!("{label}: [x, z] are numbers"))? as f32);
            on_map(x, z)
        }
        Value::String(pair) if pair.trim_start().starts_with('[') => {
            let xz: Vec<f32> = serde_json::from_str(pair).map_err(|_| format!("{label}: [x, z] are two numbers"))?;
            let [x, z] = xz[..] else { return Err(format!("{label}: [x, z] are two numbers")) };
            on_map(x, z)
        }
        Value::String(cell) => {
            let (column, row) = cell_of(cell, label)?;
            if width <= 0.0 {
                return Err("the map is not known yet; give [x, z]".to_string());
            }
            Ok((column as f32 * width / 8.0 + width / 16.0, row as f32 * height / 8.0 + height / 16.0))
        }
        _ => Err(format!("{label}: [x, z] or a grid cell such as \"E7\"")),
    }
}

/// What `orders` may batch.
/// The `predict` tool, the player's and the commander's alike (the user, 2026-10-17: "some way for the player to report
/// hidden state predictions throughout the game that we can easily score later ... numeric predictions of mex count
/// and army value", with his advanced factory as the third field and the commander given it too).
pub(super) fn predict_tool() -> Value {
    json!({ "name": "predict",
        "description": "Your estimate of what the opponent has NOW, which you cannot see: his extractors standing, the metal value of his army alive (soldiers, not buildings), and whether an advanced factory of his stands. The report gives sightings with their age and never his state; this is where your reading of them, the clock and the brief's tables goes. One number each, your best single guess, and a sentence on what it rests on. Written beside the game time with what the report showed, scored against the truth after the game; nothing in the game reads it. Give it every few minutes and whenever your reading changes.",
        "inputSchema": { "type": "object", "properties": {
            "extractors": { "type": "integer", "minimum": 0, "description": "his extractors standing now" },
            "army_metal": { "type": "integer", "minimum": 0, "description": "the metal value of his army alive now" },
            "advanced_factory": { "type": "string", "enum": ["none", "building", "standing"], "description": "his advanced factory now: none started, being built, or standing" },
            "basis": { "type": "string", "description": "one sentence: what the estimate rests on" }
        }, "required": ["extractors", "army_metal", "advanced_factory", "basis"], "additionalProperties": false } })
}

/// One `predict` row: the numbers, the game time, and what the report showed at that turn (his soldiers seen and
/// their metal, his extractors seen and the ages of the looks, his factories seen, our own extractors and army), so
/// the scorer can tell a reading from a copy of the floor. Appended to `predictions-<name>.jsonl`; the last one is
/// kept for the takeover prompt. For the commander the field is the side's.
pub(super) fn predict(arguments: &Value, shared: &Arc<Shared>) -> Result<String, String> {
    let extractors = arguments["extractors"].as_u64().ok_or("`extractors` takes his extractors standing now, a whole number")?;
    let army_metal = arguments["army_metal"].as_u64().ok_or("`army_metal` takes the metal of his army alive now, a whole number")?;
    let advanced = arguments["advanced_factory"].as_str().filter(|s| ["none", "building", "standing"].contains(s)).ok_or("`advanced_factory` is one of none, building, standing")?;
    let basis = arguments["basis"].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("`basis` takes a sentence on what the estimate rests on")?;
    let field = shared.field();
    let s = &field.score;
    let time = shared.briefing().game_time;
    let row = json!({
        "frame": s.frame, "clock": time,
        "extractors": extractors, "army_metal": army_metal, "advanced_factory": advanced, "basis": basis,
        "report": {
            "enemy_soldiers_seen": s.enemy_soldiers_seen, "enemy_soldiers_seen_metal": s.enemy_soldiers_seen_metal,
            "enemy_extractors_seen": s.enemy_spots.len(),
            "youngest_look_s": s.enemy_spots.iter().map(|(_, _, ago)| *ago).min(), "oldest_look_s": s.enemy_spots.iter().map(|(_, _, ago)| *ago).max(),
            "enemy_factories_seen": s.enemy_factories.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
            "ours": { "extractors": s.extractors, "army_metal": s.army_metal },
        },
    });
    if let Some(file) = shared.predictions.lock().unwrap().as_mut() {
        writeln!(file, "{row}").map_err(|e| format!("the prediction could not be written: {e}"))?;
    }
    let words = format!("his extractors {extractors}, his army {army_metal} metal, his advanced factory {advanced}: {basis}");
    *shared.last_prediction.lock().unwrap() = Some(format!("[{time}] {words}"));
    Ok(format!("recorded at {time}: {words}; scored against the truth after the game"))
}

fn batchable() -> &'static [&'static str] {
    &["instruct", "queue", "lane", "mark", "produce", "remove", "fire", "transfer", "signal", "say", "note", "predict", "wait"]
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
    if shared.commander.load(Ordering::Relaxed) {
        if let Some(outcome) = super::command::call_tool(name, arguments, shared) {
            return outcome;
        }
    } else if name == "surrender" && shared.side.get().is_some() {
        return Err("giving the game up is the commander's in this game. Write a `note` when you think it is lost: the commander reads it".into());
    } else if name == "say" && shared.side.get().is_some() {
        return Err("chat is the commander's in this game: it hears the people and answers them. Write a `note` when there is something it should say".into());
    } else if !tool_list().as_array().is_some_and(|tools| tools.iter().any(|t| t["name"] == name)) {
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
        "predict" => predict(arguments, shared),
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
            flag("missile_ready", &mut wake.missile_ready);
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
            let marks = shared.mark_points();
            planning::run(name, arguments, 0.0, &context, &marks)
        }
        "search" => {
            // Beside the game by default, the answer with the next report; `wait` holds the turn for it. The budget's
            // cap is the mode's: lockstep holds the game anyway, realtime does not (the user, 2026-09-22: in-game
            // thinking time can cost games).
            let context = shared.plan_context.lock().unwrap().clone().ok_or("the simulator has no picture of the game yet: try again in a few seconds")?;
            let marks = shared.mark_points();
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
            let field = shared.field();
            let roster: Vec<String> = field.roster.iter().map(|(name, _)| name.clone()).collect();
            let lists = arguments.as_object().filter(|o| !o.is_empty()).ok_or("queue takes an object: builder name (commander or constructor_N) to a list of steps, or null to cancel")?;
            let mut parsed: Vec<(String, Option<Vec<String>>)> = Vec::new();
            for (name, value) in lists {
                let name = &shared.actor_name(name);
                // Refused now, not accepted and dropped by the hands seconds into the game.
                if let Some(own) = shared.own_commander.lock().unwrap().as_ref().filter(|own| name.starts_with("commander") && name != *own) {
                    return Err(format!("{name}: your seat's commander is {own} (or `commander`); another seat's is its own player's"));
                }
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
                            let (body, tag) = step_id(step);
                            // The word `low` (the builder draws only surplus on the step) is not one of the step's words.
                            let (body, _) = crate::brain::pianist::step_low(body);
                            // The word `start` (the builder leaves the frame once it stands up) is not one of the step's words either.
                            let (body, start) = crate::brain::pianist::step_start(&body);
                            let mut words = body.split_whitespace();
                            let kind = words.next().unwrap_or_default();
                            if start && matches!(kind, "assist" | "stop" | "reclaim" | "finish" | "extractor") {
                                return Err(format!("{name}: '{step}': `start` goes on a step that builds a structure by name (cormoho spot_28 start): the builder places the frame and moves on"));
                            }
                            // An id is the step's last word (`#avp1`), on a step that builds something.
                            if body.split_whitespace().any(|w| w.starts_with('#')) {
                                return Err(format!("{name}: '{step}': an id is one word after everything else in the step (armavp avp_yard #avp1)"));
                            }
                            if let Some(tag) = tag {
                                if matches!(kind, "assist" | "stop" | "reclaim" | "finish") {
                                    return Err(format!("{name}: '{step}': an id goes on a step that builds something, not on {kind}"));
                                }
                                if tag.len() > 24 || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                    return Err(format!("{name}: '{step}': an id is letters, digits and underscores, 24 at most"));
                                }
                            }
                            if kind == "stop" && i > 0 {
                                return Err(format!("{name}: 'stop' only begins a list (it drops the build in progress, then the list follows)"));
                            }
                            // A unit of another faction is a step too when this builder makes it (a constructor given by a
                            // seat of that faction: Center Command 2v4, 2026-10-19, `cormoho` refused to the Armada seat
                            // holding a Cortex advanced constructor, which then held no mohos all game).
                            let builder_makes = field.builders.get(name.as_str()).is_some_and(|can| can.contains_key(kind));
                            if !matches!(kind, "extractor" | "assist" | "stop" | "reclaim" | "finish") && !crate::brain::roster::ROLE_WORDS.contains(&kind) && !roster.is_empty() && !roster.contains(&kind.to_string()) && !builder_makes {
                                return Err(format!("{name}: '{step}' is not a step; a step is extractor, assist, reclaim, finish, a role word ({}) or a unit's internal name from the roster (armsolar, armllt spot_3), or of another faction when this builder makes it", crate::brain::roster::ROLE_WORDS.join(", ")));
                            }
                            // `finish <handle>`: an unfinished structure of ours by its handle.
                            if kind == "finish" {
                                let what = body.split_whitespace().nth(1).unwrap_or_default();
                                if what.is_empty() || !what.contains('_') || body.split_whitespace().count() != 2 {
                                    return Err(format!("{name}: '{step}': finish takes the handle of an unfinished structure of ours (finish armsolar_31002), as the report's `unfinished structures` line writes it"));
                                }
                            }
                            // `reclaim <what>`: a unit of ours by handle, a wreck field by its grid cell or by a place.
                            if kind == "reclaim" {
                                let what = body.split_whitespace().nth(1).unwrap_or_default();
                                let cell = what.len() == 2 && what.starts_with(|c: char| c.is_ascii_alphabetic()) && what.ends_with(|c: char| c.is_ascii_digit());
                                let handle = what.contains('_') || what == "commander";
                                if what.is_empty() || !(cell || handle || what.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
                                    return Err(format!("{name}: '{step}': reclaim takes a unit of ours by handle (armsolar_31002), a wreck field by its grid cell (reclaim E4) or by a place (reclaim spot_40)"));
                                }
                            }
                            // What this builder can make, and what needs a place, refused now rather than skipped by
                            // the hands with a line in the next report (the impedance tally, 2026-10-16: 180 "this
                            // builder cannot build it" and 35 "it needs a place" skips in 23 games, ten episodes in
                            // nuke-2-hard alone). A builder the field does not know yet (named before it stands) is not checked.
                            if let Some(can) = field.builders.get(name.as_str()).filter(|_| !matches!(kind, "assist" | "stop" | "reclaim" | "finish")) {
                                let place = words.next().filter(|p| !(kind == "extractor" && *p == "nearest"));
                                match can.get(kind) {
                                    // An id's building it cannot make: it helps whoever builds that id (the hands).
                                    None if tag.is_some() => {}
                                    None => {
                                        let mut makes: Vec<&str> = can.keys().map(String::as_str).filter(|w| !crate::brain::roster::ROLE_WORDS.contains(w) && *w != "extractor").collect();
                                        makes.sort_unstable();
                                        return Err(format!("{name}: '{step}': it cannot build {kind}; what it builds: {} (with an id, '{kind} #name', it helps whichever builder starts that building)", makes.join(", ")));
                                    }
                                    Some(true) if place.is_none() => return Err(format!("{name}: '{step}': a {kind} stands at a place: write the place after it (armllt spot_3, armrad home)")),
                                    _ => {}
                                }
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
            // Steps carrying the same id are one building, so one id names one thing; and the same factory step on
            // two builders without an id is two factories (player-33, 12:17: an Advanced Vehicle Plant twice from
            // three lists with the same step), which the answer says without refusing it.
            let mut tags: std::collections::BTreeMap<&str, (String, &str)> = Default::default();
            let mut untagged: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
            for (name, list) in &parsed {
                for step in list.iter().flatten() {
                    let (body, tag) = step_id(step);
                    let (body, _) = crate::brain::pianist::step_low(body);
                    let (body, _) = crate::brain::pianist::step_start(&body);
                    match tag {
                        Some(tag) => match tags.get(tag) {
                            Some((first, on)) if *first != body => return Err(format!("#{tag} names two buildings: '{first}' on {on}'s list and '{body}' on {name}'s; an id is one building, written the same on every list")),
                            Some(_) => {}
                            None => {
                                tags.insert(tag, (body, name));
                            }
                        },
                        None if body.split_whitespace().next().is_some_and(|kind| crate::brain::roster::FACTORY_ROLES.contains(&kind) || crate::brain::pianist::glossary::entry(kind).is_some_and(|e| e.class.contains("factory"))) => {
                            let on = untagged.entry(body).or_default();
                            if !on.contains(&name.as_str()) {
                                on.push(name);
                            }
                        }
                        None => {}
                    }
                }
            }
            let twice: Vec<String> = untagged.iter().filter(|(_, on)| on.len() > 1).map(|(step, on)| format!("note: {} each have '{step}' without an id: that is {} factories, one each; for one factory that they build together give the step the same id on every list ('{step} #{}1')", on.join(" and "), on.len(), step.split_whitespace().next().unwrap_or("id"))).collect();
            let mut queues = shared.queues.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, list) in parsed {
                said.push(match &list {
                    Some(steps) if steps.len() == 1 && steps[0] == "stop" => format!("{name}: list cancelled and its build in progress dropped (a frame begun after the report you read finishes first instead; the hands' line says which, and what was dropped)"),
                    Some(steps) if steps[0] == "stop" => format!("{name}: its build in progress dropped (a frame begun after the report you read finishes first instead; the hands' line says which), then {} steps in order from its next look", steps.len() - 1),
                    Some(steps) => format!("{name}: {} steps, done in order from its next look", steps.len()),
                    None => format!("{name}: list cancelled; a build in progress finishes (begin the list with \"stop\" to drop it)"),
                });
                queues.insert(name, list);
            }
            said.extend(twice);
            Ok(said.join("; "))
        }
        "lane" => {
            let settings = arguments.as_object().filter(|o| !o.is_empty()).ok_or("lane takes an object: group name (or \"all\") to \"raw\", \"on\" or a list of the rules to keep")?;
            // Roving is the hands' `rove` move, a pick, since 2026-10-07, and any later pick the way back since
            // 2026-10-08 (docs/design/2026-10-07-rove-approval.md): this tool is footwork only.
            let mut parsed: Vec<(String, super::shared::Footwork)> = Vec::new();
            for (name, value) in settings {
                if name != "all" && !name.starts_with("group_") {
                    return Err(format!("{name}: settings are by group name (group_A) or \"all\""));
                }
                let footwork = match value {
                    Value::String(s) if s == "raw" => super::shared::Footwork::raw(),
                    Value::String(s) if s == "on" => super::shared::Footwork::default(),
                    Value::String(s) if s == "rove" => return Err(format!("{name}: roving is your hands' `rove` move (or `rove_1` for one soldier), picked from the packet's words, not a lane setting; the way back is a pick too: name the group's new job in the packet")),
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
                lane.insert(name.clone(), footwork);
            }
            Ok(format!("footwork set; {}", said.join("; ")))
        }
        "mark" => {
            let marks = arguments.as_object().filter(|o| !o.is_empty()).ok_or("mark takes an object: place name to [x, z], a grid cell such as \"E7\", or null to forget it")?;
            let map = shared.map.lock().unwrap().clone();
            let (width, height) = (map["width"].as_f64().unwrap_or(0.0) as f32, map["height"].as_f64().unwrap_or(0.0) as f32);
            let mut parsed: Vec<(String, Option<Mark>)> = Vec::new();
            for (name, value) in marks {
                let word = !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') && name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
                let taken = name == "home" || name == "all" || ["spot_", "passage_", "group_"].iter().any(|p| name.starts_with(p));
                if !word || taken {
                    return Err(format!("{name}: a mark's name is lower-case words with underscores, and home, spot_N, passage_N and group_N are taken"));
                }
                let mark = match value {
                    Value::Null => None,
                    Value::String(null) if null.trim() == "null" => None,
                    Value::String(range) if range.contains('-') => Some(region_of(range, width, height, name)?),
                    other => Some(point_of(other, width, height, name).map(|(x, z)| Mark { x, z, cells: None })?),
                };
                parsed.push((name.clone(), mark));
            }
            let mut marks = shared.marks.lock().unwrap();
            let mut said: Vec<String> = Vec::new();
            for (name, mark) in parsed {
                match mark {
                    Some(m) => {
                        said.push(match m.cells {
                            Some(cells) => format!("{name}: the cells {} (its point the centre, ({:.0}, {:.0}))", super::shared::cells_words(cells), m.x, m.z),
                            None => format!("{name} at ({:.0}, {:.0})", m.x, m.z),
                        });
                        marks.insert(name, m);
                    }
                    None => {
                        marks.remove(&name);
                        said.push(format!("{name} forgotten"));
                    }
                }
            }
            Ok(format!("marked: {}; your hands see the place from their next look", said.join("; ")))
        }
        "signal" => signal(arguments, shared),
        "surrender" => surrender(arguments, shared),
        "say" => {
            let text = arguments["text"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("say needs text")?;
            let text: String = text.chars().take(240).collect();
            shared.chat_out.lock().unwrap().push(text.clone());
            Ok(format!("said: {text}"))
        }
        "produce" => {
            let lists = arguments.as_object().filter(|o| !o.is_empty()).ok_or("produce takes an object: factory name (lab_N or plant_N) or \"all\" to a list of unit names, or to {\"units\": [...], \"group\": \"group_A\" | \"new\" | \"by_type\", \"join_in\": N} for a factory")?;
            // The seat's roster, and what its builders and factories make now: a factory or constructor of another
            // faction, given by an ally, makes its own faction's units.
            let field = shared.field();
            let mut known: Vec<String> = field.roster.iter().map(|(name, _)| name.clone()).collect();
            known.extend(field.buildable.iter().map(|(name, _)| name.clone()));
            let groups: Vec<String> = shared.hands_merged(false).picture["actors"].as_object().map(|a| a.keys().filter(|k| k.starts_with("group_")).cloned().collect()).unwrap_or_default();
            let check_units = |name: &str, items: &[Value]| -> Result<Vec<String>, String> {
                let units: Vec<String> = items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{name}: unit names are strings"))).collect::<Result<_, _>>()?;
                // "corck:1": at most one of it from now, then the rest of the list (human-7: told "one
                // constructor first, then raiders", the hands, who cannot count, made three).
                for unit in &units {
                    let (unit_name, cap) = crate::brain::pianist::allowance(unit);
                    if !known.is_empty() && !known.contains(&unit_name.to_string()) && !crate::brain::roster::ROLE_WORDS.contains(&unit_name) {
                        return Err(format!("{unit_name} is not a unit of our roster (internal names as the roster lists them, or a role word: {})", crate::brain::roster::ROLE_WORDS.join(", ")));
                    }
                    if unit.contains(':') && cap.is_none() {
                        return Err(format!("{unit}: a count after the colon is a whole number of one or more (corck:1)"));
                    }
                }
                Ok(units)
            };
            // (actor, units or None to clear, the group its soldiers join: Some(Some(name)) sets, Some(None) clears, None
            // leaves; how many walk to it together, likewise)
            let mut parsed: Vec<(String, Option<Option<Vec<String>>>, Option<Option<String>>, Option<Option<usize>>)> = Vec::new();
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
                let name = &shared.actor_name(name);
                let factory = ["lab_", "plant_", "factory_"].iter().any(|p| name.starts_with(p));
                // `commander_tN` beside other seats of ours: the bare word alone was let through, and a seat's tagged
                // commander could be given no allowance.
                if !matches!(name.as_str(), "all" | "all_builders") && !name.starts_with("commander") && !factory && !name.starts_with("constructor_") {
                    return Err(format!("{name}: lists are by actor name (lab_N, plant_N, factory_N, commander, constructor_N), \"all_builders\" or \"all\""));
                }
                // A factory the picture does not name gets nothing (models-medium-gpt56-terra: five lists for "lab_1",
                // a name the player made up, accepted and never reaching a lab).
                if factory && !factories.contains(name) {
                    return Err(format!("{name}: not a factory in the picture ({}); use its name as the report gives it, or \"all\"", if factories.is_empty() { "none named yet".to_string() } else { factories.join(", ") }));
                }
                // A list written as a JSON string, as `queue` takes it (models-medium-sonnet5: eight refusals), and the
                // object form or a null likewise (human-21 13:57: the object as a string was cut at its commas and
                // refused as unit names).
                let unpacked;
                let value = match value {
                    Value::String(s) => {
                        unpacked = serde_json::from_str::<Value>(s).ok().filter(|v| v.is_array() || v.is_object() || v.is_null()).unwrap_or_else(|| Value::Array(s.split(',').map(|u| Value::String(u.trim().to_string())).filter(|u| u != "").collect()));
                        &unpacked
                    }
                    other => other,
                };
                match value {
                    Value::Null => parsed.push((name.clone(), Some(None), None, None)),
                    Value::Array(items) => parsed.push((name.clone(), Some(Some(check_units(name, items)?)), None, None)),
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
                            Some(Value::String(g)) if g == "new" || g == "by_type" => Some(Some(g.clone())),
                            Some(Value::String(g)) if groups.contains(g) => Some(Some(g.clone())),
                            Some(Value::String(g)) => return Err(format!("{name}: {g} is not a group in the picture ({}); \"new\" makes one, \"by_type\" one per unit type", if groups.is_empty() { "none yet".to_string() } else { groups.join(", ") })),
                            Some(_) => return Err(format!("{name}: group is a group name, \"new\" or \"by_type\"")),
                        };
                        let join_in = match fields.get("join_in") {
                            None => None,
                            Some(Value::Null) => Some(None),
                            Some(v) => match v.as_u64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())) {
                                Some(n) if (2..=16).contains(&n) => Some(Some(n as usize)),
                                _ => return Err(format!("{name}: join_in is a whole number of 2 to 16 (how many new soldiers walk to their group together), or null")),
                            },
                        };
                        if units.is_none() && group.is_none() && join_in.is_none() {
                            return Err(format!("{name}: the object form takes units, group and/or join_in"));
                        }
                        parsed.push((name.clone(), units, group, join_in));
                    }
                    _ => return Err(format!("{name}: a list of unit names, null, or {{\"units\": [...], \"group\": ..., \"join_in\": N}}")),
                }
            }
            let mut allowed = shared.allowed.lock().unwrap();
            let call = shared.produce_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let mut said: Vec<String> = Vec::new();
            for (name, units, group, join_in) in parsed {
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
                        said.push(match g.as_str() {
                            "new" => format!("{name}'s new soldiers form a group of their own from now"),
                            "by_type" => format!("{name}'s new soldiers form one group per unit type from now (the report names each as it forms)"),
                            _ => format!("{name}'s new soldiers join {g}"),
                        });
                        Some(g)
                    }
                    Some(None) => {
                        said.push(format!("{name}'s new soldiers gather in its own group again"));
                        None
                    }
                    None => before.as_ref().and_then(|a| a.group.clone()),
                };
                let join_in_now = match join_in {
                    Some(Some(n)) => {
                        said.push(format!("{name}'s new soldiers wait beside it and walk to their group {n} together"));
                        Some(n)
                    }
                    Some(None) => {
                        said.push(format!("{name}'s new soldiers walk to their group as each is made"));
                        None
                    }
                    None => before.as_ref().and_then(|a| a.join_in),
                };
                match (units_now, group_now, join_in_now) {
                    (None, None, None) => {
                        allowed.remove(&name);
                    }
                    (units, group, join_in) => {
                        allowed.insert(name, Allowance { call: if units.is_some() { call } else { before.map_or(call, |a| a.call) }, units: units.unwrap_or_default(), group, join_in });
                    }
                }
            }
            Ok(format!("{}; they see it from their next look", said.join("; ")))
        }
        "transfer" => {
            use super::shared::Transfer;
            // Under a commander the side's state knows every seat (a player with one seat knows its own alone),
            // and every seat's brain takes its transfers from there.
            let shared = shared.side.get().unwrap_or(shared);
            let team_of = |v: &Value| -> Result<i32, String> {
                let text = v.as_str().ok_or("from and to are seat tags such as \"t2\"")?;
                text.trim().trim_start_matches('t').parse::<i32>().map_err(|_| format!("{text}: a seat tag is t<team>, as the seats line writes it"))
            };
            // Lifting a standing transfer names the sender alone.
            let lift = matches!(&arguments["standing"], Value::Bool(false)) || arguments["standing"].as_str().is_some_and(|s| matches!(s.trim(), "off" | "stop" | "no"));
            if lift {
                let from = team_of(&arguments["from"])?;
                return match shared.standing.lock().unwrap().remove(&from) {
                    Some(s) => Ok(format!("t{from}'s standing transfer to t{} is lifted; it sent {:.0} metal and {:.0} energy since {}", s.to_team, s.sent_metal, s.sent_energy, crate::brain::pianist::clock(s.since))),
                    None => Err(format!("t{from} has no standing transfer")),
                };
            }
            let to = team_of(&arguments["to"])?;
            let live = shared.live_seats();
            // An allied seat that is not ours takes metal and energy (H-TRANSFER-TO-ALLIES; the user, 2026-10-18: a
            // sprinting ally is fed the same way), never units: its player is not ours to hand them to.
            let allied = shared.briefing().allied_teams;
            let to_ally = !live.contains(&to) && allied.contains(&to);
            if !live.contains(&to) && !to_ally {
                let tags = |teams: &[i32]| teams.iter().map(|t| format!("t{t}")).collect::<Vec<_>>().join(", ");
                return Err(format!("t{to} is not a seat of our side in this game (ours: {}; allies: {})", tags(&live), if allied.is_empty() { "none".to_string() } else { tags(&allied) }));
            }
            let metal = arguments["metal"].as_f64().unwrap_or(0.0) as f32;
            let energy = arguments["energy"].as_f64().unwrap_or(0.0) as f32;
            let units: Vec<String> = arguments["units"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.trim().to_string())).collect()).unwrap_or_default();
            // A standing transfer (H-PLAYER-STANDING-TRANSFER): `keep_metal` and/or `keep_energy` with from and to,
            // kept by the hands every second until `standing: false` (or "off") with from lifts it.
            let keep_metal = arguments["keep_metal"].as_f64().map(|v| v.max(0.0) as f32);
            let keep_energy = arguments["keep_energy"].as_f64().map(|v| v.max(0.0) as f32);
            if keep_metal.is_some() || keep_energy.is_some() {
                let from = team_of(&arguments["from"])?;
                if from == to {
                    return Err("from and to are the same seat".into());
                }
                if !live.contains(&from) {
                    return Err(format!("t{from} is not a seat of ours in this game"));
                }
                let frame = shared.briefing().frame;
                let mut standing = shared.standing.lock().unwrap();
                let before = standing.get(&from).map(|s| (s.sent_metal, s.sent_energy, s.since)).unwrap_or((0.0, 0.0, frame));
                standing.insert(from, super::shared::Standing { to_team: to, keep_metal, keep_energy, since: before.2, sent_metal: before.0, sent_energy: before.1, minute_metal: 0.0, minute_energy: 0.0, minute_from: frame });
                let what: Vec<String> = [keep_metal.map(|k| format!("every metal above {k:.0} in its store")), keep_energy.map(|k| format!("every energy above {k:.0}"))].into_iter().flatten().collect();
                return Ok(format!("standing: t{from} sends t{to} {} every second from now until you lift it ({{\"from\": \"t{from}\", \"standing\": false}}); its report and yours say what went. What t{to}'s store cannot hold stays with t{from}, so t{to} spends or stores as it comes.", what.join(" and ")));
            }
            if metal <= 0.0 && energy <= 0.0 && units.is_empty() {
                return Err("transfer takes metal and/or energy with from and to, or units with to; or keep_metal / keep_energy with from and to for a standing transfer".into());
            }
            if to_ally && !units.is_empty() {
                return Err(format!("t{to} is an allied seat, not ours: metal and energy may go to it, units only to a seat of ours"));
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
            // "<type> at <place or cell>" (armwin at spot_64, armsolar at E4): the hands take apart the nearest
            // standing unit of that type to the place. For a reclaim only: a destruct's blast is checked per unit,
            // so it takes the exact handle.
            let places: Vec<String> = shared.hands_merged(false).picture["places"].as_object().map(|p| p.keys().cloned().collect()).unwrap_or_default();
            let at_form = |h: &str| -> Result<(String, String), String> {
                let h = h.strip_prefix("unfinished ").unwrap_or(h);
                let (kind, at) = h.split_once(" at ").ok_or_else(|| format!("{h}: nothing of ours has that name (handles are <unit>_<id> as the picture writes them, e.g. armsolar_31002, an actor name, or \"<type> at <place or cell>\" such as \"armwin at spot_64\")"))?;
                let (kind, at) = (kind.trim(), at.trim());
                if !cards.iter().any(|c| c.unit == kind) {
                    return Err(format!("{h}: no {kind} of ours stands (internal unit names, as the picture writes them)"));
                }
                let cell = at.len() == 2 && at.starts_with(|c: char| c.is_ascii_alphabetic()) && at.ends_with(|c: char| c.is_ascii_digit());
                if !cell && !places.contains(&at.to_string()) {
                    return Err(format!("{h}: {at} is not a place in the picture nor a grid cell (A1 to H8)"));
                }
                Ok((kind.to_string(), at.to_string()))
            };
            for h in &reclaim {
                if find(h).is_none() {
                    at_form(h)?;
                }
            }
            for h in &destruct {
                find(h).ok_or_else(|| format!("{h}: nothing of ours has that name (handles are <unit>_<id> as the picture writes them, e.g. armsolar_31002, or an actor name; a destruct takes the exact handle, since its blast is checked per unit)"))?;
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
                match find(h) {
                    Some(c) => said.push(format!("{h} ({}, {:.0} metal) is taken apart by {}; most of its metal comes back", c.unit, c.metal, by.as_deref().unwrap_or("the nearest builder without a list"))),
                    None => {
                        let (kind, at) = at_form(h).expect("checked above");
                        said.push(format!("the nearest standing {kind} to {at} is taken apart by {}; the hands name it in their report", by.as_deref().unwrap_or("the nearest builder without a list")));
                    }
                }
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
        "fire" => {
            let cards: Vec<super::shared::UnitCard> = shared.own_cards.lock().unwrap().values().flatten().cloned().collect();
            if cards.is_empty() {
                return Err("your hands have not published our units yet: try again in a few seconds".into());
            }
            let handle = arguments["silo"].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("fire takes {\"silo\": \"armsilo_N\", \"at\": [x, z] or a grid cell}")?;
            let card = cards.iter().find(|c| c.handle == handle).ok_or_else(|| format!("{handle}: nothing of ours has that name (a silo's handle is as the picture's `stockpilers` entry writes it, e.g. armsilo_31002)"))?;
            if card.stockpile_seconds <= 0.0 {
                return Err(format!("{handle} is a {}, not a silo: it has no missile to fire", card.unit));
            }
            // Firings ordered this turn and not yet taken by the pianist still count against the card's rounds: two calls
            // in one batch on a silo with one missile fire one (nuke-2-hard, 2026-10-09: both answered "2 left").
            let ordered = shared.firings.lock().unwrap().iter().filter(|f| f.silo == handle).count() as i32;
            let ready = card.stockpile - ordered;
            if ready < 1 {
                return Err(if ordered > 0 {
                    format!("{handle} has no missile left to fire: {ordered} already ordered this turn against its {} ready; nothing fired", card.stockpile)
                } else {
                    format!("{handle} has no missile ready ({} stockpiling, one every {:.0} s at full energy); nothing fired", card.stockpile_queued, card.stockpile_seconds)
                });
            }
            let map = shared.map.lock().unwrap().clone();
            let (width, height) = (map["width"].as_f64().unwrap_or(0.0) as f32, map["height"].as_f64().unwrap_or(0.0) as f32);
            let at = point_of(&arguments["at"], width, height, "at")?;
            shared.firings.lock().unwrap().push(Firing { silo: handle.to_string(), at });
            Ok(format!("{handle} fires a missile at ({:.0}, {:.0}) from your hands' next look: it launches in about 9 s and lands 13 s later, 12,500 damage at the point and nothing 640 away; {} missile{} left ready after it", at.0, at.1, ready - 1, if ready - 1 == 1 { "" } else { "s" }))
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
        let card = |handle: &str, actor: Option<&str>, at: (f32, f32), health: f32, blast: Option<(f32, f32)>| UnitCard { handle: handle.into(), actor: actor.map(str::to_string), unit: handle.split('_').next().unwrap().into(), at, health, metal: 155.0, self_destruct: blast, self_destruct_seconds: 5.0, ..UnitCard::default() };
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

    /// The first batch of the impedance study's "no lever" asks (2026-10-16): `produce` groups a factory's output
    /// per unit type (`by_type`) and has its newcomers walk to their group so many together (`join_in`); `queue`
    /// takes a `reclaim` step by handle, grid cell or place, and an id step on a builder that cannot make the type
    /// (it helps whoever builds it); `remove` takes "<type> at <place or cell>" for a reclaim, not for a destruct.
    #[test]
    fn produce_by_type_and_join_in_queue_reclaim_and_help_steps_and_remove_at_a_place() {
        use super::super::shared::{Field, Removal, UnitCard};
        use std::collections::BTreeMap;
        let shared = Arc::new(Shared::default());
        shared.hands.lock().unwrap().entry(0).or_default().picture = json!({ "actors": { "plant_7": {} }, "places": { "spot_64": { "grid": "E4" } } });
        let said = call_tool("produce", &json!({ "plant_7": { "units": ["armfav", "armflash"], "group": "by_type", "join_in": 4 } }), &shared).unwrap();
        assert!(said.contains("one group per unit type") && said.contains("walk to their group 4 together"), "{said}");
        let a = shared.allowed.lock().unwrap().get("plant_7").cloned().unwrap();
        assert_eq!((a.group.as_deref(), a.join_in, a.units.len()), (Some("by_type"), Some(4), 2));
        assert!(call_tool("produce", &json!({ "plant_7": { "join_in": 1 } }), &shared).unwrap_err().contains("2 to 16"));
        assert!(call_tool("produce", &json!({ "plant_7": { "join_in": null } }), &shared).unwrap().contains("as each is made"));
        let a = shared.allowed.lock().unwrap().get("plant_7").cloned().unwrap();
        assert_eq!((a.group.as_deref(), a.join_in, a.units.len()), (Some("by_type"), None, 2), "the rest of the allowance stands");
        // queue: reclaim steps, and an id step on a builder that cannot make the type.
        let builders = BTreeMap::from([("constructor_3".to_string(), BTreeMap::from([("armsolar".to_string(), false), ("extractor".to_string(), false)]))]);
        shared.publish_field(0, Field { roster: vec![("armsolar".into(), 155), ("armsilo".into(), 5000)], builders, ..Default::default() });
        assert!(call_tool("queue", &json!({ "constructor_3": ["reclaim E4", "reclaim armsolar_31002", "reclaim spot_40", "armsolar"] }), &shared).is_ok());
        assert!(call_tool("queue", &json!({ "constructor_3": ["reclaim"] }), &shared).unwrap_err().contains("reclaim takes"));
        assert!(call_tool("queue", &json!({ "constructor_3": ["reclaim E4 #x"] }), &shared).unwrap_err().contains("an id goes on a step that builds"));
        assert!(call_tool("queue", &json!({ "constructor_3": ["armsilo"] }), &shared).unwrap_err().contains("cannot build armsilo"));
        assert!(call_tool("queue", &json!({ "constructor_3": ["armsilo #silo1"] }), &shared).is_ok(), "with an id it helps whoever builds it");
        // remove: "<type> at <place or cell>" for a reclaim.
        let turbine = UnitCard { handle: "armwin_5".into(), unit: "armwin".into(), at: (4500.0, 2600.0), health: 300.0, metal: 40.0, ..UnitCard::default() };
        shared.own_cards.lock().unwrap().insert(0, vec![turbine]);
        let said = call_tool("remove", &json!({ "reclaim": ["armwin at spot_64"] }), &shared).unwrap();
        assert!(said.contains("the nearest standing armwin to spot_64 is taken apart"), "{said}");
        assert!(call_tool("remove", &json!({ "reclaim": ["armwin at E4"] }), &shared).is_ok());
        assert!(call_tool("remove", &json!({ "reclaim": ["armsolar at E4"] }), &shared).unwrap_err().contains("no armsolar of ours stands"));
        assert!(call_tool("remove", &json!({ "reclaim": ["armwin at spot_9"] }), &shared).unwrap_err().contains("not a place in the picture"));
        assert!(call_tool("remove", &json!({ "destruct": ["armwin at E4"] }), &shared).unwrap_err().contains("exact handle"));
        let removals = shared.removals.lock().unwrap().clone();
        assert!(matches!(&removals[0], Removal::Reclaim { targets, .. } if targets == &["armwin at spot_64".to_string()]), "{removals:?}");
    }

    /// The `fire` tool: a silo with a missile ready fires at a point or a grid cell's centre and the firing waits for the
    /// pianist; one still stockpiling is refused, as is anything that is not a silo, and a point off the map. Firings
    /// ordered in the same turn count against the rounds: the third call on a silo with two is refused.
    #[test]
    fn fire_sends_a_ready_missile_at_a_point_and_refuses_an_empty_silo() {
        use super::super::shared::UnitCard;
        let shared = Arc::new(Shared::default());
        *shared.map.lock().unwrap() = json!({ "width": 8192.0, "height": 6144.0 });
        let silo = |handle: &str, ready: i32, queued: i32| UnitCard { handle: handle.into(), unit: "armsilo".into(), at: (500.0, 500.0), health: 5000.0, metal: 8100.0, stockpile: ready, stockpile_queued: queued, stockpile_seconds: 120.0, ..UnitCard::default() };
        let solar = UnitCard { handle: "armsolar_7".into(), unit: "armsolar".into(), at: (100.0, 100.0), health: 400.0, metal: 155.0, ..UnitCard::default() };
        shared.own_cards.lock().unwrap().insert(0, vec![silo("armsilo_12", 2, 8), silo("armsilo_13", 0, 10), solar]);
        assert!(call_tool("fire", &json!({ "silo": "armsilo_13", "at": [6600, 3072] }), &shared).unwrap_err().contains("no missile ready"));
        assert!(call_tool("fire", &json!({ "silo": "armsolar_7", "at": [6600, 3072] }), &shared).unwrap_err().contains("not a silo"));
        assert!(call_tool("fire", &json!({ "silo": "armsilo_12", "at": [9000, 3072] }), &shared).unwrap_err().contains("off the map"));
        assert!(shared.firings.lock().unwrap().is_empty(), "a refused firing orders nothing");
        let said = call_tool("fire", &json!({ "silo": "armsilo_12", "at": [6600, 3072] }), &shared).unwrap();
        assert!(said.contains("fires a missile at (6600, 3072)") && said.contains("1 missile left ready"), "{said}");
        let said = call_tool("fire", &json!({ "silo": "armsilo_12", "at": "E7" }), &shared).unwrap();
        assert!(said.contains("0 missiles left ready"), "{said}");
        let refused = call_tool("fire", &json!({ "silo": "armsilo_12", "at": "E7" }), &shared).unwrap_err();
        assert!(refused.contains("2 already ordered this turn"), "{refused}");
        let firings = shared.firings.lock().unwrap().clone();
        assert_eq!(firings.len(), 2);
        assert_eq!((firings[0].silo.as_str(), firings[0].at), ("armsilo_12", (6600.0, 3072.0)));
        assert_eq!(firings[1].at, (4.0 * 1024.0 + 512.0, 6.0 * 768.0 + 384.0), "E7 is its cell's centre");
    }

    /// `transfer` reaches an allied seat that is not ours with metal and energy, never with units (H-TRANSFER-TO-ALLIES).
    #[test]
    fn a_transfer_feeds_an_allied_seat_but_gives_it_no_units() {
        let shared = Arc::new(Shared::default());
        shared.publish_briefing(5, Default::default(), crate::strategist::shared::Briefing { allied_teams: vec![6, 7], ..Default::default() });
        let said = call_tool("transfer", &json!({ "metal": 300, "from": "t5", "to": "t7" }), &shared).unwrap();
        assert!(said.contains("t5 sends 300 metal and 0 energy to t7"), "{said}");
        let err = call_tool("transfer", &json!({ "units": ["group_A"], "to": "t7" }), &shared).unwrap_err();
        assert!(err.contains("allied seat, not ours"), "{err}");
        // A standing transfer (H-PLAYER-STANDING-TRANSFER): set with a keep, lifted by the sender alone.
        let standing = call_tool("transfer", &json!({ "from": "t5", "to": "t7", "keep_metal": 100 }), &shared).unwrap();
        assert!(standing.contains("standing: t5 sends t7 every metal above 100"), "{standing}");
        assert_eq!(shared.standing.lock().unwrap().get(&5).map(|s| (s.to_team, s.keep_metal, s.keep_energy)), Some((7, Some(100.0), None)));
        assert!(call_tool("transfer", &json!({ "from": "t5", "standing": false }), &shared).unwrap().contains("lifted"));
        assert!(call_tool("transfer", &json!({ "from": "t5", "standing": false }), &shared).unwrap_err().contains("no standing transfer"));
        let err = call_tool("transfer", &json!({ "metal": 300, "from": "t5", "to": "t2" }), &shared).unwrap_err();
        assert!(err.contains("not a seat of our side") && err.contains("allies: t6, t7"), "{err}");
    }

    #[test]
    fn the_player_has_its_lever_and_none_of_the_commanders() {
        let player: Vec<String> = tool_list().as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
        assert_eq!(player, ["overview", "map", "situation", "units", "plan", "search", "instruct", "queue", "lane", "mark", "produce", "transfer", "remove", "fire", "say", "signal", "surrender", "orders", "wait", "note", "predict"]);
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
        // The relative step of an opening written before the start is known: a bare `extractor`, or `extractor nearest`.
        assert!(call_tool("queue", &json!({ "commander": ["extractor", "extractor nearest", "armsolar", "armvp", "assist"] }), &shared).unwrap().contains("5 steps"));
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist", "armsolar"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist 2", "armsolar"] }), &shared).is_err());
        // With a roster published, a step must be a unit of it; without one any word passes and the bot skips it.
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armllt".into(), 85)], ..Default::default() });
        assert!(call_tool("queue", &json!({ "commander": ["turrett spot_3"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "commander": ["armllt spot_3", "armsolar"] }), &shared).is_ok());
        // A role word passes with or without a roster, and a factory's role word gets the note an untagged factory step gets.
        assert!(call_tool("queue", &json!({ "commander": ["turret spot_3", "solar", "lab"], "constructor_7": ["lab"] }), &shared).unwrap().contains("note: commander and constructor_7 each have 'lab' without an id"));
        assert!(call_tool("produce", &json!({ "all": ["constructor:1", "raider", "armsolar"] }), &shared).is_ok());
        // With the builders' words published, a step the builder cannot make and a placed building with no place are
        // refused now, not skipped by the hands later; a builder the field does not know is not checked.
        let builders = std::collections::BTreeMap::from([(
            "commander".to_string(),
            std::collections::BTreeMap::from([("armsolar".to_string(), false), ("armllt".to_string(), true), ("turret".to_string(), true), ("extractor".to_string(), false)]),
        )]);
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armllt".into(), 85), ("armnanotc".into(), 190)], builders, ..Default::default() });
        assert!(call_tool("queue", &json!({ "commander": ["armnanotc"] }), &shared).unwrap_err().contains("cannot build armnanotc"));
        assert!(call_tool("queue", &json!({ "commander": ["armllt"] }), &shared).unwrap_err().contains("stands at a place"));
        assert!(call_tool("queue", &json!({ "commander": ["turret"] }), &shared).unwrap_err().contains("stands at a place"));
        assert!(call_tool("queue", &json!({ "commander": ["armllt spot_3", "turret spot_4", "extractor", "extractor nearest", "armsolar low", "assist"] }), &shared).is_ok());
        assert!(call_tool("queue", &json!({ "constructor_9": ["armnanotc"] }), &shared).is_ok());
        // A constructor of another faction (given by an ally) takes its own faction's names: what it makes is a step
        // whatever the seat's roster says (Center Command 2v4: `cormoho` refused to the Armada seat's borrowed Cortex constructor).
        let borrowed = std::collections::BTreeMap::from([("constructor_14291".to_string(), std::collections::BTreeMap::from([("cormoho".to_string(), true), ("advanced_extractor".to_string(), true), ("corfus".to_string(), false)]))]);
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armmoho".into(), 620)], builders: borrowed, ..Default::default() });
        assert!(call_tool("queue", &json!({ "constructor_14291": ["cormoho spot_2", "advanced_extractor spot_4 start", "corfus home"] }), &shared).is_ok());
        assert!(call_tool("queue", &json!({ "constructor_14291": ["corgant"] }), &shared).is_err());
        assert!(call_tool("queue", &json!({ "constructor_14291": ["assist 30 start"] }), &shared).unwrap_err().contains("start"));
        // `signal` goes in an `orders` batch like the other levers (impedance tally 2026-10-16: 38 refusals in 23 games).
        assert!(batchable().contains(&"signal"));
        assert!(call_tool("produce", &json!({ "all": ["raiders"] }), &shared).unwrap_err().contains("role word"));
        // Every player tool but the readers and `orders` itself can be batched (comet-3: `queue` was refused by the
        // batch as "not a tool that can be batched" and the game ran without the list).
        for tool in ["instruct", "queue", "lane", "mark", "produce", "say", "note", "predict", "wait"] {
            assert!(batchable().contains(&tool), "{tool}");
        }
        assert!(orders(&json!({ "calls": [{ "tool": "queue", "arguments": { "commander": ["armsolar"] } }] }), &shared).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "group_A": ["armsolar"] }), &shared).is_err());
        // Ids: the last word of a building step; one id is one building, written the same on every list; the same
        // factory step on two lists without an id is said to be two factories, not refused.
        shared.publish_field(0, super::super::shared::Field { roster: vec![("armsolar".into(), 155), ("armllt".into(), 85), ("armvp".into(), 590)], ..Default::default() });
        let one = call_tool("queue", &json!({ "constructor_7": ["armvp home #vp2", "armsolar #s1"], "constructor_8": ["armvp home #vp2"] }), &shared).unwrap();
        assert!(!one.contains("note:"), "{one}");
        assert!(call_tool("queue", &json!({ "constructor_7": ["armvp home #vp2"], "constructor_8": ["armsolar #vp2"] }), &shared).unwrap_err().contains("#vp2 names two buildings"));
        assert!(call_tool("queue", &json!({ "commander": ["armvp", "assist 20 #a"] }), &shared).unwrap_err().contains("an id goes on a step that builds something"));
        assert!(call_tool("queue", &json!({ "commander": ["armvp #vp2 home"] }), &shared).unwrap_err().contains("an id is one word after everything else"));
        assert!(call_tool("queue", &json!({ "commander": ["armvp #vp-2"] }), &shared).unwrap_err().contains("letters, digits and underscores"));
        let two = call_tool("queue", &json!({ "constructor_7": ["armvp home"], "constructor_8": ["armvp home"] }), &shared).unwrap();
        assert!(two.contains("note: constructor_7 and constructor_8 each have 'armvp home' without an id: that is 2 factories"), "{two}");
        assert!(call_tool("queue", &json!({ "commander": ["windmill"] }), &shared).is_err());
        // The shapes the model actually sent to cancel or replace a list (comet-5).
        for form in [json!({ "commander": "null" }), json!({ "commander": [] }), json!({ "commander": null })] {
            assert!(call_tool("queue", &form, &shared).unwrap().contains("cancelled"), "{form}");
            assert!(shared.queues.lock().unwrap().get("commander").cloned().flatten().is_none());
            let said = call_tool("queue", &json!({ "constructor_7": "stop", "commander": ["stop", "armsolar"] }), &shared).unwrap();
            assert!(said.contains("constructor_7: list cancelled and its build in progress dropped") && said.contains("commander: its build in progress dropped (a frame begun after the report you read finishes first instead; the hands' line says which), then 1 steps"), "{said}");
            assert_eq!(shared.queues.lock().unwrap().get("constructor_7").cloned().flatten(), Some(vec!["stop".to_string()]));
            assert!(call_tool("queue", &json!({ "commander": ["armsolar", "stop"] }), &shared).is_err());
        }
        assert!(call_tool("queue", &json!({ "commander": "[\"assist\"]" }), &shared).unwrap().contains("1 steps"));
        assert!(call_tool("queue", &json!({ "commander": "assist" }), &shared).unwrap().contains("1 steps"));
    }

    /// A player with one seat among several of ours writes `commander` and means its own (human-19).
    #[test]
    fn a_single_seats_player_may_call_its_commander_by_the_bare_word() {
        let shared = Arc::new(Shared::default());
        *shared.own_commander.lock().unwrap() = Some("commander_t2".into());
        call_tool("queue", &json!({ "commander": ["extractor", "solar"] }), &shared).unwrap();
        assert_eq!(shared.queues.lock().unwrap().keys().cloned().collect::<Vec<_>>(), ["commander_t2"]);
        call_tool("queue", &json!({ "commander_t2": ["solar"] }), &shared).unwrap();
        assert_eq!(shared.queues.lock().unwrap().len(), 1);
        assert!(call_tool("queue", &json!({ "commander_t1": ["solar"] }), &shared).unwrap_err().contains("your seat's commander is commander_t2"));
    }

    #[test]
    fn the_lane_tool_sets_footwork_by_group() {
        use super::super::shared::Footwork;
        let shared = Arc::new(Shared::default());
        assert!(call_tool("lane", &json!({ "group_A": "raw", "all": ["fan", "kite"] }), &shared).is_ok());
        let lane = shared.lane.lock().unwrap().clone();
        assert_eq!(lane["group_A"], Footwork::raw());
        assert_eq!(lane["all"], Footwork { flee: false, fan: true, kite: true });
        // `form` and `march` were rules until design D15 (2026-10-24 night): a group forms by definition now.
        assert!(call_tool("lane", &json!({ "group_A": ["form"] }), &shared).is_err());
        assert!(call_tool("lane", &json!({ "group_A": ["dance"] }), &shared).is_err());
        assert!(call_tool("lane", &json!({ "raiders": "raw" }), &shared).is_err());
        assert!(call_tool("lane", &json!({}), &shared).is_err());
        // "on" for a group is said explicitly.
        assert!(call_tool("lane", &json!({ "group_A": "on" }), &shared).is_ok());
        assert_eq!(shared.lane.lock().unwrap()["group_A"], Footwork::default());
        // Roving is the hands' move, never a setting (2026-10-07), and the way back is a pick (2026-10-08).
        assert!(call_tool("lane", &json!({ "group_B": "rove" }), &shared).unwrap_err().contains("`rove` move"));
        assert!(!shared.lane.lock().unwrap().contains_key("group_B"));
    }

    #[test]
    fn marks_are_named_places_by_coordinates_or_cell() {
        let shared = Arc::new(Shared::default());
        *shared.map.lock().unwrap() = json!({ "width": 8000.0, "height": 8000.0 });
        assert!(call_tool("mark", &json!({ "south_gate": [3600, 5400], "far_east": "H4" }), &shared).is_ok());
        let marks = shared.marks.lock().unwrap().clone();
        assert_eq!((marks["south_gate"].x, marks["south_gate"].z, marks["south_gate"].cells), (3600.0, 5400.0, None));
        assert_eq!((marks["far_east"].x, marks["far_east"].z), (7500.0, 3500.0));
        // A region: two cells with a dash, either way round; its point is the centre.
        let said = call_tool("mark", &json!({ "east_half": "E1-H8", "his_corner": "H8-G7" }), &shared).unwrap();
        assert!(said.contains("east_half: the cells E1-H8 (its point the centre, (6000, 4000))") && said.contains("his_corner: the cells G7-H8"), "{said}");
        let marks = shared.marks.lock().unwrap().clone();
        assert_eq!(marks["east_half"].cells, Some((4, 0, 7, 7)));
        assert_eq!((marks["his_corner"].x, marks["his_corner"].z, marks["his_corner"].cells), (7000.0, 7000.0, Some((6, 6, 7, 7))));
        assert!(call_tool("mark", &json!({ "bad": "E1-Z9" }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "spot_3": [1.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "Gate": [1.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "x": [9000.0, 1.0] }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "x": "Z9" }), &shared).is_err());
        assert!(call_tool("mark", &json!({ "south_gate": null }), &shared).is_ok());
        assert!(!shared.marks.lock().unwrap().contains_key("south_gate"));
        // null written in a string forgets a mark and a signal alike (signal-rehearsal 3:20, 4:50).
        assert!(call_tool("mark", &json!({ "far_east": "null" }), &shared).is_ok());
        assert!(!shared.marks.lock().unwrap().contains_key("far_east"));
        assert!(call_tool("signal", &json!({ "go": "6:00" }), &shared).is_ok());
        assert!(call_tool("signal", &json!({ "go": "null" }), &shared).is_ok());
        assert!(shared.signals.lock().unwrap().is_empty());
    }

    /// The object form written as a JSON string is the object (human-21 13:57: it was cut at its commas and
    /// refused as unit names).
    #[test]
    fn produce_takes_its_object_form_written_in_a_string() {
        let shared = Arc::new(Shared::default());
        shared.hands.lock().unwrap().entry(0).or_default().picture = json!({ "actors": { "lab_7": {} } });
        call_tool("produce", &json!({ "lab_7": "{\"units\": [\"armpw\"], \"group\": \"new\"}" }), &shared).unwrap();
        assert_eq!(shared.allowed.lock().unwrap()["lab_7"].units, vec!["armpw".to_string()]);
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
                let palette = Palette::roster(units, ctx.game.commander, ctx.turret, ctx.water, &[]);
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
                let mut words = crate::brain::pianist::step_id(&text).0.split_whitespace();
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
                    "help" => Item::Help,
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
            Item::Help => return "help".into(),
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

#[cfg(test)]
mod predict_tests {
    use super::*;

    /// A `predict` row carries the numbers, the game time and what the report showed, lands in the predictions file and
    /// in the takeover words; a missing or odd field is refused with the field named.
    #[test]
    fn a_prediction_is_written_with_what_the_report_showed_and_bad_fields_are_refused() {
        let shared = Arc::new(Shared::default());
        let path = std::env::temp_dir().join(format!("within-reason-predict-test-{}.jsonl", std::process::id()));
        *shared.predictions.lock().unwrap() = Some(std::fs::OpenOptions::new().create(true).append(true).truncate(false).open(&path).unwrap());
        let ok = predict(&json!({ "extractors": 17, "army_metal": 4200, "advanced_factory": "building", "basis": "a Hound seen at E3 and the brief's 10:44" }), &shared).unwrap();
        assert!(ok.contains("his extractors 17, his army 4200 metal, his advanced factory building"), "{ok}");
        let row: Value = serde_json::from_str(std::fs::read_to_string(&path).unwrap().lines().next().unwrap()).unwrap();
        assert_eq!(row["extractors"], 17);
        assert_eq!(row["advanced_factory"], "building");
        assert!(row["report"]["enemy_soldiers_seen"].is_number() && row["report"]["ours"]["extractors"].is_number(), "{row}");
        assert!(shared.last_prediction.lock().unwrap().as_ref().unwrap().contains("his army 4200 metal"));
        assert!(predict(&json!({ "extractors": 17, "army_metal": 4200, "advanced_factory": "maybe", "basis": "x" }), &shared).unwrap_err().contains("advanced_factory"));
        assert!(predict(&json!({ "extractors": "many", "army_metal": 4200, "advanced_factory": "none", "basis": "x" }), &shared).unwrap_err().contains("extractors"));
        assert!(predict(&json!({ "extractors": 1, "army_metal": 1, "advanced_factory": "none", "basis": " " }), &shared).unwrap_err().contains("basis"));
        std::fs::remove_file(&path).ok();
    }
}
