# The plant ramp in the program: turrets at the plants, bounded by their rings (proposal, 2026-10-25)

A proposal, not built. Three changes to the territory planner (`crates/buildorder/src/lp.rs`, `docs/design/2026-10-20-territory-lp.md`)
so that its plan can contain what thebluegecko's game does from 7:48: construction turrets round the advanced plant,
an advanced constructor every few seconds, and upgrades beside home while the far spots are still being taken. The
first change is the one asked for first, the ring bound; the other two are what makes the program want it.

## Where we stand

| | 5:00 | 10:00 | 15:00 | 20:00 |
|---|---|---|---|---|
| the program's plan from 0:00 (terr95 game 00) | 62 | 220 | 579 | 1,315 |
| realized, terr95 | 50-55 | 163-204 | 471-520 | 969-1,032 |
| thebluegecko against InactiveAI, 2026-10-21 | 62 | 213 | 651 | 1,706 |

The plan matches his curve to 10:00 and falls under it after, because what he does from 7:48 is not in it. His record
(`run/matches/1792859881-replay-gecko-inactiveai-comet-2026-10-21/record-0.jsonl`, read with `run/record_builders.py`
and `run/eco_marks.py`):

- 29 construction turrets within 400 of his advanced vehicle plant by 10:00, 97-362 from its centre, 9.7 times the
  plant's own 600 build power; the first three raised at 4:35-5:28 beside the plant's site, before the plant.
- An advanced constructor finished every 6-8 s from 9:25 (16,000 build time at 6,400 power is 2.5 s of work; the rest
  is the pad), fifteen by 10:15; ours come every 28-30 s from one unassisted plant.
- His advanced constructors' first job was a fusion at 6:45, then upgrades from 7:00: 1, 2, 5 and 10 mohos in
  minutes 7, 8, 9 and 10, while his basic constructors were still placing 7, 11, 4 and 2 basic extractors in the same
  minutes. The program's plan upgrades nothing before 11:00, after the last basic spot.
- His basic constructors built 16 turrets in minute 8 and 6 in minute 9, after the extractors were mostly placed.

Why the program has none of this, each with its receipt:

1. The turret-assist pair is off (`nanoassist` 0). A turret in the plan helps frames and never a plant's products.
2. When on, it is capped at `nanoassist` times the advanced plants' own power, and turrets at six per factory
   (`nanos_per_factory`), because a turret in the model is 200 build power with no reach to run out of: the first
   solves bought 285-441 turrets, and without `help_factor` the program ran a converter spiral to 4,000 metal a second.
   The caps stand in for geometry the model cannot see.
3. The pair is for advanced plants' products only: on every factory's products the program pumped 38-50 basic
   constructors from the vehicle plant by 10:00 and held the advanced plant at a fraction (terr67-68).
4. Even with the pair on, the program asked for one or two turrets' worth at 10:00 and three to six at 14:00 (terr69),
   because in its model an advanced constructor's upgrades wait for the last basic spot: one walk price for the whole
   map makes the eightieth spot as cheap as the first, and an advanced constructor born at the plant is priced at a
   constructor's chain walk of 15-40 s to a spot that is really 2,500 out.

## 1. The ring bound

**What exists.** The hands already place a plant's turrets in a ring (T-PLANT, `hands.rs` `farm_site`): the 16-grid
points within `PLANT_RING` (330) of the plant's centre, free by the farm's rules (ground the footprint can take, not in
any factory's yard or exit lane, `FARM_GAP` clear of every standing building, frame and pending site, not crowded by a
mobile unit), the point nearest the centre first. When no point is free the `plant:` line says how many candidates
there were and why each was out. A turret at 330 reaches the plant: `NANO_REACH` 400 plus the plant's own radius.

The program's turret assist today (row 3, `lp.rs`): the sum of the turrets' Assist flows on advanced products in a
bucket is at most `nano_assist` times the standing advanced plants' power times the bucket's length. The power on the
right is the plants' own; the number of turrets a ring holds does not appear.

**The change.** One row per factory type and bucket in place of the single turret row:

    sum over turret Assist pairs on F's product columns of x[p, t]  <=  turret power × ring(F) × dt(t)

where `ring(F)` is the ring capacity of the plants of type F standing or committed: for each such plant, the turrets
standing or framed within its ring plus the free ring points, summed. The hands compute it every plan (the candidate
enumeration in `farm_site` factored into `plant_ring_capacity(centre)`), and `lib.rs` passes it in `LpOptions` as
`plant_rings: Vec<(factory unit, capacity)>`; the CLI takes `--plant-ring F:N`. A plant that is a fraction in the plan
has no ring (terr67's lesson stands: the bound counts `n0`, the plants standing or committed, never `n[f, t]`).

`nano_assist` and `nanos_per_factory` go. The turrets' count is then bounded by the farm's `max_nanos` where the farm
is on, and by the rings and the frames to help where it is not; the ring bound replaces both caps at the plants.

**What it does and does not do.** It lets the plan hold up to `ring(F)` turrets' worth of assist on a plant: about
30 for an advanced plant in open ground, as his held 29 (the hands' count will say exactly; the plant's own yard and
lane and the base's buildings take points). It does not make the program want them: at the standing value of an
advanced constructor in the model (point 4 above) the program asked for one or two even when the cap allowed six.
Alone, the ring bound is the honest cap and nothing more. The two changes below are what makes the plan want to fill
it.

**Both plants.** The row is per factory type, so the vehicle plant's products get a ring of their own and the
restriction to advanced plants (point 3) can be lifted. Lifting it alone would bring back terr68's shape, basic
constructors pumped from a ringed vehicle plant, because in the model a basic constructor's extractor is cheap at
any distance. Change 3 bounds how fast a plant can make them at all; change 2 is what makes the far spots dear. The
proposal lifts the restriction only with both in, with the arm `plant_products=advanced|all` to measure it apart.

## 2. The advanced constructor's spots priced from home

**The flaw.** The basic extractor's walk is one price per builder class: a constructor's 15 s growing to 40 by 10:00
(the realized chain: a constructor out among the spots walks 300-500 between them), the commander's from the free spots
within its leash (terr90). An advanced constructor is priced as a constructor. At 7:00 it stands at the plant, the free
spots lie 2,000-3,000 out, and the plan sends it there for 15 s of walk. Its alternative, the upgrade beside home
(a moho at 620 metal for 2.5 times a basic's draw, no walk), loses to a 50-metal spot at every instant until the spots
run out, so the plan's upgrades begin at 11:00, his at 7:00.

**The change.** The commander's template of terr90, applied to the advanced constructor: its basic-extractor pair
priced at the mean distance from home of the free spots at its speed plus the ring overhead, re-derived every plan
(`lib.rs` beside `commander_walk_extractor`; a third branch in the walk closure, `advanced(b)`). Early the free spots
are near and the price is small; from 6:00 they are 2,000 out and the price is 40-60 s of a 180-power builder, and the
moho beside home wins. The basic constructor's chain walk and the upgrade's 40 s stay as they are.

**What it does.** The plan's advanced constructors upgrade from about 7:00 as his do, which raises what an advanced
constructor is worth and so what a faster plant is worth, which is what the ring bound then lets the plan buy. It
also stops the hands' advanced constructors being sent across the map for a basic spot.

## 3. The pad: a plant makes one product at a time

**The flaw.** A plant's output in the model is build power over build time: ten turrets on an advanced plant make
ten times the constructors, without limit. The game's plant finishes a unit, rolls it off the pad, and starts the
next: his advanced plant at 9.7 times its power made an advanced constructor every 6-8 s, of which the work was 2.5 s.
The caps of point 2 exist to stop the model exploiting the absence of this.

**The change.** One row per factory column and bucket:

    sum over F's product columns of x[p, t] / cost[p]  <=  n0[F] × dt(t) / roll_off(F)

the products a plant can finish in a bucket at most the bucket over the pad's time, counted on the plants standing
or committed. `roll_off` is per factory type, read from the records at saturation: his advanced vehicle plant 6-8 s
a constructor at 6,400 power (2.5 s of work, so 4-5 s of pad); his vehicle plant 10-12 s a constructor at 395 power
(10.6 s of work, so a second or two). The constant starts as the pad's share alone (`--roll-off avp:4,vp:1.5`) and is
calibrated from our own batches' `begun:` lines once plants run assisted.

**What it does.** With it the assist's value saturates where the game's does, and `assist_factor` (mobile guards at
twice the plant's power) can go the same way as the turret caps: the guards' value is bounded by the pad, not by a
multiple. It also bounds terr68's pumping physically: a ringed vehicle plant makes at most one constructor per
`roll_off + work`, not one per the ring's power.

## The hands

Nothing new for the ring bound: T-PLANT places a turret into the ring of the plant the program puts the most turret
power on, while the program's assist over the next `PLANT_LEAD` runs half a turret ahead of the turrets standing,
framed or ordered within reach, and a turret within reach of a plant at work guards it first. For change 2 the
hands' `basic_to_upgrade` already takes the nearest basic to the builder. For change 3 nothing: the engine enforces
the pad.

Two readers to add before the first batch, both cheap: the `turrets:` line to carry each plant's ring capacity and
the turrets in it, and a per-plant cadence (products finished per minute) from the `begun:` lines, so the roll-off
constant is measured, not assumed.

## Measurement

One batch each, the benchmark as it stands (from his start, `--starts 941,2534:6899,684`, the four seeds), judged by
the marks moving together and by three things his record gives numbers for: turrets within 400 of the advanced plant
by 10:00 (his 29), advanced constructors finished by 10:15 (his 15), the first upgrade (his 7:00).

1. The ring bound alone, advanced plants only. The expected reading is the honest cap with little change in the
   marks: the plan may still ask for few turrets. It tells us the capacity the hands count and whether the plan's
   demand moves at all once the cap is a ring rather than a multiple.
2. Plus the advanced constructor's spots priced from home. Expected: upgrades from about 7:00 in the plan, the
   plan's demand for advanced constructors and for turrets at the plant up, 15:00 and 20:00 up.
3. Plus the pad, and `assist_factor` gone. Expected: level with 2 or a little under, the plan's turret count at the
   plant settling where the pad saturates.
4. Both plants (`plant_products=all`). Expected: the vehicle plant ringed early if the plan finds constructors
   worth it under the pad; terr68's pumping is the failure to watch for.

If the ring bound alone changes nothing (1), that is the expected result and not a reason to stop: it is the cap;
the value comes with 2.

**Batch 1, measured (terr96, 2026-10-25; the user: "Start with batch 1 alone, and the mobile guard's cap stays until
the pad is calibrated").** Built as above: `LpOptions.plant_rings` and the per-factory-type row in place of
`nano_assist`; the hands' `plant_rings` counting each plant's ring by packing turrets into it as `farm_site` would;
the `turrets:` line carrying the rings; `run/plant_cadence.py`. The hands count the advanced plant's ring at 24-34
turrets (his 29 within 400) and the vehicle plant's at 14-37, shrinking as the base fills round it; four turrets were
placed in the ring and no search failed. The program's demand did not move: no turret assist to 10:00, 1.5-3.6
turrets' worth at 10:00-12:00, 3-4 turrets at the advanced plant by 12:00-14:00; the plant's cadence 28 s a
constructor at 6:00-8:00 and 12 s by 12:00, as before. The marks level with terr95 (5:00 50-57, 10:00 156-241, 15:00
464-490, 20:00 975-1,081). The cap is honest; the demand is batch 2's.

**Batch 2, measured (terr97), and its premise corrected.** Built as above (`LpOptions.advanced_walk_extractor`,
derived from every free spot's mean distance at the advanced constructor's speed). Level: 5:00 48-55, 10:00 156-195,
15:00 440-520, 20:00 1,010-1,033; the first upgrade frame 7:24-9:07 against terr96's 7:46-8:59; the program's turret
demand as in terr96; the plan's forecast from the start unchanged to the digit. The premise of change 2 was wrong on
two counts, read from the orders by builder type: the hands never sent an advanced constructor to a basic spot before
the change either (its first job is a fusion in minute 6, then upgrades, fusions, help on advanced fusion frames and
guard chunks on the vehicle plant), and the plan's upgrades do not wait for the last spot because of the walk price:
they begin when the spots run out and the metal allows, 7:45-9:00 in the hands and from 11:00 in the plan's own
count, where his begin at 7:00 and run 1, 2, 5, 10 a minute: the same shape a minute later. The price is kept as the
honest one. What holds the program's demand for the plant's turrets down is therefore not answered by this proposal's
second change, and change 3 is a cap, not a demand. Open, and to read before another batch: where the plan's metal
goes in minutes 10-15 against where his went (his 311 turrets in the farm by 15:00 against the plan's 47, his
advanced fusions and converters' timing), with `run/eco_marks.py --shape` on his record and the plan's `targets:`
lines.

## What could go wrong

- terr67's shape, the advanced plant held at a fraction while turrets are bought for it: guarded by counting
  standing plants only in both new rows. If it returns, the commit's affordability (T-COMMIT) is the lever, not the ring.
- terr68's shape, basic constructors pumped: the pad bounds it; the arm keeps advanced plants only as the fallback.
- The metal: 29 turrets are 6,670 metal and 93,000 energy. The program weighs that against extractors and fusions;
  his record shows the weighing is right at his income (97-130 a second at 8:00). Ours is 94-128 at 8:00 now.
- The farm's `help_factor` and `max_nanos` are untouched; the ring bound is for the plants only.

## Open for the user

- Start with batch 1 alone, or 1 and 2 together? Alone is the clean reading of the cap; together is one batch fewer.
- The roll-off constant: the pad's share from his record (4-5 s for the advanced vehicle plant), or measured first
  from our own plants under the commander's guard (one batch with the cadence reader and no model change).
- Whether `assist_factor` should go with change 3 or stay as a belt beside the pad until the pad is calibrated.
