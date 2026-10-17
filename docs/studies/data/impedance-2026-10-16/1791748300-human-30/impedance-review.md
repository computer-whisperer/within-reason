# Impedance review: 1791748300-human-30 (the second human-30, Eclipsed 0.9.8)

A realtime game against two people ([gecko]thebluegecko and [gecko]u6bkep, Armada). We had four seats (t2-t5 = strategist-0..3, each an Opus 5.5 medium player on Jev 1.13.0) under an Opus 5.5 commander. The result is likely a loss at about 21:45. The chat has no result line. The record ends at f=39162 with "unexpected end of file", 17 s after his Liche atomic bombers killed t3's commander (21:28) and t5's (21:35). t4's commander had already died at 14:52, so only t2's commander was still alive. The commander wrote at 14:15: "The lead is gone".

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 2 |
| A2 accepted, not executed | 4 |
| A3 overruled by the pick | 6 |
| A4 overruled by code | 1 |
| A5 late | 0 |
| A6 read differently | 2 |
| B1 in the hands' picture, not the report | 4 |
| B2 in the record, in nobody's picture | 0 |
| B3 report wrong/stale | 0 |
| B4 in the report, missed (Opus's miss) | 1 |
| B5 hands' lines misleading | 1 |
| B6 commander-to-player hop | 2 |
| C followed, context enough | 3 of 12 moments |

Denominators:
- Judgement moments: 12 (three a seat), 3 of them clean.
- Orders: 643 player orders across the four seats, plus 29 from the commander.
- Hands' lines: 1,481, of which the sweep flagged 158.
- Refused: 2.
- The cost-ranked items: four `large`, the rest `small` or `none`.

The commander's turns ran 11-43 s of wall time each, the seats' about 13 s (median). Orders landed at the end of the turn, so no turn's lateness changed an outcome I could measure: A5 is 0.

## Items, by cost

1. **5:15-8:15, commander, B6, large (medium).**
   - Wanted (commander 5:15): "Every seat's raiders join into one body at meet_d5 ... never one by one". The seats' packets repeated it.
   - Happened: no seat waited for the others. t3 left meet_d5 for his_post_f7 at 6:40, t5 held at spot_31 at 6:58, and t4 gathered at spot_31 at 8:15.
   - Evidence: commander 8:15: "three directions asking them to meet were each read by a seat on its own ... we lost 3,900 against 2,555".
   - Fix class: tool (a gather across seats). Also in the no-lever list below.
2. **14:35, t4, B4, large (medium).**
   - The report said: "commander_t4: at spot_20 (D2); walking to back_yard, 969 to go ... party_54 (2 armjanus, 6 armstump, 6 unidentified) 444 away ... contact in 6 s". Opus's note answered "Commander back home." and kept the walk rule.
   - Happened: the walk, picked at 14:31 under "A party that outweighs it: it walks to back_yard", ran east through E2 into the block. Hp went 82 → 6 over 14:39-14:51, and the commander died at 14:52 at (3849,1280).
   - Fix class: report. What would have surfaced it: a wake for "commander under fire", whose actor line names the party its walk runs into.
3. **13:23, t4, A2, large (high).**
   - Wanted: `armavp home #avp` on four constructors. The commander had made tier 2 at t4 "the priority".
   - Happened: the record has `no_site` for all four at 13:36. The host dropped the command (crates/ai-host/src/lib.rs) and the constructors stood in the plant's exit lane.
   - Then came three more failures (items 11-13). The first frame stood at 15:21 (at D1, abandoned) and at 15:48 (at E2), and the plant finished at 17:20, about 4 min after the order. Mausers never reached t3.
   - Fix class: tool.
4. **6:03-6:37, t2, A3, large (medium).**
   - Wanted (5:47): "the whole group attacks his buildings at spot_37 ... until the armbeamer is dead. Its way after that: meet_d5". Then 6:02: "fights its way to his_com_f7".
   - Happened: PLAYS 6:03 "attack party_5 (1 armflash)", 6:09 "attack party_15 (4 armflash)", 6:22 "attack party_15 (5 armflash) with the whole group" (attack_party_15 rated 0.55). The decode read his_com_f7 at 0.98, so the mark landed.
   - Cost: 7 Blitzes died at B5 (770 metal), the group going from 11 to 4 in 15 s. Opus's own lane had flee off.
   - Fix class: jev.
5. **2:06-18:36, t5, B1, small (high).** 10 extractor steps were skipped because an ally held the spot or was walking to it ("spot_20 is being taken by a builder of another seat of ours"). t5 stood at 2-4 extractors all game. Opus, 3:08: "Other seats keep taking the spots we walk to". The report gives a free-spot count and a nearest list, but no allied holdings or claims. Fix class: report.
6. **2:13-18:36, t2, B1, small (high).** 19 such skips; t2 stood at 5-7 extractors from 4:00 on. At 8:15 Opus believed allies had taken "the spots assigned to t2 (49, 50, 51 ...)", but 49-51 were t3's in a split that t2 never saw: each seat is shown only its own paragraph. Fix class: report.
7. **11:34-14:20, t2, A6, small (medium).**
   - Wanted (11:34): "its Rover leaves alone to rove (rove_1), his start box's south-west first (spot_55, spot_58)". This followed the commander's 9:45 and 11:15 asks for A7/A8.
   - Happened: the rover worked the north ("12:19 gave up looking at spot_0", then spot_23, spot_1, spot_43). No unit of any seat reached the south-west corner after 9:00. His advanced labs and aircraft plant were never seen, and the Liche was first seen at 21:15.
   - Fix class: menu.
8. **15:26-15:42, t3, A3, small (medium).**
   - Wanted (14:54, then 15:17): attack "with the whole group".
   - Happened: PLAYS "8 of group_I_t3 hunt party_61 (1 armzeus) as group_J_t3", then the same as group_K_t3. The hunt ended on the 900 leash at 15:42.
   - Cost: 2 Stouts (450 metal); spot_40 was lost at 15:49.
   - Fix class: jev.
9. **14:37-14:51, t4, A3, small (medium).**
   - Wanted (14:14): "a party of three or more of his at our extractors or turrets on the D1/D2 row (spot_20 ...) is its business first, whole".
   - Happened: party_54 stood at spot_20 from 14:35. Its answer was rated 0.78-0.84, but group_H_t4.attack_party_54 only 0.19. The pick detached 2 against party_55 at 14:45, then played "gather on its front", and attacked only at 14:51.
   - Cost: spot_20 lost at 14:49, rebuilt at 15:21.
   - Fix class: jev.
10. **12:51-13:04, t3, A3, small (medium).**
    - Wanted (12:36): "holds at spot_46 as one body ... never advances on a `shelling` place".
    - Happened: the pick alternated between "walk to spot_46" and "shell party_39 from their reach" about every second (ten changes in 12 s).
    - Cost: 4 soldiers lost. Opus, 12:55: "the hands kept switching between shelling and walking".
    - Fix class: jev.
11. **14:36, t4, A2, small (high).** The second order, `armavp avp_yard #avp1` on five constructors, was placed at (5928,648), 780 elmos from the mark. Then "the engine refused a armavp at G1: the site was bad" five times. Fix class: tool.
12. **15:18, t4, A6, small (high).** The third order, `armavp #avp2` on seven constructors, had no place and one shared id. The constructors got different sites, and a frame went up at D1 on the front. Opus, 15:24: "started at spot_20 (front) by the shared-id list; stopped it". The frame was abandoned at 0% at 15:57. Fix class: tool.
13. **15:26, t4, A4, small (medium).** The fourth order, `armavp back_yard #avp3`, hit the blocked-site watchdog four times ("refused for 90 s, and the order goes again elsewhere"). The re-placed site was E2 at e2_gate, the front passage. Opus, 16:09: "three constructors stood 30 s unable to reach the armavp frame at E2". Fix class: code-rule.
14. **13:36-14:14, t4, B5, small (high).** The report showed "build a armavp at home (E1) (#avp) (from its list)" and "walking to build a Advanced Vehicle Plant (armavp) at home (E1), 409 to go" for 38 s after the host had found no site. Opus, 13:59: "lane block ... transient, left alone". Fix class: report.
15. **0:20, commander, B6, small (high).** Wanted: "Each seat: five constructors by 5:00". No seat changed its produce list, and every seat had 3 constructors by 5:00. Fix class: brief.

Also counted, in impedance.json:
- B1 t4: 9 skips, among them a 4:08 list of six spots that allies already held.
- B1 t3: 4 skips.
- A3 t5 7:24: "advance to spot_37" against "never attacks into the Beamer at D4"; one Gator lost.
- A2 t5 4:18: a cormex at D6 refused, the spot being his.
- A2 t5 15:25: corhllt at G2 refused twice.
- A1 t3 21:30: the transfer was refused, `group_Q_t3` having merged one second earlier. The note assumed it was done, and it was never re-issued; cost none.
- A1 t5 6:29: `group_A` for `group_A_t5`, re-issued 11 s later; cost none.
- A3 t3 20:54: a detachment of 8 against "with the whole group"; the hunt ended on the leash at 0 s and the detachment rejoined; cost none.

Clean moments:
- t2 21:17-21:45: anti-air lists done. The order to walk the commander to sea_f1 landed at 21:45, and the commander was alive.
- t3 8:40-9:56: the route was the packet's. The deaths under the D4 Beamer followed Opus's own "Footwork flee off".
- t5 21:13-21:35: lists done. The commander died to the Liche as its last order landed.

Flagged lines judged and not counted:
- 'abandoned' lines followed Opus's own `stop` or a raid retreat.
- The remaining leash ends (t3 8:36, t4 4:27) were not raiders the packet wanted dead.
- The t5 14:09 spot_31 loss: the packet said to answer, the pick did not answer 13 units with 7, and Opus's 14:12 note endorsed falling back.

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 72 turns, 1 session replacements, 169 orders, 0 refused, 50 flagged of 319 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 72, 'queue': 36, 'instruct': 35, 'produce': 18, 'mark': 5, 'lane': 2, 'transfer': 1}
   flagged: {'step-skipped': 20, 'step-waits': 10, 'stopped-on-stop': 9, 'other-fault': 7, 'abandoned': 4}
-- strategist-1.jsonl: 81 turns, 2 session replacements, 174 orders, 1 refused, 31 flagged of 496 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 81, 'instruct': 39, 'queue': 26, 'produce': 14, 'lane': 12, 'mark': 1, 'transfer': 1}
   refused: {'transfer': 1}
   flagged: {'step-waits': 8, 'step-skipped': 7, 'stopped-on-stop': 5, 'other-fault': 4, 'hunt-ended-leash': 4, 'hunt-ended-other': 2, 'abandoned': 1}
-- strategist-2.jsonl: 59 turns, 1 session replacements, 140 orders, 0 refused, 44 flagged of 395 hands' lines; landing median 0 s (p90 0), turn wall median 15 s
   orders: {'wait': 58, 'queue': 31, 'instruct': 31, 'produce': 12, 'mark': 4, 'lane': 4}
   flagged: {'step-skipped': 15, 'stopped-on-stop': 12, 'engine-refused': 6, 'site-blocked': 4, 'step-waits': 3, 'hunt-ended-leash': 1, 'hunt-ended-other': 1, 'abandoned': 1, 'other-fault': 1}
-- strategist-3.jsonl: 74 turns, 1 session replacements, 160 orders, 1 refused, 33 flagged of 271 hands' lines; landing median 0 s (p90 0), turn wall median 11 s
   orders: {'wait': 73, 'queue': 36, 'instruct': 33, 'produce': 16, 'lane': 1, 'transfer': 1}
   refused: {'produce': 1}
   flagged: {'step-skipped': 13, 'engine-refused': 6, 'other-fault': 5, 'stopped-on-stop': 4, 'step-waits': 2, 'abandoned': 2, 'site-blocked': 1}
-- commander.jsonl: 15 turns, 0 session replacements, 29 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 31 s
   orders: {'direct': 15, 'transfer': 8, 'mark': 4, 'say': 2}
   flagged: {}
```

## What Opus wanted that the harness has no lever for

- **5:15, commander** (again at 6:45; the seats repeated it at t2 5:47, t5 5:52 and t4 7:06): "Every seat's raiders join into one body at meet_d5 ... They go in all together, from more than one side, never one by one."
  - Every seat can walk its group to meet_d5, but no move makes a group wait for, or march with, another seat's group.
  - The only join across seats is the commander's `transfer`, which was first used at 8:15, after the bodies had been worn down one at a time.
- **3:45, commander** (again at 9:45): "Spots, so that seats stop walking to each other's: ... free spots go to the seat named in its own paragraph and to no other."
  - Nothing carries spot ownership between seats. A step on another seat's spot is accepted and skipped only when the builder arrives: 42 such skips across the seats.
  - Each seat sees only its own paragraph of the split.
- **11:34, t2** (from the commander's 9:45 and 11:15): "its Rover leaves alone to rove (rove_1), his start box's south-west first (spot_55, spot_58)".
  - The rove takes no destination or order of places, and a lone scout can leave a group only as a rove.
  - The south-west was never looked at after 9:00, and his air plant was never seen before the Liches came.
- **21:35, t5**: "The commander attacks the nearest armliche with its laser now and keeps attacking while one is in reach."
  - The commander's weapons do not fire at aircraft, and no menu move attacks an air unit with them. The commander died the same second.

## Commands run

From the repository root. MATCH = run/matches/1791748300-human-30/00. S = the scratch directory, /tmp/claude-1000/-home-christian-workspace-playground-bar-bots/93f9e8f3-fea1-482e-a1c2-cb750e01d6e2/scratchpad/impedance/agents/1791748300-human-30.

For the seat-0-only tools, I made symlink directories S/seat1/00, S/seat2/00 and S/seat3/00 for seats 1-3. Each holds `record-0.jsonl`, `jev-0.jsonl`, `strategist-0.jsonl`, `terrain-0.bin`, `script.txt` and `commander.jsonl`, pointing at that seat's files.

- `grep -n 'name=' MATCH/script.txt`; `tail -1 MATCH/record-N.jsonl` (N = 0-3); `grep '"k":"chat"' MATCH/record-0.jsonl` (11 lines, all banners and "gl hf"); `tail -40 MATCH/bot.log`
- `run/impedance_sweep.py MATCH --counts`; `--seat 0..3`, saved to S/sweep-N.txt; `--commander`
- `run/minutes.py MATCH --to 22`; `run/raid_ledger.py MATCH` and `S/seatN/00`; `run/pro_baseline.py MATCH` (no carded side)
- `run/hands_window.py`:
  - MATCH 5:47 6:35 and 6:17 6:45 (group_B_t2)
  - S/seat1/00 9:20 10:05 (group_D_t3)
  - S/seat1/00 12:36 13:30 (group_D_t3)
  - S/seat1/00 15:20 15:50 (group_I_t3, group_K_t3)
  - S/seat1/00 20:54 21:15 (group_O_t3)
  - S/seat2/00 14:36 14:53 (commander_t4)
  - S/seat3/00 13:55 14:14 (group_D_t5, commander_t5)
- Helpers written for this review, in S/bin:
  - `turns.py` (a seat's prompts and calls in a window)
  - `jw.py` (jev log decode, gate, menu and played rows in a window)
  - `deaths.py` (our losses in a window)
  - `write_json.py` (writes impedance.json)
- Python reads of the records for extractor events by seat, commander deaths, armavp `no_site`/`placed`/`created`, the t4 commander's track, first sightings of enemy types, the `xi` damage by type, and units in the south-west after 9:00.
- `crates/ai-host/src/lib.rs` and `crates/bot/src/brain/mod.rs` read for how `BuildSiteNotFound` is handled.
