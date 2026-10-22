<!--meta
id: M-format-intro
layer: meta
population: the three formats we play or aim at: duels, team games of two to four a side, and 8v8
dates: 2026-10-22 (the user's ruling that the formats are different games; the forty SMRTBARb cards)
ground: mixed (each block says)
sources: {code: [docs/briefs/meta/pool.md, docs/briefs/meta/people.md, docs/briefs/meta/barb.md, docs/studies/2026-10-22-episode-sweep/games.jsonl, run/replays/team_set.py, run/data/replays/manifest.jsonl]}
status: live
-->
## The formats: what is common and what each changes

The rules, the prices, the maps and the units are the same in every format; what changes with the number of players
is the ground each player holds, the number of fronts, how long a game lasts, whether an economy can be pooled, and
what the opponents are tuned for. The user's ruling (2026-10-22): 8v8 is its own developed meta, the popular category
is eight people against eight AIs on a map like All That Glitters, our 1v1 proficiencies do not carry into it, and
BARb and Apex are probably tuned with that mode in view. The evidence base differs by format and this document says
which: duels have the pool (40 carded games at OS 40+); team games of two to four a side have our 57 lobby games with
people (12 won; the people document); 8v8 has forty carded public games of eight people against eight SMRTBARb on All
That Glitters (2026-10-04 to 10-21), our four games of eight seats against eight BARb, the lobby 5v5, and the pool's
51 games against eight BARb and 81 between people replaying (2026-10-22).

**Common to every format (the facts and the pool).** The opening is bound by energy and then by metal; the first
units cross to the opponent because nothing else tells you what he is doing (M-pool-pressure); extractors are lost all
game by everyone; tier 2 costs what it costs and the sprint's arithmetic is the same (F-eco-build); sight is the same
coarse grid; a commander's death ends a side only when it was its last. What a derivation may carry across formats is
the arithmetic; what it may not carry is any clock or count read from one format's games.

<!--meta
id: M-format-1v1
layer: meta
premises: [M-pool-closing, M-pool-expansion, M-pool-pressure, M-pool-tier2-and-wrecks]
population: duels (the pool at OS 40+; our arena 1v1s against BARb and Apex; the 1v1 lobby games)
map: Comet Catcher Remake 1.8 mostly
dates: 2026-10-22
ground: card (the pool); truth (the arena)
sources: {code: [docs/briefs/meta/pool.md, docs/briefs/meta/barb.md]}
status: live
-->
### Duels

Twelve to fifteen minutes, tier 1, one front and the whole map to expand into: the duel is decided by the extractor
gap of 8:00 to 12:00 and the army that comes with it, the winner holds twice the army at minute 7 and ends it three
and a half minutes later, and tier 2 comes in four of eighty sides. Every spot on the map is somebody's to take and
nobody covers a flank for you, so raids both ways run from 2:00 to the end and both sides lose extractors every
minute. The commander is a fighting piece (its D-gun answers a Pawn party; a snipe ends the game). Our arena record
against BARb hard_aggressive at +0 is twelve wins in fourteen on this shape, and the bonus ladder says the duel is
even at about +15 to +20 for it.

<!--meta
id: M-format-team-small
layer: meta
premises: [F-eco-build, F-eco-multipliers, M-gecko-team, M-barb-team]
lessons: [L-tier2-go, L-tier2-tier1-under-tier2, L-raid-naked-mexes]
population: team games of two to four a side: our lobby games against one to three people with two to four seats (2026-09-26 to 10-20), the Altored 3v3s against Apex, human-14 beside the user
dates: 2026-10-22
ground: sightings, model, record (the people document's blocks); truth (the Apex 3v3s)
sources: {code: [docs/briefs/meta/people.md, docs/briefs/meta/apex.md, docs/experiments.md], claims: [K-t2-people-share-advanced-constructors-across-factions, K-t2-feeding-is-production-not-surplus, K-opp-people-nuke-late-or-never]}
status: live
-->
### Two to four a side

Longer (15 to 40 minutes in our lobby games), and tier 2 comes early for the side that can afford one sprint: a person's
advanced lab at 6:56 (Cape Violet), 7:08 (Center Command), by 9:00 (Azurite) and 11:18 to 17:33 against our three
seats on Comet, with the whole side's build power finishing the frames and the advanced constructors passed between
seats whatever their faction (M-gecko-team). Each seat holds a strip or a row and the seams between seats are where the
people put forward posts and walk their commanders (D4 on Comet, E2 on Center Command, E7 and F7 on SD-129 and
Gasbag); the raids of the first minutes go into every seat at once and the weakest seat is the one they finish
first. The sea is a seat of its own on water maps (u6bkep on Azurite, thebluegecko on SailAway and Coast To Coast). The
people we play have never built a silo in these games; an anti-nuke before 25:00 against them buys nothing. Our own
shape in these games, read by the sweep and by the user: a sprint seat starved by its feeders, first soldiers sent to
die in his half with nothing left in front, bodies that reach the seam one seat at a time, and no army of ours in his
half after 8:00. Against three Apex seats on Altored the 3v3 was won at +0 to +50 and contested at +60 (M-apex-ladder).

<!--meta
id: M-format-8v8
layer: meta
premises: [F-map-grid, F-eco-build, F-eco-conversion, M-barb-team]
population: 8v8 on All That Glitters: forty public games of eight people against eight SMRTBARb (2026-10-04 to 10-21, replayed and carded 2026-10-22; the people's sides of mean OS 15.7 / 18.6 / 20.7 with the highest seat 33.6 median, the AIs at +30 to +75 resource bonus by the start script's handicap, +50 in 18 games; 33 won by the AIs, 4 by the people at 24:23, 24:49, 56:43 and 58:55, 3 stopped by the host); our eight seats against eight BARb hard_aggressive on the same map (four games, 2026-10-04 to 10-10); the lobby 5v5 on Altored (2026-10-18); the pool's 51 games against eight BARb and 81 between people replaying (2026-10-22)
map: All That Glitters v2.2.3 (6144 by 10240, 152 spots, two rows of eight starts across the short axis, the lanes north to south, about nine and a half spots a seat)
dates: 2026-10-22
ground: card (the forty SMRTBARb games: `run/replays/team_set.py --map "All That Glitters v2.2.3" --sides 8v8 --ais SMRTBARb`, quartiles over the 33 lost sides unless said); truth (our BARb games); sightings and model (the 5v5)
sources: {code: [run/replays/team_set.py, run/replays/card.py, run/data/replays/manifest.jsonl, docs/briefs/meta/barb.md, docs/briefs/meta/people.md, run/data/demos-client], claims: [K-team-eight-seats-lose-to-eight-barbs-on-one-curve]}
status: live
-->
### 8v8

**The shape (card, forty games).** Sixteen starts in two rows across the short axis, eight lanes of about 750 elmos
running the long way, nine or ten spots a seat. The games run 20:37 / 25:05 / 32:27 (p25 / median / p75); the
people's four wins were two at 24 minutes and two at 57 to 59. A side's seats are ranked by lane from the west below
(lane 1 the west-most start), and the people's roles sit on fixed lanes: the lobby's start positions carry the roles.

**The people's side: three roles by convention (card; their own words).** One seat opens on an aircraft plant in 30
of 40 sides, on lane 3 in 28 of them, at 0:49 / 0:57 / 1:35; one seat techs first in every side, on lane 5 in 31 of
40, with an advanced bot lab at 3:46 / 4:14 / 4:59 (the won sides 4:28 / 4:46 / 5:02); the other six hold lanes. The
median side has 3 seats with an advanced plant by 10:00, 5 by 15:00, 7 by 20:00. The roles are agreed in the lobby and
the first minute ("tom you good tech? ill air", "sure, tech good!", "Ill take Tech", "i start here for air", "then you
air", "I can switch out of tech if someone really wants it", "whats my lane on this one?").

- **The air seat (lane 3)** puts the aircraft plant on three extractors with winds and a converter before it
  (1:35 in the Legion sample, tier-1 fighters from 1:58, a vehicle plant for scouts at 4:18, the advanced aircraft
  plant at 9:33 / 13:23 / 15:37). Income 21 at 10:00 and 64 at 20:00; army 904 at 10:00 and 3,045 at 20:00; it is the
  side's second crosser (its first soldier in the AIs' half 5:11 / 8:27 / 13:10). What 23 of them built over their
  games: legfig 2,123, legscout 1,356, corvamp 916, legwin 561, corveng 464, leghades 410, armwin 366, armhawk 364,
  legafigdef 363, corwin 336. It is also the side's main giver: lane 3 to lane 5 is the largest metal flow (36,150
  summed over the 33 lost sides; 18,480 over the 4 won), and the Legion sample gave 5,252 by hand in five gifts from
  0:47 and 57 leghades to a front seat. Its own words: "gunna TRY to hybrid spam/t1 figs while eco scaling", "don't
  go t2 before you help your team", "i'm making air and ground I aint idle".
- **The tech seat (lane 5)** takes six extractors by 2:44, winds by the dozen (63 in one sample), construction turrets
  from 2:04, converters from 2:39, and the advanced bot lab at 3:45 / 4:15 / 5:24 (lane 5's quartiles) by its fourth
  to sixth constructor; the first advanced construction bot 48 to 85 s after the lab, mohos on its own spots from
  5:39, and then one advanced constructor to each front seat (the sample gave one to four teammates; "t2 shop open"
  at 5:44, "all front should have t2 now ... urs are comin" at 9:55, "Asss tech 2 const ... Xavierhusky tech 2 const
  ... Slickers tech 2 const" at 13:05 to 14:23, "who needs T2?" at 14:20). It has no army to 12:00 to 18:00 (172 at
  10:00 median; 3,750 at 20:00; the won sides 187 and 4,887), the highest income of the lanes at 20:00 (26 at 10:00,
  72 at 20:00), never crosses (its first soldier in the AIs' half 17:13 median, never in 24 of 40), and is where the
  side's metal goes: the top receiver in 13 of 33 lost sides, 2,965 metal received median and 10,562 at p75, with
  the energy by the hundred thousand (101,333 and 2.65 million in the two samples). The front asks it for tier 2
  ("5 min T2 plz", "T2 plz" 7:02, "share T2" 14:04, "T2 PLEASEEEEE" 13:12, "can someone give me t2?" 12:17, "need tech"
  11:49, "not even got t2 con yet" 14:07) and later for units ("Ill send units", "send M if you want t3 units", "i
  make t3", "Tech 3 for eco"). What 33 of them built: armflea 2,388, armwin 1,289, armmakr 568, corwin 502, corak
  479, legwin 394, armnanotc 390, cornanotc 379, leggob 340, armfido 151; the won sides' tech seats armpw 3,657 and
  armrectr 455 over four seats (2,803 armpw from one seat, from seven more bot labs from 16:11).
- **The front seats (lanes 1, 2, 4, 6, 7, 8)** open bot labs two times in three (bots 22 and vehicles 11 on lane 1,
  29 and 4 on lane 2, 24 and 9 on 4, 18 and 15 on 6, 24 and 9 on 7, 21 and 12 on 8), put up their advanced plant at
  11:03 to 12:59 (the medians by lane; 26 to 29 of 33 sides), make 16 to 25 income at 10:00 and 34 to 64 at 20:00,
  and hold 980 to 1,880 of army at 10:00 and 940 to 4,795 at 20:00, the largest on lane 8, then lanes 7 and 1. What
  176 of them built: armflea 5,365, armpw 3,641, legdrone 3,391, corak 2,979, armwin 2,953, corwin 2,864, legwin 2,553,
  leggob 2,367, legscout 1,435, cornecro 1,129. The front's first soldier in the AIs' half: lane 8 at 4:40 / 6:52 /
  9:35, lane 1 at 6:21 / 8:49 / 11:15, lane 6 at 7:47 / 10:15 / 12:59, lane 7 at 6:48 / 11:56 / 15:27, lane 2 never in
  30 of 40, lane 4 never in 26 of 40. The side kills its first AI building at 3:32 / 3:53 / 4:14 and its first AI
  extractor at 5:16 / 5:43 / 6:29. What the front says of itself: "front is bring your com and build LLTs", "send
  your com or spec until you know how to play front", "im 3 gummies in front hodl ty", "why am i covering such a huge
  lane on my own", "i cant cover whole front", "I cant take this wide of the front guys", "timmy get in lane", "yo my
  front needs to wake up", and of the seats that tech instead: "too many of you tried to tech from the front", "I went
  t2 because no one else went in time for team", "30 min t2 is not good xd", "cant just eco, the bots push early and
  hard".

**The transfers (card: the player list's gift lines, the engine's figure, the units handed over).** Gifts by hand
per lost side 821 / 1,965 / 9,072 metal and 113 / 5,003 / 23,629 energy; per won side 7,439 / 10,462 / 18,236 metal
and 0 / 16,499 / 62,694 energy. The flows by lane, summed over the lost sides: 3 to 5 36,150, 8 to 5 20,410, 4 to 5
17,481, 2 to 5 11,633, every one into the tech seat; over the won sides 3 to 5 18,480, 5 to 7 7,729 (the tech seat
paying a front seat back), 3 to 4 7,069, 2 to 1 6,578. The engine's sent figure, overflow included, is 3,036 by 10:00
and 7,816 by 20:00 per lost side; the gift lines are the count of what was meant. Units handed over in forty games:
winds (legwin 448, armwin 315, corwin 113), construction turrets (legnanotc 193) and converters (armmakr 160) move
whole bases when a neighbour takes over a departed seat (six seats taken over in the lost games at 9:19 / 10:51 /
19:48, three in one won game at 45:55 to 51:17); armfido 328, armack 107, leghades 90 and corak 78 are the tech
seat's advanced constructors and the units sent to the front (lane 5 gives 10 units a game median, the most of any
lane; the front seats 0 to 2). "cant share unit for some reason" and "meant to share con" are the hand slipping.

**The side's curve and where the games are lost (card).** The lost sides against the winning AIs, at 4 / 8 / 10 /
20 / 24 minutes: extractors 51 / 63 / 65 / 55 / 52 against 46 / 65 / 70 / 83 / 81; metal income 113 / 167 / 224 /
497 / 590 against 150 / 259 / 351 / 1,056 / 1,546; energy income 1,517 / 3,306 / 4,254 / 15,391 / 22,320 against
2,137 / 5,057 / 6,293 / 33,130 / 47,500; constructors 25 / 40 / 45 / 60 / 73 against 27 / 51 / 66 / 190 / 258; army
metal 3,137 / 8,587 / 10,950 / 26,339 / 39,228 against 5,644 / 16,189 / 15,702 / 82,785 / 110,063; soldiers 42 / 72 /
85 / 110 / 149 against 52 / 113 / 110 / 261 / 359. The people are ahead on extractors to 6:00 and level at 8:00; behind
on income from 2:00 (66 against 89, the bonus), 1.5 times at 8:00, 2.1 at 20:00, 2.6 at 24:00; on army three times
behind at 2:00 (13 soldiers against 50: the AIs' first wave) and twice at 8:00, nearest at 10:00 to 12:00 (1.4 times, the
AIs' dip while their seats build advanced plants), twice at 14:00, 2.6 at 16:00, 3.1 at 20:00. The won sides had the
lost sides' extractors and income (51 / 68 / 72 at 4 / 8 / 10; 116 / 185 / 261) and more soldiers from 4:00 on: 50 /
82 / 106 / 122 / 155 / 166 at 4 / 6 / 8 / 10 / 12 / 14 against 42 / 57 / 72 / 85 / 103 / 111, army 13,018 at 10:00 and
23,450 at 14:00 against 10,950 and 15,455, five times the gifts by hand, and the AIs they beat were behind their own
curve at 10:00 (army 11,801 against 15,702, extractors 62 against 70, income 326 against 351): the people's units
showing in the AIs' figures. The four wins came at +30, +50, +50 and +55. What ends the lost games, in their words:
the AIs' air from 10:00 ("WHY IS IT AIR EVERYTIME" 10:15, "do we not have an air player ??" 15:56, "just lost my t2
to 5xbombers" 19:22, "then lost air by 10 fighters" 20:52, "the air raids are insane" 24:41, "i cant out air them lol"
28:52), the AIs' nuke ("what ai is this 12min nuke" 12:40, "13 min nuke" 14:28, "we need anti nuke in 12 min",
"first nuke in 15 min", "just rush anti nuke after t2", "building anti nuke now" 14:46, "out of anti nuke" 20:04; the
record: the AIs' tech seat begins a silo in 24 of 40 games, at 8:30 to 9:21 in seven of them, standing 10:03 to
12:08, at 13:12 to 17:23 in seven more and after 20:00 in ten; the people never build one, and begin their first
anti-nuke in 37 of 40 games at 9:05 to 19:38, 13:47 median, standing 15:07 median, on the tech seat's lane in 11 and
the air seat's in 6), and the front breaking from 13:00 ("front fell" 14:38, "front lost" 15:16, "front is falling" 23:14,
"need mid front now" 32:41), with the seats that leave taken over by a neighbour at 9:19 to 19:48.

**Our eight seats against eight BARb hard_aggressive (truth).** The same curve from the other side: level at 8:00 (70
extractors against 67, 98 soldiers against 92), his armed units 2.6 times ours by 16:00 while ours stood, his
extractors rising to 111 to 116 at 28:00 against ours falling from 70 to 13 to 20, every fight in our half from 8:00
in the three lost games, his Lazarus raising our dead where they fell (M-barb-team). The commander's own reading: the
2 to 2.5 times lead of 5:00 to 10:00 was never taken into his half, the bodies fell back to their own spots and fought
one at a time, and eight BARbs out-produced eight seats of about 15 income each. The fourth game, with Opus in the
seats, was undecided at 30:00 with ours advanced from 16:00.

**The lobby 5v5 (sightings, model).** Five people of mixed strength played as separate bases, walked their commanders
forward with turret nests, and the strongest of them turtled from 27:00 behind Bastions, shields and jammers on a
ridge our radar could not see over; we won when they left at 57:38. Their own words were about the format: "help
would be nice ... people just ignore" at 10:00, "soo late this is not eco game" at 32:27.

**What 8v8 changes, as far as these games show it.** Every seat has an opponent in front and a neighbour on each
side, so a lane holds or falls on its own seat and a neighbour's spare, and a seat that leaves is a lane a neighbour
must take over. The economy is the side's, pooled two ways: the people pool it by hand into one tech seat (2,000 to
10,000 metal a game and energy by the ten thousand), which pays the front back in advanced constructors from 6:00 to
14:00 and in units later; the AIs pool it by overflow (a full store spills to the side, 13,000 metal and 2.2 million
energy by 20:00) and by handing mohos and advanced constructors across lanes. Air is a seat of its own on each side,
and the side without a working air seat loses its tier 2 to bombers and its lanes to fighters. The game runs 20 to
32 minutes and turns at 10:00 to 12:00: the people's army is nearest the AIs' then and a third of it by 20:00, the
AIs' nuke lands from about 12:00, and the people's advanced plants come at 11:00 to 13:00 against the AIs' 9:00 to
12:00 with their tech seat's at 2:40. Nobody on the people's side crosses the middle before 7:00 while the AIs cross
every lane by 2:20 and kill the first extractor at 3:17: the duel's first handful (M-pool-pressure) is absent from
the people's 8v8 and present in the AIs'. The four sides that won made soldiers from 4:00 and fed their tech seat
five times the metal. None of this is derived yet; it is what forty cards and four truth files show.

<!--meta
id: M-format-smrtbarb
layer: meta
premises: [F-barb-how-it-decides, F-eco-conversion, F-eco-build]
population: SMRTBARb, 320 seats in forty public 8v8 games against people on All That Glitters, 2026-10-04 to 10-21 (ten game versions 31479 to 31691), at +30 to +75 resource bonus (+50 in 18, +55 in 9, +30 in 5, +60 in 3, +45 and +65 in 2 each, +75 in 1); won 33 of 37 decided games (lost at +30, +50, +50, +55), three stopped
map: All That Glitters v2.2.3
dates: 2026-10-22
ground: card (quartiles over the 33 winning sides unless said; the lanes from the west, the AIs' tech seat on the C8 start)
sources: {code: [run/replays/team_set.py, run/replays/card.py, run/data/replays/manifest.jsonl, run/data/demos-client, docs/briefs/meta/barb.md]}
status: live
-->
### SMRTBARb, the competition in that category

The user (2026-10-22): as far as he knows the state of the art, the bot we will meet head to head once we beat the
easier 8v8 bots; watch its play. It is a community build of BARb (its seats are named SMRTBARbstable in the start
script), so BARb's rules are the starting point (F-barb-how-it-decides) and its differences are what to read. Every
public game has a resource bonus for the AIs (the lobby's handicap, in the start script; +50 in 18 of 40), so its
income below is what its economy makes times 1.3 to 1.75, and it won 33 of 37 decided games against people of mean OS
15.7 / 18.6 / 20.7. Its factions are random; its roles are by seat.

- **The same eight seats every game.** The seat on lane 4 (the C8 start) is the tech seat in 38 of 40 sides, always on
  a bot lab; the seat on lane 5 (D8) is the air seat in 35 of 40 (lane 6 in 4, lane 3 in 1); the other six are lane
  seats on vehicle plants more often than bot labs (vehicles 22 of 33 on lanes 1 and 6, 23 on lane 7, 28 on lane 2, 20
  on lane 8, 19 on lane 3).
- **The lane seats raid every lane by 2:20.** A lab at 0:28 to 0:48 (the two games read by hand), and the first
  soldier in the people's half at 1:53 to 1:58 / 2:04 to 2:22 / 2:23 to 3:17 on every lane seat (p25 / median / p75
  over 40 sides); the side has 50 soldiers and 2,112 of army at 2:00 (the people 13 and 664), kills its first
  building of theirs at 2:09 / 2:35 / 3:09 and its first extractor at 2:46 / 3:17 / 3:46. Each lane seat's advanced
  plant comes at 8:49 (lane 2), 9:00 (3), 9:29 (6), 10:30 (1), 11:20 (8) and 12:20 (7), the median side having 5 seats
  with one by 10:00 and all 8 by 15:00. Their armies at 20:00: 12,662 / 13,980 / 13,880 on lanes 1 / 2 / 3, 9,510 on
  6, 8,734 on 8, 7,710 on 7, the three largest 0.59 of the side's (the people's 0.72): the army is spread over the
  lanes. Their income 25 to 56 at 10:00 and 84 to 151 at 20:00, lanes 2, 3 and 6 the richest. What 166 of them built:
  leggob 6,582, armpw 5,256, leghades 5,065, armflash 4,798, corak 3,009, corgator 2,517, legdrone 2,429, legnanotc
  1,604, armstump 1,529, armnanotc 1,468: tier 1 by the thousand to the end of the game.
- **The tech seat (lane 4) is a pure economy seat for sixteen minutes.** Its opening: three extractors by 0:21, the
  bot lab at 0:29 to 0:32, two constructors by 1:12, winds, the sixth extractor by 2:27 to 2:47, a converter or a
  storage at 2:01 to 2:24, and the advanced bot lab at 2:31 / 2:40 / 2:54 (lane 4's quartiles over the 33 winning
  sides) by its third or fourth constructor on six extractors; the first advanced constructor at 3:56 to 3:59 in the
  two Cortex games read, mohos on its own six spots from 4:18 (all six by 8:35 to 10:55),
  construction turrets from 2:56 to 4:29 (69 to 179 over a game), converters (171 in the +65 game), advanced solars
  and fusions. It hands one advanced constructor to each of about five lane seats (the three games read gave to five,
  five and three teammates; 253 advanced constructors changed hands on the AI sides in forty games, and lane 4 gives
  9 units a game median and 28 at most, the most of the lanes): the "T2 con share" the people ask of their own tech seat, done
  by 5:00. It has no army at all to 16:00 to 18:00 (0 / 0 / 0 at 10:00; 810 / 1,242 / 1,840 at 20:00), never crosses
  (its first soldier in their half 12:27 / 16:01 / 18:34, never in 6 of 40), holds six extractors all game in the
  three games read, and makes
  the side's highest income from 10:00 (55 / 63 / 72 at 10:00, 172 / 193 / 301 at 20:00, 714 and 742 at 20:00 in the
  two +65 and +60 games). Its store overflows to the side from the first minute: 33,951 metal median over a game
  (1,980 at p25, 101,415 at p75, 229,000 in one) and 0.8 to 1.8 million energy, and the overflow lands on the lane
  seats with the biggest armies (the top receiver lane 1 in 7 of 33 sides, lanes 8 and 6 in 5 each, lane 2 in 4,
  taking 0.37 of what the side received). From 11:00 to 15:00 it adds an aircraft plant and then an advanced one, a
  silo in 24 of 40 games (on lane 4 in 23, the air seat in one; begun at 8:30 to 9:21 in seven games and standing
  10:03 to 12:08, the people's "12 min nuke"; at 13:12 to 17:23 in seven more; after 20:00 in ten) and an anti-nuke
  in 34 of 40 at 13:12 to 20:56 on any seat, tier-1 bot labs by the half-dozen from 18:33 (528 corak in the +65 game, from
  20:00 on) and gantries from 21:16 (corkorg by the handful by 28:00).
- **The air seat (lane 5) builds tier-2 fighters by the sixty.** An aircraft plant at 1:15 / 1:20 / 1:24 (the people's
  at 0:49 / 0:57 / 1:35), the advanced aircraft plant at 8:34 / 9:12 / 11:52, income 23 at 10:00 and 97 at 20:00, army
  1,200 at 10:00 and 7,286 at 20:00, its first soldier in their half at 3:18 / 6:34 / 8:08. What 33 of them built:
  armhawk 2,270, legvenator 1,885, legnanotc 775, armnanotc 706, armpnix 615, corvamp 547, legfig 508, armfig 491,
  legmos 438, legstronghold 403. The people's words for it: "like this one builds huge air squadrons from what I
  remember" 1:23, "WHY IS IT AIR EVERYTIME" 10:15, "just lost my t2 to 5xbombers" 19:22, "the air raids are insane"
  24:41, "i cant out air them lol ... I have t2 air ffs" 28:52; their answers: "so guyse make AA not just rely on your
  Air guy to make planes", "making like 10-20 t2 flak trucks and using them to respond to air raids is very
  effective", "make flak it's 30 minutes in".
- **Mohos handed across lanes.** The AI seats gave 1,216 advanced extractors to each other in forty games (armmoho
  512, legmoho 395, cormoho 309: about thirty a game), between neighbouring lanes (2 to 3 111, 3 to 6 99, 2 to 1 99,
  6 to 3 85, 8 to 7 77, 7 to 8 66, and the tech seat to lanes 1, 2 and 3), every seat giving 1 to 9 units a game
  median and up to 28; the reading is that a seat's constructor builds the moho on a neighbour's spot and hands it
  over, which the record shows only as the hand-over. Also handed over: armfboy 42, armfast 30, transports.
- **The side.** Medians over the 33 winning sides at 2 / 4 / 6 / 8 / 10 / 12 / 16 / 20 / 24 / 28 minutes, with the
  bonus: metal income 89 / 150 / 194 / 259 / 351 / 476 / 734 / 1,056 / 1,546 / 1,928; energy income 1,258 / 2,137 /
  4,076 / 5,057 / 6,293 / 8,373 / 19,002 / 33,130 / 47,500 / 69,935; extractors 24 / 46 / 58 / 65 / 70 / 76 / 77 / 83 /
  81 / 85; constructors 11 / 27 / 39 / 51 / 66 / 86 / 123 / 190 / 258 / 316; army metal 2,112 / 5,644 / 10,944 / 16,189
  / 15,702 / 19,595 / 49,234 / 82,785 / 110,063 / 142,154; soldiers 50 / 52 / 75 / 113 / 110 / 104 / 172 / 261 / 359 /
  442. The army dips at 10:00 (15,702 after 16,189 at 8:00; soldiers 110 and 104 at 10:00 and 12:00 after 113: the
  first wave spent in the people's lanes while the seats build their advanced plants) and triples from 12:00 to
  20:00; the energy overflow within the side is 230,529 by 8:00 and 2.24 million by 20:00, two to three times the
  people's. In its four lost games the side was behind its own curve at 10:00 (extractors
  62 against 70, army 11,801 against 15,702, income 326 against 351) and the people's soldiers were 1.4 times the
  lost sides' from 6:00: it loses to pressure before 10:00, not to anything later.

What this is not yet: a description of its rules (read its scripts the way BARb's were read, if they are published),
a game without a bonus (none in the pool), or a comparison with stock BARb at the same bonus on the same map (the
51 BARb games are replaying; `--ais BARb` on the same aggregate).

<!--meta
id: M-format-coop-many
layer: meta
population: two or three people against five to sixteen BARb: the user's own games in his client (Boulder Beach 2v16 hard and hard_aggressive 2026-09-22, 36 min; Evad River Confluence 3v12 hard_aggressive 10-03, 36 min; Ceasefire 2v6 hard_aggressive 09-29, 71 min; Altair Crossing 2v8 hard 10-09, 47 min; Pinewood Derby 2v5 hard_aggressive 10-11, 27 min; Azurite Shores 2v5 hard_aggressive with a distilled seat 10-18, 30 min)
dates: 2026-09-22 to 2026-10-18
ground: the demos' start scripts only (copied to run/data/demos-client; not yet replayed: three need game versions not on disk and four maps are not in run/data/maps)
sources: {code: [run/data/demos-client]}
status: live
-->
### Two or three people against many AIs

A fourth format the user plays himself: two or three people against five to sixteen BARb on a large map, thirty to
seventy minutes long. Six such demos are copied from his client and listed above; none is carded yet, and the game
versions and maps three of them need are not on disk. What they will show is how people hold and win against many
stock BARbs with a shared economy, the format closest to the 8v8 lobbies from the people's side.
