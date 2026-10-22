# Part a: `docs/briefs/player.md`, passages beginning at lines 1-600 (reader A)

Rows: `part-a.jsonl`, 94 rows over 65 passages, covering lines 1-657 without a gap (the last passage of the range,
the Comet opening bullet, begins at 581 and runs to 657; reader B starts at 658). A bullet that argues several
subjects is split into parts with the same line range. Quotes are the passage's opening words; the brief wraps its
lines, so a quote may span a line break.

Sources used: the text; `docs/experiments.md` (the ledger rows of every game the range names); the status lines of
every `K-` claim the range cites (all 50 exist in `docs/knowledge/`); `docs/glossary/units.csv`;
`docs/briefs/experience/*.md`, `docs/briefs/reference/*.md`, `crates/bot/src/strategist/*.md` and
`crates/bot/src/brain/pianist/rules.md` for duplicates; `git blame` for the date each line was written.

## Counts

By layer (what the passage is now): mixed 27, tool 20, fact 15, derived 10, meta 9, episode 9, ruling 4.

By fate (what it would become): rederive 31 (derivable yes 22, partly 9, no 0), tool 21, fact 15, meta 14,
episode 9, discard 4.

Doctrine flags: 38 of 94 rows carry a rule ("never", "always", "the only question"): 15 from the pool's numbers, 14
from a ruling, 9 from one case. Lens: ledger 65, person 19, model 4, chat 3, unknown 3. 28 rows carry conflicts
(48 in all); 29 rows carry a glossary check, 4 with a mismatch.

Four discards: the 2026-09-21 provenance paragraph (5-9), BARb medium's army curve on Quicksilver (364-368 part 2),
the Economy heading's three 2026-09-19 conjectures (373), and the old-hands vocabulary mapping (446-451).

## The derivation backlog

Each subject with the lessons the derivation must reproduce or explain away (the rows' `premises` give the inputs).

1. **The scenario variables** (16-29). Reproduce: a Comet duel is decided inside fifteen minutes; a team game runs
   past forty and goes to tier 2; a seat's tier-1 ceiling is spots over seats (84 and five seats: eight each). This
   is the skeleton every clock below hangs on; derive it first.
2. **Pressure from the first units** (30-55 parts 3-4; 334-343 part 2; 493-500). Reproduce: the pool's soldier in his
   half by 2:06 and first building dead by 3:49; a light post (1-3 Sentries) falls to the handful as one body (the
   simulator's shares); the picket is the turret beside the outer pair. Explain away or scope: the Quicksilver
   "first Grunts are the guard" case (a person's 2:00 Pawn rush) and the escalate-4 pickets rule (BARb's raid gate).
3. **A lead decays** (56-91 parts 1-2). Reproduce: the pool's 3.4-minute median from 2:1 to the end; the losses of
   player-9/10/18 from 2.4:1 to 4.1:1; the push's destination is his base. Inputs: income parity, walk time across
   5,500-6,500 elmos, BARb's tier-2 clock.
4. **Body size and splash** (56-91 part 3; 555-569). Reproduce: 25 Stouts lose to 4 Fatboys by 0.55 at 56 spacing
   and 0.22 at 160; 6 Stouts beat 1 Fatboy; the pros' dozen raiders and eight Stouts as the habit the arithmetic
   predicts; "spread and few against area, tight and many against single-target".
5. **The tier-2 sprint** (92-145 parts 1-2; 146-156). Reproduce: the army does not grow for 3-5 minutes; the gap is
   lethal in a tight duel (the pool techs in 4 of 58) and covered by allies; one seat sprints and gives advanced
   constructors away; tier-1 bodies lose to tier 2/3 at any count (the +60 Altored pair). Fix the advanced
   constructor's cost (410-470 is the bots'; the vehicles are 530-580). Resolve: "starts now" against "at the first
   bank or when the spots are taken".
6. **Income past the spot count** (157-167; 183-200; 201-214). Reproduce: moho +6 for 620 repaid in 104 s and one
   advanced solar per four mohos; the fusion/advanced-fusion/converter ladder and its paybacks; the advanced solar's
   44% less metal and 24% less build time, drawn over 88 s. Already arithmetic; resolve the order (advanced solars
   beside the mohos, or fusions after them). Meta gap: where the pros switch from basic to advanced solars.
7. **The anti-nuke's timing** (168-182). Reproduce: the silo's clock against the anti-nuke's 60,000 build time and 90 s
   interceptor; the +60 Altored strike nine minutes after the silo stood. The 2026-10-19 "once his tier 2 stands" was
   reversed on 2026-10-20 in `experience/late-game.md` ("on the opponent's clock"); the derivation must say which.
8. **Unit counters** (215-233 part 2; 328-331; 332-333; 570-580 part 2). Reproduce: Janus (380) over Stout (350);
   artillery (710) behind a screen; line bots over raiders; Rocketeer (475) over the Sentry (430). Explain away
   against the glossary's sim: "Pawn beats only Grunt" (the csv lists seven others and not Grunt), "a Stout beats any
   tier-1 bot head on" (the csv has it losing to the Mace at -0.34).
9. **Raids and catching** (234-246; 332-333). Reproduce: the Rover (168) catches a Tick (132), a Blitz (101) never;
   BARb's squad skips a target worth three quarters of it; a turret beside each outer pair. Fix the name: the
   168 car is the Rover (`armfav`), the Rascal (`corfav`) is 153.
10. **The bank** (247-254; 374-385 part 1; 543-547). Reproduce: a bank means build power missing; the pool's five
    constructors by 5:00 and never past 700. Resolve three orders: constructors and nanos first (247), a second lab
    or nano (374), a constructor a minute to four by 4:00 and eight by 8:00 from an unnamed person (543, above the
    pool's medians 3/5/8 at 5/8/12). Note escalate-5/6's bank was partly a hands defect fixed before escalate-7.
11. **The first four minutes** (255-281 parts 2-3; 486-492; 520-530; 548-552 part 2; 581-657 part 5). Reproduce:
    metal-bound, the store at 150 at 2:00 and 100 at 3:00; three solars and never a fourth before the plant; a
    constructor under the assist draws about 220 with the commander; the lab before any generator empties the store
    (human-3). Fix: "the plant alone draws 48 a second" (72 by the csv's 150 build power; repeated in cases.md:304)
    and "a lab draws about 80" (the vehicle plant's figure; the lab is 57-63). Resolve: eight solars "before and
    while the plant goes up" (548) against never a fourth before it (584).
12. **Turret lines and value on the spot** (282-292; 350-354; 293-301). Reproduce: reach decides (Gauntlet 1,220,
    Beamer 490, Bull 460 against 350-380); a turret counts about three times its metal (a 2026-09-19 fit on the
    heuristic bot's easy/medium games: re-derive from turret dps, hit points and range); one body at one place
    (speeds string a body out). Note "the Pillager (1,300" is not a glossary name; 1,300 is the Ambassador's range.
13. **Scouting** (302-311; 454-475 part 2). Reproduce: no push onto an unseen place; a radar by 3:00; his base at the
    end of his strip. The scouts' deaths behind it (player-9) were the old hands' retreat and hunt rules, fixed
    2026-10-01/07; the rove move is tool text.
14. **The commander** (312-321). Reproduce: it fights at home and builds at the plant (2,700 metal, 300 build power,
    37.5 speed, the D-gun). Resolve: "never a walker" against the pros' walk toward the middle from 2:00 (653) and
    the reclaim for 2,700 (121). Two of the five cases were harness faults walking it.
15. **Expansion** (386-387; 543-547). Reproduce from BARb's raid targeting: constructors die alone at far spots.
    Resolve "steps the army has covered" against "the midline is not a wall, nearest first".
16. **Air** (344-349). Reproduce: tier-1 ground does a fifth of its damage to aircraft (csv 15-22%); derive the
    gunship clock from the Banshee's 5,050 build time instead of human-9's ninety seconds.
17. **Wrecks** (388-398 part 2). One rule to derive: never a rez bot onto a live field (220 hit points).

## The meta catalogue

Entries the rework keeps as observed behaviour, with population, map and date.

- The pool at OS 40+, Comet Catcher, 80 sides of 40 duels (2026-10-03): a soldier in his half by 2:06 (Armada
  1:40, Cortex 2:48), first building dead 3:49, winners 40 s earlier with 78 of 280 army metal in his half at
  3:00, 1.85 extractors lost a game-minute and winning anyway (30-55 part 1).
- The pool at OS 40+, Comet, 2026-09-24 (33 duels; the median order): three solars, the plant standing 1:07, the
  fourth solar 1:33, five solars and eighteen factory units by 4:00, bank 567/152/99 at 1:00/2:00/3:00, the store
  under 20 in 9% of seconds, the commander on the plant 50 s; winners assist less (32 s) and bank less at 3:00 (72)
  than losers (93 s, 107); 7, 13, 17 extractors at 4:00, 6:00, 8:00 (p25 6, 10, 12); the second plant by 7:20; the
  first turret pairs 13 by 8:00 against 9.5 (581-657 parts 1-2, 4).
- The pool at OS 40+, Comet (the body-sizes study, 2026-09-28): the largest body of one type a dozen raiders or
  line tanks and eight Stouts; bodies over thirty late and mixed (56-91 part 3).
- The pool at OS 40+, Comet: a Comet duel ends 3.4 minutes (median) after one side first holds twice the other's
  army; tier 2 in 4 of 58 sides (56-91 part 1; 92-145 part 1).
- The pool at OS 25+, Comet, 29 duels (2026-09-23): resurrection bots in 24 of 58 sides (median ten a side, the
  first between 4:00 and 10:00), worked on their own ground; a bot within 400 of a death in the next 90 s in 22 of
  2,730 deaths; winners' income above their extractors' in 12% of minutes against 7% (388-398 part 1).
- The pool, Comet: an energy storage by 9:36 in 23 of 80 duels; five constructors by 5:00, three nanos on one plant
  by 6:37, never past 700 banked (374-385 part 2; 247-254).
- thebluegecko (OS 48.5), Comet, 13 of 14 Armada sides (10 won against OS 29-43), 2026-09-24: the Rover mass (three
  solars, the plant 0:58, a Blitz then Rovers every three seconds, 24 factory units by 4:00, Rovers one by one to
  the enemy's extractors from 1:54); his bot-lab line once (581-657 parts 3-4).
- thebluegecko, chat in the 3v1s of 2026-09-27/28 (Comet, SailAway): "Top should stop building other units while
  doing t2 transition"; "i think you made too many lashers ... so I was safe to go T2"; "brutes would have given you
  the ability to apply more pressures"; "mongonals ... 1-3 of them with other units in front"; "eat your bot lab";
  "make sure your units are more of a straight line"; "up by almost 400% army value, time to go fight" (215-233
  part 1; 92-145; 293-301).
- VAK (OS 52) against Artur91, Comet, one game (2026-09-22): the Rascal-constructor line, thirteen Blitzes by 5:00,
  9/13/19 extractors at 4:00/5:00/6:52, three nanos by 6:37, the commander toward the middle from 2:00 (581-657
  parts 4-5).
- Matt, Quicksilver, one replay against BARb medium (2026-09-20/22): two extractors, three wind, the lab 0:33, seven
  wind and a solar as a batch, one constructor then Pawns from 1:12, a second constructor at 1:45, raiders out from
  2:30 (501-519).
- The lobby players on Quicksilver, 2026-09-22 (OS unknown): wind 12 is good, one solar then wind, twenty energy per
  metal, "two constructors is very greedy on this map"; a person's 2:00 Pawn rush (486-500).
- BARb's profiles on Comet (the arena, 2026-09-23): a bot lab in 91 of 104 games; hard_aggressive: Pawns and Ticks
  from 2:08, six extractors at 3:00 and fifteen at 8:00, turrets from 4:00, a Beamer by 8:00, the advanced lab near
  11:50 (553-554; 581-657 part 4). BARb's code (fact, not meta): the raid squad of four, the three-quarters threat
  gate, the attack gate, the counter-build rules (355-372).
- Apex v0.3.2 on Altored at +60 (2026-10-19): a silo at 24:28 in one game, none in the next (168-182).

## The episode index

Game, what the brief takes from it, the harness defects of its time, the lens.

| game | taken as | defects of the time | lens |
|---|---|---|---|
| comet-1 (easy, 2026-09-23) | no radar; fourteen Blitzes into the D-gun | the first Comet hands; the plant at 1:40 on the old menu words | person (the user: "radars would have helped") |
| comet-3, comet-4, comet-5 (medium) | six solars; 22 Blitzes onto turrets; 2,100 banked; Janus over Stouts; artillery eaten | the first queue list; five guard groups; lists that could not be cancelled | ledger |
| escalate-1 (easy), escalate-3 (hard) | piecemeal into the D-gun; the walk round the fortress | pre-hands-rebuild | ledger |
| escalate-4 (hard_aggressive, 2026-09-22) | 33 extractors to single Ticks; the commander never fought the Razorback | the old engage pick; an old assist list blocking the fight | person (the user's pickets rule) |
| escalate-5, escalate-6 | the bank of 1,600-1,900 | the plant chose a constructor 1 of 17 times; `armcv:2` misread (fixed before escalate-7) | ledger |
| escalate-7 (WON 16:00) | pickets, constructors 8 of 13 asks, the block broken at D4 | the two produce fixes just landed | ledger |
| realtime-2 (hard, 2026-09-22) | the ball after one scout car | the old engage pick | ledger |
| human-1, -3, -5, -6, -7, -8 (Quicksilver, 2026-09-22) | the lab before generators; the second constructor at 1:15; Grunts in ones and twos | the lab asked three times in six seconds; generators one pick at a time; the hands could not count; newcomers in groups of one and two (each fixed the same day) | person (the user) for human-8, ledger for the rest |
| human-9 (Comet, 2026-09-23) | an air plant seen means gunships in ninety seconds | the refused extractor site; the spot table off by 82 elmos | person/ledger |
| human-10 (Gecko Isle, 2026-10-02) | 35 extractors and 70 a second: "not enough mex" | the first lobby game on the rebuilt hands; prose-only pre-game turn | chat (thebluegecko) |
| bluegecko-3v1-1..4, -9, -10, -11 (2026-09-27/28) | 17 extractors unanswered; Blitz dives at the D-gun; groups alone; Incisor swarms; 94 tanks to 15 Bulls; the sprint botched; the hover army | the standing tool per seat broken (1); the retreat rule walking the commander home (2); the Cortex roster refused (3); lobby boxes not honoured (10); the first water map (11) | chat (thebluegecko) and ledger; 3v1-10's root is a model review |
| player-9 (2026-09-27) | the lead unspent at 6:00; scouts that never arrived; the Gauntlet nest | the raid state never picked; the old retreat and hunt rules | person ("look even earlier: 6:00") over the model review |
| player-10, player-11 (2026-09-28) | the lab seen at 3:45 by rove; the lead spent at 8:27; the tech matched late | station walks; rovers ranking seen places first | ledger (player-11 not reviewed) |
| player-12 (2026-09-28) | the starved opening: two constructors under the assist | the first game with the timed assist holding | model review, verified in the ledger |
| player-15 (2026-09-28) | 24 Stouts in one body | the Rover-mass line; the `all` list binding the constructors | model review, verified |
| player-18 (2026-09-28) | five launches at the empty strip | the engagement planner before hysteresis; the fall-back rule | model review, verified |
| player-20, -21, -22 (2026-09-28/29) | rove kills nothing at a patrolled base; the committed mass dies | the old `lane rove`; the leash before its anchor fix; the order storm the user called broken | model review (20, 21), the user's call (22) |
| player-40 to -45 (2026-10-02/03) | the first soldiers kept as pickets; his first building dead late or never | the places layer and thrash pair before human-11's fixes; -41 and -43 cancelled on harness bugs | ledger |
| player-49 (WON 15:01), player-50 | the raid recalled; the light post at F1 razed and "never attacks turrets" | the march fixes of the day | person (the user's replay reading of player-50) |
| rove-task-1 (2026-10-08) | "run from what can catch them" read as a walk home | the watchdog's six false alarms; the brief's wording changed after | ledger |
| human-8v8-1 (Altored 5v5, 2026-10-18) | Apex's Legion seat sprinting at 5:20 on income 27; our D8 lab killed with 4,000 in it | +50% income; the first lobby game with Legion | ledger |
| apex-v032-altored-3v3-bonus50-1 | 820 (or 2,000) energy a second wasted on one converter | the bonus words' first game | ledger |
| apex60-1, apex60-2 (2026-10-19) | two seats moho-less for an hour; the silo seen nine minutes late; then every seat a moho and an anti-nuke | apex60-1 ran before the 4b3980c awareness passages; H-REPORT-BLAST-CAUSE not yet in | ledger |
| human-26, human-27 | the lane took a group off a raid (1:57); "leave it to rove" | no ledger rows exist for these games | unknown |

## Numbers checked against `docs/glossary/units.csv`

Matches: the whole tier-1 bot roster (322-327) and vehicle roster (570-580); Rocketeer 475 against Sentry 430;
speeds Tick 132, Rover 168, Rascal 153, Blitz 101, Pawn 87; armavp 2,600/14,000 and coravp 16,000; armmoho 620 and
20 upkeep, cormoho 640; armadvsol 350/5,000/80 and coradvsol 370/4,000, armsolar 155/0/20, corsolar 150; the
advanced solar's 44%/24%, 88 s, 57 and 4 a second, 62 s (all follow); armfus 750, corfus 850; armafus 9,700,
3,000 a second, 312,500 build time (nine minutes on two advanced constructor vehicles, 143 s with eight nanos);
armmmkr 380; 11,600 for +52; armestor 170/6,000; armmstor 330/3,000; armsilo 8,100, corsilo 7,700; the three
anti-nukes 1,500 and 38,000-40,000, built by advanced constructors only; the commander 2,700, 300 build power;
Juggernaut (corkorg) 29,000 metal, 950 range, 640/10,600 death blast; Gauntlet 1,220, Beamer 490, Bull 460, Janus
380, Pounder 315, Mauser 820, Fatboy 700, Mangonel 700, Shellshocker 710; armcv 1,950 over 4,050; Pawn 1,650 at the
lab's 150 and the commander's 300; Blitz 900/110, Stout 2,100/225, Mason 1,950/135; the vehicle plant's 82 a
second at the commander's 300; tier-1 ground damage against aircraft 15-22%; armvp 590, corvp 570, armlab 500.

Mismatches: "the plant alone draws 48 a second" for a constructor (72 at armvp's 150 build power; cases.md:304
repeats it); "a Blitz is two Pawns' ... speed" (101 against 87); "a Stout beats any tier-1 bot head on" (the csv's
sim has it losing to the Mace at -0.34); "Pawn beats only Grunt and loses badly to Thug" (the csv's Pawn beats
seven types, not Grunt, and loses to Thug at -0.32); "the first advanced constructor (410 to 470)" is the bots'
range, the vehicles are 530-580; "a lab draws about 80 energy a second" is the vehicle plant's figure, the lab's is
57-63 at the commander's build power. Names not in the csv: Pillager (1,300 is the Ambassador's range),
Tumbleweed/Rascal for the two scout cars (the csv: Rover `armfav`, Rascal `corfav`), Bulldog, Instigator, Leveler,
Slasher, Coyote (Brute, Incisor, Pounder, Lasher, Garpike); "Rascal (168)" at 245 is the Rover.

Not checkable in the csv: areas of effect and damages (555-569), the Whistler's anti-air 700, the silo's 120/180 s,
the anti-nuke's 2,000 cover and 90 s, the converter's 600 into 10.3, the D-gun's 500, twenty solars' 2,000-2,500
storage, the start's 1,240 energy, the radar's 2,000 sight, map data (spots, wind, 2.3 a spot).

## Conflicts

Within the range and against the other briefs; the merge pass joins them.

1. **The anti-nuke's timing.** 168-182 (due diligence once his tier 2 stands; the user 2026-10-19) against
   `experience/late-game.md:172` (on the opponent's clock; the user 2026-10-20: "we have definitely been building
   the anti-nuke too early"). A reversed ruling that never reached player.md.
2. **BARb's tier-2 clock.** 369-372 (a tier-2 lab by 15-20 at the latest; K-barb-tier2's Quicksilver-medium
   conjecture) against 656 (hard_aggressive's advanced lab near 11:50) and `experience/cases.md:175` (10:41 median
   in 73 of 74 games).
3. **What the bank buys first.** 247-254 (constructors and nanos, a plant only after) against 374-385 (a second lab
   or a nano) against 543-547 (a constructor a minute per plant, four by 4:00, eight by 8:00, from an unnamed
   person, above the pool's 3/5/8 at 5/8/12 in cases.md:282 and the pros' five by 5:00 at 251).
4. **Solars before the plant.** 548-552 (about eight solars, built beside the commander before and while the plant
   goes up) against 584-590 (three, never a fourth before the plant) and 255-281 part 3 (a solar with energy not
   STALLING is three extractors spent on nothing).
5. **The commander's walk.** 312-321 (never a walker) against 653-655 (from 2:00 it can walk toward the middle as
   VAK's did), 584-590 (on the plant from the second it stands) and 121 (reclaimable for 2,700).
6. **When the sprint starts.** 108-109 ("the sprint starts now" once long, covered, none) against 110-111 ("at the
   first bank or when the side's spots are taken, whichever comes first").
7. **Who the first units are.** 30-55 part 3 (the first raiders go to his nearest spots; the picket is the turret)
   against 334-343 part 2 (pickets of one or two Blitzes at each outer cluster from the first Blitzes), 603-604 (two
   Rovers stay home from 2:30) and 493-500 (on Quicksilver the first Grunts are the guard).
8. **Energy for the mohos.** 157-167 (fusions after the mohos) against 183-200 (an advanced solar per four mohos).
9. **Expansion.** 386-387 (steps the army has covered) against 543-547 (the midline is not a wall, nearest first)
   and 653 (the commander toward the middle from 2:00).
10. **Light posts.** 30-55 part 4 (overrun by the handful as one body) against 282-292 (static defence never walked
    into); reconciled only by "a nest it outweighs".
11. **One episode, two numbers.** The +50 Altored converter: 820 a second for thirty minutes (382) against 2,000 a
    second for sixteen minutes (166).
12. **Against the glossary.** 570-580 part 2's Stout against the csv's sim column (loses to the Mace).

## Era-bound passages

- Quicksilver and BARb easy/medium, 2026-09-19 to 09-22 (the heuristic bot and the first lobby games; the map has no
  pool cards and no game since): the provenance (5-9), BARb's medium curve (364-368 part 2), K-barb-tier2's 15-20
  clock (369-372), the economy conjectures (373-387), the combat fit (350-354), the Quicksilver section and Matt's
  line (480-530), the eight-solars sentence (548-552 part 2).
- Pre-hands-rebuild (before 2026-10-02): the cases in 234-321 (raids, bank, opening, static defence, one body,
  scouting, commander) rest on games whose hands fell back, hunted, split, or could not count; the old-options
  mapping (446-451) exists only for them. The ledger names the fix beside most of them; the rows carry it.
- The Rover-mass games player-20/21/22 (2026-09-28/29) tested the old `lane rove` and the leash, not the build; the
  ruling that the mass needs micro the hands lack (613-629) was made on those games.
- Apex and the Altored 3v3s (2026-10-18/19): the sprint's team-game procedure, the tier-3 sentence, the anti-nuke,
  the post-moho ladder; each rests on one or two games on a map played four times.
- The pool (Comet, OS 40+, 2026-09-24 to 10-03) is the one population measured more than once; every clock from it
  is a Comet duel's at +0, which the scenario paragraph (16-29) already says.

## What I could not classify

- "The person's count" behind four constructors by 4:00 and eight by 8:00 (543): unattributed.
- Whether `rules.md` still carries the detachment sentence "word for word" (334): its first lines name the raids,
  not the detachment; reader E's file.
- Whether the hands still hold no spacing (565-566): true when written (2026-09-27), unknown after the rebuild.
- The scout sentence "that held under the enemy commander's eyes" (442): the game is not named.
- human-26 and human-27 (463, 473): cited as evidence, no ledger rows.
