# Impedance review: 1791751159-human-32

Human game, 2026-10-11, Flats and Forests v2.2, realtime. Three Opus 5.5 medium seats (t2 = strategist-0 middle, t3 = strategist-1 south, t4 = strategist-2 north) under an Opus 5.5 commander, against thebluegecko (Cortex, lobby skill 34) and u6bkep (Armada, 19). LOST: the commander conceded at 27:20, and the record ends at 27:35. The chat said "the Shiva switch at 21 min broke us", and a person answered "gg wp".

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 2 |
| A2 accepted, not executed | 2 |
| A3 overruled by the pick | 7 |
| A4 overruled by code | 1 |
| A5 late | 0 |
| A6 read differently | 2 |
| B1 in the hands' picture, not in the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 in the report, wrong or stale | 1 |
| B4 in the report, missed by Opus | 0 |
| B5 the hands' own lines misleading | 0 |
| B6 commander-to-player hop | 2 |
| **C followed, with context enough** | **1 of 12 moments** |

Denominators:
- Judgement moments: 12 (4 a seat), of which 1 was clean.
- Orders: 249 / 271 / 239 (t2 / t3 / t4), plus 36 from the commander.
- Hands' lines: 876 / 939 / 972.
- Refused: 0 / 1 / 1, and 0 from the commander.
- Takeovers: 2 per seat.

The seat-0 tools (minutes, raid_ledger, pro_baseline, hands_window) were run on symlink directories for seats 1 and 2 (`scratchpad/impedance/agents/1791751159-human-32/seat1`, `seat2`).

## The twelve judgement moments

| # | seat | moment | verdict |
|---|---|---|---|
| 1 | t2 | (a) opening 0:00-4:00: 3 mex, 3 solar, plant 1:24, produce order made exactly | A2 small (turret at spot_77 skipped) |
| 2 | t2 | (b) Rover raid 4:54-5:53 (18 armfav, 6 extractors lost) | B6 small (packet named groups transferred away at 5:01) |
| 3 | t2 | (c) go_west push 8:01-11:03, group_P_t2 along spot_99, blitz_g6, 139, 154, 157, 156 as written | A3 small (4 detached against "as one body") |
| 4 | t2 | (d) packet changes 16:10/17:09/17:24, W1 at the F4 nest | A3 large |
| 5 | t3 | (a) opening: 4 mex, 4 wind, plant 1:38 | A6 small (produce counts restarted) |
| 6 | t3 | (b) Rover raid 3:05-3:58 (all 5 extractors, 3 constructors) | A1 none, otherwise as wanted |
| 7 | t3 | (c) first push 10:50-11:31, t2's body given to t3 mid-fight | B6 large |
| 8 | t3 | (d) packet 18:40 (landed 18:57) and 19:53: two bodies kill green's party, join, route | **C** |
| 9 | t4 | (a) opening: 4 mex, solar, wind, plant 1:26; commander penned 3:12 | A1 large |
| 10 | t4 | (b) party_81 raid on H1-H3 13:26-14:33 (answer "none" on 5 spots) | A3 small, A3 small, A4 small |
| 11 | t4 | (c) go_west at 7:45, group_O_t4 west along the north | A3 large |
| 12 | t4 | (d) commander's meet_south order, packet landed 16:01 | A3 small |

Four further items come from checking the players' notes against the logs: t2 at 21:34, t3 at 21:53 and 22:20, and t4 at 25:05.

## Items, costliest first

1. **3:27, t4, A1, large, high.**
   - Wanted: `remove {"by":"commander_t4","reclaim":["corwin"]}`. The note read "commander_t4 stuck at spot_64 among plant/wind turbines ... Wish: picture handles for our buildings".
   - Happened: the tool answered `corwin: nothing of ours has that name (handles are <unit>_<id> ...)`. No t4 report from 3:00 to 23:00 carries a handle for any of our buildings, so Opus never re-issued the order.
   - Cost: the commander's moves failed from 3:12 to 4:13, and it then stood within about 130 elmos of one point until 20:30. It was freed only by reclaiming plant_2006, the seat's only factory, at 22:29. It died at 25:42.
   - Fix: a report line giving our buildings' handles, or a remove that takes a place.
2. **17:14, t2, A3, large, medium.**
   - Wanted (16:10 packet): "group_W1_t2 ... afterwards returns to spot_80 and holds there ... never goes toward his turret nest at spot_93, spot_92 and spot_91". The decode read spot_80 at 0.98.
   - Happened: `PLAYS group_W1_t2: advance to spot_87 [plan]` at 17:14 and 17:17. The fight-move ran the body into the F4/E4 nest ("met 2 cordrag, 2 corexp, 3 corhllt ... reached spot_93"). The body went from 35 soldiers at 17:13 to 24 at 17:25, then thrashed between shelling and spot_80 until 17:38.
   - Opus's note blamed the 'never' sentence for putting the nest on the menu. The nest spots were on the menu (0.96), but the pick never played a move to them.
   - Fix: jev.
3. **8:38, t4, A3, large, high.**
   - Wanted (8:24 packet): "go_west is given ... fights its way west as one body: spot_24, spot_6 ... It does not chase small parties south or east of its way".
   - Happened: the pick played "8 of group_O_t4 hunt party_47 (1 corgator)" at 8:38, then "attack party_51 (1 corgator)" at 8:43 and "attack party_49" at 8:45. This continued a chase that began at 7:59 (attack_party_47 at 0.61 against fight_to_spot_24 at 0.31).
   - Cost: O went from 17 soldiers to 12 by 8:34, then to 4 once 8 had split off. Its first stop was reached at 9:45, two minutes after go_west.
   - Fix: menu. Whole-group attack moves carry no `.forbidden` reading.
4. **25:05, t4, A3, large, medium.**
   - Wanted (24:48 packet, landed 25:04): "The commander walks south along the east edge now ... never stops to build or fight on the way".
   - Happened: `PLAYS commander_t4: attack party_202 [plan]`. The menu called the radar Shiva "1 unidentified ... we outweigh it heavily". The commander stood at (11482,1705) from 25:00 until it died at 25:42, and t4 was eliminated.
   - Fix: jev, and the report's odds words for an unidentified contact.
5. **22:20, t3, A2, large, high.**
   - Wanted: `remove plant_6196` (the duplicate advanced-plant frame), issued at 22:20 and 22:35, and by another builder at 22:50.
   - Happened: every remove was accepted. The hands' lines read "22:32 constructor_2000 takes plant_6196 apart next" and then "22:33 constructor_2000: help plant_6196 build (from its list)".
   - Cost: the frame was never reclaimed. It went from 9% to 38% by 24:00, about 750 metal sunk, and was destroyed unfinished at 26:30.
   - Fix: code-rule.
6. **11:03, t3, B6, large, medium.**
   - Wanted: the commander gave t2's southern body to t3 at 10:50 "so one player and one pair of hands strike green".
   - Happened: the transfer landed at 11:03 with the body already fighting. t3's picks played "group_M_t3: attack party_73" alone at 11:04 and 11:11. t3's packet naming the group landed at 11:27, when 2 of its 8 soldiers were left.
   - Fix: tool. A transfer should arrive with the receiving packet's clause, or hold until it does.
7. **21:34, t2, B3, large, low.**
   - The report read: "party_151 (8 corgator, 1 unidentified) ... outweighs it heavily (10.9 to 1); under_fire ... by corgator". The unidentified contact was a corshiva that the 21:03 wake had named.
   - Opus kept W1 attacking. W1 went from 35 soldiers (6,645 metal) at 21:34 to 7 at 21:51, with 4 Shivas seen between 21:36 and 21:46.
   - Fix: report.
8. **21:53, t3, A6, small, high.**
   - Wanted: six builders on "armavp avp_yard #avp1".
   - Happened: two frames were started from the same id, plant_6196 at 21:53 and plant_13221 at 22:01. The known same-id race was fixed after this game.
   - Fix: code-rule.
9. **4:57, t2, B6, small, high.**
   - Wanted (4:57 raid packet): "group_G_t2, group_H_t2 and group_I_t2 all join each other and attack the armfav".
   - Happened: the commander's 4:50 transfer moved G and H to t3 at 5:01, before the packet landed. The 4:57 report still read "direction of 3:35 stands".
   - Fix: tool.
10. **13:34, t4, A3, small, medium.**
    - Wanted (packet landed 13:33): "group_P1_t4, X1, Z1 attack party_81 ... now".
    - Happened: P1 kept engaging party_91 until 13:47. attack_party_81 was its top move, but only at 0.43-0.49, and the news layer closed the group from 13:35 to 13:41.
    - Cost: spot_20, spot_15, spot_13 and spot_22 were lost to party_81 in that window.
    - Fix: jev.
11. **14:23, t4, A4, small, high.**
    - Wanted (14:11 packet): "it sends eight soldiers at party_81 ... now".
    - Happened: "group_B2_t4's hunt of party_81 ended after 11 s: the leash of 900 ... reached". D2's hunt ended the same way at 14:29, one second after party_81 killed spot_60.
    - Fix: code-rule (the leash).
12. **14:16, t4, A3, small, medium.**
    - Wanted (same packet): two detachments of eight, and "The rest of the body stands at spot_37".
    - Happened: the pick made six more detachments after single corgators: party_98, party_99 (three times) and party_100 (twice). Opus's 14:33 note: "army had shattered into ten detachments".
    - Fix: jev.
13. **16:01, t4, A3, small, high.**
    - Wanted (packet landed 16:01): "goes south now to meet_south ... does not turn back north for blue's corgator".
    - Happened: P1 kept shelling party_106 to the north until 16:19, 1,200 elmos the wrong way. The change was rated 0.71 and fight_to_spot_139 0.54 at 16:01, but neither was taken.
    - Fix: jev.
14. **8:16, t2, A3, small, high.**
    - Wanted (8:01 packet): "fights its way west as one body".
    - Happened: `4 of group_P_t2 hunt party_47 (2 corgator) as group_U_t2`, with send_4_party_47.forbidden at 0.66, below the 0.7 bar at which the menu line says so. The decode read detach|4 at 0.71 from that packet.
    - Fix: jev.
15. **3:01, t3, A6, small, high.**
    - Wanted: produce `{"group": "new"}` at 2:47, meant only to open a new group.
    - Happened: the call restarted the counts of the 1:28 list. The plant ordered armcv twice more (3:01, 3:09) in place of the Rovers the 3:05 note asked for. The 3:05 report read "armcv (2 more allowed)" at the list's head.
    - Fix: tool.

Not ranked:
- **4:02, t2, A2, small.** The turret step at spot_77 was skipped with "the extractor at spot_77 was refused lately" while our extractor stood there. spot_77 was lost unturreted at 5:00.
- **3:31, t3, A1, none.** produce `group_B_t3` was refused ("not a group in the picture") and re-issued with "new" at 3:46.

## Sweep counts, verbatim

```
-- strategist-0.jsonl: 118 turns, 2 session replacements, 249 orders, 0 refused, 130 flagged of 876 hands' lines; landing median 0 s (p90 0), turn wall median 11 s
   orders: {'wait': 118, 'queue': 51, 'instruct': 48, 'produce': 22, 'mark': 6, 'lane': 2, 'transfer': 2}
   flagged: {'step-skipped': 53, 'step-waits': 30, 'hunt-ended-other': 11, 'engine-refused': 11, 'stopped-on-stop': 8, 'other-fault': 5, 'hunt-ended-leash': 4, 'follow-ended': 4, 'abandoned': 4}
-- strategist-1.jsonl: 115 turns, 2 session replacements, 271 orders, 1 refused, 97 flagged of 939 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 114, 'queue': 57, 'instruct': 51, 'produce': 31, 'lane': 9, 'mark': 5, 'remove': 4}
   refused: {'produce': 1}
   flagged: {'step-skipped': 25, 'stopped-on-stop': 21, 'engine-refused': 16, 'step-waits': 14, 'abandoned': 8, 'other-fault': 6, 'hunt-ended-other': 4, 'hunt-ended-leash': 2, 'follow-ended': 1}
-- strategist-2.jsonl: 113 turns, 2 session replacements, 239 orders, 1 refused, 143 flagged of 972 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 113, 'instruct': 53, 'queue': 42, 'produce': 15, 'signal': 4, 'mark': 4, 'situation': 2, 'remove': 2, 'lane': 2, 'transfer': 2}
   refused: {'corwin': 1}
   flagged: {'step-skipped': 54, 'step-waits': 19, 'hunt-ended-other': 18, 'hunt-ended-leash': 18, 'follow-ended': 10, 'engine-refused': 8, 'other-fault': 7, 'stopped-on-stop': 4, 'abandoned': 3, 'site-blocked': 2}
-- commander.jsonl: 19 turns, 0 session replacements, 36 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 25 s
   orders: {'direct': 18, 'transfer': 7, 'mark': 5, 'units': 3, 'signal': 1, 'say': 1, 'surrender': 1}
   flagged: {}
```

Notes on the sweep:
- **The landing figure is misleading here.** The sweep's "landing median 0 s" counts from the turn's end. In this realtime game, packets reached the hands' log 10-19 s after the turn's clock, for example 17:09 to 17:19, 21:34 to 21:46, and 15:43 to 16:01. No item here turned on that delay alone, so A5 is 0.
- **Most engine-refused lines cost nothing.** In the samples, they were the second builder on a frame that already stood (coravp G3 at 23:04, corvp H1 at 24:53, armhlt #h2) or a mex on a taken spot.
- **Most leash ends were not faults.** The other hunt-ended-leash lines sampled were at raiders the packet did not want chased, or at parties larger than the hunters.

## What Opus wanted that the harness has no lever for

- **3:31, t4.** Opus wanted to reclaim the wind turbine penning commander_t4 at spot_64. The commander repeated it at 22:50: "a constructor reclaims whatever building pens it in now".
  - No lever: `remove` takes handles only, and the picture names our buildings by type with no handle ("our Wind Turbine (corwin)" beside spot_64). Only actors and a factory's exit-lane yard entry carry handles.
- **16:10, 17:09 and 17:24, t2.** "It never goes toward the turret nest at spot_93, spot_92, spot_91", later "never goes west of spot_87", while the body fights at F4.
  - No lever: the harness has no keep-out area. The decode opens places named in a 'never' sentence (spot_93 at 0.96), and a fight_to or advance move has no bound, so a fight_to spot_87 engaged the nest's turrets.
- **25:09, t4.** "the unidentified party at H2 (party_202) is a corshiva's company and the commander never walks toward it".
  - No lever: the player has no way to tell the hands what a radar contact is. The menu kept calling it "1 unidentified ... we outweigh it heavily".

## Commands run (from the repository root; M = run/matches/1791751159-human-32/00)

```
grep -n 'name=' M/script.txt; tail -c 600 M/record-0.jsonl; tail -1 M/record-{1,2}.jsonl; grep '"k":"chat"' M/record-0.jsonl
run/impedance_sweep.py M --counts; run/impedance_sweep.py M --seat 0|1|2; run/impedance_sweep.py M --commander
run/minutes.py M; run/minutes.py SCRATCH/seat1; run/minutes.py SCRATCH/seat2
run/raid_ledger.py M; run/raid_ledger.py SCRATCH/seat1; run/raid_ledger.py SCRATCH/seat2
run/pro_baseline.py M / SCRATCH/seat1 / SCRATCH/seat2      (no card for this map)
run/hands_window.py M 17:10 17:40 group_W1_t2; ... 17:25 17:45; 8:05 8:30 group_P_t2 group_U_t2; 21:15 21:52 group_W1_t2
run/hands_window.py SCRATCH/seat2 13:33 13:50 group_P1_t4; 25:03 25:43 commander_t4
scratch tools (read-only on M): opening.py (record events by def name), track.py (one unit's positions),
  turns.py (a seat's prompts and calls by clock), tools/jidx.py (an offset index of each jev log),
  tools/jq.py (packets, decode reads, menus with ratings, plays for named actors), tools/gtrack.py (group size,
  place and task from call rows), tools/near.py (enemies near a point in a record sample)
```

Seats 1 and 2 were read through symlink directories in the scratch directory, which link record-0, jev-0, strategist-0 and terrain-0 to the seat's own files. There is no truth file (human game), and wake_read was not run (arena only).
