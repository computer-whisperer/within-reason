# Impedance review: 1791600181-glitters-8v8-late-1

An 8v8 arena game on All That Glitters v2.2.3, played in realtime on 2026-10-09. Eight Sonnet 5.5 medium players (seats t0 to t7) played under an Opus 5.5 high commander against eight BARb hard_aggressive. We **lost at 28:30**; the commander surrendered at 28:15. This is the only game in the tally where the player is Sonnet, so "the player" below means Sonnet; the commander is Opus.

## Tally

| category | count | | category | count |
|---|---|---|---|---|
| A1 refused at the tool | 17 | | B1 in the hands' picture, not the report | 1 |
| A2 accepted, not executed | 4 | | B2 in nobody's picture | 0 |
| A3 overruled by the pick | 2 | | B3 report wrong or stale | 2 |
| A4 overruled by code | 0 | | B4 in the report, missed | 4 |
| A5 late | 0 | | B5 hands' lines misleading | 0 |
| A6 read differently | 0 | | B6 commander-to-player hop | 2 |
| **C clean** | **3 of 12 moments** | | | |

Denominators:
- 12 judgement moments.
- 2,144 orders. The sweep counts `wait` among them.
- 5,376 hands' lines, 701 of them flagged.
- 17 refusals. The sweep's REFUSED pattern finds 8. The other 9 are refusals over the shape of the arguments ("arguments: lists are by builder name ...", "a grid cell is A1 to H8") and corrected copies sent after `wait`, which were dropped with "Your turn is over". All 9 are tallied here and listed in `impedance.json` as `refused_missed_by_sweep`.

## The moments examined

I made symlink directories for seats 1, 5 and 6 under the scratch directory. The seat-0 tools (hands_window, raid_ledger, pro_baseline, the decode reader) ran on those directories.

| # | Seat | Moment | Result |
|---|---|---|---|
| 1 | t1 | (a) opening, 3:12 extractor lists | B4 |
| 2 | t1 | (b) raid at spot_70, 6:34 | C: group_C attacked party_90 with the whole group, as the packet said |
| 3 | t1 | (c) push_centre given at 9:49 | A3 |
| 4 | t1 | (e) last turns, 27:45 to 28:21 | C (the 28:10 and 28:21 transfer refusals are A1, counted separately) |
| 5 | t5 | (c) east push, 7:16 to 8:46 | C: the hands went to spot_91 and then spot_98 as one body and came back on the 8:26 packet |
| 6 | t5 | (d) the main army handed over from t3, 15:00 | B6 |
| 7 | t5 and commander | (d) strike on his advanced lab at F5, 16:15 to 17:19 | The hands followed (fight_to spot_83 with every party fought on the way, as written); B3 at the commander |
| 8 | t6 | Nettles ordered by the commander, 4:54 to 6:21 | A2 + B4 |
| 9 | t6 | Retaking home spots with advanced constructors, 14:16 to 22:55 | B1 |
| 10 | t6 | (e) last turns, 27:38 to 28:26 | A2 (B1 repeats) |
| 11 | commander | t6's army given to t5, 12:55 | B4 |
| 12 | commander | Convergence at centre_hold, then the army_meet attack, 22:15 to 24:47 | B6 + B3 |

## Items ranked by cost

1. **12:55, commander, B4, cost large (confidence medium).**
   - Wanted: the commander told t6 "You keep group_K1 at home; group_Y went to t5's east army", and transferred group_Y_t6.
   - What happened: group_K1 belonged to t5. In the commander's own picture, t6 had only group_Y_t6 (10 soldiers). t6 was left with no army. From 13:20 to 13:43, 24 corak plus 3 corstorm took spot_56, spot_57, spot_48 and spot_65. Raid_ledger shows no soldier answered and none of the four was rebuilt.
   - Fix: brief (the commander checks a seat's groups before it transfers them).
2. **16:15, commander, B3, cost large (confidence medium).**
   - Wanted: strike his advanced lab, which the commander described as having "only two light turrets beside it", on the reading "Armies are even (20,350 against 20,200 seen)".
   - Truth: 89 of his units stood within 900 of the lab at 16:15 and 104 at 16:35, including 16 Thugs, a corhllt, a corhlt and a cormaw. His whole army was 28,420 at 16:00.
   - What happened: group_D2_t5 went from 40 to 24 by 17:19 without reaching the lab.
   - Caveat: the score line did say "may be much more".
   - Fix: report.
3. **23:35, commander, B3, cost large (confidence medium).**
   - Wanted: "his block of about 70 at E3-F3, about 10,000-12,000 seen". The commander gave four seats' bodies to t6 to attack it.
   - Truth: 166 units within 1,500 of E3/F3 at 23:35, and 151 near army_meet at 24:20, including a corsumo and 2 corhrk.
   - What happened: group_P1_t6 went from 62 to 44 in 25 s (24:13 to 24:38).
   - The report's "70 at E3" was true to what our units saw: this gap is fog.
4. **9:49, t1, A3, cost small (confidence high).**
   - Packet (8:54): "when push_centre is given it fights as one body to spot_97, spot_106 ... nobody chases aircraft". The decode read spot_97 at 0.6 and spot_106 at 0.64, so those marks landed. The hands' picture showed `"push_centre":"given, at 9:49"`.
   - Plays instead:
     - "9:49 walk to spot_51"
     - "9:51 attack party_180 (1 corfink) with the whole group"
     - "9:52 attack party_183 (1 corshad)"
     - then a chase of party_187 west into C4
   - The group went from 12 to 9 soldiers and never moved toward spot_97.
   - Fix: jev.
5. **3:12, t1, B4, cost small (confidence high).**
   - Wanted: 12 extractor steps; 11 were skipped as "spot_X is not free". The records show every one of them held by an ally (t3, t4, t5, t6) since 0:16 to 2:57.
   - The report had "nearest on foot: #33 D2, #38 G2, #43 F3, #51 F3, #48 G3".
   - Seat 1 stayed at 6 extractors from 3:01 to 5:23.
   - Fix: report. The spot list should mark allied extractors, and the skip line should name the ally.
6. **9:01, t5, B4, cost small (confidence high).**
   - Woken by "no growth ... 49 free spots we can walk to", the player queued 21 steps on home-half spots. 14 were skipped as "not free". All were allied-held since 0:19 to 5:00.
   - The score line named #72, #75, #66, #71 and #50.
   - Seat 5 stayed at 7 extractors until about 11:00.
   - Fix: report.
7. **14:16 to 28:16, t6, B1, cost small (confidence high).**
   - About 35 tier-1 steps (extractors, Sentries, solars) were queued on armacv builders and skipped with "this builder cannot build it". This happened six times; at one point the player put armmoho and then extractor lists on the same builders.
   - The report entry "constructor_10908: at spot_13 (H1); helping ..." never names the kind. The hands' state does: `"is":"Advanced Construction Vehicle (armacv ...)"`.
   - Fix: report.
8. **6:59, t5, A3, cost small (confidence high).**
   - Packet: "group_O joins group_I at once; group_I ... is the east body".
   - Play: "group_I_t5: join group_O_t5". The decode read O-into-I at 0.65 (Opus's mark landed) but also I-into-O at 0.51.
   - The body's name pointed at nothing. Detachments and merges then renamed it O, R, S, U and W by 7:11. The player re-wrote the packet for group_W at 7:16.
   - Fix: jev.
9. **22:15 to 23:35, t6, B6, cost small (confidence medium).**
   - The commander's "go to centre_hold now and fight the block with t4, t3 and t1" was not carried out. t6's note: "holds at east_hold until it is clear t4/t3/t1 are actually converging".
   - A seat's report shows no allied group.
   - The commander at 23:35: "Every seat waited for the others ... so nobody went".
   - Fix: report.
10. **5:24, t3, A1, cost small (confidence high).**
    - `signal go_south` inside an orders batch was refused: "not a tool that can be batched". The same turn's packet said "go_south has been given".
    - The signal was given 21 s later, at 5:45.
    - Fix: tool.
11. **12:28 and 12:37, t1, A1, cost small (confidence medium).**
    - Sentries were wanted at home against a raid of 18. The queue came twice as `{"arguments": "<string>"}` and was refused; the corrected copy came after `wait` and was dropped. One step entered at 12:45, 17 s late.
    - Fix: tool.
12. **21:10, t6, A1, cost small (confidence medium).**
    - Produce lists for three idle plants came as a string and were refused; the corrected copy after `wait` was dropped. The lists entered at 21:23.
    - Fix: tool.
13. **25:45, t5, A1, cost small (confidence low).**
    - `mark {"places": "..."}` for north_fall was refused ("a grid cell is A1 to H8") while the packet sent the commander and the groups there. The corrected copy was dropped after `wait`; north_fall was marked at 25:57.
    - Two constructors died at 25:58 and 26:03.
14. **21:39, t5, A1, cost small (confidence low).**
    - The queue that gave four builders one shared plant (`#vp1`) was sent after `wait` and dropped.
    - At 23:02 to 23:03 the engine refused three builders' armvp at H2: "the site was bad".

The other 18 items cost nothing; all are in `impedance.json`:
- 11 A1:
  - commander transfers at 11:15 (group_F1_t2) and 14:35 (group_B1_t3). Each named a group that was gone by the end of a realtime turn of 33 to 39 s; each was re-issued at once.
  - t1 transfers at 28:10 and 28:21; t1 mark at 22:58.
  - t6 signal at 23:52; t6 queues at 11:20 and 11:40.
  - t7 removes at 7:04 and 9:04, both by "commander".
  - t3 queue at 18:01.
- 4 A2:
  - armrl and armrad steps with no place, skipped as "it needs a place": t1 at 4:39 and t6 at 5:03.
  - t1's six armllt engine refusals. Each is the turret step after an extractor step skipped for an ally-held spot.
  - t6's armvp at spot_10 at 28:08.
- 2 B4 and B6: t6 re-queued placeless armrl at 5:39 and 5:46; t5's 15:00 "no group_L/group_B1 of t3 appeared in my picture" (t3's group arrived as group_D2_t5 with no provenance).

The commander's realtime turns took 22 to 43 s of wall time. Each direction landed 15 to 30 s after the report it was written from. I found no moment where that delay measurably changed the outcome, so there is no A5 item.

## The sweep's counts per seat (verbatim)

```
-- strategist-0.jsonl: 159 turns, 2 session replacements, 252 orders, 0 refused, 72 flagged of 691 hands' lines; landing median 0 s (p90 0), turn wall median 6 s
   orders: {'wait': 158, 'instruct': 46, 'queue': 34, 'produce': 14}
   flagged: {'step-skipped': 42, 'stopped-on-stop': 9, 'hunt-ended-other': 5, 'step-waits': 4, 'abandoned': 4, 'engine-refused': 3, 'hunt-ended-leash': 3, 'follow-ended': 1, 'site-blocked': 1}
-- strategist-1.jsonl: 153 turns, 3 session replacements, 257 orders, 2 refused, 101 flagged of 660 hands' lines; landing median 0 s (p90 0), turn wall median 7 s
   orders: {'wait': 154, 'instruct': 44, 'queue': 43, 'produce': 13, 'transfer': 2, 'mark': 1}
   refused: {'transfer': 1, '?': 1}
   flagged: {'step-skipped': 71, 'hunt-ended-leash': 7, 'stopped-on-stop': 7, 'engine-refused': 6, 'step-waits': 5, 'hunt-ended-other': 4, 'site-blocked': 1}
-- strategist-2.jsonl: 162 turns, 3 session replacements, 242 orders, 0 refused, 51 flagged of 596 hands' lines; landing median 0 s (p90 0), turn wall median 6 s
   orders: {'wait': 158, 'instruct': 46, 'queue': 29, 'produce': 8, 'mcp__wreason__wait': 1}
   flagged: {'step-skipped': 32, 'hunt-ended-other': 6, 'hunt-ended-leash': 5, 'site-blocked': 3, 'other-fault': 2, 'stopped-on-stop': 1, 'engine-refused': 1, 'step-waits': 1}
-- strategist-3.jsonl: 170 turns, 4 session replacements, 290 orders, 1 refused, 81 flagged of 702 hands' lines; landing median 0 s (p90 0), turn wall median 6 s
   orders: {'wait': 171, 'instruct': 56, 'queue': 41, 'produce': 15, 'signal': 2, 'lane': 2, 'mark': 2, 'remove': 1}
   refused: {'signal': 1}
   flagged: {'step-skipped': 44, 'engine-refused': 6, 'hunt-ended-other': 6, 'site-blocked': 5, 'stopped-on-stop': 5, 'other-fault': 4, 'abandoned': 4, 'step-waits': 3, 'hunt-ended-leash': 2, 'follow-ended': 2}
-- strategist-4.jsonl: 161 turns, 4 session replacements, 272 orders, 0 refused, 101 flagged of 694 hands' lines; landing median 0 s (p90 0), turn wall median 6 s
   orders: {'wait': 163, 'instruct': 50, 'queue': 43, 'produce': 16}
   flagged: {'step-skipped': 64, 'site-blocked': 9, 'stopped-on-stop': 6, 'hunt-ended-other': 6, 'other-fault': 5, 'abandoned': 5, 'step-waits': 2, 'engine-refused': 2, 'hunt-ended-leash': 1, 'follow-ended': 1}
-- strategist-5.jsonl: 168 turns, 4 session replacements, 281 orders, 0 refused, 104 flagged of 781 hands' lines; landing median 0 s (p90 0), turn wall median 7 s
   orders: {'wait': 142, 'instruct': 51, 'queue': 34, 'mcp__wreason__wait': 27, 'produce': 10, 'mark': 4, 'mcp__wreason__queue': 4, 'lane': 2, 'mcp__wreason__instruct': 2, 'mcp__wreason__note': 2, 'mcp__wreason__produce': 2, 'overview': 1}
   flagged: {'step-skipped': 63, 'hunt-ended-other': 14, 'stopped-on-stop': 10, 'hunt-ended-leash': 7, 'engine-refused': 5, 'abandoned': 2, 'step-waits': 2, 'other-fault': 1}
-- strategist-6.jsonl: 178 turns, 4 session replacements, 272 orders, 1 refused, 136 flagged of 825 hands' lines; landing median 0 s (p90 0), turn wall median 6 s
   orders: {'wait': 178, 'queue': 40, 'instruct': 35, 'produce': 18, 'signal': 1}
   refused: {'signal': 1}
   flagged: {'step-skipped': 104, 'hunt-ended-leash': 9, 'hunt-ended-other': 6, 'step-waits': 6, 'engine-refused': 4, 'other-fault': 3, 'site-blocked': 2, 'follow-ended': 1, 'abandoned': 1}
-- strategist-7.jsonl: 144 turns, 2 session replacements, 233 orders, 2 refused, 55 flagged of 427 hands' lines; landing median 0 s (p90 0), turn wall median 7 s
   orders: {'wait': 145, 'instruct': 39, 'queue': 29, 'produce': 11, 'remove': 7, 'situation': 1, 'mark': 1}
   refused: {'commander': 1, 'remove': 1}
   flagged: {'step-skipped': 24, 'other-fault': 13, 'step-waits': 5, 'engine-refused': 4, 'stopped-on-stop': 2, 'abandoned': 2, 'site-blocked': 2, 'hunt-ended-leash': 2, 'hunt-ended-other': 1}
-- commander.jsonl: 18 turns, 0 session replacements, 45 orders, 2 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 33 s
   orders: {'direct': 16, 'transfer': 11, 'mark': 9, 'wait': 5, 'signal': 2, 'say': 1, 'surrender': 1}
   refused: {'?': 2}
   flagged: {}
```

Reading the flagged lines:
- **step-skipped (444 in all):** mostly "spot_X is not free" and "is being taken by a builder of another seat of ours". Eight seats were racing for one half's spots; the report never says which spots allies hold.
- **hunt-ended-leash in t1, t5 and t6:** these hunts were of 1-to-5-soldier detachments chasing small raid parties or parties the packet did not name. I found none where the packet in force wanted that raider dead.

## What Opus wanted that the harness has no lever for

1. **22:15, the commander to t1, t3, t4 and t6:** "fights the block at centre_hold ... meet it there together". t6 at 22:59: "holds at east_hold until it is clear t4/t3/t1 are actually converging".
   - No move lets one seat's hands wait on or move with another seat's group, and a seat's report shows no allied group.
   - The only lever was the commander's transfer, used at 23:35.
2. **11:15, the commander to t1, t2 and t4 (again at 12:55 and 14:35):** "Your plant's new Stouts, Janus and Whistlers walk to t3's army and fight beside it".
   - There is no join or follow across seats; a seat's soldiers can only be sent to a place.
   - t1 departed from the direction at 11:48 for this reason.
3. **12:55, the commander:** "New Stouts, Janus and Whistlers ... walk to centre_army in fours, never alone".
   - No move or list step holds new soldiers until a count is reached and then sends them together (as far as the tool list shows).
4. **11:15, the commander, and t1 10:02, t5 15:08, t6 23:52:** "Stouts in front, Whistlers and Janus behind them, Blitzes on the flanks".
   - The lane tool's `form` puts a group in ranks of six across its heading regardless of type; nothing orders a group's soldiers by type.
5. **6:12, t1:** "never enters more than two of his Guards at a time".
   - No menu move or setting counts the turrets a path or a fight enters; only Jev's reading of the prose carries it.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
run/impedance_sweep.py MATCH --counts
run/impedance_sweep.py MATCH --seat N   (N = 0..7)   and   --commander
run/minutes.py MATCH;  run/wake_read.py MATCH
run/raid_ledger.py SCRATCH/seat{1,5,6}/00;  run/pro_baseline.py SCRATCH/seat{1,5,6}/00   (no carded side for this map)
run/hands_window.py SCRATCH/seat1/00 6:34 6:50 group_C_t1;  9:45 10:20 group_C_t1
run/hands_window.py SCRATCH/seat5/00 6:52 7:00 group_I_t5 group_O_t5;  7:00 7:14;  7:30 8:30 group_W_t5;  8:30 8:45;  8:45 9:10;  15:24 15:50 group_K1_t5;  16:47 17:20 group_D2_t5
run/hands_window.py SCRATCH/seat6/00 24:08 24:45 group_P1_t6
scratch readers (jevlog-based): decode rows (seat 1 8:55-10:20, seat 5 6:55-7:12 and 15:20-15:50); call-state grep (push_centre at 9:51 seat 1; constructor_10908 at 22:24 seat 6);
  turn prompts (t1 3:12/3:52/4:29, t5 9:01/15:00/15:08/15:24, t6 14:16/22:23/22:59, commander 11:15/12:55/14:35/16:15);
  every tool call and result of every transcript (refusals the sweep's pattern misses, calls dropped after `wait`);
  the records' extractor events of all eight seats (who held each spot when); record destroyed events (t1, t5); truth counts near places and times
```

Here MATCH is `run/matches/1791600181-glitters-8v8-late-1/00` and SCRATCH is the agent scratch directory given in the prompt.
