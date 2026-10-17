# Impedance review: 1791749780-human-31 (human game, 2026-10-11)

Comet Catcher Remake 1.8, realtime, three Cortex seats of ours (strategist-0 = t2 north G1, strategist-1 = t3 south H8, strategist-2 = t4 middle G4), each an Opus 5.5 medium player with Jev 1.13.0 hands, under an Opus 5.5 commander at high effort, against two people: [gecko]thebluegecko (Cortex) and [gecko]u6bkep (Armada). **Lost at 19:50**: the commander surrendered after t4 (commander dead 17:30) and t2 (commander dead 19:33) fell to thebluegecko's Tiger block; the record ends at 20:09. No truth file, so the enemy checks use the record's `enemy_seen` only. The only thing a person said in the chat was "gg wp" (20:02).

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 3 |
| A2 accepted, not executed | 3 |
| A3 overruled by the pick | 2 |
| A4 overruled by code | 1 |
| A5 late | 1 |
| A6 read differently | 3 |
| B1 in the hands' picture, not the report | 0 |
| B2 in the record, in nobody's picture | 0 |
| B3 report wrong or stale | 0 |
| B4 in the report, missed (Opus's miss) | 0 |
| B5 the hands' own lines misleading | 0 |
| B6 commander-to-player hop | 3 |
| **C**: clean judgement moments | **4 of 12** |

Denominators: 12 judgement moments (four a seat); orders 188 / 186 / 193 (seats 0 / 1 / 2) and 21 from the commander; hands' lines 552 / 454 / 488; refused 0 / 0 / 3.

The moments were (a) the opening, (b) one raid from the raid ledger, (c) the first push or the first window at 1.5x, and (d) or (e) one more for each seat. I took one of each kind per seat instead of using up a seat's four on raids, so that every seat got an opening and a push. Clean moments: the opening of all three seats, which ran exactly as listed (3 extractors, 3 solars, plant at 1:09 / 1:15 / 1:16, constructor first), and t2's 3:42 raid on spot_6 (answered 1 s from first sight; the 900 leash ended group_G_t2's hunt at 3:40, the pick attacked again at 3:41, and the quarry was dead by 3:53).

## Items, costliest first

1. **16:50-17:21, commander -> t4, B6, cost large, confidence low.**
   - Wanted (commander 16:50): "All of it gathers at t4_line (G4, spot_34/spot_35) as one body, on your turrets, and meets the Tiger block there together."
   - What happened: the transfer landed at 17:04 ("group_C1_t4 formed from the 19 soldiers given to this seat"; the north and south bodies, about 4,000 apart, became one group). The direction was held back until "their next report after 17:15". At 17:09 t4 read "the commander's direction of 15:37 stands" and ordered "attack party_134 ... every group". The hands split group_C1 into hunts at 17:05 and 17:06, and the body never gathered: at 17:21 group_E1_t4's tail was 4,012 behind. t4 first saw the direction at 17:21, 31 s after it was written, with its commander at 3%.
   - Cost: t4's commander died at 17:30; t4 lost 14,075 metal at G4 in minute 18. Whether one gathered body would have held is not measurable.
   - Fix (report): show a direction to the seat in the same look as the transfer it explains.
2. **10:50-11:04, commander (t4 -> t3), A6, cost large, confidence medium.**
   - Wanted: transfer group_N_t4 to t3, with the paragraph "t4's 7 Brutes (group_N) are now yours ... Join them into one body now".
   - What happened: before the transfer ran, t4's pick split the group (10:56 "4 of group_N_t4 hunt party_58 ... as group_S_t4"), so at 11:04 the name meant 2 units and t3 was given 2 corraid. t3 at 11:13: "only 2 of t4's Brutes reached us ... So I am holding the body on the spot_75 Twin Guard." While t3 held, it lost spot_46, spot_55, spot_53 and spot_57 (11:09-11:45). The other Brutes reached t3 only with the 12:20 transfer, at 12:38.
   - Fix (tool): resolve a group's members when the transfer is called, and say in the result how many units went.
3. **19:28-19:33, t2, A3, cost large, confidence medium.**
   - Wanted (19:18 packet, landed 19:28): "commander_t2: it walks away from the Tigers along the east edge: spot_17, then spot_25, ...".
   - What happened: the decode read the places at 0.95-0.97, and Jev's change flag stood at 0.77-0.85 every second, but go_spot_17 was rated 0.32-0.45 and never picked. The commander kept its 19:06 "follow group_Q_t2" and died at home at 19:33, which eliminated the seat. 14-24 Tigers stood at G1, so an escape was not assured.
   - Fix (jev): when change is rated high and the move the packet names exists, a single remaining move should not fall under the bar.
4. **0:20 (by 6:00) - 11:47, commander -> t4, B6, cost small, confidence high.**
   - Wanted: "One anti-air turret at your plant by 6:00."
   - What happened: no AA in any t4 packet or list until 11:26; first Thistle at 11:49. The commander's own 12:20 note says "my 0:20 AA ask was not carried out".
   - Cost: t4 lost 1,375 metal to bombers and its commander was hit to 61%. But t2, which had a Thistle from 7:33, lost 1,170 to the same bombers, so one turret's worth is unclear.
   - Fix (brief).
5. **0:20 - 11:29, commander -> t3, B6, cost small, confidence high.**
   - The same AA ask. t3 carried it in its 1:14 and 1:58 packets ("One corrl at home by the plant once we have four constructors"), then dropped it from the 2:13 Rover-crisis packet onward. First Thistle at 11:54, killed at 11:59.
   - Cost: 1,130 metal lost to bombers.
   - Fix (brief).
6. **13:12-13:31, t2, A5, cost small, confidence medium.**
   - Wanted: "group_G_t2 shells his commander (corcom) now from the Lashers' reach ... From a party that outweighs it, it walks to spot_16."
   - What happened: the turn took 19 s, so the packet landed at 13:27. Until then the 12:56 packet's fallback "walks to spot_15" was in force, and his commander stood at spot_15. The pick split the group and walked it to spot_15 into his commander and three Quakers. About 1,245 metal was lost, nearly all before 13:27.
   - Opus's 13:31 note blamed its own order: "The strike at his commander failed: 7 units lost". The wake line's "since your last orders" covered losses from before the orders landed.
   - Fix (report): say when the last orders reached the hands in a realtime game.
7. **2:10-2:27, t3, A4, cost small, confidence medium.**
   - Wanted (1:58 packet): Rovers at our extractors "are group_A_t3's business: it attacks them with the whole group"; only its fastest Rascal roves.
   - What happened: the whole group was roving. Its menu at 2:10-2:11 had no attack on the raiding parties, the rove itself "ran from his Rover ... from spot_70", and from 2:13 to 2:24 the group was closed (not asked) while constructor_26240 was hit every second. Lost in the window: spot_70 twice and a solar.
   - Fix (code-rule): ask a roving group again, with attack moves, when a party its packet names stands at our extractors.
8. **10:55-11:01, t4, A6, cost small, confidence high.**
   - Wanted: "It walks to spot_40 now, without stopping to fight."
   - What happened: the decode read a detachment as allowed (group_N_t4 detach 4 at 0.64). The pick sent 4 Brutes after a single Brute, they met the E6 mass at D5, and one died. This split is also the cause of item 2.
   - Fix (words: "sends no detachment" lands; "without stopping to fight" does not).
9. **5:16-5:29, t2, A6, cost small, confidence medium.**
   - Wanted (5:02 packet): "staying out of reach of the corexp at spot_0"; group_J_t2 holds at spot_1.
   - What happened: the decode read "group_J_t2|place|spot_0 = 0.92". The pick thrashed between spot_1, spot_13 and "advance to shelling" (5:23, 5:28), and an Incisor died at 5:29 to the corexp.
   - Fix (jev): the decode reads named places without their negation. The same thing happened later, when "never walks to spot_8" read as spot_8 at 0.95-0.97, with no play following.
10. **13:21-13:53, t3, A3, cost small, confidence medium.**
    - Wanted: "At spot_53 it gathers until group_R_t3 has joined it."
    - What happened: the decode put the join at 0.75, but at 13:23 the pick sent group_D_t3 on to e6_beamer alone, and group_R fought alone at E5. The group came back at 13:41 and took group_R in at 13:53; the whole body then killed both Beamers by 14:24.
    - Fix (jev).
11. **11:26-12:17, t4, A2, cost small, confidence high.**
    - What happened: during the bomber raid the tool accepted "corrl", "corrl" and "cormadsam" with no place ("3 steps, done in order"), then the hands skipped each one: "it needs a place" at 11:33 (four steps) and 12:17. Opus re-issued with places at 11:38.
    - Fix (tool): refuse a turret step without a place at the call.
12. **6:56-7:29 (and 8:36-8:52), t3, A2, cost small, confidence high.**
    - What happened: a duplicate extractor order on spot_57, started by another builder 2 s earlier, came back "the engine refused ... the site was bad". That refusal then made the hands skip the queued Guard: "the extractor at spot_57 was refused lately" at 6:57, 7:14 and 7:29, although our extractor had stood there since 6:54. spot_57 never got its Guard (the extractor was lost at 11:45). spot_55 had the same pattern, and its Guard came only at 10:26.
    - Fix (code-rule): clear the refusal memory once our extractor stands on the spot.
13. **7:27, t2, A2, cost small, confidence medium.**
    - What happened: "the engine refused a cormex at H2: the site was bad" on spot_12, which stood empty from 4:11 until t4 took it at 9:40. A Guard of ours stood there from about 5:30; the cause is not settled.
14. **17:21 / 17:30 / 17:38, t4, A1 x3, cost small, confidence high.**
    - What happened: each whole transfer was refused for one name that had died or merged during the 8-9 s turn ("REFUSED: constructor_17376", "group_G1_t4", "group_F1_t4: nothing of ours has that name"). Opus noticed every time and at 17:47 sent eleven one-name transfers, which all went through (18 units given at 17:53).
    - Fix (tool): transfer the names that stand and list the missing ones.

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 86 turns, 2 session replacements, 188 orders, 0 refused, 67 flagged of 552 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 86, 'instruct': 43, 'queue': 36, 'produce': 16, 'transfer': 3, 'mark': 2, 'lane': 2}
   flagged: {'step-skipped': 17, 'step-waits': 15, 'other-fault': 9, 'engine-refused': 7, 'stopped-on-stop': 6, 'abandoned': 5, 'hunt-ended-leash': 4, 'hunt-ended-other': 3, 'follow-ended': 1}
-- strategist-1.jsonl: 82 turns, 2 session replacements, 186 orders, 0 refused, 69 flagged of 454 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 81, 'queue': 45, 'instruct': 41, 'produce': 15, 'lane': 3, 'mark': 1}
   flagged: {'stopped-on-stop': 19, 'step-skipped': 18, 'step-waits': 9, 'abandoned': 9, 'other-fault': 5, 'hunt-ended-leash': 5, 'hunt-ended-other': 2, 'engine-refused': 2}
-- strategist-2.jsonl: 85 turns, 2 session replacements, 193 orders, 3 refused, 73 flagged of 488 hands' lines; landing median 0 s (p90 0), turn wall median 10 s
   orders: {'wait': 85, 'queue': 36, 'instruct': 36, 'transfer': 20, 'produce': 11, 'lane': 5}
   refused: {'transfer': 3}
   flagged: {'step-skipped': 24, 'step-waits': 16, 'stopped-on-stop': 16, 'other-fault': 5, 'engine-refused': 4, 'hunt-ended-leash': 3, 'hunt-ended-other': 3, 'abandoned': 1, 'follow-ended': 1}
-- commander.jsonl: 14 turns, 0 session replacements, 21 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 30 s
   orders: {'direct': 12, 'transfer': 4, 'say': 3, 'mark': 1, 'surrender': 1}
   flagged: {}
```

The hunt-ended-leash lines I sampled were not faults. On t2 at 3:40 the pick re-attacked at once. On t3 at 5:36 the leash stopped a chase of a lone armflash, which agreed with the packet's "never chases it out of our lane". Most engine-refused extractor lines were harmless duplicates of an extractor another of our builders had just started (t3 6:56 and 8:36, t4 11:19). Some engine-refused lines were spots that he held (spot_23 and spot_21 for t2, spot_37 at 4:46 for t4). The harm from the duplicates came afterwards, in the Guard skips of item 12.

## What Opus wanted that the harness has no lever for

- **10:03 (landed 10:12), t4:** "group_N_t4 attacks his buildings at spot_37 (E4) now with the whole group, closing in on the corllt there together, and keeps attacking until the corllt is dead." The corllt was out of sight and not a party, so the menu had no move against it: only go_spot_37 (0.20) and fight_to_shelling (0.48). The pick took the route's next stop instead (fight_to_spot_44, 0.50). No move attacks a known building by its place when it is not in sight.
- **16:50, commander:** one body of two seats' groups, standing at t4_line at once. The commander's only levers are `transfer` and prose. The receiving seat's hands read only their own player's packet, so units given before that player has written for them fall to the old packet. There is no commander-side way to give the hands a standing task with the units it gives (item 1).

## Checks with nothing to report

At every moment I compared the report against the record. The `enemy in sight`, the parties and our own state lines matched the record's `enemy_seen` and our units. Stuck units, blocked exit lanes and the metal store were all carried as wake lines. I found no B1-B5 fault that changed a decision. One near miss, not counted: at 14:12 t2 queued spot_51 and spot_48, which belonged to t4. The report's list of free spots did not name them, and nothing was lost.

## Commands run

```
grep -n 'name=' MATCH/script.txt; tail -1 MATCH/record-{0,1,2}.jsonl
run/impedance_sweep.py MATCH --counts; run/impedance_sweep.py MATCH --seat 0|1|2; run/impedance_sweep.py MATCH --commander
run/minutes.py MATCH --to 20; run/minutes.py SEAT1DIR --to 20; run/minutes.py SEAT2DIR --to 20
run/raid_ledger.py MATCH | SEAT1DIR | SEAT2DIR; run/pro_baseline.py MATCH | SEAT1DIR | SEAT2DIR (no carded pool for G1 and H8)
run/commander_turns.py MATCH --calls (and on SEAT1DIR, SEAT2DIR)
run/hands_window.py on: seat0 3:30-3:58, 5:14-5:32, 13:12-13:44, 19:12-19:34; seat1 2:00-2:45, 5:24-5:44, 11:00-11:45, 13:01-14:26; seat2 9:50-10:46, 10:10-10:14, 10:44-11:08, 17:03-17:22
scratch readers (jevlog rows): decode rows and packets per window; pass/worlds_gate rows for group_A_t3 2:09-2:28, group_N_t4 10:11-10:13, commander_t2 19:27-19:34
record readers: build/loss events 0:00-4:00, AA buildings, given/taken, extractor histories of refused spots, losses to corshad
grep '"k":"chat"' MATCH/record-0.jsonl
```

MATCH = run/matches/1791749780-human-31/00. SEAT1DIR and SEAT2DIR are symlink directories in my scratch directory (record, jev, strategist, terrain and script.txt of seats 1 and 2), used for the seat-0-only tools.
