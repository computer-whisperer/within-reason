# WRTerritory: a bot that plans its economy by search over the simulator

A bot of `bots/`: code decides everything, no model is asked, a game costs nothing (`../README.md` for the regime).
Its design is `docs/design/2026-10-20-territory-bot.md` (the user, 2026-10-20: "a high-level decision process
dedicated to optimizing metal production across a basic algebraic model for how the game works"). The model is
`crates/buildorder`'s economy simulator; the decider is a rolling-horizon search over it; no rule here names a unit.
Any faction, any map with metal spots; one seat. Stage 1 (this): metal against an opponent that does nothing
(`--opponent NullAI:0.1`, the lobby's InactiveAI). Stage 2 (not built): his reach and our cover as terms of the
same search.

## What decides

| ID | What | Values | Why | Code |
|---|---|---|---|---|
| T-LP | The planner is a linear program over the simulator's accounting (`docs/design/2026-10-20-territory-lp.md`; `buildorder::lp`): a snapshot of the game (stocks, standing economy, every builder with its job) solved every 30 s on the planner thread, and at the first orders on the engine thread, for the build-power-seconds each builder type spends on each unit type in buckets of 10 s for the first minute, 60 s to the fifth, 120 s after, to the game's end; the most metal made. Offered the types the hands could start now (metal cost within the stock and 90 s of income: a fraction of what cannot be started is energy and build power the engine never has, terr21), plus whatever stands or is under way; a factory ordered or framed stands in the snapshot with the rest of its cost drawn from the stocks (`State::committed`: the first lab is a lump the plan's 0.4 of one by 1:00 does not know, and terr22 built four solars beside it and starved for three minutes). Bounded by the hands' capacity (forty constructors of each tier, four factories, six construction turrets a factory, turret help at most three times the builders' direct power, assists at most twice the factories' own power; every cap loosened to what already stands, since a cap bounds what the plan adds and a first bucket past one has no feasible point: terr19, 27-34 solves a game `PrimalInfeasible`) and by the walks measured from the `begun:` lines (a constructor's extractor 65 s, its other building 3, a factory 40; the commander's 16, 5, 9). A solve costs 150-450 ms of wall, 4-9 game-seconds stale at speed 50. `planner=anneal` keeps the first design's search | `planner=lp`, `dt=10`, `mid_dt=60`, `coarse_dt=120`, `help=3`, `nanos=6`, `assist=2`, `walk_*`, `com_walk_*` | terr5-all: the anneal missed the tier-2 branch in half the games; a chain of prerequisites is a valley a sequence search does not cross, and the accounting is linear in build rates | `planner.rs`, `lib.rs` `request`, `crates/buildorder/src/lp.rs` |
| T-DISPATCH | An idle builder follows the program. The lumpy things go by class: the factories as a whole, and a factory's products by role and tier (an advanced constructor builds an upgrade), start a unit when the program's count of the class runs half a unit ahead of what stands, is framed or is ordered, and the type started is the one of the class the program wants most; a builder that cannot build it leaves the class to one that can, and a factory that does not make it substitutes its own best of the class only while no factory standing, framed or ordered makes it. Everything else goes by the program's flows for the builder's type: its credit on a unit type is the power the program has the type spend on it to the clock plus 45 s, less what the type has committed to it since the plan was adopted (the standing orders' unbuilt cost seeds it); the largest credit as a share of the unit's cost is taken, however small, since standing idle is never what the program planned: a building built where T-SITES puts it, a factory product as a twenty-second guard on the factory that makes it. With no credit, the largest count shortfall of a type it builds; with none, help on the frame with the most metal left per builder on it (terr22: 48 on one frame while five fusions stood with a builder each). A builder of a type the plan had none of at its start goes by the count shortfalls until the re-plan its coming asks for arrives. Energy has a backstop class: while the energy stock is under half its storage, the program's planned energy income at the lead, less what stands, is framed or is ordered, is realized by the best affordable energy building of the program's types the builder builds (energy a second per metal), by any builder with at most three orders pending; the flows carry energy like anything else since terr22 (excluded, the late program's fusions never came: eight by 20:00 in terr21 with 49,000 banked). Nothing starts that the metal stock and 90 s of income do not cover; a type the host found no site for is left alone a minute | `LEAD` 45 s, `START_AT = 0.5`, `AFFORD_SECONDS` 90, `ENERGY_LOW` 0.5, `ENERGY_PENDING` 3, `TYPE_REFUSED` 1 min | terr13: read by type no count ever ran half a unit ahead, since the program spreads a plant over the factory types it is indifferent between and grows constructors through a fractional advanced plant; the lab stood idle from 7:00 to 14:00 with both stocks at their caps. The credit dispatch fa9723d's message described was never in the tree (its edit scripts asserted before writing; terr9 to terr13 ran terr8's count shortfalls); built as above for terr14. terr15: the lab filled the forty-constructor cap with its own kind while the program wanted the advanced plant's, which made seven and stopped, seven mohos at 15:00 against the ceiling's eighty. terr17: energy at zero from 1:00 to 5:00 with the fusion's fraction carrying the program's energy and the commander on extractors; the plant committed whole at 3:55 on an income of ten. terr18: the energy class drawn from the builder's whole build list picked the geothermal plant, with no vent on the map, 5,500 no-site drops a game; and solars ahead of the flows on a stock of zero left the metal at zero from 2:00 to 8:00 with the energy at its cap. terr19: read by every idle builder it put the commander on eight solars and the first constructors on four advanced solars by 4:31; and the lab substituting its kind through the plant's frame stood 51 against the tier's cap, which left the program no feasible point. terr20: substituting for the program's fractional advanced vehicle constructors the lab ran to 136 once the caps no longer stopped it; two pending solars by constructors alone left the energy at zero to 8:00 | `hands.rs` `dispatch`, `seed_committed`, `order_one` |
| T-COMMIT | A factory type the program wants at all within three minutes, among whose products is one the program wants that no factory standing, framed or ordered makes, is started whole, before anything else, once the metal stock and 90 s of income cover its cost: the first lab, and later the plant of the tier-2 constructors the upgrades need. The program builds fractions of a factory for minutes (0.3 of a lab in the ceiling's first minute, 0.3 of an advanced plant from 7:00 for every moho it makes) and a fraction of a factory makes nothing in the engine | `COMMIT_WITHIN` 180 s, `COMMIT_AT = 0.05` | terr6: targets read 20 s ahead never crossed a threshold for a factory the program wanted slowly, since every re-solve restarts the ramp. terr16: the class rule took the larger fraction each time, four labs stood by 9:48, the caps filled with labs and their kind, and no plan from a later snapshot could hold an advanced plant or a tier-2 constructor: no moho all game | `hands.rs` `dispatch` |
| T-PLAN | (`planner=anneal`) A snapshot of the game as the simulator's state (stocks, standing economy, every builder with its place and job) is annealed into the plan with the most metal made to the game's end, on the bot's own threads, every 30 s and when a factory or constructor comes (10 s apart at least); the first plan on the engine thread at the first orders. The plan in force is the warm start with its taken steps removed; a builder's leading step is skipped when its current order is that step (the re-basing) | `replan` 30 s, `budget` 400 ms, `first` 1,000 ms, `threads` 4, `factories` 3 and `constructors` 14 queues for builders yet to come | The flagship's rolling planner of 2026-09-21 won the economy against the ladder; the search is the model's own answer, with nothing hand-written about tempo | `lib.rs` `request`, `adopt_planned`, `warm_start`; `planner.rs` |
| T-OBJECTIVE | Metal made over the horizon (every simulated second's income summed) plus `terminal` seconds of the income at the horizon, nothing else; the horizon is the game's end less the clock, 120 s at least | `end` 1,200 s, `terminal` 0 | The user's score for stage 1 is metal income over the course of the game; with the horizon the whole remaining game no window-end patch is needed, and converters count because their metal is metal | `buildorder::anneal::Objective::Made` |
| T-PALETTE | The roster: every economy building, factory and construction turret the commander or any reachable constructor builds, assist and help; the factories' list cut to builders unless `soldiers=on`; no turret on offer | `soldiers=off` | Stage 1 has no enemy; soldiers on the list only cost the search moves | `lib.rs` `start` |
| T-SCENARIO | The spots nearer home than its mirror through the map's centre (`spots=all` for every spot), nearest first, less the dead spots within 1,000 of his start or of an armed unit of his in sight (NullAI's commander shoots and follows what comes near; the user's ruling: a radius); a constructor past its queue takes the nearest free spot; construction turrets feed the first factory; no leash, no tax | `spots=half`, `DEAD_SPOT = 1000` | Our half is what a 1v1 can count on and what the flagship's medians are measured on; the ceiling is `spots=all` | `snapshot.rs` `scenario` |
| T-ORDERS | One order per builder from its queue. It stands until the engine names its frame and the frame stands, or it comes to nothing: refused by the host, rejected by the engine, dropped (the builder idle again after the grace), or held two minutes with no frame, counted from the first frame the builder stands within 300 of the site (a builder never that near is given three times that). A step that came to nothing is given once more, an extractor's spot refused for a minute meanwhile, then skipped; a frame begun clears the builder's attempt count; nothing counts before 3 s, when the engine drops everything. A frame let go of is given back by a repair. Every frame begun is logged with its order's age and the builder's distance (`begun:`); every order that came to nothing (`dropped:`) | `GRACE` 45 frames, `HELD` 2 min, `HELD_NEAR` 300, `ATTEMPTS = 2`, `FIRST_ORDERS` 3 s, `SPOT_REFUSED` 1 min | The distilled bot's D-ORDERS: a shorter watchdog took orders from builders in range of their spots; the engine starts no frame while it reclaims a wreck on the site. terr14: the held clock from the order cut every walk past two minutes as the builder reached the 300 ring (130 extractor orders in four games, the begun ages a wall at 120 s); an attempt count never cleared sent a builder's economy buildings to the base spiral from afar after any one failure | `hands.rs` `run_hands`, `review`, `failed`, `frame_begun` |
| T-SITES | Placed as the simulator places them: an extractor on its spot (the step's, else the nearest free one; the commander's within 1,500 of home); an upgrade over the standing basic extractor nearest the builder (the commander's within the leash); an economy building beside the builder when it stands within 600 of home (the simulator charges no walk; with the program's targets beside the builder wherever it stands), else on the simulator's golden-angle spiral round home; a factory 350 toward his start from home, each after the first a quarter turn further round and a dropped order one more, with its facing kept and every factory's exit lane and yard (a disc of 200) kept out of, as is every building site placed and not yet begun (a disc of 70: the host's search knows nothing of sites chosen seconds before); an order's position is the site the host placed (`BuildSited`), where the frame will stand; a construction turret 250 behind the nearest factory; `assist` a guard on the first factory for 20 s; `help` a repair on the frame with the most metal left; a factory one unit at a time; a factory beyond the plan's queues is passed over | `BESIDE = 600`, `COMMANDER_LEASH` 1,500, `FACTORY_YARD` 200, `PENDING_SITE` 70, `NANO_BEHIND` 250, `ASSIST_CHUNK` 20 s | What the hands do must be what the search priced, or the model error is the hands'. terr15: the commander followed its extractor flows spot by spot across the map and died to his commander 7,300 elmos from home; second labs and construction turrets asked for against the first lab's open yard were dropped by the engine (seven and 48 in four games). terr16: 72 economy orders in four games dropped by the engine 5-30 s after the order with the builder idle a few hundred elmos off, forty constructors placing solars beside themselves round one base | `hands.rs` `give`, `free_spot`, `base_site`, `keep_out`, `sited` |
| T-NANOS | A construction turret guards the nearest factory within 330 | `NANO_REACH = 330` | The calibrated replays' turrets fed the first factory; `nanos_help_frames` is off in the scenario to match | `hands.rs` `run_nanos` |
| T-PRIORITY | Nobody is set to low priority | | The simulator shares a stall over every build, so the hands do too; a calibration item, not a rule | |

| T-STRUCK | A unit type whose orders the engine has dropped or rejected four times, more often than its frames have begun, leaves the palette for the rest of the game and its steps are passed over; never a factory, and never for a site the host could not find or an order held mid-walk | `STRIKE = 4` | terr1: the wind generator cannot be built on Comet and the search kept writing it; terr8: converters and construction turrets were struck on site failures round a crowded base | `hands.rs` `failed` |

Instruments in the engine log (`[territory ai=N] f=F ...`), read by `run/territory_curves.py`: `curve:` a minute
(extractors, constructors, factories, nanos, build power, both stocks and incomes, metal made as the integral of the
engine's income, orders, frames begun, orders dropped, steps skipped, plans); `plan:` per re-plan (the snapshot's
clock, the wall, the score, the predicted income and extractors at +60, +120 and +300 s and at the horizon, the
staleness) with the plan one `queue:` line a builder; `jobs:` a minute; `begun:`; `dropped:`; `dump:` the file
`territory-<ai>.json` beside the match (the `Hello`, home, the commander's type) for the oracle:
`buildorder optimize --hello territory-0.json --objective made --minutes 20 --palette roster --builders-only`.

## Arms

`WITHIN_REASON_TERRITORY=key=value,...`: `end` (seconds the horizon runs to, 1200), `terminal` (0), `budget` (ms,
400), `first` (ms, 1000), `threads` (4), `replan` (s, 30), `spots=half|all`, `soldiers=on|off`, `factories` (3),
`constructors` (14), `dump=on|off`.

## Running

```
target/release/arena --our-ai WRTerritory:0.1 --opponent NullAI:0.1 --matches 4 --parallel 4 --speed 50 \
    --map "Comet Catcher Remake 1.8" --max-minutes 20 --play-out --seed-base 1 --label terr-...
run/territory_curves.py terr-... --oracle
```

## Results

Against NullAI on Comet Catcher, seeds 1-4, +0, twenty minutes (ledger rows `terr-smoke-1`, `terr1-nullai-comet-1`,
`terr2-nullai-comet-1`, `terr3-nullai-comet-1`; the curves and the oracle from `run/territory_curves.py <batch>
--oracle`). The oracle is the model's own best plan from the same start, the ceiling with no enemy.

| game (terr3, 2026-10-20) | 5:00 mex / cons / income | 10:00 | 15:00 | 20:00 | oracle income 5 / 10 / 15 / 20 | tier-2 extractors begun |
|---|---|---|---|---|---|---|
| seed 1 Armada | 14 / 7 / 34.8 | 37 / 10 / 91.8 | 40 / 12 / 115.8 | 40 / 12 / 128.9 | 26.5 / 128 / 314 / 425 | 0 (the advanced lab's order dropped twice at its site, then skipped) |
| seed 2 Armada | 13 / 8 / 31.8 | 32 / 15 / 117.8 | 39 / 16 / 252.9 | 40 / 17 / 362.4 | 29.5 / 85 / 299 / 403 | 39 |
| seed 3 Cortex | 14 / 6 / 34.1 | 28 / 14 / 93.8 | 38 / 16 / 255.5 | 40 / 16 / 373.2 | 29.5 / 117 / 378 / 484 | 40 |
| seed 4 Cortex | 21 / 8 / 47.9 | 39 / 9 / 91.9 | 39 / 14 / 173.9 | 40 / 14 / 281.8 | 31.8 / 133 / 312 / 396 | 31 |

The flagship's medians on Comet against BARb (`run/flagship_curves.py`) are 9 / 20 / 29 extractors and income 22 /
49 / 69 at 5 / 10 / 15, under raids. Read from the three batteries:

- The hands reach the model's ceiling to 10:00 within about ten percent, and the bot's own early marks beat the
  twenty-minute optimum's (13-21 extractors at 5:00 against its 12-13: the optimum spends its first minutes on
  constructors). From 10:00 the realized income runs 15-35 % behind the oracle: the tier-2 energy and the mohos come
  slower in the engine than in the model (fusions and advanced solars begun 50-120 s after their order, no
  construction turrets helping the big frames), which is the next calibration.
- Model error, each re-plan's prediction against the income that came: +60 s median -2 to -4 (the engine a little
  ahead of the model), +120 s within 2, +300 s median -3 to +7 with worsts of +140 to +250 where a plan's tier 2 did
  not come on time.
- Re-plans cost 350-770 ms of wall on four threads and arrive 20-36 game-seconds after their snapshot at speed 50;
  the re-basing on frames begun since the snapshot absorbs that.
- Two engine facts found on the way: the wind generator cannot be built on Comet (every order dropped; T-STRUCK
  takes it off the palette after four), and a Cortex tier-2 extractor must be ordered on the engine's build grid, not
  at the position the AI interface reports for the standing extractor (K-mech-tier-2-extractor-stands-on-ours; the
  host snaps every extractor order now).
- The bank is not in the objective: 25,000-36,000 metal stands unspent at 20:00 once the spots are taken. Stage 2's
  soldiers are what it is for.

**The flagship on the same task** (terr-flagship-nullai-1, the user's ask: Opus 5.5 medium as the player, told the
opponent does nothing and the score is metal income): income 36 / 177 / 463 / 802 at 5 / 10 / 15 / 20 on 78 of the
map's 80 spots (77 tier 2 at 20:00), constructors 8 / 28 / 87 / 97, 111 converters. The bot's rows above play half
the map with fourteen constructor queues. With the whole map and forty queues (terr5-all-nullai-comet-1):

| game (terr5-all) | 10:00 mex / cons / income | 15:00 | 20:00 | oracle income 10 / 15 / 20 | tier-2 extractors begun |
|---|---|---|---|---|---|
| seed 1 Armada | 45 / 15 / 105 | 72 / 20 / 202 | 77 / 20 / 317 | 147 / 461 / 737 | 23 |
| seed 2 Armada | 71 / 16 / 165 | 78 / 14 / 194 | 78 / 13 / 209 | 132 / 411 / 679 | 0 |
| seed 3 Cortex | 72 / 14 / 165 | 77 / 13 / 188 | 78 / 18 / 199 | 170 / 428 / 738 | 0 |
| seed 4 Cortex | 49 / 13 / 115 | 76 / 18 / 225 | 76 / 16 / 321 | 157 / 414 / 661 | 24 |
| the flagship, seed 1 | 68 / 28 / 177 | 78 / 87 / 463 | 78 / 97 / 802 | | 77 |

**With the linear program as the planner** (T-LP, terr13-lp-nullai-comet-1, the same seeds and room): income 27-34 /
98-110 / 135-142 / 230-237 at 5 / 10 / 15 / 20, one lab until 10:00, 10-11 constructors at 5:00, 36-42 frames begun
by 5:00; the program's own ceiling from the same starts 49-52 / 262-268 / 885-902 / 1,362-1,410. Level with the
anneal bot at 20:00 and behind it at 10 and 15; the model error at +60 s is +8 to +13 metal a second and compounds.
Those games ran terr8's count-shortfall dispatch, not the credit dispatch fa9723d's message described (its edit scripts
asserted before writing hands.rs): read again from the jobs and curve lines, both stocks sat at their storage caps
from 6:00, the lab stood idle from 7:00 to 14:00, and the constructors stayed at 17 while the program asked for three
more each minute of a tier-2 kind through a fractional advanced plant nobody started. With T-DISPATCH as it stands
(terr14-lp-nullai-comet-1): 36-40 constructors at 10:00, four factories by 15:00, build power 7,000-8,000 at 15:00;
income 20-30 / 102-121 / 146-216 / 222-332, the bank 29,000-45,000 at 20:00, because 24-28 of the forty stand walking
at any minute and 130 extractor orders in four games were dropped as held on arrival at far spots (T-ORDERS, fixed
for terr15). With the hold from arrival (terr15): 70-72 extractors at 15:00, income 103-121 / 184-225 at 10 / 15, the
spent share 60-76 %, and two games lost to our commander walking its extractor flows across the map; the leash, the
yard keep-out and the class refinement followed (terr16: no game lost, income 112-116 / 194-209 / 297-338 at 10 / 15
/ 20, and no moho in any game: four labs by 9:48 filled the factory cap and their kind the constructor cap, so no
later plan could hold the tier-2 line; T-COMMIT by capability and the caps per tier followed, terr17).
The planner is global now; the hands are the gap, from the first minute.

To 10:00 the anneal bot is the flagship's equal in two games; after it the rolling search finds no tier 2 in two games
(every plan from 9:19 ends with no advanced plant and income flat at 200) and finds it late in the other two. The
long search from the start says tier 2 pays (the oracle's 660-738, within a tenth of the flagship's 802): this is
search regret, not model error. An advanced plant, its constructor and the mohos are a valley the 400 ms
warm-started anneal does not cross with forty queues. The next step is the search, not the model: a chain-seeded
restart beside the warm start (`Palette::chain_seed` exists), or a longer search at a lower cadence.
