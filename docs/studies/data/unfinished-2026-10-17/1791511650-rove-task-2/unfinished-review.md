# Unfinished structures: 1791511650-rove-task-2/00

Arena duel on Comet Catcher Remake 1.8 (commit 97f51e5), 2026-10-08. One seat: Armada, Opus 5.5 medium as the player, no commander. Layers: news, same, fuse, openers, split, tick, places, again, near. We **won at 21:11** against BARb hard_aggressive (Cortex).

frames.py, seat 0 (verbatim):

    1791511650-rove-task-2 seat 0, record to 21:10: frames 163 (decayed 5, finished 150, killed 6, unfinished at end 2), metal in unfinished frames 283, build orders never started 27

`run/floor.py` gives `aband` 5, which equals frames.py's decayed count.

**Two of those five were kills, not decays.** By the record's `dmg` rows, two of the five "decayed" frames were shot to death by raiders we could see:
- armmex_14554 (8:46): a corgator, which the `xi` row also names.
- armllt_1240 (13:32): a corraid, then corlevlrs.

Their `destroyed` events name no attacker. So frames.py and `aband` count them as decayed, and the report told the player "abandoned an unfinished ... (9% / 14% built)". The tables below use the fate as I read it: decayed 3, killed 8, at end 2.

## Tally (13 frames, 283 metal)

| cause | n | metal |
|---|---|---|
| U4 the builder died or fled, or the frame was killed | 10 | 172 |
| U8 game end | 2 | 111 |
| U9 harness fault | 1 | 0 |
| U0, U1, U2, U3, U5, U6, U7 | 0 | 0 |

Two of the U4 frames also carry U1: armllt_2190 and armmex_8605, where the pick's walk home was a retreat under fire.

| fate (as read) | n | metal | frames.py's count |
|---|---|---|---|
| decayed | 3 | 37 | 5 (139 metal) |
| killed | 8 | 135 | 6 (33 metal) |
| at end | 2 | 111 | 2 (111 metal) |

| type | n | metal |
|---|---|---|
| armllt (light turret) | 6 | 198 |
| armmex (extractor) | 6 | 85 |
| armsolar (solar) | 1 | 0 |

None of the unfinished frames was a plant, nano, advanced solar, moho, fusion or silo. All 12 plants, nanos and advanced solars started in this game finished.

**What the player knew:**
- 7 frames were in a report before the cause acted: armllt_600, armmex_14554, armllt_1240, armmex_8605, armmex_1360 and the two U8 frames.
- 6 were reported only after the loss.
- None was never reported.
- The player dropped none itself.

## Items, ranked by metal put in

1. **20:37, armllt_23923 at E4 (spot_32), constructor_5722: U8.** 88% built at the end and still being built. The 21:01 queue-ahead step was queued behind it and did not pull the builder off. The 21:07 report listed it at 81%. Fix: none. Cost: small (75).
2. **13:05, armllt_1240 at D4 (spot_33), constructor_11458: U4, killed (frames.py says decayed).**
   - The list's turret at a forward spot reached 75% by 13:27, with its builder standing on it the whole time.
   - From 13:28, party_36 (a corraid) and then party_37 (4 corlevlr and 2 corraid) shot it: dmg 97, 100, 100, 200, 100. It died at 13:32, destroyed by None.
   - The player knew beforehand. The 13:24 report said "building a Sentry (armllt) at spot_33 (D4) (64 % built) ... enemies_near: party_36 (1 corraid) 562 away", and the pick's group_L1 attack was "about 10 s of walking away".
   - Afterwards the report misnamed the kill: "abandoned an unfinished armllt (14% built)".
   - Fix: none (the army was gathering 2,300 away). Cost: small (64). Confidence: high.
3. **8:27, armmex_14554 at D5 (spot_47), constructor_27504: U4, killed (frames.py says decayed).**
   - The frame reached 76% at 8:43. A visible corgator then shot it (`xi` corgator to armmex: 48, 95, 20) and it died at 8:46. The builder never moved.
   - group_M's hunt of the corgator was 1,100 behind. The leash ended it at 8:44 and it was re-sent at 8:45.
   - The player knew beforehand. The 8:41 report showed "enemy in sight: 1 at D5 (3155, 3680) ... corgator" and the frame at 66%, and that turn ordered only a signal.
   - The 8:51 report then said "abandoned an unfinished armmex (9% built)".
   - Fix: player. Cost: small (38). Confidence: high.
4. **20:58, armllt_28804 at F5 (spot_41), constructor_26737: U8.** 42% built at the end. The 21:07 report listed it at 31%. Fix: none. Cost: small (36).
5. **14:27, armmex_8605 at C3 (spot_24), constructor_27504: U4, also U1, decayed.**
   - The builder was hit twice by party_40/45 (corlevlr, corraid, corwolv) and fell to 57% health. At 14:37 the pick walked it home, 3,840 away: "PICK ... took w2 of 1 at 0.56", go_home=0.56.
   - No enemy was in its picture from 14:39. Even so, its menus from 14:39 to 14:50 offered no move back to its own 55% frame: only turrets "at spot_24 (... our extractor (being built))".
   - The frame decayed from 14:48 and was gone at 15:02.
   - The player knew beforehand. The 14:30 report had the frame at 15% with the raid 697 away, and the 14:38 report said the builder was "leaving its course (builds a Metal Extractor (armmex) at spot_24 (C3))".
   - Fix: hands-rule. Cost: small (28). Confidence: high.
6. **7:13, armllt_600 at D5 (spot_47), constructor_27504: U4, killed.**
   - A second corgator (party_8) appeared at 7:18 and killed the turret at 16% at 7:19. group_I was hunting party_1 at the time.
   - The player knew beforehand: the 7:17 report said "(12 % built) ... party_1 (1 corgator) 760 away".
   - Fix: none. Cost: small (14).
7. **15:40, armmex_1360 at E5 (spot_49), constructor_21956: U4, killed.**
   - Two corgators stood 200-390 from the frame from 15:41, and it died at 22% at 15:46.
   - The player knew beforehand. The 15:42 report said "(7 % built) ... party_52 (2 corgator) 340 away", and that turn sent group_C at his commander. The hunts of party_52 started at 15:50, after the kill.
   - Fix: player. Cost: small (11).
8. **3:13, armllt_2190 at C7 (spot_69), constructor_10476: U4, also U1, decayed.**
   - A corfav was shooting the builder the second the frame began. At 3:15 the pick played "walk to home; its list step 'armllt spot_69' waits for it", with go_home=0.55.
   - The frame decayed from 3:26 with no damage on it.
   - The packet's 2:40/2:50 raid answer (group_A fights it at home) was not played until 3:30.
   - The player learned of it after the cause acted: the 3:24 report said "leaving its course (builds a Sentry ...)", and the 3:34 report gave the abandoned line.
   - Fix: hands-rule. Cost: small (9).
9. **2:35, armmex_29787 at C8 (spot_69), constructor_10476: U4, killed.**
   - A corfav killed it at 8%, four seconds after it began.
   - The 2:30 report had said "party_2 (1 corfav) last seen 0:12 ago at spot_69", and the list still sent the builder there. group_A was roving in his half.
   - Fix: player. Cost: small (4).
10. **3:52, armmex_6635 at A7 (spot_62), commander: U4, killed.**
    - A corgator killed it one second in. The commander stood beside it: dgun_party_1 was rated 0.52 and not played, because the clause in force named only corfav and corak.
    - The player learned of it after the loss. Fix: none. Cost: small (4).
11. **5:29, armllt_30966 at C7 (spot_63), constructor_25985: U4, killed at 0%** by an arriving corgator (party_6). No metal was put in. Fix: none.
12. **19:01, armmex_30529 at D4 (spot_31), constructor_25985: U4, killed at 0%** by party_66's corgator. No metal was put in. Fix: none.
13. **8:16, armsolar_29164 at B7, commander: U9, decayed at 0%.** This was the blocked-site watchdog firing as the commander arrived.
    - Its clock started at 8:06, when the queued solar became the task with the commander standing 146 from the site. At 97f51e5 the loop skips moving units instead of resetting them, so the clock kept running through the 8-second walk.
    - At 8:15 it reported "never started in the 9 s it stood at the site", although the 8:14 report had shown the commander "walking ... 208 to go". It then sent a stop.
    - The engine created the frame at 8:16. The list placed the solar again 96 east (built 8:19-8:29), and the 0% frame decayed.
    - This has been fixed since in de7dc80. Fix: hands-rule. Cost: none. Confidence: high.

## Build orders never started

frames.py lists 27. Eight of them are not items:
- Five were started at a site the host placed more than 300 from the order's point: the 7:25 armvp (656 away), 11:16 armllt, 12:08 armvp (392 away), 13:14 armsolar and 18:53 armsolar. All of them finished.
- Two were long walks that then started at the site: the 11:17 armllt and the 18:32 armmex.
- One was re-sent and then started: the 5:30 armmex at spot_52.

All 19 real items are classified in the JSON (U3 6, U6 6, U4 3, U9 2, U8 2). The cap of fifteen did not have to drop any.
- **14:29, armsolar at solar_field, constructor_10476: U6, stuck.** move_failed from 15:12, and every report from 15:14 said "stuck: it cannot move from solar_field (B7)". It was never re-tasked, which left a constructor idle for about 6 minutes.
- **3:17 spot_76 (constructor_18284) and 5:31 spot_59 (constructor_27504): U9, also U3.**
  - In both, a new queue began with the builder's unstarted order. That step was "taken as done", and a second later the next step replaced the order, because a list newer than the order stays "ready".
  - spot_76: the builder was turned 1,800 back to A7 while 460 from the spot.
  - spot_59: a turret went up there with no extractor under it.
  - Both extractors came about 125-135 s late. This has been fixed since in e9f5175 (`list_caught_up`).
- **Rejected because two builders were sent to one spot: U6.**
  - 5:39 spot_71 (also U3): two of the player's lists named it.
  - 6:14 spot_45 (also U3): two of the player's lists named it; constructor_11096 walked 48 s for nothing.
  - 14:28 spot_44 (also U1): a pick sent constructor_26962 to it while constructor_21956's list was walking there for 58 s.
  - spot_28 also had four builders sent toward it by picks and lists between 10:05 and 11:11.
- **17:44 and 19:20, spot_21, constructor_11458: U6, also U4.** His cormex was rebuilt there during our 78-second walk, and raids sent the builder home twice. At 19:53 the list skipped the step: "spot_21 holds his extractor".
- **U3, the player re-listed (all deliberate or tool semantics):**
  - 3:06 spot_62: given to the commander.
  - 5:57 armllt: displaced by the 6:02 list.
  - 6:17 solar: replaced by an advanced solar, 782 short of the site.
  - 7:50 spot_28: the 7:45 list began with a bare armadvsol, which sent the builder home first.
  - 10:05 and 11:11 spot_28: given to constructor_27504.
- **U4:**
  - 2:40 spot_61 and 12:22 spot_46: retreats under raids, then a re-list.
  - 20:34 spot_13: the builder was killed on the way.
- **U8:** 20:35 spot_40 and 21:01 spot_27.

## The three mechanisms

1. **An armed answer that reaches a forward frame under a raider before the frame dies.** This covers armllt_1240, armmex_14554, armllt_600 and armmex_1360: 127 metal, the bulk of the 172 lost to U4.
   - Each time, the report showed the raider within 340-760 of the standing frame.
   - The answers did not cover them: one detachment was 1,100 behind and ended by the leash, one attack group was 10 s away, the hunts came after the kill, and twice the player's turn ordered nothing for the frame.
2. **Taken as done for an unstarted order, then displaced by the next step.** This covers the never-started 3:17 spot_76 and 5:31 spot_59.
   - It cost no metal in frames, but two extractors came about 260 extractor-seconds late in total. At the report's 3.0 a second per extractor, that is on the order of 750 metal of income (an estimate).
   - It has been fixed since in e9f5175.
3. **A way back to the builder's own unfinished frame once the threat is gone.** This covers armmex_8605 and armllt_2190 (37 metal), and the lost orders 2:40 spot_61 and 12:22 spot_46.
   - It is still live at HEAD. `menu.rs` offers `repair_` only for units that are not being built, and `help_<builder>` only for a frame that another builder's started task is on. A frame nobody is on is on no builder's menu.

Also worth knowing:
- The stuck builder at 14:29 cost about 6 minutes of a constructor. The impedance review's item 13 already asks for a line that surfaces it.
- The "abandoned" line and `aband` count a kill by an out-of-sight attacker as an abandonment. Here that was 2 of 5.

## Commands run

- `cat run/matches/1791511650-rove-task-2/results.jsonl`; `python3 -c "...batch.json"`
- `run/frames.py run/matches/1791511650-rove-task-2/00 --seat 0` (also with `--json` into scratch)
- `run/floor.py run/matches/1791511650-rove-task-2/00`
- `run/hands_window.py MATCH` for these windows:
  - 2:30-2:42 constructor_10476
  - 3:08-3:32 constructor_10476
  - 3:44-3:56 commander
  - 5:20-5:34 constructor_25985
  - 7:05-7:21 constructor_27504
  - 8:39-8:47 group_M, group_G, constructor_27504
  - 13:15-13:37 constructor_11458
  - 14:29-14:45 constructor_27504
- `run/impedance_sweep.py MATCH --seat 0 --all` for 2:00-4:10, 5:25-7:10, 5:50-6:12, 7:40-8:06, 8:30-8:52, 9:55-11:30, 12:20-13:16 and 17:30-20:00.
- Scratch scripts in the session scratchpad:
  - `turns.py`: report lines by turn
  - `rec.py`: record ev, cmd, d and s rows for given units
  - `unst.py`: each never-started order's next 100 s
  - `make_json.py`
- A damage and enemy scan of every unfinished frame from the record's `s` rows (`dmg`, `en`, `xi`).
- The truth file for his extractor at spot_21.
- `jevlog.lines` for constructor_27504's menus from 14:37 to 14:50.
- `git show 97f51e5:` of `pianist/mod.rs` (the watchdog) and `pianist/lists.rs` (taken as done).
- `git log -S list_caught_up` and the commit messages of de7dc80 and e9f5175.
- HEAD `menu.rs`, read for the repair and help moves.

Seat symlinks were not needed (one seat). Nothing was torn or unread.
