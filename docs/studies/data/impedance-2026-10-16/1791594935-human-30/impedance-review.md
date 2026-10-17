# Impedance review: 1791594935-human-30 / 00

**The game.** A human game on Comet Catcher Remake 1.8, played 2026-10-09. One seat of ours, strategist-0 ("Honkwaddle"): an Opus 5.5 medium player under an Opus 5.5 high commander, with Jev 1.13.0 as the hands. The opponent was [gecko]u6bkep, a person with lobby skill 30, playing Armada. **We won at 19:37.** group_G killed his commander at (2941,2767) in C4, and everything of his was destroyed between 19:37 and 19:41. The record runs to 19:49 and ends with 208 units of ours alive. The person said nothing in the chat; the only chat lines are our two announcements. Our economy led from about 5:00 (pro_baseline: 21 extractors at 8:00 against a pool median of 15). Only one extractor was lost all game (spot_21 at 13:45).

## Tally

| A1 | A2 | A3 | A4 | A5 | A6 | B1 | B2 | B3 | B4 | B5 | B6 | C |
|----|----|----|----|----|----|----|----|----|----|----|----|---|
| 4  | 2  | 3  | 4  | 0  | 3  | 0  | 0  | 0  | 1  | 1  | 2  | 3 |

**Denominators**
- Judgement moments examined: 8, of which 3 were clean.
- Turns and orders: 65 player turns with 163 orders, and 13 commander turns with 14 orders.
- Hands' lines: 814, of which 39 were flagged.
- Refusals: 1 refused per the sweep, plus 3 that the sweep does not flag. Those three are `signal: not a tool that can be batched` results inside `orders`.
- Takeovers: 1 session replacement, at 11:12.

**The eight moments**
1. The opening: A3 at 3:58 and A2 at 4:08.
2. The raid at spot_21 (13:45): clean.
3. The first 1.5x window (5:00-11:00) with the 4:06 raid order: clean.
4. The first push (8:12-9:30): A6 at 8:12, A1 at 8:36, B4 at 8:36 and B6 at 9:27.
5. The packet change at about a third (6:31, landed 6:49): A3 at 6:50, A4 ×3 between 7:53 and 8:09, and A2 at 8:11.
6. The packet change at about two thirds (13:24, landed 13:44): A3 at 13:49.
7. The last turns, 18:39 and 19:07: A4 and B5 (the duplicate advanced plant).
8. The last turn, 19:29: clean.

## Items, ranked by cost

The table holds 15 of the 20 items. The other five are in `impedance.json`: the A1 refusal at 2:43, the A4 leashes at 7:55 and 8:09, and the A2 turret skips at 4:08 and 8:11. All five cost nothing.

| # | Clock | Cat | Cost (confidence) | Item |
|---|-------|-----|-------------------|------|
| 1 | 14:36 | A4 | large (high) | **Six builders on one `#avp1` built two advanced vehicle plants.**<br>**Wanted:** 14:13 `queue` for six constructors, `["armavp #avp1", "assist"]`.<br>**Happened:** At 14:36 both constructor_540 and constructor_7444 played `build a armavp at spot_6 (G1) (#avp1) (from its list)`. The record shows armavp 16012 created at 14:46 at (6013,264) and armavp 20158 created at 14:51 at (5821,296); they finished at 17:50 and 17:56. 20158 stood in 16012's exit lane: Bull 4844 had `move_failed` from 18:54 to the end, and Bull 18257 was stuck from 19:33.<br>**Cost:** 2,600 metal and 14,000 energy went into a duplicate plant, the helpers were split between the two frames, and 1,900 metal of Bulls were trapped. Energy sat at 0 from about 19:07.<br>**Fix:** the same-tick id race, already fixed in 4ed68d8 on 2026-10-12, after this game. |
| 2 | 14:53 | B5 | large (medium) | **The report hid the second advanced plant for three minutes.**<br>From 14:53 to 17:43 both builders' entries read `building a Advanced Vehicle Plant ... it will be plant_16012`, with the same build percentage every turn (1, 8, 19 … 92%), at two different positions.<br>The player learned of the second plant only at 18:12, and reclaimed it at 19:07 once it showed in the lane.<br>**Cost:** the same 2,600 metal as item 1.<br>**Fix:** report. Each entry should name its own frame. |
| 3 | 16:08 | A6 | small (medium) | **The south raid group was named by description, so the hands never carried its route.**<br>**Wanted:** "The Blitz raid group (the armflash group from plant_19529, its own group): ... its way is spot_58, spot_63, ...". This carried the commander's 15:20 second front.<br>**Happened:** The decode rows at 16:08, 16:36, 17:10 and 17:33 read no place on the route for any group. At 18:05 group_M's route read 0.30-0.46. Only at 18:32, once the packet named group_M, did it read 0.98.<br>At 17:34 the player sent group_K to join group_G, not knowing that group_K was plant_19529's new group.<br>**Cost:** 8 Blitzes (880 metal) never raided. The commander at 18:20: "it doesn't stand at E6".<br>**Fix:** tool. `produce` with `"new"` gives no group name. |
| 4 | 11:36 | A6 | small (medium) | **Thrashing at even odds between two clauses of one packet.**<br>**Wanted:** "attacks the armstump party at D3 now ... until it is dead" and "Against a party that outweighs it, it walks to spot_21".<br>**Happened:** PLAYS were attack at 11:36, walk to spot_21 at 11:41, advance to spot_24 at 11:45, attack at 11:46, walk to spot_21 at 11:51, and advance to spot_24 at 11:59. The player's 11:45 note also calls it thrashing.<br>**Cost:** group_G lost 2+2 Stouts (900 metal).<br>**Fix:** jev. |
| 5 | 8:12 | A6 | small (medium) | **An attack order became a 1,100-elmo chase into his main body.**<br>**Wanted:** "group_G attacks the armflash at our extractor at spot_13 now ... until they are dead", in a packet that also says "holds at spot_1".<br>**Happened:** The engage task followed the retreating Blitz from (4138,986) to (2974,1348) by 8:29, into 8-9 of his Blitzes near his C1/C2 turret. group_G went from 5 to 4 soldiers and group_H from 5 to 3.<br>**Cost:** 3-4 units (about 400 metal).<br>**Fix:** words. Nothing in the packet bounded the attack. |
| 6 | 6:50 | A3 | small (high) | **Detachments hunted under his turret despite the packet.**<br>**Wanted:** "It does not go west of spot_1 and never fights under his turret at spot_4."<br>**Happened:** PLAYS `2 of group_B hunt party_15 (1 armflash) as group_C` at 6:50 and `... as group_D` at 6:54, with the quarry about 190 from the turret at spot_4. At ~7:02 there was a third hunt, as group_E. The gate read the 6:50 hunt as forbidden (`send_2_party_15.forbidden` 0.69), and the pick took it anyway at 0.72.<br>**Cost:** 2 Blitzes.<br>**Fix:** jev. |
| 7 | 3:58 | A3 | small (high) | **group_A kept roving after the packet stopped it.**<br>**Wanted:** "The Blitzes stop roving and fight as one body along this way: spot_26, spot_22, ..."<br>**Happened:** group_A kept roving from 3:58 to 4:23. The decode had read spot_26 at 0.96. The gate's `change` stood at 0.55-0.68, but the best move, `fight_to_spot_26`, was only 0.42-0.43, so the pick never moved the group.<br>The player gave the raid to group_B at 4:06: "group_A would not leave its rove".<br>**Fix:** jev. |
| 8 | 13:49 | A3 | small (high) | **Plants idled on their open entries.**<br>**Wanted:** "plants: work their production lists; never idle."<br>**Happened:** With only open entries left, plant_4070 stood idle from 13:49 to 13:55 with `make_armjanus` at 0.32-0.46 and `make_armstump` at 0.33-0.47. Over minutes 11-14 it was idle 36 s, and plant_3914 23 s.<br>The player (14:01) and the commander (13:50) both saw it. The player fixed it with counted lists at 14:13.<br>**Fix:** menu. |
| 9 | 8:36 | B4 | small (medium) | **The player blamed its go-now order for losses that came before the order landed.**<br>**Wanted:** the 8:15 "the body goes into his half now", which landed at 8:31. This carried the commander's 7:50 direction.<br>**Happened:** At 8:36, five seconds after landing, the player blamed the order for the chase losses in item 5: "the 'go now' body trickled into his C1/C2 Blitzes ...". It reversed to holding at spot_13 until the commander gave the push at 9:27.<br>**A line that would have surfaced it:** "your orders of 8:15 came into force at 8:31, 5 s ago".<br>**Fix:** report. |
| 10 | 12:24 | B6 | none (high) | **The player never carried the commander's scouting request.**<br>**Wanted:** the commander at 12:20: "Send one fresh Rover from the new plant ahead of the body to look at B4-B5 and A2-A5".<br>**Happened:** No order carried it. The only Rover made (12:36) died in our half at 14:46. His base went unscouted from about 8:00, and his advanced plant was never seen.<br>**Fix:** brief. |
| 11 | 9:27 | B6 | none (medium) | **The commander misread the seat's hold as a tool limit.**<br>The commander noted "player held group_G at spot_13 waiting on a 'push' signal it cannot give itself; gave push now". In fact the player had set `push` for 30:00 on purpose: "Will give push at about 8 Stouts + Blitzes (~2.5k)".<br>**Fix:** brief. |
| 12 | 8:36 | A1 | none (high) | **A batched `signal` was refused.**<br>`signal {"push": null}` inside `orders` returned `signal: not a tool that can be batched`. The player knew, and re-issued it alone at 9:00.<br>The sweep counts none of these three `signal` refusals (items 12-14).<br>**Fix:** tool. |
| 13 | 11:45 | A1 | none (high) | **A batched `signal` was refused.**<br>`signal {"push2": "12:20"}` returned `signal: not a tool that can be batched`. The player re-batched it at 12:07.<br>**Fix:** tool. |
| 14 | 12:07 | A1 | none (high) | **A batched `signal` was refused, and the player believed it set.**<br>`signal {"push2": "12:40"}` returned `not a tool that can be batched`, yet the player's note read "push2 set for 12:40". The commander gave push2 at 12:24, which made it moot.<br>**Fix:** tool. |
| 15 | 7:53 | A4 | none (high) | **The main body changed its name three times in three seconds.**<br>`8 of group_B hunt party_19 as group_F`, then `group_B: join group_F`, then `8 of group_F hunt ... as group_G`. From 7:55 to 8:12 the packet's group_B paragraph pointed at nothing.<br>**Fix:** code-rule. |

## The sweep's counts, verbatim

```
-- strategist-0.jsonl: 65 turns, 1 session replacements, 163 orders, 1 refused, 39 flagged of 814 hands' lines; landing median 0 s (p90 0), turn wall median 15 s
   orders: {'wait': 64, 'instruct': 38, 'queue': 29, 'produce': 18, 'lane': 5, 'signal': 5, 'mark': 3, 'remove': 1}
   refused: {'produce': 1}
   flagged: {'step-skipped': 14, 'engine-refused': 8, 'site-blocked': 4, 'stopped-on-stop': 3, 'hunt-ended-other': 3, 'abandoned': 3, 'hunt-ended-leash': 2, 'step-waits': 1, 'other-fault': 1}
-- commander.jsonl: 13 turns, 0 session replacements, 14 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 18 s
   orders: {'direct': 12, 'signal': 2}
   flagged: {}
```

The sweep's `refused: 1` misses three `signal: not a tool that can be batched` results (items 12-14).

## What Opus wanted that the harness has no lever for

- **3:41 (landed 3:58): splitting a group by unit type.**
  - **Wanted:** "group_A (the raid in his half): its Rovers leave it to rove on their own (rove_1, one Rover at a time) and are left to rove. The Blitzes stop roving and fight as one body".
  - **No lever:** no move splits a group by unit type. A roving group's menu offers stay, hold, go and fight_to places, join, and follow. `rove_N` is not offered while the group roves, and it does not choose soldiers by type.
- **13:24 (in every packet from 13:44 to the end): keeping helpers out of a factory's exit lane.**
  - **Wanted:** "constructors: ... Never stand in a plant's exit lane."
  - **No lever:** a builder's moves are build, help, go, follow, take_apart, repair and stay. The engine places a helper, and no move or rule keeps it out of the lane. At 18:12 constructor_16324 and constructor_7444 were each "helping lab_N build ... standing in plant_N's exit lane". At 12:28 two constructors stood in plant_4070's lane.
- **11:12, 13:24, 17:43: roles by unit type inside one group.**
  - **Wanted:** "Shellshockers stay behind the Stouts." Also: "Shellshockers and Janus fire on his commander from their reach; the Stouts and Blitzes close on it all together, never one or two walking into its D-gun alone".
  - **No lever:** each group takes one task. Nothing gives roles or spacing by unit type within a group, and no attack waits for the group's tail to arrive. Reinforcements drive in one by one: at 19:07 the tail was 3,626 behind the front.
  - **Confidence:** low. The lane footwork (form, march, fan) may cover part of this.

## Commands run

All from the repository root, with M = run/matches/1791594935-human-30/00.

- `grep -n 'name=' M/script.txt; tail -c 600 M/record-0.jsonl`; `grep '"k":"chat"' M/record-0.jsonl`.
- `run/impedance_sweep.py M --counts`, `--seat 0` (read in full), `--commander`, and `--seat 0 --all --from 15:45 --to 16:50` and `--from 8:15 --to 9:00`.
- `run/minutes.py M`; `run/raid_ledger.py M`; `run/pro_baseline.py M` (no carded side for our start) and `run/pro_baseline.py M --all-starts`.
- `run/commander_turns.py M --calls` and `--notes`.
- `run/hands_window.py M` over these windows: 1:30-2:10, 3:56-4:24, 6:48-7:06, 6:48-7:56, 7:53-8:12, 8:12-8:40, 11:30-11:58 and 11:30-12:02.
- Scratch readers in the scratch directory, over `jevlog.lines` (decode rows, packet landings, group menus and gate flags), the record (events, idle flags, enemy sightings) and the transcripts (signal results, report lines).
- Every seat was read; seat 0 is the only seat, so no symlink directories were needed. `wake_read.py` was not run: it is for arena games. Nothing was unreadable.
