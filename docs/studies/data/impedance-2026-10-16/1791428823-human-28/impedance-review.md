# Impedance review: human-28 (1791428823-human-28/00)

Lost, a 15-minute realtime duel on Comet Catcher Remake 1.8 (2026-10-07). One seat of ours, strategist-0 "Jaedrik": an Opus 5.5 medium player under an Opus 5.5 commander, with hands Jev 1.13.0 and the layers news, same, fuse, openers, split, tick and places. The opponent was [gecko]u6bkep, a person of lobby skill 30, also Armada. We lost the Blitz mirror (4-8 min) and sat at 8-9 extractors from 5:00 to 12:00. His Stout block killed our plant at 12:12 and our commander conceded in the chat at 15:24. The record ends at 15:32.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 0 |
| A2 accepted, not executed | 2 |
| A3 overruled by the pick | 5 |
| A4 overruled by code | 1 |
| A5 late | 0 |
| A6 read differently | 4 |
| B1 in the hands' picture, not the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 report wrong or stale | 0 |
| B4 in the report, missed by Opus | 1 |
| B5 the hands' own lines misleading | 0 |
| B6 commander-to-player hop | 2 |
| C followed, with context enough | 2 |
| judgement moments examined | 8 |
| orders (player / commander) | 141 / 14 |
| hands' lines (player seat) | 440 (62 flagged) |
| refused | 0 |

The eight moments, with what each one contained:

- **(a) The opening, to 4:00.** Not clean: A3 1:44, B4 2:17, A6 2:35 and A4 3:21. The build side ran exactly as written. The commander built its 3 extractors 0:03-0:26, its 3 solars 0:33-0:52 and the plant 1:04-1:23. The produce order came out as listed: armcv, then 3 Rovers, then armcv, then 4 Blitzes, then armcv.
- **(b1) The raid at spot_16/spot_11, 5:24-5:49 (party_6, 7 Blitzes).** Not clean (A3 5:40). The whole group attacked at 5:24, as the 5:01 raid clause asked.
- **(b2) The raid at spot_18, then base G2, then G4, 6:58-7:36 (party_13).** Not clean (A3 7:05). group_F attacked with the whole group at 7:07.
- **(b3) The Stout raid at spot_3, 10:01.** **C.** group_K gathered at spot_11 at 9:48 and attacked party_22 with the whole group at 10:01, both as the 9:44 packet asked. The report carried the group's tail ("1317 behind") and its stuck Stout (1:19).
- **(c) The first push, ordered 7:59 and landed 8:14.** Not clean (A3 8:13). The 3:00-6:00 window at 1.5x was the trickling raid body (B6 3:40).
- **(d1) The 5:01 packet.** **C.** The decode read spot_18 at 0.98 and group_G's join at 0.78. group_F walked to spot_18 at 5:04 and took in group_G at 5:19. The packet's "beside the plant, not in front of it" has no lever (see that section).
- **(d2) The 10:37 packet.** Not clean (A6 10:27). group_K held at spot_11 by 10:49 (decode 0.97) and the commander went to the plant at 10:38, both as asked. The constructor clause was fused away.
- **(e) The last three turns, 15:14-15:31.** Not clean (B6 15:20). The player's picture matched the record: 3 extractors, 3 constructors, and the frame at 18% at 15:14 and 49% at 15:21.

The fixed context check was run at every moment. The "enemy in sight" lines, the "recently seen" lines and the group entries agreed with the record wherever they were checked. The "what your hands did" blocks at 7:20 and 8:18 matched the log's plays.

## Items, by cost

1. **12:10, strategist-0, A3, cost large, confidence medium, fix: code-rule.** The 12:06 packet said: "commander: attacks his army at home beside group_K, standing behind our Stouts, using its D-gun". Instead the log has "12:10 PLAYS commander: walk to spot_12" at 100% health. The commander had already walked home at 11:48 and stood 480-700 from the fight. The plant's health went 100 → 52 → 0 between 12:08 and 12:12, and no factory stood from 12:12 to 13:26.
   The decode landed: spot_6 at 0.95 and follow group_K at 0.61. Jev's picture said "against this unit alone, it outweighs us ... 11 of them outreach its D-gun", and the hands' rules text says the commander "never ... attacks a party that outreaches its D-gun or outweighs it". That rule beat the packet. Whether the D-gun would have saved the plant is a reading, not something the tools show.
2. **7:05, strategist-0, A3, cost small (530 metal), confidence medium, fix: code-rule.** The packet said: "commander: while his Blitzes stand at spot_18 ... or our home spots, it goes there and attacks them, using its D-gun". The commander kept walking home, from (6342,856) to (6744,743). Meanwhile 6 Blitzes killed 2 solars (7:09, 7:10) and 2 of our Blitzes (7:12, 7:14) 200-400 from where it had stood. The gate rated attack_party_13 at only 0.22-0.33, and the rule was the same as in item 1 ("against this unit alone, it outweighs us").
3. **10:27, strategist-0, A6, cost small, confidence high, fix: words.** The 10:21 and 10:37 packets said: "A constructor with no list retakes spot_3, spot_18, spot_29 ... with a light turret (armllt) beside each extractor". The decode read constructor_22980|build|armmex at 0.18 (0.27 at 10:36), so all 19 of its extractor moves were fused off from 10:27 to 10:56. The log has "10:28 PICK constructor_22980: help plant_2232 build" at 0.69, and it helped the plant until the player's own list landed at about 10:58. The commander complained at 10:50: "Four constructors are helping the plant while metal runs out". The same shape appears at 8:16 and 9:24, both without cost.
4. **12:24, strategist-0, A6, cost small, confidence high, fix: code-rule.** Three lists each had ["armvp spot_17 #vp2", "assist"], all played in the same second, and the one id made two plants: frame 19699 at 12:44 and frame 16631 at 12:46. The second frame reached about 52% before it was reclaimed. Plant 16631 stood at 13:26, about 15 s later than one frame would have, with the metal store empty from 13:10. This is the list-id same-tick race, fixed 2026-10-12.
5. **2:35, strategist-0, A6, cost small (172 metal), confidence medium, fix: jev.** The 0:11 raid order was "spot_21, then spot_24, then spot_22, then spot_19, then spot_28, then spot_36". The places layer read the first five stops at 0.10-0.27 and blanked their moves, leaving only his plant (spot_36 at 0.32). The log has "2:35 PLAYS group_A: advance to spot_36". The Rovers ran ahead onto B4 and died at 3:08 and 3:11, and a Blitz died at 3:18.
6. **8:13, strategist-0, A3, cost small, confidence high, fix: jev.** The 8:14 packet said "Otherwise it goes west as one body, fighting: spot_29, then spot_35 ...", and the decode read spot_29 at 0.97. The gate read group_F.change at 0.39, which held every push move. Then "8:14 group_F: 4 of group_F hunt party_20 (1 armstump) as group_K", and the news layer kept group_F quiet from 8:14 to 8:31. The player saw it at 8:18: "The army never went west. The hands kept it following the commander at home." It then dropped the push.
7. **1:44, strategist-0, A3, cost small (62 metal), confidence high, fix: jev.** The packet said: "the group of Rovers roves (scouting his start box first)". The menu rated rove at 0.60, but the picks were "kept w1 over w2 of 1 at 0.54" and "kept w1 over w3 of 2 at 0.18". Two Rovers held at spot_6 from 1:42 to 2:35, then rode in the raid and died at B4.
8. **5:40, strategist-0, A3, cost small (110 metal), confidence high, fix: code-rule.** The packet said: "his party at our extractors at spot_16, spot_18 or spot_11 ... it attacks them now with the whole group". The log has "5:40 PLAYS group_F: 4 of group_F hunt party_12 (1 armflash) as group_H", even though the gate read that detachment as forbidden at 0.69. The pick was sampled at 0.14 confidence, and the forbidden mark binds only at 0.8. group_H met 4 Blitzes at F1 and lost one.
9. **3:40, strategist-0, B6, cost small, confidence low, fix: brief.** The commander wrote at 3:20: "New ones join the body where it stands in his half and do not trickle across alone." The player's 3:40 produce sent every new Blitz singly to group_C, holding at spot_24. The player said "the plant's new Blitzes now go to the raid group". The body never reached four, and 3 Blitzes died between 3:57 and 4:17 for 1 of his. The player noted at 4:16 "trickle lost 4 for 2", and the commander's final note blames "trickling Blitzes in singly". The direction's two halves pull against each other unless the Blitzes are sent in batches.
10. **13:08, strategist-0, A6, cost small, confidence high, fix: tool.** constructor_29073 was given ["stop", "assist"]; the note said "one plant frame at spot_17 (two were started, one dropped)". The log has "13:08 constructor_29073: help plant_19699 build": "assist" helped the nearest factory, which was the frame it had just stopped. The player reclaimed that frame at 13:12.
11. **3:21, strategist-0, A4, cost none, confidence high, fix: code-rule.** The 3:00 packet (a 26 s wall turn) named "group_A (the raid)". Before it landed, a detachment had been sent "as group_C" (3:13) and then "group_A: join group_C" (3:15). The decode read group_C's every place under 0.3, so 42 of its 47 place moves were fused until the 3:36 packet.
12. **2:17, strategist-0, B4, cost none (the loss is on item 7), confidence high, fix: report.** The report said "group_A: ... 2 Rover (armfav) ... at spot_6 (G2); standing with no order for 38 s", but the orders did not touch the Rovers. A line that sets the state against the packet ("2 Rovers your packet says rove have stood 38 s") would surface it.
13. **13:59, strategist-0, A2, cost none, confidence high, fix: code-rule.** The log has "the engine refused a armsolar at H4: the site was bad" during an energy stall (0/1500). A solar frame stood 9 s later, at 14:08.
14. **14:07, strategist-0, A2, cost none, confidence high, fix: code-rule.** The log has "the engine refused a armllt at G5: the site was bad", for the list's turret at spot_48. The turret frame stood at 14:10.
15. **15:20, commander, B6, cost none, confidence low, fix: report.** The commander conceded on "No factory (a frame at 17%)". Its own report said only "labs 0"; the 17% was the player's note from 15:14. The seat's 15:21 report said "(49 % built)", and the record has the frame at 42% at 15:20 and 58% at 15:24. The player's 15:14 note said "not conceding while the commander lives". The player was never told of the surrender, and its 15:21 packet landed at 15:28, after it. With 31 metal of army against 5,110, the concession would very likely have stood.

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 63 turns, 1 session replacements, 141 orders, 0 refused, 62 flagged of 440 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 62, 'instruct': 34, 'queue': 27, 'produce': 15, 'lane': 1, 'remove': 1, 'mark': 1}
   flagged: {'step-waits': 34, 'step-skipped': 6, 'stopped-on-stop': 6, 'hunt-ended-other': 3, 'abandoned': 3, 'engine-refused': 3, 'other-fault': 3, 'hunt-ended-leash': 2, 'follow-ended': 2}
-- commander.jsonl: 11 turns, 0 session replacements, 14 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 16 s
   orders: {'direct': 8, 'wait': 2, 'mark': 2, 'say': 1, 'surrender': 1}
   flagged: {}
```

How the flagged lines were judged:

- **engine-refused:** 13:59 and 14:07 are items 13 and 14. The 9:52 commander solar was refused without a stall (energy 1131/1950) and stood 10 s later, so it is not an item.
- **step-skipped (6):** every one says "holds our extractor already".
- **abandoned (3):** the 7:36 extractor and the 14:40 and 15:02 turrets were the player's lists being replaced, or builders fleeing under its own flee clause.
- **hunt-ended-leash (2):** at 3:15 and 4:54 no packet wanted that raider dead. The 4:28 packet kept the body home.
- **hunt-ended-other (3):** the quarry did die at 5:44 and 6:22. At 4:43 one of party_6's three lived, but nothing was decided on that line.
- **other-fault (3):** 13:54 is "the party is gone". 14:47 and 14:49 are the player's own list replacements, and the plant stood at 15:02.
- **follow-ended (2):** 8:50 and 14:17 are joins.

## What Opus wanted that the harness has no lever for

- **5:01, strategist-0.** "commander: ... helping the plant ..., standing beside the plant, not in front of it." The player repeated it at 5:36, 10:21 and 10:37, and at 11:48 wrote "never in the plant's exit lane". help_plant is one move and code chooses where the helper stands. At 5:56 the bot reported "plant_2232's exit lane is blocked: ... (armcom_10999) have stood in it for 0:30". At 11:50 the commander stood west of the west-facing plant.
- **12:06, strategist-0 (also 7:05 and 12:15).** "commander: attacks his army at home beside group_K, standing behind our Stouts, using its D-gun". The hands' fixed rules say the commander never attacks a party that outweighs or outreaches it, and its picture weighs it alone, not beside the group. The rules give the group an exception when the instructions speak to this very fight; the commander has none, so no packet wording gets it into that fight (items 1 and 2).
- **3:20, commander.** "A Rover keeps looking at his base now and then for an air plant." Roving runs in code ("beyond your hands' reach") and picks the least-known places itself. Nothing tells a roving Rover to revisit a known place on a schedule. The player never passed the sentence on.

## Commands run

All were run from /home/christian/workspace/playground/bar_bots with MATCH=run/matches/1791428823-human-28/00.

```
grep -n 'name=' MATCH/script.txt; tail -1 MATCH/record-0.jsonl; grep '"k":"chat"' MATCH/record-0.jsonl
run/impedance_sweep.py MATCH --counts | --seat 0 | --commander | --seat 0 --all --from 12:18 --to 13:13 | --from 14:30 --to 15:32 | --from 13:51 --to 14:18
run/minutes.py MATCH;  run/raid_ledger.py MATCH;  run/pro_baseline.py MATCH [--all-starts];  run/layers_read.py MATCH --faults
run/commander_turns.py MATCH --calls
run/hands_window.py MATCH 1:34 2:10 | 1:50 3:25 | 3:05 3:40 | 3:52 4:30 | 4:45 5:22 | 5:15 5:40 | 5:20 5:50 | 6:56 7:40 | 7:04 7:16 | 7:17 7:36 | 8:10 8:58 | 9:42 10:06 | 10:15 10:59 | 11:50 12:16 | 11:48 12:12 (with actors)
python3 scripts (scratch bin/): turns.py (prompts, calls), menu.py (menus with gate flags), mexfused.py (fused extractor moves),
  deaths.py (record deaths by name), jevlog reads of decode rows, packet changes, place flags, the header's rules, call-row pictures,
  record samples (commander and plant positions/health, frames' health, energy)
```

No seat needed symlinks (one seat, seat 0). Nothing was unreadable. The hands' log has one error row (10443: "the answer to the request of frame 10350 has not come in 3 s; dropped").
