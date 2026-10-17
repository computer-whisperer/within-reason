# Unfinished structures: 1791748300-human-30 (Eclipsed 0.9.8)

A realtime 4v2 against two people ([gecko]thebluegecko, lobby skill 34, and [gecko]u6bkep, 19, both Armada): four seats of ours (seats 0-2 Armada, seat 3 Cortex; Opus 5.5 medium players on Jev 1.13.0 under an Opus 5.5 commander). The record ends at 21:45 with "unexpected end of file" and no result line; it was likely a loss (Liche atomic bombers had killed the seat 1 and seat 3 commanders at 21:28 and 21:35, seat 2's at 14:52; whole-game 32,869 metal lost against 18,849 destroyed at 20:15).

## Denominators (frames.py's first line, per seat)

- `1791748300-human-30 seat 0, record to 21:45: frames 38 (decayed 4, finished 31, unfinished at end 3), metal in unfinished frames 109, build orders never started 27`
- `1791748300-human-30 seat 1, record to 21:45: frames 44 (decayed 5, finished 36, killed 2, unfinished at end 1), metal in unfinished frames 1184, build orders never started 17`
- `1791748300-human-30 seat 2, record to 21:45: frames 57 (decayed 3, finished 49, killed 2, unfinished at end 3), metal in unfinished frames 890, build orders never started 35`
- `1791748300-human-30 seat 3, record to 21:45: frames 44 (decayed 3, finished 38, killed 3), metal in unfinished frames 657, build orders never started 22`

Twenty-nine unfinished frames, 2,840 metal in them. `run/floor.py`'s `aband` for seat 0 is 4, equal to frames.py's decayed count. frames.py was edited at 11:29, during this review: its never-started counts for seats 1-3 were 20/37/30 before and are 17/35/22 now (the frames and seat 0 did not change). The lines above are the current tool's.

## Tally (frames only)

| cause | n | metal |
|---|---|---|
| U0 unknown | 0 | 0 |
| U1 pick sent the builder elsewhere | 0 | 0 |
| U2 list moved on | 0 | 0 |
| U3 player re-listed / stopped | 10 | 1029 |
| U4 builder died / fled, frame killed | 10 | 1150 |
| U5 economy | 1 | 384 |
| U6 site | 0 | 0 |
| U7 helpers withdrawn | 0 | 0 |
| U8 game end | 6 | 276 |
| U9 harness fault | 2 | 1 |

| fate | n | metal |
|---|---|---|
| decayed | 15 | 1527 |
| killed | 7 | 653 |
| at_end | 7 | 660 |

Two of the 15 decayed frames were in fact killed by unseen fire (armhlt_28100, armbeamer_13042: the record's damage rows show it). Their fate stays as frames.py reads it, so these sums add up to the denominators.

| type | n | metal |
|---|---|---|
| armadvsol | 3 | 637 |
| armhlt | 2 | 550 |
| armmoho | 1 | 384 |
| armllt | 10 | 332 |
| coradvsol | 2 | 311 |
| cornanotc | 1 | 147 |
| armbeamer | 1 | 114 |
| corhlt | 1 | 96 |
| armsolar | 2 | 78 |
| corrl | 1 | 72 |
| armavp | 1 | 52 |
| corllt | 1 | 31 |
| armrl | 1 | 26 |
| armferret | 1 | 10 |
| armmex | 1 | 0 |

The player's share: the report told the player about 16 frames before the cause acted, 6 only by the abandoned line after, and 7 never. The player's own orders dropped 10 (all of U3). Six of those it meant to drop (wrong site or an ally's spot). The other four it stopped believing the builder was stuck or idle, because the report had lost the frame.

## Items by cost (twenty of twenty-nine)

**armmoho_23112**, seat 2, D1, 18:49 to at_end 21:45 at 62%, 384 of 620 metal, constructor_22150: **U5**. player, large, high.
- What happened: The moho at spot_17 rose to 45% by 19:59 and from 45% to 62% over the last 1:46 with its builder on it: metal store 0 at +14.5 to +19.5 a second shared by ten constructors and two labs, and an energy stall from 19:19 to 19:49. Started 2:56 before the end.
- Evidence: 18:49 created armmoho_23112; the builder 115-149 from it to the end | 19:32 eco 'energy 1/1900 (-275) STALLING'; 20:21 eco 'metal 0 (+17.0/-16.9)' | 21:01 report 'building a Advanced Metal Extractor (armmoho) at spot_17 (D1) (60 % built)'
- What the player knew: before: every entry from 19:12 gave it and its share

**armhlt_28100**, seat 1, H6, 18:17 to decayed 21:27 at 87%, 383 of 440 metal, constructor_5262: **U4** (also U1). player, large, high.
- What happened: The #hlt49 tower at spot_49 took three minutes at metal store 0 (78% at 20:43, 87% at 21:05). At 21:05 the pick walked constructor_5262 to spot_14 under the 20:55 packet's 'They keep away from his Welders: a constructor near them walks to spot_14 and waits there'. The Welders then shot the frame down (85% to 0 in 14 s) and it was destroyed at 21:27 with no attacker named, which is why frames.py reads it as decayed; the builder died at 21:25.
- Evidence: 21:05 (plan) walk to spot_14; its list step 'armhlt spot_49 #hlt49' waits for it | menu 21:04 constructor_5262.go_spot_14: '... stepping away from party_66 (10 armzeus): against it alone, we cannot hit it; leaves the Overwatch (armhlt) at about 90%, which decays'; gate change 0.64, go_spot_14 0.53 | packet 20:55: 'They keep away from his Welders: a constructor near them walks to spot_14 and waits there.'
- What the player knew: before: 20:55 report 'constructor_5262: at spot_50 (H7); building a Overwatch (armhlt) at spot_49 (H6) (83 % built)', the Welders in the same report

**armadvsol_31825**, seat 1, H3, 9:09 to decayed 16:00 at 95%, 332 of 350 metal, constructor_2106: **U3** (also U9). report, large, high.
- What happened: List step 'armadvsol home' (9:00), placed at 6960,2080. At 9:16 another advanced solar (armadvsol_30782, 115 from the site) finished and the UnitFinished rule cleared constructor_2106's task; from 9:23 to 13:54 its entry said 'finishing an order' while it built the frame to 93% at metal store 0 (+18.6/-18.6: 4.5 minutes for a 350-metal frame). At 13:35 the player gave 'two idle constructors' tower lists; at 13:54 it wrote that 2106 and 23326 'had both been stuck on "finishing an order" for about five minutes ... They first drop whatever they were stuck on' and sent a list beginning 'stop', landed 13:58 with the frame at 95%. The builder died at G5 at 15:55.
- Evidence: 9:00 (list) build a armadvsol at home (H3); placed 6960,2080; 9:09 created armadvsol_31825 | 9:16 finished armadvsol_30782 at 6848,2048 | 9:23 to 13:54 reports: 'constructor_2106: at home (H3); finishing an order'
- What the player knew: never: four and a half minutes of 'finishing an order'; no abandoned line at 16:00

**coradvsol_18647**, seat 3, G1, 13:26 to decayed 17:00 at 84%, 311 of 370 metal, constructor_2870: **U3** (also U9). report, large, medium.
- What happened: Shared step 'coradvsol home #as1' for constructor_2870 and constructor_18866 (13:11). At 13:26 the blocked-site watchdog refused 2870's order ('never started in the 8 s it stood at the site') in the very tick 2870 created the frame, and the list made 2870 a repair-helper of its own frame; the entries then said 'repairing our coradvsol' with no share. Re-lists pulled both builders off it: 18866 at 14:22 (left 14:31 at 76%), 2870 at 14:45 (a repair is not a build in progress, so the new list started at once: left 14:54 at 84%), 18866 again at 15:36 (left 15:41 at 58%). It decayed by 17:00 and no line told the player.
- Evidence: 13:26 '13:26 constructor_2870: its coradvsol at G1 never started in the 8 s it stood at the site: the site is blocked ...'; 13:26 created coradvsol_18647 by constructor_2870 | 13:27 (list) help build #as1 (repair) | 14:45 report 'constructor_2870: at home (G1); repairing our coradvsol, since 78 s ago'; 14:45 queue constructor_2870: ["corhlt spot_38 #hlt38"]; 14:54 (list) build a corhlt at spot_38 (F5)
- What the player knew: before, without the share: 'repairing our coradvsol' in 2870's entries at 14:22 and 14:45 and in 18866's at 15:36; the player's texts never mention the frame; no abandoned line at 17:00

**armadvsol_23138**, seat 1, H3, 10:16 to decayed 15:11 at 54%, 189 of 350 metal, constructor_23326: **U3** (also U9). report, large, high.
- What happened: List step 'armadvsol home' (10:08), placed at 7072,2016. Its entry carried other builders' frames: '36 %' at 10:30 was armadvsol_31825's (constructor_2106's; its own stood at 3%) and '96 %' at 13:12 was armadvsol_2503's (constructor_6856's, 113 from its site; its own at 43%). When 2503 finished at 13:16 the task was cleared and the entry said 'finishing an order' from 13:26. The same 13:54 stop dropped it at 54%.
- Evidence: 10:08 (list) build a armadvsol at home (H3); placed 7072,2016; 10:16 created armadvsol_23138 | 13:12 report 'constructor_23326: at spot_24 (H3); building a Advanced Solar Collector (armadvsol) at spot_24 (H3) (96 % built)'; 13:16 finished armadvsol_2503 (by constructor_6856) at 7056,1904 | 13:26 to 13:54 reports 'constructor_23326: at spot_24 (H3); finishing an order'
- What the player knew: never: the share it was shown was another frame's; no abandoned line at 15:11

**armhlt_9506**, seat 1, G5, 14:52 to killed 15:44 at 38%, 167 of 440 metal, constructor_2106: **U4** (also U5). player, large, high.
- What happened: The #hlt40 tower at spot_40 (G5, the army's forward post) rose slowly at metal store 0 (26% to 38% in 30 s). party_61 (4 armzeus) came onto it; the builder stayed until the frame was shot down at 15:44 and died itself at 15:55. Its partner constructor_23326 never reached the site (see the never-started list).
- Evidence: 14:52 created armhlt_9506 by constructor_2106 (#hlt40) | 15:35 report: 'building a Overwatch (armhlt) at spot_40 (G5) (37 % built), ordered 96 s ago; enemies_near: party_61 (4 armzeus) 496 away ... contact in 10 s if it stays' | 15:44 destroyed by 30424 (armzeus); 38% at 15:37, 9% at 15:43
- What the player knew: before: the 15:35 entry gave the frame (37%) and the four Welders 496 away

**cornanotc_15067**, seat 3, G1, 20:33 to killed 21:35 at 64%, 147 of 230 metal, constructor_31221: **U4**. none, large, high.
- What happened: A Liche atomic bomb killed commander_t5 beside the base at 21:35; the commander's death blast (destroyed rows with by = our commander 9782) killed this frame, its builder and the other home frames. The player had seen three Liches at 21:21 and put the base on anti-air.
- Evidence: 21:35 commander 9782 destroyed by 21276 (armliche) | 21:35 cornanotc_15067 destroyed by 9782 (our commander's blast); constructor_2870, 18866 and 31221 the same | 21:35 report 'constructor_31221: ... building a Construction Turret (cornanotc) at home (G1) (64 % built); enemies_near: party_87 (2 armliche) 268 away'
- What the player knew: before: the 21:35 entry and the 21:21 'enemy in sight: ... armliche 3'

**armadvsol_14798**, seat 2, E1, 20:53 to at_end 21:45 at 33%, 116 of 350 metal, constructor_9786: **U8**. none, large, high.
- What happened: Started 52 s before the record ended, at metal store 0; the builder stayed on it.
- Evidence: 20:53 created armadvsol_14798
- What the player knew: before: 21:01 report 'building a Advanced Solar Collector (armadvsol) at back_yard (E1) (5 % built)'

**armbeamer_13042**, seat 2, D4, 19:07 to decayed 19:57 at 60%, 114 of 190 metal, constructor_9724: **U4**. player, large, high.
- What happened: Beamer at spot_31 (D4) shelled from out of sight (the player: 'a Rattlesnake (armamb, 1380 reach, pop-up artillery) shells spot_31 from the south-west (C4)'). The builder took 293 to 450 a hit from 19:43 and died at 19:48; the frame took 112 to 350 a hit from 19:47 and fell from 59% to 0, so it was killed, though no attacker is named and frames.py reads it as decayed. group_H_t4 had walked off toward spot_16 at 19:47.
- Evidence: dmg [13042, 112] 19:47, [13042, 307] 19:52, [13042, 350] 19:54, [13042, 350] 19:57; no enemy in sight | 19:48 constructor_9724 destroyed, attacker unseen | 19:47 report 'building a Beamer (armbeamer) at spot_31 (D4) (54 % built); health badly hurt, near death (21%); under_fire: yes, hit this second by something unseen'
- What the player knew: before: the 19:47 entry; the 20:02 report then carried 'abandoned an unfinished armbeamer (10% built) x1'

**corhlt_18717**, seat 3, G2, 20:50 to killed 21:35 at 20%, 96 of 480 metal, constructor_2870: **U4**. none, small, high.
- What happened: Killed with its builder by the same commander death blast after the Liche bomb.
- Evidence: 21:35 corhlt_18717 and constructor_2870 destroyed by 9782 (our commander's blast) | 21:35 report 'constructor_2870: ... building a Warden (corhlt) ... (20 % built)'
- What the player knew: before: the 21:35 entry

**armllt_18366**, seat 2, D2, 13:08 to killed 14:43 at 92%, 78 of 85 metal, constructor_10429: **U4** (also U5). player, small, high.
- What happened: Turret at spot_20 (D2) built for 82 s at metal store 0 to 89%; a Stout killed the builder at 14:30 (the pick's walk to spot_15 came the same second) and the frame was shot down at 14:43.
- Evidence: 14:30 (plan) walk to spot_15; its list step 'armllt spot_20' waits for it; 14:30 constructor_10429 destroyed by 15571 (armstump) | 14:43 armllt_18366 destroyed by 24672 (armstump)
- What the player knew: before: 14:14 report 'building a Sentry (armllt) at spot_20 (D2) (78 % built)'

**corrl_16784**, seat 3, G2, 21:25 to killed 21:35 at 90%, 72 of 80 metal, commander: **U4**. none, small, high.
- What happened: The commander's own Thistle (anti-air) at 90% when the Liche bomb killed the commander; its blast took the frame.
- Evidence: 21:35 commander 9782 destroyed by armliche; corrl_16784 destroyed by 9782 | 21:35 report 'commander_t5: ... building a Thistle (corrl) at 420 from home (G2) (90 % built) ... party_87 (2 armliche) 20 away'
- What the player knew: before: the 21:35 entry

**armllt_2038**, seat 2, B1, 8:47 to killed 9:45 at 80%, 68 of 85 metal, constructor_20838: **U4** (also U5). player, small, high.
- What happened: Turret at spot_7 (B1, in his start box; its extractor step skipped because 'spot_7 is being taken by a builder of another seat of ours'). 58 s at metal store 0 brought it to 77%; party_30 (2 armstump) arrived at 9:44 and killed it at 80%.
- Evidence: 8:24 (list) build a armllt at spot_7 (B1); 'skipped the step 'extractor spot_7' of its list: spot_7 is being taken by a builder of another seat of ours' | 8% at 8:54, 77% at 9:39, m 0/1650 | 9:45 destroyed by 18706 (armstump)
- What the player knew: before: 9:30 report 'building a Sentry (armllt) at spot_7 (B1) (59 % built)'; the Stouts were not in sight until 9:44

**armllt_20968**, seat 1, H7, 7:25 to decayed 8:48 at 72%, 61 of 85 metal, constructor_10982: **U3** (also U9). report, small, high.
- What happened: The 5:52 list step 'armllt spot_50 #t50' walked 93 s and the engine started the frame at 6864,5440 at 7:25. The hands had taken constructor_5262's turret armllt_17443 (181 away) as this builder's build ('building a Sentry (armllt) at spot_50 (H7) (94 % built)' at 7:30) and cleared its task when 17443 finished at 7:35 (pianist/mod.rs, Event::UnitFinished: any finished unit of the build's type within 250 of the task's site ends the task). From 7:39 its entry said 'finishing an order' while the frame rose to 72%; the player took it for stuck and sent ['stop', solars] at 8:00, landed 8:13.
- Evidence: 7:25 created armllt_20968 (from the 5:52 order; no new command) | 7:35 finished armllt_17443 (by constructor_5262) 181 from the site | 7:39, 7:47, 8:00 reports: 'constructor_10982: at spot_50 (H7); finishing an order'
- What the player knew: never: the frame never appeared in its entry, and no abandoned line came at 8:48

**armllt_1998**, seat 0, E2, 20:38 to at_end 21:45 at 69%, 59 of 85 metal, constructor_15980: **U8** (also U5). none, small, high.
- What happened: Started 67 s before the record ended; the builder stayed on it throughout at metal store 0 (+12.2/-12.2), the 85-metal turret rising about 1% a second. The 21:32 list help step was queued behind it and never pulled the builder.
- Evidence: 20:38 created armllt_1998; 2% at 20:40, 68% at 21:43; m 0/1450 (+12.2/-12.2) throughout | 21:32 (list) help build #e2t (next, queued: repair)
- What the player knew: before: 21:29 report 'constructor_15980: at e2_field (E2); building a Sentry (armllt) at e2_field (E2) (58 % built)'

**armavp_13123**, seat 2, D1, 15:21 to decayed 15:57 at 2%, 52 of 2600 metal, constructor_4475: **U3** (also U6). tool, large, high.
- What happened: The shared step 'armavp #avp2' (14:56) had no place: constructor_4475 placed its plant at spot_20 (D1, the front of the extractor row) while the other four got no_site at back_yard. The player saw it at 15:24 and stopped it ('The advanced vehicle plant had started in the wrong place again. Last turn I left the site to the bot ... it was at 0%, so almost nothing is lost'), landed 15:39. The four helpers the list sent to repair it at 15:27 were about 1000 away and never reached it. The plant was started at back_yard (572) at 15:48.
- Evidence: 15:18 (list) build a armavp at spot_20 (D1) (#avp2); placed 3400,840; 15:21 created armavp_13123 | 15:24 report 'constructor_4475: at spot_20 (D1); building a Advanced Vehicle Plant (armavp) at spot_20 (D1) (0 % built)' | 15:24 queue constructor_4475: "stop" -> 'list cancelled and its build in progress dropped'; 15:39 stop; 15:41 (plan) reclaim wrecks at e2_gate
- What the player knew: before: the 15:24 entry; it dropped the frame on purpose

**armsolar_23869**, seat 2, E1, 20:35 to decayed 21:12 at 27%, 42 of 155 metal, constructor_14540: **U3**. tool, small, high.
- What happened: The 20:21 turn stopped the four constructors 'still walking 800 to 1,400 across the map to the basic solars I cancelled last turn'; constructor_14540 began its solar at spot_8 (E1) at 20:35 while the 31 s turn ran. The stop landed at 20:52 with the frame at 27%, and the pick sent the idle builder to help the moho at 20:53.
- Evidence: 20:21 report 'constructor_14540: at back_yard (E1); walking to build a Solar Collector (armsolar) at spot_8 (E1), 839 to go' | 20:21 queue constructor_14540: "stop" | 20:35 created armsolar_23869; '20:52 constructor_14540: stopped what it was doing on your `stop`'
- What the player knew: never: it began after the report and was gone before the next (21:01); no abandoned line

**armsolar_23768**, seat 2, E2, 21:31 to at_end 21:45 at 23%, 36 of 155 metal, constructor_25929: **U8**. none, small, high.
- What happened: Started 14 s before the record ended.
- Evidence: 21:31 created armsolar_23768
- What the player knew: never: seat 2's last report was at 21:01

**corllt_11059**, seat 3, D4, 18:45 to decayed 19:21 at 35%, 31 of 90 metal, constructor_2870: **U3**. player, small, high.
- What happened: The player stopped it on purpose at 18:51 ('I stopped its light turret frame at D4 (9% built), next to his Beamers'); the stop landed at 18:59 with the frame at 34% and the pick sent the builder to reclaim at D2.
- Evidence: 18:51 report 'building a Guard (corllt) at (D4) (9 % built)' | 18:51 queue constructor_2870: ["stop"] | 18:59 stop; 19:01 (plan) reclaim wrecks at (D2)
- What the player knew: before: the 18:51 entry, and it meant to drop it

**armllt_20518**, seat 0, E2, 21:11 to at_end 21:45 at 34%, 29 of 85 metal, constructor_27770: **U8** (also U5). none, small, high.
- What happened: Started 34 s before the record ended beside armllt_1998, at metal store 0; its builder stayed on it.
- Evidence: 21:11 created armllt_20518; m 0/1400 to the end | 21:17 report 'constructor_27770: ... building a Sentry (armllt) at e2_field (E2) (44 % built)': the share is armllt_1998's, the entry named the neighbouring frame
- What the player knew: before: the 21:17 and 21:29 entries said it was building a Sentry at e2_field, with the neighbour's share

The other nine, each 26 metal or less: armrl_14382 (seat 1, 21:38, U8, 26 m); armllt_11869 (seat 1, 19:11, U4, 25 m); armllt_28016 (seat 0, 8:22, U3, 10 m); armferret_15987 (seat 0, 21:36, U8, 10 m); armllt_29241 (seat 0, 8:12, U3/U9, 1 m); armllt_30891 (seat 1, 6:03, U9/U3, 1 m); armllt_13977 (seat 0, 9:20, U9/U3, 0 m); armmex_9213 (seat 0, 9:47, U4/U1, 0 m); coradvsol_21032 (seat 3, 9:50, U3/U9, 0 m). Their entries are in unfinished.json.

## Orders never started

101 orders across the four seats. 14 are not items: long walks that then started the same build, and id partners whose id's frame stood. The other 87 were classified: U3 36, U6 21, U4 12, U1 9, U2 5, U8 3, U5 1. 59 of them were read deeply (unfinished.json `never_started`). The ones that matter:

- **Seat 2's advanced vehicle plant: 14 orders over 2:03 for one 2,600-metal plant, 13 of them U6 and one U3.** 13:36, four builders on 'armavp home #avp': no_site. No hands' line said it; the builders stood 38 s in plant_25770's exit lane, and the player moved the site at 14:14 ('site likely unreachable'). 14:29, five on #avp1 at avp_yard: 'the engine refused a armavp at G1: the site was bad'. 15:18, the place-less #avp2: four got no_site and then the watchdog's 'site is blocked', constructor_14540's site was replaced by the player's 15:24 re-list (U3), and constructor_4475 placed its plant at the front (item armavp_13123). The plant was finally started at back_yard at 15:48.
- **Seat 1, 13:59, constructor_23326, #hlt40 (440), U6.** It stood 159 short of the tower its partner had started ('159 to go' at 15:09 and 15:17) and never helped. The pick walked it off at 15:22, and the frame (armhlt_9506) was killed at 15:44.
- **Seat 1, 19:35, two #hlt51 towers (440 each), U3.** The player cancelled them at 20:15 because the walk 'ran right past the Welders'.
- **Seat 3, 14:54, two corhlt at spot_38 (480 each), U3.** The player's 15:04 re-list moved the tower home. **Seat 3, 14:31-15:27, two corhllt (195 each), U6:** 'the engine refused a corhllt at G2: the site was bad'.
- **Turret steps beside allied extractors (most of the U3).** The lists skipped extractor steps ('spot_X is not free', 'being taken by a builder of another seat of ours') and walked on to the turret step beside the ally's extractor. The player then stopped them: seat 0's 8:15 packet says 'never builds a turret beside an extractor that is not ours'. These are 3:19, 3:38, 4:22, 7:36, 7:45 and 8:14 in seat 0, 5:41 in seat 1, and 2:06 and 2:53 in seat 3.
- **'Taken as done' with no frame of the builder's own (U2):** seat 0 0:18 (commander, spot_6) and 8:02; seat 1 3:32 (spot_38); seat 2 5:00; seat 3 19:13 (corhllt). The seat 2 case was right, since another builder's frame stood. The others are low-confidence readings.

## The three mechanisms

1. The report lost the builder's own frame (seat 1 and seat 3, 893 metal): the hands end a builder's build task when any of our units of that type finishes within 250 of the task's site (crates/bot/src/brain/pianist/mod.rs, the Event::UnitFinished arm), so a neighbour's frame finishing made the entry read 'finishing an order' (or carry the neighbour's share) while the builder kept building, and the player stopped the 'stuck' builders: armadvsol_31825 (332), armadvsol_23138 (189), armllt_20968 (61); and in seat 3 the starter of coradvsol_18647 (311) was demoted to a repair-helper of its own frame by a watchdog refusal in the tick it created it, its entry said 'repairing our coradvsol' with no share, and re-lists dropped it because a repair is not a build in progress. Matching the task to the builder's own `created` frame (and keeping a repair-help frame in force across a re-list, or saying 'leaves the X at N%' in the queue result) keeps all four.
2. Turret frames started inside an enemy's reach while the metal store sat at 0, then shot down or left to the threat (seats 1 and 2, 835 metal): armhlt_28100 (383, left by the packet's Welder rule at 87% and shot down), armhlt_9506 (167), armbeamer_13042 (114, unseen artillery), armllt_18366 (78), armllt_2038 (68), armllt_11869 (25); each stood one to three minutes at store 0, so the threat arrived before the frame finished. The fix is the player's: fewer, covered turret frames at the front, or the metal concentrated on one at a time.
3. Metal starvation at the end (seat 2, 384 metal recoverable): armmoho_23112 stood 2:56 with its builder on it at metal store 0 (+14.5 to +19.5 a second over ten constructors and two labs, an energy stall 19:19-19:49) and ended at 62%; the at-end frames armllt_1998, armllt_20518 and armadvsol_14798 (204) stood under the same store. Smaller, by seam (54 metal): a stop or re-list written against a walking builder landed 10-31 s later, after the frame began (armllt_28016, armsolar_23869), and the old list's queued-ahead step stayed in the engine queue after a re-list and was started then dropped or duplicated (armllt_29241, armllt_13977, armllt_30891, coradvsol_21032).

## Commands run (from the repository root; MATCH = run/matches/1791748300-human-30/00)

- `grep -n 'name=' MATCH/script.txt`; `ls MATCH`; the record's chat rows; `tail MATCH/bot.log`; the commander's last report
- `run/frames.py MATCH --seat N` and `--json` for N = 0..3 (re-run after the tool changed at 11:29); `run/floor.py MATCH`
- `run/hands_window.py MATCH m:ss m:ss constructor_N` (8:10-8:16, 9:44-9:50 seat 0)
- `run/impedance_sweep.py MATCH --seat N --from m:ss --to m:ss --all` (seat 0 6:30-8:45; seat 2 13:20-15:00; seat 3 9:30-10:05, 13:00-15:50, 18:00-19:25)
- `run/commander_turns.py MATCH --notes`
- `crates/bot/src/brain/pianist/mod.rs` (the Event::UnitFinished arm) and `picture.rs` (task_course: 'finishing an order') read for the mechanism
- scratch readers (scratch directory `unfinished/tools/`): `trace.py` (a frame's share, its builder's distance, the stores, the builder's cmd/ev/d rows from record-N), `menu.py` (an actor's menus, gate flags and picks from jev-N.jsonl), `turns.py` and `say.py` (the report lines, tool calls, text and notes of strategist-N), `unstarted.py` and `handlines.py` (each never-started order's decision row, later frames near the site, and the hands' lines and queue calls about it)
- symlink directories for seats 1-3 (`record-0`, `jev-0`, `strategist-0`, `terrain-0` pointing at seat N's files) under the scratch directory `unfinished/1791748300-human-30/seatN`; seat 0 read in place
