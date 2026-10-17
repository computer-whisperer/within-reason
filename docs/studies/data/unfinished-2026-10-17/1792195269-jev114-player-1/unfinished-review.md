# Unfinished structures: 1792195269-jev114-player-1

This was an arena duel on Comet Catcher Remake 1.8, **won 19:09** against BARb hard_aggressive. It had one seat: the player was Opus 5.5 medium, the hands ran Jev 1.14.0, and the layers were news, same, fuse, openers, split, tick, places, again and near. There was no commander. The session was replaced twice, at 10:11 and at 16:52.

## Denominator (seat 0)

```
1792195269-jev114-player-1 seat 0, record to 19:09: frames 137 (decayed 7, finished 120, killed 2, unfinished at end 8), metal in unfinished frames 984, build orders never started 24
```
`run/floor.py` reports aband 7, which equals frames.py's decayed count. Seventeen frames were unfinished, which is over the cap of fifteen. Even so, every frame that was not U8 got the full read, and the eight U8 frames were checked against the reports.

## Tally

| cause | n | metal |
|---|---|---|
| U3 the player re-listed | 4 | 218 |
| U4 raided or fled | 3 | 74 |
| U8 game end | 8 | 420 |
| U9 harness fault | 2 | 272 |
| U0, U1, U2, U5, U6, U7 | 0 | 0 |

| fate | n | metal |
|---|---|---|
| decayed | 7 | 510 |
| killed | 2 | 54 |
| at end | 8 | 420 |

| type | n | metal |
|---|---|---|
| armadvsol | 3 | 312 |
| armsolar | 5 | 257 |
| armllt | 5 | 164 |
| armavp | 1 | 130 |
| armnanotc | 2 | 104 |
| armmex | 1 | 17 |

What the player knew: it was told about 10 frames before the cause acted (5 of them U8), it learned of 4 only from the after-line, and 3 it was never told about (all U8, started after the last report). It dropped 4 itself (U3, 218 metal). Every decayed frame got a line under "fights since last turn" ("abandoned an unfinished X (N% built) x1"), but N is the share left when the frame was destroyed, which was 0-1% every time. Only 4 of the 7 also got the line that names the cell.

## Items by cost

1. **13:28, seat 0, armadvsol_12108 (B8, 77%, 270 of 350), constructor_18284. U9.** The frame was the list step `armadvsol home #as6`, and at 60% the list queued "help build #as5" behind it. At 13:28 (f24246) armadvsol_12737 finished 200 away, and nine frames later the hands sent the builder to guard constructor_30757, its queued help. `Event::UnitFinished` ends or promotes every started build of the same type within 250 of a finished unit, so the neighbour's finish counted as this builder's.
   The player was told before: at 13:26 the report said "building a Advanced Solar Collector (armadvsol) at spot_74 (B8) (74 % built)". From 13:32 it said only "helping lab_30757 build", and the 13:39 armmakr list went out blind. The after-line came at 15:16: "abandoned an unfinished armadvsol (0% built) x1". Fix: hands-rule. Cost: large. Confidence: high.
2. **10:19, seat 0, armsolar_26962 (B8, 86%, 133), constructor_26737 (walled in since 6:50). U3, also U6.** The takeover session's 10:11 queue was `["stop", "extractor spot_45", "armllt spot_45"]` ("Stuck builders re-listed to spot_52/45"). The stop dropped the solar at 86%, and the builder never reached spot_45 (move_failed to 19:06). The player knew: the 10:11 report said "building a Solar Collector ... (55 % built) ... stuck: it cannot move". Fix: player. Cost: large. Confidence: high.
3. **3:02-3:06, seat 0, armsolar_25985 (C7, 53%, 82), constructor_18284. U3, also U1.** At 2:37, constructor_904's solar finished 97 away and closed this builder's task, the same handler as item 1. From then on the builder's entry read "finishing an order". At 2:58 the player queued `["stop", "extractor spot_69", ...]` (note: "metal 0, energy full ... Dropped constructor solars"). Before that landed at 3:05, the pick walked the builder home from a lone armflea at 3:02, with no "leaves the armsolar" in the move's words. The stop came at 3:06.
   The player was never told about the frame; the after-line came at 3:38: "abandoned an unfinished armsolar (1% built) x1". Fix: player (the report hid the share). Cost: small. Confidence: high.
4. **3:03, seat 0, armsolar_26776 (C8, 24%, 37), constructor_904. U4 (killed by the armflea at home).** The 2:08 packet said "Commander: a lone armflea or armpw at home or at spot_73 ... it attacks". From 2:35 to 3:03 the gate rated commander.attack_party_2 at 0.25-0.49, and no pick sent it. At 3:02 the pick walked the builder home ("leaves the Solar Collector (armsolar) at about 10%, which decays"). The player knew: the 2:58 report had the frame at 24% and "armflea 1; killing our Solar Collector (armsolar) (155 metal) now". Fix: hands-rule. Cost: small. Confidence: high.
5. **13:20, seat 0, armllt_23066 (C4, 23%, 20), constructor_23698. U4 (sent home under fire).** The frame came from a forward pick at spot_39. A lone armham hit the builder until it was at 4%, and the pick (go_home 0.53) said "leaves the Sentry (armllt) at about 20%, which decays". The home guard group_T1 had been walked off the armham at 13:14 by a pick at 0.02 confidence. The player knew: the 13:16 report said "(5 % built) ... health hurt (45%)". Fix: hands-rule. Cost: small. Confidence: high.
6. **18:51, seat 0, armmex_31508 (B3, 34%, 17), constructor_54. U4 (killed by an armpw).** The spot was far, 3,859 from home, with no turret ("B3 (1 enemies, NO turret)"). The hunters K2 and L2 started 2,000-3,300 away. The player knew: the 18:43 report said "(4 % built) ... party_50 (1 armpw) 95 away". Fix: player. Cost: small. Confidence: high.
7. **10:19, seat 0, armsolar_17595 (B8, 2%, 3), constructor_23698. U3, also U9.** The 10:11 queue (#as1, #as3) replaced the old list's unstarted solar at f18570, and the engine laid the old order's frame at f18582. The player knew only after: "10:30 abandoned an unfinished armsolar at B8 (1% built)". Fix: tool. Cost: small. Confidence: high.
8. **10:32, seat 0, armsolar_31890 (C7, 1%, 2), constructor_30757. U9.** At 9:00 the created-event handler took the nano frame 27075 as the new solar's frame ("building a Solar Collector (armsolar) at plant2_east (C7) (not yet started)"). That made the queue-ahead fire early at 9:32, and the C7 solar order ended up held by the engine alone after 10:14 ("finishing an order"). The builder laid the frame, walked off it with no command, and rebuilt it at 10:47 (armsolar_23071, finished 11:16).
   The player knew only after, and the line was misleading: "its builder was sent elsewhere". Fix: hands-rule. Cost: small. Confidence: medium.
9. **9:00, seat 0, armnanotc_27075 (C7, 0%, 0), constructor_30757. U3, also U9.** The player's 8:52 solar list replaced the queued second nano at f16200, and the engine laid that nano's frame at f16224. The player knew only after: "9:10 abandoned an unfinished armnanotc at C7 (0% built)". Fix: tool. Cost: none. Confidence: high.
10. **U8, the eight frames standing at the end (420 metal).** All started 18:42-19:07 and were still progressing at 19:09: armavp_30459 (C8, 5%, 130), armnanotc_17819 (C8, 45%, 104), armllt_17574 (D4, 77%, 65), armllt_1684 (E5, 71%, 60), armadvsol_2092 (B7, 10%, 35), armllt_21938 (D4, 15%, 13), armadvsol_8045 (B8, 2%, 7) and armllt_4648 (C3, 7%, 6). The 19:03 report named the first five, and the last three started after it. Fix: none. Cost: none. Confidence: high.

## Orders never started

Of the 24 orders never started, 23 are items and one is excluded: the 14:37 armmakr was refused at 14:59, re-sent, and started at 15:15. The JSON lists the fifteen costliest under `never_started` and the other eight under `never_started_beyond_cap`. By cause, 12 are U6, 5 U3, 4 U1 and 2 U8.
- **U6, stuck builders (11).** The commander, constructor_26737 and constructor_18784 were walled in by the solar field at spot_73. Their orders were the commander's armvp #plant2 three times (6:09, 6:30, 6:40), armlab #lab1 (7:29), solars at 8:38, 8:45 and 9:43, a turret at 17:00, and extractors at 6:39 and twice at 10:20. The watchdog lines came at 6:26, 9:26 and 10:15, and the moves failed to 19:01/19:06. The plant and lab orders are large: the bot never built a lab, and the second plant came from constructors (7:14-7:53).
- **U6, blocked site (1).** 18:07, constructor_23698's armnanotc sat beside constructor_18284's nano frame of the same type, "about to build" for 31 s, until the engine refused it at 18:38. The watchdog skips a site with a frame of the same type within 200.
- **U3 (5).** The player's lists replaced the order before the builder arrived: 8:33 (by the 8:52 list), 10:13 and 10:16 (by the 10:11 lists headed by `stop`), 11:39 (by the 11:36 queue) and 16:46 (by the 16:42 #vp3 queue).
- **U1 (4).** A pick replaced a pick: 11:39 (a retreat from a lone armpw), 11:59 (3 s later), 18:10 (mid-walk, swapped for a farther turret) and 18:27 (7 s later).
- **U8 (2).** 18:38 and 18:53, still walking when the record ended.

## The three mechanisms

1. **The finished-event match by type within 250** (`pianist/mod.rs` `Event::UnitFinished`). It covers armadvsol_12108 (270) and armsolar_25985 (82, where it hid the frame from the report and from the pick's words). Matching the builder's own frame by unit id would have kept 270.
2. **`stop`-first re-lists over started frames.** They cover armsolar_26962 (133; its builder was stuck, so its new step was unreachable) and armsolar_25985 (82). A queue result naming the share it drops, or a `stop` that spares the frame a stuck builder stands at, would have kept up to 215.
3. **Lone raiders where the packet's answer did not land.** They cover armsolar_26776 (37; the commander's flea clause rated 0.25-0.49), armllt_23066 (20; T1 walked off by a 0.02 pick) and armmex_31508 (17; far, no turret): 74 in all.

A related hands fault: the created-event handler takes any frame the builder makes as its build. This is behind armsolar_31890 (2), and the same tick race left armnanotc_27075 and armsolar_17595 (3).

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
run/frames.py MATCH --seat 0; run/frames.py MATCH --seat 0 --json; run/floor.py MATCH
run/hands_window.py MATCH 2:55 3:10 constructor_18284 constructor_904; MATCH 13:12 13:24 constructor_23698
run/impedance_sweep.py MATCH --seat 0 --all --from 1:50 --to 3:45 / 8:50-9:15 / 9:40-10:50 / 11:50-13:50 / 11:20-11:50 / 14:28-15:10 / 16:38-16:56
run/commander_turns.py MATCH --calls
python3 over jevlog.rows (pass menus and stay words at 3:02, 13:20, 11:42; worlds_gate commander flags 2:35-3:04; menu courses of
  constructor_18284/904 2:10-3:10 and constructor_30757 8:58-10:50)
python3 over record-0.jsonl (cmd/ev/d rows and s positions for the builders and frames named above; armsolar and armadvsol
  created/finished/placed events 8:58-10:20 and 12:00-15:15; unit costs from the header)
python3 over strategist-0.jsonl turn prompts (builder entries, abandoned and fights lines, the 2:08 packet)
crates/bot/src/brain/pianist/mod.rs (Event::UnitCreated, Event::UnitFinished), menu.rs (builder_course, leaves_words)
```
There was one seat only, so no symlink directories were needed. MATCH = run/matches/1792195269-jev114-player-1/00.
