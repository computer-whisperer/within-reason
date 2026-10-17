# Unfinished structures: rove-move-1 (2026-10-07)

**The game.** An arena duel on Comet Catcher Remake 1.8 at commit a4c57ed. We won at 23:52 against BARb hard_aggressive. We played one seat:
- The player was Opus 5.5 at medium effort, with no commander.
- The hands' layers were news, same, fuse, openers, split, tick, places, again and near.
- There were 4 session replacements, at 5:15, 12:27, 18:48 and 19:58.

**frames.py, seat 0 (verbatim):**

```
1791420311-rove-move-1 seat 0, record to 23:52: frames 217 (decayed 10, finished 200, killed 5, unfinished at end 2), metal in unfinished frames 1681, build orders never started 35
```

`run/floor.py` shows `aband` 10, which equals the decayed count.

frames.py changed while I worked (11:29, PLACED_RADIUS 700). The run at the start of the review printed "build orders never started 49"; the line above is the current tool. The frames part is identical in both.

## Tally (17 frames, 1681 metal)

| cause | n | metal |
|---|---|---|
| U3 the player re-listed or stopped the builder | 5 | 1220 |
| U9 harness fault | 2 | 208 |
| U4 killed, or the builder died | 8 | 177 |
| U8 game end | 2 | 76 |
| U0, U1, U2, U5, U6, U7 | 0 | 0 |

The three advanced solars (914 metal) are counted as U3 with U9 also: the player stopped builders that the report had called idle. The moho (242 metal) is counted as U3 with U2 also.

| fate | n | metal |
|---|---|---|
| decayed | 10 | 1535 |
| killed | 5 | 70 |
| unfinished at end | 2 | 76 |

Two of the "decayed" frames were in fact shot down by unseen enemies (armmex_30948 and armllt_10067).

| type | n | metal |
|---|---|---|
| armadvsol | 3 | 914 |
| armmoho | 1 | 242 |
| armavp | 2 | 208 |
| armllt | 6 | 172 |
| armmex | 4 | 81 |
| armsolar | 1 | 64 |

**What the player knew.**
- Before the cause acted: 11 frames.
- Only afterwards, from the fights or abandoned line: 5 frames.
- Never: 1 frame.
- Dropped by its own order: 7 frames. These are the five U3 items plus the two armavp frames. It knew of four of them; it did not know the three advanced solars were still standing.
- The fights line said "(0% built)" for each advanced solar. Those frames had reached 81-97%.

## Items, ranked by metal

1. **armadvsol_27527, B7, 10:53 to 14:30: 340 metal (97%), constructor_15017. U3, also U9. Large.**
   - What happened: armadvsol_8893 belonged to constructor_10588 and stood 107 away. When it finished at 11:28, the hands' finished event also ended 15017's build task. That event clears every started build of the same type within 250 of the finished unit (`pianist/mod.rs` Event::UnitFinished, unchanged at HEAD).
   - What the report said: at 11:26 the entry read "building ... (97 % built)", which was 8893's share. From 11:32 to 12:17 it read "finishing an order".
   - The player's stop: its 12:17 `queue` began with "stop" (it said the builders "had sat stuck for minutes, so I stopped them"). The stop landed at 12:25.
   - Player knew: no. Fix class: hands-rule. Confidence: high.
2. **armadvsol_26769, B7, 9:05 to 12:20: 290 metal (83%), constructor_29164. U3, also U9. Large.**
   - What happened: armadvsol_17727 (constructor_15017's frame, 96 away) finished at 9:22 and ended 29164's build task the same way. The entry read "finishing an order" from 9:27 to 10:26, and its 9:37 and 9:56 lists played nothing.
   - The player's stop: the 10:26 `queue` ["stop", "armavp avp_yard #avp1", ...], with the reason "had sat 'finishing an order' for nearly two minutes". It landed at 10:31 with the frame at 82%.
   - Player knew: no. Afterwards: "12:27 abandoned an unfinished armadvsol (0% built)". Fix class: hands-rule. Confidence: high.
3. **armadvsol_10240, B7, 11:05 to 14:11: 284 metal (81%), constructor_10859. U3, also U9. Large.**
   - The same mechanism: the same finished frame 8893, 181 away. The same 12:17 stop, which landed at 12:25.
   - The entry read "97 % built" at 11:26 and "finishing an order" from 11:32 to 12:17.
   - Player knew: no. Fix class: hands-rule. Confidence: high.
4. **armmoho_28891, B8 (spot_73), 17:03 to 19:05: 242 metal (39%), constructor_16111. U3, also U2. Large.**
   - The drop: the 17:19 report said the moho was 25% built and that its builder stood in plant_27255's exit lane. The `queue` ["stop", "armmoho spot_68", ...] dropped it on purpose ("I dropped that build, which was only 25% done"). It landed at 17:27.
   - The attempted recovery: the 17:49 list for constructor_19709 began with "armmoho spot_73". At 17:57 it played "help finish the armmoho already under way at spot_73". At 17:58 it said "nothing to do ... at spot_67", and at 17:59 it played "build a armmoho at spot_69".
   - Why the recovery failed: the list counts a repair task as a filler and takes the next step at once. The helper was still 1,000 away and never arrived.
   - Player knew: yes. Fix class: hands-rule. Confidence: high.
5. **armavp_17970, A8, 15:18 to 15:57: 104 metal (4%), constructor_15017. U9, also U3. Large (a plant frame).**
   - What happened: "armavp #avp2" on three lists made three frames at three places at 15:13, the same-tick id race (fixed later in 4ed68d8).
   - The player's attempt: at 15:22 it ordered `remove` to reclaim the two extra frames. The same turn's `queue` opened with "stop" and cancelled the reclaim: "15:29 takes plant_17970 apart next" was followed by "15:30 stopped what it was doing on your `stop`".
   - Player knew: yes (its 15:22 note). Fix class: tool. Confidence: high.
6. **armavp_6420, B8 (fus_site2), 15:19 to 15:55: 104 metal (4%), constructor_11572. U9, also U3. Large.**
   - The second extra #avp2 frame, with the same cancelled reclaim. It decayed with "15:55 abandoned an unfinished armavp at B8 (0% built)".
   - Fix class: tool. Confidence: high.
7. **armsolar_25985, A7, 2:32 to 3:07: 64 metal (41%), commander. U3. Small.**
   - What happened: the 2:35 `queue` opened with "stop" and sent the commander to take spot_62 (energy 1339 of 1400, metal 0). The player saw the solar at 10%. With the 7 s think penalty, the stop landed at 2:43 with the solar at 41%.
   - Player knew: yes, and it meant to drop it. Fix class: player. Confidence: high.
8. **armllt_10067, E3, 23:15 to 23:33: 55 metal (65%), constructor_29524. U4. Small.**
   - What happened: the frame went from 65% to destroyed in one second while its builder stood on it. An unseen corban was 556-589 away (truth file).
   - The hands' line was wrong: "23:33 abandoned an unfinished armllt at E3 (65% built): its builder was sent elsewhere and the frame decayed".
   - Player knew: the frame yes (62% at 23:32), the threat no. Fix class: none. Confidence: medium.
9. **armmex_30948, C3 (spot_24), 10:57 to 11:14: 32 metal (64%), constructor_12458. U4. Small.**
   - What happened: an unseen corgator 217-291 away shot it down. It fell 64, 52, 39, 26 and 13% a second apart and was then destroyed with no attacker recorded.
   - The fights line misnamed the kill: "abandoned an unfinished armmex (13% built)".
   - Player knew: the frame yes (64% at 11:10), the raider no. Fix class: none. Confidence: medium.
10. **armmex_7267, E4, 12:52 to 13:04: 23 metal (46%), constructor_18284. U4. Small.**
    - What happened: Jev picked this extractor in his half (12:45, rated 0.54). A corgator came from the north unseen and killed it.
    - Player knew: the frame yes (41% at 13:01), the threat no. Fix class: none. Confidence: high.
11. **armllt_15344, F5, 18:40 to 18:51: 20 metal (24%), constructor_3560. U4. Small.**
    - What happened: the builder went from 98% to 26% health in the second the frame began and died at 18:46 to an unseen shooter (a corban 610-660 away). The frame then decayed.
    - Player knew: no. Afterwards: "18:58 abandoned an unfinished armllt (24% built)". Fix class: none. Confidence: medium.
12. **armmex_14842, D4, 17:14 to 17:22: 16 metal (31%), constructor_18284. U4. Small.**
    - What happened: its list walked it to spot_31 next to a correap that was in sight. It was hit every second from 17:11. Its menu read change 0.81-0.89 and go_home 0.29-0.46, but the pick never played it. It died at 17:20 and the frame went at 17:22.
    - Player knew: yes. The 17:19 entry said "near death (19%) ... party_40 (1 correap) 343 away". Fix class: hands-rule. Confidence: high.
13. **armllt_5915, B3 (spot_19), 16:46 to 16:54: 13 metal (15%), constructor_8763. U4. Small.**
    - What happened: its list step began the turret beside a corgator that had been killing our extractor there since 16:30. The builder was hit every second (change 0.66-0.83) and never moved.
    - Player knew: yes (the 16:30 and 16:47 reports). Fix class: hands-rule. Confidence: high.
14. **armmex_12375, C1 (spot_0), 17:38 to 17:43: 10 metal (20%), constructor_29158. U4. Small.**
    - What happened: its list step went to a spot his cormex held, 148 from his corllt. It was hit every second from 17:31 to 17:42 (change 0.65-0.84) and never played. It died at 17:42.
    - Player knew: yes (17:36 "under_fire"). Its 17:42 re-list came too late. Fix class: hands-rule. Confidence: high.
15. **armllt_10320, D3, 11:18 to 11:24: 8 metal (9%), constructor_12458. U4, also U1. Small.**
    - What happened: the turret step queued at 11:10 began 19 from the corgator that had just shot the extractor down. The pick walked the builder home at 11:20 ("walk to home; its list step 'armllt spot_24' waits for it"). The corgator killed the frame.
    - Player knew: the threat yes, the frame no. Fix class: hands-rule. Confidence: high.
16. **armllt_25534, B7, 23:28: 75 metal (88% at the end), constructor_4247.** U8: the frame began 24 s before the game ended.
17. **armllt_12813, C7, 23:51: 1 metal, constructor_29164.** U8: Jev's pick one second before the end.

## Orders never started (15 of 35, by cost)

- 13:16 constructor_19709 **armfus** at eco_field (A8). U6, rejected: "13:41 the engine refused a armfus at A8: the site was bad". eco_field was full (four more refusals there from 13:39 to 13:42). The fusion was created at fus_site at 15:07. Small cost (an energy stall).
- 9:23 constructor_15017 armvp #vp3. U6: the frame was placed 739 from the ordered point (23638, finished 10:29). No cost.
- Seven armadvsol orders were refused at full marks. U6, no cost:
  - eco_field: 10:52 constructor_10588 (refused 11:31 and 11:33); 12:33 constructor_11572 and constructor_4247 (refused 13:39).
  - solar_west: 13:10 constructor_10588 (refused 13:58).
  - fus_site2: 15:31 constructor_15017 (refused 16:30); 16:07 constructor_27504 (refused 17:35).
  - home: 10:31 constructor_10859 armnanotc (refused 10:54).
- 8:33 constructor_15017 armadvsol, queued behind its build. U3: "dropped the step 'armadvsol home' its old list had queued ... you have replaced or cancelled that list since". The 9:07 queue replaced it.
- Jev's picks replaced by the player's lists as they landed (U3): 6:52 constructor_21029 armnanotc (one second later); 6:44 constructor_15017 armsolar (stopped 6:53); 14:53 constructor_15017 armsolar (replaced by #avp2 at 15:13).
- Jev's picks of a Sentry at spot_73 in the crowded base (U6, refused): 18:45 constructor_29164 (refused 19:28); 20:36 constructor_4247 (refused 22:30).

The other 20 never-started orders are extractor and turret steps that the builder's next list step or pick replaced within seconds, plus two more refusals (8:51 armllt at C4, 9:11 armmex at D4).

## Three mechanisms that would have kept the most metal

1. **The finished event is keyed on type and distance, not on the frame (914 metal: items 1-3).** Code: `pianist/mod.rs` Event::UnitFinished, `def == done.def && near.dist2d(done.pos) < 250.0`, unchanged at HEAD.
   - When one advanced solar finished, it ended the build tasks of its neighbours still on their own frames.
   - Their entries showed the neighbour's percentage and then "finishing an order". Their lists stalled (a builder with no task that is not idle takes no step).
   - The player stopped them as stuck at 81-97%.
   - Matching the finished event to the frame id from the builder's `created` event fixes the entry, the list and the stop.
2. **The id race plus a `stop` that cancels the same turn's `remove` (208 metal: items 5 and 6).** The race is fixed (4ed68d8). A queue's leading `stop` still overrides a reclaim that a `remove` in the same turn has just started; that is a question of tool semantics.
3. **A list's help-finish step is overrun by its own next step (242 metal: item 4).** Code: `lists.rs` `Some(_) => true`. A repair task counts as a filler. Only reclaim steps were exempted later (5046f71).

Smaller patterns:
- Four U4 items (47 metal) share one shape: a builder hit every second for 8-11 s, its change rated 0.65-0.89, was never played (items 12-15). List steps began next to a raider or turret already in sight.
- The abandoned line treats every destruction with no attacker as a decay. It told two kills by unseen shooters as abandonments (items 8 and 9, 87 metal), so the player never heard of the corban or the corgator.

## Commands run (from the repository root; M = run/matches/1791420311-rove-move-1/00)

```
cat M/../results.jsonl; python3 -c "...json.load(open('M/../batch.json'))"
run/frames.py M --seat 0 [--json]   (twice: before and after the tool's 11:29 change)
run/floor.py M
run/commander_turns.py M --calls ; run/impedance_sweep.py M --seat 0
run/impedance_sweep.py M --seat 0 --all --from/--to  15:00-16:00, 13:05-13:52, 6:38-6:58, 8:30-9:25, 10:28-11:45,
   15:29-16:35, 16:05-17:38, 18:44-19:40, 20:35-22:35, 8:08-8:53
run/hands_window.py M 16:45 16:56 constructor_8763 ; 17:08 17:21 constructor_18284 ; 17:30 17:43 constructor_29158
scratch scripts (scratchpad .../unfinished/1791420311-rove-move-1/bin): turn.py (a turn's prompt by clock, grep),
   think.py + say.py (the player's text and calls of a turn), rq.py (record cmd/d/ev rows and samples for units),
   near.py (truth-file enemies within a radius of a point), unst.py (never-started orders with cost, later frames, d rows)
git show a4c57ed:crates/bot/src/brain/pianist/{picture.rs,mod.rs,execute.rs,lists.rs}, briefing.rs abandoned();
   HEAD crates/bot/src/brain/pianist/{mod.rs,lists.rs}
```

Seat 0 only, so I needed no symlink directories. Nothing was unreadable.
