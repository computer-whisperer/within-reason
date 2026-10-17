# Unfinished structures: apex-v022-carrot-sym50-opus-1 / 00

We won at 22:17 against Apex v0.2.2 (standard profile) on Carrot Mountains v2.0, with a +50 income bonus for both sides. This was an arena game with one seat: Opus 5.5 medium played under an Opus 5.5 high commander, with all twelve hand layers on. Every tool reads seat 0, so I made no symlink directories.

frames.py, seat 0, verbatim:

```
1791902948-apex-v022-carrot-sym50-opus-1 seat 0, record to 22:16: frames 235 (decayed 8, finished 221, killed 5, unfinished at end 1), metal in unfinished frames 1131, build orders never started 65
```

`run/floor.py` gives `aband` 8, which equals frames.py's decayed count.

## Tally (14 frames, 1,131 metal)

| cause | n | metal | | fate | n | metal | | type | n | metal |
|---|---|---|---|---|---|---|---|---|---|---|
| U3 player re-listed / stopped | 7 | 585 | | decayed | 8 | 604 | | armnanotc | 4 | 501 |
| U4 raid killed frame or builder | 4 | 136 | | killed | 5 | 136 | | armmoho | 1 | 391 |
| U6 site (stuck builder) | 1 | 19 | | at end | 1 | 391 | | armbeamer | 1 | 89 |
| U8 game end | 1 | 391 | | | | | | armavp | 1 | 52 |
| U9 harness (own D-gun) | 1 | 0 | | | | | | armadvsol | 3 | 39 |
| U0, U1, U2, U5, U7 | 0 | 0 | | | | | | armmex | 2 | 38 |
| | | | | | | | | armsolar | 1 | 19 |
| | | | | | | | | armllt | 1 | 2 |

What the player knew:
- 10 frames were in the report before their cause acted.
- 4 were told only afterwards.
- None were never told.
- The player's own orders dropped 7 frames (all the U3 items).
  - 5 of those it dropped on purpose: the four exit-lane frames and the second advanced plant.
  - 1 more (armadvsol_25660) it also meant to drop, though it had not yet seen the frame stand.
  - 1 (armllt_14630) it dropped without meaning to. Its new list named that same Sentry again.

Two report faults touch what the player knew:
- **A builder's entry can show a neighbour's frame.** constructor_18918's entry read "74 %" at 13:15 and "91 %" at 13:25, while its own nano stood at about 3% and 19%. Those figures are armnanotc_443's, a nano 130 away. constructor_145's entry read "33 %" at 18:03 on a frame that had already decayed; 33% is armsolar_27325's progress.
- **The fights line gives the leftover share, not the share when the builder left.** It says "abandoned an unfinished armnanotc (0% built)" for frames left at 39-74%.

## Items by cost

1. **armmoho_7970: 21:43, seat 0, A5, unfinished at end (63%, 391 metal). U8.**
   - Happened: a tier-2 extractor started 33 s before the record ended; constructor_22066 was repairing it.
   - The player knew: yes. At 22:11: "building a Advanced Metal Extractor (armmoho) at spot_80 (A5) (53 % built)".
   - Fix: none. Cost: none. Confidence: high.
2. **armnanotc_2761: 14:40, seat 0, B5, decayed 16:32 at 74% (170 metal). U3.**
   - Happened: the 15:18 queue `["stop","armnanotc vp_yard2","armadvsol"]` (landed 15:23) dropped the nano. The builder, not the frame, stood in plant_31293's exit lane.
   - The player's note: "Dropping a build abandons it, so the metal already put into those three frames is lost."
   - The player knew, and dropped it on purpose. At 15:18: "(64 % built) ... standing in plant_31293's exit lane since 21 s ago". At 16:39 it was told "abandoned ... (0% built)".
   - Fix: hands-rule (the builder's build position). Cost: large. Confidence: high.
3. **armnanotc_22524: 14:42, seat 0, B6, decayed 16:28 at 70% (161 metal). U3.**
   - Happened: the same 15:18 turn. `["stop","extractor spot_114","armllt spot_114"]` took the builder out of plant_9971's lane and sent it to D6, where it died at 16:47.
   - The player knew, and dropped it on purpose. At 15:18: "(60 % built) ... standing in plant_9971's exit lane since 18 s ago".
   - Fix: hands-rule. Cost: large. Confidence: high.
4. **armnanotc_27106: 14:43, seat 0, B5, decayed 15:47 at 39% (90 metal). U3.**
   - Happened: the 14:59 queue `["stop","armnanotc vp_yard2","armadvsol"]` (landed 15:06). The player's note: "inside plant_9971's exit lane, so nothing that plant made could leave. I dropped that build."
   - The player knew. At 14:59: "(32 % built) ... standing in plant_9971's exit lane since 15 s ago". At 15:51 it was told "abandoned ... (0% built)".
   - Fix: hands-rule. Cost: large. Confidence: high.
5. **armbeamer_30104: 14:59, seat 0, B5, decayed 16:06 at 47% (89 metal). U3.**
   - Happened: the step "armbeamer home" was placed by the engine at spot_99, 301 from the command. The 15:18 stop moved the builder out of plant_13578's lane.
   - The player knew. At 15:18: "(35 % built) ... standing in plant_13578's exit lane since 13 s ago". At 16:11 it was told "abandoned ... (0% built)".
   - Fix: hands-rule. Cost: small. Confidence: high.
6. **armnanotc_26291: 13:13, seat 0, B5, killed 13:36 at 35% (80 metal). U4.**
   - Happened: six Pawns (party_22) raided home from 13:11. The builder was hit from 13:30 and died at 13:34; the pick never took a walk away (`go_home` rated 0.21-0.33). The frame was killed at 13:36. The home picket was away, and the commander turned on the Pawns only at 13:28.
   - The player knew, but with the wrong progress: its 13:15 report said "(74 % built) ... party_22 (5 armpw) 692 away". It had written "RAID AT HOME, FIRST" at 13:15.
   - Fix: player (no guard at home). Cost: large. Confidence: high.
7. **armavp_25896: 17:00, seat 0, B5, decayed 17:23 at 2% (52 metal). U3, also U1.**
   - Happened: at 16:51, a second after the first advanced plant stood, the pick played "build a armavp at energy_field". The PICK line: `build_armavp=0.56`. The packet still said "help build the Advanced Vehicle Plant at avp_yard until it stands".
   - The player's 16:57 stop landed at 17:05, but the frame had started at 17:00, inside the think penalty.
   - The player knew it was walking ("342 to go") and meant to drop it. At 17:27: "17:23 abandoned ... (0% built)".
   - Fix: hands-rule. Cost: large. Confidence: high.
8. **armmex_12981: 16:26, seat 0, D6 (spot_114, lost twice before), killed 16:38 at 55% (28 metal). U4.**
   - Happened: party_74 (2, then 4 Pawns) killed it. The 16:29 packet made it the home guard's job, but group_V1 stood 2,778 away and attacked at 16:38.
   - The player knew. At 16:29: "(15 % built) ... party_74 (2 armpw) 234 away".
   - Fix: player. Cost: small. Confidence: high.
9. **armadvsol_25660: 7:27, seat 0, B5, decayed 7:49 at 6% (21 metal). U3.**
   - Happened: the bare step "armadvsol" was placed by spot_99, with the builder standing in plant_13578's exit lane. At 7:24 the player sent `stop` plus solar_field ("I stopped that build"). The frame started at 7:27 and the stop landed at 7:31.
   - The player's report at 7:24 said "walking ... 294 to go"; at 7:54 it was told "abandoned".
   - Fix: hands-rule (the placement). Cost: large (an advanced-economy frame). Confidence: high.
10. **armsolar_4072: 17:45, seat 0, B6 (avp_yard), decayed 18:03 at 12% (19 metal). U6, also U5.**
    - Happened: the builder got `move_failed` at 17:50 and wandered 100-200 from the frame with its order still on. The energy store sat at 0-145 of 4,460. The same order rebuilt the solar on the same site at 18:09, and that one finished.
    - The player knew. At 17:53: "(12 % built) ... stuck: it cannot move from avp_yard (B6)".
    - At 18:03 the hands' line said "its builder was sent elsewhere", which was wrong.
    - Fix: hands-rule. Cost: small. Confidence: medium.
11. **armadvsol_29628: 13:51, seat 0, B5, killed 13:56 at 5% (18 metal). U4.**
    - Happened: the list (three bare advanced solars) ordered the step at 13:49 with the six Pawns 660 away. The packet's "walk away from the Pawns" reached the builder only when the pick moved it at 14:00.
    - The player knew. At 13:52: "(0 % built) ... party_22 (5 armpw) 424 away".
    - Fix: hands-rule (hold a list step under a raid). Cost: large. Confidence: high.
12. **armmex_4975: 9:14, seat 0, D6 (spot_111), killed 9:23 at 20% (10 metal). U4.**
    - Happened: an undefended forward extractor. party_30 (2 Pawns) was first seen 1 s before the kill.
    - The player knew the frame ("(18 % built)" at 9:21) but not the party.
    - Fix: none. Cost: small. Confidence: high.
13. **armllt_14630: 7:53, seat 0, B5, decayed 8:04 at 2% (2 metal). U3, also U9.**
    - Happened: the old list's Sentry was queued in the engine (7:45). The 7:44 list replaced the old one at 7:51, but the engine still held the queued Sentry and started its frame at 7:53, in the same second as the stop that dropped the old step. The new list named "armllt spot_87" as its third step.
    - The player was told only after: at 8:04, "abandoned".
    - Fix: hands-rule. Cost: small. Confidence: high.
14. **armadvsol_20221: 13:58, seat 0, B5, killed 13:59 at 0% (0 metal). U9, also U4.**
    - Happened: our own commander destroyed it with the pick's "D-gun party_22's nearest unit" (13:57-13:58, `dgun 8925 22522`). The `destroyed` row's `by` is 8925, our armcom.
    - The report said "lost armadvsol to unseen at home".
    - Fix: hands-rule. Cost: none. Confidence: high.

## Build orders never started

- frames.py lists 65 orders as never started. 16 of these did start, at the engine's `placed` site 301-595 from the command, which frames.py does not check.
- 4 more were long walks or queued waits that then started at the same site.
- 4 fell in the last 90 s of the game.
- That leaves 41 real ones: U3 20, U6 15, U2 5, U4 1.
- The JSON keeps the fifteen costliest. They are summarised here, and the other 26 are summarised in the last two bullets.

- **18:34 armfus (constructor_22971), U6, medium.** The builder walked for 53 s toward a site it could not reach ("878 / 710 / 421 / 542 / 561 to go"). The player re-sited it to fus_site at 19:39, and it was built 20:31-21:56. Cost: small.
- **14:17 armvp #vp5 (constructor_4020), U6.** It was placed on the same site as #vp4, in the same tick. The engine refused it at 14:38 ("the site was bad"). It was re-sited to vp_yard2 and built 15:09-16:14. Cost: small.
- **7:33 armadvsol (constructor_18000) and 8:19 armadvsol ×2 (constructor_145), U2.** The re-sent list's first steps were "taken as done" while the builder was only walking: 7:42, 8:22 and 8:23. The next step then replaced the walk. Three advanced solars were never built from these lists. Cost: small.
- **13:33 armavp #avp2 (constructor_22187), U3.** The player stopped it because the raid was at avp_yard. Cost: none.
- **16:51 armavp (constructor_15527, a pick), U3, also U1.** The player stopped the second advanced plant. Cost: none.
- **14:17 armvp (commander, a pick at avp_yard), U3, also U1.** The player stopped it. Cost: none.
- **Seven more U3 items in the fifteen** were re-listed by the player before the builder arrived. Cost: none.
  - 9:03 armadvsol ×2 (20065, 9145): moved to basic solars, which need no energy.
  - 15:13 armadvsol (30733): re-listed during a 4,000-elmo walk.
  - 17:34 armadvsol ×2 (1657, 10655): moved to basic solars when energy ran out.
  - 12:53 and 13:07 armnanotc (31160, 22187): re-listed for the #avp2 rebuild.
  - 14:40 armnanotc (22187): re-listed for #vp5.
- **The other 26, U6 (13).** Each lost the site to another builder of ours:
  - The same spot was on two builders' lists, then refused when the second arrived: spot_119, spot_91, spot_71, spot_52, spot_49, spot_143 ×2, spot_142, spot_164.
  - One site was placed for several builders in the same tick: 9:42 three wind turbines, 10:38 two solars, 17:44 five solars, four of them refused at 18:30.
- **The other 26, U3 (9), U2 (3) and U4 (1):**
  - U3: re-listed or stopped by the player.
  - U2: the opening extractors at spot_109, spot_87 and spot_78 were "taken as done" at 2:35, 2:45 and 3:15 with no frame. Each was built 3.1-3.7 min later.
  - U4: 16:39, the Sentry at spot_114 (constructor_31160). The pick walked the builder away from the raid, and it died at 16:47.

## Three mechanisms

1. **Builders stand in a plant's exit lane while they build beside it, and dropping the build is the player's only lever.**
   - Covers armnanotc_2761, armnanotc_22524, armnanotc_27106 and armbeamer_30104: 510 metal.
   - The frames were 39-74% built, and each builder was 20-40 s from done.
   - A build position outside the lane, or a lever that steps the builder out and keeps the build, would have kept all four.
2. **There was no guard where the frame stood when raiders came.**
   - Covers armnanotc_26291 and armadvsol_29628 in the 13:11-14:06 home raid, and armmex_4975 and armmex_12981 at D6: 136 metal.
   - A guard kept at the base and at D6 would have kept most of it, together with holding a list step while a party is within reach of its site (one step was ordered 660 from the Pawns).
3. **The hands began builds the player then had to drop.**
   - Covers armadvsol_25660 (a bare step placed by spot_99, with its builder standing in the plant's exit lane) and armavp_25896 (a second advanced plant, from the packet's "until it stands"). Both started inside the 7 s think penalty.
   - Also armllt_14630, an old list's engine-queued Sentry.
   - Together: 75 metal.
   - Placing builds out of the lanes, offering no second plant from "until it stands", and cancelling the engine queue when a list is replaced would have kept it.

For never-started orders, the mechanisms are different. There are 15 site collisions between our own builders, 5 "taken as done" steps that had only been walked to, and one unreachable fusion site.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...batch.json"
run/frames.py MATCH --seat 0 [--all | --json]; run/floor.py MATCH; run/raid_ledger.py MATCH
run/impedance_sweep.py MATCH --seat 0 --all [--from 7:00 --to 8:10 | 13:00-14:10]
run/commander_turns.py MATCH --calls | --notes
run/hands_window.py MATCH 9:10 9:26 constructor_31160 | 13:27 13:36 constructor_18918 constructor_18000 | 16:49 16:53 constructor_30137 constructor_15527 | 17:48 17:58 constructor_145
scratch scripts over record-0.jsonl (cmd/d/ev/s rows per builder, placed events, same-site placements, frames at the placed site) and strategist-0.jsonl (turn prompts, the player's text and tool calls)
```
