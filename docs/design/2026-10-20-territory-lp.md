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
  prefer HiGHS from the start).
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

The plan is `n[u, t]` for every type. At each half second, for each idle builder `b` of a type that can build or
assist `u`: the shortfall of `u` is `n[u, now] - (built[u] + ordered[u])`, in units, times `bt[u]` in
build-power-seconds; the builder takes the type with the largest shortfall it can serve, placed as the hands place
things today (its spot, its site, an upgrade over the nearest basic extractor, a factory's product queued on that
factory, an assist as a guard on the factory whose products are shortest). A building starts only when its shortfall
passes one half. With nothing short, a mobile builder assists the nearest factory with a shortfall, else the
dearest frame; a factory stands idle. The order review, the retry at another site, the struck rule and the
instruments stay as they are.

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
