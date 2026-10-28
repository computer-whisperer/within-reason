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
sources: {code: [upstream/RecoilEngine/rts/Sim/Misc/GlobalConstants.h:24-45 (SQUARE_SIZE 8, BUILD_SQUARE_SIZE 16, 8 elmos a metre), upstream/RecoilEngine/rts/Map/ReadMap.cpp:741-765 (the slope map at half resolution), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/MoveMath.cpp:107-140, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:442-443 (speed in elmos a second), crates/bot/src/world.rs:49-52 (the 8 by 8 grid), crates/ai-host/src/engine.rs:284-310 (our terrain at 16 elmos), crates/terrain/src/lib.rs:20-45 (walking cost: slopes priced in both directions, no depth or terrain-type term)], game: 54199a0d}
status: live
-->
### Distances, the grid, the clock

The unit of distance is the elmo; eight make a metre, so Comet Catcher's 8,192 by 6,144 is a kilometre by three
quarters. The ground is a height map with a sample every 8 elmos; slope, terrain type and metal are kept at half
that, one value per 16-elmo square, and buildings snap to that 16-elmo grid. The speeds in the unit tables are elmos
a second on flat ground, so a walk in elmos over a speed is seconds: at 100 a second a unit crosses Comet Catcher's
long side in 82 seconds and a map of 12,288 in two minutes. The reports name places by an 8 by 8 grid of the map,
columns A to H west to east and rows 1 to 8 north to south, so a cell is an eighth of the map each way (1,024 by 768
on Comet Catcher, 896 square on Quicksilver) and the cell says nothing finer. The sheet's walks sum the engine's
slope law along the cheapest route (flat ground at cost one, a slope at one plus the slope times the class's factor),
so they are close to the seconds the engine takes, not straight lines; they price a slope in both directions where
the engine charges only climbing, and leave out the slowing of water and of a map's painted terrain types, so they
run long over hills and short through fords.
<!--meta
id: F-map-ground
layer: fact
premises: [F-fight-ground]
sources: {code: [upstream/Beyond-All-Reason/gamedata/movedefs.lua:17-92 (DEPTH, SLOPE, SLOPE_MOD, the depth curves), 110-360 (the classes: depthMod 0 on the amphibious bots and tanks), 545-556 (setMaxSlope: the written angle divided by 1.5 before the engine's 1.5, slopeMod 4 for every class named BOT), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveDefHandler.cpp:86-96, 170-176, 195-260, 347, 744-770 (GetDepthMod; the game's maxValue key is not read, maxScale is), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/GroundMoveMath.cpp:12-52, HoverMoveMath.cpp:12-38, ShipMoveMath.cpp:11-28, MoveMath.cpp:81-140 (terrain-type multipliers; the direction term), 205-217 (crushing), upstream/RecoilEngine/rts/Sim/MoveTypes/GroundMoveType.cpp:1305-1313 (movement uses the directional modifier; pathing does not: IPathManager.h:29), upstream/RecoilEngine/rts/Sim/Path/QTPFS/PathSearch.cpp:1240-1262 (partial paths), upstream/RecoilEngine/rts/Sim/Features/FeatureDefHandler.cpp:142-149 (a feature's resistance is its mass, 0.4 of its metal plus a tenth of its health), upstream/Beyond-All-Reason/units/ArmBots/armpw.lua (wreck and heap blocking false), upstream/Beyond-All-Reason/gamedata/modrules.lua:98 (QTPFS), run/data/maps/*.sd7 mapinfo.lua terrainTypes and mapconfig/mapinfo/0_apply_options.lua, maps/*.smf type maps (parsed 2026-10-22: Cape Violet uses its default type on 94.7 percent and its 1.3 type on 5.1, nearly all sea; Quicksilver seabed 45, grass 40, rock 12, sand 2; Gasbag Grabens toxic waste 35), run/matches/1792636232-snapshot-smoke/00/record-0.jsonl (the hello's move_classes: bots 54.0 degrees, tanks 27.0, hovercraft 33.0)], game: 54199a0d, claims: [K-maps-terrain-not-straight-lines, K-maps-early-walks-nearly-straight, K-maps-quicksilver-is-an-island, K-maps-factory-by-terrain, K-maps-mirror-start-is-a-beach]}
status: live
-->
### Ground: how terrain changes speed

The fight model gives each class's limits (tanks climb to 27 degrees, hovercraft to 33, bots and the commander to 54;
tanks and bots wade to 20 deep, the commander and the amphibious units walk the sea floor, ships need 8). Short of
the limit, a slope slows a unit by one over one plus the slope times its class's factor, where the slope is one minus
the cosine of the ground's angle and the factor is 4 for every bot class and 18 to 42 for the tank classes: on a 20
degree slope a bot runs at 81 percent of its speed and a Stout at 48. Only climbing counts: the movement code charges
the slope by how much of it lies ahead, so going uphill is slower and going downhill or across is the flat speed
(the pathfinder, which prices every square as if it were climbed, does not know this and routes as if descending
were slow too). Water slows the waders from four deep by one over one plus three hundredths of the depth (at 10 deep
77 percent of speed, at 20 deep 62); the commander by a gentler curve floored at two thirds (92 percent at 20 deep,
two thirds from 47 down); the amphibious bots and tanks, hovercraft and ships are not slowed by water at all. Each
map paints its ground in terrain types that multiply speed on top of this: on Comet Catcher, Great Divide, Center
Command, Gecko Isle, Isidis Crack, Mithril Mountain, Carrot Mountains, Feast of Hades and All That Glitters every
type is one, so only slopes matter; on Quicksilver the seabed (45 percent of the map, all of it sea) is 0.8 for tanks
and bots and 0.9 for hovercraft, its rock (12 percent) 0.7 and 0.8, its grass 1.05; Altored Divide's sand (9 percent)
is 0.9 and its rocky ground (5 percent) 1.1 for tanks and bots, hovercraft unchanged; Cape Violet's file names three
roads but paints none on land (its one fast type, 1.3 for every class, covers 5 percent of the map, nearly all of it
sea). Gasbag Grabens paints a toxic waste no class can enter over 35 percent of the map, two thirds of it its water,
and its water hurts units 1 a second; no other map we play damages units in water, though Great Divide and Cape
Violet carry a map option the host can switch on that makes it 50. Where a unit cannot go, the engine's quad-tree
pathfinder routes it round, through gaps at least its footprint wide, and a column strings out at every narrowing;
where no route reaches the goal the unit walks to the reachable point nearest it and stands there (the army on the
Quicksilver beach). Buildings block movement, and a factory's open side is its only exit. Wrecks and trees are
features, which a mover crushes and destroys when its crush strength is at least the feature's resistance (its mass:
0.4 of its metal plus a tenth of its health unless the file sets one) and is blocked by otherwise: a tier-1 wreck
resists 100 to 250, so the medium and heavy tank classes (250 and up) drive through tier-1 wrecks while the
commander (50) and the bots go round; a Pawn's wreck and every heap block nothing.
<!--meta
id: F-map-metal
layer: fact
premises: [F-eco-extractors]
sources: {code: [upstream/RecoilEngine/rts/Map/MetalMap.cpp:29-44, upstream/Beyond-All-Reason/common/upgets/api_resource_spot_finder.lua:24-60, 120-200 (the strips, IsBuildingPositionValid: the radius plus one square of slack), 232-300 (GetSpotsMetal, the spot's worth), 300-330 (mex_count game rules), upstream/Beyond-All-Reason/luarules/gadgets/cmd_mex_denier.lua:30-100 (the duplicate check is per ally team), upstream/RecoilEngine/rts/Map/MapInfo.cpp:106-107 (maxMetal, extractorRadius), crates/ai-host/src/engine.rs:119-160 (the game's list, the squares per spot), crates/bot-protocol/src/messages.rs:88-93, 156-170, run/data/maps/*.sd7 mapinfo.lua, run/metal_killtest.py (spot_spacing: the nearest-neighbour distances over a record's metal_spots)], game: 54199a0d, claims: [K-maps-quicksilver-spot-value-and-wind]}
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
lowest, median and highest). Spots also differ in how they lie: Comet Catcher's 80 stand 400 / 476 / 595 elmos from their
nearest neighbour (quartiles), the closest pair 229 apart, four spots with a neighbour within 300, and a start has four
within 650 of it (the metal spots of a Comet record of ours, `run/metal_killtest.py openers`), so the map has pairs
where the team maps played as duels have triples, and a constructor's walk between spots is part of every spot's price. An extractor draws from every metal square within the map's extractor radius of it
(Comet Catcher, Quicksilver and Altored Divide 120; Cape Violet, Great Divide, Gecko Isle and Carrot Mountains 90;
Feast of Hades 80; Isidis Crack 75; Center Command, Mithril Mountain and All That Glitters 50; Gasbag Grabens 40), so
the radius sets how far off a spot's centre an extractor may sit and still take the whole spot: the game refuses any
extractor order whose radius (plus one square of slack) does not cover every square of the nearest spot, and any
second extractor on a spot the same side already holds (an upgrade in place excepted), so a tier-1 extractor cannot
be placed a little off for cover and no spot holds two of one side's. Nothing but an extractor is refused on a spot:
a turret or a solar built there blocks the spot until reclaimed.
<!--meta
id: F-map-build
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Game/GameHelper.cpp:984-1000 (snapping), 1190-1262 (GetBuildHeight: the floor is the centre square's height, xsize and zsize 1), 1419-1500 (TestBuildSquare: features, units, yardmaps), 1582-1640 (CheckTerrainConstraints: each square within maxHeightDif of the floor either way), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:423-430 (maxHeightDif = 40 tan(maxSlope); water depths), 609, 692 (levelGround), 847-848 (geothermal yardmap), upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:575-577 (maxslope times 1.5, rounded, for every def), 708-714 (terraformspeed set to thirty times the build power), upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Builder.cpp:114, 205-235, 849-867, 890-910 (levelling the footprint; the cost at least three times the height moved), upstream/Beyond-All-Reason/units (maxslope: 276 defs at 10, 81 at 20, 75 at 18, 61 at 16, 50 at 15; maxwaterdepth 0 in 449 defs; armmex.lua no depth limit, category CANBEUW; armtide.lua minwaterdepth 20; armsy.lua 30; armfhp.lua 5; armgeo.lua maxwaterdepth 5; corllt.lua maxslope 10, corgeo.lua 15; armvp.lua:18, corvp.lua:19 levelground false), run/data/maps/great_divide_v1.sd7 mapinfo.lua (notDeformable true)], game: 54199a0d}
status: live
-->
### Where a building may stand

A building's footprint is in 16-elmo squares (a light laser tower 2 by 2, so 32 elmos; a wind turbine or
construction turret 3; an extractor 4; a solar 5; a fusion 6 by 5; a lab or a plant 6 by 6, 96 elmos), snapped to
that grid. Its floor is the ground's height at its centre, and it may stand only where every square of the footprint
lies within a tolerance of that floor, above or below: the written slope limit times one and a half (the game scales
every slope limit so, rounding to whole degrees) turned into a height, 40 times the tangent. For a solar, a wind
turbine, a construction turret or a fusion (limit 10, so 15 degrees) that is 10.7 elmos either way, so a solar can
stand on a footprint that rises 21 across; an Armada light laser tower 15.4 (Cortex's 10.7); a lab or plant 17; a
geothermal plant 23 (Cortex's 17); an extractor 40, which is why extractors stand on ledges nothing else will. Where
the ground is within tolerance but not flat, the builder first levels the footprint to the floor height; the game
sets that work at thirty times the builder's build power, so it takes a fraction of a second even on a footprint at
the tolerance's edge. Vehicle plants, walls and dragon's teeth never level and sit on the ground as it is, and on
Great Divide, the one map we play whose ground cannot be deformed, nothing does. Nearly every land building is
limited to a water depth of zero, but a tier-1 extractor has no depth limit and stands on the sea floor at any
depth, which is how the under-water spots of Cape Violet and Quicksilver are taken: by an amphibious, hover or ship
constructor, or by a land constructor on the shore within its build distance of 130; a tidal generator needs 20 of
water, a hover platform 5, a shipyard 30; a geothermal plant stands only over a vent. A blocking feature in the
footprint (a tree, a rock, most wrecks; not a heap, not a Pawn's wreck) blocks the site until reclaimed, and a
builder told to build there reclaims it first; another unit in the footprint makes the builder wait, or is pushed
off if it is mobile.
<!--meta
id: F-map-features
layer: fact
premises: [F-eco-reclaim]
sources: {code: [upstream/Beyond-All-Reason/features/enginetrees_override.lua (250 energy, reclaimTime 1500, blocking, mass 20), features/rocks30.lua (metal 9 plus the number, 10 to 39), upstream/RecoilEngine/rts/Sim/Features/FeatureDefHandler.cpp:118 (reclaimTime defaults to six times metal plus energy), 142-149 (mass and resistance), 187-191 (the engine's geothermal vent: non-blocking, indestructible, unreclaimable), run/data/maps/*.sd7 features/ (a census over the ten archives that ship features: Great Divide pines 50 to 300 and firs 250, Gasbag's trees 100 to 350, Cape Violet's cedars 200 and non-blocking, Center Command's oaks 150 and kapoks 300, rocks 10 and 35), upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:261-310, upstream/Beyond-All-Reason/luarules/gadgets/unit_reclaim_fix.lua:31-70 (the same bonus as for wrecks, on any feature), upstream/Beyond-All-Reason/gamedata/modrules.lua:88-90 (featureVisibility 3), upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:136-150], game: 54199a0d, claims: [K-maps-features-are-early-energy]}
status: live
-->
### Trees, rocks, vents

A map's trees hold energy and no metal (the game's standard tree 250, a map's own 50 to 350), its rocks metal and
no energy (10 to 39 each), and a geothermal vent nothing: it is indestructible and only marks where a geothermal
plant may stand. Reclaiming a feature takes its reclaim time (six times its resources: 1,500 for a tree) over 1.05
times the reclaimer's build power plus 135, the same bonus the game gives wreck reclaim, so a constructor of 90
empties a tree in 6.5 seconds (38 energy a second, twice a solar) and the commander in 3.3 (75 a second), and a
rock of 20 metal goes in half a second. The standard tree (mass 20) is crushed by nearly every unit that walks into
it, the commander and the tank and bot classes included; a map's heavy trees (Great Divide's pines) block until
reclaimed or shot down. Every feature on the map is visible to every side at all times, wrecks included: a fresh
wreck is a report of a fight to whoever looks, anywhere.
<!--meta
id: F-map-sight
layer: fact
premises: [F-fight-sight]
sources: {code: [upstream/Beyond-All-Reason/gamedata/modrules.lua:54 (requireSonarUnderWater), 59-61 (losMipLevel 3, airMipLevel 4, radarMipLevel 2), upstream/RecoilEngine/rts/Sim/Misc/LosHandler.cpp:83-100 (mipDiv = 8 << level; the mip height map; aircraft sight a plain circle), 159-162 (the eye at midPos plus losHeight, floored at 0, in 32-elmo steps), 895-899, upstream/RecoilEngine/rts/Sim/Misc/LosMap.cpp:652, 741 (heights below 0 count as 0), upstream/RecoilEngine/rts/Map/ReadMap.cpp:647-667 (each mip level the mean of two by two)], game: 54199a0d}
status: live
-->
### Sight and the shape of the ground

The fight model says sight and radar are ray casts over the ground from the unit's eye. The ground they are cast over
is coarse: sight is computed on 64-elmo cells of a height map averaged down from the real one, radar on 32-elmo
cells, so a ridge or a cut narrower than a cell is diluted into its cell's average (a ridge 32 wide and 100 high
raises its cell by about 50) and hides less than it looks, while a hill of a few hundred elmos hides everything
behind it exactly as the map shows; aircraft are seen in a plain circle that no ground blocks. Water counts as
ground at the surface for sight: a unit on the sea floor sees over the water from the surface, and the sea bed never
blocks a line; but a fully submerged unit is itself unseen without sonar or radar on it. The eye's height is the
unit's middle plus its sight height, in steps of 32, so a tall unit or one on a rise sees over what a low one cannot.
<!--meta
id: F-map-weather
layer: fact
premises: [F-eco-energy]
sources: {code: [upstream/RecoilEngine/rts/Map/MapInfo.cpp:101-107, 135-136 (gravity, tidal, wind read from the map), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1094-1098 (wind and tidal income), upstream/Beyond-All-Reason/modoptions.lua:609-625 (map_tidal: unchanged, 13, 18, 23), 1144-1150 (map_waterlevel), upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:1046-1066 (every gravity-affected weapon without its own gravity gets 0.1445 an elmo a frame squared, 130 a second squared; the Armada and Cortex missile ships exempt by name), upstream/RecoilEngine/rts/Sim/Weapons/Cannon.cpp:35 (a weapon without its own falls under the map's), upstream/Beyond-All-Reason/units (a census of the 351 cannon weapons: 207 carry the flag and take the standard, 35 set their own, 109 neither, of which nearly all are sound and footstep effects and the rest the missile ships' rockets, Armada's naval gauss turret and a few Legion cannons), upstream/RecoilEngine/rts/Sim/MoveTypes/GroundMoveType.cpp:525, 1550-1601, upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:604 (map gravity on falling units and wrecks), run/data/maps/*.sd7 mapinfo.lua], game: 54199a0d, claims: [K-maps-read-wind-at-start, K-maps-tidal-is-poor-here, K-maps-quicksilver-spot-value-and-wind (the 12.8 measured agrees with three quarters of 17)]}
status: live
-->
### Wind, tide, gravity

Each map sets its wind's range and its tidal strength; the economy says a turbine makes the current wind, which
averages about three quarters of the range's top (the 12.8 a turbine measured over our Quicksilver games agrees with
three quarters of its 17), and a tidal generator makes the tidal strength, steady, wherever 20 of water lets it
stand. The lobby may override the tide (13, 18 or 23 a second) and raise or lower the water line; our lobbies have
not. Gravity differs between maps (70 on Comet Catcher, 120 on Quicksilver, 150 on Gecko Isle) but hardly reaches
the game's guns: every Armada and Cortex cannon flies under a standard gravity the game sets whatever the map (130
elmos a second squared) or under its own (the Guardian's high arc), so a shell's arc and flight time are the same on
every map; only the missile ships' rockets, Armada's naval gauss turret and a few Legion cannons fly under the
map's own, which otherwise moves only falling units and wrecks.
<!--meta
id: F-map-starts
layer: fact
sources: {code: [crates/bot-protocol/src/messages.rs:75-84 (start boxes, start_pos_type), crates/bot/src/brain/briefing.rs:1038-1047 (what the report says of the boxes and of a fixed start), crates/arena/src/script.rs:6-18, 186-207 (the lobby's boxes copied into startboxes.dat; type 3 with every team's position when the arena places us, 2 otherwise), crates/arena/src/place.rs:1-5, upstream/Beyond-All-Reason/common/lib_startpoint_guesser.lua:3-6, 145-200, 243-258, 294-306 (the guesser's choice among free spots in the box and within 750 outside it, then a step of 120 off the spot), upstream/Beyond-All-Reason/luarules/gadgets/game_initial_spawn.lua:473-480, 656-660, run/data/maps/*.sd7 mapinfo.lua (teams startPos)], game: 54199a0d, claims: [K-maps-barb-starts-where-the-guesser-puts-it, K-map-lobby-boxes-not-honoured, K-maps-mirror-start-is-a-beach, K-team-allies-are-invisible-by-default]}
status: live
-->
### Where the sides start

A map file lists fixed start positions, but the lobbies we play in use start boxes: each ally team gets a rectangle
(the lobby's saved boxes per map and team count unless the host draws others; the saved ones on Comet Catcher are two
full-height strips a fifth of the map wide at the west and east edges, on Quicksilver two full-width strips) and
every seat starts somewhere inside its box. A person chooses the point; an AI seat with no point is placed by the
game's guesser at the free metal spot in its box (or within 750 outside it) whose neighbours within 575 elmos and
300 of height score best, stepped 120 elmos off the spot, which is a cluster's middle, not the box's centre and not
the mirror of ours (on Comet Catcher's strips that is the north end of the east strip). Under the boxes the script
tells nobody where another team started: an enemy's start is known only as "inside his box" until something of ours
sees it, while an ally's commander stands in the shared sight from the first frame. The host may also place seats by
hand, in which case the boxes mean nothing, and the report says so when our own start lies in another team's box or
outside ours. When the starts are fixed instead (the map's positions, or positions chosen before the game, as in the
arena when the search places our commander) every team's position is in the script and the report gives the enemy's
start exactly.
<!--meta
id: F-map-catalogue
layer: fact
sources: {code: [docs/briefs/reference/map-sheets.md (run/map_sheets.py over the latest recorded game of each map, regenerated 2026-10-22 with Gasbag Grabens and Project SD-129; the sheet's tidal reads 0 for Quicksilver, Mithril Mountain and Feast of Hades from records older than the field, and Feast of Hades' draw and walks are unknown for the same reason: the maps' files are used for those), run/data/maps/*.sd7 mapinfo.lua (the metal scale, extractor radius, wind, tide, deformability, water damage), crates/terrain/src/sheet.rs (walks, passable share, the ways' width), maps/*.smf type maps (Gasbag Grabens' waste); Boreal Falls and Eye of Horus from `run/map_sheets.py --print` of 2026-10-27 over run/matches/1792809574-human and 1792810743-human, their radius from the records' headers], game: 54199a0d, claims: [K-maps-quicksilver-facts, K-map-cape-violet-v1-a-water-map-with-a-third-of-the-spots-under-water]}
status: live
-->
### The maps we have played

One row per map with a recorded game. "Walk" is the sheet's start-to-start walk in seconds for a Pawn and a Blitz
(bots and vehicles, the pool's or our two starts); "stand" is the share of the map each class can stand on (bots /
vehicles / hovercraft); "tightest" is the narrowest the routes between the starts get for vehicles, in elmos, against
the routes' median width (a pass is a tightest a fraction of the median). A dash is a map whose sheet has no walk:
one recorded start, or a record older than the sheet's fields. A question mark is a map whose archive we do not
hold. Draw is a tier-1 extractor's metal a second on the lowest, median and highest spot.

| Map | Elmos | Spots | Draw | Radius | Wind | Tide | Water | Stand | Walk | Tightest |
|---|---|---|---|---|---|---|---|---|---|---|
| Comet Catcher Remake 1.8 | 8,192 x 6,144 | 80 | 2.30 all | 120 | 1-4 | 0 | 0% | 98 / 93 / 94 | 68 / 59 | 2,768 of 3,748 (open) |
| Quicksilver Remake 1.24 | 7,168 x 7,168 | 44 | 2.00 all | 120 | 3-17 | 10 | 49% | 47 / 42 / 93 | 40 / 34 | 1,676 of 2,036 |
| Great Divide V1 | 3,072 x 4,096 | 22 | 2.12 / 2.17 / 2.32 | 90 | 0-20 | 16 | 0% | 94 / 87 / 89 | 41 / 35 | 744 of 1,700 (a pass at E5) |
| Center Command BAR v1.0 | 8,192 x 4,096 | 48 | 1.79 / 1.80 / 1.81 | 50 | 1-19 | 75 | 2% | - | - | - |
| Cape Violet V1 | 10,240 x 5,120 | 72 | 1.99 / 2.07 / 2.24 | 90 | 8-20 | 20 | 40% | 58 / 51 / 87 | 117 / 106 | 728 of 1,664 (at G6) |
| Gecko Isle Remake v1.2.1 | 8,192 x 9,216 | 88 | 1.68 / 2.48 / 2.48 | 90 | 2-16 | 25 | 30% | 66 / 62 / 92 | 93 / 81 | 2,000 of 4,808 |
| All That Glitters v2.2.3 | 6,144 x 10,240 | 152 | 1.83 / 1.84 / 3.18 | 50 | 0-16 | 0 | 0% | 92 / 85 / 86 | 107 / 92 | 1,968 of 4,520 |
| All That Glitters Extended v1.0.2 | 15,360 x 10,240 | 437 | 1.83 / 1.84 / 3.18 | ? | 0-16 | 0 | 0% | - | - | - |
| Altored Divide Bar Remake 1.6.2 | 8,192 x 8,192 | 84 | 1.99 all | 120 | 4-19 | 10 | 0% | 95 / 84 / 88 | 97 / 87 | 2,772 of 3,992 |
| Isidis crack 1.1 | 7,168 x 7,168 | 82 | 1.84 all | 75 | 2-20 | 0 | 0% | 98 / 92 / 93 | 83 / 73 | 3,444 of 4,868 |
| Mithril Mountain v2.0.1 | 6,144 x 6,144 | 53 | 0.49 / 2.00 / 2.00 | 50 | 2-17 | 10 | 0% | 99 / 95 / 96 | 67 / 63 | 4,028 of 4,852 |
| Carrot Mountains v2.0 | 12,288 x 12,288 | 172 | 1.85 / 2.37 / 2.39 | 90 | 6-12 | 20 | 0% | 99 / 95 / 97 | 105 / 101 | 4,076 of 5,404 |
| Gasbag Grabens 1.1.1 | 9,216 x 6,144 | 48 | 2.10 / 2.29 / 2.44 | 40 | 4-16 | 0 | 35% | - | - | - |
| Project SD-129 v1.0 | 14,336 x 14,336 | 148 | 1.70 / 1.72 / 3.20 | ? | 0 (as recorded) | 0 | 44% | - | - | - |
| Feast of Hades 1.0.1 | 8,192 x 8,192 | 92 | (unknown; scale 2.9) | 80 | 4-14 | 15 | 7% | - | - | - |
| Avalanche 3.4 | 4,096 x 4,096 | 19 | 1.99 all | ? | 3-19 | 20 | 0% | - | - | - |
| Azurite Shores 1.0.2 | 9,216 x 8,192 | 76 | 2.45 all | ? | 1-20 | 20 | 26% | - | - | - |
| Boreal Falls 1.0.2 | 7,168 x 7,168 | 60 | 2.04 all | 60 | 2-16 | 13 | 0% | 95 / 84 / 87 | - | - |
| Coast To Coast BAR v1.0 | 6,144 x 4,096 | 24 | 1.41 / 2.16 / 2.17 | ? | 5-20 | 16 | 60% | - | - | - |
| Eclipsed 0.9.8 | 7,168 x 7,168 | 71 | 1.45 / 1.45 / 2.58 | ? | 0-10 | 18 | 13% | - | - | - |
| Eye Of Horus 1.7.1 | 6,144 x 7,168 | 46 | 2.87 all | 90 | 1-25 | 18 | 0% | 96 / 81 / 87 | - | - |
| Flats and Forests v2.2 | 12,288 x 12,288 | 204 | 2.06 all | ? | 4-12 | 15 | 0% | - | - | - |
| SailAway 2 | 10,240 x 10,240 | 96 | 1.95 all | ? | 5-20 | 0 | 69% | - | - | - |

Great Divide is the one map whose ground cannot be deformed, and it has no water, so its tide of 16 is unusable.
Quicksilver is an island whose starts share the land (the sea surrounds it; the sea floor is walkable by the
commander and the amphibious units, its seabed type at 0.8 of speed and the commander slowed further by depth; bots
reach 38 of its 44 spots). Cape Violet's land routes west run through two passes 560 to 660 wide and a ford 14 to
20 deep, 28 of its 72 spots are off the ground (26 under water), and its ways narrow to 730 at G6, under half their
median. Gasbag Grabens has a third of its map in toxic waste no unit enters and water that hurts. Center Command's
tide is the strongest we play (75 a second from a 90-metal generator wherever a pond is 20 deep). Comet Catcher's
wind is the weakest (about 3 a second a turbine).
<!--meta
id: M-map-seats
layer: meta
population: our seats in four games against thebluegecko and u6bkep: Comet Catcher (the north-east seat G1, 2026-10-23 and 10-25), Boreal Falls (F2, 10-23), Eye of Horus (E7, 10-23), All That Glitters Extended (H8, 10-25)
dates: 2026-10-23 to 2026-10-25 (written 2026-10-27 from the live edge)
ground: model (our player's and commander's notes in the games and the players' debriefs); the wind ranges are the catalogue's
sources: {code: [run/matches/1792807358-human/00, run/matches/1792969892-human/00, run/matches/1792809574-human/00, run/matches/1792810743-human/00, run/matches/1792971120-human/00 (strategist-*.jsonl, commander.jsonl), docs/briefs/inbox/1792807358-human.md, 1792969892-human.md, 1792809574-human.md, 1792810743-human.md, 1792971120-human.md]}
status: live
-->
### Where our seats were hit, by map

What our players found about their own seat's ground in games against people, in their words' substance; the people
themselves are in the meta's people document (M-gecko-team).

- **Comet Catcher, the north-east seat (G1), both games.** Home is the F and G rows. The west-edge spots 20, 23, 15, 8
  and 21 were picked off all game in both, and his way in was the D1-D3 column: a turret line crept down it at about
  10:00 in the first game, Twin Guards stood at E1-F1 by 11:38 in the second. Rows 6 to 8 were the other seats' fight,
  and our body drifting there did nothing. Holding beside our own turrets at spot_20 and spot_23 traded best (1,070 of
  his for 330 of ours at 9:05).
- **Boreal Falls, F2 (the front one of our three seats in the north-east).** Wind 2 to 16: the player opened with three
  turbines before the plant. passage_1 at E2, with spot_7 at D1 down it, is the road the Blitz masses took into F2
  (6:39, 9:00, 10:07; 12 to 16 Blitzes): the seat fell with a turret there and no body standing beside it.
- **Eye of Horus, E7 (the middle of three in the south).** Wind 1 to 25, 19 at the start: turbines (43 metal) over
  solars (150). The centre column (spots 43, 42, 33, 32, 28, 24, 25) runs between both enemy seats; his nest at spot_25
  and a gun post at E5 closed it by 6:00, and the seat was raided from both sides. The wreck fields at B4-B5 and E5
  (7,000 metal by 10:00), D7 (5,400) and F6 (8,000) were the cheapest metal once the line moved.
- **All That Glitters Extended, the south-east (H8).** Wind 0 to 16, about 12 at the start, fell to 1 or 2 at 9:24 and
  emptied the energy store for a minute: turbines alone are not an energy plan here. An allied seat shares the corner
  (G8) and took the spots north of it (403, 417, 422 by 4:07), so our share ran north up the east edge (H7-H5) and to
  the wrecks at D5-C5 (2,600 metal).
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
  expected wind: about 13 on Comet Catcher (1 to 4), about 3.1 on Quicksilver (3 to 17), 2.8 on Center Command, 2.7
  on Great Divide, 3.3 on Gecko Isle and All That Glitters; so solars on Comet Catcher and turbines nearly everywhere
  else, with the swing of the wind to be covered by storage. A tidal generator is 90 over the tide: 1.2 on Center
  Command, 3.6 on Gecko Isle, 4.5 on Cape Violet, 9 on Quicksilver; it needs 20 of water, which Great Divide has
  none of.
- **Spots within reach set the opening's ceiling.** The sheet counts spots within 30, 60, 90 and 120 seconds of each
  start for the commander, a construction bot and a construction vehicle: on Comet Catcher from the west start 37
  within a minute for a construction vehicle against 21 for the commander and 19 for a construction bot (11 against
  5 within half a minute), so the vehicle plant's first constructor takes the strip at nearly twice the commander's
  pace and the bot lab's at the commander's.
- **Where a map is open, the ways are wide.** Comet Catcher's tightest way is about three quarters of the median:
  nothing funnels an army, and a force is met wherever the other side chooses. Great Divide's and Cape Violet's
  narrows (744 and 730 elmos, under half their ways' median) are where a column strings out; whoever holds the far
  mouth meets the column piecemeal.
- **Ledges are bot ground.** Bots climb what tanks cannot (54 against 27 degrees), and climbing is slower than
  descending; a bot force on a ledge above a tank route cannot be reached by the tanks and sees over them, with the
  beam lasers' full height bonus and the cannons' shortened upward reach from the fight model.
- **Sight is coarse and water is flat.** A ridge narrower than 64 elmos hides much less than it looks; a wading unit
  is seen as on the surface and a submerged one not at all without sonar. Hills of hundreds of elmos are the only
  cover, and the eye on top of one sees far.
- **Under-water spots belong to whoever brings the right constructor.** A tier-1 extractor stands on the sea floor,
  so on Cape Violet, where 26 spots lie under water, the amphibious, hover or ship constructor takes a third of the
  map that a land constructor reaches only from a shore within 130; on Quicksilver bots reach all but 6 of the 44
  and on Gecko Isle all 88, so there the sea costs little. A land army raids a flooded extractor only from the shore
  or by wading to 20 deep.
- **The enemy's start is a cluster in his box, not a point.** The guesser puts an AI seat beside the best-connected
  free spot in its box; a person chooses, usually near the same cluster. Scouting the box's metal clusters finds
  him; walking to the map's mirror point found a beach on Quicksilver.
- **Trees are a second energy store.** At 38 energy a second a constructor against a tree, four trees are the 1,000 of
  the starting store in under half a minute of one constructor's time; a map's tree line near the start (Great
  Divide's firs, Center Command's kapoks at 300) is energy with no build time.