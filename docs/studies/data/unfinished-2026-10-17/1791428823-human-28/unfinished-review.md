# Unfinished structures: human-28 (1791428823-human-28/00)

This was a 15-minute realtime 1v1 on Comet Catcher Remake 1.8 (2026-10-07). We had one seat (Opus 5.5 medium player under a commander, Jev 1.13.0). We played Armada against a person, [gecko]u6bkep (lobby skill 30), also on Armada. We lost the game: we conceded in the chat at 15:24 and the record ends at 15:32.

frames.py, seat 0 (the only seat):

    1791428823-human-28 seat 0, record to 15:32: frames 91 (decayed 6, finished 76, killed 9), metal in unfinished frames 1645, build orders never started 22

floor.py gives `aband` 6, which equals frames.py's decayed count. Its `aband_m` is 538, against 740 in frames.py's decayed frames; the two measure differently. This game has 15 unfinished frames, so it is at the cap rather than over it, and every frame got the deep read.

## Tally

| cause | n | metal in |
|---|---|---|
| U1 the pick sent the builder elsewhere | 3 | 158 |
| U3 the player re-listed or removed | 3 | 522 |
| U4 the frame killed / builder fled under fire | 8 | 458 |
| U8 game end (the surrender) | 1 | 507 |
| U0, U2, U5, U6, U7, U9 | 0 | 0 |
| **all** | **15** | **1645** |

Secondary causes (`also`):
- U1 on five U4 items.
- U9 on armvp_19699 and armllt_15126.
- U3 on armllt_419.
- U4 on armsolar_21368 and armadvsol_18655.
- U5 on armllt_12233.

| fate | n | metal in |
|---|---|---|
| killed | 9 | 905 (313 of it is our own reclaim of a duplicate plant, most of it returned) |
| decayed | 6 | 740 (507 of it is the last plant, self-destructed at the surrender) |
| unfinished at end | 0 | 0 |

| type | n | metal in |
|---|---|---|
| armvp (plant) | 2 | 820 |
| armadvsol | 2 | 308 |
| armsolar | 2 | 270 |
| armllt | 6 | 182 |
| armmex | 3 | 65 |

## Items, by metal in

1. **15:02-15:32 armvp_1000 (plant) at H8, constructor_22980 and three helpers, U8, 507 metal (86%).**
   - The third plant, #vp3 at spot_72. The player conceded at 15:24 and the bot sent `selfdestruct` to every unit at 15:27. The frame went at 15:32, the last frame of the record.
   - The player knew: the 15:14 report showed it "(17 % built) ... it will be plant_1000".
   - Fix: none. Cost: none.
2. **12:44-13:54 armvp_19699 (plant) at H3, constructor_29073, U3 also U9, 313 metal (53%), most of it reclaimed.**
   - The 12:18 queue gave three constructors `armvp spot_17 #vp2`, and all three lists landed on one tick (12:24). Each builder saw the id free, so two plants went up 270 apart: 29073's at 12:44 and 24942's at 12:46.
   - The player saw both in the 12:53 report (13% and 10%) and queued 29073 `["stop","assist"]`. Its `assist` step went straight back to plant_19699 (13:08 "help plant_19699 build").
   - At 13:12 the player sent `remove {"by":"constructor_29073","reclaim":["plant_19699"]}`. The "killed by armcv" is our own reclaim.
   - Fix: hands-rule. Same-tick id helping was built on 2026-10-12, after this game. Cost: large in builder time; the metal mostly came back.
3. **11:44-12:51 armadvsol_19202 at G2, constructor_15974, U4, 234 metal (67%).**
   - The second advsol of the 9:33 list. A metal stall (store at 0) held its first 18 s; after that it built at the lone-constructor rate of about 1.1% a second.
   - The builder stayed on it through his first pass into our base. His 10-unit block came back at 12:46 and killed it at 12:51, the same second the pick walked the builder to spot_12.
   - The player knew: the 12:37 entry said "(54 % built)".
   - Fix: none. Cost: large.
4. **6:54-7:09 armsolar_21368 at G2, the commander, U1 also U4, 135 metal (87%).**
   - At 7:01 the commander was unhit and about 2 s from finishing a 9 s solar. The pick walked it home: go_home 0.52 against attack_party_13 0.17. The move's words were "stepping away from party_13 (6 armflash) ... leaves the Solar Collector (armsolar) at about 80%, which decays".
   - The packet in force (6:16) said: "His Blitzes within its short walk ... it attacks them, using its D-gun". The six Blitzes killed the frame at 7:09.
   - The player knew only after: the 6:55 entry said "walking to build ... 146 to go", and the loss reached it as "lost armsolar" at 7:20.
   - Fix: hands-rule. Cost: large.
5. **9:56-11:03 armsolar_22394 at G2, constructor_9761, U3, 135 metal (87%).**
   - The pick put the newborn constructor on a solar at 9:54. The player's 9:47 list landed about 10:00 and correctly waited behind that build, queuing its first step at 10:13.
   - The 10:09 entry showed only "building a Solar Collector ... (47 % built) ... last_pick: 9:54". The player wrote "constructor_9761 picked up a solar instead. I resent both lists with a stop at the front".
   - The `stop` landed at 10:20 with the frame at 83%, and it decayed. The abandoned line later read "armsolar (1% built)".
   - Fix: report. Cost: large.
6. **12:49-13:19 armadvsol_18655 at G1, constructor_8662, U3 also U4, 74 metal (21%).**
   - The player's own 10:25 list started this advsol in the old base, with his block 707 away (the 12:53 entry said "(4 % built) ... contact in 7 s").
   - The player dropped it on purpose: the 12:53 queue began with `stop`. The builder was killed walking out at 13:15.
   - The frame went from 19% to 8% to gone in 2 s, at 13:17-13:19. That points to his unseen fire, not decay.
   - Fix: player. Cost: small.
7. **9:37-10:02 armllt_9191 at E1, constructor_8662, U4 also U1, 68 metal (80%).**
   - The list re-played `armllt spot_3` on the same site after the first turret there died. At 76%, with about 6 s left, the pick walked the builder to spot_6 ("leaves the Sentry (armllt) at about 70%, which decays").
   - Five Stouts killed the frame at 10:02. The player knew: the 9:47 entry said "(37 % built)", and its own 9:09 packet put his Stouts at spot_3.
   - Fix: player. Cost: small.
8. **8:53-9:16 armllt_6701 at F2, constructor_11105, U4, 60 metal (71%).**
   - The builder stayed on it. The energy store was at 0 from 9:03 to 9:11, which slowed it but did not decide it. Five Stouts killed the frame.
   - The player knew: the 9:09 entry said "(51 % built) ... party_22 631 away ... contact in 8 s".
   - Fix: none. Cost: small.
9. **5:56-6:14 armmex_20679 at F2, constructor_12512, U4 also U1, 36 metal (71%).**
   - The builder was already being hit when the pick walked it home at 6:10, as the 5:24 packet asked: "an enemy party within reach of a constructor: it walks back to home".
   - Two Blitzes killed the frame at 6:14 and the builder at 6:17. The player knew: the 6:05 entry said "(46 % built)".
   - Fix: none. Cost: small.
10. **5:14-5:31 armmex_26965 at F2, constructor_22980, U4, 28 metal (57%).**
    - A 7-Blitz raid killed the frame with the builder still on it, while group_F was on its way under the 5:24 packet.
    - The player knew: the 5:24 entry said "(41 % built) ... party_6 (7 unidentified) 791 away".
    - Fix: none. Cost: small.
11. **8:59-9:11 armllt_8713 at E1, constructor_8662, U4 also U1, 23 metal (27%).**
    - The player's list put the turret at spot_3 while his Stouts stood there. The pick walked the builder off at 9:10 ("leaves the Sentry (armllt) at about 30%"), and the Stouts killed the frame at 9:11.
    - The player knew: the 9:09 entry said "(26 % built) ... contact in 6 s".
    - Fix: player. Cost: small.
12. **14:37-15:02 armllt_419 at G4, constructor_24325, U1 also U3, 21 metal (25%).**
    - The pick walked the builder toward spot_72 at 14:44 ("leaves the Sentry (armllt) at about 20%, which decays"). The player's `stop` landed at 14:49 ("I dropped that build and sent it to the plant").
    - The player knew: the 14:44 entry said "(22 % built)".
    - Fix: none. Cost: small.
13. **11:34-11:49 armllt_12233 at F1, constructor_8662, U4 also U5, 8 metal (9%).**
    - The metal store was at 0 the whole time the frame stood, so it built at 1% a second. His main army arrived, the pick walked the builder off at 11:42, and the army killed the frame.
    - The player knew: the 11:37 entry said "(2 % built) ... party_27 ... 743 away".
    - Fix: none. Cost: small.
14. **14:29-14:40 armllt_15126 at G4, constructor_24325, U1 also U9, 2 metal (2%).**
    - The pick walked the builder off a second after the frame started. Its words took the build for unstarted: "drops the Sentry (armllt) it was going to build".
    - At 14:34 the list re-played the step, and the builder started a second frame 64 away (armllt_419) instead of resuming this one.
    - The player knew only after: the 14:44 report said "abandoned ... (0% built)".
    - Fix: hands-rule. Cost: none.
15. **7:25-7:36 armmex_6039 at F4, constructor_2206, U4 also U1, 1 metal (2%).**
    - The builder placed the frame the same second seven Blitzes started hitting it, and the pick walked it home. The builder was killed at 7:46.
    - The player knew only after: the 7:37 report said "abandoned an unfinished armmex at F4 (0% built)".
    - Fix: none. Cost: small.

## Orders never started

22 in all: 15 classified (the 15 costliest) and 7 skipped. The 15 split U3 7, U4 4, U1 3, U6 1.

- **U3, 7.** Each time, the player's next `queue` replaced the order before the builder got there, and meant to: 4:53, 6:30, 10:19, 13:04 (advsol), 13:07, 14:32 and 14:36. Two of these (4:53 and 14:32) were a spot on two lists at once, already taken by the other builder.
- **U4, 4.** The builder was walked home under fire and killed: 7:37 and 7:43 (the armnanotc), 13:08, and 14:39.
- **U1, 3.** The pick walked the builder home under the raid: 5:32 armllt (re-sent at 5:50), 5:54, and 6:02 armsolar. The 6:02 step named no place and was built at G2 at 6:24.
- **U6, 1.** At 14:49 the player gave the 25%-health constructor_29073 an unplaced `armsolar`. The hands put the site at spot_5, in the old base his army held, 2,900 away, although the packet asked for solars near spot_72. The player re-listed it at 14:54, and the builder died at 14:59.

Skipped:
- 10:13 and 10:21 (spot_42) and 10:57 (spot_48): re-sent and started at 12:17 and 12:43, after two minutes of picks walking them back from a lone Blitz.
- The three `#vp2` orders at 12:24: the plants were started 391-410 from the ordered point, beyond frames.py's 300 radius, so they are not lost orders.
- 5:50 spot_16: the cap; it is the same order as the 6:30 item.

## The player's share

- The report showed the player 12 of the 15 frames before the cause acted. It learned of the other 3 only afterwards, from an "abandoned" or "lost" line: armsolar_21368, armmex_6039 and armllt_15126.
- The player dropped 4 frames itself: armsolar_22394, armvp_19699 and armadvsol_18655 as the cause, and armllt_419 as a second cause.
- Its lists sent constructor_8662 twice to spot_3, where its own packet said his Stouts stood (armllt_8713 and armllt_9191, 91 metal).
- None of the 63 reports' 380 builder entries names the builder's list.

## The three mechanisms

1. **Same-tick id claim.** H-HANDS-LISTS `same_tick_help`, built on 2026-10-12, would have made the three `#vp2` builders one plant. It covers armvp_19699: 313 metal in, mostly reclaimed, plus 70 s of constructor_29073 and a slower plant_16631.
2. **Report.**
   - A builder's entry that names its list, and says the list waits behind the build in progress, would have kept the player from stopping constructor_9761's 83% solar: armsolar_22394, 135 metal.
   - An abandoned line that gives the share when the builder left, instead of when the frame vanished ("1% built" for an 87% solar), would have shown the size of the loss.
3. **Finish-first rule in the hands.** A pick that walks an unhit builder off a frame a few seconds from done, against a packet that asked it to stand and fight, should let the frame finish first. It covers armsolar_21368 (135 metal, the commander 2 s from done) and perhaps armllt_9191 (68 metal, about 6 s left with the Stouts about 4 s away).

## Commands run (from the repository root; M = run/matches/1791428823-human-28/00)

- Orientation: `grep -n 'name=' M/script.txt`, `ls M/record-*.jsonl`, `tail M/bot.log`, the record's last rows and chat events.
- `python3 run/frames.py M --seat 0` and `--all`, `python3 run/floor.py M`.
- `python3 run/impedance_sweep.py M --seat 0 --all`, `python3 run/commander_turns.py M --calls`, `python3 run/hands_window.py M 6:05 6:12 constructor_12512`.
- Scratch readers (in the scratchpad's `unfinished/1791428823-human-28/tools/`), all over `record-0.jsonl`, `strategist-0.jsonl` and `run/jevlog.py` rows of `jev-0.jsonl.zst`:
  - `rec.py`: unit rows, events, commands and decision rows in a window.
  - `jevwin.py`: an actor's menu with the gate's ratings, and the split pick.
  - `menuwords.py`: a move's full words.
  - `turn.py`: a turn's report lines.
  - `think.py`: a turn's assistant text.
  - `packets.py`: the packet in force.
- No symlinked seat directory was needed: there is one seat.
