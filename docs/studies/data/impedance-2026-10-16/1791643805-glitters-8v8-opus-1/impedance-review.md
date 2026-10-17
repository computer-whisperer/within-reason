# Impedance review: 1791643805-glitters-8v8-opus-1

All That Glitters v2.2.3, realtime 8v8 (2026-10-10). Eight Opus 5.5 medium players (strategist-0 to -7) under an Opus 5.5
high commander played eight BARb hard_aggressive. The game was undecided at the 30:00 cap with all sixteen commanders
alive. Seats 0, 1 and 5 were read in full: they have the most refusals plus flagged lines (109, 104 and 100). Seats 2, 3,
4, 6 and 7 got counts and refusals only. The twelve judgement moments are four from seat 0, three from seat 1, three from
seat 5 and two on the commander hop.

## Tally

| Category | Count |
|---|---|
| A1 refused at the tool | 8 |
| A2 accepted, not executed | 1 |
| A3 overruled by the pick | 3 |
| A4 overruled by code | 1 |
| A5 late | 2 |
| A6 read differently | 3 |
| B1 in the hands' picture, not the report | 6 |
| B2 in the truth, in nobody's picture | 1 |
| B3 in the report, wrong or stale | 0 |
| B4 in the report, missed (Opus's miss) | 1 |
| B5 the hands' own lines misleading | 0 |
| B6 commander-to-player hop | 0 |
| C (clean moments) | 5 of 12 |

Denominators:

- **Moments:** 12 judgement moments.
- **Orders:** 1,735 player orders across 8 seats, plus 33 commander orders.
- **Hands' lines:** 6,356, of which the sweep flagged 726.
- **Refused:** 8, all A1.

The 12 moments:

- **Seat 0:**
  - Opening (A5, B1).
  - Raids at 5:43 and 6:04 (C). The 5:43 raid came under a packet with no raid clause for outer spots, and the 6:04 raid was answered within 1 s.
  - Push into the Pounders at 8:14 (A3, A5).
  - Last three turns (C). A corgol and a corban stood within 1,500 of group_T1_t0 and were not in the picture, but no decision would have differed with 10 s left.
- **Seat 1:**
  - Opening (B1).
  - Push at 13:47 (A1, A3).
  - Toaster strike from 19:20 to 20:52 (C). The commander's direction reached the seat at 19:38 and the packet landed at 19:57. The hands then pulled back to spot_55 under the packet's own "from a party that outweighs it" clause.
- **Seat 5:**
  - Opening (B1; the exit-lane extractor item is A6).
  - Push from 6:14 to 7:50 (A1, A4, B4).
  - Corsumo fight from 24:04 to 24:42 (A3, B2).
- **Commander hop:**
  - Regroup at 10:20: t0, t1 and t5 carried it, with stated departures (C).
  - Hold and tech at 14:50: t1 held at spot_51 rather than spot_71, a choice it gave a reason for, and t5's armavp stood at 20:18 (C).

## Items by cost

1. **5:20-7:50, seat 5, A4, large, medium.**
   - **Wanted:** the 7:02 packet: "group_A ... leaves east_lane now and does not wait for any tail or straggler".
   - **What happened:** 11 of group_A_t5's 15 soldiers stood still at G3/H3 for about 150 s. Their only commands were short moves at 5:38, 6:23 and 7:06, while the task read fight_to east_lane. The 7:20 picture said: "its body has not got nearer its goal for 120 s: stalled".
   - **Cost:** about 1,250 metal of the lane body idle through the commander's "keep going" window.
   - **Fix class:** code-rule (re-send the course to joined or parked members).
2. **14:04-14:29, seat 1, A3, large, medium.**
   - **Wanted:** the 13:47 packet: "it walks to spot_105 and gathers there, every soldier".
   - **What happened:** group_G_t1 alternated "shell party_3xx" and "gather on its front" every second, 24 picks in 24 s.
   - **Evidence:** the decode read spot_105 at 0.98. The gate rated go_spot_105 at 0.27-0.48 and shell and gather at 0.5-0.82.
   - **Cost:** 50 soldiers fell to 33. The 14:11 report: "we lost 13 units worth 2925 metal". It is a reading whether walking to G6 would have cost less.
   - **Fix class:** jev (a thrash guard).
3. **24:24-24:45, seat 5, A3, large, medium.**
   - **Wanted:** the 24:04 packet: "does not walk away from this fight while it outweighs them". The 24:23 packet: "never walks to spot_52, never gathers, never shells from a distance".
   - **What happened:** PLAYS lines show walks to spot_52 or home 8 times, plus gathers and shells, with one attack.
   - **Evidence:** the decode read spot_52 at 0.92 (named as the step-back place) and home at 0.39.
   - **Cost:** 19 soldiers fell to 11 and the H4 line fell.
   - **Fix class:** jev.
4. **24:04, seat 5, B2, large, low.**
   - **What Opus believed:** "a lone corsumo ... all that is left".
   - **The truth:** within 1,500 of group_R1_t5 stood 1 corsumo, 5 corstorm, 4 corthud, 4 cornecro and 1 corwolv. The report showed 1 corsumo, 1 corstorm and 1 corthud.
   - **Cost:** the group committed raw onto the corsumo and was cut down. The cause is a reading.
   - **Fix class:** unknown.
5. **15:58-23:16, seat 5, A2, small, high.**
   - **Wanted:** remove armmex_24287 by constructor_9066, to clear plant_23270's exit lane.
   - **What happened:** reclaim commands went out at 16:39 and 16:59, then the list moved on at 17:03 with the extractor still standing. No line said so.
   - **Evidence:** the extractor died only at 23:16, after the player re-issued the remove at 22:44.
   - **Fix class:** report (say when a take-apart ends undone).
6. **1:49 on, seat 5, A6, small, medium.**
   - **What happened:** the plant was placed with spot_37 in its exit lane, and the commander's bare "extractor" step took that spot.
   - **Evidence:** the exit-lane wakes named armmex_24287 at 7:28, 8:24, 10:54, 15:09 and 17:08.
   - **Fix class:** code-rule. This is the known exit-lane bug, fixed after this game in 3aabc77.
7. **22:17-23:25, seat 5, A6, small, high.**
   - **Wanted:** the 18:14 produce list's bare tail (`"armstump", "armsam"`), meant as "keep making these".
   - **What happened:** the counted entries ran out at 22:17 and the plant started nothing for 68 s. The player was away (the turn was abandoned from 20:40 to 22:44) and its 22:44 list was all bare. The player at 23:19: "idle for minutes with uncounted entries".
   - **Fix class:** words.
8. **23:43, seat 1, A6, small, high.**
   - **Wanted:** "armmex spot_55" (rebuild the extractor).
   - **What happened:** "nothing to do: no extractor of ours stands at spot_55 to upgrade into a armmex". The extractor was rebuilt at 24:09.
   - **Fix class:** tool (read "armmex spot_N" as "extractor spot_N").
9. **8:16-8:29, seat 0, A3, small, medium.**
   - **Wanted:** the 7:40 packet: "attacks the corlevlr party with the whole group at once ... never a few at a time".
   - **What happened:** shell picks at 8:17, 8:21 and 8:29 (2 Whistlers firing). The attack moves were rated 0.17-0.48.
   - **Cost:** most losses came from the gather the packet itself asked for.
   - **Fix class:** jev.
10. **1:27-2:31, seat 0, A5, small, medium.**
    - **Wanted:** the list for constructor_16706 sent at 1:25 (the turn ran 24 s).
    - **What happened:** the list landed about 1:40, after the hands had picked a solar for the idle new constructor at 1:35, in the plant's exit lane. spot_19 started at 2:31. The player's 1:49 note claimed it "had ignored its list".
    - **Fix class:** code-rule (hold a newborn builder for a list in flight).
11. **3:15, seat 1, B1, small, high.** Eleven of twelve extractor steps were skipped at 3:25. Allied extractors already stood on nine of those spots at 3:11 (record 'al'), but the report gave only "free spots ... 93, nearest five". **Fix class:** report (a line of allied-held and allied-claimed spots).
12. **3:35, seat 0, B1, small, high.** All four D1 steps (spots 12, 9, 16, 21) were skipped "not free": t4's extractors stood on them. **Fix class:** report.
13. **22:22-23:25, seat 0, B1, small, high.** Advanced constructors given by t6 appeared as "constructor_N" with no type, so their extractor and Sentry steps failed with "this builder cannot build it". constructor_18472 was "left alone" idle, although the hands' picture says "is: Advanced Construction Vehicle (armacv ...)". The same happened in seat 1 at 21:53 and 23:59, and in seat 5 at 25:00, 26:20 and 28:34, each counted once per seat. **Fix class:** report.
14. **7:02, seat 5, B4, small, medium.** The packet said group_A was "standing at east_lane". The entry read "at front at east_lane (H5) ... 1 of 11 arrived ... stalled". A line led by where the body stands ("11 of 15 at G3, still since 5:38") would have surfaced it. **Fix class:** report.
15. **8:14-8:29, seat 0, A5, small, low.** The 8:14 packet ("attacks ... at once, does not gather, shell") landed at 8:29 (the turn ran 21.6 s), and the next pick attacked as asked. The group fell from 13 to 5 while the packet was in flight. It is a reading whether an earlier landing would have saved them.

The 8 A1 items cost nothing:

- **Signals refused in a bundle.** Four signals were refused with "signal: not a tool that can be batched": seat 1 at 13:47 and 20:52, seat 5 at 7:02, seat 3 at 7:45. Each was re-issued alone, superseded by a later packet, or covered by the commander's own signal. The 13:47 note and the 7:02 packet assumed the signal was in force.
- **Seat 3 at 11:55:** produce to group_N_t3, which did not exist. Re-issued as "new" at 12:42.
- **Seat 2 at 14:44:** transfer of group_I1_t2, which the hands had merged into group_G. Re-issued at 14:59.
- **Seat 7 at 7:33 and 14:30:** remove by "commander", where the seat's commander is commander_t7. Fixed at 7:42; the second time it was not needed.

The fix class for the bundled signals and the "commander" name is tool. Half of all refusals are the signal-in-orders refusal.

Also judged and found harmless (not counted):

- Engine refusals right after an #id building stood (seat 1: armnanotc at 12:29, armvp at 27:34; seat 5: armmex at spot_57 at 6:27, armavp at 20:18). A second builder's attempt came after the first had already raised the building.
- Seat 0's notes at 15:38 and 17:12 say the plant stood "idle". The record shows gaps of 20 s and 24 s.

## Sweep counts per seat (verbatim)

```
-- strategist-0.jsonl: 107 turns, 2 session replacements, 209 orders, 0 refused, 109 flagged of 1005 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 106, 'queue': 47, 'instruct': 36, 'produce': 14, 'transfer': 4, 'lane': 2}
   flagged: {'step-skipped': 53, 'step-waits': 18, 'engine-refused': 10, 'hunt-ended-leash': 9, 'hunt-ended-other': 6, 'other-fault': 4, 'site-blocked': 3, 'stopped-on-stop': 2, 'follow-ended': 2, 'abandoned': 2}
-- strategist-1.jsonl: 113 turns, 2 session replacements, 218 orders, 2 refused, 102 flagged of 869 hands' lines; landing median 0 s (p90 0), turn wall median 15 s
   orders: {'wait': 113, 'queue': 49, 'instruct': 34, 'produce': 15, 'signal': 3, 'lane': 2, 'mark': 1, 'remove': 1}
   refused: {'signal': 2}
   flagged: {'step-skipped': 61, 'stopped-on-stop': 18, 'engine-refused': 7, 'abandoned': 4, 'site-blocked': 4, 'other-fault': 3, 'hunt-ended-other': 2, 'hunt-ended-leash': 1, 'follow-ended': 1, 'step-waits': 1}
-- strategist-2.jsonl: 99 turns, 3 session replacements, 198 orders, 1 refused, 69 flagged of 693 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 98, 'queue': 38, 'instruct': 34, 'produce': 14, 'transfer': 13, 'mark': 1}
   refused: {'transfer': 1}
   flagged: {'step-skipped': 33, 'hunt-ended-other': 9, 'stopped-on-stop': 9, 'hunt-ended-leash': 4, 'follow-ended': 4, 'engine-refused': 4, 'step-waits': 3, 'other-fault': 2, 'site-blocked': 1}
-- strategist-3.jsonl: 115 turns, 2 session replacements, 226 orders, 2 refused, 79 flagged of 813 hands' lines; landing median 0 s (p90 0), turn wall median 12 s
   orders: {'wait': 114, 'queue': 49, 'instruct': 35, 'produce': 15, 'lane': 8, 'transfer': 4, 'signal': 1}
   refused: {'signal': 1, 'produce': 1}
   flagged: {'step-skipped': 33, 'engine-refused': 15, 'stopped-on-stop': 9, 'step-waits': 6, 'hunt-ended-leash': 6, 'other-fault': 4, 'site-blocked': 3, 'hunt-ended-other': 2, 'abandoned': 1}
-- strategist-4.jsonl: 116 turns, 2 session replacements, 237 orders, 0 refused, 97 flagged of 927 hands' lines; landing median 0 s (p90 0), turn wall median 11 s
   orders: {'wait': 115, 'instruct': 49, 'queue': 48, 'produce': 16, 'mark': 5, 'lane': 3, 'remove': 1}
   flagged: {'step-skipped': 47, 'stopped-on-stop': 18, 'step-waits': 9, 'engine-refused': 8, 'hunt-ended-leash': 5, 'abandoned': 5, 'other-fault': 2, 'site-blocked': 2, 'hunt-ended-other': 1}
-- strategist-5.jsonl: 97 turns, 2 session replacements, 205 orders, 1 refused, 99 flagged of 791 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'wait': 97, 'queue': 37, 'instruct': 36, 'produce': 21, 'lane': 6, 'mark': 2, 'signal': 2, 'transfer': 2, 'remove': 2}
   refused: {'signal': 1}
   flagged: {'step-skipped': 42, 'step-waits': 17, 'engine-refused': 14, 'hunt-ended-other': 7, 'stopped-on-stop': 6, 'site-blocked': 4, 'other-fault': 4, 'abandoned': 2, 'follow-ended': 2, 'hunt-ended-leash': 1}
-- strategist-6.jsonl: 105 turns, 2 session replacements, 217 orders, 0 refused, 84 flagged of 593 hands' lines; landing median 0 s (p90 0), turn wall median 10 s
   orders: {'wait': 104, 'queue': 49, 'instruct': 24, 'transfer': 24, 'produce': 16}
   flagged: {'step-skipped': 37, 'engine-refused': 25, 'other-fault': 11, 'hunt-ended-other': 6, 'step-waits': 2, 'stopped-on-stop': 1, 'hunt-ended-leash': 1, 'site-blocked': 1}
-- strategist-7.jsonl: 122 turns, 3 session replacements, 225 orders, 2 refused, 87 flagged of 665 hands' lines; landing median 0 s (p90 0), turn wall median 9 s
   orders: {'wait': 122, 'queue': 47, 'instruct': 27, 'produce': 16, 'transfer': 5, 'remove': 4, 'mark': 3, 'situation': 1}
   refused: {'remove': 2}
   flagged: {'step-skipped': 36, 'hunt-ended-other': 15, 'step-waits': 8, 'stopped-on-stop': 8, 'site-blocked': 6, 'follow-ended': 5, 'other-fault': 4, 'engine-refused': 2, 'abandoned': 2, 'hunt-ended-leash': 1}
-- commander.jsonl: 19 turns, 0 session replacements, 33 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 26 s
   orders: {'direct': 18, 'transfer': 6, 'mark': 3, 'units': 3, 'wait': 2, 'signal': 1}
   flagged: {}
```

"landing median 0 s" holds because this is realtime. Each order lands when the turn ends, so the real lag is the turn's
wall time: a median of 9 to 15 s, often 20 to 30 s. That lag is behind both A5 items.

Three patterns run through the step-skipped lines. In the openings, seats aimed at extractor spots that allies already
held or claimed. Later, steps went to advanced constructors that cannot build them. Late in the game, "#id stands
already" lines followed rebuilds.

## What Opus wanted that the harness has no lever for

- **13:06, seat 1:** the packet asked constructors to take apart "the nearest field where no enemy is within 800". The note added: "Wish: a list step 'reclaim the wreck field at <place>'". The menu offers only the first 3 wreck fields of 100 metal or more, and no list step aims a constructor at a field.
- **21:23, 21:53, 25:31, 28:05, seat 1:** "[the commander] stands beside the plant" and "[constructors] never stand in front of the plant's exit". Where a helping builder stands is the engine's choice, so the exit-lane wakes recur (armcom_765 at 28:05).
- **6:14-7:50, seat 5:** "it leaves east_lane now and does not wait for any tail or straggler". Nothing re-sends a stalled group's course to its parked members. The lane tool sets footwork only.
- **25:20 and 28:20, commander to t6:** "The Basilica ... Its targets in order: the Persecutor at E5, then his block". A turret has no menu, and the fire tool is for a silo only.
- **Near-misses (a lever exists):**
  - Seat 1 at 1:28 said "hands cannot count" about the commander's "two Blitzes per seat ... as home defence". A counted produce into a "new" group plus a hold sentence would do it.
  - Seat 0 at 17:54: "Would like a lane setting that disables detachments for a group". The packet's forbidden mark already does this in words.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
run/impedance_sweep.py MATCH --counts
run/impedance_sweep.py MATCH --seat N   (N = 0..7)  ;  --commander
run/impedance_sweep.py MATCH --seat 5 --all --from 15:58 --to 17:40 ; --seat 1 --all --from 23:43 --to 24:00
run/minutes.py MATCH ; run/wake_read.py MATCH ; run/raid_ledger.py MATCH
run/pro_baseline.py MATCH, SCRATCH/seat1, SCRATCH/seat5   (no carded side on this map)
run/hands_window.py MATCH 5:30 5:58 ; 8:14 8:36 group_I_t0
run/hands_window.py SCRATCH/seat5 7:05 7:40 group_A_t5 ; 7:23 8:10 group_A_t5 ; 24:10 24:45 group_R1_t5
run/hands_window.py SCRATCH/seat1 13:47 14:45 group_G_t1 ; 19:38 20:15 group_G_t1 ; 19:58 20:52 group_G_t1 ; 15:05 15:30 group_G_t1
scratch helpers (jevlog iteration): jv.py (menus, gate flags, plan picks, decode reads), grp.py (group members' positions
  in the record), opening.py (created/finished to 4:00), truthnear.py (truth within 1,500 of each group vs the report's
  enemy-in-sight), turns.py (a transcript's turns, calls and results by clock)
record/truth one-off reads: allied extractors on listed spots ('al' samples), cmd rows for group_A_t5 members and for
  armmex_24287, plants' created-by events (15210, 23270, 29504)
```

Symlink directories for the seat-0-only tools were made for seats 1 and 5 under
`/tmp/claude-1000/-home-christian-workspace-playground-bar-bots/93f9e8f3-fea1-482e-a1c2-cb750e01d6e2/scratchpad/impedance/agents/1791643805-glitters-8v8-opus-1/`
(seat1, seat5).
