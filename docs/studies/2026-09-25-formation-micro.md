# Formation micro in the engine: the lane driving one side of a duel

Part B of `docs/design/2026-09-25-formation-micro.md`, 2026-09-25, by Claude (Fable 5.1). The duel harness runs the
bot's control lane (`crates/micro`) for one army (`duel --lane`), so a fight can be fought with the lane off (the
director's plain attack-move, the harness as it was), with the old lane (`old`: flee, fan, kite; focus while it
existed) and with the formation (`on`: the old lane and H-MICRO-FORM), on the same sites and seeds, and read with
the fire instrument (muzzled seconds by cause, friendly fire and its victims, the exchange) and the shape instrument
(nearest-friend spacing and the friend-on-the-line share at contact, `run/replays/shapes.py`'s definitions, so the
numbers sit beside the pro survey's: `docs/knowledge/formations.md`). Every arm is 8 duels unless said; `margin` is
the first army's value left minus the other's; `k/l` is metal killed per metal lost over the arm; `nn` the median
nearest-friend distance at contact and 10 s later; `fol` the share of units in reach with a friend within a hull of
their line of fire; `ff` friendly fire over all damage done. Comparisons are `run/duel_ab.py <off> <arm>`.

## What voided the first day's batches, and the checks that stand

Two things had to be found before any number below meant anything.

1. **The lane took the other site's units.** A match fights two sites at once and the team's snapshot holds every
   unit of the team, so the lane on one site treated the other duel's units of its team as its own, uncommitted, and
   fled them from everything. The tell was a null arm: the lane present with every rule switched off lost 0.27 of
   margin to the plain order (form2-blitz-old, 7 losses of 8 in fights twice as long) while the lane's own debug
   count showed no claim on its own army. Fixed with `View::mine`; the null arm then matched the plain order
   (Blitz +0.152 +- 0.049 against +0.154 +- 0.047; Stout +0.026 +- 0.048 against +0.034 +- 0.057). Every lane arm
   before the fix is void (the ledger names them); the retirements taken on them were undone and re-measured.
2. **Single-rule ablations had never worked.** The lane checked `enabled` only for H-MICRO-LANE and H-MICRO-FORM;
   `WITHIN_REASON_DISABLE=H-MICRO-FOCUS` ran the whole lane. Each rule is gated by its ID now, in the bot and the
   harness.

Smaller: a stray sweep bomb wiped one army in 4.9 s with nothing dealt by either side (one row of ~250;
`duel_ab.py` drops such rows), and two runs of the same `off` arm differ by more than their standard errors
(Blitz +0.154 +- 0.047 and +0.210 +- 0.050): read the between-batch spread, not the within-batch error, as the noise.

## What the smoke duels taught (one site, 1-2 duels each, `WITHIN_REASON_MICRO_DEBUG` naming what held each muzzled unit)

11 Blitzes against 22 Pawns. With the lane off the Blitzes win (+0.26) firing 6.6 shots per in-reach second, 0%
muzzled. The first formation re-ordered a unit between standing, stepping back and closing every few frames: 1.8
shots per in-reach second, 24% muzzled, cause "clear". One Stop per stand: 2.4. The old lane alone: 24-33% muzzled,
21 of 36 muzzled seconds under H-MICRO-FLEE, which fled seven to ten Blitzes at contact (their fire on the ball,
2,100-2,450 a second, is "lethal" for every 730-hp unit in it) in a fight the Blitzes win. Strength odds (damage a
second times health: 22 Pawns out-shoot 11 Blitzes and do not out-fight them) and an "already under the guns" test
took the flee to 0% muzzled in both arms. So: an order costs shots, a unit that walks does not fire, and the
formation's contact behaviour became one Stop, held while anything is within reach + 100.

## The batches (8 duels an arm, `--parallel 2 --sites 2`, Quicksilver Remake 1.24, speed 50)

TABLES

## The two scenarios (Comet Catcher Remake 1.8, the recorded places, `--reps 8`)

SCENARIOS

## The live game (`form-1`)

LIVE

## What it says

SAYS
