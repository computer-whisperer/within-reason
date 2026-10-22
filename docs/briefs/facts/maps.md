<!--meta
id: F-map-intro
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Map/, upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/, upstream/Beyond-All-Reason/gamedata/modrules.lua, run/data/maps/*.sd7 (mapinfo.lua), docs/briefs/reference/map-sheets.md, crates/terrain/src/sheet.rs], game: 54199a0d, engine: 2026.07.04, dates: [2026-10-22]}
status: live
-->
## Maps: how the ground, the metal, the water and the weather work

Read from the engine, the game's rules and the maps' own files, with the numbers of the maps we play from their
archives and from the map sheets computed off recorded games. What a particular map is like for the game you are in
comes with the first report (its sheet, from your start); this says what the sheet's numbers mean and what holds on
every map.

<!--meta
id: F-map-grid
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Misc/GlobalConstants.h:24-45 (SQUARE_SIZE 8, BUILD_SQUARE_SIZE 16, 8 elmos a metre), upstream/RecoilEngine/rts/Map/ReadMap.cpp:741-765 (the slope map at half resolution), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/MoveMath.cpp:107-140, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:442-443 (speed in elmos a second), crates/bot/src/world.rs:49-52 (the 8 by 8 grid), crates/ai-host/src/engine.rs:284-310 (our terrain at 16 elmos), crates/terrain/src/lib.rs:20-45 (walking cost)], game: 54199a0d}
status: live
-->
### Distances, the grid, the clock

The unit of distance is the elmo; eight make a metre, so Comet Catcher's 8,192 by 6,144 is a kilometre by three
quarters. The ground is a height map with a sample every 8 elmos; slope, terrain type and metal are kept at half
that, one value per 16-elmo square, and buildings snap to that 16-elmo grid. The speeds in the unit tables are elmos
a second on flat ground, so a walk in elmos over a speed is seconds: at 100 a second a unit crosses Comet Catcher's
long side in 82 seconds and a map of 12,288 in two minutes. The reports name places by an 8 by 8 grid of the map,
columns A to H west to east and rows 1 to 8 north to south, so a cell is an eighth of the map each way (1,024 by 768
on Comet Catcher, 896 square on Quicksilver) and the cell says nothing finer. The sheet's walks are the engine's
own law summed along the cheapest route (flat ground at cost one, a slope at one plus the slope times the class's
factor), so they are the seconds the engine will take, not straight lines.

<!--meta
id: F-map-ground
layer: fact
premises: [F-fight-ground]
sources: {code: [upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/GroundMoveMath.cpp:12-52, HoverMoveMath.cpp:12-38, ShipMoveMath.cpp:11-28, MoveMath.cpp:81-140 (terrain-type multipliers; the direction of travel), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveDefHandler.cpp:195-260 (depth parameters), 744-770 (GetDepthMod), 157-161 (water damage), upstream/Beyond-All-Reason/gamedata/movedefs.lua:17-92 (DEPTH, SLOPE, the depth curves), upstream/Beyond-All-Reason/gamedata/modrules.lua:84 (allowDirectionalPathing), 98 (QTPFS), run/data/maps/*.sd7 mapinfo.lua terrainTypes], game: 54199a0d, claims: [K-maps-terrain-not-straight-lines, K-maps-early-walks-nearly-straight, K-maps-quicksilver-is-an-island, K-maps-factory-by-terrain]}
status: live
-->
### Ground: how terrain changes speed

The fight model gives each class's limits (tanks climb to about 40 degrees, hovercraft 50, bots and the commander 80;
tanks and bots wade to 20 deep, the commander and the amphibious units walk the sea floor, ships need 8). Short of
the limit, a slope slows a unit by one over one plus the slope times its class's factor, and only climbing counts:
the engine's directional pathing makes going uphill slower and downhill the flat speed, so the same ridge is crossed
faster coming down it. Water slows the waders from four deep by one over one plus three hundredths of the depth
(at 10 deep 77 percent of speed, at 20 deep 62) and the commander by a gentler curve floored at two thirds (92
percent at 20 deep, two thirds from 47 down); hovercraft and ships are not slowed by water. Each map paints its ground
in terrain types that multiply speed on top of this: on Comet Catcher, Great Divide, Center Command, Gecko Isle,
Isidis Crack, Mithril Mountain, Carrot Mountains, Feast of Hades and All That Glitters every type is one, so
only slopes matter; on Quicksilver the seabed is 0.8 for tanks and bots (0.9 for hovercraft), rock 0.7 (0.8) and
grass 1.05; Cape Violet's three roads run at 1.1, 1.2 and 1.3; Altored Divide's sand is 0.9 and its rocky ground
1.1. No map we play damages units in water. Where a unit cannot go, the engine's quad-tree pathfinder routes it
round, through gaps at least its footprint wide, and a column strings out at every narrowing; where no route exists
the order fails and the unit stands. Buildings, wrecks and trees block movement (a wreck in a lane stops the lane
until reclaimed); a factory's open side is its only exit.

<!--meta
id: F-map-metal
layer: fact
premises: [F-eco-extractors]
sources: {code: [upstream/RecoilEngine/rts/Map/MetalMap.cpp:29-44, upstream/Beyond-All-Reason/common/upgets/api_resource_spot_finder.lua:24-60, 120-200 (the strips, IsBuildingPositionValid), 232-300 (GetSpotsMetal, the spot's worth), 300-330 (mex_count game rules), upstream/Beyond-All-Reason/luarules/gadgets/cmd_mex_denier.lua:30-100, upstream/RecoilEngine/rts/Map/MapInfo.cpp:106-107 (maxMetal, extractorRadius), crates/ai-host/src/engine.rs:119-160 (the game's list, the squares per spot), crates/bot-protocol/src/messages.rs:88-93, 156-170, run/data/maps/*.sd7 mapinfo.lua], game: 54199a0d, claims: [K-maps-quicksilver-spot-value-and-wind]}
status: live
-->
### Metal spots and where an extractor may stand

Metal lies in the map's metal map, a byte per 16-elmo square worth the byte times the map's metal scale (Comet
Catcher 0.75, Quicksilver 1.30, Center Command 1.00, Great Divide 1.20, All That Glitters 2.05, Feast of Hades 2.9). A
spot is a cluster of such squares the game finds at load (strips of metal joined where they touch within a square),
and its worth is the sum of its squares; the spot list the reports number (spot_1, spot_2, ...) is the game's own,
read from its game rules, so our numbers, the replay cards' and the players' snapping widget's agree. Spots differ
within a map where their clusters do: a tier-1 extractor draws 2.30 a second on every Comet Catcher spot but 1.83 to
3.18 on All That Glitters, 0.49 to 2.00 on Mithril Mountain, 1.45 to 2.58 on Eclipsed (the sheet lists each map's
lowest, median and highest). An extractor draws from every metal square within the map's extractor radius of it
(Comet Catcher, Quicksilver and Altored Divide 120; Cape Violet, Great Divide, Gecko Isle and Carrot Mountains 90;
Feast of Hades 80; Isidis Crack 75; Center Command, Mithril Mountain and All That Glitters 50), so the radius sets
how far off a spot's centre an extractor may sit and still take the whole spot: the game refuses any extractor order
whose radius does not cover every square of the nearest spot, and any second extractor on a spot an allied one
already holds (an upgrade in place excepted), so a tier-1 extractor cannot be placed a little off for cover and no
spot ever holds two. Nothing but an extractor is refused on a spot: a turret or a solar built there blocks the
spot until reclaimed.

<!--meta
id: F-map-build
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Game/GameHelper.cpp:984-1000 (snapping), 1190-1262 (GetBuildHeight), 1419-1500 (TestBuildSquare: features, units, yardmaps), 1582-1640 (CheckTerrainConstraints), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:423-430 (maxHeightDif = 40 tan(maxSlope); water depths), 609, 692 (levelGround), 847-848 (geothermal yardmap), upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:575-577 (maxslope times 1.5 for every def), upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Builder.cpp:205-235, 690-712, 854-867 (levelling the footprint), upstream/Beyond-All-Reason/units (maxslope: 276 defs at 10, 81 at 20, 75 at 18, 61 at 16, 50 at 15; maxwaterdepth 0 in 449 defs; armmex.lua no depth limit, category CANBEUW; armtide.lua minwaterdepth 20; armsy.lua 30; armfhp.lua 5; armgeo.lua maxwaterdepth 5), run/data/maps/great_divide_v1.sd7 mapinfo.lua (notDeformable true)], game: 54199a0d}
status: live
-->
### Where a building may stand

A building's footprint is in 16-elmo squares (a light laser tower 2 by 2, so 32 elmos; a wind turbine or
construction turret 3; an extractor 4; a solar 5; a fusion 6 by 5; a lab or a plant 6 by 6, 96 elmos), snapped to that grid.
Its floor is the average height of its footprint's rim, and it may stand only where every square of the footprint
lies within a tolerance of that floor: the written slope limit times one and a half (the game scales every slope
limit so) turned into a height, 40 times the tangent. For a solar, a wind turbine, a construction turret or a fusion
(limit 10, so 15 degrees) that is 10.7 elmos of rise across the footprint; a light laser tower 15.4; a lab or plant
16.6; a geothermal plant 23; an extractor 40, which is why extractors stand on ledges nothing else will. Where the
ground is within tolerance but not flat, the builder first levels the footprint to the floor height, a step paid at
its build power and costing at least three times the height moved summed over the footprint's corners: up to about
ten seconds for a constructor of 90 on a solar site that rises its whole tolerance, three for the commander, nothing
on flat ground. Great Divide, the one map we play whose ground cannot be deformed, skips the step and the building
stands on the ground as it is. Nearly every land building is limited to a water depth of zero, but a tier-1 extractor has no depth limit and stands on the sea
floor at any depth, which is how the under-water spots of Cape Violet and Quicksilver are taken by an amphibious,
hover or ship constructor; a tidal generator needs 20 of water, a hover platform 5, a shipyard 30; a geothermal plant
stands only over a vent. A feature in the footprint (a tree, a rock, a wreck) blocks the site until reclaimed, and a
builder told to build there reclaims it first; another unit in the footprint makes the builder wait, or is pushed
off if it is mobile.

<!--meta
id: F-map-features
layer: fact
premises: [F-eco-reclaim]
sources: {code: [upstream/Beyond-All-Reason/features/enginetrees_override.lua (250 energy, reclaimTime 1500, blocking), features/rocks30.lua (metal 9 plus the number, 10 to 39), features/geovent.lua (geothermal, indestructible, 0), run/data/maps/great_divide_v1.sd7 features/ (fir 250 energy, rock 10 metal), center_command_bar_v1.0.sd7 features/ (kapok 300, oak 150), upstream/RecoilEngine/rts/Sim/Features/FeatureDefHandler.cpp:118 (reclaimTime defaults to six times metal plus energy), upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:261-310, upstream/Beyond-All-Reason/luarules/gadgets/unit_reclaim_fix.lua:31-70 (the same bonus as for wrecks, on any feature), upstream/Beyond-All-Reason/gamedata/modrules.lua:88-90 (featureVisibility 3), upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:136-150], game: 54199a0d, claims: [K-maps-features-are-early-energy]}
status: live
-->
### Trees, rocks, vents

A map's trees hold energy and no metal (the game's standard tree 250, a map's own 150 to 300), its rocks metal and
no energy (10 to 39 each), and a geothermal vent nothing: it is indestructible and only marks where a geothermal
plant may stand. Reclaiming a feature takes its reclaim time (six times its resources: 1,500 for a tree) over 1.05
times the reclaimer's build power plus 135, the same bonus the game gives wreck reclaim, so a constructor of 90
empties a tree in 6.5 seconds (38 energy a second, twice a solar) and the commander in 3.3 (75 a second), and a
rock of 20 metal goes in half a second. Trees block movement until they are gone. Every feature on the map is visible
to every side at all times, wrecks included: a fresh wreck is a report of a fight to whoever looks, anywhere.

<!--meta
id: F-map-sight
layer: fact
premises: [F-fight-sight]
sources: {code: [upstream/Beyond-All-Reason/gamedata/modrules.lua:59-61 (losMipLevel 3, airMipLevel 4, radarMipLevel 2), upstream/RecoilEngine/rts/Sim/Misc/LosHandler.cpp:83-100 (mipDiv = 8 << level; the mip height map), 159-162 (the eye at midPos plus losHeight, floored at 0), upstream/RecoilEngine/rts/Sim/Misc/LosMap.cpp:652, 741 (heights below 0 count as 0)], game: 54199a0d}
status: live
-->
### Sight and the shape of the ground

The fight model says sight and radar are ray casts over the ground from the unit's eye. The ground they are cast over
is coarse: sight is computed on 64-elmo cells of an averaged height map, radar on 32-elmo cells and sight of
aircraft on 128, so a ridge or a cut narrower than that is partly averaged away and hides less than it looks, while a
hill of a few hundred elmos hides everything behind it exactly as the map shows. Water counts as ground at the
surface for sight: a unit on the sea floor sees over the water as if standing on it, and the sea bed never blocks.
The eye's height is the unit's middle plus its sight height, so a tall unit or one on a rise sees over what a low
one cannot.

<!--meta
id: F-map-weather
layer: fact
premises: [F-eco-energy]
sources: {code: [upstream/RecoilEngine/rts/Map/MapInfo.cpp:101-107, 135-136 (gravity, tidal, wind read from the map), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1094-1098 (wind and tidal income), upstream/Beyond-All-Reason/modoptions.lua:609-625 (map_tidal: unchanged, 13, 18, 23), 1144-1150 (map_waterlevel), upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:1046-1066 (every gravity-affected weapon without its own gravity gets 0.1445 an elmo a frame squared, 130 a second squared; the missile ships exempt), upstream/Beyond-All-Reason/units (a census of the 351 cannon weapons: 207 carry the flag and take the standard, 35 set their own, 109 neither, of which nearly all are sound and footstep effects and the rest Legion's and the naval gauss turret's, which fall under the map's), run/data/maps/*.sd7 mapinfo.lua], game: 54199a0d, claims: [K-maps-read-wind-at-start, K-maps-tidal-is-poor-here, K-maps-quicksilver-spot-value-and-wind (the 12.8 measured agrees with three quarters of 17)]}
status: live
-->
### Wind, tide, gravity

Each map sets its wind's range and its tidal strength; the economy says a turbine makes the current wind, which
averages about three quarters of the range's top (the 12.8 a turbine measured over our Quicksilver games agrees with
three quarters of its 17), and a tidal generator makes the tidal strength, steady, wherever 20 of water lets it
stand. The lobby may override the tide (13, 18 or 23 a second) and raise or lower the water line; our lobbies have
not. Gravity differs between maps (70 on Comet Catcher, 120 on Quicksilver, 150 on Gecko Isle) but does not reach
the game's guns: every Armada and Cortex cannon flies under a standard gravity the game sets whatever the map (130
elmos a second squared) or under its own (the Guardian's high arc), never the map's, so a shell's arc and flight time
are the same on every map; only a few Legion cannons and Armada's naval gauss turret fly under the map's own.

<!--meta
id: F-map-starts
layer: fact
sources: {code: [crates/bot-protocol/src/messages.rs:75-84 (start boxes, start_pos_type), crates/bot/src/brain/briefing.rs:1038-1047 (what the report says of the boxes), crates/arena/src/script.rs:6-18, 194-207 (the lobby's boxes copied into startboxes.dat; type 3 when the arena places us, 2 otherwise), crates/arena/src/place.rs:1-5, upstream/Beyond-All-Reason/common/lib_startpoint_guesser.lua:4-5, 145-200 (the guesser's choice among free spots in the box), upstream/Beyond-All-Reason/luarules/gadgets/game_initial_spawn.lua:473-480, 656-660, run/data/maps/*.sd7 mapinfo.lua (teams startPos)], game: 54199a0d, claims: [K-maps-barb-starts-where-the-guesser-puts-it, K-map-lobby-boxes-not-honoured, K-maps-mirror-start-is-a-beach, K-team-allies-are-invisible-by-default]}
status: live
-->
### Where the sides start

A map file lists fixed start positions, but the lobbies we play in use start boxes: each ally team gets a rectangle
(the lobby's saved boxes per map and team count; on Comet Catcher two full-height strips a fifth of the map wide at
the west and east edges, on Quicksilver two full-width strips) and every seat starts somewhere inside its box. A
person chooses the point; an AI seat with no point is placed by the game's guesser at the free metal spot in its box
whose neighbours within 575 elmos and 300 of height score best, which is a cluster's middle, not the box's centre and
not the mirror of ours (on Comet Catcher's strips that is the north end of the east strip). The script tells nobody
where another team started: an enemy's start is known only as "inside his box" until something of ours sees it, and
an ally's when its commander is seen. The host may also place seats by hand, in which case the boxes mean nothing,
and the report says so when our own start lies in another team's box or outside ours. In the arena, when the search
places our commander, every start is fixed in the script; otherwise the game places all seats as above.

<!--meta
id: F-map-catalogue
layer: fact
sources: {code: [docs/briefs/reference/map-sheets.md (run/map_sheets.py over the latest recorded game of each map; the sheet's tidal reads 0 for Quicksilver, Mithril Mountain and Feast of Hades from records older than the field, and Feast of Hades' draw is unknown for the same reason: the maps' files are used for those), run/data/maps/*.sd7 mapinfo.lua (the metal scale, extractor radius, wind, tide, deformability), crates/terrain/src/sheet.rs (walks, passable share, the ways' width)], game: 54199a0d, claims: [K-maps-quicksilver-facts, K-map-cape-violet-v1-a-water-map-with-a-third-of-the-spots-under-water]}
status: live
-->
### The maps we have played

One row per map with a recorded game. "Walk" is the sheet's start-to-start walk in seconds for a Pawn and a Blitz
(bots and vehicles, the pool's or our two starts); "stand" is the share of the map each class can stand on (bots /
vehicles / hovercraft); "tightest" is the narrowest the routes between the starts get for vehicles, in elmos, against
the routes' median width (a pass is a tightest a fraction of the median). A dash is a map with only one recorded start, whose sheet has
no walk. Draw is a tier-1 extractor's metal a second on the lowest, median and highest spot.

| Map | Elmos | Spots | Draw | Radius | Wind | Tide | Water | Stand | Walk | Tightest |
|---|---|---|---|---|---|---|---|---|---|---|
| Comet Catcher Remake 1.8 | 8,192 x 6,144 | 80 | 2.30 all | 120 | 1-4 | 0 | 0% | 98 / 93 / 94 | 68 / 59 | 2,768 of 3,748 (open) |
| Quicksilver Remake 1.24 | 7,168 x 7,168 | 44 | 2.00 all | 120 | 3-17 | 10 | 49% | 47 / 42 / 93 | 40 / 34 | 1,676 of 2,036 |
| Great Divide V1 | 3,072 x 4,096 | 22 | 2.12 / 2.17 / 2.32 | 90 | 0-20 | 16 | 0% | 94 / 87 / 89 | 41 / 35 | 744 of 1,700 (a pass at E5) |
| Center Command BAR v1.0 | 8,192 x 4,096 | 48 | 1.79 / 1.80 / 1.81 | 50 | 1-19 | 75 | 2% | - | - | - |
| Cape Violet V1 | 10,240 x 5,120 | 72 | 1.99 / 2.07 / 2.24 | 90 | 8-20 | 20 | 40% | 58 / 51 / 87 | 117 / 106 | 728 of 1,664 (at G6) |
| Gecko Isle Remake v1.2.1 | 8,192 x 9,216 | 88 | 1.68 / 2.48 / 2.48 | 90 | 2-16 | 25 | 30% | 66 / 62 / 92 | 93 / 81 | 2,000 of 4,808 |
| All That Glitters v2.2.3 | 6,144 x 10,240 | 152 | 1.83 / 1.84 / 3.18 | 50 | 0-16 | 0 | 0% | 92 / 85 / 86 | 107 / 92 | 1,968 of 4,520 |
| All That Glitters Extended v1.0.2 | 15,360 x 10,240 | 437 | 1.83 / 1.84 / 3.18 | 50 | 0-16 | 0 | 0% | - | - | - |
| Altored Divide Bar Remake 1.6.2 | 8,192 x 8,192 | 84 | 1.99 all | 120 | 4-19 | 10 | 0% | 95 / 84 / 88 | 97 / 87 | 2,772 of 3,992 |
| Isidis crack 1.1 | 7,168 x 7,168 | 82 | 1.84 all | 75 | 2-20 | 0 | 0% | 98 / 92 / 93 | 83 / 73 | 3,444 of 4,868 |
| Mithril Mountain v2.0.1 | 6,144 x 6,144 | 53 | 0.49 / 2.00 / 2.00 | 50 | 2-17 | 10 | 0% | 99 / 95 / 96 | 67 / 63 | 4,028 of 4,852 |
| Carrot Mountains v2.0 | 12,288 x 12,288 | 172 | 1.85 / 2.37 / 2.39 | 90 | 6-12 | 20 | 0% | 99 / 95 / 97 | 105 / 101 | 4,076 of 5,404 |
| Feast of Hades 1.0.1 | 8,192 x 8,192 | 92 | (unknown; scale 2.9) | 80 | 4-14 | 15 | 7% | - | - | - |
| Avalanche 3.4 | 4,096 x 4,096 | 19 | 1.99 all | ? | 3-19 | 20 | 0% | - | - | - |
| Azurite Shores 1.0.2 | 9,216 x 8,192 | 76 | 2.45 all | ? | 1-20 | 20 | 26% | - | - | - |
| Coast To Coast BAR v1.0 | 6,144 x 4,096 | 24 | 1.41 / 2.16 / 2.17 | ? | 5-20 | 16 | 60% | - | - | - |
| Eclipsed 0.9.8 | 7,168 x 7,168 | 71 | 1.45 / 1.45 / 2.58 | ? | 0-10 | 18 | 13% | - | - | - |
| Flats and Forests v2.2 | 12,288 x 12,288 | 204 | 2.06 all | ? | 4-12 | 15 | 0% | - | - | - |
| SailAway 2 | 10,240 x 10,240 | 96 | 1.95 all | ? | 5-20 | 0 | 69% | - | - | - |

Great Divide is the one map whose ground cannot be deformed. Quicksilver is an island whose starts share the land
(the sea surrounds it; the sea floor is walkable by the commander and the amphibious units, its seabed type at 0.8 of
speed and the depth slowing further). Cape Violet's land routes west run through two passes 560 to 660 wide and a
ford 14 to 20 deep, 28 of its 72 spots are off the ground (26 under water), and its ways narrow to 730 at G6, under
half their median. Center Command's tide is
the strongest we play (75 a second from a 90-metal generator wherever a pond is 20 deep). Comet Catcher's wind is
the weakest (about 3 a second a turbine).

<!--meta
id: F-map-consequences
layer: fact
premises: [F-map-grid, F-map-ground, F-map-metal, F-map-build, F-map-features, F-map-sight, F-map-weather, F-map-starts, F-map-catalogue, F-eco-energy, F-eco-consequences, F-fight-range]
status: live
-->
### What follows directly

- **The clock of a map is its walk.** A raider reaching the other start on Comet Catcher is 59 seconds out as a
  Blitz, 68 as a Pawn, and half that to the middle; on Great Divide 35 and 41; on Cape Violet 106 and 117 by land,
  96 as a hovercraft. Whatever is to be at his base by a given minute leaves that many seconds earlier, and a unit
  that turns back has spent the walk twice.
- **Energy by map, from the economy's prices.** A solar is 7.75 metal per energy a second. A turbine is 40 over the
  expected wind: about 13 on Comet Catcher (1 to 4), about 3.1 on Quicksilver (3 to 17), 2.8 on Center Command, 2.6
  on Great Divide, 3.3 on Gecko Isle and All That Glitters; so solars on Comet Catcher and turbines nearly everywhere
  else, with the swing of the wind to be covered by storage. A tidal generator is 90 over the tide: 1.2 on Center
  Command, 3.6 on Gecko Isle, 4.5 on Cape Violet, 5.6 on Great Divide, 9 on Quicksilver; it needs 20 of water.
- **Spots within reach set the opening's ceiling.** The sheet counts spots within 30, 60, 90 and 120 seconds of each
  start for the commander, a construction bot and a construction vehicle: on Comet Catcher from the west start 37
  within a minute for a construction vehicle against 21 for the commander and 19 for a construction bot (11 against
  5 within half a minute), so the vehicle plant's first constructor takes the strip at nearly twice the commander's
  pace and the bot lab's at the commander's.
- **Where a map is open, the ways are wide.** Comet Catcher's tightest way is about three quarters of the median:
  nothing funnels an army, and a force is met wherever the other side chooses. Great Divide's and Cape Violet's
  narrows (744 and 730 elmos, under half their ways' median) are where a column strings out; whoever holds the far
  mouth meets the column piecemeal.
- **Ledges are bot ground.** Bots climb what tanks cannot (80 against 40 degrees), and climbing is slower than
  descending; a bot force on a ledge above a tank route cannot be reached by the tanks and sees over them, with the
  beam lasers' full height bonus and the cannons' shortened upward reach from the fight model.
- **Sight is coarse and water is flat.** A ridge narrower than 64 elmos hides nothing; a unit in shallow water is
  seen as if on the surface. Hills of hundreds of elmos are the only cover, and the eye on top of one sees far.
- **Under-water spots belong to whoever brings the right constructor.** A tier-1 extractor stands on the sea floor,
  so on Cape Violet (26 spots under water), Quicksilver (49 percent sea) and Gecko Isle the amphibious, hover or ship
  constructor takes a third of the map that land constructors cannot, and a land army cannot raid those extractors
  either.
- **The enemy's start is a cluster in his box, not a point.** The guesser puts an AI seat at the best-connected free
  spot in its box; a person chooses, usually near the same cluster. Scouting the box's metal clusters finds him;
  walking to the map's mirror point finds a beach.
- **Trees are a second energy store.** At 38 energy a second a constructor against a tree, four trees are the 1,000 of
  the starting store in under half a minute of one constructor's time; a map's tree line near the start (Great
  Divide's firs, Center Command's kapoks at 300) is energy with no build time.
