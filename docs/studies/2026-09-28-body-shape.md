# The body's shape, judged in the duel harness (stage 1 of the micro answer)

The design is `docs/design/2026-09-28-body-shape.md` (the user's ruling: "various pitches and concaves of a line
formation"). This page is its step 4: every shape judged before a game, on the E3 and F3 cuts of player-14 and
player-15 and on the tier-2 arms of the body-size study (main tree, `docs/studies/2026-09-28-body-sizes.md`). Every
margin here comes from a `duels.csv` this worktree wrote (`run/matches/<stamp>-duel-<label>/`), read with
`run/shape_table.py` (the judge's columns) and `run/scenario_table.py --position`. Worktree `bar_bots-tas`, branch
`tas-micro`, speed 50, two engines, `--base-port 9500`.

## What was built

- **The definitions' area** (step 1, ea06b96): `UnitDefInfo::blast_radius`, the largest area of effect over a type's
  ordinary weapons, as the engine gives it. The engine's figure is a **radius**, half the unit file's `areaofeffect`
  (`WeaponDef.cpp`, `scaleValue(0.5f)`): the duel prints armfboy 150, armbull 65, armstump 24, armflash 4 at every
  start. The design's "the Fatboy's is 300" is the file's diameter.
- **The shape** (`crates/micro/src/form.rs`, steps 2 and 3, 149dbd7 and after). At contact (an armed enemy within 700
  of the body's centre) a body takes a target: the host's Attack target; else the one it had while it is still there;
  else the nearest soldier of theirs that no other building of theirs covers (its reach + 40); else the nearest
  soldier; else the nearest building. The target's party is the foes of its kind chained within 150, fitted with a
  segment along its major axis. The slots lie on the near side of the points within `R - 20 - spread` of that segment
  (`R` the body's shortest reach, `spread` the party's distance off its axis): an arc centred on a point target, a
  rank along a line whose ends curl round its ends, so every slot has the party's nearest unit in reach.
  - **Spacing** (H-MICRO-FORM-SPACING): 64 (two hulls), or twice the largest blast radius of anything of theirs in
    sight or remembered within 1,500 of the body's centre, at most 160: 130 against Bulls, 160 against a Fatboy.
  - **The arc count** (H-MICRO-FORM-ARC): the near half holds `(segment + pi r) / s + 1` slots (17 Stouts at 64 on a
    point, 8 Blitzes at 65); a count past it continues round the target's sides (the second arc), past a full ring the
    spacing closes to fit, down to 64, and past that a second ring stands 96 outside. This replaces the cut of a body
    at 12 (deleted, `BODY_MAX`); the march is still ranks of six at the spacing.
  - **Pitch**: the arc is turned about the target by the first of 0, ±15, ±30, ±45, ±60 degrees that leaves the
    fewest slots inside another building's reach (+40).
  - **Orders**: a unit with an enemy in its reach stands (no order); one with nothing in reach walks to its slot and
    keeps that slot until it gets there or the target changes; at its slot with nothing in reach it waits, and is
    sent on when the target has walked the slot 96 away. The walk is a **Move against area** (whenever the spacing is
    widened) and a **Fight** otherwise. Deleted: the rank at the standing front's depth (H-MICRO-FORM-FLANK's gate)
    and the `Close` step forward along the heading (the "closes on the nearest" clause).
- **H-MICRO-STEP-OUT** (a unit inside a turret's reach with nothing of its own in reach steps out): built, judged
  (below), retired, its code deleted after 900d8a3.
- `WITHIN_REASON_FORM_SPACING=factor,max` sets the spacing rule's numbers for a duel batch (the search below).

## The judge

"Old lane" is the lane at 899d64a (the formation as a transform with FORM-FLANK, the cut at 12 and `Close`), run
before any change here with that commit's binary; "off" is the director's attack-move. Scenario arms are `--seeds
1,2,3 --reps 8 --time-limit 120 --economy fed` (24 duels); tier-2 arms and the mirror are 8 duels on Mithril Mountain
v2.0.1 at spawn spacing 56 (the bench's settings). Columns are our side's: in reach while engaged (in-reach seconds
over engaged seconds), muzzled (of in-reach seconds), friend on the line (of units in reach, at the first damage and
ten seconds later). "The final lane" is the committed code after the step-out's retirement (`fin-*`).

### E3 (player-14, 9:17): 14 Blitzes against three Centurions and two light turrets (900 cut), plus five Pawns and two Rocketeers (r1250)

| Cut / their side | Arm | Batch | Fell, ahead | Margin, mean ± sd | Worst | In reach | Muzzled | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| 900 / track | attack-move (phase 1) | `1790555226-duel-tas-e3-v0-director` | 21/24 | +0.120 ± 0.089 | -0.065 | 0.40 | 0.00 | 0.55 |
| | old lane | `1790559323-duel-bs-e3-old-track` | 15/24 | +0.107 ± 0.140 | -0.159 | 0.45 | 0.00 | 0.44 |
| | **final lane** | `1790563386-duel-fin-e3-track` | **23/24** | **+0.214 ± 0.085** | -0.077 | 0.51 | 0.00 | 0.47 |
| | the TAS (v5) | `1790555558-duel-tas-e3-v5` | 24/24 | +0.318 ± 0.055 | +0.216 | 0.26 | 0.01 | 0.80 |
| 900 / director | attack-move | `1790559573-duel-bs-e3-off-dir` | 24/24 | +0.283 ± 0.068 | +0.168 | 0.45 | 0.01 | 0.52 |
| | old lane | `1790559449-duel-bs-e3-old-dir` | 24/24 | +0.270 ± 0.078 | +0.138 | 0.39 | 0.02 | 0.40 |
| | **final lane** | `1790563510-duel-fin-e3-dir` | 24/24 | **+0.292 ± 0.064** | +0.143 | 0.40 | 0.00 | 0.52 |
| | the TAS (v5) | `1790557506-duel-tas-e3-v5-vsdirector` | 23/24 | +0.294 ± 0.124 | -0.021 | 0.28 | 0.02 | 0.69 |
| r1250 / track | attack-move (phase 1) | `1790555800-duel-tas-e3w-v0-director` | 1/24 | -0.312 ± 0.100 | -0.442 | 0.50 | 0.02 | 0.43 |
| | old lane | `1790559388-duel-bs-e3w-old-track` | 0/24 | -0.416 ± 0.106 | -0.552 | 0.60 | 0.00 | 0.11 |
| | **final lane** | `1790563445-duel-fin-e3w-track` | 0/24 | **-0.233 ± 0.051** | -0.391 | 0.60 | 0.01 | 0.35 |
| | the TAS (w-v8) | `1790556753-duel-tas-e3w-w-v8` | 21/24 | +0.126 ± 0.077 | -0.040 | 0.36 | 0.04 | 0.82 |
| r1250 / director | attack-move | `1790559632-duel-bs-e3w-off-dir` | 2/24 | -0.285 ± 0.161 | -0.482 | 0.63 | 0.01 | 0.61 |
| | old lane | `1790559512-duel-bs-e3w-old-dir` | 1/24 | -0.296 ± 0.110 | -0.452 | 0.56 | 0.01 | 0.50 |
| | old lane, the design's figure (8 duels, one seed, 60 s) | `1790556055-duel-val2-e3w-lane` | 0/8 | -0.320 ± 0.112 | -0.456 | 0.54 | 0.01 | 0.36 |
| | **final lane** | `1790563570-duel-fin-e3w-dir` | 0/24 | **-0.189 ± 0.080** | -0.375 | 0.60 | 0.04 | 0.46 |
| | the TAS (w-v8) | `1790557575-duel-tas-e3w-w-v8-vsdirector` | 0/24 | -0.288 ± 0.116 | -0.481 | 0.54 | 0.01 | 0.78 |

("Fell, ahead": the Centurions and turrets all dead with our value above theirs; `run/scenario_table.py --position`.)

### F3 (player-15, 9:50, r1500): 10 Blitzes, 4 Stouts and 3 Rovers against 8 Pawns, 5 Rocketeers and a light turret

| Their side | Arm | Batch | W-L-D | Margin, mean ± sd | Worst | In reach | Muzzled | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| track | attack-move | `1790559695-duel-bs-f3-off-track` | 0-0-24 | +0.036 ± 0.031 | -0.040 | 0.11 | 0.05 | 0.44 |
| | old lane | `1790559779-duel-bs-f3-old-track` | 2-0-22 | +0.125 ± 0.186 | -0.177 | 0.22 | 0.06 | 0.07 |
| | **final lane** | `1790563633-duel-fin-f3-track` | 4-0-20 | **+0.229 ± 0.144** | -0.007 | 0.30 | 0.04 | 0.10 |
| director | attack-move | `1790559878-duel-bs-f3-off-dir` | 24-0-0 | +0.476 ± 0.058 | +0.380 | 0.35 | 0.04 | 0.26 |
| | old lane | `1790559948-duel-bs-f3-old-dir` | 24-0-0 | +0.466 ± 0.052 | +0.377 | 0.47 | 0.02 | 0.01 |
| | **final lane** | `1790563725-duel-fin-f3-dir` | 24-0-0 | **+0.457 ± 0.045** | +0.351 | 0.52 | 0.02 | 0.07 |

(On the track the enemy walks away and most duels end at the time limit: W-L-D counts wipes.)

### Tier 2: Stouts against Fatboys and Bulls at equal metal (the body-size bench's arms)

| Pairing | Arm | Batch | W-L | Margin, mean ± sd | In reach | Muzzled | Friend on the line | Nearest friend |
|---|---|---|---|---|---|---|---|---|
| 25 Stouts v 4 Fatboys (5,400) | attack-move | `1790560019-duel-bs-fboy5400-off` | 0-8 | -0.515 ± 0.082 | 0.65 | 0.00 | 0.23 | 40 |
| | old lane | `1790560046-duel-bs-fboy5400-old` | 0-8 | -0.335 ± 0.104 | 0.68 | 0.00 | 0.39 | 41 |
| | final lane, spacing rule off (64, Fight) | `1790562325-duel-nw-fboy5400-nosp` | 0-8 | -0.429 ± 0.077 | 0.67 | 0.00 | 0.40 | 41 |
| | final lane, Move at 65 | `1790563237-duel-nw2-fboy5400-move64` | 8-0 | +0.424 ± 0.079 | 0.75 | 0.20 | 0.23 | 46 |
| | final lane, Move at 110 | `1790562453-duel-nw-fboy5400-sp110` | 8-0 | +0.427 ± 0.080 | 0.73 | 0.30 | 0.29 | 54 |
| | **final lane** (Move at 160) | `1790563796-duel-fin-fboy5400` | 8-0 | **+0.462 ± 0.113** | **0.72** | 0.31 | 0.19 | 54 |
| | final lane, Move at 240 | `1790562426-duel-nw-fboy5400-sp240` | 8-0 | +0.437 ± 0.099 | 0.73 | 0.28 | 0.17 | 55 |
| 6 Stouts v 1 Fatboy (1,800) | attack-move | `1790560124-duel-bs-fboy1800-off` | 8-0 | +0.142 ± 0.112 | 0.82 | 0.00 | 0.00 | 42 |
| | old lane | `1790560149-duel-bs-fboy1800-old` | 8-0 | +0.439 ± 0.061 | 0.81 | 0.00 | 0.00 | 65 |
| | final lane, spacing rule off | `1790562376-duel-nw-fboy1800-nosp` | 8-0 | +0.417 ± 0.054 | 0.81 | 0.00 | 0.00 | 65 |
| | **final lane** | `1790563851-duel-fin-fboy1800` | 8-0 | **+0.648 ± 0.031** | 0.80 | 0.18 | 0.00 | 140 |
| 21 Stouts v 5 Bulls (5,400) | attack-move | `1790560072-duel-bs-bull5400-off` | 0-8 | -0.329 ± 0.046 | 0.78 | 0.00 | 0.20 | 40 |
| | old lane | `1790560097-duel-bs-bull5400-old` | 0-8 | -0.385 ± 0.061 | 0.56 | 0.00 | 0.07 | 43 |
| | final lane, spacing rule off (64, Fight) | `1790562351-duel-nw-bull5400-nosp` | 0-8 | -0.352 ± 0.051 | 0.63 | 0.00 | 0.09 | 42 |
| | final lane, Move at 65 | `1790563263-duel-nw2-bull5400-move64` | 6-2 | +0.154 ± 0.232 | 0.70 | 0.17 | 0.04 | 48 |
| | final lane, Move at 98 | `1790562506-duel-nw-bull5400-sp98` | 8-0 | +0.323 ± 0.056 | 0.69 | 0.20 | 0.13 | 47 |
| | **final lane** (Move at 130) | `1790563824-duel-fin-bull5400` | 8-0 | **+0.366 ± 0.064** | 0.70 | 0.23 | 0.10 | 50 |
| | final lane, Move at 160 | `1790562480-duel-nw-bull5400-sp160` | 8-0 | +0.369 ± 0.133 | 0.71 | 0.25 | 0.10 | 48 |
| 8 Stouts v 2 Bulls (1,800) | attack-move | `1790560174-duel-bs-bull1800-off` | 0-8 | -0.186 ± 0.090 | 0.82 | 0.00 | 0.00 | 40 |
| | old lane | `1790560199-duel-bs-bull1800-old` | 0-8 | -0.221 ± 0.033 | 0.81 | 0.00 | 0.00 | 40 |
| | final lane, spacing rule off | `1790562401-duel-nw-bull1800-nosp` | 0-8 | -0.196 ± 0.056 | 0.81 | 0.00 | 0.00 | 41 |
| | **final lane** | `1790563876-duel-fin-bull1800` | 7-1 | **+0.189 ± 0.241** | 0.80 | 0.19 | 0.00 | 54 |

### The mirror: 24 Stouts a side (the shape against the same reach)

| Their shape | Arm | Batch | W-L | Margin, mean ± sd | In reach | Friend on the line |
|---|---|---|---|---|---|---|
| ball (ranks of eight, attack-move) | attack-move | `1790560223-duel-bs-mirror24-off` | 2-6 | -0.033 ± 0.119 | 0.91 | 0.56 |
| | old lane | `1790560250-duel-bs-mirror24-old` | 6-2 | +0.083 ± 0.131 | 0.89 | 0.36 |
| | **final lane** | `1790563902-duel-fin-mirror24` | 8-0 | **+0.146 ± 0.061** | 0.88 | 0.44 |
| line (`--formation ranks8/line`) | attack-move | `1790562589-duel-off-mirror24-line` | 8-0 | +0.141 ± 0.078 | 0.84 | 0.57 |
| | **final lane** | `1790563931-duel-fin-mirror24-line` | 8-0 | **+0.172 ± 0.044** | 0.84 | 0.41 |

### H-MICRO-STEP-OUT on and off (the rule as built, then as refined; 24 duels an arm)

| Arm | With (149dbd7) | With, idle units only (900d8a3) | Without |
|---|---|---|---|
| E3 900 / track | +0.056 (`nw-e3-track`) | +0.146 (`nw2-e3-track`) | +0.217 (`nw-e3-track-nostep`) |
| E3 r1250 / track | -0.213 (`nw-e3w-track`) | -0.273 (`nw2-e3w-track`) | -0.259 (`nw-e3w-track-nostep`) |
| E3 900 / director | +0.332 (`nw-e3-dir`) | +0.295 (`nw2-e3-dir`) | +0.294 (`nw-e3-dir-nostep`) |
| E3 r1250 / director | -0.223 (`nw-e3w-dir`) | -0.207 (`nw2-e3w-dir`) | -0.166 (`nw-e3w-dir-nostep`) |
| F3 / track | +0.080 (`nw-f3-track`) | +0.161 (`nw2-f3-track`) | +0.230 (`nw2-f3-track-nostep`) |
| F3 / director | +0.436 (`nw-f3-dir`) | +0.445 (`nw2-f3-dir`) | +0.446 (`nw2-f3-dir-nostep`) |

The final lane (the rule deleted) reproduces the "without" column: +0.214, -0.233, +0.292, -0.189, +0.229, +0.457.

## What it says

- **Against area fire the shape wins, and the walk is most of it.** 25 Stouts against 4 Fatboys go from -0.515
  (attack-move) and -0.335 (old lane) to **+0.462, 8 of 8**; 21 against 5 Bulls from -0.329 / -0.385 to **+0.366, 8 of
  8**; the 1,800 arms from +0.142 to +0.648 and from -0.186 to +0.189. The same concave walked with a Fight at 64
  (the spacing rule off) loses as the old lane did (-0.429, -0.352). Walked with a Move, the Fatboy arm is the same at
  every spacing from 65 to 240 (+0.424 to +0.462): the Fatboys' damage per counted shot falls from 2,074 hit points
  (`nw-fboy5400-nosp`) to 1,379 (`nw2-fboy5400-move64`) and 1,246 (`fin-fboy5400`), while the Stouts keep walking to
  slots that move with the Fatboys (walking tanks are missed by a slow shell; not separated from the spread). Against the Bulls both count: the Move from 64 is +0.51 (-0.352 to +0.154) and the
  spacing from 65 to 130 another +0.21 (+0.154 to +0.366, 98 between). So the rule's cap stands at 160 and its
  factor at two radii; the search found no better cap for the Fatboy (110, 160, 240 within noise).
- **In reach while engaged against the Fatboys moved from 0.63 (the bench) / 0.65 (attack-move here) to 0.72-0.75.**
  The muzzled share rose from 0.00 to 0.2-0.3 on the tier-2 arms: a unit walking under a Move toward its slot is in
  reach without firing (the fire instrument counts it muzzled "clear"); the trade is still +0.8 of margin. Against
  direct fire (E3, F3) the walk is a Fight and nothing is muzzled (0.00-0.04).
- **The E3 r1250 margin under the lane moved from -0.320 (the design's figure; -0.296 on this page's settings) to
  -0.189 against the reacting director, and from -0.416 to -0.233 against the recorded track.** Still 0 of 24 fell:
  the position as it was is not taken by fourteen Blitzes under any control tried (the TAS's best script, w-v8, 21 of
  24 on the track and 0 of 24 against the director). On the 900 cut the final lane takes the position 23 of 24 on the
  track (+0.214, old lane +0.107 and 15 of 24, attack-move +0.120), 0.10 short of the TAS's v5; against the director it
  equals attack-move and the TAS (+0.292 / +0.283 / +0.294). The target choice (the nearest soldier no turret covers,
  kept while it lives) is the TAS's order: the Centurions first, outside turret cover, then the turrets.
- **F3**: on the track +0.229 against the old lane's +0.125 and attack-move's +0.036; against the director the three
  are the same (+0.457 / +0.466 / +0.476).
- **The mirror costs nothing**: against a ball +0.146 (attack-move -0.033, old lane +0.083), against a line +0.172
  (attack-move +0.141). The rank along a line and the concave on a ball fire as much as the ball (in reach 0.84-0.91).
- **The friend-on-the-line share did not move one way.** Final against attack-move: E3 900 track 0.47 / 0.55, r1250
  director 0.46 / 0.61, the mirror 0.44 / 0.56 (down); Fatboys 0.19 / 0.23, Bulls 0.10 / 0.20 (down); E3 900 director
  0.52 / 0.52, r1250 track 0.35 / 0.43. Against the old lane it is higher in most arms (the old lane's standing-front
  rank kept files apart at the cost of reach: E3 r1250 track 0.11 friend on the line and 0.60 in reach, -0.416).
- **Never idle under a turret did not pay** as a per-unit step. Stepping out every unit with nothing in reach inside a
  turret's reach pulled the Blitzes walking to their slots back out of the fight (the E3 900 duel on the track from
  22 s to 35 s, +0.217 to +0.056); stepping out only idle units still cost 0.07 on two arms and gained nothing on four.
  What the TAS showed ("after a kill, leave the turret's reach") is covered in the shape by the target choice (a
  soldier under cover is taken after one in the open) and the pitch; the rest (gather outside, go in together) is a
  body-level decision for the stage-2 planner, not a unit's reflex. Retired, code deleted.
- **Two faults found while building it** (probe batches, same settings): slots pushed straight out of a turret's reach
  left their units out of reach of the target and inside its own (E3 900 track -0.140, `pr-fight-e3-track`); slots
  re-dealt every tick re-ordered the Blitzes twice a second (E3 r1250 track -0.338 under Move, `pr-move-e3w-track`;
  now a unit keeps its slot until it gets there). Both fixed before the judge.

## Not measured, and what is next

- Live play: no arena game or player game was run (the worktree's constraints). The main session's live check is
  the next step, read with `run/fire.py` and `run/micro_ledger.py` for the muzzled share under the Move.
- The E3 r1250 position is still lost 24 of 24 under the lane: the TAS says what is missing is at the body level
  (gather outside the turrets, go in together, the right mix), which is the stage-2 planner's.
- `run/tas_diff.py` (the design's step 5) is not built.
- Terrain: every tier-2 arm is on flat ground; the arcs are snapped to reachable ground by the host but the duel's
  view does not snap (`DuelView::snap` is the identity).
