# Unfinished structures: 1791749780-human-31 (human game, 2026-10-11)

This was a realtime 3v2 on Comet Catcher Remake 1.8. We had three Cortex seats: seat 0 = t2 (north, home G1), seat 1 = t3 (south, home H8) and seat 2 = t4 (middle, home G4). Each was an Opus 5.5 medium player with Jev 1.13.0 hands, under an Opus 5.5 high commander. The opponents were [gecko]thebluegecko (Cortex) and [gecko]u6bkep (Armada). We **lost at 19:50**: the commander surrendered after the Tiger block wiped out t4 and t2, and the record runs to 20:09.

## Denominators (frames.py, first line per seat, verbatim)

```
1791749780-human-31 seat 0, record to 20:09: frames 76 (decayed 14, finished 60, killed 2), metal in unfinished frames 1396, build orders never started 33
1791749780-human-31 seat 1, record to 20:09: frames 89 (decayed 16, finished 60, killed 13), metal in unfinished frames 1389, build orders never started 27
1791749780-human-31 seat 2, record to 20:09: frames 88 (decayed 8, finished 76, killed 4), metal in unfinished frames 2818, build orders never started 26
```

`run/floor.py` reports `aband` 14 for seat 0, which equals frames.py's decayed count.

The game had 57 unfinished frames holding 5,603 metal, and 86 build orders that never started. Of those 86, I classified 41 and skipped 15 that did start (see below).

## Tally

| cause | n | metal | | fate | n | metal |
|---|---|---|---|---|---|---|
| U1 the pick sent the builder elsewhere | 5 | 69 | | decayed | 38 | 2,467 |
| U2 the list moved on | 1 | 0 | | killed | 19 | 3,136 |
| U3 the player re-listed or stopped it | 15 | 684 | | unfinished at end | 0 | 0 |
| U4 builder died or fled, frame killed | 29 | 3,679 | | | | |
| U5 economy (as the main cause) | 0 | 0 | | | | |
| U6 the site | 0 | 0 | | | | |
| U7 helpers withdrawn | 0 | 0 | | | | |
| U8 game end | 2 | 542 | | | | |
| U9 harness fault | 5 | 629 | | | | |
| U0 unknown | 0 | 0 | | | | |

The fate column follows frames.py, with two exceptions to how it reads:
- Eight of the 38 "decayed" frames were really kills by unseen enemies. The `dmg` field of the `s` rows shows the hits (corllt_21173, corhlt_16044, corllt_11481, cornanotc_17937, corllt_9958, cormex_2359, corllt_9823, cormex_16194).
- The two U8 Wardens died to the surrender's self-destruct.

U5 was never the main cause, but it was the "also" on 5 frames, including the three costliest kills.

| type | n | metal | | seat × cause | n / metal |
|---|---|---|---|---|---|
| coravp | 1 | 2,366 | | seat 0: U4 7 / 799, U9 4 / 505, U3 3 / 76, U1 1 / 16, U2 1 / 0 | |
| corhlt | 6 | 1,152 | | seat 1: U4 15 / 429, U8 2 / 542, U3 7 / 241, U9 1 / 124, U1 4 / 53 | |
| coradvsol | 4 | 673 | | seat 2: U4 7 / 2,451, U3 5 / 367 | |
| corllt | 19 | 613 | | | |
| corsolar | 7 | 286 | | fix class: player 28 / 3,726, none 11 / 835, hands-rule 10 / 673, tool 8 / 369 | |
| cormex | 15 | 209 | | | |
| cornanotc | 1 | 152 | | | |
| cormadsam | 1 | 119 | | | |
| corrad | 1 | 23 | | | |
| corrl | 2 | 10 | | | |

**What the player knew.**
- For 43 of the 57 frames, the report had shown the frame, or the walk to build it, before the cause acted.
- 14 were told only afterwards, by an "abandoned" or "lost" line.
- None were never told.
- 15 frames were dropped by the player's own order (U3). Of these:
  - 5 began during the player's own 8–23 s realtime turn, so the stop it wrote against a walk landed on a frame.
  - 1 (coradvsol_21260) was dropped without meaning to.
  - The rest were deliberate: duplicate turrets, advanced solars in an energy stall, and turrets in the Tigers' path.

## Items, costliest first (20 of 57)

1. **17:42, seat 2, coravp_17359 (G4), U4 + U5, killed at 91%, 2,366 metal.**
   - What happened: t4's advanced vehicle plant (#avp1) was begun at 14:45 by eight constructors with the metal store at 0 of 1650 on +27 income. It crawled for 2:57: 7% at 15:07, 47% at 16:16, 90% at 17:30. The Tiger block reached G4 at 17:21, killed its builder at 17:30 and the frame at 17:42.
   - Evidence: frames.py stores min metal 0.0; "17:21 enemies_near: party_134 (1 corraid, 6 correap) 584 away"; 17:42 destroyed by correap.
   - Player knew: yes, every report from "0 % built" to "90 % built". The commander's surrender note says "our AVP began 14:32 and fell at 90%".
   - Fix: player (economy). Cost: large, the single largest loss of the game (42% of its unfinished metal). I did not judge it `game`, because the base fell with it.
2. **19:06, seat 0, corhlt_7461 (G2), U4 + U5, killed at 75%, 360.**
   - What happened: t2's home Warden was built from 17:20 with the metal store at 0. It was at 57% by 18:48, with three helpers from 18:10.
   - Evidence: the picks walked the helpers away from the Tigers at 18:53–18:57, and 12 correap killed the frame at 19:06.
   - Player knew: yes. The 18:36 entry read "building a Warden (corhlt) at home (G2) (29 % built)", and the Tiger parties showed as "moving toward our side".
   - Fix: player. Cost: large.
3. **12:26, seat 0, coradvsol_13353 (G2), U9 + U1, decayed from 84%, 311.**
   - What happened: constructor_16072's own advanced solar finished 160 away at 12:25. The finished-event handler took it for constructor_7314's build. 7314's list had been cancelled at 11:43, so the hands dropped its queued solar step at 12:26 with a `stop`, and that stop also ended the build in progress ("99 % built" in the hands' picture at 12:24).
   - Then the idle builder was walked to spot_11 under the bomber raid (w1: "constructor_7314 idle, doing nothing"). The frame survived the bombs at 62% and decayed while the builder put up a cormex 200 away on the 12:48 list.
   - Evidence: the 12:26 line "dropped the step 'corsolar' its old list had queued behind the build it was on"; `stop 7314`.
   - Player knew: before (12:16: "93 % built"). Afterwards it got only "abandoned an unfinished coradvsol (0% built) x1" at 13:47.
   - Fix: hands-rule. Cost: large.
4. **20:09, seat 1, corhlt_27502 (H7), U8, 60%, 288.** A last-stand Warden begun at 19:00. The commander surrendered at 19:50 and every unit self-destructed at 20:03. Player knew: yes (19:59: 56%). Fix: none. Cost: large.
5. **20:09, seat 1, corhlt_60 (H8), U8, 53%, 254.** The shared Warden #w72, begun at 19:31 and ended by the same self-destruct. Fix: none. Cost: large.
6. **10:55, seat 2, coradvsol_21260 (F5), U3, decayed from 64%, 237.**
   - What happened: the player meant to pull constructor_3826 off its queued Guard at spot_49 ("I pulled constructor_578 and constructor_3826 off their builds by spot_49"). Its `["stop", "assist"]` dropped the build in progress instead: the advanced solar, which the entry it had just read showed "(62 % built)". The frame then stood for 87 s beside the plant that the builder was helping.
   - Player knew: yes, and dropped it unintentionally. The tool result said "its build in progress dropped".
   - Fix: tool (a stop should name the build it drops, with its share). Cost: large.
7. **14:49, seat 0, corhlt_16044 (F3), U4 + U5, "decayed" at 42%, 202.**
   - What happened: a Warden at the place the report itself named `shelling_G_t2`. It was built at metal 0 by its builder and a helper, who both stayed on it.
   - Evidence: unseen Quaker fire hit it for 421, 459 and 728 (dmg, 14:45–14:50).
   - Player knew: yes. The 14:41 entry read "39 % built", and the shelling wake read "a cormart ... 62 hits in 75 s". The report afterwards called it "abandoned ... (15% built)".
   - Fix: player. Cost: large.
8. **12:25, seat 0, cornanotc_17937 (G2), U4 + U1, 66% to 2% in one sample, 152.**
   - What happened: 8 corshad bombers hit it for 343. The pick walked the builder off in the same second ("leaves the Construction Turret (cornanotc) at about 10%"), and what was left decayed.
   - Player knew: yes (12:16: 59%; bombers noted at 11:17).
   - Fix: none. Cost: large (nano).
9. **10:40, seat 1, corsolar_25015 (G8), U9 + U3, decayed from 83%, 124.**
   - What happened: the commander's solar finished 181 away at 10:02, and the hands dropped constructor_18018's task while the engine kept building. The report then read "finishing an order; standing in plant_3186's exit lane" at 10:15 and 10:33, and the list never advanced.
   - The player stopped it to clear the lane: "Its new list starts with stop, which drops whatever it was finishing". The stop landed at 10:40 with the frame at 82%.
   - Player knew: never as a frame. Afterwards: "abandoned an unfinished corsolar (1% built) x1".
   - Fix: hands-rule. Cost: large.
10. **12:11, seat 0, cormadsam_27505 (G1), U9 + U1, decayed from 34%, 119.**
    - What happened: a 0.06-confidence pick walked the builder off ("leaves the SAM (cormadsam) at about 30%"). Two seconds later its list re-played "cormadsam spot_6 #sam2" as a new build, and the engine placed a second SAM frame (22622) 115 away. The first decayed; the second finished at 13:49.
    - Player knew: yes (11:56: 12%).
    - Fix: hands-rule. Cost: large.
11. **9:45, seat 1, coradvsol_19977 (H8), U3 + U5, 25%, 92.** Dropped on purpose in an energy stall: "energy 1/1750 (-96) STALLING ... I dropped both, at 4% and 18%" (with coradvsol_12110, 33). Fix: player. Cost: large.
12. **11:56, seat 1, corsolar_1136 (H7), U4, killed at 61%, 92.** Bombers. t3 had no anti-air: its first Thistle began at 11:54 and died at 11:59. Player knew: yes (11:53: 59%). Fix: player. Cost: small.
13. **10:59, seat 1, corllt_22402 (E6), U4, killed at 93%, 84.** The Blitz mass killed the builder at 10:57 and then the frame. Player knew: yes (10:45: 59%, with 18 enemies 747 away). Fix: player. Cost: small.
14. **8:23, seat 0, corllt_26308 (E2), U9, decayed from 82%, 74.**
    - What happened: two Guards finished 181 and 192 away (8:22, 8:23). The same finished-event match promoted the builder's queued "help #m20" step (8:23 `guard 7314 18724`). constructor_16072 then began a second Guard 64 from it at 8:24.
    - Player knew: yes (8:06: 71%).
    - Fix: hands-rule. Cost: small.
15. **12:05, seat 1, corllt_9958 (E8), U4, "decayed" at 74%, 67.** The Guard at spot_79, beside his Beamer, took 112 + 112 + 225 from unseen fire with its builder on it. The 12:03 entry said "finishing an order" (the hands had lost the task to a Guard that finished 242 away at 11:55). Fix: player. Cost: small. Confidence: medium.
16. **6:54, seat 1, corllt_4488 (F8), U4, killed at 58%, 52.** One of eight frames killed rebuilding spot_70, spot_75 and spot_79 under the Rover and Blitz raids from 2:13 to 8:19 (126 metal in all). Fix: player. Cost: small.
17. **4:59, seat 2, cormex_16194 (F5), U4, 91%, 46.** A Blitz killed the builder at 4:56, then the frame. Fix: none. Cost: small.
18. **11:08, seat 2, corllt_5654 (E4), U3, 49%, 44.** Pulled off on purpose ("I pulled constructor_14857 off the Guard it was building at spot_37"). It grew from 16% to 49% between the report and the stop landing. Fix: player. Cost: small.
19. **9:42, seat 1, cormex_9648 (E7), U4, killed at 86%, 43.** The player's own words at 9:39: "three Blitzes are next to the constructor finishing spot_66 (77%). Our main body ... too far to help". Fix: player. Cost: small.
20. **11:12, seat 0, corsolar_27034 (G1), U3, 28%, 42.** "Energy is full ... I cancelled the two solars ... about to start." The frame began at 10:54, while the 23 s turn was running. Fix: tool. Cost: small.

The other 37 frames are in `unfinished.json`. Notable among them:
- the seat 1 commander's solar/reclaim thrash at 2:37–2:43 (two frames, U1 + U9: each build pick placed a new solar at a new site);
- the duplicate Wardens at spot_75 that the player dropped at 16:40 (its untagged steps gave each builder its own frame);
- cormex_31095 (U9: a frame the builder began under its old list was taken as the start of the new list's step, and the stale-queue drop stopped it).

## Never started (41 classified: U3 29, U4 6, U1 4, U6 2)

- Seat 0, 18:24: the #w17 Warden at H3, ordered to three builders. The picks walked each home from the Tigers before it began, and all three died between 18:40 and 18:52 (U4).
- Seat 0, 11:22: cormadsam at G1. "The engine refused a cormadsam at G2: the site was bad" (U6, rejected).
- Seat 0, 17:54: constructor_18724 and constructor_28710 played their t2 lists of 14:20 again once t4 gave them back. The lists had survived the round trip through t4. The player's 18:12 list replaced them (U3).
- Seat 1, 15:46: corhlt at spot_70. The player's untagged Warden step went to two builders. 18018 stood 142 short of its site while 25147's frame stood there, and the player dropped it at 16:40 (U3).
- Seat 1, 2:41, and seat 2, 5:51: the commanders' solars. The picks sent the commanders to reclaim and to attack instead (U1).
- Seat 2, 11:44: cormadsam at G4. The builder died to the bombers in the same second (U4).
- Seats 1 and 2: 19 orders (seat 1: 7 Guards; seat 2: 10 Guards, a Thistle and a nano turret) were ended by the player's own list beginning with "stop", or by a list replacement that dropped the queued step (U3). Seat 1's 8:04 and 9:36 Guards ended with their builders' deaths (U4), and its 15:11 Guard at spot_55 with the picks' walks home from the Tigers (U1).
- Seat 2, 4:44: cormex at spot_37 (E4). It was refused because his extractor stood there (U6).

Skipped, not items (15 orders):
- t4's AVP: 7 orders that were the #avp1 id's helpers.
- The #w16, #agi and #w72 helpers (3).
- Four orders begun at the engine's placed site, 311–388 from the point ordered (2602's Thistle, 25147's solar, 12885's #w72 Warden, 8427's Thistle); frames.py measures 300 from the ordered point.
- One step taken as done (the commander built that Guard).

## The three mechanisms that would have kept the most metal

1. **Long builds left crawling at an empty metal store in the path of the push** (2,928 metal: coravp_17359, corhlt_7461, corhlt_16044). Each stood 62–177 s at metal 0 and was killed unfinished. No hands rule saves these. The lever is the player's spending; a report line giving each frame's time to finish at the current income would help it see that.
2. **The finished-event match by type within 250** (510 metal: coradvsol_13353, corllt_26308, corsolar_25015, cormex_31095). `UnitFinished` in `crates/bot/src/brain/pianist/mod.rs` promotes the queued task of, or clears the task of, any builder whose started build is the same type within 250 of the finished unit. A related path: any frame a builder begins marks its task started, whatever its type. Matching by the frame's unit id would have kept all four. It also stops the report from calling a live build "finishing an order".
3. **A queue's leading "stop" drops the build in progress even when it was written against a walk or a queued step** (445 metal: coradvsol_21260 237 and corsolar_25015 124 were not meant; corsolar_27034, cormex_22762, corllt_31669, corsolar_21587 and corllt_13504 began during the player's turn).
   - In realtime the stop lands 8–23 s after the report the player read, and frames grow in that time (14% to 46% on seat 2's duplicate Guards).
   - A stop that names what it drops and its share in the result, and that does not drop a frame begun after that report unless the player names it, would have kept 237 metal and shown the player the rest.

Other report findings:
- The "abandoned an unfinished X (N% built)" line gives the share at death, so coradvsol_13353 at 84% was reported as "0% built".
- The same line, "its builder was sent elsewhere", was printed for six frames killed by unseen fire with their builders on them (corllt_21173, corhlt_16044, corllt_11481, corllt_9958, cormex_2359, corllt_9823).

## Commands run

```
grep -n 'name=' MATCH/script.txt; ls MATCH/record-*.jsonl; tail -c 400 MATCH/record-0.jsonl; cat MATCH/impedance-review.md
run/frames.py MATCH --seat 0|1|2 (and --json); run/floor.py MATCH
run/impedance_sweep.py MATCH --seat 0|1|2 --all (whole game, read by window)
run/hands_window.py MATCH 12:22 12:30 constructor_7314; run/hands_window.py MATCH 3:47 3:55 commander_t2
run/commander_turns.py MATCH --notes; grep surrender MATCH/commander.jsonl
scratch readers (scratch/tools): win.py (record d/cmd/ev/s rows per builder and window), dmg.py (dmg and health of every unfinished frame),
  pick.py (jev-N worlds_gate lines, choice answers and plan rows at a clock), turn.py / said.py / knew.py (report entries, player text and calls per turn),
  ns.py (decision rows after each never-started order), a placed-site and created-by-anyone check of the never-started orders,
  a same-type-finish-within-250 check of every unfinished frame
source read: crates/bot/src/brain/pianist/mod.rs (take_queued, list_replaced, the UnitCreated/UnitFinished handlers), picture.rs (task_course)
```

MATCH = run/matches/1791749780-human-31/00. I made symlink directories for seats 1 and 2 in the scratch directory, but the per-seat picks were read straight from jev-1 and jev-2 with run/jevlog.py.
