# Unfinished structures: 1791643805-glitters-8v8-opus-1

All That Glitters v2.2.3, realtime 8v8 (2026-10-10, commit 8c770ab): eight Opus 5.5 medium players under an Opus 5.5 high commander against eight BARb hard_aggressive; undecided at the 30:00 cap (Timeout), all sixteen commanders alive. All eight seats read; 126 unfinished frames, 8,322 metal in them.

## Denominators (frames.py, first line per seat)

- `1791643805-glitters-8v8-opus-1 seat 0, record to 30:00: frames 137 (decayed 7, finished 107, killed 17, unfinished at end 6), metal in unfinished frames 1769, build orders never started 43`
- `1791643805-glitters-8v8-opus-1 seat 1, record to 30:00: frames 108 (decayed 5, finished 97, killed 3, unfinished at end 3), metal in unfinished frames 1353, build orders never started 27`
- `1791643805-glitters-8v8-opus-1 seat 2, record to 30:00: frames 104 (decayed 4, finished 93, killed 7), metal in unfinished frames 418, build orders never started 16`
- `1791643805-glitters-8v8-opus-1 seat 3, record to 30:00: frames 92 (decayed 4, finished 81, killed 7), metal in unfinished frames 531, build orders never started 37`
- `1791643805-glitters-8v8-opus-1 seat 4, record to 30:00: frames 122 (decayed 12, finished 99, killed 7, unfinished at end 4), metal in unfinished frames 1452, build orders never started 39`
- `1791643805-glitters-8v8-opus-1 seat 5, record to 30:00: frames 100 (decayed 6, finished 87, killed 7), metal in unfinished frames 624, build orders never started 33`
- `1791643805-glitters-8v8-opus-1 seat 6, record to 30:00: frames 109 (decayed 2, finished 104, unfinished at end 3), metal in unfinished frames 453, build orders never started 44`
- `1791643805-glitters-8v8-opus-1 seat 7, record to 30:00: frames 85 (decayed 4, finished 63, killed 12, unfinished at end 6), metal in unfinished frames 1722, build orders never started 26`

`run/floor.py` gives seat 0 `aband` 7, equal to frames.py's decayed count. Of the 44 frames frames.py calls decayed, 16 (352 metal) were destroyed by fire from out of sight while their builder stood on them (damage on the frame in the second it died, last progress under 9 s before): fate kept as `decayed`, cause U4.

## Tally

| cause | n | metal in |
|---|---|---|
| U0 unknown | 0 | 0 |
| U1 the pick sent the builder elsewhere | 5 | 244 |
| U2 the list moved on | 1 | 10 |
| U3 the player re-listed or moved it | 16 | 931 |
| U4 builder died/fled or frame killed | 77 | 2241 |
| U5 economy | 7 | 1387 |
| U6 the site | 0 | 0 |
| U7 helpers withdrawn | 0 | 0 |
| U8 game end | 16 | 2648 |
| U9 harness fault | 4 | 861 |

| fate | n | metal in |
|---|---|---|
| decayed | 44 | 2459 |
| killed | 60 | 1847 |
| at_end | 22 | 4016 |

| type | n | metal in |
|---|---|---|
| armpb | 6 | 1884 |
| armadvsol | 8 | 1559 |
| armmoho | 7 | 1475 |
| armllt | 50 | 1227 |
| armsolar | 7 | 678 |
| armmmkr | 5 | 612 |
| armmex | 27 | 344 |
| armrl | 11 | 311 |
| armnanotc | 2 | 212 |
| armwin | 2 | 20 |
| armmakr | 1 | 0 |

What the player knew: the report listed the frame before the cause acted for 83 of 126 (the 16 U8 frames among them); 29 were told only after (an abandoned or lost line); 14 never. 16 were dropped by the player's own order (15 U3 and the U5 converter it re-listed): 11 of them frames its report had shown it (seven because the builder stood in a plant's exit lane, mechanism 2), 5 begun or queued after the report it acted on, under a list that began with `stop` or replaced the old list (fix `tool`).

U4 in detail (77 frames, 2,241 metal): 63 at forward sites in rows 3-5 (1,306 metal; 31 of them in row 5, his half), Sentries, extractors and Nettles killed by raiders while the builder stood on them, the threat already in the builder's entry or the enemy-in-sight lines in most (fix `player` 56 of the 77); 14 in the home rows (935 metal), six of them the one raid of mechanism 3 (780), the rest early raids at 1:52-3:03 and the H2 raid at 25-27 min. Fix `report` 12: frames killed by fire from out of sight that the hands then reported as 'abandoned an unfinished X'. Fix `hands-rule` 5: the five extractor frames at E5 spot_77 (seat 4, 12:25-15:12, 107 metal) that the pick ordered four times and the list once more under artillery fire its move's words never mentioned.

## Items by metal in (the twenty costliest)

**29:12-30:00 seat 1 armpb_31419 at F1, at_end at 95%, 646 metal: U8**, constructor_29204.
- begun 29:12, 48 s before the record ended; its builder on it to the end (95%)
- Evidence: 29:12 created; unfinished at end 30:00 at 95%
- Player knew: yes: the 29:35 report: 'constructor_29204: at spot_14 (F1); building a Pit Bull (armpb) at spot_14 (F1) (47 % built), ordered 32 s ago; last_pick: 28:50 picked help'
- Fix `none`, cost `none`, confidence high.

**28:29-30:00 seat 4 armpb_23379 at D2, at_end at 92%, 626 metal: U8**, constructor_28678.
- begun 28:29, 91 s before the record ended; its builder on it to the end (92%)
- Evidence: 28:29 created; unfinished at end 30:00 at 92%
- Player knew: yes: the 29:54 report: 'constructor_28678: at pb_site (D2); building a Pit Bull (armpb) at pb_site (D2) (80 % built), ordered 95 s ago; last_pick: 24:09 picked help'
- Fix `none`, cost `none`, confidence high.

**22:34-30:00 seat 7 armmoho_1655 at B2, at_end at 92%, 570 metal: U5**, constructor_26177.
- 7.5 min with its builder on it, 92% at the end; seat 7 held 2 extractors from about 22:00 (A3/B2 lost to raids) at +5.7 metal/s ('eco: metal 0 (+5.7/-5.5)' at 25:06) and kept two to three economy frames going at once; metal store under 3% for 100% of its life
- Evidence: metal store share < 3% for 100% of its 446 s; income ~6/s; 27:11 a second builder told to help it ('help finish the armmoho already under way at spot_28')
- Player knew: yes: the 26:45 report listed it at 56%
- Fix `player`, cost `large`, confidence high.

**29:24-30:00 seat 1 armpb_31115 at E1, at_end at 68%, 462 metal: U8**, constructor_27463.
- begun 29:24, 36 s before the record ended; its builder on it to the end (68%)
- Evidence: 29:24 created; unfinished at end 30:00 at 68%
- Player knew: yes: the 29:35 report: 'constructor_27463: at spot_20 (F1); building a Pit Bull (armpb) at spot_20 (E1) (20 % built), ordered 32 s ago; last_pick: 28:50 picked help'
- Fix `none`, cost `none`, confidence high.

**28:06-30:00 seat 0 armmoho_11613 at B1, at_end at 54%, 335 metal: U8**, constructor_2622.
- begun 28:06, 114 s before the record ended; its builder on it to the end (54%)
- Evidence: 28:06 created; unfinished at end 30:00 at 54%
- Player knew: yes: the 29:57 report: 'constructor_2622: at home (A1); building a Advanced Metal Extractor (armmoho) at spot_15 (B1) (53 % built), ordered 134 s ago; last_pick: 22'
- Fix `none`, cost `none`, confidence high.

**17:08-21:46 seat 0 armadvsol_28728 at B2, decayed at 92%, 322 metal: U9 (also U2)**, constructor_30349.
- 18:48 its list queued the armmakr step behind the advsol (60% queue-ahead); 19:39 a neighbour's armadvsol (unit 10512, constructor_16706's, 112 away) finished and the hands' UnitFinished handler (it matches any started build of the same type within 250) promoted this builder's queued armmakr as if its own advsol had finished; 8 s later the blocked-site watchdog saw the builder 'standing at' the armmakr site with no armmakr frame and stopped it, dropping the advsol at 92% (the report's 99% was the neighbour's share); decayed to 21:46
- Evidence: 19:39 ev finished u=10512 armadvsol at (896,1280), 112 from the frame; 19:48 cmd stop 30349; 19:48 hands: 'its armmakr at B2 never started in the 8 s it stood at the site: the site is blocked ..., refused for 90 s'
- Player knew: yes: the 19:39 report: constructor_30349 'building a Advanced Solar Collector (armadvsol) at 400 from spot_17 (B2) (99 % built)'; the 19:54 report carried only the watchdog line about the armmakr; told at 21:52 'fights since last turn: abandoned an unfinished armadvsol (0% built) x1'
- Fix `hands-rule`, cost `large`, confidence high.

**22:46-30:00 seat 0 armmmkr_6066 at B2, at_end at 82%, 312 metal: U5**, constructor_19390.
- 7 min with its builder on it: seat 0's metal store was under 3% for the whole of its 434 s at about 18 metal/s (the home raid of 21:47-22:16 had taken its B1/A1 extractors); 82% at the end
- Evidence: metal store share < 3% for 100% of its life; income ~18/s; 28:19 (list) next, queued: build a armmmkr at spot_24 (B2) (the next converter waited behind it to the end)
- Player knew: yes: the 27:52 report listed it at 51%
- Fix `player`, cost `large`, confidence high.

**17:26-22:35 seat 5 armadvsol_15472 at H2, decayed at 81%, 284 metal: U9 (also U5)**, constructor_16338.
- slow under a metal stall (81% in 3.5 min, metal store under 3% the whole time); 19:57 its list queued 'help #avp1' and at 20:18 the 'assist' step behind it; 20:49 a neighbour's armadvsol (unit 5087, 144 away) finished and the UnitFinished handler promoted this builder's queued assist as if its own frame had finished: the 20:50 guard on plant_29504 took it off its 81% advsol, which decayed to 22:35. The hands' picture showed it '(98 % built)' at 20:48 and '(100 % built)' at 20:49 (the share of the best same-type frame within 200)
- Evidence: 20:49 ev finished u=5087 armadvsol 144 from the frame; 20:50 cmd guard 16338 29504 (no decision row: the promotion); jev picture 20:49: constructor_16338 'building a Advanced Solar Collector (armadvsol) at spot_37 (H2) (100 % built)'; 20:50: 'helping lab_29504 build, since 0 s ago'
- Player knew: yes: the 19:43 report listed it building (61% in the entry); told after only as 'abandoned an unfinished armadvsol (0% built)'
- Fix `hands-rule`, cost `large`, confidence high.

**17:39-22:20 seat 7 armadvsol_30083 at B2, killed at 69%, 241 metal: U4 (also U5)**, constructor_14453.
- killed by corpyro at 69% at B2 (our home row) while its builder stood on it (139 away)
- Evidence: 22:20 killed by corpyro at 69%; dmg on the frame 22:19 265, 22:20 340, 22:21 189; enemies at 22:21: corpyro 119, corpyro 139
- Player knew: yes: the 22:15 report: 'constructor_14453: at spot_24 (B2); building a Advanced Solar Collector (armadvsol) at spot_24 (B2) (68 % built), ordered 303 s ago; enemies_near: party_645 (1 corpyro) 7'; the threat was in it; told after: 'fights since last turn: lost armadvsol to corpyro at home x1'
- Fix `player`, cost `large`, confidence high.

**13:47-17:45 seat 6 armadvsol_14766 at G1, decayed at 62%, 217 metal: U9**, constructor_4341.
- 16:19 its new list ['armrl spot_22 #rl22'] queued 'help #rl22' behind the advsol; 16:21 a neighbour's armadvsol (unit 4784, 121 away) finished and the UnitFinished handler promoted the queued help: the 16:22 guard on constructor_22943 took the builder off its 62% advsol; 16:49 the pick sent it to help constructor_6518; decayed to 17:45
- Evidence: 16:21 ev finished u=4784 armadvsol 121 from the frame; 16:22 cmd guard 4341 22943; 16:19 (list) help build #rl22 (next, queued: help constructor_22943 build)
- Player knew: yes: the 16:07 report listed constructor_4341 building it ('88 % built', another frame's share); its 16:07 queue did not begin with stop (it meant the advsol to finish); told after as 'abandoned an unfinished armadvsol (0% built)'
- Fix `hands-rule`, cost `large`, confidence high.

**24:24-30:00 seat 7 armadvsol_30305 at B3, at_end at 61%, 214 metal: U5**, constructor_26980.
- 5.5 min with its builder on it, 61% at the end; seat 7 held 2 extractors from about 22:00 (A3/B2 lost to raids) at +5.7 metal/s ('eco: metal 0 (+5.7/-5.5)' at 25:06) and kept two to three economy frames going at once
- Evidence: metal store share < 3% for 100% of its 336 s; income ~6/s
- Player knew: yes: the 29:34 report listed it at 57%
- Fix `player`, cost `large`, confidence high.

**5:39-7:38 seat 4 armnanotc_12832 at E1, decayed at 73%, 168 metal: U3**, constructor_21877.
- the player re-listed constructor_21877 ['stop', 'armwin', 'armwin', ...] at 6:20 because it stood in plant_30852's exit lane; the stop landed 6:31 with the nano at 73%; decayed to 7:38
- Evidence: 6:20 queue constructor_21877: ["stop", "armwin", "armwin", "extractor spot_39 #m39", "armllt spot_39 #t39b"]; 6:31 cmd stop 21877; 6:32 (list) build a armwin at spot_21 (D1); 7:38 decayed, metal in 168 of 230
- Player knew: yes: the 6:20 report: 'building a Construction Turret (armnanotc) at spot_16 (E1) (56 % built) ...; standing in plant_30852's exit lane since 40 s ago'; dropped it itself
- Fix `hands-rule`, cost `large`, confidence high.

**20:35-22:23 seat 7 armmoho_27594 at B2, killed at 25%, 155 metal: U4**, constructor_26177.
- killed by corpyro at 25% at B2 (our home row) while its builder stood on it (140 away)
- Evidence: 22:23 killed by corpyro at 25%; dmg on the frame 22:22 50, 22:23 474, 22:24 214; enemies at 22:24: corpyro 75
- Player knew: yes: the 22:23 report: 'constructor_26177: at spot_24 (B2); building a Advanced Metal Extractor (armmoho) at spot_24 (B2) (7 % built), ordered 187 s ago; health lightly hurt (91%); enemies_near:'; the threat was in it
- Fix `player`, cost `large`, confidence high.

**9:40-10:45 seat 3 armsolar_15818 at F2, decayed at 97%, 150 metal: U3**, commander.
- woken by the exit-lane line, the player re-listed the commander ['stop', 'armsolar spot_38', 'armsolar spot_25'] at 9:42 to clear plant_1272's lane; the stop landed 16 s later, at 9:58, with the solar at 97%; decayed to 10:45
- Evidence: 9:42 queue commander_t3: ["stop", "armsolar spot_38", "armsolar spot_25"]; 9:58 cmd stop 7039; 9:59 (list) build a armsolar at spot_38 (G2); 10:45 decayed, metal in 150 of 155
- Player knew: yes: the 9:42 report: 'building a Solar Collector (armsolar) at spot_35 (F2) (11 % built), ordered 2 s ago; standing in plant_1272's exit lane since 22 s ago'; dropped it itself (11% when it decided, 97% when the stop landed)
- Fix `hands-rule`, cost `large`, confidence high.

**20:36-22:42 seat 4 armmoho_2146 at D1, killed at 24%, 149 metal: U4 (also U5)**, constructor_13752.
- killed by corak at 24% at D1 (our home row) while its builder stood on it (132 away)
- Evidence: 22:42 killed by corak at 24%; dmg on the frame 22:40 81, 22:41 81, 22:42 81, 22:43 63; enemies at 22:43: corak 177
- Player knew: yes: the 22:38 report: 'constructor_13752: at spot_4 (D1); building a Advanced Metal Extractor (armmoho) at spot_4 (D1) (14 % built), ordered 179 s ago; enemies_near: party_645 (1 corak) 298 awa'; the threat was in it; told after: 'fights since last turn: lost armmoho to corak at home x1'
- Fix `player`, cost `large`, confidence high.

**29:15-30:00 seat 1 armmmkr_31952 at F1, at_end at 37%, 141 metal: U8**, constructor_6518.
- begun 29:15, 45 s before the record ended; its builder on it to the end (37%)
- Evidence: 29:15 created; unfinished at end 30:00 at 37%
- Player knew: yes: the 29:35 report: 'constructor_6518: at spot_7 (F1); building a Advanced Energy Converter (armmmkr) at spot_7 (F1) (16 % built), ordered 32 s ago; last_pick: 2'
- Fix `none`, cost `none`, confidence high.

**27:12-30:00 seat 7 armmoho_18092 at B2, at_end at 22%, 136 metal: U5**, constructor_9543.
- begun 27:12 (2:48 before the end), 22% at the end; seat 7 held 2 extractors from about 22:00 (A3/B2 lost to raids) at +5.7 metal/s ('eco: metal 0 (+5.7/-5.5)' at 25:06) and kept two to three economy frames going at once
- Evidence: metal store share < 3% for 100% of its 168 s; income ~6/s; 27:12 (list) build a armmoho at spot_24 (B2) (#m24)
- Player knew: yes: the 29:34 report listed it at 18%
- Fix `player`, cost `large`, confidence high.

**6:52-8:03 seat 5 armsolar_16263 at H3, decayed at 87%, 135 metal: U3**, constructor_3926.
- the player re-listed constructor_3926 ['stop', 'extractor spot_72', ...] at 7:02 because it stood in plant_23270's exit lane; the stop landed 18 s later, at 7:20, with the solar at 87%; decayed to 8:03
- Evidence: 7:02 queue constructor_3926: ["stop", "extractor spot_72", "armllt spot_72", ...]; 7:20 cmd stop 3926; 7:21 (list) build a armllt at spot_72 (G4); 8:03 decayed, metal in 135 of 155
- Player knew: yes: the 7:02 report: 'building a Solar Collector (armsolar) at spot_37 (H3) (30 % built) ...; standing in plant_23270's exit lane since 33 s ago'; dropped it itself
- Fix `hands-rule`, cost `large`, confidence high.

**2:12-3:22 seat 3 armsolar_30896 at F2, decayed at 86%, 133 metal: U1 (also U3)**, commander.
- 2:35 the pick sent the commander after party_22 (3 corfav killing the other solar) with the solar at about 70% ('leaves the Solar Collector (armsolar) at about 70%, which decays', gate 0.39-0.59, pick 0.72); the player's 2:31 list (no solar step) landed at 2:39 and replaced the old list, so the waiting 'armsolar' step was gone and the commander never went back; decayed to 3:22
- Evidence: 2:35 PICK commander_t3: took w2 of 1 at 0.72; (plan) attack party_22; its list step 'armsolar' waits for it; 2:39 (list) build a armmex at spot_38 (G2); 3:22 'fights since last turn: abandoned an unfinished armsolar (0% built)'
- Player knew: yes: the 2:31 report: 'building a Solar Collector (armsolar) at spot_35 (F2) (57 % built)'; the 2:44 report carried the pick's line; told after by the abandoned line
- Fix `hands-rule`, cost `large`, confidence high.

**15:47-17:45 seat 2 armadvsol_5052 at E2, decayed at 37%, 130 metal: U3**, constructor_28739.
- woken by 'plant_13596's exit lane is blocked: armcv_28739 stood in it for 0:30', the player re-listed it ['stop', 'armrl spot_41', 'armadvsol spot_26'] at 16:42; the stop landed 16:51 with the advsol at 37%; decayed to 17:45
- Evidence: 16:42 queue constructor_28739: ["stop", "armrl spot_41", "armadvsol spot_26"] -> 'its build in progress dropped'; 16:51 stopped what it was doing on your `stop`; 16:52 (list) build a armrl at spot_41 (D3); 17:45 decayed, metal in 130 of 350
- Player knew: yes: the 16:42 report: 'building a Advanced Solar Collector (armadvsol) at 311 from spot_39 (E2) (33 % built) ...; standing in plant_13596's exit lane since 39 s ago'; dropped it itself
- Fix `hands-rule`, cost `large`, confidence high.

The remaining 106 are in `unfinished.json`.

## Orders never started

frames.py lists 265; 104 had a frame of that type begun by one of ours at the ordered or the engine's placed site within 90 s (id'd buildings with several builders; the armavp placed 317 from its ordered site), 12 began later from the same order, 11 were issued in the last 90 s. Rough causes of the other 138: U3 player stop / new list 43 (6900 metal cost); U6 rejected (engine refused the site) 38 (3110 metal cost); U2/U3 next list step (list moved on or a new list) 28 (3205 metal cost); U4 builder died 11 (1250 metal cost); U6 stuck 7 (935 metal cost); U1 the pick replaced it 6 (475 metal cost); U3 new list dropped the queued step (tool) 3 (250 metal cost); U0 other 1 (350 metal cost); U0 no later command 1 (380 metal cost). The ten costliest:

- 22:23 seat 0 constructor_2622 armpb at B3: U3, walking from home to spot_45 (1611 to go at 22:37); the player's 22:37 list ['armpb spot_23', 'armmoho spot_17', 'armmoho spot_15'] replaced it before it arrived (landed 22:45)
- 27:40 seat 5 constructor_25391 armpb at H2: U1, six seconds after the order the pick walked the builder to refuge from the H2 raid (armstil, armstump), then the player's stop ended the list
- 25:44 seat 6 constructor_20352 armpb at H1: U3, the pick's Pit Bull order was replaced 11 s later by the player's list (#pb3 at spot_22), whose Pit Bull stood 26:43-26:58: replaced, nothing lost
- 26:08 seat 1 constructor_27463 armmoho at F3: U3, queued behind the builder's build in progress (#m51); the player's stop at 26:35 ended it before its turn came
- 24:48 seat 3 constructor_12809 (+12973, 18631: one #m56 building) armmoho at G3: U3, three builders sent to one id'd moho at spot_56; the player's stop 14 s later replaced the list before any arrived
- 25:14 seat 5 constructor_29415 armhlt at H3: U4, on its way to passage_1_t5 its list moved to an llt, then the pick walked it to spot_32 and it died at 25:43
- 28:19 seat 0 constructor_19390 armmmkr at B2: U5, queued behind armmmkr_6066, which crawled under the metal stall and was still at 82% at the end
- 20:55 seat 0 constructor_10627 armadvsol at A1: U1, queued behind a Nettle at C3, then a 2300-long walk to A1; at 22:12 the pick walked it to spot_34 away from the A1 raid (corak, 2 corpyro)
- 14:58 seat 6 constructor_11595 armadvsol at H1: U3, queued behind its build in progress; a new list arrived and the hands dropped the old list's queued step when its turn came (the 2026-10-01 law)
- 15:43 seat 6 constructor_12278 armadvsol at H1: U3, the same: queued behind its build, dropped as stale when a new list arrived

## The three mechanisms

1. A neighbour's frame finishing takes a builder off its own (U9, 823 metal, three advanced solars: 0:armadvsol_28728 322, 5:armadvsol_15472 284, 6:armadvsol_14766 217): the hands' UnitFinished handler promotes the queued step of every builder whose started build is of the finished type within 250 of it, so when one of two adjacent advanced solars finished, the other builder's queued step began (a guard at once, or an armmakr that the blocked-site watchdog stopped 8 s later) and its frame was left at 62-92%; matching the finished unit to the builder's own frame (the id its UnitCreated gave) would have kept all three.
2. The exit-lane line answered with a leading 'stop' (U3, 704 metal, seven frames: 4:armnanotc_12832 168, 3:armsolar_15818 150, 5:armsolar_16263 135, 2:armadvsol_5052 130, 1:armsolar_20449 74, 4:armpb_21494 27, 4:armwin_144 20): a builder working from its plant's exit lane was reported as blocking it and the player re-listed it with 'stop' first; the stop landed 4-18 s after the report the player read, on frames that had grown meanwhile (11% to 97%, 30% to 87%, 56% to 73%); a rule that lets such a frame finish and then steps the builder out of the lane (or a stop that spares a frame past half) would have kept them.
3. One undefended home raid (U4, 780 metal, six frames over three seats: 7:armadvsol_30083 241, 7:armmoho_27594 155, 4:armmoho_2146 149, 0:armadvsol_20555 126, 0:armmoho_16900 99, 7:armrl_16876 10): party_645 (1 corak, 2 corpyro Fiends, tier 2) walked the north home row from A1 at 21:47 to D1 at 22:42 while every seat's army held the front; each seat's report carried it on the builder ('we cannot hit it ... it outclasses us: tier 2 against our tier 1') and the builders kept building; a home picket that can shoot tier-2 raiders (the players' or the commander's call) would have kept them.

Two report findings beside them: the 'abandoned an unfinished X (N% built)' line (hands' line and fights summary) gives the share at destruction, 0-2% for frames that decayed from 48-99%, and was said of 15 of the 16 frames destroyed by fire from out of sight while their builder stood on them (352 metal), so the player read a harness fault where there was an enemy it could not see; a builder's entry gives the share of the best same-type frame within 200 of its site, so it read 99% and 100% for two advanced solars of mechanism 1 that stood at 86% and 81%.

## Commands

- `cat MATCH/../results.jsonl; python3 -c "import json;print(json.load(open('MATCH/../batch.json')))"`
- `run/frames.py MATCH --seat N and --json, N = 0..7`
- `run/impedance_sweep.py MATCH --seat N --all, N = 0..7`
- `run/floor.py MATCH`
- run/hands_window.py on symlink dirs (scratch seatN with record-0/jev-0/strategist-0/terrain-0 -> seat N): seat0 19:44-19:52 constructor_30349; seat5 20:47-20:52 constructor_16338; seat3 2:33-2:40 commander_t3
- scratch scripts (tools/): scan.py (each frame's health, damage, builder distance, d rows, cmds, enemies from record-N), econ.py (store shares, income), knew.py (the builder's entry and enemy lines in the last report before the cause, the line after), move.py (a move's words and gate rating from jev-N), told.py and turnctx.py (report lines and the turn's queue call), unstarted.py (never-started orders against placed sites and frames of any builder)
- git show 8c770ab:crates/bot/src/brain/pianist/{mod,lists,execute,picture}.rs (UnitFinished promotion, the blocked-site watchdog, take_queued, the entry's percent)
