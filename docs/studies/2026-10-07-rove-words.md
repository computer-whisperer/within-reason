# The rove move's words

**Status.** 2026-10-07 night, after rove-move-1 (`docs/experiments.md`): the group menu's `rove` move (roving as a
pick, `docs/design/2026-10-07-rove-approval.md`) was offered at 1,905 gates and never picked, 0.10 under a packet
paragraph reading "group_R1: roves. Its one move is rove; it never follows or joins another group". The user: "Let's
try replaying the gates with rewordings." Tool: `run/jev_rove_words_ab.py`; data:
`docs/studies/data/jev-rove-words-1791420311-rove-move-1.jsonl`; cost $0.16. The `plain` words are in the menu since
(this commit).

## Method

Every gate of rove-move-1 whose request carried a `group_X.rove` question (551; the `same` layer resends a question
only when its words change, so the other 1,354 flags were standing answers). Each gate is classed by the packet in
force: `named` (a line with "rove" naming this group), `generic` (a line with "rove" naming no group, and the group
is fast units: "Rover groups (armfav): they rove, from the first Rover"), `none` (no such line, or a Stout army under
a generic line: a wording that lifts rove here is a bad wording). All 6 named, 17 generic and 20 of the 528 none
gates spread over the game; each sent whole, as recorded, with only the rove question's "The move: ..." replaced.
`recorded` is the recorded words asked again (the noise floor: median 0.01 from the logged answer).

The wordings, for group_X:

- `recorded`: "group_X roves whole: every soldier runs on its own in code, ten times a second, beyond your hands'
  reach from then on: it looks at what we know least, his start box first, kills what it finds unguarded (a
  constructor, an extractor, a radar) and keeps out of the reach of anything that can shoot it, whatever the odds;
  nothing of your hands moves it again until the player takes it back"
- `nofear`: the same without "beyond your hands' reach from then on", "whatever the odds" and "nothing of your
  hands moves it again until the player takes it back"
- `scoutlike`: the scout move's words for the whole group: "group_X roves: every soldier looks at what we know
  least, his start box first, kills what it finds unguarded and keeps out of the reach of anything that can shoot
  it, on its own in code; the rest of your hands' moves for it end"
- `short`: "group_X roves: its soldiers scout on their own, his start box first, kill what they find unguarded and
  stay out of reach"
- `plain`: "group_X roves: its soldiers scout his side on their own, looking at what we know least, killing
  unguarded constructors and extractors, and keeping out of every reach; the player's lane setting brings it back"

## Result

Median rove noul by class and wording (gates at 0.5 or over in brackets):

| class | n | recorded | nofear | scoutlike | short | plain |
|---|---|---|---|---|---|---|
| named | 6 | 0.15 (0) | 0.37 (1) | 0.57 (5) | 0.64 (5) | 0.65 (5) |
| generic | 17 | 0.17 (1) | 0.40 (7) | 0.55 (11) | 0.60 (11) | 0.65 (12) |
| none | 20 | 0.04 (0) | 0.05 (0) | 0.06 (0) | 0.08 (0) | 0.07 (0) |

Per gate where the packet asked:

| gate | group | class | logged | recorded | nofear | scoutlike | short | plain | best other move (recorded) |
|---|---|---|---|---|---|---|---|---|---|
| 1:08 | group_A | generic | 0.12 | 0.10 | 0.51 | 0.55 | 0.61 | 0.65 | hold 0.49 |
| 1:32 | group_A | generic | 0.25 | 0.15 | 0.39 | 0.60 | 0.60 | 0.65 | follow_commander 0.28 |
| 1:40 | group_A | generic | 0.19 | 0.17 | 0.33 | 0.51 | 0.58 | 0.59 | hold 0.34 |
| 1:41 | group_A | generic | 0.18 | 0.20 | 0.40 | 0.52 | 0.57 | 0.62 | follow_constructor_18284 0.30 |
| 1:43 | group_A | generic | 0.16 | 0.13 | 0.27 | 0.40 | 0.42 | 0.49 | go_home 0.45 |
| 2:10 | group_A | generic | 0.17 | 0.17 | 0.33 | 0.47 | 0.44 | 0.52 | go_home 0.44 |
| 2:15 | group_A | named | 0.14 | 0.15 | 0.34 | 0.63 | 0.65 | 0.62 | attack_party_1 0.57 |
| 2:16 | group_A | named | 0.09 | 0.14 | 0.13 | 0.21 | 0.36 | 0.26 | send_1_party_1 0.40 |
| 3:02 | group_A | generic | 0.06 | 0.06 | 0.07 | 0.18 | 0.16 | 0.10 | gather 0.49 |
| 3:02 | group_D | generic | 0.07 | 0.06 | 0.08 | 0.36 | 0.24 | 0.17 | fight_to_spot_63 0.34 |
| 3:07 | group_A | named | 0.18 | 0.17 | 0.40 | 0.56 | 0.67 | 0.67 | gather 0.26 |
| 3:07 | group_D | named | 0.26 | 0.32 | 0.60 | 0.59 | 0.63 | 0.70 | go_spot_62 0.22 |
| 20:52 | group_J1 | generic | 0.40 | 0.40 | 0.55 | 0.67 | 0.78 | 0.70 | join_group_P 0.54 |
| 21:00 | group_K1 | generic | 0.34 | 0.37 | 0.56 | 0.68 | 0.78 | 0.73 | go_spot_73 0.60 |
| 21:02 | group_K1 | generic | 0.34 | 0.38 | 0.54 | 0.66 | 0.81 | 0.71 | join_group_P 0.59 |
| 21:08 | group_L1 | generic | 0.29 | 0.35 | 0.58 | 0.64 | 0.78 | 0.70 | go_spot_73 0.60 |
| 21:10 | group_L1 | generic | 0.34 | 0.33 | 0.56 | 0.62 | 0.81 | 0.73 | join_group_P 0.61 |
| 21:14 | group_P | generic | 0.05 | 0.04 | 0.05 | 0.06 | 0.10 | 0.06 | gather 0.36 |
| 21:15 | group_P | generic | 0.05 | 0.03 | 0.05 | 0.06 | 0.10 | 0.07 | attack_party_52 0.28 |
| 21:16 | group_M1 | generic | 0.28 | 0.50 | 0.62 | 0.66 | 0.79 | 0.75 | go_spot_73 0.61 |
| 22:14 | group_Q1 | generic | 0.18 | 0.13 | 0.45 | 0.60 | 0.77 | 0.74 | go_spot_73 0.50 |
| 22:48 | group_R1 | named | 0.10 | 0.15 | 0.45 | 0.70 | 0.77 | 0.69 | send_2_party_58 0.21 |
| 23:10 | group_R1 | named | 0.09 | 0.07 | 0.26 | 0.50 | 0.59 | 0.54 | follow_group_D1 0.36 |

## Reading

- **The words were the cause.** Where the packet asked for roving the recorded words rated 0.15; `plain` and
  `short` rate 0.65 and 0.64 on the named gates, 17 and 16 of 23 at 0.5 or over. Where the packet did not ask,
  nothing moves: 0.07 at most, 0 of 20 at 0.5.
- **Not only the fear words.** Dropping "beyond your hands' reach", "whatever the odds" and "nothing of your hands
  moves it again" (`nofear`) lifts the median only to 0.37-0.40: "roves whole: every soldier runs on its own in
  code, ten times a second" weighs too. The plain description of what the soldiers do is what rates.
- **The chase stays a chase.** At 2:15-2:16 (group_A on his Rascal) and 3:02 (gathering after the chase) every
  wording stays under 0.5, with the attack or the gather rated higher: a group in a fight is not pulled off it by
  the packet's rove line, which is what human-26 1:57 asked for.
- **`plain` chosen** over `short`: the same medians, and it says the way back, which is true and new to Jev.
- Next: rove-move-2 with these words, to see the move picked in a game and the player's late-game Rover scouts
  actually rove.
