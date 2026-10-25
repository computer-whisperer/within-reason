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
- 2026-10-24 late night, scaling toward his curve (the user: "keep scaling this -- using thebluegecko's game as the
  goal to beat"): his shape by minute read from his record (all 75 spots by 9:00; turrets 29 at 10:00 and 112 at 15:00;
  the advanced plant's constructor every 20 s under them; an air plant at 10:27 whose 46 air constructors placed 276
  turrets; 22 advanced fusions in his last five minutes, built in seconds). terr58-61 on the hands' allocation: within
  the noise (the turret-help restriction worse, the power-seconds sort a help flood until the Frames credits were
  read from the mobile flows alone). terr62 the vehicle line: 15:00 367-423, 20:00 881-980 (T-LINE, the default).
  terr63-64 converters in the turrets' reach: the rule found no slot; the bank term: hoarding; both reverted. terr65
  the horizon at 25:00: his composition at 20:00 (turrets 142-163, advanced fusions 11-13, advanced converters 24-30)
  and the marks unmoved, the engine two to four minutes behind his; the baseline from here. terr66 the air plant: was offered and never taken (the program planned a fraction of one, which cost 10:00 twelve to thirty in all four games; reverted). The standing state is terr65-66's code: 5:00 32-34, 10:00 128-163, 15:00 351-411, 20:00 843-991 against his 62 / 213 / 651 / 1,706, the composition at 20:00 his (turrets 126-181, advanced fusions 9-14, advanced converters 24-30) and two to four minutes behind his. Then the ramp (T-PLANT, terr67-69): a turret's assist on a factory in the program and the hands placing such turrets round the plant. On every factory's products and against the plan's fractional factories (terr67) the program pumped basic constructors from the vehicle plant and held the advanced plant at a fraction to 7:50-9:00, 5:00 up 7-10 and 15:00 down 50-100 in all four games; bounded by the factories standing (terr68) the same; for advanced plants' products only (terr69) the plant was back at 5:41-6:18, the turrets stood 200 from it when asked, and the program asked for 1-2 turrets' worth at 10:00 and 3-6 at 14:00 where his plant had 29 at 10:00: level at every mark (32-34 / 137-165 / 353-397 / 871-927), the arm off by default. The turrets' place is built; what is small is the program's demand for advanced constructors (10-12 by 11:00, one every 16-20 s, bounded by its metal and its upgrade pace, which starts at 11:00 after the last spots where his started at 9:00). The open levers: the spots by 9:00 (his nine constructors took 75 in eight minutes; the bot's 25 take 60-65 by 10:00, the chain's contention), the upgrades from 9:00, the reclaim order (`reclaim=off`), and the program's optimism at +300 s (+150-180 a second, fractional production).
- 2026-10-25, the chain's far end (the user: constructors building one extractor, starting home, turning round for
  the next spot; "fix it, instrument first then the rule"). The instrument, one `order:` line per order given to a
  builder (terr70, the standing code): constructors beyond 2,000 of home were given 17-30 guard chunks a game (a
  77-90 s walk for a 20-s chunk that ends while they walk), 8-13 turrets on the farm (98-127 s) and 3-10 frames at
  the base; 31-46 turn-arounds a game, 55-74 constructor-minutes of walking toward home, 16-24% of a constructor's
  time between frames on neither building nor walking. The mechanism: T-CHAIN held only while an extractor flow had
  credit, the credit drains when fifteen finish within a minute of each other around 9:00, and the next largest
  credit was at the base with nothing weighing the walk. The rule (terr71-72): a far constructor without extractor
  credit keeps the rings' flows and the cheap buildings beside itself, takes no turret, guard or frame at the base,
  and after the flows takes the nearest free spot or upgrade nearer than home (`chain_on`); the plant's frame
  helpers from the base only. Ahead of the stall's solar (terr71) it starved the energy; behind it (terr72) 5:00,
  10:00 and 15:00 up in all four games (46 / 39 / 41 / 39, 175 / 174 / 151 / 170, 464 / 416 / 417 / 457 against
  34, 135-154, 352-432) and 20:00 three of four (899-1,040). The energy still runs dry at 5:00-7:00 in two games
  (the extractors outrun it): the far constructors' energy is the next item.
- 2026-10-25, the opening (the user: "Focus on the openings: any amount we fall behind in the first 5 minutes is
  basically unrecoverable", and thebluegecko's 62 at 5:00 is above the program's own ceiling). Read first: the
  program's plan from the seed-1 start promised 45 at 5:00 with 19 extractors, the hands' figures exactly; at the
  realized 15-s extractor walk (12-19 s through 9:00 once the constructors chain; the 65 was measured when they
  walked home between spots) it promises 59 with 25 extractors and 8 constructors, his 62 / 26 / 9, and the
  annealer's five-minute opening is 59.8 with 27: his 5:00 is above both models. The constructors chain at full rate
  from their first order (a frame every 33-36 s); the plan was the limit, then the constructors' minutes. Eleven
  batches (terr73-83, ledger): the walk alone changed nothing the hands could use (the three constructors on the
  advanced plant from 3:40 were the missing ones); the plant committed on the bank (`commit_afford` 45 s of income;
  90 before) with its frame's helpers from anywhere (`plant_helpers_from`) stands at 5:20-5:26, his 5:35, built in
  57-88 s; the commander kept home (`com_leash` 700, its walk priced at 40) builds its solars from 2:46 and guards
  the plant on a metal budget (two of the product in the stock and twenty seconds of income; on the stock alone it
  never guarded, ungated it starved the stock to zero for two minutes); a constructor with a spot's credit leaves
  the stall's solar to the others (its first extractor at 1:48 against 2:33); the first-minute variants (four spots,
  his three solars before the plant, one spot) all lost 5-15 at 5:00: the plant at 0:32 after two spots stands, as
  terr48 found. The extractor walk grows with the plan's t0 (15 to 40 at 10:00; one 15-s walk cost 20:00 40-70).
  The standing state (terr83, the defaults): 5:00 43.3 / 43.3 / 46.8 / 43.3 (terr72 45.6 / 38.7 / 41.0 / 38.7),
  6:00 68.6 / 66.3 / 68.6 / 61.7 (59.4 / 54.8 / 55.9 / 52.5, up 9-13 in all four), 8:00 98-110, 10:00 149-176,
  15:00 404-517, 20:00 962-1,007. The gap to his 5:00: his seven constructors by 3:16 against the hands' by 4:00
  (his plant, fed and guarded, one every 17 s; the hands' one every 27-30 s on a stock at zero from 2:00 to 3:30)
  and his third extractor at 0:25 (the hands' third spot is 562 out) compounding from there.
- 2026-10-25, the start (the user: "set the games to place the start point matching his -- or at least on a triple
  to even things out"). Comet Catcher Remake has no triple: the best starts hold two spots within 250 and a third at
  500-640; his start (941,2534, read from the replay's record) has 166, 251 and 537, the arena's south-west start
  118, 312, 562 and 631. The benchmark runs from his start and InactiveAI's from here (`arena --starts
  941,2534:6899,684`). The bot had taken his start from the other start box's centre, which with pinned starts is
  the seat's box, not the start: as the second seat its two nearest spots fell inside the dead radius round "his
  start" (terr84: no extractor in the first minute, 17 a second at 5:00); it now trusts the boxes only while its own
  holds its commander, else mirrors home through the map's centre (terr85). From his start the hands' opening reads
  level or a little under the south-west's (5:00 43.3 / 38.7 / 41.0 / 38.7; 6:00 68.6 / 54.8 / 64.0 / 61.7; 20:00
  942-1,020): his 62 at 5:00 is not the map's.
- 2026-10-25, his opening (the user: "Does the LP actually prefer a 2 mex start still on that location?
- 2026-10-25, the second minute (the user: "continue to focus on early divergences first and track down why"): his record's per-second builder targets show his commander guarding one constructor (1:24-1:33, the energy stock 1,237 to 94), a solar, then a trip at 1:40 to two spots 1,007 and 1,215 out and 210 apart with two solars beside them, back guarding at 2:57, his third constructor guarding the fourth meanwhile; the hands' commander spent that minute on five solars and a 20-s guard with the stock at zero from 2:45. Built and kept: the program's commander extractor pair priced at the mean walk of the free spots within its leash and bounded by their number (`commander_spots`, the constant walk and its arm gone), the far commander chaining a leashed spot nearer than home (`com_far`), and `buildorder lp --game RECORD --from S`. Measured with the leash at 1,300: one spot a trip with the guard's credit walking it home between (terr90), one spot since the second constructor claims the pair's nearer spot at 2:06 before the commander sets out at 2:19 (terr91, level), and the pair by 2:22 with no guard before it, a third spot 1,850 away and the plant unguarded to 3:00 when the trip went ahead of the guard and the solars with ten-second guard chunks (terr92, 5:00 down 7-18 in all four; reverted). The leash stays at 700 (terr93 at terr89's figures). Open: the sequence guard, pair, guard on an energy stock run to the floor with the pair claimed before the second constructor's birth and a constructor guarding meanwhile; the first minute's walking (the hands' extractors on the spot points where his sit 93 nearer on each patch's edge, the solars 142-352 apart against his 82-102), his plant at 1:01 against 1:14.
- 2026-10-25, the builder-priority flag (the user: "Is this about construction priorities? Do we set builder priorities correctly in the territory bot?"): nobody of ours was low (the README's old T-PRIORITY: the simulator shares a stall over every build), so the engine's creation order fed a stall to the commander's solars first, the plant second and the constructors' extractors last (24-32 s an extractor frame against 20 at 2:37-3:03 of terr89 game 00); his record's flags: the plant low from birth, the constructors high, the commander high through the opening and the trip and low from 2:50 to past 7:00. Built (T-PRIORITY, `run_priorities`): plants and construction turrets low from birth, constructors high, the commander low from its first guard onward (`com_low=after`, the default; `guard` for low only while guarding), read back each tick and sent when it differs. Low only while guarding halved the stall's cost on the extractors and read level (terr94); low from the first guard onward took every extractor frame to its 20 s and moved 5:00 to 54.8 / 54.8 / 52.5 / 50.2 (terr89 47.9 / 45.6 / 47.9 / 50.2) and 8:00 to 108-128 (92-115), up in all four games (terr95). The program still shares a stall over every build, so in a stall its model error now falls on the plant's side.
- 2026-10-25, the hands' far fallbacks (the user, watching terr100 game 00: "At 10:43 we are completely full on metal
  and energy, and yet we only built two construction turrets for the first advanced fusion?"; "Build it, one batch").
  Read: the plan had asked for eleven more turrets within a minute; the count rule's turret orders went to constructors
  4,500-7,300 elmos off (87-144 s), whose orders covered the count and the credit for the constructors at the farm; the
  metal at its cap at 10:00-11:00 in every game of terr100 and terr97; his 29 turrets at 10:00 on a bank spent to 88 (he
  too was at his cap at 7:00 and 8:00). Built: a constructor's count rule and dearest frame reach 45 s of walking with
  the farm in reach, else a Walk to the farm's core that covers nothing and is re-offered the flows every 15 s
  (`run/dispatch_latency.py` reads the rules' walks). terr101 (the walk ending at the reach's rim) down 47-166 at
  20:00 in all four, the advanced constructors trapped on far metal makers; terr102 level with terr100 and the walks
  gone. Not moved: the turrets at 10:00-12:00 and the float, since the base constructors never reach the count
  fallback: the flows rank by power-seconds and the plant's guards and the moho help outrank the turrets until the
  spots run out. Next: instrument the credits at dispatch; the ranking (proportional service of the plan's flows
  rather than the largest first) is the turrets' lever, a design question.
- 2026-10-25, the cross-map baseline (the user: "Let's work on making this more general ... How canned is the current
  economy growth build? We will need to find a way to let it adapt to different terrain and allowed territory
  expansion"; "Run the baseline with those maps"): the bot unchanged at aa48726 on Comet from the map's own starts,
  Great Divide, All That Glitters, Mithril Mountain and Quicksilver, four seeds each (terr103-107;
  `run/territory_geometry.py` reads each map's spots, spacing, water and farm ground from the dump). Against the
  program's ceiling from each start the new starts read 0.47-0.61 at 5:00 (0.18-0.19 on Glitters north) and at 20:00
  0.48-0.50 on Comet, 0.33-0.42 on Mithril and Quicksilver (Mithril game 00 0.11), 0.24-0.27 on Great Divide and
  0.01-0.16 on Glitters, where thebluegecko's start on Comet reads 0.69-0.73 and 0.54-0.58. Hard failures: a vehicle plant facing a rock with its constructors
  held on the pad and the commander leashed to five spots (Glitters north, 11.2 a second all game); an advanced plant
  whose order cycled between two unreachable sites for fifteen minutes (Glitters south, one factory all game); his
  start taken as his box's centre on a box the map's width, our constructors and then our commander in his
  commander's range (Great Divide, a commanders' duel won and one lost at 16:00). Softer: the program prices both
  Comet starts identically to the unit (one ring, a fixed walk), and from the map's own start the commander takes a
  fourth spot and the plant stands 27 s later (5:00 34-41 against 48-55); the late economy floats on a windy map
  (Mithril game 00 at its metal cap 8:00-20:00 on 45-51 wind generators a minute); the spots under water are walked
  to and never taken (Quicksilver).
- 2026-10-25, the plant ramp in the program (`docs/design/2026-10-25-territory-plant-ramp.md`; the user: "Proceed. Start with batch 1 alone, and the mobile guard's cap stays until the pad is calibrated"): the turrets' assist on a plant bounded by its ring's capacity as the hands count it (built, terr96: the ring 24-34 at the advanced plant, his 29; the program's demand unmoved, 3-4 turrets at the plant by 14:00; the marks level), the advanced constructor's spots priced from home (built, terr97: level, the premise wrong: the hands never sent one to a spot, and the plan's upgrades wait on the spots and the metal, not the price; kept as the honest price), and the pad's roll-off as the plant's own ceiling in place of the caps (not built: a cap, not a demand; what holds the program's demand down is open, the farm's timing against his the thing to read next).
- 2026-10-24, the advanced constructors' count (the user: "Why are we making so many advanced constructors? Does the LP not model using a t1 constructor to help build a t2 mex?"; "Build it, one batch, with a low help factor"): the program had no pair for a basic constructor on an upgrade frame (a mobile builder's pairs were its build list and a factory's products), so every upgrade was priced on the advanced constructors alone and it bought 29-32 by 15:00 against his 15 (and 10 advanced aircraft). Built as `LpOptions.mobile_help` (row 3d, `mhelp` 0.35: his basic constructors' share of the moho frames' builder-seconds), the hands reading the help from this type's own flows and choosing the frame by the finish counted from the walk. terr98 invalid (the hands' flag read off any type's flows, the extractors fell to the count rule), terr99 the help bounced between nearly-finished frames and 20:00 fell in all four, terr100 level at every mark with 2-7 fewer advanced constructors by 20:00: kept as the honest model. The count is the plant's cadence and the upgrades' return, not their price in advanced power.
  thebluegecko's better start at that location tells us something must be wrong"). The program never chose the two
  spots: T-OPENING's radius did. Asked, the program's fluid opening from his start is a third of a lab, solars and
  no commander spot (the commander's extractor priced at one 40-s walk whatever the distance, so its 166 and 251
  read as 1,500 out); the annealer's is one spot, the plant and three solars; and the simulator, given his order
  against the two-spot order with everything after held the same, prefers the two (41 against 50 at 5:00) and
  leaves 500-2,000 metal idle under his, since none of the three guards the plant on the bank as he does. In the
  hands his order stands (terr86-89): three spots, solars before the plant while the supply is under its draw,
  the plant at 1:17 guarded on the metal and energy budgets, five constructors by 2:42 as his, and the constructors
  chaining spots from the start for three minutes (the first constructor's first order had been the plan's solar).
  From his start: 5:00 47.9 / 45.6 / 47.9 / 50.2 against the two-spot opening's 43.3 / 38.7 / 41.0 / 38.7, 6:00 and
  10:00 up in all four games too. What is wrong, then: the models price the commander's near spots as far ones and
  value no bank, so they never propose his order; and at 3:00 his 20 a second against the hands' 13.5 is his eight
  extractors against five, the constructors' first spots 740-1,040 out against his 300-500.
