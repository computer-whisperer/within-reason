# Target design: the rear behind a firing front

Written 2026-09-25 by Claude (Fable 5.1) before any behaviour code, at the user's direction: "a body of units where
the front line stops and fires and prevents the rest from reaching the enemy". Branch `micro-formation` after the
formation arc (`docs/design/2026-09-25-formation-micro.md`), the same method: measure it in the records, cut
scenarios, reproduce in the duel harness, try the answers with on/off batches and a null arm.

## What is true today

**The instrument** (`run/queued.py`, and the same definitions in the duel harness's `fire.rs`: `engaged_s`,
`queued_s` columns; `run/duel_ab.py` prints the share). A soldier-second is *engaged* when four or more armed enemies
stand within 700 (a fight, not a mop-up: a body of 70 against eleven Rectifiers has nothing to reach). An engaged
soldier with no enemy within its reach + 20 is *queued* when a friendly soldier within 300 of it has an enemy in
reach, stands nearer that enemy than it does, and fired this second. The *geometric* count drops the shot (a
replay record has no shots), so the pros are counted that way and ours both ways.

| Game | Engaged soldier-s | In reach | Queued (shots) | Queued (geometry) |
|---|---|---|---|---|
| bank-1 | 9,418 | 40% | 25% | 37% |
| 2v1b-hard | 4,494 | 60% | 15% | 24% |
| 2v1b-hard_aggressive | 1,293 | 39% | 10% | 23% |
| family-1 | 4,797 | 54% | 14% | 27% |
| the pros (40 Comet Catcher duels at OS 40+, both sides) | 243,795 | 38% | - | **20%** |

By type: Blitzes 30% (bank-1) and Stouts 22%; the pros' Gators 21%, Blitzes 19%, Stouts 15%. By the ball: ours
rises from 10% alone to 29% with six or more friends within 120 (bank-1); the pros' from 13% to 25%. So the failure
is real and ours is worse than the pros' by 3-17 points of engaged time, worst in the biggest balls, and it is
present in the pros' fights too: a fifth of an engaged unit's time is spent behind a friend that can reach.

**Where it happens.** The ten-second windows with the most queued seconds in clashes (8+ armed enemies near):
bank-1 28:10 (450 engaged soldier-s, 127 in reach, 78 firing, 131 queued, 29%; the Fatboy fight, out-ranged),
bank-1 13:50 (307 / 104 / 42 / 95, 31%: 33 Blitzes and Stouts against 18 Pawns, Rockos and Hammers), family-1 12:30
(216 / 64 / 36 / 67, 31%: 27 against 8 Pawns), family-1 11:00 (191 / 72 / 28 / 59, 31%: 21-23 against 19-23).
The shape every time: our body two to three times the enemy's, the front in reach and firing, the rest walking
toward the same point behind it. Two scenarios are cut with `run/engagement.py --enemies truth --radius 1000`
(`docs/data/scenarios-2026-09-25/`): bank-1 13:50 (45 of ours: 23 Blitz, 19 Stout, 2 LLT, 1 Janus; 15 of theirs:
8 Rocko, 5 Pawn, 2 Hammer; live 60 s later ours 0.64, theirs 0.13) and family-1 11:05 (37: 23 Blitz, 10 Stout, 3
LLT, 1 Rover; 13: 6 Rocko, 3 Hammer, 3 Warrior, 1 Pawn; live 0.68 / 0.68).

**Why the current formation does not cure it, by construction.** H-MICRO-FORM's slots are ranks of six, 96 apart.
A rear-rank unit's slot is 96 behind the front rank; the front stands when an enemy is inside its reach by 20
(a Stout at up to 330 from its target), so the rear at 426 is out of reach and gets the `Close` stance: a Fight at
0.92 of its reach from the nearest enemy along its own line, which is where the front rank stands. It walks into the
front's backs and is held there by the hulls: queued. A body of 45 is four sub-bodies side by side, each with this
depth. The plain attack-move (`--lane off`) has every unit closing on one point, so the rear pushes through and
round the front, slowly; the engine's collision handling decides how long that takes.

**What the pros' shapes say** (`docs/studies/2026-09-24-pro-fight-shapes.md`): bodies of 6, cores 190 x 165, a
line at contact trading +5 over a blob, a wider core than the enemy's trading 51 against 49 (not significant), and
no layering. Nothing there is deeper than one or two ranks; their queue is the arriving units (arrival spread 13 s).

## The answers to try (each an arm; the lane's `--lane on` of today is the control, `--lane off` the ball)

1. **The rear files round the flanks** (`H-MICRO-FORM-FLANK`, the design's first choice). When a body is engaged
   its slots become one rank of everyone across the enemy's direction, at the depth of the standing front (the mean
   position along the heading of the units that have an enemy in reach), so a unit with nothing in reach walks to
   the end of the front line, not into its back; only a unit already at its flank slot and still out of reach closes
   on the nearest enemy to 0.92 of reach. The rank is capped at 12 per sub-body as before (700 wide); a sub-body
   beyond that is its own rank beside it.
2. **The body wider than the front** (`WITHIN_REASON_FORM_FILES=12`, a knob for the batch only): ranks of twelve
   on the march too, so there is no rear rank in a body of 12 or fewer. Tests whether the depth on the march, not
   only at contact, is the cost. Deleted after the measurement if it does not pay.
3. **The front advancing through**: this is the ball (`--lane off`): every unit closes until it has a target. The
   existing off arms measure it.
4. **Ranks rotating** (the front rank steps back through gaps after firing and the rear rank takes its place): not
   built unless 1 and 2 leave the queue where it is; an order costs shots and the engine has no strafe.

The flank slot for the raider body (the Stouts' shells on the Blitzes in front, open from the formation arc) is the
same shape as answer 1 if the raider body's rank is placed at the ends of the line body's rank rather than in front
of it: taken in the same pass when 1 pays.

## Judged by

The two scenarios and the Blitz pack, Stout mirror and the user's mixed force, `--reps 8`, arms `off`, `on` (the
control), `on` + flank, `on` + files 12, on the same sites and seeds, `run/duel_ab.py`: the queued share of engaged
seconds first (the claim: the flank arm takes it from the control's toward the pros' 20% or under), then the
exchange, muzzled seconds, friendly fire, nearest-friend spacing and the friend-on-the-line share, so that the cure
is not a return to the ball. A null arm (the lane present, every rule off) once per pairing. Then the queued share
of a live game read with `run/queued.py` beside the four games above.

## Status
- 2026-09-25: written; the instrument built (`run/queued.py`, the harness columns); scenarios cut; the control and
  ball arms of the two scenarios running; no behaviour code.
