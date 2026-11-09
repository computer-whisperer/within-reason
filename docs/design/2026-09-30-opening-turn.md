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
   In a realtime game whose people ready up before the turn ends the game runs and the hands wait for it (from
   2026-11-08 the game is paused for it: "The hold", below). A session
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
  (2026-10-10, after glitters-8v8-late-1's seats first ordered at 0:10 to 0:14: under `--realtime` the arena now
  writes `GameStartDelay=15` (`--start-delay S`), the engine's countdown after every seat has loaded, and the AIs
  are created before it, so the turn runs in it as in a lobby. Lockstep keeps 0 and the hold.)
- opening-smoke-3-two-seats (lockstep): one opening turn naming `commander_t0` and `commander_t1`, each seat
  M M M S S S, the plants at 1:10 and 1:08.

Seen since in the lobby games (read 2026-11-08 from their records): `init` precedes the placing phase as the engine
source says, and in the 32 seats of the client games from 1793582746-human to 1794106612-human the opening turn (11 to
41 s of wall) was over before frame 0 and the first order went out at 0:02 to 0:06. A game nobody places a start in
has no such phase: "The hold", below.

## The hold: a game that runs before the opening turn is over (2026-11-08)

Step 5 let a realtime game run while the hands waited for the turn. In 1794184846-apex_2v1 (the user's client game,
a spectator hosting three AI seats) nobody had a start to place: frame 0 came about 3 s after Hello, the turn took 38 s
to its `wait`, and the commander stood on 1,000 metal until 0:38 (every earlier client game, 32 seats, ordered at
0:02 to 0:06: its people's placing covered turns of 11 to 41 s). The user, 2026-11-08: "The missing player countdown
pause is almost certainly the cause ... implemented generically if our bot has not finished its preparation turn in
any live game. If that doesn't work, the answer is probably to have canned build orders on basic heuristics."

So the game is paused for the turn, in any realtime game:

- **The bot asks.** The session's fast thread (`session.rs`), answering a tick of the game's first second
  (`HOLD_BY_FRAME`) while the opening turn asked at Hello is still in hand (`Shared::opening_in_hand`), adds
  `Command::HoldGame { seconds }` to its answer, waits for the turn to end (`Shared::wait_for_opening`: its `wait`,
  its response's end, its cap, or its session gone) and then writes one more frame, unasked: the release. Not in
  lockstep, which holds the first tick itself; not for a bot started into a game under way.
- **The host pauses** (`ai-host` `Instance::hold`): `COMMAND_PAUSE` as the AI's host player, a line in the chat
  saying whose pause it is, and the seat's connection handed to a thread, because a paused game sends an AI no update
  and so calls no export of ours to wait in. The thread reads the socket for the release, lifts the pause and hands
  the connection back; the engine thread takes it up at the next update and the ticks go on. The pause goes out with
  the first seat to hold and is lifted when the last is released (`HOLDS`), so eight seats with eight players wait
  for the slowest. It is lifted without a release after `seconds` (the turn's cap and thirty), or when the bot's
  connection is lost; when the engine releases the AI the thread is cancelled and joined and the pause left alone.
- **What the engine does with it** (read in `upstream/`): the pause travels as the host player's
  (`CAICallback::HandleCommand`, `clientNet->Send` under the net lock, which is what makes the thread's call safe);
  the server takes it from a local host always, from a remote player when the game may be paused, and never from a
  remote spectator (`GameServer.cpp` `NETMSG_PAUSE`). Where it is refused the game runs on, the seat is silent
  until its release, and the hands begin when the turn ends, as before the hold.
- **No game decision is made by this.** The hold changes when frame 1 follows frame 0, not what anyone orders.

Not done: lifting the pause when a person lifts it first and pauses again for their own reason (ours would lift
theirs); a second hold later in a game (there is one opening turn).

Measured, hold-smoke-1 (arena, `--realtime --start-delay 0`, the 1794184846 situation): the pause went out at frame 1
("arena paused the game"), the game stood at frame 1 for 23.2 s of wall, the release lifted it, and the commander's
opening went out as one queue at 0:02. Unmeasured: a client game hosted from the lobby (the same local-host path), a
dedicated host with people in it, several seats.

## One list per faction (2026-11-04)

The role words made one opening serve whichever faction the lobby gave the seat, and in 1793584014-human that one
opening was Armada's clock on a Cortex seat (K-player-a-faction-blind-opening-ran-the-armada-clock-on-a-cortex-seat).
There are three factions, so the pre-game `queue` and `produce` now take one list per faction for a builder or an
actor (`{"commander": {"armada": [...], "cortex": [...], "legion": [...]}}`): the tools check each list in its own
faction's names (`check_steps` with the faction, the id and factory checks run once per faction's game), the shared
state holds them as `Listed::ByFaction` and `Allowance::by_faction`, and the hands take the seat's faction's list at
their first look (`take_lists`, `take_allowances`), saying in their first line which was taken and which dropped. A
builder of another faction (an ally's gift) takes its own faction's list. A faction left out keeps what it had, or
plays from the menu, and the line says so. Role words stay for a step that is the same in every faction. The arena
fixes the side, so only a lobby seat on Random exercises the form; it is unmeasured in a game.

## The sides before the game (2026-11-04)

The review of what the player reads in order (2026-11-04) found the opponent's lobby faction and skill known at Hello
(the start script's `side` and `skill` per team, in `hello.teams`) and withheld until the first report's sides line:
in 1793584014-human the player wrote its opening knowing u6bkep's name and nothing else of him, and read "Armada in
the lobby, lobby skill 19" at 0:10. The opening prompt now carries the sides line in the running report's words
(`report.rs` `sides_line`, one function for both), filled at `seat_said_hello` from `Briefing::sides`; before the
game a seat of ours on Random reads "faction chosen by the game at the start" there, as its seat line does.

