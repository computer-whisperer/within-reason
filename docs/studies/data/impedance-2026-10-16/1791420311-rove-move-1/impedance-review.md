# Impedance review: rove-move-1 (2026-10-07)

Arena duel on Comet Catcher Remake 1.8. We won at 23:52 against BARb hard_aggressive. One seat (strategist-0) was played by claude-opus-5-5 at medium effort, with no commander. The hands' layers were news, same, fuse, openers, split, tick, places, again and near, and a truth file exists. There were 4 session replacements, at 5:15, 12:27, 18:48 and 19:58. This was the first game with `rove` as a pick on the group menu.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 0 |
| A2 accepted, not executed | 2 |
| A3 overruled by the pick | 7 |
| A4 overruled by code | 2 |
| A5 late | 0 |
| A6 read differently | 5 |
| B1 in the hands' picture, not the report | 0 |
| B2 in the record/truth, in nobody's picture | 0 |
| B3 report wrong/stale | 0 |
| B4 in the report, missed by Opus | 0 |
| B5 hands' lines misleading | 0 |
| B6 commander-to-player hop | 0 (no commander) |
| C, clean judgement moments | 1 of 8 |

Denominators:
- judgement moments: 8
- orders: 262 (130 waits)
- hands' lines: 993 (95 flagged)
- refused: 0
- takeovers: 4

All 16 items cost `none` or `small`. None plausibly decided the game, which was won with the army 5 to 21 times his from 11:00 on. A pick going against the packet is the dominant shape: the A3 and A6 items together number 12 of 16.

## The eight judgement moments

**(a) Opening, to 4:00 (A3, item 4).** The first `queue`, `produce` and `instruct` were executed exactly:
- The commander built 2 extractors and 2 solars, and started the vehicle plant at 0:43.
- The plant made Rover, Rover, constructor, Rover, Rover, constructor, then Blitz ×3, constructor, Blitz ×3, constructor, as listed.

The exception was the packet's "Rover groups ... they rove, from the first Rover". The whole-group rove move never cleared 0.25, so group_A held its Rovers at home, in the plant's exit lane, until 2:15. Jev's own `scout` picks at 1:12 and 1:38 did send two lone Rovers out, and they found his plant at 2:16. The report carried everything knowable, including the exit-lane wake at 2:06.

**(b1) Raids on home extractors, 3:50-4:12 (C).** A corgator killed spot_62, spot_68 and spot_63. Every raid was answered within 1-2 s by the one-or-two-Blitz detachments the packet asked for. The losses came from Opus's guard place (group_E at spot_64, 2,500 from the raided spots) and the corgator's speed, not from the hands. The report showed the raider as it happened, matching the truth.

**(b2) spot_24 and spot_26 at C3, 11:14 and 11:38 (A3, item 11).**
- spot_24 was covered by no packet sentence and fell to an unseen killer. That part is clean.
- From 11:23 the packet put group_H at spot_38, covering spot_26. The decode read the place at 0.97, but the gate held group_H at spot_54 for 13 s with go_spot_38 at 0.17.
- The 11:38 loss was not reachable in time either way.

**(b3) Correap raid, 17:22-17:46 (A3, item 12).** The packet that landed at 17:26 said "the correap at spot_31 now is its job". group_D1's attack moves rated 0.13-0.37 while the correap was in sight. It attacked at 17:39, too late for spot_37 (17:31, an unseen killer), and spot_44 fell at 17:46 after the attack began.

**(b4) Cormuskrat at spot_79, 20:06-20:21 (A4, item 10).**
- The hands sent the packet's detachment of four at once (20:06).
- The 900 leash ended that hunt after 1 s, and a second one after 0 s, because the quarry stood ~2,300 away.
- Shell, attack and detachment picks then churned. The extractor was lost either way.

**(c) First window and first push: the raid at 5:15-5:45 (A3, item 3).** The window was at 2.0x at 5:00 with our mass at B8. The 5:15 takeover packet sent group_E raiding along a route. The decode read the route (spot_56 0.96), but the move rated 0.38 and the group held at home for 21 s. It advanced at 5:43 only after a `signal raid now` and a sentence added at 5:35, "never holds at home". After that the body went where the packet said, as one body. The enemy estimate ("~266 seen" at 5:15, against 480-1190 in the truth) was labelled a floor in the report, which is correct.

**(d1) The 7:55 packet change: go_east at 8:30 (A6, item 13).**
- In the next 90 s every join played as written: group_Q, group_R, group_E and group_N joined group_P between 8:02 and 8:59.
- The army advanced at 8:45 along the named way, and the decode read spot_37, spot_27 and spot_29 at 0.93-0.95.
- One pick crossed the packet. At 8:14 a two-Blitz detachment chased "a lone corgator", which the packet said was "NOT their business". The decode had read detachments at 0.72-0.77.

**(d2) The 16:07 and 16:37 packet changes (A3, item 1).** The 16:20 attack on party_39 (2 corraid and a correap) was licensed. When party_39 shrank to one corraid, the 53-soldier army went on chasing it west for 33 s. This was against "never shells, chases or detaches after a single ... corraid", and the new packet did not re-ask it (group_P.change 0.41-0.47). Opus diagnosed this exactly in its 16:53 note.

**Missing-context check at every moment.** At each of the eight turns I compared the report with what was knowable:
- The report's "enemy in sight" lines matched the enemy units our units had seen within 60 s, within 1,500 of our groups (truth file against the record's `enemy_seen`).
- Every play in the jev log appears in the next report's "what your hands did" block, except 34 of 874 that fell on the boundary second of the previous report.

I found no B item in which a decision would plausibly have changed.

## Items, ranked by cost

All 16 items are in impedance.json; the 15 costliest are listed here. Items 1-9 are `small` and 10-15 are `none`.

1. **16:44-16:54, A3, small.**
   - Wanted (16:37): "never shells, chases or detaches after a single corgator, corraid, corwolv or corlevlr ... goes on along its way".
   - Happened: group_P (53 soldiers) kept its 16:20 attack course after the party became one corraid. It walked 1,100 west from (5574,3032) to (4515,3404). Change stayed at 0.41-0.47 until 16:54, and it advanced at 16:56.
   - Cost: about 80 s of the army's march.
   - Fix: code-rule (re-ask a kept attack when its quarry shrinks to what the packet forbids).
2. **20:56-21:20, A3, small.**
   - Wanted (20:49): "His commander in sight is attacked by every soldier at once, before anything else".
   - Happened: the course "fight_to his_avp" was kept. attack_party_52 rated 0.19-0.24; the 21:19 entry read "within 800 of 3 of its 9: it outweighs us ... best_rated: the attack on party_52 (0.32)". The attack came at 21:20.
   - Cost: 24 s.
   - Fix: menu (the weighing counted the 9 of 38 in the body).
3. **5:22-5:43, A3, small.**
   - Wanted (5:15): "Raid group group_E: it raids his half. Its way, in order: spot_56, ...".
   - Happened: decode spot_56 0.96, gate 0.38, held 21 s. "5:43 PLAYS group_E: advance to spot_56" came after a signal and a rewrite.
   - Cost: 21 s and a turn.
   - Fix: jev.
4. **1:08-3:12, A3, small.**
   - Wanted: "Rover groups (armfav): they rove, from the first Rover"; then "the group roves at once (rove), never holds".
   - Happened: group_A.rove 0.06-0.25, group_D.rove 0.07-0.26. group_A held two Rovers in the exit lane 1:08-2:15 (wake: "exit lane is blocked: ... armfav_17003 ... 0:30").
   - Cost: 2 of the first 4 Rovers never roved.
   - Fix: menu (the move's words; offline study 0.15 → 0.65 with plain words).
5. **20:19-23:48, A3, small.**
   - Wanted: "Scouts: the new group of armfav ... roves at once"; and "group_R1: roves. Its one move is rove; it never follows or joins another group."
   - Happened: all 8 Rovers made for this joined or followed other groups. At 22:48 group_R1's menu read "change=0.73 ... scout=0.29 ... rove=0.1", and it followed group_M1 to the end.
   - Cost: no scouting while his commander was unseen from 21:51 to 23:02; what rovers would have found is unknowable.
   - Fix: menu.
6. **13:41, A2, small.**
   - Wanted (13:06): "armfus eco_field #fus1" ("energy is the bottleneck").
   - Happened: "the engine refused a armfus at A8: the site was bad", plus 4 more refusals at eco_field. Re-marked at 13:51; the builder finished a moho first.
   - Cost: the fusion was created at 15:07, 86 s after the refusal, during a stall.
   - Fix: code-rule (search near a full mark, or say the mark is full).
7. **15:13, A6, small.**
   - Wanted: `armavp #avp2` on three lists.
   - Happened: three frames at fus_site2, fus_site and spot_74, all at 15:13.
   - Cost: two frames left at 0%, the builders' walks, and a cleanup turn.
   - Fix: tool (the same-tick id race, fixed later in 4ed68d8).
8. **10:02-10:22, A6, small.**
   - Wanted (9:46): the army "holds at spot_31 ... as one body ... the whole group together".
   - Happened: "8 of group_P hunt party_14 (1 corgator)". The decode read detach|8 at 0.71; the forbidden noul was 0.59, under its 0.7 bar. By 10:13 the army was in six groups.
   - Cost: the army split for 20 s and lost one Blitz.
   - Fix: jev (the decode).
9. **12:51-13:21, A6, small.**
   - Wanted (12:43): group_J "holds at spot_49"; "every other soldier group joins group_P".
   - Happened: decode group_J|join|group_P 0.53, then "13:21 group_J join group_P".
   - Cost: none measured beyond a re-made guard.
   - Fix: words ("every soldier group other than group_H and group_J", which Opus had used at 10:16, read correctly).
10. **20:06-20:21, A4, none.**
    - Wanted: "send a detachment of four at each" (the cormuskrat).
    - Happened: "group_E1's hunt of party_48 ended after 1 s: the leash of 900 from where it began reached".
    - Cost: the quarry was out of reach anyway.
    - Fix: code-rule.
11. **11:23-11:36, A3, none.** group_H was held at spot_54 for 13 s under "holds at spot_38". The decode read 0.97; the gate rated go_spot_38 at 0.17. Fix: jev.
12. **17:27-17:39, A3, none.** "the correap at spot_31 now is its job"; group_D1's attack rated 0.13-0.37 and came at 17:39. Fix: jev.
13. **8:14, A6, none.** "A lone corgator is NOT their business"; a two-Blitz hunt went out anyway (decode detach 0.72), and the detachment rejoined at 8:17. Fix: jev.
14. **7:41, A4, none.** The leash ended group_N's hunt one second before the packet asking for it landed. The quarry died at 7:54 to another detachment. Fix: code-rule.
15. **15:29, A6, none.** The `remove` reclaim of the two extra frames was cancelled by the same turn's `queue` starting with `stop`: "takes plant_17970 apart next" (15:29), then "stopped ... on your `stop`" (15:30). The frames decayed at 0% on their own. Fix: tool.

(Item 16, A2, none: from 6:43 to 10:41, ten turret steps were skipped with "the extractor at spot_X was refused lately", while our own extractor, built by another builder, stood there. Six turrets were built by the other builder; spot_45, spot_78 and spot_55 never got one and were never raided.)

Other sweep lines I checked and did not count:
- The 9 extractor refusals all hit spots where another builder's extractor already stood (Opus's overlapping lists).
- The home and eco_field refusals other than the fusion were re-placed within seconds or replaced by Opus.
- The leashes at 10:02 and 10:11 ended hunts the packet did not want.
- The 22:55 leash was on a constructor party no packet named.
- 17:47 "no builder of ours by that name stands": the constructor was at 5% under three turrets in the report it answered, so the order could not have saved it.
- The two abandoned turns (5:05, 19:48) cost 10 s each with no consequence.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 132 turns, 4 session replacements, 262 orders, 0 refused, 95 flagged of 993 hands' lines; landing median 7 s (p90 7), turn wall median 8 s
   orders: {'wait': 130, 'instruct': 53, 'queue': 38, 'produce': 16, 'mark': 11, 'signal': 8, 'lane': 5, 'remove': 1}
   flagged: {'engine-refused': 29, 'step-skipped': 22, 'hunt-ended-other': 13, 'hunt-ended-leash': 12, 'stopped-on-stop': 9, 'step-waits': 4, 'other-fault': 3, 'abandoned': 3}
```

## What Opus wanted that the harness has no lever for

- **7:25, 19:58, 23:02:** "Constructors never stand in front of the plant / in a plant's exit lane"; the commander "helps the plant from beside it, never in front of a plant". No menu move or list step says where a builder stands. The lane was blocked at 2:06, 4:25, 19:08 and 21:29.
- **2:45, 3:05, 3:35:** "A lone corfav that runs away is never chased past spot_59" and "Never chase a fleeing raider past spot_47". A hunt's only bound is code's 900 leash from where it began.
- **16:53 (note):** "a way to force a group's course to end and re-ask (a 'drop current attack' order)". No tool ends a course, and a packet reaches a busy group only through its change gate.
- **22:19 (note):** "a produce option that sends a plant's units straight to rove". Produce sends units to "new" or to a named group; roving happens only when Jev picks it.
- **20:19:** "Scouts ... roves at once ... it is looking for his commander and any new factory of his". The rove move takes no goal; the rover's targets are code's (his start box, then the stalest spots).

## Commands run (from the repository root; M = run/matches/1791420311-rove-move-1/00)

```
cat M/../results.jsonl; python3 -c "...json.load(open('M/../batch.json'))"
run/impedance_sweep.py M --counts
run/impedance_sweep.py M --seat 0                      (saved to the scratch dir, read in full)
run/impedance_sweep.py M --seat 0 --all --from 9:46 --to 10:16   (and 13:00-15:10, 15:05-16:10, 4:15-5:20)
run/minutes.py M;  run/wake_read.py M;  run/raid_ledger.py M
run/pro_baseline.py M  (no carded side from B8) ;  run/pro_baseline.py M --all-starts
run/commander_turns.py M --calls / --notes             (packets re-extracted whole from strategist-0.jsonl)
run/hands_window.py M <window> <actors>                 3:00-3:14 group_A group_D; 5:22-5:45 group_E; 8:12-8:15 group_P;
   10:00-10:04 group_P group_V; 11:22-11:42 group_H; 16:19-16:22, 16:30-16:31, 16:44-16:45, 16:52-16:56 group_P;
   17:24-17:48 group_D1; 20:04-20:24 group_D1 group_E1 group_F1 group_G1 group_H1; 20:57-21:01 group_P; 22:36-22:50 group_R1
python3 scripts in the scratch dir: a one-pass index of jev-0.jsonl.zst through run/jevlog.py (decode, call groups and parties,
   pass/plan plays, worlds_gate rove/change nouls, audits); per-turn prompt reader; report-vs-jev-log play comparison;
   truth-vs-report fog check at the eight moment turns; record queries (unit fates, xi damage by type, created/refused sites,
   enemy_seen of his commander, corpun, corhllt, corlevlr)
```

Seat 0 only, so no symlink directories were needed. Nothing was unreadable. pro_baseline needed `--all-starts`.
