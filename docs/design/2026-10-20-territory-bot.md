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
