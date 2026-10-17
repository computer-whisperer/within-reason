# Unfinished structures: rove-task-1 (1791508617-rove-task-1/00)

An arena duel on Comet Catcher Remake 1.8 (2026-10-08, commit 6ea433b), one seat (an Opus 5.5 medium player, Jev hands, layers news, same, fuse, openers, split, tick, places, again, near) against BARb hard_aggressive. We lost at 32:28; the record runs to 32:21. From 25:00 his tier-2 and tier-3 army came into our half.

## Denominators

frames.py, seat 0, verbatim:

```
1791508617-rove-task-1 seat 0, record to 32:21: frames 204 (decayed 9, finished 172, killed 19, unfinished at end 4), metal in unfinished frames 1519, build orders never started 48
```

- **floor.py:** `aband` is 9, equal to frames.py's decayed count.
- **floor.py's `aband_m` is 131 against 705 for the same 9 frames in frames.py.** Floor multiplies the share left after decay or damage; frames.py takes the peak.
- **Six of the nine "decayed" frames were killed by enemies out of our sight**, not left to decay: armllt_13435, armmex_11072, armmex_12528, armllt_8038, armhlt_30842 and armrl_8479, 489 metal together. The fate column keeps the brief's definition (destroyed with no attacker); the cause column says what happened.

## Tally (32 frames, 1,519 metal)

| cause | n | metal |
|---|---|---|
| U2 the builder's own list moved on (also U9) | 1 | 152 |
| U3 the player re-listed or moved the builder | 1 | 62 |
| U4 the builder died or fled, or the frame was killed | 26 | 838 |
| U8 game end | 4 | 467 |
| U0, U1, U5, U6, U7, U9 as the main cause | 0 | 0 |

U5 appears once as `also`: the armhlt_30842 frame (400 metal) stood at a metal store of 0. U1 appears once as `also`: armmex_14107 (2 metal).

| fate | n | metal |
|---|---|---|
| decayed (destroyed by nobody; 6 of them were unseen kills) | 9 | 705 |
| killed | 19 | 347 |
| unfinished at end | 4 | 467 |

| type | n | metal |
|---|---|---|
| armhlt (Overwatch) | 2 | 598 |
| armllt | 14 | 252 |
| armsolar | 2 | 214 |
| armmoho | 1 | 198 |
| armrl | 4 | 135 |
| armmex | 9 | 122 |

**What the player knew** (before, after and never cover all 32 frames):
- **before:** 18. The report had shown the frame standing before the cause acted.
- **after:** 12. The player was told only by an "abandoned" line or a "lost X to Y" line after the loss.
- **never:** 2.
- **dropped it itself:** 1, armsolar_26351. This one is also among the 18 "before".

**Fix classes:** hands-rule 12, player 11, none 6, report 3.

**Costs:** large 4, small 25, none 3.

## Items ranked by cost

1. **30:43 (started 28:06), seat 0, armhlt_30842 at C6 (spot_63), 91%, 400 metal. U4, also U5.**
   - What happened: the player's 25:17 list `['stop','armhlt spot_61','armhlt spot_63']` started it. constructor_10161 stood alone on it for 157 s with the metal store at 0 ("metal 1 (+56.8/-55.7)"), making about 1% every 1-3 s. Two hits of about 1,000 each, 7 s apart, came from his Banisher (corban 24612) 768 away under three coreter jammers. The truth file has the Banisher; our picture called it "unidentified".
   - Evidence: dmg 1003 at 30:36 and 1004 at 30:43; destroyed by nobody; the metal store at 0 in the s rows. At a full build rate (an armcv's 90 build power against 11,300) it would have stood near 30:12, before the first hit (an inference).
   - What the player knew: every entry from 28:21 "(10 % built)" to 30:33 "(90 % built)". It was then told "abandoned an unfinished armhlt (12% built)", the wrong cause and the wrong share.
   - Fix: player (spending priority). Cost large.
2. **30:25, armhlt_25880 at C8, 45%, 198 metal. U8.** Started 1:56 before the record's end, at a metal store of 0. The player saw it in every entry. Fix none. Cost large.
3. **31:57, armmoho_10735 at B8, 32%, 198 metal. U8.** Started 24 s before the end. Seen in the 31:59 and 32:15 entries. Fix none. Cost large.
4. **2:03, armsolar_28188 at B8, 98%, 152 metal. U2, also U9.**
   - What happened: the commander's opening list queued "help plant_27255" behind the solar at 2:09. At 2:10 constructor_18284's armsolar_904 finished 213 away. The hands' finished-event handler (`pianist/mod.rs`, which matches a started build of the same type within 250 of its site) took that for the commander's build and promoted the queued help. The guard went out at 2:11 with no queue flag and replaced the build; progress stopped at 98% at 2:12. The frame decayed by 3:19.
   - Evidence: "2:09 (list) next, queued: help plant_27255 build"; "2:10 finished armsolar_904"; "2:11 cmd guard 25947 27255".
   - What the player knew: the 2:07 entry showed the solar at 40%. After 2:11 the entries said only "helping lab_27255 build". At 3:26 it was told "abandoned an unfinished armsolar (0% built)".
   - Fix: hands-rule (match the frame's id, not type and distance). Cost large.
5. **32:00, armrl_3354 at C8, 81%, 65 metal. U8.** Started 21 s before the end. Seen in the 32:15 entry. Fix none. Cost small.
6. **3:20, armsolar_26351 at B7, 40%, 62 metal. U3.**
   - What happened: the player's 3:26 list `['stop','assist']` landed at 3:31 and dropped the commander's started solar. The 3:16 list had landed 3 s after the step began.
   - What the player knew: the 3:26 entry showed it at 18%. The player chose to drop it: "Metal is what we're short of ... Dropping it at 18% loses about 28 metal".
   - Fix: player. Cost small.
7. **31:06, armrl_28002 at C7 (spot_69), 72%, 58 metal. U4.**
   - What happened: his tier-2 and tier-3 block (corkarg, corgol, corshiva) came into our base, and a Shiva killed the frame at 31:38. The queue-ahead had queued the next step at 31:34, but the builder stayed on this one.
   - What the player knew: the 31:09, 31:19 and 31:29 entries showed it at 4%, 20% and 45%. The 31:59 entry still read "(not yet started)".
   - Fix: none. Cost small.
8. **21:41, armllt_25508 at D5 (spot_44), 47%, 40 metal. U4.**
   - What happened: an unseen corwolv shelled the builder from 21:48 ("hit this second by something unseen") and hit the frame at 21:55. Progress stopped at 47%, and corgator 26519 finished it off at 22:06.
   - What the player knew: the 21:48 report named "shelled from out of sight by a corwolv". The entries showed the frame at 24% and 22%.
   - Fix: player. Cost small.
9. **21:47, armmex_12528 at C3 (spot_26), 77%, 38 metal. U4.**
   - What happened: corgator 27825, 183 to 138 away in the truth file and out of our sight, took the frame from 77% to 8% between 22:04 and 22:06. The record says destroyed by nobody.
   - What the player knew: the entries showed it at 3% and 53%. It was told "abandoned an unfinished armmex (10% built)".
   - Fix: player. Cost small.
10. **3:05, armmex_6304 at C6 (spot_61), 73%, 36 metal. U4.**
    - What happened: this was the first forward extractor, with no turret. A lone corgator killed it at 3:49.
    - What the player knew: the 3:46 wake named it: "killing our Metal Extractor", "C6 (1 enemies, NO turret)". Its orders landed at 3:53, after the kill.
    - Fix: player. Cost small.
11. **12:16, armllt_13591 at E7 (spot_66), 41%, 35 metal. U4.**
    - What happened: the player had listed "armllt spot_66" for two builders (10:48 and 10:58) with no id, so two frames stood 192 apart. Corgator party_13 killed this one.
    - What the player knew: the 12:26 entry showed 37%. The same report's "E7 (1 enemies, turret)" counted the frames as a turret.
    - Fix: player. Cost small.
12. **11:17, armllt_1832 at C4 (spot_39), 34%, 29 metal. U4.** Three corgators killed it at 11:29. The player saw it at 1% and 30%, with a corgator in sight at 11:28. Fix player. Cost small.
13. **9:33, armllt_15814 at B4 (spot_36), 30%, 26 metal. U4.**
    - What happened: a corgator came at 9:38. The 9:37 report said "B4 (1 enemies, turret)", but the only turret there was this frame at 11%. `briefing.rs` `turret_within_300` counts frames being built.
    - Fix: report. Cost small.
14. **8:29, armllt_12458 at D6 (spot_59), 29%, 25 metal. U4.** The corgator came unannounced at 8:41 and killed the frame in 3 s. The player had seen it at 15% at 8:35. Fix player. Cost small.
15. **12:25, armllt_11366 at E7 (spot_66), 29%, 25 metal. U4.** The second turret of item 11; the same corgator killed it 5 s later. Fix player. Cost small.
16. **22:09, armllt_17753 at C3 (spot_26), 28%, 24 metal. U4.** The queued turret began 3 s after corgator 27825 killed the extractor beside it, with that corgator 222 away. Fix hands-rule. Cost small.
17. **21:15, armllt_13435 at D5 (spot_47), 22%, 19 metal. U4.** One 301-damage hit from an unseen corwolv 668 away (in the truth file). It was reported as "abandoned (24% built)". Fix report. Cost small.
18. **21:26, armmex_11072 at D5 (spot_44), 37%, 18 metal. U4.** The same corwolv, 515 away: one 301 hit. It was reported as "abandoned (40% built)". Fix report. Cost small.
19. **7:09, armmex_18847 at D7 (spot_64), 29%, 14 metal. U4.** A lone corgator; the 7:14 report said "D7 (1 enemies, NO turret)", with the frame at 14%. Fix player. Cost small.
20. **22:08, armllt_278 at E5 (spot_49), 17%, 14 metal. U4.** The list began it with corgator 26519 519 away. That corgator had been named at E5 in the 22:07 report, "killing our Sentry". Fix hands-rule. Cost small.

**The other 12 frames, 32 metal together**, all U4 except the last:
- armllt_8038 (8). The pick placed it 250 from his unseen corllt, and it was reported "abandoned (2%)".
- armrl_8479 (6). Its builder was killed by an unseen shot 2 s after the frame began.
- armmex_12149 (12).
- armllt_4289 (3), armllt_7170 (3), armmex_1695 (2), armllt_16178 (1), armmex_23716 (0), armllt_22529 (0) and armmex_10562 (0). The list or the pick started each with a corgator already within about 250 of the builder, or hitting it.
- armmex_14107 (2), also U1. The pick walked the builder away under fire in the second the frame began, and the builder died.
- armrl_11127 (6), U8.

## Orders never started

**frames.py's 48 orders reduce to 19 real ones.** frames.py measures from the command's point, but the host places a build up to 726 away, and the `placed` event records where. Of the 48:
- 19 were started by the same builder at the placed site. Examples: the armavp of 25:25 stood at 27:04; the two armvp of 15:51 stood at 17:02 and 17:12; the armhlt of 30:10 is armhlt_25880.
- 9 were absorbed by another builder's frame at the same site.
- 1 was re-sent and then started.

The 19 real orders by cause: U3 11, U4 6, U1 1, U8 1. Every one cost 50-85 metal (small). The 15 costliest are in the JSON:

- **24:48 to 25:25.** U3, 5 extractors (constructor_10161, 11652, 600, 29791 and 7490). They were 1,650-3,290 from their sites when the player's 25:17 re-lists stopped them, after the note "Disaster: 3 corban + corgol(Tzar) killed 16 Stouts", to build armavp and armhlt.
- **26:13 to 27:15.** U3, constructor_8793 (armmex), constructor_2891 (armrad) and constructor_31542 (armllt). The 27:07 re-lists stopped them on the walk, after "A correap (Tiger) is shooting it now at spot_38".
- **4:06.** U4, constructor_6311's next extractor; the builder died under the corgator at 4:11.
- **20:21.** U4, constructor_19887. It walked 3,300 for 84 s (one move_failed at 20:22 cleared) and was hit 265 from the site. The pick walked it home and it died at 21:48.
- **31:02 and 31:50.** U4, constructor_24774's turret at spot_63. The pick retreated it at 31:39 and 31:53, and it died at 31:59.
- **31:27.** U1, the commander's turret at B8. The player cancelled the list at 31:29 and the pick moved the commander off at 31:53 ("follow group_G7").
- **31:58.** U3, the commander's turret at A7. The player's 31:49 stop list landed at 32:07.
- **32:08.** U8, ordered 13 s before the end.
- **Beyond the cap:** 2 more U3 extractors at D5 (26:41 and 26:51, stopped at 27:34) and 2 more U4 (31:02 and 31:27, whose builders were retreated and died).

## The three mechanisms that would have kept the most metal

1. **A tier-2 defence frame built alone at a metal store of 0, in the path of his tier-2 army (400 metal, armhlt_30842).** It stood 157 s at about 1% every 1-3 s, with spending equal to income and six labs producing, until an unseen Banisher killed it at 91%. At a full build rate it would have stood about 24 s before the first hit (an inference). The levers are spending priority for the defence or a second builder on it.
2. **Forward extractor and turret builds left alone against lone-corgator raids for twenty minutes (12 frames, 317 metal).** The 12 frames: armmex_6304, armmex_18847, armllt_12458, armllt_15814, armllt_1832, armllt_13591, armllt_11366, armmex_12149, armmex_12528 and armllt_25508, plus the corwolv's armllt_13435 and armmex_11072. The raider came during the build at spots 1,400-3,500 from home, with no escort and no finished turret. The report's "turret" word counted the unfinished frame at B4 (9:37) and E7 (12:26). Builds the list or the pick started toward a raider already in sight, or with the builder already under fire, cost another 11 frames but only 57 metal.
3. **A queued help promoted by a neighbour's frame finishing (152 metal, armsolar_28188 at 98%).** The finished-event handler in `pianist/mod.rs` matches by type within 250 of the site, not by the frame's id.

## The report, beside the causes

- **The "abandoned ... its builder was sent elsewhere and the frame decayed" line went out for 6 frames killed by unseen enemies** (489 metal): corwolv, corllt, corgator and corban.
  - `briefing.rs` `abandoned` treats any destruction with no attacker id as an abandonment.
  - The share it quotes is the frame's health after damage or decay: "armhlt (12% built)" for a 91% frame, "armsolar (0% built)" for a 98% one.
  - The same corwolv hit three frames at D5 between 21:15 and 22:06. After the first, the player read "abandoned".
- **The threat line counts an unfinished turret frame as the extractor's turret.** `briefing.rs` filters own units by type only, not by being built.
- **Stale progress words.**
  - At 2:37 the commander's new solar was given the 87% of the old frame 150 away.
  - At 31:59 a Nettle killed at 31:38 still read "(not yet started)".

## Commands run

All from the repository root, with `M=run/matches/1791508617-rove-task-1/00` and the scratch directory `S`. One seat, so no symlink directories were needed.

Repository tools:
```
cat $M/../results.jsonl; python3 -c "import json;print(json.load(open('$M/../batch.json')))"
python3 run/frames.py $M --seat 0            (and --json > $S/frames.json)
python3 run/floor.py $M
python3 run/impedance_sweep.py $M --seat 0 --from 1:40 --to 4:10 --all
python3 run/impedance_sweep.py $M --seat 0 --from 24:40 --to 27:40
python3 run/impedance_sweep.py $M --seat 0 --from 31:20 --to 32:21 --all
python3 run/hands_window.py $M 2:08 2:14 commander
python3 run/hands_window.py $M 17:25 17:29 constructor_7206
python3 run/commander_turns.py $M --calls ; python3 run/commander_turns.py $M --notes
```

Scratch scripts, read-only, run with `python3 -I`, in `$S/bin`:
- `trace.py`: each frame's share second by second, the builder's distance, enemies in sight, damage and events.
- `knew.py`: the report lines on the builder, the enemy-in-sight and threat lines, and the lines after the loss.
- `turns.py` and `think.py`: a turn's prompt, and the player's words.
- `truth_near.py`: the truth file's enemies near a frame at its death.
- `unstarted.py` and `unstarted2.py`: the never-started orders checked against the `placed` site.
- `build_json.py`: writes the JSON.

Other reads:
- inline record queries for the commander at 2:00-3:40, the armhlt_30842 trace, constructor_19887 and the last minute's orders;
- `git show 6ea433b:` of `pianist/mod.rs` (`take_queued`, `promote`, the UnitFinished handler), `brain/briefing.rs` (`abandoned`, `turret_within_300`) and `run/floor.py` `abandoned_builds`.
