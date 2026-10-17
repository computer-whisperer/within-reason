# Impedance review: 1791335540-human-26/00

Human 2v2 on Comet Catcher Remake 1.8 (2026-10-06, realtime). Our side had two Opus 5.5 medium players, t2 "gajop" (G1, strategist-0) and t3 "Glass" (H8, strategist-1), under an Opus 5.5 commander. They played against [gecko]Covariance and [gecko]u6bkep (Armada, both on Rover then Blitz swarms; the plant was seen at B8). The lobby start boxes were not honoured. The record ends at 10:07 ("unexpected end of file") and nobody stated a result: t2's commander died at 7:20 at F1 (the notes say 7:28, the turn that reported it), t2's plant died at 9:15, and t2 was out by 9:49. t3 was still standing with 8 soldiers against 28 Blitzes in sight. No person wrote in the chat (4 chat lines, all ours).

## Tally

| A1 | A2 | A3 | A4 | A5 | A6 | B1 | B2 | B3 | B4 | B5 | B6 | C |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 1 | 5 | 1 | 1 | 4 | 2 | 1 | 1 | 0 | 1 | 4 | 0 |

Denominators: 8 judgement moments (4 a seat; the commander hop checked at each), 0 clean. Orders: 210 (t2 85, t3 116, commander 9). Hands' lines: 428 (222 and 206), 53 of them flagged. Refused: 0. Takeovers: 1 (t3 at 8:37).

Where the cost sits: all three large items are in t2's game (the commander's death at 7:20 and the Rover raid on its home at 3:33). Across both seats, the recurring shape is A3. The packet said "attacks party_N now with the whole group", and the gate rated that move 0.2-0.49, under 0.5, at every second; the pick held, gathered, or walked to the clause's "afterwards" place. That happened in 5 cases. In one more (t3 at 6:37) the rating reached 0.49 and the move was taken at 0.76 two seconds later.

## Items, ranked by cost

1. **7:01, t2, A6, large (medium).** Wanted (6:52 packet): "group_H_t2: the home guard. It stands at spot_11 and attacks any enemy armfav or raider at our extractors ... never chases them past spot_18 or spot_17." Happened: the pick had merged group_H into group_I at 6:58, so the packet landed at 7:01 naming nothing. The decode gave group_I no mark at 0.5 or over, and at 7:05-7:06 the pick sent all 4 of group_I's Blitzes after party_39, one Blitz at D8 about 4,300 away (2 at 7:05 as group_J, the other 2 at 7:06). From 7:06, party_18 (2 armfav, 8 armflash) killed the commander, which died at 7:20 (2,700 metal, the seat's builder). Fix: code-rule (keep a group's name, or re-point the packet's name, when the pick joins a group into its own detachment).
2. **7:03, t2, A4, large (medium).** Wanted: the same home-guard clause, with party_18 at our spot_11 extractor. Happened: "group_I_t2's hunt of party_18 ended after 8 s: the leash of 900 from where it began reached". The hunt had begun at D2, where the guard had chased west, and it ended 950 short of party_18, 3 s before the commander was hit. This is the same death as item 1. Fix: code-rule (measure the leash from the guard's zone, not from where the hunt began).
3. **3:27, t2, B2, large (low).** Wanted: "group_B_t2: the raid. It goes west now ... Enemy armfav at our home extractors are not its business once it has left spot_18." Not known: the record shows 8 armfav at G6 and 1 at H6 at 3:03, heading north. None of the game's 81 reports has a recently-seen/out-of-sight block, and the 3:27 report showed only 2 armfav at B3. group_B left; 11 armfav hit home at 3:33; the commander's fights table reads "3:35-4:25 at G2 ... We lost 3 armcv, 2 armsolar, 4 armmex, 1 armllt (1000)". The cost link is an inference. Fix: report.
4. **3:53, t2, A3, small (high).** Wanted (3:45 packet, said again at 3:56): "group_B_t2 attacks party_1 now with the whole group and keeps attacking until it is dead". Happened: "kept w1 over w3 ... changed=[]" at 3:56, 3:59, 4:01, 4:03, 4:04 and 4:05, with group_B standing at spot_27 1,500-1,900 away, then "4 of group_B_t2 hunt party_14 (1 armfav)". attack_party_1 was rated 0.26-0.49 throughout, and the decode's home mark landed (0.87). Two solars were lost (310 metal). Fix: jev.
5. **2:10, t3, A3, small (high).** Wanted (2:00 and 2:13 packets): "group_A_t3 attacks party_1 now with the whole group and keeps attacking until it is dead." Happened: "gather on its front" at H5 (2:10) and "hold where it stands" at H6 (2:19), with attack_party_1 at 0.20-0.33. Constructor_9614 died at 2:21 and spot_65's extractor at 2:31. Fix: jev.
6. **6:21, t3, A6, small (medium).** Wanted (6:19 packet, and 5:48): "group_F_t3 ... attacks enemy armfav that come within reach of home, spot_65, ... spot_70 or spot_75, with the whole group". Happened: party_31 was armflash, not armfav. group_F trailed the commander for 81 s (attack_party_31 0.32-0.35) while spot_75's extractor, a Sentry and two constructors died (405 metal, constructors 3 to 1). Fix: words (say "raiders"; Opus's 6:30 packet fixed it).
7. **6:48, t3, A3, small (high).** Wanted (6:39 packet): "group_F_t3 no longer follows the commander. group_F_t3 attacks party_31 now ... keeps attacking until they are dead or gone. ... Afterwards it stands at spot_77". Happened: "walk to spot_77" (6:48), then "walk to home" (6:52), with attack_party_31 at 0.19-0.22. party_31 had grown to 7; one Blitz died on the walk. Fix: jev.
8. **9:02, commander, A5, small (medium).** Wanted (8:50): "t2's six idle Blitzes at G1 have been given to t3". Happened: send_units ran at 9:02, 12 s into a 27.6 s commander turn. By then the 7 Blitzes were fighting 13 armflash at t2's home (in sight from 8:52). t3's hands walked them home to H8 under fire and 2 died (9:13, 9:16); t2's plant died at 9:15. Fix: tool (a transfer that says or re-checks what the units are doing when it lands).
9. **8:59, t2, B6, small (medium).** Wanted (t2's 8:59 packet): "group_I_t2: the one army. It stands at home beside the plant". Not known: the commander had given group_I away, and the direction reached t2 only "after 9:09". t2 learned at 9:10: "after our six Blitzes were given to t3". Its constructors stayed home building solars; two died (9:07, 9:20). Fix: report (tell the giving seat at once).
10. **3:44, t2, A2, small (high).** Wanted: the commander's list step "extractor" (the nearest free spot). Happened: it walked to spot_11, which constructor_5873's list started at 3:24; "the engine refused a armmex at F2: the site was bad". The commander stood "idle, waiting for an order" at F2 as 11 armfav hit G1. Fix: code-rule (skip a spot another builder's list holds).
11. **3:27, t2, B6, small (medium).** Wanted (commander 2:50, shown 3:27): "Keep two Blitzes at home as the north-east picket against Rovers." Happened: t2 sent all 4 Blitzes and 1 Rover west and wrote "the next two Blitzes out of the plant stand at spot_12". The next Blitz was started at 4:03; home was empty for the 3:33 raid. Fix: brief.
12. **8:55, t3, B1, small (medium).** Wanted (the earlier session, 8:27): constructor_22438 "armbeamer spot_77" ("Beamers ... hit much harder"). Not known: the takeover session (8:37) replaced the list at 8:55. The takeover prompt and the builder's entry show the build under way but not the list in force. Then "dropped the step 'armbeamer spot_77'" (9:14), and spot_77 was lost at 9:25. Fix: report.
13. **0:10, t3, B3, none (high).** Wanted: "The enemy is two people in the north (rows 1-2), likeliest straight north of us around spot_10, spot_12, spot_5. Our ally's seat is beside us in the south." Report: "its commander was placed at 0:00 inside its lobby box, cells A1-H2", while the boxes were not honoured and our ally t2 stood there. No line gave the ally's start, which the hands knew ("ally team 2 starts at (6899, 681)"). Corrected at 2:00. Fix: report.
14. **0:11, t2, B1, none (high).** Wanted: "armfav scouts drive south toward spot_70, spot_75, spot_72, spot_65 to find him", which are t3's home spots. The ally's start (H8) was in the hands but in no report line. Rewritten at 1:25, before the first Rover. Fix: report.
15. **7:01, t3, A6, none (high).** Wanted: the 6:39 remove (reclaim armmex_27451), and at 7:01 "it finishes taking apart the extractor blocking the plant's exit lane". Happened: the 7:01 queue ["extractor spot_72", "assist"] replaced the list whose front step was that reclaim. The result said only "2 steps, done in order". Opus at 8:08: "The first order at 6:39 was never carried out." This was harmless by luck (see the B5 below). Fix: tool.

Not in the top fifteen (all cost none, in impedance.json):
- 5:13, t3, A3: a detachment of 2 of 3 against "with the whole group" (send_2 0.58, forbidden 0.55, decode detach|2 0.61).
- 5:20, t3, A3: group_A kept following the commander after "It does not follow the commander".
- 5:33, t3, A6: the decode read "group_E_t3 and group_F_t3 join group_A_t3" both ways (group_A_t3|join|group_F_t3 0.68), so "group_A_t3: join group_F_t3" was played.
- 6:39-8:55, t3, B5: the yard lines said the exit lane was blocked by our own spot_65 extractor while the plant kept putting units out. Opus ordered that extractor reclaimed twice, then cancelled: "I'd misread it as a stray building".
- 1:14, t3, B6: Opus held the commander's "Blitzes cross west along rows 6-8", with a reason given in a note.
- 4:53, t3, B6: Opus's "It never goes at his vehicle plant or his commander" against the commander's "go west ... to his plant and commander at B8".

## Sweep counts, verbatim

```
-- strategist-0.jsonl: 34 turns, 0 session replacements, 85 orders, 0 refused, 23 flagged of 222 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 34, 'instruct': 20, 'queue': 14, 'produce': 6, 'transfer': 6, 'lane': 4, 'mark': 1}
   flagged: {'step-waits': 6, 'other-fault': 5, 'hunt-ended-other': 4, 'hunt-ended-leash': 3, 'follow-ended': 2, 'engine-refused': 1, 'step-skipped': 1, 'stopped-on-stop': 1}
-- strategist-1.jsonl: 47 turns, 1 session replacements, 116 orders, 0 refused, 30 flagged of 206 hands' lines; landing median 0 s (p90 0), turn wall median 11 s
   orders: {'wait': 46, 'queue': 27, 'instruct': 27, 'produce': 10, 'lane': 4, 'remove': 2}
   flagged: {'step-waits': 14, 'other-fault': 8, 'step-skipped': 4, 'hunt-ended-other': 2, 'abandoned': 1, 'stopped-on-stop': 1}
-- commander.jsonl: 7 turns, 0 session replacements, 9 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 24 s
   orders: {'direct': 6, 'wait': 2, 'transfer': 1}
   flagged: {}
```

How the sampled flagged lines were judged:
- **engine-refused (t2 3:44):** item 10.
- **hunt-ended-leash (t2):** 7:03 is item 2. 6:57 (group_H after party_37, far west) was within the packet's own "never past spot_18/17", so it is not an item. 4:15 (group_C after party_14) is not counted, because the packet in force named party_1 and not party_14.
- **abandoned (t3 6:44):** the spot_70 frame decayed when its builder fled a 6-Blitz raid, as the packet said ("while enemies are within reach, a constructor walks to the commander"), so it is not an item.
- **step-skipped (5):** spots already held, so none are items.
- **other-fault (t3 9:14):** item 12. The rest of the other-fault lines are dead wards and parties gone.

## What Opus wanted that the harness has no lever for

- **3:27, t2 (and the commander at 2:50):** "it goes round a turret with soldiers beside it" and "otherwise go round it". No move or tool routes a group around a turret; moves are place to place and the engine paths.
- **4:28, t2 (again 4:48, 5:38, 6:52, 7:05):** "It does not chase past spot_18 or spot_17". A hunt cannot be bounded by a named place; the only bound is the 900 leash from where the hunt began, and at 7:03 that began far outside the zone.
- **6:39, t3:** "What I'd want is an order that ends a group's current course outright when a new packet lands." No tool clears a group's course. group_F's follow stood 81 s across two packets.
- **0:20, commander to t3 (also to t2):** "Your first two or three Rovers go on rove: the look we need is columns A-C, rows 5-8". Rove cannot be aimed; it looks at his lobby box first, which here held our ally, and t3's rovers drove north to H5-H6.

## Judgement moments (i wanted, ii done, iii known, iv verdict)

- **t2 (a), opening.** The commander list (3 extractors, 3 solars, plant at 1:07-1:26, assists) and the produce counts ran as written; an 11 s turn let the old list's next cv and Blitz through before the new list began. Not clean: items 14 and 10. The commander hop (0:20 "Rovers on rove ... expand through F1-F3, E1-E3, H2-H3") was carried.
- **t2 (b), Rover raid 3:34-4:12.** The answer came 20 s after first sight (commander_t2 attack party_1), but group_B never attacked. The report's enemy lines matched the record, and its hands block matched the jev plays (3:49 advance to spot_27, 3:53 commander attack). Not clean: items 3, 4 and 11.
- **t2 (b), commander's death 7:03-7:20.** The reports were accurate: 7:05 said "group_I_t2 ... reads: no place" and "9 at F2 ... killing our Armada Commander". The fault is in the hands. Not clean: items 1 and 2. The 7:18 order to D-gun landed at about 7:28, after the death; the turn was 10.5 s, so it is not counted as A5. The commander hop was carried (7:20: the raid stays west).
- **t2 (b), base loss 8:52-9:47.** Not clean: items 8 and 9. Nobody saw the 15-Blitz pack before 8:52 (t2's record shows no enemy within 2,500 of the plant 8:30-8:51), so the commander's choice to give the Blitzes away was not short of context.
- **t3 (a), opening.** The role-word commander list and the produce counts ran as written (plant at 1:16-1:35, cv, 3 Rovers, cv, Blitzes). Not clean: items 13 and the 1:14 B6.
- **t3 (b), raid 1:51-2:31.** The answer came 2 s after first sight (the Rovers attacked party_1 at 1:54), then the rove ordered at 1:51 took them north. Not clean: item 5.
- **t3 (b), swarm 4:34-5:37.** The report matched the record (16 armfav at G6 at 5:23). Not clean: the 5:13 and 5:20 A3s, the 5:33 A6 and the 4:53 B6.
- **t3 (b), Blitz raids 6:21-7:41.** The report's actor entries named the exit lane and the 81 s follow. Not clean: items 6, 7 and 15, and the B5.
- **Fixed check, everywhere:** the report's enemy lines matched the record at every moment checked (t2 3:45, 3:56, 7:05, 7:18, 8:59, 9:10; t3 2:13, 2:22, 5:23, 6:39, 6:51). The hands blocks matched the plays, and the stuck units and exit lanes were reported. The one thing missing throughout was enemies out of sight: 0 of 81 prompts had a recently-seen block.

## Seats and tools

Seat 1 (t3) was read through symlinks in `/tmp/claude-1000/-home-christian-workspace-playground-bar-bots/93f9e8f3-fea1-482e-a1c2-cb750e01d6e2/scratchpad/impedance/agents/1791335540-human-26/seat1/00` (record-0, jev-0.jsonl.zst, strategist-0, terrain-0 pointing at the seat-1 files).

Tool caveats:
- minutes.py on seat 1 takes our ally's G1 as "his start", so its sent-to-his-side table for t3 is wrong and was ignored.
- pro_baseline has no carded side for these starts.
- There is no truth file, so raid_ledger's paid column is 0 throughout.

Commands run (from the repository root; M = run/matches/1791335540-human-26/00, S1 = the seat-1 symlink directory):

```
grep -n 'name=' M/script.txt; tail -c 600 M/record-0.jsonl M/record-1.jsonl; cat M/bot.log
python3 run/impedance_sweep.py M --counts | --seat 0 | --seat 1 | --commander | --seat N --all --from m:ss --to m:ss
python3 run/minutes.py M ; python3 run/minutes.py S1
python3 run/raid_ledger.py M ; python3 run/raid_ledger.py S1
python3 run/pro_baseline.py M ; python3 run/pro_baseline.py S1
python3 run/commander_turns.py M --calls ; python3 run/commander_turns.py S1 --calls
python3 run/hands_window.py M 1:45 2:26 plant_28196 | 3:28 4:30 | 3:53 4:08 group_B_t2 | 6:52 7:30 commander_t2 group_I_t2 group_J_t2 group_K_t2 group_H_t2
python3 run/hands_window.py S1 1:50 2:40 | 5:10 5:42 | 6:15 6:40 | 9:01 9:30 group_I_t3
grep '"k":"chat"' M/record-0.jsonl
scratch scripts (jevlog.rows over jev-0/jev-1 for decode/worlds_gate/plan/call rows; record events, cmd rows, positions; turn prompts and assistant text) in .../agents/1791335540-human-26/bin/
```
