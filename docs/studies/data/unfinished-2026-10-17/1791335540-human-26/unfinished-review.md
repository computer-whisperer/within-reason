# Unfinished structures: 1791335540-human-26/00

This was a human 2v2 on Comet Catcher Remake 1.8 (realtime, 2026-10-06). Our seats were t2 (seat 0, G1) and t3 (seat 1, H8), each played by Opus 5.5 medium under an Opus commander. The opponents were [gecko]u6bkep and [gecko]Covariance, who played Rover and then Blitz swarms. Nobody stated a result: the record ends at 10:07 with "unexpected end of file". t2 lost its commander at 7:20 and its plant at 9:15, and gave its last constructors to t3 at 9:32. t3 was still standing. Seat 1 was read through a symlink directory for hands_window and floor.

frames.py, first line per seat (verbatim):
- `1791335540-human-26 seat 0, record to 10:07: frames 31 (decayed 2, finished 25, killed 4), metal in unfinished frames 329, build orders never started 9`
- `1791335540-human-26 seat 1, record to 10:07: frames 34 (decayed 2, finished 24, killed 6, unfinished at end 2), metal in unfinished frames 319, build orders never started 18`

floor.py's `aband` is 2 for seat 0 and 2 for seat 1, which equals frames.py's decayed count. Its `aband_m` (5 and 2) is far below the 181 and 62 metal that went into the decayed frames, because floor takes the last sampled share, after the decay.

## Tally (16 frame items, 648 metal)

| cause | n | metal |
|---|---|---|
| U4 builder died or fled, or the frame was killed | 11 | 458 |
| U9 harness fault | 2 | 84 |
| U3 player re-listed or moved the builder | 1 | 76 |
| U8 game end | 2 | 30 |
| U0, U1, U2, U5, U6, U7 | 0 | 0 |

Some frames have a second cause: U1 on 3 (armllt_30702, armsolar_29701, armllt_23890), U5 on 2 (armllt_4438, armmex_4899) and U9 on 1 (armllt_28950).

| fate | n | metal |
|---|---|---|
| killed | 10 | 375 (one of them, armllt_30702, by our own D-gun) |
| decayed | 4 | 243 |
| unfinished at end | 2 | 30 |

| type | n | metal |
|---|---|---|
| armllt | 9 | 404 |
| armsolar | 3 | 198 |
| armmex | 4 | 46 |

No plant, nano, advanced solar, moho, fusion or silo frame went unfinished.

What the player knew: 9 frames were in its report before the cause acted, 5 reached it only afterwards (an abandoned line, or a loss line in the fights table), 2 were never reported, and it dropped 1 itself.

## Items by cost

1. **3:16-4:09, seat 0, armsolar_12701 at H1 (105 of 155): U4, large, high.** An 11-armfav raid hit the home solar field at 3:37. The pick walked the helper constructor_7913 off at 3:39 (it died at 3:42), and constructor_2193 died on the frame at 3:45. The frame was shot from 68% down to 39%, then decayed by 4:09. Home had no guard, because the 3:27 turn had sent all of group_B west.
   - Evidence: `3:45 destroyed 2193 by armfav`; the 3:45 report: "11 at H1 ... killing our Solar Collector (armsolar), 2 of our Construction Vehicle".
   - Player knew: before. The 3:27 report showed "(28 % built)", but the raid was not in sight. The 4:21 report had "abandoned an unfinished armsolar (0% built) x1".
   - Fix: player.
2. **9:08-9:49, seat 0, armsolar_9171 at H1 (76 of 155): U3, small, high.** At 9:10 the player queued `["stop", "extractor spot_25", "armllt spot_25"]` to flee the 15 armflash at the plant. The stop landed at 9:22, when the solar stood at 49% and nothing was attacking it. The 9:26 transfer then gave the builder to t3, and the frame decayed.
   - Evidence: "9:22 constructor_30251: stopped what it was doing on your `stop`", then the transfer.
   - Player knew: before, and dropped it itself. The 9:10 report said "(6 % built)". The note: "Constructors fleeing south-east to spot_25/29/34".
   - Fix: player.
3. **9:45-10:06, seat 1, armllt_7141 at G6 (63 of 85): U4, small, high.** party_59 (14 or more armflash) killed the Sentry at 74% with its builder on it.
   - Player knew: before. The reports said 9:49 "(16 % built)" and 10:01 "(57 % built) ... party_59 (14 armflash) 718 away".
   - Fix: none.
4. **4:46-5:15, seat 1, armllt_4438 at G6 (61 of 85): U4, also U5, small, high.** A lone Sentry at a forward extractor. The metal store was 0 from 4:58, which halved its build rate, and 3 armfav killed it at 72% at 5:15. At full rate it would have finished at about 5:13.
   - Player knew: before. The 5:07 report said "(63 % built)" with "eco: metal 0 (+11.2/-18.4)". The 5:13 report said "armfav 3; killing our Sentry".
   - Fix: player.
5. **3:47-4:11, seat 0, armllt_29515 at F2 (60 of 85): U4, small, high.** Armfav killed it at 71% with the builder standing on it. frames.py's "builder left 4:04" is the 60% queue-ahead, not a departure: the builder stood at (5750,825) until 4:12.
   - Player knew: before. The 4:08 report said "(70 % built)" and "armfav 4; killing our Sentry (armllt)".
   - Fix: player.
6. **7:18-8:11, seat 1, armllt_23890 at F8 (60 of 85): U9, also U1, small, high.** At 7:36 the pick walked the builder home for 2 s ("took w2 of 1 at 0.04"). At 7:38 the list re-sent `build 5199 146 6088 5432`. The host's site search found that site taken by our own 71% frame and created armllt_24004 64 elmos north at (6080,5376), which finished. The 71% frame decayed beside the working builder.
   - Player knew: before. The 7:28 report said "(36 % built)". From then on the entry described the new frame as the same one (7:52: "(55 % built), ordered 13 s ago"). The 8:18 report had "abandoned an unfinished armllt (1% built) x1".
   - Fix: hands-rule.
7. **8:05-8:29, seat 1, armllt_28950 at G8 (59 of 85): U4, also U9, small, medium.** At 8:24 the pick walked the builder to spot_65 with 5 armflash 299 away, and armflash killed the frame at 69%. The frame was an unwanted third Sentry. The 7:57 queue's first step "armllt spot_70" was meant as the turret under way ("it finishes the turret there (67% built)"), but the hands played it as a new build: "8:04 next, queued: build a armllt at spot_70".
   - Player knew: before. The 8:18 report said "(44 % built)".
   - Fix: hands-rule.
8. **7:06-7:25, seat 0, armllt_23204 at F3 (47 of 85): U4, small, high.** The builder stayed on it, and party_42 (4 armfav) killed it at 55% during the raid that killed t2's commander.
   - Player knew: before. The 7:18 report said "(44 % built)".
   - Fix: none.
9. **10:02, seat 1, armllt_301 at H8 (25 of 85): U8, none.** The commander's list Sentry was started 5 s before the record ended.
10. **6:45-6:51, seat 0, armllt_30702 at F2 (24 of 85): U9, also U1, small, high.** Our own D-gun destroyed this frame. The 6:50 pick fired "D-gun party_18's nearest unit" at armfav 4777, 30 elmos from the frame, following the 5:38 packet's "its D-gun on them when they bunch".
    - Evidence: the destroyed event names our commander as the killer (`by 15349`); the 6:52 s row has `ff [[15349, 25208]]` and `dmg [[30702, 25242]]`.
    - Player knew: only after, and wrongly. The 6:52 report said "lost armllt to unseen at home x1".
    - Fix: hands-rule.
11. **5:45-6:21, seat 1, armmex_4899 at F8 (20 of 50): U4, also U5, small, high.** For 36 s the metal store was 0, with income 6.6 against use 6.6, so the frame rose about 1%/s. Armflash killed it at 41%. At full rate it would have stood by about 6:05.
    - Player knew: before. The reports said 5:48 "(2 % built)", 5:59 "(14 %)" and 6:19 "(40 %)", each with "eco: metal 0 (+6.6/-6.6)".
    - Fix: player.
12. **9:50-10:00, seat 1, armmex_17056 at H6 (20 of 50): U4, small, high.** party_59 killed the extractor at 41%.
    - Player knew: only after. The 10:01 report had "lost armmex to armflash in our half x1".
    - Fix: none.
13. **8:53-9:04, seat 0, armsolar_29701 at F2 (17 of 155): U4, also U1, small, high.** At 8:52 the pick started a solar 480 elmos from party_43 (4 armflash, growing to 13). At 8:56 it had the builder follow group_I_t2 home ("took w2 of 1 at 0.46"). Armflash killed the frame at 11%, and the builder died at 9:07.
    - Player knew: only after. The 8:59 report said "leaving its course (builds a Solar Collector ...)", with no share.
    - Fix: hands-rule.
14. **10:06, seat 1, armllt_5400 at H8 (5 of 85): U8, none, medium.** A second commander Sentry 80 elmos from armllt_301, started after a 10:04 D-gun play. It may be the queue's second "armllt home" step, or armllt_23890's mechanism again; the record does not settle which.
15. **9:23-9:25, seat 1, armmex_18665 at G8 (4 of 50): U4, small, high.** The takeover session's list walked a hurt builder to spot_77, where party_53 (10 armflash) was in sight 202 away. The frame was killed 2 s after it was placed.
    - Player knew: only after (9:37).
    - Fix: hands-rule.
16. **6:33-6:44, seat 1, armmex_12914 at G8 (2 of 50): U4, small, high.** The list step walked the builder into party_31 (6 armflash). It was hit as it placed the frame, the pick walked it off at 6:34 ("took w2 of 1 at 0.48"), and it died at 6:36. The frame decayed.
    - Player knew: only after. The 6:51 report said "6:44 abandoned an unfinished armmex at G8 (2% built): its builder was sent elsewhere".
    - Fix: hands-rule.

## Never started (23 items from frames.py's 27; 4 are not items)

- **U4, 13 orders (1,425 metal of types).** The builder died, or fled under fire, before it reached the site. Seat 0: 4:04, 7:26 (later re-sent, then cancelled by the player at 8:01), 8:33, and 9:07 (rejected the tick the builder died). Seat 1: 1:57, 2:18, 5:05, 6:03, 6:22, 8:33 (armbeamer), 9:15, 9:26, and 9:44 (an armhlt, the builder killed on a 2,300-elmo walk).
- **U3, 7 orders (630).** The player re-listed, cancelled or transferred the builder. Seat 0: 9:13 and 9:23, both by the transfer to t3. Seat 1: 3:18, 4:41, 5:12, and 6:57 (cancelled); and 9:02, an armbeamer at spot_77 queued by the earlier session and dropped when the 8:37 takeover re-listed the builder without seeing its list.
- **U2, 1 order (50).** Seat 1, 6:52, the extractor at spot_70. The hands marked it "taken as done" at 7:11 while the builder was still 280 elmos short, and the next step's Sentry order cancelled the walk. Today's lists.rs holds such a step instead ("the step waits for the frame").
- **U8, 2 orders (525).** Seat 1, 9:44 armhlt and 10:02 armllt: constructor_30251 was on a 4,150-elmo walk when the record ended.
- **Not items (4).** Seat 0 at 2:45 and 3:17: the builder helped the same building that another builder had started, and that frame was started or finished. Seat 0 at 1:46 and seat 1 at 2:41: the order was re-sent and started later.

## The three mechanisms

1. **Raider swarms at undefended build sites: U4, 11 frames, 458 metal, plus 13 never-started orders.** Two parts were recoverable:
   - t2's 3:27 decision to send all of group_B west, against the commander's "keep two Blitzes at home", just before the 3:33 raid (armsolar_12701 and armllt_29515, 165 metal).
   - Builds started or walked into parties already in sight (armsolar_29701, armmex_12914 and armmex_18665: 23 metal, plus 3 constructors).
2. **A builder returning to its own unfinished frame starts a new one: armllt_23890 and armllt_28950, 119 metal.** The host's `find_build_site` treats our standing frame as an obstacle, and a re-listed step meant as the build under way was played as a new build. The fix is a hands-rule: when the site holds our unfinished frame of that type, the build resumes it, and a step that names it is taken as done.
3. **Orders that land without regard to the frame: armsolar_9171 and armllt_30702, 100 metal.**
   - The flight "stop" landed at 49% on a solar no enemy was hitting, while the player had seen 6%. The stop's result could say the frame and its share.
   - The D-gun fired at a raider 30 elmos from our own frame. The D-gun move should keep its blast off our frames, and the fights line should report friendly fire as ours, not "unseen".

## Commands run

```
grep -n 'name=' MATCH/script.txt; ls MATCH/record-*.jsonl; tail -c 400 MATCH/record-{0,1}.jsonl
python3 run/frames.py MATCH --seat 0 [--json];  python3 run/frames.py MATCH --seat 1 [--json]
python3 run/floor.py MATCH;  python3 run/floor.py SCRATCH/seat1   (symlinks: record-0 -> record-1, jev-0 -> jev-1, strategist-0 -> strategist-1, terrain-0 -> terrain-1)
python3 run/hands_window.py MATCH 8:51 8:58 constructor_23714;  ... MATCH 6:45 6:51 commander
python3 run/hands_window.py SCRATCH/seat1 6:30 6:37 constructor_22164;  ... 7:33 7:39 constructor_5199;  ... 10:00 10:04 constructor_30251 constructor_8745
python3 run/impedance_sweep.py MATCH --seat 0 --from 9:05 --to 9:35 --all  (also 1:40-1:56, 7:28-8:30)
python3 run/impedance_sweep.py MATCH --seat 1 --from 6:45 --to 8:30 --all  (also 3:10-5:36, 8:27-10:07)
python3 run/commander_turns.py MATCH --calls
scratch tools: unit.py (a unit's events, cmds, d rows, positions and stores in a window), turn.py (a turn's prompt lines), unstarted.py (each never-started order's next 90 s)
sed -n on crates/ai-host/src/lib.rs (apply / find_build_site), crates/ai-host/src/engine.rs, crates/bot/src/brain/pianist/lists.rs
```
