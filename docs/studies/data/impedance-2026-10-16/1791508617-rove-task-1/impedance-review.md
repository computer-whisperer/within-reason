# Impedance review: rove-task-1 (1791508617-rove-task-1/00)

Arena duel on Comet Catcher Remake 1.8, 2026-10-08, against BARb hard_aggressive. One seat (strategist-0, Opus 5.5 medium), no commander, five session replacements. We lost at 32:28. Our army was 1.6-4.4x his from 12:00 to 25:00, then his tier 2 and 3 came into our half from 25:00.

## Tally

| category | count |
|---|---|
| A1 refused at the tool | 3 |
| A2 accepted, not executed | 2 |
| A3 overruled by the pick | 6 |
| A4 overruled by code | 6 |
| A5 late | 2 |
| A6 read differently | 4 |
| B1 in the hands' picture, not in the report | 0 |
| B2 in the record/truth, in nobody's picture | 0 |
| B3 in the report, wrong/stale | 1 |
| B4 in the report, missed by Opus | 0 |
| B5 the hands' own lines misleading | 0 |
| B6 commander-to-player hop | 0 (no commander) |
| **C clean judgement moments** | **2 of 8** |

Denominators:
- **Judgement moments:** 8. A moment counts as clean when no item above cost `none` came out of it.
- **Orders:** 382.
- **Hands' lines:** 1,752, of which the sweep flagged 123.
- **Refused:** 3. The sweep reports 0 because it misses the result "signal: not a tool that can be batched", which three batched `signal` calls got.

How the 8 moments came out:

| moment | result |
|---|---|
| (a) opening | not clean: A6 at 2:15, A5 at 3:16; the commander's list and the produce list were done exactly |
| (b1) raid at 3:49, spot_61/63 | clean (leash A4, cost none) |
| (b2) raid at 8:51, spot_59 | clean (leash A4, cost none) |
| (b3) raid at 16:07, spot_33 | not clean: A6, large |
| (b4) raids at 21:19-21:22, spot_30/43 | not clean: A4, large |
| (c) first push at 9:37 | not clean: A3 and B3 |
| (d1) push2 at 11:28 | not clean: A3 |
| (d2) the Tzar order at 21:48-22:07 | not clean: A3, large |

## Items ranked by cost

1. **A6, large, high. 16:39-20:13.** Wanted (every packet from 16:57): "constructors: ... Their job is taking free metal spots ... spot_68 (rebuild), spot_66 (rebuild) ...".
   - What happened: the decode read the builders' extractor moves as not given and fused them off. At 20:11 `constructor_*|build|armmex` read 0.12-0.34, 8 of 10 under the 0.3 bar. All 6 extractors started 12:31-20:32 came from `queue` lists. Eight spots lost 16:39-18:48 stood empty 85-214 s, until the lists Opus wrote at 20:13 ("Constructors weren't rebuilding under prose").
   - Fix: jev (the build question for this phrasing), or a report line showing builders' decoded build marks.
2. **A3, large, high. 21:56-22:21.** Wanted (21:48): "His Tzar (corgol) at spot_39 ... the business of group_F1 and group_Z2 together, now: both fight to spot_39"; again for spot_33 at 22:07.
   - What happened: the marks landed (spot_39 at 0.97, spot_33 at 0.93), but the gate rated fight_to_spot_39 at 0.25. The picks kept F1 (34 soldiers) on hold ("kept w1 over w3 of 2 at 0.8") for 25 s. Z2 went in alone and lost 885 metal; the Tzar escaped.
   - Fix: jev.
3. **A3, large, medium. 20:44-21:36.** Wanted (20:36, 21:08): "group_A2: it does not hold at home. It walks to spot_58 (E6), whole, and joins group_F1 there".
   - What happened: spot_58 read 0.96-0.98; the gate rated go_spot_58 0.54-0.68 and follow 0.69-0.84. A2 (47 soldiers, ~8,700 metal) still held at home for 52 s; the 21:16 pick took in group_V3 and "keeps its course". It first moved at 21:36, and its tail was 1,900-2,400 behind when the strike began.
   - Fix: jev.
4. **A6, large, medium. 23:11-23:51.** Wanted (22:26): "group_A2: it follows group_F1 wherever group_F1 goes, whole, and fights beside it".
   - What happened: the follow ran as a trail, 844-1,283 behind, while F1 fought into the G5 nest alone. F1 fell from 33 to 20 soldiers; A2 (47-55) was not in the fight.
   - Fix: menu (a follow that keeps pace, or an "arrive together" move).
5. **A4, large, high. 21:16-21:23.** Wanted (21:08): "group_Z2 ... one body that never splits. corgator ... at our extractors (spot_30 ... spot_43) ... it attacks them".
   - What happened: Z2's menu had no whole-group attack, because WHOLE_GROUP_OVER withholds it at 4x odds. Every detachment carried the packet's forbidden mark (0.90). Z2 held, and spot_30 (21:19) and spot_43 (21:22) fell unanswered.
   - Fix: code-rule, and tell the player (brief).
6. **A6, large, medium. 16:03-16:10.** Wanted (15:43, 16:02): "home guard (group_S1, group_R1, group_P1 ...): they join each other at spot_61 ... A party of three or more there: the whole home guard fights it."
   - What happened: the decode read "join each other" at only 0.25-0.41, so the guard stayed in four pieces (1, 3, 4 and 2 soldiers). No piece attacked party_34 (rated 0.09-0.20). spot_33 fell at 16:07, the answer came 70 s later, and it was not rebuilt.
   - Fix: words (name one body to join the other).
7. **A3, small, high. 11:29-11:48.** Wanted (11:18): "holds gathered at spot_56 until push2 is given. ... When push2 is given it fights as one body ... spot_55".
   - What happened: push2 was seen at 11:29 and spot_55 read 0.95. The pick kept the hold ("kept w1 over w3 of 5 at 0.74"), then the news layer closed group_R 11:32-11:41. The push left at 11:48, 19 s late.
   - Fix: jev, and the news layer.
8. **A3, small, high. 9:40-9:43.** Wanted (9:03 packet, in force): a lone corgator "is attacked by a detachment of four".
   - What happened: the pick sent "8 of group_R hunt party_13" and then 4 more, so the push left with 10 of 21. Both detachments rejoined by 9:55.
   - Fix: jev.
9. **A3, small, high. 17:10.** Wanted (16:57): "group_F1 never stands at spot_41, spot_40, spot_35, spot_53 or spot_42" (the corpun's reach).
   - What happened: "PLAYS group_F1: advance to shelling". `shelling` is the corpun's own place, 176 from spot_42, and the decode has no reading for it. Opus banned it at 17:17, before F1 got in reach.
   - Fix: menu (an inferred place inherits the bans on the spot beside it).
10. **A3, small, medium. 24:08-24:29.** Wanted (24:01): "Its way: spot_35, then spot_29 ... The corban, correap and corvrad at C4/D4 on our side are not its business".
    - What happened: A2 kept the 24:03 engage on party_61 and walked 1,400 west, although spot_35 read 0.99. Opus ratified the turn at 24:19 ("Accepting it").
    - Fix: jev.
11. **A4, small, high. 10:20-11:44.**
    - Wanted: "armllt spot_47", queued twice (10:28, 10:58).
    - What happened: the step was skipped 4 times with "the extractor at spot_47 was refused lately; a defence there waits for it". In fact our extractor had stood there since 10:12; the refusal was a second builder's duplicate attempt. spot_47 fell to two corgators at 11:44 with no turret (the same happened at spot_59 at 9:44 and 9:46).
    - Fix: code-rule.
12. **A4, small, medium. 17:42-17:48.** Wanted: "group_X1: the raider hunters ... the whole group attacks it there and chases it off our side".
    - What happened: four detachments at party_39 ended "after 0 s: the leash of 900", leaving fragments, and party_39 went on raiding C3/B4.
    - Fix: code-rule.
13. **A4, small, high. 2:24-2:35.** Wanted (2:17): "Rover groups (armfav): an enemy corfav or corgator at our home buildings is attacked first, whole."
    - What happened: group_B was roving, and a roving group's menu carries no attack. The Rascal shot a constructor until the 2:27 packet stopped the rove.
    - Fix: menu.
14. **B3, small, medium. 9:37.** Opus pushed on "army 3122 vs ~1360 seen".
    - What happened: the report gave only a floor ("8 worth 1360 metal; its army is at least that"), while the truth was 2,685-3,510. The push met 5 corlevlr and 3 corraid at F4 and fell back at 10:48, about 7 Blitzes down.
    - Fix: report.
15. **A6, small, high. 2:15-2:37.** The same extractor fuse as item 1, in the opening.
    - What happened: armmex read 0.18-0.29. One constructor helped the plant and the other walked; there was no constructor extractor until Opus's 2:37 lists (the first at 2:55).
    - Fix: jev.

Also in impedance.json, each at cost none or small:
- **A1:** three batched `signal` calls refused with "signal: not a tool that can be batched", at 9:03, 11:18 and 18:44. Each was re-issued or superseded; north at 18:44 waited 10 s.
- **A5:**
  - At 3:16 the commander's list landed after its solar had started, and Opus stopped it; the frame decayed, about 28 metal lost.
  - At 9:47 the takeover packet making group_T the raider hunters landed a second after the pick had merged group_T into group_R.
- **A2:** the commander was given steps it cannot build, and the tool accepted them: armnanotc at 6:18 and armavp at 26:03. Opus's note of 25:56 assumed the armavp was in force.
- **A4, cost none:** leash ends at 3:59 and 8:51, each followed by a new attack pick within a second.

## Sweep counts (verbatim)

```
-- strategist-0.jsonl: 182 turns, 5 session replacements, 382 orders, 0 refused, 123 flagged of 1752 hands' lines; landing median 7 s (p90 7), turn wall median 9 s
   orders: {'wait': 182, 'instruct': 98, 'queue': 58, 'produce': 27, 'signal': 10, 'lane': 5, 'remove': 2}
   flagged: {'hunt-ended-leash': 39, 'step-skipped': 21, 'stopped-on-stop': 14, 'follow-ended': 12, 'step-waits': 10, 'engine-refused': 9, 'hunt-ended-other': 8, 'site-blocked': 6, 'other-fault': 3, 'abandoned': 1}
```

How the flagged lines were judged:
- **Engine refusals and site-blocked lines at the base (B7/B8):** the solar, advsol and nano builds were placed again within 3 s, or another builder put up the same nano. They cost nothing and are not counted.
- **The 4:25 dropped-step line:** the turret at spot_62 stood at 4:29.
- **The 26:23 one-unit produce lists:** idle on purpose ("plant_27255 and plant_7982 stay off so metal goes to Bulls", 27:07).
- **hunt-ended-leash:** 14 of the 39 hunts ended 0-3 s after they were played. That is the fragmentation Opus fought from 17:00 on ("Western hunters fragmented into 2-unit groups again").

## What Opus wanted that the harness has no lever for

- **21:18 and 22:26, "group_A2: it follows group_F1 wherever group_F1 goes, whole, and fights beside it".** The plan in the 20:36 note was "arriving together at 2:1 over the nest". A follow is a trail with no pace: no move keeps a follower beside its ward or makes two bodies arrive together.
- **16:57, 21:08 and every raider-hunter paragraph after: "one body that never splits ... corgator at our extractors ... it attacks them and chases them off".** Three things together leave no way to do it:
  - At 4x odds the whole-group attack is withheld (WHOLE_GROUP_OVER).
  - The same sentence forbids every detachment.
  - A hunt ends at the 900 leash.
- **22:17, the note "Wish: a way to force a holding group to re-pick (it kept 'hold' for minutes against the packet)".** No tool makes the hands ask a holding actor again. A new packet opens it for one ask; otherwise the news layer keeps it closed.
- **2:17, "Rover groups ... an enemy corfav or corgator at our home buildings is attacked first, whole. Otherwise they rove".** A roving group has no attack move, and nothing sends it back to roving after it answers.

## Commands run

All from the repository root, with `M=run/matches/1791508617-rove-task-1/00`. One seat only, so no symlink directories were needed.

Repository tools:
```
cat $M/../results.jsonl; python3 -c "...json.load(open('$M/../batch.json'))"
run/impedance_sweep.py $M --counts;  run/impedance_sweep.py $M --seat 0
run/minutes.py $M;  run/wake_read.py $M;  run/raid_ledger.py $M
run/pro_baseline.py $M  (no carded side from B8)
run/pro_baseline.py $M --all-starts --until 10
run/commander_turns.py $M --calls;  run/commander_turns.py $M --notes
run/hands_window.py $M <window> <actors>
```
hands_window was run on these windows: 1:05-2:12, 1:30-2:40, 2:24-2:36, 3:46-4:12, 8:40-8:56, 9:38-9:58, 11:26-11:32, 11:33-12:05, 16:03-16:14, 17:04-17:20, 17:36-17:50, 21:12-21:28, 21:54-22:12, 22:13-22:24 and 24:08-24:20.

Scratch scripts, all read-only, under the scratch directory's `agents/1791508617-rove-task-1/bin/`:
- `dump_calls.py` and `dump_think.py`: every call in full, and the player's turn text.
- `extract.py`: one pass over jev-0.jsonl.zst, writing out the decode, plan, audit, packet, group, pass and worlds_gate rows.
- `ctx.py`: truth units within 1,500 of each group, idle builders and the store.
- `menu.py`: one actor's full menu at a given second.
- `places.py`: place coordinates.
- `didcheck.py`: the report's hands block against the log's plays at 9:47, 16:12 and 22:17. All three matched.

Also: inline record queries for created/destroyed/placed events, the truth file at moment clocks, and code reads of `crates/bot/src/brain/pianist/decode.rs` (the 0.3 FUSE bar) and `menu.rs` (WHOLE_GROUP_OVER, roving menus).
