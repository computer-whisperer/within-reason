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
| `header` | first tick | `format` ("within-reason-record"), `version`, `ai_id`, `team`, `ally_team`, `side` (first three letters of our first unit's name: `arm`, `cor`), `mode` (`heuristic`, `strategist`, `commander`, `pianist`, `player`: the pianist with an Opus player over it), `start_frame`, `seats` [{`team`, `ally_team`, `side`, `controller`: "Unknown" | {`Person`: {name, skill}} | {`Ai`: {name, short_name, version, profile}}}] (who plays every seat, from the start script; records from 2026-09-23 on), `first_tick_frame`, `frames_per_second`, `sample_frames`, `tick_frames` (frames between ticks; absent before 2026-09-20 evening, when it was 15), `wall_start` (unix s), `map` {`name`, `width`, `height`, `wind_min`, `wind_max`, from 2026-09-28 `tidal`}, `start_pos_type` (the script's, from 2026-09-28; `script.txt` beside the record holds the whole start script of every game from then), `grid` {`columns`, `rows`} (the A-H / 1-8 cells of `World::grid`), `metal_spots` [[x, z, amount]] (amount: from 2026-10-03 evening, marked by the header's `metal_spots_amount`: "worth", the patch's worth, its squares' metal times the map's `maxMetal`, so that amount times an extractor's `extracts_metal` is what it draws a second; before that the engine's own figure for its spot point, a raw sum round a point off the patch and without the scale, 2,295 on Comet Catcher where the worth is 1,900; records before 2026-09-20 lack it; from 2026-09-23 the position is the metal-weighted centroid of the patch's squares, not the engine's spot point, H-ECO-SPOT-CENTROID; from 2026-09-24 the game's own list (`mex_count`/`mex_x<i>`/`mex_z<i>` rules params) when the game publishes one, which the replay cards use too, so their spot numbers match), `unit_defs` [{`id` (engine's), `name`, `class`, `metal`, `energy`, `speed`, `weapons`; from 2026-09-28 `tidal_make`, `sonar_range`, `hits_submerged`, `water_only` (the sea: what a tidal generator makes, what a sonar sees, whether any or every weapon is a water weapon; a `submerges` field stood in records of 2026-09-28 only: the engine's isAbleToSubmerge is true for seaplanes and false for submarines); and, in records from 2026-09-20 on, the numbers the build-order simulator reads: `build_time`, `build_speed`, `build_distance`, `builds` [engine ids], `extracts_metal`, `metal_make`, `energy_make`, `energy_upkeep` (negative for solar collectors), `wind_cap`, `metal_storage`, `energy_storage`, `radar_range` (records from 2026-09-20 evening on), `footprint` [xsize, zsize in 8-elmo squares], `death_blast` and `selfd_blast` [radius, damage] or null, `selfd_seconds` (records from 2026-09-22 late night on), `reach` and `reload` (the range and seconds between shots of the type's longest ordinary weapon, manual-fire weapons such as the D-gun left out; 0 unarmed; records from 2026-09-24 on), `blast_radius` (the largest area of effect among its ordinary weapons, a radius as the engine gives it: the Fatboy 150 where its unit file says 300, the Bull 65, the Stout 24; records from 2026-09-28 on), `converter` [energy/s taken, metal per energy] or null, `move` [kind as in `move_classes`, max_slope, depth, slope_mod (records from 2026-09-20 night on)] or null}], `siblings` {`bot_log`, `engine_log`, `replay` (glob), `decision_logs` [file names; the LLM transcript when a session runs]} |
| `s` | every `sample_frames` (30) | state sample. `m`, `e`: [current, income, usage, storage]. `own`: [[id, def, x, z, health %, flags]], flags 1 being built, 2 idle (4 and 8 were the heuristic army's attacker and squad roles; the recorder is passed no roles since that bot was deleted, so they are never set). `en`: enemies in sight or radar [[id, def, x, z, health, being built 0/1]] (absolute health; the protocol has no maximum for enemies; the sixth field from 2026-09-28 on, records before it have five). `al`: allied units (other seats of ours, other players on our side) [[id, def, x, z, team, being built]]; each seat of ours writes its own record, and any one of them shows the whole side through this. `dmg`: [[own unit id, damage taken since the last sample]]. `ms`: slowest `Brain::decide` since the last sample, wall milliseconds (in lockstep mode this includes the commander's thinking time). `late`: the most frames a tick of the interval was sent after it fell due, waiting for the bot's previous answer (always 0 in lockstep; absent before 2026-09-20 evening). `lane`: [claims ended, path walked while held, net displacement, reversals over ninety degrees] summed over the control lane's claims that ended since the last sample (the milling instrument, H-HANDS-LANE; absent when none ended). The fire instrument (records from 2026-09-24 on; `run/fire.py` reads it): `dealt`: [[own unit id, damage it dealt to enemies since the last sample]] (the engine's enemy-damaged event, sent to the attacker's own team only and only while the enemy is in sight or on radar, so a replay's spectator seat gets none); `shots`: [[own unit id, shots fired]] (the weapon-fired event); `ff`: [[own unit id, damage it did to our own side]] (friendly fire: the engine refuses a shot across a friend, `avoidFriendly`, but a shell already flying and any splash still land); `xo`: [[our type, enemy type, damage]] our hits on them by type pair; `xi`: [[enemy type, our type, damage]] their hits on us (-1 for a type never seen: fire from out of sight). Types are indices into `unit_defs`, as `d` `xf` (records from 2026-09-24 evening on): friendly fire by type pair [[shooter's type, victim's type, damage]], the victims the `ff` totals by shooter do not name. `bp` (from 2026-10-13): [[unit id, builder priority, unit it is building or 0]] for every unit that carries the game's builder-priority command (build speed with assist or build options: commanders, constructors, factories, construction turrets); the priority is the gadget's `builderPriority` unit rules param, 0 low, 1 high. In a replay record (the dump widget) -1 is never set (treated as high) and the third field is the build target; in the bot's own records the flag is as the shim reads it (never set reads 1) and the third field is always 0 (the interface has no read for the target). `tr` (replay records from 2026-10-22 only, the dump widget): [metal sent, metal received, energy sent, energy received, units sent, units received], the engine's cumulative team statistics (`TeamStatistics`), so a seat's transfers within its side are read as differences; sharing by hand and a full store's overflow to the side both count. |
| `sense` | every 300 frames, when the shim sends the grids (shims from 2026-10-18; a tick carries them once a second) | our side's sensor coverage as the engine computes it over the terrain: `los`, `radar`, `air`, each `[cols, rows, square (elmos a side), bitmap]`, the bitmap standard base64 of one bit a square, row-major from the north-west, the first square in the lowest bit of the first byte (BAR: sight 64-elmo squares, radar 32, air 128). A zero bit is no coverage; enemy jamming is not in it (BAR jams per side: the engine's jammer map is our own), it follows from a seen jammer's `jam_range`. The header's `unit_defs` carry from the same date `sight` (the engine's flat sight radius), `jam_range`, `stealth`, `cloak` ([energy a second, decloak distance, decloaks on fire 0/1, starts cloaked 0/1] or null), `shield` ([radius, power, intercept mask, exterior 0/1] or null) and `intercepted_by` (the mask of shields that stop its shots: a shield takes a shot when `shield mask & intercepted_by != 0`); 0, false or null in older records. `docs/design/2026-10-18-sight-jammers-cloak-shields.md` |
| `end` | when the engine closes the connection | from 2026-10-03 night: `f` the last frame ticked, `own_units` what this seat had then, `why` the read error. The bot's own last row; in the arena the referee's `result` row follows it, in a game with people nothing does (no referee: who won is not in the record). |
| `ev` | as they happen | engine event. `k`: `created` (+`by` builder id), `finished` (+`facing`: the engine's building facing, 0 south +z, 1 east, 2 north, 3 west; a factory's units leave through its front; records from 2026-09-22 late night on), `destroyed` (+`by` attacker id, `by_d` its type; from 2026-10-19 `w`: the name of the weapon that killed it as the engine's kill event says, a gun's `corstorm_cor_bot_rocket`, a silo's warhead `corsilo_crblmssl` or an explosion such as `fusionexplosion` (the engine names no attacker for a missile strike or a death blast, K-mech-missile-and-blast-deaths-unattributed; until the shim of 2026-10-19 evening `w` was the last hit's weapon and only when `by` was null), or `c` for the engine's own causes: `reclaimed` (by `by`, our constructor), `decayed` (an unfinished frame), `selfd`, `debris`, `other-N` by the engine's `DamageType` number (collisions 2 and 3, crushed 7, an aircraft's crash 8, a transport's or a factory's death 14 and 15, a Lua kill 21 and beyond)), `move_failed`, `enemy_seen`, `enemy_lost`, `enemy_destroyed` (from 2026-10-18 +`by`: the killer when it is allied with us, ours or an ally's, else null; `by_d` its type; `by_ours`: whether it was one of our own units, H-PLAYER-OWN-TRADE), `no_site` (+`what` def), `placed` (+`what` def, `x`, `z`: a build order's site as the host placed it, the engine's closest free site to the one asked for in the `cmd` row; records from 2026-10-08), `rejected` (+`code`), `chat` (+`player`, `text`: a line from a person in the game), from 2026-09-27 evening `given` and `taken` (+`from_team`, `to_team`: a unit that came to this seat from another team, or left it; the transfer tool, a capture; the dump widget writes both to a replay's records from 2026-10-22, and `chat` (+`player`, `text`) for a playing person's lines as the engine logs them, the side's channel included), and from 2026-10-09 `round` (+`n` rounds ready, `queued` still stockpiling: a silo's missile or an anti-nuke's interceptor came ready). `u` unit id, `d` its type, `x`, `z` its last known place (0, 0 when never seen). Not recorded singly: `UnitIdle` (it is a flag) and `UnitDamaged` (summed into `dmg`) |
| `cmd` | ticks with commands | `c`: [["build", unit, def, x, z] or ["build", unit, def] for a factory, ["move", unit, x, z], ["fight", unit, x, z], ["stop", unit], ["repeat", unit, 0 or 1], ["guard", unit, target unit], ["repair", unit, target unit], ["reclaim", unit, x, z, radius] (an area; records before 2026-09-20 only), ["reclaim_feature", unit, feature], ["resurrect", unit, feature], ["say", text] (the game chat: the bot's banner at its first orders), ["attack", unit, target unit] (a turret, deliberately, or the unit an `attack_unit` group was given), ["movestate", unit, 0 hold position / 1 manoeuvre / 2 roam] (air groups hold by it, docs/design/2026-09-22-domains.md); from the duel harness only: ["give", def, x, z], ["selfdestruct", unit], ["firestate", unit, 0 hold fire / 1 return fire / 2 fire at will]; from 2026-09-27 evening (H-PLAYER-TRANSFER, H-HANDS-DGUN): ["send_resources", metal, energy, to team], ["send_units", [unit ids], to team], ["dgun", unit, target unit]; from 2026-10-09: ["fire_at", unit, x, z] (a manual-fire weapon at the ground: a silo's missile), ["stockpile", unit, change] (the kept count of a stockpiler moved by `change` rounds; the game fills a silo or an anti-nuke to its limit on its own)]; from 2026-10-13: ["priority", unit, 0 low or 1 high] (the game's builder-priority flag, H-HANDS-PRIORITY-BIRTH and H-HANDS-LIST-LOW) |
| `intent` | when it changes | `home` and `enemy_start` (presumed), each [x, z]. Until 2026-09-25 also the heuristic army's `station`, `target`, `staging` (null since). |
| `d` | as they happen | decision record, see below |
| `result` | appended by the arena after the match | `result` (the arena's `MatchResult`: outcome, side, corner, game minutes, wall seconds), `opponent`, `replay` (path relative to the match dir, or null) In a replay's records (the dump widget) the row is `{outcome: "replay"}` with `opponent` "replay", and from 2026-10-22 `winners`: the ally team numbers the game's end named (the start script's `allyteam`), empty when the game was stopped before its end. |

`class` is derived from the definition's numbers only, so it holds for every faction and for enemy types:
`commander`, `extractor`, `factory`, `turret`, `building`, `builder`, `army`, `other`; from 2026-09-22 (the domains
design) also `aircraft`, `air_builder` (mobile with no movement class), `hover` and `ship` (armed, by movement kind).

## Decision records
One shape for every decision source, so a new layer slots in without a format change:

`{"t":"d", "f":<frame>, "source":<who>, "kind":<what>, "inputs":<what it was decided on>, "outputs":<what was decided>, "latency_ms":<optional>}`

| `source` | `kind` | `inputs` / `outputs` |
|---|---|---|
| `heuristic` | `rules` | outputs: {heuristic ID (docs/heuristics.md): firings} over the sample interval ending at `f`. Same counts as the `rules:` line of `bot.log`, per second instead of per minute |
| `plan` | `builder`, `lab`, `group` | the hands' play the pick put in force (the rebuilt hands, 2026-10-02 on). inputs {`actor`, `move` (the move's id, `<actor>.<verb>_<what>`)}; outputs {`did` (a sentence)} |
| `plan` | `worlds` | one per pick. inputs {`components`}; outputs {`changed` ["actor: what"], `split` (as the Jev log's `plan` row)} |
| `list` | `builder`, `lab` | a step of the player's build list begun by code. inputs {`actor`, `step`}; outputs {`did`} |
| `lane` | `group` | records of 2026-10-07 only (retired 2026-10-08: roving is a group task set by the pick, whose `plan` row is the record, and the way back is a pick too): the lane's switch of a group into or out of roving (H-MICRO-ROVE), into it on the look after the pick's `rove` move, out of it on the player's `lane` setting. inputs {`actor`, `left` (what the group was doing)}; outputs {`did`}. The viewer still reads it |
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
- **Compressed once the match is closed (2026-10-09 on).** The log is a gigabyte a seat by a game's end (30 to 50 MB a game
  minute, nearly all of it the `pass` rows' `menus`), so the arena turns it into `jev-<ai_id>.jsonl.zst` when the match
  ends: one zstd frame a line, and last a seek table (the Zstandard Seekable Format's skippable frame: an entry a frame
  with its compressed and uncompressed size, then the frame count, a descriptor byte and the magic 0x8F92EAB1), so that a
  reader reaches one line without reading the rest while `zstd -d` reads the whole; 25 to 1 on a player game (rove-task-1:
  1.46 GB to 55 MB). The raw log goes only after every frame has read back as its line. Readers go through `run/jevlog.py`
  (`seat`, `logs`, `lines`, `rows`, `frames`) and see no difference between the two forms; the header's `decision_logs`
  keeps the raw name. `arena --compress-logs DIR` does it to a match (or a batch's matches) recorded before, or whose
  arena was killed first.
- `{"t":"header","format":"within-reason-jev","version":1|2|3,"ai_id","model","interval_frames","rules"}` first: `rules` is
  the standing text every call's picture carried (it is left out of the calls).
- From 2026-10-04 the header also carries `standing_cut` and `wire_cut` (the lean diet's two cuts, H-HANDS-STANDING-CUT and
  H-HANDS-WIRE-CUT). A call's `state` and `questions` are the request as the hands built it: under `standing_cut` a
  gate's `rules` are the short ones and the pick's the whole (so `rules` is logged again on each change of kind); under
  `wire_cut` Jev was sent the same request without backticks and with unit numbers as letters, which the log does not
  show, and `usage` counts what was sent (`run/jev_context_ab.py`'s `no backticks+short ids` rebuilds it).
- `{"t":"call","f","ms","model","usage":{input_tokens,...},"retries","batches","state","questions","answers","groups","places","parties"}`
  per request (`batches`: how many requests a large gate was split into, each carrying the picture; `rules` is a
  field of the call only when the standing text changed; `played` stood here in version 1 only): `state` is the picture without `instructions` and `rules`; `instructions` (the player's packet) is a
  field of the call only when it changed since the last logged call, so a reader carries it forward; `questions` and
  `answers` are the API's own shapes (`docs/harness/jev.md`); `played` is one entry per menu answered, the decision
  record's fields plus `kept` (a busy actor held its course); `groups` [{`name`, `members` [unit ids], `at` [x, z],
  `task` {`kind` hold/move_to/fight_to/engage/attack_unit/hunt/escort, `place`, `to` [x, z]; a hunt's `party` and `quarry`, an escort's `ward` (the builder's name, from 2026-10-01)}}], `places` [{`name`, `x`, `z`, `spot`}] and
  `parties` [{`name`, `ids`, `x`, `z`, `metal`, `composition`}] are what the picture named, so a reader can draw them.
- `{"t":"error","f","error"}` for a call that failed (every actor kept its task).
- From 2026-09-24 late to 2026-09-25 a group's `do` was asked in two levels (H-HANDS-TWO-LEVEL, deleted): those logs
  carry `two_level: true`, the flat `questions`, the composed `<group>.do` in `answers`, and `raw` the `<group>.kind`
  and `<group>.<family>` answers (family-1's log, the first, lacks `raw`).
- From 2026-09-25 (H-HANDS-STANDING) two more line kinds: `{"t": "decompress", "f", "packet_frame", "ms", "usage",
  "orders": {actor: {rule: value}}, "answers"}` when a packet was read into standing orders, and `{"t": "standing",
  "f", "mode", "orders": {actor: {do, params, rule}}, "illegal", "continued", "played"}` each second the executor gave
  an order (mode `on` plays them: `played` entries carry source `standing`; mode `filter` puts a `<actor>.standing`
  question on the call and the `played` entry of that call carries the `verdict`; mode `off` only logs the orders).
- From 2026-09-26 midday (version 2, H-HANDS-ONE-PASS, `docs/design/2026-09-26-one-pass.md`) the executor's
  `standing` line is gone and a `pass` line stands each second the pass had anything to say: `{"t": "pass", "f",
  "open": [slot names with a state to consider], "plan": ["actor: what the base world started"], "hunts": [ends], "rove": ["group_X <clock> what a rover found, attacked, ran from or gave up"] (from 2026-09-27, H-MICRO-ROVE),
  "played": [{actor, kind, played: state id, did, source: rule|list}]}`; on a second that asks it also carries
  `slots` ([{name, kind: "threat at spot_N"|builder|lab|group, base, idle, states: [{id, actor, words, dim, metal,
  default, current, pair_only}]}], index 0 the `party_N.leave` or `<actor>.keep` state), `gate` (the noul ids of the
  first call: `party_N.answer` and one per threat state, `<actor>.change` per actor with an open state, `dim.<kind>`
  per kind of action among them), `events` (what made it ask out of turn) and, from 2026-09-30, `closed` (the busy
  actors not asked about this second, `plan::settle`: absent from `open`); a second with something open and the
  picture as at the last ask carries `quiet` instead. A `worlds_gate` line follows the first call's answers (`flags`
  {noul id: p}, from 2026-09-30 with `<hunt state>.forbidden`, the gate's reading of whether the instructions forbid
  that hunt: at 0.7 or over the hunt's line says so, `worlds` [[state index per slot]] or null, `lines`); the second call is its own `call` line with
  `worlds.pick` alone, and a `plan` line records its pick (`pick`, `confidence`, `changed`, `played` with source
  `plan`). `call` lines carry no `played`: every play is on a `pass` or `plan` line. The header carries
  `worlds_cap`, and until 2026-09-27 evening `packet_rules` (false when the packet was left as prose,
  `WITHIN_REASON_RULES=off`; the `decompress` line then carried `skipped`); from then the decompression is deleted
  (the packet is prose every second, the `standing` tool the one source of rules), so `decompress` lines and
  `packet_rules` appear in older logs only. Among the `events`, `party_N appeared` says a party got a threat slot
  this second (2026-09-27). With `WITHIN_REASON_TOLD=1` the `worlds_gate` `flags` carry `<state>.told` beside each
  builder state's noul, and the viewer's pass panel shows it as a second bar. Logs from 2026-09-26 morning (threats-smoke-1 to -5) carry the `standing` line in its threat form
  (`threats`, `plan`, `gate`/`quiet`, `hunts`, `played` with source `rule`/`plan`) and logs from 2026-09-25 night to
  2026-09-26 morning (worlds-smoke-1 to nouls-1) the retired H-HANDS-WORLDS lines (`quiet`, `groups`, `worlds`,
  `lines`, `played_outright`, `gate` per group, and `worlds_gate` with `flags` per group).
- From 2026-09-28 (H-HANDS-ENGAGEMENT-PLAN, `docs/design/2026-09-28-engagement-plan.md`) an `engagement` line per plan
  question answered: `{"t": "engagement", "f", "group", "position" (the elements' names, sorted, comma-joined), "reason" (why it asked: `first`, `<element> died`,
  `new static turret_N`, `new party party_N (M metal)`, `stale: N s since the last ask`; from 2026-09-28 evening), "ms",
  "model", "battlefield" (the words Jev read), "options" {key: words}, "probabilities" {key: p}, "taken" (the plan
  made the group's task: the top above 0.4 and above the decline's, else `decline`)}`, or `{"t": "engagement", "f",
  "group", "position", "error"}` for a call that failed. Not the pass's `plan` line (the worlds pick). The plan runs as
  the group's task: `groups` in a `call` line say `{"kind": "plan", "plan", "phase", "to"}`, and a `pass` line's
  `played` carries the taking with source `engagement`. The planner, `bot --plan-replay` and `run/plan_replay.py`
  were deleted on 2026-09-29 (ccc7858): logs from then on carry no such line.
- From 2026-10-02 (version 3, the rebuilt hands, `docs/design/2026-10-01-hands-rebuild.md`, H-HANDS-MENU) there are
  no slots and no threat slots: every actor has one menu and every enemy party is a question. The header carries
  `layers` (the layers that are on, `--hands-layers`). A `pass` line on a second that asks carries `menus` ([{name,
  kind: builder|lab|group, idle, course (the course in force by kind), quiet (closed by the `news` layer), audit
  (closed and asked anyway for the layer's audit), moves: [{id: `<actor>.<verb>_<what it is aimed at>`, words, party
  (the party the move is an answer to, or null)}]}], index 0 the actor's `stay`), `open` and `closed` (actor
  names), `layers` ({news: {on, skipped (questions the layer did not send), audited (actors it asked anyway)}}),
  `gate` (the noul ids: `party_N.answer` for every party in the picture; per asked actor `<actor>.change` unless it
  is idle and one id per move; `<move id>.forbidden` beside every `send_N_party_M`), and `events`. An `audit` line
  per audited actor follows the gate's answer: `{"t": "audit", "f", "layer", "actor", "assumed", "change", "best"
  (its best-rated move's id), "best_p", "fault" (both at 0.5 or over: the layer would have been wrong to close it)}`.
  `worlds_gate` is as before with `worlds` [[move index per menu]] (0 is `stay`; world 1 is all zeros) and `flags`
  holding every noul by id. A `call` line's groups say `task.kind` hold (with `picked`), move_to, fight_to, engage,
  attack_unit, hunt, follow (`ward`: a builder's or a group's name) or, from 2026-10-08, rove. `plan` lines are unchanged. A gate that
  fails writes an `error` line and is asked again at the next second.
  With the `same` layer on, the pass line's `layers.same` counts the questions not sent and the audited ones, the
  `call` line carries only what was sent (the standing answers are in `worlds_gate.flags`), and an audit line reads
  `{"t": "audit", "layer": "same", "id", "assumed" (the standing answer), "fresh", "fault"}`. With the `fuse` layer
  on, a `decode` line stands for each decode request: `{"t": "decode", "f", "ms", "usage", "batches", "asked",
  "reads": {"<actor>|place|<place>" | "<actor>|build|<type>" | "<actor>|join|<group>" | "<actor>|follow|<ward>" |
  "<actor>|detach|<n>" | "<group>|back": p}}`; a fused move carries `fused: true` (and `audit` when asked anyway)
  in the pass line's `menus`, and an audit line reads `{"layer": "fuse", "id", "assumed" ("off", or the decoded
  mark), "fresh", "fault"}`. With the `openers` layer on, a move held for its opener carries `held: true`, and an
  audit line reads `{"layer": "openers", "id", "opener" (the best of its openers this second), "fresh", "fault"}`;
  an `<actor> opened` or `party_N opened` event says an opener came back at 0.5 or over while its moves were held.
  With the `split` layer on (2026-10-02), the pick is one request: its questions are `worlds.c<j>.pick` (a Choice
  over component j's changes, keyed `w2`..; absent when the component has one change) and `worlds.c<j>.w<k>` (a
  Choice between `w1` and `w<k>`, one for every change); the `worlds_gate` line carries `components` [{`actors`,
  `worlds`, `lines`}] beside the joint `worlds` and `lines` (still composed, asked only for the audit); the `plan`
  line carries `split` [{`actors`, `changes`, `candidate`, `sampled_p`, `taken`, `confidence`}] in place of `pick`,
  `confidence` and `candidate`; and on the audited share of picks with two actors or more the joint pick's two calls
  are logged as before and an audit line reads `{"layer": "split", "split": [move ids taken], "joint": [move ids of
  the joint pick's world], "joint_confidence", "fault"}` (a fault: the two sets differ). `WITHIN_REASON_SPLIT_AUDIT`
  sets that share (default 0.02).
  Later layers, as the logs of 2026-10-04 stand (read off `crates/bot/src/brain/pianist/mod.rs` and the logs): the
  pass line's `layers` holds `news`, `same`, `again`, `fuse`, `openers`, `places`, `near`, each {`on`, `skipped`,
  `audited`}. The `places` layer: the gate asks `<place>.place` once a place, a move whose place said no carries
  `blanked: true`, and its audit reads `{"layer": "places", "id", "place", "fresh", "fault"}`. The `again` layer's
  audits read as `same`'s with `"layer": "again"`. The `near` layer (code, H-HANDS-NEAR-BUILDER) sets `fused: true`
  on the moves it shuts: the menus do not carry its own mark (`far`), so a reader cannot tell a move `near` shut from
  one the decode fused except by the audit rows (`{"layer": "near", "id", "assumed", "fresh", "fault"}`) and the
  absence of a decode reading under the fuse bar. A `shed` line (`{"t": "shed", "f", "shed"}`) says what the
  picture dropped to fit its size. In a realtime game the gate's `call`, `audit`, `worlds_gate` and `plan` lines
  carry the frame the answer landed (10 to 30 frames after the `pass` line that asked), and no second gate goes out
  while one is pending: a gate is its `pass` line and every line up to the next `pass` line with `gate`. A `decode`
  line is written before the `pass` line of its second.
Size (2026-10-04): a `pass` line that asks is 150 KB to 1.4 MB (its `menus`: every move of every actor in words, nine
tenths of the file), a `call` line 15 to 75 KB: 230 MB for a 15-minute duel, 0.8 to 2.4 GB a seat in a team game.
`run/viewer_index.py` indexes a log without parsing the menus; nothing should read one whole into memory. Logs from
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

## The player's predictions of his state
`predictions-<session>.jsonl` beside each transcript (`predictions-strategist-<n>.jsonl`, `predictions-commander.jsonl`),
one row per `predict` call (H-PLAYER-PREDICT, 2026-10-17): `frame`, `clock`, the player's numbers `extractors`,
`army_metal`, `advanced_factory` (`none` / `building` / `standing`) and `basis`, and `report`, what the report showed
at that turn (`enemy_soldiers_seen`, `enemy_soldiers_seen_metal`, `enemy_extractors_seen`, `youngest_look_s`,
`oldest_look_s`, `enemy_factories_seen`, `ours.extractors`, `ours.army_metal`). The file is appended across session
replacements. `run/predictions.py MATCH` scores the rows against the truth file; without one it lists them.

## Ground truth about the opponent
The record holds the bot's fair view only. With `WITHIN_REASON_OBSERVE=1` the shim writes the once-a-minute census of
both sides into `engine.log` (see `arena.md`) and the truth file (below). The viewer reads the truth file alone
(2026-10-04; it read the census as a fallback before): the seats of one match write byte-identical truth files (one
ally team sees one enemy), so it loads the first.

## Viewer
`run/view_match.py` (the matches directory, the default) serves `viewer/` and every match under `run/matches/` (the batches of 2026-09-19 through 2026-10-06 live in `/ceph/christian/within-reason/run/matches-archive/`, see the ledger's head; give the viewer that directory to see them) at
`/matches/` and opens on a **match browser**: every batch newest first with its label, opponent, map, commit, hands
(the team sizes, the players' model and whether there is one a seat, the commander's model) and each match's result
(whether it was called, and its Jev bill as bot.log's minute lines sum it, a little under the logs' own total; a
batch without a results line is still being played), a filter
box, and a link per match (`?match=matches/<batch>/<NN>/`); "matches" in the header goes back. Given one batch it
lists that batch; given one match (`run/view_match.py run/matches/<batch>/<NN>`, the old form) it opens on it. It
listens on `::` (every interface, IPv6 and IPv4) so another machine on the network can open it; `--bind 127.0.0.1`
keeps it to this machine.
Plain HTML, CSS and JS, no build step, nothing fetched from outside. `viewer/record.js` is the parser and model (also
runs under node), `viewer/app.js` the loading, the map, the timeline and the playhead, `viewer/panels.js` the side
panels. URL parameters: `t=<seconds>` start position, `seat=<team>` the seat the page opens narrowed to, `bg=<image URL>`.
Terrain: an image at `viewer/maps/<map name>.png` (whole map, north up) is drawn under the map when present; none ship.
The refresh of 2026-10-04 (`docs/design/2026-10-04-viewer-refresh.md`) made it a viewer of the match, not of one seat:

**Every seat.** The page loads every `record-<n>.jsonl` of the match directory, the players' transcripts
(`strategist-<n>.jsonl`: one a seat, or one for all seats), `commander.jsonl`, one truth file and `bot.log`. Each file
is a feed: the page asks for its new bytes (`<file>?from=<byte offset>`, answered by `run/view_match.py` with an
`X-From` header; any other server sends the whole file and the page takes what is new), parses the complete lines
once and adds the rows to the model. A sample's `al` rows of a seat that is loaded are dropped at parse time. Units
are tinted by seat; the enemy is the truth layer and what any seat sees, once per unit. The **seat selector** (all,
or one) scopes the stats, the lanes, the charts and the panels, and dims the other seats on the map. With all seats
a chart is the seats stacked (the top edge is the side's sum, the opponent's truth a line over it); with one seat it
is that seat's line against the others' faint ones. A chart's axis ends a little over its 99th percentile.
Live: a match with no `result` line (and not every seat's `end` line) is still being played; the page polls every 3 s
and with "follow live" ticked stays on the newest sample. Scrubbing, stepping or playing unticks it. Opened details
and the scroll position of the decision list survive each refresh.

**The hands' logs are never fetched whole.** The server indexes each `jev-<n>.jsonl` (or the `.jsonl.zst` the arena
leaves, which answers to the raw name) once (`run/viewer_index.py`: it reads the log without parsing the menus, a raw
log by byte offset and a compressed one a frame a line by its seek table, and keeps the index in a `.viewer-jev-<n>.json`
sidecar beside it, the only file the viewer's server writes) and answers `?index` (per game minute the tokens by request kind and question
family, calls, gates, audits; the frames of the gates and the calls), `?gate=<frame>` (the gate at or before the
frame, whole, with the decode's readings in force and the packet as the hands had it) and `?picture=<frame>` (the
groups, places, parties and actors of the last call). Measured 2026-10-04: human-18's three logs (5.9 GB) are indexed
in 8 s the first time (from the page cache) and at once after; a gate comes back in 10 to 20 ms (3 MB, 200 KB
gzipped) and is on the page 50 to 175 ms after a seek; the 8v8 (eight records of 21 MB) opens in 2 s.

**Tabs, by who decides**: **Commander** (the direction in force: the plan and each seat's paragraph with its age;
signals and whether each is given; marks; transfers; chat; notes; its turn at the playhead), **Player** (with several
players a line a seat; for one seat the packet in force, a packet being written, the commander's words to it, the
build lists as last given with the steps begun since, the produce allowances, footwork settings, its notes and its
turn), **Hands** (the gate at the playhead: what opened it, the layers' counts, the party and place questions, every
actor's menu with each move's state in words, the pick by component, what was played, the audits, the picture Jev was
sent; logs older than version 3 get a notice), **Cost** (Jev in dollars and tokens by seat, request kind, question
family and game minute), **Unit**, **Decisions** (the commander's turns, the players' turns, the hands' plays, the
brain's events; narrowed to the hands' actor in focus) and **Log** (the footwork rules fired this minute, the build
order of the first eight minutes, bot.log near the playhead). A move's state is worked out by `gateView` in
`record.js`: asked and answered; asked for a layer's audit; or not asked because its actor is closed by news, the
decode fused it (the reading is shown), `near` shut it (inferred, see the log's section above), its opener holds it,
its place blanked it, or an earlier answer stands (`same` or `again`: the log does not say which per question).
Clicking an actor in the Hands tab rings its units on the map and narrows the decision list to it; clicking a unit
opens the Unit tab with its actor's entry in the picture, that actor's plays and a button to its menu. The tab and
the two modes are remembered in the browser: "⟷ wide" (key `w`) gives the side pane most of the window, "▾ charts"
(key `c`) folds the timeline's charts away and keeps the lanes.
The timeline's lanes: unit, building and extractor losses, kills (a kill every seat saw is one), the commander's
turns and the players' (as wide as the turn took, in a realtime game), the hands' gates, and the hands' plays by
builders, plants and army (the pick's solid, the lists' faint). The charts: metal income, energy income, extractors,
army metal, Jev dollars a minute.
The map (2026-09-24, docs/design/2026-09-24-viewer-overhaul.md): every unit is the game's own minimap icon
(`viewer/icons.json` and `viewer/icons/`, exported unmodified from BAR's `gamedata/icontypes.lua` by `run/icons.py`;
CC BY-NC-ND, tinted by seat on the canvas as the engine tints them by team; a unit without an icon keeps its class
glyph). The wheel zooms about the cursor (1x to 32x the fitted scale), a drag pans, a double-click or `0` resets,
`+`/`-` step. From 4x buildings show their footprints (`footprint` x 8 elmos, turned by the `finished` facing, a
factory's front edge heavier) and units their names, from 8x their ids, and every unit's standing order (the last
command the bot sent it, ended by a `stop` or the idle flag: a line to the point or target, a square at a build
site); below 4x the last 3 s of orders. The `marks` layer draws the commander's marks, each player's own and the
seats' starts; the `hands` layer the named places, the parties and the groups with a line to where each is going,
for the seats in scope. A click selects a unit (ring; `Escape` clears).
Tests: `node viewer/test/smoke.js <match dir>` (every seat loaded as the page loads them, the transcripts' parsers,
the truth, and through `run/viewer_index.py` each Jev log's index, a gate and its view, a picture) and
`node viewer/test/browser.js <url> [shot.png]` (the real page in headless Chromium: the match browser when the url
is the server's root, then load, follow a live match if it is one, scrub, play, hover, zoom, select, the seat
selector, and every panel the match has; fails on any page error; saves a screenshot when asked).

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
