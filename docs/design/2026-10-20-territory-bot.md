# Target design: WRTerritory, a bot that plans its economy by search over the simulator

Written 2026-10-20 at the user's direction, before the first line of code: "a new heuristic bot ... a high-level
decision process dedicated to optimizing metal production across a basic algebraic model for how the game works.
We can start with arena games against InactiveAI where the goal is to simply optimize metal income over the course
of the game. Once we have a satisfactory result there, we can try to add a simplified model of how military units
on the map are able to defend territory and try to balance the additional dimensions in the same system." The shape
below was agreed the same evening ("Agreed with this shape, go ahead and build it"). A bot of `bots/` (the
heuristic regime, `bots/README.md`): nothing of it goes into `crates/bot`.

## What is true today (read from the code, 2026-10-20)

- `crates/buildorder` is the model: a fixed-step (0.5 s) simulator of the economy over a game's own unit numbers and
  ground (resources with storage, extractors paying their spot's amount, generators, converters, factories,
  constructors, construction turrets, assist and help steps, stalls shared by every build, walks over the map's own
  slopes), and a simulated annealer over per-builder queues (`anneal_within`: the best plan in a wall budget on N
  threads, warm-started). Calibrated to about ten percent to minute 6 on quiet games; 15-25 % high on income from
  minute 4 in player games and never calibrated past minute 6, because real games lose units
  (`docs/studies/build-order.md`). One evaluation costs 0.2-0.4 ms for a 20-minute horizon from the start.
- The flagship had a rolling planner over it on 2026-09-21 (`docs/design/2026-09-21-rolling-planner.md`): a
  snapshot of the live game as `sim::State`, a re-plan every 30 s on its own thread, the plan re-based on what began
  since the snapshot. It won the economy and lost the early army; the fix was hand-written expectation curves in
  the objective. The rolling search was deleted with the heuristic bot on 2026-09-25; the snapshot code remains for
  the player's `plan` and `search` tools (`crates/bot/src/brain/planner.rs`).
- InactiveAI is the lobby's name for the engine's NullAI (BAR's scenario scripts seat `ShortName = NullAI` under
  that name). The arena seats it as `--opponent NullAI:0.1`; the smoke game nullai-smoke-1 (the distilled bot, Comet,
  seed 1) ran: his commander stood still, ours reached 31 extractors by 10:00 and killed him at 12:00. A bot that
  builds no soldiers runs to `--max-minutes`.
- A bot of `bots/` gets the `Hello` (unit numbers, metal spots with amounts, the terrain) and a `Tick` (own units
  with idle and health, the stocks and incomes, events) and answers with commands; its `tick` runs on the engine
  thread. It writes no record; its instruments are its own lines in the engine log.

## Decisions

1. **The decider is a rolling-horizon search over the model.** Every `replan` seconds (30), and when a factory or
   constructor of ours is finished (at least 10 s apart), the bot takes a snapshot (`State`: stocks, standing
   economy, each builder with its place and job) and anneals a plan from it on its own threads (`budget` 400 ms,
   `threads` 4), warm-started from the plan in force with the steps already taken removed. The first plan is
   searched on the engine thread at the first orders (1 s of wall once: the engine waits, and twenty game-seconds of
   idle commander at speed 50 would cost more than it). While a search runs the builders keep the plan in force;
   when it arrives, each builder's queue is re-based: a leading step whose unit is what the builder has started
   since the snapshot is skipped.
2. **The objective is metal made.** `Objective::Made { terminal }` (new, `anneal.rs`): the sum of every simulated
   second's metal income over the horizon, plus `terminal` seconds of the income at the horizon; no stall penalty,
   no army term, no expectations. The horizon is the game's end (`end`, 1200 s) less the snapshot's clock, never
   under 120 s; `terminal` is 0 for the score. With the horizon the whole remaining game there is no window-end
   artefact to patch, and converters count because their metal is metal.
3. **The palette is the roster with the soldiers off.** `Palette::roster`: every economy building, factory and
   nano the commander or any reachable constructor builds, assist and help; the factory list cut to builders
   (`soldiers=off`). Tier 2 is in the space by construction (advanced constructors, advanced extractors as upgrades,
   fusions, converters).
4. **The scenario is our half.** The spots nearer to home than to its mirror through the map's centre
   (`Game::own_half`; `spots=all` for the ceiling), sorted by distance; constructors with an empty queue take the
   nearest free spot; no leash, no tax; construction turrets feed the first factory, as the hands have them do.
5. **The hands mirror the simulator** (T-SITES, T-ORDERS). One order per builder from its queue; an order stands
   until its frame appears (the engine's `UnitCreated` naming the builder) and stands finished, or comes to
   nothing: refused by the host, rejected by the engine, dropped (the builder idle again within 3 s) or held two
   minutes with no frame. A step that comes to nothing is tried once more at another site, then skipped. Sites:
   an extractor on its spot; an upgrade over the standing extractor nearest the builder; an economy building beside
   the builder when it stands within 600 of home, else on the base spiral the simulator uses; a factory 350 toward
   his start from home with its facing and exit lanes kept (the distilled bot's placements); a nano behind the
   nearest factory; `assist` is a guard on the first factory for 20 s; `help` is repair on the frame with the most
   metal left. Factories take one unit at a time. Nobody is set to low priority: the simulator shares a stall
   over every build, so the hands do too (a calibration item, not a rule).
6. **Instruments.** `curve:` a minute (extractors, constructors, factories, nanos, build power, both incomes and
   stocks, metal made so far as the integral of the engine's income); `plan:` per re-plan (the snapshot's clock,
   wall, evaluations, score, the predicted income and extractors at +60, +120, +300 s and at the horizon, then the
   plan one queue a line); `begun:` per frame with the order's age and the builder's distance; `dropped:` per
   order that came to nothing. The dump `territory-<ai>.json` (the `Hello`, the commander's type and home) in the
   match directory (`WITHIN_REASON_TRUTH_DIR`) for the oracle.
7. **The oracle and the score.** `buildorder optimize --hello DUMP --objective made --minutes 20 --palette roster
   --builders-only` is the ceiling from the same start (offline, 40,000 x 8). `run/territory_curves.py` prints per
   game the realized curve at 5, 10, 15 and 20, the metal made, the oracle's curve beside it, and the search's own
   predictions against what came: the first gap (oracle less planner, in the model) is search regret, the second
   (planner's prediction less the engine) is model error. Against InactiveAI the second is clean for the first
   time, since nothing dies.
8. **Stage 2, not built:** his reach as a field from his seen units and turrets; each spot's income in the model
   times its survival given that reach and our cover near it; turrets and soldiers at a station as steps whose only
   value is the income they protect. Objective terms and step types in the one search, not a second decider.

## Judged by

The curve against the flagship's medians on Comet (extractors 9 / 20 / 29, income 22 / 49 / 69, constructors
4 / 10 / 12 at 5 / 10 / 15; `run/flagship_curves.py`) and against the oracle; the two gaps; the ledger rows
`terr*`. Four games a row (seeds 1-4, Comet +0, `--max-minutes 20`).

## Status

- 2026-10-20: written; the smoke game played; nothing of the bot built.
- 2026-10-20 night, built as decided (`bots/territory`, `Objective::Made`, `Game::from_hello`, `buildorder optimize
  --hello`, `run/territory_curves.py`) and measured over four batteries against NullAI (the README's results):
  the hands reach the model's ceiling to 10:00 within about ten percent and run 15-35 % behind it from 10:00 (the
  tier-2 energy and the mohos slower in the engine than in the model). Found and fixed on the way: the first search's
  cost probe inflated by the ground's fields (pre-warmed), re-basing on one step only (now on every frame begun since
  the snapshot), unit types the engine never builds kept in the palette (T-STRUCK), the floating units offered on a
  map with puddles (named sea-bound in the roster), and the Cortex tier-2 extractor refused on seat 0 (the host snaps
  extractor orders to the build grid: a change to the shared host, for the flagship too). Open: the tier-2
  calibration (fusions and advanced solars begun 50-120 s after their order, no nanos on the big frames), the
  twenty-second staleness, the bank the objective ignores.
- 2026-10-20 night, the flagship on the task (the user: "run a test with the real within reason bot at this task"):
  income 36 / 177 / 463 / 802 at 5 / 10 / 15 / 20 on 78 of 80 spots with 97 constructors (terr-flagship-nullai-1).
  The bot given the same room (terr5-all: every spot, forty constructor queues) is its equal to 10:00 and then
  199-321 at 20:00 against the model's own ceiling of 661-738: the rolling search misses the tier-2 branch in half
  the games (an advanced plant, its constructor and the mohos are a valley the 400 ms warm-started anneal does not
  cross with forty queues), while the ceiling sits within a tenth of the flagship. The two gaps as decision 7 meant
  them: the model error is small; the search regret is the gap. Next: a chain-seeded restart beside the warm start
  (`Palette::chain_seed`), or a longer search at a lower cadence; then the tier-2 hands calibration.
- 2026-10-24, in the `territory` worktree (the user: continue iterating; first get it running and verify the
  worktree layout): terr37, terr36's code as a control, 811 / 724 / 691 / 779 at 20:00 against 691 / 695 / 729 /
  693, nothing broken, and the two batches put one batch's 20:00 noise at about ±60. Then the mid-game, read from
  the energy curve: every game of every batch stalled on energy two to five minutes between 5:00 and 11:00, the
  extractors' output a quarter down and every build at half speed through it, half the constructors on advanced
  solars of 5,000 energy. The program's reserve was not the lever (terr38, half the storage: no change; a reserve
  is a stock and the hands' overshoot is a flow). The budget was: it credited every frame as finishing inside the
  window at full speed, and in a stall the frames crawl (terr39: the credit at the stall share, half the games
  fixed; terr40: one budget in a stall, since a producer's own net read ninety seconds of income as free; terr41:
  no credit at all in a stall). terr41: 131 / 137 / 174 / 75 at 10:00 against terr37's 112 / 110 / 95 / 112. Then
  the program's energy margin (production a quarter over the build draw from the second bucket, `emargin`,
  terr42): the first batch whose worst game at 10:00 is above 120 (133 / 142 / 193 / 124), 20:00 unchanged at
  about 750 (the late bank, 30,000-39,000 unspent, is what stands between 750 and the flagship's 802 now, and
  between that and the ceiling's 1,650). Crediting the production of energy frames about to stand cut the
  refusals (terr43: 34-47 a game against 84-202) and let the stalls back in (terr43, terr44: the frames taken at
  full speed while the metal stood at zero), so it is out again and the frames' realized pace is the next step for
  both credits; a guard on a factory's exit lane moves aside first (terr43 game 01: the plant blocked from 7:06 to
  11:42, 526 at 20:00; terr44: no factory order held). The `ordered:` line names the
  dispatch rule behind every order, which is how each of these was read within a minute of the batch.
- 2026-10-24 evening, the benchmark example (the user: "I have a new example. Look in my game client's replays
  for the comet catcher game between thebluegecko and InactiveAI on oct 21st"): his lobby game of 2026-10-21
  (Cortex, OS 34, 21:42) replayed through the headless engine (`run/matches/1792859881-replay-gecko-inactiveai-
  comet-2026-10-21`, `run/eco_marks.py` the reader, new): income 62 / 213 / 651 / 1,706 at 5 / 10 / 15 / 20 and
  2,270 at 21:00, twice the flagship's metal made at every mark and twice the bot's income from 10:00. The
  README's paragraph has the shape; what it says for this design: (1) the opening is three extractors before
  anything and the constructors on extractors only (26 at 5:00 with nine constructors; the bot's hands put a solar
  first at 0:07 and the lab at 0:24, the first extractor at 0:52, and its lab makes 43-45 constructors a game
  where his made nine), and the program's own 5:00 is 21 extractors, so the early marks are the model's (its walk
  constant of 65 s an extractor is the bot's realized walk, his was 55 s with the build); (2) the late economy is
  advanced fusions and advanced converters built in a nano farm of 300 turrets fed by air constructors, no basic
  converter at all, where the flagship, the bot and the program's ceiling are all basic converters on solars and
  fusions; the same program with `--nanos-per-factory 80` answers 2,046 at 20:00 with 114 turrets, 77 advanced
  converters and 17 advanced fusions, so the late mark is the room the arms give the program (the turret cap of
  six a factory, decision 1's caps) before it is the hands'; (3) the energy: an energy storage at 4:06, a fusion
  begun by the first advanced constructor at 6:57, four by 10:10, four advanced solars in all, and the stock
  under five percent of storage for 66 s in the whole game; (4) the reclaim, which the first reading missed (the
  user): the plant reclaimed by the commander at 4:41 once its nine constructors are out (570 metal, 54 s before
  the advanced plant), the advanced plant by its turrets at 10:22 once its fifteen are out (2,600, the air plant
  and the farm's first turrets on it), 17 solars by the aircraft at 13:44-14:41 as the fusions stand (2,550, the
  first six where the farm goes), the four advanced solars at 15:01-15:26 (1,480): the metal of a tier's
  infrastructure is a loan the next tier repays, returned whole and at once, no energy, at a cost of the
  reclaimer's time. For the program this is a column the model has not got: a standing unit taken down for its
  metal and its ground; for the hands an order the protocol already carries (ReclaimUnit) and the bot never
  gives. Nothing built from it yet; the order proposed to the
  user: the turret cap and the frames' site in the hands (a farm: turrets in a block, tier-2 frames placed inside
  their reach), the opening prefix, the constructor count.
- 2026-10-24 evening, the program first (the user: "focus on iterating the LP engine now to try and include the
  extra actions needed to match or beat this new curve"): the reclaim column and the farm arms, in the LP note's
  section of that name. The program beats his curve at every mark given his walk and his room (62 / 316 / 1,058 /
  2,357 from the bot's seed-1 start) and at 10, 15 and 20 with the hands' walk; what the hands owe it next, in
  the marks' order: the extractor chain (the 5:00 mark is the hands' alone: 36 s an extractor against their 65),
  the farm's placing with the turret cap as the arm, the reclaim order. The bot's defaults are unchanged.
- 2026-10-24 evening, the hands (the user: "let's work on improving the territory bot to match"), one batch each,
  read from the two records first: his constructors chain the spots every 31 s with a 528-elmo step and six of his
  nine do nothing else, the bot's every 62 s with a 1,110 step and solars between. T-CHAIN (terr46: 5:00 up 3-7 in
  three games, the chains to 45-64 s and 712-1,207 elmos, half the way, the rest contention and the metal); the
  program's energy margin from 4:00 (terr47: its own 5:00 is 50 without the margin, 40 with it, 48 with it from
  4:00; the bot 36-39 at 5:00); the pros' plant after three extractors tested and refused by the program and the
  engine alike (terr48: 32-34); T-OPENING, the commander's first minute on the spots within 500 of home (terr49:
  45.6 / 47.9 / 38.7 / 41.0 at 5:00 against terr45's 33.0-38.7, every game up, the first batch above 45; the
  program prices the commander's every extractor at one walk of 16 s where these cost six). The bot stands at its
  plan's honest 5:00 now: the plan's 48 counts a tenth of an advanced fusion as 300 energy a second where the hands
  build thirteen to seventeen solars, as he did. The mid-game then took two games of terr49 (20:00 566 / 623
  against 733 / 800): the tier-2 plant committed at 6:20 in both Armada games against 4:55 in both Cortex games,
  and in game 00 to a constructor 2,000 elmos out that walked 88 s to place it; the lab idle while it was framed.
  Next: the plant's commit (its timing by the plan's own counts, its placer a builder at the base), then the farm
  and the reclaim order.
- 2026-10-24 night, the mid-game (terr50-53): the plant's placer at the base and a five-minute commit window
  (terr50: 10:00 145-155 in every game, 846 at 20:00 in one); the program held to the line the first lab opened
  (`planner.rs factory_line`; terr51 read factories from the wrong list, terr52: the Armada plants from 6:30 to
  4:30-5:00, since on its own the program wanted the vehicle line through a vehicle lab it planned a tenth of);
  T-YARD (a factory's own product never guards it, six guards a factory, a builder with nothing to do steps out
  of the yard; terr53: no product held in any game, 15:00 358-403 and 20:00 761-850, the best batch yet, the
  standing state). Against the flagship's 36 / 177 / 463 / 802 and his 62 / 213 / 651 / 1,706: level with the
  flagship at 5:00 and 20:00, behind at 10:00 and 15:00 by the plant's timing and the converters, a third of his
  at 15:00 and half at 20:00. Next: the farm (turrets in a block, the tier-2 frames inside their reach, the
  program's `maxnanos` and `help` arms on) and the reclaim order, which are base layout, the user's call.
- 2026-10-24 night, the farm (the user: "build the layout engine the way you see fit to start, and we can iterate it
  if there are problems when we run it"), T-FARM over four batches with the program's room on (`maxnanos=300`,
  `help=40`): a sunflower of everything in one sequence round home (terr54: every big frame with 0-5 turrets in
  reach, 736-763 at 20:00); a lattice of turret rows and building rows from an origin on the flattest ground off
  his line, the core's frames at the slot with the most turrets in reach (terr55: 20-33 turrets within 400 from
  15:00, but the host's ring walk put a fifth of the orders 200 off their slot where a parked constructor stood,
  the engine dropped the later orders against the first's building and struck the basic converter in game 00:
  612-846); the slot asked for exactly (`BuildSite.exact`, a protocol field the host honours) and the crowd's
  slots held (terr56: the mechanism gone, 702-794; the cost the walking, 29-36 s a base order against terr53's
  3-6 s); streets through the block, the core filled from home's side, cheap buildings beside their builder
  (terr57: terr57 stands at 45.6 / 45.6 / 41.0 / 36.4 at 5:00, 147 / 168 / 137 / 132 at 10:00 (terr53 140-160), 359 / 389 / 345 / 320 at 15:00 (358-403) and 818 / 912 / 912 / 896 at 20:00 (761-850): every game above the flagship's 802 and the best batch yet, a base order 9-12 s to its frame (terr53 3-6, terr56 29-36), 244-322 basic converters a game, 66-77 turrets standing at 20:00 with 41-57 of 73 busy at 18:00 and 0-28 at 20:00, the frames under way then nearly all beyond every turret's reach (converters beside their builders, extractors at the spots): the turrets' idle half is the next lever.) The turrets' use is in the log's `turrets:` line.
