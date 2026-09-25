# Target design: the formation as a transform of the order, not a control loop

Written 2026-09-25 by Claude (Fable 5.1) before any code, at the user's direction after the chaser fix ("if that was
the formation system trying to run then it was very broken and just changing some coefficients probably isn't the
full answer"). Re-derives what H-MICRO-FORM is for from the record and the harness, and says what it should do
while a body moves to a far goal against what it should do at contact.

## What the formation rule is for (the evidence it rests on)

1. **Spacing and one point per unit.** The pros give a group one point per unit, 64 apart, on a line; at contact
   their soldiers stand 67-70 from the nearest friend and a friend is on 22% of their lines of fire; ours stood at
   25-40 with 43% (`docs/knowledge/formations.md`, K-form-pros-order-one-point-per-unit, -stand-two-hulls-apart,
   -friend-on-the-line-twice-ours). In the arena the rule reached the pros' line-of-fire share live (lane-ab: 23%
   at contact against 32-33%; flank-ab: 26% against 31%) and halved the muzzling by friends.
2. **Standing at contact instead of crowding into one point.** Every intent source gives a body one point, and the
   engine turns it into a crowd that pushes through its own front (the queued-rear arc: the walk-up's cost is the
   rear's shells on the front, 40% of the damage done in the bank-1 13:50 cut). A unit with a target in reach
   should be given nothing (a Stop drops the weapon's target; every order costs shots: form6/form7 smokes).
3. **The rear at the ends of the front's line** (K-form-the-rear-at-the-ends-not-in-the-back): +0.15 and +0.06 over
   the ball on the two cuts, friendly fire 40% to 4%.

Nothing in that evidence asks for a **moving slot**: a rank that leads the body's centroid and is re-issued as the
body walks. That was my design choice on 2026-09-25 ("slots move with the centroid, so the body walks together
and the slow are not left behind"), and it is the control loop the user saw.

## What it does today, and what that costs

- On the march (no armed enemy within 700 of the body's centre) every unit in a body gets a `Fight` at a slot
  `lead` ahead of the centroid (150 until today, now three seconds of the fastest member's speed), re-issued every
  time the slot has moved half the lead: a goal that is always within the unit's braking distance for the slower
  lead (K-engine-a-short-move-order-brakes-the-unit: Rovers at 75% of their speed under it, chase-lane-fav-prefix),
  and an order stream of its own: **0.30 orders per soldier-second on lane-only ticks** in lead-ab-new2 (51,402 in
  8 games), as many as the whole think pass issues.
- At contact a unit with nothing in reach got `Close`: a `Fight` at a point short of the nearest enemy (0.92 of
  reach until today), re-issued every 64 elmos the enemy moved. Against a fleeing Flea that is a goal 130-280 elmos
  from a Rover re-issued twice a second: standing-1 3:29-3:53, the Rover at 33-44% of its speed, the orders in the
  record 64-66 elmos apart with no decision row behind them. (The follow rule, gated at 2 s and 150 elmos, was
  not it.)
- The host's own re-issue of the group order (the raid and army every 4 s, follow every 2 s) reaches the engine
  unchanged: for those seconds the units walk to the one point until the slot moves enough to be re-issued. The
  lane's "same order again" rule keeps the claim but does not put the per-unit point back.
- Move failures (the engine's `UnitMoveFailed`, a goal it cannot reach or a unit that cannot get there) a game:
  8 with no lane, 31 with the lane before today's lead change, 50 with it, 106 with the lane of 2026-09-24 (its
  100-elmo flee steps); every lane loop makes them, the plain order almost none.
- The march's cohesion (the slow not left behind) is already H-ARMY-MARCH's job for waves, squads and the
  pianist's advances (`march.rs`); the lane duplicated it with a worse tool.

The gains measured for the rule (the Blitz pack +0.32 against +0.15; the cuts) came from the contact behaviour
(spacing at contact, the stand, the flank), not from the march loop: the form-* on arms' spacing at contact (59)
is set when the body is engaged (the rank re-forms on the enemy's direction), and the flank arm's gains came with
the rank at the front's depth.

## Target behaviour

The lane's formation is **a layer over the host's orders, as CustomFormations2 is over a human's drag: it rewrites
a group order into per-unit orders at the moment the host gives it, and gives no order of its own between the
host's.** Concretely, `Lane::tick` takes the host's commands of the tick mutably and:

1. **Bodies** as today (units under one group order, chained within 400, raiders and line units apart, at most 12).
2. **On the march** (no armed enemy within 700 of the body's centre): a `Fight` or `Move` to a point is rewritten,
   for each member the host ordered this tick, into the same kind of order at that member's slot **at the
   destination**: ranks of six across the approach, 64 apart, centred on the goal, assigned by least travel,
   snapped to reachable ground. Nothing is issued between host orders: the units walk at full speed to their own
   points and arrive as a line. A member the host did not order this tick is not touched. An `Attack` on a unit
   is left as it is (the engine walks each unit into range of its target).
3. **At contact** (an armed enemy within 700 of the body's centre) the shape is the one that won the cuts: one rank
   across the enemy's direction at the depth of the standing front. A member **with an enemy inside its reach** is
   given nothing, and a host order to it this tick is dropped (it is shooting; a `Fight` to a far point would walk
   it off its target). A member **with nothing in reach** gets a `Fight` at its slot in that rank when it is more
   than 48 from it, else a `Fight` at the point one lead (three seconds of its speed) past its nearest enemy along
   its own line, so a line's ends curl in and a chaser's goal is never within its braking distance; these are
   issued when the body enters contact, when the host re-issues the group order, and when the member's stance
   changes (in reach / not), never on a distance the slot has moved.
4. **Leaving contact** (no armed enemy within 700 any more): the host's standing order returns as the march's
   per-unit point, once.
5. The flee's release gives a unit its per-unit point back, not the host's one point.
6. **Deleted:** the slot advance (`form::anchor`, `LEAD`, `LEAD_SECONDS`, `lead_for`), `FORM_REORDER_SHARE`, the
   `Advance` and `Close` stances as re-issuing claims, the `holding` report for anything but a standing unit (so
   the duel director's idle re-send treats a walking formed unit as it treats any walking unit).

What this gives up: the body arriving together (H-ARMY-MARCH keeps that where it was wanted), and the moving
rank's spacing en route (the pros' spacing is at contact, and the contact rank sets it). What it removes: 0.30
orders per soldier-second and every near goal the lane made.

The retired H-MICRO-SPREAD (2026-09-20) put per-unit points around the destination too and was dropped for "nothing
measurable in the arena" and move failures (225 a game against 155). Its points were an 8-wide block with ranks
behind at the destination for committed attackers only, with no contact behaviour; the failures were points on
unreachable ground. Here the points are snapped to reachable ground and the measure is the harness first; the
move failures per game are a column of the judgement.

## Judged by (the harness before any arena batch)

1. **The chase**: `duel --chase 60 --lane on` against `--lane off` (a host that re-sends a far goal every 2 s, as
   the follow rule does), 4 Rovers chasing 4 Fleas and 4 Blitzes chasing 4 Pawns: the lane's chaser speed should
   equal the director's (168 and 99), where today's lane gives 126-138 and 87.
2. **The form-* pairings** on the same sites and seeds as before, `on` against `off`: the Blitz pack, the Stout
   mirror, the Stouts against the bot mix, the user's mixed force; the contact numbers (spacing, friend on the
   line, muzzled by friends, friendly fire, exchange) should hold where the rule won and not fall where it drew.
3. **The two cuts** (bank-1 13:50, family-1 11:05) `on` against `off`: +0.84 / +0.71 against +0.69 / +0.66 today.
4. A null arm (the lane present, every rule off) once.
5. Then the arena: 24 games an arm on lane-ab's settings against the last build (kept as a binary), read with the
   speed-by-orders measure (the lane-tick orders per soldier-second should fall from 0.30 toward 0), the move
   failures per game, `run/micro_ledger.py`, the fire and shape instruments.

## What building it taught (the ledger's t-*, u-*, v-* rows)
- **Points at the destination alone** (build A) free the chaser (Rovers 146 of 168 at the follow's cadence, the
  loop 126, the director alone 162) and keep the cuts (+0.83, +0.68), but contact en route meets the spawn's
  spacing and the Blitz pack's, the Stouts-against-the-mix and the mixed force's gains go (+0.18 / -0.04 / -0.26).
- **Form up first, then march** (build B: one queued pair per host order, a spaced rank three seconds ahead then the
  destination points) brings the pack back (+0.26) but a `Close` at the nearest enemy folds the formed line into a
  ball in the last 200 elmos (spacing 43-49 at contact, the Stout mirror -0.08).
- **Close along the heading on the unit's own line** (build C, kept): nothing lost to the ball within the noise
  (Blitz +0.24 against +0.15, Stout +0.04, Stouts-v-mix +0.02, the mixed force -0.11 against -0.18, the cuts +0.82
  and +0.66), the chaser at the director's speed (156 / 98), the friend-on-the-line share at contact 9-30% against
  the ball's 6-37%. The loop's Blitz +0.32 and Stouts-v-mix +0.13 are not reached, and the nearest-friend spacing
  at first damage reads 41-45 where the loop read 57-62: the line forms (the line-of-fire share says so) but does
  not read as spaced at first damage. Open; the suspect is the re-slotting of the rank on every host re-issue (a
  unit crosses to a new slot each time the director re-sends the idle), which a stable assignment would remove.
- The order stream: one Blitz duel under the transform issued 42 claims in 24 s (11 March once, then Flank, Close,
  Stand once each) where the loop issued two a second.

## Status
- 2026-09-25: written; nothing built.
- 2026-09-25 late: built as build C (the design's sections 1-6; `Lane::tick` takes the host's commands mutably and
  rewrites them). Judged in the harness (above). Not yet: the arena batch (section 5 of "Judged by"), the stable
  slot assignment, a live reading.
