# Unfinished structures: nuke-2-hard

Lost 32:22 to BARb hard_aggressive on Comet Catcher Remake 1.8. Arena game, one seat (Opus 5.5 medium as the player), layers news,same,fuse,openers,split,tick,places,again,near. The game's objective was the nuclear silo.

## Denominator (seat 0, verbatim)

```
1791577775-nuke-2-hard seat 0, record to 32:13: frames 211 (decayed 22, finished 177, killed 9, unfinished at end 3), metal in unfinished frames 3935, build orders never started 52
```

`run/floor.py` gives `aband` 22, which equals frames.py's decayed count.

**What "decayed" means in this game.** frames.py calls a frame decayed when its `destroyed` event names no attacker. In this game:
- **18 of the 22 were killed from out of sight (1,627 metal).** Each took enemy damage, in the record's `dmg`, in the second it died, and its builder was on it or had just died. The attackers stood out of our sight in the truth file:

  | attacker | frames |
  |---|---|
  | corvroc | 8 (its rocket does 1,700; four of the hits read 1,700-1,704) |
  | corwolv | 2 |
  | corpun | 2 |
  | corgol or cormart | 2 |
  | corban | 1 |
  | corsok | 1 |
  | unidentified | 2 |

- **4 were true decays, with no damage.** Three were the player's own `stop` (637 metal). One decayed after its builder was killed (armbeamer_19122, 30 metal).
- **The fate columns below keep frames.py's labels,** so this game sums with the other games. Each item also carries a `fate_read` field with the reading above.

## Tally (34 frames, 3,935 metal)

| cause | n | metal |
|---|---|---|
| U3 player re-listed (stop) | 3 | 637 |
| U4 frame killed / builder died | 28 | 2,903 |
| U8 game end | 3 | 395 |
| U0, U1, U2, U5, U6, U7, U9 | 0 | 0 |

U9 appears only as the `also` of the three U3 items.

| fate (frames.py) | n | metal |
|---|---|---|
| decayed | 22 | 2,294 |
| killed | 9 | 1,246 |
| at end | 3 | 395 |

| type | n | metal |
|---|---|---|
| armfus | 1 | 1,005 |
| armpb | 6 | 1,427 |
| armadvsol | 3 | 637 |
| armferret | 1 | 327 |
| armllt | 8 | 299 |
| armmex | 12 | 135 |
| armmoho | 2 | 75 |
| armbeamer | 1 | 30 |

**What the player knew:**
- **Before (18 frames):** the report named the frame and its share before the cause acted.
- **After only (14 frames):** it learned of the frame from the abandoned or lost line.
- **Never (2 frames):** both are the U8 frames that started in the last 5 s.
- **Dropped it itself (3 frames):** all three U3 frames, and it did not know they stood.

The player was told "abandoned" for all 22 decayed frames. For 19 of them that was false: 18 were kills and 1 lost its builder. It believed the lines: "two half-built Pit Bulls were abandoned" (28:30), and "its half-built Pit Bull at spot_71 decayed when the hands moved it away" (31:55).

## Items, ranked by metal

1. **19:50 · armfus_10810 · A8 back_field · U4 · large · medium.**
   - What happened: one corgator (party_44) reached the fusion at 19:36. The 19:25 packet gave back_field to group_Q1/group_T1: "The commander attacks and D-guns raiders at home too". The commander's attack_party_44 rated 0.21-0.26 and was never played. group_Q1 was sent at 19:43 ("attack party_44 (1 corgator) with the whole group"). The frame died at 30% (1,005 of 3,350). constructor_17595 stayed on it.
   - The player knew: the 19:40 report said "enemy in sight: 1 at A8 ...: corgator 1; killing our Fusion Reactor (armfus) (3350 metal) now", and the player's turn was a wait.
   - Fix: hands-rule.
2. **28:40 · armpb_12693 · C6 spot_63 · U4 · large · high.**
   - What happened: constructor_15597 stayed on it ([build] change=0.72-0.80). One 1,700 hit at 28:41 killed it at 50%. The corvroc was 507 away, unseen.
   - The player knew: the 28:30 and 28:40 entries said "(30 % built)" and "(50 % built)", with only corca/corfink near. The 28:50 report said "abandoned an unfinished armpb (50% built)".
   - Fix: report.
3. **32:13 · armferret_26787 · C7 spot_69 · U8 · none.** Started 31:12; 99% when the record ended.
4. **31:54 · armpb_1528 · D7 spot_71 · U4 · large · high.**
   - What happened: two 751 hits (corsok lasers) at 31:51 and 31:55. The two corsok were 534-636 away and showed as "unidentified". The builder stayed ([build] change ≈0.8).
   - The player knew: the 31:36 and 31:46 entries said 18% and 37%. Then a false line: "31:54 abandoned an unfinished armpb at D7 (26% built): its builder was sent elsewhere and the frame decayed".
   - Fix: report.
5. **28:28 · armpb_1106 · C7 spot_69 · U4 · large · high.**
   - What happened: it was on the 27:26 queue `["armpb spot_69"]`. One 1,700 hit; the corvroc was 1,260 away, unseen.
   - The player knew: the 28:15 and 28:23 entries said 19% and 35%. The 28:30 report said "abandoned ... (44% built)".
   - Fix: report.
6. **28:54 · armpb_22119 · D7 spot_64 · U4 · large · high.**
   - What happened: one 1,701 hit at 28:55; the corvroc was 748 away.
   - The player knew: the entries said 11% and 31%; then "abandoned ... (39% built)".
   - Fix: report.
7. **12:37 · armadvsol_27075 · C7 · U3 (also U9) · large · high.**
   - What happened: the 10:30 entry gave the neighbour's share, "(98 % built)" (armadvsol_5903 at 98%; 27075 was at 26%). From 10:32, the second after 5903 finished, the entry read "finishing an order". At 11:00 the player queued `["stop","armadvsol","armadvsol","armadvsol"]` ("I dropped whatever they were on"). The stop landed at 11:08 with the frame at 66%, and the new frame went up 317 away.
   - The player knew: no; it was misled. After the fact: "abandoned ... (0% built)".
   - Fix: report.
8. **13:32 · armadvsol_29443 · B7 · U3 (also U9) · large · high.**
   - What happened: from 11:18 the entry showed armadvsol_27863's "78 % built", and at 11:36, the second 27863 finished, it read "finishing an order". At 12:07 the player sent "stop" first ("I made it drop whatever it was doing, which loses any partly built frame"). The stop landed at 12:11 with the frame at 60%. The builder then got stuck at home.
   - The player knew: no.
   - Fix: report.
9. **12:25 · armadvsol_5192 · C7 · U3 (also U9) · large · high.** Same mechanism and the same 11:00 stop as item 7. The frame was at 56%; the builder was sent to B5.
10. **28:49 · armpb_25629 · D6 spot_59 · U4 (corraid, seen) · large · medium.**
    - What happened: a corraid and two correap at 209-254 killed it at 25%.
    - The player knew: the 28:40 entry said 14%, with only corca/corfink near. The 28:50 report said "lost armpb to corraid".
    - Fix: player.
11. **23:40 · armllt_28157 · D3 spot_24 · U4 · small · high.**
    - What happened: built to 97%; a 1,439 hit followed. The corvroc was 862 away and had never been seen.
    - The player knew: the entries said 55% and 93%; then "abandoned ... (99% built)".
    - Fix: report.
12. **23:56 · armllt_13783 · D5 spot_47 · U4 · small · high.**
    - What happened: an 815 hit at 92%; the corvroc was 766 away.
    - The player knew: the entries said 28% and 65%; then "abandoned ... (93% built)".
    - Fix: report.
13. **25:28 · armllt_9995 · C4 spot_38 · U4 · small · high.**
    - What happened: a 1,702 hit at 72%; the corvroc was 980 away (last seen 1:09 earlier).
    - The player knew: before, at 27% and 50%.
    - Fix: report.
14. **32:13 · armmoho_31484 · C8 · U8 · none.** Started 32:08.
15. **28:24 · armpb_4877 · C6 spot_61 · U4 · small · high.**
    - What happened: one 900 hit (a corgol shell); two corgol were 479-519 away, unseen.
    - The player knew: before, at 7%.
    - Fix: report.
16. **23:10 · armmex_7871 · C3 spot_24 · U4 · small · high.**
    - What happened: a 1,704 hit at 66%. The 23:08 "next, queued" was queued behind the build; the builder never moved.
    - The player knew: only after.
    - Fix: report.
17. **13:02 · armllt_16133 · D5 spot_44 · U4 · small · high.**
    - What happened: one corwolv shell hit the builder (267) and the frame (263). The three corwolv were 714-748 away, unseen.
    - The player knew: before, at 15%.
    - Fix: report.
18. **24:56 · armllt_26968 · D6 spot_56 · U4 · small · low.**
    - What happened: a correap killed the builder at 24:41. The frame was then hit unseen (37+74); a corgator stood 173 away.
    - The player knew: before, at 15%.
    - Fix: hands-rule. The correap hunts ended on the leash.
19. **29:30 · armbeamer_19122 · C7 spot_69 · U4 · small · high.**
    - What happened: the builder was killed at 29:09 from out of sight (1,410). The frame took no damage and decayed.
    - The player knew: only a false line after: "its builder was sent elsewhere and the frame decayed".
    - Fix: report.
20. **27:45 · armmex_10759 · C6 spot_61 · U4 · small · high.**
    - What happened: one rocket killed both the builder (1,381) and the frame (1,704). The corvroc was 1,110 away.
    - The player knew: only after.
    - Fix: report.

**The other 14 (116 metal).** 13 are U4 (104 metal). The fourteenth is armmex_19326 (U8, 12 metal), started at 32:08.

Killed by seen raiders:
- armmex_28931 3:41 (corgator)
- armmex_30526 4:57 (corfav)
- armmex_20308 11:16 (corgator; the list played the step into party_27's 4 corgator)
- armmex_11572 12:04 (corgator)
- armmex_29237 12:05 (corgator; the pick walked the builder off at 12:02)
- armmoho_7623 19:02 (corraid; the list started it with party_42 666 away, as the 18:58 entry said)
- armllt_3117 19:33 (corgator, home raid)

Killed from out of sight:
- armllt_24376 12:32 (corwolv; the builder died first)
- armmex_12149 19:56 (corpun, 350)
- armllt_2772 20:03 (corpun splash)
- armmex_28804 24:47 (unidentified, 97)
- armmex_321 28:38 (cormart/corgol)
- armmex_23205 29:59 (corban; the builder with it)

## Builds never started (52)

- **21 started after all.** The frame came from that very order at the engine's `placed` site, 306-624 from the command's point. frames.py measures its 300 radius from the command. Or the order was one building shared with its id's other builders. These include the silo, both fusions, both advanced plants, the Mercury, the Chainsaw, the flak gun, two Ferrets and two advanced solars.
- **5 spots another of the player's lists took first** (U2, cost none).
- **7 game end** (U8, the last 90 s).
- **19 real:** U4 7, U3 5, U1 3, U2 2, U6 2. The costliest:
  - **27:34 constructor_2009 armpb, C5 (U4, large).** It walked 2,473 toward spot_52 and was killed from out of sight at 28:21, 669 short.
  - **19:03 constructor_31089 armmoho, D6 (U4, large).** Its list's next step after the corraids killed the D5 moho; it died to a corraid at 19:12.
  - **19:08 constructor_18465 armmoho, D6 (U4, large).** The pick walked it home from party_45 at 20:01. The corgator killed spot_56's extractor at 19:57, and at 20:09 the list said "nothing to do: no extractor of ours stands at spot_56".
  - **28:39 constructor_5199 armbeamer (U4).** Its builder died at 28:40.
  - **22:17 constructor_1544 armllt spot_26 and 3:43 constructor_10476 armmex spot_64 (U2).** A list re-sent with the order not yet begun as its first step, and the hands took it as done: "the step '...' of its list is the build already under way: taken as done". No frame stood, so the turret at spot_26 was never built, and spot_64 got a turret beside an empty spot. H-HANDS-LISTS says this was fixed on 2026-10-16.
  - **28:13 constructor_31327 armrad home (U6 rejected).** "the engine refused a armrad at B8: the site was bad". Radar was the player's answer to the unseen fire; it stood only at 29:57.
  - **13:06 constructor_20491 armmakr (U6 stuck).** 144 `move_failed` from 13:22.
  - **The rest are cost none:**
    - 2:37 commander armsolar, U3: the 2:45 list replaced the old one.
    - 21:46, 22:12 and 28:16 pick turrets, U3: displaced by the player's new lists.
    - 26:28 and 28:52 commander turrets, U1: the pick walked the commander home before they began.
    - 24:48 and 14:12, U4: the builders died.

## Three mechanisms

1. **Fire from out of sight is never said for builders or frames, and the frames it kills are called abandoned (report; up to 1,627 metal, 18 frames).**
   - One corvroc killed 8 frames with their builders on them, 1,173 metal: armmex_7871, armllt_28157, armllt_13783, armllt_9995, armmex_10759, armpb_1106, armpb_12693 and armpb_22119. The truth file has it from 20:44 to 29:06; it was in our sight once, at 24:16.
   - corwolv, corpun, corgol, cormart, corban and corsok killed 10 more (454 metal).
   - The shelling wake that fired in this game names soldiers ("our soldiers are being shelled from out of sight"). The frames' losses came as "abandoned ... its builder was sent elsewhere and the frame decayed", and the player believed them.
   - What would have helped: a line naming a builder or frame being shelled, and an abandoned line that reads the frame's damage ("destroyed while being built, from out of sight"). These would have told the player from 23:10 that its D3/C4 rebuilds and the 28:xx Pit Bulls stood in a rocket truck's reach.
2. **A builder's build is matched to any frame of its type within 200 of its site (report and hands bookkeeping; 637 metal, 3 frames).**
   - In the packed advanced-solar field at B7/C7, three builders' entries showed a neighbour's share, then "finishing an order" the second the neighbour finished: constructor_10629 and constructor_11249 after armadvsol_5903 at 10:31, constructor_20491 after armadvsol_27863 at 11:35.
   - The player read "finishing an order; standing in the exit lane" as stuck and sent `stop`, dropping armadvsol_27075, armadvsol_5192 and armadvsol_29443 at 56-66%.
   - Matching by the builder's own `created` frame would have kept all three.
3. **The home guard answered one corgator at the back field too late (hands-rule, medium; 1,005 metal, 1 frame).**
   - The packet covered back_field. The commander's attack was never played, and group_Q1 was sent 7 s after the corgator reached the fusion.
   - armfus_10810 died at 30%; armllt_3117 (2 metal) died in the same raid.

## Commands run

```
cat MATCH/../results.jsonl; python3 -c "...json.load(open('MATCH/../batch.json'))"
python3 run/frames.py MATCH --seat 0 [--all] [--json]
python3 run/floor.py MATCH
python3 run/impedance_sweep.py MATCH --seat 0 --all --from 10:00 --to 10:56 | 10:55 11:12 | 19:15 19:50 | 18:40 19:05 | 2:30 2:55 | 3:40 4:00 | 22:10 22:30 | 28:05 28:25
python3 run/impedance_sweep.py MATCH --seat 0 --from 27:15 --to 28:35
python3 run/hands_window.py MATCH 10:29 10:42 constructor_10629 constructor_11249 | 19:36 19:51 constructor_17595 commander | 28:34 28:41 constructor_15597 constructor_5199 | 31:45 31:55 constructor_29730 | 18:52 19:03 constructor_31089 | 12:01 12:03 constructor_18284
python3 run/commander_turns.py MATCH --calls
scratch scripts (python3 -I, read-only): record dmg/xi/xf per frame at its fate, builder track and deaths, truth enemies within range (corvroc 1310, corwolv 710, corpun 1245, corgol 650, cormart 800, corban 800, corsok 725) at each fate, every turn's builder entry and the line after, the jev log's constructor_10629/11249/20491 entries second by second (10:29-10:33, 11:05-11:46), never-started orders against placed sites and builder tracks
read: docs/studies/data/unfinished-2026-10-17/BRIEF.md, MATCH/impedance-review.md, run/frames.py, docs/harness/record-format.md (ev, s rows), docs/heuristics.md (H-HANDS-SCRIPT, -QUEUE, -STARTED, -LISTS), crates/combatsim/data/units.json (weapons), git show 6779238:crates/bot/src/brain/pianist/picture.rs (task_course)
```
