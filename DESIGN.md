# Within Reason — target design (MVP)

Ratified 2026-09-19. Findings live under `docs/` (start at `docs/README.md`).

## Shape

```
 Recoil engine client (spring / spring-headless)          bot process (Rust binary)
 ┌───────────────────────────────────────────┐            ┌──────────────────────────────┐
 │ libSkirmishAI.so  = crates/ai-shim        │   Unix     │ crates/bot                   │
 │  engine thread only:                      │  socket    │  one thread per AI instance  │
 │   events ──► buffer                       │ ─────────► │  Hello, then Tick → Commands │
 │   every N frames: snapshot via callbacks  │ ◄───────── │  brain: world model + pianist │
 │   apply Commands via Engine_handleCommand │            │  later: Jev / LLM layers     │
 └───────────────────────────────────────────┘            └──────────────────────────────┘
```

The shim is the only code that touches the engine. It is dumb on purpose: no strategy, no blocking.
The bot process never calls into the engine; everything it knows arrives in `Hello` and `Tick`.

## Crates
- `recoil-ai-sys` — bindgen over vendored engine headers (`rts/ExternalAI/Interface/*.h`, `System/{Export,Main}Defines.h`,
  GPL-2.0-or-later, so this crate and the shim are too). Header version is pinned by the vendored copy; re-vendor on engine bumps.
- `bot-protocol` — serde message types + length-prefixed postcard framing. Shared by shim and bot. No engine types leak into it.
- `ai-shim` — cdylib. `engine.rs` is the safe wrapper over the callback table, exposing only what the snapshot and commands need.
- `bot` — the bot binary. Listens on the socket; the world model, the pianist and the player's session.
- `buildorder` — offline study tool, not part of the running system: a tier-1 economy simulator and a simulated-annealing
  build-order search over it (`docs/studies/build-order.md`). Depends on nothing else in the workspace.
- `micro` — the control lane (2026-09-25): a `Lane` that turns a host's orders into per-unit footwork every tick
  (flee, fan, kite, formation slots) over a `View` of the host; the bot's brain and the duel director both
  implement the view, so the harness fights with the bot's own footwork (`docs/design/2026-09-25-formation-micro.md`).
  The one unit it runs whole is a rover (2026-09-27, `rove.rs`, H-MICRO-ROVE): a group set to `rove` takes no host
  order; the lane chooses where it looks and what unguarded thing it kills, and keeps it out of every reach.

## Protocol (credit-based, so the shim never blocks the sim)
1. shim → bot `Hello { ai_id, team, ally_team, game_id, teams, start_boxes, frame, map, unit_defs, metal_spots, terrain }` once per
   connection. `teams` is every seat with its ally team and faction; `start_boxes` come from the setup script (the interface
   has no call for them); `game_id` is a hash of that script, by which the bot process finds the sessions that play together.
2. bot → shim `Commands(vec)` — also serves as "ready for next tick" credit.
3. shim → bot `Tick { frame, late, events, snapshot }` — due every `tick_frames` frames (3, 10 Hz, since 2026-09-20; `Hello`
   carries the setting, `WITHIN_REASON_TICK_FRAMES` overrides it with another divisor of 15) and sent at the first UPDATE at
   which the shim holds credit, `late` being the frames it waited. Events that occur while waiting are buffered, not
   dropped. Snapshot = economy, the wind blowing now, own units (with velocity and the frame each can next fire), allied units (seen, never
   commanded), visible enemies with their team and velocity.
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
Armada and Cortex name tables only; Legion and everything else is out of scope for the MVP.

## Done means
`run/` script plays shim+bot vs BARb headless; log shows buildings completed, units produced, an attack order issued, and
the bot process can be killed and restarted mid-game without stalling the match.

## Match record and viewer (2026-09-19)
The bot process, which sees everything the brain sees, writes a JSON Lines record per match (`crates/bot/src/recorder.rs`,
format in `docs/harness/record-format.md`): state samples once a game second, engine events, commands, and decision
records in one source-agnostic shape (source, frame, inputs, outputs, latency) fed by the brain's journal
(`brain/journal.rs`). `viewer/` is a static page that replays it beside the LLM transcript and the census.
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
