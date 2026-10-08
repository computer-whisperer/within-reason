# Raid packets: what let a Rover swarm burn red's base while six Blitzes stood by

**Status.** 2026-10-07 night, from the user's human-26 review at 3:30 (seat 0, "red", t2, the north-east base).
Offline replays of the recorded gates under alternate packets and alternate move words (`run/jev_packet_ab.py`,
variants and rows under `docs/studies/data/raid-packets/`, $0.03 in all; noise floor 0.01 from
`docs/studies/2026-10-07-rove-words.md`). Findings only; nothing built from it yet.

## The moments (seat 0, Jev's log and the player's)

- 2:50 seven enemy Rovers in red's picture at yellow's base (shared vision); out of sight 3:06-3:33 crossing the
  middle; the player's 3:27 report (a timer wake) lists only what is in sight. No warning.
- 3:33 the seven appear on red's H2 extractor. The packet in force (2:49) ends its Blitz paragraph with "Enemy
  armfav groups of three or more at our constructors or extractors: the nearest Blitz group fights them there".
  Six Blitzes stand at home. Jev: party_1 needs answering 0.80-0.90, group_B should change 0.69, attack party_1
  0.33-0.49, detachments forbidden 0.64-0.75 (the same paragraph's "a lone enemy raider ... never the whole
  group"). Nothing clears 0.5 for six gates.
- 3:39 the player's 3:27 packet lands, written before the raid was visible: "group_B_t2: the raid. It goes west
  now as one group by spot_18, then spot_27 ... Enemy armfav at our home extractors are not its business once it
  has left spot_18." Jev picks the walk to spot_18 (0.52) at 3:40 and the advance to spot_27 (0.75) at 3:49.
- 3:53 the player's 3:45 packet lands: "group_B_t2: attacks the armfav at our home now with the whole group and
  keeps attacking until they are dead or gone". From spot_27, 1,500 away, the attack rates 0.24-0.49 through
  4:05; the group holds. The player's 3:56 packet ("attacks party_1 now ... until it is dead") lands at 4:04,
  same ratings. At 4:07 a lone Rover comes within reach of spot_27 and a hunt is picked; the swarm comes to the
  Blitzes at spot_11 at 4:10-4:25 and dies there.

## Task 1: the 3:33-3:38 gates under the 2:49 packet

Variants: `raid_first` puts a raid paragraph at the top of the packet ("Raids on our base come before everything:
an enemy party at our extractors, constructors or home is answered at once by the nearest Blitz group with the
whole group, whatever it was gathering for or walking to; the route resumes when none stands in our base.");
`raid_stand` adds "They outrun Blitzes, so a chase is not the point: Blitzes standing on what the raiders want
drive them off or kill them, so the group goes to the building they are shooting and fights there."; `words_nooutrun`
cuts the move's "it outruns everything here ... it chooses the fight" clause (the hands' words, not the packet).

| gate | variant | attack_party_1 | change | send_4_party_1 |
|---|---|---|---|---|
| 3:33 | recorded | 0.44 | 0.70 | 0.41 |
| 3:33 | words_nooutrun | 0.51 | 0.71 | 0.43 |
| 3:33 | raid_first | 0.61 | 0.75 | 0.44 |
| 3:33 | raid_stand | 0.64 | 0.76 | 0.43 |
| 3:33 | raid_first_nooutrun | 0.60 | 0.75 | 0.46 |
| 3:34 | recorded | 0.42 |  | 0.43 |
| 3:34 | words_nooutrun | 0.44 |  | 0.44 |
| 3:34 | raid_first | 0.61 |  | 0.44 |
| 3:34 | raid_stand | 0.61 |  | 0.44 |
| 3:34 | raid_first_nooutrun | 0.55 |  | 0.47 |
| 3:36 | recorded | 0.34 |  | 0.35 |
| 3:36 | words_nooutrun | 0.37 |  | 0.39 |
| 3:36 | raid_first | 0.45 |  | 0.33 |
| 3:36 | raid_stand | 0.51 |  | 0.27 |
| 3:36 | raid_first_nooutrun | 0.46 |  | 0.35 |
| 3:37 | recorded | 0.50 |  | 0.40 |
| 3:37 | words_nooutrun | 0.45 |  | 0.42 |
| 3:37 | raid_first | 0.56 |  | 0.42 |
| 3:37 | raid_stand | 0.59 |  | 0.37 |
| 3:37 | raid_first_nooutrun | 0.53 |  | 0.40 |

Medians (gates at 0.5 or over / asked):

| variant | attack_party_1 | change | send_4_party_1 |
|---|---|---|---|
| recorded | 0.43 (1/4) | 0.70 (1/1) | 0.41 (0/4) |
| words_nooutrun | 0.45 (1/4) | 0.71 (1/1) | 0.42 (0/4) |
| raid_first | 0.58 (3/4) | 0.75 (1/1) | 0.43 (0/4) |
| raid_stand | 0.60 (4/4) | 0.76 (1/1) | 0.40 (0/4) |
| raid_first_nooutrun | 0.54 (3/4) | 0.75 (1/1) | 0.43 (0/4) |

The packet is the lever: the raid paragraph at the top lifts the attack from 0.43 to 0.58-0.60 and 3-4 of 4 gates
clear; the 2:49 packet's own raid sentence, last in the Blitz paragraph behind a gather and a route, did not. The
move's outrun words alone change little (0.45).

## Task 1b: the 3:40-3:49 gates under the 3:27 route packet

`tolerant`: the "not its business" sentence cut and the raid paragraph added; `noturn`: the move words without
the turn-cost clause (below).

| gate | variant | attack_party_1 | change | go_spot_18 | go_spot_27 |
|---|---|---|---|---|---|
| 3:40 | recorded |  | 0.71 | 0.52 | 0.41 |
| 3:40 | noturn |  | 0.71 | 0.49 | 0.42 |
| 3:40 | tolerant |  | 0.72 | 0.41 | 0.40 |
| 3:40 | tolerant_noturn |  | 0.73 | 0.41 | 0.42 |
| 3:41 | recorded | 0.41 | 0.41 |  | 0.30 |
| 3:41 | noturn | 0.42 | 0.44 |  | 0.31 |
| 3:41 | tolerant | 0.46 | 0.54 |  | 0.30 |
| 3:41 | tolerant_noturn | 0.48 | 0.50 |  | 0.35 |
| 3:46 | recorded | 0.39 | 0.46 |  |  |
| 3:46 | noturn | 0.37 | 0.46 |  |  |
| 3:46 | tolerant | 0.43 | 0.68 |  |  |
| 3:46 | tolerant_noturn | 0.46 | 0.70 |  |  |

Medians (gates at 0.5 or over / asked):

| variant | attack_party_1 | change | go_spot_18 | go_spot_27 |
|---|---|---|---|---|
| recorded | 0.40 (0/2) | 0.46 (1/3) | 0.52 (1/1) | 0.35 (0/2) |
| noturn | 0.40 (0/2) | 0.46 (1/3) | 0.49 (0/1) | 0.36 (0/2) |
| tolerant | 0.45 (0/2) | 0.68 (3/3) | 0.41 (0/1) | 0.35 (0/2) |
| tolerant_noturn | 0.47 (0/2) | 0.70 (3/3) | 0.41 (0/1) | 0.39 (0/2) |

With the tolerant packet the group would not have walked off (go_spot_18 0.41, change 0.68-0.70) but would not
have attacked either (0.43-0.48): it holds at home. Half a fix.

## Task 2: the 3:54-4:05 gates under the 3:45 attack packet (the ignored order)

Packet variants: `by_name` ("attacks party_1 now" for "the armfav at our home"), `no_route` (the paragraph cut to
the attack sentence), `far_ok` ("It is at spot_27 now, far from home: it walks back and attacks; the distance is
no reason to wait"); `words_nooutrun` as above.

| gate | variant | attack_party_1 | fight_to_home | go_home |
|---|---|---|---|---|
| 3:54 | recorded | 0.49 |  |  |
| 3:54 | by_name | 0.49 |  |  |
| 3:54 | no_route | 0.43 |  |  |
| 3:54 | far_ok | 0.43 |  |  |
| 3:54 | words_nooutrun | 0.48 |  |  |
| 3:56 | recorded | 0.27 | 0.21 | 0.11 |
| 3:56 | by_name | 0.36 | 0.17 | 0.12 |
| 3:56 | no_route | 0.29 | 0.19 | 0.11 |
| 3:56 | far_ok | 0.29 | 0.41 | 0.19 |
| 3:56 | words_nooutrun | 0.31 | 0.22 | 0.13 |
| 3:59 | recorded | 0.30 | 0.27 | 0.12 |
| 3:59 | by_name | 0.35 | 0.27 | 0.13 |
| 3:59 | no_route | 0.32 | 0.32 | 0.12 |
| 3:59 | far_ok | 0.28 | 0.42 | 0.22 |
| 3:59 | words_nooutrun | 0.29 | 0.33 | 0.15 |
| 4:01 | recorded | 0.24 | 0.35 | 0.16 |
| 4:01 | by_name | 0.35 | 0.28 | 0.13 |
| 4:01 | no_route | 0.25 | 0.36 | 0.17 |
| 4:01 | far_ok | 0.28 | 0.44 | 0.27 |
| 4:01 | words_nooutrun | 0.27 | 0.32 | 0.15 |
| 4:03 | recorded | 0.30 | 0.25 | 0.13 |
| 4:03 | by_name | 0.35 | 0.19 | 0.12 |
| 4:03 | no_route | 0.30 | 0.27 | 0.12 |
| 4:03 | far_ok | 0.29 | 0.46 | 0.23 |
| 4:03 | words_nooutrun | 0.25 | 0.24 | 0.12 |
| 4:04 | recorded |  |  |  |
| 4:04 | words_nooutrun |  |  |  |

Medians (gates at 0.5 or over / asked):

| variant | attack_party_1 | fight_to_home | go_home |
|---|---|---|---|
| recorded | 0.30 (0/5) | 0.26 (0/4) | 0.12 (0/4) |
| by_name | 0.35 (0/5) | 0.23 (0/4) | 0.12 (0/4) |
| no_route | 0.30 (0/5) | 0.30 (0/4) | 0.12 (0/4) |
| far_ok | 0.29 (0/5) | 0.43 (0/4) | 0.23 (0/4) |
| words_nooutrun | 0.29 (0/5) | 0.28 (0/4) | 0.14 (0/4) |

No packet wording moves it: the attack stays 0.24-0.36 and the walk home under 0.5. The packet is not what Jev
is reading against.

## Task 2b: the same gates with the turn-cost words cut

The attack's words at 3:59 end "group_B_t2 has changed course 2 times in the last 18 s and stands 1671 from where
it stood at the first of them: a body that keeps turning walks and does not arrive or fight" (H-HANDS-TURNS, the
thrash words of 2026-10-03). `noturn` cuts that clause from the group's questions.

| gate | variant | attack_party_1 | fight_to_home | send_4_party_1 |
|---|---|---|---|---|
| 3:54 | recorded | 0.48 |  | 0.44 |
| 3:54 | raid_first | 0.57 |  | 0.36 |
| 3:54 | noturn | 0.57 |  | 0.45 |
| 3:54 | noturn_by_name | 0.66 |  | 0.44 |
| 3:54 | noturn_raid_first | 0.73 |  | 0.36 |
| 3:56 | recorded | 0.31 | 0.21 | 0.34 |
| 3:56 | raid_first | 0.41 | 0.25 | 0.32 |
| 3:56 | noturn | 0.43 | 0.44 | 0.35 |
| 3:56 | noturn_by_name | 0.49 | 0.38 | 0.34 |
| 3:56 | noturn_raid_first | 0.52 | 0.50 | 0.33 |
| 3:59 | recorded | 0.30 | 0.30 | 0.39 |
| 3:59 | raid_first | 0.37 | 0.36 | 0.32 |
| 3:59 | noturn | 0.44 | 0.49 | 0.39 |
| 3:59 | noturn_by_name | 0.50 | 0.41 | 0.43 |
| 3:59 | noturn_raid_first | 0.54 | 0.56 | 0.34 |
| 4:01 | recorded | 0.24 | 0.33 | 0.36 |
| 4:01 | raid_first | 0.29 | 0.39 | 0.33 |
| 4:01 | noturn | 0.40 | 0.53 | 0.36 |
| 4:01 | noturn_by_name | 0.48 | 0.47 | 0.35 |
| 4:01 | noturn_raid_first | 0.45 | 0.61 | 0.34 |

Medians (gates at 0.5 or over / asked):

| variant | attack_party_1 | fight_to_home | send_4_party_1 |
|---|---|---|---|
| recorded | 0.30 (0/4) | 0.30 (0/3) | 0.38 (0/4) |
| raid_first | 0.39 (1/4) | 0.36 (0/3) | 0.33 (0/4) |
| noturn | 0.43 (1/4) | 0.49 (1/3) | 0.38 (0/4) |
| noturn_by_name | 0.49 (2/4) | 0.41 (0/3) | 0.39 (0/4) |
| noturn_raid_first | 0.53 (3/4) | 0.56 (3/3) | 0.34 (0/4) |

The thrash words are what held the turn-back down, and the packet's priority paragraph is the other half: cut
alone 0.43/0.49, paragraph alone 0.39/0.36, both 0.53/0.56 with 3 of 4 attacks and 3 of 3 walks home over the
bar. The two course changes the words counted were the packet's own route stops: spot_18 reached at 3:48, spot_27
picked at 3:49 and reached at 3:55. A group that reaches a named stop and leaves for the next has not turned.

## Reading

1. **Opus's packet closed the door twice.** At 2:49 the raid rule was last in a paragraph that opened with a gather
   and a route; at 3:27 it said in words that home raids were not the raid's business. A raid paragraph at the top,
   general and imperative, is what Jev follows (Task 1). This is the per-model tuning the user suspects: the
   packet's shape, not its facts.
2. **The turn-back was refused by our own thrash words**, which count a route's stops as turns and so price the
   one turn that mattered. Candidate refinement (not built): a stop reached and left for the next named place is
   progress, not a turn; count only courses abandoned before their end. Test it on player-58's 5:55-6:50 (the
   thrash case the words were built on) before changing anything.
3. **No warning was possible from the report.** The report says what is in sight; a party last seen 27 s ago
   heading north does not exist in it. The picture's `enemy` section has the same gap. Whether the player would
   have reasoned "they left yellow's base intact, they are coming here" is moot while the report cannot say it.
