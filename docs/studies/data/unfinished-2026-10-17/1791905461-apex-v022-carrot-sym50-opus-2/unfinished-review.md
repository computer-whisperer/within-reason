# Unfinished structures: apex-v022-carrot-sym50-opus-2 / 00

An arena duel on Carrot Mountains v2.0 (+50/+50, mirror, seed 4), played 2026-10-13 on commit 4ed68d8: **won 18:10** against Apex v0.2.2 (standard). One seat, played by Opus 5.5 medium under an Opus 5.5 high commander, with all twelve hands layers on. All the tools ran on seat 0 directly, so I made no symlink directories.

## Denominator (frames.py, verbatim)

```
1791905461-apex-v022-carrot-sym50-opus-2 seat 0, record to 18:09: frames 168 (decayed 7, finished 155, killed 1, unfinished at end 5), metal in unfinished frames 1896, build orders never started 36
```

`run/floor.py` reports `aband` 7, which equals frames.py's decayed count. Its `aband_m` is 20 rather than 638 because it multiplies each frame's share at the last tick (about 0) by its cost; frames.py uses the share the frame reached.

## Tally

| cause | n | metal in |
|---|---|---|
| U1 the pick sent the builder elsewhere | 1 | 113 |
| U3 the player re-listed the builder (`stop`) | 6 | 525 |
| U4 the builder died / the frame was killed | 1 | 45 |
| U8 game end | 5 | 1213 |
| U0, U2, U5, U6, U7, U9 | 0 | 0 |
| **all** | **13** | **1896** |

Secondary causes (`also`): U5 on 3 items (the energy stall), and U9 on 2 (armsolar_1515: nothing returned the builder to its frame; armllt_21831: the hunts ended on the leash).

| fate | n | metal in |
|---|---|---|
| decayed | 7 | 638 |
| killed | 1 | 45 |
| unfinished at end | 5 | 1213 |

| type | n | metal in |
|---|---|---|
| armadvsol | 5 | 469 |
| armfus | 1 | 502 |
| armmoho | 2 | 310 |
| armsilo | 1 | 243 |
| armsolar | 2 | 221 |
| armvp | 1 | 106 |
| armllt | 1 | 45 |

**What the player knew.** The report listed all 8 frames lost before the end, with their percentages, before the cause acted. The player dropped 6 of them itself with `stop` lists and said why each time: 3 to clear plant_30437's exit lane, and 3 because of the energy stall. Each of the 7 decayed frames was followed by an "abandoned an unfinished X (N% built)" line, but that line always read 0-1%. Frames.py has no cap issue here: 13 unfinished frames, all read in full.

## Items by cost

1. **6:11, armadvsol_25662, A5 (39%, 136 metal): U3, also U5, large, high confidence.**
   - What happened: constructor_11252's #as1 advanced solar crawled in an empty energy store. The player's 6:03 `queue` began with `stop` and moved the builder to basic solars ("6:11 stopped what it was doing on your `stop`").
   - What the player knew: the report said "(32 % built), ordered 70 s ago". Its note: "advanced solars and wind cost energy to build and crawled at 32% for 70 s". It said: "I dropped the two other advanced solars ... the metal in them is lost."
   - Fix class: player.
2. **6:52, armsolar_1515, B5 (73%, 113 metal): U1, also U9, large, high confidence.**
   - What happened: a lone Tick hit constructor_426. The pick played `go_home` (0.5, change 0.85), whose words read "leaves the Solar Collector (armsolar) at about 70%, which decays". The packet's sentence was: "A Tick or raid party hitting them: they walk to home and come back when it is gone."
   - Nothing came back. The frame belonged to a replaced list, so at 7:06 the new list's unplaced `armsolar` began a fresh solar at home, 400 away, and the 73% frame decayed from 7:03 to 7:27.
   - What the player knew: the 6:48 report gave "(57 % built) ... party_17 (1 armflea) 288 away". The abandoned line at 7:34 said "1% built".
   - Fix class: hands-rule.
3. **4:36, armsolar_24908, B5 (70%, 108 metal): U3, large, high confidence.**
   - What happened: the pick's solar "beside itself" (4:15) put constructor_11252 in plant_30437's exit lane. The player's 4:28 `queue` began with `stop`.
   - What the player knew: the report said "(39 % built) ... standing in plant_30437's exit lane since 13 s ago". It said: "I dropped it".
   - Fix class: hands-rule (the siting). Letting the frame finish was the player's other option.
4. **7:32, armvp_6172, B5 (18%, 106 metal; a plant frame): U3, also U5, large, high confidence.**
   - What happened: the third vehicle plant (#vp3) was begun at 7:17 with the energy store at 0. The player's 7:25 `queue` gave `stop` and six solars.
   - What the player knew: it said "I dropped the third vehicle plant. It was only 9% built, and a third plant would have pulled more energy we don't have." The plant was begun again at 8:28 and finished.
   - Fix class: player.
5. **5:22, armadvsol_12142, B6 (25%, 88 metal): U3, large, high confidence.**
   - What happened: constructor_426's staged 4:18 list played its unplaced `armadvsol` beside itself the moment it rolled out of plant_30437, in that plant's exit lane (4:48). The 4:44 list landed at 4:51, after the frame had begun, so by the rules it left the build to finish.
   - The player's 5:14 `stop` dropped the frame. The report had said "(19 % built) ... standing in plant_30437's exit lane since 26 s ago".
   - Fix class: hands-rule.
6. **6:11, armadvsol_12344, A5 (14%, 49 metal): U3, also U5, large, high confidence.**
   - What happened: dropped by the same 6:03 `stop` re-list as item 1.
   - What the player knew: the report gave this builder "(32 % built)". That figure was constructor_11252's frame 96 away; this builder's own frame stood at about 8%.
   - Fix class: player.
7. **4:52, armadvsol_26536, B6 (11%, 38 metal): U3, large, high confidence.**
   - What happened: the 4:28 list's unplaced `armadvsol` was sited beside constructor_11252, which was still in the exit lane. The player marked `solar_field` and re-listed with `stop`.
   - What the player knew: the report said "(5 % built) ... standing in plant_30437's exit lane since 29 s ago".
   - Fix class: hands-rule.
8. **12:57, armllt_21831, E4 (53%, 45 metal; its 135-metal constructor died too): U4, also U9, small, high confidence.**
   - What happened: a pick sent constructor_6816 to a forward Sentry at mid-map. From 12:40 a lone Pawn shot it from 512 away from our standing Sentry, outside that Sentry's 430 reach.
   - Its menu read change 0.76-0.83 every second, but no move reached the 0.5 flag (go_home 0.24-0.34). It was never in a pick component ("Left to nobody: party_47 ... killing our Construction Vehicle"), and three hunts sent at the Pawn "ended after 0-1 s: the leash of 900".
   - What the player knew: the 12:45 and 12:55 reports said "under_fire ... near death (14%)". The 12:45 packet forbade any Blitz answer to a lone Pawn.
   - Fix class: hands-rule.
9. **18:09 (U8, cost none):** armfus_10730 (15%, 502 metal), armsilo_15388 (3%, 243), armmoho_26545 (30%, 186), armadvsol_13747 (45%, 158) and armmoho_31418 (20%, 124). Each was begun from 17:29 to 17:59 with its builder still on it at the record's end.

## Build orders never started

Of frames.py's 36 entries, 13 did start, so they are skipped:

- **11 helped another builder's frame at the same site.** Six builders listed "armadvsol solar_field" without ids at 16:40. constructor_21920 began the frame and the other five stood 105-187 from it while it went up, and again at 17:26 on constructor_5110's frame. The 12:23 case was three builders' solars placed on one site.
- **2 were placed beyond frames.py's 300 radius.** A solar placed 320 from its ordered point became armsolar_1515, and the silo placed 472 away was begun at 17:53.

The other 23 orders are classified below.

- **U2, 2 orders (small cost, high confidence).** 6:36-6:39, constructor_19403: the 6:29 re-list's three unplaced `armsolar` steps were each "taken as done" against its unstarted solar order. The fourth step then replaced that order with the builder 363 from the site, so four planned solars were never built during the energy stall. At 2:41 the same thing happened to constructor_11430's `extractor spot_103`. The 2026-10-16 H-HANDS-LISTS change (a re-sent first step waits for the frame) should cover the first step of such a list; that is unmeasured.
- **U3, 10 orders (cost none).** A new player list replaced a step queued at 60%, or replaced an unstarted pick:
  - two mohos at 16:40, dropped for #fus2 and #silo1 ("dropped the step ... its old list had queued");
  - a moho at 17:42 that the pick sent to a spot another builder's list held;
  - three steps at 9:19, replaced by #avp1;
  - steps at 2:43, 3:48, 7:16 and 8:16.
- **U6, 9 orders (cost none).** These were all extractor orders rejected with "the site was bad", because another builder of ours took the spot first. Seven were two player lists holding the same spot (spot_95, 122, 134, 129, 142, 143); two were picks sent to a spot another builder's list was walking to (12:34 spot_95, 11:09 spot_129). Fix class: tool (two lists can hold one spot).
- **U8, 2 orders.** Both came within 13 s of the end, one of them the #fus3 second fusion.

## Three mechanisms

1. **291 metal (items 1, 4, 6): the energy cost of advanced builds.** Advanced generators (5000 energy each, while a basic solar costs none) and a third plant were begun in an empty energy store. They crawled, and the player then dropped them. A builder's entry that gave the time to finish at the current energy income, or the energy price before an advanced solar is listed, would have kept them unstarted. Fix class: report.
2. **234 metal (items 3, 5, 7): siting in exit lanes.** Builds sited "beside itself" while the builder stood in plant_30437's exit lane were dropped by the player to clear the lane. Siting unplaced and beside-itself builds outside exit lanes would have prevented all three. Fix class: hands-rule.
3. **113 metal (item 2): no way back to a frame.** A builder that the pick walks off its frame is never sent back to it, and the new list's unplaced step begins a fresh frame where the builder stands. The packet's "come back when it is gone" had no lever. Fix class: hands-rule. Close behind is item 8: a builder under fire whose change reads high while no move reaches the flag is never moved.

## Side findings

- The report's "abandoned an unfinished X (N% built)" gives the share at the last tick (`briefing.rs` `unfinished`), so every decayed frame read 0-1%. armsolar_1515 was at 73% when its builder left and was reported "1% built". This is still the code at HEAD.
- A builder's entry takes the most advanced frame of its type within 200 of the task's place (`picture.rs` `task_course`). Three builders at `solar_field` all read 27%, then 32%, while constructor_426's own frame stood at 4-8%. This is still the code at HEAD.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...batch.json"
run/frames.py MATCH --seat 0; run/frames.py MATCH --seat 0 --json > scratch/frames.json
run/floor.py MATCH
run/impedance_sweep.py MATCH --seat 0 --from 3:30 --to 5:30 --all | --from 6:20 --to 8:00 --all | --from 16:20 --to 18:10 --all | --all (saved to scratch)
run/hands_window.py MATCH 6:44 7:08 constructor_426 | 12:17 12:58 constructor_6816 | 12:49 12:58 constructor_6816
scratch scripts over strategist-0.jsonl (turn prompts, assistant text, tool calls for turns 10-28, 56-60), record-0.jsonl (decision rows, commands, events, samples per builder and frame; created/placed events by site; builder distances to frames), jev-0.jsonl.zst via run/jevlog.py (full menus at 6:51-6:52 and 12:44-12:45, worlds_gate components and plan rows at 12:45)
code read at 4ed68d8 and HEAD: brain/pianist/compose.rs (FLAG 0.5), picture.rs task_course, briefing.rs abandoned/unfinished, pianist/mod.rs abandoned line; docs/heuristics.md H-HANDS-SCRIPT, -QUEUE, -STARTED, -LISTS
```
