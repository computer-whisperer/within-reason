# Unfinished structures: 1791594935-human-30 / 00

**The game.** A human game on Comet Catcher Remake 1.8, played 2026-10-09, 1v1 against [gecko]u6bkep (a person, lobby skill 30, Armada). We had one seat: an Opus 5.5 medium player under an Opus 5.5 high commander, with Jev 1.13.0 as the hands. **We won at 19:37**, when his commander died at (2941,2767). The record runs to 19:49. The person said nothing in chat.

**Denominator, seat 0, quoted from `run/frames.py`:**

> 1791594935-human-30 seat 0, record to 19:49: frames 152 (decayed 8, finished 138, unfinished at end 6), metal in unfinished frames 1448, build orders never started 35

`run/floor.py` prints `aband` 8, which equals frames.py's decayed count. Its `aband_m` is 15 against frames.py's 334 for the same 8 frames. floor multiplies by the last share it sampled, after the frame had decayed; frames.py uses the share the frame reached.

Of the 35 orders frames.py lists as never started, 24 are items. The other 11 are not:
- **Six false positives.** frames.py measures from the command's coordinates, not the `placed` site. The engine placed these 330-740 elmos away, and they started there.
- **Five id helpers.** They merged into frames that other builders began.

## Tally

**By cause.** These are frame items only; U8 is excluded from the fault total below.

| cause | n | metal in |
|---|---|---|
| U1: the pick sent the builder elsewhere | 4 | 62 |
| U3: the player re-listed the builder | 2 | 82 |
| U4: frame killed (unseen raid) | 1 | 38 |
| U9: harness fault | 1 | 152 |
| U8: game end | 6 | 1114 |
| U0, U2, U5, U6, U7 | 0 | 0 |
| **all** | **14** | **1448** |

The faults (U1, U3, U4, U9) come to 8 frames and 334 metal. All of it is metal in decayed frames; nothing was killed while it was seen.

**By fate.**

| fate | n | metal |
|---|---|---|
| decayed | 8 | 334 |
| killed | 0 | 0 |
| at end | 6 | 1114 |

One of the 8 decayed frames, armllt_17620, was in fact killed by an enemy out of sight. The record names no attacker, so it counts as decayed.

**By type.**

| type | n | metal |
|---|---|---|
| armsolar | 7 | 569 |
| armadvsol | 4 | 302 |
| armmoho | 1 | 304 |
| armfus | 1 | 235 |
| armllt | 1 | 38 |

Without the U8 frames, solars and advanced solars account for 7 of the 8 faults and 296 metal.

**Never-started orders (24 items):** U1 10, U3 6, U6 3, U4 1, U8 4. None of them lost metal.

**What the player knew.**
- **Before the cause acted:** 10 frames. Four are decayed frames; the other six are the U8 frames, all listed in the 19:49 report.
- **Only afterwards, from an abandoned line:** 4 frames.
- **Never told:** 0.
- **Dropped by its own order:** 2. One of them was a frame it had never been shown.

## Items, ranked by metal (faults first, then game end)

1. **6:27, armsolar_8043 (F2, 6072,920), constructor_3904, decayed 7:14 at 98%. 152 metal, large, U9, high.**
   - **What happened.** The frame was built by its list's solar step. At 61% (6:10) the next solar was queued behind it. At 6:18 the commander's solar 10447 finished 224 away. The hands' finished-unit handler (`pianist/mod.rs` UnitFinished: any started build of that type within 250) took that finish as constructor_3904's and promoted the queued step, while the engine kept building 8043 from 70% to 97%. The blocked-site watchdog's exemption covers only a same-type frame within 200 of the queued site, and 8043 stood 208 off. After 8 s it fired: "its armsolar at F2 never started in the 8 s it stood at the site ... refused for 90 s", then `stop 3904`, and the step went again elsewhere.
   - **What the player knew.** The 6:07 report said "building a Solar Collector (armsolar) at 431 from spot_6 (F2) (52 % built)". The 6:31 report showed the watchdog line and an entry reading "(98 % built)". The loss itself came only in the 7:20 fights tally: "abandoned an unfinished armsolar (0% built) x1".
   - **Fix:** hands-rule.

2. **7:16, armadvsol_23255 (G2), constructor_8007, decayed 7:46 at 18%. 63 metal, small, U3, high.**
   - **What happened.** The 6:52 pick began it "beside itself" while the constructor stood in plant_4070's exit lane. The 7:10 queue `["stop", "armsolar" x4]` dropped it: "7:16 constructor_8007: stopped what it was doing on your `stop`".
   - **What the player knew.** The 7:10 entry said "(14 % built) ... standing in plant_4070's exit lane since 17 s ago". The player wrote "That constructor has dropped that build". It dropped the frame on purpose.
   - **Fix:** player.

3. **10:03, armsolar_10959 (G2), constructor_31474, decayed 10:22 at 25%. 39 metal, small, U1, high.**
   - **What happened.** The pick played "build a armadvsol at 531 from passage_1 (G2)". The gate rated change 0.64 and the advsol 0.59; the pick took it at 0.32. The move's words: "leaves the Solar Collector (armsolar) at about 20%, which decays". The packet said "with the energy line STALLING, an armsolar at home; otherwise with energy under half, an armadvsol at home". The store went from 1 at 9:56 to 358 at 10:05, so the answer flipped.
   - **What the player knew.** Before, from the 9:56 entry "(1 % built)". After, from "10:22 abandoned an unfinished armsolar at G2 (1% built)".
   - **Fix:** hands-rule.

4. **9:24, armllt_17620 (D4, 3664,2448), constructor_23114, destroyed by null at 45%. 38 metal, small, U4, high.**
   - **What happened.** It was a forward turret, the 8:55 pick's spot_31. It took 197 and 195 damage from out of sight (the `dmg` rows at 9:24 and 9:25). party_25 (4 armstump) came into sight beside it at 9:26 and killed the builder at 9:29.
   - **What the player knew.** The 9:12 report showed the frame at 2% and "nothing of his, in sight now: ... D2-D4". Afterwards it got the 9:29 tally "abandoned an unfinished armllt (16% built)", a kill reported as an abandonment.
   - **Fix:** none.

5. **1:48, armsolar_14870 (G2), constructor_23943, decayed 2:02 at 12%. 19 metal, small, U3 (also U1), high.**
   - **What happened.** The pick began this solar at 1:45, in place of the advanced solar it had begun at 1:44. The player's 1:43 queue `["stop", "extractor spot_12", ...]` landed at 1:48 and dropped it.
   - **What the player knew.** It had read "walking to build a Advanced Solar Collector ... 134 to go" and wrote "I took the first constructor off an advanced solar". It never saw this frame. Afterwards it got the 2:13 tally.
   - **Fix:** player.

6. **9:16, armsolar_25530 (G1), constructor_540, decayed 9:28 at 8%. 12 metal, small, U1 (also U2), high.**
   - **What happened.** The pick played "build a armadvsol at home (G1)". The gate rated the advsol 0.58 and change 0.58; the pick took it at 0.2, with the words "leaves the Solar Collector (armsolar) at about 10%, which decays". The builder was on the menu because the player's 9:00 list of four "armsolar" steps had been "taken as done" (9:07-9:10) by this one walking build. The 9:12 re-list with ids landed only at 9:18.
   - **What the player knew.** Only afterwards, from the 9:28 abandoned line.
   - **Fix:** hands-rule.

7. **10:25, armadvsol_20119 (F1), constructor_31726, decayed 10:37 at 2%. 7 metal, small, U1, high.**
   - **What happened.** The pick played "build a armsolar at 337 from spot_11 (F1)" at 0.58, with the words "the safe generator from an empty store ... leaves the Advanced Solar Collector (armadvsol) at about 0%". It was this builder's fourth flip in 16 s.
   - **What the player knew.** Only afterwards, from the 10:37 abandoned line.
   - **Fix:** hands-rule.

8. **1:45, armadvsol_3273 (G2), constructor_23943, decayed 1:55 at 1%. 4 metal, small, U1 (also U3), high.**
   - **What happened.** The pick played "build a armsolar at 505 from spot_6 (G2)" at 0.42, with the words "leaves the Advanced Solar Collector (armadvsol) at about 0%, which decays". The player's stop three seconds later would have dropped it anyway.
   - **What the player knew.** Only afterwards, from the 2:13 tally.
   - **Fix:** hands-rule.

9. **Game end, U8, cost none.** Each frame below was begun from 18:59 on, still had its builder on it, was progressing at 19:49, and was in the 19:49 report.

   | frame | cell | builder | started | at end | metal in |
   |---|---|---|---|---|---|
   | armmoho_5430 | F2 | constructor_22167 | 19:16 | 49% | 304 |
   | armfus_8996 | H1 | constructor_5925 | 19:33 | 7% | 235 |
   | armadvsol_16458 | G1 | constructor_7444 | 18:59 | 65% | 228 |
   | armsolar_5828 | G1 | constructor_8007 | 19:21 | 95% | 147 |
   | armsolar_14615 | G1 | constructor_16526 | 19:22 | 90% | 140 |
   | armsolar_2881 | G1 | constructor_31726 | 19:37 | 39% | 60 |

## Never-started orders (24 items)

- **U1, 8 generator flips.** The pick swapped advsol and solar orders at spot_5 (G1) before the builder arrived, under the same energy sentence:
  - constructor_31474 at 9:04, 9:07, 9:08, 9:22 and 9:29, while walking 2,500 elmos from G5;
  - constructor_31726 at 10:09 and 10:11;
  - constructor_25385 at 10:26.
- **U1, 2 other picks.**
  - constructor_14058's forward turret at E4 (10:16): after a 39 s walk the pick turned it back to spot_13, with party_28 (1 armjanus, 6 armstump) about 1,100 off. Also U4.
  - constructor_3904's extractor at G6 (11:59): re-targeted to spot_37 three seconds later, at 0.12.
- **U3, 6 replaced by the player's new lists.**
  - The 2:13 list for 23114 (extractor at spot_17), the 4:31 list for 10909 (turret at spot_25), the 7:20 list for 14058 (spot_1), the 8:36 list for 8007 (turret at spot_51) and the 13:24 #hlt23 list for 14058 (turret at spot_6).
  - The 9:00 stop for 23943's nano, 2,188 away, which was deliberate: "I took constructor_23943 off its long walk".
- **U6, 3 extractors refused by the engine.** "the engine refused a armmex at F3 / F5 / E3: the site was bad", after 32, 49 and 17 s. Each turret step for those spots was then skipped.
- **U4, 1.** constructor_427, one of three #hlt23 builders, walked home from party_33 (5 armstump): "its list step 'armhlt spot_23 #hlt23' waits". The tower was finished at 15:29 by the other two.
- **U8, 4.** Orders queued from 19:21 on.
- **Not items.** The 6 false positives and the 5 id helpers named under the denominator; they are listed in `unfinished.json` under `never_started_not_items`.

## Outside the tally: a duplicate advanced plant

The 14:13 queue gave six constructors "armavp #avp1", meant as one building. All six began on the same tick (14:36), and the engine placed the orders at three sites 100-200 elmos apart. Two plants started: 16012 by constructor_540 (14:46) and 20158 by constructor_7444 (14:51). Both finished, at 17:50 and 17:56.

That is a second 2,600-metal advanced vehicle plant, built while the metal store sat at 0 from 14:30 to 17:10. It is the same-tick id race fixed on 2026-10-12 (`lists::nearest_first`, `same_tick_help`); this game predates the fix. It is not an unfinished frame, but at 2,600 metal it is larger than every item above together.

## The three mechanisms

1. **Matching a finished frame by type and distance, not by unit id.** This, together with the watchdog's 200-elmo exemption, stopped a builder that was still building and lost armsolar_8043 (152 metal). Both pieces of code are still in HEAD. The fix: match on the finished frame's unit id, and exempt a builder whose own frame is still rising.
2. **The pick swapping one generator for the other.** H-HANDS-STARTED lets a pick of another type replace a started build. The packet's energy sentence changed its answer each time the store crossed zero. This lost armadvsol_3273, armsolar_25530, armsolar_10959 and armadvsol_20119 (62 metal) and caused 8 never-started flips. The fix: don't offer the other generator "beside itself" while the builder's generator frame stands.
3. **A list headed by stop drops whatever is being built when it lands.** In real time, that is seconds after the report the player read. This lost armsolar_14870 (never in a report) and armadvsol_23255 (dropped knowingly, in the exit lane): 82 metal.

**Report gaps** (they cost no metal here):
- The abandoned line gives the share at destruction, so the 98% solar read "0% built".
- 5 of the 8 decayed frames had no hands' line naming a cell; the player saw only the tally.
- A kill by an unseen raid was reported as "abandoned".

## Commands run

```
grep -n 'name=' MATCH/script.txt; ls MATCH; tail MATCH/bot.log; head MATCH/impedance-review.md
python3 run/frames.py MATCH --seat 0            (and --json)
python3 run/floor.py MATCH
python3 run/impedance_sweep.py MATCH --seat 0 --all  [--from/--to 0:00-2:10, 5:40-7:20, 9:00-9:30]
python3 run/commander_turns.py MATCH --calls
python3 run/hands_window.py MATCH 1:38 1:52 constructor_23943
python3 run/hands_window.py MATCH 9:02 9:30 constructor_540 constructor_31474
python3 run/hands_window.py MATCH 10:53 10:56 constructor_14058 ; 12:01 12:03 constructor_3904 ; 13:47 13:50 constructor_427
scratch scripts (in the scratch directory): grepturn.py (a turn's report lines), dig.py (record rows for a frame and builder),
  menu.py (a builder's menu, gate ratings and pick from jev-0.jsonl), ns.py (decision rows per never-started order), build_json.py
record-0.jsonl reads: created/finished/placed events of plants, nanos and heavy turrets; dmg and en rows at D4 9:00-9:32;
  the #avp1 builders 14:30-18:10; the enemy commander's death (19:37)
git show 08599ab / b0d7213 (the watchdog and the UnitFinished handler as they stood when the game was played); git show e9f5175
```
One seat, so no symlink directory was needed. Nothing was torn or unread. hands_window printed no menus for the builders, so menus and ratings came from jev-0.jsonl directly.
