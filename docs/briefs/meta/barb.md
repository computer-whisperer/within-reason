<!--meta
id: F-barb-how-it-decides
layer: fact
population: BARb (the stock skirmish AI, CircuitAI's BAR build), every profile
dates: 2026-09-19 (the source read; CircuitAI 1.6.30 source against the 1.6.28 binary the game ships)
ground: code (BARb's own configuration and AngelScript in the game archive, its C++ in upstream/CircuitAI)
sources: {code: [upstream/CircuitAI/src/task/fighter/AttackTask.cpp:248-333, upstream/CircuitAI/src/task/fighter/RaidTask.cpp:250-319, upstream/CircuitAI/src/module/MilitaryManager.cpp:616-635, upstream/CircuitAI/src/module/MilitaryManager.cpp:1667-1714, upstream/CircuitAI/src/task/fighter/DefendTask.cpp:107-118, upstream/CircuitAI/src/task/RetreatTask.cpp:90-188, upstream/CircuitAI/src/module/BuilderManager.cpp:938-989, upstream/CircuitAI/src/module/EconomyManager.cpp:1049-1191, upstream/CircuitAI/src/module/FactoryManager.cpp:1038-1072, upstream/CircuitAI/src/map/ThreatMap.cpp:374-408, docs/knowledge/opponents-barb.md], claims: [K-barb-anti-stat, K-barb-raid-threat-gate, K-barb-raid-timing, K-barb-attack-gate, K-barb-retreat-random, K-barb-commander, K-barb-economy-rules, K-barb-constructors, K-barb-tier2, K-barb-information, K-barb-base-defence-clock, K-barb-response, K-barb-no-resource-cheats]}
status: live
-->
## BARb: how it decides, and what it does

BARb is a task-pool AI. Its builders take the cheapest-to-reach open task from one list (extractor, energy, factory,
construction turret, defence, reclaim, repair); its factories roll dice on a probability table chosen by income tier,
overridden by a counter table when the enemy has much metal in some role; its fighters are sorted by role. The rules
below are read from its code and configuration and hold in every profile unless a profile is named. No profile has a
resource, build-speed or vision bonus: difficulty is made of caps, pauses and tables only.

- **Its armies go for buildings, not armies.** Every mobile land fighter is marked to target static things only, so an
  attack or raid squad walks to our buildings, the outermost extractors first, and shoots our units only on the way.
  A squad will not take a target whose threat on its map is at or above about three quarters of the squad's own
  power: a defended extractor is skipped, a bare one dies.
- **Raiders leave in fours.** Raider-role units wait at base until their power reaches a threshold worth about four
  Pawns or Grunts, then go; once a raid exists every new raider is released at once and merges on the way. Everything
  else gathers at home and is promoted to an attack when the group's power passes a bar (40 on medium, 60 on hard,
  or the threat of the second-largest enemy cluster it knows of); after the first attack exists, reinforcements
  trickle forward one by one. An attack targets the nearest reachable enemy cluster whose threat it can afford, and
  within it the nearest building; with none, it walks to a front position or home.
- **Its units run home when hurt, at a threshold drawn once a game.** The retreat threshold is uniform between 55% and
  100% of health on medium (50-100% hard, 10-100% easy), one draw per game; a hurt unit goes to a haven by its
  factories, asks for repair at high priority and returns at 98% health. Chip damage removes its units from the field
  for a long walk; pursuit kills them.
- **Its commander is leashed.** After three minutes (seven on hard) it takes tasks only within 1,000 elmos of its base
  (2,000 on hard until minute 7), hides when threatened, retreats at 89-100% health, never joins an attack, and D-guns
  what comes close: a party of Pawns on it loses three to seven a shot.
- **Its economy runs on storage fractions.** Energy stalling (store under 20%, or income under draw with the store
  under 60%) puts energy at the head of every builder's list; otherwise an extractor upgrade, a new extractor, a
  converter only when energy is full and metal is not, then reclaim. Constructors are made whenever mobile build power
  is under a factor times income (0.6 easy and medium, 1.2 hard): medium is constructor-poor, hard has twice the budget.
  A new factory or construction turret comes when income exceeds what the factories can spend. Static defence comes on
  a clock (medium: a light turret at 120 s and 240 s, a Beamer at 420 s, anti-air at 520 s, a heavy turret at 760 s)
  plus one light turret behind every tier-1 factory and defence per extractor cluster bounded by income.
- **It knows only what it has seen,** remembers a seen enemy at its last place until it looks there again, and
  rebuilds its threat map several times a second from everything remembered. It has no assumption about where we
  start: its first raid needs a scout to have found something. Killing its one scout delays every raid.
- **Tier 2.** On medium the second factory is an advanced lab chosen by reach, built when spare income covers it or at
  the factory switch every 15 to 20 minutes; once it stands the tier-1 lab makes only constructors. On hard the switch
  is driven by stored metal and income and comes near 11:00 on Comet (the observed sets below).

<!--meta
id: F-barb-profiles
layer: fact
population: BARb's profiles easy, medium, hard, hard_aggressive
dates: 2026-09-19; 2026-10-20 (hard against hard_aggressive diffed from the game archive)
ground: code
sources: {code: [run/barb_profiles.py, run/data/barb-profiles, docs/knowledge/opponents-barb.md], claims: [K-opp-barb-tiers, K-opp-barb-hard-vs-hard-aggressive, K-barb-medium-vs-easy, K-barb-easy-faction-asymmetry, K-opp-faction-asymmetry]}
status: live
-->
### The profiles

- **easy** and **medium** run the same scripts; the difference is configuration. Easy has no Armada raiders (its Pawns
  and Blitzes are relabelled skirmishers, but its Cortex Grunts still raid, so easy BARb is much harder as Cortex than
  as Armada), its scouts off, extractors capped at 30% of the map's spots, a three-second pause after every build,
  retreat anywhere from 10% health, an attack group at power 30, no counter-building but anti-air. Medium raids with
  both factions from the first lab, caps extractors at 80%, pauses one second fading to none by minute 15, runs the
  full counter table (riot against raiders, assault against turrets, anti-air against air), retreats at 55-100%,
  escorts its first two constructors, attacks at 40, allows fusions.
- **hard** and **hard_aggressive** have their own scripts: randomised openers of 3 to 10 raiders and 3 to 5
  constructors, economy-driven factory switching, twice the constructor budget, no build pause, no extractor cap, 8 to
  12 construction turrets a lab, a commander that roams to 2,000 elmos until minute 7, attack at 60. For Armada and
  Cortex on land the two differ only in the construction-turret cap a factory (hard 8/8/12 against 3/4/6), the tier-1
  labs' switch weight (hard never returns to a tier-1 lab; hard_aggressive may) and the threat-range floor at high unit
  counts. "Aggressive" is not in the scripts. At +0 on Comet the caps did not bind and one game on each profile at the
  same seed ended alike (won 16:30 and 16:54); with a bonus the caps bind and hard turns the income into turrets and an
  early army (eight turrets at its lab by 12:00, 4,126 of army at 10:00 at +20 against hard_aggressive's 1,379-3,040).

<!--meta
id: M-barb-hard-aggressive-comet
layer: meta
premises: [F-barb-how-it-decides, F-barb-profiles]
population: BARb hard_aggressive at +0 on Comet Catcher: 150 games of ours that lasted six minutes or more, 2026-09-27 to 2026-10-20, across every version of our side
map: Comet Catcher Remake 1.8
dates: 2026-10-22 (regenerated from the truth files of run/matches and the archive; the reference set of 2026-10-17 had 110)
ground: truth (the truth file: everything he had, seen by us or not)
sources: {code: [run/opponent_set.py, docs/briefs/reference/comet-catcher-remake-1.8-barb-hard_aggressive.md], claims: [K-barb-hard-aggressive-tech-clock, K-barb-anti-nuke-by-minute]}
status: live
-->
### hard_aggressive on Comet Catcher, +0

The arena's standing opponent. Its games against us ran 12:44 / 20:56 / 29:13 (p25 / median / p75). It is a
description of what it does against the play we gave it, not a pace to match.

- **Opening.** A bot lab in 107 of 150 games and a vehicle plant in 36 (Cortex 7), started 0:52 / 0:54 / 1:02. Its first
  soldier is in our half at 1:48 / 1:50 / 1:54, a Tick in 107 games and a Rover in 36; our first extractor dies at
  2:46 / 3:15 / 4:27. Two extractors at 1:00, 5 at 3:00, 6 at 4:00, 8 at 5:00, 10 at 6:00, 13 at 8:00, 16 at 10:00, 18 at
  12:00, 21 at 16:00, 25 at 20:00, 28 at 25:00, 32 at 30:00 (medians). Constructors 5 at 4:00, 9 at 7:00, 11 at 10:00, 15 at
  16:00, 23 at 20:00, 48 at 30:00. Turrets 2 at 4:00, 7 at 6:00, 11 at 8:00, 16 at 10:00, 21 at 12:00, 28 at 16:00, 35 at
  20:00, 57 at 30:00: a turret beside nearly every extractor and nests in our half later.
- **Army.** Metal standing 270 at 4:00, 625 at 6:00, 1,572 at 8:00, 2,572 at 10:00, 3,274 at 12:00, flat at 2,786-2,792
  through 14:00-16:00 (its tier-2 switch), 3,583 at 18:00, 4,674 at 20:00, 8,781 at 25:00, 16,360 at 30:00. Soldiers in
  our half: 1 to 2 until 9:00, 3 to 4 from 10:00 to 20:00, 12 at 25:00, 15 at 30:00 (medians; upper quartiles 5 to 13 from
  8:00 on). Composition: Pawns half of the metal to 6:00 with Blitzes a third; Centurions and Rocketeers from 7:00
  (Centurions 23% at 10:00, Rocketeers 17%, Pawns 20%, Hammers 7%); Fidos and Mavericks from 14:00; Sharpshooters 11-16%
  from 16:00 to 18:00 and Fatboys 13-22% from 18:00 to 25:00 with Bulls, Zeus and Stouts; Razorbacks 8% at 25:00 and
  24% at 30:00 with Vanguards 10% and Thors.
- **Tier 2.** An advanced factory in 115 of 150 games and in 103 of the 106 that lasted fifteen minutes, started 10:11 /
  11:04 / 11:57 (an advanced bot lab in 79, an advanced vehicle plant in 30), with 15 / 16 / 17 extractors and 2,152 /
  2,568 / 3,307 of army when placed. The one observed clock of its climb (escalate-4, 2026-09-22): Centurions and a
  Gunslinger by 12:00, a Fatboy and Sharpshooters by 19:00, a Pulsar and a Marauder by 24:00, a Razorback at 25:00; the
  set's composition confirms the order.
- **The nuke.** An anti-nuke in 27 of the 33 recorded games that passed twenty minutes (2026-10-09), first standing at
  minute 21 to 25 (median 22); its own silo in 14 of 848 long games against BARb of any profile, at 25 to 38 minutes, and
  one begun at 29:50 in the hard game of 2026-10-20. A nuke that is to land on it must be stockpiled before about 21:00.

<!--meta
id: M-barb-medium
layer: meta
premises: [F-barb-how-it-decides, F-barb-profiles]
population: BARb medium: 29 games on Comet Catcher with truth files (2026-09-22 to 10-20, mostly the escalation and pianist series); 430 games on Quicksilver in the September batches (the tempo study; the records of that time carry no seats, so the generator does not read them)
map: Comet Catcher Remake 1.8; Quicksilver Remake 1.24
dates: 2026-10-22 (Comet set); 2026-09-19/20 (Quicksilver claims)
ground: truth
sources: {code: [run/opponent_set.py, run/compare_census.py, run/batch_curves.py, docs/studies/tempo-model.md], claims: [K-barb-medium-observed-build, K-barb-medium-army-curve, K-barb-opening-varies, K-barb-tempo-is-half-clock-half-sighting, K-barb-we-lose-the-fights-not-the-build, K-opp-medium-wins-by-20]}
status: live
-->
### medium

**On Comet (29 games).** A vehicle plant in 18 and a bot lab in 10, started 0:56 / 1:06 / 1:06; its first soldier in our
half at 2:14 / 2:18 / 2:32, a Blitz in 18 games; our first extractor dies at 3:29 / 4:02 / 4:22. It expands slowly (5
extractors at 4:00, 7 at 6:00, 9 at 8:00, 11 at 10:00, 14 at 12:00, 17 at 14:00, 24 at 20:00) with one to two
constructors to 8:00 and never more than seven, but turns its metal into army: 550 at 4:00, 1,220 at 6:00, 2,038 at
8:00, 3,303 at 10:00, 4,663 at 12:00, 5,108 at 14:00, 5,972 at 20:00, with 5 of its 14 soldiers in our half at 6:00, 10 of
20 at 8:00, 13 of 33 at 12:00 and 16 to 22 of 38 to 45 from 14:00: the raids in fives from 6:00 and the block near 10:00.
Blitzes are 95% of its metal at 2:00, 73% at 4:00, 59% at 6:00, 41% at 8:00; then Hammers, Janus, Stouts and Rocketeers
(Stouts 24-32% from 14:00, Janus 20-25%). An advanced factory in 9 of 29 games (9 of the 21 that lasted fifteen minutes)
at 19:32 / 19:52 / 21:16, with 26 extractors and 7,814 of army when placed.

**On Quicksilver (September, 430 games against the heuristic bot).** One lab and one construction turret for the whole
game, about one turret an extractor, one or two constructors to minute 9 and then ten to twenty; a bot lab in 70% of
games and a vehicle plant in 30% at the same profile (its dice, which no seed controls), and the two openings made
different games against us. Its army, from the north-west corner: 520 at 4:00, 1,250 at 6:00, 2,800 at 10:00, 3,800 at
12:00, 4,700 at 15:00, 5,400 at 20:00, hoarded and growing rather than spent; twelve Blitzes by minute 10 and the block
at 16 in one series. Its state at a minute can be estimated from what we have seen to about half the error of the
clock alone, all of the gain after first contact. Against the bot of September it won every game between minutes 16 and
20, the game decided at 8-10 when our extractors fell to raids walking past our army.

<!--meta
id: M-barb-bonus
layer: meta
premises: [F-eco-multipliers, M-barb-hard-aggressive-comet]
population: BARb hard_aggressive with a lobby bonus (the engine's team Handicap, an income multiplier) in 1v1s on Comet Catcher against the flagship, 2026-10-19; hard at +0, +10 and +20, 2026-10-20
map: Comet Catcher Remake 1.8
dates: 2026-10-19, 2026-10-20
ground: truth and the ledger
sources: {code: [docs/experiments.md, run/matches/1792433593-barb-ladder-50-1, run/matches/1792475322-barb-hard-10-1], claims: [K-opp-barb-hard-vs-hard-aggressive]}
status: live
-->
### With a bonus

Twelve 1v1s with the bonus on BARb: +0 1-0, +10 2-0-1, +20 0-2, +35 1-1, +50 0-2, +100 0-1, +150 0-1. hard_aggressive
evens the score at about +15 to +20. Every loss had one shape: a lead or a level game to about 20:00 not converted, then
the bonus income, a static line (25 light turrets, 13 Beamers, 7 Annihilators and 7 Ambushers in one game) and the
gantry (Big Berthas, Annihilators, a Thor) past 25:00; the wins all ended by 25:00. The largest unconverted lead of the
ladder was 10.4 times his army at 13:00 and 3 times at 30:00, standing in front of his line until the timeout. hard at
+10 and +20 was won both times (21:14, 21:16): it spent its bonus in early turrets and attacks that lost.

<!--meta
id: M-barb-team
layer: meta
premises: [F-barb-how-it-decides, F-eco-reclaim]
population: eight BARb hard_aggressive against eight seats of ours on All That Glitters (four games, 2026-10-04 to 10-10); two BARb medium against a seat of ours and the user on Altair Crossing (human-14, 2026-10-03)
map: All That Glitters v2.2.3; Altair_Crossing_V4.1
dates: 2026-10-04 to 2026-10-10
ground: truth (Glitters), sightings (Altair)
sources: {code: [docs/experiments.md, run/team_read.py], claims: [K-team-eight-seats-lose-to-eight-barbs-on-one-curve]}
status: live
-->
### Eight of them

Eight hard_aggressive against eight seats of ours on All That Glitters, three games lost at 28 to 30 minutes on one
curve and a fourth undecided at 30:00: level at 8:00 (70 extractors against 67, 98 soldiers against 92), his army
triples by 16:00 while ours stands, every fight in our half from 8:00, his extractors 65, 67, 73, 81, 94 from 12:00 on
against ours falling. His 247 Lazarus raised 286 of our dead in one game (142 standing at the end, a quarter of his
army, a commander of ours among them); his advanced labs came at 12:06, 16:24 and 19:04 in the fourth game. On Altair
Crossing two medium BARbs (one Armada bots at H5, one Cortex vehicles at G1-G2) sent Pawn swarms of 11 to 14 at
8:13-9:16 and Levelers that broke a Pawn raid and beat 16 Maces with five at 10:07.
