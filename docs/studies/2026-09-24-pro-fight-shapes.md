# How the pros shape their forces in fights, and how we do

Written 2026-09-24 by Claude (Opus 5.5) from the match records only (no game run). The question (the user, from a
replay the experienced players flagged): our Blitz packs could not fire past each other and the Stouts behind them
shot them (K-engine-a-shot-is-refused-across-a-friend). What shapes do high-level players form at contact, how do they
mix the types, and which shapes trade well, so the micro engine's formation control can copy the useful ones.
Claims drawn from it: `docs/knowledge/formations.md`. Rerun: `run/replays/shapes.py --orders --boot` (20 s).

## Data

- **Pros:** the 40 carded Comet Catcher Remake 1.8 duels with both players at OS 40 and above (the manifest, as
  `run/replays/recheck.py` selects them): 693 fights, 1,386 fight sides.
- **Us:** bank-1, 2v1b-hard, 2v1b-hard_aggressive (all Comet Catcher, all lost; the Opus player with Jev, Armada, a
  Stout and Blitz army): 60 fights. The opponent in those games (BARb hard and hard_aggressive) is a column of its own.
- Every pool counts fight sides. A pro fight gives one side to the winners' pool and one to the losers'; the pros'
  overall exchange is 50% by construction, so compare the shape rows against each other, not against 50%.

## Definitions (and where the brief's were not usable)

- **Damage.** The replay records' `dmg` is always empty (the dump widget does not hook UnitDamaged), so the brief's
  fight definition from `dmg` was not possible. Damage is the drop in health between consecutive one-second samples,
  for both sides and for our games too so the instrument is the same: net of repair within the second, and for the
  opponent in our games only while we saw it. Our records' `enemy_destroyed` is complete (checked against the ground
  truth files: 264 of 266 BARb soldiers that left the truth list have the event).
- **Fight.** Hits on mobile combatants (soldiers: class `army`/`hover` with a weapon; and commanders) linked when within
  8 s and 600 elmos of each other, kept when each side took 200 or more. Contact t0 = the first hit; the contact place C
  = the centroid of the hits in the first 10 s.
- **Body** = a side's soldiers within 700 of C at a second. **Core** = the body's largest group (soldiers chained at
  200 elmos or less). Shapes are measured on the core; the first version measured the whole body and called a tight
  five-Incisor ball with one scout 500 elmos off a "line", so the core replaced it.
- **Across / along**: the core's extent (max minus min) across and along the line from its centroid to the centroid
  of the enemy's combatants near C. Line: across at least twice along; stream: along at least twice across; blob
  otherwise (3+ soldiers).
- **Friend on the line** (the muzzling geometry of `run/fire.py`): of the body's units with an enemy unit within
  reach + 20, those with a friendly soldier within 24 elmos of the segment to the nearest one, nearer than it.
- **Hull** = the footprint (4 x 8 = 32 elmos for the Stout, Blitz and the other tier-1 tanks and bots; 48 for the
  missile trucks and artillery). The brief's "about 40" is the Stout's hull with some clearance.
- **Trade**: soldier and commander metal destroyed in the fight (a unit hit in it, or dying within 700 of C from
  t0 - 5 to the last hit + 5); exchange share = theirs / (ours + theirs), metal-weighted over the sides.

## Pros against us

| At contact (t0; t0+10 in brackets) | pros | pro winners | pro losers | us | BARb vs us |
|---|---|---|---|---|---|
| soldiers in the body, median (bodies of 3+) | 6 (961) | 6 (482) | 6 (479) | 13 (34) | 4 (32) |
| core across x along, median elmos | 190 x 165 (731) | 185 x 163 | 203 x 167 | 205 x 184 (33) | 122 x 114 (27) |
| line / blob / stream (cores of 3+) | 27 / 57 / 16% of 731 | 29 / 56 / 16% | 25 / 59 / 16% | 15 / 70 / 15% of 33 | 19 / 67 / 15% of 27 |
| friend on the line | **22%** of 2,429 (31% of 2,684) | 22% (30%) | 22% (31%) | **43%** of 140 (54% of 178) | 21% (25%) |
| Stouts: friend on the line (t0 and t0+10) | 24% of 228 | 23% of 178 | 26% of 50 | **53% of 247** | 15% of 62 |
| Blitzes: friend on the line | 23% of 1,334 | 22% of 750 | 24% of 584 | 35% of 60 | 18% of 34 |
| Stout nearest friend, median; share < 1.5 hulls | 70; 37% (446) | 68; 37% | 81; 36% | **40; 79%** (799) | 78; 26% |
| Blitz nearest friend, median; share < 1.5 hulls | 67; 38% (4,425) | 69; 38% | 64; 39% | **25; 78%** (435) | 55; 39% |

Friend on the line grows with the body: the size of the ball, not only the spacing, sets it.

| Friend on the line by body size (t0 and t0+10) | pros | us | BARb vs us |
|---|---|---|---|
| 3-6 soldiers | 17% of 1,520 (1,035 bodies) | 39% of 31 (18) | 18% of 77 (36) |
| 7-12 | 30% of 1,594 (563) | 21% of 24 (15) | 31% of 106 (20) |
| 13+ | 41% of 1,463 (283) | **59% of 227 (36)** | 28% of 18 (5) |

## What the pros ordered before contact

The last group order (three or more of the fight's participants given move or fight in one frame) in the 30 s before
contact. The records show per-unit points; which widget made them (CustomFormations2's drag, by the evidence of the
straightness) is not in the record.

| | pros | pro winners | pro losers | us |
|---|---|---|---|---|
| sides with such an order | 1,070 of 1,386 | 534 of 693 | 536 of 693 | 44 of 60 |
| one point per unit (distinct targets for 80%+ of the units) | **753 (70%)** | 400 | 353 | 2 |
| one point for all | 223 (21%) | 93 | 130 | **34 (77%)** |
| targets' straightness (minor / major principal extent; 0 = a straight line), median | **0.06** (833) | 0.06 | 0.06 | 0.56 (4) |
| spacing between targets, median elmos | **64** (833) | 64 | 65 | 29 (4) |
| targets' extent across the travel, median | 279 | 267 | 291 | 929 (4) |
| straight lines at 60+ degrees to the travel / 30-60 / under 30 | 326 / 242 / 187 of 755 | 158 / 127 / 104 | 168 / 115 / 83 | - |

## Type layering and separate groups

| | pros | pro winners | pro losers | us |
|---|---|---|---|---|
| cores of 3+ holding both short-range (reach <= 250) and long-range (>= 300) soldiers | **10%** (73 of 731) | 13% | 7% | **61%** (20 of 33) |
| the same, when the side owned 2+ of each on the map | 33% (71 of 212) | 38% of 121 | 27% of 91 | 78% (18 of 23) |
| in mixed cores: short ahead of long (> +50) / beside / behind (< -50) | 38 / 29 / 33% of 73 | 38 / 31 / 31% of 48 | 40 / 24 / 36% of 25 | 25 / 50 / 25% of 20 |
| the side's short-range metal in the body at contact | 27% | 26% | 28% | 39% |
| the side's long-range metal in the body at contact | 17% | 18% | 16% | 37% |
| bodies of 3+ split in two or more groups | 196 of 961 (20%) | 97 of 482 | 99 of 479 | 10 of 34 |

Mean rank in the core (1 frontmost, 0 rearmost), pros: Incisor 0.53 (2,077), Blitz 0.52 (1,538), Grunt 0.53 (1,132),
Brute 0.48 (381), Stout 0.49 (132), Pounder 0.58 (46), Lasher 0.45 (35), Mace 0.60 (40). Us: Blitz 0.43 (170), Stout
0.47 (342). No type stands consistently in front or behind, for them or us.

## Arrival

| | pros | pro winners | pro losers | us | BARb vs us |
|---|---|---|---|---|---|
| participants, median | 8 (1,160) | 8 | 8 | 20 (47) | 10 (42) |
| participants within 700 of C by contact, median | 82% | 83% | 80% | 87% | 60% |
| arrival spread p90 - p10, median s | 13 | 11 | 15 | 12 | 24 |
| transit shape 20 s before, line / blob / stream | 180 / 658 / 232 of 1,070 | 88 / 318 / 133 | 92 / 340 / 99 | 3 / 38 / 6 of 47 | 2 / 14 / 3 |

## Trade against shape (pros; bootstrap over the 40 games)

| a - b (exchange share, metal-weighted) | difference | 95% interval | sides a / b | unweighted means |
|---|---|---|---|---|
| line - blob at contact | **+5.2** | +0.8 .. +10.1 | 192 / 402 | 52% / 48% |
| line - stream | +8.1 | -0.1 .. +18.4 | 192 / 110 | 52% / 40% |
| blob - stream | +2.9 | -5.0 .. +13.4 | 402 / 110 | 48% / 40% |
| friend on the line for a third or more - for less | +4.0 | -3.6 .. +11.5 | 141 / 141 | 48% / 51% |
| mixed core - pure core | +3.4 | -2.8 .. +8.0 | 70 / 634 | 52% / 47% |

Within the pools: a line at contact traded 60% for the eventual winners (102 sides) and 40% for the losers (90); blob
53% / 40%; stream 41% / 45%. A wider core than the enemy's traded 51% against 49% for the narrower (165 pairs); a
body in two groups 47% against 48% in one; arriving within 10 s 45% against 52% for a spread arrival (the spread sides
include the ones that kept feeding a fight they were winning, so that row is not a verdict on arriving together).
Ours: 35% over 58 sides (102k metal); by shape our cells are 5-23 sides and say nothing.

## By fight size (participants of both sides)

| pool | size | fight sides | line / blob / stream at t0 | friend on the line at t0 | exchange |
|---|---|---|---|---|---|
| pros | 2-6 | 188 | 4 / 9 / 9 of 22 | 5% of 222 | 50% |
| pros | 7-15 | 590 | 74 / 155 / 51 of 280 | 18% of 886 | 50% |
| pros | 16+ | 606 | 118 / 256 / 55 of 429 | 28% of 1,321 | 50% |
| pro winners | 16+ | 303 | 65 / 124 / 25 of 214 | 27% of 690 | 56% |
| pro losers | 16+ | 303 | 53 / 132 / 30 of 215 | 29% of 631 | 44% |
| us | 2-6 | 16 | 0 / 1 / 0 of 1 | 21% of 19 | 34% |
| us | 7-15 | 9 | 1 / 3 / 3 of 7 | 29% of 21 | 59% |
| us | 16+ | 35 | 4 / 19 / 2 of 25 | **50% of 100** | 34% |

## Conduct

| | pros | pro winners | pro losers | us | BARb vs us |
|---|---|---|---|---|---|
| damage dealt that landed on units that died in the fight, median | 74% (1,048) | 80% | 67% | 57% (48) | 57% (52) |
| the same in even fights (exchange 35-65%) | 82% (281) | 85% | 78% | 73% (16) | 65% (15) |
| the most-hit enemy's share of a second's damage (2+ hit), median | 63% | 62% | 63% | 63% | 61% |
| units that fell below 40%: survived / stepped back 150+ within 8 s | 24% / 16% of 6,782 | 27 / 17% | 22 / 16% | 24% / 17% of 423 | 59% / 15% of 455 |
| distance to the nearest enemy over own reach, short-range units, median | **0.94** (86k unit-s) | 0.95 | 0.94 | **0.86** (2.9k) | 0.96 |
| the same, line units (reach 300-450) | 0.87 (48k) | 0.86 | 0.89 | 0.88 (11k) | 0.74 |
| the same, long-range (450+) | 0.73 (6.9k) | 0.77 | 0.69 | 0.93 (90) | 0.58 |

## What the pros do that we do not

They order a group as a line of separate points. Seven in ten of their last group orders before a fight give each unit
its own point, on a nearly straight line (straightness 0.06), 64 elmos apart: two hull widths. We send three in four
of ours to one point. What arrives is not a line abreast: at contact the pros' cores are blobs more often than lines
(57% against 27%), and only 43% of their drawn lines (326 of 755) run across the travel; the rest are slanted or along it. What the drag buys is
spacing: their Stouts and Blitzes stand 67-70 elmos from the nearest friend, ours 25-40, and 78-79% of ours are within
one and a half hulls of a friend against 37-38% of theirs.

They fight in small bodies. Their median body at contact is 6 soldiers, ours 13; only a quarter of their raider metal
and a sixth of their line metal is at any one fight. The muzzling geometry follows size and spacing: a friend is on
the line of 22% of their units in reach at contact against 43% of ours, 24% of their Stouts against 53% of ours; in
bodies of 13 or more they reach 41% and we 59%.

They rarely put raiders and line units in one body: 10% of their cores are mixed (a third when they own both kinds)
against 61% of ours. When they do mix, the raiders are ahead, beside or behind in about equal thirds; there is no
layering rule in these games to copy.

Their short-range units fight at the edge of their reach (0.94 of it) where ours close to 0.86.

Shape and trade: a line at contact traded 5 points better than a blob (interval +1 to +10) and about 8 better than a
stream, mostly in the eventual winners' hands. Front width relative to the enemy, splitting, mixing and the muzzling
geometry itself did not separate the pros' good trades from their bad ones within the intervals here.

## What this could not measure

- Whether a muzzled unit actually held fire, and friendly fire: replay records carry no `shots`, `dealt` or `ff`. The
  friend-on-the-line share is geometry, as in `run/fire.py`'s cause column; terrain (a cannon's arc over a friend on a
  slope) is not modelled.
- Retreat of damaged units shows no difference (16-17% stepped back everywhere); BARb's 59% survival below 40% health is
  unexplained (its retreat behaviour or health read from `en` while seen) and was not chased.
- Focus fire: the per-second concentration is the same for everyone (61-63%); the pros' higher share of damage on units
  that died is partly the winner's effect (80% winners, 67% losers) and ours rests on 16-48 sides.
- The pros' armies here are raider-heavy (Incisors, Blitzes, Grunts); Stout snapshots are 446 against our 799, Pounders
  and Janus few. The shapes of large tier-1 line armies at 16+ are the thinnest part of the pro data.
- Our side is three lost games against BARb, not against these players; every "us" cell is small and a shape-by-trade
  cell for us has 2-23 sides.
- The order table cannot say which widget or key made a per-unit order, and our bot's micro re-orders units every tick,
  so "the last group order" is the heuristic layer's order for us.
