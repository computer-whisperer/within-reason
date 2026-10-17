# Impedance review: nuke-2-hard (2026-10-09)

Lost 32:22 to BARb hard_aggressive on Comet Catcher Remake 1.8. One seat (strategist-0), played by Opus 5.5 medium; there was no commander. The objective was the nuke. The hands ran the layers news,same,fuse,openers,split,tick,places,again,near. The game ran 170 turns with 4 session replacements, a think penalty of 1 capped at 7 s, and a truth file.

## Tally

| category | count | | category | count |
|---|---|---|---|---|
| A1 refused at the tool | 2 | | B1 in the hands' picture, not the report | 1 |
| A2 accepted, not executed | 10 | | B2 in record/truth, in nobody's picture | 0 |
| A3 overruled by the pick | 6 | | B3 report wrong/stale | 0 |
| A4 overruled by code | 4 | | B4 in the report, missed by Opus | 1 |
| A5 late | 0 | | B5 hands' lines misleading | 0 |
| A6 read differently | 2 | | B6 commander-to-player hop | 0 (no commander) |
| **C (clean moments)** | **4 of 8** | | | |

Denominators: 8 judgement moments, 353 orders, 1255 hands' lines (116 flagged). The sweep counts 1 refused (`fire`). A second order also never entered: a `signal` inside `orders` was answered "not a tool that can be batched", which the sweep does not count as refused (see A1, 29:53).

All 10 A2 items are `queue` steps that the builder could not make: 27 "this builder cannot build it" lines and 2 "it needs a place" lines, in 10 episodes. Each was Opus's own error, accepted at the tool and skipped silently a few seconds later.

## The ledger's reading, checked

The reading holds, with these refinements:
- **The silo and its rounds.** The silo stood at 16:32. Rounds came ready at 18:38.7, 20:38.7 and 22:38.8 (the record's `round` events), so one missile was ready from 18:39, two from 20:39 and three from 22:39.
- **His anti-nuke.** The truth file first shows his anti-nuke (corfmd) at 21:40, 9% built; it stood at 22:08.
- **The refused fire.** The player's 18:18 `fire` was refused 20 s before the first round, and the refusal was correct.
- **What Opus could see.** The player called `situation` at 17:41 (0 ready) and next at 23:39 (3 ready). None of the 41 reports from 16:43 to 23:39 contained the words silo, missile, stockpile or round. The hands' picture carried `stockpilers: "... 1 missile ready"` at 18:45, "2 missiles ready" at 20:45 and "3 missiles ready" at 22:45.
- **Opus's belief.** Its notes read "silo still 0 ready (energy-limited)" at 20:53 and "silo had 3 missiles ready (unseen in reports)" at 23:39. The silo was never energy-limited: the rounds came at the full 120 s rate.
- **Category.** This is B1, as the ledger says: the information was in the picture and in the `situation` read, but not in the report.

## Items ranked by cost

1. **18:39-23:39 · B1 · game · medium.**
   - Wanted: a missile on his base before his anti-nuke stood (16:33 note: "First missile ~18:40 if energy holds; target his advanced plant if found, else his corvp").
   - Happened: one round was ready from 18:39 and two from 20:39, but the report never said so. Opus fired nothing until 23:39, and the three missiles it then aimed at his base (23:47 x2, 25:46) were all intercepted. From 20:08 Opus pushed the army north "so the missile has a target", although it had named his corvp at G1 in its own 18:18 `fire`.
   - Fix (report): built after this game as the report's `missiles` line and the missile_ready wake.
2. **31:58-32:13 · A6 · game · low.**
   - Wanted: the commander's flee rule (31:55 packet: "...the corcat, corgol, corkarg, Tigers (correap), cormart, corban, corvroc and corshiva outrange its D-gun: while one is within 900 of it, it walks to home").
   - Happened: two unidentified radar contacts (corsoks per the truth file; corsok is not in Opus's list) closed on the commander from 31:58 while it walked to its own list's extractor at spot_71. The rule names types, so it never fired. Hit at 32:05 (60%), the commander was rated go_home 0.28-0.35, then closed by the news layer at 32:07-32:08. It walked only at 32:09 (19%) and died at 32:13; Opus's "walks to home now" landed at 32:13.
   - Low confidence: from 650 away at 60% health, it likely could not have outwalked two hovercraft whatever the hands did. Fix: words / menu, since the move told Jev "against this unit alone, we outweigh it" about two unknown contacts that were killing it.
3. **19:05-19:50 · A3 · small · medium.**
   - Wanted: the 18:58 packet, "the commander attacks and D-guns any corgator or corraid there"; the 19:25 packet made group_T1 a home guard.
   - Happened: the commander's attack_party_22 was rated 0.47 at 19:06 but not picked; the news layer closed it from 19:09 until 19:22. It attacked at 19:23, after the order was repeated. The armmoho at spot_68 (620 metal) died at 19:22. group_T1 followed group_D, 2,000 from home, through 19:48, although its home marks read 0.81-0.92.
   - Not charged: the fusion lost at 19:50 (3,350 metal); the commander was about 17 s away.
4. **14:09 · A3 · small · high.**
   - Wanted: "group_D sends eight soldiers ... the rest of group_D holds spot_52".
   - Happened: the pick sent three detachments of 8 (groups Y, Z and A1) in three seconds, taking 24 of 28 soldiers; the remnant walked to spot_61. The decode read the 8-detachment as allowed (0.10), as the packet wrote it. Fix: jev/menu.
5. **15:01-15:13 · A3 · small · medium.**
   - Wanted: the 14:46 packet, "group_D fights it whole, together, when it reaches ... spot_47".
   - Happened: the block stood at spot_47 from 15:01, but the whole-group attack was rated only 0.42 (and 0.38 at 15:10), so group_D held at spot_52 until Opus's explicit 15:06 order landed at 15:13. Two armcv (270 metal) died meanwhile.
6. **11:16 · A4 · small · high.**
   - Wanted: "A party of three or more ... is the business of group_D whole".
   - Happened: against 4 corgator, group_D's menu had no whole-group attack, only send_1/2/4/8, because WHOLE_GROUP_OVER = 4 withholds it from a body that outweighs the party four times. The pick sent 4 and then 8, and both hunts ended on the leash (11:25, 11:30). spot_26 was lost at 11:16 and rebuilt 674 s later.
7. **19:55-20:07 · A3 · small · high.**
   - Wanted: "one to three corraid or corgator ... gets four soldiers sent from group_D".
   - Happened: six detachments (17 soldiers) went after one corgator. Five hunts ended on the leash at 20:13, and the corgator raided spot_58 at 25:04. Opus spent its 20:08 turn re-forming the army.
8. **28:05-28:37 · A3 · small · medium.**
   - Wanted: "Aircraft (corfink, corca) are ignored by soldiers" (27:55 packet).
   - Happened: seven detachments hunted corca or corfink. The move's words call the plane "unarmed and cannot fight back" and never say the tanks cannot hit it; the forbidden reading was 0.57, under the mark's 0.8 threshold. The raid at spot_61 (2 corgol, corvroc) had a corfink hunt as its answer. The hunting stopped once the 28:30 AIRCRAFT RULE packet landed. Fix: menu.
9. **22:04-22:49 · A4 · small · medium.**
   - Wanted: attack the corpun and corllt at punisher_c3 (21:53 packet).
   - Happened: "advance to punisher_c3" was picked at 22:04, and the body then stood within about 100 of where it began for 45 s of the push window: the place was out of its reach. The mark's removal at 22:33 did not end the task. The not-nearer wake fired at 22:49.
10. **19:16 / 19:20 (and 14:11, 24:46, 24:59) · A4 · small · high.**
    - Wanted: detachments sent at raiders at home ("group_D sends four soldiers to them").
    - Happened: both hunts ended "after 0 s: the leash of 900 from where it began reached". The leash centre is the hunters' own centroid (`groups.rs hunt_of`), so a detachment spread more than 900 ends at birth. The same 0 s end recurs at 14:11, 24:46 and 24:59. Fix: code-rule.
11. **24:46-24:59 · A4 · small · medium.** The correap hunts the 24:46 packet ordered ("met by eight of its soldiers ... who kill them") ended on the leash, two of them after 0 s. The correaps then took spot_64 (25:13) and spot_38 (25:37).
12. **24:44 · A6 · small · medium.** "group_D goes (walks, without stopping to fight) to spot_26 now, every soldier" (a missile was landing beside it). The pick first detached 4 after party_40, and walked the body away only at 24:46. The decode had read the forbid on 4 at 0.48. Fix: words.
13. **24:49 / 24:53 · A3 · small · medium.** "The corfink scout plane is ignored." Home-guard detachments hunted the corfink twice (groups Q2 and R2).
14. **13:24-32:13 · B4 · small · high.** constructor_20491 was "stuck: it cannot move from home (B8)" in 119 reports and never built the converters Opus listed at 12:07; Opus never re-listed it or mentioned it. Fix (report): a stuck-builders count on the eco line, or a wake. constructor_2204 was stuck too (137 reports) but was helping the advanced plant build the whole time, so it is not counted.
15. **A2 (10 episodes), cost none to small · high.** Steps given to builders that cannot make them:
    - commander: armavp (7:28) and nanos (18:46; the advanced fusion's nanos waited 62 s);
    - T1 constructors: silo or fusion (11:45, 14:44, 17:19) and armafus (22:11). At 14:44, constructor_15278 fell to its `assist` step and helped the plant for over 100 s instead of the silo;
    - advanced constructors: nanos (20:41), Beamers and extractors (27:25-27:34; Opus's own 27:26 note says they cannot, yet the same turn listed them), and armcir (28:13);
    - silo steps without a place (11:39, 14:45);
    - 31:16: three extractors skipped, after which Opus listed the commander for five extractors and walked it east to spot_71, where it died (item 2).
    - Fix: the tool refuses at `queue` what the named builder cannot build, so these become A1 with an immediate answer.

A1, not ranked (cost none):
- 18:18 `fire`: "REFUSED: armsilo_282 has no missile ready (10 stockpiling, one every 120 s at full energy)". The refusal was correct; Opus never re-tried until 23:39.
- 29:53 `signal` inside `orders`: "signal: not a tool that can be batched". Opus re-issued it alone at 30:03.

Judged, not items:
- 5:05 leash ends: the quarry died to another hunt 4 s later.
- 10:04 leash end: the pick re-attacked with the same group 1 s later.
- 13:16, 14:11 (group_A1) and 25:11 leash ends: each ended a hunt the packet did not want.
- The 14 engine-refused sites: each list carried on, and no loss could be measured.
- 14:26: a turn ran 45 s of wall time and was abandoned, with a replacement session at 14:36. The block it might have answered was already covered by the packet in force.
- The `fire` tool answered "2 missiles left ready" to both 23:47 calls (the same-turn bug the ledger notes); Opus's own count was right.
- The 21:23 report still listed party_38 (corban + corgol) as recently seen after its 20:48 deaths; Opus had already noted them dead.

## Judgement moments (8)

- **(a) Opening, 0:00-4:00: C.** The commander list ran in order (mex 0:03, mex 0:13, solars 0:20 and 0:29, plant 0:43, guard 1:02, mex 2:03 and 2:14), and the produce list was made in its listed order. The 1:40 solar by constructor_18284 followed the packet's own rule ("When the energy line reads STALLING and metal is above 150, one solar"); the economy read "STALLING", metal 312. pro_baseline flags: 12 factory units by 4:00 (<p25), extractors 2 at minute 2 (<p25), 9 at minute 4 (>p75).
- **(b1) 11:16 spot_26 (party_27, 4 corgator): A4** (item 6).
- **(b2) 14:09 party_34: A3** (item 4). His block was out of sight (truth: 8 corlevlr and 3 corwolv at E5, 6 corraid at D5); the report honestly said "last seen 1:01 ago at (G4)". Fog, not an item.
- **(b3) 14:43-15:14, the corlevlr/corraid block at spot_58/56/47: A3** for 15:01-15:13 (item 5). Holding at 14:44-14:54 followed the packet ("never chases it east past spot_47"; the block was at spot_49).
- **(b4) 19:05-19:50, the home raid: A4 and A3** (items 10 and 3).
- **(c) First window at 1.5x (2:00-3:00), "Rovers pressing his half": C.** The group roved from 1:08 and found his corvp at G1 at 2:08; one Rover was lost to a corllt (31 metal).
- **(d1) The 10:40 packet (group_D posts at spot_52, constructors' spot order): C.** Decode group_D|place|spot_52 read 0.98; group_D held spot_52; the lists took spot_26, spot_43 and spot_47; the Rovers raided cormex at F4/F5.
- **(d2) The 21:23 packet (attack the D4 turrets, then push north): C.** The decode marks landed (spot_33 0.96, spot_24 0.97, ...); the turrets at D4 were dead by 21:33, and the body continued toward spot_24.

The fixed missing-context check:
- At 10:40, 14:16, 19:15, 21:23 and 32:05, the report's enemy lines matched the truth for everything our units could see. The rest was fog: we had no radar from start to 28:05 (radars 0 by 8:00, <p25).
- At the 20:08 push decision, the report said "7 worth 1415 metal ... may be much more". The truth had a corban, a corgol, a correap, 5 corraid and 2 corwolv within 1,500 of group_D, all unseen.
- For the 14:16, 19:15 and 32:05 turns, the "what your hands did" block matched the jev plan rows; no play was found listed and then replaced.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 170 turns, 4 session replacements, 353 orders, 1 refused, 116 flagged of 1255 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 169, 'queue': 65, 'instruct': 64, 'produce': 31, 'fire': 8, 'lane': 5, 'mark': 4, 'signal': 4, 'situation': 3}
   refused: {'fire': 1}
   flagged: {'step-skipped': 45, 'hunt-ended-leash': 20, 'hunt-ended-other': 18, 'engine-refused': 14, 'step-waits': 7, 'other-fault': 5, 'stopped-on-stop': 3, 'abandoned': 2, 'follow-ended': 1, 'site-blocked': 1}
```

## What Opus wanted that the harness has no lever for

- **10:40, in force at 11:16:** "A party of three or more ... is the business of group_D whole". When the body outweighs the party four times, the menu withholds the whole-group attack (WHOLE_GROUP_OVER) and offers only detachments.
- **14:01:** "group_D sends eight soldiers ... the rest of group_D holds spot_52". The forbidden mark works per detachment size; nothing caps how many detachments go at one party.
- **18:58 / 19:15:** "group_D sends four [eight] soldiers" at raiders at home, about 2,400 from the army's post. The 900 leash, measured from the hunters' own centre, ends such a hunt at birth; no move sends a detachment across the map.
- **24:36 / 25:06 / 27:55 / 28:30:** "The corfink scout plane is ignored" ... "AIRCRAFT RULE: No group ever sends, hunts or attacks a corfink or a corca". No mark forbids a kind of quarry, and the menu offers ground soldiers hunts of aircraft, worded "it is unarmed and cannot fight back". It took the 28:30 packet's wording to stop them.

## Commands run

```
cat run/matches/1791577775-nuke-2-hard/results.jsonl; python3 -c "...json.load(open('.../batch.json'))"
python3 run/impedance_sweep.py run/matches/1791577775-nuke-2-hard/00 --counts
python3 run/impedance_sweep.py run/matches/1791577775-nuke-2-hard/00 --seat 0
python3 run/impedance_sweep.py run/matches/1791577775-nuke-2-hard/00 --seat 0 --all --from 10:40 --to 11:30
python3 run/minutes.py run/matches/1791577775-nuke-2-hard/00
python3 run/wake_read.py run/matches/1791577775-nuke-2-hard/00
python3 run/raid_ledger.py run/matches/1791577775-nuke-2-hard/00
python3 run/pro_baseline.py run/matches/1791577775-nuke-2-hard/00 [--all-starts; the plain call finds no carded side]
python3 run/commander_turns.py run/matches/1791577775-nuke-2-hard/00 --calls / --notes
python3 run/hands_window.py MATCH 1:10 2:15 constructor_10476 constructor_18284 | 11:08 11:40 group_D | 11:33 12:10 group_D | 14:06 14:30 | 14:40 15:16 group_D | 19:05 19:58 | 19:38 19:53 commander group_Q1 group_T1 | 21:28 22:00 group_D | 21:56 22:50 group_D | 24:42 25:03 | 9:50 10:06 | 31:58 32:14 commander
python3 (jevlog.rows): decode rows extracted; pass menus and gate flags at 11:15-11:18; call questions/answers at 14:44-15:14, 19:39-19:50, 28:15; call state stockpilers at 18:45/20:45/22:45; commander/15278 'doing' 14:51-20:30; plan rows for party_45 and for aircraft hunts
python3 (record-0.jsonl): created/finished 0-4 min; `round` and fire_at rows; destroyed 19:20-20:05; enemy_seen 14:00-14:30; commander track and corsok truth 31:15-32:13
python3 (truth-0.jsonl): his corfmd/coravp/corgant/corfus/cormabm first and complete; his mobile units by cell at 10:40, 14:16, 20:08, 21:23 and within 1,500 of group_D
python3 (strategist-0.jsonl): prompts 16:00-24:00 searched for silo/missile/stockpil/round; situation and fire results; stuck lines counted; all 64 packets extracted
read: crates/bot/src/brain/pianist/menu.rs (WHOLE_GROUP_OVER), groups.rs (hunt_of, HUNT_LEASH), decode.rs (questions), docs/code-decisions.md, docs/experiments.md (nuke-2-hard row)
```
