# Impedance review: 1791573192-nuke-1-medium/00

The game: an arena duel on 2026-10-09 on Comet Catcher Remake 1.8. One seat (strategist-0), Opus 5.5 medium, no commander, against BARb medium. The objective was the nuke. It ended undecided at the 40:00 cap. The silo stood at 17:31 and fired 8 missiles. The first one wiped out his G2 base at about 21:30. His commander survived to the end.

## Tally

| category | count | | category | count |
|---|---|---|---|---|
| A1 refused at the tool | 7 | | B1 in the hands' picture, not in the report | 2 |
| A2 accepted, not executed | 4 | | B2 in the record/truth, in nobody's picture | 0 |
| A3 overruled by the pick | 7 | | B3 in the report, wrong or stale | 2 |
| A4 overruled by code | 3 | | B4 in the report, missed by Opus | 1 |
| A5 late | 0 | | B5 the hands' own lines misleading | 1 |
| A6 read differently | 4 | | B6 commander-to-player hop | 0 (no commander) |

Denominators:
- **Judgement moments:** 9 examined, 4 clean (C). The brief asks for 8. I added (e), the last turns, because his commander walked past the main army there.
  - (a) the opening: A3.
  - (b1) the 7:23-8:30 corak raid: A3, A6, A6.
  - (b2) the 15:24-16:44 block at spot_58/59/64: C.
  - (b3) the 18:24-19:10 west block and the solar_field raid: A3.
  - (b4) the 21:08-22:30 block and the silo under fire: C.
  - (c) the first attack order at 25:43, inside the 22:00-40:00 window at 1.5x or more: C. The windows at 2:00 and 9:00 lasted one minute each, and no push was ordered in them.
  - (d1) the 13:12 packet: C.
  - (d2) the 26:43 packet: A3, B3. Its sequel at 29:05 was a B4.
  - (e) 39:08-40:00: A3, A4.
- **Orders:** 439 (213 of them `wait`).
- **Hands' lines:** 2222, of which 197 were flagged.
- **Refused:** the sweep counted 6, all `fire` calls with "no missile ready". The tally has 7, adding the 37:23 `signal` ("not a tool that can be batched"), which the sweep's count missed.

The remaining items were found as follows:
- Items from the sweep's flagged lines: 12:20 A4, 28:34 A3, the A2 items, 32:38 A6 and 36:36 B5.
- Items that check claims in Opus's notes: 6:01 A6, 34:28 A4 and 35:09 A3.

## Items by cost (15 of 31; all 31 are in impedance.json)

1. **39:31, A4, game, low confidence.**
   - Wanted (39:08): "while that corcom is in sight, the whole group attacks it, never a detachment."
   - Happened: his commander was in sight 39:31-39:35, about 1,050 from group_T2 (93 soldiers, 31k metal). Its menu offered only `send_1/2/4/8_party_195`, the decode had marked detachments forbidden (0.83), and nothing was played at the commander. He ended at 100% health.
   - Cause: the menu withholds the whole-group attack when the body outweighs a party four times (`WHOLE_GROUP_OVER = 4.0`, listed in code-decisions.md).
   - Fix: code-rule. Keep the whole-group attack on a party that holds his commander, or on a party the packet names.
2. **39:16, A3, large, low confidence.**
   - Wanted (39:08): "its way is spot_18, then spot_20, then spot_23".
   - Happened: the picks were `advance to his_g2` and then `advance to spot_20`. spot_18 was skipped, though his commander walked past it (about 350 away at 39:30). The body kept walking to spot_20 after the 39:34 packet dropped it, until 39:56.
   - Evidence: decode 39:16 read spot_18/20/23 at 0.98; the order of a route is not decoded.
   - Fix: menu (a route as one move, or the next unreached waypoint only).
3. **20:00-37:13, B1, large, medium confidence.**
   - What was missing: no report named the silo's rounds. Only `situation` had them ("stockpilers: armsilo_17869 ... 0 missiles ready, 10 stockpiling"), and Opus called it once, at 18:39.
   - Happened: rounds came ready every 120 s from 20:00. Opus fired 6 times before a round was ready (the A1 items). Rounds sat unused for 67, 66 and 67 s. Three rounds went unseen from 32:47 to 37:13, while the 33:43 note said "nukes landed already" and sent the army instead. Opus learned of them only from the 37:13 fire result ("2 missiles left ready"). Three rounds were unfired at 40:00.
   - Fix: report. The missiles line and the missile_ready wake were built later, in b0d7213.
4. **26:23, B3, large, high confidence.**
   - What was wrong: his E4 lab died to our third missile at about 25:22, out of sight. The report kept "Bot Lab corlab at E4" under `to win` until 40:00. It also said "his, in E4 (all of it in sight now): 1 corlab" at 28:09-29:05, while our army stood there.
   - Happened: the 26:23-29:05 attack was routed to a dead lab. Missile #5 (29:13) landed on no enemy unit and killed our Janus.
   - Fix: report. Clear a remembered building when its ground is in sight and empty.
5. **27:42, B3, large, high confidence.**
   - What was wrong: his G2 lab and advanced lab died at about 21:32 (missile #1). Both the `to win` line and the G2 scouting line kept the lab to the end. At 38:31 the report said "G2 (all of it in sight now): 1 corhllt, 1 corlab, 2 corllt, 3 corsolar", while the truth had nothing of his in G2.
   - Happened: missiles #4, #6 and #7 were aimed at "his main lab G2". #6 landed on no enemy unit. Opus's 38:31 note: "G2 lab (still standing after the nukes)".
   - Fix: report.
6. **7:48, A3, small, medium confidence.**
   - Wanted (7:36): "first the corak party at spot_61 nearest home ... No group ... hunts single corak on its own".
   - Happened: `group_L: attack party_4 (1 corak) with the whole group`. That corak was 1,500 north. party_10 (6-7 corak) killed the advanced plant frame at avp_yard at 7:48, then constructors and home extractors. group_L reached party_10 at 7:59.
   - Fix: jev.
7. **7:43, A6, small, medium confidence.**
   - Wanted (7:36): "commander: the corak party ... near home is its fight ... using its D-gun".
   - Happened: the commander's list ended in `assist`, so it kept "helping lab_27255 build" and was not asked until the 7:46 cancel landed at 7:53. It attacked at 7:57. Opus's 8:23 note: "commander was on an assist list".
   - Fix: tool. The instruct result or the report could say that an actor working a list does not read its packet sentence.
8. **7:44, A6, small, medium confidence.**
   - Wanted (7:36): "Every Blitz group ... joins group_L".
   - Happened: group_M, N and O were born at 7:40-7:43, after the report and before landing, so the packet could not name them. The decode read their joins at 0.15-0.16 and fused them off. group_M chased lone corak (7:49, 7:50, 7:52) until the 7:46 packet named it.
   - Fix: code-rule. Count `newer` from the report the packet answered, not from its landing.
9. **2:29, A3, small, high confidence.**
   - Wanted (2:20): "group_A ... roves, whole, now and from here on ... never walks to spot_59".
   - Happened: group_A finished its walk to spot_59 and held there until 4:06. Its menu rated `rove` 0.60-0.78 every second, but the pick kept "nothing changes". The first soldier reached his half at 4:24 (pool p75 1:49).
   - Fix: jev.
10. **18:47, A3, small, high confidence.**
    - Wanted (18:39): "the commander walks to solar_field and D-guns them".
    - Happened: the menu rated `attack_party_63` 0.42 against `help_plant_27255` 0.52. The PICK at 18:48 kept the help, and two solars died. The decode read solar_field at 0.97.
    - Fix: jev.
11. **28:34, A3, small, high confidence.**
    - Wanted (28:19): "The body never turns back for raiders ... [only] armflash ... sends four".
    - Happened: six detachments of 4-8 went at one corthud between 28:34 and 28:44, each ending at once on the 900 leash.
    - Fix: jev. Also, the menu offered hunts already past their leash.
12. **26:48, A3, small, medium confidence.**
    - Wanted (26:43): "never walks back west of spot_46 ... a lone corak is not its business".
    - Happened: group_T2 attacked lone corak/corthud at 26:48, 26:57 and 27:25, and walked about 1,600 west of spot_46 before turning back at 27:28.
    - Fix: jev.
13. **12:20, A4, small, medium confidence.**
    - Wanted (11:38): two Blitzes kill "one or two corak at our extractors".
    - Happened: group_K1's hunt of party_38 began 1,500 away and ended after 4 s on the leash. The corak then killed spot_52 at 12:26.
    - Fix: code-rule (the leash).
14. **34:28, A4, small, medium confidence.**
    - Wanted: the way spot_46, spot_40 and on.
    - Happened: under the default footwork, the march rule held 97 soldiers at (4099,3653) for 30 s, "19 of 95 arrived, its tail 1532 short ... stalled". The body moved only after Opus dropped march (34:52) and set raw (35:00).
    - Fix: code-rule.
15. **23:47, B1 + A2, small, high confidence.**
    - What was missing: report entries never name a constructor's type. After the 23:38 takeover, Opus gave tier-1 retake lists to armacv at 23:47, 23:57 and 25:03.
    - Happened: the tool accepted all 12 steps; the hands skipped them as "this builder cannot build it". Opus noticed at 25:53.
    - Fix: report (name the type) and tool (refuse a step the builder cannot build at the call).

Smaller items, all in the JSON:
- 35:09, A3: the army went south-east to spot_42 and chased artillery.
- 29:05, B4: Opus fired a missile at e4_lab while group_T2 stood on it. The fire tool names nothing of ours within 640, as the self-destruct tool does.
- 6:01, A6: four `armsolar solar_field` steps without ids were "taken as done" as one.
- 32:38, A6: `armsilo #silo2` "needs a place", though the brief says a placeless silo goes to the back field. The 17:20 #silo1 skip was the same.
- 5:08, 17:47 and 30:28, A2: "this builder cannot build it".
- 36:36, B5: "engine refused a armsilo at A8" in the second the silo stood.
- Seven A1 refusals.

## The sweep's counts (run/impedance_sweep.py --counts), verbatim

```
-- strategist-0.jsonl: 213 turns, 5 session replacements, 439 orders, 6 refused, 197 flagged of 2222 hands' lines; landing median 7 s (p90 7), turn wall median 6 s
   orders: {'wait': 213, 'instruct': 80, 'queue': 73, 'produce': 37, 'fire': 14, 'mark': 8, 'lane': 8, 'signal': 3, 'overview': 1, 'situation': 1, 'say': 1}
   refused: {'?': 6}
   flagged: {'hunt-ended-leash': 62, 'step-skipped': 48, 'hunt-ended-other': 24, 'step-waits': 17, 'engine-refused': 12, 'other-fault': 11, 'follow-ended': 11, 'site-blocked': 5, 'stopped-on-stop': 4, 'abandoned': 3}
```

Flagged lines I judged and did not tally:
- **Engine-refused and site-blocked extractors** (14:40, 26:12, 26:13, 28:35, 29:06, 21:58, 24:27): in each case another builder of ours was on the same spot, and the extractor stood within seconds.
- **Most leash ends** (5:29, 7:39, 9:52, 10:27, 16:09, 17:47, 25:13-25:20): either the hunt was one the packet in force did not want, or no loss followed.
- **"Assist ... dropped" lines** (11:36, 11:46, 31:54): the helpers helped anyway through picks.
- **Code placement in plant_27255's exit lane:** an extractor (armmex_15017, later armmoho_318) stood in the lane all game. This is the exit-lane bug, fixed in 3aabc77. Opus never asked for it, and chose to leave the lane as it was at 12:36 and 37:13.

## What Opus wanted that the harness has no lever for

- **39:08 and 39:34:** "while that corcom is in sight, the whole group attacks it, never a detachment".
  - For a party the body outweighs four times, the menu has no whole-group attack, only detachments, and the packet forbade those. This is also the A4 item at 39:31.
- **25:13 to the end:** "A constructor never stands in a plant's exit lane".
  - A builder that helps or builds beside a plant stands where the engine puts it, and no move helps from outside the lane.
  - The lane wake still named constructors working there at 29:41, 31:16, 32:31, 33:58 and 39:59.
- **3:35:** group_A "kills his extractors, constructors and solars where they are undefended, and it never attacks turrets" along a named route.
  - A walk fights nothing. An advance fights "everything here, turrets included". Only roving skips guarded targets, and roving takes no route.

## Commands run (from the repository root, M = run/matches/1791573192-nuke-1-medium/00)

```
cat $M/../results.jsonl; python3 -c "import json;print(json.load(open('$M/../batch.json')))"
run/impedance_sweep.py $M --counts;  run/impedance_sweep.py $M --seat 0
run/minutes.py $M;  run/wake_read.py $M;  run/raid_ledger.py $M;  run/pro_baseline.py $M --all-starts
run/commander_turns.py $M --calls
run/hands_window.py $M 7:36 8:22 commander; 7:43 8:22; 7:46 7:56; 2:25 3:50 group_A; 3:46 4:30 group_A;
  12:10 12:28; 13:17 14:47 group_M1; 16:15 16:45 group_M1; 17:44 17:56 group_P2 group_O2 group_Q2;
  18:44 19:10 commander group_M2 group_P2 group_M1; 21:26 22:30 group_T2; 26:48 28:18 group_T2;
  28:28 28:46 group_T2; 33:50 35:50 group_T2; 38:13 40:00 group_T2 group_X7
scratch scripts (read-only, in the scratch directory): tx.py (turn prompts and calls), pass1.py and dec.py
  (decode rows), state.py (picture entries), menu.py (menu move ids), rec.py, mex.py and losses.py (record events),
  truth.py and impact.py (truth lifetimes, missile landings against the truth)
git show e6fa397:crates/bot/src/brain/pianist/lists.rs; grep of menu.rs, decode.rs, report.rs and code-decisions.md
```
