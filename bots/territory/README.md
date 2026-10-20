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
| T-PLAN | A snapshot of the game as the simulator's state (stocks, standing economy, every builder with its place and job) is annealed into the plan with the most metal made to the game's end, on the bot's own threads, every 30 s and when a factory or constructor comes (10 s apart at least); the first plan on the engine thread at the first orders. The plan in force is the warm start with its taken steps removed; a builder's leading step is skipped when its current order is that step (the re-basing) | `replan` 30 s, `budget` 400 ms, `first` 1,000 ms, `threads` 4, `factories` 3 and `constructors` 14 queues for builders yet to come | The flagship's rolling planner of 2026-09-21 won the economy against the ladder; the search is the model's own answer, with nothing hand-written about tempo | `lib.rs` `request`, `adopt_planned`, `warm_start`; `planner.rs` |
| T-OBJECTIVE | Metal made over the horizon (every simulated second's income summed) plus `terminal` seconds of the income at the horizon, nothing else; the horizon is the game's end less the clock, 120 s at least | `end` 1,200 s, `terminal` 0 | The user's score for stage 1 is metal income over the course of the game; with the horizon the whole remaining game no window-end patch is needed, and converters count because their metal is metal | `buildorder::anneal::Objective::Made` |
| T-PALETTE | The roster: every economy building, factory and construction turret the commander or any reachable constructor builds, assist and help; the factories' list cut to builders unless `soldiers=on`; no turret on offer | `soldiers=off` | Stage 1 has no enemy; soldiers on the list only cost the search moves | `lib.rs` `start` |
| T-SCENARIO | The spots nearer home than its mirror through the map's centre (`spots=all` for every spot), nearest first; a constructor past its queue takes the nearest free spot; construction turrets feed the first factory; no leash, no tax | `spots=half` | Our half is what a 1v1 can count on and what the flagship's medians are measured on; the ceiling is `spots=all` | `snapshot.rs` `scenario` |
| T-ORDERS | One order per builder from its queue. It stands until the engine names its frame and the frame stands, or it comes to nothing: refused by the host, rejected by the engine, dropped (the builder idle again after the grace), or held two minutes with no frame. A step that came to nothing is given once more, an extractor's spot refused for a minute meanwhile, then skipped; nothing counts before 3 s, when the engine drops everything. A frame let go of is given back by a repair. Every frame begun is logged with its order's age and the builder's distance (`begun:`); every order that came to nothing (`dropped:`) | `GRACE` 45 frames, `HELD` 2 min, `ATTEMPTS = 2`, `FIRST_ORDERS` 3 s, `SPOT_REFUSED` 1 min | The distilled bot's D-ORDERS: a shorter watchdog took orders from builders in range of their spots; the engine starts no frame while it reclaims a wreck on the site | `hands.rs` `run_hands`, `review`, `failed`, `frame_begun` |
| T-SITES | Placed as the simulator places them: an extractor on its spot (the step's, else the nearest free one); an upgrade over the standing basic extractor nearest the builder; an economy building beside the builder when it stands within 600 of home (the simulator charges no walk), else on the simulator's golden-angle spiral round home; a factory 350 toward his start from home with its facing and exit lanes kept; a construction turret 150 behind the nearest factory; `assist` a guard on the first factory for 20 s; `help` a repair on the frame with the most metal left; a factory one unit at a time; a factory beyond the plan's queues is passed over | `BESIDE = 600`, `ASSIST_CHUNK` 20 s | What the hands do must be what the search priced, or the model error is the hands' | `hands.rs` `give`, `free_spot`, `base_site` |
| T-NANOS | A construction turret guards the nearest factory within 330 | `NANO_REACH = 330` | The calibrated replays' turrets fed the first factory; `nanos_help_frames` is off in the scenario to match | `hands.rs` `run_nanos` |
| T-PRIORITY | Nobody is set to low priority | | The simulator shares a stall over every build, so the hands do too; a calibration item, not a rule | |

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
