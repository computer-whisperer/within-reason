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
**Used by.** (none yet; motivates per-unit move targets spaced about 64 elmos in the micro engine's group orders)

### K-form-pros-stand-two-hulls-apart
**Claim.** The pros' soldiers stand about two hulls from their nearest friend at contact; ours stand at about one.
**Status.** measured (2026-09-24), nearest-friend distance in the body at t0 and t0+10: Stout (hull 32) pros median 70
elmos, 37% within 48 (446 unit-snapshots), us 40, 79% (799); Blitz pros 67, 38% (4,425), us 25, 78% (435). Incisor 48,
Grunt 40, Brute 54, Pounder 115 for the pros.
**Would be wrong if.** A rerun with our new records showed our spacing at 60+ without a spacing change in the bot (the
three games being unrepresentative), or the pros' spacing on another map at one hull.
**Used by.** (none yet)

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
**Used by.** (none yet; with K-engine-a-shot-is-refused-across-a-friend)

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
**Used by.** (none yet; motivates forming Blitz packs and Stout lines as separate groups rather than one ball)

### K-form-a-line-at-contact-trades-better
**Claim.** Among the pros a force that meets the enemy as a line (across at least twice along) trades a few points better
than a blob and more than a stream; front width relative to the enemy's does not matter by itself.
**Status.** measured (2026-09-24), weak. Metal-weighted exchange share: line 52% (192 sides), blob 46% (402), stream
43% (110); line - blob +5.2, interval +0.8 .. +10.1 over resampled games; line - stream +8.1, -0.1 .. +18.4. The
eventual winners' lines traded 60% (102), the losers' 40% (90). Wider core than the enemy's 51% against 49% (165
pairs). Observational: a stream may be a chase or a retreat rather than a choice of shape.
**Would be wrong if.** Our arena games with the micro engine forming lines at contact traded no better than its blobs
over 24+ games.
**Used by.** (none yet)

### K-form-pros-raiders-fight-at-reach
**Claim.** The pros' short-range units hold the edge of their reach in a fight; ours close in.
**Status.** measured (2026-09-24). Distance to the nearest enemy combatant over own reach, unit-seconds with one within
reach + 100: short-range pros median 0.94 (86k), us 0.86 (2.9k). Line units 0.87 against 0.88.
**Would be wrong if.** The difference vanished when counted only for Blitzes against the same enemy types.
**Used by.** (none yet)
