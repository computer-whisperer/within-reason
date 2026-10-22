<!--meta
id: M-pool-intro
layer: meta
population: duels between people at OS 40 and above on Comet Catcher Remake 1.8
map: Comet Catcher Remake 1.8
dates: games 2026-08-09 to 2026-09-23 (game test-31251 to test-31383); cards cut 2026-09-23 and 10-03, re-cut 2026-10-13; the reference set regenerated 2026-10-17
ground: card (the engine replay of every game, every unit of both sides)
sources: {code: [run/data/replays/manifest.jsonl, run/replays/card.py, run/reference_set.py, docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md, docs/knowledge/replays.md], claims: [K-map-comet-catcher-remake-1-8-higher-os-differences]}
status: live
-->
## What strong players do: the pool

The pool is forty public duels on Comet Catcher Remake 1.8 in which both players were rated OS 40 or above (the
players OS 40 to 54, median 43), played between August 9 and September 23, 2026, replayed through the engine and cut
into cards: every unit of both sides, every build order with its builder, every death with its killer where the
engine names one. The Armada sides are 41 of 30 duels (24 won), the Cortex sides 39 of 29 (16 won). Three players
hold most of the sides: Artur91 (OS 42-44) 22, Hellontoast (40-41) 13, thebluegecko (48.5) 9. A wider pool of 29
duels at OS 25 and above (2026-09-23) is cited where it adds something; its numbers are marked.

What the pool is not: it is one map, 1v1, tier 1 in all but four sides, twelve to fifteen minutes long at the median,
and it was played by people who were not playing us. It is the floor of strong play on this map and the measure a
Comet game of ours is read against; it says nothing about team games, other maps, or what a person does against our
particular shape. Quicksilver, Great Divide and the lobby maps have no cards (two Full Metal Plate duels at OS 40+ are
thebluegecko's, in his section of the people document). Every number below is a median unless it says otherwise, and
"p25 / median / p75" is the lower quartile, the median and the upper quartile of the sides. The full tables, by the
minute and by milestone, are in the two reference documents named in this block's sources; this document carries what
a player needs in play.

<!--meta
id: M-pool-opening
layer: meta
premises: [F-eco-energy, F-eco-build]
lessons: [L-opening-solars, L-opening-commander]
population: the Comet pool at OS 40+ (41 Armada sides, 39 Cortex)
map: Comet Catcher Remake 1.8
dates: 2026-10-17 (the reference set); 2026-09-24 (the assist study)
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md, run/replays/assist.py, run/replays/recheck.py], claims: [K-map-comet-catcher-remake-1-8-two-solars-before-the-plant, K-map-comet-catcher-remake-1-8-commander-assists-the-plant, K-map-comet-catcher-remake-1-8-two-openings, K-map-comet-catcher-remake-1-8-first-factory-kind, K-map-comet-catcher-remake-1-8-turret-at-0-27, K-open-comet-thebluegecko-rover-mass]}
status: live
-->
### The opening

**Armada opens on two solars and a vehicle plant at 0:34.** The commander's order in 22 of 41 sides is extractor,
extractor, solar, solar, vehicle plant (started 0:34, standing 0:53, its first unit at 0:54); the plant is a vehicle
plant in 40 of 41. Two solars stand before the plant in the median side (2 / 2 / 2); three in 4 of 41, four in none.
The third and fourth solars come after the plant, by 2:00 in most sides, five by 3:00 and eight by 5:00. The three-solar
line with the plant at 0:58 is thebluegecko's (his section in the people document), and it does not separate winners
from losers here: the count of solars by 2:00 is the same for both.

**Cortex opens later and with a turret.** The most played order (11 of 39) is extractor, extractor, solar, light laser
turret at 0:25-0:28, a third extractor, solar, vehicle plant at 0:59; a bot lab after two extractors and one or two
solars in 7 of 39. The Cortex plant makes a constructor first (16 of 23 sides) and its first soldier at 1:46; the Armada
plant makes a Rover first (18 of 22) and its first constructor at 1:30. In the OS 25+ pool the later plant won 18 of 29
games to the earlier's 11, but the later plant was the higher-rated player's in most of those, so the clock itself is
not what won.

**The commander's work after the plant is the plant.** In the first four minutes the commander guards the plant for
50 s at the median (30 s or more in 45 of 80 sides), first at 1:35, stepping out for a solar or an extractor and back;
thebluegecko guards it 140 s of the first 240. The plant's units come 1.5 to 1.6 times faster than the plant alone makes
them, not the 3.0 the commander's build power would give, because the ceiling is metal: the store sits under 20 in 9%
of the seconds from 1:00 to 4:00. Winners assist less than losers (32 s against 93), bank less at 3:00 (72 against 107)
and stall more often: they spend harder. The players' own reasons are in the lessons (three solars for a fast
constructor, the commander on the lab as it comes up); the pool shows the first as a minority line and the second as
everyone's habit.

<!--meta
id: M-pool-first-units
layer: meta
premises: [F-unit-armfav, F-unit-armflash, F-unit-corak, F-unit-corgator]
population: the Comet pool at OS 40+
map: Comet Catcher Remake 1.8
dates: 2026-10-17
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md], claims: [K-map-comet-catcher-remake-1-8-raids-met-by-turrets]}
status: live
-->
### The first units and what the armies are made of

The factory makes 17 / 19 / 22 units by 4:00. Armada's first seven are scout cars and constructors: "two Rovers, two
constructors, three Rovers" (4 sides, all won), "a constructor, six Rovers" (4, three won); Blitzes come from about
2:00. Cortex's bot lab makes "two constructors, five Grunts" (10 sides), its vehicle plant "a constructor, two Rascals,
a constructor, three Rascals" (8). Five or more scout cars are out by 3:00 in 38 of 45 vehicle-plant sides, winners and
losers alike: the scout cars are the raiders and the raid answer of the first five minutes.

The armies are one unit type for ten minutes. Armada: Rovers 55% and Blitzes 42% of the army's metal at 2:00, Blitzes
67% at 4:00 and 84-88% from 6:00 to 10:00, Stouts 38% only at 14:00. Cortex: Grunts 41% and Incisors 39% at 4:00,
Incisors 67-76% from 6:00 to 10:00, Brutes 37% at 14:00 and 63% at 18:00, with Levelers a tenth and Pounders never
above 5%. Army metal standing: Armada 578 / 1,080 / 1,496 / 2,530 / 3,570 / 4,718 at 4, 6, 8, 10, 12 and 14 minutes
(9, 12, 17, 23, 35 and 40 soldiers); Cortex 510 / 1,252 / 2,066 / 2,752 / 4,780 / 7,415.

<!--meta
id: M-pool-pressure
layer: meta
premises: [F-map-grid, F-unit-armfav, F-unit-corfav]
lessons: [L-raid-first-handful, L-raid-naked-mexes, L-raid-keep-moving]
population: the Comet pool at OS 40+ (80 sides)
map: Comet Catcher Remake 1.8
dates: 2026-10-03 (the pressure fields), 2026-10-17
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md, run/pro_baseline.py], claims: [K-pro-the-first-soldiers-are-in-his-half-by-two-minutes, K-map-comet-catcher-remake-1-8-first-extractor-lost, K-map-comet-catcher-remake-1-8-raids-met-by-turrets]}
status: live
-->
### The first soldiers cross the map, and both sides lose extractors all game

**Every side of the pool had a soldier in the enemy's half within three minutes.** Armada's first soldier crosses the
middle at 1:22 / 1:40 / 1:49, a Rover in 37 of 41 sides, 26 to 44 s after it was made; it is within 1,500 of his start
at 1:54 and kills a building of his at 2:18 / 2:48 / 4:13 (an extractor at 3:04). Cortex, whose Rascals come after a
constructor, crosses at 2:16 / 2:48 / 3:23 and kills at 3:33 / 4:35 / 5:35. All but one of 80 sides killed a building
of his. Winners cross 40 s earlier than losers and have 78 of their 280 army metal in his half at 3:00 against the
losers' 43 of 334. The players' reason, relayed by the user: the space of what the opponent may be doing is too broad
to leave unpressed, and pressure is what stops an extremely greedy opening (the lessons on raids).

**Extractors are lost at 1.5 to 2 a game-minute, by winners too.** The first extractor falls at 4:21 for Armada (2:36 /
4:21 / 5:17) and 3:55 for Cortex; every side of the OS 25+ pool lost one. Over the game a side loses 1.54 (Armada) or
2.09 (Cortex) a minute: the winners 0.99 and 0.90, the losers 2.52 and 2.69. The killer, where the card names it, is a
scout car in 21 of 48 sides, an Incisor in 10, a Blitz in 8, a Grunt in 6. A side re-takes 7.5 spots a game and rebuilds
two before 8:00. The raids are met by turrets and scout cars, not by the army: light turrets by 8:00 are 7 / 10 / 13
for Armada (winners 11, losers 9) and 5 / 7 / 8 for Cortex (8 against 5), and the fights of the first five minutes fall
half in each side's own half, which is raiding both ways, not a battle.

<!--meta
id: M-pool-expansion
layer: meta
premises: [F-map-metal, F-eco-extractors]
lessons: [L-mex-half-the-map, L-mex-game-over]
population: the Comet pool at OS 40+; the spot orders from the OS 25+ pool (29 duels)
map: Comet Catcher Remake 1.8
dates: 2026-10-17; 2026-09-23 (the spot orders)
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md], claims: [K-map-comet-catcher-remake-1-8-extractor-curve, K-map-comet-catcher-remake-1-8-home-spots-then-flanks, K-map-comet-catcher-remake-1-8-a4-expansion, K-map-comet-catcher-remake-1-8-b5-winners-go-south-east, K-map-comet-catcher-remake-1-8-h5-winners-go-south, K-map-comet-catcher-remake-1-8-g4-and-h1-expansion, K-map-comet-catcher-remake-1-8-decided-by-the-8-to-12-gap]}
status: live
-->
### Expansion: the curve, the order of the spots, and the gap that decides

**The curve.** Extractors standing, Armada, p25 / median / p75 with the winners' and losers' medians: 4:00 6 / 7 / 8
(8 and 6); 6:00 8 / 11 / 14 (12 and 9); 8:00 11 / 15 / 18 (17 and 13); 10:00 14 / 17 / 20 (19 and 14); 12:00 19 / 22 / 27
(27 and 18); 14:00 16 / 27 / 30 (30 and 16). Cortex runs one higher through 10:00 (8, 12, 16, 21 at 4, 6, 8, 10) and
reaches 24 at 12:00. Metal income is 2.0 to 2.3 a second an extractor over minutes 3 to 8 (the sheet's 2.30 for a spot
here), so a side at 17 extractors at 10:00 has about 40 a second. Constructors: the first at 1:30, 3 standing at 4:00,
5 at 8:00, 8 at 12:00 (4 / 5 / 7 made by 8:00); nobody opens with a constructor count, they come one in every four or
five units.

**Where the spots are.** By 4:00, 80% of the spots a side has taken lie in the nearest quarter of the way to his start
and 1% past the middle; by 8:00, 60% in the nearest quarter, 27% in the next, 9% within sight of the middle and 4%
nearer him; by 12:00, 51 / 32 / 12 / 5. The commander takes the three home spots (A4: spot_30, spot_28, then spot_36 at
0:45; B5: spot_50, spot_45, spot_43 by 1:17; H5: spot_48, spot_51, spot_42 by 0:48; G4: spot_34, spot_29, spot_35 by 1:25)
and often the next pair toward the middle (A4 to spot_45 and spot_50 at 2:06-2:33; H5 to spot_34 and spot_29 at
2:12-2:36); constructors take the flank strips. From A4 the north strip goes spot_19 (B3) 2:51, spot_22 3:50, spot_14 (A2)
4:10, spot_7 (A1) 4:46, spot_2 (B1) 5:12, spot_9 5:57, spot_4 (C1) 6:14, winners and losers on the same clock. From B5
the winners went south-east (spot_52 C5 2:47, spot_61 C6 3:17, spot_63 C7 3:32, spot_64 D7 3:44, spot_59 D6 4:44) and
took the back corner (spot_54, spot_62 at A6-A7) only at 5:56-6:19; the losers took the back corner first. From H5 the
winners took the south strip (spot_60 G6 2:06, spot_65 H7 3:14, spot_72 H8 4:00, spot_57 G6 4:09) and the losers the
north (spot_25 H3 at 4:42); four of the seven H5 wins are one player, so this may be him rather than the map.

**The gap opens between 8:00 and 12:00.** Winner minus loser in extractors: 0 at 2:00, 1 at 4:00, 2 at 6:00, 3 at
8:00, 6.5 at 10:00, 12 at 12:00 (OS 25+ pool, 18 games reaching 12:00); from 8:00 to 12:00 the winners added 11
extractors and the losers 1, and the winners had 25 or more at 12:00 in 14 of 18. The card cannot tell whether the
losers' flat curve is extractors killed or never built; the loss rates above say much of it is killed.

<!--meta
id: M-pool-energy-and-power
layer: meta
premises: [F-eco-energy, F-eco-build, F-eco-store]
lessons: [L-energy-nanos, L-energy-converters, L-energy-priority]
population: the Comet pool at OS 40+; the energy-storage and converter counts from the whole-game records of the 80 sides; the priority toggles from 75 duels at OS 25+
map: Comet Catcher Remake 1.8
dates: 2026-10-17; 2026-10-19 (storage); 2026-10-13 (priority)
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md, docs/studies/2026-10-13-builder-priority.md, run/replays/priority.py], claims: [K-pro-comet-first-construction-turret-before-the-second-factory, K-eco-winner-pulls-away-8-to-12, K-eco-energy-storage-is-the-cheap-buffer, K-econ-commander-low-while-assisting, K-econ-expanding-constructor-high, K-open-comet-radar-nano-solars-habits, K-map-comet-catcher-remake-1-8-second-factory]}
status: live
-->
### Energy, build power, the bank

**Solars only, and more of them than we build.** By 8:00 a side has 12 / 14 / 17 solars (Armada; winners 16, losers
13) or 14 / 16 / 18 (Cortex); no side built a wind generator before 8:00 (the map's wind is 1 to 4). Energy is tight
on both sides: the store was under 50 at some minute between 3:00 and 8:00 in 40 of 58 sides (OS 25+), and the winners
buy more of it (energy income 400 against 330 at 8:00). Radars: 1 / 2 / 2 by 8:00 (Cortex 2 / 3 / 4), the first at about
3:00. An energy storage: 23 of 80 sides, the first at 8:51 / 9:36 / 12:27; a metal storage 5 of 80; converters 10 of 80
(three to twenty-four of them); a fusion none, the duels ending at 12:56 at the median.

**The first construction turret goes on the plant before the second factory.** Armada: begun at 5:24 / 5:56 / 6:31 in
30 of 41 sides (winners 5:45, losers 6:25) with 9 / 11 / 13 extractors, 20 / 25 / 28 metal income, three constructors and
358 stored. Cortex: 4:45 / 5:06 / 6:10 in 31 of 39. Of the sides that built both, 23 of 24 (Armada) and 24 of 27 (Cortex)
put the turret first. The second factory follows at 7:12 / 8:11 / 8:56 (Armada, 30 of 41: a vehicle plant 16, a bot lab
12, an air plant 2) and 6:14 / 8:16 / 9:36 (Cortex, 31 of 39: a vehicle plant 22, a bot lab 6, an air plant 3), with
14 / 16 / 19 extractors and 898 stored. The higher-rated player started the second factory earlier in 17 of 29 games.

**The bank peaks at 8:00 and the winners spend it.** Metal stored, Armada median: 52 at 3:00, 260 at 5:00, 537 at 7:00,
837 at 8:00 (p75 1,431), 356 at 10:00 after the second factory, 410 at 12:00. At 10:00 7 of 22 winners and 15 of 22
losers held 500 or more (OS 25+); the income gap is small to 8:00 (winner minus loser 6.9 a second) and opens after
(13.6 at 10:00, 21.2 at 12:00).

**Builder priority.** Fourteen of 41 players in 75 duels set the commander to low priority while it assists the plant
or builds a solar (52 toggles, 42 of them before 5:00, 21 in the first minute, the metal store under a tenth in 23) and
high again when it takes a build of its own; thebluegecko did it in 10 of his 12 toggles. Labs and construction
turrets are left on the widget's low from birth by everyone (0 of 172 hand toggles). A second school (5 of 41 players,
one of them 65 of the 81 toggles) has every constructor born low and flips the one carrying the next extractor or
turret to high as it gets the order.

<!--meta
id: M-pool-fights
layer: meta
premises: [F-fight-range, F-fight-sight, F-map-grid]
lessons: [L-lead-together, L-fight-sight]
population: the Comet pool at OS 40+ (693 fights; the shapes study)
map: Comet Catcher Remake 1.8
dates: 2026-09-24 (shapes), 2026-09-23 (fight cells), 2026-10-03 (army in his half)
ground: card (geometry only: the records cannot show shots or friendly fire for the pros)
sources: {code: [docs/studies/2026-09-24-pro-fight-shapes.md, run/replays/shapes.py, docs/briefs/reference/comet-catcher-remake-1.8-arm.md], claims: [K-form-pros-order-one-point-per-unit, K-form-pros-stand-two-hulls-apart, K-form-friend-on-the-line-twice-ours, K-form-pros-fight-with-pure-bodies, K-form-a-line-at-contact-trades-better, K-form-pros-raiders-fight-at-reach, K-map-comet-catcher-remake-1-8-fights-on-rows-4-5]}
status: live
-->
### How they fight

**Small bodies, spaced, of one type.** The median body at contact is six soldiers (ours in the three games measured
beside them, thirteen); the largest body of one type seen was a dozen raiders or line tanks and eight Stouts; bodies
over thirty are late and mixed. Soldiers stand about two hulls from the nearest friend (Blitz 67 elmos, Stout 70; ours
25 and 40), because the players give a group one point per unit on a line about 64 apart (70% of the last orders
before contact; we give one shared point). A friend stands on a unit's line of fire for 22% of the units in reach at
contact (ours 43%). Raiders and line units are rarely in one body (10% of cores; 33% when the side owned both), and
short-range units hold the edge of their reach (0.94 of it; ours close to 0.86). A force that meets the enemy as a line
across trades about five points better than a blob, and the eventual winners' lines traded 60% to the losers' 40%.

**Where the fighting is.** Rows 4 and 5, the middle band between the starts, hold 68% of the deaths in the eight
costliest cell-minutes of each game (G5, B5, B4, D5 the costliest cells). The first five minutes fall half in each side's
own half; from minute 6 the fighting moves into the loser's half (479 of 697 deaths in minutes 6-9, 1,390 of 1,645 from
14 on) and the loser loses two to three and a half times what the winner does. Army metal in his half: the winners
carry 376 of their 1,070 there at 6:00 and 1,540 of 4,444 at 12:00; the losers a fifth or less (172 of 1,080; 361 of
2,786).

<!--meta
id: M-pool-closing
layer: meta
premises: [F-fight-damage, M-pool-expansion]
lessons: [L-lead-use-it, L-lead-together]
population: the Comet pool at OS 40+ (40 games); endings from the OS 25+ pool (29)
map: Comet Catcher Remake 1.8
dates: 2026-09-27 (the close), 2026-09-23 (endings)
ground: card
sources: {code: [docs/studies/2026-09-27-review-skill.md, docs/briefs/reference/comet-catcher-remake-1.8-arm.md], claims: [K-map-comet-catcher-remake-1-8-winners-close-within-four-minutes-of-twice-the-army, K-map-comet-catcher-remake-1-8-endings, K-map-comet-catcher-remake-1-8-higher-os-differences]}
status: live
-->
### How a duel ends

Duels last 12:05 at the median for the Armada sides (9:04 to 15:20) and 14:52 for the Cortex sides; 7 of 29 under
ten minutes, 10 of 29 over fifteen, the longest 21:31. The eventual winner first holds twice the loser's army metal at
minute 7 (median; 6.5 to 13; in 27 of 40 games at all) and the game ends 3.4 minutes later (1.2 to 5.4; over six
minutes in 5 of 27). The extractor lead reaches five at minute 7. At the end the winner holds 27 extractors to 14.5
and 1.8 times the army; the loser's commander and every factory are among the winner's kills in 29 of 29, and the
last minute's fights are in or beside the loser's start cell in 28 of 29. Three of 29 winners closed while behind or
level on extractors, each with the larger army. The higher-rated player won 22 of 29 in the OS 25+ pool but only 18 of
33 (55%) at 40+: the habits above are the pool's, and at the top the result is close to even.

<!--meta
id: M-pool-tier2-and-wrecks
layer: meta
premises: [F-eco-reclaim, F-unit-armavp, F-unit-coravp]
lessons: [L-tier2-go, L-reclaim-field, L-reclaim-map]
population: the Comet pool at OS 40+; wrecks from the OS 25+ pool's records
map: Comet Catcher Remake 1.8
dates: 2026-10-17; 2026-09-23 (wrecks)
ground: card
sources: {code: [docs/briefs/reference/comet-catcher-remake-1.8-arm.md, docs/briefs/reference/comet-catcher-remake-1.8-cor.md], claims: [K-map-comet-catcher-remake-1-8-tier-2-is-rare, K-eco-comet-pros-rez-bots]}
status: live
-->
### Tier 2 is rare, wrecks are worked at home

An advanced factory was started by 1 of 41 Armada sides (at 20:28) and 3 of 39 Cortex sides (15:07 / 17:02 / 18:36); of
the sides whose game passed 15:00, 1 of 12 and 3 of 18. Never before 12:51 in any pool. The Cortex sides that did had
32 / 33 / 40 extractors, 116 / 119 / 262 metal income and 8,500 to 11,900 of army when they placed it, and 1 of 3 won. The
duels are decided on tier 1; the people's own rule for when to go is in the lessons.

Wrecks: resurrection bots by 8:00 are 0 / 0 / 0 (Armada) and 0 / 0 / 1 (Cortex). In the OS 25+ pool 24 of 58 sides built
them, a median of ten a side, the first between minutes 4 and 10, and they worked wrecks on their own ground: a bot
stood within 400 of a death in the next 90 s after 22 of 2,730 deaths on the winners' side. Income above the
extractors' yield (reclaim and resurrection) in 12% of the winners' minutes against 7% of the losers'.

<!--meta
id: M-pool-limits
layer: meta
population: the pool's own limits
map: Comet Catcher Remake 1.8
dates: 2026-10-22
ground: card
sources: {code: [docs/knowledge/replays.md, run/replays/card.py, run/data/replays/manifest.jsonl]}
status: live
-->
### What the pool cannot say

The cards carry no unit positions over time, so "held back" against "chasing" cannot be read, and a raid's answer is
inferred from turret counts and fight cells. The killer of the first extractor is unnamed on 10 of 58 sides. A resign's
self-destruct is counted among the winner's kills, so a hunted commander and a resign look alike at the end. Tier 2,
air and the sea are nearly absent (no side opened with an aircraft plant; four advanced factories in 80 sides). The
numbers are one game version (test-31251 to 31383; we play test-31677) and one map; the map sheet carries the walks and
spot counts that let a number be carried to another map, and the derivations say which numbers travel. Team games with
people are not in the pool at all: what the people we play do in them is the people document.
