# Impedance review: 1791511650-rove-task-2/00

Arena duel, 2026-10-08, Comet Catcher Remake 1.8, our start B8 (Armada). **Won 21:11** against BARb hard_aggressive (Cortex).
One seat (strategist-0). The player was Opus 5.5 at medium effort (`claude-opus-5-5`), with no commander. Hands' layers:
news, same, fuse, openers, split, tick, places, again, near. Think penalty median 7 s. There were 2 session replacements
(9:50, 15:52), and neither delayed a landing. A truth file exists.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 1 |
| A2 accepted, not executed | 3 |
| A3 overruled by the pick | 6 |
| A4 overruled by code | 5 |
| A5 late | 0 |
| A6 read differently | 3 |
| B1 in the hands' picture, not the report | 0 |
| B2 in record/truth, in nobody's picture | 0 |
| B3 report wrong/stale | 0 |
| B4 in the report, missed (Opus's miss) | 2 |
| B5 hands' own lines misleading | 1 |
| B6 commander-to-player hop | 0 (no commander) |
| **C** (clean judgement moments) | **2** |

The four denominators:
- **Judgement moments examined:** 8. These were (a) the opening; four raids from (b): 2:39 spot_69, 3:53 spot_62, 12:32-12:50 spot_49/44 and 15:41-15:53 spot_49/44; (c) the 9:40 push; and two (d) packet changes, 7:17 and 14:38. The clean ones were the 3:53 raid and the 14:38 packet.
- **Orders:** 236.
- **Hands' lines:** 1116, of which 64 were flagged.
- **Refused:** 1.

## Items ranked by cost (15 of 21; all 21 are in impedance.json)

1. **13:30-17:36, A4, large, medium confidence, code-rule.**
   - Wanted (packets 13:51-17:19): "Without a list, the others take free spots on our side in this order: spot_33, spot_49...".
   - Happened: the constructors without a list helped the plants for about 4 min. The `near` layer shut their extractor moves (constructor_18284 and constructor_25985: every `build_armmex_spot_*` had far=True and fused=True), and the `news` layer kept closing the helpers (constructor_26737 was "help quiet" while spot_49/37/31 stood open to it).
   - Measured: extractors flat at 33 for 4 min with 30 free spots. The metal store held 0-158 and energy stayed full from 13:30 to 18:00 (record). After the player's own lists at 17:36, extractors went from 34 to 39 by 20:00.
   - Fix: the near/news layers should yield to a packet sentence about list-less builders.
2. **15:50-15:53, A3, large, medium confidence, code-rule.**
   - Wanted: at 15:32, "It never splits and never chases single raiders"; at 15:42, "group_C goes there now as one body and attacks his commander with every soldier".
   - Happened: four PICKs in a row, "8 of group_C hunt party_52 (2 corgator)" as A3, B3, C3 and D3. group_C went from 53 to 18 soldiers while his commander stood at spot_35. The `.forbidden` gate read 0.85, then 0.69, 0.69, 0.31 and 0.44, and the 15:50 decode read detach|8 at 0.59.
   - Measured: the strike paused until about 16:40. His commander (truth) was at (6100,2795) at 15:40 and walked home to (7716,476) by 17:20; it was not in our sight again until 20:57.
   - Fix: a forbidden reading of 0.5 or more should shut the detachment, not only be phrased at 0.7.
3. **13:30-17:36, B4, large (same window as item 1), medium confidence, report.**
   - Every report carried the plateau, for example at 15:42: "no new high for 2:10; free spots we can walk to 30", "metal 117 (+117.0/-49.5)", and constructors "helping lab_27255 build, since 208 s ago".
   - Opus's notes were all about the army until the 17:36 no-growth wake ("five constructors were guarding plant_27255 with metal at 0").
   - Line that would surface it: an eco line such as "N constructors without a list are helping plants while the metal store is empty; the hands take far spots only from a list".
4. **0:35-1:57, A6, small, medium confidence, code-rule (fixed since by 3aabc77).**
   - Wanted: a bare `armvp` and bare `extractor` steps.
   - Happened: the plant faced east with spot_73 in its exit lane, and the bare extractor put armmex_28188 there.
   - Cost: lane alarms at 2:30, 4:24 and 7:35; the player reclaimed the extractor, then self-destructed it at 7:55.
5. **2:48, A3, small, high confidence, jev.**
   - Wanted (2:40): "group_A stops roving and fights it there with the whole group". The decode read group_A|place|home at 0.67.
   - Happened: the menu for group_A [rove] read change=0.76 but fight_to_home=0.29, so group_A kept roving while a corfav shot constructor_10476.
6. **3:12-3:28, A4, small, high confidence, code-rule.**
   - Wanted (2:50): "group_A fights to home ... and attacks it".
   - Happened: party_2 came back, and the pass rows show closed=['group_A'] (news) for 17 s.
   - Cost: the armllt frame at spot_69 was destroyed at 3:28, and constructor_10476 walked home again. group_A only turned home at 3:30, from about 3,700 away.
7. **7:24-8:13, A3, small, high confidence, jev.**
   - Wanted (7:17): "group_G ... stands at spot_73". The decode read 0.97.
   - Happened: group_G kept following the commander inside plant_27255's exit lane, with one soldier stuck there; go_spot_73 rated 0.35-0.40. It moved only on the 8:04 packet.
8. **7:52-8:58, A6, small, medium confidence, tool.**
   - Wanted: the new list `["armadvsol", "extractor spot_28", ...]` for constructor_31791.
   - Happened: the constructor was already walking to spot_28. The bare generator went to the back field at home, about 3,300 away (as the queue tool's text says), and was started at 8:58.
   - Cost: spot_28 was taken only at 11:23. Fix: the queue result could state where a bare step will stand.
9. **8:04, B5, small, medium confidence, report.**
   - The lane wake said "in the lane: armmex_28188" with no spot, and the player's 2:30 note had put it at "spot_67".
   - After destroying it, the player queued "extractor spot_73" (8:04). armmex_15605 was rebuilt in the same lane at 8:25, and the alarms came back at 13:51, 19:14 and 20:27. Fix: name the spot in the lane line.
10. **11:19-11:20, A3, small, high confidence, code-rule.**
    - Wanted (10:36): "a single raider ... is answered by sending two soldiers from the home guard".
    - Happened: the pick sent two pairs out of group_C (the army) after one corgator about 2,500 behind them. The decode read detach|2 at 0.76 and the `.forbidden` gate read 0.69, then 0.50.
    - The player's 11:23 packet said "group_C never sends any...".
11. **11:29, A4, small, high confidence, code-rule.**
    - Happened: the home guard's pair (group_A1) on party_25 was ended by the leash and then walked to spot_69, while the corgator stood at spot_33.
    - Cost: spot_33's extractor was lost at 11:29 (rebuilt 12:41), and none of the raiders paid.
12. **12:16-12:35, A3, small, high confidence, jev.**
    - Wanted (12:09): "group_A1 fights to spot_58, then spot_57, then joins group_C". The decode read 0.98.
    - Happened: the openers layer held those moves on change=0.45, then news closed the actor from 12:18 to 12:27. group_A1 (11 soldiers, 2,360 metal) held at spot_69 for 19 s, and the player re-ordered it at 12:29.
13. **15:14-21:11, B4, small, high confidence, report.**
    - constructor_10476 was stuck beside the solar_field mark (a cliff pocket at B7), and every report from 15:14 on said "stuck: it cannot move from solar_field (B7)" (178 move_failed). It was never re-tasked.
    - Line that would surface it: a wake or eco-line count of stuck builders.
14. **15:27, A2, small, medium confidence, code-rule.**
    - Happened: "skipped the step 'armllt spot_44' ...: the extractor at spot_44 was refused lately", while our extractor (rebuilt 14:39) stood there. The refusal was of a duplicate step. spot_44 was lost again at 15:51.
15. **9:58-10:19, A6, none, medium confidence, words.**
    - Wanted (8:51): "When the home guard holds six or more soldiers and push is given, it walks to spot_58". The decode read group_P|place|spot_58 at 0.06.
    - Happened: group_P (6) gathered at home until the 10:12 packet named it.

Six more items, all of cost none, are in the JSON:
- A4 8:44: the leash ended group_M's hunt, re-sent 1 s later.
- A4 12:49: the leash ended group_L1's and group_M1's hunts, re-sent 1 s later.
- A1 16:40: `mark {"name":"his_g3","value":"G3"}` was REFUSED ("name: a grid cell is A1 to H8") and re-issued correctly at 16:50.
- A3 17:26-18:01: group_L2 followed group_B3 instead of joining it (decode 0.64; join rated 0.27-0.34).
- A2 18:32: the commander cannot build armnanotc. The tool accepted the two steps and the hands skipped them.
- A2 6:07: the turret at spot_71 was skipped because of a duplicate's refusal, beside our standing extractor.

Moments with no item:
- **3:53 raid (spot_62):** the clause in force named only corfav/corak, and the commander's D-gun was rated 0.52 under a busy gate of 0.47.
- **14:38 packet:** the marks landed at 0.97, and the army attacked a corwolv, then advanced to spot_23 and gathered for its tail as written.
- **The 9:40 push body:** group_C, with group_G joined at 9:22, left whole at 9:45. Only the home-guard clause (item 15) failed.

Report against truth at every moment:
- His army estimate ("seen in the last 3 min") was 56-95% of truth, and always labelled a floor. At the 8:31 push order it read 1,425 against a truth of 2,295.
- Our state lines (lane blocks, stuck units, the empty metal store) were carried.
- The hands' blocks matched the plan plays in the windows checked (2:40-2:50, 7:17-7:27, 12:39-12:49, 15:42-15:52).

Looked at, not counted (cause not settled by the tools):
- **18:40-19:40:** group_L2 stalled short of his_g3, then his_avp. The front reached his_g3 at 18:44, but the tail stayed about 1,000 behind at F4 turrets. Opus gave three different causes (march, the mark on cliffs, the G3 cliffs).
- **12:19:** Opus's claim that flee footwork held group_C inside turret reach.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 112 turns, 2 session replacements, 236 orders, 1 refused, 64 flagged of 1116 hands' lines; landing median 7 s (p90 7), turn wall median 8 s
   orders: {'wait': 112, 'instruct': 48, 'queue': 34, 'produce': 15, 'signal': 10, 'lane': 7, 'mark': 7, 'remove': 2, 'say': 1}
   refused: {'mark': 1}
   flagged: {'step-waits': 15, 'step-skipped': 15, 'hunt-ended-leash': 12, 'hunt-ended-other': 8, 'engine-refused': 4, 'site-blocked': 3, 'other-fault': 2, 'abandoned': 2, 'follow-ended': 2, 'stopped-on-stop': 1}
```

How the flagged lines were judged:
- **Engine-refused:** 3 of the 4 were Opus's own duplicate extractor steps on spots we already held. They led to the two A2 turret skips. The fourth (12:20) was a nano re-placed and built at 12:40.
- **Site-blocked and abandoned:** all were re-placed and built within 15 s, except the abandoned C3 frame at 15:02, whose builder walked home from a raid as the packet asked. None are items.
- **Leash (12 lines):** 7 of them fall in three A4 items: 8:44 (one line), 11:29 (four lines; two of them were group_C's unwanted pairs) and 12:49 (two lines). The other five are not items:
  - 4:03: the quarry was already out of sight.
  - 16:02: B3, which Opus did not want hunting.
  - 19:00-19:17 (three lines): hunts after one corcv or corgator, which no clause covered.
- **Other-fault:** both were dropped steps from lists Opus itself replaced.

## What Opus wanted that the harness has no lever for

- **12:09, then in every packet to 20:47.** Opus wrote for the commander: "helps the plants from beside them, never standing in a plant's exit lane", and for constructors: "Never build in a plant's exit lane."
  - No menu move or list step says where a helping builder stands, and code picks a bare building's site.
  - The lane alarms continued: armcv_21956 at 13:51, and armjanus_14107 at 19:14 and 20:27.
- **7:17, and 14:38-15:22.** Opus wrote: "A corwolv alone or at the back of his party is the first thing to kill." and "It kills his soldiers first, then his turrets ..., extractors and constructors".
  - A group's attack moves take a whole party, or his buildings at a place. No move, lane setting or tool sets the micro engine's kill order by unit type inside a mixed party.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
run/impedance_sweep.py MATCH --counts;  run/impedance_sweep.py MATCH --seat 0
run/minutes.py MATCH;  run/wake_read.py MATCH;  run/raid_ledger.py MATCH
run/pro_baseline.py MATCH  (no carded side from B8) ; run/pro_baseline.py MATCH --all-starts
run/commander_turns.py MATCH --calls
run/hands_window.py MATCH 2:30 3:35 group_A commander | 3:45 4:20 | 7:22 8:52 | 8:30 8:52 | 9:20 10:40 | 11:17 11:36 | 12:14 12:32 | 12:30 12:58 | 14:43 15:30 | 15:36 16:06 | 17:26 17:40 | 18:40 19:45
scratch scripts (scratch dir tools/): packets.py (full call text), events.py (record events), jevindex.py + decode.py (decode rows, packet changes, pass closed/asked, worlds_gate flags), builders.py (builder menus' far/fused extractor moves), inline jevlog reads of pass menus at 12:16-12:18 and 16:25-16:50
inline: report-vs-truth army metal at 11 turns; record metal/energy store 12:00-19:00; move_failed counts; corcom truth track; stuck/exit-lane lines across all prompts
```

Every seat was read; no symlink directories were needed (one seat). Nothing unreadable.
