# Impedance review: 1791427881-human-27

Human game, 2026-10-07, Comet Catcher Remake 1.8, realtime: one seat of ours (strategist-0 "Chris", Opus 5.5 medium, Cortex, start G1) under an Opus 5.5 commander, against one person, [gecko]u6bkep (lobby skill 30, Armada, start B5). The person said nothing in chat. **Lost at 13:48.** We were held to 4 extractors from 4:30 on by Rover and then Blitz raids. Stouts and a Janus killed the plant at 11:51 and razed the old base by 13:20. The record ends as our commander dies at the new plant at spot_17 (78% at 13:43, 11% at 13:48).

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 2 |
| A2 accepted, not executed | 3 |
| A3 overruled by the pick | 5 |
| A4 overruled by code | 6 |
| A5 late | 0 |
| A6 read differently | 3 |
| B1 in the hands' picture, not the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 report wrong/stale | 0 |
| B4 in the report, missed (Opus's miss) | 0 |
| B5 hands' lines misleading | 1 |
| B6 commander-to-player hop | 1 |
| C (clean judgement moments) | 3 of 8 |

Denominators: 8 judgement moments; 128 player orders and 14 commander orders; 393 hands' lines (49 flagged); 2 refused.

The 8 moments:
- (a) The opening: items at 1:39, 1:47, 2:08 and 2:09.
- (b) Raid at 2:41 (spot_5, a Rover): C.
- (b) Raid at 3:54-4:11 (6 Rovers): C. group_D attacked whole at +1 s as written; the Rovers outran it.
- (b) Raid at 4:21-4:26 (spot_3): item 4:21.
- (b) Raids at 5:29 and 7:45 (spot_25): item 7:25.
- (c) The 3:00 window at 3.3x and the first push west (3:34/4:12): item 4:27.
- (d) The 4:32/4:48 packets: item 5:00.
- (d) The 9:22 packet (reclaim at F1/F2): C. The commander and three constructors reclaimed 9:35-10:03 and fled 2 s after the 10:01 Blitz wave was first in the picture.

The last turns (e) were not counted as moments. Their faults (11:11, 12:16, 13:38) came from the sweep and are items.

Packets reached the hands a median 12.7 s of game time after each report (the jev log's `instructions` changes). The longest delays were the 40-s-wall turns: 10:40 (27 s, the takeover) and 11:53 (23 s). The sweep's "landing median 0 s" does not show this realtime delay.

## Items, ranked by cost (15 of 21; the other six are in impedance.json)

1. **13:38-13:48, A3, large.** Packet: "If Stouts come to spot_17, it walks to spot_25, and back when they leave." 3 Janus and 8 Stouts reached spot_17, and the commander kept helping there until it died. The decode read spot_25 at 0.96, but the gate rated `commander.go_spot_25` at 0.22 and then 0.35, so nothing changed. The picture said "its fastest (75) catch our slowest (38)", so the walk may only have delayed the death. Fix: jev.
2. **11:11-11:23, A3, small.** Packet: "It never walks toward a group of Stouts out in the open ... With no enemy near it ... builds a Guard at spot_6." The pick still played "build a corllt at spot_6", with 5 Stouts and 2 Blitzes within 200 of the site. At 11:11 that move was sampled at 0.35 against the walk to spot_17 at 0.65. The Guard's line said nothing of the party at the site. The commander fell from 95% to 54%. Fix: menu.
3. **5:00-5:16, A4, small (group_H lost, 360 metal).** Packet: "group_H: joins group_D now ... one body." The pick played a follow instead. One frame after the follow's walk order (record cmd f9080), fight orders sent the three Incisors toward party_12 (f9081), and no further order came until f9411. They stood 11 s at F2 under 4 Blitzes and all died. Fix: code-rule (the micro engine overrode the follow, and the follow does not re-issue its walk while the ward stands).
4. **6:56, A4, small.** constructor_833's list was re-issued with the same first step, "extractor spot_6". The code reported "the build already under way: taken as done", but the cormex had not started, and the Guard order replaced it at 6:57. No extractor was built anywhere between 4:56 and 7:42. Fix: code-rule (lists.rs).
5. **11:07, A4, small.** The same bug for constructor_16990's "extractor spot_25", in flight since 10:23 and unstarted. It was taken as done and replaced by the Guard order at 11:08; the extractor was never built. Fix: code-rule.
6. **9:43-9:59, A2, small.** The 9:37 list (and the commander's "spot_6 ... first") put an extractor at spot_6. The engine refused it: "the engine refused a cormex at G1: the site was bad". The site was the dead extractor's place. It was re-queued at 10:40 and played twice more, and spot_6 stayed empty from 4:00 to the end. The cause is unknown and the report never gave one. Fix: report.
7. **10:36, A4, small.** "skipped the step 'corhlt spot_6' ...: the extractor at spot_6 was refused lately; a defence there waits for it". The turret was Opus's home defence against the coming Stouts, not the extractor's guard. Fix: code-rule.
8. **12:16-12:55, A6, small.** Two builders were given "corvp spot_17 #vp2" and played it in the same second, so two plant frames went up (22076 at 12:24, 1291 at 12:49). The commander helped the wrong frame, and Opus abandoned the second one at 12:55 (29% at the end). Fix: code-rule (the same-tick tag race, fixed 2026-10-12).
9. **2:08-2:54, A2, small.** The commander's list step "solar" lost its site to constructor_16990's solar placed a second later. It stood 46 s and built nothing until the 2:43 list landed. The report showed it as "about to build". Fix: code-rule (built in 6ea433b).
10. **2:09-2:34, A4, small.** Packet: "group_A ... roves now". The `same` layer kept 0.23 (asked again: 0.57), and group_A held 25 s more. Fix: code-rule (built in 6ea433b).
11. **1:47-2:34, A4, small.** Two joiners (Rascal 26385 and Incisor 25076) stood idle at the yard for 35-47 s while the body held at spot_18. Fix: code-rule (Group::admit, built in 6ea433b).
12. **1:39-2:09, A6, small.** Packet: "with fewer than three roving, a corfav from the group goes to scout (rove)". It was read as rove 0.23 and scout 0.40, so nothing scouted. The first Rascal crossed at 2:51 (pool median: 2:05). Fix: words.
13. **3:02, A6, small.** Packet to group_B: "Its corfav join group_C". The decode read it as group_C joining group_B (0.51), and the pick merged the home picket into the raid group. That caused the 2:59 refusal (item 17 in impedance.json). Fix: jev.
14. **7:25-7:41, A3, small.** Packet: "a lone armflash at ... spot_25: send two Incisors at it". A lone Blitz sat at spot_25. The send rated 0.49-0.58 and was not played; the pick took the home attack instead. The extractor begun at spot_25 at 7:42 died at 7:45 to that Blitz. Fix: jev.
15. **10:40, A1, small.** At the takeover, `produce` to "group_D" was refused (group_D died at 10:25). The note assumed the list was in force. The old list ran on and made Pounders where Opus wanted a constructor second; Opus produced again at 11:20. Fix: tool.

The other six are in impedance.json:
- 10:43, A2: the Warden at spot_10 was refused for a bad site.
- 2:24, B5, low: the report showed "about to build" and "on the way to join" for units that were stuck.
- 2:59, A1, none: `produce` to group_C was refused because group_C had merged.
- 4:21, A3, none: a 2-unit hunt crossed "it never chases armfav or armflash"; the forbid read 0.71, under the 0.8 bar.
- 4:27-4:34, A3, none: "group_D: joins group_B now" ran in reverse (0.68 against 0.52), so the push west never left.
- 11:20, B6, none: the commander's "add them to group_A's roving" went into a new group with no packet line.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 49 turns, 1 session replacements, 128 orders, 2 refused, 49 flagged of 393 hands' lines; landing median 0 s (p90 0), turn wall median 14 s
   orders: {'wait': 48, 'queue': 31, 'instruct': 30, 'produce': 17, 'mark': 2}
   refused: {'produce': 2}
   flagged: {'step-waits': 28, 'other-fault': 6, 'stopped-on-stop': 6, 'step-skipped': 4, 'engine-refused': 3, 'hunt-ended-other': 1, 'follow-ended': 1}
-- commander.jsonl: 11 turns, 0 session replacements, 14 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 15 s
   orders: {'direct': 9, 'wait': 2, 'say': 1, 'mark': 1, 'units': 1}
   flagged: {}
```

The 28 step-waits are the packet's own rule ("an enemy within reach: it walks to the commander"), held for minutes by the Blitz waves. constructor_16990's "corllt spot_25" waited from 5:22 to 7:02. They are Opus's ask, not faults. That rule and the 6:56, 9:59 and 11:07 items together kept the extractors at 4.

## What Opus wanted that the harness has no lever for

- **1:54, 2:24, 2:43:** "Its corfav stay at spot_18 as the picket against his raiders" and "Its corfav join group_C". This splits the Rascals off a mixed group by unit type. The menus can join or follow whole groups and detach n soldiers after a party. No move sends a group's units of one type to a place or into another group.
- **3:34, 4:12, 4:32:** group_B's raid route: "It keeps out of reach of his commander (armcom) and never goes into his_base itself." The path of an advance is chosen by code. The hands can pick or refuse the move, but cannot put an avoid-region or avoid-reach constraint on it.

## Commands run

```
grep -n 'name=' MATCH/script.txt; tail -c 600 MATCH/record-0.jsonl; grep '"k":"chat"' MATCH/record-0.jsonl
run/impedance_sweep.py MATCH --counts | --seat 0 | --commander | --seat 0 --all --from 1:54 --to 3:18 (and 4:48-10:45, 11:53-13:48)
run/minutes.py MATCH; run/raid_ledger.py MATCH; run/layers_read.py MATCH --faults
run/pro_baseline.py MATCH (no carded side) and --all-starts --faction any
run/commander_turns.py MATCH --calls | --notes
run/hands_window.py MATCH 1:39 2:09; 2:05 2:45 group_A; 2:56 3:04 group_C group_B; 3:50 4:16; 4:18 4:24; 4:26 4:46; 4:44 5:34; 9:54 10:06 (constructors); 11:08 11:30 commander; 13:30 13:48 commander
python3 over strategist-0.jsonl (full packets and results, turn prompts at 2:24, 6:49, 7:16, 7:37, 9:22, 10:40, direction lines, packet landing frames)
python3 over jevlog rows (decode reads, call answers and gate lines at 2:02, 4:20-5:16, 7:25-7:43, 11:07-11:15, 13:32-13:44; group tasks)
python3 over record-0.jsonl (created/finished/destroyed, cmd rows for group_H, 833, 16990 and 24695, s rows for the Rascals and the commander's health)
sed crates/bot/src/brain/pianist/decode.rs, lists.rs (what a reading means; 'taken as done'); git show --stat 6ea433b
```

Notes: hands_window printed PLAYS and PICK lines but no per-actor menu ratings for this log, so the ratings were read from the `call` rows' `answers` and the `worlds_gate` lines. Only one seat: no symlink directories were needed.
