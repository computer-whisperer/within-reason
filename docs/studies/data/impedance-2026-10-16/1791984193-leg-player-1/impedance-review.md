# Impedance review: leg-player-1, match 00

**The game.** WON 25:54 as Legion (our first Legion seat) against BARb hard_aggressive (Armada) on Comet Catcher Remake 1.8; one seat (strategist-0), the player Opus 5.5 medium, no commander; layers news, same, fuse, openers, split, tick, places, again, near; think penalty 1, capped at 7 s (every order landed +7 s). Reviewed 2026-10-16 by Claude Opus 5.5 for the impedance tally; one seat, so no symlink directories were needed.

## Tally

| Category | Count | Of which cost large / small / none |
|---|---|---|
| A1 refused at the tool | 0 | |
| A2 accepted, not executed | 5 | 0 / 3 / 2 |
| A3 overruled by the pick | 9 | 1 / 6 / 2 |
| A4 overruled by code | 6 | 0 / 6 / 0 |
| A5 late | 0 | |
| A6 read differently | 4 | 0 / 4 / 0 |
| B1 in the hands' picture, not in the report | 1 | 0 / 1 / 0 |
| B2 in the record or truth, in nobody's picture | 0 | |
| B3 in the report, wrong or stale | 2 | 0 / 0 / 2 |
| B4 in the report, missed by Opus | 5 | 0 / 0 / 5 |
| B5 the hands' lines misleading | 0 | |
| B6 commander-to-player hop | 0 (no commander) | |
| **C: judgement moments clean** | **0 of 8** | |

Denominators: 8 judgement moments; 295 orders (143 of them `wait`), 0 refused; 1,127 hands' lines, 108 flagged; 143 turns, 4 session replacements.

The eight moments, each with what kept it from C:

| # | Moment | Items |
|---|---|---|
| a | Opening, 0:00-4:00 | Lists exact (commander's 11 steps, the plant's counted entries in order). A3 1:34 (constructor helped the plant against the packet) |
| b1 | Tick at home, 2:04-3:07 | A6 2:24 (first Alaris in the Wheelies' rove group), A3 2:06 (commander's D-gun clause, low confidence) |
| b2 | Pawn party, 6:11-6:46 | Answer as written at 6:28; A3 6:37 (group_K walked away) |
| b3 | Centurion/Rocketeer block, 11:14-11:29 | Hands followed the packet's outweigh clause (C for the hands); B3 + B4 at 11:26; B1 (metal full) through the window |
| b4 | spot_46, 20:23-20:36 | A6 (place read 0.44; no group answered) |
| c | First window 7:00-8:00 and `go_east` | `go_east` played at once (8:30); A6 ("join each other" fused off), A3 none (spot_61) |
| d1 | Packet of 9:42 (a third in) | Push at 10:11 played as written; A3 none at 9:53 |
| d2 | Packet of 17:16 (two thirds in) | A3 large (17:24-18:27), B4 17:16 |

## Items, ranked by cost (15 of 32; the rest are in impedance.json)

1. **17:24-18:27, A3, large (high).** Wanted (17:16, again 17:36/17:46/17:56): "It advances (fight to) along this way ... spot_18, then spot_10, then his_lab"; "its next move is the advance (fight to) spot_18". Happened: group_A2 kept "task fight_to spot_27" under the unseen Ambusher until "18:27 PLAYS group_A2: walk to spot_29". Evidence: decode spot_18 0.98-0.99 (landed); best rating "fight_to_spot_18=0.39"; "PICK split [group_A2: kept w1 over w2 of 1 at 0.8]". Cost: 9 Decurions (2,700 metal) died in place in those 63 s (spot_18 lay in more turret reach, so the cost is an upper bound). Fix: a move the packet can force (Opus asked for it at 18:06).
2. **11:09-13:56, B1, small (high).** The picture said "metal: 2546 of 2550 stored (full: what comes in beyond this is wasted)"; the report's eco line said "metal 2548 (+59.4/-35.1)" with no cap, while energy shows "166/2900". Opus wrote "Energy 166 is still the cap on spending 2548 metal" and never saw that income was thrown away. Metal sat at 98% or more of its cap 148 s in minutes 11-13: 2,213 metal of income over use (an upper bound on the waste). Fix: report.
3. **15:41-25:54, A3, small (medium).** Wanted: lists ending in an uncounted "leggat" and "Plants: make what their lists say, in order." Happened: the pick declined "build a leggat" on many asks ("20:33 PICK split [plant_27255: kept w1 ... at 0.5; plant_16133: kept w1 ... at 0.6; plant_1544: kept w1 ... at 0.5]"). Plants idled 351 plant-seconds, 144 of them with metal over 300: about 4 Decurions. Opus saw it only at 24:09. Fix: tool (an uncounted entry as a standing order).
4. **20:21-20:46, A4, small (high).** Wanted: "its way is spot_55, then spot_57 ...". Happened: "at front at spot_55 ... its front 22 to go, 15 of 40 arrived, its tail 3030 short, for 31 s". The stop never counted as reached until Opus re-sent the route; it was picked at 20:48. Fix: code-rule (arrival by the body's core within 300).
5. **21:16-21:34, A4, small (high).** The same at spot_57: 36 s, picked 2 s after the re-sent route landed ("21:34 PLAYS group_A2: advance to spot_42"). Fix: code-rule.
6. **21:59-22:25, A3, small (medium).** Wanted (21:51): "it has passed spot_55, spot_57 and spot_42. Its way now is spot_34". Happened: it held "fight_to spot_42" with "go_spot_34=0.22", and the news layer closed it for 6 s. layers_read shows "22:19 group_A2.go_spot_34: assumed its opener says no, asked again 0.62". It walked at 22:25, 26 s after the packet. Fix: jev, plus the openers layer.
7. **24:27-24:50, A3, small (high).** Wanted (24:19): "Its move now is the attack on his buildings at his_lab". Happened: "kept w1" (move_to spot_17) for 23 s, "go_his_lab=0.43..0.47", forbidden detachments rated 0.64. Fix: jev.
8. **20:23-20:36, A6, small (medium).** Wanted (19:56): "a party of his that comes onto our side near ... spot_46 ... it fights whole first". Happened: Fatboy, Fido and 2 Centurions, then a lone Centurion, stood 330-650 from spot_46; "attack_party_58" was rated 0.28-0.48 and never picked; spot_46 died at 20:36 and was not rebuilt. The decode read group_A2|place|spot_46 at 0.44. Fix: words.
9. **7:37-8:30, A6, small (high).** Wanted (7:29, 7:58): "they join each other into one body, gathering at spot_61". The decode read the joins at 0.08-0.24 and group_B's spot_61 at 0.29, all fused off. The fuse audits found them wrong ("7:54 group_R.join_group_S: assumed off, asked again 0.51"; 7:56 0.54). At `go_east` (8:30) only group_B (5) and group_R (6) went: group_O (5-6) held at spot_61, and one soldier walked alone to spot_20. It recurred at 9:50. Fix: words (one direction: "X joins group_R", as Opus wrote at 10:01).
10. **23:40-24:05, A2, small (high).** Wanted (22:01): constructor_8343 "extractor spot_14", "leglht spot_14". Happened: "the engine refused a legmex at A2" because another builder's frame held the spot, then "skipped the step 'leglht spot_14' ... the extractor at spot_14 was refused lately". The extractor stood from 23:48 with no turret and died to a Tick at 24:05; it was not rebuilt. Fix: code-rule.
11. **11:46-13:21, A4, small (medium).** Wanted (every packet from 9:22): "No constructor stands or builds in a plant's exit lane." Happened: code placed the listed advanced solars ("legadvsol #as1".."#as7", no place) in plant_16133's and plant_27255's lanes. Three frames were abandoned (12:58, 12:58, 13:21). Fix: code-rule.
12. **12:13-12:23, A6, small (high).** Wanted (12:06): "legadvsol solar_field #as3" (and #as4, #as5), to move off the lanes. Happened: "12:15 constructor_1281: help build #as3 (repair)": the ids named the lane frames, so the builders stayed in the lanes until the 12:16 re-list. Fix: tool (warn when an id already names a frame elsewhere).
13. **20:55-21:53, A2, small (high).** Wanted (20:48): commander "legavp #avp1", "assist", which by the tool means "any other helps build it". Happened: "20:56 commander: skipped the step 'legavp #avp1' of its list: this builder cannot build it", and its 'assist' was dropped. The advanced plant was 49% built at 20:58 and stood at 21:53 without the commander's 300 build power. Fix: tool.
14. **16:27-18:01, A4, small (high).** Wanted (16:18): group_L1/Q1/Y1 "are the raider hunters ... Any armpw ... at our extractors or constructors anywhere on our side". Happened: four L1 hunts of lone Pawns "ended after 0 s / 2 s: the leash of 900 from where it began reached", and the hunters joined group_Y1. The leash failure "is remembered 3 minutes" (docs/code-decisions.md). constructor_5624 and spot_58 had died the same second (16:28). What the leash cost was the Pawns living on: party_54 drove constructor_2996 off its list at 18:00. Fix: code-rule.
15. **3:07-5:43, A4, small (medium).** Wanted (2:34, 5:22): the Tick at home "is killed by the nearest group of leghades". Happened: four hunts of party_1 ended on the leash (3:07, 5:15 twice, 5:32). The Tick lived to 5:43 and killed spot_50 at 5:04. Fix: code-rule.

The other 17 items: A4 23:43-24:05 (the leash ended the spot_14 Tick's hunts), A3 1:34, A6 2:24, A3 2:06 (low), A3 6:37, A3 9:53 (none), A3 8:06 (none), A2 spot_36 and spot_55 turret skips (none), A2 legnanotc 5:25 (small), and the B items below.

B items, all cost none:
- B4 5:12: the roster listed legnanotc's makers without legcom.
- B3 11:26: party_30 showed only its 660 in sight while 1,550 stood within 1,500 and the picture had seen 1,550 five seconds before.
- B4 11:26: Opus blamed flee footwork for "11:20 group_R: walk to spot_59 (plan)", which was its own outweigh clause.
- B4 16:28: it listed spot_26 and spot_24, which the score line named as his.
- B4 17:16: "It has reached spot_27" against "7 of 22 arrived".
- B4 21:24: "a stop never counts as reached until it is exact" against "15 of 40 arrived".
- B3 25:52: party_95 still listed his commander beside "killed armcom x1".

Added up, the four march stops (items 4-7) are 116 s of a 17-26k army parked between 20:21 and 24:50.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 143 turns, 4 session replacements, 295 orders, 0 refused, 108 flagged of 1127 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 143, 'instruct': 59, 'queue': 47, 'produce': 26, 'lane': 9, 'signal': 8, 'mark': 2, 'remove': 1}
   flagged: {'step-skipped': 32, 'hunt-ended-leash': 23, 'hunt-ended-other': 18, 'stopped-on-stop': 13, 'engine-refused': 12, 'step-waits': 4, 'other-fault': 4, 'abandoned': 2}
```

How the flagged lines were judged:
- **engine-refused (12).** Seven are extractor steps on spots where another builder's frame already stood. The extractor went up each time, but the refused builder's turret step was then skipped. That is the A2 items at spot_14, spot_36 and spot_55; at spot_71, spot_50 and spot_79 another builder built the turret. Five are advanced solars at the solar_field mark, each re-placed and begun within about 3 s (no item).
- **hunt-ended-leash (23).** These are 14 events.
  - Seven are in three items: the Tick 3:07-5:32, L1 16:27-18:01 and spot_14 23:43-23:54. Even there, group_K re-attacked at 5:33, but the Tick lived.
  - At five, the pick re-gave the attack or task within a second, with cost none and no item: 5:15 group_E, 9:52-9:53, 12:24, 12:38, 14:30.
  - 6:54: the detachment rejoined group_B, as the 6:38 packet said.
  - 19:30: the hunt was on ground the packet forbade (spot_31/37).
- **The rest.** The step-skipped lines are mostly "holds our extractor already". stopped-on-stop and step-waits are the player's own asks. Both abandoned frames followed the player's own re-lists.

**A5.** The five sessions (0:00, 9:32, 16:08, 19:39, 23:01) are not a lateness source in an arena game. The game pauses while the player thinks, each takeover's orders landed at the usual +7 s, and each takeover note carried the plan forward. bot.log ties the 19:39 and 23:01 sessions to the brief being edited on disk mid-game. That is a process fault, not an impedance item.

**B2.** The F2 Ambusher that shelled spot_27 was never seen by our units (no `enemy_seen` for armamb), so it is not a B2.

## What Opus wanted that the harness has no lever for

- **18:06, a direct move order.** "I want a hard move order for a group that bypasses Jev's rating" (again 22:11 "Wish: a direct move order for a group", 24:37). Packets 17:36-18:23 said "its next move is the advance (fight to) spot_18 ... that is the move it takes whenever it stands anywhere short of spot_18". No tool sets a group's move. The packet only weights a rating, and a named move rated 0.39 stays unplayed.
- **21:24, when a stop counts as reached.** "Wish: a group 'reached' within ~150 of a stop" (22:11, 24:37). Arrival is code: the body's core within 300. The only lever is re-sending the route from the next stop.
- **9:22, several groups arriving together.** "all fight it together, as one body arriving at once ... a group that arrives first waits at spot_49 for the others" (9:42, 10:01, 10:49). No move synchronises several groups. Joining is the only lever, and "join each other" read as no join.
- **23:11 and 25:29, which unit in a party to shoot first.** "it attacks that party whole and kills the armrectr, then goes on"; "every soldier shoots the armcom until it is dead". The group menu has attack, shell and send against a whole party; no move names the unit inside it.
- **9:22 onward, exit lanes.** "No constructor stands or builds in a plant's exit lane." Code placement of a list step without a place, and where an assisting builder stands, do not read the packet. The workaround is a marked place on each step (12:06 solar_field).
- **15:41, uncounted produce entries.** "Plants: make what their lists say, in order." with an uncounted last entry. An uncounted produce entry is a per-ask pick, never standing. Counted entries are the workaround (24:09).

## Commands run

```
cat run/matches/1791984193-leg-player-1/results.jsonl
python3 -c "import json;print(json.load(open('run/matches/1791984193-leg-player-1/batch.json')))"
python3 run/impedance_sweep.py run/matches/1791984193-leg-player-1/00 --counts
python3 run/impedance_sweep.py run/matches/1791984193-leg-player-1/00 --seat 0
python3 run/minutes.py run/matches/1791984193-leg-player-1/00
python3 run/wake_read.py run/matches/1791984193-leg-player-1/00
python3 run/raid_ledger.py run/matches/1791984193-leg-player-1/00
python3 run/commander_turns.py run/matches/1791984193-leg-player-1/00 --calls
python3 run/commander_turns.py run/matches/1791984193-leg-player-1/00 --notes
python3 run/layers_read.py run/matches/1791984193-leg-player-1/00 --faults
python3 run/hands_window.py run/matches/1791984193-leg-player-1/00 M:SS M:SS [actor]
    1:24-2:04 constructor_17003; 2:04-2:58; 2:36-3:00; 6:10-6:52; 7:30-7:46; 7:39-8:30 group_B; 8:30-9:12;
    9:40-10:02; 10:16-10:50 group_R; 11:12-11:34 group_R; 12:22-12:27; 12:36-12:41; 14:28-14:33;
    17:20-18:00 group_A2; 18:12-18:36 group_A2; 20:12-21:30 group_A2; 20:22-20:40 group_A2 group_L1;
    21:28-21:40 group_A2; 21:58-22:30 group_A2; 22:19-22:52 group_A2; 24:20-24:52 group_A2
```

Also small read-only scripts in the scratch directory, all reading files of this match:
- `dump_calls.py`: every order's full text.
- `extract_jev.py`: a compact copy of jev-0.jsonl.zst through run/jevlog.py.
- `turn.py`: a turn's prompt.
- `fixed.py`: the step-4 check, which sets the truth file's enemy within 1,500 of each group beside the report's enemy, eco and hands' lines, and the hands' lines beside the log's plays.
- `rec.py`: record events, plant idle seconds, metal at the cap, our losses per window.
- The decode `reads` per packet.
- One `call` row's `state` (11:19 group_R entry, 11:36 economy).
- `git show 7a94e80:` of `crates/bot/src/brain/pianist/{mod,decode}.rs` and `crates/bot/src/strategist/mcp.rs` for the decode, the fuse bar and the tools' words.
