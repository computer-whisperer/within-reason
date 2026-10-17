# Unfinished structures: 1791851365-human-33 (human-33)

Gecko Isle Remake v1.2.1, realtime, four seats of ours under an Opus commander: seat 0 = t2 Cortex D7, seat 1 = t3 Cortex F8, seat 2 = t4 Armada B7, seat 3 = t5 Armada G8. The opponents were two people, [gecko]thebluegecko and [gecko]u6bkep. **Lost.** The side surrendered at 22:53 ("t3 and t5 have no factory, t2 lost both advanced plants and its lab"), its units self-destructed at 22:54, and the record ends at 22:59.

## Denominators (frames.py, first line per seat, verbatim)

- `1791851365-human-33 seat 0, record to 22:59: frames 100 (decayed 9, finished 84, killed 6, unfinished at end 1), metal in unfinished frames 1762, build orders never started 50`
- `1791851365-human-33 seat 1, record to 22:59: frames 78 (decayed 3, finished 74, killed 1), metal in unfinished frames 1591, build orders never started 45`
- `1791851365-human-33 seat 2, record to 22:59: frames 113 (decayed 16, finished 92, killed 4, unfinished at end 1), metal in unfinished frames 293, build orders never started 32`
- `1791851365-human-33 seat 3, record to 22:59: frames 66 (decayed 5, finished 58, killed 3), metal in unfinished frames 184, build orders never started 39`

`run/floor.py` gives seat 0 `aband` 9, which equals frames.py's decayed 9. In all, 49 frames were unfinished, with 3,830 metal in them. Six of the frames destroyed at 22:59 were destroyed by the surrender's self-destruct (frames.py calls them "decayed"). Started after 20:59, they are U8. frames.py's never-started counts are too high: 30 of the orders it lists (7, 10, 7 and 6 by seat) did start, at a site the engine placed more than 300 from the commanded point. Others were finished by another builder with the same id or place. I classified 60 never-started orders (up to fifteen per seat by cost).

## Tally (49 frames)

| cause | n | metal in |
|---|---|---|
| U1 the pick sent the builder elsewhere | 12 | 128 |
| U2 the builder's own list moved on | 1 | 179 |
| U3 the player re-listed or moved the builder | 13 | 880 |
| U4 builder died or fled, or the frame was killed | 15 | 1780 |
| U5 economy | 0 (5 as `also`) | 0 |
| U6 the site | 0 | 0 |
| U7 helpers withdrawn | 0 | 0 |
| U8 game end | 7 | 863 |
| U9 harness fault | 1 (8 as `also`) | 0 |
| U0 unknown | 0 | 0 |

| fate | n | metal in |
|---|---|---|
| decayed (6 of them by the surrender) | 33 | 1968 |
| killed | 14 | 1779 |
| unfinished at end | 2 | 83 |

| type | n | metal in |
|---|---|---|
| corpun | 1 | 1274 |
| cormadsam | 2 | 423 |
| coradvsol | 4 | 403 |
| cornanotc | 2 | 402 |
| cormoho | 1 | 243 |
| corsolar | 3 | 215 |
| armsolar | 3 | 188 |
| corhllt | 1 | 123 |
| armrl | 6 | 121 |
| corestor | 1 | 116 |
| corllt | 3 | 109 |
| armllt | 8 | 105 |
| armrad | 4 | 38 |
| corwin / armwin / corrl | 1 / 2 / 1 | 29 / 24 / 16 |
| armmakr / armferret | 5 / 1 | 1 / 0 |

By seat: seat 0 had 16 frames (U3 6 for 493, U4 6 for 469, U2 1 for 179, U8 3 for 621). Seat 1 had 4 (U3 3 for 317, U4 1 for 1274). Seat 2 had 21 (U1 12 for 128, U3 2 for 46, U4 4 for 36, U8 2 for 83, U9 1 for 0). Seat 3 had 8 (U3 2 for 24, U4 4 for 1, U8 2 for 159).

**What the player knew:** the report had shown 27 of the frames standing, with their share, before the cause acted. The other 22 it learned of only afterwards, from an abandoned or lost line. None went unreported. The player dropped 13 frames itself (U3). Ten it dropped knowingly: 6 to clear a factory's exit lane, 3 advanced solars as economy triage and 1 turret replaced by a radar. Two came from re-lists that landed in the second the frame began. One, the seat 1 nano, was a frame the report had hidden. The abandoned lines give the share at decay ("1% built" for a nano left at 78% or 97%).

## Items ranked by metal (top twenty)

1. **20:51 · seat 1 · corpun_27143 (F6, pun_site)** · U4, also U5 · large (1274 of 1300). Five constructors raised the Agitator on a metal store at 0 from 18:56 (income about 20 a second). It took 2:35 to reach 98%, after a 26 s start delay at a blocked site. A lone Tzar killed it at 98%. Evidence: "17:57 constructor_12478: its corpun at F6 never started in the 8 s it stood at the site: the site is blocked"; created 18:16; "20:51 killed by corgol at 98%". Knew: yes, the 20:45 entry read "(96 % built) ... party_151 (1 corgol) 782 away", and the player sent group_J at the Tzar. Fix: player (funding and an unblocked start).
2. **22:59 · seat 0 · cormadsam_20188 (C7)** · U8 · large (315). A SAM at 90% when the surrender self-destructed its builder at 22:54. Knew: 47% at 22:48. Fix: none.
3. **22:59 · seat 0 · cormoho_13460 (D6)** · U8 · large (243). Started 22:27; destroyed in the surrender. Knew: 24% at 22:48. Fix: none.
4. **10:06 · seat 1 · cornanotc_4226 (F7)** · U3, also U9 · large (223 of 230). After the 9:06 queue-ahead ("next, queued: build a corllt at spot_52"), the hands and the report had constructor_6390 "walking to build a Guard (corllt) at spot_52 (E5), 2575 to go" (9:41, 10:00). In fact it stood 151 from the nano, which rose from 78% to 95%. The player re-listed it without a stop as "a pointless walk to spot_52". The list's first step ("10:06 help build #h49b (repair)") pulled it off at 97%. Knew: no, the frame was hidden. Fix: hands-rule and report.
5. **9:11 · seat 0 · cornanotc_15475 (D7)** · U2, also U9 · large (179). The 8:55 queue-ahead hid the frame, so the 9:02 entry read "about to build a Construction Turret ... ordered 1 s ago". The player's 9:02 list began with "cornanotc". "9:10 the step 'cornanotc' of its list is the build already under way: taken as done", and at 9:11 the corsolar step went out unqueued, which walked the builder off at 78%. Knew: no; after, the 10:36 line said "abandoned an unfinished cornanotc (1% built)". Fix: hands-rule.
6. **20:30 · seat 0 · coradvsol_9490 (D7)** · U3, also U5 · large (133). The player cancelled two advanced solars to finish two mohos stalled on an income of 29: "I cancelled the two advanced solars under way ... their half-built frames are lost" (20:17 `queue constructor_25847: stop`). Knew: yes, 30% built. Fix: player.
7. **9:06 · seat 0 · corhllt_4998 (C4)** · U4, also U1 · large (123). Seven Blitzes were on the builder. "8:59 PICK constructor_1696.go_spot_47 ... leaves the Twin Guard (corhllt) at about 60%, which decays", then a walk home. The Blitzes killed the frame and the builder at 9:06. Knew: 41% at 8:47; the Blitzes appeared after that report. Fix: player (a forward turret with no cover).
8. **20:30 · seat 0 · coradvsol_29999 (D8)** · U3, also U5 · large (122). The same 20:17 triage (`queue constructor_24513: stop`). Knew: 28%. Fix: player.
9. **6:47 · seat 0 · corestor_16891 (D7)** · U3, also U1 · large (116). The 6:12 pick put a storage in plant_19336's exit lane. The player dropped it: "I dropped that build, so the frame will decay out of the lane" (6:39 list beginning with stop). Knew: yes, "(54 % built) ... standing in plant_19336's exit lane". Fix: hands-rule (lane keep-out).
10. **22:46 · seat 0 · cormadsam_19764 (D7)** · U4 · large (108). A SAM begun under about 25 Wasps over home; "killed by corape at 31%". Knew: 15% at 22:30. Fix: player.
11. **22:59 · seat 3 · armsolar_3741 (G8)** · U8 · large (105). Started 21:38 on an empty metal store; destroyed in the surrender. Fix: none.
12. **22:43 · seat 0 · coradvsol_19111 (D7)** · U3, also U8 · large (92). With energy at 546: "constructor_5031 dropped its advanced solar" for plain solars (22:30 list beginning with stop). The surrender destroyed it 16 s later. Knew: 9%. Fix: player.
13. **7:29 · seat 1 · corsolar_28743 (F7)** · U3 · small (78). The builder stood in plant_25905's exit lane; "I cancelled its solar there" (7:19 stop). Knew: 17%. Fix: hands-rule (lane).
14. **8:23 · seat 0 · corllt_6860 (D4)** · U4 · small (76). Nine Grunts came. The pick kept the builder on the turret (no move above 0.5; go_home 0.32-0.49), and both died at 85%. Knew: 47% at 8:10, with "corak 9" in sight. Fix: player.
15. **19:47 · seat 0 · corsolar_2325 (C7)** · U4, also U1 · small (74). "19:46 PICK constructor_6718.go_home ... stepping away from party_175 (5 corshad) ... leaves the Solar Collector (corsolar) at about 50%"; the bombs killed the frame a second later. Knew: 48%. Fix: none.
16. **22:59 · seat 0 · corsolar_25160 (C7)** · U8 · small (63). Standing at the end.
17. **22:59 · seat 2 · armrl_23605 (A7)** · U8 · small (63). At 79% at the surrender.
18. **19:46 · seat 0 · coradvsol_22383 (C7)** · U4, also U5 · large (56). 15% after 37 s on an empty metal store; "killed by corshad at 15%". Knew: yes, "(15 % built) ... party_175 (5 corshad) 213 away". Fix: player (the anti-air ordered at 19:17 was not up).
19. **22:59 · seat 3 · armsolar_21001 (G8)** · U8 · small (54).
20. **17:22 · seat 2 · armllt_10535 (C3)** · U1 · small (41). "PICK constructor_14124.go_spot_44 conf=0.5 'stepping away from party_132 (2 armrectr, 1 armstump, under 1 turret: 1 armbeamer) ... leaves the Sentry (armllt) at about 50%'". Knew: 24% at 17:11. Fix: hands-rule.

The many small ones on seat 2 come from one pattern, the pick and the list fighting over a contested site (10 U1 frames for 118 metal, plus 2 frames killed there for 25). Five turrets at spot_38 (C4) were started between 11:56 and 12:32. Each time, the pick walked constructor_17073 to spot_44 away from Januses and Stouts, and 1-9 s later the list replayed "armllt spot_38" and began a new frame beside the old one. Then six radars and Nettles were started at spot_29 (C3/D3) between 17:23 and 18:08, three of them dropped by the pick in the second they began ("drops the Radar Tower (armrad) it was going to build"). Separately, seat 2's armferret_24104 was an anti-air turret dropped the second it began, because the pick fled the very bombers it was meant to answer. Seat 2's armmakr_15594 is the only U9: the blocked-site watchdog fired in the second the frame was created ("never started in the 8 s it stood at the site"). Seat 3's armmakr_31395 at 83% was most likely killed by the death blast of the converter beside it, which an unseen enemy destroyed a second earlier. The report called it "abandoned".

## Orders never started (60 classified: U3 32, U4 6, U6 6, U1 5, U2 5, U8 4, U5 1, U9 1)

- U3 (32): mostly the player's own re-lists, beginning with stop, or replacing a queued step before the builder arrived. Four of them are the Agitator #pun1 at F5 (seat 1, 17:05), moved south at 17:11 when two Tzars walked to F5.
- U6 (6): **the same named place gives the same point.** Seat 0 22:13: the corflak #fl1 at power_field was placed on (3016,7832), where #sam1's SAM began at 22:30; "22:32 the engine refused a corflak at C7: the site was bad". Seat 0 17:52: coradvsol #as4 was placed on #as5's frame and refused at 18:55. Seat 3: "armllt home" was given to two builders at one point, and one was refused 74 s later. Seat 1 17:25: the #pun2 site was blocked for 26 s (the watchdog).
- U2 (5): three are the list replacing the pick's choice 2-3 s later (seat 0 17:31 cormoho, seat 1 8:56 corllt, seat 2 6:17 armsolar), with nothing lost. Two are **"taken as done" on a build that had not begun.** At seat 0 7:48, "the step 'corlab lab_yard' of its list is the build already under way: taken as done" fired at 8:00 on a re-sent identical list while the builder was still walking. The second lab was not begun until 9:17, by another builder. At 9:10-9:12 three corsolar steps were taken as done by one solar that never started (outside the cap). Seat 0 8:55: the second nano was consumed the same way (item 5).
- U4 (6): builders killed on the way (seat 1 corpun #pun2 ×2 at F5, Twin Guards under Grunts).
- U1 (5): seat 0 22:33 corflak #fl2 (the pick walked the builder back under the Wasps); seat 2's spot_29 Nettles.
- U5 (1): seat 0 19:51, a moho at spot_64 queued behind a metal-starved moho (63% at 20:17, 64% at 20:34), then dropped by the 20:49 re-list.
- U9 (1): seat 3 17:30, the #g54 Guardian step replayed 9 s after that Guardian was finished, and refused.
- U8 (4): seat 0 cormoho, two cormadsam orders, and a seat 2 Nettle in the last minute.

## The three mechanisms

1. **A big defence frame starved and delayed under a known threat** (corpun_27143, 1274 metal). The 1300-metal Agitator stood 2:35 on a store at 0, after a 26 s blocked-site start. An unblocked start alone plausibly finishes it near 20:25, while the Tzar still stood about 1,100 away at F5 (the 20:30 report). Funding it first finishes it earlier still.
2. **The 60% queue-ahead hides the frame still being built** (cornanotc_15475 179, cornanotc_4226 223, plus the never-started second nano: 402 metal). Once the next step is queued, the hands and the report treat it as the builder's course. A re-list then either takes a step as done on the hidden frame and sends the next step unqueued, or replaces the list without the "build in progress finishes" protection. The fix: count any frame the builder is still building as the build in progress, and say "building X (N%), then Y queued".
3. **Builds in a factory's exit lane** (corestor_16891 116, corwin_18235 29, corsolar_28743 78, corrl_14191 16, armsolar_13067 29, armwin_8832 24: 292 metal in six frames). Picks, the lists' place-less steps and the named places put frames or builders in the lane. The blocked-lane wake line then made the player drop the frame every time. A keep-out for the lane in every placement keeps them.

Next by metal: the three advanced solars on seat 0 that the player cancelled knowingly (347 metal), because it had committed six advanced-solar ids and three mohos to an income of 29. Also seat 2's pick and list fight at contested sites (10 frames, 118 metal).

## Commands run

- `grep -n 'name=' MATCH/script.txt; ls MATCH/record-*.jsonl`; `tail MATCH/bot.log` (the surrender line); the record headers (seat to team, side).
- `run/frames.py MATCH --seat N` and `--json` for N = 0..3; `run/floor.py MATCH`.
- `run/impedance_sweep.py MATCH --seat N --from m:ss --to m:ss --all` around every item (seats 0-3).
- `run/hands_window.py MATCH 8:55 9:15 constructor_24513_t2` and `... 8:58 9:01 constructor_1696`. It printed no menu ratings for this game: the worlds_gate lines fall on other frames than the pass lines. I read the ratings, picks and move words with my own reader over `jevlog.rows(jevlog.seat(MATCH, N))` (scratch `tools/pick.py`).
- Scratch readers, all under the scratch directory: the report entries per turn (`tools/report.py`), the player's text per turn (`tools/said.py`), a frame's share, the stores and the builder's distance from the `s` rows (`tools/prog.py`), the never-started orders ranked by cost with their d/cmd/ev rows (`tools/unstarted.py`), the placed-site check (`tools/unstarted_filter.py`), and the hands' lines and queue calls per order (`tools/stops.py`).
- For seats 1-3 I made symlink directories (`seat1`, `seat2`, `seat3`: record-0, jev-0, strategist-0, terrain-0 pointing at seat N). In the end I read seats 1-3 through `--seat N` and my own readers, which take the seat directly.
