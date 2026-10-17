# Impedance review: apex-v022-carrot-sym0-opus-1/00

We won at 10:57 against Apex v0.2.2 (standard profile) on Carrot Mountains v2.0 at 0 bonus. We had one seat: an Opus 5.5 medium player under an Opus 5.5 high commander, with the twelve layers. His commander died at 10:50 near spot_105 (record `enemy_destroyed` u=765, 86 HP at 10:49), and all his units were gone by 10:54. The record does not say whether Apex also resigned.

## Tally

| A1 | A2 | A3 | A4 | A5 | A6 | B1 | B2 | B3 | B4 | B5 | B6 | C |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 1 | 4 | 2 | 1 | 5 | 0 | 1 | 0 | 1 | 0 | 4 | 1 |

Denominators:
- 8 judgement moments, 1 of them clean.
- Player: 43 turns, 97 orders, 0 refused, 349 hands' lines, 1 takeover at 10:27.
- Commander: 7 turns, 7 orders (4 `direct`), 0 refused.

Two items were large, both from the same shape: group_I attacking strung out. They cost 2,062 metal at 10:00-10:50 and 1,100 metal at 8:58-9:16. Every other item cost little or nothing.

Moments examined (8; the raid ledger is empty, with no extractor lost, so the raids come from the wake lines):

1. **Opening, 0:00-4:00: C.** Queue and produce were made in list order: plant 1:10, Rovers 1:15/1:19/1:44/1:52, constructors 1:39/2:21/3:05. The energy stall was in the eco line, and Opus fixed it at 1:35.
2. **Tick at A5, 6:19-6:31:** A4 (leash), A6 (group_I detachments).
3. **Tick through D6, C6 and our home, 7:37-8:25:** A3 (group_C kept holding).
4. **7:03 push into his half:** the way was walked in order (spot_93 7:52, spot_105 8:17, spot_116 8:27). The walk to spot_107 passed under turrets: A6, low confidence.
5. **8:48 attack order on his lab:** A3 (8:58), A5 (9:08), A3 (9:21), B6 (8:03 direction).
6. **2:55 packet:** A6 (the new picket group was never decoded).
7. **6:29 packet, the west picket:** A3 and B4 (group_G never moved).
8. **Last three turns, 10:27-10:47:** A6 (whole-group attack fed in piecemeal), B2 (his commander's health).

The truth check at those turns agrees with what Opus believed: his commander alone at 10:37 (the nearest Pawn 1,297 away), his army a few hundred metal. The truth also holds two turrets no unit of ours ever saw: a Beamer by stage_east from 8:34 and an HLT at (10256, 7072) from 7:42. Neither was available to Opus, so neither is an item.

## Items, by cost

1. **10:00-10:50 · A6 · large (medium confidence).**
   - Wanted: 9:58 "the whole group attacks it at once, every soldier"; 10:37 "close on it from all sides at once".
   - Happened: the pick played the whole-group attack (10:00 and 10:15) and it stayed in force, but soldiers arrived one or two at a time. Reports: 10:27 "within 800 of 2 of its 20 ... the other 18 are 800-6869 behind"; 10:37 "some of them standing with no order"; newcomers walked alone from the plants, 6,400 behind.
   - Cost: 16 of ours were killed by armcom (u=765) between 10:06 and 10:49, 2,062 metal.
   - Fix class: menu. A converge or synchronised-strike move is needed.
2. **8:58 · A3 · large (medium confidence).**
   - Wanted: 8:48 "Its way is stage_east, then his_lab, in that order ... At stage_east it closes up on its front".
   - Happened: `PLAYS group_I: advance to his_lab [plan] PICK ... took w2 of 2 at 0.5`. stage_east was rated 0.08/0.15 and described as "a walk back toward our side". The decode had stage_east at 0.97.
   - Cost: 10 Blitzes died at his base 9:00-9:16 (1,100 metal), and the push was pulled back at 9:08. The player's own flee-off lane change at 8:48 shares the blame.
   - Fix class: menu (words measured from the front of a strung-out body).
3. **9:08 · A5 · small.**
   - Wanted: the pullback to stage_east.
   - Happened: the pullback landed at 9:15, 7 s after the report. Five Blitzes died beside the Beamer 9:09-9:16.
   - Cost: 550 metal. This is part of item 2's cost, not in addition to it.
4. **6:01-10:47 · A4 · small (high confidence).**
   - Wanted: "They never stand in front of a plant's exit".
   - Happened: code put list steps and assists in exit lanes: a construction turret at 6:01, an advanced solar at 8:18, the same site again at 9:48, and the commander assisting plant_4709 at 10:10. The player stopped each one.
   - Cost: three frames decayed (6:53, 9:11, 10:14), about 140 metal, plus five yard wakes. The first construction turret was delayed until 8:48.
   - Fix class: code-rule (site choice should keep clear of the lane).
5. **4:45 · A6 · small (high confidence).**
   - Wanted: the 3:05 list's "armllt spot_122".
   - Happened: the 4:30 list replaced the old one whole and dropped the Sentry. The 4:30 entry showed only the extractor being built, not the queued step.
   - Cost: the Sentry stood at 9:31.
   - Fix class: report (show a builder's remaining queued steps).
6. **4:59 · A2 · small (high confidence).**
   - Wanted: `queue commander: ["armnanotc #nano1", "armnanotc #nano2", "assist"]`.
   - Happened: "skipped the step 'armnanotc #nano1' of its list: this builder cannot build it", and the 'assist' was dropped. Opus moved the turrets to a constructor at 5:11.
   - Fix class: tool (refuse at the tool).
7. **8:27-8:58 · A6 · small (low confidence).**
   - Wanted: 7:03 way "... spot_116, spot_107 ... never two turrets together".
   - Happened: "walk to spot_107" took the front past spot_104's Beamer and Sentry. The move's words named only its end.
   - Cost: 4 Blitzes, 440 metal.
   - Fix class: menu.
8. **8:48 · B6 · small (low confidence).**
   - Wanted (commander, 8:03): "kills what stands outside the turrets' reach ... A Rover looks at G5-H5 once more ... new Blitzes and Stouts walk to stage_east and join".
   - Happened: the player wrote "joins group_I wherever it is" and went straight for the lab.
   - Fix class: brief.
9. **6:37-10:57 · A3 · none (high confidence).**
   - Wanted: "group_G is the west picket. Its place is spot_87; it stands there". The decode read spot_87 at 0.97-0.98.
   - Happened: the pick kept "nothing changes" over "group_G walks to spot_87" about 20 times; at 7:37 it chose w1 at p 0.73. group_G followed group_C at spot_103 until the end.
   - Fix class: jev (the course in force is never re-read against a new packet).
10. **6:43-10:47 · B4 · none.**
    - Happened: 23 reports showed "group_G: ... at spot_103 (C5); following group_C", and Opus rewrote the spot_87 sentence six times without noticing.
    - A line that would surface it: "your packet places it at spot_87; it stands at spot_103, 2,500 away".
11. **8:02 · A3 · none (medium confidence).**
    - Wanted: the east picket "attacks [a raider at spot_96/99/102]".
    - Happened: the Tick at home was held from the menu until 8:00 (the party's answer was 0.47, under the openers' bar). At 8:02 the pick "kept w1 over w2 at 0.68" with send_1 at 0.5. One Blitz went after the Tick only at 8:25.
12. **9:21 · A3 · none (medium confidence).**
    - Wanted: 9:08 "Its place now is stage_east: it walks there, closes up and holds there".
    - Happened: the pick played "gather on its front" at F5 instead. This probably helped: an unseen Beamer had stood by stage_east since 8:34.
13. **6:22-6:28 · A4 · none (high confidence).**
    - Wanted: Ticks at our constructors "met by one or two Blitzes of group_C or group_G".
    - Happened: the pickets stood 3,000 east, and the 900 leash ended their hunts after 1 s. The Sentry at spot_80 killed the Tick at 6:30.
    - Fix class: code-rule.
14. **6:26-6:28 · A6 · none (high confidence).**
    - Wanted: group_I "single armflea it does not chase".
    - Happened: two detachments of 2 went after a Tick 4,200 behind the body. The forbidden mark read only 0.38-0.40.
    - Fix class: words.
15. **3:52 · A6 · none (high confidence).**
    - Wanted: "The Blitz group from the plant ... Its place is spot_103; it stands there".
    - Happened: group_B was born at 3:20, after the packet, and the packet named it only by description. Under decode.rs's `newer` rule nothing was read for it. The pick sent 2 to hunt a constructor 7,000 away, and the leash ended it.
    - Fix class: code-rule.

Not listed above (all cost none):
- **B2, 10:27-10:47:** his commander's health was in the record's `en` (2,306, then 2,084, then 456, then 86) and in no picture or report line.
- **B6 ×3:**
  - 4:52: the player kept group_C home on purpose; its reason was party_12 heading toward us.
  - 0:21: the anti-air turret "by about 7:00" was never built; the commander withdrew it at 8:03.
  - 9:44: "Shellshockers four to six" became 9.

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 43 turns, 1 session replacements, 97 orders, 0 refused, 22 flagged of 349 hands' lines; landing median 7 s (p90 7), turn wall median 8 s
   orders: {'wait': 43, 'queue': 22, 'instruct': 15, 'produce': 10, 'lane': 6, 'mark': 1}
   flagged: {'step-skipped': 7, 'hunt-ended-leash': 5, 'other-fault': 3, 'hunt-ended-other': 3, 'stopped-on-stop': 3, 'abandoned': 1}
-- commander.jsonl: 7 turns, 0 session replacements, 7 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 16 s
   orders: {'direct': 4, 'wait': 1, 'units': 1, 'mark': 1}
   flagged: {}
```

How the flagged lines were judged:
- **step-skipped:** the two 4:59 lines are item 6. The other five (3:31 spot_96, 8:47 spot_86, 9:42 spot_106, 10:04 spot_89, 10:07 spot_90) found the extractor already ours or being taken, so nothing was lost.
- **hunt-ended-leash:** 3:53 is item 15 (the leash enforced the packet). 5:42 was not a raider the packet wanted dead. 6:26 group_K is item 13. 6:26 and 6:28 group_L/M are item 14.
- **other-fault:** 4:45 is item 5, and 4:59 is part of item 6. 6:09 dropped spot_67, but the new list carried it.
- **abandoned:** the 10:14 frame is part of item 4.

## What Opus wanted that the harness has no lever for

- **8:48, also 9:08 and the 9:48 note.** "At stage_east it closes up on its front, then goes on at once, as one body." The gather ends only when the tail is up, and the tail counts newcomers walking from the plants 6,000 behind ("reinforcements stream so gathering never 'ends'"). No move launches the body that has gathered while holding the stream back.
- **10:37.** "Its Shellshockers shell it from their 710 reach while the Stouts and Blitzes close on it from all sides at once." The menu offers the whole-group attack (which feeds soldiers in as they arrive) or the shelling, one at a time. There is no converge-from-all-sides move.
- **2:55 to 10:37, every packet.** Constructors "never stand in front of a plant's exit" (10:37: "or build"). Code places list steps and assists, so the packet cannot steer the site; the yard watchdog only wakes the player after 30 s.
- **7:03, also 0:10.** "never two turrets together" / "never walk into his turrets" for a walk. The route is the engine's path, the move's words describe only its end, and no move routes around a turret's reach.

## Commands run (from the repository root; M = run/matches/1791907954-apex-v022-carrot-sym0-opus-1/00)

```
cat M/../results.jsonl; python3 -c "...json.load(open('M/../batch.json'))"
run/impedance_sweep.py M --counts; run/impedance_sweep.py M --seat 0; run/impedance_sweep.py M --commander
run/minutes.py M; run/wake_read.py M; run/raid_ledger.py M; run/pro_baseline.py M (no carded side on this map); run/floor.py M
run/commander_turns.py M --calls
run/hands_window.py M 3:45 3:58 | 6:14 6:34 | 7:10 7:30 | 7:36 8:05 group_C group_G group_N | 8:00 8:26 group_C commander
  | 8:05 8:15 | 8:25 8:30 group_I | 8:48 9:08 group_I | 9:08 9:58 group_I | 9:56 10:25 group_I | 10:25 10:57 group_I | 6:29 10:57 group_G
scratch scripts (the scratch directory's scripts/): packets.py (every packet and note in full), turn.py (one turn's prompt),
  decodes.py (decode rows), menu.py (an actor's moves and answers), groupstate.py (a group's picture), events.py (record events), truth.py
grep / python over record-0.jsonl (en health of u=765, enemy_destroyed), truth-0.jsonl (his turrets' first frames, his commander's health)
read: docs/code-decisions.md, docs/harness/record-format.md (decode, record `en`), crates/bot/src/brain/pianist/decode.rs (`newer`),
  crates/bot/src/brain/yards.rs (the exit-lane wake text)
```

There was one seat, so no symlink directories were needed.
