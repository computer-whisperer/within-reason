# Impedance review: apex-v024-carrot-sym50-opus-1

Won 16:40 against Apex v0.2.4 (standard profile) on Carrot Mountains v2.0, +50 income both sides, seed 3. One seat (strategist-0: Opus 5.5 medium) under an Opus 5.5 high commander (10 `direct` paragraphs), twelve hand layers, truth file read. The army was 1.5x his or more from 11:00. Every fault below came while we were ahead; none decided the game.

## Tally

| Category | Count |
|---|---|
| A1 refused at the tool | 2 |
| A2 accepted, not executed | 3 |
| A3 overruled by the pick | 3 |
| A4 overruled by code | 3 |
| A5 late | 0 |
| A6 read differently | 5 |
| B1 in the hands' picture, not the report | 0 |
| B2 in the record/truth, in nobody's picture | 0 |
| B3 in the report, wrong | 1 |
| B4 in the report, missed by Opus | 1 |
| B5 the hands' own lines misleading | 1 |
| B6 commander-to-player hop | 2 |
| C followed, context enough | 1 of 8 moments |

Denominators: 8 judgement moments, 1 clean. 191 player orders, 15 commander orders. 1,203 hands' lines, 66 flagged. Refused: 2. The sweep prints 0 because its REFUSED pattern looks for "is not a tool" and the batch's line reads "signal: not a tool that can be batched".

## Judgement moments

| Moment | What Opus wanted | What the hands did | Verdict |
|---|---|---|---|
| (a) Opening, 0:00-4:00 | Commander list (2 extractors, 2 solars, plant, assist) and produce list (cv, 2 fav, cv, 2 fav, ...); 1:40 wind turbines | All done in order: extractors 0:09 and 0:22, solars 0:31 and 0:41, armvp 1:10, cv 1:22, fav 1:26 and 1:30, 8 turbines by 2:40. The 3:00 `armnanotc` was skipped (A2) | not clean (A2, cost none) |
| (b1) 6:22 spot_87, Tick | 6:02 home guard: "sending one or two at a lone Tick" | party_17 seen 6:09; 6:14 one Blitz sent, 6:16 the whole of group_F. The Tick was about 2,300 from the guard and killed the extractor before they arrived | **C** |
| (b2) 6:48 spot_109, Tick | 6:23 home guard: "the Rovers first at Ticks" | group_J hunted; its hunt ended on the leash at 6:38; it was told to attack again at 6:41; the extractor died at 6:48 | A4 (low) |
| (b3) 11:41 spot_129, Pawns | 10:37 home guard at spot_109; 11:32 "go_g5 is given ... never chases a ... Pawn" | The home guard joined the army at 11:24 (A6). Two 8-soldier hunts ended at 0 s (A4). The whole army shelled the Pawns westward 11:34-12:03 (A3). The batched go_g5 signal was refused (A1) | not clean |
| (b4) 15:13 spot_56, Pawn | 14:34 "every Blitz group ... joins the others into one group ... the whole Blitz group attacks" | No join mark was read. The guard stayed in seven handfuls; nobody answered party_87 for 4 s (A6) | not clean |
| (c) break_g5 at 8:06 (first push) | 7:59 packet: "When break_g5 is given: it fights to spot_98" | The advance to spot_98 was played at 8:08. The commander gave the signal early on a group entry that overstated the body (B5) | not clean (B5) |
| (d1) packet 5:24, landed 5:32 | "group_D ... attacks as one whole body ... the Sentry at spot_98" | Advance at 5:34. The front reached spot_98 at 5:47 with the body strung out; the group walked back to spot_116. The player had misread the entry (B4) and overrode the commander (B6) | not clean |
| (d2) packet 10:37, landed 10:45 | "sends two Blitzes (a detachment of 2)"; "Its rally place is rally_e2" | A detachment of 8 went at 10:45 (A6). The group walked on to the commander's rally_east, which the player's null had not removed (A2) | not clean (cost none) |

Missing-context check at every moment:
- All 440 of the pick's plays in the jev log appear in the next report's "what your hands did" block (none missing, none replaced unlisted).
- Exit-lane blocks woke the player at 7:51, 12:58 and 14:50.
- The three engine-refused extractor sites (5:24 C6, 11:52 E5, 12:29 A2) already held our own extractor, built 5:08, 11:46 and 12:13. The refusals cost nothing.
- At 5:24 the report's enemy estimate ("7 worth 573 metal") was under half the truth near spot_98 (1,181 metal of soldiers and 3 Sentries). The report labels it a floor of what was seen, so it is not counted.
- At 7:58 (2,591 seen against 2,441) and 11:32 (about 4,000 against 5,900) the estimates were close enough that the decisions stand.

## Items by cost

1. **11:34-12:03, strategist-0, A3, large.**
   - Wanted (packet landed 11:40): "go_g5 is given, so it does not wait or gather any more: its way is spot_93 ... never chases a Tick, Pawn".
   - Happened: "11:34 group_H: shell party_59 from their reach with 9 long-reach soldiers, 52 between". party_59 was 2-4 Pawns in C6. The pick kept that course at 11:41, 11:50, 11:51 and 12:00 over go_spot_93 (0.53-0.61), with the decode at spot_93 0.93 and detachments forbidden at 0.90 or more.
   - Cost: the army's centroid went from 4,717 to 5,795 from his lab and got back only at about 12:26. That is about 50 s of the 69-soldier army after the go order.
   - Fix: jev. The pick's own course outlived an explicit go; the shell move also took the whole body after raiders.
2. **7:58-8:57, commander, B5, large.**
   - Wanted: "break_g5 is given now. group_H (19 units, 3,010 metal) is at spot_116 and outweighs what he has shown". The player had set 9:30.
   - The entry read "19 soldiers worth 3010 metal; at spot_116 ... within 800 of 0 of its 10", with no tail line. Only 10 Blitzes (1,100 metal) stood there; the 8 Stouts were 2,700-6,700 back.
   - Cost: all 10 Blitzes plus one more and a Stout died at G5/G6 8:15-8:57, 4 of them to his commander. About 1,435 metal, for about 1,000 of his by the commander's count.
   - Fix: report. Lead with what stands at the place and where the rest is.
3. **9:20-10:03, strategist-0, A3, large (medium confidence).**
   - Wanted (9:12): "Until push2 is given: it gathers at spot_93 ... Any party ...: the whole group attacks it ... then comes back to spot_93". After push2 at 9:40: "fights to spot_98 as one body".
   - Happened: "9:20 shell party_40", "9:30 shell party_42", "9:49 shell party_47". The group stayed on standoff until 10:03.
   - Cost: 6 Stouts (1,350 metal) died one at a time at G5/F5 9:27-10:03. The player said the same at 9:59; the commander read it as joiners trickling in.
   - Fix: menu. The shell screen at 0.55 of the Whistlers' reach stands the Stouts under his Sentries and Centurions.
4. **5:24, strategist-0, B4, small.**
   - Wanted (note): "group_D (14 Blitzes, 9 more coming) is at spot_98 ... 14 Blitzes clearly outweigh one Sentry and two Maces".
   - The report said 14 in all, 9 of them on the way: "0 of 5 arrived ... within 800 of 5 of its 5: an even fight ... goes in as one body or not at all".
   - Cost: 4 Blitzes (440 metal) at 5:47-5:53.
   - Fix: report (same shape as item 2).
5. **5:24, strategist-0, B6, small.**
   - The commander at 4:58: "Stay out of the two Sentries at G5 ... Do not go at his lab". The player: "departing from the direction's 'stay out of the Sentries'".
   - Cost: the same 4 Blitzes as item 4. Do not add it twice.
   - Fix: brief.
6. **11:33-11:41, strategist-0, A4, small.**
   - The home guard clause wanted Pawns at spot_129 attacked.
   - Happened: "group_W's hunt of party_59 ended after 0 s: the leash of 900 ... reached", and the same for group_Y at 11:39. The leash counts from the hunters' centroid, so a detachment of 8 from a strung-out army ends at once.
   - Cost: a turret (11:40) and the extractor (11:41).
   - Fix: code-rule.
7. **11:24, strategist-0, A6, small.**
   - Wanted: "Home guard: the group made by plant_3993 ... stands at spot_109".
   - Happened: "group_V: join group_H" (confidence 0.18). The decode never asked about group_V's own joins or places, only about others joining it.
   - Cost: the base was empty for the 11:33 Pawns; 4 units were re-made.
   - Fix: words (name the group), or the decode asking a new group's own reads.
8. **14:42-15:13, strategist-0, A6, small.**
   - Wanted: "every Blitz group in our half (group_M1, group_R1, ...) joins the others into one group".
   - The joins among them read 0.10-0.32: no mark, and several fell under the 0.3 fuse. The guard stayed in 1-2-Blitz handfuls, and party_87 at spot_56 was never answered.
   - Cost: extractors at spot_56, spot_114 and spot_129.
   - Fix: words (name one group to join).
9. **11:14-14:57, strategist-0, A6, small.**
   - Wanted: bare `armnanotc` after `armavp #avp1`, "construction turrets after".
   - Happened: the nanos stood 808, 999 and 1,135 from the advanced plant, outside their reach of 400; one was refused at B5 at 12:02. The plant had no help until 14:57. Its entries carry no "nanos" field when it has none.
   - The commander caught it at 13:58: "the advanced plant has nothing helping it".
   - Fix: words (`armnanotc spot_109`), or a report line saying the plant has no nanos.
10. **15:51, strategist-0, A3, small.**
    - Wanted: "Its way ... his_alab, then spot_100".
    - Happened: "15:51 group_H: advance to shelling" over his_alab (0.63), with the his_alab place read at 0.97.
    - Cost: about 14 s until the next packet.
    - Fix: jev.
11. **15:57-16:17, commander, B3, small.**
    - The reports counted our own moho upgrades as losses ("extractors lost in the last 3 min: 10", "lost armmex to unseen at home"); 6 of the 10 were ours.
    - The commander: "his Pawns killed 10 extractors in three minutes". He turned two plants to 6 Blitzes and 2 rebuild constructors.
    - Fix: report.
12. **9:16-9:59, strategist-0, B6, small.**
    - The commander at 8:58: "The Blitzes and Stouts of group_M walk to group_H now".
    - The player's 9:04 and 9:14 packets kept group_M home until the commander repeated it at 9:58.
    - Cost: 12 units idle about 45 s more.
    - Fix: brief.
13. **14:06-14:17, strategist-0, A4, small (medium confidence).**
    - Five home-guard hunts on Pawns in D2-D4 ended on the leash.
    - party_91 killed a constructor at 14:17.
    - Fix: code-rule.
14. **6:38-6:48, strategist-0, A4, small (low confidence).**
    - group_J's hunt of a Tick ended on the leash. It was re-sent at 6:41, and the Tick killed spot_109's extractor at 6:48.
    - Fix: code-rule.
15. **9:13-9:18, strategist-0, A6, none.**
    - group_H sent detachments of 1, 4 and 8 at a lone Tick while the packet gathered every soldier at spot_93.
    - The forbid reads were 0.58-0.74, under the 0.8 mark.
    - Fix: words.

Also in the JSON, all cost none:
- 10:45 A6: a detachment of 8 where the packet said 2 (forbid read 0.26).
- 10:27 A2: the mark tool answered "rally_east forgotten", but a player's null cannot remove the commander's side mark. group_H walked on to rally_east while the player's 11:13 note said "arriving at rally_e2".
- 3:08 and 14:56 A2: the queue tool accepted `armnanotc` for the commander and `armfus` for T1 constructors, and both were skipped with "this builder cannot build it".
- 6:52 and 11:32 A1: "signal: not a tool that can be batched". `batchable()` in mcp.rs lacks signal, and the orders tool's description leaves it out.

## Sweep counts, verbatim

```
-- strategist-0.jsonl: 86 turns, 2 session replacements, 191 orders, 0 refused, 66 flagged of 1203 hands' lines; landing median 7 s (p90 7), turn wall median 7 s
   orders: {'wait': 86, 'instruct': 35, 'queue': 34, 'produce': 23, 'lane': 5, 'mark': 4, 'signal': 4}
   flagged: {'hunt-ended-leash': 21, 'step-skipped': 16, 'hunt-ended-other': 13, 'engine-refused': 6, 'stopped-on-stop': 4, 'follow-ended': 3, 'step-waits': 2, 'site-blocked': 1}
-- commander.jsonl: 13 turns, 0 session replacements, 15 orders, 0 refused, 0 flagged of 0 hands' lines; landing median 0 s (p90 0), turn wall median 13 s
   orders: {'direct': 10, 'wait': 3, 'signal': 1, 'mark': 1}
   flagged: {}
```

The true refused count for strategist-0 is 2 (6:52 and 11:32), as noted under the tally.

How the flagged lines were judged:
- **hunt-ended-leash (21).** Items 6, 13 and 14 cover the raiders the packet wanted dead. The rest: the quarry died within seconds (9:15/9:18, 13:48/13:50), the quarry was in his half (6:17, 13:00), or the hunt is part of item 15.
- **step-skipped (16).** Mostly "holds our extractor already" or "is being taken by". The real ones are the two "cannot build it" items.
- **engine-refused (6).** The three extractors were duplicates of ours. The nano at 12:02 is inside item 9. 16:25 came too late to matter.

## What Opus wanted that the harness has no lever for

- **9:59 packet (landed 10:07), and the commander at 7:58 and 9:58.** "The Blitzes and Stouts of group_M ... join group_H" while "the Rovers (armfav) of group_M ... stand at spot_109". No move splits a group by unit type. group_M joined whole at 10:08, and the player wrote "the base relies on its turrets".
- **15:58 commander, 16:07 player.** "After the lab, group_H splits into two bodies of about 45 each." Detachments come only in 1, 2, 4 and 8. The player: "the hands cannot split a group by count".
- **8:25, 9:12, 10:37 and 11:32 packets; the commander at 8:58 and 11:28.** "Stouts in front, Whistlers (armsam) behind them shooting Sentries from their 550 reach." No move or lane word orders a formation by unit type or gives one type its target type. Shell is the nearest move, and it sets the rest of the group as a screen at 0.55 of their reach.
- **10:37 packet; the commander at 9:58 (partial lever).** "join group_H from there, six or more together, never one at a time into his yard." Joiners are walked singly by code; nothing waits for a count of joiners.

## Commands run

All run from the repository root, with M=run/matches/1791989571-apex-v024-carrot-sym50-opus-1/00.

Orientation and sweep:
```
cat $M/../results.jsonl; python3 -c "import json;print(json.load(open('$M/../batch.json')))"
python3 run/impedance_sweep.py $M --counts
python3 run/impedance_sweep.py $M --seat 0
python3 run/impedance_sweep.py $M --commander
python3 run/minutes.py $M
python3 run/wake_read.py $M
python3 run/raid_ledger.py $M
python3 run/pro_baseline.py $M        # no carded side on this map
python3 run/commander_turns.py $M --calls
python3 run/commander_turns.py $M --notes
```

hands_window windows:
```
python3 run/hands_window.py $M 6:08 6:26
python3 run/hands_window.py $M 6:36 6:50 group_F group_G group_J group_K group_L commander
python3 run/hands_window.py $M 5:30 5:56 group_D
python3 run/hands_window.py $M 8:04 8:36 group_H
python3 run/hands_window.py $M 9:19 9:52 group_H
python3 run/hands_window.py $M 10:43 10:48 group_H group_S group_T
python3 run/hands_window.py $M 11:14 11:33 group_H
python3 run/hands_window.py $M 11:19 11:25 group_V
python3 run/hands_window.py $M 11:39 12:05 group_H
python3 run/hands_window.py $M 15:02 15:14
python3 run/hands_window.py $M 15:07 15:14 group_S1 group_R1 group_M1
python3 run/hands_window.py $M 15:44 15:53 group_H
```

Scratch scripts (in the scratch directory's `tools/`):
- `turns.py`: turn prompts and tool results.
- `events.py`: record events.
- `scan.py`: one pass over jev-0.jsonl.zst for decode, instruction and plan rows.
- Inline python over `jevlog.rows`: worlds_gate forbidden flags, group_H's track, party ids against the record's kills, and the plays-against-report check.
- Inline python over the truth file: enemy metal near spot_98 and spot_116.

Code read: crates/bot/src/strategist/mcp.rs (`batchable`, `signal`, `mark`), strategist/shared.rs (`marks_all`), crates/micro/src/lib.rs (the hunt leash), brain/pianist/decode.rs (detach reads).
