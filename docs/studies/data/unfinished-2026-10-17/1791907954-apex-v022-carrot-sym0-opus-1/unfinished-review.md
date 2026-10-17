# Unfinished structures: apex-v022-carrot-sym0-opus-1/00

We won at 10:57 (10.9 game minutes) against Apex v0.2.2 (standard profile) on Carrot Mountains v2.0 at +0. This was an arena game with one seat: an Opus 5.5 medium player under an Opus 5.5 high commander, with twelve hand layers.

frames.py, seat 0, verbatim:

```
1791907954-apex-v022-carrot-sym0-opus-1 seat 0, record to 10:56: frames 102 (decayed 3, finished 88, unfinished at end 11), metal in unfinished frames 1199, build orders never started 18
```

`run/floor.py` gives `aband` 3, which equals frames.py's decayed count.

## Tally (14 frame items, 1,199 metal)

| cause | n | metal |
|---|---|---|
| U3 the player re-listed the builder | 3 | 231 |
| U8 game end | 11 | 968 |
| U0-U2, U4-U7, U9 | 0 | 0 |

| fate | n | metal |
|---|---|---|
| decayed | 3 | 231 |
| killed | 0 | 0 |
| unfinished at end | 11 | 968 |

| type | n | metal |
|---|---|---|
| armadvsol | 5 | 655 |
| armnanotc | 2 | 198 |
| armllt | 2 | 93 |
| armsolar | 1 | 88 |
| armestor | 1 | 75 |
| armmex | 2 | 72 |
| armwin | 1 | 18 |

**What the player knew.**
- 13 frames were in a report before the cause acted. That is the 3 decayed, plus 10 of the 11 unfinished at end, which were in the 10:47 report.
- 0 frames were known only from the abandoned line.
- 1 frame was never in a report (armadvsol_28331, started 10:52).
- The player dropped 3 frames itself: all three decayed frames, each by a `queue` that began with `stop`, and each with a sentence saying the metal was lost.

**Not counted as never-started items: four orders frames.py lists that did start.** frames.py measures its 300 from the commanded point. The engine placed these four 342-599 away and the frame started there:
- 0:38 commander armvp: plant_13578 at (2432,6960), 397 away.
- 5:09 commander armvp: plant_4709 at (3024,7120), 599 away.
- 6:28 constructor_4020 armsolar: armsolar_1262, 342 away.
- 10:18 constructor_18766 armsolar: armsolar_1457, 370 away.

## Items by cost

1. **armadvsol_25757 · seat 0 · B5 (2912,6800) · decayed at 30%, 105 metal · U3, also U6**
   - 8:00 constructor_13453 began the 7:49 list's site-less `armadvsol` step. The hands chose the site: "301 from spot_96".
   - The builder worked from (2828,6898), inside plant_13578's east exit lane. The 8:18 report said: "building a Advanced Solar Collector (armadvsol) at 513 from spot_96 (B5) (19 % built) ... standing in plant_13578's exit lane since 17 s ago".
   - 8:18 `queue constructor_13453: ["stop", "extractor spot_106", ...]` returned "its build in progress dropped". The stop landed at 8:26, and the frame decayed at 9:11.
   - The player knew, and dropped it on purpose: "Some metal in that frame is lost". The 9:18 report said "abandoned an unfinished armadvsol (0% built) x1".
   - Fix: hands-rule. Cost: large (an advanced-economy frame). Confidence: high.
2. **armnanotc_21799 · seat 0 · B5 (2936,7128) · decayed at 44%, 101 metal · U3, also U6**
   - 5:31 constructor_4020 began the 5:11 list's `armnanotc #nano1`, sited by the hands at "403 from spot_99".
   - The builder stood at (2811,7047), in plant_13578's lane. The energy store was empty.
   - 6:01 wake: "plant_13578's exit lane is blocked: 1 of ours (armcv_4020) have stood in it for 0:30". The report: "(35 % built) ... standing in plant_13578's exit lane since 29 s ago".
   - 6:01 `queue constructor_4020: ["stop", "armsolar", ...]` returned "its build in progress dropped". The player: "The half-built turret will decay; about a third of its 230 metal is lost". It decayed at 6:53, and the 7:03 report said "abandoned an unfinished armnanotc (0% built) x1".
   - Fix: hands-rule. Cost: large (a nano frame). Confidence: high.
3. **armadvsol_5959 · seat 0 · B5 (2912,6800), the same site as item 1 · decayed at 7%, 25 metal · U3, also U5**
   - 9:48 constructor_23125 began the 9:38 list's `armadvsol` with the energy store at "25/2956 (+1) STALLING".
   - 9:48 `queue constructor_23125: ["stop", "armwin" x6, "armsolar" x2]` landed at 9:56. The player: "Each advanced solar needs 5,000 energy to build, so in a stall they barely progress ... I dropped its advanced solar, which had barely been started".
   - Its builder also stood at (2827,6895), in the lane, but no wake named it.
   - 10:14: "abandoned an unfinished armadvsol at B5 (0% built)".
   - Fix: player. Cost: large by the type rule (25 metal in). Confidence: high.
4. **The U8 frames: 11, all started within the last two minutes (9:23-10:52), 968 metal.** None was lost to the result, so their cost is none.
   - armadvsol_21714 (B5): 86%, 301 metal.
   - armadvsol_3554 (B5): 60%, 210.
   - armnanotc_21609 (B5): 42%, 97. This is the #n6 turret, late because of never-started item 1 below.
   - armsolar_1457: 57%, 88.
   - armestor_27502: 44%, 75.
   - armllt_24972 (D6): 60%, 51.
   - armmex_12633 (E5): 87%, 44.
   - armllt_22522 (E4): 49%, 42.
   - armmex_14957 (C4): 56%, 28.
   - armwin_28069: 46%, 18.
   - armadvsol_28331: 4%, 14.

   Each builder was on its frame to the end, or had only its next step queued behind it at 60%. The two older advanced solars ran slowly through the energy stall: 21714 is 93 s in against 88 s of build time alone at full stores.

## Builds never started (14 items: U3 8, U9 1, U8 5)

1. **8:56 constructor_6889 · armnanotc #n6 · B5 · U9 · small · medium confidence.**
   - 8:50: the pick sent the builder east ("walk to spot_101", plan).
   - 8:56: the list sent an unqueued build at (2920,7128), to the west.
   - The builder kept walking east, (3534,7133) at 8:55 to (6247,7238) at 9:50, and the build never began. The record has no `rejected` and no `no_site` event, so it does not say why the engine kept the move.
   - Its report entry, every turn from 8:58 to 9:38: "walking to build a Construction Turret ... 769 / 1244 / 1783 / 2295 / 2835 to go". The player read it as "driving 2,800 back home" and re-listed it in the east at 9:38.
   - #n6 began only at 10:21, on another builder.
2. **4:37 constructor_3402 · armllt spot_122 · C6 · U3 (tool) · small.**
   - The step was queued behind the extractor from the 3:05 list.
   - The 4:30 list replaced the list whole and dropped the Sentry. The report showed only "building a Metal Extractor ... (26 % built)", not the steps behind it. The hands' line came at 4:45: "dropped the step 'armllt spot_122' ...".
   - The Sentry stood there only at 9:03.
3. **3:42 constructor_15425 · armmex spot_78 · U3 · none.**
   - The player sent the builder elsewhere: "The commander keeps spot_78". The commander built it at 4:01.
4. **5:55 constructor_15425 · armmex spot_67 · U3 · none.**
   - The 5:51 list put three solars first and spot_67 fourth. The queued step was dropped at 6:09, and the extractor was built at 9:04.
5. **7:27 constructor_23008 · armsolar home · U3 · none.**
   - The player saw "walking to build ... 554 to go" and replaced the step with a third plant.
6. **7:31 constructor_18766 · armsolar spot_96 · U3 · none.**
   - The old list's step began after the 7:30 report and before the new list landed at 7:36.
7. **8:50 constructor_3402 · armllt spot_99 · U3 · none.**
   - The pick filled an idle builder while the 8:48 queue was in flight. The list replaced the pick at 8:56.
8. **8:50 constructor_15091 · armmex spot_69 · U3 · none.**
   - The same as item 7: the pick in the gap, the list at 8:56.
9. **9:45 constructor_23125 · armestor · U3 · none.**
   - The same: the pick at 9:45, the 9:38 list at 9:46.
10. **U8 (5):** five orders in the last 30 s before the record ended at 10:56, each queued behind a frame or a pick at the end:
    - 10:26 constructor_20403 armadvsol.
    - 10:51 constructor_23346 armllt.
    - 10:52 constructor_15425 armmex.
    - 10:56 constructor_23008 armwin.
    - 10:56 constructor_13581 armestor.

## The three mechanisms

1. **Builders working in a plant's exit lane.**
   - The hands sited site-less nano and advanced-solar steps just past the end of plant_13578's 320-elmo east lane. The builder, standing about 130 short of its frame, stood inside the lane.
   - The yard wake then left the player only `stop`.
   - A keep-out that covers the builder's working ring, or the hands moving a working builder out of the lane, would have kept items 1 and 2 (206 metal). Item 3 stood at the same site.
   - This refines the impedance review's A4. The building keep-out (H-ECO-YARD-LANE) held, but the builder's own standing place is not kept out of the lane.
2. **Advanced solars ordered into an empty energy store.**
   - armadvsol is 5 of the 14 frames and 655 of the 1,199 metal. The energy store read 0 for most of 9:40-10:56 and of 5:40-6:40, while metal sat at 800-1,580.
   - The player found the stall at 9:48 and dropped item 3 (25 metal).
   - A report line saying a 5,000-energy build will crawl at an empty store, or wind and solar ordered first, would have kept those 25 metal and finished the U8 advanced solars sooner. No metal was lost to the result.
3. **A list's build that did not replace the pick's walk** (never-started item 1, U9).
   - The builder walked 2,900 the wrong way for 50 s while the report said it was closing on its site.
   - A check that a builder ordered to build is closing on its site would have started #n6 about 75 s sooner. No metal was lost.

**A report finding with no metal behind it.** The abandoned line reads the frame's share when the frame is gone, so it says "(0% built)" for frames left at 44%, 30% and 7%. The player cannot learn from it what a drop cost.

## Commands run (from the repository root; M = run/matches/1791907954-apex-v022-carrot-sym0-opus-1/00)

```
cat M/../results.jsonl; python3 -c "...json.load(open('M/../batch.json'))"
run/frames.py M --seat 0; run/frames.py M --json; run/floor.py M
run/impedance_sweep.py M --seat 0 --from 5:20 --to 7:10 --all | 7:25-8:50 | 9:20-10:57 | 3:30-4:50 | 2:50-3:20 | 4:55-5:15
run/hands_window.py M 8:49 9:00 constructor_6889; run/hands_window.py M 9:44 9:47 constructor_23125
scratch scripts (scratchpad .../unfinished/1791907954-apex-v022-carrot-sym0-opus-1/scripts/): turn.py (one turn's prompt or lines of it),
  after.py (the player's reply and calls after a turn), unit.py (a unit's cmd/ev/d rows in a window), ns.py (each never-started
  order's placement, creations, commands, decisions), write_json.py (this review's JSON)
python over record-0.jsonl (builder positions in s rows, the stores, frames at the never-started sites, plant facings, unit_defs build
  times); grep engine.log for unit 6889's build lines
read: the brief, M/impedance-review.md, docs/harness/record-format.md, docs/heuristics.md (H-ECO-YARD-LANE, H-HANDS-SCRIPT,
  -QUEUE, -STARTED), crates/bot/src/brain/pianist/execute.rs (execute_builder), lists.rs (play_lists)
```

There was one seat, so no symlink directories were needed.
