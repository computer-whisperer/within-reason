# Match record — format version 1

One file per AI per match, `record-<ai_id>.jsonl`, written by the bot process (`crates/bot/src/recorder.rs`) into
`$WITHIN_REASON_LOG_DIR` when `WITHIN_REASON_RECORD=1`; the arena always sets it. It holds what the bot saw and what it
decided, so a finished (or killed) match can be replayed in `viewer/` with `run/view_match.py <match dir>`.

It is not the engine's replay. The engine's own `demos/*.sdfz` re-simulates the game inside the engine, cannot be read by
a web page and knows nothing of our decisions; the record names it (`result.replay`) so the real thing can be opened too.

## Rules of the format
- JSON Lines, UTF-8, one record per line, `t` names the record type, `f` is the game frame (30 per second).
- Append-only, written once per tick (every `tick_frames` frames: 15 before 2026-09-20 evening, 3 since) in a single write. A killed match is valid up to its last tick;
  readers must skip a last line that does not parse and must not require a `result` line.
- Records are in frame order, except that a `rules` decision record covers the sample interval that ends at its frame.
- The first line is the header. A second header later in the file means the bot process was restarted mid-match
  (`first_tick_frame` says when); the tables are the same, the brain's memory is not.
- Readers skip record types and fields they do not know. Adding either does not bump `version`; changing the meaning or
  layout of an existing one does.
- Positions are whole elmos `x, z` (west to east, north to south). Unit types are indices into the header's `unit_defs`,
  `-1` for a radar contact never identified.

## Records
| `t` | When | Fields |
|---|---|---|
| `header` | first tick | `format` ("within-reason-record"), `version`, `ai_id`, `team`, `ally_team`, `side` (first three letters of our first unit's name: `arm`, `cor`), `mode` (`heuristic`, `strategist`, `commander`, `pianist`, `player`: the pianist with an Opus player over it), `start_frame`, `seats` [{`team`, `ally_team`, `side`, `controller`: "Unknown" | {`Person`: {name, skill}} | {`Ai`: {name, short_name, version, profile}}}] (who plays every seat, from the start script; records from 2026-09-23 on), `first_tick_frame`, `frames_per_second`, `sample_frames`, `tick_frames` (frames between ticks; absent before 2026-09-20 evening, when it was 15), `wall_start` (unix s), `map` {`name`, `width`, `height`, `wind_min`, `wind_max`}, `grid` {`columns`, `rows`} (the A-H / 1-8 cells of `World::grid`), `metal_spots` [[x, z, amount]] (amount as the engine reports the spot; records before 2026-09-20 lack it; from 2026-09-23 the position is the metal-weighted centroid of the patch's squares, not the engine's spot point, H-ECO-SPOT-CENTROID; from 2026-09-24 the game's own list (`mex_count`/`mex_x<i>`/`mex_z<i>` rules params) when the game publishes one, which the replay cards use too, so their spot numbers match), `unit_defs` [{`id` (engine's), `name`, `class`, `metal`, `energy`, `speed`, `weapons`; and, in records from 2026-09-20 on, the numbers the build-order simulator reads: `build_time`, `build_speed`, `build_distance`, `builds` [engine ids], `extracts_metal`, `metal_make`, `energy_make`, `energy_upkeep` (negative for solar collectors), `wind_cap`, `metal_storage`, `energy_storage`, `radar_range` (records from 2026-09-20 evening on), `footprint` [xsize, zsize in 8-elmo squares], `death_blast` and `selfd_blast` [radius, damage] or null, `selfd_seconds` (records from 2026-09-22 late night on), `reach` and `reload` (the range and seconds between shots of the type's longest ordinary weapon, manual-fire weapons such as the D-gun left out; 0 unarmed; records from 2026-09-24 on), `converter` [energy/s taken, metal per energy] or null, `move` [kind as in `move_classes`, max_slope, depth, slope_mod (records from 2026-09-20 night on)] or null}], `siblings` {`bot_log`, `engine_log`, `replay` (glob), `decision_logs` [file names; the LLM transcript when a session runs]} |
| `s` | every `sample_frames` (30) | state sample. `m`, `e`: [current, income, usage, storage]. `own`: [[id, def, x, z, health %, flags]], flags 1 being built, 2 idle, 4 attacker (committed wave), 8 in a commander's squad. `en`: enemies in sight or radar [[id, def, x, z, health]] (absolute health; the protocol has no maximum for enemies). `al`: allied units (other seats of ours, other players on our side) [[id, def, x, z, team, being built]]; each seat of ours writes its own record, and any one of them shows the whole side through this. `dmg`: [[own unit id, damage taken since the last sample]]. `ms`: slowest `Brain::decide` since the last sample, wall milliseconds (in lockstep mode this includes the commander's thinking time). `late`: the most frames a tick of the interval was sent after it fell due, waiting for the bot's previous answer (always 0 in lockstep; absent before 2026-09-20 evening). `lane`: [claims ended, path walked while held, net displacement, reversals over ninety degrees] summed over the control lane's claims that ended since the last sample (the milling instrument, H-HANDS-LANE; absent when none ended). The fire instrument (records from 2026-09-24 on; `run/fire.py` reads it): `dealt`: [[own unit id, damage it dealt to enemies since the last sample]] (the engine's enemy-damaged event, sent to the attacker's own team only and only while the enemy is in sight or on radar, so a replay's spectator seat gets none); `shots`: [[own unit id, shots fired]] (the weapon-fired event); `ff`: [[own unit id, damage it did to our own side]] (friendly fire: the engine refuses a shot across a friend, `avoidFriendly`, but a shell already flying and any splash still land); `xo`: [[our type, enemy type, damage]] our hits on them by type pair; `xi`: [[enemy type, our type, damage]] their hits on us (-1 for a type never seen: fire from out of sight). Types are indices into `unit_defs`, as `d` `xf` (records from 2026-09-24 evening on): friendly fire by type pair [[shooter's type, victim's type, damage]], the victims the `ff` totals by shooter do not name. |
| `ev` | as they happen | engine event. `k`: `created` (+`by` builder id), `finished` (+`facing`: the engine's building facing, 0 south +z, 1 east, 2 north, 3 west; a factory's units leave through its front; records from 2026-09-22 late night on), `destroyed` (+`by` attacker id, `by_d` its type), `move_failed`, `enemy_seen`, `enemy_lost`, `enemy_destroyed`, `no_site` (+`what` def), `rejected` (+`code`), `chat` (+`player`, `text`: a line from a person in the game). `u` unit id, `d` its type, `x`, `z` its last known place (0, 0 when never seen). Not recorded singly: `UnitIdle` (it is a flag) and `UnitDamaged` (summed into `dmg`) |
| `cmd` | ticks with commands | `c`: [["build", unit, def, x, z] or ["build", unit, def] for a factory, ["move", unit, x, z], ["fight", unit, x, z], ["stop", unit], ["repeat", unit, 0 or 1], ["guard", unit, target unit], ["repair", unit, target unit], ["reclaim", unit, x, z, radius] (an area; records before 2026-09-20 only), ["reclaim_feature", unit, feature], ["resurrect", unit, feature], ["say", text] (the game chat: the bot's banner at its first orders), ["attack", unit, target unit] (a turret, deliberately, or the unit an `attack_unit` group was given), ["movestate", unit, 0 hold position / 1 manoeuvre / 2 roam] (air groups hold by it, docs/design/2026-09-22-domains.md); from the duel harness only: ["give", def, x, z], ["selfdestruct", unit], ["firestate", unit, 0 hold fire / 1 return fire / 2 fire at will]] |
| `intent` | when it changes | where the army stands in the brain's mind: `home`, `enemy_start` (presumed), `station`, `target`, `staging`; each [x, z] or null |
| `d` | as they happen | decision record, see below |
| `result` | appended by the arena after the match | `result` (the arena's `MatchResult`: outcome, side, corner, game minutes, wall seconds), `opponent`, `replay` (path relative to the match dir, or null) |

`class` is derived from the definition's numbers only, so it holds for every faction and for enemy types:
`commander`, `extractor`, `factory`, `turret`, `building`, `builder`, `army`, `other`; from 2026-09-22 (the domains
design) also `aircraft`, `air_builder` (mobile with no movement class), `hover` and `ship` (armed, by movement kind).

## Decision records
One shape for every decision source, so a new layer slots in without a format change:

`{"t":"d", "f":<frame>, "source":<who>, "kind":<what>, "inputs":<what it was decided on>, "outputs":<what was decided>, "latency_ms":<optional>}`

| `source` | `kind` | `inputs` / `outputs` |
|---|---|---|
| `heuristic` | `rules` | outputs: {heuristic ID (docs/heuristics.md): firings} over the sample interval ending at `f`. Same counts as the `rules:` line of `bot.log`, per second instead of per minute |
| `heuristic` | `wave` | inputs {`home_group`}; outputs {`wave`, `target` {grid, x, z}, `first_stop`} |
| `heuristic` | `contact` | H-ARMY-CONTACT, when the number answering a party changes. inputs {`party` (units), `metal`, `at` [x, z], `forced` (at the lab or the commander), `within_reach` (soldiers asked about), `burned_if_nobody` (metal, the simulator's)}; outputs {`sent`, `worth` (metal gained over sending nobody), `caught` (share of runs)} |
| `heuristic` | `scout` | H-SCOUT-ROUTE, a raider sent on a route. inputs {`unit`, `route` ([x, z] per spot)}; outputs {`squares` (grid names)} |
| `heuristic` | `recall` | inputs {`intruders`}; outputs {`attackers_called_home`} |
| `heuristic` | `assault` | inputs {`gathered`, `attackers`} |
| `heuristic` | `event` | outputs: the text the brain notes for the strategist ("lost armmex at C3", trigger texts) |
| `llm:strategist`, `llm:commander`, `llm:player` | `turn` | not in the record file: the viewer builds these from the sibling `strategist-<ai_id>.jsonl` (inputs {`wake`, `prompt`}; outputs {`calls` [{tool, arguments, result}], `said`, `thinking`}; `latency_ms` from `turn_end`) |
| `jev` | `builder`, `lab`, `group`, `global` | the pianist (`docs/design/2026-09-21-pianist.md`), one per menu answered. inputs {`actor` (its name in the picture: `commander`, `constructor_<id>`, `lab_<id>`, `group_<name>`), `options` (the menu's keys), `busy`}; outputs {`choice` (what Jev picked), `played` (what the hands did: `continue` when a busy actor's pick did not beat continue by the margin), `probability`, `confidence`, `where`, `where_extractor`, `where_scout`, `whom`, `how_many`, `did` (a sentence, or null)}. `global`: outputs {question id: probability of yes}. The whole call is in the pianist's log (below) |

The brain's side is `crates/bot/src/brain/journal.rs`: `self.journal.note(frame, kind, inputs, outputs)` at the decision,
`self.fire(...)` for rule counts. `main` drains the journal every tick whether or not a record is written. A Jev layer
should write through the same journal with its own `source` (`Note::source`; `Journal::note` is the heuristic brain's).

## The pianist's log
`jev-<ai_id>.jsonl` beside the record, written when the bot runs `--pianist` with `WITHIN_REASON_JEV_LOG=1` (the arena
sets it); the record header names it under `siblings.decision_logs`. JSON Lines; readers skip a torn last line.
- `{"t":"header","format":"within-reason-jev","version":1,"ai_id","model","interval_frames","rules"}` first: `rules` is
  the standing text every call's picture carried (it is left out of the calls).
- `{"t":"call","f","ms","model","usage":{input_tokens,...},"retries","state","questions","answers","played","groups","places","parties"}`
  per request: `state` is the picture without `instructions` and `rules`; `instructions` (the player's packet) is a
  field of the call only when it changed since the last logged call, so a reader carries it forward; `questions` and
  `answers` are the API's own shapes (`docs/harness/jev.md`); `played` is one entry per menu answered, the decision
  record's fields plus `kept` (a busy actor held its course); `groups` [{`name`, `members` [unit ids], `at` [x, z],
  `task` {`kind` hold/move_to/fight_to/engage, `place`, `to` [x, z]}}], `places` [{`name`, `x`, `z`, `spot`}] and
  `parties` [{`name`, `ids`, `x`, `z`, `metal`, `composition`}] are what the picture named, so a reader can draw them.
- `{"t":"error","f","error"}` for a call that failed (every actor kept its task).
- From 2026-09-24 late, a group's `do` is asked in two levels (H-HANDS-TWO-LEVEL): `two_level: true` on the call,
  `questions` still the flat layout as built, and `answers` the composed `<group>.do` beside the raw `<group>.kind`
  and `<group>.<family>` answers it was composed from.
Size: 15-30 KB a call (3-6k tokens of state and questions), 20-60 calls a minute: 25 MB for a 20-minute game. Logs from
the first morning (2026-09-21, before the header line) carry `instructions` and `rules` in every state and no `played`;
the viewer reads those too, taking the decisions from the answers.

## Size (measured 2026-09-19, Quicksilver, speed 50)
| Match | Game minutes | Units at the end (ours + seen) | File | Per game minute |
|---|---|---|---|---|
| viewer-test, easy | 14.2 | 88 | 1.5 MB | 108 KB |
| viewer-test-40, medium (loss) | 17.9 | 59 | 1.4 MB | 80 KB |
| viewer-test-40b, easy (loss) | 26.6 | 152 | 5.2 MB | 35 KB in minutes 0-5, 118 in 5-10, ~290 from minute 15 |

Samples are about 85 % of the file and grow with the unit count (about 20 bytes per unit per game second); events about
10 %, half of them `move_failed`. At the late-game rate a 40-minute match is 10-12 MB; gzip takes it to about a tenth.
The header is 60 KB (the unit table). Wall-clock cost was not measurable: 14 game minutes in 49 s with recording on.

## Ground truth about the opponent
The record holds the bot's fair view only. With `WITHIN_REASON_OBSERVE=1` the shim writes the once-a-minute census of
both sides into `engine.log` (see `arena.md`); the viewer reads those lines next to the record and draws them as faint
markers and as the "theirs" series. Census positions are the mean of all units of a type, so a marker for scattered
extractors sits between them.

## Viewer
`run/view_match.py run/matches/<batch>/<NN>` serves `viewer/` and the match directory and opens the browser. It listens on `::` (every interface, IPv6 and IPv4) so another machine on the network can open it; `--bind 127.0.0.1` keeps it to this machine.
Plain HTML, CSS and JS, no build step, nothing fetched from outside. `viewer/record.js` is the parser and model (also
runs under node), `viewer/app.js` the page. URL parameters: `t=<seconds>` start position, `record=<file>`, `bg=<image URL>`.
Terrain: an image at `viewer/maps/<map name>.png` (whole map, north up) is drawn under the map when present; none ship.
Live: a record with no `result` line is a match still being played. The page then asks the server every 3 s for each
file's new bytes (`<file>?from=<byte offset>`, answered by `run/view_match.py` with an `X-From` header; any other server
sends the whole file and the page copes), parses up to the last complete line, and with "follow live" ticked stays on the
newest sample. Scrubbing, stepping or playing unticks it; ticking it again jumps to the newest sample. Opened details and
the scroll position of the decision list survive each refresh. Polling stops when the result line arrives.
The side pane is tabs (2026-09-21 evening, the user: too much at the top level): **Decisions** (the list with its
filters), **Rules & log** (heuristic firings this minute, bot.log near the playhead), and **Pianist** for a match with a
`jev-<ai_id>.jsonl`. The tab and the two modes below are remembered in the browser. "⟷ wide" (key `w`) gives the side
pane most of the window and the map a third; "▾ charts" (key `c`) folds the timeline's charts away and keeps the lanes.
The layer switches and the legend fold into one corner box over the map.
The Pianist tab: "each actor now" lists every actor of the call at the playhead with what it was doing and its last
decision (a kept course said in amber); click one to open its entry in the picture and its last twelve decisions, and to
narrow the Decisions tab to it. "The call at the playhead" is every question with its answer as bars (the played option
marked), the options as worded, and beside it the picture by section (in the wide mode side by side; the selected actor's
questions first); "pianist per minute" is calls, latency, tokens, questions, changes, kept and failures. The timeline gains
lanes for the pianist's builders, labs and army (a change of course solid, a continue faint); the map draws the named
places, the parties and the groups with a line to where each is going (the `pianist` layer); the decision list takes
"pianist" and "changes only" filters and an actor select, and shows at most 4,000 rows.
The map (2026-09-24, docs/design/2026-09-24-viewer-overhaul.md): every unit is the game's own minimap icon
(`viewer/icons.json` and `viewer/icons/`, exported unmodified from BAR's `gamedata/icontypes.lua` by `run/icons.py`;
CC BY-NC-ND, tinted by side on the canvas as the engine tints them by team; a unit without an icon keeps its class
glyph). The wheel zooms about the cursor (1x to 32x the fitted scale), a drag pans, a double-click or `0` resets,
`+`/`-` step. From 4x buildings show their footprints (`footprint` x 8 elmos, turned by the `finished` facing, a
factory's front edge heavier) and units their names, from 8x their ids, and every unit's standing order (the last
command the bot sent it, ended by a `stop` or the idle flag: a line to the point or target, a square at a build
site); below 4x the last 3 s of orders as before. A click selects a unit (ring; `Escape` clears) and opens the
**Unit** tab: type, health, flags, place, created and destroyed by what, its standing order, its part in the
pianist's picture at the playhead (its actor entry or its group and the group's task, its last decisions, a button
to open it in the Pianist tab) and its history (orders and events, click to seek). The Pianist tab's **Build order**
strip lists every unit begun in the first eight minutes with the clock, the builder and the place (click to seek and
select). The model has `ordersAt(match, frame)`, `unitHistory(match, id)` and `facings(match)` for this.
Tests: `node viewer/test/smoke.js <match dir>` (model, truncated file) and `node viewer/test/browser.js <url> [shot.png]`
(the real page in headless Chromium: load, follow a live match if it is one, scrub, play, hover, toggles, the pianist's
panels when the match has a log; fails on any page error; saves a screenshot when asked).

## Terrain

The header's `terrain` object names a sibling binary file (`terrain-<ai_id>.bin`) and how to read it: `width` x `height`
cells of `cell` elmos (the engine's slope-map resolution, 16), row-major from the north-west; first every height as a
little-endian i16 (elmos; water level is 0, negative is under water), then every slope as a u8 (the engine's slope
value, 1 - the ground normal's y, times 255); then, when the header's `terrain.metal` is true (records from 2026-09-24
evening on), every cell's raw metal as a u8 (the engine's metal map has the same cells; `terrain.metal_max` is the
largest value), which the viewer draws as the "metal patches" layer beside the spots' extractor-radius circles.
`move_classes` lists the distinct movement classes among the unit types
(`kind` tank/bot/hover/ship, `max_slope` in the same slope units, `depth`: deepest water waded, or for ships the
shallowest floated in; `units`: how many unit types use it), so a reader can work out where each kind cannot go. The
viewer renders relief with water and the "bots cannot go" / "vehicles cannot go" layers from it.

## Opponent ground truth

`truth-<ai_id>.jsonl`, written by the shim (not the bot) when the match runs with `WITHIN_REASON_OBSERVE=1`: one line
every two seconds, `{"f": frame, "enemy": [[id, name, x, z, health %, being built 0/1], ...]}`, every enemy unit
wherever it is. The viewer draws it as the faint "opponent (truth)" layer under what our units could see, and uses it
for the opponent's curves; `run/analyze_match.py` derives the opponent's deaths from units leaving the list.


The match server (`run/view_match.py`) lists the engine's replays (`demos/*.sdfz`) in `index.json` under `replays`;
the viewer's header links the newest for download. Copy it into BAR's own demos folder to watch it in the game.
