# The pick's framing, replayed offline on player-26 (2026-09-29)

Recorded picks of `run/matches/1790731150-player-26-joint/00` re-asked with the same state and the options changed
(scripts in the session's scratchpad: `drop_arms.py`, `head_to_head.py`, `deprivilege.py`, `twostage.py`; each takes
the match directory and calls Jev as `run/jev_ab.py` does). The question: why "nothing changes" (w1) won 142 of 315
picks while the factories idled with 1,500 metal banked.

## 1. Arms dropped from the 10:44 pick (48 worlds, w1 at 0.34 as recorded)

| option set | options | choice | w1 | the Mace's mass, summed | pairs of the rest in the recorded order |
|---|---|---|---|---|---|
| as recorded, twice | 48 | w1 | 0.35, 0.32 | 0.36, 0.38 | 566, 595 of 1128 (chance 564) |
| shuffled | 48 | w1 | 0.33 | 0.43 | 449 of 1128 |
| a random half, twice | 25 | w1 | 0.33, 0.28 | 0.46, 0.58 | 115, 141 of 300 |
| w1 and the 24 Mace worlds | 25 | w1 | 0.27 | 0.73 | 155 of 300 |
| without w1 | 47 | w3 | 0 | 0.63 | 474 of 1081 |

The order among the deviations is chance; w1's share holds at 0.27-0.41 whatever is dropped; a change's mass grows
with the number of worlds holding it.

## 2. Head to head, three asks each

| options | w1 | the Mace |
|---|---|---|
| w1 against the Mace's recorded line | 0.50-0.52 | 0.48-0.50 |
| the question without its "w1 changes nothing" sentence | 0.42-0.47 | 0.53-0.58 |
| the Mace's line cut to its move alone ("lab_16324 makes a Mace: no longer idle") | 0.30 | 0.70 |
| w1, the Mace, the Mace with a merge | 0.38-0.40 | 0.20-0.24 (+0.38-0.40 on the pair) |

The recorded Mace line ended with the idle list and the store, meant for w1's line alone: the deviation's
"only what differs" printed the whole list when one member changed (fixed with the two-stage pick).

## 3. World 1 de-privileged (44 picks: 16 idle-factory, 14 w1 won, 14 w1 lost)

| set | framing | w1 median p | chose w1 | chose a build | builds' summed mass |
|---|---|---|---|---|---|
| idle factory | as recorded | 0.11 | 6 of 16 | 2 | 0.48 |
| idle factory | no w1 sentence / momentum sentence / w1 as a plan / w1 last | 0.10-0.12 | 5-6 | 2-3 | 0.46-0.54 |
| w1 had won | as recorded / no w1 sentence | 0.15 / 0.17 | 10 / 12 of 14 | 0 / 1 | 0.01 |

No framing moves it: w1 wins as the plurality of a spread (0.11 against 31 build worlds at 0.07 for the best),
not by preference.

## 4. Two stages (all 92 idle-factory seconds, plus the 28 others)

| final pick on the 92 idle-factory seconds | recorded | two-stage |
|---|---|---|
| world 1 | 44 | 8 |
| a factory build | 9 | 29 |
| a constructor or commander step | 19 | 18 |
| a hunt or attack | 13 | 12 |
| a merge | 2 | 10 |

Stage two: w1 at a median 0.27, won 8 of 92. Stage one still split the build (its mass 0.63, a build won stage one
32 of 92). The user's ruling: stage one sampled by the mass, stage two a plain Choice; built as H-HANDS-ONE-PASS
amended 2026-09-29 and tried in player-27.

## 5. Jev's cost and where the tokens go (player-28, the win)

At $0.042 a million input tokens (output free; the user, 2026-09-30) player-28 cost $2.59: 61.7M input tokens over
2,497 calls, 44.3M of them the gate. Of the text, 65% is questions and 35% state; 49% of the gate questions' text is
the preamble repeated 214,681 times; a builder's states (85,687 questions, 48M characters) and a group's advances
and walks to every named place (88,874, 41M) are half of everything.

## 6. Pruning replayed offline (24 gate calls of 6,407 nouls; 40 stage-two and 30 stage-one picks)

| gate variant | mean change in a noul | flag flips | moves of 0.2 or more | input tokens, median |
|---|---|---|---|---|
| as recorded, again (Jev's noise) | 0.017 | 33-36 of 6,407 | 2 | 58k |
| preamble once, every kind | 0.033 | 77 | 32 | 39k |
| preamble once, the move only (no current course) | 0.042 | 81 | 112 | 32k |
| the bulk kinds only (builders, walks, advances) | 0.032 | 56 | 28 | 40k |
| the bulk kinds, the current course cut to its first clause | 0.032 | 54 | 27 | 38k |
| no `places` in the state | 0.023 | 48 | 7 | 48k |
| the bulk kinds short and no `places` | 0.030 | 54 | 30 | 35k |

The flips of the every-kind form sat on the factory nouls (7 of 20) and the actor-change nouls (5 of 63), which
hover near 0.5; builder states flipped 1 of 605 and destinations 1 of 957. Built: the bulk kinds short under one
`asking` line in the state, and no `places` (the words name each place's distance and what stands at it): about 40%
off the gate, some 30% off the game, roughly $0.75 of $2.59.

| pick variant | same choice as recorded | mean change in w1's p | tokens, median |
|---|---|---|---|
| stage two as recorded, again | 38 of 40 | -0.005 | 9.4k |
| stage two, only the actors and places its two lines name | 34 of 40 | -0.020 | 8.3k |
| stage two, no `places` | 36 of 40 | -0.034 | 6.4k |
| stage one as recorded, again | 26 of 30 | 0.000 | 14.1k |
| stage one, no `places` | 24 of 30 | 0.000 | 11.2k |

The pick's states are already dieted to what the worlds name; the savings there are a tenth of a game's cost for a
measurable drift, so the picks keep their state apart from `places`, which goes with the gate's (one state feeds
both).
