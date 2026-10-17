# Unfinished structures: 1791573192-nuke-1-medium/00

An arena duel on Comet Catcher Remake 1.8: one seat (Opus 5.5 medium player, Jev hands, layers news,same,fuse,openers,split,tick,places,again,near) against BARb medium (Cortex). The objective was the nuke. The game ended undecided at the 40:00 cap (Timeout). The first silo stood at 17:31, and one missile wiped his G2 base at about 21:30.

frames.py, seat 0 (verbatim):

```
1791573192-nuke-1-medium seat 0, record to 40:01: frames 264 (decayed 14, finished 228, killed 19, unfinished at end 3), metal in unfinished frames 11521, build orders never started 37
```

`run/floor.py` reports `aband` 14, which equals frames.py's decayed count. Its `aband_m` is 252, against the 1641 metal frames.py puts in the decayed frames, so the two measure different things.

**Six of the 14 "decayed" frames were shot.** armllt_26519, armmoho_7673, armmex_684, armadvsol_17686, armmex_27116 and armllt_13435 (681 metal) were destroyed with no attacker recorded. But damage rows on each frame in its last seconds (or the second after), and for 7673 and 17686 enemies in sight beside them, show they were killed. They keep fate `decayed` as frames.py has it, and their cause is U4.

## Tally

| cause | n | metal |   | fate | n | metal |   | type | n | metal |
|---|---|---|---|---|---|---|---|---|---|---|
| U1 pick | 0 | 0 | | decayed | 14 | 1641 | | armfus | 2 | 3952 |
| U2 own list | 1 | 79 | | (of which shot) | (6) | (681) | | armafus | 1 | 3589 |
| U3 player | 3 | 410 | | killed | 19 | 2339 | | armavp | 1 | 1664 |
| U4 died/killed | 25 | 3020 | | at end | 3 | 7541 | | armadvsol | 4 | 776 |
| U5 economy | 0 (1 as also) | 0 | | | | | | armmoho | 2 | 613 |
| U6 site | 0 | 0 | | | | | | armllt | 7 | 274 |
| U7 helpers | 0 | 0 | | | | | | armnanotc | 2 | 202 |
| U8 game end | 3 | 7541 | | | | | | armsolar | 3 | 175 |
| U9 harness | 4 | 471 | | | | | | armmex | 12 | 144 |
| U0 unknown | 0 | 0 | | | | | | armpb | 1 | 122 |
| | | | | | | | | armbeamer | 1 | 10 |

**U8 deviation.** armafus_19543 (3589) and armfus_15500 (3316, 99% at the last line) started 2:58 and 2:20 before the end, which is outside the two-minute window. They are U8 because no other class fits: the builders stayed on them, the metal store was at 91% or more the whole time, and both progressed to the last second. Reclassify them if the sum needs the strict window.

**What the player knew** (the 36 frames):
- 24 were in the report before the cause acted.
- 3 were told only by the abandoned line (armllt_22170, both nanos).
- 9 were never said, all forward extractors and turrets killed within seconds.
- The player dropped 3 itself (U3: armsolar_6821, armadvsol_5624, armllt_26900), each with the frame in its report.

## Items by metal (20 of 36; all are in unfinished.json)

1. **37:03, seat 0, armafus_19543 B7, U8 (3589 of 9700, 37%).**
   - Jev's pick started it ("build a armafus at spot_67", 37:00), with constructor_988 helping. It progressed every second to the 40:00 cap with the store full.
   - Player knew: in every report from 37:13. Fix none; cost none.
2. **37:41, seat 0, armfus_15500 B7, U8 (3316 of 3350, 99%).**
   - Jev's pick ("build a armfus at home"). It was rising about 1.5% a second at the last line. Fix none; cost none.
3. **7:48, seat 0, armavp_2014 B7 (avp_yard, home), U4 (1664 of 2600, 64%).**
   - Killed by party_10 (4-7 corak) in the 7:23 raid.
   - The menus rated attack_party_10 at 0.26-0.30 while the picks hunted lone corak 1,000-1,500 away ("7:43 PLAYS group_A: 1 of group_A hunt party_8 (1 corak)"). The commander was on an assist list until 7:53.
   - Player knew: the 7:36 entry said "(62 % built)", with 8 corak in sight 1254 from home.
   - Fix: hands-rule. Cost: large; tier 2 came about 2 min later, as plant_21205 stood at 10:41.
4. **39:29, seat 0, armfus_3736 A7, U8 (636, 19%).** Started 32 s before the end. Cost none.
5. **19:46, seat 0, armmoho_7673 B5, U4 (322, 52%; recorded as decayed).**
   - Two corthud and an armflash in sight shot it to death (104/209/623/409 damage at 19:43-19:46).
   - Player knew: the 19:39 entry said "(46 % built)" with no enemy near. The 19:47 report said "abandoned an unfinished armmoho (7% built)", which is wrong.
   - Fix player; large; medium confidence.
6. **17:50, seat 0, armmoho_1240 A6 (spot_54, home), U4, also U5 (291, 47%).**
   - The metal store was empty from 17:12 (silo and fusions drawing it), so the frame was half-built after 49 s.
   - A lone armflash (party_96) shot it from 17:36, joined by 2 corak from 17:45. Ten menus offered attack_party_96 (0.15-0.47), but the picks gathered and followed. The player's 17:39 order landed too late.
   - Player knew: the 17:39 entry said "(38 % built)", with the armflash "killing our Advanced Metal Extractor".
   - Fix hands-rule; large.
7. **10:09, seat 0, armadvsol_5624 A8 (solar_field), U3 (259, 74%).**
   - The 8:23 queue ["stop", "armavp avp_yard2 #avp2", "assist"] returned "its build in progress dropped"; the stop landed at 8:31 with the frame at 72%.
   - Over its 98 s decay, two other "armadvsol solar_field" steps placed new frames 100-350 away.
   - Player knew: the 8:23 entry said "(64 % built)", and the player dropped it itself. Fix player; large.
8. **22:01, seat 0, armadvsol_17686 C7, U4 (248, 71%; recorded as decayed).**
   - "#adv2" was given no place, and the host put it 362 from spot_69 on the raid path. Corstorm and corthud in sight shot it (157 at 21:54, then 106/209/439).
   - Player knew: the 21:58 entry said "(62 % built)", with party_135 499 away. Fix player; large.
9. **10:07, seat 0, armadvsol_14742 A8, U9, also U3 (241, 69%).**
   - The 8:23 re-list had no `stop`; the player meant "each finish an advanced solar".
   - At 8:36 the hands dropped the old list's queued step with a `stop`: "dropped the step 'armadvsol solar_field' its old list had queued behind the build it was on". The stop also ended the 69% build. At 8:37 the new step placed a new frame (8265) about 100 away instead of taking the build as under way.
   - Player knew: the 8:23 entry said "(53 % built)". From 8:42 the entry showed the new frame under the same words, so the drop was invisible. Fix hands-rule; large.
10. **32:42, seat 0, armnanotc_14058 B8, U9, also U2 (200, 87%).**
    - 31:09: the 60% rule queued the next step (an advanced solar at spot_73).
    - 31:23: with the nano at 85% and still being built, the watchdog said "its armadvsol at B8 never started in the 8 s it stood at the site ... refused for 90 s" and sent `stop`.
    - Nobody returned to the nano. #nano_b was placed as a new frame 64 away at 31:37.
    - Player knew: never; the 31:16 entry showed the queued solar as the current order. After: only "abandoned an unfinished armnanotc (0% built) x1" at 32:51. Fix hands-rule; large.
11. **21:35, seat 0, armpb_10174 C6 (spot_63), U4 (122, 18%).**
    - Step #pb4 of the player's list, killed by a corstorm. The 21:29 report had the corstorm in sight at C6. Fix player; large.
12. **3:08, seat 0, armsolar_6821 B7, U3 (119, 77%).**
    - The 2:20 lane-clearing queue began with `stop`. The player said "it drops the solar it had just started (5% built, so little was lost)", but the order landed after the 7 s penalty and stopped the frame at 77%.
    - Player knew: the 2:20 entry said "(5 % built)". Fix tool (a stop drops whatever share the frame has reached by landing). Large.
13. **20:22, seat 0, armllt_22170 A8, U2, also U9 (79, 93%).**
    - "#sf1", shared with constructor_14492, was placed at two sites in the same tick, so it became two frames. #sf2 was placed on 14492's #sf1 site, and the engine refused it at 19:38.
    - The commander's queued "help build #sf2" was then advanced: 19:39 stop and 19:40 "help build #sf3", leaving its own frame at 93%.
    - Player knew: after only, "20:22 abandoned an unfinished armllt at A8 (0% built)". Fix hands-rule; small.
14. **17:16, seat 0, armllt_26519 C4, U4 (72, 85%; recorded as decayed).**
    - An unseen attacker hit it for 157/104/209/263 in four seconds; a corthud showed itself at 17:19.
    - The 17:22 report said "abandoned ... (19% built)", which is wrong. Fix player; small; medium confidence.
15. **20:32, seat 0, armsolar_10632 A7 (avp_yard2), U4 (48, 31%).**
    - The 20:14 list asked for 3 solars with no place, and the host put them in the path of party_119 (11 corstorm/corthud, 641 away in the 20:24 entry). The list re-placed the same solar at the same site (armsolar_493, killed at 5%) and tried a third time. Fix player; small.
16. **12:29, seat 0, armllt_16966 C5, U4 (43, 51%).**
    - One corak, 108 away in the 12:26 entry. group_K1's hunt ended on the 900 leash. Fix hands-rule; small; medium.
17. **10:42, seat 0, armllt_26900 C8, U3 (32, 38%).**
    - The 10:11 lane-clearing queue (["stop", ...]) dropped it. The 10:11 entry said "(30 % built) ... standing in plant_27255's exit lane". The player dropped it itself; small.
18. **24:44, seat 0, armmex_28165 C6, U4 (28, 55%).** A forward extractor; the corthud was 339 away in the 24:37 entry. Small.
19. **31:52, seat 0, armadvsol_6548 B8, U9 engine (28, 8%).**
    - The builder drove a 350-elmo loop through plant_27255's yard with no command or decision row. The frame decayed in 18 s, and on its return the builder began a new frame at the same site (32:03).
    - Low confidence: nothing records what moved the builder. Large by type.
20. **8:33, seat 0, armllt_31972 C7, U4 (26, 31%).** The builder was killed by 5 corak, 42 away in the 8:33 entry. Small.

Not listed above:
- 14 more extractors and turrets killed by raiders, most within seconds of starting (C4-D6, A4-B5, A6, C7; 148 metal together), all from the player's lists.
- armsolar_493 (item 15's second try).
- armnanotc_17753 (U9, 2 metal). For 17753, the old list's queued nano stayed in the engine until the build in progress finished, the engine started it at 30:06, and the hands dropped it the same second ("dropped the step 'armnanotc' its old list had queued").

## Build orders never started (27 real of 37; 15 in the JSON)

By cause:
- U6, site refused or blocked: 10.
- U3, replaced by the player before it began: 8.
- U4, the builder fled or died: 8.
- U1, the pick replaced the step: 1.

Ten of the 37 are not items:
- Six started at the engine's site more than 300 from the asked point (6:40, 8:32, 8:33, 28:45, and both #afus1 orders at 30:47).
- Three were a shared id's late arrivals that built the first builder's frame at the same site (19:17 #nano1, and both #silo2 orders at 32:46: silo 9632 rose at three armacv's rate).
- One was re-sent and then started (27:57 constructor_23348).

The largest:
- 32:32, constructor_21938, armfus, U6: a blocked site in the crowded B8 yard. After a 68 s walk, the watchdog said "its armfus at B8 never started in the 8 s it stood at the site". The fusion was never built.
- 8:20, constructor_6635, armavp #avp1, U3: by design. The 8:23 queue moved the plant to avp_yard2 after the raid.
- Moho upgrades, six orders. Three were dropped by the player's re-lists or cancels (19:54, 20:01, 24:18, U3). Three were refused or blocked (27:01, 27:44, 30:01, U6); at 27:44 the extractor to upgrade was gone ("no extractor of ours stands at spot_43").
- 18:57, constructor_1726, armllt #sf2, U6: placed on the site the host gave #sf1. The engine refused it at 19:38, so a turret the player listed never stood. This is the same id fault as item 13.
- 20:05, 20:37, 23:19, 26:32, 27:24, 29:35, 29:38, 30:22, U4: retreat picks and dead builders at forward spots.

## Three mechanisms

1. **Defend a home frame that is under fire.** Answer a party shooting one of our frames at home with the nearest group that outweighs it, not with detachments at lone raiders elsewhere. This covers armavp_2014 and armmoho_1240: 1955 metal, plus about 2 min of tier 2.
2. **A same-type order near a standing unfinished frame repairs that frame instead of placing a new one** (within about 200 of the order's site). This covers armadvsol_14742, armadvsol_5624 and armnanotc_14058: 700 metal.
3. **Never end the build in progress when the hands drop or advance a step queued behind it.** Remove only the queued command (re-list drop, help-step promotion), and let the watchdog time a step only after the builder has left its previous build. This covers armadvsol_14742, armnanotc_14058, armllt_22170 and armnanotc_17753: 522 metal, overlapping mechanism 2 on 14742 and 14058.

Also seen, outside the tally:
- The report's "fights since last turn: abandoned an unfinished X (N% built)" gives the share at death, not the share reached: 74% and 69% advanced solars appeared as "0% built x2". It also calls the six frames that were shot "abandoned".
- An id step given to two builders in the same tick is placed twice. #sf1 made two turrets, and #afus1 made two advanced fusions (24911 and 29194), both finished.

## Commands run (from the repository root, M = run/matches/1791573192-nuke-1-medium/00, S = the scratch directory)

```
cat $M/../results.jsonl; python3 -c "import json;print(json.load(open('$M/../batch.json')))"
run/frames.py $M --seat 0; run/frames.py $M --seat 0 --json > $S/frames.json; run/floor.py $M
run/impedance_sweep.py $M --seat 0 --from 1:30 --to 3:30 --all (and 5:20-8:40, 7:46-10:50, 18:50-20:40, 29:20-32:50, 32:30-36:50, whole game --all to $S/sweep_all.txt)
run/hands_window.py $M 7:40 7:48 group_L group_E group_A; 17:36 17:50; 31:20 31:56 constructor_21763
run/commander_turns.py $M --calls > $S/calls.txt
scratch scripts (python3 -I, read-only): $S/bin/turn.py (a turn's report lines), said.py (the player's words and results of a turn),
  trace.py (units, stores, events, cmds and d rows per second), dmg.py (damage rows and enemies near), scan.py (every unfinished
  frame's last damage, health and builder distance), knew.py (the report entries while each frame stood), ns.py (never-started
  orders against later frames and site events), items.py, ns_items.py, build.py (write unfinished.json)
grep of bot.log and record-0.jsonl for single unit ids; truth-0.jsonl for the killers' side
```
