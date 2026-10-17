# Impedance review: 1791943206-prio-player-1 (seat strategist-0)

Arena duel, 2026-10-13, Comet Catcher Remake 1.8, we start B8: **won 18:02** against BARb hard_aggressive. One seat, the
player Opus 5.5 medium, no commander, layers news,same,fuse,openers,split,tick,places,again,near. 94 turns, 192
orders, 2 session takeovers (9:19, 15:59), landing median 7 s. The first game with the builder-priority `low` word.

## Tally

| category | count | | category | count |
|---|---|---|---|---|
| A1 refused at the tool | 1 | | B1 in the hands' picture, not the report | 1 |
| A2 accepted, not executed | 4 | | B2 in the record/truth, in nobody's picture | 1 |
| A3 overruled by the pick | 4 | | B3 report wrong/stale | 0 |
| A4 overruled by code | 1 | | B4 in the report, missed by Opus | 3 |
| A5 late | 0 | | B5 hands' lines misleading | 0 |
| A6 read differently | 4 | | B6 commander-to-player hop | 0 (no commander) |
| **C clean moments** | **2 of 8** | | | |

Denominators: 8 judgement moments; 192 orders; 954 hands' lines (76 flagged); refused 1. The sweep reports
`0 refused`: its pattern misses the one refusal ("signal: not a tool that can be batched", 5:05), counted here as A1.

## Judgement moments

| moment | Opus wanted | what happened | verdict |
|---|---|---|---|
| (a) opening 0:00-4:00 | commander list (extractor, extractor, armsolar x2, armvp, `assist 45 low`, ... `armsolar low`); plant 2 Rovers, cv, 2 Rovers, cv, Blitzes | list played in order (plant at 0:35); priority 0 set on every `low` step (1:04, 2:05, 2:48, 4:03) and 1 restored on unmarked steps, lags of 4 s at two step changes; first Blitz joined the Rovers' group_A | A6 (Blitz into group_A); the `low` word: C |
| (b) 3:14 spot_68 | lone Pawn answered by one or two soldiers (2:09 packet) | group_B (1 Blitz, born 3:10) followed, then advanced on spot_68; the extractor died 3:14 as it arrived | C |
| (b) 7:38-7:42 spot_71, spot_64 | detachment of four only before the strike; one body after | strike at 7:40; decode `detach|4` 0.15 let two detachments go after it; spot_71's turret had been skipped at 7:18 | A6 + A2 |
| (b) 11:16-11:26 four Pawns at spot_71/64 | group_O and group_J together, whole; stray Blitz groups join group_O | 11:24 group_J shells and group_O attacks party_53 (pick 0.92); unnamed group_I1/J1's join read 0.28 and was fused off | A6 (none) |
| (b) 15:05 spot_28, eight Pawns | home guard answers raids at our extractors | 5 Pawns seen 14:49-14:52 dropped out of party_70 when out of sight; 14:55 report showed "1 armflash"; after 15:05 the Pawns were gone from sight, P1/Q1 joined N1 at 15:16-15:17 | B2 |
| (c) first push: raid_go 5:05 | group_B "leaves home now" on raid_go | signal refused in the batch; sent alone 5:15; group_B advanced 5:16 (column strung out by reinforcements, which the report said) | A1 |
| (d) 6:32 packet | group_B turns back, whole, on nine Pawns at D6-D7 | 8 sent as detachment "group_J", leash ended it in 0 s, group_J took in group_B and kept its name; whole body attacked at 6:48 | A4 (none) |
| (d) 11:46 packet | group_J commits by spot_39-47-49-40-27-29-34, no detachments | 11:55 "advance to spot_39", no detachment picked through 12:12 | C |

## Items, by cost (15 of 19; all 19 in impedance.json)

1. **10:45-14:25, A6, large.** Wanted (10:37 on): "Otherwise: first a light turret ... beside every extractor of ours that has none (...); then free metal spots in this order: spot_56, ...". Happened: decode `build|armmex` for constructors fell from 0.46-0.71 (old words "their job is taking free metal spots") to 0.09-0.44 (FUSE 0.3); at 12:50 0 of 16 extractor moves were open for six constructors; 8-11 of 12-13 stood helping plants. Cost: extractors 19/20/18/17 at 10/11/12/14 min against the pool median 17/20/22/27; the 15:15 wake "not held more than 20 extractors for 4 min, with 53 free spots". Readings recovered with "their job is extractors" (14:25: 0.30-0.70); 26 extractors by 17:14. Raids also took 11 extractors in 8-12 min (medium confidence on the share). Fix: code-rule (one yes/no reading should not fuse every extractor move of a builder) and words.
2. **10:45-14:25, B1, large (same window, not additive).** Group entries show `reads: ...`; builder entries show none, so the report never said his paragraph had removed extractor building. The 14:17 rewrite was a guess. Fix: report (a builder's fused build types on its entry).
3. **11:46-14:07, B4, large (same window).** Every report showed 8-11 constructors "helping lab_N build, since N s" and 50+ walkable free spots; 15 turns of notes never register it; acted 14:17, lists 14:55 and 15:39. A line that would have surfaced it: "N constructors have helped plants over 60 s while M free spots are walkable" (the 15:15 no-growth wake, 4 minutes late). Fix: report.
4. **9:56-10:20, A3, small.** Wanted (9:48, again 10:08): "At spot_29 it attacks his buildings ... it does not walk away from that fight half done." Happened: group_J kept its 9:47 fallback task `fight_to spot_40` ("PICK ... kept w1 over w2 of 1 at 0.78", fight_to_spot_29 0.39-0.43, spot_29 read 0.97), then 10:20 "walk to spot_40". 10 -> 7 Blitzes (330 metal); the LLT fell ~10:16 anyway. Fix: jev.
5. **15:57-16:16, A3, small.** Wanted (15:49, 15:59): group_N1 kills Pawns at our home extractors, "group_N1 and group_S1 attack them together". Happened: attack_party_70 rated 0.24-0.40, N1 closed by the news layer 15:58-16:06, fight_to_spot_74 0.47 at 16:08; only the commander went (16:08). Home lost 972 metal by 16:13 (plant_7477 590), but N1 (~2,500 away) and S1 (~1,600) could not have arrived before 16:11.7 (medium). Fix: jev.
6. **7:40-7:41, A6, small.** Wanted (7:32): "Before strike is given, a party of three or more armpw ... gets a detachment of four"; one body after. Happened: decode `group_J|detach|4` 0.15 for the whole packet; at the strike "4 of group_J hunt party_24 (1 armpw) as group_M", then "4 ... hunt party_22 (3 armpw) as group_N": the strike started with 8 of 16. Fix: code-rule (a mark from a clause conditioned on a signal must be re-read when the signal flips, or left to the per-second question).
7. **7:13-7:38, A2, small.** Two lists named "extractor spot_71" without an id; constructor_2945 built it (created 7:05) while constructor_56's duplicate was "refused ... the site was bad", and that refusal made 2945 skip its own turret: "skipped the step 'armllt spot_71' ... the extractor at spot_71 was refused lately". The bare extractor died to one Pawn at 7:38. Recurred at spot_49 16:51 (none). Fix: code-rule (a turret waits on the spot's extractor, not on one builder's refusal).
8. **14:53-14:55, B2, small.** Five Pawns seen at A3 14:49-14:52 (record enemy_seen; call rows "1 armflash, 5 armpw") left party_70 when out of sight while its Blitz stayed; the 14:55 report: "party_70 (1 armflash) at our extractor at spot_28", nothing in "recently seen". Truth: 8 armpw within ~200. spot_28 died 15:05 (the guard was ~2,300 away, so medium on cost). Fix: code-rule (a party keeps its members seen in the last minutes).
9. **14:36-14:41, A3, small.** Wanted (14:27): "never turns aside for a lone ... constructor (armck) ... away from its way". The 38-strong army kept attacking party_88 (1 armck): "kept w1 over w2 of 2 at 0.5" with fight_to_spot_18 0.59 (spot_18 read 0.97), again 14:40. Fix: jev.
10. **2:33-2:39, A6, small.** produce without a group put the first Blitz into the plant's default group_A (members [17003, 904, 10476, 6821]), which roved; Opus wanted Blitz groups raiding. Fixed by `"group": "new"`. Fix: brief.
11. **7:57, A2, small.** After two skipped steps, "the step 'assist low' of its list could not be played this second and was dropped": the commander idled until the 7:59 list landed 8:05. Fix: code-rule (retry, do not drop).
12. **7:52, B4, small.** Opus queued two armnanotc for the commander; the 0:10 roster line lists armnanotc's builders without armcom; both "skipped ... this builder cannot build it". The repeat at 9:29 came from the takeover session (roster in its prompt 10 s earlier; none). Fix: tool (refuse a step the builder cannot build at the tool).
13. **16:44, A2, small.** Two lists shared "armvp home #vp3"; constructor_10629 started it and constructor_27504's step, the same second, "could not be played this second and was dropped" (same at 16:52 for #nano_a). Re-sent; helped from 16:52. The same-tick id fix converts the second builder to "help", which cannot be played before the frame exists. Fix: code-rule.
14. **5:05, A1, none.** "signal: not a tool that can be batched": the packet said "raid_go is given" with no signal; Opus saw it, sent the signal alone at 5:15, group_B advanced 5:16. Fix: tool (accept signals in `orders`, or refuse the whole batch).
15. **6:46-6:59, A4, none.** A detachment of 8 named group_J (leash end after 0 s) took in group_B and kept its name, so the 6:32 packet's group_B paragraph pointed at nothing until 6:59; the body attacked whole at 6:48. Fix: code-rule (a take-in keeps the larger body's name).

Also in the JSON, cost none: 3:52-4:22 A3 (group_B kept its raid course over the home-guard paragraph, spot_73 read 0.97, picks "kept w1" six times); 11:24 A6 (unnamed group_I1/J1's join read 0.28, fused off); 9:29 B4 (commander armnanotc again after the takeover); 16:43-18:00 A2 (the lane extractor's reclaim waited behind an LLT build, done at 18:00).

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 94 turns, 2 session replacements, 192 orders, 0 refused, 76 flagged of 954 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 94, 'instruct': 40, 'queue': 32, 'produce': 15, 'lane': 4, 'signal': 3, 'mark': 3, 'remove': 1}
   flagged: {'step-skipped': 21, 'hunt-ended-leash': 13, 'hunt-ended-other': 12, 'step-waits': 9, 'other-fault': 8, 'engine-refused': 8, 'abandoned': 2, 'site-blocked': 1, 'stopped-on-stop': 1, 'follow-ended': 1}
```

Sample judged: the 8 engine refusals are all extractors at spots holding a wreck or another builder's frame or
extractor (7:13 is item 7; 8:34 spot_71 was the 7:38 wreck, rebuilt 8:59; the four at 16:21-16:51 collide with
spots other builders had just taken, cost none). Leash ends: only 6:46 crossed a packet that wanted the party dead
(item 15, cost none); the rest were detachments after lone Ticks or Pawns. The two `abandoned` frames followed
Opus's own cancels (10:20) or a pick (11:21). The hands' blocks match the log (261 of 291 plays in the next report,
the rest on the boundary second of the report before).

Side findings, not items: (1) in the 2:36-3:00 metal stall the plant, low by birth default, held its second Blitz
at 21% for 24 s while the high constructors finished a solar and an extractor; it came out at 3:10, as the first
Pawn raid killed spot_68 (3:14). (2) plant_27255 (0:35, facing east) had spot_73 in its exit lane and the
commander's own "extractor" step built armmex_28188 there at 1:56, although 3aabc77 is in this build; the yard
words flagged it at 2:29, 6:02, 14:55 and 16:36.

## What Opus wanted that the harness has no lever for

- **9:19** (and lane wakes at 2:29, 6:02, 8:37, 10:18, 14:55): "A constructor standing in a plant's exit lane walks out of it at once; they never build or wait in a plant's exit lane" / the commander "helps the plants from beside them". No move places a helper beside a plant or steps it out of a lane, and a builder on a list is off the menu; the only lever was a new list walking it elsewhere.
- **13:13**: "Stouts closing on it together from several sides". No move or footwork rule surrounds a target; the body arrived as a column (11-19 of 30+ engaged, 13:23-13:49).
- **10:37** (and 11:46): "the Stouts lead and the Blitzes come in beside them, never alone ahead" / "The Stouts lead, the Shellshockers shell from behind them". A group is one actor; nothing orders its members by type (the shell move covers the artillery half only).
- **8:59** (and 0:10 "it never goes past spot_69 or spot_54"): "never walks into the Beamer's reach at G2". The fuse shuts moves to named places, not routes through a turret's reach or across a line; the pull-back crossed the reach (9:09: "the Beamer took four more Blitzes on the way out").

## Commands

```
cat run/matches/1791943206-prio-player-1/results.jsonl; python3 -c "...json.load(open('.../batch.json'))"
run/impedance_sweep.py MATCH --counts | --seat 0 | --seat 0 --all --from 16:36 --to 17:05 | --from 7:02 --to 7:42
run/minutes.py MATCH; run/wake_read.py MATCH; run/raid_ledger.py MATCH; run/pro_baseline.py MATCH --all-starts
run/commander_turns.py MATCH --calls
run/hands_window.py MATCH for 2:30-2:40, 3:04-3:17, 3:48-4:22, 4:49-4:58, 5:00-5:22, 6:38-6:58, 7:37-7:50,
  9:44-10:20, 10:56-11:14, 11:22-11:50, 11:52-12:12, 12:44-13:05, 14:20-14:42, 15:10-15:30, 15:50-16:18, 16:36-16:56
scratch scripts (python3, reading the match read-only): dump.py (every call, result and reply), jevscan.py (decode
  and audit rows), dec.py (decode readings by actor and window), prio.py (a unit's commands and builder-priority flag),
  est.py (report's enemy estimate against the truth), playcmp.py (hands' plays against the report blocks), plus
  one-off reads of the record (unit_defs builds, losses, positions), the truth file and the pass menus' fused flags
```
