# Unfinished structures: apex-v024-carrot-sym50-opus-1

We won 16:40 against Apex v0.2.4 (standard profile) on Carrot Mountains v2.0, with +50 income on both sides and seed 3. One seat (strategist-0, Opus 5.5 medium) played under an Opus 5.5 high commander, with twelve hand layers, in an arena game.

frames.py, seat 0, first line verbatim:

    1791989571-apex-v024-carrot-sym50-opus-1 seat 0, record to 16:39: frames 210 (decayed 3, finished 196, unfinished at end 11), metal in unfinished frames 5448, build orders never started 25

floor.py: `aband` 3 equals the decayed count. Its `aband_m` reads 6 against frames.py's 569 because the two measure different shares. floor takes each frame's last sampled share before destruction, which is near 0 after a decay. frames.py takes the peak share.

## Tally

Only 3 frames were lost (569 metal). The other 11 (4,879 metal) were still rising when the record ended; that includes the 76% fusion.

| Cause | n | metal in |
|---|---|---|
| U3 the player re-listed | 2 | 247 |
| U8 game end | 11 | 4879 |
| U9 harness fault | 1 | 322 |
| U0, U1, U2, U4, U5, U6, U7 | 0 | 0 |

| Fate | n | metal in |
|---|---|---|
| decayed | 3 | 569 |
| killed | 0 | 0 |
| unfinished at end | 11 | 4879 |

| Type | n | metal in |
|---|---|---|
| armfus | 1 | 2546 (at end) |
| armmoho | 5 | 1884 (all at end) |
| armnanotc | 5 | 529 (159 decayed, 370 at end) |
| armadvsol | 1 | 322 (decayed) |
| armvp | 1 | 88 (decayed) |
| armllt | 1 | 79 (at end) |

**What the player knew.**
- Before the cause acted: 14 frames, which is all of them.
- After, from the abandoned line alone: 0.
- Never told: 0.
- Dropped by its own order: 2.

The abandoned lines it got all read "(0% built)", so it was never told the metal it lost.

**The fusion counts as U8 although it started 2:22 before the end.** That is 22 s outside the window. No other class fits:
- Its single advanced constructor stood on it at the full 0.53%/s.
- No store fell below 0.83.
- It was due about 30 s after the win.

## Items, ranked by cost

1. **11:26-14:48, seat 0, armadvsol_19432 (B5, 1904,7216), constructor_15311: U9, 92%, 322 metal.**
   - What happened:
     - 12:21: the list queued #as10 behind the frame at 60%.
     - 12:26: armadvsol_6103 finished 229 away. The finish handler (same type within 250) took it as 15311's build and promoted #as10 to its task while 19432 stood at 71%.
     - 12:49: the watchdog said "its armadvsol at B5 never started in the 8 s it stood at the site: the site is blocked ..., refused for 90 s" and stopped it with 19432 at 86%. It looked for a same-type frame only within 200 of #as10's site (1808,7024), and 19432 was 215 away. The bot.log line: "armadvsol order for unit 15311 at (1808, 7024) blocked at the site".
   - Evidence: hands_window shows no pick for 15311.
   - The player knew: the 12:26 entry said "building ... (66 % built)". From 12:34 it said "walking to build ... 277 to go". At 14:50: "abandoned an unfinished armadvsol (0% built)".
   - Fix: hands-rule. Cost: large. Confidence: high.
2. **9:27-11:12, seat 0, armnanotc_372 (B6, 2968,7912), constructor_10329: U3, 69%, 159 metal.**
   - What happened: the 9:59 queue gave constructor_10329 `["stop", "extractor spot_87", ...]` with the result "its build in progress dropped". The stop landed at 10:07 with the frame at 69%.
   - The player knew: the report said "(54 % built) ... standing in plant_15091's exit lane since 32 s ago". The player wrote: "I dropped the construction turret that was blocking plant_15091's exit ... The turret's frame was 54% built, so that metal is lost."
   - The report named the constructor as the blocker, not the frame. Letting it finish (about 45 s) or moving only the builder would have cleared the lane.
   - Fix: player. Cost: large (a nano frame). Confidence: high.
3. **9:23-9:55, seat 0, armvp_22187 (C5, 3454,7232), constructor_3778: U3, 15%, 88 metal.**
   - What happened: the commander asked for an advanced vehicle plant, and the player replaced the fifth T1 plant with it. Its 9:24 queue gave 3778 `["stop", "armavp #avp1", ...]` and moved the helper 23346 onto #avp1 in the same call. It landed at 9:31-9:33 with the frame at 15%.
   - The player knew: the report said "(1 % built)". The player wrote: "The plain plant frame was only 1% built, so dropping it lost almost nothing."
   - Fix: player. Cost: large (a plant frame, 88 metal). Confidence: high.
4. **14:17, armfus_611 (B5), constructor_5185: U8, 76%, 2546 metal.** It was 55% at 16:00 and 76% at 16:39 at a steady rate, with the 16:09 queue-ahead moho behind it. The 16:36 report said "(75 % built)". Cost none, confidence high.
5. **15:57, armmoho_19801 (A5), constructor_7176: U8, 81%, 502 metal.** It rose to 16:39 with the 16:29 queue-ahead order behind it. Cost none.
6. **15:58, armmoho_4594 (A5), constructor_22492: U8, 80%, 496 metal.** It rose to 16:39. Cost none.
7. **16:10, armmoho_30861 (C6), constructor_663: U8, 57%, 353 metal.** Cost none.
8. **16:14, armmoho_20430 (A6), constructor_22934, helped by constructor_16338: U8, 47%, 291 metal.** Cost none.
9. **16:26, armmoho_31058 (C5), constructor_29628: U8, 39%, 242 metal.** Cost none.
10. **16:00, armnanotc_17780 (B6), constructor_3778: U8, 65%, 150 metal.** armnanotc_24575 finished 244 away at 16:20, and the same 250-elmo match ended 3778's task. Its entry read "finishing an order" from 16:26 while it kept building. Cost none.
11. **16:11, armnanotc_21784 (B5), constructor_3402: U8, 47%, 108 metal.** Cost none.
12. **16:18, armnanotc_1289 (B5), constructor_10329: U8, 35%, 80 metal.** Cost none.
13. **16:14, armllt_19157 (D6), constructor_28932: U8, 93%, 79 metal.** Cost none.
14. **16:30, armnanotc_18930 (A6), constructor_15311: U8, 14%, 32 metal.** The 16:36 entry gave it "(60 % built)", which is armnanotc_17780's share. Cost none.

## Orders never started (25)

| Class | n |
|---|---|
| U6 | 7 |
| U3 | 4 |
| U2 | 3 |
| U4 | 1 |
| U8 | 6 |
| skipped | 4 |

- **U6, rejected (the spot already held ours), 3 orders.** All three came from free_spots, which does not count a spot as taken while the claiming builder's order has not started and that builder has a list.
  - 5:08 constructor_2337, spot_110 (C6): 29599's queued order took it the same second.
  - 10:37 constructor_18918, spot_41 (A2): 4020's list took it at 12:13; 18918's walk of 1:53 was wasted.
  - 10:45 constructor_11809, spot_83 (E5): the 11:28 pick sent 11094, which started at 11:46; 11809 arrived to a refusal at 11:52.
- **U6, the site taken by a sibling order, 2 orders.**
  - 11:15 constructor_3778, armnanotc: it and 19434 got the same step on the same tick and both were placed at (2216,7112). 3778 probably assisted; the nano rose in 31 s.
  - 15:03 constructor_4373, armadvsol: it and 10442 were both placed at (2592,7088). 4373 stood "151 to go" for 50 s.
  - Each case gave one building where the lists asked for two.
- **U6, stuck in the packed B5 yard, 2 orders.**
  - 14:56 constructor_25112 and 14:56 constructor_10329, both "armadvsol home". The player had sent them there to clear plant exit lanes.
  - Each held within 150 elmos for 70-100 s, was never idle, and had no move_failed event.
  - The entries said "walking to build ... 756 to go" at every turn. 10329 was re-listed by the player at 16:05; 25112 stood until the end. Confidence medium.
- **U3, the player re-listed, 4 orders.**
  - 6:02 constructor_29599: a "stop", solars first during the energy stall.
  - 6:16 constructor_11094 and 6:28 constructor_19434: the new lists replaced the old ones.
  - 10:03 constructor_10329: the queued second nano, dropped with item 2.
- **U2, also U9, taken as done on an order not begun, 3 orders.** This was fixed after this game in e9f5175.
  - 6:57 constructor_10090, armllt spot_87.
  - 7:19 constructor_23346, armmex spot_87: spot_87 had no extractor from 6:22 to 11:00.
  - 15:08 constructor_22492, armmoho spot_91: it never got its moho.
  - Cost: small each.
- **U4, 1 order.** 13:41 constructor_11094, armllt spot_61 (D3): a Pawn shot it on the way, and the pick walked it, then sent it home. It was killed at 14:17. The reports had said "armpw 1; killing our Construction Vehicle" since 13:59.
- **U8, 6 orders.** At 16:09, 16:15, 16:28, 16:29, 16:29 and 16:31, queue-ahead orders waited behind builds in progress, and their 90-s windows ran past the record's end.
- **Skipped, 4 orders.**
  - Three long walks whose frames started at the site: 7:48 constructor_21799 (started 9:26), 9:48 constructor_22732 (12:07) and 10:07 constructor_13423 (11:49).
  - 12:21 constructor_15311's #as10: re-sent at 12:51 and started at 13:25. It is item 1's mechanism.

## Three mechanisms

1. **The finished-build match and the blocked-site watchdog, 322 metal (item 1).**
   - The finish handler matches a builder's started build by type within 250 elmos.
   - The watchdog checks for the builder's own frame only within 200 of the queued order's site.
   - Keying the task to the frame's unit id, and skipping builders with a frame in progress, would have kept the 322.
   - The same match emptied 15311's task at 15:56 and 3778's at 16:20, with no loss before the end.
2. **The player's `stop` on started frames, 247 metal (items 2 and 3).**
   - The player saw 54% and 1%. When its orders landed the frames stood at 69% and 15%.
   - The nano was dropped to clear a lane that its builder, not the frame, stood in.
   - A queue result or entry giving the share at landing and the seconds to finish, or moving only the builder, would have kept both.
3. **No third mechanism lost frame metal.**
   - The report's abandoned line gives the share at destruction, "(0% built)", for all three lost frames (569 metal), so the player never learned what it lost. That line recovers nothing by itself.
   - The costliest never-started pattern was the list's "taken as done" on an order not begun (6:57, 7:19, 15:08; fixed in e9f5175). spot_87 stood without an extractor or turret from 7:30 to 11:00, and spot_91 never got its moho.

## Commands run

- `cat MATCH/../results.jsonl`; the batch.json print
- `run/frames.py MATCH --seat 0` (also with `--json`)
- `run/floor.py MATCH`
- `run/impedance_sweep.py MATCH --seat 0 --from F --to T --all` for the windows 4:50-5:30, 5:50-7:40, 9:00-9:40, 9:40-10:15, 11:00-11:16, 11:40-12:32, 12:20-13:00, 13:30-14:20, 14:40-16:39 and 15:00-15:20
- `run/hands_window.py MATCH 12:25 12:28 constructor_15311` and `run/hands_window.py MATCH 12:47 12:51 constructor_15311`
- Scratch readers of `record-0.jsonl` (the events, commands and `d` rows per builder, the share series) and `strategist-0.jsonl` (the turn prompts, the tool calls and results, the player's text), plus greps of `bot.log`, `engine.log` and `commander.jsonl`
- Read: `crates/bot/src/brain/pianist/mod.rs` (the UnitFinished match, `take_queued`/`promote`, the blocked-site watchdog), `lists.rs` (taken as done, at this game's commit 77f57de and now), `menu.rs` (`free_spots`); `git log -S "the step waits for the frame"`
- Seat 0 only, so no symlink directories were needed.
