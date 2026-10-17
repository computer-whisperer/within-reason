# Unfinished structures: 1791427881-human-27

This was a human realtime game on 2026-10-07 on Comet Catcher Remake 1.8. We had one seat (seat 0: an Opus 5.5 medium player under an Opus 5.5 commander, Cortex, start G1) against one person, [gecko]u6bkep (Armada, lobby skill 30). **We lost at 13:48.** Blitz raids held us to 4 extractors from 4:30, Stouts and a Janus killed the plant at 11:51 and razed home, and the record ends as our commander dies at the new plant at spot_17.

frames.py, seat 0, verbatim:

`1791427881-human-27 seat 0, record to 13:48: frames 47 (decayed 3, finished 36, killed 6, unfinished at end 2), metal in unfinished frames 860, build orders never started 26`

- floor.py `aband` 3 equals frames.py's decayed count of 3. Its `aband_m` of 33 differs from the decayed frames' metal_in of 248 because floor.py multiplies by the last sampled share (the decayed remnant), while frames.py uses the share the frame reached.
- One of the three "decayed" frames, corhllt_10583, was killed by Stout fire: the dmg rows show the hits from 12:26, but the destroyed event names no attacker.
- Two of the 26 never-started orders did start: the 12:16 `corvp spot_17 #vp2` orders became corvp_22076 and corvp_1291, 408 and 396 from the ordered point, outside frames.py's 300 radius. They are not items here, so 24 never-started items remain.

## Tally (11 frames, 860 metal)

| cause | n | metal |
|---|---|---|
| U1 the pick moved the builder | 1 | 34 |
| U3 the player dropped it | 2 | 99 |
| U4 raided or killed (builder fled or frame shot) | 6 | 378 |
| U8 game end | 1 | 64 |
| U9 harness (same-tick id race) | 1 | 285 |
| U0, U2, U5, U6, U7 | 0 | 0 |

| fate | n | metal |
|---|---|---|
| decayed | 3 | 248 (one of them really killed by fire) |
| killed | 6 | 263 |
| unfinished at end | 2 | 349 |

| type | n | metal |
|---|---|---|
| corvp (plant) | 1 | 285 |
| cornanotc (nano) | 3 | 234 |
| corhllt (Twin Guard) | 1 | 168 |
| corsolar | 3 | 128 |
| corllt | 1 | 37 |
| cormex | 2 | 8 |

**What the player knew:**
- **Before the cause acted (8):** corllt_12691, corsolar_20037, corsolar_22545, cornanotc_20410, corhllt_10583, cornanotc_22093, corvp_1291, corsolar_31698.
- **Only afterwards (3):** cormex_1850, cormex_21827 and cornanotc_3536. It learned of them from the loss or abandoned line.
- **Never:** 0.
- **Dropped by its own order (3):** corsolar_22545, cornanotc_22093, and corvp_1291 (whose primary cause is U9).

## Items, ranked by metal put in

1. **12:49, seat 0, corvp_1291 (H3), U9 + U3, 285 of 570, large, high.**
   - What happened: constructor_16990 and constructor_24695 both played `corvp spot_17 #vp2` at 12:16, and each saw the id free. That made two plants: 22076 at 12:24 and this one at 12:49. The commander's 12:32 list guarded 24695 ("help build #vp2 (help constructor_24695 build)").
   - Evidence: at 12:55 the player stopped 24695 ("list cancelled and its build in progress dropped"); the stop landed at 13:08 at about 43%. The frame peaked at 50% at 13:13 and decayed to 26%. At 13:36 the re-used `#vp2` step on 16990's 13:24 list repaired it ("help build #vp2 (repair)"), so it stood at 29% at the end.
   - The player knew: the 12:55 report said "constructor_24695: ... building a Vehicle Plant (corvp) at 307 from spot_25 (H3) (12 % built)", and the player wrote "Two plant frames had gone up instead of one". At 13:13 it believed "The second plant frame is gone".
   - Why U9, not U8: its builder was taken off 40 s before the end and the frame was decaying.
   - Fix: hands-rule. The same-tick race was fixed 2026-10-12; the id's claim on the dropped frame at 13:36 was not looked at.
2. **11:35, seat 0, corhllt_10583 (G2), U4, 168 of 195, large, high.**
   - What happened: constructor_20595 built the Twin Guard from its list ("corhllt spot_10") and stayed on it. It was at 86% at 12:25 when party_27 (5-6 Stouts and a Janus at 258-424) shot it down: 82%, 34%, 14% between 12:26 and 12:29.
   - Evidence: dmg rows [10583, 833] and [20595, 198] at 12:27; destroyed by null at 12:29; the builder died at 12:34.
   - The player knew: the 11:53 report said "building a Twin Guard (corhllt) at spot_10 (G2) (26 % built) ... enemies_near: party_27 (1 armjanus, 3 armstump) 433 away". Afterwards the 12:32 report said "abandoned an unfinished corhllt (14% built)", which names the wrong cause and the remnant share.
   - Fix: none (army cover). The energy stall cost it only about 5 s.
3. **10:48, seat 0, cornanotc_20410 (G2), U4 + U5, 147 of 230, large, high (U5 medium).**
   - What happened: the nano came from the 10:31 list ["corhlt spot_10", "cornanotc"]. The energy store was at 0 from 11:04 (+130-270 a second), so it stood at only 64% after 71 s (56 s at the full rate). party_27 hit the builder at 11:54 and 11:58.
   - Evidence: "11:58 (plan) walk to spot_12; its list step 'cornanotc' waits for it", PICK at 0.64, the move saying "leaves the Construction Turret (cornanotc) at about 60%, which decays". This followed the 11:40 packet's "an enemy party near it: it walks to spot_12 beside the commander". An armstump killed the frame at 25% at 12:43.
   - The player knew: the 11:53 report said "(cornanotc) at spot_10 (G2) (56 % built) ... enemies_near: party_27 ... 378 away" and "energy 0/1400 (+1) STALLING". Afterwards: "lost cornanotc to armstump".
   - Fix: player (the stall came from its own concurrent energy-heavy lists).
4. **13:35, seat 0, corsolar_31698 (H3), U8, 64 of 150, small, high.**
   - The first solar of the player's 13:13 list ["stop", "corsolar spot_25", ...]. It was still building when the record ended 13 s after it was started.
   - The 13:42 report listed it at 21%. Fix: none.
5. **12:45, seat 0, cornanotc_22093 (H2), U3 + U5, 53 of 230, large, high.**
   - What happened: this was the third frame of 26263's "cornanotc" step. With energy at 0 it reached 23% in 35 s. The player's 13:13 list `["stop", "corsolar spot_25", "corsolar spot_25", "assist"]` dropped it ("its build in progress dropped"; it landed at 13:20), and an armstump killed it at 13:42.
   - The player knew and meant it: the reports showed "(13 % built)" at 12:55 and "(21 % built) ... STALLING" at 13:13. The player wrote: "dropped its construction turret (21% built) to put up two solars ... Solars need no energy to build."
   - Fix: player.
6. **9:20, seat 0, corsolar_22545 (G1), U3, 46 of 150, small, high.**
   - What happened: the solar came from a list step. At 9:22 the player sent "queue constructor_31089: stop" ("list cancelled and its build in progress dropped") to put every builder on the wrecks. The stop landed at 9:33 with the frame at 31%; the builder was put on the wrecks at 9:37 and the frame decayed at 9:56.
   - The player knew and meant it: the 9:22 report said "(corsolar) at spot_6 (G1) (1 % built), ordered 1 s ago" with energy at 1842/1850. Its note: "Every free builder + commander reclaiming ... is the fastest economy". The 10:04 report then said "abandoned ... (1% built)".
   - Fix: player. The realtime landing delay raised the loss from about 2 metal to 46.
7. **5:06, seat 0, corllt_12691 (H3), U4, 37 of 90, small, high.**
   - What happened: the near-dead builder (29%) was building the Guard at outer spot_25. A Blitz hit it at 5:22, and the pick walked it to spot_6 (0.58) and then home (0.86). The Blitz killed the 41% frame at 5:25.
   - The player knew: the 5:21 report said "building a Guard (corllt) at spot_25 (H3) (40 % built) ... near death (29%); enemies_near: party_15 (1 armflash) 207 away".
   - Fix: none.
8. **12:13, seat 0, cornanotc_3536 (G1), U1 + U9, 34 of 230, large, high.**
   - What happened: after the 11:58 walk, the stale "cornanotc" step was played again at 12:12 at spot_5, at home, instead of resuming 20410 (57%, 450 away), with the Stouts 885 away.
   - Evidence: "12:21 (plan) walk to spot_17; its list step 'cornanotc' waits for it", split 0.36, the move saying "stepping away from party_27 (3 armstump) ... leaves the Construction Turret (cornanotc) at about 10%, which decays". This followed the 12:16 packet's "an enemy party within reach: it walks to the commander at spot_17". The builder was not hit; the frame decayed to 0 by 12:43.
   - The player knew only afterwards: no report came between 12:13 and 12:21, and the 12:46 line said "abandoned an unfinished cornanotc (0% built)".
   - Fix: hands-rule (resume the standing frame).
9. **5:19, seat 0, corsolar_20037 (F2), U4, 18 of 150, small, high.**
   - What happened: a list solar on the base's west edge. party_12's Blitzes killed it at 12% with the builder on it; the builder was alarmed at 5:26.
   - The player knew: the 5:21 report showed "(2 % built)" and party_12 (3 armflash) "moving toward our side".
   - Fix: none.
10. **4:23, seat 0, cormex_1850 (F1), U4, 4 of 50, small, high.**
    - What happened: "next, queued: build a cormex at spot_3 (F1)" (4:03) was promoted while the builder was being hit, with a Blitz 115-244 away. The pick then walked it to spot_10 (4:25, 0.66), and the Blitz killed the 8% frame at 4:26.
    - The player knew only afterwards: "lost cormex to armflash" at 4:32.
    - Fix: hands-rule.
11. **7:42, seat 0, cormex_21827 (H3), U4, 4 of 50, small, high.**
    - What happened: a step queued at 7:08 walked the builder onto a Blitz that had stood 94 from spot_25 since 7:30. The builder started the frame beside it, and the Blitz killed it at 8% at 7:45.
    - The player knew the threat but saw the frame only afterwards: the 7:37 report said "walking to build a Metal Extractor (cormex) at spot_25 ... party_15 (1 armflash) 414 away".
    - Fix: hands-rule.

## Never started (24 items; by cause: U1 8 orders / 640 metal of builds, U4 6 / 540, U6 4 / 770, U3 4 / 240, U2 2 / 100)

- **U6 site:**
  - 10:36 constructor_26263 corhlt at G2 (480): "the engine refused a corhlt at G2: the site was bad".
  - 6:33 constructor_15016 corllt at G2: refused, "the site was bad".
  - 9:43 constructor_24695 cormex at spot_6: a `no_site` event. spot_6 stayed empty from 4:00 to the end.
  - 2:08 commander corsolar at G2 (low confidence): it stood 88 from the point, inside its build distance, for 42 s, and the build never started. The placed site is not in this record.
- **U1 pick:**
  - 5:39, 6:35 and 6:57 constructor_16990 corllt spot_25 (8 orders): the list re-ordered the step every 5-6 s, and each time the pick walked the near-dead builder home under the packet's "badly hurt ... walks to the commander and is repaired". The list hold covers only a builder threatened within 800.
  - 10:27, 10:49 and 11:11 commander corllt at spot_6: the pick's own build/walk alternation, with move_failed at 10:48 and 11:09.
  - 11:58 and 12:01: list extractors toward the Stouts, walked away by the pick the same second.
- **U4 builder hit, alarmed or killed:**
  - 4:33 constructor_23546 cormex at F1.
  - 5:32 constructor_15016 corsolar at G2.
  - 7:17 constructor_833 cormex at F3 (the builder died at 7:41).
  - 7:44 constructor_24695 corsolar at G2.
  - 7:47 constructor_15016 corllt at spot_25 (the Blitz still stood there).
  - 12:30 constructor_20595 cormex at F1 (the builder died at 12:34).
- **U3 player's re-list:**
  - 9:59 constructor_24695 (its 10:04 list began with `stop`).
  - 10:56 constructor_20595 corllt queued behind a build: "dropped the step 'corllt spot_17' its old list had queued".
  - 12:05 and 12:14: replaced by the 11:53 #vp2 lists, which landed at 12:16.
- **U2 taken as done:**
  - 6:55 constructor_833 cormex at spot_6.
  - 10:23 constructor_16990 cormex at spot_25.
  - In both, a re-list's first step matched the unstarted order and was "taken as done", and the next step replaced the order (a Guard was built at spot_25 over no extractor). This is the 2026-10-16 H-HANDS-LISTS fix.

## Three mechanisms

1. **The same-tick id race on `#vp2`** cost corvp_1291, 285 metal: a second plant on an income of 7-11. It is fixed since 2026-10-12 and unmeasured since. The id's claim still pointed at the dropped frame at 13:36.
2. **The energy stall of 11:04-12:00.** The store sat at 0 on +130-270 a second while three Twin Guards, a nano, a refused HLT and the plant drew from the player's 10:04-10:40 lists.
   - cornanotc_20410 was at 64% after 71 s; at the full rate it would have stood by about 11:44, before its builder was hit at 11:54.
   - cornanotc_22093 was dropped at 23% for solars.
   - That is 200 metal, plus corhllt_10583 (168) marginally.
3. **Frames started into a known enemy's path with no army to cover them**, 265 metal in all:
   - The Blitz raids took cormex_1850, corllt_12691, corsolar_20037 and cormex_21827 (63 metal). Two of these were queued list steps that walked their builder onto a standing Blitz or were promoted while it was hit; those 8 metal are the hands-rule part.
   - party_27's Stouts at home took corhllt_10583 (168) and cornanotc_3536 (34). The nano came from a stale list step re-sited instead of resuming the standing frame.

Also noted:
- The report's abandoned line gives the remnant share ("1%", "0%") rather than the share the frame reached (31%, 15%), and calls a frame shot at 86% "abandoned".
- The realtime landing delay raised the loss on both of the player's drops: 1% to 31% for the solar, 12% to about 43% for the plant.

## Commands run

```
grep -n 'name=' MATCH/script.txt; ls MATCH/record-*.jsonl; tail MATCH/record-0.jsonl MATCH/bot.log
run/frames.py MATCH --seat 0 | --all | --json
run/floor.py MATCH
run/impedance_sweep.py MATCH --all (whole game) and --all --from 9:00 --to 10:00
run/hands_window.py MATCH 11:53 12:00 constructor_26263; 12:10 12:22 constructor_26263; 4:22 4:27 constructor_23546;
  5:18 5:27 constructor_16990 constructor_15016; 7:38 7:48 constructor_15016; 12:20 12:31 constructor_20595
python3 over strategist-0.jsonl (turn prompts 5, 11, 12, 14, 16, 17, 26, 27, 35, 37, 40-48; the player's text and calls at 12:55-13:24)
python3 over record-0.jsonl (d/cmd/ev rows per builder; s rows of every unfinished frame; en/dmg rows near the frames)
python3 over jev-0.jsonl.zst via run/jevlog.py (call/plan/worlds_gate rows at 5:45, 11:58, 12:21; threat events; the packets from 10:30)
```

Single seat: no symlink directory was needed. Nothing was torn and no tool crashed.
