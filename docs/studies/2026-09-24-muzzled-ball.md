# The muzzled ball: what the engine says about a Stout ball's own hulls

Phases 1 and 2 of `docs/design/2026-09-24-muzzle-project.md`: the duel harness measures muzzling, and recorded
engagements are replayed in the engine from a scenario file. The definitions are `run/fire.py`'s throughout: a soldier
is in reach when an enemy stands within its reach plus 20; muzzled when it has been in reach without a shot for 3 s or
two reloads, whichever is longer; the cause is a friend within 24 of the line to the nearest enemy in reach and nearer
than it, else that enemy within 40 of the reach's edge, else a clear line.

## Phase 1: Stouts against Stouts on flat ground (batch `muzzle-stout-shapes`, 2026-09-24)

24 Stouts against 24 (equal metal, 5,400 a side), `--formation ranks8,line,line/line --reps 8`, both sides the duel
director's plain attack-move, spacing 56, Mithril Mountain v2.0.1 (two flat sites of 1564 x 1528; no other installed
map offered a site that holds a line of 24, see `docs/harness/duels.md`), speed 50, two engines. Every repetition
rotates west/east and the team slot. `run/matches/1790287202-duel-muzzle-stout-shapes`.

| First army / second | Duels | Margin of the first (mean +- s.e.) | W-L | Muzzled share, first / second | Shots per in-reach second (0.83 possible) | Friendly fire share of damage done | Spread at contact | Mean seconds |
|---|---|---|---|---|---|---|---|---|
| ranks of 8 / ranks of 8 | 8 | -0.049 +- 0.038 | 2-6 | 0.1% / 0.0% | 0.83 / 0.83 | 0.12% / 0.13% | 102 / 103 | 84 |
| line / ranks of 8 | 8 | **-0.132 +- 0.034** | 1-7 | 0.3% / 0.4% | 0.81 / 0.79 | 0.00% / 0.08% | 384 / 101 | 80 |
| line / line | 8 | +0.013 +- 0.010 | 5-3 | 0.0% / 0.0% | 0.84 / 0.84 | 0.00% / 0.00% | 389 / 389 | 33 |

The muzzled seconds are too few to read their causes (5, 12 and 25 seconds of 4,588-6,090 in-reach seconds an arm).

**What it says.**
- **A ball of Stouts on flat ground does not muzzle itself.** In reach, a Stout in a ball of 24 fires at its full rate
  (0.83 a second, one shot per 1.2 s reload) whatever the formation; the muzzled share is under half a percent in every
  arm. The engine's line-of-fire refusal (K-engine-a-shot-is-refused-across-a-friend) is real, but the cannon's arc
  clears the hulls of a ball closing on a mirror at attack-move.
- **The line loses to the ball**, 1 win in 8, -0.13 of margin. The line's ends are out of reach while the ball's whole
  front fires (in-reach seconds 4,796 for the line against 6,090 for the ball): local superiority, not muzzling, decides.
  Two lines trade evenly and fast (33 s against 80-84).
- **The live numbers are not this.** The same definitions over the live records (`run/fire.py`) give Stouts 0.44-0.45
  shots per in-reach second and 14-18% muzzled:

| Record | Stout in-reach s | Stout shots per in-reach s | Stouts muzzled | All soldiers muzzled, by friends within 120: alone / 1-2 / 3-5 / 6+ | Causes (all soldiers): clear / friend on line / edge | Friendly fire share (all types) |
|---|---|---|---|---|---|---|
| 2v1b-hard (`1790261454-2v1b-hard/00`) | 3,385 | 0.44 | 18% (594 s) | 26% / 22% / 13% / 11% | 56% / 26% / 19% | 5.6% |
| bank-1 (`1790257668-bank-1/00`) | 5,558 | 0.45 | 14% (751 s) | 14% / 12% / 18% / 10% | 46% / 42% / 12% | 13.2% |

  In the live games the muzzled share is highest for a soldier with nobody near it (26% in 2v1b-hard) and lowest in
  the biggest balls, and a clear line is the largest cause. Radar-only contacts do not explain it: split by whether a
  ground enemy was in sight inside the reach, the Stouts' seconds with one in sight are 17% and 14% muzzled at 0.45
  shots a second (a scratch split of `run/fire.py`, not kept). What differs between the duel and the game is still
  open: terrain (a cannon's arc against slopes and ridges), the control lane's and the hands' orders (a unit that keeps
  being re-ordered), mixed enemies and targets it will not or cannot shoot, and friends of other types in front (the
  Flashes: 13% friendly fire in bank-1 against 0.1% here). Phase 2 replays recorded engagements in the engine so that
  these can be told apart.

The claim is K-units-duel-a-stout-ball-is-not-muzzled-on-flat-ground (`docs/knowledge/units.md`); the formation knob is
H-DUEL-FORMATION (`docs/heuristics.md`, harness only).
