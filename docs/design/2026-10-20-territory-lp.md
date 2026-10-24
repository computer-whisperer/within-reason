# Target design: the territory bot's planner as a linear program

Written 2026-10-20 night at the user's direction, before any code: "the local minima in this game are quite harsh ...
my initial thoughts went in a different direction from the annealing approach (more towards linear programming or
gradient descent on a policy)"; agreed shape: the LP as the planner, the hands dispatching by its plan, the
simulator as the verifier; "write the LP formulation note first". Replaces decision 1 of
`docs/design/2026-10-20-territory-bot.md` (the rolling anneal) once built; everything else of that note stands.

## Why

terr5-all-nullai-comet-1: with the whole map and forty constructor queues the rolling anneal was the flagship's
equal to 10:00 and then found no tier 2 in two games of four (every plan from 9:19 on ended with no advanced plant,
income flat at 200 to 20:00), while the long search from the same start says tier 2 pays (661-738 at 20:00) and
the flagship made 802. The tier-2 branch is a chain of three investments worthless alone (the advanced plant, its
constructor, the upgrades): no single mutation of a sequence improves the score, so a local search over sequences
does not walk into it, and a warm start keeps it out. Every prerequisite in the game has the shape. The economy's
accounting, on the other hand, is linear in build rates, so the plan can be found globally.

## What the LP decides, and what it leaves to the hands

The LP decides **how much build power of each builder type goes to each unit type in each time bucket** over the
remaining game, from the live snapshot: the macro flow. It does not decide which spot, which site, which of two
constructors, or when one builder walks. Those are the hands' (`bots/territory/src/hands.rs`, placed as today) by a
dispatch rule over the LP's cumulative targets (T-DISPATCH below). Geometry enters the LP only as calibrated
overheads on build time; integrality only through the dispatch, with integer first units as a fallback.

## The model

### Time

Buckets of `Δ` = 10 s, `t = 0 .. T-1`, `T = ceil((end - t0) / Δ)` with `end` the game's end (1,200 s) and `t0` the
snapshot's clock; 120 buckets at the start. A count at `t` means at the bucket's start; what finishes inside a bucket
works from the next one, which is conservative by half a bucket on average.

### Sets and data (all from the `Hello`, nothing compiled in)

- `U`: the unit types on offer: the roster's economy buildings, factories, construction turrets and mobile builders
  (`Palette::roster` with soldiers off); later the soldiers and turrets of the territory stage. The commander is in
  `U` as a builder type with a fixed count of one.
- `B ⊆ U`: the builder types: the commander, each mobile builder, each factory, the construction turret.
- `can(b, u)`: `b` builds `u` (the build options). Two additions to the raw options, both of which the engine allows:
  a mobile builder assists any product of a standing factory (a guard order), and a construction turret helps any
  frame in its reach, which the model takes as everything but extractors and their upgrades (those stand afield).
- Per unit type `u`: metal cost `cm[u]`, energy cost `ce[u]`, build time `bt[u]` (build-power-seconds), build power
  `bp[u]` (0 for non-builders), net constant energy `em[u]` (production less upkeep: a solar +20, an extractor -3,
  a tier-2 extractor -20, a fusion +750), wind cap `wc[u]`, metal and energy storage `sm[u]`, `se[u]`, converter
  capacity `cc[u]` (energy a second) and efficiency `cf[u]` (metal per energy: the basic converter 70 and 0.0143,
  the advanced 600 and 0.0172), extraction rate `ex[u]` (the basic extractor 0.001, the tier-2 0.004), and the
  calibrated walk overhead `wk[b, u]` in seconds a mobile builder of type `b` loses per unit of `u` (below).
- The map: the spots with their amounts `a[k]` (Comet: 80 spots, all 2,295, so a basic extractor pays 2.3 a
  second and a tier-2 one 9.2), sorted by distance from home; `S` the spots in play (our half, or all); the mean
  wind `W` by the engine's law (Comet 3.0).
- The snapshot: standing counts `n0[u]`, frames in progress as fractions (a frame at progress `ρ` of type `u` is
  `ρ` of a unit: the engine draws cost evenly with progress, so the remaining `1-ρ` costs and takes exactly its
  share), the stocks `m0`, `e0`, and the base storages.

### Variables, all non-negative

- `x[b, u, t]`: build-power-seconds of builder type `b` spent on unit type `u` in bucket `t`; defined only where
  `can(b, u)`.
- `n[u, t]`: units of type `u` complete at the start of bucket `t` (fractional).
- `m[t]`, `e[t]`: metal and energy in stock at the start of `t`.
- `c[t]`: energy converted to metal during `t`.

### Constraints

1. **Completion.** `n[u, t+1] = n[u, t] + Σ_b x[b, u, t] / bt'[b, u]` with `bt'[b, u] = bt[u] + bp[b] · wk[b, u]`:
   a mobile builder's walk is build power spent on nothing. `n[u, 0] = n0[u]`.
2. **Build power.** For each builder type `b` and bucket `t`: `Σ_u x[b, u, t] ≤ bp[b] · n[b, t] · Δ`. The commander's
   `n` is 1 throughout.
3. **Assist and help, bounded.** Build power that mobile builders put on factory products is at most `A` times the
   factories' own: `Σ_{b mobile} Σ_{u a product} x[b, u, t] ≤ A · Σ_f bp[f] · n[f, t] · Δ`, `A = 2` to start (the
   yard clears between units; the distilled bot's games showed three turrets on one plant did not spend). The
   construction turret's power is bounded by what stands in its reach only through the same factor on its own
   capacity; geometry is the hands'.
4. **Metal.** `m[t+1] ≤ m[t] + Δ · (im[t]) + cf · c[t] - Σ_u cm[u] · Σ_b x[b, u, t] / bt'[b, u]` with the income
   `im[t] = Σ_u mm[u] · n[u, t] + Σ_u ex[u] · ā · n[u, t]` over the extractor types, `ā` the mean amount of the
   spots in play (the refinement for maps with unequal spots: a variable `z[k, t]` per spot, ordered nearest first,
   with its own amount; Comet needs none). The inequality discards overflow. `m[t] ≤ sm0 + Σ_u sm[u] · n[u, t]`.
5. **Energy.** `e[t+1] ≤ e[t] + Δ · (Σ_u em[u] · n[u, t] + Σ_u min(W, wc[u]) · n[u, t]) - c[t] - Σ_u ce[u] ·
   Σ_b x[b, u, t] / bt'[b, u]`; `e[t] ≤ se0 + Σ_u se[u] · n[u, t]`. Upkeep is in `em`.
6. **Conversion.** `c[t] ≤ Δ · Σ_u cc[u] · n[u, t]`, and, because the game's converters burn only energy above three
   quarters of storage, `e[t] ≥ 0.75 · (se0 + Σ_u se[u] · n[u, t])` for every bucket from the first one in which a
   converter stands (a restriction, not a relaxation: a plan that converts keeps its energy high; the engine would
   convert a little less in the buckets the plan dips, which the verifier measures).
7. **Spots and upgrades.** `Σ_{u basic extractor} n[u, t] ≤ |S|`; `Σ_{u upgrade} n[u, t] ≤ Σ_{u basic} n[u, t]`. An
   upgrade's income counts its increment over the basic rate, as the simulator counts it; a standing tier-2
   extractor in the snapshot counts as one basic plus one upgrade.
8. **Stocks never negative.** `m[t], e[t] ≥ 0`: the LP does not stall; a stall is an inefficiency it has no reason
   to choose, and the engine's shared stall is what the hands get when they run ahead of the plan.
9. **Factories.** A factory type's products are bounded by its own build power through 2; one unit at a time is
   exact for fractional flow.

### Objective

Maximize `Σ_t Δ · im[t] + terminal · im[T]`: metal made over the horizon plus `terminal` seconds of the final
income (0 for the stage-1 score, as today). The duals of 4 and 5 are the marginal worth of a metal and an energy at
each minute, and the duals of 2 the worth of a build-power-second of each builder type: the algebraic model's
prices, derived.

### Size and solver

With 30 unit types, 5 builder types, `can` pruning `x` to about 60 pairs, and 120 buckets: about 7,200 `x`, 3,600 `n`,
360 stocks and conversions; about 8,000 constraints, sparse. An interior-point solve is tens of milliseconds, a
simplex similar. The solver: `clarabel` (pure Rust, Apache-2, no C++ build in a library the engine loads) for the
LP; if it proves slow or imprecise at this size, HiGHS through `highs` (a cmake build of the C++ library, a
heavier dependency). Decided at step 2 by measuring both on the same problem.

### What the relaxation gets wrong, and the answer to each

- **Fractional chain buildings.** The LP can build 0.3 of an advanced plant to get 0.3 of its build power early. The
  dispatch starts a building only when its cumulative target passes one half, so the hands never place a fraction;
  the next solve sees the frame. If the plans show the artefact mattering (an advanced plant drawn out over minutes
  at a trickle), the first unit of each factory type becomes an integer variable gating its build power (a MILP of a
  dozen binaries, which HiGHS takes in the same time; `clarabel` has no integers, and this is the one reason to
  prefer HiGHS from the start). It mattered (terr16: 0.3 of an advanced plant for minutes, never started, no moho
  all game); the answer taken is the hands' (T-COMMIT by capability: the first unit of a factory kind whose
  products nothing standing makes is started whole as soon as the program wants any of it), not the MILP.
- **Walks and geometry.** `wk[b, u]` is one number per builder type and unit type, calibrated from the `begun:` lines
  of the bot's own games (the age of each order at its frame, by type: extractors 5-20 s with the far spots, base
  buildings 2-5 s, the tier-2 projects 50-120 s today). The far spots cost more walk than the near ones; the mean
  overhead is right for the plan's shape and wrong at the margin. The verifier (below) measures what that costs.
- **Builder-second granularity.** A unit built by several builders in parallel is exact; a single builder on a
  5-minute project is exact; the model cannot say which builder stands where, and does not need to.
- **Stalls and priority.** The LP never stalls; the engine shares a stall over every build. The hands run ahead
  of the plan only when the dispatch over-orders, which the cumulative targets prevent.
- **Converters' threshold** is a restriction (6), not a relaxation, and small.
- **Wind** at the engine's process mean, as the simulator prices it; on Comet it is struck anyway.

## The planner in the bot

### Snapshot to the LP

The snapshot is what `snapshot.rs` builds today, read into `n0`, the fractions, and the stocks. Struck types
(T-STRUCK) leave `U`. Spots within his commander's reach leave `S` (the NullAI dead spot, the user 2026-10-20:
"the null ai still has a commander that automatically attacks anything near it ... a dead spot of 2-3 mexes": the
first instance of the territory term).

### Solve cadence

A solve from the snapshot every 30 s and when a factory or constructor comes, on the planner thread as today, with
the first solve blocking at the first orders; a solve costs well under the 400 ms budget, so the plan is at most a
few game-seconds stale at speed 50 and the re-basing becomes trivial (the targets are cumulative counts, not
sequences: nothing to re-base).

### Dispatch (T-DISPATCH)

As first written: the plan is `n[u, t]` for every type; at each half second an idle builder takes the type with the
largest shortfall `n[u, now + lead] - (built[u] + ordered[u])` in build-power-seconds that it can serve, a building
only when the shortfall passes one half; with nothing short a mobile builder assists, else helps the dearest frame; a
factory stands idle. That rule ran in terr6 to terr13 and failed on the relaxation's own artefact (above, "fractional
chain buildings"): the program is indifferent between factory types and spreads a plant over all of them, and grows
its constructors through a fractional advanced plant, so by type no count ever ran half a unit ahead and the lab
stood idle for seven minutes with both stocks at their caps (terr13).

As built for terr14 (`hands.rs` `dispatch`): the lumpy things go by class. The factories as a whole, and a factory's
products by role, start a unit when the program's count of the class runs half a unit ahead of what stands, is
framed or is ordered, and the type started is the one of the class the program wants most; a builder that cannot
build it leaves the class to one that can, and a factory that does not make it substitutes its own best of the
class only while no standing factory makes it (else the lab fills the constructor cap with its own kind and the
advanced plant has no room for the tier-2 constructors the mohos need: terr15). Everything else goes by the
program's flows `x[b, u, t]` for the builder's type: its credit on `u` is the program's power on `(b, u)` to
`now + lead`, less what type `b` has committed to `u` since the plan was adopted (an order's cost to its type, a
guard's chunk of power; the standing orders' unbuilt cost seeds it at each adopt); the largest credit as a share of
`cost[b, u]` is taken, however small, since standing idle is never what the program planned; a product's flow is a
guard on the factory that makes it. With no credit, the largest count shortfall of a type it builds; with none, the
dearest frame. The order review, the retry at another site, the struck rule and the instruments stay as they are.

## Columns: the frames under way and the spots by ring

Written 2026-10-20 before the build (terr28), from the one-minute error of terr25 and the lock the user saw at
12:14 (three fusions and an advanced one framed together while the energy stood at zero).

The program's unknowns are per *column*, not per unit type. A column is a unit type and a class:

- **Whole**: units of the type started from nothing. The pairs' cost is the build time plus the builder's walk.
- **Frames**: the frames of the type under way at the start, framed or still walking, which the snapshot reported
  as jobs. One column per type with jobs: its count runs from zero to the number of frames, a frame costing the
  mean work left on them and no walk, its makers the type's makers (the factory for a product, with assist and
  turret help as for the whole). The frame's share of the type's production, storage and caps follows its count,
  so a frame produces as the program pays what is left on it and nothing at the start. Before this a frame entered
  as a fraction of a standing unit (`n0 += progress`): a fusion a third built was a third of a fusion's energy
  from the first bucket, and with four energy frames the program read a thousand energy a second that did not
  exist, planned upgrades on it, and the hands followed it into the lock. Extractor frames stay fractions in their
  ring (a spot, not work, is what they hold; the hands' commitment keeps them from being ordered twice).
- **Ring k** (extractors only, basic and upgrade): the spots in distance ring `k` of `RINGS` from home, the rings
  by distance quantiles so each holds a like number of spots. A ring's column has the ring's mean spot value, the
  ring's spot count as its cap (loosened to what stands in it), its upgrades bounded by its basics, and a walk of
  the base overhead plus the ring's mean distance over the builder's speed (armck 36 elmo/s: 1,500 elmos is 42 s,
  6,000 is 167 s, against one constant of 65 for every spot, with the hands measured at 22-28 s within 1,500 and
  134-191 beyond 6,000). The spot choice then belongs to the program: a near upgrade against a far basic is
  priced by both its spot value and its walk.

Measured (terr31-33): the frames column holds; the rings' walk from home is wrong in kind. The hands' realized
extractor walk is a median 56-66 s in every batch whatever the plan prices, because a constructor takes its next
spot from where it stands, not from home; priced at 59 / 118 / 177 s by ring the plan deferred the far spots and
the hands stood 35-46 extractors at 10:00 against 50-61. The rings stay as an arm (`rings=N`, one by default with
the constant walk); a ring model that pays would price the step between neighbouring spots, not the trip from home.
terr34 then found that every columns batch had dropped the commander's extractor pairs (the reach test against the
ring's mean distance, 1,900 for ring 0 and 4,000 for one ring, against 1,500), so terr31-33's openings are not the
rings' verdict; the test is the ring's inner bound now. terr35 (the frames alone): 718 / 640 / 755 / 587, the openings back,
games 00-02 ahead of terr30 at 15:00 and game 02 the best game yet; game 03's plant frame unhelped for five minutes
(the Frames flows outrank the dearest-frame fallback, and a factory's frame is no Frames column). terr36: a
factory's frame helped first, six on it at most: 691 / 695 / 729 / 693, every game above 690, the steadiest batch
of the arc; committed as the standing state. Open: the late orders (the plan's hundred converters a minute against
the hands' 25-50; the far upgrades walking), his roaming commander, the factory mix, then step 3.

The hands read the columns through `LpPlan`: `count_at(unit, t)` sums the unit's columns; a `Flow` carries its
column's class, so a flow on a Frames column is power to put on the type's frames (the help the plan wants, not the
dearest frame), and a flow on a ring is an extractor on a spot of that ring (T-SITES takes the spot from the
ring). The credits are by (unit, class); the standing orders' seed counts extractor orders only, since the Frames
columns carry the rest.

## Reclaim and the farm (2026-10-24, from thebluegecko's game against InactiveAI)

The user's benchmark example (`docs/experiments.md` replay-gecko-inactiveai-comet-2026-10-21; the bot's README) does
two things the program had no column for, and one thing its arms forbade.

- **Reclaim** (`reclaim`, off by default in the bot, `--reclaim` on the CLI): a pair of kind `Reclaim` for every
  builder but a factory and every Whole column of a standing building (not an extractor, whose upgrade replaces it;
  not a frame; nothing mobile). Its power counts the other way in the completion row, its metal comes back whole
  (the engine returns a reclaimed unit's metal in the second the reclaim ends and none of its energy, read from his
  record at 4:41 and 10:22), its work is the unit's build time at the reclaimer's power (his advanced plant, 28,000,
  in about seven seconds under the turrets round it), a turret without a walk and a mobile builder with its base
  walk. One metal a unit in the objective as a tie-breaker, and none in the last bucket: without them the interior
  point took down six fusions and the storages in minute 19 to loosen the energy reserve at the horizon, where
  nothing counts. The plan then does what he did: from the bot's seed-1 start it takes the lab down at minute 9
  once its constructors are out (he took his at 4:41: his lab made nine and stopped, the program's makes fifty)
  and the advanced plant at 10-11 once its constructors are out (his at 10:22), and a few solars at 8-9. In the
  program's own accounting reclaim is worth about two percent at 20:00 (2,049 to 2,087 with the farm; 1,656 to
  1,652 without it, inside the solver's noise), as in his game (7,200 of 499,000 made): the action's worth is in
  the moment and the ground, which the plan does not price, and the hands will.
- **The farm** (`max_nanos`, `maxnanos=N`, `--max-nanos N`; `help_factor` as a physical bound): his 311 turrets
  stood in a block south of his base and his advanced fusions and converters were placed inside their reach (39
  and 28 turrets within 300 of a frame at its placing, a median), the aircraft placing and the turrets building.
  The program's bound of six turrets a factory was the hands' rule (turrets behind the plant), not the game's;
  `max_nanos` replaces it with a number when above zero (the farm's size is the base's ground, which the program
  cannot see). The help factor is the placer's share: at 40 a 310-power constructor puts 26 s on an advanced
  fusion the turrets finish, and his advanced aircraft put less. With both (`--help-factor 40 --max-nanos 300`)
  the program answers 2,049 at 20:00 from the bot's start (2,087 with reclaim), against 1,656 under the bot's
  arms, and builds his economy: 135 turrets, 85 advanced converters, 16 advanced fusions, 32 fusions.
- **The walk**: the early marks are the model's walk constant, not the room. With `--walk-extractor 36` the
  program's 5:00 is 62 with 26 extractors, his figure exactly (his nine constructors took 55 s an extractor
  including the build; the constant 65 is the hands' own realized median, 56-66 s over every batch since terr25,
  since they take spots as the flows credit them and not as a chain from where they stand). The program cannot
  fix the hands; the number says what the hands' chaining is worth: 62 / 316 / 1,058 / 2,357 at the marks with
  all three arms against his 62 / 213 / 651 / 1,706, and 50 / 258 / 937 / 2,087 with the hands' walk.

What the program still has not got of his: the air line (his farm's turrets were built by 46 constructor
aircraft; the land-lines rule stands, since the hands fly nothing), an energy storage before 5:00 (a plan that
never stalls has no use for one; the hands have), and the end of the horizon, where under `terminal=0` the last
two minutes buy solars and basic converters by the hundred because the bank is worth nothing at 20:00 (the
composition at 16:00 is the shape to read: 169 turrets, 80 converters, 17 energy buildings).

## Verification, in order

1. **The LP against the simulator.** The LP's plan from the empty start, dispatched inside the simulator by the same
   rule (a `Policy` mode for `simulate`: queues filled from cumulative targets instead of a fixed plan), gives a
   simulated curve; the LP's own `im[t]` is another. Their gap is the discretization, the walk overheads and the
   fractions. Target: within ten percent of each other at every mark to 20:00.
2. **The LP against the oracle and the flagship.** From terr5-all's dumps: the LP's ceiling beside the anneal's
   (661-738 at 20:00 with forty queues) and the flagship's 802. The LP should not sit below the anneal; it has
   nothing to get stuck in.
3. **In the engine.** The rolling LP planner on the terr5-all seeds (every spot, `--max-minutes 20`), the realized
   curve against the LP's own plan (the model error, per look-ahead as today) and against the flagship. The gap of
   interest is now the hands': the tier-2 frames' 50-120 s delays, the walks.

## The territory stage, as this model takes it

A spot's income becomes its income less its expected loss, the loss a slack variable bounded below by the income
less the cover: `loss[k, t] ≥ im_k · z[k, t] - cover[k, t]`, with `cover[k, t]` a concave piecewise-linear function
of the defense standing near `k` (turrets within reach, soldiers' metal at the station) and his reach at `k` read
from the field the hands keep; soldiers and turrets are unit types whose only worth is the cover they add. All
linear, in the same program, with the prices of cover falling out as duals. Written when stage 1 is done.

## Steps

1. `crates/buildorder/src/lp.rs`: the model above from a `Scenario`, `State` and palette; `buildorder lp --hello
   DUMP` prints the plan by minute and its duals; the test in `tests/arithmetic.rs` checks the accounting on a
   two-type problem by hand (an extractor pays its cost back in its cost over its yield seconds).
2. The solver decided by measurement (clarabel against HiGHS on the same problem).
3. The `Policy` dispatch inside `simulate`, and verification 1 and 2 in the study document.
4. `bots/territory`: the planner thread solves the LP; T-DISPATCH replaces the queue-walking in `hands.rs`; the
   anneal stays reachable as an arm (`planner=anneal|lp`) until the LP's rows beat its own.
5. Verification 3; the ledger rows; the README's T- rows rewritten (T-PLAN, T-DISPATCH).

## Open questions for the user

- The dead spots by his commander: leave them out by a radius round his start (400), or by his seen units'
  reach (the stage-2 field, early)? The radius is a one-liner; the field is the right thing and costs a day.
- The assist factor `A`: 2 to start, calibrated from the flagship's game (its commander assisting the plant).
- Integer first units from the start (HiGHS), or only if the fractional artefact shows (clarabel first)?

## Status

- 2026-10-20 night: written; nothing built.
- 2026-10-20 night, the user's rulings on the three questions: the dead spots by a radius round his start; the
  assist factor 2 to start; clarabel first, integers only if the fractional artefact shows. Building step 1.
- Step 1 built (`crates/buildorder/src/lp.rs`, `buildorder lp`, the test): the first solves on terr5-all's seed 1
  with every spot showed three things the note had not: construction turrets are a free lunch to the program (441
  of them, a converter spiral to 4,000 metal a second), the roster's air, hover and gantry lines give it fractional
  factories of every kind, and the economy is exponential in the model with a few minutes' doubling once fusions
  and converters pay (which the flagship's 177 / 463 / 802 at 10 / 15 / 20 shows the game is too). Added to the
  program: turret help at most three times the mobile builders' direct power on a type (3b), turrets at most six a
  factory (3c), the land lines only (a factory that makes no ground builder, and a builder that flies or hovers or
  builds no extractor and no factory, are out), caps as the hands' capacity (forty constructors, four factories),
  and buckets that grow from 10 s to 30 s after the first minute and 60 s after the fifth, since a solve's cost
  grows with the square of the bucket count (120 ten-second buckets 30 s, 29 buckets 1.6 s). The ceiling it gives
  from that start with the caps: income 89 / 423 / 1,138 / 1,643 at 5 / 10 / 15 / 20, 38 extractors at 5:00 on 11
  constructors, every spot by 8:00, every upgrade by 12:00, then converters; twice the flagship. Whether the hands can
  realize it is step 3's question.
- Step 4 built before step 3 (the engine run is the test, and the plan lines already give the model error):
  `planner=lp` the default, T-DISPATCH over the program's targets, T-COMMIT for the first factory. Eight batches
  (terr6 to terr13) on the hands: the anneal plan's zero factory queues refused every lab, factories were passed
  over for want of a queue, a finished frame counted as a job beside its standing unit (two labs where one stood),
  the walks were three to ten times under (calibrated from the begun lines: a constructor's extractor 50 s, its
  base building 12, the commander's 18 and 45), and his roaming commander ate the far spots (the dead spots follow
  his seen armed units). **Correction, 2026-10-20 evening:** the dispatch those batches ran was the count-shortfall
  rule as first written; the flow-following credit dispatch that fa9723d's message, the ledger rows terr9 to terr12
  and the README's row of the time describe was never in the tree (each edit script asserted before writing
  hands.rs, found from the file times and the task outputs), and the diagnoses of terr10 and terr11 were of code
  that never ran. terr13: income 27-34 / 98-110 / 135-142 / 230-237 at 5 / 10 / 15 / 20 against the program's own
  ceiling of 49-52 / 262-268 / 885-902 / 1,362-1,410, and against the anneal bot's 105-165 / 188-225 / 199-321 at
  10 / 15 / 20 on the same seeds: level at 20:00, behind at 10 and 15. Read again from its jobs and curve lines:
  both stocks at their storage caps from 6:00 to 20:00, the lab idle from 7:00 to 14:00, constructors flat at 17
  while the program asked for three more each minute of the tier-2 kind through a fractional advanced plant nobody
  started (by type no count ever ran half a unit ahead), and 21 extractor orders held two minutes each.
- T-DISPATCH rebuilt as the section above says (terr14): the factory side is fixed (36-40 constructors at 10:00,
  four factories by 15:00, build power 7,000-8,000), the income barely moved (102-121 / 146-216 / 222-332 at 10 / 15
  / 20) and the bank reached 29,000-45,000: 24-28 of forty constructors walking at any minute, 130 extractor orders
  in four games dropped as held on arrival at far spots (the held clock ran from the order, every walk past two
  minutes cut at the 300 ring), and a stale attempt count sending economy buildings to the base spiral from afar.
  Both fixed for terr15: no order held any more, the far spots reached (70-72 extractors at 15:00), income 103-121 /
  184-225 at 10 / 15, the spent share 60-76 %; and three more causes read from its lines: the commander walking its
  extractor flows across the map to its death (two games lost), second labs and turrets asked for against the first
  lab's open yard and dropped, and the lab filling the constructor cap with its own kind so the advanced plant made
  seven tier-2 constructors and stopped (seven mohos at 15:00 against eighty). The leash, the yard keep-out and the
  class refinement above went in for terr16: no game lost, 72-73 extractors at 15:00, income 112-116 / 194-209 /
  297-338 at 10 / 15 / 20, and no moho in any game: the program spreads a factory over kinds and wants labs for
  their assist capacity, the class rule took the larger fraction each time, four labs by 9:48 filled the factory
  cap and their kind the constructor cap, and no plan from a later snapshot could hold the tier-2 line (the
  ceiling's own runs on 0.8 of an advanced vehicle plant). This is the note's "fractional chain buildings" artefact
  as the hands meet it. For terr17: T-COMMIT by capability (a factory type the program wants at all, with a product
  it wants that nothing standing makes, started whole), the constructor cap per tier, newborn builders on count
  shortfalls until the re-plan, the walks at the new medians (a constructor's extractor 65 s, its other building 3,
  the commander's 16 and 5). terr17: the tier-2 line opens (the plant in every game, 77-80 constructors at 15:00,
  income 485-609 at 20:00 against 297-338) and the early game falls (7-23 at 5:00 against the ceiling's 51): the
  plant committed whole at 3:55 on an income of ten, and energy at zero from 1:00 to 5:00 because the program's
  cheapest energy is a fusion it builds as a fraction from its second minute, drawing energy from it at once, while
  the engine's first energy is solars whose flows are small beside it. The fractional artefact again, in energy. For
  terr18: nothing starts that the stock and 45 s of income do not cover, and energy goes by class (the program's
  planned energy income realized by the best affordable energy building), with the host's placed site taken as the
  order's position and every pending site a keep-out disc. terr18 collapsed on the energy class as first written
  (candidates from the builder's whole build list: the geothermal plant with no vent on the map, 5,500 no-site drops
  a game; and solars ahead of the flows on a stock of zero: metal at zero from 2:00 to 8:00 with energy at its cap):
  income 6.6-8.9 at 5:00. For terr19 the class takes the program's own types and fires only while the energy stock
  is under half its storage, and a type the host finds no site for is left alone a minute. terr19: the early game
  back (20-27 at 5:00) and the late game kept (570-610 at 20:00 in three games, the fourth 308); the model error at
  +60 s a median of about zero now. Found in it: the program had no feasible point from 9:05 on in every game (27-34
  solves `PrimalInfeasible`), since the hands stood 51 basic constructors against the tier's cap of forty and a cap
  on the first bucket cannot hold them: every cap now bounds what the plan adds (loosened to what stands). And the
  energy class read by every idle builder put the commander on solars instead of extractors (constructors carry it
  now, two orders at a time). terr20: no solve failed, 30-53 mohos a game; and two regressions: the lab
  substituting its kind for the program's fractional advanced vehicle constructors ran to 136 constructors once the
  caps no longer stopped it (a product's class is its role and tier now), and two pending solars by constructors
  alone left the energy at zero to 8:00 (any builder, three pending, and the affordability window 90 s so the plant
  comes nearer 7:00). terr21: the best and steadiest 20:00 yet, 636-667 in every seed (the flagship's 802), the
  plant at 5:49-6:38, 44-59 mohos; late the hands spend a third of the income (eight fusions all game, since energy
  was class-only and the class fires on a low stock) and early 11 extractors at 5:00 (the fusion's fraction still
  the program's energy). For terr22 the program's palette is what the hands could start now (metal cost within the
  stock and 90 s of income): its early energy is solars and advanced solars honestly, and the plant and fusions
  enter the program when a whole one can be started; energy back in the flows, the class a backstop. The relaxation
  answered by the offer rather than by integers. terr22: mixed (463-658 at 20:00; the Cortex games' openings worse),
  and the opening's lump read from the lines: the program's honest first minute is three solars, two constructors
  and 0.4 of a lab; the hands commit the lab whole and the commander builds the solars beside it, so the starting
  metal is gone by 1:09 and every frame crawls for three minutes. For terr23 a factory ordered or framed is a
  commitment the program takes as standing with the rest of its cost drawn from the stocks (`State::committed`), and
  the help spreads over frames by metal left per builder (48 stood on one frame while five fusions had one each).
  terr23: worse than terr21 (481-567 at 20:00; the plant at 7:13-8:46 since a factory not yet affordable was not
  offered), and the Cortex openings unchanged: the program's second plan wanted two extractors and two solars by
  one minute, with the extractors on the fractional constructors it imagines and the commander's power on solars,
  and the commander followed its own flows. For terr24 the credits are by unit type over every builder type's
  flows (the orphan power of imagined builders falls to those that stand), and factories are offered whatever they
  cost. terr24: steady at 598-626 (terr21 636-667), and the Cortex openings unchanged: the energy backstop ran ahead
  of the flows on a stock the lab had drained, and the flows tied two solars and two extractors wanted by one
  minute. The tool's `--flows` shows the program's own first minute (0.37 of a lab, two solars, the commander
  assisting the lab, extractors from 0:40). For terr25 the backstop runs ahead of the flows only in a stall (under a
  fifth of the storage), else after them, starting a whole unit's shortfall, and a tied share goes to what the plan
  wants sooner. terr25: the best batch on every mark and steady, 25-34 / 117-124 / 241-278 / 552-678 at 5 / 10 /
  15 / 20 (the flagship 36 / 177 / 463 / 802, the ceiling 51 / 266 / 961 / 1,665), the openings fixed, the plant at
  4:39-7:08, 51-58 mohos. The one-minute model error by phase: +4..+6 to 5:00, +17..+22 to 10:00, +42..+141 after.
  Next, in the order the error says: the far spots' walks in the program (two extractor classes by distance, the
  mid-game's 12-21 constructors walking 70 s a spot), the late plan's fractional converter spiral (fusions and
  converters by the hundred that the hands start one at a time: the offer by affordability covers the start of
  each, not the rate), his roaming commander's kills (4-5 constructors a game), the late build power (the ceiling's
  24 turrets and its assist factor against 15-25), the objective's end (under `terminal=0` nothing from 18:00 that
  pays past 20:00; the bank 42,000-51,000 at 20:00), then step 3.
- 2026-10-20, later: the user named the lock (12:14, three fusions and an advanced one framed together while
  energy-locked). Read from the lines: the plan counted the fusion frames as fractions of standing fusions (a
  thousand energy a second that did not exist), planned upgrades on it, and the hands followed it to zero from
  12:00 to 15:00. Built in order, a batch each: T-ENERGY's budget (terr26 over-refused on walking orders' whole
  draws; terr27 discounted them and held producers to the stock), the no-site strikes (terr28: one converter's
  failure had banned the type for everyone a minute), the pad walk and the plant on the producers' budget (terr29
  took the commander off the yard every chunk and drained the energy; terr30 narrowed both). terr27t: `terminal=300`
  changed nothing; the late bank is the hands'. terr30: 736 / 636 / 717 / 716 at 20:00, the steadiest batch, every
  game within 170 of the flagship. Then the columns (above): the program's ceiling with honest walks is 1,498 at
  20:00, the user's figure, and the program takes the vehicle line (an advanced vehicle constructor is 290 build
  power at speed 52 for 550 metal; the bot's is 210 at 33 for 430). terr31 regressed (520-645): with the frames in
  their own column nothing seeded the whole-unit credits and an advanced fusion started at every re-plan. terr32
  (a whole unit at half its cost of credit; producers held to their own frames' draw): 696 / 235 / 704 / 515, two
  games at terr30's level and two lost to rules meeting the program's new wants (two factories committed in the
  first minute; the vehicle plant's pad past the hundred). terr33 (one commitment at a time; the pad test by the
  factory's own product inside its yard) measures the columns a third time. Open after that: the late orders
  (converters by the hundred the sites cannot take; far upgrades walking), his roaming commander, then step 3.
- Decision 6 amended while building: the converters' threshold is dropped from the program (a full relaxation,
  converters bounded by capacity alone). The engine's converters burn what sits above three quarters of storage;
  a plan that runs them at their capacity has the stock climb to that level once and burn the inflow from then on,
  so the relaxation costs one transient of three quarters of the energy storage, not a steady-state error. The
  verifier measures it.
- 2026-10-24: the energy margin. The reserve (a share of the energy storage in stock) does not keep the hands out
  of a stall: storage is eight seconds of the mid-game's usage, and the hands' overshoot is a flow, whole units
  started where the plan runs fractions (terr38, `reserve=0.5`: every game stalled as before). A new row from the
  second bucket, (1 + margin) times the build draw at most the bucket's production with the converters on what is
  left (`energy_margin`, `emargin=0.25` the bot's default, `--energy-margin` on the CLI, 0 for the ceiling): the
  plan buys its energy a quarter ahead of its draw and the hands' overshoot lives in the quarter. terr42: 133 /
  142 / 193 / 124 at 10:00 against 131 / 137 / 174 / 75 with the hands' stall rules alone and 112 / 110 / 95 / 112
  before them; 20:00 unchanged, about 750.
- 2026-10-24 evening: the reclaim column and the farm arms (the section above), from the user's benchmark example
  and his word to iterate the program for the actions his curve needs. The program beats the curve at every mark
  once given his walk and his room (62 / 316 / 1,058 / 2,357), and at 10, 15 and 20 with the hands' walk (50 / 258
  / 937 / 2,087); the hands now owe it three things, in the order the marks say: the extractor chain (the 5:00
  mark is theirs alone), the farm's placing (turrets in a block, tier-2 frames inside their reach, the turret cap
  as the arm), and the reclaim order (`ReclaimUnit`, which the protocol carries and the bot has never sent).
  The bot's defaults since the farm (terr54, 2026-10-24 night): `maxnanos=300`, `help=40`; `reclaim=off` still.
- 2026-10-24 late night, three options from the scaling arc, each measured once and two left off: `farm_big`
  (a turret's help only on a turret or a building of that much build time; at 20,000 the program bought constructors
  from the first minute for the dearer placing and lost every mark, terr58), `bank` (the metal bank at the horizon
  counted as made, to stop the end-of-horizon dump of 400 solars and converters into the last buckets; it hoarded
  instead, since metal kept read as income while the marks are income, terr64), and the horizon at 25:00 in a game of
  20 (`end=1500`, on: the program invests for the rate, his composition stood at 20:00, the marks unmoved, terr65).
  The pair for a turret on a factory (Assist is for mobile builders) is the program's next want: his 29 turrets on the
  advanced plant at 10:00 made a constructor every 20 s.
- 2026-10-25, the turrets' assist (`nano_assist`, `--nano-assist`; T-PLANT in the bot's README): a turret's Assist
  pair on the products of an advanced factory (one whose constructors build an extractor upgrade), under its own row
  bounded by F times the power of the advanced factories standing or committed (`n0`), not the plan's fractions.
  Three readings in the engine (terr67-69): against the plan's fractional factories a tenth of an advanced plant with
  ten times its power in turrets was a whole plant's production for a tenth of its price, and the plan held the plant
  at a fraction to 8:00; on every factory's products it pumped basic constructors from the vehicle plant (38-50 by
  10:00 against 29-36) and the engine's extractors did not follow (the constant extractor walk prices the eightieth
  spot as the first); on advanced plants' products only, the plan asks for 1-2 turrets' worth at 10:00 and 3-6 at
  14:00 against his 29, since its advanced constructors (one every 16-20 s) are what its metal and upgrade pace want.
  The marks level; the arm off by default. The test: an advanced lab with six turrets makes 7.5 constructors in a
  minute with the option against 3.2 without, and a basic lab gets no pair.
- 2026-10-25, the extractor walk (terr73-83): `walk_extractor` 15 (65 before), `walk_upgrade` 40, the commander's
  40 (16 before), and in the territory bot the extractor's walk grows from 15 at the start to `walk_late` 40 at
  `walk_late_from` 600 s by the plan's t0 (lib.rs): the realized order age at a constructor's extractor frame is
  12-19 s through 9:00 once the constructors chain spot to spot and 28-42 s from 10:00, the 65 having been measured
  when the hands walked home between spots. At 65 the plan from the start was 18 extractors and 43 a second at 5:00
  with 11 constructors, the hands' figures exactly; at 15 it is 25 and 59 with 8, thebluegecko's 26 and 62 with 9;
  the annealer over the simulator gives 27 and 59.8 for five minutes. His 5:00 is above both models: the sixty-second
  buckets from 1:00 (a constructor finished in a bucket builds from the next), the reserve, and the first factory
  at a fraction (0.9 at 5:00) are the program's pessimism. One 15-s walk for the whole game had the plans from
  10:00 want 21-24 constructors against 25-28 and 20:00 read 40-70 under; grown to 40 by t0 it reads 962-1,007.
  The commander at 16 was given nine spots by 5:00 and the hands walked it 1,950 out for the fifth; at 40 two spots
  and ten solars, his commander's day. `--reclaim` takes nothing down before 13:00 from the start: the plant's
  reclaim at 4:41 is not in the program's interest while it wants the plant's constructors.
