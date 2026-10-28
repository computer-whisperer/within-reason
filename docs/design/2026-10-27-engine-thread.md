# The engine thread in games with people (2026-10-27)

**Status.** Agreed by the user 2026-10-27 ("Agreed with all four, including the smaller distance maps"); built, the smaller
maps replaced by none where nothing reads them (item 3) and the request made cheap rather than moved (item 4).

## Why

The game with people of 2026-10-25 (`run/matches/1792971120-human`, All That Glitters Extended, six AI seats of our
side on the user's client, which spectated) stalled about 30 s at its start and stuttered after. The user: "that game
had some significant lag sources between the five of our bots, including a many-second stall at the very beginning";
after the review, "The 40s stall was not normal -- I think that was caused by the territory bots starting up."

What the review found (ledger lagprobe1-ext, lagprobe2-nukes):

- The server sets the game's speed from the players' CPU only, spectators left out (`GameServer.cpp`,
  `LagProtection`), so a slow AI on a spectator's client never slows the game: the client falls behind and fast-forwards.
  Our bots' banners, said at the client's frame 60, reached the server at its frame 960: about 30 s behind.
- An in-library bot's tick runs on the engine thread (`bots/README.md`). Reproduced with a per-seat timer in the host:
  frame 0 took 34.4 s, each WRTerritory seat's first tick 17.2 s, walking 446 places (`ground: 446 places walked in
  16670 ms`): one full-map distance field a place, 960 x 640 cells of 16 elmos, 2.5 MB each, about 1.1 GB a seat kept for
  the game.
- After it, WRTerritory 0.6 / 1.2-1.35 / 2.05-2.1 s of engine thread a minute at 5 / 10 / 20 min, about a hundred ticks
  over 100 ms in 20 min: T-RACE every 30 s (105 ms at 0:30 to 434-620 ms at 12:00 with eight groups of ours) and the
  replan's request every ~10 s from 5:00 (120-170 ms). WRDistilled under 0.12 s a minute and no tick over 25 ms, its
  silos firing included.
- Since 31dc99e every AI of ours sends `/cheat 1` at its first update (for the duel harness's `/give`). On a dedicated
  server the server refuses it and tells every player "Player N is not allowed to execute command cheat"; on a client
  that hosts the game it would turn cheating on.

## The changes

1. **The host's cheat only for a give.** `/cheat 1` goes out with the first `GiveUnit` and again `CHEAT_RESEND` frames
   later, spawns held `CHEAT_LAG` frames after it, as before; no bot of a game with people gives units, so none sends
   it. Rejected: an environment variable for the duel harness (one more thing a run must remember; the give is the
   only reason for the flag).
2. **The host's per-seat timer, standing.** Per instance, the wall time of its reads of the game (the snapshot, wrecks,
   sensors), of the bot (`send`: the in-library bot's tick, the shim's socket write) and of applying its commands (the
   build-site search): a `timing` line a minute and a line for any single call over `SLOW_CALL` (50 ms), and the hello's
   reads and the bot's start once. A client's log then names the seat.
3. **WRTerritory's ground: no fields where nothing reads them.** The walk ground's fields (one a place walked to) are
   read by the anneal's simulator alone; the program, the planner by default, prices its rings by the straight line from
   home (`lp.rs`), so the 446 fields of the start were made and held for nothing. Under the program none is made, but
   for a plan an unsolved program hands to the anneal (28 of 1,195 games to terr337), on the planner's thread. Under the
   anneal the start makes them on `available_parallelism` threads (at most 8: 2.2 s for Glitters Extended's 438 where
   one thread took 17 s), and a field asked for while another thread makes it is waited for, not made twice, with no
   lock held while one is made (the map's lock was held through each search, so the anneal's threads queued behind one
   another).
   Agreed first as "the smaller distance maps", built, checked and rejected (2026-10-27): fields over 32-elmo cells
   took a quarter of the time and memory (8.1 ms and 0.6 MB a field on Glitters Extended) and walked within 1 % at the
   median on five maps (home to every spot, every spot to its eight nearest and eight others; the same pairs reached on
   either grid), but no rule for a coarse cell's passability held Quicksilver: passable at two of its four cells, a thin
   wall opened (30 pairs up to 2,704 elmos short); at three or four, a narrow passage closed (50 pairs up to 3,600
   long, 2.1 times). The check was a `buildorder ground` subcommand, deleted with the coarse grid. Also rejected: u16
   costs in `terrain::Field` (a shared-crate type the flagship's routing uses at 16-elmo cells, where u16 tenths
   overflow on a big map); a matrix of place-to-place walks (the simulator walks from where a builder stands).
4. **WRTerritory's T-RACE off the engine thread, and its replan request cheap.** T-RACE's inputs (our standing
   buildings by region, our armed units with their classes, the sensors, the starts, the cost layers shared) are read on
   the engine thread and its fields made on a thread of its own; its line and its warnings (T-RADAR's input) are taken
   up at the first tick after it ends, the line naming the frame it was taken at (`of f=`, which `run/territory_race.py`
   reads), and no race starts while one runs. The replan request's 95-373 ms were all one call, measured by part
   (lagprobe3-ext): `plant_rings`, the turret sites each plant's ring could take, packed by picking the free 16-grid point
   nearest the plant again and again, every candidate tried at every pick; it is the minute `turrets:` line's too. A
   site taken only makes the others less free, so one pass over the candidates nearest first packs the same sites, each
   tried once; the two agreed at every call over 24 games (terr338-343, both run side by side). Rejected: moving the
   request into the planner's thread (the packing reads the farm's rules, the dead ground and the hands' state: a copy of
   much of the bot for what one pass removes).
## How it is judged

- The lag probe again (2 WRTerritory + 2 WRDistilled against 2 BARb on Glitters Extended, 20 min): frame 0 and the
  per-seat minutes and slow calls against lagprobe1-ext.
- WRTerritory's economy bench against NullAI (`run/territory_bench.sh`, six starts, four seeds), before (terr332-337,
  HEAD with the timer, which logs only) and after: T-RACE's warnings come a tick or more later and the replan's request
  changes thread, so the curves must hold.
- A duel with spawns (the cheat at the give).
