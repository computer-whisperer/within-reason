# Impedance review: apex-v022-carrot-sym50-opus-2 / 00

An arena duel on Carrot Mountains v2.0 (+50/+50, seed 4), played 2026-10-13: **won 18:10** against Apex v0.2.2 (standard). One seat (strategist-0, Opus 5.5 medium) under an Opus 5.5 high commander, which wrote 11 `direct` paragraphs. The hands ran with all twelve layers on Jev 1.13.0 (jev-latest). There were 2 session takeovers (9:49, 15:52). The army led 2.3 to 6x from 4:00 to the end, but four assaults on one turret nest at spot_98 (G5) were costly. Seat 0 only, so no symlink directories were needed.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 0 |
| A2 accepted, not executed | 3 |
| A3 overruled by the pick | 4 |
| A4 overruled by code | 7 |
| A5 late | 0 |
| A6 read differently | 6 |
| B1 in the hands' picture, not the report | 0 |
| B2 in record/truth, in nobody's picture | 0 |
| B3 report wrong/stale | 0 |
| B4 in the report, missed (Opus's miss) | 0 |
| B5 hands' lines misleading | 1 |
| B6 commander-to-player hop | 0 |
| C followed, context enough (of the moments) | 3 |
| **denominators** | 8 judgement moments; 215 player orders + 14 commander orders; 1,185 hands' lines (96 flagged); 0 refused |

Judgement moments: (a) the opening, not clean. (b) the raids: 5:45 Ticks at A6 (A6), 10:47 spot_95 (C), 15:46 spot_114 (C), 17:29 spot_62 and 17:57 spot_76 (A3, A4). (c) the first push, 6:13/6:38 (A3). (d) the packet changes of 7:44 (C) and 11:05 (A3). Eight were reached at (d), so (e), the last three turns, was not examined. B6: all 11 directions were carried into the player's orders. The 12:21 commander `signal` collided with the seat's own signal and is counted as A4. The 16:51 `north_shell` was replaced at 17:18 because of an Overwatch first seen at 16:53, after the direction was written.

## Items, ranked by cost

1. **13:41, A4, large (high).** Wanted (13:33): "Its Blitzes stand at spot_116 as the screen ... never walk into the reach of the turrets at spot_98." Happened: the shell play put the screen at 0.55 x the Mauser's 820 = 451 from spot_98 (`execute.rs` Order::Shell, SCREEN 0.55). That point is inside the Beamers' 490 and the Overwatch's 620. Measured: 30 Blitzes (3,300 metal) died 13:41-14:26, 29 of them inside a known turret's reach, and the artillery did 0 damage to turrets. Fix class: code-rule (the screen stands outside the reach of the turrets covering the target, or at the packet's place).
2. **9:19, A4, small (high).** The same mechanism under the 9:03 packet ("does not walk the Blitzes into the Beamers' reach"): the screen stood at 390 from spot_98. 8 Blitzes (880 metal) died 9:38-9:45 at 167-605 from spot_98. Opus read this as "already under Beamer fire at the standoff" and went ALL IN at 9:39. Fix class: code-rule.
3. **6:46, A3, small (high).** Wanted (6:38): "Its way: spot_116, then spot_63, then spot_79 ... never walks toward spot_107 or spot_98". The pick kept the old fight_to his_base (change 0.28-0.45; decode spot_63/79/116 all 0.94). "group_E reached spot_98" at 6:51, and 5 Blitzes (550 metal) died at G5 6:50-7:00. Fix class: jev.
4. **5:38, A6, small (medium).** The 5:14 packet said a Tick at our extractors "is met by one soldier sent from the nearest group". The same packet sends every small group to group_E, and group_E never splits. group_L and group_K played "join group_E" (0.71/0.69 against attack_party_11 0.42), and the Ticks took spot_120, spot_118, spot_124 and spot_119 (5:45-6:00). Fix class: words.
5. **1:48, A6, small (high).** The code sited the plant facing spot_99, 130 out in its exit lane. The commander's unplaced "extractor" step built on spot_99. constructor_11430 was stuck 1:51-2:33, and plant_3156 made nothing 1:51-2:34 until Opus self-destructed the extractor. Fixed afterwards by 3aabc77, which names this game. Fix class: code-rule.
6. **17:25, A3, small (high).** Wanted (17:18): Pawns at spot_62 "attacked by group_X2 and group_E3 together, whole". Happened: group_E3 kept hold at 0.61-0.66, although its decode read spot_62 at 0.54. group_X2 split 8 east at 17:29 to hunt party_109 under the nest. spot_62 was lost at 17:29 and spot_76 at 17:57. Fix class: jev.
7. **9:42, A6, small (high).** Wanted (9:03): "answered by sending four Blitzes". The pick sent 8, then 4 more, then 4 more at single Ticks and Pawns (decode detach|4 0.29, detach|8 0.46: no size forbidden). That took 16 Blitzes off the body just before the ALL IN landed. Opus at 9:49: "the all-in went in piecemeal". Fix class: jev.
8. **9:42, A4, small (high).** The hunts of group_G1, H1 and I1 ended "after 0 s" (I1 after 2 s) on the 900 leash. The leash counts any hunter over 900 from the detachment's centre (`micro` lib.rs), so a detachment taken from a strung-out body ends on its first tick and is left standing apart. Fix class: code-rule.
9. **8:45, A4, small (high).** The same leash, under the 8:35 packet ("answered by sending four Blitzes"). Three detachments of 4 at party_21 (1 Pawn) each ended after 0 s: 12 Blitzes stranded, and the Pawn lived. Fix class: code-rule.
10. **10:00-15:42, A2, small (medium).** Once a produce list's counted entries ran out, every unit of the open "armflash" entry waited for a pick. Plants stood idle in 20-33% of samples (0-3% while counted entries ran) with 1,500-1,900 metal banked. The commander caught it at 15:21: "Three vehicle plants stand idle". Fix class: code-rule.
11. **11:13, A3, small (high).** Wanted (11:05): "spot_101, then spot_116, walking now". group_E (62-70 units) stood 20 s at D5 on a stale "standoff from party_50" task (change 0.48-0.50, decode spot_116 0.96). The group moved only after Opus rewrote the line as "its place is spot_116" (change 0.63 at 11:33). Fix class: jev.
12. **15:40, A3, small (high).** Storm was given at 15:21, and the 14:26 packet said "does not come back once storm is given". At 15:40 the pick played "walk to spot_116" (0.52), and the body left the storm until the 15:42 packet landed. Fix class: jev.
13. **10:09, A6, small (medium).** The lane setting "march" held group_E's front for its tail, which was the plants' new joiners at home. The front went from 3,277 to 3,468 "to go" ("0 of 32 arrived ... for 22 s") until Opus dropped march at 10:19. Fix class: code-rule.
14. **11:43, A6, small (medium).** "armfus home #fus1" was given to two tier-1 constructors to help the fusion. Both were "skipped ...: this builder cannot build it", read as build, not as help. The fusion ran 12:43-14:30, and energy stalled at 14:03-14:13. Fix class: tool.
15. **13:43, B5, small (medium).** The play "shell ... from their reach with 21 long-reach soldiers, 37 between" and the entry "advancing to standoff" never said the screen stood inside both turrets' reach. Opus blamed the push signal (13:43) and blindness (14:03), and only held the Blitzes outside reach at 16:50 (`siege_line`). Fix class: report.

Not ranked, cost small or none: 13:33 A4 (the seat's push "20:00" was shadowed by the side's push "given 13:30" until 13:51; `shared.rs` signals_all lets the side's signal win a clash; the pick never took the push branch, but Opus re-keyed its packet onto a new signal, "storm"). 13:05 A4 (push "now", twice; the tool said "given now" while the hands read "not yet given (set for 13:30)"). 17:16 A6 (15 tier-1 constructors on "armsilo #silo1", all skipped "cannot build"; the frame only began at 17:53). 17:58 A4 (leash ended X2's hunt of the 23-Pawn raid after 0 s). 2:51 A2 (the commander was queued "armnanotc", which its build list lacks; the tool accepted it). 13:00 A2 (armllt spot_142 skipped "waits for" an extractor that another constructor had built; no turret ever stood there). All ten extractor refusals ("the site was bad") were at spots our own extractor already held, so they cost nothing. The "what your hands did" blocks matched the Jev log's plays at 13:05-13:15 and 6:48-6:58.

## Sweep counts, verbatim

```
-- strategist-0.jsonl: 95 turns, 2 session replacements, 215 orders, 0 refused, 96 flagged of 1185 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 95, 'queue': 38, 'instruct': 35, 'produce': 18, 'signal': 12, 'lane': 8, 'mark': 7, 'remove': 2}
   flagged: {'step-skipped': 37, 'engine-refused': 23, 'hunt-ended-leash': 17, 'hunt-ended-other': 8, 'stopped-on-stop': 6, 'other-fault': 3, 'follow-ended': 1, 'site-blocked': 1}
-- commander.jsonl: 12 turns, 0 session replacements, 14 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 19 s
   orders: {'direct': 11, 'signal': 2, 'mark': 1}
   flagged: {}
```

## What Opus wanted that the harness has no lever for

- **13:33** (also 9:03, 11:25, 12:45 and 13:43; the commander at 13:51): "Its Shellshockers and Mausers shell the turrets at spot_98 from their own reach, the Overwatch (armhlt) first, then the Beamers, then the Sentry, then his resurrection bots". The shell move is a fight-move to a standoff point. It cannot attack the ground at a remembered building, and it cannot set a target order. The artillery did 0 damage to turrets from 13:41 to 16:20 and hit Beamers only once they were in sight.
- **13:33** (also 9:03, 9:49, 11:25): "Its Blitzes stand at spot_116 as the screen in front of the Shellshockers and never walk into the reach of the turrets". No move places a group's artillery at its reach and its screen at a named place or outside turret reach. The screen is fixed at 0.55 x the artillery's reach from the target. The workaround Opus found at 16:50 was to hold the whole group at a marked `siege_line`.
- **10:19**: march for the body walking to spot_116 while the plants' new soldiers join from home. march waits for every member, joiners included, and cannot be limited to the body.
- **17:08** (also 11:35 #fus1): "15 idle/assisting constructors put on it (#silo1)". A list step with an id is read as building that type, so tier-1 builders cannot be listed to help a tier-2 frame. Only the packet's help moves can do it, and only once the frame stands.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "import json;print(json.load(open('MATCH/../batch.json')))"
run/impedance_sweep.py MATCH --counts | --seat 0 | --commander
run/minutes.py MATCH; run/wake_read.py MATCH; run/raid_ledger.py MATCH; run/pro_baseline.py MATCH (no carded side)
run/commander_turns.py MATCH --calls
run/hands_window.py MATCH 13:40 13:52 group_E | 13:02 13:40 group_E | 13:52 14:27 group_E | 9:30 9:58 | 9:08 9:31 group_E
run/hands_window.py MATCH 5:36 6:02 | 6:19 7:02 group_E | 7:41 8:20 group_E | 8:43 8:49 | 10:41 10:56 | 11:12 11:38 group_E
run/hands_window.py MATCH 15:24 15:50 group_E | 17:24 17:40 group_X2 group_E3 group_V3
python3 (scratch) over strategist-0.jsonl and commander.jsonl: turn prompts, full packets, signal lines and results, plant and group entries
python3 (scratch) over jev-0.jsonl.zst via run/jevlog.py: decode rows (6:46, 9:11, 9:47, 11:13-11:33, 13:30-14:00, 14:34, 17:26) and play lists for two report windows
python3 (scratch) over record-0.jsonl and truth-0.jsonl: deaths per window and their distance to known turrets, artillery shots and damage by window (xo), plant idle flags, build history (armmex/armllt/armfus/armsilo/armadvsol), his units and turrets at key clocks and when we first saw them
code read: crates/bot/src/strategist/{mcp.rs (signal, lane, produce, fire), shared.rs (signals_all)}, brain/pianist/{execute.rs (Order::Shell), decode.rs (FUSE/ON/OFF), groups.rs (HUNT_LEASH)}, crates/micro/src/lib.rs (the leash check)
```
