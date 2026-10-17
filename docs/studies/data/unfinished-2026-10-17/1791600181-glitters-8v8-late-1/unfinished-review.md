# Unfinished structures: 1791600181-glitters-8v8-late-1

A realtime 8v8 on All That Glitters v2.2.3 (2026-10-09): eight Sonnet 5.5 medium players under an Opus 5.5 high commander against eight BARb hard_aggressive (Cortex); **lost at 28:30** (the game was called; at 28:29 every unit of ours was sent `selfdestruct`, 89 in seat 0). The lost frames were tier-1 turrets and extractors, rebuilt again and again at raided outer spots. Of 116 unfinished frames, 96 (1,455 of 1,927 metal) were lost to the enemy: 93 shot down while being built, 3 left when the enemy killed or chased off the builder. 105 of the 116 were begun by the players' own lists.

## Denominators (frames.py's first line, per seat)

- `1791600181-glitters-8v8-late-1 seat 0, record to 28:30: frames 101 (decayed 5, finished 79, killed 15, unfinished at end 2), metal in unfinished frames 442, build orders never started 20`
- `1791600181-glitters-8v8-late-1 seat 1, record to 28:30: frames 82 (decayed 4, finished 68, killed 10), metal in unfinished frames 153, build orders never started 31`
- `1791600181-glitters-8v8-late-1 seat 2, record to 28:30: frames 104 (decayed 3, finished 96, killed 5), metal in unfinished frames 62, build orders never started 18`
- `1791600181-glitters-8v8-late-1 seat 3, record to 28:30: frames 97 (decayed 8, finished 77, killed 12), metal in unfinished frames 319, build orders never started 36`
- `1791600181-glitters-8v8-late-1 seat 4, record to 28:30: frames 107 (decayed 9, finished 85, killed 13), metal in unfinished frames 356, build orders never started 25`
- `1791600181-glitters-8v8-late-1 seat 5, record to 28:30: frames 98 (decayed 2, finished 85, killed 9, unfinished at end 2), metal in unfinished frames 258, build orders never started 26`
- `1791600181-glitters-8v8-late-1 seat 6, record to 28:30: frames 77 (finished 67, killed 10), metal in unfinished frames 244, build orders never started 18`
- `1791600181-glitters-8v8-late-1 seat 7, record to 28:30: frames 65 (decayed 2, finished 58, killed 4, unfinished at end 1), metal in unfinished frames 93, build orders never started 31`

`run/floor.py` prints `aband 5` for seat 0, which matches frames.py's 5 decayed.

Totals: 731 frames; 615 finished; 33 decayed; 78 killed; 5 unfinished at the end; 1,927 metal in unfinished frames; 205 build orders never started.

Two caveats about these numbers:

- **18 of the 33 'decayed' frames were shot down by an unseen attacker.** The `dmg` rows show 23 to 273 damage in the last two samples, and the share collapses (for example 70% to 35% to 5%). The fate stays 'decayed' so the counts compare with the other reviews; the cause is U4.
- **23 of the 205 never-started orders did start**, at the host's placed site 300 to 712 from the ordered point. frames.py measures from the `cmd` point, not from the `placed` event.

## Tally

| cause | n | metal |
|---|---|---|
| U0 unknown | 0 | 0 |
| U1 the pick sent the builder elsewhere | 1 | 10 |
| U2 the builder's list moved on | 0 | 0 |
| U3 the player re-listed or moved it | 11 | 214 |
| U4 builder died or fled, or the frame was killed | 96 | 1455 |
| U5 economy | 0 | 0 |
| U6 the site | 1 | 21 |
| U7 helpers withdrawn | 0 | 0 |
| U8 game end | 5 | 193 |
| U9 harness fault | 2 | 34 |

U5 appears only as `also`. Six frames, all U4, stood 38 to 81 s at an empty metal store (s0 armllt_4800, s4 armllt_23939, s5 armllt_3139, s6 armllt_23740, armllt_2994, armllt_5433).

| fate | n | metal |
|---|---|---|
| decayed | 33 | 519 |
| killed | 78 | 1215 |
| at_end | 5 | 193 |

| type | n | metal |
|---|---|---|
| armllt | 77 | 1384 |
| armmex | 31 | 325 |
| armsolar | 2 | 149 |
| armrl | 3 | 59 |
| armestor | 1 | 10 |
| armmakr | 2 | 0 |

**What the players knew.**

- **Before:** 63 frames were in a report as standing ("building a ... (N % built)") before they were lost.
- **After only:** 48 were named only after they were gone. That was by the abandoned line, or by the fights line's "lost armllt to corak" and "we lost 1 units worth 85 metal: 1 armllt". Those lines count a 0% frame at its full cost and do not say it was unfinished.
- **Never:** 5 were never named.
- **Dropped by the player:** 11 (the U3 items). Seven were deliberate: six pulled builders away from raids, and one cancelled an extractor that would have blocked the plant's exit lane. Four were stops sent while the report still showed the builder walking; the frames began during the player's turn.

The hands' line "abandoned an unfinished <type> ...: its builder was sent elsewhere" is printed for every frame destroyed by nobody, so it fires for the unseen kills too. Examples:

- s3 9:04: "abandoned an unfinished armllt at E4 (0% built): its builder was sent elsewhere" for a frame shelled by two corlevlr, with its builder standing 155 from it.
- s4 28:10: "(21% built)" for a frame shot down at 49%.

The share it quotes is the share at destruction (1% for frames left at 22 to 30%), not the share when the builder left.

## Items by cost (the twenty costliest; deep-read)

1. **26:30 s3 armsolar_4215 F1 (constructor_12799): U4, killed 27:59 at 90%, 140 of 155 metal**
   - What happened: seat 3's list step 'armsolar spot_8' (the player's 26:20 queue) began it at F1 at 26:30; it reached 90% at 27:59 with the metal store at 0 all the while (89 s), when seat 1's commander (armcom_765, death blast 350/5000) was killed by a corraid beside it at (4026,893): the blast destroyed the solar, its builder constructor_12799, constructors 26614 and 31974 and seat 3's turrets in the same frame.
   - Evidence: 26:24 (list) build a armsolar at spot_8 (F1); 26:30 created; 27:59 f50382 destroyed armsolar_4215 by 765 (by_d -1) with constructor_12799, 31974, 26614 and two armllt the same frame; seat 1 record: 27:59 destroyed armcom 765 by 21996 (by_d 437 = corraid) at (4026,893).
   - The player knew: before: the 26:40, 27:10, 27:30 reports showed it 35%, 67%, 75%; no threat near it in the 26:20 report; the blast came from an ally's commander dying.
   - Fix: none. Cost: large. Confidence: high.
2. **12:18 s6 armllt_5433 G4 (constructor_13963): U4 (also U5), killed 13:21 at 85%, 72 of 85 metal**
   - What happened: a list step (next, queued: armllt spot_57) began it at G4 at 12:18; with the metal store at 0 and income 20 it climbed 1% a second for 63 s; corak and armflash raiders killed its builder at 13:20 and the frame at 13:21 at 85%.
   - Evidence: 12:18 created by constructor_13963 (11:48 list next, queued: build a armllt at spot_57 (G4)); 12:20-13:20 metal store 0, income 20-22, share 2% -> 85%; 13:20 constructor_13963 destroyed by armflash; 13:21 armllt_5433 destroyed by corak at 85%.
   - The player knew: before: the 13:08 report showed 'building a Sentry (armllt) at spot_57 (G4) (69 % built)' with party_304 668 away; the player kept it building and rewrote the defence rule.
   - Fix: player. Cost: small. Confidence: high.
3. **14:18 s0 armllt_4800 A3 (constructor_6720): U4 (also U5), killed 14:56 at 77%, 65 of 85 metal**
   - What happened: a list step (armllt spot_42) began it at A3 at 14:18; the metal store sat at 0 from 14:32 and the frame crawled to 75% by 14:52 with its builder standing 139 from it (the list's next step was queued behind it at 60%, 14:38); corak raiders killed it at 14:56.
   - Evidence: 14:18 created by constructor_6720 (13:52 list build a armllt at spot_42 (A3)); 14:38 (list) next, queued: build a armllt at spot_49 (A3) (queued behind, the builder stayed: 134 from it at 14:57); 14:56 destroyed by corak (273) at 75-77%.
   - The player knew: before: the 14:21 and 14:30 reports showed it 11% and 45%; no enemy near it in the 14:05 report.
   - Fix: none. Cost: small. Confidence: high.
4. **10:19 s0 armllt_14470 B3 (constructor_19580): U4, decayed 11:07 at 71%, 60 of 85 metal**
   - What happened: a list step (armllt spot_45) began it at B3 at 10:19; it reached 71% at 11:04 with its builder beside it; two armflash raiders (party_224, the builder 'alarmed' at 11:06) shot it down 11:05-11:07; the record names no attacker, so frames.py calls it decayed.
   - Evidence: 10:19 created by constructor_19580 (9:27 list build a armllt at spot_45 (B3)); dmg rows on the frame: 11:05 18, 11:06 227, 11:07 193; share 70 -> 35 -> 5 in two seconds; 2 armflash within 800; 11:07 destroyed by=None.
   - The player knew: before: the 10:21, 10:31, 10:49 reports showed it 6%, 25%, 55%; after, the fights line said 'abandoned an unfinished armllt (5% built)'.
   - Fix: none. Cost: small. Confidence: medium.
5. **28:14 s5 armllt_28268 H1 (constructor_21153): U8, at_end 28:30 at 67%, 57 of 85 metal**
   - What happened: a list step began it at H1 at 28:14; 67% when the record ended at 28:30 (the game was called; its builder self-destructed 28:29).
   - Evidence: 28:14 created by constructor_21153 (28:08 list build a armllt at spot_13 (H1)); 28:29 selfdestruct 21153; record to 28:30.
   - The player knew: before: the 28:21 report showed it 26%.
   - Fix: none. Cost: small. Confidence: high.
6. **27:47 s0 armllt_6332 B2 (constructor_2191): U8, at_end 28:30 at 66%, 56 of 85 metal**
   - What happened: a list step began it at B2 at 27:47; 66% when the record ended at 28:30 (the game was called).
   - Evidence: 27:47 created by constructor_2191 (27:38 list build a armllt at spot_19 (B1)); 28:29 selfdestruct 2191; record to 28:30.
   - The player knew: before: the 27:51, 28:11, 28:17 reports showed it 5%, 37%, 44%.
   - Fix: none. Cost: small. Confidence: high.
7. **9:43 s4 armllt_6393 D4 (constructor_30733): U4, killed 10:03 at 63%, 54 of 85 metal**
   - What happened: a queued list step (armllt spot_70) began it at D4 at 9:43; it reached 63% in 17 s with metal in the store; an armflash raider killed it at 10:03 with its builder 147-160 from it.
   - Evidence: 9:43 created by constructor_30733 (8:50 list next, queued: build a armllt at spot_70 (D4)); 10:03 destroyed by armflash (106) at 63%; metal store 219-367 while it stood.
   - The player knew: before: the 9:43, 9:50, 10:02 reports showed it 1%, 24%, 37%; the 9:26 report had 1 enemy at D4 145 from the site.
   - Fix: player. Cost: small. Confidence: high.
8. **11:00 s6 armllt_23740 H5 (constructor_13963): U4 (also U5), killed 11:54 at 64%, 54 of 85 metal**
   - What happened: a list step (armllt spot_75) began it at H5 at 11:00; with the metal store at 0 (income 22 = usage 22) it took 50 s to reach 64%; a corthud killed it at 11:54 with its builder standing by (the list's next step queued behind it at 11:48).
   - Evidence: 11:00 created by constructor_13963 (10:00 list build a armllt at spot_75 (H5)); metal [0, 22, 22] at every sample 11:00-11:50; share 11/22/33/47/64%; 11:48 (list) next, queued: build a armllt at spot_57 (G4) (queued; builder stayed at (5742,5018)).
   - The player knew: before: the 11:03, 11:09, 11:20 reports showed it 3%, 10%, 22%.
   - Fix: none. Cost: small. Confidence: high.
9. **12:11 s6 armllt_2994 G3 (constructor_16136): U4 (also U5), killed 13:32 at 64%, 54 of 85 metal**
   - What happened: a list step (armllt spot_48) began it at G3 at 12:11; at metal 0 it took 77 s to reach 64%; under the corak raid the pick walked its hurt builder home at 13:28 (died 13:30 to corak) and an armflash killed the frame at 13:32.
   - Evidence: 12:11 created by constructor_16136 (11:58 list build a armllt at spot_48 (G3)); metal store 0 12:10-13:28, share 9 -> 64%; 13:28 (plan) walk to home; its list step 'armllt spot_48' waits for it.
   - The player knew: before: the 13:08 report showed 'building a Sentry (armllt) at spot_48 (G3) (46 % built)', builder health 49%.
   - Fix: player. Cost: small. Confidence: high.
10. **8:00 s5 armllt_3139 G4 (constructor_26399): U4 (also U5), killed 8:40 at 61%, 52 of 85 metal**
   - What happened: a list step (armllt spot_72) began it at G4 at 8:00; at metal 0 (income 15 = usage) it took 32 s to reach 49%; a corlevlr killed it at 8:40 with its builder beside it.
   - Evidence: 8:00 created by constructor_26399 (7:46 list build a armllt at spot_72 (G4)); metal store 0-4 every sample 8:00-8:40; 8:40 destroyed by corlevlr (396) at 61%.
   - The player knew: before: the 8:06, 8:16, 8:26 reports showed it 8%, 20%, 36%; the 7:56 report had a corshad 722 from the builder.
   - Fix: none. Cost: small. Confidence: high.
11. **27:56 s0 armllt_27938 A2 (constructor_26016): U8, at_end 28:30 at 58%, 49 of 85 metal**
   - What happened: a list step began it at A2 at 27:56; 58% at the record's end.
   - Evidence: 27:56 created by constructor_26016 (27:38 list armllt spot_23); 28:29 selfdestruct 26016.
   - The player knew: never shown standing (began after the 27:51 report).
   - Fix: none. Cost: small. Confidence: high.
12. **17:27 s3 armllt_19493 F4 (constructor_11799): U4, decayed 18:00 at 57%, 48 of 85 metal**
   - What happened: a list step (armllt spot_62) began it at F4 at 17:27 with a corak 177 from its builder; it reached 57% at 17:59, then took 125 and 243 damage and was destroyed at 18:00 with no attacker named (an unseen kill, frames.py: decayed).
   - Evidence: 17:27 created by constructor_11799 (17:25 list build a armllt at spot_62 (F4)); dmg 18:00 125, 18:01 243; share 57 -> 38 -> gone; 18:00 destroyed by=None.
   - The player knew: before: the 17:29, 17:37, 17:47 reports showed it 5%, 19%, 39%; after, the fights line said 'abandoned an unfinished armllt (38% built)'.
   - Fix: hands-rule. Cost: small. Confidence: medium.
13. **25:27 s5 armllt_31097 H2 (constructor_21153): U3 (also U4), decayed 26:35 at 56%, 48 of 85 metal**
   - What happened: the pick's Sentry at spot_22 (H2), begun 25:27; at 25:57 the player, with his tier-2 block (Stouts, Brutes, two Tigers, a Sheldon) inside the base, wrote 'Constructors (constructor_17391, constructor_21153, ...): walk to north_fall at once and stay there'; the pick played it at 26:05 at 56% and the frame decayed by 26:35.
   - Evidence: 25:57 instruct: 'The commander and constructors leave now ... Constructors (... constructor_21153 ...): walk to north_fall at once and stay there until the party is gone.'; 26:05 PICK constructor_21153 w2 0.94: 'walks to north_fall ... stepping away from party_850 (1 armsam, 6 armstump, 1 cormort, 3 corraid, 2 correap, 1 corthud) ... leaves the Sentry (armllt) at about 60%, which decays'; 26:30-26:35 share 13 -> 0 (decay), 26:35 abandoned an unfinished armllt at H2 (1% built).
   - The player knew: before: the 25:57 report showed 'building a Sentry (armllt) at spot_22 (H2) (42 % built)'; it meant to leave ('The commander and constructors leave now').
   - Fix: player. Cost: small. Confidence: high.
14. **17:48 s1 armllt_13304 F3 (constructor_18934): U3 (also U4), killed 18:15 at 53%, 45 of 85 metal**
   - What happened: a list step began it at spot_51 (F3) at 17:48; at 18:05, with party_491 (1 corgator, 4 corlevlr, 1 corraid) 535 away, the player queued ['stop', 'armllt spot_8'] and sent the builder 2,826 away to F1; the frame was destroyed at 18:15 at 53% by an unseen attacker.
   - Evidence: 18:05 queue constructor_18934: ['stop', 'armllt spot_8']; RESULT its build in progress dropped; 18:08 stopped what it was doing on your `stop`; 18:09 build a armllt at spot_8 (F1) (from its list); 18:15 destroyed by an unnamed enemy (by_d -1).
   - The player knew: before: the 18:05 report showed 'building a Sentry (armllt) at spot_51 (F3) (45 % built)' with the party 535 away; it dropped it on purpose.
   - Fix: player. Cost: small. Confidence: high.
15. **11:37 s3 armmex_11168 F4 (constructor_15992): U4, killed 12:08 at 84%, 42 of 50 metal**
   - What happened: a list step (extractor spot_71) began it at F4 at 11:37; at metal 0 it reached 84% at 12:05 with its builder 110 from it; an armstump (one of his resurrected Stouts) killed it at 12:08.
   - Evidence: 11:37 created by constructor_15992 (10:44 list build a armmex at spot_71 (F4)); metal store 0 every sample; share 7/21/38/64/53%; 12:08 destroyed by armstump (217).
   - The player knew: before: the 11:44, 11:55, 12:04 reports showed it 12%, 40%, 80%.
   - Fix: none. Cost: small. Confidence: high.
16. **27:52 s4 armllt_30053 D1 (constructor_30733): U4, decayed 28:10 at 49%, 42 of 85 metal**
   - What happened: a list step (armllt spot_4) began it at D1 at 27:52 with 23-34 enemies within 800; it reached 49% at 28:08, took 99 and 273 damage and was destroyed at 28:10 with no attacker named (unseen kill, frames.py: decayed).
   - Evidence: 27:52 created by constructor_30733 (27:46 list build a armllt at spot_4 (D1)); dmg 28:09 99, 28:10 273; share 49 -> 37 -> gone; 28:10 abandoned an unfinished armllt at D1 (21% built): its builder was sent elsewhere (it was not: 133 from the frame).
   - The player knew: before: the 27:53, 28:03, 28:09 reports showed it 29%, 37%, 44%; after, the abandoned line, which blamed the builder.
   - Fix: none. Cost: small. Confidence: medium.
17. **7:53 s4 armllt_27037 D4 (constructor_28831): U4, killed 8:12 at 47%, 40 of 85 metal**
   - What happened: a list step (armllt spot_63) began it at D4 at 7:53, two corshad and raiders near; it reached 47% at 8:08; a corlevlr killed it at 8:12 with the builder 56 from it.
   - Evidence: 7:53 stop 28831 + (list) build a armllt at spot_63 (D4); created the same tick; 8:12 destroyed by corlevlr (396) at 28-47%.
   - The player knew: before: the 7:59 and 8:10 reports showed it 17% and 25%; the 7:49 report had 2 enemies at D4 387 away.
   - Fix: hands-rule. Cost: small. Confidence: high.
18. **26:46 s1 armllt_15643 F1 (constructor_26324): U4, killed 27:00 at 44%, 37 of 85 metal**
   - What happened: the pick's Sentry at spot_20 (F1), begun 26:46 with his army 790 away at F2; it reached 40% at 26:57 and an armstump killed it at 27:00 with the builder beside it (the pick walked the builder home at 27:01).
   - Evidence: 26:46 created by constructor_26324 (26:31 plan build a armllt at spot_20 (F1)); 27:00 destroyed by armstump (217) at 35-44%; 27:01 (plan) walk to home.
   - The player knew: before: the 26:56 report showed it 34%; the 26:42 report had 22 enemies at F2 790 away.
   - Fix: player. Cost: small. Confidence: high.
19. **6:27 s6 armmex_11116 H4 (constructor_23993): U4, killed 6:47 at 71%, 36 of 50 metal**
   - What happened: a queued list step (extractor spot_75) began it at H4 at 6:27; it reached 71% at 6:42 (the list's next step queued behind it at 6:40, the builder stayed 138 from it); corak raiders shot it from 71% to 19% in four seconds and killed it at 6:47.
   - Evidence: 6:27 created by constructor_23993 (6:08 list next, queued: build a armmex at spot_75 (H4)); 6:40 (list) next, queued: build a armllt at spot_75 (H5); share 71 -> 46 -> 19, 6:47 destroyed by corak (273).
   - The player knew: before: the 6:33 and 6:38 reports showed it 27% and 57%; no enemy near it in the 6:22 report.
   - Fix: none. Cost: small. Confidence: high.
20. **26:02 s4 armrl_27967 D2 (constructor_9780): U9 (also U1), decayed 26:54 at 43%, 34 of 80 metal**
   - What happened: the list's Nettle (armrl spot_21) at 3048,1496, begun 26:02, reached 43% at 26:32; the pick walked its 7%-health builder to spot_9 at 26:33 'stepping away from party_929 (1 corhurc)'; six seconds later the list played the waiting step again, 'build a armrl at spot_21', and the engine placed a second Nettle at 2968,1528, 86 from the first, which decayed by 26:54 (the second was killed by an armstump at 27:07, 30%).
   - Evidence: 26:33 PICK constructor_9780 w2 0.91: 'walks to spot_9 ... stepping away from party_929 (1 corhurc) ... leaves the Nettle (armrl) at about 40%, which decays'; 26:39 (list) build a armrl at spot_21 (D2); cmd build 9780 188 2792 1448; placed 2968,1528; 26:48 created armrl_21963 at 2968,1528 while armrl_27967 stood at 3048,1496 (43%).
   - The player knew: before: the 26:07, 26:13, 26:19 reports showed it 7%, 15%, 23%; after, the abandoned line at 0%.
   - Fix: hands-rule. Cost: small. Confidence: high.

The other 96 frames are in `unfinished.json` with the same fields, classified from frames.py's entries, the decision rows, the reports and the `dmg` rows. 82 of them are U4, most of them list steps killed at 0 to 48% at raided spots. The others are nine U3, one U1, one U6, one U9 and two U8.

## Orders never started (15 of 205 by cost; the full set in the JSON)

- 5:28 s2 constructor_7194 armvp E2: **U3**. The old list's second vehicle plant (queued behind the nano at 60%, 5:28) was dropped by the player's 5:28 list of four solars: 'Energy has crashed to 237 of 1500 ... I dropped the second vehicle plant for now'; the hands stopped the queued plant when the nano finished at 5:55.
- 25:10 s3 commander armlab F1: **U3**. The player's 25:06 list 'armlab north_hide #lab1' (commander and constructor_10444) was replaced three seconds after it landed by 'armvp north_hide #vp1' on four builders (25:13); the vehicle plant armvp_19781 stood at G1 at 25:37.
- 25:10 s3 constructor_10444 armlab F1: **U3**. The same #lab1 replaced by #vp1 at 25:13; this builder then started the plant (armvp_19781, 25:23, at the host's site 462 from north_hide).
- 2:31 s1 constructor_4720 armsolar F1: **U2**. The pick's solar at spot_20 (placed 428 away, not begun) met the player's 2:32 list ['armsolar', 'armsolar', 'extractor spot_18', ...]: both solar steps were 'taken as done' against the unstarted order, then the list's Sentry step replaced it; no solar was built (the 2026-10-16 rule 'a list re-sent with the build the builder was ordered and has not begun ... waits for the frame' now covers this).
- 5:42 s2 constructor_24252 armsolar D2: **U3**. Its list's solar 'at home' was walking (14 s) when the player's 5:51 list moved all four solars to spot_50.
- 26:06 s3 constructor_12799 armsolar F2: **U3** (also U9). The player's 26:00 list ['armsolar'] carried no place and the hands sited it at army_meet (F2), where the same turn's packet said his block of 45+ stood; the same second the pick ordered the builder an armllt; the player re-listed ['stop', 'armsolar spot_8'] at 26:20 (that solar is armsolar_4215, killed at 90% by the commander's blast).
- 19:33 s5 commander armsolar H2: **U3**. The pick's solar at spot_37 (19:33) was replaced four seconds later by the player's 19:32 list landing ('armllt spot_36' first); the player never saw the pick's order.
- 26:06 s3 constructor_12799 armllt F2: **U9**. Two orders to one builder in the same second: the pick's Sentry at 336 from spot_40 and the list's solar; the list's build command went second and replaced the pick's.
- 6:06 s7 constructor_29053 armllt B2: **U6**. Stuck in plant_4020's exit lane: move_failed 6:08, 6:09, 6:22, 6:26, 6:45, 6:46; the player had the commander take it apart at 7:04.
- 6:30 s7 constructor_6806 armllt B2: **U6**. The second stuck constructor in the same lane: move_failed 6:31, 6:31, 6:32; the watchdog refused the site at 7:12.

The other five in the JSON:

- one U9: s3 26:06, a pick and a list order in the same second
- three U3: s1 14:02 x3 stops after allied-spot skips; s0 22:15; s5 26:41
- one U3 over allied-spot skips: s4 5:19

Plant orders frames.py lists that are **not items**, because the plant was built by the same builder at the host's placed site:

- s3 25:17 armvp, 462 away
- s4 9:47 armvp, 712 away
- s5 0:41 armvp, 323 away
- s5 21:43 armvp, 475 away
- s6 5:59 armvp, 348 away
- s6 27:42 armvp, 430 away

Two nano orders in s6 are not items either. The 8:37 one was queued behind the advanced plant and started at 10:13. The 9:53 one was re-sent to another site at 9:59 and started at 10:11.

## The three mechanisms

### List steps played into a visible raid

List steps played into a party the picture already showed: 66 of the 96 U4 frames (674 metal) began with an enemy within 900 of the site in the last report or in the builder's own entry (47 frames, 299 metal, had the party in the builder's enemies_near); 105 of all 116 frames came from the players' lists, rebuilt again and again at the same raided outer spots (spot_75 H4/H5 9 frames across seats 5 and 6, spot_42/spot_49 A3 8 in seat 0, spot_66/spot_73 E4 7 in seat 1, spot_45 B3 7 in seats 0 and 7, spot_21 D2 4 in seat 4); a hold on a list step whose builder or site has an enemy party within 800, asked of Jev rather than gated in code (the 2026-10-01 code gate of this kind was removed on the user's word), would have held these steps until the party left: 674 metal at most, an upper bound, since some raids would have reached the frames anyway.

Items: s4:armllt_6393, s5:armllt_3139, s3:armllt_19493, s3:armmex_11168, s4:armllt_27037, s1:armllt_15643, s5:armmex_10967, s0:armllt_20079, s5:armllt_16285, s4:armrl_21963, s2:armmex_14044, s0:armllt_9626, s1:armllt_16747, s1:armmex_21784, s0:armllt_10356, s4:armllt_19142, s4:armmex_9981, s1:armmex_20813, s2:armllt_16388, s3:armllt_22422, s5:armsolar_15044, s5:armllt_18958, s2:armmex_30896, s3:armllt_29714, s3:armmex_9185, s4:armllt_21804, s3:armllt_3635, s4:armmex_15147, s4:armllt_5104, s1:armllt_5820, s3:armllt_3112, s2:armmex_25566, s2:armllt_28139, s3:armmex_19070, s5:armllt_4481, s6:armllt_23462, s2:armllt_6950, s3:armllt_4149, s4:armllt_6691, s7:armllt_29881, s1:armmex_19423, s1:armllt_405, s3:armmex_15283, s6:armmex_22271, s0:armmex_18361, s3:armllt_28775, s4:armllt_23570, s4:armllt_15609, s4:armrl_13177, s0:armllt_13078, s0:armllt_21004, s0:armllt_1032, s0:armmex_19086, s0:armllt_24446, s1:armmex_31285, s1:armmex_22757, s1:armllt_13250, s1:armllt_20875, s2:armmex_6122, s3:armllt_28525, s3:armmakr_23286, s3:armllt_22994, s3:armllt_7502, s5:armllt_3626, s6:armllt_17105, s6:armmex_11060

### Slow builds at an empty store

Slow builds at an empty metal store on contested spots: 12 frames (652 metal) stood 30-89 s with the metal store at 0 (income equal to use) and were lost at 22-90%, most of them the costliest items (s6 armllt_5433 85%, armllt_23740 64%, armllt_2994 64%, s0 armllt_4800 77%, armllt_14470 71%, s3 armsolar_4215 90%, armmex_11168 84%, s5 armllt_3139 61%); at an empty store a seat's frames share its income, so several lists each starting a frame make every frame stand longer under the raids; putting the seat's builders on one frame at a time (help the standing frame) while the store is empty would have shortened each frame's exposure (an inference, not measured here).

Items: s0:armllt_14470, s0:armllt_4800, s3:armmex_11168, s3:armllt_19493, s3:armsolar_4215, s4:armllt_25730, s5:armllt_3139, s6:armllt_2573, s6:armllt_23740, s6:armllt_2994, s6:armllt_5433, s7:armmex_15128

### A stop landing on a frame begun during the turn

The player's stop that lands on a frame begun during its turn: in a realtime game the orders land 6-10 s after the report, and four frames that began in that gap were dropped by stops meant for walks (s0 armllt_22338, armllt_8646, armllt_10868, s1 armmex_8758: 54 metal); the queue tool could answer what the stop now drops ('its Sentry began 4 s ago, 22%') or keep a frame that began after the report; the other seven U3 drops (160 metal) were deliberate: six pulled builders away from raids, one (s3 armmex_9173) cancelled an extractor that would have blocked the plant's exit lane.

## Commands run (from the repository root; scratch = the scratch directory in the prompt)

```
cat run/matches/1791600181-glitters-8v8-late-1/results.jsonl; python3 -c '...json.load(open(.../batch.json))'; grep -n 'name=\|Side' MATCH/script.txt
run/frames.py MATCH --seat N  and  --json  (N = 0..7)
run/floor.py MATCH
run/impedance_sweep.py MATCH --seat N --from m:ss --to m:ss --all  (s0 22:30-23:05, 23:45-25:25, 22:05-22:33; s1 17:10-18:20, 13:55-14:10, 2:20-2:40; s2 5:10-6:00, 5:44-5:57; s3 8:05-9:10, 12:35-13:10, 25:00-25:40, 26:00-26:25; s4 0:00-2:00, 15:35-15:50, 20:20-20:32, 5:00-5:40; s5 12:40-13:06, 19:25-19:40, 26:35-27:25; s7 5:45-7:40)
run/hands_window.py MATCH 11:00 11:08 constructor_19580; run/hands_window.py scratch/seatN ... (s1 27:44-27:56 constructor_26324; s4 1:36-1:42 commander, 26:30-26:50 constructor_9780; s5 25:58-26:08 constructor_21153; s7 6:28-6:50 commander_t7)
scratch/scripts/detail.py (share series, dmg rows, enemies within 800, the destroyed event per frame), knew.py (the reports' building/% lines and threats), origin.py (the decision row that began each frame), told_after (the next reports' loss lines), trace.py (frames, builders, cmd, ev, d rows in a window), menu.py (a builder's menu, world lines and the pick from jev-N.jsonl.zst via run/jevlog.py), turn.py (a turn's report lines, calls and text), placed_check.py (never-started orders begun at the placed site)
symlink directories for seats 1-7: scratch/seatN/{record,jev,strategist,terrain,truth}-0 -> MATCH/...-N
```

Nothing was unread. Seat-0-only tools (hands_window, and my menu reader over the hands' log) ran on symlink directories for seats 1-7 under the scratch directory. frames.py's never-started test measures from the build command's point, not the host's placed site: 23 of the 205 orders it lists began at the placed site 300-712 away (8 plant orders among them: s3 25:17, s4 9:47, s5 0:41 and 21:43, s6 5:59 and 27:42), so they are not items. frames.py's 'decayed' (destroyed by nobody) includes 18 frames the dmg rows show were shot down by an unseen attacker (23-273 damage in the last two samples, share falling fast); they are tallied by cause U4 and kept under fate 'decayed' for comparability. The 25551 kill (s2) had no enemy in sight within 800: low confidence. The never-started list is 15 of 205 by cost; the rest were classified only in aggregate.
