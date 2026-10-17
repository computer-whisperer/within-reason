# Impedance review: 1791593626-human-29 (2026-10-09, Comet Catcher Remake 1.8)

This was a realtime 1v1 against a person, [gecko]u6bkep (lobby skill 30, Armada). We had one seat, strategist-0 "Rysica" (an Opus 5.5 medium player with Jev 1.13.0 as its hands), under an Opus 5.5 commander. **We won at 18:03.** The record shows his commander destroyed at B4 at 18:03, and every unit of his destroyed between 18:03 and 18:07. Seventeen of those units were never seen, and some stood far from our army. The person wrote nothing in the chat. The record ends at 18:14 with 165 of our units alive. The win came from an economy lead (22 extractors at 10:00 against his 5 seen) and one late body of 41 soldiers (7.8k metal).

## Tally

| Category | Count |
|---|---|
| A1 refused at the tool | 2 |
| A2 accepted, not executed | 2 |
| A3 overruled by the pick | 6 |
| A4 overruled by code | 1 |
| A5 late | 1 |
| A6 read differently | 4 |
| B1 in the picture, not the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 report wrong or stale | 1 |
| B4 in the report, missed | 0 |
| B5 the hands' lines misleading | 0 |
| B6 commander-to-player hop | 2 |
| **C, clean moments** | **2 of 8** |

Denominators:
- **strategist-0:** 68 turns (1 session takeover at 11:58), 159 orders, 1 refused per `--counts` (2 per `--all`, see below), and 631 hands' lines, 44 of them flagged.
- **commander:** 12 turns, 13 orders, 0 refused.

The eight moments examined, and what each found:

- **(a) Opening:** A6.
- **(b) Raids:**
  - 4:16 at spot_25: C.
  - 10:07 at spot_6, bombers: A6, A3, and A4.
  - 12:36 at F5: A3.
  - 15:04 at E4: C.
- **(c) The 3:00-4:00 window and the first raid orders:** A6.
- **(d) Packet changes:**
  - 6:25/6:38: A3.
  - 12:09: A3.

The commander's 9:20 claim, "We have no anti-air; it has been asked for since 0:20 and was never built", is **false**. A Nettle finished at the home plant at 7:53 and a second at 9:32, both from the player's own lists ("armrl home", queued 6:38 and 5:40). The seat's report showed them being built. The commander's report counts only "turrets 17". So this is B6, not A2 or A6, and not the player's omission.

## Items, ranked by cost

The fifteen most costly of the 19 items are below. All 19 are in impedance.json.

1. **6:28-7:07, strategist-0, A3, large.**
   - **Wanted (6:25 packet):** "group_D attacks those armflash with the whole group now"; while under ten, it "stands at spot_18 and fights his raiders on our strip".
   - **Happened:** three detachments (6:28, 6:32, 6:39). The last was "group_G: 4 of group_G hunt party_19 (2 armflash) as group_H". group_H, now attacking "with the whole group" and so without the hunt's 900 leash, chased from E3 to C3. There all 4 Blitzes died between 7:00 and 7:07: 440 metal lost against 110 of his.
   - **Evidence:** in the Jev log at f=11997, send_4_party_19 rated 0.54 and its forbidden mark **0.66**. The forbid read the packet and the pick crossed it.
   - **Fix:** menu. A move whose forbid mark reads at 0.5 or more should not be offered, and a detachment's attack should not shed the leash.
2. **16:23-16:51, strategist-0, A3, small.**
   - **Wanted (15:58 packet):** "attacks it now ... until it is dead" and "Against a party that outweighs the whole group, it walks to spot_47".
   - **Happened:** "16:23 group_P: advance to spot_47", then "16:26 walk to spot_47". The body walked about 600 back east from his base.
   - **Evidence:** the report said the whole group "matches it (1.0 to 1)". The decode read spot_47 at 0.87 and back at 0.75, so the mark landed.
   - **Fix:** jev. The player's note "the hands walked it back" checks out.
3. **16:55-17:13, strategist-0, A3, small.**
   - **Wanted (16:27 packet):** "When go_b4 is given, it attacks his buildings at spot_36 with the whole group at once", and aircraft including the armca "are never chased by any group".
   - **Happened:** after go_b4 at 16:53, the pick alternated "shell party_45 (1 armca) from their reach with 10 long-reach soldiers" (five times) with "advance to spot_36". The body did not march until 17:19, costing about 25 s.
   - **Evidence:** shell_party_45 rated 0.56. The same packet also says "Whistlers (armsam) answer them".
   - **Fix:** menu. Shell moves carry no forbid mark.
4. **13:41-14:15, strategist-0, A6, small.**
   - **Wanted (13:16 packet):** group_P "comes back to spot_18; against a party that outweighs it, it walks to home".
   - **Happened:** "13:41 group_P: walk to home". Its 13 soldiers (2,045 metal) walked about 2,100 home while group_G fought his army at C4.
   - **Evidence:** the actor entry leads with "for the 1 of its 8 soldiers near it, it outweighs us" while the whole group "outweighs it narrowly (1.2 to 1)". So "outweighs it" was read on the local comparison.
   - **Fix:** report.
   - **Checks:**
     - The commander flagged it at 13:50.
     - The player's note at 14:13, "group_P ignored 'fight to spot_21'", is half right. That packet reached the hands at 14:09 and the pick turned group_P at 14:15.
5. **12:23-12:29, strategist-0, A3, small.**
   - **Wanted (12:09 packet):** "group_G ... attacks the armstump party ... now with the whole group and keeps attacking until it is dead".
   - **Happened:** the pick sent "4 of group_G hunt party_41 (1 unidentified) as group_Q", then "8 of group_G hunt party_41 (1 armjanus) as group_R".
   - **Evidence:** the forbid mark on send_8_party_41 read 0.66-0.67. The Stouts walked on into our half, and spot_40, spot_41 and spot_35 were lost between 12:36 and 12:53.
   - **Fix:** menu.
6. **12:36-12:57, strategist-0, A3, small.**
   - **Wanted (12:25 packet):** "group_P fights to spot_32 now with the whole group".
   - **Happened:** group_P (1,950 metal) kept "task hold" for 21 s during the F5 raid.
   - **Evidence:** the actor entry read "holding here on your pick for 44 s; best_rated: the advance to spot_32 (0.48)". The decode read spot_32 at 0.97, so the mark landed and the rating sat just under the bar.
   - **Fix:** jev.
7. **16:27-16:51, strategist-0, A5, small.**
   - **Wanted:** "group_P does not walk back east. It holds at spot_43".
   - **Happened:** the turn took 24.3 s of wall time, which in realtime is game time. The packet landed at 16:51, after the body had walked about 600 east. Opus at 16:51: "it never got to spot_43".
   - **Fix:** unknown.
8. **1:40-4:04, strategist-0, A6, small.**
   - **Wanted:** at 0:31, "the Rovers (armfav) rove ... The Blitzes ... raid his half along spot_13, spot_24, ...". At 2:29, "It stays in his half."
   - **Happened:** group_A roved as a whole: the pick chose rove when the group was one Rover, and every Blitz that joined roved with it. Units were back in our half at 2:39, 2:53, 3:18 and 4:03 (minutes.py).
   - **Evidence:** commander, 3:20: "Our Blitzes were set to rove and are scattered through our own half."
   - **Fix:** menu.
9. **2:29-3:45, strategist-0, A6, small.**
   - **Wanted:** `produce {"plant_20622": {"group": "new"}}`, which changes only the group.
   - **Happened:** the result said only "new soldiers form a group of their own". The plant lost the counted "all" list: the actor entry at 2:49 has no "allowed" clause, at 3:04 it reads "idle: building nothing", and the builds from 3:08 to 3:56 are "(plan)". Nothing started between 2:52 and 3:08.
   - **Evidence:** in mcp.rs, a produce without units carries the plant's own earlier entry, and this plant had none.
   - **Fix:** tool.
10. **10:47-18:03, strategist-0, A2, small.**
    - **Wanted:** the list step "extractor spot_6".
    - **Happened:** the extractor was never rebuilt. The record has no_site at 10:59, then "the site is blocked" at 13:13 and "the site was bad" at 13:16. The raid ledger has "rebuilt no", so the extractor was missing for about 7 minutes.
    - **Cause:** not settled. Our own Nettles, ordered as "armrl spot_6", stood about 110 from the spot.
    - **Fix:** unknown.
11. **10:21, strategist-0, A3, none.**
    - **Wanted:** "Aircraft (armthund bombers, armpeep scouts) are never chased by any group".
    - **Happened:** "group_J: 2 of group_J hunt party_30 (1 armthund) as group_L". send_2 rated 0.55 with its forbid at 0.65, and the leash ended the hunt after 0 s.
    - **Fix:** menu.
12. **16:27, strategist-0, A1, none.**
    - **Wanted:** signal `{"go_b4": "17:15"}` inside an orders call.
    - **Happened:** "signal: not a tool that can be batched". Opus noticed ("the orders call rejected the go_b4 signal") and gave it on its own at 16:53.
    - **Fix:** tool.
13. **9:20, commander, B6, none.**
    - **Wanted (the commander's belief):** "We have no anti-air; it has been asked for since 0:20 and was never built". At 7:50: "still owed".
    - **Fact:** the Nettles finished at 7:53 and 9:32. The seat's report said "building a Nettle (armrl) at home (G1)". The commander's report has only "turrets 17".
    - **Fix:** report. The commander needs defences by type.
14. **18:06, strategist-0, B3, none.**
    - **The report said:** "its commander was last seen at B4 ... 0:04 ago; its factories standing ...: Vehicle Plant armvp ... armvp" and "killed armcom x1".
    - **The record:** his commander died at 18:03 and his whole side was gone by 18:07. Neither armvp had been in sight since 11:43 and 15:47, though our body stood at B4 from 17:29.
    - **Opus believed:** "the commander seen may be another/decoy".
    - **Fix:** report.
15. **10:27, strategist-0, A4, none.**
    - **Happened:** "group_M's hunt of party_31 ended after 0 s: the leash of 900 from where it began reached". party_31 was the Stout at spot_37, a ground raider the 9:58 packet gave to the home group.
    - **Why no cost:** the extractor had died at 10:26.
    - **Fix:** code-rule.

Four more items in impedance.json cost nothing:

- **4:04, A1:** a produce to group_B, which had died. Opus knew and re-issued it.
- **4:56-5:36, A2:** three turrets skipped at spot_18, spot_16 and spot_15. The skip line said "the extractor ... was refused lately", but our own extractor stood there: the refusal was a sibling constructor racing for the same spot.
- **10:07, A6:** group_J and group_K walked home under bombers on the packet's fall-back clause.
- **9:20-17:34, B6:** the commander twice sent the Rover to look for the air plant at A1-A3. Every player packet kept "group_C: roves".

A pattern runs through the A3 items. The forbid mark on detachments read between 0.61 and 0.79 in every case, and the pick still played the detachment at 6:28, 6:32, 6:39, 10:21, 12:23 and 12:29.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 68 turns, 1 session replacements, 159 orders, 1 refused, 44 flagged of 631 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 68, 'instruct': 35, 'queue': 27, 'produce': 22, 'lane': 5, 'signal': 2}
   refused: {'produce': 1}
   flagged: {'step-skipped': 17, 'engine-refused': 7, 'step-waits': 6, 'hunt-ended-other': 6, 'hunt-ended-leash': 4, 'other-fault': 2, 'follow-ended': 1, 'site-blocked': 1}
-- commander.jsonl: 12 turns, 0 session replacements, 13 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 17 s
   orders: {'direct': 10, 'say': 1, 'mark': 1, 'signal': 1}
   flagged: {}
```

With `--all`, the same seat reports `refused: {'produce': 1, 'signal': 1}`. The extra one is the batched signal at 16:27.

There was also a lane call at 10:21 whose rules were written as a string. It was rejected with "group_J: "raw", "on" or a list of rules to keep", re-sent correctly in the same turn, and is not counted.

How the flagged lines were judged:

- **engine-refused (7):**
  - **4:56, 5:12, 5:34:** a sibling constructor took the spot first.
  - **10:56:** a Nettle placed elsewhere.
  - **13:16:** the spot_6 extractor, item 10.
  - **17:05 and 17:32:** a sibling took spot_41 and spot_51.
- **site-blocked (1):** 13:13, the spot_6 extractor (item 10).
- **hunt-ended-leash (4):**
  - **10:21:** a bomber, which the packet forbade chasing.
  - **10:27:** the A4 item.
  - **16:38 (two):** a radar dot that turned out to be the armca, hunted while still "unidentified", so not counted.
- **follow-ended (1):** 11:37. group_J had joined group_G, and group_N joined group_G one pick later at no cost.
- **other-fault (2):**
  - **4:26:** the commander's attack ended when the party left.
  - **16:46:** three converters dropped when Opus replaced constructor_14761's list to move it out of an exit lane. Nothing shows that Opus meant to keep them, so it is not counted.

## What Opus wanted that the harness has no lever for

- **0:31, player:** "the Rovers (armfav) rove, his box first ... The Blitzes ... raid his half along spot_13, spot_24, ...".
  - A group has one task, so the Rover's rove covered every Blitz that joined it.
  - There is no per-type split inside a group, and rove takes no region ("his box first").
- **2:29, player:** "it roves; leave it to rove. It stays in his half."
  - Rove has no region bound. Its soldiers went back into our half from 2:39 to 4:03.
- **9:58 onward in every packet, player:** "Aircraft (armthund bombers, armpeep scouts, armca) are never chased by any group".
  - The forbid mark is asked only of detachments (send_N), and the pick crosses it (0.65 at 10:21).
  - Shell and attack moves at an aircraft party carry no mark (16:55, shell party_45, the armca).
- **14:56 onward in every packet, player:** "Constructors never stand in front of a plant."
  - No decode mark or menu move keeps a builder out of a factory's exit lane.
  - The exit-lane wake fired at 13:00 and 15:13, and at 16:27 Opus found constructor_14761 standing in the lane again.
- **11:39 and 16:27, player (uncertain):** "Stouts in front, Whistlers behind" and "the Whistlers (armsam) stand back at their reach".
  - Menu moves are per group. The lane's form rule may order a group's soldiers, but no move or mark places one unit type behind another.

## Commands run

One seat, so no symlink directories were needed. MATCH = run/matches/1791593626-human-29/00.

- **Orientation:**
  - `grep -n 'name=' MATCH/script.txt; tail -1 MATCH/record-0.jsonl`
  - `grep '"k":"chat"' MATCH/record-0.jsonl`
- **Sweep:**
  - `run/impedance_sweep.py MATCH --counts`
  - `run/impedance_sweep.py MATCH --seat 0`
  - `run/impedance_sweep.py MATCH --commander`
  - `run/impedance_sweep.py MATCH --seat 0 --all --from ... --to ...` for 1:20-4:05, 4:05-7:00, 4:40-5:45, 10:40-18:10 and 16:40-16:50.
- **Review tools:** `run/minutes.py MATCH`, `run/raid_ledger.py MATCH`, `run/pro_baseline.py MATCH` (no carded side for this start).
- **hands_window:** `run/hands_window.py MATCH <from> <to> [actors]` for these windows:
  - 1:35-2:50
  - 3:55-5:05
  - 4:12-4:30
  - 6:20-6:39 and 6:38-7:40
  - 10:05-10:35
  - 10:58-11:20 and 11:30-11:50
  - 11:58-12:40, 12:30-13:05 and 12:54-13:20
  - 13:20-13:50, 13:40-13:50 and 13:50-14:20
  - 14:09-14:40
  - 14:50-15:20
  - 16:00-16:52, 16:29-16:40, 16:52-17:06 and 17:06-17:45
- **Python reads (transcripts and record):**
  - The commander's and player's transcripts: every call with its result, the turn prompts, and the player's text at 2:49-3:45, 4:04, 4:23, 10:21 and 16:27-17:02.
  - The record: unit_defs by index, enemy_destroyed at the end, his plants in sight, armrl/armmex/armllt/armmakr events by position, and constructor_3766's samples.
- **Python reads (Jev log):**
  - A one-pass index of jev-0.jsonl by row kind and frame, with every decode row (185).
  - Call, plan and decode rows at the frames named in the items.
  - The produce handler in crates/bot/src/strategist/mcp.rs.
