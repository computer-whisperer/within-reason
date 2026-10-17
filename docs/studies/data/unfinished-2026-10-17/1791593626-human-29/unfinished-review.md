# Unfinished structures: 1791593626-human-29

This was a realtime 1v1 on Comet Catcher Remake 1.8 against a person, [gecko]u6bkep (lobby skill 30, Armada). We had one seat (Opus 5.5 medium as the player, Jev 1.13.0 as its hands, an Opus commander above it). **We won at 18:03**, when his commander was destroyed at B4. The record ends at 18:14. The person wrote nothing in the chat.

frames.py, seat 0 (verbatim):

`1791593626-human-29 seat 0, record to 18:14: frames 131 (finished 124, killed 5, unfinished at end 2), metal in unfinished frames 263, build orders never started 20`

floor.py gives `aband` 0 for seat 0, which equals frames.py's decayed count (0). No frame decayed in this game. Every unfinished frame was either killed by the enemy or still standing at the end.

## Tally

| Cause | n | metal |
|---|---|---|
| U4 the builder died, or the frame was killed | 5 | 190 |
| U8 game end | 2 | 73 |
| U0-U3, U5-U7, U9 | 0 | 0 |
| **total** | **7** | **263** |

U5 (economy) is the `also` cause on two of the U4 items: armrl_5197 and armllt_17783, 116 metal between them.

| Fate | n | metal |
|---|---|---|
| decayed | 0 | 0 |
| killed | 5 | 190 |
| at end | 2 | 73 |

| Type | n | metal |
|---|---|---|
| armllt | 3 | 130 |
| armmex | 3 | 77 |
| armrl | 1 | 56 |

No plant, nano, advanced solar, moho, fusion or silo frame was left unfinished.

What the player knew:
- **before (5):** the report told it the frame stood before the cause acted, for armrl_5197, armllt_17783, armllt_26740 and both frames standing at the end.
- **after (2):** it learned of armllt_20492 and armmex_1994 only from the fights line, "lost ... in our half". Each of those frames lived three seconds or less, between two turns.
- **never (0)** and **dropped itself (0).**

Never-started orders: 20 listed. 2 are skipped as long queues whose frame then started, which leaves 18 items:

| Cause | n |
|---|---|
| U6 the site | 7 |
| U8 game end | 5 |
| U3 the player re-listed | 4 |
| U1 the pick | 2 |

## Items, by metal put in

1. **15:05, seat 0, armllt_26740 at E4 (78%, 66 of 85 metal). Cause U4: the builder and the frame were both killed.**
   - *What happened:* constructor_3766 stayed on its list's Sentry at spot_37 while his main army (9 armstump, 2 armjanus, about 2.6k metal) crossed E4. Both the frame and the constructor (135 metal) died at 15:05.
   - *Evidence:* the gate read `constructor_3766.change` at 0.69-0.78 from 14:57, and the builder was hit from 15:01. Its best moves were go_home at 0.42-0.47 and go_spot_35 at 0.45-0.48. Neither entered a world, so constructor_3766 stood at 0 ("stay") in every world.
   - *What the player knew:* the 14:56 report read "building a Sentry (armllt) at spot_37 (E4) (50 % built) ... enemies_near: party_43 (2 armjanus, 9 armstump, 1 unidentified) 648 away ... contact in 9 s if it stays". The player sent both groups at the army and said nothing of the constructor.
   - *Fix class:* hands-rule. *Cost:* small (66), plus the constructor. *Confidence:* high.
2. **10:20, seat 0, armllt_17783 at E4 (71%, 60 metal). Cause U4, also U5.**
   - *What happened:* the list's Sentry at spot_37 was built with the energy store at 0-1 of 2600 the whole time. It reached 71% in 28 s, 2.5% a second against 3.75% at full rate, which would have finished it at about 10:15. A lone armstump came into sight at 10:16 and shot the frame from 71% to 7% by 10:19; it was destroyed at 10:20.
   - *Evidence:* the builder stood 120 away the whole time. The gate read change at 0.79-0.81 and no move rated over 0.40. The builder lived.
   - *What the player knew:* the 10:14 report read "building a Sentry (armllt) at spot_37 (E4) (67 % built)" and "energy 0/2600 (+2) STALLING". The threat was not yet in sight. After the kill, the 10:21 entry still read "(-16 % built)", a stale line.
   - *Fix class:* player (the stall). *Cost:* small. *Confidence:* medium.
3. **10:06, seat 0, armrl_5197 at G1, at home (70%, 56 metal). Cause U4, also U5.**
   - *What happened:* this was the first Nettle of the player's 9:33 list. It was built with the energy store at 0 (3.2% a second against 5% at full rate, so it would have stood at about 10:04). Five armthund bombed it at 10:06.
   - *Evidence:* the 10:03 queue-ahead did not move the builder: the frame kept rising, 62% to 70%. The pick's walk home at 10:05 (PICK `constructor_2839: took w2 of 1 at 0.58`, following the packet's "A constructor under bombers walks to home") came one second before the kill and did not cause it.
   - *What the player knew:* the 9:58 report read "building a Nettle (armrl) at spot_6 (G1) (45 % built)", "energy 0/2600 (-0) STALLING" and "enemy in sight: 3 at D1 ... armthund 3".
   - *Fix class:* player. *Cost:* small. *Confidence:* medium. That the finished Nettle would have survived five bombers is not shown.
4. **18:14, seat 0, armmex_7601 at D3 (82%, 41 metal). Cause U8.**
   - *What happened:* the frame was started 17 s before the record ended and was still rising 5% a second with its builder, constructor_19374, on it.
   - *What the player knew:* the 18:06 report showed it at 45%.
   - *Fix class:* none. *Cost:* none. *Confidence:* high.
5. **18:14, seat 0, armmex_9695 at G6 (65%, 32 metal). Cause U8.**
   - *What happened:* the frame was started 13 s before the end and was still rising with constructor_23822 on it.
   - *What the player knew:* the 18:06 report showed it at 27%.
   - *Fix class:* none. *Cost:* none. *Confidence:* high.
6. **4:18, seat 0, armllt_20492 at H3 (5%, 4 metal). Cause U4: the frame was killed.**
   - *What happened:* the player's 1:29 list queued this Sentry at spot_25. It was placed at 4:10 and started at 4:15 with two armflash 67 and 251 from the site, and was killed three seconds later. The pick walked constructor_28420 home at 4:19.
   - *What the player knew:* the 4:04 report gave the builder "enemies_near: party_12 (2 armflash) 737 away" and "extractors under threat: H3 (1 enemies, NO turret)".
   - *Fix class:* hands-rule. *Cost:* small. *Confidence:* high.
7. **6:14, seat 0, armmex_1994 at E3 (9%, 4 metal). Cause U4: the frame was killed.**
   - *What happened:* the list's extractor at spot_20 (from the player's 4:23 list) was started at 6:12 with party_19 (2 armflash) 130-290 from it, and was killed two seconds later.
   - *What the player knew:* the 6:10 report read "extractors under threat: E3 (2 enemies, NO turret)", and the builder was "237 to go ... enemies_near: party_19 (2 unidentified) 773 away". The player's 6:10 orders did not touch the builder.
   - *Fix class:* hands-rule. *Cost:* small. *Confidence:* high.

## Never-started orders (18 items, plus 2 skipped)

- **U6, rejected (5): two of the player's lists named the same spot.** At 4:25 constructor_28420 and constructor_493 both had spot_18; at 5:03 constructor_493 and constructor_2839 had spot_16; at 5:21 constructor_2839 and constructor_493 had spot_15. At 16:43 constructor_23822 and constructor_4602 had spot_41; at 17:05 constructor_23822 and constructor_31617 had spot_51.
  - In each case the sibling's frame started first, and the second builder got "the engine refused a armmex at ...: the site was bad".
  - Each refusal also skipped the turret step that followed ("skipped the step 'armllt spot_N' ... a defence there waits for it").
  - Also U3 (the player's overlapping lists). No metal was lost.
- **U6, no site (2): the home extractor at spot_6 (G1).**
  - 10:59 (constructor_3766, also U9): the engine gave `no_site`, but the step stood for 2:14 while the report said "walking to build a Metal Extractor (armmex) at spot_6 (G1), 212 / 617 / 735 / 736 / 295 to go". At 13:13 the watchdog said "never started in the 8 s it stood at the site". What blocked the site is not in the record: the bombed extractor's wreck, our Nettle 3850 and plant_20622's exit lane are the candidates.
  - 13:14: the same order was re-sent and got `no_site` again, and the list moved on.
  - spot_6 held no extractor from 10:07 to the end.
- **U3, the player re-listed (4):**
  - 2:36: the pick's extractor at spot_13 was replaced by the 2:29 list.
  - 5:35: the pick's Sentry at spot_25 was replaced by the 5:40 list.
  - 7:05: the first step of constructor_4602's list was replaced by the 7:01 list, which put solars first.
  - 16:34: constructor_14761's converter was dropped by the 16:27 list ("dropped the step 'armmakr' ... you have replaced or cancelled that list since").
- **U1, the pick (2):**
  - 10:03: constructor_2839's queued Nettle at spot_10 was given up when the builder walked home under the bombers. constructor_18057 built that Nettle from 10:26.
  - 15:03: the pick's Sentry at spot_6 was replaced 2 s later by the pick's own Sentry at spot_35. The PICK lines read "took w2 of 1 at 0.36", then "at 0.32".
- **U8, game end (5):** at 17:41, 17:59 (an armnanotc queued behind a Sentry), 18:03, 18:10 and 18:13. Each was queued behind a build under way or still a walk away when the record ended.
- **Skipped, two long queues that then started:**
  - 6:52: constructor_31617's extractor at spot_41 was queued behind a nano that the 6:46-7:57 energy stall slowed. The same builder started it at the site at 8:34.
  - 11:30: constructor_4602's Sentry at spot_32 started at the site at 13:10.

## The three mechanisms

1. **The energy store sat at zero.** It was empty from 8:17 to 11:15, 178 s (floor.py's `e0%` is 29.7 over the game). armrl_5197 and armllt_17783 were built at about two thirds of full speed. At full rate they would have stood about 2 s and 1 s before their killers arrived. The player was told "STALLING" at 9:58 and 10:14. Fixing this keeps up to 116 metal, if the finished turrets had then survived.
2. **A hit builder stays on its frame when no move clears the world bar.** The gate read change at 0.7 or more, but no move was offered in any world: armllt_26740's builder died with its frame, and armllt_17783's builder stood by while its frame was shot down. A rule here would have kept constructor_3766 (135 metal), not the frames (126).
3. **A refused or `no_site` extractor step stays standing.** This covers never-started 10:59 and 13:14 at spot_6. A constructor sat 2:14 under a "walking to build" line, and the home spot went empty for 8 minutes. The income lost is not measured here.

Smaller patterns:
- List steps start frames beside raiders: armllt_20492 and armmex_1994, 8 metal.
- Two of the player's lists named one spot: 5 orders refused and 5 turret steps skipped, 0 metal.

## Commands run

- `grep -n 'name=' MATCH/script.txt; ls MATCH/`, and the record's tail (orientation).
- `run/frames.py MATCH --seat 0`, and `--json`.
- `run/floor.py MATCH`.
- `run/commander_turns.py MATCH --calls`.
- `run/impedance_sweep.py MATCH --seat 0 --all` for 2:25-7:20, 10:55-13:20 and 14:55-18:14.
- `run/hands_window.py MATCH 14:55 15:06 constructor_3766` and `run/hands_window.py MATCH 10:03 10:07 constructor_2839`.
- Scratch readers in the scratch directory: `ev.py` (the record's events, commands, decision rows, samples and nearby enemies for given unit ids), `turn.py` (a turn's prompt lines), `ns.py` (each never-started order's 90 s) and `jw.py` (an actor's menus, gate ratings and worlds from jev-0.jsonl via run/jevlog.py).
- The energy-stall runs and the extractor events at each spot, read from the `s` and `ev` rows.

There was one seat, so no symlink directory was needed.
