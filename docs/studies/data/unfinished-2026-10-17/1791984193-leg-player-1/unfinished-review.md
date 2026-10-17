# Unfinished structures: 1791984193-leg-player-1 (match 00)

**The game.** An arena game on Comet Catcher Remake 1.8. Our one seat played Legion from the W corner (the Opus 5.5 medium player, Jev (jev-latest) hands, layers news,same,fuse,openers,split,tick,places,again,near) against BARb hard_aggressive (Armada). We **won at 25:56**.

**Denominator, seat 0** (frames.py first line, verbatim, the version of 11:35 today):
`1791984193-leg-player-1 seat 0, record to 25:56: frames 187 (decayed 12, finished 164, killed 9, unfinished at end 2), metal in unfinished frames 1070, build orders never started 21`

- `floor.py` prints aband 12, which equals frames.py's decayed 12. Its aband_m (123) is not frames.py's decayed metal (467). The two use different measures, and I did not reconcile them.
- The frames.py I first ran, before its 11:35 change, listed 28 orders never started. The current version matches each order to the engine's `placed` site.
  - 6 of the 7 orders it dropped had their frame start at the placed site, 300-714 from the ask: 10:28 and 15:26 constructor_27740 legvp; 13:52 constructor_27659, 19:28 constructor_3137 and 24:07 constructor_23595 legadvsol; 19:37 constructor_1281 legavp.
  - The seventh, 14:03 constructor_1251, is now counted as a re-send of its 12:43 order.
  - I struck nothing more by hand.

## Tally (frame items, one cause each)

| cause | n | metal in |
|---|---|---|
| U3 the player re-listed or moved the builder | 5 | 352 |
| U4 builder died or the frame was killed | 15 | 446 |
| U8 game end | 2 | 272 |
| U9 harness fault | 1 | 0 |
| U0, U1, U2, U5, U6, U7 | 0 | 0 |

| fate | n | metal in |
|---|---|---|
| decayed | 12 | 467 |
| killed | 9 | 331 |
| unfinished at end | 2 | 272 |

**Six of the twelve "decayed" frames were shot down by an unseen enemy, not decayed:**
- The evidence is the `dmg` rows: one hit of 300 (the E3 armguard's reach), or 800 from a Fatboy, with nothing of his in sight. The frames are leglht_5486, legmex_7170, leglht_25003, leglht_17119, legmex_29730 and legrad_25767.
- A seventh, legmex_17819, decayed after its builder was shot.
- The hands' report called five of these "abandoned ... its builder was sent elsewhere and the frame decayed".

| type | n | metal in |
|---|---|---|
| legadvsol | 4 | 468 |
| leglht | 7 | 203 |
| legmex | 8 | 174 |
| legdtr | 1 | 122 |
| legnanotc | 1 | 78 |
| legrad | 1 | 25 |
| legsolar | 1 | 0 |

**What the player knew:** 19 frames before the cause acted, 3 only after, 1 never. The player dropped 5 frames itself (the U3 items).

## Items by cost (20 of 23)

1. **24:41 legadvsol_15447 A8, U8 (also U5), 232 metal.** Started 75 s before the end and built at about 0.7% a second with the metal store at 1-2%; 50% at the end. The entries from 24:47 on listed it. Fix: none. Cost: large (no fault).
2. **13:55 legdtr_24253 D6, U4, 122 metal, killed 14:19 by armmav at 42%.**
   - The Dragon's Jaw came from the player's 11:56 turtle list. The army (21 units, 6120 metal) was walking north on the player's 13:56 route. The armmav, armwar and 2 armrock party (about 1160 metal) came onto the turret line.
   - Evidence: the 14:12 entry, "(33 % built) ... party_42 (1 armmav, 2 armrock, 1 armwar) 711 away"; the 14:12 note, "Army ... turns to kill it first"; the frame died 7 s later.
   - The player knew (the 13:56, 14:06 and 14:12 entries). Fix: player. Cost: large.
3. **11:47 legadvsol_3770 B8, U3, 116 metal, decayed 13:21.**
   - The list step "legadvsol #as3" had no place. The hands asked for (1752,5746), inside plant_16133's exit lane, and the engine placed the frame 32 elmos into the lane.
   - 12:06: the player's queue `['stop','legadvsol solar_field #as3',...]` kept the id, so at 12:15 the builder went back with "help build #as3 (repair)".
   - 12:16: a re-list with fresh ids (#sf3); the builder left at 12:24. constructor_2996's queued help never arrived.
   - The player knew (the 12:06 entry: "13 % built ... standing in plant_16133's exit lane"). It dropped the frame on purpose (its 12:06 and 12:16 notes). Fix: hands-rule. Cost: large.
4. **11:24 legnanotc_18073 B8, U3, 78 metal, decayed 12:21.**
   - The 11:36 entry: "building a Construction Turret (legnanotc) ... (19 % built) ...; standing in plant_16133's exit lane since 12 s ago".
   - The 11:36 queue `['stop', ...]`, with the note "nano dropped".
   - The frame itself was 16 elmos clear of the lane. Its builder was in the lane.
   - Fix: report (say the frame's lane apart from the builder's). Cost: large (nano).
5. **12:05 legadvsol_21461 B8, U3, 60 metal, decayed 12:58.** Dropped by the 12:06 and 12:16 re-lists. The frame only touched plant_27255's lane at a corner. Its builder stood in the lane: the 12:06 entry said "(1 % built) ... standing in plant_27255's exit lane since 34 s ago". Fix: report. Cost: large.
6. **12:05 legadvsol_9057 B8, U3, 60 metal, decayed 12:58.**
   - Dropped with the other two. This frame was clear of every lane, and its builder's entry had no lane line. The player's 12:06 note, "advanced solars were going up in plant exit lanes", generalised.
   - Fix: player. Cost: large.
   - Items 3-6 came while energy capped spending: the store was at 5-8%, and the player's notes said "Energy 166 is still the cap on spending 2548 metal" and "energy STALLING".
7. **11:24 leglht_23841 D4, U4, 50 metal, killed 11:47 by armpw at 71%.** His second wave's Pawns killed it while its near-dead builder (18-27%) kept building. The 11:46 entry showed "party_32 (3 armpw) 215 away". Fix: none. Cost: small.
8. **25:41 leglht_1347 F6, U8, 40 metal.** The pick's Pharos at spot_55, started 15 s before the end. Fix: none. Cost: small.
9. **19:30 legmex_29730 E4, U3 (also U4), 38 metal, destroyed 19:50 at 77%.**
   - The 19:39 takeover re-list dropped it on purpose. Its note: "the spot_37 extractor frame [is] inside the E3 Gauntlet's 1220 reach".
   - The stop landed at 19:47. A 300 hit from out of sight came 3 s later.
   - The player knew. Fix: player. Cost: small. Confidence: medium (U3 against U4).
10. **12:08 leglht_7860 E5, U4, 35 metal, killed 12:23 by armpw at 50%.** The builder survived (the pick walked it off at 12:24). The 12:16 entry showed it at 30% with no enemy near. Fix: none. Cost: small.
11. **11:01 leglht_5486 E4, U4, 32 metal, frames.py "decayed" at 46%.**
    - It was shot (dmg 31, 134, 176) by the armwar block that had been in sight at E4 since 10:59. Its builder was killed at 11:16 by an armwar.
    - Its list kept it building forward of the army's 10:59 fallback. The 11:18 report's "abandoned (15% built)" line was wrong.
    - Fix: player. Cost: small.
12. **16:06 legmex_10058 E6, U4, 32 metal, killed 16:28 at 65%.**
    - Two Pawns hit the builder every second from 16:16.
    - The pick kept it on the build: hands_window shows "change=0.85..0.89", with the best move go_spot_61 at 0.29-0.35, so no play.
    - The builder died at 16:23. The 16:18 report said "killing our Metal Extractor (legmex), our legcv now".
    - Fix: hands-rule. Cost: small.
13. **6:14 legmex_30757 C6, U4, 28 metal, killed 6:32.**
    - The builder was at 38% health with 3, then 6, Pawns 670 away. The 6:11 entry's best_rated move was the walk home at 0.46.
    - The pick kept the build (change 0.76-0.90, best move 0.27-0.40). Builder and frame died together at 6:32.
    - Fix: hands-rule. Cost: small.
14. **20:59 legrad_25767 E5, U4, 25 metal, frames.py "decayed" at 42%.**
    - An 800 shell (dmg row at 21:06) from his armfboy, 553-612 away. The Fatboy was in the truth file but not in our sight.
    - The 21:14 line "abandoned an unfinished legrad (42% built)" was wrong. Fix: none. Cost: small.
15. **19:04 leglht_25003 D4, U4, 24 metal, "decayed" at 35%.**
    - A 300 hit at 19:15, 1143 from the E3 armguard. It came from the player's 16:28 list ("leglht spot_33").
    - The player was told only by the false "abandoned (35% built)" line. Fix: report. Cost: small.
16. **5:11 leglht_2014 B5, U4, 22 metal, killed 5:26 by armflea.** The same Flea had killed the extractor beside it. The 5:22 report said "killing our Pharos now". Fix: none. Cost: small.
17. **10:49 legmex_29133 E6, U4, 18 metal, killed 10:59 by a lone armpw.** The builder survived. The player was told only as the frame died. Fix: none. Cost: small.
18. **19:24 legmex_17819 D4, U4, 18 metal, decayed 19:52.**
    - The player's 19:09 queue sent constructor_1251 to spot_31, where constructor_2996 had just been shot to 10%.
    - 1251 took 300 hits from 19:17 and died at 19:32 (attacker: nobody).
    - The 19:52 line "its builder was sent elsewhere" was wrong. Fix: player. Cost: small.
19. **18:54 legmex_7170 D4, U4, 16 metal, "decayed" at 33%.**
    - A 300 hit. constructor_2996 took 300 a hit from 18:57 and died at 19:10.
    - The 18:59 entry: "health lightly hurt (77%); enemies_near: party_59 (2 armck)". The same report: "his, in E3 ...: 1 armguard".
    - Fix: report. Cost: small.
20. **9:36 legmex_1774 C4, U4, 13 metal, killed 9:43 by a lone armpw**, which the 9:42 entry named, 237 away. Fix: none. Cost: small.

**The other three:**
- legmex_16248 (B5, killed by the Flea at 22%, 11 metal; U4, none).
- legsolar_4881 (C8, 0%, 0 metal; U9 also U1). At 7:40 the list's build and the pick's walk ("its list step 'legsolar' waits for it") played in the same second. The frame was laid, then the builder walked off. Fix: hands-rule.
- leglht_17119 (D4, 0%, 0 metal; U4). It was destroyed in the second it was laid, under the same unseen fire. Never reported.

## Orders never started (21: U9 11, U3 5, U1 2, U8 3; none cost metal)

- **U9, "taken as done" on an unbegun order (2):** 3:40 commander leglht A7, and 8:11 constructor_5624 legmex C6. A re-sent list began with the order the builder had and had not started. The hands took it as done ("the step 'leglht spot_68' ... is the build already under way: taken as done") and played the next step. The Pharos and the extractor were never built. H-HANDS-LISTS, 2026-10-16, has fixed this since.
- **U9, two lists name one spot (5):** 8:39 constructor_2996 spot_71, 9:17 constructor_1251 spot_50, 10:05 constructor_1251 spot_36, 20:00 constructor_23698 spot_55, 20:25 constructor_23595 spot_79.
  - Another list's builder had begun the spot. The late arrival idled.
  - The hands said "the engine refused a legmex at ...: the site was bad" for 3 of them. That is false: our own frame stood there.
  - The 90 s refusal it set then skipped the spot's Pharos step ("the extractor at spot_71 was refused lately").
- **U9, same-tick orders placed on one site (4):** 11:07 constructor_1251 (#as2, with 2996), 12:26 constructor_27740 (#sf1, with 1281 and 18227), 12:43 constructor_1251 (#sf7 and #sf9, with 3137), 24:07 constructor_24498 (with 23595). The engine put every order on one site. One builder built and the others assisted, so advanced solars ordered in parallel went up one at a time. The hands twice called these "refused: the site was bad".
- **U3 (5):** 5:47 constructor_4634, where the pick was replaced by a list landing a second later; 9:21 constructor_17003 (the 9:32 re-list); 10:20 constructor_27740 (the 10:21 queue for the third plant); 16:36 constructor_17003 C3 (re-listed on purpose, away from his Beamer); 19:45 constructor_24498 (the 19:39 drop).
- **U1 (2):** 19:18 constructor_23698 spot_46 and 22:09 constructor_8343 spot_14. In both, the pick sent another constructor to the spot first, while the list's builder walked a long way.
- **U8 (3):** 25:10 constructor_3166 leglht, 25:28 constructor_5527 legmoho, 25:35 constructor_25372 legmoho.

## Three mechanisms

1. **Say whether the frame is in the exit lane, apart from the builder.** Also keep an unplaced advanced-solar footprint out of the lane.
   - Items 18073, 3770, 21461 and 9057: 314 metal in four advanced-economy frames.
   - The player dropped all four as "in the lanes", at the energy cap. Only 3770 overlapped a lane.
2. **Name unseen fire and a remembered turret's reach on the builder's entry and on the loss line.**
   - Items 7170, 25003, 17119, 17819, 29730 and 25767: 121 metal of frames, plus constructors 2996 and 1251 (250).
   - The entries named two armck. The hands called the kills "abandoned", and the player sent a third constructor to spot_31.
3. **Hold or pull a forward build that has a raid party in contact.**
   - Items 16248, 2014, 30757, 1774, 29133, 23841, 7860, 24253 and 10058: 331 metal.
   - For 30757 and 10058 the pick read change at 0.76-0.90 with no move over its bar, and the builders died.
   - The largest, the Dragon's Jaw 24253 (122), died with the army walked off the turret line.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...batch.json..."
python3 run/frames.py MATCH --seat 0          (twice: before and after the script's 11:35 change)
python3 run/frames.py MATCH --seat 0 --json
python3 run/floor.py MATCH
python3 run/impedance_sweep.py MATCH --seat 0 --from {11:00,5:40,5:50,8:00,13:40,16:30,18:20} --to ... [--all]
python3 run/hands_window.py MATCH 7:38 7:42 constructor_3166 | 6:24 6:32 constructor_1595 | 16:15 16:23 constructor_5624 | 5:46 5:50 constructor_4634 | 20:13 20:16 constructor_23698
python3 run/commander_turns.py MATCH --calls
the turn prompts of strategist-0.jsonl (turns 12-142) grepped per builder; every 'abandoned' line in the reports
scratch scripts (look.py: ev/cmd/d/s rows per unit and site; truth_near.py: the truth file around a site; unstarted.py)
  and the record's dmg rows for the suspect frames; yards.rs lane geometry for the lane frames
```
