<!--meta
id: M-format-intro
layer: meta
population: the three formats we play or aim at: duels, team games of two to four a side, and 8v8
dates: 2026-10-22 (the user's ruling that the formats are different games; the first 8v8 card)
ground: mixed (each block says)
sources: {code: [docs/briefs/meta/pool.md, docs/briefs/meta/people.md, docs/briefs/meta/barb.md, docs/studies/2026-10-22-episode-sweep/games.jsonl, run/matches/1791158837-replay-smrtbarb-8v8-glitters/card.md]}
status: live
-->
## The formats: what is common and what each changes

The rules, the prices, the maps and the units are the same in every format; what changes with the number of players
is the ground each player holds, the number of fronts, how long a game lasts, whether an economy can be pooled, and
what the opponents are tuned for. The user's ruling (2026-10-22): 8v8 is its own developed meta, the popular category
is eight people against eight AIs on a map like All That Glitters, our 1v1 proficiencies do not carry into it, and
BARb and Apex are probably tuned with that mode in view. The evidence base differs by format and this document says
which: duels have the pool (40 carded games at OS 40+); team games of two to four a side have our 57 lobby games with
people (12 won; the people document); 8v8 has one carded public game of eight people against eight SMRTBARb, a second
being carded, our four games of eight seats against eight BARb, the lobby 5v5, and a public pool not yet mined (the
replay service lists 92 games of eight people against eight AIs on All That Glitters in ten days, and 100 8v8 games
between people in two days).

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
population: 8v8: eight people against eight SMRTBARb on All That Glitters (2026-10-04, replayed and carded: people of lobby 2 to 57, lost 32:46); our eight seats against eight BARb hard_aggressive on the same map (four games, 2026-10-04 to 10-10); the lobby 5v5 on Altored (2026-10-18); the public pool's team games with bots on Glitters (150 in ten days, 92 of them eight against eight; medians 26 minutes, p75 41) and between people (100 in two days; medians 25, p75 34)
map: All That Glitters v2.2.3 (6144 by 10240, 152 spots, two rows of eight starts across the short axis, the lanes north to south, about nine and a half spots a seat)
dates: 2026-10-22
ground: card (the SMRTBARb game); truth (our BARb games); sightings and model (the 5v5); the replay service's listing
sources: {code: [run/matches/1791158837-replay-smrtbarb-8v8-glitters/card.md, docs/briefs/meta/barb.md, docs/briefs/meta/people.md, run/data/demos-client], claims: [K-team-eight-seats-lose-to-eight-barbs-on-one-curve]}
status: live
-->
### 8v8

**The shape of the one carded game (card).** Sixteen starts in two rows across the short axis, eight lanes of about
750 elmos running the long way, nine or ten spots a seat. The people (lobby 2 to 57) opened four bot labs, one
vehicle plant, two Legion plants and one aircraft plant at 0:49, with tier 2 early: an advanced bot lab at 3:00 (the
Cortex seat, on six extractors), 7:08, 8:24 and 10:11, an advanced vehicle plant at 8:33, mohos from 6:07, and a
gantry at 26:36. Two seats played roles the duel does not have: an economy seat (68 wind generators, eight converters
and nineteen extractors by 10:00, the advanced vehicle plant at 8:33, the gantry) and an air seat (72 wind generators,
the aircraft plant at 0:49, an advanced one at 15:37, 56 Highwinds at 22:00 and 33 Highwinds with 15 Blizzards at
24:00). Their first soldier reached the enemy's half at 5:51 to 15:18 (two seats never): nobody raided early. The AI
side's first soldier was in their half at 1:32 to 2:43 in every lane. The side totals: people's extractors 57, 72, 73,
74, 68, 63 at 4, 8, 12, 16, 20 and 24 minutes against the AIs' 46, 64, 76, 72, 79, 76; metal income 117, 227, 339, 546,
596, 853 against 161, 278, 532, 1,075, 1,922, 2,172; constructors 25, 35, 58, 81, 81 against 32, 57, 117, 174, 237 at
4 to 20; army metal 2,900, 7,800, 12,500, 11,300, 25,900, 39,200 against 3,300, 14,800, 12,500, 25,500, 64,600, 119,200.
The fighting stood in the middle lanes (E5-F5) from 16:00 to 24:00 (1,800 deaths there), moved to rows 3 and 4 at
24:00 and into the people's bases at 28:00; the people lost 4,483 units worth 1.18 million metal, the AIs 3,413 worth
640,000, and the AIs' losses were Grunts (829) and Incisors (425) by the hundred. The people's economy was level with
the AIs' at 8:00 and a third of it by 20:00: the game was lost on the economy's scaling from 12:00, not in the lanes.

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
side, so a raid hits a lane and a neighbour can answer it; the economy is the side's, pooled by transfers and shared
build power, so one seat's sprint feeds eight and the side's income is what matters, not a seat's; the game runs 25
to 40 minutes, so tier 2 by 10:00 and gantries and fusions by 20:00 to 30:00 decide it, and a side that is level at
8:00 and does not scale its economy is a third of the other by 20:00; air is a lane of its own with a player on it;
and the AI side that holds the category raids every lane by 2:00 and builds economy and construction turrets before
army. None of this is derived yet; it is what one card and four truth files show.

<!--meta
id: M-format-smrtbarb
layer: meta
premises: [F-barb-how-it-decides, F-eco-conversion, F-eco-build]
population: SMRTBARb, eight seats against eight people on All That Glitters, 2026-10-04 (one carded game; a second being carded); 320 of its seats in the public pool's Glitters games of the last ten days, not yet carded
map: All That Glitters v2.2.3
dates: 2026-10-22
ground: card
sources: {code: [run/matches/1791158837-replay-smrtbarb-8v8-glitters/card.md, run/data/demos-client, docs/briefs/meta/barb.md]}
status: live
-->
### SMRTBARb, the competition in that category

The user (2026-10-22): as far as he knows the state of the art, the bot we will meet head to head once we beat the
easier 8v8 bots; watch its play. It is a community build of BARb (its seats are named SMRTBARbstable in the start
script), so BARb's rules are the starting point (F-barb-how-it-decides) and its differences are what to read. In the
one carded game, as eight seats of all three factions:

- **Openings by seat.** A bot lab at 0:28, 0:29 and 0:33, a vehicle plant at 0:43 to 0:48, and one seat an aircraft
  plant at 1:11; first soldiers in the enemy's half at 1:32 to 2:43, six of eight seats under 2:10: it raids every lane
  before the people have left their bases.
- **One seat is an economy seat.** The Cortex seat at C8 placed its advanced bot lab at 2:28 on six extractors, built 21
  wind generators, 18 construction turrets, 14 advanced construction bots and a fusion by 10:00, never held more than
  six extractors, and ran 60 metal a second at 8:00, 342 at 16:00, 714 at 20:00 and 728 at 24:00 on converters; it
  made Grunts and Fiends, then gantries at 21:16, 27:46 and 31:07 and three Juggernauts by 28:00, with 62,000 of army
  at 24:00. The side's other seats went to tier 2 at 9:13 to 10:46 with mohos from 5:59.
- **The others expand and specialise.** The vehicle seat at C7 held 15 extractors by 10:00 with six advanced solars,
  five converters and six mohos, two advanced vehicle plants at 9:13 and 9:16, Tremors and Banishers; the Armada air
  seat at D8 (seven advanced solars, seven converters, eight construction turrets by 10:00, an advanced aircraft plant
  at 9:20 and five more by 26:37) stood 58 Highwinds and 11 Blizzards at 28:00; the Armada bot seat went to Fatboys,
  Sharpshooters and Welders; the Legion seat to drones, Hades and heat tanks.
- **The side.** 32 constructors at 4:00 and 237 at 20:00; income 161 at 4:00 (the people's 117), 532 at 12:00, 1,075 at
  16:00, 2,172 at 24:00; army 14,800 at 8:00, 25,500 at 16:00, 64,600 at 20:00, 119,000 at 24:00, 211,000 at 28:00; its
  losses Grunts and Incisors by the hundred, 255 Highwinds and 166 Hades, 640,000 metal in all against the people's
  1.18 million. It won at 32:46 against people of lobby 2 to 57.

What this is not yet: a description of its rules (read its scripts the way BARb's were read, if they are published),
a sample (one game against a mixed lobby), or a comparison with stock BARb at the same bonus on the same map. The
public pool's 320 SMRTBARb seats on Glitters are the sample to card next.

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
