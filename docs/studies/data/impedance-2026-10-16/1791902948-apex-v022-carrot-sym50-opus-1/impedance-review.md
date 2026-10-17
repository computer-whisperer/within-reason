# Impedance review: apex-v022-carrot-sym50-opus-1 / 00

We won at 22:17 against Apex v0.2.2 (standard profile) on Carrot Mountains v2.0, with a +50 income bonus for both sides. There was one seat (strategist-0): Opus 5.5 medium played under an Opus 5.5 high commander, with all twelve hand layers on. The truth file was read. Every tool read seat 0, so no symlink directories were needed.

## Tally

| category | count | | category | count |
|---|---|---|---|---|
| A1 refused at the tool | 3 | | B1 in the hands' picture, not the report | 0 |
| A2 accepted, not executed | 7 | | B2 in record/truth, in nobody's picture | 0 |
| A3 overruled by the pick | 11 | | B3 report wrong/stale | 0 |
| A4 overruled by code | 7 | | B4 in the report, missed by Opus | 3 |
| A5 late | 0 | | B5 hands' lines misleading | 1 |
| A6 read differently | 2 | | B6 commander-player hop | 1 |
| **C (clean moments)** | **0 of 8** | | | |

Denominators:
- Judgement moments: 8 examined, 0 clean. The push at 6:37 was followed except for one route skip that cost nothing.
- Orders: 281 from the player, 19 from the commander.
- Hands' lines: 1,520, of which 109 were flagged.
- Refused: 2 player calls. One commander `mark` was also turned away with an error, but it was not labelled REFUSED.

The A3 count is mostly one pattern. In 5 of the 11 A3 items the pick took a detachment whose gate `forbidden` reading was 0.61 to 0.69, mostly where the decode had already read the packet's "whole" or "never sends detachments" at 0.54 to 0.72. In the fifth, the 17:38 Hound hunt, the sentence crossed was an east limit, not a ban on detachments.

The A4 count is mostly the leash. Of the 31 `hunt-ended-leash` lines, 27 fall in 6 quarry episodes where the packet in force wanted that raider dead; these are counted once per episode. The other 4 lines are not items:
- 13:35: a lone constructor.
- 14:17 and 14:46: Pawns that group_A or the picket was told not to chase.
- 17:39: a Hound beyond the guard's east limit.

16 of the 31 hunts ended within 2 s of starting, which means the quarry was already beyond the 900 leash when the pick took the hunt.

## Items by cost (15 of 35; all 35 are in impedance.json)

1. **1:08, strategist-0, A6, large, high.**
   - Wanted: the opening list `["extractor","extractor","armsolar","armsolar","armvp","extractor",...]`.
   - Happened: the armvp step placed plant_13578 facing east with spot_99 in its exit lane. The next bare "extractor" step built armmex_29599 on that spot at 1:08. armcv_23315 hit move_failed every second from 1:27 to 2:38, and the plant built nothing from 1:26 to 2:39 (73 s).
   - Evidence: the 1:57 wake, "exit lane is blocked ... in the lane: armmex_29599". Commit 3aabc77, made after this game, cites it.
   - Fix: code-rule (done in 3aabc77).
2. **2:45, strategist-0, A4, large, high.**
   - Wanted: `queue commander ["extractor spot_87","armvp #vp2","extractor spot_78",...]` at 2:37, and spot_78 again at 3:07.
   - Happened: "the step 'extractor spot_87' of its list is the build already under way: taken as done", although nothing had started. The next step replaced the walk a second later. The same happened to spot_78 at 3:15.
   - Cost: spot_87 started at 6:03 and spot_78 at 6:41, about 3.3 min late each and roughly 750 metal at the spots' base yield, during the 3-4-extractor opening.
   - Fix: code-rule.
3. **8:55, strategist-0, B4, large, medium.**
   - Opus's note: "group_A (31 Blitzes ...) is at H4 beside his base ... Sending it into the lab now".
   - The report had said: "reinforcements: 23 of its soldiers are on the way ... the farthest 8286 behind ... not in its fights yet" and "within 800 of 0 of its 12".
   - Happened: the body went in front first ("3 of 12 arrived, its tail 957 short") and stalled at spot_98. Opus pulled it back at 10:04.
   - Cost: 1,886 metal lost at H5 in minute 10, an even trade by the commander's count (2,830 each).
   - Fix: report. The group's headline should give the number in the body and the number on the way.
4. **15:02, strategist-0, A3, small, high.**
   - Wanted: group_V "attacks it whole until it is dead or gone".
   - Happened: the pick split it into hunts of 2, 4, 8 and 8 (`2 of group_V hunt party_86 ... as group_B1`, and so on). The forbidden readings were 0.66 to 0.69 and the decode read detach at 0.54 to 0.63.
   - Cost: the D6 raid then took spot_108 (15:28), spot_111 (15:30) and a Sentry (15:35).
   - Fix: code-rule (a forbidden reading of 0.5 or more should shut the detachment).
5. **13:10, strategist-0, A3, small, medium.**
   - Wanted (12:45): "a party of three or more ... shooting a plant ...: the picket and the commander fight it".
   - Happened: party_22 (5-6 Pawns) shot the advanced plant from 13:11 to 13:20. The commander was closed by the news layer (attack_party_22 rated 0.22-0.42), and group_V took single Pawns elsewhere (13:09, 13:19).
   - Cost: the plant (2,600 metal) and a 97% Bull died at 13:20. Neither actor could have arrived within the 9 s of fire, so the loss is not attributed to the pick.
   - Fix: jev.
6. **6:48, strategist-0, A4, small, high.**
   - Wanted: the picket keeps attacking a Tick "until it is dead or gone".
   - Happened: the hunts of party_15 ended on the 900 leash (after 7, 5 and 0 s; a fourth at 7:11). The Tick killed spot_87 the same second; it was rebuilt at 7:32.
   - Fix: code-rule.
7. **18:15, strategist-0, A4, small, medium.**
   - Wanted (18:01): the whole guard takes a party of three or more, and 2-4 Blitzes take a single raider.
   - Happened: group_Y2's hunt of party_123 (5 Platypus, 2 Blitzes) ended on the leash after 9 s, and spot_69 fell to that party at 18:18. Three hunts of one Pawn ended after 0-1 s.
   - Fix: code-rule.
8. **15:08, strategist-0, A4, small, medium.**
   - Happened: the hunts of party_86 and party_82 ended after 5-6 s, including group_E1's 14 soldiers. The same raiding party killed two extractors at D6 about 20 s later.
   - Fix: code-rule.
9. **17:30, strategist-0, A4, small, medium.**
   - Wanted (17:07): single raiders at our extractors are the guard's job "at once".
   - Happened: six hunts of single raiders ended after 0-13 s. While K2's hunt of party_112 was cut, D3 lost spot_56 (17:44) and spot_55 (17:48).
   - Fix: code-rule.
10. **2:37, strategist-0, B4, small, high.**
    - Opus queued `"extractor spot_99"` on constructor_23315 after its own 1:57 note, "the extractor at spot_99 sealed the plant's exit".
    - Happened: the extractor was rebuilt at 3:52, constructor_1657 was stuck in the lane from 4:31 to 5:39, and a second round of removals ran from 4:47 to 5:37.
    - Fix: tool. The queue answer could name the spots that lie in a factory's lane.
11. **2:17, strategist-0, A2, small, high.**
    - Wanted: `remove {"by":"commander","reclaim":["armmex_29599"]}`.
    - Happened: the reclaim started at 2:15 and stopped at 2:17 with the extractor at 87%, when the next list step walked the commander away. No hands line said so. Opus found out at 2:27 ("Reclaim ... didn't happen") and destructed it.
    - Fix: code-rule.
12. **9:01, strategist-0, A3, small, high.**
    - Wanted: group_A "never comes back west"; Ticks are the picket's job.
    - Happened: PICK `8 of group_A hunt party_18 (4 armflea) as group_M`, about 4,000 west of the body. The forbidden reading was 0.66 and the decode read detach|8 at 0.64. The group shrank from 37 to 29 just before the base attack; 2 of the 8 went into his base alone.
    - Fix: code-rule.
13. **16:55, strategist-0, A3, small, high.**
    - Wanted: "Pawns ... raiding on our side are the home guard's job, never group_Q1's".
    - Happened: the whole army (73-78) took `shell party_104` (one Pawn) and walked about 900 west for 20 s after the strike, until the 17:07 rewrite.
    - Fix: jev.
14. **14:58, strategist-0, A3, small, high.**
    - Wanted: "group_A: it stands at his_com (beside spot_79) as one body and holds there".
    - Happened: PICK "hold where it stands" (rated 0.69) in F5, 3,400 short of his_com, held until 16:09. The decode read his_com at 0.96.
    - Fix: jev.
15. **15:33, strategist-0, A3, small, medium.**
    - Wanted: "group_E1, group_D1, group_F1 and group_L1 ... all attack that party together now, whole".
    - Happened: group_E1 (9-18 soldiers) held at spot_110 until 15:49. Its attack moves were rated 0.35-0.51, and the news layer closed it from 15:34 to 15:38.
    - Cost: a Sentry lost at 15:35.
    - Fix: jev.

Not ranked above (details in the JSON):
- A1: 4:47 destruct refused for blast; 12:19 empty `remove`; 15:23 the commander's `mark` given "spot_101".
- A2: 2:36 the commander cannot build armnanotc; 5:08 the reclaim by constructor_1657 never played; 7:21 and 13:25 a Sentry step skipped although our own extractor stood on the spot; 14:17 the commander cannot build armavp; 18:30 four solars placed on one site and refused.
- A3: 6:41 the picket split; 6:48 group_A sent 8 after a Tick; 7:56 spot_64 skipped; 16:51 a second advanced plant started; 17:38 the guard hunted a Hound 7,600 east.
- A4: 9:01 and 15:40 leash episodes that cost nothing.
- A6: 16:14 plant_2408 ran the opening `produce all` list.
- B4: 15:41 "his_com, where group_A holds", while every report put group_A in E5-G5.
- B5: 2:45 "taken as done".
- B6: 16:53 the commander's report showed plant_13578 "idle" while it made a Blitz every 4-6 s.

Opus notes checked against the logs:
- 13:25 "the commander never turned ... despite the raid sentence": the sentence landed at 13:22, after the plant died at 13:20. The 12:45 raid clause was in force, and the commander was closed by the news layer.
- 17:46 "37 of group_Q1's 85 soldiers were held back by flee footwork ... its advance crawled": not borne out. The body's centroid moved at about 53 elmo/s before the lane change and 48 after, which is Stout pace.

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 124 turns, 3 session replacements, 281 orders, 2 refused, 109 flagged of 1520 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 124, 'queue': 45, 'instruct': 44, 'produce': 26, 'mark': 20, 'signal': 9, 'lane': 7, 'remove': 6}
   refused: {'remove': 2}
   flagged: {'hunt-ended-leash': 31, 'step-skipped': 23, 'engine-refused': 18, 'hunt-ended-other': 14, 'stopped-on-stop': 11, 'other-fault': 4, 'abandoned': 4, 'step-waits': 3, 'follow-ended': 1}
-- commander.jsonl: 15 turns, 0 session replacements, 19 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 16 s
   orders: {'direct': 15, 'mark': 3, 'signal': 1}
   flagged: {}
```

How the flagged lines were judged:
- **Engine-refused (18):** 8 were extractor orders on spots where our own extractor already stood; they led to the two Sentry items. 4 were solars placed on one site at 18:30. The rest were solars and wind turbines on a crowded field at 10:28-11:07, with no measured cost.
- **Abandoned (4):** all followed the player's own list replacements or stops.
- **Other-fault (4):** 3 were steps dropped by a list replacement that the new list carried again; 1 was the commander's `assist` at 14:17.

## What Opus wanted that the harness has no lever for

- **6:16 onward, strategist-0, in every packet: constructors "never stand in a plant's exit lane".** The help-a-plant move stands the helper wherever the engine guards from, often inside the lane. The 12:27 picture shows it: "helping lab_31293 build ... standing in plant_31293's exit lane since 28 s ago: its units cannot leave". No move or rule helps from beside the lane. Opus moved helpers out by hand with lists (12:29, 17:27, 21:44) after the yard wakes.
- **12:45, strategist-0 (commander 12:23 and 21:23, also 15:23 "as one wide line"): "every soldier in the same moment from several sides".** The hands have advance, fight_to and attack-party moves, plus the lane's form (ranks across one heading) and march. Nothing splits one group into converging approaches or times their arrival.
- **9:23 and 19:53, commander: "The Rovers in group_G look for it ... in H6-H8 and H1-H3", and "rove the cells seen least (F7, E7, E3, D3)".** Roving picks its own targets in code, "beyond your hands' reach". The player's packets could only say "group_G (Rovers): it roves. Leave it to rove."

## Moments examined

- (a) The opening, 0:00-4:00: the lane block, "taken as done", the reclaim, the nano, spot_99.
- (b1) The Tick raid at spot_87, 6:41-6:48.
- (b2) The raid on the advanced plant, 13:01-13:20.
- (b3) The D6 raid, 15:02-15:50.
- (b4) The D3 raid, 17:30-17:58.
- (c) The first push order, 6:37, through group_A's route to 8:50.
- (d1) The 8:55 packet sending group_A into his base.
- (d2) The 14:49 packet telling group_A to hold at his_com.

At each moment the report's enemy, scouting and group lines were put beside the truth file and the record's `enemy_seen`. No B1, B2 or B3 was found. The report carried group shape, reinforcements, stuck units and exit lanes. His second Beamer at spot_98 was built at 8:34 and first seen at 9:07, so it was unknowable at 8:55. The hands' block in the report matched the log's plays in the three windows compared (6:37-6:56 and 17:36-17:46).

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "import json;print(json.load(open('MATCH/../batch.json')))"
run/impedance_sweep.py MATCH --counts | --seat 0 | --commander | --seat 0 --all --from 2:37 --to 3:50
run/minutes.py MATCH; run/wake_read.py MATCH; run/raid_ledger.py MATCH; run/pro_baseline.py MATCH [--all-starts] (no carded side on this map)
run/layers_read.py MATCH --faults
run/hands_window.py MATCH 13:00 13:35 commander group_V | 6:38 6:52 | 6:47 7:05 group_C.. | 15:32 15:46 group_*1 | 15:33 15:50 group_E1 | 17:34 17:50 | 15:01 15:12 | 14:56 15:06 group_A | 7:53 7:58 group_A
commander.jsonl / strategist-0.jsonl read with python (calls, results, prompts at 8:55, 9:28-9:54, 14:59-16:01, 13:05, 16:53 commander)
jev-0.jsonl.zst digested once through run/jevlog.py (decode, instructions, groups, pass menus, gate flags, plans, audits) into a scratch pickle
record-0.jsonl events/cmd/decision rows and truth-0.jsonl read with scratch scripts; bot.log grepped for "never started"
```
