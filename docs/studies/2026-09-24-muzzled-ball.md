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

A rerun of the same batch after phase 2's rework of the director (`muzzle-stout-shapes-recheck`,
`run/matches/1790288926-duel-muzzle-stout-shapes-recheck`) gave the same picture: the line against the ball -0.155
(0-8), ranks against ranks -0.004, lines against lines +0.009; muzzled 0.0-0.5%; 0.79-0.84 shots per in-reach second.
Over both batches the line lost 15 of 16.

The claim is K-units-duel-a-stout-ball-is-not-muzzled-on-flat-ground (`docs/knowledge/units.md`); the formation knob is
H-DUEL-FORMATION (`docs/heuristics.md`, harness only).

## Phase 2: three recorded engagements fought again (2026-09-24)

Cut with `run/engagement.py` from the main checkout's records (read-only) and fought with `duel --scenario FILE --reps 8
--parallel 2` on Comet Catcher Remake 1.8 at the recorded place; the files are in `docs/data/scenarios-2026-09-24/`.
Two of the three named clocks have no enemy in sight (13:50: radar contacts only; 27:34: the Fatboys fire from out
of sight), so the enemies are cut from the opponent's ground truth (`--enemies truth`, the `truth-0.jsonl` both
matches carry); 15:10 is also cut with the enemies in sight only, to show what that choice does. Radius 1,100 (1,300
at 13:50, to take in the Janus group that was coming). Units: armed only, no commander. "Live" is how the same units
stood 30 and 60 s after the clock in the game itself (the record and the truth file), scored the same way; it is not
the same experiment (the live enemy kept its own orders, both sides were reinforced, units walked out of the circle).

| Scenario | Ours / theirs (hurt) | Health error, ours / theirs | Engine W-L (8) | Value left, ours / theirs: mean [range] | Margin sd | Seconds [range] | Muzzled, ours / theirs | Friendly fire share, ours / theirs | Live value left at 30 s / 60 s, ours / theirs |
|---|---|---|---|---|---|---|---|---|---|
| 2v1b-hard 15:10, truth (the D6 fight) | 33 (30 Stouts, 3 LLT; 23) / 12 (7) | 0.015 / 0.012 | 8-0 | 0.80 [0.79-0.82] / 0 | 0.010 | 24 [22-26] | 2.7% / 0.6% | 0.1% / 0.3% | 0.73 / 0.58, 0.71 / 0.59 |
| 2v1b-hard 15:10, sight | 29 (21) / 8 (8) | 0.018 / 0.018 | 8-0 | 0.86 [0.84-0.88] / 0 | 0.013 | 20 [18-21] | 3.3% / 3.0% | 0.0% / 0.2% | 0.72 / 0.56, 0.69 / 0.58 |
| 2v1b-hard 13:50, truth (the trade) | 35 (0) / 16 (8) | 0.000 / 0.010 | 8-0 | 0.56 [0.53-0.63] / 0 | 0.035 | 47 [44-52] | 2.2% / 1.8% | 1.7% / 0.3% | 1.00 / 1.00, 0.86 / 0.93 |
| bank-1 27:34, truth (Fatboy splash) | 25 (17) / 18 (3) | 0.058 / 0.003 | 0-8 | 0 / 0.62 [0.43-0.72] | 0.094 | 98 [82-132] | **23.0%** / 4.2% | 1.8% / **31.0%** | 0.98 / 0.97, 0.45 / 0.83 |

Batches `scen-2v1b-hard-1510-truth`, `scen-2v1b-hard-1510-sight`, `scen-2v1b-hard-1350-truth`,
`scen-bank-1-2734-truth` (`run/matches/1790288983-...` to `1790289044-...`). Every fight ended with one side wiped.

**What it says.**
- **The replay is repeatable.** Over 8 repetitions the margin's standard deviation is 0.01-0.04 for the three
  2v1b-hard cuts and 0.09 for bank-1; nobody's result changed sign. An engagement cut from a record is a stable
  engine measurement, so an A/B of a footwork rule on it (phase 4) needs few repetitions.
- **The 15:10 fight is won as it was won**: our side keeps 0.80 of its value (live: 0.73 after 30 s) and the enemy
  in the circle is wiped (live: 0.58 left, because the live enemy fell back and was reinforced). The cut with the enemies
  in sight only (its contact point moves, and it holds 29 of ours against 8 instead of 33 against 12) leaves us 0.86.
- **13:50 is not an engagement at the clock** (nothing had happened 30 s later, live): the replay makes both sides
  attack-move at each other, which the live enemy did not, so its 0.56 is what a head-on fight would have cost, not
  what the trade cost.
- **bank-1 27:34 reproduces the loss and the muzzling.** Our 25 lose all 8 repetitions, leaving the enemy 0.62 of its
  value (live, 60 s later: ours 0.45, theirs 0.83); our side sits muzzled 23% of its in-reach seconds (531 of 2,311:
  a clear line 324, a friend on the line 153, the edge 54), against 0.6-3.3% for either side in every other scenario and under 0.5% in the
  flat Stout duels. The enemy here out-ranges us (Fatboys 700, Fidos 650) and splashes its own side (31% of the damage
  it does is to itself). So the muzzled share is a property of this matchup and ground, not of a Stout ball as such;
  which of range, height, targets and splash makes it is the next question, and this scenario is where to ask it.

**The simulator cannot take these files yet.** `combatsim` places units by group (a front, a facing, a spacing), which
could express one unit per group, and covers every type in the four files, but it has no per-unit health (every unit
starts whole: `sim.rs`, `hp: unit.health`), and 8 to 30 units of each file start hurt; no first orders beyond a
group's intent (fight, raid, flee, guard: no move to a point, no attack on a named unit); no reader for the scenario
file; and, as the design says, no hulls, no line-of-fire test, no refusal and no splash on friends, which is what the
bank-1 replay measures. Those are phase 3's input; no simulator number stands beside the engine's here.
