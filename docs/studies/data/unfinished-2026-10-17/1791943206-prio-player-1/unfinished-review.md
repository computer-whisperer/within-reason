# Unfinished structures: 1791943206-prio-player-1 (seat 0)

An arena duel on Comet Catcher Remake 1.8, played 2026-10-13. We started at B8 as Armada, with one seat: the player was Opus 5.5 medium, there was no commander, and the hands ran with the layers news,same,fuse,openers,split,tick,places,again,near. **We won at 18:02** against BARb hard_aggressive.

frames.py, seat 0 (the only seat), verbatim:

```
1791943206-prio-player-1 seat 0, record to 18:02: frames 137 (decayed 3, finished 111, killed 18, unfinished at end 5), metal in unfinished frames 1222, build orders never started 25
```

`run/floor.py` gives `aband` = 3, which equals frames.py's decayed count. The game has only seat 0, so no symlink directories were needed.

## Tally (26 frames, 1222 metal put in)

| cause | n | metal |  | fate | n | metal |  | type | n | metal |
|---|---|---|---|---|---|---|---|---|---|---|
| U4 died/fled/killed | 19 | 386 |  | killed | 18 | 383 |  | armnanotc | 4 | 451 |
| U8 game end | 5 | 659 |  | decayed | 3 | 180 |  | armvp | 1 | 307 |
| U9 harness fault | 1 | 177 |  | at end | 5 | 659 |  | armllt | 13 | 301 |
| U3 player dropped it | 1 | 0 |  | | | |  | armmex | 7 | 163 |
| U0, U1, U2, U5, U6, U7 | 0 | 0 |  | | | |  | armadvsol | 1 | 0 |

Two items carry a second cause in `also`: armnanotc_27342 (U2) and armllt_10658 (U5).

What the player knew:
- **Before the cause acted:** 18 frames. These are 12 of the 18 killed frames, the decayed nano, and the 5 frames standing at the end.
- **Only afterwards:** 7 frames. Six killed frames were reported only by the fights line ("lost armllt to armpw in our half"), which never says the frame was unfinished. One decayed turret was reported by the abandoned line.
- **Never told:** 0.
- **Dropped by the player itself:** 1 (the 0% armadvsol).
- **The threat:** for 11 of the 18 killed frames, the player had seen the raid in the report before the frame began.

## Items, ranked by cost (20 of 26; all 26 are in unfinished.json)

1. **armnanotc_27342, B8, 8:00 to 9:56, decayed at 77%, 177 metal. Cause U9 (also U2). Large, high confidence. Fix: hands-rule.**
   - **What happened:** constructor_16840's nano came from its list. At 60% the list queued an armadvsol behind it (8:36). At 8:37, constructor_2945's nano finished 222 elmos away. The finished event matches any started build of the same type within 250, so it took 2945's nano for 16840's and promoted the queued advsol. The blocked-site watchdog then read "standing at an unstarted site": the builder was still building the nano. At 8:45 the watchdog sent `stop` with the frame at 76%. Its line: "its armadvsol at B8 never started in the 8 s it stood at the site ... the order goes again elsewhere". At 8:46 the list placed the advsol 400 elmos away, and the nano decayed.
   - **Evidence:** the `cmd stop 16840` row at 8:45, "8:46 (list) build a armadvsol at home (B8)", and `Event::UnitFinished` with `near.dist2d(done.pos) < 250.0`. That line is unchanged at HEAD.
   - **What the player knew:** before, from the 8:37 report: "(62 % built)". From 8:43 the builder's entry named only the advsol. Afterwards, from the 9:58 fights line: "abandoned an unfinished armnanotc (1% built) x1". That line gave no place and no hands' line followed.
2. **armllt_1371, D7, 6:34 to 6:48, killed by armpw at 71%, 60 metal. Cause U4. Fix: player.**
   - **What happened:** a list turret at spot_64 (the 5:15 queue), started with nine Pawns at D6 in the 6:32 report. The builder's entry said: "enemies_near: party_16 (9 armpw) 756 away". The builder stayed on it, and the Pawns killed it while group_J came back.
   - **What the player knew:** the 6:42 report showed "(54 % built)", then "lost armllt to armpw in our half x1".
3. **armllt_10658, B4, 9:48 to 10:14, killed by armflea at 48%, 41 metal. Cause U4 (also U5). Fix: player.**
   - **What happened:** the metal store sat at 0 to 18 from 9:56, and progress stopped by 10:03. A lone Tick then shot the frame for 11 s while its builder stood on it.
   - **What the player knew:** the 10:08 report said "enemy in sight: 1 at B4 ... armflea 1; killing our Sentry (armllt) (85 metal) now", with the frame at 27%. The player gave no order.
4. **armllt_977, D8, 11:09 to 11:21, killed by armpw at 39%, 33 metal. Cause U4. Fix: player.**
   - **What happened:** a list turret (the 10:37 queue) at spot_71. Four Pawns were 630 to 750 elmos away at the start, and no armed unit of ours was within 800.
   - **What the player knew:** the 11:06 report said "enemy in sight: 2 at E7 ... armpw 2" and "extractors under threat: E7 (2 enemies, NO turret)". The 11:16 report showed "(24 % built)".
5. **armmex_15814, D5, 8:59 to 9:13, killed by armpw at 61%, 30 metal. Cause U4. Fix: player.**
   - **What happened:** the second extractor frame at spot_47 within 15 s. The 8:59 report showed a Pawn at D5, and the builder's entry said "enemies_near: party_37 (1 armflea) 264 away". The 9:09 report showed it at 46%.
6. **armmex_1611, E5, 17:04 to 17:18, killed by armpw at 59%, 30 metal. Cause U4. Fix: none.**
   - **What happened:** a list extractor (the 15:39 queue) in his half, with no guard. No enemy was within 900 at the start.
   - **What the player knew:** the frame was reported at 49% at 17:14. The Pawn that killed it was never in a report.
7. **armmex_24498, F5, 17:16 to 17:32, killed by armpw at 61%, 30 metal. Cause U4. Fix: player.**
   - **What happened:** a list extractor (the 15:39 queue) in his half, with no guard. At the start the Pawn was 576 elmos away. The 17:14 report showed it at E5, and the builder's entry said "enemies_near: party_95 (1 armpw) 484 away". The frame reached 54% by 17:31.
8. **armmex_28190, E4, 17:07 to 17:21, killed by armflash at 58%, 29 metal. Cause U4. Fix: player.**
   - **What happened:** the extractor was started while a Tick was hitting its builder. The builder's entry said: "enemies_near: party_112 (1 armflea) 107 away ... under_fire: yes". Our three units nearby moved on, and a Blitz killed the frame.
9. **armllt_21205, B5, 7:05 to 7:18, killed by armflea at 25%, 21 metal. Cause U4. Fix: none.**
   - **What happened:** no enemy was within 900 at the start and the site had no guard. An unseen Tick killed it. The 7:12 report showed it at 19%.
10. **armmex_6304, C8 near home, 3:13 to 3:28, killed by armflea at 41%, 20 metal. Cause U4. Fix: none.**
    - **What happened:** an extractor from the opening list. An unseen Tick killed it, and nothing armed of ours was within 800. The report then said "lost armmex to armflea at home x1".
11. **armllt_15605, E7, 7:54 to 8:01, killed by armpw at 20%, 17 metal. Cause U4. Fix: none.**
    - **What happened:** two Pawns were 520 to 540 elmos away at the start. They were in the record but in no report. They killed the frame, and its builder constructor_56 a second later (8:02).
12. **armmex_7898, D7, 8:18 to 8:26, killed by armpw at 32%, 16 metal. Cause U4. Fix: player.**
    - **What happened:** started at spot_64, where the 6:48 turret had died. The 8:18 report said "extractors under threat: D6 (2 enemies, NO turret)". Five Blitzes stood within 800, and the frame was lost anyway.
    - **What the player knew:** no report entry showed the frame.
13. **armllt_13253, D6, 8:19 to 8:31, killed by armflea at 18%, 15 metal. Cause U4. Fix: player.**
    - **What happened:** a turret at spot_56, a spot with no extractor (the 7:40 "taken as done" item below), started inside the D6 raid. The builder's entry said "enemies_near: party_29 (1 armpw) 467 away".
14. **armllt_12979, E5, 17:35 to 17:43, killed by armpw at 15%, 13 metal. Cause U4. Fix: player.**
    - **What happened:** started 17 s after spot_46's extractor died. A Pawn was 292 elmos away at the start, and the 17:31 report showed it there.
15. **armllt_11161, C4, 12:03 to 12:08, killed by armpw at 14%, 12 metal. Cause U4. Fix: none.**
    - **What happened:** no enemy was within 900 at the start. Four Pawns and a Tick arrived within 5 s. Two Blitzes were near.
16. **armmex_13685, D5, 8:44 to 8:49, killed by armpw at 16%, 8 metal. Cause U4. Fix: player.**
    - **What happened:** the builder was already hurt when it started. The 8:43 report showed "enemy in sight: 1 at D5 ... armpw 1", and the builder's entry said "health hurt (63%); enemies_near: party_29 ... 496 away".
17. **armllt_19130, D6, 8:22 to 8:24, killed by armpw at 4%, 3 metal. Cause U4. Fix: player.**
    - **What happened:** the list started the turret with three Pawns 65 to 155 elmos from the site. The 8:18 report showed the D6 raid.
18. **armllt_30020, D7, 8:31 to 8:33, killed by armpw at 3%, 3 metal. Cause U4. Fix: player.**
    - **What happened:** the third frame at spot_64 in two minutes. Pawns were 170 to 350 elmos away. The 8:27 report said "enemy in sight: 3 at D7 ... armpw 3; killing our Metal Extractor".
19. **armllt_13591, C4, 11:10 to 11:21, decayed at 3%, 3 metal. Cause U4 (fled). Fix: none.**
    - **What happened:** a Pawn stood 46 elmos from the site as the frame began. In the hands' log for 11:10, the actor was "alarmed by party_54". Its menu read go_spot_63=0.54, and the pick line was "PICK ... constructor_30353: took w2 of 1 at 0.6", which walked it to spot_63. The builder was hit, falling from 100% to 60% health.
    - **What the player knew:** only afterwards, from the line "11:21 abandoned an unfinished armllt at C4 (1% built)".
20. **armllt_1059, E4, 17:50 to 17:52, killed by armflash at 2%, 2 metal. Cause U4. Fix: hands-rule.**
    - **What happened:** the list started the turret with a Tick 85 and a Blitz 271 elmos from the site. Its builder was at 33% health. The 17:45 report showed nothing at E4.

**Also listed, cost none:**
- **U8 (game end), five frames, all started 17:29 to 17:51 and still progressing at 18:02:**
  - armvp_12149 (#vp4), 52%, 307 metal.
  - armnanotc_2935 (#nano_a), 54%, 124.
  - armnanotc_885 (#nano_b), 36%, 83.
  - armllt_21106 (F3), 92%, 78.
  - armnanotc_1321 (#nano_vp3), 29%, 67.
- **U3, armadvsol_31890 (B7, 10:09 to 10:20, 0%, 0 metal):** the player cancelled the list at 9:58 ("cancelled the home constructors' solar lists so they expand instead"). The engine began the queued advsol in the same tick that the hands dropped it: "10:09 constructor_18284: dropped the step 'armadvsol' its old list had queued behind the build it was on". Fix: tool.

## Build orders never started

frames.py lists 25. Six of them are not items: the frame started at the host's placed site, 338 to 600 elmos from the ordered point, or after a 110 s walk (constructor_18284 to F3). Of the 19 real items, by cause: U6 7, U3 4, U8 4, U2 2, U1 1, U4 1. None put in any metal.

- **U6, rejected at spots other builders had just taken (7: 6:40, 9:37, 10:25, 15:16, 15:32, 15:47 twice).** Each said "the engine refused a armmex at ...: the site was bad". Two lists, or a list and a pick, named the same spot without an id. At 7:18 the refusal also made constructor_2945 skip its own turret at spot_71.
- **U2, taken as done:**
  - 4:59: constructor_1412's spot_63 turret. The shared id #t63 was built by constructor_2945, so nothing was lost.
  - **7:36: constructor_24993, "extractor spot_56".** The re-sent list's first step was "taken as done" at 7:40. Then "7:41 (list) build a armllt at spot_56" replaced the walk. spot_56 got a turret frame (item 13) and no extractor. This fault has since been fixed (e9f5175).
- **U3, the player's new list or cancel replaced the order:**
  - 9:44: constructor_16840's extractor, dropped by the 9:58 cancel.
  - 10:46: constructor_24993's nano, a pick order replaced by the 11:36 #vp3 list after a 58 s walk.
  - 14:56: constructor_27107's turret, replaced by the 14:55 list.
  - 16:45: constructor_27504's #nano_vp3, after its #vp3 step "could not be played this second and was dropped". That drop is the same-tick id race, U9 in `also`.
- **U1:** at 16:25 a turret the pick had ordered was replaced 2 s later by "help constructor_10629 build".
- **U4:** at 8:02, constructor_56 was killed the second its extractor order went out.
- **U8:** orders at 17:31, 17:33, 17:50 and 17:53, still walking when the game ended.

## The three mechanisms

1. **A builder's own frame is not checked by its unit id, and the watchdog does not check what the builder is actually doing.** The finished event is matched by type within 250 elmos, so another builder's nano finishing nearby counts as this builder's. The blocked-site watchdog then treats a builder still building as standing at an unstarted site. This lost armnanotc_27342: 177 metal, a nano 10 s from done. Both lines are still in HEAD (`pianist/mod.rs`).
2. **Lists start frames inside a raid, and the builder stays until the frame dies.**
   - **Scope:** 11 of the 18 killed frames, 197 metal: armllt_1371, armllt_15605, armmex_7898, armllt_13253, armllt_19130, armllt_30020, armmex_13685, armmex_15814, armmex_24498, armllt_12979, armllt_1059.
   - **The evidence:** each began with an armed enemy within 600 of the site. For 9 of them that raid was in the report the player had just read.
   - **Why:** lists play under threat by rule (H-HANDS-SCRIPT), and Jev kept every builder on its frame.
   - **Fixes:** a list step could wait while an armed party is within about 600 of its site. A version at 800 was rejected on 2026-10-01, so this needs the user's word. Or the player could send the extractor-and-turret lists behind an escort or after the raid.
3. **Raiders reach unguarded frames that nothing showed.**
   - **Scope:** the other 7 killed frames, 186 metal: armmex_6304, armllt_21205, armllt_10658, armllt_977, armllt_11161, armmex_1611, armmex_28190.
   - **What happened:** a lone Tick or a few Pawns shot each frame for 2 to 11 s while its builder stood on it. 16 of the 18 builders survived their frames.
   - **Fixes:** the builder could reclaim the frame when a raider it cannot fight opens fire. I believe the engine refunds a nanoframe's metal on reclaim, but that is unmeasured here. Or an escort, or a turret first.

The 659 metal in the U8 frames was not lost: the game was won.

**Report finding:** a frame killed while unfinished reaches the player only as "lost armllt to armpw ...", the same words as a finished building. The 9:56 nano decay came through as "(1% built)", with no place.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...batch.json"
run/frames.py MATCH --seat 0 ; run/frames.py MATCH --seat 0 --json --all
run/floor.py MATCH
run/impedance_sweep.py MATCH --seat 0 --from 8:20 --to 9:00 --all ; run/impedance_sweep.py MATCH --seat 0 --all
run/commander_turns.py MATCH --calls
run/hands_window.py MATCH 8:35 8:47 constructor_16840 ; run/hands_window.py MATCH 11:07 11:12 constructor_30353
scratch scripts over record-0.jsonl and strategist-0.jsonl (per-unit rows, the context at each frame's start and fate, the report entries, the never-started orders)
git show 59b8cf4: crates/bot/src/brain/pianist/{mod,execute,lists}.rs, crates/bot/src/brain/briefing.rs (the finished-event match, the watchdog, the abandoned line)
```
