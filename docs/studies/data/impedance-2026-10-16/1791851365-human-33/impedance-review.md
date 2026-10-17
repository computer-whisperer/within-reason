# Impedance review: 1791851365-human-33 (human-33)

Gecko Isle Remake v1.2.1, 2026-10-12, realtime: four Opus 5.5 medium seats (t2 Cortex D7, t3 Cortex F8, t4 Armada B7, t5 Armada G8) under an Opus 5.5 commander, hands on Jev 1.13.0, against [gecko]thebluegecko (Cortex) and [gecko]u6bkep (Armada). **Lost:** the commander surrendered at 22:45. At 10:00 the side led about 3 to 1 in army, but his tier 2 (Tzars from 13:00, bombers at 19:14, about 37 Wasps at 22:00) took apart the bodies and the homes. There is no truth file.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 6 |
| A2 accepted, not executed | 5 |
| A3 overruled by the pick | 1 |
| A4 overruled by code | 5 |
| A5 late | 1 |
| A6 read differently | 5 |
| B1 in the picture, not in the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 in the report, wrong | 2 |
| B4 in the report, missed (Opus's miss) | 0 |
| B5 the hands' own lines misleading | 2 |
| B6 commander-to-player hop | 1 |
| C (clean judgement moments) | 2 of 12 |

Denominators: 12 judgement moments (3 per seat). There were 820 orders: 177 + 202 + 230 + 211 from the players and 43 from the commander. The reports carried 2,569 hands' lines (578 + 658 + 811 + 522), of which 278 were flagged. The tools refused 6 orders. The sweep printed 4 refusals, because it misses two `signal` calls refused as "not a tool that can be batched". Cost: four items are large. Three of them (A6 12:41, A2 13:23, B5 13:45) are one chain, about 2,600 metal of t2's tier-2 money, not three separate losses.

Judgement moments, with the brief's letter:
- **t2:**
  - (c) 4:47 go_north push: the body went north whole. A1 (the signal was refused; the packet stood without it).
  - (d) 12:22 #avp1: A6, A2 and B5.
  - (e) 21:59-22:48 anti-air at power_field and avp_yard: the picks followed the packets. A1 at 22:30.
- **t3:**
  - (d) 7:54 Grunt fight and the lone-Rascal scout: no lever.
  - (d) 14:44 Tzar swarm: A6.
  - (e) 22:07 commander under the sea: A4, cost none.
- **t4:**
  - (a) opening: **C**. It had 3 extractors and 3 solars by 0:56, the plant at 1:20, 3 tidals at 2:51-3:09, and 9 extractors by 4:00.
  - (d) 17:11 pull back to west_stage: **C**. The decode read west_stage at 0.98 at 17:22; the pick shelled party_132 at 17:22 (allowed) and walked to west_stage at 17:24.
  - (b) 18:49-20:01 the home-raid clause: A6.
- **t5:**
  - (a) opening: A4 (the plant site was moved 418 north).
  - (d) 12:32 the Rover: A6.
  - (e) 22:01-22:37 commander to sea_hide: A4.

The people's chat speaks to strategy, not to any harness moment, so no item rests on it. thebluegecko said at 21:09 "too many T1 arty units". At 22:11-22:19 he said "it isn't just unit choice, it is eco ... I have more money, so I have more units". That last line goes against the side's count of his extractors (20 to 24 known against our 48). Without a truth file, this review cannot measure that gap. The engine refused our extractors at spots holding his unseen extractors (the two B3 items). That is the only measured sign that the count ran low.

## Items ranked by cost (15 of 28; the rest are listed below)

1. **12:41 t2 A6 (large).**
   - Wanted: `coravp avp_yard #avp1` on four builders (12:22), with the commander's "no second one anywhere".
   - Happened: "12:41 constructor_26284: build a coravp at avp_yard (E7) (#avp1)" and "12:41 constructor_6718: build ... (#avp1)" in the same second. The host placed them at (4296,7496) and (4504,7496). Two frames went up, plant_27644 and plant_27021, and both stood (16:47, 16:54), each costing 2,600 metal.
   - Fix (code-rule): one id gives one frame. This was fixed after the game in 4ed68d8.
2. **13:23 t2 A2 (large).**
   - Wanted: `remove {"by": "constructor_6718", "reclaim": ["plant_27021"]}`. The tool accepted it.
   - Happened: "13:39 constructor_6718 takes plant_27021 apart next, at the front of its list", then "stopped ... on your `stop`", then "13:40 ... help plant_27644 build (from its list)". The record has no reclaim command. The #avp1 steps then repaired plant_27021 (13:40, 14:02), and the frame went from 2% to 100%.
   - Fix (code-rule): when a `stop` step and a remove arrive in one turn, the remove must survive.
3. **13:45-17:18 t2 B5 (large).**
   - Opus's notes kept "the advanced vehicle plant at avp_yard (10%)", "31%" and "62%": it believed in one plant.
   - No report between 13:45 and 17:18 named plant_27021. The only trace was "14:02 constructor_25847: help build #avp1 (repair)": the id, not the frame Opus had ordered taken apart. Opus found the second plant at 17:18.
   - Fix (report): a builder's line should name the frame it works on, and every frame under construction should be listed.
4. **13:23 t3 A5 (large, medium confidence).**
   - Wanted: "The east body regroups at spot_49", in a 21-s turn.
   - Happened: the packet reached the hands at 13:39. Under the 13:07 packet the pick played "13:27 group_J_t3: attack party_109 (1 corgol) with the whole group". The 13:44 report said: "while your orders were on their way (18 s) we lost 7 units worth 1250 metal".
   - Fix: unknown (the realtime turn length).
5. **12:32 t5 A6 (small).**
   - Wanted: produce `{"group": "new", "units": ["armfav:1", ...]}` and "A group that is a single Rover ... walks (go) to spot_37, then his_corner, then spot_12 ... never joins another group". This was the commander's fourth ask for a look at H1-H2.
   - Happened: the list's next soldiers joined the Rover's group_Q_t5 (1 soldier at 13:11, 5 by 13:49). "14:06 group_Q_t5: walk to spot_46", then "14:15 group_Q_t5: join group_L_t5". The Rover walked again only after the fifth ask (decode at 15:43) and died at G4 at 16:37. H1-H2 was never seen in the game.
   - Fix (tool): a produce entry should be able to carry its own group.
6. **10:56 t3 A4 (small).**
   - Wanted: the 10:30 packet's route for "group_C_t3 (Pounders and Lashers, the east body)".
   - Happened: "8 of group_C_t3 hunt party_92 as group_I_t3" ended after 0 s on the 900 leash; then "group_C_t3: join group_I_t3". The same happened again at 10:59 and 11:01, making group_J_t3. The body was renamed twice in 6 s, and the packet's name pointed at nothing until the 11:03 orders.
   - Fix (code-rule): a body should not join its own detachment, and the menu should not offer a hunt the leash ends at once.
7. **12:35 t4 A3 (small).**
   - Wanted: "group_E_t4 attacks party_101 ... now with the whole group ... it does not walk away from it".
   - Happened: the picks sent "8 of group_E_t4 hunt party_101 as group_W_t4", then X (4), Y (4) and Z (2), and every hunt "ended after 0 s: the leash of 900". The decode read the detachment as forbidden at only 0.53-0.54, under the 0.8 mark bar, so nothing stopped the split. At 12:41 Opus had to rejoin W, Y and Z.
   - Fix (jev): the decode's reading of "with the whole group" as forbidding a detachment.
8. **19:47 t4 A6 (small).**
   - Wanted: "A party of his of three or more at our home ... is met by eight soldiers sent from the group's tail".
   - Happened: the pick applied this to five bombers. "19:47 group_E_t4: 8 of group_E_t4 hunt party_175 (5 corshad) as group_K1_t4" was followed by L1, M1 and N1 by 19:57, and every hunt ended on the leash at 0 s. Opus forbade chasing bombers at 20:01.
   - Fix (menu): no ground hunt should be offered at aircraft.
9. **20:50 t4 A4 (small).**
   - Wanted: "it sends eight soldiers from its tail nearest home to attack them [the Caymans] there".
   - Happened: "group_P1_t4's hunt of party_189 ended after 0 s: the leash of 900" (about 2,000 to the quarry). Then "20:51 group_P1_t4: follow group_E_t4", back north. It attacked only at 21:00. The Caymans killed 3 winds, a Whistler, a nano and a Nettle by 21:00, and the plant at 21:26.
   - Fix (code-rule): a leash measured from where the hunt starts cannot answer a raid at home.
10. **17:23 t4 A2 (small).**
    - Wanted: `armrad spot_29`. The note: "The west push is off until we can see".
    - Happened: the list step and the pick alternated: "build a armrad at spot_29", then "walk to spot_44 (plan)". There were three "abandoned an unfinished armrad at D3", and a fourth frame there was killed by the Tzar at 18:10. The spot_29 radar never stood; the spot_38 radar did, at 18:02.
    - Fix (code-rule): a builder should not restart and drop a frame every few seconds.
11. **16:05 t4 A2 (small).**
    - Wanted: `queue constructor_17613 ["armrl", "armrl spot_29"]`. The tool accepted a turret with no place.
    - Happened: "skipped the step 'armrl' of its list: it needs a place". The second step thrashed under the Tzar (abandoned at 17:56). t4's first anti-air turret stood at 19:44, and t4 lost 1,011 metal to bombers.
    - Fix (tool): refuse a placed building that has no place.
12. **6:00-16:00 t4 B6 (small).**
    - The commander's plan said "From 6:00 one anti-air turret at each plant and by the commander, because this player has used gunships" from 0:06 through 10:14.
    - t4's player ordered none until 16:00. The other seats stood their first anti-air turrets at 6:44 (t2), 7:17 (t5) and 11:07 (t3).
    - Fix (brief): the direction's standing items should stay in the player's checklist.
13. **10:54-12:08 t5 B3 (small, medium confidence).**
    - The report listed "#37 H4" as a free spot and "nothing of his ... H4 (8:15 ago)". Meanwhile the engine refused our extractor there at 6:03, 11:50 and 12:08 ("the site was bad", then "wrecks or a unit on it"). His extractor stood there unseen until 12:43, and Opus queued it three more times.
    - t3 had the same at 10:57: six refusals at F4, G4 and H4, which Opus called "free-looking spots". That is a separate B3 item.
    - Fix (report): after a refusal, say that a spot is "refused: likely his extractor" and take it out of the free count.
14. **6:33 t4 B5 (small, medium confidence).**
    - Wanted: four tidals at sea_home during an energy stall (thebluegecko at 0:12: "tidals are strong here").
    - The report said "walking to build a Tidal Generator ... 660 to go", then "692 to go": the straight-line distance, which grows along the detour. Opus: "never arriving; switched to plain solars". At 6:36 the commander was at the route's turning point; the same route had reached the site in 50 s at 2:40.
    - Fix (report): give the route's time to go.
15. **15:03 t3 A6 (small, medium confidence).**
    - Wanted: "attacks his Tzar ... until it is dead. ... after the Tzar is dead, ... From a party ... that outweighs it, it steps back to spot_54".
    - Happened: "15:03 group_J_t3: walk to spot_54", then the attack again at 15:05. The decode read back at 0.92 without the sentence's condition. Opus took this as "hands thrash between attack and step-back" and called the swarm off.
    - Fix (words): the condition should be written into the sentence itself.

Also counted:
- **A4, small:** 0:47 t5. The host moved the plant site 418 north; the commander walked a 30-s detour, and the plant stood at 1:38, last of the four seats.
- **A4, small:** 22:27 t5. The commander's old list step "help factory_8787 build" took it back under the Wasps for 3 s.
- **A4, none:** 22:19 t3. The same with "guard plant_25905", for 1 s.
- **A6, small:** 8:56 t2. #lab1 had the same race as #avp1: a second lab was tried, and only "9:54 the engine refused a corlab at E8" stopped it.
- **A2, small:** 12:14-12:50 t4. Five "abandoned an unfinished armllt at C4": the list step and the walk to spot_44 alternated.
- **A2, none:** 5:59 t2. "skipped the step 'corrl' ...: it needs a place". Opus re-issued it at 6:12.
- **A1, none:**
  - 4:47 t2: signal go_north, "not a tool that can be batched". The packet stood alone.
  - 13:06 t4: signal west_go 13:40, refused the same way. Opus re-signalled alone at 13:21.
  - 12:00 t2: produce into group_A_t2. The commander had given that group to t4 at 12:06, inside the player's turn.
  - 15:05 t5: produce into group_L_t5, which the commander had moved at 15:07.
  - 15:32 t5: transfer of group_T_t5, which had merged during the turn.
  - 22:30 t2: produce on plant_27644, which died at 22:40, during the turn.

  Opus re-issued every one that still mattered within one turn. The four produce and transfer refusals were the commander's transfers landing mid-turn, or a group merging or a plant dying during a realtime turn.

## Sweep counts per seat (verbatim, `run/impedance_sweep.py MATCH --counts`)

```
-- strategist-0.jsonl: 76 turns, 1 session replacements, 177 orders, 2 refused, 65 flagged of 578 hands' lines; landing median 0 s (p90 0), turn wall median 15 s
   orders: {'wait': 76, 'queue': 39, 'instruct': 30, 'produce': 21, 'mark': 3, 'transfer': 3, 'lane': 2, 'remove': 2, 'signal': 1}
   refused: {'produce': 2}
   flagged: {'step-skipped': 19, 'stopped-on-stop': 16, 'engine-refused': 10, 'other-fault': 7, 'step-waits': 7, 'abandoned': 2, 'follow-ended': 2, 'hunt-ended-other': 1, 'hunt-ended-leash': 1}
-- strategist-1.jsonl: 87 turns, 2 session replacements, 202 orders, 0 refused, 52 flagged of 658 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 86, 'instruct': 47, 'queue': 37, 'produce': 17, 'mark': 6, 'lane': 6, 'transfer': 3}
   flagged: {'step-skipped': 15, 'stopped-on-stop': 11, 'engine-refused': 9, 'other-fault': 7, 'step-waits': 6, 'hunt-ended-leash': 2, 'abandoned': 1, 'site-blocked': 1}
-- strategist-2.jsonl: 103 turns, 2 session replacements, 230 orders, 0 refused, 87 flagged of 811 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 103, 'instruct': 52, 'queue': 41, 'produce': 25, 'lane': 6, 'signal': 2, 'mark': 1}
   flagged: {'step-skipped': 21, 'step-waits': 21, 'abandoned': 15, 'hunt-ended-leash': 11, 'stopped-on-stop': 10, 'hunt-ended-other': 4, 'engine-refused': 3, 'site-blocked': 1, 'other-fault': 1}
-- strategist-3.jsonl: 90 turns, 2 session replacements, 211 orders, 2 refused, 74 flagged of 522 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 89, 'instruct': 46, 'queue': 39, 'produce': 25, 'lane': 5, 'transfer': 4, 'mark': 3}
   refused: {'produce': 1, 'transfer': 1}
   flagged: {'step-skipped': 33, 'engine-refused': 13, 'stopped-on-stop': 10, 'step-waits': 9, 'other-fault': 7, 'hunt-ended-other': 1, 'abandoned': 1}
-- commander.jsonl: 23 turns, 0 session replacements, 43 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 22 s
   orders: {'direct': 20, 'say': 10, 'transfer': 6, 'units': 3, 'mark': 2, 'wait': 1, 'surrender': 1}
   flagged: {}
```

Notes on the flagged lines:
- **The leash ends at once.** Of the 14 `hunt-ended-leash` lines, 12 ended "after 0 s". The menu offers a hunt whose quarry is already past the leash. That happened at t2 14:41, t3 10:55 and 10:59, and t4 12:35-12:38, 19:47-19:57 and 20:50. The other two, t4 at 2:49 and 2:51 (after 10 s and 4 s), were lone Blitzes chasing raiders 4,000-5,000 away, which no packet wanted dead.
- **Engine refusals.** Most of the 35 `engine-refused` lines on extractors are spots an ally had just taken. The ones that mattered are the two B3 items.
- **Step-skipped lines.** Most of the `step-skipped` lines are lists naming spots already taken ("being taken by a builder of another seat", "holds our extractor already"). They are harmless.

## What Opus wanted that the harness has no lever for

- **7:54 t3** (the commander's asks at 7:14, 8:44, 10:14, 11:44 and 14:44): "one Rascal is sent alone to spot_12 and then h1_look ... The rest rove".
  - A detachment move exists only at an enemy party. The group's walk to spot_12 moves all five, and it stood held for its opener. Rove takes no destination.
  - The player, at 10:30: "Could not send one Rascal to a named spot: rove has no destination, send needs a party." His base corner H1-H2 was never seen in 22 minutes.
- **10:37 t5** (repeated at 11:26, 11:41, 11:55 and 12:13): "Its Rover (armfav) leaves now as rove_1 up the east edge to spot_37 and spot_12; leave it to rove."
  - Rove has no destination. The rover kept to our own half (the commander at 14:44: "Your Rover group_R is roving in our own half").
  - The only other way is a group of one, and a produce list's `new` group gathers every unit of the list (item 5).
- **18:49, 19:02, 19:37 and 20:38 t4:** "eight soldiers sent from the group's tail nearest home" to a raid at home, 2,000-3,000 from the body.
  - The only detachment is a hunt leashed at 900 from where it begins, so every one ended in its first second.
  - No move walks part of a group to a place and fights there.

## Commands run

All from the repository root, with MATCH = run/matches/1791851365-human-33/00 and SEATn = the scratch directories of seats 1-3: `/tmp/claude-1000/-home-christian-workspace-playground-bar-bots/93f9e8f3-fea1-482e-a1c2-cb750e01d6e2/scratchpad/impedance/agents/1791851365-human-33/seat1`, `.../seat2`, `.../seat3`. Each holds symlinks record-0, jev-0, strategist-0 and terrain-0 to that seat's files, plus script.txt, commander.jsonl and bot.log.

- `grep -n 'name=' MATCH/script.txt`
- `tail -c 600 MATCH/record-0.jsonl`
- `grep '"k":"chat"' MATCH/record-0.jsonl` (30 lines)
- `run/impedance_sweep.py MATCH --counts`
- `run/impedance_sweep.py MATCH --seat 0|1|2|3`, and `--commander`
- `run/impedance_sweep.py MATCH --seat N --all --from m:ss --to m:ss` (many windows)
- `run/minutes.py MATCH` and `run/minutes.py SEATn`
- `run/raid_ledger.py MATCH` and `run/raid_ledger.py SEATn`
- `run/pro_baseline.py MATCH` and `run/pro_baseline.py SEATn` (no carded side on this map)
- `run/hands_window.py`:
  - `MATCH 5:10 5:50 group_A_t2`
  - `SEAT1 8:10 9:10 group_A_t3`
  - `SEAT1 14:55 15:10 group_J_t3`
  - `SEAT2 17:12 17:50 group_E_t4`
  - `SEAT2 19:44 20:00 group_E_t4 group_K1_t4 group_L1_t4 group_M1_t4 group_N1_t4`
  - `SEAT2 20:47 21:12 group_P1_t4`
  - `SEAT3 13:08 14:20` and `14:20 15:45`, both `group_Q_t5 group_R_t5`

Scratch scripts (in the scratch directory's `scripts/`):
- `ev.py`: record events and commands by type, unit or window.
- `turn.py`: one turn's calls, results and prompt lines.
- `decode.py`: decode rows of a jev log, filtered.
- `packet_at.py`: the packet in force at a clock.
- `refused_spots.py`: each engine-refused extractor against enemy sightings at its spot.
- Inline Python over the records and jev logs for:
  - the AVP frames' progress;
  - the commanders' positions;
  - the transfer (`taken`) events;
  - anti-air turrets per seat;
  - losses to his aircraft;
  - same-second placements of one type at two sites;
  - instruction landing frames.
