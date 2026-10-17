# Impedance review: 1792195269-jev114-player-1

Arena duel, 2026-10-16, Comet Catcher Remake 1.8: **won 19:09** against BARb hard_aggressive. One seat (strategist-0), player Opus 5.5 medium, no commander, the hands on **Jev 1.14.0** (every other game of the tally ran on 1.13), layers news,same,fuse,openers,split,tick,places,again,near. Truth file read.

## Tally

| A1 | A2 | A3 | A4 | A5 | A6 | B1 | B2 | B3 | B4 | B5 | B6 | C (clean moments) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 1 | 1 | 4 | 0 | 2 | 1 | 1 | 2 | 0 | 0 | 0 | 3 of 8 |

Denominators: 8 judgement moments; 95 turns; 193 orders; 813 hands' lines (51 flagged); 0 refused; 2 session takeovers (10:11, 16:52).
Four items come from the sweep and the player's notes, outside the eight moments: A6 15:19, A3 18:51, B3 19:03, and the A4 leash at 17:58.

## Items, by cost

1. **6:18-19:09, A2, large, high.** Wanted: the commander's `armvp #plant2` (6:01, 6:21, 6:31), `armlab #lab1` (7:21, with constructor_26737), solars (8:29), and constructor_26737's `extractor spot_45` (10:11). Happened: the code chose sites for their own unsited `armsolar` steps (4:22-6:06: nine 10x10 solar frames round (2066-2223, 5394-5501), plus armllt_1736) and walled both builders in. Every move failed from 6:18 (commander: 232 move_failed to 19:01) and 6:50 (constructor_26737: 334 to 19:06). "6:26 commander: its armvp at C8 never started in the 8 s it stood at the site: the site is blocked". The armlab was placed 7:29 and never created. Cost: the commander out of the game for 12.8 min and the constructor for 12.3 min. No bot lab all game, and other constructors built the second plant (7:14-7:53). Fix: code-rule (a site search must leave its builder a way out).
2. **9:52, B3, large, medium.** The player ordered the push on "Its soldiers seen in the last 3 min and not seen to die: 20 worth 2146 metal; its army is at least that and may be much more". The player's own reading was "2481 vs 2146 seen", and at 10:11 "parity-plus". The truth: 3476 at 9:52 (the floor was 62% of it) and 3768 at 10:11 (57%), and the floor did not change in between. The push met his block of 16 (6 armham, 5 armrock, 5 armwar) at E4. Report 12:06: "in the last 3 min we lost 3114 metal ... destroyed 914". The line is labelled a floor, so the misreading is partly Opus's (B4). The report offers no estimate beside it. Fix: report.
3. **7:38-9:40 and 16:52-19:09, B1, large, medium.** The hands' picture said "metal: 1896 of 1900 stored (full: what comes in beyond this is wasted)". The report said "eco: metal 1896 (+38.7/-22.5), energy 1969/2000 (+179)": metal had no store size and no "full", while energy shows both and says STALLING when empty. Measured: 241 of 1150 s (21%) at 98% of the store or more, about 5,500 metal of income over spending in those seconds (about 1,240 in the first window, about 4,280 in the second). The player knew of a bank (6:51, 8:09, 16:52) but named the cap only at 8:52. Fix: report.
4. **6:21-19:09, B2, large, medium.** Every report said "commander: ... stuck: it cannot move from spot_69 (C8), its moves have failed since N s ago". No report said what walled it in (its own solars, by handle) or that `remove` on one would free it. The exit-lane yard line does both ("In the lane: armmex_28188; the player's remove tool takes them away"), and the player used that tool. Notes: 6:51 "likely boxed by solars", 12:06 "tried stop+assist", 12:46 "yard seems jammed by buildings". The player never removed a solar. Cost: the same 12.8 min as item 1. Fix: report (name the blockers and the remedy, as the yard line does).
5. **1:41, A4, small, medium.** Packet 0:10: "Constructors: their job is taking free metal spots, in this order as they are free: spot_73, spot_68, spot_69, ...". constructor_18284 went idle in the same gate that first asked spot_73.place (0.70) and spot_68.place (0.64). The places layer opens a place's moves only the second after it says yes, so every `build_armmex_spot_N` was `blanked: true`, and the hands played "constructor_18284: help plant_27255 build [plan]" (0.73). The openers layer then held it until the 2:08 list (34 s). No decode row exists for the 0:10 packet. Fix: code-rule (an idle builder's pick should wait for its places).
6. **0:35-12:46, A4, small, high.** The code sited plant_27255 at (1584,5456), facing east, with spot_73 in its exit lane (the 3aabc77 keep-out is in this build). The commander's own `extractor` step then built armmex_28188 there at 2:02. Wakes 2:31, 11:26 and 12:46: "in the lane: armmex_28188". Two soldiers stood stuck 12:16-12:46. The player reclaimed it (12:16) and self-destructed it (12:36, gone 12:46); spot_73 was rebuilt at 16:03. Fix: code-rule (the exit-lane pattern recurs after its fix).
7. **9:54-10:02, A6, small, medium.** Packet 9:22: "A single raider at an outer extractor is met by one or two soldiers sent from the nearest group, never the whole body" and "group_K, the body: until push is given, it holds at spot_59". For one armpw at A8, 2,600 away, the hands played: "9:54 2 of group_K hunt party_24 as group_X", "9:55 4 ... as group_Y", "10:02 4 of group_K hunt party_24 (1 armpw) as group_B1 [plan]". The decode (9:42) read group_K|detach|4 at 0.71 and |8 at 0.70, so the limit Opus wrote did not land. The gate's send_4.forbidden was 0.59-0.62, under its 0.7 bar. The body went into the 10:05 push with 11 of its 15. Fix: words.
8. **15:19, A6, small, high.** Packet 15:14: "its way from spot_57, in order, never going back to a place it has reached: spot_42, spot_35, ...". group_H1 had reached spot_42 at 15:11, but "15:19 group_H1: advance to spot_42 [plan]" (0.50 against go_spot_35 at 0.51). The decode read spot_42 at 0.99: the way's order has no decode form. The 35-soldier body walked back for 33 s, until the player's 15:44 packet. Fix: menu.
9. **10:11, A4, none, high.** Packet: "an armpw or armflea at our extractors at home ... is attacked by any Blitz at home until it is dead or gone". "10:11 group_A1's hunt of party_24 ended after 11 s: the leash of 900 from where it began reached", and the same for B1 and C1. The Pawn had already killed spot_68 (10:03) and spot_74 (10:10). It died at 10:24, and nothing else was lost.
10. **17:58, A4, none, high.** Packet 17:49: "a lone armpw ... at our extractors or constructors on our side ... is attacked ... until it is dead or gone". The G2 and I2 hunts of party_36 ended on the 900 leash. armpw_15605 died at 18:10, and nothing of ours was lost in between.
11. **18:51-19:09, A3, none, high.** Packet 18:51: "group_A2 and group_F2: they do not follow anyone and do not hold. They advance (fight to) spot_29, then spot_25, then spot_17". The decode landed (spot_29 0.99, spot_25 0.98, spot_17 0.98). Their change openers rated 0.77 and 0.61, but go_spot_29 rated only 0.41/0.30 and F2's fight_to_spot_29 0.09, and no pick moved them. They kept following at G5: A2 since 17:01, F2 since 18:35. His commander died at 19:01 without them. Fix: jev.
12. **19:03, B3, none, high.** "to win: its commander was last seen at H3 (7516, 1628) 0:02 ago" and "party_49 (3 armck, 1 armcom ...)" appeared beside "killed armcom x1". The record has armcom_21640 destroyed at 19:01. Player: "(decoy? or a misread)". The game ended at 19:09. Fix: report.

## The sweep's counts (verbatim)

```
-- strategist-0.jsonl: 95 turns, 2 session replacements, 193 orders, 0 refused, 51 flagged of 813 hands' lines; landing median 7 s (p90 7), turn wall median 8 s
   orders: {'wait': 95, 'instruct': 35, 'queue': 30, 'lane': 10, 'produce': 9, 'signal': 7, 'mark': 4, 'remove': 2, 'situation': 1}
   flagged: {'stopped-on-stop': 10, 'hunt-ended-leash': 10, 'hunt-ended-other': 8, 'engine-refused': 6, 'step-skipped': 5, 'abandoned': 4, 'step-waits': 3, 'site-blocked': 3, 'follow-ended': 2}
```

Flagged lines judged:
- **Folded into item 1:** site-blocked 6:26, 9:26 and 10:15, and engine-refused 9:48 and 16:58. All are the walled-in commander, constructor_26737 or constructor_18784 (the last stuck 10:04-11:03).
- **Leash ends:** 10:11 and 17:58 are items. 6:29 is not: the packet said "met by one or two soldiers" and group_O killed the flea at 6:36. 10:25 is not: the quarry was already dead. 11:49 is not: no packet clause covered a Pawn at a constructor away from home.
- **Abandoned:** 9:10, 10:30 and 10:42 come from lists the player replaced on purpose. 13:38 is minor.
- **Step-skipped:** 7:05 and 11:44 (spot already ours) and 13:10 (the extractor already gone) are correct. 11:39 and 16:58 follow the engine's refusals.
- **follow-ended (6:36, 9:21):** wards that were picked, not asked for in the packet.

Report against the log: all 675 plays in the hands' log appear in the "what your hands did" blocks. 29 plays were replaced within a second, all ordinary split-then-join pairs, and none misled a decision.

## The eight moments

| # | Moment | Verdict |
|---|---|---|
| a | Opening 0:00-4:00. The commander's list was done in order (mex 0:03 and 0:13, solars 0:20 and 0:29, plant 0:44). The produce list was followed in order (fav, fav, cv, cv, fav ×3, cv, flash ×3, cv). | Items 5, 6 |
| b1 | Pawn raid at home, 9:50-10:24 (spot_68 lost 10:03, spot_74 lost 10:10) | Items 7, 9 |
| b2 | spot_49 (E5) lost 12:16 to armrock nobody saw (truth: 2 armrock, armham, armwar, all unseen). The packet's "attack a party we outweigh at spot_49" had no party in the picture; the body held. | C |
| b3 | spot_52 (C5) lost 12:51 to a lone armham, first seen after the death. After go2 the packet left no clause for it. The 12:56 packet added one, and T1 attacked at 12:58; the armham paid. | C |
| b4 | spot_22 (B3) lost 18:51 to armpw_17109. group_K2 answered at 18:49 (+2 s); both Pawns died. | C |
| c1 | First window at 1.5x (4:00-5:00). The Blitz gathered at spot_63/64 as the packet said, and the Rovers roved. The commander's solar lists given here are what walled it in. | Items 1, 4 |
| c2 | First push (signal 10:15, then 10:05). group_K advanced to spot_40 at 10:07, then spot_27, but with 11 to 14 of its soldiers and its tail 756-1008 behind. The strung-out march is reported in the report and was not judged (low confidence). The flee footwork was reported ("held back by their own footwork") and the player turned it off with `lane` at 10:57. | Item 2 |
| d1 | Packet change at 7:21 (in the hands from 7:29). The decode landed group_K's way (0.99 each), and the body advanced to spot_46, answered parties and walked back to spot_56 from party_9, as written. The 7:44 report left the metal cap out. | Item 3 (the lab1 order in this turn is item 1) |

The fixed check at each moment (enemy in sight and recently seen, against the truth within 1,500, and our own state) found no unseen enemy that our units had seen and the report dropped. The gaps it did find are items 2, 3, 4 and 12.

## What Opus wanted that the harness has no lever for

- **7:21, 9:22, 10:19, 12:24, 13:04, 14:57, 15:14, 17:49:** "fights by this way, in order, never going back to a place it has reached". The menu offers one place per move, and the decode reads places one at a time. Nothing carries the order or closes a reached way-point (item 8).
- **12:24, 13:04, 14:57, 15:14:** "At each spot it kills his armck constructors first, then soldiers, extractors and solars". No move or tool sets the order of targets inside a fight.
- **18:34 packet (turn 18:26):** "the whole body attacks it together, surrounding it". There is no surround move.
- **18:26, 18:43, 18:53:** "A constructor never stands in a plant's exit lane." Nothing decides where a builder stands. The yard watchdog only reports, after 30 s.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
run/impedance_sweep.py MATCH --counts; run/impedance_sweep.py MATCH --seat 0 [--all --from 11:36 --to 11:56]
run/minutes.py MATCH; run/wake_read.py MATCH; run/raid_ledger.py MATCH; run/pro_baseline.py MATCH (no carded side)
run/commander_turns.py MATCH --calls / --notes
run/hands_window.py MATCH 1:30 2:20 constructor_18284 constructor_904 commander; 4:00 5:10; 7:28 8:20; 9:50 9:56 group_K; 9:56 10:32;
  10:10 11:10 group_K group_B1 group_X group_V; 11:40 11:52 group_K1 group_V; 12:03 12:20 group_H1; 12:44 13:00; 15:10 15:53 group_H1;
  17:48 18:02 group_G2 group_I2; 18:50 19:09 group_A2 group_F2
python3 over jevlog.rows (decode, plan, packets, audits, plays; the pass row at f3030; worlds_gate flags for party_24; call-row economy at f13920)
python3 over record-0.jsonl (created/finished/placed/destroyed/move_failed events, positions and store levels by second) and truth-0.jsonl (his army at each moment's turn)
python3 over strategist-0.jsonl (turn prompts: stuck lines, yard lines, the eco, enemy and to-win lines, the plays block against the log's plays)
```
One seat only, so no symlink directories were needed. MATCH = run/matches/1792195269-jev114-player-1/00.
