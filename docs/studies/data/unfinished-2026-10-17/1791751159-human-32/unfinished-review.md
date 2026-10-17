# Unfinished structures: 1791751159-human-32

Human game, 2026-10-11, Flats and Forests v2.2, realtime. Three Opus 5.5 medium seats under an Opus 5.5 commander: t2 = seat 0 (middle, Cortex), t3 = seat 1 (south, Armada), t4 = seat 2 (north, Cortex). The opponents were thebluegecko (Cortex) and u6bkep (Armada). LOST: the side conceded at 27:20 ('the Shiva switch at 21 min broke us') and the record ends at 27:34. t4 was eliminated at 25:42 and t2 at 26:52.

## Denominators (frames.py, verbatim)

- `1791751159-human-32 seat 0, record to 27:34: frames 140 (decayed 7, finished 126, killed 7), metal in unfinished frames 636, build orders never started 55`
- `1791751159-human-32 seat 1, record to 27:34: frames 172 (decayed 14, finished 150, killed 8), metal in unfinished frames 2800, build orders never started 59`
- `1791751159-human-32 seat 2, record to 27:34: frames 139 (decayed 3, finished 120, killed 16), metal in unfinished frames 1078, build orders never started 33`

`run/floor.py` aband is 7, 14 and 3 for the three seats, which equals the decayed counts. Seats 1 and 2 were read through the symlink directories.

55 unfinished frames held 4,514 metal: 14 frames and 636 metal in t2, 22 and 2,800 in t3, 19 and 1,078 in t4. No frame was standing unfinished at the end, because every one was destroyed first.

## Tally

| cause | n | metal |
|---|---|---|
| U0 unknown | 0 | 0 |
| U1 pick sent it elsewhere | 1 | 0 |
| U2 own list moved on | 0 | 0 |
| U3 player re-listed / moved | 16 | 1655 |
| U4 builder died / fled, frame killed | 35 | 1332 |
| U5 economy | 0 | 0 |
| U6 site | 0 | 0 |
| U7 helpers withdrawn | 0 | 0 |
| U8 game end | 0 | 0 |
| U9 harness fault | 3 | 1527 |

Second causes (`also`): U1 1, U3 1, U4 6, U5 5, U6 1, U9 3. U5 is never the first cause. It is the second cause on 5 frames that crawled on a metal store at 0 before they were dropped or killed. U9 is the second cause on 3: corsolar_27254, armsolar_21183 and corvp_4612.

| fate | n | metal |
|---|---|---|
| decayed | 24 | 2252 |
| killed | 31 | 2262 |
| at_end | 0 | 0 |

Of the 24 decayed frames, 6 were killed outright by out-of-sight fire, and 2 more took a final shot after decaying.

| type | n | metal |
|---|---|---|
| armavp | 1 | 1040 |
| armhlt | 3 | 902 |
| corllt | 16 | 687 |
| corvp | 2 | 587 |
| corhllt | 4 | 311 |
| armmoho | 1 | 285 |
| armadvsol | 1 | 154 |
| armllt | 6 | 136 |
| armap | 1 | 111 |
| armsolar | 3 | 75 |
| armbeamer | 2 | 61 |
| cormex | 5 | 53 |
| armmex | 4 | 36 |
| corwin | 3 | 35 |
| cornanotc | 1 | 30 |
| corsolar | 2 | 11 |

**What the players knew.** 38 of the 55 frames were in a report before the cause acted. 16 were reported only afterwards, by the abandoned line or the 'lost' line. 1 was never reported. The players dropped 16 frames themselves, all U3 (1,655 metal). 15 of those drops were deliberate and seen beforehand. The exception is t4's commander plant, where the player's stop dropped a frame it did not know had started.

By seat:
- **t2** lost 636 metal across 14 frames. 376 of it is the duplicate refuge plant.
- **t3** lost 2,800 metal across 22 frames:
  - 1,151 in two duplicate plants;
  - 1,187 in three Overwatch towers and a moho during the Bull overrun;
  - the rest in small early drops and raids.
- **t4** lost 1,078 metal across 19 frames, all killed:
  - corgator and Rascal raids on the northern spots;
  - Shiva fire at H2-H4;
  - the commander's duplicate plant.

## Items, costliest first

**1. 21:53-26:30, t3 (seat 1, south), armavp_6196 at H7: U9 (also U3), decayed at 40%, 1040 of 2600 metal, large**
- What happened: six builders had 'armavp avp_yard #avp1'; two frames rose from the one id (plant_6196 at 21:53, plant_13221 at 22:01). The player's removes of plant_6196 (22:20, 22:35, 22:50) were all accepted, but the 'stop','assist' list sent with the first turned constructor_2000 into a helper of the frame it was to take apart, and 16022 and 31560 kept helping it from their old #avp1 steps: 9% at 22:10 to 40% at 24:04, about 800 metal after the first remove. Its last helper (31560) was re-listed at 23:58 for standing in plant_13221's lane; the frame decayed from 40% to 6% and Bulls finished it at 26:30.
- Evidence: 21:53 created u6196 by 2000; 22:01 created u13221 by 16022; 22:20 remove {by: constructor_2000, reclaim: [plant_6196]} accepted; 22:32 'takes plant_6196 apart next'; 22:33 PLAYS constructor_2000: help plant_6196 build [list]; 23:26 constructor_31560: help plant_6196 build (from its list).
- What the player knew: before: 22:20 note 'two advanced-plant frames had been started at avp_yard (plant_13221 at 19%, plant_6196 at 9%); reclaiming the second'; 23:28-23:58 reports 'constructor_31560: helping lab_6196 build'; 26:42 'abandoned an unfinished armavp (0% built) x1'.
- Fix: hands-rule. Confidence: high.

**2. 24:41-26:37, t2 (seat 0, middle), corvp_2012 at H1: U9 (also U4), decayed at 66%, 376 of 570 metal, large**
- What happened: 'corvp refuge #vp2' went to six builders the same tick at 23:26 and the host placed two sites: (11728,1104) for 29566, (11840,1232) for 30073. 30073 raised plant_16321 (24:31-24:52) with the others helping; 29566 began a second plant frame at 24:41 and stayed on it alone for two minutes on a metal store at 0 (its 25:21 queue left 'the build in progress to finish'), standing in plant_16321's exit lane; the Shivas shelled it and its builder from out of sight at 26:35 and both were destroyed at 26:37 at 58-66%.
- Evidence: 23:26 placed u29566 corvp (11728,1104); placed u30073 corvp (11840,1232) (same frame 42203); 24:31 created 16321 by 30073; 24:41 created 2012 by 29566; record M0 from 24:44 to 26:37.
- What the player knew: after: 24:47 report gave constructor_29566 'building a Vehicle Plant (corvp) at refuge (H1) (87 % built) ... it will be plant_16321' (the other frame's figure), then 'finishing an order' to 26:23; 26:39 'abandoned an unfinished corvp (36% built) x1' for a frame killed by fire.
- Fix: hands-rule. Confidence: high.

**3. 24:47-27:23, t3 (seat 1, south), armhlt_28022 at H6: U3 (also U4), killed at 82%, 361 of 440 metal, large**
- What happened: Overwatch #h3 at home crawled 2.5 min on a metal store at 0 (11 constructors on three towers, plants and Starlights); at 26:59, 12 Bulls 600-700 away in our home cell, the player cancelled eight constructors' lists with 'stop' so they could run; landed 27:04 at ~80%; the Bulls killed it 27:23.
- Evidence: stores min metal 0.0 from 24:47 to 27:04; 26:59 queue constructor_19990: stop -> 'list cancelled and its build in progress dropped'; 27:04 stop; 27:05 (plan) help factory_30909 build.
- What the player knew: before: 26:42 report 'building a Overwatch (armhlt) at cmd_post (H6) (77 % built) ... enemies_near: party_245 (12 armbull, 1 unidentified) 672 away'; 26:59 text 'the other seven were on the half-built Overwatch tower'; it dropped the frame itself.
- Fix: none. Confidence: high.

**4. 25:04-27:20, t3 (seat 1, south), armhlt_20798 at H6: U3 (also U4), killed at 75%, 330 of 440 metal, large**
- What happened: Overwatch #h5 at home, the twin of 28022 (repaired by 8254, 27071 and 19809 from 25:39 and 26:12), dropped by the same 26:59 'stop' at ~77% and killed by the Bulls at 27:20.
- Evidence: 26:59 queue constructor_16022: stop -> 'list cancelled and its build in progress dropped'; 27:04 stop; 27:05 (plan) help factory_30909 build; 27:20 destroyed u20798 by armbull.
- What the player knew: before: 26:42 report 'constructor_16022: ... building a Overwatch (armhlt) at cmd_post (H6) (77 % built)'; it dropped the frame itself.
- Fix: none. Confidence: high.

**5. 26:18-27:20, t3 (seat 1, south), armmoho_12300 at H6: U3 (also U4), killed at 46%, 285 of 620 metal, large**
- What happened: the advanced constructor's moho at spot_144, dropped by the 26:59 'stop' ('it was 40% into an upgraded extractor') so it could run from 12 Bulls; killed 27:20.
- Evidence: 26:59 queue constructor_18146: stop -> 'list cancelled and its build in progress dropped'; 27:04 stop; 27:05 (plan) help factory_30909 build; 27:20 destroyed u12300 by armbull.
- What the player knew: before: 26:42 report 'building a Advanced Metal Extractor (armmoho) at spot_144 (H6) (40 % built) ... party_245 (12 armbull) 643 away'; it dropped the frame itself.
- Fix: none. Confidence: high.

**6. 24:59-26:26, t3 (seat 1, south), armhlt_7003 at G6: U4 (also U5), killed at 48%, 211 of 440 metal, large**
- What happened: Overwatch at def_west: seven builders on 'armhlt def_west #h1' split over two frames the same second (8449 finished 25:57; #h2 was skipped as 'stands already' elsewhere, so this one stood in for it); its builder alone on a metal store at 0; at 26:22 the pick walked it to spot_164 as party_204 (10-15 Bulls) came within 435; the Bulls killed the frame at 48%.
- Evidence: 24:59 created u7003 by 13601 and u8449 by 2000 (both from #h1); 26:22 PLAYS constructor_13601: walk to spot_164 [plan] (took w2 of 1 at 0.64); 26:22 record: armbull at 435-560.
- What the player knew: before: the 25:xx reports listed 'constructor_13601: ... building a Overwatch (armhlt) at def_west (G6) (28 % built)'.
- Fix: none. Confidence: high.

**7. 24:59-25:42, t4 (seat 2, north), corvp_4612 at H1: U3 (also U9), killed at 37%, 211 of 570 metal, large**
- What happened: 'corvp north_yard #vp3' went to the commander and constructor_27436 the same tick at 23:48 and the host placed two sites; 27436 raised plant_19840 from 24:35; the commander began a second plant frame on arrival at 24:59. The player's 24:48 'stop' for commander_t4 (meant to abandon 'the new plant (34% built)', which was 27436's) landed at 25:04 and dropped the commander's frame; the pick then sent the commander at party_202 (a radar Shiva); a corshiva killed it at 25:42 and its death blast destroyed both plant frames.
- Evidence: 23:48 placed u23146 corvp (11504,1504); placed u27436 corvp (11632,1632); 24:35 created u19840 by 27436; 24:59 created u4612 by the commander; 24:48 queue commander_t4 ['stop'] -> 'list cancelled and its build in progress dropped'; 25:04 stopped on your `stop`; 25:05 (plan) attack party_202.
- What the player knew: after: 24:48 report 'commander_t4: ... walking to build a Vehicle Plant (corvp) at north_yard (H1), 558 to go'; 25:44 'lost corvp to unseen in our half x2'; its stop dropped a frame it did not know of.
- Fix: hands-rule. Confidence: high.

**8. 20:01-21:40, t3 (seat 1, south), armadvsol_17435 at H6: U3 (also U6), decayed at 44%, 154 of 350 metal, large**
- What happened: the list step 'armadvsol' had no place and the host's site put the builder in plant_10491's exit lane; at 20:33 the player dropped it ('I dropped that build, losing the metal in the frame') and re-listed it at spot_162; landed 20:39 at ~44%.
- Evidence: 20:33 woken: 'plant_10491's exit lane is blocked: 1 of ours (armcv_8451) have stood in it for 0:30'; 20:33 queue constructor_8451 ['stop','armadvsol spot_162'] -> 'its build in progress dropped'; 20:39 stopped on your `stop`; 20:40 (list) build a armadvsol at spot_162 (H7).
- What the player knew: before: 20:33 report 'building a Advanced Solar Collector (armadvsol) at 571 from spot_142 (H6) (36 % built); standing in plant_10491's exit lane since 31 s ago'; it dropped the frame itself.
- Fix: hands-rule. Confidence: high.

**9. 17:38-18:49, t4 (seat 2, north), corhllt_28232 at H1: U4 (also U5), killed at 69%, 135 of 195 metal, large**
- What happened: Twin Guard at spot_15 crawling on an empty metal store (81% at 18:06, progress to 18:42); party_136 (19 corgator) arrived at 18:42 and killed it at 69%.
- Evidence: stores min metal 0.0; first metal after 5 s; 18:42 record: first corgator within 700; 18:49 destroyed u28232 by corgator.
- What the player knew: before: 18:06 report 'building a Twin Guard (corhllt) at spot_15 (H1) (81 % built)', then 'finishing an order'.
- Fix: none. Confidence: high.

**10. 25:34-26:17, t3 (seat 1, south), armap_30335 at H7: U9 (also U1), decayed at 17%, 111 of 650 metal, large**
- What happened: three builders on 'armap air_yard #ap1' the same tick were placed at two sites (6264 at (11952,9496), the others at (11744,9496)); factory_30909 rose at 25:21 and 6264 began a second air plant at 25:34. At 25:47 ('The shared build label failed again') the player stopped 6264 and ordered the duplicate taken apart; the stop cancelled the list that carried the reclaim, and the pick sent 6264 to help constructor_25791 at 25:56; the frame decayed.
- Evidence: 25:02 placed u6264 armap (11952,9496); placed u20455 and u25791 (11744,9496); 25:47 queue constructor_6264 ['stop'] + remove {by: constructor_6264, reclaim: [factory_30335]}; 25:54 'takes factory_30335 apart next, at the front of its list' then 'stopped what it was doing on your `stop`'.
- What the player knew: before: 25:29 report 'building a Aircraft Plant (armap) at air_yard (H7) (11 % built); it will be factory_30335'; 25:47 text names both frames; it ordered a reclaim, not a drop.
- Fix: hands-rule. Confidence: high.

**11. 24:00-24:26, t4 (seat 2, north), corllt_23182 at H3: U4, decayed at 97%, 87 of 90 metal, small**
- What happened: Guard at 580 from spot_64 destroyed by out-of-sight fire (1207 damage in its last second) at 97% while its builder stood on it; the report then said its builder was sent elsewhere and it decayed.
- Evidence: 24:09 woken: 'our soldiers are being shelled from out of sight by a corshiva (range 650) from the north-west'; 24:26 destroyed by nobody at 97%; the record's dmg row 1207; 24:40 report: '24:26 abandoned an unfinished corllt at H3 (97% built): its builder was sent elsewhere and the frame decayed'.
- What the player knew: before: 24:25 report 'building a Guard (corllt) at 580 from spot_64 (H3) (93 % built)'; the abandoned line after misnamed the cause.
- Fix: report. Confidence: high.

**12. 24:36-25:07, t4 (seat 2, north), corllt_640 at H2: U4, decayed at 94%, 85 of 90 metal, small**
- What happened: Guard at spot_41 kept building 560 from a corshiva in sight; one 601-damage hit destroyed it at 94%. The 24:48 orders sent the seat south but cancelled five builders' lists, not this one's.
- Evidence: 24:59-25:06 record: corshiva in sight 552-610 from the site; 25:07 destroyed by nobody, dmg 601; 24:48 queue: commander_t4, 11897, 20919, 27436, 3543 null; 23706 not named.
- What the player knew: before: 24:48 report 'building a Guard (corllt) at spot_41 (H2) (45 % built)'; the 24:48 note 'Shiva shelling at H2 beside our commander'.
- Fix: player. Confidence: high.

**13. 12:45-13:11, t4 (seat 2, north), corhllt_16498 at F1: U4, killed at 43%, 84 of 195 metal, small**
- What happened: Twin Guard at spot_19 with its builder on it; the corgator raid arrived at 13:07 and killed it at 43%.
- Evidence: 13:07 record: first corgator within 700; 13:11 destroyed u16498 by corgator.
- What the player knew: before: 12:59 report 'building a Twin Guard (corhllt) at spot_19 (F1) (26 % built)', no enemy at F1.
- Fix: none. Confidence: high.

**14. 23:38-24:02, t4 (seat 2, north), corllt_1769 at G4: U4, killed at 87%, 78 of 90 metal, small**
- What happened: Guard at G4 built about 150 from the Shiva block (corshiva at 620-1050 and radar contacts at 520); destroyed by fire from out of sight at 87%.
- Evidence: 23:53 report: 'enemy in sight: 4 at G4 ... corshiva 1, unidentified 3'; 24:02 destroyed u1769 by 19676 (type unseen), dmg 830.
- What the player knew: before: 23:53 report 'building a Guard (corllt) at (G4) (55 % built)'.
- Fix: player. Confidence: high.

**15. 15:35-16:45, t3 (seat 1, south), armllt_12363 at F5: U3 (also U4), decayed at 85%, 72 of 85 metal, small**
- What happened: 16:04: 'I pulled constructor_27774 back from spot_129 (F5). It was building a turret there with 3 of green's Stouts close by' (report 64%); ['stop','armsolar','armhlt spot_152 #h1'] landed 16:12 when the frame stood at 85%; the builder died to armstump on its walk home (16:22) and the frame decayed to 22% before a shot finished it (16:45).
- Evidence: 16:04 queue constructor_27774 ['stop','armsolar','armhlt spot_152 #h1'] -> 'its build in progress dropped'; 16:12 stopped on your `stop`; 16:13 (plan) walk to home; 16:22 builder destroyed by armstump.
- What the player knew: before: 16:04 report 'building a Sentry (armllt) at spot_129 (F5) (64 % built)'; it dropped the frame itself, 8 s before the stop landed at 85%.
- Fix: tool. Confidence: high.

**16. 24:32-24:58, t4 (seat 2, north), corllt_21323 at H3: U4, decayed at 80%, 72 of 90 metal, small**
- What happened: the builder's next Guard at spot_64: the builder was killed by out-of-sight fire at 24:53 (1692 damage) and the frame destroyed the same way at 24:58 at 80%.
- Evidence: 24:53 destroyed u21164 (builder) by nobody, dmg 1692; 24:58 destroyed u21323 by nobody, dmg 601; 25:09 report: '24:58 abandoned an unfinished corllt at H3 (80% built): its builder was sent elsewhere and the frame decayed'.
- What the player knew: before: 24:48 report 'building a Guard (corllt) at spot_64 (H3) (61 % built)'; the abandoned line after misnamed the cause.
- Fix: report. Confidence: high.

**17. 4:35-4:56, t2 (seat 0, middle), corllt_11173 at F4: U4, killed at 70%, 63 of 90 metal, small**
- What happened: the list's Guard beside extractor spot_83; 18 Rovers (party_11) reached F4 at 4:51 and killed the frame at 70% with its builder on it; the builder died 4:59. t2 had no picket: the commander's 4:50 transfer had moved its home Incisors to t3.
- Evidence: 4:51 record: 5 armfav within 465 of the site, none before; 4:56 destroyed u11173 by armfav (1762); 4:57 report: 'enemy in sight: 18 at F4 ... armfav 18; killing 2 of our Metal Extractor (cormex), our Guard (corllt)'.
- What the player knew: before: 4:48 report 'constructor_15573: ... building a Guard (corllt) at spot_83 (F4) (47 % built)' with no enemy at F4; 4:57 'lost corllt to armfav in our half x1'.
- Fix: player. Confidence: high.

**18. 9:46-10:04, t4 (seat 2, north), corllt_15540 at F1: U4, killed at 60%, 54 of 90 metal, small**
- What happened: Guard at spot_24 kept building as party_53 (7 corgator) came from 780 to 134; killed at 60%.
- Evidence: 10:04 destroyed u15540 by corgator; builder was told nothing else while the frame stood.
- What the player knew: before: 9:56 report 'building a Guard (corllt) at spot_24 (F1) (35 % built) ... enemies_near: party_53 (7 corgator) 780 away'.
- Fix: player. Confidence: high.

**19. 9:18-9:34, t4 (seat 2, north), corllt_8666 at F1: U4, killed at 54%, 49 of 90 metal, small**
- What happened: Guard at spot_19 with its builder on it; a corgator raid unseen until it struck killed it at 54%.
- Evidence: 9:34 destroyed u8666 by corgator; frame dmg 127 then 269; builder was told nothing else while the frame stood.
- What the player knew: before: 9:23 report 'building a Guard (corllt) at spot_19 (F1) (16 % built)', no enemy at F1.
- Fix: none. Confidence: high.

**20. 16:29-17:02, t4 (seat 2, north), corhllt_528 at H1: U4 (also U5), killed at 24%, 47 of 195 metal, small**
- What happened: Twin Guard at spot_20 on a metal store at 0 (13-19% in 26 s); party_106 (2 corgator) killed it at 24%.
- Evidence: stores min metal 0.0; stalled 8 s before the fate; 17:02 destroyed u528 by corgator.
- What the player knew: before: 16:55 report 'building a Twin Guard (corhllt) at spot_20 (H1) (19 % built) ... enemies_near: party_106 (2 corgator) 287 away'.
- Fix: player. Confidence: high.

The other 35 frames each held under 40 metal. All of them are in unfinished.json, and they break down as follows:

- U1: 1, U3: 10, U4: 24.
- Most are U4: Rover, Rascal and Incisor raids on extractor and turret frames from 3:37 on, and the Shiva fire on t2's refuge from 26:22 to 26:58.
- The ten U3 frames were all deliberate drops: t3's solars in the exit lane with energy full, its redundant home Sentries and its F6 Beamers ahead of green's Stouts, and t2's nano and G5 Guard.
- The single U1 is t2's 0% solar at 10:57.

## Orders never started

frames.py lists 55, 59 and 33 never-started orders, but most are not real misses:
- 86 of the 147 saw a frame of that type rise at the site within 150 s, mostly an id's other builders.
- 21 were placed more than 300 from the command's site. Examples: t2's four coravp #avp1 orders, placed 595 away and built as plant_13514; t3's #h3 and #h5 towers.

I read the costliest fifteen real orders in each seat, 44 in all: U1 4, U3 22, U4 6, U6 3, U8 9.

- **U3, 22 orders:** the player's own re-list or 'stop' replaced the step, usually a queued turret. Examples: t2's 13:20-16:39 re-lists, and t4's #vp2 moved to #vp3 at 23:35.
- **U1, 4 orders:** the pick walked the builder home before it arrived. Example: t2's two home solars at 13:30 during the 29-Incisor raid.
- **U4, 6 orders:** the builder fled a raid and died.
- **U6, 3 orders:** t4's commander, penned at spot_64 by its own wind turbines from 3:12 (15 move_failed). The watchdog refused one of its sites: "never started in the 8 s it stood at the site: the site is blocked".
- **U8, 9 orders:** t3's last builds at H8 at 27:15-27:28, cut off by the surrender's self-destruct at 27:29.

## The three mechanisms

1. One shared id, one frame: the same-tick id race split a shared id over two sites and raised a second, unwanted frame four times (t2 #vp2 corvp_2012, t3 #avp1 armavp_6196, t3 #ap1 armap_30335, t4 #vp3 corvp_4612; 1738 metal), and twice the player's remove of the duplicate never ran: the 'stop','assist' list beside it turned the reclaimer into a helper (armavp_6196) and a 'stop' in the same turn cancelled the list that held the reclaim (armap_30335). The race is the one fixed after this game (2026-10-12, unmeasured); a remove that survives a same-turn 'stop' and never becomes a help step would have taken back most of armavp_6196's 1040 too.
2. One frame at a time on an empty metal store: t3's three Overwatch towers, the moho and the def_west tower (armhlt_28022, armhlt_20798, armhlt_7003, armmoho_12300) stood 1.5-2.5 min at metal 0 with eleven constructors spread over them, two plants and the Starlights, and t4's Twin Guard at spot_15 (corhllt_28232) and t2's nano and H3 Guard did the same; finishing them in turn before the Bulls and the corgator raid arrived would have kept up to 1411 metal (the last-minute 'stop' at 26:59 was the player's sound call once the Bulls stood in the base cell).
3. No frame under a raid that is already there: a list step started, and the hands kept the builder on, frames with a raider standing at the spot or a raid party or Shiva within 600 (t2's H4 extractors beside a sitting Rascal and the F4 Guard with 7 corgator 385 away, t3's spot_162 re-sent three times into 4 Rovers, t4's F2/H1 turrets and the H2/H3/G4 Guards under out-of-sight Shiva fire); holding the step until the spot is clear would have kept about 572 metal. Six frames that out-of-sight fire killed (t2 corvp_2012, corsolar_21511, corwin_18135; t4 corllt_23182, corllt_21323, corllt_640) were reported 'abandoned ... its builder was sent elsewhere and the frame decayed', so the report gave the players the wrong cause.

## Commands run

From the repository root. M = run/matches/1791751159-human-32/00. SCR = the scratch directory, with symlink directories seat1 and seat2.

```
grep -n 'name=' M/script.txt; ls M/; tail -3 M/record-0.jsonl; grep '"k":"chat"' M/record-0.jsonl
run/frames.py M --seat 0|1|2 [--json]
run/floor.py M; run/floor.py SCR/seat1; run/floor.py SCR/seat2
run/impedance_sweep.py M --seat 0|1|2 --all   (saved whole, read by window)
run/hands_window.py M 10:54 11:06 constructor_29566; M 19:55 20:04 constructor_21393; M 24:28 24:56 constructor_29566 constructor_30073
run/hands_window.py SCR/seat1 26:15 26:27 constructor_13601; SCR/seat1 25:53 25:57 constructor_6264; SCR/seat1 22:32 22:48 constructor_2000
run/commander_turns.py M --notes
scratch readers (read-only on M): trace.py (per-second frame share, damage, builder distance, stores, enemies near, events and cmds),
  threat.py (enemies within 700 at start, frame and builder damage), knew.py (the builder's report entry and the frame's lines per turn),
  turns.py / said.py (a seat's prompts and the player's text by clock), unstarted.py (never-started orders against the placed site, with d rows)
```
