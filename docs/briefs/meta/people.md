<!--meta
id: M-people-intro
layer: meta
population: the people we have played in lobby games, 2026-09-21 to 2026-10-20 (57 games with people in the records)
dates: 2026-10-22
ground: mixed (each block says: card, sightings, person, model)
sources: {code: [docs/studies/2026-10-22-episode-sweep/games.jsonl, docs/studies/2026-10-22-episode-sweep/episodes.jsonl, run/matches/*-replay-human-*, docs/briefs/lessons.md]}
status: live
-->
## The people we play

What the people we have met in lobby games actually did, by person, with the games and dates. Their words are in the
lessons document and are not repeated here; this document is what they built and when, where they fought, and how
they ended games. Ratings: "OS" is the ranked duel rating, "lobby" the skill the lobby shows for team games (the same
person reads 48 in duels and 34 in a team lobby). Our records have no truth file for a lobby game, so unless a game
was replayed through the engine, what we know of a person is what our seats saw: sightings are floors, and a base
never scouted is a base we know nothing about. Each block names its ground: **card** (an engine replay, every unit of
every side), **sightings** (our seats' records: enemy units seen and destroyed), **person** (the user's or a player's
own reading), **model** (an Opus reader's reading of our records in the episode sweep of 2026-10-22, unchecked by a
person unless said). The sweep's rows are a model's reading until the user has checked a sample.

<!--meta
id: M-gecko-duels
layer: meta
population: [gecko]thebluegecko, OS 48.5, in 17 public ranked duels (13 on Comet Catcher, 2 Full Metal Plate, 1 Gasbag Grabens, 1 Comet against an OS 29), 2026-08-09 to 2026-09-13
map: Comet Catcher Remake 1.8; Full Metal Plate 1.7; Gasbag Grabens 1.1.1
dates: 2026-09-24 (carded), 2026-09-28 (the Rover count corrected)
ground: card
sources: {code: [run/data/replays/manifest.jsonl, docs/knowledge/replays.md, run/replays/assist.py], claims: [K-open-comet-thebluegecko-rover-mass, K-map-comet-catcher-remake-1-8-two-solars-before-the-plant, K-econ-commander-low-while-assisting]}
status: live
-->
### thebluegecko in ranked duels

The strongest player in the pool (OS 48.5; 12 of 17 duels won, against Artur91 at 42-43 and Hellontoast at 40-41
mostly). On Comet Catcher as Armada he plays one line in 13 of 14 sides: extractors 0:03, 0:10 and the third home spot
0:25, solars 0:31, 0:40, 0:49, the vehicle plant at 0:58 standing 1:17, the fourth solar at 1:32, six by 4:00; the
commander on the plant 140 s of the first 240 and set to low priority while it assists (10 of his 12 toggles). The plant
makes a Blitz at 1:17 and then Rovers every three seconds, the first constructor seventh (about 1:40), a second at about
2:40, and 24 factory units by 4:00 against the pool's 18; 4 to 9 extractors at 4:00 (the pool 7). Both stores run empty
and he keeps going: the energy store under 20 from about 1:45 to 3:50 in 11 of 14 sides. The Rovers go to the enemy's
base one by one as they are built, never as a body: the first within 1,500 of his start at 1:54, 2 to 12 alive at 3:00,
10 to 24 lost by 5:00 with nearly every death in the enemy's half. The user's ruling on this line (2026-09-28, from the
experienced players): valid only with specific Rover micro, which our hands do not have; it is an example, not the
line. His one bot-lab side (the lab at 0:03 before any solar, eight extractors by 4:00, Pawns and Ticks) won in 8:01.

Off Comet he plays Cortex. Full Metal Plate against Artur91 (two games, 2026-09-12, a metal-rich map of twelve
thousand elmos): a vehicle plant at 0:50 and seven more by 8:48, Rascals by the hundred then Incisors, Brutes and
Levelers (100 Brutes and 52 Levelers standing at 12:00), two advanced vehicle plants at 11:09, an experimental gantry at
14:45 and four more by 21:00, 154 extractors at 10:00, 408 at 20:00, 632 at 30:00; he won the 30-minute game with
Shivas and Juggernauts and lost the 16-minute one to Artur91's 166 Stouts and 34 Bulls. Gasbag Grabens (Cortex, won
7:06): a bot lab at 0:58, 16 Grunts by 4:00 and 33 by 6:00, 15 extractors at 6:00, the first Grunt in his half 2:59.

<!--meta
id: M-gecko-alone-vs-barb
layer: meta
population: thebluegecko alone against six and against two BARb hard_aggressive (his own replays, 2026-09-26)
map: Great Divide V1; Altair_Crossing_V4.1
dates: 2026-09-27 (carded)
ground: card
sources: {code: [/ceph/christian/within-reason/run/matches-archive/1790471170-replay-gecko-gd-1v6-barb/card.md, /ceph/christian/within-reason/run/matches-archive/1790471255-replay-gecko-altair-1v2-barb/card.md], claims: [K-opp-thebluegecko-one-ball-behind-turrets-then-tier-2]}
status: live
-->
### thebluegecko alone against BARb

Great Divide, Armada north, one against six hard_aggressive (won 19:45): two spots by 0:30, the bot lab at 1:11,
three Beamers and two rocket turrets at the pass's north mouth by 5:10 and nothing else forward, two to seven Pawns and
two Warriors for the first ten minutes (army 582 to 918 from 4:00 to 10:00), all eleven northern spots between 6:39
and 9:02 with wind generators, the advanced bot lab at 10:09 (first tier-2 unit 11:01), then Fidos, Zeus, Fatboys and
Sharpshooters: 5.9k of army at 16:00, 9.6k at 18:00, and one sweep through all six seats from 15:00 (26 of theirs dead
at the pass for none of his in minutes 5 and 7). Altair Crossing, Cortex, one against two (won 20:42): the bot lab at
0:42, 16 Grunts by 4:00 to take and hold the west strip's spots, then Thuds only (13 at 8:00, 20 at 10:00, 32 at 16:00,
36 at 18:00) with a light turret and a radar at each outer spot; 10 spots at 12:00, 15 at 20:00; no tier 2 at all; one
push at 18:00 into both bases that ended it. The shape in both: the whole side's spots early, one unit type massed into
one body on its own turrets, and one push when it outweighs what stands there.

<!--meta
id: M-gecko-vs-us-comet
layer: meta
premises: [M-pool-expansion, M-pool-closing]
lessons: [L-lead-use-it, L-lead-together, L-mix-adversarial, L-raid-naked-mexes]
population: thebluegecko against two or three seats of ours on Comet Catcher (the 3v1 series of 2026-09-26/28: games 1, 2, 3, 8b, 10; human-15, -19, -20, -31), lobby 34 to 41
map: Comet Catcher Remake 1.8
dates: 2026-09-26 to 2026-10-11
ground: card for the four replayed games (1, 2, 3, 8b); sightings and model for the rest
sources: {code: [/ceph/christian/within-reason/run/matches-archive/1790471894-replay-human-comet-3v1/card.md, /ceph/christian/within-reason/run/matches-archive/1790471894-replay-human-comet-3v1-2/card.md, /ceph/christian/within-reason/run/matches-archive/1790483183-replay-human-comet-3-gecko/card.md, /ceph/christian/within-reason/run/matches-archive/1790483183-replay-human-comet-9-gecko/card.md, docs/studies/2026-10-22-episode-sweep/episodes.jsonl], claims: [K-opp-thebluegecko-one-ball-behind-turrets-then-tier-2]}
status: live
-->
### thebluegecko against our seats on Comet Catcher

**His economy, from the four replayed games (card).** The same plant at 0:58 at B4 as in his duels. Extractors 8 to 10
at 4:00, 14 to 19 at 6:00, 18 to 27 at 8:00, 22 to 29 at 10:00, 25 to 32 at 12:00, 32 to 35 at 16:00, 31 to 57 at 20:00
(the strong duellists' 22 at 12:00; our three seats together held 32 to 47). Metal income 34 to 43 at 6:00, 41 to 59 at
8:00, 48 to 69 at 10:00, 57 to 85 at 12:00, 82 to 132 at 16:00, 133 to 264 at 20:00. Constructors 4 at 4:00, 4 to 8 at
6:00, 5 to 10 at 8:00, 13 to 18 at 16:00, 24 to 28 at 20:00. A second vehicle plant at 7:28, 8:49, 9:05 and 9:12; a third,
forward at D6, at 12:51 in one game; an aircraft plant at 11:29, 14:26, 16:42 and 17:06; the advanced vehicle plant at
11:18, 13:58, 16:08 and 17:33 (the first advanced constructor 35 s to 2:00 after); a gantry at 23:13 in the 26-minute
game. He lost his first extractor at 5:13, 5:41, 7:40 and 8:51 and 14 to 37 over a game; we lost 70, 43, 37 and 35 to him.

**His army (card).** Rovers 5 to 9 at 2:00-4:00 and 1 to 5 Blitzes; Blitzes 7 to 13 at 6:00-8:00; Stouts and Janus from
7:50 to 10:00 (5 to 7 Stouts and 2 to 5 Janus at 10:00, 12 to 16 Stouts and 7 to 11 Janus at 12:00, 15 to 25 and 6 to 12
at 14:00); then tier 2 and air: Bulls (5 at 14:00 and 24 at 20:00 in one game; 3 to 15 at 18:00-26:00 in the others),
Thunder bombers (3 at 16:00, 10 at 18:00-20:00), Banshees (20 at 14:00 in game 3), fighters (8 to 41 from 16:00 against
our bombers), Mausers (7 to 8 from 16:00), Razorbacks (2 to 7 from 24:00). Army metal 500 to 740 at 4:00, 1,100 to 1,800
at 8:00, 1,600 to 4,900 at 10:00, 4,350 to 5,700 at 12:00, 5,200 to 8,500 at 14:00, 7,100 to 13,300 at 16:00, 8,400 to
24,300 at 20:00. Over a game he lost 63 to 82 Stouts, 18 to 28 Janus and 28 to 56 Rovers, and killed 81 to 126 Blitzes,
42 to 80 Stouts and 35 to 70 extractors of ours.

**How he fought us (card, confirmed by the user's reviews of the time).** Single Rovers and Blitzes at the outer
extractors of every seat from 2:06 to 2:50, at both ends of our strip at once; a forward post of light turrets, a
Beamer, extractors and constructors at D3-E3 by 4:43-8:00; the one even trade at D3 at 9:00 (21 lost for 32, his
push into our middle); then blocks of ten to twenty Stouts and Janus into one seat at a time (the north seat's home
at 12:00-14:30, the south's at 16:30, G4 and G5 at 15:00-19:00: 3 lost for 33, 3 for 14, 7 for 23), never meeting our
whole army. His losses fell in our half almost entirely (17.2k of his in our half, 4.3k at our base, 2.5k in his half,
1.3k at his base, in game 2's record). Our commanders died at their plants to the block. Where our ball reached his
side it died on the post or his base (B4 at 8:00 in game 2: 5 of his for 16 of ours).

**The later Comet games (sightings, model).** From human-15 (2026-10-03) he played Cortex in every Comet game though
the lobby listed Armada: Rascals at both ends at 2:41-2:57, Incisors from 3:31-4:00, Pounders from 5:24-5:40 against our
light tanks, armed extractors (Exploiters) with a light turret and a radar on the centre column at E3-E4 by 6:01-7:54
and a nest at C1, Twin Guards at D5 and E6 later, Incisor swarms of 5 to 11 through the south-west at 8:00-11:30,
Shuriken drones from 12:32, tier 2 at 17:46 (a Tiger) in game 10; in human-19 Incisors held C3 behind Exploiters, his
commander D-gunned seven Stouts at his plant, and he resigned at 15:23 when his D4 army died; in human-31 his
commander laid extractors with Guards at C4 by 2:24, at E4 by 4:50 and at E2 by 13:12, in our half, behind a Pounder
block, with Whirlwind bombers from 11:15 and Tigers from 13:21 (25 by 19:00). We won two of these (human-15 at 10:20,
human-19 at 15:23), both 3v1 or 2v1 at even resources where our raids held him to 13 extractors in sight.

<!--meta
id: M-gecko-other-maps
layer: meta
premises: [F-map-ground, F-map-metal]
lessons: [L-water-tidals, L-water-assist, L-mix-hovers, L-mix-arty-not-the-body]
population: thebluegecko against our seats on water and other maps (Cape Violet 2026-09-26 and 10-03 and 10-18; SailAway 2 twice 2026-09-27; Gecko Isle 2026-10-02, 10-12, 10-19; Coast To Coast 10-03; All That Glitters 10-03; Isidis 10-03; Eclipsed 10-11; Flats and Forests 10-11; Azurite Shores 10-18 twice; Great Divide 2v1 2026-09-26)
dates: 2026-09-26 to 2026-10-19
ground: card for Cape Violet (2026-09-26) and Great Divide; sightings and model for the rest
sources: {code: [/ceph/christian/within-reason/run/matches-archive/1790483222-replay-human-cape-violet/card.md, /ceph/christian/within-reason/run/matches-archive/1790471126-replay-human-gd-2/card.md, docs/studies/2026-10-22-episode-sweep/episodes.jsonl], claims: [K-map-cape-violet-v1-a-water-map-with-a-third-of-the-spots-under-water, K-opp-thebluegecko-one-ball-behind-turrets-then-tier-2]}
status: live
-->
### thebluegecko on water and other maps

**Cape Violet, 2026-09-26, alone against three seats (card).** Armada bots from B3: four wind turbines, the bot lab
at 0:34, two Pawns as his whole army until 7:41. He expanded harder than anyone we have carded: 12 extractors at 4:00,
18 at 6:00, 31 at 8:00 (constructors 7, 12, 15), geothermals from 4:20, shipyards at 4:21 and 5:15 whose construction
ships took the sea (16 under-water extractors by 8:00, 24 by 10:00, eight of them on our side), the bot lab reclaimed at
5:12 and both shipyards by 7:42, the advanced bot lab at 6:56 standing 7:21 with a moho at 7:34. Then Welders (Zeus):
3 at 8:00, 12 at 10:00, 24 at 14:00, through the fords into our strip; an air plant at 9:53, an advanced one at 12:11,
Liche atomic bombers with radar planes that killed all three of our commanders 17:19-18:18; a missile ship and a
battleship on the coasts. Income 68 at 8:00 and 273 at 10:00 (the 10:00 figure includes the reclaim of our Blitz column
that died in his base at 8:35-9:03: 60 of ours for 2 of his). He lost 26 extractors and 52 Welders over the game.

**Great Divide, 2026-09-26, against two seats (card).** Not a pass camp: the bot lab at 1:25, all eleven northern spots
by 5:25 with three constructors, two light turrets at the pass's north mouth at 7:05, and his commander through the
pass at 4:26 with a Rocko and a Warrior, in our half every minute from 5:12 to the end, building a forward nest at
D6-D7 (light turrets from 8:02, a Beamer 12:07, a heavy turret 12:27). Warriors only (8 at 8:00, 12 at 12:00) that met
our Hammers and Thugs arriving one group at a time at the pass's south mouth: he lost 16 Warriors and killed 42 Thuds,
37 Hammers and 16 Storms. The advanced bot lab at 11:16, Zeus and a Fatboy by 14:00 for the sweep.

**Sea maps (sightings, model, and his words).** SailAway 2 (twice): Cortex, a shipyard by 3:30, Riptides from 4:10
(480 reach against our 280) as the main body for the whole game, a sonar tower at the crossing his raids came through,
Orca submarines from about 9:00 behind the Riptides, a seaplane platform at 9:05 with Cutlasses from 10:04 (27 in one
game), Buccaneers (tier 2) at 21:23; he took our fleet piece by piece in our water. Coast To Coast: Armada navy from
the start, Eels shelling from out of sight by 3:26, Dolphin packs of 3 to 9 on our undefended sea extractors 7:42-9:33,
Corsairs (700 reach) with Skaters and Ellysaws beating our hovers from 10:00. Azurite Shores 2v1 (2026-10-18, our win
at 18:32): Cortex, lost 15 extractors and 7 constructors to our Goons by 9:00, answered across the water with Garpike
landings from 7:04 that took 21 of our extractors with no landing party paid for, tier 2 by 9:00 (nine mohos seen from
9:58), two fusions by 13:54; he held the trade even to 16:00 on 6 to 10 extractors in sight against our 35 to 38. His
word after game 11, relayed by the user: better than BARb at water; artillery and Mangonels one to three behind a body,
never the body (the lessons).

**Land maps with two or three seats of ours (sightings, model).** Gecko Isle (human-10, 3v1, lost 23:45): Cortex, Grunt
raids in waves of 6 to 8 into all three seats from 4:04, a forward outpost of Exploiter, four Twin Guards and a radar at
D4 by 5:29-6:09, tier 2 unseen behind it (Reapers 14:00, a Tzar 14:05, Shadow bombers 16:12, Shuriken drones 18:31, a
Shiva 19:36, a Behemoth by 20:44); the user put his income at six times ours at 20:19. All That Glitters (human-18,
3v1, lost 32:19): Rascal and Incisor raids on our outer spots from 4:55, his own spots held by Exploiters and Pounders
with Twin Guard nests, Brute masses into our homes from 18:36, Graverobbers raising our dead (138 Armada units fought
for him), Tzars and Tigers from 22:00; his mobile units destroyed in our sight came to 186k metal by the end, about 12k
a minute from 18:00 to 30:00, from a half we never scouted. Isidis (human-22, 2v1, lost 18:40): Grunts singly and in
small parties at our outer extractors from 3:13 (14 by 7:35, none of the parties paid), two Exploiters and a Guard on the
middle spots by 6:13, a vehicle plant by 7:42 (Pounders, Incisors), Graverobbers raising our Pawns from 8:13, Whirlwind
bombers from 12:57. In every one of these his factories, commander and economy stayed out of our sight.

<!--meta
id: M-gecko-team
layer: meta
premises: [F-eco-build, F-eco-extractors]
lessons: [L-tier2-go, L-tier2-tier1-under-tier2, L-energy-nanos]
population: thebluegecko with u6bkep (and Covariance once) against three or four seats of ours: Eclipsed 2026-10-11, human-31 Comet 10-11, Flats and Forests 10-11, Gecko Isle 10-12 and 10-19, Azurite Shores 10-18, Center Command 10-19, Gasbag Grabens 10-20, Project SD-129 10-20
dates: 2026-10-11 to 2026-10-20
ground: card for Center Command (the engine replay, every team); person (the user's reviews of Center Command and SD-129); model for the rest
sources: {code: [run/matches/1792466765-replay-center-command-2v4b, run/matches/1792460593-human, run/matches/1792547311-human/review.md, docs/studies/2026-10-22-episode-sweep/episodes.jsonl], claims: [K-t2-people-share-advanced-constructors-across-factions, K-t2-moho-frames-finished-by-tier-1-build-power, K-t2-people-eat-each-plant-for-the-next, K-opp-people-nuke-late-or-never]}
status: live
-->
### thebluegecko and u6bkep as a team

Two people against three or four seats of ours, since 2026-10-11, won every game but none of them ours. They split
the map by row or by land and sea (thebluegecko Cortex and the land or the north, u6bkep Armada vehicles and the
south, or the sea on Azurite), raid from the first minutes with whatever is fastest, and tech behind the raids out of
our sight.

**The sprint, Center Command 2026-10-19 (card and the user's review).** Two people on about twelve spots each.
thebluegecko placed his advanced bot lab at 7:08 on 11 extractors and a 1,165 bank and reclaimed his tier-1 lab the
same second; it stood at 8:52 and made five advanced constructors by 10:15, the second handed to u6bkep, who built
Cortex mohos on his Armada spots with it from 9:47. Mohos went up at 19 to 33 s each with the two advanced constructors
starting frames and tier-1 constructors and construction turrets finishing them: 8 standing at 12:00, 12 at 13:00, 21 at
17:00, 34 to 37 at 20:00; the construction turrets grew to 73 by 21:00. Each advanced plant was eaten for the next: the
bot lab reclaimed at 13:20 for an advanced vehicle plant placed forward at E2 in our half (standing in 45 s), that one
reclaimed at 18:44 for an advanced air plant (standing in 5 s with 26 construction turrets on it); fusions at 18:00
and 18:18. The side's income: 55 at 8:00, 106 at 12:00, 250 at 15:00, 277 at 20:00 (ours 59, 56, 63, 56). In the field
his commander led a heavy-turret creep from D2 to E1-E2 from 6:24 that took our north spots; Sheldons with a jammer,
Tzars, Mausers and Wasps finished it at 21:05.

**Project SD-129 and Gasbag Grabens, 2026-10-20 (the user's review; sightings).** On SD-129 he raided with 4 to 6
Rascals from 4:21, guarded his extractors with armed extractors, had an air plant by 6:17 (Whirlwinds killed a fusion
frame at 14:07), and sent Brutes and a Pounder through the middle from 10:38, ten to eleven Brutes into one seat's home
at 12:41-14:55; u6bkep walked his commander east to a post of three Sentries at E7 by 5:00, stood Blitzes at E6 from
5:02, took one seat's home with 8 Blitzes and 3 Stouts at 8:17-9:37 and roamed with Stouts and Janus after. No tier 2
of either was seen in fifteen minutes: about 12,000 metal of tier-1 vehicles by unique id (41 Brutes, 40 Blitzes, 28
Stouts) against a side that had built economy and had 17 soldiers from 4:00 to 6:00. On Gasbag the same pair: u6bkep's
commander forward to F7 in our half with a bot lab and a Beamer by 5:49, Pawns and Maces razing two homes 8:00-10:09;
thebluegecko's Grunts taking all six of one seat's extractors 6:13-7:14 and its advanced lab at 9:17, tier 2 by about
12:38 and a forward nest with a Scorpion and a Persecutor that out-ranged our last base.

**Nukes (person).** Neither has built a silo in any game of ours: none in 21 minutes on Center Command at 37 mohos and
two fusions; the user's ruling of 2026-10-20 is that they nuke very late or never, while BARb and Apex have clocks of
their own (the opponent documents).

<!--meta
id: M-u6bkep
layer: meta
premises: [F-unit-armflash, F-unit-armstump]
lessons: [L-raid-naked-mexes, L-other-u6bkep-nests-and-air]
population: [gecko]u6bkep, lobby 19 (2026-09-27) rising to 30 (2026-10-07 on); Armada vehicles in every game but one; alone against one to three seats in games 7 and 8 of the 3v1 series (2026-09-27), human-21 (2026-10-03), human-25 to -30 (2026-10-06 to 10-09)
map: Comet Catcher Remake 1.8 (all of his solo games)
dates: 2026-09-27 to 2026-10-09
ground: sightings, model
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl, docs/briefs/lessons.md]}
status: live
-->
### u6bkep alone

Armada vehicles from one plant at B4 in every solo game, his commander kept at home. He opens with Rovers at 2:30-3:00
(one at our outer spots, then six at 3:53 in human-27 that died to the commander and the Guards) and switches to
Blitzes at about 4:20: waves of 6 to 11 every minute and a half into one home (human-27: 4:26, 6:22, 7:41, 8:46, 10:07;
45 Blitzes and 14 Rovers lost holding us at four extractors), or packs of 5 to 7 on the middle spots (human-28: eight
extractors by 8:05, 23 Blitzes lost). Light turrets at his outer spots by 3:00 and Blitz pairs on the middle kill our
single raiders. Stouts from 9:00 to 10:35 and Janus from 10:54 to 11:18, then the Stout and Janus block kills the plant
(11:51, 12:12, 14:02) and follows the commander. Beamers behind the block from 8:27; a heavy laser tower at C3 by 6:55
in human-29 with Sentries on four home spots. In the games we won he was raided to 4 to 7 extractors in sight (human-29,
-30), built energy heavily (12 solars, 4 advanced solars, 2 energy stores died at his base) and reached tier 2 late (an
advanced constructor at 16:25 and 17:29; a moho at 18:36); in human-29 he went air at 9:16 (a Peeper, five Thunders on
our home extractor at 10:06). By unique id in human-30 we saw 77 Stouts, 28 Janus and 32 Blitzes, about 27.5k of soldiers:
far more than the "three to one" our player assumed. His own reading at 5:01 of game 7, spectating thebluegecko: "the
human player is taking mexes so much faster that 3v1 is about fair" (the lessons). In the 3v1 games 7 and 8 (lobby 19)
his Blitz bands of 2 to 10 took 18 of our extractors and a commander by 6:12 and he lost 46 Blitzes doing it; we won
both at 9:13 and 13:59.

<!--meta
id: M-irishstud14
layer: meta
lessons: [L-mix-pounders-mediums, L-energy-nanos, L-reclaim-field]
population: [Stud]Irishstud14, OS 48, Cortex, alone against three seats (bluegecko-3v1-comet-catcher-4, 2026-09-27, lost by us about 20:40)
map: Comet Catcher Remake 1.8
dates: 2026-09-27
ground: sightings, model
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl, docs/knowledge/_inbox/players-chat-2026-09-27.md]}
status: live
-->
### Irishstud14

One game, Cortex, the west box. He said "I greed then" at 0:27 and did: naked extractors in the middle and his half
(his first Guard seen at 5:16), Rascal swarms (10 seen at 2:58, 29 killed) on our outer spots 3:53-6:22, Incisors from
6:52 (100 seen killed), Brutes from 7:13 (64), Pounders from 8:10 (59), all from vehicle plants. The Pounders stopped our
push at his base at 8:00 and then roamed our north in pairs; his commander went forward to D3 at 11:50 with Brutes and
Guards. From 13:22 he raided in three or four parties, the biggest 14 to 18 Incisors, choosing our outer and southern
extractors over our ball (our extractors 33 to 9 by 19:00 while we won every fight we had); Graverobbers reclaimed and
resurrected at E3 from 17:28 (our Stouts fought for him). No tier 2 and no aircraft in twenty minutes. His army in
sight never passed 4,070, yet what we saw die or stand of his (45k) exceeded all three seats' army: one person
out-produced three seats. His advice to the player in game 3, spectating, is in the lessons.

<!--meta
id: M-other-gecko-people
layer: meta
population: [gecko]Infern8 (lobby 25; games 5 and 6 of the 3v1 series, 2026-09-26/27) and [gecko]Covariance (lobby 17; with u6bkep in human-25 and -26, 2026-10-06, and in the Gecko Isle 4v3 of 2026-10-19)
map: Comet Catcher Remake 1.8; Gecko Isle Remake v1.2.1
dates: 2026-09-26 to 2026-10-19
ground: sightings, model
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl]}
status: live
-->
### Infern8 and Covariance

**Infern8 (25).** Game 5 (we won 9:35): Armada, one vehicle plant at B4 in sight from 2:00, Blitzes out by 2:09 raiding
in fours and fives, a Blitz body of 11 to 16 at D4-E4 from 6:59 in front of a turret nest (six turrets and three
extractors), Stouts and two Janus at home, his commander fighting at B4-C4 from 8:39 (26 of our raiders killed). Game 6
(we lost about 17:05): he played Cortex though the lobby said Armada; a plant at B4 with a construction turret by 2:36,
Incisors from 2:36, his commander forward to D4 at 6:38 with a second plant and a light turret, Brutes and from 8:29
Pounders held D4 as one block with the commander inside through our attack of 8:35-9:20 (3.5k lost, the block held),
Brute raids of 7 to 9 into our north from 10:05, then the one block (13 Pounders and 15 Brutes at 14:45) moved from
seat to seat; Graverobbers raising our wrecks from 15:02, Reapers at 17:00. We saw 66 Brutes and 40 Pounders and 9
extractors.

**Covariance (17) with u6bkep (19), human-25 and -26.** Two Armada vehicle players: a Rover mass of 14 to 17 into our
homes from 1:51 (65 Rovers died by 5:50 and took 9 extractors, 5 constructors, 3 solars and 3 Sentries), Sentries on
every outer spot, Blitz packs of 4 to 8 from 6:20, a Blitz mass of 15 to 25 in a base at 9:13-10:07 (we lost human-26 at
10:07); in human-25 (we won 24:22) a commander walked forward and raised a vehicle plant at D8, 900 from our south seat's
spot, at 8:44, Beamer nests with commanders beat our pushes, and a Stout and Janus army in blocks of 14 to 17 ate our
middle extractors from 12:50 with Lazarus raising our wrecks from 15:58.

<!--meta
id: M-thedujin
layer: meta
population: [APM]thedujin, lobby 37, Armada, two 1v1s on Comet Catcher (human-23, 2026-10-04), both lost by us
map: Comet Catcher Remake 1.8
dates: 2026-10-04
ground: sightings, model, and his own announced plans in the chat
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl]}
status: live
-->
### thedujin

Two 1v1s at lobby 37. The first he announced ("i will try cheese", "t2 gunship rush") and played: almost no ground
army (one Pawn seen at 4:20), Sentries beside his spread extractors, and the economy into an advanced aircraft plant at
A8 with three construction turrets, six advanced solars, two energy storages and three Beamers (all first seen 9:52);
seven Brawlers out by 9:49, eight with a radar plane raiding our F-column extractors 10:29-10:42 and killing our
commander at about 11:00; "why AI no try attakc" at 7:38. The second ("1 scout 4 con greedy expansion open"): Rovers,
one in our base at 2:02 and a party of ten at 4:05 that lost eight to our commander and four Blitzes; naked expansion
(our three Rovers and a Blitz killed six of his extractors 3:02-5:52); Blitz raids of 2 to 5 on our home spots 6:30-9:00
("i'm fee3ding" at 8:07) that still wrecked our economy; an advanced solar and an energy store by 6:58; 16 Blitzes and 4
Stouts at D3 by 10:55, a scout plane and Thunder bombers at 10:50-11:15, a Stout push of nine at 12:14 that ended it at
14:13. No tier 2 seen in either.

<!--meta
id: M-lobby-5v5
layer: meta
population: five people of lobby 8 to 36 (Gabiiiiiii7 36, apexearth 31, SSFA 17, drakcore 11, [Crd]TimmyT 8) against our two seats and three Apex seats, all five of ours at +50% income, Altored Divide, 2026-10-18 (human-8v8-1, won when they left at 57:38)
map: Altored Divide Bar Remake 1.6.2
dates: 2026-10-18
ground: sightings, person (the user's review), model
sources: {code: [run/matches/1792302895-human-8v8-1, docs/studies/2026-10-22-episode-sweep/episodes.jsonl]}
status: live
-->
### A lobby team of mixed strength

Five people playing as separate bases, Armada, Cortex and two Legion. The Armada players walked their commanders
forward with turrets (B4-A5 nests of a heavy laser tower, Beamers and light turrets; a D3 nest that grew an
Annihilator and a Pulsar) that killed our short-ranged raiders 5:00-10:43; an advanced bot lab at D2 seen going up at
12:15; a Legion Incinerator at G2-H3 killed 130 of our 131 Thugs that walked into its 725 reach one by one from 14:00
to 18:30; Cortex gunships from 14:07; fusions and advanced fusions (55k of economy at D1 died at 35:25). The decisive
player was the Legion north-east, who turtled from 27:00 behind 13 and more Bastions (reach 1,100), Soteria shields,
Erebus jammers, Tyrannus gunships from 39:30, an Astraeus (3,100) and a gantry, with a silo at H2 that fired once at
48:26; our radar at G4 was blind to that base by terrain (a ridge of 450 to 500 elmos). Their words: "help would be nice
... people just ignore" (drakcore 9:50), "soo late this is not eco game" (Gabiiiiiii7 32:27), "mass bastions does hold
off enemies pretty well" (apexearth 49:59). They left one by one from 44:26.

<!--meta
id: M-quicksilver-people
layer: meta
lessons: [L-other-quicksilver]
population: unnamed people in the user's lobby on Quicksilver, 2026-09-21/22 (human-1 to -8, eight 1v1s, all lost by us in 3:31 to 6:13); ratings not recorded
map: Quicksilver Remake 1.24
dates: 2026-09-21, 2026-09-22
ground: sightings, model
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl, docs/briefs/lessons.md]}
status: live
-->
### The Quicksilver people

Eight games, all Armada Pawn rushes from a bot lab, and all over by 6:13. The first Pawn is seen in our half at 1:51
to 2:08 (at D4-E4, the middle of the map), two to five by 2:10-2:40, seven to ten by 3:00-4:20, twelve to fifteen by
4:30-5:50; nothing but Pawns was ever seen (one Lazarus with them at 6:00 in human-7). They go for the expanding
constructors and the naked extractors and wind generators first (the first kill at 1:59-2:39), the lab when they have
eight on it (2:35-4:46), the commander last; they lose Pawns to the commander and keep feeding them (13 to 33 seen
dead a game, 486 to 1,780 metal, against 1,862 to 4,840 of ours). No building of theirs was ever seen and none of our
units reached their side. One opponent's own account (4:20 of human-3): "2 mex, wind because the speed was high. one
construction bot expanding all else was pawns". Their rules of thumb for the map (wind 12, one solar then wind, about
twenty energy for every metal, two constructors is greedy) are in the lessons.

<!--meta
id: M-matt-and-vak
layer: meta
population: Matt (a friend of the user's; one replay against BARb medium on Quicksilver, 2026-09-20) and [MADO]VAK (OS 52) against [MADO]Artur91 (43) on Comet Catcher (one replay, 2026-09-09)
map: Quicksilver Remake 1.24; Comet Catcher Remake 1.8
dates: 2026-09-22 (both transcribed)
ground: card
sources: {code: [/ceph/christian/within-reason/run/matches-archive/1790122851-replay-vak-vs-artur-comet/card.md, run/matches/1789930356-replay-player2-vs-medium], claims: [K-open-matt-order-timings, K-open-comet-pro-order]}
status: live
-->
### Two single replays: Matt on Quicksilver, VAK on Comet

**Matt, Quicksilver, against BARb medium.** Extractors 0:03 and 0:10, wind 0:16, 0:22, 0:28, the bot lab 0:33 standing
0:50 with energy at 490 and rising; the commander guards the lab at 0:50, then orders seven wind generators and a solar
as one batch at 0:53-1:04 and has them by 1:30 (ten generators), then guards the lab for good from 1:36. The lab: a
constructor at 0:50, then Pawns from 1:12 every eleven seconds alone and every four to fourteen with the commander
helping, a second constructor at 1:45 after six Pawns; raiders out from about 2:30; the first far extractor at 1:45
with a turret beside it at 2:32; a converter at 2:55. Energy dipped to 15 once (1:01-1:07) and was full by 1:35; metal
spent as it came (8 banked at 2:00). His commander never left the base. The user's ruling (2026-09-22): the Quicksilver
line to copy.

**VAK (52) against Artur91 (43), Comet, 6:52.** VAK from A4: extractors 0:03, 0:10, 0:24, solars 0:31, 0:40, 0:49, the
plant 0:58 standing 1:16; two Rascals, a constructor 1:24, a second 1:41, Blitzes from 2:12 every ten to fifteen seconds
(13 by 5:00), a constructor at 3:09, 4:30 and 5:46; 9, 13 and 19 extractors at 4:00, 5:00 and 6:52 (income 23, 32, 43);
metal under 30 through minutes 2-4 then 300-500 banked from 5:00; three construction turrets on the one plant by 6:37
and an energy storage at 6:18, no second factory; his first soldier crossed at 3:53 and he killed Artur91's base at
6:50 with ten Blitzes (60 of Artur91's units died in the last fifteen seconds). Artur91 opened the pool's line (two
solars, the plant at 0:34) and ran his energy at 0-30 through minutes 4-6 on 18 solars.

<!--meta
id: M-user-as-opponent
layer: meta
population: computer_whisperer (the user), lobby 6, playing against our seats: human-11 to -14 (2026-10-02/03, Avalanche, Quicksilver, Altair) and the Cape Violet 1v1 at +100% income (2026-10-18)
map: Avalanche 3.4; Quicksilver Remake 1.24; Altair_Crossing_V4.1; Cape_Violet_V1
dates: 2026-10-02 to 2026-10-18
ground: sightings, model; the user's own account where he gave one
sources: {code: [docs/studies/2026-10-22-episode-sweep/episodes.jsonl, run/matches/1792372195-human/review.md]}
status: live
-->
### The user as the opponent

Armada bots. Pawns across the map from 1:50-2:18 (human-11, -13): singles and parties of three to four at our outer
spots and constructors, several parties at once, none of them paid for; an outpost of an extractor, a Sentry and a
constructor in the middle by 6:25, Sentries crept onto our spots later; Maces and Centurions from 7:54-8:27 (reach 380
and 325 against the Pawns' 180) that walked through our raid and our extractors and left sight again; fifteen Pawns on
our east at 10:58-11:42; a construction aircraft at 11:01; his commander at home. We won human-12 and -13 (his "silly
build" of construction aircraft and Banshees; his commander at E3 killed at 14:57). At +100% on Cape Violet (we conceded
33:31): a Guard and four Centurions at the B3 passage by 5:00, a Pawn, Rocketeer and Centurion wave of 3 to 5k at
7:45 that broke on our turrets, a shipyard by 11:59 (Dolphins, Corsairs, a construction ship on our spot), an advanced
bot lab by 12:30, resurrection bots on our wrecks from 12:19, six to nine Fatboys from 15:13 with the fleet, Titans
with eleven Lazarus from 23:55, an Epoch at 29:56, and a silo whose missiles landed at 31:02 and 32:20. His losses fell
in our half (46k) and at our base (14k) against 370 in his own; his base was never seen. The user's own review of that
game: our player never took the thirteen under-water spots in our half.
