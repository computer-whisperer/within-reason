# Within Reason — target design (MVP)

Ratified 2026-09-19. Findings live under `docs/` (start at `docs/README.md`).

## Shape

```
 Recoil engine client (spring / spring-headless)          bot process (Rust binary)
 ┌───────────────────────────────────────────┐            ┌──────────────────────────────┐
 │ libSkirmishAI.so  = crates/ai-shim        │   Unix     │ crates/bot  (the flagship)   │
 │  engine thread only:                      │  socket    │  one thread per AI instance  │
 │   events ──► buffer                       │ ─────────► │  Hello, then Tick → Commands │
 │   every N frames: snapshot via callbacks  │ ◄───────── │  brain: world model + pianist │
 │   apply Commands via Engine_handleCommand │            │  later: Jev / LLM layers     │
 └───────────────────────────────────────────┘            └──────────────────────────────┘
```

The host (`crates/ai-host`) is the only code that touches the engine. It is dumb on purpose: no strategy, no blocking.
The bot never calls into the engine; everything it knows arrives in `Hello` and `Tick`.

Where the bot is, is the library's choice of transport (2026-10-05), not the bot's nature:
- **Behind the socket** (above): `crates/ai-shim` is the host over a Unix socket to the bot process. The flagship's:
  its deciders are models, which block, and a bot process can be restarted or crash without the game.
- **Inside the library**: `ai_host::InProcess<B>` calls a `bot_protocol::Bot` on the engine thread, the same `Hello`,
  `Tick` and `Commands` with no socket and no second process. For heuristic bots (`bots/`), which answer inside a
  frame and are shipped as one file. `bots/template` is the skeleton.

Two regimes, kept apart by directory: `crates/bot` is the flagship, where no code decides a live game without Jev or
the player (`docs/code-decisions.md` is its roster); `bots/*` are heuristic bots, which decide in code and cost no
tokens (`bots/README.md`).

## Crates
- `recoil-ai-sys` — bindgen over vendored engine headers (`rts/ExternalAI/Interface/*.h`, `System/{Export,Main}Defines.h`,
  GPL-2.0-or-later, so this crate and the shim are too). Header version is pinned by the vendored copy; re-vendor on engine bumps.
- `bot-protocol` — serde message types + length-prefixed postcard framing, and the `Bot` trait an in-library bot implements. Shared by host and bot. No engine types leak into it.
- `ai-host` — the engine's side of any AI library: `engine.rs` is the safe wrapper over the callback table, exposing only
  what the snapshot and commands need; `lib.rs` the instance (ticks, credit, build sites, the arena's heartbeat, census and
  truth lines), the `Transport` it speaks through, `InProcess`, and `export!`, which writes a library's three exports.
- `ai-shim` — cdylib, the flagship's library: the host over the socket (`link.rs`).
- `bots/template` — cdylib, lobby name `WRTemplate`: the host over a bot inside the library; the skeleton to copy.
- `bot` — the bot binary. Listens on the socket; the world model, the pianist and the player's session.
- `buildorder` — offline study tool, not part of the running system: a tier-1 economy simulator and a simulated-annealing
  build-order search over it (`docs/studies/build-order.md`). Depends on nothing else in the workspace.
- `micro` — the control lane (2026-09-25): a `Lane` that turns a host's orders into per-unit footwork every tick
  (flee, fan, kite) over a `View` of the host, and runs the host's declared groups whole (`Lane::set_group`, since
  2026-10-24: `docs/design/2026-10-24-formation-drills.md`): a formation of kind Hold, Go, FightTo or Attack is an
  anchor the lane paths over the ground every member's class stands on, paced by its body's lag, with ranks of slots
  hung from it and every member sent to its slot carried a lead ahead; a hunt (H-MICRO-HUNT) and a rove (`rove.rs`,
  H-MICRO-ROVE) are groups too. The host names members, a kind and route points, and orders no member; it reads the
  group's state (anchor, pace, formed, exceptions by cause, arrived, files, room, the way left). The bot's brain and
  the duel director both implement the view, so the drills (`docs/harness/duels.md`) run the bot's own group control;
  the flagship's hands declare their ground holds and walks since 2026-10-25 (`docs/design/2026-10-25-hands-migration.md`)
  and attach the rest chunk by chunk.

## Protocol (credit-based, so the shim never blocks the sim)
0. Both sides write the protocol's fingerprint (eight bytes: a hash of `bot-protocol`'s message and framing sources
   without their comments, `crates/bot-protocol/build.rs`) and read the other's before any frame
   (`bot_protocol::handshake`). Postcard is not self-describing, so builds of two trees would misread each other's
   frames; on a mismatch the bot refuses the connection before any session and says so once in its log, and the shim
   says so once in the game chat and tries again every half minute.
1. shim → bot `Hello { ai_id, team, ally_team, game_id, teams, start_boxes, frame, map, unit_defs, metal_spots, terrain }` once per
   connection, sent at the engine's `init` (the end of loading, before the starts are placed and long before the first
   frame: the player takes its opening turn then, `docs/design/2026-09-30-opening-turn.md`), or at an update when no bot
   was listening at `init`. `teams` is every seat with its ally team and faction; `start_boxes` come from the setup script (the interface
   has no call for them); `game_id` is a hash of that script, by which the bot process finds the sessions that play together.
2. bot → shim `Commands(vec)` — also serves as "ready for next tick" credit.
3. shim → bot `Tick { frame, late, events, snapshot }` — due every `tick_frames` frames (3, 10 Hz, since 2026-09-20; `Hello`
   carries the setting, `WITHIN_REASON_TICK_FRAMES` overrides it with another divisor of 15) and sent at the first UPDATE at
   which the shim holds credit, `late` being the frames it waited. Events that occur while waiting are buffered, not
   dropped. Snapshot = economy, the wind blowing now, own units (with velocity and the frame each can next fire), allied units (seen, never
   commanded), visible enemies with their team and velocity. Once a second the snapshot also carries `sensors`: our ally team's sight, radar and air-sight coverage grids as the engine computes them over the terrain (`Map_getLosMap`, `Map_getRadarMap`, `Map_getAirLosMap`; BAR squares of 64, 32 and 128 elmos), one bit a square; enemy jamming is not in them (BAR jams per side) and follows from a seen jammer's `jam_range`. The unit definitions carry `sight`, `jam_range`, `stealth`, `cloak`, `shield` and `intercepted_by` from 2026-10-18 (`docs/design/2026-10-18-sight-jammers-cloak-shields.md`).
4. Shim reads the socket non-blocking at each UPDATE and applies any `Commands` on the engine thread.

The bot runs two passes on every tick (`brain/mod.rs` `decide`): the control lane (`crates/micro`, driven from `brain/micro.rs`) every tick, and the
whole brain (`think`) on the ticks due at multiples of `BRAIN_FRAMES` (15, 2 Hz), fed every event since it last ran. The
brain's schedule keys on the due frame (`Tick::due`), so a late tick is not a missed one.
Design: `docs/design/2026-09-20-micro-lane.md`.

If the bot is absent or dies, the shim keeps the game running and retries the connection about once a second, re-sending `Hello`.
Socket path: `$WITHIN_REASON_SOCKET`, else `$XDG_RUNTIME_DIR/within-reason.sock`.

### Transfers between seats and the D-gun (2026-09-27)
`Command::SendResources { metal, energy, to_team }` and `Command::SendUnits { units, to_team }` are the AI interface's
`COMMAND_SEND_RESOURCES` and `COMMAND_SEND_UNITS` (the game allows both between allies, capping a resource transfer at the
receiver's store); the engine answers with `Event::UnitGiven` to the new owner and `Event::UnitTaken` to the old.
`Command::DGun { unit, target }` is `COMMAND_UNIT_D_GUN`, the commander's manual-fire weapon at one unit. All three are
the player's `transfer` tool and the hands' D-gun state (`docs/design/2026-09-27-posing-changes.md` §12b, §8.1).
`Event::Received { metal, energy }` (2026-10-25) is what allies' overflow put into our stock over the engine's last
second (`Economy_getReceived`), sent once a second while above zero: the engine adds an ally's excess straight to the
stock (`CTeam::SlowUpdate`, BAR's default without a sharing tax), and its income figure leaves it out. The distilled
bot counts it as income (D-RECEIVED); the flagship's recorder passes over it.

### The silo (2026-10-09)
`Command::FireAt { unit, at }` fires a command-fire weapon at a place: BAR's nuclear missile is `commandfire` (the
engine's `manualfire` tag), so a silo's round sits until this order. The engine's manual-fire command
(`COMMAND_UNIT_D_GUN_POS`) is allowed only to a type with `canManualFire`, which BAR gives the commander and not the
silo, and for a type without it the weapon code skips the manual-fire check, so the host sends the commander's D-gun as
manual fire and the silo's missile as `COMMAND_UNIT_ATTACK_AREA` with no radius, an attack at the ground
(K-rules-the-silo-fires-on-an-attack-at-the-ground; measured 2026-10-09 in the `nuke-check-*` scenarios). `Command::Stockpile { unit, change }` is `COMMAND_UNIT_STOCKPILE` one order a step, up with no
option and down with the right-mouse one: the game's "Stockpile control" gadget fills every silo and anti-nuke to its
unit file's limit on its own from the moment it stands and reads a stockpile order as a step of that kept count, so
there is nothing to order a round with (K-rules-a-stockpile-fills-itself-to-its-limit). `OwnUnit` carries `stockpile`
and `stockpile_queued` (asked of the engine only for types with a stockpiled weapon) and `UnitDefInfo`
`stockpile_seconds` (120 for Armada's silo, 90 for an anti-nuke; 0 for everything else), `stockpile_energy` (the round's
cost: over the seconds, the draw) and `coverage` (an interceptor's: 2,000). The record writes a `round` event when a
count rises (`docs/harness/record-format.md`). The player fires with the `fire` tool (a silo's handle and a place in
coordinates or a grid cell; the user's ruling of 2026-10-09: the player chooses the exact place), carried out by the
pianist as `remove` is (`crates/bot/src/brain/pianist/fire.rs`); the picture lists our silos and anti-nukes with their
rounds and names his anti-nukes with their coverage. The briefing carries the stockpilers and every report prints a
`missiles` line from them; a round coming ready is a done line of the hands (`note_rounds`), and a silo's wakes the
player (`Wake::missile_ready`, on by default), after nuke-2-hard (2026-10-09) where two missiles sat ready for five
minutes before his anti-nuke while nothing the player read named them. `Command::Priority { unit, low }` is BAR's builder-priority flag (`CMD_PRIORITY` 34571 through `COMMAND_UNIT_CUSTOM`, one
parameter, 0 low and 1 high): a low builder draws only the team's surplus, full speed or nothing; the gadget consumes the
order in `AllowCommand`, nothing is queued, and the shim reads the flag back as `OwnUnit::priority_low` from the unit
rules param `builderPriority` (K-mech-builder-priority-is-a-builder-flag; the study `docs/studies/2026-10-13-builder-priority.md`).

### The banner (2026-09-20)
The game names every AI at random (`ai_namer.lua`: donor and contributor names) and takes nothing from the AI, so at
its first orders (frame 60) the bot says a chat line: `<its given name> is Within Reason <commit, "+" when the tree
was dirty at build> | player|pianist (model, effort) | off: <disabled heuristics> | seat aiN team T
side`. `Command::Say` carries it; the shim sends it as `/say`, the only form the engine takes text from an AI in,
so the line appears as chat from the AI's host player (the arena's spectator, or the human hosting the AI). The
commit comes from `crates/bot/build.rs`.

## MVP brain (deliberately weak; exists to exercise the loop end to end)
The game places AI teams itself (`game_initial_spawn.lua` guesses a spot in the start box), so no start position is sent.
Commander and constructors build metal extractors on nearest spots, energy, a bot lab (more when metal floats) → labs queue cheap combat units →
idle army is sent with Fight orders at the enemy start / last seen enemy once it reaches a threshold.
Armada and Cortex name tables only at the MVP; Legion joined them on 2026-10-14 (a third `Roster`, the glossary and the combat table regenerated from `legcom`, the arena's `--side legion` / `--their-side legion` writing the game's `experimentallegionfaction` option; `docs/knowledge/legion.md`).

## Done means
`run/` script plays shim+bot vs BARb headless; log shows buildings completed, units produced, an attack order issued, and
the bot process can be killed and restarted mid-game without stalling the match.

## Match record and viewer (2026-09-19)
The bot process, which sees everything the brain sees, writes a JSON Lines record per match (`crates/bot/src/recorder.rs`,
format in `docs/harness/record-format.md`): state samples once a game second, engine events, commands, and decision
records in one source-agnostic shape (source, frame, inputs, outputs, latency) fed by the brain's journal
(`brain/journal.rs`). `viewer/` is a static page that replays every seat's record beside the commander's and the players' transcripts, the hands' logs and the opponent's truth.
Chose-because: the shim and the protocol stay untouched, and the record is complete for our side by construction.
Rejected: parsing the engine's `.sdfz` (it re-simulates in the engine and holds none of our decisions); scraping `bot.log`
(free text, per-minute granularity).

---

# The LLM beside the brain: history (2026-09-19 to 2026-09-25)

Three ways of putting a Claude session beside the brain were built and two deleted. The **strategist** (2026-09-19)
set expiring directives the heuristics consulted while the game ran on; the **field commander** (2026-09-19 to 24,
Sonnet, the game held still during its turns, squads and a unit mix over the heuristic economy and army) taught what
the levers were; the **player** (2026-09-21, below) writes the packet the pianist plays and is the one session since
2026-09-25, when the heuristic bot's deciders, both older modes and the Lua policy were deleted
(`docs/design/2026-09-25-one-decider.md`). What those modes measured is in `docs/knowledge/heuristic-bot.md` and the
ledger; their designs stay in `docs/design/` as history. Verified mechanics of the session itself (accounts, limits,
the transcript) are in `docs/harness/claude-p.md`.

## The pianist and the player (2026-09-21)

`bot --pianist`: Jev (TypeSafe's System One model) plays every unit from a prose packet of standing instructions, one
call a game second over a picture of the game and a menu per free actor; no decision heuristic runs beneath it (the
control lane and the tracking do). `bot --pianist --player`: an Opus session (`strategist` mode `Player`) writes the
packet; its turns hold the game as the commander's do, its one lever is `instruct`, its `situation` is the picture the
hands read, and the pianist wakes it when Jev judges the situation needs it. Design, rulings and the series:
`docs/design/2026-09-21-pianist.md`; the hands' rules are `H-HANDS-*` in `docs/heuristics.md`; what Jev needs is
`docs/knowledge/jev.md`.

The hands as rebuilt on 2026-10-02 (`docs/design/2026-10-01-hands-rebuild.md`): three deciders (the player, Jev, the
micro engine); one menu per actor and one move per actor a second, a move being a verb from a short fixed list and
what it is aimed at (`pianist/menu.rs`); code chooses neither what a move is aimed at nor for whom, and reads the
packet only for names; enemy parties are questions, never menus; a gate of nouls, worlds joined from the best-rated
moves, and a pick that is one Choice over the worlds with the course among them, the world drawn by its probability
(`compose.rs`; two stages until 2026-10-24, `docs/design/2026-10-24-pick-rework.md`); what Jev must remember is printed in each actor's entry (the
registers `reached` and `met`; the last-pick register and every clock on a course retired 2026-10-24: the policy of t-1
does not enter Jev's words); and every saving on what is asked is a layer with a switch and an audit (`layers.rs`, `--hands-layers`).

### The game's snapshot (2026-10-21)

A game runs from a snapshot, not the checkout (the user: edits to key parts of the codebase must not touch a running
game; `docs/design/2026-10-21-brief-rework.md` §5). `bot --snapshot DIR` (`crates/bot/src/texts.rs`) copies every
text the models read, the reference tables, the bot binary itself and the checkout's git state (head, status, diff)
into `run/matches/<batch>/snapshot/` with the checkout's relative paths; the arena does it once a batch and
`run/human_game.sh` once a game, and both run `snapshot/bot` with `WITHIN_REASON_TEXTS` pointing at it. The texts are
still read from disk at every use (the 2026-09-22 ruling: an edit needs no rebuild), from the snapshot, so an edit
reaches the next game. The record thereby carries the texts and the code state a game was played with, which a
review reads (`run/brief_view.py --match <batch>`). The shim stays shared: installed by rename, kept by the engine
that mapped it, loaded anew by each later match.

The briefs are manifests (`docs/briefs/rework.manifest`, the default since 2026-10-23 for the player and the commander
alike; `docs/briefs/experience.manifest` and `docs/briefs/standard.manifest`, the two old briefs kept runnable as
controls by `arena --brief` / `WITHIN_REASON_BRIEF`; the design's §3, §4):
one document path a line, assembled by the bot at session start (`texts::assemble_parts`), every HTML comment and
every passage whose `<!--meta ... -->` block says `status: deprecated` stripped, so provenance written in comments
never reaches a model (`run/brief_check.py` checks the blocks' ids and references). Nothing assembled is committed;
the generated documents under `docs/briefs/reference/` are written by their generators (`run/regenerate_reference.py`).
`bot --print-prompt ROLE [--sources]` prints a role's prompt as a session gets it, and `run/brief_view.py` shows that,
never a composition of its own.

## Terrain (2026-09-19)

The shim sends the ground once, in `Hello`: heights and slopes at the engine's slope-map resolution (16 elmos), and
each unit type's movement class (kind, steepest slope, water depth). The bot builds walking-distance fields over it
(`crates/terrain`, Dijkstra on the passable cells of our soldiers' class) from home and from where the enemy
is believed to live, and the brain's geometry goes through them (`brain/routes.rs`): which metal spots are ours, what
"forward of home" means (along the route, not the straight line), which attack targets can be walked to, where a wave
stages. Without terrain data everything falls back to straight lines. Known simplifications: one movement class stands
for the whole army; buildings and wrecks are not obstacles; the commander uses the soldiers' field. The recorder writes
the grid beside the match record and the viewer draws it (`docs/harness/record-format.md`, "Terrain").


## Duels (2026-09-19)

Unit-against-unit tests for the matchup table (`docs/harness/duels.md`): a headless match in which both teams are our AI
and the `duel` binary (in the arena crate) is the bot for both, so one director spawns both armies, orders them and
scores the fight. The protocol gained two commands at the end of `Command`, `GiveUnit` (the AI interface's spawn cheat)
and `SelfDestruct`, and on 2026-09-24 `FireState` (a recorded engagement's units hold fire while the director hurts them
to their recorded health, `docs/harness/duels.md`, Scenarios); normal matches never send them. The arena crate became a library plus two binaries so that `duel`
shares the autohost channel and the engine chores (`crates/arena/src/harness.rs`).
Chose-because: the director needs both sides' state on the same frame (to start both armies together and to know when
a fight is over), which two independent brain sessions in `bot` would have had to share through a side channel.
Rejected: `/give` over the autohost channel (needs `/cheat`, and the runner would not learn unit ids); a Lua gadget
(needs a game fork or mutator).
