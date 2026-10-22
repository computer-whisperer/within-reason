# Part c: `docs/briefs/experience/{intro,winning,cases,late-game}.md` (reader C)

Rows: `part-c.jsonl`, 187 rows over the four part files as they stand in the working tree at d44e318 (intro 26 lines,
winning 77, cases 431, late-game 190; every non-blank line is in some row's range). Quotes were checked against the
files (whitespace and bold markers normalised); every line parses and carries the README's keys. Game facts come from
the text, `docs/experiments.md` (dates from the match directories' epochs) and `docs/knowledge/`; the glossary checks
are against `docs/glossary/units.csv`.

## Counts

By layer (what the passage is now): episode 93, meta 26, fact 22, derived 16, tool 14, mixed 10, ruling 6.

By fate (what it would become): episode 89, meta 28, rederive 27, fact 24, tool 12, discard 7.

| file | rows | fact | meta | rederive | episode | tool | discard |
|---|---|---|---|---|---|---|---|
| intro.md | 8 | 0 | 1 | 1 | 0 | 5 | 1 |
| winning.md | 21 | 1 | 3 | 12 | 3 | 0 | 2 |
| cases.md | 123 | 12 | 19 | 8 | 77 | 4 | 3 |
| late-game.md | 35 | 11 | 5 | 6 | 9 | 3 | 1 |

Doctrine flags: rule-from-one-case 12, rule-from-ruling 7, rule-from-pool 5. Of the 27 rederive rows, 20 look derivable
outright and 7 partly (a derivable core with a situational remainder). 23 rows carry a conflict, 82 a duplicate, 53 a
glossary or knowledge check, 7 a mismatch.

The shape of these files: cases.md is four fifths episodes (77 of 123 rows), nearly all from 2026-09-22 to 10-04 under
the pre-rebuild or early-layer hands; winning.md is the one arguing part and is almost entirely rederivable; late-game.md
is the best-grounded file (eleven fact rows, every quoted cost reproducing from units.csv) with its episodes drawn from
two games of 2026-10-09 and the Altored/Center Command games of 10-18..20.

## The derivation backlog (subjects, with the lessons each must reproduce or explain away)

1. **Economy is a means; only the army wins** (winning 7-13). Premises: the game-end rule (see conflict 1), what deals
   damage. Must reproduce: a larger economy that never attacks has not won (player-50, player-54-experience).
2. **Idle army metal does no work; value is threatened or dealt damage** (winning 15-18). Partly: the pool's "a third to a
   half forward" is the population's habit, not a consequence. Must reproduce: winners' 376 of 1,070 forward at 6:00.
3. **Scale every number by the map** (intro 13-17): map distances, pass widths, spot values, unit speeds. Must reproduce
   the Rover crossing example (18 s on Comet, 10 s on Great Divide: unchecked).
4. **The tier-2 sprint and the covered gap** (winning 25-39; cases 117-131, 133-145, 175-179 part 2). Premises: plant
   2,600 / 14-16k energy / 27,000 build time, advanced constructor 430-580, moho 620 with ~104 s payback, the seat's
   income, his distance and army, allies (meta), the pool's 4 of 58. Must reproduce: the gap is lethal in a tight duel
   (player-10/11, the distilled bot's 1 of 16 at income 22) and covered in a team game (human-8v8-1's Apex seat at 5:20);
   the decision procedure "long, covered, none: now" is doctrine from the ruling and should come out as a consequence
   or be dropped.
5. **Feeding is production, not surplus** (winning 30-32; cases 136-145). Premises: income is continuous; a store fills
   only when orders stop; transfer moves the store. Must reproduce: 450 metal fed in nineteen minutes against three asks
   (Center Command).
6. **Frame time is build time over power when the store pays** (cases 160-162; late-game 21-24, 84-88). Must reconcile:
   "about 70 s a moho alone" (arithmetic) against "90 to 135 s" (measured on an empty store), and player-32's fusions at
   274-276 s against 186.
7. **Metal needs build power on the frame** (cases 169-171, the user 2026-09-28). Follows from 6.
8. **A lead is spent or teched, never banked** (winning 41-47; late-game 89-91). Premises: stored metal has no effect;
   build power is the conversion rate; his growth is independent of our store. Must reproduce human-18's 6,200 bank and
   player-50's; must *not* reproduce "his tier 2 comes on its clock" as a general law (BARb-specific).
9. **Early pressure as information and cost** (winning 49-51; cases 15-38). Premises: sight rules; a constructor's value
   is its remaining output; his greedy opening is unguarded (meta). Must reproduce the pool's 2:06 and player-40..45's
   late first kills.
10. **Targets: economy and production, not turret posts; go round on open ground** (winning 53-58). Premises: the win
    condition; a turret's reach against the map's width. Must reproduce player-54-experience's five minutes on a post.
11. **Concentration; what a turret is worth** (winning 60-62; cases 339-353). Partly: "three times its metal" is the
    hands' odds constant (player.md:381), not a game fact. Must reproduce escalate-1 (piecemeal, then 57 together).
12. **Scout before committing** (winning 64-65; cases 372-388). Premises: sight and radar rules; scout costs (Rover 31,
    Tick 21). Must reproduce comet-1 (fourteen Blitzes into an unseen D-gun).
13. **Cheap static answers to raids let the army leave** (winning 67-70; cases 256-278). Partly: the pool's 1.85 a
    minute is meta; the derivable core is extractor 50 against Sentry 85 against the army's walk home. Must reproduce
    escalate-4 (33 extractors to single Ticks) and escalate-7 (nine lost with pickets and turrets).
14. **Spend** (winning 72-73; cases 280-297). Premises: build power as the rate limit; storage caps.
15. **What ends a game** (winning 75-77): becomes a fact once conflict 1 is settled.
16. **Where the body stood separates wins from losses with a lead** (cases 92-94). Partly: the "within two minutes" is
    situational. Must reproduce player-9/10 (lead at home, lost) against escalate-7/player-52.
17. **Answers that take minutes start on earlier signs** (cases 368-370). Premises: build times of the answer and of
    his threat; sight rules; chat. Must reproduce human-23 (anti-air ordered at the first attack).
18. **The water-map opening names the under-water spots** (cases 427-431). Partly: the shipyard-first and torpedo
    procedure is one case each. Must reproduce the two Cape Violet losses and the Azurite seat.
19. **The order of the late economy** (late-game 52-77 part 2). Premises: metal per +1 a second by source, payback, the
    advanced fusion's energy bill and death blast, converters above 75% storage. Must reproduce the user's recollection
    (fusions as the transition, then heavy advanced converters) and the +50 Altored seat's 2,000 e/s unused.
20. **The nuke window by opponent** (late-game 154-158). Premises: silo build time, the 120 s round, the interceptor's
    90 s and 2,000 coverage, the opponent's anti-nuke clock (meta). Must reproduce nuke-2-hard (two rounds ready a minute
    before his anti-nuke was begun, none fired).
21. **Where a missile goes after his anti-nuke; the army ends the game** (late-game 166-168). Partly: one game.
22. **The anti-nuke's timing is the opponent's habit or the price of insurance** (late-game 172-188). Premises: sight
    rules, the missile needs no sight, 60,000 build time and 90 s interceptor, 1,500 = 2.4 mohos, the opponent's silo
    habit (meta). Must reproduce apex60-1 (first silo seen 33:19, its missile landed 33:12) and Center Command (ours at
    12-16 min, nothing flew).

## The meta catalogue

| passage | observation | population | map | date |
|---|---|---|---|---|
| intro 18-21, late 1-7 | duels end 12-15 min median; tier 2 in 4 of 58 sides (late-game says 4 of 80) | OS 40+ pool, 29/40 games | Comet Catcher | 2026-09-23..10-03 |
| winning 15-18 (2) | winners' army share in his half 376/1,070 at 6:00, 1,540/4,444 at 12:00; losers a fifth or less | 41 Armada sides of the pool | Comet | 2026-10-03 |
| winning 20-23, cases 63-65 | the game ends 3.4 min (median) after one side first holds twice the army, in 27 of 40 games | pool | Comet | 2026-09-27 |
| winning 21-23 (2), cases 175-179 | BARb's advanced factory at 10:41 median in 73 of 74 games; army flat 12:00-16:00, doubling every five minutes from 20:00 | BARb, profile unstated (hard_aggressive by era); source claim not found | Comet | 2026-10 |
| winning 27, cases 125 | tech in 4 of 58 sides | pool, 29 games | Comet | 2026-09-27 |
| cases 15-20 | first soldier in his half 2:06 (Armada 1:40, Cortex 2:48); first building killed 3:49; winners 40 s earlier; 1.85 extractors a minute lost by winners | 80 sides | Comet | 2026-10-03 |
| cases 22-23 | what the first soldiers meet at 2:00-4:00 | unstated | Comet | unknown |
| cases 36-38, winning 49-51 | "the first handful runs to the other side of the map" | experienced players via the user | general | 2026-10-03 |
| cases 96-97, 252-254, 297, 322, 352-353, late 15-16 | thebluegecko's and Irishstud14's words | two or three people spectating/playing the 3v1s | Comet, SailAway | 2026-09-27/28 |
| cases 99-103 | largest body of one type: a dozen raiders or line tanks, eight Stouts | pool | Comet | 2026-09-28 |
| cases 150-160 | one advanced lab serves the side; constructors passed across factions | thebluegecko and u6bkep, one game | Center Command | 2026-10-19 |
| cases 162-169 | the tier-1 lab eaten the second the advanced lab is placed; each plant eaten for the next | thebluegecko, one game | Center Command | 2026-10-19 |
| cases 224-225 | "not a unit choice problem, a not enough mex problem" | the opponent of human-10 | Gecko Isle | 2026-10-03 |
| cases 231-232, late 76-77 | the user on mohos, advanced solars, the fusion-to-converter order | the user | general | 2026-10-02/03, 10-18 |
| cases 234-237 | Armada armies 84-88% Blitzes to 10:00; Cortex 67-76% Incisors | pool | Comet | 2026-09-27 |
| cases 240-242 | Pounders beat light tanks; a Lasher mass lets him tech | three people | Comet | 2026-09-27/28 |
| cases 256-259 | losers lose extractors 2.52 a minute, winners 0.99; 7/10/13 light turrets by 8:00 | pool, Armada | Comet | 2026-09-27 |
| cases 280-284 | constructors 3/5/8 standing at 5/8/12; store 260/837/410; three nanos by 6:37 in one replay | pool, Armada | Comet | 2026-09-27 |
| cases 299-302 | the opening mex, mex, solar, solar, plant at 0:34 in 22 of 41 sides | pool, Armada | Comet | 2026-09-27 |
| cases 336-337 | "the players expect a camp at the pass" | unnamed | Great Divide | unknown |
| cases 377-378 | BARb's base in the north from the corner start | BARb hard | Comet | 2026-09-23 |
| cases 388 | radars by 8:00 1/2/2 | pool, Armada | Comet | 2026-09-27 |
| late 33-35 | "on Comet the experienced players build no converters" (the claim says 10 of 80 did) | pool | Comet | 2026-10-19 |
| late 151-154 | BARb hard_aggressive's anti-nuke at 21-25 in 27 of 33 long games; its silo in 14 of 848 | BARb, truth files | Comet | 2026-10-09 |
| late 178-182 | people build a silo very late or never | thebluegecko, u6bkep, computer_whisperer | Center Command, Cape Violet | 2026-10-18..20 |

## The episode index

Dates are the match directories' epochs (UTC); "pre-rebuild" means before the hands rebuild of 2026-10-02 and carries the
stale under-fire record, the list bugs, the escort and waiting-list steps (fixed 2026-10-01) and every later fix;
"Oct-3" means the rebuilt hands before rove-as-task (10-07/08), the exit-lane and priority fixes (10-13), the impedance
fixes (10-16/17), the chase/frames/spending fixes (10-17) and the standing transfer (10-19/20).

| game | date | opponent / era | passages | defects the lesson rests on | lens |
|---|---|---|---|---|---|
| human-3, -5, -8 | 2026-09-22 | people, Quicksilver, pre-rebuild | cases 309-313, 247-248, 396 | the earliest hands; the produce cap (human-8) | ledger |
| comet-1 | 2026-09-22 | BARb easy, pre-rebuild, player at low effort | cases 375-376 | pre-rebuild | the user |
| comet-3, -4, -5 | 2026-09-22 | BARb medium, pre-rebuild, low effort | cases 60-61, 243-245, 273, 291, 314-315, 327, 393-394 | pre-rebuild | ledger |
| pianist-player-2, -3 | ~2026-09-21 | BARb easy (the brief says medium for -3), pre-rebuild | cases 77-81 | the control lane's hold at an unpriced turret's reach (fixed after game 2) | ledger |
| pianist-player-6 | ~2026-09-21 | BARb medium, Quicksilver | cases 270-272 | pre-rebuild; the ledger says no Pawns were built | ledger |
| escalate-1 (easy), -3 (hard), -4..-7 (hard_aggressive) | 2026-09-23 | BARb, pre-rebuild | cases 74-76, 264-265, 274-275, 287-293, 334-335, 342-343, 377-378, 397-398 | pre-rebuild; escalate-4's list step held the commander; the two hands fixes before escalate-7 | ledger; the user for escalate-4 |
| bluegecko-3v1-1..-5, -9, -10, -12 | 2026-09-27/28 | people (thebluegecko OS 48; game 4 Irishstud14; game 5 Infern8), Comet, pre-rebuild | cases 111, 194-205, 240-242, 246, 266-269, 331-332, 344-347, 382, 395 | pre-rebuild; game 9's "go home" pick (fixed 9b4ad85); game 10's lobby boxes not honoured | chat and ledger |
| bluegecko-3v1-11 (SailAway 2) | 2026-09-27 | people, water, pre-rebuild | cases 249-250 | pre-rebuild | model review (root 4:23) + chat |
| bluegecko-3v1-cape-violet | 2026-09-27 | people, Cape Violet, pre-rebuild | cases 333, 423-425 | pre-rebuild | ledger |
| player-8, -9, -10, -11, -12 | 2026-09-27 | BARb hard_aggressive, Comet, pre-rebuild | cases 68-73, 182-183, 196-198, 316-320, 328-330, 348-349, 379-381; late 84-88 | pre-rebuild; player-9's scouts turned home and sent hunting by the hands | ledger; the user for player-9 |
| player-15, -16, -18 | 2026-09-28 | BARb hard_aggressive, pre-rebuild | cases 82-86, 110; late 81-83 | pre-rebuild; the march (fixed 10-03) | ledger |
| player-30, -31, -32 | 2026-10-01 | BARb hard_aggressive, pre-rebuild | late 81-94 | pre-rebuild; the brief of the time argued against tier 2 | ledger |
| player-40..-45 | 2026-10-02/03 | BARb hard_aggressive, the layer series | cases 26-28, 223; late 81-83 | -40 a frozen pre-layer bot; -41..-44 before the human-11 fixes; Oct-3 | ledger; the user called -44, -45 |
| player-49, -50, -52, -54 pair | 2026-10-03 | BARb hard_aggressive | winning 9-13, 45-47, 56-58; cases 33-34, 56-59, 87-88, 226-228, 278; late 89-91 | Oct-3 | the user watched player-50 and reviewed its replay; player-54's transcript "not read yet" (ledger) |
| sonnet-low-1, sonnet-medium-1 | 2026-10-03 | BARb hard_aggressive, Sonnet 5.5 commander | cases 29-32, 89-90, 112-113, 276-277, 294-295, 350, 385-386 | Oct-3; the model is a confound | ledger |
| human-10 | 2026-10-03 | people, Gecko Isle | cases 224-225 | Oct-3 | chat |
| human-18 | 2026-10-03 | thebluegecko, All That Glitters, three seats under one player | winning 45-46; late 89-90 | one player over three seats fell behind the game | the player's own report line |
| human-23 (two games) | 2026-10-04 | a person, Comet; Sonnet player under an Opus commander | cases 358-366, 383-384 | wake lines under-counted losses; the commander seat not woken; scout asks dropped by the packet | model review (candidates) + the user's reading |
| going-1 | ~2026-10-04 | BARb hard_aggressive (the standing duel) | cases 362-364 | the unbuilt turrets traced to the player's lists, "not a harness fault" | ledger |
| distilled-t2b-*, -t2c-divide-* | 2026-10-09 | BARb hard_aggressive (Comet) and medium (Great Divide), bots/distilled | cases 183-186; late 95-110 | the bot's own rules | ledger (8 games a row) |
| nuke-1-medium | 2026-10-09 | BARb medium, Comet, the nuke objective | late 111-122, 159-161 | the picture kept a dead lab; no missiles line in the report | ledger + the player's notes |
| nuke-2-hard | 2026-10-09 | BARb hard_aggressive, the nuke objective | late 123-136, 161-168 | no report line named the rounds 18:28-23:39 (fixed that evening) | ledger + the player's notes |
| human-8v8-1 (a 5v5) | 2026-10-18 | people + Apex v0.3.0 allies, Altored, +50% | cases 187-193, 202-203 | before the standing transfer; the commander named no sprint | ledger (record census) |
| Azurite Shores with people | 2026-10-18 | people | cases 414-416 | before the standing transfer and the roster fix | ledger (economy.md:452) |
| Cape Violet 1v1 at +100% | 2026-10-18 | computer_whisperer | cases 417-422; late 179-180 | the lone constructor's 35 s of flips | ledger |
| apex40-1, apex50-1, apex60-1/-2 | 2026-10-18/19 | Apex v0.3.2, Altored 3v3 | winning 33-37; cases 156-160; late 71-77, 173-177, 181 | apex60-1 ran on the regressed brief (4049e21); the roster-by-builder fix; the sight arc | ledger + commit 4b3980c |
| human (Center Command 2v4) | 2026-10-19 | thebluegecko and u6bkep | winning 32; cases 137-173; late 178-184 | no standing transfer; the tagged plant step fired twice; cross-faction constructor words | the engine replay read by the main session; the user's rulings |

## Numbers checked against `docs/glossary/units.csv`

Matches (one lookup each): armavp 2,600 / 14,000 and coravp 16,000; armmoho 620 / 7,700 / 20 upkeep / 14,900 build
time; cormoho 640; armfus 3,350 / 750 e/s / 54,000 and corfus 3,600 / 850; armafus 9,700 / 69,000 / 312,500 / 3,000 /
9,000 store and corafus 48,000; armsilo 8,100 / 90,000 / 178,500 and corsilo 7,700 / 82,000; armadvsol 350 / 5,000 /
80 / 7,950; armsolar 155 / 0 / 20 / 2,600; armwin 40; armgeo 560 / 300; armageo 1,600 / 1,250; armmakr 1 / 1,150;
armmmkr 380 / 21,000 and cormmkr 370; armamd 1,500 / 38,000; armshltx 7,900 / 1,800 power and corgant 8,400; armestor
170 / 6,000; armnanotc 230 / 200 (1.15 metal a point); armmex 50 / 500 / 3; armcv 1,950 energy / 4,050 / 90 power;
armack 430, corack 470, armacv 550, coracv 580; armcom 2,700; corkorg death blast 640 / 10,600; armguard 1,220; armbull
460; armbeamer 490; armjanus 380; armstump 350 and speed 75; armflea 132; armfav 168 and 370 energy; corfav 153;
armflash 101; armpw 87; armmship 1,550; armaca health 200. Every ratio in late-game.md 25-30 and 52-77 reproduces.

Mismatches:
1. cases 118: advanced constructor "410 to 470": Armada/Cortex bots are 430 / 470, vehicles 550 / 580; 410 is Legion's
   legack. Line 148 of the same file says 430 to 580.
2. cases 304-305: "the plant alone draws 48 energy a second" making a constructor: armvp's build power is 150, which
   draws 72; 48 implies 100. The 220 with the commander and the Rover every 3 s do reproduce.
3. cases 329: "a Pillager (range 1,300, from the advanced plant)": no unit named Pillager in units.csv; the 1,300-range
   unit from armavp is the Ambassador (armmerl), which player.md:153 already uses.
4. cases 336: the Great Divide E5 pass "700 wide": docs/knowledge/maps/great_divide_v1.md:29 says 644. The "1,700 wide
   at the median" was not found.
5. cases 105: "six Stouts beat one Fatboy (+0.12)": docs/knowledge/units.md:209 reports the margin moved to +0.52 after
   the area-of-effect correction.
6. late 34: "on Comet Catcher the experienced players build no converters": K-eco-energy-storage-is-the-cheap-buffer
   counts 10 of 80 sides with converters (3 to 24).
7. late 153: BARb medium's anti-nuke "in one long game of six, at 25": K-barb-anti-nuke-by-minute says 135 of 815 long
   games, median 25. Possibly a different denominator (long *player* games on Comet); unknown.

Not in the glossary, grounded elsewhere: the missile's 1,000 / 125,000 / 120 s, the interceptor's 150 / 7,500 / 90 s,
the warhead's 12,500 over 640 and 13 s of flight, the anti-nuke's 2,000 coverage (weapon files and the 2026-10-09
scenarios, docs/knowledge/game-rules.md); the wreck's 60% (alldefs_post.lua); the moho's 600 metal of storage (no
column); the nano's reach 400; the commander's amphibious flag and the advanced solar's builders (the made-by column,
not read); the Comet spot's 2.3 / 9.2 metal a second (map data); "the start's 1,240 energy" (a 1:15 sample in
player-12, docs/knowledge/openings.md:527, not the start).

## Conflicts

1. **What ends a game.** winning 75-77 says "his commander or his ability to produce"; cases 168-169 says "a side loses
   only when it has no commanders left (the game-end rule)"; apex60-1's row says the game needs a living commander
   (Lazarus raised two). One engine fact, stated three ways.
2. **The anti-nuke's timing.** late-game 172-188 (the user, 2026-10-20: on the opponent's clock, not on his tier 2)
   replaced the 2026-10-19 "due diligence once his tier 2 stands", which docs/briefs/player.md:168-178 still carries.
   The standing brief argues the ruling this file retired.
3. **A moho frame on one advanced constructor.** late-game 23-24: about 70 s (arithmetic); cases 160-162: 90 to 135 s
   (measured on Center Command with the store empty). The brief presents the measured time as the builder's.
4. **The pool's tier-2 count.** winning 27 and cases 125: 4 of 58 sides; late-game 3-4: 4 of 80 sides.
5. **The game-name key.** cases 10-13 says human-N games are on Quicksilver unless said; human-10 was on Gecko Isle,
   human-18 on All That Glitters, human-23 on Comet Catcher, and none of their passages says so.
6. pianist-player-3 is "against BARb medium" in cases 77; the ledger row says easy.
7. pianist-player-6: cases 270-272 says Pawns on the raiders' passages stopped the raids; the ledger row says the player
   built no Pawns.
8. cases 240-241 attributes games 3-5 to "a person" (he); the ledger has three opponents (thebluegecko, Irishstud14,
   Infern8).
9. BARb's tier-2 clock (winning 21-23, cases 175-179): the same curve with and without counts, the profile unstated in
   both; the source claim not found in docs/knowledge.
10. The Pillager / Ambassador name (mismatch 3) against player.md:153.
11. The Great Divide pass width (mismatch 4) against the map file.
12. The converter habit of the pool (mismatch 6) and medium's anti-nuke count (mismatch 7) against their claims.
13. "No game of ours has met [Great Divide] yet" (cases 336): the ledger has bluegecko-2v1-great-divide and the
    distilled Great Divide rows.

## Era-bound passages

- BARb-specific curves presented as general: winning 21-23 (his army flat from 10:40, doubling from 20:00), winning
  45 ("his tier 2 comes on its clock whatever the store holds"), cases 175-179 (10:41 in 73 of 74), cases 377-378 (his
  base in the north from the corner start), late 81-83 (his plant 10:41 and moho 15:12), late 151-158 and 180-181 (the
  anti-nuke at 21-25). The profile is named only in the late-game passages.
- Pre-rebuild hands (before 2026-10-02): 59 rows, every case from comet-N, escalate-N, pianist-player-N, the 3v1s and
  player-9..32. Lessons that are partly a defect since fixed: pianist-player-2's standing ball (the control lane's hold),
  player-9's scouts (turned home, sent hunting), escalate-4's commander held by a list step, player-10's thin line (new
  units joined at the front), human-23's dropped scout ask, nuke-2-hard's unfired rounds (no missiles line).
- The 2026-10-03 layer series (player-40..54, sonnet-*): the rebuilt hands before rove-as-task, the exit-lane and
  priority fixes, the impedance fixes and the standing transfer.
- The lobby and Apex games of 2026-10-18..20: before the standing transfer and the roster-by-builder fix; apex60-1 on
  the regressed brief (commit 4049e21), which the "mohos are every seat's" and "anti-nuke" lessons draw on.
- The pool rows (18) are Comet Catcher Remake 1.8 only, OS 40+, mined 2026-09-23..10-03; Quicksilver has no cards.
- Undated (50 rows): the facts and tool text, the simulator tables (the body-sizes and light-post numbers carry their
  study dates), and two cases with no game named (cases 229, 399).

## What could not be classified

- cases 22-23 ("what the first soldiers met there"): population unknown.
- cases 229 (advanced solars started at 18:22 into an empty store) and 399 (the hands' default-packet games): no game or
  date; proposed discard.
- The source of BARb's 10:41 / 73 of 74 / army doubling (winning 21-23, cases 175-179): not found in docs/knowledge by
  search; grounding marked unknown.
- "A turret counts about three times its metal" (winning 61-62): a harness constant whose origin is unknown.
- cases 336-337's "the players expect a camp at the pass" and "1,700 wide at the median": no person or source found.
