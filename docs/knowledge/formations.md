# Formations

How forces are shaped in fights: spacing, grouping, type mix, commanded lines. Measured on the 40 Comet Catcher Remake
1.8 duels with both players at OS 40+ (693 fights) and three of our games on the same map (bank-1, 2v1b-hard,
2v1b-hard_aggressive; 60 fights), `run/replays/shapes.py --orders --boot`, study
`docs/studies/2026-09-24-pro-fight-shapes.md` (definitions of fight, body, core, friend on the line there). The records
cannot show shots or friendly fire for the pros; every muzzling number here is geometry. Scope: that map, game
test-31383, tier 1.

### K-form-pros-order-one-point-per-unit
**Claim.** High-OS players give a group its own point per unit on a straight line spaced about two hulls apart, and we
give a group one point. The spacing, not a line abreast, is what arrives: the pros' cores at contact are blobs more
often than lines.
**Status.** measured (2026-09-24). Last group order (3+ of the fight's participants, one frame) in the 30 s before
contact: pros one point per unit in 753 of 1,070 sides (70%), one shared point in 223 (21%); targets' straightness
(minor over major principal extent) median 0.06 and spacing median 64 elmos (833 orders); 326 of 755 straight lines at
60+ degrees to the travel, 242 at 30-60, 187 under 30. Us: one shared point in 34 of 44, one point per unit in 2.
At contact the pros' cores (3+) are line / blob / stream 27 / 57 / 16% of 731.
**Would be wrong if.** The per-unit points came from something other than a drawn formation (a widget spreading plain
clicks), which the record cannot tell; or the same count on another map or pool showed shared points as the norm.
**Used by.** H-MICRO-FORM (one slot per unit, 64 apart, across the heading).

### K-form-pros-stand-two-hulls-apart
**Claim.** The pros' soldiers stand about two hulls from their nearest friend at contact; ours stand at about one.
**Status.** measured (2026-09-24), nearest-friend distance in the body at t0 and t0+10: Stout (hull 32) pros median 70
elmos, 37% within 48 (446 unit-snapshots), us 40, 79% (799); Blitz pros 67, 38% (4,425), us 25, 78% (435). Incisor 48,
Grunt 40, Brute 54, Pounder 115 for the pros.
**Would be wrong if.** A rerun with our new records showed our spacing at 60+ without a spacing change in the bot (the
three games being unrepresentative), or the pros' spacing on another map at one hull.
**Used by.** H-MICRO-FORM (the slot spacing).

### K-form-friend-on-the-line-twice-ours
**Claim.** A friend stands on the line of fire of about twice as many of our units in reach as of the pros'; part of the
gap is that we fight in bigger bodies, the rest that we pack them tighter (at the same size we are still 18 points
higher).
**Status.** measured (2026-09-24). Units with an enemy in reach and a friendly soldier within 24 elmos of the segment to
it: at contact pros 542 of 2,429 (22%), us 60 of 140 (43%); 10 s in 31% against 54%. Stouts 24% of 228 against 53% of
247. By body size (t0 and t0+10): 3-6 soldiers pros 17% (1,520), 7-12 30% (1,594), 13+ 41% (1,463); us 13+ 59% of
227 (36 bodies). Median body at contact: pros 6 soldiers (961), us 13 (34). Among the pros the share did not predict
the trade: a third or more on the line traded 4.0 points better than less, interval -3.6 .. +11.5 (141 / 141 sides).
**Would be wrong if.** A shots-based count (run/fire.py's muzzled seconds, our records) did not fall when our bodies
were spaced and cut to the pros' size.
**Amended 2026-09-25 (lane-ab, 24 games an arm, the heuristic bot against medium on Comet Catcher).** It fell.
With H-MICRO-FORM the friend-on-the-line share at contact is 23% of 1,900 units in reach (402 fight sides; the pros'
22%) against 32-33% for the old lane and no lane; the nearest friend 51 against 40 (the pros 67-70); muzzled seconds
by a friend on the line 737 of 51,475 in-reach seconds against 1,399 (old) and 3,143 (none). Supported, live.
**Used by.** H-MICRO-FORM (ranks of six, bodies of at most 12); the duel harness's shape instrument measures it (`fol_in`, `fol_blocked`).

### K-form-pros-fight-with-pure-bodies
**Claim.** The pros rarely fight with raiders and line units in one body, and when they do there is no fixed layering
to copy; we fight mixed most of the time.
**Status.** measured (2026-09-24). Cores of 3+ holding both short-range (reach <= 250) and long-range (>= 300)
soldiers: pros 73 of 731 (10%), 71 of 212 (33%) when the side owned 2+ of each on the map; us 20 of 33 (61%), 18 of
23 (78%). In the pros' mixed cores the short-range median stood ahead (> +50 elmos) in 38%, beside in 29%, behind in
33% (73). Mean front rank (1 front, 0 back) is 0.45-0.60 for every common type. The pros' mixed cores did not trade
differently from pure ones: +3.4 points, interval -2.8 .. +8.0 (70 / 634 sides). The pool's armies are raider-heavy
(Incisor, Blitz, Grunt), so a Stout-and-Blitz army at 16+ is thinly sampled.
**Would be wrong if.** A pool with more tier-1 line armies showed mixed bodies with the raiders consistently in front or
behind.
**Used by.** H-MICRO-FORM (raiders and line units under one order are two bodies).

### K-form-a-line-at-contact-trades-better
**Claim.** Among the pros a force that meets the enemy as a line (across at least twice along) trades a few points better
than a blob and more than a stream; front width relative to the enemy's does not matter by itself.
**Status.** measured (2026-09-24), weak. Metal-weighted exchange share: line 52% (192 sides), blob 46% (402), stream
43% (110); line - blob +5.2, interval +0.8 .. +10.1 over resampled games; line - stream +8.1, -0.1 .. +18.4. The
eventual winners' lines traded 60% (102), the losers' 40% (90). Wider core than the enemy's 51% against 49% (165
pairs). Observational: a stream may be a chase or a retreat rather than a choice of shape.
**Would be wrong if.** Our arena games with the micro engine forming lines at contact traded no better than its blobs
over 24+ games.
**Used by.** H-MICRO-FORM (one rank across the heading).

### K-form-pros-raiders-fight-at-reach
**Claim.** The pros' short-range units hold the edge of their reach in a fight; ours close in.
**Status.** measured (2026-09-24). Distance to the nearest enemy combatant over own reach, unit-seconds with one within
reach + 100: short-range pros median 0.94 (86k), us 0.86 (2.9k). Line units 0.87 against 0.88.
**Would be wrong if.** The difference vanished when counted only for Blitzes against the same enemy types.
**Used by.** H-MICRO-FORM (a unit closes on the nearest enemy to 0.92 of its reach and stands there; the raider step-back tried first cost shots and was dropped, form-smoke).

### K-form-a-fifth-of-an-engaged-unit-waits-behind-a-friend
**Claim.** In a fight, a soldier with no enemy in reach stands behind a friend that has one and is shooting for
about a fifth of its engaged time among the pros and a quarter to a third among us; the share grows with the ball
(ours 10% alone to 29% with six or more friends within 120; the pros' 13% to 25%). The front stops when it has a
target and the rest, sent to the same point, are held behind it.
**Status.** measured (2026-09-25), `run/queued.py` (engaged: four or more armed enemies within 700; queued: no enemy
within reach + 20, a friend within 300 in reach, nearer its enemy, and firing this second; geometric: without the
shot). Ours, shots / geometry: bank-1 25% / 37% of 9,418 engaged soldier-seconds, 2v1b-hard 15% / 24% of 4,494,
2v1b-hard_aggressive 10% / 23% of 1,293, family-1 14% / 27% of 4,797; the pros on Comet Catcher at OS 40+ (40
games, both sides) 20% geometric of 243,795. Reproduced in the duel harness: the bank-1 13:50 cut (45 against 15)
13% queued under the plain attack-move, 17% under H-MICRO-FORM's ranks of six (the rear rank's slot is out of
reach behind a standing front); family-1 11:05 (37 against 13) 12% and 8%.
**Would be wrong if.** A body whose rear filed to the ends of its front (H-MICRO-FORM-FLANK) still spent a fifth
of its engaged time queued in the harness, or the pros' share on another map fell well under 20%.
**Amended the same day (the blocked refinement).** Queued with the firing friend within 40 of the unit's line to
its nearest enemy (physically in the way) is *blocked*: the pros 8% geometric; ours 22% / 16% / 11% / 17%
geometric (bank-1, 2v1b-hard, 2v1b-hard_aggressive, family-1), 14% / 10% / 4% / 9% with the shot. Nearly all of
our blocked seconds are units *moving* (bank-1: 1,245 of 1,335; family-1 403 of 409), most in the first ten seconds
of their engagement: the walk-up of the rear behind a front that has stopped, not a standing jam (90 still seconds of
9,418 engaged in bank-1). In the harness the blocked share is 6-10% in every arm, the ball's included (the bodies
reach the enemy within seconds), and the live medium game form-7 6%. The cost of the walk-up is friendly fire, the
rear firing into the front: 37-40% of the damage done in the bank-1 13:50 cut under the plain order, 18% in
family-1 11:05; H-MICRO-FORM-FLANK takes those to 4% and 3% and the fight from +0.69 to +0.84 and +0.66 to +0.71.
**Used by.** H-MICRO-FORM-FLANK; the harness's `engaged_s` / `queued_s` / `blocked_s` columns.

### K-form-the-rear-at-the-ends-not-in-the-back
**Claim.** When a body's front stops with targets in reach, the units behind it should file to the ends of the
front's line rather than close on the same point: the walk into the front's back costs the fight its friendly fire
(the rear's shells land on the front) and its exchange, and the ends of the line are where the rear can reach.
**Status.** measured (2026-09-25) in the duel harness, 8 repetitions an arm on the same sites and seeds, with a null
arm: bank-1 13:50 (45 against 15) plain +0.693 +- 0.016, the formation without the rule +0.804, with it
+0.840 +- 0.006, killed per lost 0.67 / 1.05 / 1.30, friendly fire 40% / 17% / 4%; family-1 11:05 (37 against 13)
+0.658 / +0.578 / +0.713, friendly fire 18% / 22% / 3%; the user's mixed force (8 Blitz, 7 Stout against 14 Pawn,
6 Stout, 2 Janus) -0.182 +- 0.056 without, -0.072 +- 0.066 with; 12 Stouts against 20 Grunts and 13 Thuds -0.001
without, +0.132 +- 0.042 with; the Blitz pack and the Stout mirror unchanged (+0.287 against +0.154 / +0.32 with the
formation alone; +0.023 against +0.034). Not measured in a live game with fights (form-7: exchange 1.34, one game).
**Would be wrong if.** An arena A/B of H-MICRO-FORM-FLANK over 24 games traded no better, or a live game with the
rule showed the ends of the line standing out of reach while the front lost (the wide line's known cost).
**Amended 2026-09-25 (lane-ab).** The whole final lane (the flank in it) against the old lane and no lane, 24 games
an arm: metal lost per metal killed 0.79 [0.65-0.95] against 0.84 [0.73-0.93] and 0.94 [0.83-1.05]; friendly fire
3.6% against 4.6% and 4.6%; blocked 6% against 7% and 6%. The rule is not separated from the rest of the lane in
the arena (the arms are whole binaries); the trade moved the way the harness said, within intervals that overlap.
**Used by.** H-MICRO-FORM-FLANK.
