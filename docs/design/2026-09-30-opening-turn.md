# The opening turn: the player thinks before the game (2026-09-30)

## Why

The player's first orders used to land some seconds into the game (0:09 in the arena under the think penalty, ten
to thirty seconds in a realtime game), and the hands played those seconds under the default "no player is
connected" instructions: in players 29 and 30 that was a fourth solar and the plant ten seconds late
(`docs/studies/2026-09-29-pick-framing.md` §9). The user, 2026-09-30: "Allowing the player a turn to think before
the game makes sense overall if we can make it work in all modes. In human games, can we start the first opus call
during the countdown while humans place their starting orders before the game actually starts?"; then "go ahead
with the relative step".

## What the engine and the game give (read in `upstream/`, not run)

- The engine creates the AI and sends `init` at the end of loading (`CGame::Load` step 8, `LoadSkirmishAIs`, after
  `LoadLua`): before the people place their starts. Updates come only with simulation frames (`CGame::SimFrame`):
  none while the starts are placed or the countdown runs. Chat still arrives.
- In a game where starts are chosen (`startpostype` 2) BAR's `game_initial_spawn.lua` refuses a start sent by an AI
  team (`AllowStartPosition`), sets one at once when an allied person places it (`aiPlacedPosition`), and otherwise
  guesses one at `GameStart`. Before the game we know our start box, not our spot, and with no updates we learn
  neither a person's placement of us nor theirs until frame 0.

## The design

One path in every mode: **the opening turn is asked for when every seat of ours has said Hello, and nothing of ours
acts until it is over.**

1. **The shim says Hello at `init`** (the same message as before; when no bot is listening it connects at an update
   as it did).
2. **The bot asks for the opening turn at Hello**, once every seat of ours the script lists has said it
   (`Shared::seat_said_hello`). The turn's prompt is the map as it can be known without a start (the spots, the start
   boxes, the seats and their factions) and what the turn is for; the report of a running game is not in it. The
   tools work as in any turn: `instruct`, `produce` (`all`), `queue`, `wait`.
3. **The relative step.** A bare `extractor` in a list (also written `extractor nearest`) takes the free spot the
   builder reaches soonest when the step comes up (`next_list_step`, as it always did): the opening list names no
   spot, so it does not depend on the start.
4. **Lockstep holds the first tick** until the opening turn is over (`Shared::hold_for_opening`): the arena's game
   stands at frame 0 as a human game's stands in its countdown. The turn's orders are in force when it ends: the
   think penalty is for turns taken while a live game would have run on.
5. **The hands ask nothing until the opening turn is over** (`Shared::opening_pending`; the pass and the lists wait).
   In a realtime game whose people ready up before the turn ends the game runs and the hands wait for it. A session
   that failed to start, or a turn abandoned at its cap, ends the wait: the hands play on the default instructions
   as before.
6. **The first report of the running game** is a full one and carries the map again, now with our start, the
   walking distances and the passages. It comes when the player's own `wait` says (the prompt asks for a short one).

The fallback when a seat never says Hello: the first turn is asked at a tick as before (every seat published, or
15 s), held in lockstep, its orders in force when it ends.

## What is not decided here

- Whether the brief's opening for a map should be rewritten around the bare `extractor` step (today it gives lists
  of named spots per start): the role text and the brief's Comet line say how to write the opening before the
  start is known; the lines per start stay as what the pros did from each.
- Our start position stays the game's or a teammate's to place.

## Measured (2026-10-01, three arena smokes) and what waits for a game with people

- The shim connected at `init` and its map reads worked there: the spot list came from the game's rules params, as
  at an update (`engine.log`: "connected to bot at frame 0", "metal spots: 80 from the game's rules params").
- opening-smoke-1 (lockstep, one seat): the opening turn at frame 0, 9.9 s of wall, the list written with bare
  `extractor` steps and in force at the first pass (0:02); M M M S S S and the plant at 1:08 (players 14 to 28; 1:18
  in players 29 and 30). The second turn at 0:10 carried the map with the start and rewrote the packet with places.
- opening-smoke-2-realtime: the arena has no placing phase, so the turn (14.6 s) ran into the game: the hands waited
  and first played at 0:10 from the list, the plant at 1:15. In a game with people the placing phase is that time.
- opening-smoke-3-two-seats (lockstep): one opening turn naming `commander_t0` and `commander_t1`, each seat
  M M M S S S, the plants at 1:10 and 1:08.

Not yet seen: that `init` precedes the placing phase in a lobby-hosted game as the engine source says (a rehearsal
with a person in the lobby shows it). The shim changed: `run/install_to_bar.sh` before a game with people.
